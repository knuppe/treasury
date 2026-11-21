# Screen 05 · Budget: tabs, planned vs. actual for the month (summary, editable table, copy
# from the previous month and suggest from the average) (docs/i18n-design.md). Goals, projects,
# emergency fund and retirement have their own catalogs; subscriptions and scenarios are in
# planejamento.ftl. Revision: AI draft, not yet reviewed by a native speaker.

## Tabs

orcamento-abas = Budget sections
orcamento-aba-previsto = Planned vs. actual
orcamento-aba-metas = Goals
orcamento-aba-projetos = Projects
orcamento-aba-reserva = Emergency fund
orcamento-aba-aposentadoria = Retirement
orcamento-aba-assinaturas = Subscriptions
orcamento-aba-parcelas = Installments
orcamento-aba-cenarios = Scenarios
orcamento-carregando = Adding up the budget

## Repeated labels

orcamento-previsto = Planned
orcamento-realizado = Actual
orcamento-recebido = Received
orcamento-comprometido = Committed
orcamento-restante = Remaining
orcamento-definir = Set
orcamento-erro-gravar = Could not save.

## Month summary

orcamento-resumo-de = Summary for { $mes }
orcamento-base = Basis
orcamento-base-caixa = cash
orcamento-base-competencia = accrual
orcamento-mes-decorrido = Month elapsed
orcamento-despesas-de = Expenses in { $mes }
orcamento-comprometido-dica = planned, not yet cleared
orcamento-ritmo = Month's pace
orcamento-de = Budget for { $mes }
orcamento-ritmo-texto = { $pct } of planned with { $decorrido } of the month · projection { $projecao }

## Copy from the previous month

orcamento-copiar-titulo = Copy budget
orcamento-copiar-descricao = Copies the lines from { $de } to { $para }. Archived categories are left out.
orcamento-copiar-anterior = Copy from previous month
orcamento-copiado = Budget copied
orcamento-copiar = Copy
orcamento-substituir = Replace the lines that already exist in { $mes }

## Suggest from the average

orcamento-aplicar = Apply
orcamento-atual = Current
orcamento-media = Average
orcamento-diferenca = Difference
orcamento-sugestao = Suggestion from the average
orcamento-aplicar-a = Apply to { $categoria }
orcamento-sugerir = Suggest from the average
orcamento-sugerir-descricao = Average spending of each category over the previous full months, rounded up to the next multiple of { $valor }.
orcamento-atualizado = Budget updated
orcamento-atualizado-descricao =
    { $n ->
        [one] { $n } category from the average.
       *[other] { $n } categories from the average.
    }
orcamento-aplicar-n =
    { $n ->
        [0] Apply
       *[other] Apply { $n }
    }
orcamento-meses-media = Months in the average
orcamento-n-meses = { $n } months
orcamento-calculando = Calculating the average…
orcamento-igual-media = The budget already matches the average.

## Table and notices

orcamento-gravar-erro = Could not save the planned amount
orcamento-adicionar-aria = Add category to the budget
orcamento-adicionar = Add category…
orcamento-sem-orcamento-aria = Categories with spending and no budget
orcamento-n-sem-orcamento =
    { $n ->
        [one] { $n } category with spending and no budget
       *[other] { $n } categories with spending and no budget
    }
orcamento-despesas-categoria = Expenses by category
orcamento-vazio = No budget or spending in { $mes }. Copy from the previous month, use the average or add a category.
orcamento-tabela-despesas = Expense budget by category
orcamento-receitas-previstas = Planned income
orcamento-tabela-receitas = Income budget by category
orcamento-cambio = Exchange rate
orcamento-sem-cambio = No exchange rate for { $moedas }: those transactions were left out of the actual amount.
orcamento-previsto-de = Planned for { $categoria }
orcamento-soma-filhos = sum of subcategories
orcamento-editar-previsto-de = Edit planned for { $categoria }
orcamento-gravando = saving…
orcamento-soma-subcategorias-dica = Sum of the subcategories (the parent has no amount of its own)
orcamento-soma-subcategorias = sum of the subcategories
orcamento-previsto-realizado = Planned vs. actual
orcamento-recebido-previsto = Received vs. planned
orcamento-percentual = Percentage
orcamento-editar-o-previsto = Edit the planned amount
orcamento-definir-o-previsto = Set the planned amount
orcamento-ver-lancamentos-mes = See transactions for { $mes }
orcamento-recolher-subcategorias = Collapse subcategories
orcamento-expandir-subcategorias = Expand subcategories
orcamento-acoes-de = Actions for { $categoria }
orcamento-recolher = Collapse { $categoria }
orcamento-expandir = Expand { $categoria }
orcamento-arquivada = archived
orcamento-clique-realizado = Click Actual to see the transactions
orcamento-ver-lancamentos-categoria = See transactions for { $categoria } in { $mes }

## Where the income goes (spending groups)

orcamento-grupos-titulo = Where the income goes
orcamento-grupos-aria = Spending groups for { $mes }
orcamento-grupos-carregando = Calculating the groups…
orcamento-grupos-renda = Income this month
orcamento-grupos-total = In the four groups
orcamento-grupos-livre = Unassigned
orcamento-grupos-livre-dica = Income left after the four groups.
orcamento-grupos-pct-renda = { $pct } of income
orcamento-grupos-meta = target { $pct }
orcamento-grupos-sem-meta = no target
orcamento-grupos-acima = above target
orcamento-grupos-dentro = within target
orcamento-grupos-abaixo = below target
orcamento-grupos-atingida = target reached
orcamento-grupos-barra = { $grupo }: { $pct } of income
orcamento-grupos-sem-renda = No income this month: the share of income stays blank.
orcamento-grupos-futuro-nota = Future adds up contributions and transfers to goal accounts; withdrawals don't reduce it.
orcamento-grupos-ver-maiores = Show the largest in { $grupo }
orcamento-grupos-esconder-maiores = Hide the largest in { $grupo }
orcamento-grupos-maiores = Largest in { $grupo }
orcamento-grupos-nada = Nothing in this group this month.
orcamento-grupos-relatorios = See the trend in Reports
orcamento-grupos-categorias = Adjust the category groups
orcamento-grupos-editar-metas = Edit targets
orcamento-grupos-metas-titulo = Targets by group
orcamento-grupos-metas-descricao = How much of your income you want to go to each group. Leave it blank for no target. A common guide is 50% for essentials, 30% for non-essentials and 20% for the future.
orcamento-grupos-meta-de = { $grupo } target (% of income)
orcamento-grupos-soma = Sum of targets: { $pct }
orcamento-grupos-soma-acima = The sum is over 100% of income.
orcamento-grupos-padrao = Back to 50/30/20
orcamento-grupos-meta-invalida = Use a number from 0 to 100.
orcamento-grupos-metas-salvas = Group targets saved
