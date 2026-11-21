# Bóveda: crear, desbloquear, recuperar y cambiar de bóveda; país de la familia (docs/i18n-design.md).
# Revisión: borrador por IA, sin revisión de un hablante nativo.

## Marco de las pantallas de la bóveda (VaultLayout)

cofre-ficha-cifra = Cifrado
cofre-ficha-chave = Clave
cofre-ficha-chave-valor = Argon2id sobre la contraseña maestra
cofre-ficha-arquivo = Archivo
cofre-ficha-rede = Red
cofre-ficha-rede-valor = No necesita conexión
cofre-lema = Medido, nunca inventado.

## Puerta de la bóveda (VaultGate)

cofre-lendo = Leyendo la bóveda
cofre-nucleo-indisponivel = Núcleo no disponible
cofre-nucleo-erro = No se pudo leer el estado de la bóveda.
cofre-nucleo-tentar = Intentar de nuevo
cofre-erro-desconhecido = Error desconocido.

## Contraseña maestra (común a crear y recuperar)

cofre-senha-mestra = Contraseña maestra
cofre-confirmar-senha = Confirmar contraseña
cofre-nova-senha-mestra = Nueva contraseña maestra
cofre-confirmar-nova-senha = Confirmar nueva contraseña
cofre-senha-erro-vazia = Crea una contraseña maestra.
cofre-senha-erro-curta = Usa al menos { $min ->
        [one] { $min } carácter
       *[other] { $min } caracteres
    }.
cofre-senha-erro-fraca = La contraseña es demasiado débil para proteger la bóveda. Mira las sugerencias de abajo.
cofre-senha-erro-confirmar = Las contraseñas no coinciden.

## Medidor de fuerza de la contraseña

cofre-forca-rotulo = Fuerza de la contraseña
cofre-forca-sem-senha = Sin contraseña
cofre-forca-descricao = { $nivel }, unos { $bits } bits
cofre-forca-bits = ≈ { $bits } bits

## Crear la bóveda

cofre-criar-kicker = 00 · Nueva bóveda
cofre-criar-kicker-nomeado = 00 · Nueva bóveda · { $nome }
cofre-criar-titulo = Una bóveda cifrada para las finanzas de la familia
cofre-criar-lead = Todo queda en un único archivo en esta computadora, cifrado con una clave derivada de tu contraseña maestra. Nada sale de aquí sin que lo pidas.
cofre-criar-card-titulo = Crear bóveda
cofre-criar-arquivo-unico = único
cofre-criar-nome-familia = Nombre de la familia
cofre-criar-nome-familia-exemplo = Familia García
cofre-criar-pais-dica = { $suporte } Moneda base: { $moeda }. Puedes cambiarlo después en Ajustes › Familia.
cofre-criar-membros = Miembros de la familia
cofre-criar-membros-ajuda = Quién aparece como titular de cuentas, ingresos y gastos en los informes. Puedes cambiarlo después.
cofre-criar-membro-n = Miembro { $n }
cofre-criar-membro-exemplo = Ana
cofre-criar-membro-placeholder = Nombre
cofre-criar-remover-membro = Quitar miembro { $n }
cofre-criar-adicionar-membro = Agregar miembro
cofre-criar-aviso-forte = Sin la contraseña maestra y sin el kit de recuperación, no hay forma de recuperar los datos.
cofre-criar-aviso = Treasury no guarda la contraseña ni tiene servidor: nadie, ni siquiera nosotros, puede abrir la bóveda sin ella. Anótala en un gestor de contraseñas y genera el kit en Ajustes.
cofre-criar-aviso-entendi = Entiendo que, si olvido la contraseña y no tengo el kit de recuperación, pierdo los datos.
cofre-criar-carregando = Derivando clave y cifrando…
cofre-criar-botao = Crear bóveda cifrada
cofre-criar-erro-nome-familia = Escribe el nombre de la familia.
cofre-criar-erro-sem-membros = Escribe al menos un miembro.
cofre-criar-erro-membros-repetidos = Hay miembros con el mismo nombre.
cofre-criar-erro-aviso = Confirma que entendiste el aviso de arriba.
cofre-criar-toast-titulo = Bóveda creada y cifrada
cofre-criar-toast-descricao = Guarda la contraseña maestra en un lugar seguro y genera el kit de recuperación en Ajustes.

## Desbloquear la bóveda

cofre-desbloquear-cofre-padrao = Bóveda de Treasury
cofre-desbloquear-kicker = 00 · Bóveda bloqueada
cofre-desbloquear-kicker-nomeado = 00 · Bóveda bloqueada · { $nome }
cofre-desbloquear-lead-senha = La bóveda está cifrada en reposo. Escribe la contraseña maestra para abrir los datos de esta familia.
cofre-desbloquear-lead-pin = El PIN solo vale en esta computadora. Cinco errores desactivan el PIN y piden la contraseña maestra.
cofre-desbloquear-lead-recuperacao = Usa el kit, dos de las tres partes de recuperación o la llave de seguridad para abrir la bóveda y definir una nueva contraseña maestra.
cofre-desbloquear = Desbloquear
cofre-desbloquear-touch-id = Desbloquear con Touch ID
cofre-desbloquear-senha-mac = Desbloquear con la contraseña de la Mac
cofre-desbloquear-derivando = Derivando clave…
cofre-desbloquear-erro-vazia = Escribe la contraseña maestra.
cofre-desbloquear-esqueceu = ¿Olvidaste la contraseña?
cofre-auto-bloqueio = Bloqueo automático
cofre-auto-bloqueio-minutos = { $minutos } min
cofre-keychain = Llavero
cofre-ligado = activado
cofre-desligado = desactivado
cofre-usar-pin = Usar PIN
cofre-pin-titulo = PIN
cofre-pin-campo = PIN de esta computadora
cofre-pin-conferindo = Verificando…
cofre-usar-senha-mestra = Usar la contraseña maestra

## Recuperar la bóveda

cofre-recuperar = Recuperar la bóveda
cofre-recuperar-como = Cómo recuperar
cofre-recuperar-metodo-kit = Kit
cofre-recuperar-metodo-partes = Partes
cofre-recuperar-metodo-chave = Llave de seguridad
cofre-recuperar-codigo = Código de recuperación
cofre-recuperar-codigo-mock = Mock: usa { $codigo }
cofre-recuperar-codigo-dica = El código que imprimiste o exportaste cuando generaste el kit.
cofre-recuperar-primeira-parte = Primera parte
cofre-recuperar-segunda-parte = Segunda parte
cofre-recuperar-partes-dica = Dos cualesquiera de las tres partes (TRSP-…).
cofre-recuperar-chave = Llave de seguridad
cofre-recuperar-pin-chave = PIN de la llave de seguridad (si tiene)
cofre-recuperar-pin-chave-dica = Toca la llave cuando parpadee.
cofre-recuperar-toque-chave = Toca la llave de seguridad…
cofre-recuperar-recifrando = Volviendo a cifrar la bóveda…
cofre-recuperar-botao = Restablecer contraseña y abrir
cofre-recuperar-voltar = Volver a la contraseña
cofre-recuperar-erro-codigo = Escribe el código del kit de recuperación.
cofre-recuperar-erro-partes = Escribe dos de las tres partes.
cofre-recuperar-erro-sem-chave = No hay ninguna llave de seguridad registrada.
cofre-recuperar-toast-titulo = Contraseña maestra restablecida
cofre-recuperar-toast-descricao = Los otros métodos de recuperación siguen valiendo. Guarda la nueva contraseña.

## Lista de bóvedas (VaultSwitcher) y menú lateral (VaultMenu)

cofre-menu-titulo = Bóvedas
cofre-menu-descricao = Cada bóveda es un archivo con su propia contraseña, desbloqueo rápido y copias de seguridad. Cambiar cierra la bóveda actual.
cofre-menu-cofre-atual = Bóveda: { $nome }
cofre-menu-trocar = Cambiar o crear bóveda
cofre-lista-rotulo = Bóvedas
cofre-novo-cofre = Nueva bóveda
cofre-abrir-existente = Abrir archivo existente
cofre-demonstracao = Bóveda de demostración
cofre-outros = Otras bóvedas
cofre-outros-n = Otras bóvedas ({ $n })
cofre-em-uso = en uso
cofre-ainda-nao-criado = aún no creada
cofre-acoes-de = Acciones de { $nome }
cofre-mais-acoes = Más acciones (o haz clic derecho)
cofre-acao-abrir = Abrir
cofre-acao-renomear = Renombrar y finalidad
cofre-acao-copiar = Crear una copia
cofre-acao-copiar-indisponivel = La bóveda aún no se ha creado.
cofre-acao-copiar-caminho = Copiar la ruta del archivo
cofre-acao-remover = Quitar…
cofre-acao-remover-indisponivel = Esta es la bóveda en uso: cambia de bóveda antes de quitarla.
cofre-copia-toast-titulo = Copia creada
cofre-copia-toast-descricao = { $nome } (copia), con la misma contraseña.
cofre-caminho-copiado = Ruta copiada
cofre-nome = Nombre
cofre-finalidade = Finalidad
cofre-remover-titulo = ¿Quitar { $nome }?
cofre-remover-descricao = Sus copias de seguridad se quedan donde están.
cofre-remover-confirmar-lixeira = Quitar y mover a la Papelera
cofre-remover-confirmar-lista = Quitar de la lista
cofre-remover-opcoes = Qué hacer con el archivo
cofre-remover-so-lista = Solo quitar de la lista
cofre-remover-so-lista-dica = El archivo sigue en { $caminho }.
cofre-remover-lixeira = Quitar de la lista y mover el archivo a la Papelera
cofre-remover-lixeira-dica = Puedes recuperarlo desde la Papelera mientras no se vacíe.
cofre-remover-toast-titulo = Se quitó { $nome }
cofre-remover-toast-lixeira = El archivo se movió a la Papelera.
cofre-remover-toast-lista = El archivo sigue en su lugar.
cofre-novo-nome = Nombre de la bóveda
cofre-novo-nome-exemplo = Ej.: Empresa, Casa de los padres
cofre-novo-finalidade = Finalidad (opcional)
cofre-novo-ajuda = La bóveda nueva tiene su propia contraseña, desbloqueo rápido y copias de seguridad. Después defines su contraseña.
cofre-novo-botao = Crear bóveda
cofre-existente-arquivo = Archivo de la bóveda (.db)
cofre-existente-arquivo-dica = La ruta completa, como /Users/ana/bovedas/empresa.db.
cofre-existente-escolher = Elegir…
cofre-existente-nome = Nombre (opcional)
cofre-existente-abrir = Abrir este archivo

## País de la familia (CountryPicker)

cofre-pais-rotulo = País donde vive la familia
cofre-pais-legenda = ★ con adaptación propia; los demás países usan el catálogo de índices.
cofre-pais-suporte-completo = Todo lo de la app: B3, Tesouro Direto, fondos de la CVM, IR con DARF, INSS e IRRF en el recibo de sueldo, FIPE y tasas del Banco Central.
cofre-pais-suporte-adaptado = Moneda, inflación y tasas de interés del país, nómina (descuentos y pagos extra), estimación de impuestos, y categorías y reglas listas de allá; lo que es solo de Brasil queda desactivado.
cofre-pais-suporte-generico = Moneda, inflación, tasas de interés y tipo de cambio del país desde el catálogo de índices; sin reglas locales de impuestos ni nómina, y sin lo que es solo de Brasil.

## Fuerza de la contraseña (lib/passwordStrength.ts)

cofre-forca-muito-fraca = Muy débil
cofre-forca-fraca = Débil
cofre-forca-razoavel = Aceptable
cofre-forca-boa = Buena
cofre-forca-forte = Fuerte
cofre-sugestao-longa = Usa al menos 12 caracteres: las contraseñas largas resisten mucho más.
cofre-sugestao-nomes = No uses el nombre de la familia ni de sus miembros.
cofre-sugestao-comuns = Evita palabras y contraseñas comunes, incluso con cambios como 4 en lugar de a.
cofre-sugestao-anos = Evita años y fechas, que son fáciles de adivinar.
cofre-sugestao-sequencias = Evita secuencias como abc, 123 o qwerty.
cofre-sugestao-repetir = Evita repetir el mismo carácter.
cofre-sugestao-frase = Una frase con 4 o más palabras sin relación entre sí es fuerte y fácil de recordar.

cofre-so-no-app = Disponible solo en la app Treasury.

## Demostración sin contraseña

cofre-demo-kicker = 00 · Demostración · { $nome }
cofre-demo-lead = Bóveda de demostración con datos ficticios: se abre sin contraseña.
cofre-demo-aviso = Cualquiera con la app puede abrir esta bóveda. Úsala solo para probar, nunca con datos reales.
cofre-demo-abrir = Abrir la demostración

## Crear: bóveda real o demostración

cofre-escolha-lead = Para empezar, elige: la bóveda de la familia, con contraseña maestra, o una demostración para conocer la app.
cofre-escolha-rotulo = Qué bóveda crear
cofre-escolha-real = Bóveda real
cofre-escolha-real-dica = Para las finanzas de la familia: cifrada con tu contraseña maestra, solo tú la abres.
cofre-escolha-demo = Demostración
cofre-escolha-demo-dica = Datos ficticios de una familia de ejemplo, listos en segundos y sin contraseña.
cofre-escolha-demo-titulo = Bóveda de demostración
cofre-escolha-demo-lead = Elige el país y el perfil de la familia de ejemplo. La bóveda se crea y se abre enseguida.

## Recuperar desde una copia de seguridad (docs/backup-recovery.md)

cofre-desbloquear-lead-backup = Elige una copia de seguridad guardada en esta computadora y restaura la bóveda desde ella. La bóveda que no abre no se borra: queda guardada al lado.
cofre-desbloquear-nao-abre = ¿La bóveda no abre?
cofre-backup-abrir-recuperacao = Recuperar desde una copia
cofre-backup-recuperar-titulo = Recuperar desde una copia de seguridad
cofre-backup-motivo = La bóveda no abrió: { $erro }
cofre-backup-recuperar-texto = Estas copias están en la carpeta de esta bóveda, de la más nueva a la más antigua. Elige una e ingresa la contraseña.
cofre-backup-procurando = Buscando las copias…
cofre-backup-nenhum = No hay copias en la carpeta de esta bóveda. Si tienes una copia en otro lugar (pendrive, disco externo, nube), usa "Traer una copia de afuera".
cofre-backup-lista = Copias encontradas
cofre-backup-tipo-desconhecido = Copia de seguridad
cofre-backup-importado = { $tipo } (traída de afuera)
cofre-backup-salvar-copia = Guardar una copia
cofre-backup-salvar-copia-de = Guardar una copia de la copia del { $data }
cofre-backup-copia-titulo = Guardar una copia de la copia de seguridad
cofre-backup-copia-salva = Copia guardada
cofre-backup-copia-erro = No se pudo guardar la copia
cofre-backup-copia-so-app = Guardar una copia solo funciona en la app instalada.
cofre-backup-trazer = Traer una copia de afuera
cofre-backup-trazer-titulo = Elige el archivo de la copia (.db.enc)
cofre-backup-trazido = Copia traída a la carpeta de la bóveda
cofre-backup-trazer-erro = No se pudo traer ese archivo
cofre-backup-senha-mestra = Contraseña maestra
cofre-backup-senha-mestra-dica = Vale para la bóveda restaurada. Normalmente es la contraseña que ya usas.
cofre-backup-outra-senha = La copia se hizo con otra contraseña (la anterior o la contraseña de copias propia)
cofre-backup-senha-backup = Contraseña de la copia
cofre-backup-ao-lado-aviso = La bóveda actual queda al lado, con "dañada" y la fecha en el nombre, y no se borra.
cofre-backup-restaurar = Restaurar esta copia
cofre-backup-restaurando = Restaurando…
cofre-backup-restaurado = Bóveda restaurada desde la copia
cofre-backup-restaurado-ao-lado = La bóveda que no abría quedó guardada en { $caminho }.
