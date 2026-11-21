# Screen 01 · Dashboard: header, instrument cards (net worth, cash flow, budget, portfolio,
# categories, due dates, uncategorized) and "What Treasury noticed" (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Header

painel-atualizado = Updated { $data }
painel-descricao-membro = { $nome }'s finances: the transactions, accounts, assets and debts that belong to them (joint accounts by each person's share).
painel-descricao-simples = How much came in, how much went out and what was left this month.
painel-voltar-simples = Back to the simple dashboard
painel-carregando = Measuring the dashboard
painel-ver-completo = See the full dashboard (net worth and portfolio)

## Labels repeated across the cards

painel-base = Basis
painel-base-caixa = cash
painel-base-competencia = accrual
painel-fonte = Source
painel-data = Date
painel-periodo = Period
painel-cambio = Exchange rate
painel-cambio-estimado = ≈ estimated

## 01 · Net worth

painel-patrimonio = Net worth
painel-patrimonio-fonte = Accounts + portfolio + assets
painel-contas = Accounts
painel-patrimonio-origem = Account balances, market value of the portfolio and value of physical assets
painel-patrimonio-estimado = Includes estimated values: the previous day's exchange rate (PTAX not yet published), quotes or assets valued by estimate.
painel-patrimonio-em = Net worth on { $data }
painel-desde = since { $mes }
painel-patrimonio-historico = Net worth over the last { $n } months
painel-investimentos = Investments
painel-bens = Assets
painel-dividas = Debts
# The part between ** is shown in bold.
painel-sem-cambio = **No exchange rate for { $moedas }**: { $contas } was left out of the total. Add the rate in Settings › Data sources.

## 02 · Cash flow this month

painel-fluxo = Cash flow this month
painel-receitas = Income
painel-receitas-de = Income in { $mes }
painel-despesas = Expenses
painel-despesas-de = Expenses in { $mes }
painel-saldo-mes = Net for the month
painel-taxa-poupanca = Savings rate
painel-taxa-poupanca-de = Savings rate in { $mes }
painel-media-12m = 12-month average
painel-taxa-poupanca-historico = Savings rate month by month, { $n } months

## 03 · Budget this month

painel-orcamento = Budget this month
painel-orcamento-carregando = Loading the budget…
painel-orcamento-abrir = Open budget
painel-orcamento-sem = No budget for { $mes }.
painel-orcamento-gasto = Spent this month: { $valor }. Set the amounts or copy them from the previous month.
painel-orcamento-de-valor = of { $valor }
painel-orcamento-de = Budget for { $mes }
painel-orcamento-ritmo = { $pct } of planned · { $decorrido } of the month
painel-orcamento-comprometido = Committed (planned): { $valor }

## 04 · Portfolio

painel-real = Real
painel-nominal = nominal
painel-carteira-carregando = Loading the portfolio…
painel-carteira-vazia = No positions yet.
painel-carteira-cadastrar = Add operations
painel-valor-mercado = Market value
painel-carteira-origem = Portfolio quotes
painel-sem-cotacao = No quote: { $ativos }
painel-resultado-total = total return
painel-xirr-nominal = Nominal XIRR
painel-xirr-real = Real XIRR
painel-xirr-real-dica = Arrives with the inflation indices

## 05 · Top categories

painel-categorias = Top categories
painel-despesas-mes = Expenses this month
painel-categorias-vazio = No expenses this month.
painel-categorias-lista = Categories with the most spending this month
painel-categoria-ver = { $categoria }: see transactions

## 06 · Upcoming due dates

painel-vencimentos = Upcoming due dates
painel-janela = Window
painel-30-dias = 30 days
painel-saidas-previstas = Planned outflows
painel-itens = Items
painel-vencimentos-vazio = Nothing is due in the next 30 days.
painel-hoje = today
painel-amanha = tomorrow
painel-em-dias = in { $n } d
painel-fatura = Card bill
painel-previsto = Planned
painel-mais-itens =
    { $n ->
        [one] + { $n } item in the 30 days
       *[other] + { $n } items in the 30 days
    }

## 07 · Uncategorized

painel-para-revisar = Transactions to review
painel-tudo-categorizado = ✓ everything categorized
painel-fora-categorias = left out of the categories in reports
painel-categorizar = Categorize now

## What Treasury noticed (insights calculated by the app, no AI)

painel-percepcoes = What Treasury noticed
painel-percepcoes-sem-ia = calculated on your computer, no AI
painel-inflacao-grupos = Personal inflation by group
painel-inflacao-grupo = IPCA group
painel-inflacao-peso = Weight in your spending
painel-inflacao-subiu = Went up
painel-esconder-grupos = Hide groups
painel-ver-grupos = See by group
painel-abrir = Open
painel-dispensar = Dismiss
painel-mostrar-menos = Show less
painel-ver-todas = See all { $n }
