# The family's country

Treasury was born for families in Brazil. It asks where the family lives and adapts to that
country: which data sources it queries, which inflation and which interest rate it uses in
comparisons, how it calculates salary deductions, which categories and ready-made rules it
creates and which screens it shows. The interface language is chosen separately
([i18n-design.md](i18n-design.md)). What disappears outside Brazil is in
[international.md](international.md).

- **Where it lives:** vault creation asks for the country (the base currency comes from it).
  Afterwards, it changes in **Settings › Family › Country**.
- **Core:** `household.country` (V123), module `country` (`country.get`, `country.set`) and
  `app.countries` (the list, with no vault open). Only the interface changes the country; the
  agent does not.
- **Vaults created before the country existed:** stay as Brazil.

## Three levels

| Level | Countries | What the app does |
|---|---|---|
| Full | Brazil | Everything: B3 (COTAHIST, brokerage notes, Área do Investidor), Tesouro Direto, CVM funds, Fundamentus, IR with DARF, INSS and IRRF on the payslip, FGTS, FIPE, Banco Central interest rates and portability. |
| Adapted | United States, Portugal, Spain, United Kingdom, Canada, France, Germany, Argentina, Chile | The country's currency, inflation and reference interest rate, 2026 payroll deductions, extra payments of the year, tax estimate, reading of the salary slip, categories and ready-made rules from there. |
| Generic | the other 53 countries of the index catalog | Currency, inflation, interest rates and exchange rates from the catalog (OECD, World Bank, ECB and central banks). No local payroll and tax rules. In the European Union, the VAT estimate also applies. |

In all countries outside Brazil, what only exists in Brazil is turned off and hidden.

## What changes when choosing the country

| Part of the app | Brazil | United States | Portugal | Spain | Generic |
|---|---|---|---|---|---|
| Base currency (new vault) | BRL | USD | EUR | EUR | the country's own |
| Reference inflation | IPCA (BCB) | CPI-U (FRED with a key; without a key, OECD CPI) | HICP (ECB, `HICP/M.PT…`) | HICP (ECB) | the catalog's |
| Reference interest rate (portfolio, Sharpe, % of the rate) | CDI | Effective Fed Funds (NY Fed, no key) | €STR (ECB) | €STR (ECB) | the catalog's |
| Exchange rate | PTAX | ECB, crossing through the euro | ECB | ECB | ECB, crossing through the euro |
| Payroll deductions | INSS and IRRF | FICA and federal tax | Social Security and IRS | Seguridad Social and IRPF | not calculated |
| Extra payments | 13th salary and vacation third | none | Christmas and vacation subsidies | extra pay in June and December | none |
| Public retirement in the planner | INSS under the EC 103/2019 rules (13 payments) | Social Security: PIA or AIME, ages 62 to 70 | Social Security old-age pension (14 payments) | only the amount entered | only the amount entered |
| Salary slip | payslip (CNPJ) | paystub | recibo de vencimento | nómina | the net amount entered |
| Taxes | full | state sales tax, federal payroll | mainland VAT and payroll | VAT in the European Union; outside it, the screen disappears |
| Reports › Income tax | yes | no | no | no |
| Debts: market rates and portability | yes | no | no | no |
| Portfolio: import from B3 and notes | yes | no | no | no |
| Assets: Fundamentus and announced dividends | yes | no | no | no |
| Physical assets: FIPE valuation | yes | no | no | no |
| Bank statement | OFX, CSV with preset | OFX/QFX, guessed CSV (mm/dd) | OFX, guessed CSV | OFX, Norma 43, guessed CSV | OFX, guessed CSV |
| Categories and ready-made rules | Brazil's | the US's | Portugal's | Spain's | only the global ones |

## Data sources

When the country is chosen, the sources that make no sense there are turned off and its own are
turned on (`country::apply_sources`). The app notes which ones it turned off
(`country.sources_off`) and only turns those back on: what the user turned off by hand stays off
in any country. In Brazil, all the sources stay on, as always.

| Country | Sources on |
|---|---|
| Brazil | all |
| United States | Yahoo, SEC EDGAR, FRED, NY Fed, ECB, OECD, World Bank, CoinGecko, DefiLlama |
| Portugal and European Union | Yahoo, SEC EDGAR, ECB, Eurostat, OECD, World Bank, CoinGecko, DefiLlama |
| Canada / United Kingdom | the global ones + Bank of Canada / ONS and Bank of England |
| Others | the global ones and those of the country's catalog |

BCB is turned off outside Brazil: the real (of an account in Brazil) comes from the ECB, and the
Focus projections for the IPCA are not downloaded.

The **NY Fed effective Fed Funds** (`nyfed:effr`, series `us_effr`, V123) is new: daily, no key,
through the public API of the Federal Reserve Bank of New York
(`markets.newyorkfed.org/api/rates/unsecured/effr/search.json`). It goes ahead of the FRED monthly
series, which requires a key.

## United States

2026 rules in `tax_burden/data/us.json` (`payroll`), with the sources:

- **Federal tax** (IRS, Rev. Proc. 2025-32): brackets for those filing alone (single), from 10%
  up to US$ 12,400 to 37% above US$ 640,600, with the standard deduction of US$ 16,100. The year
  is divided by 12 (the monthly table has an amount to deduct).
- **Filing status:** in US payroll it is what the employee declares on the W-4 form (step 1(c)
  and the step 2 box; IRS, Publication 15-T). The app takes it from the family record
  (`tax_burden::filing`):
    - no spouse registered and with dependents in the income: **head of household**, table 2 of
      the Rev. Proc. (10% up to US$ 17,700, 12% up to US$ 67,450…) and standard deduction of
      US$ 24,150;
    - with a spouse registered and no income of theirs in the app: **married filing jointly**,
      table 1 (10% up to US$ 24,800, 12% up to US$ 100,800…) and standard deduction of
      US$ 32,200;
    - with a spouse and both with income: each by the individual table, which is what the W-4
      step 2 box produces (half the joint brackets);
    - other members with their own income: individual table.
- **FICA:** Social Security 6.2% up to US$ 184,500 per year (SSA) and Medicare 1.45%, plus 0.9%
  above US$ 200 thousand (IRS, topic 560). FICA does not come out of the federal tax base.
- **Child credit:** up to US$ 2,200 per dependent in the year (IRS), deducted from the tax month
  by month. The total falls US$ 50 for each US$ 1,000 (or fraction) of income above US$ 200
  thousand in the year (US$ 400 thousand on a joint return; IRC § 24(b);
  `dependent_credit_phase_out`).
- **Not included:** state and local tax, the settlement of the joint return when the two have
  different incomes (the sum of the two individual tables comes out a little above), deductions
  before tax (401(k), health plan) and the Additional Child Tax Credit. With the imported
  paystub, its values apply.

Other numbers from the country used in the tips and by the agent: emergency fund of 3 to 6 months
(FINRA, Investor.gov), 401(k) with a limit of US$ 24,500 in 2026, IRA of US$ 7,500, the 50/30/20
rule.

**Paystub:** the reader recognizes `Gross Pay`, `Net Pay`, `Federal Income Tax`/`FIT`,
`State Income Tax`/`SIT`, `Social Security`/`OASDI`, `Medicare`, `401(k)`, `Medical`… It uses the
period column (the second to last), not the year one (YTD). The competence comes from the payment
date or the end of the period (`MM/DD/YYYY`). State tax goes together with federal in the tax
withheld. Name, SSN and EIN are not stored.

**Categories:** property tax, vehicle registration (DMV), 401(k) and IRA, children's savings
(529), Social Security, SNAP, SSI, tax credits (EITC and CTC), tax refund; no 13th salary,
vacation, JCP, IOF and meal voucher. **Ready-made rules:** DoorDash, Walmart, Costco, Kroger, CVS,
Walgreens, Shell, Chevron, Xfinity, Verizon, Uber and Lyft, electricity and water bills; Zelle,
Venmo and ACH transfers become a transfer.

## Portugal

2026 rules in `tax_burden/data/eu.json` (`payroll` with `region: "PT"`):

- **IRS** (CIRS art. 68.º): the table as worded by Decreto-Lei n.º 212/2026, of November 4,
  effective January 1, 2026: 12.2% up to €8,342, 15.2%, 20.7%, 23.6%, 30.6%, 34.6%, 43.1%, 44.6%
  and 48% above €86,634.
- **Specific deduction** (art. 25.º): the larger of €4,587.09 (8.54 × IAS) and Social Security.
  **Dependents** (art. 78.º-A): €600 per dependent, on the tax due.
- **Existence minimum** (art. 70.º): a reduction of the income, not a floor. Up to the reference
  value (€12,880) there is no tax; from there to `L` (€14,628 in 2026) the reduction falls €2.60
  per euro of gross; above `L`, €1.35, and it ends near €15,886 (`existence` in `PayrollRules`).
  The law's formula counts on the deduction of **general family expenses** at the limit of €250
  (art. 78.º-B), and the estimate does too (`personal_credit`). In a single-parent family
  (dependents in the income and no spouse registered) the limit is €335 (n.º 9).
- **Social Security:** 11% from the worker.
- **14 payments:** the annual tax is divided across the 12 salaries and the vacation and
  Christmas subsidies. The Christmas subsidy goes in December and the vacation one in the month of
  the vacation, each with one salary and the same deductions.
- **Not included:** the monthly withholding tables (the app estimates from the annual tax, which
  is what is settled in the return), the 50% withholding of November 2026, IRS Jovem, Azores and
  Madeira (own IRS and VAT rates), joint taxation and duodécimos (twelfths).

Tips for the country: emergency fund of 4 to 6 months (Banco de Portugal), Certificados de Aforro
(2.5% in October 2026), PPR with a 20% deduction up to €400, €350 or €300 depending on age,
Euribor on home loans, the 50/30/20 rule (DECO).

**Recibo de vencimento (salary slip):** the reader recognizes `Vencimento Base`, `Subsídio de
Alimentação`, `Segurança Social`/`TSU`/`CGA`, `IRS`/`Retenção na Fonte`, `Total Ilíquido`,
`Total Descontos`, `Líquido a Receber`, and numbers such as `1.234,56`, `1 234,56` and `1234,56`.
A title with "Subsídio de Natal" or "Subsídio de Férias" gives the type. NIF and NISS are not
stored.

**Categories:** home rent, IMI, IUC and inspection, tolls, transit pass, health insurance, mobile
phone, stamp duty, IRS, PPR, Christmas, vacation and meal subsidies, Social Security pension,
Caixa Geral de Aposentações, family allowance, RSI, local lodging, IRS refund. **Ready-made
rules:** Continente, Pingo Doce, Lidl, Mercadona, Galp, Repsol, EDP, Endesa, MEO, NOS, Vodafone,
Via Verde, Metro, Carris, CP, Glovo, Bolt, stamp duty and fees; MB WAY and SEPA transfers become a
transfer; meal subsidy top-ups (Edenred, Coverflex, Pluxee) go to the subsidy.

## Spain

Adapted on 10/22/2026 ([international.md](international.md)). 2026 rules in
`tax_burden/data/eu.json` (`payroll` with `region: "ES"`):

- **IRPF** by the general scale (state + common regional): 19% up to €12,450, 24%, 30%, 37%, 45%
  and 47% above €300,000, in 14 payments. Coming out of the base are Seguridad Social, the €2,000
  of "otros gastos deducibles" (art. 19.2.f) and the reduction for work income (art. 20: €7,302
  up to €14,852 net, falling to zero at €19,747.50). The personal minimum of €5,550 and the €2,400
  per descendant reduce at 19%. New fields in `PayrollRules`: `fixed_deduction`,
  `personal_credit` and `work_reduction` (points with linear interpolation).
- **Seguridad Social** of the worker: 6.50% (4.70% common contingencies, 1.55% unemployment,
  0.15% MEI, 0.10% training) up to the maximum base of €5,101.20 per month (Orden PJC/297/2026).
- **Extra pay** in June and December, one salary each, with the same withholding.
- **Not included:** each community's own scales, the foral regime (Navarra and Basque Country),
  the Canary Islands (IGIC), the withholding algorithm and calculated public retirement (the
  amount entered applies).
- **Deduction for those earning near the minimum wage** (additional provision 61.ª, Real
  Decreto-ley 5/2026): €590.89 on the tax up to €17,094 gross per year, falling 20% of what goes
  beyond that until it reaches zero at €20,048.45 (`earned_credit`). With it, the 2026 minimum
  wage (€1,221 in 14 payments) pays no IRPF.

**Nómina (salary slip):** the reader recognizes `Total devengado`, `Total a deducir`, `Líquido a
percibir`, `Contingencias comunes`, `Desempleo`, `Formación profesional`, `MEI`, `Retención
IRPF`, `Salario base` and the months in Spanish. NIF and affiliation number are not stored.
"Paga extra" in the title gives the type.

**Norma 43:** the account statement that Spanish banks export (records `11`, `22`, `23`, `33`,
`88`) is read like an OFX: account, period, final balance and each movement with the concept.

**Categories:** extra pay, IBI, IVTM and ITV, IRPF, pension plan, Seguridad Social pension,
Ingreso Mínimo Vital, IRPF refund and unemployment benefit; no 13th salary, vacation, FGTS, JCP
and IOF. **Ready-made rules:** Mercadona, Carrefour, Lidl, Alcampo, Eroski, Repsol, Cepsa,
Iberdrola, Endesa, Naturgy, Movistar, Vodafone, Orange, Renfe, Metro, EMT, Glovo, Just Eat, Cabify
and fees; Bizum and SEPA transfers become a transfer.

## United Kingdom, Canada, France, Germany, Argentina and Chile

Payroll is a 2026 approximation for a single employee, with the sources in
`tax_burden/data/payroll.json` (United Kingdom, Canada, Argentina, Chile) and in `eu.json`
(France, Germany); inflation and reference interest rate by the names of the index catalog.

| Country | Tax | Contributions | Extra payments | Slip | Retirement plans |
|---|---|---|---|---|---|
| United Kingdom | 20/40/45% after the personal allowance of £12,570, which falls £1 for every £2 above £100 thousand (60% effective up to £125,140); Scotland separate | National Insurance 8% up to £50,270, 2% above | — | payslip | ISA (£20 thousand), Lifetime ISA (£4 thousand), pension (£60 thousand), year starting April 6 |
| Canada | federal + Ontario added (19.05% to 46.16%), personal credits | CPP 5.95%, CPP2 4%, EI 1.63% | — | pay stub | RRSP ($33,810), TFSA ($7,000), FHSA ($8,000) |
| France | 1-part barème on 90% of taxable net (10% deduction up to €14,555), with the décote | 20.84% up to the PASS, 19.79% up to 4 PASS, 19.96% up to 8 PASS and 10.10% above; 2.85% (CSG and CRDS) do not come out of the tax base | — | bulletin de paie | PER and PEA (no fixed annual cap in the app) |
| Germany | § 32a EStG (class I) in linear segments, with the flat allowances, plus the Solidaritätszuschlag above €20,350 of tax; unemployment insurance and 4% of health insurance do not come out of the base | RV 9.3%, AV 1.3%, KV 8.75%, PV 1.8% with the caps | — | Lohnabrechnung | Riester (€2,100), Rürup |
| Argentina | Ganancias (annual scale ÷ 13), GNI and special deduction | 17% up to the maximum base | aguinaldo (SAC) in June and December | recibo de sueldo | — |
| Chile | Impuesto Único in UTM | 18.7% (AFP, health, cesantía) up to 90 UF | — | liquidación de sueldo | APV (600 UF) |

Categories with variants (`--gb`, `--ca`, `--fr`, `--de`, `--ar`, `--cl`) for property tax,
income tax, public and private retirement (and the aguinaldo in Argentina); ready-made rules with
each country's networks and the transfer rule (Faster Payments, Interac e-Transfer, virement/Wero,
Überweisung, CBU/CVU, TEF). The slip reader uses the labels of `income/payslip_specs.rs`.

## Switching country in a vault that already has data

- The country, the sources, the inflation, the reference interest rate and the tax estimate change
  right away.
- The base currency only changes if asked for (a box in the confirmation). Transactions stay in
  each account's currency; totals now convert to the new currency. The cost of investments
  already recorded stays converted at the rate of the time.
- Existing categories, rules and transactions do not change. The categories that are missing from
  the country's set appear as a suggestion in Settings › Categories.

## Research sources (October 2026)

- IRS: [inflation adjustments 2026](https://www.irs.gov/newsroom/irs-releases-tax-inflation-adjustments-for-tax-year-2026-including-amendments-from-the-one-big-beautiful-bill), [topic 560](https://www.irs.gov/taxtopics/tc560), [Child Tax Credit](https://www.irs.gov/credits-deductions/individuals/child-tax-credit), [401(k) and IRA 2026](https://www.irs.gov/newsroom/401k-limit-increases-to-24500-for-2026-ira-limit-increases-to-7500)
- SSA: [2026 Social Security changes](https://www.ssa.gov/news/en/cola/factsheets/2026.html)
- NY Fed: [Markets Data API](https://markets.newyorkfed.org/static/docs/markets-api.html)
- FINRA: [emergency fund](https://www.finra.org/investors/personal-finance/start-emergency-fund)
- Tax Foundation: [sales tax 2026](https://taxfoundation.org/data/all/state/2026-sales-tax-rates-midyear/)
- Portal das Finanças: CIRS [art. 68.º](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/cirs_rep/Pages/irs68.aspx), [art. 25.º](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/cirs_rep/Pages/irs25.aspx), [art. 70.º](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/cirs_rep/Pages/irs70.aspx), [art. 78.º-A](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/cirs_rep/Pages/irs78a.aspx), [EBF art. 21.º (PPR)](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/bf_rep/Pages/ebf-artigo-21-ordm-.aspx), [CIVA art. 18.º](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/civa_rep/Pages/iva18.aspx)
- Spain: [Ley 35/2006 del IRPF](https://www.boe.es/buscar/act.php?id=BOE-A-2006-20764), [reduction for work income (AEAT)](https://sede.agenciatributaria.gob.es/Sede/en_gb/ayuda/manuales-videos-folletos/manuales-ayuda-presentacion/irpf-2025/7-cumplimentacion-irpf/7_1-rendimientos-trabajo-personal/7_1_6-reduccion-obtencion-rendimientos-trabajo.html), [contribution bases and rates 2026](https://www.ineaf.es/tribuna/base-maxima-cotizacion-2026/)
- November 2026 check (cases and sources in [tax-validation-countries.md](tax-validation-countries.md)): [Rev. Proc. 2025-32](https://www.irs.gov/pub/irs-drop/rp-25-32.pdf), CIRS [art. 70.º](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/cirs_rep/Pages/irs70.aspx), [French barème for 2026](https://www.service-public.gouv.fr/particuliers/vosdroits/F1419), [§ 32a EStG](https://www.gesetze-im-internet.de/estg/__32a.html), [income over £100,000](https://www.gov.uk/income-tax-rates/income-over-100000)
- Government of Portugal: [IRS reduction of September 2026](https://portugal.gov.pt/api/media/edge/Project/Portal-do-Governo/Portal-do-Governo/gc25/documentos/Financas/20260917_IRS_Set2026_simulacoes.pdf)
- Segurança Social: [employees](https://www.seg-social.pt/trabalhadores-por-conta-de-outrem)
- ECB: [Data Portal](https://data.ecb.europa.eu) (€STR, HICP, Euribor); IGCP: [Certificados de Aforro](https://www.igcp.pt/en/aforristas/produtos-de-aforro/certificados-de-aforro/ficha-tecnica-certificados-de-aforro-serie-f)

## Limitations

- More provinces and states (Canada beyond Ontario, Scotland) and the details missing in each adapted country.
- US state tax and joint return; IRS monthly withholding tables.
- CPI-U without a key through the BLS API; INE's CPI in Portugal.
- Local exchanges as a reference (S&P 500 in the US, PSI in Portugal) instead of the Ibovespa.
