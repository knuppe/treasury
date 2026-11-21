# Backups: recovery and copy

The app makes encrypted backups on its own ([architecture.md](architecture.md#managed-backups)).
This document covers what to do with them: restoring when the vault does not open, taking a copy to
other media (USB drive, external disk, cloud) and bringing back a backup from outside.

## What you can do

1. **Recover from a backup on the unlock screen.** A "Recover from a backup" link is always on the
   unlock screen, and the recovery screen opens by itself when the vault fails to open for a reason
   other than just the password (see "When the screen appears"). The screen lists the backups in the
   vault folder (and the imported ones), from newest to oldest, and restores the chosen one with the
   password, **without needing to open the vault**.
2. **Save a copy of the backup** (Settings › Backups and the recovery screen): the system "Save as"
   dialog picks where, and the core copies the file and checks the copy (SHA-256 equal to the
   original's).
3. **Bring a backup from outside** (Settings › Backups and the recovery screen): the "Open" dialog
   picks the file, the core checks that it is a Treasury backup (SQLCipher header) and copies it to
   `backups/importados/` in the vault folder. From there it shows up in the list and can be restored
   like the others.

## When the screen appears

| What happens on opening | What the unlock screen shows |
| --- | --- |
| Wrong password, and it does not open the newest backup either | "Incorrect password". The link to recovery stays there. |
| Wrong password for the vault, but **right for the newest backup** | Warning: "This password opens your backups but not the vault. The vault may be damaged (or the password changed after those backups)." Button for recovery. |
| Database error (damaged file, `SQLITE_CORRUPT`/`NOTADB`), I/O, encryption or migration error | The recovery screen opens directly, with the error in one line. |
| Vault created by a newer version of the app | Message to update the app (restoring an old backup would lose what was done afterward). Recovery stays available, with this warning. |

Testing the password on the newest backup costs one extra key derivation (about a second), and only
happens when the password did not open the vault. When the newest backup is the one from before the
password change (`pre_rekey`), the test does not run: it opens with the old password, and someone who
types the old one right after the change does not have a damaged vault.

## What happens to the broken vault

**It is never deleted.** Before the backup is put in place, the vault file (and the `-wal`/`-shm`
files, if any) is **moved aside**, in the same folder, as `treasury.db.danificado-AAAAMMDD-HHMMSS`.
That way you can try to salvage something from it later, or send it for analysis. The screen says
where it ended up.

The restore itself is the usual one (`backup::prepare_restore`): the backup is opened with its own
password, checked (`integrity_check` and `cipher_integrity_check`), re-encrypted with the master
password typed (new salt), checked again and only then swapped in. A backup made by a newer version
of the app is refused before the swap. If any step fails, the broken vault stays where it was.

## Passwords

- The backup opens with the password the vault had **when the backup was made** (or with the
  backup's own password, if it is turned on). The screen asks for the master password (which will
  apply to the restored vault) and, in an optional field, "the backup password, if it is different".
- Whoever forgot the password uses the recovery kit ([unlock-options.md](unlock-options.md)); recovery
  by backup does not replace the kit.

## Encryption of the exported copy

The backup is **already encrypted** (SQLCipher, AES-256) with the vault password or the backup
password. The exported copy is the same file, byte for byte: it stays encrypted, and whoever finds it
on a USB drive reads nothing without the password. **There is no unencrypted export.** The suggested
name is the backup's (`treasury-2026-10-22T03-00-00.000Z-auto_daily.db.enc`), which already tells the
date.

## Where the list comes from

The list on the recovery screen comes from the folder (the file names tell the date and the type),
because with a broken vault the `backup_record` table is not accessible. The imported ones stay in a
subfolder and do not enter the automatic rotation (they are never deleted by it).

## Core commands

| Command | Vault | What it does |
| --- | --- | --- |
| `backup.scan` | closed or open | Lists the backup files in the vault folder and in `importados/` (date, type, size, origin). |
| `backup.restore_file` | closed | Restores a file from the list: moves the current vault aside, swaps and opens. |
| `backup.export_file` | closed or open | Copies a file from the list to the path chosen in the dialog and checks the SHA-256. |
| `backup.import_file` | closed or open | Checks and copies a file from outside into `importados/`. |

Only the interface calls them (never the agent or the MCP). The source paths of `restore_file` and
`export_file` must be in the backup folder of the active vault.

## Limitations

- There is no unencrypted export of the backup; readable data leaves through Settings › Export data
  (CSV/JSON), with a warning.
- The app does not remind you to copy the backup out. To always have a copy on other media, use an
  extra destination folder in Settings › Backups (external disk, synced folder), which also receives
  every backup.
