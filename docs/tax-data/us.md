# United States: tax reference data (version 2026-10)

Reference data for the in-app tax estimate (Conservador / Provável / Elevado). Rates are in percent, money in USD. Every figure has a source id; see the source table at the end. The data was compiled on 2026-10-17.

## 1. Federal income tax

### Tax year 2025

Brackets source: `irs_ir_2024_273`, `irs_brackets_page`. Standard deduction source: `irs_ir_2025_103`, `irs_obbb_individuals`.

| Rate | Single: taxable income over | up to | MFJ: taxable income over | up to |
|---|---|---|---|---|
| 10% | $0 | $11,925 | $0 | $23,850 |
| 12% | $11,925 | $48,475 | $23,850 | $96,950 |
| 22% | $48,475 | $103,350 | $96,950 | $206,700 |
| 24% | $103,350 | $197,300 | $206,700 | $394,600 |
| 32% | $197,300 | $250,525 | $394,600 | $501,050 |
| 35% | $250,525 | $626,350 | $501,050 | $751,600 |
| 37% | $626,350 | - | $751,600 | - |

Standard deduction: single $15,750, married filing jointly $31,500, head of household $23,625.
Before the One Big Beautiful Bill Act (P.L. 119-21, signed 2025-07-04), Rev. Proc. 2024-40 had set the 2025 amounts at $15,000 / $30,000 / $22,500 (`irs_ir_2024_273`).

### Tax year 2026

Brackets source: `irs_ir_2025_103`. Standard deduction source: `irs_ir_2025_103`.

| Rate | Single: taxable income over | up to | MFJ: taxable income over | up to |
|---|---|---|---|---|
| 10% | $0 | $12,400 | $0 | $24,800 |
| 12% | $12,400 | $50,400 | $24,800 | $100,800 |
| 22% | $50,400 | $105,700 | $100,800 | $211,400 |
| 24% | $105,700 | $201,775 | $211,400 | $403,550 |
| 32% | $201,775 | $256,225 | $403,550 | $512,450 |
| 35% | $256,225 | $640,600 | $512,450 | $768,700 |
| 37% | $640,600 | - | $768,700 | - |

Standard deduction: single $16,100, married filing jointly $32,200, head of household $24,150.

### OBBBA deductions for 2025-2028 (`irs_obbb_individuals`)

- Senior deduction: an extra $6,000 per person aged 65 or older. It phases out above a MAGI of $75,000 ($150,000 for MFJ).
- Tips deduction: up to $25,000 a year. It phases out above a MAGI of $150,000 ($300,000 for MFJ).
- Overtime deduction: up to $12,500 ($25,000 for MFJ). It uses the same phase-out thresholds as the tips deduction.

## 2. Payroll taxes (FICA)

| Item | Employee rate | Employer rate | Wage base 2025 | Wage base 2026 | Source |
|---|---|---|---|---|---|
| Social Security (OASDI) | 6.2% | 6.2% | $176,100 | $184,500 | `irs_pub15_2025`, `irs_pub15_2026`, `irs_tc751` |
| Medicare (HI) | 1.45% | 1.45% | no limit | no limit | same |
| Additional Medicare Tax | 0.9% above threshold | none | n/a | n/a | `irs_addl_medicare_qa`, `irs_tc751` |

Additional Medicare Tax thresholds: single, head of household and qualifying surviving spouse $200,000; married filing jointly $250,000; married filing separately $125,000. Employers withhold it on wages above $200,000 whatever the filing status.

The SSA fact sheets returned HTTP 403, so the wage bases come from IRS Publication 15 (2025 and 2026 editions).

## 3. State individual income tax: top marginal rates (as of 2026-01-01, `tf_income_2026`)

Local income taxes are excluded. Tax Foundation reports average local effective rates as a share of AGI (2023 data): AL 0.07, IN 0.35, IA 0.08, KY 0.93, MD 2.4, MI 0.16, MO 0.18, NY 1.60, OH 1.2, OR 0.18 and PA 0.99.

Selected large or notable states only; all 50 states plus DC are in `us.json` under `income.state.states`. None: AK, FL, NV, NH, SD, TN, TX, WY (WA taxes capital gains only).

- **Arizona**
  - **Type:** flat
  - **Top rate:** 2.5%
  - **Starts at (single):** $0
  - **Starts at (MFJ):** $0
- **California**
  - **Type:** graduated
  - **Top rate:** 13.3%
  - **Starts at (single):** $1,000,000
  - **Starts at (MFJ):** $1,485,906
  - **Note:** Top rate includes the 1% mental health services tax on taxable income over $1M. Rates exclude the SDI payroll tax (TF footnote says 1.3% with no wage ceiling; TF intro text says 1.1% - see unverified).
- **Colorado**
  - **Type:** flat
  - **Top rate:** 4.4%
  - **Starts at (single):** $0
  - **Starts at (MFJ):** $0
- **Florida**
  - **Type:** none
  - **Top rate:** 0%
  - **Note:** No individual income tax.
- **Georgia**
  - **Type:** flat
  - **Top rate:** 5.19%
  - **Starts at (single):** $0
  - **Starts at (MFJ):** $0
  - **Note:** Flat 5.19% as of 2026-01-01 per TF; later 2026 legislation not checked.
- **Hawaii**
  - **Type:** graduated
  - **Top rate:** 11%
  - **Starts at (single):** $325,000
  - **Starts at (MFJ):** $650,000
- **Illinois**
  - **Type:** flat
  - **Top rate:** 4.95%
  - **Starts at (single):** $0
  - **Starts at (MFJ):** $0
  - **Note:** Flat 4.95%.
- **Massachusetts**
  - **Type:** graduated
  - **Top rate:** 9%
  - **Starts at (single):** $1,083,150
  - **Starts at (MFJ):** $1,083,150
  - **Note:** 5% flat plus 4% surtax on income over $1,083,150 (shown as 9% top bracket). Excludes PFML payroll contribution (0.46% employee withholding).
- **Michigan**
  - **Type:** flat
  - **Top rate:** 4.25%
  - **Starts at (single):** $0
  - **Starts at (MFJ):** $0
- **Minnesota**
  - **Type:** graduated
  - **Top rate:** 9.85%
  - **Starts at (single):** $203,150
  - **Starts at (MFJ):** $337,930
- **Nevada**
  - **Type:** none
  - **Top rate:** 0%
  - **Note:** No individual income tax.
- **New Jersey**
  - **Type:** graduated
  - **Top rate:** 10.75%
  - **Starts at (single):** $1,000,000
  - **Starts at (MFJ):** $1,000,000
- **New York**
  - **Type:** graduated
  - **Top rate:** 10.9%
  - **Starts at (single):** $25,000,000
  - **Starts at (MFJ):** $25,000,000
  - **Note:** Local income taxes (e.g., New York City) excluded; NY has 'tax benefit recapture' for high incomes.
- **North Carolina**
  - **Type:** flat
  - **Top rate:** 3.99%
  - **Starts at (single):** $0
  - **Starts at (MFJ):** $0
- **Ohio**
  - **Type:** flat
  - **Top rate:** 2.75%
  - **Starts at (single):** $26,050
  - **Starts at (MFJ):** $26,050
  - **Note:** Single 2.75% rate on income over $26,050 (Ohio moved to a single rate in 2026); local income taxes excluded.
- **Oregon**
  - **Type:** graduated
  - **Top rate:** 9.9%
  - **Starts at (single):** $125,000
  - **Starts at (MFJ):** $250,000
- **Pennsylvania**
  - **Type:** flat
  - **Top rate:** 3.07%
  - **Starts at (single):** $0
  - **Starts at (MFJ):** $0
  - **Note:** Flat 3.07%; local earned income taxes excluded (TF: avg local effective rate 0.99% of AGI, 2023 data).
- **Tennessee**
  - **Type:** none
  - **Top rate:** 0%
  - **Note:** No individual income tax.
- **Texas**
  - **Type:** none
  - **Top rate:** 0%
  - **Note:** No individual income tax.
- **Virginia**
  - **Type:** graduated
  - **Top rate:** 5.75%
  - **Starts at (single):** $17,000
  - **Starts at (MFJ):** $17,000
- **Washington**
  - **Type:** capital_gains_only
  - **Top rate:** 9%
  - **Starts at (single):** $1,000,000
  - **Starts at (MFJ):** $1,000,000
  - **Note:** No tax on wage income; 7% / 9% tax on capital gains only (TF footnote tt). Excludes 0.58% WA Cares payroll tax.
- **Washington DC**
  - **Type:** graduated
  - **Top rate:** 10.75%
  - **Starts at (single):** $1,000,000
  - **Starts at (MFJ):** $1,000,000
  - **Note:** District of Columbia.

## 4. State and local sales tax (as of 2026-07-01, `tf_sales_2026_midyear`)

Edition: Tax Foundation, State and Local Sales Tax Rates, Midyear 2026 (published 2026-07-06; rates as of 2026-07-01). The population-weighted national average combined rate is 7.53%. 45 states levy a statewide sales tax, and 38 allow local sales taxes.

| State | State rate | Avg local | Combined | Max local | Rank | Groceries (state level) |
|---|---|---|---|---|---|---|
| Alabama (AL) | 4% | 5.46% | 9.46% | 8% | 5 | reduced (2%) |
| Alaska (AK) | 0% | 1.82% | 1.82% | 7.85% | 46 | no state sales tax |
| Arizona (AZ) | 5.6% | 2.94% | 8.54% | 5.3% | 11 | exempt |
| Arkansas (AR) | 6.5% | 2.98% | 9.48% | 6.13% | 4 | state-exempt, local may apply |
| California (CA) | 7.25% | 1.78% | 9.03% | 5.25% | 7 | exempt |
| Colorado (CO) | 2.9% | 4.99% | 7.89% | 9.1% | 16 | exempt |
| Connecticut (CT) | 6.35% | 0% | 6.35% | 0% | 33 | exempt |
| Delaware (DE) | 0% | 0% | 0% | 0% | 47 | no state sales tax |
| District of Columbia (DC) | 6% | 0% | 6% | 0% | 38 | exempt |
| Florida (FL) | 6% | 0.98% | 6.98% | 2% | 28 | exempt |
| Georgia (GA) | 4% | 3.56% | 7.56% | 5% | 18 | exempt |
| Hawaii (HI) | 4% | 0.5% | 4.5% | 0.5% | 45 | taxed at full rate (4%) |
| Idaho (ID) | 6% | 0.03% | 6.03% | 3% | 37 | taxed at full rate (6%) |
| Illinois (IL) | 6.25% | 2.73% | 8.98% | 4.75% | 8 | state-exempt, local may apply |
| Indiana (IN) | 7% | 0% | 7% | 0% | 25 | exempt |
| Iowa (IA) | 6% | 0.94% | 6.94% | 1% | 29 | exempt |
| Kansas (KS) | 6.5% | 2.21% | 8.71% | 4.25% | 9 | state-exempt, local may apply |
| Kentucky (KY) | 6% | 0% | 6% | 0% | 38 | exempt |
| Louisiana (LA) | 5% | 5.13% | 10.13% | 7% | 1 | exempt |
| Maine (ME) | 5.5% | 0% | 5.5% | 0% | 43 | exempt |
| Maryland (MD) | 6% | 0% | 6% | 0% | 38 | exempt |
| Massachusetts (MA) | 6.25% | 0% | 6.25% | 0% | 35 | exempt |
| Michigan (MI) | 6% | 0% | 6% | 0% | 38 | exempt |
| Minnesota (MN) | 6.88% | 1.26% | 8.14% | 3% | 15 | exempt |
| Mississippi (MS) | 7% | 0.06% | 7.06% | 1% | 24 | reduced (5%) |
| Missouri (MO) | 4.23% | 4.22% | 8.44% | 6.25% | 12 | reduced (1.225%) |
| Montana (MT) | 0% | 0% | 0% | 0% | 47 | no state sales tax |
| Nebraska (NE) | 5.5% | 1.48% | 6.98% | 2% | 27 | exempt |
| Nevada (NV) | 6.85% | 1.39% | 8.24% | 1.53% | 13 | exempt |
| New Hampshire (NH) | 0% | 0% | 0% | 0% | 47 | no state sales tax |
| New Jersey (NJ) | 6.63% | -0.02% | 6.6% | 3.31% | 30 | exempt |
| New Mexico (NM) | 4.88% | 2.8% | 7.68% | 4.56% | 17 | exempt |
| New York (NY) | 4% | 4.54% | 8.54% | 4.88% | 10 | exempt |
| North Carolina (NC) | 4.75% | 2.35% | 7.1% | 3.5% | 22 | exempt |
| North Dakota (ND) | 5% | 2.09% | 7.09% | 3.75% | 23 | exempt |
| Ohio (OH) | 5.75% | 1.54% | 7.29% | 2.25% | 21 | exempt |
| Oklahoma (OK) | 4.5% | 4.56% | 9.06% | 7% | 6 | state-exempt, local may apply |
| Oregon (OR) | 0% | 0% | 0% | 0% | 47 | no state sales tax |
| Pennsylvania (PA) | 6% | 0.34% | 6.34% | 2% | 34 | exempt |
| Rhode Island (RI) | 7% | 0% | 7% | 0% | 25 | exempt |
| South Carolina (SC) | 6% | 1.49% | 7.49% | 3% | 19 | exempt |
| South Dakota (SD) | 4.2% | 1.91% | 6.11% | 4.5% | 36 | taxed at full rate (4.2%) |
| Tennessee (TN) | 7% | 2.61% | 9.61% | 2.75% | 2 | reduced (4%) |
| Texas (TX) | 6.25% | 1.95% | 8.2% | 2% | 14 | exempt |
| Utah (UT) | 6.1% | 1.32% | 7.42% | 4.7% | 20 | reduced (1.75%) |
| Vermont (VT) | 6% | 0.43% | 6.43% | 1% | 32 | exempt |
| Virginia (VA) | 5.3% | 0.47% | 5.77% | 2.7% | 41 | reduced (1%) |
| Washington (WA) | 6.5% | 3.07% | 9.57% | 4.2% | 3 | exempt |
| West Virginia (WV) | 6% | 0.6% | 6.6% | 1.4% | 31 | exempt |
| Wisconsin (WI) | 5% | 0.72% | 5.72% | 2.9% | 42 | exempt |
| Wyoming (WY) | 4% | 1.39% | 5.39% | 3% | 44 | exempt |

Notes from the Tax Foundation table:
- CA (1.25%), UT (1.25%) and VA (1%) mandatory statewide local add-ons are included in the state rate.
- HI, NM and SD sales taxes have broad bases that include many business-to-business services.
- Special local resort-area taxes are not counted.
- NJ: Salem County and Urban Enterprise Zones collect 3.3125%; shown as a negative average local rate (-0.02).
- Average local rates are population-weighted; the rate at a specific address can differ.
- New Mexico: if gross receipts tax revenue in any FY 2026-2029 falls below 95% of the prior year, the rate returns to 5.125% on the following July 1.

### Groceries

Most states exempt groceries from the sales tax; some tax them at a reduced rate and a few at the full rate (Tax Foundation, Midyear 2026). Per-state grocery treatment in 'consumption[].groceries' combines the Tax Foundation table as of 2020-01-01 with later changes verified from state revenue departments / Tax Foundation: Kansas (0% from 2025), Oklahoma (state 0% from 2024-08-29), Arkansas and Illinois (state 0% from 2026), Alabama (2% from 2025-09-01), Mississippi (5% from 2025-07-01), Virginia (1%), Tennessee (4%). 'Exempt' refers to state-level treatment; local taxes on groceries may apply in some states.

- Taxed at the full state rate: Hawaii, Idaho, South Dakota (Tax Foundation table as of 2020, not re-verified).
- Taxed at a reduced state rate: Alabama 2%, Mississippi 5%, Tennessee 4%, Missouri 1.225%, Utah 1.75%, Virginia 1%. AL, MS, TN and VA come from state revenue departments; MO and UT from the Tax Foundation 2020 table.
- Exempt at state level, local taxes may apply: Arkansas, Illinois, Kansas, Oklahoma.

## 5. Context: OECD tax wedge for the US (`oecd_taxing_wages_db`)

Single worker on the average wage, no children:

| Year | Tax wedge (% labour cost) | Income tax + employee SSC (% gross) | Income tax (% gross) | Employee SSC | Employer SSC | Average wage |
|---|---|---|---|---|---|---|
| 2024 | 30.07 | 24.4 | 16.75 | 7.65 | 8.1 | $70,695 |
| 2025 | 29.98 | 24.31 | 16.66 | 7.65 | 8.09 | $73,520 |

The OECD-average tax wedge was 34.93% in 2024 and 35.08% in 2025.

## 6. Suggested mapping to scenarios (methodology, not a sourced figure)

Federal income tax and FICA are deterministic given income and filing status (same in all scenarios unless the user's deductions are unknown: Conservador could use the standard deduction plus any OBBBA deductions the user plausibly qualifies for; Elevado the standard deduction only). State income tax: Conservador = user's state with local income tax ignored; Provável = state top/brackets plus TF average local effective rate where listed; Elevado = add the max local income tax. Sales tax on taxable spending: Conservador = state rate only and groceries treated per state rule; Provável = combined (state + population-weighted avg local) rate; Elevado = state rate + max local rate.

## Unverified / not included

- **SSA COLA fact sheets 2025/2026 and SSA contribution-base page**: ssa.gov returned HTTP 403 to automated fetches. The wage bases ($176,100 for 2025, $184,500 for 2026) were verified from IRS Publication 15 (2025 and 2026) and IRS Topic 751 instead.
- **Additional Medicare Tax thresholds are not indexed for inflation**: Widely understood (statutory amounts), but the IRS Q&A page read does not state it; thresholds shown are those on the page (reviewed 2026-08-11).
- **Grocery treatment for Hawaii, Idaho, South Dakota (full rate), Missouri (1.225%), Utah (1.75%) and all 'exempt' states**: Taken from the Tax Foundation table as of 2020-01-01; not re-verified against 2026 state sources (Missouri DOR and Utah Tax Commission pages returned HTTP 403). Changes since 2020 were checked only for KS, OK, AR, IL, AL, MS, VA, TN.
- **Utah grocery rate composition**: Unclear whether the TF 1.75% figure includes Utah's mandatory 1.25% local add-on; the commonly cited combined grocery rate (3%) was not verified from a read source.
- **Local grocery taxes in AR, IL, KS, OK, AK and elsewhere**: Local rates on food vary by jurisdiction and were not compiled.
- **California SDI payroll tax rate**: Tax Foundation 2026 page is internally inconsistent: intro text mentions a 1.1% payroll tax, footnote (l) says 1.3% with no wage ceiling. Not checked against the California EDD.
- **Tax Foundation Midyear 2026 text inconsistencies**: Article text says Louisiana combined rate is 10.11% in one paragraph while the table and key findings say 10.13%; text says local averages are 'as of January 1, 2026' while the table header says 'as of July 1, 2026'. JSON uses the table values.
- **State income tax changes after the TF 2026 table (published 2026-02-17)**: Mid-2026 legislation (e.g., Georgia, South Carolina's scheduled reversion to 6.2% on 2026-07-01) not checked; table reflects rates as of 2026-01-01, with some 2025 bracket widths.
- **Self-employment tax, state payroll taxes (CA SDI, NY PFL, NJ, WA Cares, MA PFML), and itemized deductions**: Out of scope for this pass; not researched.
- **OECD Taxing Wages 2026 report and press release pages**: oecd.org returned HTTP 403; US/OECD wedge values read directly from the OECD SDMX database instead. Publication month (April 2026) inferred from the OECD URL path in search results.

## Sources

- ``irs_ir_2024_273``
  - **Publisher:** Internal Revenue Service (IRS)
  - **Title:** [IRS releases tax inflation adjustments for tax year 2025 (IR-2024-273; Rev. Proc. 2024-40)](https://www.irs.gov/newsroom/irs-releases-tax-inflation-adjustments-for-tax-year-2025)
  - **Published:** 2024-10-22
  - **Supports:** 2025 federal brackets (single, MFJ); original 2025 standard deduction before OBBBA ($15,000 / $30,000 / $22,500).
- ``irs_brackets_page``
  - **Publisher:** Internal Revenue Service (IRS)
  - **Title:** [Federal income tax rates and brackets](https://www.irs.gov/filing/federal-income-tax-rates-and-brackets)
  - **Published:** page last reviewed 2026-07-27
  - **Supports:** Confirms the 2025 brackets (single, MFJ) still apply after OBBBA.
- ``irs_ir_2025_103``
  - **Publisher:** Internal Revenue Service (IRS)
  - **Title:** [IRS releases tax inflation adjustments for tax year 2026, including amendments from the One, Big, Beautiful Bill (IR-2025-103; Rev. Proc. 2025-32)](https://www.irs.gov/newsroom/irs-releases-tax-inflation-adjustments-for-tax-year-2026-including-amendments-from-the-one-big-beautiful-bill)
  - **Published:** 2025-10-09
  - **Supports:** 2026 federal brackets and 2026 standard deduction; 2025 standard deduction as raised by OBBBA ($15,750 / $31,500 / $23,625).
- ``irs_obbb_individuals``
  - **Publisher:** Internal Revenue Service (IRS)
  - **Title:** [One, Big, Beautiful Bill provisions - Individuals and workers](https://www.irs.gov/newsroom/one-big-beautiful-bill-provisions-individuals-and-workers)
  - **Published:** page last reviewed 2026-08-27
  - **Supports:** 2025 standard deduction under OBBBA; senior deduction ($6,000, 2025-2028); tips deduction (max $25,000); overtime deduction (max $12,500 / $25,000 MFJ).
- ``irs_pub15_2025``
  - **Publisher:** Internal Revenue Service (IRS)
  - **Title:** [Publication 15 (2025), (Circular E), Employer's Tax Guide](https://www.irs.gov/pub/irs-prior/p15--2025.pdf)
  - **Published:** 2024-12-17
  - **Supports:** 2025 Social Security wage base $176,100; Social Security 6.2% each (employee/employer); Medicare 1.45% each, no wage base.
- ``irs_pub15_2026``
  - **Publisher:** Internal Revenue Service (IRS)
  - **Title:** [Publication 15 (2026), (Circular E), Employer's Tax Guide](https://www.irs.gov/publications/p15)
  - **Published:** 2026 edition (accessed online)
  - **Supports:** 2026 Social Security wage base $184,500; Social Security 6.2% and Medicare 1.45% each for employee and employer.
- ``irs_tc751``
  - **Publisher:** Internal Revenue Service (IRS)
  - **Title:** [Topic no. 751, Social Security and Medicare withholding rates](https://www.irs.gov/taxtopics/tc751)
  - **Published:** page last reviewed 2026-09-24
  - **Supports:** Employee 6.2% Social Security and 1.45% Medicare; 2026 wage base $184,500; employers withhold 0.9% Additional Medicare Tax on wages over $200,000.
- ``irs_addl_medicare_qa``
  - **Publisher:** Internal Revenue Service (IRS)
  - **Title:** [Questions and answers for the Additional Medicare Tax](https://www.irs.gov/businesses/small-businesses-self-employed/questions-and-answers-for-the-additional-medicare-tax)
  - **Published:** page last reviewed 2026-08-11
  - **Supports:** Additional Medicare Tax 0.9%; thresholds $250,000 MFJ, $125,000 MFS, $200,000 single/HoH/qualifying widow(er).
- ``tf_sales_2026_midyear``
  - **Publisher:** Tax Foundation
  - **Title:** [State and Local Sales Tax Rates, Midyear 2026 (Abir Mandal)](https://taxfoundation.org/data/all/state/2026-sales-tax-rates-midyear/)
  - **Published:** 2026-07-06
  - **Supports:** State, average local, max local and combined sales tax rates for 50 states + DC as of 2026-07-01; population-weighted national average 7.53%; Illinois ended its 1% state grocery tax on 2026-01-01.
- ``tf_income_2026``
  - **Publisher:** Tax Foundation
  - **Title:** [State Individual Income Tax Rates and Brackets, 2026 (Janelle Fritts, Katherine Loughead)](https://taxfoundation.org/data/all/state/state-income-tax-rates-2026/)
  - **Published:** 2026-02-17
  - **Supports:** State individual income tax rates and brackets as of 2026-01-01 (top marginal rates, flat/graduated/none); local income tax note.
- ``tf_state_changes_2026``
  - **Publisher:** Tax Foundation
  - **Title:** [2026 State Tax Changes Taking Effect January 1st (Nicole Fox, Jacob Macumber-Rosin)](https://taxfoundation.org/research/all/state/2026-state-tax-changes/)
  - **Published:** 2025-12-18 (updated 2026-01-06)
  - **Supports:** Arkansas and Illinois state grocery tax repealed 2026-01-01 (local taxes may still apply); state income tax rate cuts for 2026.
- ``tf_state_changes_2025``
  - **Publisher:** Tax Foundation
  - **Title:** [2025 State Tax Changes Taking Effect January 1st](https://taxfoundation.org/research/all/state/2025-state-tax-changes/)
  - **Published:** 2024-12-19 (updated 2025-06-25)
  - **Supports:** Kansas state grocery tax cut from 2% to 0% on 2025-01-01.
- ``tf_grocery_2020``
  - **Publisher:** Tax Foundation
  - **Title:** [Halloween Candy Tax: Sales Tax Treatment of Groceries, Candy, and Soda (Janelle Fritts) - table 'as of January 1, 2020'](https://taxfoundation.org/data/all/state/halloween-candy-tax-groceries-soda-sales-tax/)
  - **Published:** 2020-10-28 (updated 2023-07-24)
  - **Supports:** Baseline state grocery treatment (full / reduced / exempt) as of 2020-01-01; used only where no newer verified change exists.
- ``ok_otc_grocery``
  - **Publisher:** Oklahoma Tax Commission
  - **Title:** [State Sales Tax on Food and Food Ingredients](https://oklahoma.gov/tax/businesses/state-sales-tax-on-food-and-food-ingredients.html)
  - **Published:** n/d (effective 2024-08-29)
  - **Supports:** Oklahoma state sales tax on food eliminated from 2024-08-29 (HB 1955); local sales taxes still apply.
- ``ms_dor_grocery``
  - **Publisher:** Mississippi Department of Revenue
  - **Title:** [Reduced Sales Tax on Groceries Begins July 1](https://www.dor.ms.gov/news/reduced-sales-tax-groceries-begins-july-1)
  - **Published:** 2025-05-01
  - **Supports:** Mississippi grocery sales tax reduced from 7% to 5% effective 2025-07-01 (HB 1, 2025).
- ``al_dor_food``
  - **Publisher:** Alabama Department of Revenue
  - **Title:** [NOTICE: State Sales and Use Tax Rate Reduced on Food Beginning September 1, 2025](https://www.revenue.alabama.gov/notice-state-sales-and-use-tax-rate-reduced-on-food-beginning-september-1-2025/)
  - **Published:** 2025-07-02
  - **Supports:** Alabama state sales tax on food reduced from 3% to 2% effective 2025-09-01 (Act 2025-305).
- ``tn_dor_rates``
  - **Publisher:** Tennessee Department of Revenue
  - **Title:** [Due Dates and Tax Rates (Sales and Use Tax)](https://www.tn.gov/revenue/taxes/sales-and-use-tax/due-dates-and-tax-rates.html)
  - **Published:** n/d
  - **Supports:** Tennessee general state rate 7%; food and food ingredients 4%; local rate cap 2.75%.
- ``va_tax_grocery``
  - **Publisher:** Virginia Department of Taxation
  - **Title:** [Grocery Tax; and news 'Grocery Tax Reduced to 1% Beginning Jan. 1, 2023'](https://www.tax.virginia.gov/grocery-tax)
  - **Published:** n/d (rate effective 2023-01-01)
  - **Supports:** Food for home consumption taxed at a reduced 1% throughout Virginia (from 2023-01-01).
- ``oecd_taxing_wages_db``
  - **Publisher:** OECD
  - **Title:** [Taxing Wages database - 'Labour taxation - OECD comparative country indicators' (dataflow OECD.CTP.TPS DSD_TAX_WAGES_COMP@DF_TW_COMP v2.1), read via the OECD SDMX API](https://sdmx.oecd.org/public/rest/data/OECD.CTP.TPS,DSD_TAX_WAGES_COMP@DF_TW_COMP,/all?startPeriod=2023&dimensionAtObservation=AllDimensions)
  - **Published:** data through 2025 (Taxing Wages 2026 edition, published April 2026)
  - **Supports:** US tax wedge and components for a single worker at 100% of the average wage, 2024 and 2025 (context only).

All sources accessed 2026-10-17.
