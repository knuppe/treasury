# FGTS

FGTS linked accounts (the Brazilian severance fund; one account per employment relationship),
with import of the Caixa statement, calculation and checking of the interest and adjustment
credits (JAM), withdrawals (including use for housing), the birthday withdrawal, comparison with
IPCA, CDI, savings and M2, the link to retirement and the Life timeline, and MCP tools for the
agent. Brazilian vaults only. No worker CPF, PIS, work card or name is stored.

## Official rules (with source and date)

| Rule | Value | Source |
|---|---|---|
| Monthly deposit | 8% of pay (2% for apprentices; domestic workers 8% + 3.2% advance for the penalty) | Lei 8.036/1990, art. 15; LC 150/2015 |
| Deposit deadline | by the 20th of the following month (FGTS Digital, since 03/01/2024); before that, the 7th | Lei 8.036, art. 15, as amended by Lei 14.438/2022 |
| Adjustment | savings parameters (TR) + 3% a.a. interest, monthly compounding | Lei 8.036, art. 13 caput |
| Credit day (JAM) | the 10th until March/2024; **the 21st** since April/2024, on the balance of the day of the previous credit, less debits; in the month of the change, two credits (10th and 21st) with pro rata JAM, which may come with a regularization line | Lei 8.036, art. 13 §§ 1, 1-A, 2 (as amended by Lei 14.438/2022) |
| Progressive interest | 3% to 6% a.a. only for those who opted before 09/22/1971 | Lei 8.036, art. 13 § 3 |
| Profit distribution | part of the base year's profit, proportional to the balance on 12/31, credited by 08/31 of the following year; not part of the base of the termination penalty | Lei 8.036, art. 13 §§ 5–7 (Lei 13.446/2017 and 13.932/2019) |
| 2026 distribution (base year 2025) | R$ 13.04 bn = 89% of the R$ 14.655 bn profit; **index 0.01868341** per real of balance on 12/31/2025; approved on 07/28/2026 | FGTS Board of Trustees (Conselho Curador), reported by DGABC/Diário do Comércio (Jul/2026) |
| Inflation floor | total remuneration (JAM + distribution) must be at least the IPCA; in years when it does not reach it, the Board of Trustees defines the compensation; applies to balances from the publication of the minutes (judgment on 06/12/2024), with no retroactive effect | STF, ADI 5090, STF news item of 06/12/2024 |
| Birthday withdrawal | rate by band of the total balance + additional amount: up to 500: 50%; 500.01–1,000: 40% + 50; 1,000.01–5,000: 30% + 150; 5,000.01–10,000: 20% + 650; 10,000.01–15,000: 15% + 1,150; 15,000.01–20,000: 10% + 1,900; above 20,000: 5% + 2,900. Window: from the birthday month to the last business day of the 2nd following month. On dismissal without cause, only the 40% penalty is released. Going back to termination withdrawal only on the 1st day of the 25th month after the request; anticipation (assignment) prevents going back | Lei 8.036, arts. 20-A to 20-D and Annex (Lei 13.932/2019). **Check the numbers against the Annex on Planalto before implementing** (two secondary sources agree; Planalto did not open from here) |
| Termination penalty | 40% on all deposits of the contract, adjusted and with interest, including those already withdrawn ("value for termination purposes" on the statement) | Lei 8.036, art. 18 § 1 |

Project policy: tax and legal rules are an **estimate**, with source and date on screen, and
only what two independent sources confirm gets corrected. The birthday withdrawal table and each
year's distribution index live in a built-in data file (`fgts/data.rs`), reviewed every year in
August (the same routine as INSS in January).

## The Caixa statement (format)

PDF "Histórico" issued by the FGTS app/Caixa website, **one per linked account** (one per
employer). Read with `crate::pdf::pages` (the same one as `pension/statement.rs`).

Header (label on top, value below, in three columns):

- worker name, PIS/PASEP, work card → **not read**;
- employer (name, may wrap onto two lines), employer registration (CNPJ), admission date, option
  date, separation date and code, category;
- account no. (establishment/account + unit): stored only as a hash, to recognize the same
  account on a new import;
- account type (OPTANTE), interest rate (3.0% a.a.), **value for termination purposes**.

Variation of the same "Histórico" in pages: each page repeats the "Histórico emitido em" line
and the table labels; the employer name wraps before the admission date (the second part of the
name shares the line with the date and the PIS); the employer registration comes without
punctuation and without the leading zero (13 digits, completed and checked by the check digits)
on the same line as the work card and the account no.; the table starts with `SALDO ANTERIOR`;
and the distribution carries `12/AAAA` on the values line. Invented fixture:
`fgts/fixtures/extrato_historico_paginas.txt`.

Lines: `DD/MM/AAAA  HISTÓRICO  R$ VALOR  R$ TOTAL` (negative value as `-R$`). The "TOTAL" is the
running balance **in printed order**, which is not date order (the JAM credit of the 10th comes
before the deposit of the 6th of the same month). Histories in the layout:

| History | Type | Note |
|---|---|---|
| `115-DEPOSITO <MÊS> <ANO>` | monthly deposit | reference month = month/year in the text |
| `650-DEPOSITO <MÊS> <ANO>` | complementary deposit/difference | same reference month; add it to the 115 when checking the payslip |
| `CREDITO DE JAM 0,002466` | JAM | the coefficient is on the line (6 decimal places) |
| `REGULARIZACAO CREDITO DE JAM` | JAM adjustment | enters the base of the next credit |
| `AC CRED DIST RESULTADO ANO BASE` + `12/AAAA` on the next line | distribution | **history on two lines, with the date and values on the middle line** |
| `SAQUE DEP COD nn [ref]` / `SAQUE JAM COD nn [ref]` | withdrawal (deposits part / JAM part) | two entries of the same withdrawal; group by code + reference + date |
| `AC REPOSICAO JAM EM DD/MM/AAAA` | withdrawal reversal | counts as an inflow on the day of the original withdrawal for the JAM base |
| `AC AUT JAM CANCELAMENTO SAQUE` | JAM of the period of the reversed withdrawal | earnings |

Some known withdrawal codes: 01 (termination without cause), 50 (the 2019 immediate withdrawal,
up to R$ 500, which could be undone until April/2020), 19E (the 2020 emergency withdrawal), 60
(birthday withdrawal) and 91–96 (housing). Code 99, with a reference number, is generic. The
importer shows the code and, when it does not recognize it, asks for the reason (list:
termination, retirement, housing, birthday withdrawal, illness, emergency, other).

## JAM algorithm (validated to the cent)

```
juros_mes   = 1,03^(1/12) − 1 = 0,0024663
coeficiente = (1 + TR do período) × (1 + juros_mes) − 1      (6 casas, como no extrato)
base        = saldo logo após o crédito anterior (inclui o próprio crédito, regularizações e
              reposições de saque datadas no período) − saques do período
JAM         = truncar(base × coeficiente, 2 casas)            (trunca, não arredonda)
```

Deposits of the month only enter the base of the following month's credit. The cases the
implementation must cover are the withdrawal reversal and the JAM regularization: both enter the
base of the next credit. With a TR of zero (2018 to 2021), the coefficient is 0.002466, which
confirms the monthly interest.

Invented examples: a balance of R$ 10,000.00 after the previous credit, no withdrawal in the
period and coefficient 0.004150 → JAM = R$ 41.50. A balance of R$ 300.00 and coefficient
0.002634 → 0.7902 → R$ 0.79 (truncates, does not round).

The distribution is `index × balance on 12/31 of the base year`. Example: with a balance of
R$ 50,000.00 on 12/31/2025 and the index 0.01868341, the credit is R$ 934.17. The "value for
termination purposes" comes from a shadow ledger, with all deposits and the JAM they would have
earned, with no withdrawals and no distribution. The implementation must close this value to the
cent with a synthetic fixture, including the 2024 transition month.

## Data model (V134)

1. **New account type `fgts`** in `account.kind` (rebuilds `account` like V112, enters
   `storage::FK_OFF_MIGRATIONS`). One account per relationship, holder = the member. Currency
   BRL. It counts toward net worth, but is outside `forecast::DAILY`, the budget and the
   "uncategorized" list (it is non-liquid wealth, like the broker).
2. `fgts_account` (1:1 extension of the account): `account_id`, `employer_name`,
   `employer_cnpj`, `admission_date`, `option_date`, `leave_date`, `leave_code`, `category`,
   `interest_rate` ('3' or progressive), `account_hash` (hash of the account no., for
   re-importing), `rescission_base` (cents, from the last statement), `rescission_base_date`,
   `income_source_id` (the linked CLT income, to check against the payslip).
3. `fgts_entry` (1:1 extension of the transaction): `txn_id`, `kind` ∈ {`deposit`,
   `deposit_extra`, `jam`, `jam_adjustment`, `distribution`, `withdrawal`, `reversal`},
   `competence` (YYYY-MM), `coefficient` (decimal text), `base_year`, `withdrawal_code`,
   `withdrawal_reason`, `withdrawal_ref`, `statement_total` (printed balance, to reconcile),
   `computed` (0/1: came from the statement or was projected by the app).
4. `fgts_modality` per member: `member_id`, `modality` ∈ {`rescisao`, `aniversario`}, `since`,
   `return_requested_at` (effective return = 1st day of the 25th month),
   `anticipation_debt_id` (anticipation loan, when there is one).

The entries are ordinary `transaction`s in the FGTS account (balance, history, undo and audit
already work): deposit and JAM as an inflow with no budget income category; withdrawal as a
**transfer** to the account that received it; use for housing as an extra payment on the debt
with `from_account_id` = FGTS account (replaces the current `without_txn`, which stays for those
who do not register the FGTS account).

## Features

1. **Import statement** (PDF or text): a preview with a recognized or new account, employer, new
   × existing entries (dedupe by date + type + amount + printed balance), reconciliation of the
   final balance with the "TOTAL", JAM checked line by line (a warning when Caixa credited a
   different amount than calculated), withdrawals grouped with a request for the reason. Saving
   = one undoable batch.
2. **Check against the payslip**: for each reference month, `payslip.fgts` × deposits 115 + 650.
   Alert "the employer did not deposit the FGTS for <month>" when the 20th of the following month
   passes with no deposit (one alert per episode, link to the account).
3. **Projection**: future JAM credits with the last TR held constant (as in the debts), 8%
   deposits of the linked income's gross pay, distribution estimated by the average of the last
   3 indices (marked as an estimate until the official one is out).
4. **Real yield and comparison**: the FGTS IRR (flows = deposits and withdrawals, final value =
   balance) and, for the same flows on the same dates, what it would have become in IPCA,
   savings, 100% of the CDI, 85% of the CDI (approximation of an exempt investment, since FGTS
   pays no IR) and **M2 expansion** (`m2_br`, already synced from the BCB; the same deflator the
   app uses in "monetary expansion"). A table by year: JAM, JAM + distribution, IPCA, CDI, M2,
   and the "compensation owed" under ADI 5090 in the years (since 2024) when JAM + distribution <
   IPCA. Neutral language: it shows numbers with source and date, without recommending whether to
   withdraw.
5. **Birthday withdrawal × termination**: this year's amount by the table (sum of all the
   member's accounts), month and window, what stays blocked on a dismissal, the earliest date to
   go back, and the cost of anticipation (if there is a linked debt, it uses its CET).
6. **Housing**: when paying off/amortizing a real estate contract, "use FGTS" lists the holder's
   FGTS accounts with a balance and posts the outflow on them; a warning about the minimum
   interval of 2 years between uses for amortization (Lei 8.036, art. 20, V–VII; to be checked).
7. **Retirement and Life timeline**: the FGTS balance (projected to retirement age) enters as a
   life event "FGTS withdrawal at retirement" in the plan (`goals.retirement`), with the value in
   today's reais (deflated by the IPCA); a milestone on the Life timeline when it is on.
8. **Country**: account types exclusive to Brazil, starting with `fgts`, are only listed when
   the vault is set up for Brazil (`country.fgts` true). This holds for the type selector in
   Accounts, the new vault wizard, ⌘K, the texts and the description of `accounts.create` in
   MCP, and for the `fgts.*` tools, which refuse with a clear message outside Brazil. Accounts
   that already exist remain visible if the country changes later. The rule stays generic (one
   country per account type), to serve future local types.

## MCP tools (descriptions in English)

| Tool | Level | What it does |
|---|---|---|
| `fgts.list` | read | FGTS accounts, balance, employer, status (active/inactive), last import, modality |
| `fgts.entries` | read | entries of an account with type, reference month, coefficient and balance |
| `fgts.statement_preview` | read | reads a statement (attachment or text) and returns the preview with checks |
| `fgts.statement_import` | sensitive | saves the approved preview (undoable batch) |
| `fgts.check` | read | check: JAM calculated × credited, deposits × payslip, gaps in reference months |
| `fgts.yield_compare` | read | IRR and comparison with IPCA, savings, CDI, 85% CDI and M2 over the requested period |
| `fgts.project` | read | projected balance at a date/age (JAM, deposits, estimated distribution) |
| `fgts.record_withdrawal` | sensitive | records a withdrawal (code, reason, destination account as a transfer) |
| `fgts.use_for_housing` | sensitive | amortizes/pays off a real estate contract with FGTS balance (calls the debt payoff with `from_account_id` = FGTS account) |
| `fgts.anniversary_simulate` | read | the year's birthday withdrawal, window, block on termination, return date |
| `fgts.set_modality` | sensitive | changes the recorded modality (record only; the real request is made in the Caixa app) |
| `fgts.update_account` | reversible | employer, dates, separation, linked CLT income |

`accounts.create` now accepts `kind: "fgts"`. Answers cite the source in words ("Caixa statement
of 10/21/2026", "distribution index approved by the Board of Trustees on 07/28/2026"), never the
tool name. The agent never asks for CPF/PIS.

## Out of scope

- Login to the FGTS app or Caixa (no public API; the PDF statement is the way).
- Progressive interest (only those who opted before 1971): store the rate and warn, without
  calculating.
- FGTS review lawsuits (TR × INPC): only cite ADI 5090; the app does not promise past correction.
- Birthday withdrawal anticipation as a product: record it as an existing debt, without
  simulating the contracting.

## Tests and validation

- **Synthetic** fixture with the same layout (never the real statement in the repository):
  distribution history on two lines, printed order ≠ date order, withdrawal + reversal, the
  10 → 21 transition of April/2024, deposit 650, coefficients with zero and positive TR.
- Property test: with the rule from "JAM algorithm", recalculate all the fixture's JAMs to the
  cent.
- Test of the birthday withdrawal table at the edges (500.00; 500.01; 20,000.00; 20,000.01).
- Comparison: synthetic flows with fixed IPCA/CDI/M2 checked by hand.
- Texts in pt-BR, en and es (`locales/*/fgts.ftl`, `fgts-…` keys).

## In the code

- Core: `fgts` module (`data.rs` with the birthday withdrawal table, the JAM and deposit days and
  the distribution indices; `statement.rs` with the statement reading and the JAM rule;
  `compare.rs` with the IRR and the series; `api.rs` with the 12 `fgts.*` commands). Migration
  V134 (`fgts` type, `fgts_account`, `fgts_entry`, `fgts_modality`). `accounts.create` refuses
  the type outside Brazil (`AccountKind::brazil_only`).
- Statement entries go in as an adjustment (outside the budget and the "uncategorized" list); a
  withdrawal with a destination account becomes a transfer; use for housing records the
  withdrawal on the FGTS account and the extra payment on the contract
  (`fgts.use_for_housing`, also through the amortization screen of a real estate contract). Code
  99 is treated as housing, with a warning in the preview.
- Checking: JAM recalculated by the rule from "JAM algorithm"; deposits by reference month ×
  `payslip.fgts` of the linked income (or of the same CNPJ); the `fgts_missing_deposit` alert in
  `reports.insights` only for reference months whose deadline fell before the last imported
  statement.
- Retirement: the projected balance (today's reais) enters as a life event in the plan
  (`goals.retirement`, `include_fgts: false` turns it off) and as a milestone on the life
  timeline.
- Built-in distribution indices: base year 2024 (0.02042919, Caixa FAQ) and 2025 (0.01868341).
  Earlier years wait until there are two sources.
- Tests with the synthetic statements in `crates/treasury-core/src/fgts/fixtures/` (all
  invented), including the paged "Histórico" layout; the reading was also checked with a real
  PDF, outside the repository.

## Limitations

- The "value for termination purposes" is the one printed on the statement; the app does not
  recalculate it.
- Built-in distribution indices only for base years 2024 and 2025.
