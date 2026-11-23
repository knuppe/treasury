# Treasury documentation

The documents describe Treasury as it is: what each part does, the rules, where it lives in the
code and what it does not do yet. They are written in English. To start, read the
[vision](vision.md), the [architecture](architecture.md) and the
[development](development.md) guide.

## Document standard

Each document follows the same form:

1. **Title** with the name of the feature.
2. **Opening** of one or two paragraphs: what it is and where it appears in the app.
3. **Body**: how it works, rules and calculations, data and sources.
4. **In the code**: modules, commands, MCP tools and migrations.
5. **Limitations**: what it does not do or does not do yet.

A few more rules:

- Documentation is written in English. A document may have an optional copy in another
  language next to it, named `<name>.<lang>.md` (like `README.pt-BR.md`); the English file is
  the reference.
- Screens are cited by name (Portfolio, Budget), without the menu number, which changes when a
  screen is added. The list is in [design.md](design.md#screens).
- Dates appear only when they say until when a piece of data is valid (a 2026 table, a source checked
  on a given day), not when something was requested or delivered.
- Examples use made-up data; no real document of anyone goes in here
  ([development.md](development.md#personal-data)).
- Competitors and analysis sites are not cited by name; data sources and APIs the app uses
  are (D31 in [decisions.md](decisions.md)).
- Tables only for short data (numbers, names, yes/no), with few columns. Long text or
  many columns become a list, with one item per row of the old table.
- File names in English and lowercase, with a hyphen between words (`asset-score.md`). Once
  created, the name stays stable, because the code cites it in comments.
- What applies to a single country (rules, sources, bank and government formats) lives in the
  country's folder, like [brazil/](#brazil-only-brazil). Someone living in another country, or an agent working
  on a general part of the app, can skip the folder. A document that covers several countries stays in the
  root and says what changes in each.

## Product

| Document | Contents |
|---|---|
| [vision.md](vision.md) | Problem, goals, out of scope, modules and principles |
| [decisions.md](decisions.md) | Architecture and product decisions in force |
| [roadmap.md](roadmap.md) | Status and next steps |
| [design.md](design.md) | Visual direction, themes, components, visualizations and screens |
| [profile.md](profile.md) | Experience levels and welcome questionnaire |
| [countries.md](countries.md) | The family's country and what changes in each |
| [international.md](international.md) | What goes away and what comes in outside Brazil |
| [i18n-design.md](i18n-design.md) | Languages and catalogs |
| [terms.md](terms.md) | Terms of use and where the notices appear |
| [support.md](support.md) | How the app asks for support |

## Architecture, development and distribution

| Document | Contents |
|---|---|
| [architecture.md](architecture.md) | Stack, processes, storage, backups, data model, calculations |
| [development.md](development.md) | Structure, commands, tests, build, conventions, personal data |
| [distribution.md](distribution.md) | What goes in each release and the step by step |
| [trust.md](trust.md) | Who publishes Treasury, how that is guaranteed and how to check |
| [auto-update.md](auto-update.md) | Signed automatic update (minisign and YubiKey), in all packages |
| [background-tasks.md](background-tasks.md) | Background collection and calculations |
| [invariants.md](invariants.md) | Money invariants and validity of the embedded tables |
| [demo.md](demo.md) | Demo vault |

## Security

| Document | Contents |
|---|---|
| [security.md](security.md) | Threat model, encryption, AI and MCP, webview, Downloads, email |
| [local-security.md](local-security.md) | Malicious programs on the same machine |
| [unlock-options.md](unlock-options.md) | Unlock and recovery |
| [backup-recovery.md](backup-recovery.md) | Recovering and copying backups |
| [vaults.md](vaults.md) | Multiple vaults |

## AI agent and MCP

| Document | Contents |
|---|---|
| [ai-agent.md](ai-agent.md) | Manager agent, MCP server, tools, permissions, engines |
| [mcp-extras.md](mcp-extras.md) | MCPs from other programs for a vault's agent |
| [pdf-markdown.md](pdf-markdown.md) | PDF to Markdown for the agents |
| [inbox.md](inbox.md) | File inbox folder |

## Everyday money

| Document | Contents |
|---|---|
| [members.md](members.md) | Finances per family member |
| [income.md](income.md) | Income, payslips and income forecast |
| [account-freshness.md](account-freshness.md) | Accounts up to date and possibly incomplete balance |
| [design/cartoes.md](design/cards.md) | Cards: additional cards, limit, costs, cashback, points |
| [design/parcelas-detectadas.md](design/detected-installments.md) | Installments detected from the description |
| [learned-rules.md](learned-rules.md) | Learned rules and annual expenses in the forecast |
| [subscriptions.md](subscriptions.md) | Subscriptions and recurring charges |
| [forecast.md](forecast.md) | Balance forecast and how much to spend today |
| [notifications.md](notifications.md) | Reminders and system notifications |
| [to-resolve.md](to-resolve.md) | To resolve, next step and closing the month |
| [insights.md](insights.md) | What the app notices without AI |
| [health.md](health.md) | Financial health |
| [goal-sources.md](goal-sources.md) | Goal sources |
| [design/meta-faculdade.md](design/college-goal.md) | College goal for the children |
| [scenarios.md](scenarios.md) | "What if" scenarios |
| [calculators.md](calculators.md) | Calculators |

## Import

| Document | Contents |
|---|---|
| [layouts.md](layouts.md) | File identification and layout versions |
| [card-statement-pdf.md](card-statement-pdf.md) | Card statement in PDF |
| [email-ingest.md](email-ingest.md) | Ingestion email |
| [ocr-other-documents.md](ocr-other-documents.md) | Boleto, Pix, receipts and text from images |

## Consumption

| Document | Contents |
|---|---|
| [consumption.md](consumption.md) | Purchases item by item, products and categories |
| [receipts.md](receipts.md) | Fiscal receipts (NFC-e) and tax per item |
| [ocr-receipts.md](ocr-receipts.md) | Receipts by photo (OCR) |
| [gtin-inflation.md](gtin-inflation.md) | GTIN, basket inflation and open product data |
| [consumption.md](consumption.md) | Old name of the Consumption screen (redirects) |

## Debts and assets

| Document | Contents |
|---|---|
| [debt-payoff.md](debt-payoff.md) | Payoff simulator |

## Investments

| Document | Contents |
|---|---|
| [investments.md](investments.md) | Portfolio, indicators, exchanges, risk and retirement |
| [indicators.md](indicators.md) | Fundamental indicators from official sources, input validation and the audit |
| [real-return.md](real-return.md) | Real result on the cards |
| [portfolio-compare.md](portfolio-compare.md) | Compare with any series, period and cards |
| [sales-real-return.md](sales-real-return.md) | Sales ledger, nominal and real |
| [cost-method.md](cost-method.md) | Cost method by country and IRR per sale |
| [scheduled-contributions.md](scheduled-contributions.md) | Scheduled contributions and portfolio by goal |
| [concentration.md](concentration.md) | Concentration and asset tags |
| [asset-page.md](asset-page.md) | Asset page |
| [asset-overview.md](asset-overview.md) | Asset overview, news and documents |
| [asset-score.md](asset-score.md) | Health score per asset |
| [agent-enrichment.md](agent-enrichment.md) | Asset data enrichment and AI Study |
| [design/explorador-de-mercado.md](design/market-explorer.md) | Market: exploring assets |

## Taxes

| Document | Contents |
|---|---|
| [taxes.md](taxes.md) | Estimate of the taxes paid in the year |
| [tax-validation-countries.md](tax-validation-countries.md) | Validation of the US and Portugal taxes and the known divergences |
| [tax-data/](tax-data/sources.md) | Survey of rates and sources (Brazil, states, US, EU) |

## Data sources

| Document | Contents |
|---|---|
| [data-sources.md](data-sources.md) | Prices, indices, exchange rates, fundamentals and the usage rules |

## Life timeline

| Document | Contents |
|---|---|
| [life-expectancy.md](life-expectancy.md) | Life timeline design |
| [life-expectancy-rationale.md](life-expectancy-rationale.md) | Rationale, calculation and how to update |
| [life-expectancy-validation.md](life-expectancy-validation.md) | Validation against official sources |

## Brazil only (`brazil/`)

Rules, sources and formats that only exist for families in Brazil. The app shows these parts only in
Brazilian vaults ([countries.md](countries.md)).

| Document | Contents |
|---|---|
| [income-tax-validation.md](brazil/income-tax-validation.md) | IR validation: rules, sources and official cases |
| [fgts.md](brazil/fgts.md) | FGTS accounts |
| [iptu.md](brazil/iptu.md) | IPTU of the properties |
| [floating-rate-loans.md](brazil/floating-rate-loans.md) | Floating-rate loans and loans indexed to an index (CDI, Selic, IPCA, TR) |
| [brapi.md](brazil/brapi.md) | brapi.dev as a fallback for the B3 price and a secondary source |
| [caixa-statement-by-period.md](brazil/caixa-statement-by-period.md) | Caixa statement that is image only |
| [pension-statement.md](brazil/pension-statement.md) | Pension fund statement |

Brazil's tax data (rates by product and by state) lives with that of the other
countries in [tax-data/](tax-data/sources.md), with the country at the start of the name (`br.md`,
`br_states.md`, `us.md`, `eu.md`), because the scripts in `scripts/tax-data/` read the whole folder.

## Records

| Document | Contents |
|---|---|
| [assessment-2026-10.md](assessment-2026-10.md) | Security and quality review of October 2026 |
