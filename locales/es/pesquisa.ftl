# Análisis de activos: indicadores, precio calculado, historial, documentos, rankings, screener,
# comparador y datos del análisis (docs/i18n-design.md). Los nombres y fórmulas de los
# indicadores vienen del núcleo. Revisión: borrador por IA, sin revisión nativa.

## Sección del análisis y análisis por ticker

pesquisa-analise-fundamentalista = Análisis fundamental
pesquisa-analise-do-ativo = Análisis del activo
pesquisa-aba-indicadores = Indicadores
pesquisa-aba-preco-justo = Precio calculado
pesquisa-aba-historico = Historial
pesquisa-aba-documentos = Documentos
pesquisa-analisar-papel = Analizar un valor
pesquisa-ticker-para-analisar = Ticker para analizar
pesquisa-ticker-exemplo-analise = Ticker (p. ej.: PETR4, HGLG11, AAPL)
pesquisa-analisar = Analizar
pesquisa-n-papeis-universo = { $n ->
    [one] { $n } valor en el universo
   *[other] { $n } valores en el universo
}
pesquisa-abrir-pagina-ativo = Abrir la página del activo (posición y operaciones)
pesquisa-escolha-papel = Elige un valor
pesquisa-escolha-papel-texto = Escribe el ticker de cualquier acción o FII del universo descargado (registrado o no) para ver indicadores, precio calculado, historial y hechos relevantes.

## Universo vacío y activo sin datos

pesquisa-sem-coleta = Sin recolección
pesquisa-sem-universo = Todavía no hay valores en el universo
pesquisa-sem-universo-texto = { $quem ->
    [ticker] El análisis por ticker funciona sobre el universo de acciones y FII descargado de la CVM, la B3 y la SEC. Haz la primera recolección (unos 230 MB, de 1 a 2 minutos) en Datos del análisis; después todo funciona sin conexión.
    [screener] El screener funciona sobre el universo de acciones y FII descargado de la CVM, la B3 y la SEC. Haz la primera recolección (unos 230 MB, de 1 a 2 minutos) en Datos del análisis; después todo funciona sin conexión.
    [rankings] Los rankings funcionan sobre el universo de acciones y FII descargado de la CVM, la B3 y la SEC. Haz la primera recolección (unos 230 MB, de 1 a 2 minutos) en Datos del análisis; después todo funciona sin conexión.
   *[geral] Los rankings y el screener funcionan sobre el universo de acciones y FII descargado de la CVM, la B3 y la SEC. Haz la primera recolección (unos 230 MB, de 1 a 2 minutos) en Datos del análisis; después todo funciona sin conexión.
}
pesquisa-ir-dados = Ir a Datos del análisis
pesquisa-dados-analise = Datos del análisis
pesquisa-sem-dados-ativo = No hay datos del análisis para este activo
pesquisa-sem-dados-ativo-texto = { $erro }. Los fundamentos vienen de la recolección de la CVM, la B3 y la SEC.

## Grupos de indicadores

pesquisa-grupo-mercado = Mercado
pesquisa-grupo-valuation = Valuación
pesquisa-grupo-rentabilidade = Rentabilidad
pesquisa-grupo-endividamento = Endeudamiento
pesquisa-grupo-crescimento = Crecimiento
pesquisa-grupo-por-acao = Por acción
pesquisa-grupo-dividendos = Dividendos
pesquisa-grupo-preco-justo = Precio calculado
pesquisa-grupo-fii = Fondo inmobiliario
pesquisa-grupo-bancos = Bancos
pesquisa-grupo-score = Score del Treasury

## Operadores y campos del screener

pesquisa-logica-e = Y
pesquisa-logica-ou = O
pesquisa-op-entre = entre
pesquisa-op-um-de = es uno de
pesquisa-op-nao-um-de = no es uno de
pesquisa-op-contem = contiene
pesquisa-op-disponivel = disponible
pesquisa-op-indisponivel = no disponible
pesquisa-campo-ticker = Ticker
pesquisa-campo-nome = Nombre
pesquisa-campo-classe = Clase
pesquisa-campo-setor = Sector
pesquisa-campo-segmento = Segmento
pesquisa-campo-moeda = Moneda
pesquisa-campo-faixa-score = Franja del score (robusto, saudavel, atencao, risco)
pesquisa-campo-red-flags = Red flags del score
pesquisa-campo-na-carteira = En la cartera
pesquisa-campo-na-watchlist = En la watchlist

## Fuentes y estado de la recolección

pesquisa-fonte-fca = CVM FCA · registro y tickers
pesquisa-fonte-dfp = CVM DFP · estados anuales
pesquisa-fonte-itr = CVM ITR · trimestres
pesquisa-fonte-fii = CVM · informe mensual de FII
pesquisa-fonte-fii-trimestral = CVM · informe trimestral de FII
pesquisa-fonte-ipe = CVM IPE · hechos relevantes
pesquisa-fonte-b3 = B3 COTAHIST · cotizaciones
pesquisa-fonte-b3-listados = B3 empresas y fondos listados · dividendos oficiales
pesquisa-fonte-sec = SEC EDGAR · empresas de EE. UU.
pesquisa-fonte-bacen = Banco Central de Brasil IF.data · bancos
pesquisa-fonte-fundos = CVM y B3 · registro de fondos y ETF
pesquisa-fonte-cripto = CoinGecko y DefiLlama · criptoactivos
pesquisa-fonte-brapi = brapi.dev · verificación (opcional)
pesquisa-situacao-atualizado = Actualizado
pesquisa-situacao-sem-mudanca = Sin cambios
pesquisa-situacao-inexistente = Inexistente
pesquisa-situacao-pulado = Omitido
pesquisa-situacao-erro = Error

## Valores compactos (el número ya viene formateado, con la moneda)

pesquisa-compacto-tri = { $valor } B
pesquisa-compacto-bi = { $valor } mil M
pesquisa-compacto-mi = { $valor } M
pesquisa-compacto-mil = { $valor } mil

## Indicadores

pesquisa-formula = Fórmula
pesquisa-indisponivel = No disponible
pesquisa-periodo = Período
pesquisa-fonte = Fuente
pesquisa-status = Estado
pesquisa-estimado = ≈ Estimado
pesquisa-conferencia = Verificación de los datos
pesquisa-conferencia-conferido = Datos de entrada verificados
pesquisa-conferencia-corrigido = Corregido con otra fuente oficial
pesquisa-conferencia-aviso = Con aviso
pesquisa-conferencia-bloqueado = Bloqueado: dato de entrada inválido
pesquisa-conferencia-marca-corrigido = { $indicador }: corregido con otra fuente oficial
pesquisa-conferencia-marca-aviso = { $indicador }: con un aviso sobre los datos
pesquisa-indicador-indisponivel = { $indicador }: no disponible
pesquisa-indisponivel-motivo = No disponible: { $motivo }
pesquisa-calculando-indicadores = Calculando indicadores
pesquisa-cotacao = Cotización
pesquisa-cotacao-anterior = { $cotacao } (anterior)
pesquisa-sem-cotacao = sin cotización
pesquisa-fluxos = Flujos
pesquisa-fluxos-12-meses = 12 meses hasta { $data }
pesquisa-informe-mensal = informe mensual
pesquisa-calculado-em = Calculado el
pesquisa-notas-calculo = Notas del cálculo
pesquisa-indicadores-grupo = Indicadores · { $grupo }
pesquisa-indicadores-dica = Pasa el mouse o haz clic en el nombre de un indicador para ver la fórmula, el período y la fuente. La utilidad y el patrimonio son los de los controladores; flujos de los últimos 12 meses (DFP + ITR).

## Precio calculado

pesquisa-preco-justo-metodo = Precio calculado · { $metodo }
pesquisa-preco-justo-de = Precio calculado ({ $metodo })
pesquisa-margem-cotacao = Margen hasta la cotización
pesquisa-cotacao-valor = cotización { $valor }
pesquisa-premissas = Supuestos · { $metodo }
pesquisa-premissa = Supuesto
pesquisa-valor = Valor
pesquisa-nao-recomendacao = No es una recomendación de inversión
pesquisa-aviso-estimativas = Estimaciones a partir de fórmulas clásicas y de los datos informados; no son una recomendación de inversión.
pesquisa-taxa-minima-bazin = Tasa mínima (Bazin)
pesquisa-taxa-invalida = Ingresa una tasa entre 0 y 100%.
pesquisa-padrao-ajustes = predeterminado de los ajustes
pesquisa-media-dividendos = Promedio de dividendos
pesquisa-n-anos = { $n ->
    [one] { $n } año
   *[other] { $n } años
}
pesquisa-usar-padrao = Usar el predeterminado
pesquisa-calculando-preco-justo = Calculando el precio
pesquisa-cotacao-em = { $preco } el { $data } · { $fonte }

## Documentos

pesquisa-doc-entregue = Entregado
pesquisa-doc-categoria = Categoría
pesquisa-doc-assunto = Asunto
pesquisa-doc-documento = Documento
pesquisa-doc-sem-assunto = (sin asunto informado)
pesquisa-doc-abrir-cvm-rotulo = Abrir en la CVM: { $categoria } del { $data }
pesquisa-doc-abrir-cvm = Abrir en la CVM ↗
pesquisa-doc-categorias = Categoría de los documentos
pesquisa-doc-fatos = Hechos relevantes
pesquisa-doc-comunicados = Comunicados
pesquisa-doc-avisos = Avisos
pesquisa-doc-periodo = Últimos 12 meses · CVM IPE
pesquisa-doc-vazio = No hay documentos en el período
pesquisa-doc-vazio-texto = Nada en las categorías y el período elegidos.
pesquisa-doc-tabela = Documentos de la compañía
pesquisa-doc-rodape = El asunto y el resumen son textos de la propia compañía, mostrados tal cual. El documento oficial se abre en el sitio de la CVM.

## Historial

pesquisa-hist-receita-liquida = Ingresos netos
pesquisa-hist-lucro-liquido = Utilidad neta
pesquisa-hist-margem-ebitda = Margen EBITDA
pesquisa-hist-margem-liquida = Margen neto
pesquisa-hist-roe = ROE
pesquisa-hist-dividendos-acao = Dividendos por acción
pesquisa-hist-rendimento-cota = Rendimiento por cuota
pesquisa-hist-vp-cota = Valor patrimonial por cuota
pesquisa-hist-carregando = Cargando historial
pesquisa-hist-vazio = No hay estados descargados para este activo
pesquisa-hist-vazio-texto = El historial viene de los DFP de la CVM (o del XBRL de la SEC). Haz la recolección en Datos del análisis.
pesquisa-hist-resultado-distribuivel = Resultado distribuible por ejercicio
pesquisa-hist-receita-lucro = Ingresos y utilidad por ejercicio
pesquisa-hist-fonte-fii = Fuente: informes mensuales de la CVM · utilidad de los controladores · { $moeda }
pesquisa-hist-fonte-dfp = Fuente: DFP de la CVM / 10-K de la SEC · utilidad de los controladores · { $moeda }
pesquisa-hist-receita-lucro-rotulo = Ingresos y utilidad neta de { $nome } por ejercicio, { $periodo }.
pesquisa-hist-margens = Márgenes y ROE
pesquisa-hist-margens-fonte = Margen = resultado ÷ ingresos netos; ROE = utilidad ÷ patrimonio neto al cierre del ejercicio
pesquisa-hist-margens-rotulo = Márgenes y ROE de { $nome } por ejercicio, { $periodo }.
pesquisa-hist-margens-tabela = Márgenes y ROE por ejercicio
pesquisa-hist-dividendos = Dividendos por acción por año
pesquisa-hist-dividendos-fonte = Dividendos por acción con fecha de corte en el año (agenda); sin agenda, los declarados en el DFP
pesquisa-hist-dividendos-rotulo = Dividendos por acción de { $nome } por año, { $periodo }.
pesquisa-hist-rendimento = Rendimiento por cuota, 24 meses
pesquisa-hist-fonte-informe-fii = Fuente: informe mensual de FII (CVM)
pesquisa-hist-rendimento-rotulo = Rendimiento mensual por cuota de { $nome }, 24 meses.
pesquisa-hist-rendimento-tabela = Rendimiento por cuota por mes
pesquisa-hist-vp = Valor patrimonial por cuota, 24 meses
pesquisa-hist-vp-rotulo = Valor patrimonial por cuota de { $nome }, 24 meses.
pesquisa-hist-vp-tabela = Valor patrimonial por cuota por mes

## Comparador

pesquisa-cmp-tabela = Comparación de indicadores
pesquisa-cmp-indicador = Indicador
pesquisa-cmp-melhor = (mejor)
pesquisa-cmp-sem-historico = No hay historial de cotizaciones para comparar
pesquisa-cmp-sem-historico-texto = La cotización normalizada usa el historial de los activos registrados (Yahoo Finance). Los valores que solo están en el universo tienen cotización de pocas sesiones.
pesquisa-cmp-cotacao-normalizada = Cotización normalizada (base 100), 12 meses
pesquisa-cmp-fonte = Base 100 el { $data }, primera sesión con cotización de todos · cierre sin ajuste por dividendos
pesquisa-cmp-fonte-faltando = { $fonte } · sin historial: { $tickers }
pesquisa-cmp-grafico-rotulo = Cotización normalizada en base 100 de { $tickers } desde el { $data }.
pesquisa-cmp-tabela-mensal = Cotización normalizada por mes
pesquisa-cmp-data = Fecha
pesquisa-cmp-ja-esta = { $ticker } ya está en la comparación.
pesquisa-cmp-maximo = Compara como máximo { $n } activos.
pesquisa-cmp-fora-universo = { $ticker } no está en el universo descargado.
pesquisa-cmp-adicionar-rotulo = Agregar a la comparación
pesquisa-cmp-ticker = Ticker para comparar
pesquisa-cmp-ticker-exemplo = Ticker (p. ej.: VALE3)
pesquisa-cmp-adicionar = Agregar
pesquisa-cmp-papeis = Valores comparados
pesquisa-cmp-tirar = Quitar { $ticker } de la comparación
pesquisa-cmp-escolha = Elige de 2 a 6 valores
pesquisa-cmp-escolha-texto = Acciones, FII y acciones estadounidenses del universo, lado a lado: indicadores con el mejor de cada fila destacado y la cotización normalizada en base 100.
pesquisa-cmp-exemplo = Ejemplo: PETR4 × VALE3 × WEGE3
pesquisa-cmp-comparando = Comparando
pesquisa-cmp-rodape = ◆ mejor valor de la fila según el orden natural del indicador (menor P/U, mayor DY…); las filas sin orden natural (cotización, valor de la empresa) no se destacan. Valores en la moneda de cada valor. Calculado el { $data }. No es una recomendación de inversión.

## Filtro del universo

pesquisa-recorte = Filtro del universo
pesquisa-classes = Clases
pesquisa-so-carteira = Solo cartera y watchlist
pesquisa-setor-contem = Sector contiene
pesquisa-varios-virgula = Varios: sepáralos con comas
pesquisa-setor-exemplo = p. ej.: energia, financeiro
pesquisa-segmento-contem = Segmento contiene
pesquisa-segmento-dica = FII: logística, shoppings, papel…
pesquisa-segmento-exemplo = p. ej.: bancos, logística
pesquisa-liquidez-minima = Liquidez diaria mínima
pesquisa-liquidez-dica = En la moneda del activo (R$ en la B3, USD en EE. UU.)
pesquisa-liquidez-invalida = Ingresa un valor (p. ej.: 1.000.000).

## Constructor de condiciones

pesquisa-tipo-texto = Texto
pesquisa-tipo-sim-nao = Sí/no
pesquisa-logica-todas = Todas (Y)
pesquisa-logica-todas-dica = El valor debe cumplir todas
pesquisa-logica-qualquer = Cualquiera (O)
pesquisa-logica-qualquer-dica = Basta con cumplir una
pesquisa-cond-n-grupo = { $n } del grupo { $grupo }
pesquisa-cond-campo = Campo de la condición { $n }
pesquisa-cond-operador = Operador de la condición { $n }
pesquisa-cond-valor = Valor de la condición { $n }
pesquisa-cond-minimo = Mínimo de la condición { $n }
pesquisa-cond-maximo = Máximo de la condición { $n }
pesquisa-cond-remover = Quitar la condición { $n }
pesquisa-cond-classe = Clase
pesquisa-cond-min = mín.
pesquisa-cond-max = máx.
pesquisa-cond-lista = a, b, c
pesquisa-cond-texto = texto
pesquisa-cond-e = y
pesquisa-combinar = Combinar
pesquisa-combinar-tudo = Combinar condiciones y grupos
pesquisa-sem-condicoes = Sin condiciones: todos los valores del filtro.
pesquisa-condicoes = Condiciones
pesquisa-grupo-n = Grupo { $n }
pesquisa-combinar-grupo = Combinar condiciones del grupo { $n }
pesquisa-remover-grupo = Quitar grupo
pesquisa-condicoes-grupo = Condiciones del grupo { $n }
pesquisa-condicao-no-grupo = Condición en el grupo
pesquisa-adicionar-condicao = Agregar condición
pesquisa-adicionar-grupo = Agregar grupo (O…)

## Errores de los filtros

pesquisa-erro-operador = El operador no aplica a este campo.
pesquisa-erro-lista = Ingresa uno o más valores separados por comas.
pesquisa-erro-texto = Ingresa el texto.
pesquisa-erro-numero = Ingresa un número (p. ej.: 10 o 6,5).
pesquisa-erro-maximo = Ingresa el máximo.
pesquisa-erro-maximo-menor = El máximo es menor que el mínimo.
pesquisa-erro-grupo-vazio = Grupo vacío: agrega una condición.

## Screener

pesquisa-scr-salvar-titulo = Guardar vista del screener
pesquisa-scr-salvar-texto = La vista guarda los filtros y el filtro del universo; ejecútala de nuevo cuando quieras (los datos del día cambian).
pesquisa-scr-salvar-rotulo = Guardar vista
pesquisa-scr-nome = Nombre
pesquisa-scr-informe-nome = Ingresa el nombre.
pesquisa-scr-descricao = Descripción (opcional)
pesquisa-scr-salvar = Guardar vista
pesquisa-scr-colunas = Columnas del resultado ({ $n })
pesquisa-scr-tela-salva = Vista “{ $nome }” guardada
pesquisa-scr-tela-atualizada = Vista “{ $nome }” actualizada
pesquisa-scr-tela-excluida = Vista “{ $nome }” eliminada
pesquisa-scr-erro-excluir = No se pudo eliminar
pesquisa-scr-desfeita = Cambio deshecho
pesquisa-scr-erro-desfazer = No se pudo deshacer
pesquisa-scr-ticker = Ticker
pesquisa-scr-nome-col = Nombre
pesquisa-scr-universo-vazio = El universo todavía está vacío
pesquisa-scr-universo-vazio-texto = Ya puedes armar y guardar vistas; los resultados aparecen después de la primera recolección en Datos del análisis.
pesquisa-scr-tela = Vista · { $nome }
pesquisa-scr-filtros = Filtros
pesquisa-scr-telas-salvas = Vistas guardadas
pesquisa-scr-abrir-tela = Abrir vista guardada…
pesquisa-scr-nenhuma-tela = Ninguna vista guardada
pesquisa-scr-salva-em = Guardada: { $resumo } · modificada el { $data }
pesquisa-ordenar-por = Ordenar por
pesquisa-scr-ordem = Orden del resultado
pesquisa-ordem-natural = Natural
pesquisa-maior-primeiro = Mayor primero
pesquisa-menor-primeiro = Menor primero
pesquisa-scr-rodar = Ejecutar screener
pesquisa-scr-salvar-alteracoes = Guardar cambios
pesquisa-scr-salvar-nova = Guardar como nueva…
pesquisa-scr-salvar-tela = Guardar vista…
pesquisa-scr-nova = Nueva vista
pesquisa-scr-excluir = Eliminar vista
pesquisa-scr-excluir-texto = La vista “{ $nome }” sale de la lista. Puedes deshacerlo enseguida o en Ajustes › Historial de cambios.
pesquisa-scr-corrija = Corrige los filtros marcados.
pesquisa-scr-desfazer = Deshacer
pesquisa-scr-desfazer-texto = También puedes deshacerlo en Ajustes › Historial de cambios.
pesquisa-scr-instrucoes = Arma los filtros y haz clic en { $botao }. Porcentajes en % (DY ≥ 6), múltiplos en veces (P/U < 12), dinero en la moneda del activo.
pesquisa-scr-rodando = Ejecutando el screener
pesquisa-scr-resultado = Resultado del screener
pesquisa-scr-n-papeis = { $n ->
    [one] { $n } valor
   *[other] { $n } valores
}
pesquisa-universo-de = universo del { $data }
pesquisa-scr-nenhum = Ningún valor pasó los filtros.
pesquisa-scr-mostrando = Se muestran los primeros { $n }; ajusta los filtros para ver el resto.

## Rankings

pesquisa-rank-maiores-dy = Mayores DY
pesquisa-rank-menor-pl = Menor P/U
pesquisa-rank-margem-graham = Margen hasta Graham
pesquisa-rank-margem-bazin = Margen hasta Bazin
pesquisa-rank-sem-prejuizo = Nunca tuvo pérdidas
pesquisa-rank-maior-roe = Mayor ROE
pesquisa-rank-valor-mercado = Valor de mercado
pesquisa-rank-liquidez-fii = Liquidez de FII
pesquisa-rank-pvp-fii = P/VL de FII
pesquisa-rank-menor-primeiro = menor primero
pesquisa-rank-maior-primeiro = mayor primero
pesquisa-rank-nome = Nombre
pesquisa-rank-liquidez = Liquidez diaria
pesquisa-rank-prontos = Rankings listos
pesquisa-rank-indicador = Indicador
pesquisa-rank-ordem = Orden
pesquisa-rank-ordem-rotulo = Orden del ranking
pesquisa-rank-mostrar = Mostrar
pesquisa-rank-ordem-natural = orden natural: { $ordem }
pesquisa-rank-montando = Armando ranking
pesquisa-rank-tabela = Ranking · { $indicador }
pesquisa-rank-vazio = Ningún valor del filtro tiene este indicador.
pesquisa-rank-total = { $n ->
    [one] { $n } valor con el indicador
   *[other] { $n } valores con el indicador
}
pesquisa-rank-sem-valor = { $n } sin valor
pesquisa-rank-sem-valor-multiplo = { $n } sin valor (o ≤ 0, que no cuenta como barato)
pesquisa-rank-rodape = universo del { $data } · moneda de cada valor (B3 en R$, EE. UU. en USD). No es una recomendación de inversión.

## Datos del análisis

pesquisa-dados-coleta-fonte = Recolección por fuente
pesquisa-dados-fonte = Fuente
pesquisa-dados-arquivos = Archivos
pesquisa-dados-situacao = Estado
pesquisa-dados-baixado = Descargado
pesquisa-dados-linhas = Filas
pesquisa-dados-com-problema = { $n } con problemas
pesquisa-dados-n-atualizados = { $n ->
    [one] { $n } actualizado
   *[other] { $n } actualizados
}
pesquisa-dados-n-ok = { $n } ok
pesquisa-dados-n-pulados = { $n } omitido(s), sin problema
pesquisa-dados-total = Total
pesquisa-dados-aviso = Aviso de la recolección
pesquisa-dados-concluida = Recolección del análisis terminada
pesquisa-dados-concluida-avisos = Recolección terminada con avisos
pesquisa-dados-concluida-texto = { $tamanho } descargados · { $n } valores en el universo
pesquisa-dados-coleta-titulo = Recolección del análisis de activos
pesquisa-dados-coletando-live = RECOLECTANDO
pesquisa-dados-fontes = Fuentes
pesquisa-dados-primeira-coleta = Primera recolección
pesquisa-dados-ultima-coleta = Última recolección
pesquisa-dados-nunca = nunca
pesquisa-dados-universo = Universo
pesquisa-dados-universo-valor = { $n } valores · { $data }
pesquisa-dados-vazio = vacío
pesquisa-dados-emissores = Emisores
pesquisa-dados-papeis = Valores
pesquisa-dados-fatos = Hechos
pesquisa-dados-documentos = Documentos
pesquisa-dados-cotacoes-ate = Cotizaciones hasta
pesquisa-dados-coletando = Recolectando…
pesquisa-dados-coletar-agora = Recolectar ahora
pesquisa-dados-baixando = Descargando y calculando
pesquisa-dados-primeira-leva = la primera tarda de 1 a 2 minutos
pesquisa-dados-so-mudou = solo lo que cambió
pesquisa-dados-falhou = La recolección falló
pesquisa-dados-ultima-por-fonte = Última recolección por fuente
pesquisa-dados-resumo = { $qual ->
    [esta] Esta recolección
   *[ultima] Última recolección
}: { $tamanho } en { $segundos } s · { $universo } valores en el universo · { $radar } dividendos proyectados por el radar
pesquisa-dados-coletar-titulo = Recolectar datos del análisis
pesquisa-dados-coletar-primeira = La primera recolección descarga unos 230 MB de la CVM (DFP, ITR, FCA, FII, IPE), de la B3 (COTAHIST) y de la SEC y tarda de 1 a 2 minutos. No queda nada en disco aparte de la base de datos cifrada.
pesquisa-dados-coletar-incremental = Descarga solo los archivos que cambiaron desde la última recolección (normalmente pocos MB) y recalcula el universo y el radar de dividendos.
pesquisa-dados-baixar-230 = Descargar ~230 MB
pesquisa-dados-coletar = Recolectar
pesquisa-dados-baixar-tudo = Descargar todo de nuevo
pesquisa-dados-baixar-tudo-dica = Ignora lo que no cambió (~230 MB de nuevo). Solo para corregir datos.
pesquisa-dados-ajustes-salvos = Ajustes del análisis guardados
pesquisa-dados-ajustes = Ajustes del análisis
pesquisa-dados-sec-email = Configura el correo de la SEC
pesquisa-dados-sec-ua = User-Agent de la SEC EDGAR
pesquisa-dados-sec-ua-dica = Nombre de la app y un correo de contacto real; la SEC bloquea las solicitudes sin identificación.
pesquisa-dados-sec-ua-erro = Incluye un correo (p. ej.: Treasury/0.1 tu.nombre@email.com).
pesquisa-dados-sec-ua-exemplo = Treasury/0.1 tu.nombre@email.com
pesquisa-dados-taxa-invalida = Entre 0 y 100%.
pesquisa-dados-bazin-dica = Supuestos predeterminados del precio techo de Bazin (promedio de dividendos por acción ÷ tasa mínima); puedes cambiarlos caso por caso en la pestaña Precio calculado del activo.
pesquisa-dados-descartar = Descartar
pesquisa-dados-salvar-ajustes = Guardar ajustes
pesquisa-doc-assembleias = Asambleas y reuniones
pesquisa-doc-janela = Período de los documentos
pesquisa-doc-12-meses = 12 meses
pesquisa-doc-tudo-coletado = Todo lo recogido
pesquisa-doc-ultimo = El último documento guardado es del { $data }.
pesquisa-doc-ampliar = Ver desde { $data }
pesquisa-doc-sem-fonte = Sin fuente de documentos para esta clase
pesquisa-doc-sem-fonte-texto = Los hechos relevantes e informes de FII, ETF y fondos se publican en Fundos.NET (B3), que la app todavía no recoge. Las compañías abiertas vienen del IPE de la CVM.
pesquisa-doc-sem-emissor = No encontramos el emisor de este activo
pesquisa-doc-sem-emissor-texto = El activo no tiene CNPJ en el registro y el ticker no está en el registro de compañías de la CVM (FCA). Indica el CNPJ de la compañía en el activo para traer los documentos.
pesquisa-doc-informar-cnpj = Indicar el CNPJ
pesquisa-doc-sem-coleta = La recogida de la CVM todavía no se ejecutó
pesquisa-doc-sem-coleta-texto = Ejecuta la recogida en Datos del análisis para traer los documentos de las compañías.
pesquisa-doc-quieta = Ningún documento desde el { $data }
pesquisa-doc-quieta-texto = La compañía no publicó hecho relevante, comunicado, aviso, asamblea ni reunión en las categorías guardadas desde esa fecha.
pesquisa-doc-diagnostico = Diagnóstico de los documentos
pesquisa-doc-diag-fonte = Fuente
pesquisa-doc-diag-fonte-ipe = IPE de la CVM (compañías abiertas)
pesquisa-doc-diag-emissor = Emisor
pesquisa-doc-diag-sem-emissor = no encontrado
pesquisa-doc-diag-coleta = Recogida
pesquisa-doc-diag-coleta-em = { $data }, guardando desde el { $desde }
pesquisa-doc-diag-sem-coleta = todavía no se ejecutó
pesquisa-doc-diag-guardados = Guardados
pesquisa-doc-diag-nenhum = ninguno
pesquisa-doc-diag-categorias = Categorías guardadas
pesquisa-nao-tem-ativo = No tienes este activo. Observar lo pone en la watchlist, sin operación, y activa el score con historial, las alertas y el Estudio con IA.
pesquisa-observado-pelo-mercado = Observado desde Mercado: dejar de observar lo quita del registro si nunca tuvo operación.
pesquisa-observar = Observar
pesquisa-deixar-observar = Dejar de observar
pesquisa-observando = Observando { $ticker }
pesquisa-observando-texto = Está en la watchlist, sin operación.
pesquisa-deixou-observar = Dejaste de observar
pesquisa-observar-erro = No se pudo observar
pesquisa-scr-metodos = Métodos
pesquisa-scr-avisar = Avisar cuando entre un valor nuevo
pesquisa-tags = Características
pesquisa-tags-ajuda = Marque las que el valor debe tener (todas juntas). La regla aparece al pasar el ratón.
pesquisa-tags-limpar = Limpiar características
pesquisa-tag-chip = { $tag } · { $n }
pesquisa-tag-vazia = { $regra }
    Ningún valor del resultado actual tiene esta característica.
pesquisa-tags-mais = { $tags } y { $n } más
