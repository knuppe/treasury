# Tela 07 · Dívidas: contratos, cronograma, figuras, simuladores, pagamento de parcelas, extrato
# e juros do mercado (docs/i18n-design.md). Resumos e avisos que vêm do núcleo não passam por
# aqui. pt-BR é o idioma de origem.

## Rótulos repetidos

dividas-contrato = Contrato
dividas-contrato-min = contrato
dividas-um-contrato = um contrato
dividas-contratos = Contratos
dividas-novo-contrato = Novo contrato
dividas-saldo-devedor = Saldo devedor
dividas-proxima-parcela = Próxima parcela
dividas-parcela = Parcela
dividas-parcelas = Parcelas
dividas-juros = Juros
dividas-juros-a-pagar = Juros a pagar
dividas-amortizacao = Amortização
dividas-amortizacao-extraordinaria = Amortização extraordinária
dividas-com-simulada = Com a amortização simulada
dividas-correcao = Correção
dividas-correcao-saldo = Correção do saldo
dividas-seguros-taxas = Seguros e taxas
dividas-sistema = Sistema
dividas-taxa = Taxa
dividas-data = Data
dividas-valor = Valor
dividas-ano = Ano
dividas-hoje = Hoje
dividas-vencimento = Vencimento
dividas-situacao = Situação
dividas-diferenca = Diferença
dividas-acoes = Ações
dividas-col-numero = Nº
dividas-data-pagamento = Data do pagamento
dividas-conta-de-debito = Conta de débito
dividas-conta-de-debito-min = conta de débito
dividas-categoria-pagamentos = Categoria dos pagamentos
dividas-custos-troca = Custos da troca
dividas-parcelas-que-faltam = Parcelas que faltam
dividas-prazo-meses = Prazo (meses)
dividas-calculando = Calculando…
dividas-sem-descricao = sem descrição
dividas-opcional = Opcional.
dividas-nenhuma = Nenhuma
dividas-nenhuma-min = nenhuma
dividas-arquivado = arquivado
dividas-arquivar = Arquivar
dividas-reativar = Reativar
dividas-excluir = Excluir
dividas-desligar = Desligar
dividas-estimado = ≈ estimado
dividas-quitado = Quitado
dividas-quitado-min = quitado
dividas-atrasada-alerta = ! atrasada
dividas-parcela-n-data = Parcela { $n } · { $data }
dividas-parcela-original = Parcela { $n } do contrato original
dividas-n-meses = { $n } meses

## Taxas (a.a. = ao ano, a.m. = ao mês)

dividas-taxa-aa = { $taxa } a.a.
dividas-taxa-am = { $taxa } a.m.
dividas-taxa-linear = { $taxa } (linear)
dividas-taxa-pct-cdi = { $pct } do CDI
dividas-taxa-pct-selic = { $pct } da Selic

## Prazo e situação das parcelas

dividas-prazo-parcelas =
    { $n ->
        [1] { $n } parcela
       *[other] { $n } parcelas
    }
dividas-prazo-longo =
    { $meses ->
        [0] { $n } parcelas ({ $anos } anos)
        [1] { $n } parcelas ({ $anos } anos e { $meses } mês)
       *[other] { $n } parcelas ({ $anos } anos e { $meses } meses)
    }
dividas-status-paga = Paga
dividas-status-atrasada = Atrasada
dividas-status-proxima = Próxima
dividas-status-prevista = Prevista
dividas-parcelas-atrasadas =
    { $n ->
        [1] { $n } parcela atrasada
       *[other] { $n } parcelas atrasadas
    }
dividas-igual = igual
dividas-pago-a-mais = Pago a mais que a parcela
dividas-pago-a-menos = Pago a menos que a parcela

## Tipos de contrato e enums do núcleo

dividas-tipo-imobiliario = Financiamento imobiliário
dividas-tipo-veiculo = Financiamento de veículo
dividas-tipo-pessoal = Empréstimo pessoal
dividas-tipo-consignado = Consignado
dividas-tipo-estudantil = Crédito estudantil
dividas-tipo-outro = Outro

dividas-sistema-sac-dica = Amortização constante: a parcela começa maior e cai com o tempo.
dividas-sistema-price-dica = Parcela constante (sem índice): no começo quase tudo é juros. Com TR ou IPCA, o saldo é corrigido antes e a parcela é recalculada todo mês.
dividas-sistema-bullet-dica = Só juros a cada vencimento e o principal inteiro na última parcela.
dividas-sistema-sac-dica-pos = Amortização constante + juros do período pelo índice: a parcela varia.
dividas-sistema-price-dica-pos = Parcela recalculada a cada vencimento com o índice do período.
dividas-sistema-bullet-dica-pos = Só os juros do período a cada vencimento; o principal vai no fim.

dividas-indexador-nenhum = Pré-fixado
dividas-indexador-poupanca = Poupança

dividas-prorata-nenhum = Sem pró-rata (mês cheio)
dividas-prorata-corridos = Por dias corridos
dividas-prorata-uteis = Por dias úteis

dividas-base-mensal = Mensal equivalente
dividas-base-252 = Dias úteis (base 252)
dividas-base-365 = Dias corridos (base 365)

dividas-referencia-mes = Variação do mês
dividas-referencia-aniversario = Data de aniversário

dividas-spread-exponencial = Exponencial (252 dias úteis)
dividas-spread-linear = Linear (360 dias corridos)

dividas-modo-prazo = Reduzir prazo
dividas-modo-parcela = Reduzir parcela
dividas-modo-prazo-dica = Mantém a parcela e encurta o contrato: economiza mais juros.
dividas-modo-parcela-dica = Mantém o prazo e baixa as próximas parcelas: alivia o mês.

## Tela e menu do contrato

dividas-menu-abrir = Abrir contrato
dividas-menu-amortizacao-extra = Amortização extra…
dividas-menu-editar = Editar…
dividas-menu-excluir = Excluir…
dividas-conciliar = Conciliar com os pagamentos
dividas-conciliar-sem-conta = Defina a conta de débito ou a categoria dos pagamentos para conciliar
dividas-carregando-contratos = Carregando os contratos
dividas-vazio-kicker = Sem contratos
dividas-vazio-titulo = Cadastre o primeiro empréstimo ou financiamento
dividas-vazio-descricao = Informe valor, taxa, sistema (SAC ou Price), prazo e indexador: o cronograma sai na hora, com juros × amortização, simulador de amortização extraordinária e o custo real descontado pela inflação.
dividas-mostrar-arquivados = Mostrar arquivados

## Resumo e lista de contratos

dividas-resumo-saldo-total = Saldo devedor total
dividas-resumo-contratos-ativos =
    { $n ->
        [1] { $n } contrato ativos
       *[other] { $n } contratos ativos
    }
dividas-resumo-compromisso = Compromisso mensal
dividas-resumo-proximas-somadas = Próximas parcelas somadas
dividas-resumo-em-atraso = ! { $n } em atraso
dividas-resumo-ate-o-fim = Até o fim dos contratos
dividas-resumo-pelo-cronograma = pelo cronograma atual
dividas-acoes-de = Ações de { $nome }
dividas-col-taxa-aa = Taxa a.a.
dividas-col-progresso = Progresso
dividas-col-custo-real = Custo real
dividas-numero-contrato = nº { $numero }
dividas-pre = pré
dividas-pos = pós
dividas-atrasada-em = ! atrasada · { $data }

## Detalhe do contrato

dividas-conciliadas =
    { $n ->
        [1] { $n } parcela conciliada
       *[other] { $n } parcelas conciliadas
    }
dividas-conciliar-nada = Nenhum pagamento novo encontrado
dividas-conciliar-procurado = Procurado em { $conta }: valor da parcela ±1% e até 7 dias do vencimento.
dividas-conciliar-erro = Não foi possível conciliar
dividas-reativado = Contrato reativado
dividas-arquivado-ok = Contrato arquivado
dividas-contrato-aria = Contrato { $nome }
dividas-de-membro = de { $nome }
dividas-contrato-de-data = contrato de { $data }
dividas-contrato-numero-de-data = contrato nº { $numero } de { $data }
dividas-importar-extrato = Importar extrato
dividas-amortizacao-extra = Amortização extra
dividas-atrasadas-texto =
    { $n ->
        [1] Vencida e sem pagamento registrado. Se já pagou, concilie com os lançamentos da conta de débito ou marque como paga na tabela.
       *[other] Vencidas e sem pagamento registrado. Se já pagou, concilie com os lançamentos da conta de débito ou marque como paga na tabela.
    }
dividas-marcadas-titulo = Parcelas marcadas como pagas
dividas-marcadas-descricao =
    { $n ->
        [one] { $n } parcela vencida, no vencimento e pelo valor do cronograma.
       *[other] { $n } parcelas vencidas, no vencimento e pelo valor do cronograma.
    }
dividas-marcar-vencidas = Marcar as { $n } vencidas como pagas
dividas-de-financiados = de { $valor } financiados
dividas-parcelas-pagas-de = { $pagas } de { $total } parcelas pagas
dividas-numero-data = Nº { $n } · { $data }
dividas-em-aberto = { $prazo } em aberto
dividas-taxa-custo = Taxa e custo
dividas-sistema-pre = pré-fixado
dividas-sistema-pos = pós-fixado, juros pelo { $indice } acumulado nos dias úteis
dividas-sistema-pos-defasagem = pós-fixado, juros pelo { $indice } acumulado nos dias úteis (D−{ $dias })
dividas-sistema-correcao = correção { $indice } ({ $detalhes })
dividas-correcao-aniversario = no aniversário
dividas-correcao-mes = do mês
dividas-correcao-piso = piso zero
dividas-correcao-prorata = 1º período pró-rata
dividas-cet-contratado = CET contratado
dividas-custo-realizado = Custo realizado
dividas-cet-realizado = CET realizado
dividas-custo-real-ipca = Custo real (− IPCA)
dividas-juros-pagos = Juros pagos
dividas-seguros-taxas-pagos = Seguros e taxas pagos
dividas-montando-cronograma = Montando o cronograma
dividas-simulador-amortizacao = Simulador de amortização extraordinária
dividas-antecipar-parcelas = Antecipar parcelas
dividas-simulador-portabilidade = Simulador de portabilidade
dividas-tabela-amortizacao = Tabela de amortização
dividas-seguros-taxas-mes = Seguros + taxas/mês
dividas-origem = Origem
dividas-origem-valor = cronograma do núcleo · ≈ índice estimado
dividas-observacoes-valor = Observações: { $notas }
dividas-excluir-titulo = Excluir { $nome }
dividas-excluir-descricao = O contrato, o cronograma e as amortizações registradas saem. Os lançamentos das contas continuam. Dá para desfazer pelo histórico de alterações.
dividas-excluir-confirmar = Excluir contrato
dividas-excluido = Contrato excluído

## Figuras

dividas-grafico-saldo-estimado = Saldo devedor (≈ índice estimado)
dividas-grafico-parcelas-n = Parcelas ({ $n })
dividas-grafico-parcela-estimada = Parcela { $n } · { $data } · ≈ índice estimado
dividas-grafico-correcao-nota = A correção não sai na parcela: é somada ao saldo devedor.
dividas-figura-saldo-titulo = Saldo devedor ao longo do contrato
dividas-figura-saldo-fonte = Cronograma do contrato
dividas-figura-saldo-fonte-indice = Cronograma do contrato · correção pelo índice (≈ meses sem dado oficial usam o último índice)
dividas-figura-saldo-aria = Saldo devedor de { $nome } ao longo do contrato.
dividas-figura-saldo-aria-simulada = Saldo devedor de { $nome } ao longo do contrato, com a amortização simulada.
dividas-figura-saldo-tabela = Saldo devedor por vencimento
dividas-figura-saldo-col = Saldo
dividas-figura-composicao-titulo =
    { $modo ->
        [ano] Juros × amortização por ano
       *[parcela] Juros × amortização por parcela
    }
dividas-figura-composicao-fonte-correcao = A correção do saldo pelo índice não é paga na parcela: entra no saldo devedor e é amortizada depois.
dividas-figura-composicao-fonte = Composição de cada parcela do cronograma.
dividas-figura-composicao-agrupar = Agrupar a composição
dividas-figura-composicao-aria =
    { $modo ->
        [ano] Composição das parcelas de { $nome } por ano: { $partes }.
       *[parcela] Composição das parcelas de { $nome } por parcela: { $partes }.
    }
dividas-figura-composicao-tabela =
    { $modo ->
        [ano] Composição por ano
       *[parcela] Composição por parcela
    }
dividas-figura-pago-no-ano = Pago no ano

## Juros do mercado (Banco Central)

dividas-mercado-titulo = Juros do mercado
dividas-mercado-intro = Quanto os bancos cobram em cada tipo de crédito, pelos dados do Banco Central. Serve para saber se o seu contrato está caro e se vale negociar ou levar a dívida para outro banco (portabilidade).
dividas-mercado-abaixo = Abaixo do mercado
dividas-mercado-media = Na média do mercado
dividas-mercado-acima = Acima do mercado
dividas-mercado-bem-acima = Bem acima do mercado
dividas-mercado-seus-contratos = Seus contratos × mercado
dividas-mercado-taxas = Taxas do mercado
dividas-mercado-credito = Crédito
dividas-mercado-mais-barato = Mais barato (a.m.)
dividas-mercado-mediana = Mediana (a.m.)
dividas-mercado-mais-caro = Mais caro (a.m.)
dividas-mercado-bancos = Bancos mais baratos
dividas-mercado-ao-ano = { $taxa } ao ano
dividas-mercado-atualizado = Atualizado em { $data }.
dividas-mercado-atualizar = Atualizar

## Antecipar parcelas

dividas-antecipar-dica-ultimas = Quita as últimas: o contrato termina antes e o desconto é maior.
dividas-antecipar-dica-proximas = Paga agora as próximas: o desconto é menor e o prazo fica igual.
dividas-antecipar-em-aberto = Em aberto: { $n }
dividas-antecipar-ordem = Ordem
dividas-antecipar-ordem-aria = Ordem da antecipação
dividas-antecipar-ultimas = Das últimas
dividas-antecipar-proximas = Das próximas
dividas-antecipar-informe = Informe quantas parcelas antecipar.
dividas-antecipar-resumo = Pagando { $valor } em { $data }, o desconto dos juros é de { $desconto }.
dividas-antecipar-resumo-encargos = Pagando { $valor } em { $data }, o desconto dos juros é de { $desconto } e deixam de ser cobrados { $encargos } de seguros e taxas.
dividas-antecipar-valor-cronograma = Valor no cronograma
dividas-antecipar-a-pagar = A pagar agora
dividas-antecipar-ficam = Ficam em aberto
dividas-antecipadas = Parcelas antecipadas
dividas-antecipadas-descricao =
    { $n ->
        [one] { $n } parcela por { $valor }
       *[other] { $n } parcelas por { $valor }
    }
dividas-antecipar-registrar = Registrar antecipação
dividas-antecipar-com-conta = A saída é lançada na conta de débito do contrato.
dividas-antecipar-sem-conta = Sem conta de débito: só as parcelas ficam pagas.

## Simulador de amortização extraordinária

dividas-simulador-valor = Valor da amortização
dividas-simulador-acima-saldo = Acima do saldo devedor ({ $saldo }): quita o contrato.
dividas-simulador-saldo-hoje = Saldo devedor hoje: { $saldo }
dividas-simulador-informe = Informe um valor para comparar as duas formas de amortizar. A simulação não grava nada.
dividas-simulador-forma = Forma de amortizar
dividas-simulador-juros-economizados = Juros economizados
dividas-simulador-parcelas-restantes = Parcelas restantes
dividas-simulador-a-menos = { $prazo } a menos
dividas-simulador-quitacao = Quitação
dividas-simulador-nota = Simulação pelo cronograma atual. Confirme os valores com o banco antes de pagar.
dividas-simulador-nota-indice = Simulação pelo cronograma atual, com o índice projetado pelo último conhecido. Confirme os valores com o banco antes de pagar.
dividas-simulador-registrar = Registrar esta amortização

## Simulador de portabilidade

dividas-portabilidade-nova-taxa = Nova taxa (% a.a.)
dividas-portabilidade-hoje = Hoje: { $taxa }
dividas-portabilidade-prazo-erro = Entre 1 e 600 meses.
dividas-portabilidade-restam = Restam { $n }
dividas-portabilidade-sistema-dica = Da nova dívida.
dividas-portabilidade-sistema-aria = Sistema da nova dívida
dividas-portabilidade-custos-dica = Tarifas, avaliação, registro.
dividas-portabilidade-informe = Informe a taxa oferecida pelo outro banco para comparar. A simulação não grava nada.
dividas-portabilidade-vale = Vale a pena: economia de { $valor } no total, já com os custos da troca.
dividas-portabilidade-nao-vale = Não compensa: custo a mais de { $valor } no total, já com os custos da troca.
dividas-portabilidade-saldo-levado = Saldo levado
dividas-portabilidade-falta-atual = Falta pagar no atual
dividas-portabilidade-total-nova = Total na nova dívida
dividas-portabilidade-novo-prazo = Novo prazo
dividas-portabilidade-nota = A nova dívida é calculada sem indexador (taxa prefixada). Contratos com TR ou IPCA no atual podem ter custo real diferente.

## Ligar parcela a um lançamento

dividas-ligar-ok = Parcela { $n } ligada ao lançamento
dividas-descricao = Descrição
dividas-conta = Conta
dividas-conta-debito-contrato = Conta de débito do contrato
dividas-debito = débito
dividas-ligar-titulo = Ligar a parcela { $n } a um lançamento
dividas-ligar-descricao = { $contrato } · vencimento em { $data } · parcela de { $valor }. Saídas até 20 dias do vencimento que ainda não pagam outra parcela, as de valor mais próximo primeiro. A parcela fica paga na data e pelo valor do lançamento.
dividas-ligar-trocar = Trocar pelo lançamento
dividas-ligar-ao-lancamento = Ligar ao lançamento
dividas-lancamento-ligado = Lançamento ligado
dividas-paga-pelo-lancamento = Paga pelo lançamento
dividas-desligada = Parcela { $n } desligada do lançamento e em aberto
dividas-desligar-lancamento = Desligar o lançamento
dividas-ligar-procurando = Procurando as saídas perto do vencimento…
dividas-ligar-tabela = Lançamentos para a parcela { $n }
dividas-ligar-vazio = Nenhuma saída sem parcela até 20 dias do vencimento. Importe o extrato da conta ou lance o pagamento em Lançamentos.

## Tabela de amortização

dividas-paga-pelo-lancamento-titulo = Paga pelo lançamento { $lancamento }
dividas-paga-pelo-lancamento-titulo-conta = Paga pelo lançamento { $lancamento } em { $conta }
dividas-mais-extra = + extra { $valor }
dividas-juros-periodo = Juros do período
dividas-juros-periodo-dica = Índice acumulado nos dias úteis do período × spread
dividas-dias-uteis = { $n } dias úteis
dividas-encargos-detalhe = MIP { $mip } · DFI { $dfi } · outros seguros { $outros } · taxas { $taxas }
dividas-pago-valor = pago { $valor }
dividas-saldo-apos = Saldo após
dividas-paga-em = Paga em { $data }
dividas-indice-estimado-dica = Índice do mês ainda não divulgado: usa o último conhecido
dividas-indice-estimado = índice estimado
dividas-ligar-reticencias = Ligar…
dividas-desligar-parcela = Desligar o lançamento da parcela { $n }
dividas-desfazer-parcela = Desfazer pagamento da parcela { $n }
dividas-desfazer = Desfazer
dividas-marcar-parcela-aria = Marcar parcela { $n } como paga
dividas-marcar-paga = Marcar paga
dividas-menu-marcar-paga = Marcar como paga…
dividas-menu-ligar-outro = Ligar a outro lançamento…
dividas-menu-ligar-um = Ligar a um lançamento…
dividas-menu-ver-lancamento = Ver o lançamento
dividas-menu-desfazer = Desfazer o pagamento
dividas-filtro-aria = Parcelas a mostrar
dividas-filtro-proximas = Próximas
dividas-filtro-abertas = Em aberto · { $n }
dividas-filtro-pagas = Pagas · { $n }
dividas-filtro-todas = Todas · { $n }
dividas-tabela-de = Tabela de amortização · { $nome }
dividas-acoes-parcela = Ações da parcela { $n }
dividas-filtro-vazio = Nenhuma parcela neste filtro.

## Marcar parcela como paga e desfazer

dividas-paga-sincronizada = Parcela { $n } paga e saldo sincronizado
dividas-marcada-paga = Parcela { $n } marcada como paga
dividas-pagar-titulo = Parcela { $n } paga
dividas-pagar-descricao = Vencimento em { $data }. Marca a parcela como paga sem lançamento; se o pagamento já está em Lançamentos, use “Ligar…” na parcela (ou “Conciliar com os pagamentos” para todas).
dividas-marcar-como-paga = Marcar como paga
dividas-valor-pago = Valor pago
dividas-valor-pago-dica = Com multa e juros de atraso, se houver.
dividas-saldo-depois = Saldo devedor depois do pagamento
dividas-saldo-depois-dica = Opcional, do comprovante: sincroniza o saldo e recalcula as próximas parcelas.
dividas-amortizacao-parcela = Amortização da parcela
dividas-amortizacao-parcela-dica = Com o saldo, separa a amortização extra (abre nova versão do contrato).
dividas-faltam-dica = Do comprovante; vazio mantém o prazo atual.
dividas-desligar-descricao = A parcela deixa de estar paga pelo lançamento { $lancamento } e volta a ficar em aberto; o lançamento continua na conta. Dá para desfazer pelo histórico de alterações.
dividas-desfazer-descricao = A parcela volta a ficar em aberto. Dá para desfazer pelo histórico de alterações.
dividas-desfazer-pagamento = Desfazer pagamento
dividas-em-aberto-de-novo = Parcela { $n } em aberto de novo

## Extrato do contrato

dividas-extrato-importadas =
    { $n ->
        [one] { $n } parcela do extrato registrado
       *[other] { $n } parcelas do extrato registradas
    }
dividas-extrato-projecao =
    { $n ->
        [one] Projeção a partir do saldo de { $data }, com { $n } parcela restante.
       *[other] Projeção a partir do saldo de { $data }, com { $n } parcelas restantes.
    }
dividas-extrato-titulo = Importar extrato do contrato
dividas-extrato-descricao = Cole o histórico do banco (da planilha ou do internet banking), com o cabeçalho. As parcelas ficam com os valores do banco, as amortizações extraordinárias são achadas pelo saldo e o cronograma futuro parte do último saldo. Também dá para mandar a imagem ou o PDF para o agente.
dividas-extrato-conferir = Conferir
dividas-extrato-importar = Importar
dividas-extrato-importar-n =
    { $n ->
        [one] Importar { $n } parcela
       *[other] Importar { $n } parcelas
    }
dividas-extrato-colado = Extrato colado
dividas-extrato-colado-dica = Colunas: data, prestação, amortização, juros, seguros, taxas, saldo devedor, correção monetária e valor pago (os nomes do cabeçalho bastam; a ordem da planilha da Caixa também é reconhecida).
dividas-extrato-faltam-dica = Depois da última linha. Vazio: calculado pela última prestação.
dividas-extrato-marcar-anteriores = Marcar como pagas as parcelas antes do extrato
dividas-extrato-previa = Prévia do extrato
dividas-extrato-periodo = Período
dividas-extrato-periodo-valor = { $de } a { $ate }
dividas-extrato-ultimo-saldo = Último saldo
dividas-extrato-prazo-restante = Prazo restante
dividas-extrato-juros = Juros do extrato
dividas-extrato-juros-valor = { $mensal }% a.m. ({ $nominal }% nominal, { $efetiva }% efetiva a.a.)
dividas-extrato-extras = Amortizações extraordinárias achadas
dividas-extrato-extras-aria = Amortizações achadas
dividas-extrato-avisos = Avisos
dividas-extrato-taxa-ajustada = Taxa ajustada para { $taxa }% a.a. efetiva
dividas-extrato-usar-taxa = Usar a taxa do extrato ({ $taxa }% efetiva)

## Pagamento de parcela a partir de um lançamento

dividas-pagamento-so-saida = Só uma saída (valor negativo) pode pagar uma parcela.
dividas-pagamento-transferencia = É uma transferência entre suas contas: desfaça a transferência para ligar à parcela.
dividas-pagamento-previsto = O lançamento ainda está previsto: efetive-o antes de ligar à parcela.
dividas-pagamento-ligado-ok = Lançamento ligado à parcela { $n } de { $contrato }
dividas-pagamento-paga-em = Parcela paga em { $data }.
dividas-pagamento-desligado = Lançamento desligado da parcela { $n } de { $contrato }
dividas-pagamento-voltou = A parcela voltou a ficar em aberto.
dividas-pagamento-este = Este lançamento
dividas-pagamento-titulo = Pagamento de parcela
dividas-pagamento-trocar-para = Trocar para a parcela { $n }
dividas-pagamento-ligar-n = Ligar à parcela { $n }
dividas-pagamento-ligar = Ligar à parcela
dividas-pagamento-saiu-de = saiu de { $conta }
dividas-pagamento-ligado = Este lançamento paga a parcela { $n } de { $contrato }.
dividas-pagamento-ligado-texto = Escolha outra parcela para trocar, ou desligue: a parcela volta a ficar em aberto.
dividas-pagamento-carregando-contratos = Carregando os contratos…
dividas-pagamento-sem-contrato = Nenhum contrato ativo. Cadastre o empréstimo ou financiamento em { $tela }.
dividas-pagamento-sem-contrato-moeda = Nenhum contrato ativo em { $moeda }. Cadastre o empréstimo ou financiamento em { $tela }.
dividas-pagamento-escolha = Escolha o contrato
dividas-pagamento-carregando-parcelas = Carregando as parcelas…
dividas-pagamento-parcelas-de = Parcelas de { $nome }
dividas-pagamento-vazio = Nenhuma parcela sem lançamento neste contrato.
dividas-pagamento-nota = As 12 parcelas sem lançamento de vencimento mais perto de { $data }. A escolhida fica paga em { $data } pelo valor do lançamento.
dividas-pagamento-nota-categoria = As 12 parcelas sem lançamento de vencimento mais perto de { $data }. A escolhida fica paga em { $data } pelo valor do lançamento, e o lançamento ganha a categoria dos pagamentos do contrato.

## Cadastro e edição do contrato

dividas-form-editar = Editar contrato
dividas-form-editar-descricao = Corrija qualquer campo: as parcelas em aberto são recalculadas. Parcelas pagas (com o valor e a data que você registrou), amortizações extraordinárias e antecipações ficam como estão.
dividas-form-novo-descricao = Empréstimo ou financiamento: o cronograma (SAC ou Price, com carência, indexador, seguros e taxas) é gerado na hora.
dividas-form-erro-nome = Dê um nome ao contrato.
dividas-form-erro-principal = Informe o valor financiado.
dividas-form-erro-parcela = Informe o valor da parcela.
dividas-form-erro-parcela-nao-fecha = Com essa parcela e esse prazo a taxa não fecha: a parcela vezes o prazo precisa passar do valor financiado.
dividas-form-erro-taxa = Informe a taxa efetiva ao ano.
dividas-form-erro-taxa-intervalo = Taxa fora do intervalo.
dividas-form-erro-data-contrato = Informe a data do contrato.
dividas-form-erro-primeiro-vencimento = Informe o primeiro vencimento.
dividas-form-erro-vencimento-antes = A primeira parcela não pode vencer antes do contrato.
dividas-form-erro-prazo = Informe o prazo em meses.
dividas-form-erro-carencia = Carência em meses (0 se não houver).
dividas-form-erro-carencia-prazo = A carência deve ser menor que o prazo.
dividas-form-erro-percentual = Percentual do índice entre 0 e 1000%.
dividas-form-erro-defasagem = Defasagem de 0 a 10 dias úteis.
dividas-form-erro-bullet = No bullet não há carência: a parcela já é só juros.
dividas-form-erro-valor = Valor inválido.
dividas-form-alterado = Contrato alterado
dividas-form-cadastrado = Contrato cadastrado
dividas-form-cadastrado-descricao = { $nome }: { $n } parcelas no cronograma
dividas-form-dados = Dados do contrato
dividas-form-nome = Nome do contrato
dividas-form-nome-exemplo = Financiamento do apartamento
dividas-form-numero = Número do contrato
dividas-form-numero-dica = Como aparece no contrato ou no internet banking. Opcional.
dividas-form-principal = Valor financiado
dividas-form-primeiro-vencimento = Primeiro vencimento
dividas-form-sistema = Sistema de amortização
dividas-form-tipo-taxa = Tipo de taxa
dividas-form-pre = Pré-fixada
dividas-form-pos = Pós-fixada (CDI, Selic)
dividas-form-indice = Índice
dividas-form-indice-dica = Juros de cada período pelo índice acumulado nos dias úteis (B3, base 252).
dividas-form-percentual = Percentual do índice
dividas-form-percentual-dica = 100% na maioria dos contratos; ex.: 120% do CDI.
dividas-form-spread = Spread (% a.a.)
dividas-form-spread-dica = Taxa acima do índice: { $indice } + spread. Deixe vazio se for só o percentual.
dividas-form-taxa-juros = Taxa de juros
dividas-form-pela-parcela = Calcular pela parcela
dividas-form-informar-taxa = Informar a taxa
dividas-form-primeira-parcela = Primeira parcela
dividas-form-parcela-juros = Parcela de juros
dividas-form-valor-parcela = Valor da parcela
dividas-form-parcela-dica-encargos = Como no boleto ou no extrato: os seguros e taxas por mês (abaixo) saem da conta.
dividas-form-parcela-dica = Como no boleto ou no extrato, com seguros e taxas (informe-os abaixo).
dividas-form-taxa-calculada = Taxa calculada
dividas-form-taxa-nominal = Taxa nominal (% a.a.)
dividas-form-taxa-efetiva = Taxa efetiva (% a.a.)
dividas-form-como-taxa = Como o contrato traz a taxa
dividas-form-efetiva = Efetiva
dividas-form-nominal = Nominal
dividas-form-nominal-dica = Nominal ao ano (a mensal é a anual ÷ 12, como na Caixa).
dividas-form-nominal-dica-equivale = Nominal ao ano (a mensal é a anual ÷ 12, como na Caixa). Equivale a { $taxa }% a.a. efetiva.
dividas-form-efetiva-dica = Taxa de juros do contrato, sem o indexador. A mensal é a equivalente.
dividas-form-detalhes = Detalhes
dividas-form-tipo = Tipo
dividas-form-instituicao = Instituição
dividas-form-nao-informar = Não informar
dividas-form-membro = Membro da família
dividas-form-familia = Família
dividas-form-data-contrato = Data do contrato
dividas-form-data-contrato-dica = Um mês antes do 1º vencimento.
dividas-form-forma-spread = Forma do spread
dividas-form-forma-spread-dica = Exponencial é o padrão (CCB, capital de giro); linear aparece em contratos antigos.
dividas-form-defasagem = Defasagem do índice (dias úteis)
dividas-form-defasagem-dica = 0 na maioria; alguns bancos usam o DI de 1 ou 2 dias úteis antes.
dividas-form-correcao-dica = Correção monetária mensal do saldo (TR no SFH, IPCA ou INCC no imobiliário).
dividas-form-juros-dica = Como a taxa (ou os juros reais) corre em cada período. Mensal é o mais comum.
dividas-form-indice-usado = Índice usado
dividas-form-indice-aniversario-dica = O do período que começa no aniversário do contrato (padrão da TR e da poupança).
dividas-form-indice-mes-dica = A variação de um mês, com a defasagem ao lado (o IPCA sai no mês seguinte).
dividas-form-mes-indice = Mês do índice
dividas-form-mes-indice-dica = Vencimento em M usa o índice de M − n.
dividas-form-mesmo-mes = Do mesmo mês (M)
dividas-form-deflacao = Ignorar deflação
dividas-form-deflacao-dica = Meses com índice negativo não reduzem o saldo (piso zero).
dividas-form-prorata = Pró-rata do 1º período
dividas-form-prorata-dica = Quando a assinatura não é exatamente um mês antes do 1º vencimento.
dividas-form-prorata-pos = Pelos dias úteis do período (pós)
dividas-form-carencia = Carência (meses)
dividas-form-carencia-dica = Só juros nesse período.
dividas-form-valor-imovel = Valor do imóvel
dividas-form-valor-imovel-dica = Base do seguro DFI (danos físicos).
dividas-form-mip-taxa = Seguro MIP (% a.m.)
dividas-form-mip-valor = Seguro MIP ({ $moeda } por mês)
dividas-form-como-mip = Como o contrato traz o MIP
dividas-form-mip-taxa-dica = Morte e invalidez, sobre o saldo devedor: cai junto com a dívida.
dividas-form-mip-taxa-dica-hoje = Morte e invalidez, sobre o saldo devedor: cai junto com a dívida. Hoje ≈ { $valor } por mês.
dividas-form-mip-valor-dica = Valor fixo de morte e invalidez em toda parcela, como no contrato.
dividas-form-mip-valor-dica-equivale = Valor fixo de morte e invalidez em toda parcela, como no contrato. Equivale a { $pct }% a.m. do saldo de { $saldo }.
dividas-form-dfi-taxa = Seguro DFI (% a.m.)
dividas-form-dfi-valor = Seguro DFI ({ $moeda } por mês)
dividas-form-como-dfi = Como o contrato traz o DFI
dividas-form-dfi-taxa-dica = Danos físicos, sobre o valor do imóvel.
dividas-form-dfi-taxa-dica-mes = Danos físicos, sobre o valor do imóvel. ≈ { $valor } por mês.
dividas-form-dfi-valor-dica = Valor fixo de danos físicos em toda parcela (não precisa do valor do imóvel).
dividas-form-dfi-valor-dica-equivale = Valor fixo de danos físicos em toda parcela (não precisa do valor do imóvel). Equivale a { $pct }% a.m. do imóvel.
dividas-form-outros-seguros = Outros seguros por mês
dividas-form-seguros = Seguros por mês
dividas-form-outros-seguros-dica = Valor fixo, além do MIP e do DFI.
dividas-form-seguros-dica = Prestamista e outros.
dividas-form-taxa-adm = Taxa de administração por mês
dividas-form-taxa-adm-dica = Tarifa mensal do contrato (no financiamento imobiliário, costuma ser R$ 25).
dividas-form-cet = CET contratado (% a.a.)
dividas-form-cet-dica = Do contrato ou da proposta. Opcional.
dividas-form-conta-dica = Para conciliar as parcelas com os lançamentos.
dividas-form-categoria-dica = Um lançamento nesta categoria (importado ou digitado) marca a parcela mais próxima como paga.
dividas-form-observacoes = Observações
dividas-form-ja-paguei =
    { $n ->
        [1] Já paguei as { $n } parcela vencida até hoje
       *[other] Já paguei as { $n } parcelas vencidas até hoje
    }
dividas-form-ja-paguei-dica = Ficam pagas no vencimento, pelo valor do cronograma. Desligue para marcar uma a uma (ou conciliar com os lançamentos da conta).
dividas-form-pos-nota = A parcela varia com o { $indice }: ao salvar, o cronograma usa as taxas já publicadas e, para o futuro, a última conhecida (parcelas marcadas como estimadas e refeitas a cada coleta de índices).
dividas-form-na-carencia = Na carência
dividas-form-ultima-parcela = Última parcela
dividas-form-correcao-valor = + { $indice } no saldo
dividas-form-salvar = Salvar alterações
dividas-form-cadastrar = Cadastrar contrato

## Amortização extraordinária

dividas-extra-descricao = { $nome }: o pagamento abate o saldo devedor e o cronograma em aberto é recalculado. Dá para desfazer pelo histórico de alterações.
dividas-extra-erro-valor = Informe o valor.
dividas-extra-erro-data = Informe a data (dd/mm/aaaa).
dividas-extra-registrada = Amortização registrada
dividas-extra-registrada-descricao = { $nome }: saldo devedor e cronograma recalculados
dividas-extra-depois = Depois da amortização
dividas-extra-sai-da-conta = Sai da conta
dividas-extra-sem-conta-dica = Registra só no contrato, sem lançamento (ex.: FGTS ou pagamento por outra pessoa).
dividas-extra-com-conta-dica = Cria o lançamento de saída nessa conta.
dividas-extra-sem-lancamento = Sem lançamento (FGTS, outra fonte)
dividas-extra-fgts-saldo = conta FGTS, saldo { $saldo }
dividas-extra-fgts-descricao = A saída ficou na conta FGTS, como saque para moradia.
dividas-extra-data-passada = Data passada ({ $data }): entra depois da última parcela vencida até lá.
dividas-extra-registrar = Registrar amortização
