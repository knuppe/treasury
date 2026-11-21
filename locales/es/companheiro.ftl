# Agente acompañante: el panel lateral del agente en todas las pantallas (⌘J), con el contexto
# de la pantalla, los pedidos rápidos y las propuestas aprobadas ahí mismo
# (docs/i18n-design.md). Revisión: borrador por IA, sin revisión nativa.

## Pedidos rápidos (el texto va al motor de IA)

companheiro-pedido-aqui-rotulo = ¿Qué puedo hacer aquí?
companheiro-pedido-aqui = ¿Qué puedo hacer en esta pantalla? Usa el contexto de la pantalla y los datos para sugerir los próximos pasos concretos y ofrécete a proponer los cambios.
companheiro-pedido-resumo-rotulo = Resumir esta pantalla
companheiro-pedido-resumo = Resume lo que muestra esta pantalla ahora, con los números principales, y señala lo que merece atención.
companheiro-pedido-cadastro-rotulo = ¿Dónde tienen cuentas?
companheiro-pedido-cadastro = Quiero registrar dónde tiene dinero la familia. Pregunta dónde tenemos cuentas (bancos, tarjetas de crédito, casas de bolsa, cuentas en el exterior) y, para cada una: propón el registro (create_account con institution_name; tarjetas con día de corte y de vencimiento), dime la mejor forma de traer los datos (OFX o CSV del banco, estado de cuenta en PDF, informe del Área do Investidor de B3, notas de corretaje en PDF) y dónde descargar cada archivo. No inventes saldos: pregunta.

## Panel

companheiro-rotulo = Agente acompañante
companheiro-abrir = Abrir el agente
companheiro-fechar = Cerrar el agente
companheiro-nova-conversa = Nueva conversación
companheiro-anteriores = Conversaciones anteriores
companheiro-opcoes = Opciones del agente
companheiro-largura = Ancho del panel del agente
companheiro-log = Conversación en el panel del agente
companheiro-mensagem = Mensaje para el agente (panel)
companheiro-placeholder = Pregunta o pide un registro…
companheiro-enviar-assim = Enviar así
companheiro-aviso = La IA puede equivocarse. Revisa antes de aprobar.
companheiro-aviso-envio = Esta conversación se enviará a { $provedor }. La IA puede equivocarse. Revisa antes de aprobar.
companheiro-autonomo-tag = Autónomo
companheiro-autonomo-tag-dica = Modo autónomo activado: las escrituras reversibles valen al momento. Desactívalo en Opciones del agente.
companheiro-fora-das-telas = Fuera de las pantallas

## Opciones y modo autónomo

companheiro-autonomo-ligado-dica = Las escrituras reversibles (categorizar, registrar, presupuesto) valen al momento y se pueden deshacer. Las sensibles siguen pidiendo aprobación una por una.
companheiro-autonomo-desligado-dica = Desactivado: toda escritura del agente se convierte en propuesta para que la apruebes.
companheiro-autonomo-ligado = Modo autónomo activado
companheiro-autonomo-desligado = Modo autónomo desactivado
companheiro-autonomo-ligado-texto = Las escrituras reversibles valen al momento; las sensibles van a aprobación.
companheiro-autonomo-desligado-texto = Toda escritura del agente se convierte en propuesta.
companheiro-nao-foi-possivel = No se pudo cambiar
companheiro-tela-cheia = Abrir en pantalla completa ({ $tela })

## Contexto de la pantalla

companheiro-contexto-vai = Va al motor de IA junto con cada mensaje, como dato (nunca como instrucción). Montos en centavos.
companheiro-contexto-desligado = Desactivado: solo el mensaje va al motor.
companheiro-contexto-grande = Contexto demasiado grande para enviar; solo va el mensaje.
companheiro-contexto-previa = Vista previa del contexto de la página
companheiro-contexto-ver = Ver lo que va junto con el mensaje
companheiro-contexto-pagina = Contexto de esta página: { $tela }
companheiro-enviar-contexto = Enviar contexto

## Bienvenida y sugerencias

companheiro-sugestoes = Sugerencias del agente
companheiro-como-ajudar = ¿Cómo puedo ayudar?
companheiro-vendo = Estoy viendo { $tela }. Puedo consultar, registrar y organizar; toda escritura se convierte en propuesta para que la apruebes aquí.
companheiro-pedidos-rapidos = Pedidos rápidos
companheiro-nesta-tela = En esta pantalla
companheiro-nada-pendente = Nada pendiente en esta pantalla.
companheiro-perguntar = Preguntar al agente: { $titulo }
companheiro-abrir-tela = Abrir la pantalla

## Conversaciones anteriores

companheiro-voltar = Volver a la conversación
companheiro-buscar-conversa = Buscar conversación
companheiro-buscar-titulo = Buscar por título
companheiro-nenhuma-com = Ninguna conversación con “{ $busca }”.
companheiro-fixadas = Fijadas
companheiro-conversas-fixadas = Conversaciones fijadas
companheiro-recentes = Recientes
companheiro-conversas-recentes = Conversaciones recientes
companheiro-ver-na-tela = Ver en la pantalla { $tela }

## Propuesta en la conversación

companheiro-falha-aplicar = La propuesta no se pudo aplicar
companheiro-aprovada = Propuesta aprobada
companheiro-aprovada-texto = El cambio vale ahora y se puede deshacer en el historial.
companheiro-rejeitada = Propuesta rechazada
companheiro-carregando = Cargando la propuesta…
companheiro-ver-alteracoes = Ver los cambios
companheiro-sensivel = Sensible: revisa antes de aprobar.
