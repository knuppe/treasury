# Security policy

[Português](SECURITY.pt-BR.md)

## Reporting a vulnerability

**Do not open a public issue** for a security problem. Use GitHub's private reporting:
[Security › Report a vulnerability](../../security/advisories/new). Only the maintainer sees the
report, and the conversation continues there until the fix ships.

In the report, include:

- the Treasury version and your system (macOS, Linux or Windows);
- what the flaw allows and under which conditions (vault unlocked or locked, MCP server on,
  imported file, automatic update);
- steps to reproduce, preferably in the **demo vault**.

Never send bank statements, passwords, API keys or real financial data, not even in the private
report. If a file is needed to reproduce the issue, build one with made-up values.

## What to expect

- I acknowledge the report within 7 days.
- We agree on the fix timeline and the disclosure date together. The fix ships in a new version,
  delivered by the automatic update, and the security advisory credits the reporter if they wish.
- Treasury is maintained by one person, with no bug bounty program.

## Supported versions

Only the latest version receives security fixes. Automatic updates download on their own by default (Settings ›
Updates, which can also only notify or be turned off) and only install packages signed with the project's keys (see
[docs/auto-update.md](docs/auto-update.md)).

## In scope

- Vault encryption and unlocking (SQLCipher, Argon2id, Keychain, Touch ID, hardware key).
- The local MCP server: access without a valid key, writes that bypass the approval queue, reads
  beyond what the key allows.
- The AI agent: a change applied without the person's approval, data sent to an engine that was
  not chosen.
- Automatic updates: a package accepted without the right signature, a swapped download address.
- File import (statements, PDFs, spreadsheets, email): a malicious file that runs code, reads
  files outside the vault or crashes the app.
- The app window (webview): script execution from imported data or AI responses.

## Out of scope

- Attacks that require an already compromised computer with the vault unlocked (whoever controls
  the open session sees what the person sees).
- What happens to data after the person sends it to a cloud AI engine or to an MCP client they set
  up themselves.
- Flaws in third-party services (quotes, indices, AI providers) that do not affect the app.
- Social engineering and attacks that depend on the person installing a package from outside the
  official releases.

The full threat model is in [docs/security.md](docs/security.md) and the local defenses in
[docs/local-security.md](docs/local-security.md).

## Good faith

Research done in good faith, on your own installation and the demo vault, without accessing other
people's data and allowing time for a fix before disclosure, is welcome. I will not take action
against anyone acting this way.
