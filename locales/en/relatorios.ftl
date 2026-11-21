# Reports (src/features/reports): the controls also used on the main screens (month, accrual ×
# cash, loading, error) and "Explain" for each amount (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Controls

relatorios-competencia = Accrual
relatorios-competencia-dica = By purchase date
relatorios-caixa = Cash
relatorios-caixa-dica = By payment date (the card bill's due date for cards)
relatorios-base-datas = Date basis
relatorios-mes-anterior = Previous month
relatorios-proximo-mes = Next month
relatorios-falha-carregar = Failed to load.
relatorios-erro-carregar = Loading error
relatorios-tentar-de-novo = Try again
relatorios-sem-dados-periodo = No data for the period

## Explain an amount

relatorios-explicar = Explain { $titulo }
relatorios-explicar-dica = Explain: { $titulo }
relatorios-explicacao = Explanation
relatorios-explicacao-de = Explanation · { $titulo }
relatorios-ver-lancamentos = See transactions
relatorios-calculando-origem = Working out the source
relatorios-fontes = Sources
relatorios-itens = { $n ->
        [one] { $n } item
       *[other] { $n } items
    }
relatorios-mostrando = showing { $n }
relatorios-formula = Formula
relatorios-inclui-estimado = ≈ Includes an estimated amount (exchange rate or the previous day's quote).
relatorios-sem-cambio = No exchange rate for { $moedas }: those amounts were left out of the total.
relatorios-fontes-valor = Sources of the amount
relatorios-nenhuma-fonte = No sources: the amount is zero.
relatorios-lista-recentes = The list shows the { $n } most recent transactions; the full sum is in the formula.
relatorios-descricao = Description
relatorios-taxa = rate
relatorios-fonte-lancamento = Transaction
relatorios-fonte-conta = Account
relatorios-fonte-cambio = Exchange rate
relatorios-fonte-saldo-inicial = Opening balance
relatorios-fonte-indice = Index
relatorios-fonte-carteira = Portfolio
relatorios-fonte-divida = Debt
relatorios-fonte-bem-fisico = Physical asset

## Screen 08 · Reports

relatorios-descricao-tela = Spending in every shape: where it went, what weighs the most, when, whether it is going up, where you went over and how much is already committed. Click any piece to see the transactions.
relatorios-descricao-ir = Income tax on variable income: the month's tax, the DARF and the amounts for the annual return.
relatorios-descricao-saude = Financial health: a score from 0 to 100 calculated from your data, its dimensions, each indicator × target and what to improve.
relatorios-imposto-renda = Income tax
relatorios-saude-financeira = Financial health
relatorios-visao-frente = Looking ahead
relatorios-visualizacoes = Views
relatorios-aba-aria = { $vista }: { $pergunta }
relatorios-aba-ir-sigla = IR
relatorios-aba-ir-dica = Income tax, DARF and annual return
relatorios-aba-ir-aria = Income tax: calculation, DARF and annual return
relatorios-aba-saude-sigla = FH
relatorios-aba-saude-dica = Financial health score, dimensions and suggestions
relatorios-aba-saude-aria = Financial health: score, indicators and suggestions

## Views (tab and question)

relatorios-vista-fluxo = Flow
relatorios-vista-fluxo-pergunta = Where does the income go?
relatorios-vista-composicao = Composition
relatorios-vista-composicao-pergunta = What weighs the most in the budget?
relatorios-vista-ritmo = Rhythm
relatorios-vista-ritmo-pergunta = On which days and weeks do we spend the most?
relatorios-vista-tendencia = Trend
relatorios-vista-tendencia-pergunta = Is spending going up? On what?
relatorios-vista-orcamento = Budget
relatorios-vista-orcamento-pergunta = Where did we go over?
relatorios-vista-comparacao = Comparison
relatorios-vista-comparacao-pergunta = What changed from one period to another?
relatorios-vista-inflacao = Inflation
relatorios-vista-inflacao-pergunta = Did we spend more, or did things just get more expensive?
relatorios-vista-titulares = Members
relatorios-vista-titulares-pergunta = Who spends on what?
relatorios-vista-recorrentes = Recurring
relatorios-vista-recorrentes-pergunta = How much is committed every month?
relatorios-vista-cartoes = Cards
relatorios-vista-cartoes-pergunta = How much is already committed in the coming months?
relatorios-vista-patrimonio = Net worth
relatorios-vista-patrimonio-pergunta = Is net worth growing?

## Periods and filters

relatorios-periodo-mes = This month
relatorios-periodo-mes-anterior = Last month
relatorios-periodo-trimestre = This quarter
relatorios-periodo-trimestre-anterior = Last quarter
relatorios-periodo-3m = Last 3 months
relatorios-periodo-6m = Last 6 months
relatorios-periodo-ano = This year
relatorios-periodo-12m = Last 12 months
relatorios-periodo-ano-anterior = Last year
relatorios-periodo-personalizado = Custom
# A period of several months or of loose dates: "January 2026 to March 2026".
relatorios-periodo-intervalo = { $de } to { $ate }
relatorios-filtros = Report filters
relatorios-periodo = Period
relatorios-periodo-anterior = Previous period
relatorios-proximo-periodo = Next period
relatorios-de = From
relatorios-ate = To
relatorios-data-inicial = Start date
relatorios-data-final = End date
relatorios-base = Basis
relatorios-membro = Member
relatorios-conta = Account
relatorios-todos = All
relatorios-todas = All
# Date basis in the middle of the source line ("Source: … · cash · …").
relatorios-base-caixa = cash
relatorios-base-competencia = accrual
relatorios-filtro-nota-membro-conta = This figure shows the whole family: the member and account filter applies to Composition.
relatorios-filtro-nota-membro = This figure shows the whole family: the member filter applies to Composition.
relatorios-filtro-nota-conta = This figure shows the whole family: the account filter applies to Composition.

## Common figure pieces

relatorios-outras = Other
relatorios-sem-categoria = Uncategorized
relatorios-direto = { $categoria } (direct)
relatorios-categoria = Category
relatorios-receita = Income
relatorios-receitas = Income
relatorios-despesas = Expenses
relatorios-total = Total
relatorios-mes = Month
relatorios-grupo = Group
relatorios-forma = Shape
relatorios-novo = new
relatorios-nenhum = none
relatorios-pct-receita = % of income
relatorios-pct-total = % of total
relatorios-da-receita = Of income
relatorios-do-total = { $pct } of total
relatorios-item-em = { $nome } in { $periodo }
relatorios-receitas-de = Income in { $periodo }
relatorios-despesas-de = Expenses in { $periodo }
relatorios-fonte-sem-cambio = { $fonte } · no exchange rate: { $moedas }
relatorios-fonte-despesas = Source: settled expense transactions · { $base }
relatorios-fonte-despesas-filtro = Source: settled expense transactions · { $base } · with member/account filter

## 01 · Flow

relatorios-carregando-fluxo = Tracing the flow
relatorios-detalhe = Detail
relatorios-detalhe-categorias = Categories
relatorios-detalhe-categorias-dica = Top-level categories only
relatorios-detalhe-subcategorias = Subcategories
relatorios-detalhe-subcategorias-dica = Categories and subcategories
relatorios-grupo-in = Inflows
relatorios-grupo-out = Outflows
relatorios-grupo-result = Result
relatorios-vs-anterior = vs previous:
relatorios-periodo-anterior-dica = Previous period: { $periodo }
relatorios-ver-compoe-dica = See the transactions that make up this amount
relatorios-ver-compoe-sr = (see what makes up { $item })
relatorios-pontos = { $valor } pp
relatorios-para = To
relatorios-anterior = Previous
relatorios-o-periodo-anterior = the previous period
relatorios-diferenca-de = the difference in { $nome }
relatorios-sobra = Surplus
relatorios-deficit = Deficit
relatorios-fluxo-pergunta = Where the income goes: what comes in (income and withdrawals) goes through the budget and leaves as expenses by category, debt payments, investments and the surplus.
relatorios-fluxo-pergunta-comparado = Where the income goes: what comes in (income and withdrawals) goes through the budget and leaves as expenses by category, debt payments, investments and the surplus, compared with { $periodo }.
relatorios-taxa-poupanca = Savings rate
relatorios-taxa-poupanca-de = Savings rate in { $periodo }
relatorios-no-periodo = in the period
relatorios-investimentos-aportes = Investments (contributions) { $valor }
relatorios-resgates = Withdrawals { $valor }
relatorios-pagamento-dividas = Debt payments { $valor } (included in expenses)
relatorios-fluxo-em-andamento = The period is still in progress (up to today) and is compared with all of { $periodo }. To compare closed periods, use the arrows or choose “Last month”.
relatorios-fluxo-titulo = Flow · { $periodo }
relatorios-fluxo-fonte = Source: settled transactions · { $base } · transfers between accounts and card bill payments are left out; contributions and withdrawals shown separately
# $investimentos and $sobra: "sim" (yes) or "nao" (no).
relatorios-fluxo-aria = Flow for { $periodo }: income split across { $n } expense categories{ $investimentos ->
        [sim] , investments
       *[nao] {""}
    }{ $sobra ->
        [sim] {" "}and the surplus
       *[nao] , with a deficit
    }.
relatorios-sem-movimento = No activity in the period
relatorios-fluxo-tabela = Flow: source, destination and amount
relatorios-comparativo-titulo = Comparison · { $anterior } × { $atual }
relatorios-comparativo-tabela = Flow comparison: { $anterior } and { $atual }
relatorios-comparativo-dica = Click an amount to see the transactions that make it up (in the month, in the previous one or in the difference).

## 02 · Composition

relatorios-carregando-composicao = Adding up the categories
relatorios-composicao-pergunta = What weighs the most: each block is a category, split into its subcategories, sized by spending.
relatorios-maior-categoria = Largest category
relatorios-peso-maior = Share of the largest
relatorios-composicao-titulo = Composition · { $periodo }
relatorios-forma-treemap = Treemap
relatorios-forma-sunburst = Sunburst
relatorios-composicao-aria = Composition of expenses in { $periodo }: { $maiores } are the largest.
relatorios-composicao-tabela = Composition by category

## 03 · Rhythm

relatorios-carregando-ritmo = Building the calendar
relatorios-dia = Day
relatorios-semana = Weekday
relatorios-ritmo-pergunta = On which days and weeks we spend the most: one square per day, darker the more was spent.
relatorios-despesas-12-meses = Expenses in 12 months
relatorios-media-por-dia = Daily average
relatorios-dia-mais-caro = Most expensive day
relatorios-dias-sem-gasto = Days without spending
relatorios-ritmo-titulo = Rhythm · { $periodo }
relatorios-ritmo-fonte = Source: settled expense transactions · { $base } · bands by quantiles of the days with spending
relatorios-ritmo-aria = Spending calendar for { $periodo }; the most expensive day was { $dia }.
relatorios-media-dia-semana-aria = Average spending by day of the week
relatorios-media-dia-semana = Average by day of the week
relatorios-dias-maior-gasto = Highest spending days
relatorios-dia-semana-data = { $semana }, { $data }
relatorios-gasto-por-dia = Spending by day

## 04 · Trend

relatorios-carregando-tendencia = Adding up the months
relatorios-media-3m = 3m average
relatorios-media-movel-3m = Moving average (3 months)
relatorios-tendencia-pergunta = Is spending going up? On what? Bars by category, the 3-month moving average and income for reference.
relatorios-media-movel-ultimo = Moving average (last month)
relatorios-variacao-media-movel = Change in moving average
relatorios-contra = vs { $mes }
relatorios-media-meses = { $n }-month average
relatorios-receitas-periodo = Income in the period
relatorios-tendencia-titulo = Trend · { $n } months to { $mes }
relatorios-tendencia-fonte = Source: settled expense and income transactions · { $base } · moving average of the month and the two before it
relatorios-tendencia-aria = Expenses by category over { $n } months to { $mes }, with a 3-month moving average and income.
relatorios-tendencia-tabela = Expenses by month and category

## 05 · Budget

relatorios-orcamento-vazio = No budget or spending in { $mes }
relatorios-montar-orcamento = Set up the budget
relatorios-previsto = Planned
relatorios-realizado = Actual
relatorios-comprometido = Committed
relatorios-restante = Remaining
relatorios-situacao = Status
relatorios-orcamento-pergunta = Where we went over: planned (mark), actual (bar), committed (light segment) and the month's pace (thin line).
relatorios-estouradas = Over budget
relatorios-mes-decorrido = Month elapsed
relatorios-orcamento-titulo = Budget · { $mes }
relatorios-orcamento-fonte = Source: the month's budget and expense transactions · { $base } · actual includes subcategories
relatorios-orcamento-aria = Planned × actual for { $mes }
relatorios-clique-nome = Click the name to see the transactions
relatorios-sem-previsto = nothing planned
relatorios-orcamento-tabela = Planned × actual

## 06 · Comparison

relatorios-carregando-comparacao = Comparing the periods
relatorios-comparacao-pergunta = What changed: the change in each category between { $a } (A) and { $b } (B). Spending more shows as ▲.
relatorios-comparacao-a = A · { $periodo }
relatorios-comparacao-b = B · { $periodo }
relatorios-variacao-pct = Change %
relatorios-comparar-com = Compare with
relatorios-um-ano-antes = One year earlier
relatorios-comparacao-em-andamento = B is still in progress (up to today): part of the period's expenses has not happened yet. To compare closed months, choose “Last month”.
relatorios-comparacao-titulo = Comparison · { $a } × { $b }
relatorios-comparacao-fonte = Source: settled expense transactions · { $base } · top-level categories
relatorios-forma-barras = Bars
relatorios-forma-slope = Slope
relatorios-comparacao-aria = Change in spending by category between { $a } and { $b }.
relatorios-comparacao-tabela = Comparison by category

## 07 · Inflation (placeholder)


## 08 · Members

relatorios-carregando-membros = Splitting by member
relatorios-membros-pergunta = Who spends on what: each member's spending by category (a split's part counts for that part's member).
relatorios-membros-titulo = Members · { $periodo }
relatorios-membros-fonte = Source: settled expense transactions · { $base } · { $n } categories
relatorios-membros-fonte-outras = Source: settled expense transactions · { $base } · { $n } categories ({ $outras } in "Other")
relatorios-membros-aria = Spending by member and category in { $periodo }.
relatorios-membros-tabela = Spending by member and category

## 09 · Recurring

relatorios-sem-recorrencias = No recurrences
relatorios-sem-recorrencias-titulo = No subscriptions or fixed bills set up
relatorios-sem-recorrencias-descricao = Set up recurrences (rent, school, subscriptions) on the transactions screen: they create the planned transactions and show up here.
relatorios-carregando-recorrencias = Reading the recurrences
relatorios-regra = Rule
relatorios-proximas = Next
relatorios-recorrentes-pergunta = How much is already committed every month to subscriptions and fixed bills, and when each one is due.
relatorios-comprometido-mes = Committed per month
relatorios-receitas-recorrentes = Recurring income
relatorios-das-receitas-recorrentes = Of recurring income
relatorios-despesas-recorrentes = Recurring expenses
relatorios-recorrentes-titulo = Recurring · next { $n } days
relatorios-recorrentes-fonte = Source: active recurrences (rule and next dates) · monthly equivalent by frequency
relatorios-recorrentes-aria = Timeline of recurrences over the next { $n } days
relatorios-recorrencias = Recurrences

## 10 · Cards

relatorios-carregando-cartoes = Adding up the bills
relatorios-sem-cartoes = No cards
relatorios-sem-cartoes-titulo = No credit card set up
relatorios-sem-cartoes-descricao = With cards and installment purchases set up, this figure shows how much is already due each month.
relatorios-fatura = Bill
relatorios-cartoes-pergunta = How much is already committed in the coming months: what is left to pay on each bill, including future installments.
relatorios-proximos-meses = Next { $n } months
relatorios-a-pagar-em = Due in { $mes }
relatorios-proxima-fatura = Next bill
relatorios-cartoes-titulo = Cards · bills from { $mes }
relatorios-cartoes-fonte = Source: unpaid bills and planned installments · bill due month · amounts in the base currency
relatorios-cartoes-aria = Upcoming card bills over the next { $n } months.
relatorios-cartoes-tabela = Upcoming bills by card
relatorios-total-faturas = Total of bills

## 11 · Net worth

relatorios-carregando-patrimonio = Measuring net worth
relatorios-contas = Accounts
relatorios-investimentos = Investments
relatorios-bens = Assets
relatorios-dividas = Debts
relatorios-liquido = Net
relatorios-patrimonio-liquido = Net worth
relatorios-patrimonio-pergunta = Is net worth growing? Accounts, investments and assets above zero, debts below, and the net amount as the line.
relatorios-liquido-em = Net in { $mes }
relatorios-patrimonio-liquido-em = Net worth on { $data }
relatorios-explicar-patrimonio = Explain net worth on { $data }
relatorios-variacao-desde = Change since { $mes }
relatorios-patrimonio-titulo = Net worth · { $n } months to { $mes }
# $estimado and $semcambio: "sim" (yes) or "nao" (no).
relatorios-patrimonio-fonte = Source: balances on the last day of each month + market value of the portfolio + value of assets{ $estimado ->
        [sim] {" "}· ≈ includes estimated exchange rates or quotes
       *[nao] {""}
    }{ $semcambio ->
        [sim] {" "}· no exchange rate: { $moedas }
       *[nao] {""}
    } · click a month to see where it comes from
relatorios-patrimonio-aria = Net worth over { $n } months to { $mes }: accounts, investments and debts.
relatorios-patrimonio-tabela = Net worth by month

## Breakdown of a flow line

relatorios-analitico = Breakdown
relatorios-analitico-de = Breakdown · { $titulo }
relatorios-carregando-lancamentos = Adding up the transactions
relatorios-diferenca-delta = Difference (Δ)
relatorios-agrupar-categoria-dica = By category and subcategory
relatorios-agrupar-descricao-dica = By transaction description
relatorios-dividido-dica = Split transaction: change each part's category in the editor
relatorios-dividido = · split
relatorios-categoria-de = Category of { $nome }
relatorios-categoria-alterada = Category changed
relatorios-relatorio-recalculado = { $nome }: the report was recalculated.
relatorios-categoria-nao-mudou = Could not change the category
relatorios-data = Date
relatorios-linha = Line
relatorios-editar-item = Edit { $nome }
relatorios-editar-dica = Change the category right on the line: the report recalculates at once. "Edit" (or double-click) opens the full transaction.
relatorios-parte-dividido = part of a split transaction
relatorios-fatura-vence = bill due { $data }
relatorios-soma-confere = { $n ->
        [one] The item adds up
       *[other] The { $n } items add up
    } to { $soma }, equal to the amount for { $periodo } in the table.
relatorios-soma-difere = The sum of the items ({ $soma }) differs from the table amount ({ $valor }). Reload the report.
relatorios-cambio-estimado = ≈ Includes an amount converted at an estimated exchange rate.
relatorios-por-categoria-em = By category in { $periodo }
relatorios-o-que-compoe = What makes up { $nome } in { $periodo }
relatorios-nada-em = Nothing in { $periodo }.
relatorios-abre-analitico = Enter or double-click a line to open its breakdown.
relatorios-fora-total = Left out of the total
relatorios-fora-total-n = Left out of the total ({ $n })
relatorios-fora-total-de = Left out of the total of { $nome } in { $periodo }
relatorios-por-que-fora = Why it is left out
relatorios-diferenca-resumo = From { $anterior } ({ $antes }) to { $atual } ({ $depois }): { $diferenca }. Each line below shows how much it changed; the changes add up to the difference in the table.
relatorios-agrupar-por = Group by
relatorios-agrupar-diferenca = Group the difference by
relatorios-diferenca-por-categoria = Difference in { $nome } by category
relatorios-diferenca-por-descricao = Difference in { $nome } by description
relatorios-nada-mudou = Nothing changed between the two periods.
relatorios-lancamentos-de = Transactions of { $nome }
relatorios-lancamentos-dois-periodos = Transactions of { $nome } in both periods
relatorios-linhas-dois-periodos = Lines of { $nome } in both periods
relatorios-nenhum-lancamento = No transactions.

## ⌘K bar commands (search words are separated by commas)

relatorios-cmd-grupo-busca = report,chart
relatorios-cmd-fluxo = Report: this month's flow
relatorios-cmd-fluxo-busca = sankey,where did it go,where does it go,income,expenses,surplus
relatorios-cmd-fluxo-trimestre = Report: this quarter's flow × previous quarter
relatorios-cmd-fluxo-trimestre-busca = sankey,income,compare,investments,debts
relatorios-cmd-fluxo-anterior = Report: last month's flow
relatorios-cmd-fluxo-anterior-busca = sankey
relatorios-cmd-composicao = Report: spending composition
relatorios-cmd-composicao-busca = treemap,sunburst,categories,weighs
relatorios-cmd-ritmo = Report: spending calendar
relatorios-cmd-ritmo-busca = heatmap,days,week,rhythm
relatorios-cmd-tendencia = Report: spending trend (12 months)
relatorios-cmd-tendencia-busca = evolution,moving average,going up
relatorios-cmd-orcamento = Report: planned × actual
relatorios-cmd-orcamento-busca = bullet,over budget,budget
relatorios-cmd-comparacao = Report: compare with the previous month
relatorios-cmd-comparacao-busca = a × b,change,what changed
relatorios-cmd-titulares = Report: spending by family member
relatorios-cmd-titulares-busca = members,family,who spends
relatorios-cmd-recorrentes = Report: subscriptions and fixed bills
relatorios-cmd-recorrentes-busca = recurrences,committed,monthly
relatorios-cmd-cartoes = Report: upcoming card bills
relatorios-cmd-cartoes-busca = installments,installment plans,card
relatorios-cmd-patrimonio = Report: net worth over 24 months
relatorios-cmd-patrimonio-busca = assets,liabilities,debts,investments
relatorios-cmd-inflacao = Report: spending adjusted for inflation
relatorios-cmd-inflacao-busca = ipca,real,nominal
relatorios-cmd-ir-mes = IR: the month's income tax and DARF
relatorios-cmd-ir-mes-busca = income tax,darf,capital gains,day trade,exemption,20 thousand
relatorios-cmd-ir-ano = IR: report for the annual return
relatorios-cmd-ir-ano-busca = income tax,irpf,assets and rights,exempt income,return
relatorios-cmd-impostos-ano = Taxes paid in the year (estimate)
relatorios-cmd-impostos-ano-busca = tax burden,taxes,consumption,icms,inss,scenarios
relatorios-cmd-impostos-grupos = Taxes: category groups
relatorios-cmd-impostos-grupos-busca = own rate,tax group,categories,mapping
relatorios-cmd-divida-nova = Add a loan or financing
relatorios-cmd-divida-nova-busca = debt,contract,sac,price,payroll loan,financing,cdc,mortgage
relatorios-cmd-bem-novo = Add a property, vehicle or asset
relatorios-cmd-bem-novo-busca = asset,net worth,property,apartment,house,car,land,fipe
relatorios-cmd-bens = See assets (physical net worth)
relatorios-cmd-bens-busca = assets,properties,vehicles,improvements,appraisal,appreciation
relatorios-cmd-orcamento-mes = This month's budget
relatorios-cmd-orcamento-mes-busca = planned,actual,budget
relatorios-cmd-metas = Goals
relatorios-cmd-metas-busca = goals,trip,payoff,progress
relatorios-cmd-reserva = Emergency fund calculator
relatorios-cmd-reserva-busca = reserve,emergency,cushion,emergency fund
relatorios-cmd-sem-categoria = Uncategorized transactions this month
relatorios-cmd-sem-categoria-busca = categorize,review,uncategorized

## 12 · Groups (where the income goes)

relatorios-vista-grupos = Groups
relatorios-vista-grupos-pergunta = How much of the income goes to essentials, debts, the future and the rest?
relatorios-cmd-grupos = Report: where the income goes (groups)
relatorios-cmd-grupos-busca = groups,essentials,debts,future,non-essential,50/30/20,income
relatorios-grupos-pergunta = Where the income goes: how much of each month went to essentials, debts, the future (contributions and goals) and non-essentials, against each group's target.
relatorios-grupos-carregando = Calculating the groups…
relatorios-grupos-renda = Income in { $periodo }
relatorios-grupos-stat-dica = { $valor } · target { $meta }
relatorios-grupos-stat-dica-sem-meta = { $valor } · no target
relatorios-grupos-titulo = Where the income goes: { $n } months to { $mes }
relatorios-grupos-fonte = Source: settled transactions, contributions and transfers to goals · { $base }
relatorios-grupos-medida = Measure
relatorios-grupos-medida-pct = % of income
relatorios-grupos-medida-valor = Amount
relatorios-grupos-forma-barras = Bars by month
relatorios-grupos-forma-linhas = Trend
relatorios-grupos-aria-barras = Spending groups by month, { $n } months to { $mes }
relatorios-grupos-aria-linhas = Share of income of each group, { $n } months to { $mes }
relatorios-grupos-tabela = Spending groups by month
relatorios-grupos-maiores-titulo = Largest categories of each group in { $periodo }
relatorios-grupos-maiores-tabela = Largest categories by group
relatorios-grupos-grupo = Group
relatorios-grupos-pct-grupo = % of group
relatorios-grupos-pct-renda = % of income
relatorios-grupos-meta = Target
relatorios-grupos-sem-renda = No income in the period: the share of income stays blank.
relatorios-grupos-nota-futuro = Future adds up contributions and transfers to goal accounts; withdrawals don't reduce it.
relatorios-grupos-metas-link = Edit targets
relatorios-grupos-categorias-link = Adjust groups
relatorios-grupos-composicao-link = See how much of the income goes to each group in 12 · Groups
relatorios-fluxo-fatura-competencia = On an accrual basis, the purchases of card bills paid in this period ({ $valor }) count in the month they were made, while redemptions and contributions count in the month the money moved. On a cash basis, the bill shows up here, split into the purchases' categories.
