# Property tax (IPTU) on real estate

The IPTU bill (carnê) for each of the family's properties or lots, with due-date reminders and what can be derived from the bill (single payment × installments, effective rate, comparison with the previous year and with market value). Only in Brazil vaults. It lives in Portfolio › Physical assets › property or lot › **IPTU** card.

Code: `crates/treasury-core/src/physical/iptu.rs` (migration V126), reminders in `notify` (`iptu_due`, `iptu_missing`), taxes paid in `tax_burden`. UI: `features/physical/IptuPanel.tsx` and `IptuDialogs.tsx`; texts in `locales/*/iptu.ftl`. Outside Brazil the card disappears (`CountryFeatures.iptu`).

## What IPTU is, and what changes between city governments

IPTU is a municipal tax: each city government has its own law, its own generic value map and its own calendar. The bill usually contains:

- **Property registration**: the number changes its name (inscrição imobiliária, indicação fiscal in Curitiba, SQL in São Paulo, índice cadastral in Belo Horizonte). The app stores it as text.
- **Assessed value (valor venal)** of the land and of the building (predial), whose sum is the total assessed value. It is the calculation base, set by the city from the value map; it tends to sit below the market price.
- **Taxes**, territorial and predial (assessed value × the municipality's rate, sometimes progressive) and the total taxes.
- **Fees** charged along with it: public lighting (in some cities only on land without a building, because otherwise it comes on the electricity bill), garbage collection and others.
- **Discounts**: single payment (cota única, paid up front; ranges from 3% in São Paulo to 10% or more in other cities), good payer (those who are up to date), sometimes digital bill.
- **Total due**, the **single payment** and the **installments** (up to 10 or 12, almost always interest-free), with the first due date between January and March.

Sources consulted on 2026-10-19: the glossary of the Joinville bill, the São Paulo IPTU 2026 guide (single payment with a 3% discount until the first installment's due date, up to 10 interest-free installments, fine of 0.33% per day up to 20% plus IPCA + 1% per month) and news from São José dos Campos and Greater Vitória about up-front discounts. The numbers vary by city and year: the app embeds none, it uses what the user copies from the bill.

## What the app stores

One bill per property and year (`property_tax_bill`), with all the fields above (all optional except the total due), the chosen payment method (single payment, installments or "not decided yet") and the installments (`property_tax_installment`, number 0 = single payment), each with its payment (date and, if any, the transaction). Saving the same year again replaces the bill and keeps the payment of the installments that remain.

## What the app derives from the bill

| Feature | How |
|---|---|
| **Due-date reminder** | `iptu_due` alert, one per installment (never repeats), N days before (the same setting as bills, minimum 3). With no method chosen, it reminds only of the first date and invites the person to decide. It disappears if there is a transaction with "IPTU" in the text or in the category with the installment's amount near the due date. |
| **Unregistered bill for the year** | `iptu_missing` alert, once per property and year, 20 days before the previous year's first due date (same day and month), only for properties that already had a bill in the previous year. |
| **Payment linked to the transaction** | Each open installment shows the likely transaction (an outflow of a similar amount, ±2%, from 25 days before to 15 after the due date, first those that mention IPTU); "Confirm" links it. It can also be marked by date. Paying the single payment selects single payment; paying an installment selects installments. |
| **Single payment × installments** | The up-front discount becomes a monthly rate (the one that equates the single payment to the present value of the installments) and is compared with the monthly CDI: if the discount earns more, pay up front; if it earns less, pay in installments and keep the money invested. |
| **Effective rate** | Total taxes ÷ total assessed value. |
| **Against the previous year** | Change in assessed value and taxes, and the IPCA of the previous year's 12 months (the usual adjustment of the value map). Tax rising more than 0.5 point above the IPCA becomes a warning, reminding of the review request within the bill's deadline. |
| **Assessed value × market value** | Against the asset's value in the app (appraisal or estimate) at the start of the bill's year. Assessed value above market becomes a warning (a review can be requested); far below (less than 40%, with a real appraisal) reminds that the assessed value is not a price for the property. |
| **Total check** | Taxes + fees − discounts different from the total due (by more than R$ 1) becomes a warning to check the bill. |
| **Assessed value history** | Year-by-year table with the assessed value, taxes and total. |
| **Taxes paid** (Taxes) | With no IPTU recorded in a category in the period, the IPTU from the bills counts: the chosen method (with no method, the single payment) with a due date in the period, as an "IPTU from the bill" line in the wealth part. |

The agent reads with `get_iptu` (`physical.iptu_list`) and proposes a bill with `record_iptu` (`physical.iptu_save`, reversible), for example from a photo of the bill that the user attached.

## Ideas not done yet

- **Deduction on rent (carnê-leão)**: the IPTU paid by the owner of a rented property can be deducted from the rent received (taxes and fees on the asset that generates the income). It needs to link the property to the rent income source (`income`, rent type).
- **IPTU in the balance forecast and in the budget**: open installments as forecast outflows (today the forecast does not see the bill, only forecast transactions and recurrences).
- **Reading the bill as a PDF**: an `iptu.*` layout like those for payslips and card statements, per city (São Paulo, Curitiba, Belo Horizonte…), waiting for real bills.
- **Exemptions**: retirees, a single low-value property, entities; they depend on each city's law (the app could just remind the person to check).
- **Challenge deadline** in the new bill's reminder (usually 30 to 90 days from delivery), when the assessed value rises above the IPCA.
- **ITBI** (property transfer tax): the assessed value is not the ITBI base (STJ, Tema 1.113: the base is the transaction value); when buying a property, the app could check the ITBI paid against the city's rate.
- **Condominium and IPTU per fraction**: in an apartment, the land area is the ideal fraction; the app stores the bill number without interpreting it.
