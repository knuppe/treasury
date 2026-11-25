# Settings (screen 12) and its sections (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.
# Text between ** is shown in bold; the arguments link, arquivo, icone and url are elements
# (a link, code) that the screen puts in place.

## Screen: search, overview and warnings

ajustes-descricao = Everything you can configure, in five groups. Search for what you need or pick a group.
ajustes-secoes = Settings sections
ajustes-buscar = Search settings
ajustes-buscar-placeholder = Search settings…
ajustes-visao-geral = Overview
ajustes-visao-geral-rotulo = Settings overview
ajustes-busca-vazia = Nothing for “{ $busca }”. Try another word (e.g. password, theme, backup).
ajustes-comando = Settings: { $secao }
ajustes-cartao = { $secao }: { $dica }
ajustes-aviso-backup = More than 48 h without a valid backup

## Groups

ajustes-grupo-voce = You and your family
ajustes-grupo-voce-dica = How the app talks to you, who is in the family and what to notify
ajustes-grupo-dinheiro = Organizing your money
ajustes-grupo-dinheiro-dica = Categories and the financial health score
ajustes-grupo-seguranca = Security and data
ajustes-grupo-seguranca-dica = Password, vaults, backups, history, export and the database
ajustes-grupo-integracoes = Integrations
ajustes-grupo-integracoes-dica = Market data sources, AI engines and MCP
ajustes-grupo-treasury = Treasury
ajustes-grupo-treasury-dica = About the app and how to support the person who makes it

## Sections (search words are comma-separated)

ajustes-secao-perfil = Profile
ajustes-secao-perfil-dica = Experience level, language style and term explanations
ajustes-secao-perfil-busca = profile,level,beginner,intermediate,advanced,language,help
ajustes-secao-familia = Family
ajustes-secao-familia-dica = Name, country, members and institutions
ajustes-secao-familia-busca = owner,member,bank,institution,person,country,currency,where you live
ajustes-secao-preferencias = Appearance and use
ajustes-secao-preferencias-dica = Language, theme, accrual × cash, privacy on open
ajustes-secao-preferencias-busca = language,idioma,theme,color,dark,light,cash,accrual,privacy
ajustes-secao-notificacoes = Notifications
ajustes-secao-notificacoes-dica = Due date reminders and system notifications
ajustes-secao-notificacoes-busca = notification,reminder,alert,due date,warning
ajustes-secao-categorias = Categories
ajustes-secao-categorias-dica = Category tree, spending groups, merging and suggestions
ajustes-secao-categorias-busca = category,subcategory,essential,group,debt,future
ajustes-secao-saude = Financial health
ajustes-secao-saude-dica = Score model: I-SFB or classic indicators
ajustes-secao-saude-busca = health,score,isf,i-sfb,indicators,febraban
ajustes-secao-seguranca = Security
ajustes-secao-seguranca-dica = Master password, PIN, recovery kit and auto-lock
ajustes-secao-seguranca-busca = password,keychain,recovery,pin,lock,touch id
ajustes-secao-cofres = Vaults
ajustes-secao-cofres-dica = Several databases: create, open and switch
ajustes-secao-cofres-busca = vault,database,file,.db,switch,business,demo
ajustes-secao-backups = Backups
ajustes-secao-backups-dica = Backup health, restore and rotation
ajustes-secao-backups-busca = backup,restore,copy
ajustes-secao-historico = Change history
ajustes-secao-historico-dica = What changed, with the difference, undo and redo
ajustes-secao-historico-busca = undo,audit,changes,diff
ajustes-secao-dados = Export data
ajustes-secao-dados-dica = Everything as JSON or CSV, to keep or take to another app
ajustes-secao-dados-busca = export,json,csv,spreadsheet
ajustes-secao-banco = Database
ajustes-secao-banco-dica = Size, tables, maintenance and purge
ajustes-secao-banco-busca = vacuum,integrity,tables,space,purge,cleanup
ajustes-secao-fontes = Data sources
ajustes-secao-fontes-dica = Indices, exchange rates and quotes (BCB, IBGE, Yahoo, FRED…)
ajustes-secao-fontes-busca = fred,ipca,ptax,quotes,indices,exchange rate
ajustes-secao-ia = AI and agent
ajustes-secao-ia-dica = Engines, what the agent can reach, costs and history
ajustes-secao-ia-busca = ai,agent,engines,ollama,claude,openai,cost,memory,mcp,cli,access,permissions
ajustes-secao-mcp = MCP integrations
ajustes-secao-mcp-dica = Use Treasury in other AI assistants and coding agents
ajustes-secao-mcp-busca = mcp,claude,ai,integration
ajustes-secao-atualizacoes = Updates
ajustes-secao-atualizacoes-dica = How Treasury gets new versions: download on its own, notify only or off
ajustes-secao-atualizacoes-busca = update,updates,version,new version,install,download
ajustes-secao-sobre = About and support
ajustes-secao-sobre-dica = Support Treasury: GitHub Sponsors or helping without money
ajustes-secao-sobre-busca = support,donation,donate,sponsors,about,version

## Words repeated across sections

ajustes-remover = Remove
ajustes-adicionar = Add
ajustes-copiar = Copy
ajustes-imprimir = Print
ajustes-arquivar = Archive
ajustes-reativar = Reactivate
ajustes-nunca = never
ajustes-nome = Name
ajustes-arquivo = File
ajustes-cofre = Vault
ajustes-linhas = Rows
ajustes-tamanho = Size
ajustes-quando = When
ajustes-situacao = Status
ajustes-senha-mestra = Master password
ajustes-nao-alterou = Couldn't make the change
ajustes-nao-copiou = Couldn't copy
ajustes-copiar-a-mao = Select the text and copy it by hand.
ajustes-copiado = Copied ✓

## Appearance and use

ajustes-pref-exibicao = Display
ajustes-pref-tema = Theme
ajustes-pref-tema-dica = Also in the theme icon on the top bar and in the command bar (⌘K / Ctrl+K): type “theme”.
ajustes-pref-privacidade = Privacy mode
ajustes-pref-privacidade-dica = Replaces every amount with •••••. Shortcut: ⌘⇧H / Ctrl+Shift+H.
ajustes-pref-privacidade-abrir = Open the vault with privacy mode on
ajustes-pref-privacidade-abrir-dica = When you unlock, amounts start hidden; show them with the eye icon on the top bar.
ajustes-pref-orcamento = Budget and reports
ajustes-pref-base = Default date basis (budget and reports)
ajustes-pref-base-dica = Accrual uses the purchase date; cash uses the card bill's due date for card purchases. Transactions and reports can switch on the spot.
ajustes-pref-base-caixa = Default: cash
ajustes-pref-base-competencia = Default: accrual
ajustes-pref-base-fluxo = Flow diagram (Sankey)
ajustes-pref-base-fluxo-dica = Cash (recommended): the card bill counts in the month it's paid, split into the purchases' categories, alongside the month's redemptions and contributions. Accrual: purchases count in the month they were made.
ajustes-pref-base-fluxo-caixa = Flow diagram: cash
ajustes-pref-base-fluxo-competencia = Flow diagram: accrual
ajustes-pref-competencia = Accrual
ajustes-pref-caixa = Cash
ajustes-pref-bandeja = Keep routines running in the tray when the window closes
ajustes-pref-bandeja-dica = Backups and data collection keep going with the window closed.
ajustes-pref-formatos = Formats
ajustes-pref-downloads = Downloads folder
ajustes-pref-downloads-ligado = Watching the Downloads folder
ajustes-pref-downloads-ligado-dica = The next statements and notes you download show up in a notice to import.
ajustes-pref-downloads-desligado = Downloads folder not watched
ajustes-pref-downloads-acompanhar = Watch the Downloads folder
ajustes-pref-downloads-acompanhar-dica = When an OFX/CSV statement, a PDF brokerage note, a B3 Área do Investidor report or an image (proof of payment, receipt, invoice, screenshot) lands in the Downloads folder, Treasury shows a notice to import it. Images go to the AI agent, which reads them and proposes the entry. Only new files, after the download finishes, and only with the vault open.
ajustes-pref-downloads-apagar = Delete from Downloads after importing
ajustes-pref-downloads-apagar-dica = When a file offered by the notice is imported successfully, it goes to the Trash (you can get it back from there). Images sent to the agent aren't deleted, because the entry depends on your approval.
ajustes-pref-downloads-privacidade = To recognize the format, Treasury reads the new file on your computer (up to 25 MB) without saving anything; the content only goes into an import if you click Import. Nothing leaves your computer. The first time, macOS may ask for permission for Treasury to access the Downloads folder.
ajustes-pref-caixa-entrada = Watch the file inbox
ajustes-pref-caixa-entrada-dica = Files put in { $pasta } (by you or by an AI agent) go to the inbox in Transactions: Treasury reads them, keeps them in the vault and removes them from the folder. Nothing leaves the computer.
ajustes-pref-dev = Developer
ajustes-pref-dev-ligado = Developer options on
ajustes-pref-dev-ligado-dica = Screen 15 · Developer shows up in the menu.
ajustes-pref-dev-desligado = Developer options off
ajustes-pref-dev-opcoes = Developer options
ajustes-pref-dev-opcoes-dica = Agent feedback: the agent gets tools to record and reinforce technical feedback about Treasury (missing tools, confusing data, bugs, documentation gaps), which shows up on screen 15 · Developer.
ajustes-pref-dev-privacidade = The agent's feedback stays only in the vault, outside the change history; nothing is sent out.

## Data sources

ajustes-sync-indice = Index
ajustes-sync-cambio = Exchange rate
ajustes-sync-cotacao = Quote
ajustes-sync-resultado = Update result
ajustes-sync-ok = All up to date ✓
ajustes-sync-parcial = { $ok } of { $total } series updated; { $falhas } with problems.
ajustes-sync-ipca =
    { $n ->
        [one] IPCA estimated for { $n } month (projection).
       *[other] IPCA estimated for { $n } months (projection).
    }
ajustes-sync-novos = { $n } new
ajustes-sync-sem-fonte = No quote source ({ $n })
ajustes-brapi-titulo = brapi.dev (optional)
ajustes-brapi-texto = B3 data source with a free plan: quote backup when Yahoo fails, warnings about renamed tickers and, on paid plans, a cross-check of indicators and dividends against the official sources (CVM and B3). Only the ticker and the token are sent, never what you hold. Create a token at { $link }; it stays encrypted in the vault and never comes back to the screen.
ajustes-brapi-plano = Plan
ajustes-brapi-plano-gratuito = Free
ajustes-brapi-plano-desconhecido = not read yet (use "Test token")
ajustes-brapi-uso = Usage this month
ajustes-brapi-uso-valor = { $usados } of { $limite } requests (plan quota: { $cota })
ajustes-brapi-gratuito-texto = On the free plan, brapi backs up the quotes of B3 tickers when Yahoo fails (and serves the assets that choose it as their source), at most once per ticker per day, and warns about renamed tickers. The indicator and dividend cross-check is for paid plans and stays off, without spending quota.
ajustes-brapi-pago-texto = Paid plan: besides the quote backup and renamed tickers, brapi cross-checks the score's indicators and dividends against the official sources, once a day per ticker.
ajustes-brapi-limite = App limit
ajustes-brapi-limite-opcao = { $pct }% of the quota
ajustes-brapi-testar = Test token
ajustes-brapi-testado = Token accepted: { $plano } plan
ajustes-brapi-token = Token
ajustes-brapi-cadastrado = saved
ajustes-brapi-nao-cadastrado = not saved (the cross-check uses only Fundamentus)
ajustes-brapi-trocar = Change the token
ajustes-brapi-campo = brapi.dev token
ajustes-brapi-salvo = brapi.dev token saved
ajustes-brapi-removido = brapi.dev token removed
ajustes-brapi-salvar = Save token
ajustes-tiingo-titulo = Tiingo (optional, US quotes)
ajustes-tiingo-texto = Daily quotes, dividends and splits of US stocks, ETFs and REITs, with your own free key, instead of Yahoo. Create an account at { $link } and paste the API token here. The free plan is for personal use: the data stays in your vault and isn't passed on.
ajustes-eodhd-titulo = EODHD (optional, world exchanges)
ajustes-eodhd-texto = Daily quotes, dividends and splits from exchanges around the world, with your own key, instead of Yahoo. EODHD's free plan is for testing only (a few requests a day); following a portfolio takes the paid personal plan. Get the key at { $link }. EODHD only accepts the key in the request address, always over an encrypted connection.
ajustes-chave-fonte = Key
ajustes-chave-fonte-cadastrada = saved
ajustes-chave-fonte-nao-cadastrada = not saved
ajustes-chave-fonte-trocar = Change the key
ajustes-chave-fonte-campo = { $fonte } key
ajustes-chave-fonte-salva = { $fonte } key saved
ajustes-chave-fonte-removida = { $fonte } key removed
ajustes-chave-fonte-salvar = Save key
ajustes-fontes-indices = Indices, exchange rates and quotes
ajustes-fontes-indices-texto = Treasury fetches IPCA, Selic, CDI and PTAX from Brazil's Central Bank, quotes from Yahoo Finance and Tesouro Transparente and, depending on the reference country, inflation, interest rates and exchange rates of other countries (ECB, Eurostat, OECD, Bank of Canada, ONS, Bank of England and the World Bank). The data stays in the vault; nothing about your portfolio leaves your computer.
ajustes-fontes-ultima = Last update
ajustes-fontes-atualizando = Updating…
ajustes-fontes-atualizar = Update indices, exchange rates and quotes
ajustes-fontes-analise = Asset analysis (CVM, B3 and SEC)
ajustes-fred-titulo = FRED (US data)
ajustes-fred-texto = CPI, M2 and Fed Funds come from FRED (Federal Reserve Bank of St. Louis), which requires a free key. Create an account at { $link } (menu “My Account › API Keys”) and paste the 32-character key here. It's stored encrypted in the vault and never shown on screen again.
ajustes-fred-chave = Key
ajustes-fred-cadastrada = saved
ajustes-fred-nao-cadastrada = not saved
ajustes-fred-trocar = Change the key
ajustes-fred-campo = FRED key
ajustes-fred-placeholder = 32 letters and numbers
ajustes-fred-salva = FRED key saved
ajustes-fred-removida = FRED key removed
ajustes-fred-salvar = Save key
ajustes-fred-tamanho = The key has exactly 32 letters and numbers ({ $n } now).

## Data sources: reference country and providers

ajustes-pais-br = Brazil
ajustes-pais-us = United States
ajustes-pais-us-curto = US
ajustes-pais-eu = European Union
ajustes-pais-eu-curto = Europe
ajustes-pais-zona-euro = Euro area
ajustes-pais-gb = United Kingdom
ajustes-pais-ca = Canada
ajustes-pais-int = International
ajustes-pais-int-curto = Global
ajustes-cambio-bcb = PTAX from Brazil's Central Bank
ajustes-cambio-ecb = ECB reference rates (euro against ~30 currencies)
ajustes-pais-ref-alternativa = (alternative)
ajustes-pais-ref-deflator = Deflator
ajustes-pais-ref-deflator-valor = Country inflation ({ $pais }) = { $serie }
ajustes-pais-ref-inflacao = Inflation
ajustes-pais-ref-juros = Interest rates
ajustes-pais-ref-moeda = Currency
ajustes-pais-ref-auto = Automatic (base currency { $moeda } → { $pais })
ajustes-pais-ref-titulo = Reference country
ajustes-pais-ref-texto = The reference country picks the inflation for the “Country inflation (…)” deflator and which indices, interest rates and exchange rates the collection fetches, on top of the usual series. In Brazil nothing changes: IPCA, CDI and PTAX from the Central Bank.
ajustes-pais-ref-pais = Country
ajustes-pais-ref-alterado = Reference country changed
ajustes-pais-ref-alterado-dica = { $pais }. Update the indices to download the new series.
ajustes-pais-ref-nota = { $moeda ->
        [BRL] The app's base currency is still the real: the portfolio and reports convert to BRL using PTAX.
       *[other] The family's base currency is still { $moeda }: the portfolio and reports convert to { $moeda }.
    } The chosen country's exchange rate is available as a series (09 · Assets › Indices and quotes) and for conversions between other currencies.
ajustes-provedor-em-dia = Up to date
ajustes-provedor-erro = With errors
ajustes-provedor-nunca = Not collected yet
ajustes-provedor-fora = Not in use
ajustes-provedor-col = Provider
ajustes-provedor-exige-chave = Requires a key.
ajustes-provedor-col-uso = In use
ajustes-provedor-uso = { $uso } of { $total }
ajustes-provedor-col-coletado = Collected at
ajustes-provedor-col-ate = Data up to
ajustes-provedores-titulo = Data providers
ajustes-provedores-texto = Official, free sources, no key needed (except FRED). Each value keeps its source and collection date; a source that's down doesn't stop the others.
ajustes-provedores-vazio = No providers.

## Data sources: external sources

ajustes-externas-titulo = External sources
ajustes-externas-ligadas = { $ligadas } of { $total } on
ajustes-externas-texto = Each source below is queried over the network only to fetch public data (quotes, indices, indicators); nothing about your portfolio is sent. Turn off the ones you don't want to use: what was already downloaded stays in the vault.
ajustes-externas-buscar = Search sources
ajustes-externas-buscar-placeholder = Search by name, data or country…
ajustes-externas-filtrar = Filter sources by country
ajustes-externas-todas = All
ajustes-externas-mostrando = { $n } of { $total }
ajustes-externas-nenhuma = No sources found.
ajustes-externas-nenhuma-busca = No sources found for “{ $busca }”.
ajustes-externas-nenhuma-pais = No sources found in { $pais }.
ajustes-externas-nenhuma-busca-pais = No sources found for “{ $busca }” in { $pais }.
ajustes-externas-desligada-app = Turned off by the app after { $dias } days of failures in a row.
ajustes-externas-falhando =
    { $dias ->
        [one] Failing for { $dias } day; turns itself off at 3.
       *[other] Failing for { $dias } days; turns itself off at 3.
    }
ajustes-externas-ultimo-erro = Last error: { $erro }
ajustes-externas-ligada = { $fonte } on
ajustes-externas-desligada = { $fonte } off

## Export data

ajustes-exportar-feito = Exported without encryption
ajustes-exportar-aviso = The exported file comes out **without encryption**: anyone with access to it can see your data. Keep it somewhere safe and delete it when you no longer need it. Treasury's own folders aren't accepted as a destination.
ajustes-exportar-json = Everything, as JSON (open format)
ajustes-exportar-json-dica = Full path of the file (an existing folder, outside Treasury's folders).
ajustes-exportar-json-botao = Export JSON
ajustes-exportar-csv = Transactions, as CSV (spreadsheet)
ajustes-exportar-csv-dica = All transactions, one row per transaction.
ajustes-exportar-csv-botao = Export CSV
ajustes-exportar-csv-nome = transactions-{ $data }.csv
ajustes-exportar-versao = Version
ajustes-exportar-versao-valor = { $versao } · schema { $esquema }

## MCP integrations

ajustes-mcp-col-cliente = Client
ajustes-mcp-col-ferramenta = Tool
ajustes-mcp-col-resultado = Result
ajustes-mcp-ok = ok
ajustes-mcp-erro = error
ajustes-mcp-col-tempo = Time
ajustes-mcp-usar = Use Treasury in other AI apps (MCP)
ajustes-mcp-desktop-texto = In Settings › Developer › Edit Config, add this snippet to { $arquivo } and restart Claude.
ajustes-mcp-config = MCP configuration
ajustes-mcp-copiar-config = Copy configuration
ajustes-mcp-terminal = In the terminal:
ajustes-mcp-comando = Claude Code command
ajustes-mcp-copiar-comando = Copy command
ajustes-mcp-code-texto = With the app open, MCP talks to it (including writes, which go into the history as “AI”). With the app closed, it reads in read-only mode if keychain unlock is on (Settings › Security).
ajustes-mcp-opencode-texto = Add this snippet to { $arquivo } (the global one lives in ~/.config/opencode; an opencode.json in the project folder works too). Works with OpenCode 1.x and 2.x.
ajustes-mcp-opencode-config = OpenCode MCP configuration
ajustes-mcp-copiar-opencode = Copy OpenCode snippet
ajustes-mcp-ultimas = Latest calls from MCP clients
ajustes-mcp-auditoria = MCP audit
ajustes-mcp-vazio = No MCP client has used Treasury yet.
ajustes-mcp-escopo-leitura = Read
ajustes-mcp-escopo-escrita = Write (proposal)
ajustes-mcp-atualizado = MCP access updated
ajustes-mcp-agentes = External agents
ajustes-mcp-chave-agora-titulo = An API key is now required
ajustes-mcp-chave-agora-texto = To protect the vault from other programs on this computer, external MCP clients (AI assistants and coding agents) now need a key. Create one below and add it to the client; the app's own agent keeps working without it.
ajustes-mcp-chave-agora-entendi = Got it
ajustes-mcp-cliente-novo-titulo = A new MCP client connected
ajustes-mcp-cliente-novo-item = { $cliente } · { $programa } · PID { $pid } · { $quando }
ajustes-mcp-cliente-programa-desconhecido = unknown program
ajustes-mcp-cliente-novo-texto = If you didn't set up this client, revoke its API key below and replace the others.
ajustes-mcp-cliente-novo-toast = A new MCP client connected to Treasury. Check Settings › MCP integrations.
ajustes-mcp-app-fechado = Answer with the app closed
ajustes-mcp-app-fechado-dica = When the app is closed, treasury mcp opens the vault read-only through keychain unlock, and only for clients with a valid API key (TREASURY_API_KEY). When off, it only asks to open Treasury: no other program reads the vault without you.
ajustes-mcp-http = Local MCP HTTP server
ajustes-mcp-http-dica = For agents that speak MCP over HTTP. Only on 127.0.0.1 and always with Authorization: Bearer <key>.
ajustes-mcp-porta = Port
ajustes-mcp-salvar-porta = Save port
ajustes-mcp-nao-subiu = Didn't start: { $erro }
ajustes-mcp-endereco = Address: { $url }
ajustes-mcp-no-ar = running
ajustes-mcp-iniciando = starting…
ajustes-mcp-so-no-app = only in the app
ajustes-mcp-copie-agora = Copy it now: the key won't be shown again. Scopes: { $escopos }.
ajustes-mcp-copiar-chave = Copy key
ajustes-mcp-escopos = Scopes
ajustes-mcp-ultimo-uso = Last used
ajustes-mcp-ate = until { $data }
ajustes-mcp-ativa = active
ajustes-mcp-chave-revogada = Key revoked
ajustes-mcp-revogar = Revoke
ajustes-mcp-chave-sempre = An API key is always required: each external MCP client (AI assistant or coding agent) connects with its own key and only sees what its scopes allow.
ajustes-mcp-revogada-em = revoked { $data }
ajustes-mcp-sai-da-lista = leaves the list { $data }
ajustes-mcp-excluir = Delete
ajustes-mcp-chave-excluida = Key deleted
ajustes-mcp-chaves = API keys
ajustes-mcp-nova-chave = New key
ajustes-mcp-sem-chaves = No keys. Create one for each external agent, with only the scopes it needs.
ajustes-mcp-chave-criada = Key created
ajustes-mcp-nova-chave-titulo = New API key
ajustes-mcp-nova-chave-dica = No scope approves proposals or touches settings, keys, the vault or sessions.
ajustes-mcp-ja-copiei = I've copied it
ajustes-mcp-criar-chave = Create key
ajustes-mcp-nome-placeholder = Brokerage agent
ajustes-mcp-leitura-sempre = Read (always)
ajustes-mcp-escrita-opcao = Write: creates proposals for you to approve
ajustes-mcp-validade = Expiration
ajustes-mcp-sem-vencimento = Never expires
ajustes-mcp-dias = { $n } days
ajustes-mcp-um-ano = 1 year

## Backups

ajustes-backup-restaurado = Backup restored
ajustes-backup-restaurado-dica = Data from { $data }. The previous state became a backup.
ajustes-backup-restaurar-titulo = Restore backup
ajustes-backup-restaurar-texto = Your data goes back to { $data } ({ $tipo }). First, Treasury backs up the current state, and the restored database is re-encrypted with your current master password.
ajustes-backup-restaurando = Restoring…
ajustes-backup-restaurar = Restore
ajustes-backup-senha-atual = Current master password
ajustes-backup-senha = Backup password
ajustes-backup-senha-dica = The password in use when this backup was made (an old master password or a separate backup password).
ajustes-backup-outra-senha = This backup uses another password
ajustes-backup-verificar-titulo = Verify with another password
ajustes-backup-verificar-texto = This backup didn't open with the current key. Enter the password in use when it was made.
ajustes-backup-integro = Backup is intact
ajustes-backup-problema = Backup has a problem
ajustes-backup-verificar = Verify
ajustes-backup-nao-verificou = Couldn't verify
ajustes-backup-hora = Daily backup at
ajustes-backup-diarios = Daily backups kept
ajustes-backup-semanais = Weekly
ajustes-backup-mensais = Monthly
ajustes-backup-ao-fechar = Back up when closing the app
ajustes-backup-ao-fechar-dica = Only when something changed since the last backup.
ajustes-backup-destinos = Extra destinations (always encrypted)
ajustes-backup-destino-novo = New destination folder
ajustes-backup-destino-placeholder = /Volumes/Backup/Treasury or a synced folder
ajustes-backup-chave = Backup key
ajustes-backup-chave-mestra = The master password
ajustes-backup-chave-propria = Separate backup password
ajustes-backup-senha-propria = Backup password (min. 8)
ajustes-backup-senha-propria-dica = Set it before saving. Keep it safe: without it, new backups won't open.
ajustes-backup-senha-definida = Backup password set
ajustes-backup-definir-senha = Set password
ajustes-backup-ajustes-salvos = Backup settings saved
ajustes-backup-salvar-ajustes = Save backup settings
ajustes-backup-descartar = Discard
ajustes-backup-col-tipo = Type
ajustes-backup-col-onde = Where
ajustes-backup-col-acoes = Actions
ajustes-backup-pasta-app = app folder
ajustes-backup-sumiu = file missing
ajustes-backup-verificado = ✓ verified
ajustes-backup-verificado-propria = ✓ verified · separate password
ajustes-backup-nao-verificado = not verified
ajustes-backup-saude = Backup health
ajustes-backup-fazendo = Backing up
ajustes-backup-feito = Backup done
ajustes-backup-feito-dica = { $tamanho } · encrypted
ajustes-backup-fazer = Back up now
ajustes-backup-atrasado = The last valid backup is { $horas } hours old (more than 48 h). Back up now.
ajustes-backup-nenhum = There's no valid backup. Back up now.
ajustes-backup-ultimo = Last valid backup
ajustes-backup-ha = Age
ajustes-backup-horas = { $horas } h
ajustes-backup-no-disco = Backups on disk
ajustes-backup-espaco = Space
ajustes-backup-pendente = Daily backup pending: it runs at the next check
ajustes-backup-ajustes = Backup settings

## Categories

ajustes-cat-uso-lancamentos =
    { $n ->
        [one] { $n } transaction
       *[other] { $n } transactions
    }
ajustes-cat-uso-divisoes =
    { $n ->
        [one] { $n } split part
       *[other] { $n } split parts
    }
ajustes-cat-uso-orcamento =
    { $n ->
        [one] { $n } budget line
       *[other] { $n } budget lines
    }
ajustes-cat-uso-regras =
    { $n ->
        [one] { $n } rule
       *[other] { $n } rules
    }
ajustes-cat-uso-recorrencias =
    { $n ->
        [one] { $n } recurrence
       *[other] { $n } recurrences
    }
ajustes-cat-uso-parcelamentos =
    { $n ->
        [one] { $n } installment plan
       *[other] { $n } installment plans
    }
ajustes-cat-uso-subcategorias =
    { $n ->
        [one] { $n } subcategory
       *[other] { $n } subcategories
    }
ajustes-cat-em-uso = Used by { $usos }.
ajustes-cat-sem-uso = Not in use: it can be deleted without reassigning.
ajustes-cat-principal = — (top-level category)
ajustes-cat-nova = New category
ajustes-cat-nova-sub-de = New subcategory of { $nome }
ajustes-cat-nova-sub-em = New subcategory in { $nome }
ajustes-cat-renomear = Rename
ajustes-cat-renomear-nome = Rename { $nome }
ajustes-cat-mover = Move
ajustes-cat-mover-nome = Move { $nome }
ajustes-cat-juntar = Merge
ajustes-cat-juntar-nome = Merge { $nome }
ajustes-cat-juntar-titulo = Merge { $nome } into another
ajustes-cat-apagar-nome = Delete { $nome }
ajustes-cat-arquivar-nome = Archive { $nome }
ajustes-cat-reativar-nome = Reactivate { $nome }
ajustes-cat-criada = Category created
ajustes-cat-renomeada = Category renamed
ajustes-cat-movida = Category moved
ajustes-cat-juntadas = Categories merged
ajustes-cat-apagada = Category deleted
ajustes-cat-reatribuidos =
    { $n ->
        [one] { $n } transaction reassigned.
       *[other] { $n } transactions reassigned.
    }
ajustes-cat-juntar-texto = Everything that uses this category (transactions, splits, budget, rules, recurrences) moves to the chosen one, including its subcategories. You can undo it from the history.
ajustes-cat-apagar-texto = If it's in use, choose who gets the transactions. To just hide it from pickers, archive it.
ajustes-cat-dentro-de = Inside
ajustes-cat-juntar-em = Merge into
ajustes-cat-reatribuir = Reassign to (if in use)
ajustes-cat-escolha = Choose the category
ajustes-cat-nao-reatribuir = Don't reassign
ajustes-cat-recolher = Collapse { $nome }
ajustes-cat-expandir = Expand { $nome }
ajustes-cat-arquivada = archived
ajustes-cat-sugeridas-despesa =
    { $n ->
        [one] { $n } suggested category from the default set (bill payments, loans and card; school, health, insurance, kids' savings…). The ones you already have don't change; in “See list”, choose which ones to add.
       *[other] { $n } suggested categories from the default set (bill payments, loans and card; school, health, insurance, kids' savings…). The ones you already have don't change; in “See list”, choose which ones to add.
    }
ajustes-cat-sugeridas-receita =
    { $n ->
        [one] { $n } suggested category from the default set, { $receitas } for income (redemptions, sale of assets, debts received, app-based work, sales and side jobs, retirement and government benefits…). The ones you already have don't change; in “See list”, choose which ones to add.
       *[other] { $n } suggested categories from the default set, { $receitas } for income (redemptions, sale of assets, debts received, app-based work, sales and side jobs, retirement and government benefits…). The ones you already have don't change; in “See list”, choose which ones to add.
    }
ajustes-cat-ver-lista = See list
ajustes-cat-esconder-lista = Hide list
ajustes-cat-adicionadas =
    { $n ->
        [one] { $n } category added
       *[other] { $n } categories added
    }
ajustes-cat-desfazer-dica = Undo it in Settings › Change history if you want.
ajustes-cat-adicionar-sugeridas = Add suggested
ajustes-cat-adicionar-selecionadas = Add { $n } selected
ajustes-cat-sugeridas = Suggested categories
ajustes-cat-receita-item = { $caminho } (income)
ajustes-cat-principais = Top-level
ajustes-cat-subcategorias = Subcategories
ajustes-cat-tipo = Category type
ajustes-cat-o-que = Categories of
ajustes-cat-lancamentos = Transactions
ajustes-cat-produtos = Products (purchases)
ajustes-cat-despesas = Expenses
ajustes-cat-receitas = Income
ajustes-cat-mostrar-arquivadas = Show archived
ajustes-cat-arvore-despesa = Expense categories
ajustes-cat-arvore-receita = Income categories
ajustes-cat-grupos-explicacao = Each expense goes into a group: Essentials, Debts, Future or Non-essential. The budget and the reports show how much of your income goes to each one, and essentials count toward the emergency fund.
ajustes-cat-grupo = Group
ajustes-cat-grupo-de = Group of { $nome }
ajustes-cat-grupo-campo-dica = Where the expense shows up in “Where the income goes”. Essentials count toward the emergency fund.
ajustes-cat-grupo-alterado = { $nome } is now in { $grupo }
ajustes-cat-grupo-alterado-subs =
    { $n ->
        [one] { $nome } and 1 subcategory are now in { $grupo }
       *[other] { $nome } and { $n } subcategories are now in { $grupo }
    }
ajustes-cat-grupo-lote =
    { $n ->
        [0] No category changed group
        [one] 1 category is now in { $grupo }
       *[other] { $n } categories are now in { $grupo }
    }
ajustes-cat-filtro-grupo = Filter by group
ajustes-cat-todos-grupos = All
ajustes-cat-filtro-vazio = No categories in { $grupo }.
ajustes-cat-selecionar = Select { $nome }
ajustes-cat-selecionadas =
    { $n ->
        [one] 1 category selected
       *[other] { $n } categories selected
    }
ajustes-cat-selecao-aria = Selected categories
ajustes-cat-mudar-grupo = Change group
ajustes-cat-mudar-grupo-aria = Change the group of the selected ones
ajustes-cat-limpar-selecao = Clear selection
ajustes-cat-grupos-rodape = Groups
ajustes-cat-grupos-rodape-valor = essentials count toward the emergency fund

## Database

ajustes-banco-integridade = Check integrity
ajustes-banco-integridade-dica = Checks pages, indexes and references between tables.
ajustes-banco-estatisticas = Update statistics
ajustes-banco-estatisticas-dica = ANALYZE: helps the database pick the right indexes.
ajustes-banco-otimizar = Optimize
ajustes-banco-otimizar-dica = PRAGMA optimize: quick, safe tune-ups.
ajustes-banco-compactar = Compact
ajustes-banco-compactar-dica = VACUUM: gives the free space inside the file back to the disk.
ajustes-banco-expurgo-historico = Old change history
ajustes-banco-expurgo-auditoria = Agent and MCP audit
ajustes-banco-expurgo-conversas = Old agent conversations
ajustes-banco-expurgo-caches = Market caches
ajustes-banco-expurgo-analise = Asset analysis data
ajustes-banco-col-tabela = Table
ajustes-banco-col-guarda = Holds
ajustes-banco-col-indices = Indexes
# The "tamanho" argument is empty or " · before → after", when the file size changed.
ajustes-banco-relatorio-ok = { $acao }: all good in { NUMBER($segundos, maximumFractionDigits: 1) } s{ $tamanho }.
ajustes-banco-relatorio-problemas = { $acao }: found problems in { NUMBER($segundos, maximumFractionDigits: 1) } s{ $tamanho }.
ajustes-banco-idade = Minimum age: { $item }
ajustes-banco-mais-de-dias = more than { $n } days
ajustes-banco-previa = Preview
ajustes-banco-nada = Nothing to delete.
ajustes-banco-previa-texto =
    { $n ->
        [one] { $n } record, about { $tamanho }. { $consequencia }
       *[other] { $n } records, about { $tamanho }. { $consequencia }
    }
ajustes-banco-apagar-titulo = Delete: { $item }
ajustes-banco-apagar-texto =
    { $n ->
        [one] { $n } record. { $consequencia } First, Treasury makes a verified backup of the vault.
       *[other] { $n } records. { $consequencia } First, Treasury makes a verified backup of the vault.
    }
ajustes-banco-apagar-confirmar = Back up and delete
ajustes-banco-apagados =
    { $n ->
        [one] { $n } record deleted
       *[other] { $n } records deleted
    }
ajustes-banco-apagados-dica = Backup made first. File: { $antes } → { $depois }.
ajustes-banco-espaco-livre = Free space inside it
ajustes-banco-cifra = Cipher
ajustes-banco-versao-esquema = Schema version
ajustes-banco-conjuntos =
    { $n ->
        [one] { $n } change set
       *[other] { $n } change sets
    }
ajustes-banco-conjuntos-desde =
    { $n ->
        [one] { $n } change set since { $data }
       *[other] { $n } change sets since { $data }
    }
ajustes-banco-ultima-manutencao = Last maintenance
ajustes-banco-manutencao = Maintenance
ajustes-banco-rodar = Run
ajustes-banco-maiores = Largest tables
ajustes-banco-so-maiores = Only the largest
ajustes-banco-todas = All ({ $n })
ajustes-banco-maiores-rotulo = Largest tables in the database
ajustes-banco-clique-duplo = Double-click a table (or press Enter) to see its first 100 rows.
ajustes-banco-expurgo = Purge old data
ajustes-banco-expurgo-texto = Only what isn't family data: transactions, accounts, portfolio, contracts and records never go in here, nor agent proposals awaiting approval. Each purge shows a preview, asks for confirmation and makes a verified backup before deleting.
ajustes-tabela-nulo = null
ajustes-tabela-titulo = Table { $tabela }
ajustes-tabela-lendo = Reading the table
ajustes-tabela-resumo =
    { $linhas ->
        [one] { $linhas } row
       *[other] { $linhas } rows
    } in total · { $colunas ->
        [one] { $colunas } column
       *[other] { $colunas } columns
    } · order: { $ordem } · read-only
ajustes-tabela-primeiras = First rows of { $tabela }
ajustes-tabela-vazia = The table is empty.

## Family: name, country, members and institutions

ajustes-familia-renomear = Rename { $nome }
ajustes-familia-novo-nome = New name for { $nome }
ajustes-familia-nome = Family name
ajustes-familia-nome-dica = Shows in the sidebar and on the unlock screen.
ajustes-familia-a-familia = the family
ajustes-familia-nome-alterado = Family name changed
ajustes-familia-moeda = Base currency: { $moeda }
ajustes-familia-fontes-desligadas =
    { $n ->
        [one] { $n } source turned off
       *[other] { $n } sources turned off
    }
ajustes-familia-fontes-religadas =
    { $n ->
        [one] { $n } source turned back on
       *[other] { $n } sources turned back on
    }
ajustes-familia-moeda-agora = base currency now in { $moeda }
ajustes-familia-fontes-certas = The data sources were already right.
ajustes-pais-titulo = Country
ajustes-pais-onde-mora = Country where the family lives
ajustes-pais-resumo = Base currency: { $moeda } · inflation: { $inflacao } · reference rate: { $juro }
ajustes-pais-mudar-titulo = Change the family's country to { $pais }?
ajustes-pais-mudar-fontes = The data sources for { $pais } start being collected, and the ones that make no sense there are turned off. The ones you turned off by hand stay as they are.
ajustes-pais-mudar-referencia = The reference inflation, the reference interest rate and the tax estimate become those of { $pais }.
ajustes-pais-mudar-mantem = The categories, rules and transactions you already have don't change.
ajustes-pais-trocar-moeda = Change the base currency from { $de } to { $para }
ajustes-pais-trocar-moeda-dica = The vault already has accounts and transactions: they stay in their own currency and totals start being converted to { $moeda }. The cost of investments already recorded stays in the currency of the time.
ajustes-pais-alterado = Family's country: { $pais }
ajustes-pais-mudar = Change to { $pais }
ajustes-membros-titulo = Members
ajustes-membros-esconder = Hide archived
ajustes-membros-mostrar = Show archived
ajustes-membros-lista = Family members
ajustes-membros-nao-renomeou = Couldn't rename
ajustes-membros-nao-arquivou = Couldn't archive
ajustes-membros-principal = main
ajustes-membros-arquivado = archived
ajustes-membros-tornar-principal = Make main
ajustes-membros-adicionado = Member added
ajustes-membros-novo = New member's name
ajustes-membros-novo-placeholder = Member's name
ajustes-membros-familia-tela = Each member's income, pay stubs, accounts and cards are on the { $link } screen.
ajustes-inst-tipo-de = Type of { $nome }
ajustes-inst-apagada = Institution deleted
ajustes-inst-nao-apagou = Couldn't delete
ajustes-inst-titulo = Institutions
ajustes-inst-nenhuma = No institutions saved.
ajustes-inst-criada = Institution created
ajustes-inst-nome = Institution name
ajustes-inst-nome-placeholder = Bank, broker or issuer
ajustes-inst-tipo-nova = Type of the new institution
ajustes-inst-codigo = Code (COMPE)
ajustes-inst-codigo-placeholder = Code
ajustes-inst-criar = Create

## Change history

ajustes-hist-aplicada = Applied
ajustes-hist-desfeita = Undone
ajustes-hist-pendente = Pending
ajustes-hist-rejeitada = Rejected
ajustes-hist-criou = created { $itens }
ajustes-hist-alterou = changed { $itens }
ajustes-hist-apagou = deleted { $itens }
ajustes-hist-situacao-em = { $situacao } on { $data }
ajustes-hist-desfeito = Change undone
ajustes-hist-refeito = Change redone
ajustes-hist-desfazer = Undo
ajustes-hist-refazer = Redo
ajustes-hist-conflito-desfazer = Can't undo it on its own.
ajustes-hist-conflito-refazer = Can't redo it on its own.
ajustes-hist-conflito-dica = First undo, in the list beside it, the more recent changes that touch the same rows.
ajustes-hist-col-alteracao = Change
ajustes-hist-col-autor = Author
ajustes-hist-todos = All
ajustes-hist-alteracoes = Changes
ajustes-hist-vazio = No changes.
ajustes-hist-paginas = History pages
ajustes-hist-recentes = ← Newer
ajustes-hist-pagina = page { $n }
ajustes-hist-antigas = Older →
ajustes-hist-detalhe = Details
ajustes-hist-escolha = Choose a change to see the before and after of each row. Imports, rules and AI actions are here too, and everything can be undone.

## Notifications

ajustes-notif-contas = Bills and card statements coming due
ajustes-notif-contas-dica = Scheduled transactions and card bills close to their due date.
ajustes-notif-saldo = Balance going negative
ajustes-notif-saldo-dica = Based on the balance forecast, an account goes negative in the next 30 days.
ajustes-notif-orcamento = Budget exceeded
ajustes-notif-orcamento-dica = A category this month is above plan.
ajustes-notif-assinaturas = Subscriptions
ajustes-notif-assinaturas-dica = A price increase or a new recurring charge.
ajustes-notif-proventos = Dividend record dates
ajustes-notif-proventos-dica = Portfolio assets with a record date in the next 3 days.
ajustes-notif-watchlist = Watchlist price
ajustes-notif-watchlist-dica = A quote above or below the alert you set.
ajustes-notif-scores = Asset health score
ajustes-notif-scores-dica = A drop in band or a new red flag.
ajustes-notif-contas-paradas = Accounts without a recent statement
ajustes-notif-contas-paradas-dica = Once a week, when an everyday account's balance is possibly incomplete.
ajustes-contas-paradas-titulo = Accounts without a recent statement
ajustes-contas-paradas-dica = Days without a transaction, import or balance check before an account shows as "updated N days ago" and as "possibly incomplete". An account that usually has transactions every month and closed last month with none is also possibly incomplete.
ajustes-contas-paradas-tipo = Account type
ajustes-contas-paradas-col-envelhece = Updated
ajustes-contas-paradas-col-incompleto = Incomplete
ajustes-contas-paradas-envelhece = { $tipo }: days until "updated N days ago"
ajustes-contas-paradas-incompleto = { $tipo }: days until "possibly incomplete"
ajustes-contas-paradas-padrao = Back to default
ajustes-notif-agente = Other agent alerts
ajustes-notif-agente-dica = Month-end close, portfolio outside its band.
ajustes-notif-so-no-app = Notifications only in the installed app
ajustes-notif-teste-titulo = Sample bill due tomorrow
ajustes-notif-teste-valores = { $valor } in Checking account, due tomorrow.
ajustes-notif-teste-sem-valores = Open Treasury to see the details.
ajustes-notif-sistema = System notifications
ajustes-notif-sistema-dica = With the app open and the vault unlocked. When off, reminders keep going to the Agent's alert inbox.
ajustes-notif-valores = Show amounts in notifications
ajustes-notif-valores-dica = Notifications appear on the lock screen and in the system's notification center. When off, only the title (no amounts).
ajustes-notif-lembretes = Reminders
ajustes-notif-antecedencia = Warn about bills how many days ahead
ajustes-notif-antecedencia-dica = From 0 (on the day only) to 15.
ajustes-notif-dias = days
ajustes-notif-teste-falhou = The system didn't show the notification
ajustes-notif-testar = Send a test notification
ajustes-notif-novos =
    { $n ->
        [one] { $n } new reminder
       *[other] { $n } new reminders
    }
ajustes-notif-nenhum = No new reminders
ajustes-notif-caixa = Alerts are in the Agent's inbox.
ajustes-notif-conferir = Check reminders now
ajustes-notif-titulo = Notifications and reminders

## Profile

ajustes-perfil-tom-simples = Very simple
ajustes-perfil-tom-equilibrado = Balanced
ajustes-perfil-tom-tecnico = Technical
ajustes-perfil-dicas-nivel = By level
ajustes-perfil-dicas-sempre = Always
ajustes-perfil-dicas-nunca = Never
ajustes-perfil-origem-questionario = from your answers to the questionnaire
ajustes-perfil-origem-pulou = from skipping the questionnaire
ajustes-perfil-origem-escolha = chosen here
ajustes-perfil-origem-padrao = default for people who already used the app
ajustes-perfil-origem-familia = no questionnaire of their own yet: the family's profile applies
ajustes-perfil-carregando = Loading the profile
ajustes-perfil-nivel = Experience level
ajustes-perfil-nivel-de = { $nome }'s experience level
ajustes-perfil-texto = The level decides what shows by default: in Beginner, the dashboard is simple and the advanced screens go under “More screens”; in Advanced, everything is in view. Nothing is deleted when you change it. Each family member can have their own level and language style: choose the member in “View as”, at the top.
ajustes-perfil-atual = Current: { $nivel } ({ $origem }).
ajustes-perfil-atual-em = Current: { $nivel } ({ $origem }, on { $data }).
ajustes-perfil-modo = { $nivel } mode
ajustes-perfil-linguagem = Language style
ajustes-perfil-linguagem-dica = It's also how the agent talks to you.
ajustes-perfil-explicacoes = Term explanations
ajustes-perfil-explicacoes-dica = { $icone } icons next to terms like CDI, XIRR and accrual; “By level” shows them in Beginner and Intermediate.
ajustes-perfil-respostas = Your answers

## Security

ajustes-seg-nunca = Never (not recommended)
ajustes-seg-minutos =
    { $n ->
        [one] { $n } minute
       *[other] { $n } minutes
    }
ajustes-seg-horas =
    { $n ->
        [one] { $n } hour
       *[other] { $n } hours
    }
ajustes-seg-trocada = Master password changed
ajustes-seg-trocada-dica = A backup was made before the change. If you had a recovery kit, make a new one: the old one no longer opens the vault.
ajustes-seg-trocar-form = Change the master password
ajustes-seg-senha-atual = Current password
ajustes-seg-informe-atual = Enter your current password.
ajustes-seg-nova = New password
ajustes-seg-confirmar = Confirm new password
ajustes-seg-recifrando = Re-encrypting the vault…
ajustes-seg-trocar = Change master password
ajustes-seg-kit-texto = The recovery kit opens the vault if the master password is forgotten. Whoever has the code has access to everything: **print it and keep it on paper, away from the computer**. Don't photograph it or save it to the cloud.
ajustes-seg-kit-nenhum = No kit made yet. Without it, forgetting the master password loses the data.
ajustes-seg-kit-ultimo = Last kit made on { $data }. It is valid until the next password change.
ajustes-seg-kit-velho = The master password changed after the last kit: the code you kept no longer opens the vault. Make a new kit and replace the paper.
ajustes-seg-kit-gerar = Generate recovery kit
ajustes-seg-kit = Recovery kit
ajustes-seg-kit-confirme = Confirm your master password
ajustes-seg-kit-fechar = Close (the code disappears from the screen)
ajustes-seg-kit-mostrar = Show the kit
ajustes-seg-kit-alerta = This code opens the vault without the password. Print it, keep it somewhere safe and close this window.
ajustes-seg-kit-gerado = Generated on
ajustes-seg-kit-como = How to use it
ajustes-seg-kit-como-valor = On the unlock screen, choose “Forgot your password?” and type the code.
ajustes-seg-keychain-desligado = Keychain unlock turned off
ajustes-seg-keychain-ligado = Keychain unlock turned on
ajustes-seg-touch-id = Unlock with Touch ID
ajustes-seg-touch-id-dica = The vault key stays in the keychain and only comes out after Touch ID (or your Mac password). It also lets `treasury mcp` read with the app closed.
ajustes-seg-keychain-indisponivel = Keychain not available on this system.
ajustes-seg-keychain-ligar = Turn on the keychain
ajustes-seg-keychain-ligar-dica = Confirm your master password to store the vault key in the keychain.
ajustes-seg-ligar = Turn on
ajustes-seg-bloqueio = Auto-lock
ajustes-seg-bloqueio-dica = After this much time without use, the vault closes and the key leaves memory.
ajustes-seg-bloqueio-ajustado = Auto-lock set
ajustes-seg-dormir = Lock on sleep or screen lock
ajustes-seg-dormir-dica = The vault closes when the computer sleeps, when the screen is locked (by shortcut, screen saver or hot corner) and when someone else switches into the session. On macOS; on other systems the inactivity lock applies.
ajustes-seg-capturas-titulo = Screenshots
ajustes-seg-capturas = Allow screenshots of Treasury
ajustes-seg-capturas-dica = Off (default): with privacy mode on, the window disappears from screenshots, recordings and screen sharing. On macOS and Windows.
ajustes-seg-capturas-dica-liberadas = On: the window shows up in screenshots, recordings and screen sharing, even with privacy mode on. Amounts stay hidden while privacy mode is on.
ajustes-seg-capturas-confirmar-titulo = Allow screenshots?
ajustes-seg-capturas-confirmar-texto = Any program that captures the screen (recorders, meeting apps, a program installed without you noticing) will see the Treasury window, even with privacy mode on. Turn it off again when you're done.
ajustes-seg-capturas-confirmar = Allow screenshots
ajustes-seg-capturas-liberadas = Screenshots allowed
ajustes-seg-capturas-bloqueadas = Screenshots blocked again
ajustes-atualizacao-titulo = Updates
ajustes-atualizacao-modo = What to do with a new version
ajustes-atualizacao-modo-baixar = Download and notify
ajustes-atualizacao-modo-baixar-dica = Checks when the app opens and every 6 hours, downloads and verifies the signature, and tells you it is ready. Installing always waits for your confirmation.
ajustes-atualizacao-modo-avisar = Notify only
ajustes-atualizacao-modo-avisar-dica = Checks on the same schedule and only tells you a new version exists. Nothing is downloaded until you ask, which helps on a metered connection.
ajustes-atualizacao-modo-desligado = Off
ajustes-atualizacao-modo-desligado-dica = Does not check on its own. New versions, including security fixes, only arrive when you check.
ajustes-atualizacao-modo-salvo = Updates: { $modo }
ajustes-atualizacao-disponivel = Version { $versao } available
ajustes-atualizacao-baixar = Download and verify
ajustes-atualizacao-privacidade = It only asks Treasury's GitHub releases which version is the newest: nothing about you or the vault goes along. Every downloaded version is checked against Treasury's signature before it can be installed.
ajustes-atualizacao-versao = Installed version: { $versao }
ajustes-atualizacao-ultima = last check: { $quando }
ajustes-atualizacao-indisponivel = Automatic updates start once Treasury is published on GitHub. Until then, new versions are installed by hand.
ajustes-atualizacao-erro = The last check failed: { $erro }
ajustes-atualizacao-pronta = Version { $versao } ready to install
ajustes-atualizacao-instalar = Install and restart
ajustes-atualizacao-procurar = Check now
ajustes-atualizacao-traduzir = Translate
ajustes-atualizacao-traduzido = Translated on this computer
ajustes-atualizacao-ver-original = Show original
ajustes-atualizacao-traduzir-google = Translate in Google Translate
ajustes-atualizacao-traduzir-google-dica = Opens the browser with only this version's notes: nothing about you or the vault goes along.
ajustes-atualizacao-traduzir-cortado = The notes are long: Google Translate got only the beginning.
ajustes-atualizacao-traduzir-falhou = macOS could not translate right now.
ajustes-atualizacao-em-dia = No new version found
ajustes-atualizacao-confirmar-titulo = Install version { $versao }?
ajustes-atualizacao-confirmar-texto = Treasury closes, installs the new version and opens again. The vault is saved first and will ask to be unlocked.
ajustes-atualizacao-ver = View
ajustes-seg-desbloqueio = Unlocking and locking
ajustes-seg-dividida = Split recovery
ajustes-seg-fido2 = Security keys (FIDO2)
ajustes-pin-desligado = PIN turned off
ajustes-pin-desbloquear = Unlock with a PIN on this computer
ajustes-pin-desbloquear-dica = A 6- to 12-digit PIN, stored only in this computer's keychain. Five wrong tries turn the PIN off and ask for the master password.
ajustes-pin-sem-keychain = The PIN needs the system keychain, which isn't available here.
ajustes-pin-ligar-titulo = Turn on the PIN
ajustes-pin-ligar-dica = Choose the PIN and confirm your master password.
ajustes-pin-ligar = Turn on PIN
ajustes-pin-campo = PIN (6 to 12 digits)
ajustes-pin-ligado = PIN turned on for this computer
ajustes-partes = Recovery shares
ajustes-partes-copiada = Share copied
ajustes-partes-texto = Keep each share in a different place (for example: on paper at home, with your spouse, in a bank safe deposit box). Any two recover the vault; one alone reveals nothing. They won't be shown again.
ajustes-partes-parte = Share { $n }
ajustes-partes-fido2-removidas =
    { $n ->
        [one] { $n } security key stopped working: register it again.
       *[other] { $n } security keys stopped working: register them again.
    }
ajustes-partes-geradas = Shares generated on { $data }. They keep working after you change the password.
ajustes-partes-dica = Splits recovery into 3 shares: any two open the vault if you forget the password.
ajustes-partes-novas-copias = Show new copies of the shares
ajustes-partes-gerar = Generate the 3 shares
ajustes-partes-trocar = Replace the shares (invalidates the old ones)
ajustes-partes-trocar-titulo = Replace the shares
ajustes-partes-trocar-dica = The old shares and the registered security keys stop working.
ajustes-partes-confirme = Confirm your master password to generate the shares.
ajustes-partes-gerar-botao = Generate
ajustes-partes-ja-guardei = I've stored them
ajustes-fido2-lista = Security keys
ajustes-fido2-cadastrada = Security key registered
ajustes-fido2-cadastrada-em = registered { $data }
ajustes-fido2-texto = A YubiKey or another FIDO2 key recovers the vault if you forget the password.
ajustes-fido2-so-no-app = Only in the Treasury app
ajustes-fido2-cadastrar = Register a security key
ajustes-fido2-toque = Touch the key twice when it blinks (create and confirm).
ajustes-fido2-conecte = Connect the security key (USB, or NFC on the reader) and try again.
ajustes-fido2-cadastrar-botao = Register
ajustes-fido2-placeholder = Home YubiKey 5C
ajustes-fido2-pin = Security key PIN (if it has one)
ajustes-fido2-remover = Remove security key
ajustes-fido2-remover-dica = It will no longer recover the vault.
ajustes-fido2-removida = Security key removed

## Vaults

ajustes-cofres-texto = Each vault is an encrypted .db file, with its own password, quick unlock (Touch ID, PIN) and backups in its own folder. Use them to keep things separate, like a business or your parents' finances. Switching vaults closes the current one (with the closing backup) and asks for the other one's password. Removing it from the list doesn't delete the file.
ajustes-backup-salvar-copia = Save copy
ajustes-backup-trazer = Bring from elsewhere
ajustes-backup-trazidos = Backups brought from elsewhere
ajustes-backup-trazidos-dica = They live in the "importados" subfolder of the backup folder and are not rotated: they only go away if you delete the file.
ajustes-pref-carteira = Portfolio
ajustes-pref-twr-diario = Daily return (advanced)
ajustes-pref-twr-diario-dica = Calculates the portfolio return day by day, chaining each business day. It is slower; the default is the monthly return (Modified Dietz).
ajustes-pref-twr-diario-ligado = Daily return on
ajustes-pref-twr-diario-desligado = Monthly return (default)

## Cost method for sales (Family)

ajustes-metodo-custo-titulo = Cost method for sales
ajustes-metodo-custo-dica = How the sales book calculates the cost of stocks and funds. The default follows the tax rule of the family's country; the result by average cost stays alongside, for Brazilian income tax.
ajustes-metodo-custo-campo = Method
ajustes-metodo-custo-auto = By the family's country ({ $metodo })
ajustes-metodo-custo-average = Average cost
ajustes-metodo-custo-fifo = First in, first out (FIFO)
ajustes-metodo-custo-uk_pool = UK rule (same day, 30 days and pool)
ajustes-metodo-custo-salvo = Cost method updated
ajustes-metodo-custo-renda-fixa = Fixed income and Treasury bonds stay by lots (first in, first out). The position cost and the unrealized result of the portfolio stay by average cost.
