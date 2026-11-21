# Activos, mercado e índices (núcleo: `assets`, `market`, `indices`): errores de validación,
# motivos del historial, avisos y errores de la descarga de cotizaciones e índices, nombres y
# términos de los proveedores y nombres y notas de los países del catálogo, en el idioma de la
# instalación. Los nombres de las clases vienen de carteira.ftl (`carteira-classe-*`) y los de
# los países de nucleo-pais.ftl; los nombres de las series de índices (guardados en la bóveda)
# quedan como están.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Activos

nucleo-ativos-ref-ativo = activo { $id }
nucleo-ativos-motivo-cadastrado = Activo registrado: { $ativo }
nucleo-ativos-motivo-atualizado = Activo actualizado
nucleo-ativos-motivo-removido = Activo eliminado
nucleo-ativos-erro-classe-invalida = clase de activo no válida: { $classe }
nucleo-ativos-erro-taxa-invalida = tasa no válida: { $taxa }
nucleo-ativos-erro-nome = indica el nombre del activo
nucleo-ativos-erro-renda-fixa-termos = la renta fija necesita sus condiciones (indexador, tasa, vencimiento)
nucleo-ativos-erro-previdencia-classe = el plan de pensiones es para activos de la clase Planes de pensiones
nucleo-ativos-erro-bolsa-fora = la bolsa { $bolsa } no está en el catálogo: registra primero la bolsa (código, país, moneda, huso horario y horario de negociación)
nucleo-ativos-erro-mercado = el mercado { $mercado } no existe: usa una bolsa del catálogo o uno de estos: { $lista }
nucleo-ativos-erro-bolsa-falta = indica la bolsa (LSE, XETRA, TSE…) de las acciones y ETF de otras bolsas
nucleo-ativos-erro-duplicado = el activo { $ticker } ya está registrado
nucleo-ativos-erro-moeda-invalida = moneda no válida: { $moeda }
nucleo-ativos-erro-moeda-em-uso = el activo ya tiene operaciones o dividendos en esta moneda; crea otro activo
nucleo-ativos-erro-bolsa-catalogo = las acciones y ETF de otras bolsas necesitan una bolsa del catálogo (LSE, XETRA, TSE…)
nucleo-ativos-erro-apagar-em-uso = el activo tiene operaciones o dividendos; archívalo en lugar de eliminarlo

## Búsqueda de tickers

nucleo-ativos-tipo-acao = Acción
nucleo-ativos-tipo-fundo = Fondo
nucleo-ativos-erro-busca-yahoo = búsqueda de Yahoo no disponible: { $erro }

## Bolsas

nucleo-ativos-ref-bolsa = bolsa { $bolsa }
nucleo-ativos-motivo-bolsa-salva = Bolsa guardada: { $bolsa }
nucleo-ativos-motivo-bolsa-removida = Bolsa eliminada: { $bolsa }
nucleo-ativos-erro-horario = hora no válida: { $horario } (usa HH:MM)
nucleo-ativos-erro-fuso = huso horario desconocido: { $fuso }
nucleo-ativos-erro-bolsa-codigo = código de la bolsa: hasta 12 letras y números (p. ej.: LSE)
nucleo-ativos-erro-bolsa-nome = indica el nombre de la bolsa
nucleo-ativos-erro-bolsa-pais = país: código ISO de 2 letras (p. ej.: GB)
nucleo-ativos-erro-bolsa-abertura = la apertura tiene que ser antes del cierre
nucleo-ativos-erro-bolsa-almoco = la pausa del almuerzo tiene que quedar dentro de la sesión
nucleo-ativos-erro-bolsa-dias = días de negociación: dígitos de 1 (lunes) a 7 (domingo)
nucleo-ativos-erro-bolsa-sufixo = el sufijo de Yahoo empieza con punto (p. ej.: .L)
nucleo-ativos-erro-bolsa-do-catalogo = las bolsas del catálogo no se pueden eliminar (ajusta el horario, si hace falta)
nucleo-ativos-erro-bolsa-com-ativos = hay activos en esta bolsa

## Cotizaciones

nucleo-ativos-erro-chaves-so-interface = las claves de las fuentes de datos solo se cambian desde la app
nucleo-ativos-erro-chave-fred = la clave de la FRED tiene 32 letras y números
nucleo-ativos-erro-token-brapi = el token de brapi.dev tiene letras, números, guiones o guiones bajos
nucleo-ativos-erro-preco-invalido = precio no válido: { $preco }
nucleo-ativos-erro-preco-positivo = el precio debe ser positivo
nucleo-ativos-erro-preco-banco = precio no válido en la base de datos: { $preco }
nucleo-ativos-erro-intradiario-periodo = período intradía: 1d o 5d
nucleo-ativos-erro-moeda-yahoo = la moneda de Yahoo ({ $moeda_yahoo }) es distinta de la del activo ({ $moeda_ativo })
nucleo-ativos-sem-cotacao-manual = cotización manual: registra los precios en el activo
nucleo-ativos-sem-cotacao-fundo-sem-cnpj = fondo sin CNPJ: indica el CNPJ para buscar la cuota en la CVM
nucleo-ativos-sem-cotacao-sem-ticker = sin ticker: indica el ticker o el símbolo del proveedor en el activo
nucleo-ativos-sem-cotacao-sem-bolsa = sin bolsa del catálogo: indica la bolsa (LSE, XETRA, TSE…) en el activo
nucleo-ativos-sem-cotacao-sem-provedor = clase sin proveedor predeterminado: indica el símbolo de Yahoo en el activo
nucleo-ativos-motivo-desdobramento-yahoo = Desdoblamiento informado por Yahoo: { $ativo }
nucleo-ativos-nota-bonificacao-yahoo = Bonificación registrada automáticamente por la descarga de cotizaciones (Yahoo la trata como desdoblamiento); indica el costo asignado por la empresa, si lo hay
nucleo-ativos-nota-evento-automatico = Registrado automáticamente por la descarga de cotizaciones
nucleo-ativos-erro-yahoo-sem-resultado = Yahoo: sin resultado
nucleo-ativos-erro-yahoo-sem-preco = Yahoo: cotización sin precio
nucleo-ativos-erro-yahoo-sem-horario = Yahoo: cotización sin hora
nucleo-ativos-erro-yahoo-intervalo = intervalo { $intervalo }/{ $periodo } no admitido
nucleo-ativos-erro-yahoo-busca = respuesta de la búsqueda de Yahoo no válida: { $erro }
nucleo-ativos-erro-cvm-layout = formato del informe diario de la CVM no reconocido
nucleo-ativos-erro-cvm-status = la CVM respondió { $status } para { $url }
nucleo-ativos-erro-cvm-fundo-ausente = fondo { $fundo } no encontrado en el informe diario de la CVM
nucleo-ativos-erro-tesouro-titulo-ausente = título no encontrado en el Tesouro Transparente: { $titulo }
nucleo-ativos-erro-tesouro-colunas = el formato del CSV cambió: faltan columnas esperadas ({ $colunas })
nucleo-ativos-erro-tesouro-poucas-linhas = CSV con pocas filas válidas ({ $linhas }): el archivo puede estar incompleto o haber cambiado
nucleo-ativos-erro-tesouro-linhas-ruins = { $ruins } de { $total } filas ilegibles o con precio fuera de rango: el formato puede haber cambiado
nucleo-ativos-erro-tesouro-parado = datos detenidos: el precio más reciente es del { $data }
nucleo-ativos-erro-tesouro-http = el Tesouro Transparente respondió HTTP { $status }
nucleo-ativos-erro-tesouro-pagina = el Tesouro Transparente devolvió una página en lugar del CSV

## Índices: catálogo de países

nucleo-ativos-erro-pais-fora = país fuera del catálogo: { $pais }
nucleo-ativos-erro-pais-so-interface = el país de referencia solo se cambia desde la app
nucleo-ativos-pais-nota-br = Como siempre: IPCA, CDI/Selic y PTAX del Banco Central.
nucleo-ativos-pais-nota-us = El CPI-U de la FRED requiere la clave gratuita; sin ella, se usa el CPI de la OCDE (sin clave).
nucleo-ativos-pais-nota-banco-mundial = Inflación anual del Banco Mundial, interpolada mes a mes (estimada).

## Índices: proveedores (nombre y términos de uso)

nucleo-ativos-provedor-bcb_sgs-nome = Banco Central de Brasil (SGS, PTAX, Focus)
nucleo-ativos-provedor-bcb_sgs-termos = Datos abiertos del BCB, de uso libre citando la fuente.
nucleo-ativos-provedor-fred-nome = FRED (Federal Reserve Bank of St. Louis)
nucleo-ativos-provedor-fred-termos = Gratuito con clave personal; cada serie sigue los términos de su fuente original.
nucleo-ativos-provedor-yahoo-nome = Yahoo Finance
nucleo-ativos-provedor-yahoo-termos = Fuente no oficial, sin garantía; se usa solo para los cierres diarios.
nucleo-ativos-provedor-ecb-nome = Banco Central Europeo (ECB Data Portal)
nucleo-ativos-provedor-ecb-termos = Reutilización libre citando la fuente (términos del ECB Data Portal).
nucleo-ativos-provedor-eurostat-nome = Eurostat
nucleo-ativos-provedor-eurostat-termos = Reutilización libre citando la fuente (política de datos abiertos de la Comisión Europea, CC BY 4.0).
nucleo-ativos-provedor-oecd-nome = OCDE (OECD Data Explorer)
nucleo-ativos-provedor-oecd-termos = Datos de la OCDE bajo CC BY 4.0; la API pública limita la cantidad de consultas por hora.
nucleo-ativos-provedor-boc-nome = Bank of Canada (Valet)
nucleo-ativos-provedor-boc-termos = Uso libre citando la fuente (términos del Bank of Canada).
nucleo-ativos-provedor-ons-nome = Office for National Statistics (Reino Unido)
nucleo-ativos-provedor-ons-termos = Open Government Licence v3.0.
nucleo-ativos-provedor-boe-nome = Bank of England (Database)
nucleo-ativos-provedor-boe-termos = Uso libre citando la fuente (términos del Bank of England).
nucleo-ativos-provedor-worldbank-nome = Banco Mundial (World Development Indicators)
nucleo-ativos-provedor-worldbank-termos = CC BY 4.0. Inflación anual; los meses se interpolan y quedan marcados como estimados.
nucleo-ativos-provedor-ibge_sidra-nome = IBGE (SIDRA)
nucleo-ativos-provedor-ibge_sidra-termos = Datos públicos del IBGE, de uso libre citando la fuente.
nucleo-ativos-provedor-nyfed-nome = Federal Reserve Bank of New York (Markets Data)
nucleo-ativos-provedor-nyfed-termos = Datos públicos del Federal Reserve Bank of New York, de uso libre citando la fuente.

## Índices: descarga

nucleo-ativos-erro-resposta-invalida = respuesta de { $fonte } no válida: { $erro }
nucleo-ativos-erro-resposta-invalida-sem-detalhe = respuesta de { $fonte } no válida
nucleo-ativos-aviso-sem-chave-fred = para descargar el CPI, el M2 y los Fed Funds de EE. UU., registra la clave gratuita de la FRED en Ajustes › Fuentes de datos
nucleo-ativos-erro-numero = número no válido: { $numero }
nucleo-ativos-erro-periodo = período no válido: { $periodo }
nucleo-ativos-erro-decimal-banco = decimal no válido en la base de datos: { $valor }
nucleo-ativos-erro-fonte-desconhecida = fuente desconocida: { $fonte }
nucleo-ativos-erro-adaptador-proprio = { $fonte } se descarga con su propio adaptador
nucleo-ativos-erro-oecd-sem-resposta = la OCDE no respondió
nucleo-ativos-erro-codigo-ons = código ONS no válido: { $codigo }
nucleo-ativos-erro-codigo-sidra = código SIDRA no válido: { $codigo }
nucleo-ativos-erro-sgs-data = fecha del SGS no válida: { $data }
nucleo-ativos-erro-boc-sem-observacoes = respuesta del Bank of Canada sin observaciones
nucleo-ativos-erro-eurostat-sem-tempo = respuesta de Eurostat sin la dimensión de tiempo
nucleo-ativos-erro-eurostat-varias-series = la consulta a Eurostat debe elegir una sola serie (geo, unit y coicop18)
nucleo-ativos-erro-eurostat-indice-tempo = índice de tiempo de Eurostat no válido
nucleo-ativos-erro-eurostat-valor = valor de Eurostat no válido: { $valor }
nucleo-ativos-erro-eurostat-posicao = posición de Eurostat no válida
nucleo-ativos-erro-eurostat-valores = valores de Eurostat no válidos
nucleo-ativos-erro-nyfed-sem-taxas = respuesta del NY Fed sin tasas
nucleo-ativos-erro-sdmx-nao-csv = respuesta SDMX inesperada (no es CSV)
nucleo-ativos-erro-sdmx-csv = CSV SDMX no válido: { $erro }
nucleo-ativos-erro-sdmx-coluna = CSV SDMX sin la columna { $coluna }
nucleo-ativos-erro-ons-mes = mes de la ONS no válido: { $mes }
nucleo-ativos-erro-boe-nao-csv = respuesta inesperada del Bank of England (no es CSV)
nucleo-ativos-erro-boe-data = fecha del Bank of England no válida: { $data }

## Datos de los activos con fuente (enriquecimiento, fase 1)

nucleo-ativos-motivo-dados = Datos de los activos completados con las fuentes
nucleo-ativos-motivo-campo-travado = Campo del activo bloqueado para sugerencias
nucleo-ativos-motivo-campo-destravado = Campo del activo desbloqueado para sugerencias
nucleo-ativos-dados-erro-campo = campo de datos del activo desconocido: { $campo }
nucleo-ativos-dados-erro-vazio = ningún campo elegido
nucleo-ativos-dados-erro-repetido = el campo { $campo } de { $ativo } aparece más de una vez
nucleo-ativos-dados-erro-travado = el campo { $campo } de { $ativo } está bloqueado para sugerencias
nucleo-ativos-dados-erro-sem-fonte = el valor "{ $valor }" para { $campo } de { $ativo } no está en ninguna fuente registrada en la bóveda
nucleo-ativos-dados-erro-preenchido = el campo { $campo } de { $ativo } ya tiene un valor tuyo; para cambiarlo, pide el cambio (replace)
nucleo-ativos-dados-efeito-score = el score pasa a comparar con los pares de { $segmento }

## Estudio con IA

nucleo-ativos-estudo-ref = estudio { $id }
nucleo-ativos-estudo-erro-contexto = el contexto para la IA puede tener como máximo { $max } caracteres
nucleo-ativos-estudo-erro-pergunta = la pregunta puede tener como máximo { $max } caracteres
nucleo-ativos-estudo-erro-sem-motor = Ningún motor de IA configurado. Registra uno en Ajustes › IA y agente y ponlo en la ruta "Estudio de activos" o en la del chat.
nucleo-ativos-estudo-erro-formato = El motor respondió fuera del formato del estudio. Inténtalo de nuevo o elige otro motor.
nucleo-ativos-estudo-erro-aviso = Antes del primer estudio, lee y acepta el aviso en la pestaña Estudio con IA del activo.
nucleo-ativos-estudo-erro-rodando = Ya se está generando un estudio; espera a que termine.
nucleo-ativos-estudo-erro-so-interface = solo desde la interfaz de la app
nucleo-ativos-estudo-titulo-job = Estudio con IA
nucleo-ativos-motivo-estudo = Estudio con IA: { $ativo }
nucleo-ativos-motivo-contexto-estudo = Contexto del activo para la IA
nucleo-ativos-motivo-estudo-apagado = Estudio con IA eliminado
nucleo-ativos-motivo-estudo-pedido = Estudio con IA pedido por el agente: { $ativo }
nucleo-ativos-estudo-pronto = Estudio listo
nucleo-ativos-consulta-cvm-sem-registro = El registro de fondos de la CVM aún no se descargó: usa "Recolectar ahora" en Recolección del análisis de activos y consulta de nuevo.

## brapi.dev

nucleo-ativos-brapi-sem-token = brapi.dev sin token: registra uno en Ajustes › Fuentes de datos.
nucleo-ativos-erro-cripto-sem-cambio = sin tipo de cambio del dólar a { $moeda }: la cotización de cripto viene en dólares (ejecuta la recolección de índices o informa el tipo de cambio)
nucleo-ativos-erro-fonte-sem-chave = falta la clave de la fuente: regístrala en Ajustes › Fuentes de datos
nucleo-ativos-erro-moeda-fonte = la moneda de la fuente ({ $moeda_fonte }) es distinta de la del activo ({ $moeda_ativo })
nucleo-ativos-motivo-desdobramento-coleta = Desdoblamiento informado por la descarga de cotizaciones: { $ativo }
nucleo-ativos-erro-eodhd-rede = no fue posible comunicarse con EODHD
nucleo-ativos-erro-chave-fonte = clave inválida: revisa el valor copiado del sitio de la fuente
nucleo-ativos-brapi-ja-hoje = brapi.dev: la cotización de este papel ya se buscó hoy (una por día, para ahorrar la cuota).
nucleo-ativos-brapi-limite = brapi.dev: se alcanzó el límite de uso del mes; la cotización vuelve después de la renovación o con un límite mayor en Ajustes › Fuentes de datos.
nucleo-ativos-erro-limite-brapi = el límite de uso de brapi va de 10% a 100% de la cuota
nucleo-ativos-erro-conta-custodia = cuenta de custodia no encontrada: { $conta }
nucleo-ativos-erro-custodia-cartao = la cuenta donde está el activo no puede ser una tarjeta de crédito
