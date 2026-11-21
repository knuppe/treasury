# Multiple vaults

A family can have more than one vault, each with its own password and its own backups: the business, the parents' finances, a test vault. They are in Settings › **Vaults**, the "Vaults ▾" menu in the sidebar and "Other vaults" on the unlock and create screens. Core in `crates/treasury-core/src/vaults.rs`, commands `vaults.*`.

Each vault is an encrypted .db file (SQLCipher) with **its own password**, quick unlock
(keychain/Touch ID, PIN and hardware keys are per file) and **backups in its own folder**. The
app opens one vault at a time; switching closes the current one (with the closing backup) and asks
to unlock the other.

## Where it lives

- The list lives in `~/.treasury/config.toml` (`[[vaults]]` with id, name, path and purpose,
  and `active_vault`), outside the vaults. Without a list, there is only the **main** vault: the
  usual file (`db_path` from the config or `~/.treasury/data/treasury.db`), with the usual backups.
- New vaults live in `~/.treasury/data/<id>.db`; their backups in
  `~/.treasury/backups/<id>/`.
- The app (and `treasury mcp` with the app closed) opens the last vault chosen.

## Commands

| Command | What it does |
|---|---|
| `vaults.list` | Lists (name, purpose, path, size, whether it exists, which one is in use) |
| `vaults.new {name, purpose?, path?}` | Registers a new vault and starts using it; the screen continues to `vault.create` |
| `vaults.add {path, name?, purpose?, select?}` | Registers an existing .db file (refuses unencrypted SQLite and duplicate files) |
| `vaults.select {id}` | Switches vault (locks the current one) |
| `vaults.update {id, name?, purpose?}` | Renames or changes the purpose |
| `vaults.remove {id, delete_file?}` | Removes it from the list; the file stays where it is, or, with `delete_file`, goes to the Trash (along with the `.keys`; the backups stay, and its quick unlock is turned off); the vault in use cannot be removed |
| `vaults.copy {id, name?}` | Creates a copy of the vault (same password), listed as "… (copy)", without switching vaults; the open one is copied through SQLite's backup API |
| `vaults.new_demo {name, household_name, password, profile}` | Creates a separate demo vault and fills it with the assistant's history ([demo.md](demo.md)) |

Only the app's interface manages vaults (MCP clients get `forbidden`).

On screen, each vault has a context menu (right click, Shift+F10 or the ⋯ button): open,
rename and purpose, create a copy, copy the file path and remove, which asks first
whether to take it off the list only or also send the file to the Trash (macOS: `~/.Trash`; Linux: the
freedesktop trash; other systems: the `removidos` folder in the data directory).
