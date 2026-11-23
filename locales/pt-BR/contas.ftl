# Tela 03 · Contas: lista e resumo das contas, cartão de cada conta, cadastro, detalhe (histórico
# de saldo, conciliação, sincronia com o banco, câmbio, cotas capital, cheque especial) e as
# faturas e parcelamentos do cartão (docs/i18n-design.md). pt-BR é o idioma de origem.

## Rótulos repetidos

contas-data = Data
contas-saldo = Saldo
contas-tipo = Tipo
contas-moeda = Moeda
contas-valor = Valor
contas-total = Total
contas-restante = Restante
contas-calculado = Calculado
contas-conciliado = Conciliado
contas-previsto = Previsto
contas-pago = Pago
contas-vence = Vence
contas-mes = Mês
contas-em-moeda = Em { $moeda }
contas-sem-limite = sem limite
contas-cartao = Cartão
contas-faturas = Faturas
contas-parcelamentos = Parcelamentos
contas-historico-saldo = Histórico de saldo
contas-conciliacao = Conciliação
contas-cambio = Câmbio
contas-reativar = Reativar
contas-arquivar = Arquivar
contas-desfazer = Desfazer
contas-limite-moeda = Limite ({ $moeda })

## Cabeçalho, resumo e grupos da tela

contas-descricao = Saldos medidos (saldo inicial + lançamentos), previstos e conciliados; cartões com faturas e limite; contas em outra moeda convertidas para { $moeda }.
contas-membros-familia = Membros da família
contas-novo-cartao = Novo cartão
contas-nova-conta = Nova conta
contas-carregando = Carregando contas
contas-vazio-kicker = Nenhuma conta
contas-vazio-titulo = Cadastre a primeira conta
contas-vazio-descricao = Conta corrente, poupança, dinheiro, cartão de crédito, corretora ou conta no exterior. O saldo vem do saldo inicial mais os lançamentos.
contas-resumo = Resumo das contas
contas-saldo-liquido = Saldo líquido em { $moeda }
contas-saldo-liquido-fonte = Contas que entram no patrimônio (sem investimentos a mercado)
contas-saldo-liquido-estimado = Há conta em outra moeda com câmbio de mais de 7 dias.
contas-sem-cambio-fora = { $n ->
        [1] { $n } conta sem câmbio fora da soma
       *[other] { $n } contas sem câmbio fora da soma
    }
contas-em-contas = Em contas
contas-devido-cartoes = Devido nos cartões
contas-credito-disponivel = Crédito disponível
contas-mostrar-arquivadas = Mostrar arquivadas
contas-convertidas-moeda = convertidas para { $moeda }
contas-cambio-origem = Câmbio (PTAX ou informado à mão)
contas-acoes-de = Ações de { $nome }
contas-conta-reativada = Conta reativada
contas-conta-arquivada = Conta arquivada
contas-erro-alterar = Não foi possível alterar a conta

## Comandos (as palavras de busca vão separadas por vírgula)

contas-cmd-nova-busca = banco,cartão,corretora
contas-cmd-novo-cartao = Novo cartão de crédito
contas-cmd-novo-cartao-busca = fatura,limite
contas-cmd-abrir = Abrir conta: { $nome }
contas-cmd-abrir-busca = saldo

## Cartão de cada conta

contas-sem-cambio = Sem câmbio { $moeda }/{ $base }: informe uma taxa.
contas-cambio-manual = Câmbio informado à mão
contas-cambio-fonte = Câmbio { $fonte }
contas-cambio-nota = 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $base }.
contas-cambio-nota-velha = 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $base }. A taxa é de mais de 7 dias atrás.
contas-abrir = Abrir { $nome }
contas-detalhes = Detalhes →
contas-instituicao = Instituição
contas-membros-rotulo = { $n ->
        [1] Membro
       *[other] Membros
    }
contas-ultimo = Último
contas-fatura-saldo-devedor = Fatura e saldo devedor
contas-saldo-hoje = Saldo hoje
contas-arquivada = arquivada
contas-previsto-fim-mes = Previsto (fim do mês)
contas-disponivel = Disponível
contas-ciclo = Ciclo
contas-ciclo-valor = fecha dia { $fechamento } · vence dia { $vencimento }
contas-limite-comprometido = { $pct } do limite comprometido
contas-abrir-site = Abrir o site de { $nome } no navegador
contas-site = Site ↗

## Menu de contexto da conta

contas-menu-abrir-detalhes = Abrir detalhes
contas-menu-nova-compra = Nova compra neste cartão
contas-menu-novo-lancamento = Novo lançamento nesta conta
contas-menu-compra-parcelada = Compra parcelada
contas-menu-transferir = Transferir desta conta
contas-menu-ver-lancamentos = Ver lançamentos
contas-menu-importar = Importar extrato ou fatura
contas-menu-conciliar = Conciliar com o extrato
contas-menu-cambio = Câmbio { $moeda }/{ $base }
contas-menu-editar = Editar…

## Detalhe da conta

contas-novo-lancamento = Novo lançamento
contas-lancamentos-link = Lançamentos →
contas-fechar-detalhe = Fechar detalhe
contas-saldo-inicial = Saldo inicial
contas-desde = Desde
contas-patrimonio = Patrimônio
contas-patrimonio-entra = entra
contas-patrimonio-fora = fora
contas-saldo-devedor = Saldo devedor
contas-previsto-ate-fim-mes = Previsto até o fim do mês
contas-limite-de = de { $limite }
contas-cotas-capital = Cotas capital
contas-secoes-de = Seções de { $nome }

## Conciliação com o extrato

contas-n-conciliados =
    { $n ->
        [one] { $n } lançamento conciliado
       *[other] { $n } lançamentos conciliados
    }
contas-ate-data = até { $data }
contas-conciliar-explicacao = Pegue o saldo do extrato do banco numa data e compare com o calculado (saldo inicial + lançamentos efetivados até a data). Se bater, os lançamentos até ali viram { $conciliados } e ficam protegidos contra mudança acidental de valor ou data.
contas-conciliados-destaque = conciliados
contas-data-extrato = Data do extrato
contas-saldo-extrato = Saldo no extrato ({ $moeda })
contas-comparar = Comparar
contas-extrato-banco = Extrato do banco
contas-diferenca = Diferença
contas-n-marcados-conciliados =
    { $n ->
        [one] ✓ { $n } lançamento marcado como conciliado.
       *[other] ✓ { $n } lançamentos marcados como conciliados.
    }
contas-conciliacao-desfeita = Conciliação desfeita
contas-confere = Confere ✓
contas-marcar-conciliado = Marcar como conciliado até { $data }
contas-nao-confere = Não confere. Procure lançamentos faltando ou duplicados até { $data }: { $link }.
contas-nao-confere-link = ver os efetivados ainda não conciliados

## Sincronia de saldo com o banco

contas-sincronia-sem-motivo = sem motivo
contas-sincronia-explicada = explicada
contas-sincronia-aceita = aceita
contas-sincronizar-explicacao = Não sabe o motivo? Sincronize com o banco: a diferença vira um lançamento de { $ajuste } (não conta como receita nem despesa), o saldo passa a bater com o extrato e o marco fica guardado para investigar depois.
contas-ajuste-destaque = ajuste
contas-observacao-opcional = Observação (opcional)
contas-observacao-exemplo = Ex.: saldo do app do banco às 18h
contas-saldo-sincronizado = Saldo sincronizado com o banco
contas-ajuste-lancado = Ajuste lançado em { $data }.
contas-sincronizar = Sincronizar com o banco
contas-ajuste-resumo = Ajuste de { $diferenca } em { $data }: o banco mostrava { $banco } e o Treasury { $treasury }.
contas-causas-provaveis = Causas prováveis
contas-ver-lancamentos-periodo = Ver os lançamentos do período
contas-pedir-agente = Pedir ao agente para investigar
contas-pedido-agente = Investigue a diferença de saldo da conta { $conta } em { $data } (sincronia { $id }): o banco mostrava { $banco } e o Treasury { $treasury }. Use get_balance_sync_hints e os lançamentos do período para achar o motivo e, se achar, registre com resolve_balance_sync.
contas-motivo = Motivo, se você já sabe
contas-motivo-exemplo = Ex.: tarifa de setembro não veio no OFX
contas-anotar-motivo = Anotar o motivo
contas-aceitar-sem-investigar = Aceitar sem investigar
contas-sincronias = Sincronias de saldo
contas-conferia = Conferia com o banco
contas-ajuste-de = Ajuste de { $valor }
contas-investigar = Investigar

## Câmbio manual da conta

contas-cambio-explicacao = { $base ->
        [BRL] O patrimônio em BRL usa a PTAX quando a coleta está ligada
       *[other] O patrimônio em { $base } usa o câmbio coletado quando a coleta está ligada
    }; sem ela, ou para corrigir, informe a taxa à mão. Taxas com mais de 7 dias deixam o valor marcado como estimado (≈).
contas-taxa-rotulo = 1 { $moeda } =
contas-taxa-dica = { $base ->
        [BRL] Em reais, ex.: 5,4321
       *[other] Em { $base }
    }
contas-cambio-salvo = Câmbio salvo
contas-cambio-salvo-descricao = { $moeda }/{ $base } em { $data }
contas-salvar-taxa = Salvar taxa
contas-taxas-cambio = Taxas de câmbio
contas-taxa-valor = 1 { $base } = { NUMBER($taxa, maximumFractionDigits: 6) } { $cotacao }
contas-informada-mao = informada à mão
contas-remover = Remover
contas-nenhuma-taxa = Nenhuma taxa { $moeda }/{ $base } ainda.

## Histórico de saldo

contas-periodo-grafico = Período do gráfico
contas-n-meses = { $n } meses
contas-desde-inicio = Desde o início
contas-incluir-previstos = Incluir previstos (próximos 2 meses)
contas-variacao-periodo = Variação no período: { $valor }
contas-saldo-de = Saldo de { $nome }
contas-saldo-fonte = Fonte: saldo inicial + lançamentos { $previstos ->
        [sim] efetivados e previstos
       *[nao] efetivados
    } · { $intervalo ->
        [dia] diário
       *[mes] fim de cada mês
    }
contas-saldo-grafico = Saldo de { $nome } de { $de } a { $ate }
contas-saldo-por-data = Saldo de { $nome } por data
contas-hoje = hoje

## Cadastro da conta

contas-erro-nome = Informe o nome da conta.
contas-erro-data = Informe a data do saldo inicial.
contas-conta-atualizada = Conta atualizada
contas-conta-criada = Conta criada
contas-nova-instituicao-opcao = + Nova instituição…
contas-editar-nome = Editar { $nome }
contas-criar-conta = Criar conta
contas-nome = Nome
contas-nome-exemplo = Conta do banco, Nubank, Carteira…
contas-tipo-fixo = O tipo não muda depois de criada.
contas-moeda-fixa = A moeda não muda depois de criada.
contas-moeda-dica = Contas no exterior usam o câmbio para o patrimônio em { $moeda }.
contas-fatura-inicial = Fatura em aberto no início ({ $moeda })
contas-saldo-inicial-moeda = Saldo inicial ({ $moeda })
contas-na-data = Na data
contas-agencia = Agência
contas-agencia-dica = Sem o dígito, se o banco não usa.
contas-numero-conta = Conta
contas-numero-conta-dica = Com o dígito; reconhece a conta nos extratos OFX.
contas-membro-familia = Membro da família
contas-sem-membros = Cadastre os membros na tela Família.
contas-membro-explicacao = Com um membro só, os lançamentos desta conta já ficam com ele (dá para mudar um a um). Com dois ou mais, ou nenhum, ficam da família.
contas-dia-fechamento = Dia do fechamento
contas-dia-vencimento = Dia do vencimento
contas-vence-mes-seguinte = Vence no mês seguinte ao fechamento.
contas-vence-mesmo-mes = Vence no mesmo mês do fechamento.
contas-conta-pagadora = Conta que paga a fatura
contas-bandeira = Bandeira
contas-bandeira-exemplo = Visa, Mastercard, Elo…
contas-final-cartao = Final do cartão
contas-entra-patrimonio = Entra no patrimônio líquido
contas-entra-patrimonio-dica = Desligue para contas de terceiros que você só acompanha.
contas-notas = Notas
contas-apagar-nome = Apagar { $nome }
contas-apagar-descricao = Só é possível apagar contas sem lançamentos, recorrências ou parcelamentos. Para guardar o histórico, arquive.
contas-apagar-conta = Apagar conta
contas-conta-apagada = Conta apagada

## Banco da lista oficial

contas-banco-exemplo = Nome ou código do banco (ex.: 260, Itaú)
contas-banco-exemplo-exterior = Nome do banco ou da instituição
contas-plano-aposentadoria = Plano
contas-plano-aposentadoria-dica = O limite de aportes do ano depende do plano.
contas-cobertura = Cobertura
contas-cobertura-individual = Individual
contas-cobertura-familia = Família
contas-aposentadoria-titulo = { $plano } em { $ano }
contas-aposentadoria-de = { $aportado } de { $limite }
contas-aposentadoria-sem-limite = { $aportado } no ano (sem limite anual)
contas-aposentadoria-falta = Ainda cabem { $falta } este ano.
contas-aposentadoria-acima = Os aportes do ano passaram do limite: confira com o plano antes do prazo de correção.
contas-aposentadoria-beneficio = É o aporte que já dá todo o benefício fiscal.
contas-aposentadoria-idade = Limite para { $idade } anos.
contas-aposentadoria-divide = Divide o limite com { $contas }.
contas-aposentadoria-ano-anterior = O limite deste ano ainda não saiu: vale o de { $ano }.
contas-aposentadoria-fonte = Fonte do limite
contas-nova-instituicao = Nova instituição
contas-bancos-bc = Bancos do Banco Central
contas-banco-oficial = Banco da lista oficial; o código reconhece os extratos OFX.
contas-lista-oficial = Lista oficial do Banco Central, atualizada toda semana.

## Cheque especial

contas-cheque-especial = Cheque especial
contas-tem-cheque-especial = A conta tem limite de cheque especial
contas-cheque-especial-dica = O saldo pode ficar negativo até o limite; o Treasury mostra o quanto está em uso e uma estimativa de juros e IOF.
contas-juros-mes = Juros ao mês (%)
contas-desconto = Desconto (%)
contas-desconto-dica = Sobre a taxa de juros.
contas-juros-ano = Juros ao ano (%)
contas-juros-ano-spread = Juros ao ano sobre o índice (%)
contas-juros-ano-dica = Pode escrever com o índice, como o banco informa: 4,75 + TR.
contas-taxa-indexada-exemplo = ex.: 4,75 + TR
contas-indexador = Indexador
contas-indexador-dica = Pós-fixado: a taxa acima é somada ao índice (TR, CDI, IPCA…).
contas-cheque-taxa-indexada = · taxa efetiva ≈ { $taxa }% a.m. (com { $indice } de { $valor }% em { $data })
contas-cheque-taxa-indexada-estimada = · taxa efetiva ≈ { $taxa }% a.m. (com { $indice } projetado de { $valor }% em { $data })
contas-iof-basico = IOF básico (% ao dia)
contas-iof-adicional = IOF adicional (%)
contas-cet-mes = CET ao mês (%)
contas-cet-ano = CET ao ano (%)
contas-cheque-uso = { $usado } de { $limite }
contas-cheque-disponivel = Disponível com o limite: { $valor }
contas-cheque-custo = · custo estimado em 30 dias: { $valor }
contas-cheque-incerto = Conta sem extrato recente: o negativo pode ser lançamento faltando, não necessariamente cheque especial.
contas-atualizacao-ha = atualizado há { $dias } dias
contas-atualizacao-incompleto = possivelmente incompleto
contas-atualizacao-ultimo = Último lançamento em { $data }.
contas-atualizacao-sem-lancamentos = Nenhum lançamento ainda.
contas-atualizacao-mes-vazio = { $mes } ficou sem nenhum lançamento.
contas-atualizacao-negativo = O saldo negativo pode ser lançamento faltando, não necessariamente cheque especial.
contas-cheque-custo-juros = · custo estimado em 30 dias: { $valor } (juros de { $taxa }% a.m. + IOF)

## Cotas capital

contas-cotas-explicacao = Em cooperativas de crédito, o cooperado tem cotas capital: uma participação que rende conforme a cooperativa decide (juros ao capital, sobras). Ative para acompanhar o saldo das cotas, os aportes que saem desta conta e a rentabilidade de cada período (variação do saldo menos os aportes).
contas-cotas-ativar = Ativar cotas capital nesta conta
contas-cotas-saldo = Saldo das cotas
contas-cotas-aportes-periodo = Aportes no período
contas-cotas-rentabilidade = Rentabilidade
contas-cotas-nota = Nota
contas-cotas-pct = { NUMBER($valor, maximumFractionDigits: 2) }%
contas-cotas-em = em { $data }
contas-cotas-informe-saldo = informe o saldo
contas-cotas-aportes = Aportes
contas-cotas-depois-ultimo = { $valor } depois do último saldo
contas-cotas-rentabilidade-apurada = Rentabilidade apurada
contas-cotas-12-meses = Últimos 12 meses
contas-cotas-data-saldo = Data do saldo
contas-cotas-saldo-moeda = Saldo das cotas ({ $moeda })
contas-cotas-nota-opcional = Nota (opcional)
contas-cotas-nota-exemplo = Ex.: extrato anual da cooperativa
contas-cotas-saldo-informado = Saldo das cotas informado
contas-cotas-informar-saldo = Informar saldo
contas-cotas-saldos = Saldos das cotas capital
contas-cotas-aportes-identificados = Aportes identificados
contas-cotas-aportes-identificados-n = Aportes identificados ({ $n })
contas-cotas-ja-fora = Os aportes já estavam fora das despesas
contas-cotas-n-tirados =
    { $n ->
        [one] { $n } aporte tirado das despesas
       *[other] { $n } aportes tirados das despesas
    }
contas-cotas-tirar = Tirar os aportes das despesas
contas-cotas-como-identificar = Como identificar os aportes
contas-cotas-texto-extrato = Texto no extrato
contas-cotas-texto-dica = Separe por | (ex.: COTA CAPITAL|INTEGRALIZACAO).
contas-cotas-ou-categoria = Ou a categoria
contas-cotas-categoria = Categoria dos aportes
contas-cotas-nenhuma = Nenhuma
contas-cotas-somar = Somar o saldo das cotas ao patrimônio
contas-cotas-somar-dica = Entra como investimento, pelo último saldo informado.

## Faturas do cartão

contas-pagar-fatura-de = Pagar fatura de { $mes }
contas-pagar-fatura-descricao = Lança uma transferência do tipo “pagamento de fatura”: não conta como despesa (as compras já contaram). Datas futuras ficam como previstas.
contas-pagamento-lancado = Pagamento lançado
contas-pagar = Pagar
contas-ja-pago = Já pago
contas-vencimento = Vencimento
contas-pagar-com = Pagar com
contas-escolha-conta = Escolha a conta
contas-fatura-de = Fatura de { $mes }
contas-ver-em-lancamentos = Ver em lançamentos →
contas-compras = Compras
contas-compras-periodo = { $inicio } a { $fim }
contas-lancamentos = Lançamentos
contas-total-informado = Total informado pelo banco
contas-total-informado-salvo = Total informado salvo
contas-salvar-total-informado = Salvar total informado
contas-confere-calculado = Confere com o calculado ✓
contas-diferenca-banco = Diferença banco − calculado: { $valor }
contas-pagar-fatura = Pagar fatura
contas-lancamentos-fatura = Lançamentos da fatura
contas-pagamento = Pagamento
contas-sem-categoria = sem categoria
contas-fatura = Fatura
contas-situacao = Situação
contas-banco-calc = Banco × calc.
contas-confere-curto = confere
contas-faturas-vazio-kicker = Sem faturas
contas-faturas-vazio-titulo = Nenhuma compra neste cartão ainda
contas-faturas-vazio-descricao = As faturas aparecem sozinhas a partir das compras, pelo dia de fechamento e de vencimento.

## Parcelamentos e compromissos futuros do cartão

contas-a-pagar = A pagar
contas-compromissos-titulo = Compromissos futuros do cartão, 12 meses
contas-compromissos-fonte = Fonte: faturas em aberto, com parcelas futuras e recorrências previstas
contas-compromissos-grafico = Valor a pagar por mês nas próximas faturas
contas-compromissos-tabela = Compromissos futuros do cartão por mês
contas-compras-parceladas = Compras parceladas
contas-sem-parceladas = Nenhuma compra parcelada. Lance uma em Lançamentos › Novo › Parcelado no cartão.
contas-compra-em = compra em { $data }
contas-parcelas-pagas = { $pagas }/{ $total } pagas
contas-falta = falta
contas-apagar-parcelamento = Apagar parcelamento
contas-apagar-parcelamento-descricao = Todas as parcelas (passadas e futuras) saem do cartão. Dá para desfazer pelo histórico.
contas-apagar-parcelas = Apagar parcelas
contas-n-parcelas-apagadas =
    { $n ->
        [one] { $n } parcela apagada
       *[other] { $n } parcelas apagadas
    }

## Lista e página da conta

contas-lista = Contas
contas-col-conta = Conta
contas-col-membros = Membros
contas-col-saldo = Saldo
contas-sem-membro = Família
contas-voltar-lista = Todas as contas
contas-detalhe-resumo = Resumo da conta
contas-ultimos-lancamentos = Lançamentos
contas-col-descricao = Descrição
contas-col-categoria = Categoria
contas-col-valor = Valor
contas-sem-lancamentos = Nenhum lançamento nesta conta ainda.
contas-ver-todos-lancamentos = Ver todos em Lançamentos

## Ajuste de saldo

contas-ajustar-saldo = Ajustar saldo
contas-ajuste-explicacao = Informe o saldo que a conta tinha numa data (o do banco ou do extrato). O Treasury lança a diferença como ajuste: entra no saldo e no histórico da conta, mas fica fora de receitas, despesas e orçamento. Os lançamentos até a data ficam conciliados.
contas-ajuste-conta = Conta
contas-ajuste-data = Data do saldo
contas-ajuste-saldo = Saldo na data ({ $moeda })
contas-ajuste-saldo-dica = O que o banco mostra no fim do dia.
contas-ajuste-previa = Prévia do ajuste
contas-ajuste-calculado = Calculado em { $data }
contas-ajuste-informado = Informado
contas-ajuste-valor = Ajuste
contas-ajuste-confere = Já confere: só marca a data como conciliada.
contas-ajuste-fora-relatorios = Fica fora de receitas, despesas e orçamento.
contas-ajuste-sem-diferenca = Saldo conferido

## Cartões do cartão de crédito: titular, adicionais e virtuais
contas-cartoes = Cartões
contas-plasticos-explicacao = Cada cartão (o seu, os adicionais da família e os virtuais) tem o final e a pessoa que usa. A fatura e o limite continuam sendo um só; cada compra fica com quem usou o cartão.
contas-plasticos-vazio = Nenhum cartão cadastrado. Cadastre o seu e os adicionais para saber quem gastou.
contas-plastico-tipo = Tipo
contas-plastico-tipo-holder = Titular
contas-plastico-tipo-additional = Adicional
contas-plastico-tipo-virtual = Virtual
contas-plastico-tipo-temporary = Temporário
contas-plastico-final = Final { $final }
contas-plastico-final-rotulo = Final do cartão
contas-plastico-final-dica = Só os 4 últimos dígitos.
contas-plastico-membro = Quem usa
contas-plastico-titular-conta = Titular da conta
contas-plastico-do-titular = Titular da conta ({ $nome })
contas-plastico-apelido = Apelido
contas-plastico-apelido-exemplo = Ex.: assinaturas, viagem
contas-plastico-sublimite = Sublimite por fatura
contas-plastico-sublimite-dica = Só um aviso: o app não bloqueia compras.
contas-plastico-gasto-ciclo = Nesta fatura:
contas-plastico-uso-sublimite = { $pct }% do sublimite de
contas-plastico-acima-sublimite = passou do sublimite
contas-plastico-perto-sublimite = perto do sublimite
contas-plastico-novo = Novo cartão
contas-plastico-dialogo-descricao = As compras feitas com este cartão ficam com a pessoa escolhida.
contas-plastico-salvar = Salvar
contas-plastico-salvo = Cartão salvo
contas-plastico-editar = Editar
contas-plastico-ativo = Ativo
contas-plastico-inativo = Inativo
contas-plastico-remover = Remover
contas-plastico-remover-titulo = Remover { $cartao }?
contas-plastico-remover-confirmar = As compras feitas com ele voltam para o titular da conta.
contas-plastico-removido = Cartão removido
contas-fatura-por-cartao = Por cartão
contas-fatura-sem-cartao = Sem cartão informado

## Limite, datas da fatura e anuidade
contas-limite-anuidade = Limite e anuidade
contas-limite-uso = Uso do limite
contas-limite-usado = Usado:
contas-limite-de-total = de
contas-limite-disponivel = Disponível:
contas-limite-parcelas-futuras = Parcelas futuras já tomam
contas-limite-sem = Este cartão não tem limite cadastrado.
contas-melhor-dia = Melhor dia de compra: { $data }. O que for comprado a partir daí vence em { $vencimento } ({ $dias } dias para pagar).
contas-limite-historico = Histórico do limite
contas-limite-historico-vazio = Sem registros de limite.
contas-limite-novo = Registrar mudança de limite
contas-limite-novo-descricao = O banco aumentou ou reduziu o limite? Registre a data e o novo valor.
contas-limite-salvo = Limite registrado
contas-limite-da-fatura = Da fatura
contas-limite-informado = Informado
contas-anuidade = Anuidade
contas-anuidade-vazio = Informe a anuidade do cartão para o app conferir as cobranças, mostrar quanto falta para a isenção e lembrar você antes da renovação.
contas-anuidade-informar = Informar anuidade
contas-anuidade-mudar = Mudar regra
contas-anuidade-sem = Cartão sem anuidade
contas-anuidade-sem-titulo = Este cartão deixou de ter anuidade?
contas-anuidade-sem-descricao = A regra termina hoje. As cobranças antigas continuam no histórico.
contas-anuidade-regra = Anuidade do cartão
contas-anuidade-regra-descricao = O que foi combinado com o banco. Mudar a regra cria uma nova a partir da data de início.
contas-anuidade-salva = Anuidade salva
contas-anuidade-valor-ano = Valor do ano
contas-anuidade-parcelas = Parcelas
contas-anuidade-em-parcelas = em { $n }× de
contas-anuidade-mes = Mês de renovação
contas-anuidade-mes-nao-sei = Não sei
contas-anuidade-inicio = Vale a partir de
contas-anuidade-isencao = Isenção
contas-anuidade-isencao-none = Sem isenção
contas-anuidade-isencao-full = Isenta
contas-anuidade-isencao-spend = Por gasto na fatura
contas-anuidade-isencao-investment = Por valor investido no banco
contas-anuidade-isencao-discount = Desconto fixo
contas-anuidade-gasto-minimo = Gasto na fatura que isenta
contas-anuidade-investimento-minimo = Valor investido que isenta
contas-anuidade-desconto = Desconto (%)
contas-anuidade-notas = Observações
contas-anuidade-pago-12m = Pago em 12 meses
contas-anuidade-renova = Renova em
contas-anuidade-isenta-agora = A fatura em aberto já atingiu a isenção.
contas-anuidade-falta-gastar = Faltam nesta fatura para isentar a parcela:
contas-anuidade-falta-investir = Faltam no banco para a isenção:
contas-anuidade-cobrancas = Cobranças de anuidade
contas-anuidade-sem-cobrancas = Nenhuma cobrança de anuidade nos últimos 12 meses.
contas-anuidade-esperado = Esperado:
contas-anuidade-problema-isenta = Cobrada numa fatura que tinha atingido a isenção.
contas-anuidade-problema-valor = Valor diferente da parcela combinada.
contas-anuidade-problema-parcela = Passa do número de parcelas do ano.
contas-fatura-ajustar-botao = Ajustar datas e valores
contas-fatura-ajustar = Fatura de { $mes }
contas-fatura-ajustar-descricao = Use quando o banco adiar o fechamento ou mudar o vencimento de um mês. Datas ajustadas não mudam mais com o ciclo do cartão.
contas-fatura-datas-calculadas = Voltar às datas do ciclo
contas-fatura-datas-manuais = ajustada
contas-fatura-datas-impressas = da fatura do banco
contas-fatura-fechamento = Fechamento
contas-fatura-minimo = Pagamento mínimo
contas-fatura-limite = Limite na fatura
contas-fatura-saldo-anterior = Fatura anterior
contas-fatura-alterada = Fatura alterada
contas-rotativo-titulo = A fatura de { $mes } entrou no rotativo
contas-rotativo-sem-pagar = Ficou sem pagar:
contas-rotativo-encargos = Juros e encargos lançados desde então:
contas-rotativo-regras-br = O rotativo só pode durar até a próxima fatura, e juros e encargos do rotativo e do parcelamento da fatura não podem passar de 100% da dívida original (Lei 14.690/2023).
contas-rotativo-regras = O saldo que fica no cartão costuma ter juros altos; quitar ou trocar por um empréstimo mais barato costuma sair melhor.
contas-rotativo-acima-teto = Os encargos lançados já passam do valor da dívida original: confira a fatura com o banco.
contas-rotativo-alem-da-fatura = Já passou mais de uma fatura no rotativo: o banco deveria ter oferecido o parcelamento.

## Cashback e pontos
contas-recompensa-aba = Cashback e pontos
contas-recompensa-titulo = Cashback e pontos
contas-recompensa-vazio = Informe o cashback ou os pontos do cartão para o app calcular o previsto de cada fatura, conferir o que caiu e avisar quando o crédito não vier.
contas-recompensa-informar = Informar programa
contas-recompensa-sem = Sem programa
contas-recompensa-sem-titulo = Este cartão deixou de ter cashback ou pontos?
contas-recompensa-regra = Cashback e pontos do cartão
contas-recompensa-regra-descricao = O que o cartão devolve em cada compra. Mudar a regra cria uma nova a partir da data de início.
contas-recompensa-salva = Programa salvo
contas-recompensa-tipo = Tipo
contas-recompensa-cashback = Cashback
contas-recompensa-pontos = Pontos ou milhas
contas-recompensa-taxa = Taxa básica
contas-recompensa-taxa-cashback = Cashback (%)
contas-recompensa-taxa-pontos = Pontos por unidade de moeda
contas-recompensa-pontos-por = { $n } pontos por { $moeda }
contas-recompensa-moeda-pontos = Moeda da regra
contas-recompensa-moeda-pontos-dica = Muitos programas contam pontos por dólar gasto.
contas-recompensa-teto-cashback = Teto por fatura
contas-recompensa-teto-pontos = Teto de pontos por fatura
contas-recompensa-destino = Onde o cashback cai
contas-recompensa-destino-fatura = Crédito na fatura
contas-recompensa-destino-conta = Numa conta
contas-recompensa-destino-investimento = Num investimento
contas-recompensa-conta-destino = Conta que recebe
contas-recompensa-prazo = Prazo do crédito (dias)
contas-recompensa-prazo-dica = Dias depois do fechamento em que o cashback costuma cair.
contas-recompensa-por-categoria = Taxas por categoria
contas-recompensa-taxa-categoria = Taxa da categoria
contas-recompensa-nova-categoria = Adicionar categoria
contas-recompensa-previsto-12m = Previsto em 12 meses
contas-recompensa-recebido-12m = Recebido em 12 meses
contas-recompensa-como-conta = O cashback entra como receita (categoria Cashback e recompensas), sem diminuir o gasto das compras. Créditos com "cashback" na descrição também contam.
contas-recompensa-por-fatura = Por fatura
contas-recompensa-previsto = Previsto
contas-recompensa-recebido = Recebido
contas-recompensa-nao-veio = não veio até { $data }
contas-recompensa-n-pontos = { $n } pontos

## Pontos e milhas
contas-pontos-titulo = Pontos e milhas
contas-pontos-vazio = Cadastre os programas de pontos e milhas da família (Livelo, Esfera, Smiles, LATAM Pass…) para acompanhar o saldo de cada pessoa, os vencimentos e as transferências com bônus.
contas-pontos-fora-patrimonio = Pontos não entram no patrimônio; o valor estimado usa o valor do milheiro que você informar.
contas-pontos-programa = Programa
contas-pontos-programa-novo = Novo programa
contas-pontos-programa-descricao = Um programa por pessoa: a conta de pontos é de quem tem o CPF.
contas-pontos-programa-salvo = Programa salvo
contas-pontos-nome = Nome
contas-pontos-membro = De quem é
contas-pontos-sem-membro = Sem membro
contas-pontos-valor-milheiro = Valor de 1.000 pontos
contas-pontos-valor-milheiro-dica = Sua estimativa, para ver quanto o saldo vale. Fica fora do patrimônio.
contas-pontos-tipo-bank = Banco
contas-pontos-tipo-airline = Companhia aérea
contas-pontos-tipo-retail = Varejo
contas-pontos-tipo-other = Outro
contas-pontos-mov-earn = Acúmulo
contas-pontos-mov-bonus = Bônus
contas-pontos-mov-purchase = Compra de pontos
contas-pontos-mov-transfer_in = Transferência recebida
contas-pontos-mov-transfer_out = Transferência enviada
contas-pontos-mov-redeem = Resgate
contas-pontos-mov-expire = Expiração
contas-pontos-mov-adjust = Ajuste
contas-pontos-movimento-novo = Registrar pontos
contas-pontos-movimento-descricao = Acúmulo, bônus, compra, resgate ou expiração, como aparece no extrato do programa.
contas-pontos-movimento-salvo = Pontos registrados
contas-pontos-quantidade = Pontos
contas-pontos-vence-em = Vencem em
contas-pontos-vence = vencem em { $data }
contas-pontos-custo = Quanto custou
contas-pontos-transferir = Transferir
contas-pontos-transferir-descricao = Os pontos saem de um programa e entram em outro, com o bônus da campanha.
contas-pontos-transferencia-salva = Transferência registrada
contas-pontos-de = De
contas-pontos-para = Para
contas-pontos-saem = Pontos que saem
contas-pontos-bonus = Bônus (%)
contas-pontos-entram = Entram { $n } pontos no destino.
contas-pontos-sem-movimentos = Nenhum movimento ainda.
contas-pontos-n = { $n } pontos
contas-pontos-proximo-vencimento = { $n } vencem em { $data }
contas-pontos-cartoes = Acumula com: { $cartoes }
contas-recompensa-programa-pontos = Programa de pontos
contas-recompensa-programa-pontos-nenhum = Nenhum (só o previsto)

## Este cartão vale a pena?
contas-vale-titulo = Este cartão vale a pena?
contas-vale-periodo = De { $de } a { $ate }: o que o cartão custou e o que devolveu.
contas-vale-anuidade = Anuidade
contas-vale-encargos = Juros e encargos
contas-vale-iof = IOF
contas-vale-cashback = Cashback recebido
contas-vale-pontos = Pontos (valor estimado)
contas-vale-resultado = Resultado
contas-vale-simulacao = Se todas as compras da família fossem para um cartão só
contas-vale-simulacao-descricao = Retorno pelas regras de cada cartão sobre as compras de todos os cartões nos últimos 12 meses:
contas-vale-retorno = Retorno
contas-vale-aviso = É uma análise dos dados da família, não uma recomendação de produto financeiro. Benefícios como seguros e salas VIP não entram na conta.
