# To solve, next step and closing the month

"To solve" is the single list of what needs the person: due dates, accounts without a recent
statement, tasks, alerts and attention insights, from most serious to least serious, with the line
that says until when the numbers are valid. Next to it, "Your next step" says what to do first, and
"Close the month" is a five-step script for the month that just ended. They are on the Dashboard and
in the bell in the top bar.

## The list

The app has three sources of warnings: the **alerts** (`ai::alerts`, fed by the reminders from
`notify`), the **insights** (`insights`) and the **suggestions** (`ai::suggestions`). A core module,
`attention`, **reads and composes** these sources into a single list, so the person does not need to
open the Agent screen to see a bill coming due. No money rule lives in it.

| Origin (`source`) | Where it comes from | Note |
|---|---|---|
| `due` | `reports::upcoming`: expected transactions and card statements to pay within the lead time set in Settings › Notifications, plus expected transactions overdue by up to 30 days | It is the state of now: a paid bill leaves the list without anyone resolving an alert. An overdue, unpaid card statement stays with the revolving-credit alert |
| `data` | `accounts::freshness` (everyday accounts possibly incomplete) and the file inbox | The same rule as the weekly reminder and the financial health (`everyday_incomplete`) |
| `todo` | `ai::suggestions`: first setup, no category, budget exceeded, asset without a quote, B3 import, liquidity outside the emergency fund, proposals to approve | The suggestions already covered by the two lines above are left out |
| `alert` | New or seen alerts from the last 30 days | The types computed on the spot are left out (`LIVE_ALERT_KINDS`: `due_soon`, `budget_over`, `stale_accounts`), so the same fact does not appear twice |
| `insight` | Attention insights | The informational ones, the good news and the personal inflation stay in "What Treasury noticed" |

The order is the severity (`critical`, `warning`, `info`) and, within it, the order of the table.

The category rules ready to be created ([learned-rules.md](learned-rules.md)) also enter as
`todo`, and, only for those who registered some purchase in Consumption in the last 90 days, the
month's grocery purchases without items (from two on; the link opens "Itemize" for the most recent
one; once dismissed, it only returns the following month; [consumption.md](consumption.md)).

### Dismiss

- An **alert** becomes solved in the alerts inbox (the button says "Mark as solved").
- The rest is postponed in the `attention.dismissed` setting, **inside the vault**, so it holds on
  any computer. Due dates and insights carry the date or the month in the key and do not come back.
  Data and tasks come back in 7 days if they are still valid: dismissing "12 transactions without a
  category" does not hide the problem forever.
- Only the interface dismisses. MCP clients and routines receive `forbidden`.

### Until when the numbers are valid

`attention.list` also returns `coverage`:

| `status` | When | What the interface shows |
|---|---|---|
| `empty` | no everyday account | nothing |
| `complete` | all everyday accounts up to date | "Accounts up to date: data until DD/MM" (the oldest news among them) |
| `partial` | some possibly incomplete | amber warning with the names and "Import statement" |

The line appears at the top of the Dashboard at every level. The `partial` warning also appears in
the balance forecast and in "how much can I spend today", which are the numbers that mislead most
when a statement is missing. The financial health and the goals already had their own warning
([account-freshness.md](account-freshness.md)).

## Where it appears

- **Bell in the top bar**, with the count. It turns amber when something is urgent or needs
  attention. Clicking opens the Dashboard at `?resolver=1`, with the whole list. There is no floating
  panel: the list has one place only.
- **"To solve" card** at the top of the Dashboard, at every level and also in a member's view. It
  shows the three most serious items and "See all N", so it does not push the rest of the Dashboard
  down.
- **Agent › Alerts** stays the same and is where the full text of the alerts lives.
- **MCP**: `get_attention` (`attention.list`), read only.

Commands: `attention.list`, `attention.coverage` (read) and `attention.dismiss` (interface).

### What the list does not do

- **It does not replace the three sources.** `ai::alerts`, `insights` and `ai::suggestions` stay
  underneath; the list is a reading of them.
- **The "dismiss" on the insights card** stays on the computer (`localStorage`), because it only
  holds informational items.
- **The mock** (`apps/desktop/src/api/mock/attention.ts`) only composes the other mocks and
  recalculates nothing. Its due dates come from the mock's `due_soon` alerts, not from the expected
  transactions.
- **System notifications** come from the alerts, not from the list.

## Your next step: the steps

The financial health score says how the family is doing. The step says **what to do first**, with a
single action. There are six, always in the same order, and the current one is the first not yet
met (`attention::ladder`, command `attention.next_step`, MCP tool `get_next_step`):

| # | Step (`key`) | Met when | If not, the action leads to |
|---|---|---|---|
| 1 | Know where the money goes (`know`) | there is a registered account, 3 complete months of transactions and 80% of the transactions in the last 90 days have a category | Accounts, import or the uncategorized transactions |
| 2 | Spend less than you earn (`balance`) | over the sum of the last 3 complete months, expenses do not exceed income (and there is income recorded) | Budget |
| 3 | Get out of expensive credit (`credit`) | no revolving card credit or overdraft in the last 3 complete months and the current one | Debts |
| 4 | Build the emergency fund (`reserve`) | the fund covers 3 months of essential spending (with no essential category marked, 3 months of average spending); the intermediate milestone is 1 month | Budget › Emergency fund |
| 5 | Make debts light (`debts`) | installments of loans and financing take up to 30% of the window's income | Debts |
| 6 | Invest and protect what you have (`invest`) | it is the top, never met | Goals (without investments) or Portfolio › Allocation and rebalancing |

Rules that apply to all:

- **The facts are those of the financial health** (`health::facts::Facts`). No calculation is
  redone: monthly flow, revolving credit, overdraft, emergency fund and installments come from there.
- **No data, no penalty.** What cannot be measured (no checking account, no contracts) counts as
  met. The lack of transactions is step 1 itself.
- **No money amounts in the texts**, as in the financial health: only months and percentages.
- **No product recommendation.** Step 6 points to the allocation, the concentration and the cost of
  fees, which are analyses of what the person already has.
- With an everyday account without a recent statement, the answer comes out with `estimated` and the
  card warns that the reading is provisional.

The limits (3 months, 80%, 3 months of emergency fund, 30% of income) are constants in
`attention/ladder.rs`. They are usual financial education references, chosen so that the rule can be
explained in one sentence.

## Close the month in five steps

A short script for the month that has just ended (`attention::closing`, commands
`attention.month_close` and `attention.month_close_set`, MCP tool `get_month_close`). The card
appears on the Dashboard until the month is marked as closed and disappears when there are no
transactions in the month.

| # | Step (`key`) | How it is done |
|---|---|---|
| 1 | Bring in the statements (`statements`) | measured: no everyday account possibly incomplete |
| 2 | Categorize the transactions (`categories`) | measured: no income or expense transaction of the month without a category |
| 3 | Check the balances (`balances`) | the person marks it |
| 4 | Solve what is pending (`review`) | measured: no due date, data or task open in "To solve" (besides those of steps 1 and 2) |
| 5 | Read the month's summary (`summary`) | the person marks it; the shortcut opens the month's Reports |

What the person marks and the closing date are kept in the `attention.month_close` setting, inside
the vault, for the last 24 months. A month can only be closed with the five steps done (the command
refuses before that), and only a month that has already ended. A closed month does not appear again,
even if a new transaction arrives later; it can be reopened through the command (`step: closed`,
`done: false`). Only the interface marks and closes.

## In the code

- `crates/treasury-core/src/attention/` (`mod.rs`: the list and the coverage; `ladder.rs`: the
  steps; `closing.rs`: the month closing); catalog `locales/*/atencao.ftl`; interface in
  `apps/desktop/src/features/attention/`.
- MCP: `get_attention`, `get_next_step`, `get_month_close` (read).

## Limitations

- The app does not suggest moving up a level when the family reaches step 5 ([profile.md](profile.md)).
- Step 5 of the closing leads to the Reports; the deterministic closing report
  (`ai/routines.rs`) does not yet appear inside it.
- A closed month can only be reopened through the command; there is no button in the interface.
