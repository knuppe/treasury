# Screen 12 · Agent and the agent pieces also used in the side panel: conversation, message
# box, approval queue, change review, alerts, memory, procedures, tool history and AI engine
# settings (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Screen and tabs

agente-aba-conversa = Conversation
agente-aba-aprovacao = Approval
agente-aba-alertas = Alerts
agente-aba-ferramentas = Tools
agente-aba-memoria = Memory
agente-aba-configurar = Set up
agente-secoes = Agent sections
agente-cmd-aba = Agent: { $aba }
agente-cmd-busca = agent,ai
agente-configurar-motores = Set up AI engines
agente-n-pendentes = ({ $n } pending)
agente-n-novos = ({ $n } new)
agente-motores-ligados = { $n ->
        [one] { $n } engine on
       *[other] { $n } engines on
    }
agente-badge-propostas = { $n ->
        [one] { $n } pending proposal
       *[other] { $n } pending proposals
    }
agente-badge-alertas = { $n ->
        [one] { $n } new alert
       *[other] { $n } new alerts
    }

## Numbers, times and quotes

agente-tokens-mil = { NUMBER($n, maximumFractionDigits: 1) }K
agente-tokens-mi = { NUMBER($n, maximumFractionDigits: 1) }M
agente-menos-1-s = less than 1 s
agente-segundos = { $s } s
agente-minutos = { $m } min
agente-minutos-segundos = { $m } min { $s } s
agente-aspas = “{ $texto }”
agente-lista-e = { $inicio } and { $fim }
agente-lista-e-mais = { $inicio } and { $n } more
agente-sim = yes
agente-nao = no

## Tasks, engine types and states

agente-tarefa-chat = Chat with the agent
agente-tarefa-categorizacao = Categorization
agente-tarefa-pdf = PDF reading
agente-tarefa-fechamento = Monthly close
agente-tarefa-alertas = Alerts
agente-tarefa-estudo = Asset studies
agente-tipo-motor-api = API
agente-tipo-motor-cli = Local CLI
agente-tipo-motor-local = Local model
agente-destino-api = data sent to the provider's API
agente-destino-cli = data goes through the local CLI (and its provider)
agente-destino-local = data stays on this computer
agente-nivel-reversivel = Reversible · batch
agente-nivel-sensivel = Sensitive · one by one
agente-proposta-pendente = Pending
agente-proposta-aprovada = Approved
agente-proposta-rejeitada = Rejected
agente-proposta-falhou = Failed
agente-proposta-retirada = Withdrawn
agente-alerta-novo = New
agente-alerta-visto = Seen
agente-alerta-resolvido = Resolved
agente-gravidade-info = Info
agente-gravidade-atencao = Warning
agente-gravidade-critico = Critical
agente-job-andamento = Running
agente-job-concluido = Done
agente-job-falhou = Failed
agente-job-cancelado = Canceled
agente-memoria-perfil = Investor profile
agente-memoria-regra = Family rule
agente-memoria-nota = Note
agente-problema-desligado = engine off
agente-problema-sem-chave = no API key saved
agente-problema-sem-template = no command template
agente-problema-sem-modelo = no model set

## Proposed write commands

agente-comando-criar-lancamento = Create transaction
agente-comando-alterar-lancamento = Edit transaction
agente-comando-categorizar = Categorize transactions
agente-comando-dividir = Split transaction
agente-comando-criar-transferencia = Create transfer
agente-comando-apagar-lancamento = Delete transaction
agente-comando-apagar-feedback = Delete developer feedback
agente-comando-descartar-feedback = Discard developer feedback
agente-comando-criar-etiqueta = Create tag
agente-comando-criar-categoria = Create category
agente-comando-alterar-categoria = Edit category
agente-comando-cadastrar-instituicao = Add institution
agente-comando-alterar-instituicao = Edit institution
agente-comando-cadastrar-conta = Add account or card
agente-comando-alterar-conta = Edit account
agente-comando-alterar-recorrencia = Edit recurrence
agente-comando-alterar-ativo = Edit asset
agente-comando-cadastrar-bem = Add physical asset
agente-comando-alterar-bem = Edit physical asset
agente-comando-avaliacao-bem = Record asset valuation
agente-comando-benfeitoria = Record improvement
agente-comando-criar-regra = Create rule
agente-comando-alterar-regra = Edit rule
agente-comando-aplicar-regra = Apply rule
agente-comando-criar-recorrencia = Create recurrence
agente-comando-definir-orcamento = Set budget
agente-comando-copiar-orcamento = Copy budget
agente-comando-criar-meta = Create goal
agente-comando-alterar-meta = Edit goal
agente-comando-cadastrar-ativo = Add asset
agente-comando-acompanhar-ativo = Watch asset
agente-comando-registrar-operacao = Record trade
agente-comando-registrar-provento = Record dividend
agente-comando-proventos-ticker = Record dividends by ticker
agente-comando-evento-corporativo = Record corporate action
agente-comando-importar-extrato = Import statement
agente-comando-importar-lidos = Import transactions read by the agent
agente-comando-investimentos-lote = Add investments in bulk
agente-comando-importar-b3 = Import from B3
agente-comando-gravar-memoria = Save to the manager's memory

## Procedure topics

agente-assunto-extrato = Statement
agente-assunto-fatura = Card bill
agente-assunto-investimentos = Investments
agente-assunto-proventos = Dividends
agente-assunto-cotacoes = Quotes and indicators
agente-assunto-imposto = Income tax
agente-assunto-cadastro = Registration
agente-assunto-outro = Other

## Database tables in changes

agente-tabela-lancamento = Transaction
agente-tabela-divisao = Transaction split
agente-tabela-etiqueta-lancamento = Transaction tag
agente-tabela-etiqueta = Tag
agente-tabela-categoria = Category
agente-tabela-regra = Rule
agente-tabela-aplicacao-regra = Rule match
agente-tabela-recorrencia = Recurrence
agente-tabela-orcamento = Budget
agente-tabela-linha-orcamento = Budget line
agente-tabela-meta = Goal
agente-tabela-conta-meta = Goal account
agente-tabela-projeto = Project
agente-tabela-ativo = Asset
agente-tabela-operacao = Trade
agente-tabela-provento = Dividend
agente-tabela-evento-corporativo = Corporate action
agente-tabela-acompanhamento = Watchlist item
agente-tabela-lote-importacao = Import batch
agente-tabela-memoria = Manager's memory
agente-tabela-procedimento = Agent procedure
agente-tabela-feedback = Developer feedback
agente-tabela-nota-feedback = Feedback note
agente-tabela-vinculo-feedback = Feedback link
agente-tabela-conta = Account
agente-tabela-membro-conta = Account member
agente-tabela-cartao = Credit card
agente-tabela-instituicao = Institution
agente-tabela-renda-fixa = Fixed income
agente-tabela-bem = Physical asset
agente-tabela-dono-bem = Asset owner
agente-tabela-custo-aquisicao = Acquisition cost
agente-tabela-benfeitoria = Improvement
agente-tabela-avaliacao-bem = Asset valuation
agente-tabela-financiamento-bem = Asset loan

## Counts per table

agente-n-lancamento = { $n ->
        [one] { $n } transaction
       *[other] { $n } transactions
    }
agente-n-divisao = { $n ->
        [one] { $n } transaction split
       *[other] { $n } transaction splits
    }
agente-n-etiqueta-lancamento = { $n ->
        [one] { $n } transaction tag
       *[other] { $n } transaction tags
    }
agente-n-etiqueta = { $n ->
        [one] { $n } tag
       *[other] { $n } tags
    }
agente-n-categoria = { $n ->
        [one] { $n } category
       *[other] { $n } categories
    }
agente-n-regra = { $n ->
        [one] { $n } rule
       *[other] { $n } rules
    }
agente-n-aplicacao-regra = { $n ->
        [one] { $n } rule match
       *[other] { $n } rule matches
    }
agente-n-recorrencia = { $n ->
        [one] { $n } recurrence
       *[other] { $n } recurrences
    }
agente-n-orcamento = { $n ->
        [one] { $n } budget
       *[other] { $n } budgets
    }
agente-n-linha-orcamento = { $n ->
        [one] { $n } budget line
       *[other] { $n } budget lines
    }
agente-n-meta = { $n ->
        [one] { $n } goal
       *[other] { $n } goals
    }
agente-n-conta-meta = { $n ->
        [one] { $n } goal account
       *[other] { $n } goal accounts
    }
agente-n-projeto = { $n ->
        [one] { $n } project
       *[other] { $n } projects
    }
agente-n-ativo = { $n ->
        [one] { $n } asset
       *[other] { $n } assets
    }
agente-n-operacao = { $n ->
        [one] { $n } trade
       *[other] { $n } trades
    }
agente-n-provento = { $n ->
        [one] { $n } dividend
       *[other] { $n } dividends
    }
agente-n-evento-corporativo = { $n ->
        [one] { $n } corporate action
       *[other] { $n } corporate actions
    }
agente-n-acompanhamento = { $n ->
        [one] { $n } watchlist item
       *[other] { $n } watchlist items
    }
agente-n-lote-importacao = { $n ->
        [one] { $n } import batch
       *[other] { $n } import batches
    }
agente-n-memoria = { $n ->
        [one] { $n } manager's memory note
       *[other] { $n } manager's memory notes
    }
agente-n-procedimento = { $n ->
        [one] { $n } agent procedure
       *[other] { $n } agent procedures
    }
agente-n-feedback = { $n ->
        [one] { $n } developer feedback item
       *[other] { $n } developer feedback items
    }
agente-n-nota-feedback = { $n ->
        [one] { $n } feedback note
       *[other] { $n } feedback notes
    }
agente-n-vinculo-feedback = { $n ->
        [one] { $n } feedback link
       *[other] { $n } feedback links
    }
agente-n-conta = { $n ->
        [one] { $n } account
       *[other] { $n } accounts
    }
agente-n-membro-conta = { $n ->
        [one] { $n } account member
       *[other] { $n } account members
    }
agente-n-cartao = { $n ->
        [one] { $n } credit card
       *[other] { $n } credit cards
    }
agente-n-instituicao = { $n ->
        [one] { $n } institution
       *[other] { $n } institutions
    }
agente-n-renda-fixa = { $n ->
        [one] { $n } fixed income term
       *[other] { $n } fixed income terms
    }
agente-n-bem = { $n ->
        [one] { $n } physical asset
       *[other] { $n } physical assets
    }
agente-n-dono-bem = { $n ->
        [one] { $n } asset owner
       *[other] { $n } asset owners
    }
agente-n-custo-aquisicao = { $n ->
        [one] { $n } acquisition cost
       *[other] { $n } acquisition costs
    }
agente-n-benfeitoria = { $n ->
        [one] { $n } improvement
       *[other] { $n } improvements
    }
agente-n-avaliacao-bem = { $n ->
        [one] { $n } asset valuation
       *[other] { $n } asset valuations
    }
agente-n-financiamento-bem = { $n ->
        [one] { $n } asset loan
       *[other] { $n } asset loans
    }
agente-n-alteracoes = { $n ->
        [one] { $n } change
       *[other] { $n } changes
    }

## Summary and sentences about what changes

agente-resumo-cria = creates { $itens }
agente-resumo-altera = changes { $itens }
agente-resumo-apaga = deletes { $itens }
agente-frase-cria = Creates { $itens }: { $nomes }
agente-frase-apaga = Deletes { $itens }: { $nomes }
agente-frase-registra = Records { $itens }: { $nomes }
agente-frase-grava = Saves { $itens }: { $nomes }
agente-frase-acompanha = Watches { $itens }: { $nomes }
agente-frase-inclui = Adds { $itens }: { $nomes }
agente-frase-poe = Adds { $itens }: { $nomes }
agente-frase-altera = Changes { $campo } of { $alvo }
agente-frase-altera-para = Changes { $campo } of { $alvo } to { $para }
agente-frase-altera-de-para = Changes { $campo } of { $alvo }: { $de } → { $para }
agente-artigo-campo = the field
agente-artigo-categoria = the category
agente-artigo-valor = the amount
agente-artigo-membro = the member
agente-artigo-conta = the account
agente-artigo-situacao = the status
agente-artigo-tipo = the type
agente-artigo-nome = the name
agente-artigo-descricao = the description
agente-artigo-favorecido = the payee
agente-artigo-notas = the notes
agente-artigo-data = the date
agente-artigo-ativacao = the active setting
agente-artigo-arquivamento = the archived setting
agente-artigo-prioridade = the priority
agente-artigo-categoria-pai = the parent category
agente-artigo-instituicao = the institution
agente-artigo-essencial = the “essential” flag
agente-artigo-grupo-gasto = the spending group
agente-artigo-condicoes = the conditions
agente-artigo-acoes = the actions
agente-artigo-cor = the color

## Friendly reading of changes: operations, titles and values

agente-op-criacao = Creation
agente-op-inclusao = Insertion
agente-op-alteracao = Change
agente-op-exclusao = Deletion
agente-lado-compra = Buy
agente-lado-venda = Sell
agente-sem-categoria = No category
agente-sem-membro = No member
agente-sem-pai = None (top level)
agente-sem-descricao = no description
agente-lancamento-sem-descricao = Transaction without description
agente-outro-lancamento = another transaction
agente-nao-encontrado = { $ref } (not found)
agente-ref-categoria = category
agente-ref-conta = account
agente-ref-membro = member
agente-ref-instituicao = institution
agente-ref-ativo = asset
agente-ref-contrato = loan
agente-ref-etiqueta = tag
agente-ref-regra = rule
agente-ref-meta = goal
agente-ref-recorrencia = recurrence
agente-ref-lancamento = transaction
agente-ref-bem = physical asset
agente-regra-categoria = category { $nome }
agente-regra-membro = member { $nome }
agente-regra-etiquetas = tags { $etiquetas }
agente-regra-favorecido = payee “{ $nome }”
agente-regra-tipo = type { $tipo }
agente-regra-nenhuma = none
agente-titulo-etiqueta-em = #{ $etiqueta } on { $lancamento }
agente-titulo-parte-de = Part of { $lancamento }
agente-titulo-de = { $tipo }: { $ativo }
agente-titulo-meta = goal { $nome }
agente-operacao = Trade
agente-provento = Dividend
agente-evento = Event
agente-nota-gestor = Manager's note

## Change fields

agente-campo-nome = Name
agente-campo-nome-completo = Full name
agente-campo-nome-curto = Short name
agente-campo-tipo = Type
agente-campo-classe = Class
agente-campo-moeda = Currency
agente-campo-valor = Amount
agente-campo-data = Date
agente-campo-situacao = Status
agente-campo-descricao = Description
agente-campo-favorecido = Payee
agente-campo-notas = Notes
agente-campo-nota = Note
agente-campo-conteudo = Content
agente-campo-titulo = Title
agente-campo-categoria = Category
agente-campo-categoria-pai = Parent category
agente-campo-membro = Member
agente-campo-conta = Account
agente-campo-conta-destino = Destination account
agente-campo-conta-debito = Debit account
agente-campo-conta-pagamento = Payment account
agente-campo-instituicao = Institution
agente-campo-emissor = Issuer
agente-campo-ativo = Asset
agente-campo-ativo-novo = New asset
agente-campo-contrato = Loan
agente-campo-etiqueta = Tag
agente-campo-regra = Rule
agente-campo-meta = Goal
agente-campo-recorrencia = Recurrence
agente-campo-lancamento = Transaction
agente-campo-transferencia = Transfer
agente-campo-fatura = Card bill
agente-campo-parcelamento = Installment plan
agente-campo-parcela = Installment
agente-campo-parcelas = Installments
agente-campo-orcamento = Budget
agente-campo-bem = Asset
agente-campo-arquivado = Archived
agente-campo-ativa = Active
agente-campo-ligada = On
agente-campo-essencial = Essential
agente-campo-grupo-gasto = Spending group
agente-campo-principal = Primary
agente-campo-patrimonio = Counts toward net worth
agente-campo-todas-condicoes = All conditions
agente-campo-isento-ir = Income tax exempt
agente-campo-day-trade = Day trade
agente-campo-fixado = Pinned
agente-campo-saldo-inicial = Opening balance
agente-campo-data-inicial = Opening date
agente-campo-limite = Limit
agente-campo-dia-fechamento = Closing day
agente-campo-dia-vencimento = Due day
agente-campo-total-informado = Stated total
agente-campo-valor-total = Total amount
agente-campo-data-compra = Purchase date
agente-campo-primeira-parcela = First installment
agente-campo-prioridade = Priority
agente-campo-condicoes = Conditions
agente-campo-acoes = Actions
agente-campo-repeticao = Repeats
agente-campo-inicio = Start
agente-campo-fim = End
agente-campo-taxa = Rate
agente-campo-taxa-ano = Annual rate
agente-campo-taxa-mes = Monthly rate
agente-campo-indexador = Index
agente-campo-pct-indexador = % of index
agente-campo-liquidez = Liquidity
agente-campo-emissao = Issue date
agente-campo-vencimento = Maturity
agente-campo-carencia = Grace period until
agente-campo-ticker = Ticker
agente-campo-mercado = Market
agente-campo-pais = Country
agente-campo-setor = Sector
agente-campo-segmento = Segment
agente-campo-fonte-cotacao = Quote source
agente-campo-simbolo-fonte = Symbol at source
agente-campo-operacao = Side
agente-campo-quantidade = Quantity
agente-campo-preco = Price
agente-campo-preco-unitario = Unit price
agente-campo-custo-unitario = Unit cost
agente-campo-custos = Fees
agente-campo-impostos = Taxes
agente-campo-cambio = Exchange rate
agente-campo-nota-corretagem = Brokerage note
agente-campo-fator = Factor
agente-campo-data-com = Record date
agente-campo-pagamento = Payment
agente-campo-bruto = Gross
agente-campo-liquido = Net
agente-campo-ir-retido = Tax withheld
agente-campo-preco-alvo = Target price
agente-campo-alerta-acima = Alert above
agente-campo-alerta-abaixo = Alert below
agente-campo-valor-meta = Goal amount
agente-campo-prazo = Deadline
agente-campo-valor-inicial = Initial amount
agente-campo-aporte-mensal = Monthly contribution
agente-campo-valor-financiado = Amount financed
agente-campo-prazo-meses = Term (months)
agente-campo-sistema = System
agente-campo-data-contrato = Contract date
agente-campo-numero-contrato = Contract number
agente-campo-primeiro-vencimento = First due date
agente-campo-cor = Color
agente-campo-icone = Icon
agente-campo-ordem = Order
agente-campo-mes = Month
agente-campo-custo = Cost
agente-campo-origem = Source
agente-campo-dominio = Domain
agente-campo-rotulo = Label
agente-campo-assunto = Topic
agente-campo-passos = Steps
agente-campo-agencia = Branch

## Change review and exact diff

agente-campo = Field
agente-valor = Value
agente-antes = Before
agente-depois = After
agente-para = to
agente-nenhuma-linha-alteracao = No database row changes with this change.
agente-nenhuma-linha-proposta = No database row changes with this proposal.
agente-o-que-muda = What changes
agente-mostrar-linhas = Show all { $n } rows
agente-ver-tecnicos = Show technical details
agente-ocultar-tecnicos = Hide technical details
agente-diff-exato = Exact diff of what will be saved: tables, ids and raw values (money in cents).
agente-so-o-que-muda = Only what changes
agente-mostrar-campos = Show all fields (+{ $n })

## Conversation

agente-motor-em-uso = Engine in use
agente-proximo-motor = Next engine
agente-motor-em-uso-nome = Engine in use: { $motor }
agente-proximo-motor-nome = Next engine: { $motor }
agente-nenhum-motor-dica = No engine ready: set one up in Settings › AI and agent
agente-motor-recebe = Engine that receives the data
agente-respondendo = Answering
agente-recebendo = Receiving
agente-nenhum-motor-pronto = no engine ready
agente-proposta-criada = Proposal created: { $nome }
agente-aguarda-aprovacao = awaiting approval
agente-revisar-fila = Review in the queue
agente-fallback = { $motor } did not answer ({ $motivo }); trying the next engine in the route.
agente-agora-com = Now with { $motor } ({ $tipo }).
agente-uso-resposta = Usage for this answer
agente-tokens-entrada = { $n } input tokens
agente-tokens-saida = { $n } output
agente-custo = cost { $valor }
agente-custo-parcial = cost { $valor } (partial: model without a price)
agente-voce = You
agente-motor-generico = engine
agente-alteracao = change
agente-sem-motor-dica = Add an engine (API, local CLI or local model) and set the chat route in { $link }.
agente-sem-motor-titulo = No AI engine set up
agente-nao-respondeu = The agent did not answer
agente-aguardando-motor = waiting for the engine
agente-resposta-cancelada = Answer canceled.
agente-conversas = Conversations
agente-nova = New
agente-nenhuma-conversa = No saved conversations.
agente-confirmar-custo = Confirm cost
agente-custo-texto = The first turn with **{ $motor }** should cost about **{ $custo }**, above the { $limite } warning set in the spending limits.
agente-enviar-mesmo-assim = Send anyway
agente-automatico = Automatic (chat route)
agente-conversa-com = Conversation with the agent
agente-historico-desligado = History off: each question starts from scratch and nothing is saved.
agente-vazio-titulo = Ask about spending, budget, debts or the portfolio.
agente-vazio-texto = The agent uses Treasury's tools; every write waits for your approval in the queue. Investment recommendations are analyses of your data, not professional advice.
agente-vazio-texto-autonomo = The agent uses Treasury's tools. With autonomous mode on, reversible writes apply right away and can be undone in the history; sensitive ones wait for your approval. Investment recommendations are analyses of your data, not professional advice.
agente-mensagem = Message to the agent
agente-placeholder = E.g.: categorize this month's uncategorized transactions
agente-motor = Engine
agente-enter-envia = Enter sends · Shift+Enter adds a line
agente-enviar = Send
agente-enviar-assim-mesmo = Send anyway
agente-erro-sem-job = The core did not return the conversation job.

## Answer activity (reasoning and tools)

agente-pensando = Thinking…
agente-raciocinio = Reasoning
agente-raciocinio-tempo = Reasoning · { $tempo }
agente-consultando = Checking { $ferramenta }…
agente-consultou = Used { $n ->
        [one] { $n } tool
       *[other] { $n } tools
    }
agente-consultou-erro = Used { $n ->
        [one] { $n } tool
       *[other] { $n } tools
    } · { $erros } with errors
agente-ferramenta-nome = Tool { $nome }
agente-ok = ok
agente-erro = error
agente-ferramenta-cli = The CLI's own tool, outside Treasury
agente-atividade = Agent activity
agente-raciocinio-agente = Agent reasoning
agente-ferramentas-usadas = Tools used

## Saved conversations

agente-conversa-meta = { $quando } · { $n } msg
agente-fixar = Pin conversation { $titulo }
agente-desafixar = Unpin conversation { $titulo }
agente-fixada-dica = Pinned: the automatic cleanup skips it
agente-fixar-dica = Pin: the automatic cleanup will not delete it
agente-apagar-conversa = Delete conversation { $titulo }
agente-nao-apagou = Could not delete
agente-apagar-todas = Delete all conversations
agente-apagar-todas-texto = The conversation history is removed from the database. Proposals, alerts and the manager's memory stay.
agente-apagar-historico = Delete history
agente-apagar-tudo = Delete all

## Computer

agente-computador-ligado-dica = In this conversation, local agent CLIs can read files in your home folder when you ask. Terminal, writing and web stay blocked; API and local engines keep using only Treasury.
agente-computador-desligado-dica = The agent uses only Treasury's tools and does not look at your files, apps or browser. Turn this on to let a CLI read a file you point to in this conversation.
agente-computador-rotulo = Computer access in this conversation
agente-computador-desligado = Computer access off
agente-computador-liberado = Computer reading allowed
agente-computador-volta = The agent goes back to using only Treasury.
agente-computador = Computer
agente-computador-leitura = Computer: reading allowed
agente-computador-bloqueado = Computer: blocked

## Reasoning effort

agente-esforco-padrao = Default
agente-esforco-baixo = Low
agente-esforco-medio = Medium
agente-esforco-alto = High
agente-esforco-maximo = Maximum
agente-esforco-padrao-dica = what the engine uses
agente-esforco-baixo-dica = faster
agente-esforco-medio-dica = balanced
agente-esforco-alto-dica = thinks more
agente-esforco-maximo-dica = slower and pricier
agente-esforco-dica = How much the model reasons before answering in this conversation. More effort usually gives better answers to hard questions, with more delay and cost.
agente-esforco-sem-controle = { $motor } has no effort control with this model; its default applies.
agente-este-motor = This engine
agente-esforco-rotulo = Reasoning effort in this conversation
agente-esforco-valor = Effort: { $nivel }

## Attachments

agente-anexo-colado = pasted.png
agente-anexos-a-mais = { $n ->
        [one] { $n } extra file
       *[other] { $n } extra files
    }
agente-anexos-maximo = at most { $max } attachments per message
agente-anexo-grande = file too large (max. 20 MB)
agente-anexo-paginas = { $n ->
        [one] { $n } page
       *[other] { $n } pages
    }
agente-anexo-caracteres = { $n } characters
agente-anexo-escaneado = no text (scanned)
agente-anexo-cortado = text cut off
agente-anexar-dica = Attach images, PDFs, spreadsheets or text files. You can also drag files here or paste with ⌘V / Ctrl+V.
agente-anexar-arquivos = Attach files
agente-anexar = Attach
agente-remover = Remove { $nome }
agente-anexos-mensagem = Message attachments
agente-anexo-preparando = preparing…
agente-falhou = failed
agente-anexo-nao-lido = the engine cannot read this attachment
agente-anexo-motor-nao-le = { $motor } cannot read { $tipo ->
        [imagem] images
       *[pdf] scanned PDFs
    }. If you send it like this, the agent gets only the file name. Pick an engine with vision (an API one or a local model with vision) or remove the attachment.
agente-anexo-rota-nao-le = No engine in the chat route can read { $tipo ->
        [imagem] images
       *[pdf] scanned PDFs
    } of this kind. If you send it like this, the agent gets only the file name. Pick an engine with vision (an API one or a local model with vision) or remove the attachment.
agente-anexos = Attachments
agente-anexo-nao-guardado = { $detalhe } · not kept
agente-anexo-guardado-dica = Kept in the vault according to the attachment retention.
agente-anexo-nao-guardado-dica = The content was sent to the engine and was not kept (attachment retention).
agente-solte = Drop to attach

## Approval queue

agente-retiradas = { $n ->
        [one] { $n } proposal had already left the queue
       *[other] { $n } proposals had already left the queue
    }
agente-retiradas-texto = The same data changed before the approval; the change you made stays.
agente-aprovadas = { $n ->
        [one] { $n } proposal approved
       *[other] { $n } proposals approved
    }
agente-rejeitadas = { $n ->
        [one] { $n } proposal rejected
       *[other] { $n } proposals rejected
    }
agente-aprovadas-texto = The changes apply now and can be undone.
agente-falha-aplicar = A proposal could not be applied
agente-dados-mudaram = The data changed after the proposal.
agente-proposta-rotulo = Proposal: { $nome }
agente-sensivel-dica = Sensitive write: approve one by one
agente-selecionar-lote = Select for batch approval: { $nome }
agente-rejeitar = Reject
agente-aprovar = Approve
agente-aprovar-item = Approve this item
agente-carregando-diff = Loading the diff
agente-erro-diff = Could not load the diff
agente-revisadas = Recently reviewed
agente-desfeita = Change undone
agente-nao-desfez = Could not undo
agente-nao-aprovou = Could not approve
agente-nao-rejeitou = Could not reject
agente-autonomo = Autonomous mode
agente-autonomo-ligado = Autonomous mode on: reversible AI writes are applied without asking
agente-autonomo-desligado = Autonomous mode off: every AI write waits for your approval
agente-alterar-configurar = Change in Set up
agente-sensiveis-texto = Sensitive writes (investment trades, imports, deletions) always ask for approval one by one. Everything you approve can be undone.
agente-pendentes = Pending
agente-pendentes-n = Pending · { $n }
agente-selecionar-reversiveis = Select reversible ones
agente-rejeitar-selecionadas = Reject selected
agente-aprovar-selecionadas = Approve selected ({ $n })
agente-fila-vazia = No proposals waiting for approval.

## Alerts and routines

agente-filtro-abertos = Open (new and seen)
agente-filtro-novos = New
agente-filtro-vistos = Seen
agente-filtro-resolvidos = Resolved
agente-nao-atualizou = Could not update
agente-alerta-rotulo = Alert: { $titulo }
agente-alerta-abrir-asset = Open asset
agente-alerta-abrir-account = Open account
agente-alerta-abrir-budget = Open budget
agente-alerta-abrir-transactions = See transactions
agente-alerta-abrir-card-statement = See statement
agente-alerta-abrir-physical-asset = Open property
agente-alerta-abrir-allocation = Open allocation
agente-alerta-abrir-reports = Open reports
agente-alerta-abrir-sources = Open data sources
agente-alerta-abrir-asset-data = See asset data
agente-alerta-abrir-import = Import statement
agente-alerta-avancado = Advanced
agente-alerta-fontes = Sources of the numbers (Treasury tools, for agents): { $lista }
agente-marcar-visto = Mark as seen
agente-resolver = Resolve
agente-reabrir = Reopen
agente-apagar-alerta = Delete alert { $titulo }
agente-rotinas = Manager routines
agente-rotina-ligada = on
agente-rotina-desligada = off
agente-rotina-com-motor = with an AI engine
agente-rotina-local = local and deterministic (no engine in the route)
agente-rotina-ultima = last: { $quando }
agente-rotina-nunca = never
agente-rotina-andamento = { $rotina }: running
agente-nao-rodou = Could not run
agente-rodando = Running…
agente-rodar-agora = Run now
agente-caixa-alertas = Alert inbox
agente-marcar-todos-vistos = Mark all as seen
agente-estado = Status
agente-gravidade = Severity
agente-toda-gravidade = Any severity
agente-tipo = Type
agente-todos-tipos = All types
agente-sem-alertas = No alerts with these filters.

## Manager's memory

agente-gravada-voce = saved by you
agente-gravada-agente = saved by the agent (approved)
agente-esquecer = Forget
agente-esquecer-titulo = Forget this note
agente-esquecer-texto = The agent stops considering it in every conversation and routine. You can undo it in the change history.
agente-nota-esquecida = Note forgotten
agente-nao-esqueceu = Could not forget
agente-gravado-memoria = Saved to the manager's memory
agente-nao-gravou = Could not save
agente-memoria-placeholder = E.g.: don't suggest crypto
agente-lembrar = Remember
agente-memoria-rodape = Applies to all engines. The agent saves new notes only with your approval.
agente-memoria-vazia = The memory is empty.

## Learned procedures

agente-proc-atualizado = Procedure updated
agente-proc-gravado = Procedure saved
agente-proc-ocultados = { $n ->
        [one] { $n } sensitive snippet hidden (passwords, account or document numbers).
       *[other] { $n } sensitive snippets hidden (passwords, account or document numbers).
    }
agente-proc-editar = Edit procedure
agente-proc-novo = New procedure
agente-proc-dialogo-texto = Notes the agent checks before searching again. Do not write passwords, codes, or account or document numbers.
agente-assunto = Topic
agente-titulo = Title
agente-site = Site
agente-site-dica = Just the domain, e.g.: www.chase.com
agente-formato = Format
agente-passos = Steps
agente-passos-dica = Menus, filters, the export button and what the file contains.
agente-proc-por-voce = Saved by you on { $data }
agente-proc-pelo-agente = Saved by the agent on { $data }
agente-proc-pelo-cliente = Saved by { $cliente } on { $data }
agente-proc-pelo-agente-cliente = Saved by the agent ({ $cliente }) on { $data }
agente-proc-rotulo = Procedure: { $titulo }
agente-proc-funcionou-dica = Times the agent followed the note and it worked
agente-proc-funcionou = worked { $n }×
agente-proc-falhou-dica = Times the note did not work
agente-proc-falhou = failed { $n }×
agente-proc-arquivo = { $formato } file
agente-proc-observacao = Last remark: { $nota }
agente-proc-conversa = Conversation: { $titulo }
agente-proc-ver-conversa = See the conversation
agente-proc-atualizado-em = updated { $quando }
agente-proc-ultimo-uso = last used { $quando }
agente-proc-ultimo-uso-resultado = last used { $quando } ({ $resultado ->
        [worked] worked
       *[failed] failed
    })
agente-proc-apagar-titulo = Delete this procedure
agente-proc-apagar-texto = The agent stops using this note. You can undo it in the change history.
agente-proc-apagado = Procedure deleted
agente-procs-titulo = Learned procedures
agente-adicionar = Add
agente-procs-rodape = The agent saves and updates these notes on its own, without a proposal: everything stays in the change history and can be undone. They go along with related requests as data, never as instructions.
agente-buscar = Search
agente-procs-placeholder = E.g.: chase statement, b3, card bill
agente-todos-assuntos = All topics
agente-procs-filtro-vazio = No procedures with this filter.
agente-procs-vazio = The agent has not saved any procedures yet. When it finds out where to download a statement or where some data comes from, the note shows up here.
agente-procs = Procedures

## Tool history

agente-col-quando = When
agente-col-cliente = Client
agente-col-ferramenta = Tool
agente-col-parametros = Parameters
agente-col-resultado = Result
agente-col-duracao = Duration
agente-col-inicio = Start
agente-col-tarefa = Task
agente-ferramentas-titulo = Tools used by the AI
agente-fonte = Source
agente-chamadas = Calls
agente-todos-clientes = All clients
agente-so-erros = Only calls with errors
agente-historico-ferramentas = Tool history
agente-nenhuma-chamada = No calls recorded.
agente-jobs-titulo = Recent agent jobs
agente-jobs = Agent jobs
agente-jobs-vazio = No jobs in the last 30 minutes.

## AI settings

agente-ajustes-titulo = AI engines and routing
agente-ajustes-ia = AI settings
agente-nao-salvou = Could not save
agente-opcoes-titulo = Agent settings
agente-permissoes = Permissions
agente-autonomo-dica = Applies reversible AI writes without asking (categorize, create rule, budget). Sensitive writes keep asking for approval one by one; everything can be undone.
agente-mcp-escrita = Writes by external MCP clients
agente-mcp-escrita-dica = AI assistants and coding agents connected through MCP can propose writes (always as a proposal in the queue).
agente-guardar-historico = Keep the conversation history
agente-guardar-historico-dica = Without history, each question starts from scratch and nothing from the conversation stays in the database.
agente-rotinas-legenda = Routines (with the app open and unlocked)
agente-revisao-continua = Continuous review
agente-revisao-continua-dica = When importing statements: categorizes, points out duplicates and suggests rules (as proposals).
agente-fechamento-dica = Monthly report with 3 to 5 recommendations, delivered as an alert.
agente-alertas-dica = Budget running over, due dates, expensive debt, unbalanced portfolio.

## Routing by task

agente-rota-chat-nota = Also applies to categorization and PDF when they have no route of their own.
agente-rota-rotina-nota = Background routine: uses only the engines in this list.
agente-rota-estudo-nota = AI study of an asset. With no engine here, it uses the chat route. Only the asset data package goes to the engine.
agente-rota-categorizacao-nota = Without its own route, the continuous review runs locally and deterministically.
agente-nao-salvou-rota = Could not save the route
agente-rota = Route: { $tarefa }
agente-sem-motor-rota = No engine in this route.
agente-motor-removido = engine removed
agente-subir = Move { $motor } up
agente-descer = Move { $motor } down
agente-tirar-rota = Remove { $motor } from the route
agente-adicionar-motor-rota = Add an engine to the { $tarefa } route
agente-adicionar-fallback = Add fallback…
agente-adicionar-motor-mais = Add engine…
agente-roteamento = Routing by task
agente-rotas = Routes by task

## AI engines

agente-respondeu-em = ✓ answered in { $tempo }
agente-motor-rotulo = Engine { $nome }
agente-pronto = ready
agente-nao-alterou = Could not change
agente-modelo = Model
agente-url-local = Local URL
agente-endpoint = Endpoint
agente-api-key = API key
agente-chave-cadastrada = saved
agente-chave-cadastrada-dica = saved · { $dica }
agente-chave-nao-cadastrada = not saved
agente-dados = Data
agente-testar = Test
agente-excluir = Delete
agente-excluir-motor = Delete { $nome }
agente-excluir-motor-texto = The engine leaves the task routes and the saved key is deleted. The usage history stays.
agente-nao-excluiu = Could not delete
agente-motores-titulo = AI engines
agente-adicionar-motor = Add engine
agente-motores-rodape = APIs charge per token; local CLIs use the subscription you already pay for; local models send nothing out.
agente-motores-vazio = No engines added. Add an API, a local CLI or a local model.
agente-motores = Engines
agente-procurando-clis = Looking for CLIs in the PATH
agente-clis-encontradas = CLIs found
agente-clis-detectadas = Detected CLIs
agente-cli-instalada = installed
agente-cli-login-ok = · login found
agente-cli-sem-login = · not logged in
agente-cli-login-incerto = · login: can't tell without running it
agente-cli-nao-encontrada = not found
agente-cli-historico-fora = This CLI keeps the conversation history in its own folder, outside the vault. The Claude Code CLI runs without history.
agente-motor-cadastrado = Engine added
agente-motor-atualizado = Engine updated
agente-editar-motor = Edit engine · { $nome }
agente-novo-motor = New AI engine
agente-cadastrar = Add
agente-provider = Provider
agente-exige-chave = { $tipo } · requires an API key
agente-nome = Name
agente-nome-dica = How the engine shows up in the app.
agente-modelo-dica = Model names change over time; check with the provider.
agente-modelo-placeholder = e.g.: llama3.2
agente-modelo-opcional = Model (optional)
agente-modelo-opcional-dica = Goes into the template's { "{model}" } placeholder.
agente-url-servidor = Local server URL
agente-url-servidor-dica = http only for this computer (localhost/127.0.0.1).
agente-endpoint-opcional = Endpoint (optional)
agente-endpoint-padrao = Default: { $url }
agente-endpoint-dica = Base of the OpenAI-compatible API (https://…/v1).
agente-max-tokens = Max. tokens in the answer
agente-max-tokens-dica = Empty: 4,096.
agente-max-tokens-erro = Use a positive whole number.
agente-chave-dica = Saved encrypted in the vault; never shown again.
agente-cadastre-de-novo = enter it again
agente-sem-chave = no key
agente-substituir-chave = Type to replace the key
agente-cole-chave = Paste the provider's key
agente-chave-apagada = Key deleted
agente-apagar-chave = Delete the saved key
agente-template = Command template
agente-template-dica = Placeholders: { "{prompt}" }, { "{mcp_config}" }, { "{model}" }, { "{workdir}" }. Parts between [ ] disappear when the placeholder is empty.
agente-template-placeholder = my-cli -p { "{prompt}" }
agente-template-padrao = Empty uses the default, with the CLI's native tools turned off: { $template }
agente-motor-ligado = Engine on

## Spending limits

agente-limites-salvos = { $provider } limits saved
agente-limites-rotulo = Limits: { $provider }
agente-gasto = today { $hoje } · month { $mes }
agente-por-dia = Per day (USD)
agente-por-mes = Per month (USD)
agente-confirmar-acima = Confirm above (USD)
agente-valor-invalido = Invalid amount
agente-sem-limite = no limit
agente-nunca-perguntar = never ask
agente-limites-titulo = Spending limits
agente-limites-rodape = Costs in dollars, estimated from the price table. Above the limit the engine is skipped (fallback); above the warning, the chat asks for confirmation before sending.
agente-sem-provider = No API provider in use.
agente-limites-por-provider = Limits by provider
agente-limitar-outro = Limit another provider
agente-limitar-outro-mais = Limit another provider…

## Usage and cost

agente-periodo-1 = This month
agente-periodo-3 = Last 3 months
agente-periodo-6 = Last 6 months
agente-periodo-12 = Last 12 months
agente-tokens-entrada-saida = Tokens in / out
agente-tokens-entrada-saida-tooltip = Tokens (in / out)
agente-col-custo = Cost
agente-erros = Errors
agente-sem-uso = No usage in the period.
agente-uso-titulo = Usage and cost
agente-periodo = Period
agente-moeda = Currency
agente-fonte-uso = ai_usage · price table
agente-custo-estimado = Estimated cost
agente-tokens = Tokens
agente-sem-preco = { "*" } { $n ->
        [one] { $n } call has no price set for the model
       *[other] { $n } calls have no price set for the model
    }: cost left out of the total. Add the price in the table below.
agente-custo-mes = Estimated cost per month
agente-custo-fonte = Source: recorded calls × price table (local CLIs and models cost zero in the app)
agente-custo-ia-mes = Estimated AI cost per month
agente-mes = Month
agente-por = By
agente-agrupar = Group usage by
agente-tarefa = Task
agente-uso-por-provider = Usage by provider
agente-uso-por-motor = Usage by engine
agente-uso-por-tarefa = Usage by task

## Prices per model

agente-preco-salvo = Price saved
agente-preco-de = Price · { $modelo }
agente-novo-preco = New price
agente-modelo-prefixo = Model or prefix
agente-modelo-prefixo-dica = { "*" } = provider default
agente-entrada-mtok = Input (USD / Mtok)
agente-saida-mtok = Output (USD / Mtok)
agente-cache-mtok = Cache (USD / Mtok)
agente-opcional = Optional
agente-entrada = Input
agente-saida = Output
agente-cache = Cache
agente-origem = Source
agente-editado = edited
agente-padrao = default
agente-acoes = Actions
agente-apagar-preco = Delete the price for { $modelo }
agente-precos-titulo = Prices per model · USD per million tokens
agente-precos = Prices per model
agente-precos-vazio = No prices set: the cost of the calls is left out of the total.

## History and attachments in the vault

agente-ret-nenhum-dica = The file stays only in the app's memory while the vault is open (the agent rereads it in the next turns of the conversation) and does not go into the vault; the conversation keeps only the name and size.
agente-ret-7-dica = Keeps the files for 7 days and then deletes them (the daily cleanup handles it).
agente-ret-sempre-dica = Keeps the files forever, until you delete the conversation.
agente-n-dias = { $n } days
agente-1-ano = 1 year
agente-nunca = Never
agente-faxina-feita = Cleanup done
agente-faxina-texto = { $conversas ->
        [one] { $conversas } conversation
       *[other] { $conversas } conversations
    } and { $anexos ->
        [one] { $anexos } expired attachment
       *[other] { $anexos } expired attachments
    } deleted.
agente-faxina-texto-encolheu = { $conversas ->
        [one] { $conversas } conversation
       *[other] { $conversas } conversations
    } and { $anexos ->
        [one] { $anexos } expired attachment
       *[other] { $anexos } expired attachments
    } deleted; the vault shrank by { $tamanho }.
agente-nao-limpou = Could not clean up
agente-hist-titulo = History and attachments in the vault
agente-anexos-enviados = Attachments sent to the agent
agente-guardar-anexos = Keep attachments in the vault
agente-nao-guardar = Don't keep
agente-7-dias = 7 days
agente-sempre = Always
agente-recomendado = Recommended: the vault stays light and no document sent to the agent is kept.
agente-nao-recomendado = Not recommended: images and PDFs make the encrypted database large (slower, bigger backups) and keep copies of sensitive documents. Prefer "Don't keep".
agente-nao-recomendado-hoje = Not recommended: images and PDFs make the encrypted database large (slower, bigger backups) and keep copies of sensitive documents. Prefer "Don't keep". Today: { $tamanho } in attachments.
agente-apagar-sem-uso = Delete conversations unused for
agente-faxina-diaria = The cleanup runs once a day with the vault open. Pinned conversations stay.
agente-faxina-nunca = Conversations stay until you delete them. The vault may grow over time.
agente-retencao-conversas = Conversation retention
agente-conversas-fixadas = { $n } ({ $fixadas } pinned)
agente-mensagens = Messages
agente-anexos-guardados = { $n } ({ $guardados } kept)
agente-historico = History
agente-cofre = Vault
agente-cofre-livres = { $tamanho } ({ $livres } free)
agente-ultima-faxina = Last cleanup
agente-ainda-nao = not yet
agente-limpar-agora = Clean up now
agente-limpar-agora-dica = Applies the rules above right away and compacts the vault if there is free space.
agente-conversas-abrir = Show conversations
agente-conversas-recolher = Hide conversations
agente-conversa-nova-titulo = New conversation
agente-exportar = Export
agente-exportar-salvar = Save the conversation
agente-exportar-md-dica = Save the conversation as Markdown (.md)
agente-exportar-pdf-dica = Save the conversation as PDF
agente-exportar-feito = Conversation exported
agente-exportar-feito-texto = Saved to { $caminho }.
agente-exportar-feito-segredos = Saved to { $caminho }. { $n ->
    [one] 1 key or password was removed.
   *[other] { $n } keys or passwords were removed.
}
agente-exportar-erro = Could not export the conversation
agente-exportar-md = Markdown
agente-mensagens-conversa = Conversation messages

# Imagens com texto lido pelo OCR: o que vai ao motor.
agente-anexo-texto-lido = text read on this computer
agente-imagem-modo-rotulo = What to send from the images
agente-imagem-modo-pergunta = Text read on this computer ({ $confianca }% confidence). Send:
agente-imagem-modo-texto = Text only
agente-imagem-modo-imagem = Image only
agente-imagem-modo-ambos = Both
agente-imagem-so-texto = This engine doesn't read images: the text read on this computer goes instead ({ $confianca }% confidence).

## Extra MCPs (this vault only)

agente-mcpx-titulo = Extra MCPs (this vault only)
agente-mcpx-adicionar = Add MCP
agente-mcpx-vazio = No extra MCPs. Register an outside MCP server (for example, one that controls your phone) for this vault's agent to use.
agente-mcpx-explica = Each tool call asks for your confirmation in the chat, unless "Allow without asking" is on. This applies to every engine, including CLI ones: the tools reach them through Treasury's own MCP.
agente-mcpx-seguranca = A local command runs on this computer with your user's permissions. Only add servers you trust.
agente-mcpx-lista = This vault's extra MCPs
agente-mcpx-linha = MCP { $nome }
agente-mcpx-tipo-stdio = Local command
agente-mcpx-tipo-http = HTTP address
agente-mcpx-ferramentas-prefixo = Tools
agente-mcpx-permitir-sem-perguntar = Allow without asking
agente-mcpx-excluir = Delete { $nome }
agente-mcpx-excluir-texto = The server leaves this vault's agent and the secret values saved for it are erased.
agente-mcpx-nao-excluiu = Could not delete the MCP
agente-mcpx-testando = Connecting… (may take up to 20 s)
agente-mcpx-ferramentas-encontradas =
    { $n ->
        [one] 1 tool found
       *[other] { $n } tools found
    }
agente-mcpx-ferramentas = MCP tools
agente-mcpx-sem-ferramentas = Connected, but the server offers no tools.
agente-mcpx-novo = New extra MCP
agente-mcpx-editar = Edit { $nome }
agente-mcpx-rotulo = Label
agente-mcpx-rotulo-dica = How it shows in the chat (e.g. My phone).
agente-mcpx-rotulo-erro = Enter a label of up to 60 characters.
agente-mcpx-nome = Name
agente-mcpx-nome-dica = Tool prefix ({ $exemplo }).
agente-mcpx-nome-erro = Use 2 to 24 lowercase letters, digits or _, starting with a letter (and not "treasury").
agente-mcpx-tipo = Type
agente-mcpx-comando = Command
agente-mcpx-comando-dica = The program that starts the MCP server (e.g. npx or the full path).
agente-mcpx-comando-erro = Enter the command.
agente-mcpx-argumentos = Arguments
agente-mcpx-argumentos-dica = One per line.
agente-mcpx-variaveis = Environment variables
agente-mcpx-url = URL
agente-mcpx-url-dica = https://, or http://localhost for a server on this computer.
agente-mcpx-url-erro = The address must start with https:// (or http://localhost).
agente-mcpx-cabecalhos = Headers
agente-mcpx-cabecalhos-dica = An access key goes here, marked as secret: { $exemplo }
agente-mcpx-chave = Name
agente-mcpx-valor = Value
agente-mcpx-secreta = Secret
agente-mcpx-manter-segredo = leave blank to keep
agente-mcpx-variavel-n = Variable { $n }
agente-mcpx-cabecalho-n = Header { $n }
agente-mcpx-adicionar-variavel = Add variable
agente-mcpx-adicionar-cabecalho = Add header
agente-mcpx-remover = Remove
agente-mcpx-ligado = On (this vault's agent may use it)
agente-mcpx-permitir-dica = The agent calls the tools without asking for confirmation.
agente-mcpx-salvo = MCP saved
agente-mcpx-pedido-rotulo = External tool confirmation
agente-mcpx-pedido = The agent wants to use { $ferramenta } from the MCP { $servidor }
agente-mcpx-parametros = Parameters
agente-mcpx-ver-tudo = Show all
agente-mcpx-recolher = Collapse
agente-mcpx-permitir = Allow
agente-mcpx-recusar = Decline
agente-mcpx-permitida = Allowed
agente-mcpx-recusada = Declined
agente-mcpx-aguardando = Waiting for your answer
agente-mcpx-nao-respondeu = Could not answer the request

## Ajustes › IA tabs

agente-ajustes-abas = Parts of the AI settings
agente-ajustes-aba-motores = Engines
agente-ajustes-aba-motores-dica = Which AI engines Treasury uses and which one handles each task, with the fallback when one fails.
agente-ajustes-aba-acesso = Agent access
agente-ajustes-aba-acesso-dica = What the agent may do on its own, what it sees and which outside tools are within its reach.
agente-ajustes-aba-custos = Costs
agente-ajustes-aba-custos-dica = What the AI has cost, the spending limits and the price of each model.
agente-ajustes-aba-historico = History
agente-ajustes-aba-historico-dica = The conversations and attachments kept in the vault, and for how long.

# User CLI MCPs (Settings › AI)
agente-climcp-titulo = MCPs from your CLIs
agente-climcp-explica = MCPs you already have in Claude, Cursor, Grok, Codex, Gemini or OpenCode. By default none of them joins this vault's agent conversations: only Treasury and the extra MCPs. Turn on only the ones the agent needs.
agente-climcp-seguranca = An MCP allowed here is called by the CLI directly, without Treasury's confirmation or audit.
agente-climcp-vazio = No installed CLI loads other MCPs.
agente-climcp-lista = CLI MCPs
agente-climcp-nao-encontrado = not found in the CLIs now
agente-climcp-liberar = Use { $nome } in conversations
agente-mcpx-permitir-conversa = Allow in this conversation
agente-mcpx-permitir-conversa-dica = Allows every tool of the MCP { $servidor } until the end of this conversation, without asking again. Lasts until the vault locks; each call is still audited.
agente-mcpx-permitida-conversa = Allowed in this conversation ({ $servidor })
agente-tempo-cli = Time limit for CLI answers
agente-tempo-cli-dica = Long collections (extra MCPs, a device) need more time. Near the end, the agent is warned to save what it collected; if time runs out, the part already done stays in the conversation.
agente-tempo-cli-minutos = { $n } min

## Agent question in the conversation (ask_user)

agente-pergunta-rotulo = Question from the agent
agente-pergunta-uma = Choose one option.
agente-pergunta-varias = Choose one or more options.
agente-pergunta-recomendado = Recommended
agente-pergunta-outro = Other
agente-pergunta-outro-texto = Your answer
agente-pergunta-outro-dica = Type your answer
agente-pergunta-enviar = Send
agente-pergunta-aguardando = The agent is waiting for your answer.
agente-pergunta-respondeu = You answered: { $resposta }
agente-pergunta-sem-resposta = No answer
agente-pergunta-nao-enviou = Could not send the answer
