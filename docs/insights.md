# What Treasury notices without AI

Many families, especially lower-income ones, do not have access to an AI model (cost, a computer too weak for a local model). The app tries to be useful anyway: rules, simple statistics and official data, calculated on the computer, with the numbers in view and what to do.

Code: `crates/treasury-core/src/insights/` (`reports.insights`; MCP tool `get_insights` for those who have an agent). Interface: the attention insights appear in **"To solve"** ([to-resolve.md](to-resolve.md)), with the dismissal saved in the vault; the informational ones, the good news and personal inflation stay on the Dashboard's **"What Treasury noticed"** card (simple and full; "Dismiss" hides it on that computer).

## Insights

| Insight | How it is calculated | Why it helps |
|---|---|---|
| **Personal inflation** (12 months) | Spending over the 12 complete months by category, each category in an IPCA group (by the words of the subcategory, then of the parent category); the weight of each group in the family's spending × the group's change at the IBGE (SIDRA 7060), against the full IPCA. Only appears with at least half of the spending in a group and IBGE data for the groups | Whoever spends more on food and housing feels a different inflation; it serves to ask for a raise and to adjust the budget |
| **Fees and interest paid** (12 months) | Statement texts (fee, service package, annual fee, interest, IOF, charges, late fee, overdraft, revolving credit) and the fee and interest categories | Money that buys nothing; it recalls the free essential services package (Resolução CMN 3.919) and that overdraft and revolving credit are the most expensive credits |
| **Account in the red** (90 days) | Daily balance of checking accounts; days with a negative balance and the minimum, with the average overdraft rate at the BCB | Recurring overdraft calls for switching to a cheaper credit (Debts › Market interest rates) |
| **Out-of-the-ordinary spending** | Parent category of the last complete month against the median of the previous 6 months (at least 4 months with spending): 40% and R$ 100 above is attention; 30% and R$ 100 below is good news | The median does not get carried away by an atypical month; it shows where to look first |
| **Small purchases** | Purchases of up to R$ 30 in the last complete month: with 15 or more adding up to R$ 200, the total and the categories that weighed most | Spending that seems small one by one |
| **Spending above income** (3 months) | Income × expenses of the last 3 complete months | The difference came from saved balance, overdraft, card or loan |
| **Essential × rest × surplus** (3 months) | Expenses in categories marked as essential × the others × what was left of the income, against the 50/30/20 reference | Shows whether the problem is in the fixed bills (renegotiate) or in the rest (a cap in the budget) |

Also without AI, in other parts of the app: ready-made rules for common statement texts (`rules::builtin`), category suggested from the history (`transactions.suggest`, review routine), detection of transfers between accounts, subscriptions and new charges (`subscriptions`), balance forecast and "how much can I spend today" (`forecast`), payoff plan (`debt.payoff_plan`), market interest rates by bank (`debt.market_rates`), emergency fund from essential spending (`goals.emergency_reserve`), financial health (`health`), reminders (`notify`) and the agent panel's suggestions (`ai::suggestions`), learned rules and annual expenses in the forecast ([learned-rules.md](learned-rules.md)).

## Ideas not done yet

- **Category by the CNAE of the CNPJ** of the payee (Pix and boleto carry the CNPJ): it would require the Receita's CNPJ database (several GB) or an external service; see [data-sources.md](data-sources.md).
- **Full seasonality**: essential annual expenses already enter the balance forecast; the credit card, annual income (13th salary) and the budget are missing.
- **Comparison with the city's basic basket** (DIEESE): no public API.
- **Rights and benefits** (social energy tariff, exemptions): depends on income and registry data the app does not have; risky to get wrong.
- **Alert for the insights**: today they stay on the Dashboard; the attention ones could become a notification (with the preferences in Settings › Notifications).
