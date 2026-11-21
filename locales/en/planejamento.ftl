# Planning: how much can I spend today, balance forecast, debt payoff plan, subscriptions and
# "what if" scenarios (docs/i18n-design.md). Explanations and notes of the simulations come
# from the core. Revision: AI draft, not yet reviewed by a native speaker.

## Repeated labels

planejamento-n-meses = { $n } months
planejamento-n-anos = { $n } years
planejamento-data = Date
planejamento-nome = Name
planejamento-valor = Amount
planejamento-por-mes = Per month
planejamento-mais-50-anos = over 50 years
planejamento-tirar = Remove { $nome }

## How much can I spend today

planejamento-hoje-titulo = How much can I spend today
planejamento-hoje-estourado = This month, everything that came in is already committed. Hold off on the spending that can wait until the next paycheck.
planejamento-hoje-usado = Today's amount has already been used. It comes back tomorrow, recalculated with what is left.
planejamento-hoje-apertado = Things are tight: it's worth sticking to the essentials until the end of the month.
planejamento-hoje-ok = Within this amount, the month closes without putting bills and goals at risk.
planejamento-hoje-por-dia =
    { $n ->
        [one] { $valor } per day until { $data } ({ $n } day)
       *[other] { $valor } per day until { $data } ({ $n } days)
    }
planejamento-hoje-gasto = Already spent today: { $valor }
planejamento-hoje-esconder = Hide the math
planejamento-hoje-como = How I calculated it
planejamento-hoje-receitas = Income this month (received and planned)
planejamento-hoje-gastos = Spent before today
planejamento-hoje-contas = Bills still due this month
planejamento-hoje-metas = What's left to save for goals
planejamento-hoje-sobra = Left at the start of today
planejamento-hoje-amanha-titulo = Per day from tomorrow
planejamento-hoje-amanha =
    { $n ->
        [one] { $valor } per day from tomorrow ({ $n } day)
       *[other] { $valor } per day from tomorrow ({ $n } days)
    }
planejamento-hoje-gasto-linha = Spent today
planejamento-hoje-sobra-amanha = Left from tomorrow on
planejamento-hoje-dividido-amanha =
    { $n ->
        [one] Divided by { $n } day, from tomorrow.
       *[other] Divided by { $n } days, from tomorrow.
    }
planejamento-hoje-dividido =
    { $n ->
        [one] Divided by { $n } day, counting today.
       *[other] Divided by { $n } days, counting today.
    }

## Balance forecast

planejamento-origem-previsto = Planned
planejamento-origem-recorrencia = Recurrence
planejamento-origem-fatura = Card bill
planejamento-origem-parcela = Installment
planejamento-origem-parcela-detectada = Detected installment (estimate)
planejamento-origem-sazonal = yearly expense (estimate)
planejamento-previsao-titulo = Balance forecast
planejamento-previsao-horizonte = Forecast horizon
planejamento-previsao-total-hoje = Total today
planejamento-previsao-em = On { $data }
planejamento-previsao-ponto-baixo = Lowest point
planejamento-previsao-valor-em = { $valor } on { $data }
planejamento-previsao-negativa = { $conta } goes negative on { $data } and reaches { $minimo } on { $quando }.
planejamento-previsao-ja-negativa = { $conta } is already negative today and reaches { $minimo } on { $quando }.
planejamento-previsao-nenhuma-negativa = No account goes negative through { $data }.
planejamento-previsao-por-conta = Forecast balance by account
planejamento-previsao-fonte = Source: today's balance + planned transactions, recurrences, open card bills (paid from the card's payment account) and loan installments
planejamento-previsao-grafico = Forecast balance from { $de } to { $ate }
planejamento-previsao-eventos = What moves the balance
planejamento-previsao-conta = Account
planejamento-previsao-descricao = Description
planejamento-previsao-origem = Source
planejamento-previsao-total = Total

## Debt payoff plan

planejamento-quitacao-minimo = Minimum only
planejamento-quitacao-bola-neve = Snowball
planejamento-quitacao-avalanche = Avalanche
planejamento-quitacao-minimo-dica = Only the installments, no extra.
planejamento-quitacao-bola-neve-dica = Extra on the smallest debt first: quick wins.
planejamento-quitacao-avalanche-dica = Extra on the highest rate first: less interest.
planejamento-quitacao-menos-juros = less interest
planejamento-quitacao-livre = Debt-free in
planejamento-quitacao-meses = Months
planejamento-quitacao-juros-total = Total interest
planejamento-quitacao-sem-fim = never ends
planejamento-quitacao-juros-a-menos = Interest saved
planejamento-quitacao-meses-a-menos = Months saved
planejamento-quitacao-titulo = Debt payoff plan
planejamento-quitacao-intro = How much is left each month to pay beyond the installments? Once a debt is paid off, its installment also goes to the next one. Include debts that aren't in the app, like an overdraft or a revolving card balance.
planejamento-quitacao-extra = Extra per month
planejamento-quitacao-fora-app = Debt outside the app
planejamento-quitacao-exemplo = Overdraft
planejamento-quitacao-saldo-devedor = Balance owed
planejamento-quitacao-juros-ano = Interest per year
planejamento-quitacao-minimo-mes = Minimum per month
planejamento-quitacao-tirar-esta = Remove this debt
planejamento-quitacao-grafico = Total balance owed, month by month
planejamento-quitacao-fonte =
    { $n ->
        [one] Source: { $n } debt; minimum installments + { $extra } extra per month
       *[other] Source: { $n } debts; minimum installments + { $extra } extra per month
    }
planejamento-quitacao-grafico-aria = Balance owed by strategy
planejamento-quitacao-dividas = Debts in the simulation
planejamento-quitacao-divida = Debt
planejamento-quitacao-saldo = Balance
planejamento-quitacao-juros-aa = Interest p.a.
planejamento-quitacao-taxa = { NUMBER($taxa, maximumFractionDigits: 2) }%
planejamento-quitacao-minimo-col = Minimum
planejamento-quitacao-ordem = Payoff order
planejamento-quitacao-estrategia = Order strategy
planejamento-quitacao-ordem-de = Payoff order: { $estrategia }
planejamento-quitacao-quitada-em = Paid off in
planejamento-quitacao-juros-pagos = Interest paid
planejamento-quitacao-aviso = Simulation with a fixed rate and constant installment; not financial advice.

## Subscriptions and recurring charges

planejamento-assinaturas-em-dia = Up to date
planejamento-assinaturas-nova = New
planejamento-assinaturas-nao-veio = Missing
planejamento-assinaturas-recorrencia-criada = Recurrence created
planejamento-assinaturas-recorrencia-criada-descricao = { $nome }: the next charges come in as planned.
planejamento-assinaturas-criar-recorrencia = Create recurrence
planejamento-assinaturas-recorrencia-ok = recurrence ok
planejamento-assinaturas-nao-e = Not a subscription
planejamento-assinaturas-assinatura = Subscription
planejamento-assinaturas-frequencia = Frequency
planejamento-assinaturas-mensal = Monthly
planejamento-assinaturas-anual = Yearly
planejamento-assinaturas-desde-anterior = { $pct } since the previous one
planejamento-assinaturas-proxima = Next
planejamento-assinaturas-situacao = Status
planejamento-assinaturas-vazio = No subscriptions
planejamento-assinaturas-vazio-titulo = Nothing repeats every month in your transactions yet
planejamento-assinaturas-vazio-descricao = Subscriptions show up here on their own after three similar monthly charges from the same merchant (or two yearly ones). Import your statements and card bills to find yours.
planejamento-assinaturas-titulo = Subscriptions and recurring charges
planejamento-parcelas-titulo = Installments in progress
planejamento-parcelas-vazio-titulo = No installments in progress
planejamento-parcelas-vazio = When a transaction's description says it is an installment ("Parcela 3/10", "PARC 02/12", "PREST.EMPREST 003/005"), the app shows here how much is left and when it ends.
planejamento-parcelas-descricao = Purchase or contract
planejamento-parcelas-parcela = Installment
planejamento-parcelas-valor = Installment amount
planejamento-parcelas-falta = Left to pay
planejamento-parcelas-termina = Ends in
planejamento-parcelas-total = Left in total
planejamento-parcelas-proximo-mes = Next month
planejamento-parcelas-ignorar = Not an installment
planejamento-parcelas-ignorada = Series removed from installments
planejamento-parcelas-nao-apareceu = the next one didn't show up
planejamento-parcelas-tipo-card = Card purchase
planejamento-parcelas-tipo-loan = Loan
planejamento-parcelas-tipo-bill = Card bill installment plan
planejamento-parcelas-tipo-account = Installment on the account
planejamento-parcelas-nota = Estimate based on transaction descriptions; it goes into the balance forecast and the card commitments. Installment purchases entered in Treasury and debt contracts are left out so nothing is counted twice.
planejamento-assinaturas-por-ano = Per year
planejamento-assinaturas-detectadas = Detected
planejamento-assinaturas-subiu = { $nome } went up { $pct }: from { $antes } to { $agora } on { $data }.
planejamento-assinaturas-cobranca-nova = New charge: { $nome }, { $valor } since { $data }. If you don't recognize it, check with your bank.
planejamento-assinaturas-atrasada = { $nome } was expected on { $data } and didn't come: was it canceled?
planejamento-assinaturas-tabela = Detected subscriptions
planejamento-assinaturas-dispensadas =
    { $n ->
        [one] { $n } dismissed detection
       *[other] { $n } dismissed detections
    }
planejamento-assinaturas-trazer = Bring back
planejamento-assinaturas-fonte = Source: expenses from the last 2 years, excluding installment purchases and transfers. The monthly total doesn't include the missing ones.

## "What if" scenarios

planejamento-cenarios-imovel = Buy a home
planejamento-cenarios-filho = Have a child
planejamento-cenarios-pausa = Take a career break
planejamento-cenarios-renda = Change income
planejamento-cenarios-unico = One-off expense or income
planejamento-cenarios-projeto = From a project
planejamento-cenarios-evento-unico = { $valor } on { $data }
planejamento-cenarios-evento-mensal =
    { $meses ->
        [0] { $valor } per month from { $inicio }
       *[other] { $valor } per month from { $inicio } for { $meses } months
    }
planejamento-cenarios-evento-renda =
    { $meses ->
        [0] income at { $pct }% from { $inicio }
       *[other] income at { $pct }% from { $inicio } for { $meses } months
    }
planejamento-cenarios-evento-bem =
    { $taxa ->
        [0] asset worth { $valor } on { $data }
       *[other] asset worth { $valor } on { $data }, { $taxa }% p.a. real
    }
planejamento-cenarios-evento-financiamento = { $valor } loan over { $meses } months at { $taxa }% p.a. real
planejamento-cenarios-compra-imovel = Home purchase
planejamento-cenarios-imovel-bem = Home
planejamento-cenarios-financiamento = Mortgage
planejamento-cenarios-filho-evento = Child
planejamento-cenarios-pausa-evento = Career break
planejamento-cenarios-renda-em = Income at { $pct }%
planejamento-cenarios-mudanca-renda = Income change
planejamento-cenarios-gasto-unico = One-off expense
planejamento-cenarios-sem-orcamento = { $projeto }: no budget left to spend
planejamento-cenarios-a-partir = From
planejamento-cenarios-preco-imovel = Home price
planejamento-cenarios-entrada = Down payment
planejamento-cenarios-prazo = Term
planejamento-cenarios-anos = years
planejamento-cenarios-juros-reais = Real interest
planejamento-cenarios-juros-reais-dica = Above inflation (e.g., TR + 9% ≈ 5% real).
planejamento-cenarios-ao-ano = % p.a.
planejamento-cenarios-data-compra = Purchase date
planejamento-cenarios-custo-mes = Cost per month
planejamento-cenarios-renda-pausa = Income during the break
planejamento-cenarios-renda-pausa-dica = 0% = no income; 50% = part-time.
planejamento-cenarios-quanto-tempo = For how long
planejamento-cenarios-meses = months
planejamento-cenarios-novo-emprego = New job
planejamento-cenarios-exemplo-unico = Trip, inheritance…
planejamento-cenarios-diferenca-mes = Difference per month
planejamento-cenarios-negativo-dica = Negative for money going out (e.g., -5,000.00).
planejamento-cenarios-projeto-campo = Project
planejamento-cenarios-projeto-dica = The budget still to be spent goes out on the end date.
planejamento-cenarios-escolha = Choose
planejamento-cenarios-nenhum-projeto = No projects
planejamento-cenarios-sem-data-fim = If there's no end date
planejamento-cenarios-adicionar = Add to scenario
planejamento-cenarios-patrimonio-base = Net worth (baseline)
planejamento-cenarios-patrimonio-cenario = Net worth (scenario)
planejamento-cenarios-liquido-cenario = Accounts and investments (scenario)
planejamento-cenarios-base = Baseline
planejamento-cenarios-cenario = Scenario
planejamento-cenarios-patrimonio-em =
    { $anos ->
        [one] Net worth in { $anos } year
       *[other] Net worth in { $anos } years
    }
planejamento-cenarios-base-valor = baseline { $valor } · { $diferenca }
planejamento-cenarios-liquido = Accounts and investments
planejamento-cenarios-acabam = run out in { $mes } · { $diferenca }
planejamento-cenarios-ja-negativos = already negative today · { $diferenca }
planejamento-cenarios-nao-acabam = don't run out in the period · { $diferenca }
planejamento-cenarios-aposentadoria = Retirement
# The part between ** is shown smaller, in gray.
planejamento-cenarios-chance = { $pct }% **chance**
planejamento-cenarios-aposentadoria-base = baseline { $pct }% · at { $idade }: { $valor }
planejamento-cenarios-sem-idade = Fill in the age in Budget › Retirement to see the effect.
planejamento-cenarios-grafico-patrimonio = Net worth: scenario vs. baseline
planejamento-cenarios-fonte-patrimonio = Source: average of the last 12 months, today's balance and portfolio, scenario events · real values
planejamento-cenarios-grafico-patrimonio-aria = Net worth in the scenario and the baseline
planejamento-cenarios-tabela-patrimonio = Net worth by month
planejamento-cenarios-mes = Month
planejamento-cenarios-grafico-saldo = Next 12 months: account balance
planejamento-cenarios-saldo-negativo = The balance goes negative on { $data } in the scenario
planejamento-cenarios-saldo-positivo = The account balance doesn't go negative in the scenario
planejamento-cenarios-grafico-saldo-aria = Balance forecast in the scenario and the baseline
planejamento-cenarios-tabela-saldo = Balance forecast with the scenario
planejamento-cenarios-aviso = Simulation, not financial advice.
planejamento-cenarios-titulo = "What if" scenarios
planejamento-cenarios-intro = Build a scenario with life events and compare it with what happens if nothing changes: net worth over the years, the account balance over the next 12 months and the chance of retirement working out.
planejamento-cenarios-salvos = Saved:
planejamento-cenarios-apagar = Delete { $nome }
planejamento-cenarios-evento = Life event
planejamento-cenarios-horizonte = Horizon
planejamento-cenarios-nome = Scenario name
planejamento-cenarios-nome-exemplo = Apartment in 2027
planejamento-cenarios-salvo = Scenario saved
planejamento-cenarios-limpar = Clear
planejamento-assinaturas-tipo = Type
planejamento-assinaturas-conta-fixa = Fixed bill
planejamento-assinaturas-tipo-assinatura = Subscription
planejamento-assinaturas-contas-fixas-mes = Fixed bills per month
