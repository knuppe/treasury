# Mercado: búsqueda de tickers, elección de activo, cotización intradía y bolsas
# (docs/i18n-design.md). Revisión: borrador por IA, sin revisión de un hablante nativo.

## Búsqueda de tickers

mercado-origem-cadastrado = registrado
mercado-origem-analise = análisis
mercado-origem-yahoo = Yahoo
mercado-busca-placeholder = Ticker o nombre (PETR4, Vodafone, 7203…)
mercado-busca-rotulo = Buscar ticker
mercado-sugestoes = Sugerencias de activos
mercado-bolsa-fora = bolsa fuera del catálogo
mercado-buscando = Buscando…
mercado-so-locais = { $erro }. Se muestran solo los activos locales.

## Elección de activo

mercado-ativo-novo = Activo nuevo
mercado-bolsa-fora-catalogo = La bolsa { $bolsa } no está en el catálogo: regístrala en Activos › Bolsas.
mercado-cadastrar-usar = Registrar y usar

## Cotización intradía

mercado-atraso = retraso de ~{ $min } min
mercado-agora = ahora
mercado-intradiaria = Cotización intradía
mercado-ultimo-negocio = Último negocio: { $quando }
mercado-ultimo-negocio-bolsa = Último negocio: { $quando } ({ $hora } en la bolsa)
mercado-fonte-yahoo = Yahoo Finance, sin clave, con unos { $min } min de retraso. El historial usa los cierres diarios.
mercado-cotacao-com = Cotización con { $atraso }
mercado-cotacao-com-estado = Cotización con { $atraso }, { $estado }

## Gráfico intradía

mercado-fech-anterior = Cierre ant.
mercado-buscando-cotacao = Buscando cotización…
mercado-sem-intradiaria = Todavía sin cotización intradía.
mercado-periodo-intradiario = Período intradía
mercado-periodo-1d = 1D
mercado-periodo-5d = 5D
mercado-nao-buscou = No se pudo obtener la cotización ahora.
mercado-intradiaria-aria = { $dias ->
    [one] Cotización intradía de { $nome } (1 día)
   *[other] Cotización intradía de { $nome } ({ $dias } días)
}
mercado-sem-pontos = Sin puntos intradía. Llegan con la sesión abierta (cada 5 min).

## Bolsas (Activos › Bolsas)

mercado-dias-seg-sex = lun–vie
mercado-dias-dom-qui = dom–jue
mercado-dias-curtos = lun,mar,mié,jue,vie,sáb,dom
mercado-almoco = {" "}(almuerzo { $inicio }–{ $fim })
mercado-bolsa = Bolsa
mercado-pais = País
mercado-moeda = Moneda
mercado-cambio-manual = tipo de cambio manual
mercado-cambio-manual-dica = Sin PTAX en el Banco Central ni tipo de referencia del BCE: informa el tipo de cambio manualmente.
mercado-sem-sufixo = sin sufijo
mercado-sem-sufixo-dica = El símbolo en Yahoo es el propio ticker
mercado-pregao-local = Sesión (local)
mercado-agora-col = Ahora
mercado-hora-local = { $hora } local
mercado-remover-bolsa = Quitar bolsa
mercado-remover-bolsa-descricao = { $codigo } sale del catálogo (solo si no hay activos en ella).
mercado-bolsa-removida = Bolsa quitada
mercado-filtrar-bolsas = Filtrar bolsas
mercado-filtrar-placeholder = Filtrar por código, país, moneda…
mercado-resumo = { $n ->
    [one] { $n } bolsa
   *[other] { $n } bolsas
} · { $abertas } en sesión ahora
mercado-cadastrar-bolsa = Registrar bolsa
mercado-bolsas-ajuda = El código de la bolsa es el “mercado” del activo: de él vienen la moneda y el país predeterminados, el símbolo de Yahoo (ticker + sufijo) y el horario de la recolección intradía (cada 5 min, con ~15 min de retraso, solo con la sesión abierta). Los feriados no están en el catálogo.
mercado-nenhuma-bolsa = Ninguna bolsa
mercado-nada-filtro = Nada coincide con el filtro.
mercado-dialogo-descricao = Horario de la sesión regular en la zona horaria de la bolsa (IANA, ej.: Europe/London).
mercado-erro-codigo = Hasta 12 letras y números.
mercado-erro-nome = Ingresa el nombre.
mercado-erro-pais = Código ISO de 2 letras.
mercado-erro-moeda = Código ISO de 3 letras.
mercado-erro-fuso = Ingresa la zona horaria (ej.: Asia/Manila).
mercado-bolsa-atualizada = Bolsa actualizada
mercado-bolsa-cadastrada = Bolsa registrada
mercado-iso-2 = ISO de 2 letras.
mercado-iso-3 = ISO de 3 letras.
mercado-sufixo = Sufijo en Yahoo
mercado-sufixo-dica = Ej.: .PS; vacío en EE. UU.
mercado-fuso = Zona horaria (IANA)
mercado-mic-dica = Opcional (ISO 10383).
mercado-abertura = Apertura
mercado-dias-pregao = Días de sesión
mercado-dias-pregao-dica = 1 = lunes … 7 = domingo.
mercado-almoco-inicio = Almuerzo: inicio
mercado-almoco-fim = Almuerzo: fin
mercado-apelidos = Alias
mercado-apelidos-dica = Separados por coma (códigos de Yahoo, nombres).
mercado-salvar-bolsa = Guardar bolsa

## Tela Mercado (docs/design/market-explorer.md)

mercado-secoes = Secciones del Mercado
mercado-aba-panorama = Panorama
mercado-aba-explorador = Explorador
mercado-aba-mapa = Mapa
mercado-aviso = Lista de quien cumple los criterios, a partir de datos públicos. No es recomendación de inversión.
mercado-panorama-titulo = El mercado hoy
mercado-acoes = Acciones
mercado-fiis = FII
mercado-todos = Todos
mercado-com-score = Con score
mercado-score-de = Score calculado el { $data }
mercado-score-pendente = El score del mercado sale en la próxima recogida
mercado-dados = Datos de
mercado-coleta-em = Recogida el { $data }
mercado-sem-coleta = La recogida todavía no se ejecutó
mercado-panorama-de = Panorama de
mercado-acoes-por-setor = Acciones por sector
mercado-fiis-por-segmento = FII por segmento
mercado-col-setor = Sector
mercado-col-segmento = Segmento
mercado-col-papeis = Valores
mercado-col-saudaveis = Robustos y saludables
mercado-col-score = Score (mediana)
mercado-col-pl = P/U (mediana)
mercado-col-pvp = P/VL (mediana)
mercado-col-dy = DY (mediana)
mercado-panorama-rodape = Medianas del sector (P/U y P/VL solo con valores positivos). Haz clic en el nombre de un sector (o doble clic en la fila) para abrir el explorador con sus valores saludables.
mercado-sem-universo = El universo todavía no se descargó
mercado-sem-universo-texto = Ejecuta la recogida del análisis en Activos › Datos para traer las acciones y los FII de la B3.
mercado-mapa-titulo = Salud × Atractivo
mercado-mapa-saude = Salud
mercado-mapa-atratividade = Atractivo
mercado-mapa-classe = Clase en el mapa
mercado-mapa-aria = Mapa de { $n } valores: salud en el eje vertical, atractivo en el horizontal
mercado-mapa-vazio = Todavía sin score del mercado
mercado-mapa-vazio-texto = El mapa usa el score que la recogida calcula para todo el universo. Ejecuta la recogida en Activos › Datos.
mercado-mapa-leitura = Cómo leer el mapa
mercado-mapa-leitura-texto = Cada burbuja es un valor, del tamaño del valor de mercado y del color de la franja del score. La esquina de arriba a la derecha junta salud y atractivo altos. El contorno marca los valores de la cartera y de la watchlist. Haz clic para abrir el valor.
mercado-aba-metodos = Métodos
mercado-metodo-criterios = Criterios
mercado-metodo-opcional = (sin el dato, no excluye)
mercado-metodo-adaptado = Adaptado a la B3. Original: { $original }
mercado-metodo-higiene = Higiene
mercado-metodo-ordem = Orden
mercado-metodo-nao-se-aplica = No se aplica a
mercado-metodo-fonte = Fuente
mercado-col-score-papel = Score
mercado-metodo-por-que = Por qué un valor cumple o queda fuera
mercado-metodo-por-que-exemplo = Ej.: PETR4
mercado-metodo-explicar = Explicar
mercado-metodo-passa = { $ticker } cumple todos los criterios ({ $rank }º en el orden del método).
mercado-metodo-fora = { $ticker } queda fuera antes de los criterios: { $motivo }.
mercado-metodo-nao-passa = { $ticker } no cumple todos los criterios:
mercado-metodo-sem-dado = sin el dato
mercado-metodo-voltar = Todos los métodos
mercado-metodo-lista = Quién cumple { $metodo }
mercado-metodo-resultado = { $passam } de { $avaliados } valores evaluados cumplen todos los criterios hoy.
mercado-metodo-ninguem = Ningún valor cumple todos los criterios hoy.
mercado-metodo-cumprem = { $n } valor(es) cumplen los criterios hoy
mercado-metodo-contando = Contando…
mercado-metodo-ver-lista = Ver la receta y la lista
mercado-metodo-fonte-versao = { $fonte } · v{ $versao }
mercado-abrir-setor = Abrir { $setor } en el explorador
