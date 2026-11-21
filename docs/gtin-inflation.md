# GTIN and per-product inflation

The GTIN (the barcode) links the same product across different purchases and stores, and with it the app
measures the inflation of the family's own basket, product by product, next to the IPCA "Alimentação no
domicílio" (food at home) (or the country's food index). It also finds shrinkflation and, if the person turns it on,
brings name, brand and nutrition information from Open Food Facts. It lives in the Prices and Health tabs of
[Consumption](consumption.md). Examples are invented.

## GTIN as a key

### Why not use the GTIN as the primary key of `product`

The right idea is to treat the GTIN as a **strong natural key**: when it exists and checks out, it decides
alone that two purchases are the same item, in any store and any country. But it does not work
as the product's primary key, for four reasons:

- **Items without a GTIN** are a large slice of the basket, precisely the one that rises most in high inflation:
  meat, cold cuts, produce, bread (scale, prefix 2), street market, manual entry, a receipt read by OCR.
- **One product, several GTINs**: the unit and the case (GTIN-14 with indicator 1–8), the
  promotional pack, the manufacturer that changes the code without changing the contents, the merging of two products
  by the person (which is why a product keeps several GTINs).
- **Reused GTINs**: GS1's rule today forbids reusing the GTIN of consumer goods, but old codes
  were reused, and small stores use invalid codes or codes of another product.
- **Reading errors**: a store code that happens to have a correct digit (a 1 in 10 chance with
  any 13-digit number).

So `product.id` is the internal id and the GTIN lives in `product_gtin`, whose primary key **is the
normalized GTIN itself**, with its own history, without blocking the items that have no GTIN.

### Normalization (GTIN-14)

Always store with 14 digits, padding with zeros on the left. This way EAN-13 `7891234567895`,
GTIN-14 `07891234567895` and the same code read with an extra zero stop being different products.
UPC-A (12) and EAN-8 fit too. Show on screen in the original format (EAN-13 without the zero).

### Codes that are not a product GTIN

After normalizing to 14 digits, the GS1 prefix is in positions 2–4 (the indicator is the 1st).
These codes are not product GTINs: the app refuses them as a GTIN and stores them as a store code in the
alias (`receipts::nfce::valid_gtin`):

| Prefix (after the indicator) | What it is |
|---|---|
| `020`–`029` | restricted circulation within the company (scale, variable weight) |
| `040`–`049` | company internal use |
| `050`–`059` | coupons (US) |
| `200`–`299` | regional restricted circulation (scale in Brazil) |
| `980`–`999` | vouchers, coupons, refund receipts |
| EAN-8 with `0` or `2` | restricted circulation (RCN-8) |
| all zeros | invalid |

Magazines (ISSN, `977`) and books (ISBN, `978`–`979`) are products and stay as a GTIN.

Indicator 1–8 in a GTIN-14 is a **case/bale**: the size comes from the manufacturer, and the price per unit
needs the case quantity ("C/12" in the description, or Open Food Facts). Indicator 9 is variable weight.

The country prefix (789/790 = GS1 Brazil, 560 = Portugal, 84 = Spain, 00–13 = US/Canada…) says
where the **company registered**, not where the product was made. It may appear as a detail ("code
registered in Brazil"), never as "origin".

### Scale items

Codes `2…` carry an internal store code (PLU) and, in most Brazilian stores, the price
or weight embedded. The layout varies by scale and by store, so **do not decode the price**: the
receipt already brings the quantity in KG and the total. What matters is using the PLU as the stable store code in the
alias (as `code`), so that "PATINHO MOIDO KG" is the same product across the purchases
of that store even when the description changes a little.

Linking the ground beef of two different stores is not automatic (there is no GTIN): it is left for the person to merge,
or for the suggestion by similar description within the same category ([consumption.md](consumption.md)).
For the index, that is enough: the comparison is of the same item in the same store, month by month.

### Order for finding the product (updated)

1. Normalized GTIN in `product_gtin` → product.
2. Store alias with the same code (includes scale PLU).
3. Store alias with the same description.
4. (Optional, later) similar description + same category → **suggestion**, never an automatic
   merge.
5. New product.

When step 2 or 3 finds a product and the item brings a new GTIN, the GTIN is **attached** to the product
(the case "the store started sending the cEAN"), as long as the pack size is not different.

## Unit and packaging

What already exists (kg, L, un) is right. Adjustments:

- **Size by GTIN**, not only by product: `product_gtin.pack_qty/pack_unit`. The same coffee in
  500 g and 250 g has different GTINs and a different price per kg; comparing them distorts the index.
  Rule: product = item of one size; different sizes are different products of the same
  **line** (brand + name). The line (`product.line_key`, normalized text) serves for shrinkflation
  ("500G" → "450G" with the price per kg rising) and for "the same coffee in another size".
- **Buy 3 pay 2, bales and cases**: indicator 1–8 or "C/12" → unit `un` with the pack
  quantity; if it is not possible to know, the item stays out of the index (no guessing).
- **Discount**: in high inflation, club promotions weigh. Each item's `vDesc` from the XML, when
  present, becomes the net price (`receipt_item.discount`); with no per-item discount, the receipt's discount
  is prorated by value (marked as prorated) for the index only.
- **Unit check**: if a product's price per kg changes more than 5× from one month to the
  next, it is almost always a misread size. The point stays out of the index and appears in "Review".
- **Precision**: `base_price` in cents per kg is enough (R$ 0.01/kg). For very cheap items
  per unit (screws, small bags) the loss is small; it is not worth changing.

## Your basket's index

### Method

Inspired by what IBGE does for the IPCA, simplified for one family:

1. **Price of the month by product and store**: median price per kg/L/unit of the month's purchases
   (net of discount). Product + store is the elementary unit: the same store over time,
   so that switching supermarkets does not look like inflation or deflation.
2. **Elementary change**: ratio between the month's price and that of the last month with a purchase, up to 3 months
   back (coffee bought every 5 weeks does not disappear). Further back than that, the product leaves until it returns.
3. **Aggregation**: Laspeyres chained month to month, with weights = the family's spending in each category in the
   previous 12 months; within the category, the geometric mean of the products' changes (Jevons).
   Products with no pair in the month inherit the category's average change.
4. **Coverage**: the month only counts if the products with a pair add up to at least 40% of the month's spending
   on purchases with items. Below that the screen says "few repeat purchases this month".
5. **Comparison**: the same period in the IPCA "Alimentação no domicílio" (SIDRA 7060, `c315` 7170, which
   the code already uses) and, by category, the IPCA subitem stored in `inflation_key`. Outside Brazil,
   the country's food index.

### What to show (simple headline)

For someone non-technical, one number and one sentence:

> "The same purchases you made from July to September would have cost **R$ 1,120** a year ago. Today
> they cost **R$ 1,268** (+13.2%). The IPCA for food at home rose 7.9% in the period."

(invented numbers). This is the fixed-basket Laspeyres in plain language.

Below, without needing to open anything:

- **Chart**: your basket × IPCA food, 12-month cumulative (monthly line).
- **What rose / fell the most**: products with the largest change over 12 months, with the price per kg before and
  now and how much that weighed on your pocket (change × quantity bought).
- **By category**: your change × the IPCA subitem (meats, milk, vegetables…).
- **Shrinkflation**: alerts once per product (the existing alerts rule: one per episode).
- On the product (Products tab), the existing history gains the GTIN, the line and the 12-month change.

### Where it lives

**Consumption** screen, Overview tab: "Your basket's inflation" card; new **Prices** tab with the
chart, the lists and the product search. The index also feeds the "personal inflation" in
Reports as an extra line (when there is coverage), without replacing the current one by category.
The index comes out through the `food.basket_inflation` command, used by the screen. The agent and MCP clients
read the same result through the `get_basket_inflation` tool (read only): the basket month by month
next to the official index, the cost of the same purchases today and a year ago, and the products that most
rose and fell. With not enough repeat purchases, the tool says so instead of giving a
number.

### Calculation and cache

Calculated in the core, on demand, from `receipt_item`. With thousands of items per year it is
cheap; no index table needed. If it gets slow, a recomputable monthly cache
(`food_price_month`) can come later without a data migration.

## Several countries

- **GTIN is global**: the same validation and the same restricted prefixes work everywhere
  (in the US, UPC number system 2 = variable weight, 4 = internal use, 5 = coupon; all fall in the
  ranges of "Codes that are not a product GTIN" after normalizing).
- **Currency**: `receipt.currency` (default: base currency). The index uses only purchases in the base currency; a
  purchase on a trip appears in Consumption, but stays out of the basket.
- **Official food index** (V164, codes checked on 2026-11-17; each country's series
  is in `food::profile`, and the indices sync includes the family country's):
  - Brazil: IPCA food at home (SIDRA 7060), subitems in the same table.
  - Portugal, Spain, France, Germany: HICP COICOP 01.1 "Food" from the ECB
    (`HICP/M.<country>.N.011000.4D0.INX`).
  - US: CPI "Food at home" from FRED (`CUSR0000SAF11`), which requires the free key registered
    in Settings; without it, the fallback with no key applies, the OECD CP01 group (food and non-alcoholic
    beverages).
  - United Kingdom, Canada and Chile: OECD CP01.
  - Argentina and the rest: no food series; the screen shows only the family's basket.
  - The series from outside Brazil are level series (index): the basket uses the change from one month to the
    next.
- **Price per unit**: always stored per kg, L or unit, without the tax added at the end of the
  receipt; in the US the screen shows per pound and per gallon (`features/food/units.tsx`).
- **Receipts without a GTIN outside Brazil** (OCR of PT, ES etc.): the index works through the store alias,
  same method.

## Open sources of name and brand by GTIN

| Source | License / cost | Privacy | In the app |
|---|---|---|---|
| **Open Food Facts** (+ Open Beauty/Products Facts) | ODbL, free; read by GTIN with a per-minute limit and a mandatory own User-Agent ([docs](https://openfoodfacts.github.io/documentation/docs/Product-Opener/api/), [conditions](https://support.openfoodfacts.org/help/en-gb/12-api-data-reuse/94-are-there-conditions-to-use-the-api)) | Only the GTIN leaves; the server still sees which GTINs the IP queries | **Used, optional, off by default** |
| Cosmos (Bluesoft) | Token; small free quota, then paid | A personal token ties the queries to an account | Not used |
| GS1 Brasil (CNP / Verified by GS1) | For members | — | Not used |
| Cadastro Centralizado de GTIN (SEFAZ) | Requires a company certificate | — | Not suitable for an individual |

Rules for Open Food Facts:

- Turned on in Settings › Data sources ("Open Food Facts": name, brand, nutrition information,
  NOVA group and allergens by barcode), off by default, with attribution in
  DATA_LICENSES.md.
- Sends **only the GTIN**, never store, date, price, receipt or the family's health data; no account;
  queries each GTIN **once** (asking only for the fields used, `fields=`) and stores the response in the
  vault. Revalidates every 6 months, only for products still bought.
- Queries in a slow batch, in shuffled order and spaced out, so as not to reveal the receipt as a
  block; respects the per-minute limit and sends an app User-Agent ("Treasury/version"), never
  anyone's email.
- What comes from outside is a **suggestion** (like asset enrichment): it fills an empty brand; it does not
  replace what the person edited.
- Never in tests with real data; tests use invented GTINs with a valid digit and invented recorded
  responses.

### What the license allows (checked on 2026-10-22)

Sources: [Open Food Facts terms of use](https://world.openfoodfacts.org/terms-of-use),
[license page of the documentation](https://openfoodfacts.github.io/documentation/docs/Product-Opener/api/tutorials/license-be-on-the-legal-side/),
[ODbL on Wikipedia](https://en.wikipedia.org/wiki/Open_Database_License). This is not legal advice.

| Part | License | What we may do | What we must do |
|---|---|---|---|
| The database (the collection) | ODbL 1.0 | Query, store in the vault, combine with purchases, calculate indices, commercial use included | Give credit; if we **distribute** a derived database (e.g., embedding an extract of the database in the installer), it must go under the ODbL |
| Each data item (name, nutrients, NOVA…) | DbCL 1.0 | Use freely | — |
| Photos | CC BY-SA 3.0 | Show with credit | The app does not download photos |

- **A cache in the vault does not oblige sharing**: the ODbL's "share alike" applies to a derived
  database used publicly; the vault is private and is not distributed.
- **The code's AGPL and the data's ODbL coexist**: different works; the credit goes in
  DATA_LICENSES.md and on the screen.
- **Credit on screen**: on every product sheet with outside data, "Data: Open Food Facts (ODbL) ·
  view the product", with a link to the product page; on the Health tab, the credit in the footer.
- **No warranty and no medical use**: the terms say the data is indicative and not suitable
  for medical purposes; the screen says this in plain words.
- **Open Food Facts logo**: not used (only the name as text).
- **Nutri-Score has an owner** (Santé publique France): the label is not drawn; only the text "Nutri-Score
  X according to Open Food Facts", when there is one. In Brazil, Anvisa's front-of-package labeling applies.

### Health: what the app shows

Everything is an estimate of what was **bought**, not of what was eaten. Consumption › Health:

- **Ultra-processed (NOVA 4)**: the share of food spending with data that went to
  ultra-processed foods, in the last 3 months and a year earlier (the Ministry of Health's Dietary Guidelines
  use this classification).
- **"High in"** added sugar, saturated fat and sodium by Anvisa's limits (RDC
  429/2020 and IN 75/2020): per 100 g, ≥ 15 g, ≥ 6 g and ≥ 600 mg; per 100 ml, half. Brazil only;
  outside it, the nutrients with no label.
- **Protein per real**: grams of protein per R$ 1, from the protein per 100 g and the price per kg.
- **Allergies and restrictions by member** (optional, set in the member's profile in Family):
  a warning when something with an allergen of someone in the household comes on the receipt, or "may contain". Sensitive
  data: it stays only in the vault.
- On the product sheet: NOVA group, "high in", nutrients, allergens and the credit.
- Minimum coverage of 30% of food spending with data to show the parts; below that, the screen
  says there is still little data. A product with no data: "no information"; it is never guessed from the
  description. No moral judgments.
- Fixed footer: "Indicative information from Open Food Facts and the Anvisa limits. It is not
  medical or nutritional advice; check the label."

## Migration (V138)

`crates/treasury-core/migrations/V138__gtin_dos_produtos.sql`:

- `product_gtin (gtin PK GTIN-14, product_id, origin, first_seen, last_seen, created_at)`.
- `product_open_data (gtin PK, source, found, fetched_at, name, brand, quantity, categories,
  nova, nutriscore, per, energy_kcal, sugars, added_sugars, saturated_fat, sodium_mg, protein,
  fiber, allergens, traces, labels, page_url, source_updated)`.
- `member_diet (member_id, restriction, created_at)`, restrictions: gluten, milk, eggs, peanuts,
  nuts, soybeans, fish, crustaceans, sesame, vegan, vegetarian.
- `product.line_key`, `receipt.currency`, `receipt_item.discount`.
- `ipca_food_home` series (IPCA "Alimentação no domicílio", SIDRA 7060/7171, checked on
  2026-10-22).
- Backfill in SQL: GTIN-14 of products and items; duplicates `0789…`/`789…` merged into the
  oldest product (items and aliases move to it); internal or restricted codes are dropped.
  `product.gtin` is no longer read or written.

## Limitations

- With few repeat purchases, the index does not come out: the coverage rule shows a message instead
  of a number with a thin base.
- Automatic merging of products is by GTIN only; by similar description, only a suggestion.
- A misread package size distorts the price per unit; changes above 5× go to
  "Review".
- No data leaves the computer without the source turned on, and only the GTIN goes to Open Food Facts.
- The agent and MCP clients read the basket index (`get_basket_inflation`) and the receipts
  summary (`get_purchases_summary`), but not a product's price history.
