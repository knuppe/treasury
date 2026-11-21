# Local security: malicious programs on the same machine

What Treasury does against a malicious program running on the same computer, with the same user,
and what it cannot prevent. The item numbers (1 to 16) are cited in the code. The general threat
model is in [security.md](security.md).

## The honest limit

A malicious program running **as the same macOS user** (or as root) can, in the end, see what
Treasury sees while the vault is open: read the process memory (with root or with debugger
permission), capture the screen (with the Screen Recording permission), log keystrokes (with the
Input Monitoring permission) or replace the app with a tampered version and wait for the next
unlock. No user app blocks this completely; not even password managers like 1Password and
Bitwarden promise that.

The realistic goal is different, in three layers:

1. **Closed vault = nothing usable.** Without the password, the file and the backups do not
   open, and no secret stored on the machine opens the vault without the owner present (Touch
   ID/password).
2. **Open vault = the malware needs something expensive and noisy** (root, debugger permission,
   Screen Recording or Accessibility permission, which macOS asks the user for), instead of a
   simple command as the same user.
3. **Short window**: the vault stays open the minimum, and the key and the data in memory
   disappear on lock.

## The foundation

| Item | Evidence |
|---|---|
| Encrypted database (SQLCipher 4) with an Argon2id key (64 MiB, t=4), salt in the header | `crates/treasury-core/src/vault.rs` (`KDF_V1`, `DbKey::derive`) |
| Database key in `Zeroizing<[u8;32]>`, `Debug` without the key, hex copies also `Zeroizing` | `vault.rs` (`DbKey`, `pragma_value`, `to_secret_hex`) |
| Secrets (API keys, IMAP password) with XChaCha20-Poly1305 per field, HKDF subkey, decrypted into `Zeroizing` | `ai/secret.rs`, `mail/sync.rs:260` |
| Shamir, PIN, recovery and FIDO2 zero the intermediates | `unlock/shamir.rs`, `unlock/mod.rs`, `src-tauri/src/fido2.rs` |
| `~/.treasury` 0700, files 0600; MCP socket and token 0600 in `run/` | `paths.rs`, `mcp/socket.rs:3` |
| MCP HTTP server only on 127.0.0.1, refuses a non-local `Host`, requires a key | `mcp/http_server.rs` |
| On lock: agent jobs canceled, session closed, the interface's query cache and mutations erased, agent state zeroed | `api/system.rs:1047` (`lock`), `apps/desktop/src/api/vaultQueries.ts:76` (`clearSensitiveCache`) |
| Automatic lock on inactivity (default 15 min) | `api/mod.rs:212` |
| WebView: restricted CSP, no `remote`, ACL per command, `acl_tests` test; DevTools not enabled in the release build (no `devtools` feature in `Cargo.toml`) | `tauri.conf.json`, `capabilities/default.json`, `src-tauri/Cargo.toml` |
| `localStorage` only with interface preferences (table width, recent commands, dismissed insights); never financial data | `state/ui.ts`, `state/agentSession.ts` |
| `journal_mode = DELETE` and VACUUM with `temp_store = MEMORY` | `storage.rs:141`, `ai/history.rs:136` |
| Touch ID before using the Keychain key; the interface does not call `vault.unlock_keychain` directly | `src-tauri/src/biometric.rs` |
| AI CLIs in a private folder, per-job token revoked at the end, folders deleted | `ai/agent.rs:896-916` |

## What the app does

| Item | What the app does | Where |
|---|---|---|
| 1 | MCP API key always required (V139 turned the requirement on in all vaults; the setting only exists in tests). A connection without a key is refused at the handshake, and the key is checked again on every tool or resource call (`TreasuryMcp::refusal`), so a revoked or expired key stops working on an already open connection. A revoked key stays faded for 7 days in the list, with delete. | `mcp/keys.rs`, `V139__mcp_chave_obrigatoria.sql`, `McpAccess.tsx` |
| 2 | `treasury mcp` with the app closed is off by default; when on, it requires a valid `TREASURY_API_KEY`. | `mcp/cli.rs` (`closed_app_access`) |
| 3 | Lock on sleep, on screen lock and on user switch (macOS), setting on by default. | `src-tauri/src/session_watch.rs`, `api/system.rs` |
| 4 | Hardened runtime and empty entitlements in the release, also ad hoc. | `scripts/release.sh`, `src-tauri/release.entitlements` |
| 6, 7 | Password, PIN, code, parts and backup and e-mail passwords in `Zeroizing` from the command on; `argon2` with `zeroize`; the recovery kit too. | `vault.rs` (`Secret`), `api/mod.rs` (`parse_ref_args`, `scrub_secrets`) |
| 8 | Database key on a page of its own with `mlock` (and `MADV_DONTDUMP` on Linux); `cipher_memory_security` in the release. | `vault.rs` (`LockedKey`), `storage.rs` |
| 9 | Shamir parts and API key copied with `org.nspasteboard.ConcealedType`, only on this Mac (no universal clipboard) and erased after 60 s if nothing was copied afterward. Windows and Linux: normal copy. | `src-tauri/src/os_guard.rs`, `lib/osGuard.ts` |
| 10 | The CLI that supports it (Claude Code) runs with `--no-session-persistence`; the others show on the engine screen that they keep history outside the vault. | `ai/cli.rs`, `EnginesPanel.tsx` |
| 11 | Privacy mode also removes the window from screenshots and screen sharing (`setContentProtected`: `sharingType = none` on macOS, `WDA_EXCLUDEFROMCAPTURE` on Windows). The "Allow screenshots of Treasury" setting (Settings › Security, off by default, with a confirmation when turning on) releases it. | `lib/osGuard.ts` (`updateCapture`), `SecuritySection.tsx` |
| 12 | Secure Event Input on with focus in the password field, including with the password shown (WebKit only turns it on by itself in `type=password`). | `os_guard.rs`, `PasswordInput.tsx` |
| 13 | A test that runs password, PIN, kit, parts, backup and password change with invented data and looks for the secrets, the hex key, a description and a value in the logs, in the app's files, in recent temporary files, in errors and in the history; in the interface, `localStorage`, `sessionStorage` and the mutation cache. | `api/tests_leaks.rs`, `vault/vaultFlow.test.tsx` |
| 14 | Warning for a new MCP client (name + the program that opened `treasury mcp`, with PID): a toast right away and a list in MCP integrations until dismissed. | `mcp/clients.rs`, `mcp/socket.rs`, `McpAccess.tsx` |
| 15 | Check of its own signature on opening (release), with a system warning if the bundle changed. With an ad hoc signature, it catches a damaged bundle, not an attacker (whoever swaps the binary re-signs ad hoc); real protection comes from the Developer ID (item 16). | `src-tauri/src/integrity.rs` |
| Linux | `PR_SET_DUMPABLE = 0` and `RLIMIT_CORE = 0` in the release (`RLIMIT_CORE` on macOS too). | `vault.rs` (`harden_process`), `main.rs` |

### Item 5: key in the Secure Enclave

The Keychain item today is an ordinary "generic password" in the login keychain, with no
biometric access control: Touch ID protects the interface, not the item. The item's real
protection is the Keychain ACL, which trusts the binary that created it; a command that asks for
the item only shows the "allow access" dialog (in the ad hoc build, that dialog shows up with
every new version). There are two ways to change this:

- **A**: an item in the Data Protection Keychain with a biometric `SecAccessControl`. It requires
  the keychain-access-groups entitlement, that is, a Developer ID.
- **B**: a Secure Enclave key (P-256 with biometric access control) that wraps the database key;
  the blob stays in `~/.treasury` and only this Secure Enclave, with the fingerprint, unwraps it.

The proof of concept works without a Developer ID and without entitlements, in the dev binary
and in the ad hoc one with hardened runtime (`keychain/enclave_poc.rs`, `cargo test -p
treasury-core --lib enclave_poc -- --ignored`). The P-256 key is born in the Secure Enclave
with no Keychain item, the system returns a blob of ~324 bytes that only this Secure Enclave
uses, and `SecKeyCreateWithData` recreates the key from it **with the `toid` also in the
parameters** (without it, the system creates another key). An important finding: another binary,
with another signature, also recreates the key and opens the wrap. The blob is not tied to the
app; what protects is the access control (`biometryCurrentSet`/`userPresence`), which the Secure
Enclave enforces on every use. Missing: run the test with Touch ID
(`wraps_and_unwraps_with_touch_id`, needs someone at the Mac) and replace the Keychain item with
the wrap, with a migration.

## Release build only

The full hardening applies in the release, at no cost in the development cycle and in the tests:
hardened runtime and minimal entitlements through the `codesign` of `scripts/release.sh`,
`PRAGMA cipher_memory_security = ON` and the check of its own signature only under
`cfg(not(debug_assertions))`, `PR_SET_DUMPABLE` and `RLIMIT_CORE` in `vault::harden_process`. The
core tests keep the fast KDF (`fast-kdf`) and in-memory key storage.

## Systems

### Linux

- **`prctl(PR_SET_DUMPABLE, 0)` at startup** (done, in the release): prevents `ptrace` and
  reading `/proc/<pid>/mem` by other processes of the same user (without root) and turns off core
  dumps.
- **`mlock` matters more**: swap on Linux is often not encrypted.
- **Secret Service has no per-app ACL**: with the collection unlocked, any process of the user
  reads the item. Quick unlock on Linux should be only through the PIN with a strong Argon2id or
  FIDO2, and `treasury mcp` with the app closed turned off.
- `RLIMIT_CORE = 0` and `MADV_DONTDUMP` on the key page (done).
- An optional sandbox (Flatpak) would isolate the app from other processes of the user, at a
  packaging cost.

### Windows

- DPAPI (Credential Manager) also does not separate processes of the same user; the same care as
  on Linux applies. Windows Hello can gate the key with `KeyCredentialManager`.
- `SetProcessMitigationPolicy` (block DLLs not signed by Microsoft/unsigned),
  `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` for screenshots, `VirtualLock` on the key.
- The Windows code stays behind `#[cfg(windows)]`; the `windows` crate is already in
  `Cargo.lock` (a Tauri dependency).

## Limitations

- Item 5 has not replaced the Keychain yet: the test with Touch ID
  (`wraps_and_unwraps_with_touch_id`) and replacing the Keychain item with the wrap, with a
  migration, are missing.
- The 5-attempt PIN limit only holds inside the app: whoever copies the PIN's Keychain item and
  `treasury.db.keys` can test PINs outside it, against the Argon2id (a 6-digit PIN falls in hours
  on an ordinary computer). Item 5 would greatly reduce this risk.
- Item 16 (Developer ID and notarization) does not exist: the Keychain ACL changes with each
  version, and the check of its own signature does not protect against swapping the app.
- Lock on sleep only on macOS; on Linux (logind) and Windows (WTS), only on inactivity.
- On Windows, `VirtualLock`, `SetProcessMitigationPolicy`, the clipboard exclusion formats and
  `WinVerifyTrust` are still missing.
- The other AI CLIs still keep their history outside the vault.
