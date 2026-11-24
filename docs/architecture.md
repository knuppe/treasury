# Architecture

The stack, the processes, on-disk storage, backups, the high-level data model and the core
calculations. Code conventions and development commands are in
[development.md](development.md); the MCP server and the agent are in [ai-agent.md](ai-agent.md).

## Stack

| Layer | Choice | Why |
|---|---|---|
| Desktop shell | **Tauri 2** | Small binary, uses the native WebView and packages `.dmg`/`.app` (macOS Apple Silicon), `.AppImage`/`.deb`/`.rpm` (Linux) and the NSIS/`.msi` installer (Windows) from the same code. |
| Core | **Rust** | Memory safety for an app that holds financial data and keys; `rust_decimal` for monetary precision; great support for SQLCipher, Argon2 and the keychain. |
| UI | **React + TypeScript + Vite** | Mature ecosystem for a rich, good-looking UI; makes the "instrument" look easy (mono typography, phosphor themes, dense charts). |
| UI kit | Tailwind CSS + Radix UI (accessible primitives) + own components | Full control of the look without fighting a ready-made design system. |
| Charts | **Apache ECharts** (Sankey, treemap, sunburst, calendar heatmap, bars, lines, areas) + **TradingView Lightweight Charts** (price series) | Covers every expense visualization with interactivity and custom themes; performs well with long series. This was one of the reasons to keep a web UI instead of a native GUI. |
| Database | **SQLite + SQLCipher** (via `rusqlite` with `bundled-sqlcipher`) | Single file, transparent AES-256 encryption, no server, easy to back up. |
| Migrations | `refinery` (supports `rusqlite`) | Schema versioning; migrations are embedded in the binary and preceded by an automatic backup. |
| UI state | TanStack Query + Zustand | Cache of core calls and simple local state. |
| AI | Own HTTP client in Rust per provider (`ai/`), local CLIs as a child process | The agent is a client of the app's MCP server; there is no parallel tool layer. |
| MCP | `rmcp` crate (official Rust SDK) | A single MCP server in the core: the internal agent uses it in process, external clients use the `treasury mcp` subcommand (stdio) of the same binary. See [ai-agent.md](ai-agent.md). |
| Tests | `cargo test`, Vitest, Playwright (E2E in the WebView) | |
| CI/Release | GitHub Actions (`ci.yml` on macOS, Ubuntu and Windows; `release.yml` for the Linux and Windows packages) | macOS is built and signed on the maintainer's Mac, because signing the update needs the YubiKey ([distribution.md](distribution.md)). |

### Alternatives considered

- **Electron + Node**: simpler for someone who already does web, but 150 MB+ binaries, more memory, and encryption/decimal math would live in JS. Discarded.
- **Wails (Go + web)**: a good candidate, small binaries; Tauri has a more mature plugin ecosystem (keychain, dialogs, deep link) and more mature Linux packaging.
- **Flutter desktop**: a beautiful single UI, but Linux desktop is less polished and there are fewer financial chart libraries.
- **Swift/AppKit + C++/Qt**: excellent native look, but two UI codebases to maintain. Not worth it for a one-person team.

## Overview

```
┌──────────────────────────────────────────────────────────────┐
│  Treasury.app (Tauri)                                        │
│                                                              │
│  ┌──────────── WebView: React + TS ───────────────────┐      │
│  │ Painel · Família · Contas · Lançamentos · …        │      │
│  │ Carteira · Ativos · Mercado · Agente · Ajustes     │      │
│  └───────────────▲────────────────────────────────────┘      │
│                  │ comandos Tauri (IPC tipado)               │
│  ┌───────────────┴──────────── núcleo Rust ────────────────┐ │
│  │ treasury-core (um módulo por área)                      │ │
│  │  ├─ api         (despachante call(cmd, args), política) │ │
│  │  ├─ changes     (conjuntos de alterações, desfazer)     │ │
│  │  ├─ transactions, accounts, budget, cards, debt, goals  │ │
│  │  ├─ portfolio   (operações, preço médio, TWR, XIRR)     │ │
│  │  ├─ indices, market, fx (índices, cotações, câmbio)     │ │
│  │  ├─ import, receipts, mail, ocr (dados que entram)      │ │
│  │  ├─ research, score (fundamentos, screener, score)      │ │
│  │  ├─ tax, tax_burden (IR, impostos pagos)                │ │
│  │  ├─ ai          (motores, agente, rotinas, memória)     │ │
│  │  ├─ mcp         (servidor MCP, chaves, auditoria)       │ │
│  │  ├─ vault, unlock, keychain (senha, desbloqueio)        │ │
│  │  └─ storage, backup (SQLCipher, migrações, backups)     │ │
│  └───────────────┬─────────────────────────────────────────┘ │
│                  │                                           │
│        treasury.db (SQLCipher, AES-256)                      │
└──────────────────┼───────────────────────────────────────────┘
                   │ socket local autenticado
           ┌───────┴────────┐
           │ treasury mcp   │  (subcomando do mesmo binário)
           │ stdio          │  ⇄ clientes MCP externos
           └────────────────┘
```

### Repository layout

```
treasury/
├─ apps/desktop/          # Tauri: src-tauri (Rust) + src (React/TS, componentes e tokens em src/design)
├─ crates/
│  ├─ treasury-core/      # domínio, cálculos, storage, mercado, IA, MCP (um módulo por área)
│  └─ treasury-dev/       # núcleo real por HTTP para a interface e o E2E, sem Tauri
├─ locales/               # catálogos Fluent (.ftl) e textos longos
├─ methodology/           # pesos, limiares e catálogos do score e do Mercado
├─ scripts/               # release, Linux e Windows por Docker, dados embutidos
├─ docs/
└─ .github/workflows/
```

The list of core modules and the conventions are in [development.md](development.md).

### Processes and concurrency

- **Single instance**: the app prevents a second window/process (Tauri's `single-instance` plugin) and holds an exclusive lock on the file, so there are never two writers.
- The app opens the database when the vault is unlocked and keeps one write connection. The journal stays in `DELETE`/`TRUNCATE` mode (not WAL), so at rest there is **a single file**; the temporary journal only exists during a transaction.
- Quote and index updates run in scheduled `tokio` tasks, with a rate limit per provider.
- **Every write** (from the user, an import or the AI) goes through the `changes` module, which groups the changes into a *change set* with author, reason and the before/after of each row. This gives undo, history and audit with a single mechanism.
- The `treasury mcp` subcommand does not open the database directly while the app is open: it forwards calls to the app over an authenticated local socket, so writes go through the same approval queue. With the app closed, it opens the database read-only using the keychain key (if enabled). With the vault locked, the socket refuses calls.
- **Background routines** (quotes, backups, agent routines) run while the app is open and unlocked. Closing the window can keep the app in the menu bar/tray (configurable), so the daily routines continue.

### On-disk storage

```
~/.treasury/                 (0700)
├─ data/
│  └─ treasury.db            banco único, SQLCipher (0600)
├─ backups/                  backups gerenciados (padrão; destino configurável)
│  └─ treasury-2026-10-14T03-00-00Z.db.enc
├─ config.toml               só o mínimo para iniciar (caminho do banco e dos backups, idioma); sem segredos
└─ logs/                     logs técnicos sem dados financeiros
```

- The same path on macOS, Linux and Windows (in the user's home folder).
- All other settings (AI, data sources, themes, budget) live **inside the database**.
- The path of the database and the backups folder can be changed in Settings (saved in `config.toml`, because it must be read before the database is opened); the default is the one above.
- The database opens with just the password and the file itself: no other file is needed to recover the data (see [security.md](security.md#encryption)).

### Managed backups

- **When**: automatic daily (configurable time), on closing the app if there were changes, before every schema migration, before large imports and before approved AI write batches, and manually through the button.
- **How**: a consistent copy made while the app is open. With the same database key, it uses SQLite's online backup API; with its own backup password, it uses `sqlcipher_export()` into an attached database with the other key. The backup is always a complete SQLCipher database that opens with the matching password.
- **Verification**: each backup is opened and runs `PRAGMA integrity_check` before it counts as valid; the SHA-256 hash is recorded.
- **Rotation** is configurable (default: 7 daily, 4 weekly, 12 monthly).
- Optional **extra destinations**: another folder (for example an external disk or an iCloud/Dropbox/Nextcloud synced folder), always encrypted.
- **Restore** from the interface: a list of backups with date, size and state; restoring first creates a backup of the current state.
- **Health**: the Settings panel shows the date of the last valid backup and warns if more than 48 h pass without one.

### Distribution

- **macOS**: `.app` and `.dmg` for Apple Silicon, with no dependencies (Intel is not supported).
- **Linux**: **AppImage** (a single portable file that bundles WebKitGTK) and also `.deb`/`.rpm` that use the system `webkit2gtk`.
- **Windows**: NSIS installer (per user, no administrator) and `.msi`, with the system WebView2. No code signing.
- The interface, migrations and SQLCipher are embedded in the executable; MCP is a subcommand, so there is no second binary.

## Data model (high level)

Conventions:
- **Monetary values** (balances, transactions, installments) are **integers in the smallest currency unit** (cents), always with `currency` (ISO 4217).
- **Asset quantities and unit prices** (fractions of foreign shares, crypto with 8 decimal places, Tesouro unit price, fund shares) are **exact decimals** (`rust_decimal`, stored as text), never floating point.
- Dates in ISO 8601; transaction dates have no time zone, technical instants are in UTC.
- Every row has `id` (UUID v7), `created_at`, `updated_at`.

### People and accounts
- **household**: the family (one vault = one family). Base currency (BRL).
- **member**: family members (holders).
- **institution**: banks, brokers, issuers.
- **account**: account (checking, savings, card, broker, cash, foreign, meal and food voucher), currency, holder(s), institution, opening balance. Broker accounts have their own cash balance: buys, sells and distributions move that cash. Benefit cards (`benefit`: Flash, Alelo, Pluxee, VR, Ticket, Caju) have their own balance: the employer's credits come in as income in Benefits › Meal and food vouchers (a ready-made rule) and purchases come out of the balance; Flash's CSV statement is recognized by its header (`flash` preset: values like "-R$ 40,90", date, time, merchant, and the value as an identifier to avoid duplicates, payee without the card-machine prefix or the country).
- **credit_card**: extension of the card-type account: limit, closing day, due day.
- **card_statement**: card statement (reference month, closing, due date, total, status, the `transaction` that paid it).

### Cash flow
- **category** (tree): income and expenses, with an editable starter set in Portuguese. Expenses: Housing, Food, Transportation, Health, Education, Leisure, Taxes and fees, Payments, Subscriptions, Pets, Gifts and donations, Kids… Income: work (salary, pro-labore, services and odd jobs, gig work, sales and own business), earnings, benefits, retirement and government aid, reimbursements, prizes and indemnities, gifts and inheritance, and money coming back from assets (investment redemptions, sale of assets, debt collection, loans received). Older vaults get what is missing as a suggestion in Settings › Categories (`categories.missing_defaults` and `add_defaults`, with the choice of which ones to add).
- **txn** (transaction): date, account, amount, currency, category, holder, description, status (planned, posted, reconciled), `kind` (income, expense, transfer, card statement payment, investment contribution/redemption, adjustment), `transfer_id` for transfers, `import_id` and `external_id` (e.g. the OFX `FITID`) to track the origin and avoid duplicates.
- **txn_split**: split of a transaction across several categories/holders (e.g. a supermarket purchase with part going to Pets).
- **tag** / **txn_tag**: free-form markers (e.g. "trip-2026").
- **recurrence**: repeat rule (RRULE) that generates planned transactions.
- **installment_plan**: installment purchase on a card.
- **budget** / **budget_line**: budget by month and category.
- **goal**: goals (emergency fund, trip, debt payoff), target amount, deadline, linked accounts or assets.
- **attachment**: receipts as blobs inside the database itself (encrypted by SQLCipher), keeping the single file.
- **rule**: automatic categorization rules.

### Debts
- **loan**: principal, rate, system (SAC/Price/custom), index, start date, term, grace period, debit account.
- **loan_installment**: planned installment (interest, amortization, insurance, fees) and the link to the `transaction` that paid it.

### Investments
- **asset**: ticker, name, class, market/exchange, currency, country, identifiers (ISIN, fund CNPJ, Tesouro code), preferred quote provider.
- **fixed_income_terms**: for fixed income: index, rate, maturity, grace period, liquidity, IR exemption.
- **trade**: buy/sell (date, asset, account, quantity, price, fees, taxes, exchange rate of the day).
- **corporate_action**: split, reverse split, stock bonus, subscription, merger, ticker change.
- **income_event**: dividends, JCP, FII income, coupons, share lending (gross amount, withheld tax, net).
- **target_allocation**: target allocation by class/asset, used in rebalancing alerts.
- **watchlist_item**: tracked assets, with target price and alerts.
- **dividend_event**: distributions announced by the market (ex-date, payment, amount per share), the basis of the calendar and the radar.
- **fundamental**: (issuer, item, period type, period end) → decimal value, currency, source, source document (DFP, ITR, FII report, SEC) and collection date. The issuer is the CNPJ (companies and FIIs) or the SEC CIK; the asset reaches the issuer through **research_security** (ticker ↔ CNPJ/CIK, from the FCA, the FII's ISIN or the SEC). Types: balance, fiscal year, year to date, month (FII) and trailing twelve months (calculated).
- **research_issuer**: public companies, FIIs and US companies (sector, segment, statement layout: industrial, bank, insurer, fund).
- **corporate_document**: material facts and notices (IPE/CVM) with a link and an optional agent summary.
- **research_quote** (quotes and volume of the universe, B3 COTAHIST), **research_universe** (daily indicators for rankings and the offline screener) and **research_file** (ETag of the collections): technical, outside the history.
- **screen**: saved screener filters (user data, with history and undo).

### Taxes
- IR rules are versioned by year in the code (`tax/rules.rs`); the monthly assessment (stocks, day trade, FIIs, foreign, loss carryforward) is calculated from the trades, with no table of its own.
- **darf**: generated DARF (revenue code, period, amount, due date, paid/unpaid, link to the payment transaction).
- **tax_category** (V109): the user's choice for the taxes-paid estimate: the category's tax group and its own rate ([taxes.md](taxes.md)).
- **tax_income_report** (V109): income statements from work by year and paying source (gross, INSS, IRRF, 13th salary), which replace the estimate based on net salary.

### Profile and derived data
- Profile and level of each member ([profile.md](profile.md)), used by the interface and the agent.
- **position_snapshot** (derived, recomputable): daily position per asset for fast charts.

### Market and indices
- **price**: (asset, date) → open, high, low, close, adjusted close, source.
- **fx_rate**: (pair, date) → rate, source (PTAX buy/sell).
- **index_series**: index definition (IPCA, IGP-M, CDI, Selic, US CPI, M2 BR, M2 US), frequency, source.
- **index_value**: (series, date) → value and/or change, marked **official** or **estimated** (Focus projection for months not yet published).
- **holiday**: holidays; the national ones that close the market (ANBIMA/B3 calendar) also come from `finance/calendar.rs`, for counting business days.

### AI
- **ai_engine**: configured engine: type (API, local CLI, local model), provider, default model, endpoint or command template, key reference.
- **ai_task_route**: for each task (categorize, PDF, chat, monthly close, alerts), the order of engines to try.
- **ai_usage**: tokens and estimated cost per call, engine and task; the basis of the spending limits.
- **secret**: secrets (API keys) encrypted with their own subkey. The sessions of the old assisted browser (`browser:session:*`) were deleted in V108, along with `browser_site` and `browser_capture`.
- **ai_conversation** / **ai_message**: optional agent history.
- **ai_proposal**: AI write proposals awaiting approval (linked to a pending `change_set`).
- **ai_audit_log**: every tool called by the AI (agent, CLI or external MCP client), with engine, parameters and a summarized result.
- **alert**: alerts and reminders generated by rules or by the agent, with a state (new, seen, resolved).

### System
- **change_set** / **change_item**: each change set (author: user, import, AI; reason; state: pending, applied, undone) and the before/after of each changed row.
- **import_batch**: imported file, hash, count, errors.
- **backup_record**: backups made, destination, size, hash, verification result.
- **setting**, **schema_migration**.

## Core calculations

### Average price and result
- Weighted average price per purchase (Receita Federal rule), including brokerage and exchange fees.
- Sales realize profit/loss against the average price; the FIFO option is available for foreign-asset reports.
- Corporate events adjust quantity and average price retroactively.
- Positions are valued at the **close not adjusted for distributions** (adjusted only for splits and reverse splits), because distributions are already recorded as `income_event`; using the adjusted close would count dividends twice.

### Fixed income
- Post-fixed instruments at a % of the CDI accrue through the daily product `(1 + CDI_day × percentage)` in **business days** (base 252, ANBIMA calendar).
- IPCA+ and fixed-rate instruments follow market conventions (base 252 business days).
- The position shows the **gross** value and the **estimated net** value after regressive IR (22.5% up to 180 days → 15% above 720 days) and IOF in the first 30 days, respecting exemptions (LCI, LCA, CRI, CRA, incentivized debentures). This is a redemption estimate, not the IR assessment.

### Returns
- **TWR** (time-weighted): compares with benchmarks, neutralizes contributions and redemptions.
- **XIRR** (money-weighted): the investor's effective return considering the dates and amounts of each flow.
- Both by asset, class, account and portfolio, in BRL and in the original currency.

### Real return
For a flow invested at date *t₀* and valued at *t₁*:

```
retorno_real = (1 + retorno_nominal) / (1 + inflação[t₀→t₁]) − 1
```

- **IPCA deflator** (default for BRL), **US CPI** (for USD assets in the original currency), **M2** (money supply expansion of Brazil or the US) as an alternative measure of currency "dilution", or the reference **country's inflation** (country catalog: HICP, OECD CPI, ONS, Bank of Canada or, as a worldwide fallback, the World Bank annual inflation, interpolated and estimated; in Brazil it is the IPCA). Sources in [data-sources.md](data-sources.md).
- For multiple flows, each contribution and redemption is brought to the values of a base date by the chosen deflator before the real XIRR is calculated.
- The real result in today's currency appears next to the nominal one on every portfolio screen.
- Monthly indices (IPCA, CPI) are spread within the month geometrically pro rata; months not yet published use the Focus projection (or the last data point) and the value is marked **estimated** until the official index is out, when it is recalculated.
- Foreign-currency assets: the BRL result includes the exchange-rate variation (PTAX) and is deflated by the IPCA; in the original currency, by the CPI.

### Debts
- SAC and Price schedule generation, with adjustment by the contract's index.
- Charges beyond interest: home insurance (MIP and DFI), administration fee, IOF; the CET stated in the contract is recorded and compared with the realized one.
- Realized total effective cost (interest + insurance + fees) and comparison with inflation over the period (the debt's "real cost").
- Contract total effective cost computed by Res. CMN 4.881/2020 (upfront IOF and fees out of the amount released, floating index left out), next to the one typed from the contract, with the US APR and the local name per country (design/loan-cet.md).

### Cash flow
- Transfers between accounts, card statement payments and investment contributions/redemptions do **not** count as income or expense; card spending counts at purchase.
- Budget and reports can be viewed on an **accrual** basis (purchase date) or a **cash** basis (card statement payment date), configurable.
- **Savings rate** = (income − expenses) ÷ income of the period.
- **Cash flow drill-down** (`reports.sankey_items`): each value in the Sankey and the comparison (month, previous and Δ) opens the transactions that make it up, to the cent, by the same rules as the diagram: one item per transaction and category (parts of a split kept separate), converted at the transaction date, with the card statement due date on the cash basis. Groups the diagram leaves out (negative balance from a refund) and currencies without an exchange rate appear in "Outside the total", with the reason; surplus and deficit list the lines that enter the calculation. Tests check, line by line, that the sum of the items is the value of the cell.

## Quality and tests

- Financial calculations have tests with known reference values (e.g. IPCA adjustment checked against the Banco Central Citizen's Calculator; SAC/Price schedules checked against spreadsheets).
- Money invariants are checked on demo vaults (the same number through different paths of the core; [invariants.md](invariants.md)).
- Migrations are tested against real vaults from earlier versions (the 0.9.38 vault, whose history becomes V1, and the 0.1.0 vault, which is refused; `api::tests_upgrade`) and against the published ones frozen in `migrations.lock` ([development.md](development.md)); backup → restore has tests in the core.
- MCP tools have contract tests (input, output and permission policy).
