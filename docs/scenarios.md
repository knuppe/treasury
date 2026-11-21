# "What if" scenarios

A scenario groups life events (buying a home, having a child, pausing a career, changing income, a one-off expense or inflow, a project) and compares them with the **base scenario**, the one where nothing changes. The calculation runs month by month, in today's real values.

## Base

- **Income and spending per month**: the average of the last 12 full months, by accrual. In a new vault, the average uses only the months with activity. Both values can be overridden in the call.
- **Accounts and investments**: net worth minus physical assets plus debts. Each month's surplus goes here, and a positive balance earns the real return of the retirement plan (default 4% a year).
- **Assets and debts that already exist**: stay at today's value. The current installments are already in the average spending.

## Events

| Type | What it does |
|---|---|
| `one_off` | a single inflow (+) or outflow (−) on a date |
| `monthly` | an extra (+) or lower (−) amount per month, for `months` months or forever |
| `income_pct` | income at `pct`% of normal (0 = career pause, 50 = part time) |
| `asset` | an asset that enters net worth, with optional real appreciation |
| `loan` | financing by the Price table: the amount enters cash and becomes debt, and the installment is paid from the next month |

The **Buy a home** shortcut builds three events:

- `one_off` with the price, negative;
- `asset` with the home's value;
- `loan` with the financed part (price minus down payment), with a term and real interest.

Net worth changes only through interest and appreciation, while cash loses the down payment and the installments. **From a project** uses the budget still to be spent as a one-off outflow on the project's end date.

## Result

- **Paths**: base and scenario month by month, with accounts and investments, net worth, income and spending. Each path carries the month when the available money runs out (`first_short`) and the differences at the end of the horizon (5 to 40 years; default 10).
- **Balance forecast**: the next 12 months of the balance forecast ([forecast.md](forecast.md)) with the cash effect of the events, and the first negative day of the base and of the scenario.
- **Retirement**: with an age in the retirement plan (Budget › Retirement), the saved plan is simulated twice through the planner API (`goals::retirement::simulate`, without changing the plan). In the second run, the scenario's yearly cash-flow difference enters as life events. The result carries the success chance, the net worth at retirement and the sustainable spending in both cases.

## In the code

- Interface: **Budget › Scenarios**, with shortcuts, event list, horizon, charts and saved scenarios.
- Commands: `scenarios.simulate` and `scenarios.list` (read only); `scenarios.save` and `scenarios.delete` (interface only, in the `scenarios.saved` setting).
- MCP tools: `simulate_scenario` (with only `name`, it uses a saved scenario) and `list_scenarios`.
- Code: `crates/treasury-core/src/scenarios/`. Mock: `apps/desktop/src/api/mock/planning.ts` (no retirement).

It is a simulation, not financial advice.
