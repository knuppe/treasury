# Variante pt-PT (cofre: criar, desbloquear, recuperar e trocar; país da família): o que faltar
# cai para pt-BR. Revisão: rascunho por IA.

## Moldura dos ecrãs do cofre (VaultLayout)

cofre-ficha-chave-valor = Argon2id sobre a palavra-passe mestra
cofre-ficha-arquivo = Ficheiro
cofre-ficha-rede-valor = Nenhuma ligação necessária

## Porta do cofre (VaultGate)

cofre-lendo = A ler o cofre
cofre-nucleo-tentar = Tentar novamente

## Palavra-passe mestra (comum a criar e recuperar)

cofre-senha-mestra = Palavra-passe mestra
cofre-confirmar-senha = Confirmar palavra-passe
cofre-nova-senha-mestra = Nova palavra-passe mestra
cofre-confirmar-nova-senha = Confirmar nova palavra-passe
cofre-senha-erro-vazia = Crie uma palavra-passe mestra.
cofre-senha-erro-curta = Use pelo menos { $min ->
        [one] { $min } carácter
       *[other] { $min } caracteres
    }.
cofre-senha-erro-fraca = Palavra-passe demasiado fraca para proteger o cofre. Veja as sugestões abaixo.
cofre-senha-erro-confirmar = As palavras-passe não coincidem.

## Medidor de força da palavra-passe

cofre-forca-rotulo = Força da palavra-passe
cofre-forca-sem-senha = Sem palavra-passe

## Criar o cofre

cofre-criar-lead = Tudo fica num único ficheiro neste computador, cifrado com uma chave derivada da sua palavra-passe mestra. Nada sai daqui sem que o peça.
cofre-criar-pais-dica = { $suporte } Moeda base: { $moeda }. Pode alterá-la depois em Ajustes › Família.
cofre-criar-membros-ajuda = Quem aparece como dono de contas, receitas e despesas nos relatórios. Pode alterar depois.
cofre-criar-aviso-forte = Sem a palavra-passe mestra e sem o kit de recuperação, não há forma de recuperar os dados.
cofre-criar-aviso = O Treasury não guarda a palavra-passe nem tem servidor: ninguém, nem nós, consegue abrir o cofre sem ela. Anote-a num gestor de palavras-passe e gere o kit em Ajustes.
cofre-criar-aviso-entendi = Compreendi que, se me esquecer da palavra-passe e não tiver o kit de recuperação, perco os dados.
cofre-criar-carregando = A derivar a chave e a cifrar…
cofre-criar-erro-nome-familia = Indique o nome da família.
cofre-criar-erro-sem-membros = Indique pelo menos um membro.
cofre-criar-erro-aviso = Confirme que compreendeu o aviso acima.
cofre-criar-toast-descricao = Guarde a palavra-passe mestra num local seguro e gere o kit de recuperação em Ajustes.

## Desbloquear o cofre

cofre-desbloquear-lead-senha = O cofre está cifrado em repouso. Introduza a palavra-passe mestra para abrir os dados desta família.
cofre-desbloquear-lead-pin = O PIN só é válido neste computador. Cinco erros desativam o PIN e pedem a palavra-passe mestra.
cofre-desbloquear-lead-recuperacao = Use o kit, duas das três partes de recuperação ou a chave física para abrir o cofre e definir uma nova palavra-passe mestra.
cofre-desbloquear-senha-mac = Desbloquear com a palavra-passe do Mac
cofre-desbloquear-derivando = A derivar a chave…
cofre-desbloquear-erro-vazia = Introduza a palavra-passe mestra.
cofre-desbloquear-esqueceu = Esqueceu-se da palavra-passe?
cofre-pin-conferindo = A verificar…
cofre-usar-senha-mestra = Usar a palavra-passe mestra

## Recuperar o cofre

cofre-recuperar-codigo-dica = O código impresso ou exportado quando gerou o kit.
cofre-recuperar-pin-chave = PIN da chave física (se tiver)
cofre-recuperar-recifrando = A recifrar o cofre…
cofre-recuperar-botao = Redefinir a palavra-passe e abrir
cofre-recuperar-voltar = Voltar à palavra-passe
cofre-recuperar-erro-codigo = Introduza o código do kit de recuperação.
cofre-recuperar-erro-partes = Introduza duas das três partes.
cofre-recuperar-erro-sem-chave = Nenhuma chave física registada.
cofre-recuperar-toast-titulo = Palavra-passe mestra redefinida
cofre-recuperar-toast-descricao = Os outros métodos de recuperação continuam válidos. Guarde a nova palavra-passe.

## Lista de cofres (VaultSwitcher) e menu lateral (VaultMenu)

cofre-menu-descricao = Cada cofre é um ficheiro com palavra-passe, desbloqueio rápido e cópias de segurança próprios. Trocar fecha o cofre atual.
cofre-abrir-existente = Abrir ficheiro existente
cofre-acao-copiar-caminho = Copiar o caminho do ficheiro
cofre-acao-remover-indisponivel = Este é o cofre em uso: troque de cofre antes de o remover.
cofre-copia-toast-descricao = { $nome } (cópia), com a mesma palavra-passe.
cofre-remover-descricao = As cópias de segurança dele ficam onde estão.
cofre-remover-confirmar-lixeira = Remover e enviar para o Lixo
cofre-remover-opcoes = O que fazer com o ficheiro
cofre-remover-so-lista-dica = O ficheiro continua em { $caminho }.
cofre-remover-lixeira = Tirar da lista e enviar o ficheiro para o Lixo
cofre-remover-lixeira-dica = Pode recuperá-lo no Lixo enquanto este não for esvaziado.
cofre-remover-toast-lixeira = O ficheiro foi para o Lixo.
cofre-remover-toast-lista = O ficheiro continua no mesmo sítio.
cofre-novo-ajuda = O cofre novo tem palavra-passe, desbloqueio rápido e cópias de segurança próprios. A seguir, define a palavra-passe dele.
cofre-existente-arquivo = Ficheiro do cofre (.db)
cofre-existente-abrir = Abrir este ficheiro

## País da família (CountryPicker)

cofre-pais-rotulo = País onde a família vive
cofre-pais-suporte-completo = Tudo da aplicação: B3, Tesouro Direto, fundos da CVM, IR com DARF, INSS e IRRF no recibo de vencimento, FIPE e juros do Banco Central.
cofre-pais-suporte-adaptado = Moeda, inflação e juros do país, salários (descontos e pagamentos extra), estimativa de impostos, categorias e regras prontas de lá; o que é só do Brasil fica desligado.
cofre-pais-suporte-generico = Moeda, inflação, juros e câmbio do país pelo catálogo de índices; sem regras locais de impostos e salários, e sem o que é só do Brasil.

## Força da palavra-passe (lib/passwordStrength.ts)

cofre-sugestao-longa = Use pelo menos 12 caracteres: palavras-passe longas resistem muito mais.
cofre-sugestao-comuns = Evite palavras e palavras-passe comuns, mesmo com trocas como 4 no lugar de a.
cofre-sugestao-repetir = Evite repetir o mesmo carácter.

## Demonstração e escolha do cofre

cofre-demo-lead = Cofre de demonstração com dados fictícios: abre sem palavra-passe.
cofre-escolha-lead = Para começar, escolha: o cofre da família, com palavra-passe mestra, ou uma demonstração para conhecer a aplicação.
cofre-escolha-real-dica = Para as finanças da família: cifrado com a sua palavra-passe mestra, só o utilizador o abre.
cofre-escolha-demo-dica = Dados fictícios de uma família de exemplo, prontos em segundos e sem palavra-passe.
