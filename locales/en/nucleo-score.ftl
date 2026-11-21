# Asset health score (core: `score`): scores, bands, explanations, data source labels, coverage
# notes, reasons for missing data, warnings and errors, in the installation language. The
# methodology labels and formulas (methodology/**: classes, segments, profiles, dimensions and
# metrics) stay as they are; band names come from score.ftl (`score-faixa-*`).
# Revision: AI draft, not yet reviewed by a native speaker.

## Bands, notice and reasons for no score

nucleo-score-aviso = Score calculated from public data and rules from the literature, not yet validated against history (draft methodology). Not investment advice.
nucleo-score-faixa-robusto = Solid fundamentals on the measured criteria
nucleo-score-faixa-saudavel = OK; monitor quarterly
nucleo-score-faixa-atencao = Deterioration or relevant weakness; review the thesis
nucleo-score-faixa-risco = Compromised fundamentals on the measured criteria
nucleo-score-nao-implementado = The score for { $classe } hasn't been implemented yet (first release: Brazilian stocks, banks and FIIs).
nucleo-score-sem-score = No score: { $motivo }.
nucleo-score-sem-score-falta = No score: { $metricas } is missing, essential for this type of asset.
nucleo-score-parcial = Partial score: { $imputado }% of the weight was imputed (limit { $limite }%), so there is no final band.
nucleo-score-sem-metricas = No computable metrics.
nucleo-score-dimensao-dados = Data
nucleo-score-dado-demonstracoes = Financial statements (CVM)
nucleo-score-sem-demonstracoes = no CVM financial statements downloaded for this asset (run the asset analysis collection)

## Engine: values, peers, notes and explanations

nucleo-score-valor-dias = { $valor } days
nucleo-score-valor-anos = { $valor } years
nucleo-score-valor-bi = R$ { $valor } bn
nucleo-score-valor-mi = R$ { $valor } M
nucleo-score-valor-mil = R$ { $valor }k
nucleo-score-pares-do-setor = in the { $setor } sector
nucleo-score-pares-do-segmento = in the { $segmento } segment
nucleo-score-pares-da-classe = in the class
nucleo-score-sem-dado = no data
nucleo-score-nota-defasados = Outdated data: the latest is from { $data } (more than 2 quarters).
nucleo-score-nota-sem-data = No date for the data.
nucleo-score-nota-confianca = Confidence × { $fator }: { $motivo }.
nucleo-score-pior-caso = worst case
nucleo-score-explica-pares = { $metrica } { $valor }: p{ $percentil } among { $pares } peers { $nivel }
nucleo-score-explica-absoluta = { $metrica } { $valor }: score { $nota } by the absolute reference
nucleo-score-lista = { $itens } and { $ultimo }

## Stocks: confidence, Magic Formula and notes

nucleo-score-confianca-fonte-secundaria = secondary source (Fundamentus) for { $metricas }
nucleo-score-confianca-liquidez-baixa = low liquidity
nucleo-score-confianca-divergencia = difference above 25% from Fundamentus in some indicator
nucleo-score-magic-roc = Return on capital (EBIT ÷ working capital + fixed assets)
nucleo-score-magic-nota = Diagnostic: ROC counts in Profitability and EY in Valuation
nucleo-score-nota-financeira = Financial institution: Altman, Piotroski, Beneish and EBITDA don't apply.

## Commands

nucleo-score-erro-segmento-so-interface = the score segment can only be changed in the app
nucleo-score-erro-avaliacao-so-interface = the score feedback can only be given in the app
nucleo-score-erro-eventos-so-interface = score events can only be managed in the app
nucleo-score-motivo-avaliacao = asset health score feedback
nucleo-score-motivo-evento = qualitative health score event
nucleo-score-motivo-decisao-evento = decision on a health score event
nucleo-score-motivo-apagar-evento = delete a health score event

## Banks (IF.data)

nucleo-score-banco-grupo-grandes = Large banks (S1–S2)
nucleo-score-banco-grupo-medios = Mid-sized banks (S3)
nucleo-score-banco-grupo-pequenos = Small banks (S4)
nucleo-score-banco-trimestre = quarter to { $data }
nucleo-score-banco-sem-dados = no Central Bank IF.data for this bank
nucleo-score-banco-sem-basileia = no Basel ratio in IF.data
nucleo-score-banco-sem-capital-principal = no CET1 capital in IF.data
nucleo-score-banco-sem-alavancagem = no leverage ratio in IF.data
nucleo-score-banco-sem-inadimplencia = no default or loan portfolio data in IF.data
nucleo-score-banco-sem-ativos-problematicos = no problem assets in IF.data
nucleo-score-banco-sem-provisao = no provisions or default data in IF.data
nucleo-score-banco-sem-despesa-provisao = no 12-month provision expense in IF.data
nucleo-score-banco-sem-carteira-anterior = no loan portfolio from 12 months earlier in IF.data
nucleo-score-banco-sem-despesas-receitas = no 12-month expenses and revenue in IF.data
nucleo-score-banco-nota-relatorios = IF.data reports 2 and 4 are only downloaded for S1–S3 banks, listed banks and issuers in the portfolio
nucleo-score-sem-fonte = no source
nucleo-score-banco-nota-prudenciais = Prudential indicators of { $banco } (Central Bank IF.data, { $periodo }); 12-month results add up the half-years ({ $convencao }).
nucleo-score-banco-nota-sem-correspondencia = Bank with no match in the Central Bank IF.data (by the lead institution's CNPJ): run the asset analysis collection with the Central Bank source turned on.
nucleo-score-banco-roe-abaixo = ROE below growth: justified P/B ≤ 0
nucleo-score-banco-pvp-justo = Justified P/B { $pvp } with Ke { $ke }% and g { $g }%
nucleo-score-banco-sem-pvp-roe = no P/B or ROE
nucleo-score-banco-flag-basileia = Basel ratio of { $valor }%

## Crypto

nucleo-score-cripto-nao-comparavel = Crypto can't be compared directly with stocks: the score measures robustness within the crypto asset class.
nucleo-score-cripto-periodo-coleta = collected on { $data }
nucleo-score-cripto-sem-coingecko = no { $dado } from CoinGecko (run the asset analysis collection)
nucleo-score-cripto-dado-valor-mercado = market cap
nucleo-score-cripto-dado-ranking = ranking
nucleo-score-cripto-dado-volume = 24-hour volume
nucleo-score-cripto-dado-volume-valor = volume and market cap
nucleo-score-cripto-dado-fdv = fully diluted value
nucleo-score-cripto-dado-oferta = circulating supply
nucleo-score-cripto-dado-historico = 12-month history
nucleo-score-cripto-dado-usuarios = watchlist users
nucleo-score-cripto-dado-commits = commits
nucleo-score-cripto-dado-estrelas = GitHub stars
nucleo-score-cripto-dado-maxima = all-time high
nucleo-score-cripto-fonte-dolar = { $fonte }; US dollar at R$ { $taxa } ({ $data })
nucleo-score-cripto-sem-ptax = no US dollar rate (PTAX) to convert
nucleo-score-cripto-sem-valor-mercado = no market cap from CoinGecko or DefiLlama (run the asset analysis collection)
nucleo-score-cripto-sem-oferta-maxima = no maximum supply (continuous issuance): supply inflation applies
nucleo-score-cripto-sem-tvl = not a network with TVL on DefiLlama (or the collection is missing)
nucleo-score-cripto-periodo-genese = genesis on { $data }
nucleo-score-cripto-sem-genese = no genesis block date on CoinGecko
nucleo-score-cripto-periodo-media-30d = 30-day average to { $data }
nucleo-score-cripto-nota-coin-metrics = Coin Metrics, open data from the Community API (CC BY-NC 4.0)
nucleo-score-cripto-sem-coin-metrics = the open Coin Metrics API doesn't cover this network (or the collection is missing)
nucleo-score-cripto-periodo-semana = week to { $data }
nucleo-score-cripto-nota-nakamoto = mining pools with more than half of the week's blocks
nucleo-score-cripto-sem-nakamoto = bitcoin only (mining pools from mempool.space); other networks have no free source
nucleo-score-cripto-fonte-ataques = DefiLlama (hacks list)
nucleo-score-cripto-nota-ataques = { $n ->
    [0] no network hacks on the DefiLlama list
    [one] 1 network hack on the DefiLlama list
   *[other] { $n } network hacks on the DefiLlama list
}
nucleo-score-cripto-sem-ataques = no DefiLlama hacks list (run the asset analysis collection)
nucleo-score-cripto-periodo-regra = network rule
nucleo-score-cripto-fonte-categorias = CoinGecko (categories)
nucleo-score-cripto-nota-pow = proof of work: no allocations with unlocks; new issuance counts in supply inflation
nucleo-score-cripto-sem-desbloqueios = token unlock schedules only on paid services (Token Unlocks, Messari)
nucleo-score-cripto-fonte-github = CoinGecko (GitHub data)
nucleo-score-cripto-fonte-precos-dolar = daily US dollar prices (DefiLlama)
nucleo-score-cripto-fonte-cotacoes-app = app quotes in { $moeda }
nucleo-score-periodo-de-a = { $de } to { $ate }
nucleo-score-cripto-poucas-cotacoes = fewer than { $n } quotes in the last 12 months (run the quote collection)
nucleo-score-nota-pico-fundo = peak on { $pico }, trough on { $fundo }
nucleo-score-cripto-menos-12-meses = less than 12 months of quotes
nucleo-score-cripto-periodo-stablecoins = stablecoin list
nucleo-score-cripto-lastro-sem-nota = backing "{ $lastro }" has no score in the methodology
nucleo-score-cripto-fora-stablecoins = not on the DefiLlama stablecoin list (or the collection is missing)
nucleo-score-cripto-periodo-dias = { $de } to { $ate } ({ $dias } days)
nucleo-score-cripto-nota-paridade = peg to 1 { $moeda }
nucleo-score-cripto-sem-precos-paridade = no daily prices in the peg currency ({ $moeda }) for the last 12 months
nucleo-score-cripto-nota-cambio = Volatility and drawdown measured in { $moeda }: they include exchange rate changes.
nucleo-score-cripto-lastro-fiat = fiat currency and bonds
nucleo-score-cripto-lastro-cripto = other crypto assets (overcollateralized)
nucleo-score-cripto-lastro-algoritmico = algorithmic, no reserves
nucleo-score-cripto-flag-desvio = maximum deviation of { $desvio }
nucleo-score-cripto-flag-algoritmico = algorithmic backing (DefiLlama)
nucleo-score-cripto-flag-fdv = FDV { $vezes } the market cap
nucleo-score-cripto-flag-valor-mercado = market cap of { $valor }
nucleo-score-cripto-dado-criptoativo = Crypto asset data
nucleo-score-cripto-so-cadastrados = the crypto score only applies to registered crypto assets
nucleo-score-cripto-nota-id = CoinGecko id: { $id }.
nucleo-score-cripto-nota-coingecko-antiga = CoinGecko data collected on { $data }, more than 24 hours ago: run the asset analysis sync to refresh it.
nucleo-score-cripto-nota-sem-id = Crypto asset without a CoinGecko id: enter the ticker (symbol) or set the id with the "coingecko" quote source and the id in the provider symbol.
nucleo-score-cripto-nota-sem-pares = Crypto assets have no peer universe in the app: the scores use only the class's absolute references.

## Crypto: source errors

nucleo-score-erro-coingecko = invalid response from CoinGecko: { $erro }
nucleo-score-erro-coingecko-sem-moedas = CoinGecko response without the coin list
nucleo-score-erro-coingecko-sem-id = CoinGecko response without an id
nucleo-score-erro-defillama = invalid response from DefiLlama: { $erro }
nucleo-score-erro-defillama-sem-redes = DefiLlama response without the network list
nucleo-score-erro-defillama-sem-stablecoins = DefiLlama response without the stablecoin list
nucleo-score-erro-defillama-sem-valores = DefiLlama response without market values
nucleo-score-erro-defillama-sem-precos = DefiLlama response without prices
nucleo-score-erro-coin-metrics-catalogo = invalid Coin Metrics catalog: { $erro }
nucleo-score-erro-coin-metrics-sem-ativos = Coin Metrics catalog without the asset list
nucleo-score-erro-coin-metrics = invalid response from Coin Metrics: { $erro }
nucleo-score-erro-coin-metrics-sem-serie = Coin Metrics response without the series
nucleo-score-erro-mempool = invalid response from mempool.space: { $erro }
nucleo-score-erro-mempool-sem-blocos = mempool.space has no blocks for the week
nucleo-score-erro-ataques = invalid DefiLlama hacks list: { $erro }
nucleo-score-erro-ataques-sem-lista = DefiLlama hacks response without the list

## Crypto: collection steps

nucleo-score-cripto-doc-coingecko = CoinGecko /coins/{ $id } on { $data }
nucleo-score-cripto-passo-tvl = DefiLlama: network TVL
nucleo-score-cripto-passo-stablecoins = DefiLlama: stablecoins
nucleo-score-cripto-passo-ids = CoinGecko: ids by symbol
nucleo-score-cripto-passo-historico = CoinGecko: 12-month history of { $id }
nucleo-score-cripto-passo-precos = DefiLlama: daily prices of { $id }
nucleo-score-cripto-passo-valor-mercado = DefiLlama: market cap
nucleo-score-cripto-passo-ataques = DefiLlama: network hacks
nucleo-score-cripto-passo-redes = Coin Metrics: networks in the open API
nucleo-score-cripto-passo-enderecos = Coin Metrics: active addresses
nucleo-score-cripto-passo-pools = mempool.space: mining pools

## Stocks: red flags, cross-checks and secondary sources

nucleo-score-flag-beneish = M = { $valor } (latest fiscal year)
nucleo-score-flag-cobertura = coverage of { $ultimo }x and { $anterior }x in the last two quarters (12 months)
nucleo-score-flag-fcf-negativo = CFO − capex negative in the last 3 fiscal years
nucleo-score-flag-dividendos-acima = dividends paid above free cash flow in the last 2 fiscal years
nucleo-score-flag-diluicao = shares outstanding +{ $pct }% in 12 months
nucleo-score-flag-patrimonio-negativo = negative equity on the latest balance sheet
nucleo-score-flag-entrega-atrasada = { $documento } for { $referencia } filed on { $entrega }
nucleo-score-flag-auditor = { $antiga } replaced by { $nova } after { $anos } years
nucleo-score-flag-volume-medio = average volume of { $valor }
nucleo-score-recon-proventos-12m = Dividends per share in 12 months ({ $agenda } × B3)
nucleo-score-fonte-b3-oficial = B3 (official)
nucleo-score-recon-valor-diferente = different amount
nucleo-score-recon-so-b3 = only in B3
nucleo-score-recon-fora-b3 = not in B3
nucleo-score-recon-provento = { $tipo ->
    [amortizacao] Amortization
   *[provento] Dividend
} with ex-date { $data } ({ $fonte }: { $situacao })
nucleo-score-recon-nota-divergente = { $ticker } dividends checked against B3 (official) over 24 months: { $iguais } equal, { $diferentes } with a different amount, { $so_b3 } only in B3, { $so_agenda } only in the calendar ({ $agenda }). The DY uses B3, the official source.
nucleo-score-recon-nota-igual = { $ticker } dividends checked against B3 (official) over 24 months: { $iguais } equal to the calendar ({ $agenda }).
nucleo-score-fonte-brapi = brapi.dev, read on { $data }
nucleo-score-fonte-fundamentus = Fundamentus ({ $indicador }), read on { $data }
nucleo-score-secundaria-concordam = two secondary sources agree (average)
nucleo-score-secundaria-divergem = secondary sources differ ({ $fonte1 }: { $valor1 }; { $fonte2 }: { $valor2 }); the first one is used
nucleo-score-secundaria-uma = one secondary source
nucleo-score-secundaria-marca = secondary source
nucleo-score-secundaria-nota-motivo = official source without the data ({ $motivo }); { $nota }
nucleo-score-secundaria-nota = official source without the data; { $nota }
nucleo-score-secundaria-periodo = reported by the secondary source

## Qualitative events

nucleo-score-erro-evento-link = the evidence must be an http(s) link
nucleo-score-erro-evento-tipo = unknown event type: { $tipo }
nucleo-score-erro-evento-titulo = describe the event (up to 300 characters)
nucleo-score-erro-evento-ativo = enter the asset (asset_id or ticker)
nucleo-score-erro-evento-limites = cap and penalty go from 0 to 100
nucleo-score-ref-evento = event
nucleo-score-flag-evento = Event: { $tipo }

## Notices for the developer

nucleo-score-feedback-titulo = Asset score: critical data missing for { $ativo }
nucleo-score-feedback-resolvido-intrinseco = Resolved on its own on { $data }: not a collection gap ({ $detalhe }).
nucleo-score-feedback-resolvido-calculado = Resolved on its own on { $data }: the data arrived and the score was calculated ({ $detalhe }).
nucleo-score-feedback-resolvido = Resolved on its own on { $data }: { $detalhe }
nucleo-score-feedback-resolvido-encerrado = Resolved automatically on { $data }: the position was closed, the asset is no longer in the portfolio.
nucleo-score-feedback-corpo = The health score of { $ativo } ({ $classe }, profile { $perfil }) was not calculated on { $data } because critical data is missing:
    { $itens }

    Methodology { $versao }. See the asset in Assets › { $ativo } › Health.
nucleo-score-feedback-lacuna-conhecida = Resolved on its own: a known gap in the methodology (no free structured source), documented in docs/asset-score.md. The score renormalizes the dimension.

## FIIs

nucleo-score-periodo-12-meses = 12 months
nucleo-score-periodo-ultimos-pregoes = latest sessions
nucleo-score-fii-periodo-trimestral = quarterly report to { $data }
nucleo-score-fii-periodo-mensal = monthly report for { $mes }
nucleo-score-fii-sem-trimestral = no CVM quarterly report
nucleo-score-fii-periodo-4-trimestres = 4 quarters to { $data }
nucleo-score-fii-fonte-trimestrais = CVM quarterly reports
nucleo-score-fii-sem-rendimentos-declarados = no distributions declared in 4 quarters
nucleo-score-fii-poucos-trimestrais = fewer than 4 quarterly reports with results and distributions
nucleo-score-fii-sem-passivo = no liabilities or net assets in the monthly report
nucleo-score-fii-sem-obrigacoes = no acquisition obligations in the monthly report
nucleo-score-fii-periodo-pvp = price ÷ NAV from the latest report
nucleo-score-fii-sem-pvp = no price or net asset value
nucleo-score-fii-sem-dy = no distributions or price
nucleo-score-fii-fonte-dy-cdi = Fund DY; CDI (Central Bank, SGS 12)
nucleo-score-fii-sem-dy-fundo = no dividend yield
nucleo-score-fii-sem-cdi = no CDI for the last 12 months
nucleo-score-fii-periodo-taxa = average of { $n } monthly reports, annualized
nucleo-score-fii-fonte-taxa = CVM monthly report (Percentual_Despesas_Taxa_Administracao)
nucleo-score-fii-poucas-taxas = fewer than 3 monthly reports with the management fee
nucleo-score-fii-ipca-descontado = IPCA deducted
nucleo-score-fii-sem-ipca = no IPCA for the period: nominal change
nucleo-score-fii-fonte-vp-cota = CVM monthly report (net asset value per unit)
nucleo-score-fii-sem-vp-anterior = no net asset value per unit from 12 months earlier
nucleo-score-fii-sem-cotistas = no number of unitholders
nucleo-score-fii-flag-vacancia = financial vacancy of { $pct }
nucleo-score-fii-flag-inquilino = largest tenant with { $pct } of revenue
nucleo-score-fii-dado-informes = FII reports (CVM)
nucleo-score-fii-sem-informes = no CVM reports downloaded for this fund (run the asset analysis collection)
nucleo-score-fii-nota-composicao = Asset composition in the latest monthly report: CRI { $cri }, FII units { $fii }, properties { $imoveis }.

## Fixed income

nucleo-score-rf-flag-vencido = Matured on { $data }
nucleo-score-rf-titulo-vencido = Matured bond
nucleo-score-rf-titulo-vencido-sentido = Matured bond ({ $data }): there is nothing to rate. If it still shows a position in the Portfolio, record the redemption.
nucleo-score-rf-periodo-taxa = rate on { $data }
nucleo-score-rf-nota-ipca = 12-month IPCA to { $mes }: { $pct }%
nucleo-score-rf-nota-selic = Selic target { $selic }% + premium { $agio }%; { $ipca }
nucleo-score-rf-periodo-hoje = today
nucleo-score-rf-periodo-meses = { $n } months
nucleo-score-rf-fonte-bcb-selic-ipca = Central Bank (SGS 432 and 433)
nucleo-score-rf-nota-juro-real-selic = Selic real rate of { $pct }% against { $meses } month-ends over { $anos } years
nucleo-score-rf-familia-ipca = IPCA+ (including Renda+ and Educa+)
nucleo-score-rf-familia-prefixados = fixed-rate bonds
nucleo-score-rf-nota-percentil-taxa = rate of { $pct }% against { $medianas } daily medians of { $familia } over { $anos } years (median { $mediana }%)
nucleo-score-rf-historico-curto = rate history of { $familia } with fewer than { $dias } days stored
nucleo-score-rf-nota-prefixado-real = fixed rate { $pct }% minus the { $ipca }
nucleo-score-rf-nota-cupom = semiannual coupon of { $pct }% p.a.; matures on { $data }
nucleo-score-rf-nota-parcelas = { $n } monthly installments to { $data }
nucleo-score-rf-nota-sem-cupom = no coupon; matures on { $data }
nucleo-score-rf-periodo-pregoes = { $n } sessions
nucleo-score-rf-nota-custodia-selic = { $pct }% p.a. on the amount above { $isento } (position of { $posicao })
nucleo-score-rf-nota-custodia = { $pct }% p.a. on the balance
nucleo-score-rf-nota-pu = Unit price { $compra } to buy and { $venda } to sell
nucleo-score-rf-sem-termos-liquidez = no bond terms (liquidity and maturity)
nucleo-score-rf-fonte-cadastro = bond record
nucleo-score-rf-sem-vencimento = no maturity date in the bond record
nucleo-score-rf-nota-pos-fixado = floating rate: the rate follows CDI/Selic, almost no interest rate sensitivity
nucleo-score-rf-nota-aproximacao = approximation: time to maturity (without the interest and amortization schedule)
nucleo-score-rf-fonte-premissa = methodology assumption (ASSUMPTIONS.md › Fixed income)
nucleo-score-rf-nota-tesouro-reais = Brazilian National Treasury in reais
nucleo-score-rf-sem-taxas = no Tesouro Transparente rates stored for this bond (run the quote collection with Tesouro Direto turned on)
nucleo-score-rf-historico-selic-curto = Selic target and IPCA history too short (run the index collection)
nucleo-score-rf-sem-selic-ipca = no Selic target or 12-month IPCA (run the index collection)
nucleo-score-rf-sem-ipca = no 12-month IPCA (run the index collection)
nucleo-score-rf-nota-selic-duration = floating rate: the price follows Selic, almost no interest rate sensitivity
nucleo-score-rf-vencimento-desconhecido = unknown maturity: enter the maturity date or the year in the bond code (e.g. Tesouro IPCA+ 2035)
nucleo-score-rf-fonte-pu = Tesouro Transparente unit price stored for the asset
nucleo-score-rf-poucos-precos = fewer than 20 bond prices in the last 12 months (run the quote collection)
nucleo-score-rf-fonte-regra-tesouro = Tesouro Direto rule
nucleo-score-rf-nota-recompra = daily buyback by the Treasury
nucleo-score-rf-fonte-regra-b3 = B3 rule
nucleo-score-rf-fora-de-venda = bond not for sale on Tesouro Direto (no purchase price)
nucleo-score-rf-nota-tipo-desconhecido = Treasury bond type not recognized in the code or the name (e.g. Tesouro IPCA+ 2035).
nucleo-score-rf-nota-soberano = Sovereign credit risk: fixed score by assumption (Brazilian National Treasury in reais; no free rating feed).
nucleo-score-rf-dado-termos = Bond terms
nucleo-score-rf-sem-termos = the bond has no terms recorded (type, index, rate, maturity)
nucleo-score-rf-sem-roe-ifdata = no 12-month net income and equity in IF.data
nucleo-score-rf-nota-emissor-banco-cnpj = Issuer: { $emissor } (Central Bank IF.data, by the CNPJ in the bond record).
nucleo-score-rf-nota-emissor-banco-nome = Issuer: { $emissor } (Central Bank IF.data), identified by the name "{ $nome }"; check it and enter the CNPJ in the record so it doesn't depend on the name.
nucleo-score-rf-flag-inativa = { $emissor } is not listed as active in the latest IF.data registry
nucleo-score-rf-fora-de-funcionamento = { $emissor } is no longer operating according to the Central Bank registry: the indicators are no longer published
nucleo-score-rf-banco-nao-identificado = issuer "{ $nome }" not identified in the Central Bank registry (IF.data): enter the issuing bank's CNPJ when editing the bond in Assets
nucleo-score-rf-banco-nao-informado = issuer not entered: fill in the issuer and the bank's CNPJ in the bond record
nucleo-score-rf-fgc-tipo-outro = bond type "other": there's no way to know whether it has FGC coverage (LF doesn't; CDB, LC, LCI, LCA and savings do)
nucleo-score-rf-fonte-regra-fgc = FGC rule
nucleo-score-rf-sem-fgc = bond without FGC guarantee
nucleo-score-rf-flag-fgc = { $excesso } above the { $limite } limit per CPF at this institution
nucleo-score-rf-titular-desconhecido = holder not entered
nucleo-score-rf-fgc-sem-titulares = accounts without registered holders count as a single CPF (exposure of the whole family)
nucleo-score-rf-fgc-por-titular = exposure per account holder (joint account split among the holders)
nucleo-score-rf-sem-posicao = no position: within the limit
nucleo-score-rf-nota-fgc = { $titulos } guaranteed bond(s) of the family at this institution; largest exposure per CPF { $exposicao }; holders: { $titulares }; { $atribuicao }
nucleo-score-rf-fonte-fgc = portfolio positions and the FGC rule
nucleo-score-rf-fonte-regra-poupanca = savings account rule
nucleo-score-rf-nota-liquidez-diaria = daily liquidity
nucleo-score-rf-nota-heranca-banco = Issuer health inherited from the bank (§7.2): Central Bank IF.data prudential indicators against banks of the same size.
nucleo-score-rf-sem-cdi = no CDI stored (run the index collection)
nucleo-score-rf-fonte-bcb-poupanca = Central Bank (SGS 12, 432 and 226)
nucleo-score-rf-nota-poupanca = savings yield { $rende }% p.a. (Selic target { $selic }%, 12-month TR { $tr }%); tax-exempt, compared with income tax of { $ir }%
nucleo-score-rf-taxa-invalida = invalid rate in the bond record
nucleo-score-rf-pct-do-indice = { $pct }% of { $indice }
nucleo-score-rf-nota-prefixado-curva = fixed rate { $pct }% against a Tesouro Prefixado of similar term ({ $curva }%)
nucleo-score-rf-fonte-bcb-cdi = Central Bank (SGS 12)
nucleo-score-rf-nota-prefixado-cdi = fixed rate { $pct }% against today's CDI ({ $cdi }%; no Treasury curve)
nucleo-score-rf-sem-curva-cdi = no Treasury curve or CDI (run the quote and index collections)
nucleo-score-rf-nota-ipca-curva = inflation + { $pct }% against a Tesouro IPCA+ of similar term ({ $real }%) and fixed rate ({ $pre }%)
nucleo-score-rf-fonte-bcb-cdi-ipca = Central Bank (SGS 12 and 433)
nucleo-score-rf-nota-ipca-cdi = inflation + { $pct }% with 12-month IPCA ({ $ipca }%) against today's CDI ({ $cdi }%; no Treasury curve)
nucleo-score-rf-sem-curva-cdi-ipca = no Treasury curve, CDI or IPCA (run the quote and index collections)
nucleo-score-rf-nota-isento = ; tax-exempt, equivalent to { $pct }% of a taxed CDI (income tax of { $ir }% for { $dias } days)
nucleo-score-rf-via-cnpj = by the CNPJ in the bond record
nucleo-score-rf-via-nome = by the name "{ $nome }"; check it and enter the CNPJ in the record
nucleo-score-rf-nota-emissor-cvm = Issuer: { $emissor } (CVM), { $como }.
nucleo-score-rf-periodo-dados-ate = data to { $data }
nucleo-score-rf-fonte-score-acoes = issuer's stock score
nucleo-score-rf-nota-score-emissor = { $emissor }: score { $score } ({ $faixa }), health { $saude }{ $parcial ->
    [sim] ; partial score (too much imputed weight)
   *[nao] {""}
}
nucleo-score-rf-score-emissor-falhou = the stock score of { $emissor } couldn't be calculated: { $faltam }
nucleo-score-rf-companhia-nao-identificada = issuer "{ $nome }" not identified among CVM listed companies: enter the company's CNPJ in the bond record (and run the asset analysis collection)
nucleo-score-rf-companhia-nao-informada = issuer not entered: fill in the issuer and the company's CNPJ in the bond record
nucleo-score-rf-nota-spread-ipca = inflation + { $pct }% against a Tesouro IPCA+ of similar term ({ $curva }%)
nucleo-score-rf-sem-curva-ipca = no Tesouro IPCA+ curve (run the quote collection)
nucleo-score-rf-sem-curva-prefixado = no Tesouro Prefixado curve (run the quote collection)
nucleo-score-rf-nota-pct-cdi = { $pct }% of CDI with CDI at { $cdi }% p.a.
nucleo-score-rf-isento = income tax exempt for individuals
nucleo-score-rf-tributado = taxed by the regressive table
nucleo-score-rf-nota-credito-privado = Collateral, covenants, subordination and rating have no free structured source: the Structure dimension is imputed (§7.5). Spread calculated with the contracted rate against the Treasury curve; ANBIMA indicative rates are not open data.

## Classic models

nucleo-score-piotroski-roa = Positive ROA
nucleo-score-piotroski-fco = Positive operating cash flow
nucleo-score-piotroski-roa-maior = ROA higher than the previous year
nucleo-score-piotroski-accrual = Operating cash ÷ assets higher than ROA (accrual quality)
nucleo-score-piotroski-alavancagem = Lower long-term leverage
nucleo-score-piotroski-liquidez = Higher current ratio
nucleo-score-piotroski-sem-emissao = Did not issue shares
nucleo-score-piotroski-margem = Higher gross margin
nucleo-score-piotroski-giro = Higher asset turnover
nucleo-score-leitura-forte = strong
nucleo-score-leitura-fraco = weak
nucleo-score-leitura-intermediario = intermediate
nucleo-score-piotroski-nota = { $n } of 9 criteria computable with the available data
nucleo-score-leitura-zona-segura = safe zone
nucleo-score-leitura-zona-cinza = gray zone
nucleo-score-leitura-zona-estresse = distress zone
nucleo-score-altman-x1 = X1 working capital ÷ assets
nucleo-score-altman-x2 = X2 retained earnings ÷ assets
nucleo-score-altman-x3 = X3 EBIT ÷ assets
nucleo-score-altman-x4 = X4 book equity ÷ liabilities
nucleo-score-altman-referencia = Altman (1968); Altman, Hartzell and Peck (1995)
nucleo-score-beneish-dsri = DSRI receivables/revenue
nucleo-score-beneish-gmi = GMI gross margin
nucleo-score-beneish-aqi = AQI asset quality
nucleo-score-beneish-sgi = SGI revenue growth
nucleo-score-beneish-depi = DEPI depreciation rate
nucleo-score-beneish-sgai = SGAI selling and administrative expenses
nucleo-score-beneish-tata = TATA total accruals
nucleo-score-beneish-lvgi = LVGI leverage
nucleo-score-leitura-manipulacao = likely manipulation (review)
nucleo-score-leitura-sem-manipulacao = no sign of manipulation
nucleo-score-beneish-nota = no data for { $variaveis } (counted as neutral)

## Feedback and segment

nucleo-score-erro-veredito = the verdict is "discordo" (disagree) or "concordo" (agree)
nucleo-score-erro-motivo = explain the reason (up to 2,000 characters)
nucleo-score-erro-ticker = enter the ticker
nucleo-score-erro-segmento = segment "{ $segmento }" doesn't exist for this class

## Funds

nucleo-score-fundo-doc-informe = CVM daily report to { $data }
nucleo-score-fundo-sem-volume = no traded volume (run the quote collection)
nucleo-score-fundo-fonte-convertido = { $fonte }; converted from { $moeda } at the rate of { $data }
nucleo-score-fundo-sem-cambio = no { $moeda }/BRL exchange rate to convert the volume
nucleo-score-fundo-fonte-cotas = share values from the CVM daily report
nucleo-score-fundo-poucas-cotas = less than 12 months of share values for { $para }
nucleo-score-fundo-para-indice = the index
nucleo-score-fundo-para-benchmark = the benchmark
nucleo-score-fundo-para-drawdown = the drawdown
nucleo-score-fundo-para-recuperacao = the recovery
nucleo-score-fundo-periodo-anos = { $de } to { $ate } ({ $anos } years)
nucleo-score-fundo-sem-cdi = no CDI for the period (run the index collection)
nucleo-score-fundo-nota-sharpe = { $valor } (return { $retorno } p.a. × CDI { $cdi } p.a.; volatility { $volatilidade })
nucleo-score-fundo-volatilidade-zero = zero volatility
nucleo-score-fundo-sortino-teto = no day below CDI: Sortino at the cap
nucleo-score-fundo-nota-benchmark = benchmark: { $benchmark }
nucleo-score-fundo-benchmark-cdi = the benchmark is CDI: the Sharpe ratio already measures the return over it
nucleo-score-fundo-nota-information-ratio = { $valor } (tracking error { $te })
nucleo-score-fundo-tracking-zero = zero tracking error
nucleo-score-fundo-sem-serie = no { $benchmark } series for the period
nucleo-score-fundo-nota-janelas = { $acima } of { $janelas } windows above { $benchmark }
nucleo-score-fundo-poucas-janelas = fewer than { $minimo } 12-month windows with share values and { $benchmark }
nucleo-score-fundo-sem-queda = no drop in the window
nucleo-score-fundo-recuperada = , recovered on { $data }
nucleo-score-fundo-nao-recuperada = , not recovered yet
nucleo-score-fundo-periodo-cotas = { $n } share values to { $data }
nucleo-score-fundo-poucas-cotas-12m = fewer than { $n } share values in the last 12 months
nucleo-score-fundo-periodo-extrato = statement of { $data }
nucleo-score-fundo-fonte-extrato = CVM: fund information statement
nucleo-score-fundo-sem-extrato = fund without an information statement at CVM
nucleo-score-fundo-nota-performance = on what exceeds { $benchmark }
nucleo-score-fundo-fonte-patrimonio = CVM: daily report (VL_PATRIM_LIQ)
nucleo-score-fundo-periodo-registro = registry of { $data }
nucleo-score-fundo-fonte-registro = CVM: registry of funds and classes
nucleo-score-fundo-sem-patrimonio = no net assets in the daily report or the CVM registry
nucleo-score-fundo-fonte-cotistas = CVM: daily report (NR_COTST)
nucleo-score-fundo-sem-cotistas = no number of unitholders in the daily report
nucleo-score-fundo-periodo-captacao = { $meses } months, { $de } to { $ate } ({ $documento })
nucleo-score-fundo-fonte-captacao = CVM: daily report (CAPTC_DIA − RESG_DIA)
nucleo-score-fundo-nota-captacao = net inflows of { $valor }
nucleo-score-fundo-pouca-captacao = fewer than 11 months of inflows in the daily report
nucleo-score-fundo-periodo-constituido = incepted on { $data }
nucleo-score-fundo-nota-tempo-gestao = proxy for management track record
nucleo-score-fundo-sem-constituicao = no inception date in the CVM registry
nucleo-score-fundo-retorno-meses = { $meses } months { $pct } p.a.{ $benchmark }
nucleo-score-fundo-nota-retorno = Share value return: { $retornos }.
nucleo-score-fundo-flag-patrimonio = net assets of { $valor }
nucleo-score-fundo-flag-resgates = net inflows of { $pct } of net assets in 12 months
nucleo-score-fundo-flag-queda = maximum drop of { $pct }
nucleo-score-fundo-cotacao-manual = manual quotes: record the share values on the asset or choose the CVM source
nucleo-score-fundo-sem-cnpj-provavel = fund without a CNPJ: by its name, it is probably { $cnpj } ({ $nome }, in the CVM registry); check it and enter the CNPJ on the asset to fetch the share values
nucleo-score-fundo-sem-cnpj = fund without a CNPJ: enter the CNPJ on the asset to fetch the share values from CVM
nucleo-score-fundo-cnpj-plano-provavel = CNPJ { $cnpj } is not in the CVM fund registry (for pensions it is usually the plan's); by name, the likely fund is { $provavel } ({ $nome }); check it and change the CNPJ on the asset to fetch the unit prices
nucleo-score-fundo-fonte-nao-cvm = the asset's quote source is not CVM: choose CVM (or leave it automatic) to fetch the share values by CNPJ
nucleo-score-fundo-cnpj-fora-do-registro = CNPJ { $cnpj } did not appear in the CVM fund registry downloaded on { $data }{ $tipo ->
    [previdencia] : for pension plans, enter the CNPJ of the fund (FIE) the plan uses, not the plan's own at Susep
   *[fundo] : check the CNPJ on the asset
}
nucleo-score-fundo-sem-cotas = no share values stored (run the quote collection)
nucleo-score-fundo-dado-cotas = Fund share values
nucleo-score-fundo-so-cadastrados = the fund score only applies to registered funds (with a CNPJ)
nucleo-score-fundo-dado-cotas-cvm = Fund share values (CVM)
nucleo-score-fundo-cadastro-classe = CVM class { $classe }
nucleo-score-fundo-cadastro-indicador = declared benchmark: { $indicador }
nucleo-score-fundo-cadastro-gestor = manager { $gestor }
nucleo-score-fundo-nota-cadastro = CVM registration: { $cadastro }.
nucleo-score-fundo-nota-exclusivo = Exclusive fund: few unitholders are expected.
nucleo-score-fundo-nota-sem-cadastro = Fund without a downloaded CVM registration: default segment and no fees (run the asset analysis collection).
nucleo-score-fundo-nota-benchmark-pares = Benchmark: { $benchmark }. Funds have no peer universe in the app: the scores use only the class's absolute references.

## ETFs

nucleo-score-etf-domicilio-br = ETF domiciled in Brazil
nucleo-score-etf-domicilio-us = ETF domiciled in the US: 30% withheld on dividends and exposure to US estate tax
nucleo-score-etf-domicilio-ucits = ETF listed in Europe (usually an Irish UCITS): 15% withheld on US dividends
nucleo-score-etf-domicilio-outro = ETF listed outside Brazil, the US and Europe
nucleo-score-etf-dado-cotacoes = ETF quotes
nucleo-score-etf-so-cadastrados = the ETF score only applies to registered ETFs
nucleo-score-etf-fonte-cotacoes = quotes ({ $fonte })
nucleo-score-etf-sem-indice = no reference index series in the app
nucleo-score-etf-indice-convertido = { $indice } in { $moeda } at the day's exchange rate
nucleo-score-etf-sem-serie = no { $indice } series (or exchange rate) for the period (run the index collection)
nucleo-score-etf-sem-dados = no data to compare
nucleo-score-etf-fonte-indice = { $fonte }; index { $indice }
nucleo-score-etf-nota-tracking = ETF { $etf } × index { $indice }
nucleo-score-etf-periodo-semanal = { $periodo }, weekly returns
nucleo-score-etf-nota-cambio = index converted at the day's exchange rate: includes the timing mismatch between the exchange rate and the closes
nucleo-score-etf-poucas-semanas = too few weeks for the tracking error
nucleo-score-etf-sem-cnpj = ETF without a CNPJ found in the CVM registry (enter the CNPJ on the asset and run the asset analysis collection)
nucleo-score-etf-sem-patrimonio = net assets of foreign ETFs have no free structured source
nucleo-score-etf-periodo-classe-bolsa = asset class and exchange
nucleo-score-etf-fonte-tabela = methodology table
nucleo-score-etf-domicilio-sem-nota = domicile without a score in the methodology
nucleo-score-etf-sem-constituicao = no inception date (only B3 ETFs with a CNPJ in the CVM registry)
nucleo-score-etf-periodo-carteira = theoretical portfolio { $indice } of { $data }
nucleo-score-etf-fonte-carteira = B3: index theoretical portfolio
nucleo-score-etf-fonte-componentes = health score of each component
nucleo-score-etf-nota-componentes = { $com_score } of { $total } components with a score ({ $peso } of the weight)
nucleo-score-etf-pouca-cobertura = only { $peso } of the portfolio weight has a score (minimum 50%; run the asset analysis collection)
nucleo-score-etf-sem-carteira = no theoretical portfolio for { $indice } (run the asset analysis collection)
nucleo-score-etf-sem-composicao = composition has no free structured source (only B3 indices)
nucleo-score-etf-periodo-fechamentos = { $n } closes to { $data }
nucleo-score-etf-poucos-fechamentos = fewer than { $n } closes in the last 12 months (run the quote collection)
nucleo-score-etf-nota-indice = Reference index: { $indice }.
nucleo-score-etf-nota-cnpj = CNPJ at CVM: { $cnpj }.
nucleo-score-etf-nota-sem-composicao = Without the ETF composition: component health and concentration give way to risk (volatility and drawdown).
nucleo-score-etf-nota-sem-pares = ETFs have no peer universe in the app: the scores use only absolute references.

## Funds and ETFs: collection

nucleo-score-erro-b3-carteira-invalida = invalid response from B3: { $erro }
nucleo-score-erro-b3-sem-carteira = B3 response without the portfolio
nucleo-score-erro-carteira-vazia = empty theoretical portfolio
nucleo-score-erro-registro-sem-classes = CVM registry without registro_classe.csv
nucleo-score-passo-yahoo-etf = Yahoo: name of ETF { $ticker }
nucleo-score-passo-carteira = B3: theoretical portfolio of { $indice }

## Foreign stocks and BDRs

nucleo-score-intl-sec-desligada = { $falta }: the SEC EDGAR source is turned off in Settings › Data sources
nucleo-score-intl-sec-email = { $falta }: the SEC only serves those who provide a contact email; enter yours in the asset's Health tab (or in Settings › Data sources › Analysis settings) and the SEC collection runs right away
nucleo-score-intl-sec-coleta = { $falta } (the daily asset analysis collection hasn't brought it yet; the result is in Assets › Analysis data)
nucleo-score-intl-dado-demonstracoes = Financial statements
nucleo-score-intl-sem-fonte = no free source of financial statements for exchanges outside Brazil and the US (quotes only)
nucleo-score-intl-sem-sec = no SEC financial statements downloaded for this asset
nucleo-score-intl-dado-sec = Financial statements (SEC EDGAR)
nucleo-score-intl-dado-historico = Balance sheet history
nucleo-score-intl-listada-ha-pouco = recently listed company: no annual report (10-K) at the SEC yet; the score comes after the first full fiscal year
nucleo-score-intl-nota-sec = SEC financial statements (US GAAP) in US dollars; real growth deflated by US CPI. No governance dimension (no free structured source).
nucleo-score-intl-nota-financeira = US bank or insurer: no prudential indicators (Basel, defaults); score by ROE, valuation and stability.
nucleo-score-intl-dado-subjacente = BDR underlying asset
nucleo-score-intl-bdr-sem-empresa = the BDR's company couldn't be identified
nucleo-score-intl-bdr-sem-sec = no SEC financial statements for { $ticker }, the BDR's likely underlying
nucleo-score-intl-dado-subjacente-sec = Underlying financial statements (SEC EDGAR)
nucleo-score-intl-nota-bdr = BDR: inherits the score of { $empresa } ({ $ticker }), the underlying.

## Financial statement metrics

nucleo-score-metrica-fonte-dfp = CVM DFP
nucleo-score-metrica-fonte-fca = CVM FCA (securities)
nucleo-score-metrica-nota-ebitda-medio = Average EBITDA of the latest fiscal years (cyclical normalization)
nucleo-score-metrica-ebitda-medio-negativo = Average EBITDA ≤ 0
nucleo-score-metrica-capital-negativo = invested capital (net debt + equity) ≤ 0
nucleo-score-metrica-divisao-zero = division by zero
nucleo-score-metrica-nota-divida-ebitda = current debt ÷ average EBITDA
nucleo-score-metrica-sem-altman = missing working capital, retained earnings, EBIT or liabilities
nucleo-score-metrica-sem-ciclo = missing receivables, payables, revenue or COGS
nucleo-score-metrica-lucro-anterior-negativo = previous fiscal year's net income ≤ 0
nucleo-score-metrica-lucro-negativo-razao = net income ≤ 0 (the ratio doesn't apply)
nucleo-score-metrica-lpa-final-negativo = ending EPS ≤ 0
nucleo-score-metrica-lpa-inicial-negativo = starting EPS ≤ 0
nucleo-score-metrica-lucro-negativo-sem-proventos = net income ≤ 0 and no dividends
nucleo-score-metrica-poucos-fechamentos = fewer than 120 closes in the last year
nucleo-score-metrica-poucos-lucros-2 = fewer than 2 fiscal years with net income
nucleo-score-metrica-poucos-ebitda = fewer than 3 fiscal years with EBITDA
nucleo-score-metrica-poucos-roic = fewer than 3 fiscal years with ROIC
nucleo-score-metrica-poucos-lucros-3 = fewer than 3 fiscal years with net income
nucleo-score-metrica-poucos-roe = fewer than 4 fiscal years with ROE
nucleo-score-metrica-poucos-lpa = fewer than 6 fiscal years with net income and shares
nucleo-score-metrica-periodo-media-exercicios = average of the latest fiscal years
nucleo-score-metrica-sem-dividendos-12m = paid no dividends in the last 12 months
nucleo-score-metrica-proventos-com-prejuizo = paid dividends with net income ≤ 0
nucleo-score-metrica-patrimonio-negativo = equity ≤ 0
nucleo-score-metrica-sem-piotroski = needs 2 fiscal years with the Piotroski items
nucleo-score-metrica-receita-negativa = starting or ending revenue ≤ 0
nucleo-score-metrica-nota-receitas-financeiras = financial income exceeds financial expenses
nucleo-score-metrica-sem-segmento-listagem = listing segment not reported in the FCA
nucleo-score-metrica-sem-ebit = no EBIT
nucleo-score-metrica-sem-ebit-12m = no 12-month EBIT
nucleo-score-metrica-sem-ebit-despesas = no EBIT or financial expenses
nucleo-score-metrica-sem-ebitda = no EBITDA
nucleo-score-metrica-sem-ipca = no IPCA for the period: nominal growth
nucleo-score-metrica-sem-circulante = no current assets or liabilities
nucleo-score-metrica-sem-ativo-anterior = no total assets from 12 months earlier
nucleo-score-metrica-sem-caixa-divida-cp = no cash or short-term debt
nucleo-score-metrica-sem-cotacao-acoes = no price or share count
nucleo-score-metrica-sem-cotacoes-12m = no 12-month quotes
nucleo-score-metrica-sem-regra-40 = no revenue growth or free cash flow margin
nucleo-score-metrica-sem-entregas = no filing dates for the financial statements
nucleo-score-metrica-sem-dividendos = no dividends declared or paid
nucleo-score-metrica-sem-dividendos-pagos = no dividends paid
nucleo-score-metrica-sem-divida-cp = no short-term debt
nucleo-score-metrica-sem-divida-caixa = no debt and cash on the balance sheet
nucleo-score-metrica-sem-divida-liquida = no net debt
nucleo-score-metrica-sem-fcl = no free cash flow
nucleo-score-metrica-sem-fco-lucro = no operating cash flow or net income
nucleo-score-metrica-sem-fco-receita = no operating cash flow or revenue
nucleo-score-metrica-sem-lucro = no net income
nucleo-score-metrica-sem-lucro-bruto-ativo = no gross profit (or revenue and COGS) or total assets
nucleo-score-metrica-sem-lucro-12m = no 12-month net income
nucleo-score-metrica-sem-lucro-ativo = no net income or total assets
nucleo-score-metrica-sem-lucro-fco-ativo = no net income, operating cash flow or assets
nucleo-score-metrica-sem-acoes-anterior = no share count from 12 months earlier
nucleo-score-metrica-sem-patrimonio = no equity
nucleo-score-metrica-sem-patrimonio-ativo = no equity or assets
nucleo-score-metrica-sem-proventos-cotacao = no dividends or price
nucleo-score-metrica-sem-receita-ativo = no revenue or assets
nucleo-score-metrica-sem-receita-positiva = no positive revenue
nucleo-score-metrica-sem-retornos = no returns
nucleo-score-metrica-ev-negativo = enterprise value ≤ 0 (cash above market cap)
nucleo-score-metrica-periodo-ultimo-balanco = latest balance sheet
nucleo-score-metrica-sem-lucro-liquido = no net income
nucleo-score-metrica-media-exercicios = average of { $n } fiscal years
nucleo-score-metrica-periodo-exercicio = fiscal year to { $data }
nucleo-score-metrica-poucos-exercicios-receita = fewer than { $n } fiscal years with revenue
nucleo-score-metrica-nota-nominal = nominal { $pct }% p.a., IPCA deducted
nucleo-score-metrica-periodo-n-exercicios = { $n } fiscal years
nucleo-score-metrica-periodo-entregas = last { $n } filings
nucleo-score-metrica-fonte-recebimento = CVM (receipt date)
nucleo-score-metrica-periodo-pregoes = { $n } sessions to { $data }
nucleo-score-metrica-fonte-fechamentos = daily closes

## Crypto: documents

nucleo-score-cripto-doc-market-chart = CoinGecko market_chart (365 days)
nucleo-score-fundo-dado-cotas-extrato = Plan units (statement)
nucleo-score-fundo-efpc-sem-cotas = a pension fund plan (EFPC, supervised by Previc) is not a CVM fund: the plan's unit price only comes from the participant's statement; import the PDF statement on the asset to record the unit price month by month
nucleo-score-fii-trimestral-sem-item = the quarterly report to { $data } does not include it (field blank or zero)
nucleo-score-fii-nota-por-area = weighted by area: the quarterly report does not give each property's share of revenue
nucleo-score-confianca-dy-estimado = DY estimated from the CVM reports (no distributions paid on the B3)
