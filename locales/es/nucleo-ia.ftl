# Agente de IA (núcleo: `ai`; docs/ai-agent.md): errores y avisos de motores, adjuntos, CLIs,
# límites de gasto, propuestas, alertas y conversaciones, y los motivos del historial de la
# memoria. Los textos que van solo al modelo (instrucciones, contexto) quedan en inglés en el
# código.

## Adjuntos

nucleo-ia-anexo = adjunto
nucleo-ia-anexo-expirou = adjunto (venció o la bóveda se bloqueó; adjúntalo de nuevo)
nucleo-ia-anexo-nao-guardado = adjunto (no guardado)
nucleo-ia-anexo-pdf-senha = el PDF está protegido con contraseña: quita la contraseña y adjúntalo de nuevo
nucleo-ia-anexo-pdf-ilegivel = no se pudo leer el PDF: { $detalhe }
nucleo-ia-anexo-planilha-ilegivel = no se pudo leer la hoja de cálculo: { $detalhe }
nucleo-ia-anexo-vazio = { $nome }: archivo vacío
nucleo-ia-anexo-grande = { $nome }: archivo demasiado grande ({ $tamanho }; máx. { $maximo } por archivo)
nucleo-ia-anexo-grande-maximo = { $nome }: archivo demasiado grande (máx. { $maximo } por archivo)
nucleo-ia-anexo-imagem-grande = { $nome }: imagen demasiado grande ({ $tamanho }; máx. { $maximo } después de reducirla)
nucleo-ia-anexo-imagem-invalida = { $nome }: imagen inválida o dañada
nucleo-ia-anexo-imagem-dimensoes = { $nome }: imagen de { $largura }×{ $altura } px (máx. { $maximo } px por lado)
nucleo-ia-anexo-heic = { $nome }: imagen HEIC sin convertir; expórtala como JPEG o PNG y adjúntala de nuevo
nucleo-ia-anexo-texto-cortado = texto cortado en los primeros { $mil } mil caracteres
nucleo-ia-anexo-pdf-sem-texto = PDF sin texto extraíble (¿escaneado?): solo lo entienden los motores que leen PDF
nucleo-ia-anexo-pdf-grande = PDF de más de { $tamanho } o { $paginas } páginas: se envía solo el texto extraído
nucleo-ia-anexo-compactado = { $nome }: tipo de archivo no aceptado (no se leen archivos comprimidos ni documentos antiguos de Word)
nucleo-ia-anexo-documento-ilegivel = { $nome }: documento ilegible
nucleo-ia-anexo-documento-sem-texto = { $nome }: documento sin texto
nucleo-ia-anexo-texto-vazio = { $nome }: archivo de texto vacío
nucleo-ia-anexo-tipo-nao-aceito = { $nome }: tipo de archivo no aceptado (imágenes PNG, JPEG, WebP, GIF o HEIC; PDF; texto, CSV, JSON, OFX, XML; hojas de cálculo y documentos de Office)
nucleo-ia-anexo-base64 = adjunto inválido (base64)
nucleo-ia-anexo-maximo = como máximo { $maximo } adjuntos por mensaje

## Motores y rutas

nucleo-ia-motor = motor
nucleo-ia-motor-id = motor { $id }
nucleo-ia-provider-desconhecido = proveedor desconocido: { $provider }
nucleo-ia-motor-desligado = motor desactivado
nucleo-ia-motor-sem-chave = sin API key registrada
nucleo-ia-motor-sem-template = sin plantilla de comando
nucleo-ia-motor-sem-modelo = sin modelo definido
nucleo-ia-endpoint-so-local = usa https:// (http solo para servidores en esta computadora)
nucleo-ia-endpoint-https = el endpoint tiene que empezar con https://
nucleo-ia-endpoint-invalido = endpoint inválido
nucleo-ia-endpoint-obrigatorio = indica el endpoint (URL base de la API)
nucleo-ia-template-vazio = la plantilla de comando está vacía
nucleo-ia-template-sem-prompt = la plantilla necesita el marcador {"{prompt}"}
nucleo-ia-template-so-cli = la plantilla de comando solo vale para CLIs
nucleo-ia-template-obrigatorio = indica la plantilla de comando de la CLI personalizada
nucleo-ia-template-colchete-sem-abrir = `]` sin `[` en la plantilla
nucleo-ia-template-colchete-sem-fechar = `[` sin `]` en la plantilla
nucleo-ia-template-aspas = comillas sin cerrar en la plantilla
nucleo-ia-nome-longo = nombre demasiado largo (máx. { $maximo })
nucleo-ia-nome-obrigatorio = indica el nombre
nucleo-ia-rota-motor-repetido = el mismo motor aparece dos veces en la lista

## CLIs

nucleo-ia-cli-concluida = terminada
nucleo-ia-cli-opencode-erro = OpenCode terminó con error
nucleo-ia-cli-prompt-argumento = la plantilla de { $cli } necesita {"{prompt}"} como argumento propio
nucleo-ia-cli-claude-stream = la plantilla de Claude Code necesita --output-format stream-json para enviar imágenes
nucleo-ia-cli-sem-imagens = { $motor } no lee imágenes
nucleo-ia-cli-nao-encontrada = { $programa } no se encontró en el PATH (¿la CLI está instalada?)
nucleo-ia-cli-nao-iniciou = no se pudo iniciar { $programa }: { $detalhe }
nucleo-ia-cli-tempo-limite = la CLI superó el tiempo límite
nucleo-ia-cli-erro = la CLI terminó con error
nucleo-ia-cli-terminou = { $programa } terminó con { $status }{ $saida }

## Agente y conversaciones

nucleo-ia-agente-nao-le-imagens = no lee imágenes ({ $nomes })
nucleo-ia-agente-nao-le-pdf = no lee el PDF sin texto ({ $nomes })
nucleo-ia-agente-contexto-objeto = el contexto de la pantalla tiene que ser un objeto JSON
nucleo-ia-agente-contexto-grande = contexto de la pantalla demasiado grande (máx. { $maximo } caracteres)
nucleo-ia-agente-sem-motor = No hay ningún motor de IA configurado para esta tarea. Agrega uno en Ajustes › IA y agente.
nucleo-ia-agente-sem-motor-erro = no hay ningún motor de IA configurado para esta tarea; agrega uno en Ajustes › IA y agente
nucleo-ia-agente-cli-sem-restricao = las CLIs no quedan limitadas a las herramientas de esta tarea
nucleo-ia-agente-precisa-api = esta tarea necesita un motor de API o un modelo local (las CLIs ven todas las herramientas)
nucleo-ia-agente-anexos-sem-motor = Ningún motor de la ruta lee estos adjuntos: { $motor } { $motivo }. Elige un motor que lea imágenes o envía sin ellos.
nucleo-ia-agente-nenhum-motor = ningún motor disponible
nucleo-ia-agente-cancelado = cancelado
nucleo-ia-agente-limite-passos = (Me detuve aquí: esta respuesta llegó al límite de pasos con herramientas.)
nucleo-ia-agente-escreva = escribe el mensaje
nucleo-ia-agente-mensagem-longa = mensaje demasiado largo (máx. 20.000 caracteres)
nucleo-ia-agente-nao-iniciou = no se pudo iniciar el agente
nucleo-ia-agente-teste-titulo = Prueba de conexión del motor
nucleo-ia-job-expirou = tarea (¿terminó hace más de 30 minutos?)
nucleo-ia-conversa = conversación
nucleo-ia-conversa-sem-historico = no hay conversación guardada: el historial del agente está desactivado en Ajustes › Agente

## Límites de gasto y precios

nucleo-ia-preco = precio
nucleo-ia-preco-modelo-obrigatorio = indica el modelo (o * para el predeterminado del proveedor)
nucleo-ia-preco-negativo = los precios no pueden ser negativos
nucleo-ia-limite-so-api = los límites de gasto valen para proveedores de API (las CLIs y los modelos locales no tienen costo en la app)
nucleo-ia-limite-negativo = los límites no pueden ser negativos
nucleo-ia-limite-diario = se alcanzó el límite de gasto diario de { $provider } ({ $gasto } de { $limite })
nucleo-ia-limite-mensal = se alcanzó el límite de gasto mensual de { $provider } ({ $gasto } de { $limite })
nucleo-ia-uso-periodo = `from` no puede ser posterior a `to`

## Claves de API

nucleo-ia-chave-obrigatoria = indica la clave
nucleo-ia-chave-invalida = clave inválida
nucleo-ia-chave-documento-grande = documento demasiado grande para la bóveda
nucleo-ia-chave-nao-abre = la clave registrada no abre con la clave actual de la bóveda; regístrala de nuevo

## Propuestas, alertas, rutinas y ajustes

nucleo-ia-proposta-id = propuesta { $id }
nucleo-ia-proposta-selecione = selecciona al menos una propuesta
nucleo-ia-proposta-nao-pendente = la propuesta { $id } no está pendiente
nucleo-ia-proposta-retirada = Retirada: { $motivo } (“{ $alteracao }”) antes de la aprobación.
nucleo-ia-proposta-retirada-usuario = cambiaste los mismos datos
nucleo-ia-proposta-retirada-importacao = una importación cambió los mismos datos
nucleo-ia-proposta-retirada-agente = otro cambio del agente, ya aplicado, cambió los mismos datos
nucleo-ia-proposta-retirada-outra = otro cambio modificó los mismos datos
nucleo-ia-alerta = alerta
nucleo-ia-alerta-id = alerta { $id }
nucleo-ia-alerta-selecione = selecciona al menos una alerta
nucleo-ia-rotina-rodando = { $rotina } ya se está ejecutando
nucleo-ia-retencao = retención de las conversaciones: elige 30, 90, 180 o 365 días, o nunca (0)

## Memoria del gestor y de procedimientos (motivos del historial)

nucleo-ia-memoria-gestor = Memoria del gestor
nucleo-ia-memoria-gestor-removida = Memoria del gestor eliminada
nucleo-ia-memoria-nota = nota
nucleo-ia-memoria-nota-tamanho = la nota tiene que tener entre 1 y 2.000 caracteres
nucleo-ia-memoria-nota-nova = Memoria de procedimientos: nota nueva
nucleo-ia-memoria-nota-atualizada = Memoria de procedimientos: nota actualizada
nucleo-ia-memoria-nota-apagada = Memoria de procedimientos: nota eliminada

## Proveedores de API

nucleo-ia-provider-rede = falla de red: { $detalhe }
nucleo-ia-provider-respondeu = { $provider } respondió { $status }: { $mensagem }
nucleo-ia-provider-conexao = conexión interrumpida: { $detalhe }
nucleo-ia-provider-resposta-invalida = respuesta inválida del proveedor: { $detalhe }
nucleo-ia-provider-streaming = error en el streaming

## MCPs extra (ai.mcp_extra.*)
nucleo-mcpx-erro-nao-encontrado = MCP extra no encontrado
nucleo-mcpx-erro-nome = el nombre del MCP debe tener de 2 a 24 letras minúsculas, números o _, empezar con letra y no ser "treasury"
nucleo-mcpx-erro-nome-repetido = ya existe un MCP con ese nombre en esta bóveda
nucleo-mcpx-erro-rotulo = indica una etiqueta de hasta 60 caracteres
nucleo-mcpx-erro-comando = indica el comando que inicia el servidor MCP
nucleo-mcpx-erro-argumentos = argumentos inválidos (hasta 64, sin saltos de línea)
nucleo-mcpx-erro-url = la dirección debe empezar con https:// (o http://localhost para un servidor local)
nucleo-mcpx-erro-variavel = variable inválida o repetida: { $nome }
nucleo-mcpx-erro-cabecalho = encabezado inválido o repetido: { $nome }
nucleo-mcpx-erro-cabecalho-valor = un valor de encabezado tiene caracteres inválidos
nucleo-mcpx-erro-segredo-vazio = indica el valor secreto de { $nome }
nucleo-mcpx-erro-limite = como máximo { $max } MCPs extra por bóveda
nucleo-mcpx-erro-conexao = no se pudo hablar con el MCP { $servidor }: { $erro }
nucleo-mcpx-erro-tempo = el servidor no respondió a tiempo
nucleo-mcpx-erro-pedido = esa solicitud de confirmación ya se respondió o expiró
nucleo-mcpx-aviso-sem-conexao = El MCP { $servidor } no se conectó; el agente sigue sin él.
nucleo-mcpx-recusada = rechazada
nucleo-ia-cli-tempo-limite-parcial = ⏱ La respuesta se cortó en el tiempo límite de { $minutos } min. Arriba está lo que el agente hizo hasta ahí; lo que grabó o propuso con las herramientas de Treasury está guardado. Para continuar, pídele que siga donde se quedó (o aumenta el tiempo en Ajustes › IA).
nucleo-ia-tempo-cli = elige 15, 30, 60 o 120 minutos

## Preguntas del agente (ask_user)
nucleo-ia-pergunta-sem-resposta = sin respuesta
nucleo-ia-pergunta-expirada = esa pregunta ya se respondió o expiró
nucleo-ia-pergunta-resposta-invalida = respuesta inválida: { $erro }
