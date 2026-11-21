# Reports (core, `reports`): labels of the groups and of the money flow, the formulas and sources
# of "Explain" (`explain_value`), upcoming due dates and errors. Built on the fly, in the
# installation language. Errors in lowercase, like a log line (the interface capitalizes them).
# Revision: AI draft, not yet reviewed by a native speaker.

## Periods and date basis

nucleo-relatorios-erro-mes-inicial-posterior = the start month ({ $de }) is after the end month ({ $ate })
nucleo-relatorios-erro-periodo-longo = period too long (up to { $maximo } months)
nucleo-relatorios-erro-data-inicial-posterior = the start date ({ $de }) is after the end date ({ $ate })
nucleo-relatorios-erro-calendario-longo = period too long for the calendar (up to 10 years)
nucleo-relatorios-base-competencia = accrual
nucleo-relatorios-base-caixa = cash

## Groups and due dates

nucleo-relatorios-sem-categoria = Uncategorized
nucleo-relatorios-sem-membro = No member
nucleo-relatorios-outras = Others
nucleo-relatorios-sem-descricao = (no description)
# Upcoming due dates: a card bill ("Nubank bill (Oct 2026)").
nucleo-relatorios-fatura = { $cartao } bill ({ $mes })

## Where the money went (flow diagram and the items of each node)

nucleo-relatorios-fluxo-orcamento = Budget
nucleo-relatorios-fluxo-receitas-sem-categoria = Uncategorized income
nucleo-relatorios-fluxo-resgates = Withdrawals
nucleo-relatorios-fluxo-deficit = Deficit
nucleo-relatorios-fluxo-dividas = Debt payments
nucleo-relatorios-fluxo-investimentos = Investments
nucleo-relatorios-fluxo-sobra = Surplus
nucleo-relatorios-fluxo-sem-subcategoria = { $categoria } (no subcategory)
nucleo-relatorios-fluxo-receita-de = Income from { $categoria }
nucleo-relatorios-fluxo-pagamento-de = The payment of { $contrato }
nucleo-relatorios-fluxo-entradas = In
nucleo-relatorios-fluxo-saidas = Out
nucleo-relatorios-fluxo-sem-cambio = No exchange rate from { $de } to { $para } on the date: left out of the total.
nucleo-relatorios-fluxo-grupo-negativo = { $grupo } added up to { $valor } in the period (refunds larger than the transactions); the diagram leaves out groups without a positive balance.
nucleo-relatorios-fluxo-sem-sobra-deficit = There was no surplus in the period: what went out exceeded what came in by { $valor }.
nucleo-relatorios-fluxo-sem-sobra = There was no surplus in the period.
nucleo-relatorios-fluxo-sem-deficit-sobra = There was no deficit in the period: { $valor } was left over.
nucleo-relatorios-fluxo-sem-deficit = There was no deficit in the period.
nucleo-relatorios-erro-no-desconhecido = unknown flow node: { $no }
nucleo-relatorios-erro-sem-nos = provide at least one flow node
nucleo-relatorios-erro-nos-sobrepostos = the requested nodes overlap (a category and one of its subcategories)

## "Explain" (`explain_value`): sources of each number

nucleo-relatorios-conta = account { $id }
nucleo-relatorios-fonte-cambio = exchange rate: 1 { $moeda } = { $taxa } { $alvo }
nucleo-relatorios-fonte-cambio-estimado = exchange rate: 1 { $moeda } = { $taxa } { $alvo } (estimated)
nucleo-relatorios-fonte-saldo-inicial = opening balance of { $conta }
# An account (or transaction) in another currency: "US account: USD 100.00 × 5.43 = BRL 543.00".
nucleo-relatorios-fonte-conta-convertida = { $conta }: { $valor } × { $taxa } = { $convertido }
nucleo-relatorios-fonte-conta-sem-cambio = { $conta }: { $valor } (no exchange rate, left out of the total)
nucleo-relatorios-fonte-convertida = { $rotulo } × { $taxa } = { $convertido }
nucleo-relatorios-fonte-carteira = Investment portfolio (market value or book value; see the portfolio)
nucleo-relatorios-fonte-estimativa = estimate
nucleo-relatorios-fonte-avaliacao = appraisal
nucleo-relatorios-fonte-dividas = Loans and financing (outstanding balance from the schedule)
nucleo-relatorios-fonte-partes =
    { $n ->
        [one] { $rotulo } (part of the split)
       *[other] { $rotulo } ({ $n } parts of the split)
    }
nucleo-relatorios-fonte-fatura = { $rotulo } (bought on { $compra }, bill due on { $vencimento })
nucleo-relatorios-fonte-lancamento-sem-cambio = { $rotulo }: { $valor } without an exchange rate, left out of the total

## "Explain": formulas

nucleo-relatorios-n-efetivados =
    { $n ->
        [one] { $n } cleared transaction
       *[other] { $n } cleared transactions
    }
nucleo-relatorios-n-despesas =
    { $n ->
        [one] { $n } cleared expense transaction
       *[other] { $n } cleared expense transactions
    }
nucleo-relatorios-n-receitas =
    { $n ->
        [one] { $n } cleared income transaction
       *[other] { $n } cleared income transactions
    }
# Goes at the end of a formula (starts with the semicolon).
nucleo-relatorios-formula-sem-cambio = ; no exchange rate for { $moedas } (left out of the total)
nucleo-relatorios-formula-saldo = opening balance of { $inicial } + { $lancamentos } up to { $data } ({ $soma }) = { $valor }
nucleo-relatorios-formula-saldo-abertura = { $lancamentos } up to { $data } = { $valor } (the account was opened on { $abertura }; the opening balance of { $inicial } only counts from that date)
nucleo-relatorios-formula-patrimonio =
    assets of { $ativos } − liabilities of { $passivos } = { $valor }, adding up the balance of { $n ->
        [one] { $n } account
       *[other] { $n } accounts
    } on { $data } (opening balance + cleared transactions up to the date; negative balances, like card debt, are liabilities), the portfolio, physical assets (appraisal or estimate) and debts{ $sem_cambio }
# Filters of the spending and income formula, after the period ("… (accrual) in Housing and
# subcategories for Ana: …").
nucleo-relatorios-filtro-categoria = in { $categoria } and subcategories
nucleo-relatorios-filtro-membro = for { $membro }
nucleo-relatorios-regra-despesa = expense = −sum of the amounts (refunds reduce the total)
nucleo-relatorios-regra-receita = income = sum of the amounts
nucleo-relatorios-formula-convertidos = ; amounts in another currency converted to { $moeda } at the exchange rate of the transaction date
nucleo-relatorios-formula-fluxo = { $lancamentos } from { $de } to { $ate } ({ $base }){ $filtros }: { $regra } = { $valor }; transfers, card bill payments and investment contributions are not included; split transactions count by their parts{ $convertidos }{ $sem_cambio }
nucleo-relatorios-formula-poupanca = savings rate for { $mes } ({ $base }) = (income of { $receitas } − expenses of { $despesas }) ÷ income of { $receitas } = { $taxa }; amount = income − expenses = { $valor }, from { $lancamentos }{ $sem_cambio }
nucleo-relatorios-formula-poupanca-sem-receitas = no income in { $mes }, so the savings rate doesn't apply; income − expenses = { $valor }, from { $lancamentos }{ $sem_cambio }

## Where the income goes (spending groups of the categories; group names in grupos.ftl)

# What goes into Future without a category: investment contributions and transfers to the
# goals' accounts.
nucleo-relatorios-grupos-aportes = Investment contributions
nucleo-relatorios-grupos-metas = Transfers to goals
# Section of the monthly PDF report.
nucleo-relatorios-grupos-pdf-titulo = Where the income went
nucleo-relatorios-grupos-pdf-dica = The month's expenses by each category's group; future includes contributions and goals.
nucleo-relatorios-grupos-pdf-parte = { $parte } of income
nucleo-relatorios-grupos-pdf-sem-renda = no income this month
nucleo-relatorios-grupos-pdf-meta = target { $meta }
nucleo-relatorios-grupos-pdf-sem-meta = no target
