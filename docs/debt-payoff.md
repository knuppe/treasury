# Debt payoff simulator

Given an extra amount per month, the simulator shows the order in which to attack the debts and compares three paths, month by month:

| Path | How it pays |
|---|---|
| Minimum only | Only the installments, no extra. This is the baseline. |
| Snowball | The extra goes to the debt with the **smallest balance**. Once one is paid off, its installment is added to the extra for the next. Gives quick wins. |
| Avalanche | The same, but the **highest rate** first. Pays less interest in total. |

For each path, the result has the months until payoff, the month when the family is free of debt, total interest, the interest and months saved compared with the minimum, the order and payoff month of each debt, and the outstanding balance summed month by month. The `recommended` strategy is the one that pays the least interest; on a tie, snowball.

## Debts

- **Active loans and financings with a balance** enter with the balance, the contracted rate and, as the minimum, interest + amortization + adjustment of the next installment. Insurance and fees do not enter, because they do not fall with the payoff.
- **Debts outside the app** (overdraft, revolving credit, an installment plan) enter with a name, balance, yearly interest and minimum payment.

## Approximations

The limitations of each contract appear in `notes`:

- the rate is fixed, without the index adjustment (TR, IPCA…);
- the installment stays constant (under SAC it falls over time, so the loan is paid off a little earlier);
- when the minimum does not cover the interest, the debt never ends and the path is `feasible: false`. The simulation stops when the balance reaches 100 times the initial one or passes 50 years.

If the minimum does not pay off the debt, the savings against the minimum are zero and the explanation compares only the two strategies.

## In the code

- Interface: **Debts › Debt payoff plan**. It also appears without contracts, to simulate with only the debts entered.
- Command: `debt.payoff_plan` (`extra_monthly`, `loan_ids`, `custom`). MCP tool: `plan_debt_payoff`.
- Code: `crates/treasury-core/src/debt/payoff.rs`.

It is a simulation, not financial advice.
