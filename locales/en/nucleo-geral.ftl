# Core texts from the smaller modules (docs/i18n-design.md, "Textos do núcleo"): family
# (members and institutions), quick unlock, backups, terms, support, developer feedback, demo
# vault, database, export, change history, trash, keychain and config.toml. Errors are
# lowercase, like a log line (the interface adds the capital letter).
# Revision: AI draft, not yet reviewed by a native speaker.

## Family: members and institutions (`household`)

nucleo-geral-membro = member { $id }
nucleo-geral-erro-membro-repetido = there's already a member named { $nome }
nucleo-geral-erro-arquivado-principal = an archived member can't be the main member
nucleo-geral-erro-principal-arquivado = the main member can't be archived; choose another main member first
nucleo-geral-instituicao = institution { $id }
nucleo-geral-erro-codigo-instituicao = invalid institution code: { $codigo }
nucleo-geral-erro-site = invalid website address: { $site }
nucleo-geral-erro-instituicao-em-uso = the institution { $instituicao } is used by { $n ->
        [one] { $n } account
       *[other] { $n } accounts
    }; change the institution of those accounts before deleting it
nucleo-geral-erro-bancos-incompleta = incomplete bank list; kept the previous one
nucleo-geral-erro-bancos = Banco Central bank list: { $erro }
nucleo-geral-motivo-familia = Family details changed
nucleo-geral-motivo-membro-criado = Member added: { $nome }
nucleo-geral-motivo-membro-alterado = Member changed
nucleo-geral-motivo-membro-arquivado = Member archived
nucleo-geral-motivo-membro-reativado = Member reactivated
nucleo-geral-motivo-instituicao-criada = Institution added: { $nome }
nucleo-geral-motivo-instituicao-alterada = Institution changed
nucleo-geral-motivo-instituicao-apagada = Institution deleted

## Quick unlock: PIN, security key and split recovery (`unlock`)

nucleo-geral-chave-fisica = security key
nucleo-geral-arquivo-chaves = vault key file
nucleo-geral-erro-chaves-ilegivel = unreadable vault key file
nucleo-geral-erro-cifrar = encryption failed
nucleo-geral-erro-recuperacao-ilegivel = unreadable recovery key
nucleo-geral-erro-sem-arquivo-chaves = the vault key file (treasury.db.keys) isn't next to the database
nucleo-geral-erro-sem-recuperacao-dividida = split recovery hasn't been set up for this vault
nucleo-geral-erro-chave-cofre-ilegivel = unreadable vault key
nucleo-geral-erro-resposta-chave-fisica = invalid response from the security key
nucleo-geral-erro-pin-tamanho = the PIN must have 6 to 12 digits
nucleo-geral-erro-pin-facil = PIN too easy (repeated or sequential digits): choose another one
nucleo-geral-erro-pin-sem-keychain = the PIN needs the system Keychain, which isn't available here
nucleo-geral-erro-pin-desligado = the PIN isn't turned on on this computer
nucleo-geral-erro-pin-ilegivel-keychain = unreadable PIN item in the Keychain
nucleo-geral-erro-pin-ilegivel = unreadable PIN item
nucleo-geral-erro-pin-bloqueado = wrong PIN { $tentativas } times: the PIN has been turned off. Sign in with the master password.
nucleo-geral-erro-pin-incorreto = Incorrect PIN. { $restam ->
        [one] { $restam } attempt left
       *[other] { $restam } attempts left
    } before the master password is required.
nucleo-geral-erro-divisao = invalid split
nucleo-geral-erro-parte-invalida = invalid recovery part
nucleo-geral-erro-duas-partes = enter at least two parts
nucleo-geral-erro-partes-outro-conjunto = the parts aren't from the same set (check that they belong to this vault)
nucleo-geral-erro-parte-repetida = the same part was entered twice
nucleo-geral-erro-partes-nao-reconstroem = the parts don't rebuild the key (was one of them mistyped?)

## Backups (`backup`)

nucleo-geral-backup = backup { $id }
nucleo-geral-arquivo-backup = backup file
# Stored as the reason why a backup found in the folder hasn't been verified yet.
nucleo-geral-backup-encontrado = found in the folder; verify it before using
nucleo-geral-erro-backup-hora = backup hour must be between 0 and 23
nucleo-geral-erro-backup-senha-antes = set the backup password before turning it on
nucleo-geral-erro-backup-senha-curta = the backup password needs at least { $minimo } characters
nucleo-geral-erro-backup-criar = couldn't create the backup: { $erro }
nucleo-geral-erro-backup-hash = the SHA-256 hash doesn't match: the file was changed or is corrupted
nucleo-geral-erro-backup-senha-antiga = enter the password that was used when this backup was made
nucleo-geral-erro-backup-senha = enter the backup password

## Terms of use (`terms`): what changed in each version

nucleo-geral-termos-1-0 = First version of the terms.
nucleo-geral-erro-termos-mudaram = the terms have changed (current version: { $versao }); read them again before accepting

## Supporting the project (`support`)

nucleo-geral-erro-pix-chave = the Pix key must be a random key
nucleo-geral-erro-pix-recebedor = name and city of the Pix recipient
nucleo-geral-erro-apoio-so-interface = the support reminder can only be changed in the app
nucleo-geral-erro-evento = unknown event: { $evento }

## Developer feedback (`devfeedback`)

nucleo-geral-feedback = feedback { $id }
nucleo-geral-erro-feedback-vazio = enter `{ $campo }`
nucleo-geral-erro-feedback-longo = `{ $campo }` can have at most { $maximo } characters
nucleo-geral-erro-feedback-importancia = importance must be between 1 and 1000
nucleo-geral-erro-modo-desenvolvedor = developer mode is off
nucleo-geral-erro-feedback-so-interface = saving the feedback to a file is only available in the app; use export_dev_feedback to get the Markdown
nucleo-geral-erro-feedback-ja-fechado = this feedback is already resolved or discarded
nucleo-geral-erro-feedback-motivo = give the reason: what fixed it and where (version, commit or change)
nucleo-geral-feedback-resolvido-por = Resolved by { $cliente }: { $motivo }
nucleo-geral-feedback-reforcado = There was already a similar open feedback ("{ $titulo }"): it was reinforced (importance { $importancia }) instead of duplicated.
nucleo-geral-feedback-registrado = Feedback recorded for the developer (Developer screen).
nucleo-geral-erro-feedback-itens = `{ $campo }` accepts at most { $maximo } items
nucleo-geral-erro-feedback-posicao = the position starts at 1 (the top of the column)
nucleo-geral-erro-feedback-prioridade = enter the importance, the position in the column or both
nucleo-geral-erro-feedback-vinculo-proprio = a feedback cannot be linked to itself
nucleo-geral-feedback-vinculo = link between feedback { $id } and { $outro }
nucleo-geral-feedback-status-nota = { $status ->
    [aberto] Reopened
    [em_andamento] In progress
    [resolvido] Resolved
   *[descartado] Discarded
}
nucleo-geral-feedback-mesclado = Duplicate merged here: { $titulo }
nucleo-geral-feedback-duplicado-de = duplicate of "{ $titulo }"
nucleo-geral-motivo-feedback-registrar = Developer feedback: record
nucleo-geral-motivo-feedback-reforcar = Developer feedback: reinforce
nucleo-geral-motivo-feedback-resolver = Developer feedback: mark as resolved
nucleo-geral-motivo-feedback-status = Developer feedback: move to another column
nucleo-geral-motivo-feedback-prioridade = Developer feedback: change the priority
nucleo-geral-motivo-feedback-editar = Developer feedback: edit
nucleo-geral-motivo-feedback-vincular = Developer feedback: link
nucleo-geral-motivo-feedback-desvincular = Developer feedback: remove link
nucleo-geral-motivo-feedback-apagar = Developer feedback: delete
nucleo-geral-feedback-md-contexto = Treasury feedback `{ $id }` · area { $area } · type { $tipo } · priority { $prioridade } ({ $reforcos } reinforcements) · { $status }.
nucleo-geral-feedback-md-problema = Problem
nucleo-geral-feedback-md-esperado-obtido = Expected × actual
nucleo-geral-feedback-md-esperado = Expected
nucleo-geral-feedback-md-obtido = Actual
nucleo-geral-feedback-md-passos = How to reproduce
nucleo-geral-feedback-md-proposta = Proposal
nucleo-geral-feedback-md-aceite = Acceptance criteria
nucleo-geral-feedback-md-sem-aceite = Not provided: define a test that proves the fix.
nucleo-geral-feedback-md-onde = Where to change
nucleo-geral-feedback-md-chamadas = Calls that led to the problem
nucleo-geral-feedback-md-relacionados = Related
nucleo-geral-feedback-md-reforcos = Reinforcements and notes
nucleo-geral-feedback-md-fechar = When done, run the tests and mark it as resolved in the Treasury MCP: `resolve_dev_feedback` with `id` = `{ $id }` and the reason (what changed, version or commit).

## Demo vault (`demo`): assistant errors and the sample data, created in the installation
## language (brands and proper names stay as they are)

nucleo-geral-erro-demo-meses = the history goes from 3 to 36 months
nucleo-geral-erro-demo-adultos = enter one or two adults
nucleo-geral-erro-demo-filhos = up to 4 children
nucleo-geral-erro-demo-nome = enter each person's name
nucleo-geral-erro-demo-nome-repetido = repeated name: { $nome }
nucleo-geral-erro-demo-renda = enter the income of at least one adult
nucleo-geral-erro-demo-banco = enter the bank of the main account
nucleo-geral-erro-demo-valores = amounts must be between zero and 100 million
nucleo-geral-erro-demo-financiamento = enter the mortgage installment
nucleo-geral-erro-demo-parcelas-carro = enter how many car installments are left (1 to 96)
nucleo-geral-erro-demo-cofre-com-dados = the demo vault can only be generated in a vault with no transactions
nucleo-geral-erro-demo-so-interface = only the app can generate the demo vault
nucleo-geral-erro-demo-interno = demo vault: { $erro }
nucleo-geral-erro-demo-categoria = default category missing: { $caminho }
nucleo-geral-erro-demo-data = invalid date
nucleo-geral-motivo-demo = Demo vault generated
nucleo-geral-demo-conta = { $nome } account
nucleo-geral-demo-cartao = { $nome } card
nucleo-geral-demo-reserva = { $nome } savings
nucleo-geral-demo-corretora = XP brokerage
nucleo-geral-demo-previdencia = { $nome } pension plan
nucleo-geral-demo-cliente-a = Client A
nucleo-geral-demo-cliente-b = Client B
nucleo-geral-demo-cliente-c = Client C
nucleo-geral-demo-projeto = Project
nucleo-geral-demo-recebimento = Payment from { $nome }
nucleo-geral-demo-salario = Salary { $nome }
nucleo-geral-demo-13-1 = 13º salary (1st installment)
nucleo-geral-demo-13-2 = 13º salary (2nd installment)
nucleo-geral-demo-aluguel-recebido = Rent received (apartment)
nucleo-geral-demo-aluguel = Rent
nucleo-geral-demo-condominio = Condo fee
nucleo-geral-demo-luz = Electricity bill
nucleo-geral-demo-agua = Water bill
nucleo-geral-demo-internet = Fiber internet
nucleo-geral-demo-celular = Phone plan
nucleo-geral-demo-padaria = Neighborhood bakery
nucleo-geral-demo-estacionamento = Parking
nucleo-geral-demo-ipva = IPVA and vehicle registration
nucleo-geral-demo-seguro-carro = Car insurance
nucleo-geral-demo-revisao = Car service and oil change
nucleo-geral-demo-bilhete = Transit card top-up
nucleo-geral-demo-mensalidade = School tuition ({ $nome })
nucleo-geral-demo-material = School supplies ({ $nome })
nucleo-geral-demo-plano-saude = Health insurance
nucleo-geral-demo-restaurante = Restaurant
nucleo-geral-demo-pizzaria = Pizzeria
nucleo-geral-demo-churrascaria = Steakhouse
nucleo-geral-demo-teatro = Theater
nucleo-geral-demo-parque = Park
nucleo-geral-demo-passeio = Outing
nucleo-geral-demo-museu = Museum
nucleo-geral-demo-viagem = Vacation trip
nucleo-geral-demo-financiamento-apartamento = Apartment mortgage
nucleo-geral-demo-financiamento-carro = Car loan
# The contract name comes in lowercase ("car loan installment").
nucleo-geral-demo-parcela = { $nome } installment
nucleo-geral-demo-ativo-nota = Demo vault asset: made-up prices.
nucleo-geral-demo-rendimento-reserva = Savings interest
nucleo-geral-demo-aporte-reserva = Transfer to savings
nucleo-geral-demo-aporte-corretora = Transfer to the broker
nucleo-geral-demo-aporte-previdencia = Pension plan contribution
nucleo-geral-demo-meta-reserva = Emergency fund

## Database: size, maintenance and purge (`maintenance`)

nucleo-geral-grupo-historico = change history
nucleo-geral-grupo-lancamentos = transactions
nucleo-geral-grupo-contas = accounts and cards
nucleo-geral-grupo-carteira = portfolio
nucleo-geral-grupo-cotacoes = prices, indices and references
nucleo-geral-grupo-analise = asset analysis (CVM, B3, SEC)
nucleo-geral-grupo-agente = agent
nucleo-geral-grupo-dividas = debts
nucleo-geral-grupo-patrimonio = physical assets
nucleo-geral-expurgo-historico = Old change history
nucleo-geral-expurgo-historico-consequencia = Deleted changes can no longer be undone; pending proposals stay.
nucleo-geral-expurgo-auditoria = Agent and MCP audit log
nucleo-geral-expurgo-auditoria-consequencia = The record of who looked at or proposed what in that period is removed.
nucleo-geral-expurgo-conversas = Old agent conversations
nucleo-geral-expurgo-conversas-consequencia = Unpinned conversations not used in the period (and their attachments) are removed.
nucleo-geral-expurgo-mercado = Market caches
nucleo-geral-expurgo-mercado-consequencia = Old intraday prices and the Fundamentus cache; they come back on the next lookup.
nucleo-geral-expurgo-analise = Asset analysis data
nucleo-geral-expurgo-analise-consequencia = Fundamentals, universe prices, documents and bank indicators downloaded from CVM, B3, SEC and Banco Central; the next collection downloads everything again (about 230 MB over the network).
nucleo-geral-erro-expurgo-backup = the backup before the purge didn't pass verification ({ $erro }); nothing was deleted
nucleo-geral-referencias-quebradas = { $n ->
        [one] { $n } broken reference
       *[other] { $n } broken references
    } between tables
nucleo-geral-tabela = table { $tabela }
nucleo-geral-ordem-chave-primaria = primary key
nucleo-geral-mostrando-linhas = showing the first { $limite } of { $total } rows
nucleo-geral-segredos-ocultos = keys, tokens and secrets are hidden
nucleo-geral-erro-banco-so-interface = database maintenance and purge are only available in the app
nucleo-geral-erro-banco-leitura = database opened read-only

## Open-format export (`export`): errors, the JSON `about` and the transactions CSV

nucleo-geral-erro-exportar-caminho = choose a full path for the file
nucleo-geral-erro-exportar-nome = enter the file name
nucleo-geral-erro-exportar-pasta-destino = invalid destination folder
nucleo-geral-erro-exportar-pasta-inexistente = the folder { $pasta } doesn't exist
nucleo-geral-erro-exportar-destino-pasta = the destination is a folder; enter the file name
nucleo-geral-erro-exportar-pasta-treasury = choose a folder outside Treasury's data and backup folders
nucleo-geral-erro-exportar-pasta = invalid folder
nucleo-geral-erro-csv = error writing the CSV: { $erro }
nucleo-geral-exportacao-descricao = Full export of Treasury data in an open format. This file is NOT encrypted.
nucleo-geral-exportacao-tabelas = Each table is a list of objects with the columns of the database schema.
nucleo-geral-exportacao-dinheiro = Money amounts are integers in the currency's smallest unit (cents), with the currency in the currency column. In txn.amount, + goes into and − goes out of the account.
nucleo-geral-exportacao-decimais = Exact quantities and rates are decimal text.
nucleo-geral-exportacao-datas = Transaction dates as YYYY-MM-DD (no time zone); timestamps in UTC (RFC 3339).
nucleo-geral-exportacao-ids = UUID v7 identifiers as text; references between tables use these ids.
nucleo-geral-exportacao-json = Columns such as rule.conditions, rule.actions and setting.value hold JSON as text.
nucleo-geral-exportacao-binarios = Binary columns (see blob_columns) are in standard base64.
nucleo-geral-exportacao-segredos = Secrets (backup password, API keys) are never exported.
nucleo-geral-csv-data = Date
nucleo-geral-csv-conta = Account
nucleo-geral-csv-descricao = Description
nucleo-geral-csv-favorecido = Payee
nucleo-geral-csv-categoria = Category
nucleo-geral-csv-titular = Member
nucleo-geral-csv-valor = Amount
nucleo-geral-csv-moeda = Currency
nucleo-geral-csv-tipo = Type
nucleo-geral-csv-situacao = Status
nucleo-geral-csv-tags = Tags
nucleo-geral-csv-notas = Notes
nucleo-geral-csv-vencimento-fatura = Card bill due date
nucleo-geral-csv-parcela = Installment
nucleo-geral-csv-divisao = Split
nucleo-geral-csv-id = Id
nucleo-geral-csv-sem-categoria = Uncategorized
nucleo-geral-csv-receita = Income
nucleo-geral-csv-despesa = Expense
nucleo-geral-csv-transferencia = Transfer
nucleo-geral-csv-pagamento-fatura = Card bill payment
nucleo-geral-csv-investimento = Investment
nucleo-geral-csv-ajuste = Adjustment
nucleo-geral-csv-previsto = Planned
nucleo-geral-csv-efetivado = Cleared
nucleo-geral-csv-conciliado = Reconciled

## Change history (`changes`)

nucleo-geral-conjunto = change set { $id }
nucleo-geral-erro-sem-motivo = every change needs a reason
nucleo-geral-erro-aprovar = only pending changes can be approved
nucleo-geral-erro-rejeitar = only pending changes can be rejected
nucleo-geral-erro-desfazer = only applied changes can be undone
nucleo-geral-erro-refazer = only undone changes can be redone
nucleo-geral-erro-alteracoes-posteriores = later changes modified this data; first undo the more recent changes that touch the same rows

## Trash, keychain and config.toml (`trash`, `keychain`, `config`)

nucleo-geral-arquivo = file { $caminho }
nucleo-geral-erro-caminho-sem-nome = path without a file name
nucleo-geral-erro-keychain-indisponivel = keychain not available on this system
nucleo-geral-erro-keychain-negado = keychain access denied or keychain locked
nucleo-geral-erro-keychain = keychain error: { $erro }
nucleo-geral-erro-config = invalid config.toml: { $erro }

# Backup recovery (docs/backup-recovery.md)
nucleo-geral-erro-backup-fora-da-pasta = This file is not in this vault's backup folder.
nucleo-geral-erro-backup-destino-igual = Choose another place for the copy: that is the backup itself.
nucleo-geral-erro-backup-copia-diferente = The copy came out different from the backup; try again or choose another place.
nucleo-geral-erro-backup-nao-e-backup = This file doesn't look like a Treasury backup (backups are encrypted databases ending in .db.enc).

## Background tasks (`tasks`)

nucleo-geral-tarefa-nao-encontrada = task not found (it may have already finished)
nucleo-geral-tarefa-nao-cancelavel = this task cannot be cancelled
nucleo-geral-tarefa-cofre-bloqueado = stopped because the vault was locked; it picks up where it left off after you unlock
