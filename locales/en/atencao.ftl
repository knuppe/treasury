# "To solve" (core `attention` and the interface): the single list of what needs the person and
# the line that says until when the numbers are complete (docs/to-resolve.md).

## Core

atencao-erro-item = This item cannot be dismissed.
atencao-erro-so-interface = This can only be done in Treasury's interface.
atencao-venceu-titulo = { $descricao } was due on { $data }
atencao-venceu-corpo = { $valor } in { $conta } is still marked as planned. If it was paid, mark it as paid or import the statement; if not, pay it to avoid interest.
atencao-caixa-titulo = { $n ->
    [one] 1 file waiting in the inbox
   *[other] { $n } files waiting in the inbox
}
atencao-caixa-corpo = Files that arrived in the inbox folder and have not been imported yet.
atencao-vendas-sem-compra-titulo = { $n ->
    [one] 1 sale without a recorded purchase
   *[other] { $n } sales without a recorded purchase
}
atencao-vendas-sem-compra-corpo = The sale was imported, but the purchase is older than the history Treasury has. It stays out of the average cost, the result and the tax until you record the purchase or accept it without a cost.


## Core: embedded yearly tables

tabelas-inss = INSS (floor and ceiling)
tabelas-pensao-pt = Segurança Social (IAS and minimum pensions)
tabelas-pensao-us = Social Security (bend points)
atencao-regras-titulo = { $n ->
    [one] 1 category rule ready to create
   *[other] { $n } category rules ready to create
}
atencao-regras-corpo = You have already given the same category three times to { $nomes }. With the rule, the next transactions come in with the category.
atencao-tabelas-titulo = { $n ->
    [one] 1 of the app's tables may be out of date
   *[other] { $n } of the app's tables may be out of date
}
atencao-tabelas-item = { $nome } for { $ano }
atencao-tabelas-corpo = In use: { $tabelas }. Estimates still work, with that year's values.
atencao-tabelas-acao = Update Treasury to get the new tables.

## Interface: dashboard card and top bar bell

atencao-titulo = To solve
atencao-origem = computed by the app, without AI
atencao-vazio = All up to date. Nothing to solve right now.
atencao-resolver = Mark as solved
atencao-dispensar-dica = Leaves the list now. If it still applies, it comes back in a week.
atencao-ver-todos = See all { $n }
atencao-ver-alertas = See all alerts
atencao-sino = { $n ->
    [0] Nothing to solve
    [one] 1 item to solve
   *[other] { $n } items to solve
}
atencao-gravidade-critical = Urgent
atencao-gravidade-warning = Attention
atencao-gravidade-info = Notice

## Interface: until when the numbers are complete

atencao-cobertura-completa = Accounts up to date: data until { $data }.
atencao-cobertura-completa-sem-data = Accounts up to date.
atencao-cobertura-parcial = { $n ->
    [one] 1 account without a recent statement ({ $contas }).
   *[other] { $n } accounts without a recent statement ({ $contas }).
} Totals may be incomplete.
atencao-cobertura-mais = { $nomes } and { $n } more
atencao-cobertura-importar = Import statement

## Core: your next step (the ladder)

degraus-know-titulo = Know where the money goes
degraus-balance-titulo = Spend less than you earn
degraus-credit-titulo = Get out of expensive credit
degraus-reserve-titulo = Build the emergency fund
degraus-debts-titulo = Make debts light
degraus-invest-titulo = Invest and protect what you have
degraus-know-sem-contas = There are no accounts yet.
degraus-know-sem-contas-acao = Add the account you use the most. You can start with just that one.
degraus-know-meses = { $n ->
    [0] There is no complete month of transactions yet.
    [one] There is 1 complete month of transactions.
   *[other] There are { $n } complete months of transactions.
} With { $alvo }, Treasury can tell where the money goes.
degraus-know-meses-acao = Import the statements of the last months or keep writing down your spending.
degraus-know-categorias = { $pct } of the transactions of the last 90 days have a category. From { $alvo } on, the totals by category become reliable.
degraus-know-categorias-acao = { $n ->
    [one] Give a category to the one transaction that is missing it.
   *[other] Give a category to the { $n } transactions that are missing one.
}
degraus-balance-sem-renda = { $n ->
    [one] There is no income recorded in the last month
   *[other] There is no income recorded in the last { $n } months
}, so it is not possible to tell whether money is left over or missing.
degraus-balance-sem-renda-acao = Record the salary and other income, or import the statement of the account where they arrive.
degraus-balance-acima = { $n ->
    [one] In the last month
   *[other] In the last { $n } months
}, { $pct } more went out than came in{ $n ->
    [one] .
   *[other] { $meses ->
        [one] , and in 1 of them spending was above income.
       *[other] , and in { $meses } of them spending was above income.
    }
}
degraus-balance-acima-acao = Look in the budget at the categories that grew the most and pick one to reduce this month.
degraus-credit-rotativo = In recent months, the card statement was not paid in full in { $rotativo ->
    [one] 1 of them
   *[other] { $rotativo } of them
}. Revolving card credit is among the most expensive kinds of credit.
degraus-credit-cheque = In recent months, the account was in the red in { $cheque ->
    [one] 1 of them
   *[other] { $cheque } of them
}. Overdraft is among the most expensive kinds of credit.
degraus-credit-ambos = In recent months there was a card statement not paid in full and an account in the red. Revolving credit and overdraft are among the most expensive kinds of credit.
degraus-credit-acao = Swap this debt for cheaper credit and see the payoff plan in Debts.
degraus-reserve-essenciais = The emergency fund covers { $meses } months of essential spending. The next milestone is { $alvo ->
    [one] 1 month
   *[other] { $alvo } months
}.
degraus-reserve-gastos = The emergency fund covers { $meses } months of spending. The next milestone is { $alvo ->
    [one] 1 month
   *[other] { $alvo } months
}.
degraus-reserve-acao = Set aside an amount every month, somewhere you can withdraw from on the same day, until you get there.
degraus-debts-peso = Loan and financing installments take { $pct } of income. Above { $alvo }, the budget gets tight.
degraus-debts-acao = Simulate in the payoff plan what happens when you pay a little extra each month on the most expensive debt.
degraus-invest-porque = The months close in the black, there is no expensive credit, the emergency fund is built and debts fit within income.
degraus-invest-comecar-acao = What goes beyond the emergency fund can go to longer-term goals. Create a goal and set how much to save each month.
degraus-invest-carteira-acao = Check the portfolio's allocation, concentration and fee costs.

## Core: closing the month

fechamento-erro-mes = Only a month that has already ended can be closed.
fechamento-erro-passo = Unknown month-closing step.
fechamento-erro-incompleto = There are still steps left before closing this month.
fechamento-statements-titulo = Bring in the statements
fechamento-statements-falta = { $n ->
    [one] 1 account without a recent statement: { $contas }.
   *[other] { $n } accounts without a recent statement: { $contas }.
}
fechamento-categories-titulo = Categorize the transactions
fechamento-categories-falta = { $n ->
    [one] 1 transaction of the month without a category.
   *[other] { $n } transactions of the month without a category.
}
fechamento-balances-titulo = Check the balances
fechamento-balances-falta = Compare each account's balance with the bank's and mark it when they match.
fechamento-review-titulo = Solve what is pending
fechamento-review-falta = { $n ->
    [one] 1 item in To solve.
   *[other] { $n } items in To solve.
}
fechamento-summary-titulo = Read the month's summary
fechamento-summary-falta = See in the reports where the money went and mark it when you have read it.

## Interface: next step and closing the month

degraus-cartao = Your next step
degraus-posicao = Step { $n } of { $total }
degraus-estimado = Provisional reading: some accounts have no recent statement.
degraus-feito = done
degraus-atual = now
degraus-depois = later
fechamento-cartao = Close { $mes }
fechamento-progresso = { $n } of { $total }
fechamento-origem = five steps, without AI
fechamento-marcar = Mark as done
fechamento-desmarcar = Unmark
fechamento-fechar = Mark the month as closed
fechamento-falta = There are steps left before closing the month.
fechamento-passo-feito = Done
fechamento-passo-pendente = Pending
atencao-compras-titulo = { $n } grocery purchases this month without items
atencao-compras-corpo = Add the items you want to follow (meat, coffee, milk): each product's price and your own basket inflation come from them. It does not have to be the whole receipt.

atencao-nao-avisar = Stop reminding this one
atencao-nao-avisar-dica = For expenses you pay without needing a reminder, such as a cleaner. It stays in the forecast and the budget; turn the reminder back on with the bell in Transactions › Recurrences.
atencao-nao-avisar-feito = Done: this recurring item no longer reminds its due date.
atencao-nao-avisar-erro = Could not turn off the reminder
