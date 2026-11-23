# Fundamental indicators

Treasury calculates the fundamental indicators of Brazilian companies (P/E, P/B, ROE, ROIC,
margins, EV/EBITDA, debt ratios, dividend yield and others) from official sources only: the CVM
open data files and the B3 listed-companies API and quotes. Each indicator comes with its formula,
period, the documents it used and the result of the **input check**: verified, corrected with
another official source, with a warning, or blocked. They appear in the asset page (Indicators
tab), the comparator, the screener, rankings, the Market screen and the MCP tools
(`get_fundamentals`, `rank_assets`, `run_screen`, `compare_assets`).

Third-party sites (Fundamentus, brapi.dev) are never a source of these numbers. When the person
turns them on, they are only used to cross-check, in the asset health score
([asset-score.md](asset-score.md)).

## Sources

- **CVM** ([data-sources.md](data-sources.md)): DFP (annual), ITR (quarterly), both in the
  consolidated and the parent-only (individual) versions, the capital composition (shares and
  treasury shares) and the FCA (registry: tickers, classes, units, sector).
- **B3**: COTAHIST (daily quotes) and the listed-companies API: cash dividends, stock events
  (splits, reverse splits, bonus shares), the capital stock share count, and the registry of a
  company (`GetDetail`) when the FCA leaves its tickers blank or invalid.

## Conventions

- **Last 12 months.** Flows (revenue, EBIT, net income, cash flow) are the last 12 months:
  previous fiscal year + year to date − same period of the previous year. The previous-year
  period uses the figures as published at the time; a later restatement is flagged (E7).
- **One balance sheet.** All balances of an indicator come from the latest balance sheet (the
  latest date of total assets and equity). An account missing from it makes the indicator
  unavailable; it is never taken from an older balance sheet. The share count is the exception:
  without a capital composition in the latest filing, the previous one holds.
- **Owners and consolidated total.** P/E, EPS, ROE and Graham use the net income and equity
  attributable to the owners of the parent. Net margin and ROA use the consolidated net income,
  including non-controlling interests, because revenue and assets are 100% of the consolidated
  group. EV and the ROIC capital include non-controlling interests, because EBITDA and EBIT are of
  the whole group.
- **Shares.** Outstanding shares: paid-in capital shares minus treasury shares, from the CVM
  capital composition, checked against the B3 capital stock count (E4). Market cap adds each class
  at its own quote when all classes have one; otherwise outstanding shares × the security's quote
  (per share, for units).
- **EBIT.** Line 3.05 of the CVM income statement, "result before financial result and taxes"
  (the definition of CVM Resolution 156). It includes other operating income and expenses and the
  equity method.
- **Banks.** Net income and equity from the parent-only statements, which follow the Central Bank
  accounting standard used by the market and for dividends; the consolidated IFRS figures appear in
  a note. EBITDA, EV, net debt, margins and current ratio do not apply.
- **Holding companies and insurers** (equity-method income above half of EBIT, or insurer by the
  FCA sector with an industrial-format statement): EBIT, EV, margins and the operating multiples do
  not apply; P/E, P/B, ROE and debt do.
- Ratios as fractions (0.083 = 8.3%), multiples in times, money in the asset's currency.

## Indicators

Market and valuation:

- **Market cap**: shares × quote (by class when possible).
- **EV**: market cap + gross debt − cash and short-term investments + non-controlling interests.
- **P/E**: market cap ÷ net income attributable to owners (12 months).
- **P/B**: market cap ÷ equity attributable to owners.
- **P/Sales (PSR)**: market cap ÷ net revenue (12 months).
- **P/Assets**: market cap ÷ total assets.
- **P/EBIT**: market cap ÷ EBIT (12 months).
- **P/Working capital**: market cap ÷ (current assets − current liabilities).
- **P/Net current assets**: market cap ÷ (current assets − total liabilities), Graham's "net-net".
- **EV/EBITDA** and **EV/EBIT**: EV ÷ (EBIT + depreciation and amortization from the value added
  statement, or the cash flow statement) and EV ÷ EBIT.
- **Earnings yield (EBIT/EV)**, **FCF yield** and **PEG**.

Profitability:

- **ROE**: net income attributable to owners ÷ equity attributable to owners.
- **ROA**: consolidated net income ÷ total assets.
- **ROIC**: EBIT × (1 − 34%) ÷ (equity + non-controlling interests + net debt). Unavailable when the
  capital is below 10% of total assets (a large net cash position makes it meaningless).
- **Gross, EBITDA, EBIT and net margins**: each over net revenue; net margin with the consolidated
  net income.
- **EBIT/Assets** and **asset turnover** (net revenue ÷ total assets).
- **ROC (Magic Formula)**: EBIT ÷ (net working capital + property, plant and equipment).

Debt: **net debt ÷ equity**, **gross debt ÷ equity**, **net debt ÷ EBITDA**, **current ratio**.

Growth: **revenue and earnings CAGR** over 5 fiscal years, **loss years** in 10, **EPS growth** in
10 years (average of the last 3 over the first 3).

Per share and dividends: **EPS**, **BVPS**, **dividend yield** (cash dividends with record date in
the last 12 months ÷ quote), **payout**, **years with dividends** and **dividend CAGR**.

Fair price: **Graham** and **Bazin**, with the assumptions shown ([investments.md](investments.md)).

## Input validation

The CVM checks the accounting identities of a filing (assets = liabilities + equity, subtotals of
the income statement, cash flow), but not the fields a company types by hand. In the November 2026
study, 22% of the consolidated ITRs had the "net income attributable to owners" line at zero, about
130 of 305 companies reported the share count in thousands without saying so, and the FCA had
invalid tickers and wrong end-of-trading dates. The core checks each input before using it.

Each indicator carries a state (`check`) and notes (`check_notes`):

- **verified**: its inputs passed the checks;
- **corrected**: an input failed and another official source took its place (the notes say which);
- **warning**: the value was calculated, but there is an unexplained inconsistency;
- **blocked**: an input is invalid and no source can fix it; the indicator has no value and the
  reason says why;
- **unchecked**: no value (missing data or not applicable).

The rules:

- **E1. Filing structure.** The identities the CVM guarantees; a broken file is not used.
- **E2. Net income attributable to owners.** The owners' line when it is filled; when it is zero
  but the total (minus non-controlling interests) is not, the net income of the parent-only
  statements for the same period (by the equity method, it is the owners' income); without those,
  the total minus non-controlling interests. Consolidated owners' income and parent-only income
  differing by more than 1% gives a warning; by more than 20% (and R$ 1 million), the indicators
  that use it are blocked.
- **E3. Equity attributable to owners.** Checked against the parent-only equity with the same
  limits.
- **E4. Share count.** Checked against the B3 capital stock count (which includes treasury
  shares): equal within 0.5% is verified; a factor of about 1,000 fixes the scale; a CVM total that
  already excludes treasury shares is corrected; stock events between the balance sheet and the
  B3 date explain a difference; an unexplained difference up to 10% is a warning, above 10% blocks.
  Without the B3 count yet, the scale is checked against net income ÷ EPS of the same filing, and
  then against the P/B.
- **E5. Registry.** The share class comes from the ticker suffix (3 = common; 4 to 8 = preferred;
  11 = unit), not from the FCA. A company with shares in the FCA but no valid ticker, or only its
  unit, is completed with the B3 registry. A ticker traded in the last 30 days is never treated as
  delisted. Unit composition counts only quantities ("1 KLBN3 + 4 KLBN4" = 5).
- **E6. Dates.** One balance sheet per indicator; a balance sheet older than 200 days gives a
  warning.
- **E7. Restatements.** When the comparative in the next filing differs by more than 0.5% from the
  figure published at the time, the indicator says so (a blank comparative does not count).
- **E8. Quote.** Age and class of the quote; per-share quote of units by the validated
  composition.
- **E9. Dividends.** The dividend yield uses official dividends (B3, or the asset's calendar
  checked against B3). Without them, an estimate from dividends paid in the cash flow statement is
  shown with a warning and left out of rankings and the screener. Opening a security without
  official dividends asks B3 once (`research.b3_refresh`).
- **E10. Applicability.** Banks, insurers and holding companies (above).
- **E11. Plausibility.** Values outside wide ranges (gross margin above 100%, ROIC above 200%, P/B
  below 0.02, dividend yield above 50%…) are not blocked, since they may be true; they go into a
  single item on the Developer screen.

## How the calculation is verified

- **Formula tests**: each indicator against a hand calculation, with edge cases
  (`research/indicators/tests.rs`).
- **Real cases**: rows cut from the CVM files (`research/fixtures/cvm/`) for Klabin, Taesa, CVC,
  Itaú Unibanco and WEG, with values recomputed independently (`research/indicators/real_tests.rs`).
- **Universe audit**: `python3 scripts/indicators-audit.py` downloads the CVM files, recomputes
  every company outside the app with the same rules and compares with a `treasury-dev serve`
  vault: extraction (each base value), formulas (each indicator from the core's own facts), the
  check states and the input validity rates. It runs before a release that touches
  `research/` and whenever `PARSER_VERSION` goes up. In November 2026: 325 companies with no
  extraction difference and 9,806 indicator values with no formula difference.

## Why the numbers differ from other sites

With the same CVM data, other sites differ from Treasury by convention, not by data:

- share count: the B3 total including treasury shares (Treasury excludes them);
- EBIT: gross profit − selling − general and administrative expenses (Treasury uses line 3.05,
  which also has other operating items and the equity method); this moves EBIT margin, EV/EBITDA
  and ROIC by 14% at the median, more in companies with large other operating expenses;
- ROIC: EBIT ÷ (assets − suppliers − cash), without tax;
- net margin: some use the consolidated total (like Treasury), but the owners' income in P/E;
- EV without non-controlling interests;
- market cap by the quote of the security, not by class;
- banks: some sites mix parent-only income with parent-only assets (the holding's, not the
  bank's);
- revenue growth over a different window.

The health score's cross-check redoes Treasury's numbers in Fundamentus's convention before
comparing ([asset-score.md](asset-score.md)), so a remaining difference points to a data error on
one side.

## In the code

- `crates/treasury-core/src/research/cvm.rs`: reading of DFP, ITR and FCA; owners' income rule
  (`controllers_income`), parent-only income and equity, share totals, restated comparatives,
  share class by ticker and incomplete registries.
- `research/b3_listados.rs`: capital stock shares (`store_shares`), registry completion
  (`detail_securities`, `GetDetail`) and the on-demand refresh (`refresh_company`).
- `research/indicators.rs`: catalog, formulas, input checks (`InputCheck`, `check_*`,
  `reconcile_shares`).
- `research/sync.rs`: order of the collection (quotes before statements), reactivation of traded
  tickers, `PARSER_VERSION` 7.
- `research/screen.rs`: universe without estimated dividends, plausibility report.
- `score/equity.rs`: `fundamentus_convention` and the cross-check.
- Commands: `research.indicators`, `research.fundamentals`, `research.b3_refresh`. No migration:
  the new values are items in `fundamental`.

## Limitations

- Only Brazilian companies get the input checks; US companies (SEC) and FIIs keep their rules.
- The B3 capital count covers the whole universe in about a week (60 companies a day plus the
  registered ones); until then the share check falls back to the filing's EPS and the P/B.
- A difference between the B3 count and the balance sheet without a stock event (for example,
  shares cancelled after the balance sheet) is a warning; the app does not read the minutes that
  would explain it.
- Lease liabilities (IFRS 16) are not in EV or net debt.
- When the consolidated owners' line is zero in some periods only, the 12 months mix the
  consolidated line with parent-only figures; they are equal under the equity method, except in
  rare cases flagged by E2.
