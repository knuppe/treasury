# Contas, cartões e câmbio (núcleo: `accounts`, `cards`, `fx`): erros de validação, motivos do
# histórico, dicas da sincronia de saldo e textos que o núcleo grava ao criar ajustes, pagamentos
# de fatura e operações de câmbio, no idioma da instalação. pt-BR é o idioma de origem.

## Contas

nucleo-contas-ref-conta = conta { $id }
nucleo-contas-ref-conta-pagadora = conta pagadora { $id }
nucleo-contas-ref-instituicao = instituição { $id }
nucleo-contas-historico-criada = Conta criada: { $nome }
nucleo-contas-historico-alterada = Conta alterada: { $nome }
nucleo-contas-historico-apagada = Conta apagada: { $nome }
nucleo-contas-historico-conciliacao = Conciliação da conta { $nome } até { $data }
nucleo-contas-erro-tipo-invalido = tipo de conta inválido: { $valor }
nucleo-contas-erro-tipo-so-brasil = Esse tipo de conta só existe no Brasil: o cofre precisa estar configurado para o Brasil.
nucleo-contas-erro-historico-longo = período longo demais para pontos diários (máximo { $max } dias); use interval = month
nucleo-contas-erro-agencia = agência: use só números e hífen
nucleo-contas-erro-numero-conta = número da conta: use só números e hífen
nucleo-contas-erro-cheque-especial-tipo = cheque especial só vale para conta corrente
nucleo-contas-erro-apagar-lancamentos =
    { $n ->
        [one] a conta { $nome } tem { $n } lançamento; apague-o antes ou arquive a conta
       *[other] a conta { $nome } tem { $n } lançamentos; apague-os antes ou arquive a conta
    }
nucleo-contas-erro-apagar-recorrencias =
    { $n ->
        [one] a conta { $nome } é usada por { $n } recorrência; apague-a antes
       *[other] a conta { $nome } é usada por { $n } recorrências; apague-as antes
    }
nucleo-contas-erro-apagar-parcelamentos =
    { $n ->
        [one] a conta { $nome } tem { $n } parcelamento; apague-o antes
       *[other] a conta { $nome } tem { $n } parcelamentos; apague-os antes
    }
# Encerramento da conta (accounts/closing.rs).
nucleo-contas-historico-encerrada = Conta encerrada: { $nome }
nucleo-contas-historico-reaberta = Conta reaberta: { $nome }
nucleo-contas-erro-encerrada = A conta { $nome } foi encerrada em { $data }. Para lançar depois dessa data, reabra a conta.
nucleo-contas-encerrar-erro-futuro = A data de encerramento não pode ser depois de hoje.
nucleo-contas-encerrar-erro-antes-abertura = A data de encerramento é antes da abertura da conta ({ $data }).
nucleo-contas-encerrar-erro-ja-encerrada = A conta já foi encerrada em { $data }.
nucleo-contas-encerrar-erro-saldo = A conta ainda tem saldo de { $valor } na data. Transfira o saldo ou lance o ajuste antes de encerrar.
nucleo-contas-encerrar-erro-depois =
    { $n ->
        [one] Há { $n } lançamento depois da data (o último em { $data }). Escolha uma data a partir dele.
       *[other] Há { $n } lançamentos depois da data (o último em { $data }). Escolha uma data a partir dele.
    }
nucleo-contas-encerrar-erro-posicoes = A conta ainda tem posição em { $ativos }. Registre a venda ou a transferência de custódia antes.
nucleo-contas-encerrar-erro-parcelas =
    { $n ->
        [one] O cartão ainda tem { $n } parcela a vencer.
       *[other] O cartão ainda tem { $n } parcelas a vencer.
    }
# Por que o OFX não achou a conta sozinho.
nucleo-contas-ofx-sem-numero = o OFX não traz o número da conta
nucleo-contas-ofx-sem-cartao = nenhum cartão com esse final; preencha o final do cartão no cadastro
nucleo-contas-ofx-sem-conta = nenhuma conta com esse número; preencha banco, agência e conta no cadastro
nucleo-contas-ofx-varias = mais de uma conta combina; escolha qual

## Cartão de crédito

nucleo-contas-cartao-erro-fechamento = o dia de fechamento deve ficar entre 1 e 31
nucleo-contas-cartao-erro-vencimento = o dia de vencimento deve ficar entre 1 e 31
nucleo-contas-cartao-erro-limite = o limite do cartão não pode ser negativo
nucleo-contas-cartao-erro-paga-propria = o cartão não pode pagar a própria fatura
nucleo-contas-cartao-erro-pagadora-cartao = a conta que paga a fatura não pode ser outro cartão
nucleo-contas-cartao-erro-final = informe os 4 últimos dígitos do cartão (só números)
nucleo-contas-cartao-erro-dados = informe os dados do cartão (dia de fechamento e de vencimento)
nucleo-contas-cartao-erro-so-cartao = dados de cartão só valem para contas do tipo cartão de crédito
nucleo-contas-cartao-erro-nao-cartao = a conta escolhida não é um cartão de crédito
nucleo-contas-cartao-erro-sem-ciclo = o cartão não tem dia de fechamento e vencimento

## Faturas e parcelamentos

nucleo-contas-ref-fatura = fatura { $id }
nucleo-contas-ref-parcelamento = parcelamento { $id }
nucleo-contas-historico-pagamento-fatura = Pagamento da fatura { $cartao } com vencimento em { $vencimento }
nucleo-contas-historico-total-informado = Total informado da fatura
nucleo-contas-historico-movidos-fatura =
    { $n ->
        [one] { $n } lançamento movido de fatura
       *[other] { $n } lançamentos movidos de fatura
    }
nucleo-contas-historico-compra-parcelada = Compra parcelada em { $n }x: { $descricao }
nucleo-contas-historico-parcelamento-apagado = Parcelamento apagado: { $descricao }
# Descrição do pagamento de fatura (fica gravada no lançamento); $mes no formato 10/2026.
nucleo-contas-fatura-descricao-pagamento = Pagamento da fatura { $cartao } ({ $mes })
nucleo-contas-fatura-erro-pagadora = informe a conta que paga a fatura
nucleo-contas-fatura-erro-moeda = a conta pagadora está em { $moeda_conta } e o cartão em { $moeda_cartao }; use uma conta na moeda do cartão
nucleo-contas-fatura-erro-valor-positivo = o valor do pagamento deve ser positivo
nucleo-contas-fatura-erro-nada-a-pagar = esta fatura não tem valor a pagar
nucleo-contas-fatura-erro-destino = informe a fatura de destino (statement_id, ou account_id e reference_month)
nucleo-contas-fatura-erro-outro-cartao = o lançamento "{ $descricao }" não é deste cartão
nucleo-contas-parcelas-erro-numero = o número de parcelas deve ficar entre 1 e 120
nucleo-contas-parcelas-erro-total = informe o valor total da compra (positivo)
nucleo-contas-parcelas-erro-menor = o valor total é menor que o número de parcelas
nucleo-contas-parcelas-erro-descricao = informe a descrição da compra
nucleo-contas-parcelas-erro-conciliada = a parcela "{ $descricao }" de { $data } está conciliada; desconcilie antes de apagar o parcelamento

## Cheque especial

nucleo-contas-cheque-erro-limite = o limite do cheque especial não pode ser negativo
nucleo-contas-cheque-erro-indexador = a taxa ao ano e a taxa ao mês trazem indexadores diferentes
nucleo-contas-cheque-erro-numero = { $campo }: número inválido ({ $valor })
nucleo-contas-cheque-erro-negativo = { $campo } não pode ser negativo
nucleo-contas-cheque-erro-acima = { $campo } acima de { $max }%: confira
nucleo-contas-cheque-campo-taxa-mes = taxa ao mês
nucleo-contas-cheque-campo-desconto = desconto
nucleo-contas-cheque-campo-taxa-ano = taxa ao ano
nucleo-contas-cheque-campo-iof-dia = IOF básico ao dia
nucleo-contas-cheque-campo-iof-adicional = IOF adicional
nucleo-contas-cheque-campo-cet-mes = CET ao mês
nucleo-contas-cheque-campo-cet-ano = CET ao ano

## Cotas capital

nucleo-contas-ref-saldo-cotas = saldo das cotas
nucleo-contas-historico-cotas-capital = Cotas capital: { $nome }
nucleo-contas-historico-cotas-removidas = Cotas capital removidas
nucleo-contas-historico-cotas-saldo = Saldo das cotas capital
nucleo-contas-historico-cotas-aportes = Aportes em cotas capital fora das despesas
nucleo-contas-historico-cotas-saldo-removido = Saldo das cotas capital removido
nucleo-contas-cotas-erro-tipo-conta = cotas capital ficam numa conta corrente (ou poupança) da cooperativa
nucleo-contas-cotas-erro-ative = ative as cotas capital nesta conta antes
nucleo-contas-cotas-erro-saldo-negativo = o saldo das cotas não pode ser negativo

## Sincronia de saldo

nucleo-contas-ref-sincronia = sincronia de saldo { $id }
nucleo-contas-historico-sincronia = Sincronia de saldo: { $nome } em { $data }
nucleo-contas-historico-sincronia-motivo = Motivo da diferença de saldo
# Descrição e nota do ajuste (ficam gravadas no lançamento).
nucleo-contas-sincronia-descricao = Sincronia de saldo
nucleo-contas-sincronia-nota = Saldo no banco em { $data }: { $banco }; o Treasury calculava { $calculado }.
nucleo-contas-sincronia-erro-futuro = a data do saldo não pode estar no futuro
nucleo-contas-sincronia-erro-motivo = escreva o motivo da diferença
# Causas prováveis da diferença.
nucleo-contas-sincronia-dica-mesmo-valor =
    { $n ->
        [one] { $n } lançamento com o valor exato da diferença ({ $valor }): pode ser um lançado a mais (ou de menos) no Treasury.
       *[other] { $n } lançamentos com o valor exato da diferença ({ $valor }): pode ser um lançado a mais (ou de menos) no Treasury.
    }
nucleo-contas-sincronia-dica-previsto = Há previsto com o valor da diferença ainda não efetivado: talvez já tenha acontecido no banco.
nucleo-contas-sincronia-dica-sinal = Lançamento de { $valor } com o sinal trocado explicaria a diferença.
nucleo-contas-sincronia-dica-duplicados =
    { $n ->
        [one] { $n } lançamento parece duplicado (mesma data, valor e descrição).
       *[other] { $n } lançamentos parecem duplicados (mesma data, valor e descrição).
    }
nucleo-contas-sincronia-dica-conferir =
    { $n ->
        [one] Nenhuma causa óbvia: confira o lançamento do período com o extrato (tarifas, juros e estornos costumam faltar).
       *[other] Nenhuma causa óbvia: confira os { $n } lançamentos do período com o extrato (tarifas, juros e estornos costumam faltar).
    }

## Câmbio

nucleo-contas-ref-taxa-cambio = taxa de câmbio
nucleo-contas-historico-cambio = Câmbio { $origem } → { $destino } em { $data }
nucleo-contas-historico-cambio-manual = Câmbio informado manualmente
nucleo-contas-historico-cambio-removido = Câmbio removido
# Descrições e nota dos lançamentos da operação (ficam gravadas).
nucleo-contas-cambio-descricao = Câmbio { $origem } → { $destino }
nucleo-contas-cambio-descricao-iof = IOF do câmbio
nucleo-contas-cambio-descricao-tarifa = Tarifa do câmbio
nucleo-contas-cambio-nota-iof = Operação de câmbio { $origem } → { $destino } ({ $pct }%).
nucleo-contas-cambio-erro-numero = { $campo } inválido: { $valor }
nucleo-contas-cambio-campo-referencia = taxa de referência
nucleo-contas-cambio-campo-spread = spread
nucleo-contas-cambio-campo-taxa = taxa
nucleo-contas-cambio-campo-iof = IOF
nucleo-contas-cambio-campo-ptax = PTAX
nucleo-contas-cambio-erro-grande = valor grande demais para o câmbio: confira o valor e a taxa
nucleo-contas-cambio-erro-mesma-moeda = câmbio é entre moedas diferentes; para a mesma moeda use transferência
nucleo-contas-cambio-erro-sem-taxa = informe a taxa aplicada ou a de referência (no Brasil, a PTAX)
nucleo-contas-cambio-erro-taxa-positiva = a taxa de câmbio deve ser positiva
nucleo-contas-cambio-erro-valor = informe o valor que sai ou o valor que entra (positivo)
nucleo-contas-cambio-erro-ptax = resposta da PTAX inválida: { $erro }
nucleo-contas-cambio-erro-referencia-brl = a referência é de uma moeda estrangeira na moeda da família
nucleo-contas-cambio-sem-referencia = sem cotação de referência de { $moeda }: informe a taxa à mão
nucleo-contas-cambio-erro-taxa = taxa inválida: { $valor }
nucleo-contas-cambio-erro-intervalo = valor convertido fora do intervalo
nucleo-contas-cambio-erro-moedas-iguais = as moedas devem ser diferentes
# Boletins da PTAX do Banco Central (o nome que o BCB dá a cada um).
nucleo-contas-ptax-abertura = Abertura
nucleo-contas-ptax-intermediario = Intermediário
nucleo-contas-ptax-fechamento = Fechamento

nucleo-contas-sincronia-copias-removidas = a diferença vinha de lançamentos copiados de outra conta, já apagados

## Plásticos do cartão (titular, adicionais, virtuais)
nucleo-contas-ref-plastico = cartão { $id }
nucleo-contas-plastico-final = final { $final }
nucleo-contas-plastico-tipo-titular = cartão do titular
nucleo-contas-plastico-tipo-adicional = cartão adicional
nucleo-contas-plastico-tipo-virtual = cartão virtual
nucleo-contas-plastico-tipo-temporario = cartão temporário
nucleo-contas-plastico-erro-tipo = tipo de cartão desconhecido (titular, adicional, virtual ou temporário)
nucleo-contas-plastico-erro-numero-inteiro = guarde só os 4 últimos dígitos do cartão, nunca o número inteiro
nucleo-contas-plastico-erro-sublimite = o sublimite não pode ser negativo
nucleo-contas-plastico-erro-final-repetido = este cartão já tem um plástico com final { $final }
nucleo-contas-plastico-erro-outro-cartao = o plástico é de outro cartão
nucleo-contas-historico-plastico-criado = Cartão adicional em { $cartao }
nucleo-contas-historico-plastico-alterado = Alteração no { $plastico }
nucleo-contas-historico-plastico-apagado = Remoção do { $plastico }

## Limite, datas da fatura e anuidade
nucleo-contas-ref-limite = registro de limite
nucleo-contas-ref-anuidade = anuidade do cartão { $id }
nucleo-contas-fatura-erro-datas = o vencimento não pode ser antes do fechamento
nucleo-contas-fatura-erro-fechamento = o fechamento não pode ser antes do início das compras da fatura
nucleo-contas-fatura-erro-valor-negativo = os valores informados da fatura não podem ser negativos
nucleo-contas-anuidade-erro-valor = o valor da anuidade não pode ser negativo
nucleo-contas-anuidade-erro-parcelas = a anuidade é cobrada em 1 a 12 parcelas
nucleo-contas-anuidade-erro-mes = o mês de aniversário vai de 1 a 12
nucleo-contas-anuidade-erro-isencao = informe o valor que isenta a anuidade (gasto na fatura ou valor investido)
nucleo-contas-anuidade-erro-desconto = o desconto é um número entre 0 e 1 (0,5 = 50%)
nucleo-contas-historico-fatura-alterada = Fatura alterada
nucleo-contas-historico-limite = Limite do cartão
nucleo-contas-historico-anuidade = Anuidade do cartão
nucleo-contas-historico-anuidade-fim = Cartão sem anuidade
nucleo-contas-recompensa-erro-taxa = a taxa é um número decimal (0,01 = 1% de cashback; pontos por unidade de moeda)
nucleo-contas-recompensa-erro-teto = o teto por fatura não pode ser negativo
nucleo-contas-recompensa-erro-prazo = o prazo do crédito vai de 0 a 180 dias
nucleo-contas-historico-recompensa = Cashback e pontos do cartão
nucleo-contas-historico-recompensa-fim = Cartão sem programa de recompensa

## Pontos e milhas
nucleo-pontos-ref-programa = programa de pontos { $id }
nucleo-pontos-ref-fatura = fatura do cartão
nucleo-pontos-ref-movimento = movimento de pontos
nucleo-pontos-erro-nome = dê um nome ao programa (ex.: Livelo, Smiles)
nucleo-pontos-erro-valor = valores não podem ser negativos
nucleo-pontos-erro-pontos = informe uma quantidade de pontos diferente de zero
nucleo-pontos-erro-transferencia-par = transferência entre programas é registrada pelos dois lados de uma vez
nucleo-pontos-erro-mesmo-programa = a transferência precisa de dois programas diferentes
nucleo-pontos-historico-programa = Programa de pontos: { $descricao }
nucleo-pontos-historico-programa-alterado = Programa de pontos alterado: { $descricao }
nucleo-pontos-historico-programa-apagado = Programa de pontos apagado: { $descricao }
nucleo-pontos-historico-movimento = Pontos em { $descricao }
nucleo-pontos-historico-transferencia = Transferência de pontos de { $descricao }
nucleo-pontos-historico-movimento-apagado = Movimento de pontos apagado

## Atualização das contas (docs/account-freshness.md)

nucleo-contas-erro-limites-atualizacao = Os dias para "possivelmente incompleto" precisam ser maiores que os de "atualizado há", e os dois maiores que zero.
nucleo-contas-historico-limites-atualizacao = Limites de atualização das contas alterados
