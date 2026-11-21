# Email import: the Email tab of Transactions, accounts (IMAP mailbox or local folder), rules and
# audit log (docs/email-ingest.md, docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.
# Keep as is in every language: the word `membros` in sender lists and the groups `valor`,
# `descricao` and `data` of the body template (the core recognizes these names), INBOX and the
# server flags \Junk, \Trash…

## Item types

email-tipo-extrato = Statement
email-tipo-fatura-pdf = PDF card bill
email-tipo-nota = Brokerage note
email-tipo-b3 = B3 report
email-tipo-previdencia = Pension plan statement
email-tipo-holerite = Payslip
email-tipo-cupom = Cupom fiscal (receipt)
email-tipo-imagem = Image
email-tipo-compra = Purchase notice
email-tipo-pix = Pix notice
email-tipo-modelo = From the rule's template
email-tipo-desconhecido = Not recognized

## Message and item status

email-status-revisar = To review
email-status-importada = Imported
email-status-repetida = Duplicate
email-status-falhou = Failed
email-status-nao-reconhecida = Not recognized
email-status-ignorada = Ignored
email-status-ia-lendo = AI reading

## What the rule does and the AI

email-acao-revisar = Review
email-acao-auto = Apply automatically
email-acao-ignorar = Ignore
email-ia-nunca = Never
email-ia-a-pedido = On request
email-ia-auto = Automatically (up to 10 a day)

## Retention (deleting from the provider)

email-retencao-nunca = Never delete
email-retencao-sucesso = After it's imported (or a duplicate)
email-retencao-dias = After a few days
email-retencao-ambos = Whichever comes first
email-estado-desligado = Deletes nothing
email-estado-teste = Test mode
email-estado-confirmacao = Awaiting confirmation
email-estado-ativo = Deleting

## Audit log

email-auditoria-busca = Fetch
email-auditoria-recebida = Received
email-auditoria-importada = Imported
email-auditoria-repetida = Duplicate
email-auditoria-apagada = Deleted
email-auditoria-apagaria = Test: would delete
email-auditoria-nao-apagou = Not deleted
email-auditoria-ia = AI
email-auditoria-revisao = Review
email-auditoria-erro = Error

## Email queue

email-fila = Email queue
email-mensagens = Messages
email-para-revisar = To review
email-tudo = All
email-vazio-kicker = Email import
email-vazio-titulo = Get statements, card bills and receipts by email
email-vazio-descricao = Create a mailbox just for Treasury (forward your bank's emails to it) or choose a folder on your computer where you save attachments and .eml messages. Whatever passes the filters shows up here; nothing is deleted unless you turn it on and confirm.
email-configurar = Set up email
email-nada-revisar = Nothing waiting for review. Accounts check on their own at each interval, with the app open and the vault unlocked.
email-nenhuma-mensagem = No messages yet.

email-mensagem = Message
email-sem-assunto = (no subject)
email-encaminhada-por = { $original } · forwarded by { $remetente }
email-pasta-principal = Main folder
email-pasta = Folder { $pasta }
email-regra-nome = rule "{ $regra }"
email-sem-regra = no rule (account default)
email-autenticado = Authenticated sender
email-nao-autenticado = Unauthenticated sender
email-ignorar-mensagem = Ignore the message

## Message items

email-cartao-final = card ending in { $final }
email-ia-propostas = { $n ->
        [one] { $n } proposal in the agent's approval queue
       *[other] { $n } proposals in the agent's approval queue
    }
email-conta-lancamento = Account for the transaction
email-escolha-conta = Choose the account
email-lancar = Record
email-importar-cupom = Import receipt
email-importar = Import
email-abrir-importador = Open in the importer
email-pedir-ia = Ask the AI
email-anexar-agente = Attach to the agent
email-ja-importei = Already imported
email-ignorar = Ignore
email-importado = Imported
email-ja-estava = Already in Treasury
email-ficou-revisao = Kept for review
email-erro-abrir = Couldn't open the item
email-erro-anexar = Couldn't attach
# Text sent ready-made to the AI agent.
email-agente-chegou = Arrived by email (from { $remetente }).

## Deletion test mode

email-teste-ate = Deletion test mode until { $data }: nothing is deleted, only logged.
email-teste-acabou = The test mode is over. Deletion only starts after you confirm; until then, it stays in test mode.
email-teriam-saido = { $n ->
        [one] { $n } message would have been removed (see the audit log).
       *[other] { $n } messages would have been removed (see the audit log).
    }
email-nada-teria-saido = Nothing would have been removed so far.
email-confirmar-exclusao = Confirm deletion (can't be undone)

## Accounts

email-contas = Accounts
email-nova-conta = New account
email-pasta-local = Local folder
email-conta-nome = Account { $nome }
email-desligada = off
email-a-cada = every { $min } min
email-n-para-revisar = { $n } to review
email-buscando = Checking now…
email-ultima-busca = Last check { $quando }: { $resultado }
email-busca-resumo = { $novas } new, { $fila } queued, { $recusadas } rejected, { $aplicadas } applied automatically
email-ainda-nao-buscou = Hasn't checked yet.
email-cadastre-senha = Add the app password (Edit).
email-backup-restaurado = Backup restored: test the connection before checking again.
email-buscar-agora = Check now
email-testar = Test
email-conexao-ok = Connection working
email-n-arquivos-pasta = { $n ->
        [one] { $n } file in the folder.
       *[other] { $n } files in the folder.
    }
email-teste-le = Reads { $pastas }; to delete, { $como }.
email-nenhuma-pasta = no folder
email-apagar-conta = Delete the account?
email-apagar-conta-descricao = This account's queue and audit log are removed from Treasury. Nothing is deleted at the provider or in the folder, and what was already imported stays.

## Rules

email-regras = Rules
email-nova-regra = New rule
email-regras-tabela = Email rules
email-todas-contas = All accounts
email-col-ordem = Order
email-col-regra = Rule
email-col-conta = Account
email-col-acao = Action
email-col-ia = AI
email-col-acoes = Actions
email-regra-de = From: { $remetentes }
email-qualquer-remetente = Any sender
email-regra-assunto = subject: { $termos }
email-sem-regras = With no rules, the IMAP mailbox ignores everything (the local folder sends it to review). Create one rule per bank or store: sender, subject and what to do.

## Audit log (card)

email-auditoria = Audit log
email-auditoria-tabela = Email audit log
email-col-quando = When
email-col-o-que = What
email-col-detalhe = Details
email-nada-registrado = Nothing logged yet.

## Email import account (dialog)

email-conta-nome-padrao = Family mailbox
email-conta-salva = Account saved
email-conta-titulo = Account: { $nome }
email-conta-nova = New email import account
email-conta-descricao = Recommended: a mailbox just for Treasury, fed by automatic forwarding of your bank's emails. It only runs with the app open and the vault unlocked; nothing is sent (no SMTP).
email-salvar-testar = Save and test
email-tipo-conta = Account type
email-tipo-imap = Email mailbox (IMAP)
email-tipo-pasta = Folder on this computer
email-nome = Name
email-pasta-vigiada = Watched folder
email-pasta-vigiada-dica = Save attachments, statements and .eml messages there (each subfolder is a folder for the filters). What was already handled can go to Processados.
email-pasta-exemplo = /Users/you/Documents/Treasury email
email-escolher = Choose…
email-endereco = Mailbox address
email-endereco-dica = Only the domain is looked up (never the address).
email-endereco-exemplo = family.treasury@gmail.com
email-procurar-config = Look up the settings
email-provedor = Provider
email-provedor-manual = Other (manual)
email-servidor = IMAP server
email-servidor-exemplo = imap.example.com
email-porta = Port
email-onde-criar = Where to create it
email-seguranca = Security
email-seguranca-dica = Except for Proton Bridge on this same computer, TLS only (port 993).
email-starttls = STARTTLS (local Bridge only)
email-usuario = Username
email-senha-app = App password
email-senha-cadastrada = Saved ({ $dica }); leave empty to keep it.
email-senha-cifrada = It's encrypted in the vault and never shown again.
email-senha-aviso = Treasury will read the folders you choose using this app password. The password stays in the vault.
email-certificado = Bridge certificate (cert.pem)
email-certificado-fixado = Certificate pinned: only this one is accepted.
email-certificado-onde = Exported in Bridge › Settings › Advanced settings › Export TLS certificates.
email-que-caixa = What mailbox is this
email-caixa-dedicada = Just for Treasury (recommended)
email-caixa-principal = Someone's main mailbox
email-caixa-principal-aviso = In a main mailbox no folder is read until you choose, and only what the rules catch comes in.
email-gmail-lixeira-aviso = In Gmail, moving to the trash removes the message from all labels.

email-pastas-remetentes = Folders and senders
email-pastas-lidas = Folders read
email-pastas-lidas-dica-pasta = INBOX is the root of the folder; subfolders by path (Banks/*).
email-pastas-lidas-dica-imap = Names or wildcard (Banks/*). A dedicated mailbox reads only the INBOX.
email-pastas-bloqueadas = Blocked folders
email-pastas-bloqueadas-dica = By the server's flags (\Junk, \Trash…), never by name. A block beats an allow.
email-remetentes-permitidos = Allowed senders (all rules)
email-remetentes-permitidos-dica = Empty: anyone. Address, domain (banco.com.br), subdomains (*.itau.com.br), wildcard, /regex/ or membros (the family addresses).
email-remetentes-bloqueados = Blocked senders
email-enderecos-familia = Family addresses
email-enderecos-familia-dica = Who forwards to the mailbox (the membros pattern in rules). A message forwarded by one of them only applies automatically if the original sender is in the rule.
email-sem-regra-case = When no rule matches
email-padrao-ignorar = Ignore (default; only counted)
email-padrao-revisao = Send to review

email-buscas-retencao = Checks and retention
email-intervalo = Check every (minutes)
email-intervalo-dica = From 5 to 120.
email-apagar-provedor = Delete from the provider
email-carencia-importada = Grace period after import (days)
email-quantos-dias = After how many days
email-quantos-dias-dica = Also applies to what's still waiting for review or wasn't recognized (the copy stays in the vault).
email-arquivo-destino = What to do with the file
email-mover-processados = Move to Processados
email-mandar-lixeira = Move to the Trash
email-purgar-gmail = Also delete from the Gmail trash
email-purgar-gmail-dica = Off: the message stays in the trash for 30 days before it's gone.
email-retencao-aviso = First, 7 days of test mode: Treasury only logs what it would have deleted. After that, it only deletes with your confirmation. It never deletes without a rule, outside the folders read, or without checking the stored copy; deletion can't be undone.
email-guardar-original = Keep the original message in the vault
email-guardar-original-dica = Recommended: before deleting from the provider, Treasury checks that the copy is complete.
email-conectou = Connected. Reads: { $pastas }.
email-conectou-nenhuma = no folder (check the folders read)
email-para-apagar = To delete, { $como }.

## Email rule (dialog)

email-regra-salva = Rule saved
email-regra-titulo = Rule: { $nome }
email-regra-nova = New email rule
email-regra-descricao = The order is always: folders, the account's blocks and then the rules in order (the first one that matches decides). Subject is compared ignoring case and accents, already without Fwd:/Enc:/RES:.
email-regra-nome-exemplo = Nubank card bills
email-regra-conta = Account
email-regra-ordem = Order
email-regra-ligada = Rule on
email-regra-de-label = From (senders)
email-regra-de-dica = name@banco.com.br, banco.com.br, *.itau.com.br, fatura*@*.com.br, /regex/ or membros. Forwarded: the original sender counts.
email-regra-bloqueados = Senders blocked in this rule
email-assunto-contem = Subject contains
email-assunto-contem-dica = Terms separated by commas.
email-assunto-contem-exemplo = bill, statement
email-assunto-nao-contem = Subject doesn't contain
email-assunto-nao-contem-exemplo = promotion
email-todos-termos = All terms (default: any of them)
email-assunto-regex = Subject by regular expression
email-assunto-regex-dica = Optional; the error says where the problem is.
email-avancadas = Folders and advanced conditions
email-so-pastas = Only in these folders
email-so-pastas-dica = Empty: all folders read.
email-avancadas-combinadas = This rule has combined advanced conditions (AND/OR/NOT) that stay as they are.
email-anexo-nome = Has an attachment named
email-anexo-nome-exemplo = *.pdf, bill*
email-corpo-contem = Body contains
email-ate-kb = Up to (KB)
email-o-que-fazer = What to do
email-acao-regra = Rule action
email-auto-aviso = Automatic only with a deterministic reader (OFX, cupom fiscal, purchase or Pix notice), a sender authenticated by the provider and listed above, and a preview with no warnings. Everything else goes to review. What the AI reads never goes in on its own.
email-ia = AI
email-ia-dica = The AI gets only the item, as third-party data, and only the preview and import tools; the result is always a proposal.
email-retencao = Retention
email-retencao-conta = The account's
email-carencia = Grace period (days)
email-depois-de = After (days)
email-modelo-corpo = Body template (a notice Treasury doesn't read)
email-modelo-expressao = Expression with the groups valor, descricao and data
email-modelo-exemplo = E.g.: { $exemplo }
email-modelo-tipo = Type
email-modelo-saida = Money out
email-modelo-entrada = Money in
email-modelo-conta = Account for the transactions
email-modelo-escolher-revisao = Choose during review
email-testar-exemplo = Test with an example
email-remetente = Sender
email-assunto = Subject
email-assunto-exemplo = Fwd: Your card bill is ready
email-exemplo-passa = Example: passes ({ $motivo })
email-exemplo-nao-passa = Example: doesn't pass ({ $motivo })
email-teste-recentes = Of the { $total } recent messages, this rule would catch { $pegaria }.
email-testar-nada = Testing doesn't import or delete anything.

email-pasta-vigiar = Folder for Treasury to watch
