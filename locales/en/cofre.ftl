# Vault: create, unlock, recover and switch vaults; the family's country (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Vault screens frame (VaultLayout)

cofre-ficha-cifra = Cipher
cofre-ficha-chave = Key
cofre-ficha-chave-valor = Argon2id over the master password
cofre-ficha-arquivo = File
cofre-ficha-rede = Network
cofre-ficha-rede-valor = No connection needed
cofre-lema = Measured, never made up.

## Vault gate (VaultGate)

cofre-lendo = Reading the vault
cofre-nucleo-indisponivel = Core unavailable
cofre-nucleo-erro = Couldn't read the vault status.
cofre-nucleo-tentar = Try again
cofre-erro-desconhecido = Unknown error.

## Master password (shared by create and recover)

cofre-senha-mestra = Master password
cofre-confirmar-senha = Confirm password
cofre-nova-senha-mestra = New master password
cofre-confirmar-nova-senha = Confirm new password
cofre-senha-erro-vazia = Create a master password.
cofre-senha-erro-curta = Use at least { $min ->
        [one] { $min } character
       *[other] { $min } characters
    }.
cofre-senha-erro-fraca = This password is too weak to protect the vault. See the suggestions below.
cofre-senha-erro-confirmar = The passwords don't match.

## Password strength meter

cofre-forca-rotulo = Password strength
cofre-forca-sem-senha = No password
cofre-forca-descricao = { $nivel }, about { $bits } bits
cofre-forca-bits = ≈ { $bits } bits

## Create the vault

cofre-criar-kicker = 00 · New vault
cofre-criar-kicker-nomeado = 00 · New vault · { $nome }
cofre-criar-titulo = An encrypted vault for your family's finances
cofre-criar-lead = Everything stays in a single file on this computer, encrypted with a key derived from your master password. Nothing leaves it unless you ask.
cofre-criar-card-titulo = Create vault
cofre-criar-arquivo-unico = single
cofre-criar-nome-familia = Family name
cofre-criar-nome-familia-exemplo = The Smiths
cofre-criar-pais-dica = { $suporte } Base currency: { $moeda }. You can change this later in Settings › Family.
cofre-criar-membros = Family members
cofre-criar-membros-ajuda = Who shows up as the owner of accounts, income and expenses in the reports. You can change this later.
cofre-criar-membro-n = Member { $n }
cofre-criar-membro-exemplo = Emma
cofre-criar-membro-placeholder = Name
cofre-criar-remover-membro = Remove member { $n }
cofre-criar-adicionar-membro = Add member
cofre-criar-aviso-forte = Without the master password and the recovery kit, there is no way to recover your data.
cofre-criar-aviso = Treasury doesn't keep your password and has no server: no one, not even us, can open the vault without it. Save it in a password manager and create the kit in Settings.
cofre-criar-aviso-entendi = I understand that if I forget the password and don't have the recovery kit, I lose my data.
cofre-criar-carregando = Deriving key and encrypting…
cofre-criar-botao = Create encrypted vault
cofre-criar-erro-nome-familia = Enter the family name.
cofre-criar-erro-sem-membros = Enter at least one member.
cofre-criar-erro-membros-repetidos = Some members have the same name.
cofre-criar-erro-aviso = Confirm that you understand the warning above.
cofre-criar-toast-titulo = Vault created and encrypted
cofre-criar-toast-descricao = Keep the master password somewhere safe and create the recovery kit in Settings.

## Unlock the vault

cofre-desbloquear-cofre-padrao = Treasury vault
cofre-desbloquear-kicker = 00 · Vault locked
cofre-desbloquear-kicker-nomeado = 00 · Vault locked · { $nome }
cofre-desbloquear-lead-senha = The vault is encrypted at rest. Enter the master password to open this family's data.
cofre-desbloquear-lead-pin = The PIN only works on this computer. Five wrong tries turn off the PIN and ask for the master password.
cofre-desbloquear-lead-recuperacao = Use the kit, two of the three recovery parts or the security key to open the vault and set a new master password.
cofre-desbloquear = Unlock
cofre-desbloquear-touch-id = Unlock with Touch ID
cofre-desbloquear-senha-mac = Unlock with your Mac password
cofre-desbloquear-derivando = Deriving key…
cofre-desbloquear-erro-vazia = Enter the master password.
cofre-desbloquear-esqueceu = Forgot your password?
cofre-auto-bloqueio = Auto-lock
cofre-auto-bloqueio-minutos = { $minutos } min
cofre-keychain = Keychain
cofre-ligado = on
cofre-desligado = off
cofre-usar-pin = Use PIN
cofre-pin-titulo = PIN
cofre-pin-campo = This computer's PIN
cofre-pin-conferindo = Checking…
cofre-usar-senha-mestra = Use the master password

## Recover the vault

cofre-recuperar = Recover the vault
cofre-recuperar-como = How to recover
cofre-recuperar-metodo-kit = Kit
cofre-recuperar-metodo-partes = Parts
cofre-recuperar-metodo-chave = Security key
cofre-recuperar-codigo = Recovery code
cofre-recuperar-codigo-mock = Mock: use { $codigo }
cofre-recuperar-codigo-dica = The code you printed or exported when you created the kit.
cofre-recuperar-primeira-parte = First part
cofre-recuperar-segunda-parte = Second part
cofre-recuperar-partes-dica = Any two of the three parts (TRSP-…).
cofre-recuperar-chave = Security key
cofre-recuperar-pin-chave = Security key PIN (if it has one)
cofre-recuperar-pin-chave-dica = Touch the key when it blinks.
cofre-recuperar-toque-chave = Touch the security key…
cofre-recuperar-recifrando = Re-encrypting the vault…
cofre-recuperar-botao = Reset password and open
cofre-recuperar-voltar = Back to the password
cofre-recuperar-erro-codigo = Enter the recovery kit code.
cofre-recuperar-erro-partes = Enter two of the three parts.
cofre-recuperar-erro-sem-chave = No security key registered.
cofre-recuperar-toast-titulo = Master password reset
cofre-recuperar-toast-descricao = The other recovery methods still work. Keep the new password safe.

## Vault list (VaultSwitcher) and sidebar menu (VaultMenu)

cofre-menu-titulo = Vaults
cofre-menu-descricao = Each vault is a file with its own password, quick unlock and backups. Switching closes the current vault.
cofre-menu-cofre-atual = Vault: { $nome }
cofre-menu-trocar = Switch or create vault
cofre-lista-rotulo = Vaults
cofre-novo-cofre = New vault
cofre-abrir-existente = Open existing file
cofre-demonstracao = Demo vault
cofre-outros = Other vaults
cofre-outros-n = Other vaults ({ $n })
cofre-em-uso = in use
cofre-ainda-nao-criado = not created yet
cofre-acoes-de = Actions for { $nome }
cofre-mais-acoes = More actions (or right-click)
cofre-acao-abrir = Open
cofre-acao-renomear = Rename and purpose
cofre-acao-copiar = Make a copy
cofre-acao-copiar-indisponivel = The vault hasn't been created yet.
cofre-acao-copiar-caminho = Copy file path
cofre-acao-remover = Remove…
cofre-acao-remover-indisponivel = This is the vault in use: switch to another vault before removing it.
cofre-copia-toast-titulo = Copy created
cofre-copia-toast-descricao = { $nome } (copy), with the same password.
cofre-caminho-copiado = Path copied
cofre-nome = Name
cofre-finalidade = Purpose
cofre-remover-titulo = Remove { $nome }?
cofre-remover-descricao = Its backups stay where they are.
cofre-remover-confirmar-lixeira = Remove and move to Trash
cofre-remover-confirmar-lista = Remove from list
cofre-remover-opcoes = What to do with the file
cofre-remover-so-lista = Only remove from the list
cofre-remover-so-lista-dica = The file stays at { $caminho }.
cofre-remover-lixeira = Remove from the list and move the file to the Trash
cofre-remover-lixeira-dica = You can restore it from the Trash until the Trash is emptied.
cofre-remover-toast-titulo = { $nome } removed
cofre-remover-toast-lixeira = The file was moved to the Trash.
cofre-remover-toast-lista = The file stays where it is.
cofre-novo-nome = Vault name
cofre-novo-nome-exemplo = E.g.: Business, Parents' home
cofre-novo-finalidade = Purpose (optional)
cofre-novo-ajuda = The new vault has its own password, quick unlock and backups. Next, you'll set its password.
cofre-novo-botao = Create vault
cofre-existente-arquivo = Vault file (.db)
cofre-existente-arquivo-dica = The full path, like /Users/ana/vaults/business.db.
cofre-existente-escolher = Choose…
cofre-existente-nome = Name (optional)
cofre-existente-abrir = Open this file

## Family's country (CountryPicker)

cofre-pais-rotulo = Country where the family lives
cofre-pais-legenda = ★ with its own adaptation; other countries use the index catalog.
cofre-pais-suporte-completo = Everything in the app: B3, Tesouro Direto, CVM funds, IR with DARF, INSS and IRRF on the pay slip, FIPE and Banco Central interest rates.
cofre-pais-suporte-adaptado = The country's currency, inflation and interest rates, payroll (deductions and extra payments), tax estimates, and ready-made local categories and rules; Brazil-only features are turned off.
cofre-pais-suporte-generico = The country's currency, inflation, interest and exchange rates from the index catalog; no local tax or payroll rules, and none of the Brazil-only features.

## Password strength (lib/passwordStrength.ts)

cofre-forca-muito-fraca = Very weak
cofre-forca-fraca = Weak
cofre-forca-razoavel = Fair
cofre-forca-boa = Good
cofre-forca-forte = Strong
cofre-sugestao-longa = Use at least 12 characters: long passwords hold up much better.
cofre-sugestao-nomes = Don't use the family's or members' names.
cofre-sugestao-comuns = Avoid common words and passwords, even with swaps like 4 for a.
cofre-sugestao-anos = Avoid years and dates, which are easy to guess.
cofre-sugestao-sequencias = Avoid sequences like abc, 123 or qwerty.
cofre-sugestao-repetir = Avoid repeating the same character.
cofre-sugestao-frase = A phrase of 4 or more unrelated words is strong and easy to remember.

cofre-so-no-app = Only available in the Treasury app.

## Demo without a password

cofre-demo-kicker = 00 · Demo · { $nome }
cofre-demo-lead = Demo vault with made-up data: it opens without a password.
cofre-demo-aviso = Anyone with the app can open this vault. Use it only for testing, never with real data.
cofre-demo-abrir = Open the demo

## Create: real vault or demo

cofre-escolha-lead = To start, choose: the family's vault, with a master password, or a demo to get to know the app.
cofre-escolha-rotulo = Which vault to create
cofre-escolha-real = Real vault
cofre-escolha-real-dica = For the family's finances: encrypted with your master password, only you can open it.
cofre-escolha-demo = Demo
cofre-escolha-demo-dica = Made-up data for a sample family, ready in seconds and without a password.
cofre-escolha-demo-titulo = Demo vault
cofre-escolha-demo-lead = Choose the country and the sample family's profile. The vault is created and opens right away.

## Recover from a backup (docs/backup-recovery.md)

cofre-desbloquear-lead-backup = Choose a backup saved on this computer and restore the vault from it. The vault that won't open is not deleted: it is kept alongside.
cofre-desbloquear-nao-abre = Vault won't open?
cofre-backup-abrir-recuperacao = Recover from a backup
cofre-backup-recuperar-titulo = Recover from a backup
cofre-backup-motivo = The vault didn't open: { $erro }
cofre-backup-recuperar-texto = These backups are in this vault's folder, newest first. Choose one and enter the password.
cofre-backup-procurando = Looking for backups…
cofre-backup-nenhum = No backups found in this vault's folder. If you have a copy somewhere else (USB drive, external disk, cloud), use "Bring a backup from elsewhere".
cofre-backup-lista = Backups found
cofre-backup-tipo-desconhecido = Backup
cofre-backup-importado = { $tipo } (brought from elsewhere)
cofre-backup-salvar-copia = Save a copy
cofre-backup-salvar-copia-de = Save a copy of the backup from { $data }
cofre-backup-copia-titulo = Save a copy of the backup
cofre-backup-copia-salva = Backup copy saved
cofre-backup-copia-erro = Couldn't save the copy
cofre-backup-copia-so-app = Saving a copy only works in the installed app.
cofre-backup-trazer = Bring a backup from elsewhere
cofre-backup-trazer-titulo = Choose the backup file (.db.enc)
cofre-backup-trazido = Backup brought into the vault's folder
cofre-backup-trazer-erro = Couldn't bring that file
cofre-backup-senha-mestra = Master password
cofre-backup-senha-mestra-dica = It applies to the restored vault. Usually it's the password you already use.
cofre-backup-outra-senha = The backup was made with another password (the old one or the separate backup password)
cofre-backup-senha-backup = Backup password
cofre-backup-ao-lado-aviso = The current vault is set aside, with "damaged" and the date in its name, and is not deleted.
cofre-backup-restaurar = Restore this backup
cofre-backup-restaurando = Restoring…
cofre-backup-restaurado = Vault restored from the backup
cofre-backup-restaurado-ao-lado = The vault that wouldn't open was kept at { $caminho }.
