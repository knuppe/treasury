# Who publishes Treasury, and why you can check it

Treasury holds a family's financial life. Whoever installs it needs to trust that the version
that arrives through the automatic update is the one the project published, and not a version
altered by someone who broke into an account, a server or a computer. This document says,
plainly, who can publish a version, how that is guaranteed, what you can check and what still
depends on trust.

## The rule

**Only the project maintainer, Knuppe, publishes Treasury versions.** No other person, no
service and no automation can make the installed app accept a new version.

The code is open, and anyone can read it, build it and propose changes. Publishing is something
else: it is one person's word saying "this is the Treasury version". That is why it is tied to
something only that person holds.

## Contributions

Anyone can propose changes, but what goes into the code and what becomes a version is Knuppe's
decision:

- a contribution only reaches users after it is reviewed, accepted and included in a version
  built and signed by Knuppe;
- a contribution can be declined, without discussion, when it does not fit the project's vision
  ([vision.md](vision.md));
- the rules for contributors are in [CONTRIBUTING.md](../CONTRIBUTING.md).

## How this is guaranteed

1. **Two signatures on every update package.** The app only installs a package that has:
   - the **minisign** signature of the project key, tied to the announced version (an old
     version does not pass as a new one);
   - the signature of a maintainer **hardware key**: the one in the Secure Enclave of their Mac,
     which requires a fingerprint on each signature, or the one on a YubiKey (FIDO2), which
     requires a touch on the device and the PIN. The private key is born inside the chip and does
     not leave it; there is no copy in a file, on a server or on GitHub. With the Mac key, the
     fingerprint requirement is enforced by the chip and does not show up in the signature: the
     app trusts the declared key, and the release scripts check the protection before signing
     ([auto-update.md](auto-update.md), "Chaves de hardware").
   The accepted public keys are compiled into the app
   (`apps/desktop/src-tauri/update-keys.pub` and `update-minisign.pub`).
2. **Without the hardware key, nothing passes.** An app with no embedded hardware keys refuses
   every update, and the release scripts refuse to publish a version without them. The minisign
   key is also encrypted for the YubiKey and only decrypted in memory, at the moment of signing.
3. **CI builds, but does not sign.** The macOS, Linux and Windows packages are built on GitHub
   Actions, on a clean machine, from a tag signed by the maintainer. Each file comes out with a
   public **provenance attestation** (Sigstore): the proof of which repository, workflow and
   commit generated it. No signing key sits on GitHub.
4. **Signing happens on the maintainer's computer, after checking.** The
   `scripts/sign-release.sh`:
   - checks the tag signature and the attestation of each file;
   - refuses any file that did not come out of the official workflow;
   - only then signs, with the YubiKey in hand.
5. **Publishing is a manual step.** The release stays a draft until the maintainer checks and
   publishes it. Once published, it is immutable: the files cannot be swapped afterward; a fix
   becomes a new version.
6. **The app only downloads from the project's own repository releases**, over HTTPS, and only
   installs with your confirmation. The update sends nothing about you or your vault: no
   identifier, no telemetry.

## What this protects, and what it does not

It protects against:

- someone who breaks into the project's GitHub account: they can delete or swap files, but
  cannot sign, and the app refuses what is not signed;
- a server or a network along the way altering the download;
- a compromised dependency or CI automation producing a different binary: the attestation and
  the check on the maintainer's computer exist to catch that before signing;
- the leak of the minisign key alone: without the YubiKey it is not enough.

It does not protect against:

- **the maintainer themselves** deciding to publish something bad. That is the trust you place in
  a person. The open code, the public attestations and the repository history exist so that
  anyone can check what changed between one version and another;
- the maintainer's computer being compromised **and** the YubiKey they use at the same time.
  Checking the attestations reduces this risk, but does not eliminate it;
- the first installation, which does not yet have a Treasury to check the signatures. For it,
  check the download (below). macOS is not yet notarized by Apple and the Windows installer does
  not yet have a code signature: the system will warn you on first opening.

## How you check

On the release page:

- **`SHA256SUMS.txt`** lists the hash of each file, and **`SHA256SUMS.txt.sig`** is its signature
  by one of the hardware keys (the Mac's Secure Enclave or the YubiKey). The `allowed_signers`
  file carries the public keys; compare them with the repository's `update-keys.pub` before
  trusting it.
- The **attestation** of each file lives on GitHub itself.

```bash
ssh-keygen -Y verify -f allowed_signers -I treasury -n treasury-release -s SHA256SUMS.txt.sig < SHA256SUMS.txt
```

```bash
shasum -a 256 -c SHA256SUMS.txt --ignore-missing
```

```bash
gh attestation verify Treasury_X.Y.Z_aarch64.dmg --repo knuppe/treasury
```

On Linux, `sha256sum -c` instead of `shasum -a 256 -c`; on Windows (PowerShell), `Get-FileHash`
and compare with the line in `SHA256SUMS.txt`.

## If the keys are lost

The app accepts two keys, the one on the maintainer's Mac and the one on a YubiKey kept
elsewhere, and either one signs. There is also a recovery key, kept outside any computer. To
replace a key, first a version comes out that accepts the new and the old one; then one that
only accepts the new one. If all the keys are lost, the installed apps stop accepting updates
and the new version will have to be installed by hand, checking the download as above. Any key
change is announced in the release notes and in this repository.

## For those who make a fork

The code is yours to use under the AGPL-3.0, but a fork's versions are not Treasury versions. A
fork needs to build with `TREASURY_UPDATE_REPO=none` (no update) or with its own repository, and
replace the public keys with its own. Without that, the fork's app looks for versions in the
Treasury repository and refuses all of them.

## In the code

- Update: `apps/desktop/src-tauri/src/updater/` and `update_sig.rs`, with the public keys in
  `update-minisign.pub` and `update-keys.pub`
  ([auto-update.md](auto-update.md)).
- Release: `.github/workflows/release.yml`, `scripts/sign-release.sh`
  ([distribution.md](distribution.md)).
- The files that decide what the app accepts are in `.github/CODEOWNERS`: no change to them goes
  in without the maintainer's approval.

## Status

- The final hardware keys and the minisign key are already in the app. No public version has been
  published yet.
- The dependencies go through checks for known vulnerabilities (RustSec and npm) in CI and before
  each release; a known vulnerability blocks the release until it is fixed.
- Apple notarization and Windows code signing: not yet ([roadmap.md](roadmap.md)).
