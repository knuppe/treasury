# Profile: welcome and questionnaire, simple dashboard and per-member panel (docs/profile.md,
# docs/i18n-design.md). Revision: AI draft, not yet reviewed by a native speaker. Text between
# ** is shown in bold; the arguments valor, meta, entrou, saiu, botao and link are elements that
# the screen puts in place.

## Welcome and questionnaire

perfil-confianca = Your answers help make the app work your way. You can change this later.
perfil-questionario-de = { $nome }'s questionnaire
perfil-refazer = Retake the questionnaire
perfil-boas-vindas = Welcome to Treasury
perfil-boas-vindas-texto = In 2 minutes the app adapts to you: what shows on screen, how things are explained and where to start.
perfil-personalizar = I want to tailor the app to me
perfil-personalizar-dica = 7 quick multiple-choice questions.
perfil-completo = I know what I want: use the full interface
perfil-completo-dica = Advanced mode, with every screen in view. No questions.
perfil-demo-convite = Just want to try the app? { $botao } with sample data, separate from this one.
perfil-demo-criar = Create a demo vault
perfil-resultado = We set up the app in { $nivel } mode
perfil-resultado-dica = You can change the level whenever you want in Settings › Profile, and everything that's hidden stays one click away.
perfil-comecar = Get started
perfil-pergunta = Question { $n } of { $total }
perfil-progresso = Questionnaire progress
perfil-pular = Skip this question
perfil-concluir = Finish

## Simple dashboard

perfil-mes-vazio = No money in or out recorded this month yet. How about recording the first one?
perfil-mes-sem-entrada = { $saida } went out and no income is recorded this month yet.
perfil-mes-sobrou = { $sobra } left over, or { $pct }% of what came in. You're on the right track.
perfil-mes-faltou = { $falta } more went out than came in. It happens: see below where the money went and where you can adjust.
perfil-resumo-mes = Month summary
perfil-entrou = Money in
perfil-saiu = Money out
perfil-sobrou = Left over
perfil-reserva = Your emergency fund
perfil-calculando = Calculating…
perfil-reserva-comecar = Record your everyday spending (groceries, rent, bills) for a month and the app works out how much to set aside. To start, a good goal is **{ $valor }**.
perfil-reserva-meta = Goal: { $meta }, or { $meses } months of your essential spending.
perfil-reserva-ja-tem = You already have { $valor }.
perfil-reserva-nada = Nothing saved yet: you can start small.
perfil-reserva-progresso = Emergency fund progress
perfil-reserva-completa = Congratulations! Your emergency fund is complete.
perfil-reserva-caminho = { $pct }% of the way.
perfil-reserva-criar = Create the emergency fund goal
perfil-reserva-criar-frase = { $link } to track how much you've saved.
perfil-onde-foi = Where the money went
perfil-onde-item = { $valor } ({ $pct }% of what went out)
perfil-onde-vazio = No spending recorded this month yet.
perfil-sem-categoria =
    { $n ->
        [one] { $n } transaction without a category.
       *[other] { $n } transactions without a category.
    }
perfil-escolher-categoria = Choose the category
perfil-seu-mes = Your month: { $mes }
perfil-anotar-gasto = Record an expense
perfil-anotar-entrada = Record income
perfil-dica-do-dia = Tip of the day
perfil-ver-completo = See full dashboard
perfil-o-que-e = What is { $termo }?

## Per family member panel

perfil-membros-titulo = By family member
perfil-membros = Members
perfil-membro-nivel-familia = { $nivel } (family's)
perfil-membro-entrou = In
perfil-membro-saiu = Out
perfil-membro-sobrou = Left over
perfil-membro-patrimonio = Net worth { $valor }
perfil-membro-guardou = saved { $pct } of what came in over 12 months
perfil-membro-contas =
    { $n ->
        [one] { $n } account
       *[other] { $n } accounts
    }
perfil-membro-dividas =
    { $n ->
        [one] { $n } debt
       *[other] { $n } debts
    }
perfil-ver-como = View as { $nome }
perfil-fazer-questionario = Take the questionnaire
perfil-membros-familia = Family (no member, like joint accounts): { $entrou } in, { $saiu } out.
