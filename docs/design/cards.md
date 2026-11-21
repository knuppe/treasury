# Credit cards

Everything Treasury does with credit cards beyond the card statement: additional and virtual cards
with each one's member, limit and the real statement cycle, costs (charges, annual fee, revolving
credit, purchases abroad), cashback, points and miles, and the question "is this card worth it?".
The sections numbered by phase describe each part; the "Implementation" sections say what stayed in
the code (migrations V155 to V159). Numbers and example names are made up.

## Additional and virtual cards

### Model decision

The additional card is **not another account**: the card statement is one, the limit is one and the
holder is the one responsible for the debt. So the additional card becomes a **plastic** (a physical
or virtual card) inside the card's account.

```
conta "Cartão Azul" (titular: Ana)
 ├── plástico titular   final 1111  → Ana
 ├── plástico adicional final 2222  → Bruno
 ├── plástico adicional final 3333  → Clara (filha), sublimite R$ 500
 └── cartão virtual     final 4444  → Ana, "assinaturas"
```

Two different questions are kept apart:

- **Who spent** is the plastic's member: it goes to the transaction and to the per-member reports.
- **Who owes** is still the account's holder: the card debt in each member's net worth does not
  change.

### Member rule

Order of precedence on the card transaction:

1. Member chosen by hand (`member_origin = 'manual'`): never changes.
2. The plastic's member, when the transaction has a plastic (`member_origin = 'card'`, new value).
3. The account's single holder (`'auto'`, as today).
4. Family.

Changing a plastic's member redoes its `card` transactions, the same way `reinherit_members` does
today with the account. The mock (`ledger/domain.ts`) repeats the rule.

### How the transaction gets the plastic

- **Typed**: "Card (last digits)" field on the transaction, only on a card account with more than one plastic.
- **Row import and MCP** (`import_transactions`): `card_last4` column, per row or per batch, like
  `statement_due_date`.
- **Card statement PDF**: Brazilian card statements separate by blocks ("Cartão final 2222", the
  cardholder's name, subtotal). The generic layout starts keeping the block's last digits on each row.
- **OFX**: usually does not bring the last digits; it stays without a plastic (falls to rule 3).
- **Unknown last digits**: the preview shows "last digits 5555 not registered" and offers to create
  the plastic, choosing the member. It never creates a member by itself.

Only the **last 4 digits** are stored. The app refuses a number longer than that in the field, so it
never stores the full card number.

### Screens

- **Accounts › card**: "Cards" section with the plastics (last digits, type, member, sublimit, active).
- **Card statement**: group by last digits, with a subtotal per person, as the bank prints it. This
  also helps check the total.
- **Family**: for each member, their plastics and the month's spending on each.
- **Additional card sublimit**: the bank sometimes allows it, and the family uses it as an
  "allowance on the card". Here it is **informational**: it alerts when the plastic's spending in
  the cycle passes 80% and 100%.
- **Virtual card**: the subscriptions (`subscriptions`) show which last digits they charge.
  Deactivating a virtual card warns which subscriptions were on it.

### Data model (one migration)

```sql
CREATE TABLE card_plastic (
    id          TEXT PRIMARY KEY,
    account_id  TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    kind        TEXT NOT NULL CHECK (kind IN ('holder', 'additional', 'virtual', 'temporary')),
    last4       TEXT CHECK (last4 IS NULL OR (length(last4) = 4 AND last4 GLOB '[0-9][0-9][0-9][0-9]')),
    member_id   TEXT REFERENCES member (id),
    label       TEXT,
    spend_limit INTEGER CHECK (spend_limit IS NULL OR spend_limit >= 0),   -- sublimite informativo
    active      INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE (account_id, last4)
) STRICT;
ALTER TABLE txn ADD COLUMN card_plastic_id TEXT REFERENCES card_plastic (id);
```

`member_origin` gets the value `'card'` (the V122 CHECK must be redone, which in SQLite requires
recreating the column; an alternative without recreating: store `'auto'` and decide by the presence
of `card_plastic_id`). The migration creates a `holder` plastic for each card that already has a
`last4`, without touching any transaction.

### MCP and agent

- `list_accounts` and `list_card_statements` start bringing the plastics and the subtotal per last digits.
- `import_transactions` accepts `card_last4`.
- New proposal `create_card_plastic` / `update_card_plastic` (goes to the approval queue).
  It does not exist yet: today the MCP only reads the plastics, with `list_card_plastics`.
- `spending_summary` already accepts a member; it starts applying to the additional card with no change.

## Limit, statement cycle and annual fee

The annual fee is part of this phase; its design is in "Card costs".

### Limit

- **Limit usage** on the card and on the dashboard: `committed ÷ limit`, with the future-installments
  part separated ("R$ 3,000 of installments already take 20% of the limit").
- **Limit history** (`card_limit_change`: date, limit, manual or import origin), to see the
  evolution and calculate the usage of past months.
- **Shared limit** (optional): two cards from the same bank that share the limit point to a group;
  the available amount belongs to the group.
- Configurable usage **alerts** (default 80% and 95%), through the same `notify` mechanism.
- **Financial health**: the average limit usage enters as a factor (constantly above ~30% is a sign
  of strain). In the US and Canada this weighs on the *credit score*, so the text changes by country.

### Real statement dates

Today the dates are always calculated. In practice the bank postpones the closing for weekends or
holidays and sometimes changes the due date for a single month.

- The card statement gets `dates_source` (`calc`, `manual`, `import`). `manual` or `import` dates are
  not redone by `recompute_statements`.
- PDF import and `statement_due_date` store the printed due date as `import`.
- Edit the closing and due dates of a card statement on the statement screen.

### What the PDF already shows and is discarded today

The PDF reader already recognizes "limit" and "minimum payment" in order to skip those lines. It
starts storing them in the card statement: `informed_minimum`, `informed_limit` (becomes a record in
the limit history) and `informed_previous_balance`, to check the balance that came from the previous
card statement.

### Best day to buy

The day after the closing, with the time until the due date ("buy from 11-26 and pay on 01-05, 40
days"). It appears on the card and the agent knows how to answer "which card do I use today?".

### Credit balance

A payment above the total (or a refund after payment) leaves credit. Check that the next card
statement deducts it from the balance of the remainder; if not, fix it in `statement_to_pay` and in
the view.

## Card costs

### Charges as a category, not just as text

System categories "Card charges" (spending group **Debts**): revolving interest, installment
interest, IOF (a Brazilian tax on financial transactions), penalty and late interest, annual fee,
insurance and fees. The existing category rules (`rules`) receive the text patterns that `insights`
uses today, so the import classifies by itself.

### Annual fee

The annual fee goes together with the limit (it is the most relevant cost to control). What the app
stores and does:

- **Rule per card** (`card_annual_fee`): full amount, in how many installments it is charged,
  anniversary month, start and end date (rule change = new record, as in cashback).
- **Waiver**: full, by minimum spending in the cycle (e.g., R$ 3,000), by amount invested with the
  bank, or partial discount (e.g., 50% above R$ 1,500). The card shows "R$ 420 short this cycle to
  waive the December installment".
- **Recognized charge**: transactions with "anuidade" in the description (`insights` already finds
  them by text) receive the "Annual fee" category and are linked to the rule. That way it is
  possible to check: charged when it should not have been (the waiver was reached), charged a
  different amount from the agreed one, or charged an extra installment.
- **Expected**: the future installments enter the cash forecast and the card's commitment.
- **Negotiation reminder**: 30 days before the anniversary, a notice "the R$ 1,200 annual fee of the
  Blue Card renews in December; worth asking for a waiver or switching". It is the moment when the
  family usually gets a discount.
- **Total per year**: how much the family paid in annual fees in 12 months, per card and in total,
  on the card panel and in an insight; and it enters the "is it worth it?" of phase 6.

```sql
CREATE TABLE card_annual_fee (
    id              TEXT PRIMARY KEY,
    account_id      TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    amount          INTEGER NOT NULL CHECK (amount >= 0),   -- valor cheio do ano, em centavos
    installments    INTEGER NOT NULL DEFAULT 12 CHECK (installments BETWEEN 1 AND 12),
    anniversary_month INTEGER CHECK (anniversary_month BETWEEN 1 AND 12),
    waiver_kind     TEXT NOT NULL DEFAULT 'none'
                    CHECK (waiver_kind IN ('none', 'full', 'spend', 'investment', 'discount')),
    waiver_threshold INTEGER,       -- gasto ou valor investido mínimo, em centavos
    waiver_discount  TEXT,          -- decimal: 0.5 = 50% (kind = 'discount')
    starts_on       TEXT NOT NULL,
    ends_on         TEXT,
    notes           TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
) STRICT;
```

MCP: `get_card_overview` brings the annual fee (rule, how much is left to waive, paid in 12 months)
and the proposal `set_card_annual_fee` goes to the approval queue.

### Revolving credit and installment plan of the card statement

- Card statement overdue and partly paid: the card warns that the remainder went into revolving
  credit and shows the charges that came on the next card statement.
- **Splitting the card statement into installments** becomes a contract in the debt module (`debt`),
  with the installments posted on the card. That way `plan_debt_payoff` and `compare_loan_rates`
  already see this debt, which tends to be the family's most expensive.
- Brazilian rules, with source and date, as in the tax data: revolving credit can only last until the
  next card statement (Res. CMN 4.549/2017, a Central Bank council resolution) and interest and
  charges of the revolving credit and of the card statement installment plan cannot exceed 100% of
  the original amount (Lei 14.690/2023). The app uses this to warn when the posted charges seem to
  exceed the cap, never to calculate interest the bank did not report. Check the validity before
  implementing.

### Purchases abroad

A card transaction with the original amount and currency (`orig_amount`, `orig_currency`), the
applied exchange rate and the IOF of the purchase. It shows the real cost of the trip and feeds the
taxes screen (IOF paid). The IOF rate comes from a versioned file with a source, like the tax data,
because it changed several times in 2025; check the current one at implementation.

## Cashback

- **Program per card**: base percentage, multipliers by category or merchant (optional), cap per
  cycle, minimum amount for redemption and destination (credit on the card statement, account or
  investment).
- **Expected**: calculated per card statement on the eligible purchases (without IOF, charges, annual
  fee, payments and refunds).
- **Received**: the credit on the card statement or the deposit in the account, marked as cashback
  (income category "Cashback and rewards").
- **Check**: expected against received per cycle; alerts when a cycle closed and the cashback did not
  come after the program's deadline.
- **Accounting decision**: cashback as **income**, without reducing the expense. That way each
  category's spending stays the price paid, and the card's return appears separately. The
  alternative is to treat it as a refund of the purchase.

```sql
CREATE TABLE card_reward_program (
    id            TEXT PRIMARY KEY,
    account_id    TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    kind          TEXT NOT NULL CHECK (kind IN ('cashback', 'points')),
    loyalty_program_id TEXT REFERENCES loyalty_program (id),   -- fase 5, quando kind = 'points'
    base_rate     TEXT NOT NULL,      -- decimal: 0.01 = 1% (cashback) ou pontos por unidade (points)
    rate_currency TEXT,               -- points: moeda da regra (muitos programas contam por US$)
    cycle_cap     INTEGER,            -- teto por ciclo, em centavos (cashback) ou pontos
    starts_on     TEXT NOT NULL,
    ends_on       TEXT,               -- mudança de regra = novo registro
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;
CREATE TABLE card_reward_multiplier (            -- opcional
    program_id   TEXT NOT NULL REFERENCES card_reward_program (id) ON DELETE CASCADE,
    category_id  TEXT REFERENCES category (id),
    payee        TEXT,
    rate         TEXT NOT NULL
) STRICT;
```

## Points and miles

### Concepts

- **Program**: from the bank (Livelo, Esfera, own programs) or from the airline (Smiles, LATAM Pass,
  Azul Fidelidade and equivalents from other countries). Each program belongs to **a member** (the
  account is per CPF, the Brazilian taxpayer ID), which fits the family.
- **Movement**: accrual from the card, bonus, purchase of points, transfer between programs (with
  transfer bonus), redemption, expiration and adjustment.
- **Expiration**: each accrued batch has an expiration date; the app warns 90, 30 and 7 days before.

### What the app does

- **Expected** accrual per card statement, by the card's rule (phase 4, `kind = 'points'`),
  converting by the rule's currency when it counts in dollars. Compared with the **credited**
  amount the family reports.
- Transfer with bonus: a pair of movements (10,000 out of the bank, 20,000 into the airline).
- Cost per thousand when the family buys points, and the **estimated value** per thousand as a
  manual assumption (the app does not fetch mileage quotes).
- **Net worth**: points stay **out of** net worth by default. They appear on a separate line,
  "estimated value in points", which does not enter the totals.
- Input: manual, through the agent (text or a screenshot of the program's statement, with OCR) and
  through the MCP. No integration with the programs' sites (same reason as D20).

```sql
CREATE TABLE loyalty_program (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    kind            TEXT NOT NULL CHECK (kind IN ('bank', 'airline', 'retail', 'other')),
    member_id       TEXT REFERENCES member (id),
    unit_label      TEXT,              -- "pontos", "milhas"
    value_per_1000  INTEGER,           -- premissa da família, em centavos; NULL = sem valor
    value_currency  TEXT,
    active          INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
) STRICT;
CREATE TABLE loyalty_entry (
    id              TEXT PRIMARY KEY,
    program_id      TEXT NOT NULL REFERENCES loyalty_program (id) ON DELETE CASCADE,
    date            TEXT NOT NULL,
    points          INTEGER NOT NULL,  -- com sinal
    kind            TEXT NOT NULL CHECK (kind IN ('earn', 'bonus', 'purchase', 'transfer_in',
                                                  'transfer_out', 'redeem', 'expire', 'adjust')),
    expires_on      TEXT,
    statement_id    TEXT REFERENCES card_statement (id),
    transfer_id     TEXT,              -- liga saída e entrada de uma transferência
    cost            INTEGER,           -- compra de pontos, em centavos
    cost_currency   TEXT,
    notes           TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
) STRICT;
```

Points are integers, with no cents; never added to money.

## "Is this card worth it?"

Per card, over the last 12 months: annual fee + charges + IOF − cashback received − points credited ×
value per thousand (if the family defined it). It shows the net result and what would change by
concentrating the spending on another of the family's cards. It is an **analysis of the data**, not a
recommendation of a financial product, and the text says so.

Read MCP: `get_card_overview` (plastics, limit usage, cycle, best day, costs and return),
`list_loyalty_programs`, `get_card_value`. Proposals: `record_loyalty_entry`,
`set_card_reward_program`. The agent gets a "card statement arrived" playbook: import, check the
total, minimum and limit, check cashback and points.

## Other countries

The model is the same in all of them (plastic, limit, card statement, cashback, points). What changes:

| Country | Difference |
|---|---|
| Brazil | Revolving credit and card statement installment plan with the CMN rules and Lei 14.690/2023; IOF abroad; interest-free installments are common. |
| US, Canada | No installments (or "pay in 4" outside the card); APR, *grace period*, *statement balance* and limit usage weighing on the *credit score*. |
| Portugal, Spain | Card with a payment mode (100%, percentage or fixed amount); the mode becomes a card field and changes what the cash forecast pays. |
| Others | Generic, with no interest rule of their own. |

The demo vault gets an additional card, a subscriptions virtual card, a cashback and a points
program in each country.

## Out of scope

- Full number, CVV or card expiry: never.
- Open Finance or integration with banks and points programs.
- Card insurance and benefits (VIP lounge, travel insurance): at most a free note.
- Automatic debit of the card statement and payment boleto.

## Implementation: additional and virtual cards (V155)

- **V155** creates `card_plastic` and `txn.card_plastic_id`. The last digits the card already had
  become the holder's plastic, with the account's own id and no member (the account's holder
  applies). No transaction changes. `member_origin` did not get the `card` value: the member coming
  from the plastic is `auto`, and the presence of `card_plastic_id` says where it came from (without
  recreating the column).
- Core: `cards/plastics.rs` (registry, last digits normalized and refused above 4 digits, card
  statement subtotals), `transactions::inherited_member` and `reinherit_one`. Changing the last
  digits in the card's registry carries the holder's plastic along.
- Commands: `cards.list_plastics`, `create_plastic`, `update_plastic`, `delete_plastic`;
  `cards.get_statement` brings `by_plastic`; `TxnInput`/`TxnPatch` accept `card_plastic_id`;
  `TxnView` brings `card_plastic_id` and `card_last4`.
- Import: `card_last4` per row or per batch in `import.rows` (and in the MCP
  `import_transactions`); the card statement PDF reads the last digits of each block. Unknown last
  digits become a warning in the preview; the plastic is looked up again when saving.
- MCP: `list_card_plastics` (read). The agent's `card_bill` guide explains `card_last4`.
- Interface: **Cards** tab on the card account (registry, who uses it, spending on the open card
  statement and a sublimit warning at 80% and 100%), **Card used** field on the transaction (only
  with more than one card registered), **By card** block in the card statement detail and the last
  digits next to each purchase.
- Not done: MCP write proposals for the plastics (`create_card_plastic`), each member's spending per
  card on the Family screen and the warning about subscriptions on a deactivated virtual card.

## Implementation: limit, cycle and annual fee

- **V156**: `card_statement` gets `dates_source` (`calc`, `manual`, `import`),
  `informed_minimum`, `informed_limit` and `informed_previous_balance`; tables
  `card_limit_change` (limit history; each card's current limit becomes the first record, at the
  account's opening, with the account's id) and `card_annual_fee`.
- **Limit** (`cards/limits.rs`): `cards.overview` brings usage (outstanding balance with future
  installments), future installments, available, usage in base points, best day to buy (the day
  after the closing of the open card statement, with the due date and the days to pay), the open
  card statement, the limit history and the annual fee. Changing the limit in the registry records it
  in the history; `cards.add_limit_change` records a change with a date; the most recent record
  becomes the limit in force. The limit printed on a PDF card statement enters as `import`.
- **Real statement dates**: `cards.update_statement` changes the closing and due dates of a card
  statement (they become `manual` and are not redone when the card's cycle changes; the next card
  statement starts the day after the new closing), goes back to the calculated ones (`reset_dates`)
  and stores the informed minimum, limit and previous balance. The PDF card statement stores the
  printed due date (`import`) and these values, which the reader now recognizes.
- **Annual fee** (`cards/annual_fee.rs`): rule per card with the year's amount, installments,
  renewal month and waiver (none, full, by spending on the card statement, by balance at the
  institution, discount). Charge = transaction with "anuidade" in the description. Check of the
  12-month charges: charged on a card statement that reached the waiver, an amount different from
  the installment (the first may carry the cent from the division) or an installment beyond the
  agreed number. Waiver by spending: the installment charged on a card statement is waived when the
  purchases on that same card statement reach the amount.
- **Warnings** (reminders, tied to the accounts and card statements): limit usage from 80% and 95%
  (one per band and card statement), annual fee renewing in 30 days (one per renewal) and an annual
  fee charge that does not match (one per charge).
- **MCP**: `get_card_overview` (read) and `set_card_annual_fee` (proposal).
- **Interface**: **Limit and annual fee** tab on the card account (usage bar with the future
  installments part, best day to buy, limit history, annual fee rule, how much is left for the
  waiver, charges with the problem pointed out) and **Adjust dates and amounts** in the card
  statement detail.
- Not done: shared limit between cards, limit usage in the financial health and the credit balance
  check.

## Implementation: costs

- **V157**: `txn.orig_amount` and `txn.orig_currency` (purchase abroad; the amount carries the
  transaction's sign). The applied exchange rate is `amount ÷ orig_amount`; the IOF stays as a line
  of its own on the card statement, in the IOF category.
- **Purchase abroad**: `TxnInput`/`TxnPatch` accept `orig_amount` (positive) and `orig_currency`;
  the two go together and the currency cannot be the account's. `import.rows` and the MCP
  `import_transactions` accept both per row; the card statement PDF reads "USD 80,00", "US$ 12,50",
  "EUR 1.200,00" in the description. In the interface, the expense transaction on the card gained
  "Purchase currency" and "Amount in the purchase currency", and the card statement shows the
  original amount next to the purchase.
- **Charges**: a new ready-made rule (version 4, Brazil only) "Juros e encargos (rotativo, multa,
  mora)" puts interest, charges, revolving credit, late penalty and late interest in Taxes and fees
  › Interest and penalties (Debts group). The annual fee already went to Bank fees and the IOF to
  IOF. Existing vaults receive the new rule once, undoable through the history.
- **Revolving credit** (`cards/revolving.rs`): overdue card statement with a balance to pay, only on
  cards with some card statement payment recorded (with none, it is missing data). It joins the
  consecutive overdue card statements with a balance, adds the charges posted afterward (interest,
  charges, revolving, penalty, late interest, IOF, without the IOF of international purchases) and
  compares with what was left unpaid. In Brazil it shows the rules: revolving credit goes only until
  the next card statement (Res. CMN 4.549/2017) and charges do not exceed 100% of the original debt
  (Lei 14.690/2023, in force since January 2024). It enters `cards.overview` (`revolving`,
  `brazil_rules`), the Limit and annual fee tab and the reminders `card_revolving` (one per card
  statement) and `card_revolving_cap` (charges above the cap, Brazil only).
- **Not done**: "splitting the card statement into installments" becoming a contract in the debt
  module. The bank charges the installment plan as lines on the card statement, which are already
  imported; creating the contract without counting the debt twice requires a decision on how to
  link the installments to the contract, and stays for a round of its own.

## Implementation: cashback

- **V158**: `card_reward_program` (cashback or points, base rate, the rule's currency for points, cap
  per card statement, cashback destination, credit deadline in days, validity) and
  `card_reward_multiplier` (rate per category). New income category **Reimbursements › Cashback and
  rewards** in the initial set (old vaults receive it as a suggestion in Settings › Categories, like
  the other new ones).
- **Expected** (`cards/rewards.rs`): per card statement, purchases minus refunds, without annual fee,
  charges and cashback credits, times the category's rate (or its parent's) or the base one, with
  the cap. Points: per unit of the rule's currency (converted by the closing exchange rate when it is
  another, e.g., points per dollar).
- **Received** (cashback only): credits with the Cashback and rewards category or "cashback" in the
  description, on the card or on the destination account; each credit counts for the last card
  statement that closed before it. Cashback is income, it does not reduce the expense.
- **Warning** `card_cashback_missing` (one per card statement): the deadline passed (closing + days)
  and less than half of the expected amount came.
- **MCP**: `get_card_rewards` (read) and `set_card_reward_program` (proposal).
- **Interface**: **Cashback and points** tab on the card account (rule, rates per category, expected
  and received over 12 months and per card statement, what did not come).
- The points actually credited stay with the points programs ("Points and miles").

## Implementation: points and miles

- **V159**: `loyalty_program` (name, type bank/airline/retail/other, member, value per thousand as an
  assumption, active) and `loyalty_entry` (accrual, bonus, purchase with cost, transfer in and out
  with the same `transfer_id`, redemption, expiration, adjustment; expiration of the entries; the
  card statement that generated the accrual). `card_reward_program.loyalty_program_id` links the
  card's points program to the program where the points land.
- **Balance** (`cards/loyalty.rs`): sum of the movements minus the points that expired unused.
  Outflows consume first the batches that expire earlier (those with no expiration last); from this
  come the points that expire in 90 days and the next expiration. Estimated value = balance × value
  per thousand, outside net worth. Average cost per thousand of the points purchases.
- **Expected × credited**: in the Cashback and points tab, a points card linked to a program compares
  the expected points of each card statement with the accruals linked to it; the
  `card_cashback_missing` warning also applies to points, with its own text.
- **Warnings** `loyalty_expiring`: one per program, expiration date and band (90, 30 and 7 days).
- **MCP**: `list_loyalty_programs` (read) and `record_loyalty_entry` (proposal; transfers stay in the
  interface).
- **Interface**: **Points and miles** panel at the end of the Accounts screen (programs, balance,
  next expiration, estimated value, cards that accrue, movements when opening the program; record
  points, transfer with bonus, new program) and the points program in the card form.
- Agent prompt: the tool catalog has a limit of 16,000 characters and lists all the tools; the area
  descriptions were shortened to fit those of the cards.

## Implementation: "is it worth it?"

- No migration (the reserved V160 stayed free).
- **Cost × return** (`cards/value.rs`, `cards.value`): per non-archived card, over the last 12
  months, purchases (minus refunds), annual fee, interest and charges, IOF, cashback received (on the
  card or on the destination account) and points credited × value per thousand; `net` = return −
  costs.
- **Simulation**: for each card with a cashback or points program, the return by its rules (rate per
  category, cap per card statement × 12) on the purchases of all the family's cards in the base
  currency, minus the annual fee with that concentration (waiver by spending if the monthly average
  gets there). Points without a value per thousand appear only as points. The per-dollar conversion
  of the points rules does not enter the simulation (only the expected amount of each card
  statement).
- **MCP** `get_card_value` (read); the agent's `card_bill` guide now checks total, minimum, limit,
  cashback and points after importing the card statement.
- **Interface**: **Is this card worth it?** block in the Cashback and points tab, with the notice that
  it is analysis and not a product recommendation.

## Limitations

- Splitting the card statement into installments becoming a contract in Debts.
- Shared limit between cards, limit usage in the financial health, check of the credit balance.
- MCP write proposals for the plastics, spending per card on the Family screen, subscription warning
  on a deactivated virtual card.
- Demo vault with an additional card, annual fee, cashback and points in each country.
- Card benefits (insurance, VIP lounges).
