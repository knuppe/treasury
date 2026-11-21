# Life timeline (docs/life-expectancy.md): the bar with each member's life stages, the financial
# milestones and the faded end from population averages, on the Family screen, in the member's
# profile, in the retirement planner, in Settings › Profile and in the ⌘K bar. Core texts live in
# nucleo-longevidade.ftl.
# Revision: AI draft, not yet reviewed by a native speaker.

longevidade-titulo = Life timeline
longevidade-titulo-de = { $nome }'s life timeline

## Life stages (section 5)

longevidade-fase-infancia = Childhood
longevidade-fase-adolescencia = Adolescence
longevidade-fase-juventude = Youth
longevidade-fase-vida-adulta = Adulthood
longevidade-fase-longevidade = Later life
longevidade-fase-longevidade-avancada = Advanced later life

## The bar

longevidade-cabecalho =
    { $nome } · { $idade ->
        [one] 1 year
       *[other] { $idade } years
    } · { $fase }
longevidade-hoje = { $idade } · today
longevidade-final-numeros = { $mediana } · { $p25 } · { $p10 }
longevidade-final-dica = Half live past { $mediana }, 1 in 4 lives past { $p25 } and 1 in 10 lives past { $p10 }.
longevidade-marco-quando = at { $idade } · in { $ano }
longevidade-marco-rotulo = { $rotulo }: at { $idade }, in { $ano }
longevidade-marcos-rotulo = Milestones at { $idade }: { $rotulos }
longevidade-marco-aposentadoria = Retirement
longevidade-marco-inss = INSS
longevidade-marco-plano = Retirement plan
longevidade-marco-divida = Debt paid off
longevidade-marco-meta = Goal
longevidade-marco-evento = Life event

## Text next to the bar, in the profile's tone

# $onde: longevidade-onde (the country with the right preposition).
longevidade-onde =
    { $pais ->
        [US] for the United States
       *[other] for { $nome }
    }
longevidade-texto-simples =
    { $sexo ->
        [male] By the averages { $onde }, out of every 100 men born in { $ano } who reached { $idade }, half live past { $mediana } and 1 in 4 lives past { $p25 }.
        [female] By the averages { $onde }, out of every 100 women born in { $ano } who reached { $idade }, half live past { $mediana } and 1 in 4 lives past { $p25 }.
       *[other] By the averages { $onde }, out of every 100 people born in { $ano } who reached { $idade }, half live past { $mediana } and 1 in 4 lives past { $p25 }.
    }
longevidade-texto-equilibrado =
    { $sexo ->
        [male] By the averages { $onde }, out of every 100 men born in { $ano } who reached { $idade }, half live past { $mediana }, 1 in 4 lives past { $p25 } and 1 in 10 lives past { $p10 }.
        [female] By the averages { $onde }, out of every 100 women born in { $ano } who reached { $idade }, half live past { $mediana }, 1 in 4 lives past { $p25 } and 1 in 10 lives past { $p10 }.
       *[other] By the averages { $onde }, out of every 100 people born in { $ano } who reached { $idade }, half live past { $mediana }, 1 in 4 lives past { $p25 } and 1 in 10 lives past { $p10 }.
    }
longevidade-texto-tecnico =
    Median { $mediana } · P25 { $p25 } · P10 { $p10 } · e({ $idade }) = { $restante } (cohort, { $fonte }, { $pais }, { $sexo ->
        [male] male
        [female] female
       *[other] both sexes
    })
longevidade-texto-coorte = Life expectancy at birth for the { $ano } generation: { $e0 } years.
longevidade-texto-periodo = Period table ({ $fonte }): e0 = { $e0 } · e({ $idade }) = { $restante }.
longevidade-texto-crianca = Current stage: { $fase }. For people under 18, the end of the bar stays hidden; an adult can show it in Settings › Profile › Life timeline.
longevidade-fonte = Population averages. Source: { $fonte }.
longevidade-fonte-sintetica = Synthetic development data, not the official tables.

## Text alternative for the bar (the same numbers)

longevidade-alt =
    { $nome }'s life timeline: { $idade ->
        [one] 1 year
       *[other] { $idade } years
    }, { $fase }.
longevidade-alt-final = Half of the people of the same generation live past { $mediana }, 1 in 4 lives past { $p25 } and 1 in 10 lives past { $p10 }.
longevidade-alt-marcos = Milestones: { $marcos }.
longevidade-alt-marco = { $rotulo } at { $idade } ({ $ano })

## Family card (02 · Family)

longevidade-familia-intro = Each person's bar shows the stages of life, where they are today and the money plans along the way. The faded end comes from the country's population averages, not from a prediction.
longevidade-vista = Life timeline view
longevidade-vista-idade = By age
longevidade-vista-tempo = Timeline
longevidade-legenda-final = Faded end: half, 1 in 4 and 1 in 10 of the people of the same generation live past these ages.
longevidade-casal = At least one of you two: by the averages, the chance drops to 25% in { $ano25 } and to 10% in { $ano10 }.
longevidade-casal-marca = { $pct } in { $ano }
longevidade-tempo-na-aposentadoria =
    At { $nome }'s retirement ({ $ano }): { $idade ->
        [one] 1 year
       *[other] { $idade } years
    } · { $fase }
longevidade-eixo-anos = Years
longevidade-nenhum = No one in the family has a date of birth yet.
longevidade-escondida = { $nome }'s bar is hidden (Settings › Profile › Life timeline).
longevidade-como = How we calculate
longevidade-como-geracao = By generation: it follows the death rates each person's generation faces at each age (past ones and those projected by the UN), not the rates of a single year.
longevidade-como-idade = From today's age: someone who has reached an age has already been through the earlier risks, so the expected age goes up.
longevidade-como-limites = These are the country's population averages. They don't take into account health, income, smoking, region or family history, which change each person's outcome a lot.
longevidade-como-futuro = The future is the UN's average projection, not a fact. Everything is calculated on this computer; nothing leaves the machine.

## What is missing in the profile

longevidade-falta-titulo = To complete the life timeline
longevidade-falta-dica = The date of birth and sex pick the right table. “Prefer not to say” uses the average of all people.
longevidade-falta-de = { $nome }: missing { $itens }
longevidade-falta-nascimento = date of birth
longevidade-falta-sexo = sex
longevidade-falta-nascimento-de = { $nome }'s date of birth
longevidade-falta-sexo-de = { $nome }'s sex
longevidade-falta-salvo = { $nome }'s profile saved

## Settings › Profile › Life timeline

longevidade-ajustes-ligar = Show the life timeline
longevidade-ajustes-ligar-dica = A bar with each person's life stages and the money plans along the way, based on the country's population averages. Some people prefer not to see this kind of information.
longevidade-ajustes-minha = Show my bar
longevidade-ajustes-minha-dica = When hidden, your bar disappears from Family and the retirement planner; the other members still see theirs.
longevidade-ajustes-barras = Visible bars
longevidade-ajustes-barra-de = Show { $nome }'s bar
longevidade-ajustes-criancas = Show the end of the bar for children
longevidade-ajustes-criancas-dica = For people under 18, the bar shows the stages and milestones; the faded end only appears with this setting.
longevidade-ajustes-avancada = Highlight advanced later life (80+)
longevidade-ajustes-avancada-dica = Splits the last stage at 80, when health and care weigh more on the budget.
longevidade-ajustes-nada-guardado = Turning it off deletes nothing: the timeline is calculated on the spot, on this computer, from the date of birth, sex and country.
longevidade-ajustes-ligada = Life timeline on
longevidade-ajustes-desligada = Life timeline off

## Retirement planner (section 7)

longevidade-apos-sugestao = Life timeline suggestion: { $idade } (1 in 10 people like you lives past that age).
longevidade-apos-sugestao-casal = Life timeline suggestion: { $idade } (the chance that at least one of you two lives past that age is 1 in 10).
longevidade-apos-usar = Use { $idade }
longevidade-apos-risco =
    Your plan runs to age { $idade }. By the averages, { $n ->
        [0] fewer than 1 in 100 people like you live past that age.
        [1] about 1 in 100 people like you lives past that age.
        [10] 1 in 10 people like you lives past that age.
       *[other] about { $n } in 100 people like you live past that age.
    }
longevidade-apos-risco-casal =
    Your plan runs to age { $idade }. By the averages, the chance that at least one of you two lives past that age is { $n ->
        [0] less than 1 in 100.
        [10] 1 in 10.
       *[other] about { $n } in 100.
    }

## Questionnaire and ⌘K bar

longevidade-onboarding-ligada = The life timeline is on. If someone's date of birth or sex is missing, the Family screen asks for it.
longevidade-onboarding-ir = Open Family
longevidade-cmd-busca = life timeline, life expectancy, longevity, life stages, retirement, age
