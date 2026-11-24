# Total effective cost (CET) of loans

Study of 24/11/2026: why most loans showed no CET, how Brazilian rules define it, how other
countries define the equivalent rate, the plan, and how it was implemented (see
[Implementation](#implementation)). Numbers and examples are made up.

## Why loans showed no CET (before this change)

Treasury has two CET numbers on each loan, and neither is the one the user expects.

- **Contracted CET** (`loan.cet_annual`) is a free text field in the loan dialog ("From the contract
  or the proposal. Optional."). Nothing computes it: the statement importer does not read it, no
  MCP tool writes it, and the list shows "—" when it is empty. So the gap is a **missing input
  with no calculation behind it**.
- **Actual CET** (`LoanView.realized_cet`, `debt/mod.rs`) is an XIRR over the principal on the
  contract date, the installments paid, and today's balance. It is computed, but it has four
  defects that make it read too low:
  1. **Extra payments are left out.** The flow uses `paid_amount` or `total` and ignores
     `extra_payment`, so every prepayment (own money or FGTS through `use_fgts_for_housing`)
     disappears from the cost. A contract with prepayments can show half its real cost.
  2. **Balance drops with no payment are left out.** When an imported statement shows the balance
     falling with no matching amortization or extra payment, the money that left the borrower is
     not a flow either.
  3. **The open balance is dated today without accrued interest.** The balance after the last
     installment due is put at today's date. Between two due dates this lends the borrower money
     at 0%, so the rate drops by one or two points in the middle of the month.
  4. **Upfront costs do not exist in the model.** IOF, the registration fee (tarifa de cadastro),
     appraisal, registry and single-premium insurance have no fields. The principal is treated as
     money received in full, which is the main thing CET exists to correct.

There is also no computed contract CET, although the schedule Treasury already builds is enough to
compute the number the bank must disclose and to check the one the user typed.

## The Brazilian rule

Resolução CMN nº 4.881, of 23/12/2020, in force since 01/02/2021, defines the CET. It revoked
Resolução nº 3.517/2007 (the older rule) and Resoluções 3.909/2010 and 4.197/2013. The points that
matter for Treasury:

- **Scope (art. 1º, 9º):** credit and financial leasing offered to individuals, sole
  proprietors, micro and small companies. Rural credit and foreign-funds onlending are out.
- **What goes in (art. 3º):** the credit granted and everything charged to the borrower:
  amortization, interest, fees (tarifas), taxes (IOF), insurance and other expenses tied to the
  operation, including third-party services the institution hires and the borrower pays, **even
  when they are not added to the credit amount**.
- **Formula (art. 4º):**

  ```
  Σ (j = 1..N)  FCj / (1 + CET) ^ ((dj − d0) / 365)  −  FC0  =  0
  ```

  - `FC0`: the credit granted, minus the expenses and fees paid upfront (the money that actually
    reaches the borrower).
  - `FCj`: each amount charged, periodic or not (amortization, interest, fees, taxes, insurance).
  - `dj − d0`: calendar days between the release of the credit and each payment; the exponent
    divides by 365.
  - The result is an **effective annual rate**, disclosed **with two decimal places**, rounded by
    ABNT NBR 5891 (a trailing 5 goes to the even digit; the standard itself is paid and was not
    read here, and with rates computed in floating point an exact midpoint practically never
    happens).
- **Floating rates (art. 5º):** floating rates and price indices (CDI, TR, IPCA…) are **not**
  part of the calculation; they are only stated in the breakdown. A TR + 9% contract has the CET
  of the 9% plus the charges.
- **Revolving credit (art. 6º):** overdraft, deposit advances, discount and revolving lines use a
  30-day term over the agreed limit.
- **Disclosure (art. 7º, 8º):** the bank must show the CET and a breakdown of every flow before
  contracting, put the breakdown in the contract, and state the CET in any advertising that shows
  the interest rate.

### What this means in numbers

A made-up personal loan: R$ 10.000,00 released, R$ 850,00 of IOF and registration fee added to
the contract (R$ 10.850,00 financed), 1,90% a month, 12 Price installments of R$ 1.019,68, contract
on 10/03/2026, first installment on 10/04/2026.

- Interest rate alone: 25,34% a year (effective).
- CET with `FC0 = 10.850,00` (what Treasury does today): 25,19%. The costs vanish.
- CET with `FC0 = 10.000,00` (the rule): **46,49%**.
- Same contract signed on 25/03 with the first installment still on 10/04 (a short first period
  with a full month of interest): 51,45%.

The second and third lines are the whole point of CET; today Treasury can only show the second.

## Other countries

All the rates below answer the same question as CET (the yearly price of the money, all charges
included), but not all of them produce the same number.

- **European Union: APRC (TAEG, TAE).** Spain's Ley 16/2011, which transposes the Consumer Credit
  Directive 2008/48/EC, uses the same structure as the Brazilian formula: drawdowns equal
  repayments discounted at `(1 + X) ^ −t`, with `t` in years (365 days, 366 in leap years, or 12
  months of 365/12 days), an **effective** rate, rounded to at least one decimal. It includes
  interest, fees and taxes; insurance only when the credit depends on it; notary fees are out. It
  must be disclosed before the consumer is bound, in the contract and in advertising with cost
  figures. Portugal (TAEG), France (TAEG), Germany (effektiver Jahreszins) and the other member
  states transpose the same directive; the new directive (EU) 2023/2225 replaces 2008/48/EC. I could
  not open EUR-Lex, Banco de Portugal or the other national texts from here, so these countries
  are assumed to follow the Spanish text until checked.
- **United States: APR (Truth in Lending Act, Regulation Z, 12 CFR 1026.22 and Appendix J).** The
  equation is actuarial too, but the disclosed APR is **nominal**: the rate per unit period (one
  month for monthly loans) times the number of periods in a year. All months count as equal. The
  finance charge (1026.4) includes interest, points, loan fees and required insurance; taxes,
  voluntary insurance and bona fide real-estate fees (title, notary, appraisal, document
  preparation) are out. A US APR is accurate within 1/8 of a point (1/4 for irregular loans). In
  the example above the US APR would be **39,00%**, not 46,49%: same loan, different convention.
- **Canada: APR (Financial Consumer Protection Framework Regulations, SOR/2021-181, s. 32 to 40).**
  It replaced the Cost of Borrowing Regulations in 2022. The text I read defines the APR as equal to
  the interest rate when interest is the only cost, and requires it in the initial disclosure when
  it differs; it does not give the equation, so it stays unverified here.
- **United Kingdom (APR, FCA CONC App 1.2), Argentina (CFT, BCRA) and Chile (CAE, Ley 20.555):**
  the official pages did not load from here. They are listed as gaps, not as findings.

### Does it make sense outside Brazil?

Yes. Every country I could check (Brazil, Spain, the United States, Canada) requires lenders to
disclose an all-in annual rate, and
the user's question is the same everywhere: "how much does this loan really cost, and is it cheaper
than the other offer?". Two adjustments are needed:

- **The name.** "CET" means nothing to a user in Spain or the United States. The label follows
  the family's country: CET (BR), TAEG (PT, FR), TAE (ES), effektiver Jahreszins (DE), APR (US,
  GB, CA), CFT (AR), CAE (CL), and a generic "annual effective cost" elsewhere. The English
  translation stops saying "CET".
- **The convention.** Brazil and the EU publish an effective rate; the US publishes a nominal one.
  To check the number on a US contract, Treasury must compute the US APR the US way, otherwise the
  user sees 46,49% where the paper says 39,00% and concludes one of them is wrong.

## Plan

All four phases below are implemented; [Implementation](#implementation) describes the result.
Each phase is small enough to review on its own. Money stays in integer cents; the rate is
computed in `f64` like the other XIRRs and stored nowhere (it is derived on read). Tests use
made-up contracts.

### Phase 1: fix the actual CET (no migration)

- Add `extra_payment` to the paid flows, on the date of the payment.
- Put the open balance on the date of the last installment paid, not today. (The alternative,
  balance today plus interest accrued pro rata, is more exact but needs the daily convention of
  each system; the simpler one is enough for a yearly rate.)
- Treat a balance drop with no explaining payment in imported statements as an amortization
  outside the schedule, and list it in a note so the user can confirm or fix it.
- Same fixes in the interface mock (`api/mock/debt.ts`).
- Tests: a contract with prepayments, one measured in the middle of a month, one with an
  unexplained drop.

### Phase 2: computed contract CET (no migration)

- New `debt::cet` module with the art. 4º formula: `FC0` on the contract date, each scheduled
  installment (amortization, interest, insurance, fees) on its due date, days / 365, result rounded
  half to even to two decimals for display.
- Floating-rate contracts follow art. 5º: the schedule is generated with the index at zero, so
  only the fixed rate and the charges count. The screen says the index is not included.
- `LoanView` gains `cet_computed` and a short list of what was left out ("IOF not informed").
  When the user typed a contracted CET, show both and flag a gap above 0,10 point with a hint
  ("the bank's number includes costs Treasury does not know; check IOF and fees").
- MCP: `list_loans` returns `cet_computed`; `compare_loan_rates` and the portability simulation
  compare by CET when it exists.

### Phase 3: upfront costs (one migration, V2)

- New loan fields, all optional, in cents: `iof_amount`, `upfront_fees` (registration, appraisal,
  registry, single-premium insurance), and `costs_financed` (true when they were added to the
  principal, false when paid from the borrower's pocket).
- `FC0 = principal − upfront costs` in both cases: when financed, the principal already contains
  them; when paid in cash, the borrower received the principal but paid the costs on day zero.
- The loan dialog gets a collapsed "Upfront costs" group; the statement importer and the
  document OCR fill them when the contract shows them.
- The actual CET from phase 1 uses the same `FC0`.

### Phase 4: other countries

- Label per country (list above), in all four locales and the glossary.
- US: compute the nominal APR (unit period × periods per year, Appendix J) next to the effective
  rate, and show the APR as the main number when the family's country is the US.
- Other countries use the effective rate with their local name. Before claiming equivalence for a
  country, read its official text (EU directive annex, FCA CONC App 1.2, BCRA, CMF/Ley 20.555).

### Not in this plan

- Estimating IOF when the contract does not show it. The IOF rates live in Decreto 6.306/2007 and
  change by decree; estimating would need an official table kept up to date. The user types the
  amount from the contract instead.
- CET for overdraft and revolving lines (art. 6º: 30 days over the limit). The overdraft already
  stores the bank's CET; computing it is a separate, small follow-up.

## Implementation

The user types the rate printed on the contract, and Treasury shows its own calculation next to
it, step by step, so the user can check one against the other.

### Inputs

- The contract as registered: principal, contract date (`d0`), first due date, term, rate,
  system, grace, monthly insurance and fees, housing insurance (MIP, DFI) and the floating index.
- Upfront costs (`loan.upfront_iof`, `loan.upfront_fees`, `loan.upfront_insurance`, migration V2),
  all optional, in cents. Validation: none negative, and together below the principal.
- The CET typed from the contract (`loan.cet_annual`, as before), optional.

### Contract CET (`debt::cet::contract`)

1. Rebuild the schedule of the contract **as signed**: the original principal and term, no extra
   payments, no statement rebase, and the floating index at zero (art. 5º). For TR, IPCA, IGP-M,
   INPC, INCC and savings the correction factors are dropped; for CDI and Selic each period
   keeps only the spread, `(1 + s)^(du/252)` or `1 + s × dc/360`, on the business-day due dates.
   Grace, pro rata of the first period and the interest basis follow the contract.
2. `FC0 = principal − (IOF + fees + single-premium insurance)`, dated on the contract date.
3. `FCj` = the total of each installment (amortization, interest, insurance, fees) on its due
   date.
4. Solve `Σ FCj / (1 + x)^((dj − d0)/365) = FC0` with the XIRR of `finance::returns`
   (Newton-Raphson, then bisection), which uses exactly this day count.
5. `annual_pct` is `x` in % with two decimals, rounding a midpoint to the even digit (NBR 5891);
   `monthly_pct` is `(1 + x)^(1/12) − 1`, also with two decimals.
6. `us_apr`: the same flows, monthly unit period, whole months counted back from each payment and
   the odd days over 30 (Appendix J, (b)(5)(ii)), each flow discounted by `(1 + f·i)(1 + i)^t`, and
   `12 × i`. The time counting was read in the regulation's text; the general equation of
   (b)(8) is an image in every official copy that could be opened, so the `(1 + f·i)` form for the
   odd days follows its usual reading. The difference to a fully compounded fraction is far below
   the 1/8 point tolerance of 1026.22.
7. `contracted_gap_pp` = typed − computed, in percentage points. The screen warns above 0,10
   point and suggests the likely cause (costs not registered when the bank's number is higher).

The report also carries the sums of each component, the total paid (installments plus upfront
costs), the number of installments, `d0`, the last date and `N` in days, and the index left out.

### Realized CET (`debt::cet::realized`)

- Day zero: the same `FC0` (principal minus upfront costs).
- Each installment marked paid, on the date it was paid: the amount paid (or the installment's
  total) **plus its extra payment** plus any **balance drop with no payment** above R$ 50,00
  (previous balance + correction − amortization − extra − new balance), checked only between
  consecutive installments.
- While a balance is owed: that balance, on the due date of the last installment paid (no more
  "balance today" with no interest accrued).
- The drops it counted come back in `realized_drops`, and the screen lists them so the user can
  record the payment (FGTS, prepayment) or fix the contract amount.

### Screen and assistants

- Card 06.3 (rate and cost): "CET do contrato" (typed, or "não informado") and "CET pelo nosso
  cálculo". Card 06.4: "CET do que já foi pago" and the drops it counted.
- Card 06.4b, "Como calculamos o CET": the formula, step 1 (contract amount − each upfront cost =
  amount released), step 2 (installments from `d0` to the last date, each component and its share
  of the total paid), step 3 (our rate a year and a month, the US APR in the US, the contract's
  rate), the warning when they disagree, the index left out, a hint when no upfront cost is
  registered, and the rule it follows.
- The list's column shows the typed rate, or ours with "≈".
- The loan dialog has a "Custos na contratação" group (IOF, fees and registration, single-premium
  insurance).
- The name follows the family's country: CET (BR), TAEG (PT, FR), TAE (ES), Effektiver
  Jahreszins (DE), APR (US, GB, CA), CFT (AR), CAE (CL); elsewhere CET in Portuguese, APR in
  English and TAE in Spanish. In the US the headline is the nominal APR.
- MCP: `list_loans` returns `cet`, `upfront` and `realized_drops`, described in the tool.

### Tests

- `debt::cet::tests`: the examples of this document (25,19%, 46,49%, 51,45%, US APR 39,00%),
  the Appendix J time counting, the rounding, a balance drop counted as paid, the open balance on
  the last installment paid, and a gap in the paid installments that is not a drop.
- `debt::tests`: contract CET with upfront costs through the API (and the refusal of costs above
  the principal), the TR left out of a mortgage, and the realized CET with an extra payment.
- Interface: the mock mirrors the calculation (`api/mock/debt/cet.ts`, tested in
  `api/mock/ledger/loanCet.test.ts`), and `Dividas.test.tsx` checks the cards.

## Sources

- Resolução CMN nº 4.881/2020 (official PDF, Banco Central do Brasil):
  <https://www.bcb.gov.br/content/estabilidadefinanceira/especialnor/Resolu%C3%A7%C3%A3o4881.pdf>
- Resolução nº 3.517/2007 (revoked, for history):
  <https://normativos.bcb.gov.br/Lists/Normativos/Attachments/48005/Res_3517_v4_P.pdf>
- Spain, Ley 16/2011 de contratos de crédito al consumo, art. 6, 9, 10 and Anexo I (BOE):
  <https://www.boe.es/buscar/act.php?id=BOE-A-2011-10970>
- United States, Regulation Z, 12 CFR 1026.4, 1026.22 and Appendix J (CFPB):
  <https://www.consumerfinance.gov/rules-policy/regulations/1026/22/>,
  <https://www.consumerfinance.gov/rules-policy/regulations/1026/j/>,
  <https://www.consumerfinance.gov/rules-policy/regulations/1026/4/>
- United States, Appendix J text (time counting, unit periods), Cornell LII copy of the CFR:
  <https://www.law.cornell.edu/cfr/text/12/appendix-J_to_part_1026>
- Canada, Financial Consumer Protection Framework Regulations, SOR/2021-181:
  <https://laws-lois.justice.gc.ca/eng/regulations/SOR-2021-181/index.html>
