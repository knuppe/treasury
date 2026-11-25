# Methodology changelog

## Inputs of 2026-11 (no change of methodology version)

Weights, thresholds and formulas of the score do not change; some inputs do ([indicators.md](../docs/indicators.md)):

- Net income attributable to owners: when a company leaves the owners' line at zero in the consolidated statements (about one filing in five), the parent-only net income of the same period is used (before, the zero was used, and metrics like ROE and the earnings yield came out at zero). Banks use the parent-only statements (Central Bank standard) for net income and equity.
- Share count checked against the B3 capital stock count (scale, treasury shares already excluded, stock events).
- Cross-check with Fundamentus in its own convention: the confidence drops only when the data disagrees, no longer because of a formula difference (EBIT margin, net margin, EV/EBITDA).
- FII largest tenant: a tenant row of the quarterly report can no longer weigh more than its own property in the fund's revenue (about one report in twenty put the property's share in the fund's column, which made the largest tenant exceed the largest property); without the fund's share, the row's share of the property times the property's weight. The WALT keeps the 5-year premise for leases above 36 months, and the metric now carries a note with the revenue share in that band and the WALT with 3 years, the least the report allows. The income coverage gets a note when sales of assets are more than 10% of the 4-quarter result.

## 0.1.0 (2026-10-16)

First version, from Knuppe's specification (weights and thresholds from the literature, no backtest yet).

- Classes: Brazilian stocks (general and eight segments), banks (CAMELS, with Bacen IF.data), insurers (no Susep solvency), brick-and-mortar REITs (FIIs), paper, hybrid and fund-of-funds FIIs.
- Differences from the specification, with the reason:
  - `fcf_to_net_income` and `dividend_fcf_coverage` use a range (anchor only): the percentile rewarded huge values that only indicate a small profit or dividend.
  - "No peers" effect: metrics without enough peers (≥ 8 at some level) use only the absolute anchor and lower the confidence.
  - Banks: peers by prudential size (S1–S2 together, S3, S4), because small banks carry capital far above the large ones and distorted the Basel percentile.
  - Bank efficiency: (personnel + administrative expenses) ÷ (intermediation result **before** expected loss + fees and services).
  - Paper FII: credit risk through proxies (concentration by securitizer, real change in net asset value per share, yield coverage) while LTV and rating have no source.
- Extension: fixed income (`classes/fixed_income.json`): Tesouro Direto, bank paper with FGC and private credit. Issuer health capped at 35 (cap per dimension): the 5 points went to the FGC for bank paper (total credit risk stays 65) and to the spread for private credit; structure and rating have no source (imputed).
- Extension: funds, ETFs and crypto (`classes/funds.json`, `classes/crypto.json`): funds by CVM class (fixed income, multimarket, equity, pension) with Sharpe, Sortino, return over the benchmark, consistency in 12-month windows, drawdown and recovery, costs, net assets, shareholders, inflows and redemption period; ETFs with tracking, liquidity, net assets, domicile and, for Brazilian equity ETFs, inheritance of the components' score and concentration (without composition, risk instead); crypto and stablecoins by robustness within the class. Differences from the specification's weights in ASSUMPTIONS.md.
- Evidence: none yet (IC/AUC pending, §12.1).
- Extension: US stocks, REITs and BDRs (SEC EDGAR; `classes/intl.json`).
- Qualitative events (§8.2): types, cap/penalty and keywords in `thresholds.json` (`events`); only those accepted by the user count, with a half-life of 4 quarters; CVM material facts become suggestions.
- Adjustment (2026-10-18): a bank-sector company with ordinary-company statements does not use the bank profile (the IF.data indicators are the bank's, not the holding's). Holdings in this situation (Itaúsa) go to the `holdings` segment (general profile); companies that are not holdings go to the general one. Rule in `segments.json` (`layout_required`, `holding_segment`).
- Adjustment (2026-11-13): the meaning of the Robust and Risk bands describes only what was measured ("on the measured criteria"), without "hold or accumulate" or "sell candidate". The score does not say what to do with the asset. Weights, thresholds and scores do not change.
