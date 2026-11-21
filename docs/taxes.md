# Estimate of taxes paid

The **Taxes** screen and the `tax_burden.estimate` command (core in
`crates/treasury-core/src/tax_burden`, MCP tools `estimate_taxes_paid` and
`get_city_tax_revenue`).

The goal is to show, realistically and verifiably, how much of the year's financial life went to
taxes, from the data the app already has: spending by category, salary, recorded taxes, assets and
dividends. It is an **estimate**, not an official figure. Each number says where it
came from.

## Principles

- Official sources only: IBGE (National Accounts), Receita Federal, Previdência Social, Tesouro
  Nacional, Petrobras, laws on the Planalto site and, outside Brazil, the Tax Foundation and the European
  Commission. Each percentage points to its sources, with publication and consultation dates.
- Neutral, factual language. The screen does not compare the user's burden with value judgments; the
  country's tax burden appears only as a reference, with the source.
- Where there is no reliable data (rent, health insurance, ride-hailing apps, purchases with no
  detail), the spending shows on the screen as **no reliable data** and does not become estimated tax.
  We prefer to underestimate than to invent.
- The mandatory notice is always in view: *"This is an estimate based on average tax burdens per spending category and the income tax rules in force. Actual amounts may vary with the state, how businesses are taxed and specific tax breaks."*

## Scenarios

| Scenario | Use |
|---|---|
| **Conservative** | the lowest reasonable value: the product with the lowest burden in the group, or the reduced rate |
| **Likely** (default) | the central value: the median of the products in the group, or the most recent official figure |
| **High** | the highest reasonable value: the product with the highest burden in the group, or the standard rate |

The scenarios change only the consumption tax. Income tax, INSS, recorded taxes and IPVA
follow real rules or values and are the same in all three.

## What counts

1. **Consumption.** For each spending category of the year (accrual, in the household currency): spending
   × the fraction of the final price that is tax in the category's **group**. The groups are the same in
   all countries (`mapping.rs`); each country gives its percentages (`data/<country>.json`).
   - The default mapping comes from the names of the starter set's categories; a subcategory with no
     mapping inherits its parent's. The user can change the group of any category or set
     their own rate (valid in all three scenarios; `tax_category` table, V109).
   - **Brazil, state:** the consumption percentages are national (IBGE National Accounts) and
     do not change with the chosen state, which only affects IPVA.
   - **US and European Union (draft):** the state's average sales tax (Tax Foundation)
     or the country's VAT (European Commission), as a fraction of the amount paid: rate ÷ (100 +
     rate).
2. **Taxes paid directly**: what was recorded in the categories Income tax, IOF, IPTU,
   IPVA and licensing, and Taxes and fees counts at the recorded amount.
3. **Work income (Brazil)**: with the year's income statement (`tax_income_report`
   table), its values apply. With no statement, each month's gross is estimated
   from the net salary received in the categories Salary, Vacation pay, Overtime and Commissions
   (severance and PLR are left out), by the IRPF and INSS tables in force that month
   (`rules.rs`), with no other deductions; the 13th is taxed separately
   (Lei 8.134/1990, art. 16). Since January 2026, the reduction of Lei 15.270/2025 applies: up to
   R$ 312.89 for taxable income of up to R$ 5,000 and R$ 978.62 − 0.133145 × income between
   R$ 5,000.01 and R$ 7,350, capped at the tax.
4. **Assets**: IPTU and IPVA recorded count at the recorded amount. With no IPVA recorded in the year, it
   is estimated from each vehicle's value at the end of the period and the state's rate.
5. **IR withheld on dividends** (JCP, foreign dividends), from the portfolio records.
6. **Receipts (Brazil):** purchases with a receipt read and linked to a transaction use
   the receipt's tax (the approximate taxes printed, Lei 12.741/2012, or the amount estimated item by
   item by the consumption group's percentage) instead of the category average; the amount paid leaves the
   category and becomes the "Scanned receipts" line. See [receipts.md](receipts.md).

**Work days for taxes:** estimated tax ÷ gross income × days in the year, per
scenario (with no known income, it does not appear). Beside it, the country's reference: the
gross tax burden from the Tesouro Nacional (32.4% of GDP in 2025) equals 118 days of the year.
It is context, not a target: the reference divides taxes by GDP, and the family's figure divides what the app knows
about them by gross income.

## Brazil only: where the tax goes and the city's revenue

Brazil's tax data is offered only to Brazilian vaults: outside Brazil, the "Brazil" set
does not appear in the list of data countries, cannot be chosen and the estimate does not bring the
blocks below (`dataset::all_for`, `tax_burden::settings`).

**Where the tax goes.** The estimated total, in each scenario, is split in proportion to the
general government's spending (federal, state and municipal) by function, from the report "Despesa por
Função do Governo Geral" (general government spending by function) of the Tesouro Nacional and the Secretaria de Orçamento Federal (COFOG
classification, 2024 results): social protection 36.11%, general public services 24.20% (includes
debt interest), education 11.14%, health 10.99%, public order and safety 6.45%, economic affairs 4.94%,
housing and community services 2.97%, environmental protection 1.21%, defense
1.10% and recreation, culture and religion 0.88%. It is a proportional split, not the destination of each
real: the government is also financed by debt and other revenue. The rounding remainder goes
to the largest function, and the sum matches the total (`TaxEstimate.destination`).

**The city's revenue.** The person chooses the family's city (state and municipality, IBGE list;
`tax_burden.city` setting, with the IBGE code). On request, the app fetches from the Tesouro
Nacional (Siconfi) the municipality's most recent Relatório Resumido da Execução Orçamentária
(Anexo 03, up to six bimesters back) and stores in the vault the sum of the last 12 months of
taxes, fees and betterment contributions, by tax (IPTU, ISS, ITBI, income tax
withheld from civil servants and fees), the population reported in the report and the amount per inhabitant
(`tax_burden/city.rs`, `tax_burden.city_revenue` setting).

- The read (`tax_burden.city`) only returns what is stored; choosing the city
  (`set_city`) and the collection (`refresh_city`) are interface only, outside the history.
- The requests carry the municipality code to the Tesouro and the state to IBGE, and nothing about the family.
- Sources with an on/off switch in Settings › Data sources: "Tesouro Nacional (Siconfi)" for
  the revenue and "IBGE" for the city list. With the Tesouro one off, the card warns and
  does not fetch.
- It is what the city government collected, not what each resident pays: companies pay a good part of ISS
  and IPTU, and federal and state taxes are left out.

The Impostômetro (IBPT/ACSP) is neither a source nor an integration: the site's terms of use do not
allow reusing the numbers, and the per-municipality revenue it shows comes from the same
official sources used here.

## What does not count (and why)

- Charges paid by the employer (employer INSS, FGTS): they do not come out of the user's salary.
- Taxes embedded in services with no reliable data per category: rent, condo fees,
  domestic services, health insurance and appointments, ride-hailing apps, parking,
  glasses, school supplies, leisure, gifts and purchases with no detail.
- The 2026 tax reform test (CBS 0.9% and IBS 0.1%), offsettable and with no
  expected effect on prices (LC 214/2025).
- Values prior to LC 192 and 194/2022 for electricity, telecommunications and cooking gas
  (which capped the ICMS on those items).

## Main sources (Brazil, version 2026-11)

- **Consumption**: IBGE National Accounts, Supply and Use Tables of 2021 (the last year with
  128 products). Each product's percentage is the total of taxes on it (import
  tax, IPI, ICMS and other taxes minus subsidies) divided by supply at consumer price;
  a group with several products uses the average weighted by household consumption.
  The number counts only the taxes charged on the product (not those embedded in the chain, nor
  payroll charges), so it is a floor, and it is the same in the three scenarios. Gasoline, diesel and cooking
  gas: Petrobras price composition (week of 2026-10-04 to 2026-10-10). Bank
  fees: the legal rates of PIS, Cofins and ISS. Insurance: IOF of Decreto 6.306/2007. Seven
  groups are left with no official data (health insurance, domestic services, vehicle gas,
  motorcycles and purchases with no detail) and out of the estimate. The Impostômetro (IBPT) product list,
  the studies by sector associations and the adjustment by each state's ICMS
  are no longer used.
- **IRPF**: Leis 14.848/2024, 15.191/2025 and 15.270/2025; Receita Federal tables.
- **INSS**: Portarias Interministeriais MPS/MF 6/2025 and 13/2026.
- **Context**: 2025 gross tax burden (Tesouro Nacional, preliminary estimate) and
  general government spending by function for 2024 (Tesouro Nacional and SOF, COFOG).
- **States**: IPVA by state (state laws and SEFAZ), see `data/br.json`. The state does not change the
  consumption percentages, which are national.

The full list, with links and dates, is in `crates/treasury-core/src/tax_burden/data/` and
shows on the screen under "See sources and methodology". The survey that originated the numbers (each
product, each source and what was not verified) is in `docs/tax-data/`.

## Data updates

The data is versioned (`version` and `updated` in each `data/<country>.json`) and each estimate
shows the version used. After 6 months from the review (`dataset::STALE_MONTHS`), the screen marks the
data as outdated.

Review **at least every six months** and whenever any of these changes:

1. The IRPF table or the minimum wage (January), the INSS table (January ordinance).
2. Petrobras price composition (fuels and LPG change every week; the scenarios use
   a reference week, and temporary cuts in federal taxes change the Conservative one).
3. IBGE National Accounts: when a new year of the Supply and Use Tables with
   128 products comes out, update `docs/tax-data/research/br_ibge_tru_<year>.json` and run
   `python3 scripts/tax-data/br_consumo_ibge.py`. The gross tax burden from the Tesouro (work
   days) comes out every year, and spending by function (COFOG) in December
   (`docs/tax-data/research/br_cofog_<year>.json`).
4. The states' IPVA rates (state laws, usually in force in January or after 90
   days).
5. IOF (decrees), sales tax tables (Tax Foundation, semiannual) and European Union
   VAT.

Process:

1. Update the survey in `docs/tax-data/` (each number with source, date and what was not
   verified): `research/br.json` (consumption, IRPF, INSS, context), `research/br_icms.json`,
   `research/br_ipva.json`, `research/us.json` and `research/eu.json`; the `.md` files beside them explain
   each number.
2. Regenerate the app's data:

   ```bash
   python3 -I scripts/tax-data/build_br_dataset.py docs/tax-data/research crates/treasury-core/src/tax_burden/data/br.json
   python3 -I scripts/tax-data/build_us_eu.py docs/tax-data/research crates/treasury-core/src/tax_burden/data
   ```

3. Run `cargo test -p treasury-core tax_burden` (every rate needs a known source and
   must be between 0 and 100%), bump `version` and `updated` and record the change in the history below.

Notes from the 2026-10 survey: Alagoas enters with 21.5% (20.5% + 1% of the poverty
fund, which applies to almost everything according to a SEFAZ-AL query of July 2026; one of the 2026
tables shows only 20.5%). Roraima is left with no estimated IPVA (no official source checked).

## History

- **2026-11-18**: the consumption percentages now come from the IBGE National Accounts (2021),
  and the work-days reference from the Tesouro's gross tax burden. The Impostômetro (IBPT) product
  list, embedded since October, the sector studies and the adjustment
  by the state's ICMS were removed. The percentages dropped (groceries from 28.45% to 7.42%, clothing from 35.66%
  to 14.70%), because IBGE counts only the taxes on the product. "Where the tax goes" and the
  city's revenue came in, and Brazil's data now appears only in a Brazilian vault.

- **2026-10-19**: the IBPT product list was checked again at the source (the values are the
  same as on 2026-10-17; an automatic read from another analysis had brought different numbers,
  which are not on the page). Groups that had no data gained a range from the list: school
  supplies (26.42% · 39.25% · 51.70%), glasses (38.49% · 41.20% · 43.91%), leisure with tickets
  (27.10% · 31.44% · 35.78%) and the new cleaning supplies group (34.58% · 38.01% · 41.44%).
  The whole list is now embedded (`data/br_products.json`) for the receipts.
  Work-days reference (IBPT 2026) and receipts in the estimate.
- **2026-10 (0.6.0)**: first version. Brazil complete (consumption, IRPF 2025/2026 with the
  reduction of Lei 15.270, INSS 2025/2026, ICMS and IPVA by state); US and European Union as a draft
  (sales tax and VAT only).
