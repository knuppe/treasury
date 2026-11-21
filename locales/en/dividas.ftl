# Screen 07 · Debts: loans, schedule, charts, simulators, installment payments, statement and
# market rates (docs/i18n-design.md). Summaries and warnings from the core are not here.
# Revision: AI draft, not yet reviewed by a native speaker.

## Repeated labels

dividas-contrato = Loan
dividas-contrato-min = loan
dividas-um-contrato = a loan
dividas-contratos = Loans
dividas-novo-contrato = New loan
dividas-saldo-devedor = Outstanding balance
dividas-proxima-parcela = Next installment
dividas-parcela = Installment
dividas-parcelas = Installments
dividas-juros = Interest
dividas-juros-a-pagar = Interest to pay
dividas-amortizacao = Principal
dividas-amortizacao-extraordinaria = Extra payment
dividas-com-simulada = With the simulated extra payment
dividas-correcao = Adjustment
dividas-correcao-saldo = Balance adjustment
dividas-seguros-taxas = Insurance and fees
dividas-sistema = System
dividas-taxa = Rate
dividas-data = Date
dividas-valor = Amount
dividas-ano = Year
dividas-hoje = Today
dividas-vencimento = Due date
dividas-situacao = Status
dividas-diferenca = Difference
dividas-acoes = Actions
dividas-col-numero = No.
dividas-data-pagamento = Payment date
dividas-conta-de-debito = Debit account
dividas-conta-de-debito-min = debit account
dividas-categoria-pagamentos = Payment category
dividas-custos-troca = Switching costs
dividas-parcelas-que-faltam = Installments left
dividas-prazo-meses = Term (months)
dividas-calculando = Calculating…
dividas-sem-descricao = no description
dividas-opcional = Optional.
dividas-nenhuma = None
dividas-nenhuma-min = none
dividas-arquivado = archived
dividas-arquivar = Archive
dividas-reativar = Reactivate
dividas-excluir = Delete
dividas-desligar = Unlink
dividas-estimado = ≈ estimated
dividas-quitado = Paid off
dividas-quitado-min = paid off
dividas-atrasada-alerta = ! overdue
dividas-parcela-n-data = Installment { $n } · { $data }
dividas-parcela-original = Installment { $n } of the original loan
dividas-n-meses =
    { $n ->
        [one] { $n } month
       *[other] { $n } months
    }

## Rates (p.a. = per year, p.m. = per month)

dividas-taxa-aa = { $taxa } p.a.
dividas-taxa-am = { $taxa } p.m.
dividas-taxa-linear = { $taxa } (linear)
dividas-taxa-pct-cdi = { $pct } of CDI
dividas-taxa-pct-selic = { $pct } of Selic

## Term and installment status

dividas-prazo-parcelas =
    { $n ->
        [one] { $n } installment
       *[other] { $n } installments
    }
dividas-prazo-longo =
    { $meses ->
        [0] { $n } installments ({ $anos } years)
        [one] { $n } installments ({ $anos } years and { $meses } month)
       *[other] { $n } installments ({ $anos } years and { $meses } months)
    }
dividas-status-paga = Paid
dividas-status-atrasada = Overdue
dividas-status-proxima = Next
dividas-status-prevista = Scheduled
dividas-parcelas-atrasadas =
    { $n ->
        [one] { $n } overdue installment
       *[other] { $n } overdue installments
    }
dividas-igual = same
dividas-pago-a-mais = Paid more than the installment
dividas-pago-a-menos = Paid less than the installment

## Loan types and core enums

dividas-tipo-imobiliario = Mortgage
dividas-tipo-veiculo = Car loan
dividas-tipo-pessoal = Personal loan
dividas-tipo-consignado = Payroll loan
dividas-tipo-estudantil = Student loan
dividas-tipo-outro = Other

dividas-sistema-sac-dica = Constant principal: the installment starts higher and goes down over time.
dividas-sistema-price-dica = Constant installment (no index): at first it is almost all interest. With TR or IPCA, the balance is adjusted first and the installment is recalculated every month.
dividas-sistema-bullet-dica = Interest only on each due date and the whole principal in the last installment.
dividas-sistema-sac-dica-pos = Constant principal + the period's interest from the index: the installment varies.
dividas-sistema-price-dica-pos = Installment recalculated on each due date with the period's index.
dividas-sistema-bullet-dica-pos = Only the period's interest on each due date; the principal comes at the end.

dividas-indexador-nenhum = Fixed rate
dividas-indexador-poupanca = Savings rate

dividas-prorata-nenhum = No pro rata (full month)
dividas-prorata-corridos = By calendar days
dividas-prorata-uteis = By business days

dividas-base-mensal = Equivalent monthly
dividas-base-252 = Business days (252 basis)
dividas-base-365 = Calendar days (365 basis)

dividas-referencia-mes = Month's change
dividas-referencia-aniversario = Anniversary date

dividas-spread-exponencial = Exponential (252 business days)
dividas-spread-linear = Linear (360 calendar days)

dividas-modo-prazo = Shorten the term
dividas-modo-parcela = Lower the installment
dividas-modo-prazo-dica = Keeps the installment and shortens the loan: saves more interest.
dividas-modo-parcela-dica = Keeps the term and lowers the next installments: eases the month.

## Screen and loan menu

dividas-menu-abrir = Open loan
dividas-menu-amortizacao-extra = Extra payment…
dividas-menu-editar = Edit…
dividas-menu-excluir = Delete…
dividas-conciliar = Reconcile with payments
dividas-conciliar-sem-conta = Set the debit account or the payment category to reconcile
dividas-carregando-contratos = Loading loans
dividas-vazio-kicker = No loans
dividas-vazio-titulo = Add your first loan or mortgage
dividas-vazio-descricao = Enter the amount, rate, system (SAC or Price), term and index: the schedule comes out right away, with interest vs. principal, an extra payment simulator and the real cost after inflation.
dividas-mostrar-arquivados = Show archived

## Summary and loan list

dividas-resumo-saldo-total = Total outstanding balance
dividas-resumo-contratos-ativos =
    { $n ->
        [one] { $n } active loan
       *[other] { $n } active loans
    }
dividas-resumo-compromisso = Monthly commitment
dividas-resumo-proximas-somadas = Next installments added up
dividas-resumo-em-atraso = ! { $n } overdue
dividas-resumo-ate-o-fim = Until the loans end
dividas-resumo-pelo-cronograma = on the current schedule
dividas-acoes-de = Actions for { $nome }
dividas-col-taxa-aa = Rate p.a.
dividas-col-progresso = Progress
dividas-col-custo-real = Real cost
dividas-numero-contrato = no. { $numero }
dividas-pre = fixed
dividas-pos = floating
dividas-atrasada-em = ! overdue · { $data }

## Loan detail

dividas-conciliadas =
    { $n ->
        [one] { $n } installment reconciled
       *[other] { $n } installments reconciled
    }
dividas-conciliar-nada = No new payments found
dividas-conciliar-procurado = Searched in { $conta }: installment amount ±1% and up to 7 days from the due date.
dividas-conciliar-erro = Couldn't reconcile
dividas-reativado = Loan reactivated
dividas-arquivado-ok = Loan archived
dividas-contrato-aria = Loan { $nome }
dividas-de-membro = of { $nome }
dividas-contrato-de-data = loan from { $data }
dividas-contrato-numero-de-data = loan no. { $numero } from { $data }
dividas-importar-extrato = Import statement
dividas-amortizacao-extra = Extra payment
dividas-atrasadas-texto =
    { $n ->
        [one] Past due and with no payment recorded. If you already paid, reconcile with the debit account's transactions or mark it as paid in the table.
       *[other] Past due and with no payment recorded. If you already paid, reconcile with the debit account's transactions or mark them as paid in the table.
    }
dividas-marcadas-titulo = Installments marked as paid
dividas-marcadas-descricao =
    { $n ->
        [one] { $n } past-due installment, on the due date and for the scheduled amount.
       *[other] { $n } past-due installments, on the due date and for the scheduled amount.
    }
dividas-marcar-vencidas = Mark the { $n } past-due ones as paid
dividas-de-financiados = of { $valor } borrowed
dividas-parcelas-pagas-de = { $pagas } of { $total } installments paid
dividas-numero-data = No. { $n } · { $data }
dividas-em-aberto = { $prazo } open
dividas-taxa-custo = Rate and cost
dividas-sistema-pre = fixed rate
dividas-sistema-pos = floating, interest from { $indice } accrued over business days
dividas-sistema-pos-defasagem = floating, interest from { $indice } accrued over business days (D−{ $dias })
dividas-sistema-correcao = { $indice } adjustment ({ $detalhes })
dividas-correcao-aniversario = on the anniversary
dividas-correcao-mes = same month
dividas-correcao-piso = zero floor
dividas-correcao-prorata = 1st period pro rata
dividas-cet-contratado = Contracted CET
dividas-custo-realizado = Actual cost
dividas-cet-realizado = Actual CET
dividas-custo-real-ipca = Real cost (− IPCA)
dividas-juros-pagos = Interest paid
dividas-seguros-taxas-pagos = Insurance and fees paid
dividas-montando-cronograma = Building the schedule
dividas-simulador-amortizacao = Extra payment simulator
dividas-antecipar-parcelas = Prepay installments
dividas-simulador-portabilidade = Loan portability simulator
dividas-tabela-amortizacao = Amortization table
dividas-seguros-taxas-mes = Insurance + fees/month
dividas-origem = Source
dividas-origem-valor = core schedule · ≈ estimated index
dividas-observacoes-valor = Notes: { $notas }
dividas-excluir-titulo = Delete { $nome }
dividas-excluir-descricao = The loan, the schedule and the recorded extra payments are removed. The accounts' transactions stay. You can undo it from the change history.
dividas-excluir-confirmar = Delete loan
dividas-excluido = Loan deleted

## Charts

dividas-grafico-saldo-estimado = Outstanding balance (≈ estimated index)
dividas-grafico-parcelas-n = Installments ({ $n })
dividas-grafico-parcela-estimada = Installment { $n } · { $data } · ≈ estimated index
dividas-grafico-correcao-nota = The adjustment is not part of the installment: it is added to the outstanding balance.
dividas-figura-saldo-titulo = Outstanding balance over the loan
dividas-figura-saldo-fonte = Loan schedule
dividas-figura-saldo-fonte-indice = Loan schedule · adjusted by the index (≈ months without official data use the last index)
dividas-figura-saldo-aria = Outstanding balance of { $nome } over the loan.
dividas-figura-saldo-aria-simulada = Outstanding balance of { $nome } over the loan, with the simulated extra payment.
dividas-figura-saldo-tabela = Outstanding balance by due date
dividas-figura-saldo-col = Balance
dividas-figura-composicao-titulo =
    { $modo ->
        [ano] Interest vs. principal by year
       *[parcela] Interest vs. principal by installment
    }
dividas-figura-composicao-fonte-correcao = The index adjustment of the balance is not paid in the installment: it goes into the outstanding balance and is paid down later.
dividas-figura-composicao-fonte = Breakdown of each installment in the schedule.
dividas-figura-composicao-agrupar = Group the breakdown
dividas-figura-composicao-aria =
    { $modo ->
        [ano] Breakdown of the installments of { $nome } by year: { $partes }.
       *[parcela] Breakdown of the installments of { $nome } by installment: { $partes }.
    }
dividas-figura-composicao-tabela =
    { $modo ->
        [ano] Breakdown by year
       *[parcela] Breakdown by installment
    }
dividas-figura-pago-no-ano = Paid in the year

## Market rates (Banco Central)

dividas-mercado-titulo = Market interest rates
dividas-mercado-intro = What banks charge for each type of credit, from Banco Central data. It helps you see whether your loan is expensive and whether it is worth negotiating or moving the debt to another bank (portability).
dividas-mercado-abaixo = Below the market
dividas-mercado-media = Market average
dividas-mercado-acima = Above the market
dividas-mercado-bem-acima = Well above the market
dividas-mercado-seus-contratos = Your loans vs. the market
dividas-mercado-taxas = Market rates
dividas-mercado-credito = Credit
dividas-mercado-mais-barato = Cheapest (p.m.)
dividas-mercado-mediana = Median (p.m.)
dividas-mercado-mais-caro = Most expensive (p.m.)
dividas-mercado-bancos = Cheapest banks
dividas-mercado-ao-ano = { $taxa } per year
dividas-mercado-atualizado = Updated on { $data }.
dividas-mercado-atualizar = Update

## Prepay installments

dividas-antecipar-dica-ultimas = Pays off the last ones: the loan ends sooner and the discount is bigger.
dividas-antecipar-dica-proximas = Pays the next ones now: the discount is smaller and the term stays the same.
dividas-antecipar-em-aberto = Open: { $n }
dividas-antecipar-ordem = Order
dividas-antecipar-ordem-aria = Prepayment order
dividas-antecipar-ultimas = From the last
dividas-antecipar-proximas = From the next
dividas-antecipar-informe = Enter how many installments to prepay.
dividas-antecipar-resumo = Paying { $valor } on { $data }, the interest discount is { $desconto }.
dividas-antecipar-resumo-encargos = Paying { $valor } on { $data }, the interest discount is { $desconto } and { $encargos } of insurance and fees are no longer charged.
dividas-antecipar-valor-cronograma = Scheduled amount
dividas-antecipar-a-pagar = To pay now
dividas-antecipar-ficam = Still open
dividas-antecipadas = Installments prepaid
dividas-antecipadas-descricao =
    { $n ->
        [one] { $n } installment for { $valor }
       *[other] { $n } installments for { $valor }
    }
dividas-antecipar-registrar = Record prepayment
dividas-antecipar-com-conta = The outflow is recorded in the loan's debit account.
dividas-antecipar-sem-conta = No debit account: only the installments are marked as paid.

## Extra payment simulator

dividas-simulador-valor = Extra payment amount
dividas-simulador-acima-saldo = Above the outstanding balance ({ $saldo }): pays off the loan.
dividas-simulador-saldo-hoje = Outstanding balance today: { $saldo }
dividas-simulador-informe = Enter an amount to compare the two ways to pay down. The simulation doesn't save anything.
dividas-simulador-forma = How to pay down
dividas-simulador-juros-economizados = Interest saved
dividas-simulador-parcelas-restantes = Installments left
dividas-simulador-a-menos = { $prazo } fewer
dividas-simulador-quitacao = Payoff
dividas-simulador-nota = Simulation on the current schedule. Check the amounts with the bank before paying.
dividas-simulador-nota-indice = Simulation on the current schedule, with the index projected from the last known one. Check the amounts with the bank before paying.
dividas-simulador-registrar = Record this extra payment

## Portability simulator

dividas-portabilidade-nova-taxa = New rate (% p.a.)
dividas-portabilidade-hoje = Today: { $taxa }
dividas-portabilidade-prazo-erro = Between 1 and 600 months.
dividas-portabilidade-restam = { $n } left
dividas-portabilidade-sistema-dica = Of the new loan.
dividas-portabilidade-sistema-aria = New loan's system
dividas-portabilidade-custos-dica = Fees, appraisal, registration.
dividas-portabilidade-informe = Enter the rate offered by the other bank to compare. The simulation doesn't save anything.
dividas-portabilidade-vale = Worth it: you save { $valor } in total, switching costs included.
dividas-portabilidade-nao-vale = Not worth it: it costs { $valor } more in total, switching costs included.
dividas-portabilidade-saldo-levado = Balance moved
dividas-portabilidade-falta-atual = Left to pay on the current loan
dividas-portabilidade-total-nova = Total on the new loan
dividas-portabilidade-novo-prazo = New term
dividas-portabilidade-nota = The new loan is calculated without an index (fixed rate). Current loans with TR or IPCA may have a different real cost.

## Link an installment to a transaction

dividas-ligar-ok = Installment { $n } linked to the transaction
dividas-descricao = Description
dividas-conta = Account
dividas-conta-debito-contrato = Loan's debit account
dividas-debito = debit
dividas-ligar-titulo = Link installment { $n } to a transaction
dividas-ligar-descricao = { $contrato } · due on { $data } · installment of { $valor }. Outflows up to 20 days from the due date that don't pay another installment yet, closest amounts first. The installment is marked as paid on the transaction's date and for its amount.
dividas-ligar-trocar = Switch to this transaction
dividas-ligar-ao-lancamento = Link to transaction
dividas-lancamento-ligado = Linked transaction
dividas-paga-pelo-lancamento = Paid by the transaction
dividas-desligada = Installment { $n } unlinked from the transaction and open
dividas-desligar-lancamento = Unlink the transaction
dividas-ligar-procurando = Looking for outflows near the due date…
dividas-ligar-tabela = Transactions for installment { $n }
dividas-ligar-vazio = No outflow without an installment within 20 days of the due date. Import the account statement or enter the payment in Transactions.

## Amortization table

dividas-paga-pelo-lancamento-titulo = Paid by the transaction { $lancamento }
dividas-paga-pelo-lancamento-titulo-conta = Paid by the transaction { $lancamento } in { $conta }
dividas-mais-extra = + extra { $valor }
dividas-juros-periodo = Period interest
dividas-juros-periodo-dica = Index accrued over the period's business days × spread
dividas-dias-uteis =
    { $n ->
        [one] { $n } business day
       *[other] { $n } business days
    }
dividas-encargos-detalhe = MIP { $mip } · DFI { $dfi } · other insurance { $outros } · fees { $taxas }
dividas-pago-valor = paid { $valor }
dividas-saldo-apos = Balance after
dividas-paga-em = Paid on { $data }
dividas-indice-estimado-dica = The month's index isn't out yet: uses the last known one
dividas-indice-estimado = estimated index
dividas-ligar-reticencias = Link…
dividas-desligar-parcela = Unlink the transaction from installment { $n }
dividas-desfazer-parcela = Undo payment of installment { $n }
dividas-desfazer = Undo
dividas-marcar-parcela-aria = Mark installment { $n } as paid
dividas-marcar-paga = Mark paid
dividas-menu-marcar-paga = Mark as paid…
dividas-menu-ligar-outro = Link to another transaction…
dividas-menu-ligar-um = Link to a transaction…
dividas-menu-ver-lancamento = See the transaction
dividas-menu-desfazer = Undo the payment
dividas-filtro-aria = Installments to show
dividas-filtro-proximas = Upcoming
dividas-filtro-abertas = Open · { $n }
dividas-filtro-pagas = Paid · { $n }
dividas-filtro-todas = All · { $n }
dividas-tabela-de = Amortization table · { $nome }
dividas-acoes-parcela = Actions for installment { $n }
dividas-filtro-vazio = No installments in this filter.

## Mark an installment as paid and undo

dividas-paga-sincronizada = Installment { $n } paid and balance synced
dividas-marcada-paga = Installment { $n } marked as paid
dividas-pagar-titulo = Installment { $n } paid
dividas-pagar-descricao = Due on { $data }. Marks the installment as paid without a transaction; if the payment is already in Transactions, use “Link…” on the installment (or “Reconcile with payments” for all of them).
dividas-marcar-como-paga = Mark as paid
dividas-valor-pago = Amount paid
dividas-valor-pago-dica = Including late fees and interest, if any.
dividas-saldo-depois = Outstanding balance after the payment
dividas-saldo-depois-dica = Optional, from the receipt: syncs the balance and recalculates the next installments.
dividas-amortizacao-parcela = Installment principal
dividas-amortizacao-parcela-dica = With the balance, separates the extra payment (opens a new version of the loan).
dividas-faltam-dica = From the receipt; empty keeps the current term.
dividas-desligar-descricao = The installment is no longer paid by the transaction { $lancamento } and becomes open again; the transaction stays in the account. You can undo it from the change history.
dividas-desfazer-descricao = The installment becomes open again. You can undo it from the change history.
dividas-desfazer-pagamento = Undo payment
dividas-em-aberto-de-novo = Installment { $n } open again

## Loan statement

dividas-extrato-importadas =
    { $n ->
        [one] { $n } installment recorded from the statement
       *[other] { $n } installments recorded from the statement
    }
dividas-extrato-projecao =
    { $n ->
        [one] Projection from the balance on { $data }, with { $n } installment left.
       *[other] Projection from the balance on { $data }, with { $n } installments left.
    }
dividas-extrato-titulo = Import loan statement
dividas-extrato-descricao = Paste the bank's history (from the spreadsheet or online banking), with the header. The installments take the bank's amounts, extra payments are found from the balance and the future schedule starts from the last balance. You can also send the image or the PDF to the agent.
dividas-extrato-conferir = Check
dividas-extrato-importar = Import
dividas-extrato-importar-n =
    { $n ->
        [one] Import { $n } installment
       *[other] Import { $n } installments
    }
dividas-extrato-colado = Pasted statement
dividas-extrato-colado-dica = Columns: date, installment, principal, interest, insurance, fees, outstanding balance, monetary adjustment and amount paid (the header names are enough; the column order of the Caixa spreadsheet is also recognized).
dividas-extrato-faltam-dica = After the last row. Empty: calculated from the last installment.
dividas-extrato-marcar-anteriores = Mark the installments before the statement as paid
dividas-extrato-previa = Statement preview
dividas-extrato-periodo = Period
dividas-extrato-periodo-valor = { $de } to { $ate }
dividas-extrato-ultimo-saldo = Last balance
dividas-extrato-prazo-restante = Remaining term
dividas-extrato-juros = Statement interest
dividas-extrato-juros-valor = { $mensal }% p.m. ({ $nominal }% nominal, { $efetiva }% effective p.a.)
dividas-extrato-extras = Extra payments found
dividas-extrato-extras-aria = Extra payments found
dividas-extrato-avisos = Warnings
dividas-extrato-taxa-ajustada = Rate set to { $taxa }% effective p.a.
dividas-extrato-usar-taxa = Use the statement's rate ({ $taxa }% effective)

## Installment payment from a transaction

dividas-pagamento-so-saida = Only an outflow (negative amount) can pay an installment.
dividas-pagamento-transferencia = It's a transfer between your accounts: undo the transfer to link it to the installment.
dividas-pagamento-previsto = The transaction is still scheduled: confirm it before linking it to the installment.
dividas-pagamento-ligado-ok = Transaction linked to installment { $n } of { $contrato }
dividas-pagamento-paga-em = Installment paid on { $data }.
dividas-pagamento-desligado = Transaction unlinked from installment { $n } of { $contrato }
dividas-pagamento-voltou = The installment is open again.
dividas-pagamento-este = This transaction
dividas-pagamento-titulo = Installment payment
dividas-pagamento-trocar-para = Switch to installment { $n }
dividas-pagamento-ligar-n = Link to installment { $n }
dividas-pagamento-ligar = Link to installment
dividas-pagamento-saiu-de = left { $conta }
dividas-pagamento-ligado = This transaction pays installment { $n } of { $contrato }.
dividas-pagamento-ligado-texto = Pick another installment to switch, or unlink: the installment becomes open again.
dividas-pagamento-carregando-contratos = Loading loans…
dividas-pagamento-sem-contrato = No active loans. Add the loan or mortgage in { $tela }.
dividas-pagamento-sem-contrato-moeda = No active loans in { $moeda }. Add the loan or mortgage in { $tela }.
dividas-pagamento-escolha = Choose the loan
dividas-pagamento-carregando-parcelas = Loading installments…
dividas-pagamento-parcelas-de = Installments of { $nome }
dividas-pagamento-vazio = No installments without a transaction in this loan.
dividas-pagamento-nota = The 12 installments without a transaction due closest to { $data }. The chosen one is marked as paid on { $data } for the transaction's amount.
dividas-pagamento-nota-categoria = The 12 installments without a transaction due closest to { $data }. The chosen one is marked as paid on { $data } for the transaction's amount, and the transaction gets the loan's payment category.

## Add and edit a loan

dividas-form-editar = Edit loan
dividas-form-editar-descricao = Fix any field: the open installments are recalculated. Paid installments (with the amount and date you recorded), extra payments and prepayments stay as they are.
dividas-form-novo-descricao = Loan or mortgage: the schedule (SAC or Price, with grace period, index, insurance and fees) is created right away.
dividas-form-erro-nome = Give the loan a name.
dividas-form-erro-principal = Enter the amount borrowed.
dividas-form-erro-parcela = Enter the installment amount.
dividas-form-erro-parcela-nao-fecha = With this installment and this term the rate doesn't work out: the installment times the term must be more than the amount borrowed.
dividas-form-erro-taxa = Enter the effective annual rate.
dividas-form-erro-taxa-intervalo = Rate out of range.
dividas-form-erro-data-contrato = Enter the loan date.
dividas-form-erro-primeiro-vencimento = Enter the first due date.
dividas-form-erro-vencimento-antes = The first installment can't be due before the loan.
dividas-form-erro-prazo = Enter the term in months.
dividas-form-erro-carencia = Grace period in months (0 if none).
dividas-form-erro-carencia-prazo = The grace period must be shorter than the term.
dividas-form-erro-percentual = Index percentage between 0 and 1000%.
dividas-form-erro-defasagem = Lag of 0 to 10 business days.
dividas-form-erro-bullet = A bullet loan has no grace period: the installment is already interest only.
dividas-form-erro-valor = Invalid amount.
dividas-form-alterado = Loan changed
dividas-form-cadastrado = Loan added
dividas-form-cadastrado-descricao =
    { $n ->
        [one] { $nome }: { $n } installment in the schedule
       *[other] { $nome }: { $n } installments in the schedule
    }
dividas-form-dados = Loan details
dividas-form-nome = Loan name
dividas-form-nome-exemplo = Apartment mortgage
dividas-form-numero = Loan number
dividas-form-numero-dica = As shown on the contract or in online banking. Optional.
dividas-form-principal = Amount borrowed
dividas-form-primeiro-vencimento = First due date
dividas-form-sistema = Amortization system
dividas-form-tipo-taxa = Rate type
dividas-form-pre = Fixed
dividas-form-pos = Floating (CDI, Selic)
dividas-form-indice = Index
dividas-form-indice-dica = Each period's interest from the index accrued over business days (B3, 252 basis).
dividas-form-percentual = Index percentage
dividas-form-percentual-dica = 100% in most loans; e.g. 120% of CDI.
dividas-form-spread = Spread (% p.a.)
dividas-form-spread-dica = Rate above the index: { $indice } + spread. Leave empty if it's only the percentage.
dividas-form-taxa-juros = Interest rate
dividas-form-pela-parcela = Calculate from the installment
dividas-form-informar-taxa = Enter the rate
dividas-form-primeira-parcela = First installment
dividas-form-parcela-juros = Interest installment
dividas-form-valor-parcela = Installment amount
dividas-form-parcela-dica-encargos = As on the bill or the statement: the monthly insurance and fees (below) come out of it.
dividas-form-parcela-dica = As on the bill or the statement, with insurance and fees (enter them below).
dividas-form-taxa-calculada = Calculated rate
dividas-form-taxa-nominal = Nominal rate (% p.a.)
dividas-form-taxa-efetiva = Effective rate (% p.a.)
dividas-form-como-taxa = How the contract states the rate
dividas-form-efetiva = Effective
dividas-form-nominal = Nominal
dividas-form-nominal-dica = Nominal per year (the monthly rate is the annual ÷ 12, as at Caixa).
dividas-form-nominal-dica-equivale = Nominal per year (the monthly rate is the annual ÷ 12, as at Caixa). Equal to { $taxa }% effective p.a.
dividas-form-efetiva-dica = The loan's interest rate, without the index. The monthly rate is the equivalent.
dividas-form-detalhes = Details
dividas-form-tipo = Type
dividas-form-instituicao = Institution
dividas-form-nao-informar = Don't say
dividas-form-membro = Family member
dividas-form-familia = Family
dividas-form-data-contrato = Loan date
dividas-form-data-contrato-dica = One month before the 1st due date.
dividas-form-forma-spread = Spread type
dividas-form-forma-spread-dica = Exponential is the default (CCB, working capital); linear shows up in older contracts.
dividas-form-defasagem = Index lag (business days)
dividas-form-defasagem-dica = 0 in most cases; some banks use the DI rate from 1 or 2 business days before.
dividas-form-correcao-dica = Monthly monetary adjustment of the balance (TR under SFH, IPCA or INCC for real estate).
dividas-form-juros-dica = How the rate (or the real interest) runs in each period. Monthly is the most common.
dividas-form-indice-usado = Index used
dividas-form-indice-aniversario-dica = The one for the period that starts on the loan's anniversary (default for TR and savings rate).
dividas-form-indice-mes-dica = A month's change, with the lag next to it (IPCA comes out the following month).
dividas-form-mes-indice = Index month
dividas-form-mes-indice-dica = A due date in M uses the index of M − n.
dividas-form-mesmo-mes = Same month (M)
dividas-form-deflacao = Ignore deflation
dividas-form-deflacao-dica = Months with a negative index don't lower the balance (zero floor).
dividas-form-prorata = 1st period pro rata
dividas-form-prorata-dica = When the signing is not exactly one month before the 1st due date.
dividas-form-prorata-pos = By the period's business days (floating)
dividas-form-carencia = Grace period (months)
dividas-form-carencia-dica = Interest only during this period.
dividas-form-valor-imovel = Property value
dividas-form-valor-imovel-dica = Basis for the DFI insurance (property damage).
dividas-form-mip-taxa = MIP insurance (% p.m.)
dividas-form-mip-valor = MIP insurance ({ $moeda } per month)
dividas-form-como-mip = How the contract states MIP
dividas-form-mip-taxa-dica = Death and disability, on the outstanding balance: it goes down with the debt.
dividas-form-mip-taxa-dica-hoje = Death and disability, on the outstanding balance: it goes down with the debt. Today ≈ { $valor } per month.
dividas-form-mip-valor-dica = Fixed death and disability amount in every installment, as in the contract.
dividas-form-mip-valor-dica-equivale = Fixed death and disability amount in every installment, as in the contract. Equal to { $pct }% p.m. of the { $saldo } balance.
dividas-form-dfi-taxa = DFI insurance (% p.m.)
dividas-form-dfi-valor = DFI insurance ({ $moeda } per month)
dividas-form-como-dfi = How the contract states DFI
dividas-form-dfi-taxa-dica = Property damage, on the property value.
dividas-form-dfi-taxa-dica-mes = Property damage, on the property value. ≈ { $valor } per month.
dividas-form-dfi-valor-dica = Fixed property damage amount in every installment (doesn't need the property value).
dividas-form-dfi-valor-dica-equivale = Fixed property damage amount in every installment (doesn't need the property value). Equal to { $pct }% p.m. of the property.
dividas-form-outros-seguros = Other insurance per month
dividas-form-seguros = Insurance per month
dividas-form-outros-seguros-dica = Fixed amount, on top of MIP and DFI.
dividas-form-seguros-dica = Credit life and others.
dividas-form-taxa-adm = Administration fee per month
dividas-form-taxa-adm-dica = The loan's monthly fee (for a mortgage, usually R$ 25).
dividas-form-cet = Contracted CET (% p.a.)
dividas-form-cet-dica = From the contract or the offer. Optional.
dividas-form-conta-dica = To reconcile the installments with transactions.
dividas-form-categoria-dica = A transaction in this category (imported or typed in) marks the closest installment as paid.
dividas-form-observacoes = Notes
dividas-form-ja-paguei =
    { $n ->
        [one] I already paid the { $n } installment due up to today
       *[other] I already paid the { $n } installments due up to today
    }
dividas-form-ja-paguei-dica = They are marked as paid on the due date, for the scheduled amount. Turn it off to mark them one by one (or reconcile with the account's transactions).
dividas-form-pos-nota = The installment varies with { $indice }: when you save, the schedule uses the rates already published and, for the future, the last known one (installments marked as estimated and redone each time indexes are collected).
dividas-form-na-carencia = During the grace period
dividas-form-ultima-parcela = Last installment
dividas-form-correcao-valor = + { $indice } on the balance
dividas-form-salvar = Save changes
dividas-form-cadastrar = Add loan

## Extra payment

dividas-extra-descricao = { $nome }: the payment reduces the outstanding balance and the open schedule is recalculated. You can undo it from the change history.
dividas-extra-erro-valor = Enter the amount.
dividas-extra-erro-data = Enter the date (mm/dd/yyyy).
dividas-extra-registrada = Extra payment recorded
dividas-extra-registrada-descricao = { $nome }: outstanding balance and schedule recalculated
dividas-extra-depois = After the extra payment
dividas-extra-sai-da-conta = Paid from
dividas-extra-sem-conta-dica = Records it only on the loan, without a transaction (e.g. FGTS or a payment by someone else).
dividas-extra-com-conta-dica = Creates the outflow transaction in this account.
dividas-extra-sem-lancamento = No transaction (FGTS, other source)
dividas-extra-fgts-saldo = FGTS account, balance { $saldo }
dividas-extra-fgts-descricao = The money left the FGTS account, as a housing withdrawal.
dividas-extra-data-passada = Past date ({ $data }): it goes in after the last installment due up to then.
dividas-extra-registrar = Record extra payment
