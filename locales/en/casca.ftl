# App shell: sidebar, top bar, command bar, themes, tasks and lock (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Menu groups and screens (search words are comma-separated)

casca-grupo-fluxo = Cash flow
casca-grupo-investimentos = Investments
casca-grupo-gestao = Management

casca-tela-painel = Dashboard
casca-tela-painel-resumo = Net worth, this month's cash flow, savings rate, portfolio vs. inflation and due dates.
casca-tela-painel-busca = home,dashboard,overview,net worth

casca-tela-familia = Family
casca-tela-familia-resumo = Who is in the family, each member's income (salary, payslips, informal income) and their accounts and cards.
casca-tela-familia-busca = income,salary,payslip,paycheck,members,holders,dependents,earnings,people

casca-tela-contas = Accounts
casca-tela-contas-resumo = Balances by account, cards, brokers and foreign accounts; reconciliation.
casca-tela-contas-busca = balance,bank,card,reconciliation

casca-tela-lancamentos = Transactions
casca-tela-lancamentos-resumo = Income, expenses and transfers with filters, splits, import and rules.
casca-tela-lancamentos-busca = transactions,expenses,income,statement,import,ofx,csv

casca-tela-orcamento = Budget
casca-tela-orcamento-resumo = Planned vs. actual by category, on an accrual or cash basis, and goals.
casca-tela-orcamento-busca = budget,goals,categories,planned
casca-tela-consumo = Consumption
casca-tela-consumo-resumo = Everything the family buys, item by item: groceries, clothes, electronics and the rest, with food at home and out and the price per kg over time.
casca-tela-consumo-busca = consumption,purchases,shopping,food,groceries,supermarket,clothes,electronics,receipt,products,price per kg,restaurant,delivery

casca-tela-dividas = Debts
casca-tela-dividas-resumo = Loans and financing: contracts, schedule and simulator.
casca-tela-dividas-busca = loan,financing,mortgage,amortization,installments

casca-tela-carteira = Portfolio
casca-tela-carteira-resumo = Positions, trades, dividends and nominal and real returns; physical assets.
casca-tela-carteira-busca = investments,positions,returns,income,dividends,assets,property,vehicle,physical assets

casca-tela-ativos = Assets
casca-tela-ativos-resumo = Asset detail, indicators, calculated price, comparison and calendar.
casca-tela-ativos-busca = stocks,fii,etf,watchlist,quote,indicators,compare
casca-tela-mercado = Market
casca-tela-mercado-resumo = Find assets across the whole market: overview by sector, explorer with the score, map and rankings.
casca-tela-mercado-busca = market,explorer,screener,ranking,sector,score,watch,discover

casca-tela-relatorios = Reports
casca-tela-relatorios-resumo = Reports by period, category and family member; income tax and DARF.
casca-tela-relatorios-busca = tax,income tax,darf,return,export,pdf
casca-tela-calculadoras = Calculators
casca-tela-calculadoras-resumo = Quick math to test an idea: compound interest, living off income, cash × installments and rent × buy. Nothing is saved to the vault.
casca-tela-calculadoras-busca = calculator,simulator,interest,compound interest,income,independence,installments,cash,rent,mortgage,home,buy

casca-tela-impostos = Taxes
casca-tela-impostos-resumo = Estimate of the taxes paid in the year (consumption, earned income, property), with sources.
casca-tela-impostos-busca = taxes,tax burden,taxes paid,consumption,sales tax,payroll tax,income tax,property tax,tax statement

casca-tela-agente = Agent
casca-tela-agente-resumo = AI financial manager: chat, approval queue, alerts and history.
casca-tela-agente-busca = ai,chat,approval,alerts,mcp,assistant

casca-tela-ajustes = Settings
casca-tela-ajustes-resumo = Security, backups, data sources, AI engines, themes and history.
casca-tela-ajustes-busca = settings,preferences,security,backup,theme,password,language

casca-tela-desenvolvedor = Developer
casca-tela-desenvolvedor-resumo = Agent feedback for the Treasury developer: bugs, improvements and gaps.
casca-tela-desenvolvedor-busca = feedback,dev,bug,improvement,issue,github,developer mode

## Sidebar

casca-menu-expandir = Expand sidebar
casca-menu-recolher = Collapse sidebar
casca-familia = Family
casca-sem-nome = Unnamed
casca-telas = Screens
casca-mais-telas = More screens
casca-cofre-aberto = Vault open
casca-auto-bloqueio = Auto-lock
casca-minutos = { $minutos } min
casca-desligado = off
casca-cifra = Cipher
casca-bloquear-cofre = Lock vault

## Top bar

casca-demo = Demo
casca-demo-dica = Demo vault: made-up data created by the assistant. Your vault is unchanged; switch back in Vaults.
casca-mock = Mock
casca-mock-dica = Data from a fake in-memory core (development). Nothing is saved.
casca-versao-antiga = Update the app
casca-versao-antiga-dica = This vault was already opened by Treasury { $versao }, newer than this one. Update the app: settings saved by the newer version may not show here.
casca-buscar = Search or run…
casca-privacidade = Privacy
casca-agente-ao-lado = Agent on the side
casca-modo-privacidade = Privacy mode
casca-ver-como = View as
casca-familia-toda = Whole family

## Profile levels

casca-nivel-iniciante = Beginner
casca-nivel-intermediario = Intermediate
casca-nivel-avancado = Advanced
casca-nivel-iniciante-dica = A simple dashboard (what came in, what went out, what's left), an emergency fund goal and everyday-language tips. Advanced screens are under “More screens”.
casca-nivel-intermediario-dica = A dashboard with categories and month-over-month comparison, budget and reports. Investments and taxes are under “More screens”.
casca-nivel-avancado-dica = Everything in view: full dashboard, portfolio, assets, reports and taxes.

## Themes

casca-tema-escolher = Choose theme (current: { $tema })
casca-temas = Themes
casca-temas-classicos = Classic
casca-temas-especiais = Special
casca-temas-divertidos = Fun
casca-tema-dark = Dark
casca-tema-dark-dica = Bluish graphite with a cyan accent. The default.
casca-tema-light = Light
casca-tema-light-dica = Neutral light gray, white cards and blue ink.
casca-tema-mono-dica = Shades of gray, no color at all: just the information.
casca-tema-midnight = Midnight
casca-tema-midnight-dica = Deep navy blue with a champagne accent.
casca-tema-paper = Paper
casca-tema-paper-dica = Warm light, like notebook paper, with blue ink.
casca-tema-glass = Glass
casca-tema-glass-dica = Frosted glass over a mesh of blue, teal and violet.
casca-tema-scanner = Scanner
casca-tema-scanner-dica = Glossy black, with a red scanner sweeping the top bar.
casca-tema-orbit = Orbit
casca-tema-orbit-dica = A black hole at the top of the screen: stars and the golden disk slowly turning.
casca-tema-neon = Neon
casca-tema-neon-dica = A city at night: pink and cyan neon, orange haze and endless rain.
casca-tema-lofi-dica = A calm afternoon in Japan: washi paper, sumi ink, indigo and autumn leaves drifting in the wind.
casca-tema-phosphor = Phosphor
casca-tema-phosphor-dica = Green code falling on black, like an old terminal.

## Command bar

casca-cmd-barra = Command bar
casca-cmd-placeholder = Go to a screen, run an action or ask the agent…
casca-cmd-nenhum = No commands found.
casca-cmd-navegar = navigate
casca-cmd-executar = run
casca-cmd-abrir-fechar = open/close
casca-cmd-grupo-navegacao = Navigation
casca-cmd-grupo-cadastro = Records
casca-cmd-grupo-cofre = Vault
casca-cmd-grupo-exibicao = View
casca-cmd-grupo-agente = Agent
casca-cmd-ir-para = go to,open
casca-cmd-importar = Import files
casca-cmd-importar-busca = ofx,csv,statement,credit card bill,brokerage note,sinacor,b3,pdf,spreadsheet
casca-cmd-bloquear-busca = lock,padlock,exit,close vault
casca-cmd-privacidade-ativar = Turn on privacy mode
casca-cmd-privacidade-desativar = Turn off privacy mode
casca-cmd-privacidade-busca = hide amounts,hide,show amounts,share screen
casca-cmd-menu-busca = sidebar,menu,hide menu,more space,workspace
casca-cmd-tema-alternar = Toggle light/dark theme
casca-cmd-tema-busca = theme,dark mode,light mode,appearance
casca-cmd-tema = Theme: { $tema }
casca-cmd-atual = Current
casca-cmd-temas-busca = theme,colors,appearance
casca-cmd-claro = light
casca-cmd-escuro = dark
casca-cmd-cinza-busca = gray black and white
casca-cmd-vidro-busca = frosted translucent glass
casca-cmd-agente-alternar = Open or close the agent on the side
casca-cmd-agente-busca = ai,companion,panel,chat,manager,assistant
casca-cmd-perguntar = Ask the agent
casca-cmd-perguntar-busca = ai,question,manager
casca-cmd-perguntar-texto = Ask the agent: “{ $texto }”

## Background tasks

casca-tarefas = Background tasks
casca-tarefas-andamento = Background tasks ({ $n } running)
casca-tarefas-nenhuma = No tasks running
casca-tarefas-n = { $n } running
casca-tarefas-vazio = Nothing running right now. Data collections (indices, quotes, fundamentals), agent conversations and routines (month close, alerts) show up here while they work.
casca-tarefas-em-andamento = Running tasks
casca-tarefas-ultimos = Last 30 minutes
casca-tarefas-recentes = Recent tasks
casca-tarefa-andamento = Running
casca-tarefa-concluida = Done
casca-tarefa-falhou = Failed
casca-tarefa-cancelada = Cancelled
casca-tarefa-cancelar = Cancel { $tarefa }
casca-tarefa-cancelada-toast = Task cancelled
casca-tarefa-nao-cancelou = Couldn't cancel
casca-cancelando = Cancelling…
casca-rotina = Routine
casca-agora = just now
casca-ha-segundos = { $s } s ago
casca-ha-minutos = { $m } min ago
casca-ha-horas = { $h } h { $m } ago

## Core collections and downloads (docs/background-tasks.md)

casca-coleta-market-sync = Indices, exchange rates and quotes
casca-coleta-indices-sync = Indices and exchange rates
casca-coleta-research-sync = Fundamentals and market score
casca-coleta-mail-sync = Ingestion e-mail
casca-coleta-update-download = Treasury update
casca-coleta-curta-market-sync = Quotes
casca-coleta-curta-indices-sync = Indices
casca-coleta-curta-research-sync = Fundamentals
casca-coleta-curta-mail-sync = E-mail
casca-coleta-curta-update-download = Update
casca-etapa-indices = Indices and exchange rates
casca-etapa-prices = Quotes
casca-etapa-funds = Fund and pension quotas (CVM)
casca-etapa-income = Announced income
casca-etapa-fundamentals = CVM, SEC and B3 files
casca-etapa-scores = Market score
casca-etapa-mail = Fetching messages
casca-etapa-download = Downloading
casca-tarefa-automatica = Automatic
casca-tarefa-de = { $feito } of { $total }
casca-tarefa-feitos = { $n } done
casca-tarefa-progresso = Progress: { $tarefa }
casca-coleta-chip = { $tarefa } in progress
casca-coleta-chip-pct = { $tarefa } in progress, { $pct }
casca-coleta-chip-mais = and { $n } more
casca-coleta-cancelada-dica = { $tarefa }: what had already arrived was saved. The automatic collection comes back in an hour.

## Lock and errors

casca-autobloqueio = Auto-lock
casca-autobloqueio-em = The vault will lock due to inactivity in
casca-autobloqueio-menos-minuto = less than a minute
casca-autobloqueio-continuar = . Move the mouse or press a key to continue.
casca-bloqueado-inatividade = Vault locked due to inactivity
casca-bloqueado-inatividade-dica = Enter your master password to pick up where you left off.
casca-erro-desconhecido = Unknown error.
casca-falha-titulo = Interface error
casca-falha-texto = Something went wrong showing this screen. Your data is still in the vault, untouched.
casca-recarregar = Reload
casca-ir-painel = Go to the dashboard
casca-menu-apoiar = Support the project…
casca-pular-conteudo = Skip to content
casca-carregando-tela = Loading screen
casca-abrir = Open
casca-nao-encontrada = Screen not found
casca-sem-sinal = No signal
casca-nao-encontrada-titulo = This address doesn't lead to any screen
casca-nao-encontrada-texto = Use the sidebar or the command bar (⌘K) to pick a screen.
casca-navegador-falhou = Couldn't open the browser

## Language (first screen and Settings › Appearance and use)

casca-idioma = Language
casca-idioma-titulo = Choose your language
casca-idioma-texto = You can change it later in Settings › Appearance and use.
casca-idioma-dica = Applies to this computer, in every vault, and changes right away, with no restart. Names of accounts and categories you created stay as they are.
casca-idioma-pseudo = Pseudo-language (translation test)


## ⌘K bar: destinations, quick actions and vault search

casca-cmd-grupo-recentes = Recent
casca-cmd-grupo-acoes = Quick actions
casca-cmd-grupo-ir = Go to
casca-cmd-grupo-contas = Accounts
casca-cmd-grupo-categorias = Categories
casca-cmd-grupo-metas = Goals
casca-cmd-grupo-bens = Physical assets
casca-cmd-grupo-lancamentos = Recent transactions
casca-cmd-ir-aba = { $tela } › { $aba }
casca-cmd-familia-renda = Income and payslips
casca-cmd-novo-lancamento = New transaction (expense)
casca-cmd-novo-lancamento-busca = expense,spending,purchase,payment,add
casca-cmd-nova-receita = New income
casca-cmd-nova-receita-busca = income,deposit,salary,earnings,received
casca-cmd-nova-transferencia = New transfer
casca-cmd-nova-transferencia-busca = transfer,move,between accounts,wire
casca-cmd-compra-parcelada = Card purchase in installments
casca-cmd-compra-parcelada-busca = installments,card,credit,split payment
casca-cmd-ajuste-saldo = Adjust an account balance
casca-cmd-ajuste-saldo-busca = adjustment,balance,sync,fix balance,reconcile,bank,statement
casca-cmd-importar-holerite = Import payslip
casca-cmd-nova-conta = New account
casca-cmd-nova-conta-busca = account,checking,savings,cash,bank,open account
casca-cmd-novo-cartao = New credit card
casca-cmd-novo-cartao-busca = card,credit,bill,limit
casca-cmd-nova-meta = New goal
casca-cmd-nova-meta-busca = goal,target,dream,save
casca-cmd-trocar-cofre = Switch vault
casca-cmd-trocar-cofre-busca = vault,database,open,switch,another vault
casca-cmd-cofre-demo = Create demo vault
casca-cmd-cofre-demo-busca = demo,sample,example,test,made-up
casca-cmd-idioma = Language: { $idioma }
casca-cmd-idioma-busca = language,idioma,translate
casca-cmd-entidade-conta-busca = account,balance,bank
casca-cmd-entidade-categoria-busca = category,spending,transactions
casca-cmd-entidade-meta-busca = goal,target
casca-cmd-sin-holerite = payslip,paystub,paycheck,salary,income,holerite
casca-cmd-sin-extrato = statement,ofx,csv,qif,import,bank
casca-cmd-sin-cupom = receipt,nfc-e,invoice,qr code,grocery
casca-cmd-sin-aposentadoria = retirement,pension,social security,future
casca-cmd-sin-bens = property,real estate,house,apartment,car,vehicle
casca-cmd-sin-proventos = dividends,income,interest,payouts
casca-cmd-sin-indices = indices,inflation,interest rate,exchange,dollar,quotes
casca-cmd-sin-motores = engine,ai,model,groq,openai,anthropic,api key

casca-estudo-pronto = AI study ready

# Conferência da assinatura ao abrir (src-tauri/src/integrity.rs)
casca-assinatura-titulo = Treasury was modified
casca-atualizacao-nada-baixado = No downloaded update to install.
casca-atualizacao-sem-yubikey = The new version came without the Treasury YubiKey signature and was not accepted.
casca-atualizacao-sem-chaves = The new version was not accepted: this installation does not have the Treasury YubiKey keys to check the signature.
casca-atualizacao-yubikey-invalida = The YubiKey signature of the new version does not match ({ $motivo }); it was not accepted.
casca-atualizacao-minisign-invalida = The minisign signature of the new version does not match ({ $motivo }); it was not accepted.
casca-atualizacao-versao-assinada = The new version was announced as { $anunciada }, but the package was signed for { $assinada }; it was not accepted.
casca-atualizacao-instalar-manual = The automatic installation did not finish ({ $motivo }). The package, with its signatures checked, is in { $caminho }: install it with the system package installer.
casca-assinatura-alterada = This Treasury's signature doesn't match: the app in Applications may have been replaced or changed after it was installed. Don't type your master password into it. Reinstall Treasury from the original file and open it again.

## App activity (state/activity.ts) and Phosphor pieces

casca-atividade-importacao = Import
casca-atividade-cupom = Reading a receipt
casca-atividade-email = Reading e-mail
casca-atividade-cenario = Recalculating the scenario
casca-atividade-cotacoes = Syncing quotes
casca-phosphor-processando = Processing
casca-phosphor-tarefas = { $n } tasks
casca-phosphor-falha = Failed
casca-phosphor-agente = > agent connected... analyzing data
