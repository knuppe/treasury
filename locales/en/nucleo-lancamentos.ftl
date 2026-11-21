# Transactions, recurring items, rules and categories (core: `transactions`, `recurrence`,
# `rules`, `categories`): validation errors, history reasons, suggestion reasons and labels the
# core builds on the fly, in the installation language. The default category names live in
# categorias.ftl; texts matched against bank statements (built-in rules) stay in the code.
# Revision: AI draft, not yet reviewed by a native speaker.

## Common

nucleo-lancamentos-sem-descricao = (no description)
nucleo-lancamentos-ref-lancamento = transaction { $id }
nucleo-lancamentos-ref-membro = member { $id }
nucleo-lancamentos-ref-tag = tag { $id }
nucleo-lancamentos-ref-anexo = attachment { $id }
nucleo-lancamentos-erro-nome-vazio = { $coisa ->
    [conta] enter the account name
    [categoria] enter the category name
    [tag] enter the tag name
    [membro] enter the member's name
    [familia] enter the family name
    [instituicao] enter the institution name
    [renda] enter the income name
   *[outro] enter the name
}
nucleo-lancamentos-erro-nome-longo = { $coisa ->
    [conta] the account name can have at most { $max } characters
    [categoria] the category name can have at most { $max } characters
    [tag] the tag name can have at most { $max } characters
    [membro] the member's name can have at most { $max } characters
    [familia] the family name can have at most { $max } characters
    [instituicao] the institution name can have at most { $max } characters
    [renda] the income name can have at most { $max } characters
   *[outro] the name can have at most { $max } characters
}
nucleo-lancamentos-erro-cor = invalid color: { $cor }
nucleo-lancamentos-erro-periodo-invertido = the start date is after the end date
nucleo-lancamentos-erro-maximo-lote = at most { $max } transactions at a time

## Types and statuses (lowercase, for the middle of a sentence)

nucleo-lancamentos-tipo-receita = income
nucleo-lancamentos-tipo-despesa = expense
nucleo-lancamentos-tipo-transferencia = transfer
nucleo-lancamentos-tipo-pagamento-fatura = card bill payment
nucleo-lancamentos-tipo-investimento = investment contribution/withdrawal
nucleo-lancamentos-tipo-ajuste = adjustment
nucleo-lancamentos-status-previsto = planned
nucleo-lancamentos-status-efetivado = cleared
nucleo-lancamentos-status-conciliado = reconciled

## History (reason for each change)

nucleo-lancamentos-historico-criado = Transaction created: { $descricao }
nucleo-lancamentos-historico-alterado = Transaction changed: { $descricao }
nucleo-lancamentos-historico-apagado = Transaction deleted: { $descricao }
nucleo-lancamentos-historico-apagados = { $n } transactions deleted
nucleo-lancamentos-historico-copias-apagadas = { $n } cross-account copies deleted
nucleo-lancamentos-historico-transferencia = Transfer: { $origem } → { $destino }
nucleo-lancamentos-historico-pagamento-fatura = Card bill payment: { $origem } → { $destino }
nucleo-lancamentos-historico-aporte = Investment: { $origem } → { $destino }
nucleo-lancamentos-historico-categoria-definida = { $n ->
    [one] Category { $categoria } set on { $n } transaction
   *[other] Category { $categoria } set on { $n } transactions
}
nucleo-lancamentos-historico-categoria-removida = { $n ->
    [one] Category removed from { $n } transaction
   *[other] Category removed from { $n } transactions
}
nucleo-lancamentos-historico-status = { $n ->
    [one] Status changed to { $status } on { $n } transaction
   *[other] Status changed to { $status } on { $n } transactions
}
nucleo-lancamentos-historico-membro-definido = { $n ->
    [one] Member { $membro } set on { $n } transaction
   *[other] Member { $membro } set on { $n } transactions
}
nucleo-lancamentos-historico-membro-removido = { $n ->
    [one] Member removed from { $n } transaction
   *[other] Member removed from { $n } transactions
}
nucleo-lancamentos-historico-tags = { $n ->
    [one] Tags changed on { $n } transaction
   *[other] Tags changed on { $n } transactions
}
nucleo-lancamentos-historico-par-ligado = Transactions linked as a transfer between accounts
nucleo-lancamentos-historico-pares-ligados = { $n } pairs linked as transfers between accounts
nucleo-lancamentos-historico-vinculados = { $n ->
    [one] { $n } transaction linked to { $operacao }
   *[other] { $n } transactions linked to { $operacao }
}
nucleo-lancamentos-historico-vinculado = Transaction linked: { $descricao } → { $operacao }
nucleo-lancamentos-historico-desvinculado = Link removed: { $descricao }
nucleo-lancamentos-historico-transferencia-desligada = Transfer unlinked: both legs are now loose
nucleo-lancamentos-historico-tag-criada = Tag created: { $nome }
nucleo-lancamentos-historico-tag-alterada = Tag changed
nucleo-lancamentos-historico-tag-apagada = Tag deleted: { $nome }
nucleo-lancamentos-historico-anexo-adicionado = Attachment added: { $descricao }
nucleo-lancamentos-historico-anexo-apagado = Attachment deleted: { $descricao }

## Transactions

nucleo-lancamentos-erro-nenhum-selecionado = no transaction selected
nucleo-lancamentos-erro-escolha-um = choose at least one transaction
nucleo-lancamentos-erro-informe-lancamento = enter the transaction
nucleo-lancamentos-erro-tags-lote = enter the tags to add or remove
nucleo-lancamentos-erro-tipo-invalido = invalid transaction type: { $valor }
nucleo-lancamentos-erro-status-invalido = invalid status: { $valor }
nucleo-lancamentos-erro-valor-zero = the transaction amount can't be zero
nucleo-lancamentos-erro-categoria-arquivada = the category { $nome } is archived
nucleo-lancamentos-erro-tipo-sem-categoria = { $tipo } transactions don't use a category
nucleo-lancamentos-erro-categoria-de-outro-tipo = the category { $nome } is for { $categoria ->
    [receita] income
   *[despesa] expenses
} and the transaction is { $lancamento ->
    [receita] income
   *[despesa] an expense
}
nucleo-lancamentos-erro-tipo-sem-divisao = { $tipo } transactions can't be split
nucleo-lancamentos-erro-lote-sem-categoria = contributions, card bill payments and adjustments do not use a category: they move net worth and stay out of the cash flow and the budget ({ $n } selected)
nucleo-lancamentos-erro-divisao-partes = a split needs at least two parts
nucleo-lancamentos-erro-divisao-soma = the sum of the parts ({ $soma }) differs from the total ({ $total })
nucleo-lancamentos-erro-divisao-sinal = the parts of the split must have the same sign as the total and can't be zero
nucleo-lancamentos-erro-divisao-desatualizada = { $erro }; send the updated split along (or an empty list to remove the split)
nucleo-lancamentos-erro-pagamento-duas-pernas = card bill payments have two legs; use transactions.create_transfer
nucleo-lancamentos-erro-conciliado = this transaction is reconciled; unreconcile it (change the status to cleared) before changing the amount, date or account
nucleo-lancamentos-erro-outra-perna-conciliada-previsto = the other leg of the transfer is reconciled; unreconcile it before going back to planned
nucleo-lancamentos-erro-outra-perna-conciliada = the other leg of the transfer is reconciled; unreconcile it before changing the amount or date
nucleo-lancamentos-erro-apagar-conciliado = the transaction "{ $descricao }" on { $data } is reconciled; unreconcile it before deleting
nucleo-lancamentos-erro-conta-outra-moeda = the chosen account uses { $moeda_conta } and the transaction is in { $moeda }; create a new transaction in that account
nucleo-lancamentos-erro-pagamento-destino-cartao = the destination of a card bill payment must be a card account
nucleo-lancamentos-erro-perna-mudar-tipo = you can't change the type of a transfer leg; delete the transfer and create another transaction
nucleo-lancamentos-erro-perna-sinal = in a transfer the source leg is negative and the destination leg is positive; keep the sign
nucleo-lancamentos-erro-pernas-mesma-conta = the two legs of the transfer can't be in the same account
nucleo-lancamentos-erro-transferencia-dividida = transfers can't be split
nucleo-lancamentos-erro-virar-pagamento-fatura = to turn it into a card bill payment, link it to the card leg (suggested transfers)
nucleo-lancamentos-erro-ja-ligado = this transaction is already linked to another account
nucleo-lancamentos-erro-recusar-so-interface = transfer suggestions can only be dismissed in the app

## Tags and attachments

nucleo-lancamentos-erro-tag-repetida = there's already a tag called { $nome }
nucleo-lancamentos-erro-base64 = invalid base64 content
nucleo-lancamentos-erro-arquivo-nome = enter the file name
nucleo-lancamentos-erro-arquivo-nome-longo = file name too long
nucleo-lancamentos-erro-arquivo-tipo = invalid file type: { $tipo }
nucleo-lancamentos-erro-anexo-grande = the attachment is over the { $mb } MB limit
nucleo-lancamentos-erro-arquivo-vazio = the file is empty

## Transfers

# Default descriptions of a new transfer (saved as the transaction's description).
nucleo-lancamentos-descricao-pagamento-fatura = { $conta } bill payment
nucleo-lancamentos-descricao-pagamento-recebido = Payment received from { $conta }
nucleo-lancamentos-descricao-aporte-em = Investment in { $conta }
nucleo-lancamentos-descricao-aporte-vindo = Investment from { $conta }
nucleo-lancamentos-descricao-transferencia-para = Transfer to { $conta }
nucleo-lancamentos-descricao-transferencia-de = Transfer from { $conta }
nucleo-lancamentos-erro-transferencia-mesma-conta = the source and destination accounts must be different
nucleo-lancamentos-erro-transferencia-valor = enter the transfer amount as a positive number
nucleo-lancamentos-erro-transferencia-mesma-moeda = accounts in the same currency: the amount coming in must equal the amount going out
nucleo-lancamentos-erro-transferencia-outra-moeda = the source is in { $moeda_origem } and the destination in { $moeda_destino }; enter the amount that comes into the destination (to_amount)
nucleo-lancamentos-erro-pagamento-so-cartao = a card bill payment can only have a card account as its destination
nucleo-lancamentos-erro-pagamento-de-cartao = the bill must be paid from an account that isn't a card
nucleo-lancamentos-erro-transferencia-tipo = transfers can't be of type { $tipo }
# Why a pair shows up in the suggested transfers (joined with "; ").
nucleo-lancamentos-motivo-mesmo-valor-mesmo-dia = same amount on the same day
nucleo-lancamentos-motivo-mesmo-valor-dias = { $dias ->
    [one] same amount { $dias } day apart
   *[other] same amount { $dias } days apart
}
nucleo-lancamentos-motivo-texto-transferencia = transfer wording (PIX, TED…)
nucleo-lancamentos-motivo-nome-outra-conta = the other account's name appears in the description
nucleo-lancamentos-erro-par-ja-transferencia = one of the transactions is already part of a transfer
nucleo-lancamentos-erro-par-tipo = only income, expenses and loose transfers can become a transfer
nucleo-lancamentos-erro-par-mesma-conta = the two legs must be in different accounts
nucleo-lancamentos-erro-par-sinal = the source leg must be money going out and the destination leg money coming in
nucleo-lancamentos-erro-par-valor = accounts in the same currency: the amount going out must equal the amount coming in
nucleo-lancamentos-erro-par-dividido = split transactions can't become a transfer
nucleo-lancamentos-erro-outra-conta = choose an account other than the transaction's
nucleo-lancamentos-erro-outra-conta-moeda = the other account uses { $moeda }: enter how much came in (or went out) there
nucleo-lancamentos-erro-nao-ligado = this transaction isn't linked to another account
nucleo-lancamentos-erro-ligacao-historico = this link ({ $tipo }) can't be undone here; use the history to undo it
nucleo-lancamentos-erro-nenhum-par = no pair selected

## Links to operations (portfolio, capital shares, goals)

nucleo-lancamentos-operacao = operation
nucleo-lancamentos-operacao-apagada = deleted operation
nucleo-lancamentos-operacao-aporte = Investment in { $ativo } (bought { $quantidade } on { $data })
nucleo-lancamentos-operacao-resgate = Withdrawal from { $ativo } (sold { $quantidade } on { $data })
nucleo-lancamentos-operacao-cotas-capital = Capital shares · { $conta }
nucleo-lancamentos-operacao-meta = Goal { $nome }
nucleo-lancamentos-ref-operacao-carteira = portfolio operation
nucleo-lancamentos-ref-cotas-capital = capital shares in that account
nucleo-lancamentos-ref-meta = goal
# Why the operation shows up as a candidate.
nucleo-lancamentos-motivo-operacao = { $valor ->
    [igual] same amount
   *[parecido] similar amount
} { $dias ->
    [0] on the same day
    [one] { $dias } day apart
   *[other] { $dias } days apart
}; { $diferenca } difference
nucleo-lancamentos-motivo-texto-cota-capital = the statement text mentions capital shares
nucleo-lancamentos-motivo-conta-cotas-capital = the account has capital shares
nucleo-lancamentos-motivo-texto-meta = the statement text mentions the goal
nucleo-lancamentos-motivo-poupanca-programada = the text mentions scheduled savings
nucleo-lancamentos-motivo-meta-ativa = active goal
nucleo-lancamentos-motivo-combina-com = { $motivo }; matches { $n } of { $total }
nucleo-lancamentos-erro-vinculado = the transaction is linked to { $operacao }; unlink it first
nucleo-lancamentos-erro-ja-vinculado = the transaction is already linked to { $operacao }; unlink it first
nucleo-lancamentos-erro-sem-valor = transaction without an amount
nucleo-lancamentos-erro-vincular-previsto = the transaction is still planned; clear it before linking
nucleo-lancamentos-erro-vincular-transferencia = the transaction is a transfer leg; unlink the transfer first
nucleo-lancamentos-erro-vincular-tipo = only income, expenses, loose transfers and investments can be linked to an operation
nucleo-lancamentos-erro-vincular-dividido = split transactions can't be linked to an operation
nucleo-lancamentos-erro-vincular-parcela = the transaction pays an installment of { $divida }; unlink it there first
nucleo-lancamentos-erro-vincular-caixa-corretora = the transaction is already the cash side of that operation at the broker
nucleo-lancamentos-erro-operacao-ocupada = that operation already has a linked transaction; unlink the other one first
nucleo-lancamentos-erro-aporte-saida = an investment (purchase) is paid with money going out (negative amount)
nucleo-lancamentos-erro-resgate-entrada = a withdrawal (sale) arrives as money coming in (positive amount)
nucleo-lancamentos-erro-meta-arquivada = the goal is archived
nucleo-lancamentos-erro-vinculo-nao-gravado = link not saved
nucleo-lancamentos-erro-nao-vinculado = the transaction isn't linked to an operation
nucleo-lancamentos-erro-operacao-um-lancamento = a portfolio operation takes only one transaction
nucleo-lancamentos-erro-nenhum-vinculavel = none of the transactions can be linked to { $operacao } ({ $motivo })

## Recurring items

nucleo-lancamentos-ref-recorrencia = recurring item { $id }
nucleo-lancamentos-historico-recorrencia-criada = Recurring item created: { $descricao }
nucleo-lancamentos-historico-recorrencia-alterada = Recurring item changed: { $descricao }
nucleo-lancamentos-historico-recorrencia-apagada = Recurring item deleted: { $descricao }
nucleo-lancamentos-historico-previstos = Planned transactions from recurring items
nucleo-lancamentos-historico-parcelas-efetivadas = Card installments cleared
# Who made the change in the history ("System · recurring items").
nucleo-lancamentos-autor-recorrencias = recurring items
nucleo-lancamentos-autor-cartoes = cards
nucleo-lancamentos-recorrencia-erro-horizonte = the horizon must be between 0 and { $max } days
nucleo-lancamentos-recorrencia-erro-valor-zero = the recurring amount can't be zero
nucleo-lancamentos-recorrencia-erro-destino-so-transferencia = a destination account only applies to recurring transfers
nucleo-lancamentos-recorrencia-erro-informe-destino = enter the destination account of the recurring transfer
nucleo-lancamentos-recorrencia-erro-moedas = recurring transfers between different currencies ({ $origem } → { $destino }) aren't supported
nucleo-lancamentos-recorrencia-erro-pagamento-cartao = a recurring card bill payment needs a card as its destination
nucleo-lancamentos-recorrencia-erro-transferencia-categoria = transfers don't use a category
nucleo-lancamentos-recorrencia-erro-ajuste = adjustments can't be recurring
nucleo-lancamentos-recorrencia-erro-descricao = enter the description of the recurring item
nucleo-lancamentos-recorrencia-erro-fim-antes = the end date is before the start date
nucleo-lancamentos-recorrencia-erro-sem-destino = recurring transfer without a destination account

## Repeat rule (RRULE)

nucleo-lancamentos-rrule-erro-byday-posicao = BYDAY with a position ({ $valor }) isn't supported; use only the weekday (e.g. MO)
nucleo-lancamentos-rrule-erro-dia-semana = invalid weekday: { $valor }
nucleo-lancamentos-rrule-erro-valor = invalid value in { $parte }: { $valor }
nucleo-lancamentos-rrule-erro-parte = invalid part in the rule: { $parte }
nucleo-lancamentos-rrule-erro-frequencia = unsupported frequency: { $valor } (use DAILY, WEEKLY, MONTHLY or YEARLY)
nucleo-lancamentos-rrule-erro-varios-dias = BYMONTHDAY with several days isn't supported; create one recurring item per day
nucleo-lancamentos-rrule-erro-varios-meses = BYMONTH with several months isn't supported; create one recurring item per month
nucleo-lancamentos-rrule-erro-parte-nao-suportada = unsupported rule part: { $parte } (the end of the recurrence is the end date)
nucleo-lancamentos-rrule-erro-sem-freq = the rule needs FREQ
nucleo-lancamentos-rrule-erro-intervalo = the interval must be between 1 and { $max }
nucleo-lancamentos-rrule-erro-ocorrencias = the number of occurrences must be between 1 and { $max }
nucleo-lancamentos-rrule-erro-dia-mes = the day of the month must be between 1 and 31
nucleo-lancamentos-rrule-erro-dia-mes-frequencia = the day of the month only applies to monthly or yearly recurrences
nucleo-lancamentos-rrule-erro-mes = the month must be between 1 and 12
nucleo-lancamentos-rrule-erro-mes-frequencia = the month only applies to yearly recurrences
nucleo-lancamentos-rrule-erro-sem-dias = choose at least one weekday
nucleo-lancamentos-rrule-erro-dias-frequencia = weekdays only apply to daily or weekly recurrences
# Readable description: "every month on day 5", "every 2 weeks on Mondays, 3 times". Days in the
# plural; the gender only matters in Portuguese.
nucleo-lancamentos-rrule-dia-mo = Mondays
nucleo-lancamentos-rrule-dia-tu = Tuesdays
nucleo-lancamentos-rrule-dia-we = Wednesdays
nucleo-lancamentos-rrule-dia-th = Thursdays
nucleo-lancamentos-rrule-dia-fr = Fridays
nucleo-lancamentos-rrule-dia-sa = Saturdays
nucleo-lancamentos-rrule-dia-su = Sundays
nucleo-lancamentos-rrule-lista = { $inicio } and { $ultimo }
nucleo-lancamentos-rrule-dia-com-artigo = { $genero ->
    [f] { $dia }
   *[m] { $dia }
}
nucleo-lancamentos-rrule-dias = { $genero ->
    [f] on { $lista }
    [m] on { $lista }
   *[misto] on { $lista }
}
nucleo-lancamentos-rrule-diaria = { $n ->
    [1] every day
   *[other] every { $n } days
}
nucleo-lancamentos-rrule-semanal = { $n ->
    [1] every week
   *[other] every { $n } weeks
}
nucleo-lancamentos-rrule-mensal = { $n ->
    [1] every month
   *[other] every { $n } months
}
nucleo-lancamentos-rrule-anual = { $n ->
    [1] every year
   *[other] every { $n } years
}
nucleo-lancamentos-rrule-dia-util = every weekday (Monday to Friday)
nucleo-lancamentos-rrule-so-dias = { $base }, only { $dias }
nucleo-lancamentos-rrule-nos-dias = { $base } { $dias }
nucleo-lancamentos-rrule-ultimo-dia = { $base } on the last day
nucleo-lancamentos-rrule-no-dia = { $base } on day { $dia }
nucleo-lancamentos-rrule-em-data = { $base } on { $mes } { $dia }
nucleo-lancamentos-rrule-em-mes = { $base } in { $mes }
nucleo-lancamentos-rrule-ultimo-dia-mes = { $base } on the last day of the month
nucleo-lancamentos-rrule-vezes = { $n ->
    [1] { $regra }, once
   *[other] { $regra }, { $n } times
}
nucleo-lancamentos-rrule-ate = { $regra }, until { $data }

## Categorization rules

nucleo-lancamentos-ref-regra = rule { $id }
nucleo-lancamentos-historico-regra-criada = Rule created: { $nome }
nucleo-lancamentos-historico-regra-alterada = Rule changed: { $nome }
nucleo-lancamentos-historico-regra-excluida = Rule deleted: { $nome }
nucleo-lancamentos-historico-regras-aplicadas = Categorization rules applied
nucleo-lancamentos-historico-regras-prontas = Built-in category rules (iFood, Netflix, Uber…)
nucleo-lancamentos-historico-regras-prontas-pix = PIX, TED and DOC become transfers
nucleo-lancamentos-historico-regras-prontas-restaurantes = Built-in rules: restaurants and meal and food voucher credits
nucleo-lancamentos-historico-regras-prontas-encargos = Built-in rules: card interest and charges
nucleo-lancamentos-autor-regras-prontas = built-in rules
# Name of the rule suggested from a transaction (becomes the rule's name if saved).
nucleo-lancamentos-regra-nome-sugerido = Description contains “{ $trecho }”
nucleo-lancamentos-regra-erro-texto = the condition needs a text
nucleo-lancamentos-regra-erro-centavos = the condition amount must be a whole number, in cents (e.g. -5000 = { $exemplo })
nucleo-lancamentos-regra-erro-numero = the amount condition needs a number
nucleo-lancamentos-regra-erro-regex = invalid regular expression: { $erro }
nucleo-lancamentos-regra-erro-texto-vazio = the condition text needs letters or numbers
nucleo-lancamentos-regra-erro-op-texto = text conditions use contains, equals, starts with, ends with or regex
nucleo-lancamentos-regra-erro-entre = the "between" condition needs value2
nucleo-lancamentos-regra-erro-op-valor = amount conditions use equals, between, greater than or equal, less than or equal
nucleo-lancamentos-regra-erro-op-conta = the account condition uses "equals" with the account id
nucleo-lancamentos-regra-erro-op-tipo = the type condition uses "equals" with the transaction type
nucleo-lancamentos-regra-erro-sem-condicao = the rule needs at least one condition
nucleo-lancamentos-regra-erro-nome = give the rule a name
nucleo-lancamentos-regra-erro-sem-acao = the rule needs at least one action (category, member, tags, rename or type)
nucleo-lancamentos-regra-erro-renomear = the new name can't be empty
nucleo-lancamentos-regra-erro-tags = tags can't be empty
nucleo-lancamentos-regra-erro-sem-palavras = the transaction description has no words to build a rule from

## Categories

nucleo-lancamentos-ref-categoria = category { $id }
nucleo-lancamentos-ref-categoria-pai = parent category { $id }
nucleo-lancamentos-historico-categorias-traduzidas = Default categories translated
nucleo-lancamentos-historico-categorias-sugeridas = Suggested categories added
nucleo-lancamentos-historico-categoria-criada = Category created: { $nome }
nucleo-lancamentos-historico-categoria-alterada = Category changed
nucleo-lancamentos-historico-categoria-apagada = Category deleted: { $nome }
nucleo-lancamentos-historico-categoria-apagada-reatribuida = Category deleted and reassigned: { $nome }
nucleo-lancamentos-historico-categorias-juntadas = Categories merged: { $origem } → { $destino }
nucleo-lancamentos-historico-categorias-grupo = { $n ->
        [one] 1 category moved to the { $grupo } group
       *[other] { $n } categories moved to the { $grupo } group
    }
nucleo-lancamentos-categoria-erro-repetida = there's already a category called { $nome } at this level
nucleo-lancamentos-categoria-erro-tipo-pai = the subcategory must be the same type as the parent category ({ $pai } is { $tipo ->
    [receita] an income
   *[despesa] an expense
} category)
nucleo-lancamentos-categoria-erro-ciclo = a category can't be inside itself or one of its subcategories
nucleo-lancamentos-categoria-erro-grupo-receita = income categories have no spending group (essentials, debts, future, non-essential)
nucleo-lancamentos-categoria-erro-mesmo-destino = choose a different category to receive the transactions
nucleo-lancamentos-categoria-erro-juntar-tipos = { $origem } is { $tipo_origem ->
    [receita] an income
   *[despesa] an expense
} category and { $destino } is { $tipo_destino ->
    [receita] an income
   *[despesa] an expense
} category; you can only merge categories of the same type
nucleo-lancamentos-categoria-erro-juntar-sub = { $destino } is a subcategory of { $origem }; choose a category outside it
nucleo-lancamentos-categoria-erro-tem-subcategorias = { $n ->
    [one] { $nome } has { $n } subcategory; delete, move or merge it first
   *[other] { $nome } has { $n } subcategories; delete, move or merge them first
}
nucleo-lancamentos-categoria-erro-em-uso = { $nome } is in use ({ $lancamentos } transaction(s), { $divisoes } split(s), { $orcamento } budget line(s), { $regras } rule(s), { $recorrencias } recurring item(s), { $parcelamentos } installment plan(s)); choose a category to take over that use
nucleo-lancamentos-erro-anexo-so-interface = saving an attachment to disk is only possible from the app interface
nucleo-lancamentos-erro-copias-so-interface = deleting cross-account copies is only possible from the app interface
nucleo-lancamentos-erro-origem-mesma-moeda = the original currency is the account's own; leave the original amount empty
nucleo-lancamentos-erro-origem-valor = the amount in the original currency can't be zero
nucleo-lancamentos-erro-origem-par = enter the original amount and currency together
