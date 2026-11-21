# Ingestion email

Treasury reads statements, card statements, notes, fiscal receipts and purchase notices that
arrive by email, without the family having to download and import each file. Two ways:

- **A mailbox just for Treasury** (recommended), fed by automatic forwarding of the bank's
  emails, read over IMAP with an app password.
- **A watched folder on the computer**, where the family saves attachments, statements and `.eml`
  messages (or where the email client saves the attachments). It asks for no credentials at all.

What passes the filters is identified by the app's readers (or by the AI, if the rule allows),
goes to the **review queue** and, if configured, is removed from the provider afterwards
(retention, with a test mode). There is no SMTP and no server: it only runs with the app open and
the vault unlocked.

- **Where it lives:** Transactions › Email (queue, accounts, rules and audit), with a shortcut in
  the import window.
- **Core:** `mail` (V125: `mail_account`, `mail_rule`, `mail_folder_state`, `mail_message`,
  `mail_part`, `mail_seen`, `mail_audit`). Accounts and rules go into the change history; the
  queue, the folder state and the audit stay out of it (what counts for undo is what the import
  wrote into the family's data).
- **Commands:** `mail.accounts.{list,get,save,delete,set_password,confirm_retention}`,
  `mail.rules.{list,save,delete,test}`, `mail.{list,get,audit,would_delete,providers}`,
  `mail.{part_content,mark_part,ignore}` and, outside the vault lock (network and threads),
  `mail.{discover,test_connection,sync_now,ask_ai,apply_part}`. Configuring accounts, rules,
  password and retention, opening the content of the parts and fetching is **only through the
  app's interface**; the queue and the audit can be read by MCP clients (with no secret in them).

## The path of each message

1. **Fetch** only the headers of the allowed folders with UID greater than the last one seen
   (`BODY.PEEK[HEADER]`, without marking as read). Per folder: UIDVALIDITY and the last UID; if
   UIDVALIDITY changes, start over. First fetch: the last 30 days. Up to 200 messages per folder
   per fetch.
2. **Filters** (below). A rejected one becomes just a count in the audit. A forwarded one (subject
   `Fwd:`, `Enc:`, `TR:` or a family sender) is downloaded anyway, because the sender that counts
   is the original one, which is in the body.
3. **Download** the whole message, up to 25 MB.
4. **Parts:** the attachments (up to 20; nothing compressed), the attached `.eml` (one level
   only) and the body as text (HTML becomes text with no network at all). Images in the body
   itself (logos, icons, tracking pixel) are not parts. An attachment's name never becomes a path.
5. **Identify** each part by its content (`import.sniff` and the app's readers):

   | Part | Reader | Can apply on its own? |
   |---|---|---|
   | OFX statement or card statement | statement import, account by bank, branch and number (`accounts::match_ofx`) | yes |
   | Fiscal receipt: NF-e/NFC-e XML, NFC-e DANFE PDF or the lookup link in the body | `receipts` (Brazil only) | yes |
   | Card purchase notice ("Compra de R$ 45,90 aprovada em LOJA… final 1234") | `mail/body.rs`, account by the card's last digits | yes |
   | Pix received or sent notice | `mail/body.rs` | yes, with the account given by the template |
   | Rule template (expression with the groups `valor`, `descricao` and `data`) | `mail/body.rs` | yes, with the template's account |
   | CSV statement, PDF card statement, brokerage note, B3 report, pension, payslip | opens in the app's importer | no (review) |
   | Unrecognized image or document | the AI, if the rule allows | never |

6. **Duplicate?** The same content (SHA-256), the same note (access key), the same card
   statement (bank and due month), the same Message-ID or the same file already imported into the
   account.
7. **Review or apply on its own** (below). Each import is a set of changes with the author
   "Email: <rule>" (undone through the history, like any import).
8. **Retention** (below).

Fetches run in the app's routine cycle (`ai::run_due`, every 30 s): each account fetches when its
interval has passed (15 min by default, from 5 to 120), in its own thread; "Fetch now" triggers
it right away. Limits: 200 new messages per day and per account (the rest waits for tomorrow) and
10 automatic AI reads per day and per account.

## Filters

The order is always: **folders → account blocks → rules by priority** (the first one that
matches decides). With no rule, the account default applies: **ignore** in the IMAP mailbox,
**review** in the local folder (the user put the file there). A block always beats a permission.
Everything is pure (`mail/filter.rs`): the rule's **Test** button uses the same functions against
an example and against recent messages, without importing or deleting anything.

- **Folders:** allowed (default: only `INBOX` in the dedicated mailbox; none in the main mailbox,
  until the user chooses) and blocked (default: `\Junk`, `\Trash`, `\Sent`, `\Drafts` and `\All`),
  by RFC 6154 flags, never by name; wildcard (`Bancos/*`). iCloud does not flag the folders: the
  catalog carries its names ("Deleted Messages" is `\Trash`…). In the local folder, each subfolder
  is a folder (the root is `INBOX`); hidden ones, shortcuts and `Processados` are never read.
- **Sender:** exact address, domain (`banco.com.br` or `@banco.com.br`), subdomains
  (`*.itau.com.br`, which does not catch `itau.com.br` nor `itau.com.br.outro.com`), wildcard
  (`fatura*@*.com.br`), `/regex/` and `membros` (the family's addresses, in the account). Allow
  and block lists on the account (all rules) and on each rule. On a forwarded message, the
  original sender applies.
- **Subject:** contains terms (any or all), does not contain, regular expression. Ignoring case
  and accents, with the subject decoded (RFC 2047) and without `Fwd:`/`Enc:`/`RES:` in front.
  Expressions use the `regex` crate (linear time), up to 300 characters and with a compiled-size
  limit; the syntax error appears on save, with the position.
- **Advanced:** AND/OR/NOT groups (up to 6 levels) over recipient, attachment (name or
  extension), size, date and body. A condition on the body or the attachments only decides after
  the download.

## Review and apply on its own

The default of every rule is **review**. In the queue, each part has the action that fits:
**Import** (OFX, receipt), **Record** (notice from the body, choosing the account if the notice
does not say), **Open in the importer** (the import window already with the file; when saved
there, the part leaves the queue), **Ask the AI** (if the rule allows), **Already imported** and
**Ignore**.

**Apply on its own** exists, off by default, and only happens with all of this:

- a deterministic reader (the table above);
- a sender **authenticated** by the provider (below) and listed in the rule itself (a "pass" alone
  is not enough: there has already been a forged sender with valid DKIM and DMARC);
- a preview **without warnings** (no error, no possible duplicate, no warning from the import or
  the receipt).

Otherwise it goes to review with the reason. A file dropped in the local folder counts as the
user's own. A message forwarded by an authenticated member only applies on its own if the
original sender is in the rule's allowed list.

## Retention

Per account, and the rule can have its own:

| Mode | What leaves |
|---|---|
| Never (default) | nothing |
| After imported | imported or duplicate, after the grace period (7 days by default) |
| After X days | any state, X days after arriving (including what still awaits review or was not recognized: the screen warns; the copy stays in the vault) |
| Whichever comes first | both |

It never deletes outside the allowed folders, in a blocked folder, **with no matched rule** (the
account default does not delete) nor in test mode. With **keep the original** (default), it checks
the SHA-256 of the copy in the vault before deleting; if it does not match, it does not delete and
logs it. Before deleting on IMAP, it checks that the folder is still the one read and that
UIDVALIDITY is the same as when it downloaded.

- **Test mode:** when turning on any deletion, 7 days only recording in the audit what it would
  have deleted. Afterwards, it requires **explicit confirmation** (without it, it stays in test).
  Turning deletion off resets everything; turning it on again starts another test.
- **Generic IMAP:** `UID MOVE` to the `\Trash` folder. Without MOVE (iCloud), `UID COPY` to the
  trash, `\Deleted` and `UID EXPUNGE` of only that UID (UIDPLUS). Without a trash, `\Deleted` and
  `UID EXPUNGE` of only Treasury's UIDs; without UIDPLUS, it deletes nothing.
- **Gmail:** `\Deleted` in a folder only removes the label (the message stays in All Mail), so it
  is always a move to the trash, which Gmail empties in 30 days. "Also delete from the trash"
  exists, off (finds the message there by Message-ID and removes only it). In the main mailbox,
  the screen warns that the trash removes the message from all labels.
- **Local folder:** moves to `Processados/<subfolder>` (default) or sends to the system Trash.
- Every deletion goes to the audit and **has no undo**.

## Security

- **Sender authentication:** only the top Authentication-Results block counts, in sequence, with
  the provider's authserv-id (RFC 8601: the server removes those claiming to be its own; an
  identical one written by the sender stays below and does not count). iCloud and Proton write one
  header per check; Yahoo and Fastmail change the server name (the suffix counts, `*.yahoo.com`).
  Authenticated = DMARC pass aligned with "From", or DKIM pass with the aligned domain (relaxed).
  On a forwarded message, authentication applies to whoever forwarded it; the original counts as
  not authenticated. Not authenticated is not blocked: it only loses "apply on its own". The local
  folder has no provider: its `.eml` files are never authenticated.
- **AI:** off by default; per rule, never, on request or on its own (up to 10 per day). It
  receives only the part, as third-party data (the instruction says to ignore any request written
  in it), and only the preview and import tools, plus reading accounts, categories and
  transactions to avoid duplicates. Everything it writes becomes a **proposal** in the approval
  queue, never applied on its own (not even in autonomous mode). It goes through the "PDF reading"
  route, only with API engines or local models (the CLIs see all the tools through the socket and
  are left out). The body text goes with CPF, CNPJ, card, account, email and phone masked. The app
  does not yet have an "anonymize" mode that swaps names; when it does, it applies here too.
- **Credentials:** the app password lives in the `secret` table with per-field encryption
  (XChaCha20-Poly1305, HKDF subkey of the database key), never in a log, environment variable,
  argument, MCP or agent; the interface only sees "registered" and the last 4 characters.
  Changing the server, port or user deletes the registered password (it never goes to another
  server without the user typing it again). The password is decrypted only during the fetch and
  the connection lasts the fetch (connect, read, delete, leave): locking the vault interrupts the
  fetch and nothing is left in memory. PDF passwords are not stored (the importer asks when it
  needs one). Restoring a backup marks the IMAP mailboxes to **confirm the connection** (Test)
  before fetching or deleting again.
- **TLS:** mandatory, with the certificates of the public authorities (webpki-roots), without
  accepting an invalid certificate. Exception: Proton Mail Bridge on the computer itself
  (127.0.0.1), with STARTTLS and its certificate **pinned** by SHA-256 (the `cert.pem` exported by
  the Bridge). Outside the local Bridge, only TLS from the start (port 993).
- **Limits:** 25 MB per message, 20 attachments, nothing compressed, 200 messages per day, HTML
  with no network, an attachment's name never becomes a path, logs without personal data (only the
  error code). The audit (in the vault) keeps the subject and the sender.

## Providers

Checked on each one's help pages and by direct connection (CAPABILITY before login) on
10/19/2026. The wizard fills in by the address's domain; outside the catalog, it looks for the
domain's own autoconfig (`https://autoconfig.<domínio>/mail/config-v1.1.xml` and
`/.well-known/…`) and the Thunderbird database (`autoconfig.thunderbird.net`), source
"Email autoconfiguration (Thunderbird)" in Settings › Data sources. Only the domain leaves the
computer, never the address.

| Provider | Server | Password | Note |
|---|---|---|---|
| Gmail (default) | imap.gmail.com:993 | app password (2-step verification) | work accounts, with Advanced Protection or only a security key, have no app password |
| iCloud Mail | imap.mail.me.com:993 | app password (two factors) | the user is the name before the @; no MOVE or folder flags |
| Yahoo Mail | imap.mail.yahoo.com:993 | app password | only unlocks in a browser already used for a few days |
| Fastmail | imap.fastmail.com:993 | app password (mandatory) | the Basic plan has no IMAP |
| Zoho Mail | imap.zoho.com:993 (imappro. for paid organizations; .eu, .in) | password, or app password with two factors | turn on IMAP Access; the free plan has no IMAP; authserv-id still to be confirmed with a real header |
| Proton Mail | Bridge at 127.0.0.1:1143, STARTTLS | the one the Bridge shows | paid plans; pinned certificate |
| Manual | whatever the provider indicates | | TLS (993) |

**Not yet:** Outlook.com and Hotmail (OAuth2 only since 09/16/2024; for now, the local folder),
Google OAuth and JMAP. **Out:** POP3 and the Gmail API.

## Decisions

- The local folder works with no credentials at all: the same pipeline, filters and queue.
- "Apply on its own" exists and starts off.
- The tab lives in Transactions, with a shortcut in the import window.
- Several accounts per vault (the recommendation is one family mailbox).
- A 7-day grace period after success; a 7-day test mode before any deletion.
- A message forwarded by an authenticated member is only trusted with the original sender in the
  rule's allowed list.
- Automatic AI offered, off by default, up to 10 per day.
- Keeping the original in the vault is the default.
