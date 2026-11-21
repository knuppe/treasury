# Asset page

Each asset has its own page, with everything the app knows about it, the agent conversation
focused on it and the saved analyses. It opens at `/ativo/<ref>`, outside the tabs of the Assets screen; `ref` is the
id of the registered asset or the ticker of a security in the universe (the ticker of a registered asset goes
to the id, so there is only one page per asset). The overview, the news and the reading of documents
are in [asset-overview.md](asset-overview.md).

## How it works

- **Breadcrumb by origin.** The app remembers the last screen outside `/ativo/` (with its search and filters).
  Coming from Market, the breadcrumb is "Market › PETR4" and the click goes back to the same selection; coming from
  anywhere else, "Assets › PETR4", which leads to the list. The origin is not in the URL.
- **Old links** (`/ativos?ativo=…&secao=…&estudo=…&dialogo=…` and
  `/ativos?aba=indicadores&ticker=…`) go to the new route with the same parameters.
- **Content**: position, statement, trades, dividends, events, chart, fundamentals
  (indicators, calculated price, history, documents), health, AI study and the record with the sources
  of each piece of data. A security outside the portfolio shows the fundamental analysis and "Watch".
- **Conversation focused on the asset**: "Ask the agent" opens the agent panel beside the page.
  The screen context says the user is on an asset page, and the core adds the
  guidance to focus on it and the right tools.
- **Saved analyses**: a conversation started on the page is linked to the asset
  (`ai_conversation.asset_id`) and appears in the Agent tab of the page, next to the AI studies. Opening
  a conversation continues it in the panel.
- **The agent fills in the data** only through `propose_asset_enrichment`: it becomes a proposal with the
  difference, which the user approves ([agent-enrichment.md](agent-enrichment.md)).

## What the agent has to analyze an asset

| For what | MCP tools |
|---|---|
| Record and position | `list_assets`, `get_position`, `list_trades`, `list_income_events`, `list_corporate_actions`, `get_returns` |
| Price | `get_price_history` (`market.prices`: closes by day, week or month, with change, low and high; up to 400 points) |
| Fundamentals | `get_fundamentals`, `get_fundamentus`, `fair_price`, `compare_assets`, `rank_assets` |
| Documents and news | `list_corporate_documents`, `read_corporate_document`, `get_asset_news` |
| Health | `get_asset_score` |
| Data and sources | `get_asset_data_gaps`, `get_asset_field_sources`, `lookup_asset_reference` |
| Studies | `list_asset_studies`, `get_asset_study`, `start_asset_study` |
| Indices | `get_index_series` |
| Saving what it researched | `propose_asset_enrichment` (proposal) |

`start_asset_study` (`assets.study_request`) is a sensitive write: the request becomes a proposal,
because it spends the study engine, and the user approves item by item. Once approved, the app starts the study
(one at a time, in order; `study::start_requested`). A request whose study notice has not been accepted yet,
for a nonexistent asset or with a long question is refused right away.

## In the code

- Interface: route `/ativo/<ref>` in `apps/desktop/src/app/router.tsx`.
- Migrations: V144 (`ai_conversation.asset_id`) and V146 (`asset_study_request`).
