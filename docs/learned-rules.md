# Learned rules and annual expenses in the forecast

Two ways to reduce manual work without AI: category rules offered from what the person has
already categorized, and last year's annual expenses estimated again in the balance forecast.

## Learned category rules

Categorizing is the app's most repetitive task. When the person has given the same category to
the same merchant three times, Treasury offers the ready-made rule. **The rule is only offered;
nothing is created or applied unless the person asks.**

A suggestion appears when, in the last 12 months (`rules::learned`, command `rules.learned`):

- at least 3 income or expense transactions with the same significant part of the description
  (`rules::significant_run`, the same one used by "create rule from transaction") have a
  category, and **all have the same one**. A different category in between removes the
  suggestion: there is no pattern;
- no active rule already sets the category of one of those transactions;
- the person has not dismissed the text (setting `rules.learned_ignored`, with the "Do not
  suggest" button).

Forecast transactions, transfers and split transactions are left out. Suggestions are sorted by
how many uncategorized transactions they resolve now; at most 20.

Where it appears:

- **Transactions › Rules**, in the "Suggested rules" panel: the text, the category, how many
  already have the category and how many do not. "Create rule" saves the rule (the same as
  `suggest_from_txn`, with the shortest text that does not match transactions of another
  category) and applies it to the uncategorized ones, with undo in the history.
- **To solve**, as an informational task ("2 category rules ready to create").
- **MCP**: `list_learned_rules` (read). The agent offers; creating still goes through the
  `create_rule` proposal.

The app does not record "corrections": it looks at the result. Three transactions from the same
merchant with the same category count as three corrections, whether they came from a manual edit,
the agent or a reviewed import.

## Annual expenses in the balance forecast

With only what is registered (forecast transactions, recurrences, card statements,
installments), vehicle tax (IPVA), IPTU, insurance and school supplies would show up as a
surprise in the forecast. So last year's **essential annual expenses** come back as an estimate
(`forecast::seasonal`, origin `seasonal`, label "annual expense (estimate)").

Rule:

1. The categories come from `goals::living::annual_expenses`, the same detection used for the
   emergency fund: spending in up to 3 of the last 12 complete months. Without 12 months of
   history, nothing is estimated.
2. Only **essential** categories (the "Essentials" and "Debts" spending groups). A one-off,
   non-essential purchase from last year, such as a trip, is not an annual obligation.
3. Each transaction in the category comes back **twelve months later**, in the same account and
   for the **same amount**, with the description "… (estimate based on the YYYY amount)". There
   is no inflation adjustment.
4. No duplicates. The estimate is not added if the category has an active recurrence, if there
   is a forecast transaction in the category within 45 days of the date, or if this year's
   payment has already been recorded in that window.

### Limitations

- **Credit card**: annual expenses paid by card are left out, because the forecast does not yet
  estimate future card statements by category.
- **Annual income** (13th salary, IR refund) is not estimated. The forecast errs on the
  conservative side.
- "How much can I spend today" does not change: it already subtracts the month's forecast
  bills, and the annual estimate is not a forecast bill.
