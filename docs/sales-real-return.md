# Sales: gain or loss in money, nominal and real

The sales ledger shows, for each asset sale, how much was gained or lost in money, with the
option to discount inflation (IPCA and other indices) or monetary expansion (M2), and how long
the money stayed invested. The Taxes screen keeps the income tax (IR) calculation, at average price.

## Sales ledger

For each sale, `portfolio.sales` returns:

| Field | What it is |
|---|---|
| date, asset, account, quantity | the trade |
| net sale value | price × quantity − costs |
| cost | by the country's cost method; in Brazil, average price × quantity, the same as the IR ([cost-method.md](cost-method.md)) |
| nominal result | sale − cost, in R$ and in % |
| adjusted cost | the cost brought to the sale date by the chosen deflator |
| real result | sale − adjusted cost, in R$ and in % |
| average holding period | days between the purchases and the sale, weighted by cost |
| dividends and "with dividends" | the dividends received on the units sold and the result added to them (see below) |
| IRR | annual rate of the sale ([cost-method.md](cost-method.md)) |

**What is not a sale.** A trade that only aligns the recorded quantity with what the institution
shows, with no money moved, is a **position adjustment**: the difference booked by the position
on a date (`portfolio.bulk`), or a trade marked with `adjustment` (`portfolio.create_trade`,
`portfolio.update_trade`; `record_trade` and `update_trade` for the agent). It moves the quantity
and the cost like any trade, shows as "Position adjustment" in the lists, has no cash effect and
stays out of the ledger and of its totals. A real sale recorded without a cash entry (the
statement comes later) is still a sale. The mark is kept in the trade's `external_id`
(`posicao:…` or `ajuste:…`), without a column of its own.

**How the cost is adjusted.** At average price, each sale takes the same fraction of all
the purchases that formed the position. The ledger keeps those layers (date and cost of each purchase)
and, at each sale, takes the sold fraction from each one. The adjusted cost is the sum of each
piece multiplied by the inflation between the purchase date and the sale date. It is exact for
average price and does not depend on FIFO.

**Invented example.** In 2024-01, a purchase of 100 shares at R$ 10 (cost of R$ 1,000). In
2026-01, a sale at R$ 11 (R$ 1,100). The nominal gain is **R$ 100 (+10%)**. With IPCA
accumulated at 9.5% in the period, the adjusted cost is R$ 1,095 and the real gain is **R$ 5
(+0.5%)**. With Brazil's M2 growing 18%, the sale **lost R$ 80** in purchasing power
against the monetary base.

Other points of the ledger:

- **Deflators.** IPCA (default), IGP-M, the family country's inflation, US CPI, Brazil's M2,
  US M2 or nominal. It is the same selector as Returns.
- **Dividends.** The dividends received on the units sold (net of withheld IR, excluding
  amortization) form a balance of the position. Each sale takes that balance in the same fraction as the
  cost, counting each dividend by its ex-date: someone selling on the ex-date itself no longer takes it. In real terms, each
  dividend is brought to the sale date by the index. The price result still matches the IR.
- **Fixed income and Tesouro.** They use the FIFO that the portfolio already uses.
- **Dollar assets.** Cost and sale in the base currency, at the exchange rate of each date, as the
  portfolio already does.
- **Totals.** By period, by year, by class and by asset: sales, cost, nominal and real
  result, and how many sales were gains and how many were losses.
- **Estimated data.** When inflation is missing for a month, for example the current month, the
  row shows ≈.

## Screen (Portfolio › **Sales** tab)

- **Filters:** period (year, 12 months, all), class, asset, account and deflator (the same as
  Returns).
- **Top:** total sold, nominal result, real result and the "inflation eaten" (the
  difference between the two), plus gains × losses.
- **Table by sale,** sortable. Hovering over the real result shows the
  calculation: cost, index factor and adjusted cost.
- **Chart by month,** with the nominal and real result as side-by-side bars.
- **Link on each row** to the trade (edit or delete).

## Agent (MCP)

- **`list_sales`:** the same ledger, with the filters and the deflator. It makes it possible to answer
  "how much did I really gain selling ITSA4 this year?".
- **`get_playbook`:** the portfolio guide mentions the ledger.

## In the code

- `crates/treasury-core/src/portfolio/sales.rs` (`portfolio.sales`), interface in
  `apps/desktop/src/features/portfolio/SalesPanel.tsx`.
- Calculated from the trades already stored; IGP-M comes from the BCB (SGS 189). Nothing new is stored
  in the vault.

## Sales without a recorded purchase

An official source (the B3 Investor Area, a brokerage note) can report a sale whose purchase is
older than the history in the vault. The sale is recorded as reported and nothing is made up
for the missing purchase:

- **In the import.** In the B3 preview the row comes with the status `warning` ("pending item")
  instead of an error, ticked by default. The person can still leave it out, or give the opening
  position on the Opening tab, which covers the sale right away. `b3.commit` returns
  `pending_sales`, the sales it wrote that the portfolio has no purchase for.
- **In the numbers.** The portfolio takes from the position only what it had. The part without a
  purchase stays out of the average cost, of this ledger and of the realized result. In the
  monthly IR its value counts in what the month sold (the base of the stock exemption) and the
  month is marked `incomplete`, because its result is unknown.
- **Afterwards.** `portfolio.uncovered_sales` (`portfolio/uncovered.rs`, MCP
  `list_uncovered_sales`) lists those sales straight from the replay of each asset, all accounts
  together. They show on the Sales tab, on the asset page (Trades tab) and in "To solve"
  (`data:uncovered_sales`). For each one the person records the purchase (the trade form opens
  with the account, the missing quantity and the day before the sale), accepts it without a cost
  or deletes the sale. Importing the older history takes the sale off the list by itself.
- **Accepted without a cost.** The sale stops asking and stays out of the results. The accepted
  trade ids live in the `portfolio.uncovered_sales_accepted` setting, inside the vault; there is
  no migration.

Why not an automatic opening purchase: at zero cost the whole sale would become a gain and
inflate the tax; at the sale price the result would be zero and the gap would be hidden; and
both would double the position if the older history were imported later.

## Limitations

- There is no per-row IR estimate: the IR stays on the Taxes screen.
- No comparison with the CDI ("it would have earned R$ X in the CDI") and no equity sale simulator
  (for fixed income there is `simulate_fixed_income_sale`).
- Indices beyond IGP-M (INPC, IPCA-15, INCC) are not available as deflators.
