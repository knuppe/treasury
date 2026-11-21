# Life timeline: longevity by family member

The "Life timeline" shows, for each family member, the stages of life and how far life usually goes in the country's population, as ranges and chances, never a date, to plan money (the retirement horizon, for example). It ships turned off and is turned on in Settings or in the welcome questionnaire. The data comes from the UN (World Population Prospects 2024, CC BY 3.0 IGO) and, for Brazil, from IBGE (Complete Mortality Tables 2024), trimmed by `scripts/longevity-data.py` into `crates/treasury-core/data/longevity/wpp2024.zip`, with the attribution in `DATA_LICENSES.md` and in Settings › About and support. This document is the design, with the discarded alternatives; the rationale for consultation and updates is in [life-expectancy-rationale.md](life-expectancy-rationale.md) and the checking of the numbers is in [life-expectancy-validation.md](life-expectancy-validation.md).

## Data sources

### Main source: UN, World Population Prospects 2024 (WPP2024)

- **What it is:** complete life tables (age by age, 0 to 100+), by sex (men, women, both), for 237 countries and areas, **from 1950 to 2100** (1950 to 2023 estimated; 2024 to 2100 projected, medium variant). It has `qx` (probability of dying between age x and x+1), `lx`, `Lx`, `ex` (years still remaining for someone aged x) and the rest of the table.
- **Files:** `WPP2024_Life_Table_Complete_Medium_{Both,Female,Male}_{1950-2023,2024-2100}.csv.gz`
  at https://population.un.org/wpp/downloads (CSV). Methodology:
  https://population.un.org/wpp/assets/Files/WPP2024_Methodology-Report_Final.pdf
- **Why this one:** it is the only consistent source that covers all the countries in our catalog **and** has year-by-year projections to 2100, which is what makes it possible to compute cohort expectancy.
- **License:** Creative Commons **CC BY 3.0 IGO** (attribution; no "share alike" and no "non-commercial"). It can go inside an AGPL app, as long as:
  - the attribution appears: "United Nations, Department of Economic and Social Affairs, Population Division (2024). World Population Prospects 2024, Online Edition.";
  - the app says the data was trimmed and transformed;
  - it does not suggest UN endorsement;
  - the data keeps its own license in a separate file (`DATA_LICENSES.md` or `NOTICE`), without being relicensed as AGPL.
  - The license was confirmed through mirrors (Our World in Data, the WPP data repository); the UN rights page did not open from here. **Confirm before embedding** (question P8).
- **Updates:** the UN publishes every two years (the next should be WPP 2026). The January 2026 revision only touched Togo and brought no new tables.

### Brazil: IBGE, Complete Mortality Tables 2024

- Published on 2025-11-28 (https://biblioteca.ibge.gov.br/visualizacao/periodicos/3097/tcmb_2024.pdf). Men, women and both, age by age up to 90+.
- Brazil 2024: life expectancy at birth of **76.6** years (73.3 men, 79.9 women); at age 60, **22.6** remain (20.8 men, 24.2 women); at 80, 8.3 (men) and 9.5 (women).
- It is the table INSS (Brazil's social security institute) uses in the fator previdenciário (Lei 9.876/1999 and Decreto 3.266/1999 require IBGE to publish it by December 1 of each year). It is the number Brazilians see in the news.
- **It is a period table** (one year's rates only). IBGE's population projection (2024 revision, 2000 to 2070) has life expectancy at birth rising to 83.9 in 2070, but it does not publish the complete table year by year in an easy-to-embed format.
- **License:** IBGE does not use a formal Creative Commons license; historically it allows reproduction citing the source, and raw data and official acts have no copyright protection (Lei 9.610/98, art. 8, the Brazilian copyright law). Embedding with "Source: IBGE, Tábuas Completas de Mortalidade 2024" is safe in practice.
- **Proposed use:** reference and validation for Brazil, shown alongside ("by the 2024 IBGE table, which INSS uses, …"), and not the calculation engine (question P3).

### Sources considered and not used as the base

| Source | Why it is not the base |
|---|---|
| Human Mortality Database (CC BY 4.0) | About 41 countries, almost all rich; **does not cover Brazil**. |
| ONS (United Kingdom) and SSA (US) | They publish real cohort expectancy (e.g., UK babies born in 2023: 86.7 boys and 90.0 girls, against 78.8 and 82.8 by period). They only serve one country each. **Great for validating** our calculation ([life-expectancy-validation.md](life-expectancy-validation.md)). |
| BR-EMS (SUSEP, 2021) | Tables for the insured population of private pension plans, who live longer than average. Interesting for a "conservative" mode, but with no clear open license. Left out of the first version (question P9). |
| WHO HALE (healthy life expectancy) | Uncertain license (CC BY 4.0 in one place, CC BY-NC-SA 3.0 IGO in another) and data only up to 2021. Left out of the first version. |

## Calculation method

### Period and cohort (generation)

- **Period expectancy** (the one in the news): imagines someone living their whole life at the mortality rates of a single year. It is a snapshot.
- **Cohort expectancy**: follows those born in a given year through the rates that this generation actually faces at each age (past ones, observed; future ones, projected). Since mortality keeps falling, it is higher than the period one, and more so for the young. This is what gives a newborn a much higher expectancy than their parent.

### Expectancy conditional on current age (the main number)

For a member aged `a` today (year `t`), sex `s`, country `c`:

```
q(k)  = qx do WPP para idade k, ano t + (k − a), sexo s, país c   (a diagonal da geração)
S(a)  = 1
S(x)  = S(x−1) × (1 − q(x−1))         probabilidade de estar vivo na idade x, sabendo que tem a hoje
e(a)  = Σ_{x>a} S(x) + ~0,5           anos que ainda restam, em média
```

- After 2100 it repeats the 2100 rates; after 100 it uses the table's open group.
- **It only uses years from today on.** Conditioning on the current age discards the past, including the COVID years (2020 to 2022).
- All the bar's numbers come from the `S(x)` curve:
  - **median:** the age at which S falls to 50% ("half of people like you live past …");
  - **1 in 4:** S = 25%;
  - **1 in 10:** S = 10% (the prudent horizon for money);
  - the mean `a + e(a)` stays in the technical detail, not in the bar.

### Expectancy at birth by cohort (to compare generations)

To show "your daughter, born in 2022, has a life expectancy at birth of X; you, born in 1985, had Y", the same calculation from age 0 and the birth year, along the 1950–2100 diagonal. For those born before 1950 it is not possible (it does not appear). It is a context number; the bar always uses 4.2.

### Couple and family

With two adults (account holder and spouse), assuming independence:

```
P(pelo menos um vivo no ano y) = 1 − (1 − S₁(y)) × (1 − S₂(y))
```

It is the right horizon for a couple's wealth: the money has to last while **either one** is alive. For example, in the US, for a couple aged 65 the chance that one of them reaches 90 is close to 45–50% (secondary sources; Actuaries Longevity Illustrator, from the SOA and the American Academy of Actuaries).

### Limits (shown in "How we calculate")

- They are averages for the country's population. They do not consider health, income, smoking, region or family history, which change each person's result a lot.
- The future is the UN's medium projection, not a fact.
- In Brazil, the UN rates are modeled and may differ a little from IBGE's ([life-expectancy-validation.md](life-expectancy-validation.md)).
- The country is the family's. Someone who lives in a country other than the vault's gets the vault country's table (question P10).

## Life stages (validating the categories)

The starting idea had four: childhood, youth, adulthood, seniority. Validation:

- **"Senioridade"** in Brazilian Portuguese sounds like a job level ("profissional sênior"). For the life stage, the respectful and legal term is **pessoa idosa** (older person; Estatuto da Pessoa Idosa, Lei 10.741/2003, with the name updated by Lei 14.423/2022). As a stage name, **"Later life"** is positive and neutral. Avoid "old age", "senile", "golden age" (dated) and "end of life".
- **Adolescence** is missing, which is a legal stage of its own (ECA, the Statute of the Child and Adolescent) and financially distinct (allowance, young apprentice from age 14, first account).
- "Adulthood" from 18 to 60 is too long to guide decisions. Splitting it into youth (up to 29, the limit of the Estatuto da Juventude) and adulthood helps show the accumulation stage.

**Proposal (5 stages, 6 with the optional one):**

| Stage | Ages | Basis | Financial reading |
|---|---|---|---|
| Childhood | 0–11 | ECA (Lei 8.069/1990, art. 2) | Costs of raising, education, reserve for the children |
| Adolescence | 12–17 | ECA | Allowance, financial education, first earnings |
| Youth | 18–29 | Estatuto da Juventude (Lei 12.852/2013, up to 29) | First job, college, starting to invest |
| Adulthood | 30–59 | Convention (not a legal category) | Stage of highest income and accumulation |
| Later life | 60+ | Estatuto da Pessoa Idosa and WHO (60+) | Enjoying the wealth, retirement income |
| (optional) Advanced later life | 80+ | Special priority of Lei 13.466/2017 | Health and care weigh more in the budget |

- **Retirement is not a stage, it is a milestone**, because it varies (55 in some people's plan, 65 in others') and is a financial choice, not a biological stage.
- The limits sit in a table in the core, the same for all countries in the first version (the legal frameworks above are Brazilian, but the WHO's and the UN's coincide at the ends: 60+ and adolescence around 10 to 19).
- en: Childhood, Adolescence, Youth, Adulthood, Later life (Advanced later life).
  es: Infancia, Adolescencia, Juventud, Vida adulta, Longevidad (Longevidad avanzada).

## Interface

### The bar ("Life timeline")

```
 Infância  Adolesc.  Juventude        Vida adulta                Longevidade
|████████|██████|██████████|███████████▌·······················|·········░░░░░░▒▒▒▒▓▓▓▓|
0        12     18         30          41 hoje         62 aposentadoria  84 · 91 · 96
                                                    ▲ INSS 65    ▲ plano vai até 95
```

- **Filled** up to today's age; the rest in a soft tone. The stages as segments in the theme's color.
- **A faded end, not a wall:** the end of the bar is a gradient between the median (50%), the "1 in 4" and the "1 in 10". Never an end point.
- **Financial milestones** as pins above or below the bar, with the value on hover:
  - retirement (from the plan), INSS start (INSS calculation), how far the plan lasts with 90% success;
  - end of financing and debts (`debt` module), goals with a date (`goals`), life events from the planner (college, inheritance).
- **Text alongside**, in the profile's tone:
  - simple: "By Brazil's averages, of every 100 men born in 1985 who reached 41, half live past 84 and 1 in 4 past 91.";
  - technical: "Median 84.2 · P25 91 · P10 95 · e(41) = 43.1 (cohort, WPP 2024, BR, male)".
- **Source always visible** in one line: "Population averages. Source: UN, WPP 2024" (and in Brazil, "and IBGE 2024").
- Discreet motion (the bar fills once on opening), respecting the system's "reduce motion" and the themes (in Scanner, the sweep; in Lo-fi, no extra animation), like the current animations.
- Children: shows stages and milestones (e.g., "college at 18"), and the faded end only if the adult turned on "show the end of the bar for children" (question P5).

### Two views for the family

1. **By age** (default): one bar per member, all from 0 to 100, to see which stage each one is in.
2. **Timeline**: the axis becomes the calendar year (from today to, for example, 2090), with the members' bars side by side. It answers "when I retire in 2046, Ana will be 24 and in youth". It marks the year in which the chance that **at least one** of the adults is alive drops to 25% and 10%.

### Where it appears

- **Family:** a "Life timeline" panel with the bars of all members (only with the feature on). It is the main place.
- **Member profile** (Settings › Family, and the member's detail): their bar.
- **Retirement planner:** the account holder's (or the couple's) bar above the Monte Carlo chart, and the "life expectancy" field starts suggesting the "1 in 10" age.
- **"View as" a member:** shows only that member's bar.
- **Dashboard:** does not appear (the dashboard is for routine, and the subject is sensitive). May be revisited later.
- **⌘K bar:** a "Life timeline" command that leads to the panel, only when on.

## Link with finances and retirement

- **Plan horizon:** today the default is a fixed 95. With the feature on and the member with a date of birth, the default becomes the account holder's "1 in 10" age (or the couple's, by the chance of at least one alive), rounded up. The field stays editable and the typed value always wins. With the feature off, nothing changes (95).
- **Longevity risk sentence** in the plan result: "Your plan goes to 95. By the averages, 1 in 10 people like you live past that age" or "… about 3 in 100 do".
- **Current age** in the plan comes from the member's date of birth when there is one (today it is typed).
- **INSS:** starts reading the member's sex and date of birth as the default (still editable).
- **Agent (MCP):** the read tools `get_member_outlook` and `get_family_outlook` (commands `longevity.outlook` and `longevity.family`) only exist for the agent when the feature is on. The agent only touches the subject if the person asks or in retirement planning, always as "population averages", and cites the source in words ("UN tables from 2024"), never the tool name. No write tool.
- **PDF report:** left out of the first version.

## Turning on, turning off and the questionnaire

- **Off by default** in every vault (new or old) and in the demo vaults.
- **Welcome questionnaire:** an 8th question, with no points (like the income one), at the end:
  > **Do you want to see the family's life timeline?**
  > A bar with each person's life stages and the money plans along it, based on averages for the country's population. Some people prefer not to see this kind of information.
  > · Yes, I want to see it · No, thanks · Decide later
  - "Yes" turns it on and, if anyone's sex or birth date is missing, the Family screen asks for it (with "I prefer not to say").
  - "No" and "Decide later" leave it off; "skip this question" too.
  - Whoever chooses "use the full interface" keeps it off.
  - When redoing a member's questionnaire (not the family's), the question does not appear: the key belongs to the family. The member can hide only their own bar (below).
- **Settings › Profile › Life timeline:** turns it on and off at any time, and per member "show my bar" (default yes, when on).
- **Turning off erases nothing** beyond the preference, because nothing is stored: everything is computed on the spot from the date of birth, sex and country.

## Data in the app

- **Script** `scripts/longevity-data.py` (outside the app): downloads the WPP2024 CSVs, trims the catalog's countries (`country/catalog.rs`), the three sexes, ages 0–100, and generates:
  - `qx` from 2024 to 2100 (the part for the main calculation), as 16-bit integers (qx × 65535);
  - life expectancy at birth by cohort already computed for those born from 1950 to 2026 (a small table, for the comparison between generations), instead of embedding all of 1950–2023;
  - for Brazil, the IBGE 2024 tables (period) as a reference.
- Estimated size: 63 countries × 3 sexes × 77 years × 101 ages ≈ 1.5 million values, about 3 MB raw and around 1 MB compressed in the binary (`include_bytes!`). Acceptable next to the ~13 MB of the current installer.
- **Licenses** in a `DATA_LICENSES.md` at the root and in Settings › About, with the exact UN and IBGE attribution.
- **Updates:** at each new WPP (every two years) and each IBGE table (December); it goes on the annual task list together with the January INSS.
- **No internet:** nothing is downloaded at run time; nothing about the members leaves the machine.

## In the code

- Migration V131: `member.sex TEXT NULL` with `male | female | unspecified` (no value = not stated).
- `longevity` module in `treasury-core`:
  - `outlook(conn, member_id) -> MemberOutlook { age, stage, median_age, p25_age, p10_age,
    remaining_mean, cohort_e0, period_e0_ref, curve: Vec<(age, survival)>, source }`;
  - `family_outlook(conn)` with the "at least one adult alive" curve;
  - `stages()` with the stages from "Life stages";
  - setting `longevity.enabled` (family) and `longevity.hidden_members`.
- `goals.retirement`: horizon default per "Link with finances and retirement" when on.
- MCP: `get_member_outlook` and `get_family_outlook` (read, only with the feature on).
- Texts in pt-BR, en and es (`locales/*/longevidade.ftl`, keys `longevidade-*`).
- UI: `features/longevity/LifeBar.tsx` (one bar), `FamilyLifeBars.tsx` (the two views), used in Family, in the profile and in the planner.

## Limitations

No adjustment for health, smoking or income; no healthy life expectancy (HALE); no private pension tables (BR-EMS); no region within the country; it does not appear in the PDF report or on the Dashboard.

## Sources

- UN, World Population Prospects 2024: https://population.un.org/wpp/downloads ·
  methodology https://population.un.org/wpp/assets/Files/WPP2024_Methodology-Report_Final.pdf ·
  license https://creativecommons.org/licenses/by/3.0/igo/
- IBGE, Complete Mortality Tables 2024:
  https://biblioteca.ibge.gov.br/visualizacao/periodicos/3097/tcmb_2024.pdf ·
  https://agenciadenoticias.ibge.gov.br/pt/agencia-noticias/45275-expectativa-de-vida-chega-a-76-6-anos-em-2024.html
- ONS, period and cohort:
  https://www.ons.gov.uk/peoplepopulationandcommunity/birthsdeathsandmarriages/lifeexpectancies/methodologies/periodandcohortlifeexpectancyexplained
- SSA, cohort tables: https://www.ssa.gov/oact/HistEst/CohLifeTables/2024/CohLifeTables2024.html
- Goldstein and Wachter (2006), difference between period and cohort:
  https://u.demog.berkeley.edu/~jrw/Biblio/Eprints/%20G-I/goldstein.wachter.2006_GapsLags_PopStudies.pdf
- Estatuto da Pessoa Idosa (Lei 14.423/2022): https://www.planalto.gov.br/ccivil_03/_Ato2019-2022/2022/Lei/L14423.htm
- ECA (Lei 8.069/1990), Estatuto da Juventude (Lei 12.852/2013), Lei 13.466/2017.
- WHO, ageing: https://www.who.int/news-room/q-a-detail/population-ageing
- Longevity risk and couples: Actuaries Longevity Illustrator (SOA and American Academy of
  Actuaries), https://www.planadviser.com/american-academy-actuaries-soa-update-longevity-illustrator/
- Communicating uncertainty: Spiegelhalter, Pearson and Short, "Visualizing uncertainty about the future",
  Science, 2011.
- SUSEP, BR-EMS 2021: Circular SUSEP 623/2021.
