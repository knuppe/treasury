# Assumptions (methodology 0.1.0, 2026-10-16)

| Assumption | Value | Rationale |
|---|---|---|
| Tax rate for NOPAT (ROIC) | 34% (IR 25% + CSLL 9%) | Nominal rate for non-financial companies in Brazil |
| Banks' cost of equity (Ke) and growth (g) in the fair P/B | Ke 16%, g 5% | Selic/long NTN-B + equity premium; long-term nominal growth. Review against the curve |
| EBITDA definition | EBIT from the income statement (DRE) + D&A from the value added statement (DVA) (or the cash flow statement, DFC) | Recomputed from the statements, without the company's "adjusted" figure |
| Leases (IFRS 16) | Added to debt in all sectors | Same convention for all peers; "lease" lines from 2.01.05 and 2.02.02 (those in 2.01.04 are already in debt) |
| Retained earnings (Altman Z'') | Profit reserves + accumulated profits/losses | Retained earnings do not exist as an account in the Brazilian standard |
| Cyclical normalization window | Average of 5 fiscal years of EBITDA and ROIC | §5.2; the DFP (annual financial statements filing) goes back 10 fiscal years |
| Minimum peer group | 8, widening sector → segment → class | §7.3 |
| Winsorization | 2.5% and 97.5% | §7.3 |
| Relative × absolute | 0.7 × percentile + 0.3 × anchor | §7.3 (calibrate in the backtest) |
| Stale data | > 200 days (≈ 2 quarters) | §7.5 |
| Banks: 12-month results in IF.data | Semester to date: Jun/Dec = semester + previous semester; Mar/Sep = quarter + previous semester + quarter from 9 months earlier − quarter from 12 months earlier | Convention of the balance sheets (IF.data resets the result every semester) |
| FII: WALT | Midpoint of the maturity ranges in the quarterly report; "above 36 months" = 5 years; "indefinite" excluded | The report only gives ranges |
| FII: declared distributions | Semester accumulated in the report (2nd and 4th quarters = difference from the previous one when it matches) | Observed in the 2025 files (≈ 68% of funds follow the pattern) |
| FII: management fee | Average of the last 12 monthly reports × 12 | The report gives the month's expense over net assets |
| FII: classification | CRI ≥ 50% of assets = paper; FII shares ≥ 50% = FoF; real estate ≥ 50% = brick; otherwise hybrid | Composition from the monthly report, more reliable than the declared "segment of activity" |
| Filing deadline at the CVM | DFP: 3 months after the fiscal year; ITR: 45 days after the quarter | CVM Resolution 80 |
| Auditor change outside the rotation | Previous auditor with less than 5 years | Mandatory 5-year rotation (10 with an audit committee) |
| Primary source per field | See docs/asset-score.md › Sources | Fundamentus only for cross-checking (§9.2) |

## Extension: foreign stocks, REITs and BDRs (2026-10-16)

| Assumption | Value | Rationale |
|---|---|---|
| Source | SEC EDGAR (`companyfacts` + `submissions` for the SIC) | Free and primary; requires a User-Agent with an email |
| Segment | SIC ranges (`aliases.sic_segment` in `classes/intl.json`) | The SIC is the only structured sector from the SEC |
| Anchors | ROIC/ROE/yields lower than Brazil's | Lower cost of capital in dollars (Treasury + premium) |
| Governance | No dimension (weight redistributed) | There is no listing segment or free structured source |
| Missing EBIT | Pre-tax income + financial expenses − financial income | REITs and several companies do not report `OperatingIncomeLoss` |
| Debt | `LongTermDebt`, otherwise `NotesPayable`/`SeniorNotes`/`UnsecuredDebt` | REITs report debt as notes payable |
| REITs | Net debt/EBITDA up to ~5.5x and coverage of 3x are normal | Capital-intensive sector; FFO/AFFO still has no standardized source |
| US banks and insurers | No prudential indicators | FDIC/Fed not integrated; score by ROE, valuation and stability |
| BDR | Inherits the underlying's score (table `aliases.bdr_underlying`, otherwise the ticker without the digits) | Inheritance rule of §7.2 |
| Exchanges outside Brazil and the US | No score (critical data missing) | There is no free source of financial statements |
| Peers | Only the registered US companies: usually few, so the engine uses the absolute anchor and lowers confidence | Building the whole SEC universe (frames) is left for later |

## Fixed income (extension `classes/fixed_income.json`)

| Assumption | Value | Rationale |
|---|---|---|
| Sovereign credit risk | Fixed score of 90 (value 1 = National Treasury in reais, anchor 1 → 90) | Local-currency debt, no conversion risk; there is no free rating feed. Not 100 so as not to equate it with zero risk |
| Weights | Spec with issuer health at 35 (not 40): bank FGC 30 (spec 25), private credit spread 25 (spec 20) | Cap of 35 per dimension in the methodology; keeps the total credit weight (65 for bank) |
| Treasury criticals | Percentile of today's rate over 10 years and duration | Without a historical rate there is no attractiveness (30% of the weight) and without maturity there is no horizon (25%) |
| Bank criticals | The bank's Basel index and FGC coverage | Inheriting the bank's health is the essence of the security; Basel is the only indicator published for all (IF.data report 5); the FGC is the other half of the credit risk |
| Private credit critical | Issuer health (Health sub-score of the stock score) | Inheritance from §7.2; without the company at the CVM there is no way to measure the risk |
| Savings account (poupança) | Bank profile with the account's bank (FGC covers) | Same credit risk as a CDB; the rate follows the legal rule (0.5% per month + TR when Selic > 8.5%; otherwise 70% of Selic + TR) |
| "Other" security | No FGC metric (critical): "Insufficient data" | It may be an LF (no FGC) or a guaranteed security; the registry does not say |
| Rate history | Daily median of the purchase rates by family (IPCA+ with Renda+ and Educa+; fixed-rate), securities more than one year from maturity, 10 years | One vertex per day without depending on the term; securities near maturity have meaningless rates (e.g., −1.78%) |
| Security's rate today | Morning purchase rate (sale rate if the security is not on sale) | It is the rate locked in by buying today |
| Tesouro Selic | Real Selic yield: (1 + target Selic + premium) ÷ (1 + 12-month IPCA) − 1, against 120 month-ends | The premium does not measure attractiveness; the real yield does |
| Real rate of the fixed-rate security | (1 + rate) ÷ (1 + 12-month IPCA) − 1 | Past IPCA as an approximation of inflation; the curve's implied one would give, by construction, the IPCA+ rate |
| Duration | Macaulay: no coupon = term; semiannual coupon 10% (fixed-rate) and 6% (IPCA+ and IGP-M+); Renda+ 240 and Educa+ 60 monthly installments to maturity; floating-rate ≈ 0; private credit = term (no schedule) | Rules of the Treasury securities; debenture indentures have no structured source |
| Horizon | No investor horizon: duration and price (PU) volatility measure the mark to market | The registry does not link a security to a goal |
| Custody | B3 0.20% p.a.; Tesouro Selic exempt up to R$ 10 thousand (by the security's position) | B3 rule for Tesouro Direto |
| Rate vs CDI | Floating: % contracted; fixed: ratio of daily rates against the Tesouro Prefixado of a similar term (no curve: today's CDI); IPCA+: nominal with the curve's implied inflation at the same term; exempt ÷ (1 − IR for the investment term); savings compared with 22.5% IR | Taxable equivalent to compare exempt and taxed; the Treasury curve is the best free estimate of the future CDI. There is no free base of CDB rates from issuers of the same risk |
| Private credit spread | Contracted rate − Treasury of a similar term and same index; % of the CDI: (% − 100%) × today's CDI; no gross-up of exempt ones | The exemption already weighs in its own dimension; ANBIMA (indicative rates) has reserved rights and only 5 business days on the site |
| FGC | R$ 250 thousand per CPF per prudential conglomerate; R$ 1 million per CPF cap applied proportionally when the covered sum exceeds it; joint account split among holders; accounts without a holder = one CPF | FGC rules; the global cap applies to payments over 4 years, treated here conservatively |
| Issuer by name | Normalized name without generic words equal to that of a single conglomerate (IF.data) or company (CVM) | Conservative: an ambiguous name is left without an issuer (e.g., two banks "Master") |
| Institution out of operation | Status other than "A" in the latest IF.data registry (liquidation, intervention, merger) | IF.data does not separate intervention from merger in this field |

## Funds, ETFs and crypto (extension of 2026-10-16)

| Assumption | Value | Rationale |
|---|---|---|
| Critical metrics: funds | Sharpe and maximum drawdown (need 12 months of quotes and the CDI) | Without a year of quotes there is no risk-adjusted return or measured risk; fees, registry and inflows missing only renormalize |
| Critical metrics: ETFs | Average volume (BR stocks); volume and 12-month volatility (others) | Liquidity and price history are the minimum for an exchange-traded fund; fee, replication and composition have no free source and are never critical |
| Critical metrics: crypto | Market cap and 12-month volatility | Size and price history; the rest (CoinGecko) may be missing without removing the score |
| Critical metrics: stablecoins | Largest deviation from the peg in 12 months (daily prices in dollars) | Keeping the peg is the asset's function |
| Fund window | Up to 36 months, minimum 12; collection fetches 25 months of quotes at once for a new fund | The daily report comes in monthly files of ~12 MB: 36 months at once would weigh ~430 MB |
| Risk-free and benchmark | Cumulative CDI (SGS 12); Ibovespa (Yahoo `^BVSP`) for equity funds and those that declare Ibovespa/IBrX; IPCA, IMA-B and other indices compared to the CDI | No free IMA-B series; Yahoo's Ibovespa is an unofficial source |
| Sortino | Deviation of daily returns below the CDI of the same day | Same basis as Sharpe; with no day below the CDI, score at the cap |
| Consistency | 12-month windows with a monthly step, minimum 6 | With fewer windows the fraction says nothing |
| Management tenure | Time since inception (CVM registration) | A manager change has no structured source |
| Peers | None (absolute anchors only; confidence × 0.7) | The registered funds and ETFs are too few for a percentile |
| Weights of BR equity ETFs | Those of the specification (30/20/25/15/10); in Structure, domicile 3 × replication 0.8 × tenure 1 | Replication has no source: the small weight keeps the dimension above 60% coverage |
| Weights of ETFs without composition | Efficiency 30, Liquidity 25, Structure 20, Risk 25 (volatility, drawdown) | Components and concentration only exist for B3 indices; observable risk goes in instead of imputing 35% of the weight |
| ETF tracking | 12-month difference against the index series (Ibovespa, S&P 500; converted at the day's exchange rate when the ETF is in reais); tracking error with weekly returns | Closes at different times inflate the error with daily returns |
| Inheritance from components | Average of the scores of the stocks in the theoretical portfolio weighted by share, with at least 50% of the weight having a score | §7.2; below that the average does not represent the index |
| Domicile | BR 80, UCITS (European exchanges) 70, US 40, others 55 | 30% withholding on dividends and US estate tax for US ETFs; 15% in Irish UCITS |
| Crypto: weights | Those of the specification; in Security, network age 2 × hacks 0.5 × Nakamoto 0.5 | Hacks and Nakamoto have no source: they stay visible as unavailable without imputing the dimension |
| Stablecoin: weights | Reserves 35, Peg 30, Liquidity 25, Regulation 10 (specification: 50/25/15/10) | No dimension may exceed 35; reserves remain the largest |
| Stablecoin: reserves | Grade of the backing declared on DefiLlama (fiat 75, crypto 55, algorithmic 10); attestation unavailable | Attestations are PDFs from the issuers |
| Dollar values | Converted at the day's PTAX | The anchors are in reais |
| Crypto: volatility | The app's quotes; without 120 closes in 12 months, DefiLlama's daily dollar prices | Yahoo does not have all pairs in reais |
