# Screen 08 · Portfolio: tabs, summary, positions, returns, allocation and rebalancing, trades,
# corporate actions, dividends, passive income, fee cost and risk, plus the asset class,
# dividend and data source labels also used on other screens (docs/i18n-design.md). The B3 and
# brokerage note imports are left for phase 5.
# Revision: AI draft, not yet reviewed by a native speaker.

## Tabs and header

carteira-secoes = Portfolio sections
carteira-aba-posicoes = Positions
carteira-aba-rentabilidade = Returns
carteira-aba-alocacao = Allocation and rebalancing
carteira-aba-operacoes = Trades
carteira-aba-proventos = Dividends
carteira-aba-renda = Passive income
carteira-aba-taxas = Fee cost
carteira-aba-risco = Risk
carteira-aba-saude = Asset health
carteira-aba-bens = Physical assets
carteira-carregando = Calculating the portfolio
carteira-meta = Position on { $data } · real by { $deflator }
carteira-evento-corporativo = Corporate action
carteira-registrar-proventos = Record dividends
carteira-provento-detalhado = Detailed dividend
carteira-nova-operacao = New trade
carteira-importar-b3 = Import from B3
carteira-importar-notas = Import notes
carteira-real-descontado-por = Real result discounted by
carteira-eventos-corporativos = Corporate actions

## Empty portfolio

carteira-vazia-kicker = Empty portfolio
carteira-vazia-titulo = Record your first trade
carteira-vazia-texto = Add the asset in 09 · Assets (or pick one you already added) and record the purchase with its costs. The portfolio works out the average price, result, XIRR and the real return net of inflation.
carteira-cadastrar-ativo = Add asset

## Assets valued at cost

carteira-avaliados-pelo-custo =
    { $n ->
        [one] 1 asset valued
       *[other] { $n } assets valued
    } at cost: { $lista }
carteira-lista-e-mais = { $lista } and { $n } more
carteira-avaliados-texto = Without a quote, the market value is the cost and the unrealized result stays at zero. Update quotes in { $link } (Yahoo Finance and Tesouro Transparente) or enter the quote by hand on the asset's page. Funds (CVM unit prices) are coming in a later step.
carteira-link-indices = 09 · Assets › Indices and quotes

## Command bar

carteira-cmd-evento-busca = split,reverse split,bonus shares,stock split,merger
carteira-cmd-ver-aba = Show { $aba }
carteira-cmd-ver-aba-busca = tab,show
carteira-cmd-operacao = New trade (buy or sell)
carteira-cmd-operacao-busca = record,buy,sell,trade,brokerage note,investment
carteira-cmd-provento = Record dividend
carteira-cmd-provento-busca = dividends,jcp,income,fii,coupon
carteira-cmd-proventos = Record several dividends (date, ticker and amount)
carteira-cmd-proventos-busca = dividends,jcp,income,grid,batch,paste
carteira-cmd-importar-b3 = Import from B3 (Investor Area)
carteira-cmd-importar-b3-busca = b3,investor area,trading,movements,position,excel
carteira-cmd-importar-notas = Import brokerage notes (PDF)
carteira-cmd-importar-notas-busca = brokerage note,sinacor,pdf,fees,charges,irrf
carteira-cmd-ativo-busca = new asset,stock,fii,etf,tesouro,cdb,fixed income,crypto
carteira-cmd-cotacoes = Update indices, exchange rates and quotes
carteira-cmd-cotacoes-busca = ipca,cdi,ptax,dollar,sync,yahoo,fetch
carteira-cmd-agenda = Dividend calendar
carteira-cmd-agenda-busca = record date,payment,announced dividends
carteira-cmd-watchlist = Watchlist
carteira-cmd-watchlist-busca = favorites,follow,target price,alerts
carteira-cmd-rankings = Asset rankings
carteira-cmd-rankings-busca = dividend yield,p/e,graham,bazin,top,ranking
carteira-cmd-screener = Screener (asset filter)
carteira-cmd-screener-busca = filter,screens,screener,indicators
carteira-cmd-comparador = Compare assets
carteira-cmd-comparador-busca = comparison,side by side,base 100
carteira-cmd-analise = Fetch analysis data (CVM, B3, SEC)
carteira-cmd-analise-busca = fundamentals,cvm,sec,dfp,fetch,sync
carteira-cmd-grupo-ativos = Investments · assets
carteira-cmd-abrir-ativo-busca = open asset,quote,chart

## Common states

carteira-falha-desconhecida = Unknown failure.
carteira-erro-carregar = Couldn't load
carteira-excluir = Delete
carteira-total = Total
carteira-total-valor = Total: { $valor }
carteira-opcional = Optional.
carteira-valor-invalido = Invalid amount.
carteira-escolha-ativo = Choose the asset
carteira-escolha-conta = Choose the account
carteira-nenhuma-corretora = No brokerage account yet
carteira-n-meses = { $n } months
carteira-no-ano = Year to date
carteira-periodo = Period
carteira-acoes-de = Actions for { $nome }
carteira-e = and
carteira-pp = pp
carteira-valor-pp = { $valor } pp
carteira-pct-aa = { $pct } p.a.
carteira-data-formato = mm/dd/yyyy

## Asset classes (the same names as the core)

carteira-classe-stock-br = Stocks
carteira-classe-fii = FIIs
carteira-classe-etf-br = ETFs
carteira-classe-bdr = BDRs
carteira-classe-fixed-income = Fixed income
carteira-classe-treasury-br = Tesouro Direto
carteira-classe-fund = Funds
carteira-classe-stock-us = US stocks
carteira-classe-etf-us = US ETFs
carteira-classe-reit = REITs
carteira-classe-crypto = Crypto
carteira-classe-pension = Pension plans
carteira-classe-other = Other
carteira-classe-stock-intl = Stocks on other exchanges
carteira-classe-etf-intl = ETFs on other exchanges

## Dividend and corporate action types

carteira-provento-dividend = Dividends
carteira-provento-fii-income = FII income
carteira-provento-coupon = Coupon
carteira-provento-interest = Interest
carteira-provento-rent = Stock lending
carteira-provento-amortization = Amortization
carteira-provento-other = Other payouts

carteira-evento-split = Split
carteira-evento-reverse-split = Reverse split
carteira-evento-bonus = Bonus shares
carteira-evento-subscription = Subscription
carteira-evento-merger = Merger
carteira-evento-ticker-change = Ticker change
carteira-evento-spin-off = Spin-off
carteira-evento-capital-return = Return of capital

## Real result deflator

carteira-deflator-rotulo = Real result deflator
carteira-deflator-ipca-dica = Brazil's official inflation (IBGE, via BCB/SGS 433)
carteira-deflator-pais = Country inflation
carteira-deflator-pais-de = Country inflation ({ $pais })
carteira-deflator-pais-dica = Official inflation of the reference country (Settings › Data sources)
carteira-deflator-pais-serie = { $pais }: { $serie } ({ $fonte }). Change the country in Settings › Data sources.
carteira-deflator-us-cpi-dica = US consumer inflation (FRED CPIAUCSL)
carteira-deflator-m2-br-dica = Brazil's money supply growth (BCB/SGS 27810)
carteira-deflator-m2-us = M2 US
carteira-deflator-m2-us-dica = US money supply growth (FRED M2SL)
carteira-deflator-nominal = Nominal
carteira-deflator-nominal-dica = Without discounting inflation
carteira-fonte-oficial = official source

## Fixed income

carteira-rf-debenture = Debenture
carteira-rf-debenture-incentivada = Incentivized debenture
carteira-rf-poupanca = Savings account
carteira-rf-outro = Other
carteira-indexador-prefixado = Fixed rate
carteira-liquidez-diaria = Daily
carteira-liquidez-vencimento = At maturity
carteira-liquidez-carencia = After the lock-up
carteira-taxa-dica-percentual = % of { $indice } (e.g., 110 for 110% of { $indice })
carteira-taxa-dica-prefixado = Annual rate (e.g., 12.5 for 12.5% p.a.)
carteira-taxa-dica-real = Real annual rate above { $indice } (e.g., 6.2 for { $indice } + 6.2% p.a.)
carteira-termos-percentual = { $taxa }% of { $indice }
carteira-termos-pre = Fixed { $taxa }% p.a.
carteira-termos-indice = { $indice } + { $taxa }% p.a.
carteira-termos-vencimento = matures { $data }

## Valuation and sources

carteira-avaliacao-mercado = Market quote
carteira-avaliacao-curva = Marked to the curve
carteira-avaliacao-sem-cotacao = No quote (at cost)
carteira-fonte-b3-listados = B3 (official dividends)
carteira-fonte-yahoo-intraday = Yahoo Finance (intraday, ~15 min delay)
carteira-fonte-manual-voce = Entered by you
carteira-fonte-mock = Sample data
carteira-fonte-radar = Dividend radar (projection)
carteira-fonte-bcb = Central Bank of Brazil
carteira-fonte-oecd = OECD
carteira-fonte-worldbank = World Bank (annual, interpolated)
carteira-fonte-bce = ECB
carteira-fonte-bce-ref = ECB (reference)
carteira-fonte-nao-informada = Source not given
carteira-fonte-manual = Manual
carteira-fonte-sem-cotacao = No quote
carteira-fonte-na-curva = On the curve
carteira-fonte-manual-sem-fonte = Manual (no automatic source)
carteira-fonte-manual-bolsa = Manual (enter the exchange)

## Quote and nominal × real meter

carteira-cotacao-intradia = Quote from { $quando }, about { $min } min delayed; { $sessao }. The history uses daily closes.
carteira-cotacao-defasada = Stale quote: last known close before the position date.
carteira-cotacao-ultimo-pregao = Last trading session closed before the position date.
carteira-cotacao-fechamento = Session close.
carteira-na-curva = on the curve
carteira-curva-fonte = Marked to the curve (CDI/IPCA from the BCB)
carteira-curva-nota = Fixed income valued at the contracted rate over the index, on a 252-business-day basis.
carteira-sem-cotacao = no quote
carteira-sem-cotacao-dica = No quote: the value is the cost. Update quotes or enter the quote by hand on the asset's page.
carteira-medidor = Nominal { $nominal }; real ({ $deflator }) { $real }
carteira-medidor-oculto = Nominal × real ({ $deflator }): values hidden

## Summary (top instruments)

carteira-resumo-fonte-sem-cotacao = Quotes, curve and cost ({ $n } without a quote)
carteira-resumo-fonte = Closing quotes and marking to the curve
carteira-resumo-estimado = This figure includes estimated exchange rates, indices or quotes (or assets valued at cost).
carteira-valor-mercado = Market value
carteira-custo = Cost
carteira-em = On
carteira-patrimonio-investido = Invested assets
carteira-nao-realizado-pct = unrealized { $pct }
carteira-resultado = Result
carteira-nao-realizado = Unrealized
carteira-realizado = Realized
carteira-proventos = Dividends
carteira-resultado-total = Total result
carteira-resultado-perspectiva = Result perspective
carteira-resultado-nominal = Nominal
carteira-resultado-real-de = Real result ({ $indice })
carteira-resultado-efeito = { $indice } effect
carteira-resultado-sem-indice = No { $indice } for the period: the index still needs to be collected
carteira-sobre-custo = on cost · { $pct }
carteira-xirr-titulo = Annual return (XIRR)
carteira-xirr-sem-dados-ipca = No { $deflator } data up to { $data }: update the indices in 09 · Assets › Indices and quotes.
carteira-xirr-sem-dados = No { $deflator } data up to { $data }: the index comes out late and the core doesn't estimate the missing months yet.
carteira-deflator = Deflator
carteira-metodo = Method
carteira-metodo-xirr = XIRR (dates and amounts of each cash flow)
carteira-nominal = Nominal
carteira-real-de = Real · { $deflator }
carteira-renda-12m = Passive income 12m
carteira-media-mensal = Monthly average
carteira-yield = Yield
carteira-proventos-liquidos = Net dividends
carteira-recebidos-moeda = received, in { $moeda }
carteira-carregando-curto = loading

## Positions

carteira-avisos = Warnings: { $lista }
carteira-liquido-dica = Estimated net value if redeemed today: regressive income tax (22.5% up to 180 days → 15% above 720) on the gain and IOF in the first 30 days; exempt ones (LCI, LCA, CRI, CRA, incentivized debentures) don't pay. An estimate, not the tax calculation.
carteira-liquido-abrev = net { $valor }
carteira-col-ativo = Asset
carteira-col-classe = Class
carteira-col-qtd = Qty.
carteira-col-preco-medio = Average price
carteira-col-cotacao = Quote
carteira-col-valor = Value
carteira-col-peso = Weight
carteira-col-resultado = Result
carteira-col-xirr = XIRR p.a.
carteira-col-nominal-real = Nom. × real
carteira-n-ativos =
    { $n ->
        [one] { $n } asset
       *[other] { $n } assets
    }
carteira-posicoes-de = Positions · { $grupo }
carteira-agrupar = Group positions
carteira-por-classe = By class
carteira-por-conta = By account
carteira-sem-grupo = No grouping
carteira-incluir-encerradas = Include closed positions
carteira-posicoes-atalhos = Enter or double-click opens the asset; right-click shows the shortcuts.
carteira-calculando-posicoes = Calculating positions
carteira-sem-posicoes = No positions yet
carteira-sem-posicoes-texto = Record your first purchase to see the average price, result and real return of each asset.
carteira-conta = Account
carteira-todas-contas = All accounts
carteira-todas-posicoes = All positions

## Returns

carteira-desde-inicio = Since inception
carteira-calculando-rentabilidade = Calculating returns
carteira-sem-historico = No history to calculate
carteira-sem-historico-texto = Returns show up after the first trade.
carteira-fonte-catalogo = { $indice } (index catalog)
carteira-rent-fonte = Source: trades, quotes and dividends · { $taxa } · { $deflator }
carteira-rent-estimado = Inflation months not yet published use the Focus projection (or the latest figure) until the official index comes out.
carteira-desde = Portfolio since { $mes }
carteira-pct-do = % of { $indice }
carteira-acumulado-real = Real cumulative
carteira-serie-carteira = Portfolio
carteira-serie-real = Real portfolio ({ $deflator })
carteira-serie-sp500 = S&P 500 (in { $moeda })
carteira-serie-dolar = US dollar
carteira-mes = Month
carteira-rent-acumulada-titulo = Cumulative return × { $indice } × inflation
carteira-rent-acumulada-aria-desde = Cumulative portfolio return since { $mes }, compared with { $indice } and inflation.
carteira-rent-acumulada-aria-inicio = Cumulative portfolio return since inception, compared with { $indice } and inflation.
carteira-rent-acumulada-tabela = Cumulative return by month
carteira-retorno-mes = Month's return
carteira-retorno-cada-mes = Return of each month
carteira-retorno-metodo = Method: monthly TWR (Modified Dietz), neutralizes contributions and withdrawals
carteira-retorno-aria = Monthly portfolio return in bars, gains above and losses below zero.
carteira-retorno-mensal = Monthly return
carteira-retorno = Return
carteira-patrimonio = Assets
carteira-aportes-liquidos = Net contributions
carteira-patrimonio-aportes = Assets × net contributions
carteira-patrimonio-aportes-fonte = Net contributions = purchases − sales, cumulative; the gap is the gain
carteira-patrimonio-aportes-aria = Invested assets compared with cumulative net contributions, by month.
carteira-patrimonio-aportes-tabela = Assets and contributions by month

## Price chart (asset page)

carteira-grafico-pm = Avg
carteira-grafico-fechamento = Close
carteira-grafico-preco-operacao = Trade price
carteira-marcador-compra = B { $qtd }
carteira-marcador-venda = S { $qtd }

## Allocation and rebalancing

carteira-outras = Others
carteira-outras-de = Others ({ $classes })
carteira-investido = invested
carteira-sem-alvo = no target
carteira-alvo-pct = target { $pct }
carteira-alocacao-classe = Allocation by class
carteira-alocacao-fonte = Market value (or on the curve) in { $moeda }; without a quote, at cost
carteira-alocacao-aria = Portfolio allocation by asset class, as a donut.
carteira-col-alvo = Target
carteira-col-desvio = Deviation
carteira-desvio-peso-alvo = weight { $peso } · target { $alvo }
carteira-desvio-pp = deviation { $desvio } pp
carteira-fora-tolerancia = outside the tolerance
carteira-desvio-titulo = Deviation from target, in percentage points
carteira-desvio-fonte = Solid bars: outside the set tolerance; light ones: within it
carteira-desvio-aria = Deviation of each class from the target allocation.
carteira-desvio-tabela = Deviation from target
carteira-alvos-salvos = Target allocation saved
carteira-alvos-titulo = Target allocation by class
carteira-alvos-soma-passa = Total { $total }% · over 100%
carteira-alvos-soma-completa = Total { $total }% · complete
carteira-alvos-soma-faltam = Total { $total }% · { $falta } pp to go
carteira-descartar = Discard
carteira-salvar-alvos = Save targets
carteira-alvos-tabela = Targets by class
carteira-col-atual = Current
carteira-col-alvo-pct = Target %
carteira-col-tolerancia = Tol. pp
carteira-alvo-de = Target for { $classe } (%)
carteira-tolerancia-de = Tolerance for { $classe } (pp)
carteira-adicionar-classe = Add class
carteira-adicionar-classe-aria = Add class to the target allocation
carteira-escolha-classe = Choose the class
carteira-tolerancia-texto = Tolerance: how many percentage points the weight can drift from the target before the class is “out of band” (default 5).
carteira-fora-tolerancia-alvo = Outside the target tolerance
carteira-fora-tolerancia-alvo-curto = outside the target tolerance
carteira-col-atual-pct = Current %
carteira-col-falta-sobra = Short/over
carteira-col-do-aporte = From contribution
carteira-rebal-titulo = Rebalancing with a new contribution
carteira-novo-aporte = New contribution
carteira-novo-aporte-dica = 0 = only shows the deviation. Short/over: + still to buy, − above the target.
carteira-carteira-hoje = Portfolio today
carteira-depois-aporte = After the contribution
carteira-alvos-incompletos = The targets don't add up to 100%
carteira-alvos-incompletos-texto = The suggestion splits the contribution only among the classes with a target; adjust the target allocation above for a complete calculation.
carteira-aporte-sem-destino = Part of the contribution doesn't fit the limits
carteira-aporte-sem-destino-texto = has nowhere to go: the assets reached their concentration limit. Raise a limit or pick a new asset.
carteira-defina-alvos = Set the target allocation to see how much each class is short or over.
carteira-rebal-tabela = Rebalancing by class
carteira-nao-recomendacao = Not investment advice
carteira-rebal-aviso = Analysis based on your data and the targets you set; it is not investment advice.
carteira-rebal-aviso-sugestao = The suggestion only shows how the contribution brings the portfolio closer to the targets, without selling anything.
carteira-calculando-alocacao = Calculating allocation

## Trades, corporate actions and dividends (tables)

carteira-compra = Buy
carteira-venda = Sell
carteira-ajuste-de-posicao = Position adjustment
carteira-compras = Purchases
carteira-vendas = Sales
carteira-compra-de = Purchase of { $qtd } { $ativo } on { $data }
carteira-venda-de = Sale of { $qtd } { $ativo } on { $data }
carteira-compra-qtd = Purchase of { $qtd } { $ativo }
carteira-venda-qtd = Sale of { $qtd } { $ativo }
carteira-erro-excluir = Couldn't delete
carteira-excluir-n =
    { $n ->
        [one] Delete
       *[other] Delete { $n }
    }
carteira-registros-excluir = Records to delete
carteira-e-mais = and { $n } more
carteira-menu-editar = Edit…
carteira-menu-copiar-linha = Copy row
carteira-menu-excluir = Delete…
carteira-menu-copiar-n-linhas = Copy { $n } rows
carteira-excluir-operacoes =
    { $n ->
        [one] Delete trade
       *[other] Delete { $n } trades
    }
carteira-excluir-operacoes-nota =
    { $n ->
        [one] The linked transaction in the brokerage cash goes too. You can undo it in the change history.
       *[other] The linked transactions in the brokerage cash go too. You can undo them all at once in the change history.
    }
carteira-operacoes-excluidas =
    { $n ->
        [one] Trade deleted
       *[other] { $n } trades deleted
    }
carteira-operacoes-selecionadas =
    { $n ->
        [one] { $n } trade selected
       *[other] { $n } trades selected
    }
carteira-excluir-n-operacoes =
    { $n ->
        [one] Delete { $n } trade…
       *[other] Delete { $n } trades…
    }
carteira-acoes-de-n-operacoes =
    { $n ->
        [one] Actions for { $n } trade
       *[other] Actions for { $n } trades
    }
carteira-n-operacoes =
    { $n ->
        [one] { $n } trade
       *[other] { $n } trades
    }
carteira-col-data = Date
carteira-col-tipo = Type
carteira-col-preco = Price
carteira-col-custos = Costs
carteira-col-bruto = Gross
carteira-col-caixa = Cash
carteira-fora-do-caixa = outside cash
carteira-col-cambio = FX rate
carteira-col-conta = Account
carteira-col-nota = Note
carteira-col-acoes = Actions
carteira-editar-operacao-de = Edit trade of { $data }
carteira-filtrar-ativo = Filter by asset
carteira-todos-ativos = All assets
carteira-filtrar-conta = Filter by account
carteira-operacoes = Trades
carteira-sem-operacoes = No trades recorded.
carteira-evento-texto = { $tipo } of { $ativo } on { $data }
carteira-excluir-eventos =
    { $n ->
        [one] Delete corporate action
       *[other] Delete { $n } corporate actions
    }
carteira-excluir-eventos-nota = Quantity and average price are recalculated. You can undo it in the change history.
carteira-eventos-excluidos =
    { $n ->
        [one] Corporate action deleted
       *[other] { $n } corporate actions deleted
    }
carteira-eventos-selecionados =
    { $n ->
        [one] { $n } action selected
       *[other] { $n } actions selected
    }
carteira-excluir-n-eventos =
    { $n ->
        [one] Delete { $n } action…
       *[other] Delete { $n } actions…
    }
carteira-acoes-de-n-eventos =
    { $n ->
        [one] Actions for { $n } corporate action
       *[other] Actions for { $n } corporate actions
    }
carteira-col-data-ex = Ex date
carteira-col-evento = Action
carteira-col-detalhe = Detail
carteira-fator = factor { $fator }
carteira-fator-custo = factor { $fator } · cost { $custo }
carteira-col-observacoes = Notes
carteira-col-origem = Origin
carteira-registrado-por-voce = Recorded by you
carteira-coleta-cotacoes = { $fonte } (quote collection)
carteira-evento-coletado-nota = Split reported by the quote source; can be undone in the change history.
carteira-sem-eventos = No corporate actions.
carteira-excluir-proventos =
    { $n ->
        [one] Delete dividend
       *[other] Delete { $n } dividends
    }
carteira-excluir-proventos-nota =
    { $n ->
        [one] The linked credit in cash goes too. You can undo it in the change history.
       *[other] The linked credits in cash go too. You can undo them all at once in the change history.
    }
carteira-proventos-excluidos =
    { $n ->
        [one] Dividend deleted
       *[other] { $n } dividends deleted
    }
carteira-proventos-selecionados =
    { $n ->
        [one] { $n } dividend selected
       *[other] { $n } dividends selected
    }
carteira-excluir-n-proventos =
    { $n ->
        [one] Delete { $n } dividend…
       *[other] Delete { $n } dividends…
    }
carteira-acoes-de-n-proventos =
    { $n ->
        [one] Actions for { $n } dividend
       *[other] Actions for { $n } dividends
    }
carteira-col-pagamento = Payment
carteira-col-data-com = Record date
carteira-col-base = Base
carteira-col-ir-retido = Tax withheld
carteira-col-liquido = Net
carteira-col-situacao = Status
carteira-recebido = Received
carteira-anunciado = Announced
carteira-sem-proventos-periodo = No dividends in the period.
carteira-recebido-moeda = Received ({ $moeda })
carteira-provento-texto = { $tipo } of { $ativo } paid on { $data }

## Dividends and passive income (panels)

carteira-col-12-meses = 12 months
carteira-col-yield-12m = 12m yield
carteira-somando-proventos = Adding up dividends
carteira-ultimos-12-meses = Last 12 months
carteira-por-mes = Per month
carteira-yield-carteira = Portfolio yield
carteira-yield-formula = Dividends ÷ market value
carteira-meta-renda = Monthly income goal
carteira-patrimonio-necessario = Assets needed
carteira-falta = To go
carteira-quanto-receber = How much you want per month
carteira-progresso-meta = Goal progress
carteira-sem-yield = No yield to work out the assets needed
carteira-sem-yield-texto = The portfolio hasn't received any dividends in the last 12 months.
carteira-por-ativo-12m = By asset · 12 months
carteira-renda-por-ativo = Passive income by asset
carteira-sem-proventos-12m = No dividends in the last 12 months.
carteira-patrimonio-necessario-texto = Assets needed = goal × 12 ÷ yield of the last 12 months. It's a calculation about your portfolio's past, not a promise of income.
carteira-grafico-proventos-liquidos = Net dividends
carteira-meta-curta = Goal
carteira-meta-valor = Goal { $valor }
carteira-proventos-mes-titulo = Dividends by month, last 12 months
carteira-proventos-mes-fonte = Dividends received, net of withheld tax, in { $moeda } on the payment date
carteira-meta-mensal = Monthly goal
carteira-proventos-mes-aria = Net dividends received per month in the last 12 months.
carteira-proventos-mes-tabela = Dividends by month

## Fee cost

carteira-taxas-titulo = Long-term fee cost
carteira-taxas-retorno = Real return before fees (% p.a.)
carteira-taxas-alternativa = Alternative scenario fee (% p.a.)
carteira-taxas-aporte = Monthly contribution (optional)
carteira-taxas-nenhum = No fund or plan with a known fee
carteira-taxas-nenhum-texto = Enter the management fee of your pension plans; the funds' fee comes from their CVM filings (asset analysis collection).
carteira-taxas-tabela = Fee cost
carteira-taxas-fundo = Fund or plan
carteira-taxas-hoje = Today
carteira-taxas-em-anos = In { $anos } years
carteira-taxas-economia = +{ $valor } with a lower fee
carteira-taxa-administracao = management { $pct }
carteira-taxa-performance = performance { $pct }
carteira-taxa-carregamento = loading { $pct }
carteira-taxas-sem-taxa = No known fee (left out): { $lista }.

## Risk

carteira-risco-sem-queda = no drawdown
carteira-risco-queda-inicio = { $pct } from the start to { $fundo }
carteira-risco-queda = { $pct } from { $pico } to { $fundo }
carteira-risco-recuperou = { $queda }, recovered in { $mes }
carteira-risco-nao-recuperou = { $queda }, not recovered yet
carteira-risco-volatilidade = Volatility
carteira-risco-pior-queda = Worst drawdown
carteira-risco-retorno = Return
carteira-risco-sharpe-de = Sharpe ({ $indice })
carteira-risco-melhor-pior = Best and worst month
carteira-risco-insuficiente = Not enough monthly history.
carteira-risco-retorno-aa = Return p.a.
carteira-risco-sharpe = Sharpe
carteira-risco-parte = Share of risk
carteira-risco-sem-dados = no data
carteira-risco-titulo = Portfolio risk
carteira-risco-texto = How much the portfolio swings, the largest drawdown, the return above { $indice } per unit of risk and how much the assets move together, from each asset's monthly returns with today's weights.
carteira-risco-calculando = Calculating risk
carteira-risco-sem-ativos = No assets in the portfolio
carteira-risco-sem-ativos-texto = Record trades in Portfolio › Trades to measure risk.
carteira-risco-hoje = Today's portfolio over the period
carteira-risco-historico = Actual history (purchases and sales at each point in time)
carteira-risco-por-classe = Risk by class
carteira-risco-correlacao-classes = Correlation between classes
carteira-risco-por-ativo = Risk by asset
carteira-risco-correlacao-ativos = Correlation between the largest assets
carteira-risco-parte-texto = Share of risk: how much each row accounts for the portfolio's swings (they add up to 100%). In yellow, the ones that weigh much more in risk than in value.

## Record trade

carteira-op-editar = Edit trade
carteira-op-registrar = Record trade
carteira-op-editar-texto = The trade is saved again with the new values, along with the linked transaction in the brokerage cash.
carteira-op-registrar-texto = Purchase or sale with costs. The brokerage cash gets the linked transaction (can be undone in the history).
carteira-erro-escolha-ativo = Choose the asset.
carteira-erro-escolha-conta-corretora = Choose the brokerage account.
carteira-erro-data = Enter the date (mm/dd/yyyy).
carteira-erro-quantidade = Enter a quantity greater than zero.
carteira-erro-venda-maior = Sale of { $qtd } is larger than the position of { $posicao } in this account on { $data }.
carteira-na-data = that date
carteira-erro-preco = Enter the unit price.
carteira-erro-cambio = The exchange rate must be positive.
carteira-op-alterada = Trade updated
carteira-op-registrada = Trade recorded
carteira-op-form = Trade
carteira-op-tipo = Trade type
carteira-op-renda-fixa = Fixed income: number of bonds × unit price; sales go out FIFO.
carteira-buscar-ativo = Search asset
carteira-buscar-ativo-dica = Ticker or name: pick one you already added or add a new one (B3, US, other exchanges, crypto).
carteira-ativo = Asset
carteira-conta-corretora = Brokerage account
carteira-conta-moeda = Account in { $conta }; asset in { $ativo }.
carteira-data-pregao = Trade date
carteira-quantidade = Quantity
carteira-posicao-na-data = Position in this account on that date: { $qtd }
carteira-fracoes = Fractions with up to 8 decimals (foreign, crypto, Tesouro).
carteira-preco-unitario = Unit price ({ $moeda })
carteira-cambio-moeda = Exchange rate { $moeda }/{ $base }
carteira-cambio-dica = { $base ->
        [BRL] 1 unit of the asset's currency in reais. Empty: the day's PTAX stored in the app.
       *[other] 1 unit of the asset's currency in { $base }. Empty: the day's exchange rate stored in the app.
    }
carteira-ptax-do-dia = { $base ->
        [BRL] Day's PTAX
       *[other] Day's exchange rate
    }
carteira-custos-corretagem = Costs (brokerage and exchange fees)
carteira-impostos-retidos = Taxes withheld (IRRF)
carteira-nota-corretagem = Brokerage note
carteira-observacoes = Notes
carteira-day-trade = Day trade
carteira-day-trade-dica = Buy and sell on the same day (separate income tax calculation in Phase 3).
carteira-lancar-caixa = Post to the brokerage cash
carteira-lancar-caixa-dica = Creates the linked transaction (investment contribution/withdrawal), which counts as neither income nor expense.
carteira-lancar-caixa-dica-edicao = First check whether the brokerage statement already brought this debit or credit: posting it again counts it twice in the balance.
carteira-bruto = Gross
carteira-sai-caixa = Leaves cash
carteira-entra-caixa = Enters cash
carteira-bruto-moeda = Gross in { $moeda }
carteira-preco = Price
carteira-salvar-alteracoes = Save changes
carteira-registrar-compra = Record purchase
carteira-registrar-venda = Record sale

## Record dividend

carteira-prov-titulo = Record dividend
carteira-prov-texto = Received goes into the brokerage cash as income (Investment income › Dividends/JCP); announced stays only in the calendar until payment.
carteira-erro-escolha-conta = Choose the account.
carteira-erro-data-pagamento = Enter the payment date.
carteira-erro-bruto = Enter the gross amount (or the amount per share).
carteira-erro-imposto = Tax larger than the gross amount.
carteira-prov-registrado = Dividend recorded
carteira-prov-anunciado-registrado = Announced dividend recorded
carteira-prov-form = Dividend
carteira-prov-situacao = Dividend status
carteira-ativos-com-operacoes = Assets with trades.
carteira-tipo = Type
carteira-data-com = Record date
carteira-pagamento = Payment
carteira-valor-por-cota = Amount per share ({ $moeda })
carteira-valor-por-cota-dica = Optional: with the quantity, works out the gross amount.
carteira-quantidade-base = Base quantity
carteira-na-data-qtd = On that date: { $qtd }
carteira-cotas-data-com = Shares on the record date.
carteira-valor-bruto = Gross amount
carteira-bruto-calculado = Calculated: amount per share × quantity.
carteira-imposto-retido = Tax withheld
carteira-imposto-padrao = Default: 17.5% on JCP (15% through 2025), 30% on US dividends, 0 on the rest.
carteira-cambio-pagamento = { $base ->
        [BRL] Empty: PTAX on the payment date.
       *[other] Empty: exchange rate on the payment date.
    }
carteira-lancar-credito = Post the credit to the brokerage cash
carteira-imposto = Tax
carteira-liquido = Net

## Record corporate action

carteira-ev-titulo = Record corporate action
carteira-ev-texto = Actions on a date apply before the trades of that same date (“ex” date).
carteira-ev-fator-split = Greater than 1: 2 = each share becomes two (1:2).
carteira-ev-fator-grupamento = Between 0 and 1: 0.1 = ten shares become one (10:1).
carteira-ev-fator-bonificacao = New shares per share: 0.1 = 10% bonus.
carteira-ev-fator-incorporacao = Optional: shares of the new asset per old share (default 1).
carteira-ev-fator-ticker = Optional: exchange factor (default 1).
carteira-ev-erro-fator = Enter the factor.
carteira-ev-erro-split = A split needs a factor greater than 1.
carteira-ev-erro-grupamento = A reverse split needs a factor between 0 and 1.
carteira-ev-erro-bonificacao = Bonus shares need a positive factor.
carteira-ev-erro-data = Enter the date.
carteira-ev-erro-valor = Enter the total amount returned.
carteira-ev-erro-ativo-resultante = Choose the resulting asset.
carteira-ev-registrado = Corporate action recorded
carteira-ev-data = Ex/effective date
carteira-ev-fator = Factor
carteira-ev-custo-por-acao = Cost assigned per share ({ $moeda })
carteira-ev-custo-dica = Given in the notice to shareholders.
carteira-ev-valor-devolvido = Total amount returned
carteira-ev-valor-devolvido-dica = Reduces the position's cost.
carteira-ev-ativo-resultante = Resulting asset
carteira-ev-ativo-resultante-dica = The position moves to it on the date.
carteira-ev-cisao-texto = Spin-offs and subscriptions are kept in the history; record the resulting trade (the purchase of the new asset or of the subscribed shares) separately.
carteira-ev-registrar = Record action

## Record dividends (grid and to confirm)

carteira-grade-texto = Confirm the announced dividends you received or type (or paste) date, ticker and amount. Treasury finds the dividend, the broker, the type and the tax.
carteira-grade-como = How to record
carteira-grade-a-confirmar = To confirm ({ $n })
carteira-grade-digitar = Type
carteira-grade-ticker-linha = Ticker, row { $n }
carteira-grade-ativos-linha = Assets for row { $n }
carteira-grade-em-carteira = in portfolio
carteira-proventos-registrados =
    { $n ->
        [one] { $n } dividend recorded
       *[other] { $n } dividends recorded
    }
carteira-grade-nada-novo = Nothing new to record
carteira-grade-tabela = Dividends to record
carteira-grade-ticker = Ticker
carteira-grade-valor-recebido = Amount received
carteira-grade-entendeu = What Treasury understood
carteira-grade-data-linha = Payment date, row { $n }
carteira-grade-valor-linha = Amount received, row { $n }
carteira-grade-data-invalida = Invalid date.
carteira-grade-complete = Fill in date, ticker and amount.
carteira-grade-remover-linha = Remove row { $n }
carteira-grade-ajuda = The amount received is what was credited at the broker (net). A blank date repeats the one above; Enter goes to the next row. You can paste several rows from a spreadsheet.
carteira-grade-prontas = { $n } ready
carteira-grade-duplicadas = { $n } already recorded
carteira-grade-problema =
    { $n ->
        [one] { $n } with a problem
       *[other] { $n } with problems
    }
carteira-grade-nenhuma-completa = No complete row yet.
carteira-registrar-n-proventos =
    { $n ->
        [one] Record { $n } dividend
       *[other] Record { $n } dividends
    }
carteira-registrar = Record
carteira-bruto-ir = gross { $bruto } · tax { $ir }
carteira-anunciado-curto = announced
carteira-conta-provento-de = Account for the { $ativo } dividend

carteira-conf-confere = matches
carteira-conf-divergem = sources differ
carteira-conf-uma-fonte = one source
carteira-conf-fonte = { $fonte }: { $valor } per share · record date { $data }
carteira-conf-fonte-pago = { $fonte }: { $valor } per share · record date { $data } · paid on { $pago }
carteira-conferencia = { $status }: { $fontes }
carteira-conferencia-usado = { $status }: { $fontes } · using { $fonte }
carteira-agente-pedido = Check these dividends awaiting confirmation, where the sources differ or only one source has the dividend (use list_pending_income, check_dividends and get_fundamentus and, if you can, the company's or fund's investor relations page):
carteira-agente-fonte = { $fonte } { $valor }/share, record date { $data }
carteira-agente-fonte-pago = { $fonte } { $valor }/share, record date { $data }, paid { $pago }
carteira-agente-item = - { $ativo } { $tipo } (record date { $data }): { $fontes }
carteira-agente-fecho = For each one, say whether it is actually a problem and what the correct amount per share and dates are. When a correction is needed, propose the entry with record_income_event (I approve before it is saved).
carteira-pend-vazio = No announced dividends to confirm: everything the calendar and B3 brought in for your assets is already in the portfolio. Use "Type" to enter one that didn't show up.
carteira-pend-texto = Announced with a record date when you held the asset and not entered yet. Uncheck what you didn't receive and correct the amount if your broker's statement shows a different one.
carteira-pend-tabela = Dividends to confirm
carteira-pend-marcar-todos = Check all
carteira-pend-qtd-cota = Qty. × per share
carteira-pend-recebi = I received the { $ativo } dividend paid on { $data }
carteira-pend-data-de = Payment date for { $ativo }
carteira-pend-sem-data = no date in the source: check it
carteira-pend-detalhe = { $tipo } · { $conta } · record date { $data }
carteira-ir-valor = Tax { $valor }
carteira-pend-valor-de = Amount received for { $ativo }
carteira-pend-duvidosos =
    { $n ->
        [one] { $n } dividend with conflicting sources or a single source.
       *[other] { $n } dividends with conflicting sources or a single source.
    }
carteira-pend-pedir-agente = Ask the agent to check
carteira-pend-marcados = { $n } of { $total } checked
carteira-confirmar-n-proventos =
    { $n ->
        [one] Confirm { $n } dividend
       *[other] Confirm { $n } dividends
    }
carteira-confirmar = Confirm

## Index series

carteira-unidade-anualizada = % p.a. (annualized, 252 business days)

## Allocation by sector

carteira-setor-titulo = By sector
carteira-setor-sem = No sector
carteira-setor-ativos =
    { $n ->
        [one] { $n } asset
       *[other] { $n } assets
    }
carteira-setor-vazio = No positions with a market value.
carteira-setor-rodape = Sector from each asset's record (yours or an approved suggestion from the sources).
carteira-setor-completar =
    { $n ->
        [one] Fill in the sector of { $n } asset in Asset health
       *[other] Fill in the sector of { $n } assets in Asset health
    }

## Vendas (resultado nominal e real)

carteira-aba-vendas = Sales
carteira-deflator-igpm-dica = General Market Price Index (FGV, via BCB/SGS 189)
carteira-vendas-titulo = Result of sales
carteira-vendas-total-vendido = Total sold
carteira-vendas-n = { $n ->
        [one] { $n } sale
       *[other] { $n } sales
    }: { $ganhos } with a gain, { $perdas } with a loss
carteira-vendas-nominal = Price result
carteira-vendas-nominal-dica = Sale minus the cost at the average price, as for income tax
carteira-vendas-real = Real result ({ $indice })
carteira-vendas-real-dica = { $ganhos } with a real gain, { $perdas } with a real loss
carteira-vendas-faltam = { $n ->
        [one] { $n } sale
       *[other] { $n } sales
    } without { $indice } for the period left out
carteira-vendas-efeito = Taken by { $indice }
carteira-vendas-efeito-dica = How much the cost corrected by { $indice } exceeds the original cost
carteira-vendas-por-mes = Result by month: nominal × real
carteira-vendas-tabela = Sales
carteira-vendas-col-venda = Sale
carteira-vendas-col-custo = Cost
carteira-vendas-col-nominal = Nominal (price)
carteira-vendas-col-real = Real ({ $indice })
carteira-vendas-col-indice = { $indice } over the period
carteira-vendas-col-prazo = Avg. holding
carteira-vendas-dias = { $n } d
carteira-vendas-sem-indice = No { $indice } for the whole period of this sale
carteira-vendas-dica-custo = Cost corrected by { $indice } from each purchase to the sale:
carteira-vendas-vazio-titulo = No sales in the period
carteira-vendas-vazio-texto = Sales recorded in Trades show up here, with the gain or loss in money, before and after inflation.
carteira-vendas-metodo = Cost at the average price (fixed income: first in, first out). The real result brings the cost of each purchase to the sale date by { $indice }; the holding period is weighted by cost. Amounts in the base currency, before income tax.
carteira-vendas-dica-conta = { $custo } × (1 { $variacao }) = { $corrigido }
carteira-vendas-registrar = Record trade
carteira-vendas-menu-editar = Edit sale…
carteira-vendas-menu-compra = Record a purchase of this asset…
carteira-vendas-menu-venda = Record another sale of this asset…
carteira-vendas-menu-evento = Record event (split, bonus shares…)…
carteira-vendas-menu-excluir = Delete sale…
carteira-vendas-erro-abrir = Could not open the sale
carteira-vendas-excluir-nota = The average price and the result of the following sales are recalculated. You can undo it right after.
carteira-vendas-col-proventos = Distributions
carteira-vendas-col-total = With distributions
carteira-vendas-total = With distributions
carteira-vendas-total-dica = Price plus { $proventos } of distributions received by the sold units; income tax is only on the price
carteira-vendas-dica-total = Price { $preco } + distributions { $proventos } received by the sold units between purchase and sale
carteira-vendas-dica-total-real = Real with distributions ({ $indice }): { $valor }
carteira-conta-operacao = Account (brokerage or bank)
carteira-op-modo = How to enter
carteira-op-modo-total = Total amount
carteira-op-modo-unitario = Quantity × price
carteira-valor-aplicado = Amount invested
carteira-valor-bruto-resgate = Gross redemption amount
carteira-valor-bruto-resgate-dica = Before income tax; the withheld tax goes in Taxes withheld and the net shows in the summary.
carteira-resgate-total = Redeem the whole position
carteira-erro-valor-total = Enter the amount.
carteira-erro-sem-posicao = There is no position in this account on this date.
carteira-previdencia-saldo-titulo = { $n ->
        [one] Pension plan without a recent balance
       *[other] { $n } pension plans without a recent balance
    }
carteira-previdencia-saldo-de = last balance on { $data }
carteira-previdencia-sem-saldo = no balance entered (worth what was contributed)
carteira-previdencia-saldo-texto = Plans without a public unit price are worth the balance you enter. Open the plan and use Enter balance with the amount the provider shows.
carteira-saldo-na-data = Balance in this account on the date: { $valor }
carteira-resgate-parcial = Partial withdrawal: { $pct } of the balance.
carteira-aplicacao-de = Investment of { $valor } in { $ativo } on { $data }
carteira-resgate-de = Redemption of { $valor } from { $ativo } on { $data }

## Fixed income calculator (compare offers and sell before maturity; Brazil only)
carteira-rf-aba = Fixed income calculator
carteira-rf-cmd = Fixed income calculator
carteira-rf-cmd-busca = fixed income,compare offers,cdb,lci,lca,treasury,savings,equivalence,mark to market,sell bond
carteira-rf-comparar-titulo = Compare offers
carteira-rf-comparar-texto = Type the offers you want to see side by side. Results stay in the order you typed them.
carteira-rf-valor = Amount invested
carteira-rf-inicio = Investment date
carteira-rf-inicio-dica = Empty: today
carteira-rf-ofertas = Offers
carteira-rf-oferta-n = Offer { $n }
carteira-rf-tipo = Type
carteira-rf-indice = Yield
carteira-rf-taxa = Rate (%)
carteira-rf-meses = Term (months)
carteira-rf-resgate = Or redeem on
carteira-rf-nome = Name (optional)
carteira-rf-adicionar = Add offer
carteira-rf-remover = Remove offer { $n }
carteira-rf-limite = Up to { $max } offers.
carteira-rf-indice-cdi-pct = % of CDI
carteira-rf-indice-cdi-mais = CDI + %
carteira-rf-indice-selic-mais = Selic + %
carteira-rf-indice-pre = Fixed rate
carteira-rf-indice-ipca-mais = IPCA + %
carteira-rf-indice-poupanca = Savings account
carteira-rf-premissas = Assumptions
carteira-rf-premissas-texto = Used for every offer. Type another value to simulate; clear it to go back to the vault's.
carteira-rf-cdi = CDI (% p.a.)
carteira-rf-selic = Selic target (% p.a.)
carteira-rf-ipca = Expected IPCA (% p.a.)
carteira-rf-tr = TR (% p.m.)
carteira-rf-fonte-user = typed by you
carteira-rf-fonte-vault = from the vault
carteira-rf-fonte-vault-data = from the vault · { $data }
carteira-rf-fonte-projection = projection (Focus)
carteira-rf-fonte-default = default (no data in the vault)
carteira-rf-incompleto = Complete the offers
carteira-rf-incompleto-texto = Enter the amount and, for each offer, the rate and a term in months or the redemption date.
carteira-rf-tabela = Fixed income offers
carteira-rf-col-oferta = Offer
carteira-rf-col-prazo = Term
carteira-rf-dias = { $n ->
        [1] 1 calendar day
       *[other] { $n } calendar days
    }
carteira-rf-ate = until { $data }
carteira-rf-col-bruto = Gross
carteira-rf-col-iof = IOF
carteira-rf-col-ir = Income tax
carteira-rf-isento = exempt
carteira-rf-col-custodia = Custody
carteira-rf-col-liquido = Net
carteira-rf-col-ano = Net per year
carteira-rf-col-real = Above inflation
carteira-rf-col-equivalencia = Equivalence
carteira-rf-equiv-cdb = equals { $pct }% of CDI in a CDB
carteira-rf-equiv-lci = equals { $pct }% of CDI in an LCI/LCA
carteira-rf-col-fgc = FGC
carteira-rf-sim = Yes
carteira-rf-nao = No
carteira-rf-col-avisos = Notes
carteira-rf-venda-titulo = Sell before maturity
carteira-rf-venda-texto = What a fixed-rate or IPCA+ bond is worth on the sale date at the market rate, next to holding it to maturity.
carteira-rf-venda-carteira = Fill in from a bond in the portfolio
carteira-rf-venda-carteira-escolha = Choose a bond…
carteira-rf-venda-indice = Index
carteira-rf-venda-pre = Fixed rate
carteira-rf-venda-ipca = IPCA+
carteira-rf-venda-aplicacao = Investment date
carteira-rf-venda-vencimento = Maturity
carteira-rf-venda-data = Sale date
carteira-rf-venda-contratada = Contracted rate (% p.a.)
carteira-rf-venda-mercado = Market rate today (% p.a.)
carteira-rf-venda-taxa-real = Real rate, above IPCA
carteira-rf-venda-isento = Income tax exempt
carteira-rf-venda-ipca-premissa = Assumed IPCA (% p.a.)
carteira-rf-venda-incompleto = Complete the bond
carteira-rf-venda-incompleto-texto = Enter the amount, the dates (the sale between the investment and maturity) and both rates.
carteira-rf-venda-curva = Value on the curve
carteira-rf-venda-preco = Market price
carteira-rf-venda-diferenca = Difference (mark to market)
carteira-rf-venda-diferenca-dica = Market price minus the value on the curve: positive when rates fell since the investment.
carteira-rf-venda-tabela = Sell today or hold to maturity
carteira-rf-venda-hoje = Sell today
carteira-rf-venda-levar = Hold to maturity
carteira-rf-venda-bruto = Gross
carteira-rf-venda-ir = Income tax
carteira-rf-venda-iof = IOF
carteira-rf-venda-liquido = Net
carteira-rf-venda-empate = Break-even rate
carteira-rf-venda-empate-pre = For the switch to break even, a new bond until { $data } must pay { $taxa }% p.a.
carteira-rf-venda-empate-ipca = For the switch to break even, a new bond until { $data } must pay IPCA + { $taxa }% p.a.
carteira-rf-venda-empate-nenhuma = No break-even rate for these numbers.

## Result bridge (portfolio.bridge): from the starting value to the ending value, in money.
carteira-ponte-titulo = Where the result came from
carteira-ponte-explica = Money first: what you had at the start, what prices, dividends and costs changed, what you put in and took out, and what you have at the end. Percentages come later.
carteira-ponte-inicio = Value at start
carteira-ponte-preco = Price gain
carteira-ponte-cambio = of which exchange rate: { $valor }
carteira-ponte-cambio-coluna = Exchange rate
carteira-ponte-proventos = Dividends
carteira-ponte-taxas = Fees
carteira-ponte-impostos = Taxes
carteira-ponte-aportes = Contributions
carteira-ponte-retiradas = Withdrawals
carteira-ponte-fim = Value at end
carteira-ponte-resultado = Result for the period
carteira-ponte-resultado-curto = Result
carteira-ponte-resultado-dica = Ending value minus starting value, minus contributions plus withdrawals: what the investments earned in money.
carteira-ponte-inflacao = Lost to inflation ({ $indice })
carteira-ponte-real = Real result
carteira-ponte-linhas = Result bridge lines
carteira-ponte-linha = Line
carteira-ponte-valor = Amount
carteira-ponte-aria = Result bridge from { $de } to { $ate }: from the starting value to the ending value, line by line.
carteira-ponte-periodo = From { $de } to { $ate }
carteira-ponte-fonte = Source: recorded trades, prices, exchange rates and dividends · past performance, not an investment recommendation
carteira-ponte-estimado = estimated
carteira-ponte-estimado-dica = Some value came from a stale price, an estimated exchange rate or an index not yet published.
carteira-ponte-por-ativo = By asset ({ $n })
carteira-ponte-tabela-ativos = Result bridge by asset
carteira-ponte-calculando = Building the result bridge
carteira-ponte-vazio = No amounts in the period.

## Risk of the return series (semideviation and drawdowns).
carteira-risco-semidesvio = Semideviation (the typical size of drops)
carteira-risco-queda-datas = { $pct } from { $pico } to { $fundo }
carteira-risco-voltou-em = { $queda }, back to the peak on { $data }
carteira-risco-sem-volta = { $queda }, not back to the peak yet
carteira-risco-tempo-abaixo = Longest time below the peak
carteira-risco-dias =
    { $n ->
        [one] { $n } day
       *[other] { $n } days
    }
carteira-risco-ainda-abaixo = still below
carteira-risco-metodo-mensal = Monthly return (Modified Dietz)
carteira-risco-metodo-diario = Daily return
carteira-risco-metodo-diario-fonte = Method: chained daily TWR, neutralizes contributions and withdrawals
carteira-risco-como = How we calculate: { $metodo }. Semideviation, worst drawdown and time below the peak use the same return series. Past performance does not guarantee future results.

## Standard period and comparison (docs/portfolio-compare.md)

carteira-periodo-de = Period: { $aba }
carteira-periodo-ponte = Result bridge
carteira-comparar-com = Compare with
carteira-comparar-tipo = Kind of series to compare
carteira-comparar-indice = Index
carteira-comparar-ativo = Asset
carteira-comparar-cdi-mais = { $indice } + rate
carteira-comparar-escolha = Choose the series
carteira-comparar-taxa = Rate a year, in %
carteira-comparar-ao-ano = % a year
carteira-comparar-adicionar = Add
carteira-comparar-escolhidas = Compared series
carteira-comparar-remover = Remove { $serie } from the comparison
carteira-comparar-limite = Up to { $n } series at a time.
carteira-comparar-aviso = Past performance side by side, not a recommendation or a promise of returns.
carteira-mapa-titulo = Return of each month, by year
carteira-mapa-fonte = Monthly return by Modified Dietz; the total compounds the period's months in the year and the last column is { $indice } over the same months.
carteira-mapa-ano = Year
carteira-mapa-total = Total
carteira-mapa-aria = Heat map of monthly returns
carteira-contrib-titulo = Top contributors
carteira-contrib-calculando = Working out the contributions…
carteira-contrib-somaram = Added the most
carteira-contrib-tiraram = Took away the most
carteira-contrib-fonte = Result of each asset from { $de } to { $ate } (price change, sales and income), in money and in points of the portfolio's return. Past performance.
carteira-contrib-vazio = No asset had a result in the period.
carteira-contrib-tabela = Contribution of each asset
carteira-contrib-ativo = Asset
carteira-contrib-resultado = Result
carteira-contrib-proventos = Income
carteira-contrib-pontos = Points
carteira-prov-mes-titulo = Income by month, year over year
carteira-prov-mes-calculando = Adding up the income…
carteira-prov-mes-fonte = Income received, net, in { $moeda }, over the last three years.
carteira-prov-mes-aria = Income of each month over the last three years
carteira-prov-mes-tabela = Income by month and year

## Cost method and annualized return in the sales book

carteira-vendas-col-medio = Average cost (BR tax)
carteira-vendas-col-irr = Return p.a.
carteira-vendas-irr-dica = Annualized return of the sale: the yearly rate that equates the cost of each purchase sold, the income received and the sale amount. Past performance, not a promise.
carteira-vendas-medio = By average cost (BR tax)
carteira-vendas-medio-dica = The same result by average cost, the basis of Brazilian income tax
carteira-vendas-nominal-dica-fifo = Sale minus the cost of the oldest purchases (first in, first out)
carteira-vendas-nominal-dica-pool = Sale minus the cost by the UK rule (same day, 30 days, pool)
carteira-vendas-metodo-fifo = Cost by first in, first out (fixed income: the lots); the result by average cost is in the next column. The real result brings the cost of each purchase to the sale date by { $indice }; the average holding period is weighted by cost. Amounts in the base currency, before tax.
carteira-vendas-metodo-pool = Cost by the UK rule: purchases on the same day match first, then those in the following 30 days, and the rest comes from the pool at average cost (fixed income: the lots); the result by average cost is in the next column. The real result brings the cost of each purchase to the sale date by { $indice }. Amounts in the base currency, before tax.
carteira-vendas-metodo-pais = The method follows the family's country; you can change it in Settings › Family.
carteira-vendas-metodo-escolhido = Method chosen in Settings › Family.

## Aportes programados e carteira por objetivo

carteira-objetivo-titulo = By goal
carteira-objetivo-vazio = No position is linked to a goal yet. Link positions to a goal to see the portfolio by goal.
carteira-objetivo-ir-metas = Go to Goals
carteira-objetivo-alvo = { $n } linked position(s) · { $pct } of the target
carteira-objetivo-sem = No goal
carteira-objetivo-nota = By the weights of the positions linked to each goal (fractions and shared pool).
carteira-plano-titulo = Scheduled contributions
carteira-plano-novo = New scheduled contribution
carteira-plano-editar = Edit scheduled contribution
carteira-plano-dialogo-texto = Every month, on the chosen day, the contribution shows up for you to confirm at that day's price. Nothing is recorded without your confirmation.
carteira-plano-nome = Name
carteira-plano-ativo = Asset
carteira-plano-conta = Brokerage account
carteira-plano-valor = Amount of each contribution
carteira-plano-dia = Day of the month
carteira-plano-dia-dica = From 1 to 28.
carteira-plano-inicio = Start
carteira-plano-fim = End
carteira-plano-fim-dica = Empty: no end date.
carteira-plano-meta = Goal
carteira-plano-meta-dica = Which goal the contribution is for (for reference only).
carteira-plano-sem-meta = None
carteira-plano-ativo-ligado = Active (uncheck to pause)
carteira-plano-salvo = Scheduled contribution saved
carteira-plano-a-confirmar = { $n } scheduled contribution(s) to confirm. Adjust the quantity and price if needed.
carteira-plano-a-confirmar-lista = Contributions to confirm
carteira-plano-preco-de = price from { $data }
carteira-plano-sem-preco = no quote: enter the price
carteira-plano-quantidade-de = Quantity of { $ativo } on { $data }
carteira-plano-preco-rotulo = Price of { $ativo } on { $data }
carteira-plano-confirmar = Confirm
carteira-plano-pular = Skip
carteira-plano-confirmado = Purchase of { $ativo } recorded
carteira-plano-pulado = Contribution to { $ativo } skipped
carteira-plano-erro = Could not record the contribution
carteira-plano-vazio = No scheduled contributions. Create one so you don't have to enter the same purchase every month.
carteira-plano-lista = Scheduled contributions
carteira-plano-pausado = paused
carteira-plano-resumo = { $ativo } every month on day { $dia } · { $conta }
carteira-plano-proximo = next on { $data }
carteira-plano-excluir-nome = Delete { $nome }
carteira-plano-excluido = Scheduled contribution deleted
carteira-plano-aviso = { $n } scheduled contribution(s) waiting for your confirmation.
carteira-plano-aviso-ver = See contributions

## Recent position (under 30 days: period return only)
carteira-recente-pill = recent position
carteira-recente-comprada =
    { $dias ->
        [0] Bought today
        [one] Bought 1 day ago
       *[other] Bought { $dias } days ago
    }
carteira-recente-dica = { $comprada }. Return for the period: { $retorno }. The annualized return shows up after 30 days.
carteira-recente-nota = { $comprada }. The annualized return shows up after 30 days.
carteira-recente-no-periodo = in the period
carteira-recente-anualizado = { $retorno } annualized over { $dias } days
carteira-recente-anualizado-real = real { $retorno }
carteira-recente-titulo = Return for the period

## Concentration (the family's limits; a reading of the data, not a recommendation)
carteira-conc-titulo = Concentration
carteira-conc-ajustar = Adjust limits
carteira-conc-botao = Concentration limits
carteira-conc-lista = Concentration alerts
carteira-conc-tipo-ativo = Asset
carteira-conc-tipo-grupo = Group
carteira-conc-tipo-classe = Class
carteira-conc-limite = of the portfolio · limit { $limite }
carteira-conc-acima = { $valor } over the limit
carteira-conc-sev-warning = Over the limit
carteira-conc-sev-critical = Well over the limit
carteira-conc-rodape = Weights at today's value, compared with the limits you chose. This is a reading of your data, not a recommendation to buy or sell.
carteira-conc-pill-dica = Weight over the { $limite } limit for this asset
carteira-conc-dialogo-titulo = Concentration limits
carteira-conc-dialogo-texto = Treasury lets you know when an asset or a group of assets goes over its limit. By default, the limits are 20% per asset and 30% per group; a class uses its allocation target plus the tolerance. Brazilian Treasury bonds and FGC-covered fixed income (up to R$ 250k per asset) are outside the default limit.
carteira-conc-padroes = Defaults
carteira-conc-padrao-ativo = Limit per asset (%)
carteira-conc-padrao-grupo = Limit per group (%)
carteira-conc-por-ativo = By asset
carteira-conc-por-ativo-texto = Leave the limit blank to use the default.
carteira-conc-col-limite = Limit
carteira-conc-limite-de = Limit for { $ativo } (%)
carteira-conc-isento = no limit
carteira-conc-isento-dica = Brazilian Treasury bonds and FGC-covered fixed income (up to R$ 250k per asset) are outside the default limit. Type a limit to apply it to this asset.
carteira-conc-limite-invalido = Use limits above 0% and up to 100%.
carteira-conc-salvar = Save limits
carteira-conc-salvos = Limits saved
carteira-conc-sem-ativos = No positions in the portfolio yet.
carteira-rebal-conc-texto = With no class targets, the split below follows the concentration limits: each asset gets a share of the contribution in proportion to the room it still has under its limit. Nothing is sold.
carteira-rebal-conc-tabela = Contribution by concentration limits
carteira-rebal-conc-espaco = Room under the limit
carteira-rebal-conc-acima = over the limit, no contribution

## Asset labels (concentration groups)
carteira-etiqueta-titulo = Labels
carteira-etiqueta-texto = Labels group assets together (by sector, country or issuer, for example). Mark one as a concentration group to add up the weight of its assets.
carteira-etiqueta-de = Labels for { $ativo }
carteira-etiqueta-tirar = Remove the { $etiqueta } label from { $ativo }
carteira-etiqueta-exemplo = sector, country…
carteira-etiqueta-grupo = Concentration group
carteira-etiqueta-grupo-de = { $etiqueta } is a concentration group
carteira-etiqueta-limite-de = Limit for the { $etiqueta } group (%)
carteira-etiqueta-sem = No labels yet. Type one in an asset's row.
carteira-etiqueta-sem-ativos = no assets in the portfolio
carteira-etiqueta-outros =
    { $n ->
        [one] plus 1 asset outside the portfolio
       *[other] plus { $n } assets outside the portfolio
    }

## Income ledger (features/portfolio/IncomeLedger.tsx)
carteira-fonte-aviso-oficial = Official notice
carteira-fonte-extrato-b3 = B3 statement
carteira-razao-titulo = Income by the official sources
carteira-razao-calculando = Checking the income…
carteira-razao-recebido = Received by the sources
carteira-razao-recebido-desde = Received since { $data }
carteira-razao-recebido-dica = Quantity on the record date × amount of the official notice, net of tax withheld
carteira-razao-a-receber = Upcoming
carteira-razao-lancado = recorded
carteira-razao-automatico-depois = After the review, new official distributions are recorded by themselves on the payment date.
carteira-razao-automatico-ligado = Official distributions are recorded by themselves since { $data }.
carteira-razao-automatico-desligar = Turn off
carteira-razao-revisar = Review and fix
carteira-razao-revisar-titulo = Fix the income from the official notices
carteira-razao-revisar-texto = Each line says what changes. Everything goes in a single change, with Undo. Older distributions are recorded without touching the brokerage cash.
carteira-razao-revisar-tabela = Proposed fixes
carteira-razao-nada = Nothing to fix: the recorded income matches the sources.
carteira-razao-feito = Income fixed: { $lancados } recorded, { $corrigidos } fixed and { $apagados } deleted.
carteira-razao-marcar = { $acao } { $ativo }
carteira-razao-pago-em = paid on { $data }
carteira-razao-aplicar =
    { $n ->
        [one] Apply { $n } fix
       *[other] Apply { $n } fixes
    }
carteira-razao-acao-lancar = Record
carteira-razao-acao-corrigir = Fix amount
carteira-razao-acao-apagar = Delete repeated
carteira-razao-estado-confere = Matches
carteira-razao-estado-faltando = Not recorded
carteira-razao-estado-a-receber = Upcoming
carteira-razao-estado-valor = Different amount
carteira-razao-estado-ir = Different tax
carteira-razao-estado-repetido = Repeated
carteira-razao-estado-sem-fonte = No source
carteira-razao-estado-sem-credito = No credit in the account
carteira-razao-sem-credito = { $n ->
    [one] 1 distribution received without the credit in the account
   *[other] { $n } distributions received without the credit in the account
}
carteira-razao-sem-credito-texto = The distribution is recorded, but the money did not reach the brokerage balance here in the app. Check your brokerage statement: if the credit is there, record it. If the statement you import already brings that credit, there is nothing to do.
carteira-razao-lancar-credito = Record the credit
carteira-razao-credito-lancado = { $n ->
    [one] Credit of 1 distribution recorded in the account
   *[other] Credit of { $n } distributions recorded in the account
}
carteira-razao-tabela = Income, one per payment
carteira-razao-ativos-tabela = Income by asset since the purchase
carteira-razao-col-data-com = Record date
carteira-razao-col-tipo = Kind
carteira-razao-col-qtd-valor = Qty × amount
carteira-razao-col-bruto = Gross
carteira-razao-col-ir = Tax
carteira-razao-col-liquido = Net
carteira-razao-col-fonte = Source
carteira-razao-col-situacao = Status
carteira-razao-col-recebido = Received
carteira-razao-col-lancado = Recorded
carteira-razao-col-a-receber = Upcoming
carteira-razao-col-desde = Since
carteira-razao-col-acao = What changes
carteira-razao-col-fica = Becomes
carteira-razao-por-ano = Received by year
carteira-razao-conferir =
    { $n ->
        [one] { $n } distribution to check
       *[other] { $n } distributions to check
    }
carteira-razao-parte-faltando = { $n } not recorded
carteira-razao-parte-repetidos = { $n } recorded twice
carteira-razao-parte-valor = { $n } with an amount or tax different from the official notice
carteira-razao-parte-sem-aviso = { $n } without an official notice (check your statement)
carteira-razao-na-carteira = Today the portfolio records { $valor } of this income.
carteira-razao-sem-aviso-tag = no official notice ({ $fonte }): check your statement
carteira-razao-visao = View
carteira-razao-ver-todos = All ({ $n })
carteira-razao-ver-conferir = To check ({ $n })
carteira-razao-ver-receber = Upcoming ({ $n })
carteira-razao-ver-lancamentos = Records
carteira-razao-ver-ativos = By asset
carteira-razao-ver-meses = By month
carteira-razao-vazio = Nothing here.
carteira-razao-col-conferir = To check
