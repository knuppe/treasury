#!/usr/bin/env python3
"""Poda o que ficou velho nas pastas de compilação, sem jogar fora o cache que ainda vale.

O cargo nunca apaga compilações antigas: cada mudança de versão, de dependência, de recurso
ou do compilador deixa uma cópia nova do crate (em `deps/`, `.fingerprint/`, `build/` e
`incremental/`) e a velha fica lá para sempre. Este script apaga só o que não volta a ser
usado; o que for apagado por engano é recompilado na próxima vez.

Uso:
    python3 scripts/clean-build.py              # poda target/ e os target/ dos worktrees
    python3 scripts/clean-build.py --dry-run    # só mostra o que apagaria
    python3 scripts/clean-build.py --keep 1 --days 1 --max-gb 30   # mais agressivo

Regras, por perfil (debug, release, dist):
- Cada unidade de compilação (pacote + alvo + recursos + perfil) é um "papel". Do mesmo
  papel ficam as `--keep` cópias mais recentes (padrão 3: comandos diferentes, como
  `cargo test` e `cargo build -p treasury-dev`, unificam recursos das dependências de jeitos
  diferentes e usam cópias diferentes); as mais antigas foram substituídas e saem.
- Dependências cuja versão não está mais no Cargo.lock saem, e também o que foi compilado
  por outra versão do compilador.
- `deps/*.rcgu.o`: os objetos de compilações anteriores do mesmo crate (no macOS, o rustc
  deixa os de cada compilação incremental) saem; ficam os da última.
- `incremental/`: por crate, fica a pasta mais recente e as usadas nos últimos `--days` dias
  (padrão 5).
- Se a pasta continuar acima de `--max-gb` (padrão 80), apaga `debug/incremental` inteira:
  o próximo build recompila só os crates do workspace.
- Pastas `target` de worktrees que não existem mais no git são apagadas inteiras.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import time
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# lib<nome>-<hash>.<ext>, <nome>-<hash>, <nome>-<hash>.<cgu>.<sessão>.rcgu.o, <nome>-<hash>.d …
ARTIFACT = re.compile(r"^(?:lib)?(?P<name>.+?)-(?P<hash>[0-9a-f]{16})(?:\..*)?$")
OBJECT = re.compile(r"^(?P<stem>.+-[0-9a-f]{16})\.[^.]+\.(?P<session>[^.]+)\.rcgu\.o$")
# .fingerprint/<pacote>-<hash> e build/<pacote>-<hash>
UNIT_DIR = re.compile(r"^(?P<pkg>.+)-(?P<hash>[0-9a-f]{16})$")
# incremental/<crate>-<id base36>
INCREMENTAL = re.compile(r"^(?P<name>.+)-(?P<id>[0-9a-z]{13})$")
REGISTRY_SRC = re.compile(r"/registry/src/[^/]+/(?P<dir>[^/]+)/")


def size_of(path: Path) -> int:
    if path.is_symlink() or path.is_file():
        try:
            return path.lstat().st_size
        except OSError:
            return 0
    total = 0
    for p in path.rglob("*"):
        try:
            if not p.is_symlink() and p.is_file():
                total += p.lstat().st_size
        except OSError:
            pass
    return total


def newest_mtime(path: Path) -> float:
    try:
        best = path.lstat().st_mtime
    except OSError:
        return 0.0
    if path.is_dir() and not path.is_symlink():
        for p in path.iterdir():
            try:
                best = max(best, p.lstat().st_mtime)
            except OSError:
                pass
    return best


def gb(n: int) -> str:
    return f"{n / 1024**3:.1f} GB"


def locked_packages(workspace: Path) -> set[str]:
    """`<nome>-<versão>` de cada pacote do Cargo.lock (o nome da pasta no registro)."""
    lock = workspace / "Cargo.lock"
    if not lock.is_file():
        return set()
    out, name = set(), None
    for line in lock.read_text().splitlines():
        if line.startswith("name = "):
            name = line.split('"')[1]
        elif line.startswith("version = ") and name:
            version = line.split('"')[1]
            out.add(f"{name}-{version}")
            name = None
    return out


class Unit:
    """Uma cópia compilada: `.fingerprint/<pacote>-<hash>` e o que tem o mesmo hash."""

    def __init__(self, fp_dir: Path, pkg: str, hash_: str):
        self.fp_dir = fp_dir
        self.pkg = pkg
        self.hash = hash_
        self.mtime = newest_mtime(fp_dir)
        self.role: tuple = (pkg, "?")
        self.rustc: int | None = None
        for j in sorted(fp_dir.glob("*.json")):
            try:
                d = json.loads(j.read_text())
            except (OSError, ValueError):
                continue
            self.role = (
                pkg,
                j.stem,
                d.get("features"),
                d.get("profile"),
                d.get("target"),
                d.get("compile_kind"),
                d.get("config"),
            )
            self.rustc = d.get("rustc")
            break


class Pruner:
    def __init__(self, dry_run: bool):
        self.dry_run = dry_run
        self.freed = 0
        self.removed = 0

    def remove(self, path: Path) -> None:
        size = size_of(path)
        self.freed += size
        self.removed += 1
        if self.dry_run:
            return
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path, ignore_errors=True)
        else:
            try:
                path.unlink()
            except FileNotFoundError:
                pass

    def prune_profile(self, profile: Path, locked: set[str], keep: int) -> None:
        fp_root, deps, build = profile / ".fingerprint", profile / "deps", profile / "build"
        if not fp_root.is_dir():
            return
        by_hash: dict[str, list[Path]] = defaultdict(list)
        for d in (deps, build):
            if d.is_dir():
                for entry in d.iterdir():
                    m = ARTIFACT.match(entry.name)
                    if m:
                        by_hash[m["hash"]].append(entry)

        units = []
        for entry in fp_root.iterdir():
            m = UNIT_DIR.match(entry.name)
            if m and entry.is_dir():
                units.append(Unit(entry, m["pkg"], m["hash"]))
        if not units:
            return

        # O compilador atual é o da unidade compilada por último.
        known = [u for u in units if u.rustc is not None]
        current_rustc = max(known, key=lambda u: u.mtime).rustc if known else None
        dead: set[str] = set()
        for u in units:
            if u.rustc is not None and u.rustc != current_rustc:
                dead.add(u.hash)
                continue
            # Dependência do registro numa versão que saiu do Cargo.lock.
            if locked:
                for p in by_hash.get(u.hash, []):
                    if p.suffix == ".d" and p.parent == deps:
                        with p.open(errors="replace") as f:
                            m = REGISTRY_SRC.search(f.read(4096))
                        if m and m["dir"].startswith(u.pkg + "-") and m["dir"] not in locked:
                            dead.add(u.hash)
                        break

        roles: dict[tuple, list[Unit]] = defaultdict(list)
        for u in units:
            if u.hash not in dead:
                roles[u.role].append(u)
        for same in roles.values():
            same.sort(key=lambda u: u.mtime, reverse=True)
            dead.update(u.hash for u in same[keep:])

        for u in units:
            if u.hash in dead:
                self.remove(u.fp_dir)
                for p in by_hash.pop(u.hash, []):
                    self.remove(p)

        # Objetos de compilações anteriores do mesmo crate.
        if deps.is_dir():
            sessions: dict[str, dict[str, list[Path]]] = defaultdict(lambda: defaultdict(list))
            for entry in deps.iterdir():
                m = OBJECT.match(entry.name)
                if m:
                    sessions[m["stem"]][m["session"]].append(entry)
            for by_session in sessions.values():
                if len(by_session) < 2:
                    continue
                ranked = sorted(
                    by_session.values(),
                    key=lambda objs: max(o.lstat().st_mtime for o in objs),
                    reverse=True,
                )
                # Só se a última compilação deixou o conjunto completo de objetos.
                if len(ranked[0]) < max(len(objs) for objs in ranked):
                    continue
                for objs in ranked[1:]:
                    for o in objs:
                        self.remove(o)

    def prune_incremental(self, profile: Path, days: float) -> None:
        d = profile / "incremental"
        if not d.is_dir():
            return
        cutoff = time.time() - days * 86400
        by_name: dict[str, list[tuple[float, Path]]] = defaultdict(list)
        for entry in d.iterdir():
            m = INCREMENTAL.match(entry.name)
            if m:
                by_name[m["name"]].append((newest_mtime(entry), entry))
        for dirs in by_name.values():
            dirs.sort(reverse=True)
            for i, (mtime, path) in enumerate(dirs):
                if i > 0 and mtime < cutoff:
                    self.remove(path)


def worktrees() -> tuple[list[Path], list[Path]]:
    """(worktrees vivos em .claude/worktrees, pastas de worktrees que o git não conhece mais)."""
    base = ROOT / ".claude" / "worktrees"
    if not base.is_dir():
        return [], []
    try:
        out = subprocess.run(
            ["git", "-C", str(ROOT), "worktree", "list", "--porcelain"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError):
        return [], []
    live = {
        Path(line.split(" ", 1)[1]).resolve()
        for line in out.splitlines()
        if line.startswith("worktree ")
    }
    alive, stale = [], []
    for wt in base.iterdir():
        if wt.is_dir() and (wt / "target").is_dir():
            (alive if wt.resolve() in live else stale).append(wt)
    return alive, stale


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--dry-run", action="store_true", help="só mostra o que apagaria")
    ap.add_argument("--keep", type=int, default=3, help="cópias recentes que ficam por papel")
    ap.add_argument("--days", type=float, default=5, help="incremental sem uso há mais dias sai")
    ap.add_argument("--max-gb", type=float, default=80, help="acima disso, apaga debug/incremental")
    a = ap.parse_args()

    pr = Pruner(a.dry_run)
    alive, stale = worktrees()
    for wt in stale:
        print(f"worktree removido: {wt / 'target'} ({gb(size_of(wt / 'target'))})")
        pr.remove(wt / "target")

    for workspace in [ROOT, *alive]:
        target = workspace / "target"
        if not target.is_dir():
            continue
        locked = locked_packages(workspace)
        before = size_of(target)
        freed = pr.freed
        for name in ("debug", "release", "dist"):
            profile = target / name
            if profile.is_dir():
                pr.prune_profile(profile, locked, max(a.keep, 1))
                pr.prune_incremental(profile, a.days)
        after = before - (pr.freed - freed)
        if after > a.max_gb * 1024**3 and (target / "debug" / "incremental").is_dir():
            print(f"{target}: ainda {gb(after)}; apagando debug/incremental")
            pr.remove(target / "debug" / "incremental")
            after = before - (pr.freed - freed)
        print(f"{target}: {gb(before)} → {gb(after)}")

    verb = "apagaria" if a.dry_run else "apagou"
    print(f"{verb} {pr.removed} itens, {gb(pr.freed)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
