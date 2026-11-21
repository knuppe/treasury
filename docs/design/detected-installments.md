# Installments detected from the description

Installment purchases that arrive as loose transactions ("LOJA X PARC 08/10") become a series: the app
detects them from the description, shows the installments in progress and puts the remaining ones in the balance forecast and
in the card commitments, marked as an estimate and without double counting. Nothing is converted
on its own into a Treasury installment plan (V163). Description examples are invented or generic.

## The problem

Many transactions say in their own description that they are an installment:

| Description | What it says |
|---|---|
| `Lj Exemplo Centro - Parcela 1/2` | card purchase, installment 1 of 2 |
| `PREST.EMPREST 003/005` | payment 3 of 5 of a loan, in the checking account |
| `Farmacia Filial - Parcela 1/2` | installment 1 of 2 |
| `Mercadolivre*Mercadol - Parcela 8/10` | installment 8 of 10 |

With the installment number and the total, it is possible to know **how much is still going out and until when**, without the
family registering anything. Today the app only knows this when the purchase was entered as an installment plan in
Treasury (`installment_plan`) or when there is a contract in Debts. What comes from the bank already
split into installments (OFX, CSV, PDF, agent) becomes a loose transaction per month.

## What the app already had

| Piece | Status |
|---|---|
| Card installment plan (`installment_plan`) | Generates the future installments as planned. Only for what is entered in Treasury. |
| Card statement PDF | Already recognizes `02/10`, `PARC 2/10`, `parcela 2 de 10` and puts the installment in the right month, but does not project the next ones. |
| Balance forecast (`forecast`) | Uses planned items, recurrences, open card statements and installments of Debts contracts. |
| Future card commitments | Add up what is left of the card statements, including Treasury installment plans. |
| Subscriptions | Already remove from the list what is an installment plan or a contract installment. |
| Debts (`loan`) | Contract with a schedule; manual link from the installment to the transaction that paid it. |

## How it works

### Detecting the series

A **series** is the same commitment seen over several months. It is identified by:

- account (card or checking account);
- normalized description, without the installment part ("Mercadolivre*Mercadol");
- total installments (`n`);
- similar amount (within 1 cent; the 1st installment may carry the cent from the division).

Patterns recognized at the end of the description (ignoring case and accents):

- `Parcela 8/10`, `PARC 08/10`, `PARC.08/10`, `parcela 8 de 10`;
- `PREST 3/5`, `PREST.EMPREST 003/005`, `PRESTACAO 03 DE 12`;
- `8/10` alone, only under the conditions below.

**`8/10` alone is ambiguous.** It may be a date (8 October). So it only counts:

- on the card, when the installment number matches the series (the same purchase appearing in
  consecutive months with `k` rising by 1); **or**
- outside the card, when the series is confirmed in **two consecutive months** (k and k+1).

With a keyword (`parcela`, `parc`, `prest`), one occurrence is enough. Outside those rules, the
number is left out, so as not to invent a commitment.

### Projecting what is left

For each series in progress (`k < n`):

- remaining installments: `n − k`, one per month;
- date: the same day of the month as the last installment (on the card, the purchase day, and the card statement comes out by the
  card's cycle as with any purchase);
- amount: that of the last installment;
- expected end: the month of installment `n`.

The series ends on its own when it reaches `n/n`. It also goes away when two months pass with no sign of the next
installment: the purchase may have been paid off or reversed. In that case the app shows "did not
show up in October" for a month before removing it.

### Where it shows up

- **Balance forecast**: the remaining installments enter as their own source, "Detected installments",
  marked as an estimate. On the card, they enter the statement of each month. In the checking account, they enter on the
  expected day.
- **Future card commitments**: add up the detected installments, separate from the Treasury ones.
- **"Installments in progress" list** in Planning: description, `k/n`, installment amount, how much
  is left, end month, account or card, member (by plastic or holder, [cards.md](cards.md)).
  Actions: **Ignore** (not an installment), **Convert to installment plan** (on the card) and **Create contract in
  Debts** (loan, already with `n`, current installment and amount).
- **Agent and MCP**: `list_detected_installments` (read), to answer "how much do I still have
  in installments" and "when does it end".

### Without double counting

The series does not enter the forecast when:

- the transactions already belong to a Treasury installment plan (`installment_plan_id`);
- any of them is already linked to a contract installment in Debts, or falls in a
  contract's category;
- there is already a planned transaction in the same account, with a similar amount, near the projected date (the
  same rule the forecast already uses for contracts);
- the family marked **Ignore**.

Converting or creating a contract swaps the projection for the real record, and the series stops being
"detected".

### Link with the card plan

The **card statement installment plan** appears on the statement as `PARC FATURA 03/10` or `PARCELAMENTO FAT
3/10`. It becomes a series like any other, but with a "statement installment plan" mark and the
interest visible (installment × n minus what was left unpaid). This resolves the open item without creating a
contract or counting the debt twice:

- the projection shows how much is left to pay;
- the **Create contract in Debts** action is available for those who want to simulate early
  payoff;
- in that case, the statement's installments are linked to the contract and leave the series.

## Other countries

The mechanism is the same; the word list and some formats change.

| Country | How it appears | Note |
|---|---|---|
| Brazil | `Parcela 3/10`, `PARC 03/10`, `PREST 3/5` | Interest-free installments are very common on the card. |
| Argentina, Chile, Uruguay | `CUOTA 03/06`, `Cuota 3 de 6`, `C.03/06` | Cuotas on the card are common (sometimes "Ahora 12"). |
| Mexico | `MSI 3/12`, `3 DE 12 MSI` | Months without interest. |
| Spain, Portugal | `cuota 3/12`, `prestação 3/12`, `PAGO APLAZADO` | More common in financing and split purchases. |
| US, Canada, United Kingdom | `INSTALLMENT 3 OF 4`, `AFTERPAY`, `KLARNA`, `AFFIRM`, `PAY IN 4` | BNPL (buy now, pay later): biweekly or monthly installments, sometimes outside the card. |
| France, Germany | `3X SANS FRAIS 2/3`, `Ratenzahlung 2/6`, `KLARNA` | Payment in 3 or 4 parts and Klarna. |

Differences the design needs to accept:

- **interval**: BNPL is usually biweekly (Afterpay, Klarna "pay in 4"); the series keeps the
  observed interval between installments (15 or 30 days);
- **words by country**: one table per country, like the ready-made category rules;
- **no number**: Afterpay and Klarna sometimes do not bring `k/n`. In that case they enter only through the
  recurrence of an equal amount every 2 weeks, which is already part of Subscriptions. It is not detected.

## How it is implemented

- **V163**: `detected_installment_ignore` (series key, account, description). The series are
  computed on the fly (`installments::detect`); nothing else is stored.
- **Reading** (`installments::parse`): `Parcela 8/10`, `PARC 02/12`, `parcela 3 de 6`,
  `PREST 3/5`, `PREST.EMPREST 003/005`, `PMT 2/4`; `k/n` alone at the end only with the series
  confirmed (the next installment on the next statement, on the card, or 20 to 45 days later, in the
  account). Limits: with a keyword up to 120 installments, without a keyword up to 72.
- **Series**: account + normalized description without the installment + total; the chain is made of the installments with an
  amount similar to the last one (within `n` cents or 1%). Types: card purchase, card statement installment
  plan (`fatura`, `parc fat`), loan (`prest`, `emprest`, `financ`, `consig`) and an installment in the
  account. Last 13 months; excluded: Treasury installment plans, transfers, transactions linked to a
  contract installment or in a contract's category, and the ignored ones.
- **Projection**: on the card, one statement per month after the statement of the last installment (date = the
  closing, with the due date); in the account, the same day of the month. Next installment more than
  15 days late: `missing` (stays out of the forecast); more than 45 days: the series leaves.
- **Balance forecast**: new source `detected` ("Detected installment (estimate)"); on the card, the
  purchase at closing and the payment on the due date through the payment account. It does not add when there is already
  a planned transaction in the same account with a similar amount within 7 days.
- **Future card commitments**: add up the detected installments in the due month.
- **Commands**: `installments.detected`, `installments.ignored`, `installments.ignore`,
  `installments.unignore` (undoable through the history). MCP `list_detected_installments`.
- **Interface**: **Installments** tab in Budget (description, type, account, member, `k/n`, amount,
  how much is left, end month, "Not an installment"; total left and the next month).
- Phases 3 (convert to an installment plan and create a contract) and 4 (other countries and the biweekly interval of
  BNPL) are left for the next round.

## Limitations

- There is no conversion of the series into a Treasury installment plan nor creation of a contract in Debts
  (including for the card statement installment plan).
- Only Brazil's words; the other countries and the biweekly BNPL interval are not detected.
