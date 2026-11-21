# Indices and quotes: index series, chart, exchange rate, FRED key and collection report
# (docs/i18n-design.md). Revision: AI draft, not yet reviewed by a native speaker.

## What each series is for

indices-uso-ipca = Default deflator (BRL) and Tesouro IPCA+
indices-uso-igpm = Contracts and bonds indexed to IGP-M
indices-uso-cdi = Floating-rate fixed income and benchmark
indices-uso-selic = Tesouro Selic and Selic bonds
indices-uso-selic-meta = Target set by Copom
indices-uso-tr = Savings and loans
indices-uso-m2-br = Alternative deflator: Brazil money supply growth
indices-uso-us-cpi = Deflator for USD assets
indices-uso-us-m2 = Alternative deflator: US money supply growth
indices-uso-fed-funds = US policy rate
indices-uso-ibov = Brazilian stock benchmark
indices-uso-sp500 = US stock benchmark
indices-uso-estr = Euro area reference rate
indices-uso-corra = Canada reference rate
indices-uso-sonia = UK reference rate
indices-uso-boe-bank-rate = UK policy rate
indices-uso-hicp-ea = Euro area inflation
indices-uso-cpi-ca = Canada inflation
indices-uso-cpi-gb = UK inflation
indices-uso-cpih-gb = UK inflation including housing
indices-uso-cpi-us-oecd = US inflation without a key (alternative to FRED)
indices-uso-inflacao-pais = Country inflation
indices-uso-wb-cpi = Annual inflation (World Bank), months estimated
indices-uso-rate3m = 3-month rate (reference)
indices-uso-fx = Reference exchange rate

## Series table

indices-freq-diaria = Daily
indices-freq-mensal = Monthly
indices-serie = Series
indices-unidade = Unit
indices-freq = Freq.
indices-oficial-ate = Official through
indices-estimado-ate = Estimated through
indices-pontos = Points
indices-coletado-em = Collected on
indices-series = Index series
indices-figura-series = { $figura } · Index series
indices-nenhuma-serie = No series registered.
indices-selecione = Select a series (↑/↓) to see the chart. Estimates appear in italics with ≈ until the official figure comes out.

## Single-series chart

indices-oficial = Official
indices-estimado = Estimated
indices-titulo-meses = { $serie }, { $n } months
indices-titulo-anos = { $serie }, { $n } years
indices-periodo = Series period
indices-periodo-5a = 5Y
indices-fonte-unidade = Source: { $fonte } · { $unidade }
indices-nota-ipca = Months not yet released use the Focus projection (or the latest figure) until IBGE publishes.
indices-nota-wb = World Bank annual inflation spread month by month: all points are estimated.
indices-sem-valores = No values in the period. Update the indices.
indices-grafico-aria = Series { $serie } in the period, in { $unidade }.
indices-valores-de = Values of { $serie }
indices-status = Status

## Collection report

indices-gravados = Saved
indices-ultimo-oficial = Latest official
indices-resultado = Result
indices-ok = ok
indices-cotacoes = Quotes
indices-desdobramentos = Splits
indices-ultimo-pregao = Last session
indices-relatorio = Collection report
indices-coleta-concluida = Collection finished
indices-coleta-falhas = { $n ->
    [one] Collection finished with { $n } failure
   *[other] Collection finished with { $n } failures
}
indices-coleta-resumo = { $inicio } → { $fim } · { $series } series, { $moedas } currencies and { $ativos } assets.
indices-ipca-estimado = { $n ->
    [one] {" "}IPCA with { $n } estimated month (Focus).
   *[other] {" "}IPCA with { $n } estimated months (Focus).
}
indices-indices-cambio = Indices and exchange rates
indices-coleta-indices = Index and exchange rate collection
indices-coleta-cotacoes = Quote collection
indices-sem-fonte = No asset with trades or on the watchlist has an automatic quote source.

## FRED key

indices-fred = FRED key
indices-fred-cadastrada = FRED key saved
indices-fred-removida = FRED key removed
indices-trocar = Change
indices-fred-rotulo = FRED key (US)
indices-fred-dica = Free at fred.stlouisfed.org; needed for US CPI, US M2 and Fed Funds. It is encrypted in the vault and never comes back to the interface.
indices-salvar-chave = Save key
indices-sem-fred-titulo = US series without a FRED key
indices-sem-fred-texto = US CPI, US M2 and Fed Funds are only collected with the free FRED key; without them, the US CPI and US M2 deflators have no data.

## Market data collection

indices-coleta-titulo = Market data collection
indices-coletando = Collecting
indices-coletando-reticencias = Collecting…
indices-ultima-coleta = Last collection
indices-rotina = Schedule
indices-rotina-valor = every 12 h while the app is open
indices-coleta-ajuda = Central Bank of Brazil indices (IPCA, CDI, Selic, IGP-M, TR, M2), PTAX exchange rate, US series from FRED and closing quotes (Yahoo Finance for B3, the US and crypto; Tesouro Transparente for Tesouro Direto). Each value keeps its source and collection date.
indices-atualizados = Indices, exchange rates and quotes updated
indices-concluida-com-falhas = Collection finished with failures
indices-coleta-falhou = The collection failed
indices-atualizar = Update indices, exchange rates and quotes
indices-pode-demorar = This may take a few minutes
indices-cambio-fontes = Exchange rate and sources
indices-em-data = on { $data }
