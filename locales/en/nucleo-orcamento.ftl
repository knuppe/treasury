# Budget, goals, retirement, projects and subscriptions (core: `budget`, `goals`, `projects`,
# `subscriptions`): change history reasons, retirement simulation notes, INSS rules and errors.
# Built on the fly, in the installation language. Errors in lowercase, like a log line (the
# interface capitalizes them). INSS, EC 103/2019 and Meu INSS stay as they are.
# Revision: AI draft, not yet reviewed by a native speaker.

## Monthly budget (`budget`)

nucleo-orcamento-categoria = category { $id }
nucleo-orcamento-orcamento-de = budget for { $mes }
nucleo-orcamento-erro-previsto-negativo = the planned amount can't be negative
nucleo-orcamento-erro-meses-iguais = the source and destination months are the same
nucleo-orcamento-motivo-atualizado = Budget for { $mes } updated
nucleo-orcamento-motivo-copiado = Budget for { $de } copied to { $para }
nucleo-orcamento-erro-meta-grupo = each spending group's target goes from 0% to 100% of income

## Subscriptions (`subscriptions`)

nucleo-orcamento-erro-assinaturas-so-interface = subscriptions can only be dismissed in the app's interface

## Projects (`projects`)

nucleo-orcamento-projeto = project { $id }
nucleo-orcamento-item-projeto = project item { $id }
nucleo-orcamento-bem = asset { $id }
nucleo-orcamento-lancamento = transaction { $id }
nucleo-orcamento-erro-projeto-sem-nome = give the project a name
nucleo-orcamento-erro-projeto-negativo = the project budget can't be negative
nucleo-orcamento-erro-projeto-fim-antes = the project ends before it starts
nucleo-orcamento-erro-item-sem-descricao = describe the project item
nucleo-orcamento-erro-item-valor = the item amount must be greater than zero (income or expense comes from the type)
nucleo-orcamento-motivo-projeto-criado = Project created: { $nome }
nucleo-orcamento-motivo-projeto-alterado = Project changed: { $nome }
nucleo-orcamento-motivo-projeto-removido = Project removed: { $nome }
nucleo-orcamento-motivo-lancamentos-no-projeto =
    { $n ->
        [one] { $n } transaction added to project { $nome }
       *[other] { $n } transactions added to project { $nome }
    }
nucleo-orcamento-motivo-lancamentos-tirados =
    { $n ->
        [one] { $n } transaction removed from the project
       *[other] { $n } transactions removed from the project
    }
nucleo-orcamento-motivo-item-adicionado = Item in project { $nome }: { $item }
nucleo-orcamento-motivo-item-alterado = Project item changed: { $item }
nucleo-orcamento-motivo-entradas-tiradas =
    { $n ->
        [one] { $n } entry removed from the project
       *[other] { $n } entries removed from the project
    }

## Goals (`goals`)

nucleo-orcamento-meta = goal { $id }
nucleo-orcamento-conta = account { $id }
nucleo-orcamento-erro-meta-sem-nome = enter the goal name
nucleo-orcamento-erro-meta-alvo = the goal's target amount must be positive
nucleo-orcamento-erro-meta-guardado = the amount already saved can't be negative
nucleo-orcamento-erro-meta-aporte = the planned monthly contribution can't be negative
nucleo-orcamento-erro-meta-retorno = the expected real return must be between −5% and 15% a year
nucleo-orcamento-erro-meta-arquivada = the goal is archived
nucleo-orcamento-motivo-meta-criada = Goal created: { $nome }
nucleo-orcamento-motivo-meta-alterada = Goal changed: { $nome }
nucleo-orcamento-motivo-meta-fonte-ligada = Source linked to the goal: { $nome }
nucleo-orcamento-motivo-meta-fonte-desligada = Source removed from the goal: { $nome }
nucleo-orcamento-meta-fonte-inicial = Saved outside the accounts
nucleo-orcamento-meta-fonte-carteira = Portfolio (retirement plan)
nucleo-orcamento-meta-fonte-lancamentos = Linked transactions
nucleo-orcamento-meta-erro-etiqueta = the label “{ $etiqueta }” does not exist
nucleo-orcamento-meta-erro-ativo-sem-conta = the asset is not in any account; give the account
nucleo-orcamento-meta-erro-ativo-varias-contas = the asset is in more than one account; say which
nucleo-orcamento-meta-erro-fonte-nao-ligada = that source is not linked to the goal
nucleo-orcamento-motivo-meta-removida = Goal removed: { $nome }

## Retirement (`goals::retirement`)

nucleo-orcamento-cenario = scenario { $id }
nucleo-orcamento-erro-idade-hoje = enter today's age
nucleo-orcamento-erro-idades = ages: today between 16 and 100, retirement from today on and life expectancy after retirement (up to 110)
nucleo-orcamento-erro-retorno-volatilidade = real return between −20% and 30% a year and volatility between 0 and 60%
nucleo-orcamento-erro-cenario-sem-nome = give the scenario a name
nucleo-orcamento-erro-cenario-repetido = there's already a scenario called “{ $nome }”
nucleo-orcamento-erro-cenarios-maximo = up to { $maximo } scenarios; delete one first
nucleo-orcamento-erro-plano-antes-da-meta = save the retirement plan before linking a goal
nucleo-orcamento-motivo-plano-aposentadoria = Retirement plan
nucleo-orcamento-motivo-cenario-salvo = Retirement scenario: { $nome }
nucleo-orcamento-motivo-cenario-apagado = Retirement scenario deleted: { $nome }
nucleo-orcamento-motivo-meta-ligada = Retirement goal linked to the plan
nucleo-orcamento-motivo-meta-desligada = Retirement goal unlinked from the plan
nucleo-orcamento-aposentadoria-nota-premissas = Today's real values; real return of { $retorno } a year with { $volatilidade } volatility, { $simulacoes } simulations.
nucleo-orcamento-aposentadoria-nota-patrimonio = Today's wealth: the market value of the portfolio (includes private pension plans).
nucleo-orcamento-aposentadoria-nota-inss = { $sistema } calculated by the rules: { $valor } a month from age { $idade } ({ $regra }).
nucleo-orcamento-aposentadoria-regra-escolhida = chosen rule
nucleo-orcamento-aposentadoria-nota-inss-nenhuma = { $sistema } calculated by the rules: no rule is met by age 100; the simulation goes on without it.
nucleo-orcamento-aposentadoria-nota-sem-inss = No { $sistema } amount entered: the simulation assumes there is no public benefit.
# Name and note of the goal created when linking the plan (saved in the installation language).
nucleo-orcamento-aposentadoria-meta = Retirement
nucleo-orcamento-aposentadoria-meta-nota = Linked to the retirement plan (Budget › Retirement).

## INSS estimate by the EC 103/2019 rules (`goals::inss`)

nucleo-orcamento-inss-regra-permanente = Permanent rule (age)
nucleo-orcamento-inss-regra-idade = Transition by age
nucleo-orcamento-inss-regra-pontos = Transition by points
nucleo-orcamento-inss-regra-idade-progressiva = Transition by progressive minimum age
nucleo-orcamento-inss-regra-pedagio-50 = Transition with a 50% toll
nucleo-orcamento-inss-regra-pedagio-100 = Transition with a 100% toll
nucleo-orcamento-inss-req-permanente = age { $idade } and { $contribuicao } years of contributions
nucleo-orcamento-inss-req-idade = age { $idade } and 15 years of contributions
nucleo-orcamento-inss-req-pontos = { $contribuicao } years of contributions and age + contributions of { $pontos } points in { $ano } (goes up 1 a year until { $maximo })
# The age comes spelled out ("64 years and 6 months").
nucleo-orcamento-inss-req-idade-progressiva = { $contribuicao } years of contributions and an age of { $idade } in { $ano } (goes up 6 months a year until { $maximo })
nucleo-orcamento-inss-req-pedagio-50 = more than { $minimo } years of contributions at the reform; { $contribuicao } years plus half of what was missing; benefit by the social security factor
nucleo-orcamento-inss-req-pedagio-100 = age { $idade } and { $total } of contributions ({ $contribuicao } plus the time that was missing at the reform); 100% of the average
nucleo-orcamento-inss-anos =
    { $anos ->
        [one] { $anos } year
       *[other] { $anos } years
    }
nucleo-orcamento-inss-anos-meses =
    { $meses ->
        [one] { $anos } years and { $meses } month
       *[other] { $anos } years and { $meses } months
    }
nucleo-orcamento-inss-nota-estimativa = Estimate by the EC 103/2019 rules, in today's reais; minimum and ceiling from { $ano } ({ $fonte }).
nucleo-orcamento-inss-nota-reforma-suposta = Contributions at the reform assumed: { $tempo } (contributing without a break since then).
nucleo-orcamento-inss-nota-so-permanente = No contributions before Nov 13, 2019: only the permanent rule applies.
nucleo-orcamento-inss-nota-teto = The average is above the ceiling: the calculation uses the ceiling.
nucleo-orcamento-inss-nota-fora = Not included: special activity, teachers, people with disabilities, rights acquired before the reform and the social security factor of the 50% toll. Check on Meu INSS.
nucleo-orcamento-inss-erro-idade-ou-nascimento = enter today's age or the date of birth
nucleo-orcamento-inss-erro-idade = invalid age
nucleo-orcamento-inss-erro-nascimento-futuro = date of birth in the future
nucleo-orcamento-inss-erro-contribuicao = contribution time between 0 and the age minus 12 years
nucleo-orcamento-inss-erro-media-negativa = the average salary can't be negative
nucleo-orcamento-inss-nota-13 = INSS pays 13 times a year (with the annual bonus); the plan counts all 13.
nucleo-orcamento-ssa-erro-pia = enter the PIA (the benefit at full retirement age from your SSA statement) or the AIME
nucleo-orcamento-ssa-regra-antecipada = Early claim (age 62)
nucleo-orcamento-ssa-regra-plena = Full retirement age (FRA)
nucleo-orcamento-ssa-regra-adiada = Delayed claim (age 70)
nucleo-orcamento-ssa-regra-idade-plano = Claim at { $idade }
nucleo-orcamento-ssa-req-antecipada = from age 62, with 40 credits; { $corte }% less than at full retirement age ({ $plena })
nucleo-orcamento-ssa-req-plena = age { $plena }, with 40 credits; 100% of the PIA
nucleo-orcamento-ssa-req-adiada = age 70; { $acrescimo }% more than at full retirement age (8% per year of delay)
nucleo-orcamento-ssa-req-idade-plano = at the age the plan stops working
nucleo-orcamento-ssa-nota-estimativa = Estimate by the Social Security formula, in today's dollars ({ $fonte }).
nucleo-orcamento-ssa-nota-pia = PIA computed from the AIME: 90% up to US$ { $bend1 }, 32% up to US$ { $bend2 } and 15% above. Your statement at ssa.gov/myaccount has the exact amount.
nucleo-orcamento-ssa-nota-creditos = Without 40 credits by the age you stop working: no right to the benefit.
nucleo-orcamento-ssa-nota-fora = Not included: spousal and survivor benefits, the earnings test before full retirement age and the tax on benefits.
nucleo-orcamento-pt-regra-normal = Personal retirement age
nucleo-orcamento-pt-regra-flexibilizacao = Early retirement (flexibility scheme)
nucleo-orcamento-pt-regra-carreira-longa = Very long contribution career
nucleo-orcamento-pt-req-normal = { $idade } in { $ano } (4 months less per year of career above 40) and 15 years of contributions; a monthly bonus after that, up to 70
nucleo-orcamento-pt-req-flexibilizacao = from age 60 with a 40-year career; 0.5% less per month before the personal age, plus the sustainability factor if the 40 years were not complete at 60
nucleo-orcamento-pt-req-carreira-longa = from age 60 with a 48-year career (46 if started before 17), no cut
nucleo-orcamento-pt-nota-estimativa = Estimate by the Social Security formula (general scheme), in today's euros; { $ano } values ({ $fonte }).
nucleo-orcamento-pt-nota-14 = The pension is paid 14 times a year (with the Christmas and holiday allowances); the plan counts all 14.
nucleo-orcamento-pt-nota-idade-futura = The normal access age rises with life expectancy at 65; after 2027 the app keeps 66 years and 11 months.
nucleo-orcamento-pt-nota-antes-2002 = Contributions before 2002: Social Security blends in the old formula (best 10 of the last 15 years), so the amount may change.
nucleo-orcamento-pt-nota-fora = Not included: special schemes, long-term unemployment, unified pensions and the 2018 rules when they pay more. Check the Segurança Social Direta simulator.
