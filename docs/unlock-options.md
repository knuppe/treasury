# Unlocking and recovery

The ways to open the vault day to day and to recover it when the password is gone. The app is
local-first: there is no server that could "reset the password". Each method is one more path to the
same key, so the vault is only as secure as the weakest method the person turns on; that is why all
of them are optional. They are in Settings › Security.

## Methods

- **Master password**
  - **Use:** always
  - **How it works:** Argon2id → SQLCipher key
  - **Effect on security:** The base of everything.
- **Touch ID** (macOS) or system password (Linux: Secret Service)
  - **Use:** day to day
  - **How it works:** The key stays in the system Keychain, with user presence required; it also serves `treasury mcp` with the app closed
  - **Effect on security:** Same as the computer's login; stealing the database file is not enough.
- **PIN** (for this computer)
  - **Use:** day to day, without biometrics
  - **How it works:** The stable recovery key `R` stays in the Keychain encrypted with Argon2id(PIN), never in the database file; five wrong PINs erase the item
  - **Effect on security:** Weaker than the password, but tied to the computer; whoever copies the item still has to guess the PIN against Argon2id (64 MiB per attempt).
- **FIDO2 hardware key** (with hmac-secret)
  - **Use:** recovery
  - **How it works:** Each key has a slot with `R` encrypted by the hmac-secret output
  - **Effect on security:** Stronger than the password; losing the hardware key does not lock the vault if there is another method.
- **Split recovery** (Shamir 2 of 3)
  - **Use:** recovery
  - **How it works:** `R` split into three parts (paper at home, spouse, bank safe-deposit box); any two recover it
  - **Effect on security:** One part alone reveals nothing.
- **Recovery kit**
  - **Use:** recovery
  - **How it works:** The database key itself in base32, printed
  - **Effect on security:** Whoever has the kit has the vault; keep it the way you keep the password.

## The stable recovery key

The database key changes together with the master password (`PRAGMA rekey`), but the PIN, the
hardware key and the parts must remain valid after the change. That is why `R` exists:

- `R` (32 random bytes) lives **inside** the database, as an encrypted secret (`vault:recovery_root`),
  and is re-encrypted when the password changes.
- Next to the database, the `treasury.db.keys` file holds the current database key encrypted with `R`
  (XChaCha20-Poly1305) and the FIDO2 slots. It is rewritten on every unlock and is not secret on its
  own: without `R` it reveals nothing.
- Without the `.keys` file, neither the parts nor the hardware key open the vault; the recovery kit
  keeps working, because it is the database key itself.

## What the app does not offer

- Recovery by e-mail, SMS or server: it would require a service that stored the key and would break
  local-first.
- Database key in the cloud without its own encryption, or opening with no factor at all.
- Unlocking with another family member's password (D6 in [decisions.md](decisions.md)).

## In the code

- `crates/treasury-core/src/unlock/` (`mod.rs`: `R` and the `.keys` file; `pin.rs`; `shamir.rs`),
  `crates/treasury-core/src/keychain.rs`.
- Interface: `apps/desktop/src/features/settings/UnlockMethods.tsx`. Passwords, PIN and parts live
  only in the component state and disappear when the dialog closes.
- Restoring from a backup when the vault does not open: [backup-recovery.md](backup-recovery.md).
- Protection against programs on the same machine: [local-security.md](local-security.md).
