# Textos do núcleo dos módulos menores (docs/i18n-design.md, "Textos do núcleo"): família
# (membros e instituições), desbloqueio rápido, backups, termos, apoio, feedback ao
# desenvolvedor, cofre de demonstração, banco de dados, exportação, histórico de alterações,
# lixeira, keychain e config.toml. Erros em minúsculas, como frase de log (a interface põe a
# inicial maiúscula). pt-BR é o idioma de origem.

## Família: membros e instituições (`household`)

nucleo-geral-membro = membro { $id }
nucleo-geral-erro-membro-repetido = já existe um membro chamado { $nome }
nucleo-geral-erro-arquivado-principal = um membro arquivado não pode ser o principal
nucleo-geral-erro-principal-arquivado = o membro principal não pode ser arquivado; escolha outro membro principal antes
nucleo-geral-instituicao = instituição { $id }
nucleo-geral-erro-codigo-instituicao = código da instituição inválido: { $codigo }
nucleo-geral-erro-site = endereço do site inválido: { $site }
nucleo-geral-erro-instituicao-em-uso = a instituição { $instituicao } é usada por { $n } conta(s); troque a instituição dessas contas antes de apagar
nucleo-geral-erro-bancos-incompleta = lista de bancos incompleta; mantida a anterior
nucleo-geral-erro-bancos = lista de bancos do Banco Central: { $erro }
nucleo-geral-motivo-familia = Dados da família alterados
nucleo-geral-motivo-membro-criado = Membro criado: { $nome }
nucleo-geral-motivo-membro-alterado = Membro alterado
nucleo-geral-motivo-membro-arquivado = Membro arquivado
nucleo-geral-motivo-membro-reativado = Membro reativado
nucleo-geral-motivo-instituicao-criada = Instituição criada: { $nome }
nucleo-geral-motivo-instituicao-alterada = Instituição alterada
nucleo-geral-motivo-instituicao-apagada = Instituição apagada

## Desbloqueio rápido: PIN, chave física e recuperação dividida (`unlock`)

nucleo-geral-chave-fisica = chave física
nucleo-geral-arquivo-chaves = arquivo de chaves do cofre
nucleo-geral-erro-chaves-ilegivel = arquivo de chaves do cofre ilegível
nucleo-geral-erro-cifrar = falha ao cifrar
nucleo-geral-erro-recuperacao-ilegivel = chave de recuperação ilegível
nucleo-geral-erro-sem-arquivo-chaves = o arquivo de chaves do cofre (treasury.db.keys) não está ao lado do banco
nucleo-geral-erro-sem-recuperacao-dividida = a recuperação dividida não foi configurada neste cofre
nucleo-geral-erro-chave-cofre-ilegivel = chave do cofre ilegível
nucleo-geral-erro-resposta-chave-fisica = resposta da chave física inválida
nucleo-geral-erro-pin-tamanho = o PIN precisa ter de 6 a 12 dígitos
nucleo-geral-erro-pin-facil = PIN fácil demais (repetido ou em sequência): escolha outro
nucleo-geral-erro-pin-sem-keychain = o PIN precisa do Keychain do sistema, indisponível aqui
nucleo-geral-erro-pin-desligado = o PIN não está ligado neste computador
nucleo-geral-erro-pin-ilegivel-keychain = item do PIN ilegível no Keychain
nucleo-geral-erro-pin-ilegivel = item do PIN ilegível
nucleo-geral-erro-pin-bloqueado = PIN errado { $tentativas } vezes: o PIN foi desligado. Entre com a senha mestra.
nucleo-geral-erro-pin-incorreto = PIN incorreto. Restam { $restam } tentativa(s) antes de exigir a senha mestra.
nucleo-geral-erro-divisao = divisão inválida
nucleo-geral-erro-parte-invalida = parte de recuperação inválida
nucleo-geral-erro-duas-partes = informe pelo menos duas partes
nucleo-geral-erro-partes-outro-conjunto = as partes não são do mesmo conjunto (confira se são deste cofre)
nucleo-geral-erro-parte-repetida = a mesma parte foi informada duas vezes
nucleo-geral-erro-partes-nao-reconstroem = as partes não reconstroem a chave (alguma foi digitada errado?)

## Backups (`backup`)

nucleo-geral-backup = backup { $id }
nucleo-geral-arquivo-backup = arquivo do backup
# Fica gravado como o motivo de um backup achado na pasta ainda não ter sido verificado.
nucleo-geral-backup-encontrado = encontrado na pasta; verifique antes de usar
nucleo-geral-erro-backup-hora = hora do backup deve estar entre 0 e 23
nucleo-geral-erro-backup-senha-antes = defina a senha de backup antes de ativá-la
nucleo-geral-erro-backup-senha-curta = a senha de backup precisa de pelo menos { $minimo } caracteres
nucleo-geral-erro-backup-criar = não foi possível criar o backup: { $erro }
nucleo-geral-erro-backup-hash = o hash SHA-256 não confere: arquivo alterado ou corrompido
nucleo-geral-erro-backup-senha-antiga = informe a senha usada quando este backup foi feito
nucleo-geral-erro-backup-senha = informe a senha de backup

## Termos de uso (`terms`): o que mudou em cada versão

nucleo-geral-termos-1-0 = Primeira versão dos termos.
nucleo-geral-erro-termos-mudaram = os termos mudaram (vigente: { $versao }); leia de novo antes de aceitar

## Apoio ao projeto (`support`)

nucleo-geral-erro-pix-chave = a chave Pix precisa ser uma chave aleatória
nucleo-geral-erro-pix-recebedor = nome e cidade do recebedor do Pix
nucleo-geral-erro-apoio-so-interface = o lembrete de apoio só pela interface do app
nucleo-geral-erro-evento = evento desconhecido: { $evento }

## Feedback ao desenvolvedor (`devfeedback`)

nucleo-geral-feedback = feedback { $id }
nucleo-geral-erro-feedback-vazio = informe `{ $campo }`
nucleo-geral-erro-feedback-longo = `{ $campo }` pode ter no máximo { $maximo } caracteres
nucleo-geral-erro-feedback-importancia = importância deve ficar entre 1 e 1000
nucleo-geral-erro-modo-desenvolvedor = modo desenvolvedor desligado
nucleo-geral-erro-feedback-so-interface = gravar o feedback em arquivo só pela interface do app; use export_dev_feedback para receber o Markdown
nucleo-geral-erro-feedback-ja-fechado = este feedback já está resolvido ou descartado
nucleo-geral-erro-feedback-motivo = informe o motivo: o que resolveu e onde (versão, commit ou mudança)
nucleo-geral-feedback-resolvido-por = Resolvido por { $cliente }: { $motivo }
nucleo-geral-feedback-reforcado = Já havia um feedback aberto parecido ("{ $titulo }"): ele foi reforçado (importância { $importancia }) em vez de duplicado.
nucleo-geral-feedback-registrado = Feedback registrado para o desenvolvedor (tela Desenvolvedor).
nucleo-geral-erro-feedback-itens = `{ $campo }` aceita no máximo { $maximo } itens
nucleo-geral-erro-feedback-posicao = a posição começa em 1 (o topo da coluna)
nucleo-geral-erro-feedback-prioridade = informe a importância, a posição na coluna ou as duas
nucleo-geral-erro-feedback-vinculo-proprio = um feedback não pode ser ligado a ele mesmo
nucleo-geral-feedback-vinculo = vínculo entre o feedback { $id } e o { $outro }
nucleo-geral-feedback-status-nota = { $status ->
    [aberto] Voltou a aberto
    [em_andamento] Em andamento
    [resolvido] Resolvido
   *[descartado] Descartado
}
nucleo-geral-feedback-mesclado = Duplicado mesclado aqui: { $titulo }
nucleo-geral-feedback-duplicado-de = duplicado de "{ $titulo }"
nucleo-geral-motivo-feedback-registrar = Feedback ao desenvolvedor: registrar
nucleo-geral-motivo-feedback-reforcar = Feedback ao desenvolvedor: reforçar
nucleo-geral-motivo-feedback-resolver = Feedback ao desenvolvedor: marcar como resolvido
nucleo-geral-motivo-feedback-status = Feedback ao desenvolvedor: mudar de coluna
nucleo-geral-motivo-feedback-prioridade = Feedback ao desenvolvedor: mudar a prioridade
nucleo-geral-motivo-feedback-editar = Feedback ao desenvolvedor: editar
nucleo-geral-motivo-feedback-vincular = Feedback ao desenvolvedor: vincular
nucleo-geral-motivo-feedback-desvincular = Feedback ao desenvolvedor: desfazer vínculo
nucleo-geral-motivo-feedback-apagar = Feedback ao desenvolvedor: apagar
nucleo-geral-feedback-md-contexto = Feedback `{ $id }` do Treasury · área { $area } · tipo { $tipo } · prioridade { $prioridade } ({ $reforcos } reforços) · { $status }.
nucleo-geral-feedback-md-problema = Problema
nucleo-geral-feedback-md-esperado-obtido = Esperado × obtido
nucleo-geral-feedback-md-esperado = Esperado
nucleo-geral-feedback-md-obtido = Obtido
nucleo-geral-feedback-md-passos = Como reproduzir
nucleo-geral-feedback-md-proposta = Proposta
nucleo-geral-feedback-md-aceite = Critério de aceite
nucleo-geral-feedback-md-sem-aceite = Não informado: defina um teste que prove a correção.
nucleo-geral-feedback-md-onde = Onde mexer
nucleo-geral-feedback-md-chamadas = Chamadas que levaram ao problema
nucleo-geral-feedback-md-relacionados = Relacionados
nucleo-geral-feedback-md-reforcos = Reforços e notas
nucleo-geral-feedback-md-fechar = Ao terminar, rode os testes e marque como resolvido no MCP do Treasury: `resolve_dev_feedback` com `id` = `{ $id }` e o motivo (o que mudou, versão ou commit).

## Cofre de demonstração (`demo`): erros do assistente e os dados de exemplo, que nascem no
## idioma da instalação (marcas e nomes próprios ficam como estão)

nucleo-geral-erro-demo-meses = o histórico vai de 3 a 36 meses
nucleo-geral-erro-demo-adultos = informe um ou dois adultos
nucleo-geral-erro-demo-filhos = até 4 filhos
nucleo-geral-erro-demo-nome = informe o nome de cada pessoa
nucleo-geral-erro-demo-nome-repetido = nome repetido: { $nome }
nucleo-geral-erro-demo-renda = informe a renda de ao menos um adulto
nucleo-geral-erro-demo-banco = informe o banco da conta principal
nucleo-geral-erro-demo-valores = valores entre zero e 100 milhões
nucleo-geral-erro-demo-financiamento = informe a parcela do financiamento
nucleo-geral-erro-demo-parcelas-carro = informe quantas parcelas do carro faltam (1 a 96)
nucleo-geral-erro-demo-cofre-com-dados = o cofre de demonstração só é gerado num cofre sem lançamentos
nucleo-geral-erro-demo-so-interface = só a interface do app gera o cofre de demonstração
nucleo-geral-erro-demo-interno = cofre de demonstração: { $erro }
nucleo-geral-erro-demo-categoria = categoria padrão ausente: { $caminho }
nucleo-geral-erro-demo-data = data inválida
nucleo-geral-motivo-demo = Cofre de demonstração gerado
nucleo-geral-demo-conta = Conta { $nome }
nucleo-geral-demo-cartao = Cartão { $nome }
nucleo-geral-demo-reserva = Reserva { $nome }
nucleo-geral-demo-corretora = Corretora XP
nucleo-geral-demo-previdencia = Previdência { $nome }
nucleo-geral-demo-cliente-a = Cliente A
nucleo-geral-demo-cliente-b = Cliente B
nucleo-geral-demo-cliente-c = Cliente C
nucleo-geral-demo-projeto = Projeto
nucleo-geral-demo-recebimento = Recebimento { $nome }
nucleo-geral-demo-salario = Salário { $nome }
nucleo-geral-demo-13-1 = 13º salário (1ª parcela)
nucleo-geral-demo-13-2 = 13º salário (2ª parcela)
nucleo-geral-demo-aluguel-recebido = Aluguel recebido (apartamento)
nucleo-geral-demo-aluguel = Aluguel
nucleo-geral-demo-condominio = Condomínio
nucleo-geral-demo-luz = Conta de luz
nucleo-geral-demo-agua = Conta de água
nucleo-geral-demo-internet = Internet fibra
nucleo-geral-demo-celular = Plano de celular
nucleo-geral-demo-padaria = Padaria do bairro
nucleo-geral-demo-estacionamento = Estacionamento
nucleo-geral-demo-ipva = IPVA e licenciamento
nucleo-geral-demo-seguro-carro = Seguro do carro
nucleo-geral-demo-revisao = Revisão e troca de óleo
nucleo-geral-demo-bilhete = Recarga do bilhete único
nucleo-geral-demo-mensalidade = Mensalidade escolar ({ $nome })
nucleo-geral-demo-material = Material escolar ({ $nome })
nucleo-geral-demo-plano-saude = Plano de saúde
nucleo-geral-demo-restaurante = Restaurante
nucleo-geral-demo-pizzaria = Pizzaria
nucleo-geral-demo-churrascaria = Churrascaria
nucleo-geral-demo-teatro = Teatro
nucleo-geral-demo-parque = Parque
nucleo-geral-demo-passeio = Passeio
nucleo-geral-demo-museu = Museu
nucleo-geral-demo-viagem = Viagem de férias
nucleo-geral-demo-financiamento-apartamento = Financiamento do apartamento
nucleo-geral-demo-financiamento-carro = Financiamento do carro
# O nome do contrato vem em minúsculas ("Parcela financiamento do carro").
nucleo-geral-demo-parcela = Parcela { $nome }
nucleo-geral-demo-ativo-nota = Ativo do cofre de demonstração: cotações fictícias.
nucleo-geral-demo-rendimento-reserva = Rendimento da reserva
nucleo-geral-demo-aporte-reserva = Aporte na reserva
nucleo-geral-demo-aporte-corretora = Aporte na corretora
nucleo-geral-demo-aporte-previdencia = Aporte na previdência
nucleo-geral-demo-meta-reserva = Reserva de emergência

## Banco de dados: tamanho, manutenção e expurgo (`maintenance`)

nucleo-geral-grupo-historico = histórico de alterações
nucleo-geral-grupo-lancamentos = lançamentos
nucleo-geral-grupo-contas = contas e cartões
nucleo-geral-grupo-carteira = carteira
nucleo-geral-grupo-cotacoes = cotações, índices e referências
nucleo-geral-grupo-analise = análise de ativos (CVM, B3, SEC)
nucleo-geral-grupo-agente = agente
nucleo-geral-grupo-dividas = dívidas
nucleo-geral-grupo-patrimonio = patrimônio físico
nucleo-geral-expurgo-historico = Histórico de alterações antigo
nucleo-geral-expurgo-historico-consequencia = As alterações apagadas não podem mais ser desfeitas; propostas pendentes ficam.
nucleo-geral-expurgo-auditoria = Auditoria do agente e do MCP
nucleo-geral-expurgo-auditoria-consequencia = Some o registro de quem consultou ou propôs o quê nesse período.
nucleo-geral-expurgo-conversas = Conversas antigas do agente
nucleo-geral-expurgo-conversas-consequencia = Conversas não fixadas sem uso no período (e os anexos delas) somem.
nucleo-geral-expurgo-mercado = Caches de mercado
nucleo-geral-expurgo-mercado-consequencia = Cotações intradiárias antigas e o cache do Fundamentus; voltam na próxima consulta.
nucleo-geral-expurgo-analise = Dados da análise de ativos
nucleo-geral-expurgo-analise-consequencia = Fundamentos, cotações do universo, documentos e indicadores dos bancos baixados da CVM, B3, SEC e Banco Central; a próxima coleta baixa tudo de novo (cerca de 230 MB pela rede).
nucleo-geral-erro-expurgo-backup = o backup antes do expurgo não passou na verificação ({ $erro }); nada foi apagado
nucleo-geral-referencias-quebradas = { $n } referência(s) quebrada(s) entre tabelas
nucleo-geral-tabela = tabela { $tabela }
nucleo-geral-ordem-chave-primaria = chave primária
nucleo-geral-mostrando-linhas = mostrando as { $limite } primeiras de { $total } linhas
nucleo-geral-segredos-ocultos = chaves, tokens e segredos aparecem ocultos
nucleo-geral-erro-banco-so-interface = manutenção e expurgo do banco só pela interface do app
nucleo-geral-erro-banco-leitura = banco aberto só para leitura

## Exportação em formato aberto (`export`): erros, o `about` do JSON e o CSV de lançamentos

nucleo-geral-erro-exportar-caminho = escolha um caminho completo para o arquivo
nucleo-geral-erro-exportar-nome = informe o nome do arquivo
nucleo-geral-erro-exportar-pasta-destino = pasta de destino inválida
nucleo-geral-erro-exportar-pasta-inexistente = a pasta { $pasta } não existe
nucleo-geral-erro-exportar-destino-pasta = o destino é uma pasta; informe o nome do arquivo
nucleo-geral-erro-exportar-pasta-treasury = escolha uma pasta fora das pastas de dados e de backups do Treasury
nucleo-geral-erro-exportar-pasta = pasta inválida
nucleo-geral-erro-csv = erro ao gravar CSV: { $erro }
nucleo-geral-exportacao-descricao = Exportação completa dos dados do Treasury em formato aberto. Este arquivo NÃO é criptografado.
nucleo-geral-exportacao-tabelas = Cada tabela é uma lista de objetos com as colunas do esquema do banco.
nucleo-geral-exportacao-dinheiro = Valores monetários são inteiros na menor unidade da moeda (centavos) com a moeda na coluna currency. Em txn.amount, + entra e − sai da conta.
nucleo-geral-exportacao-decimais = Quantidades e taxas exatas são texto decimal.
nucleo-geral-exportacao-datas = Datas de lançamento em AAAA-MM-DD (sem fuso); instantes em UTC (RFC 3339).
nucleo-geral-exportacao-ids = Identificadores UUID v7 em texto; as referências entre tabelas usam esses ids.
nucleo-geral-exportacao-json = Colunas como rule.conditions, rule.actions e setting.value guardam JSON em texto.
nucleo-geral-exportacao-binarios = Colunas binárias (ver blob_columns) estão em base64 padrão.
nucleo-geral-exportacao-segredos = Segredos (senha de backup, chaves de API) nunca são exportados.
nucleo-geral-csv-data = Data
nucleo-geral-csv-conta = Conta
nucleo-geral-csv-descricao = Descrição
nucleo-geral-csv-favorecido = Favorecido
nucleo-geral-csv-categoria = Categoria
nucleo-geral-csv-titular = Titular
nucleo-geral-csv-valor = Valor
nucleo-geral-csv-moeda = Moeda
nucleo-geral-csv-tipo = Tipo
nucleo-geral-csv-situacao = Situação
nucleo-geral-csv-tags = Tags
nucleo-geral-csv-notas = Notas
nucleo-geral-csv-vencimento-fatura = Vencimento da fatura
nucleo-geral-csv-parcela = Parcela
nucleo-geral-csv-divisao = Divisão
nucleo-geral-csv-id = Id
nucleo-geral-csv-sem-categoria = Sem categoria
nucleo-geral-csv-receita = Receita
nucleo-geral-csv-despesa = Despesa
nucleo-geral-csv-transferencia = Transferência
nucleo-geral-csv-pagamento-fatura = Pagamento de fatura
nucleo-geral-csv-investimento = Investimento
nucleo-geral-csv-ajuste = Ajuste
nucleo-geral-csv-previsto = Previsto
nucleo-geral-csv-efetivado = Efetivado
nucleo-geral-csv-conciliado = Conciliado

## Histórico de alterações (`changes`)

nucleo-geral-conjunto = conjunto de alterações { $id }
nucleo-geral-erro-sem-motivo = toda alteração precisa de um motivo
nucleo-geral-erro-aprovar = só é possível aprovar alterações pendentes
nucleo-geral-erro-rejeitar = só é possível rejeitar alterações pendentes
nucleo-geral-erro-desfazer = só é possível desfazer alterações aplicadas
nucleo-geral-erro-refazer = só é possível refazer alterações desfeitas
nucleo-geral-erro-alteracoes-posteriores = alterações posteriores mudaram estes dados; desfaça primeiro as alterações mais recentes que tocam nas mesmas linhas

## Lixeira, keychain e config.toml (`trash`, `keychain`, `config`)

nucleo-geral-arquivo = arquivo { $caminho }
nucleo-geral-erro-caminho-sem-nome = caminho sem nome de arquivo
nucleo-geral-erro-keychain-indisponivel = keychain indisponível neste sistema
nucleo-geral-erro-keychain-negado = acesso ao keychain negado ou keychain bloqueado
nucleo-geral-erro-keychain = erro do keychain: { $erro }
nucleo-geral-erro-config = config.toml inválido: { $erro }

# Recuperação por backup (docs/backup-recovery.md)
nucleo-geral-erro-backup-fora-da-pasta = Esse arquivo não está na pasta de backups deste cofre.
nucleo-geral-erro-backup-destino-igual = Escolha outro lugar para a cópia: esse é o próprio backup.
nucleo-geral-erro-backup-copia-diferente = A cópia saiu diferente do backup; tente de novo ou escolha outro lugar.
nucleo-geral-erro-backup-nao-e-backup = Esse arquivo não parece um backup do Treasury (os backups são bancos cifrados, terminados em .db.enc).

## Tarefas em segundo plano (`tasks`)

nucleo-geral-tarefa-nao-encontrada = tarefa não encontrada (pode já ter terminado)
nucleo-geral-tarefa-nao-cancelavel = essa tarefa não pode ser cancelada
