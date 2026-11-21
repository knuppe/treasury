# Consumption: review, simplification and other countries

Study and plan for screen 06 · Consumption (tax receipts, items, products, GTIN, basket
inflation). First version on 11/16/2026; this is the second, with the refined design and the
United States (section 5). Base documents:
[consumption.md](../consumption.md), [gtin-inflation.md](../gtin-inflation.md),
[receipts.md](../receipts.md), [ocr-receipts.md](../ocr-receipts.md) and
[ocr-other-documents.md](../ocr-other-documents.md). Examples use invented data.

## Status (11/17/2026)

Approved on 11/17/2026 and implemented in four stages (A to D), one branch per stage, all
stacked: `feat/consumo-simples`, `feat/consumo-cadastro`, `feat/consumo-eua` and
`feat/consumo-paises`. How the system ended up is described in [consumption.md](../consumption.md),
[receipts.md](../receipts.md) and [gtin-inflation.md](../gtin-inflation.md); this file remains as
the rationale. What came out different from the plan:

- **A2**: the expressions in `classify.rs` were not rewritten as per-category lists. The group
  is still the internal key between the Portuguese words and the estimated tax; the person only
  sees and chooses the category, and a single function (`food::classify`) gives the predicted
  category with the reason. Less risk for the NFC-e part, which was never tested with a real
  receipt.
- **A1**: the two equivalence tests "before deleting the old path" were not written; the
  existing NFC-e tests now run through the new path and still pass.
- **B6**: the account suggested when creating the transaction is still the first one in the
  list (not the one from the last purchase at that store).
- **C1**: fees such as a bottle deposit go into Other (there is no new "Fees and deposits"
  category); the discount on the line below an item goes to the receipt's discount, not to the
  item.
- **C2**: UPC-E is not expanded.
- **C5**: national average prices per item were left out, as planned.
- **C7**: the US demo vault did not get purchases with items.
- **D3**: only VeriFactu in Spain (no TicketBAI). The table of fields of Portugal's QR was
  written from the published specification (v1.1); the AT page did not open on 11/17/2026 to
  check it again. The same receipt from another country read twice is not blocked.
- **E**: `Product.gtin` is still in the type.

## Summary

- **Stays as it is:** the rule "the transaction is the money, the receipt is the detail"; the
  GTIN as a strong natural key (not a primary one); the product per store (alias); the basket
  index (same product, same store, month by month) with the coverage rule; Open Food Facts as
  optional and only with the GTIN.
- **Complicates without gain:** two review screens, two classifications of the items (with
  translation both ways) and two summaries.
- **Hinders entry:** you cannot itemize a transaction from Transactions; the sum of the items
  has to be exact; the "automatic" category does not show what it will pick; the product only
  matches by exact name; there is no way to repeat a purchase.
- **Other countries:** the screen exists in all of them, but the classification only understands
  Portuguese, the packaging only reads metric measures, the official food index only exists for
  Brazil and only Argentina's QR is read.
- **United States:** today **no US receipt reconciles**. The sales tax is added at the end of
  the receipt, the reader skips the tax line and compares the sum of the items with the total,
  which never matches. It is the first fix of the countries front.

Stages, in this order: **A** simplify, **B** faster entry, **C** United States, **D** other
countries, **E** small cuts. Only D has a migration (V164, data only).

## 1. What exists today (measured on main 99fa3b87)

- Core: `food/` with 5.8 thousand lines and `receipts/` with 3.4 thousand. Interface:
  `features/food` and `features/receipts` with 4.8 thousand. Mock: `api/mock/food.ts` and
  `receipts.ts`, 1.5 thousand lines that repeat core rules.
- **Entry points** (in Consumption › Purchases):
  1. "Read receipt" opens a panel toggled by a button. SEFAZ link, PDF and XML go to the
     `PreviewDialog`, which shows the **receipt group** (`ItemGroup`, 25 groups, for the
     estimated tax) and saves through `receipts.import`.
  2. A photo without a QR and an Argentine receipt go to the pre-filled `PurchaseDialog`.
  3. "New purchase" opens the empty `PurchaseDialog`, with the **product category** on each
     line, and saves through `food.purchase_save`.
- **Two classifications:** `receipts::classify` gives the `ItemGroup` from the description;
  `food::auto_category` translates group + description into a category; the manual purchase does
  the inverse (`purchase::group_of`). The MCP tool `get_purchases_summary` reads the summary by
  group (`receipts.summary`); the screen reads the summary by segment (`food.summary`).
- **Editing:** only the manual purchase. An NFC-e receipt can only be deleted.
- **Transaction:** when linking, the sum of the items must equal the amount. The dialog opens on
  "create the transaction too", on the first account in the list.
- **Product:** a `datalist` of names; it only fills in unit, price and category when the name
  is identical.
- **Outside Brazil:** `classify.rs` and `seed.rs` are Portuguese expressions; `pack.rs` reads
  kg, g, L, mL and "C/12"; `basket::official_series` only has `ipca_food_home`;
  `receipts::foreign` only reads AFIP; `receipts::without_tax` zeroes the tax of every item;
  `ocr/lines.rs` treats "tax" as a line to skip and checks `sum − discount = total`.

## 2. Stage A: simplify (no migration)

### A1. A single review

Every reading (link, PDF, XML, photo, QR from another country) and manual entry open the same
screen, the **Purchase**, with three states:

- **Tax document** (has an access key): description, quantity, price and total locked; the
  person changes category, member, store and transaction. A padlock in the table header
  explains it.
- **Reading to check** (photo, pasted text): everything editable, with the "checked / balanced /
  did not balance" banner that already exists and the photo alongside.
- **Typed**: everything editable, an empty line ready.

Sketch of the screen (the same regions in the three states):

```
Compra                                                   [foto ao lado, se houver]
Data [16/11/2026]   Loja [Mercado Exemplo ▾]   Membro [do lançamento ▾]

Item                     Qtd        Preço     Total    Categoria
Arroz tipo 1 5 kg        1 un       24,90     24,90    Mercearia · já comprado
Patinho moído            0,734 kg   39,90     29,29    Carnes · pela descrição
[buscar ou digitar…]

Soma 54,19   Desconto [0,00]   Imposto somado [0,00]¹   Pago 54,19
Lançamento: (•) Mercado Exemplo, 16/11, R$ 312,47 · faltam R$ 258,28 sem detalhe
            ( ) criar junto   ( ) sem lançamento
                                                        [Cancelar] [Gravar]
```

¹ Only in countries where the tax is added at the end of the receipt (section 5).

- Removed: `PreviewDialog`, the per-item group selector, `useItemGroups`, `receipts.groups` and
  `receipts.set_item_group`.
- Core: `receipts.import` now accepts the category per line (like `food.purchase_save`) and
  gains `receipts.update` for the saved receipt (category, member, store, transaction; never the
  tax document's values).
- Acceptance criterion: the same test NFC-e saved through the new path gives the same items,
  products, tax and payment as it does today (equivalence test in the core before deleting the
  old path).

### A2. A single classification

The product category is the only classification the person sees. The `ItemGroup` stays only as
the **Brazilian tax group**, derived from the category.

- A single classifier, `food::classify(description, store, gtin) → (category, reason)`, in this
  order:
  1. GTIN already linked to a product → its category (`known`).
  2. Store alias (code or description) → the product's category (`known`).
  3. Category from the GTIN open data, if the source is on (`open_data`, D1).
  4. Word list of the language (`words`): the expressions of `classify.rs` and `seed.rs`
     together, one list per seed category.
  5. Nothing: no category (`none`). The item goes into Other and shows up in "Review".
- The tax group comes from the category: each seed category declares its own; one created by
  the person inherits its parent's. `receipt_item.item_group` is still saved (no migration), and
  `tax_burden` does not change.
- `receipts.summary` is now computed by `food.summary`: tax by segment and by member. The tax
  card (`PurchasesCard`) goes into the Overview, only where there is tax per item. The MCP tool
  `get_purchases_summary` keeps its name and returns segments (the agent's catalog and the
  `ai::prompt` test follow).
- Acceptance criterion: on the current fixtures, every item lands in the same category as today
  (a test that compares the new classifier with the group → category path, before deleting the
  old one).

### A3. A single entry point

In Purchases, **Add purchase** opens a small dialog: drop or choose the file (photo, PDF, XML),
paste the QR link, paste the text of the items, or type. The reading steps (`ReceiptReader`)
appear inside it. The toggled panel goes away. Command search gains "Add purchase", and dropping
a file anywhere on the Purchases tab opens the same dialog.

## 3. Stage B: faster entry (no migration)

### B1. Itemize from the transaction

On expenses in Transactions, the row menu and the transaction dialog gain **Itemize**: the
Purchase opens linked to that transaction, with the date, the member and the store (the
transaction's cleaned description; if there was already a purchase linked to a transaction with
the same description, that purchase's store). A transaction row that has items shows an icon
that opens the purchase.

### B2. Itemize only part

- New rule: `paid for the purchase ≤ transaction amount`. The difference appears as "R$ 258,28
  without detail" and stays in the transaction's category.
- Several receipts on the same transaction remain valid; their sum does not exceed the amount.
- In the summary, the part of a supermarket transaction with no detail goes into "supermarket
  without receipt", just as a whole supermarket transaction without a receipt does today. The
  basket index only uses items and does not change.
- A purchase created together with the transaction is still exact (the transaction is born with
  its value).

### B3. Search for the product

- The description is a search with a list (the search component that the Transactions categories
  already use): it matches by a piece of the name without accents, by the store alias and by the
  barcode; each result shows the last price, the unit and the store; those from the chosen store
  come first.
- Choosing fills in unit, price and category. Typing something new creates the product on save.
- Quantity and unit in one field: "0,734 kg", "2", "500 g". With no unit, it is `un`.
- Keyboard: Enter goes to the next field; Enter on the total of the last line creates another;
  the empty line at the end is ignored.
- Total and price: typing one calculates the other (today only price → total).

### B4. A category you can see

The line shows the predicted category in faded text, with the reason ("already bought", "from
the description", "open data"). The `food.guess` command returns the result of `food::classify`
(A2), the same one used when saving. With no guess: "no category", and the person chooses.

### B5. Repeat a purchase

- In the purchase list, **Repeat**: a new purchase with that one's items, the same store,
  today's date and the latest prices of each product, all editable.
- In the Purchase, when choosing a store already used: "start from the last purchase at this
  store".

### B6. Existing payment first

- If there is an expense transaction with an equal amount, or a larger one (B2), within 3 days
  of the date: it opens on "link", with the most likely one selected (equal amount before larger
  amount; same store before another).
- With no candidate: "create together", on the account of the last purchase at that store.

### B7. Paste a list

"Paste items" accepts one item per line ("Arroz 5kg 24,90", "2 x leite 5,49") and runs it
through the OCR line reader (`receipts/ocr/lines.rs`, which already uses the country's decimal
separator), with the same check. It serves for the receipt that arrives by e-mail, a store's
order history and a spreadsheet.

### B8. Reminder in To solve (only for those who use it)

With at least one purchase recorded in the last 90 days: once a month, "4 supermarket purchases
this month without items", with the action to itemize the first one. Someone who never used
Consumption sees nothing.

## 4. Design common to all countries

What changes from one country to another goes into a **consumption profile**, built-in and
tested data (in the style of `country/catalog.rs`), instead of scattered `if`s:

- **Tax on the receipt**: `included` (Brazil, Europe, Argentina, Chile: the price already has
  the tax) or `added` (US and Canada: the tax is added at the end).
- **Tax per item**: `estimated` (Brazil), `printed` (what the receipt prints, apportioned) or
  `none`.
- **Units**: `metric` or `us` (lb, oz, fl oz, gal).
- **Language of the words** of the classifier.
- **Document reader**: `nfce`, `afip`, `atcud`, `verifactu` or none.
- **Official food series** (and the fallback).
- **"High in" label**: `anvisa` or none.

`useCountryFeatures()` now exposes this profile; the interface and the core ask it.

### Reconciling a receipt with added tax

A single rule for the line reader and the Purchase:

```
pago = soma dos itens − desconto + imposto somado
```

- `included`: added tax is always zero (today's rule).
- `added`: the reader now **reads** the tax line ("TAX", "SALES TAX", "TAX 1 7.25%") instead of
  skipping it, and the subtotal line. It checks `sum − discount = subtotal` and
  `subtotal + tax = total`. Both calculations balancing counts as the second check.
- Saving with no migration: `receipt.tax_total` receives the printed tax and `receipt.paid` the
  total with it. The difference `paid − (items_total − discount)` is the added tax.
- The price per unit and the index use the price **without** the added tax (the shelf price), so
  each city's rate is not mixed with the price change.

## 5. Stage C: United States (no migration)

### C1. The US receipt

There is no tax document or public QR: the path is the photo, the PDF and the text of the
digital receipt. What the reader needs to learn:

- **Added tax** (section 4). Without it, nothing reconciles.
- **Tax mark per item**: the letter after the price ("T", "F", "N", "A", "B"). The reader
  already recognizes the letter (`TAX_LETTER`) and discards it; it now keeps it. The printed tax
  is apportioned only across the items marked as taxable. With no legible mark, the
  apportionment goes to the items outside Food (in most states food for home is exempt or has a
  reduced rate, and `tax_burden/data/us.json` already carries that rule by state).
- **Variable weight**: "2.31 lb @ 3.99/lb" (the quantity pattern already accepts `@` and `lb`;
  `oz` and the price per unit with the slash are missing).
- **Discount on the line below**: a manufacturer coupon, a store card discount and "you saved"
  come as a negative line after the item. It becomes `receipt_item.discount` of the item above;
  the savings summary in the footer is not added again.
- **Lines that are money but not a product**: bottle deposit, bag fee, environmental fee. They
  go in as items of the Other › Fees and deposits category (a new seed category, outside the
  basket index).
- **Split payment**: a food benefit plus a card on the same receipt already fits in
  `receipt_payment` (one payment per transaction).
- **Date** mm/dd/yyyy and total "BALANCE DUE", "AMOUNT DUE" (the total is already recognized).
- **Tip**: a restaurant is not a Consumption item; "eating out" keeps coming from the
  transactions.

### C2. Product code

- **UPC-A** (12 digits) already becomes a GTIN-14 with zeros. Many receipts print the store's
  code, or the UPC without the check digit: only a code with the right digit counts as a GTIN;
  the rest stays as the store code in the alias (today's rule).
- **UPC-E** (8 compressed digits) is only expanded when it comes from the barcode photo, where
  the code type is known; typed, 8 digits are still EAN-8.
- **Produce PLU** (4 or 5 digits, "4011") is a store code in the alias, never a GTIN, like the
  scale item in Brazil.
- UPC prefixes 2, 4 and 5 (variable weight, internal use, coupon) already fall in the refused
  ranges.

### C3. Units

- `pack.rs` reads lb, oz, fl oz, gal, qt, pt, "12 ct", "12 pk", "pack of 12", "dz".
- Always stored in kg, L or unit (the index does not change). **Shown** in lb and gal when the
  profile is `us`; the conversion is display only, in a single function (`food::units`).
- The Purchase accepts "2.31 lb", "12 oz", "1 gal" in the quantity field.

### C4. Classification in English

The US receipt abbreviates a lot ("GV WHL MLK GAL", "ORG BNNS"). That is why the order in A2
weighs more here: known product, then the open data by UPC (the taxonomy is the same worldwide),
then the English word list with the common abbreviations. What is left stays "no category" in
Review, and the person's choice holds for the next purchases at that store.

### C5. Official index

- "Food at home" series of the CPI through FRED (`CUSR0000SAF11`), the provider `us_cpi` already
  uses. It requires the free key that the person registers in Settings.
- Without the key: the CPI food group through the OECD (the provider of `cpi_us_oecd`, no key),
  with the name stated on screen (it includes non-alcoholic beverages).
- Later, if requested: the **national average prices** per item (a dozen eggs, a gallon of milk,
  a pound of ground beef), so the person can compare the price they pay. Left out of this stage.

Both series come in together with those of the other countries in V164 (stage D); until then the
screen compares with the full index and says so.

### C6. Tax and health

- The printed, apportioned tax goes into 10 · Taxes in place of the state-rate estimate for
  those purchases (`without_tax` stops zeroing when the profile is `printed`).
- There is no mandatory front label: the Health tab shows nutrients without a label, as in the
  other countries outside Brazil.

### C7. Tests

Fixtures with only the **text** of invented receipts: with and without tax, with a tax mark,
with weight in lb, with a discount coupon on the line below, with a bottle deposit, with a split
payment. The US demo vault gets purchases like these over the last three months.

## 6. Stage D: other countries (V164, data only)

### D1. Classification by language and by open data

- Word lists as built-in data (one file per language, in the style of
  `income/payslip_specs.rs`): Portuguese and English (done in A2 and C4), then Spanish; French
  and German on request. The language comes from the country profile, with Portuguese as the
  fallback.
- A single map of the Open Food Facts categories (`en:dairies`, `en:breads`…) to the seed
  categories, used when the source is on and the item has a GTIN. Suggestion only.

### D2. Official food index

`basket::official_series` now reads the profile. V164 adds the series (rows in `index_series`
and in the country profile), through the providers that already exist:

- US: section C5.
- Portugal, Spain, France, Germany: food HICP from the ECB.
- United Kingdom (ONS), Canada (Bank of Canada) and Chile (OECD): the food series, if it exists
  at the same provider.
- Argentina and the rest: the full index, stated on screen.

Each series is checked (code, unit, first point) before it goes in, like `ipca_food_home`.

### D3. Fiscal QR of Portugal and Spain

Like Argentina: the QR does not carry the items, but it carries the date, the store's tax
identification and the total (in Portugal, also the VAT by rate). Read on the computer, with no
lookup, it is the OCR's answer key and the key against a duplicate receipt.

- Portugal: the invoice QR (ATCUD), text with fields separated by `*`.
- Spain: the VeriFactu QR (and TicketBAI), a link with NIF, number, date and amount. The app
  **does not open** the link; it only reads the fields.

The layouts are checked against the official standards before implementing; tests with invented
QRs.

### D4. Canada

It uses the same `added` profile as the US (added tax, sometimes two: federal and provincial,
read as two lines and summed) and metric units. It comes almost for free after stage C.

## 7. Stage E: small cuts

- `Product.gtin` (a single code) leaves the type; the interface already uses `gtins`.
- The consumption mock shrinks with A1 and A2; new tests outside `src/api/mock/*.ts`.
- `consumption.md`, `receipts.md`, `gtin-inflation.md` and `ocr-receipts.md` are updated at each
  stage.

## 8. What I do not propose

- **Agent classifying each item**: what the person chose, the open data and the word lists cover
  the common case without taking data off the computer.
- **Automatic merge by similar description**: stays a suggestion only.
- **Reading receipts directly from the person's account at stores**: it would require logging in
  to third-party sites. The pasted text (B7) and the forwarded e-mail cover the case.
- **Shopping list, public prices, basic basket**: another front.
- **New MCP tool for a product's price**: `get_basket_inflation` gains the optional `product`
  argument, without growing the catalog.

## 9. Risks

- A1 and A2 touch the saving of the NFC-e, never tested with a real receipt: hence the two
  equivalence tests before deleting the old path.
- A2 changes the format of `get_purchases_summary` (segments instead of groups).
- B2 changes the rule "the sum must match" of consumption.md.
- C1 depends on the OCR quality on long thermal receipts; the double check (subtotal and total)
  is what prevents saving a wrong reading.
- I have no real US receipt to test with: the fixture layouts are invented from the common
  format, and the first real photo may call for adjustments.

## 10. Order and size

1. `feat/consumo-simples`: A1, A2, A3. The largest; touches `receipts`, `food`, MCP and three
   components.
2. `feat/consumo-cadastro`: B1 to B8, stacked.
3. `feat/consumo-eua`: consumption profile (section 4) and C1 to C7, stacked.
4. `feat/consumo-paises`: D1 to D4 with V164.
5. E along with the stage where each cut fits.

No new crate or package. No release build.

## 11. Questions

1. **Order**: A, B, C, D as above? The other option is B1 and B2 first (what changes the day to
   day the most) or C before B (if the US is the priority).
2. **B2**: may I replace "the sum must match" with "the purchase does not exceed the
   transaction amount"?
3. **US, units**: is always showing lb and gal (simple and comparable) fine, or do you prefer oz
   and fl oz on small packages?
4. **US, index**: is it fine to depend on the free FRED key for "Food at home", with the OECD
   series as the fallback?
5. **The four old decisions in consumption.md** (the name "Consumption", all countries, purchase
   without a transaction and non-food classes) were implemented with my recommendations and
   never got an answer. Do I consider them confirmed?
