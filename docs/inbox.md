# File inbox

A folder on the computer that Treasury watches: a file dropped in it enters the vault and waits
for import. It is mainly for external agents (MCP), which cannot send Treasury the attachments of their
own chat: instead of copying the file as base64, they save it in the folder and
import it by id. It lives at Transactions › Inbox.

## How it works

- **Inbox folder**: default `~/Treasury/Inbox` (setting `inbox_folder`; empty = default),
  watched while the vault is open (setting `watch_inbox`, on by default; Settings ›
  Appearance and use). The app shell reads the folder every 3 s (`apps/desktop/src-tauri/src/inbox.rs`) and
  `list_inbox` also reads it before answering, so a file shows up within seconds.
- **Reading** (`import/inbox.rs`): only regular files directly in the folder; hidden and partial files
  (`.crdownload`, `.part`, `.tmp`…) are left out; a file changed less than 1 s ago waits for the
  next read (copy in progress). Up to 20 MB; a larger one stays in the folder and a warning is shown.
- **Type by content**: the same `import.sniff` as the import (OFX/CSV, brokerage note,
  B3 report, pension statement, receipt, image). An unknown type becomes an
  `unrecognized` item without breaking the read.
- **Stored in the vault**: the content goes to `inbox_item` (inside the encrypted database) and the file
  leaves the folder. The SHA-256 is unique: the same file again leaves the folder without creating another item.
  `inbox_item` stays out of the change history (like the email queue); what counts for
  undo is what the import writes.
- **States**: `pending`, `imported`, `dismissed`, `unrecognized`.

## MCP

- `list_inbox` (read; `import.inbox_list`): reads the folder and lists the items with id, name,
  size, `detected_kind`, `label` and `status` (`include_done` brings the imported and
  dismissed ones).
- The preview and import tools that take `files` (statement, PDF card statement, brokerage
  notes, payslip, receipt, pension statement, FGTS) also accept:
  - `inbox_id`: the inbox id (one or several). The server swaps it for `files` with the stored
    content, before validation (`mcp/inbox_files.rs`); the preview is the same as for a base64 upload.
  - `path`: a local path, only inside the allowed folders (Downloads and the inbox folder,
    resolved without `..` or shortcuts); outside them, refused with a message listing the folders. Up to 20 MB.
- After a successful import (`import_*`) with `inbox_id`, the item becomes `imported`.
  Since agent writes become proposals, "imported" here means "handed to the
  import"; what writes the data is still the approval of the proposal.

## Interface

- Transactions › Inbox: the items with type, size and date; "Import" opens the single
  import window with the file (the right importer for each type) and, once
  imported, the item leaves the list; "Dismiss" removes it without importing.
- A notice when a file arrives (`inbox://new`), with "Review".

## In the code

- `crates/treasury-core/src/import/inbox.rs` (reading and items), `crates/treasury-core/src/mcp/inbox_files.rs`
  (swap of `inbox_id` and `path` for `files`), `apps/desktop/src-tauri/src/inbox.rs` (periodic
  reading); table `inbox_item` (V162).

## Limitations

- The folder is read every 3 s (like the Downloads folder), without file system events.
- The folder only changes through the `inbox_folder` setting; there is no picker in the interface.
