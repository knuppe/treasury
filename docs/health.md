# Financial health

A score from 0 to 100 that says, using the vault's own data, whether the family is in good financial health. The model is chosen in **Settings › Financial health** (setting `health.settings` in the vault, no migration): **I-SFB** (default), **Classic indicators** or **Off** (the dashboard card disappears and the reports section shows only the notice).

Where it appears: the `Financial health` card on the dashboard (score, band and dimensions or indicators) and the **Financial health** tab in `Reports` (`?v=saude&mes=AAAA-MM`), with each indicator: value × target, state, points and weight, how it was calculated, the values used (with privacy mode) and 1 to 3 suggestions when it is not good.

Commands: `health.score` (optional `as_of`, `AAAA-MM`; read only), `health.get_settings` and `health.set_settings` (interface only; MCP clients and routines get `forbidden`). MCP tool: `get_financial_health`. Code in `crates/treasury-core/src/health/` (`facts.rs` reads the data, `models.rs` scores); the interface mock (`apps/desktop/src/api/mock/health.ts`) repeats the same calculations.

## Data and window

- **Window**: up to 12 complete months. In the current month, it ends in the previous month; in a past month, it ends there. Months before the first transaction are left out.
- **Flow**: the reports rules (only `income`/`expense`; transfers, card statement payments and contributions do not count; basis from the `budget_basis` setting; base currency, currencies without an exchange rate are left out and become a warning).
- **No data, no penalty**: with fewer than 3 months, or without the information (no budget, no contracts or paid card statements recorded, no essential category), the indicator is `unknown`, has no points, and leaves the average. No score when months are missing or when less than half the weight remains (classic) or fewer than 2 dimensions (I-SFB).
- **Points**: piecewise linear curves between the points of the tables below (outside them, the extreme).

| Indicator | Calculation |
|---|---|
| Savings rate | (income − expenses) ÷ income of the window |
| Regular savings | months with rate ≥ 10% ÷ months with income |
| Emergency fund | fund ÷ monthly average of expenses in essential categories (`is_essential`) over the last 6 months. Fund: in the current month, the one from `goals::emergency_reserve` (emergency goal or balance of checking + savings + cash); in past months, that balance at the end of the month |
| Income committed to installments | installments of loans/financings (not archived) that fell due in the window ÷ income. Card installment plans are expenses and do not count |
| Housing cost | expenses in the root category "Moradia" + mortgage installments recorded outside it ÷ income |
| Revolving credit and overdraft | months with any day of negative balance in a checking account (opening balance + settled transactions, day by day) or with a card statement not paid in full within 5 days after the due date. Only cards with some card statement payment recorded |
| Net worth growth | (net worth in the evaluated month − 12 months earlier) ÷ max(\|initial net worth\|, expenses of the window). Nominal |
| Bills paid on time | installments paid within 5 days after the due date (an unreconciled overdue installment counts as late) + card statements paid in full by then ÷ total due in the window |
| Spending within budget | months with expenses ≤ planned ÷ months of the window with an expense budget |
| Months in the black | months with income ≥ expenses ÷ months with activity |

## I-SFB model

Inspired by the [Brazilian Financial Health Index](https://meubolsoemdia.com.br/Materias/indice-saude-financeira-2024) (Febraban and the Central Bank of Brazil, 2024: scale 0–100, national average **56.7**). The official index comes from a questionnaire; here it is an **estimate** from the data, and the interface says so.

Four dimensions of equal weight (25%), each the weighted average of its indicators; the score is the average of the dimensions with data.

| Dimension | Indicator (weight) | Points | Good / attention |
|---|---|---|---|
| Behavior | Bills on time (40%) | 70% → 0, 90% → 50, 100% → 100 | 100% / ≥ 90% |
| | Within budget (30%) | 0 → 10, 50% → 55, 83% → 85, 100% → 100 | ≥ 83% / ≥ 50% |
| | Months in the black (30%) | 25% → 0, 50% → 40, 75% → 75, 100% → 100 | ≥ 83% / ≥ 60% |
| Skill | Savings rate (60%) | −10% → 0, 0 → 30, 10% → 60, 20% → 85, 30% → 100 | ≥ 20% / ≥ 10% |
| | Regular savings (40%) | 0 → 0, 50% → 50, 83% → 85, 100% → 100 | ≥ 75% / ≥ 50% |
| Freedom | Committed income (60%) | 0 → 100, 15% → 85, 30% → 50, 40% → 25, 60% → 0 | ≤ 15% / ≤ 30% |
| | Revolving credit and overdraft (40%), by fraction of months | 0 → 100, 1/12 → 75, 25% → 45, 50% → 15, 100% → 0 | none / ≤ 1/6 |
| Security | Emergency fund (60%), in months | 0 → 0, 1 → 30, 3 → 60, 6 → 90, 12 → 100 | ≥ 6 / ≥ 3 |
| | Net worth growth (40%) | −10% → 0, 0 → 40, 5% → 70, 10% → 100 | ≥ 5% / ≥ 0 |

Bands (rounded score): Poor 0–36, Very low 37–49, Low 50–56, Ok 57–60, Good 61–68, Very good 69–82, Excellent 83–100. A dimension is good from 69, attention from 57 to 68.

## Classic indicators

The usual ratios from financial education; the score is the weighted average of the indicators with data.

| Indicator (weight) | Points | Good / attention |
|---|---|---|
| Savings rate (25%) | 0 → 0, 10% → 60, 20% → 100 | ≥ 20% / ≥ 10% |
| Emergency fund (25%) | 0 → 0, 3 → 60, 6 → 100 months | ≥ 6 / ≥ 3 |
| Committed income (20%) | ≤ 15% → 100, 30% → 60, 40% → 20, 50% → 0 | ≤ 15% / ≤ 40% (above 40%, bad) |
| Housing cost (10%) | ≤ 30% → 100, 40% → 50, 50% → 0 | ≤ 30% / ≤ 40% |
| Revolving credit and overdraft (10%) | none → 100, 1/12 → 50, 25% → 20, 50% → 0 | only "none" is good |
| Net worth growth (10%) | −10% → 0, 0 → 50, 5% → 80, 10% → 100 | ≥ 5% / ≥ 0 |

Bands: Critical 0–39, Fragile 40–59, Stable 60–79, Healthy 80–100.
