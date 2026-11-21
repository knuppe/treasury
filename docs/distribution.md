# Distribution: how a version reaches the people who use it

What is published in each Treasury version, where each file comes from and the order of the steps. In
short: CI builds the three systems on a clean machine and generates the attestation of each file; the
maintainer checks and signs on the Mac, with the YubiKey; publishing is manual. Why it is done this way:
[trust.md](trust.md). Automatic updates (signatures, `latest.json`):
[auto-update.md](auto-update.md). Day-to-day development: [development.md](development.md).

## What goes in each release

Release `vX.Y.Z` on GitHub (`knuppe/treasury`), created as a **draft** by CI and published by
hand. All packages come from `release.yml`; the signatures, from `scripts/sign-release.sh`.

| System | File | Automatic update |
| --- | --- | --- |
| macOS (Apple Silicon) | `Treasury_X.Y.Z_aarch64.dmg` | installs the `.app`, which updates itself |
| macOS | `Treasury_X.Y.Z_aarch64.app.tar.gz` | is the update package |
| Linux (x86_64) | `Treasury_X.Y.Z_amd64.AppImage` | yes |
| Linux | `Treasury_X.Y.Z_amd64.deb` | yes (asks for the administrator password) |
| Linux | `Treasury-X.Y.Z-1.x86_64.rpm` | yes (asks for the administrator password) |
| Windows (x64) | `Treasury_X.Y.Z_x64-setup.exe` (NSIS, current user only) | yes |
| Windows | `Treasury_X.Y.Z_x64_en-US.msi` | yes |
| all | `latest.json` | the index the apps read, with both signatures of each package |
| all | `*.sig` (minisign) and `*.yubikey.sig` | signatures of each update package |
| all | `SHA256SUMS.txt`, `SHA256SUMS.txt.sig`, `allowed_signers` | hashes of everything, signed by the YubiKey |

- Each package file has a provenance attestation on GitHub (`gh attestation verify`).
- Code signing: macOS ships with an ad hoc signature and hardened runtime, without notarization;
  Windows, without code signing; Linux has no system signature. Update security does not
  depend on this (it is the YubiKey); the first install does (below).
- The names are the Tauri defaults; do not rename them, because `latest.json` points to them and the app
  only accepts packages from this repository's releases.
- Intel macOS is not supported.
- The NSIS installer follows the Windows language (Brazilian Portuguese, English or Spanish) and puts
  the app in the user's folder, without asking for an administrator. The `.msi` ships in English, for managed
  installation (the `upgradeCode` is fixed in `tauri.conf.json`).

## Before the first public release

These stay with the maintainer, because they need the physical keys or the account:

1. **Update keys.** Create the FIDO2 keys on the two YubiKeys, the new minisign key encrypted
   for them (and for the offline recovery key), replace the public one in `update-minisign.pub` and
   fill in `update-keys.pub` ([auto-update.md](auto-update.md), "Criar as chaves"). Without this,
   `release.yml` and `sign-release.sh` refuse to continue.
2. **Sign tags with the YubiKey**: `git config gpg.format ssh`, `user.signingkey` with the
   `sk-` key and `gpg.ssh.allowedSignersFile` with your keys, so the `git verify-tag` in
   `sign-release.sh` passes.
3. **Vulnerabilities**: fixed on 2026-11-13; the five with no fix available stay as an
   exception in `deny.toml`, with the reason, until the fix is out ([roadmap.md](roadmap.md)). A new
   vulnerability makes `release.yml` stop at the start.
4. **Repository**: the protections in the list below, when creating the repository.
5. **Real test**: an update from one version to the next on macOS, on an Ubuntu 24.04 VM
   (AppImage and `.deb`) and on Windows 11 (NSIS).

## Repository protections on GitHub

Turn on when creating the repository (Settings):

- Account: 2FA with a security key, no SMS; offline recovery codes; only fine-grained tokens
  with an expiry.
- Rulesets:
  - `main` with no force push or deletion, green CI and code owner review required;
  - `v*` tags created only by the maintainer, with no deletion or update;
  - signed commits and tags.
- `.github/CODEOWNERS` already lists the files that decide what the app accepts; the "code owner
  review" rule of the ruleset enforces it.
- `release` environment (Settings › Environments) with maintainer approval: the job that creates
  the draft waits for it.
- Immutable releases.
- Actions: allow only GitHub's and the ones used here; the workflows already pin them by SHA.
- Secret scanning with push protection, Dependabot alerts and security updates, and "Private
  vulnerability reporting" (`SECURITY.md` and the Code of Conduct point to it).

While the repository is private on the Free plan, GitHub does not offer rulesets, required
reviewers on an environment, approval of fork workflows, private vulnerability reporting, push
protection or build attestations. What is already on: squash merge only, branches deleted after
merge, the Actions allow list with SHA pinning required, a read-only workflow token, Dependabot
alerts and security updates, immutable releases, and the `release` environment limited to `v*`
tags. The rest is turned on the day the repository becomes public. Until then `release.yml`
stops at the first step, and no `v*` tag should be pushed.

## Step by step for a version

1. **Green main.** CI passed, no pending merge.
2. **Version.** `scripts/release.sh X.Y.Z` bumps the version and makes the commit "release: X.Y.Z" (it also
   builds on the Mac; use `--install` to install on the Mac itself).
3. **Signed tag and push.**

   ```bash
   git tag -s vX.Y.Z -m "Treasury X.Y.Z"
   ```

   ```bash
   git push origin main vX.Y.Z
   ```

4. **CI.** The tag triggers `release.yml`:
   - `preflight`: the tag is the app version, `update-keys.pub` has keys, and there is no
     known vulnerability in the dependencies (`cargo deny` and `npm audit`);
   - `build`: builds macOS, Linux and Windows in the `dist` profile, from the tag, without cache, and generates
     the attestation of each file;
   - `draft`: waits for your approval in the `release` environment and attaches everything to a draft.
5. **Sign on the Mac**, with the YubiKey:

   ```bash
   scripts/sign-release.sh vX.Y.Z --notes notas.md
   ```

   It checks the tag and the attestations, signs each package (touch and PIN for each one), builds
   `latest.json`, signs `SHA256SUMS.txt` and uploads to the draft.
6. **Check the draft**: the files in the table above, the notes, `latest.json`.
7. **Publish**: `gh release edit vX.Y.Z --draft=false --latest`. Only from here on does
   `releases/latest/download/latest.json` point to the new version.
8. **Check the update**: open a Treasury of the previous version and see that it offers and
   installs the new one.

Test version: publish as a *pre-release* (`--prerelease` in place of `--latest`). A pre-release
does not enter `releases/latest`, so no installed app updates to it.

## Checking the download (for those who install)

In the download folder, with the release's `SHA256SUMS.txt`, `SHA256SUMS.txt.sig` and `allowed_signers`
files:

```bash
ssh-keygen -Y verify -f allowed_signers -I treasury -n treasury-release -s SHA256SUMS.txt.sig < SHA256SUMS.txt
```

```bash
shasum -a 256 -c SHA256SUMS.txt --ignore-missing
```

```bash
gh attestation verify Treasury_X.Y.Z_aarch64.dmg --repo knuppe/treasury
```

On Linux, `sha256sum -c SHA256SUMS.txt --ignore-missing`. On Windows (PowerShell),
`Get-FileHash .\Treasury_X.Y.Z_x64-setup.exe` and compare with the line in `SHA256SUMS.txt`.

## No Developer ID and no Windows signature

- **macOS:** the app is not notarized. On first use, macOS blocks it ("cannot be
  verified") and the user allows it in System Settings › Privacy & Security › "Open Anyway". A
  Developer ID (with notarization) would remove the block and the Keychain access prompt on every
  version.
- **Windows:** SmartScreen shows "Windows protected your PC" until the installer has
  reputation or a signature. "More info" › "Run anyway".

## Forks

Whoever builds a fork should use `TREASURY_UPDATE_REPO=none` (the app does not look for updates) or their
own repository, and replace the public keys in `update-minisign.pub` and `update-keys.pub`. Without
this, the fork's app looks for versions in `knuppe/treasury` and refuses all of them.

## CI

- `ci.yml` (every push and PR): new dependencies against the approved list
  (`scripts/deps-check.py`), vulnerabilities (`cargo deny` with `deny.toml` and `npm audit` of the
  production dependencies), lint, formatting, types, tests, `clippy -D warnings`, E2E on Linux
  and, on `main`, test packages for the three systems as artifacts, with no automatic update.
  Read only. While the repository is private, the checks run on Linux only and the packages
  only by hand (`workflow_dispatch`), because a macOS minute counts ten times against the
  monthly Actions quota.
- `release.yml` (tag `vX.Y.Z` or by hand): the preflight, the packages with attestation and the draft. Only the
  draft job writes, after approval in the `release` environment. No signing
  secret.
- `dependabot.yml`: weekly, grouped updates of the actions, Cargo and npm, with `build(deps):`
  commit messages. An update that brings a new crate or package fails the dependency check
  until the maintainer approves it.
- `FUNDING.yml`: the repository's Sponsor button, pointing to GitHub Sponsors.
- Issue forms (bug, idea, translation) and the pull request template, in English with the
  Portuguese text next to it.
- All actions are pinned by SHA, with the version in a comment.
