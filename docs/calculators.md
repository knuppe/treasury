# Calculators

The Calculators screen is for testing an idea without touching the vault: nothing is saved, and the
calculations run instantly, in the interface (`apps/desktop/src/features/calculators/math.ts`, with tests
in `math.test.ts`). It is useful for people who are just starting and have no data in the app yet.

## The calculators

| Tab | Inputs | What it shows |
| --- | --- | --- |
| Compound interest | initial amount, monthly contribution, yearly return, term, inflation, management fee | final balance, the same in today's money, total contributed, return, cost of the management fee and the year-by-year evolution |
| Living off income | monthly spending (in today's money), return above inflation, current net worth | the net worth that pays the spending from the return alone, how much of the spending the current net worth already covers and how long it lasts while withdrawing the spending |
| Cash × installments | cash price, installment amount and number, first installment today or not, return on the money | hidden interest in the installments (per month and per year), installment total, installments in today's money and which option costs less today |
| Rent × buy | price, down payment, purchase costs, financing interest and term, Price or SAC, rent and adjustment, appreciation, maintenance and property tax, investment return, inflation, horizon | net worth of the buyer and of the renter at the end (and in today's money), property value, outstanding balance, first installment and interest paid |

## Calculation rules

- An annual rate becomes monthly through the equivalent rate, (1 + a)^(1/12) − 1, never by dividing by 12.
- Contributions at the end of each month. The management fee comes out of the return:
  (1 + return) × (1 − fee) − 1.
- "In today's money" divides by the inflation accumulated over the term.
- Living off income is the simple version (a perpetuity with the real return). It does not consider INSS,
  taxes or spending that changes with age; the screen leads to the full retirement plan.
- The hidden interest in the installments is the rate that equates the present value of the installments to the cash
  price (bisection). No interest, when the installments add up to at most the cash price.
- Rent × buy: both paths spend the same money each month. The buyer pays the installment
  and the property costs. The renter pays the rent and invests the down payment, the purchase costs and the
  difference of each month; if the rent goes above the buyer's cost, it is the buyer who invests
  the difference. The rent is adjusted once a year.

## Official references

The `indices.references` command (core, `indices/references.rs`) returns, only from what has already been
collected and each with its date:

- the reference country's basic interest rate (the Selic target in Brazil), from the catalog's interest series;
- the CDI of the last business day, annualized over 252 days (Brazil only);
- inflation over the last 12 official months (IPCA in Brazil, the country's inflation series in
  the others); projections are not included;
- savings under the Central Bank rule: 0.5% per month with the Selic above 8.5% a.a., otherwise 70% of the
  Selic, without the TR (Brazil only).

The rates never change on their own: they appear at the top of the screen and in "Use …" buttons next to each
field, which only fill in when the person clicks. With no indices collected, the screen asks for the rates by hand.

## Cautions

- The fields' initial values are just examples so the calculation does not start empty.
- Nothing is investment advice (CVM Resolution 20/2021): the rates are the ones the person enters and
  the text compares numbers, without pointing to a product or issuer.
- It works in any country, in the family's base currency.
