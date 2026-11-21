# Linha da vida (núcleo, `longevity`; docs/life-expectancy.md): fases, fonte e atribuição das
# tábuas, marcos, a pergunta do questionário, erros e os resumos do MCP. pt-BR é o idioma de
# origem. A citação da ONU fica em inglês, como a ONU pede.

## Fases da vida (seção 5)

nucleo-longevidade-fase-infancia = Infância
nucleo-longevidade-fase-adolescencia = Adolescência
nucleo-longevidade-fase-juventude = Juventude
nucleo-longevidade-fase-vida-adulta = Vida adulta
nucleo-longevidade-fase-longevidade = Longevidade

## Fonte e atribuição (DATA_LICENSES.md)

nucleo-longevidade-fonte-nome = ONU, World Population Prospects 2024
nucleo-longevidade-fonte-nome-brasil = ONU, World Population Prospects 2024, e IBGE 2024
nucleo-longevidade-fonte-nome-sintetica = Tábua sintética de desenvolvimento (não são os dados da ONU)
nucleo-longevidade-atribuicao-onu = United Nations, Department of Economic and Social Affairs, Population Division (2024). World Population Prospects 2024, Online Edition. Licença CC BY 3.0 IGO. Dados recortados e transformados pelo Treasury (só os países do app, idades de 0 a 100, variante média, expectativa por geração calculada no app); a ONU não endossa o Treasury nem este uso dos dados.
nucleo-longevidade-atribuicao-ibge = Referência do Brasil: IBGE, Tábuas Completas de Mortalidade 2024.
nucleo-longevidade-atribuicao-sintetica = Atenção: esta versão usa uma tábua sintética de desenvolvimento, não os números da ONU nem do IBGE.
nucleo-longevidade-ibge-fonte = IBGE 2024
nucleo-longevidade-ibge-fonte-sintetica = IBGE 2024 (sintético)

## Marcos sobre a barra

nucleo-longevidade-marco-aposentadoria = Aposentadoria
nucleo-longevidade-marco-inss = Início do INSS
nucleo-longevidade-marco-previdencia = Início da aposentadoria pública
nucleo-longevidade-marco-fim-plano = Fim do plano de aposentadoria
nucleo-longevidade-marco-divida = Fim de { $nome }

## Questionário de boas-vindas (8ª pergunta, só no da família)

nucleo-longevidade-pergunta = Quer ver a linha da vida da família?
nucleo-longevidade-pergunta-dica = Uma barra com as fases da vida de cada pessoa e os planos de dinheiro ao longo dela, com base em médias da população do país. Algumas pessoas preferem não ver esse tipo de informação.
nucleo-longevidade-pergunta-sim = Sim, quero ver
nucleo-longevidade-pergunta-nao = Não, obrigado
nucleo-longevidade-pergunta-depois = Decidir depois

## Erros

nucleo-longevidade-erro-sem-nascimento = { $nome } não tem data de nascimento: a linha da vida precisa dela.
nucleo-longevidade-erro-pais = Não há tábua de mortalidade para { $pais }.
nucleo-longevidade-erro-dados = As tábuas de longevidade embutidas no app não puderam ser lidas: { $erro }
nucleo-longevidade-erro-so-interface = os ajustes da linha da vida só mudam pela interface do app
nucleo-longevidade-erro-desligada = A linha da vida está desligada neste cofre (Ajustes › Perfil › Linha da vida).
nucleo-longevidade-erro-escondido = { $nome } preferiu não mostrar a própria linha da vida.

## Resumos do MCP (o agente fala de médias da população, nunca de previsão)

nucleo-longevidade-mcp-membro = { $nome }, { $idade } anos ({ $fase }): pelas médias da população (tábuas da ONU de 2024), entre as pessoas como { $nome }, metade passa dos { $mediana } anos, 1 em 4 passa dos { $p25 } e 1 em 10, dos { $p10 }.
nucleo-longevidade-mcp-membro-crianca = { $nome }, { $idade } anos ({ $fase }): o fim da barra fica oculto para crianças (ajuste da família).
nucleo-longevidade-mcp-familia = { $n ->
        [1] Linha da vida de 1 membro.
       *[other] Linha da vida de { $n } membros.
    }
nucleo-longevidade-mcp-casal = Chance de pelo menos um dos dois adultos do casal estar vivo: cai a 25% em { $ano25 } e a 10% em { $ano10 }.
nucleo-longevidade-mcp-faltam = { $n ->
        [1] Falta a data de nascimento ou o sexo de 1 membro.
       *[other] Falta a data de nascimento ou o sexo de { $n } membros.
    }
nucleo-longevidade-mcp-fonte = Médias da população; fonte: { $fonte }.
nucleo-longevidade-mcp-desligada = A linha da vida está desligada.
