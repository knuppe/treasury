# Subscriptions and recurring charges

Subscriptions and charges that repeat, found in the transactions without needing a registered recurrence, with alerts for price increases, new charges and charges that did not arrive. They are in Budget › Subscriptions.

The detector looks, in the posted expenses of the last 2 years, for what repeats at the same merchant and in the same account with a similar amount, every month or every year: streaming, gym, software, health plan, annual fee. It works with or without a registered recurrence.

## Rules

- **Merchant**: the payee (or description) normalized, without accents, punctuation or numbers. Each merchant is kept separate per account.
- **Left out of detection**: installment purchases, transfers, card statement payments, planned transactions, copies across accounts (the same bank id in two accounts, see `transactions::duplicates`) and descriptions that only name the payment method, without the merchant ("Débito em conta", "Débito automático", "Compra no débito", "Pagamento de boleto", "Pix enviado", "Saque"), which would join different charges into a false series.
- **Same series in two accounts**: same name, rhythm and amount, with the last two charges on the same dates (one account's statement imported into another without the bank id) becomes a single subscription: the card one stays, otherwise the one with a recurrence, otherwise the one with more charges. Dismissing the series from either account removes it from the list.
- **Monthly**: the median interval between charges is between 26 and 35 days, with at most one interval outside 20 to 40 days, and there are at least 3 charges. With only 2 charges, it enters only if it is new (the first one up to 70 days ago) and has an equal amount.
- **Annual**: the median is between 350 and 380 days, with at least 2 charges.
- **Amount**: the largest is at most 1.5 times the smallest. Larger variations indicate variable spending, such as groceries or pharmacy.
- **Status**: `active` (on time), `new` (the first charge was up to 70 days ago) or `late` (the expected charge did not arrive within 10 days, or 30 days for annual ones; maybe canceled). A subscription late for more than 60 days leaves the list.
- **Price increase**: the last charge compared with the previous one, when it changes by at least 1% and R$ 1.00 (`change_pct`).
- **Totals** per month and per year: only active and new subscriptions in the base currency. Annual ones count per month as 1/12.
- **Recurrence**: when none is registered (by the transaction or by the description in the same account), the result carries the `suggested_recurrence`, ready for `recurrences.create`.

The user can dismiss a detection ("Not a subscription"). The key is kept in the `subscriptions.dismissed` setting, outside the history, and can be brought back.

## In the code

- Interface: **Budget › Subscriptions**. It shows alerts for recent increases (60 days), new charges and charges that did not arrive, the table with the "Create recurrence" and "Not a subscription" buttons, and the dismissed detections.
- Commands: `subscriptions.list` (read only), `subscriptions.dismiss` and `subscriptions.restore` (interface only). MCP tool: `list_subscriptions`.
- Reminders: a price increase and a new charge in the last 15 days become an alert and a notification (see [notifications.md](notifications.md)).
- Code: `crates/treasury-core/src/subscriptions/`.
