# Linha da vida (docs/life-expectancy.md): a barra com as fases da vida de cada membro, os marcos
# financeiros e o final esfumado pelas médias da população, na tela Família, no cadastro do
# membro, no planejador de aposentadoria, em Ajustes › Perfil e na barra ⌘K. Os textos do núcleo
# ficam em nucleo-longevidade.ftl. pt-BR é o idioma de origem.

longevidade-titulo = Linha da vida
longevidade-titulo-de = Linha da vida de { $nome }

## Fases da vida (seção 5)

longevidade-fase-infancia = Infância
longevidade-fase-adolescencia = Adolescência
longevidade-fase-juventude = Juventude
longevidade-fase-vida-adulta = Vida adulta
longevidade-fase-longevidade = Longevidade
longevidade-fase-longevidade-avancada = Longevidade avançada

## A barra

longevidade-cabecalho =
    { $nome } · { $idade ->
        [1] 1 ano
       *[other] { $idade } anos
    } · { $fase }
longevidade-hoje = { $idade } · hoje
longevidade-final-numeros = { $mediana } · { $p25 } · { $p10 }
longevidade-final-dica = Metade passa dos { $mediana }, 1 em 4 passa dos { $p25 } e 1 em 10 passa dos { $p10 }.
longevidade-marco-quando = aos { $idade } · em { $ano }
longevidade-marco-rotulo = { $rotulo }: aos { $idade }, em { $ano }
longevidade-marcos-rotulo = Marcos aos { $idade }: { $rotulos }
longevidade-marco-aposentadoria = Aposentadoria
longevidade-marco-inss = INSS
longevidade-marco-plano = Plano de aposentadoria
longevidade-marco-divida = Fim da dívida
longevidade-marco-meta = Meta
longevidade-marco-evento = Evento de vida

## Texto ao lado da barra, no tom do perfil

# $onde: longevidade-onde (o país com a preposição certa).
longevidade-onde =
    { $pais ->
        [BR] do Brasil
        [PT] de Portugal
        [US] dos Estados Unidos
       *[other] do país ({ $nome })
    }
longevidade-texto-simples =
    { $sexo ->
        [male] Pelas médias { $onde }, de cada 100 homens nascidos em { $ano } que chegaram aos { $idade }, metade passa dos { $mediana } e 1 em 4 passa dos { $p25 }.
        [female] Pelas médias { $onde }, de cada 100 mulheres nascidas em { $ano } que chegaram aos { $idade }, metade passa dos { $mediana } e 1 em 4 passa dos { $p25 }.
       *[other] Pelas médias { $onde }, de cada 100 pessoas nascidas em { $ano } que chegaram aos { $idade }, metade passa dos { $mediana } e 1 em 4 passa dos { $p25 }.
    }
longevidade-texto-equilibrado =
    { $sexo ->
        [male] Pelas médias { $onde }, de cada 100 homens nascidos em { $ano } que chegaram aos { $idade }, metade passa dos { $mediana }, 1 em 4 passa dos { $p25 } e 1 em 10 passa dos { $p10 }.
        [female] Pelas médias { $onde }, de cada 100 mulheres nascidas em { $ano } que chegaram aos { $idade }, metade passa dos { $mediana }, 1 em 4 passa dos { $p25 } e 1 em 10 passa dos { $p10 }.
       *[other] Pelas médias { $onde }, de cada 100 pessoas nascidas em { $ano } que chegaram aos { $idade }, metade passa dos { $mediana }, 1 em 4 passa dos { $p25 } e 1 em 10 passa dos { $p10 }.
    }
longevidade-texto-tecnico =
    Mediana { $mediana } · P25 { $p25 } · P10 { $p10 } · e({ $idade }) = { $restante } (coorte, { $fonte }, { $pais }, { $sexo ->
        [male] masculino
        [female] feminino
       *[other] ambos os sexos
    })
longevidade-texto-coorte = Expectativa ao nascer da geração de { $ano }: { $e0 } anos.
longevidade-texto-periodo = Tábua de período ({ $fonte }): e0 = { $e0 } · e({ $idade }) = { $restante }.
longevidade-texto-crianca = Fase de hoje: { $fase }. Para menores de 18, o final da barra fica escondido; um adulto pode mostrar em Ajustes › Perfil › Linha da vida.
longevidade-fonte = Médias da população. Fonte: { $fonte }.
longevidade-fonte-sintetica = Dados sintéticos de desenvolvimento, não as tábuas oficiais.

## Texto alternativo da barra (os mesmos números)

longevidade-alt =
    Linha da vida de { $nome }: { $idade ->
        [1] 1 ano
       *[other] { $idade } anos
    }, { $fase }.
longevidade-alt-final = Metade das pessoas da mesma geração passa dos { $mediana }, 1 em 4 passa dos { $p25 } e 1 em 10 passa dos { $p10 }.
longevidade-alt-marcos = Marcos: { $marcos }.
longevidade-alt-marco = { $rotulo } aos { $idade } ({ $ano })

## Quadro da família (02 · Família)

longevidade-familia-intro = A barra de cada pessoa mostra as fases da vida, onde ela está hoje e os planos de dinheiro ao longo dela. O final esfumado vem das médias da população do país, não de uma previsão.
longevidade-vista = Vista da linha da vida
longevidade-vista-idade = Por idade
longevidade-vista-tempo = Linha do tempo
longevidade-legenda-final = Final esfumado: metade, 1 em 4 e 1 em 10 das pessoas da mesma geração passam dessas idades.
longevidade-casal = Pelo menos um de vocês dois: pelas médias, a chance cai a 25% em { $ano25 } e a 10% em { $ano10 }.
longevidade-casal-marca = { $pct } em { $ano }
longevidade-tempo-na-aposentadoria =
    Na aposentadoria de { $nome } ({ $ano }): { $idade ->
        [1] 1 ano
       *[other] { $idade } anos
    } · { $fase }
longevidade-eixo-anos = Anos
longevidade-nenhum = Ninguém da família tem data de nascimento ainda.
longevidade-escondida = A barra de { $nome } está escondida (Ajustes › Perfil › Linha da vida).
longevidade-como = Como calculamos
longevidade-como-geracao = Por geração: segue as taxas de mortalidade que a geração de cada pessoa enfrenta a cada idade (as passadas e as projetadas pela ONU), e não as de um ano só.
longevidade-como-idade = A partir da idade de hoje: quem já chegou a uma idade passou pelos riscos de antes, e a idade esperada sobe.
longevidade-como-limites = São médias da população do país. Não consideram saúde, renda, tabagismo, região ou histórico familiar, que mudam muito o resultado de cada pessoa.
longevidade-como-futuro = O futuro é a projeção média da ONU, não um fato. Tudo é calculado no próprio computador; nada sai da máquina.

## O que falta no cadastro

longevidade-falta-titulo = Para completar a linha da vida
longevidade-falta-dica = A data de nascimento e o sexo escolhem a tábua certa. “Prefiro não informar” usa a média de todas as pessoas.
longevidade-falta-de = { $nome }: falta { $itens }
longevidade-falta-nascimento = data de nascimento
longevidade-falta-sexo = sexo
longevidade-falta-nascimento-de = Nascimento de { $nome }
longevidade-falta-sexo-de = Sexo de { $nome }
longevidade-falta-salvo = Cadastro de { $nome } salvo

## Ajustes › Perfil › Linha da vida

longevidade-ajustes-ligar = Mostrar a linha da vida
longevidade-ajustes-ligar-dica = Uma barra com as fases da vida de cada pessoa e os planos de dinheiro ao longo dela, com base em médias da população do país. Algumas pessoas preferem não ver esse tipo de informação.
longevidade-ajustes-minha = Mostrar a minha barra
longevidade-ajustes-minha-dica = Escondida, a sua barra some da Família e do planejador de aposentadoria; os outros membros continuam vendo as deles.
longevidade-ajustes-barras = Barras visíveis
longevidade-ajustes-barra-de = Mostrar a barra de { $nome }
longevidade-ajustes-criancas = Mostrar o final da barra para crianças
longevidade-ajustes-criancas-dica = Para menores de 18, a barra mostra as fases e os marcos; o final esfumado só aparece com este ajuste.
longevidade-ajustes-avancada = Destacar a longevidade avançada (80+)
longevidade-ajustes-avancada-dica = Divide a última fase aos 80 anos, quando saúde e cuidados pesam mais no orçamento.
longevidade-ajustes-nada-guardado = Desligar não apaga nada: a linha é calculada na hora, no próprio computador, a partir da data de nascimento, do sexo e do país.
longevidade-ajustes-ligada = Linha da vida ligada
longevidade-ajustes-desligada = Linha da vida desligada

## Planejador de aposentadoria (seção 7)

longevidade-apos-sugestao = Sugestão pela linha da vida: { $idade } (1 em 10 pessoas como você passa dessa idade).
longevidade-apos-sugestao-casal = Sugestão pela linha da vida: { $idade } (a chance de pelo menos um de vocês dois passar dessa idade é de 1 em 10).
longevidade-apos-usar = Usar { $idade }
longevidade-apos-risco =
    Seu plano vai até os { $idade }. Pelas médias, { $n ->
        [0] menos de 1 em 100 pessoas como você passa dessa idade.
        [1] cerca de 1 em 100 pessoas como você passa dessa idade.
        [10] 1 em 10 pessoas como você passa dessa idade.
       *[other] cerca de { $n } em 100 pessoas como você passam dessa idade.
    }
longevidade-apos-risco-casal =
    Seu plano vai até os { $idade }. Pelas médias, a chance de pelo menos um de vocês dois passar dessa idade é de { $n ->
        [0] menos de 1 em 100.
        [10] 1 em 10.
       *[other] cerca de { $n } em 100.
    }

## Questionário e barra ⌘K

longevidade-onboarding-ligada = A linha da vida está ligada. Se faltar a data de nascimento ou o sexo de alguém, a tela Família pede.
longevidade-onboarding-ir = Abrir a Família
longevidade-cmd-busca = linha da vida, expectativa de vida, longevidade, fases da vida, aposentadoria, idade
