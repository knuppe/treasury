# Money invariants and validity of the embedded tables

Two safety nets against wrong numbers: tests that check, in demo vaults, that the same number
comes out equal through different paths of the core, and a registry of the annual tables embedded
in the app, which warns in "To solve" when one of them may be expired.

## Invariants in the demo vaults

`crates/treasury-core/src/demo/tests_invariants.rs` generates the five profiles of the demo
wizard in Brazil, plus one in the US (in English, dollars) and one in Portugal (in Spanish,
euros), and checks sums that must hold for any family. Each invariant compares **two core paths
for the same number**; when one breaks, the error names the profile, the country, the month and
the two values.

| # | Invariant | Paths compared |
|---|---|---|
| 1 | The month is the same number everywhere (last 3 complete months, income and expense) | `reports.month_flow` × `reports.dashboard` × the total and the sum of the categories of `reports.spending` |
| 1b | Month surplus = income − expense | `reports.month_flow` |
| 2 | Net worth = assets − liabilities; portfolio and physical assets fit within assets; contracts fit within liabilities; the dashboard shows the same net worth | `reports.net_worth` × `reports.dashboard` |
| 3 | The sum of the per-member columns is the family total | `reports.by_member` |
| 4 | The balance forecast of each account starts at today's balance | `forecast.balance` × `accounts.list` |
| 5 | "To solve" counts what it shows and does not repeat an item; the step is between 1 and 6 | `attention.list`, `attention.next_step` |
| 6 | Reading does not write: a read-only client calls 24 everyday queries and no vault table changes size | row count of all tables, before and after each call |

### What the invariants do not cover

- **Updates to existing rows.** Invariant 6 counts rows; an `UPDATE` done by a read gets through.
  It exists to catch the case that already happened (a read creating an alert, snapshot or
  feedback), not all cases.
- **Queries that use the network** (quotes, fundamentals, Banco Central rates) are left out of
  the list in invariant 6, because the test runs without internet.
- **The per-member view in shared accounts** (B10 in [assessment-2026-10.md](assessment-2026-10.md)):
  each member's share is rounded separately and may differ from the total by a few cents. There
  is no invariant for it.
- **Investments**: average price, TWR/XIRR and IR have their own tests, with no cross invariant
  here.

### How to add an invariant

Write the comparison inside `check` and run `cargo test -p treasury-core --lib
demo::tests_invariants`. If a new invariant fails, the first step is to decide which of the two
paths is right; the test only goes in green after the fix or with the difference explained in
this document.

## Validity of the embedded tables

Annual tables, such as the INSS floor and ceiling, ship inside the binary and only change in a
new Treasury version. An expired table does not break anything: the estimate is produced, only
with the previous year's values. The `crate::datasets` registry exists so the app can say so.

| Table | Country | Where the year and the source come from |
|---|---|---|
| INSS (floor and ceiling) | BR | `goals::inss::TABLE` |
| Segurança Social (IAS and minimum pensions) | PT | `goals::pension_pt::TABLE` |
| Social Security (cutoff points) | US | `goals::pension_us::TABLE` |

- Each table is valid until the end of its year. The warning starts on **February 1 of the
  following year**, because the ordinances come out in January and a new version takes a few
  days.
- Only the tables for the family's country are included.
- The warning appears in "To solve" as information ("1 of the app's tables may be out of date"),
  with a shortcut to Settings › About and support. Once dismissed, it stays away for 90 days.
- Updating the table updates the registry: the year and the source are read from the constant
  itself.

These already had their own control and were left out: the estimate of taxes paid (`tax_burden`,
field `stale` by the months since `updated`) and the IR rules, chosen by the assessment year
(`tax::rules`).

### To register another table

Add an entry to `datasets::all()` pointing to the constant's year and source, and the name key in
`locales/*/atencao.ftl` (`tabelas-*`). Candidates: the annual limits of the retirement plans by
country (`retirement`), the payroll rules in `tax_burden/data/payroll.json` and the consumption
percentages from IBGE's National Accounts.
