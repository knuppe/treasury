# Accounts, cards and currency exchange (core: `accounts`, `cards`, `fx`): validation errors,
# history reasons, balance sync hints and texts the core saves when it creates adjustments, card
# bill payments and exchange operations, in the installation language.
# Revision: AI draft, not yet reviewed by a native speaker.

## Accounts

nucleo-contas-ref-conta = account { $id }
nucleo-contas-ref-conta-pagadora = paying account { $id }
nucleo-contas-ref-instituicao = institution { $id }
nucleo-contas-historico-criada = Account created: { $nome }
nucleo-contas-historico-alterada = Account changed: { $nome }
nucleo-contas-historico-apagada = Account deleted: { $nome }
nucleo-contas-historico-conciliacao = Account { $nome } reconciled through { $data }
nucleo-contas-erro-tipo-invalido = invalid account type: { $valor }
nucleo-contas-erro-tipo-so-brasil = This account type only exists in Brazil: the vault must be set to Brazil.
nucleo-contas-erro-historico-longo = period too long for daily points (at most { $max } days); use interval = month
nucleo-contas-erro-agencia = branch: use only numbers and hyphens
nucleo-contas-erro-numero-conta = account number: use only numbers and hyphens
nucleo-contas-erro-cheque-especial-tipo = an overdraft only applies to a checking account
nucleo-contas-erro-apagar-lancamentos = { $n ->
    [one] the account { $nome } has { $n } transaction; delete it first or archive the account
   *[other] the account { $nome } has { $n } transactions; delete them first or archive the account
}
nucleo-contas-erro-apagar-recorrencias = { $n ->
    [one] the account { $nome } is used by { $n } recurring item; delete it first
   *[other] the account { $nome } is used by { $n } recurring items; delete them first
}
nucleo-contas-erro-apagar-parcelamentos = { $n ->
    [one] the account { $nome } has { $n } installment plan; delete it first
   *[other] the account { $nome } has { $n } installment plans; delete them first
}
# Closing the account (accounts/closing.rs).
nucleo-contas-historico-encerrada = Account closed: { $nome }
nucleo-contas-historico-reaberta = Account reopened: { $nome }
nucleo-contas-erro-encerrada = The account { $nome } was closed on { $data }. To record anything after that date, reopen the account.
nucleo-contas-encerrar-erro-futuro = The closing date cannot be after today.
nucleo-contas-encerrar-erro-antes-abertura = The closing date is before the account was opened ({ $data }).
nucleo-contas-encerrar-erro-ja-encerrada = The account was already closed on { $data }.
nucleo-contas-encerrar-erro-saldo = The account still has a balance of { $valor } on that date. Transfer the balance or record the adjustment before closing.
nucleo-contas-encerrar-erro-depois = { $n ->
    [one] There is { $n } transaction after that date (the last one on { $data }). Pick a date on or after it.
   *[other] There are { $n } transactions after that date (the last one on { $data }). Pick a date on or after it.
}
nucleo-contas-encerrar-erro-posicoes = The account still holds { $ativos }. Record the sale or the custody transfer first.
nucleo-contas-encerrar-erro-parcelas = { $n ->
    [one] The card still has { $n } installment to come.
   *[other] The card still has { $n } installments to come.
}
# Why the OFX didn't find the account on its own.
nucleo-contas-ofx-sem-numero = the OFX doesn't include the account number
nucleo-contas-ofx-sem-cartao = no card ends in those digits; fill in the card's last digits in its settings
nucleo-contas-ofx-sem-conta = no account has that number; fill in the bank, branch and account in its settings
nucleo-contas-ofx-varias = more than one account matches; choose which one

## Credit card

nucleo-contas-cartao-erro-fechamento = the closing day must be between 1 and 31
nucleo-contas-cartao-erro-vencimento = the due day must be between 1 and 31
nucleo-contas-cartao-erro-limite = the card limit can't be negative
nucleo-contas-cartao-erro-paga-propria = the card can't pay its own bill
nucleo-contas-cartao-erro-pagadora-cartao = the account that pays the bill can't be another card
nucleo-contas-cartao-erro-final = enter the card's last 4 digits (numbers only)
nucleo-contas-cartao-erro-dados = enter the card details (closing day and due day)
nucleo-contas-cartao-erro-so-cartao = card details only apply to credit card accounts
nucleo-contas-cartao-erro-nao-cartao = the chosen account isn't a credit card
nucleo-contas-cartao-erro-sem-ciclo = the card has no closing day and due day

## Bills and installments

nucleo-contas-ref-fatura = bill { $id }
nucleo-contas-ref-parcelamento = installment plan { $id }
nucleo-contas-historico-pagamento-fatura = { $cartao } bill payment due on { $vencimento }
nucleo-contas-historico-total-informado = Bill total entered
nucleo-contas-historico-movidos-fatura = { $n ->
    [one] { $n } transaction moved to another bill
   *[other] { $n } transactions moved to another bill
}
nucleo-contas-historico-compra-parcelada = Purchase in { $n } installments: { $descricao }
nucleo-contas-historico-parcelamento-apagado = Installment plan deleted: { $descricao }
# Description of the bill payment (saved on the transaction); $mes as 10/2026.
nucleo-contas-fatura-descricao-pagamento = { $cartao } bill payment ({ $mes })
nucleo-contas-fatura-erro-pagadora = enter the account that pays the bill
nucleo-contas-fatura-erro-moeda = the paying account is in { $moeda_conta } and the card in { $moeda_cartao }; use an account in the card's currency
nucleo-contas-fatura-erro-valor-positivo = the payment amount must be positive
nucleo-contas-fatura-erro-nada-a-pagar = this bill has nothing to pay
nucleo-contas-fatura-erro-destino = enter the destination bill (statement_id, or account_id and reference_month)
nucleo-contas-fatura-erro-outro-cartao = the transaction "{ $descricao }" isn't from this card
nucleo-contas-parcelas-erro-numero = the number of installments must be between 1 and 120
nucleo-contas-parcelas-erro-total = enter the purchase total (positive)
nucleo-contas-parcelas-erro-menor = the total is less than the number of installments
nucleo-contas-parcelas-erro-descricao = enter the purchase description
nucleo-contas-parcelas-erro-conciliada = the installment "{ $descricao }" on { $data } is reconciled; unreconcile it before deleting the installment plan

## Overdraft

nucleo-contas-cheque-erro-limite = the overdraft limit can't be negative
nucleo-contas-cheque-erro-indexador = the annual and the monthly rate have different indexes
nucleo-contas-cheque-erro-numero = { $campo }: invalid number ({ $valor })
nucleo-contas-cheque-erro-negativo = { $campo } can't be negative
nucleo-contas-cheque-erro-acima = { $campo } above { $max }%: please check
nucleo-contas-cheque-campo-taxa-mes = monthly rate
nucleo-contas-cheque-campo-desconto = discount
nucleo-contas-cheque-campo-taxa-ano = yearly rate
nucleo-contas-cheque-campo-iof-dia = basic daily IOF
nucleo-contas-cheque-campo-iof-adicional = additional IOF
nucleo-contas-cheque-campo-cet-mes = monthly total cost (CET)
nucleo-contas-cheque-campo-cet-ano = yearly total cost (CET)

## Capital shares

nucleo-contas-ref-saldo-cotas = capital shares balance
nucleo-contas-historico-cotas-capital = Capital shares: { $nome }
nucleo-contas-historico-cotas-removidas = Capital shares removed
nucleo-contas-historico-cotas-saldo = Capital shares balance
nucleo-contas-historico-cotas-aportes = Capital share contributions taken out of expenses
nucleo-contas-historico-cotas-saldo-removido = Capital shares balance removed
nucleo-contas-cotas-erro-tipo-conta = capital shares belong in a checking (or savings) account at the credit union
nucleo-contas-cotas-erro-ative = turn on capital shares for this account first
nucleo-contas-cotas-erro-saldo-negativo = the shares balance can't be negative

## Balance sync

nucleo-contas-ref-sincronia = balance sync { $id }
nucleo-contas-historico-sincronia = Balance sync: { $nome } on { $data }
nucleo-contas-historico-sincronia-motivo = Reason for the balance difference
# Description and note of the adjustment (saved on the transaction).
nucleo-contas-sincronia-descricao = Balance sync
nucleo-contas-sincronia-nota = Bank balance on { $data }: { $banco }; Treasury had { $calculado }.
nucleo-contas-sincronia-erro-futuro = the balance date can't be in the future
nucleo-contas-sincronia-erro-motivo = write the reason for the difference
# Likely causes of the difference.
nucleo-contas-sincronia-dica-mesmo-valor = { $n ->
    [one] { $n } transaction with the exact amount of the difference ({ $valor }): it may have been entered extra (or missed) in Treasury.
   *[other] { $n } transactions with the exact amount of the difference ({ $valor }): one may have been entered extra (or missed) in Treasury.
}
nucleo-contas-sincronia-dica-previsto = There's a planned transaction for the amount of the difference that isn't cleared yet: maybe it already happened at the bank.
nucleo-contas-sincronia-dica-sinal = A { $valor } transaction with the wrong sign would explain the difference.
nucleo-contas-sincronia-dica-duplicados = { $n } transactions look duplicated (same date, amount and description).
nucleo-contas-sincronia-dica-conferir = { $n ->
    [one] No obvious cause: check the period's { $n } transaction against the statement (fees, interest and refunds are often missing).
   *[other] No obvious cause: check the period's { $n } transactions against the statement (fees, interest and refunds are often missing).
}

## Currency exchange

nucleo-contas-ref-taxa-cambio = exchange rate
nucleo-contas-historico-cambio = Exchange { $origem } → { $destino } on { $data }
nucleo-contas-historico-cambio-manual = Exchange rate entered manually
nucleo-contas-historico-cambio-removido = Exchange rate removed
# Descriptions and note of the operation's transactions (saved).
nucleo-contas-cambio-descricao = Exchange { $origem } → { $destino }
nucleo-contas-cambio-descricao-iof = Exchange IOF
nucleo-contas-cambio-descricao-tarifa = Exchange fee
nucleo-contas-cambio-nota-iof = Exchange operation { $origem } → { $destino } ({ $pct }%).
nucleo-contas-cambio-erro-numero = invalid { $campo }: { $valor }
nucleo-contas-cambio-campo-referencia = reference rate
nucleo-contas-cambio-campo-spread = spread
nucleo-contas-cambio-campo-taxa = rate
nucleo-contas-cambio-campo-iof = IOF
nucleo-contas-cambio-campo-ptax = PTAX
nucleo-contas-cambio-erro-grande = amount too large for the exchange: check the amount and the rate
nucleo-contas-cambio-erro-mesma-moeda = an exchange is between different currencies; for the same currency use a transfer
nucleo-contas-cambio-erro-sem-taxa = enter the applied rate or the reference rate (PTAX in Brazil)
nucleo-contas-cambio-erro-taxa-positiva = the exchange rate must be positive
nucleo-contas-cambio-erro-valor = enter the amount going out or the amount coming in (positive)
nucleo-contas-cambio-erro-ptax = invalid PTAX response: { $erro }
nucleo-contas-cambio-erro-referencia-brl = the reference is for a foreign currency in the family's currency
nucleo-contas-cambio-sem-referencia = no reference rate for { $moeda }: enter the rate by hand
nucleo-contas-cambio-erro-taxa = invalid rate: { $valor }
nucleo-contas-cambio-erro-intervalo = converted amount out of range
nucleo-contas-cambio-erro-moedas-iguais = the currencies must be different
# Central Bank of Brazil PTAX bulletins.
nucleo-contas-ptax-abertura = Opening
nucleo-contas-ptax-intermediario = Intermediate
nucleo-contas-ptax-fechamento = Closing

nucleo-contas-sincronia-copias-removidas = the difference came from transactions copied from another account, already deleted

## Card plastics (holder, additional, virtual)
nucleo-contas-ref-plastico = card { $id }
nucleo-contas-plastico-final = ending { $final }
nucleo-contas-plastico-tipo-titular = holder's card
nucleo-contas-plastico-tipo-adicional = additional card
nucleo-contas-plastico-tipo-virtual = virtual card
nucleo-contas-plastico-tipo-temporario = temporary card
nucleo-contas-plastico-erro-tipo = unknown card type (holder, additional, virtual or temporary)
nucleo-contas-plastico-erro-numero-inteiro = store only the card's last 4 digits, never the full number
nucleo-contas-plastico-erro-sublimite = the spending limit can't be negative
nucleo-contas-plastico-erro-final-repetido = this card already has a plastic ending in { $final }
nucleo-contas-plastico-erro-outro-cartao = the plastic belongs to another card
nucleo-contas-historico-plastico-criado = Additional card on { $cartao }
nucleo-contas-historico-plastico-alterado = Change to { $plastico }
nucleo-contas-historico-plastico-apagado = Removal of { $plastico }

## Limit, bill dates and annual fee
nucleo-contas-ref-limite = limit record
nucleo-contas-ref-anuidade = card annual fee { $id }
nucleo-contas-fatura-erro-datas = the due date can't be before the closing date
nucleo-contas-fatura-erro-fechamento = the closing date can't be before the bill's first purchase day
nucleo-contas-fatura-erro-valor-negativo = the bill's informed amounts can't be negative
nucleo-contas-anuidade-erro-valor = the annual fee can't be negative
nucleo-contas-anuidade-erro-parcelas = the annual fee is charged in 1 to 12 installments
nucleo-contas-anuidade-erro-mes = the renewal month goes from 1 to 12
nucleo-contas-anuidade-erro-isencao = enter the amount that waives the fee (spending on the bill or amount invested)
nucleo-contas-anuidade-erro-desconto = the discount is a number between 0 and 1 (0.5 = 50%)
nucleo-contas-historico-fatura-alterada = Bill changed
nucleo-contas-historico-limite = Card limit
nucleo-contas-historico-anuidade = Card annual fee
nucleo-contas-historico-anuidade-fim = Card without annual fee
nucleo-contas-recompensa-erro-taxa = the rate is a decimal number (0.01 = 1% cashback; points per currency unit)
nucleo-contas-recompensa-erro-teto = the cap per bill can't be negative
nucleo-contas-recompensa-erro-prazo = the credit deadline goes from 0 to 180 days
nucleo-contas-historico-recompensa = Card cashback and points
nucleo-contas-historico-recompensa-fim = Card without reward program

## Points and miles
nucleo-pontos-ref-programa = points program { $id }
nucleo-pontos-ref-fatura = card bill
nucleo-pontos-ref-movimento = points entry
nucleo-pontos-erro-nome = give the program a name (e.g. Livelo, Smiles)
nucleo-pontos-erro-valor = amounts can't be negative
nucleo-pontos-erro-pontos = enter a number of points other than zero
nucleo-pontos-erro-transferencia-par = a transfer between programs is recorded on both sides at once
nucleo-pontos-erro-mesmo-programa = a transfer needs two different programs
nucleo-pontos-historico-programa = Points program: { $descricao }
nucleo-pontos-historico-programa-alterado = Points program changed: { $descricao }
nucleo-pontos-historico-programa-apagado = Points program deleted: { $descricao }
nucleo-pontos-historico-movimento = Points in { $descricao }
nucleo-pontos-historico-transferencia = Points transfer from { $descricao }
nucleo-pontos-historico-movimento-apagado = Points entry deleted

## Account freshness (docs/account-freshness.md)

nucleo-contas-erro-limites-atualizacao = The days for "possibly incomplete" must be more than the days for "updated", and both above zero.
nucleo-contas-historico-limites-atualizacao = Account freshness limits changed
