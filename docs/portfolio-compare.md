# Portfolio: compare, period and Returns cards

In the Portfolio, returns can be compared with any series (an asset, an index or the CDI plus an
extra), all tabs use the same period selector, and the Returns tab has the monthly heat map, the
top contributors and dividends year over year. It is only past performance side by side, with no
ranking and no recommendation (CVM Res. 20/2021; CVM is Brazil's securities regulator).

## Period selector

`features/portfolio/components/PeriodSelect.tsx`: `PeriodSelect`, `usePeriod(scope, default)`
and `periodRange(key)`. The options are: year to date, 12, 24, 36 and 60 months, since the start,
and the six previous closed years. Each tab keeps its own choice in `usePortfolioUi().periods`,
in memory only, like the deflator.

| Tab | Scope | Default | What the query receives |
| --- | --- | --- | --- |
| Returns | `rentabilidade` | since the start | `from_month`/`to_month` |
| Risk | `risco` | 36 months | `months` (only 12 to 60, looks back from today) |
| Dividends | `proventos` | 12 months | `from`/`to` (dates) |
| Sales | `vendas` | year to date | `from`/`to` (dates) |
| Result bridge | `ponte` | year to date | `from_month`/`to_month` |

For rolling periods, "12 months" runs from the month 11 months ago to the current one (returns)
or from the same date a year ago to today (dates).

## Compare with any series

`portfolio.benchmark_series` (`portfolio/benchmark.rs`) receives the months of the returns and a
target:

- `{ kind: "asset", id }`: an asset from the registry or the watchlist. Uses the close converted
  to the base currency at the day's exchange rate.
- `{ kind: "index", id }`: a series from the catalog. The calculation depends on the type:
  - level (Ibovespa, S&P 500…): ratio between levels;
  - monthly change (IPCA, IGP-M…): `series_factor`;
  - daily rate (CDI): compounded day by day;
  - annual rate: `annual_rate_growth`.
- `{ kind: "cdi_plus", spread }`: CDI plus `spread` per year. The extra is added per day with
  the rate (1 + spread)^(1/252). Outside Brazil, it uses the country's reference interest rate
  with the extra pro rata by days.

The base is the same as the returns: the end of the month before the first month. The result is
the cumulative value at the end of each month, `null` without data. The screen accepts up to
three series at a time, shown as extra lines in FIG. 01 and as removable chips.

## Cards

- **FIG. 04, heat map:** year × month with each month's return (Modified Dietz). The year's total
  compounds only the months in the period. The last column is the CDI for the same months, taken
  from the cumulative value: (1 + end of year) ÷ (1 + end of previous year).
- **FIG. 05, top contributors:** uses `portfolio.contributions` (`portfolio/contrib.rs`).
  - Each asset's result = end value − start value − purchases + sales + dividends in the period.
  - The points divide the result by the Modified Dietz denominator for the whole period: the
    start value plus the money invested, weighted by the days it stayed invested.
  - Shows the five that added the most and the five that took away the most. The table lists all
    of them.
- **Dividends by month, year over year** (Dividends tab): dividends received, net, in the base
  currency, month by month, for the last three years. A dividend in another currency with no
  stored exchange rate is left out.
