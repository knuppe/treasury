# College goal for the children

For each child, the family defines which college they want to pay for (course, public or private, city,
duration) and the app calculates how much is needed at the start of the course and how much to contribute per month, in
today's reais, with education inflation above general inflation. The plan is linked to a goal
of type `education`, with the investment positions the family set aside for it, and is
recalculated every year. Brazil only (phase 1). Example numbers are invented.

## How the cost is defined

The user answers a few questions (the rest has defaults):

| Field | Invented example | Default |
|---|---|---|
| Child | Ana, 8 years old | members with the relationship `filho` (child); also "child yet to be born" (expected date) |
| Start age | 18 | 18 (BR), 18 (US/PT) |
| Start month | February | February (BR), September (US, PT, Europe) |
| Type | private | private / public / abroad |
| Course (free text) | Engineering | optional; only as a reminder and for the agent |
| Duration | 5 years | 4 years; Medicine suggests 6 |
| Today's tuition | R$ 2,500 | no default: the app asks for a real quote from today |
| Installments per year | 12 | 12 (BR); 1 or 2 (US/PT, annual or semester) |
| Extra costs per year | R$ 3,000 (materials, enrollment) | 0 |
| Monthly living cost | — | 0; used mainly for **public** (housing, transport, food in another city) |
| Already saved | R$ 20,000 | 0 |

Details:

- **Public**: tuition 0, and the cost becomes the **living cost** during the course. Many families
  forget this; the app suggests filling it in.
- **Today's tuition is the anchor.** The index only projects the path. Once a year the app asks to
  update it with that year's real tuition (the real price is worth more than the index).
- **Scholarship or discount** (ProUni, FIES, the college's own scholarship): a discount % field,
  default 0. The app does not estimate scholarships.
- The app **does not bring a table of tuition by course**. There is no open, up-to-date official source
  per institution; inventing an average misleads more than it helps (decision subject to.

## Education inflation

### The official numbers

| Period | IPCA | IPCA · Education | Source |
|---|---|---|---|
| 2025 (year) | 4.26% | 6.22% | IBGE, IPCA 2025-12, via the Ministry of Planning, *Informe IPCA* 2026-01 |
| 2024 (year) | 4.83% | 6.70% | same report (table) |
| 2026-02 (month) | 0.70% | 5.21% (regular courses 6.20%) | IBGE, IPCA 2026-02, released on 2026-03-12 |
| 12 months to 2026-03 | 4.14% | — | IBGE, via OBECON/UFMS |

Sources consulted on 2026-10-28:
- [Informe IPCA jan/2026, Ministério do Planejamento](https://www.gov.br/planejamento/pt-br/centrais-de-conteudo/publicacoes/indicadores-economicos/2026-01-informe-ipca_v2.pdf)
- [IBGE, IPCA fev/2026 (Agência de Notícias)](https://agenciadenoticias.ibge.gov.br/agencia-noticias/46064-com-reajustes-das-mensalidades-escolares-inflacao-acelera-para-0-70-em-fevereiro.html), numbers repeated in [Money Times](https://www.moneytimes.com.br/ipca-sobe-070-em-fevereiro-e-acumula-alta-de-381-acima-das-expectativas-jcav/)
- [OBECON/UFMS, tabela do IPCA mar/2026](https://obecon.ufms.br/files/2026/04/IPCA-Tabela.docx.pdf)
- Series in the app: IBGE SIDRA, table 7060, group 8 (`ipca_g8`), already synced.

By the project's rule (only pin down what two independent sources confirm), these numbers stay
as a **reference in the document**. The app **does not write these values by hand**: it calculates the
difference from the `ipca` and `ipca_g8` series it already downloads from IBGE/BCB, showing the date of the
latest data.

Two important observations:

- **Education adjusts all at once, in February** (Lei 9.870/1999: annual adjustment of
  school tuition). That is why the group jumps in February and stays nearly flat the rest of the year.
  Comparing month to month makes no sense; the comparison is **always over 12 months or more**.
- The Education group mixes primary, secondary and higher education, free courses and materials. IBGE
  also publishes the **subitem "Higher education"** (inside Regular courses), which may rise less
  than school because of competition from distance learning.

### How the app uses inflation

Everything is calculated **in today's reais** (like retirement and the scenarios). That way only the
**difference** between education inflation and the IPCA matters:

```
spread = (1 + inflação_educação) ÷ (1 + IPCA) − 1        (ao ano)
```

- With the 2025 numbers: (1.0622 ÷ 1.0426) − 1 ≈ **1.9% a year above the IPCA**.
- **Proposed default**: geometric mean of the **last 10 complete years** of `ipca_g8` against
  `ipca`, calculated by the app from the downloaded series and shown with the period used
  ("2016-01 to 2025-12, 10 years"). Ten years cross cycles and keep one atypical year from
  dominating the calculation.
- The user can switch to: the Education group, the Higher education subitem (if Q2 is approved), plain IPCA
  (spread 0) or a typed fixed spread.
- The projection never uses Focus for education (Focus only projects the headline IPCA).

## The calculation

### Amount needed at the start of the course

For each payment month `k` of the course (0 … duration × 12 − 1):

```
pagamento_k = mensalidade_hoje × (1 + spread)^(anos até o ano letivo de k) × (1 − desconto)
            + extras e custo de vida do mesmo jeito
alvo_no_início = Σ pagamento_k ÷ (1 + r_mensal)^k
```

- `r` is the **real return** (above the IPCA) the family assumes for the saved money. The
  money stays invested during the course, which is why the target is the **present value** of the
  payments, not the sum.
- The adjustment of each school year comes in as a step (February), not month by month.

### Monthly contribution

```
aporte_hoje = (alvo_no_início − já_guardado × (1 + r_m)^n) × r_m ÷ ((1 + r_m)^n − 1)
```

`n` = months until the start. The result is **in today's reais**: in practice, the contribution **rises with the
IPCA once a year**. The screen shows both: "R$ 1,480 per month today; in 2027, about
R$ 1,540 (adjusted by the IPCA)".

Alternative offered: a **fixed nominal contribution** (same amount in reais for the whole term). It is easier
to understand, but starts higher.

### Example (invented data)

Ana, 8 years old; starts in 2036-02 (112 months from 2026-11); private, 5 years, today's tuition
R$ 2,500, 12 installments, extras R$ 3,000/year. Assumptions: IPCA 4%, education 6% (spread
1.92%), assumed real return 3% a year.

| | Value |
|---|---|
| Tuition in 2036-02, in today's reais | R$ 2,986 |
| The same, in reais of the time (with IPCA 4%) | ≈ R$ 4,307 |
| Sum of all payments, today's reais | R$ 204,832 |
| Amount needed at the start, today's reais | **R$ 190,670** |
| Monthly contribution, today's reais (rises with the IPCA) | **R$ 1,480** |
| With R$ 20,000 already saved | R$ 1,276 |
| Fixed nominal contribution (no adjustment) | R$ 1,756 |
| If it used only the IPCA (spread 0) | R$ 1,194 |

The last row is the argument of the proposal: **ignoring education inflation underestimates the contribution by
about 20%** in this example.

Pedro, 3 years old, same course, starts in 2041-02: target R$ 209,722 and contribution R$ 980 (more time,
but higher tuition because of the accumulated spread).

### Payment phase

When the course starts, the goal changes from **accumulating** to **paying**:

- Transactions in the category Education › child's college (or linked to the goal) come out of the saved
  balance.
- The screen shows "the saved money covers up to the 3rd year" and what is missing becomes a contribution during the
  course.
- At the end of the course the goal is archived automatically (with a notice), and the leftover is freed.

## Annual recalculation

- **When**: after the February IPCA comes out (IBGE releases it around March 10), which already
  includes the tuition adjustment. Proposal: **April**, in a background task, and also on every
  edit of the plan.
- **What it recalculates**: 10-year spread, projected tuition, amount already saved (real, not the
  planned one), needed contribution.
- **Asks for the real tuition** of the year, if the family knows it ("How much does the course you
  want cost today?"). If they do not answer, it follows the index.
- **History**: each recalculation becomes a record (assumptions and result), to show "in 2027
  the contribution was R$ 1,480; in 2028, R$ 1,530, because education rose 7.1%".
- **Notice** only when the needed contribution changes **more than 10%** or the goal gets off track (alert
  rule: one per episode, with a link to the goal).

## Linking investments to the goal

Today the goal only adds up the **cash** of the linked accounts. For college that does not work: the family saves
in Tesouro, CDB, funds, stocks.

Proposal: **link positions**, not only accounts.

- Link options: a whole account (cash + positions at market) **or** an asset inside an account
  **or** a fraction (%) of an asset or account (e.g. 50% of the Tesouro IPCA+ 2035 for Ana and 50%
  for Pedro).
- The value used is the **market value** of the portfolio (`position_snapshot`), with exchange rates for
  another currency.
- The sum of the fractions of the same asset across goals does not exceed 100% (validation).
- The app **shows** the maturity of the bond next to the goal date ("matures in 2035, college starts
  in 2036"), as a fact. It does not suggest a bond, a term or a product.
- On the goal screen, the list "What is saved for Ana" shows each position and what it
  is worth today.

## Several children

- **One goal per child.** Each has its own term, course and contribution. That is what is clear on the screen and what
  allows archiving one when the course ends.
- **"Children's college" view** at the top: sum of contributions per month, timeline with the overlapping
  courses (the years when two are paying at the same time are highlighted) and the total saved.
- **Shared pot (optional)**: an account or asset linked to several goals with no defined fraction. The app
  splits it by the chosen rule: **whoever starts first receives first** (proposed default, because
  their term is the shortest) or proportional to the amount needed.
- **Child yet to be born**: the goal accepts an expected date in place of a member, and becomes
  the member's goal once they are registered.

## Screens

1. **Budget › Goals › New goal › College** (3-step wizard):
   - *Who*: choose the child (shows the age and the year they turn 18).
   - *Which college*: type, duration, today's tuition, extras, living cost.
   - *Assumptions*: education inflation (with the source and the period), assumed real return, adjusted
     or fixed contribution. Fixed text: "These are hypotheses for planning, not a promise of returns nor an
     investment suggestion."
   - Result: amount at the start, contribution per month, and the "IPCA only" comparison.
2. **Goal card** (goal list): child, years until it starts, progress, contribution today, whether it is on
   track, and the badge "education inflation +1.9% a year".
3. **Goal detail**: chart with the needed path and what is actually saved; course band
   at the end; year-by-year table (projected tuition in today's reais and of the time); history of
   recalculations; list of what is saved.
4. **Family › Life timeline**: the "College" band appears on the line of each child with a goal, and the
   parents' line shows their age in those years (already near retirement?).
5. **Scenarios**: the button "Use college goals" puts the contributions as a monthly event and the
   payments as an outflow in the course period.
6. **Balance forecast**: already deducts what is still missing to save for the goals; starts using the adjusted
   contribution.

Strings in pt-BR, en and es in the Fluent catalogs (`locales/<lang>/metas.ftl` or the goals
file that already exists).

## MCP and agent

Read (always on):

- `list_goals`: now brings, for education goals, child, start, type, spread used and contribution
  in today's reais.
- `get_education_plan {goal_id}`: full plan, year-by-year table, linked positions and history.
- `simulate_education_plan {...}`: calculates without saving (the agent answers "what if it is public?" or
  "what if it starts at 19?").
- `get_education_inflation {country?, years?}`: calculated spread, series and period, with the date of the
  latest data.

Write (they do not exist yet; when they do, they stay off by default and become a proposal with
diff, undo and audit, like the others):

- `create_education_goal`, `update_education_goal` (includes entering the real tuition of the year).
- `link_goal_holdings {goal_id, holdings:[{account_id, asset_id?, share?}]}`.

The agent can explain the calculation and the assumptions, but the tool descriptions forbid suggesting a
product, bond or allocation (same rule as the market explorer).

## Data model

One migration (number to be set by the coordinator; today the last on main is V145):

```sql
-- Plano de educação ligado a uma meta `education` (docs/design/college-goal.md).
CREATE TABLE education_plan (
    goal_id              TEXT PRIMARY KEY REFERENCES goal (id) ON DELETE CASCADE,
    member_id            TEXT REFERENCES member (id),       -- NULL: filho que ainda vai nascer
    expected_birth       TEXT,                              -- AAAA-MM-DD, só sem membro
    start_age            INTEGER NOT NULL DEFAULT 18,
    start_month          INTEGER NOT NULL DEFAULT 2 CHECK (start_month BETWEEN 1 AND 12),
    start_date           TEXT,                              -- sobrepõe idade + mês, se informado
    institution_type     TEXT NOT NULL CHECK (institution_type IN ('private', 'public', 'abroad')),
    course               TEXT,
    duration_months      INTEGER NOT NULL CHECK (duration_months > 0),
    payments_per_year    INTEGER NOT NULL DEFAULT 12 CHECK (payments_per_year IN (1, 2, 4, 12)),
    fee                  INTEGER NOT NULL DEFAULT 0,        -- por parcela, moeda da meta, preço de hoje
    fee_date             TEXT NOT NULL,                     -- data da cotação da mensalidade
    yearly_extras        INTEGER NOT NULL DEFAULT 0,
    monthly_living       INTEGER NOT NULL DEFAULT 0,
    discount_pct         REAL NOT NULL DEFAULT 0 CHECK (discount_pct BETWEEN 0 AND 100),
    inflation_mode       TEXT NOT NULL DEFAULT 'series_10y'
                         CHECK (inflation_mode IN ('series_10y', 'headline', 'fixed_spread')),
    inflation_series     TEXT REFERENCES index_series (id), -- padrão do país (ipca_g8 no BR)
    fixed_spread         REAL,                              -- fração a.a., só em fixed_spread
    real_return          REAL NOT NULL DEFAULT 0.03,        -- premissa do usuário (ver Q5)
    contribution_mode    TEXT NOT NULL DEFAULT 'indexed' CHECK (contribution_mode IN ('indexed', 'flat')),
    pool_rule            TEXT NOT NULL DEFAULT 'earliest_first'
                         CHECK (pool_rule IN ('earliest_first', 'proportional')),
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL,
    CHECK (member_id IS NOT NULL OR expected_birth IS NOT NULL)
) STRICT;

-- Posições ligadas a uma meta (qualquer tipo de meta, não só educação).
CREATE TABLE goal_holding (
    id          TEXT PRIMARY KEY,
    goal_id     TEXT NOT NULL REFERENCES goal (id) ON DELETE CASCADE,
    account_id  TEXT NOT NULL REFERENCES account (id),
    asset_id    TEXT REFERENCES asset (id),          -- NULL: a conta inteira (caixa + posições)
    share       REAL CHECK (share > 0 AND share <= 1), -- NULL: pote comum, dividido pela regra
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE (goal_id, account_id, asset_id)
) STRICT;

-- Histórico dos recálculos anuais (e de cada edição).
CREATE TABLE education_plan_review (
    id               TEXT PRIMARY KEY,
    goal_id          TEXT NOT NULL REFERENCES goal (id) ON DELETE CASCADE,
    reviewed_at      TEXT NOT NULL,
    reason           TEXT NOT NULL CHECK (reason IN ('annual', 'edit', 'fee_update', 'created')),
    spread           REAL NOT NULL,
    spread_from      TEXT, spread_to TEXT,           -- período das séries
    fee              INTEGER NOT NULL,
    target_at_start  INTEGER NOT NULL,               -- reais de hoje
    saved            INTEGER NOT NULL,
    monthly_needed   INTEGER NOT NULL,
    assumptions      TEXT NOT NULL                   -- JSON com o resto
) STRICT;

-- Série de inflação da educação por país (papel novo em country_series).
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BR', 'education_inflation', 0, 'ipca_g8');
```

Notes:

- `goal.target_amount` and `goal.target_date` still exist and are **synchronized by the plan**
  (like `retirement::sync_linked_goal`), so the list, the balance forecast and the agent
  do not need to know about education.
- `goal_account` still applies (cash). `goal_holding` replaces `goal_account` over time;
  migrating the old links is optional and was not done.
- `goal_holding` serves any goal, which also improves the retirement goal and the emergency fund
  goal when they are in investments.
- The `education_inflation` role requires widening the CHECK of `country_series.role` (today
  `inflation`, `rate`, `fx`, V84), which in SQLite means recreating the table in the migration.

## Other countries

| Country | Education series | Note |
|---|---|---|
| Brazil | IPCA · Education (IBGE SIDRA 7060, `ipca_g8`) | already downloaded |
| US | CPI "College tuition and fixed fees" (BLS, series CUUR0000SEEB01; also on FRED) | it is list price, only part of the quotes discounts scholarships ([BLS](https://www.bls.gov/cpi/factsheets/college-tuition.htm), page of 2026-05-20). Annual or semester payment, start in September. |
| Eurozone and EU countries | HICP · Education (Eurostat, COICOP 10) | new series per country, same HICP source the app already uses |
| Portugal | HICP · Education PT | tuition fees (propinas) at public institutions have a legal cap (value to confirm with two sources before use) |
| Others | no education series | uses the country's general inflation with spread 0 and says so on the screen; the user can type a spread |

- Currency: the goal's (default, the family's base). A child going to study abroad: goal in the currency of the
  country of the course (e.g. EUR), with the exchange rate the app already has; the contribution appears in both currencies.
  In that case the calculation uses the inflation of the country of the course and the exchange risk appears as a warning, without an
  invented number.
- School calendar by country: default start month (Feb in BR, Sep in the northern hemisphere, Mar in
  Argentina and Chile, to be confirmed).

## Limitations

- School (primary and secondary education). The same plan works with a lower `start_age`; it could be
  phase 2 without changing the model.
- Comparing investment products, suggesting a bond or term: **never** (CVM).
- Estimating scholarships, FIES or ProUni.
- Income tax deduction for education (annual limit per person in the IRPF): the taxes module already handles it;
  the goal only shows a link.

## How it is implemented (Brazil)

- **Migration V148** (`V148__meta_de_faculdade.sql`): `education_plan`, `goal_holding`,
  `education_plan_review` and `country_series` recreated with the `education_inflation` role (Brazil:
  `ipca_g8`). The `goal_holding` fraction goes from 1% to 100% (`CHECK share BETWEEN 0.01 AND 1`); a deleted account
  or asset takes the link with it.
- **Core** (`goals/education.rs`, `goals/holdings.rs`): the calculation of sections 5.1 to 5.4, with the
  step adjustment in the start month of the school year; education spread from the geometric mean of the
  last 10 complete and consecutive years of the two series (without complete years, spread 0 and a warning);
  assumed general inflation of 4% only when the series is missing. The living cost does not rise with education. The
  `education` goal receives target, date and contribution on save and, in the list, the calculation is redone with
  today's saved amount (`GoalView.education`). Linked positions apply to any goal
  (`GoalView.holdings_value`); the shared pot splits what is left after the explicit fractions by the rule of the
  plan of the goal that starts first.
- **Annual recalculation**: in the reminders (`notify`), from April, once a year per plan;
  it writes the history and warns only if the contribution changes more than 10% or if the course has ended (alerts
  `education_review` and `education_finished`, by the budget preference). Editing the plan writes
  `edit` or `fee_update` (only the tuition changed).
- **Q8 (payments during the course)**: by the simplest and most predictable recommendation, the payments
  come out of what is actually saved: linked accounts and positions and transactions linked to the goal.
  The app does not deduct anything on its own by category.
- **API** (`goals.*`): `education_get`, `education_simulate` (fields on top of the goal's plan or of the
  default), `education_inflation`, `education_overview`, `education_save`, `holdings_get` and
  `holdings_set`.
- **Read MCP**: `get_education_plan`, `simulate_education_plan` and `get_education_inflation`;
  `list_goals` brings the plan summary. The descriptions forbid suggesting a product, bond, term or
  allocation.
- **Screens** (Budget › Goals): the "College goal" button (and the "Education" type in a new goal)
  opens the 3-step wizard with the result at each step; the card shows the child, the term and the
  education inflation badge; "View plan" opens the detail (year by year, what is saved with the
  linked positions, recalculations); "Positions" links accounts and assets to any goal; the
  "Children's college" panel adds up the plans and marks the years with two courses.
- **Left for phase 2**: US and EU (new series), write MCP, scenarios, life timeline and school.
