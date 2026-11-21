# Shared controls (features/transactions/ui): checkboxes, switches, multi-select, category picker,
# query state, error messages and the labels of core types used across screens
# (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Errors, files and query state

controles-erro-generico = Something went wrong.
controles-erro-ler-arquivo = could not read the file
controles-tamanho-kb = { NUMBER($n, maximumFractionDigits: 1) } KB
controles-tamanho-mb = { NUMBER($n, maximumFractionDigits: 1) } MB
controles-carregando = Loading
controles-tentar-de-novo = Try again
controles-ligado = On
controles-desligado = Off

## Multi-select and search

controles-todos = All
controles-n-selecionados = { $n } selected
controles-buscar-em = Search { $rotulo }
controles-buscar = Search…
controles-nada-encontrado = Nothing found.
controles-limpar-selecao = Clear selection

## Category and description

controles-categoria = Category
controles-sem-categoria = Uncategorized
controles-buscar-rotulo = Search { $rotulo }
controles-buscar-categoria = Search categories…
controles-nenhuma-categoria = No categories.
controles-sugestoes = Transaction suggestions

## Account types

controles-conta-corrente = Checking account
controles-conta-poupanca = Savings account
controles-conta-dinheiro = Cash
controles-conta-cartao = Credit card
controles-conta-corretora = Brokerage
controles-conta-exterior = Foreign account
controles-conta-beneficio = Meal and grocery vouchers
controles-conta-fgts = FGTS (Brazil)
controles-conta-aposentadoria = Retirement (401(k), IRA, PPR…)
controles-conta-outra = Other

## Groups on the Accounts screen

controles-grupo-correntes = Checking accounts
controles-grupo-poupanca = Savings
controles-grupo-dinheiro = Cash
controles-grupo-cartoes = Cards
controles-grupo-corretoras = Brokerages
controles-grupo-exterior = Abroad
controles-grupo-beneficio = Meal and grocery vouchers
controles-grupo-fgts = FGTS (Brazil)
controles-grupo-aposentadoria = Retirement accounts
controles-grupo-outras = Other

## Institution types

controles-instituicao-banco = Bank
controles-instituicao-corretora = Broker
controles-instituicao-emissor = Card issuer
controles-instituicao-outra = Other

## Transaction types and statuses

controles-tipo-receita = Income
controles-tipo-despesa = Expense
controles-tipo-transferencia = Transfer
controles-tipo-pagamento-fatura = Card bill payment
controles-tipo-aporte = Investment
controles-tipo-ajuste = Adjustment

controles-status-previsto = Planned
controles-status-efetivado = Cleared
controles-status-conciliado = Reconciled

## Card statement status and imported row status

controles-fatura-aberta = Open
controles-fatura-fechada = Closed
controles-fatura-paga = Paid
controles-fatura-parcial = Partly paid

controles-linha-nova = New
controles-linha-duplicada = Duplicate
controles-linha-possivel-duplicada = Possible duplicate
controles-linha-erro = Has errors
controles-linha-ignorada = Ignored

## Backup types

controles-backup-diario = Daily automatic
controles-backup-manual = Manual
controles-backup-migracao = Before a database update
controles-backup-importacao = Before a large import
controles-backup-ia = Before an AI action
controles-backup-restauracao = Before restoring
controles-backup-senha = Before changing the password
controles-backup-fechar = When closing the app

## Who made the change

controles-autor-voce = You
controles-autor-importacao = Import
controles-autor-ia = AI
controles-autor-sistema = System

## Rule fields and operators

controles-campo-descricao = Description
controles-campo-favorecido = Payee
controles-campo-notas = Notes
controles-campo-valor = Amount
controles-campo-conta = Account
controles-campo-tipo = Type

controles-op-contem = contains
controles-op-igual = equals
controles-op-comeca = starts with
controles-op-termina = ends with
controles-op-regex = matches the regex
controles-op-entre = between
controles-op-maior-igual = greater than or equal to
controles-op-menor-igual = less than or equal to

## Core tables in the change history

controles-tabela-familia = Family
controles-tabela-membro = Member
controles-tabela-instituicao = Institution
controles-tabela-conta = Account
controles-tabela-membro-conta = Account member
controles-tabela-cartao = Card details
controles-tabela-fatura = Card statement
controles-tabela-categoria = Category
controles-tabela-tag = Tag
controles-tabela-lancamento = Transaction
controles-tabela-parte-divisao = Split part
controles-tabela-tag-lancamento = Transaction tag
controles-tabela-anexo = Attachment
controles-tabela-parcelamento = Installment plan
controles-tabela-recorrencia = Recurrence
controles-tabela-regra = Rule
controles-tabela-lote-importacao = Import batch
controles-tabela-cambio = Exchange rate
controles-tabela-orcamento = Budget
controles-tabela-linha-orcamento = Budget line
controles-tabela-meta = Goal
controles-tabela-conta-meta = Goal account
controles-tabela-projeto = Project
controles-tabela-ativo = Asset
controles-tabela-operacao = Trade
controles-tabela-tela-screener = Screener view

## Counts per table in the history

controles-n-familia = { $n ->
        [one] { $n } family
       *[other] { $n } families
    }
controles-n-membro = { $n ->
        [one] { $n } member
       *[other] { $n } members
    }
controles-n-instituicao = { $n ->
        [one] { $n } institution
       *[other] { $n } institutions
    }
controles-n-conta = { $n ->
        [one] { $n } account
       *[other] { $n } accounts
    }
controles-n-membro-conta = { $n ->
        [one] { $n } account member
       *[other] { $n } account members
    }
controles-n-cartao = { $n ->
        [one] { $n } card details
       *[other] { $n } card details
    }
controles-n-fatura = { $n ->
        [one] { $n } card statement
       *[other] { $n } card statements
    }
controles-n-categoria = { $n ->
        [one] { $n } category
       *[other] { $n } categories
    }
controles-n-tag = { $n ->
        [one] { $n } tag
       *[other] { $n } tags
    }
controles-n-lancamento = { $n ->
        [one] { $n } transaction
       *[other] { $n } transactions
    }
controles-n-parte-divisao = { $n ->
        [one] { $n } split part
       *[other] { $n } split parts
    }
controles-n-tag-lancamento = { $n ->
        [one] { $n } transaction tag
       *[other] { $n } transaction tags
    }
controles-n-anexo = { $n ->
        [one] { $n } attachment
       *[other] { $n } attachments
    }
controles-n-parcelamento = { $n ->
        [one] { $n } installment plan
       *[other] { $n } installment plans
    }
controles-n-recorrencia = { $n ->
        [one] { $n } recurrence
       *[other] { $n } recurrences
    }
controles-n-regra = { $n ->
        [one] { $n } rule
       *[other] { $n } rules
    }
controles-n-lote-importacao = { $n ->
        [one] { $n } import batch
       *[other] { $n } import batches
    }
controles-n-cambio = { $n ->
        [one] { $n } exchange rate
       *[other] { $n } exchange rates
    }
controles-n-orcamento = { $n ->
        [one] { $n } budget
       *[other] { $n } budgets
    }
controles-n-linha-orcamento = { $n ->
        [one] { $n } budget line
       *[other] { $n } budget lines
    }
controles-n-meta = { $n ->
        [one] { $n } goal
       *[other] { $n } goals
    }
controles-n-conta-meta = { $n ->
        [one] { $n } goal account
       *[other] { $n } goal accounts
    }
controles-n-projeto = { $n ->
        [one] { $n } project
       *[other] { $n } projects
    }
controles-n-ativo = { $n ->
        [one] { $n } asset
       *[other] { $n } assets
    }
controles-n-operacao = { $n ->
        [one] { $n } trade
       *[other] { $n } trades
    }
controles-n-tela-screener = { $n ->
        [one] { $n } screener view
       *[other] { $n } screener views
    }
