# External data sources (core: `sources.rs`): name and use of each source in Settings › Data
# sources and the errors shown when one is turned off, in the installation language. Proper names
# (Yahoo Finance, Tesouro Direto, Bank of Canada…) stay the same.
# Revision: AI draft, not yet reviewed by a native speaker.

## Name and use of each source

nucleo-fontes-yahoo-nome = Yahoo Finance
nucleo-fontes-yahoo-uso = Daily and intraday quotes, dividends, splits and ticker search.
nucleo-fontes-fundamentus-nome = Fundamentus
nucleo-fontes-fundamentus-uso = Indicators (P/E, P/B, DY, ROE…), dividends and technical data for stocks and FIIs, with a local cache.
nucleo-fontes-tesouro-nome = Tesouro Direto
nucleo-fontes-tesouro-uso = Prices and rates of Brazilian government bonds.
nucleo-fontes-feedback-titulo = { $fonte } collection failing
nucleo-fontes-feedback-corpo-passageira = The { $fonte } collection failed { $dias } days in a row with a network or site error (the source stays on, with no risk of saving wrong data). Last error: { $erro }. To investigate: did the site ({ $site }) change its address or start limiting or blocking requests, or is the problem the user's network (filter, proxy, antivirus)?
nucleo-fontes-feedback-corpo-desligada = The { $fonte } collection failed { $dias } days in a row and the source was turned off so as not to save wrong data. Last error: { $erro }. Check whether the site ({ $site }) changed the address, the format or the file's columns.
nucleo-fontes-feedback-nota = Still failing ({ $dias } days in a row): { $erro }
nucleo-fontes-feedback-resolvido = The collection worked again on { $data }.
nucleo-fontes-cvm-nome = CVM Open Data
nucleo-fontes-cvm-uso = Fund share values (daily report), company financial statements, FII reports and filings.
nucleo-fontes-b3-nome = B3 (COTAHIST and listed companies)
nucleo-fontes-b3-uso = Spot market closing prices and volume, plus official dividends and corporate actions (dividends, JCP, FII income, splits) for the asset analysis universe.
nucleo-fontes-bcb-nome = Central Bank of Brazil
nucleo-fontes-bcb-uso = Indices (CDI, Selic, IPCA, TR…), PTAX exchange rate, Focus, the official list of banks and the banks' prudential indicators (IF.data).
nucleo-fontes-sec-nome = SEC EDGAR
nucleo-fontes-sec-uso = Fundamentals of US companies.
nucleo-fontes-fred-nome = FRED (St. Louis Fed)
nucleo-fontes-fred-uso = US indices (inflation, interest rates).
nucleo-fontes-ecb-nome = European Central Bank
nucleo-fontes-ecb-uso = Euro area exchange and interest rates.
nucleo-fontes-eurostat-nome = Eurostat
nucleo-fontes-eurostat-uso = Inflation of European countries.
nucleo-fontes-oecd-nome = OECD
nucleo-fontes-oecd-uso = Inflation and interest rates of other countries in the catalog.
nucleo-fontes-boc-nome = Bank of Canada
nucleo-fontes-boc-uso = Canadian indices.
nucleo-fontes-ons-nome = ONS (United Kingdom)
nucleo-fontes-ons-uso = UK inflation.
nucleo-fontes-boe-nome = Bank of England
nucleo-fontes-boe-uso = UK interest rates.
nucleo-fontes-world_bank-nome = World Bank
nucleo-fontes-world_bank-uso = Annual inflation of countries without a monthly source.
nucleo-fontes-fipe-nome = FIPE table
nucleo-fontes-fipe-uso = Valuation of vehicles in your physical assets.
nucleo-fontes-coingecko-nome = CoinGecko
nucleo-fontes-coingecko-uso = Market cap, supply, volume and development activity of the registered crypto assets (public API, no key).
nucleo-fontes-defillama-nome = DefiLlama
nucleo-fontes-defillama-uso = Network TVL, stablecoin backing and peg, US dollar prices of the registered crypto assets and the list of network hacks.
nucleo-fontes-coin_metrics-nome = Coin Metrics (Community)
nucleo-fontes-coin_metrics-uso = Daily active addresses of the networks covered by the open API (CC BY-NC 4.0), for the crypto score.
nucleo-fontes-mempool-nome = mempool.space
nucleo-fontes-mempool-uso = Bitcoin blocks of the week by mining pool, for the Nakamoto coefficient of the crypto score.
nucleo-fontes-brapi-nome = brapi.dev
nucleo-fontes-brapi-uso = Optional, with a token: a second source of indicators and dividends for B3 stocks and FIIs, to cross-check the official data (registered assets only).
nucleo-fontes-tiingo-nome = Tiingo
nucleo-fontes-tiingo-uso = Optional, with your free key: daily close, dividends and splits of US stocks, ETFs and REITs, instead of Yahoo.
nucleo-fontes-eodhd-nome = EODHD
nucleo-fontes-eodhd-uso = Optional, with your key: daily close, dividends and splits from exchanges around the world, instead of Yahoo. EODHD's free plan is for testing only.
nucleo-fontes-gleif-nome = GLEIF
nucleo-fontes-gleif-uso = LEI (global legal entity identifier) of Brazilian issuers, by CNPJ, in the asset data.
nucleo-fontes-brasilapi-nome = BrasilAPI (Federal Revenue)
nucleo-fontes-brasilapi-uso = Legal name and registration status of the issuers' and funds' CNPJ, from the public mirror of the Federal Revenue registry; only the CNPJ is sent.
nucleo-fontes-ibge-nome = IBGE (SIDRA)
nucleo-fontes-ibge-uso = IPCA by group (food, housing, transportation…), for the family's personal inflation.
nucleo-fontes-sefaz-nome = SEFAZ (NFC-e lookup)
nucleo-fontes-sefaz-uso = Items, amounts and approximate taxes of a store receipt, through the state treasury's public lookup (the link in the receipt's QR code).
nucleo-fontes-siconfi-nome = Tesouro Nacional (Siconfi)
nucleo-fontes-siconfi-uso = Tax revenue of the family's municipality (IPTU, ISS, ITBI and fees) and its population, from the report the city files with the national treasury; only the municipality code is sent.
nucleo-fontes-mail_autoconfig-nome = Email autoconfiguration (Thunderbird)
nucleo-fontes-mail_autoconfig-uso = IMAP server of a domain outside the catalog, in the email import wizard: the domain's own autoconfig and the Thunderbird database (only the domain is sent, never the address).
nucleo-fontes-ny_fed-nome = Federal Reserve Bank of New York
nucleo-fontes-ny_fed-uso = Effective Fed Funds rate (EFFR), daily and without a key: the reference interest rate for families living in the United States.

## Errors

nucleo-fontes-erro-desligada = the { $fonte } source is turned off in Settings › Data sources
nucleo-fontes-erro-so-interface = data sources can only be turned on and off in the app
nucleo-fontes-open_food_facts-nome = Open Food Facts
nucleo-fontes-open_food_facts-uso = Name, brand, nutrition facts, NOVA group and allergens of products by barcode; only the barcode is sent, once per product. Open data (ODbL), indicative, not for medical use.
nucleo-fontes-news_feeds-nome = News sites (RSS)
nucleo-fontes-news_feeds-uso = Headlines from InfoMoney, Money Times, Seu Dinheiro, E-Investidor and Suno for the asset's News tab. The request is the same for everyone: the ticker filter runs on this computer, so nothing about your portfolio leaves it. Only title, outlet, date and link.

## Terms, license and attribution of each source

nucleo-fontes-yahoo-termos = Unofficial source. Yahoo has no public API, and its terms of use don't authorize automated collection or storing the data. With this source on, the requests leave your computer by your choice; turn it off if you want official sources only.
nucleo-fontes-fundamentus-termos = The site publishes no terms of use. Treasury reads only the pages of your assets, at most once a day, and doesn't redistribute the data.
nucleo-fontes-tesouro-termos = Source: Brazilian National Treasury, Tesouro Transparente. Open data under the ODbL license.
nucleo-fontes-cvm-termos = Source: CVM (Brazil's securities commission), Open Data Portal. Open data under the ODbL license.
nucleo-fontes-b3-termos = B3 data for strictly personal use, as the site's terms require. It stays in your vault only and isn't redistributed.
nucleo-fontes-sec-termos = Source: SEC, EDGAR. Public data; the SEC asks whoever queries it to identify themselves with a contact email.
nucleo-fontes-bcb-termos = Source: Banco Central do Brasil, open data.
nucleo-fontes-fred-termos = This product uses the FRED® API but is not endorsed or certified by the Federal Reserve Bank of St. Louis.
nucleo-fontes-ecb-termos = Source: European Central Bank (ECB). Free reuse with attribution.
nucleo-fontes-eurostat-termos = Source: Eurostat. CC BY 4.0 license.
nucleo-fontes-oecd-termos = Source: OECD. CC BY 4.0 license.
nucleo-fontes-boc-termos = Source: Bank of Canada. Values marked as estimated are Treasury's own calculation.
nucleo-fontes-ons-termos = Source: Office for National Statistics. Contains public sector information licensed under the Open Government Licence v3.0.
nucleo-fontes-boe-termos = Source: Bank of England. SONIA and/or SONIA Compounded Index data licensed under the Open Government Licence v3.0 and copyright the Governor and Company of the Bank of England. The trade marks “Bank of England” and “SONIA” are registered trade marks of the Bank of England.
nucleo-fontes-world_bank-termos = Source: World Bank, World Development Indicators. CC BY 4.0 license. The monthly series is Treasury's estimate from the annual figure.
nucleo-fontes-fipe-termos = The FIPE Table belongs to Fundação Instituto de Pesquisas Econômicas (FIPE). The lookup goes through parallelum.com.br, an unofficial service; the official lookup is at veiculos.fipe.org.br.
nucleo-fontes-coingecko-termos = Data provided by CoinGecko (coingecko.com).
nucleo-fontes-defillama-termos = Source: DefiLlama (defillama.com), open API.
nucleo-fontes-coin_metrics-termos = Source: Coin Metrics (coinmetrics.io), Community data under the CC BY-NC 4.0 license, for non-commercial use.
nucleo-fontes-mempool-termos = Source: mempool.space, an open source project.
nucleo-fontes-brapi-termos = Personal use, with your own token. The data stays in your vault and isn't redistributed.
nucleo-fontes-ibge-termos = Source: IBGE, SIDRA.
nucleo-fontes-ny_fed-termos = © Federal Reserve Bank of New York. The Effective Federal Funds Rate (EFFR) is subject to the Terms of Use posted at newyorkfed.org. The New York Fed is not responsible for publication of the EFFR by Treasury, does not sanction or endorse any particular republication, and has no liability for your use.
nucleo-fontes-sefaz-termos = Public lookup of the receipt, through the QR link on your receipt, made at your request.
nucleo-fontes-siconfi-termos = Source: Tesouro Nacional, Siconfi. Open data, made available for free use; the figures are those reported by each city government.
nucleo-fontes-mail_autoconfig-termos = Thunderbird's public mail configuration database (ISPDB).
nucleo-fontes-gleif-termos = Source: GLEIF. LEI data in the public domain (CC0).
nucleo-fontes-brasilapi-termos = Source: BrasilAPI, an open project, with public company registry (CNPJ) data.
nucleo-fontes-open_food_facts-termos = Source: Open Food Facts and its contributors. Database under the ODbL license.
nucleo-fontes-news_feeds-termos = Headlines from each outlet's public RSS feed. The text belongs to the outlet, and the link leads to the article on its site.
nucleo-fontes-tiingo-termos = Personal use, with your own key: Tiingo's free plan terms don't allow showing or passing the data on to other people. It stays in your vault only.
nucleo-fontes-eodhd-termos = Personal, non-professional use, with your own key: EODHD's terms allow storing and analyzing the data for private use and forbid passing it on. It stays in your vault only.
