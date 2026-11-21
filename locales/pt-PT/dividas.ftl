# Variante pt-PT (opcional) da tela 07 · Dívidas: o que faltar cai para pt-BR (docs/i18n-design.md).
# Revisão: rascunho por IA, ainda sem revisão de um falante nativo.

## Rótulos repetidos

dividas-saldo-devedor = Capital em dívida
dividas-proxima-parcela = Próxima prestação
dividas-parcela = Prestação
dividas-parcelas = Prestações
dividas-com-simulada = Com a amortização simulada
dividas-data-pagamento = Data do pagamento
dividas-parcelas-que-faltam = Prestações em falta
dividas-calculando = A calcular…
dividas-excluir = Eliminar
dividas-desligar = Desassociar
dividas-quitado = Liquidado
dividas-quitado-min = liquidado
dividas-atrasada-alerta = ! em atraso
dividas-parcela-n-data = Prestação { $n } · { $data }
dividas-parcela-original = Prestação { $n } do contrato original

## Prazo e situação das prestações

dividas-prazo-parcelas =
    { $n ->
        [1] { $n } prestação
       *[other] { $n } prestações
    }
dividas-prazo-longo =
    { $meses ->
        [0] { $n } prestações ({ $anos } anos)
        [1] { $n } prestações ({ $anos } anos e { $meses } mês)
       *[other] { $n } prestações ({ $anos } anos e { $meses } meses)
    }
dividas-status-atrasada = Em atraso
dividas-parcelas-atrasadas =
    { $n ->
        [1] { $n } prestação em atraso
       *[other] { $n } prestações em atraso
    }
dividas-pago-a-mais = Pago a mais do que a prestação
dividas-pago-a-menos = Pago a menos do que a prestação

## Tipos de contrato e enums do núcleo

dividas-tipo-imobiliario = Crédito à habitação
dividas-tipo-veiculo = Crédito automóvel
dividas-tipo-pessoal = Crédito pessoal
dividas-tipo-consignado = Crédito consignado
dividas-tipo-estudantil = Crédito a estudantes

dividas-sistema-sac-dica = Amortização constante: a prestação começa mais alta e desce com o tempo.
dividas-sistema-price-dica = Prestação constante (sem índice): no início quase tudo são juros. Com TR ou IPCA, o capital é corrigido antes e a prestação é recalculada todos os meses.
dividas-sistema-bullet-dica = Só juros em cada vencimento e o capital inteiro na última prestação.
dividas-sistema-sac-dica-pos = Amortização constante + juros do período pelo índice: a prestação varia.
dividas-sistema-price-dica-pos = Prestação recalculada em cada vencimento com o índice do período.
dividas-sistema-bullet-dica-pos = Só os juros do período em cada vencimento; o capital vai no fim.

dividas-indexador-nenhum = Taxa fixa
dividas-indexador-poupanca = Poupança (Brasil)

dividas-modo-parcela = Reduzir prestação
dividas-modo-prazo-dica = Mantém a prestação e encurta o contrato: poupa mais juros.
dividas-modo-parcela-dica = Mantém o prazo e baixa as próximas prestações: alivia o mês.

## Tela e menu do contrato

dividas-menu-excluir = Eliminar…
dividas-conciliar-sem-conta = Defina a conta de débito ou a categoria dos pagamentos para reconciliar
dividas-conciliar = Reconciliar com os pagamentos
dividas-carregando-contratos = A carregar os contratos
dividas-vazio-titulo = Registe o primeiro empréstimo ou crédito
dividas-vazio-descricao = Indique o valor, a taxa, o sistema (SAC ou Price), o prazo e o indexante: o plano de pagamentos sai na hora, com juros × amortização, simulador de amortização antecipada e o custo real descontada a inflação.

## Resumo e lista de contratos

dividas-resumo-saldo-total = Capital em dívida total
dividas-resumo-contratos-ativos =
    { $n ->
        [1] { $n } contrato ativo
       *[other] { $n } contratos ativos
    }
dividas-resumo-proximas-somadas = Próximas prestações somadas
dividas-resumo-pelo-cronograma = pelo plano de pagamentos atual
dividas-pre = fixa
dividas-pos = variável
dividas-atrasada-em = ! em atraso · { $data }

## Detalhe do contrato

dividas-conciliadas =
    { $n ->
        [1] { $n } prestação reconciliada
       *[other] { $n } prestações reconciliadas
    }
dividas-conciliar-procurado = Procurado em { $conta }: valor da prestação ±1% e até 7 dias do vencimento.
dividas-conciliar-erro = Não foi possível reconciliar
dividas-atrasadas-texto =
    { $n ->
        [1] Vencida e sem pagamento registado. Se já pagou, reconcilie com os movimentos da conta de débito ou marque como paga na tabela.
       *[other] Vencidas e sem pagamento registado. Se já pagou, reconcilie com os movimentos da conta de débito ou marque como pagas na tabela.
    }
dividas-marcadas-titulo = Prestações marcadas como pagas
dividas-marcadas-descricao = { $n } prestação(ões) vencida(s), no vencimento e pelo valor do plano.
dividas-de-financiados = de { $valor } financiados
dividas-parcelas-pagas-de = { $pagas } de { $total } prestações pagas
dividas-sistema-pre = taxa fixa
dividas-sistema-pos = taxa variável, juros pelo { $indice } acumulado nos dias úteis
dividas-sistema-pos-defasagem = taxa variável, juros pelo { $indice } acumulado nos dias úteis (D−{ $dias })
dividas-montando-cronograma = A montar o plano de pagamentos
dividas-simulador-amortizacao = Simulador de amortização antecipada
dividas-antecipar-parcelas = Antecipar prestações
dividas-origem-valor = plano do núcleo · ≈ índice estimado
dividas-excluir-titulo = Eliminar { $nome }
dividas-excluir-descricao = O contrato, o plano de pagamentos e as amortizações registadas são removidos. Os movimentos das contas mantêm-se. Pode anular no histórico de alterações.
dividas-excluir-confirmar = Eliminar contrato
dividas-excluido = Contrato eliminado

## Figuras

dividas-grafico-saldo-estimado = Capital em dívida (≈ índice estimado)
dividas-grafico-parcelas-n = Prestações ({ $n })
dividas-grafico-parcela-estimada = Prestação { $n } · { $data } · ≈ índice estimado
dividas-grafico-correcao-nota = A correção não sai na prestação: é somada ao capital em dívida.
dividas-figura-saldo-titulo = Capital em dívida ao longo do contrato
dividas-figura-saldo-fonte = Plano de pagamentos do contrato
dividas-figura-saldo-fonte-indice = Plano de pagamentos do contrato · correção pelo índice (≈ meses sem dado oficial usam o último índice)
dividas-figura-saldo-aria = Capital em dívida de { $nome } ao longo do contrato.
dividas-figura-saldo-aria-simulada = Capital em dívida de { $nome } ao longo do contrato, com a amortização simulada.
dividas-figura-saldo-tabela = Capital em dívida por vencimento
dividas-figura-composicao-titulo =
    { $modo ->
        [ano] Juros × amortização por ano
       *[parcela] Juros × amortização por prestação
    }
dividas-figura-composicao-fonte-correcao = A correção do capital pelo índice não é paga na prestação: entra no capital em dívida e é amortizada depois.
dividas-figura-composicao-fonte = Composição de cada prestação do plano de pagamentos.
dividas-figura-composicao-aria =
    { $modo ->
        [ano] Composição das prestações de { $nome } por ano: { $partes }.
       *[parcela] Composição das prestações de { $nome } por prestação: { $partes }.
    }
dividas-figura-composicao-tabela =
    { $modo ->
        [ano] Composição por ano
       *[parcela] Composição por prestação
    }

## Juros do mercado (Banco Central do Brasil)

dividas-mercado-intro = Quanto os bancos cobram em cada tipo de crédito, pelos dados do Banco Central do Brasil. Serve para saber se o seu contrato está caro e se vale a pena negociar ou transferir a dívida para outro banco (portabilidade).
dividas-mercado-seus-contratos = Os seus contratos × mercado

## Antecipar prestações

dividas-antecipar-dica-ultimas = Liquida as últimas: o contrato termina mais cedo e o desconto é maior.
dividas-antecipar-informe = Indique quantas prestações antecipar.
dividas-antecipar-valor-cronograma = Valor no plano
dividas-antecipadas = Prestações antecipadas
dividas-antecipadas-descricao = { $n } prestação(ões) por { $valor }
dividas-antecipar-registrar = Registar antecipação
dividas-antecipar-com-conta = A saída é registada na conta de débito do contrato.
dividas-antecipar-sem-conta = Sem conta de débito: só as prestações ficam pagas.

## Simulador de amortização antecipada

dividas-simulador-acima-saldo = Acima do capital em dívida ({ $saldo }): liquida o contrato.
dividas-simulador-saldo-hoje = Capital em dívida hoje: { $saldo }
dividas-simulador-informe = Indique um valor para comparar as duas formas de amortizar. A simulação não guarda nada.
dividas-simulador-juros-economizados = Juros poupados
dividas-simulador-parcelas-restantes = Prestações restantes
dividas-simulador-quitacao = Liquidação
dividas-simulador-nota = Simulação pelo plano de pagamentos atual. Confirme os valores com o banco antes de pagar.
dividas-simulador-nota-indice = Simulação pelo plano de pagamentos atual, com o índice projetado pelo último conhecido. Confirme os valores com o banco antes de pagar.
dividas-simulador-registrar = Registar esta amortização

## Simulador de portabilidade

dividas-portabilidade-informe = Indique a taxa oferecida pelo outro banco para comparar. A simulação não guarda nada.
dividas-portabilidade-saldo-levado = Capital transferido
dividas-portabilidade-nota = A nova dívida é calculada sem indexante (taxa fixa). Contratos atuais com TR ou IPCA podem ter um custo real diferente.

## Associar uma prestação a um movimento

dividas-ligar-ok = Prestação { $n } associada ao movimento
dividas-ligar-titulo = Associar a prestação { $n } a um movimento
dividas-ligar-descricao = { $contrato } · vencimento em { $data } · prestação de { $valor }. Saídas até 20 dias do vencimento que ainda não pagam outra prestação, as de valor mais próximo primeiro. A prestação fica paga na data e pelo valor do movimento.
dividas-ligar-trocar = Trocar pelo movimento
dividas-ligar-ao-lancamento = Associar ao movimento
dividas-lancamento-ligado = Movimento associado
dividas-paga-pelo-lancamento = Paga pelo movimento
dividas-desligada = Prestação { $n } desassociada do movimento e em aberto
dividas-desligar-lancamento = Desassociar o movimento
dividas-ligar-procurando = A procurar as saídas perto do vencimento…
dividas-ligar-tabela = Movimentos para a prestação { $n }
dividas-ligar-vazio = Nenhuma saída sem prestação até 20 dias do vencimento. Importe o extrato da conta ou registe o pagamento em Lançamentos.

## Tabela de amortização

dividas-paga-pelo-lancamento-titulo = Paga pelo movimento { $lancamento }
dividas-paga-pelo-lancamento-titulo-conta = Paga pelo movimento { $lancamento } em { $conta }
dividas-ligar-reticencias = Associar…
dividas-desligar-parcela = Desassociar o movimento da prestação { $n }
dividas-desfazer-parcela = Anular o pagamento da prestação { $n }
dividas-desfazer = Anular
dividas-marcar-parcela-aria = Marcar a prestação { $n } como paga
dividas-menu-ligar-outro = Associar a outro movimento…
dividas-menu-ligar-um = Associar a um movimento…
dividas-menu-ver-lancamento = Ver o movimento
dividas-menu-desfazer = Anular o pagamento
dividas-filtro-aria = Prestações a mostrar
dividas-acoes-parcela = Ações da prestação { $n }
dividas-filtro-vazio = Nenhuma prestação neste filtro.

## Marcar prestação como paga e anular

dividas-paga-sincronizada = Prestação { $n } paga e capital sincronizado
dividas-marcada-paga = Prestação { $n } marcada como paga
dividas-pagar-titulo = Prestação { $n } paga
dividas-pagar-descricao = Vencimento em { $data }. Marca a prestação como paga sem movimento; se o pagamento já está em Lançamentos, use “Associar…” na prestação (ou “Reconciliar com os pagamentos” para todas).
dividas-saldo-depois = Capital em dívida depois do pagamento
dividas-saldo-depois-dica = Opcional, do comprovativo: sincroniza o capital e recalcula as próximas prestações.
dividas-amortizacao-parcela = Amortização da prestação
dividas-faltam-dica = Do comprovativo; vazio mantém o prazo atual.
dividas-desligar-descricao = A prestação deixa de estar paga pelo movimento { $lancamento } e volta a ficar em aberto; o movimento continua na conta. Pode anular no histórico de alterações.
dividas-desfazer-descricao = A prestação volta a ficar em aberto. Pode anular no histórico de alterações.
dividas-desfazer-pagamento = Anular pagamento
dividas-em-aberto-de-novo = Prestação { $n } novamente em aberto

## Extrato do contrato

dividas-extrato-importadas = { $n } prestação(ões) do extrato registada(s)
dividas-extrato-projecao = Projeção a partir do capital de { $data }, com { $n } prestação(ões) restante(s).
dividas-extrato-descricao = Cole o histórico do banco (da folha de cálculo ou do homebanking), com o cabeçalho. As prestações ficam com os valores do banco, as amortizações antecipadas são detetadas pelo capital e o plano futuro parte do último capital. Também pode enviar a imagem ou o PDF ao agente.
dividas-extrato-importar-n = Importar { $n } prestação(ões)
dividas-extrato-colado-dica = Colunas: data, prestação, amortização, juros, seguros, comissões, capital em dívida, correção monetária e valor pago (os nomes do cabeçalho bastam; a ordem da folha da Caixa também é reconhecida).
dividas-extrato-faltam-dica = Depois da última linha. Vazio: calculado pela última prestação.
dividas-extrato-marcar-anteriores = Marcar como pagas as prestações anteriores ao extrato
dividas-extrato-previa = Pré-visualização do extrato
dividas-extrato-ultimo-saldo = Último capital
dividas-extrato-extras = Amortizações antecipadas detetadas
dividas-extrato-extras-aria = Amortizações detetadas

## Pagamento de prestação a partir de um movimento

dividas-pagamento-so-saida = Só uma saída (valor negativo) pode pagar uma prestação.
dividas-pagamento-transferencia = É uma transferência entre as suas contas: anule a transferência para a associar à prestação.
dividas-pagamento-previsto = O movimento ainda está previsto: efetive-o antes de o associar à prestação.
dividas-pagamento-ligado-ok = Movimento associado à prestação { $n } de { $contrato }
dividas-pagamento-paga-em = Prestação paga em { $data }.
dividas-pagamento-desligado = Movimento desassociado da prestação { $n } de { $contrato }
dividas-pagamento-voltou = A prestação voltou a ficar em aberto.
dividas-pagamento-este = Este movimento
dividas-pagamento-titulo = Pagamento de prestação
dividas-pagamento-trocar-para = Trocar para a prestação { $n }
dividas-pagamento-ligar-n = Associar à prestação { $n }
dividas-pagamento-ligar = Associar à prestação
dividas-pagamento-ligado = Este movimento paga a prestação { $n } de { $contrato }.
dividas-pagamento-ligado-texto = Escolha outra prestação para trocar, ou desassocie: a prestação volta a ficar em aberto.
dividas-pagamento-carregando-contratos = A carregar os contratos…
dividas-pagamento-sem-contrato = Nenhum contrato ativo. Registe o empréstimo ou crédito em { $tela }.
dividas-pagamento-sem-contrato-moeda = Nenhum contrato ativo em { $moeda }. Registe o empréstimo ou crédito em { $tela }.
dividas-pagamento-carregando-parcelas = A carregar as prestações…
dividas-pagamento-parcelas-de = Prestações de { $nome }
dividas-pagamento-vazio = Nenhuma prestação sem movimento neste contrato.
dividas-pagamento-nota = As 12 prestações sem movimento com vencimento mais perto de { $data }. A escolhida fica paga em { $data } pelo valor do movimento.
dividas-pagamento-nota-categoria = As 12 prestações sem movimento com vencimento mais perto de { $data }. A escolhida fica paga em { $data } pelo valor do movimento, e o movimento recebe a categoria dos pagamentos do contrato.

## Registo e edição do contrato

dividas-form-editar-descricao = Corrija qualquer campo: as prestações em aberto são recalculadas. As prestações pagas (com o valor e a data que registou), as amortizações antecipadas e as antecipações ficam como estão.
dividas-form-novo-descricao = Empréstimo ou crédito: o plano de pagamentos (SAC ou Price, com carência, indexante, seguros e comissões) é gerado na hora.
dividas-form-erro-nome = Dê um nome ao contrato.
dividas-form-erro-principal = Indique o valor financiado.
dividas-form-erro-parcela = Indique o valor da prestação.
dividas-form-erro-parcela-nao-fecha = Com essa prestação e esse prazo a taxa não fecha: a prestação vezes o prazo tem de ultrapassar o valor financiado.
dividas-form-erro-taxa = Indique a taxa efetiva anual.
dividas-form-erro-data-contrato = Indique a data do contrato.
dividas-form-erro-primeiro-vencimento = Indique o primeiro vencimento.
dividas-form-erro-vencimento-antes = A primeira prestação não pode vencer antes do contrato.
dividas-form-erro-prazo = Indique o prazo em meses.
dividas-form-erro-bullet = No bullet não há carência: a prestação já é só juros.
dividas-form-cadastrado = Contrato registado
dividas-form-cadastrado-descricao = { $nome }: { $n } prestações no plano
dividas-form-nome-exemplo = Crédito à habitação do apartamento
dividas-form-numero-dica = Como aparece no contrato ou no homebanking. Opcional.
dividas-form-pre = Fixa
dividas-form-pos = Variável (CDI, Selic)
dividas-form-spread-dica = Taxa acima do índice: { $indice } + spread. Deixe vazio se for só a percentagem.
dividas-form-percentual = Percentagem do índice
dividas-form-erro-percentual = Percentagem do índice entre 0 e 1000%.
dividas-form-pela-parcela = Calcular pela prestação
dividas-form-informar-taxa = Indicar a taxa
dividas-form-primeira-parcela = Primeira prestação
dividas-form-parcela-juros = Prestação de juros
dividas-form-valor-parcela = Valor da prestação
dividas-form-parcela-dica-encargos = Como na fatura ou no extrato: os seguros e comissões por mês (abaixo) saem daí.
dividas-form-parcela-dica = Como na fatura ou no extrato, com seguros e comissões (indique-os abaixo).
dividas-form-nao-informar = Não indicar
dividas-form-correcao-dica = Correção monetária mensal do capital (TR no SFH, IPCA ou INCC no imobiliário).
dividas-form-deflacao-dica = Meses com índice negativo não reduzem o capital (piso zero).
dividas-form-mip-taxa-dica = Morte e invalidez, sobre o capital em dívida: desce juntamente com a dívida.
dividas-form-mip-taxa-dica-hoje = Morte e invalidez, sobre o capital em dívida: desce juntamente com a dívida. Hoje ≈ { $valor } por mês.
dividas-form-mip-valor-dica = Valor fixo de morte e invalidez em cada prestação, como no contrato.
dividas-form-mip-valor-dica-equivale = Valor fixo de morte e invalidez em cada prestação, como no contrato. Equivale a { $pct }% a.m. do capital de { $saldo }.
dividas-form-dfi-valor-dica = Valor fixo de danos físicos em cada prestação (não precisa do valor do imóvel).
dividas-form-dfi-valor-dica-equivale = Valor fixo de danos físicos em cada prestação (não precisa do valor do imóvel). Equivale a { $pct }% a.m. do imóvel.
dividas-form-taxa-adm = Comissão de gestão por mês
dividas-form-taxa-adm-dica = Comissão mensal do contrato (no crédito imobiliário brasileiro costuma ser R$ 25).
dividas-form-conta-dica = Para reconciliar as prestações com os movimentos.
dividas-form-categoria-dica = Um movimento nesta categoria (importado ou introduzido à mão) marca a prestação mais próxima como paga.
dividas-form-ja-paguei =
    { $n ->
        [1] Já paguei a { $n } prestação vencida até hoje
       *[other] Já paguei as { $n } prestações vencidas até hoje
    }
dividas-form-ja-paguei-dica = Ficam pagas no vencimento, pelo valor do plano. Desligue para marcar uma a uma (ou reconciliar com os movimentos da conta).
dividas-form-pos-nota = A prestação varia com o { $indice }: ao guardar, o plano usa as taxas já publicadas e, para o futuro, a última conhecida (prestações marcadas como estimadas e refeitas a cada recolha de índices).
dividas-form-ultima-parcela = Última prestação
dividas-form-correcao-valor = + { $indice } no capital
dividas-form-salvar = Guardar alterações
dividas-form-cadastrar = Registar contrato

## Amortização antecipada

dividas-amortizacao-extraordinaria = Amortização antecipada
dividas-extra-descricao = { $nome }: o pagamento abate o capital em dívida e o plano em aberto é recalculado. Pode anular no histórico de alterações.
dividas-extra-erro-valor = Indique o valor.
dividas-extra-erro-data = Indique a data (dd/mm/aaaa).
dividas-extra-registrada = Amortização registada
dividas-extra-registrada-descricao = { $nome }: capital em dívida e plano recalculados
dividas-extra-sem-conta-dica = Regista só no contrato, sem movimento (ex.: FGTS ou pagamento por outra pessoa).
dividas-extra-com-conta-dica = Cria o movimento de saída nessa conta.
dividas-extra-sem-lancamento = Sem movimento (FGTS, outra origem)
dividas-extra-data-passada = Data passada ({ $data }): entra depois da última prestação vencida até lá.
dividas-extra-registrar = Registar amortização
