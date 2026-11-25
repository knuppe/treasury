# Screen 03 · Accounts: account list and summary, each account's card, account form, details
# (balance history, reconciliation, bank sync, exchange rates, cooperative capital shares,
# overdraft) and the card bills and installments (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Repeated labels

contas-data = Date
contas-saldo = Balance
contas-tipo = Type
contas-moeda = Currency
contas-valor = Amount
contas-total = Total
contas-restante = Remaining
contas-calculado = Calculated
contas-conciliado = Reconciled
contas-previsto = Planned
contas-pago = Paid
contas-vence = Due
contas-mes = Month
contas-em-moeda = In { $moeda }
contas-sem-limite = no limit
contas-cartao = Card
contas-faturas = Card bills
contas-parcelamentos = Installments
contas-historico-saldo = Balance history
contas-conciliacao = Reconciliation
contas-cambio = Exchange rate
contas-reativar = Reactivate
contas-arquivar = Archive
contas-desfazer = Undo
contas-limite-moeda = Limit ({ $moeda })

## Header, summary and groups

contas-descricao = Measured balances (opening balance + transactions), planned and reconciled; cards with bills and limit; accounts in other currencies converted to { $moeda }.
contas-membros-familia = Family members
contas-novo-cartao = New card
contas-nova-conta = New account
contas-carregando = Loading accounts
contas-vazio-kicker = No accounts
contas-vazio-titulo = Add your first account
contas-vazio-descricao = Checking, savings, cash, credit card, brokerage or foreign account. The balance is the opening balance plus the transactions.
contas-resumo = Accounts summary
contas-saldo-liquido = Net balance in { $moeda }
contas-saldo-liquido-fonte = Accounts that count toward net worth (without investments at market value)
contas-saldo-liquido-estimado = An account in another currency has an exchange rate older than 7 days.
contas-sem-cambio-fora = { $n ->
        [one] { $n } account without an exchange rate left out of the total
       *[other] { $n } accounts without an exchange rate left out of the total
    }
contas-em-contas = In accounts
contas-devido-cartoes = Owed on cards
contas-credito-disponivel = Available credit
contas-mostrar-arquivadas = Show archived
contas-convertidas-moeda = converted to { $moeda }
contas-cambio-origem = Exchange rate (PTAX or entered by hand)
contas-acoes-de = Actions for { $nome }
contas-conta-reativada = Account reactivated
contas-conta-arquivada = Account archived
contas-erro-alterar = Could not change the account

## Closing the account (closed at the bank or the broker)

contas-encerrar = Close account
contas-menu-encerrar = Close account…
contas-reabrir = Reopen account
contas-conta-encerrada = Account closed
contas-conta-reaberta = Account reopened
contas-encerrada-em = closed on { $data }
contas-encerrada-em-nome = { $nome }, closed on { $data }
contas-encerrada-descricao = Closed on { $data }. It stays out of lists and alerts and takes no transaction after that date; its history stays in the reports.
contas-encerrar-titulo = Close { $nome }
contas-encerrar-descricao = For an account you closed at the bank or the broker. It leaves the lists and stops raising alerts; its history stays in the reports. You can reopen it later.
contas-encerrar-data = Closing date
contas-encerrar-data-dica = The day the bank closed the account, or the day of the last transaction.
contas-encerrar-saldo = Balance on { $data }
contas-encerrar-bloqueios = Before closing
contas-encerrar-bloqueio-futuro = The date cannot be after today.
contas-encerrar-bloqueio-abertura = The date is before the account was opened ({ $data }).
contas-encerrar-bloqueio-encerrada = The account was already closed on { $data }.
contas-encerrar-bloqueio-saldo = There is still { $valor } in the account on that date. Transfer the balance to another account or record an adjustment.
contas-encerrar-bloqueio-depois = { $n ->
        [one] There is { $n } transaction after the date (the last one on { $data }).
       *[other] There are { $n } transactions after the date (the last one on { $data }).
    }
contas-encerrar-bloqueio-posicoes = The account still holds { $ativos }. Record the sale or the transfer to another broker.
contas-encerrar-bloqueio-parcelas = { $n ->
        [one] The card still has { $n } installment to come.
       *[other] The card still has { $n } installments to come.
    }
contas-encerrar-transferir = Transfer the balance
contas-encerrar-ajuste = Record an adjustment
contas-encerrar-efeitos = What closing does
contas-encerrar-pronto = The account is empty on that date and can be closed.
contas-encerrar-sem-avisos = It leaves the lists and alerts; transactions after the date are blocked.
contas-encerrar-recorrencias = { $n ->
        [one] { $n } recurring item will be paused: { $nomes }.
       *[other] { $n } recurring items will be paused: { $nomes }.
    }
contas-encerrar-previstos = { $n ->
        [one] { $n } planned transaction will be deleted.
       *[other] { $n } planned transactions will be deleted.
    }
contas-encerrar-emprestimos = Loans debited from this account: { $nomes }. Change the debit account in Debts.
contas-encerrar-cartoes = Cards paid from this account: { $nomes }. Change the paying account in the card's details.
contas-encerrar-confirmar = Close account

## Commands (search words are comma-separated)

contas-cmd-nova-busca = bank,card,brokerage
contas-cmd-novo-cartao = New credit card
contas-cmd-novo-cartao-busca = bill,statement,limit
contas-cmd-abrir = Open account: { $nome }
contas-cmd-abrir-busca = balance

## Each account's card

contas-sem-cambio = No { $moeda }/{ $base } exchange rate: enter a rate.
contas-cambio-manual = Exchange rate entered by hand
contas-cambio-fonte = Exchange rate { $fonte }
contas-cambio-nota = 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $base }.
contas-cambio-nota-velha = 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $base }. The rate is more than 7 days old.
contas-abrir = Open { $nome }
contas-detalhes = Details →
contas-instituicao = Institution
contas-membros-rotulo = { $n ->
        [one] Member
       *[other] Members
    }
contas-ultimo = Last activity
contas-fatura-saldo-devedor = Bill and balance owed
contas-saldo-hoje = Balance today
contas-arquivada = archived
contas-previsto-fim-mes = Planned (end of month)
contas-disponivel = Available
contas-ciclo = Cycle
contas-ciclo-valor = closes on day { $fechamento } · due on day { $vencimento }
contas-limite-comprometido = { $pct } of the limit in use
contas-abrir-site = Open the { $nome } website in the browser
contas-site = Website ↗

## Account context menu

contas-menu-abrir-detalhes = Open details
contas-menu-nova-compra = New purchase on this card
contas-menu-novo-lancamento = New transaction in this account
contas-menu-compra-parcelada = Installment purchase
contas-menu-transferir = Transfer from this account
contas-menu-ver-lancamentos = View transactions
contas-menu-importar = Import statement or card bill
contas-menu-conciliar = Reconcile with the statement
contas-menu-cambio = { $moeda }/{ $base } exchange rate
contas-menu-editar = Edit…

## Account details

contas-novo-lancamento = New transaction
contas-lancamentos-link = Transactions →
contas-fechar-detalhe = Close details
contas-saldo-inicial = Opening balance
contas-desde = Since
contas-patrimonio = Net worth
contas-patrimonio-entra = included
contas-patrimonio-fora = excluded
contas-saldo-devedor = Balance owed
contas-previsto-ate-fim-mes = Planned by the end of the month
contas-limite-de = of { $limite }
contas-cotas-capital = Capital shares
contas-secoes-de = Sections of { $nome }

## Reconciliation with the statement

contas-n-conciliados = { $n ->
        [one] { $n } transaction reconciled
       *[other] { $n } transactions reconciled
    }
contas-ate-data = through { $data }
contas-conciliar-explicacao = Take the balance from your bank statement on a date and compare it with the calculated one (opening balance + cleared transactions up to that date). If they match, the transactions up to then become { $conciliados } and are protected against accidental changes to amount or date.
contas-conciliados-destaque = reconciled
contas-data-extrato = Statement date
contas-saldo-extrato = Statement balance ({ $moeda })
contas-comparar = Compare
contas-extrato-banco = Bank statement
contas-diferenca = Difference
contas-n-marcados-conciliados = { $n ->
        [one] ✓ { $n } transaction marked as reconciled.
       *[other] ✓ { $n } transactions marked as reconciled.
    }
contas-conciliacao-desfeita = Reconciliation undone
contas-confere = It matches ✓
contas-marcar-conciliado = Mark as reconciled through { $data }
contas-nao-confere = It does not match. Look for missing or duplicate transactions through { $data }: { $link }.
contas-nao-confere-link = see the cleared ones not yet reconciled

## Balance sync with the bank

contas-sincronia-sem-motivo = no reason
contas-sincronia-explicada = explained
contas-sincronia-aceita = accepted
contas-sincronizar-explicacao = Don't know why? Sync with the bank: the difference becomes an { $ajuste } transaction (it counts as neither income nor expense), the balance matches the statement and the checkpoint is kept so you can look into it later.
contas-ajuste-destaque = adjustment
contas-observacao-opcional = Note (optional)
contas-observacao-exemplo = E.g. balance in the bank app at 6 p.m.
contas-saldo-sincronizado = Balance synced with the bank
contas-ajuste-lancado = Adjustment recorded on { $data }.
contas-sincronizar = Sync with the bank
contas-ajuste-resumo = Adjustment of { $diferenca } on { $data }: the bank showed { $banco } and Treasury { $treasury }.
contas-causas-provaveis = Likely causes
contas-ver-lancamentos-periodo = See the transactions for the period
contas-pedir-agente = Ask the agent to investigate
contas-pedido-agente = Investigate the balance difference in the account { $conta } on { $data } (sync { $id }): the bank showed { $banco } and Treasury { $treasury }. Use get_balance_sync_hints and the transactions for the period to find the reason and, if you find it, record it with resolve_balance_sync.
contas-motivo = Reason, if you already know it
contas-motivo-exemplo = E.g. September fee was not in the OFX
contas-anotar-motivo = Note the reason
contas-aceitar-sem-investigar = Accept without investigating
contas-sincronias = Balance syncs
contas-conferia = Matched the bank
contas-ajuste-de = Adjustment of { $valor }
contas-investigar = Investigate

## The account's manual exchange rate

contas-cambio-explicacao = { $base ->
        [BRL] Net worth in BRL uses PTAX when data collection is on
       *[other] Net worth in { $base } uses the collected exchange rate when data collection is on
    }; without it, or to correct it, enter the rate by hand. Rates older than 7 days mark the value as estimated (≈).
contas-taxa-rotulo = 1 { $moeda } =
contas-taxa-dica = { $base ->
        [BRL] In reais, e.g. 5.4321
       *[other] In { $base }
    }
contas-cambio-salvo = Exchange rate saved
contas-cambio-salvo-descricao = { $moeda }/{ $base } on { $data }
contas-salvar-taxa = Save rate
contas-taxas-cambio = Exchange rates
contas-taxa-valor = 1 { $base } = { NUMBER($taxa, maximumFractionDigits: 6) } { $cotacao }
contas-informada-mao = entered by hand
contas-remover = Remove
contas-nenhuma-taxa = No { $moeda }/{ $base } rate yet.

## Balance history

contas-periodo-grafico = Chart period
contas-n-meses = { $n } months
contas-desde-inicio = Since the start
contas-incluir-previstos = Include planned (next 2 months)
contas-variacao-periodo = Change in the period: { $valor }
contas-saldo-de = { $nome } balance
contas-saldo-fonte = Source: opening balance + { $previstos ->
        [sim] cleared and planned
       *[nao] cleared
    } transactions · { $intervalo ->
        [dia] daily
       *[mes] end of each month
    }
contas-saldo-grafico = { $nome } balance from { $de } to { $ate }
contas-saldo-por-data = { $nome } balance by date
contas-hoje = today

## Account form

contas-erro-nome = Enter the account name.
contas-erro-data = Enter the date of the opening balance.
contas-conta-atualizada = Account updated
contas-conta-criada = Account created
contas-nova-instituicao-opcao = + New institution…
contas-editar-nome = Edit { $nome }
contas-criar-conta = Create account
contas-nome = Name
contas-nome-exemplo = Bank account, Nubank, Wallet…
contas-tipo-fixo = The type cannot change after the account is created.
contas-moeda-fixa = The currency cannot change after the account is created.
contas-moeda-dica = Foreign accounts use the exchange rate for net worth in { $moeda }.
contas-fatura-inicial = Bill open at the start ({ $moeda })
contas-saldo-inicial-moeda = Opening balance ({ $moeda })
contas-na-data = On date
contas-agencia = Branch
contas-agencia-dica = Without the check digit, if your bank does not use one.
contas-numero-conta = Account
contas-numero-conta-dica = With the check digit; it recognizes the account in OFX statements.
contas-membro-familia = Family member
contas-sem-membros = Add the members on the Family screen.
contas-membro-explicacao = With a single member, this account's transactions belong to them (you can change them one by one). With two or more, or none, they belong to the family.
contas-dia-fechamento = Closing day
contas-dia-vencimento = Due day
contas-vence-mes-seguinte = Due in the month after closing.
contas-vence-mesmo-mes = Due in the same month as closing.
contas-conta-pagadora = Account that pays the bill
contas-bandeira = Network
contas-bandeira-exemplo = Visa, Mastercard, Elo…
contas-final-cartao = Last 4 digits
contas-entra-patrimonio = Counts toward net worth
contas-entra-patrimonio-dica = Turn off for other people's accounts that you only keep track of.
contas-notas = Notes
contas-apagar-nome = Delete { $nome }
contas-apagar-descricao = You can only delete accounts with no transactions, recurrences or installment plans. To keep the history, archive it.
contas-apagar-conta = Delete account
contas-conta-apagada = Account deleted

## Bank from the official list

contas-banco-exemplo = Bank name or code (e.g. 260, Itaú)
contas-banco-exemplo-exterior = Bank or institution name
contas-plano-aposentadoria = Plan
contas-plano-aposentadoria-dica = The year's contribution limit depends on the plan.
contas-cobertura = Coverage
contas-cobertura-individual = Self-only
contas-cobertura-familia = Family
contas-aposentadoria-titulo = { $plano } in { $ano }
contas-aposentadoria-de = { $aportado } of { $limite }
contas-aposentadoria-sem-limite = { $aportado } this year (no annual limit)
contas-aposentadoria-falta = { $falta } still fits this year.
contas-aposentadoria-acima = This year's contributions are over the limit: check with the plan before the correction deadline.
contas-aposentadoria-beneficio = This is the contribution that already gives the full tax benefit.
contas-aposentadoria-idade = Limit for age { $idade }.
contas-aposentadoria-divide = Shares the limit with { $contas }.
contas-aposentadoria-ano-anterior = This year's limit is not out yet: { $ano }'s applies.
contas-aposentadoria-fonte = Limit source
contas-nova-instituicao = New institution
contas-bancos-bc = Banks from the Central Bank
contas-banco-oficial = Bank from the official list; the code recognizes OFX statements.
contas-lista-oficial = Official list from Brazil's Central Bank, updated every week.

## Overdraft

contas-cheque-especial = Overdraft
contas-tem-cheque-especial = The account has an overdraft limit
contas-cheque-especial-dica = The balance can go negative down to the limit; Treasury shows how much is in use and an estimate of interest and IOF.
contas-juros-mes = Monthly interest (%)
contas-desconto = Discount (%)
contas-desconto-dica = On the interest rate.
contas-juros-ano = Annual interest (%)
contas-juros-ano-spread = Annual interest over the index (%)
contas-juros-ano-dica = You can type it with the index, as the bank shows it: 4.75 + TR.
contas-taxa-indexada-exemplo = e.g. 4.75 + TR
contas-indexador = Index
contas-indexador-dica = Floating rate: the rate above is added to the index (TR, CDI, IPCA…).
contas-cheque-taxa-indexada = · effective rate ≈ { $taxa }% a month (with { $indice } at { $valor }% on { $data })
contas-cheque-taxa-indexada-estimada = · effective rate ≈ { $taxa }% a month (with projected { $indice } at { $valor }% on { $data })
contas-iof-basico = Basic IOF (% per day)
contas-iof-adicional = Additional IOF (%)
contas-cet-mes = Monthly CET (%)
contas-cet-ano = Annual CET (%)
contas-cheque-uso = { $usado } of { $limite }
contas-cheque-disponivel = Available with the limit: { $valor }
contas-cheque-custo = · estimated cost over 30 days: { $valor }
contas-cheque-incerto = No recent statement on this account: the negative balance may be missing transactions, not necessarily overdraft.
contas-atualizacao-ha = updated { $dias } days ago
contas-atualizacao-incompleto = possibly incomplete
contas-atualizacao-ultimo = Last transaction on { $data }.
contas-atualizacao-sem-lancamentos = No transactions yet.
contas-atualizacao-mes-vazio = { $mes } had no transactions.
contas-atualizacao-negativo = The negative balance may be missing transactions, not necessarily overdraft.
contas-cheque-custo-juros = · estimated cost over 30 days: { $valor } ({ $taxa }% interest per month + IOF)

## Capital shares

contas-cotas-explicacao = In credit unions, members hold capital shares: a stake that earns whatever the credit union decides (interest on capital, surplus). Turn it on to track the share balance, the contributions that leave this account and the return in each period (change in the balance minus the contributions).
contas-cotas-ativar = Turn on capital shares for this account
contas-cotas-saldo = Share balance
contas-cotas-aportes-periodo = Contributions in the period
contas-cotas-rentabilidade = Return
contas-cotas-nota = Note
contas-cotas-pct = { NUMBER($valor, maximumFractionDigits: 2) }%
contas-cotas-em = on { $data }
contas-cotas-informe-saldo = enter the balance
contas-cotas-aportes = Contributions
contas-cotas-depois-ultimo = { $valor } after the last balance
contas-cotas-rentabilidade-apurada = Return so far
contas-cotas-12-meses = Last 12 months
contas-cotas-data-saldo = Balance date
contas-cotas-saldo-moeda = Share balance ({ $moeda })
contas-cotas-nota-opcional = Note (optional)
contas-cotas-nota-exemplo = E.g. annual statement from the credit union
contas-cotas-saldo-informado = Share balance entered
contas-cotas-informar-saldo = Enter balance
contas-cotas-saldos = Capital share balances
contas-cotas-aportes-identificados = Contributions found
contas-cotas-aportes-identificados-n = Contributions found ({ $n })
contas-cotas-ja-fora = The contributions were already out of expenses
contas-cotas-n-tirados = { $n ->
        [one] { $n } contribution taken out of expenses
       *[other] { $n } contributions taken out of expenses
    }
contas-cotas-tirar = Take the contributions out of expenses
contas-cotas-como-identificar = How to find the contributions
contas-cotas-texto-extrato = Text in the statement
contas-cotas-texto-dica = Separate with | (e.g. COTA CAPITAL|INTEGRALIZACAO).
contas-cotas-ou-categoria = Or the category
contas-cotas-categoria = Contributions category
contas-cotas-nenhuma = None
contas-cotas-somar = Add the share balance to net worth
contas-cotas-somar-dica = Counts as an investment, at the last balance entered.

## Card bills

contas-pagar-fatura-de = Pay the { $mes } bill
contas-pagar-fatura-descricao = Records a transfer of the “card bill payment” type: it does not count as an expense (the purchases already did). Future dates stay as planned.
contas-pagamento-lancado = Payment recorded
contas-pagar = Pay
contas-ja-pago = Already paid
contas-vencimento = Due date
contas-pagar-com = Pay with
contas-escolha-conta = Choose the account
contas-fatura-de = { $mes } bill
contas-ver-em-lancamentos = View in transactions →
contas-compras = Purchases
contas-compras-periodo = { $inicio } to { $fim }
contas-lancamentos = Transactions
contas-total-informado = Total reported by the bank
contas-total-informado-salvo = Reported total saved
contas-salvar-total-informado = Save reported total
contas-confere-calculado = Matches the calculated total ✓
contas-diferenca-banco = Difference bank − calculated: { $valor }
contas-pagar-fatura = Pay bill
contas-lancamentos-fatura = Bill transactions
contas-pagamento = Payment
contas-sem-categoria = no category
contas-fatura = Bill
contas-situacao = Status
contas-banco-calc = Bank × calc.
contas-confere-curto = matches
contas-faturas-vazio-kicker = No bills
contas-faturas-vazio-titulo = No purchases on this card yet
contas-faturas-vazio-descricao = Bills appear on their own from the purchases, based on the closing and due days.

## Installments and the card's future commitments

contas-a-pagar = To pay
contas-compromissos-titulo = Future card commitments, 12 months
contas-compromissos-fonte = Source: open bills, with future installments and planned recurrences
contas-compromissos-grafico = Amount to pay per month on upcoming bills
contas-compromissos-tabela = Future card commitments by month
contas-compras-parceladas = Installment purchases
contas-sem-parceladas = No installment purchases. Add one in Transactions › New transaction › Card installments.
contas-compra-em = bought on { $data }
contas-parcelas-pagas = { $pagas }/{ $total } paid
contas-falta = left
contas-apagar-parcelamento = Delete installment plan
contas-apagar-parcelamento-descricao = All installments (past and future) are removed from the card. You can undo this from the history.
contas-apagar-parcelas = Delete installments
contas-n-parcelas-apagadas = { $n ->
        [one] { $n } installment deleted
       *[other] { $n } installments deleted
    }

## Account list and page

contas-lista = Accounts
contas-col-conta = Account
contas-col-membros = Members
contas-col-saldo = Balance
contas-sem-membro = Family
contas-voltar-lista = All accounts
contas-detalhe-resumo = Account summary
contas-ultimos-lancamentos = Transactions
contas-col-descricao = Description
contas-col-categoria = Category
contas-col-valor = Amount
contas-sem-lancamentos = No transactions in this account yet.
contas-ver-todos-lancamentos = See all in Transactions

## Balance adjustment

contas-ajustar-saldo = Adjust balance
contas-ajuste-explicacao = Enter the balance the account had on a date (from the bank or the statement). Treasury records the difference as an adjustment: it counts in the account's balance and history but stays out of income, expenses and the budget. Transactions up to that date are marked reconciled.
contas-ajuste-conta = Account
contas-ajuste-data = Balance date
contas-ajuste-saldo = Balance on that date ({ $moeda })
contas-ajuste-saldo-dica = What the bank shows at the end of the day.
contas-ajuste-previa = Adjustment preview
contas-ajuste-calculado = Calculated on { $data }
contas-ajuste-informado = Entered
contas-ajuste-valor = Adjustment
contas-ajuste-confere = It already matches: only marks the date as reconciled.
contas-ajuste-fora-relatorios = Stays out of income, expenses and the budget.
contas-ajuste-sem-diferenca = Balance confirmed

## Cards of a credit card account: holder, additional and virtual
contas-cartoes = Cards
contas-plasticos-explicacao = Each card (yours, the family's additional cards and the virtual ones) has its last digits and the person who uses it. The bill and the limit are still one; each purchase goes to whoever used the card.
contas-plasticos-vazio = No cards yet. Add yours and the additional ones to know who spent what.
contas-plastico-tipo = Type
contas-plastico-tipo-holder = Holder
contas-plastico-tipo-additional = Additional
contas-plastico-tipo-virtual = Virtual
contas-plastico-tipo-temporary = Temporary
contas-plastico-final = Ending { $final }
contas-plastico-final-rotulo = Card ending
contas-plastico-final-dica = Only the last 4 digits.
contas-plastico-membro = Used by
contas-plastico-titular-conta = Account holder
contas-plastico-do-titular = Account holder ({ $nome })
contas-plastico-apelido = Nickname
contas-plastico-apelido-exemplo = E.g. subscriptions, travel
contas-plastico-sublimite = Spending limit per bill
contas-plastico-sublimite-dica = Just a warning: the app doesn't block purchases.
contas-plastico-gasto-ciclo = This bill:
contas-plastico-uso-sublimite = { $pct }% of the limit of
contas-plastico-acima-sublimite = over the limit
contas-plastico-perto-sublimite = close to the limit
contas-plastico-novo = New card
contas-plastico-dialogo-descricao = Purchases made with this card go to the chosen person.
contas-plastico-salvar = Save
contas-plastico-salvo = Card saved
contas-plastico-editar = Edit
contas-plastico-ativo = Active
contas-plastico-inativo = Inactive
contas-plastico-remover = Remove
contas-plastico-remover-titulo = Remove { $cartao }?
contas-plastico-remover-confirmar = Purchases made with it go back to the account holder.
contas-plastico-removido = Card removed
contas-fatura-por-cartao = By card
contas-fatura-sem-cartao = No card given

## Limit, bill dates and annual fee
contas-limite-anuidade = Limit and annual fee
contas-limite-uso = Limit usage
contas-limite-usado = Used:
contas-limite-de-total = of
contas-limite-disponivel = Available:
contas-limite-parcelas-futuras = Future installments already take
contas-limite-sem = This card has no limit set.
contas-melhor-dia = Best day to buy: { $data }. Purchases from then on are due on { $vencimento } ({ $dias } days to pay).
contas-limite-historico = Limit history
contas-limite-historico-vazio = No limit records.
contas-limite-novo = Record a limit change
contas-limite-novo-descricao = Did the bank raise or lower the limit? Record the date and the new amount.
contas-limite-salvo = Limit recorded
contas-limite-da-fatura = From the bill
contas-limite-informado = Entered
contas-anuidade = Annual fee
contas-anuidade-vazio = Enter the card's annual fee so the app can check the charges, show how much is left for the waiver and remind you before it renews.
contas-anuidade-informar = Enter annual fee
contas-anuidade-mudar = Change rule
contas-anuidade-sem = No annual fee
contas-anuidade-sem-titulo = Does this card no longer have an annual fee?
contas-anuidade-sem-descricao = The rule ends today. Past charges stay in the history.
contas-anuidade-regra = Card annual fee
contas-anuidade-regra-descricao = What you agreed with the bank. Changing the rule creates a new one from the start date.
contas-anuidade-salva = Annual fee saved
contas-anuidade-valor-ano = Yearly amount
contas-anuidade-parcelas = Installments
contas-anuidade-em-parcelas = in { $n }× of
contas-anuidade-mes = Renewal month
contas-anuidade-mes-nao-sei = Don't know
contas-anuidade-inicio = Valid from
contas-anuidade-isencao = Waiver
contas-anuidade-isencao-none = No waiver
contas-anuidade-isencao-full = Waived
contas-anuidade-isencao-spend = By spending on the bill
contas-anuidade-isencao-investment = By amount invested at the bank
contas-anuidade-isencao-discount = Fixed discount
contas-anuidade-gasto-minimo = Bill spending that waives it
contas-anuidade-investimento-minimo = Amount invested that waives it
contas-anuidade-desconto = Discount (%)
contas-anuidade-notas = Notes
contas-anuidade-pago-12m = Paid in 12 months
contas-anuidade-renova = Renews on
contas-anuidade-isenta-agora = The open bill has already reached the waiver.
contas-anuidade-falta-gastar = Left to spend on this bill to waive the installment:
contas-anuidade-falta-investir = Left to invest at the bank for the waiver:
contas-anuidade-cobrancas = Annual fee charges
contas-anuidade-sem-cobrancas = No annual fee charges in the last 12 months.
contas-anuidade-esperado = Expected:
contas-anuidade-problema-isenta = Charged on a bill that had reached the waiver.
contas-anuidade-problema-valor = Different from the agreed installment.
contas-anuidade-problema-parcela = Goes over the year's number of installments.
contas-fatura-ajustar-botao = Adjust dates and amounts
contas-fatura-ajustar = { $mes } bill
contas-fatura-ajustar-descricao = Use it when the bank postpones the closing or changes the due date for a month. Adjusted dates no longer follow the card's cycle.
contas-fatura-datas-calculadas = Back to the cycle dates
contas-fatura-datas-manuais = adjusted
contas-fatura-datas-impressas = from the bank's bill
contas-fatura-fechamento = Closing
contas-fatura-minimo = Minimum payment
contas-fatura-limite = Limit on the bill
contas-fatura-saldo-anterior = Previous bill
contas-fatura-alterada = Bill changed
contas-rotativo-titulo = The { $mes } bill went into revolving credit
contas-rotativo-sem-pagar = Left unpaid:
contas-rotativo-encargos = Interest and charges since then:
contas-rotativo-regras-br = Revolving credit can only last until the next bill, and the interest and charges of revolving credit and bill installment plans can't exceed 100% of the original debt (Law 14,690/2023).
contas-rotativo-regras = A balance left on the card usually carries high interest; paying it off or swapping it for a cheaper loan usually works out better.
contas-rotativo-acima-teto = The charges are already above the original debt: check the bill with the bank.
contas-rotativo-alem-da-fatura = More than one bill has gone into revolving credit: the bank should have offered an installment plan.

## Cashback and points
contas-recompensa-aba = Cashback and points
contas-recompensa-titulo = Cashback and points
contas-recompensa-vazio = Enter the card's cashback or points so the app can work out what each bill should earn, check what arrived and warn you when the credit doesn't come.
contas-recompensa-informar = Enter program
contas-recompensa-sem = No program
contas-recompensa-sem-titulo = Does this card no longer have cashback or points?
contas-recompensa-regra = Card cashback and points
contas-recompensa-regra-descricao = What the card gives back on each purchase. Changing the rule creates a new one from the start date.
contas-recompensa-salva = Program saved
contas-recompensa-tipo = Type
contas-recompensa-cashback = Cashback
contas-recompensa-pontos = Points or miles
contas-recompensa-taxa = Base rate
contas-recompensa-taxa-cashback = Cashback (%)
contas-recompensa-taxa-pontos = Points per currency unit
contas-recompensa-pontos-por = { $n } points per { $moeda }
contas-recompensa-moeda-pontos = Rule currency
contas-recompensa-moeda-pontos-dica = Many programs count points per dollar spent.
contas-recompensa-teto-cashback = Cap per bill
contas-recompensa-teto-pontos = Points cap per bill
contas-recompensa-destino = Where the cashback goes
contas-recompensa-destino-fatura = Credit on the bill
contas-recompensa-destino-conta = To an account
contas-recompensa-destino-investimento = To an investment
contas-recompensa-conta-destino = Receiving account
contas-recompensa-prazo = Credit deadline (days)
contas-recompensa-prazo-dica = Days after closing until the cashback usually arrives.
contas-recompensa-por-categoria = Rates by category
contas-recompensa-taxa-categoria = Category rate
contas-recompensa-nova-categoria = Add category
contas-recompensa-previsto-12m = Expected in 12 months
contas-recompensa-recebido-12m = Received in 12 months
contas-recompensa-como-conta = Cashback counts as income (Cashback and rewards category), without lowering what the purchases cost. Credits with "cashback" in the description also count.
contas-recompensa-por-fatura = By bill
contas-recompensa-previsto = Expected
contas-recompensa-recebido = Received
contas-recompensa-nao-veio = not received by { $data }
contas-recompensa-n-pontos = { $n } points

## Points and miles
contas-pontos-titulo = Points and miles
contas-pontos-vazio = Add the family's points and miles programs to follow each person's balance, the expiry dates and transfers with bonus.
contas-pontos-fora-patrimonio = Points are not part of net worth; the estimated value uses the value per 1,000 points you enter.
contas-pontos-programa = Program
contas-pontos-programa-novo = New program
contas-pontos-programa-descricao = One program per person: the points account belongs to whoever holds it.
contas-pontos-programa-salvo = Program saved
contas-pontos-nome = Name
contas-pontos-membro = Belongs to
contas-pontos-sem-membro = No member
contas-pontos-valor-milheiro = Value of 1,000 points
contas-pontos-valor-milheiro-dica = Your estimate, to see what the balance is worth. Not part of net worth.
contas-pontos-tipo-bank = Bank
contas-pontos-tipo-airline = Airline
contas-pontos-tipo-retail = Retail
contas-pontos-tipo-other = Other
contas-pontos-mov-earn = Earned
contas-pontos-mov-bonus = Bonus
contas-pontos-mov-purchase = Points bought
contas-pontos-mov-transfer_in = Transfer in
contas-pontos-mov-transfer_out = Transfer out
contas-pontos-mov-redeem = Redeemed
contas-pontos-mov-expire = Expired
contas-pontos-mov-adjust = Adjustment
contas-pontos-movimento-novo = Record points
contas-pontos-movimento-descricao = Earned, bonus, bought, redeemed or expired, as the program's statement shows.
contas-pontos-movimento-salvo = Points recorded
contas-pontos-quantidade = Points
contas-pontos-vence-em = Expire on
contas-pontos-vence = expire on { $data }
contas-pontos-custo = What it cost
contas-pontos-transferir = Transfer
contas-pontos-transferir-descricao = Points leave one program and enter another, with the campaign bonus.
contas-pontos-transferencia-salva = Transfer recorded
contas-pontos-de = From
contas-pontos-para = To
contas-pontos-saem = Points out
contas-pontos-bonus = Bonus (%)
contas-pontos-entram = { $n } points go into the destination.
contas-pontos-sem-movimentos = No entries yet.
contas-pontos-n = { $n } points
contas-pontos-proximo-vencimento = { $n } expire on { $data }
contas-pontos-cartoes = Earned with: { $cartoes }
contas-recompensa-programa-pontos = Points program
contas-recompensa-programa-pontos-nenhum = None (expected only)

## Is this card worth it?
contas-vale-titulo = Is this card worth it?
contas-vale-periodo = From { $de } to { $ate }: what the card cost and what it gave back.
contas-vale-anuidade = Annual fee
contas-vale-encargos = Interest and charges
contas-vale-iof = IOF
contas-vale-cashback = Cashback received
contas-vale-pontos = Points (estimated value)
contas-vale-resultado = Result
contas-vale-simulacao = If all the family's purchases went to one card
contas-vale-simulacao-descricao = Return by each card's rules on the purchases of all cards in the last 12 months:
contas-vale-retorno = Return
contas-vale-aviso = This is an analysis of the family's data, not a financial product recommendation. Perks like insurance and lounges aren't counted.
