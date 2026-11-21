# Life timeline: study rationale

Why the "Life timeline" works the way it does, where the numbers come from and how to maintain
it. This is the reference document. The design, with the discarded alternatives, is in
[life-expectancy.md](life-expectancy.md), and the checked numbers are in
[life-expectancy-validation.md](life-expectancy-validation.md). For each family member, the feature
shows the stages of life and how far life usually goes, to plan the money (migration V131).

## 1. The question the app answers

The question is not "when do I die". It is "to what age does my plan need to last, and how does
the family look along the way". Three practical questions come out of it:

- Does the retirement plan go to a reasonable age?
- When I retire, how old will my children be?
- What is the chance that the money needs to last until 95?

## 2. The three method choices, and why

### 2.1 Cohort, not the current year (period)

- **Period expectancy** (the one that makes the news): imagines someone living their whole life
  with the mortality rates of a single year. It is a snapshot of today.
- **Cohort expectancy**: follows those born in a year through the rates that this generation
  will actually face at each age. Past rates are observed and future ones are projected.
- **Why cohort:** mortality keeps falling, so period expectancy underestimates how long people
  live, and underestimates it more for the youngest. In Brazil, according to the UN, someone born
  in 2023 has 84.1 years by cohort, against 76.0 from the 2024 period table. Planning money with
  the period figure makes the plan end too early.

### 2.2 Conditioned on today's age

- Someone who is already 60 has passed the risks of childhood and youth and lives, on average,
  longer than the expectancy at birth. In Brazil in 2024, according to the IBGE, expectancy at
  birth is 76.6 years, but someone who is 60 lives on average to about 82.6.
- That is why the calculation starts at today's age and only uses the years from today on. This
  also discards the effect of COVID (2020 to 2022), which weighs on the period tables of those
  years.

### 2.3 Ranges and chances, never a date

- The bar shows "half of the people like you pass 84; 1 in 4 passes 91; 1 in 10 passes 95", with
  the end faded between those three points, and never an end point.
- The **median** is the main number: it is what half pass. The mean stays only in the technical
  detail, because it is pulled up by the few who live very long.
- **"1 in 10"** is the prudent horizon for the money. That is why it is the suggestion for the
  retirement plan's horizon.

## 3. Where the numbers come from

| Source | What we use | Why |
|---|---|---|
| UN, World Population Prospects 2024 (WPP 2024), medium variant | Complete life tables age by age (0 to 100+), by sex, from 1950 to 2100, for all the countries in the catalog | It is the only source that covers the app's 63 countries and has projections year by year up to 2100, which is what makes cohort expectancy possible. |
| IBGE, Complete Mortality Tables 2024 (Tábuas Completas de Mortalidade) | Period table for Brazil, by sex, up to 90+ | It is the official table that the INSS uses in the pension factor and the number Brazilians know. It goes in as a **reference** alongside and in the tests, not as the engine, because the IBGE does not publish the age-by-age projections that the cohort needs. |

Sources considered and not used:

- **Human Mortality Database:** does not cover Brazil.
- **ONS and SSA:** each covers a single country. They are used to validate the United Kingdom and
  the US.
- **BR-EMS (SUSEP):** it is the insured population, which lives longer than average, and it has
  no clear open license.
- **WHO healthy life expectancy (HALE):** uncertain license and data only up to 2021.

### 3.1 Licenses

- **UN:** Creative Commons **CC BY 3.0 IGO**. The license was confirmed on the UN's own site: the
  data portal API specification (population.un.org/dataportalapi, swagger v1) declares "Creative
  Commons license CC BY 3.0 IGO", and the WPP 2024 documents on population.un.org carry the same
  note. Requirements, all met:
  - the exact attribution;
  - stating that the data were trimmed and transformed;
  - not suggesting UN endorsement;
  - keeping the data under their license.

  It is in `DATA_LICENSES.md` and in Settings › About and support › Third-party data.
- **IBGE:** reproduction citing the source ("IBGE, Tábuas Completas de Mortalidade 2024");
  official data have no copyright protection (Lei 9.610/98, art. 8, the Brazilian copyright law).

## 4. The calculation

For a member with age `a` today (year `t`), sex `s` and the family's country `c`:

```
q(k)   = qx da ONU na idade k, no ano t + (k − a), sexo s, país c   (a diagonal da geração)
S(a)   = 1
S(x+1) = S(x) × (1 − q(x))            chance de estar vivo na idade x+1, sabendo que tem a hoje
e(a)   = 0,5 + Σ S(x), x > a           anos que ainda restam, em média
```

- After 2100, it repeats the 2100 rates. After 100, it uses the open group of the table, stored
  as a constant force of mortality that gives the UN expectancy at 100: q = 1 − exp(−1 / e100).
- **Median, "1 in 4" and "1 in 10"** are the ages at which S falls to 50%, 25% and 10%.
- **Expectancy at birth by cohort**, shown only as a comparison between generations: the same
  calculation from age 0, for those born from 1950 to 2026. For those born before 1950 there is
  no table, and the number does not appear.
- **Couple** (holder and spouse, assuming independence): the chance that at least one is alive is
  1 − (1 − S₁) × (1 − S₂). This is the horizon of a couple's wealth, because the money needs to
  last while either of them is alive.
- **Sex:** man and woman use their own table. "Prefer not to say" and no answer use the
  both-sexes one.
- **Country:** the family's (the vault's), for all members.

Code:

- `crates/treasury-core/src/longevity/math.rs`: curve, quantiles, couple;
- `data.rs`: reading the tables;
- `mod.rs`: stages, adjustments and the result per member;
- `milestones.rs`: financial milestones.

## 5. The stages of life

| Stage | Ages | Basis |
|---|---|---|
| Childhood | 0 to 11 | ECA (Lei 8.069/1990, art. 2, the Child and Adolescent Statute) |
| Adolescence | 12 to 17 | ECA |
| Youth | 18 to 29 | Youth Statute (Lei 12.852/2013) |
| Adulthood | 30 to 59 | Convention, the stage of the most income and of accumulating |
| Longevity | 60 or more | Statute of the Older Person (Lei 10.741/2003, renamed by Lei 14.423/2022) and the WHO |
| Advanced longevity (optional, off) | 80 or more | Special priority (Lei 13.466/2017) |

- **"Seniority" was discarded:** it sounds like a job title ("senior professional").
- **"Old age", "third age" and "end of life" were also avoided.**
- **Retirement is not a stage, it is a milestone:** it depends on each person's plan.

## 6. Decisions

| # | Decision |
|---|---|
| P1 | Name "Life timeline" (pt "Linha da vida", es "Línea de vida"), and not "life expectancy", which sounds like a countdown. |
| P2 | The five stages of section 5; the 80+ one only as an optional visual subdivision. |
| P3 | Engine: UN by cohort for all countries; IBGE 2024 as the reference in Brazil. |
| P4 | Main number: the median conditioned on today's age; "1 in 4" and "1 in 10" in the fade. |
| P5 | Children: stages and milestones; the faded end only if an adult turns it on. |
| P6 | Member's sex: a new optional field, also used as the default in the INSS. |
| P7 | Retirement plan horizon: the "1 in 10" age, the couple's when there is a spouse, rounded up and editable; with the feature off, it stays at 95. |
| P8 | UN license confirmed on the official site before embedding. |
| P9 | No private pension tables (BR-EMS) for now. |
| P10 | The family's country for all members. |
| P11 | Question in the questionnaire: the 8th, no points, "Yes / No / Decide later"; the feature starts off. |
| P12 | Agent and MCP: read only, only with the feature on, and they only talk about the subject if asked or in the retirement plan, always as "population averages". |

## 7. Limits (shown in "How we calculate")

- They are averages of the country's population. They do not account for health, income,
  smoking, region or family history, which change each person's result a lot.
- The future is the UN's average projection, not a fact.
- In Brazil, the UN rates are modeled and sit slightly below the IBGE's (up to 0.9 year, section
  8).
- Someone who lives in a country different from the vault's country gets the vault country's
  table.
- Nothing leaves the computer: the tables ship inside the app and the calculation is done on the
  spot from the date of birth, sex and country. Turning the feature off does not delete anything,
  because nothing is stored beyond the preference.

## 8. Validation (summary; details in [life-expectancy-validation.md](life-expectancy-validation.md))

| Case | App (UN) | Official | Difference | Tolerance |
|---|---|---|---|---|
| Brazil 2024, at birth (period, both) | 76.03 | IBGE 76.6 | −0.58 | 1.0 |
| Brazil 2024, at 60 (period, women) | 23.26 | IBGE 24.2 | −0.89 | 1.0 |
| United Kingdom, boys of 2023 (cohort) | 88.01 | ONS 86.7 | +1.31 | 1.5 |
| United Kingdom, girls of 2023 (cohort) | 90.79 | ONS 90.0 | +0.79 | 1.5 |
| US (cohort) | 81.2 and 85.4 for those born in 1990 | SSA | pending | — |

- The built-in IBGE reference reproduces what is published.
- The app's calculation reproduces the UN's own expectancy (76.03 against 76.023 in the UN file).
- In all 63 countries and for the three sexes, the basic properties hold:
  - survival never rises with age;
  - the median is always above today's age;
  - the couple is never below either one;
  - cohort expectancy is always greater than or equal to period expectancy.
- **SSA pending:** the SSA site refused access (HTTP 403) when the validation was done.

## 9. How to update

**When:**

- with each new UN revision, which comes out every two years (the next should be WPP 2026);
- with each new IBGE table, which comes out by December 1 of each year.

It goes on the annual task list together with the INSS tables in January.

1. `python3 -I scripts/longevity-data.py --urls` shows the files and the addresses. Download only
   with the maintainer's authorization, into an empty folder outside the repository:
   - UN: six files of about 200 MB each (`WPP2024_Life_Table_Complete_Medium_{Both,Female,Male}_{1950-2023,2024-2100}.csv.gz`);
   - IBGE: three 22 KB spreadsheets (`ambos_os_sexos.xlsx`, `homens.xlsx`, `mulheres.xlsx`, in
     `ftp.ibge.gov.br/Tabuas_Completas_de_Mortalidade/`).

   Check the UN files with `gzip -t`.
2. `python3 -I scripts/longevity-data.py PASTA_DA_ONU --ibge PASTA_DO_IBGE` generates
   `crates/treasury-core/data/longevity/wpp2024.zip` (about 1.7 MB) and prints a summary that
   already compares with the IBGE and the ONS.
3. `cargo test -p treasury-core longevity` and, with `-- --nocapture`, the validation table. If a
   difference goes over the tolerance, investigate before touching the tolerance.
4. Update section 8 of this document and
   [life-expectancy-validation.md](life-expectancy-validation.md).
5. If the revision year changed, update:
   - the attributions in `DATA_LICENSES.md` and `locales/*/nucleo-longevidade.ftl`;
   - the fixed years of the zip format (`index.json`: `years`, `cohort_years`).
6. Delete the downloaded files.

## 10. Out for now (and what would change)

- **Adjustment for health, smoking or income:** would require tables by subgroup, which do not
  exist openly for all countries.
- **Healthy life expectancy:** WHO license and data.
- **Private pension tables (BR-EMS):** a "conservative" mode, after confirming the license with
  SUSEP.
- **Region within the country, country per member, PDF report and appearing on the Dashboard:**
  only if someone asks.

## Sources

- UN, World Population Prospects 2024: https://population.un.org/wpp/ ·
  methodology: https://population.un.org/wpp/assets/Files/WPP2024_Methodology-Report_Final.pdf ·
  license: https://creativecommons.org/licenses/by/3.0/igo/ · license confirmation:
  https://population.un.org/dataportalapi/index.html
- IBGE, Tábuas Completas de Mortalidade 2024:
  https://biblioteca.ibge.gov.br/visualizacao/periodicos/3097/tcmb_2024.pdf
- ONS, period and cohort:
  https://www.ons.gov.uk/peoplepopulationandcommunity/birthsdeathsandmarriages/lifeexpectancies/methodologies/periodandcohortlifeexpectancyexplained
- SSA, cohort tables: https://www.ssa.gov/oact/HistEst/CohLifeTables/2024/CohLifeTables2024.html
- Goldstein and Wachter (2006), the difference between period and cohort.
- Spiegelhalter, Pearson and Short, "Visualizing uncertainty about the future", Science, 2011.
