# Meta de faculdade dos filhos (docs/design/college-goal.md): assistente, cartão, detalhe,
# posições ligadas e os alertas do recálculo anual. pt-BR é a origem.

## Alertas do recálculo anual

educacao-alerta-recalculo-titulo = Faculdade: o aporte de { $nome } mudou
educacao-alerta-recalculo-corpo = No recálculo anual, o aporte por mês foi de { $antes } para { $depois } (em dinheiro de hoje). Confira a mensalidade de hoje no plano.
educacao-alerta-terminou-titulo = Faculdade: o curso de { $nome } terminou
educacao-alerta-terminou-corpo = A meta pode ser arquivada, e o que sobrou fica livre.

## Assistente

educacao-nova = Meta de faculdade
educacao-editar-titulo = Plano de faculdade · { $nome }
educacao-descricao = Quanto guardar por mês para pagar a faculdade de um filho, com a inflação da educação, em dinheiro de hoje.
educacao-passos = Passos do plano
educacao-passo-quem = Quem
educacao-passo-faculdade = Qual faculdade
educacao-passo-premissas = Premissas
educacao-voltar = Voltar
educacao-avancar = Avançar
educacao-quem = Para quem é
educacao-filho-cadastrado = Filho cadastrado
educacao-filho-vai-nascer = Ainda vai nascer
educacao-filho = Filho
educacao-escolha-filho = Escolha
educacao-filho-dica = { $idade } anos hoje; faz { $inicio } em { $ano }.
educacao-nascimento-previsto = Data prevista do nascimento
educacao-idade-inicio = Idade de início
educacao-mes-inicio = Mês de início
educacao-mes-inicio-dica = No Brasil, o ano letivo começa em fevereiro.
educacao-nome-meta = Nome da meta
educacao-nome-meta-dica = Opcional; o padrão é "Faculdade de" e o nome do filho.
educacao-tipo = Tipo
educacao-tipo-particular = Particular
educacao-tipo-publica = Pública
educacao-tipo-exterior = No exterior
educacao-curso = Curso
educacao-curso-dica = Opcional, só para lembrar.
educacao-duracao = Duração
educacao-duracao-dica = Medicina costuma ter 6 anos.
educacao-anos = anos
educacao-parcelas = Parcelas por ano
educacao-parcelas-n = { $n ->
    [1] 1 por ano (anuidade)
    [2] 2 por ano (semestre)
    [4] 4 por ano
   *[other] { $n } por ano (mensalidade)
}
educacao-mensalidade = Mensalidade de hoje
educacao-valor-parcela = Valor de cada parcela hoje
educacao-mensalidade-dica = Peça uma cotação à faculdade: o preço de hoje é a base da conta.
educacao-extras = Gastos extras por ano
educacao-extras-dica = Matrícula, material, livros.
educacao-custo-vida = Custo de vida por mês
educacao-custo-vida-dica = Moradia, transporte e alimentação em outra cidade, se for o caso.
educacao-custo-vida-publica = Na pública, o custo costuma ser este: moradia, transporte e alimentação.
educacao-desconto = Bolsa ou desconto
educacao-desconto-dica = Sobre a mensalidade. O app não estima bolsa.
educacao-inflacao = Inflação da educação
educacao-inflacao-serie = Média de 10 anos da série de educação
educacao-inflacao-serie-nome = Média de 10 anos: { $serie }
educacao-inflacao-geral = Só a inflação geral
educacao-inflacao-fixa = Diferença fixa
educacao-inflacao-dica = Calculada pelo app de { $de } a { $ate } ({ $anos } anos completos), com as séries do IBGE já baixadas.
educacao-inflacao-sem-dados = Ainda sem anos completos da série: a conta usa só a inflação geral até a próxima sincronia dos índices.
educacao-diferenca = Diferença sobre a inflação
educacao-diferenca-dica = Quanto a educação sobe acima da inflação geral, por ano.
educacao-pct-ano = % a.a.
educacao-retorno = Retorno real suposto
educacao-retorno-dica = Acima da inflação, para o dinheiro guardado. É uma hipótese da família.
educacao-aporte-modo = Aporte
educacao-aporte-corrigido = Corrigido pela inflação todo ano
educacao-aporte-fixo = Fixo em reais
educacao-ja-guardado-dica = Fora das contas e posições ligadas à meta.
educacao-hipoteses = São hipóteses para planejar, não promessa de rendimento nem sugestão de investimento.
educacao-falta-dado = Escolha o filho e informe a mensalidade para ver a conta.
educacao-erro-filho = Escolha o filho.
educacao-erro-nascimento = Cadastre a data de nascimento do filho em Família.
educacao-erro-nascimento-previsto = Informe a data prevista do nascimento.
educacao-erro-idade = A idade de início vai de 0 a 60.
educacao-erro-duracao = A duração vai de meio ano a 15 anos.
educacao-erro-mensalidade = Informe a mensalidade de hoje.
educacao-erro-desconto = O desconto vai de 0% a 100%.
educacao-erro-retorno = O retorno vai de −5% a 15% ao ano.
educacao-erro-diferenca = A diferença vai de −10% a 20% ao ano.

## Resultado

educacao-resultado = Resultado do plano
educacao-alvo-inicio = Necessário em { $data }
educacao-aporte-hoje = Aporte hoje
educacao-aporte-curso = Aporte durante o curso
educacao-mensalidade-inicio = Mensalidade no início
educacao-mensalidade-epoca = cerca de { $valor } em dinheiro da época
educacao-aporte-ano-que-vem = Em { $ano }, cerca de { $valor } por mês (corrigido pela inflação).
educacao-aporte-fixo-valor = Se preferir um valor fixo em reais: { $valor } por mês.
educacao-so-ipca = Se a educação subisse só com a inflação geral: { $valor } por mês.
educacao-cobre-tudo = O que está guardado cobre o curso até o fim.
educacao-cobre-ate = { $ano ->
    [0] O que está guardado não cobre o primeiro ano.
    [one] O que está guardado cobre até o 1º ano.
   *[other] O que está guardado cobre até o { $ano }º ano.
}
educacao-selo-diferenca = Educação { $pct } a.a. acima da inflação
educacao-selo-sem-diferenca = Educação com a inflação geral
educacao-fonte-inflacao = { $selo } ({ $serie }, { $de } a { $ate }).

## Cartão e detalhe

educacao-n-anos = { $n ->
    [one] 1 ano
   *[other] { $n } anos
}
educacao-n-meses = { $n ->
    [one] 1 mês
   *[other] { $n } meses
}
educacao-e = { " e " }
educacao-card-faltam = Começa daqui a { $prazo }
educacao-card-em-curso = Em curso
educacao-card-terminou = Curso terminado
educacao-ver-plano = Ver plano
educacao-ver-plano-nome = Ver o plano de { $nome }
educacao-plano = Plano de faculdade
educacao-periodo = { $inicio } a { $fim }
educacao-atualizar-mensalidade = Atualizar a mensalidade
educacao-editar-plano = Editar o plano
educacao-aba-anos = Ano a ano
educacao-aba-guardado = O que está guardado
educacao-aba-historico = Recálculos
educacao-col-ano = Ano
educacao-col-mensalidade-hoje = Mensalidade (hoje)
educacao-col-mensalidade-epoca = Mensalidade (da época)
educacao-col-pagamentos = Pagamentos no ano
educacao-col-saldo = Saldo no começo do ano
educacao-em-curso = curso
educacao-tabela-nota = Valores em dinheiro de hoje, salvo a coluna "da época", que aplica a inflação geral suposta. O saldo segue o plano: aporte até o início, pagamentos durante o curso.
educacao-guardado-hoje = guardado hoje (contas, posições e lançamentos ligados).
educacao-historico-vazio = Ainda sem recálculos.
educacao-revisao-anual = Recálculo anual
educacao-revisao-edicao = Plano editado
educacao-revisao-mensalidade = Mensalidade atualizada
educacao-revisao-criada = Plano criado

## Posições ligadas (qualquer meta)

educacao-posicoes = Posições
educacao-posicoes-nome = Posições ligadas a { $nome }
educacao-posicoes-titulo = Posições ligadas · { $nome }
educacao-posicoes-dica = Ligue uma conta inteira (caixa e investimentos a mercado) ou um ativo dela. Com fração, a meta fica com aquela parte; sem fração, o valor vira um pote comum dividido entre as metas, e quem começa primeiro recebe primeiro.
educacao-posicoes-vazio = Nenhuma posição ligada.
educacao-posicoes-salvas = Posições atualizadas
educacao-posicoes-erro = Não deu para ligar a posição
educacao-posicao-conta = Conta
educacao-posicao-escolha-conta = Escolha a conta
educacao-posicao-ativo = Ativo
educacao-posicao-conta-inteira = Conta inteira
educacao-posicao-fracao = Fração
educacao-posicao-fracao-dica = Vazio: pote comum.
educacao-posicao-ligar = Ligar
educacao-posicao-remover = Desligar { $nome }
educacao-pote = Pote comum ({ $pct })
educacao-vencimento = Vence em { $vence }
educacao-vencimento-meta = Vence em { $vence }; a meta é para { $meta }.
educacao-sem-cotacao = Sem cotação: vale pelo custo.

## Faculdade dos filhos

educacao-visao-titulo = Faculdade dos filhos
educacao-visao-aporte = Aporte por mês
educacao-visao-guardado = Guardado
educacao-visao-alvo = Necessário no início
educacao-visao-linha = Linha do tempo dos cursos
educacao-visao-curso = Curso de { $inicio } a { $fim }
educacao-visao-sobreposicao = Dois cursos ao mesmo tempo em { $anos }.
educacao-visao-sem-sobreposicao = Nenhum ano com dois cursos ao mesmo tempo.
