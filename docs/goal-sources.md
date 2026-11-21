# Goal sources: accounts, investments and labels

A goal's value comes from sources linked to it: accounts, investment positions, asset labels and
transactions. The emergency fund is usually in daily-liquidity fixed income, not in the checking
account, and an account in the red cannot make the goal negative; that is why an account's
negative balance never counts.

## Sources

A goal's current value is the sum of its sources (`goals::sources::current_value`, the same
calculation on screen, in MCP, in the emergency fund and in financial health):

| Source (`kind`) | Where it lives | Value (`value_used`) |
|---|---|---|
| `initial` | `goal.initial_amount` | the amount entered |
| `plan` | goal linked to the retirement plan, without accounts | the portfolio at market |
| `account` | `goal_account` | the balance, **never negative**: the red stays in `ignored_negative` and is debt, not reserve |
| `account_positions` | `goal_holding` without an asset (V148) | the whole account: cash and positions |
| `asset` | `goal_holding` with an asset (V148), with a share or a common pot | net of redemption or market (`value_mode`, V160) |
| `asset_tag` | `goal_source` (V160), `asset_label` label (V161) | all positions with the label, including those labeled later |
| `deposits` | transactions linked to the goal | the amount of the transactions |

Accounts live in `goal_account` and positions in `goal_holding` (a whole account or an asset,
with a share and a common pot among goals, also used by the college goal and by the portfolio by
objective); `goal_source` only stores the labels. The value type of positions (`value_mode`)
defaults to
`net_redemption`: fixed income enters at the net redemption value (already with IR and IOF) and
the rest at market value. Without a quote, the cost (`cost`) enters and the source is marked
outdated.

A position linked to the goal by the position itself does not enter again through the label.

## Response

`goals.list` and `goals.get` (in MCP, `list_goals`) return:

- `breakdown`: each source with the value, the value type used, the share and the negative left
  out;
- `sources`: the linked sources (what can be unlinked);
- `stale_sources`: the sources that may be outdated, that is, a `possibly_incomplete` account
  ([account-freshness.md](account-freshness.md)) or an asset without a quote.

## Linking and unlinking

- **API:** `goals.link_source` (`kind` `account`, `asset` or `asset_tag`; `ref_id`; for an asset,
  `account_id` when it is in more than one account, `share` and `value_mode`) and
  `goals.unlink_source`.
- **MCP:** `link_goal_source` and `unlink_goal_source` become reversible proposals, and nothing
  changes before approval.
- **Shortcut:** `goals.liquidity_candidates` lists the daily-liquidity positions outside the
  goal: fixed income with daily liquidity, Tesouro Selic, a DI or referenced fund.

## Screen

In Budget › Goals, each goal's **Sources** button opens the breakdown: how much each source is
worth and which value entered, the accounts and labels to link or unlink, and the "Use my daily
liquidity investments" button. Positions with a share stay in the position editor, right below.
The goal card warns "The amount may be outdated" when there is a source in `stale_sources`.

## Emergency fund and financial health

`goals.emergency_reserve` uses the value of the emergency fund goal. Without a goal, it sums the
checking, savings and cash accounts, also without the negative balances. The Security dimension
changes when an investment enters the emergency fund goal.
