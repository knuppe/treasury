# Validation of the income tax (Brazil)

How Treasury checks the individual income tax (IR) calculation in Brazil: the rules it applies,
the source of each one, the official cases with known input and output (gold records) and what is
still open. The rule for changing a calculation is to have **two independent sources** (legal
text and Receita guidance, official example or gold record). The screen is Taxes and the IR report
is in Reports; the estimate of taxes paid in the year is in [taxes.md](../taxes.md).

## Where each calculation lives

| Area | File | Tests |
|---|---|---|
| Variable-income rules by year (rates, exemption, IRRF, DARF) | `tax/rules.rs` | `tax/rules.rs` |
| Monthly computation: average cost, day trade per broker, ordinary sales | `tax/engine.rs` | `tax/tests.rs` |
| R$ 20 thousand exemption, loss pools (ordinary, day trade, FII), IRRF, minimum DARF | `tax/engine.rs` | same |
| DARF due date | `tax/engine.rs` | `tax/tests.rs` |
| Annual report: dividends, Bens e Direitos (assets and rights), foreign | `tax/engine.rs` | `tax/tests.rs` |
| Average cost (purchase with costs, sale, split, bonus shares, return of capital) | `finance/average_cost.rs` | `finance/average_cost.rs` |
| Salary IRRF, INSS by bracket, 2026 reduction, inverse net→gross | `tax_burden/rules.rs` | `tax_burden/rules.rs` |
| Tables by month (IRPF, INSS, reducer) | `tax_burden/data/br.json` (`payroll`) | `tax_burden/tests.rs` |
| Annual estimate for work income (income statement, payslips or net → gross; 13th salary separate) | `tax_burden/mod.rs` | `tax_burden/tests.rs` |
| Income forecast: the month's INSS/IRRF, 13th salary, vacation third | `income/mod.rs` | `income/tests*.rs` |
| Imported payslip (reads IRRF, does not compute) | `income/payslip.rs` | `income/tests_layout.rs` |
| Fixed income: regressive IR, IOF, net redemption | `finance/fixed_income.rs` | `finance/fixed_income.rs` |
| Come-cotas (May/November, 15%/20%, per lot) | `portfolio/come_cotas.rs` | `portfolio/tests.rs` |
| Pension (regressive 35→10%, progressive 15% withheld; VGBL × PGBL) | `pension/mod.rs` | `pension/tests.rs` |
| Presumed withholding on dividends (JCP, US dividends) | `portfolio/records.rs`, `import/b3/plan.rs` | `portfolio/tests.rs` |

## Rules and sources

| Rule | How the app calculates | Sources |
|---|---|---|
| JCP (interest on equity) | 15% until 2025, 17.5% since 01/01/2026, by the credit date (ex-date) or the payment date, whichever is first; applies to typed, pending, calendar and quick dividends and to the B3 import; help texts | LC 224/2025 art. 8º (Lei 9.249 art. 9º §2º) and art. 14, III; Receita, Nota Cetad/Coest nº 009/2026 |
| Crypto in Brazil | Crypto in a reais account: monthly capital gain, exempt if the total sold in crypto in the month (Brazil + abroad) ≤ R$ 35 thousand, 15% to 22.5% per bracket, DARF 4600 on the last business day of the following month; card in the month's computation. Crypto in an account of another currency stays abroad | Lei 9.250 art. 22, II; Lei 8.981 art. 21; P&R IRPF 2026, question 653; Lei 14.754 art. 3º §1º, I |
| Investments abroad | Gross income; credit for the tax paid abroad up to 15% of each income item and up to the year's tax; losses offset gains and carry to following years (since 2024), without offsetting dividends | Lei 14.754 arts. 2º §1º, 4º and 9º; P&R IRPF 2026, questions 140, 142, 474 and 476 |
| Regressive pension | On the day a contribution completes 2, 4, 6, 8 or 10 years, the previous bracket still applies | Lei 11.053 art. 1º, I to VI; P&R IRPF 2026, question 187 |
| Bens e Direitos | Bitcoin 08.01, altcoins 08.02, stablecoins 08.03; savings 04.01; LCI/LCA/CRI/CRA/incentivized debenture 04.03; equity fund 07.04 (other funds 07.01); ETF 07.06; others 99.99 | IRPF 2026 program table (via mcp-imposto-de-renda); P&R 2026, question 473 (crypto); Santander IR guide (savings, equity fund); B3 Bora Investir (LCI/LCA) |
| Equity ETF in Bens e Direitos | 07.06; 07.09 was discontinued in 2025 and reclassified to 07.06 | Receita, IRPF 2025 presentation ("3 codes discontinued"); B3 Bora Investir; Lei 14.754 art. 18, II |
| Crypto losses in Brazil | do not offset (each sale is a capital gain) | P&R 2026 question 583 (IN SRF 84/2001 art. 2º, sole paragraph), 653 and 717 |
| "Dedo-duro" IRRF (withholding tip-off) | the year's balance goes to the return (Tax paid/withheld); the day trade one only until December, then refund | Lei 11.033 art. 2º §7º, III; RIR/2018 art. 849 §6º; P&R 706, 714, 715 |
| Loss in an exempt month and day trade at the R$ 20 thousand limit | the loss carries to the following months; only ordinary spot sales count toward the limit | P&R 707, 709, 711, 729 |
| Credit for tax paid abroad | only in countries with a treaty (38, Receita list of 08/28/2026) or reciprocity (Germany, US, United Kingdom) | Lei 14.754 art. 4º; P&R 136, 140, 142 |
| Crypto: where it is held | choice per account (automatic by the account's currency) and DARF 4600 in the DARF list | Lei 14.754 art. 3º §1º; P&R 653 |
| Sale larger than the position | warning in the month's computation | — |
| Annual adjustment return | preview of complete × simplified from the vault; **matches to the cent 60 of the 65 official cases** (the 5 rural activity ones are left out); 2026 with a cap of R$ 17,640 and the annual reduction | Lei 9.250 arts. 8º, 10, 11, 11-A, 12; 2025 and 2026 tables; P&R 012, 052, 179, 267, 332, 401 |
| Dividends above R$ 50 thousand/month | lists the months and the 10% withholding on the total, since 2026 | Lei 9.250 art. 6º-A; P&R High Incomes |
| IRPFM (minimum tax for high incomes) | estimate in the annual preview (2026 onward) above R$ 600 thousand, without the art. 16-B reducer | Lei 9.250 arts. 16-A and 16-B |
| Capital gain on real estate and goods | Lei 7.713, FR1, FR2, only property, reinvestment, small value and brackets; **matches to the cent the 7 GCAP cases in Brazil**; simulation in Assets | Lei 8.981 art. 21; Lei 9.250 arts. 22 and 23; Lei 11.196 arts. 39 and 40; P&R 574, 575, 656, 679 |
| PLR (profit sharing) | exclusive table recalculated in the year; warning on the PLR payslip with a different IRRF | Lei 10.101 art. 3º; 2025/2026 tables; P&R 179 |
| Carnê-leão | the month's calculation with the 2026 reduction, DARF 0190 | P&R 266, 267 |
| Late DARF | fine of 0.33%/day up to 20% and Selic + 1% interest; recorded when marked as paid | Lei 9.430 art. 61; P&R 063, 064 |

Also checked against the rule:

- Monthly IRRF 2025/2026 and the Lei 15.270 reduction (`tax_burden/rules.rs`): Receita's 5
  official examples match to the cent when the INSS is given (see 3.1).
- Reduction and simplified deduction also on the **13th salary** (`income/mod.rs`,
  `tax_burden/mod.rs`): Lei 15.270 art. 3º-A §3º orders the reduction to apply to the 13th; the
  eSocial FAQ accepts the simplified deduction on the 13th and on vacations (probably).
- INSS 2026 table (`tax_burden/data/br.json`): brackets 1,621.00 / 2,902.84 / 4,354.27 /
  8,475.55 (Portaria Interministerial MPS/MF 13/2026).
- Fixed income: regressive IR 22.5 / 20 / 17.5 / 15% by days and IOF 96%→0% in 30 days
  (`finance/fixed_income.rs`). MP 1.303/2025 lapsed on 10/08/2025, so LCI/LCA stay exempt and the
  regressive table did not change.
- Variable income (`tax/rules.rs`): 15% ordinary, 20% day trade and FII, R$ 20 thousand exemption
  only for spot shares, IRRF 0.005% and 1%, waiver up to R$ 1, DARF 6015 with a minimum of R$ 10,
  due on the last business day of the following month, day trade losses only against day trade.

## Gold records

Copyright: laws, decrees, INs and other official acts have no protection (Lei 9.610/1998 art. 8º,
I and IV, the Brazilian copyright law), so tables, rates and formulas may be copied. The gov.br
pages are CC BY-ND 3.0: do not copy the text, only the numbers, rewritten as our own data, with the
URL. The Perguntas e Respostas IRPF (IRPF Q&A) allows reproduction with citation of the source.

### Monthly IRRF 2026 (Receita)

[Examples of applying Lei 15.270/2025](https://www.gov.br/receitafederal/pt-br/assuntos/meu-imposto-de-renda/tabelas/exemplos-de-aplicacao-da-lei-15-270-2025), salary paid on 01/02/2026. The INSS is an **input** (the examples use the 2025 table):

| Gross | INSS | Deduction used | Base | IR before | Reduction | IRRF |
|---|---|---|---|---|---|---|
| 3,036.00 | 257.73 | simplified 607.20 | 2,428.80 | 0.00 | 0.00 | **0.00** |
| 4,000.00 | 373.41 | simplified | 3,392.80 | 114.76 | 114.76 | **0.00** |
| 5,000.00 | 509.60 | simplified | 4,392.80 | 312.89 | 312.89 | **0.00** |
| 6,000.00 | 649.60 | INSS | 5,350.40 | 562.63 | 179.75 | **382.88** |
| 7,607.20 | 0.00 | simplified | 7,000.00 | 1,016.27 | 0.00 | **1,016.27** |

The five are in `tests/fixtures/ir-br/irrf-mensal-2026.json` and are checked against
`monthly_irrf` with the 2026 table of `br.json` (`tax_burden/tests.rs`): they match to the cent
when the INSS is passed as input.

### Annual return (cases from the official program)

The 65 JSON cases of the annual return for tax year 2026 from the
[mcp-imposto-de-renda](https://github.com/ALASTecnology/mcp-imposto-de-renda) project
(AGPL-3.0-only, commit `6c37d2d`), with fictional data and values generated in the official
program, are kept unchanged and with attribution in
`crates/treasury-core/tests/fixtures/ir-br/dirpf-2026/`. The test `tax/tests_dirpf_golden.rs`
checks the annual adjustment preview against them: it matches to the cent 60 of the 65 (the 5
rural activity ones are left out). The Bens e Direitos table extracted from the program served as
the source for the codes.

### Official tables

- [2025 tables](https://www.gov.br/receitafederal/pt-br/assuntos/meu-imposto-de-renda/tabelas/2025) and [2026](https://www.gov.br/receitafederal/pt-br/assuntos/meu-imposto-de-renda/tabelas/2026): monthly, annual, PLR, dependent, simplified.
- [Lei 15.270/2025](https://www.planalto.gov.br/ccivil_03/_ato2023-2026/2025/lei/l15270.htm): monthly and annual reduction, 13th salary, dividends, IRPFM.
- [Lei 14.754/2023](https://www.planalto.gov.br/ccivil_03/_ato2023-2026/2023/lei/l14754.htm): foreign, come-cotas.
- [Lei 11.053/2004](https://www.planalto.gov.br/ccivil_03/_ato2004-2006/2004/lei/l11053.htm): pension.
- [Capital gain rates](https://www.gov.br/receitafederal/pt-br/assuntos/meu-imposto-de-renda/pagamento/ganhos-de-capital/aliquotas): 15 / 17.5 / 20 / 22.5%.

## Annual checklist (January, and when a new law comes out)

1. Monthly and annual IRPF table, deduction per dependent, simplified discount (monthly and annual cap), reducer (Lei 15.270).
2. INSS table (January ordinance) and minimum wage.
3. PLR table.
4. JCP rate, dividend rules and IRPFM.
5. Variable income, fixed income, come-cotas, crypto, foreign: check whether there was an MP or a law.
6. Bens e Direitos codes in the year's IRPF program.
7. National holidays for the DARF due date.
8. Run the tests; new cases from Receita go into the year's file.

## Limitations

- The annual reduction of Lei 15.270 in both models and the order relative to donations: the law
  does not distinguish models and there is no 2027 program yet; applied in both, after the table.
- Base of the reduction in carnê-leão (gross or after the cash book): the gross is used, as in
  Receita's examples for salaries; Annex X of IN 1.500 was not read.
- PLR for 2026: Receita only publishes the table "from May 2025".
- IRPFM: without the art. 16-B reducer (depends on the companies' balance sheets); the refund of
  the 10% on dividends for those who stay below R$ 600 thousand has no published mechanics.
- Effective rate and truncations: confirmed only by the program's cases (the official text does
  not define them).
- JCP: the credit date is still approximated by the ex-date (keeping the date would require a
  migration).
- Credit abroad: US state and local taxes give no credit, but Treasury does not separate them
  from the federal one.
- Capital gain abroad (goods bought abroad) and rural activity: outside the calculation.
- Vacations: the forecast treats the vacation month as a whole month of vacation (the month's
  gross + 1/3, with no separate salary), and so the separate IRRF equals the one computed
  together; if the forecast starts paying vacation and salary in the same month, the calculation
  needs to separate them (IN 1.500 arts. 29 and 62).
- Fixed-income ETF (07.08) is not detected: Treasury does not distinguish the ETF type; the label
  warns.
- Loss offsetting in come-cotas and the top-up of the regressive table on fund redemption are not
  calculated.
