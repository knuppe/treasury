# Market data and index sources

Where the quotes, indices, exchange rates, fundamentals and other public data the app downloads come from. Each source has an isolated adapter, with a local cache, a per-site pace and the origin recorded in every stored value; each one is turned on and off in Settings › Data sources. Official and free sources take priority; those that require a token are optional.

## Indices and macro: Brazil

| Data | Source | Access |
|---|---|---|
| IPCA (monthly change) | Banco Central, SGS series **433** | `api.bcb.gov.br/dados/serie/bcdata.sgs.433/dados?formato=json`, no key |
| IPCA (index number) | IBGE SIDRA, table **1737** | SIDRA API, no key |
| IGP-M | BCB SGS series **189** | no key |
| INCC (construction cost, FGV) | BCB SGS series **192** | no key; adjusts real estate and lots when the user chooses, and contracts adjusted by the INCC |
| INPC | BCB SGS series **188** | no key; contract adjustment |
| Savings account (poupança) (return by anniversary date, rule of 2012-05-04) | BCB SGS series **195** | no key; "poupança + x%" contracts |
| Daily CDI | BCB SGS series **12** | no key |
| Daily Selic / target | BCB SGS series **11** / **432** | no key |
| TR | BCB SGS series **226** | no key |
| M2 (broad money supply) | BCB SGS series **27810** (end-of-period balance) | no key |
| PTAX exchange rate | BCB Olinda, `PTAX/versao/v1/odata` | no key |
| Focus (expectations) | BCB Olinda, `Expectativas` | used to estimate IPCA months not yet released |
| National holidays (business days) | ANBIMA, holiday calendar | public file, updated annually |
| IPCA by group (food, housing, household items, clothing, transport, health, personal expenses, education, communication) | IBGE SIDRA, table **7060**, variable 63 (monthly change), classification 315 | `apisidra.ibge.gov.br/values/t/7060/n1/all/v/63/p/{AAAAMM}-{AAAAMM}/c315/{grupo}`, no key; series `ipca_g1`…`ipca_g9` (since 01/2020), collected with the other indices (source "IBGE (SIDRA)" in Settings › Data sources). Basis of personal inflation |
| Prudential indicators of banks (Basel, core capital, leverage, delinquency, provisions) | BCB Olinda, **IF.data** | public OData API, no key; by quarter, technical tables `bank_institution`, `bank_prudential` and `bank_ifdata_fetch` (`research/ifdata.rs`); used in the indicators and in the bank score |
| Interest rates by bank and product (individuals) | BCB Olinda, **Interest rates by financial institution** (`taxaJuros/v2`) | `TaxasJurosDiariaPorInicioPeriodo` (weekly: overdraft, revolving and installment card credit, personal credit, private/public/INSS payroll loans, vehicle, other goods) and `TaxasJurosMensalPorMes` (fixed-rate mortgage, TR + rate, IPCA + rate), no key; table `credit_rate` (V119), downloaded when older than 7 days (Debts › Market interest rates and the `compare_loan_rates` tool) |

## Indices and macro: United States

| Data | Source | Access |
|---|---|---|
| CPI-U | FRED series **CPIAUCSL** | free FRED key |
| M2 | FRED series **M2SL** | free FRED key |
| Fed Funds | FRED series **FEDFUNDS** | free FRED key |
| Effective Fed Funds (daily) | NY Fed, Markets Data API, `nyfed:effr` (series `us_effr`, V123) | no key; reference interest rate for US households ([countries.md](countries.md)) |
| City tax revenue | Tesouro Nacional, Siconfi (`apidatalake.tesouro.gov.br`, RREO Annex 03), by municipality code (`tax_burden::city`, source "Tesouro Nacional (Siconfi)"); the list of municipalities comes from IBGE's localities API | no key; only at the user's request, in the Taxes screen ([taxes.md](taxes.md)) |
| NFC-e (tax receipt) | public lookup of the state's SEFAZ, through the receipt's QR link (`receipts`, source "SEFAZ") | no key; only `.gov.br` links, at the user's request ([receipts.md](receipts.md)) |
| IMAP server of a domain outside the catalog | the domain's own autoconfig (`https://autoconfig.<domínio>/mail/config-v1.1.xml`, `/.well-known/…`) and the Thunderbird database (`autoconfig.thunderbird.net/v1.1/<domínio>`), source "Email autoconfiguration" | no key; only the domain is sent, at the user's request in the ingestion email wizard ([email-ingest.md](email-ingest.md)) |
| Statements, card statements, notes and receipts by email | the family's IMAP mailbox (app password) or a local folder, in the ingestion email | only with the app open and the vault unlocked; TLS required ([email-ingest.md](email-ingest.md)) |
| CPI (keyless alternative) | OECD, `cpi_us_oecd` (see below) | no key; used by the "country inflation" deflator when there is no FRED key |

## Indices and macro: other countries (country catalog)

The **reference country** (Settings › Data sources; setting `indices.reference_country`, without it, it comes from the base currency: BRL → Brazil, USD → US, EUR → euro area, CAD → Canada, GBP → United Kingdom…) defines:

- the inflation of the **"Country inflation (…)"** deflator (`Deflator::Country`), with alternatives in order when the main one has no data for the period;
- which series the collection (`indices.sync`/`market.sync`) fetches **beyond the usual ones** (BCB, FRED and Yahoo continue to be collected for everyone);
- the reference interest rates and exchange rates shown for the country.

The catalog lives in the tables `country_profile` and `country_series` (migration V84) and is read by `indices.catalog` (and by the MCP tool `get_country_catalog`). In Brazil, IPCA, CDI/Selic and PTAX apply, with no extra series.

### Providers

All public, free and keyless. Each series stores in `index_series.source` the provider and the query (`ecb:EXR/D.USD.EUR.SP00.A`). The readers live in `crates/treasury-core/src/indices/providers/` with fixture tests; the `live_providers` test (ignored) makes one small query to each and `live_country_sync` collects the countries end to end.

- **ECB Data Portal** (`ecb:`)
  - **Data:** euro reference exchange rate against 29 currencies (`fx_eur_usd`, `fx_eur_brl`…), **€STR** (`estr`), **HICP** of the euro area (`hicp_ea`, area `U2`) and of the 21 members (`hicp_pt`…)
  - **Endpoint:** `https://data-api.ecb.europa.eu/service/data/{fluxo}/{chave}?startPeriod=AAAA-MM-DD&endPeriod=AAAA-MM-DD&format=csvdata&detail=dataonly`. Exchange rate `EXR/D.{MOEDA}.EUR.SP00.A`; €STR `EST/B.EU000A2X2A25.WT`; HICP `HICP/M.{PAÍS}.N.000000.4D0.INX` (2025=100; the old `ICP` flow stopped in 12/2025)
  - **Frequency:** daily (exchange rate, €STR), monthly (HICP; the euro area has the month's flash estimate)
  - **Terms:** free reuse with citation of the source. The portal blocks queries with wildcards (`..`): the app always asks for an exact series
- **Eurostat** (`eurostat:`)
  - **Data:** **HICP** of non-euro EU countries (CZ, DK, HU, PL, RO, SE)
  - **Endpoint:** `https://ec.europa.eu/eurostat/api/dissemination/statistics/1.0/data/prc_hicp_minr?geo={PAÍS}&unit=I15&coicop18=TOTAL&sinceTimePeriod=AAAA-MM` (JSON-stat 2.0; `prc_hicp_minr` is the ECOICOP v2 dataset that replaced `prc_hicp_midx`, frozen in 2025)
  - **Frequency:** monthly
  - **Terms:** CC BY 4.0 (open data policy of the European Commission)
- **OECD** (`oecd:`)
  - **Data:** monthly **CPI** (index 2015=100) of OECD members (`cpi_jp`, `cpi_kr`, `cpi_mx`, `cpi_ch`, `cpi_us_oecd`…) and **3-month interest rates** (`rate3m_jp`…)
  - **Endpoint:** CPI: `https://sdmx.oecd.org/public/rest/data/OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/{ISO3}.M.N.CPI.IX._T.N._Z?startPeriod=AAAA-MM&format=csvfile` and, as an alternative, the old flow `OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/…` (US, United Kingdom, Korea, Colombia, Israel and Australia are still only in it); the app queries both and keeps the one that reaches furthest. Interest rates: `OECD.SDD.STES,DSD_STES@DF_FINMARK,/{ISO3}.M.IR3TIB.PA._Z._Z._Z._Z.N`
  - **Frequency:** monthly
  - **Terms:** CC BY 4.0; the API limits queries per hour (the collection makes up to three per country: CPI in both flows and interest rates)
- **Bank of Canada / Valet** (`boc:`)
  - **Data:** total **CPI** (`cpi_ca`, V41690973, 2002=100), **CORRA** (`corra`, AVG.INTWO), USD/EUR/GBP/BRL exchange rates against CAD (`fx_usd_cad_boc`…)
  - **Endpoint:** `https://www.bankofcanada.ca/valet/observations/{SÉRIE}/json?start_date=AAAA-MM-DD&end_date=AAAA-MM-DD`
  - **Frequency:** monthly (CPI), daily
  - **Terms:** free use with citation (Bank of Canada terms)
- **ONS, United Kingdom** (`ons:`)
  - **Data:** **CPI** (`cpi_gb`, D7BT) and **CPIH** (`cpih_gb`, L522), 2015=100
  - **Endpoint:** `https://www.ons.gov.uk/economy/inflationandpriceindices/timeseries/{cdid}/mm23/data` (JSON of the series page; the old `api.ons.gov.uk` API was shut down in 11/2024)
  - **Frequency:** monthly
  - **Terms:** Open Government Licence v3.0
- **Bank of England** (`boe:`)
  - **Data:** **SONIA** (`sonia`, IUDSOIA), **Bank Rate** (`boe_bank_rate`, IUDBEDR), GBP/USD and GBP/EUR (`fx_gbp_usd`, `fx_gbp_eur`)
  - **Endpoint:** `https://www.bankofengland.co.uk/boeapps/database/_iadb-fromshowcolumns.asp?csv.x=yes&Datefrom=DD/Mon/AAAA&Dateto=now&SeriesCodes={SÉRIE}&CSVF=TN&UsingCodes=Y&VPD=Y&VFD=N`
  - **Frequency:** daily
  - **Terms:** free use with citation (Bank of England terms)
- **World Bank / WDI** (`worldbank:`)
  - **Data:** **annual consumer inflation** (`FP.CPI.TOTL.ZG`) for all countries in the catalog (`wb_cpi_pt`…), worldwide alternative
  - **Endpoint:** `https://api.worldbank.org/v2/country/{ISO2}/indicator/FP.CPI.TOTL.ZG?format=json&per_page=20000`
  - **Frequency:** annual
  - **Terms:** CC BY 4.0

### What is estimated

- **World Bank**: annual inflation becomes a monthly index. Each year's average sits on July 1 and the months between two years are interpolated geometrically; **all points are estimated**. The data comes out more than a year late, so the country deflator projects the last monthly change for up to **24 months** after the last point (also estimated). A year without data breaks the chaining (the last continuous stretch is kept): Argentina, without 2025 in the WDI, has no deflator for the recent period.
- **Monthly levels** (HICP, CPI, ONS, BoC, OECD): months not yet published repeat the last monthly change, as already happened with the US CPI and M2, and the result is marked as estimated.
- The monthly series from the new providers are **downloaded in full at each collection** (they are small), so that a change of base year does not mix bases (the HICP moved to 2025=100 in 2026). Daily series continue from the day after the last value.

### Exchange rates

- Exchange rate series (`fx_{base}_{cotada}`, 1 base = value in the quoted currency) live in `index_value` and are **copied to `fx_rate`** with the source `ecb_ref`, `boc` or `boe`, for conversions between other currencies (e.g., EUR → USD).
- **Pairs with BRL are not copied**: conversion to reais continues only through the PTAX (`bcb_ptax_venda`), as always. The ECB's EUR/BRL stays available as a series.

### Catalog coverage (64 countries)

| Group | Countries | Inflation (main → alternatives) | Interest rates | Exchange rate |
|---|---|---|---|---|
| Brazil | BR | IPCA | CDI, target Selic | PTAX |
| United States | US | CPI-U FRED → CPI OECD → World Bank | Effective Fed Funds (NY Fed) → Fed Funds (FRED) | ECB (EUR/USD, EUR/BRL) |
| Euro area | EA, AT, BE, BG, HR, CY, EE, FI, FR, DE, GR, IE, IT, LV, LT, LU, MT, NL, PT, SK, SI, ES | HICP (ECB) → World Bank | €STR | ECB |
| EU outside the euro | CZ, DK, HU, PL, RO, SE | HICP (Eurostat) → World Bank | OECD 3-month rates (except RO) | ECB |
| Other OECD members | JP, KR, MX, CL, CO, CR, IL, TR, AU, CH, NO, IS | CPI OECD → World Bank | OECD 3-month rates (except TR) | ECB |
| Canada | CA | CPI (Bank of Canada) → World Bank | CORRA | Bank of Canada |
| United Kingdom | GB | CPI (ONS) → CPIH (ONS) → World Bank | SONIA, Bank Rate | Bank of England + ECB EUR/BRL |
| Others | NZ, AR, UY, PY, PE, BO, EC, CN, IN, ID, ZA, AO, MZ, CV, SG, PH, TH, MY, AE, SA | World Bank (annual, estimated) | OECD 3-month rates (NZ only) | ECB (when the currency is quoted) |

### Known limitations

- Conversion between currencies uses the direct pair or its inverse, with no intermediate currency: a currency with no pair stored against the base currency needs a manual exchange rate.
- There is no free **monthly** inflation source for most countries outside the OECD and the EU: the World Bank is annual and delayed. The IMF API (DataMapper, with projections for the current year) refuses clients that are not browsers.
- New Zealand and Australia publish quarterly CPI; NZ uses the World Bank. Austria, Poland and Slovakia are not in the OECD monthly CPI, but use the HICP.

## Quotes: Brazil

| Class | Main source | Alternatives |
|---|---|---|
| Stocks, FIIs, ETFs, BDRs (official history) | **B3 COTAHIST** (annual/daily historical files, free) | |
| Stocks, FIIs, ETFs (recent quote) | Yahoo Finance (suffix `.SA`, unofficial), while it is on | **B3 COTAHIST** (official close of the previous session, copied from what the asset analysis already downloaded; applies alone when Yahoo is off or by choice on the asset, `market::sync::b3_source`); brapi.dev, with a token ([brapi.md](brazil/brapi.md)) |
| Tesouro Direto | **Tesouro Transparente** (CSV of historical prices and rates) | |
| Investment funds | **CVM Open Data**, daily report (quota value by CNPJ) | |
| CDB, LCI, LCA, debentures | Mark to the curve from CDI/IPCA/fixed rate | manual entry |
| Corporate events and dividends | **Fundos.NET** (income notices of the funds), B3 (listed companies and funds), Yahoo, Fundamentus | manual entry or note reading by the agent |
| Investor's trades and position | **B3 Investor Area** (trading, movement and position reports in Excel) | file import (manual or from the Downloads folder) |
| Fundamentals of listed companies | **CVM Open Data** (DFP, ITR, FCA, IPE) | |
| Fundamentals of FIIs | **CVM Open Data**, monthly FII report | |

## Quotes: international

| Class | Main source | Alternatives |
|---|---|---|
| US stocks, ETFs, REITs | **Tiingo**, with the person's own free key (`market/tiingo.rs`) | EODHD (with key), Yahoo Finance (unofficial), manual quote |
| Stocks and ETFs from other exchanges | **EODHD**, with the person's own key (`market/eodhd.rs`; `TICKER.BOLSA`, e.g., `VOD.LSE`) | Yahoo Finance (unofficial; ticker + exchange suffix, e.g., `VOD.L`, `7203.T`), manual quote |
| Fundamentals of US companies | **SEC EDGAR**, XBRL API `companyfacts` (free, requires an identified User-Agent) | |
| Crypto | **DefiLlama** (open API, no key): daily close in dollars by the asset's CoinGecko id, converted to the asset's currency at the day's exchange rate (`market/defillama.rs`) | Yahoo (`BTC-USD`), only if the person chooses it on the asset or if the asset has no known id; manual quote |
| Exchange rate of other currencies | BCB PTAX (quoted currencies, to reais) | reference rates from the ECB, Bank of Canada and Bank of England, depending on the reference country (see above) |

### Sources with the person's own key (Tiingo and EODHD)

Quote services with a documented API offer personal-use plans and forbid redistributing the data. The app respects this with the same model as FRED and brapi: **each person creates their own key**, pastes it into Settings › Data sources, and the data stays only in their vault. The project publishes the adapter, never a key or the data.

- **Source choice** (`market::sync::keyed_source`): with no choice on the asset, US securities go to Tiingo when there is a key and the source is on; other exchanges (and US ones without Tiingo) go to EODHD when there is a key and the exchange has a known code (`eodhd::exchange_code`). Without a key, the previous source (Yahoo) applies. The person can also pin `tiingo`, `eodhd` or `yahoo` on the asset, and the provider symbol accepts EODHD's full symbol for exchanges outside the list. B3 securities only go to EODHD by choice on the asset.
- **What comes in**: unadjusted daily close, dividends by ex date and by the amount paid on the date (Tiingo `divCash`, EODHD `unadjustedValue`) and splits (factor = new shares per old). Stored with the source `tiingo` or `eodhd` in `price`, `dividend_event` and `corporate_action`, with the same rules as Yahoo's splits (below).
- **Tiingo**: `api.tiingo.com/tiingo/daily/<ticker>/prices`, key in the `Authorization: Token …` header. Free plan: 50 requests per hour, 1,000 per day, 500 symbols per month, "Internal Use Only" (personal use).
- **EODHD**: `eodhd.com/api/{eod,div,splits}/<TICKER.BOLSA>` (three requests per security per collection). The API only accepts the key in the URL (`api_token`): the URL never goes to a log, report or error message. London, Johannesburg and Tel Aviv quote in a subunit (pence, cents, agorot) and are converted. The terms allow a non-professional user to store and analyze the data for private use. The free plan is for testing only.
- **Status**: Tiingo and EODHD were written from the documentation, with tests on sample responses. **Neither has been checked with a real key yet**; the EODHD exchange codes come from the documentation and from what is known about the service.
- When the key is removed, the data already downloaded stays in the vault. The terms of both ask for the data to be deleted when the account ends: that is still up to the person.

### Splits and bonus shares reported by Yahoo

The quote collection automatically records (origin "Yahoo collection") the splits and reverse splits after the asset's first trade. Yahoo calls a bonus share a "split" (10% appears as 1.1:1); it becomes a **bonus share** when the official B3 events say so, or, for B3 securities without that data, when the factor is not an integer and is below 2. Nothing is created when:

- a split, reverse split or bonus share already exists within 10 days of the date;
- the trades already carry the event: a zero-cost entry or a purchase with the quantity of the new shares (bonus share imported with the cost assigned) within 10 days of the date;
- the event has been seen before. **Deleting or undoing a collection event prevents it from coming back** in the next collections (it stays in the list `market.yahoo_splits_seen` and in the change history).

The asset statement (`portfolio.statement`, tool `get_asset_statement`) flags with `warning` the two ways of counting the event twice: a zero-cost entry near an event, and a purchase with the quantity of the new shares near an event.

## Intraday quotes and ticker search (Yahoo Finance)

| Data | Endpoint | Use |
|---|---|---|
| Intraday chart + quote summary | `query1.finance.yahoo.com/v8/finance/chart/{símbolo}?interval=5m&range=1d\|5d` (accepts `1m`, `5m`, `15m`) | last price, previous close, high/low, volume, time of the last trade, time zone and session period (`meta`), and the 5-minute points |
| Ticker search | `query2.finance.yahoo.com/v1/finance/search?q=…&quotesCount=12&newsCount=0` | autocomplete for asset registration, trade and watchlist |

- **Delay**: Yahoo quotes arrive ~15 min late (crypto, nearly real time). The interface shows the "~15 min delay" badge with the time of the last trade and the session status.
- **No key and no cookie**: the `v7/finance/quote` endpoint now requires a *crumb*; the quote summary comes from the chart's `meta`, which does not require one.
- **When it collects**: app routine every 5 min, only with the vault open, only for assets with trades or on the watchlist and only with the exchange session open (exchange catalog, in the exchange's time zone, with daylight saving time) or closed for less than 20 min. Errors back off 5 → 10 → 20 → 40 → 60 min, per symbol and overall (network down). The asset page requests a collection on opening if the quote is more than 5 min old (also outside the session).
- **Where it lives**: `market_quote` (last quote per asset) and `intraday_price` (5-minute points), technical tables outside the change history; points older than 7 days are deleted. The daily closes (`price`) remain the source of the history.
- **In the portfolio**: today's valuation uses the intraday quote when it is newer than the last close (marked with the source `yahoo_intraday`, the time and, with the session open, as estimated). Past dates use only the closes.
- **Subunits**: LSE quotes in pence (`GBp`), JSE in cents (`ZAc`) and Tel Aviv in agorot (`ILA`); Treasury converts to GBP, ZAR and ILS (also in daily closes and dividends).
- **Search**: combines the registered assets, the asset analysis universe (`research_security`) and Yahoo (network outside the database lock, 10-min cache per text, 300 ms wait while typing). Fractional B3 codes (`PETR4F`) and the like are left out; the class comes from the exchange (symbol suffix or Yahoo exchange code), the type (`EQUITY`/`ETF`), the analysis universe and the B3 importer's rules (FII, ETF, BDR).
- **Terms**: Yahoo Finance has no public API; the endpoints are those the site itself uses. Yahoo's terms ([legal.yahoo.com](https://legal.yahoo.com)) do not authorize automated collection, storing the data or commercial use, and the `robots.txt` of `query1`/`query2` blocks everything. The app identifies itself as what it is (`Treasury/versão`, never as a browser), queries little (≤ 1 per asset every 5 min), does not redistribute, and the sources screen tells the person this, who can turn the source off. The adapter is isolated in `market/yahoo.rs`; replacing it with sources using the person's own key is planned.

## Exchange rates for exchanges outside Brazil and the US

The Banco Central PTAX covers only AUD, CAD, CHF, DKK, EUR, GBP, JPY, NOK, SEK and USD (`Moedas` endpoint of the Olinda API). Currencies with an ECB reference rate against the euro (HKD, CNY, INR, KRW, SGD, ZAR, MXN, NZD, PLN, TRY, ILS, IDR, THB, MYR, PHP, CZK, HUF, RON, ISK) are collected from the ECB when there is an account or asset in them and reach the real by crossing through the euro (currency → EUR through the ECB, EUR → BRL through the PTAX; `market::exchange::ECB_CURRENCIES`, `fx::rate`). The others (TWD, ARS, CLP, SAR…) need a manual exchange rate (`fx.set`) or the exchange rate entered in the trade; the exchange catalog marks these exchanges with "manual FX".

## Asset analysis (fundamentals and universe)

| File | Years downloaded | Typical size | Use |
|---|---|---|---|
| CVM FCA `fca_cia_aberta_AAAA.zip` | current and previous | ~0.4 MB each | sector and trading codes (ticker ↔ CNPJ) |
| CVM DFP `dfp_cia_aberta_AAAA.zip` | last 10 fiscal years (+ the comparative of the oldest) | 10–14 MB each | income statement, balance sheet assets and liabilities, cash flow, value added and capital composition |
| CVM ITR `itr_cia_aberta_AAAA.zip` | 3 years | 20–32 MB each | quarterly accumulations (LTM and last 8 quarters) |
| CVM monthly FII report `inf_mensal_fii_AAAA.zip` | 6 years | 1–2 MB each | net assets, shares, NAV/share, shareholders, month's DY, segment |
| CVM IPE `ipe_cia_aberta_AAAA.zip` | current and previous | ~2 MB each | material facts and announcements (only registered issuers) |
| B3 daily COTAHIST `COTAHIST_DDDMMAAAA.ZIP` | missing sessions, up to ~1 month | ~0.6 MB each | close and financial volume of the whole spot market |
| SEC `company_tickers.json` and `companyfacts/CIK…json` | — | ~1 MB + ~0.3 MB (gzip) per company | fundamentals of registered US stocks |

- Full collection the first time: ~230 MB (only in memory, one file at a time; nothing stays on disk besides the database), about 1 to 2 minutes. After that, files that did not change are skipped by the ETag.
- The SEC requires a User-Agent with a contact email: the default (`Treasury/0.1 contato@exemplo`) warns to set your own email in Settings › Data sources.
- CVM files with share counts in thousands and no scale column are corrected by the earnings per share of the same document and checked against the quote.

## B3: dividends and official events (listed companies and funds)

Official source of dividends for the whole universe of stocks and FIIs: the public API that B3's own site uses on the "Listed companies" and "Listed funds" pages (`https://sistemaswebb3-listados.b3.com.br`, no key or registration; each request's argument is a base64 JSON in the path). The `robots.txt` of `www.b3.com.br` restricts nothing (`User-agent: *` with no `Disallow`); the subdomain does not publish a `robots.txt`.

| Request | Argument | Use |
|---|---|---|
| `listedCompaniesProxy/CompanyCall/GetListedSupplementCompany` | issuer code (`PETR`) | cash dividends of the last ~12 months by ISIN (record date, amount, approval, payment), splits, reverse splits and bonus shares, trading name |
| `listedCompaniesProxy/CompanyCall/GetListedCashDividends` | trading name, pages of 120 (the maximum accepted) | full history by type (ON, PN, PNA, UNT…), with no payment date; only issuers of registered assets, once |
| `fundsProxy/fundsCall/GetListedSupplementFunds` | fund code (`HGLG`, `typeFund` 7) | distributions and amortizations of the last ~12 months and events on shares |
| `listedCompaniesProxy/CompanyCall/GetDetail` | CVM code | the company's tickers and ISINs, only for companies whose FCA has no valid ticker or only the unit (once a week) |

- Storage: `research_dividend` (security, type `dividend`/`jcp`/`fii_income`/`amortization`/`other`, record date, exact decimal amount, amount per how many shares, approval, payment, B3 text, installment) and `research_stock_event` (security, `split`/`reverse_split`/`bonus`, record date, factor as B3 reports it: percentage in splits and bonus shares, factor in reverse splits). Technical tables, outside the change history and the backups. The ticker comes from the ISIN (`ACNOR` ON, `ACNPR` PN, `ACNPA` PNA, `CDAM` unit, `CTF` share) or from the history type, checked against the FCA codes (`research_security`), the COTAHIST quotes and the registered assets; receipts, rights and debentures are left out.
- Considerate use: one request at a time, never in parallel, with 1.5 s between requests to the same site and a retry on 429/503 respecting the `Retry-After` (`research/net.rs`); five failures in a row stop the B3 collection until the next one. Rolling collection through `research_fetch_log` (`source = b3_listados`, key `cia:PETR`, `fii:HGLG`, `hist:PETR`): the issuers and funds of registered assets every ~20 h; then a batch of others per day (60, or the universe ÷ 7 if larger, so the whole universe is covered in ~one week; technical setting `research.b3_listados_batch`), the never-queried and the oldest first, only those with a security traded in the COTAHIST; an error is retried after a day, an unknown security after a week. Several collections on the same day do not exceed the batch.
- The company supplement also brings the capital stock share count (total, common, preferred, with treasury shares): it is stored as `shares_b3*` in `fundamental` (only the latest query) and checks the balance sheet's share count ([indicators.md](indicators.md), E4). Opening a security without official dividends asks B3 once (`research.b3_refresh`), within the same ~20 h interval.
- Turns on and off together with the B3 source in Settings › Data sources; failures count toward the source's health (three days in a row turn it off).
- Use of the data: 12-month DY, last dividend and Bazin for the whole universe (screener, rankings and peers of the score). The values are adjusted by later splits, reverse splits and bonus shares (per share of today); amortizations do not enter the DY. Registered assets keep the schedule (`dividend_event`: Yahoo, Fundamentus) while it agrees with B3; with a material divergence, B3 (official) becomes the reference. Without a schedule, B3 comes before recorded dividends.
- Check (`research.dividends_check {asset_id | ticker}`, MCP tool `check_dividends`, read-only): over the last 24 months, it pairs by record date (up to 3 business days: Yahoo uses the ex date) and nature (dividend or amortization) the B3 dividends, summed by date, with those of each schedule source; it flags amounts off by more than 2% (`value_diff`), dividends only on B3 (`only_b3`, only in the period the source covers) or only on the schedule (`only_other`) and compares the 12-month sum. A value that only matches after the split counts as equal, with a note (Yahoo adjusts past values; B3 and Fundamentus do not). `material = true` = the indicators use B3. The health score shows the same in "Cross-check with other sources", without lowering confidence.

## Fundos.NET: official income notices of the listed funds

The structured notice "Rendimentos e Amortizações" that every listed FII and Fiagro files on
Fundos.NET, the disclosure system of the CVM and B3 (`https://fnet.bmfbovespa.com.br/fnet/publico`,
public pages, no key; the host publishes no `robots.txt`). It is the reference for the income of
the funds in the income ledger ([income-ledger.md](brazil/income-ledger.md)).

| Request | Argument | Use |
|---|---|---|
| `pesquisarGerenciadorDocumentosDados` | fund CNPJ, category 14, type 41, active only, pages of 100 | the fund's notices: id, version, record date, filing time |
| `downloadDocumento?id=…` | document id | XML of ~1 KB: ISIN, trading code, income or amortization (total, partial), record date, amount per unit, payment date, reference month, tax exemption |

- Two XML layouts: until 2022 one document per security, with the codes in the header
  (`CodNegociacaoCota`, `CodISINCota`, `ValorProventoCota`); since then one `Provento` block per
  security (unit and subscription receipts), with `CodNegociacao` and `ValorProvento`. The name and
  phone of the person in charge at the administrator are in the XML and are not read.
- Corrected filings: the list only brings active documents; a notice whose document left the list
  is removed. When two active documents report the same distribution, the latest filing wins.
- Storage: `research_fund_notice` (document, version, CNPJ, code, ISIN, kind, amortization type,
  record date, exact decimal amount, payment, reference month, exemption, filing time), a technical
  table outside the change history and the backups.
- Considerate use: only the funds the family holds or held (class FII, with a trade or received
  through a ticker change), by the CNPJ of the asset registry or of the CVM data; the list again
  every ~20 h; each document downloaded once (a filed notice does not change); notices with a
  record date more than 40 days before the first purchase are not downloaded; one request at a time
  with 1.5 s between requests; five failures in a row stop the collection until the next one.
- Terms: the B3 site terms (strictly personal use); nothing leaves the vault. Turns on and off in
  Settings › Data sources (`fundos_net`); failures count toward the source's health.

### Fund filings (Documents tab)

The same list, by category, gives the Documents tab of a real estate fund (`fundos_net.rs`,
`sync_docs`): management reports (category 7, "Relatórios", only the type "Relatório Gerencial"),
material facts (category 1) and market announcements (category 3), active filings delivered from
January 1 of the previous year (as the CVM IPE collection), for the funds held now, before or on the
watchlist. Only the list is stored, in `corporate_document` (id `fnet-<id>`, source `fundos_net`,
the download link); the PDF is downloaded only when the agent reads it (`read_corporate_document`,
only links of `fnet.bmfbovespa.com.br/fnet/publico/downloadDocumento`). The monthly and quarterly
reports are not listed there: their data already comes structured from the CVM open data. The
list again every ~20 h, three requests per fund (more pages only when a category has more than
100 filings in the period); a filing that left the list is removed.

### FII result from the CVM quarterly report

`research.fii_result {asset_id | ticker}` (MCP tool `get_fii_result`, read-only) shows the
income statement of the quarterly report (`inf_trimestral_fii_resultado_contabil_financeiro`,
cash-basis `_Financeiro` columns), the same lines behind the income coverage of the score: rent
(`Receita_Aluguel_Investimento`), interest on securities (`Receita_Juros_TVM`), cash investments
(`Resultado_Liquido_Recurso_Liquidez`), sales of assets (sale of properties less their cost,
inventory, sale of securities and of cash investments), the remaining lines, expenses
(`Total_Receitas_Despesas`, of which administration and performance fees), the net result
(`Resultado_Trimestral_Liquido_Financeiro`), the result without sales and the distributions
declared per quarter (the semester figure split by quarter). Every line in the file is quarterly
and adds up to the net result (checked on all 3,937 rows of the 2026 file). Per quarter (the last
eight), for the last 12 months (four quarters in a row) and the last quarter, in cents, per unit
by the units of the latest monthly report, with each quarter's document and filing date and the
obligations for property acquisitions of the monthly report.

### Dividends to confirm (cross-checking sources)

`portfolio.pending_income` (Portfolio › Record dividends › To confirm; MCP tool `list_pending_income`) suggests one dividend per event, crossing the asset's schedule (Yahoo, Fundamentus) with the official events, the funds' Fundos.NET notices first and B3's (`portfolio/pending.rs`). It is the missing part of the income ledger ([income-ledger.md](brazil/income-ledger.md)): with the automatic recording on, only the distributions without an official source wait there.

- same dividend = same asset and record date up to 2 business days apart in the B3 calendar (Yahoo uses the ex date, which can be five calendar days later around Carnival); Yahoo does not separate JCP (interest on equity) from dividends and sometimes sums the two of the same day, so its amount is checked against each official event or against the sum;
- without B3, schedule sources with different types (Yahoo calls income what Fundamentus calls amortization) are the same dividend when the dates are within 3 business days and the amounts within 2%: a single suggestion, with Fundamentus's type and payment;
- reference: B3 when it has the dividend; without it, the schedule (with Fundamentus and Yahoo diverging in amount or type, Fundamentus);
- `asset_warning`: an asset with no new quote for more than 60 days and with the last one having no trades, a sign of a delisted or merged fund (check before recording);
- `check.status`: `confirmed` (another source confirms amount, within 2%, and payment date), `divergent` (uses the reference; `note` says what the other source said) or `single` (one source only); `check.sources` shows what each source says.

With an AI engine on, "Ask the agent to check" takes the doubtful cases to the agent, which checks with `check_dividends`, `get_fundamentus` and the investor relations pages and proposes the fix with `record_income_event`; nothing is saved without the user's approval.

## Fundamentus (indicators, technical data and dividends)

B3 stocks and FIIs have, in the asset's Fundamentals tab, the indicators from [Fundamentus](https://fundamentus.com.br) (P/E, P/B, DY, ROE, margins, debt; FFO, vacancy and cap rate for FIIs), the technical data (quote, 52-week low and high, average volume, market cap, sector and subsector), the price change and the dividend history (`detalhes.php`, `proventos.php` and `fii_proventos.php`).

- Local cache (`fundamentus_cache`, outside the history): the page stays fresh for 20 hours; a security the site does not have is remembered for 7 days. Only the "Refresh" button (or `refresh` in the `get_fundamentus` tool) forces a trip to the site.
- One request at a time, with 1.5 s between requests and a 20 s timeout; the network does not hold the database.
- Uses `fundamentus.com.br` (the certificate of `www.` does not match).
- Dividends: the announced ones become asset events (`dividend_event`, `source = fundamentus`; FII income, dividend, JCP, amortization and refund as amortization, since V114), without repeating what already came from another source on the same record date with an amount equal within 1%; those the family was entitled to appear in "Dividends to confirm". The collection runs together with the quotes one (with the cache, one trip to the site per security per day) and also through the "Fetch from Fundamentus" button in the Dividends tab.
- For fundamentals with a formula, period and official source, the asset analysis keeps using the CVM.

## Turning external sources on and off

Settings › Data sources › External sources lists each source queried over the network (`crates/treasury-core/src/sources.rs`): Yahoo, Tesouro, CVM, B3, SEC, Banco Central, FRED, ECB, Eurostat, OECD, Bank of Canada, ONS, Bank of England, World Bank, FIPE, Fundamentus, CoinGecko, DefiLlama, Coin Metrics, mempool.space, brapi.dev, IBGE, NY Fed, SEFAZ, Tesouro Nacional (Siconfi), email autoconfiguration, GLEIF, BrasilAPI, Open Food Facts and the news feeds. The sources are on by default in Brazil, except Open Food Facts, which the person turns on (`sources::OPT_IN`); outside Brazil, the family's country turns off those that make no sense there ([countries.md](countries.md)) (`sources.list`, `sources.set`, setting `sources.disabled`). When off, the source leaves the collection of quotes and indices, the asset analysis, the ticker search, the intraday quote, FIPE and Fundamentus (which still shows what is in the cache); what was already downloaded stays in the vault. Only the app's interface turns sources on and off.

## Terms, license and attribution of each source

Each source has a terms text (`SourceId::terms`, catalog `nucleo-fontes-<id>-termos`): the license, attribution or notice it asks for. It appears in Settings › Data sources, next to each source, and gathered in Settings › About and support › Data sources and attributions. A test fails if a source lacks the text in the three languages: **a new source only goes in after its terms have been read**.

- Notices required word for word stay in English in all languages: FRED ("This product uses the FRED® API…"), NY Fed (notice for the reference rates), Bank of England (SONIA) and ONS (Open Government Licence v3.0).
- The crypto asset score shows the sources with a link next to the data (CoinGecko, Coin Metrics under CC BY-NC 4.0, DefiLlama, mempool.space).
- The app introduces itself to every site as `Treasury/<versão> (app local de financas pessoais; +<endereço do projeto>)`, never as a browser.
- **Google News**: its terms and `robots.txt` do not accept automated access, so the app does not query it. The asset's News tab only brings a link to the ticker search, which opens in the person's browser.
- **B3**: the site's terms allow exclusively personal use. No B3 data goes in the installer or in any file of the repository, and what each person downloads stays only in their vault.

## List of banks (Banco Central)

The choice of bank in the account registration and the recognition of an OFX's bank (`BANKID`) use the official list of STR participants of the Banco Central ([ParticipantesSTR.csv](https://www.bcb.gov.br/content/estabilidadefinanceira/str1/ParticipantesSTR.csv): ISPB, short name, COMPE code, full name). The app ships a copy (`household/bcb_participantes_str.csv`) and updates it from the site once a week, together with the market sync, with the vault open; without a network, the last list stays (`household.banks`, table `bank_registry`).

## Usage rules

- Daily closing quotes are the source of the history; the intraday quote (Yahoo, ~15 min delay) only applies to "today" and is never real time.
- Background update on opening the app and once a day, with exponential backoff on error.
- Each stored value records `source` and `fetched_at`; the UI shows the origin on hover.
- Unofficial sources (Yahoo) sit behind an isolated adapter, so they can be replaced without impact if they stop working.
- Without internet, the app uses the last known value and shows its date.
- Importing files (COTAHIST, broker CSVs, OFX) is always an alternative to the API.

## Physical assets

| Data | Source | Access |
|---|---|---|
| Average price of vehicles (FIPE table) | public API `parallelum.com.br/fipe/api/v1` (brands → models → years → price) | no key; queried only when the user asks ("Check FIPE"), saves a valuation with source `fipe` |
| Adjustment of real estate and lots | IPCA, IGP-M or INCC from the series above | estimate from the cost or the last valuation, marked as estimated |

## Multiple sources per data point (health score and asset analysis)

Each data point has an **official** source (CVM, B3, Banco Central, Tesouro, SEC) and, when possible, **secondary** sources to cross-check:

| Situation | What the app does |
|---|---|
| Official and secondary agree (within 2%) | Uses the official one |
| They diverge by more than 2% | Keeps the official one and shows the difference in the score's cross-check; above 25%, the score's confidence drops |
| Only the secondary has the data | Uses the value marked "secondary source" (with the source and date); with two secondaries agreeing (within 10%), the average; confidence drops |

Secondaries today: **Fundamentus** (local cache of the registered assets' pages) and **brapi.dev** (optional, with a token in Settings › Data sources; the token goes in the `Authorization` header, never in the URL). brapi reads the plan and usage of the API itself and the app stops at 80% of the quota (adjustable); on the free plan it is a backup for the quote of B3 securities when Yahoo fails (and a source for assets that choose it), one per security per day, and warns of renamed tickers (without a token); the cross-check of indicators and dividends is left for the paid plans. See [brapi.md](brazil/brapi.md).

Analysis sites that block automated access or whose terms forbid downloading and storing the content are not used.

### Considerate use of the services

- **Per-site pace** (`research::net::min_gap`/`pace`): minimum interval between requests to the same host, for the whole process — B3 1.5 s, Fundamentus 1.5 s, brapi.dev 1.2 s, Banco Central (Olinda) 0.5 s, CVM 0.3 s, SEC 0.15 s (published limit: 10/s), CoinGecko 6.5 s.
- **429/503**: up to 3 attempts, waiting for the site's `Retry-After` (or 2, 4, 8 s), at most 60 s; then the collection records the error and the source's health (3 days in a row of failure turn the source off with an alert).
- **Cache**: files with ETag/Last-Modified are not downloaded again; per-security queries stay in `research_fetch_log` with the date, and are only made again after the deadline (brapi.dev 20 h).
- **COTAHIST**: besides the missing daily sessions, the **monthly** files of the last 13 months (~10 MB each, at most 4 per collection, only once per closed month) form the quote history of the whole universe (1-year volatility, 12-1 momentum and liquidity in the score), kept for 400 days.

## Sources evaluated

**Adopted:**

- **IBGE SIDRA, IPCA by group (table 7060)**: official, free, keyless, stable. It enables personal inflation (the weight of each group in the family's spending in place of IBGE's basket), useful above all for lower-income families, in which food and housing weigh more than in the IPCA.
- **Banco Central, Interest rates by financial institution**: official, free, keyless, weekly. It compares the user's contract with what each bank charges in the same product and gives numbers to negotiate or to port the loan. They are each bank's averages: the rate offered to a person depends on their profile.

**Discarded, for now:**

| Source | Why |
|---|---|
| BrasilAPI for the CNPJ of transactions, postal codes (CEP) and banks | Community aggregator, with no availability guarantee; querying the CNPJ of a transaction would send statement data to a third party. The official list of banks already comes from the BCB. BrasilAPI is only used for the legal name and registration status of the CNPJ of an asset's **issuer**, in enrichment ([agent-enrichment.md](agent-enrichment.md)) |
| Receita Federal CNPJ database (open data) | Files of several GB: too large for a local app; it would serve for the category by CNAE (activity code) |
| Registrato (BCB) | Requires the user's gov.br login: not a public API |
| Open Finance Brasil | Only for regulated participating institutions |
| Portal da Transparência / Open Data Portal | Requires a key and brings no data useful to personal finance beyond what the app already has |
| DIEESE (basic basket) | No API; only spreadsheets and PDFs |
| ANP (fuel prices) | Useful, but with little impact on the current screens; a candidate for a car cost panel |
| Tesouro Direto (securities on sale) | Already covered by Tesouro Transparente |
