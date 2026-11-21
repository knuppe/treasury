# Accounts up to date: last transaction and "possibly incomplete" balance

Without the bank connected, each account's balance is whatever was entered or imported. An
account whose last transaction was weeks ago shows a balance that looks current, and that balance
contaminates financial health, the emergency fund and the debt analysis: a negative balance may be
overdraft or just missing transactions. So each account says how long it has gone without
transactions, and the screens that depend on the balance warn about it.

## Fields per account

`accounts.list` and `accounts.get` (in the MCP, `list_accounts` and `get_account_balance`) return:

| Field | What it is |
|---|---|
| `last_txn_date` | the most recent posted transaction up to today |
| `last_import_at` | the last statement import or balance check (`balance_sync`, except the ignored ones) |
| `staleness_days` | days since the more recent of the two dates; with neither, since the account was opened |
| `balance_status` | `fresh`, `aging` or `possibly_incomplete`; `null` on archived accounts |
| `missed_month` | last month, when it had no transactions at all on an account that usually has some every month |

The status comes from the days and the limits of the account type (`accounts::freshness`):

| Type | `aging` from | `possibly_incomplete` from |
|---|---|---|
| checking, savings, cash, foreign, other | 8 days | 21 days |
| credit card (monthly card statement) | 36 days | 51 days |
| benefit (voucher) | 21 days | 46 days |
| broker, retirement, FGTS | 46 days | 91 days |

The limits change in Settings › Notifications › Accounts without a recent statement (setting
`accounts.staleness`, only the types that were changed). An account is also `possibly_incomplete`
when it has transactions in each of the three months before last month and none in last month,
even with few days; the rule only applies to the types whose incomplete limit is up to 31 days.

## Where the status appears

1. **Accounts.** Card and list with the date of the last transaction and the badge: "updated N
   days ago" in gray or "possibly incomplete" in amber, with the last transaction, the empty
   month and, on a negative balance, "the negative balance may be missing transactions, not
   necessarily overdraft".
2. **Overdraft.** `overdraft_status.uncertain` when the account is not `fresh`: the estimated
   cost appears as a hypothesis.
3. **Financial health** (only in the current month's assessment). The everyday accounts in net
   worth and `possibly_incomplete` go to `stale_inputs`; the score comes out with
   `estimated: true` and a warning with the names. Their negative balance does not count as
   overdraft nor as suspicion on expensive credit: it stays on the list to be checked.
4. **Goals.** The outdated sources of a goal come out in `stale_sources`
   ([goal-sources.md](goal-sources.md)).
5. **Reminder.** "N accounts without a recent statement. Import?", at most once a week
   (`stale_accounts`, can be turned off in Settings › Notifications), with the shortcut to the
   import (`/lancamentos?importar=1&conta=…`).
6. **Dashboard, balance forecast and "how much can I spend today".** The coverage line
   (`attention.list` → `coverage`, [to-resolve.md](to-resolve.md)): "Accounts up to date:
   data until DD/MM" or the warning with the accounts without a recent statement and the shortcut
   to import.

## In the code

- `crates/treasury-core/src/accounts/freshness.rs` (days, limits and status).
