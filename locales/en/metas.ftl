# Budget › Goals, Emergency fund and Retirement: goals with progress and projection, the
# emergency fund calculator, the retirement planner (with INSS by the rules) and scenarios
# (docs/i18n-design.md). Simulation notes and rules come from the core.
# Revision: AI draft, not yet reviewed by a native speaker.

## Goal kinds and status

metas-tipo-emergencia = Emergency fund
metas-tipo-viagem = Travel
metas-tipo-quitacao = Debt payoff
metas-tipo-compra = Purchase
metas-tipo-aposentadoria = Retirement
metas-tipo-educacao = Education
metas-tipo-outra = Other

metas-situacao-atingida = Reached
metas-situacao-no-caminho = On track
metas-situacao-fora-ritmo = Off pace
metas-situacao-sem-aportes = No contributions
metas-situacao-sem-data = No target date

## Form

metas-erro-nome = Enter the goal name.
metas-erro-valor = Enter a target amount greater than zero.
metas-erro-data-passou = The target date has already passed.
metas-erro-gravar = Could not save.
metas-atualizada = Goal updated
metas-criada = Goal created
metas-editar-titulo = Edit goal · { $nome }
metas-nova = New goal
metas-criar = Create goal
metas-nome = Name
metas-tipo = Type
metas-valor-alvo = Target amount
metas-data-alvo = Target date
metas-data-alvo-dica = Optional; with no date, there is no required contribution.
metas-ja-guardado = Already saved outside the accounts
metas-aporte-planejado = Planned monthly contribution
metas-aporte-planejado-dica = Used in the projection when the linked accounts have no pace.
metas-contas-vinculadas = Linked accounts
metas-contas-vinculadas-dica = Today's balance of these accounts counts toward the current amount; the average net contribution over the last 6 months is the pace.
metas-notas = Notes

## Goal card

metas-contas = Accounts
metas-nenhuma-conta = none
metas-de-alvo = of { $valor }
metas-progresso-de = Progress of { $nome }
metas-falta = Remaining
metas-aporte-necessario = Required contribution
metas-por-mes = /month
metas-ritmo = Pace
metas-ritmo-planejado = Pace (planned)
metas-ritmo-real = Pace (6 months)
metas-projecao = Projection
metas-projecao-planejada = { $data } (at the planned contribution)
metas-sem-ritmo = no contribution pace to project
metas-sem-cambio = No exchange rate for { $moedas }: balance left out of the current amount.
metas-editar-nome = Edit { $nome }
metas-reativar-nome = Reactivate { $nome }
metas-arquivar-nome = Archive { $nome }
metas-excluir-nome = Delete { $nome }
metas-reativar = Reactivate
metas-arquivar = Archive
metas-excluir = Delete
metas-nao-arquivou = Could not archive
metas-excluir-titulo = Delete goal
metas-excluir-descricao = Delete “{ $nome }”? The linked accounts don't change. You can undo it from the change history.
metas-excluida = Goal deleted
metas-nao-excluiu = Could not delete

## List

metas-mostrar-arquivadas = Show archived
metas-vazio-kicker = No goals
metas-vazio-titulo = No goals yet
metas-vazio-descricao = Create a goal (emergency fund, trip, paying off a debt) and link the accounts where the money is kept: progress and projection come from the real balances.

## Emergency fund

metas-reserva-base = Basis
metas-reserva-caixa = cash
metas-reserva-competencia = accrual
metas-reserva-meses-analisados = Months analyzed
metas-reserva-meses-intervalo = { $de } to { $ate }
metas-reserva-fonte = Source
metas-reserva-fonte-valor = Categories marked as essential
metas-reserva-meses-cobertura = Months of coverage
metas-reserva-media-de = Average of
metas-reserva-meses-na-media = Months in the average
metas-reserva-n-meses = { $n } months
metas-reserva-meta = Fund target
metas-reserva-meta-dica = { $n } × essential spending
metas-reserva-guardado = Saved
metas-reserva-guardado-dica = checking accounts, savings and cash
metas-reserva-cobertura = Coverage
metas-reserva-cobertura-valor =
    { $n ->
        [one] { $valor } month
       *[other] { $valor } months
    }
metas-reserva-cobertura-dica = of essential spending
metas-reserva-progresso = Fund saved compared to the target
metas-reserva-gasto-essencial = Average essential spending per month
metas-reserva-gasto-total = Average total spending per month
metas-reserva-essenciais-total = Essentials in the total
metas-reserva-guardado-meta = Saved / target
metas-reserva-sem-meses = There are no complete months with spending in essential categories to work out the average yet.
metas-reserva-criar = Create emergency fund goal
metas-reserva-sem-meta = Without a goal, the saved amount is the balance of your everyday accounts.
metas-reserva-atualizada = Emergency fund goal updated
metas-reserva-atualizar = Update the goal to { $valor }
metas-reserva-meta-em = The goal “{ $nome }” is set to { $valor }.
metas-reserva-ja-usa = ✓ The goal “{ $nome }” already uses this amount.

## INSS by the reform rules

metas-inss-sexo = Sex
metas-inss-homem = Male
metas-inss-mulher = Female
metas-inss-nascimento = Date of birth
metas-inss-nascimento-dica = Empty: today's age.
metas-inss-contribuicao = Contributions to date (years)
metas-inss-contribuicao-reforma = Contributions on 11/13/2019
metas-inss-contribuicao-reforma-dica = Empty: estimated with no gaps.
metas-inss-media = Average salary since 07/1994
metas-inss-media-dica = Contribution salaries, adjusted.
metas-inss-contribui-ate = Contributes until age
metas-inss-regras = Rules: { $sistema }
metas-inss-regra = Rule
metas-inss-cumpre-em = Met on
metas-inss-beneficio-data = Benefit on that date
metas-inss-na-idade = At the plan's age
metas-inss-usada = used
metas-inss-cumpre-data = { $data } (age { $idade })
metas-inss-nao-cumpre = not met
metas-inss-fator = social security factor
metas-inss-aos = { $valor } at { $idade }
metas-ssa-pia = Benefit at full retirement age (PIA)
metas-ssa-pia-dica = The one on your statement at ssa.gov/myaccount.
metas-ssa-aime = AIME (without the PIA)
metas-ssa-aime-dica = Monthly average of the 35 best indexed years.
metas-ssa-creditos = Credits so far
metas-ssa-creditos-dica = Empty: 40 (already eligible).
metas-ssa-trabalha-ate = Works until age
metas-pt-carreira = Years of contributions so far
metas-pt-rr = Reference earnings
metas-pt-rr-dica = Career earnings ÷ (14 × years).
metas-pt-antes-17 = Started contributing before 17

## Retirement planner

metas-apos-titulo = Retirement and financial independence
metas-apos-intro = Thousands of market scenarios in real terms (above inflation): today's wealth, contributions until you stop, the monthly spending you want, the public pension ({ $sistema }) and life events.
metas-apos-idade-hoje = Age today
metas-apos-aposentadoria-aos = Retire at
metas-apos-expectativa = Life expectancy
metas-apos-patrimonio-hoje = Invested wealth today
metas-apos-patrimonio-dica-valor = Empty: the portfolio ({ $valor }).
metas-apos-patrimonio-dica = Empty: the portfolio, including pension plans.
metas-apos-aporte = Monthly contribution until you stop
metas-apos-gasto = Desired monthly spending
metas-apos-inss-informar = Enter the amount
metas-apos-inss-calcular = Calculate by the rules
metas-apos-inss-mensal = Estimated monthly amount ({ $sistema })
metas-apos-inss-a-partir = { $sistema } from age
metas-apos-retorno = Real return (% a year)
metas-apos-volatilidade = Volatility (% a year)
metas-apos-eventos = Life events
metas-apos-idade = Age
metas-apos-evento-valor = Amount (− if it goes out)
metas-apos-evento-o-que = What it is
metas-apos-remover-evento = Remove event { $n }
metas-apos-adicionar-evento = Add event (inheritance, selling a home, college…)
metas-apos-informe-idade = Enter your age today to run the simulation.
metas-apos-simulando = Simulating the scenarios
metas-apos-chance = Chance the money lasts
metas-apos-patrimonio-mediana = Wealth at retirement (median)
metas-apos-gasto-90 = Monthly spending with a 90% chance
metas-apos-necessario = Needed when you stop (90% chance)
metas-apos-acaba-aos = In the bad scenario, the money runs out at
metas-apos-n-anos = { $n } years
metas-apos-nao-acaba = doesn't run out
metas-apos-patrimonio-idade = Wealth by age
metas-apos-ruim = Bad scenario (10%)
metas-apos-mediana = Median
metas-apos-bom = Good scenario (90%)
metas-apos-salvo = Retirement plan saved
metas-apos-usar-principal = Use as main plan
metas-apos-salvar = Save plan

## Scenarios and the goal linked to the plan

metas-cen-salvo = Scenario “{ $nome }” saved
metas-cen-cenario = Scenario
metas-cen-principal = Main plan
metas-cen-atualizado = Scenario updated
metas-cen-atualizar = Update “{ $nome }”
metas-cen-apagado = Scenario deleted
metas-cen-apagar = Delete scenario
metas-cen-novo = New scenario
metas-cen-exemplo = E.g.: retire at 55
metas-cen-salvar = Save as scenario
metas-cen-comparar = Compare scenarios
metas-cen-chance = Chance
metas-cen-gasto-90 = Spending at 90%
metas-cen-necessario = Needed when you stop
metas-cen-previsto = Expected when you stop
metas-cen-na-tela = On screen now
metas-ligada = Linked goal: { $nome } · { $pct }% of { $valor }
metas-ligada-ate = Linked goal: { $nome } · { $pct }% of { $valor } by { $ano }
metas-ligada-desligar = Unlink from goal
metas-ligada-explica = A retirement goal linked to the plan tracks the wealth needed, the date and the contribution of the saved plan.
metas-ligada-toast = Goal linked to the plan
metas-ligada-toast-descricao = “{ $nome }” tracks the saved plan in Budget › Goals.
metas-ligada-criar = Create goal linked to the plan

## Metas com rendimento e gastos anuais

metas-rendimento = Real return a year
metas-rendimento-dica = How much the saved money earns above inflation, as your own assumption. Empty: the goal just divides what is left by the months.
metas-erro-retorno = Use a return between −5% and 15% a year.
metas-rendimento-linha = With { $taxa } a year above inflation: { $aportes } from contributions and { $rendimento } from returns. With no return, it would be { $linear }/month.
metas-rendimento-alvo-epoca = In today's money. On the date, with inflation of { $inflacao } a year, the target is worth about { $valor }.
metas-rendimento-premissa = The return is an assumption, not a promise.
metas-reserva-anuais = Include yearly expenses
metas-reserva-anuais-dica = Car and property taxes, insurance and school supplies count divided by 12, instead of the peak months.
metas-reserva-custo-mensal = Monthly cost of living
metas-reserva-essenciais-regulares = Monthly essentials, without the yearly ones
metas-reserva-provisao-anual = Provision for essential yearly expenses
metas-reserva-anuais-titulo = Yearly expenses in the last 12 months
metas-reserva-anuais-tabela = Yearly expenses
metas-reserva-anuais-mes = { $valor }/month
metas-reserva-anuais-nao-essencial = not in the reserve
metas-reserva-anuais-sem-ano = Yearly expenses show up once there are 12 complete months of transactions.
metas-reserva-anuais-nenhum = No yearly expenses in the last 12 months (categories with spending in at most 3 months of the year).
metas-reserva-meta-dica-custo = { $n } × monthly cost of living

## Goal sources (docs/goal-sources.md)

metas-fontes = Sources
metas-fontes-nome = Sources of { $nome }
metas-fontes-titulo = Sources of { $nome }
metas-fontes-dica = The current amount is the sum of the sources. An account counts by its balance, never below zero (an overdrawn account is debt, not savings); an investment counts at its net redemption value when there is one, otherwise at market value; a label brings in every position with it, also those labeled later.
metas-fontes-vazio = No sources yet: link an account, a label or the positions below.
metas-fontes-desatualizadas = The amount may be outdated: { $fontes }.
metas-fonte-tipo-inicial = Saved outside
metas-fonte-tipo-carteira = Portfolio
metas-fonte-tipo-conta = Account
metas-fonte-tipo-conta-inteira = Whole account
metas-fonte-tipo-ativo = Investment
metas-fonte-tipo-etiqueta = Label
metas-fonte-tipo-lancamentos = Transactions
metas-fonte-valor-saldo = balance
metas-fonte-valor-mercado = market value
metas-fonte-valor-liquido = net redemption
metas-fonte-valor-custo = cost (no price)
metas-fonte-valor-informado = amount entered
metas-fonte-remover = Remove { $nome }
metas-fonte-erro = Couldn't change the sources
metas-fonte-negativo-fora = The negative balance was left out: it is debt, not savings.
metas-fonte-desatualizada = May be outdated (no recent statement on the account or no price for the asset).
metas-fonte-ligadas = { $n ->
    [one] 1 investment linked to the goal
   *[other] { $n } investments linked to the goal
}
metas-fonte-liquidez = Use my daily liquidity investments
metas-fonte-liquidez-dica = Fixed income with daily liquidity, Tesouro Selic and DI or referenced funds with an open position that are not in this goal yet.
metas-fonte-liquidez-buscando = Looking for the investments…
metas-fonte-liquidez-vazio = No daily liquidity investment outside this goal.
metas-fonte-adicionar = Add
metas-fonte-adicionar-todos = Add all ({ $n })
metas-fonte-conta = Account
metas-fonte-escolha-conta = Choose the account
metas-fonte-etiqueta = Asset label
metas-fonte-etiqueta-dica = Every position with the label counts, also those labeled later.
metas-fonte-escolha-etiqueta = Choose the label
metas-fonte-sem-etiquetas = No labels yet
