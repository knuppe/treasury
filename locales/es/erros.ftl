# Errores del núcleo (docs/i18n-design.md): los fijos usan el código como clave (`erro-<código>`);
# `$detalhe` es el texto libre del error, que por ahora sigue en pt-BR. En minúsculas, como línea
# de log: la interfaz pone la mayúscula inicial y el punto.
# Revisión: borrador por IA, sin revisión nativa.

erro-no_vault = la bóveda no existe; crea una bóveda primero
erro-vault_exists = la bóveda ya existe en { $caminho }
erro-vault_locked = la bóveda está bloqueada
erro-wrong_password = contraseña incorrecta
erro-wrong_recovery_code = código de recuperación incorrecto
erro-already_open = Treasury ya está abierto en otro proceso
erro-not_found = no encontrado: { $detalhe }
erro-unknown_command = comando desconocido: { $detalhe }
erro-not_implemented = aún no implementado: { $detalhe }
erro-forbidden = permiso denegado: { $detalhe }
erro-db = error de base de datos: { $detalhe }
erro-migration = error de migración: { $detalhe }
erro-io = error de E/S: { $detalhe }
erro-json = datos inválidos: { $detalhe }
erro-crypto = error de cifrado: { $detalhe }
erro-pending_approval = cambio propuesto, esperando la aprobación del usuario
erro-cancelled = cancelado a pedido; lo que ya había llegado quedó guardado

## Respuestas del MCP a clientes externos (frases completas)

erro-mcp-bloqueado = La bóveda de Treasury está bloqueada. Abre la app y desbloquéala para continuar.
erro-mcp-sem-cofre = Todavía no se creó ninguna bóveda de Treasury. Abre la app y crea la bóveda para continuar.
erro-mcp-proibido = Operación no permitida: { $detalhe }.
erro-mcp-indisponivel = Esta consulta todavía no está disponible en esta versión de Treasury ({ $comando }).
erro-mcp-invalido = Parámetros inválidos: { $detalhe }.
erro-mcp-nao-encontrado = No encontrado: { $detalhe }.
erro-mcp-nao-encontrado-curto = No encontrado.
erro-mcp-conflito = Conflicto: { $detalhe }.
erro-mcp-comando = Comando desconocido en Treasury: { $detalhe }.
erro-mcp-interno = Error interno de Treasury al consultar ({ $comando }). Inténtalo de nuevo; si persiste, revisa los registros de la app.
erro-vault_damaged = la bóveda puede estar dañada
