# Orçamento, metas, aposentadoria, projetos e assinaturas (núcleo: `budget`, `goals`,
# `projects`, `subscriptions`): motivos do histórico, notas da simulação de aposentadoria, regras
# do INSS e erros. Montados na hora, no idioma da instalação. Erros em minúsculas, como frase de
# log (a interface põe a inicial maiúscula). pt-BR é o idioma de origem.

## Orçamento do mês (`budget`)

nucleo-orcamento-categoria = categoria { $id }
nucleo-orcamento-orcamento-de = orçamento de { $mes }
nucleo-orcamento-erro-previsto-negativo = o valor previsto não pode ser negativo
nucleo-orcamento-erro-meses-iguais = os meses de origem e destino são iguais
nucleo-orcamento-motivo-atualizado = Orçamento de { $mes } atualizado
nucleo-orcamento-motivo-copiado = Orçamento de { $de } copiado para { $para }
nucleo-orcamento-erro-meta-grupo = a meta de cada grupo de gasto vai de 0% a 100% da renda

## Assinaturas (`subscriptions`)

nucleo-orcamento-erro-assinaturas-so-interface = dispensar assinaturas só pela interface do app

## Projetos (`projects`)

nucleo-orcamento-projeto = projeto { $id }
nucleo-orcamento-item-projeto = item de projeto { $id }
nucleo-orcamento-bem = bem { $id }
nucleo-orcamento-lancamento = lançamento { $id }
nucleo-orcamento-erro-projeto-sem-nome = dê um nome ao projeto
nucleo-orcamento-erro-projeto-negativo = o orçamento do projeto não pode ser negativo
nucleo-orcamento-erro-projeto-fim-antes = o fim do projeto é antes do início
nucleo-orcamento-erro-item-sem-descricao = descreva o item do projeto
nucleo-orcamento-erro-item-valor = o valor do item tem de ser maior que zero (receita ou despesa vem do tipo)
nucleo-orcamento-motivo-projeto-criado = Projeto criado: { $nome }
nucleo-orcamento-motivo-projeto-alterado = Projeto alterado: { $nome }
nucleo-orcamento-motivo-projeto-removido = Projeto removido: { $nome }
nucleo-orcamento-motivo-lancamentos-no-projeto =
    { $n ->
        [one] { $n } lançamento no projeto { $nome }
       *[other] { $n } lançamentos no projeto { $nome }
    }
nucleo-orcamento-motivo-lancamentos-tirados =
    { $n ->
        [one] { $n } lançamento tirado do projeto
       *[other] { $n } lançamentos tirados do projeto
    }
nucleo-orcamento-motivo-item-adicionado = Item no projeto { $nome }: { $item }
nucleo-orcamento-motivo-item-alterado = Item de projeto alterado: { $item }
nucleo-orcamento-motivo-entradas-tiradas =
    { $n ->
        [one] { $n } entrada tirada do projeto
       *[other] { $n } entradas tiradas do projeto
    }

## Metas (`goals`)

nucleo-orcamento-meta = meta { $id }
nucleo-orcamento-conta = conta { $id }
nucleo-orcamento-erro-meta-sem-nome = informe o nome da meta
nucleo-orcamento-erro-meta-alvo = o valor-alvo da meta deve ser positivo
nucleo-orcamento-erro-meta-guardado = o valor já guardado não pode ser negativo
nucleo-orcamento-erro-meta-aporte = o aporte mensal planejado não pode ser negativo
nucleo-orcamento-erro-meta-retorno = o retorno real esperado precisa ficar entre −5% e 15% ao ano
nucleo-orcamento-erro-meta-arquivada = a meta está arquivada
nucleo-orcamento-motivo-meta-criada = Meta criada: { $nome }
nucleo-orcamento-motivo-meta-alterada = Meta alterada: { $nome }
nucleo-orcamento-motivo-meta-fonte-ligada = Fonte ligada à meta: { $nome }
nucleo-orcamento-motivo-meta-fonte-desligada = Fonte desligada da meta: { $nome }
nucleo-orcamento-meta-fonte-inicial = Guardado fora das contas
nucleo-orcamento-meta-fonte-carteira = Carteira (plano de aposentadoria)
nucleo-orcamento-meta-fonte-lancamentos = Lançamentos ligados
nucleo-orcamento-meta-erro-etiqueta = a etiqueta “{ $etiqueta }” não existe
nucleo-orcamento-meta-erro-ativo-sem-conta = o ativo não está em nenhuma conta; informe a conta
nucleo-orcamento-meta-erro-ativo-varias-contas = o ativo está em mais de uma conta; informe qual
nucleo-orcamento-meta-erro-fonte-nao-ligada = essa fonte não está ligada à meta
nucleo-orcamento-motivo-meta-removida = Meta removida: { $nome }

## Aposentadoria (`goals::retirement`)

nucleo-orcamento-cenario = cenário { $id }
nucleo-orcamento-erro-idade-hoje = informe a idade de hoje
nucleo-orcamento-erro-idades = idades: hoje entre 16 e 100, aposentadoria a partir de hoje e expectativa de vida depois da aposentadoria (até 110)
nucleo-orcamento-erro-retorno-volatilidade = retorno real entre −20% e 30% a.a. e volatilidade entre 0 e 60%
nucleo-orcamento-erro-cenario-sem-nome = dê um nome ao cenário
nucleo-orcamento-erro-cenario-repetido = já existe um cenário “{ $nome }”
nucleo-orcamento-erro-cenarios-maximo = até { $maximo } cenários; apague um antes
nucleo-orcamento-erro-plano-antes-da-meta = salve o plano de aposentadoria antes de ligar uma meta
nucleo-orcamento-motivo-plano-aposentadoria = Plano de aposentadoria
nucleo-orcamento-motivo-cenario-salvo = Cenário de aposentadoria: { $nome }
nucleo-orcamento-motivo-cenario-apagado = Cenário de aposentadoria apagado: { $nome }
nucleo-orcamento-motivo-meta-ligada = Meta de aposentadoria ligada ao plano
nucleo-orcamento-motivo-meta-desligada = Meta de aposentadoria desligada do plano
nucleo-orcamento-aposentadoria-nota-premissas = Valores reais de hoje; retorno real de { $retorno } a.a. com volatilidade de { $volatilidade }, { $simulacoes } simulações.
nucleo-orcamento-aposentadoria-nota-patrimonio = Patrimônio de hoje: o valor de mercado da carteira (inclui previdência).
nucleo-orcamento-aposentadoria-nota-inss = { $sistema } calculado pelas regras: { $valor } por mês a partir dos { $idade } anos ({ $regra }).
nucleo-orcamento-aposentadoria-regra-escolhida = regra escolhida
nucleo-orcamento-aposentadoria-nota-inss-nenhuma = { $sistema } calculado pelas regras: nenhuma regra é cumprida até os 100 anos; a simulação segue sem ele.
nucleo-orcamento-aposentadoria-nota-sem-inss = Sem valor de { $sistema } informado: a simulação supõe que não há benefício público.
# Nome e nota da meta criada ao ligar o plano (gravados no idioma da instalação).
nucleo-orcamento-aposentadoria-meta = Aposentadoria
nucleo-orcamento-aposentadoria-meta-nota = Ligada ao plano de aposentadoria (Orçamento › Aposentadoria).

## Estimativa do INSS pelas regras da EC 103/2019 (`goals::inss`)

nucleo-orcamento-inss-regra-permanente = Regra permanente (idade)
nucleo-orcamento-inss-regra-idade = Transição por idade
nucleo-orcamento-inss-regra-pontos = Transição por pontos
nucleo-orcamento-inss-regra-idade-progressiva = Transição por idade mínima progressiva
nucleo-orcamento-inss-regra-pedagio-50 = Transição com pedágio de 50%
nucleo-orcamento-inss-regra-pedagio-100 = Transição com pedágio de 100%
nucleo-orcamento-inss-req-permanente = { $idade } anos de idade e { $contribuicao } de contribuição
nucleo-orcamento-inss-req-idade = { $idade } anos de idade e 15 de contribuição
nucleo-orcamento-inss-req-pontos = { $contribuicao } anos de contribuição e idade + contribuição de { $pontos } pontos em { $ano } (sobe 1 por ano até { $maximo })
# A idade vem por extenso ("64 anos e 6 meses").
nucleo-orcamento-inss-req-idade-progressiva = { $contribuicao } anos de contribuição e { $idade } de idade em { $ano } (sobe 6 meses por ano até { $maximo })
nucleo-orcamento-inss-req-pedagio-50 = mais de { $minimo } anos de contribuição na reforma; { $contribuicao } anos mais metade do que faltava; benefício pelo fator previdenciário
nucleo-orcamento-inss-req-pedagio-100 = { $idade } anos de idade e { $total } de contribuição ({ $contribuicao } mais o tempo que faltava na reforma); 100% da média
nucleo-orcamento-inss-anos = { $anos } anos
nucleo-orcamento-inss-anos-meses = { $anos } anos e { $meses } meses
nucleo-orcamento-inss-nota-estimativa = Estimativa pelas regras da EC 103/2019, em reais de hoje; mínimo e teto de { $ano } ({ $fonte }).
nucleo-orcamento-inss-nota-reforma-suposta = Tempo de contribuição na reforma suposto: { $tempo } (contribuindo sem parar desde então).
nucleo-orcamento-inss-nota-so-permanente = Sem contribuição antes de 13/11/2019: só a regra permanente vale.
nucleo-orcamento-inss-nota-teto = A média passa do teto: o cálculo usa o teto.
nucleo-orcamento-inss-nota-fora = Fora do cálculo: atividade especial, professor, pessoa com deficiência, direito adquirido antes da reforma e o fator previdenciário do pedágio de 50%. Confira no Meu INSS.
nucleo-orcamento-inss-erro-idade-ou-nascimento = informe a idade de hoje ou a data de nascimento
nucleo-orcamento-inss-erro-idade = idade inválida
nucleo-orcamento-inss-erro-nascimento-futuro = data de nascimento no futuro
nucleo-orcamento-inss-erro-contribuicao = tempo de contribuição entre 0 e a idade menos 12 anos
nucleo-orcamento-inss-erro-media-negativa = a média dos salários não pode ser negativa
nucleo-orcamento-inss-nota-13 = O INSS paga 13 vezes por ano (com o abono anual); o plano soma as 13.
nucleo-orcamento-ssa-erro-pia = informe o PIA (o benefício na idade plena do extrato da SSA) ou o AIME
nucleo-orcamento-ssa-regra-antecipada = Pedido antecipado (62 anos)
nucleo-orcamento-ssa-regra-plena = Idade plena (FRA)
nucleo-orcamento-ssa-regra-adiada = Pedido adiado (70 anos)
nucleo-orcamento-ssa-regra-idade-plano = Pedido aos { $idade } anos
nucleo-orcamento-ssa-req-antecipada = a partir dos 62 anos, com 40 créditos; { $corte }% a menos que na idade plena ({ $plena })
nucleo-orcamento-ssa-req-plena = { $plena } de idade, com 40 créditos; 100% do PIA
nucleo-orcamento-ssa-req-adiada = 70 anos; { $acrescimo }% a mais que na idade plena (8% por ano de adiamento)
nucleo-orcamento-ssa-req-idade-plano = na idade em que o plano para de trabalhar
nucleo-orcamento-ssa-nota-estimativa = Estimativa pela fórmula da Social Security, em dólares de hoje ({ $fonte }).
nucleo-orcamento-ssa-nota-pia = PIA calculado do AIME: 90% até US$ { $bend1 }, 32% até US$ { $bend2 } e 15% acima. O extrato em ssa.gov/myaccount traz o valor exato.
nucleo-orcamento-ssa-nota-creditos = Sem os 40 créditos até a idade em que para de trabalhar: sem direito ao benefício.
nucleo-orcamento-ssa-nota-fora = Fora do cálculo: benefício de cônjuge e de sobrevivente, o teste de renda antes da idade plena e o imposto sobre o benefício.
nucleo-orcamento-pt-regra-normal = Idade pessoal de reforma
nucleo-orcamento-pt-regra-flexibilizacao = Antecipação por flexibilização
nucleo-orcamento-pt-regra-carreira-longa = Carreira contributiva muito longa
nucleo-orcamento-pt-req-normal = { $idade } em { $ano } (menos 4 meses por ano de carreira acima de 40) e 15 anos de descontos; bonificação por mês depois, até os 70
nucleo-orcamento-pt-req-flexibilizacao = a partir dos 60 anos com 40 de carreira; menos 0,5% por mês antes da idade pessoal, e o fator de sustentabilidade se os 40 anos não estavam feitos aos 60
nucleo-orcamento-pt-req-carreira-longa = a partir dos 60 anos com 48 de carreira (46 se começou antes dos 17), sem corte
nucleo-orcamento-pt-nota-estimativa = Estimativa pela fórmula da Segurança Social (regime geral), em euros de hoje; valores de { $ano } ({ $fonte }).
nucleo-orcamento-pt-nota-14 = A pensão é paga 14 vezes por ano (com os subsídios de Natal e de férias); o plano soma as 14.
nucleo-orcamento-pt-nota-idade-futura = A idade normal de acesso sobe com a esperança de vida aos 65; depois de 2027 o app mantém 66 anos e 11 meses.
nucleo-orcamento-pt-nota-antes-2002 = Carreira com descontos antes de 2002: a Segurança Social faz a média com a fórmula antiga (melhores 10 dos últimos 15 anos), e o valor pode mudar.
nucleo-orcamento-pt-nota-fora = Fora do cálculo: regimes especiais, desemprego de longa duração, pensão unificada e as regras de 2018 quando dão mais. Confira no simulador da Segurança Social Direta.
