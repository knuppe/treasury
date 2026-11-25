# Income received (income ledger)

The income ledger answers "how much did this asset pay me while I held it": for every
distribution (FII and Fiagro income and amortizations, dividends, JCP) it takes the position of
each account at the end of the record date and the amount per share of the official notice, and
compares the result with what is recorded. It appears in the Income tab of Portfolio and of each asset
page: on top, what the sources say was received (since the first purchase, on the asset page),
what is still to be paid and the years in one line, with a notice of what to check; below, one
list at a time. In Portfolio: by asset since the purchase, by month, or the records of a period.
On the asset page: every distribution (quantity, amount per share, gross, tax, net, source and
status), the ones to check, the upcoming ones, or the records themselves to edit or delete.

## Sources, in order

1. **Fundos.NET** (CVM and B3 disclosure system), for FIIs and Fiagros: the structured notice
   "Rendimentos e Amortizações" that the fund files for each distribution, with the record date
   ("data-base"), the exact amount per unit (up to nine decimals), the payment date, the reference
   month and whether the income is tax exempt. It covers the whole life of the fund. See
   [data-sources.md](../data-sources.md#fundosnet-official-income-notices-of-the-listed-funds).
2. **B3, listed companies and funds**: official dividends, JCP and fund distributions. For funds,
   only the last ~12 months, so it becomes a check of the notice; for companies it is the
   reference (full history of amounts and record dates; the payment date only for the last
   ~12 months, completed by the agenda when it has it).
3. **Agenda** (Fundamentus, Yahoo): only when no official source describes the distribution. These
   distributions are not recorded by themselves: they show up unchecked in the review of the fixes
   and in Record dividends › To confirm, to be checked against the statement first.

A value from a secondary source never replaces the official one, even when they are within 2%:
Fundamentus keeps two decimals (0.055 becomes 0.06) and Yahoo rewrites past amounts
(0.09 becomes 0.08993).

## Rules

- **Who is entitled**: whoever holds the asset at the end of the trading session of the record
  date ("data-com", the last day with the right); from the next business day the asset trades
  "ex". The trade date counts, not the settlement. Sources that only report the ex date (Yahoo,
  Tiingo, EODHD) use the position of the day before.
- **Same distribution, different dates**: the record date and Yahoo's ex date are one session
  apart, which can be five calendar days around Carnival. Dates are compared in business days of
  the B3 calendar (weekends and national holidays off), up to two apart.
- **Quantity**: replayed by the portfolio engine on the record date, with splits, reverse splits,
  bonus shares and ticker changes, so it is always in the unit of that date and no amount is
  adjusted. Positions already closed count for the period they were held.
- **Gross**: quantity × amount per share, rounded to the cent.
- **Tax withheld**: JCP at 15% until 2025 and 17.5% from 2026-01-01 (LC 224/2025), by the date of
  the credit or the payment, whichever comes first. FII and Fiagro income is exempt for
  individuals (Lei 11.033/2004, art. 3º, III, with the conditions of § 1º as written by Lei
  14.754/2023); when the notice says it is not exempt, 20% is withheld (Lei 8.668/1993, art. 17).
  The fraction of a cent is dropped, as the custodians do (a JCP of R$ 141.98 at 17.5% withholds
  R$ 24.84).
- **Amortization**: return of capital, recorded as its own kind and kept out of the yield.
- **Subscription receipts**: notices come per security (unit and receipts, by ISIN); the
  receipt's amount stays with the receipt's code.

## Matching and fixes

Each recorded distribution pairs with at most one expected one of the same family (JCP,
amortization, other income), with the record date up to two business days apart or, without a
record date, the payment within the window of the expected one. The status of each line:

- **Matches**: net and gross within a cent.
- **Different tax**: same net, gross and tax recorded at another rate (a 2026 JCP recorded at
  15%). Fix: gross and tax of the official calculation, same net.
- **Different amount**: fix to the official amount, except when the record came from the B3
  statement import, whose credited amount prevails.
- **Not recorded**: paid and missing. Fix: record it.
- **Repeated**: a second record of a payment already paired (typed again from a suggestion, or
  with a typo in the year). Fix: delete it.
- **Upcoming**: paid after today ("a receber").
- **No source**: recorded and described by nothing; nothing is proposed.

**Review and fix** shows each fix with a checkbox and applies the chosen ones in a single change
set, with Undo. Distributions recorded there do not move the brokerage cash (they are history).
Rebuilding the whole portfolio turns the **automatic recording** on: after each data collection,
official distributions already paid that are missing are recorded by themselves (author "system",
origin "official"), with the brokerage cash entry for payments from the start of the automatic
recording on. A record of the automatic recording that the family deletes or undoes does not come
back. The automatic recording can be turned off on the same screen.

The B3 statement import (Investor Area, movements) recognizes a credit already recorded from the
official notice (payment up to three days apart, net within a cent) and does not import it again.

## Credit in the account

A distribution recorded as received without a cash entry did not move the brokerage balance in the
app, even when its amounts match the source. The ledger tells the two cases apart:

- **Pending credit** (`cash_pending`): paid from the start of the automatic recording on (or in the
  last 90 days while it is off). The line shows "No credit in the account", the header counts
  them and **Record the credit** creates the income transaction of each one on its payment date
  (`portfolio.income_cash`, one change set, with Undo). When the account's statement is imported
  instead, the row with the same date and amount as a credit the portfolio recorded is a possible
  duplicate and stays out by default.
- **History**: older payments, recorded without cash on purpose (the rebuild). They only carry a
  note and wait for nothing.

A record whose payment date is implausible (a typo in the year, from before the app refused such
dates) is listed as repeated or without a source, counts nothing and opens no year or month in the
totals.

## Totals

By year, by month and by asset, in the family's base currency: what the sources say was paid
(the recorded amount when it came from the B3 statement or when nothing official describes it),
what is recorded today and what is still to be paid. Until the fixes are applied, "received" and
"recorded" differ, and the screen shows both.

## In the code

- `research/fundos_net.rs`: list and documents, both XML layouts, the collection
  (`SyncSource::FundosNet`, `SourceId::FundosNet`), storage in `research_fund_notice` (migration
  V3, outside the change history and the backups).
- `portfolio/pending.rs`: one candidate per distribution, official sources first, dates in
  business days.
- `portfolio/income_ledger.rs`: the ledger (`portfolio.income_ledger`), the rebuild
  (`portfolio.income_rebuild`), the switch (`portfolio.income_auto`) and the automatic recording
  at the end of `research.sync`.
- `portfolio/records.rs`: `withholding` (tax by kind, exemption and date),
  `update_income_values` and `add_income_cash` (the credit of a record made without one).
- `portfolio.pending_income` is built on the ledger: with the automatic recording on, only the
  distributions without an official source wait there.
- MCP: `get_income_ledger` (read only).
- Interface: `features/portfolio/IncomeLedger.tsx`.

## Limitations

- Dividends above R$ 50,000 a month from the same company to the same person have 10% withheld
  from 2026 (Lei 15.270/2025, art. 6º-A), except profits approved until 2025: the ledger does not
  apply it.
- The exemption of FII income for a holder of 10% or more of the units (or 30% for related
  people) is not checked: the ledger uses what the notice says.
- Shares lent through B3 receive a reimbursement instead of the dividend; the ledger counts them
  as held.
- Funds without a CNPJ in the asset registry nor in the CVM data are not collected from
  Fundos.NET; their income comes from B3 and the agenda.
- B3's history of companies has no payment date beyond ~12 months: without the agenda, the record
  date is used and the line says so; those distributions are not recorded automatically.
