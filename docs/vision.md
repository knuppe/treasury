# Product vision

Treasury is a desktop application for a family to manage all of its money in one place:
accounts, cards, budget, debts, assets, investments and taxes. The data stays on the computer,
in an encrypted database, and an optional AI agent helps analyze and record, with approval or,
in the autonomous mode the family turns on, with undo in the history.

## Problem

A family's finances are usually scattered: statements from several banks, card statements,
financing spreadsheets, brokerage notes, positions at brokers in the country and abroad. The
existing apps solve only a part, keep the data in a third party's cloud and almost never answer
the question that matters: **"after inflation, did I make or lose money?"**

## Goals

1. **Family cash flow**: where every real goes, by category, member and account.
2. **Debts under control**: loans and financing with a schedule (SAC, Price and post-fixed),
   outstanding balance, interest paid, effective cost and amortization simulations.
3. **Complete investment portfolio**: assets in the country and abroad, several trades per
   asset, average price, realized and unrealized result, distributions, asset analysis and IR.
4. **Real return**: performance discounted by the country's official inflation (IPCA in Brazil)
   and, as an alternative, by monetary expansion (M2), by asset and by portfolio.
5. **AI as a financial manager**: an agent that analyzes, recommends, corrects and records
   through the app's own MCP server, with the engine the family chooses. Every write waits for
   approval, except in the autonomous mode, which the family turns on and in which reversible
   writes take effect immediately and can be undone; what is sensitive always waits for
   approval item by item.
6. **Privacy by default**: local, encrypted data; nothing leaves the computer without an action
   by the person or a data source they can turn off.
7. **Useful for those who know little and those who know a lot**: experience levels and plain
   language, without hiding the advanced features.

## Out of scope

- A phone app or a hosted web version.
- Placing orders at brokers or moving money: the AI recommends and records, but does not trade.
- Cloud sync between devices.
- Company accounting.
- Editorial content: recommended portfolios, analyst reports, courses.
- Automatic login to banks (D20 in [decisions.md](decisions.md)).

## Who uses it

- **The person in charge of the family's finances**: registers accounts, imports statements,
  follows the portfolio.
- **Family members**: they are the holders of accounts, income and transactions, with their own
  reports and profile, with no login of their own (D6).

## Modules

| Area | What it covers | Document |
|---|---|---|
| Accounts and cards | Checking accounts, savings, cash, benefits, brokers, foreign; cards with statements, additional cards, limit, costs, cashback and points | [design/cartoes.md](design/cards.md), [account-freshness.md](account-freshness.md) |
| Transactions | Categories, splitting, tags, attachments, rules, recurrences, installments, spending groups | [learned-rules.md](learned-rules.md), [design/parcelas-detectadas.md](design/detected-installments.md) |
| Import | OFX/CSV, card statements and notes in PDF, B3 reports, payslips, tax receipts, e-mail, inbox, Downloads folder | [layouts.md](layouts.md), [email-ingest.md](email-ingest.md), [inbox.md](inbox.md) |
| Family and income | Members, income, payslips, income forecast, "View as", Life timeline | [members.md](members.md), [income.md](income.md), [life-expectancy.md](life-expectancy.md) |
| Planning | Budget, goals, projects, balance forecast, how much to spend today, subscriptions, scenarios, retirement, calculators | [forecast.md](forecast.md), [scenarios.md](scenarios.md), [subscriptions.md](subscriptions.md), [calculators.md](calculators.md) |
| Pensions and FGTS | FGTS, PGBL/VGBL plans, retirement accounts from other countries and the year's limit | [fgts.md](brazil/fgts.md), [pension-statement.md](brazil/pension-statement.md), [international.md](international.md) |
| Debts | Contracts, schedule, post-fixed, amortization, payoff | [floating-rate-loans.md](brazil/floating-rate-loans.md), [debt-payoff.md](debt-payoff.md) |
| Assets | Real estate, vehicles and other assets, IPTU | [iptu.md](brazil/iptu.md) |
| Consumption | Purchases item by item, price and inflation per product, basket health (D39) | [consumption.md](consumption.md), [receipts.md](receipts.md) |
| Investments | Portfolio, returns, risk, sales, scheduled contributions, asset analysis, score, market | [investments.md](investments.md), [asset-score.md](asset-score.md) |
| Taxes | Brazilian IR, estimate of the taxes paid in the year | [taxes.md](taxes.md), [income-tax-validation.md](brazil/income-tax-validation.md) |
| Financial health | Health score, insights, "To solve", next step | [health.md](health.md), [to-resolve.md](to-resolve.md) |
| Agent and MCP | Manager agent, MCP server, AI engines | [ai-agent.md](ai-agent.md) |
| Security | Encrypted vault, unlock, backups, signed update | [security.md](security.md), [local-security.md](local-security.md) |

## Principles

- **Measured, never invented**: every number shown points to its origin (transaction, quote,
  index, source and date); estimates appear marked.
- **Local-first**: the app is complete without internet; the internet only brings public data
  (quotes, indices, fundamentals, official tables, news), the e-mail the family configures, the
  AI and updates, each with its own on/off switch.
- **Decimal precision**: money never in floating point.
- **AI that acts, on a leash**: the AI can do almost everything the person does, but every write
  is reviewable, reversible and audited.
- **The data belongs to the family**: export in an open format, with no dependence on the app.
- **Brazilian Portuguese first**: the text is born in pt-BR and translated into English and
  Spanish (pt-PT partial, with pt-BR as the fallback).
- **Keyboard first**: everything is reachable through the command bar and shortcuts.
- **Each module is born with its MCP tools**: what the interface does, the agent can do too.
