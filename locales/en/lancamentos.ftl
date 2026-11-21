# Screen 04 · Transactions: table, filters, bulk actions, row menu, transaction dialog, splits,
# recurrences, transfers between accounts and links to operations (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Field labels repeated across the screen

lancamentos-campo-data = Date
lancamentos-campo-descricao = Description
lancamentos-campo-conta = Account
lancamentos-campo-categoria = Category
lancamentos-campo-projeto = Project
lancamentos-campo-membro = Member
lancamentos-campo-status = Status
lancamentos-campo-valor = Amount
lancamentos-campo-tipo = Type
lancamentos-campo-tags = Tags
lancamentos-campo-notas = Notes
lancamentos-sem-descricao = no description
lancamentos-sem-descricao-parenteses = (no description)
lancamentos-outra-conta = another account
lancamentos-contrato = contract

## Header, tabs and command bar

lancamentos-descricao = Income, expenses and transfers. Filter, split, categorize in bulk, import statements and teach rules.
lancamentos-importar = Import
lancamentos-transferencia = Transfer
lancamentos-cambio = Currency exchange
lancamentos-novo = New transaction
lancamentos-abas = Transaction sections
lancamentos-aba-lancamentos = Transactions
lancamentos-aba-recorrencias = Recurrences
lancamentos-aba-regras = Rules
lancamentos-aba-importacoes = Imports
lancamentos-aba-caixa = Inbox
lancamentos-aba-email = Email

## Commands (search words are comma-separated)

lancamentos-cmd-novo-busca = expense,income,spending
lancamentos-cmd-receita = New income
lancamentos-cmd-receita-busca = inflow,salary
lancamentos-cmd-transferencia = New transfer between accounts
lancamentos-cmd-transferencia-busca = transfer,pix,investment,exchange
lancamentos-cmd-parcelado = Card purchase in installments
lancamentos-cmd-parcelado-busca = installments,card
lancamentos-cmd-importar = Import statement (OFX/CSV)
lancamentos-cmd-importar-busca = ofx,csv,bank,card bill
lancamentos-cmd-buscar-busca = find,filter
lancamentos-cmd-sem-categoria = Show uncategorized transactions
lancamentos-cmd-sem-categoria-busca = categorize,pending
lancamentos-cmd-recorrencia-busca = subscription,rent,monthly

## Action notices and "Undo"

lancamentos-transferencia-desligada = Transfer unlinked
lancamentos-transferencia-desligada-descricao = Both sides are now loose: link each one to the right match or change the type to income or expense.
lancamentos-erro-desligar = Could not unlink
lancamentos-erro-sugerir-regra = Could not suggest a rule
lancamentos-status-rotulo = Status: { $status }
lancamentos-n-alterados = { $n ->
        [one] { $n } transaction changed.
       *[other] { $n } transactions changed.
    }
lancamentos-erro-status = Could not change the status
lancamentos-n-marcados-toast = { $n ->
        [one] { $n } transaction marked
       *[other] { $n } transactions marked
    }
lancamentos-mesma-descricao-toast = Same description (“{ $descricao }”), across all dates. Categorize or change them in bulk in the bar above.
lancamentos-mesma-descricao-lista = Same description ({ $descricoes }), across all dates.
lancamentos-erro-marcar = Could not mark
lancamentos-transferencia-ligada = Transfer linked between the accounts
lancamentos-transferencia-ligada-descricao = Both sides are now linked.
lancamentos-erro-ligar = Could not link
lancamentos-feito = Done
lancamentos-desfeito = Undone
lancamentos-erro-desfazer = Could not undo
lancamentos-desfazer = Undo
lancamentos-dispensar = Dismiss
lancamentos-apagados = Transactions deleted

## Why two marked rows cannot be linked as a transfer

lancamentos-par-mesma-pagina = Mark both on the same page of the list.
lancamentos-par-ja-ligada = One of them is already linked to another account: unlink it first (row menu).
lancamentos-par-sentidos = To link them, one must be money out and the other money in.
lancamentos-par-mesma-conta = Both are in the same account: a transfer links different accounts.
lancamentos-par-valores = The amounts are different: in the same currency, what goes out must equal what comes in.
lancamentos-par-tipos = Only loose income, expenses and transfers (without splits or installments) can be linked.

## Table and pagination

lancamentos-atualizando = Updating
lancamentos-zero = 0 transactions
lancamentos-faixa = { $de }–{ $ate } of { $total }
lancamentos-totais = Filter totals
lancamentos-entradas = In
lancamentos-saidas = Out
lancamentos-resultado = Net
lancamentos-totais-nota = Totals for the whole filter, not just this page
lancamentos-carregando = Loading transactions
lancamentos-vazio-kicker = No transactions
lancamentos-vazio-titulo = Start with your bank statement
lancamentos-vazio-descricao = Import an OFX or CSV file, or enter them by hand. Transfers between your own accounts do not count as expenses.
lancamentos-importar-extrato = Import statement
lancamentos-paginas = Pages
lancamentos-anterior = ← Previous
lancamentos-proxima = Next →
lancamentos-pagina-de = page { $pagina } of { $paginas }
lancamentos-atalho-novo = new
lancamentos-atalho-buscar = search
lancamentos-atalho-importar = import
lancamentos-atalho-navegar = move
lancamentos-atalho-marcar = mark
lancamentos-atalho-marcar-faixa = mark range
lancamentos-atalho-categoria = category
lancamentos-atalho-abrir = open
lancamentos-apagar-1 = Delete 1 transaction
lancamentos-apagar-1-descricao = “{ $descricao }”. Transfers take the other side with them. Reconciled transactions cannot be deleted. You can undo this from the history.

lancamentos-op-carteira = portfolio
lancamentos-op-cotas = shares
lancamentos-op-meta = goal
lancamentos-ligada-a = Linked to { $conta }
lancamentos-ligada-perna = Linked to the other side in { $conta }
lancamentos-solta-dica = The other side is missing. Mark the money out and the money in and link them, or use the row menu.
lancamentos-transferencia-solta = Loose transfer
lancamentos-solta = loose
lancamentos-dividido-em = Split into { $n }
lancamentos-categoria-de = Category of { $descricao }
lancamentos-erro-categoria = Could not change the category
lancamentos-selecionar-pagina = Select all on this page
lancamentos-selecionar = Select { $descricao }
lancamentos-vencimento-fatura = Card bill due date (cash basis)
lancamentos-fat = bill { $data }
lancamentos-parcela-de = Installment { $numero } of { $contrato }
lancamentos-parc = inst. { $numero }
lancamentos-vinculado = Linked: { $operacao }
lancamentos-recorrencia-dica = Created by a recurrence
lancamentos-recorrente = Recurring
lancamentos-n-anexos = { $n ->
        [one] { $n } attachment
       *[other] { $n } attachments
    }
lancamentos-acoes-marcados = Actions for { $n } marked transactions
lancamentos-vazio-filtro = No transactions match these filters.
lancamentos-acoes-de = Actions for { $descricao }
lancamentos-sem-descricao-acoes = transaction without a description

## Filters

lancamentos-filtros = Transaction filters
lancamentos-buscar = Search transactions
lancamentos-buscar-placeholder = Description, payee or notes
lancamentos-filtro-receita = income
lancamentos-filtro-arquivada = archived
lancamentos-periodo = Period
lancamentos-periodo-aria = Period: { $periodo }
lancamentos-contas = Accounts
lancamentos-todas = All
lancamentos-categorias = Categories
lancamentos-membros = Members
lancamentos-projetos = Projects
lancamentos-faixa-valor = Amount range
lancamentos-base-datas = Date basis
lancamentos-competencia = Accrual
lancamentos-caixa = Cash
lancamentos-so-sem-categoria = Uncategorized only
lancamentos-fatura-selecionada = Selected card bill
lancamentos-lote-importacao = Import batch
lancamentos-limpar-filtros = Clear filters
lancamentos-tirar-filtro = Remove filter: { $filtro }
lancamentos-de = From
lancamentos-ate = To
lancamentos-aplicar-periodo = Apply period
lancamentos-valor-absoluto = Absolute amount: applies to money in and money out.
lancamentos-minimo = Minimum
lancamentos-maximo = Maximum
lancamentos-limpar = Clear
lancamentos-aplicar = Apply

lancamentos-periodo-tudo = All dates
lancamentos-periodo-este-mes = This month
lancamentos-periodo-mes-passado = Last month
lancamentos-periodo-3-meses = Last 3 months
lancamentos-periodo-12-meses = Last 12 months
lancamentos-periodo-este-ano = This year
lancamentos-periodo-personalizado = Custom

## Row menu and bulk menu

lancamentos-marcar-como = Mark as { $status }
lancamentos-menu-abrir = Open and edit
lancamentos-menu-categorizar = Categorize
lancamentos-menu-ver-divisao = See the split
lancamentos-menu-desmarcar = Unmark
lancamentos-menu-marcar-lote = Mark for bulk action
lancamentos-menu-mesma-descricao = Mark those with the same description
lancamentos-menu-projeto = Project: { $projeto }…
lancamentos-menu-por-projeto = Add to a project…
lancamentos-menu-transferencia-para = Transfer to another account…
lancamentos-menu-transferencia-de = Transfer from another account…
lancamentos-menu-desligar-de = Unlink from { $conta }
lancamentos-desligar-transferencia = Unlink the transfer
lancamentos-menu-pagamento-fatura = Card bill payment…
lancamentos-menu-parcela = Installment { $numero } of { $contrato }…
lancamentos-menu-pagamento-parcela = Contract installment payment…
lancamentos-menu-vinculo = Link: { $operacao }…
lancamentos-menu-vincular = Link to an operation…
lancamentos-menu-abrir-conta = Open the { $conta } account
lancamentos-menu-so-conta = Only this account's transactions
lancamentos-criar-regra-deste = Create a rule from this one
lancamentos-menu-apagar = Delete…
lancamentos-conciliado-nao-apaga = Reconciled transactions cannot be deleted.
lancamentos-menu-n-marcados = { $n ->
        [one] { $n } transaction marked
       *[other] { $n } transactions marked
    }
lancamentos-menu-categorizar-marcados = Categorize marked…
lancamentos-menu-membro = Member…
lancamentos-menu-tags = Tags…
lancamentos-menu-ligar-par = Link both as a transfer
lancamentos-menu-marcar-todos = Mark all { $n } in the filter
lancamentos-criar-regra = Create rule
lancamentos-menu-apagar-n = { $n ->
        [one] Delete { $n } transaction…
       *[other] Delete { $n } transactions…
    }

## Bulk action bar

lancamentos-acoes-lote = Actions on marked transactions
lancamentos-n-marcado = { $n } marked
lancamentos-mesma-descricao = Same description
lancamentos-todos-do-filtro = All { $n } in the filter
lancamentos-categorizar-marcados = Categorize marked
lancamentos-tirar-categoria = Remove the category
lancamentos-categoria-definida = Category set
lancamentos-categoria-removida = Category removed
lancamentos-erro-categorizar = Could not categorize
lancamentos-mudar-status-marcados = Change status of marked
lancamentos-projeto-reticencias = Project…
lancamentos-membro-marcados = Member of marked
lancamentos-sem-membro = No member
lancamentos-membro-definido = Member: { $nome }
lancamentos-membro-removido = Member removed
lancamentos-erro-membro = Could not change the member
lancamentos-tags-marcados = Tags of marked
lancamentos-tag = Tag
lancamentos-nova-tag = New tag
lancamentos-por = Add
lancamentos-tags-existentes = Existing tags
lancamentos-por-tag = Add the tag { $tag }
lancamentos-tirar-tag = Remove the tag { $tag }
lancamentos-tirar = Remove
lancamentos-tag-posta = Tag “{ $tag }” added
lancamentos-tag-tirada = Tag “{ $tag }” removed
lancamentos-erro-tags = Could not change the tags
lancamentos-ligar-como-transferencia = Link as a transfer
lancamentos-vincular-operacao = Link to an operation…
lancamentos-apagar-n = { $n ->
        [one] Delete { $n } transaction
       *[other] Delete { $n } transactions
    }
lancamentos-apagar-n-descricao = Transfers take the other side with them. Reconciled transactions cannot be deleted. You can undo this from the history.
lancamentos-tags-exemplo = trip, reimbursable…

## Transaction dialog

lancamentos-membro-da-conta = From the account ({ $nome })
lancamentos-familia = Family
lancamentos-erro-conta = Choose the account.
lancamentos-erro-data = Enter a valid date (mm/dd/yyyy).
lancamentos-erro-valor = Enter the amount (greater than zero).
lancamentos-erro-destino = Choose the destination account.
lancamentos-erro-mesma-conta = The source and destination accounts must be different.
lancamentos-erro-valor-destino = Enter how much goes into { $conta } ({ $moeda }).
lancamentos-erro-descricao-compra = Enter the purchase description.
lancamentos-erro-parcelas = The number of installments goes from 1 to 120.
lancamentos-erro-valor-ligado = Enter how much came in (or went out) in { $conta } ({ $moeda }).
lancamentos-erro-duas-partes = A split needs at least two parts.
lancamentos-erro-valor-parte = Enter the amount of each part.
lancamentos-erro-soma-partes = The parts must add up to the total.
lancamentos-tipo-registrado = { $tipo ->
        [investment] Investment recorded
        [card_payment] Card bill payment recorded
       *[other] Transfer recorded
    }
lancamentos-parcelada-registrada = Installment purchase recorded
lancamentos-criado = Transaction created
lancamentos-atualizado = Transaction updated
lancamentos-editar-tipo = Edit { $tipo }
lancamentos-editar = Edit transaction
lancamentos-salvando = Saving…
lancamentos-salvar-alteracoes = Save changes
lancamentos-lancar = Add
lancamentos-tipo-lancamento = Transaction type
lancamentos-parcelado-cartao = Card installments
lancamentos-de-origem = From (source)
lancamentos-para-destino = To (destination)
lancamentos-escolha-conta = Choose the account
lancamentos-valor-sai = Amount out ({ $moeda })
lancamentos-valor-entra = Amount in ({ $moeda })
lancamentos-cambio-implicito = Implied rate: 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $origem }
lancamentos-moedas-diferentes = Accounts in different currencies: enter the amount received.
lancamentos-descricao-transferencia-dica = Optional: the app writes “Transfer to …”.
lancamentos-cartao = Card
lancamentos-data-compra = Purchase date
lancamentos-exemplo-receita = Salary, refund…
lancamentos-exemplo-despesa = Groceries, pharmacy…
lancamentos-valor-total-moeda = Total amount ({ $moeda })
lancamentos-valor-moeda = Amount ({ $moeda })
lancamentos-parcelas-previa = { $n }× of { $valor }
lancamentos-parcelas-previa-primeira = { $n }× of { $valor } (1st of { $primeira })
lancamentos-parcelas = Installments
lancamentos-parcelas-dica = Future installments stay as planned.
lancamentos-como-transferencia-dica = Money moved to another account (the family's or someone else's). With a category, it counts in cash flow under that category; without one, it stays out of income and expenses.
lancamentos-tipo-do-lancamento = Type of transaction
lancamentos-para-conta-familia = To the family account
lancamentos-da-conta-familia = From the family account
lancamentos-ligar-dica = Optional: links to the side that already exists in that account (same amount, close date) or creates that side.
lancamentos-nenhuma-terceiros = None (someone else)
lancamentos-valor-em = Amount in { $conta } ({ $moeda })
lancamentos-destinatario = Recipient
lancamentos-favorecido = Payee
lancamentos-destinatario-dica = Optional: who it went to (or came from), when it is not a family account.
lancamentos-favorecido-dica = Optional: who received or paid.
lancamentos-projeto-dica = Construction, renovation, party or trip in 05 · Budget › Projects.
lancamentos-dividir = Split into categories or members
lancamentos-anexos = Attachments
lancamentos-anexos-dica = Receipts and invoices (PDF or image, up to 20 MB). Uploaded after the transaction is added.
lancamentos-anexo-salvar = Save { $nome }
lancamentos-anexo-salvo = Attachment saved
lancamentos-nenhum-anexo = No receipt attached.
lancamentos-anexar = Attach receipt
lancamentos-apagar-item = Delete { $nome }
lancamentos-editar-item = Edit { $nome }

## Split

lancamentos-divisao = Split
lancamentos-categoria-parte = Category of part { $n }
lancamentos-membro-parte = Member of part { $n }
lancamentos-valor-parte = Amount of part { $n }
lancamentos-tirar-parte = Remove part { $n }
lancamentos-adicionar-parte = Add part
lancamentos-restante-ultima = Put the rest in the last one
lancamentos-divisao-soma = Sum { $soma } of { $total }
lancamentos-divisao-faltam = { $valor } missing
lancamentos-divisao-sobram = { $valor } over
lancamentos-divisao-minimo = use at least two parts
lancamentos-divisao-confere = adds up ✓

## Recurrences (weekdays: abbreviated from Sunday to Saturday; on the checkboxes, Monday to Sunday)

lancamentos-dias-semana-abrev = Sun,Mon,Tue,Wed,Thu,Fri,Sat
lancamentos-dias-semana-caixas = Mon,Tue,Wed,Thu,Fri,Sat,Sun
lancamentos-erro-descricao = Enter the description.
lancamentos-erro-valor-simples = Enter the amount.
lancamentos-erro-inicio = Enter the start date.
lancamentos-recorrencia-atualizada = Recurrence updated
lancamentos-recorrencia-atualizada-descricao = Future planned transactions were redone.
lancamentos-recorrencia-criada = Recurrence created
lancamentos-recorrencia-criada-descricao = Planned transactions for the next 2 months are already showing.
lancamentos-editar-recorrencia = Edit recurrence
lancamentos-nova-recorrencia = New recurrence
lancamentos-criar-recorrencia = Create recurrence
lancamentos-recorrencia-exemplo = Rent, salary, gym…
lancamentos-repeticao = Repeat
lancamentos-frequencia = Frequency
lancamentos-freq-diaria = Daily
lancamentos-freq-semanal = Weekly
lancamentos-freq-mensal = Monthly
lancamentos-freq-anual = Yearly
lancamentos-a-cada = Every
lancamentos-dia-do-mes = Day of the month
lancamentos-dia-do-mes-dica = 31 = last day in short months.
lancamentos-mes = Month
lancamentos-dias-semana = Days of the week
lancamentos-inicio = Start
lancamentos-fim = End (optional)
lancamentos-ocorrencias = Occurrences (optional)
lancamentos-previa-datas = Preview of the next dates
lancamentos-previa = Preview
lancamentos-previa-vazia = Fill in the repeat settings to see the dates.
lancamentos-col-regra = Rule
lancamentos-col-proximas = Next
lancamentos-pausada = Paused
lancamentos-col-por-mes = Per month
lancamentos-acoes = Actions
lancamentos-recorrencia-pausada = Recurrence paused
lancamentos-recorrencia-retomada = Recurrence resumed
lancamentos-erro-alterar = Could not change it
lancamentos-pausar = Pause
lancamentos-retomar = Resume
lancamentos-comprometido = Committed per month ({ $moeda })
lancamentos-recorrencias-vazio-kicker = No recurrences
lancamentos-recorrencias-vazio-titulo = Add your salary, rent and subscriptions
lancamentos-recorrencias-vazio-descricao = Recurrences create planned transactions for the next two months, which count in the projected balance and in the budget.
lancamentos-apagar-recorrencia = Delete recurrence
lancamentos-apagar-recorrencia-descricao = “{ $descricao }” stops creating transactions. Those already cleared stay, without the link. You can undo this from the history.
lancamentos-recorrencia-apagada = Recurrence deleted
lancamentos-previstos-removidos = { $n ->
        [one] { $n } planned transaction removed.
       *[other] { $n } planned transactions removed.
    }
lancamentos-manter-previstos = Keep the planned transactions already created

## Link to another account (transfer or card bill payment)

lancamentos-pagamento-fatura = Card bill payment
lancamentos-transferencia-para = Transfer to another account
lancamentos-transferencia-de = Transfer from another account
lancamentos-fatura-ligada = Card bill payment linked to the card
lancamentos-ligado-a = Linked to “{ $descricao }” in { $conta }.
lancamentos-criado-em = Transaction created in { $conta }.
lancamentos-ligar-cartao = Link to the card
lancamentos-ligar-contas = Link the accounts
lancamentos-saiu-em = went out of { $conta }
lancamentos-entrou-em = came into { $conta }
lancamentos-saiu-de = went out of { $conta }
lancamentos-nenhum-cartao = No card added. Add the card in 03 · Accounts to link the payment to its bill.
lancamentos-nenhuma-outra-conta = No other active account.
lancamentos-para-a-conta = To account
lancamentos-da-conta = From account
lancamentos-o-que-acontece = What will happen
lancamentos-procurando = Looking for the matching transaction in { $conta }…
lancamentos-achei-entrada = Found the incoming transaction in { $conta }: { $data } · “{ $descricao }”. Both will be linked.
lancamentos-achei-saida = Found the outgoing transaction in { $conta }: { $data } · “{ $descricao }”. Both will be linked.
lancamentos-crio-entrada = There is no { $valor } transaction in { $conta } around { $data }: I will create the incoming one there, with the same date and description.
lancamentos-crio-saida = There is no { $valor } transaction in { $conta } around { $data }: I will create the outgoing one there, with the same date and description.
lancamentos-informe-entrou = Enter how much came into { $conta }.
lancamentos-informe-saiu = Enter how much went out of { $conta }.
lancamentos-vira-fatura = Becomes a card bill payment: leaves expenses (the card purchases already count).
lancamentos-quita-fatura = Pays off the bill due on { $data }.
lancamentos-vira-aporte = Becomes an investment or withdrawal at the broker: stays out of income and expenses.
lancamentos-vira-transferencia = Becomes a transfer between family accounts: stays out of income and expenses.

## Suggested transfers (pairs that arrived loose)

lancamentos-aporte-ou-resgate = Investment or withdrawal
lancamentos-ligar-par-rotulo = Link { $de } → { $para } on { $data }
lancamentos-sempre-ligar = always link
lancamentos-sai-em = Out on { $data }: { $descricao }
lancamentos-entra-em = In on { $data }: { $descricao }
lancamentos-nao-e = Not a transfer
lancamentos-pares-aviso = { $n ->
        [one] 1 pair of transactions looks like a transfer between your accounts
       *[other] { $n } pairs of transactions look like transfers between your accounts
    }
lancamentos-pares-aviso-nota = · unlinked, the money out counts as an expense and the money in as income again.
lancamentos-revisar = Review
lancamentos-revisao-titulo = Transfers between your accounts
lancamentos-revisao-descricao = Pairs with the same amount that leave one account and arrive in another within 4 days. Once linked, they become transfers: they leave income and expenses (reports, budget, financial health) and lose their category.
lancamentos-ligar-sozinho = Automatically link future ones between these accounts
lancamentos-ligar-sozinho-descricao = When importing statements, pairs like these come in already linked.
lancamentos-agora-nao = Not now
lancamentos-n-transferencias-ligadas = { $n ->
        [one] 1 transfer linked
       *[other] { $n } transfers linked
    }
lancamentos-ligar-n-pares = { $n ->
        [one] Link 1 pair
       *[other] Link { $n } pairs
    }
lancamentos-pares-sugeridos = Suggested pairs

## Link to an operation (portfolio, capital shares, goal)

lancamentos-operacao-carteira = Portfolio
lancamentos-operacao-cotas = Capital shares
lancamentos-operacao-meta = Goal
lancamentos-bloqueio-transferencia = It is a transfer between your accounts: unlink the transfer to link it.
lancamentos-bloqueio-previsto = The transaction is still planned: clear it before linking.
lancamentos-bloqueio-dividido = Split transactions cannot be linked to an operation.
lancamentos-bloqueio-parcela = The transaction pays an installment of { $contrato }.
lancamentos-um-contrato = a contract
lancamentos-operacoes = Operations
lancamentos-vinculado-toast = Transaction linked
lancamentos-vinculado-descricao = { $operacao }. It left income and expenses and became an investment.
lancamentos-vinculo-desfeito = Link removed
lancamentos-vinculo-desfeito-descricao = The transaction went back to its previous type and category.
lancamentos-vincular-titulo = Link to an operation
lancamentos-vincular = Link
lancamentos-desvincular = Unlink
lancamentos-vinculado-nota = Counts as an investment ({ $tipo }), outside income and expenses. Unlinking restores the previous type and category.
lancamentos-procurando-operacoes = Looking for operations…
lancamentos-nenhuma-operacao = No operation matches. Record the investment in { $carteira } (within 7 days of the date, similar amount), turn on capital shares in the cooperative's account or create the goal, and come back here.
lancamentos-situacao = Status
lancamentos-entra = ✓ Included
lancamentos-fica-de-fora = Left out: { $motivo }
lancamentos-n-vinculados = { $n ->
        [one] { $n } transaction linked
       *[other] { $n } transactions linked
    }
lancamentos-vinculados-descricao = { $operacao }. They left income and expenses and became investments.
lancamentos-vinculados-descricao-fora = { $operacao }. { $fora ->
        [one] { $fora } was left out.
       *[other] { $fora } were left out.
    } They left income and expenses and became investments.
lancamentos-confirmar-vinculo = Confirm the link
lancamentos-n-marcados-destino = { $n ->
        [one] { $n } transaction marked
       *[other] { $n } transactions marked
    } → { $destino }
lancamentos-n-marcados-escolha = { $n ->
        [one] { $n } transaction marked
       *[other] { $n } transactions marked
    }: choose the target operation.
lancamentos-vincular-n = { $n ->
        [one] Link { $n } transaction
       *[other] Link { $n } transactions
    }
lancamentos-conferindo = Checking the transactions…
lancamentos-destino = Target:
lancamentos-total = Total:
lancamentos-n-ficam-de-fora = { $n ->
        [one] { $n } transaction is left out
       *[other] { $n } transactions are left out
    }
lancamentos-outros-vinculados = The others are linked; these stay as they are.
lancamentos-nenhum-vinculavel = None of the marked transactions can be linked to this operation.
lancamentos-a-vincular = Transactions to link
lancamentos-vinculados-nota = Once linked, they leave income and expenses and become investments, without a category. You can undo this from the history or unlink them one by one.
lancamentos-nenhuma-operacao-marcados = No operation matches the marked transactions. Portfolio buys and sells link to one transaction at a time (row menu); for several, use a goal or capital shares.

lancamentos-ajuste-saldo = Balance adjustment
lancamentos-ajustar = Adjust balance

## Cópias entre contas

lancamentos-copias-aviso = { $n ->
    [one] { $n } transaction repeats a purchase already recorded in another account
   *[other] { $n } transactions repeat purchases already recorded in another account
}
lancamentos-copias-aviso-nota = Same bank identifier: usually one account's file imported into another. They count twice in the cash flow and in "uncategorized".
lancamentos-copias-titulo = Copies between accounts
lancamentos-copias-descricao = Each row is a transaction recorded in two accounts with the same bank identifier. Check the copies to delete; the one in the right account stays (the card's, for purchases). You can undo it from the history.
lancamentos-copias-lista = Copies found
lancamentos-copia-rotulo = Delete the copy in { $conta } on { $data }: { $descricao }
lancamentos-copia-lado = { $lado }: { $conta }, { $data } · { $descricao }
lancamentos-copia-apagar = Copy
lancamentos-copia-fica = Stays
lancamentos-copia-transferencia = This copy is linked as a transfer to another transaction: deleting it unlinks them and the other side goes back to expense or income.
lancamentos-apagar-n-copias = { $n ->
    [one] Delete { $n } copy
   *[other] Delete { $n } copies
}
lancamentos-n-copias-apagadas = { $n ->
    [one] { $n } copy deleted
   *[other] { $n } copies deleted
}
lancamentos-copias-saldo-refeito = { $n ->
    [one] The balance adjustment that counted them was redone.
   *[other] The { $n } balance adjustments that counted them were redone.
}
lancamentos-copias-erro = Could not delete the copies
lancamentos-resgate = Withdrawal
lancamentos-sem-categoria-aporte = Investment contribution
lancamentos-sem-categoria-resgate = Investment withdrawal
lancamentos-sem-categoria-aporte-ajuda = Contributions and withdrawals do not use a category: the money moves within your net worth and stays out of the cash flow and the budget. The Sankey counts money leaving the bank for the brokerage (contribution) and coming back (withdrawal); purchases and sales inside the brokerage are not counted again.
lancamentos-n-alterados-n-fora = { $n } changed; { $fora } contribution(s), withdrawal(s), card bill payment(s) or adjustment(s) were left without a category, as they don't use one
lancamentos-campo-cartao = Card used
lancamentos-cartao-dica = The purchase goes to whoever uses this card, unless you choose the member.
lancamentos-cartao-nao-informado = Not given
lancamentos-campo-moeda-origem = Purchase currency
lancamentos-moeda-origem-dica = Purchase abroad: the currency and amount the merchant charged.
lancamentos-campo-valor-origem = Amount in the purchase currency
