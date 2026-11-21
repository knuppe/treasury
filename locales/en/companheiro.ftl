# Agent companion: the agent's side panel on every screen (⌘J), with the screen context, quick
# requests and proposals approved right there (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Quick requests (the text goes to the AI engine)

companheiro-pedido-aqui-rotulo = What can I do here?
companheiro-pedido-aqui = What can I do on this screen? Use the screen context and the data to suggest concrete next steps, and offer to propose the changes.
companheiro-pedido-resumo-rotulo = Summarize this screen
companheiro-pedido-resumo = Summarize what this screen shows right now, with the main numbers, and point out what needs attention.
companheiro-pedido-cadastro-rotulo = Where do you keep accounts?
companheiro-pedido-cadastro = I want to register where the family keeps money. Ask where we have accounts (banks, credit cards, brokers, foreign accounts) and, for each one: propose adding it (create_account with institution_name; cards with closing and due days), tell me the best way to bring in the data (OFX or CSV from the bank, PDF card bill, B3 Investor Area report, PDF brokerage notes) and where to download each file. Don't make up balances: ask.

## Panel

companheiro-rotulo = Agent companion
companheiro-abrir = Open the agent
companheiro-fechar = Close the agent
companheiro-nova-conversa = New conversation
companheiro-anteriores = Previous conversations
companheiro-opcoes = Agent options
companheiro-largura = Agent panel width
companheiro-log = Conversation in the agent panel
companheiro-mensagem = Message to the agent (panel)
companheiro-placeholder = Ask something or request a registration…
companheiro-enviar-assim = Send as is
companheiro-aviso = AI can make mistakes. Check before approving.
companheiro-aviso-envio = This conversation will be sent to { $provedor }. AI can make mistakes. Check before approving.
companheiro-autonomo-tag = Autonomous
companheiro-autonomo-tag-dica = Autonomous mode on: reversible writes apply right away. Turn it off in Agent options.
companheiro-fora-das-telas = Outside the screens

## Options and autonomous mode

companheiro-autonomo-ligado-dica = Reversible writes (categorize, register, budget) apply right away and can be undone. Sensitive ones keep asking for approval one by one.
companheiro-autonomo-desligado-dica = Off: every agent write becomes a proposal for you to approve.
companheiro-autonomo-ligado = Autonomous mode on
companheiro-autonomo-desligado = Autonomous mode off
companheiro-autonomo-ligado-texto = Reversible writes apply right away; sensitive ones go to approval.
companheiro-autonomo-desligado-texto = Every agent write becomes a proposal.
companheiro-nao-foi-possivel = Could not change it
companheiro-tela-cheia = Open in full screen ({ $tela })

## Screen context

companheiro-contexto-vai = Goes to the AI engine with each message, as data (never as an instruction). Amounts in cents.
companheiro-contexto-desligado = Off: only the message goes to the engine.
companheiro-contexto-grande = Context too large to send; only the message goes.
companheiro-contexto-previa = Page context preview
companheiro-contexto-ver = See what goes along with the message
companheiro-contexto-pagina = Context of this page: { $tela }
companheiro-enviar-contexto = Send context

## Welcome and suggestions

companheiro-sugestoes = Agent suggestions
companheiro-como-ajudar = How can I help?
companheiro-vendo = I'm looking at { $tela }. I can look things up, register and organize; every write becomes a proposal for you to approve here.
companheiro-pedidos-rapidos = Quick requests
companheiro-nesta-tela = On this screen
companheiro-nada-pendente = Nothing pending on this screen.
companheiro-perguntar = Ask the agent: { $titulo }
companheiro-abrir-tela = Open the screen

## Previous conversations

companheiro-voltar = Back to the conversation
companheiro-buscar-conversa = Search conversations
companheiro-buscar-titulo = Search by title
companheiro-nenhuma-com = No conversation with “{ $busca }”.
companheiro-fixadas = Pinned
companheiro-conversas-fixadas = Pinned conversations
companheiro-recentes = Recent
companheiro-conversas-recentes = Recent conversations
companheiro-ver-na-tela = See on screen { $tela }

## Proposal in the conversation

companheiro-falha-aplicar = The proposal could not be applied
companheiro-aprovada = Proposal approved
companheiro-aprovada-texto = The change applies now and can be undone in the history.
companheiro-rejeitada = Proposal rejected
companheiro-carregando = Loading the proposal…
companheiro-ver-alteracoes = See the changes
companheiro-sensivel = Sensitive: check before approving.
