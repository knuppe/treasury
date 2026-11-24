# Investments

Treasury's portfolio and asset analysis: what the family tracks, which indicators the app
calculates, where the data comes from and how risk and retirement work. Everything is calculated on the
computer, with the vault's data and the public sources listed in
[data-sources.md](data-sources.md).

## What it covers

| Feature | How it works |
|---|---|
| Portfolio (net worth, invested, income, performance, by class) | Positions per asset with average price, TWR, XIRR and real result ([real-return.md](real-return.md)) |
| B3 trades | Import of the reports from the **B3 Investor Area** (trades, movements, position) in Excel; with the Downloads folder watched, the downloaded report already opens the import. Brokerage notes in PDF (SINACOR standard, password-protected) complete the trades with fees, exchange charges and IRRF apportioned. B3's official API for investor data is restricted to accredited institutions. |
| Portfolio analysis | The manager agent points out concentration, deviation from target, real result and opportunity cost against the CDI, with traceable recommendations ([concentration.md](concentration.md)) |
| Asset page | Indicators, calculated price, income, documents, news and a conversation with the agent ([asset-page.md](asset-page.md)) |
| Calculated price | Graham: `√(22,5 × LPA × VPA)`; Bazin: `média de dividendos por ação ÷ taxa mínima` (default 6%, configurable), with the assumptions visible |
| Target price | Per asset, with an alert |
| Rankings and screener | Generated on the computer over the downloaded universe; any indicator becomes a ranking; combinable filters saved as "screens"; the agent also builds filters from natural language |
| Comparator | 2 to 6 assets side by side, with indicators and normalized price |
| Income received | Per distribution, by the position on the record date and the official notice (Fundos.NET for funds, B3 for companies), against what is recorded; totals since the first purchase, fixes in one change and automatic recording ([income-ledger.md](brazil/income-ledger.md)) |
| Dividend calendar and radar | Ex-date, payment and amount per share, filterable to "only my portfolio"; projection from the payment history, marked as **estimated** |
| Passive income | Projected income of the current portfolio and a monthly income goal |
| Material facts and notices | CVM IPE documents for the assets in the portfolio and the watchlist; the agent summarizes the new ones |
| Watchlist and alerts | Price, target price, ex-date, material fact, negative real result, rebalancing |
| IR | Monthly calculation, DARF, assets and rights, exempt and taxable income ([taxes.md](taxes.md)) |
| Sales | Sales ledger with nominal and real gain or loss and IRR per sale ([sales-real-return.md](sales-real-return.md), [cost-method.md](cost-method.md)) |
| Scheduled contributions and goals | [scheduled-contributions.md](scheduled-contributions.md) |
| Health score per asset | [asset-score.md](asset-score.md) |

Out of scope: recommended portfolios, analyst reports, courses and community (editorial
content, not a tool).

## Indicators per asset

**Stocks (BR) and stocks (US)**
- Valuation: P/E, P/B, EV/EBITDA, EV/EBIT, P/Sales (PSR), P/Assets, Dividend Yield, market cap, enterprise value.
- Profitability: ROE, ROIC, ROA, gross margin, EBITDA margin, net margin.
- Debt: net debt/equity, net debt/EBITDA, current ratio.
- Growth: 5-year revenue and profit CAGR.
- Per share: EPS, book value per share, payout.
- History: revenue, profit, dividends and price over 10 years; indicators over time.
- Banks: Basel Index and sector-specific indicators.

**FIIs (Brazilian real estate funds)**
- P/B, 12-month DY, last yield, net asset value per unit, daily liquidity, number of unit holders, vacancy (when reported), type and segment, yield history.

**ETFs, BDRs, REITs, funds, Tesouro, fixed income, crypto**
- Price and performance over standard windows, volatility, net assets (funds), rate and maturity (Tesouro/fixed income), market cap (crypto).

Each indicator shows the formula, the period (last 12 months or last fiscal year) and the source, like the rest of the app.

How the core calculates (`crates/treasury-core/src/research`; formulas, input validation and the universe audit in [indicators.md](indicators.md)): profit and equity are those of the controlling shareholders (net margin and ROA use the consolidated total); flows are for the last 12 months (DFP + ITR: previous year + year to date − year to date of the previous year); consolidated when the company publishes it, otherwise standalone. Bank indicators (Basel, Tier 1 capital, leverage, delinquency) come from the Central Bank's IF.data (`research/ifdata.rs`), and FII vacancy from the CVM monthly report.

## World exchanges

An asset's "market" is the code of an exchange in the catalog (`exchange`, seeded by V85 and editable in Assets › Exchanges). The exchange provides the registry's default currency and country, the Yahoo symbol (ticker + suffix: `PETR4.SA`, `VOD.L`, `7203.T`; in the US no suffix and with a hyphen: `BRK-B`) and the intraday collection hours (regular session in the exchange's time zone, with daylight saving time and lunch break; holidays are not in the catalog). Time zones come from our own table (`market/zone.rs`), with about 200 IANA names for the financial centers and the daylight saving rules in force: an exchange registered by hand must use one of these names, and an instant before a country's rule change comes out with today's rule.

| Region | Exchanges (code · Yahoo suffix) |
|---|---|
| Brazil | B3 `.SA` |
| US | NYSE, NASDAQ, AMEX (NYSE American), ARCA (NYSE Arca), BATS (Cboe) — no suffix |
| Canada and Latin America | TSX `.TO`, TSXV `.V`, BMV (Mexico) `.MX`, BYMA (Buenos Aires) `.BA`, Santiago Exchange `.SN` |
| Europe | LSE `.L`, Xetra `.DE`, Frankfurt `.F`, Euronext Paris `.PA`, Amsterdam `.AS`, Brussels `.BR`, Lisbon `.LS`, Dublin `.IR`, Milan `.MI`, Madrid `.MC`, SIX `.SW`, Vienna `.VI`, Stockholm `.ST`, Copenhagen `.CO`, Helsinki `.HE`, Oslo `.OL`, Warsaw `.WA`, Istanbul `.IS` |
| Africa and Middle East | JSE `.JO`, Tel Aviv `.TA`, Tadawul `.SR` (Sunday to Thursday) |
| Asia | Tokyo `.T`, Hong Kong `.HK`, Shanghai `.SS`, Shenzhen `.SZ`, NSE `.NS`, BSE `.BO`, KRX `.KS`, KOSDAQ `.KQ`, Taiwan `.TW`, TPEx `.TWO`, SGX `.SI`, Indonesia `.JK`, Thailand `.BK` |
| Oceania | ASX `.AX`, NZX `.NZ` |

Classes by exchange:

- **B3**: `stock_br`, `fii`, `etf_br`, `bdr` (unchanged; monthly variable-income calculation).
- **US**: `stock_us`, `etf_us`, `reit` ("US stocks", "US ETFs"): SEC fundamentals, presumed 30% withholding on dividends, investment abroad (Lei 14.754/2023), country "United States" in Assets and Rights — as before.
- **Other exchanges**: `stock_intl` and `etf_intl` ("Stocks/ETFs on other exchanges"), which require an exchange from the catalog. They count as investment abroad (realized gain and income in the annual adjustment of Lei 14.754/2023, outside the monthly calculation and the DARF), group 03.01 in Assets and Rights with the exchange's country, no presumed withholding on dividends (it depends on the country and the treaty: report the IR withheld) and no fundamentals (price only). In the portfolio, reports and goals they are classes like the others, with their own color.

Exchange rates: currencies with a PTAX rate (AUD, CAD, CHF, DKK, EUR, GBP, JPY, NOK, SEK, USD) are automatic; the others need a manual rate (see [data-sources.md](data-sources.md)).

## Sources for fundamentals

| Data | Source |
|---|---|
| Statements of listed companies (annual DFP, quarterly ITR) | CVM Open Data |
| Prudential indicators of banks (Basel, capital, delinquency) | Central Bank, IF.data |
| Material facts and notices (IPE) | CVM Open Data |
| Monthly report of FIIs | CVM Open Data |
| Fundamentals of US companies | SEC EDGAR, XBRL `companyfacts` API (free, requires an identified User-Agent) |
| Income and prices | see [data-sources.md](data-sources.md) |

The full universe (all listed stocks and FIIs) is downloaded and calculated locally once a day, so rankings and the screener work offline.

## Portfolio risk

Portfolio › Risk (`portfolio.risk`). Basis: the monthly return of each asset held today (TWR by Modified Dietz, with income, in reais) over the last 12 to 60 complete months. The portfolio and the classes are the average weighted by today's weights ("today's portfolio in the past"); in a month when an asset did not yet exist, its weight goes to the others. Alongside, the portfolio's actual history (with the purchases and sales of each period).

- Volatility = monthly standard deviation × √12; geometric annual return; Sharpe = mean monthly excess over the CDI ÷ monthly standard deviation × √12; worst drawdown = largest drop of the cumulative value from the previous peak, with the month of the peak, the trough and the recovery.
- Pearson correlation with at least 12 months in common, between classes and among the 15 largest assets.
- Share of risk = weight × covariance with the portfolio ÷ portfolio variance (the shares add up to 1); the screen marks the lines that weigh much more in risk than in value.
- Assets with less than 12 months of history have no metrics of their own and stay out of the correlation (the note lists which).

## Retirement

Budget › Retirement (`goals.retirement`): Monte Carlo in real values with the portfolio (includes pension plans), contributions, desired spending, INSS and life events. It shows the chance the money lasts, the spending with a 90% chance and the net worth needed at stopping (90% success from then on).

- **INSS by the rules** (`inss_calc`, `goals.inss_estimate`): sex, birth, contribution time today and at the reform (2019-11-13), average of the salaries since 1994-07 and until when the person contributes. Rules of EC 103/2019: permanent (65/62 years with 20/15 of contribution, for those who started after the reform), transition by age, points, progressive minimum age, 50% toll (the date only; the amount depends on the pension factor) and 100% toll. Benefit of 60% of the average plus 2% per year above 20 (men) or 15 (women), 100% in the 100% toll, between the minimum and the ceiling in the year's table (`goals::inss::TABLE`, update every January with the interministerial ordinance). Age counts by the calendar (exact on the birthday) and the date of each rule comes out on the day it is met; the coefficient uses complete years. The simulator uses the rule that starts paying earliest at the plan's age and adds the 13 payments of the year (annual bonus). Checked against a Meu INSS simulation of 2026-10-21 (test `matches_the_meu_inss_simulation_of_a_man_who_joined_before_the_reform`). Out: special activity, teacher, person with a disability and acquired right.
- **Outside Brazil** (`country::pension_of`): in the US, `us_ssa_calc` (`goals::pension_us`): PIA from the SSA statement or AIME by the 90/32/15% formula at the bend points (`TABLE`, update every October), full retirement age by birth year (66 to 67), minus 5/9% per month in the 36 months before it and 5/12% before that (from age 62), plus 2/3% per month after it up to 70, 40 credits. In Portugal, `pt_pension_calc` (`goals::pension_pt`, from the ISS 2026 Guia Prático Pensão de Velhice): 15 years guarantee, normal age of 66y9m in 2026 and 66y11m in 2027 (kept afterward), minus 4 months per career year above 40, RR × global rate (2% up to 20 years; 2.3% to 2% by IAS brackets), early retirement by flexibilization (−0.5% per month, sustainability factor if the person did not have 40 years at 60), very long careers, bonus up to 70 (up to 92% of the RR) and minimum pension by career; 14 payments a year. In other countries, only the amount entered, with the generic name.
- **Scenarios** (`retirement_scenarios`, `retirement_scenario_save`, `retirement_scenario_delete`): up to 20 named plans besides the main one, compared side by side.
- **Goal linked to the plan** (`retirement_link_goal`, `retirement_unlink_goal`): a retirement-type goal with target = net worth needed at stopping, date = the retirement date and contribution = the plan's; saving the main plan updates the goal, and without linked accounts its current value is the portfolio.

## Integration with the rest of the app

- **Real result** of each asset and of the portfolio, discounting the country's inflation, CPI or M2.
- **Portfolio inside the financial life**: contributions come from the budget, goals use the portfolio, and the
  agent crosses spending and investments ("if you cut R$ 400/month on delivery, the passive
  income goal arrives 2 years earlier").
- **Local, encrypted data**, no account on a third-party platform.
- **An agent that acts**: records trades from brokerage notes or the Investor Area,
  always with approval.
