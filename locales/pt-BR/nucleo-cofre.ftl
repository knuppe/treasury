# Cofres (núcleo, `vault` e `vaults`; docs/vaults.md): nomes que o app dá aos cofres e erros da
# lista de cofres e da chave do banco. pt-BR é o idioma de origem.

# Nome do cofre de sempre (o primeiro), até alguém renomear.
nucleo-cofre-principal = Principal
# Nome de um cofre adicionado sem nome nem nome de arquivo.
nucleo-cofre-nome-padrao = Cofre
nucleo-cofre-copia = { $nome } (cópia)
nucleo-cofre-cofre = cofre { $id }
nucleo-cofre-erro-sem-nome = dê um nome ao cofre
nucleo-cofre-erro-nome-longo = nome do cofre muito longo (máximo { $maximo } caracteres)
nucleo-cofre-erro-caminho-completo = informe o caminho completo do arquivo
nucleo-cofre-erro-arquivo-nao-encontrado = arquivo não encontrado: { $caminho }
nucleo-cofre-erro-escolha-arquivo = escolha um arquivo de cofre (.db) do Treasury
nucleo-cofre-erro-sqlite-sem-cifra = este arquivo é um banco SQLite sem criptografia, não um cofre do Treasury
nucleo-cofre-erro-arquivo-na-lista = este arquivo já está na lista de cofres
nucleo-cofre-erro-arquivo-existe = já existe um arquivo em { $caminho }; para usá-lo, adicione como cofre existente
nucleo-cofre-erro-caminho-na-lista = este caminho já está na lista de cofres
nucleo-cofre-erro-nao-criado = este cofre ainda não foi criado (ou o arquivo não está mais lá)
nucleo-cofre-erro-em-uso = este é o cofre em uso: troque de cofre antes de tirá-lo da lista
nucleo-cofre-erro-codigo-recuperacao = código de recuperação inválido
nucleo-cofre-erro-segredo-keychain = segredo inválido no keychain
nucleo-cofre-erro-banco-corrompido = arquivo do banco vazio ou corrompido
