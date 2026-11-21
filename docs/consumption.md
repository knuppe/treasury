# Consumption: what the family buys, item by item

The Consumption screen shows the family's purchases item by item: food, cleaning, hygiene, pharmacy,
clothes, electronics and the rest, with the price per unit over time and the inflation of the
family's own basket. Items arrive through the receipt (in Brazil, the NFC-e) or through manual entry (street market,
a store without a receipt, another country). It works in every country, in the family's currency.

Related documents: receipt reading in [receipts.md](receipts.md), OCR of photos in
[ocr-receipts.md](ocr-receipts.md), GTIN, basket index and open product data in
[gtin-inflation.md](gtin-inflation.md).

## Countries

What changes from one country to another lives in the **consumption profile** (`food/profile.rs`, command
`food.profile`):

- **All countries**: the screen, the typed purchase, the photo and the pasted text, products, price per
  unit and categories, in the family's currency.
- **Brazil**: NFC-e through the QR code, PDF or XML (SEFAZ lookup); item tax from what the receipt prints or from the percentage of the consumption group.
- **United States and Canada**: the tax is added at the end of the receipt and spread over the taxable
  items. In the US, pound, ounce, gallon and fluid ounce; the price per unit is stored in kg
  and L and shown per pound and per gallon.
- **Argentina, Portugal and Spain**: the receipt QR gives the date, the issuer and the total
  ([receipts.md](receipts.md)).
- **Classifier words**: Portuguese, English and Spanish.
- **Official food series** to compare with the basket: Brazil, US, Portugal, Spain,
  France, Germany, United Kingdom, Canada and Chile ([gtin-inflation.md](gtin-inflation.md)).

## The rule that avoids counting twice

**The transaction is the money; the receipt is the detail of the transaction.**

- Reports, budget, balance and cash flow add up **transactions**. The receipt total never
  counts as spending.
- Consumption adds up **items**, only from purchases that have a transaction, and shows the "purchases without a
  transaction" separately, with the action to create the transaction.
- A purchase paid in two ways (card and meal voucher) has one payment per transaction
  (`receipt_payment`); several receipts can point to the same transaction, and the preview warns if the
  sum exceeds its amount.
- The receipt discount is spread over the items in proportion to their value, and the leftover cents go to
  the largest item, so the sum matches what was paid.
- Only the access key (or the country's identifier) keeps the same receipt from entering twice; the
  cross-account duplicate detector does not look at receipts.

## Products

- The same product in different purchases is one `product`. Each store describes the product its own way
  (`product_alias`: store code and normalized description), and the GTIN, when there is one, links the
  purchases from different stores ([gtin-inflation.md](gtin-inflation.md)).
- **Order for finding the product**: same GTIN → store alias (code + description) → similar
  description in the same store → new product. The user merges two products or splits one.
- **Package size** comes from the description ("5KG", "500G", "LT 350ML", "C/12", "2L") and from the
  receipt unit (KG on weighed items). From it comes the price per kg, L or unit of each item
  (`receipt_item.base_qty`, `base_unit`, `base_price`).

## Product categories

An editable tree in Settings › Categories › Products (rename, move, create, delete, change the
class), with **segments** at the top:

| Segment | Seed subcategories | Default class |
|---|---|---|
| Food | Produce, Meat, Milk and cold cuts, Pantry, Bakery, Sweets and snacks, Frozen, Beverages, Alcoholic beverages | that of each one |
| Home and cleaning | Cleaning, Household goods, Bed, table and bath | Essential / Complementary |
| Hygiene and beauty | Hygiene, Cosmetics and perfumery | Essential / Complementary |
| Health | Pharmacy (medicine), Baby | Essential |
| Clothing and footwear | Clothes, Shoes, Accessories | Complementary |
| Electronics and appliances | Phones and computers, Appliances, Audio and TV | Complementary |
| Home and decor | Furniture, Decor, Tools and garden | Complementary |
| Education and stationery | Books, School supplies | Essential |
| Leisure | Toys, Games, Sports, Gifts | Superfluous |
| Pet | Pet food, Care | Complementary |
| Automotive | Parts, Accessories | Complementary |
| Cigarettes | — | Superfluous |
| Other | — | no class |

- **Class** (Essential, Complementary, Superfluous): three levels, because "essential or luxury" leaves
  half the cart in the wrong place. A subcategory without a class inherits its parent's, and a product can
  have its own. It is a renamable suggestion, never a judgment; the app does not use phrases like "you
  spent too much on sweets".
- The seed categories are created in the installation's language (`food::seed::ensure`, once per vault).
- The "receipt group" (`item_group`) serves only Brazil's tax and comes from the category; the user
  never chooses it.
- **Predicted category** (`food::classify`, command `food.guess`): the same function serves the
  review and the save, in this order: a product already bought (barcode, store alias or
  the same name); category from the barcode open data, if the source is on;
  words in the description in the country's language (Portuguese, English, Spanish). With none of these, the item
  stays in Other and the row says "no category".

## The Purchase

A single screen for every purchase, in three states: **typed** (everything editable), **reading to
check** (photo or pasted text, with how well the sum closed) and **tax document** (items,
amounts, store and date locked; category, member and transaction to choose, also after
saving). It opens through **Add purchase** (drop the photo, the PDF or the XML, paste the QR link,
paste the items as text or type) and through **Itemize** in the menu of an expense in
Transactions.

1. **Header**: date, store (with the latest ones to choose from; for a store already used, "start from the
   last purchase at this store") and member.
2. **Items**: description, quantity and unit (un, kg, g, L, mL; lb, oz, gal and fl oz where they
   are used), unit price, total and category. The description suggests products already bought with the
   last price, and the barcode typed there finds the product. Enter moves to the next field
   and, at the end of the last row, creates another. "0.734 kg" typed in the quantity splits off the unit.
   Typing only the total calculates the unit price.
3. **Predicted category**: the row shows the category the app will use and where it came from
   ("already bought", "by description", "open data"); the user changes it if they want.
4. **Payment**: link to a transaction, create the transaction along with it (choosing the account) or
   leave it without a transaction. If an expense with the same amount already exists near the date, the screen opens
   on "link" with it selected.
5. **Itemizing only part**: the purchase cannot exceed the transaction amount, but can stay
   below. The difference appears as "R$ X of the transaction stay without detail" and remains in
   its category; more than one purchase can use the same transaction. In the summary, the part without
   detail of a grocery transaction counts as groceries without a receipt.
6. **Added tax** (US, Canada): the field for the tax line at the end of the receipt; the amount paid is
   items − discount + tax.
7. The typed purchase can be edited in full later. If the transaction was born with it
   (`receipt.txn_created`), editing adjusts the amount and deleting the purchase deletes the transaction.
8. **Repeat**: in the purchase list, opens a new purchase with the same items, the same store, today's
   date and the last price of each product.

A user who has registered any purchase in the last 90 days sees in To solve, once a month, the
month's grocery purchases without items ([to-resolve.md](to-resolve.md)).

## The screen

With the month in the header:

1. **Overview**: cards by segment (how much, how many items, change against the previous month);
   food at home × eating out, per person; essential × complementary × superfluous; the products that
   rose most in price per unit and those that weighed most; the six months by segment.
2. **Purchases**: receipts and typed purchases of the period, with search, filter by segment and store, the
   **Add purchase** button and, on each row, Repeat, Edit and Delete. Where there is tax per
   item, the "What the family buys" card by segment.
3. **Products**: filter by segment and search; the product panel shows the price per kg, L or
   unit over time, the stores and the actions (category, class, merge).
4. **Prices** and **Health**: the basket index and the nutritional information
   ([gtin-inflation.md](gtin-inflation.md)).

**Food at home** = food items from purchases with a transaction (discount spread) plus groceries
and bakery without a receipt; **eating out** = restaurants and delivery.

## In the code

- Core: `crates/treasury-core/src/food/` (`mod.rs`: products, aliases, month summary,
  payments; `pack.rs`: package and price per unit; `seed.rs`: seed; `purchase.rs`: manual
  purchase; `gtin.rs`, `basket.rs`, `line.rs`, `open_data.rs`, `health.rs`), `food.*` commands.
- Migrations: V135 (products, aliases, categories, `receipt_payment`), V136 (`receipt` with the
  `manual` origin and `txn_created`), V138 (GTIN, open data, dietary restrictions).
- Interface: `apps/desktop/src/screens/Consumo.tsx` (route `/consumo`; `/alimentacao`
  redirects) and `apps/desktop/src/features/food/`. Tax receipts live only here: the old link to the
  tab that used to exist in Transactions (`/lancamentos?aba=cupons`) opens Consumption › Purchases.
- Migration V164: the official food series of the other countries (data only).
- MCP: `preview_receipt`, `import_receipt`, `get_purchases_summary` (by segment) and `get_basket_inflation`
  (basket inflation, [gtin-inflation.md](gtin-inflation.md)).
- The Brazil demo vault has invented receipts in the grocery purchases of the last three
  months, with the price rising a little each month.

## Limitations

- Splitting the transaction by receipt groups (Food, Cleaning, Hygiene) does not exist; the
  transaction keeps its own category.
- Shopping list, the DIEESE basic basket and public state prices do not exist.
- Receipt readers for other countries only read the header; items come from OCR or manual
  entry ([ocr-receipts.md](ocr-receipts.md)).
- Fees that are money but not a product (bottle deposit, bag fee) enter as
  items in Other; the discount printed on the line below an item goes to the receipt's
  discount, not to the item; UPC-E (8 compressed digits) is not expanded.
- The same receipt from another country read twice through the QR is not blocked (only the NFC-e, by its key).
- None of this was tested with a real receipt from outside Brazil: the fixtures are invented texts in the
  common format.
