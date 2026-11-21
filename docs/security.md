# Security and privacy

How Treasury protects the family's data: the threat model, the vault encryption, unlocking, AI and
the MCP, distribution and the parts of the app that touch the disk or the network. To report a flaw,
see the root [SECURITY.md](../SECURITY.md). The analysis of malicious programs on the same machine is
in [local-security.md](local-security.md).

## Threat model

| Threat | Protection |
|---|---|
| Stolen laptop or copied disk | Whole database encrypted (SQLCipher AES-256); without the master password the file is unreadable. |
| Another user/process on the same machine reading the file | Encrypted database + `~/.treasury` with `0700` permission and `0600` files. |
| Leak of AI API keys | Keys encrypted at field level inside the encrypted database; never in plain text, logs or environment variables. |
| AI receiving more data than necessary | Tools return aggregates and pages of results, not the whole database; whoever does not want to send anything out uses a local model through an endpoint. |
| Agent or MCP client changing data by mistake | Writes go through an approval queue (batch or item by item), are reversible (undo) and stay in the audit log; autonomous mode is optional and per tool. |
| Exposed or corrupted backup | Backups managed by the app, always encrypted (same key or its own password), verified with `integrity_check` and hash, with rotation. Downloaded public data (fundamentals and documents from CVM/SEC/B3) stays out of the copies and is collected again after a restore. |
| Local AI CLI accessing the user's files | The CLI runs in an empty temporary directory and only receives `treasury mcp` as a tool, with a session token. The native tools are turned off in the command itself, even with the template edited (`cli::restrict_argv`): Claude Code with `--tools ""`, `--disallowedTools` and `--allowedTools mcp__treasury__*` (without `--add-dir`, `--permission-mode` or `--dangerously-skip-permissions`) and `--no-session-persistence` (the conversation does not stay in the CLI's history); the other CLIs keep the history in their own folder, outside the vault, and the engine screen warns about it; Codex always in `--sandbox read-only`; Gemini CLI without shell, write, web and file reading. |
| Agent looking at the computer without being asked | The instructions forbid looking at files, terminal, other apps, screen or browser unless explicitly requested in the conversation; API and local engines have no tools outside Treasury. Reading is only enabled by the user, per conversation ("Computer" switch, in memory, disappears on lock), and read only: Claude Code with `Read`, `Glob` and `Grep` in the home folder, Gemini CLI with the reading tools. Shell, write and web stay blocked. In Codex the read-only sandbox still allows reading through the shell: there only the instruction applies. |
| Instruction injection via data (transaction description, card statement PDF, asset name) | Tool results are treated as data, not instructions; no write happens without going through the permission policy; proposals show the real diff before applying. The screen context sent by the agent panel goes in a delimited `<contexto_da_tela>` block, with `<`/`>` escaped, marked as data; the procedure memory notes go the same way in `<notas_do_agente>`. |
| Procedure memory storing sensitive data or becoming an injection vector | Only the app's agent writes (external clients only read); the text goes through personal data redaction (CPF, CNPJ, card, account, e-mail, phone) and through a cleanup of passwords, codes and tokens before being written; site stored only as a domain. Every write stays in the history with AI authorship and can be undone; the user sees, edits and deletes in Agent › Memory. The notes return to the model only as delimited data, with the origin (conversation and engine) recorded by the app. |
| Attachments and conversations bloating the vault or storing documents | Attachments do not stay in the vault by default (only name and size); keeping them for 7 days or always requires an explicit choice with a warning. Unpinned conversations disappear in the daily cleanup (default 90 days) and `VACUUM` returns the space. Attachments stay only in memory until sent and go to the engine in a delimited block, treated as data. |
| Screen context sent to the AI provider | Only with the user's message, as the preview of the "Context of this page" chip shows; it can be turned off; it is not stored in the conversation. |
| Bank login, session, passwords, phishing | Treasury does not open bank sites or store sessions: the assisted browser was removed (D20 in [decisions.md](decisions.md); V108 deleted sites, captures, saved sessions and permissions). Links out of the app (institution sites, links in the agent's answers) open in the system browser (`open_external`, only `http`/`https`); data comes in through file import. |
| Values in plain view on a shared screen | **Privacy mode** (shortcut ⌘/Ctrl+Shift+H) hides all values; it turns on by itself when the vault is unlocked, unless turned off in Settings › Appearance and use. With it on, the window also disappears from captures and screen sharing (macOS and Windows; partial: it does not apply to a camera pointed at the screen), unless the person turns on "Allow screenshots of Treasury" in Settings › Security (off by default, with confirmation). |
| Malicious program running as your user | There is no complete protection with the vault open ([local-security.md](local-security.md)). What Treasury does: release build with hardened runtime (no library injection or memory reading without root) and a check of its own signature on open; on Linux, `PR_SET_DUMPABLE = 0` and no core dump; database key in a locked memory page; Secure Event Input in the password field; copies of Shamir parts and API keys marked as concealed, on this Mac only and erased in 60 s; MCP with a mandatory key and a new-client warning; lock on sleep or screen lock. |

## Encryption

- **Database**: SQLite + SQLCipher 4, AES-256-CBC with HMAC-SHA512 per page.
- **Database key**: derived from the master password with **Argon2id** and handed to SQLCipher as a raw key (SQLCipher's internal PBKDF2 is left out).
- **Salt**: random, written in the first 16 bytes of the file itself, which SQLCipher already reserves for this and which are not secret. The Argon2id cost parameters follow a KDF version fixed in the app (calibrated for ~0.5 s on an ordinary machine). That way the database opens with only **the password and the file itself**, with no separate key file.
- **Password change**: `PRAGMA rekey` re-encrypts the database (seconds for the size of a personal database), preceded by a backup.
- **Secrets (API keys)**: also encrypted at field level with XChaCha20-Poly1305, using a subkey derived (HKDF) from the database key, as defense in depth.
- **Memory**: password, PIN, code and keys in `zeroize` types from the command's entry; the database key lives in a page of its own, locked with `mlock` (out of swap; on Linux also out of core dumps); discarded on lock. In the release build, `PRAGMA cipher_memory_security = ON` zeroes the memory SQLCipher releases.

## Database maintenance and purge

Settings › Database shows the size of the encrypted file, the free space and the largest tables (`database.info`, through the `dbstat` virtual table), and runs an integrity check (with the references between tables), ANALYZE, `PRAGMA optimize` and VACUUM (`database.run`). The purge (`database.purge`) only reaches what is not the family's data: old change history (applied, undone or rejected; pending proposals stay), agent and MCP audit, unpinned agent conversations, market caches and the downloaded asset analysis data. Each purge has a preview (`database.purge_preview`), a confirmation and a verified backup before deleting; maintenance and purge are only available from the app's interface (AI and MCP are refused).

## Unlocking

- **Methods** ([unlock-options.md](unlock-options.md)): master password; Touch ID or the Mac password before releasing the key stored in the Keychain (the interface cannot skip the confirmation: `api_call` refuses `vault.unlock_keychain`); a 6 to 12 digit PIN stored only in this computer's Keychain, encrypting the stable recovery key with Argon2id, with 5 attempts; recovery by the kit, by 2 of the 3 Shamir parts or by a FIDO2 key (hmac-secret), always requiring a new password. Parts and the hardware key depend on the `treasury.db.keys` file next to the database, which only holds the database key encrypted with the recovery key (without it, it reveals nothing).

- Master password required on first run, with a strength meter.
- Optional **recovery kit**: the database key printed/exported as a code (base32). Without a password and without a kit, there is no way to recover the data, and the app says so clearly when the vault is created.
- Optional: store the key in the **macOS Keychain** (with Touch ID) or in the **Secret Service/libsecret** on Linux, for quick unlock.
- Automatic lock on inactivity (default 15 min) and, on macOS, when the computer sleeps, the screen is locked (shortcut, screen saver, hot corner) or someone else enters the session ("Lock on sleep or screen lock" setting, on by default; `src-tauri/src/session_watch.rs`). On Linux and Windows, for now, only on inactivity. While locked, the MCP socket refuses calls.

## AI and MCP

- Calls to the providers go straight from the machine to the provider's API, with no intermediate server.
- A visible indicator of which provider/model is receiving data and the option of local models (Ollama, LM Studio) for whoever does not want to send anything out.
- Local CLIs run with restricted native tools: in Claude Code, `--strict-mcp-config`, `--tools ""` and `--allowedTools` limited to Treasury's tools; in Codex, `--sandbox read-only`; in Gemini CLI, the native tools excluded in `settings.json`. The template of each known CLI already comes with these restrictions and the app reapplies them on every run, removing from the command the flags that would release the computer. File reading only comes back with computer access enabled by the user in the conversation.
- The agent is instructed not to look at the user's computer (files, terminal, other apps, screen, browser) without an explicit request in the conversation; see [ai-agent.md](ai-agent.md#users-computer).
- The agent's procedure memory (where and how it obtained each piece of information) is the only AI write without a proposal: it stays in the vault, in the history (undoable), in the backups and in the export, and is visible and editable by the user.
- `treasury mcp` runs over stdio, only for local clients, and talks to the app through a local socket (`0600` + token per session). External clients follow the same permission policy as the internal agent and can be limited to read only. Details in [ai-agent.md](ai-agent.md#permissions-and-control).
- With the app closed, `treasury mcp` only opens the vault (in read mode, through the keychain unlock) with the "Answer with the app closed" setting on, which is off by default and lives inside the vault, and with a valid API key in `TREASURY_API_KEY`. Without that, it explains that Treasury must be opened: another program of the user does not read the vault just by running the app's binary ([local-security.md](local-security.md), item 2).
- Every tool use by the AI goes to `ai_audit_log`.
- A new external client in this vault (client name + program that opened `treasury mcp`) raises a warning on the spot and stays listed in MCP integrations, with the program and the PID, until dismissed (`mcp/clients.rs`).

## External agents (MCP)

- API keys per agent, with scopes (`read`; `write`, which only creates proposals; the old `browser` left the keys in V108). Only the key's SHA-256 stays in the database; the key appears once. Revocation and expiry apply immediately.
- No scope approves proposals, changes settings, creates keys, opens the vault or writes to the agent's procedure memory (external clients only query it with `recall_procedures`).
- The HTTP server is optional, listens only on `127.0.0.1`, refuses a non-local `Host` (DNS rebinding) and requires the key on every request. The local socket stays restricted to the system user (`0600` + token).
- The API key is **always required** for external clients (V139 turned the requirement on in all vaults, and the screen has no way to turn it off): the socket token is `0600`, but any process of the same user reads it, so only the key separates a known client. The app's own agent uses the job token and needs no key. The old `mcp.require_api_key` setting, writes without a key and the loopback-without-key exception only hold in the core tests (`keys::require_key`).
- A revoked key stops working immediately, stays in the list dimmed for 7 days (the person can delete it earlier) and then leaves by itself; the audit of the calls made with it remains.

## Distribution

- macOS: today with an ad hoc signature and hardened runtime (`scripts/release.sh`) and a check of its own signature on open (`src-tauri/src/integrity.rs`), which warns if the bundle was altered. Developer ID and notarization not yet ([local-security.md](local-security.md), item 16). Linux: AppImage, `.deb` and `.rpm`; Windows: NSIS installer without code signing. All with `SHA256SUMS` ([distribution.md](distribution.md)).
- Updates through the app's own updater in all packages (macOS, AppImage, `.deb`, `.rpm`, NSIS and `.msi`), with two signatures: minisign (embedded public key, version tied to the signature against downgrade) and a FIDO2 signature made on the maintainer's YubiKey, with touch and PIN, always required (without embedded keys, the app refuses every update). Downloaded only from the GitHub releases (`knuppe/treasury`); downloaded on their own by default (Settings › Updates can switch to notify only or off), and installed only with confirmation. The packages are built in CI with a provenance attestation and signed only on the maintainer's Mac. Only the maintainer publishes releases, and the why is in [trust.md](trust.md); details in [auto-update.md](auto-update.md).
- Dependencies: `cargo deny` (RustSec) and `npm audit` in CI and before each release; a known vulnerability blocks the release.
- No telemetry. Error reports only if the user sends them manually, and logs never contain values, descriptions or keys.
- Data export in an open format (CSV/JSON) warns that the file leaves **unencrypted**.

## App window (webview)

- Restricted CSP: only the app's own scripts (`script-src 'self'`), no frames, no external forms and network only through Tauri's IPC. The interface never interprets HTML coming from data (descriptions, agent answers, CVM/SEC documents are text).
- Tauri capabilities only for the `main` **webview**. The app has an ACL manifest (`build.rs`): each app command needs `allow-<command>` in the capability; remote pages receive no permission (no `remote`). The `acl_tests` test (Tauri's mock runtime) checks that another webview, on a remote or local page, calls no commands or plugins.
- Links out of the app never navigate the window: `app/externalLinks.ts` diverts every click on an `http(s)` link to another address to the system browser (`open_external`).
- `freezePrototype` stays **off**: ECharts writes properties on plain objects when loading and breaks with a frozen `Object.prototype` ("Attempted to assign to readonly property" error in WebKit). Protection against prototype pollution comes from the CSP and from never evaluating code or HTML from data. The end-to-end tests also run on WebKit, the window's engine on macOS.

## Downloads folder

- Optional and **off by default** (Settings › Appearance and use › Downloads folder, setting `watch_downloads`). When on, and only with the vault open, the app shell lists `~/Downloads` (or `~/Download`) every 3 s: name, size and date of the files, without subfolders or shortcuts. What was already in the folder when the monitoring started is not read.
- A new file with an importable extension (OFX, QFX, CSV, TXT, PDF, XLSX, XLS, ODS, and images PNG, JPEG, HEIC, WebP and GIF; up to 25 MB), neither hidden nor partial (`.crdownload`, `.part`, `.download`…), and with a stable size across two consecutive reads is read **in memory** only to recognize the format (core `import.sniff`, called directly: it does not count as activity for the automatic lock). Nothing is written or leaves the computer; an unknown format is ignored. The interface receives only the path, name, size and the recognized type (`downloads://file`, only for the `main` webview).
- The content only goes to the interface when the user clicks Import (`downloads_read`): the command requires the monitoring on and the vault open, resolves the path (`canonicalize`) and refuses what is not **directly** inside the monitored folder (subfolders, `..`, shortcuts to outside) or has no importable extension. The file stays in the folder; the import follows the normal flow (preview and confirmation).
- On macOS, the first read may ask for the system permission for the Downloads folder (`NSDownloadsFolderUsageDescription`); if denied, the app only logs a warning.
- Images have no importer of their own: accepted, they go as an attachment to the AI agent panel (same path as attachments typed by the user, with HEIC converted to JPEG in the app), which reads and **proposes** the entry through the tools; nothing is written without approval.
- **Delete after importing** (setting `downloads_delete_after_import`, off by default, checked in the notice or in Preferences): when the import of an offered file finishes successfully, the interface calls `downloads_trash`, which requires the monitoring and the setting on, the vault open and the same path rules as `downloads_read`, and **sends the file to the system Trash** (macOS: `NSFileManager.trashItemAtURL`; Linux: freedesktop Trash in `~/.local/share/Trash`), from where the user recovers it. It never deletes for good, nor files outside the monitored folder, nor images sent to the agent.

## Ingestion e-mail

- The IMAP mailbox (app password) or the local folder is only read with the app open and the vault unlocked; nothing is sent (no SMTP). Details in [email-ingest.md](email-ingest.md).
- The app password stays in the `secret` table with field-level encryption, never in a log, environment, argument, MCP or agent; changing the server, port or user deletes the registered password. It is decrypted only during the fetch, and the connection lasts the fetch. Restoring a backup requires confirming the connection before fetching or deleting.
- TLS required with the public authorities (webpki-roots); the only exception is Proton Mail Bridge on 127.0.0.1, with the certificate pinned by SHA-256.
- Configuring accounts, rules, password and retention, opening the content of the messages and fetching is only through the app's interface; the queue and the audit can be read by MCP clients.
- Sender authentication by the topmost Authentication-Results block, with the provider's authserv-id (RFC 8601); "apply by itself" only with an authenticated sender listed in the rule, a deterministic reader and a preview without warnings. What the AI reads from an e-mail always becomes a proposal, even in autonomous mode, and it only receives the preview and import tools.
- Retention starts with 7 days of test mode and only deletes after explicit confirmation; never without a matched rule, outside the folders read, or without checking the stored copy. Every deletion goes to the audit and cannot be undone.
- Limits: 25 MB per message, 20 attachments, nothing compressed, 200 messages per day, HTML without network, an attachment name never becomes a path, logs without personal data.
