# AI agent (core: `ai`; docs/ai-agent.md): errors and warnings for engines, attachments, CLIs,
# spending limits, proposals, alerts and conversations, and the memory's history reasons. Texts
# that only go to the model (instructions, context) stay in English in the code.

## Attachments

nucleo-ia-anexo = attachment
nucleo-ia-anexo-expirou = attachment (it expired or the vault was locked; attach it again)
nucleo-ia-anexo-nao-guardado = attachment (not stored)
nucleo-ia-anexo-pdf-senha = the PDF is password-protected: remove the password and attach it again
nucleo-ia-anexo-pdf-ilegivel = could not read the PDF: { $detalhe }
nucleo-ia-anexo-planilha-ilegivel = could not read the spreadsheet: { $detalhe }
nucleo-ia-anexo-vazio = { $nome }: empty file
nucleo-ia-anexo-grande = { $nome }: file too large ({ $tamanho }; max. { $maximo } per file)
nucleo-ia-anexo-grande-maximo = { $nome }: file too large (max. { $maximo } per file)
nucleo-ia-anexo-imagem-grande = { $nome }: image too large ({ $tamanho }; max. { $maximo } after resizing)
nucleo-ia-anexo-imagem-invalida = { $nome }: invalid or corrupted image
nucleo-ia-anexo-imagem-dimensoes = { $nome }: image of { $largura }×{ $altura } px (max. { $maximo } px per side)
nucleo-ia-anexo-heic = { $nome }: HEIC image not converted; export it as JPEG or PNG and attach it again
nucleo-ia-anexo-texto-cortado = text cut at the first { $mil } thousand characters
nucleo-ia-anexo-pdf-sem-texto = PDF without extractable text (scanned?): only engines that read PDFs understand it
nucleo-ia-anexo-pdf-grande = PDF over { $tamanho } or { $paginas } pages: only the extracted text is sent
nucleo-ia-anexo-compactado = { $nome }: file type not accepted (archives and old Word documents are not read)
nucleo-ia-anexo-documento-ilegivel = { $nome }: unreadable document
nucleo-ia-anexo-documento-sem-texto = { $nome }: document without text
nucleo-ia-anexo-texto-vazio = { $nome }: empty text file
nucleo-ia-anexo-tipo-nao-aceito = { $nome }: file type not accepted (PNG, JPEG, WebP, GIF or HEIC images; PDF; text, CSV, JSON, OFX, XML; Office spreadsheets and documents)
nucleo-ia-anexo-base64 = invalid attachment (base64)
nucleo-ia-anexo-maximo = at most { $maximo } attachments per message

## Engines and routes

nucleo-ia-motor = engine
nucleo-ia-motor-id = engine { $id }
nucleo-ia-provider-desconhecido = unknown provider: { $provider }
nucleo-ia-motor-desligado = engine turned off
nucleo-ia-motor-sem-chave = no API key saved
nucleo-ia-motor-sem-template = no command template
nucleo-ia-motor-sem-modelo = no model set
nucleo-ia-endpoint-so-local = use https:// (http only for servers on this computer)
nucleo-ia-endpoint-https = the endpoint must start with https://
nucleo-ia-endpoint-invalido = invalid endpoint
nucleo-ia-endpoint-obrigatorio = enter the endpoint (the API base URL)
nucleo-ia-template-vazio = the command template is empty
nucleo-ia-template-sem-prompt = the template needs the {"{prompt}"} placeholder
nucleo-ia-template-so-cli = a command template only applies to CLIs
nucleo-ia-template-obrigatorio = enter the command template for the custom CLI
nucleo-ia-template-colchete-sem-abrir = `]` without `[` in the template
nucleo-ia-template-colchete-sem-fechar = `[` without `]` in the template
nucleo-ia-template-aspas = unclosed quotes in the template
nucleo-ia-nome-longo = name too long (max. { $maximo })
nucleo-ia-nome-obrigatorio = enter the name
nucleo-ia-rota-motor-repetido = the same engine appears twice in the list

## CLIs

nucleo-ia-cli-concluida = done
nucleo-ia-cli-opencode-erro = OpenCode ended with an error
nucleo-ia-cli-prompt-argumento = the { $cli } template needs {"{prompt}"} as an argument of its own
nucleo-ia-cli-claude-stream = the Claude Code template needs --output-format stream-json to send images
nucleo-ia-cli-sem-imagens = { $motor } doesn't read images
nucleo-ia-cli-nao-encontrada = { $programa } not found in the PATH (is the CLI installed?)
nucleo-ia-cli-nao-iniciou = could not start { $programa }: { $detalhe }
nucleo-ia-cli-tempo-limite = the CLI went over the time limit
nucleo-ia-cli-erro = the CLI ended with an error
nucleo-ia-cli-terminou = { $programa } ended with { $status }{ $saida }

## Agent and conversations

nucleo-ia-agente-nao-le-imagens = doesn't read images ({ $nomes })
nucleo-ia-agente-nao-le-pdf = doesn't read the PDF without text ({ $nomes })
nucleo-ia-agente-contexto-objeto = the screen context must be a JSON object
nucleo-ia-agente-contexto-grande = screen context too large (max. { $maximo } characters)
nucleo-ia-agente-sem-motor = No AI engine is set up for this task. Add one in Settings › AI and agent.
nucleo-ia-agente-sem-motor-erro = no AI engine is set up for this task; add one in Settings › AI and agent
nucleo-ia-agente-cli-sem-restricao = CLIs are not limited to this task's tools
nucleo-ia-agente-precisa-api = this task needs an API engine or a local model (CLIs see every tool)
nucleo-ia-agente-anexos-sem-motor = No engine in the route reads these attachments: { $motor } { $motivo }. Choose an engine that reads images or send without them.
nucleo-ia-agente-nenhum-motor = no engine available
nucleo-ia-agente-cancelado = cancelled
nucleo-ia-agente-limite-passos = (I stopped here: this answer reached its limit of steps with tools.)
nucleo-ia-agente-escreva = write the message
nucleo-ia-agente-mensagem-longa = message too long (max. 20,000 characters)
nucleo-ia-agente-nao-iniciou = could not start the agent
nucleo-ia-agente-teste-titulo = Engine connection test
nucleo-ia-job-expirou = job (finished more than 30 minutes ago?)
nucleo-ia-conversa = conversation
nucleo-ia-conversa-sem-historico = no saved conversation: the agent's history is turned off in Settings › Agent

## Spending limits and prices

nucleo-ia-preco = price
nucleo-ia-preco-modelo-obrigatorio = enter the model (or * for the provider's default)
nucleo-ia-preco-negativo = prices can't be negative
nucleo-ia-limite-so-api = spending limits apply to API providers (CLIs and local models have no cost in the app)
nucleo-ia-limite-negativo = limits can't be negative
nucleo-ia-limite-diario = { $provider } daily spending limit reached ({ $gasto } of { $limite })
nucleo-ia-limite-mensal = { $provider } monthly spending limit reached ({ $gasto } of { $limite })
nucleo-ia-uso-periodo = `from` can't be after `to`

## API keys

nucleo-ia-chave-obrigatoria = enter the key
nucleo-ia-chave-invalida = invalid key
nucleo-ia-chave-documento-grande = document too large for the vault
nucleo-ia-chave-nao-abre = the saved key doesn't open with the vault's current key; save it again

## Proposals, alerts, routines and settings

nucleo-ia-proposta-id = proposal { $id }
nucleo-ia-proposta-selecione = select at least one proposal
nucleo-ia-proposta-nao-pendente = proposal { $id } is not pending
nucleo-ia-proposta-retirada = Withdrawn: { $motivo } (“{ $alteracao }”) before approval.
nucleo-ia-proposta-retirada-usuario = you changed the same data
nucleo-ia-proposta-retirada-importacao = an import changed the same data
nucleo-ia-proposta-retirada-agente = another change by the agent, already applied, changed the same data
nucleo-ia-proposta-retirada-outra = another change touched the same data
nucleo-ia-alerta = alert
nucleo-ia-alerta-id = alert { $id }
nucleo-ia-alerta-selecione = select at least one alert
nucleo-ia-rotina-rodando = { $rotina } is already running
nucleo-ia-retencao = conversation retention: choose 30, 90, 180 or 365 days, or never (0)

## Manager and procedure memory (history reasons)

nucleo-ia-memoria-gestor = Manager memory
nucleo-ia-memoria-gestor-removida = Manager memory removed
nucleo-ia-memoria-nota = note
nucleo-ia-memoria-nota-tamanho = the note must have between 1 and 2,000 characters
nucleo-ia-memoria-nota-nova = Procedure memory: new note
nucleo-ia-memoria-nota-atualizada = Procedure memory: note updated
nucleo-ia-memoria-nota-apagada = Procedure memory: note deleted

## API providers

nucleo-ia-provider-rede = network failure: { $detalhe }
nucleo-ia-provider-respondeu = { $provider } answered { $status }: { $mensagem }
nucleo-ia-provider-conexao = connection interrupted: { $detalhe }
nucleo-ia-provider-resposta-invalida = invalid response from the provider: { $detalhe }
nucleo-ia-provider-streaming = streaming error

## Extra MCPs (ai.mcp_extra.*)
nucleo-mcpx-erro-nao-encontrado = extra MCP not found
nucleo-mcpx-erro-nome = the MCP name must have 2 to 24 lowercase letters, digits or _, start with a letter and not be "treasury"
nucleo-mcpx-erro-nome-repetido = there is already an MCP with this name in this vault
nucleo-mcpx-erro-rotulo = enter a label of up to 60 characters
nucleo-mcpx-erro-comando = enter the command that starts the MCP server
nucleo-mcpx-erro-argumentos = invalid arguments (up to 64, no line breaks)
nucleo-mcpx-erro-url = the address must start with https:// (or http://localhost for a local server)
nucleo-mcpx-erro-variavel = invalid or repeated variable: { $nome }
nucleo-mcpx-erro-cabecalho = invalid or repeated header: { $nome }
nucleo-mcpx-erro-cabecalho-valor = a header value has invalid characters
nucleo-mcpx-erro-segredo-vazio = enter the secret value of { $nome }
nucleo-mcpx-erro-limite = at most { $max } extra MCPs per vault
nucleo-mcpx-erro-conexao = could not talk to the MCP { $servidor }: { $erro }
nucleo-mcpx-erro-tempo = the server did not answer in time
nucleo-mcpx-erro-pedido = this confirmation request was already answered or expired
nucleo-mcpx-aviso-sem-conexao = The MCP { $servidor } did not connect; the agent continues without it.
nucleo-mcpx-recusada = declined
nucleo-ia-cli-tempo-limite-parcial = ⏱ The answer was cut off at the { $minutos } min time limit. Above is what the agent did until then; whatever it recorded or proposed through Treasury's tools is saved. To continue, ask it to pick up where it stopped (or raise the limit in Settings › AI).
nucleo-ia-tempo-cli = choose 15, 30, 60 or 120 minutes

## Agent questions (ask_user)
nucleo-ia-pergunta-sem-resposta = no answer
nucleo-ia-pergunta-expirada = this question was already answered or expired
nucleo-ia-pergunta-resposta-invalida = invalid answer: { $erro }
