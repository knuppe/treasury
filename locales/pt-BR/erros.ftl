# Erros do núcleo (docs/i18n-design.md): os fixos têm a chave igual ao código (`erro-<código>`);
# `$detalhe` é o texto livre do erro, que ainda vem em pt-BR. Ficam em minúsculas, como frase de
# log: a interface põe a inicial maiúscula e o ponto. pt-BR é o idioma de origem.

erro-no_vault = o cofre não existe; crie um cofre primeiro
erro-vault_exists = o cofre já existe em { $caminho }
erro-vault_locked = o cofre está bloqueado
erro-wrong_password = senha incorreta
erro-wrong_recovery_code = código de recuperação incorreto
erro-already_open = o Treasury já está aberto em outro processo
erro-not_found = não encontrado: { $detalhe }
erro-unknown_command = comando desconhecido: { $detalhe }
erro-not_implemented = ainda não implementado: { $detalhe }
erro-forbidden = permissão negada: { $detalhe }
erro-db = erro de banco de dados: { $detalhe }
erro-migration = erro de migração: { $detalhe }
erro-io = erro de E/S: { $detalhe }
erro-json = dados inválidos: { $detalhe }
erro-crypto = erro de criptografia: { $detalhe }
erro-pending_approval = alteração proposta, aguardando aprovação do usuário
erro-cancelled = cancelado a pedido; o que já tinha vindo ficou gravado

## Respostas do MCP para clientes externos (frases completas)

erro-mcp-bloqueado = O cofre do Treasury está bloqueado. Abra o app e desbloqueie para continuar.
erro-mcp-sem-cofre = Nenhum cofre do Treasury foi criado ainda. Abra o app e crie o cofre para continuar.
erro-mcp-proibido = Operação não permitida: { $detalhe }.
erro-mcp-indisponivel = Esta consulta ainda não está disponível nesta versão do Treasury ({ $comando }).
erro-mcp-invalido = Parâmetros inválidos: { $detalhe }.
erro-mcp-nao-encontrado = Não encontrado: { $detalhe }.
erro-mcp-nao-encontrado-curto = Não encontrado.
erro-mcp-conflito = Conflito: { $detalhe }.
erro-mcp-comando = Comando desconhecido no Treasury: { $detalhe }.
erro-mcp-interno = Erro interno do Treasury ao consultar ({ $comando }). Tente de novo; se persistir, veja os logs do app.
erro-vault_damaged = o cofre pode estar danificado
