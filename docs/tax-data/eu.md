# European Union: tax reference data (version 2026-10)

Reference data for the in-app tax estimate (Conservador / Provável / Elevado). Rates are in percent. Every figure has a source id; see the source table at the end. The data was compiled on 2026-10-17.

## 1. EU VAT framework (`ec_taxud_vat_rates`)

Standard rate must be at least 15% with no maximum (Art. 97 VAT Directive). Since the 2022 reform (Directive (EU) 2022/542) Member States may apply up to two reduced rates of at least 5% to the 24 categories of Annex III, one super-reduced rate below 5% and one zero rate to at most seven categories covering basic needs (e.g., foodstuffs, medicines, pharmaceutical products); some derogations allow parking rates of at least 12%.

## 2. VAT rates by Member State (`youreurope_vat`, last checked 2026-07-13)

The table excludes region-specific rates such as the Azores, Madeira and the Canary Islands. A 0 in the reduced column marks a zero rate listed by Your Europe.

- **AT**
  - **Country:** Austria
  - **Standard:** 20
  - **Reduced:** 10 / 13
  - **Parking:** 13
  - **Recent changes (verified):** 2026-07-01: New 4.9% rate on selected basic foods listed by CN code in Annex 3 UStG (restaurant/catering excluded; other food stays 10%). Not yet shown in the Your Europe table checked 2026-07-13.
- **BE**
  - **Country:** Belgium
  - **Standard:** 21
  - **Reduced:** 6 / 12
  - **Parking:** 12
- **BG**
  - **Country:** Bulgaria
  - **Standard:** 20
  - **Reduced:** 9
- **CY**
  - **Country:** Cyprus
  - **Standard:** 19
  - **Reduced:** 5 / 9
  - **Super-reduced:** 3
  - **Recent changes (verified):** until 2025-12-31: Temporary zero rate on baby milk, diapers, feminine hygiene, fresh fruit and vegetables; status in 2026 not verified.
- **CZ**
  - **Country:** Czechia
  - **Standard:** 21
  - **Reduced:** 12 / 0
- **DE**
  - **Country:** Germany
  - **Standard:** 19
  - **Reduced:** 7
  - **Recent changes (verified):** 2026-01-01: Restaurant and catering food permanently at 7% (was 19%); beverages remain 19%.
- **DK**
  - **Country:** Denmark
  - **Standard:** 25
  - **Reduced:** 0
- **EE**
  - **Country:** Estonia
  - **Standard:** 24
  - **Reduced:** 9
  - **Recent changes (verified):** 2025-01-01: Accommodation (incl. with breakfast) 9% -> 13%; press publications 5% -> 9%.; 2025-07-01: Standard rate 22% -> 24%.
- **EL**
  - **Country:** Greece
  - **Standard:** 24
  - **Reduced:** 6 / 13 / 17
  - **Super-reduced:** 4
- **ES**
  - **Country:** Spain
  - **Standard:** 21
  - **Reduced:** 10
  - **Super-reduced:** 4
  - **Recent changes (verified):** 2025-01-01: Temporary food VAT cuts ended: basic foods 2% -> 4%; olive oil 2% -> 4%; seed oils and pasta 7.5% -> 10%.; 2026-03-22: Electricity (<10 kW), natural gas, biomass and motor fuels: 21% from 2026-01-01 to 2026-03-21, temporary 10% from 2026-03-22 to 2026-06-30 (conditional on April CPI). Status after 2026-06-30 not verified.
- **FI**
  - **Country:** Finland
  - **Standard:** 25.5
  - **Reduced:** 10 / 13.5
  - **Recent changes (verified):** 2024-09-01: Standard rate 24% -> 25.5%.; 2025-01-01: Books, movie tickets moved from 10% to 14%.; 2026-01-01: 14% reduced rate cut to 13.5% (books, passenger transport, accommodation, cultural events, gyms).
- **FR**
  - **Country:** France
  - **Standard:** 20
  - **Reduced:** 5.5 / 10
  - **Super-reduced:** 2.1
- **HR**
  - **Country:** Croatia
  - **Standard:** 25
  - **Reduced:** 5 / 13
- **HU**
  - **Country:** Hungary
  - **Standard:** 27
  - **Reduced:** 5 / 18
- **IE**
  - **Country:** Ireland
  - **Standard:** 23
  - **Reduced:** 9 / 13.5
- **IT**
  - **Country:** Italy
  - **Standard:** 22
  - **Reduced:** 5 / 10
  - **Super-reduced:** 4
- **LT**
  - **Country:** Lithuania
  - **Standard:** 21
  - **Reduced:** 5 / 12
- **LU**
  - **Country:** Luxembourg
  - **Standard:** 17
  - **Reduced:** 8
  - **Super-reduced:** 3
  - **Parking:** 14
- **LV**
  - **Country:** Latvia
  - **Standard:** 21
  - **Reduced:** 5 / 12
- **MT**
  - **Country:** Malta
  - **Standard:** 18
  - **Reduced:** 5 / 7
  - **Parking:** 12
- **NL**
  - **Country:** Netherlands
  - **Standard:** 21
  - **Reduced:** 9
  - **Recent changes (verified):** 2026-01-01: Accommodation (logies) VAT raised to 21% (previous rate not stated on the page read).
- **PL**
  - **Country:** Poland
  - **Standard:** 23
  - **Reduced:** 5 / 8
- **PT**
  - **Country:** Portugal
  - **Standard:** 23
  - **Reduced:** 6 / 13
  - **Parking:** 13
- **RO**
  - **Country:** Romania
  - **Standard:** 21
  - **Reduced:** 11
  - **Recent changes (verified):** 2025-08-01: Standard rate 19% -> 21%; 5% and 9% reduced rates merged into a single 11% rate (Law 141/2025).
- **SE**
  - **Country:** Sweden
  - **Standard:** 25
  - **Reduced:** 6 / 12
- **SI**
  - **Country:** Slovenia
  - **Standard:** 22
  - **Reduced:** 5 / 9.5
- **SK**
  - **Country:** Slovakia
  - **Standard:** 23
  - **Reduced:** 5 / 19
  - **Recent changes (verified):** 2025-01-01: Standard 20% -> 23%; reduced 10% -> 19% (non-basic food, electricity); 5% for basic food, medicines, books.

## 3. Which goods get reduced rates

Across Member States, reduced or super-reduced rates are most commonly applied to foodstuffs (often with a lower rate for basic staples), medicines and pharmaceutical products, books and newspapers, water supply, passenger transport, accommodation and, in several countries, restaurant food. Alcohol, sugary soft drinks and most other goods are taxed at the standard rate. Energy treatment varies by country and has changed often since 2022 (e.g., Spain's temporary 2026 cut; Portugal's 6% on electricity consumption within limits; Slovakia 19% on electricity; Italy 10% on domestic electricity and gas). See consumption[].reduced_rate_examples for verified country examples (DE, FR, ES, IT, PT, NL, AT, RO, SK, FI).

### Country examples (verified)

- **Austria (AT)**: basic foods annex3: 4.9%; other food: 10%; restaurant catering: not 4.9 (rate not verified). Sources: `usp_at_49`.
- **Germany (DE)**: food: 7%; restaurant food: 7%; restaurant beverages: 19%; books newspapers: 7%; local passenger transport: 7%; accommodation: 7%; coffee tea: 7%; packaged drinking water and juices: 19%. Sources: `de_ustg_12`, `de_breg_gastro_2026`.
- **Spain (ES)**: basic food bread milk eggs fruit veg olive oil: 4%; other food: 10%; sugary soft drinks: 21%; alcohol: 21%; restaurants hotels: 10%; passenger transport: 10%; books newspapers: 4%; medicines human: 4%; water: 10%; electricity gas fuels: 21 (2026-01-01..03-21); 10 temporary (2026-03-22..06-30). Sources: `es_aeat_2026`.
- **Finland (FI)**: books: 13.5%; passenger transport: 13.5%; accommodation: 13.5%; cultural events: 13.5%. Note: Food, restaurants and medicines rates not shown on the Vero page read (see unverified). Sources: `vero_fi_changes`.
- **France (FR)**: food shops: 5.5%; restaurant and immediate consumption: 10%; alcohol: 20%; medicines reimbursed: 2.1%; medicines not reimbursed: 10%; feminine hygiene: 5.5%. Sources: `fr_sp_food`, `fr_sp_health`.
- **Italy (IT)**: food beverages agricultural basic: 4%; certain foods: 5%; domestic electricity gas: 10%; medicines: 10%. Note: Agenzia Entrate summary (page dated 2020-12-12); full list in Tabella A, DPR 633/1972. Sources: `it_ade_aliquote`.
- **Netherlands (NL)**: food: 9%; water: 9%; medicines: 9%; books periodicals: 9%; accommodation: 21%. Sources: `nl_bd_9`.
- **Portugal (PT)**: basic food bread milk fruit veg: 6%; water: 6%; medicines: 6%; books newspapers: 6%; electricity consumption within limits: 6%; passenger transport: 6%; hotels: 6%; restaurants food services: 13%; ordinary wine: 13%; edible oils: 13%. Note: Mainland rates; Azores and Madeira set lower regional rates (not verified). Sources: `pt_at_civa`.
- **Romania (RO)**: food and beverages general: 11%; medicines human: 11%; books newspapers: 11%; water supply: 11%; restaurants excl alcohol: 11%; hotels: 11%; alcohol juices nonalcoholic beer foods 10g added sugar per 100g: 21%; veterinary medicines: 21%. Sources: `ey_romania_2025`.
- **Slovakia (SK)**: basic food: 5%; medicines: 5%; books: 5%; other food: 19%; electricity: 19%. Sources: `ec_a2m_vat_changes_2025`.

## 4. Labour taxes: OECD Taxing Wages (`oecd_taxing_wages_db`)

Household: Single person, no children, earning 100% of the national average wage (AW). Edition: OECD Taxing Wages 2026 (data years 2024 and 2025); the 2025 edition covered 2024.

- **Tax wedge** is income tax plus employee and employer social security contributions, minus cash benefits, as a share of total labour cost.
- **Payslip rate** is income tax plus employee SSC as a share of the gross wage.

### Focus countries

| Country | Tax wedge 2024 | Tax wedge 2025 | Payslip rate 2025 | Income tax 2025 | Employee SSC 2025 | Employer SSC 2025 | Avg wage 2025 |
|---|---|---|---|---|---|---|---|
| Germany | 47.93 | 49.26 | 38.68 | 17.22 | 21.46 | 20.86 | EUR 66,700 |
| Spain | 41.14 | 41.44 | 23.54 | 17.06 | 6.48 | 30.57 | EUR 32,678 |
| France | 47.12 | 47.18 | 27.98 | 16.67 | 11.31 | 36.35 | EUR 45,964 |
| Italy | 46.96 | 45.76 | 28.63 | 19.14 | 9.49 | 31.58 | EUR 36,594 |
| Netherlands | 35.9 | 35.94 | 27.86 | 17.85 | 10.02 | 12.6 | EUR 69,028 |
| Portugal | 39.58 | 39.34 | 24.93 | 13.93 | 11.0 | 23.75 | EUR 24,254 |
| **OECD average** | 34.93 | 35.08 | 25.11 | 15.49 | 9.61 | 16.61 |  |
| **EU-22 (OECD members)** | 41.64 | 41.73 | 29.24 | 17.19 | 12.04 | 21.85 |  |

### All EU members of the OECD (2025)

| Country | Tax wedge | Payslip rate | Income tax | Employee SSC | Employer SSC |
|---|---|---|---|---|---|
| Austria | 47.09 | 32.48 | 14.55 | 17.93 | 27.61 |
| Belgium | 52.48 | 39.55 | 25.55 | 13.99 | 27.22 |
| Czechia | 41.2 | 21.33 | 9.73 | 11.6 | 33.8 |
| Germany | 49.26 | 38.68 | 17.22 | 21.46 | 20.86 |
| Denmark | 35.76 | 35.33 | 35.33 | 0.0 | 0.67 |
| Spain | 41.44 | 23.54 | 17.06 | 6.48 | 30.57 |
| Estonia | 42.64 | 23.25 | 21.65 | 1.6 | 33.8 |
| Finland | 42.48 | 30.71 | 21.18 | 9.53 | 20.46 |
| France | 47.18 | 27.98 | 16.67 | 11.31 | 36.35 |
| Greece | 39.34 | 26.12 | 12.75 | 13.37 | 21.79 |
| Hungary | 41.15 | 33.5 | 15.0 | 18.5 | 13.0 |
| Ireland | 32.63 | 25.1 | 21.0 | 4.1 | 11.18 |
| Italy | 45.76 | 28.63 | 19.14 | 9.49 | 31.58 |
| Lithuania | 39.79 | 38.72 | 19.22 | 19.5 | 1.79 |
| Luxembourg | 40.16 | 31.99 | 19.69 | 12.31 | 13.65 |
| Latvia | 40.14 | 26.0 | 15.5 | 10.5 | 23.61 |
| Netherlands | 35.94 | 27.86 | 17.85 | 10.02 | 12.6 |
| Poland | 35.03 | 24.44 | 6.61 | 17.83 | 16.3 |
| Portugal | 39.34 | 24.93 | 13.93 | 11.0 | 23.75 |
| Slovak Republic | 42.72 | 24.27 | 10.87 | 13.4 | 32.2 |
| Slovenia | 45.3 | 36.21 | 12.15 | 24.07 | 16.6 |
| Sweden | 41.14 | 22.64 | 15.64 | 7.0 | 31.42 |

Values rounded to 2 decimals from the OECD database. Bulgaria, Croatia, Cyprus, Malta and Romania are not included in this OECD dataset. Search-result excerpt of the Taxing Wages 2026 executive summary reports an OECD-average wedge of 35.1% in 2025, consistent with 35.08 here.

## 5. Suggested mapping to scenarios (methodology, not a sourced figure)

VAT paid on a basket = sum(spend_i * r_i / (1 + r_i)) since EU shelf prices include VAT. Conservador: apply the lowest applicable reduced/super-reduced rate to food and essentials and the standard rate only to clearly standard-rated categories; Provável: map each spending category to its typical rate (examples per country); Elevado: standard rate to everything except categories verified as reduced. For labour taxes, the OECD 'income_tax_plus_employee_ssc_pct_gross_wage' at the average wage is a reasonable 'Provável' benchmark for a single average earner; the tax wedge (which adds employer SSC) is an upper-bound view of the total labour tax burden.

## 6. Status of the official Commission table

The DG TAXUD 'VAT rates applied in the Member States' PDF downloadable from the Commission is still the 2020 edition; the Commission now refers users to the Taxes in Europe Database (TEDB) and the Your Europe table, which is used here (last checked 2026-07-13).

## Unverified / not included

- **Taxes in Europe Database (TEDB) detail per Member State**: TEDB is a dynamic web application and was not queried; the Your Europe summary table (checked 2026-07-13) is used instead. Region-specific rates (Azores, Madeira, Canary Islands, etc.) are excluded.
- **Estonia 9% rate scope beyond press publications**: EMTA pages read confirm 13% for accommodation and 9% for press publications; other goods at 9% (e.g., books, medicines) were not read. Your Europe lists only 9% as reduced, omitting 13%.
- **Finland: rates for food, restaurants and medicines in 2026; scope of remaining 10% rate**: The Vero page read covers books, transport, accommodation, culture and gyms only. A search excerpt (not a read page) says food and pharmaceuticals are in the 13.5% rate.
- **Romania transitional rules**: Search excerpts mention housing contracts that may keep 9% until 2026-07-31; not read from a source. Romanian figures come from an EY alert (secondary but widely recognized), not from ANAF/Official Monitor directly.
- **Spain energy VAT after 2026-06-30**: AEAT 2026 table shows 10% only until 2026-06-30 (conditional); the rate applying from 2026-07-01 was not verified.
- **France energy VAT (electricity/gas subscription 5.5% abolished from 2025-08-01)**: Seen only in a search excerpt; not read from an official page.
- **Italy item-level rates (e.g., bread and milk at 4%)**: Agenzia Entrate summary page is dated 2020-12-12 and lists categories only; Tabella A of DPR 633/1972 not read.
- **Portugal regional rates (Azores 4/9/16, Madeira 4/12/22)**: Seen only in search excerpts of the Portal das Finanças tax leaflet; not read.
- **Germany medicines VAT rate**: Not stated in the § 12 UStG / Anlage 2 extracts read; not included.
- **Austria restaurant/catering VAT rate**: USP page only says catering is outside the 4.9% rate; the applicable rate (10% for food in restaurants is commonly cited) was not verified.
- **Cyprus temporary zero rate status in 2026**: Access2Markets news (2025-01-15) says it applied until 2025-12-31; any extension not checked.
- **Other 2025/2026 VAT changes not listed (e.g., Lithuania, Czechia, Croatia, Latvia, Ireland)**: Only the changes listed under recent_changes were verified; the Your Europe table reflects current rates but not change dates.
- **OECD Taxing Wages 2026 report, country notes and press release pages**: oecd.org returned HTTP 403; all wedge values were read directly from the OECD SDMX database. Publication month (April 2026) inferred from OECD URL paths in search results. Whether the latest year (2025) average wages are OECD estimates was not checked.

## Sources

- ``youreurope_vat``
  - **Publisher:** European Union - Your Europe (European Commission)
  - **Title:** [VAT rules and rates: standard, special & reduced rates - 'List of VAT rates applied in EU member countries'](https://europa.eu/youreurope/business/taxation/vat/vat-rules-rates/index_en.htm)
  - **Published:** page last checked 2026-07-13
  - **Supports:** Standard, reduced, super-reduced and parking VAT rates for all 27 Member States (excludes region-specific rates).
- ``ec_taxud_vat_rates``
  - **Publisher:** European Commission - DG Taxation and Customs Union (DG TAXUD)
  - **Title:** [VAT Rates](https://taxation-customs.ec.europa.eu/taxation/vat/vat-directive/vat-rates_en)
  - **Published:** page modified 2025-11-13
  - **Supports:** EU rules: standard rate at least 15% (Art. 97); up to two reduced rates >= 5% for Annex III categories; one super-reduced rate < 5% and one zero rate for up to seven basic-needs categories (e.g., foodstuffs, medicines); parking rate >= 12%; points to TEDB for country rates.
- ``ec_taxud_vat_rates_pdf_2020``
  - **Publisher:** European Commission - DG TAXUD
  - **Title:** [VAT rates applied in the Member States of the European Union - Situation at 1st January 2020 (taxud.c.1(2020))](https://taxation-customs.ec.europa.eu/document/download/82a38bdb-d724-472d-8e02-325b271e0d88_en?filename=vat_rates_en.pdf)
  - **Published:** 2020 (situation at 2020-01-01)
  - **Supports:** Historical only: the downloadable DG TAXUD PDF is still the 2020 edition; the Commission now points to TEDB / Your Europe. Not used for current figures.
- ``ec_a2m_vat_changes_2025``
  - **Publisher:** European Commission - Access2Markets
  - **Title:** [Recent VAT changes in certain EU Member States](https://trade.ec.europa.eu/access-to-markets/en/news/recent-vat-changes-certain-eu-member-states)
  - **Published:** 2025-01-15
  - **Supports:** Slovakia 20%->23% standard, 10%->19% reduced (other food, electricity), 5% super-reduced (basic food, medicines, books) from 2025-01-01; Spain basic foods back to 4% and seed oils/pasta to 10% from 2025-01-01; Cyprus temporary zero rate until 2025-12-31.
- ``vero_fi_changes``
  - **Publisher:** Finnish Tax Administration (Vero Skatt)
  - **Title:** [The changes to VAT rates](https://www.vero.fi/en/businesses-and-corporations/taxes-and-charges/vat/rates-of-vat/the-changes-to-VAT-rates/)
  - **Published:** page updated 2026-01-01
  - **Supports:** Finland standard rate 24%->25.5% from 2024-09-01; 14% reduced rate cut to 13.5% from 2026-01-01; books moved 10%->14% (2025)->13.5% (2026); passenger transport, accommodation, cultural events at 13.5% from 2026.
- ``emta_ee_standard``
  - **Publisher:** Estonian Tax and Customs Board (EMTA)
  - **Title:** [Standard VAT rate](https://emta.ee/en/business-client/taxes-and-payment/value-added-tax/vat-rates-and-supply-exempt-tax/standard-vat-rate)
  - **Published:** n/d
  - **Supports:** Estonia standard VAT 24% from 2025-07-01; VAT Act rates 24%, 13%, 9%, 0%.
- ``emta_ee_vat``
  - **Publisher:** Estonian Tax and Customs Board (EMTA)
  - **Title:** [Value added tax](https://www.emta.ee/en/business-client/taxes-and-payment/value-added-tax)
  - **Published:** n/d
  - **Supports:** Estonia standard VAT 22% -> 24% from 2025-07-01; accommodation (incl. with breakfast) 9% -> 13% from 2025-01-01; press publications 5% -> 9% from 2025-01-01.
- ``ey_romania_2025``
  - **Publisher:** EY (Ernst & Young) Global Tax Alert
  - **Title:** [Romanian tax changes introduced by new fiscal and budgetary measures](https://www.ey.com/en_gl/technical/tax-alerts/romanian-tax-changes-introduced-by-new-fiscal-and-budgetary-measures)
  - **Published:** 2025-07-30
  - **Supports:** Romania standard VAT 19%->21% from 2025-08-01; 5% and 9% merged into single 11% reduced rate (food, medicines, books, water, restaurants, hotels, etc.) under Law 141/2025 (Official Monitor 2025-07-25).
- ``usp_at_49``
  - **Publisher:** Austria - Unternehmensserviceportal (USP), Federal Ministry of Finance
  - **Title:** [Neuer Umsatzsteuersatz von 4,9 Prozent](https://www.usp.gv.at/aktuelles/newsliste/neuer_ustsatz.html)
  - **Published:** last updated 2026-07-10
  - **Supports:** Austria: new 4.9% VAT rate on selected basic foods (Annex 3 UStG, by CN code) from 2026-07-01; restaurant/catering excluded; non-listed foods stay at 10%.
- ``de_ustg_12``
  - **Publisher:** Germany - Federal Ministry of Justice (gesetze-im-internet.de)
  - **Title:** [Umsatzsteuergesetz (UStG) § 12 Steuersätze, and Anlage 2](https://www.gesetze-im-internet.de/ustg_1980/__12.html)
  - **Published:** consolidated law (accessed online)
  - **Supports:** Germany 19% standard, 7% reduced for Anlage 2 goods (most foods, coffee/tea, books, newspapers), local passenger transport, short-term accommodation, restaurant/catering services excluding beverages; 0% on residential solar panels. Anlage 2: https://www.gesetze-im-internet.de/ustg_1980/anlage_2.html
- ``de_breg_gastro_2026``
  - **Publisher:** Bundesregierung (German Federal Government)
  - **Title:** [Entlastungen für Pendler und Gastronomie](https://www.bundesregierung.de/breg-de/aktuelles/steueraenderungsgesetz-bundesrat-2383684)
  - **Published:** 2026-01-01
  - **Supports:** Germany: VAT on restaurant food cut from 19% to 7% since 2026-01-01 (beverages excluded).
- ``fr_sp_food``
  - **Publisher:** Service-Public.fr (Entreprendre) - French administration
  - **Title:** [Taux de TVA sur les produits alimentaires et les boissons (F22399)](https://entreprendre.service-public.gouv.fr/vosdroits/F22399)
  - **Published:** verified 2025-12-03
  - **Supports:** France: 5.5% food for deferred consumption (shops/supermarkets); 10% food and soft drinks for immediate consumption (restaurants, takeaway sandwiches); 20% alcohol.
- ``fr_sp_health``
  - **Publisher:** Service-Public.fr (Entreprendre) - French administration
  - **Title:** [Taux de TVA dans le secteur de l'hygiène et de la santé (F23567)](https://entreprendre.service-public.gouv.fr/vosdroits/F23567)
  - **Published:** verified 2025-06-18
  - **Supports:** France: 2.1% medicines reimbursed by social security; 10% non-reimbursed medicines; 5.5% feminine hygiene products and condoms.
- ``es_aeat_2026``
  - **Publisher:** Agencia Tributaria (Spain)
  - **Title:** [Tipos impositivos en el IVA 2026](https://sede.agenciatributaria.gob.es/static_files/Sede/Tema/IVA/IVA_reperc/Tipos_IVA_2026_26_02_2026.pdf)
  - **Published:** 2026-02-26
  - **Supports:** Spain: 21% general; 10% food in general, water, passenger transport, restaurants/hotels; 4% bread, milk, cheese, eggs, fruit, vegetables, olive oil, books/newspapers, human medicines; electricity/gas/fuels 21% from 2026-01-01 to 2026-03-21 and 10% from 2026-03-22 to 2026-06-30 (temporary, conditional).
- ``it_ade_aliquote``
  - **Publisher:** Agenzia delle Entrate (Italy)
  - **Title:** [IVA - Norme generali e aliquote](https://www.agenziaentrate.gov.it/portale/iva-regole-generali-aliquote-esenzioni-pagamento/norme-generali-e-aliquote)
  - **Published:** page last updated 2020-12-12
  - **Supports:** Italy: 22% ordinary; 10% domestic electricity and gas, medicines, building renovation; 5% certain foods; 4% foods, beverages, agricultural products (full list in Tabella A, DPR 633/1972).
- ``pt_at_civa``
  - **Publisher:** Autoridade Tributária e Aduaneira - Portal das Finanças (Portugal)
  - **Title:** [Código do IVA, artigo 18.º (Taxas) and Listas I e II anexas](https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/civa_rep/Pages/c-iva-listas.aspx)
  - **Published:** consolidated code (accessed online)
  - **Supports:** Portugal (mainland): 6% List I (bread, milk, fruit/vegetables, water, medicines, books/newspapers, electricity consumption within limits, passenger transport, hotels); 13% List II (restaurant/food services excl. alcohol and soft drinks, ordinary wine, edible oils); 23% other. Art. 18: https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/civa_rep/Pages/iva18.aspx
- ``nl_bd_9``
  - **Publisher:** Belastingdienst (Netherlands Tax Administration)
  - **Title:** [Goederen met 9% btw; and news 'Vanaf 1 januari 2026: btw-tarief logies omhoog naar 21%' (2025-10-30)](https://www.belastingdienst.nl/wps/wcm/connect/bldcontentnl/belastingdienst/zakelijk/btw/tarieven_en_vrijstellingen/goederen_9_btw/goederen_met_9_btw)
  - **Published:** n/d; news item 2025-10-30
  - **Supports:** Netherlands: 9% for food, water, ornamental plants, medicines and aids, art/antiques, books and periodicals; accommodation moved to 21% from 2026-01-01.
- ``oecd_taxing_wages_db``
  - **Publisher:** OECD
  - **Title:** [Taxing Wages database - 'Labour taxation - OECD comparative country indicators' (dataflow OECD.CTP.TPS DSD_TAX_WAGES_COMP@DF_TW_COMP v2.1), read via the OECD SDMX API](https://sdmx.oecd.org/public/rest/data/OECD.CTP.TPS,DSD_TAX_WAGES_COMP@DF_TW_COMP,/all?startPeriod=2023&dimensionAtObservation=AllDimensions)
  - **Published:** data through 2025 (Taxing Wages 2026 edition, published April 2026)
  - **Supports:** Average tax wedge and components (income tax, employee and employer SSC) for a single worker, no children, at 100% of the average wage, 2024 and 2025, for 22 EU-OECD members, the EU-22 average and the OECD average.

All sources accessed 2026-10-17.
