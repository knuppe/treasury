# Tax validation outside Brazil

How the payroll tax rules of the adapted countries are checked against official
sources, the known-result cases that became tests, and what the estimate still does not
consider. The payroll lives in `tax_burden/data/us.json`, `eu.json` (Portugal, Spain, France,
Germany) and `payroll.json` (United Kingdom, Canada, Argentina, Chile), is computed by
`tax_burden/rules.rs` and used in the income forecast (`income/mod.rs`) and on the Taxes screen; the
description of each country is in [countries.md](countries.md). The Brazil validation is in
[income-tax-validation.md](brazil/income-tax-validation.md). The last check is from 2026-11-17, with the 2026 rules.

The tax notice on the screens (`TaxNotice`, keys `termos-aviso-imposto-limites-us` and `-pt`) says
what the estimate still does not consider in the United States and Portugal, as the Terms
promise (5.1). Fixing a limit includes removing its passage from the notice.

## What was checked and corrected

### United States

Sources: [Rev. Proc. 2025-32](https://www.irs.gov/pub/irs-drop/rp-25-32.pdf) (sections 4.01, 4.05
and 4.14), IRC § 24(b), [Publication 15-T of 2026](https://www.irs.gov/pub/irs-pdf/p15t.pdf)
(Worksheet 1A and annual tables) and [Publication 501](https://www.irs.gov/publications/p501).

- **Right since the first version:** the seven brackets for single filers, the US$ 16,100 standard
  deduction, the US$ 2,200 credit per child, Social Security of 6.2% up to US$ 184,500 and
  Medicare of 1.45% plus 0.9% above US$ 200 thousand, with FICA outside the base.
- **Filing status:** the estimate always used the single table. In the US payroll the
  table is the one for the filing status the employee declares on Form W-4, and Publication 15-T
  gives a table for each: the sum of the line 1g amount and the 0% bracket is the standard deduction
  (US$ 16,100, 24,150 or 32,200) and the brackets are those of the Rev. Proc. The app now takes the filing
  status from the family registry, through the same questions as the W-4:
    - no spouse and with dependents: head of household (table 2);
    - with a spouse with no income in the app: married filing jointly (table 1);
    - with a spouse and both with income: the single table for each one (step 2 checkbox).
- **Child credit:** it had no income limit. It now drops US$ 50 for each US$ 1,000 (or
  fraction) of income above US$ 200 thousand in the year (US$ 400 thousand for joint filers).

### Portugal

Source: Código do IRS (the Portuguese income tax code) at the Portal das Finanças, articles
[68.º](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/cirs_rep/Pages/irs68.aspx)
and
[70.º](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/cirs_rep/Pages/irs70.aspx)
and
[78.º-B](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/cirs_rep/Pages/irs78b.aspx).

- **Table in art. 68.º:** the reduction of September 2026 became law with Decreto-Lei
  n.º 212/2026, of 4 November, effective 1 January. The limits, rates and average rates
  on the official page are the ones the app already used.
- **Minimum subsistence (art. 70.º):** it was treated as a floor (the tax did not let the net amount fall
  below 920 € per payment), which gave 1 120 € of IRS in the year for someone earning 1 000 € per
  month. The rule in force is a deduction from income with a gradual phase-out, and the official
  result is 491.90 €. The formula is in [countries.md](countries.md#portugal).
- **General family expenses (art. 78.º-B):** 35% of expenses with an invoice, up to 250 € per
  person (45% up to 335 € for a single-parent family). The formula in art. 70.º counts on the limit of
  250 €, and the IRS statistics from the Autoridade Tributária (incomes of 2021) give 1,601
  million euros for 4,890,766 households, 327 € per household, above the limit for one person
  alone. The estimate now deducts the limit at all income levels: 250 €, or 335 € when the income
  has dependents and the family has no registered spouse.

### Spain

Source: Ley 35/2006 and Real Decreto-ley 5/2026, art. 28
([Agencia Tributaria bulletin](https://sede.agenciatributaria.gob.es/static_files/Sede/Actualidad/Novedades/2026/BOLINFORM%20_RDL_5_2026.pdf)).

- **Right:** general scale, reduction for work income, personal minimum and per
  descendant, and the social security contribution of 6.50% up to 5,101.20 € per month.
- **Deduction for those earning near the minimum wage:** it was missing. It is 590.89 € off the tax up to
  17,094 € gross per year, with a 20% phase-out of the excess up to 20,048.45 €. Without it, the 2026 minimum
  wage paid about 590 € of IRPF in the estimate; the law leaves it at zero.

### France

Sources: [2026 barème](https://www.service-public.gouv.fr/particuliers/vosdroits/F1419),
[BOFiP on the décote](https://bofip.impots.gouv.fr/bofip/2495-PGP.html/identifiant=BOI-IR-LIQ-20-20-30-20260407)
and the 2026 contribution tables.

- **Right:** the five brackets (11,600 €, 29,579 €, 84,577 € and 181,917 €) and the 10% deduction.
- **Décote:** it was missing. Those with gross tax below 1,982 € get 897 € minus 45.25% of the
  tax off; the estimate for those earning up to about 2,700 € net per month came out higher.
- **Cap of the 10% deduction:** 14,555 €, from 145,550 € of taxable salary.
- **Contributions above the Social Security ceiling:** they were at about 11%; the sum of the
  legal rates (Agirc-Arrco of 8.64% + 1.08% + 0.14%, old-age 0.40%, CSG and CRDS) gives 19.79%. Up to the
  ceiling, the sum is 20.84%, in place of "about 22%".
- **Non-deductible CSG and CRDS:** 2.90% on 98.25% of the gross go back into the tax base.

### Germany

Sources: [§ 32a EStG](https://www.gesetze-im-internet.de/estg/__32a.html),
[SolzG § 3](https://www.gesetze-im-internet.de/solzg_1995/__3.html) and § 10 EStG.

- **Right:** the 2026 tariff (Grundfreibetrag of 12,348 €), the flat allowances and the rates and ceilings of the
  Sozialversicherung.
- **Solidaritätszuschlag:** it was missing. It is 5.5% of the tax when it exceeds 20,350 € in the year,
  limited to 11.9% of the excess. It weighs from about 90 thousand € gross per year.
- **Contributions in the base:** unemployment insurance (1.3%) and 4% of health insurance are not deductible;
  the estimate deducted everything.

### United Kingdom

Sources: [rates and thresholds 2026 to 2027](https://www.gov.uk/guidance/rates-and-thresholds-for-employers-2026-to-2027)
and [income over £100,000](https://www.gov.uk/income-tax-rates/income-over-100000).

- **Right:** personal allowance, the three bands and National Insurance.
- **Allowance loss:** it was missing. Above £100 thousand it drops £1 for every £2, which gives an effective
  60% up to £125,140. At £150 thousand the estimate was £5,028 below the tax.

### Canada, Argentina and Chile

They were not rechecked in this round. They only enter the property test.

## Known-result cases

They live in `crates/treasury-core/tests/fixtures/ir-paises/casos-2026.json`, with the source of each one,
and are checked by `tax_burden/tests_countries.rs`. There are 30 cases of invented people: the year's
gross, dependents, filing status and the tax expected from the official tables and formulas.

- **United States (12):** from US$ 60 thousand to US$ 420 thousand, with and without children, in the three tables
  (single, head of household and joint), including the credit reduced to US$ 3,400 with
  US$ 220 thousand and two children.
- **Portugal (9):** from the minimum wage (zero) to 3 500 € per month, with the Government simulation of
  2026-09-17 for 1 000 € per month (491.90 €), one case in each paragraph of art. 70.º and a
  single-parent family.
- **Spain (2), France (2), Germany (2) and United Kingdom (3):** a low and a high income in
  each country, computed by hand from the law.

The tolerance covers rounding per payment (up to 0.60 per year) and, in Germany, the
approximate tariff in linear segments (up to 15 € per year in the progressive zone).

The same test file checks three properties in all adapted countries: the tax never
falls when gross rises, net never falls and the tax has no jumps.

For the payroll in the whole app, `income/tests_country.rs` covers the United States (the three filing
statuses, by changing the family registry), Portugal, France, Germany and the United Kingdom.

Rights: laws, tables and official acts have no copyright protection in the checked countries, and the
IRS and SSA material is public domain (17 USC 105); the numbers in official simulations are
facts, used with citation.

## Annual checklist

- **United States:** the IRS Rev. Proc. in October or November (brackets, deductions, child
  credit) and the Social Security ceiling from the SSA in October.
- **Portugal:** the State Budget in January (art. 68.º, reference value of art. 70.º,
  IAS) and mid-year changes, like the one in 2026.
- **Spain:** the minimum wage and the deduction tied to it, the contribution order.
- **France:** the finance law (barème, décote, cap of the 10% deduction), PASS and the Agirc-Arrco
  rates in January.
- **Germany:** § 32a EStG, the Solidaritätszuschlag Freigrenze, Rechengrößen and Zusatzbeitrag.
- **United Kingdom:** rates and thresholds in April (the tax year starts on 6 April).
- When a number changes, update the corresponding case in the cases file with the new source.

## Limitations

- **The estimate is the year's tax divided by the payments**, not the withholding of
  each month. In Portugal the month's net follows the withholding tables (and the November 2026
  withholding was cut in half); in Spain and France, the withholding algorithm and rate.
- **Joint filing** only exists in the United States. When both spouses have income, each
  one follows the single table, as in the payroll; the joint return's tax may come out a
  little lower when incomes are very different.
- **The US filing status is deduced from the registry** (spouse among the members, dependents on the
  income, spouse's income in the app). For head of household the law also requires paying more than half
  of the household expenses (Publication 501), which the app presumes for whoever has the income. Someone
  who is married and did not register the spouse comes out as single or head of household.
- **United States:** state and local tax, the refundable part of the child credit and the
  EITC (the estimated tax does not go negative), 401(k) and health plan before tax. The Social Security ceiling
  and the Additional Medicare threshold are applied per month, and the law applies them
  on the year's cumulative: it only matters with irregular pay or bonuses.
- **Portugal:** the supplement per dependent up to 6 years (art. 78.º-A), IRS Jovem, Azores and Madeira.
  The tax is computed through the portion to deduct, and the official table uses the average rate of the
  previous bracket: the difference is cents per year.
- **Spain:** the scales of each autonomous community, foral regimes, higher minimum per descendant
  from the second child.
- **France:** family quotient (dependents do not change the estimate), mutuelle and
  employer pension plan, contribution on high incomes.
- **Germany:** Kirchensteuer, the Pflegeversicherung surcharge for those without children,
  tax classes other than I.
- **United Kingdom:** Scotland, pension before tax, the April-to-April tax year (the estimate
  uses the 2026/27 rules for the calendar year).
