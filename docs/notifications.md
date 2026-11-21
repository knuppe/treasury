# Reminders and system notifications

Reminders for due dates, balance, budget, cards, subscriptions and investments, which become alerts in the app and, if the person wants, system notifications.

The **reminders** are deterministic and do not use an AI engine. They run at most once an hour, with the app open and the vault unlocked, inside the app's ~30 s cycle (`ai::run_due` → `notify::run_due`). Each reminder becomes an alert in the **Agent** inbox, and the same key never generates two alerts, even after being resolved.

| Reminder | Alert type | When | Key |
|---|---|---|---|
| Bills and card statements due | `due_soon` | planned transactions and card statements to pay in the next N days (default 3, from 0 to 15) | `due:<ref>:<date>` (the same as the agent routines) |
| IPTU due (Brazil only) | `iptu_due` | installment of the chosen plan (no plan: the first date) in the next N days, at least 3, or overdue by up to 30 days, with no transaction mentioning IPTU | `iptu:<installment>` (see [iptu.md](brazil/iptu.md)) |
| IPTU bill not registered | `iptu_missing` | property with a bill last year and none for this one, from 20 days before last year's first due date | `iptu_missing:<property>:<year>` |
| Balance going negative | `negative_balance` | the 1-month balance forecast shows the account negative | one per episode (see below) |
| Card limit almost used | `card_limit` | limit usage (with future installments) from 80% and from 95% | `card_limit:<card>:<statement month>:<band>` |
| Annual fee renewing | `card_fee_renewal` | annual fee with a renewal month entered, 30 days or less away | `card_fee_renewal:<card>:<date>` |
| Annual fee that does not match | `card_fee_issue` | annual fee charge on an exempt card statement, with a different amount or an extra installment ([design/cartoes.md](design/cards.md)) | `card_fee_issue:<transaction>` |
| Card statement in revolving credit | `card_revolving` | overdue card statement with a balance due, on a card with a payment recorded | `card_revolving:<statement>` |
| Charges above the cap (Brazil only) | `card_revolving_cap` | interest and charges posted since the revolving credit above what was left unpaid | `card_revolving_cap:<statement>` |
| Cashback or points that did not arrive | `card_cashback_missing` | the credit deadline passed (closing + program days) and less than half of the expected amount arrived (points: only with a points program on) | `card_cashback_missing:<statement>` |
| Points expiring | `loyalty_expiring` | points lot with a balance expiring in 90, 30 or 7 days | `loyalty_expiring:<program>:<date>:<band>` |
| Budget exceeded | `budget_over` | expense category of the month above the planned amount | `budget_over:<month>:<category>` (the same as the routines) |
| Subscription increase | `subscription_increase` | latest charge, within 15 days, more expensive than the previous one | `sub_increase:<key>:<date>` |
| New recurring charge | `subscription_new` | new subscription with a charge within 15 days | `sub_new:<key>` |
| Ex-dividend date | `dividend_datacom` | announced (not estimated) dividend of a portfolio asset with an ex-dividend date in the next 3 days | `datacom:<asset>:<type>:<date>` |
| Watchlist price | `price_target` | quote above or below the item's alert | one per episode (see below) |

The asset health score (`asset_score`) and the agent routines (`budget_pace`, `monthly_close`, `rebalance`) create their own alerts.

## Episodes: negative balance and watchlist price

These two conditions last for days. To avoid generating an alert per day, each account (or asset) has **one alert per episode**. The episode data lives in `data.episode`: open or not, since when, and the worst value.

- **While the condition continues:** at each check, the alert is updated in place, with the text, the lowest value and the dates. The title has no date ("Account: forecast balance goes negative"). If you marked it as seen or resolved, it stays that way and does not notify again.
- **If it gets much worse:** when the lowest point falls by more than 20% (and by more than R$ 100), the alert becomes new again and notifies again. This sets `data.notify_at`, which the system notifications read along with the new alerts.
- **When it ends:** if the forecast goes back to positive (or the quote goes back within the alerts), the episode closes. The alert is resolved with a line explaining why.
- **If it happens again later:** it becomes a new episode, with a new alert.

## System notifications

The alerts created since the previous round of the cycle (`RoutineReport.new_alerts`) go through the preferences (`notify::notices`) and become `RoutineReport.notices`. The app shows up to 5 per round:

- **macOS**: `osascript` with the text passed as an argument, never inside the script;
- **Linux**: `notify-send`;
- **Windows** (10 and 11): a toast through the WinRT API via PowerShell, with no console window. The title and the text arrive through environment variables and are escaped for XML. The sender shown is PowerShell's, which is always registered; a sender of its own would require registering Treasury's AppUserModelID in the installer.

By default the body **does not show amounts**, because the notification appears on the lock screen and in the system's notification center. It only says "Open Treasury to see the details.". The title never has amounts.

## Preferences

They are in **Settings › Notifications**, in the `notify.prefs` setting:

- `system`: turns system notifications on or off. Reminders keep going to the alert inbox.
- `show_amounts`: shows amounts in the notification body.
- One on/off per type: `bills` (with `bills_days`; it also applies to IPTU), `negative_balance`, `budget`, `subscriptions`, `dividends`, `price_targets`, `scores` and `agent` (the agent's other alerts). A type turned off creates no alert and does not notify.
- "Send a test notification" uses the Tauri command `system_notify`, and "Check reminders now" uses `notify.run_now`.

Commands: `notify.get_prefs` (read-only), `notify.set_prefs` and `notify.run_now`. The last two work only through the interface; MCP clients and routines get `forbidden`.

Code: `crates/treasury-core/src/notify/` and `apps/desktop/src-tauri/src/notify.rs`.
