# Agente de IA (núcleo: `ai`; docs/ai-agent.md): erros e avisos de motores, anexos, CLIs,
# limites de gasto, propostas, alertas e conversas, e os motivos do histórico da memória. Os
# textos que vão só para o modelo (instruções, contexto) ficam em inglês no código. pt-BR é a
# origem.

## Anexos

nucleo-ia-anexo = anexo
nucleo-ia-anexo-expirou = anexo (expirou ou o cofre foi bloqueado; anexe de novo)
nucleo-ia-anexo-nao-guardado = anexo (não guardado)
nucleo-ia-anexo-pdf-senha = o PDF é protegido por senha: remova a senha e anexe de novo
nucleo-ia-anexo-pdf-ilegivel = não foi possível ler o PDF: { $detalhe }
nucleo-ia-anexo-planilha-ilegivel = não foi possível ler a planilha: { $detalhe }
nucleo-ia-anexo-vazio = { $nome }: arquivo vazio
nucleo-ia-anexo-grande = { $nome }: arquivo grande demais ({ $tamanho }; máx. { $maximo } por arquivo)
nucleo-ia-anexo-grande-maximo = { $nome }: arquivo grande demais (máx. { $maximo } por arquivo)
nucleo-ia-anexo-imagem-grande = { $nome }: imagem grande demais ({ $tamanho }; máx. { $maximo } depois de reduzida)
nucleo-ia-anexo-imagem-invalida = { $nome }: imagem inválida ou corrompida
nucleo-ia-anexo-imagem-dimensoes = { $nome }: imagem de { $largura }×{ $altura } px (máx. { $maximo } px por lado)
nucleo-ia-anexo-heic = { $nome }: imagem HEIC não convertida; exporte como JPEG ou PNG e anexe de novo
nucleo-ia-anexo-texto-cortado = texto cortado nos primeiros { $mil } mil caracteres
nucleo-ia-anexo-pdf-sem-texto = PDF sem texto extraível (escaneado?): só motores que leem PDF o entendem
nucleo-ia-anexo-pdf-grande = PDF acima de { $tamanho } ou { $paginas } páginas: vai só o texto extraído
nucleo-ia-anexo-compactado = { $nome }: tipo de arquivo não aceito (compactados e documentos antigos do Word não são lidos)
nucleo-ia-anexo-documento-ilegivel = { $nome }: documento ilegível
nucleo-ia-anexo-documento-sem-texto = { $nome }: documento sem texto
nucleo-ia-anexo-texto-vazio = { $nome }: arquivo de texto vazio
nucleo-ia-anexo-tipo-nao-aceito = { $nome }: tipo de arquivo não aceito (imagens PNG, JPEG, WebP, GIF ou HEIC; PDF; texto, CSV, JSON, OFX, XML; planilhas e documentos do Office)
nucleo-ia-anexo-base64 = anexo inválido (base64)
nucleo-ia-anexo-maximo = no máximo { $maximo } anexos por mensagem

## Motores e rotas

nucleo-ia-motor = motor
nucleo-ia-motor-id = motor { $id }
nucleo-ia-provider-desconhecido = provider desconhecido: { $provider }
nucleo-ia-motor-desligado = motor desligado
nucleo-ia-motor-sem-chave = sem API key cadastrada
nucleo-ia-motor-sem-template = sem template de comando
nucleo-ia-motor-sem-modelo = sem modelo definido
nucleo-ia-endpoint-so-local = use https:// (http só para servidores neste computador)
nucleo-ia-endpoint-https = o endpoint precisa começar com https://
nucleo-ia-endpoint-invalido = endpoint inválido
nucleo-ia-endpoint-obrigatorio = informe o endpoint (URL base da API)
nucleo-ia-template-vazio = o template de comando está vazio
nucleo-ia-template-sem-prompt = o template precisa do marcador {"{prompt}"}
nucleo-ia-template-so-cli = template de comando só vale para CLIs
nucleo-ia-template-obrigatorio = informe o template de comando da CLI personalizada
nucleo-ia-template-colchete-sem-abrir = `]` sem `[` no template
nucleo-ia-template-colchete-sem-fechar = `[` sem `]` no template
nucleo-ia-template-aspas = aspas sem fechar no template
nucleo-ia-nome-longo = nome longo demais (máx. { $maximo })
nucleo-ia-nome-obrigatorio = informe o nome
nucleo-ia-rota-motor-repetido = o mesmo motor aparece duas vezes na lista

## CLIs

nucleo-ia-cli-concluida = concluída
nucleo-ia-cli-opencode-erro = o OpenCode terminou com erro
nucleo-ia-cli-prompt-argumento = o template do { $cli } precisa de {"{prompt}"} como argumento próprio
nucleo-ia-cli-claude-stream = o template do Claude Code precisa de --output-format stream-json para enviar imagens
nucleo-ia-cli-sem-imagens = { $motor } não lê imagens
nucleo-ia-cli-nao-encontrada = { $programa } não encontrado no PATH (a CLI está instalada?)
nucleo-ia-cli-nao-iniciou = não foi possível iniciar { $programa }: { $detalhe }
nucleo-ia-cli-tempo-limite = a CLI passou do tempo-limite
nucleo-ia-cli-erro = a CLI terminou com erro
nucleo-ia-cli-terminou = { $programa } terminou com { $status }{ $saida }

## Agente e conversas

nucleo-ia-agente-nao-le-imagens = não lê imagens ({ $nomes })
nucleo-ia-agente-nao-le-pdf = não lê o PDF sem texto ({ $nomes })
nucleo-ia-agente-contexto-objeto = o contexto da tela precisa ser um objeto JSON
nucleo-ia-agente-contexto-grande = contexto da tela grande demais (máx. { $maximo } caracteres)
nucleo-ia-agente-sem-motor = Nenhum motor de IA configurado para esta tarefa. Cadastre um em Ajustes › IA e agente.
nucleo-ia-agente-sem-motor-erro = nenhum motor de IA configurado para esta tarefa; cadastre um em Ajustes › IA e agente
nucleo-ia-agente-cli-sem-restricao = as CLIs não ficam restritas às ferramentas desta tarefa
nucleo-ia-agente-precisa-api = esta tarefa precisa de um motor de API ou de um modelo local (as CLIs veem todas as ferramentas)
nucleo-ia-agente-anexos-sem-motor = Nenhum motor da rota lê estes anexos: { $motor } { $motivo }. Escolha um motor que leia imagens ou envie sem eles.
nucleo-ia-agente-nenhum-motor = nenhum motor disponível
nucleo-ia-agente-cancelado = cancelado
nucleo-ia-agente-limite-passos = (Parei aqui: o limite de passos com ferramentas desta resposta foi atingido.)
nucleo-ia-agente-escreva = escreva a mensagem
nucleo-ia-agente-mensagem-longa = mensagem longa demais (máx. 20.000 caracteres)
nucleo-ia-agente-nao-iniciou = não foi possível iniciar o agente
nucleo-ia-agente-teste-titulo = Teste de conexão do motor
nucleo-ia-job-expirou = job (encerrado há mais de 30 minutos?)
nucleo-ia-conversa = conversa
nucleo-ia-conversa-sem-historico = sem conversa gravada: o histórico do agente está desligado em Ajustes › Agente

## Limites de gasto e preços

nucleo-ia-preco = preço
nucleo-ia-preco-modelo-obrigatorio = informe o modelo (ou * para o padrão do provider)
nucleo-ia-preco-negativo = preços não podem ser negativos
nucleo-ia-limite-so-api = limites de gasto valem para providers de API (CLIs e modelos locais não têm custo no app)
nucleo-ia-limite-negativo = limites não podem ser negativos
nucleo-ia-limite-diario = limite de gasto diário de { $provider } atingido ({ $gasto } de { $limite })
nucleo-ia-limite-mensal = limite de gasto mensal de { $provider } atingido ({ $gasto } de { $limite })
nucleo-ia-uso-periodo = `from` não pode ser depois de `to`

## Chaves de API

nucleo-ia-chave-obrigatoria = informe a chave
nucleo-ia-chave-invalida = chave inválida
nucleo-ia-chave-documento-grande = documento grande demais para o cofre
nucleo-ia-chave-nao-abre = a chave cadastrada não abre com a chave atual do cofre; cadastre de novo

## Propostas, alertas, rotinas e ajustes

nucleo-ia-proposta-id = proposta { $id }
nucleo-ia-proposta-selecione = selecione ao menos uma proposta
nucleo-ia-proposta-nao-pendente = a proposta { $id } não está pendente
nucleo-ia-proposta-retirada = Retirada: { $motivo } (“{ $alteracao }”) antes da aprovação.
nucleo-ia-proposta-retirada-usuario = você mudou os mesmos dados
nucleo-ia-proposta-retirada-importacao = uma importação mudou os mesmos dados
nucleo-ia-proposta-retirada-agente = outra alteração do agente, já aplicada, mudou os mesmos dados
nucleo-ia-proposta-retirada-outra = outra alteração mudou os mesmos dados
nucleo-ia-alerta = alerta
nucleo-ia-alerta-id = alerta { $id }
nucleo-ia-alerta-selecione = selecione ao menos um alerta
nucleo-ia-rotina-rodando = { $rotina } já está rodando
nucleo-ia-retencao = retenção das conversas: escolha 30, 90, 180 ou 365 dias, ou nunca (0)

## Memória do gestor e de procedimentos (motivos do histórico)

nucleo-ia-memoria-gestor = Memória do gestor
nucleo-ia-memoria-gestor-removida = Memória do gestor removida
nucleo-ia-memoria-nota = nota
nucleo-ia-memoria-nota-tamanho = a nota precisa ter entre 1 e 2.000 caracteres
nucleo-ia-memoria-nota-nova = Memória de procedimentos: nota nova
nucleo-ia-memoria-nota-atualizada = Memória de procedimentos: nota atualizada
nucleo-ia-memoria-nota-apagada = Memória de procedimentos: nota apagada

## Provedores de API

nucleo-ia-provider-rede = falha de rede: { $detalhe }
nucleo-ia-provider-respondeu = { $provider } respondeu { $status }: { $mensagem }
nucleo-ia-provider-conexao = conexão interrompida: { $detalhe }
nucleo-ia-provider-resposta-invalida = resposta inválida do provider: { $detalhe }
nucleo-ia-provider-streaming = erro no streaming

## MCPs extras (ai.mcp_extra.*)
nucleo-mcpx-erro-nao-encontrado = MCP extra não encontrado
nucleo-mcpx-erro-nome = o nome do MCP precisa ter de 2 a 24 letras minúsculas, números ou _, começar por letra e não ser "treasury"
nucleo-mcpx-erro-nome-repetido = já existe um MCP com esse nome neste cofre
nucleo-mcpx-erro-rotulo = informe um rótulo de até 60 caracteres
nucleo-mcpx-erro-comando = informe o comando que inicia o servidor MCP
nucleo-mcpx-erro-argumentos = argumentos inválidos (até 64, sem quebras)
nucleo-mcpx-erro-url = o endereço precisa começar com https:// (ou http://localhost para um servidor local)
nucleo-mcpx-erro-variavel = variável inválida ou repetida: { $nome }
nucleo-mcpx-erro-cabecalho = cabeçalho inválido ou repetido: { $nome }
nucleo-mcpx-erro-cabecalho-valor = um valor de cabeçalho tem caracteres inválidos
nucleo-mcpx-erro-segredo-vazio = informe o valor secreto de { $nome }
nucleo-mcpx-erro-limite = no máximo { $max } MCPs extras por cofre
nucleo-mcpx-erro-conexao = não foi possível falar com o MCP { $servidor }: { $erro }
nucleo-mcpx-erro-tempo = o servidor não respondeu a tempo
nucleo-mcpx-erro-pedido = esse pedido de confirmação já foi respondido ou expirou
nucleo-mcpx-aviso-sem-conexao = O MCP { $servidor } não conectou; o agente segue sem ele.
nucleo-mcpx-recusada = recusada
nucleo-ia-cli-tempo-limite-parcial = ⏱ A resposta foi interrompida no tempo-limite de { $minutos } min. Acima está o que o agente fez até ali; o que ele gravou ou propôs pelas ferramentas do Treasury está guardado. Para continuar, peça para seguir de onde parou (ou aumente o tempo em Ajustes › IA).
nucleo-ia-tempo-cli = escolha 15, 30, 60 ou 120 minutos

## Perguntas do agente (ask_user)
nucleo-ia-pergunta-sem-resposta = sem resposta
nucleo-ia-pergunta-expirada = essa pergunta já foi respondida ou expirou
nucleo-ia-pergunta-resposta-invalida = resposta inválida: { $erro }
