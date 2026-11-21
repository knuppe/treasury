# Relatórios (src/features/reports): os controles usados também nas telas principais (mês,
# competência × caixa, carregando, erro) e o "Explicar" de cada valor (docs/i18n-design.md).
# pt-BR é o idioma de origem.

## Controles

relatorios-competencia = Competência
relatorios-competencia-dica = Pela data da compra
relatorios-caixa = Caixa
relatorios-caixa-dica = Pela data do pagamento (vencimento da fatura no cartão)
relatorios-base-datas = Base de datas
relatorios-mes-anterior = Mês anterior
relatorios-proximo-mes = Próximo mês
relatorios-falha-carregar = Falha ao carregar.
relatorios-erro-carregar = Erro ao carregar
relatorios-tentar-de-novo = Tentar de novo
relatorios-sem-dados-periodo = Sem dados no período

## Explicar um valor

relatorios-explicar = Explicar { $titulo }
relatorios-explicar-dica = Explicar: { $titulo }
relatorios-explicacao = Explicação
relatorios-explicacao-de = Explicação · { $titulo }
relatorios-ver-lancamentos = Ver lançamentos
relatorios-calculando-origem = Calculando a origem
relatorios-fontes = Fontes
relatorios-itens = { $n ->
        [1] { $n } item
       *[other] { $n } itens
    }
relatorios-mostrando = mostrando { $n }
relatorios-formula = Fórmula
relatorios-inclui-estimado = ≈ Inclui valor estimado (câmbio ou cotação do dia anterior).
relatorios-sem-cambio = Sem câmbio para { $moedas }: esses valores ficaram fora do total.
relatorios-fontes-valor = Fontes do valor
relatorios-nenhuma-fonte = Nenhuma fonte: o valor é zero.
relatorios-lista-recentes = A lista mostra os { $n } lançamentos mais recentes; a soma completa está na fórmula.
relatorios-descricao = Descrição
relatorios-taxa = taxa
relatorios-fonte-lancamento = Lançamento
relatorios-fonte-conta = Conta
relatorios-fonte-cambio = Câmbio
relatorios-fonte-saldo-inicial = Saldo inicial
relatorios-fonte-indice = Índice
relatorios-fonte-carteira = Carteira
relatorios-fonte-divida = Dívida
relatorios-fonte-bem-fisico = Bem físico

## Tela Relatórios

relatorios-descricao-tela = Despesas em todas as formas: para onde foi, o que pesa, quando, se está subindo, onde estourou e quanto já está comprometido. Clique em qualquer pedaço para ver os lançamentos.
relatorios-descricao-ir = Apuração de IR em renda variável: imposto do mês, DARF e os valores para a declaração anual.
relatorios-descricao-saude = Saúde financeira: uma nota de 0 a 100 calculada com os seus dados, as dimensões, cada indicador × meta e o que melhorar.
relatorios-imposto-renda = Imposto de renda
relatorios-saude-financeira = Saúde financeira
relatorios-visao-frente = Visão à frente
relatorios-visualizacoes = Visualizações
relatorios-aba-aria = { $vista }: { $pergunta }
relatorios-aba-ir-sigla = IR
relatorios-aba-ir-dica = Apuração de IR, DARF e declaração
relatorios-aba-ir-aria = Imposto de renda: apuração, DARF e declaração
relatorios-aba-saude-sigla = SF
relatorios-aba-saude-dica = Nota de saúde financeira, dimensões e sugestões
relatorios-aba-saude-aria = Saúde financeira: nota, indicadores e sugestões

## Visualizações (aba e pergunta)

relatorios-vista-fluxo = Fluxo
relatorios-vista-fluxo-pergunta = Para onde vai a receita?
relatorios-vista-composicao = Composição
relatorios-vista-composicao-pergunta = O que pesa mais no orçamento?
relatorios-vista-ritmo = Ritmo
relatorios-vista-ritmo-pergunta = Em que dias e semanas gastamos mais?
relatorios-vista-tendencia = Tendência
relatorios-vista-tendencia-pergunta = Os gastos estão subindo? Em quê?
relatorios-vista-orcamento = Orçamento
relatorios-vista-orcamento-pergunta = Onde estouramos?
relatorios-vista-comparacao = Comparação
relatorios-vista-comparacao-pergunta = O que mudou de um período para outro?
relatorios-vista-inflacao = Inflação
relatorios-vista-inflacao-pergunta = Gastamos mais ou só ficou mais caro?
relatorios-vista-titulares = Membros
relatorios-vista-titulares-pergunta = Quem gasta em quê?
relatorios-vista-recorrentes = Recorrentes
relatorios-vista-recorrentes-pergunta = Quanto está comprometido todo mês?
relatorios-vista-cartoes = Cartões
relatorios-vista-cartoes-pergunta = Quanto já está comprometido nos próximos meses?
relatorios-vista-patrimonio = Patrimônio
relatorios-vista-patrimonio-pergunta = O patrimônio está crescendo?

## Períodos e filtros

relatorios-periodo-mes = Este mês
relatorios-periodo-mes-anterior = Mês passado
relatorios-periodo-trimestre = Este trimestre
relatorios-periodo-trimestre-anterior = Trimestre passado
relatorios-periodo-3m = Últimos 3 meses
relatorios-periodo-6m = Últimos 6 meses
relatorios-periodo-ano = Este ano
relatorios-periodo-12m = Últimos 12 meses
relatorios-periodo-ano-anterior = Ano passado
relatorios-periodo-personalizado = Personalizado
# Período de vários meses ou de datas soltas: "janeiro de 2026 a março de 2026".
relatorios-periodo-intervalo = { $de } a { $ate }
relatorios-filtros = Filtros dos relatórios
relatorios-periodo = Período
relatorios-periodo-anterior = Período anterior
relatorios-proximo-periodo = Próximo período
relatorios-de = De
relatorios-ate = Até
relatorios-data-inicial = Data inicial
relatorios-data-final = Data final
relatorios-base = Base
relatorios-membro = Membro
relatorios-conta = Conta
relatorios-todos = Todos
relatorios-todas = Todas
# Base de datas no meio da linha de fonte ("Fonte: … · caixa · …").
relatorios-base-caixa = caixa
relatorios-base-competencia = competência
relatorios-filtro-nota-membro-conta = Esta figura mostra a família toda: o filtro de membro e conta vale para a Composição.
relatorios-filtro-nota-membro = Esta figura mostra a família toda: o filtro de membro vale para a Composição.
relatorios-filtro-nota-conta = Esta figura mostra a família toda: o filtro de conta vale para a Composição.

## Peças comuns das figuras

relatorios-outras = Outras
relatorios-sem-categoria = Sem categoria
relatorios-direto = { $categoria } (direto)
relatorios-categoria = Categoria
relatorios-receita = Receita
relatorios-receitas = Receitas
relatorios-despesas = Despesas
relatorios-total = Total
relatorios-mes = Mês
relatorios-grupo = Grupo
relatorios-forma = Forma
relatorios-novo = novo
relatorios-nenhum = nenhum
relatorios-pct-receita = % da receita
relatorios-pct-total = % do total
relatorios-da-receita = Da receita
relatorios-do-total = { $pct } do total
relatorios-item-em = { $nome } em { $periodo }
relatorios-receitas-de = Receitas de { $periodo }
relatorios-despesas-de = Despesas de { $periodo }
relatorios-fonte-sem-cambio = { $fonte } · sem câmbio: { $moedas }
relatorios-fonte-despesas = Fonte: lançamentos de despesa efetivados · { $base }
relatorios-fonte-despesas-filtro = Fonte: lançamentos de despesa efetivados · { $base } · com filtro de membro/conta

## 01 · Fluxo

relatorios-carregando-fluxo = Traçando o fluxo
relatorios-detalhe = Detalhe
relatorios-detalhe-categorias = Categorias
relatorios-detalhe-categorias-dica = Só as categorias de primeiro nível
relatorios-detalhe-subcategorias = Subcategorias
relatorios-detalhe-subcategorias-dica = Categorias e subcategorias
relatorios-grupo-in = Entradas
relatorios-grupo-out = Saídas
relatorios-grupo-result = Resultado
relatorios-vs-anterior = vs anterior:
relatorios-periodo-anterior-dica = Período anterior: { $periodo }
relatorios-ver-compoe-dica = Ver os lançamentos que compõem este valor
relatorios-ver-compoe-sr = (ver o que compõe { $item })
relatorios-pontos = { $valor } p.p.
relatorios-para = Para
relatorios-anterior = Anterior
relatorios-o-periodo-anterior = o período anterior
relatorios-diferenca-de = a diferença de { $nome }
relatorios-sobra = Sobra
relatorios-deficit = Déficit
relatorios-fluxo-pergunta = Para onde vai a receita: o que entra (receitas e resgates) passa pelo orçamento e sai em despesas por categoria, pagamento de dívidas, investimentos e a sobra.
relatorios-fluxo-pergunta-comparado = Para onde vai a receita: o que entra (receitas e resgates) passa pelo orçamento e sai em despesas por categoria, pagamento de dívidas, investimentos e a sobra, comparado com { $periodo }.
relatorios-taxa-poupanca = Taxa de poupança
relatorios-taxa-poupanca-de = Taxa de poupança de { $periodo }
relatorios-no-periodo = no período
relatorios-investimentos-aportes = Investimentos (aportes) { $valor }
relatorios-resgates = Resgates { $valor }
relatorios-pagamento-dividas = Pagamento de dívidas { $valor } (dentro das despesas)
relatorios-fluxo-em-andamento = O período ainda está em andamento (até hoje) e é comparado com { $periodo } inteiro. Para comparar períodos fechados, use as setas ou escolha “Mês passado”.
relatorios-fluxo-titulo = Fluxo · { $periodo }
relatorios-fluxo-fonte = Fonte: lançamentos efetivados · { $base } · transferências entre contas e pagamento de fatura não entram; aportes e resgates à parte
# $investimentos e $sobra: "sim" ou "nao".
relatorios-fluxo-aria = Fluxo de { $periodo }: receitas divididas entre { $n } categorias de despesa{ $investimentos ->
        [sim] , investimentos
       *[nao] {""}
    }{ $sobra ->
        [sim] {" "}e a sobra
       *[nao] , com déficit
    }.
relatorios-sem-movimento = Sem movimento no período
relatorios-fluxo-tabela = Fluxo: origem, destino e valor
relatorios-comparativo-titulo = Comparativo · { $anterior } × { $atual }
relatorios-comparativo-tabela = Comparativo do fluxo: { $anterior } e { $atual }
relatorios-comparativo-dica = Clique num valor para ver os lançamentos que o compõem (no mês, no anterior ou na diferença).

## 02 · Composição

relatorios-carregando-composicao = Somando as categorias
relatorios-composicao-pergunta = O que pesa mais: cada bloco é uma categoria, dividida nas subcategorias, do tamanho do gasto.
relatorios-maior-categoria = Maior categoria
relatorios-peso-maior = Peso da maior
relatorios-composicao-titulo = Composição · { $periodo }
relatorios-forma-treemap = Treemap
relatorios-forma-sunburst = Sunburst
relatorios-composicao-aria = Composição das despesas de { $periodo }: { $maiores } são as maiores.
relatorios-composicao-tabela = Composição por categoria

## 03 · Ritmo

relatorios-carregando-ritmo = Montando o calendário
relatorios-dia = Dia
relatorios-semana = Semana
relatorios-ritmo-pergunta = Em que dias e semanas gastamos mais: um quadrado por dia, mais escuro quanto maior o gasto.
relatorios-despesas-12-meses = Despesas em 12 meses
relatorios-media-por-dia = Média por dia
relatorios-dia-mais-caro = Dia mais caro
relatorios-dias-sem-gasto = Dias sem gasto
relatorios-ritmo-titulo = Ritmo · { $periodo }
relatorios-ritmo-fonte = Fonte: lançamentos de despesa efetivados · { $base } · faixas por quantis dos dias com gasto
relatorios-ritmo-aria = Calendário de gastos de { $periodo }; o dia mais caro foi { $dia }.
relatorios-media-dia-semana-aria = Média de gasto por dia da semana
relatorios-media-dia-semana = Média por dia da semana
relatorios-dias-maior-gasto = Dias de maior gasto
relatorios-dia-semana-data = { $semana }, { $data }
relatorios-gasto-por-dia = Gasto por dia

## 04 · Tendência

relatorios-carregando-tendencia = Somando os meses
relatorios-media-3m = Média 3m
relatorios-media-movel-3m = Média móvel (3 meses)
relatorios-tendencia-pergunta = Os gastos estão subindo? Em quê? Barras por categoria, a média móvel de 3 meses e as receitas como referência.
relatorios-media-movel-ultimo = Média móvel (último mês)
relatorios-variacao-media-movel = Variação da média móvel
relatorios-contra = contra { $mes }
relatorios-media-meses = Média de { $n } meses
relatorios-receitas-periodo = Receitas no período
relatorios-tendencia-titulo = Tendência · { $n } meses até { $mes }
relatorios-tendencia-fonte = Fonte: lançamentos de despesa e receita efetivados · { $base } · média móvel do mês e dos dois anteriores
relatorios-tendencia-aria = Despesas por categoria em { $n } meses até { $mes }, com média móvel de 3 meses e receitas.
relatorios-tendencia-tabela = Despesas por mês e categoria

## 05 · Orçamento

relatorios-orcamento-vazio = Sem orçamento nem gasto em { $mes }
relatorios-montar-orcamento = Montar o orçamento
relatorios-previsto = Previsto
relatorios-realizado = Realizado
relatorios-comprometido = Comprometido
relatorios-restante = Restante
relatorios-situacao = Situação
relatorios-orcamento-pergunta = Onde estouramos: previsto (marca), realizado (barra), comprometido (trecho claro) e o ritmo do mês (traço fino).
relatorios-estouradas = Estouradas
relatorios-mes-decorrido = Mês decorrido
relatorios-orcamento-titulo = Orçamento · { $mes }
relatorios-orcamento-fonte = Fonte: orçamento do mês e lançamentos de despesa · { $base } · o realizado inclui as subcategorias
relatorios-orcamento-aria = Previsto × realizado de { $mes }
relatorios-clique-nome = Clique no nome para ver os lançamentos
relatorios-sem-previsto = sem previsto
relatorios-orcamento-tabela = Previsto × realizado

## 06 · Comparação

relatorios-carregando-comparacao = Comparando os períodos
relatorios-comparacao-pergunta = O que mudou: a variação de cada categoria entre { $a } (A) e { $b } (B). Gastar mais aparece com ▲.
relatorios-comparacao-a = A · { $periodo }
relatorios-comparacao-b = B · { $periodo }
relatorios-variacao-pct = Variação %
relatorios-comparar-com = Comparar com
relatorios-um-ano-antes = Um ano antes
relatorios-comparacao-em-andamento = B ainda está em andamento (até hoje): parte das despesas do período ainda não aconteceu. Para comparar meses fechados, escolha “Mês passado”.
relatorios-comparacao-titulo = Comparação · { $a } × { $b }
relatorios-comparacao-fonte = Fonte: lançamentos de despesa efetivados · { $base } · categorias de primeiro nível
relatorios-forma-barras = Barras
relatorios-forma-slope = Slope
relatorios-comparacao-aria = Variação do gasto por categoria entre { $a } e { $b }.
relatorios-comparacao-tabela = Comparação por categoria

## 07 · Inflação (lugar reservado)


## 08 · Membros

relatorios-carregando-membros = Separando por membro
relatorios-membros-pergunta = Quem gasta em quê: o gasto de cada membro por categoria (a parte da divisão conta para o membro da parte).
relatorios-membros-titulo = Membros · { $periodo }
relatorios-membros-fonte = Fonte: lançamentos de despesa efetivados · { $base } · { $n } categorias
relatorios-membros-fonte-outras = Fonte: lançamentos de despesa efetivados · { $base } · { $n } categorias ({ $outras } em "Outras")
relatorios-membros-aria = Gasto por membro e categoria em { $periodo }.
relatorios-membros-tabela = Gasto por membro e categoria

## 09 · Recorrentes

relatorios-sem-recorrencias = Sem recorrências
relatorios-sem-recorrencias-titulo = Nenhuma assinatura ou conta fixa cadastrada
relatorios-sem-recorrencias-descricao = Cadastre as recorrências (aluguel, escola, assinaturas) na tela de lançamentos: elas geram os previstos e aparecem aqui.
relatorios-carregando-recorrencias = Lendo as recorrências
relatorios-regra = Regra
relatorios-proximas = Próximas
relatorios-recorrentes-pergunta = Quanto já está comprometido todo mês com assinaturas e contas fixas, e quando cada uma vence.
relatorios-comprometido-mes = Comprometido por mês
relatorios-receitas-recorrentes = Receitas recorrentes
relatorios-das-receitas-recorrentes = Das receitas recorrentes
relatorios-despesas-recorrentes = Despesas recorrentes
relatorios-recorrentes-titulo = Recorrentes · próximos { $n } dias
relatorios-recorrentes-fonte = Fonte: recorrências ativas (regra e próximas datas) · valor mensal equivalente pela frequência
relatorios-recorrentes-aria = Linha do tempo das recorrências nos próximos { $n } dias
relatorios-recorrencias = Recorrências

## 10 · Cartões

relatorios-carregando-cartoes = Somando as faturas
relatorios-sem-cartoes = Sem cartões
relatorios-sem-cartoes-titulo = Nenhum cartão de crédito cadastrado
relatorios-sem-cartoes-descricao = Com cartões e compras parceladas cadastrados, esta figura mostra quanto já vence em cada mês.
relatorios-fatura = Fatura
relatorios-cartoes-pergunta = Quanto já está comprometido nos próximos meses: o que falta pagar de cada fatura, com as parcelas futuras.
relatorios-proximos-meses = Próximos { $n } meses
relatorios-a-pagar-em = A pagar em { $mes }
relatorios-proxima-fatura = Próxima fatura
relatorios-cartoes-titulo = Cartões · faturas a partir de { $mes }
relatorios-cartoes-fonte = Fonte: faturas não pagas e parcelas previstas · mês de vencimento da fatura · valores na moeda base
relatorios-cartoes-aria = Faturas futuras dos cartões nos próximos { $n } meses.
relatorios-cartoes-tabela = Faturas futuras por cartão
relatorios-total-faturas = Total das faturas

## 11 · Patrimônio

relatorios-carregando-patrimonio = Medindo o patrimônio
relatorios-contas = Contas
relatorios-investimentos = Investimentos
relatorios-bens = Bens
relatorios-dividas = Dívidas
relatorios-liquido = Líquido
relatorios-patrimonio-liquido = Patrimônio líquido
relatorios-patrimonio-pergunta = O patrimônio está crescendo? Contas, investimentos e bens acima do zero, dívidas abaixo, e o líquido na linha.
relatorios-liquido-em = Líquido em { $mes }
relatorios-patrimonio-liquido-em = Patrimônio líquido em { $data }
relatorios-explicar-patrimonio = Explicar patrimônio líquido em { $data }
relatorios-variacao-desde = Variação desde { $mes }
relatorios-patrimonio-titulo = Patrimônio · { $n } meses até { $mes }
# $estimado e $semcambio: "sim" ou "nao".
relatorios-patrimonio-fonte = Fonte: saldos no último dia de cada mês + valor de mercado da carteira + valor dos bens{ $estimado ->
        [sim] {" "}· ≈ inclui câmbio ou cotação estimados
       *[nao] {""}
    }{ $semcambio ->
        [sim] {" "}· sem câmbio: { $moedas }
       *[nao] {""}
    } · clique num mês para ver a origem
relatorios-patrimonio-aria = Patrimônio líquido em { $n } meses até { $mes }: contas, investimentos e dívidas.
relatorios-patrimonio-tabela = Patrimônio por mês

## Analítico de uma linha do fluxo

relatorios-analitico = Analítico
relatorios-analitico-de = Analítico · { $titulo }
relatorios-carregando-lancamentos = Somando os lançamentos
relatorios-diferenca-delta = Diferença (Δ)
relatorios-agrupar-categoria-dica = Por categoria e subcategoria
relatorios-agrupar-descricao-dica = Pela descrição do lançamento
relatorios-dividido-dica = Lançamento dividido: a categoria de cada parte se troca no editor
relatorios-dividido = · dividido
relatorios-categoria-de = Categoria de { $nome }
relatorios-categoria-alterada = Categoria alterada
relatorios-relatorio-recalculado = { $nome }: o relatório foi recalculado.
relatorios-categoria-nao-mudou = Não foi possível mudar a categoria
relatorios-data = Data
relatorios-linha = Linha
relatorios-editar-item = Editar { $nome }
relatorios-editar-dica = Troque a categoria na própria linha: o relatório recalcula na hora. "Editar" (ou duplo clique) abre o lançamento completo.
relatorios-parte-dividido = parte de um lançamento dividido
relatorios-fatura-vence = fatura vence { $data }
relatorios-soma-confere = { $n ->
        [1] A soma do item
       *[other] A soma dos { $n } itens
    } é { $soma }, igual ao valor de { $periodo } na tabela.
relatorios-soma-difere = A soma dos itens ({ $soma }) difere do valor da tabela ({ $valor }). Recarregue o relatório.
relatorios-cambio-estimado = ≈ Inclui valor convertido com câmbio estimado.
relatorios-por-categoria-em = Por categoria em { $periodo }
relatorios-o-que-compoe = O que compõe { $nome } em { $periodo }
relatorios-nada-em = Nada em { $periodo }.
relatorios-abre-analitico = Enter ou duplo clique numa linha abre o analítico dela.
relatorios-fora-total = Fora do total
relatorios-fora-total-n = Fora do total ({ $n })
relatorios-fora-total-de = Fora do total de { $nome } em { $periodo }
relatorios-por-que-fora = Por que fica fora
relatorios-diferenca-resumo = De { $anterior } ({ $antes }) para { $atual } ({ $depois }): { $diferenca }. Cada linha abaixo mostra quanto ela mudou; a soma das variações é a diferença da tabela.
relatorios-agrupar-por = Agrupar por
relatorios-agrupar-diferenca = Agrupar a diferença por
relatorios-diferenca-por-categoria = Diferença de { $nome } por categoria
relatorios-diferenca-por-descricao = Diferença de { $nome } por descrição
relatorios-nada-mudou = Nada mudou entre os dois períodos.
relatorios-lancamentos-de = Lançamentos de { $nome }
relatorios-lancamentos-dois-periodos = Lançamentos de { $nome } nos dois períodos
relatorios-linhas-dois-periodos = Linhas de { $nome } nos dois períodos
relatorios-nenhum-lancamento = Nenhum lançamento.

## Comandos da barra ⌘K (as palavras de busca vão separadas por vírgula)

relatorios-cmd-grupo-busca = relatório,gráfico
relatorios-cmd-fluxo = Relatório: fluxo do mês
relatorios-cmd-fluxo-busca = sankey,para onde foi,para onde vai,receitas,despesas,sobra
relatorios-cmd-fluxo-trimestre = Relatório: fluxo do trimestre × trimestre anterior
relatorios-cmd-fluxo-trimestre-busca = sankey,receita,comparar,investimentos,dívidas
relatorios-cmd-fluxo-anterior = Relatório: fluxo do mês passado
relatorios-cmd-fluxo-anterior-busca = sankey
relatorios-cmd-composicao = Relatório: composição dos gastos
relatorios-cmd-composicao-busca = treemap,sunburst,categorias,pesa
relatorios-cmd-ritmo = Relatório: calendário de gastos
relatorios-cmd-ritmo-busca = heatmap,dias,semana,ritmo
relatorios-cmd-tendencia = Relatório: tendência dos gastos (12 meses)
relatorios-cmd-tendencia-busca = evolução,média móvel,subindo
relatorios-cmd-orcamento = Relatório: previsto × realizado
relatorios-cmd-orcamento-busca = bullet,estouramos,orçamento
relatorios-cmd-comparacao = Relatório: comparar com o mês anterior
relatorios-cmd-comparacao-busca = a × b,variação,mudou
relatorios-cmd-titulares = Relatório: gastos por membro da família
relatorios-cmd-titulares-busca = membros,família,quem gasta
relatorios-cmd-recorrentes = Relatório: assinaturas e contas fixas
relatorios-cmd-recorrentes-busca = recorrências,comprometido,mensal
relatorios-cmd-cartoes = Relatório: faturas futuras dos cartões
relatorios-cmd-cartoes-busca = parcelas,parcelamentos,cartão
relatorios-cmd-patrimonio = Relatório: patrimônio em 24 meses
relatorios-cmd-patrimonio-busca = ativos,passivos,dívidas,investimentos
relatorios-cmd-inflacao = Relatório: gasto corrigido pela inflação
relatorios-cmd-inflacao-busca = ipca,real,nominal
relatorios-cmd-ir-mes = IR: apuração do mês e DARF
relatorios-cmd-ir-mes-busca = imposto de renda,darf,ganho de capital,day trade,isenção,20 mil
relatorios-cmd-ir-ano = IR: relatório para a declaração anual
relatorios-cmd-ir-ano-busca = imposto de renda,irpf,bens e direitos,rendimentos isentos,declaração
relatorios-cmd-impostos-ano = Impostos pagos no ano (estimativa)
relatorios-cmd-impostos-ano-busca = carga tributária,tributos,consumo,icms,inss,cenários
relatorios-cmd-impostos-grupos = Impostos: grupos das categorias
relatorios-cmd-impostos-grupos-busca = alíquota própria,grupo de imposto,categorias,mapeamento
relatorios-cmd-divida-nova = Cadastrar empréstimo ou financiamento
relatorios-cmd-divida-nova-busca = dívida,contrato,sac,price,consignado,financiamento,cdc
relatorios-cmd-bem-novo = Cadastrar imóvel, veículo ou bem
relatorios-cmd-bem-novo-busca = bem,patrimônio,imóvel,apartamento,casa,carro,terreno,fipe
relatorios-cmd-bens = Ver bens (patrimônio físico)
relatorios-cmd-bens-busca = bens,imóveis,veículos,benfeitorias,avaliação,valorização
relatorios-cmd-orcamento-mes = Orçamento do mês
relatorios-cmd-orcamento-mes-busca = previsto,realizado,budget
relatorios-cmd-metas = Metas
relatorios-cmd-metas-busca = objetivos,viagem,quitação,progresso
relatorios-cmd-reserva = Calculadora de reserva de emergência
relatorios-cmd-reserva-busca = reserva,emergência,colchão
relatorios-cmd-sem-categoria = Lançamentos sem categoria deste mês
relatorios-cmd-sem-categoria-busca = categorizar,revisar,sem categoria

## 12 · Grupos (para onde vai a renda)

relatorios-vista-grupos = Grupos
relatorios-vista-grupos-pergunta = Quanto da renda vai para essenciais, dívidas, futuro e o resto?
relatorios-cmd-grupos = Relatório: para onde vai a renda (grupos)
relatorios-cmd-grupos-busca = grupos,essenciais,dívidas,futuro,não essencial,50/30/20,renda
relatorios-grupos-pergunta = Para onde vai a renda: quanto de cada mês foi para os essenciais, as dívidas, o futuro (aportes e metas) e o não essencial, contra as metas de cada grupo.
relatorios-grupos-carregando = Calculando os grupos…
relatorios-grupos-renda = Renda de { $periodo }
relatorios-grupos-stat-dica = { $valor } · meta { $meta }
relatorios-grupos-stat-dica-sem-meta = { $valor } · sem meta
relatorios-grupos-titulo = Para onde vai a renda: { $n } meses até { $mes }
relatorios-grupos-fonte = Fonte: lançamentos efetivados, aportes e transferências para metas · { $base }
relatorios-grupos-medida = Medida
relatorios-grupos-medida-pct = % da renda
relatorios-grupos-medida-valor = Valor
relatorios-grupos-forma-barras = Barras por mês
relatorios-grupos-forma-linhas = Evolução
relatorios-grupos-aria-barras = Grupos de gasto por mês, { $n } meses até { $mes }
relatorios-grupos-aria-linhas = Parte da renda de cada grupo, { $n } meses até { $mes }
relatorios-grupos-tabela = Grupos de gasto por mês
relatorios-grupos-maiores-titulo = Maiores categorias de cada grupo em { $periodo }
relatorios-grupos-maiores-tabela = Maiores categorias por grupo
relatorios-grupos-grupo = Grupo
relatorios-grupos-pct-grupo = % do grupo
relatorios-grupos-pct-renda = % da renda
relatorios-grupos-meta = Meta
relatorios-grupos-sem-renda = Sem receitas no período: a parte da renda fica em branco.
relatorios-grupos-nota-futuro = Futuro soma os aportes e as transferências para contas de metas; resgates não abatem.
relatorios-grupos-metas-link = Editar metas
relatorios-grupos-categorias-link = Ajustar grupos
relatorios-grupos-composicao-link = Ver quanto da renda vai para cada grupo em 12 · Grupos
relatorios-fluxo-fatura-competencia = Em competência, as compras das faturas pagas neste período ({ $valor }) contam no mês em que foram feitas, enquanto resgates e aportes contam no mês do dinheiro. Em Caixa, a fatura aparece aqui, aberta nas categorias das compras.
