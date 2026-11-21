# Floating-rate and index-linked loans

How the schedule of a debt contract calculates interest when the rate depends on an index: CDI and
Selic (daily rate), IPCA, IGP-M, INPC and INCC (monthly adjustment), TR and savings (anniversary
date). The setup is in Debts; the defaults reproduce a monthly fixed-rate contract, so a contract
without these fields is calculated as it always was.

## Indexers

| Indexer | Interest factor for the period | Where it appears |
|---|---|---|
| **CDI or Selic + spread** (exponential 252) | `FatorDI × (1 + s)^(du/252)`, with `FatorDI = Π (1 + TDIₖ × p)` over business days | Working capital, CCB (bank credit note), business credit, home equity, some CDC (consumer credit) |
| **% of CDI** (with or without spread) | `Π (1 + TDIₖ × p)`, times the spread if any | CCB, credit secured by investments |
| **CDI + linear spread** | `FatorDI × (1 + s × dc/360)` | Old or business contracts |
| **IPCA, IGP-M, INPC, INCC + interest** | change of the index for month M − n (default M − 1), with optional pro rata of the first period, × the interest | IPCA-linked mortgage, off-plan property (INCC), IGP-M contracts |
| **TR (SFH/SFI)** | balance × (1 + TR of the period starting on the **anniversary date**), then the interest | Mortgage financing |
| **Savings + x%** | adjustment by the savings yield on the anniversary, then the interest | "Savings" mortgage lines |

Terms: `TDIₖ` is the daily rate (SGS 12 for the CDI, SGS 11 for the Selic, already in % per day),
`du` the business days of the period, `dc` the calendar days, `s` the annual spread and `p` the
percentage of the index.

## Conventions

- **Period**: from the base date (signing or previous due date), inclusive, to the due date,
  exclusive, counting business days of the ANBIMA national calendar. A due date on a non-business
  day moves to the next business day, and interest runs until then.
- **Rounding** (B3 methodology for the DI): daily factor and product with 16 digits truncated; index
  factor in the period with 8 digits; spread and interest factor with 9 digits; values in cents.
- **Lag**: in business days for the CDI and the Selic (the DI from `n` days before each day); in
  months for the monthly indexes.
- **Adjustment reference**: by month (`Month`, default for monthly indexes) or by anniversary date
  (`Anniversary`, default for the TR and savings).
- **Deflation**: optional floor of zero (some contracts do not apply a negative index).
- **Pro rata of the first period**, when the first due date does not close a month: none, by
  calendar days or by business days.
- **Fixed or real interest**: equivalent monthly rate (default), `(1 + r)^(du/252)` or
  `(1 + r)^(dc/365)`.

## How the installment is formed

- **SAC** (constant amortization): amortization = balance ÷ remaining installments; interest =
  balance × (factor − 1). The installment changes every month.
- **Price**: at each due date the installment is recalculated on the adjusted balance and the
  remaining term.
- **Bullet**: only the interest at each due date and the principal at the last one (CCB, working
  capital).
- **Grace period**: pays the interest during the grace period.

## Future installments

Days without a published rate use the last known rate and mark the period as **estimated**; with no
rate at all in the vault, the projected CDI is the target Selic − 0.10 p.p. Estimated installments
are recalculated at each index collection. When an installment is paid, the reconciled value applies.

## In the code

- `crates/treasury-core/src/debt/postfixed.rs` (CDI and Selic), `index_factors` and
  `interest_periods` in `crates/treasury-core/src/debt/mod.rs` (monthly indexes, TR, savings, pro
  rata, interest basis).
- Migrations V90 (daily-rate floating loans) and V91 (lag, anniversary, pro rata, interest basis).

## Limitations

- No rates from other countries (SOFR, €STR) and no 30/360 convention.
- A grace period that adds the interest to the balance (capitalization) is not modeled.
- The contract's own amortization table (percentage per installment) is not imported.
- Late fees and default interest do not enter the schedule.
