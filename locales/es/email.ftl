# Importación por correo: la pestaña Correo de Movimientos, las cuentas (buzón IMAP o carpeta
# local), las reglas y la auditoría (docs/email-ingest.md, docs/i18n-design.md).
# Revisión: borrador por IA, sin revisión nativa.
# Quedan igual en cualquier idioma: la palabra `membros` en las listas de remitentes y los grupos
# `valor`, `descricao` y `data` del modelo del cuerpo (el núcleo reconoce esos nombres), INBOX y
# las marcas \Junk, \Trash… del servidor.

## Tipos de pieza

email-tipo-extrato = Extracto
email-tipo-fatura-pdf = Resumen de tarjeta en PDF
email-tipo-nota = Nota de corretaje
email-tipo-b3 = Informe de B3
email-tipo-previdencia = Extracto del plan de pensión
email-tipo-holerite = Recibo de nómina
email-tipo-cupom = Cupom fiscal (ticket)
email-tipo-imagem = Imagen
email-tipo-compra = Aviso de compra
email-tipo-pix = Aviso de Pix
email-tipo-modelo = Por el modelo de la regla
email-tipo-desconhecido = No reconocido

## Estado del mensaje y de la pieza

email-status-revisar = Por revisar
email-status-importada = Importado
email-status-repetida = Repetido
email-status-falhou = Falló
email-status-nao-reconhecida = No reconocido
email-status-ignorada = Ignorado
email-status-ia-lendo = IA leyendo

## Qué hace la regla y la IA

email-acao-revisar = Revisar
email-acao-auto = Aplicar automáticamente
email-acao-ignorar = Ignorar
email-ia-nunca = Nunca
email-ia-a-pedido = A pedido
email-ia-auto = Automática (hasta 10 por día)

## Retención (borrar del proveedor)

email-retencao-nunca = Nunca borrar
email-retencao-sucesso = Después de importado (o repetido)
email-retencao-dias = Después de algunos días
email-retencao-ambos = Lo que ocurra primero
email-estado-desligado = No borra nada
email-estado-teste = Modo de prueba
email-estado-confirmacao = Esperando confirmación
email-estado-ativo = Borrando

## Auditoría

email-auditoria-busca = Búsqueda
email-auditoria-recebida = Recibido
email-auditoria-importada = Importado
email-auditoria-repetida = Repetido
email-auditoria-apagada = Borrado
email-auditoria-apagaria = Prueba: borraría
email-auditoria-nao-apagou = No se borró
email-auditoria-ia = IA
email-auditoria-revisao = Revisión
email-auditoria-erro = Error

## Cola del correo

email-fila = Cola del correo
email-mensagens = Mensajes
email-para-revisar = Por revisar
email-tudo = Todo
email-vazio-kicker = Importación por correo
email-vazio-titulo = Recibe extractos, resúmenes y tickets por correo
email-vazio-descricao = Crea un buzón solo para Treasury (reenvíale los correos del banco) o elige una carpeta de tu computadora donde guardas adjuntos y mensajes .eml. Lo que pase los filtros llega aquí; no se borra nada sin que lo actives y lo confirmes.
email-configurar = Configurar el correo
email-nada-revisar = Nada esperando revisión. Las cuentas buscan solas en cada intervalo, con la app abierta y la bóveda desbloqueada.
email-nenhuma-mensagem = Todavía no hay mensajes.

email-mensagem = Mensaje
email-sem-assunto = (sin asunto)
email-encaminhada-por = { $original } · reenviado por { $remetente }
email-pasta-principal = Carpeta principal
email-pasta = Carpeta { $pasta }
email-regra-nome = regla "{ $regra }"
email-sem-regra = sin regla (predeterminado de la cuenta)
email-autenticado = Remitente autenticado
email-nao-autenticado = Remitente no autenticado
email-ignorar-mensagem = Ignorar el mensaje

## Piezas del mensaje

email-cartao-final = tarjeta terminada en { $final }
email-ia-propostas = { $n ->
        [one] { $n } propuesta en la cola de aprobación del agente
       *[other] { $n } propuestas en la cola de aprobación del agente
    }
email-conta-lancamento = Cuenta del movimiento
email-escolha-conta = Elige la cuenta
email-lancar = Registrar
email-importar-cupom = Importar ticket
email-importar = Importar
email-abrir-importador = Abrir en el importador
email-pedir-ia = Pedir a la IA
email-anexar-agente = Adjuntar al agente
email-ja-importei = Ya lo importé
email-ignorar = Ignorar
email-importado = Importado
email-ja-estava = Ya estaba en Treasury
email-ficou-revisao = Quedó en revisión
email-erro-abrir = No se pudo abrir la pieza
email-erro-anexar = No se pudo adjuntar
# Texto que va listo al agente de IA.
email-agente-chegou = Llegó por correo (de { $remetente }).

## Modo de prueba del borrado

email-teste-ate = Modo de prueba del borrado hasta el { $data }: no se borra nada, solo se registra.
email-teste-acabou = El modo de prueba terminó. El borrado solo empieza después de tu confirmación; hasta entonces, sigue en prueba.
email-teriam-saido = { $n ->
        [one] { $n } mensaje habría salido (míralo en la auditoría).
       *[other] { $n } mensajes habrían salido (míralos en la auditoría).
    }
email-nada-teria-saido = Nada habría salido hasta ahora.
email-confirmar-exclusao = Confirmar el borrado (no se puede deshacer)

## Cuentas

email-contas = Cuentas
email-nova-conta = Nueva cuenta
email-pasta-local = Carpeta local
email-conta-nome = Cuenta { $nome }
email-desligada = desactivada
email-a-cada = cada { $min } min
email-n-para-revisar = { $n } por revisar
email-buscando = Buscando ahora…
email-ultima-busca = Última búsqueda { $quando }: { $resultado }
email-busca-resumo = { $novas ->
        [one] { $novas } nuevo
       *[other] { $novas } nuevos
    }, { $fila } en cola, { $recusadas ->
        [one] { $recusadas } rechazado
       *[other] { $recusadas } rechazados
    }, { $aplicadas ->
        [one] { $aplicadas } aplicado automáticamente
       *[other] { $aplicadas } aplicados automáticamente
    }
email-ainda-nao-buscou = Todavía no buscó.
email-cadastre-senha = Registra la contraseña de aplicación (Editar).
email-backup-restaurado = Copia de seguridad restaurada: prueba la conexión antes de volver a buscar.
email-buscar-agora = Buscar ahora
email-testar = Probar
email-conexao-ok = Conexión funcionando
email-n-arquivos-pasta = { $n ->
        [one] { $n } archivo en la carpeta.
       *[other] { $n } archivos en la carpeta.
    }
email-teste-le = Lee { $pastas }; para borrar, { $como }.
email-nenhuma-pasta = ninguna carpeta
email-apagar-conta = ¿Eliminar la cuenta?
email-apagar-conta-descricao = La cola y la auditoría de esta cuenta salen de Treasury. No se borra nada en el proveedor ni en la carpeta, y lo que ya se importó se queda.

## Reglas

email-regras = Reglas
email-nova-regra = Nueva regla
email-regras-tabela = Reglas del correo
email-todas-contas = Todas las cuentas
email-col-ordem = Orden
email-col-regra = Regla
email-col-conta = Cuenta
email-col-acao = Acción
email-col-ia = IA
email-col-acoes = Acciones
email-regra-de = De: { $remetentes }
email-qualquer-remetente = Cualquier remitente
email-regra-assunto = asunto: { $termos }
email-sem-regras = Sin reglas, el buzón IMAP ignora todo (la carpeta local lo manda a revisión). Crea una regla por banco o tienda: remitente, asunto y qué hacer.

## Auditoría (tarjeta)

email-auditoria = Auditoría
email-auditoria-tabela = Auditoría del correo
email-col-quando = Cuándo
email-col-o-que = Qué
email-col-detalhe = Detalle
email-nada-registrado = Todavía no hay nada registrado.

## Cuenta de importación por correo (ventana)

email-conta-nome-padrao = Buzón de la familia
email-conta-salva = Cuenta guardada
email-conta-titulo = Cuenta: { $nome }
email-conta-nova = Nueva cuenta de importación por correo
email-conta-descricao = Recomendado: un buzón solo para Treasury, alimentado por el reenvío automático de los correos del banco. Solo funciona con la app abierta y la bóveda desbloqueada; no se envía nada (sin SMTP).
email-salvar-testar = Guardar y probar
email-tipo-conta = Tipo de cuenta
email-tipo-imap = Buzón de correo (IMAP)
email-tipo-pasta = Carpeta de la computadora
email-nome = Nombre
email-pasta-vigiada = Carpeta vigilada
email-pasta-vigiada-dica = Guarda ahí adjuntos, extractos y mensajes .eml (cada subcarpeta es una carpeta de los filtros). Lo que ya se procesó puede ir a Processados.
email-pasta-exemplo = /Users/tu/Documents/Treasury correo
email-escolher = Elegir…
email-endereco = Dirección del buzón
email-endereco-dica = Solo se consulta el dominio (nunca la dirección).
email-endereco-exemplo = familia.treasury@gmail.com
email-procurar-config = Buscar la configuración
email-provedor = Proveedor
email-provedor-manual = Otro (manual)
email-servidor = Servidor IMAP
email-servidor-exemplo = imap.ejemplo.com
email-porta = Puerto
email-onde-criar = Dónde crearla
email-seguranca = Seguridad
email-seguranca-dica = Salvo Proton Bridge en la misma computadora, solo TLS (puerto 993).
email-starttls = STARTTLS (solo Bridge local)
email-usuario = Usuario
email-senha-app = Contraseña de aplicación
email-senha-cadastrada = Registrada ({ $dica }); déjala vacía para mantenerla.
email-senha-cifrada = Queda cifrada en la bóveda y nunca vuelve a aparecer.
email-senha-aviso = Treasury va a leer las carpetas que elijas usando esta contraseña de aplicación. La contraseña queda en la bóveda.
email-certificado = Certificado de Bridge (cert.pem)
email-certificado-fixado = Certificado fijado: solo se acepta este.
email-certificado-onde = Exportado en Bridge › Configuración › Configuración avanzada › Exportar certificados TLS.
email-que-caixa = Qué buzón es
email-caixa-dedicada = Solo para Treasury (recomendado)
email-caixa-principal = El buzón principal de alguien
email-caixa-principal-aviso = En el buzón principal no se lee ninguna carpeta hasta que elijas, y solo entra lo que atrapen las reglas.
email-gmail-lixeira-aviso = En Gmail, mandar a la papelera quita el mensaje de todas las etiquetas.

email-pastas-remetentes = Carpetas y remitentes
email-pastas-lidas = Carpetas leídas
email-pastas-lidas-dica-pasta = INBOX es la raíz de la carpeta; subcarpetas por la ruta (Bancos/*).
email-pastas-lidas-dica-imap = Nombres o comodín (Bancos/*). El buzón exclusivo lee solo la INBOX.
email-pastas-bloqueadas = Carpetas bloqueadas
email-pastas-bloqueadas-dica = Por las marcas del servidor (\Junk, \Trash…), nunca por el nombre. El bloqueo gana sobre el permiso.
email-remetentes-permitidos = Remitentes permitidos (todas las reglas)
email-remetentes-permitidos-dica = Vacío: cualquiera. Dirección, dominio (banco.com.br), subdominios (*.itau.com.br), comodín, /regex/ o membros (las direcciones de la familia).
email-remetentes-bloqueados = Remitentes bloqueados
email-enderecos-familia = Direcciones de la familia
email-enderecos-familia-dica = Quién reenvía al buzón (el patrón membros de las reglas). Un mensaje reenviado por uno de ellos solo se aplica automáticamente si el remitente original está en la regla.
email-sem-regra-case = Cuando ninguna regla coincide
email-padrao-ignorar = Ignorar (predeterminado; solo se cuenta)
email-padrao-revisao = Mandar a revisión

email-buscas-retencao = Búsquedas y retención
email-intervalo = Buscar cada (minutos)
email-intervalo-dica = De 5 a 120.
email-apagar-provedor = Borrar del proveedor
email-carencia-importada = Plazo de gracia después de importado (días)
email-quantos-dias = Después de cuántos días
email-quantos-dias-dica = También vale para lo que todavía espera revisión o no se reconoció (la copia queda en la bóveda).
email-arquivo-destino = Qué hacer con el archivo
email-mover-processados = Mover a Processados
email-mandar-lixeira = Mandar a la Papelera
email-purgar-gmail = Borrar también de la papelera de Gmail
email-purgar-gmail-dica = Desactivado: el mensaje queda 30 días en la papelera antes de desaparecer.
email-retencao-aviso = Primero, 7 días de modo de prueba: Treasury solo registra lo que habría borrado. Después, solo borra con tu confirmación. Nunca borra sin regla, fuera de las carpetas leídas ni sin verificar la copia guardada; el borrado no se puede deshacer.
email-guardar-original = Guardar el mensaje original en la bóveda
email-guardar-original-dica = Recomendado: antes de borrar del proveedor, Treasury verifica que la copia esté completa.
email-conectou = Conectado. Lee: { $pastas }.
email-conectou-nenhuma = ninguna carpeta (revisa las carpetas leídas)
email-para-apagar = Para borrar, { $como }.

## Regla del correo (ventana)

email-regra-salva = Regla guardada
email-regra-titulo = Regla: { $nome }
email-regra-nova = Nueva regla del correo
email-regra-descricao = El orden es siempre: carpetas, bloqueos de la cuenta y las reglas en orden (la primera que coincida decide). El asunto se compara sin distinguir mayúsculas ni acentos, ya sin Fwd:/Enc:/RES:.
email-regra-nome-exemplo = Resúmenes de Nubank
email-regra-conta = Cuenta
email-regra-ordem = Orden
email-regra-ligada = Regla activada
email-regra-de-label = De (remitentes)
email-regra-de-dica = nombre@banco.com.br, banco.com.br, *.itau.com.br, fatura*@*.com.br, /regex/ o membros. Reenviado: vale el remitente original.
email-regra-bloqueados = Remitentes bloqueados en esta regla
email-assunto-contem = El asunto contiene
email-assunto-contem-dica = Términos separados por coma.
email-assunto-contem-exemplo = resumen, extracto
email-assunto-nao-contem = El asunto no contiene
email-assunto-nao-contem-exemplo = promoción
email-todos-termos = Todos los términos (predeterminado: cualquiera)
email-assunto-regex = Asunto por expresión regular
email-assunto-regex-dica = Opcional; el error dice dónde está el problema.
email-avancadas = Carpetas y condiciones avanzadas
email-so-pastas = Solo en estas carpetas
email-so-pastas-dica = Vacío: todas las leídas.
email-avancadas-combinadas = Esta regla tiene condiciones avanzadas combinadas (Y/O/NO) que quedan como están.
email-anexo-nome = Tiene un adjunto llamado
email-anexo-nome-exemplo = *.pdf, resumen*
email-corpo-contem = El cuerpo contiene
email-ate-kb = Hasta (KB)
email-o-que-fazer = Qué hacer
email-acao-regra = Acción de la regla
email-auto-aviso = Automático solo con un lector determinista (OFX, cupom fiscal, aviso de compra o de Pix), un remitente autenticado por el proveedor y listado arriba, y una vista previa sin avisos. El resto va a revisión. Lo que lee la IA nunca entra solo.
email-ia = IA
email-ia-dica = La IA recibe solo la pieza, como dato de terceros, y solo las herramientas de vista previa e importación; el resultado siempre es una propuesta.
email-retencao = Retención
email-retencao-conta = La de la cuenta
email-carencia = Plazo de gracia (días)
email-depois-de = Después de (días)
email-modelo-corpo = Modelo del cuerpo (aviso que Treasury no lee)
email-modelo-expressao = Expresión con los grupos valor, descricao y data
email-modelo-exemplo = Ej.: { $exemplo }
email-modelo-tipo = Tipo
email-modelo-saida = Salida
email-modelo-entrada = Entrada
email-modelo-conta = Cuenta de los movimientos
email-modelo-escolher-revisao = Elegir en la revisión
email-testar-exemplo = Probar con un ejemplo
email-remetente = Remitente
email-assunto = Asunto
email-assunto-exemplo = Fwd: Llegó tu resumen
email-exemplo-passa = Ejemplo: pasa ({ $motivo })
email-exemplo-nao-passa = Ejemplo: no pasa ({ $motivo })
email-teste-recentes = De los { $total } mensajes recientes, esta regla atraparía { $pegaria }.
email-testar-nada = Probar no importa ni borra nada.

email-pasta-vigiar = Carpeta que Treasury va a vigilar
