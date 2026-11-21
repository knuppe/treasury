# Bóvedas (núcleo, `vault` y `vaults`; docs/vaults.md): nombres que la app da a las bóvedas y
# errores de la lista de bóvedas y de la clave de la base de datos.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

# Nombre de la bóveda de siempre (la primera), hasta que alguien la renombre.
nucleo-cofre-principal = Principal
# Nombre de una bóveda agregada sin nombre ni nombre de archivo.
nucleo-cofre-nome-padrao = Bóveda
nucleo-cofre-copia = { $nome } (copia)
nucleo-cofre-cofre = bóveda { $id }
nucleo-cofre-erro-sem-nome = ponle un nombre a la bóveda
nucleo-cofre-erro-nome-longo = nombre de la bóveda demasiado largo (máximo { $maximo } caracteres)
nucleo-cofre-erro-caminho-completo = ingresa la ruta completa del archivo
nucleo-cofre-erro-arquivo-nao-encontrado = archivo no encontrado: { $caminho }
nucleo-cofre-erro-escolha-arquivo = elige un archivo de bóveda (.db) de Treasury
nucleo-cofre-erro-sqlite-sem-cifra = este archivo es una base SQLite sin cifrado, no una bóveda de Treasury
nucleo-cofre-erro-arquivo-na-lista = este archivo ya está en la lista de bóvedas
nucleo-cofre-erro-arquivo-existe = ya existe un archivo en { $caminho }; para usarlo, agrégalo como bóveda existente
nucleo-cofre-erro-caminho-na-lista = esta ruta ya está en la lista de bóvedas
nucleo-cofre-erro-nao-criado = esta bóveda todavía no se creó (o el archivo ya no está ahí)
nucleo-cofre-erro-em-uso = esta es la bóveda en uso: cambia de bóveda antes de quitarla de la lista
nucleo-cofre-erro-codigo-recuperacao = código de recuperación inválido
nucleo-cofre-erro-segredo-keychain = secreto inválido en el keychain
nucleo-cofre-erro-banco-corrompido = archivo de la base de datos vacío o dañado
