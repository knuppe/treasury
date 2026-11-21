# Budget › Projects: construction work, renovations, parties and trips with a budget, expenses
# and income, the timeline, highlights and entries; setup, items entered by hand and
# transactions added to a project (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Status and repeated labels

projetos-status-planejado = Planned
projetos-status-andamento = In progress
projetos-status-concluido = Done
projetos-titulo = Projects
projetos-projeto = Project
projetos-o-projeto = project
projetos-projeto-nome = Project { $nome }
projetos-novo = New project
projetos-novo-reticencias = New project…
projetos-sem-projeto = No project
projetos-nome = Name
projetos-bem = Asset
projetos-nenhum = None
projetos-orcamento = Budget
projetos-situacao = Status
projetos-observacoes = Notes
projetos-tipo = Type
projetos-descricao = Description
projetos-valor = Amount
projetos-data = Date
projetos-despesas = Expenses
projetos-receitas = Income
projetos-entradas = Entries
projetos-sem-descricao = no description
projetos-hoje = today
projetos-editar = Edit { $nome }
projetos-excluir = Delete
projetos-acoes-de = Actions for { $nome }
projetos-tirar = Remove from project
projetos-por = Add to project

## List and detail

projetos-ver-lancamentos = View transactions
projetos-lancamentos = Transactions
projetos-novo-item = New item
projetos-abrir = Open project
projetos-novo-item-menu = New item…
projetos-por-lancamentos-menu = Add transactions…
projetos-editar-menu = Edit…
projetos-excluir-menu = Delete…
projetos-vazio-kicker = No projects
projetos-vazio-titulo = Group the expenses and income of a construction job, renovation, party or trip
projetos-vazio-descricao = A project adds up the expenses and income linked to it (transactions from your accounts and items entered by hand) and compares them with the budget. Linked to an asset, it feeds its improvements: the cost of the home or car follows what the work cost.
projetos-situacao-lista = Project status
projetos-todos-n = All ({ $n })
projetos-abertos-n = Open ({ $n })
projetos-concluidos-n = Done ({ $n })
projetos-nenhum-aberto = No open projects: see the finished ones.
projetos-nenhum-concluido = No finished projects.
projetos-sem-bem = no asset
projetos-periodo = Period
projetos-gasto = Spent
projetos-resta = Left
projetos-excluir-titulo = Delete { $nome }
projetos-excluir-descricao = The transactions and improvements stay, they just leave the project. You can undo it from the history.
projetos-excluir-projeto = Delete project
projetos-excluido = Project deleted

## Timeline, figures and highlights

projetos-por-dia = By day
projetos-por-mes = By month
projetos-linha-tempo = Timeline
projetos-visao-linha = Timeline view
projetos-acumulado = Cumulative
projetos-grafico-acumulado = Cumulative expenses and income of { $nome } from { $de } to { $ate }, with the result.
projetos-grafico-acumulado-orcamento = Cumulative expenses and income of { $nome } from { $de } to { $ate }, with the result and the budget.
projetos-grafico-periodo-dia = Income and expenses of { $nome } by day, with the cumulative result.
projetos-grafico-periodo-mes = Income and expenses of { $nome } by month, with the cumulative result.
projetos-montando = Building the timeline…
projetos-linha-vazia = No entries yet. { $lance } or use “Transactions” to mark the ones from your accounts.
projetos-linha-vazia-lance = Enter an expense or income by hand
projetos-maior-despesa = Biggest expense
projetos-maior-receita = Biggest income
projetos-dia-maior-gasto = Day with the most spending
projetos-mes-maior-gasto = Month with the most spending
projetos-orcamento-acaba = Budget runs out
projetos-orcamento-acaba-em = in { $mes }, at the current pace
projetos-acima-orcamento = over budget
projetos-resultado = Result
projetos-resultado-dica = income − expenses
projetos-passou = over by { $valor }
projetos-resta-valor = { $valor } left
projetos-sem-orcamento = no budget
projetos-pct-orcamento = { $pct }% of the budget
projetos-gasto-mes = Spending per month
projetos-media-meses =
    { $n ->
        [one] average over { $n } month
       *[other] average over { $n } months
    }
projetos-entradas-dica =
    { $lancamentos ->
        [one] { $lancamentos } transaction
       *[other] { $lancamentos } transactions
    } · { $itens ->
        [one] { $itens } item
       *[other] { $itens } items
    }
projetos-destaques = Highlights
projetos-alimenta =
    { $n ->
        [one] Feeds { $n } improvement of { $bem }: the asset's cost follows the project's expenses (income does not reduce it).
       *[other] Feeds { $n } improvements of { $bem }: the asset's cost follows the project's expenses (income does not reduce it).
    }
projetos-um-bem = an asset
projetos-por-categoria = By category
projetos-despesas-categoria = Expenses of { $nome } by category
projetos-nenhuma-despesa = No expenses yet.
projetos-receitas-categoria = Income of { $nome } by category
projetos-nenhuma-receita = No income.

## Project entries

projetos-entrada-sem-descricao = Entry without description
projetos-entrada-sem-descricao-min = entry without description
projetos-editar-item = Edit item…
projetos-editar-lancamento = Edit transaction…
projetos-ver-em-lancamentos = View in Transactions
projetos-excluir-item = Delete item…
projetos-tirar-menu = Remove from project…
projetos-n-entradas-selecionadas = { $n } entries selected
projetos-tirar-n = Remove { $n } entries from the project…
projetos-entradas-de = Entries of { $nome }
projetos-acoes-n-entradas = Actions for { $n } entries
projetos-entradas-vazio = No entries. Enter an { $item } or mark { $lancamentos }.
projetos-entradas-vazio-item = item
projetos-entradas-vazio-lancamentos = transactions
projetos-origem = Source
projetos-previsto = planned
projetos-item-do-projeto = Project item
projetos-n-entradas =
    { $n ->
        [one] { $n } entry
       *[other] { $n } entries
    }
projetos-excluir-item-titulo = Delete project item
projetos-tirar-lancamento-titulo = Remove transaction from project
projetos-tirar-n-titulo = Remove { $n } entries from the project
projetos-tirar-itens-descricao = The item only exists in the project and will be deleted. You can undo it from the history.
projetos-tirar-misto-descricao = The transactions stay in the accounts, just without the project; items entered by hand are deleted. You can undo it from the history.
projetos-tirar-lancamento-descricao = The transaction stays in the account, just without the project. You can undo it from the history.
projetos-entradas-tiradas = Entries removed from the project
projetos-entradas-tiradas-descricao =
    { $n ->
        [one] { $n } entry from { $nome }.
       *[other] { $n } entries from { $nome }.
    }

## Project setup

projetos-salvo = Project saved
projetos-criado = Project created
projetos-form-descricao = Group the expenses and income of a construction job, renovation, party or trip. Linked to an asset, the project feeds its improvements.
projetos-criar = Create project
projetos-nome-exemplo = Bathroom renovation, car bodywork…
projetos-bem-dica = Home, vehicle or other asset of the work (optional).
projetos-orcamento-dica = How much you plan to spend (optional).
projetos-inicio = Start
projetos-fim = End

## Adding transactions to a project

projetos-lanc-no-projeto = Transactions in the project
projetos-lanc-fora = Transactions out of the project
projetos-n-lancamentos =
    { $n ->
        [one] { $n } transaction.
       *[other] { $n } transactions.
    }
projetos-titulo-lancamentos =
    { $n ->
        [one] Project of the transaction
       *[other] Project for { $n } transactions
    }
projetos-link-descricao = The project's spending is the sum of its transactions; improvements linked to the project take that spending as their amount.
projetos-nome-novo = Name of the new project
projetos-nome-novo-exemplo = Kitchen renovation
projetos-add-titulo = Transactions for { $nome }
projetos-add-descricao = Mark the project's expenses (purchases, PIX, statements) and income. Transactions from another project move to this one.
projetos-add-marcados = { $n } marked · { $valor }
projetos-add-toast = Transactions in { $nome }
projetos-add-toast-descricao =
    { $n ->
        [one] { $n } transaction in the project.
       *[other] { $n } transactions in the project.
    }
projetos-buscar = Search
projetos-buscar-exemplo = Description, payee or note
projetos-de = From
projetos-ate = To
projetos-entradas-saidas = Money in and out
projetos-so-saidas = Money out only
projetos-so-entradas = Money in only
projetos-encontrados = Transactions found
projetos-buscando = Searching…
projetos-nenhum-filtro = No transactions in this filter.
projetos-em-projeto = in { $nome }

## Item entered by hand

projetos-item-receita-em = Income in { $nome }.
projetos-item-despesa-em = Expense in { $nome }.
projetos-item-salvo = Item saved
projetos-lancamento-criado = Transaction created
projetos-item-lancado = Item entered
projetos-item-novo-em = New item in { $nome }
projetos-item-editar-descricao = Item entered by hand in the project, outside the accounts.
projetos-item-descricao = An expense or income of the project. Project only doesn't touch any account; in an account, it becomes a transaction there, already in the project.
projetos-lancar-conta = Enter in the account
projetos-lancar-projeto = Enter in the project
projetos-item-exemplo-receita = Selling leftovers, help from family…
projetos-item-exemplo-despesa = Builder, materials…
projetos-onde-lancar = Where to enter it
projetos-onde-conta = Counts in the account's balance and reports.
projetos-onde-projeto = Project only: outside the accounts and reports.
projetos-so-no-projeto = Project only
