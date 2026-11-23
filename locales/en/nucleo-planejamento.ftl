# Planning (core: `forecast`, `scenarios`, `debt`, `pension`, `finance`): balance forecast and
# "spend today", scenarios, debts (schedule, payoff, market rates, statements and portability),
# private pension and errors. Built on the fly, in the installation language. Errors in
# lowercase, like a log line (the interface capitalizes them). INSS, IR, VGBL, PGBL, EFPC, CNPJ,
# CVM, BCB, SAC, MIP, DFI, CET, TR and IPCA stay as they are.
# Revision: AI draft, not yet reviewed by a native speaker.

## Balance forecast and how much you can spend today (`forecast`)

nucleo-planejamento-conta-ou-arquivada = account (or archived)
nucleo-planejamento-fatura = { $cartao } bill
nucleo-planejamento-parcela = { $contrato } installment
nucleo-planejamento-sazonal = { $descricao } (estimate from the { $ano } amount)
nucleo-planejamento-gastar-hoje =
    This month's income { $receitas } − spent before today { $gastos } − bills still due { $contas } − left to save { $guardar } = { $disponivel } for { $dias ->
        [one] { $dias } day
       *[other] { $dias } days
    }: { $por_dia } a day.
nucleo-planejamento-gastar-amanha = Today's spending so far is { $hoje }; from tomorrow on, { $amanha } a day.
nucleo-planejamento-gastar-sem-renda = No income received or expected this month yet. Without it, there is nothing to divide by the days left.

## "What if" scenarios (`scenarios`)

nucleo-planejamento-cenario-nome = Scenario
nucleo-planejamento-cenario-explicacao =
    In { $anos ->
        [one] { $anos } year
       *[other] { $anos } years
    }, net worth is { $cenario } in the scenario versus { $base } in the baseline ({ $diferenca } difference; { $liquido } in accounts and investments).
nucleo-planejamento-cenario-dinheiro-acaba = The available money runs out in { $mes }.
nucleo-planejamento-cenario-saldo-negativo = In the next 12 months, the account balance goes negative on { $data }.
nucleo-planejamento-cenario-aposentadoria = Retirement: chance of success from { $base } to { $cenario }.
nucleo-planejamento-cenario-nota-premissas = Today's real values; income of { $renda } and spending of { $gasto } a month (average of the last 12 months), real return of { $retorno } a year.
nucleo-planejamento-cenario-nota-bens = Existing assets and debts stay at today's value; current installments are already in the average spending.
nucleo-planejamento-cenario-nota-sem-idade = No age in the retirement plan (Budget › Retirement): the effect on retirement is left out.
nucleo-planejamento-cenario-erro-evento-sem-nome = each event needs a name
nucleo-planejamento-cenario-erro-renda = { $evento }: income from 0% to 1000%
nucleo-planejamento-cenario-erro-financiamento = { $evento }: financing with a valid amount, term (1 to 600 months) and rate
nucleo-planejamento-cenario-erro-bem = { $evento }: the asset value must be positive
nucleo-planejamento-cenario-erro-so-interface = saved scenarios only through the app's interface
nucleo-planejamento-cenario-erro-sem-nome = give the scenario a name

## Private pension: plan, withdrawal and statement (`pension`). The texts of the statement rows
## ("Contribuição do participante"…) stay as in the document.

nucleo-planejamento-previdencia-plano = plan
nucleo-planejamento-previdencia-taxa-administracao = management fee
nucleo-planejamento-previdencia-taxa-carregamento = loading fee
nucleo-planejamento-previdencia-erro-taxa-invalida = invalid { $taxa }: { $valor }
nucleo-planejamento-previdencia-erro-taxa-faixa = { $taxa } must be between 0 and 100%
nucleo-planejamento-previdencia-erro-classe = the pension plan is for assets in the Pension plans class
nucleo-planejamento-previdencia-erro-resgate-positivo = enter a positive withdrawal amount
nucleo-planejamento-previdencia-erro-resgate-saldo = the withdrawal exceeds the plan balance ({ $saldo })
nucleo-planejamento-previdencia-nota-vgbl-regressivo = Regressive VGBL: final IR only on the earnings, by the age of each contribution (the oldest go out first).
nucleo-planejamento-previdencia-nota-pgbl-regressivo = Regressive PGBL: final IR on the whole amount, by the age of each contribution (the oldest go out first).
nucleo-planejamento-previdencia-nota-vgbl-progressivo = Progressive VGBL: 15% of the earnings withheld at source; the final IR comes from the annual table in the tax return.
nucleo-planejamento-previdencia-nota-efpc-regressivo = Pension fund (EFPC), regressive: final IR on the whole amount, by the age of each contribution (the oldest go out first).
nucleo-planejamento-previdencia-nota-efpc-progressivo = Pension fund (EFPC), progressive: 15% of the whole amount withheld at source; the final IR comes from the annual table in the tax return.
nucleo-planejamento-previdencia-nota-pgbl-progressivo = Progressive PGBL: 15% of the whole amount withheld at source; the final IR comes from the annual table in the tax return.
nucleo-planejamento-previdencia-aviso-sem-plano = enter the plan type (VGBL or PGBL) and the tax regime so the IR comes out right; for now the calculation assumes regressive VGBL
nucleo-planejamento-previdencia-aviso-sem-cota = no recent unit price: today's value uses the price of the last contribution. Enter the fund's CNPJ (CVM data) or the current unit price
nucleo-planejamento-previdencia-motivo-plano = Pension plan: { $nome }
nucleo-planejamento-previdencia-motivo-extrato = Pension statement imported
nucleo-planejamento-previdencia-erro-pdf = couldn't read this PDF: { $erro }
nucleo-planejamento-previdencia-erro-sem-contribuicoes = no contributions recognized in this statement (lines "MM/YYYY DD/MM/YYYY DD/MM/YYYY description amounts")
nucleo-planejamento-previdencia-erro-sem-arquivo = send the statement PDF or its text
nucleo-planejamento-previdencia-erro-extrato-classe = the pension statement goes into an asset in the Pension plans class
nucleo-planejamento-previdencia-erro-conta = choose the account for the pension units
nucleo-planejamento-previdencia-erro-interno = pension statement: { $erro }
nucleo-planejamento-previdencia-aviso-cotas = the sum of the contribution units ({ $soma }) doesn't match the unit balance on the statement ({ $saldo }): check whether a row was left out
nucleo-planejamento-previdencia-aviso-saldo = the balance from the units ({ $calculado }) differs from the participant balance on the statement ({ $extrato })
nucleo-planejamento-previdencia-aviso-sem-cnpj = couldn't find the plan's CNPJ on the statement
# Name of the asset created by the statement when it doesn't include the plan name.
nucleo-planejamento-previdencia-ativo-padrao = Pension (pension fund)
# Notes saved on the contribution and on the risk expense ("Contribuição do participante (reference month 03/2026)").
nucleo-planejamento-previdencia-nota-aporte = { $descricao } (reference month { $competencia })
nucleo-planejamento-previdencia-nota-risco = Pension { $plano } (reference month { $competencia })

## Portfolio average cost (`finance::average_cost`)

nucleo-planejamento-erro-compra-positiva = the purchase quantity must be positive
nucleo-planejamento-erro-venda-positiva = the sale quantity must be positive
nucleo-planejamento-erro-venda-maior = sale larger than the position (short selling isn't supported)
nucleo-planejamento-erro-fator-desdobramento = invalid split/reverse split factor

## Debts: loans, installments, prepayment and portability (`debt`)

nucleo-planejamento-divida-emprestimo = loan { $id }
nucleo-planejamento-divida-contrato = loan
nucleo-planejamento-divida-parcela = installment
nucleo-planejamento-divida-categoria = category
nucleo-planejamento-divida-parcela-do-mes = installment for the payment month
# Field names in the invalid value errors ("invalid rate: abc").
nucleo-planejamento-divida-campo-taxa = rate
nucleo-planejamento-divida-campo-taxa-dfi = DFI rate
nucleo-planejamento-divida-campo-taxa-mip = MIP rate
nucleo-planejamento-divida-campo-percentual-indice = index percentage
nucleo-planejamento-divida-campo-cet = CET
nucleo-planejamento-divida-erro-invalido = invalid { $campo }: { $valor }
nucleo-planejamento-divida-erro-taxa-seguro = { $campo }: enter the monthly rate in % (e.g. 0.0247)
nucleo-planejamento-divida-erro-sem-nome = give the loan a name
nucleo-planejamento-divida-erro-valor-positivo = the financed amount must be positive
nucleo-planejamento-divida-erro-bullet-carencia = a bullet loan has no grace period: the installment is already interest only
nucleo-planejamento-divida-erro-percentual-indice = index percentage out of range (0 to 1000%)
nucleo-planejamento-divida-erro-defasagem-mensal = monthly index lag from 0 to 3 months
nucleo-planejamento-divida-erro-defasagem-diaria = index lag from 0 to 10 business days
nucleo-planejamento-divida-erro-posfixado-spread = in floating-rate loans the installment changes with the index: enter the spread (0 if there's none)
nucleo-planejamento-divida-erro-taxa-ou-parcela = enter the annual rate or the installment amount to calculate the rate
nucleo-planejamento-divida-erro-taxa-calculada = invalid calculated rate
nucleo-planejamento-divida-erro-taxa-intervalo = annual rate out of range
nucleo-planejamento-divida-erro-indexador-diferente = the rate has an index different from the one chosen for the contract
nucleo-planejamento-divida-erro-taxa-invalida = invalid rate: { $taxa }
nucleo-planejamento-divida-erro-primeira-parcela = the first installment can't be due before the loan date
nucleo-planejamento-divida-erro-prazo = invalid term (the grace period must be shorter than the term)
nucleo-planejamento-divida-erro-quantas-antecipar = enter how many installments to prepay
nucleo-planejamento-divida-erro-sem-futuras = there are no open future installments to prepay
nucleo-planejamento-divida-erro-poucas-futuras =
    { $n ->
        [one] there's only { $n } open future installment
       *[other] there are only { $n } open future installments
    }
nucleo-planejamento-divida-erro-parcela-seguros = the installment must be larger than the monthly insurance and fees
nucleo-planejamento-divida-erro-taxa-nao-fecha = with this installment and this term the rate doesn't work out: check the amounts (the installment times the term must exceed the financed amount)
nucleo-planejamento-divida-erro-pagamento-antes = the payment is before the loan's first due date
nucleo-planejamento-divida-erro-pagamento-negativo = payment amounts can't be negative
nucleo-planejamento-divida-erro-saldo-restante = there's still a balance owed: enter how many installments are left
nucleo-planejamento-divida-erro-amortizacao-positiva = the extra payment amount must be positive
nucleo-planejamento-divida-erro-imovel-negativo = the property value can't be negative
nucleo-planejamento-divida-erro-seguro-negativo = the insurance amount can't be negative
nucleo-planejamento-divida-erro-dfi-sem-imovel = the DFI as a percentage is calculated on the property value: enter the property value (or the DFI in reais)
nucleo-planejamento-divida-erro-categoria-despesa = the payments category must be an expense category
nucleo-planejamento-divida-erro-categoria-ligada = this category is already linked to the loan { $contrato }
nucleo-planejamento-divida-erro-conciliar-sem-conta = set the loan's debit account (or the payments category) to reconcile
nucleo-planejamento-divida-erro-pagamento-saida = an installment payment must be an outflow (negative amount)
nucleo-planejamento-divida-erro-pagamento-transferencia = the transaction is a transfer between your accounts and can't pay an installment
nucleo-planejamento-divida-erro-pagamento-previsto = the transaction is still planned; clear it before linking it to the installment
nucleo-planejamento-divida-erro-pagamento-moeda = the transaction is in { $moeda } and the loan in { $moeda_contrato }
nucleo-planejamento-divida-erro-pagamento-ja-ligado = the transaction is already linked to installment { $numero } of { $contrato }; unlink it there first
nucleo-planejamento-divida-erro-pagamento-amortizacao = the transaction is an extra payment of { $contrato }
# Author and reason in the change history when new index values recalculate the installments.
nucleo-planejamento-divida-autor-indices = indexes
nucleo-planejamento-divida-motivo-indices = Installments recalculated with the latest indexes
nucleo-planejamento-divida-motivo-extrato = Loan statement imported
nucleo-planejamento-divida-motivo-cadastrado = Loan added: { $nome }
nucleo-planejamento-divida-motivo-alterado = Loan changed
nucleo-planejamento-divida-motivo-removido = Loan removed
nucleo-planejamento-divida-motivo-amortizacao = Extra payment
nucleo-planejamento-divida-motivo-antecipadas = Installments prepaid
nucleo-planejamento-divida-motivo-vencidas-pagas = Past-due installments marked as paid
nucleo-planejamento-divida-motivo-conciliadas = Installments reconciled with the payments
nucleo-planejamento-divida-motivo-pagamento = Installment payment recorded
nucleo-planejamento-divida-motivo-parcela-paga = Installment paid
nucleo-planejamento-divida-motivo-parcela-ligada = Installment { $numero } of { $contrato } linked to the transaction
nucleo-planejamento-divida-motivo-pagamento-desfeito = Installment payment undone
# Description of the transactions created by the extra payment and by the prepayment.
nucleo-planejamento-divida-lancamento-amortizacao = Extra payment · { $contrato }
nucleo-planejamento-divida-lancamento-antecipacao =
    { $n ->
        [one] Prepayment of { $n } installment · { $contrato }
       *[other] Prepayment of { $n } installments · { $contrato }
    }

## Market interest rates (Banco Central do Brasil, `debt::market_rates`)

nucleo-planejamento-mercado-modalidade-cheque-especial = Overdraft
nucleo-planejamento-mercado-modalidade-cartao-rotativo = Credit card: revolving
nucleo-planejamento-mercado-modalidade-cartao-parcelado = Credit card: installments
nucleo-planejamento-mercado-modalidade-credito-pessoal = Personal loan
nucleo-planejamento-mercado-modalidade-consignado-privado = Payroll loan (private company)
nucleo-planejamento-mercado-modalidade-consignado-publico = Payroll loan (civil servant)
nucleo-planejamento-mercado-modalidade-consignado-inss = INSS payroll loan
nucleo-planejamento-mercado-modalidade-veiculo = Car loan
nucleo-planejamento-mercado-modalidade-outros-bens = Financing of other goods
nucleo-planejamento-mercado-modalidade-imobiliario-pre = Mortgage (fixed rate)
nucleo-planejamento-mercado-modalidade-imobiliario-tr = Mortgage (TR + rate)
nucleo-planejamento-mercado-modalidade-imobiliario-ipca = Mortgage (IPCA + rate)
nucleo-planejamento-mercado-resumo-dentro = { $contrato }: { $taxa } a month, within what the market charges ({ $mediana } a month median across { $bancos } banks).
nucleo-planejamento-mercado-resumo-acima = { $contrato }: { $taxa } a month, above the market median ({ $mediana } a month); { $mais_baratos } of { $bancos } banks charge less{ $melhor }. It's worth simulating portability.
# Goes after "charge less" (starts with the comma).
nucleo-planejamento-mercado-resumo-melhor = , the cheapest ({ $banco }) charges { $taxa } a month
nucleo-planejamento-mercado-fonte = Banco Central do Brasil — interest rates by financial institution (average of each bank's operations; your loan depends on your profile).
nucleo-planejamento-mercado-aviso-fonte-desligada = The Banco Central source is turned off in Settings › Data sources.
nucleo-planejamento-mercado-aviso-falha = Couldn't update the BCB rates ({ $erro }); the last downloaded ones apply.
nucleo-planejamento-mercado-aviso-sem-taxas = No BCB rates downloaded yet.
nucleo-planejamento-mercado-erro-resposta = invalid BCB response: { $erro }
nucleo-planejamento-mercado-erro-sem-periodo = the BCB didn't return the latest period
nucleo-planejamento-mercado-erro-sem-taxas = the BCB didn't return any rates

## Loan statement pasted from the bank (`debt::statement`)

nucleo-planejamento-divida-extrato-linhas-nao-entendidas =
    { $n ->
        [one] { $n } line not understood, for example: "{ $exemplo }"
       *[other] { $n } lines not understood, for example: "{ $exemplo }"
    }
nucleo-planejamento-divida-extrato-erro-vazio = paste the loan statement (or send the rows)
nucleo-planejamento-divida-extrato-erro-nenhuma-linha = no statement line was understood: paste it with the header (date, amortization, interest, balance owed…)
nucleo-planejamento-divida-extrato-aviso-juros = the statement's interest is equivalent to { $mensal }% a month ({ $nominal }% a year nominal, { $efetiva }% a year effective); the loan is set at { $contrato }% a year effective. Adjust the rate so the projection matches the bank
nucleo-planejamento-divida-extrato-aviso-prazo = couldn't calculate the remaining term from the last installment: enter the remaining term
nucleo-planejamento-divida-extrato-erro-prazo = enter the remaining term (how many installments are left after the last line)
nucleo-planejamento-divida-extrato-erro-antes-do-vencimento = the line of { $data } is before the loan's first due date ({ $vencimento }): fix the first due date

## Payoff plan: snowball × avalanche (`debt::payoff`)

nucleo-planejamento-quitacao-contrato = loan { $id }
nucleo-planejamento-quitacao-erro-extra-negativo = the extra per month can't be negative
nucleo-planejamento-quitacao-erro-divida-incompleta = each debt entered needs a name and a positive balance and minimum payment
nucleo-planejamento-quitacao-erro-taxa-negativa = negative rate
nucleo-planejamento-quitacao-erro-sem-dividas = no debt with a balance: add a loan or enter a debt
nucleo-planejamento-quitacao-nota-indexador = { $contrato }: only the contract rate, without the index adjustment.
nucleo-planejamento-quitacao-nota-indexador-valor = { $contrato }: contract rate plus { $indice } at { $valor }% a month (latest value, from { $data }), held constant to the end.
nucleo-planejamento-quitacao-nota-sac = { $contrato }: with SAC the installment goes down over time; here it stays fixed (pays off a little sooner).
nucleo-planejamento-quitacao-nota-minimo-nao-quita = Paying only the minimum, some debt doesn't end within 50 years: the installment doesn't cover the interest.
nucleo-planejamento-quitacao-mais-de-50-anos = more than 50 years
nucleo-planejamento-quitacao-avalanche = avalanche
nucleo-planejamento-quitacao-bola-de-neve = snowball
nucleo-planejamento-quitacao-explicacao-minimo = Paying only the minimum: debt-free in { $quando } with { $juros } in interest.
nucleo-planejamento-quitacao-explicacao-minimo-nunca = Paying only the minimum, some debt never ends: the installment doesn't cover the interest.
nucleo-planejamento-quitacao-explicacao-estrategias = Snowball: { $bola_quando } ({ $bola_juros } in interest). Avalanche: { $avalanche_quando } ({ $avalanche_juros } in interest).
nucleo-planejamento-quitacao-explicacao-iguais = Both give the same result here; the snowball pays off the first debt sooner.
# The strategy comes from the catalog ("avalanche", "snowball").
nucleo-planejamento-quitacao-explicacao-economia = The { $estrategia } method saves { $valor } in interest compared with the minimum.
nucleo-planejamento-quitacao-explicacao-menos-juros = The { $estrategia } method pays { $valor } less in interest.
nucleo-planejamento-cenario-negativo-hoje = The available money is already negative today.
nucleo-planejamento-divida-parcela-n = installment { $n } of this loan
nucleo-planejamento-previdencia-motivo-pagamentos = Pension payment history imported
nucleo-planejamento-previdencia-pagamentos-erro-classe = The payment history is for assets of the Pension class.
nucleo-planejamento-previdencia-pagamentos-erro-vazio = Enter at least one payment (date and amount).
nucleo-planejamento-previdencia-pagamentos-erro-valor = Each payment amount must be greater than zero.
nucleo-planejamento-previdencia-pagamentos-erro-conta = Choose the account that holds the plan's units.
nucleo-planejamento-previdencia-pagamentos-sem-cnpj = No unit price: enter the CNPJ of the plan's fund (the FIE) on the asset, to fetch the prices from the CVM.
nucleo-planejamento-previdencia-pagamentos-sem-cota = { $n ->
        [one] { $n } contribution
       *[other] { $n } contributions
    } still without a CVM unit price, since { $data }: they go into the proposal with the units pending and become unit purchases on their own when the quote collection fetches the fund's history.
nucleo-planejamento-previdencia-pagamentos-nada = Nothing new to save: the payments were already imported.
nucleo-planejamento-previdencia-pagamentos-ilegivel-valor = Screenshot { $print }: the line of { $lido } has an unreadable amount. Check it in the insurer's app and enter the amount.
nucleo-planejamento-previdencia-pagamentos-ilegivel-data = Screenshot { $print }: the payment of { $lido } has no readable date. Check it in the insurer's app and enter the date.
nucleo-planejamento-previdencia-pagamentos-cortada-alto = Screenshot { $print }: the first line is cut (only { $lido }). Send the screenshot above with the whole line, or enter the payment.
nucleo-planejamento-previdencia-pagamentos-cortada-fim = Screenshot { $print }: the last line is cut (only { $lido }). Send the screenshot below with the whole line, or enter the payment.
nucleo-planejamento-previdencia-pagamentos-mes-faltando = No payment in { $lido }, in the middle of a monthly series. Check whether there was a contribution that month (a screenshot may be missing).
nucleo-planejamento-previdencia-aviso-aportes-sem-cota =
    { $n ->
        [one] { $n } contribution still without a CVM unit price counts in the balance at the amount paid, with no return, until the quote collection brings the price of the day
       *[other] { $n } contributions still without a CVM unit price count in the balance at the amount paid, with no return, until the quote collection brings the price of the day
    }
nucleo-planejamento-previdencia-pagamentos-vao = Screenshots { $print }: the day { $lido } has fewer contributions than the other days of the series; a payment may have been left between the two screenshots. Send a screenshot showing that part, or enter the payment.
nucleo-planejamento-previdencia-pagamentos-conta-nova = Units · { $plano }
nucleo-planejamento-previdencia-pagamentos-conta-nota = Account holding the units of the plan { $plano }, created when the payment history was imported (no cash movement).
nucleo-planejamento-previdencia-motivo-pagamentos-n = { $plano }: { $com } contribution(s) with a unit price, { $sem } waiting for the CVM unit price and { $pendencias } item(s) to check
nucleo-planejamento-previdencia-motivo-cotas-completadas = CVM unit prices filled in on { $n } pension contribution(s)
nucleo-previdencia-saldo-so-previdencia = the reported balance is only for pension plans
nucleo-previdencia-saldo-positivo = the balance must be greater than zero
nucleo-previdencia-saldo-sem-cotas = the plan has no units on { $data }: record the contributions before entering the balance

## Installments detected from the description
nucleo-parcelas-ref-serie = installment series
nucleo-parcelas-historico-ignorar = Not an installment: { $descricao }
nucleo-parcelas-historico-voltar = Installment series restored
nucleo-parcelas-previsao = { $descricao } · detected installment
