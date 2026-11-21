# Automatic update

How the installed Treasury receives new versions through GitHub releases and why it is safe. The
update covers all packages (the macOS `.app`, the AppImage, the Linux `.deb` and `.rpm`, the NSIS
installer and the Windows `.msi`), downloads on its own by default, can only notify or be turned
off in Settings › Updates, checks two signatures and only installs with the person's
confirmation. Who can
publish and why: [trust.md](trust.md). The step by step of a version:
[distribution.md](distribution.md).

## How it works

1. **Where to look.** The release app looks at
   `https://github.com/knuppe/treasury/releases/latest/download/latest.json`. The address goes
   into the build: `scripts/release.sh` reads `TREASURY_UPDATE_REPO` (default `knuppe/treasury`;
   `none` turns it off) and passes `TREASURY_UPDATE_ENDPOINT` to the build, which `updater/` reads
   with `option_env!`; `release.yml` passes the same address to the CI packages. Without a published
   release (GitHub answers 404), the check only says there is no new version. In the
   development build the update is off.
2. **Which package.** The app looks in `latest.json` for the entry of the installed package,
   `<system>-<architecture>-<package>`, and, without it, `<system>-<architecture>`:

   | Installed package | Entry | What the app downloads and how it installs |
   |---|---|---|
   | macOS (`.app`, also the one from the `.dmg`) | `darwin-aarch64-app` | `.app.tar.gz`; replaces the bundle |
   | AppImage | `linux-x86_64-appimage` | the new AppImage; replaces the file in place |
   | `.deb` | `linux-x86_64-deb` | the `.deb`; `dpkg -i`, with the system administrator password |
   | `.rpm` | `linux-x86_64-rpm` | the `.rpm`; `rpm -U`, with the system administrator password |
   | Windows NSIS | `windows-x86_64-nsis` | the new installer, in passive mode |
   | Windows `.msi` | `windows-x86_64-msi` | the `.msi`, through `msiexec` |
3. **When, and what the app does on its own.** Settings › Updates has three modes, stored in
   `config.toml` (`update_mode`, outside the vault, because the check runs with the vault
   locked):
   - **Download and notify** (default): one minute after opening and every 6 hours, it checks,
     downloads, verifies both signatures and tells you the version is ready to install.
   - **Notify only**: same schedule, but it only tells you a new version exists (once per
     version). Nothing is downloaded until the person clicks "Download and verify", which helps
     on a metered connection.
   - **Off**: no check on its own.

   "Check now" works in every mode (in "Notify only" it only checks). The old on/off setting
   (`auto_update = false`) counts as "Off" until the mode is saved.
4. **What goes along.** Only the request for `latest.json` and, when there is a new version, the one for the package. The
   user agent is just `Treasury`, with no version or identifier of the app, the person or the
   vault. Https only, with a size limit (1 MB for `latest.json`, 1 GB for the package). No
   telemetry.
5. **Integrity, two signatures.**
   - **minisign** (Ed25519 over the package's BLAKE2b-512, `updater/minisign.rs`), checked with
     the public key in `apps/desktop/src-tauri/update-minisign.pub`. The version announced in
     `latest.json` must be newer than the installed one and equal to the one recorded in the signature's
     trusted comment (`--app-version`), which the global signature covers: nobody can swap
     `latest.json` to push an old, legitimate version with a known flaw.
   - **Hardware key** (`update_sig.rs`): an SSH signature (`ssh-keygen -Y sign`, SSHSIG format)
     from a key that only exists inside a device of the maintainer: a YubiKey (FIDO2,
     `sk-ssh-ed25519` or `sk-ecdsa-sha2-nistp256`), with touch and PIN at each signature (the app refuses a signature without both flags). The accepted public keys are in
     `apps/desktop/src-tauri/update-keys.pub`, one per YubiKey; the key of any one is valid. The
     app refuses the package without this signature, even with the right minisign, and **fails closed**:
     an app without embedded YubiKey keys refuses every update. It goes in `latest.json`, in `platforms.<platform>.yubikey_signature`. The signature's
     namespace carries the version (`treasury-update@X.Y.Z`): someone with only the minisign key
     cannot republish an old package, with its YubiKey signature, as a new version.
     An invalid `update-keys.pub` refuses the update instead of dispensing with the YubiKey.
   - The package is only downloaded from the releases of the same repository as `latest.json` (https,
     `github.com/knuppe/treasury/releases/download/…`); any other address is refused before the
     download.
   - GitHub's HTTPS is an extra layer, not the main one.
6. **Install.** Once downloaded and checked, the version appears in a notice ("Version X ready to
   install") and in Settings › Updates, with the release notes. Installing always waits for the
   person to confirm: the app saves and closes the vault (backup on close), replaces the bundle and opens
   again, asking for the unlock.

## Publishing a version

`release.yml` builds the packages in the CI, with provenance attestation, and
`scripts/sign-release.sh vX.Y.Z` signs on the maintainer's Mac: it checks the tag and the attestations,
signs each update package with the YubiKey and with minisign, assembles `latest.json` with all
the entries in the table above and signs `SHA256SUMS.txt`. It asks for the YubiKey once to decrypt the
minisign key and, afterward, touch and PIN for each package. Step by step in
[distribution.md](distribution.md).

To test the update on your own machine, without publishing: `scripts/release.sh X.Y.Z --update`
generates the signed macOS package and a `latest.json` in `target/release/bundle/update/`.

Tests check that the package signed by the script opens with the embedded public key, that the version
goes in the signed comment, that one extra byte in the package makes the signature fail, and that a
FIDO2 signature in the app's format (built with a test key of each type, Ed25519 and ECDSA
P-256) is accepted by OpenSSH's `ssh-keygen -Y verify`, while the app refuses a modified package,
an unknown key, another namespace, a signature without touch or without PIN, an ordinary key (in a file) and
a signature of one type with a key of another, and that the exception for the Mac's key applies only to the line
declared. With the real keys, the ignored test `real_yubikey_signatures_are_accepted`
checks signatures made with them (`TREASURY_SSHSIG_REAL`).

## Hardware keys

Tauri's minisign does not sign with hardware, so the hardware comes in two ways:

1. **A separate signature from a hardware key (the strong protection).** Two keys are valid, and
   either one signs:
   - **the Secure Enclave of the maintainer's Mac**, for day-to-day use. macOS creates the key
     inside the chip, non-exportable, and requires the fingerprint on each signature. In the signature it
     appears as `sk-ecdsa-sha2-nistp256`, with the presence flag and **without** the user-verified
     flag; a key created without the fingerprint requirement signs the same way (checked). What
     requires the fingerprint is the chip, by the way the key was created, and the app cannot see
     that. So its line in `update-keys.pub` starts with `secure-enclave` (only for
     that line does the app accept the signature without the verification flag), and
     `scripts/update-signing.sh` checks on macOS, before each signature, that the key is
     `p-256-ne` and `bio`;
   - **a YubiKey** (FIDO2), kept away from the computer, as a backup. The key never leaves
     the YubiKey, and the app requires the touch and PIN flags in the signature. It needs firmware
     5.2.3 or newer: OpenSSH does not create a key with a mandatory PIN on a YubiKey without
     "credProtect" ("requested feature not supported"; `ykman info` shows the firmware).

   Losing one of them does not block updates: the other signs the version that registers the
   replacement.
2. **Minisign key encrypted to the hardware (protection at rest).** The key file is kept
   encrypted with `age` for the Mac's Secure Enclave (`age-plugin-se`, opens with the fingerprint)
   and for the YubiKey (`age-plugin-yubikey`, which uses the series 5 PIV, with PIN and touch), and is only
   decrypted in memory at signing time. Any of the identities opens it; the scripts
   accept any `age` identity.

The minisign key never goes into git. release.sh passes it only through the process environment, never
as an `npx` argument (npm writes arguments to `~/.npm/_logs`). The public key in
`update-minisign.pub` and those in `update-keys.pub` are already the final ones, created by the step by step
below. After the first public release, losing the keys means installed apps no longer accept updates (a version
with the new keys would have to be installed by hand); hence two hardware keys.

### Creating the keys (once, before the first release)

1. Install the tools (OpenSSH with FIDO2 support and `age`):
   `brew install openssh age age-plugin-yubikey`
2. Create the Mac's signing key, in the Secure Enclave, non-exportable (`-ne`) and with the fingerprint
   required (`-t bio`), and generate the files that Apple's `ssh-keygen` uses (they only point to the
   key):
   ```
   sc_auth create-ctk-identity -l treasury-update -k p-256-ne -t bio
   cd <pasta privada>/updater && /usr/bin/ssh-keygen -w /usr/lib/ssh-keychain.dylib -K -N ""
   ```
   The line of `id_ecdsa_sk_rk_treasury-update.pub` goes into `update-keys.pub` with
   `secure-enclave ` in front. `sc_auth list-ctk-identities` shows the protection (`bio`) and the
   identity's validity (one year): renew before then, creating another key and registering it through an
   update. Only the maintainer's fingerprints should be registered on the Mac.
3. Create the FIDO2 signing key on the backup YubiKey (it asks for the FIDO2 PIN; if there is
   none yet, `ykman fido access change-pin` creates it):
   `ssh-keygen -t ed25519-sk -O resident -O verify-required -O application=ssh:treasury-update -C "treasury-update yubikey-1" -f <pasta privada>/updater/yubikey-1`
   The content of the `.pub` (it is public) goes into `update-keys.pub`, with no word in front.
4. Create the `age` identities that open the minisign key and put them together in
   `<pasta privada>/updater/age-identities.txt`, the Mac's first (`age` tries in order). The
   recipients (`age1se1…`, `age1yubikey1…`) are public.
   - On the Mac, through the Secure Enclave (`brew install age-plugin-se`): creating does not ask for the fingerprint, opening
     does. `age-plugin-se keygen --access-control=any-biometry -o <pasta privada>/updater/mac-age-identity.txt`
   - On each YubiKey, through PIV: `age-plugin-yubikey` (interactive). On a YubiKey with firmware 5.7
     or newer, before it: `ykman piv access change-management-key -a TDES --protect` (the
     factory management key of these is AES, which the plugin refuses after the PIN has already been
     changed). The PIV PIN is different from the FIDO2 one, and has 6 to 8 characters.
5. Replace the minisign key with a new one, already encrypted to these identities, and delete the current one:
   ```
   cd <pasta privada>/updater
   ~/repos/treasury/apps/desktop/node_modules/.bin/tauri signer generate --ci -p "" -w nova.key
   age -r age1se1… -r age1yubikey1… -o treasury-updater.key.age nova.key
   age -d -i age-identities.txt treasury-updater.key.age | cmp - nova.key   # confere antes de apagar
   rm nova.key treasury-updater.key
   mv nova.key.pub treasury-updater.key.pub
   ```
   and replace the content of `apps/desktop/src-tauri/update-minisign.pub` with that of the new `.pub`
   (the file's text, decoded from the base64 that `tauri signer generate` writes:
   `base64 -d nova.key.pub`).
6. Keep the YubiKey away from the computer. To sign with it instead of the Mac:
   `TREASURY_UPDATE_SK_KEY=<pasta privada>/updater/yubikey-1 scripts/sign-release.sh vX.Y.Z`.

### Renewing the Mac's key

The identity that `sc_auth` creates is valid for one year, and the command has no term option. The
"Valid To" column of `sc_auth list-ctk-identities` shows the date. It was not checked whether macOS stops
signing with the expired identity; the scripts treat it as if it does: they warn when less than
60 days remain and, with it expired, refuse and tell you to sign with the YubiKey.

Renewing means creating a new key and registering it through an update, while the current one still signs:

1. Create the new key with another label and generate its files:
   ```
   sc_auth create-ctk-identity -l treasury-update-AAAA -k p-256-ne -t bio
   cd <pasta privada>/updater && /usr/bin/ssh-keygen -w /usr/lib/ssh-keychain.dylib -K -N ""
   ```
2. Add the new `.pub` line to `update-keys.pub`, with `secure-enclave ` in front,
   **without removing the old one**, and publish a version signed by the current key (or by the YubiKey).
3. In the following versions, sign with the new one (`TREASURY_UPDATE_SK_KEY=<pasta
   privada>/updater/id_ecdsa_sk_rk_treasury-update-AAAA`).
4. Remove the old line from `update-keys.pub` after a few versions, when those who use the app have
   already gone through the version of step 2, and delete the old identity
   (`sc_auth delete-ctk-identity -h <hash>`).

Anyone who stays on a version before step 2 for longer than that keeps updating
as long as there is a key in common: that is why the YubiKey never leaves the file.

## Decisions

- **Own updater** (`apps/desktop/src-tauri/src/updater/`), in place of the Tauri plugin
  used up to 0.9.38: the plugin brought about 20 crates, among them an old `zip` with security
  warnings, for a small job. Ours reads `latest.json`, downloads over https (the core's `ureq`),
  checks the minisign with `ring` and `blake2`, which were already in the project, and installs
  each package: on macOS it extracts with the system `tar` and replaces the `.app`; for the AppImage it writes alongside
  and renames over it; for `.deb` and `.rpm` it uses `pkexec`; on Windows it opens the installer in passive
  mode. The `latest.json` format and the signatures are the same as before, so a version
  with the plugin updates to one without it. The app's commands stay the same
  (`update_status`, `update_check`, `update_install`).
- **Setting in `config.toml`, not in the vault:** the check runs with the vault closed. It is a setting of the
  installation, like the language.
- **Downloads on its own, installs only with confirmation:** the person is never surprised by the app
  restarting and the vault closing.
- **All packages update:** the AppImage, the `.deb`, the `.rpm`, the NSIS and the `.msi`, with one
  entry per package in `latest.json`. The `.deb` and `.rpm` ask for the system administrator password
  on install (`pkexec`), like any system package; without `pkexec`, or with the
  password refused, the checked package stays in a folder and the app says where, to install by hand.
- **CI builds, YubiKey signs:** no signing key stays on GitHub
  ([trust.md](trust.md)).
- **Address fixed at build time**, not in a setting: a person or a program cannot point the app
  to another server. Even so, another server could not install anything without the keys.
- **YubiKey by SSHSIG/FIDO2**, not PIV or OpenPGP: FIDO2 exists on every current YubiKey, OpenSSH's
  `ssh-keygen` already signs, and checking in the app is Ed25519 with `ring` and `sha2`, which were already
  in the project. OpenPGP would require parsing OpenPGP packets in the app; Ed25519 on PIV only
  exists from firmware 5.7.

- **Three modes instead of on/off**: "Notify only" serves whoever does not want a download in
  the background (metered connection, or wanting to read the notes first) without giving up the
  security fixes, which "Off" delays. Installing never happens on its own in any mode.
- **Own section in Settings**: updates used to be a card in About and support, where people did
  not look for them.

## Limitations

- The Linux and Windows update has not yet been tested on a real installation.
- The keys (YubiKeys and the new minisign) do not exist yet: until then, no update is accepted.
- The release notes come from `--notes` of `sign-release.sh`, written by hand for each version.
