# Core texts that don't belong to one area (docs/i18n-design.md, "Textos do núcleo"): generic
# errors of the commands and of the vault (`api`), database (`storage`), typed dates and amounts
# (`time`, `money`), PDF reading (`pdf`) and layouts of imported files (`layouts`). Errors in
# lowercase, like a log line (the interface capitalizes them).
# Revision: AI draft, not yet reviewed by a native speaker.

## Dates, amounts, PDF and database

nucleo-base-erro-data = invalid date: { $valor }
nucleo-base-erro-data-improvavel = date outside the accepted range: { $valor } (check the year)
nucleo-base-erro-mes = invalid month (use YYYY-MM): { $valor }
nucleo-base-erro-moeda = invalid currency: { $valor }
nucleo-base-erro-valor = invalid amount: { $valor }
nucleo-base-erro-casas-decimais = amount with too many decimal places: { $valor }
nucleo-base-erro-pdf = couldn't read the PDF
nucleo-base-erro-esquema-novo = this vault was updated by a newer version of Treasury (schema { $esquema }; this app knows up to { $conhecido }). Update the app, or restore a backup made by this version.
nucleo-base-erro-backup-esquema-novo = this backup was made by a newer version of Treasury (schema { $esquema }; this app knows up to { $conhecido }). Update the app to restore it; the current vault was not changed.
nucleo-base-erro-esquema-antigo = this vault was written by a Treasury version older than 0.9.38. Open it once in version 0.9.38, which updates it, and then in this one.
nucleo-base-erro-esquema-diferente = this vault's schema does not match version 1.0 ({ $objeto } different or missing). Nothing was changed; restore a backup or open it in version 0.9.38.
nucleo-base-erro-banco-leitura = database opened read-only

## Commands and vault (`api`)

nucleo-base-erro-cliente-leitura = read-only client
nucleo-base-erro-nao-pela-ia = { $comando } can't be done by the AI
nucleo-base-erro-comando-leitura = { $comando } isn't allowed for read-only clients
nucleo-base-erro-bloqueie-antes = lock the vault before switching
# Answer to the agent and to MCP clients when a write becomes a proposal.
nucleo-base-proposta-criada = Proposal created. The user needs to approve it in Treasury's approval queue before the change takes effect.
nucleo-base-erro-argumentos = invalid arguments: { $erro }
nucleo-base-erro-informe-id = enter the record's `id`
nucleo-base-erro-senha-curta = the master password needs at least 10 characters
nucleo-base-erro-nome-familia = enter the family name
nucleo-base-cofre = vault { $id }
nucleo-base-erro-cofre-so-interface = only the app's interface can manage the vault
nucleo-base-erro-demo-com-senha = this vault isn't a password-free demo: use the password
nucleo-base-chave-keychain = key in the keychain
nucleo-base-erro-pais = country not on the list: { $pais }
# Author and reason of the first change in a new vault's history.
nucleo-base-autor-criacao = vault creation
nucleo-base-motivo-criacao = Vault creation
nucleo-base-erro-backup-migracao = the backup before the database update failed verification ({ $erro }); nothing was changed
nucleo-base-erro-cofre-aberto = the vault is already open
nucleo-base-erro-backups-interface = backups are managed from the app's interface
nucleo-base-erro-ajustes-interface = settings only from the app's interface
nucleo-base-erro-ajuste-desconhecido = unknown setting: { $ajuste }
nucleo-base-erro-bloqueio-automatico = auto-lock must be between 0 (off) and 1440 minutes

## Layouts of imported files (`layouts`, docs/layouts.md)

# What the file is about; used in the sentences below ("the file looks like a bank statement").
nucleo-base-layout-extrato-conta = bank statement
nucleo-base-layout-fatura-cartao = credit card bill
nucleo-base-layout-extrato-beneficio = meal and food voucher statement
nucleo-base-layout-nota-corretagem = brokerage note
nucleo-base-layout-extrato-previdencia = pension statement
nucleo-base-layout-holerite = payslip
# Issuer, version and description of each known layout.
nucleo-base-layout-quanta = Quanta Previdência (and funds with the same statement)
nucleo-base-layout-quanta-colunas = v1 (2023), text in columns
nucleo-base-layout-quanta-colunas-descricao = Participant financial statement: accrual month, due date and payment date followed by the description and nine amounts per line
nucleo-base-layout-quanta-colado = v1 (2023), run-together text
nucleo-base-layout-quanta-colado-descricao = The same statement as the app's PDF reader delivers it: amounts run together and in reverse order, dates after the description
nucleo-base-layout-caixa = Caixa Econômica Federal
nucleo-base-layout-caixa-periodo = Statement by period v1 (image PDF, read on this computer)
nucleo-base-layout-caixa-periodo-descricao = Internet Banking account statement: date and time, document, description, payee, amount and balance with C/D, daily balance
nucleo-base-layout-sinacor = Brokers using the SINACOR standard (XP, Clear, Rico, BTG, Inter…)
nucleo-base-layout-sinacor-descricao = Trade confirmation for B3 stocks and FIIs
nucleo-base-layout-generico-empresa = Generic (any company)
nucleo-base-layout-holerite-descricao = Payslip: reference month, earnings, deductions (INSS, IRRF), net pay and FGTS
nucleo-base-layout-generico-banco = Generic (any bank)
nucleo-base-layout-fatura-descricao = Credit card bill in PDF: date, description and amount per line, due date and bill total
nucleo-base-layout-ofx = Banks (OFX 1.x SGML and 2.x XML)
nucleo-base-layout-ofx-descricao = Statement or bill in OFX
# Note for the agent about an attached PDF: the tools that read the file (MCP names).
nucleo-base-layout-ferramentas = { $leitura } and { $importacao }
nucleo-base-layout-ferramentas-fatura = preview_card_statement_pdf and import_statement_file (on the card's account)
nucleo-base-layout-reconhecido = [File recognized: { $assunto } ({ $emissor } { $versao }, layout { $layout }). To import it, use { $ferramentas } with this attachment.]
nucleo-base-layout-atencao = [Warning: { $problema }.]
nucleo-base-layout-desconhecido = the file looks like a { $assunto }, but doesn't match any known layout ({ $conhecidos }). It may be a new version: nothing was imported. Send the file so the layout can be added
nucleo-base-layout-outro-assunto = this file is a { $assunto } ({ $descricao }), not a { $esperado }
nucleo-base-layout-nao-reconhecido = couldn't recognize this file as a { $assunto }; known layouts: { $conhecidos }. Send the original file (attached in the conversation, in `attachments`) or the full text of the document, without summarizing
nucleo-base-erro-cofre-danificado = This password opens your backups but not the vault. The vault may be damaged (or the password changed after those backups). Recover from a backup.
nucleo-base-notificacoes-so-interface = notifications only through the app's interface
