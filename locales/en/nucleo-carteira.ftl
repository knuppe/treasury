# Portfolio and physical assets (core, `portfolio` and `physical`): errors, history reasons,
# descriptions of the transactions the app creates, warnings and notes about positions,
# dividends, fee cost, risk, rebalancing, the asset statement, the agent's batch import, quickly
# typed dividends and the formula of physical asset values (net worth "Explain"). Errors in
# lowercase, like a log line (the interface capitalizes them). Official and market terms (JCP,
# FII, CVM, B3, FIPE, IPCA, IGP-M, INCC, BCB SGS, CDI, TWR, come-cotas) stay as they are.
# Revision: AI draft, not yet reviewed by a native speaker.

## Fields (used in "invalid <field>: <value>")

nucleo-carteira-erro-campo-invalido = invalid { $campo }: { $valor }
nucleo-carteira-campo-quantidade = quantity
nucleo-carteira-campo-preco = price
nucleo-carteira-campo-cambio = exchange rate
nucleo-carteira-campo-fator = factor
nucleo-carteira-campo-custo = cost
nucleo-carteira-campo-alvo = target
nucleo-carteira-campo-tolerancia = tolerance
nucleo-carteira-campo-preco-alvo = target price
nucleo-carteira-campo-alerta = alert
nucleo-carteira-campo-valor-cota = amount per share

## Dividend kinds

nucleo-carteira-provento-dividendos = Dividends
nucleo-carteira-provento-jcp = JCP
nucleo-carteira-provento-fii = FII income
nucleo-carteira-provento-cupom = Coupon
nucleo-carteira-provento-juros = Interest
nucleo-carteira-provento-aluguel = Stock lending
nucleo-carteira-provento-amortizacao = Amortization
nucleo-carteira-provento-outros = Other payouts
# Description of a dividend transaction ("Dividends PETR4").
nucleo-carteira-provento-lancamento = { $tipo } { $ativo }

## Trades, corporate actions, dividends, targets and watchlist (`records`)

nucleo-carteira-operacao = trade { $id }
# Description of a trade transaction ("Buy 10 PETR4 @ R$ 38.50").
nucleo-carteira-operacao-compra = Buy { $quantidade } { $ativo } @ { $preco }
nucleo-carteira-operacao-venda = Sell { $quantidade } { $ativo } @ { $preco }
nucleo-carteira-erro-moedas = the account is in { $conta } and the asset in { $ativo }: enter the exchange rate or turn off the cash movement
nucleo-carteira-erro-quantidade = the quantity must be positive
nucleo-carteira-erro-preco = the price can't be negative
nucleo-carteira-erro-custos = costs and taxes can't be negative
nucleo-carteira-erro-cambio = the exchange rate must be positive
nucleo-carteira-erro-venda-maior = sale of { $quantidade } is larger than the position of { $posicao } in this account on that date
nucleo-carteira-erro-ajuste-com-caixa = a position adjustment does not move cash: remove the cash entry of the trade (cash_movement false)
nucleo-carteira-erro-ajuste-importada = this trade came from the brokerage with its own identifier: it is a real trade, not a position adjustment
nucleo-carteira-erro-desdobramento = a split needs a factor greater than 1 (e.g. 2 for 1→2)
nucleo-carteira-erro-grupamento = a reverse split needs a factor between 0 and 1 (e.g. 0.1 for 10→1)
nucleo-carteira-erro-bonificacao = a stock bonus needs the factor (new shares per share, e.g. 0.1 = 10%)
nucleo-carteira-erro-bonificacao-quantidade = only a bonus can be credited by quantity, in one account and without a factor
nucleo-carteira-erro-devolucao = a capital return needs the total amount
nucleo-carteira-erro-ativo-resultante = enter the resulting asset
nucleo-carteira-erro-bruto = the dividend's gross amount must be positive
nucleo-carteira-erro-imposto-retido = invalid withheld tax
nucleo-carteira-erro-alvos-escopo = all targets must have the same scope
nucleo-carteira-erro-alvo = the target must be between 0% and 100%
nucleo-carteira-erro-alvos-soma = the class targets add up to more than 100%
nucleo-carteira-erro-alvos-soma-classe = the targets of the { $classe } assets add up to more than 100% of the class
nucleo-carteira-watchlist-acima = price above { $valor }
nucleo-carteira-watchlist-abaixo = price below { $valor }

## Commands (`portfolio.*`): history reasons and answers

nucleo-carteira-erro-nada-selecionado = no record selected
nucleo-carteira-nada-novo = Nothing new: everything was already registered or was skipped (see the report).
nucleo-carteira-nenhuma-linha-pronta = No line ready: all already recorded or with errors (see the report).
nucleo-carteira-motivo-operacao-registrada = Trade recorded
nucleo-carteira-motivo-operacao-alterada = Trade changed
nucleo-carteira-motivo-operacoes-removidas = { $n ->
        [one] Trade removed
       *[other] { $n } trades removed
    }
nucleo-carteira-motivo-evento-registrado = Corporate action recorded
nucleo-carteira-motivo-eventos-removidos = { $n ->
        [one] Corporate action removed
       *[other] { $n } corporate actions removed
    }
nucleo-carteira-motivo-provento-registrado = Dividend recorded
nucleo-carteira-erro-sem-proventos = no dividend to record
nucleo-carteira-erro-proventos-demais = at most 500 dividends at a time
nucleo-carteira-motivo-proventos-confirmados = { $n ->
        [one] { $n } dividend confirmed
       *[other] { $n } dividends confirmed
    }
nucleo-carteira-motivo-proventos-removidos = { $n ->
        [one] Dividend removed
       *[other] { $n } dividends removed
    }
nucleo-carteira-motivo-come-cotas = Come-cotas rule
nucleo-carteira-motivo-alvos = Target allocation set
nucleo-carteira-motivo-watchlist = Asset on the watchlist
nucleo-carteira-motivo-watchlist-removido = Asset removed from the watchlist

## Agent's batch import (`bulk`)

nucleo-carteira-erro-lote-fonte = say where the data came from (`source`)
nucleo-carteira-erro-lote-vazio = nothing to register: enter assets, trades, positions or dividends
nucleo-carteira-erro-lote-itens = at most { $maximo } items per call; split it into parts
nucleo-carteira-erro-lote-conta = trades, positions and dividends need the account (`account_id` or `account`)
nucleo-carteira-erro-conta-cartao = the investment account can't be a credit card
# History reason: "Investments in batch (2 assets, 5 trades): XP statement".
nucleo-carteira-motivo-lote = Investments in batch ({ $itens }): { $fonte }
nucleo-carteira-lote-ativos = { $n ->
        [one] { $n } asset
       *[other] { $n } assets
    }
nucleo-carteira-lote-operacoes = { $n ->
        [one] { $n } trade
       *[other] { $n } trades
    }
nucleo-carteira-lote-posicoes = { $n ->
        [one] { $n } position
       *[other] { $n } positions
    }
nucleo-carteira-lote-proventos = { $n ->
        [one] { $n } dividend
       *[other] { $n } dividends
    }
nucleo-carteira-lote-conta = account
# The account in the batch report.
nucleo-carteira-lote-conta-existente-id = existing { $id }
nucleo-carteira-lote-conta-existente = existing: { $nome }
nucleo-carteira-lote-conta-nova = new: { $nome }
# Skipped items in the report ("trade 3: already recorded").
nucleo-carteira-lote-ativo-erro = asset { $n } ({ $ativo }): { $erro }
nucleo-carteira-lote-operacao-sem-ativo = trade { $n }: asset { $ativo } not registered (include it in `assets`)
nucleo-carteira-lote-operacao-repetida = trade { $n }: already recorded
nucleo-carteira-lote-operacao-erro = trade { $n }: { $erro }
nucleo-carteira-lote-posicao-sem-ativo = position { $n }: asset { $ativo } not registered (include it in `assets`)
nucleo-carteira-lote-posicao-quantidade = position { $n }: quantity must be positive
nucleo-carteira-lote-posicao-erro = position { $n }: { $erro }
nucleo-carteira-lote-posicao-confere = position { $n } ({ $ativo }): already matches
nucleo-carteira-lote-posicao-maior = position { $n } ({ $ativo }): { $registrado } already recorded, more than { $informado }; record the sale with `record_trade` (`side: sell`; `cash_movement: false` if the money is already in the account)
nucleo-carteira-lote-posicao-repetida = position { $n }: already recorded
nucleo-carteira-lote-provento-sem-ativo = dividend { $n }: asset { $ativo } not registered (include it in `assets`)
nucleo-carteira-lote-provento-repetido = dividend { $n }: already recorded
nucleo-carteira-lote-provento-erro = dividend { $n }: { $erro }
# Note saved on the batch's trades. The beginning of each one (before the first argument)
# marks the trade as the agent's in the asset statement, in any language.
nucleo-carteira-nota-lote = Agent batch: { $fonte }
nucleo-carteira-nota-posicao = Position reported on { $data } ({ $fonte }): adjustment of { $diferenca } at the average price

## Quickly typed dividends (`quick_income`)

nucleo-carteira-rapido-erro-ticker = enter the ticker
nucleo-carteira-rapido-erro-valor = the amount must be positive
nucleo-carteira-rapido-erro-ativo = { $ticker } isn't registered in the portfolio
nucleo-carteira-rapido-erro-conta = account { $conta } not found
nucleo-carteira-rapido-erro-sem-operacoes = no trades of { $ticker } recorded: choose the account
nucleo-carteira-rapido-erro-sem-posicao = no position in { $ticker } on { $data }: choose the account
nucleo-carteira-rapido-ja-registrado = already recorded
nucleo-carteira-rapido-repetido = repeats a line above
nucleo-carteira-rapido-erro-sem-linhas = no dividend lines
nucleo-carteira-rapido-erro-linhas = at most { $maximo } lines at a time (received { $recebidas })
nucleo-carteira-rapido-nota-sem-posicao = no position in { $ticker } on the date
nucleo-carteira-rapido-nota-maior-posicao = asset in more than one account: the one with the largest position was chosen
nucleo-carteira-rapido-nota-conta = account from the position that explains the amount: { $conta }
# "Dividends of 0.5 per share (B3, record date Oct 10, 2026, paid on Oct 20, 2026)".
nucleo-carteira-rapido-nota-anunciado = { $tipo } of { $valor } per share ({ $fonte }, paid on { $pagamento })
nucleo-carteira-rapido-nota-anunciado-com = { $tipo } of { $valor } per share ({ $fonte }, record date { $datacom }, paid on { $pagamento })
nucleo-carteira-rapido-nota-confere = matches: { $quantidade } × { $valor } = { $total }
nucleo-carteira-rapido-nota-diferente = amount differs from the expected one ({ $esperado } gross for { $quantidade } shares): check it
nucleo-carteira-rapido-nota-bruto = amount typed treated as gross (before IR)
nucleo-carteira-rapido-nota-sem-anuncio = no announced dividend near { $data }: kind from the asset class ({ $tipo })
nucleo-carteira-rapido-nota-ir = presumed withheld IR: { $imposto } (gross { $bruto })
nucleo-carteira-rapido-nota-futuro = future payment: stays as announced, no cash
# History reason: "3 dividends recorded: PETR4, VALE3, ITUB4".
nucleo-carteira-rapido-motivo = { $n ->
        [one] { $n } dividend recorded: { $lista }
       *[other] { $n } dividends recorded: { $lista }
    }
nucleo-carteira-rapido-e-mais = { $lista } and { $n } more

## Announced dividends to check (`pending`)

nucleo-carteira-pendente-data-agenda = payment date from the calendar (the official source hasn't announced it yet)
nucleo-carteira-pendente-valor-divergente = { $fonte } said { $valor } per share; the official amount was used ({ $b3 })
nucleo-carteira-pendente-data-divergente = { $fonte } said payment on { $data }; the official date was used ({ $b3 })
nucleo-carteira-pendente-tipos-divergentes = the sources give different kinds ({ $lista }); the one from { $fonte } was used
nucleo-carteira-pendente-fontes-discordam = the dividend isn't in the official sources and the sources disagree ({ $lista }); the one from { $fonte } was used: check your statement
nucleo-carteira-pendente-so-uma-fonte = only on { $fonte }: no official notice of this dividend (not downloaded yet or not on the official list)

## Positions, performance and rebalancing (`engine`)

nucleo-carteira-erro-cadeia-ticker = chain of ticker changes too long (or circular)
nucleo-carteira-aviso-sem-cambio = no { $moeda }/{ $base } exchange rate on { $data }
nucleo-carteira-aviso-venda-maior = sale larger than the position on { $data }
nucleo-carteira-aviso-cisao = spin-off/subscription: record the resulting trade manually
nucleo-carteira-aviso-sem-cotacao = no price: valued at cost
nucleo-carteira-aviso-aportes-sem-cota = { $valor } in pension contributions still without a unit price: valued at the amount paid
nucleo-carteira-aviso-cotacao-parada = no new price since { $data } and no trading: the asset may have been delisted or merged; check before recording
nucleo-carteira-erro-periodo-longo = period too long
# Name of the comparison series outside Brazil, when the country has no name for it.
nucleo-carteira-juro-referencia = Reference rate
nucleo-carteira-rebalanceamento-aviso = Analysis based on your data and the targets you set; this is not investment advice.

## Come-cotas (`come_cotas`, the twice-yearly tax on Brazilian funds)

nucleo-carteira-come-cotas-so-fundos = only open-ended funds have come-cotas
nucleo-carteira-come-cotas-escolhida = rule chosen by you
nucleo-carteira-come-cotas-cvm-isento = CVM classification: { $classe } (no come-cotas)
nucleo-carteira-come-cotas-cvm-longo = CVM classification: { $classe }; long term (15%) until you say it's short term
nucleo-carteira-come-cotas-sem-classificacao = fund without a classification downloaded from the CVM: long term (15%); adjust it if it's a stock or short-term fund
nucleo-carteira-come-cotas-sem-cota = no share value on { $data }: come-cotas for that date not applied
nucleo-carteira-come-cotas-erro-valor = invalid come-cotas value: { $valor }

## Fee cost (`fee_cost`)

nucleo-carteira-custo-erro-parametros = real return between −50% and 50%, alternative fee between 0 and 20% and excess between 0 and 50%
nucleo-carteira-custo-erro-horizontes = horizons from 1 to 60 years
nucleo-carteira-custo-fonte-previdencia = pension plan registration
nucleo-carteira-custo-fonte-cvm = the fund's CVM statement
nucleo-carteira-custo-fonte-cvm-data = the fund's CVM statement ({ $data })
nucleo-carteira-custo-nota-retorno = Real values in today's money, with a return of { $retorno } a year above inflation before fees.
nucleo-carteira-custo-nota-alternativa = Alternative scenario: management fee of { $taxa } a year (or today's, if already lower), with no performance or load fee.
nucleo-carteira-custo-nota-performance = Performance fee with no projected cost: it only matters if the fund beats the benchmark (enter the expected excess).

## Portfolio risk (`risk`)

nucleo-carteira-risco-erro-meses = analyze from 12 to 120 months
nucleo-carteira-risco-nota-retornos = Monthly returns from { $de } to { $ate } (TWR, with dividends, in { $moeda }). The portfolio and the classes use today's weights; in a month when an asset didn't exist, its weight goes to the others.
nucleo-carteira-risco-nota-pouco-historico = Less than { $minimo } months of history (no metrics of their own or correlation): { $ativos }.
nucleo-carteira-risco-nota-sem-cdi = No CDI series in the period: the Sharpe ratio is left out (Settings › Indexes).
nucleo-carteira-risco-nota-estimados = There are estimated or missing prices in the period: the numbers are approximate.

## Asset statement (`statement`)

nucleo-carteira-extrato-desdobramento = Split { $proporcao }
nucleo-carteira-extrato-grupamento = Reverse split { $proporcao }
nucleo-carteira-extrato-bonificacao-pct = Stock bonus of { $pct }%
nucleo-carteira-extrato-bonificacao = Stock bonus
nucleo-carteira-extrato-subscricao = Subscription
nucleo-carteira-extrato-incorporacao = Merger
nucleo-carteira-extrato-troca-ticker = Ticker change
nucleo-carteira-extrato-cisao = Spin-off
nucleo-carteira-extrato-devolucao = Capital return
nucleo-carteira-extrato-herdada = Inherited position (ticker change or merger)
nucleo-carteira-extrato-entrada-sem-custo = Zero-cost entry (split or stock bonus recorded as a buy)
nucleo-carteira-extrato-compra = Buy
nucleo-carteira-extrato-venda = Sell
nucleo-carteira-extrato-ajuste = Position adjustment
nucleo-carteira-extrato-come-cotas =
    { $cotas ->
        [one] Come-cotas: { $cotas } share redeemed for IR
       *[other] Come-cotas: { $cotas } shares redeemed for IR
    }
# Where the statement line came from.
nucleo-carteira-extrato-origem-nota = note { $numero }
nucleo-carteira-extrato-origem-agente = agent
nucleo-carteira-extrato-origem-importacao = import
nucleo-carteira-extrato-origem-manual = manual
nucleo-carteira-extrato-origem-sistema = system
nucleo-carteira-extrato-origem-yahoo = Yahoo download
nucleo-carteira-extrato-origem-coleta = price download
nucleo-carteira-extrato-origem-b3 = B3 import
nucleo-carteira-extrato-aviso-desdobramento-duplo = possible split counted twice: there's a zero-cost entry and a corporate action a few days apart; delete one of them
nucleo-carteira-extrato-aviso-bonificacao-dupla = possible stock bonus counted twice: there's a corporate action and a buy with the same number of new shares a few days apart; delete one of them

## Physical assets (`physical`): registration, improvements, appraisals and financing

nucleo-carteira-bem = asset { $id }
nucleo-carteira-bem-benfeitoria = improvement { $id }
nucleo-carteira-bem-avaliacao = appraisal { $id }
nucleo-carteira-bem-ligacao = link between the loan and the asset
nucleo-carteira-bem-campo-taxa-depreciacao = depreciation rate
nucleo-carteira-bem-campo-valor-residual = residual value
nucleo-carteira-bem-campo-taxa = rate
nucleo-carteira-bem-campo-residual = residual
nucleo-carteira-bem-erro-fora-intervalo = { $campo } out of range
nucleo-carteira-bem-erro-nome = give the asset a name
nucleo-carteira-bem-erro-preco = the purchase price can't be negative
nucleo-carteira-bem-erro-vida-util = useful life between 1 and 100 years
nucleo-carteira-bem-erro-fipe-dados = to use the FIPE table, enter the vehicle's type, make, model and year
nucleo-carteira-bem-erro-fipe-ref = enter the vehicle's type, make, model and year from the FIPE table
nucleo-carteira-bem-erro-venda-antes = the sale can't be before the purchase
nucleo-carteira-bem-erro-venda-negativa = sale amounts can't be negative
nucleo-carteira-bem-erro-custos = acquisition costs can't be negative
nucleo-carteira-bem-erro-benfeitoria = describe the improvement or the work
nucleo-carteira-bem-erro-valor-obra = the amount of the work must be positive
nucleo-carteira-bem-erro-valor-projeto = enter the (estimated) amount of the work or link transactions to the project
nucleo-carteira-bem-erro-avaliacao = the appraisal amount can't be negative
nucleo-carteira-bem-erro-contrato-outro = this loan already finances another asset ({ $bem }); unlink it from there first
nucleo-carteira-bem-motivo-cadastrado = Asset added: { $nome }
nucleo-carteira-bem-motivo-alterado = Asset changed
nucleo-carteira-bem-motivo-removido = Asset removed
nucleo-carteira-bem-motivo-benfeitoria = Improvement recorded: { $descricao }
nucleo-carteira-bem-motivo-benfeitoria-alterada = Improvement changed
nucleo-carteira-bem-motivo-benfeitoria-removida = Improvement removed
nucleo-carteira-bem-motivo-avaliacao = Appraisal recorded
nucleo-carteira-bem-motivo-avaliacao-removida = Appraisal removed
nucleo-carteira-bem-motivo-financiamento-ligado = Loan linked to the asset
nucleo-carteira-bem-motivo-financiamento-desligado = Loan unlinked from the asset
nucleo-carteira-bem-motivo-venda = Asset sale recorded

## Physical assets: FIPE table (`physical::fipe`)

nucleo-carteira-fipe-erro-codigo = invalid FIPE code: { $codigo }
nucleo-carteira-fipe-erro-ilegivel = unreadable FIPE table response
nucleo-carteira-fipe-erro-sem-modelos = FIPE table response without models
nucleo-carteira-fipe-erro-inesperada = unexpected FIPE table response
nucleo-carteira-fipe-erro-sem-codigo = FIPE table option without a code
nucleo-carteira-fipe-erro = FIPE table: { $erro }
nucleo-carteira-fipe-erro-sem-preco = FIPE table response without a price
nucleo-carteira-fipe-erro-rede = couldn't query the FIPE table ({ $erro }); try again later or record the appraisal manually
nucleo-carteira-fipe-erro-codigo-diferente = the vehicle's FIPE code ({ $esperado }) doesn't match the one from the query ({ $consulta })
nucleo-carteira-fipe-motivo = FIPE price for { $mes }

## Physical assets: value on a date and the net worth "Explain" formula (`physical::valuation`)

# Source of a recorded appraisal.
nucleo-carteira-bem-fonte-manual = own estimate
nucleo-carteira-bem-fonte-avaliacao = appraisal
nucleo-carteira-bem-fonte-laudo = appraisal report
nucleo-carteira-bem-fonte-fipe = FIPE
nucleo-carteira-bem-fonte-indice = index
# Warnings about the adjustment index (the month comes as MM/YYYY).
nucleo-carteira-bem-indice-sem-dados = no index data: sync the indexes
nucleo-carteira-bem-indice-publicado-ate = index published up to { $mes }; the following period isn't adjusted
nucleo-carteira-bem-indice-desde = index available since { $mes }
nucleo-carteira-bem-indice-faltando = index series with missing months: value not adjusted
nucleo-carteira-bem-vida-encerrada = useful life already over at the appraisal
# Parts of the formula: "appraisal of Jan 10, 2025 (appraisal report): R$ 500,000.00 adjusted by IPCA from … = R$ 520,000.00".
nucleo-carteira-bem-formula-indice = adjusted by { $indice } from { $de } to { $ate } (factor { $fator })
nucleo-carteira-bem-formula-depreciacao = depreciated at { $taxa }% a year for { $anos } years (factor { $fator })
nucleo-carteira-bem-formula-linear = depreciated straight-line over { $anos ->
        [one] { $anos } year
       *[other] { $anos } years
    } down to { $residual }% (factor { $fator })
nucleo-carteira-bem-formula-comprado = only bought on { $data }
nucleo-carteira-bem-formula-vendido = sold on { $data } for { $valor } (net)
nucleo-carteira-bem-formula-avaliacao = appraisal of { $data } ({ $fonte }): { $valor }
nucleo-carteira-bem-formula-custo = acquisition cost of { $valor } on { $data }
nucleo-carteira-bem-benfeitorias = { $n ->
        [one] 1 improvement
       *[other] { $n } improvements
    }
nucleo-carteira-bem-formula-benfeitorias-custo = { $benfeitorias } at cost ({ $valor })
nucleo-carteira-bem-formula-benfeitorias-regra = { $benfeitorias } of { $custo } by the same rule ({ $valor })
nucleo-carteira-bem-formula-sem-avaliacao = no appraisal or automatic estimate: { $formula }; record an appraisal for the market value
# Short source of the value.
nucleo-carteira-bem-fonte-sem-estimativa = no automatic estimate
nucleo-carteira-bem-fonte-indice-bcb = { $indice } (BCB SGS)
nucleo-carteira-bem-fonte-fipe-depreciacao = latest FIPE price + depreciation of { $taxa }% a year
nucleo-carteira-bem-fonte-depreciacao = depreciation of { $taxa }% a year
nucleo-carteira-bem-fonte-linear = straight-line depreciation over { $anos ->
        [one] { $anos } year
       *[other] { $anos } years
    }
nucleo-carteira-bem-fonte-sem-posse = not owned
nucleo-carteira-bem-fonte-venda = sale
nucleo-carteira-bem-fonte-avaliacao-de = appraisal · { $fonte }
nucleo-carteira-bem-fonte-custo = acquisition cost
nucleo-carteira-bem-fonte-avaliacao-regra = appraisal · { $fonte } + { $regra }

## Fixed income calculators (portfolio.compare_fixed_income and simulate_fixed_income_sale)
nucleo-rf-erro-valor = enter the amount invested
nucleo-rf-erro-ofertas = enter 1 to { $max } offers
nucleo-rf-erro-prazo = enter a term in months or a redemption date after the investment date
nucleo-rf-erro-taxa = invalid rate
nucleo-rf-erro-datas-venda = the sale must fall between the investment date and maturity
nucleo-rf-tipo-cdb = CDB
nucleo-rf-tipo-rdb = RDB
nucleo-rf-tipo-lc = LC
nucleo-rf-tipo-lci = LCI
nucleo-rf-tipo-lca = LCA
nucleo-rf-tipo-cri = CRI
nucleo-rf-tipo-cra = CRA
nucleo-rf-tipo-debenture = Debenture
nucleo-rf-tipo-debenture-incentivada = Incentivized debenture
nucleo-rf-tipo-tesouro-selic = Tesouro Selic
nucleo-rf-tipo-tesouro-prefixado = Tesouro Prefixado
nucleo-rf-tipo-tesouro-ipca = Tesouro IPCA+
nucleo-rf-tipo-tesouro-renda = Tesouro Renda+
nucleo-rf-tipo-poupanca = Savings account (poupança)
nucleo-rf-tipo-outro = Fixed income
nucleo-rf-taxa-cdi-pct = { $taxa }% of CDI
nucleo-rf-taxa-cdi-mais = CDI + { $taxa }%
nucleo-rf-taxa-selic-mais = Selic + { $taxa }%
nucleo-rf-taxa-pre = { $taxa }% p.a.
nucleo-rf-taxa-ipca-mais = IPCA + { $taxa }%
nucleo-rf-nota-iof = Redeemed before 30 days: IOF is charged on the income.
nucleo-rf-nota-aniversario = Poupança only earns on its monthly anniversary: redeeming before it loses the current month.
nucleo-rf-nota-custodia = Includes the B3 custody fee (0.20% p.a.; on Tesouro Selic, only above R$ 10,000).
nucleo-rf-nota-venda = Selling before maturity gets the day's market price, which can be above or below the curve.
nucleo-rf-nota-venda-ipca = IPCA up to the sale and up to maturity is the assumption you entered, not the published index.
nucleo-rf-fonte-ir = Regressive income tax (22.5% to 15%) and the exemption for LCI, LCA, CRI, CRA and poupança: Law 11,033/2004; incentivized debentures: Law 12,431/2011.
nucleo-rf-fonte-iof = Regressive IOF on income up to day 29: Decree 6,306/2007.
nucleo-rf-fonte-custodia = Tesouro Direto custody: B3 charges 0.20% p.a.; no fee on Tesouro Selic up to R$ 10,000.
nucleo-rf-fonte-poupanca = Poupança: 0.5% a month + TR when the Selic is above 8.5% p.a.; otherwise 70% of the Selic + TR (Law 8,177/1991).
nucleo-rf-aviso = An estimate under the assumptions shown, not a promise of returns or an investment recommendation.
nucleo-carteira-erro-ponte-periodo = the end of the period must come after the start

## Compare with any series (`portfolio.benchmark_series`)

nucleo-carteira-comparar-serie-desconhecida = unknown index series
nucleo-carteira-comparar-sem-juro = the reference country has no reference rate in the catalog
nucleo-carteira-comparar-cdi-mais = { $indice } + { $taxa } a year

nucleo-carteira-motivo-metodo-custo = Cost method for sales
nucleo-carteira-motivo-venda-sem-compra = Sale without a recorded purchase
nucleo-carteira-venda-sem-compra-nao-encontrada = sale

## Aportes programados

nucleo-carteira-plano = scheduled contribution { $id }
nucleo-carteira-plano-erro-nome = enter a name for the scheduled contribution
nucleo-carteira-plano-erro-valor = the contribution amount must be positive
nucleo-carteira-plano-erro-dia = the contribution day must be between 1 and 28
nucleo-carteira-plano-erro-fim = the end of the scheduled contribution can't come before the start
nucleo-carteira-plano-erro-data = this date is not a pending contribution of this plan
nucleo-carteira-plano-nota = Scheduled contribution: { $plano }
nucleo-carteira-motivo-plano-criado = Scheduled contribution created
nucleo-carteira-motivo-plano-alterado = Scheduled contribution changed
nucleo-carteira-motivo-plano-removido = Scheduled contribution removed
nucleo-carteira-motivo-aporte-confirmado = Scheduled contribution confirmed
nucleo-carteira-motivo-aporte-pulado = Scheduled contribution skipped

## Asset labels and concentration (V161)
nucleo-ativos-etiqueta-invalida = a label must have 1 to 40 characters (at most 20 per asset)
nucleo-ativos-motivo-etiquetas = Asset labels
nucleo-carteira-limite-invalido = the limit must be between 0 and 100% of the portfolio
nucleo-carteira-limite-sem-ativo = give the asset for the limit
nucleo-carteira-motivo-limite = Concentration limit

## Income ledger (portfolio/income_ledger.rs)
nucleo-carteira-razao-nota-creditado = amount credited by B3 in the imported statement: it prevails over the calculation
nucleo-carteira-razao-nota-pagamento-estimado = no source gave the payment date: the record date was used
nucleo-carteira-razao-nota-repete = repeats the distribution paid on { $data }
nucleo-carteira-razao-nota-data-improvavel = payment date far in the future: check the year
nucleo-carteira-razao-nota-sem-fonte = no source describes this distribution: check your brokerage statement
nucleo-carteira-razao-nota-sem-caixa = recorded as received, without the credit in the account: check the brokerage statement and record the credit
nucleo-carteira-razao-nota-historico-sem-caixa = history: recorded without moving the brokerage cash
nucleo-carteira-razao-erro-caixa-ja-lancado = this distribution already has its credit in the account
nucleo-carteira-razao-erro-caixa-anunciado = distribution not received yet: there is no credit to record
nucleo-carteira-razao-motivo-caixa = Credit of { $n } distribution(s) recorded in the account
nucleo-carteira-razao-motivo-reconstruir =
    { $n ->
        [one] Distributions rebuilt from the official notices ({ $n } fix)
       *[other] Distributions rebuilt from the official notices ({ $n } fixes)
    }
nucleo-carteira-razao-motivo-automatico =
    { $n ->
        [one] Official distribution recorded automatically ({ $n })
       *[other] Official distributions recorded automatically ({ $n })
    }
nucleo-carteira-razao-erro-so-usuario = Only you can turn the automatic recording of distributions on or off.
