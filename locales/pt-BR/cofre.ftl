# Cofre: criar, desbloquear, recuperar e trocar de cofre; país da família (docs/i18n-design.md).
# pt-BR é o idioma de origem.

## Moldura das telas do cofre (VaultLayout)

cofre-ficha-cifra = Cifra
cofre-ficha-chave = Chave
cofre-ficha-chave-valor = Argon2id sobre a senha mestra
cofre-ficha-arquivo = Arquivo
cofre-ficha-rede = Rede
cofre-ficha-rede-valor = Nenhuma conexão necessária
cofre-lema = Medido, nunca inventado.

## Porta do cofre (VaultGate)

cofre-lendo = Lendo o cofre
cofre-nucleo-indisponivel = Núcleo indisponível
cofre-nucleo-erro = Não foi possível ler o estado do cofre.
cofre-nucleo-tentar = Tentar de novo
cofre-erro-desconhecido = Erro desconhecido.

## Senha mestra (comum a criar e recuperar)

cofre-senha-mestra = Senha mestra
cofre-confirmar-senha = Confirmar senha
cofre-nova-senha-mestra = Nova senha mestra
cofre-confirmar-nova-senha = Confirmar nova senha
cofre-senha-erro-vazia = Crie uma senha mestra.
cofre-senha-erro-curta = Use pelo menos { $min ->
        [one] { $min } caractere
       *[other] { $min } caracteres
    }.
cofre-senha-erro-fraca = Senha fraca demais para proteger o cofre. Veja as sugestões abaixo.
cofre-senha-erro-confirmar = As senhas não conferem.

## Medidor de força da senha

cofre-forca-rotulo = Força da senha
cofre-forca-sem-senha = Sem senha
cofre-forca-descricao = { $nivel }, cerca de { $bits } bits
cofre-forca-bits = ≈ { $bits } bits

## Criar o cofre

cofre-criar-kicker = 00 · Novo cofre
cofre-criar-kicker-nomeado = 00 · Novo cofre · { $nome }
cofre-criar-titulo = Um cofre cifrado para as finanças da família
cofre-criar-lead = Tudo fica num único arquivo neste computador, cifrado com uma chave derivada da sua senha mestra. Nada sai daqui sem você pedir.
cofre-criar-card-titulo = Criar cofre
cofre-criar-arquivo-unico = único
cofre-criar-nome-familia = Nome da família
cofre-criar-nome-familia-exemplo = Família Silva
cofre-criar-pais-dica = { $suporte } Moeda base: { $moeda }. Dá para mudar depois em Ajustes › Família.
cofre-criar-membros = Membros da família
cofre-criar-membros-ajuda = Quem aparece como dono de contas, receitas e despesas nos relatórios. Dá para mudar depois.
cofre-criar-membro-n = Membro { $n }
cofre-criar-membro-exemplo = Ana
cofre-criar-membro-placeholder = Nome
cofre-criar-remover-membro = Remover membro { $n }
cofre-criar-adicionar-membro = Adicionar membro
cofre-criar-aviso-forte = Sem a senha mestra e sem o kit de recuperação, não há como recuperar os dados.
cofre-criar-aviso = O Treasury não guarda a senha nem tem servidor: ninguém, nem nós, consegue abrir o cofre sem ela. Anote-a num gerenciador de senhas e gere o kit em Ajustes.
cofre-criar-aviso-entendi = Entendi que, se esquecer a senha e não tiver o kit de recuperação, perco os dados.
cofre-criar-carregando = Derivando chave e cifrando…
cofre-criar-botao = Criar cofre cifrado
cofre-criar-erro-nome-familia = Informe o nome da família.
cofre-criar-erro-sem-membros = Informe pelo menos um membro.
cofre-criar-erro-membros-repetidos = Há membros com o mesmo nome.
cofre-criar-erro-aviso = Confirme que entendeu o aviso acima.
cofre-criar-toast-titulo = Cofre criado e cifrado
cofre-criar-toast-descricao = Guarde a senha mestra num lugar seguro e gere o kit de recuperação em Ajustes.

## Desbloquear o cofre

cofre-desbloquear-cofre-padrao = Cofre do Treasury
cofre-desbloquear-kicker = 00 · Cofre bloqueado
cofre-desbloquear-kicker-nomeado = 00 · Cofre bloqueado · { $nome }
cofre-desbloquear-lead-senha = O cofre está cifrado em repouso. Digite a senha mestra para abrir os dados desta família.
cofre-desbloquear-lead-pin = O PIN só vale neste computador. Cinco erros desligam o PIN e pedem a senha mestra.
cofre-desbloquear-lead-recuperacao = Use o kit, duas das três partes de recuperação ou a chave física para abrir o cofre e definir uma nova senha mestra.
cofre-desbloquear = Desbloquear
cofre-desbloquear-touch-id = Desbloquear com Touch ID
cofre-desbloquear-senha-mac = Desbloquear com a senha do Mac
cofre-desbloquear-derivando = Derivando chave…
cofre-desbloquear-erro-vazia = Digite a senha mestra.
cofre-desbloquear-esqueceu = Esqueceu a senha?
cofre-auto-bloqueio = Auto-bloqueio
cofre-auto-bloqueio-minutos = { $minutos } min
cofre-keychain = Keychain
cofre-ligado = ativo
cofre-desligado = desligado
cofre-usar-pin = Usar PIN
cofre-pin-titulo = PIN
cofre-pin-campo = PIN deste computador
cofre-pin-conferindo = Conferindo…
cofre-usar-senha-mestra = Usar a senha mestra

## Recuperar o cofre

cofre-recuperar = Recuperar o cofre
cofre-recuperar-como = Como recuperar
cofre-recuperar-metodo-kit = Kit
cofre-recuperar-metodo-partes = Partes
cofre-recuperar-metodo-chave = Chave física
cofre-recuperar-codigo = Código de recuperação
cofre-recuperar-codigo-mock = Mock: use { $codigo }
cofre-recuperar-codigo-dica = O código impresso ou exportado quando você gerou o kit.
cofre-recuperar-primeira-parte = Primeira parte
cofre-recuperar-segunda-parte = Segunda parte
cofre-recuperar-partes-dica = Quaisquer duas das três partes (TRSP-…).
cofre-recuperar-chave = Chave física
cofre-recuperar-pin-chave = PIN da chave física (se ela tiver)
cofre-recuperar-pin-chave-dica = Toque na chave quando ela piscar.
cofre-recuperar-toque-chave = Toque na chave física…
cofre-recuperar-recifrando = Recifrando o cofre…
cofre-recuperar-botao = Redefinir senha e abrir
cofre-recuperar-voltar = Voltar para a senha
cofre-recuperar-erro-codigo = Digite o código do kit de recuperação.
cofre-recuperar-erro-partes = Digite duas das três partes.
cofre-recuperar-erro-sem-chave = Nenhuma chave física cadastrada.
cofre-recuperar-toast-titulo = Senha mestra redefinida
cofre-recuperar-toast-descricao = Os outros métodos de recuperação continuam valendo. Guarde a nova senha.

## Lista de cofres (VaultSwitcher) e menu lateral (VaultMenu)

cofre-menu-titulo = Cofres
cofre-menu-descricao = Cada cofre é um arquivo com senha, desbloqueio rápido e backups próprios. Trocar fecha o cofre atual.
cofre-menu-cofre-atual = Cofre: { $nome }
cofre-menu-trocar = Trocar ou criar cofre
cofre-lista-rotulo = Cofres
cofre-novo-cofre = Novo cofre
cofre-abrir-existente = Abrir arquivo existente
cofre-demonstracao = Cofre de demonstração
cofre-outros = Outros cofres
cofre-outros-n = Outros cofres ({ $n })
cofre-em-uso = em uso
cofre-ainda-nao-criado = ainda não criado
cofre-acoes-de = Ações de { $nome }
cofre-mais-acoes = Mais ações (ou clique com o botão direito)
cofre-acao-abrir = Abrir
cofre-acao-renomear = Renomear e finalidade
cofre-acao-copiar = Criar uma cópia
cofre-acao-copiar-indisponivel = O cofre ainda não foi criado.
cofre-acao-copiar-caminho = Copiar o caminho do arquivo
cofre-acao-remover = Remover…
cofre-acao-remover-indisponivel = Este é o cofre em uso: troque de cofre antes de removê-lo.
cofre-copia-toast-titulo = Cópia criada
cofre-copia-toast-descricao = { $nome } (cópia), com a mesma senha.
cofre-caminho-copiado = Caminho copiado
cofre-nome = Nome
cofre-finalidade = Finalidade
cofre-remover-titulo = Remover { $nome }?
cofre-remover-descricao = Os backups dele ficam onde estão.
cofre-remover-confirmar-lixeira = Remover e mandar para a Lixeira
cofre-remover-confirmar-lista = Tirar da lista
cofre-remover-opcoes = O que fazer com o arquivo
cofre-remover-so-lista = Só tirar da lista
cofre-remover-so-lista-dica = O arquivo continua em { $caminho }.
cofre-remover-lixeira = Tirar da lista e mandar o arquivo para a Lixeira
cofre-remover-lixeira-dica = Dá para recuperar pela Lixeira enquanto ela não for esvaziada.
cofre-remover-toast-titulo = { $nome } removido
cofre-remover-toast-lixeira = O arquivo foi para a Lixeira.
cofre-remover-toast-lista = O arquivo continua no lugar.
cofre-novo-nome = Nome do cofre
cofre-novo-nome-exemplo = Ex.: Empresa, Casa dos pais
cofre-novo-finalidade = Finalidade (opcional)
cofre-novo-ajuda = O cofre novo tem senha, desbloqueio rápido e backups próprios. Em seguida você define a senha dele.
cofre-novo-botao = Criar cofre
cofre-existente-arquivo = Arquivo do cofre (.db)
cofre-existente-arquivo-dica = O caminho completo, como /Users/ana/cofres/empresa.db.
cofre-existente-escolher = Escolher…
cofre-existente-nome = Nome (opcional)
cofre-existente-abrir = Abrir este arquivo

## País da família (CountryPicker)

cofre-pais-rotulo = País onde a família mora
cofre-pais-legenda = ★ com adaptação própria; os outros países usam o catálogo de índices.
cofre-pais-suporte-completo = Tudo do app: B3, Tesouro Direto, fundos da CVM, IR com DARF, INSS e IRRF no holerite, FIPE e juros do Banco Central.
cofre-pais-suporte-adaptado = Moeda, inflação e juros do país, folha de pagamento (descontos e pagamentos extras), estimativa de impostos, categorias e regras prontas de lá; o que é só do Brasil fica desligado.
cofre-pais-suporte-generico = Moeda, inflação, juros e câmbio do país pelo catálogo de índices; sem regras locais de impostos e folha, e sem o que é só do Brasil.

## Força da senha (lib/passwordStrength.ts)

cofre-forca-muito-fraca = Muito fraca
cofre-forca-fraca = Fraca
cofre-forca-razoavel = Razoável
cofre-forca-boa = Boa
cofre-forca-forte = Forte
cofre-sugestao-longa = Use pelo menos 12 caracteres: senhas longas resistem muito mais.
cofre-sugestao-nomes = Não use o nome da família nem dos membros.
cofre-sugestao-comuns = Evite palavras e senhas comuns, mesmo com trocas como 4 no lugar de a.
cofre-sugestao-anos = Evite anos e datas, que são fáceis de adivinhar.
cofre-sugestao-sequencias = Evite sequências como abc, 123 ou qwerty.
cofre-sugestao-repetir = Evite repetir o mesmo caractere.
cofre-sugestao-frase = Uma frase com 4 ou mais palavras sem relação entre si é forte e fácil de lembrar.

cofre-so-no-app = Disponível só no app Treasury.

## Demonstração sem senha

cofre-demo-kicker = 00 · Demonstração · { $nome }
cofre-demo-lead = Cofre de demonstração com dados fictícios: abre sem senha.
cofre-demo-aviso = Qualquer pessoa com o app abre este cofre. Use só para testar, nunca com dados reais.
cofre-demo-abrir = Abrir a demonstração

## Criar: cofre real ou demonstração

cofre-escolha-lead = Para começar, escolha: o cofre da família, com senha mestra, ou uma demonstração para conhecer o app.
cofre-escolha-rotulo = Que cofre criar
cofre-escolha-real = Cofre real
cofre-escolha-real-dica = Para as finanças da família: cifrado com a sua senha mestra, só você abre.
cofre-escolha-demo = Demonstração
cofre-escolha-demo-dica = Dados fictícios de uma família de exemplo, prontos em segundos e sem senha.
cofre-escolha-demo-titulo = Cofre de demonstração
cofre-escolha-demo-lead = Escolha o país e o perfil da família de exemplo. O cofre é criado e abre em seguida.

## Recuperar de um backup (docs/backup-recovery.md)

cofre-desbloquear-lead-backup = Escolha um backup salvo neste computador e restaure o cofre a partir dele. O cofre que não abre não é apagado: fica guardado ao lado.
cofre-desbloquear-nao-abre = O cofre não abre?
cofre-backup-abrir-recuperacao = Recuperar de um backup
cofre-backup-recuperar-titulo = Recuperar de um backup
cofre-backup-motivo = O cofre não abriu: { $erro }
cofre-backup-recuperar-texto = Os backups abaixo estão na pasta deste cofre, do mais novo para o mais antigo. Escolha um e informe a senha.
cofre-backup-procurando = Procurando os backups…
cofre-backup-nenhum = Nenhum backup encontrado na pasta deste cofre. Se você tem uma cópia em outro lugar (pen drive, HD externo, nuvem), use "Trazer um backup de fora".
cofre-backup-lista = Backups encontrados
cofre-backup-tipo-desconhecido = Backup
cofre-backup-importado = { $tipo } (trazido de fora)
cofre-backup-salvar-copia = Salvar uma cópia
cofre-backup-salvar-copia-de = Salvar uma cópia do backup de { $data }
cofre-backup-copia-titulo = Salvar uma cópia do backup
cofre-backup-copia-salva = Cópia do backup salva
cofre-backup-copia-erro = Não deu para salvar a cópia
cofre-backup-copia-so-app = Salvar uma cópia só funciona no app instalado.
cofre-backup-trazer = Trazer um backup de fora
cofre-backup-trazer-titulo = Escolha o arquivo de backup (.db.enc)
cofre-backup-trazido = Backup trazido para a pasta do cofre
cofre-backup-trazer-erro = Não deu para trazer esse arquivo
cofre-backup-senha-mestra = Senha mestra
cofre-backup-senha-mestra-dica = Ela vale para o cofre restaurado. Normalmente é a senha que você já usa.
cofre-backup-outra-senha = O backup foi feito com outra senha (a antiga ou a senha de backup própria)
cofre-backup-senha-backup = Senha do backup
cofre-backup-ao-lado-aviso = O cofre atual vai para o lado, com "danificado" e a data no nome, e não é apagado.
cofre-backup-restaurar = Restaurar este backup
cofre-backup-restaurando = Restaurando…
cofre-backup-restaurado = Cofre restaurado do backup
cofre-backup-restaurado-ao-lado = O cofre que não abria ficou guardado em { $caminho }.
