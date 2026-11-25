# Carteira e bens (núcleo, `portfolio` e `physical`): erros, motivos do histórico, descrições
# dos lançamentos que o app cria, avisos e notas das posições, proventos, custo das taxas, risco,
# rebalanceamento, extrato do ativo, cadastro em lote do agente, proventos digitados às pressas e
# a fórmula do valor dos bens (o "Explicar" do patrimônio). Erros em minúsculas, como frase de
# log (a interface põe a inicial maiúscula). pt-BR é o idioma de origem.

## Campos (entram em "<campo> inválido: <valor>")

nucleo-carteira-erro-campo-invalido = { $campo } inválido: { $valor }
nucleo-carteira-campo-quantidade = quantidade
nucleo-carteira-campo-preco = preço
nucleo-carteira-campo-cambio = câmbio
nucleo-carteira-campo-fator = fator
nucleo-carteira-campo-custo = custo
nucleo-carteira-campo-alvo = alvo
nucleo-carteira-campo-tolerancia = tolerância
nucleo-carteira-campo-preco-alvo = preço-alvo
nucleo-carteira-campo-alerta = alerta
nucleo-carteira-campo-valor-cota = valor por cota

## Tipos de provento

nucleo-carteira-provento-dividendos = Dividendos
nucleo-carteira-provento-jcp = JCP
nucleo-carteira-provento-fii = Rendimentos de FII
nucleo-carteira-provento-cupom = Cupom
nucleo-carteira-provento-juros = Juros
nucleo-carteira-provento-aluguel = Aluguel de ações
nucleo-carteira-provento-amortizacao = Amortização
nucleo-carteira-provento-outros = Outros proventos
# Descrição do lançamento de um provento ("Dividendos PETR4").
nucleo-carteira-provento-lancamento = { $tipo } { $ativo }

## Operações, eventos corporativos, proventos, alvos e watchlist (`records`)

nucleo-carteira-operacao = operação { $id }
# Descrição do lançamento de uma operação ("Compra 10 PETR4 @ R$ 38,50").
nucleo-carteira-operacao-compra = Compra { $quantidade } { $ativo } @ { $preco }
nucleo-carteira-operacao-venda = Venda { $quantidade } { $ativo } @ { $preco }
nucleo-carteira-erro-moedas = a conta é em { $conta } e o ativo em { $ativo }: informe o câmbio ou desligue a movimentação de caixa
nucleo-carteira-erro-quantidade = a quantidade deve ser positiva
nucleo-carteira-erro-preco = o preço não pode ser negativo
nucleo-carteira-erro-custos = custos e impostos não podem ser negativos
nucleo-carteira-erro-cambio = o câmbio deve ser positivo
nucleo-carteira-erro-venda-maior = venda de { $quantidade } maior que a posição de { $posicao } nesta conta na data
nucleo-carteira-erro-ajuste-com-caixa = um ajuste de posição não movimenta o caixa: tire o lançamento de caixa da operação (cash_movement falso)
nucleo-carteira-erro-ajuste-importada = esta operação veio da corretora com identificador próprio: é uma operação de verdade, não um ajuste de posição
nucleo-carteira-erro-desdobramento = desdobramento precisa de fator maior que 1 (ex.: 2 para 1→2)
nucleo-carteira-erro-grupamento = grupamento precisa de fator entre 0 e 1 (ex.: 0,1 para 10→1)
nucleo-carteira-erro-bonificacao = bonificação precisa do fator (ações novas por ação, ex.: 0,1 = 10%)
nucleo-carteira-erro-devolucao = devolução de capital precisa do valor total
nucleo-carteira-erro-ativo-resultante = informe o ativo resultante
nucleo-carteira-erro-bruto = o valor bruto do provento deve ser positivo
nucleo-carteira-erro-imposto-retido = imposto retido inválido
nucleo-carteira-erro-alvos-escopo = todos os alvos devem ter o mesmo escopo
nucleo-carteira-erro-alvo = o alvo deve estar entre 0% e 100%
nucleo-carteira-erro-alvos-soma = a soma dos alvos por classe passa de 100%
nucleo-carteira-erro-alvos-soma-classe = a soma dos alvos dos ativos de { $classe } passa de 100% da classe
nucleo-carteira-watchlist-acima = cotação acima de { $valor }
nucleo-carteira-watchlist-abaixo = cotação abaixo de { $valor }

## Comandos (`portfolio.*`): motivos do histórico e respostas

nucleo-carteira-erro-nada-selecionado = nenhum registro selecionado
nucleo-carteira-nada-novo = Nada novo: tudo já estava cadastrado ou foi pulado (veja o relatório).
nucleo-carteira-nenhuma-linha-pronta = Nenhuma linha pronta: todas já registradas ou com erro (veja o relatório).
nucleo-carteira-motivo-operacao-registrada = Operação registrada
nucleo-carteira-motivo-operacao-alterada = Operação alterada
nucleo-carteira-motivo-operacoes-removidas = { $n ->
        [1] Operação removida
       *[other] { $n } operações removidas
    }
nucleo-carteira-motivo-evento-registrado = Evento corporativo registrado
nucleo-carteira-motivo-eventos-removidos = { $n ->
        [1] Evento corporativo removido
       *[other] { $n } eventos corporativos removidos
    }
nucleo-carteira-motivo-provento-registrado = Provento registrado
nucleo-carteira-erro-sem-proventos = nenhum provento para registrar
nucleo-carteira-erro-proventos-demais = no máximo 500 proventos por vez
nucleo-carteira-motivo-proventos-confirmados =
    { $n ->
        [one] { $n } provento confirmado
       *[other] { $n } proventos confirmados
    }
nucleo-carteira-motivo-proventos-removidos = { $n ->
        [1] Provento removido
       *[other] { $n } proventos removidos
    }
nucleo-carteira-motivo-come-cotas = Regra do come-cotas
nucleo-carteira-motivo-alvos = Alocação-alvo definida
nucleo-carteira-motivo-watchlist = Ativo na watchlist
nucleo-carteira-motivo-watchlist-removido = Ativo removido da watchlist

## Cadastro em lote do agente (`bulk`)

nucleo-carteira-erro-lote-fonte = diga de onde vieram os dados (`source`)
nucleo-carteira-erro-lote-vazio = nada para cadastrar: informe ativos, operações, posições ou proventos
nucleo-carteira-erro-lote-itens = no máximo { $maximo } itens por chamada; divida em partes
nucleo-carteira-erro-lote-conta = operações, posições e proventos precisam da conta (`account_id` ou `account`)
nucleo-carteira-erro-conta-cartao = a conta dos investimentos não pode ser cartão de crédito
# Motivo do histórico: "Investimentos em lote (2 ativo(s), 5 operação(ões)): extrato da XP".
nucleo-carteira-motivo-lote = Investimentos em lote ({ $itens }): { $fonte }
nucleo-carteira-lote-ativos =
    { $n ->
        [one] { $n } ativo
       *[other] { $n } ativos
    }
nucleo-carteira-lote-operacoes = { $n } operação(ões)
nucleo-carteira-lote-posicoes = { $n } posição(ões)
nucleo-carteira-lote-proventos =
    { $n ->
        [one] { $n } provento
       *[other] { $n } proventos
    }
nucleo-carteira-lote-conta = conta
# A conta do relatório do lote.
nucleo-carteira-lote-conta-existente-id = existente { $id }
nucleo-carteira-lote-conta-existente = existente: { $nome }
nucleo-carteira-lote-conta-nova = nova: { $nome }
# Itens pulados do relatório ("operação 3: já registrada").
nucleo-carteira-lote-ativo-erro = ativo { $n } ({ $ativo }): { $erro }
nucleo-carteira-lote-operacao-sem-ativo = operação { $n }: ativo { $ativo } não cadastrado (inclua em `assets`)
nucleo-carteira-lote-operacao-repetida = operação { $n }: já registrada
nucleo-carteira-lote-operacao-erro = operação { $n }: { $erro }
nucleo-carteira-lote-posicao-sem-ativo = posição { $n }: ativo { $ativo } não cadastrado (inclua em `assets`)
nucleo-carteira-lote-posicao-quantidade = posição { $n }: quantidade precisa ser positiva
nucleo-carteira-lote-posicao-erro = posição { $n }: { $erro }
nucleo-carteira-lote-posicao-confere = posição { $n } ({ $ativo }): já confere
nucleo-carteira-lote-posicao-maior =
    { $registrado ->
        [one] posição { $n } ({ $ativo }): já há { $registrado } registrado, mais que { $informado }; registre a venda com `record_trade` (`side: sell`; `cash_movement: false` se o dinheiro já entrou na conta)
       *[other] posição { $n } ({ $ativo }): já há { $registrado } registrados, mais que { $informado }; registre a venda com `record_trade` (`side: sell`; `cash_movement: false` se o dinheiro já entrou na conta)
    }
nucleo-carteira-lote-posicao-repetida = posição { $n }: já registrada
nucleo-carteira-lote-provento-sem-ativo = provento { $n }: ativo { $ativo } não cadastrado (inclua em `assets`)
nucleo-carteira-lote-provento-repetido = provento { $n }: já registrado
nucleo-carteira-lote-provento-erro = provento { $n }: { $erro }
# Observação gravada nas operações do lote. O começo de cada uma (antes do primeiro argumento)
# identifica a operação como do agente no extrato do ativo, em qualquer idioma.
nucleo-carteira-nota-lote = Lote do agente: { $fonte }
nucleo-carteira-nota-posicao = Posição informada em { $data } ({ $fonte }): ajuste de { $diferenca } ao preço médio

## Proventos digitados às pressas (`quick_income`)

nucleo-carteira-rapido-erro-ticker = informe o ticker
nucleo-carteira-rapido-erro-valor = o valor deve ser positivo
nucleo-carteira-rapido-erro-ativo = { $ticker } não está cadastrado na carteira
nucleo-carteira-rapido-erro-conta = conta { $conta } não encontrada
nucleo-carteira-rapido-erro-sem-operacoes = nenhuma operação de { $ticker } registrada: escolha a conta
nucleo-carteira-rapido-erro-sem-posicao = sem posição de { $ticker } em { $data }: escolha a conta
nucleo-carteira-rapido-ja-registrado = já registrado
nucleo-carteira-rapido-repetido = repete uma linha acima
nucleo-carteira-rapido-erro-sem-linhas = nenhuma linha de provento
nucleo-carteira-rapido-erro-linhas = no máximo { $maximo } linhas por vez (recebidas { $recebidas })
nucleo-carteira-rapido-nota-sem-posicao = sem posição de { $ticker } na data
nucleo-carteira-rapido-nota-maior-posicao = ativo em mais de uma conta: escolhida a de maior posição
nucleo-carteira-rapido-nota-conta = conta pela posição que explica o valor: { $conta }
# "Dividendos de 0,5 por ação (B3, data-com 10/10/2026, pago em 20/10/2026)".
nucleo-carteira-rapido-nota-anunciado = { $tipo } de { $valor } por ação ({ $fonte }, pago em { $pagamento })
nucleo-carteira-rapido-nota-anunciado-com = { $tipo } de { $valor } por ação ({ $fonte }, data-com { $datacom }, pago em { $pagamento })
nucleo-carteira-rapido-nota-confere = confere: { $quantidade } × { $valor } = { $total }
nucleo-carteira-rapido-nota-diferente = valor diferente do esperado ({ $esperado } bruto para { $quantidade } cotas): confira
nucleo-carteira-rapido-nota-bruto = valor digitado tratado como bruto (antes do IR)
nucleo-carteira-rapido-nota-sem-anuncio = nenhum provento anunciado perto de { $data }: tipo pela classe ({ $tipo })
nucleo-carteira-rapido-nota-ir = IR retido presumido: { $imposto } (bruto { $bruto })
nucleo-carteira-rapido-nota-futuro = pagamento futuro: fica como anunciado, sem caixa
# Motivo do histórico: "3 provento(s) registrados: PETR4, VALE3, ITUB4".
nucleo-carteira-rapido-motivo =
    { $n ->
        [one] { $n } provento registrado: { $lista }
       *[other] { $n } proventos registrados: { $lista }
    }
nucleo-carteira-rapido-e-mais = { $lista } e mais { $n }

## Proventos anunciados a conferir (`pending`)

nucleo-carteira-pendente-data-agenda = data de pagamento da agenda (a fonte oficial ainda não informou)
nucleo-carteira-pendente-valor-divergente = { $fonte } dizia { $valor } por cota; usado o oficial ({ $b3 })
nucleo-carteira-pendente-data-divergente = { $fonte } dizia pagamento em { $data }; usado o oficial ({ $b3 })
nucleo-carteira-pendente-tipos-divergentes = as fontes dão tipos diferentes ({ $lista }); usado o do { $fonte }
nucleo-carteira-pendente-fontes-discordam = sem o provento nas fontes oficiais e as fontes discordam ({ $lista }); usado o do { $fonte }: confira no extrato
nucleo-carteira-pendente-so-uma-fonte = só no { $fonte }: sem aviso oficial deste provento (ainda não baixado ou fora da lista oficial)

## Posições, desempenho e rebalanceamento (`engine`)

nucleo-carteira-erro-cadeia-ticker = cadeia de mudanças de ticker longa demais (ou circular)
nucleo-carteira-aviso-sem-cambio = sem câmbio { $moeda }/{ $base } em { $data }
nucleo-carteira-aviso-venda-maior = venda maior que a posição em { $data }
nucleo-carteira-aviso-cisao = cisão/subscrição: registre a operação resultante manualmente
nucleo-carteira-aviso-sem-cotacao = sem cotação: avaliado pelo custo
nucleo-carteira-aviso-aportes-sem-cota = { $valor } em aportes de previdência ainda sem cota: avaliados pelo valor aportado
nucleo-carteira-aviso-cotacao-parada = sem cotação nova desde { $data } e sem negócios: o ativo pode ter sido deslistado ou incorporado; confira antes de lançar
nucleo-carteira-erro-periodo-longo = período longo demais
# Nome da série de comparação fora do Brasil, quando o país não tem nome para ela.
nucleo-carteira-juro-referencia = Juro de referência
nucleo-carteira-rebalanceamento-aviso = Análise baseada nos seus dados e nos alvos que você definiu; não é recomendação de investimento.

## Come-cotas (`come_cotas`)

nucleo-carteira-come-cotas-so-fundos = só fundos abertos têm come-cotas
nucleo-carteira-come-cotas-escolhida = regra escolhida por você
nucleo-carteira-come-cotas-cvm-isento = classificação da CVM: { $classe } (sem come-cotas)
nucleo-carteira-come-cotas-cvm-longo = classificação da CVM: { $classe }; longo prazo (15%) até você indicar curto prazo
nucleo-carteira-come-cotas-sem-classificacao = fundo sem classificação baixada da CVM: longo prazo (15%); ajuste se for de ações ou curto prazo
nucleo-carteira-come-cotas-sem-cota = sem cota em { $data }: come-cotas dessa data não aplicado
nucleo-carteira-come-cotas-erro-valor = valor inválido no come-cotas: { $valor }

## Custo das taxas (`fee_cost`)

nucleo-carteira-custo-erro-parametros = retorno real entre −50% e 50%, taxa alternativa entre 0 e 20% e excesso entre 0 e 50%
nucleo-carteira-custo-erro-horizontes = horizontes de 1 a 60 anos
nucleo-carteira-custo-fonte-previdencia = cadastro do plano de previdência
nucleo-carteira-custo-fonte-cvm = extrato do fundo na CVM
nucleo-carteira-custo-fonte-cvm-data = extrato do fundo na CVM ({ $data })
nucleo-carteira-custo-nota-retorno = Valores reais de hoje, com retorno de { $retorno } a.a. acima da inflação antes das taxas.
nucleo-carteira-custo-nota-alternativa = Cenário alternativo: taxa de administração de { $taxa } a.a. (ou a de hoje, se já for menor), sem performance nem carregamento.
nucleo-carteira-custo-nota-performance = Taxa de performance sem custo projetado: só pesa se o fundo superar o benchmark (informe o excesso esperado).

## Risco da carteira (`risk`)

nucleo-carteira-risco-erro-meses = analise de 12 a 120 meses
nucleo-carteira-risco-nota-retornos = Retornos mensais de { $de } a { $ate } (TWR, com proventos, em { $moeda }). A carteira e as classes usam os pesos de hoje; num mês em que um ativo não existia, o peso dele vai para os outros.
nucleo-carteira-risco-nota-pouco-historico = Menos de { $minimo } meses de histórico (sem métricas próprias nem correlação): { $ativos }.
nucleo-carteira-risco-nota-sem-cdi = Sem a série do CDI no período: o Sharpe fica de fora (Ajustes › Índices).
nucleo-carteira-risco-nota-estimados = Há preços estimados ou faltando no período: os números são aproximados.

## Extrato do ativo (`statement`)

nucleo-carteira-extrato-desdobramento = Desdobramento { $proporcao }
nucleo-carteira-extrato-grupamento = Grupamento { $proporcao }
nucleo-carteira-extrato-bonificacao-pct = Bonificação de { $pct }%
nucleo-carteira-extrato-bonificacao = Bonificação
nucleo-carteira-extrato-subscricao = Subscrição
nucleo-carteira-extrato-incorporacao = Incorporação
nucleo-carteira-extrato-troca-ticker = Troca de ticker
nucleo-carteira-extrato-cisao = Cisão
nucleo-carteira-extrato-devolucao = Devolução de capital
nucleo-carteira-extrato-herdada = Posição herdada (troca de ticker ou incorporação)
nucleo-carteira-extrato-entrada-sem-custo = Entrada sem custo (desdobramento ou bonificação lançado como compra)
nucleo-carteira-extrato-compra = Compra
nucleo-carteira-extrato-venda = Venda
nucleo-carteira-extrato-ajuste = Ajuste de posição
nucleo-carteira-extrato-come-cotas =
    { $cotas ->
        [one] Come-cotas: { $cotas } cota resgatada para o IR
       *[other] Come-cotas: { $cotas } cotas resgatadas para o IR
    }
# De onde veio a linha do extrato.
nucleo-carteira-extrato-origem-nota = nota { $numero }
nucleo-carteira-extrato-origem-agente = agente
nucleo-carteira-extrato-origem-importacao = importação
nucleo-carteira-extrato-origem-manual = manual
nucleo-carteira-extrato-origem-sistema = sistema
nucleo-carteira-extrato-origem-yahoo = coleta do Yahoo
nucleo-carteira-extrato-origem-coleta = coleta de cotações
nucleo-carteira-extrato-origem-b3 = importação B3
nucleo-carteira-extrato-aviso-desdobramento-duplo = possível desdobramento contado duas vezes: há uma entrada a custo zero e um evento corporativo a poucos dias; apague um dos dois
nucleo-carteira-extrato-aviso-bonificacao-dupla = possível bonificação contada duas vezes: há um evento corporativo e uma compra com a mesma quantidade de ações novas a poucos dias; apague um dos dois

## Bens (`physical`): cadastro, benfeitorias, avaliações e financiamento

nucleo-carteira-bem = bem { $id }
nucleo-carteira-bem-benfeitoria = benfeitoria { $id }
nucleo-carteira-bem-avaliacao = avaliação { $id }
nucleo-carteira-bem-ligacao = ligação do contrato com o bem
nucleo-carteira-bem-campo-taxa-depreciacao = taxa de depreciação
nucleo-carteira-bem-campo-valor-residual = valor residual
nucleo-carteira-bem-campo-taxa = taxa
nucleo-carteira-bem-campo-residual = residual
nucleo-carteira-bem-erro-fora-intervalo = { $campo } fora do intervalo
nucleo-carteira-bem-erro-nome = dê um nome ao bem
nucleo-carteira-bem-erro-preco = o preço de compra não pode ser negativo
nucleo-carteira-bem-erro-vida-util = vida útil entre 1 e 100 anos
nucleo-carteira-bem-erro-fipe-dados = para usar a tabela FIPE informe tipo, marca, modelo e ano do veículo
nucleo-carteira-bem-erro-fipe-ref = informe tipo, marca, modelo e ano do veículo na tabela FIPE
nucleo-carteira-bem-erro-venda-antes = a venda não pode ser antes da compra
nucleo-carteira-bem-erro-venda-negativa = valores da venda não podem ser negativos
nucleo-carteira-bem-erro-custos = custos de aquisição não podem ser negativos
nucleo-carteira-bem-erro-benfeitoria = descreva a benfeitoria ou a obra
nucleo-carteira-bem-erro-valor-obra = o valor da obra deve ser positivo
nucleo-carteira-bem-erro-valor-projeto = informe o valor (estimado) da obra ou ligue lançamentos ao projeto
nucleo-carteira-bem-erro-avaliacao = o valor da avaliação não pode ser negativo
nucleo-carteira-bem-erro-contrato-outro = este contrato já financia outro bem ({ $bem }); desligue-o de lá primeiro
nucleo-carteira-bem-motivo-cadastrado = Bem cadastrado: { $nome }
nucleo-carteira-bem-motivo-alterado = Bem alterado
nucleo-carteira-bem-motivo-removido = Bem removido
nucleo-carteira-bem-motivo-benfeitoria = Benfeitoria registrada: { $descricao }
nucleo-carteira-bem-motivo-benfeitoria-alterada = Benfeitoria alterada
nucleo-carteira-bem-motivo-benfeitoria-removida = Benfeitoria removida
nucleo-carteira-bem-motivo-avaliacao = Avaliação registrada
nucleo-carteira-bem-motivo-avaliacao-removida = Avaliação removida
nucleo-carteira-bem-motivo-financiamento-ligado = Financiamento ligado ao bem
nucleo-carteira-bem-motivo-financiamento-desligado = Financiamento desligado do bem
nucleo-carteira-bem-motivo-venda = Venda do bem registrada

## Bens: tabela FIPE (`physical::fipe`)

nucleo-carteira-fipe-erro-codigo = código FIPE inválido: { $codigo }
nucleo-carteira-fipe-erro-ilegivel = resposta da tabela FIPE ilegível
nucleo-carteira-fipe-erro-sem-modelos = resposta da tabela FIPE sem modelos
nucleo-carteira-fipe-erro-inesperada = resposta da tabela FIPE inesperada
nucleo-carteira-fipe-erro-sem-codigo = opção da tabela FIPE sem código
nucleo-carteira-fipe-erro = tabela FIPE: { $erro }
nucleo-carteira-fipe-erro-sem-preco = resposta da tabela FIPE sem preço
nucleo-carteira-fipe-erro-rede = não foi possível consultar a tabela FIPE ({ $erro }); tente de novo mais tarde ou registre a avaliação manualmente
nucleo-carteira-fipe-erro-codigo-diferente = o código FIPE do veículo ({ $esperado }) não confere com o da consulta ({ $consulta })
nucleo-carteira-fipe-motivo = Cotação FIPE de { $mes }

## Bens: valor numa data e a fórmula do "Explicar" do patrimônio (`physical::valuation`)

# Fonte de uma avaliação registrada.
nucleo-carteira-bem-fonte-manual = estimativa manual
nucleo-carteira-bem-fonte-avaliacao = avaliação
nucleo-carteira-bem-fonte-laudo = laudo
nucleo-carteira-bem-fonte-fipe = FIPE
nucleo-carteira-bem-fonte-indice = índice
# Avisos do índice de correção (o mês vem como MM/AAAA).
nucleo-carteira-bem-indice-sem-dados = sem dados do índice: sincronize os índices
nucleo-carteira-bem-indice-publicado-ate = índice publicado até { $mes }; o período seguinte fica sem correção
nucleo-carteira-bem-indice-desde = índice disponível desde { $mes }
nucleo-carteira-bem-indice-faltando = série do índice com meses faltando: valor sem correção
nucleo-carteira-bem-vida-encerrada = vida útil já encerrada na avaliação
# Pedaços da fórmula: "avaliação de 10/01/2025 (laudo): R$ 500.000,00 corrigido pelo IPCA de … = R$ 520.000,00".
nucleo-carteira-bem-formula-indice = corrigido pelo { $indice } de { $de } a { $ate } (fator { $fator })
nucleo-carteira-bem-formula-depreciacao = depreciado a { $taxa }% a.a. por { $anos } anos (fator { $fator })
nucleo-carteira-bem-formula-linear = depreciado linearmente em { $anos } anos até { $residual }% (fator { $fator })
nucleo-carteira-bem-formula-comprado = comprado só em { $data }
nucleo-carteira-bem-formula-vendido = vendido em { $data } por { $valor } (líquido)
nucleo-carteira-bem-formula-avaliacao = avaliação de { $data } ({ $fonte }): { $valor }
nucleo-carteira-bem-formula-custo = custo de aquisição de { $valor } em { $data }
nucleo-carteira-bem-benfeitorias = { $n ->
        [1] 1 benfeitoria
       *[other] { $n } benfeitorias
    }
nucleo-carteira-bem-formula-benfeitorias-custo = { $benfeitorias } pelo custo ({ $valor })
nucleo-carteira-bem-formula-benfeitorias-regra = { $benfeitorias } de { $custo } pela mesma regra ({ $valor })
nucleo-carteira-bem-formula-sem-avaliacao = sem avaliação nem estimativa automática: { $formula }; registre uma avaliação para o valor de mercado
# Fonte curta do valor.
nucleo-carteira-bem-fonte-sem-estimativa = sem estimativa automática
nucleo-carteira-bem-fonte-indice-bcb = { $indice } (BCB SGS)
nucleo-carteira-bem-fonte-fipe-depreciacao = última consulta FIPE + depreciação de { $taxa }% a.a.
nucleo-carteira-bem-fonte-depreciacao = depreciação de { $taxa }% a.a.
nucleo-carteira-bem-fonte-linear = depreciação linear em { $anos } anos
nucleo-carteira-bem-fonte-sem-posse = sem posse
nucleo-carteira-bem-fonte-venda = venda
nucleo-carteira-bem-fonte-avaliacao-de = avaliação · { $fonte }
nucleo-carteira-bem-fonte-custo = custo de aquisição
nucleo-carteira-bem-fonte-avaliacao-regra = avaliação · { $fonte } + { $regra }

## Calculadoras de renda fixa (portfolio.compare_fixed_income e simulate_fixed_income_sale)
nucleo-rf-erro-valor = informe o valor aplicado
nucleo-rf-erro-ofertas = informe de 1 a { $max } ofertas
nucleo-rf-erro-prazo = informe um prazo em meses ou uma data de resgate depois da aplicação
nucleo-rf-erro-taxa = taxa inválida
nucleo-rf-erro-datas-venda = a venda precisa ficar entre a aplicação e o vencimento
nucleo-rf-tipo-cdb = CDB
nucleo-rf-tipo-rdb = RDB
nucleo-rf-tipo-lc = LC
nucleo-rf-tipo-lci = LCI
nucleo-rf-tipo-lca = LCA
nucleo-rf-tipo-cri = CRI
nucleo-rf-tipo-cra = CRA
nucleo-rf-tipo-debenture = Debênture
nucleo-rf-tipo-debenture-incentivada = Debênture incentivada
nucleo-rf-tipo-tesouro-selic = Tesouro Selic
nucleo-rf-tipo-tesouro-prefixado = Tesouro Prefixado
nucleo-rf-tipo-tesouro-ipca = Tesouro IPCA+
nucleo-rf-tipo-tesouro-renda = Tesouro Renda+
nucleo-rf-tipo-poupanca = Poupança
nucleo-rf-tipo-outro = Renda fixa
nucleo-rf-taxa-cdi-pct = { $taxa }% do CDI
nucleo-rf-taxa-cdi-mais = CDI + { $taxa }%
nucleo-rf-taxa-selic-mais = Selic + { $taxa }%
nucleo-rf-taxa-pre = { $taxa }% a.a.
nucleo-rf-taxa-ipca-mais = IPCA + { $taxa }%
nucleo-rf-nota-iof = Resgate antes de 30 dias: paga IOF sobre o rendimento.
nucleo-rf-nota-aniversario = A poupança só rende no aniversário: o resgate antes dele perde o mês corrente.
nucleo-rf-nota-custodia = Inclui a taxa de custódia da B3 (0,20% a.a.; no Tesouro Selic, só acima de R$ 10 mil).
nucleo-rf-nota-venda = Vender antes do vencimento recebe o preço de mercado do dia, que pode ficar acima ou abaixo da curva.
nucleo-rf-nota-venda-ipca = O IPCA até a venda e até o vencimento é a premissa informada, não o índice publicado.
nucleo-rf-fonte-ir = IR regressivo (22,5% a 15%) e isenção de LCI, LCA, CRI, CRA e poupança: Lei 11.033/2004; debêntures incentivadas: Lei 12.431/2011.
nucleo-rf-fonte-iof = IOF regressivo sobre o rendimento até o 29º dia: Decreto 6.306/2007.
nucleo-rf-fonte-custodia = Custódia do Tesouro Direto: 0,20% a.a. da B3; Tesouro Selic sem taxa até R$ 10 mil.
nucleo-rf-fonte-poupanca = Poupança: 0,5% a.m. + TR com a Selic acima de 8,5% a.a.; senão, 70% da Selic + TR (Lei 8.177/1991).
nucleo-rf-aviso = Estimativa com as premissas mostradas, não promessa de rendimento nem recomendação de investimento.
nucleo-carteira-erro-ponte-periodo = o fim do período precisa vir depois do início

## Comparar com qualquer série (`portfolio.benchmark_series`)

nucleo-carteira-comparar-serie-desconhecida = série de índice desconhecida
nucleo-carteira-comparar-sem-juro = o país de referência não tem juro de referência no catálogo
nucleo-carteira-comparar-cdi-mais = { $indice } + { $taxa } ao ano

nucleo-carteira-motivo-metodo-custo = Método de custo das vendas

## Aportes programados

nucleo-carteira-plano = aporte programado { $id }
nucleo-carteira-plano-erro-nome = informe o nome do aporte programado
nucleo-carteira-plano-erro-valor = o valor do aporte precisa ser positivo
nucleo-carteira-plano-erro-dia = o dia do aporte precisa ficar entre 1 e 28
nucleo-carteira-plano-erro-fim = o fim do aporte programado não pode vir antes do início
nucleo-carteira-plano-erro-data = essa data não é um aporte pendente deste plano
nucleo-carteira-plano-nota = Aporte programado: { $plano }
nucleo-carteira-motivo-plano-criado = Aporte programado criado
nucleo-carteira-motivo-plano-alterado = Aporte programado alterado
nucleo-carteira-motivo-plano-removido = Aporte programado removido
nucleo-carteira-motivo-aporte-confirmado = Aporte programado confirmado
nucleo-carteira-motivo-aporte-pulado = Aporte programado pulado

## Etiquetas de ativos e concentração (V161)
nucleo-ativos-etiqueta-invalida = a etiqueta precisa ter de 1 a 40 caracteres (no máximo 20 por ativo)
nucleo-ativos-motivo-etiquetas = Etiquetas de ativos
nucleo-carteira-limite-invalido = o limite precisa ficar entre 0 e 100% da carteira
nucleo-carteira-limite-sem-ativo = informe o ativo do limite
nucleo-carteira-motivo-limite = Limite de concentração

## Income ledger (portfolio/income_ledger.rs)
nucleo-carteira-razao-nota-creditado = valor creditado pela B3 no extrato importado: ele vale sobre o cálculo
nucleo-carteira-razao-nota-pagamento-estimado = nenhuma fonte informou a data de pagamento: usada a data-com
nucleo-carteira-razao-nota-repete = repete o provento pago em { $data }
nucleo-carteira-razao-nota-data-improvavel = data de pagamento muito no futuro: confira se o ano foi digitado certo
nucleo-carteira-razao-nota-sem-fonte = nenhuma fonte descreve este provento: confira no extrato da corretora
nucleo-carteira-razao-nota-sem-caixa = lançado como recebido, sem o crédito na conta: confira no extrato da corretora e lance o crédito
nucleo-carteira-razao-nota-historico-sem-caixa = histórico: lançado sem movimentar o caixa da corretora
nucleo-carteira-razao-erro-caixa-ja-lancado = este provento já tem o crédito na conta
nucleo-carteira-razao-erro-caixa-anunciado = provento ainda não recebido: não há crédito para lançar
nucleo-carteira-razao-motivo-caixa = Crédito de { $n } provento(s) lançado na conta
nucleo-carteira-razao-motivo-reconstruir =
    { $n ->
        [one] Proventos reconstruídos pelos avisos oficiais ({ $n } correção)
       *[other] Proventos reconstruídos pelos avisos oficiais ({ $n } correções)
    }
nucleo-carteira-razao-motivo-automatico =
    { $n ->
        [one] Provento oficial lançado automaticamente ({ $n })
       *[other] Proventos oficiais lançados automaticamente ({ $n })
    }
nucleo-carteira-razao-erro-so-usuario = Só você pode ligar ou desligar o lançamento automático dos proventos.
