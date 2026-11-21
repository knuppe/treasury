# Financial health (core, `health`): the score models, bands, dimensions, indicators (target,
# explanation, calculation and suggestions), summary and notes (docs/health.md). Built on the
# fly, in the installation language.
# Revision: AI draft, not yet reviewed by a native speaker.

## Models (Settings › Financial health)

nucleo-saude-modelo-isf = I-SFB (Febraban and Banco Central)
nucleo-saude-modelo-isf-descricao = Inspired by the Brazilian Financial Health Index: a score from 0 to 100 across four dimensions (behavior, skill, freedom and security), compared with the national average of 56.7 in 2024. An estimate from your data, not the official questionnaire.
nucleo-saude-modelo-classicos = Classic indicators
nucleo-saude-modelo-classicos-descricao = The usual personal finance ratios, each with a target: save 20% of income, an emergency fund of 6 months of essential spending, installments up to 15% of income, housing up to 30%, no revolving credit and a growing net worth.
nucleo-saude-modelo-desativado = Off
nucleo-saude-modelo-desativado-descricao = No financial health analysis: the card disappears from the dashboard and the reports section stays empty.
nucleo-saude-desativada = Financial health analysis is turned off in Settings.
nucleo-saude-erro-mes-futuro = financial health can only be assessed up to the current month
nucleo-saude-erro-so-interface = the financial health model can only be changed in the app

## Measured values and periods

nucleo-saude-valor-mes = { $n } month
nucleo-saude-valor-meses = { $n } months
nucleo-saude-valor-de-meses = { $n } of { $total } months
nucleo-saude-valor-em-dia = { $n } of { $total } on time
nucleo-saude-nenhum-mes = no months
nucleo-saude-n-meses = { $n ->
        [one] 1 month
       *[other] { $n } months
    }
nucleo-saude-sem-meses = no months
nucleo-saude-base-competencia = accrual
nucleo-saude-base-caixa = cash

## Amounts used in the calculation

nucleo-saude-quantia-receitas = Income
nucleo-saude-quantia-despesas = Expenses
nucleo-saude-quantia-sobra = Left over
nucleo-saude-quantia-reserva = Emergency fund
nucleo-saude-quantia-gasto-essencial = Essential spending per month
nucleo-saude-quantia-parcelas = Installments
nucleo-saude-quantia-moradia = Housing
nucleo-saude-quantia-patrimonio-em = Net worth in { $mes }
nucleo-saude-quantia-variacao = Change

## No data (the reason)

nucleo-saude-sem-dados-meses = No data: at least { $minimo } full months of transactions are needed ({ $n } analyzed).
nucleo-saude-sem-dados-receita = No data: no income recorded in { $periodo }.
nucleo-saude-sem-dados-gastos = No data: at least { $minimo } full months of spending are needed for the average ({ $n } analyzed).
nucleo-saude-sem-dados-essenciais = No data: no expenses in essential categories in recent months. Mark the essential categories in Settings › Categories.
nucleo-saude-sem-dados-moradia = No data: there is no "Housing" expense category and no mortgage.
nucleo-saude-sem-dados-credito = No data: no checking account and no card with a recorded bill payment.
nucleo-saude-sem-dados-patrimonio = No data: at least { $minimo } months of net worth are needed ({ $n } found).
nucleo-saude-sem-dados-parcelas = No data: no loan installment or card bill with a recorded payment was due in the period.
nucleo-saude-sem-dados-orcamento = No data: no month in { $periodo } has an expense budget.
nucleo-saude-sem-dados-movimento = No data: no activity in the period.

## How it was calculated

nucleo-saude-conta-poupanca = (income − expenses) ÷ income for { $periodo } ({ $n } months, { $base } basis); transfers, card bill payments and contributions don't count.
nucleo-saude-conta-regular = Months with a savings rate of 10% or more ÷ months with income, in { $periodo }.
nucleo-saude-conta-reserva = Emergency fund ({ $fonte }) ÷ average monthly expenses in essential categories over the last { $n } full months.
nucleo-saude-fonte-meta = the current amount of the emergency fund goal (the accounts and positions linked to it)
nucleo-saude-fonte-saldos = the balance of checking, savings and cash accounts
nucleo-saude-conta-comprometimento = Installments due in { $periodo } ÷ income for the period; { $contratos }. Card installment purchases count as expenses, not here.
nucleo-saude-contratos-nenhum = no loan or financing with an installment in the period
nucleo-saude-contratos = installments of { $n ->
        [one] { $n } contract
       *[other] { $n } contracts
    } (loans and financing)
nucleo-saude-conta-moradia = Expenses in Housing (rent, condo fees, household bills…) and mortgage installments recorded outside it ÷ income for { $periodo }.
nucleo-saude-credito-cheque = overdraft in { $meses }
nucleo-saude-credito-rotativo = card bill not paid in full in { $meses }
nucleo-saude-credito-nenhum = no use
nucleo-saude-conta-credito = Months in { $periodo } with a negative checking balance (overdraft) or a card bill not paid in full within { $dias } days of the due date (revolving credit): { $detalhe }.
nucleo-saude-conta-credito-sem-cartoes = Cards with no recorded bill payment are left out.
nucleo-saude-base-patrimonio-inicial = the starting net worth
nucleo-saude-base-despesas = the period's expenses (the starting net worth is small)
nucleo-saude-conta-patrimonio = Change in net worth (accounts, portfolio, assets and debts) from { $de } to { $ate } ÷ { $base }. Nominal value, not adjusted for inflation.
nucleo-saude-conta-em-dia = Loan installments ({ $parcelas_em_dia } on time, { $parcelas_atraso } late) and card bills ({ $faturas_pagas } paid in full, { $faturas_nao } not) due in { $periodo }, with { $dias } days of grace. An installment not reconciled after the due date counts as late.
nucleo-saude-conta-orcamento = Months with expenses within the planned total ÷ months with a budget in { $periodo } ({ $base } basis).
nucleo-saude-conta-azul = Months in which income covered expenses ÷ months with activity, in { $periodo } ({ $base } basis).

## Indicators: name, target, what it measures, suggestions and what to do without data

nucleo-saude-sem-dados-3-meses = Record or import income and expenses for at least 3 full months.

nucleo-saude-poupanca = Savings rate
nucleo-saude-poupanca-meta = ≥ 20% of income
nucleo-saude-poupanca-explicacao = How much of your income is left after expenses, over the last 12 months.
nucleo-saude-poupanca-dica-1 = Pay yourself first: schedule a transfer to savings or to your broker on payday.
nucleo-saude-poupanca-dica-2 = Review your three biggest spending categories in Reports › Composition and set limits in the budget.
nucleo-saude-poupanca-dica-3 = Cancel subscriptions and fixed bills you don't use (Reports › Recurring).

nucleo-saude-regular = Regular saving
nucleo-saude-regular-meta = 10% or more every month
nucleo-saude-regular-explicacao = In how many months you managed to save at least 10% of your income.
nucleo-saude-regular-dica-1 = Automate a fixed savings amount every month, even a small one.
nucleo-saude-regular-dica-2 = Set money aside during the year for seasonal expenses (IPVA, IPTU, school supplies) so they don't wipe out what's left over.
nucleo-saude-regular-dica-3 = In weaker months, check Reports › Comparison to see what changed compared with a good month.

nucleo-saude-reserva = Emergency fund
nucleo-saude-reserva-meta = ≥ 6 months of essential spending
nucleo-saude-reserva-explicacao = For how many months the emergency fund covers essential spending with no income at all.
nucleo-saude-reserva-dica-1 = Set (or raise) the emergency fund goal in 05 · Budget › Goals, aiming for 6 months of essential spending.
# The part in parentheses is the same as `nucleo-pais-reserva-br` (swapped for the country's products).
nucleo-saude-reserva-dica-2 = Keep the emergency fund in a low-risk investment with daily liquidity (Tesouro Selic or a CDB with daily liquidity).
nucleo-saude-reserva-dica-3 = Put extras such as the 13º, the income tax refund and profit sharing (PLR) into the emergency fund until you reach the target.
nucleo-saude-reserva-sem-dados = Mark the essential categories in Settings › Categories and record at least 3 months of spending.

nucleo-saude-comprometimento = Income committed to installments
nucleo-saude-comprometimento-meta = ≤ 15% of income
nucleo-saude-comprometimento-explicacao = Share of income that goes to loan and financing installments.
nucleo-saude-comprometimento-dica-1 = Avoid new installments until they take up less than 15% of your income.
nucleo-saude-comprometimento-dica-2 = Simulate moving the most expensive contract to another bank or paying extra on it in 07 · Debts.
nucleo-saude-comprometimento-dica-3 = Pay off the debts with the highest interest first (credit card, overdraft, personal loans).
nucleo-saude-comprometimento-sem-dados = Record income for at least 3 full months and add the contracts in 07 · Debts.

nucleo-saude-moradia = Housing cost
nucleo-saude-moradia-meta = ≤ 30% of income
nucleo-saude-moradia-explicacao = Share of income that goes to housing: rent or mortgage, condo fees and household bills.
nucleo-saude-moradia-dica-1 = Housing above 30% of income squeezes the rest of the budget: review utility bills, condo fees and insurance.
nucleo-saude-moradia-dica-2 = When renewing the lease or taking out a new mortgage, aim for a cost of up to 30% of income.
# The part "paying it down with the FGTS…" is the same as `nucleo-pais-fgts-br`.
nucleo-saude-moradia-dica-3 = If the mortgage weighs on you, simulate paying it down with the FGTS or moving it to a lower rate in 07 · Debts.
nucleo-saude-moradia-sem-dados = Record household spending in the Housing category (or add the mortgage in 07 · Debts).

nucleo-saude-credito = Revolving credit and overdraft
nucleo-saude-credito-meta = no months
nucleo-saude-credito-explicacao = Months in which expensive credit was used: overdraft or the card's revolving credit.
nucleo-saude-credito-dica-1 = Pay the card bill in full: revolving credit charges the highest interest on the market.
# The part in parentheses is the same as `nucleo-pais-consignado-br`.
nucleo-saude-credito-dica-2 = If you run short, swap revolving credit or the overdraft for cheaper credit (a payroll-deductible loan, for example).
nucleo-saude-credito-dica-3 = Add the overdraft limit and rates to the account, in 03 · Accounts, to see the estimated cost.
nucleo-saude-credito-sem-dados = Record the card bill payments and the checking account transactions.

nucleo-saude-patrimonio = Net worth growth
nucleo-saude-patrimonio-meta = grow ≥ 5% in 12 months
nucleo-saude-patrimonio-explicacao = How much your net worth grew (or fell) over the last 12 months.
nucleo-saude-patrimonio-dica-1 = Put part of what's left over each month into investments or into paying down debt.
nucleo-saude-patrimonio-dica-2 = Update the value of your assets and the portfolio prices so your net worth reflects reality.
nucleo-saude-patrimonio-dica-3 = Review the year's big expenses in Reports › Trend: they explain the drop.
nucleo-saude-patrimonio-sem-dados = Add your accounts with their opening balance and keep a few months of transactions.

nucleo-saude-em-dia = Bills on time
nucleo-saude-em-dia-meta = all paid by the due date
nucleo-saude-em-dia-explicacao = Loan installments and card bills paid in full by the due date.
nucleo-saude-em-dia-dica-1 = Turn on automatic debit for the installments and the card bill.
nucleo-saude-em-dia-dica-2 = Reconcile the installments in 07 · Debts: paid but unreconciled ones count as late.
nucleo-saude-em-dia-dica-3 = Use Upcoming due dates on the dashboard to plan ahead.
nucleo-saude-em-dia-sem-dados = Add the contracts in 07 · Debts and record the card bill payments.

nucleo-saude-orcamento = Spending within budget
nucleo-saude-meta-todos-os-meses = every month
nucleo-saude-orcamento-explicacao = In how many months expenses stayed within the budget's planned total.
nucleo-saude-orcamento-dica-1 = Review the budget for the months you went over and adjust the limits to reality.
nucleo-saude-orcamento-dica-2 = Follow the spending pace on the dashboard's budget card throughout the month.
nucleo-saude-orcamento-dica-3 = Keep a budget line for unexpected expenses.
nucleo-saude-orcamento-sem-dados = Set up a monthly budget in 05 · Budget (copy last month's or use the suggestion).

nucleo-saude-azul = Months in the black
nucleo-saude-azul-explicacao = In how many months income covered expenses.
nucleo-saude-azul-dica-1 = For months in the red, check Reports › Comparison to see what changed compared with a month in the black.
nucleo-saude-azul-dica-2 = Keep a cushion for irregular expenses so a bad month doesn't turn into debt.
nucleo-saude-azul-dica-3 = Plan yearly expenses (insurance, taxes, trips) as monthly installments in the budget.

## I-SFB: bands and dimensions

nucleo-saude-faixa-ruim = Poor
nucleo-saude-faixa-muito-baixa = Very low
nucleo-saude-faixa-baixa = Low
nucleo-saude-faixa-ok = Ok
nucleo-saude-faixa-boa = Good
nucleo-saude-faixa-muito-boa = Very good
nucleo-saude-faixa-otima = Excellent

nucleo-saude-dimensao-comportamento = Behavior
nucleo-saude-dimensao-comportamento-descricao = Paying bills on time, spending within the budget and ending months in the black.
nucleo-saude-dimensao-habilidade = Skill
nucleo-saude-dimensao-habilidade-descricao = Being able to save part of your income, regularly.
nucleo-saude-dimensao-liberdade = Freedom
nucleo-saude-dimensao-liberdade-descricao = How much of your income is free from installments and expensive credit (revolving credit and overdraft).
nucleo-saude-dimensao-seguranca = Security
nucleo-saude-dimensao-seguranca-descricao = A cushion for emergencies and a growing net worth.

## Classic indicators: bands

nucleo-saude-faixa-critica = Critical
nucleo-saude-faixa-fragil = Fragile
nucleo-saude-faixa-estavel = Stable
nucleo-saude-faixa-saudavel = Healthy

## Summary and notes

nucleo-saude-resumo-acima = Score { $nota } ({ $faixa }), above the national average of { $media }.
nucleo-saude-resumo-abaixo = Score { $nota } ({ $faixa }), below the national average of { $media }.
nucleo-saude-resumo-igual = Score { $nota } ({ $faixa }), equal to the national average of { $media }.
nucleo-saude-resumo-dimensao-fraca = Weakest dimension: { $dimensao }.
nucleo-saude-resumo-nota = Score { $nota } ({ $faixa }).
nucleo-saude-resumo-ponto-fraco = Weakest point: { $indicador }.
nucleo-saude-resumo-tudo-na-meta = All indicators on target.
nucleo-saude-insuficiente-meses = Not enough data: at least { $minimo } full months of transactions are needed (there are { $n }).
nucleo-saude-insuficiente-indicadores = Not enough data: too few indicators could be measured.
nucleo-saude-aviso-isf = An estimate from your data (transactions, installments, card bills and balances), in the spirit of the I-SFB by Febraban and Banco Central; the official index comes from a questionnaire.
nucleo-saude-aviso-classicos = Classic personal finance indicators calculated from your data; the usual personal finance targets.
nucleo-saude-aviso-pouco-historico = With less than { $minimo } months of history, indicators show no data instead of pulling the score down.
nucleo-saude-aviso-sem-cambio = No exchange rate for { $moedas }: those amounts were left out of the calculation.
nucleo-saude-aviso-contas-desatualizadas = Estimated score: { $contas } without recent transactions. The balance may be incomplete, and a negative balance there is left to check instead of counting as overdraft.
nucleo-saude-referencia = I-SFB national average in 2024
nucleo-saude-referencia-curta = National average
nucleo-saude-credito-suspeito = possible overdraft or incomplete import in { $contas } (30 days or more below zero without the limit set up): set up the account's overdraft in 03 · Accounts or import the full statement
