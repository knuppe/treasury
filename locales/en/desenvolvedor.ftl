# Developer screen (agent feedback) and the internal component catalog (#/dev/componentes)
# (docs/i18n-design.md). Revision: AI draft, not yet reviewed by a native speaker.

## Feedback areas, kinds and statuses

desenvolvedor-area-agente = Agent
desenvolvedor-area-mcp = MCP
desenvolvedor-area-importacao = Import
desenvolvedor-area-lancamentos = Transactions
desenvolvedor-area-carteira = Portfolio
desenvolvedor-area-ui = Interface
desenvolvedor-area-dados = Data
desenvolvedor-area-outro = Other
desenvolvedor-tipo-bug = Bug
desenvolvedor-tipo-melhoria = Improvement
desenvolvedor-tipo-ferramenta-faltando = Missing tool
desenvolvedor-tipo-dados = Data
desenvolvedor-tipo-documentacao = Documentation
desenvolvedor-status-aberto = Open
desenvolvedor-status-em-andamento = In progress
desenvolvedor-status-resolvido = Resolved
desenvolvedor-status-descartado = Dismissed
desenvolvedor-voce = you

## Markdown for an issue

desenvolvedor-md-metadados = **Area:** { $area } · **Kind:** { $tipo } · **Status:** { $status }
desenvolvedor-md-importancia = { $n ->
    [one] **Importance:** { $importancia } ({ $n } reinforcement)
   *[other] **Importance:** { $importancia } ({ $n } reinforcements)
}
desenvolvedor-md-registrado = **Logged by:** { $autor } on { $data }
desenvolvedor-md-reforcos = Reinforcements
desenvolvedor-md-titulo = Agent feedback · Treasury
desenvolvedor-md-exportado = { $n ->
    [one] Exported on { $data } · { $n } item.
   *[other] Exported on { $data } · { $n } items.
}

## Screen

desenvolvedor-erro-copiar = Couldn't copy
desenvolvedor-atualizado = Feedback updated
desenvolvedor-erro-gravar = Couldn't save
desenvolvedor-editar-titulo = Edit feedback
desenvolvedor-titulo = Title
desenvolvedor-area = Area
desenvolvedor-tipo = Kind
desenvolvedor-importancia = Importance
desenvolvedor-em-andamento = In progress
desenvolvedor-resolvido = Resolved
desenvolvedor-descartar = Dismiss
desenvolvedor-voltar-aberto = Back to open
desenvolvedor-reabrir = Reopen
desenvolvedor-feedback-rotulo = Feedback: { $titulo }
desenvolvedor-importancia-dica = Importance: starts at 1 and each reinforcement adds 1
desenvolvedor-n-reforcos = { $n ->
    [one] { $n } reinforcement
   *[other] { $n } reinforcements
}
desenvolvedor-ultimo = last { $data }
desenvolvedor-reforcos = Reinforcements
desenvolvedor-sem-nota = (no note)
desenvolvedor-status-mudou = { $status ->
    [aberto] Feedback reopened
    [em_andamento] Feedback in progress
    [resolvido] Feedback resolved
   *[descartado] Feedback dismissed
}
desenvolvedor-erro-status = Couldn't change the status
desenvolvedor-md-copiado = Markdown copied
desenvolvedor-copiar-md = Copy Markdown
desenvolvedor-apagar-titulo = Delete this feedback
desenvolvedor-apagar-texto = The feedback, its reinforcements and links disappear from the screen. You can undo it from the notice or from the change history.
desenvolvedor-apagado = Feedback deleted
desenvolvedor-erro-apagar = Couldn't delete
desenvolvedor-descricao = What the agent noticed while working and wants you to know as a developer: missing tools, confusing data, bugs and documentation gaps. It searches first and reinforces what already exists instead of duplicating.
desenvolvedor-n-feedbacks = { $n ->
    [one] { $n } item
   *[other] { $n } items
}
desenvolvedor-n-copiados = { $n ->
    [one] { $n } item copied
   *[other] { $n } items copied
}
desenvolvedor-copiar-tudo = Copy all
desenvolvedor-exportar-md = Export Markdown
desenvolvedor-exportar-salvar = Save feedback as Markdown
desenvolvedor-exportados = { $n ->
    [one] { $n } item exported
   *[other] { $n } items exported
}
desenvolvedor-erro-exportar = Could not export the feedback
desenvolvedor-desligado = Developer mode off
desenvolvedor-desligado-titulo = Turn on the developer options to see the agent's feedback
desenvolvedor-desligado-texto = With developer mode on, the agent gets tools to log and reinforce technical feedback about Treasury, which shows up here.
desenvolvedor-abrir-ajustes = Open Settings
desenvolvedor-card = Agent feedback
desenvolvedor-card-rodape = The agent logs, reinforces and marks as resolved, directly and without a proposal. Moving on the board, editing and deleting go into the change history and can be undone.
desenvolvedor-filtro-status = Status
desenvolvedor-buscar = Search feedback
desenvolvedor-buscar-dica = Words from the title, text, reinforcements or who recorded it
desenvolvedor-n-de-feedbacks = { $n } of { $total }
desenvolvedor-abertos = Open and in progress
desenvolvedor-todos = All
desenvolvedor-todas-areas = All areas
desenvolvedor-nenhum-filtro = No feedback with this filter.
desenvolvedor-nenhum = No open feedback. When the agent runs into a missing tool, confusing data or a bug, it logs it here.
desenvolvedor-lista = Feedback

## Board (default view) and side panel

desenvolvedor-visao = View
desenvolvedor-visao-quadro = Board
desenvolvedor-visao-lista = List
desenvolvedor-quadro = Feedback board
desenvolvedor-coluna = { $status }: { $n ->
    [1] { $n } item
   *[other] { $n } items
}
desenvolvedor-coluna-vazia = Drag a card here.
desenvolvedor-todos-tipos = All types
desenvolvedor-proximo = Next up
desenvolvedor-prioridade-n = Priority { $n }
desenvolvedor-prioridade-curta = prio.
desenvolvedor-reordenado = Column order updated
desenvolvedor-mover-para = { $status ->
    [aberto] Back to open
    [em_andamento] Start
    [resolvido] Resolved
   *[descartado] Discard
}
desenvolvedor-subir = Move up
desenvolvedor-descer = Move down
desenvolvedor-secao-problema = Problem
desenvolvedor-secao-proposta = Proposal
desenvolvedor-secao-aceite = Acceptance criteria
desenvolvedor-secao-onde = Where to change
desenvolvedor-secao-relacionados = Related
desenvolvedor-secao-passos = How to reproduce
desenvolvedor-secao-vazia = Not provided.
desenvolvedor-onde-dica = One path per line, with the line if you know it (crates/treasury-core/src/mod.rs:120).
desenvolvedor-responsavel = Assignee
desenvolvedor-etiquetas = Labels
desenvolvedor-etiquetas-dica = Separated by commas.
desenvolvedor-impacto = { $impacto ->
    [alto] High impact
    [medio] Medium impact
   *[baixo] Low impact
}
desenvolvedor-confirmado = Confirmed by the user
desenvolvedor-nota-tipo = { $kind ->
    [reforco] reinforcement
    [status] status change
    [mescla] merged
   *[comentario] comment
}
desenvolvedor-vinculo = { $kind ->
    [duplicado] { $entrada ->
        [sim] Has as duplicate
       *[nao] Duplicate of
    }
    [depende_de] { $entrada ->
        [sim] Prerequisite of
       *[nao] Depends on
    }
   *[relacionado] Related to
}
desenvolvedor-copiar-caminho = Copy the path
desenvolvedor-caminho-copiado = Path copied
desenvolvedor-origem = Logged by { $autor } on { $data }
desenvolvedor-versao = version { $versao }
desenvolvedor-cc-contexto = Treasury feedback `{ $id }` · area { $area } · type { $tipo } · priority { $prioridade } · { $status }.
desenvolvedor-cc-esperado = Expected
desenvolvedor-cc-obtido = Actual
desenvolvedor-cc-sem-aceite = Not provided: define a test that proves the fix.
desenvolvedor-cc-fechar = When done, run the tests and mark it as resolved in the Treasury MCP: `resolve_dev_feedback` with `id` = `{ $id }` and the reason (what changed, version or commit).

## Component catalog (development only; made-up values)

desenvolvedor-comp-selo = Component catalog · dev
desenvolvedor-comp-tema = Theme
desenvolvedor-comp-privacidade = Privacy mode
desenvolvedor-comp-voltar = Back to the app →
desenvolvedor-comp-titulo = Design system
desenvolvedor-comp-descricao = Treasury's signature components, with the look of a measuring instrument. Change the theme above to check contrast and palettes. All values are made up.
desenvolvedor-comp-meta = Visual review
desenvolvedor-comp-tokens = Color tokens
desenvolvedor-comp-contraste = Contrast on --bg · { $tema } theme
desenvolvedor-comp-tipografia = Typography
desenvolvedor-comp-rotulo-tipo = Label · 11px · uppercase · 0.12em
desenvolvedor-comp-titulo-tela = Screen title · Inter 600
desenvolvedor-comp-texto-corrido = Body text in Inter. Where did this month's money go? Did we spend more or did things just get pricier? Every number has a traceable source.
desenvolvedor-comp-texto-secundario = Secondary text in --text-dim.
desenvolvedor-comp-medida = Measure { $tamanho } · { $px }px mono 500
desenvolvedor-comp-cards = Instrument cards
desenvolvedor-comp-patrimonio = Net worth
desenvolvedor-comp-fonte = Source
desenvolvedor-comp-contas-carteira = Accounts + portfolio
desenvolvedor-comp-atualizado = Updated
desenvolvedor-comp-total-brl = Total in BRL
desenvolvedor-comp-12-meses = over 12 months
desenvolvedor-comp-saldos-conciliados = Reconciled balances
desenvolvedor-comp-patrimonio-subindo = Net worth rising over the last 24 months
desenvolvedor-comp-ipca = IPCA · 12 months
desenvolvedor-comp-ref = Ref.
desenvolvedor-comp-ref-valor = Oct 26 ≈
desenvolvedor-comp-inflacao = Cumulative inflation
desenvolvedor-comp-delta-ipca = ▲ +0.12 pp
desenvolvedor-comp-projecao-focus = Focus projection for October
desenvolvedor-comp-nota-focus = October not released yet: uses the Focus median until the official figure comes out.
desenvolvedor-comp-poupanca = Savings rate
desenvolvedor-comp-periodo = Period
desenvolvedor-comp-periodo-valor = Oct 26
desenvolvedor-comp-base = Basis
desenvolvedor-comp-competencia = accrual
desenvolvedor-comp-formula-poupanca = Income − expenses ÷ income
desenvolvedor-comp-delta-poupanca = ▼ −2.4 pp
desenvolvedor-comp-vs-setembro = vs. September
desenvolvedor-comp-receitas = Income
desenvolvedor-comp-despesas = Expenses
desenvolvedor-comp-saldo = Balance
desenvolvedor-comp-figura = Figure
desenvolvedor-comp-patrimonio-inflacao = Net worth vs. inflation
desenvolvedor-comp-patrimonio-24 = Net worth, 24 months
desenvolvedor-comp-nominal = Nominal
desenvolvedor-comp-real-ipca = Real (IPCA)
desenvolvedor-comp-fonte-figura = Source · reconciled balances; IBGE/SIDRA 1737 deflator
desenvolvedor-comp-grafico-rotulo = Net worth over 24 months: nominal rose from 412 to 548 thousand reais; adjusted for IPCA, to 501 thousand.
desenvolvedor-comp-tabela = Technical table
desenvolvedor-comp-tabela-dica = ↑/↓ · Home/End · Enter · click a header to sort
desenvolvedor-comp-lancamentos-outubro = October transactions
desenvolvedor-comp-lancamentos-exemplo = Sample transactions
desenvolvedor-comp-abrir = Open “{ $descricao }”
desenvolvedor-comp-ativado = Activated with Enter or a double click.
desenvolvedor-comp-saldo-periodo = Period balance
desenvolvedor-comp-data = Date
desenvolvedor-comp-descricao-col = Description
desenvolvedor-comp-categoria = Category
desenvolvedor-comp-membro = Member
desenvolvedor-comp-valor = Amount
desenvolvedor-comp-curva-cdi = CDI curve (Focus)
desenvolvedor-comp-rendimento-projetado = Yield projected through month end.
desenvolvedor-comp-ex-supermercado = Neighborhood Supermarket
desenvolvedor-comp-ex-salario = Salary
desenvolvedor-comp-ex-farmacia = Central Pharmacy
desenvolvedor-comp-ex-aluguel = Rent
desenvolvedor-comp-ex-escola = School — monthly fee
desenvolvedor-comp-ex-cdb = CDB yield (estimated)
desenvolvedor-comp-ex-streaming = Streaming
desenvolvedor-comp-ex-luz = Electricity bill
desenvolvedor-comp-cat-alimentacao = Food
desenvolvedor-comp-cat-receitas = Income
desenvolvedor-comp-cat-saude = Health
desenvolvedor-comp-cat-moradia = Housing
desenvolvedor-comp-cat-educacao = Education
desenvolvedor-comp-cat-investimentos = Investments
desenvolvedor-comp-cat-assinaturas = Subscriptions
desenvolvedor-comp-dinheiro = Money and source
desenvolvedor-comp-neutro = Neutral
desenvolvedor-comp-ganho = Gain
desenvolvedor-comp-perda = Loss
desenvolvedor-comp-outra-moeda = Other currency
desenvolvedor-comp-estimado = Estimated
desenvolvedor-comp-sem-moeda = No currency
desenvolvedor-comp-sempre-visivel = Always visible
desenvolvedor-comp-selo-oficial = Official badge
desenvolvedor-comp-selo-estimado = Estimated badge
desenvolvedor-comp-focus-mediana = Focus (median)
desenvolvedor-comp-projecao-ipca = Projection until the official IPCA comes out.
desenvolvedor-comp-privacidade-dica = Turn on privacy mode ({ $atalho }) to see every value turn into •••••.
desenvolvedor-comp-campos = Fields
desenvolvedor-comp-texto = Text
desenvolvedor-comp-ajuda = Help below the field.
desenvolvedor-comp-familia-exemplo = Silva family
desenvolvedor-comp-com-erro = With error
desenvolvedor-comp-informe-familia = Enter the family name.
desenvolvedor-comp-senha = Password
desenvolvedor-comp-senha-exemplo = any password
desenvolvedor-comp-centavos = Cents: { $valor }
desenvolvedor-comp-moeda = Currency
desenvolvedor-comp-real = Brazilian real
desenvolvedor-comp-dolar = US dollar
desenvolvedor-comp-euro = Euro
desenvolvedor-comp-iene = Yen
desenvolvedor-comp-bitcoin = Bitcoin (coming soon)
desenvolvedor-comp-botoes = Buttons
desenvolvedor-comp-primario = Primary
desenvolvedor-comp-secundario = Secondary
desenvolvedor-comp-fantasma = Ghost
desenvolvedor-comp-perigo = Danger
desenvolvedor-comp-demonstracao = Demo
desenvolvedor-comp-derivando = Deriving key…
desenvolvedor-comp-carregando = Loading
desenvolvedor-comp-desabilitado = Disabled
desenvolvedor-comp-pequeno = Small
desenvolvedor-comp-grande = Large
desenvolvedor-comp-bloquear = Lock vault
desenvolvedor-comp-alternancia = Toggle on
desenvolvedor-comp-dialogos = Dialog, tabs and notifications
desenvolvedor-comp-confirmar-exclusao = Confirm deletion
desenvolvedor-comp-confirmar-texto = The transaction goes to the change history and can be undone.
desenvolvedor-comp-abrir-dialogo = Open dialog
desenvolvedor-comp-excluir = Delete
desenvolvedor-comp-toast-info = Quotes updated
desenvolvedor-comp-toast-sucesso = Backup verified
desenvolvedor-comp-toast-atencao = IPCA estimated
desenvolvedor-comp-toast-erro = Import failed
desenvolvedor-comp-toast-texto = Sample Treasury notification.
desenvolvedor-comp-notificacao = { $tom ->
    [info] Notification · info
    [success] Notification · success
    [warning] Notification · warning
   *[error] Notification · error
}
desenvolvedor-comp-visao = View
desenvolvedor-comp-m2 = M2 · Phase 2
desenvolvedor-comp-rent-nominal = Nominal return
desenvolvedor-comp-rent-real = Real return
desenvolvedor-comp-estados = States and indicators
desenvolvedor-comp-fase = Phase 2 · Indices and portfolio
desenvolvedor-comp-carteira-fase = The portfolio arrives in Phase 2
desenvolvedor-comp-vazio-texto = Empty state for a screen not built yet, with what will show up here.
desenvolvedor-comp-item-posicoes = Positions and average price
desenvolvedor-comp-item-proventos = Distributions
desenvolvedor-comp-item-twr = TWR and XIRR
desenvolvedor-comp-item-real = Real return
desenvolvedor-comp-roadmap = See roadmap
desenvolvedor-comp-importando = Importing
desenvolvedor-comp-sincronizando = Syncing quotes
desenvolvedor-comp-pausado = Paused
desenvolvedor-comp-rotulo-dim = Dim label
desenvolvedor-comp-rotulo-fg = Fg label
desenvolvedor-comp-rotulo-accent = Accent label
