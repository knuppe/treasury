# Children's college goal (docs/design/college-goal.md): wizard, card, detail, linked holdings
# and the annual review alerts.

## Annual review alerts

educacao-alerta-recalculo-titulo = College: the contribution for { $nome } changed
educacao-alerta-recalculo-corpo = In the annual review, the monthly contribution went from { $antes } to { $depois } (in today's money). Check today's tuition in the plan.
educacao-alerta-terminou-titulo = College: the course for { $nome } has ended
educacao-alerta-terminou-corpo = The goal can be archived, and what is left is free.

## Wizard

educacao-nova = College goal
educacao-editar-titulo = College plan · { $nome }
educacao-descricao = How much to save each month to pay for a child's college, with education inflation, in today's money.
educacao-passos = Plan steps
educacao-passo-quem = Who
educacao-passo-faculdade = Which college
educacao-passo-premissas = Assumptions
educacao-voltar = Back
educacao-avancar = Next
educacao-quem = Who it is for
educacao-filho-cadastrado = Child in the family
educacao-filho-vai-nascer = Not born yet
educacao-filho = Child
educacao-escolha-filho = Choose
educacao-filho-dica = { $idade } years old today; turns { $inicio } in { $ano }.
educacao-nascimento-previsto = Expected birth date
educacao-idade-inicio = Starting age
educacao-mes-inicio = Starting month
educacao-mes-inicio-dica = In Brazil, the academic year starts in February.
educacao-nome-meta = Goal name
educacao-nome-meta-dica = Optional; the default is "College for" and the child's name.
educacao-tipo = Type
educacao-tipo-particular = Private
educacao-tipo-publica = Public
educacao-tipo-exterior = Abroad
educacao-curso = Course
educacao-curso-dica = Optional, just as a reminder.
educacao-duracao = Length
educacao-duracao-dica = Medicine usually takes 6 years.
educacao-anos = years
educacao-parcelas = Payments per year
educacao-parcelas-n = { $n ->
    [1] 1 a year (annual)
    [2] 2 a year (semester)
    [4] 4 a year
   *[other] { $n } a year (monthly)
}
educacao-mensalidade = Today's monthly tuition
educacao-valor-parcela = Each payment today
educacao-mensalidade-dica = Ask the university for a quote: today's price is the basis of the calculation.
educacao-extras = Extra costs per year
educacao-extras-dica = Enrollment, materials, books.
educacao-custo-vida = Living costs per month
educacao-custo-vida-dica = Housing, transport and food in another city, if that's the case.
educacao-custo-vida-publica = At a public university, this is usually the cost: housing, transport and food.
educacao-desconto = Scholarship or discount
educacao-desconto-dica = On the tuition. The app doesn't estimate scholarships.
educacao-inflacao = Education inflation
educacao-inflacao-serie = 10-year average of the education series
educacao-inflacao-serie-nome = 10-year average: { $serie }
educacao-inflacao-geral = General inflation only
educacao-inflacao-fixa = Fixed difference
educacao-inflacao-dica = Calculated by the app from { $de } to { $ate } ({ $anos } complete years), with the series already downloaded.
educacao-inflacao-sem-dados = No complete years of the series yet: the calculation uses general inflation only until the next index sync.
educacao-diferenca = Difference over inflation
educacao-diferenca-dica = How much education rises above general inflation, per year.
educacao-pct-ano = % a year
educacao-retorno = Assumed real return
educacao-retorno-dica = Above inflation, for the saved money. It's the family's assumption.
educacao-aporte-modo = Contribution
educacao-aporte-corrigido = Adjusted for inflation every year
educacao-aporte-fixo = Fixed amount
educacao-ja-guardado-dica = Outside the accounts and holdings linked to the goal.
educacao-hipoteses = These are assumptions for planning, not a promise of returns or an investment suggestion.
educacao-falta-dado = Choose the child and enter the tuition to see the calculation.
educacao-erro-filho = Choose the child.
educacao-erro-nascimento = Add the child's birth date in Family.
educacao-erro-nascimento-previsto = Enter the expected birth date.
educacao-erro-idade = The starting age goes from 0 to 60.
educacao-erro-duracao = The length goes from half a year to 15 years.
educacao-erro-mensalidade = Enter today's tuition.
educacao-erro-desconto = The discount goes from 0% to 100%.
educacao-erro-retorno = The return goes from −5% to 15% a year.
educacao-erro-diferenca = The difference goes from −10% to 20% a year.

## Result

educacao-resultado = Plan result
educacao-alvo-inicio = Needed by { $data }
educacao-aporte-hoje = Contribution today
educacao-aporte-curso = Contribution during the course
educacao-mensalidade-inicio = Tuition at the start
educacao-mensalidade-epoca = about { $valor } in money of the time
educacao-aporte-ano-que-vem = In { $ano }, about { $valor } a month (adjusted for inflation).
educacao-aporte-fixo-valor = If you prefer a fixed amount: { $valor } a month.
educacao-so-ipca = If education only rose with general inflation: { $valor } a month.
educacao-cobre-tudo = What is saved covers the course to the end.
educacao-cobre-ate = { $ano ->
    [0] What is saved doesn't cover the first year.
    [one] What is saved covers up to year 1.
   *[other] What is saved covers up to year { $ano }.
}
educacao-selo-diferenca = Education { $pct } a year above inflation
educacao-selo-sem-diferenca = Education with general inflation
educacao-fonte-inflacao = { $selo } ({ $serie }, { $de } to { $ate }).

## Card and detail

educacao-n-anos = { $n ->
    [one] 1 year
   *[other] { $n } years
}
educacao-n-meses = { $n ->
    [one] 1 month
   *[other] { $n } months
}
educacao-e = { " and " }
educacao-card-faltam = Starts in { $prazo }
educacao-card-em-curso = In progress
educacao-card-terminou = Course ended
educacao-ver-plano = View plan
educacao-ver-plano-nome = View the plan for { $nome }
educacao-plano = College plan
educacao-periodo = { $inicio } to { $fim }
educacao-atualizar-mensalidade = Update the tuition
educacao-editar-plano = Edit the plan
educacao-aba-anos = Year by year
educacao-aba-guardado = What is saved
educacao-aba-historico = Reviews
educacao-col-ano = Year
educacao-col-mensalidade-hoje = Tuition (today)
educacao-col-mensalidade-epoca = Tuition (of the time)
educacao-col-pagamentos = Payments in the year
educacao-col-saldo = Balance at the start of the year
educacao-em-curso = course
educacao-tabela-nota = Amounts in today's money, except the "of the time" column, which applies the assumed general inflation. The balance follows the plan: contributions until the start, payments during the course.
educacao-guardado-hoje = saved today (linked accounts, holdings and transactions).
educacao-historico-vazio = No reviews yet.
educacao-revisao-anual = Annual review
educacao-revisao-edicao = Plan edited
educacao-revisao-mensalidade = Tuition updated
educacao-revisao-criada = Plan created

## Linked holdings (any goal)

educacao-posicoes = Holdings
educacao-posicoes-nome = Holdings linked to { $nome }
educacao-posicoes-titulo = Linked holdings · { $nome }
educacao-posicoes-dica = Link a whole account (cash and investments at market value) or one of its assets. With a share, the goal gets that part; without a share, the value becomes a common pot split among the goals, and whoever starts first gets first.
educacao-posicoes-vazio = No linked holdings.
educacao-posicoes-salvas = Holdings updated
educacao-posicoes-erro = Couldn't link the holding
educacao-posicao-conta = Account
educacao-posicao-escolha-conta = Choose the account
educacao-posicao-ativo = Asset
educacao-posicao-conta-inteira = Whole account
educacao-posicao-fracao = Share
educacao-posicao-fracao-dica = Empty: common pot.
educacao-posicao-ligar = Link
educacao-posicao-remover = Unlink { $nome }
educacao-pote = Common pot ({ $pct })
educacao-vencimento = Matures on { $vence }
educacao-vencimento-meta = Matures on { $vence }; the goal is for { $meta }.
educacao-sem-cotacao = No quote: valued at cost.

## Children's college

educacao-visao-titulo = Children's college
educacao-visao-aporte = Monthly contribution
educacao-visao-guardado = Saved
educacao-visao-alvo = Needed at the start
educacao-visao-linha = Courses timeline
educacao-visao-curso = Course from { $inicio } to { $fim }
educacao-visao-sobreposicao = Two courses at the same time in { $anos }.
educacao-visao-sem-sobreposicao = No year with two courses at the same time.
