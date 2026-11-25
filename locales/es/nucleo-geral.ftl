# Textos del núcleo de los módulos más pequeños (docs/i18n-design.md, "Textos do núcleo"):
# familia (miembros e instituciones), desbloqueo rápido, copias de seguridad, términos, apoyo,
# comentarios al desarrollador, bóveda de demostración, base de datos, exportación, historial
# de cambios, papelera, keychain y config.toml. Errores en minúsculas, como línea de log (la
# interfaz pone la mayúscula inicial).
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Familia: miembros e instituciones (`household`)

nucleo-geral-membro = miembro { $id }
nucleo-geral-erro-membro-repetido = ya existe un miembro llamado { $nome }
nucleo-geral-erro-arquivado-principal = un miembro archivado no puede ser el principal
nucleo-geral-erro-principal-arquivado = el miembro principal no se puede archivar; elige otro miembro principal antes
nucleo-geral-instituicao = institución { $id }
nucleo-geral-erro-codigo-instituicao = código de institución inválido: { $codigo }
nucleo-geral-erro-site = dirección del sitio inválida: { $site }
nucleo-geral-erro-instituicao-em-uso = la institución { $instituicao } la usa { $n ->
        [one] { $n } cuenta
       *[other] { $n } cuentas
    }; cambia la institución de esas cuentas antes de eliminarla
nucleo-geral-erro-bancos-incompleta = lista de bancos incompleta; se mantuvo la anterior
nucleo-geral-erro-bancos = lista de bancos del Banco Central: { $erro }
nucleo-geral-motivo-familia = Datos de la familia modificados
nucleo-geral-motivo-membro-criado = Miembro creado: { $nome }
nucleo-geral-motivo-membro-alterado = Miembro modificado
nucleo-geral-motivo-membro-arquivado = Miembro archivado
nucleo-geral-motivo-membro-reativado = Miembro reactivado
nucleo-geral-motivo-instituicao-criada = Institución creada: { $nome }
nucleo-geral-motivo-instituicao-alterada = Institución modificada
nucleo-geral-motivo-instituicao-apagada = Institución eliminada

## Desbloqueo rápido: PIN, llave física y recuperación dividida (`unlock`)

nucleo-geral-chave-fisica = llave física
nucleo-geral-arquivo-chaves = archivo de claves de la bóveda
nucleo-geral-erro-chaves-ilegivel = archivo de claves de la bóveda ilegible
nucleo-geral-erro-cifrar = falló el cifrado
nucleo-geral-erro-recuperacao-ilegivel = clave de recuperación ilegible
nucleo-geral-erro-sem-arquivo-chaves = el archivo de claves de la bóveda (treasury.db.keys) no está junto a la base de datos
nucleo-geral-erro-sem-recuperacao-dividida = la recuperación dividida no está configurada en esta bóveda
nucleo-geral-erro-chave-cofre-ilegivel = clave de la bóveda ilegible
nucleo-geral-erro-resposta-chave-fisica = respuesta de la llave física inválida
nucleo-geral-erro-pin-tamanho = el PIN debe tener de 6 a 12 dígitos
nucleo-geral-erro-pin-facil = PIN demasiado fácil (repetido o en secuencia): elige otro
nucleo-geral-erro-pin-sem-keychain = el PIN necesita el Keychain del sistema, que no está disponible aquí
nucleo-geral-erro-pin-desligado = el PIN no está activado en esta computadora
nucleo-geral-erro-pin-ilegivel-keychain = elemento del PIN ilegible en el Keychain
nucleo-geral-erro-pin-ilegivel = elemento del PIN ilegible
nucleo-geral-erro-pin-bloqueado = PIN incorrecto { $tentativas } veces: el PIN se desactivó. Entra con la contraseña maestra.
nucleo-geral-erro-pin-incorreto = PIN incorrecto. { $restam ->
        [one] Queda { $restam } intento
       *[other] Quedan { $restam } intentos
    } antes de exigir la contraseña maestra.
nucleo-geral-erro-divisao = división inválida
nucleo-geral-erro-parte-invalida = parte de recuperación inválida
nucleo-geral-erro-duas-partes = ingresa al menos dos partes
nucleo-geral-erro-partes-outro-conjunto = las partes no son del mismo conjunto (verifica que sean de esta bóveda)
nucleo-geral-erro-parte-repetida = la misma parte se ingresó dos veces
nucleo-geral-erro-partes-nao-reconstroem = las partes no reconstruyen la clave (¿alguna se escribió mal?)

## Copias de seguridad (`backup`)

nucleo-geral-backup = copia de seguridad { $id }
nucleo-geral-arquivo-backup = archivo de la copia de seguridad
# Queda guardado como el motivo por el que una copia encontrada en la carpeta aún no se verificó.
nucleo-geral-backup-encontrado = encontrada en la carpeta; verifícala antes de usarla
nucleo-geral-erro-backup-hora = la hora de la copia de seguridad debe estar entre 0 y 23
nucleo-geral-erro-backup-senha-antes = define la contraseña de copia de seguridad antes de activarla
nucleo-geral-erro-backup-senha-curta = la contraseña de copia de seguridad necesita al menos { $minimo } caracteres
nucleo-geral-erro-backup-criar = no se pudo crear la copia de seguridad: { $erro }
nucleo-geral-erro-backup-hash = el hash SHA-256 no coincide: el archivo fue modificado o está dañado
nucleo-geral-erro-backup-senha-antiga = ingresa la contraseña que se usaba cuando se hizo esta copia
nucleo-geral-erro-backup-senha = ingresa la contraseña de copia de seguridad

## Términos de uso (`terms`): qué cambió en cada versión

nucleo-geral-termos-1-0 = Primera versión de los términos.
nucleo-geral-erro-termos-mudaram = los términos cambiaron (vigente: { $versao }); léelos de nuevo antes de aceptar

## Apoyo al proyecto (`support`)

nucleo-geral-erro-pix-chave = la clave Pix debe ser una clave aleatoria
nucleo-geral-erro-pix-recebedor = nombre y ciudad del destinatario del Pix
nucleo-geral-erro-apoio-so-interface = el recordatorio de apoyo solo se cambia desde la app
nucleo-geral-erro-evento = evento desconocido: { $evento }

## Comentarios al desarrollador (`devfeedback`)

nucleo-geral-feedback = comentario { $id }
nucleo-geral-erro-feedback-vazio = ingresa `{ $campo }`
nucleo-geral-erro-feedback-longo = `{ $campo }` puede tener como máximo { $maximo } caracteres
nucleo-geral-erro-feedback-importancia = la importancia debe estar entre 1 y 1000
nucleo-geral-erro-modo-desenvolvedor = modo desarrollador desactivado
nucleo-geral-erro-feedback-so-interface = guardar los comentarios en un archivo solo se hace desde la app; usa export_dev_feedback para recibir el Markdown
nucleo-geral-erro-feedback-ja-fechado = este comentario ya está resuelto o descartado
nucleo-geral-erro-feedback-motivo = indica el motivo: qué lo resolvió y dónde (versión, commit o cambio)
nucleo-geral-feedback-resolvido-por = Resuelto por { $cliente }: { $motivo }
nucleo-geral-feedback-reforcado = Ya había un comentario abierto parecido ("{ $titulo }"): se reforzó (importancia { $importancia }) en lugar de duplicarlo.
nucleo-geral-feedback-registrado = Comentario registrado para el desarrollador (pantalla Desarrollador).
nucleo-geral-erro-feedback-itens = `{ $campo }` acepta como máximo { $maximo } elementos
nucleo-geral-erro-feedback-posicao = la posición empieza en 1 (la parte superior de la columna)
nucleo-geral-erro-feedback-prioridade = indica la importancia, la posición en la columna o ambas
nucleo-geral-erro-feedback-vinculo-proprio = un comentario no puede vincularse consigo mismo
nucleo-geral-feedback-vinculo = vínculo entre el comentario { $id } y el { $outro }
nucleo-geral-feedback-status-nota = { $status ->
    [aberto] Reabierto
    [em_andamento] En curso
    [resolvido] Resuelto
   *[descartado] Descartado
}
nucleo-geral-feedback-mesclado = Duplicado fusionado aquí: { $titulo }
nucleo-geral-feedback-duplicado-de = duplicado de "{ $titulo }"
nucleo-geral-motivo-feedback-registrar = Comentario al desarrollador: registrar
nucleo-geral-motivo-feedback-reforcar = Comentario al desarrollador: reforzar
nucleo-geral-motivo-feedback-resolver = Comentario al desarrollador: marcar como resuelto
nucleo-geral-motivo-feedback-status = Comentario al desarrollador: cambiar de columna
nucleo-geral-motivo-feedback-prioridade = Comentario al desarrollador: cambiar la prioridad
nucleo-geral-motivo-feedback-editar = Comentario al desarrollador: editar
nucleo-geral-motivo-feedback-vincular = Comentario al desarrollador: vincular
nucleo-geral-motivo-feedback-desvincular = Comentario al desarrollador: quitar el vínculo
nucleo-geral-motivo-feedback-apagar = Comentario al desarrollador: eliminar
nucleo-geral-feedback-md-contexto = Comentario `{ $id }` de Treasury · área { $area } · tipo { $tipo } · prioridad { $prioridade } ({ $reforcos } refuerzos) · { $status }.
nucleo-geral-feedback-md-problema = Problema
nucleo-geral-feedback-md-esperado-obtido = Esperado × obtenido
nucleo-geral-feedback-md-esperado = Esperado
nucleo-geral-feedback-md-obtido = Obtenido
nucleo-geral-feedback-md-passos = Cómo reproducirlo
nucleo-geral-feedback-md-proposta = Propuesta
nucleo-geral-feedback-md-aceite = Criterio de aceptación
nucleo-geral-feedback-md-sem-aceite = No informado: define una prueba que demuestre la corrección.
nucleo-geral-feedback-md-onde = Dónde cambiar
nucleo-geral-feedback-md-chamadas = Llamadas que llevaron al problema
nucleo-geral-feedback-md-relacionados = Relacionados
nucleo-geral-feedback-md-reforcos = Refuerzos y notas
nucleo-geral-feedback-md-fechar = Al terminar, ejecuta las pruebas y márcalo como resuelto en el MCP de Treasury: `resolve_dev_feedback` con `id` = `{ $id }` y el motivo (qué cambió, versión o commit).

## Bóveda de demostración (`demo`): errores del asistente y los datos de ejemplo, que nacen en
## el idioma de la instalación (las marcas y los nombres propios quedan como están)

nucleo-geral-erro-demo-meses = el historial va de 3 a 36 meses
nucleo-geral-erro-demo-adultos = ingresa uno o dos adultos
nucleo-geral-erro-demo-filhos = hasta 4 hijos
nucleo-geral-erro-demo-nome = ingresa el nombre de cada persona
nucleo-geral-erro-demo-nome-repetido = nombre repetido: { $nome }
nucleo-geral-erro-demo-renda = ingresa los ingresos de al menos un adulto
nucleo-geral-erro-demo-banco = ingresa el banco de la cuenta principal
nucleo-geral-erro-demo-valores = valores entre cero y 100 millones
nucleo-geral-erro-demo-financiamento = ingresa la cuota del crédito hipotecario
nucleo-geral-erro-demo-parcelas-carro = ingresa cuántas cuotas del auto faltan (1 a 96)
nucleo-geral-erro-demo-cofre-com-dados = la bóveda de demostración solo se genera en una bóveda sin movimientos
nucleo-geral-erro-demo-so-interface = solo la interfaz de la app genera la bóveda de demostración
nucleo-geral-erro-demo-interno = bóveda de demostración: { $erro }
nucleo-geral-erro-demo-categoria = falta la categoría predeterminada: { $caminho }
nucleo-geral-erro-demo-data = fecha inválida
nucleo-geral-motivo-demo = Bóveda de demostración generada
nucleo-geral-demo-conta = Cuenta { $nome }
nucleo-geral-demo-cartao = Tarjeta { $nome }
nucleo-geral-demo-reserva = Ahorro { $nome }
nucleo-geral-demo-corretora = Corredora XP
nucleo-geral-demo-previdencia = Plan de pensión { $nome }
nucleo-geral-demo-cliente-a = Cliente A
nucleo-geral-demo-cliente-b = Cliente B
nucleo-geral-demo-cliente-c = Cliente C
nucleo-geral-demo-projeto = Proyecto
nucleo-geral-demo-recebimento = Cobro { $nome }
nucleo-geral-demo-salario = Sueldo { $nome }
nucleo-geral-demo-13-1 = 13º salario (1.ª cuota)
nucleo-geral-demo-13-2 = 13º salario (2.ª cuota)
nucleo-geral-demo-aluguel-recebido = Alquiler recibido (departamento)
nucleo-geral-demo-aluguel = Alquiler
nucleo-geral-demo-condominio = Expensas
nucleo-geral-demo-luz = Cuenta de luz
nucleo-geral-demo-agua = Cuenta de agua
nucleo-geral-demo-internet = Internet de fibra
nucleo-geral-demo-celular = Plan de celular
nucleo-geral-demo-padaria = Panadería del barrio
nucleo-geral-demo-estacionamento = Estacionamiento
nucleo-geral-demo-ipva = IPVA y licencia del vehículo
nucleo-geral-demo-seguro-carro = Seguro del auto
nucleo-geral-demo-revisao = Servicio y cambio de aceite
nucleo-geral-demo-bilhete = Recarga de la tarjeta de transporte
nucleo-geral-demo-mensalidade = Cuota escolar ({ $nome })
nucleo-geral-demo-material = Útiles escolares ({ $nome })
nucleo-geral-demo-plano-saude = Seguro de salud
nucleo-geral-demo-restaurante = Restaurante
nucleo-geral-demo-pizzaria = Pizzería
nucleo-geral-demo-churrascaria = Parrilla
nucleo-geral-demo-teatro = Teatro
nucleo-geral-demo-parque = Parque
nucleo-geral-demo-passeio = Paseo
nucleo-geral-demo-museu = Museo
nucleo-geral-demo-viagem = Viaje de vacaciones
nucleo-geral-demo-financiamento-apartamento = Crédito hipotecario del departamento
nucleo-geral-demo-financiamento-carro = Préstamo del auto
# El nombre del contrato llega en minúsculas ("Cuota préstamo del auto").
nucleo-geral-demo-parcela = Cuota { $nome }
nucleo-geral-demo-ativo-nota = Activo de la bóveda de demostración: cotizaciones ficticias.
nucleo-geral-demo-rendimento-reserva = Rendimiento del ahorro
nucleo-geral-demo-aporte-reserva = Aporte al ahorro
nucleo-geral-demo-aporte-corretora = Aporte a la corredora
nucleo-geral-demo-aporte-previdencia = Aporte al plan de pensión
nucleo-geral-demo-meta-reserva = Fondo de emergencia

## Base de datos: tamaño, mantenimiento y depuración (`maintenance`)

nucleo-geral-grupo-historico = historial de cambios
nucleo-geral-grupo-lancamentos = movimientos
nucleo-geral-grupo-contas = cuentas y tarjetas
nucleo-geral-grupo-carteira = cartera
nucleo-geral-grupo-cotacoes = cotizaciones, índices y referencias
nucleo-geral-grupo-analise = análisis de activos (CVM, B3, SEC)
nucleo-geral-grupo-agente = agente
nucleo-geral-grupo-dividas = deudas
nucleo-geral-grupo-patrimonio = patrimonio físico
nucleo-geral-expurgo-historico = Historial de cambios antiguo
nucleo-geral-expurgo-historico-consequencia = Los cambios eliminados ya no se pueden deshacer; las propuestas pendientes se mantienen.
nucleo-geral-expurgo-auditoria = Auditoría del agente y del MCP
nucleo-geral-expurgo-auditoria-consequencia = Desaparece el registro de quién consultó o propuso qué en ese período.
nucleo-geral-expurgo-conversas = Conversaciones antiguas del agente
nucleo-geral-expurgo-conversas-consequencia = Las conversaciones no fijadas sin uso en el período (y sus adjuntos) desaparecen.
nucleo-geral-expurgo-mercado = Cachés de mercado
nucleo-geral-expurgo-mercado-consequencia = Cotizaciones intradía antiguas y la caché de Fundamentus; vuelven en la próxima consulta.
nucleo-geral-expurgo-analise = Datos del análisis de activos
nucleo-geral-expurgo-analise-consequencia = Fundamentos, cotizaciones del universo, documentos e indicadores de los bancos descargados de la CVM, la B3, la SEC y el Banco Central; la próxima recolección descarga todo de nuevo (unos 230 MB por la red).
nucleo-geral-erro-expurgo-backup = la copia de seguridad previa a la depuración no pasó la verificación ({ $erro }); no se eliminó nada
nucleo-geral-referencias-quebradas = { $n ->
        [one] { $n } referencia rota
       *[other] { $n } referencias rotas
    } entre tablas
nucleo-geral-tabela = tabla { $tabela }
nucleo-geral-ordem-chave-primaria = clave primaria
nucleo-geral-mostrando-linhas = mostrando las primeras { $limite } de { $total } filas
nucleo-geral-segredos-ocultos = las claves, los tokens y los secretos aparecen ocultos
nucleo-geral-erro-banco-so-interface = el mantenimiento y la depuración de la base de datos solo se hacen desde la app
nucleo-geral-erro-banco-leitura = base de datos abierta solo para lectura

## Exportación en formato abierto (`export`): errores, el `about` del JSON y el CSV de movimientos

nucleo-geral-erro-exportar-caminho = elige una ruta completa para el archivo
nucleo-geral-erro-exportar-nome = ingresa el nombre del archivo
nucleo-geral-erro-exportar-pasta-destino = carpeta de destino inválida
nucleo-geral-erro-exportar-pasta-inexistente = la carpeta { $pasta } no existe
nucleo-geral-erro-exportar-destino-pasta = el destino es una carpeta; ingresa el nombre del archivo
nucleo-geral-erro-exportar-pasta-treasury = elige una carpeta fuera de las carpetas de datos y de copias de seguridad de Treasury
nucleo-geral-erro-exportar-pasta = carpeta inválida
nucleo-geral-erro-csv = error al escribir el CSV: { $erro }
nucleo-geral-exportacao-descricao = Exportación completa de los datos de Treasury en formato abierto. Este archivo NO está cifrado.
nucleo-geral-exportacao-tabelas = Cada tabla es una lista de objetos con las columnas del esquema de la base de datos.
nucleo-geral-exportacao-dinheiro = Los valores monetarios son enteros en la unidad mínima de la moneda (centavos), con la moneda en la columna currency. En txn.amount, + entra y − sale de la cuenta.
nucleo-geral-exportacao-decimais = Las cantidades y tasas exactas son texto decimal.
nucleo-geral-exportacao-datas = Fechas de los movimientos en AAAA-MM-DD (sin zona horaria); instantes en UTC (RFC 3339).
nucleo-geral-exportacao-ids = Identificadores UUID v7 en texto; las referencias entre tablas usan esos ids.
nucleo-geral-exportacao-json = Columnas como rule.conditions, rule.actions y setting.value guardan JSON en texto.
nucleo-geral-exportacao-binarios = Las columnas binarias (ver blob_columns) están en base64 estándar.
nucleo-geral-exportacao-segredos = Los secretos (contraseña de copia de seguridad, claves de API) nunca se exportan.
nucleo-geral-csv-data = Fecha
nucleo-geral-csv-conta = Cuenta
nucleo-geral-csv-descricao = Descripción
nucleo-geral-csv-favorecido = Beneficiario
nucleo-geral-csv-categoria = Categoría
nucleo-geral-csv-titular = Miembro
nucleo-geral-csv-valor = Valor
nucleo-geral-csv-moeda = Moneda
nucleo-geral-csv-tipo = Tipo
nucleo-geral-csv-situacao = Estado
nucleo-geral-csv-tags = Etiquetas
nucleo-geral-csv-notas = Notas
nucleo-geral-csv-vencimento-fatura = Vencimiento del resumen
nucleo-geral-csv-parcela = Cuota
nucleo-geral-csv-divisao = División
nucleo-geral-csv-id = Id
nucleo-geral-csv-sem-categoria = Sin categoría
nucleo-geral-csv-receita = Ingreso
nucleo-geral-csv-despesa = Gasto
nucleo-geral-csv-transferencia = Transferencia
nucleo-geral-csv-pagamento-fatura = Pago de tarjeta
nucleo-geral-csv-investimento = Aporte
nucleo-geral-csv-ajuste = Ajuste
nucleo-geral-csv-previsto = Previsto
nucleo-geral-csv-efetivado = Efectuado
nucleo-geral-csv-conciliado = Conciliado

## Historial de cambios (`changes`)

nucleo-geral-conjunto = conjunto de cambios { $id }
nucleo-geral-erro-sem-motivo = todo cambio necesita un motivo
nucleo-geral-erro-aprovar = solo se pueden aprobar cambios pendientes
nucleo-geral-erro-rejeitar = solo se pueden rechazar cambios pendientes
nucleo-geral-erro-desfazer = solo se pueden deshacer cambios aplicados
nucleo-geral-erro-refazer = solo se pueden rehacer cambios deshechos
nucleo-geral-erro-alteracoes-posteriores = cambios posteriores modificaron estos datos; deshaz primero los cambios más recientes que tocan las mismas filas

## Papelera, keychain y config.toml (`trash`, `keychain`, `config`)

nucleo-geral-arquivo = archivo { $caminho }
nucleo-geral-erro-caminho-sem-nome = ruta sin nombre de archivo
nucleo-geral-erro-keychain-indisponivel = keychain no disponible en este sistema
nucleo-geral-erro-keychain-negado = acceso al keychain denegado o keychain bloqueado
nucleo-geral-erro-keychain = error del keychain: { $erro }
nucleo-geral-erro-config = config.toml inválido: { $erro }

# Recuperación desde copias de seguridad (docs/backup-recovery.md)
nucleo-geral-erro-backup-fora-da-pasta = Ese archivo no está en la carpeta de copias de esta bóveda.
nucleo-geral-erro-backup-destino-igual = Elige otro lugar para la copia: ese archivo es la propia copia de seguridad.
nucleo-geral-erro-backup-copia-diferente = La copia quedó distinta del original; inténtalo de nuevo o elige otro lugar.
nucleo-geral-erro-backup-nao-e-backup = Ese archivo no parece una copia de seguridad de Treasury (son bases cifradas que terminan en .db.enc).

## Tareas en segundo plano (`tasks`)

nucleo-geral-tarefa-nao-encontrada = tarea no encontrada (puede que ya haya terminado)
nucleo-geral-tarefa-nao-cancelavel = esta tarea no se puede cancelar
nucleo-geral-tarefa-cofre-bloqueado = interrumpida porque la bóveda se bloqueó; sigue donde se quedó cuando la desbloquees
