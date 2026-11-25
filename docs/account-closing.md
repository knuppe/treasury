# Closing an account

Over the years a family closes checking accounts, cards and brokerage accounts. An account that
was closed at the bank should leave the lists and stop raising alerts (bills due, accounts
without a recent statement, negative balance ahead, subscriptions that "did not charge"), while
its history stays in the reports and in net worth on the dates it had a balance.

## Closed and archived

An account can be **archived** (hidden from the lists, no date) or **closed** (archived with a
closing date, `account.closed_on`, migration V5). Closing sets both, so every place that already
leaves archived accounts out of lists and alerts leaves closed accounts out too. Reopening
clears both; `archived: false` on a closed account also reopens it.

The date lives in the account row, not in the settings, so undoing a closing and the agent's
proposals carry it like any other change.

## When an account can close

`accounts.close_preview` (`accounts::closing::preview`) lists what stops the closing on the date:

- the date is after today, before the opening date, or the account is already closed;
- the balance on the date (cleared and reconciled transactions) is not zero: transfer it to
  another account or record a balance adjustment first (the dialog has both shortcuts);
- transactions or trades recorded after the date (pick a later date);
- assets still held at the broker on the date or today (record the sale or the custody transfer);
- card installments still to come.

`accounts.close` refuses with the first of them. The agent closes through `update_account` with
`closed_on` (a sensitive proposal, approved one by one) and reopens with `clear: ["closed_on"]`.

## What closing does

- Active recurrences from or to the account are paused, ending on the closing date (an earlier
  end date stays). Reopening does not reactivate them.
- Planned transactions of the account are deleted, with the other leg of planned transfers.
- The account is archived. Loans debited from it and cards paid from it are listed as a warning:
  change the account there.
- No transaction can be recorded after the closing date (`transactions::insert` and edits that
  move the date or the account check it, imports and recurrences included).

## Alerts

Besides everything that already skipped archived accounts (card statements, negative balance,
accounts without a recent statement, OFX and B3 account suggestions, insights):

- upcoming bills (`reports::upcoming`, used by the reminders and "To resolve") skip planned items
  of archived accounts and statements of closed cards (an archived card that still owes keeps its
  statement);
- subscription detection and detected installments skip closed accounts;
- a recurrence reactivated by hand generates nothing after the closing date.
