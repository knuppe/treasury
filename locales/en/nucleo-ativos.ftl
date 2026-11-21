# Assets, market and indices (core: `assets`, `market`, `indices`): validation errors, history
# reasons, warnings and errors from fetching quotes and indices, provider names and terms, and
# the names and notes of the countries in the catalog, in the installation language. Class names
# come from carteira.ftl (`carteira-classe-*`) and country names from nucleo-pais.ftl; index
# series names (stored in the vault) stay as they are.
# Revision: AI draft, not yet reviewed by a native speaker.

## Assets

nucleo-ativos-ref-ativo = asset { $id }
nucleo-ativos-motivo-cadastrado = Asset added: { $ativo }
nucleo-ativos-motivo-atualizado = Asset updated
nucleo-ativos-motivo-removido = Asset removed
nucleo-ativos-erro-classe-invalida = invalid asset class: { $classe }
nucleo-ativos-erro-taxa-invalida = invalid rate: { $taxa }
nucleo-ativos-erro-nome = enter the asset name
nucleo-ativos-erro-renda-fixa-termos = fixed income needs its terms (index, rate, maturity)
nucleo-ativos-erro-previdencia-classe = the pension plan is for assets in the Pension plans class
nucleo-ativos-erro-bolsa-fora = exchange { $bolsa } is not in the catalog: add the exchange (code, country, currency, time zone and trading hours) first
nucleo-ativos-erro-mercado = market { $mercado } does not exist: use an exchange from the catalog or one of these: { $lista }
nucleo-ativos-erro-bolsa-falta = enter the exchange (LSE, XETRA, TSE…) for stocks and ETFs on other exchanges
nucleo-ativos-erro-duplicado = asset { $ticker } is already registered
nucleo-ativos-erro-moeda-invalida = invalid currency: { $moeda }
nucleo-ativos-erro-moeda-em-uso = the asset already has trades or dividends in this currency; create another asset
nucleo-ativos-erro-bolsa-catalogo = stocks and ETFs on other exchanges need an exchange from the catalog (LSE, XETRA, TSE…)
nucleo-ativos-erro-apagar-em-uso = the asset has trades or dividends; archive it instead of deleting

## Ticker search

nucleo-ativos-tipo-acao = Stock
nucleo-ativos-tipo-fundo = Fund
nucleo-ativos-erro-busca-yahoo = Yahoo search unavailable: { $erro }

## Exchanges

nucleo-ativos-ref-bolsa = exchange { $bolsa }
nucleo-ativos-motivo-bolsa-salva = Exchange saved: { $bolsa }
nucleo-ativos-motivo-bolsa-removida = Exchange removed: { $bolsa }
nucleo-ativos-erro-horario = invalid time: { $horario } (use HH:MM)
nucleo-ativos-erro-fuso = unknown time zone: { $fuso }
nucleo-ativos-erro-bolsa-codigo = exchange code: up to 12 letters and digits (e.g. LSE)
nucleo-ativos-erro-bolsa-nome = enter the exchange name
nucleo-ativos-erro-bolsa-pais = country: 2-letter ISO code (e.g. GB)
nucleo-ativos-erro-bolsa-abertura = the opening time must be before the closing time
nucleo-ativos-erro-bolsa-almoco = the lunch break must be within trading hours
nucleo-ativos-erro-bolsa-dias = trading days: digits from 1 (Monday) to 7 (Sunday)
nucleo-ativos-erro-bolsa-sufixo = the Yahoo suffix starts with a dot (e.g. .L)
nucleo-ativos-erro-bolsa-do-catalogo = exchanges from the catalog can't be deleted (adjust the hours if needed)
nucleo-ativos-erro-bolsa-com-ativos = there are assets on this exchange

## Quotes

nucleo-ativos-erro-chaves-so-interface = data source keys can only be set in the app
nucleo-ativos-erro-chave-fred = the FRED key has 32 letters and digits
nucleo-ativos-erro-token-brapi = the brapi.dev token has letters, digits, hyphens or underscores
nucleo-ativos-erro-preco-invalido = invalid price: { $preco }
nucleo-ativos-erro-preco-positivo = the price must be positive
nucleo-ativos-erro-preco-banco = invalid price in the database: { $preco }
nucleo-ativos-erro-intradiario-periodo = intraday period: 1d or 5d
nucleo-ativos-erro-moeda-yahoo = Yahoo's currency ({ $moeda_yahoo }) differs from the asset's ({ $moeda_ativo })
nucleo-ativos-sem-cotacao-manual = manual quotes: record the prices on the asset
nucleo-ativos-sem-cotacao-fundo-sem-cnpj = fund without a CNPJ: enter the CNPJ to fetch the share value from CVM
nucleo-ativos-sem-cotacao-sem-ticker = no ticker: enter the ticker or the provider symbol on the asset
nucleo-ativos-sem-cotacao-sem-bolsa = no exchange from the catalog: enter the exchange (LSE, XETRA, TSE…) on the asset
nucleo-ativos-sem-cotacao-sem-provedor = class without a default provider: enter the Yahoo symbol on the asset
nucleo-ativos-motivo-desdobramento-yahoo = Split reported by Yahoo: { $ativo }
nucleo-ativos-nota-bonificacao-yahoo = Bonus shares recorded automatically by the quote collection (Yahoo treats them as a split); enter the cost assigned by the company, if any
nucleo-ativos-nota-evento-automatico = Recorded automatically by the quote collection
nucleo-ativos-erro-yahoo-sem-resultado = Yahoo: no result
nucleo-ativos-erro-yahoo-sem-preco = Yahoo: quote without a price
nucleo-ativos-erro-yahoo-sem-horario = Yahoo: quote without a time
nucleo-ativos-erro-yahoo-intervalo = interval { $intervalo }/{ $periodo } not supported
nucleo-ativos-erro-yahoo-busca = invalid Yahoo search response: { $erro }
nucleo-ativos-erro-cvm-layout = CVM daily report layout not recognized
nucleo-ativos-erro-cvm-status = CVM returned { $status } for { $url }
nucleo-ativos-erro-cvm-fundo-ausente = fund { $fundo } not found in the CVM daily report
nucleo-ativos-erro-tesouro-titulo-ausente = bond not found in Tesouro Transparente: { $titulo }
nucleo-ativos-erro-tesouro-colunas = the CSV layout changed: expected columns are missing ({ $colunas })
nucleo-ativos-erro-tesouro-poucas-linhas = CSV with too few valid rows ({ $linhas }): the file may be incomplete or may have changed
nucleo-ativos-erro-tesouro-linhas-ruins = { $ruins } of { $total } rows unreadable or with a price out of range: the format may have changed
nucleo-ativos-erro-tesouro-parado = stale data: the latest price is from { $data }
nucleo-ativos-erro-tesouro-http = Tesouro Transparente returned HTTP { $status }
nucleo-ativos-erro-tesouro-pagina = Tesouro Transparente returned a web page instead of the CSV

## Indices: country catalog

nucleo-ativos-erro-pais-fora = country not in the catalog: { $pais }
nucleo-ativos-erro-pais-so-interface = the reference country can only be changed in the app
nucleo-ativos-pais-nota-br = Same as always: IPCA, CDI/Selic and PTAX from the Central Bank.
nucleo-ativos-pais-nota-us = FRED's CPI-U requires the free key; without it, the OECD CPI (no key) is used.
nucleo-ativos-pais-nota-banco-mundial = Annual inflation from the World Bank, interpolated month by month (estimated).

## Indices: providers (name and terms of use)

nucleo-ativos-provedor-bcb_sgs-nome = Central Bank of Brazil (SGS, PTAX, Focus)
nucleo-ativos-provedor-bcb_sgs-termos = BCB open data, free to use with attribution.
nucleo-ativos-provedor-fred-nome = FRED (Federal Reserve Bank of St. Louis)
nucleo-ativos-provedor-fred-termos = Free with a personal key; each series follows the terms of its original source.
nucleo-ativos-provedor-yahoo-nome = Yahoo Finance
nucleo-ativos-provedor-yahoo-termos = Unofficial source, no guarantees; used only for daily closing prices.
nucleo-ativos-provedor-ecb-nome = European Central Bank (ECB Data Portal)
nucleo-ativos-provedor-ecb-termos = Free reuse with attribution (ECB Data Portal terms).
nucleo-ativos-provedor-eurostat-nome = Eurostat
nucleo-ativos-provedor-eurostat-termos = Free reuse with attribution (European Commission open data policy, CC BY 4.0).
nucleo-ativos-provedor-oecd-nome = OECD (OECD Data Explorer)
nucleo-ativos-provedor-oecd-termos = OECD data under CC BY 4.0; the public API limits the number of queries per hour.
nucleo-ativos-provedor-boc-nome = Bank of Canada (Valet)
nucleo-ativos-provedor-boc-termos = Free to use with attribution (Bank of Canada terms).
nucleo-ativos-provedor-ons-nome = Office for National Statistics (United Kingdom)
nucleo-ativos-provedor-ons-termos = Open Government Licence v3.0.
nucleo-ativos-provedor-boe-nome = Bank of England (Database)
nucleo-ativos-provedor-boe-termos = Free to use with attribution (Bank of England terms).
nucleo-ativos-provedor-worldbank-nome = World Bank (World Development Indicators)
nucleo-ativos-provedor-worldbank-termos = CC BY 4.0. Annual inflation; the months are interpolated and marked as estimated.
nucleo-ativos-provedor-ibge_sidra-nome = IBGE (SIDRA)
nucleo-ativos-provedor-ibge_sidra-termos = IBGE public data, free to use with attribution.
nucleo-ativos-provedor-nyfed-nome = Federal Reserve Bank of New York (Markets Data)
nucleo-ativos-provedor-nyfed-termos = Federal Reserve Bank of New York public data, free to use with attribution.

## Indices: collection

nucleo-ativos-erro-resposta-invalida = invalid response from { $fonte }: { $erro }
nucleo-ativos-erro-resposta-invalida-sem-detalhe = invalid response from { $fonte }
nucleo-ativos-aviso-sem-chave-fred = to download US CPI, M2 and Fed Funds, add the free FRED key in Settings › Data sources
nucleo-ativos-erro-numero = invalid number: { $numero }
nucleo-ativos-erro-periodo = invalid period: { $periodo }
nucleo-ativos-erro-decimal-banco = invalid decimal in the database: { $valor }
nucleo-ativos-erro-fonte-desconhecida = unknown source: { $fonte }
nucleo-ativos-erro-adaptador-proprio = { $fonte } is collected by its own adapter
nucleo-ativos-erro-oecd-sem-resposta = no response from the OECD
nucleo-ativos-erro-codigo-ons = invalid ONS code: { $codigo }
nucleo-ativos-erro-codigo-sidra = invalid SIDRA code: { $codigo }
nucleo-ativos-erro-sgs-data = invalid SGS date: { $data }
nucleo-ativos-erro-boc-sem-observacoes = Bank of Canada response without observations
nucleo-ativos-erro-eurostat-sem-tempo = Eurostat response without the time dimension
nucleo-ativos-erro-eurostat-varias-series = the Eurostat query must pick a single series (geo, unit and coicop18)
nucleo-ativos-erro-eurostat-indice-tempo = invalid Eurostat time index
nucleo-ativos-erro-eurostat-valor = invalid Eurostat value: { $valor }
nucleo-ativos-erro-eurostat-posicao = invalid Eurostat position
nucleo-ativos-erro-eurostat-valores = invalid Eurostat values
nucleo-ativos-erro-nyfed-sem-taxas = NY Fed response without rates
nucleo-ativos-erro-sdmx-nao-csv = unexpected SDMX response (not CSV)
nucleo-ativos-erro-sdmx-csv = invalid SDMX CSV: { $erro }
nucleo-ativos-erro-sdmx-coluna = SDMX CSV without the { $coluna } column
nucleo-ativos-erro-ons-mes = invalid ONS month: { $mes }
nucleo-ativos-erro-boe-nao-csv = unexpected Bank of England response (not CSV)
nucleo-ativos-erro-boe-data = invalid Bank of England date: { $data }

## Asset data with sources (enrichment, phase 1)

nucleo-ativos-motivo-dados = Asset data completed from sources
nucleo-ativos-motivo-campo-travado = Asset field locked for suggestions
nucleo-ativos-motivo-campo-destravado = Asset field unlocked for suggestions
nucleo-ativos-dados-erro-campo = unknown asset data field: { $campo }
nucleo-ativos-dados-erro-vazio = no field chosen
nucleo-ativos-dados-erro-repetido = the { $campo } field of { $ativo } appears more than once
nucleo-ativos-dados-erro-travado = the { $campo } field of { $ativo } is locked for suggestions
nucleo-ativos-dados-erro-sem-fonte = the value "{ $valor }" for { $campo } of { $ativo } is not in any source recorded in the vault
nucleo-ativos-dados-erro-preenchido = the { $campo } field of { $ativo } already has a value of yours; to change it, ask for the replacement (replace)
nucleo-ativos-dados-efeito-score = the score starts comparing with { $segmento } peers

## AI study

nucleo-ativos-estudo-ref = study { $id }
nucleo-ativos-estudo-erro-contexto = the context for the AI can have at most { $max } characters
nucleo-ativos-estudo-erro-pergunta = the question can have at most { $max } characters
nucleo-ativos-estudo-erro-sem-motor = No AI engine configured. Add one in Settings › AI and agent and put it on the "Asset studies" route or the chat route.
nucleo-ativos-estudo-erro-formato = The engine replied outside the study format. Try again or choose another engine.
nucleo-ativos-estudo-erro-aviso = Before the first study, read and accept the notice in the asset's AI study tab.
nucleo-ativos-estudo-erro-rodando = A study is already being generated; wait for it to finish.
nucleo-ativos-estudo-erro-so-interface = only from the app's interface
nucleo-ativos-estudo-titulo-job = AI study
nucleo-ativos-motivo-estudo = AI study: { $ativo }
nucleo-ativos-motivo-contexto-estudo = Asset context for the AI
nucleo-ativos-motivo-estudo-apagado = AI study deleted
nucleo-ativos-motivo-estudo-pedido = AI study requested by the agent: { $ativo }
nucleo-ativos-estudo-pronto = Study ready
nucleo-ativos-consulta-cvm-sem-registro = The CVM fund registry has not been downloaded yet: use "Collect now" in Asset research collection and look it up again.

## brapi.dev

nucleo-ativos-brapi-sem-token = brapi.dev has no token: add one in Settings › Data sources.
nucleo-ativos-erro-cripto-sem-cambio = no US dollar exchange rate for { $moeda }: crypto quotes come in dollars (run the index sync or enter the rate)
nucleo-ativos-erro-fonte-sem-chave = the source's key is missing: add it in Settings › Data sources
nucleo-ativos-erro-moeda-fonte = the source's currency ({ $moeda_fonte }) differs from the asset's ({ $moeda_ativo })
nucleo-ativos-motivo-desdobramento-coleta = Split reported by the price download: { $ativo }
nucleo-ativos-erro-eodhd-rede = couldn't reach EODHD
nucleo-ativos-erro-chave-fonte = invalid key: check the value copied from the source's site
nucleo-ativos-brapi-ja-hoje = brapi.dev: this ticker's quote was already fetched today (one per day, to save quota).
nucleo-ativos-brapi-limite = brapi.dev: this month's usage limit was reached; quotes come back after the renewal or with a higher limit in Settings › Data sources.
nucleo-ativos-erro-limite-brapi = the brapi usage limit goes from 10% to 100% of the quota
nucleo-ativos-erro-conta-custodia = custody account not found: { $conta }
nucleo-ativos-erro-custodia-cartao = the account where the asset is held can't be a credit card
