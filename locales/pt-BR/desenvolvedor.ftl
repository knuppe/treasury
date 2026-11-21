# Tela Desenvolvedor (feedback do agente) e catálogo interno de componentes (#/dev/componentes)
# (docs/i18n-design.md). pt-BR é o idioma de origem.

## Áreas, tipos e status do feedback

desenvolvedor-area-agente = Agente
desenvolvedor-area-mcp = MCP
desenvolvedor-area-importacao = Importação
desenvolvedor-area-lancamentos = Lançamentos
desenvolvedor-area-carteira = Carteira
desenvolvedor-area-ui = Interface
desenvolvedor-area-dados = Dados
desenvolvedor-area-outro = Outro
desenvolvedor-tipo-bug = Bug
desenvolvedor-tipo-melhoria = Melhoria
desenvolvedor-tipo-ferramenta-faltando = Ferramenta faltando
desenvolvedor-tipo-dados = Dados
desenvolvedor-tipo-documentacao = Documentação
desenvolvedor-status-aberto = Aberto
desenvolvedor-status-em-andamento = Em andamento
desenvolvedor-status-resolvido = Resolvido
desenvolvedor-status-descartado = Descartado
desenvolvedor-voce = você

## Markdown para issue

desenvolvedor-md-metadados = **Área:** { $area } · **Tipo:** { $tipo } · **Status:** { $status }
desenvolvedor-md-importancia = { $n ->
    [1] **Importância:** { $importancia } ({ $n } reforço)
   *[other] **Importância:** { $importancia } ({ $n } reforços)
}
desenvolvedor-md-registrado = **Registrado por:** { $autor } em { $data }
desenvolvedor-md-reforcos = Reforços
desenvolvedor-md-titulo = Feedback do agente · Treasury
desenvolvedor-md-exportado = { $n ->
    [1] Exportado em { $data } · { $n } feedback.
   *[other] Exportado em { $data } · { $n } feedbacks.
}

## Tela

desenvolvedor-erro-copiar = Não foi possível copiar
desenvolvedor-atualizado = Feedback atualizado
desenvolvedor-erro-gravar = Não foi possível gravar
desenvolvedor-editar-titulo = Editar feedback
desenvolvedor-titulo = Título
desenvolvedor-area = Área
desenvolvedor-tipo = Tipo
desenvolvedor-importancia = Importância
desenvolvedor-em-andamento = Em andamento
desenvolvedor-resolvido = Resolvido
desenvolvedor-descartar = Descartar
desenvolvedor-voltar-aberto = Voltar a aberto
desenvolvedor-reabrir = Reabrir
desenvolvedor-feedback-rotulo = Feedback: { $titulo }
desenvolvedor-importancia-dica = Importância: começa em 1 e cada reforço soma 1
desenvolvedor-n-reforcos = { $n ->
    [1] { $n } reforço
   *[other] { $n } reforços
}
desenvolvedor-ultimo = último { $data }
desenvolvedor-reforcos = Reforços
desenvolvedor-sem-nota = (sem nota)
desenvolvedor-status-mudou = { $status ->
    [aberto] Feedback aberto
    [em_andamento] Feedback em andamento
    [resolvido] Feedback resolvido
   *[descartado] Feedback descartado
}
desenvolvedor-erro-status = Não foi possível mudar o status
desenvolvedor-md-copiado = Markdown copiado
desenvolvedor-copiar-md = Copiar Markdown
desenvolvedor-apagar-titulo = Apagar este feedback
desenvolvedor-apagar-texto = O feedback, os reforços e os vínculos somem da tela. Dá para desfazer pelo aviso ou pelo histórico de alterações.
desenvolvedor-apagado = Feedback apagado
desenvolvedor-erro-apagar = Não foi possível apagar
desenvolvedor-descricao = O que o agente percebeu ao trabalhar e quer que você saiba como desenvolvedor: ferramentas que faltaram, dados confusos, bugs e lacunas da documentação. Ele busca antes e reforça o que já existe em vez de duplicar.
desenvolvedor-n-feedbacks = { $n ->
    [1] { $n } feedback
   *[other] { $n } feedbacks
}
desenvolvedor-n-copiados = { $n ->
    [1] { $n } feedback copiado(s)
   *[other] { $n } feedbacks copiado(s)
}
desenvolvedor-copiar-tudo = Copiar tudo
desenvolvedor-exportar-md = Exportar Markdown
desenvolvedor-exportar-salvar = Salvar feedbacks em Markdown
desenvolvedor-exportados = { $n ->
    [one] { $n } feedback exportado
   *[other] { $n } feedbacks exportados
}
desenvolvedor-erro-exportar = Não deu para exportar os feedbacks
desenvolvedor-desligado = Modo desenvolvedor desligado
desenvolvedor-desligado-titulo = Ligue as opções de desenvolvedor para ver o feedback do agente
desenvolvedor-desligado-texto = Com o modo desenvolvedor ligado, o agente ganha ferramentas para registrar e reforçar feedback técnico sobre o Treasury, que aparece aqui.
desenvolvedor-abrir-ajustes = Abrir Ajustes
desenvolvedor-card = Feedback do agente
desenvolvedor-card-rodape = O agente registra, reforça e marca como resolvido, direto e sem proposta. Mover no quadro, editar e apagar entram no histórico de alterações e podem ser desfeitos.
desenvolvedor-filtro-status = Status
desenvolvedor-buscar = Buscar nos feedbacks
desenvolvedor-buscar-dica = Palavras do título, do texto, dos reforços ou de quem registrou
desenvolvedor-n-de-feedbacks = { $n } de { $total }
desenvolvedor-abertos = Abertos e em andamento
desenvolvedor-todos = Todos
desenvolvedor-todas-areas = Todas as áreas
desenvolvedor-nenhum-filtro = Nenhum feedback com esse filtro.
desenvolvedor-nenhum = Nenhum feedback aberto. Quando o agente esbarrar numa ferramenta que falta, num dado confuso ou num bug, ele registra aqui.
desenvolvedor-lista = Feedback

## Quadro (visão padrão) e painel lateral

desenvolvedor-visao = Visão
desenvolvedor-visao-quadro = Quadro
desenvolvedor-visao-lista = Lista
desenvolvedor-quadro = Quadro de feedback
desenvolvedor-coluna = { $status }: { $n ->
    [1] { $n } item
   *[other] { $n } itens
}
desenvolvedor-coluna-vazia = Arraste um card para cá.
desenvolvedor-todos-tipos = Todos os tipos
desenvolvedor-proximo = Próximo da fila
desenvolvedor-prioridade-n = Prioridade { $n }
desenvolvedor-prioridade-curta = prior.
desenvolvedor-reordenado = Ordem da coluna atualizada
desenvolvedor-mover-para = { $status ->
    [aberto] Voltar a aberto
    [em_andamento] Começar
    [resolvido] Resolvido
   *[descartado] Descartar
}
desenvolvedor-subir = Subir
desenvolvedor-descer = Descer
desenvolvedor-secao-problema = Problema
desenvolvedor-secao-proposta = Proposta
desenvolvedor-secao-aceite = Critério de aceite
desenvolvedor-secao-onde = Onde mexer
desenvolvedor-secao-relacionados = Relacionados
desenvolvedor-secao-passos = Como reproduzir
desenvolvedor-secao-vazia = Não informado.
desenvolvedor-onde-dica = Um caminho por linha, com a linha se souber (crates/treasury-core/src/mod.rs:120).
desenvolvedor-responsavel = Responsável
desenvolvedor-etiquetas = Etiquetas
desenvolvedor-etiquetas-dica = Separadas por vírgula.
desenvolvedor-impacto = { $impacto ->
    [alto] Impacto alto
    [medio] Impacto médio
   *[baixo] Impacto baixo
}
desenvolvedor-confirmado = Confirmado pelo usuário
desenvolvedor-nota-tipo = { $kind ->
    [reforco] reforço
    [status] mudança de status
    [mescla] mesclado
   *[comentario] comentário
}
desenvolvedor-vinculo = { $kind ->
    [duplicado] { $entrada ->
        [sim] Tem como duplicado
       *[nao] Duplicado de
    }
    [depende_de] { $entrada ->
        [sim] É pré-requisito de
       *[nao] Depende de
    }
   *[relacionado] Relacionado a
}
desenvolvedor-copiar-caminho = Copiar o caminho
desenvolvedor-caminho-copiado = Caminho copiado
desenvolvedor-origem = Registrado por { $autor } em { $data }
desenvolvedor-versao = versão { $versao }
desenvolvedor-cc-contexto = Feedback `{ $id }` do Treasury · área { $area } · tipo { $tipo } · prioridade { $prioridade } · { $status }.
desenvolvedor-cc-esperado = Esperado
desenvolvedor-cc-obtido = Obtido
desenvolvedor-cc-sem-aceite = Não informado: defina um teste que prove a correção.
desenvolvedor-cc-fechar = Ao terminar, rode os testes e marque como resolvido no MCP do Treasury: `resolve_dev_feedback` com `id` = `{ $id }` e o motivo (o que mudou, versão ou commit).

## Catálogo de componentes (só em desenvolvimento; valores fictícios)

desenvolvedor-comp-selo = Catálogo de componentes · dev
desenvolvedor-comp-tema = Tema
desenvolvedor-comp-privacidade = Modo privacidade
desenvolvedor-comp-voltar = Voltar ao app →
desenvolvedor-comp-titulo = Design system
desenvolvedor-comp-descricao = Componentes-assinatura do Treasury, no visual de instrumento de medição. Troque o tema acima para conferir contraste e paletas. Todos os valores são fictícios.
desenvolvedor-comp-meta = Revisão visual
desenvolvedor-comp-tokens = Tokens de cor
desenvolvedor-comp-contraste = Contraste sobre --bg · tema { $tema }
desenvolvedor-comp-tipografia = Tipografia
desenvolvedor-comp-rotulo-tipo = Rótulo · 11px · caixa alta · 0,12em
desenvolvedor-comp-titulo-tela = Título de tela · Inter 600
desenvolvedor-comp-texto-corrido = Texto corrido em Inter. Para onde foi o dinheiro do mês? Gastamos mais ou só ficou mais caro? Todo número tem origem rastreável.
desenvolvedor-comp-texto-secundario = Texto secundário em --text-dim.
desenvolvedor-comp-medida = Medida { $tamanho } · { $px }px mono 500
desenvolvedor-comp-cards = Cards de instrumento
desenvolvedor-comp-patrimonio = Patrimônio líquido
desenvolvedor-comp-fonte = Fonte
desenvolvedor-comp-contas-carteira = Contas + carteira
desenvolvedor-comp-atualizado = Atualizado
desenvolvedor-comp-total-brl = Total em BRL
desenvolvedor-comp-12-meses = em 12 meses
desenvolvedor-comp-saldos-conciliados = Saldos conciliados
desenvolvedor-comp-patrimonio-subindo = Patrimônio subindo nos últimos 24 meses
desenvolvedor-comp-ipca = IPCA · 12 meses
desenvolvedor-comp-ref = Ref.
desenvolvedor-comp-ref-valor = out/26 ≈
desenvolvedor-comp-inflacao = Inflação acumulada
desenvolvedor-comp-delta-ipca = ▲ +0,12 p.p.
desenvolvedor-comp-projecao-focus = projeção Focus para outubro
desenvolvedor-comp-nota-focus = Outubro ainda não divulgado: usa a mediana do Focus até o dado oficial sair.
desenvolvedor-comp-poupanca = Taxa de poupança
desenvolvedor-comp-periodo = Período
desenvolvedor-comp-periodo-valor = out/26
desenvolvedor-comp-base = Base
desenvolvedor-comp-competencia = competência
desenvolvedor-comp-formula-poupanca = Receitas − despesas ÷ receitas
desenvolvedor-comp-delta-poupanca = ▼ −2,4 p.p.
desenvolvedor-comp-vs-setembro = vs. setembro
desenvolvedor-comp-receitas = Receitas
desenvolvedor-comp-despesas = Despesas
desenvolvedor-comp-saldo = Saldo
desenvolvedor-comp-figura = Figura
desenvolvedor-comp-patrimonio-inflacao = Patrimônio × inflação
desenvolvedor-comp-patrimonio-24 = Patrimônio líquido, 24 meses
desenvolvedor-comp-nominal = Nominal
desenvolvedor-comp-real-ipca = Real (IPCA)
desenvolvedor-comp-fonte-figura = Fonte · saldos conciliados; deflator IBGE/SIDRA 1737
desenvolvedor-comp-grafico-rotulo = Patrimônio líquido em 24 meses: nominal subiu de 412 para 548 mil reais; corrigido pelo IPCA, para 501 mil.
desenvolvedor-comp-tabela = Tabela técnica
desenvolvedor-comp-tabela-dica = ↑/↓ · Home/End · Enter · clique no cabeçalho ordena
desenvolvedor-comp-lancamentos-outubro = Lançamentos de outubro
desenvolvedor-comp-lancamentos-exemplo = Lançamentos de exemplo
desenvolvedor-comp-abrir = Abrir “{ $descricao }”
desenvolvedor-comp-ativado = Ativado com Enter ou duplo clique.
desenvolvedor-comp-saldo-periodo = Saldo do período
desenvolvedor-comp-data = Data
desenvolvedor-comp-descricao-col = Descrição
desenvolvedor-comp-categoria = Categoria
desenvolvedor-comp-membro = Membro
desenvolvedor-comp-valor = Valor
desenvolvedor-comp-curva-cdi = Curva CDI (Focus)
desenvolvedor-comp-rendimento-projetado = Rendimento projetado até o fechamento do mês.
desenvolvedor-comp-ex-supermercado = Supermercado Bairro
desenvolvedor-comp-ex-salario = Salário
desenvolvedor-comp-ex-farmacia = Farmácia Central
desenvolvedor-comp-ex-aluguel = Aluguel
desenvolvedor-comp-ex-escola = Escola — mensalidade
desenvolvedor-comp-ex-cdb = Rendimento CDB (estimado)
desenvolvedor-comp-ex-streaming = Streaming
desenvolvedor-comp-ex-luz = Conta de luz
desenvolvedor-comp-cat-alimentacao = Alimentação
desenvolvedor-comp-cat-receitas = Receitas
desenvolvedor-comp-cat-saude = Saúde
desenvolvedor-comp-cat-moradia = Moradia
desenvolvedor-comp-cat-educacao = Educação
desenvolvedor-comp-cat-investimentos = Investimentos
desenvolvedor-comp-cat-assinaturas = Assinaturas
desenvolvedor-comp-dinheiro = Dinheiro e origem
desenvolvedor-comp-neutro = Neutro
desenvolvedor-comp-ganho = Ganho
desenvolvedor-comp-perda = Perda
desenvolvedor-comp-outra-moeda = Outra moeda
desenvolvedor-comp-estimado = Estimado
desenvolvedor-comp-sem-moeda = Sem moeda
desenvolvedor-comp-sempre-visivel = Sempre visível
desenvolvedor-comp-selo-oficial = Selo oficial
desenvolvedor-comp-selo-estimado = Selo estimado
desenvolvedor-comp-focus-mediana = Focus (mediana)
desenvolvedor-comp-projecao-ipca = Projeção até o IPCA oficial sair.
desenvolvedor-comp-privacidade-dica = Ative o modo privacidade ({ $atalho }) para ver todos os valores virarem •••••.
desenvolvedor-comp-campos = Campos
desenvolvedor-comp-texto = Texto
desenvolvedor-comp-ajuda = Ajuda abaixo do campo.
desenvolvedor-comp-familia-exemplo = Família Silva
desenvolvedor-comp-com-erro = Com erro
desenvolvedor-comp-informe-familia = Informe o nome da família.
desenvolvedor-comp-senha = Senha
desenvolvedor-comp-senha-exemplo = uma senha qualquer
desenvolvedor-comp-centavos = Centavos: { $valor }
desenvolvedor-comp-moeda = Moeda
desenvolvedor-comp-real = Real
desenvolvedor-comp-dolar = Dólar americano
desenvolvedor-comp-euro = Euro
desenvolvedor-comp-iene = Iene
desenvolvedor-comp-bitcoin = Bitcoin (em breve)
desenvolvedor-comp-botoes = Botões
desenvolvedor-comp-primario = Primário
desenvolvedor-comp-secundario = Secundário
desenvolvedor-comp-fantasma = Fantasma
desenvolvedor-comp-perigo = Perigo
desenvolvedor-comp-demonstracao = Demonstração
desenvolvedor-comp-derivando = Derivando chave…
desenvolvedor-comp-carregando = Carregando
desenvolvedor-comp-desabilitado = Desabilitado
desenvolvedor-comp-pequeno = Pequeno
desenvolvedor-comp-grande = Grande
desenvolvedor-comp-bloquear = Bloquear cofre
desenvolvedor-comp-alternancia = Alternância ligada
desenvolvedor-comp-dialogos = Diálogo, abas e notificações
desenvolvedor-comp-confirmar-exclusao = Confirmar exclusão
desenvolvedor-comp-confirmar-texto = O lançamento vai para o histórico de alterações e pode ser desfeito.
desenvolvedor-comp-abrir-dialogo = Abrir diálogo
desenvolvedor-comp-excluir = Excluir
desenvolvedor-comp-toast-info = Cotações atualizadas
desenvolvedor-comp-toast-sucesso = Backup verificado
desenvolvedor-comp-toast-atencao = IPCA estimado
desenvolvedor-comp-toast-erro = Falha na importação
desenvolvedor-comp-toast-texto = Exemplo de notificação do Treasury.
desenvolvedor-comp-notificacao = { $tom ->
    [info] Notificação · aviso
    [success] Notificação · sucesso
    [warning] Notificação · atenção
   *[error] Notificação · erro
}
desenvolvedor-comp-visao = Visão
desenvolvedor-comp-m2 = M2 · Fase 2
desenvolvedor-comp-rent-nominal = Rentabilidade nominal
desenvolvedor-comp-rent-real = Rentabilidade real
desenvolvedor-comp-estados = Estados e indicadores
desenvolvedor-comp-fase = Fase 2 · Índices e carteira
desenvolvedor-comp-carteira-fase = A carteira chega na Fase 2
desenvolvedor-comp-vazio-texto = Estado vazio de tela ainda não implementada, com o que vai aparecer aqui.
desenvolvedor-comp-item-posicoes = Posições e preço médio
desenvolvedor-comp-item-proventos = Proventos
desenvolvedor-comp-item-twr = TWR e XIRR
desenvolvedor-comp-item-real = Rentabilidade real
desenvolvedor-comp-roadmap = Ver roadmap
desenvolvedor-comp-importando = Importando
desenvolvedor-comp-sincronizando = Sincronizando cotações
desenvolvedor-comp-pausado = Pausado
desenvolvedor-comp-rotulo-dim = Rótulo dim
desenvolvedor-comp-rotulo-fg = Rótulo fg
desenvolvedor-comp-rotulo-accent = Rótulo accent
