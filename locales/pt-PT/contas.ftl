# Variante pt-PT (ecrã 03 · Contas): o que faltar cai para pt-BR. Revisão: rascunho por IA.
# A fatura do cartão de crédito chama-se aqui "extrato do cartão"; o cheque especial, "descoberto
# autorizado"; as cotas capital, "quotas de capital".

## Rótulos repetidos

contas-faturas = Extratos
contas-parcelamentos = Prestações

## Cabeçalho, resumo e grupos do ecrã

contas-descricao = Saldos medidos (saldo inicial + lançamentos), previstos e conciliados; cartões com extratos e limite; contas noutra moeda convertidas para { $moeda }.
contas-carregando = A carregar as contas
contas-vazio-titulo = Registe a primeira conta
contas-vazio-descricao = Conta à ordem, poupança, dinheiro, cartão de crédito, corretora ou conta no estrangeiro. O saldo vem do saldo inicial mais os lançamentos.
contas-saldo-liquido-fonte = Contas que entram no património (sem investimentos a preço de mercado)
contas-saldo-liquido-estimado = Há uma conta noutra moeda com câmbio de há mais de 7 dias.
contas-cambio-origem = Câmbio (PTAX ou indicado à mão)

## Comandos (as palavras de pesquisa vão separadas por vírgula)

contas-cmd-novo-cartao-busca = extrato,fatura,limite

## Cartão de cada conta

contas-sem-cambio = Sem câmbio { $moeda }/{ $base }: indique uma taxa.
contas-cambio-manual = Câmbio indicado à mão
contas-cambio-nota-velha = 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $base }. A taxa é de há mais de 7 dias.
contas-fatura-saldo-devedor = Extrato e saldo devedor
contas-ciclo-valor = fecha no dia { $fechamento } · vence no dia { $vencimento }

## Menu de contexto da conta

contas-menu-compra-parcelada = Compra em prestações
contas-menu-importar = Importar extrato de conta ou de cartão

## Detalhe da conta

contas-patrimonio = Património
contas-previsto-ate-fim-mes = Previsto até ao fim do mês
contas-cotas-capital = Quotas de capital
contas-secoes-de = Secções de { $nome }

## Conciliação com o extrato

contas-conciliar-explicacao = Pegue no saldo do extrato do banco numa data e compare-o com o calculado (saldo inicial + lançamentos efetuados até essa data). Se bater certo, os lançamentos até ali passam a { $conciliados } e ficam protegidos contra alterações acidentais de valor ou data.
contas-confere = Bate certo ✓
contas-nao-confere = Não bate certo. Procure lançamentos em falta ou duplicados até { $data }: { $link }.
contas-nao-confere-link = ver os efetuados ainda não conciliados

## Sincronia de saldo com o banco

contas-sincronizar-explicacao = Não sabe o motivo? Sincronize com o banco: a diferença passa a um lançamento de { $ajuste } (não conta como receita nem despesa), o saldo passa a bater certo com o extrato e o marco fica guardado para investigar depois.
contas-observacao-exemplo = Ex.: saldo da aplicação do banco às 18h
contas-pedido-agente = Investigue a diferença de saldo da conta { $conta } em { $data } (sincronia { $id }): o banco mostrava { $banco } e o Treasury { $treasury }. Use get_balance_sync_hints e os lançamentos do período para encontrar o motivo e, se o encontrar, registe-o com resolve_balance_sync.
contas-motivo = Motivo, se já o sabe
contas-motivo-exemplo = Ex.: a comissão de setembro não veio no OFX
contas-conferia = Batia certo com o banco

## Câmbio manual da conta

contas-cambio-explicacao = { $base ->
        [BRL] O património em BRL usa a PTAX quando a recolha está ligada
       *[other] O património em { $base } usa o câmbio recolhido quando a recolha está ligada
    }; sem ela, ou para corrigir, indique a taxa à mão. Taxas com mais de 7 dias deixam o valor marcado como estimado (≈).
contas-cambio-salvo = Câmbio guardado
contas-salvar-taxa = Guardar taxa
contas-informada-mao = indicada à mão

## Histórico de saldo

contas-saldo-fonte = Fonte: saldo inicial + lançamentos { $previstos ->
        [sim] efetuados e previstos
       *[nao] efetuados
    } · { $intervalo ->
        [dia] diário
       *[mes] fim de cada mês
    }

## Registo da conta

contas-erro-nome = Indique o nome da conta.
contas-erro-data = Indique a data do saldo inicial.
contas-nome-exemplo = Conta do banco, Revolut, Carteira…
contas-moeda-dica = As contas no estrangeiro usam o câmbio para o património em { $moeda }.
contas-fatura-inicial = Extrato em aberto no início ({ $moeda })
contas-agencia = Balcão
contas-agencia-dica = Sem o dígito, se o banco não o usa.
contas-sem-membros = Registe os membros no ecrã Família.
contas-membro-explicacao = Com um só membro, os lançamentos desta conta ficam logo com ele (pode mudar um a um). Com dois ou mais, ou nenhum, ficam da família.
contas-dia-fechamento = Dia do fecho
contas-vence-mes-seguinte = Vence no mês seguinte ao fecho.
contas-vence-mesmo-mes = Vence no mesmo mês do fecho.
contas-conta-pagadora = Conta que paga o cartão
contas-bandeira = Rede
contas-bandeira-exemplo = Visa, Mastercard, American Express…
contas-entra-patrimonio = Entra no património líquido
contas-entra-patrimonio-dica = Desligue para contas de terceiros que só acompanha.
contas-apagar-nome = Eliminar { $nome }
contas-apagar-descricao = Só é possível eliminar contas sem lançamentos, recorrências ou prestações. Para guardar o histórico, arquive-a.
contas-apagar-conta = Eliminar conta
contas-conta-apagada = Conta eliminada

## Descoberto autorizado

contas-cheque-especial = Descoberto autorizado
contas-tem-cheque-especial = A conta tem descoberto autorizado
contas-cheque-especial-dica = O saldo pode ficar negativo até ao limite; o Treasury mostra quanto está em uso e uma estimativa de juros e IOF.
contas-cheque-custo-juros = · custo estimado em 30 dias: { $valor } (juros de { $taxa }% ao mês + IOF)

## Quotas de capital

contas-cotas-explicacao = Nas cooperativas de crédito, o cooperante tem quotas de capital: uma participação que rende conforme a cooperativa decide (juros ao capital, excedentes). Ative para acompanhar o saldo das quotas, os reforços que saem desta conta e a rentabilidade de cada período (variação do saldo menos os reforços).
contas-cotas-ativar = Ativar quotas de capital nesta conta
contas-cotas-saldo = Saldo das quotas
contas-cotas-aportes-periodo = Reforços no período
contas-cotas-informe-saldo = indique o saldo
contas-cotas-aportes = Reforços
contas-cotas-saldo-moeda = Saldo das quotas ({ $moeda })
contas-cotas-saldo-informado = Saldo das quotas indicado
contas-cotas-informar-saldo = Indicar saldo
contas-cotas-saldos = Saldos das quotas de capital
contas-cotas-aportes-identificados = Reforços identificados
contas-cotas-aportes-identificados-n = Reforços identificados ({ $n })
contas-cotas-ja-fora = Os reforços já estavam fora das despesas
contas-cotas-n-tirados = { $n } reforço(s) retirados das despesas
contas-cotas-tirar = Retirar os reforços das despesas
contas-cotas-como-identificar = Como identificar os reforços
contas-cotas-categoria = Categoria dos reforços
contas-cotas-somar = Somar o saldo das quotas ao património

## Extratos do cartão

contas-pagar-fatura-de = Pagar o extrato de { $mes }
contas-pagar-fatura-descricao = Lança uma transferência do tipo “pagamento do cartão”: não conta como despesa (as compras já contaram). As datas futuras ficam como previstas.
contas-fatura-de = Extrato de { $mes }
contas-total-informado = Total indicado pelo banco
contas-total-informado-salvo = Total indicado guardado
contas-salvar-total-informado = Guardar total indicado
contas-confere-calculado = Bate certo com o calculado ✓
contas-pagar-fatura = Pagar o extrato
contas-lancamentos-fatura = Lançamentos do extrato
contas-fatura = Extrato
contas-confere-curto = bate certo
contas-faturas-vazio-kicker = Sem extratos
contas-faturas-vazio-descricao = Os extratos aparecem sozinhos a partir das compras, pelo dia de fecho e de vencimento.

## Prestações e compromissos futuros do cartão

contas-compromissos-fonte = Fonte: extratos em aberto, com prestações futuras e recorrências previstas
contas-compromissos-grafico = Valor a pagar por mês nos próximos extratos
contas-compras-parceladas = Compras em prestações
contas-sem-parceladas = Nenhuma compra em prestações. Registe uma em Lançamentos › Novo › Em prestações no cartão.
contas-apagar-parcelamento = Eliminar as prestações
contas-apagar-parcelamento-descricao = Todas as prestações (passadas e futuras) saem do cartão. Pode desfazer no histórico.
contas-apagar-parcelas = Eliminar prestações
contas-n-parcelas-apagadas = { $n } prestação(ões) eliminada(s)

## Lista, página da conta e ajuste de saldo

contas-ultimos-lancamentos = Movimentos
contas-sem-lancamentos = Ainda não há movimentos nesta conta.
contas-ver-todos-lancamentos = Ver todos em Movimentos
contas-ajuste-explicacao = Indique o saldo que a conta tinha numa data (o do banco ou do extrato). O Treasury regista a diferença como ajuste: entra no saldo e no histórico da conta, mas fica fora de receitas, despesas e orçamento. Os movimentos até à data ficam conciliados.
contas-ajuste-previa = Pré-visualização do ajuste
