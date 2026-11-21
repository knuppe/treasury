# October 2026 assessment: security, bugs and improvements

Record of a security and quality review done on 10/19/2026 by reading the code, without running attacks. It stays as history: the line numbers are from that date, and the current state of each area is in its own document ([security.md](security.md), [local-security.md](local-security.md)). Each finding has where it was, the risk and what was done: **Done** (fixed) or **Noted** (left for later, with the reason).

## Summary

- **No serious security problem.** Access control is central:
  - `Api::call_as_inner` blocks write commands for MCP clients in read mode;
  - `write_as` turns every agent write into a proposal;
  - the `READ_ONLY` lists only have reads (checked one by one);
  - the SQL builds only fixed or checked identifiers, with values always as parameters;
  - secrets are encrypted (XChaCha20) or hashed (SHA-256);
  - external commands run without a shell;
  - the CSP is strict and Tauri's permissions apply only to the main window.
- **`npm audit` (production dependencies):** 0 vulnerabilities. `cargo audit` is not installed on the machine, so the Rust dependencies were not audited.
- **The most important bugs are about performance and the interface cache.** No money error was found in the main calculations: rounding, flow, transfers and dates are correct.

## Security

- **S1**
  - **Sev.:** medium
  - **Finding:** The whole prompt (with financial data) goes to the AI CLIs as a command-line argument, visible in `ps` to other processes on the same computer; long conversations can exceed the `ARG_MAX` limit
  - **Where:** `ai/engine.rs:90-99`, `ai/agent.rs:701-818`
  - **Status:** **Done** (9e65057): prompt through standard input or a 0600 file; not tested with the real CLIs
- **S2**
  - **Sev.:** medium
  - **Finding:** Codex with "computer access" turned off can still read any file (`--sandbox read-only` reads the disk); only the prompt's instruction prevents it
  - **Where:** `ai/cli.rs:521-526`
  - **Status:** **Done** (9e65057): without computer access, Codex runs with `shell_tool` and `view_image` turned off
- **S3**
  - **Sev.:** medium
  - **Finding:** A malicious B3 spreadsheet can exhaust memory: calamine builds a dense range (a cell at A1 and at XFD1048576 ≈ 17 billion cells)
  - **Where:** `import/b3/sheet.rs:138-141`
  - **Status:** **Done**: checks the sheet's dimension before reading (maximum of 2 million cells)
- **S4**
  - **Sev.:** low
  - **Finding:** The optional "no key on loopback" setting of the HTTP MCP server lets any local process in
  - **Where:** `mcp/http_server.rs:94-125`
  - **Status:** **Done** (3f0e2f4): confirmation and a strong warning when turning it on
- **S5**
  - **Sev.:** low
  - **Finding:** A ticker with an accent broke the check of brokerage notes (cut at byte 4 in the middle of a letter)
  - **Where:** `import/notes/engine.rs:216, 241, 331`
  - **Status:** **Done**: cut only at a character boundary
- **S6**
  - **Sev.:** low
  - **Finding:** Links in agent responses do not show the domain before opening (a manipulated agent could carry data in a URL)
  - **Where:** `design/components/Markdown.tsx`
  - **Status:** **Done** (a0cc7d0): the real domain appears next to the text, with a warning when the text looks like another address
- **S7**
  - **Sev.:** low
  - **Finding:** `freezePrototype` turned off in Tauri
  - **Where:** `tauri.conf.json:30`
  - **Status:** **Noted**; later it was left off on purpose, because ECharts breaks with a frozen `Object.prototype` ([security.md](security.md#app-window-webview))

## Bugs and quality

- **B1**
  - **Sev.:** high
  - **Finding:** Dividends to confirm rebuilt the asset's ledger and history (4 to 5 queries) for each dividend × account: thousands of queries per call
  - **Where:** `portfolio/engine.rs` (`pending_income`)
  - **Status:** **Done**: ledger loaded once per asset and account, accounts per asset cached, `prepare_cached`
- **B2**
  - **Sev.:** high
  - **Finding:** Editing or archiving an account (holders) did not refresh the per-member screens, scenarios, subscriptions and debts until the cache expired
  - **Where:** `api/hooks/coreQuery.ts` (`AFFECTS.accounts`, `AFFECTS.fx`)
  - **Status:** **Done**
- **B3**
  - **Sev.:** medium
  - **Finding:** The mock computed the card statement status differently from the core (a closed statement with total ≤ 0 stayed "closed" instead of "paid")
  - **Where:** `api/mock/ledger/reports.ts`
  - **Status:** **Done**: same rule as `cards::compute_status`
- **B4**
  - **Sev.:** medium
  - **Finding:** The mock did not use the account's single holder as the holder of a transaction without a holder (the core does): different per-member views in tests
  - **Where:** `api/mock/ledger/engine.ts` (`flowParts`)
  - **Status:** **Done**
- **B5**
  - **Sev.:** medium
  - **Finding:** A value that overflows in `to_minor` became `i64::MAX` (and sums afterwards could go wrong) and, in exchange, silently became 0
  - **Where:** `portfolio/records.rs:29`, `fx/exchange.rs:164`
  - **Status:** **Done**: explicit limit with a clear error in exchange; `to_minor` saturates at a ceiling that does not overflow in sums
- **B6**
  - **Sev.:** medium
  - **Finding:** The asset statement rebuilds the whole history for each line (quadratic in the number of operations)
  - **Where:** `portfolio/statement.rs:199`
  - **Status:** **Noted**: it only matters with thousands of operations on one asset
- **B7**
  - **Sev.:** low
  - **Finding:** Missing indexes on `corporate_action(new_asset_id)` (used in every ledger load) and `card_statement(due_date)` (cash-basis reports)
  - **Where:** migrations
  - **Status:** **Done**: migration V118
- **B8**
  - **Sev.:** low
  - **Finding:** `parse_amount` read "1.234" as 1234 units in currencies with 3 decimal places (BHD, KWD, OMR, TND)
  - **Where:** `money.rs:105-106`
  - **Status:** **Done**
- **B9**
  - **Sev.:** low
  - **Finding:** Dates in the interface: "today" in UTC (after 9 pm in Brazil it is already tomorrow) and months that roll into the next one (05/31 − 3 months = 03/03)
  - **Where:** `features/portfolio/components/Quote.tsx:21`, `IncomePanels.tsx:34`, `assets/AssetDetail.tsx:87`
  - **Status:** **Done**
- **B10**
  - **Sev.:** low
  - **Finding:** Each member's share of a shared account was rounded separately: the sum of the members can differ from the family total by 1 cent per account
  - **Where:** `reports/member.rs:75-81`
  - **Status:** **Noted**: the difference is cents and the per-member view is approximate by definition; the exact distribution requires rewriting the four reports

## Improvements suggested at the time

- **Install and run `cargo audit`** (or `cargo deny`) in `scripts/release.sh`, as is already done with `npm audit`.
- **Mock contract test:** a script that compares the shapes and some numbers of the mock with the core in more modules, beyond the cash ledger; B3 and B4 went unnoticed for lack of one.
