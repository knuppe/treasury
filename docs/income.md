# Income and payslips

The app stores each family member's income sources, the imported payslips and a forecast of income for the coming months. It all lives in the **Family** screen of the sidebar, together with each member's accounts and cards ([members.md](members.md)).

With no income registered, the screen opens with a start guided by the income answer in the questionnaire ([profile.md](profile.md)): fixed income suggests salary or payslip and "pay yourself first"; variable suggests variable income, planning around the weak month and a 6 to 12 month emergency fund; benefits suggests the benefit as the base of the month; mixed suggests registering each part. The button already opens the income with the suggested type.

## Outside Brazil

The family's country ([countries.md](countries.md)) decides the deductions and the extra payments:

- **United States:** FICA (Social Security and Medicare) and 2026 federal tax, with the marital status from the family profile (single, head of household or joint filing; [countries.md](countries.md#united-states)), with no 13th salary or vacation bonus. The paystub is read in English (the period column, not the year-to-date one).
- **Portugal:** Segurança Social at 11% and 2026 IRS computed from the annual tax divided by the 14 payments; Christmas subsidy in December and holiday subsidy in the vacation month, one salary each. The recibo de vencimento is read in European Portuguese.
- **Other countries:** the app does not compute deductions; the net amount entered, or the one on the payslip, is used.

## Member profile

Each member has a relationship in the family (head, spouse, child, parent, other), date of birth, whether they are a dependent for income tax, occupation and notes. The app does not store CPF or other documents. Editing is in **Family › Profile** and in the command `household.update_member` (`profile`).

## Income sources

A member can have one or more income sources:

| Type | Formal | Payroll deduction |
|---|---|---|
| CLT (formal employment), public servant | yes | INSS (or RPPS) and IRRF |
| Retirement, pension | yes | IRRF (no INSS) |
| PJ (contractor with a company) | yes | no |
| Self-employed, informal, rent, social benefit, other | no | no |

Each income source stores:

- **Month's amounts:** reference gross and net.
- **Employer and CNPJ:** the CNPJ links payslips to the income source without having to choose.
- **Calendar:** payment day, 13th salary, vacation month, start and end.
- **Account and category:** where the money lands.
- **Variable income:** for commission or gigs, the forecast uses the average of the last 6 months of transactions in the income source's category.

## Payslips

The `payslip.generico.v1` layout ([layouts.md](layouts.md)) reads the PDF or text of the payslip (holerite, or contracheque). If the PDF has a password, it is requested at import and is not stored. The reader extracts from the payslip:

- **Period:** "Referência: setembro/2026" or "09/2026".
- **Type:** monthly, advance, 13th salary (1st or 2nd installment), vacation or PLR (profit sharing), read from the title (only lines without an amount: a "13 Salário Adiantado" item in the middle of a monthly payslip does not change the type).
- **Employer:** the CNPJ and the name that comes before it, without labels such as "Empresa:"; with the CNPJ alone on the line, the name comes from the label ("Empregador: …") or from the line above.
- **Earnings and deductions lines:** a signed amount ("-R$ 400,00", "400,00-", "(400,00)") is a deduction, whatever the item's name; without a sign, the names decide. INSS and IRRF are taken from the lines by name; the rest becomes "other deductions" (health plan, transit voucher, alimony…). The item code is only removed when followed by a space ("1/3 Férias" stays whole). The informational column (FGTS) is neither an earning nor a deduction.
- **Totals:** total earnings (the gross), total deductions, net and the month's FGTS, or the "Totais" line with the earnings and deductions columns. Without the totals, they are summed from the lines.
- **Check:** the lines are checked against the printed totals. An unsigned earning that is exactly what is missing from the deductions becomes a deduction, with a warning; if it still does not add up, the totals prevail and the preview warns. When gross minus deductions does not equal net, the preview also warns.

The employee's name and CPF are not stored.

### Import

The payslip goes to:

1. the chosen income source;
2. otherwise, the income source with the same employer CNPJ;
3. otherwise, a new CLT income source for the given member.

The most recent monthly payslip becomes the income source's reference gross and net. The payslip is also linked to the salary transaction: an income with the same net amount, from 5 days before the period until 10 days after the following month. A second payslip for the same income source, period and type is only accepted with "replace".

## Forecast

Per income source and per month, for the next 12 months (1 to 24 through the API):

- **Base:** the latest monthly payslip, if it is at most 3 months old. Otherwise, the gross and net entered. For variable income, the recent average.
- **Only gross entered:** INSS and IRRF come from the year's rules (`tax_burden`, with the income source's dependents).
- **Only net entered:** the gross is computed backwards.
- **Both entered:** the difference left after INSS and IRRF becomes "other deductions".
- **13th salary:** half in November, with no deductions. The rest in December, with INSS and IRRF computed on the whole 13th salary.
- **Vacation:** the vacation bonus (one third) enters in the chosen month, with the extra tax.
- **Start and end:** the income source only counts in the months in which it is active.

The summary brings, per member and for the family:

- the month-by-month forecast;
- the sum of the coming months;
- the payslips of the last 12 months (gross, INSS, IRRF, other, net), with the payments beyond salary that were included in the sum (PLR, 13th salary, vacation);
- the list of payslips of the last 12 months with month, type, gross and net (`payslips`). PLR and other one-off payments are saved on the employer's income source, but stay out of "actual × forecast" and of the forecast; the Family table shows the type of each payslip;
- the effective rate: (INSS + IRRF) ÷ gross on the payslips.

In **Taxes**, the work income of anyone with payslips in the year uses their values (marked "payslips"), in place of the estimate from the net amount. The annual income statement (informe de rendimentos), when entered, still has priority.

## Simulate dismissal (Brazil only)

On a CLT income source, the "Simulate dismissal" button (and the MCP tool `simulate_termination`, command `income.simulate_termination`) estimates what the person would receive on leaving the job. Nothing is saved; the official result is the employer's termination document (TRCT).

- **Inputs:** reason (dismissal without cause, resignation, agreement under art. 484-A, dismissal for cause, end of the trial period, trial period ended early by the company or by the employee), last day worked, notice (paid out, worked or not served on resignation), hire date, overdue vacation, FGTS balance for the job and how many times unemployment insurance was already received. Salary, hire date and dependents come from the income source; the FGTS balance comes from the FGTS account linked to it (value for termination purposes), when there is one.
- **Items:** salary balance (÷ 30), notice of 30 days + 3 per completed year up to 90 (Lei 12.506/2011), which projects the contract when paid out; half under the agreement. Proportional 13th salary and vacation by 15-day fractions, with 1/3 on vacation; dismissal for cause loses both (Súmula 171 of the TST, the labor court); resignation keeps vacation (Súmula 261). Art. 479 compensation and art. 480 deduction in the trial period; 30-day deduction for notice not served.
- **Deductions:** INSS and IRRF by the tables of the termination month (the same as the tax burden, `tax_burden/data/br.json`), only on the salary balance and, separately, on the 13th salary. Paid-out notice, paid-out vacation and compensation pay no INSS or IR.
- **FGTS:** deposit of 8% on the balance, 13th salary and paid-out notice; 40% fine (dismissal without cause, paid by the company) or 20% (agreement); withdrawal of 100% or 80% (agreement). Under the birthday withdrawal (saque-aniversário), the balance stays locked and only the fine is paid out.
- **Unemployment insurance:** only on dismissal without cause; waiting period of 12 months in the last 18 (1st claim), 9 in the last 12 (2nd) and 6 (3rd onward); 3 to 5 installments by months worked; amount by the average of the last salaries in the table for the date (`income/termination.rs`, `UI_TABLES`: 2025-01-11 and 2026-01-11, Lei 7.998/1990 and Res. CODEFAT 957/2022). **Update every January** with the adjustment published by the Ministry of Labor.
- **Total:** net termination pay + withdrawable FGTS + insurance, and how many months of today's net salary that covers.
- Not covered: collective agreements, overtime and commission averages, absences and advances (the 1st installment of the 13th salary is noted as a warning).

## Commands and tools

- **Read-only commands:** `income.list_sources`, `income.summary`, `income.payslip_preview` and `income.list_payslips`.
- **Write commands:** `income.create_source`, `income.update_source`, `income.delete_source`, `income.payslip_import` and `income.delete_payslip`. For the agent, all become proposals; the import and the deletions are sensitive.
- **MCP tools:** `get_income_summary`, `preview_payslip` and `import_payslip`. In the app's agent, the attached PDF goes in `attachments`.
- **Code:** `crates/treasury-core/src/income/` (migration V120). The UI mock (`api/mock/income.ts`) does not read PDFs and forecasts only from the values entered.
