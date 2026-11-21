# Fiscal receipts

Treasury reads the fiscal receipt of purchases (NFC-e, the Brazilian electronic consumer invoice) item by item: what was bought, in which
product category, by which family member and how much tax was in each item. The NFC-e
and the tax per item are Brazilian ([countries.md](countries.md)); the QR of Argentine,
Portuguese and Spanish receipts and the receipt with tax summed at the end (US, Canada) are covered in "Receipts from
other countries".

- **Where it is:** Consumption › Purchases › Add purchase (up to 0.8, in Transactions ›
  Fiscal receipts; the tab was removed on 2026-11-13 and the old link opens Consumption › Purchases;
  [consumption.md](consumption.md)).
- **Core:** `receipts` (V124: `receipt`, `receipt_item`), commands `receipts.preview`,
  `receipts.import`, `receipts.update`, `receipts.list`, `receipts.get`, `receipts.summary`,
  `receipts.read_text`, `receipts.foreign_qr` and `receipts.delete`.
- **Agent:** `preview_receipt`, `import_receipt` (proposal) and `get_purchases_summary`.

## How the receipt comes in

In Consumption › Purchases › **Add purchase** (`features/receipts/ReceiptReader.tsx`) there is a
single area: drop or choose the photo, PDF or XML, or paste the QR link (pasting already reads). The app
works out what it is and shows the steps with progress (find the QR in the photo, query SEFAZ (the state tax authority) or
read the AFIP note, read the items from the photo's text, check items and total), then the result
(how many items, from which store, whether the sum matches the total, the next step) with the button to
review and save. When something fails, the error comes with what to do: open the lookup in the app (captcha),
read from the photo (when the photo had the QR), paste the page text or register by hand. Outside
Brazil, the same area accepts the photo and the AFIP QR.

| Way in | How |
|---|---|
| QR link | The phone camera reads the receipt's QR and shows the link to the public SEFAZ lookup. Pasted into the app, Treasury opens the page (only `.gov.br` links, always over https; source "SEFAZ" in Settings › Data sources) and reads the items. |
| QR photo | The app finds the QR in the photo (jsQR, on the computer itself; the image is neither stored nor sent) and continues like the link. |
| Receipt PDF | The DANFE NFC-e (the printed representation of the NFC-e) that some stores send by email or through the app. |
| Invoice XML | The NF-e or NFC-e XML (`<infNFe>`) that the store sends by email: store, key, items, totals and approximate taxes (`vTotTrib`, and the split in `infCpl`). Nothing is read from the recipient. |
| By email | The ingestion email ([email-ingest.md](email-ingest.md)) recognizes the XML, the PDF and the lookup link in the body and sends them to the review queue (or applies it on its own, if the rule allows). |
| Lookup window | For the SEFAZ that ask for a captcha: "Open the lookup here" opens the lookup in an app window; the person solves the captcha and the receipt page is read automatically ([ocr-receipts.md](ocr-receipts.md) §13). |
| Page text | Without the app (or if the window fails): open the link in the browser, copy the page and paste. |
| Photo without QR | OCR on the computer (Apple Vision on the Mac, built-in PaddleOCR on the other systems), a line reader with arithmetic checks and, if the person asks, AI. It opens the purchase filled in for review ([ocr-receipts.md](ocr-receipts.md)). It works in any country. The agent can also read the photo and call `import_receipt` with `manual`. |

The reader (`receipts/nfce.rs`) understands the common layout of the SEFAZ lookups (`txtTit`, `RCod`,
`Rqtd`, `RUN`, `RvlUnit`, `valor`, `totalNumb`) and the DANFE text. It extracts the store and CNPJ, the access key
(44 digits, with the check digit verified; the state comes from the first two), the date and
time of issue, the items, the total, the discount, the amount paid and the printed **approximate taxes**
(Lei 12.741/2012, the Brazilian law on tax transparency), with the federal, state and municipal parts when the note has them. The consumer's CPF
and name are neither read nor stored.

The same note (by access key) does not enter twice; importing again asks to replace.

## Category and tax of each item

The person sees and chooses only the **product category** ([consumption.md](consumption.md)). The receipt
group (`item_group`) is still stored, but only as a Brazilian tax group, derived from the
category: changing an item's category (in review or later, through `receipts.update`) changes the
group, and the item's estimated tax starts using the median of the new group.

`receipts/classify.rs` gives the group from the receipt description, with the common abbreviations ("SAB PO
OMO", "LTE COND", "CERV BRAHMA LT"). The item's rate is that of its consumption group in the
tax estimate (`tax_burden`, IBGE National Accounts: supermarket 7.42%, beverages 18.36%,
cleaning 20.08%…); a group with no official data gets no estimated tax. The product list
of the Impostômetro (IBPT), which gave one rate per product, is no longer used; the column
`receipt_item.ibpt_product` stays in the table, empty.

- **Note with printed taxes:** the printed total is prorated across the items in proportion to each one's estimated
  tax. The sum matches the note.
- **No printed taxes:** each item's tax is the one estimated by the group's percentage
  (origin `estimated`; in the `tax_origin` column the stored value is still the old one, `ibpt`).

Groups: produce; meat, poultry and fish; dairy, cold cuts and eggs; grocery; bakery; sweets,
cookies and snacks; frozen and ready meals; beverages; alcoholic beverages; cleaning; hygiene and
beauty; baby; pet; pharmacy; home and housewares; cigarettes; other.

## Transaction and member

The fiscal receipt is reviewed on the same screen as other purchases (the Purchase), with what the document
says locked (items, amounts, store, date). The person chooses each item's category, the member and
the transaction, with the same three ways as a typed purchase: link it to an expense that already exists
(the app suggests the one with the same amount between 2 days before and 5 days after; card posts later), create the
expense along with it (deleting the receipt deletes it) or leave it with no transaction. After it is saved, the receipt
stays editable on these three points. The default member is the transaction's
([members.md](members.md)).

## What the family buys

`receipts.summary` adds up, for the period, by consumption segment (the root of the product category:
spending, tax, items), by member and the items that weigh the most. It is what the agent's
`get_purchases_summary` tool returns. On screen: "What the family buys" (this month, 3 months, the year).

## In the tax estimate

Receipts linked to a transaction enter the Taxes screen in place of the average of the transaction's
category: the amount paid leaves the category and becomes the line "Scanned receipts", with the note's tax
(the printed one, or the estimate item by item), the same in the three scenarios. The screen says how much of
the spending came from receipts.

## Receipts from other countries

- **Argentina (AFIP).** The QR of Argentine notes (RG 4892/2020) leads to
  `https://www.afip.gob.ar/fe/qr/?p=…`, with a base64 JSON: date, CUIT of the issuer, point
  of sale, type and number of the voucher, amount, currency and exchange rate. `receipts.foreign_qr`
  (`receipts/foreign.rs`) reads this link in any country of the family, also in a vault in Brazil
  (a purchase made on a trip), and the screen opens the **manual purchase filled in**: date, store as
  `CUIT 30-…`, one line with the voucher and, when the app has the exchange rate for the note's currency, the
  amount in the family's currency. The QR **does not carry the items nor the store name**: the purchase warns about it,
  and the items come in through the receipt photo or by hand.
- **Other countries.** The link of a QR that is neither from SEFAZ nor from AFIP is rejected with a
  message saying where it is from and what can be done (a photo of the receipt or a purchase by hand).

## One query per QR

The preview and the save read the **same page**: the lookup page with items is kept in the
app's session (12 hours, by access key), and "Save receipt" does not query SEFAZ again,
which might ask for a captcha the second time. A QR whose key was already saved does not even reach SEFAZ: the
app warns that the receipt was already imported (to read it again, delete the receipt first).

## Limits

- Each SEFAZ has its own page; the reader covers the common layout. A different page, with a captcha or
  down: the app warns and suggests the PDF or the text.
- The group's percentage is national and counts only the taxes on the product. The tax printed
  on the note is already from the right state and tends to be higher, because the table used by stores adds up the
  taxes of the whole chain.
- With the tax reform (CBS from 2027, IBS from 2029 to 2032), the tax will come itemized on the note:
  reading the note ages less than keeping a table.
