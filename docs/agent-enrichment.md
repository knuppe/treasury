# Asset data enrichment and AI Study

The data of each asset (sector, segment, CNPJ, ISIN, class, manager…) is completed by cross-checking
sources, official ones first, and each field records where it came from, when and with what confidence.
Every write is a **proposal** per field, with source, date, confidence and undo, and never overwrites
what the person typed; not even autonomous mode applies enrichment on its own. Alongside, the **AI
Study** of an asset: the person gives context and a question, and the agent writes a study with a notice that
it may contain, and probably does contain, incorrect information. The name is "Study", not
"Analysis", because an analysis report is an activity regulated by the CVM (CVM Resolution 20/2021).

## Which fields to enrich

Two groups: fields that are already columns of `asset` (today's readers, such as reports, allocation, score and search, keep working unchanged) and new fields, kept in an attributes table so that no migration per field is needed.

### The fields

Columns that `asset` already had: sector (`sector`), segment (`segment`), CNPJ of the vehicle itself
(`cnpj`), ISIN (`isin`) and country of domicile (`country`). New fields, in the attributes table:
subsector, classification scheme, issuer CNPJ and name, security type (ON, PN, UNIT, CTF,
COMMON, ADR), listing segment, CVM code or CIK, LEI, country of exposure, the fund's CVM and
ANBIMA class, benchmark, manager and administrator, FII management type and mandate, taxation,
crypto category, IR (investor relations) site, short business description and status (active, canceled, in
judicial recovery).

### By class

**Brazilian stocks**
- Sector, subsector and segment, in the B3 scheme (with the CVM as a fallback).
- Issuer CNPJ and name, ISIN, security type, listing segment, CVM code, LEI, country.
- IR site, business description and status.

**FII**
- Segment (brick-and-mortar, paper, hybrid, FoF and the segment of activity), in the CVM scheme.
- Fund CNPJ, administrator (as issuer), manager, ISIN, security type, CVM code, country.
- Management type (active or passive) and mandate.
- IR site, business description and status.

**Brazilian ETF**
- Fund CNPJ, issuer, ISIN, CVM code, country of domicile and of exposure.
- CVM and ANBIMA class, benchmark, manager and administrator.
- IR site and description.

**BDR**
- Sector (that of the company abroad), subsector and segment, in the SIC or B3 scheme.
- Depositary CNPJ (as issuer), issuer name, ISIN, country of domicile and of exposure.
- IR site, description and status.

**US stocks, ETFs and REITs**
- Sector, subsector and segment, in the SIC scheme.
- Issuer, ISIN, security type (COMMON, ADR), CIK, LEI, country.
- Benchmark (for ETFs), IR site, description and status.

**Stocks and ETFs on other exchanges**
- Sector, subsector and segment, in Yahoo's classification (similar to GICS).
- Issuer, ISIN, security type, LEI, country of domicile and of exposure.
- Benchmark (for ETFs), IR site and description.

**Funds**
- CNPJ, CVM code, country of domicile and of exposure.
- CVM and ANBIMA class, benchmark, manager and administrator, taxation (long term, short term,
  equity or exempt).
- Site, description and status.

**Fixed income**
- Issuer sector, issuer CNPJ and name (today in `fixed_income_terms.issuer`), ISIN
  (debentures, CRI, CRA), LEI, country.
- Taxation (checks `ir_exempt`), incentivized debenture under Lei 12.431 (checks `kind`),
  index, rate and maturity (check only).
- Site and status.

**Tesouro**
- ISIN, country and the check of index, rate and maturity.

**Crypto**
- Category (L1, stablecoin, DeFi…), official site and description.

**Pension plans**
- CNPJ of the plan or of the FIE, insurer (as issuer), country.
- CVM and ANBIMA class, benchmark, manager and administrator, taxation.

**Outside enrichment:** quantity, average price, trades, income received (these are the family's data, not the asset's) and prices (they already have their own collection). Fixed income and Tesouro terms are only **checked** (a divergence warning), never replaced, because they change the portfolio's valuation.

### Sector taxonomy

Each source classifies in its own way: B3 has three levels (economic sector, subsector, segment), the CVM's FCA has a single "Sector of activity", the SEC uses SIC, Yahoo uses a classification similar to GICS. Proposal:

- The stored value keeps the **scheme** with it (`b3`, `cvm_fca`, `sic`, `yahoo`).
- Main scheme: **B3 for Brazilian assets, SIC for US ones** (official). FCA and Fundamentus serve as a check; Yahoo is the main one only for exchanges outside Brazil and the US, where there is no free official source.
- The comparison between sources happens after converting each value to the **app's canonical sector**, expanding the methodology's `sector_map` and `sector_aliases` (`methodology/v0.1.0/`), which already do this for the score. That way "Energia Elétrica" (FCA), "Utilidade Pública › Energia Elétrica" (B3) and "Electric Services" (SIC 4911) count as agreement.
- Allocation gains the **by sector** view once the data exists (phase 2).

---

## Sources, priority and cross-checking

### Source tiers

| Tier | What it is | Sources | Base weight |
|---|---|---|---|
| **1 · Official** | Regulator, exchange, public registry | CVM Open Data (FCA, FII report, registry of funds and classes, IPE); B3 (listed companies and funds, sector classification, ISIN); Receita Federal (CNPJ: legal name, CNAE, status, state); SEC EDGAR (submissions, SIC); Tesouro Transparente; Central Bank (IF.data, prudential segment of issuing banks); GLEIF (LEI, issuer country) | 90 |
| **2 · Issuer** | Document of the issuer itself or of the administrator | IR page, fund regulation and fact sheet, prospectus and indenture (linked to the CVM documents) | 80 |
| **3 · Reputable secondary** | Well-known aggregators | Fundamentus, Yahoo Finance, brapi, CoinGecko, OpenFIGI, Receita mirrors (BrasilAPI) | 60 |
| **4 · AI** | Value stated by a model | With a citation checked by the app: 45. Without a citation: 15 | 45 / 15 |

**ANBIMA:** ANBIMA's fund and debenture data is licensed (paid feed). The app **does not use** this data without a license; the ANBIMA class we need already comes from the CVM's class registry. If one day there is a user credential, it enters as tier 1.

**Receita (CNPJ):** the full open data is several GB per month. Proposal: on-demand lookup at a public mirror (BrasilAPI), marked "Receita Federal, via BrasilAPI", tier 3, with the CNPJ as the only data sent. The CVM's FCA already covers the legal name of listed companies at tier 1.

**New sources to confirm at implementation** (marked *(to confirm)* until validated against the API): the endpoint for B3's sector classification in the listed-companies API (`GetDetail` or equivalent), the ETFs' reference index at B3, the GLEIF API (`api.gleif.org`, CC0) and OpenFIGI (free with a limit, optional key).

### Pipeline of an asset

```
asset ──► known keys (ticker, ISIN, CNPJ, CIK, CVM code)
            │
            ├─ 1. data already in the vault (research_issuer/security, fund registry,
            │     Fundamentus cache, Yahoo search cache)                   no network
            ├─ 2. official sources with on/off switch (CVM, B3, SEC, GLEIF…)  public identifiers only
            ├─ 3. secondary (Fundamentus, Yahoo, CoinGecko, BrasilAPI)      public identifiers only
            └─ 4. AI (optional, `enrich` route)                            only what was missing or diverged
                     │
                     ▼
            observations (field, value, source, date, reference)
                     │  normalization (canonical sector, CNPJ digits only, ISIN with check digit)
                     ▼
            candidate per field + confidence + conflicts
                     │
                     ▼
            proposal per field (diff) ──► approve / reject / undo
```

- **Each reading becomes a recorded observation** (table `asset_observation`), even when it does not become a proposal. This gives the history ("the CVM changed the sector in 2027-03") and the "where it came from" screen.
- **Keys first:** without a ticker or ISIN there is nothing to cross-check; the first step is to find the ISIN and the CNPJ, which unlock the other sources. ISIN and CNPJ go through check-digit validation before becoming a key.
- **Pace and cache:** the same `research/net.rs` (pace per site, retry on 429/503) and the `research_fetch_log`. An asset successfully queried only goes back to the source after 30 days (sectors change little), unless explicitly requested.

### Confidence and conflicts

Confidence from 0 to 100 per field:

```
confidence = weight of the best tier
           + 5 per independent source that agrees (up to +10)
           − 25 if a source of equal or higher tier disagrees
           − 10 if the newest observation is older than 1 year (sector) or 90 days (status, manager)
           capped at 100; AI alone never exceeds 45
```

Bands: **high** ≥ 85, **medium** 60 to 84, **low** < 60.

Conflict rules:

1. The **highest tier** wins. Two sources of the same tier that disagree = **conflict**: the proposal shows both, neither is marked, and the user chooses.
2. "Agreeing" is after normalization (canonical sector, names without accents and without "S.A."), to avoid creating a false conflict.
3. **Independent** source: Fundamentus repeats B3's classification and does not count as a second opinion on it; the Receita mirror does not confirm the Receita.
4. An older tier 1 value loses to a newer tier 1 value from the same body (e.g. the new year's FCA).
5. **The user's value never enters the contest:** it is shown as "your value" and, if the official source disagrees, an informational divergence appears.

### When the AI cites a source (not yet implemented)

A value stated by the AI only rises to 45 if the app **checks the citation**: the model returns the URL and a literal excerpt; the app downloads the page (only domains from the allowed list: `.gov.br`, `cvm.gov.br`, `b3.com.br`, `sec.gov`, `gleif.org` and the issuer's already registered IR domain), looks for the excerpt and the value in the text and records the URL and the date as a reference. A page that does not open or an excerpt that does not appear = 15 and a "not checked" label.

---

## How enrichment writes

### Always a proposal, per field

- New action `assets.enrich_apply`: a `change_set` with **one item per field**, each with before, after, source, source date, confidence and link. Approval can be for the set or field by field (check/uncheck).
- **Filling an empty field**: its own "always approve" level (does not apply in autonomous mode), but approvable in bulk with one click. **Replacing an existing value**: sensitive, item by item, always.
- Undo through the history like any change; undo also deletes the provenance record of that field and restores the previous one.
- A pending proposal leaves the queue on its own if the user edits the same field first (current `withdrawn` behavior).
- The diff shows the **effect**: "the score now compares with Energia Elétrica peers" when the sector changes the score's segment; "enters Utilidade Pública in the allocation by sector".

### Never overwrite the user

Each field gets an **origin** (table `asset_field_meta`):

| Origin | When | Enrichment may |
|---|---|---|
| `user` | Typed in the interface | Only show a divergence; replace only if the user asks, item by item |
| `legacy` | Already filled in before this feature (no provenance) | Same as `user` (when in doubt, protect) |
| `import` | Came from an import (B3, notes, statements) | Propose a replacement as sensitive |
| `enrichment` | Came from an approved proposal | Propose an update when the source changes |
| `search` | Came from the registry search (Yahoo / lookup) | Propose a replacement by a higher-tier source |

The user can **lock** a field ("Don't suggest changes to this field"); once locked, not even the divergence appears.

---

## When it runs, cost and limits

### Triggers

| Trigger | What it does | AI |
|---|---|---|
| **Per asset** | "Fill in data" button in the asset's Details tab | Only if there is an engine on the `enrich` route and the deterministic path does not finish the job |
| **In bulk** | Portfolio › Asset health: "Fill in what is missing" (all or the selected ones, by field) | Same, with a cost estimate first |
| **At registration** | When creating an asset (form or import), the deterministic search suggests the fields already in the form or in a proposal right after the import | No |
| **By the agent** | "Fill in the sector of my assets" in the chat: the agent uses the tools under "MCP tools" | The chat's engine |
| **Scheduled (optional)** | Routine "Keep asset data up to date", weekly, off by default: checks official sources and proposes only what changed | Only on the `enrich` route, never falls back to the chat route (current rule for routines) |

### Without AI (structured sources)

Everything deterministic works: copying from the vault, querying official and secondary sources, normalizing, scoring confidence, showing conflicts and proposing. From what the code already downloads, this should cover sector, segment, ISIN, CNPJ, security type, listing segment, fund class, benchmark, manager and administrator for the vast majority of Brazilian stocks, FIIs, funds and US stocks (inferred from what the tables hold; measure in phase 1).

### With AI (not yet implemented)

- New tasks in the routing (`ai/routing.rs`): **`enrich`** and **`study`**, each with its list of engines, so that the user can put a cheap engine on enrichment and a better one on the study.
- **Enrichment without a tool loop:** a single call per batch, with compact JSON input (asset, keys, observations and what is missing) and JSON output with a fixed schema. Batches of up to 10 assets per call (5 in lean mode). It costs a fraction of the chat with tools.
- **Estimate before the batch:** "23 assets, ~2 calls, ~US$ 0.01 on engine X" with the prices from `ai/usage.rs`; above the confirmation limit, it asks for OK; the provider's daily and monthly limit applies as today.
- **Web search by the engine** (Anthropic, OpenAI and Gemini have native search with a domain list): off by default; when on, only on the domains of the allowed list.
- **Groq (lean mode):** no tools and no web search; batch of 5; input limited to ~2,000 tokens per batch; waits out the per-minute limit as in the chat. The study on Groq uses the asset's lean summary and an answer of up to `LEAN_MAX_TOKENS`.
- **Local engines (Ollama, LM Studio):** same single-call format; everything stays on the computer.
- **CLIs:** still without `WebFetch`/`WebSearch` (current decision); they receive the same package in the prompt.

---

## AI study of the asset

### What the user provides

In the asset's new tab, a **"Context for the AI"** field saved per asset (free text, up to 2,000 characters), with suggested questions:

- Why I hold this asset (thesis) and what would make me sell
- Horizon and goal (income, growth, protection, reserve)
- What worries me now
- What I already know or have read (links, news)

Plus, per request: the **question** of the moment ("What changed since the last result?") and optional **attachments** (release PDF, material fact, regulation), through the same path as the chat's attachments (`ai/attachments.rs`, with the current rule on whether to keep them).

The manager's memory notes linked to the asset (`ai/memory_graph.rs`) are included too, as already happens in the chat.

### What the agent receives

A **package assembled by the core**, without depending on the model calling tools (works on any engine, including Groq and local ones), with each number carrying its source and date:

| Block | Content | Sent by default |
|---|---|---|
| Registry | Class, exchange, currency, sector/segment with source and confidence | Yes |
| Score | Score, band, confidence, dimensions, red flags, what is missing (`score.get`) | Yes |
| Fundamentals | Last 8 quarters and 5 years of the main items (`fundamental`) | Yes |
| Income | 24 months checked against B3 (`research.dividends_check`) | Yes |
| Price | Summary for 1, 3 and 5 years (change, high, low, volatility), without the full series | Yes |
| Calculated price | Graham and Bazin with the assumptions (`fair_price`) | Yes |
| Documents | Last 10 CVM documents (title and date) | Yes |
| Peers | 5 peers of the same segment with the score's metrics | Yes |
| **My position** | Weight in the portfolio (%) and gain (%) | **No (option per study)** |
| Position values | Quantity, average price, value in reais | **Never by default; separate option** |
| User context | What the user provides, after redaction (`ai/redact.rs`) | Yes |
| Previous study | Summary and date of the last study, for "what changed" | Yes |

The package goes in a block delimited like `<contexto_da_tela>` (data, never instructions), with `<` and `>` escaped. Target size ~6,000 tokens; ~2,500 in lean mode (only registry, summarized score, 4 quarters, 12 months of income and context). Engines with tools can ask for more through today's read tools.

### What comes back

Output in JSON with a fixed schema, designed by the app (the text is not free markdown):

1. **Summary** in up to 3 sentences
2. **What the company or fund does** (when there is a source description)
3. **Strengths** and **risks**, each with the data that supports it
4. **Key numbers**: each number points to an item of the package (`ref`); the app checks the value against the package and marks **"not found in the data"** when it does not match
5. **What changed** since the previous study
6. **Your context**: do the thesis's assumptions still hold? What to confirm
7. **What to check by hand**: a list of checks with where to check (e.g. "Net debt: DFP 2025 at the CVM")
8. **Sources** used, with date
9. **Confidence** of the study itself (low, medium, high) and what was missing

Rules in the prompt (in English, like the others): do not recommend buying, selling or holding, do not give a target price or price forecast; speak in assumptions and risks; never make up a number outside the package; cite sources in plain words, never tool names (current rule).

### Where it lives

- Table `asset_study` in the vault (encrypted): date, engine and model, question, context used, package hash, output, number checks, notice version.
- **"AI study"** tab on the asset page, with the history (default: the last 20 per asset) and "Continue in chat", which opens the agent panel with the study as context.
- Not included in reports, PDF or alerts in the first version.

---

## Mandatory notice

Consistent with the terms (items 5.2 and 5.3) and with the current notices ("AI can be wrong. Check before approving." and "Study tool. Not investment advice.").

### Texts

**pt-BR**

- Título: **Gerado por IA: pode conter erros**
- Texto: Este estudo foi escrito por um modelo de inteligência artificial a partir dos dados do Treasury e das fontes citadas. Ele pode conter, e provavelmente contém, informações incorretas, desatualizadas ou inventadas, mesmo quando parecem confiantes. Confira cada informação nas fontes oficiais antes de usá-la. É uma ferramenta de estudo: não é recomendação, oferta, análise ou consultoria de investimento, e a decisão de comprar, vender ou manter é sua.
- Selo curto: Gerado por IA · pode conter erros · confira nas fontes
- Caixa da primeira vez: Entendi que o estudo pode estar errado e que preciso conferir as informações antes de decidir.
- Enriquecimento (rodapé da proposta): Dados de fontes externas e de IA podem estar errados ou desatualizados. Confira a fonte de cada campo antes de aprovar.

**en**

- Title: **AI-generated: may contain errors**
- Text: This study was written by an artificial intelligence model from Treasury's data and the sources cited. It may contain, and probably does contain, information that is incorrect, outdated or made up, even when it sounds confident. Check every piece of information against official sources before using it. It is a study tool: it is not an investment recommendation, offer, analysis or advice, and the decision to buy, sell or hold is yours.
- Short badge: AI-generated · may contain errors · check the sources
- First-time checkbox: I understand the study may be wrong and that I need to check the information before deciding.
- Enrichment (proposal footer): Data from external sources and AI may be wrong or outdated. Check each field's source before approving.

**es**

- Título: **Generado por IA: puede contener errores**
- Texto: Este estudio fue escrito por un modelo de inteligencia artificial a partir de los datos de Treasury y de las fuentes citadas. Puede contener, y probablemente contiene, información incorrecta, desactualizada o inventada, incluso cuando parece segura. Verifica cada dato en las fuentes oficiales antes de usarlo. Es una herramienta de estudio: no es una recomendación, oferta, análisis ni asesoría de inversión, y la decisión de comprar, vender o mantener es tuya.
- Distintivo corto: Generado por IA · puede contener errores · verifica las fuentes
- Casilla de la primera vez: Entiendo que el estudio puede estar equivocado y que debo verificar la información antes de decidir.
- Enriquecimiento (pie de la propuesta): Los datos de fuentes externas y de IA pueden estar equivocados o desactualizados. Verifica la fuente de cada campo antes de aprobar.

(pt-PT optional, like the other texts.)

### Where it appears

1. **Top of the AI study tab**, always visible, with no close button.
2. **On each saved study**, attached to the title, with the date and the engine ("AI-generated on 2026-10-20 with <engine> · may contain errors").
3. **Before the first study** of each vault: the first-time checkbox, not pre-checked (like the terms screen); the acceptance is kept in the vault with the notice version.
4. **In the chat**, when the answer comes from a study or the user asks for an analysis of an asset: the short badge below the answer.
5. **When copying or exporting** the study: the full text goes along, at the end.
6. **In the enrichment proposal**: the short footer.
7. **Terms:** add "asset studies" to item 5.3, version 1.0 → 1.1 (discreet notice, no new acceptance), since the connections are already covered by items 4.1 and 4.2.

---

## Privacy

| Destination | What goes | What never goes |
|---|---|---|
| Public sources (CVM, B3, SEC, GLEIF, BrasilAPI, Yahoo, Fundamentus, CoinGecko) | Only public identifiers of the asset: ticker, ISIN, CNPJ of the issuer or fund, CIK, coin id | Quantity, average price, accounts, names, CPF, any family data. The query does not say that the asset is in a portfolio |
| SEC EDGAR | The required User-Agent, with a **generic project contact** (already open in "SEC EDGAR contact"); never the personal email | — |
| AI engine in **enrichment** | Identifiers, name, class and the asset's public observations | Position, the asset's notes (`asset.notes`, which may have personal text), the manager's memory, any family data |
| AI engine in the **study** | The study package, the user's context after redaction and the attachments the user chose | Position values (only with an explicit option), accounts, transactions, income, members |
| The engine's web search (if on) | The query the model builds, restricted to the allowed domains | The prompt asks for public identifiers only; the app cannot intercept the native search query, which is why it starts off |
| Local engine | Everything stays on the computer | — |

- Before each study with a cloud engine, the **"What goes to [provider]"** chip shows the exact package, like the "Context of this page" of the panel.
- Every new source enters Settings › Data sources with an on/off switch; when off, it is not queried.
- Nothing is sent to the person responsible for the project; no telemetry.

---

## Interface and command design

### Interface sketches

**Portfolio › Asset health** (new tab or section inside "Asset health")

```
┌ Dados dos ativos ─────────────────────────────────────────────────────────┐
│ 31 ativos · 19 sem setor · 7 sem ISIN · 4 conflitos        [Completar o que falta ▾] │
│ Filtro: [Todos] [Sem setor] [Conflitos] [Com IA] [Por classe ▾]                       │
├──────────┬────────────┬──────────────────────┬────────────┬──────────────┤
│ Ativo    │ Campo      │ Sugestão             │ Fonte      │ Confiança    │
├──────────┼────────────┼──────────────────────┼────────────┼──────────────┤
│ ☑ TAEE11 │ Setor      │ Utilidade Pública    │ B3 · 10/26 │ ●●● alta 95  │
│          │ Segmento   │ Energia Elétrica     │ B3 · 10/26 │ ●●● alta 95  │
│ ☑ HGLG11 │ Segmento   │ Tijolo · Logística   │ CVM · 09/26│ ●●● alta 90  │
│ ☐ XPTO3  │ Setor      │ ⚠ Conflito: Varejo   │ B3         │ escolha      │
│          │            │   ou Comércio        │ FCA        │              │
│ ☐ VWRA   │ Setor      │ — (ETF)              │            │              │
└──────────┴────────────┴──────────────────────┴────────────┴──────────────┘
 Dados de fontes externas e de IA podem estar errados ou desatualizados…
                     [Rejeitar marcados]   [Propor marcados (12 campos)]
```

**Asset › Details**: each field with the origin chip

```
Setor      Utilidade Pública      [B3 · 14/10/2026 · alta]  ⓘ de onde veio
Segmento   Energia Elétrica       [você]  ⚠ B3 diz "Transmissão"  [Ver]
ISIN       BRTAEECDAM10           [CVM · 01/2026 · alta]
                                              [Completar dados]  [🔒 travar campo]
```

"Where it came from" opens the list of observations: each source, value, date and link.

**Proposal (agent panel or approval queue)**

```
Completar dados de 3 ativos · 7 campos
  TAEE11  Setor     (vazio) → Utilidade Pública    B3 · alta 95   ☑
          efeito: score passa a comparar com pares de Energia Elétrica
  HGLG11  Segmento  (vazio) → Tijolo · Logística   CVM · alta 90  ☑
  ...
  Dados de fontes externas e de IA podem estar errados…   [Rejeitar] [Aprovar 7]
```

**Asset › AI study**

```
┌ ⚠ Gerado por IA: pode conter erros ──────────────────────────────────────┐
│ Este estudo foi escrito por um modelo de inteligência artificial…          │
└────────────────────────────────────────────────────────────────────────────┘
Contexto para a IA  [Tenho para renda; venderia se a dívida passar de 3x…   ] ✎
Pergunta            [O que mudou desde o último resultado?                     ]
☐ Incluir meu peso e ganho na carteira   Motor: [Motor A ▾] ~US$ 0,03
[O que vai para o provedor]                                 [Gerar estudo]

Estudos anteriores
 20/10/2026 · Motor A · confiança média · 2 números não conferidos   [Abrir]
 12/07/2026 · Motor B · confiança baixa                             [Abrir]
```

Open study: Summary, Strengths, Risks, Key numbers (each with its source; unchecked ones highlighted), What changed, Your context, What to check by hand, Sources. Actions: Continue in chat, Copy (with the notice), Delete.

### Core commands

| Command | Type | What it does |
|---|---|---|
| `assets.data_gaps` | read | Missing assets and fields, conflicts, divergences with the user |
| `assets.field_sources` | read | Origin of each field and all the observations |
| `assets.enrich_lookup` | read (network) | Deterministic lookup of one asset; records observations, returns candidates and confidence; does not change the asset |
| `assets.enrich_run` | job | Batch (deterministic + optional AI), with cost estimate; returns a proposal |
| `assets.enrich_apply` | write (proposal) | Records the chosen fields with provenance |
| `assets.field_lock` | write (interface only) | Locks or unlocks a field |
| `assets.study_context_set` | write (interface only) | The asset's context for the AI |
| `assets.study_package` | read | The exact package that would go to the engine (preview) |
| `assets.study_start` / `study_list` / `study_get` / `study_delete` | job / read / write | Asset study |

### MCP tools

| Tool | Command | Level | In lean mode |
|---|---|---|---|
| `get_asset_data_gaps` | `assets.data_gaps` | read | Yes |
| `get_asset_field_sources` | `assets.field_sources` | read | No |
| `lookup_asset_reference` | `assets.enrich_lookup` | read (queries public sources) | No |
| `propose_asset_enrichment` | `assets.enrich_apply` | "always approve" proposal; replacement = sensitive | Yes |
| `list_asset_studies` / `get_asset_study` | `assets.study_list` / `study_get` | read | No |
| `start_asset_study` | `assets.study_request` | write (Sensitive: proposal; once approved, triggers the study) | No |
| `get_price_history` | `market.prices` | read | No |

`propose_asset_enrichment` refuses a value that is not in a recorded observation and carries no checkable citation: the agent cannot record a sector "from memory". External MCP clients follow the same rules. Descriptions in English, like the others.

### Migrations (V130 and V132)

- **V130 · provenance and observations of assets**
  - `asset_field_meta (asset_id, field, origin, source_id, source_ref, as_of, confidence, locked, change_set_id, updated_at)`, PK `(asset_id, field)`. Fills in `legacy` for every non-empty field that already exists.
  - `asset_attribute (asset_id, field, value, scheme)`, PK `(asset_id, field)`: the new fields from "By class" (subsector, issuer CNPJ, ANBIMA class, benchmark, manager, LEI…), with a field catalog in the code.
  - `asset_observation (id, asset_id, field, value, value_norm, scheme, source_id, tier, source_ref, published_at, observed_at, run_id)`, index `(asset_id, field, observed_at)`.
  - `enrichment_run (id, scope, started_at, finished_at, engine_id, tokens_in, tokens_out, cost_micros, status, summary)`.
- **V132 · asset studies**
  - `asset_study_context (asset_id, text, updated_at)`.
  - `asset_study (id, asset_id, created_at, engine_id, provider, model, question, context_snapshot, package_hash, include_position, output_json, checks_json, confidence, disclaimer_version, conversation_id)`.
  - Acceptance of the notice per vault in a setting (`study.disclaimer_accepted`: version and date).
- New sources in `SourceId` (B3 classification, GLEIF, BrasilAPI, OpenFIGI): code, no migration.

## How it turned out relative to the design

- Provenance ended up in V130, not V131, which was used by the Life timeline. The `enrichment_run` table was not created: it only serves AI in enrichment (phase 3).
- The study package also carries the score's indicators (`research.indicators`) outside lean mode. When it goes over the size, it cuts in this order: documents, indicators, calculated price, income, fundamentals and score.
- Commands beyond table 10.2: `assets.study_accept`, `assets.study_state` and `assets.data_alert`. `assets.enrich_run` does not exist yet (phase 3).
- The study uses the first ready engine of the `study` route; the tab shows which one it is, but does not let you change it.

## Limitations

- Enrichment with AI (the `enrich` route, a one-call batch, checked citation, web search by
  domain, cost estimate) does not exist; `assets.enrich_run` and the `enrichment_run` table were not
  created. Enrichment is only through the structured sources.
- The registration form does not suggest the fields, and imports do not propose enrichment right
  after; the missing fields show up in Asset health and in the alert.
- Missing are the ETFs' reference index at B3, the check of fixed income and Tesouro terms,
  OpenFIGI and a weekly data maintenance routine.
- `start_asset_study` exists in the MCP ([asset-page.md](asset-page.md)).
- The AI Study package does not carry the segment peers, the manager's memory notes or
  attachments; the tab does not let you choose the engine nor show the estimated cost.
