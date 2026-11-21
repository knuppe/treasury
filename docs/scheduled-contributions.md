# Scheduled contributions and portfolio by goal

Contributions that repeat every month into an asset, always confirmed by the person, and the view of the portfolio
by the goals each position is linked to. They live in Portfolio › Trades ("Contributions to confirm")
and in Portfolio › Allocation and rebalancing ("By goal").

## Scheduled contributions

A plan says "buy R$ 500 of asset X, in account Y, every 10th" (day 1–28, start, optional end,
optional and informative goal, pause). Tables `investment_plan` and
`investment_plan_run` (`portfolio/plans.rs`).

- **Nothing is saved on its own.** On the date, the contribution appears in Portfolio › Trades, under
  "Contributions to confirm", with the day's close (or the last one before) and the quantity the
  amount buys: whole units for B3 stocks, FIIs, ETFs and BDRs; two decimals for the Tesouro; six for the
  others. The person adjusts the quantity and price and confirms (it becomes an ordinary trade, with the
  outflow from the brokerage cash) or skips. Confirm and skip both have "Undo".
- Contributions from the last 12 months that have no decision yet appear. The decided ones are kept in
  `investment_plan_run` and do not come back. Outside the Trades tab, a notice at the top of the Portfolio
  reminds how many are waiting.
- Deleting a plan keeps the trades already confirmed.
- Commands: `portfolio.plans`, `plans_pending`, `plan_save`, `plan_delete`, `plan_confirm`,
  `plan_skip`. MCP (read): `list_investment_plans`, `list_pending_contributions`; the agent
  guides the person to confirm in the app.

## Portfolio by goal

The "Goal" classification uses the positions linked to the goals that already existed (whole accounts or
assets, with a fraction or a shared pot; [design/meta-faculdade.md](design/college-goal.md)): each goal with linked
positions receives what they are worth for it today, and the rest of the portfolio goes under "No goal".
Portfolio › Allocation and rebalancing shows the "By goal" card (value, share of the portfolio and how much of the target
is already covered). Command `goals.objectives`; MCP `get_portfolio_by_goal`. The weights are those of the linked
positions, with no table of their own.

## In the code

- `crates/treasury-core/src/portfolio/plans.rs`; tables `investment_plan` and
  `investment_plan_run` (V153).

## Limitations

- The only classification is "Goal"; there are no free-form classifications or rebalancing by
  goal.
- The agent does not propose a contribution plan; it reads the plans and guides the person to confirm in the app.
