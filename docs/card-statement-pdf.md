# Card statement PDF

The card statement PDF enters through the statement import, in the card's account. The ways in are the account's Import button, the general import (drag the file) or the agent, with `preview_card_statement_pdf` to read and `import_statement_file` to save. The import uses the same deduplication and the same category rules as an OFX or CSV.

## Reading

The text comes from the app's PDF reader (`crate::pdf`). A protected PDF asks for the password, which is usually the first digits of the CPF. The password is used only for reading and is not stored. The layout is identified through the layout registry ([layouts.md](layouts.md)). A PDF that looks like a card statement but has no readable lines is rejected with a clear warning.

The `card.generico.v1` layout reads:

- **Transaction**: lines that start with a date (`12/10`, `12/10/26` or `12 OUT`), followed by the description and the amount at the end (`1.234,56`, with an optional `R$`). Credits and refunds come with `-`, `CR` or `C`, or by the word (estorno, crédito, devolução).
- **Year**: comes from the due date. A purchase from a month after the due date's month belongs to the previous year.
- **Installments**: `02/10`, `PARC 2/10` or `parcela 2 de 10` at the end of the description. Installment k goes to the month in which it is charged (purchase date + k − 1 months), and the number stays in the description, which separates the installments in deduplication.
- **Card of each line**: statements with additional cards separate the transactions into blocks ("Cartão final 1234", "**** **** **** 1234 NOME", "CARTÃO VIRTUAL FINAL 1234"). The block's last digits go on each of its lines and link the purchase to the registered card with those digits, which gives the member. Unregistered last digits become a warning in the preview.
- **Left out of the import**: payments of the previous card statement ("pagamento recebido"), which enter through the account that paid, and previous balance, totals, subtotals, limit and minimum payment.
- **Due date and total**: read from "vencimento" and from "total desta fatura" (or da fatura, a pagar, valor total).

## Saving

All transactions go to the **card statement of the printed due month**, including purchases after closing and old installments. The card statement is created if it does not exist yet, and the printed total becomes its reported total, used for checking. The preview shows the printed total and the total read from the lines, and warns when the two do not match, when the due date or total is missing and when payments were left out.

The batch gets the `pdf` format (migration V114). Reimporting the same file warns that it was already imported; another card statement with the same purchases falls into deduplication.

## In the code

- Commands: `import.detect`, `import.preview` (format `pdf`, with `password`), `import.commit` and `import.card_pdf_preview` (read-only).
- Code: `crates/treasury-core/src/import/card_pdf.rs`. The interface mock does not read PDFs.

## Limitations

- Only the generic layout exists, written without real PDFs from each bank. When a bank's card statement is read wrongly, keep a sample without personal data and add the `card.<bank>.v1` layout before the generic one, as described in [layouts.md](layouts.md).
