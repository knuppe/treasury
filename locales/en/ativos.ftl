# 09 · Assets: tabs, list, registration, asset page, watchlist, dividend calendar, income to confirm
# and private pension (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Screen and tabs

ativos-cadastrar = Add asset
ativos-secoes = Asset sections
ativos-aba-watchlist = Watchlist
ativos-aba-agenda = Dividend calendar
ativos-aba-indices = Indices and quotes
ativos-aba-indicadores = Indicators
ativos-aba-rankings = Rankings
ativos-aba-screener = Screener
ativos-aba-comparador = Compare
ativos-aba-dados = Analysis data
ativos-aba-bolsas = Exchanges

## Asset context menu

ativos-menu-abrir = Open asset
ativos-menu-nova-operacao = New trade
ativos-menu-registrar-provento = Record income
ativos-menu-evento-corporativo = Corporate action
ativos-menu-ver-extrato = View statement
ativos-menu-operacoes = Trades
ativos-menu-proventos = Income
ativos-menu-grafico = Price chart
ativos-menu-fundamentos = Fundamentals
ativos-menu-acompanhar = Add to watchlist…
ativos-menu-informar-cotacao = Enter quote…
ativos-menu-comparar = Compare with others
ativos-menu-editar = Edit details…

## Asset list

ativos-score-classe-sem-score = No score for this class yet
ativos-score-falta = Not enough data: missing { $falta }
ativos-score-parcial = Partial score (over 30% of the weight estimated) · confidence { $pct }%
ativos-score-faixa = { $faixa } · confidence { $pct }%
ativos-score-nota = Score { $nota } out of 100: { $detalhe }
ativos-score-sem-nota = Score: { $detalhe }
ativos-score-sem-dados = no data
ativos-col-score = Score
ativos-col-ticker = Ticker
ativos-col-nome = Name
ativos-nome-oficial = Official name
ativos-nome-oficial-dica = Optional. The name in the registry (CVM, exchange, insurer), kept and searchable; the app shows the name above.
ativos-nome-oficial-de = Official name: { $nome }
ativos-col-classe = Class
ativos-col-mercado = Market
ativos-col-moeda = Currency
ativos-col-setor-termos = Sector · terms
ativos-col-cotacao = Quote
ativos-arquivado = archived
ativos-buscar-rotulo = Search assets by ticker or name
ativos-buscar = Search by ticker or name…
ativos-filtrar-classe = Filter by class
ativos-todas-classes = All classes
ativos-filtrar-posicao = Filter by position
ativos-posicao-aberta = In portfolio
ativos-posicoes-encerradas = Closed positions
ativos-posicao-com-operacoes = With trades (open and closed)
ativos-posicao-todos = All registered
ativos-incluir-arquivados = Include archived
ativos-n-ativos = { $n ->
    [one] { $n } asset
   *[other] { $n } assets
}
ativos-vazio-titulo = No assets yet
ativos-vazio-descricao = Add stocks, REITs, ETFs, Brazilian treasury bonds, CDBs, foreign assets or crypto to record trades and track them.
ativos-tabela = Registered assets
ativos-nenhum-filtro = No assets match these filters.
ativos-acoes-de = Actions for { $nome }

## Asset page: manual quote

ativos-cotacao-titulo = Enter quote · { $nome }
ativos-cotacao-descricao = For assets without an automatic source (funds, private pension, private bonds). It is marked as “Entered by you”.
ativos-cotacao-informada = Quote saved
ativos-cotacao-informada-em = { $preco } on { $data }
ativos-cotacao-manual = Manual quote
ativos-data = Date
ativos-informe-data = Enter the date.
ativos-fechamento-moeda = Close ({ $moeda })
ativos-informe-preco = Enter the price.
ativos-salvar-cotacao = Save quote

## Asset page: fixed income terms

ativos-termos-titulo = Fixed income terms
ativos-termos-resumo = Summary
ativos-termos-tipo = Type
ativos-termos-indexador = Index
ativos-termos-taxa = Rate
ativos-termos-emissao = Issue date
ativos-termos-vencimento = Maturity
ativos-termos-carencia = Lock-up
ativos-termos-liquidez = Liquidity
ativos-termos-ir = Income tax
ativos-termos-isento = Exempt
ativos-termos-regressiva = Regressive table
ativos-termos-emissor = Issuer

## Asset page: header and tabs

ativos-carregando-ativo = Loading asset
ativos-nao-encontrado = Asset not found
ativos-sem-mercado = no market
ativos-informar-cotacao = Enter quote
ativos-acompanhar = Watch
ativos-evento = Corporate action
ativos-provento = Income
ativos-secoes-ativo = Asset sections
ativos-aba-resumo = Overview
ativos-aba-previdencia = Pension
ativos-aba-extrato = Statement
ativos-aba-eventos = Corporate actions
ativos-aba-grafico = Chart
ativos-aba-saude = Health
ativos-aba-cadastro = Details

## Asset page: summary

ativos-posicao = Position
ativos-desde = Since
ativos-custo = Cost
ativos-quantidade = Quantity
ativos-preco-medio = Average price
ativos-valor = Value
ativos-liquido-estimado = Estimated net (taxes)
ativos-bruto-curva = Gross on the curve
ativos-pelo-custo = At cost (no quote)
ativos-valor-mercado = Market value
ativos-peso = weight { $peso }
ativos-na-moeda-do-ativo = { $valor } in the asset's currency
ativos-resultado = Result
ativos-nao-realizado = Unrealized
ativos-realizado = Realized
ativos-resultado-total = Total result
ativos-retorno-anual = Annual return (XIRR)
ativos-real-por = Real by
ativos-nominal = Nominal
ativos-real = Real
ativos-posicao-encerrada = Position closed
ativos-sem-posicao = No position in this asset
ativos-encerrada-texto = All units were sold; the realized result and the income stay in the history.
ativos-sem-posicao-texto = Record a purchase to track average price, result and real return.
ativos-realizado-proventos = Realized { $realizado } · Income { $proventos }
ativos-avisos-calculo = Calculation warnings
ativos-hoje = Today · { $simbolo }
ativos-hoje-fonte = Source · Yahoo Finance (unofficial), ~15 min delay · history from daily closes

## Asset page: corporate actions and chart

ativos-eventos-ajuda = Split: factor greater than 1 (1 share becomes 5 = factor 5). Reverse split: factor less than 1 (10 become 1 = 0.1). Bonus: new shares per share (10% = 0.1). Record each event only once: if the statement already shows the entry at zero cost, do not record the event.
ativos-grafico-titulo = Price, trades and average price
ativos-grafico-periodo = Chart period
ativos-periodo-6m = 6M
ativos-periodo-1a = 1Y
ativos-periodo-2a = 2Y
ativos-periodo-max = Max
ativos-grafico-fonte = Source: { $fonte } · close as of each day (splits applied on their date), not adjusted for income
ativos-grafico-sem-cotacoes = No quotes: the line connects the prices of your trades
ativos-carregando-cotacoes = Loading quotes
ativos-sem-historico = No quote history for this asset
ativos-sem-historico-renda-fixa = Fixed income is valued on the curve (rate over the index), without a market quote.
ativos-sem-historico-texto = The chart shows the price of each trade. Update quotes in Indices and quotes or enter the quote manually.
ativos-grafico-aria = { $n ->
    [one] Price of { $nome } with { $n } trade marked and the average price as steps.
   *[other] Price of { $nome } with { $n } trades marked and the average price as steps.
}
ativos-fechamento = Close
ativos-preco-operacoes = Trade prices
ativos-compra = ▲ Buy
ativos-venda = ▼ Sell
ativos-tabela-operacoes = Trades and average price
ativos-qtd = Qty
ativos-preco = Price
ativos-preco-medio-depois = Average price after
ativos-analise-fundamentalista = Fundamental analysis

## Asset page: details

ativos-reativado = Asset restored
ativos-arquivado-toast = Asset archived
ativos-desarquivar = Unarchive
ativos-arquivar = Archive
ativos-excluir = Delete asset
ativos-excluir-descricao = { $nome } is removed, along with its watchlist entry and corporate actions. Assets with trades or income can only be archived.
ativos-excluido = Asset deleted
ativos-nao-excluiu = Could not delete
ativos-so-arquivar = With trades, the asset can only be archived.

## Registration: what each type asks for

ativos-fonte-manual = Manual (I enter it)
ativos-fonte-nenhuma = No quote
ativos-fonte-auto-yahoo = Automatic (the source turned on for this type)
ativos-fonte-auto-cripto = Automatic (DefiLlama, in US dollars)
ativos-fonte-yahoo = Yahoo Finance (unofficial source)
ativos-fonte-tiingo = Tiingo (with your key)
ativos-fonte-eodhd = EODHD (with your key)
ativos-fonte-brapi = brapi.dev (with a token, once a day)
ativos-fonte-cotahist = B3 COTAHIST (official, previous session's close)
ativos-fonte-auto-cvm = Automatic (CVM unit price by CNPJ)
ativos-fonte-auto-tesouro = Automatic (Tesouro Transparente)
ativos-fonte-auto-curva = Automatic (from the terms curve)
ativos-fonte-auto-nenhuma = Automatic (none)
ativos-fonte-yahoo-simbolo = Yahoo Finance (by symbol)
ativos-fonte-cota-cvm = CVM unit price
ativos-campo-setor = Sector
ativos-campo-setor-exemplo = Energy, Banks…
ativos-campo-segmento = Segment
ativos-opcional = Optional
ativos-opcional-ponto = Optional.
ativos-exemplo = E.g. { $exemplo }.
ativos-segmento-fii-exemplo = Logistics, Offices, Paper…
ativos-segmento-reit-exemplo = Retail, Data centers…
ativos-tipo-stock-br = Stocks traded on B3.
ativos-tipo-fii = Real estate funds traded on B3.
ativos-tipo-etf-br = ETFs traded on B3.
ativos-tipo-bdr = Receipts of foreign stocks traded on B3.
ativos-tipo-stock-us = Stocks on US exchanges (NYSE, NASDAQ), in dollars.
ativos-tipo-etf-us = ETFs on US exchanges, in dollars.
ativos-tipo-reit = US real estate funds, in dollars.
ativos-tipo-stock-intl = Stocks on other exchanges (London, Frankfurt, Tokyo…): the exchange sets the currency.
ativos-tipo-etf-intl = ETFs on other exchanges: the exchange sets the currency.
ativos-tipo-crypto = Cryptocurrencies, quoted on Yahoo Finance against a currency (BTC-USD).
ativos-tipo-treasury-br = Tesouro Direto bonds: the price comes from Tesouro Transparente by bond name.
ativos-tipo-fixed-income = CDB, LCI, LCA, CRI, CRA, debentures and savings: valued on the terms curve.
ativos-tipo-fund = Investment funds: the daily unit price comes from CVM by CNPJ.
ativos-tipo-pension = VGBL or PGBL private pension: contributions and withdrawals are the trades; the unit price comes from CVM by the plan fund's CNPJ.
ativos-tipo-other = Any other investment: enter the currency and, if you want, the quote symbol.
ativos-nome-cdb-exemplo = CDB Bank X 110% CDI 2028
ativos-cnpj-emissor = Issuing bank CNPJ
ativos-cnpj-emissor-dica = For the health score: the bank's indicators come from the Central Bank of Brazil. Without it, the issuer is looked up by name (Nubank, Itaú, Inter…).
ativos-nome-fundo = Fund name
ativos-cnpj-fundo = Fund CNPJ
ativos-cnpj-fundo-dica = For the daily CVM unit price.
ativos-categoria = Category
ativos-categoria-fundo-exemplo = Multi-market, Equities, Fixed income…
ativos-nome-plano = Plan name
ativos-cnpj-fie = Fund CNPJ (FIE)
ativos-cnpj-fie-dica = The plan fund's, on the insurer's statement: for the daily CVM unit price. Without it, enter the unit price under Quote.
ativos-codigo = Code
ativos-grupo-bolsa-br = Brazilian exchange
ativos-grupo-renda-fixa = Fixed income
ativos-grupo-fundos = Funds and pension
ativos-grupo-exterior = Abroad
ativos-grupo-outros = Other

## Add and edit asset

ativos-form-editar = Edit { $nome }
ativos-form-editar-descricao = Market, currency, country and identifiers don't change after the asset is added.
ativos-form-novo-descricao = Choose the asset type: the form shows only what it needs.
ativos-form-tipo = Asset type
ativos-form-erro-tipo = Choose the asset type.
ativos-form-erro-nome = Enter the asset name.
ativos-form-erro-moeda = Use the 3-letter ISO code (BRL, USD).
ativos-form-erro-taxa = Enter the rate.
ativos-form-erro-bolsa = Choose the exchange (or add it in Assets › Exchanges).
ativos-form-atualizado = Asset updated
ativos-form-cadastrado = Asset added
ativos-form-bolsa = Exchange
ativos-form-so-leitura = Read-only.
ativos-form-bolsa-obrigatoria = Required for other exchanges.
ativos-form-padrao = Default: { $padrao }.
ativos-form-nenhuma = none
ativos-form-escolha-bolsa = Choose the exchange
ativos-form-padrao-opcao = Default ({ $padrao })
ativos-form-moeda-cotacao = Quote currency
ativos-form-padrao-sem-ptax = Default: { $moeda }. No PTAX from the Central Bank of Brazil: the exchange rate to BRL is manual.
ativos-form-pais = Country
ativos-form-pais-dica = 2-letter code (BR, US).
ativos-form-rotulo = Asset
ativos-form-ou-busque = Or search by ticker or name:
ativos-form-buscar = Search asset
ativos-form-preenchido = Filled in with { $ativo }.
ativos-form-preenchido-cadastrado = Filled in with { $ativo }. This asset is already registered.
ativos-form-buscar-dica = Stocks, real estate funds, ETFs and BDRs from B3, the US and other exchanges, crypto. Pick one to fill in the form.
ativos-form-titulo-tesouro = Treasury bond
ativos-form-titulo-tesouro-dica = As in Tesouro Transparente (e.g. Tesouro IPCA+ 2035).
ativos-form-informar-termos = Enter index and rate
ativos-form-informar-termos-dica = Without a Tesouro Transparente unit price, the bond is valued on the curve with these terms.
ativos-form-carencia-ate = Lock-up until
ativos-form-isento-ir = Income tax exempt
ativos-form-isento-ir-dica = LCI, LCA, CRI, CRA, incentivized debentures and savings.
ativos-form-plano = Pension plan
ativos-form-plano-dica = Contributions and withdrawals are recorded as trades of the asset, in Portfolio.
ativos-form-cotacao-identificadores = Quote and identifiers
ativos-form-fonte = Quote source
ativos-form-simbolo = Symbol at the source
ativos-form-simbolo-automatico = Automatic: { $simbolo }.
ativos-form-simbolo-dica = Optional: PETR4.SA, VOD.L, BTC-USD…
ativos-form-observacoes = Notes
ativos-form-salvar = Save asset

## Announced income and Fundamentus (B3)

ativos-fonte = Source
ativos-pagamento = Payment
ativos-atualizar = Refresh
ativos-anunciados-data-com = Record date
ativos-anunciados-valor-cota = Amount per unit/share
ativos-anunciados-fii = Income announced by the fund
ativos-anunciados-titulo = Announced income
ativos-anunciados-novos = { $n ->
    [0] No new income
    [one] { $n } new income announced
   *[other] { $n } new income payments announced
}
ativos-anunciados-novos-dica = The ones you were entitled to show up in "Income to confirm" to record.
ativos-anunciados-buscar = Fetch from Fundamentus
ativos-anunciados-fonte = fundamentus.com.br (one-day cache)
ativos-anunciados-tabela = Income announced for { $ticker }
ativos-anunciados-nenhum = No income announced on Fundamentus.
ativos-fundamentus-data-com = Record date
ativos-fundamentus-cache = From cache, read on
ativos-fundamentus-lido = Read on
ativos-fundamentus-sem-papel = Fundamentus doesn't have { $ticker }.
ativos-fundamentus-fundo = Fund
ativos-fundamentus-empresa = Company
ativos-fundamentus-gestao = Management
ativos-fundamentus-subsetor = Subsector
ativos-fundamentus-cotacao-site = Quote on the site
ativos-fundamentus-cotacao-em = { $preco } on { $data }
ativos-fundamentus-oscilacao = Change
ativos-fundamentus-tabela = Income for { $ticker } on Fundamentus

## Watchlist

ativos-acoes = Actions
ativos-remover = Remove
ativos-adicionar = Add
ativos-watch-titulo = Watchlist · { $nome }
ativos-watch-adicionar-titulo = Add to watchlist
ativos-watch-descricao = Target price and alerts in the asset's currency. Alerts fire when the collected quote crosses the limit.
ativos-watch-atualizada = Watchlist updated
ativos-watch-adicionado = Asset on the watchlist
ativos-watch-buscar-dica = Ticker or name; a new asset is added when you pick it.
ativos-watch-escolha = Choose the asset.
ativos-watch-alvo-moeda = Target price ({ $moeda })
ativos-watch-alvo-dica = My target price.
ativos-watch-alertar-acima = Alert above
ativos-watch-alertar-abaixo = Alert below
ativos-watch-notas = Notes
ativos-watch-sem-cotacao = no quote
ativos-watch-hoje = Today
ativos-watch-alvo = Target price
ativos-watch-ate-alvo = To target
ativos-watch-alertas = Alerts
ativos-watch-acima-de = ▲ above { $preco }
ativos-watch-abaixo-de = ▼ below { $preco }
ativos-watch-situacao = Status
ativos-watch-remover-titulo = Remove from watchlist
ativos-watch-remover-descricao = { $nome } leaves the watchlist; the asset stays registered.
ativos-watch-removido = Removed from watchlist
ativos-watch-resumo = { $n ->
    [one] { $n } asset watched
   *[other] { $n } assets watched
} · { $disparados } with an alert triggered
ativos-watch-vazia = Watchlist empty
ativos-watch-vazia-descricao = Track assets you don't own yet, with a target price and quote alerts.
ativos-watch-editar-alvo = Edit target price and alerts…

## Asset statement and come-cotas

ativos-come-cotas = Come-cotas
ativos-come-cotas-regra = Come-cotas rule
ativos-come-cotas-auto = Automatic
ativos-come-cotas-longo = Long term (15%)
ativos-come-cotas-curto = Short term (20%)
ativos-come-cotas-isento = No come-cotas
ativos-come-cotas-descontos = { $n ->
    [one] { $n } deduction, total income tax { $total }
   *[other] { $n } deductions, total income tax { $total }
}
ativos-extrato-montando = Building the statement
ativos-extrato-confira = Check these lines
ativos-extrato-aviso = { $n ->
    [one] { $aviso } ({ $n } line flagged).
   *[other] { $aviso } ({ $n } lines flagged).
}
ativos-extrato-ajuda = In calculation order: corporate actions on a date apply before trades on the same day. Quantity and average price are the ones after each line.
ativos-extrato-qtd-registrada = (trade of { $qtd })
ativos-extrato-tabela = Asset statement
ativos-extrato-vazio = No trades or corporate actions.
ativos-extrato-lancamento = Entry
ativos-extrato-posicao-depois = Position after
ativos-extrato-origem = Origin
ativos-extrato-linha = { $lancamento } on { $data }
ativos-extrato-duplicata = Looks repeated: another line has the same date, quantity and price.
ativos-extrato-duplicatas-titulo = Entries that look repeated
ativos-extrato-duplicatas = { $n ->
    [one] { $n } line has the same date, quantity and price as another. If it is the same trade recorded twice, delete the extra one from the right-click menu; nothing is deleted until you confirm.
   *[other] { $n } lines have the same date, quantity and price as another. If it is the same trade recorded twice, delete the extra one from the right-click menu; nothing is deleted until you confirm.
}

## Pension plan

ativos-plano-tipo = Plan type
ativos-plano-tipo-dica = VGBL: income tax only on the gain, no deduction. PGBL and pension fund (EFPC, a closed company or cooperative plan): contributions deducted in the tax return (up to 12% of taxable income) and income tax on the full amount at withdrawal.
ativos-plano-efpc = Pension fund
ativos-plano-regime = Tax regime
ativos-plano-regime-dica = Regressive: 35% up to 2 years for each contribution, falling to 10% after 10 years (final). Progressive: 15% withheld and adjusted by the annual table.
ativos-plano-regressivo = Regressive
ativos-plano-progressivo = Progressive
ativos-plano-seguradora = Insurer
ativos-plano-inicio = Plan start
ativos-plano-taxa-adm = Management fee (% p.a.)
ativos-plano-taxa-carregamento = Loading fee (%)
ativos-plano-carregamento-dica = On each contribution, if any.

## Pension tab

ativos-prev-bruto = Gross
ativos-prev-base-ir = Taxable base
ativos-prev-liquido = Net
ativos-prev-aliquota = Effective rate
ativos-prev-plano-salvo = Pension plan saved
ativos-prev-salvar-plano = Save plan
ativos-prev-avisos = Pension warnings
ativos-prev-saldo = Balance and full withdrawal
ativos-prev-cota = Unit price
ativos-prev-aportado = Contributed
ativos-prev-valor-hoje = Value today
ativos-prev-rendimento = Gain
ativos-prev-resgatar-tudo = If you withdraw everything today
ativos-prev-mudam-faixa = { $valor } in contributions moves to a lower tax bracket in the next 90 days: waiting may mean paying less income tax.
ativos-prev-pagamentos-titulo = Payment history
ativos-prev-pagamentos-tabela = Pension payments
ativos-prev-pagamentos-pendencias = Payment history items to check
ativos-prev-pagamento-situacao = Status
ativos-prev-pagamento-cotas = { $cotas } units
ativos-prev-pagamento-sem-cota = Waiting for the CVM unit price
ativos-prev-pagamento-conta = Units account
ativos-prev-custodia = Account holding the plan's units
ativos-prev-custodia-conta = { $n ->
        [one] This plan's units are in the account { $contas }.
       *[other] This plan's units are in the accounts { $contas }.
    }
ativos-prev-custodia-sem-conta = This plan has no account for its units yet. Importing the payment history creates one together with the contributions.
ativos-prev-aportes-pendentes =
    { $n ->
        [one] { $n } contribution, { $valor } in total, has been waiting for the CVM unit price since { $data }: it becomes units on its own when the quote collection fetches the fund's history.
       *[other] { $n } contributions, { $valor } in total, have been waiting for the CVM unit price since { $data }: they become units on their own when the quote collection fetches the fund's history.
    }
ativos-prev-plano = Plan
ativos-prev-aportes-titulo = Contributions and tax bracket
ativos-prev-aportes-tabela = Pension contributions
ativos-prev-nenhum-aporte = No contributions: record contributions as purchase trades of the plan.
ativos-prev-aporte = Contribution
ativos-prev-ir-hoje = Regressive tax today
ativos-prev-proxima-faixa = Next bracket
ativos-prev-proxima-em = { $pct } on { $data }
ativos-prev-ja-menor = already at the lowest (10%)
ativos-prev-simular-titulo = Simulate withdrawal
ativos-prev-valor-bruto = Gross amount to withdraw
ativos-prev-simular = Simulate
ativos-prev-simular-dica = The oldest contributions go first (lowest rate under the regressive regime).
ativos-prev-resultado = Simulation result

## Import the pension statement (PDF)

ativos-prev-linha-participante = Your contribution
ativos-prev-linha-empregador = Employer extra
ativos-prev-linha-invalidez = Disability insurance
ativos-prev-linha-morte = Life insurance
ativos-prev-linha-nova = New
ativos-prev-linha-duplicada = Already imported
ativos-prev-linha-informativa = Info only
ativos-prev-competencia = Period
ativos-prev-cotas = Units
ativos-prev-importar = Import statement (PDF)
ativos-prev-importar-titulo = Import pension statement
ativos-prev-importar-descricao = The participant statement PDF (pension fund, such as Quanta Previdência): you review the preview before saving.
ativos-prev-extrato-pdf = Statement PDF
ativos-prev-lendo = Reading the statement…
ativos-prev-layout = Recognized layout
ativos-prev-cnpj-plano = Plan CNPJ
ativos-prev-adesao = Joined
ativos-prev-taxas = Fees
ativos-prev-taxas-valor = management { $adm }% p.a. · loading { $carregamento }%
ativos-prev-regime = Regime
ativos-prev-suas-contribuicoes = Your contributions
ativos-prev-extras-empregador = Employer extras
ativos-prev-seguros-risco = Risk insurance
ativos-prev-saldo-extrato = Balance on the statement
ativos-prev-cotas-conferem = The contribution units add up to the statement's unit balance.
ativos-prev-linhas = Statement lines
ativos-prev-conta-cotas = Account for the units
ativos-prev-conta-cotas-dica = Contributions become unit purchases without touching the account's cash.
ativos-prev-crie-corretora = Create a brokerage account
ativos-prev-seguros-dica = Disability and life cover are insurance costs, not units.
ativos-prev-nao-lancar = Don't record
ativos-prev-despesa-em = Expense in { $conta }
ativos-prev-importado = Statement imported
ativos-prev-importado-resumo = { $contribuicoes ->
    [one] { $contribuicoes } contribution
   *[other] { $contribuicoes } contributions
}{ $seguros ->
    [0] {""}
    [one] {" "}and { $seguros } risk insurance
   *[other] {" "}and { $seguros } risk insurance payments
}{ $puladas ->
    [0] {""}
   *[other] ; { $puladas } were already in the app
}.
ativos-prev-importar-linhas = { $n ->
    [one] Import { $n } line
   *[other] Import { $n } lines
}
ativos-prev-nada-novo = Nothing new to import

## Income to confirm

ativos-por-cota = Per unit
ativos-pendentes-data-pagamento = Payment date for { $nome }
ativos-pendentes-palpite = ≈ guess: ex-date
ativos-pendentes-titulo = Income to confirm
ativos-pendentes-titulo-n = Income to confirm ({ $n })
ativos-pendentes-conta = Account
ativos-pendentes-lancado = Income from { $nome } recorded
ativos-pendentes-pagamento-em = Paid on { $data } · { $conta }
ativos-pendentes-nao-lancou = Could not record
ativos-pendentes-ir = Tax { $valor }
ativos-pendentes-origem = Announced by the market source; not yet recorded in the portfolio.
ativos-pendentes-confirmar = Confirm
ativos-pendentes-confirmar-de = Confirm income from { $nome }
ativos-pendentes-dispensar = Dismiss
ativos-pendentes-dispensar-de = Dismiss income from { $nome }
ativos-pendentes-ajuda = Income announced with a record date on which the family held the asset and that is not in the portfolio yet. Confirming records the credit in the brokerage cash; dismissing hides it until you reopen the app.

## Dividend calendar

ativos-agenda-sua-qtd = Your qty
ativos-agenda-previsto-liquido = Expected net
ativos-agenda-projecao = ≈ Projection
ativos-agenda-ja-ocorreu = Already happened
ativos-agenda-anunciado = Announced
ativos-agenda-radar = Dividend radar (projection from history)
ativos-agenda-jcp = Expected already net of the income tax withheld on JCP (17.5% since 2026; 15% through 2025).
ativos-agenda-so-data-com = The source only gave the record date; payment comes with the company's notice.
ativos-agenda-periodo = Calendar period
ativos-agenda-ultimos-30 = Last 30 days
ativos-agenda-proximos = Next { $n }
ativos-agenda-so-minha = Only my portfolio
ativos-agenda-mostrar-projecoes = Show radar projections
ativos-agenda-previsto-moeda = Expected ({ $moeda })
ativos-agenda-vazia = No income on the calendar
ativos-agenda-vazia-descricao = The calendar comes from income collected with quotes (Yahoo Finance) and from radar projections. Update quotes in Indices and quotes, or widen the period.
ativos-agenda-n-eventos = { $n ->
    [one] { $n } event
   *[other] { $n } events
}
ativos-agenda-de = Calendar for { $mes }
ativos-agenda-ajuda = Expected = your quantity on the record date (or today, if it hasn't come yet) × amount per unit; JCP already net of the income tax withheld (17.5% since 2026; 15% through 2025). Radar projections use the payment history and change when the income is announced.

# Página dedicada do ativo (docs/asset-page.md).
ativo-trilha = Breadcrumb
ativo-perguntar-agente = Ask the agent
ativo-agente-pergunta = Analyze { $ativo } with the data Treasury has on it (position, fundamentals, calculated price, health, documents and studies) and tell me what stands out. If you find new, cross-checked data, propose completing the record.
ativo-aba-conversas = Conversations
ativo-conversas-texto = Conversations with the agent started on this page are kept here. Full analyses, with checked sources, are in AI study.
ativo-ver-estudos = See studies
ativo-conversas-vazio = No conversations about this asset yet
ativo-conversas-vazio-texto = Use "Ask the agent" to talk about { $ativo }; the conversation is kept here.
ativo-conversa-detalhe =
    { $n ->
        [one] { $quando } · { $n } message
       *[other] { $quando } · { $n } messages
    }
ativo-abrir-conversa = Open

# Visão geral e notícias do ativo (docs/asset-overview.md).
ativo-visao-geral = Overview
ativo-empresa = Company
ativo-setor = Sector
ativo-cotacao = Price
ativo-cotacao-em = on { $data } ({ $fonte })
ativo-score = Health score
ativo-visao-geral-sem-dados = No market data for this asset yet (price and indicators come from the asset analysis collection).
ativo-indicadores-chave = Key indicators
ativo-indicadores-sem = No indicators for this asset.
ativo-ver-fundamentos = See all fundamentals
ativo-ultimas = Latest news and filings
ativo-ultimas-vazio = Nothing recent here.
ativo-ver-noticias = See all news
ativo-aba-noticias = News
ativo-noticias-texto = Official CVM filings and headlines from financial news sites, newest first. Headlines from the news sites are filtered on this computer: nothing about your portfolio leaves it. Only title, outlet, date and link.
ativo-noticias-google-dica = Want more news? The Google News search opens in your browser: Treasury doesn't query Google.
ativo-noticias-abrir-google = Open in Google News
ativo-noticias-vazio = No recent news
ativo-noticias-vazio-texto = The news sites only carry each day's latest headlines; CVM filings show up after the asset analysis collection.
ativo-noticias-falhou = Couldn't read right now: { $fontes }.
ativo-noticia-origem-cvm = CVM
ativo-noticia-origem-portal = news site
ativo-resumo-do-ativo = Asset at a glance
ativos-conta-custodia = Held at (account)
ativos-conta-custodia-dica = The brokerage, the bank of the CD or fund, or the foreign account. Purchases, sales and withdrawals use this account.
ativos-conta-custodia-nenhuma = No account set
ativos-prev-informar-saldo = Enter balance
ativos-prev-informar-saldo-dica = For plans without a public unit price (closed pension funds, cooperatives): the balance the provider shows becomes the plan's value.
ativos-prev-saldo-data = Balance date
ativos-prev-saldo-valor = Balance
ativos-prev-salvar-saldo = Save balance
ativos-prev-saldo-salvo = Balance saved

# Fixed income without quantity (CDB, RDB, LCI…): values only.
ativos-aplicado = Amount invested
ativos-extrato-custo-depois = Cost after

## Trades tab summary

ativos-resumo-ops-aria = Trade summary
ativos-resumo-ops-compras = Buys
ativos-resumo-ops-vendas = Sells
ativos-resumo-ops-custos = Costs
ativos-resumo-ops-posicao = Current position
ativos-resumo-ops-total-comprado = Total bought
ativos-resumo-ops-total-vendido = Total sold
ativos-resumo-ops-total-custos = Fees and taxes
ativos-resumo-ops-operacoes = Trades
ativos-resumo-ops-preco-medio-compra = Average buy price
ativos-resumo-ops-preco-medio-venda = Average sell price
ativos-resumo-ops-resultado = Realized gain
ativos-resumo-ops-taxas = Brokerage and fees
ativos-resumo-ops-ir = Tax withheld
ativos-resumo-ops-do-volume = Of traded volume
ativos-resumo-ops-custo-posicao = Position cost
ativos-resumo-ops-situacao = Status
ativos-resumo-ops-zerada = Position closed
ativos-resumo-ops-periodo = Trades from { $de } to { $ate }
ativos-resumo-ops-periodo-um-dia = Trades on { $data }
ativos-resumo-ops-ajustado = Quantities and average prices adjusted for splits and reverse splits
ativos-resumo-ops-eventos = { $n ->
        [one] The current position includes one event (bonus, come-cotas or ticker change), so it differs from bought − sold.
       *[other] The current position includes { $n } events (bonuses, come-cotas or ticker changes), so it differs from bought − sold.
    }
