# Asset health score

A grade from 0 to 100 for each asset in the portfolio or in the downloaded universe, through a buy-and-hold lens: it measures the health of the business (and, separately, the attractiveness of the price), not timing. It is different from the [family's financial health](health.md).

The methodology is at version `0.1.0` (draft). Weights, thresholds and segments live in [`methodology/v0.1.0/`](../methodology/v0.1.0), never in the code; assumptions are in [`methodology/ASSUMPTIONS.md`](../methodology/ASSUMPTIONS.md), literature in [`LITERATURE.md`](../methodology/LITERATURE.md) and changes in [`CHANGELOG.md`](../methodology/CHANGELOG.md).

## Where it appears

- **Health** tab on the asset page (`?secao=saude`): grade, band, confidence, Health and Attractiveness sub-scores, segment (with manual override), why this grade (factors for and against, red flags), dimensions with each metric against peers, frameworks (Piotroski, Altman Z'', Beneish, Magic Formula), cross-check with Fundamentus and of dividends with B3, history and the **Disagree** button.
- **Asset health** tab in Portfolio (`?aba=saude`): all assets with a position or on the watchlist.
- MCP: `get_asset_score` (`score.get`) and `get_portfolio_scores` (`score.portfolio`).

Commands: `score.get`, `score.portfolio`, `score.history`, `score.methodology`, `score.feedback.list` (read-only); `score.set_segment` and `score.feedback.add` only through the interface. Code in `crates/treasury-core/src/score/`.

## Covered classes

| Class | Status |
|---|---|
| Brazilian stocks (non-financial), by segment | ✅ general, cyclicals, utilities, retail, technology, health, construction, industry, telecom, agribusiness |
| Banks (CAMELS) | ✅ with Bacen IF.data |
| Insurers | ✅ partial: without Susep solvency (no source) |
| Brick-and-mortar, paper (mortgage-backed), hybrid and fund-of-funds FIIs | ✅ with the CVM monthly and quarterly reports |
| US stocks, REITs and BDRs | ✅ SEC EDGAR (segment by SIC; a BDR inherits the underlying) |
| Stocks on other exchanges | No score: there is no free source of financial statements |
| Fixed income: Tesouro Direto, bank (CDB, LC, LCI, LCA, savings) and private credit (debentures, CRI, CRA) | ✅ see [Fixed income](#fixed-income); structure and rating of private credit have no source |
| Funds (fixed income, multimarket, equity, pension) | ✅ with the share values, net assets, shareholders and fundraising from the CVM daily report, the fund and class registry and the statement (fees and terms); CDI benchmark (Ibovespa for equity funds); no peers |
| ETFs | ✅ Brazilian equity ETFs that track B3 indices (inheriting the score of the theoretical portfolio components, concentration); others (foreign, fixed income) with risk in place of composition; fee (TER) and replication have no source |
| Crypto and stablecoins | ✅ robustness within the class (not comparable with stocks), CoinGecko and DefiLlama; active addresses from Coin Metrics (open API, only the covered networks), Bitcoin's Nakamoto coefficient from mempool.space, network attacks from the DefiLlama list, zero unlocks on proof-of-work coins; concentration in the largest wallets, token unlocks, Nakamoto of the other networks and stablecoin reserves/regulation have no source |
| Commodities | ⏸ no class of their own in the registry (gold ETFs count as ETFs) |

## Pipeline

1. **Classify**: class (`equity_br`, `reit`…) → segment → weight profile. Stocks by the sector in the CVM FCA (holdings inherit the sector), with the statements layout (bank, insurer) taking precedence over the sector. A bank sector without bank statements (a holding such as Itaúsa) goes to "Holding companies", with the general profile: the Bacen IF.data indicators belong to the invested bank, not to the holding. FIIs by the composition of assets in the monthly report (CRI ≥ 50% = paper; FII shares ≥ 50% = FoF; real estate ≥ 50% = brick-and-mortar; otherwise hybrid). The user can change the segment (setting `score.segment_overrides`).
2. **Raw metrics** with formula, period, source and, for the key metrics (ROIC, ROE, net margin, net debt/EBITDA, free cash margin), the average of the last 5 fiscal years and the annual slope.
3. **Peers**: the day's universe (`score_universe`), recomputed after each collection. The finest level with at least 8 peers: sector (FCA; for banks, the prudential size S1–S2, S3, S4; for FIIs, the segment of activity) within the same segment → segment → class.
4. **Metric grade**: `0,7 × winsorized percentile (2,5%–97,5%) + 0,3 × absolute anchor`. Optimal-range metrics (payout, current ratio, P/B of a FII…) and those without peers (momentum, volatility, listing segment) use only the anchor. Key metrics: `0,5 × level + 0,25 × average + 0,25 × trend`. A negative denominator (EBITDA ≤ 0, equity ≤ 0) explicitly gets the worst grade.
5. **Dimensions**: weighted average of the metrics that have data. Coverage < 60% → imputed grade (peer median × 0.9).
6. **Score** = weighted sum of the dimensions; **red flags** apply a cap and penalties; bands: Robust ≥ 75, Healthy 60–74, Caution 40–59, Risk < 40.
7. **Confidence** = weighted coverage × freshness (0.8 if the data is more than 200 days old) × peer level (sector 1; segment 0.95; class 0.85; no peers 0.7) × low liquidity (0.9) × divergence from Fundamentus above 25% (0.9).
8. **Snapshot** in `score_snapshot` (the day's is redone until the day ends; earlier ones never change) and an alert (`alert`, type `asset_score`) only when health really worsens, between two final scores: a drop of 10 points or more, a band that falls to Caution or Risk, or a new serious red flag. Leaving "Insufficient data", improving, becoming partial or a matured security do not alert (`thresholds.json` › `alerts`).

## Qualitative events

Judicial recovery (bankruptcy protection), fraud or restatement, relevant litigation, environmental disaster, governance and regulation (types, caps, penalties and keywords in `thresholds.json` › `events`). Each event has a date, severity and evidence (link). Only the ones **accepted** by the user affect the grade: the cap relaxes and the penalty decreases with a half-life of 4 quarters, until the event is marked as resolved. The CVM material facts and notices (IPE) of the last 24 months whose subject matches the keywords appear as **suggestions** in the Health tab ("Apply to the score"). Commands `score.events.list` (read), `score.events.add`, `score.events.set_status` and `score.events.delete` (interface only).

## Missing data

- **Critical** (the `critical` list of each profile in `weights.json`: e.g. ROE, net margin, net debt/EBITDA and earnings history for stocks; ROE, Basel ratio and delinquency for banks; P/B, DY and vacancy for brick-and-mortar FIIs): **no score**, band "Insufficient data" with what is missing, and a feedback item on the **Developer** screen (area data; one per asset while it is open).
- **Non-critical**: renormalizes within the dimension; with less than 60% coverage, it imputes. More than 30% of the weight imputed → partial score without a final band.
- **No source** (banks' LCR, Susep solvency, LTV and rating of CRIs, ETF TER and replication, paid on-chain metrics): the metric appears as unavailable, with the reason (`unavailable` in the methodology), and the dimension renormalizes. It is a known gap, with no developer feedback (since 2026-10-20; the old "no source for …" notices close on their own at the next evaluation).
- **Newly listed** (a US stock with no annual report yet at the SEC): "Insufficient data" with the key `short_history`, no feedback: it is a lack of history of the asset, not of the collection.

## Sources

| Data | Source |
|---|---|
| Statements (income, balance sheet, cash flow, value added), filing dates, auditor | CVM Open Data: DFP, ITR, FCA |
| Basel ratio, core capital, leverage, 90-day delinquency, problem assets, expected loss, expenses, intermediation result | Bacen IF.data (reports 1, 2, 4, 5 and 16; prudential conglomerate by the leading CNPJ) |
| Vacancy, delinquency, concentration, contract terms, result × yield, CRI by issuer | CVM: FII quarterly report |
| Equity, liabilities, obligations, composition, management fee, shareholders | CVM: FII monthly report |
| Quote and volume | B3 COTAHIST; the app's quotes (Yahoo) for volatility and momentum of registered assets |
| IPCA, CDI | Central Bank of Brazil (SGS) |
| Fund shares, net assets, shareholders and fundraising | CVM: daily report (`inf_diario_fi`), monthly summary in `fundamental` with the fund's CNPJ |
| Class, benchmark, inception, manager and net assets of B3 funds and ETFs; fees and redemption terms | CVM: fund and class registry (`registro_fundo_classe.zip`) and information statement (`extrato_fi.csv`), technical setting `score.fund_registry`; ETF CNPJ by name (Yahoo) |
| Composition of Brazilian equity ETFs | B3: the day's theoretical portfolio of the indices (IBOV, IBXX, IBXL, SMLL, IDIV), setting `score.index_portfolios` |
| Ibovespa and S&P 500 (benchmark of equity funds and ETF tracking) | Yahoo (`^BVSP`, `^GSPC`), series `ibov` and `sp500` (unofficial source) |
| Market cap, ranking, FDV, supply, volume, all-time high, development and categories of crypto assets | CoinGecko (public API, ~10 requests/min), issuer `coingecko:<id>` in `fundamental` |
| Network TVL, stablecoins (backing, peg), market cap and daily prices in dollars | DefiLlama (public API) |
| Dividends (DY, Bazin) | The asset's calendar (Yahoo, Fundamentus) checked against B3 (API of listed companies and funds, `research_dividend`); with a material divergence, or without a calendar, B3 (official) |
| Cross-check | Fundamentus (local cache; never a primary source); dividends with B3, no effect on confidence |

## Fixed income

Class by the type of security: Tesouro (`treasury_br` or `tesouro_*`) → `fi_sovereign`; debentures, CRI and CRA → `fi_corporate`; CDB, LC, LCI, LCA, savings and "other" → `fi_bank`. Weights in [`classes/fixed_income.json`](../methodology/v0.1.0/classes/fixed_income.json); assumptions in [ASSUMPTIONS.md › Renda fixa](../methodology/ASSUMPTIONS.md).

| Profile | Dimensions (weight) | Critical |
|---|---|---|
| Tesouro Direto | Sovereign risk 20 (fixed assumption); Rate attractiveness 30 (percentile of today's rate over 10 years, real rate); Horizon and duration 25 (duration, price volatility); Liquidity 15; Costs 10 (B3 custody, cost of exiting today) | rate percentile, duration |
| Bank | Issuer health 35 (Basel ratio, core capital, leverage, delinquency, coverage, ROE and efficiency of the bank in IF.data against banks of the same size: inherited from §7.2); FGC coverage 30; Rate vs CDI 20; Liquidity and term 15 | Basel ratio, FGC coverage |
| Private credit | Issuer health 35 (Health sub-score of the company's stock: inherited); Structure 20 (collateral, covenants, subordination: no source, imputed); Spread vs risk 25 (spread over the Tesouro; rating has no source); Duration and liquidity 10; Tax exemption 10 | issuer health |

- **Issuer**: by the CNPJ in the security's registration (bank: the prudential conglomerate in IF.data; company: the CVM issuer). Without a CNPJ, by the issuer name equal, dropping generic words ("banco", "S.A.", "prudencial"…), to a single conglomerate or company. For banks, also by brand: a list of brands with the issuer's CNPJ (Nubank → Nu Financeira, Itaú, Inter, BTG, C6, XP…), checked against the name IF.data gives that CNPJ, or by a single distinctive word ("Credivale") in the name of only one conglomerate. The issuing bank's CNPJ can be entered when registering or editing the security. The IF.data collection starts requesting reports 2 and 4 for those banks. Without identification: "Insufficient data" and feedback for the developer.
- **FGC** (the Brazilian deposit guarantee fund): R$ 250 thousand per CPF per conglomerate, adding up all the family's covered securities in it, with a ceiling of R$ 1 million per CPF; the position of each account is split among the registered holders (accounts without a holder count as a single CPF). LF and debentures have no FGC; "other" is left without the metric (critical).
- **Red flags**: exposure above the FGC limit (−5), bank with a Basel ratio below the minimum + 1 p.p. (cap 40), institution not operating in the IF.data registry (liquidation, intervention; cap 20) and a maturity already past (cap 50).

| Data | Source |
|---|---|
| Buy and sell rates, buy and sell unit prices, the day's curve | Tesouro Transparente (Tesouro Direto CSV), downloaded by the quote collection; 10-year daily medians (IPCA+ and fixed-rate families) and the latest curve saved in `fundamental` (`tesouro:real`, `tesouro:nominal`, `tesouro:titulo:…`) |
| Selic target, CDI, IPCA, TR | Central Bank of Brazil (SGS 432, 12, 433 and 226) |
| Issuing bank's health | Bacen IF.data (the same indicators as for banks) |
| Issuing company's health | Stock score (CVM) |
| Position and holders | Portfolio trades and account holders |
| Indicative debenture rates (ANBIMA), collateral, covenants, rating | No open source: the spread uses the contracted rate against the Tesouro curve |

## Limitations of 0.1.0

- Non-standardized sector metrics (AISC, lifting cost, SSS, NRR, VSO…) have no free structured source; the segments differ by weights and thresholds.
- The backtest (§12) is not implemented yet.
- In English and Spanish, the methodology texts (metrics, formulas, profiles, segments, dimensions, red flags) are translated only for what applies outside Brazil: foreign stocks, ETFs listed abroad and crypto assets (`locales/*/nucleo-metodologia.ftl`). B3 stocks, FIIs, CVM funds and local fixed income stay in Portuguese ([i18n-design.md](i18n-design.md)).
- The auditor's qualification and covenants are not read.
