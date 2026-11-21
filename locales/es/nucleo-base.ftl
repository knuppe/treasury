# Textos del núcleo que no son de un área (docs/i18n-design.md, "Textos do núcleo"): errores
# genéricos de los comandos y de la bóveda (`api`), base de datos (`storage`), fechas y montos
# escritos (`time`, `money`), lectura de PDF (`pdf`) y formatos de archivos importados
# (`layouts`). Errores en minúsculas, como línea de log (la interfaz pone la mayúscula inicial).
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Fechas, montos, PDF y base de datos

nucleo-base-erro-data = fecha inválida: { $valor }
nucleo-base-erro-mes = mes inválido (usa AAAA-MM): { $valor }
nucleo-base-erro-moeda = moneda inválida: { $valor }
nucleo-base-erro-valor = monto inválido: { $valor }
nucleo-base-erro-casas-decimais = monto con demasiados decimales: { $valor }
nucleo-base-erro-pdf = no se pudo leer el PDF
nucleo-base-erro-esquema-novo = esta bóveda fue actualizada por una versión más nueva de Treasury (esquema { $esquema }; esta app conoce hasta el { $conhecido }). Actualiza la app o restaura una copia de seguridad hecha por esta versión.
nucleo-base-erro-backup-esquema-novo = esta copia de seguridad fue hecha por una versión más nueva de Treasury (esquema { $esquema }; esta app conoce hasta el { $conhecido }). Actualiza la app para restaurarla; la bóveda actual no se modificó.
nucleo-base-erro-esquema-antigo = esta bóveda fue guardada por una versión de Treasury anterior a la 0.9.38. Ábrela una vez en la versión 0.9.38, que la actualiza, y después en esta.
nucleo-base-erro-esquema-diferente = el esquema de esta bóveda no coincide con el de la versión 1.0 ({ $objeto } diferente o ausente). No se cambió nada; restaura una copia de seguridad o ábrela en la versión 0.9.38.
nucleo-base-erro-banco-leitura = base de datos abierta en modo de solo lectura

## Comandos y bóveda (`api`)

nucleo-base-erro-cliente-leitura = cliente de solo lectura
nucleo-base-erro-nao-pela-ia = { $comando } no puede hacerlo la IA
nucleo-base-erro-comando-leitura = { $comando } no está permitido para clientes de solo lectura
nucleo-base-erro-bloqueie-antes = bloquea la bóveda antes de cambiar
# Respuesta al agente y a los clientes MCP cuando una escritura se vuelve propuesta.
nucleo-base-proposta-criada = Propuesta creada. El usuario debe aprobarla en la cola de aprobación de Treasury antes de que el cambio tenga efecto.
nucleo-base-erro-argumentos = argumentos inválidos: { $erro }
nucleo-base-erro-informe-id = indica el `id` del registro
nucleo-base-erro-senha-curta = la contraseña maestra necesita al menos 10 caracteres
nucleo-base-erro-nome-familia = indica el nombre de la familia
nucleo-base-cofre = bóveda { $id }
nucleo-base-erro-cofre-so-interface = solo la interfaz de la app puede administrar la bóveda
nucleo-base-erro-demo-com-senha = esta bóveda no es una demostración sin contraseña: usa la contraseña
nucleo-base-chave-keychain = clave en el keychain
nucleo-base-erro-pais = país fuera de la lista: { $pais }
# Autor y motivo del primer cambio del historial de una bóveda nueva.
nucleo-base-autor-criacao = creación de la bóveda
nucleo-base-motivo-criacao = Creación de la bóveda
nucleo-base-erro-backup-migracao = la copia de seguridad previa a la actualización de la base de datos no pasó la verificación ({ $erro }); no se cambió nada
nucleo-base-erro-cofre-aberto = la bóveda ya está abierta
nucleo-base-erro-backups-interface = las copias de seguridad se administran desde la interfaz de la app
nucleo-base-erro-ajustes-interface = ajustes solo desde la interfaz de la app
nucleo-base-erro-ajuste-desconhecido = ajuste desconocido: { $ajuste }
nucleo-base-erro-bloqueio-automatico = el bloqueo automático debe estar entre 0 (desactivado) y 1440 minutos

## Formatos de archivos importados (`layouts`, docs/layouts.md)

# De qué trata el archivo, con el artículo; entra en las frases de abajo ("el archivo parece un estado de cuenta").
nucleo-base-layout-extrato-conta = un estado de cuenta
nucleo-base-layout-fatura-cartao = un resumen de tarjeta
nucleo-base-layout-extrato-beneficio = un extracto de vales de alimentación y comida
nucleo-base-layout-nota-corretagem = una nota de corretaje
nucleo-base-layout-extrato-previdencia = un extracto de previsión
nucleo-base-layout-holerite = un recibo de sueldo
# Emisor, versión y descripción de cada formato conocido.
nucleo-base-layout-quanta = Quanta Previdência (y fondos con el mismo extracto)
nucleo-base-layout-quanta-colunas = v1 (2023), texto en columnas
nucleo-base-layout-quanta-colunas-descricao = Extracto financiero del participante: mes de devengo, vencimiento y pago seguidos de la descripción y de nueve montos por línea
nucleo-base-layout-quanta-colado = v1 (2023), texto pegado
nucleo-base-layout-quanta-colado-descricao = El mismo extracto tal como lo entrega el lector de PDF de la app: montos pegados y en orden inverso, fechas después de la descripción
nucleo-base-layout-caixa = Caixa Econômica Federal
nucleo-base-layout-caixa-periodo = Extracto por período v1 (PDF con imagen, leído en la computadora)
nucleo-base-layout-caixa-periodo-descricao = Extracto de cuenta del Internet Banking: fecha y hora, documento, concepto, beneficiario, monto y saldo con C/D, saldo del día
nucleo-base-layout-sinacor = Corredoras con el estándar SINACOR (XP, Clear, Rico, BTG, Inter…)
nucleo-base-layout-sinacor-descricao = Nota de negociación de acciones y FII de la B3
nucleo-base-layout-generico-empresa = Genérico (cualquier empresa)
nucleo-base-layout-holerite-descricao = Recibo de sueldo: mes de referencia, haberes, descuentos (INSS, IRRF), neto y FGTS
nucleo-base-layout-generico-banco = Genérico (cualquier banco)
nucleo-base-layout-fatura-descricao = Resumen de tarjeta en PDF: fecha, descripción y monto por línea, vencimiento y total del resumen
nucleo-base-layout-ofx = Bancos (OFX 1.x SGML y 2.x XML)
nucleo-base-layout-ofx-descricao = Estado de cuenta o resumen en OFX
# Nota para el agente sobre un PDF adjunto: las herramientas que leen el archivo (nombres del MCP).
nucleo-base-layout-ferramentas = { $leitura } e { $importacao }
nucleo-base-layout-ferramentas-fatura = preview_card_statement_pdf e import_statement_file (en la cuenta de la tarjeta)
nucleo-base-layout-reconhecido = [Archivo reconocido: { $assunto } ({ $emissor } { $versao }, formato { $layout }). Para importarlo, usa { $ferramentas } con este adjunto.]
nucleo-base-layout-atencao = [Atención: { $problema }.]
nucleo-base-layout-desconhecido = el archivo parece { $assunto }, pero no coincide con ningún formato conocido ({ $conhecidos }). Puede ser una versión nueva: no se importó nada. Envía el archivo para que se agregue el formato
nucleo-base-layout-outro-assunto = este archivo es { $assunto } ({ $descricao }), no { $esperado }
nucleo-base-layout-nao-reconhecido = no reconocí este archivo como { $assunto }; formatos conocidos: { $conhecidos }. Envía el archivo original (adjunto en la conversación, en `attachments`) o el texto completo del documento, sin resumir
nucleo-base-erro-cofre-danificado = Esta contraseña abre tus copias de seguridad, pero no la bóveda. La bóveda puede estar dañada (o la contraseña cambió después de esas copias). Recupera desde una copia de seguridad.
nucleo-base-notificacoes-so-interface = notificaciones solo desde la interfaz de la app
