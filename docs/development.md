# Development

A guide for anyone changing the code (people and coding agents): structure, commands, tests, build and conventions. The product description is in the other documents in `docs/` ([README.md](README.md)); how to contribute is in [CONTRIBUTING.md](../CONTRIBUTING.md).

## Structure

```
treasury/
├─ Cargo.toml                 Rust workspace
├─ crates/treasury-core/      core: domain, calculations, SQLCipher, history, backups, API, MCP
│  ├─ migrations/             schema SQL (refinery, embedded in the binary)
│  └─ src/
│     ├─ api/                 `call(cmd, args)` dispatcher + system commands (vault, backup, settings)
│     ├─ changes.rs           change sets (undo/redo/audit)
│     ├─ storage.rs, vault.rs SQLCipher + Argon2id
│     ├─ backup.rs            managed backups (without the downloaded public data)
│     ├─ market/, indices/    prices (Yahoo, Tesouro, CVM) and indices (BCB, FRED…)
│     ├─ research/            CVM/SEC/B3 fundamentals, indicators, ranking and screener
│     ├─ import/              OFX/CSV statements, B3 Investor Area, PDF brokerage notes
│     ├─ ai/, mcp/            agent and MCP server
│     └─ <module>/            domain: mod.rs (DTOs and logic) + api.rs (commands)
├─ crates/treasury-dev/       real core over HTTP (interface and E2E without Tauri), fast KDF
├─ apps/desktop/              Tauri app: src-tauri (Rust) + src (React/TS)
│  └─ src/api/generated/      TS types generated from the Rust DTOs (do not edit)
├─ locales/                   Fluent catalogs per language (locales/GUIDE.md)
├─ methodology/               versioned weights, thresholds and catalogs of the score and the Market
├─ scripts/                   release.sh, Linux and Windows via Docker, embedded data, hook
└─ docs/
```

## Commands

The Rust version is pinned in `rust-toolchain.toml` (rustup installs it on the first build), and the
workflows use the same one, so a lint from a newer stable release does not break CI by surprise.
Moving to a newer Rust is a change like any other: update the file and the workflows, then run
clippy and the tests.

```bash
cargo test -p treasury-core            # core tests (also regenerates the TS types)
cargo clippy --all-targets -- -D warnings
cargo fmt --all
cd apps/desktop && npm test            # interface tests
cd apps/desktop && npm run lint && npm run typecheck
```

**Memory to build.** The core is a single crate, and each build of it needs about 4 GB of
RAM (measured on macOS, at any level of parallelism). `cargo test` and `cargo clippy
--all-targets` build the library and its tests at the same time, plus the dependencies in
parallel: with 16 GB or more there is nothing to do. With 8 GB or less (a VM, Docker Desktop with
the default limit), the system kills the compiler (`signal: 9, SIGKILL`); in that case run one task
at a time:

```bash
CARGO_BUILD_JOBS=1 cargo test -p treasury-core
```

### Running the interface against the real core (without Tauri)

```bash
cargo run -p treasury-dev -- serve --home /tmp/tdev --port 7777 --token dev
cd apps/desktop && VITE_TREASURY_API=http://127.0.0.1:7777 VITE_TREASURY_TOKEN=dev npm run dev
```

`treasury-dev` uses a fast KDF and an in-memory keychain, and refuses `~/.treasury`. Without `VITE_TREASURY_API`, the interface uses the in-memory mock (`src/api/mock.ts` + `src/api/mock/*.ts`). The tax estimate mock (`mock/taxBurden.ts`) reads the same `crates/treasury-core/src/tax_burden/data/*.json` as the core (`?raw` import; `vite.config.ts` allows the folder in `server.fs.allow`).

### End-to-end tests (E2E)

```bash
cd apps/desktop && npm run e2e
```

It builds `treasury-dev`, starts the real core on a temporary vault and the interface on port 1420, and runs Playwright (`apps/desktop/e2e/`). Without the Playwright Chromium downloaded, point `PW_CHROMIUM` to a local Chromium.

The scenarios share one core and one vault and run in alphabetical order: `01-vault` creates the vault; the others use `openVault` from `e2e/helpers.ts`, which creates, unlocks or reuses the open vault. No scenario uses the network or real AI engines.

### Release app

To publish a version (GitHub release, packages for all systems, `SHA256SUMS.txt`), see
[distribution.md](distribution.md).

**macOS only by default.** `scripts/release.sh` builds and installs only the Mac app. The Linux packages (`scripts/linux/build.sh`: AppImage and `.deb`) and Windows packages (`scripts/windows/build.sh`: NSIS installer by cross-compiling with MinGW) come out through Docker, on a Mac or a Linux machine, and only run when called; on release, the CI's `release.yml` builds both ([distribution.md](distribution.md)).

```bash
scripts/release.sh                    # prunes target/, checks and builds the current version
scripts/release.sh 0.6.1 --install    # bumps the version, commits, builds and installs (macOS)
scripts/release.sh --skip-checks      # build only, to iterate fast
scripts/release.sh --dist --dmg       # public installer: full LTO and .dmg
```

The script prunes the old cache (`clean-build.py`), runs `cargo fmt --check`, `clippy`, the workspace
tests and the interface checks (prettier, tsc, eslint and vitest in parallel with
clippy), bumps the app version, records the new migrations in `migrations.lock`, makes the
commit `release: X.Y.Z (…)`, builds the `.app` (on Linux, AppImage and .deb) and, with `--install`, closes
Treasury, sends the old app to the Trash and installs the new one in `/Applications`. It refuses to run
with another `tauri build` in progress or with less than 15 GB free. Output in
`target/release/bundle/` (with `--dist`, `target/dist/bundle/`); check logs in
`target/release-logs/`.

The direct build remains valid: `cd apps/desktop && npx tauri build --bundles app`. The npm
package `@tauri-apps/api` must be on the same minor version as the `tauri` crate (the build refuses the
difference).

**Versions.** The app version lives in `apps/desktop/src-tauri/Cargo.toml` (plus
`tauri.conf.json`, `package.json`, `Brand.tsx` and `mock/settings.ts`, which the script bumps
together); the core receives it at startup (`config::app_version`). The workspace version, used
by the core and by `treasury-dev`, stays fixed: cargo puts the version in the hash of each
build, and bumping it on every release recompiled the whole core and left in `target/` a new
copy of several GB per version.

**Profiles.** `release` (what goes to `/Applications`) uses thin LTO and parallel compilation;
`dist` (`--dist`) uses full LTO and a single code unit, for public binaries. In
`dev`, the dependencies compile optimized and without debug information.

### Disk space

Cargo never deletes old builds: each version, dependency, feature or compiler
leaves a copy of the crate in `target/` (in `deps/`, `.fingerprint/`, `build/` and
`incremental/`), and on macOS each incremental build leaves one more set of objects
(`.rcgu.o`) in `deps/`; the folder went past 60 GB. `scripts/release.sh` already prunes before the
build; to prune by hand:

```bash
python3 scripts/clean-build.py --dry-run   # shows what it would delete
python3 scripts/clean-build.py
python3 scripts/clean-build.py --keep 1 --days 1 --max-gb 30   # more aggressive
```

It removes only the copies that will not be used again: those replaced by a newer build
of the same unit (the 3 most recent of each stay, because different commands unify
features in different ways), dependencies at versions that left `Cargo.lock`, what was
built by another compiler, objects from earlier builds and incremental state unused
for 5 days. It also deletes the `target/` of worktrees that no longer exist. `cargo clean` deletes
everything and forces the dependencies to be rebuilt (several minutes).

### App commands (Tauri) and ACL

The app has an ACL manifest (`apps/desktop/src-tauri/build.rs`): a new `#[tauri::command]` must go in the `AppManifest::commands` list **and** get `allow-<command-with-hyphens>` in `capabilities/default.json` (which applies only to the `main` webview); without that the interface gets "not allowed by ACL". The generated permissions live in `src-tauri/permissions/autogenerated/` (when removing a command, delete its `.toml`). Any other webview has no permissions, and links outside the app open in the system browser (`open_external`).

### MCP

- App open: `treasury mcp` (same binary) becomes a proxy to the socket `~/.treasury/run/mcp.sock` (0600 + per-session token).
- App closed: reads in read-only mode with the keychain key, if keychain unlock is on.
- Claude Code: `claude mcp add treasury -- /Applications/Treasury.app/Contents/MacOS/treasury mcp`.
- External agents: API keys in Settings › MCP integrations (scopes `read` and `write` = proposals). Over stdio, `TREASURY_API_KEY=trsk_… treasury mcp`; over local HTTP (off by default), `POST http://127.0.0.1:<port>/mcp` with `Authorization: Bearer trsk_…`. With "Require API key", clients without a key are refused.
- Development: `claude mcp add treasury-dev -e TREASURY_HOME=/tmp/tdev -- <repo>/target/debug/treasury-dev mcp` (use a short `--home`: the socket path has a limit of 104 bytes on macOS).

## Local signing on macOS (Touch ID without asking for the password)

Local builds come out with an ad hoc signature, which changes with each version; the Keychain ties access to the vault item to that signature and, with each new version, asks for the Mac password after Touch ID. To avoid this, create the local identity once with `scripts/macos-local-signing.sh` (a self-signed certificate only for code signing, key in the session Keychain; macOS asks for the password once to trust it) and build with `APPLE_SIGNING_IDENTITY="Treasury Local Signing" npx tauri build --bundles app`. On the first launch of a build signed this way, choose "Always Allow" in the Keychain prompt: later versions, with the same certificate, do not ask again.


`scripts/release.sh` re-signs the `.app` after `tauri build` with the **hardened runtime** (`codesign --options runtime`, empty entitlements in `apps/desktop/src-tauri/release.entitlements`; [local-security.md](local-security.md), item 4), with the local identity when it exists and ad hoc when not; with `--dmg`, it rebuilds the `.dmg` with the signed app. Release only: the development build and the tests do not change. On launch, the release build checks its own signature (`src-tauri/src/integrity.rs`) and warns if the bundle was altered; a bundle that was never sealed (`tauri build` without the script) is not checked. The same build hardens the process at startup (`vault::harden_process`: no core dump and, on Linux, `PR_SET_DUMPABLE = 0`); in the development build nothing changes, so the debugger works. Tests that need the Mac's hardware stay out of the suite, with `#[ignore]`: the Secure Enclave proof of concept (`cargo test -p treasury-core --lib enclave_poc -- --ignored`; the Touch ID one opens the dialog) and the check of a real bundle (`TREASURY_APP_BUNDLE=… cargo test -p treasury-desktop bundle_from_env -- --ignored`).
## Core conventions

- **API**: every command is `module.action` and takes/returns JSON. Each domain module has `api.rs` with `handle(ctx, action, args)` and `READ_ONLY` (actions without side effects; only they are available to read-only MCP clients). Arguments with `parse_args::<T>(args)`, return with `to_value(...)`.
- **DTOs**: `#[derive(Serialize, Deserialize, TS)] #[ts(export)]`. `i64` fields take `#[ts(type = "number")]` (ts-rs generates `bigint` by default). Optional input fields take `#[ts(optional)]`. In *patches*, nullable fields are `Option<Option<T>>` with `#[serde(default, deserialize_with = "crate::patch::double")]` (absent = leave alone, `null` = clear).
- **Writes**: always inside `ctx.write("motivo em português", |tx| { ... })`. This records a `change_set` with the before/after of each row (SQLite's *session* extension) and gives undo for free. Never write outside it, except in technical tables (`setting`, `backup_record`).
- **Money**: `i64` in the smallest unit + `currency`. Never `f64` for amounts; `f64` only for ratios (percentages) for display. Asset quantities/unit prices (phase 2) in `rust_decimal` stored as text.
- **Sign**: `txn.amount` is relative to the account (+ in, − out). An expense in a checking account and a card purchase are negative.
- **Cash flow**: only `kind IN ('income','expense')` counts as income/expense. Transfers, card statement payments and contributions do not.
- **Accrual × cash**: accrual uses `txn.date`; cash uses `card_statement.due_date` for card transactions (and `txn.date` for the others).
- **Split**: if a transaction has `txn_split`, reports use the parts (category/holder of each part) in place of the parent.
- **Error messages**: user-facing, in the installation's language. The text comes from the core catalog (`locales/<language>/nucleo-<area>.ftl`, with `i18n::t` and `i18n::t_args`), written in pt-BR and translated to en and es (`locales/GUIDE.md`); numbers in the arguments go as text, because Fluent would group the thousands. Use `Error::invalid`, `Error::not_found`, `Error::conflict`.
- **Dates**: `YYYY-MM-DD` (no time zone) for transaction dates; technical instants in UTC RFC 3339 (`time::now_utc()`).
- **Migrations**: new files in `crates/treasury-core/migrations/` (`V<n>__name.sql`), applied when the vault opens, always forward, after a verified backup (`pre_migration`). Rules:
  - Never change or delete a published migration: the published ones are in `crates/treasury-core/migrations.lock` with the sha256, and the test `released_migrations_are_frozen` fails if any changes.
  - The number of a new migration must be **greater than that of the last published one** (the last one is at the end of `migrations.lock`). refinery refuses versions lower than the last applied.
  - When publishing a version, add the new migrations to `migrations.lock` under the version's header.
  - **V1 is the schema up to 1.0.0.** The 94 migrations from before 1.0.0 (V1 to V164) were merged into `V1__esquema.sql`: the same schema and seed data the old chain produced (`v1_is_the_old_chain_schema` checks it against a real 0.9.38 vault). New migrations start at V2. The V-numbers above 1 cited in the docs and in code comments (V84, V134…) are those old migrations: they say when a table or column appeared.
  - A vault from 0.9.38 (history V1 to V164, `storage::LEGACY_HISTORY`) is recognized when it opens: after the `pre_migration` backup, each V1 object is checked against the vault's schema and the history becomes a single V1 row; tables and data are not touched (`a_0_9_38_vault_moves_to_the_v1_history`, with the real vault `tests/fixtures/vault-v0.9.38.db`). A vault from before 0.9.38 is refused with the instruction to open it once in 0.9.38 (`tests/fixtures/vault-v0.1.0.db`). Do not regenerate those fixtures.
  - `api::tests_upgrade` opens the 0.9.38 vault through the app (`pre_migration` backup verified before migrating, version record, everyday reads, the old backup still restorable) and refuses to restore a backup from a newer version (the current vault does not change).
  - The vault keeps the app versions that opened it in the `vault.versions` setting (`vault_versions.rs`): first, last and newest, and each change of app or schema version. When a newer version has already opened the vault, `vault.status` carries `newer_app_version` and the top bar suggests updating the app.
  - A vault written by a newer version of the app is refused with `schema_too_new` (going back a version = restoring a backup from that version).
  - To change a `CHECK` or rebuild a table referenced by foreign keys (as the old V85 did with `asset`), use SQLite's 12-step procedure (new `CREATE` → `INSERT … SELECT` → `DROP` → `ALTER … RENAME`) and add the version to `storage::FK_OFF_MIGRATIONS`: it runs with `foreign_keys = OFF` (with them on, the `DROP` would cascade-delete) and is checked with `PRAGMA foreign_key_check`. List the columns explicitly and check whether earlier migrations added columns to the table.
- **Tests**: each module has unit tests with a temporary database (`Db::create(..., KDF_TEST)`), including calculation edge cases. API tests in `api/tests.rs` exercise the flow through the dispatcher.
- **Tests are written in English**: test names (`describe`/`it`, Rust `#[test]` functions), comments and assertion messages. What the test feeds to the app or compares with it stays as the app uses it: UI text (the tests run in pt-BR), made-up data, expected values and error messages.

## Dependencies

Each dependency is third-party code running next to the financial life of whoever uses the app, and
each one is one more door for a vulnerability or a tampered package. So the rule is
to have only the necessary ones:

- **First, what already exists.** Before adding a crate or an npm package, see whether the
  standard library or something already in the tree solves it (`cargo tree -i <crate>`). Often
  it does: the updater checks the minisign with the `ring` and `blake2` that were already in the project,
  and the exchange time zones are our own table (`market/zone.rs`).
- **Make our own version** when the job is small and well defined (a file format,
  a table, a calculation), with tests that check against the reference. Cryptography primitives,
  TLS, SQLite and large parsers stay with mature libraries: rewriting them would be
  more risk, not less.
- **When one must come in**, check first: how many new crates it brings (`cargo tree -e normal`
  before and after), whether it is maintained (recent version, more than one maintainer or a well-known project), the
  license (the `deny.toml` list), RustSec or `npm audit` advisories, and whether it has a `build.rs` that
  downloads things or compiles native code. Prefer turning off the default features and turning on only the
  ones used. Write in `Cargo.toml`, in a comment beside it, what it is for.
- **Approval is the maintainer's.** `scripts/deps-check.py` runs in the CI and fails when
  `Cargo.lock` or `package-lock.json` has a name that is not in `scripts/deps-approved.txt`;
  bumping the version of something already on the list needs nothing. The list is in
  `.github/CODEOWNERS`. With approval, `python3 -I scripts/deps-check.py --update` rewrites the
  list (and also removes what left the tree).
- **Security advisories** block the CI and the release (`cargo deny`, `npm audit`). An exception in
  `deny.toml` only goes in with the maintainer's "yes", the reason beside it and an item in
  [roadmap.md](roadmap.md) to remove it.

## Personal data

Statements, payslips, simulations, PDFs and screenshots from people who use the app serve as a basis for the implementation, but **never go into the repository**, neither whole nor in pieces:

- The documents stay outside the repository, in `~/Treasury-privado/` (or in the `TREASURY_PRIVATE_DIR` folder).
- Tests and fixtures are always made up: name, employer, city, merchants, amounts, dates, units, account numbers, branch, contract and registration numbers, plan or product name. Copy the **format** of the real document, never the content. Example CPF: `123.456.789-09` or `000.000.000-00`.
- To check the code against a real document, write an `#[ignore]` test that reads the path from an environment variable (like `real_statement_from_env` in `debt/tests.rs`, with `TREASURY_LOAN_CONTRACT` and `TREASURY_LOAN_STATEMENT`).
- Commit messages and documents (`docs/`) also do not cite amounts, names, portfolio assets or details of a real document; describe the rule ("checked against a real statement"), not the content.
- The `pre-commit` hook (`.githooks/pre-commit`, enabled with `git config core.hooksPath .githooks`, only in this repository) runs `scripts/check-private.sh`: it refuses PDF, OFX, spreadsheet, image and email outside the `fixtures/` folders, CPFs with a valid check digit and the terms in `~/Treasury-privado/termos-privados.txt`. The term list stays outside the repository so it does not become a leak itself.

## Commits

Commit messages are in English. The first line is a short summary in the imperative with the module prefix (`core:`, `ui:`, `mcp:`, `docs:`, `build:`), up to about 72 characters. Then, if needed, a blank line and at most two short paragraphs explaining what changed and why. Example:

```
core: reject a loan prepayment larger than the balance

The simulator accepted any amount and showed a negative balance.
It now returns an invalid-argument error and the interface shows it.
```

Pull requests from outside follow [CONTRIBUTING.md](../CONTRIBUTING.md): each commit carries the DCO `Signed-off-by` line (`git commit -s`), without a CLA, and the contribution is distributed under the AGPL-3.0-only.
