# Variante pt-PT (opcional) dos relatórios (src/features/reports): o que faltar cai para pt-BR
# (docs/i18n-design.md). Só ficam aqui as mensagens que mudam em relação ao pt-BR.
# Revisão: rascunho por IA, ainda sem revisão de um falante nativo.

relatorios-tentar-de-novo = Tentar novamente
relatorios-ver-lancamentos = Ver movimentos
relatorios-calculando-origem = A calcular a origem
relatorios-lista-recentes = A lista mostra os { $n } movimentos mais recentes; a soma completa está na fórmula.
relatorios-fonte-lancamento = Movimento
relatorios-descricao-tela = Despesas em todas as formas: para onde foi, o que pesa, quando, se está a subir, onde ultrapassou e quanto já está comprometido. Clique em qualquer pedaço para ver os movimentos.
relatorios-vista-orcamento-pergunta = Onde ultrapassámos?
relatorios-vista-inflacao-pergunta = Gastámos mais ou só ficou mais caro?
relatorios-vista-patrimonio = Património
relatorios-vista-patrimonio-pergunta = O património está a crescer?
relatorios-fonte-despesas = Fonte: movimentos de despesa efetivados · { $base }
relatorios-fonte-despesas-filtro = Fonte: movimentos de despesa efetivados · { $base } · com filtro de membro/conta
relatorios-carregando-fluxo = A traçar o fluxo
relatorios-ver-compoe-dica = Ver os movimentos que compõem este valor
relatorios-deficit = Défice
relatorios-fluxo-fonte = Fonte: movimentos efetivados · { $base } · transferências entre contas e pagamento de fatura não entram; aportes e resgates à parte
relatorios-comparativo-dica = Clique num valor para ver os movimentos que o compõem (no mês, no anterior ou na diferença).
relatorios-carregando-composicao = A somar as categorias
relatorios-carregando-ritmo = A montar o calendário
relatorios-ritmo-fonte = Fonte: movimentos de despesa efetivados · { $base } · faixas por quantis dos dias com gasto
relatorios-carregando-tendencia = A somar os meses
relatorios-tendencia-fonte = Fonte: movimentos de despesa e receita efetivados · { $base } · média móvel do mês e dos dois anteriores
relatorios-montar-orcamento = Criar o orçamento
relatorios-orcamento-pergunta = Onde ultrapassámos: previsto (marca), realizado (barra), comprometido (trecho claro) e o ritmo do mês (traço fino).
relatorios-estouradas = Ultrapassadas
relatorios-orcamento-fonte = Fonte: orçamento do mês e movimentos de despesa · { $base } · o realizado inclui as subcategorias
relatorios-clique-nome = Clique no nome para ver os movimentos
relatorios-carregando-comparacao = A comparar os períodos
relatorios-comparacao-fonte = Fonte: movimentos de despesa efetivados · { $base } · categorias de primeiro nível
relatorios-carregando-membros = A separar por membro
relatorios-membros-fonte = Fonte: movimentos de despesa efetivados · { $base } · { $n } categorias
relatorios-membros-fonte-outras = Fonte: movimentos de despesa efetivados · { $base } · { $n } categorias ({ $outras } em "Outras")
relatorios-sem-recorrencias-titulo = Nenhuma assinatura ou conta fixa registada
relatorios-sem-recorrencias-descricao = Registe as recorrências (renda, escola, assinaturas) no ecrã de movimentos: elas geram os previstos e aparecem aqui.
relatorios-carregando-recorrencias = A ler as recorrências
relatorios-carregando-cartoes = A somar as faturas
relatorios-sem-cartoes-titulo = Nenhum cartão de crédito registado
relatorios-sem-cartoes-descricao = Com cartões e compras a prestações registados, esta figura mostra quanto já vence em cada mês.
relatorios-cartoes-pergunta = Quanto já está comprometido nos próximos meses: o que falta pagar de cada fatura, com as prestações futuras.
relatorios-cartoes-fonte = Fonte: faturas não pagas e prestações previstas · mês de vencimento da fatura · valores na moeda base
relatorios-carregando-patrimonio = A medir o património
relatorios-patrimonio-liquido = Património líquido
relatorios-patrimonio-pergunta = O património está a crescer? Contas, investimentos e bens acima do zero, dívidas abaixo, e o líquido na linha.
relatorios-patrimonio-liquido-em = Património líquido em { $data }
relatorios-explicar-patrimonio = Explicar património líquido em { $data }
relatorios-patrimonio-titulo = Património · { $n } meses até { $mes }
relatorios-patrimonio-aria = Património líquido em { $n } meses até { $mes }: contas, investimentos e dívidas.
relatorios-patrimonio-tabela = Património por mês
relatorios-carregando-lancamentos = A somar os movimentos
relatorios-agrupar-descricao-dica = Pela descrição do movimento
relatorios-dividido-dica = Movimento dividido: a categoria de cada parte se troca no editor
relatorios-editar-dica = Troque a categoria na própria linha: o relatório recalcula na hora. "Editar" (ou duplo clique) abre o movimento completo.
relatorios-parte-dividido = parte de um movimento dividido
relatorios-lancamentos-de = Movimentos de { $nome }
relatorios-lancamentos-dois-periodos = Movimentos de { $nome } nos dois períodos
relatorios-nenhum-lancamento = Nenhum movimento.
relatorios-cmd-orcamento-busca = bullet,ultrapassámos,orçamento
relatorios-cmd-cartoes-busca = prestações,pagamento a prestações,cartão
relatorios-cmd-patrimonio = Relatório: património em 24 meses
relatorios-cmd-divida-nova = Registar empréstimo ou financiamento
relatorios-cmd-bem-novo = Registar imóvel, veículo ou bem
relatorios-cmd-bem-novo-busca = bem,património,imóvel,apartamento,casa,carro,terreno,fipe
relatorios-cmd-bens = Ver bens (património físico)
relatorios-cmd-sem-categoria = Movimentos sem categoria deste mês
relatorios-vista-tendencia-pergunta = Os gastos estão a subir? Em quê?
relatorios-tendencia-pergunta = Os gastos estão a subir? Em quê? Barras por categoria, a média móvel de 3 meses e as receitas como referência.
relatorios-fluxo-aria = Fluxo de { $periodo }: receitas divididas entre { $n } categorias de despesa{ $investimentos ->
        [sim] , investimentos
       *[nao] {""}
    }{ $sobra ->
        [sim] {" "}e a sobra
       *[nao] , com défice
    }.
