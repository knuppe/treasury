# Lançamentos, recorrências, regras e categorias (núcleo: `transactions`, `recurrence`, `rules`,
# `categories`): erros de validação, motivos do histórico, motivos das sugestões e rótulos que o
# núcleo monta na hora, no idioma da instalação. pt-BR é o idioma de origem. Os nomes das
# categorias padrão ficam em categorias.ftl; os textos que casam com o extrato do banco (regras
# prontas) ficam no código.

## Comum

nucleo-lancamentos-sem-descricao = (sem descrição)
nucleo-lancamentos-ref-lancamento = lançamento { $id }
nucleo-lancamentos-ref-membro = membro { $id }
nucleo-lancamentos-ref-tag = tag { $id }
nucleo-lancamentos-ref-anexo = anexo { $id }
nucleo-lancamentos-erro-nome-vazio = { $coisa ->
    [conta] informe o nome da conta
    [categoria] informe o nome da categoria
    [tag] informe o nome da tag
    [membro] informe o nome do membro
    [familia] informe o nome da família
    [instituicao] informe o nome da instituição
    [renda] informe o nome da renda
   *[outro] informe o nome
}
nucleo-lancamentos-erro-nome-longo = { $coisa ->
    [conta] o nome da conta pode ter no máximo { $max } caracteres
    [categoria] o nome da categoria pode ter no máximo { $max } caracteres
    [tag] o nome da tag pode ter no máximo { $max } caracteres
    [membro] o nome do membro pode ter no máximo { $max } caracteres
    [familia] o nome da família pode ter no máximo { $max } caracteres
    [instituicao] o nome da instituição pode ter no máximo { $max } caracteres
    [renda] o nome da renda pode ter no máximo { $max } caracteres
   *[outro] o nome pode ter no máximo { $max } caracteres
}
nucleo-lancamentos-erro-cor = cor inválida: { $cor }
nucleo-lancamentos-erro-periodo-invertido = a data inicial é posterior à data final
nucleo-lancamentos-erro-maximo-lote = no máximo { $max } lançamentos por vez

## Tipos e status (minúsculos, para o meio da frase)

nucleo-lancamentos-tipo-receita = receita
nucleo-lancamentos-tipo-despesa = despesa
nucleo-lancamentos-tipo-transferencia = transferência
nucleo-lancamentos-tipo-pagamento-fatura = pagamento de fatura
nucleo-lancamentos-tipo-investimento = aporte/resgate de investimento
nucleo-lancamentos-tipo-ajuste = ajuste
nucleo-lancamentos-status-previsto = previsto
nucleo-lancamentos-status-efetivado = efetivado
nucleo-lancamentos-status-conciliado = conciliado

## Histórico (motivo de cada alteração)

nucleo-lancamentos-historico-criado = Lançamento criado: { $descricao }
nucleo-lancamentos-historico-alterado = Lançamento alterado: { $descricao }
nucleo-lancamentos-historico-apagado = Lançamento apagado: { $descricao }
nucleo-lancamentos-historico-apagados = { $n } lançamentos apagados
nucleo-lancamentos-historico-copias-apagadas = { $n } cópias entre contas apagadas
nucleo-lancamentos-historico-transferencia = Transferência: { $origem } → { $destino }
nucleo-lancamentos-historico-pagamento-fatura = Pagamento de fatura: { $origem } → { $destino }
nucleo-lancamentos-historico-aporte = Aporte: { $origem } → { $destino }
nucleo-lancamentos-historico-categoria-definida = Categoria { $categoria } definida em { $n } lançamento(s)
nucleo-lancamentos-historico-categoria-removida = Categoria removida de { $n } lançamento(s)
nucleo-lancamentos-historico-status = Status alterado para { $status } em { $n } lançamento(s)
nucleo-lancamentos-historico-membro-definido = Membro { $membro } definido em { $n } lançamento(s)
nucleo-lancamentos-historico-membro-removido = Membro removido de { $n } lançamento(s)
nucleo-lancamentos-historico-tags = Tags alteradas em { $n } lançamento(s)
nucleo-lancamentos-historico-par-ligado = Lançamentos ligados como transferência entre contas
nucleo-lancamentos-historico-pares-ligados = { $n } pares ligados como transferências entre contas
nucleo-lancamentos-historico-vinculados = { $n } lançamento(s) vinculado(s) a { $operacao }
nucleo-lancamentos-historico-vinculado = Lançamento vinculado: { $descricao } → { $operacao }
nucleo-lancamentos-historico-desvinculado = Vínculo desfeito: { $descricao }
nucleo-lancamentos-historico-transferencia-desligada = Transferência desligada: as duas pernas ficaram soltas
nucleo-lancamentos-historico-tag-criada = Tag criada: { $nome }
nucleo-lancamentos-historico-tag-alterada = Tag alterada
nucleo-lancamentos-historico-tag-apagada = Tag apagada: { $nome }
nucleo-lancamentos-historico-anexo-adicionado = Anexo adicionado: { $descricao }
nucleo-lancamentos-historico-anexo-apagado = Anexo apagado: { $descricao }

## Lançamentos

nucleo-lancamentos-erro-nenhum-selecionado = nenhum lançamento selecionado
nucleo-lancamentos-erro-escolha-um = escolha ao menos um lançamento
nucleo-lancamentos-erro-informe-lancamento = informe o lançamento
nucleo-lancamentos-erro-tags-lote = informe as tags a pôr ou a tirar
nucleo-lancamentos-erro-tipo-invalido = tipo de lançamento inválido: { $valor }
nucleo-lancamentos-erro-status-invalido = status inválido: { $valor }
nucleo-lancamentos-erro-valor-zero = o valor do lançamento não pode ser zero
nucleo-lancamentos-erro-categoria-arquivada = a categoria { $nome } está arquivada
nucleo-lancamentos-erro-tipo-sem-categoria = lançamentos do tipo { $tipo } não usam categoria
nucleo-lancamentos-erro-categoria-de-outro-tipo = a categoria { $nome } é de { $categoria ->
    [receita] receita
   *[despesa] despesa
} e o lançamento é uma { $lancamento ->
    [receita] receita
   *[despesa] despesa
}
nucleo-lancamentos-erro-tipo-sem-divisao = lançamentos do tipo { $tipo } não podem ser divididos
nucleo-lancamentos-erro-lote-sem-categoria = aporte, pagamento de fatura e ajuste não usam categoria: são movimentação do patrimônio e ficam fora do fluxo de caixa e do orçamento ({ $n } marcado(s))
nucleo-lancamentos-erro-divisao-partes = uma divisão precisa de pelo menos duas partes
nucleo-lancamentos-erro-divisao-soma = a soma das partes ({ $soma }) difere do total ({ $total })
nucleo-lancamentos-erro-divisao-sinal = as partes da divisão devem ter o mesmo sinal do total e não podem ser zero
nucleo-lancamentos-erro-divisao-desatualizada = { $erro }; envie a divisão atualizada junto (ou uma lista vazia para remover a divisão)
nucleo-lancamentos-erro-pagamento-duas-pernas = pagamentos de fatura têm duas pernas; use transactions.create_transfer
nucleo-lancamentos-erro-conciliado = este lançamento está conciliado; desconcilie (mude o status para efetivado) antes de alterar valor, data ou conta
nucleo-lancamentos-erro-outra-perna-conciliada-previsto = a outra perna da transferência está conciliada; desconcilie-a antes de voltar para previsto
nucleo-lancamentos-erro-outra-perna-conciliada = a outra perna da transferência está conciliada; desconcilie-a antes de alterar valor ou data
nucleo-lancamentos-erro-apagar-conciliado = o lançamento "{ $descricao }" de { $data } está conciliado; desconcilie antes de apagar
nucleo-lancamentos-erro-conta-outra-moeda = a conta escolhida usa { $moeda_conta } e o lançamento está em { $moeda }; crie um lançamento novo nessa conta
nucleo-lancamentos-erro-pagamento-destino-cartao = o destino de um pagamento de fatura precisa ser uma conta de cartão
nucleo-lancamentos-erro-perna-mudar-tipo = não é possível mudar o tipo de uma perna de transferência; apague a transferência e crie outro lançamento
nucleo-lancamentos-erro-perna-sinal = numa transferência a perna de origem é negativa e a de destino é positiva; mantenha o sinal
nucleo-lancamentos-erro-pernas-mesma-conta = as duas pernas da transferência não podem ficar na mesma conta
nucleo-lancamentos-erro-transferencia-dividida = transferências não podem ser divididas
nucleo-lancamentos-erro-virar-pagamento-fatura = para transformar em pagamento de fatura, ligue-o à perna do cartão (transferências sugeridas)
nucleo-lancamentos-erro-ja-ligado = este lançamento já está ligado a outra conta
nucleo-lancamentos-erro-recusar-so-interface = recusar sugestões de transferência só pela interface do app

## Tags e anexos

nucleo-lancamentos-erro-tag-repetida = já existe uma tag chamada { $nome }
nucleo-lancamentos-erro-base64 = conteúdo base64 inválido
nucleo-lancamentos-erro-arquivo-nome = informe o nome do arquivo
nucleo-lancamentos-erro-arquivo-nome-longo = nome de arquivo longo demais
nucleo-lancamentos-erro-arquivo-tipo = tipo de arquivo inválido: { $tipo }
nucleo-lancamentos-erro-anexo-grande = o anexo passa do limite de { $mb } MB
nucleo-lancamentos-erro-arquivo-vazio = o arquivo está vazio

## Transferências

# Descrições padrão de uma transferência nova (ficam gravadas como a descrição do lançamento).
nucleo-lancamentos-descricao-pagamento-fatura = Pagamento da fatura { $conta }
nucleo-lancamentos-descricao-pagamento-recebido = Pagamento recebido de { $conta }
nucleo-lancamentos-descricao-aporte-em = Aporte em { $conta }
nucleo-lancamentos-descricao-aporte-vindo = Aporte vindo de { $conta }
nucleo-lancamentos-descricao-transferencia-para = Transferência para { $conta }
nucleo-lancamentos-descricao-transferencia-de = Transferência de { $conta }
nucleo-lancamentos-erro-transferencia-mesma-conta = a conta de origem e a de destino devem ser diferentes
nucleo-lancamentos-erro-transferencia-valor = informe o valor da transferência como um número positivo
nucleo-lancamentos-erro-transferencia-mesma-moeda = contas na mesma moeda: o valor que entra deve ser igual ao que sai
nucleo-lancamentos-erro-transferencia-outra-moeda = a origem está em { $moeda_origem } e o destino em { $moeda_destino }; informe o valor que entra no destino (to_amount)
nucleo-lancamentos-erro-pagamento-so-cartao = pagamento de fatura só pode ter uma conta de cartão como destino
nucleo-lancamentos-erro-pagamento-de-cartao = a fatura deve ser paga a partir de uma conta que não seja cartão
nucleo-lancamentos-erro-transferencia-tipo = transferências não podem ser do tipo { $tipo }
# Por que um par aparece nas transferências sugeridas (juntados com "; ").
nucleo-lancamentos-motivo-mesmo-valor-mesmo-dia = mesmo valor no mesmo dia
nucleo-lancamentos-motivo-mesmo-valor-dias = mesmo valor com { $dias } dia(s) de diferença
nucleo-lancamentos-motivo-texto-transferencia = texto de transferência (PIX, TED…)
nucleo-lancamentos-motivo-nome-outra-conta = o nome da outra conta aparece na descrição
nucleo-lancamentos-erro-par-ja-transferencia = um dos lançamentos já faz parte de uma transferência
nucleo-lancamentos-erro-par-tipo = só receitas, despesas e transferências soltas viram transferência
nucleo-lancamentos-erro-par-mesma-conta = as duas pernas precisam estar em contas diferentes
nucleo-lancamentos-erro-par-sinal = a perna de origem precisa ser uma saída e a de destino uma entrada
nucleo-lancamentos-erro-par-valor = contas na mesma moeda: o valor que sai precisa ser igual ao que entra
nucleo-lancamentos-erro-par-dividido = lançamentos divididos não viram transferência
nucleo-lancamentos-erro-outra-conta = escolha uma conta diferente da do lançamento
nucleo-lancamentos-erro-outra-conta-moeda = a outra conta usa { $moeda }: informe quanto entrou (ou saiu) nela
nucleo-lancamentos-erro-nao-ligado = este lançamento não está ligado a outra conta
nucleo-lancamentos-erro-ligacao-historico = esta ligação ({ $tipo }) não se desfaz por aqui; use o histórico para desfazer
nucleo-lancamentos-erro-nenhum-par = nenhum par selecionado

## Vínculo com operações (carteira, cotas capital, metas)

nucleo-lancamentos-operacao = operação
nucleo-lancamentos-operacao-apagada = operação apagada
nucleo-lancamentos-operacao-aporte = Aporte em { $ativo } (compra de { $quantidade } em { $data })
nucleo-lancamentos-operacao-resgate = Resgate em { $ativo } (venda de { $quantidade } em { $data })
nucleo-lancamentos-operacao-cotas-capital = Cotas capital · { $conta }
nucleo-lancamentos-operacao-meta = Meta { $nome }
nucleo-lancamentos-ref-operacao-carteira = operação da carteira
nucleo-lancamentos-ref-cotas-capital = cotas capital nessa conta
nucleo-lancamentos-ref-meta = meta
# Por que a operação aparece como candidata.
nucleo-lancamentos-motivo-operacao = { $valor ->
    [igual] mesmo valor
   *[parecido] valor parecido
} { $dias ->
    [0] no mesmo dia
   *[other] a { $dias } dia(s)
}; { $diferenca } de diferença
nucleo-lancamentos-motivo-texto-cota-capital = o texto do extrato fala em cota capital
nucleo-lancamentos-motivo-conta-cotas-capital = a conta tem cotas capital
nucleo-lancamentos-motivo-texto-meta = o texto do extrato cita a meta
nucleo-lancamentos-motivo-poupanca-programada = o texto fala em poupança programada
nucleo-lancamentos-motivo-meta-ativa = meta ativa
nucleo-lancamentos-motivo-combina-com = { $motivo }; combina com { $n } de { $total }
nucleo-lancamentos-erro-vinculado = o lançamento está vinculado a { $operacao }; desvincule antes
nucleo-lancamentos-erro-ja-vinculado = o lançamento já está vinculado a { $operacao }; desvincule antes
nucleo-lancamentos-erro-sem-valor = lançamento sem valor
nucleo-lancamentos-erro-vincular-previsto = o lançamento ainda está previsto; efetive-o antes de vincular
nucleo-lancamentos-erro-vincular-transferencia = o lançamento é uma perna de transferência; desligue a transferência antes
nucleo-lancamentos-erro-vincular-tipo = só receitas, despesas, transferências soltas e aportes se vinculam a uma operação
nucleo-lancamentos-erro-vincular-dividido = lançamentos divididos não se vinculam a uma operação
nucleo-lancamentos-erro-vincular-parcela = o lançamento paga uma parcela de { $divida }; desligue de lá antes
nucleo-lancamentos-erro-vincular-caixa-corretora = o lançamento já é o caixa dessa operação na corretora
nucleo-lancamentos-erro-operacao-ocupada = essa operação já tem um lançamento vinculado; desvincule o outro antes
nucleo-lancamentos-erro-aporte-saida = um aporte (compra) é pago por uma saída (valor negativo)
nucleo-lancamentos-erro-resgate-entrada = um resgate (venda) chega como uma entrada (valor positivo)
nucleo-lancamentos-erro-meta-arquivada = a meta está arquivada
nucleo-lancamentos-erro-vinculo-nao-gravado = vínculo não gravado
nucleo-lancamentos-erro-nao-vinculado = o lançamento não está vinculado a uma operação
nucleo-lancamentos-erro-operacao-um-lancamento = a operação da carteira aceita um lançamento só
nucleo-lancamentos-erro-nenhum-vinculavel = nenhum dos lançamentos pode ser vinculado a { $operacao } ({ $motivo })

## Recorrências

nucleo-lancamentos-ref-recorrencia = recorrência { $id }
nucleo-lancamentos-historico-recorrencia-criada = Recorrência criada: { $descricao }
nucleo-lancamentos-historico-recorrencia-alterada = Recorrência alterada: { $descricao }
nucleo-lancamentos-historico-recorrencia-apagada = Recorrência apagada: { $descricao }
nucleo-lancamentos-historico-previstos = Lançamentos previstos de recorrências
nucleo-lancamentos-historico-parcelas-efetivadas = Parcelas do cartão efetivadas
# Quem fez a alteração no histórico ("Sistema · recorrências").
nucleo-lancamentos-autor-recorrencias = recorrências
nucleo-lancamentos-autor-cartoes = cartões
nucleo-lancamentos-recorrencia-erro-horizonte = o horizonte deve ficar entre 0 e { $max } dias
nucleo-lancamentos-recorrencia-erro-valor-zero = o valor da recorrência não pode ser zero
nucleo-lancamentos-recorrencia-erro-destino-so-transferencia = conta de destino só vale para transferências recorrentes
nucleo-lancamentos-recorrencia-erro-informe-destino = informe a conta de destino da transferência recorrente
nucleo-lancamentos-recorrencia-erro-moedas = transferências recorrentes entre moedas diferentes ({ $origem } → { $destino }) não são suportadas
nucleo-lancamentos-recorrencia-erro-pagamento-cartao = pagamento de fatura recorrente precisa de um cartão como destino
nucleo-lancamentos-recorrencia-erro-transferencia-categoria = transferências não usam categoria
nucleo-lancamentos-recorrencia-erro-ajuste = ajustes não podem ser recorrentes
nucleo-lancamentos-recorrencia-erro-descricao = informe a descrição da recorrência
nucleo-lancamentos-recorrencia-erro-fim-antes = a data final é anterior à data inicial
nucleo-lancamentos-recorrencia-erro-sem-destino = transferência recorrente sem conta de destino

## Regra de repetição (RRULE)

nucleo-lancamentos-rrule-erro-byday-posicao = BYDAY com posição ({ $valor }) não é suportado; use só o dia da semana (ex.: MO)
nucleo-lancamentos-rrule-erro-dia-semana = dia da semana inválido: { $valor }
nucleo-lancamentos-rrule-erro-valor = valor inválido em { $parte }: { $valor }
nucleo-lancamentos-rrule-erro-parte = parte inválida na regra: { $parte }
nucleo-lancamentos-rrule-erro-frequencia = frequência não suportada: { $valor } (use DAILY, WEEKLY, MONTHLY ou YEARLY)
nucleo-lancamentos-rrule-erro-varios-dias = BYMONTHDAY com vários dias não é suportado; crie uma recorrência por dia
nucleo-lancamentos-rrule-erro-varios-meses = BYMONTH com vários meses não é suportado; crie uma recorrência por mês
nucleo-lancamentos-rrule-erro-parte-nao-suportada = parte da regra não suportada: { $parte } (o fim da recorrência é a data final)
nucleo-lancamentos-rrule-erro-sem-freq = a regra precisa de FREQ
nucleo-lancamentos-rrule-erro-intervalo = o intervalo deve ficar entre 1 e { $max }
nucleo-lancamentos-rrule-erro-ocorrencias = o número de ocorrências deve ficar entre 1 e { $max }
nucleo-lancamentos-rrule-erro-dia-mes = o dia do mês deve ficar entre 1 e 31
nucleo-lancamentos-rrule-erro-dia-mes-frequencia = o dia do mês só vale para recorrências mensais ou anuais
nucleo-lancamentos-rrule-erro-mes = o mês deve ficar entre 1 e 12
nucleo-lancamentos-rrule-erro-mes-frequencia = o mês só vale para recorrências anuais
nucleo-lancamentos-rrule-erro-sem-dias = escolha pelo menos um dia da semana
nucleo-lancamentos-rrule-erro-dias-frequencia = dias da semana só valem para recorrências diárias ou semanais
# Descrição legível: "todo mês no dia 5", "a cada 2 semanas às segundas, 3 vezes". Os dias no
# plural; o gênero (f: segundas a sextas, m: sábados e domingos) escolhe "às"/"aos".
nucleo-lancamentos-rrule-dia-mo = segundas
nucleo-lancamentos-rrule-dia-tu = terças
nucleo-lancamentos-rrule-dia-we = quartas
nucleo-lancamentos-rrule-dia-th = quintas
nucleo-lancamentos-rrule-dia-fr = sextas
nucleo-lancamentos-rrule-dia-sa = sábados
nucleo-lancamentos-rrule-dia-su = domingos
nucleo-lancamentos-rrule-lista = { $inicio } e { $ultimo }
nucleo-lancamentos-rrule-dia-com-artigo = { $genero ->
    [f] às { $dia }
   *[m] aos { $dia }
}
nucleo-lancamentos-rrule-dias = { $genero ->
    [f] às { $lista }
    [m] aos { $lista }
   *[misto] { $lista }
}
nucleo-lancamentos-rrule-diaria = { $n ->
    [1] todo dia
   *[other] a cada { $n } dias
}
nucleo-lancamentos-rrule-semanal = { $n ->
    [1] toda semana
   *[other] a cada { $n } semanas
}
nucleo-lancamentos-rrule-mensal = { $n ->
    [1] todo mês
   *[other] a cada { $n } meses
}
nucleo-lancamentos-rrule-anual = { $n ->
    [1] todo ano
   *[other] a cada { $n } anos
}
nucleo-lancamentos-rrule-dia-util = todo dia útil (segunda a sexta)
nucleo-lancamentos-rrule-so-dias = { $base }, só { $dias }
nucleo-lancamentos-rrule-nos-dias = { $base } { $dias }
nucleo-lancamentos-rrule-ultimo-dia = { $base } no último dia
nucleo-lancamentos-rrule-no-dia = { $base } no dia { $dia }
nucleo-lancamentos-rrule-em-data = { $dia ->
    [1] { $base } em 1º de { $mes }
   *[other] { $base } em { $dia } de { $mes }
}
nucleo-lancamentos-rrule-em-mes = { $base } em { $mes }
nucleo-lancamentos-rrule-ultimo-dia-mes = { $base } no último dia do mês
nucleo-lancamentos-rrule-vezes = { $n ->
    [1] { $regra }, uma vez
   *[other] { $regra }, { $n } vezes
}
nucleo-lancamentos-rrule-ate = { $regra }, até { $data }

## Regras de categorização

nucleo-lancamentos-ref-regra = regra { $id }
nucleo-lancamentos-historico-regra-criada = Regra criada: { $nome }
nucleo-lancamentos-historico-regra-alterada = Regra alterada: { $nome }
nucleo-lancamentos-historico-regra-excluida = Regra excluída: { $nome }
nucleo-lancamentos-historico-regras-aplicadas = Aplicação de regras de categorização
nucleo-lancamentos-historico-regras-prontas = Regras prontas de categoria (iFood, Netflix, Uber…)
nucleo-lancamentos-historico-regras-prontas-pix = PIX, TED e DOC passam a ser transferências
nucleo-lancamentos-historico-regras-prontas-restaurantes = Regras prontas: restaurantes e créditos de vale-alimentação e refeição
nucleo-lancamentos-historico-regras-prontas-encargos = Regras prontas: juros e encargos do cartão
nucleo-lancamentos-autor-regras-prontas = regras prontas
# Nome da regra sugerida a partir de um lançamento (vira o nome da regra, se gravada).
nucleo-lancamentos-regra-nome-sugerido = Descrição contém “{ $trecho }”
nucleo-lancamentos-regra-erro-texto = a condição precisa de um texto
nucleo-lancamentos-regra-erro-centavos = valor da condição deve ser inteiro, em centavos (ex.: -5000 = { $exemplo })
nucleo-lancamentos-regra-erro-numero = a condição de valor precisa de um número
nucleo-lancamentos-regra-erro-regex = expressão regular inválida: { $erro }
nucleo-lancamentos-regra-erro-texto-vazio = o texto da condição precisa ter letras ou números
nucleo-lancamentos-regra-erro-op-texto = condições de texto usam contém, igual, começa com, termina com ou regex
nucleo-lancamentos-regra-erro-entre = a condição "entre" precisa de value2
nucleo-lancamentos-regra-erro-op-valor = condições de valor usam igual, entre, maior ou igual, menor ou igual
nucleo-lancamentos-regra-erro-op-conta = condição de conta usa "igual" com o id da conta
nucleo-lancamentos-regra-erro-op-tipo = condição de tipo usa "igual" com o tipo do lançamento
nucleo-lancamentos-regra-erro-sem-condicao = a regra precisa de pelo menos uma condição
nucleo-lancamentos-regra-erro-nome = dê um nome à regra
nucleo-lancamentos-regra-erro-sem-acao = a regra precisa de pelo menos uma ação (categoria, membro, tags, renomear ou tipo)
nucleo-lancamentos-regra-erro-renomear = o novo nome não pode ficar vazio
nucleo-lancamentos-regra-erro-tags = tags não podem ficar vazias
nucleo-lancamentos-regra-erro-sem-palavras = a descrição do lançamento não tem palavras para montar uma regra

## Categorias

nucleo-lancamentos-ref-categoria = categoria { $id }
nucleo-lancamentos-ref-categoria-pai = categoria-pai { $id }
nucleo-lancamentos-historico-categorias-traduzidas = Categorias padrão traduzidas
nucleo-lancamentos-historico-categorias-sugeridas = Categorias sugeridas adicionadas
nucleo-lancamentos-historico-categoria-criada = Categoria criada: { $nome }
nucleo-lancamentos-historico-categoria-alterada = Categoria alterada
nucleo-lancamentos-historico-categoria-apagada = Categoria apagada: { $nome }
nucleo-lancamentos-historico-categoria-apagada-reatribuida = Categoria apagada com reatribuição: { $nome }
nucleo-lancamentos-historico-categorias-juntadas = Categorias juntadas: { $origem } → { $destino }
nucleo-lancamentos-historico-categorias-grupo = { $n ->
        [1] 1 categoria passou para o grupo { $grupo }
       *[other] { $n } categorias passaram para o grupo { $grupo }
    }
nucleo-lancamentos-categoria-erro-repetida = já existe uma categoria chamada { $nome } neste nível
nucleo-lancamentos-categoria-erro-tipo-pai = a subcategoria deve ser do mesmo tipo da categoria-pai ({ $pai } é de { $tipo ->
    [receita] receita
   *[despesa] despesa
})
nucleo-lancamentos-categoria-erro-ciclo = uma categoria não pode ficar dentro dela mesma nem de uma subcategoria sua
nucleo-lancamentos-categoria-erro-grupo-receita = categorias de receita não têm grupo de gasto (essenciais, dívidas, futuro, não essencial)
nucleo-lancamentos-categoria-erro-mesmo-destino = escolha uma categoria diferente para receber os lançamentos
nucleo-lancamentos-categoria-erro-juntar-tipos = { $origem } é de { $tipo_origem ->
    [receita] receita
   *[despesa] despesa
} e { $destino } é de { $tipo_destino ->
    [receita] receita
   *[despesa] despesa
}; só é possível juntar categorias do mesmo tipo
nucleo-lancamentos-categoria-erro-juntar-sub = { $destino } é subcategoria de { $origem }; escolha uma categoria fora dela
nucleo-lancamentos-categoria-erro-tem-subcategorias = { $nome } tem { $n } subcategoria(s); apague, mova ou junte as subcategorias antes
nucleo-lancamentos-categoria-erro-em-uso = { $nome } está em uso ({ $lancamentos } lançamento(s), { $divisoes } divisão(ões), { $orcamento } linha(s) de orçamento, { $regras } regra(s), { $recorrencias } recorrência(s), { $parcelamentos } parcelamento(s)); escolha uma categoria para receber esse uso
nucleo-lancamentos-erro-anexo-so-interface = salvar anexo no disco só pela interface do app
nucleo-lancamentos-erro-copias-so-interface = apagar cópias entre contas só pela interface do app
nucleo-lancamentos-erro-origem-mesma-moeda = a moeda de origem é a mesma da conta; deixe o valor de origem vazio
nucleo-lancamentos-erro-origem-valor = o valor na moeda de origem não pode ser zero
nucleo-lancamentos-erro-origem-par = informe o valor e a moeda de origem juntos
