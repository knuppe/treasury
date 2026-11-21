# Life timeline (core, `longevity`; docs/life-expectancy.md): stages, source and attribution of
# the tables, milestones, the questionnaire question, errors and the MCP summaries. The UN
# citation stays in English, as the UN asks.

## Stages of life (section 5)

nucleo-longevidade-fase-infancia = Childhood
nucleo-longevidade-fase-adolescencia = Adolescence
nucleo-longevidade-fase-juventude = Youth
nucleo-longevidade-fase-vida-adulta = Adulthood
nucleo-longevidade-fase-longevidade = Later life

## Source and attribution (DATA_LICENSES.md)

nucleo-longevidade-fonte-nome = UN, World Population Prospects 2024
nucleo-longevidade-fonte-nome-brasil = UN, World Population Prospects 2024, and IBGE 2024
nucleo-longevidade-fonte-nome-sintetica = Synthetic development table (not the UN data)
nucleo-longevidade-atribuicao-onu = United Nations, Department of Economic and Social Affairs, Population Division (2024). World Population Prospects 2024, Online Edition. License CC BY 3.0 IGO. Data subset and transformed by Treasury (only the app's countries, ages 0 to 100, medium variant, cohort life expectancy computed in the app); the UN does not endorse Treasury or this use of the data.
nucleo-longevidade-atribuicao-ibge = Brazil reference: IBGE, Tábuas Completas de Mortalidade 2024.
nucleo-longevidade-atribuicao-sintetica = Warning: this version uses a synthetic development table, not the UN or IBGE figures.
nucleo-longevidade-ibge-fonte = IBGE 2024
nucleo-longevidade-ibge-fonte-sintetica = IBGE 2024 (synthetic)

## Milestones on the bar

nucleo-longevidade-marco-aposentadoria = Retirement
nucleo-longevidade-marco-inss = INSS starts
nucleo-longevidade-marco-previdencia = Public pension starts
nucleo-longevidade-marco-fim-plano = End of the retirement plan
nucleo-longevidade-marco-divida = End of { $nome }

## Welcome questionnaire (8th question, family questionnaire only)

nucleo-longevidade-pergunta = Would you like to see the family's life timeline?
nucleo-longevidade-pergunta-dica = A bar with each person's stages of life and the money plans along it, based on population averages for the country. Some people prefer not to see this kind of information.
nucleo-longevidade-pergunta-sim = Yes, show me
nucleo-longevidade-pergunta-nao = No, thanks
nucleo-longevidade-pergunta-depois = Decide later

## Errors

nucleo-longevidade-erro-sem-nascimento = { $nome } has no date of birth: the life timeline needs it.
nucleo-longevidade-erro-pais = There is no life table for { $pais }.
nucleo-longevidade-erro-dados = The longevity tables built into the app could not be read: { $erro }
nucleo-longevidade-erro-so-interface = the life timeline settings can only be changed in the app
nucleo-longevidade-erro-desligada = The life timeline is turned off in this vault (Settings › Profile › Life timeline).
nucleo-longevidade-erro-escondido = { $nome } chose not to show their own life timeline.

## MCP summaries (the agent talks about population averages, never predictions)

nucleo-longevidade-mcp-membro = { $nome }, { $idade } years old ({ $fase }): by population averages (UN 2024 life tables), among people like { $nome }, half live past { $mediana }, 1 in 4 past { $p25 } and 1 in 10 past { $p10 }.
nucleo-longevidade-mcp-membro-crianca = { $nome }, { $idade } years old ({ $fase }): the end of the bar is hidden for children (family setting).
nucleo-longevidade-mcp-familia = { $n ->
        [one] Life timeline of 1 member.
       *[other] Life timeline of { $n } members.
    }
nucleo-longevidade-mcp-casal = Chance that at least one of the couple's two adults is alive: falls to 25% in { $ano25 } and to 10% in { $ano10 }.
nucleo-longevidade-mcp-faltam = { $n ->
        [one] 1 member is missing the date of birth or sex.
       *[other] { $n } members are missing the date of birth or sex.
    }
nucleo-longevidade-mcp-fonte = Population averages; source: { $fonte }.
nucleo-longevidade-mcp-desligada = The life timeline is turned off.
