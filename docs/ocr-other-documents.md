# OCR on other documents

Besides receipts ([ocr-receipts.md](ocr-receipts.md)), local OCR reads boletos (Brazilian payment slips), utility bills, Pix QR codes and payment proofs, and passes the text of images to the agent and to the ingestion email. This document describes what exists and records the other documents that were evaluated. Examples are invented; no real document goes into the repository ([development.md](development.md), "Personal data").

## What exists

- **Common layer** `crate::ocr` (`read_text`, `OcrText`; command `ocr.read_text`): Vision through the app, on the Mac, or PaddleOCR; the lines come out of the same joining step as the receipt (`lines::row_texts`).
- **Agent:** an attached image gets the OCR text (`ai::attachments::add_ocr`, with the confidence and the type it seems to be). The message box asks what to send: only the text, only the image or both (`ChatStartArgs.image_mode`, remembered per conversation); suggestion by confidence (≥ 90%: only the text); with no vision engine, only the text. `read_attachment` also returns the text.
- **Email:** an image goes through OCR at identification (boleto, Pix, proof; `dedup_key` by the code, by the Pix hash or by the end-to-end ID) and, in the rule's AI, only the text goes when OCR read it.
- **Boleto, utility bill and Pix** (`crate::payments::{boleto, pix}`): 44 digits or digitable line (47/48) with modulo 10/11 check digit and the due-date factor with the 2025 restart; BR Code with CRC16, without querying the dynamic Pix URL. `payments.plan` creates the **planned transaction** (`status = planned`, `external_id` `boleto:<44>` or `pix:<hash>`, digitable line in the note) that goes into bills due and into the "due_soon" reminder. Category by segment.
- **Proof** (`payments::proof`): amount, date and time, who received or paid, institution, end-to-end ID (the format checks out and gives the date and the bank), scheduling (becomes planned). `payments.save_proof` links to the statement transaction (same amount, ±2 days), puts the payee and the data read in the note, settles the planned item it pays; with no transaction, it creates one. **No attachment**: the note keeps the file's `sha256:`, and `payments.read` warns when the same file (or the same end-to-end ID) was already linked.
- **Codes in the photo:** on the Mac, `read_payment_codes_native` (Vision: QR and interleaved 2 of 5); elsewhere, the QR through the JavaScript reader and the digitable line through the OCR text. No `rxing`.
- **Interface:** the import window opens `PaymentDialog` for boletos, Pix and proofs (review, account, category; copy the digitable line). The agent is the last resort.
- **Measurement** with synthetic samples kept outside the repository (test `payments::tests::bench_with_private_samples`, ignored, numbers only): boletos 16/16, utility bills 12/12, proofs 14/14 (amount, date and who received), PNG and photo. The Pix QR only through the QR reader (the image text does not carry the code).

## Other documents evaluated (not implemented)

Scores: value for the family (V), effort (E: P ≤ 3 days, M ≤ 1–2 weeks, G > 2 weeks), expected accuracy without AI (A), new migration (Mig).

| # | Document | V | E | A | Mig | Priority |
|---|---|---|---|---|---|---|
| 5 | Payslip as a photo or scanned PDF | High | P (with the scanned PDF read per page) | High | No | **2** |
| 6 | Scanned card statement, bank statement, brokerage note, FGTS and pension | Medium | P each (with the scanned PDF read per page) | Medium–high | No | **2** |
| 7 | Income statement (text and image) | High (Mar–May) | M | High | No | **2** |
| 8 | Health and education receipts for the IR | High (Mar–May) | M | Medium | Yes | **3** |
| 9 | Utility bills (electricity, water, gas, phone) | Medium | M–G | High on the amount, medium on consumption | Yes | **3** |
| 10 | IPTU carnê | Medium | M | High on the installments, medium on the rest | No | **3** |
| 11 | Store and financing carnês | Medium | P (with 2) | Very high | No | **3** |
| 12 | Screenshots of bank and broker apps | Low–medium | G | Low | No | **4** |
| 13 | Searchable text of attachments | Medium | M | — | Yes (FTS) | **4** |
| 14 | Property documents, contracts, deeds | Low | — | Low | — | Do not do |

### Payslip (#5)

`income/payslip.rs` reads PDF and text and refuses images. With OCR outside the receipt and the scanned PDF read per page, a photo and a scanned PDF become text and go through the same reader and the same layouts of `payslip_specs.rs`. The check already exists by nature: earnings − deductions = net. It only goes in without review when it adds up. A lot of return for little code, after 2.2. Risk: columns misaligned by OCR (code, description, reference, earnings, deductions); the layout tests also run on the invented "messy" text as it comes out of OCR.

### Scanned card statement, bank statement, brokerage note, FGTS and pension (#6)

Same path: 2.2 and the current reader. Each already has its check (note: financial summary; FGTS: JAM rule; card statement: total; bank statement: previous balance + movements = final balance). Suggested order by how often they occur at home: old scanned brokerage note and FGTS statement as a photo (Caixa Tem), then card statement and bank statement, pension last. A scanned statement with many pages is the slowest case (2 s per page in Paddle).

### Income statement (#7)

Today it is typed by hand (`tax_burden`, "informes"). The income statement from a legal entity and from a financial institution follow Receita templates with **numbered boxes** (taxable income, exempt, subject to exclusive taxation, additional information), the same at any company. A reader per box works for the PDF with text (most of them) and, with 2.2, for the photo. Check: CNPJ of the source (check digit), calendar year, sum of the boxes. High value from March to May; it can also serve the tax return ([income-tax-validation.md](brazil/income-tax-validation.md)). Effort **M**.

### Health and education receipts for the IR (#8)

Today the return's medical expense is the sum of the Health categories (`tax/annual_draft.rs`). The Receita asks, per payment, for the **provider's CPF/CNPJ**, name, amount, who was treated and the reimbursement. OCR of the receipt (CPF/CNPJ with check digit, amount, date, patient name, CRM/CRO when present) allows a "Deductible payments" list with the receipt attached, ready for the "Pagamentos efetuados" (payments made) form and for the fine mesh (the kept receipt is the proof). It needs a new table (deductible payment linked to the transaction and the member). Risk: handwritten receipt (weak OCR: stays in "review"); health data is sensitive, it stays only in the vault.

### Utility bills (#9)

- **First step, no table:** the utility bill code (3.2) already gives the amount, segment and, with the text, the due date. This covers "pay and remind".
- **Second step, with a table:** the month's consumption (kWh, m³), reference month, tariff flag, and the **12-month history that the bill itself prints** (only the numbers in the table; not the bar chart). It would give consumption × spending, an alert for unusual consumption and the cost per kWh against the rate increase. The layout varies by utility: per-company profiles, waiting for real bills.

### IPTU carnê (#10)

Today the agent reads the photo and calls `record_iptu`. With 3.2, the installments (segment 1, city halls) come out of the carnê's barcodes with amount and check digit; with OCR, the assessed value and the due date by words ("Valor venal", "Cota única"). It closes the check that `physical/iptu.rs` already does (taxes + fees − discounts = total). [iptu.md](brazil/iptu.md) already foresaw "carnê reading" per city hall.

### Store and financing carnês (#11)

One page with several boletos: 3.2 reads all of them, and the app proposes the debt (Debts) with the planned installments. Cheap after the boleto.

### Screenshots of bank and broker apps (#12)

They vary with each app update and by bank; the numbers appear without a stable label. It would only be worth it for "portfolio position" (recognized ticker + quantity, checked against the app's assets) and for a proof screenshot (already covered by 3.4). Leave it to the AI, receiving the OCR text (3.1).

### Searchable text of attachments (#13)

Keep the OCR text of each attachment (proof, receipt, boleto) in an SQLite FTS5 index, in the vault. "Find the dentist receipt from March" becomes a local search, and the agent queries it too. Useful, but only once there are attachments in volume (3.4, 3.8).

### Property documents and contracts (#14)

Deed, registration, rental contract: little structured data that the app would use, a lot of legal text. Keep as an attachment and, at most, index it (3.13). Not worth a dedicated reader.

## Limitations

- Outside the Mac, the Pix QR is only read by the interface's JavaScript reader, and the boleto through the digitable line in the OCR text (no ITF barcode reader).
- The app does not keep the proof's image, only the file's `sha256` in the note.

## Sources

- FEBRABAN, technical specifications of the collection boleto (barcode, digitable line, due-date factor and the restart on 02/22/2025).
- FEBRABAN, standard layout for collection/receipt with barcode (segments and check digit).
- Banco Central, Pix Initiation Standards Manual (BR Code, EMV QRCPS-MPM, CRC16) and DICT/SPI manual (end-to-end ID format).
- Receita Federal, templates of the Comprovante de Rendimentos Pagos e de Imposto sobre a Renda Retido na Fonte (statement of income paid and tax withheld) and of the financial institutions' income statement.
- Treasury code: `import/image.rs`, `receipts/ocr/`, `pdf.rs`, `income/payslip.rs`, `ai/attachments.rs`, `payments/br_code.rs`, `physical/iptu.rs`, `tax/annual_draft.rs`, `src-tauri/src/{ocr,qr}.rs`, `UniversalImport.tsx`.
