# Core errors (docs/i18n-design.md): fixed errors use the error code as the key (`erro-<code>`);
# `$detalhe` is the error's free text, still in pt-BR for now. Lowercase, like a log line: the
# interface adds the capital letter and the period.
# Revision: AI draft, not yet reviewed by a native speaker.

erro-no_vault = the vault doesn't exist; create a vault first
erro-vault_exists = the vault already exists at { $caminho }
erro-vault_locked = the vault is locked
erro-wrong_password = wrong password
erro-wrong_recovery_code = wrong recovery code
erro-already_open = Treasury is already open in another process
erro-not_found = not found: { $detalhe }
erro-unknown_command = unknown command: { $detalhe }
erro-not_implemented = not implemented yet: { $detalhe }
erro-forbidden = permission denied: { $detalhe }
erro-db = database error: { $detalhe }
erro-migration = migration error: { $detalhe }
erro-io = I/O error: { $detalhe }
erro-json = invalid data: { $detalhe }
erro-crypto = encryption error: { $detalhe }
erro-pending_approval = change proposed, waiting for the user's approval
erro-cancelled = cancelled on request; what had already arrived was kept

## MCP answers to external clients (full sentences)

erro-mcp-bloqueado = The Treasury vault is locked. Open the app and unlock it to continue.
erro-mcp-sem-cofre = No Treasury vault has been created yet. Open the app and create the vault to continue.
erro-mcp-proibido = Operation not allowed: { $detalhe }.
erro-mcp-indisponivel = This query isn't available in this version of Treasury yet ({ $comando }).
erro-mcp-invalido = Invalid parameters: { $detalhe }.
erro-mcp-nao-encontrado = Not found: { $detalhe }.
erro-mcp-nao-encontrado-curto = Not found.
erro-mcp-conflito = Conflict: { $detalhe }.
erro-mcp-comando = Unknown Treasury command: { $detalhe }.
erro-mcp-interno = Treasury internal error while querying ({ $comando }). Try again; if it keeps happening, check the app logs.
erro-vault_damaged = the vault may be damaged
