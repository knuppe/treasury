# Textos do núcleo que não são de uma área (docs/i18n-design.md, "Textos do núcleo"): erros
# genéricos dos comandos e do cofre (`api`), banco de dados (`storage`), datas e valores
# digitados (`time`, `money`), leitura de PDF (`pdf`) e layouts de arquivos importados
# (`layouts`). Erros em minúsculas, como frase de log (a interface põe a inicial maiúscula).
# pt-BR é o idioma de origem.

## Datas, valores, PDF e banco de dados

nucleo-base-erro-data = data inválida: { $valor }
nucleo-base-erro-data-improvavel = data fora do intervalo aceito: { $valor } (confira o ano)
nucleo-base-erro-mes = mês inválido (use AAAA-MM): { $valor }
nucleo-base-erro-moeda = moeda inválida: { $valor }
nucleo-base-erro-valor = valor inválido: { $valor }
nucleo-base-erro-casas-decimais = valor com casas decimais demais: { $valor }
nucleo-base-erro-pdf = não foi possível ler o PDF
nucleo-base-erro-esquema-novo = este cofre foi atualizado por uma versão mais nova do Treasury (esquema { $esquema }; este app conhece até { $conhecido }). Atualize o app, ou restaure um backup feito por esta versão.
nucleo-base-erro-backup-esquema-novo = este backup foi feito por uma versão mais nova do Treasury (esquema { $esquema }; este app conhece até { $conhecido }). Atualize o app para restaurá-lo; o cofre atual não foi alterado.
nucleo-base-erro-esquema-antigo = este cofre foi gravado por uma versão do Treasury anterior à 0.9.38. Abra-o uma vez na versão 0.9.38, que o atualiza, e depois nesta.
nucleo-base-erro-esquema-diferente = o esquema deste cofre não confere com o da versão 1.0 ({ $objeto } diferente ou faltando). Nada foi alterado; restaure um backup ou abra-o na versão 0.9.38.
nucleo-base-erro-banco-leitura = banco aberto em modo leitura

## Comandos e cofre (`api`)

nucleo-base-erro-cliente-leitura = cliente somente leitura
nucleo-base-erro-nao-pela-ia = { $comando } não pode ser feito pela IA
nucleo-base-erro-comando-leitura = { $comando } não está liberado para clientes somente leitura
nucleo-base-erro-bloqueie-antes = bloqueie o cofre antes de trocar
# Resposta ao agente e aos clientes MCP quando uma escrita vira proposta.
nucleo-base-proposta-criada = Proposta criada. O usuário precisa aprovar na fila de aprovação do Treasury antes de a alteração valer.
nucleo-base-erro-argumentos = argumentos inválidos: { $erro }
nucleo-base-erro-informe-id = informe o `id` do registro
nucleo-base-erro-senha-curta = a senha mestra precisa de pelo menos 10 caracteres
nucleo-base-erro-nome-familia = informe o nome da família
nucleo-base-cofre = cofre { $id }
nucleo-base-erro-cofre-so-interface = só a interface do app pode gerenciar o cofre
nucleo-base-erro-demo-com-senha = este cofre não é uma demonstração sem senha: use a senha
nucleo-base-chave-keychain = chave no keychain
nucleo-base-erro-pais = país fora da lista: { $pais }
# Autor e motivo da primeira alteração do histórico de um cofre novo.
nucleo-base-autor-criacao = criação do cofre
nucleo-base-motivo-criacao = Criação do cofre
nucleo-base-erro-backup-migracao = o backup antes da atualização do banco não passou na verificação ({ $erro }); nada foi alterado
nucleo-base-erro-cofre-aberto = o cofre já está aberto
nucleo-base-erro-backups-interface = backups são gerenciados pela interface do app
nucleo-base-erro-ajustes-interface = ajustes só pela interface do app
nucleo-base-erro-ajuste-desconhecido = ajuste desconhecido: { $ajuste }
nucleo-base-erro-bloqueio-automatico = bloqueio automático deve ficar entre 0 (desligado) e 1440 minutos

## Layouts de arquivos importados (`layouts`, docs/layouts.md)

# Do que o arquivo trata; entra nas frases abaixo ("o arquivo parece um extrato de conta").
nucleo-base-layout-extrato-conta = extrato de conta
nucleo-base-layout-fatura-cartao = fatura de cartão
nucleo-base-layout-extrato-beneficio = extrato de vale-alimentação e refeição
nucleo-base-layout-nota-corretagem = nota de corretagem
nucleo-base-layout-extrato-previdencia = extrato de previdência
nucleo-base-layout-holerite = holerite
# Emissor, versão e descrição de cada layout conhecido.
nucleo-base-layout-quanta = Quanta Previdência (e fundos com o mesmo extrato)
nucleo-base-layout-quanta-colunas = v1 (2023), texto em colunas
nucleo-base-layout-quanta-colunas-descricao = Extrato Financeiro do Participante: competência, vencimento e pagamento seguidos do histórico e de nove valores por linha
nucleo-base-layout-quanta-colado = v1 (2023), texto colado
nucleo-base-layout-quanta-colado-descricao = O mesmo extrato como o leitor de PDF do app entrega: valores colados e em ordem inversa, datas depois do histórico
nucleo-base-layout-caixa = Caixa Econômica Federal
nucleo-base-layout-caixa-periodo = Extrato por período v1 (PDF com imagem, lido no computador)
nucleo-base-layout-caixa-periodo-descricao = Extrato de conta do Internet Banking: data e hora, documento, histórico, favorecido, valor e saldo com C/D, saldo do dia
nucleo-base-layout-sinacor = Corretoras no padrão SINACOR (XP, Clear, Rico, BTG, Inter…)
nucleo-base-layout-sinacor-descricao = Nota de negociação de ações e FIIs da B3
nucleo-base-layout-generico-empresa = Genérico (qualquer empresa)
nucleo-base-layout-holerite-descricao = Holerite ou contracheque: competência, vencimentos, descontos (INSS, IRRF), líquido e FGTS
nucleo-base-layout-generico-banco = Genérico (qualquer banco)
nucleo-base-layout-fatura-descricao = Fatura de cartão em PDF: data, descrição e valor por linha, vencimento e total da fatura
nucleo-base-layout-ofx = Bancos (OFX 1.x SGML e 2.x XML)
nucleo-base-layout-ofx-descricao = Extrato ou fatura em OFX
# Nota para o agente sobre um PDF anexado: as ferramentas que leem o arquivo (nomes do MCP).
nucleo-base-layout-ferramentas = { $leitura } e { $importacao }
nucleo-base-layout-ferramentas-fatura = preview_card_statement_pdf e import_statement_file (na conta do cartão)
nucleo-base-layout-reconhecido = [Arquivo reconhecido: { $assunto } ({ $emissor } { $versao }, layout { $layout }). Para importar, use { $ferramentas } com este anexo.]
nucleo-base-layout-atencao = [Atenção: { $problema }.]
nucleo-base-layout-desconhecido = o arquivo parece um { $assunto }, mas não bate com nenhum layout conhecido ({ $conhecidos }). Pode ser uma versão nova: nada foi importado. Mande o arquivo para o layout ser acrescentado
nucleo-base-layout-outro-assunto = este arquivo é um { $assunto } ({ $descricao }), não um { $esperado }
nucleo-base-layout-nao-reconhecido = não reconheci este arquivo como { $assunto }; layouts conhecidos: { $conhecidos }. Mande o arquivo original (anexado na conversa, em `attachments`) ou o texto inteiro do documento, sem resumir
nucleo-base-erro-cofre-danificado = Essa senha abre os seus backups, mas não abre o cofre. O cofre pode estar danificado (ou a senha mudou depois desses backups). Recupere de um backup.
nucleo-base-notificacoes-so-interface = notificações só pela interface do app
