# Vaults (core, `vault` and `vaults`; docs/vaults.md): names the app gives to vaults and errors
# from the vault list and the database key.
# Revision: AI draft, not yet reviewed by a native speaker.

# Name of the original (first) vault, until someone renames it.
nucleo-cofre-principal = Main
# Name of a vault added with no name and no file name.
nucleo-cofre-nome-padrao = Vault
nucleo-cofre-copia = { $nome } (copy)
nucleo-cofre-cofre = vault { $id }
nucleo-cofre-erro-sem-nome = give the vault a name
nucleo-cofre-erro-nome-longo = vault name too long (maximum { $maximo } characters)
nucleo-cofre-erro-caminho-completo = enter the file's full path
nucleo-cofre-erro-arquivo-nao-encontrado = file not found: { $caminho }
nucleo-cofre-erro-escolha-arquivo = choose a Treasury vault file (.db)
nucleo-cofre-erro-sqlite-sem-cifra = this file is an unencrypted SQLite database, not a Treasury vault
nucleo-cofre-erro-arquivo-na-lista = this file is already on the vault list
nucleo-cofre-erro-arquivo-existe = there's already a file at { $caminho }; to use it, add it as an existing vault
nucleo-cofre-erro-caminho-na-lista = this path is already on the vault list
nucleo-cofre-erro-nao-criado = this vault hasn't been created yet (or the file is no longer there)
nucleo-cofre-erro-em-uso = this is the vault in use: switch vaults before removing it from the list
nucleo-cofre-erro-codigo-recuperacao = invalid recovery code
nucleo-cofre-erro-segredo-keychain = invalid secret in the keychain
nucleo-cofre-erro-banco-corrompido = database file is empty or corrupted
