# Score de salud por activo (núcleo: `score`): notas, franjas, explicaciones, etiquetas de las
# fuentes de datos, notas de cobertura, motivos de datos faltantes, avisos y errores, en el
# idioma de la instalación. Las etiquetas y fórmulas de la metodología (methodology/**: clases,
# segmentos, perfiles, dimensiones y métricas) quedan como están; los nombres de las franjas
# vienen de score.ftl (`score-faixa-*`).
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Tramos, aviso y motivos de no haber score

nucleo-score-aviso = Score calculado con datos públicos y reglas de la literatura, aún no validado con el historial (metodología en borrador). No es una recomendación de inversión.
nucleo-score-faixa-robusto = Fundamentos sólidos en los criterios medidos
nucleo-score-faixa-saudavel = Bien; monitorear cada trimestre
nucleo-score-faixa-atencao = Deterioro o debilidad relevante; revisar la tesis
nucleo-score-faixa-risco = Fundamentos comprometidos en los criterios medidos
nucleo-score-nao-implementado = El score de { $classe } aún no se implementó (primera entrega: acciones de Brasil, bancos y FII).
nucleo-score-sem-score = Sin score: { $motivo }.
nucleo-score-sem-score-falta = Sin score: falta { $metricas }, esencial para este tipo de activo.
nucleo-score-parcial = Score parcial: el { $imputado }% del peso fue imputado (límite { $limite }%), por eso no hay tramo definitivo.
nucleo-score-sem-metricas = No hay métricas calculables.
nucleo-score-dimensao-dados = Datos
nucleo-score-dado-demonstracoes = Estados financieros (CVM)
nucleo-score-sem-demonstracoes = no hay estados financieros de la CVM descargados para este activo (haz la recolección del análisis de activos)

## Motor: valores, pares, notas y explicaciones

nucleo-score-valor-dias = { $valor } días
nucleo-score-valor-anos = { $valor } años
nucleo-score-valor-bi = R$ { $valor } mil M
nucleo-score-valor-mi = R$ { $valor } M
nucleo-score-valor-mil = R$ { $valor } mil
nucleo-score-pares-do-setor = del sector { $setor }
nucleo-score-pares-do-segmento = del segmento { $segmento }
nucleo-score-pares-da-classe = de la clase
nucleo-score-sem-dado = sin dato
nucleo-score-nota-defasados = Datos desactualizados: los más recientes son del { $data } (más de 2 trimestres).
nucleo-score-nota-sem-data = Sin fecha de los datos.
nucleo-score-nota-confianca = Confianza × { $fator }: { $motivo }.
nucleo-score-pior-caso = peor caso
nucleo-score-explica-pares = { $metrica } { $valor }: p{ $percentil } entre { $pares } pares { $nivel }
nucleo-score-explica-absoluta = { $metrica } { $valor }: nota { $nota } por la referencia absoluta
nucleo-score-lista = { $itens } y { $ultimo }

## Acciones: confianza, Magic Formula y notas

nucleo-score-confianca-fonte-secundaria = fuente secundaria (Fundamentus) en { $metricas }
nucleo-score-confianca-liquidez-baixa = liquidez baja
nucleo-score-confianca-divergencia = diferencia de más del 25% con Fundamentus en algún indicador
nucleo-score-magic-roc = Retorno sobre el capital (EBIT ÷ capital de trabajo + activo fijo)
nucleo-score-magic-nota = Diagnóstico: el ROC entra en Rentabilidad y el EY en Valuación
nucleo-score-nota-financeira = Institución financiera: Altman, Piotroski, Beneish y EBITDA no aplican.

## Comandos

nucleo-score-erro-segmento-so-interface = el segmento del score solo se cambia desde la app
nucleo-score-erro-avaliacao-so-interface = la evaluación del score solo se hace desde la app
nucleo-score-erro-eventos-so-interface = los eventos del score solo se gestionan desde la app
nucleo-score-motivo-avaliacao = evaluación del score de salud del activo
nucleo-score-motivo-evento = evento cualitativo del score de salud
nucleo-score-motivo-decisao-evento = decisión sobre un evento del score de salud
nucleo-score-motivo-apagar-evento = eliminar un evento del score de salud

## Bancos (IF.data)

nucleo-score-banco-grupo-grandes = Bancos grandes (S1–S2)
nucleo-score-banco-grupo-medios = Bancos medianos (S3)
nucleo-score-banco-grupo-pequenos = Bancos pequeños (S4)
nucleo-score-banco-trimestre = trimestre hasta el { $data }
nucleo-score-banco-sem-dados = sin datos del IF.data del Banco Central para este banco
nucleo-score-banco-sem-basileia = sin Índice de Basilea en el IF.data
nucleo-score-banco-sem-capital-principal = sin capital principal en el IF.data
nucleo-score-banco-sem-alavancagem = sin razón de apalancamiento en el IF.data
nucleo-score-banco-sem-inadimplencia = sin morosidad o cartera en el IF.data
nucleo-score-banco-sem-ativos-problematicos = sin activos problemáticos en el IF.data
nucleo-score-banco-sem-provisao = sin provisión o morosidad en el IF.data
nucleo-score-banco-sem-despesa-provisao = sin gasto de provisión de 12 meses en el IF.data
nucleo-score-banco-sem-carteira-anterior = sin la cartera de 12 meses antes en el IF.data
nucleo-score-banco-sem-despesas-receitas = sin gastos e ingresos de 12 meses en el IF.data
nucleo-score-banco-nota-relatorios = los informes 2 y 4 del IF.data solo se descargan para bancos S1–S3, listados y emisores de la cartera
nucleo-score-sem-fonte = sin fuente
nucleo-score-banco-nota-prudenciais = Indicadores prudenciales de { $banco } (IF.data del Banco Central, { $periodo }); los resultados de 12 meses suman los semestres ({ $convencao }).
nucleo-score-banco-nota-sem-correspondencia = Banco sin correspondencia en el IF.data del Banco Central (por el CNPJ de la institución líder): haz la recolección del análisis de activos con el Banco Central encendido.
nucleo-score-banco-roe-abaixo = ROE por debajo del crecimiento: P/VL justo ≤ 0
nucleo-score-banco-pvp-justo = P/VL justo { $pvp } con Ke { $ke }% y g { $g }%
nucleo-score-banco-sem-pvp-roe = sin P/VL o ROE
nucleo-score-banco-flag-basileia = Basilea de { $valor }%

## Cripto

nucleo-score-cripto-nao-comparavel = Las cripto no se comparan directamente con acciones: el score mide la solidez dentro de la clase de criptoactivos.
nucleo-score-cripto-periodo-coleta = recolección del { $data }
nucleo-score-cripto-sem-coingecko = sin { $dado } de CoinGecko (haz la recolección del análisis de activos)
nucleo-score-cripto-dado-valor-mercado = valor de mercado
nucleo-score-cripto-dado-ranking = ranking
nucleo-score-cripto-dado-volume = volumen de 24 horas
nucleo-score-cripto-dado-volume-valor = volumen y valor de mercado
nucleo-score-cripto-dado-fdv = valor totalmente diluido
nucleo-score-cripto-dado-oferta = oferta en circulación
nucleo-score-cripto-dado-historico = historial de 12 meses
nucleo-score-cripto-dado-usuarios = usuarios que lo siguen
nucleo-score-cripto-dado-commits = commits
nucleo-score-cripto-dado-estrelas = estrellas en GitHub
nucleo-score-cripto-dado-maxima = máximo histórico
nucleo-score-cripto-fonte-dolar = { $fonte }; dólar a R$ { $taxa } ({ $data })
nucleo-score-cripto-sem-ptax = sin la cotización del dólar (PTAX) para convertir
nucleo-score-cripto-sem-valor-mercado = sin valor de mercado de CoinGecko ni de DefiLlama (haz la recolección del análisis de activos)
nucleo-score-cripto-sem-oferta-maxima = sin oferta máxima (emisión continua): vale la inflación de la oferta
nucleo-score-cripto-sem-tvl = no es una red con TVL en DefiLlama (o falta la recolección)
nucleo-score-cripto-periodo-genese = génesis el { $data }
nucleo-score-cripto-sem-genese = sin fecha del bloque génesis en CoinGecko
nucleo-score-cripto-periodo-media-30d = promedio de 30 días hasta el { $data }
nucleo-score-cripto-nota-coin-metrics = Coin Metrics, datos abiertos de la API Community (CC BY-NC 4.0)
nucleo-score-cripto-sem-coin-metrics = la API abierta de Coin Metrics no cubre esta red (o falta la recolección)
nucleo-score-cripto-periodo-semana = semana hasta el { $data }
nucleo-score-cripto-nota-nakamoto = pools de minería con más de la mitad de los bloques de la semana
nucleo-score-cripto-sem-nakamoto = solo para bitcoin (pools de minería de mempool.space); otras redes sin fuente gratuita
nucleo-score-cripto-fonte-ataques = DefiLlama (lista de ataques)
nucleo-score-cripto-nota-ataques = { $n ->
    [0] ningún ataque a la red en la lista de DefiLlama
    [one] 1 ataque a la red en la lista de DefiLlama
   *[other] { $n } ataques a la red en la lista de DefiLlama
}
nucleo-score-cripto-sem-ataques = sin la lista de ataques de DefiLlama (haz la recolección del análisis de activos)
nucleo-score-cripto-periodo-regra = regla de la red
nucleo-score-cripto-fonte-categorias = CoinGecko (categorías)
nucleo-score-cripto-nota-pow = prueba de trabajo: sin asignaciones con desbloqueo; la emisión nueva entra en la inflación de la oferta
nucleo-score-cripto-sem-desbloqueios = calendarios de desbloqueo de tokens solo en servicios pagos (Token Unlocks, Messari)
nucleo-score-cripto-fonte-github = CoinGecko (datos de GitHub)
nucleo-score-cripto-fonte-precos-dolar = precios diarios en dólares (DefiLlama)
nucleo-score-cripto-fonte-cotacoes-app = cotizaciones de la app en { $moeda }
nucleo-score-periodo-de-a = { $de } a { $ate }
nucleo-score-cripto-poucas-cotacoes = menos de { $n } cotizaciones en los últimos 12 meses (haz la recolección de cotizaciones)
nucleo-score-nota-pico-fundo = pico el { $pico }, fondo el { $fundo }
nucleo-score-cripto-menos-12-meses = menos de 12 meses de cotizaciones
nucleo-score-cripto-periodo-stablecoins = lista de stablecoins
nucleo-score-cripto-lastro-sem-nota = respaldo "{ $lastro }" sin nota en la metodología
nucleo-score-cripto-fora-stablecoins = fuera de la lista de stablecoins de DefiLlama (o falta la recolección)
nucleo-score-cripto-periodo-dias = { $de } a { $ate } ({ $dias } días)
nucleo-score-cripto-nota-paridade = paridad con 1 { $moeda }
nucleo-score-cripto-sem-precos-paridade = sin precios diarios en la moneda de la paridad ({ $moeda }) de los últimos 12 meses
nucleo-score-cripto-nota-cambio = Volatilidad y drawdown medidos en { $moeda }: incluyen la variación del tipo de cambio.
nucleo-score-cripto-lastro-fiat = moneda fiduciaria y títulos
nucleo-score-cripto-lastro-cripto = otros criptoactivos (sobrecolateralizado)
nucleo-score-cripto-lastro-algoritmico = algorítmico, sin reservas
nucleo-score-cripto-flag-desvio = desviación máxima de { $desvio }
nucleo-score-cripto-flag-algoritmico = respaldo algorítmico (DefiLlama)
nucleo-score-cripto-flag-fdv = FDV { $vezes } el valor de mercado
nucleo-score-cripto-flag-valor-mercado = valor de mercado de { $valor }
nucleo-score-cripto-dado-criptoativo = Datos del criptoactivo
nucleo-score-cripto-so-cadastrados = el score de cripto solo vale para criptoactivos registrados
nucleo-score-cripto-nota-id = Id en CoinGecko: { $id }.
nucleo-score-cripto-nota-coingecko-antiga = Datos de CoinGecko recolectados el { $data }, hace más de 24 horas: ejecuta la recolección del análisis de activos para renovarlos.
nucleo-score-cripto-nota-sem-id = Criptoactivo sin id de CoinGecko: indica el ticker (símbolo) o fija el id con la fuente de cotización "coingecko" y el id en el símbolo del proveedor.
nucleo-score-cripto-nota-sem-pares = Los criptoactivos no tienen universo de pares en la app: las notas usan solo las referencias absolutas de la clase.

## Cripto: errores de las fuentes

nucleo-score-erro-coingecko = respuesta de CoinGecko no válida: { $erro }
nucleo-score-erro-coingecko-sem-moedas = respuesta de CoinGecko sin la lista de monedas
nucleo-score-erro-coingecko-sem-id = respuesta de CoinGecko sin id
nucleo-score-erro-defillama = respuesta de DefiLlama no válida: { $erro }
nucleo-score-erro-defillama-sem-redes = respuesta de DefiLlama sin la lista de redes
nucleo-score-erro-defillama-sem-stablecoins = respuesta de DefiLlama sin la lista de stablecoins
nucleo-score-erro-defillama-sem-valores = respuesta de DefiLlama sin valores de mercado
nucleo-score-erro-defillama-sem-precos = respuesta de DefiLlama sin precios
nucleo-score-erro-coin-metrics-catalogo = catálogo de Coin Metrics no válido: { $erro }
nucleo-score-erro-coin-metrics-sem-ativos = catálogo de Coin Metrics sin la lista de activos
nucleo-score-erro-coin-metrics = respuesta de Coin Metrics no válida: { $erro }
nucleo-score-erro-coin-metrics-sem-serie = respuesta de Coin Metrics sin la serie
nucleo-score-erro-mempool = respuesta de mempool.space no válida: { $erro }
nucleo-score-erro-mempool-sem-blocos = mempool.space sin bloques en la semana
nucleo-score-erro-ataques = lista de ataques de DefiLlama no válida: { $erro }
nucleo-score-erro-ataques-sem-lista = respuesta de ataques de DefiLlama sin la lista

## Cripto: pasos de la recolección

nucleo-score-cripto-doc-coingecko = CoinGecko /coins/{ $id } el { $data }
nucleo-score-cripto-passo-tvl = DefiLlama: TVL de las redes
nucleo-score-cripto-passo-stablecoins = DefiLlama: stablecoins
nucleo-score-cripto-passo-ids = CoinGecko: ids por símbolo
nucleo-score-cripto-passo-historico = CoinGecko: historial de 12 meses de { $id }
nucleo-score-cripto-passo-precos = DefiLlama: precios diarios de { $id }
nucleo-score-cripto-passo-valor-mercado = DefiLlama: valor de mercado
nucleo-score-cripto-passo-ataques = DefiLlama: ataques a las redes
nucleo-score-cripto-passo-redes = Coin Metrics: redes de la API abierta
nucleo-score-cripto-passo-enderecos = Coin Metrics: direcciones activas
nucleo-score-cripto-passo-pools = mempool.space: pools de minería

## Acciones: red flags, comprobaciones y fuentes secundarias

nucleo-score-flag-beneish = M = { $valor } (último ejercicio)
nucleo-score-flag-cobertura = cobertura de { $ultimo }x y { $anterior }x en los dos últimos trimestres (12 meses)
nucleo-score-flag-fcf-negativo = FCO − capex negativo en los 3 últimos ejercicios
nucleo-score-flag-dividendos-acima = dividendos pagados por encima del flujo de caja libre en los 2 últimos ejercicios
nucleo-score-flag-diluicao = acciones en circulación +{ $pct }% en 12 meses
nucleo-score-flag-patrimonio-negativo = patrimonio neto negativo en el último balance
nucleo-score-flag-entrega-atrasada = { $documento } de { $referencia } entregado el { $entrega }
nucleo-score-flag-auditor = { $antiga } sustituida por { $nova } después de { $anos } años
nucleo-score-flag-volume-medio = volumen medio de { $valor }
nucleo-score-recon-proventos-12m = Dividendos por acción en 12 meses ({ $agenda } × B3)
nucleo-score-fonte-b3-oficial = B3 (oficial)
nucleo-score-recon-valor-diferente = valor diferente
nucleo-score-recon-so-b3 = solo en la B3
nucleo-score-recon-fora-b3 = fuera de la B3
nucleo-score-recon-provento = { $tipo ->
    [amortizacao] Amortización
   *[provento] Dividendo
} con fecha de corte { $data } ({ $fonte }: { $situacao })
nucleo-score-recon-nota-divergente = Dividendos de { $ticker } comprobados con la B3 (oficial) en 24 meses: { $iguais } iguales, { $diferentes } con valor diferente, { $so_b3 } solo en la B3, { $so_agenda } solo en el calendario ({ $agenda }). El DY usa la B3, fuente oficial.
nucleo-score-recon-nota-igual = Dividendos de { $ticker } comprobados con la B3 (oficial) en 24 meses: { $iguais } iguales al calendario ({ $agenda }).
nucleo-score-fonte-brapi = brapi.dev, leído el { $data }
nucleo-score-fonte-fundamentus = Fundamentus ({ $indicador }), leído el { $data }
nucleo-score-conferencia-mesma-regra = { $metrica } (rehecha con la fórmula de Fundamentus)
nucleo-score-secundaria-concordam = dos fuentes secundarias coinciden (promedio)
nucleo-score-secundaria-divergem = las fuentes secundarias difieren ({ $fonte1 }: { $valor1 }; { $fonte2 }: { $valor2 }); vale la primera
nucleo-score-secundaria-uma = una fuente secundaria
nucleo-score-secundaria-marca = fuente secundaria
nucleo-score-secundaria-nota-motivo = fuente oficial sin el dato ({ $motivo }); { $nota }
nucleo-score-secundaria-nota = fuente oficial sin el dato; { $nota }
nucleo-score-secundaria-periodo = informado por la fuente secundaria

## Eventos cualitativos

nucleo-score-erro-evento-link = la evidencia tiene que ser un enlace http(s)
nucleo-score-erro-evento-tipo = tipo de evento desconocido: { $tipo }
nucleo-score-erro-evento-titulo = describe el evento (hasta 300 caracteres)
nucleo-score-erro-evento-ativo = indica el activo (asset_id o ticker)
nucleo-score-erro-evento-limites = el tope y la penalización van de 0 a 100
nucleo-score-ref-evento = evento
nucleo-score-flag-evento = Evento: { $tipo }

## Avisos para el desarrollador

nucleo-score-feedback-titulo = Score de activos: faltan datos críticos en { $ativo }
nucleo-score-feedback-resolvido-intrinseco = Resuelto solo el { $data }: no es una falta de la recolección ({ $detalhe }).
nucleo-score-feedback-resolvido-calculado = Resuelto solo el { $data }: llegaron los datos y se calculó el score ({ $detalhe }).
nucleo-score-feedback-resolvido = Resuelto solo el { $data }: { $detalhe }
nucleo-score-feedback-resolvido-encerrado = Resuelto automáticamente el { $data }: la posición se cerró, el activo ya no está en la cartera.
nucleo-score-feedback-corpo = El score de salud de { $ativo } ({ $classe }, perfil { $perfil }) no se calculó el { $data } porque faltan datos críticos:
    { $itens }

    Metodología { $versao }. Ver el activo en Activos › { $ativo } › Salud.
nucleo-score-feedback-lacuna-conhecida = Resuelto solo: laguna conocida de la metodología (sin fuente estructurada gratuita), documentada en docs/asset-score.md. El score renormaliza la dimensión.

## FII

nucleo-score-periodo-12-meses = 12 meses
nucleo-score-periodo-ultimos-pregoes = últimas sesiones
nucleo-score-fii-periodo-trimestral = informe trimestral hasta el { $data }
nucleo-score-fii-periodo-mensal = informe mensual de { $mes }
nucleo-score-fii-sem-trimestral = sin el informe trimestral de la CVM
nucleo-score-fii-periodo-4-trimestres = 4 trimestres hasta el { $data }
nucleo-score-fii-fonte-trimestrais = informes trimestrales CVM
nucleo-score-fii-sem-rendimentos-declarados = sin rendimientos declarados en 4 trimestres
nucleo-score-fii-poucos-trimestrais = menos de 4 informes trimestrales con resultado y rendimientos
nucleo-score-fii-sem-passivo = sin pasivo o patrimonio en el informe mensual
nucleo-score-fii-sem-obrigacoes = sin obligaciones por adquisición en el informe mensual
nucleo-score-fii-periodo-pvp = cotización ÷ VP del último informe
nucleo-score-fii-sem-pvp = sin cotización o valor patrimonial
nucleo-score-fii-sem-dy = sin rendimientos o cotización
nucleo-score-fii-fonte-dy-cdi = DY del fondo; CDI (Banco Central, SGS 12)
nucleo-score-fii-sem-dy-fundo = sin dividend yield
nucleo-score-fii-sem-cdi = sin CDI de los últimos 12 meses
nucleo-score-fii-periodo-taxa = promedio de { $n } informes mensuales, anualizado
nucleo-score-fii-fonte-taxa = CVM informe mensual (Percentual_Despesas_Taxa_Administracao)
nucleo-score-fii-poucas-taxas = menos de 3 informes mensuales con la comisión de administración
nucleo-score-fii-ipca-descontado = IPCA descontado
nucleo-score-fii-sem-ipca = sin IPCA del período: variación nominal
nucleo-score-fii-fonte-vp-cota = CVM informe mensual (valor patrimonial por cuota)
nucleo-score-fii-sem-vp-anterior = sin valor patrimonial por cuota de 12 meses antes
nucleo-score-fii-sem-cotistas = sin número de cuotistas
nucleo-score-fii-flag-vacancia = vacancia financiera de { $pct }
nucleo-score-fii-flag-inquilino = mayor inquilino con { $pct } de los ingresos
nucleo-score-fii-dado-informes = Informes del FII (CVM)
nucleo-score-fii-sem-informes = no hay informes de la CVM descargados para este fondo (haz la recolección del análisis de activos)
nucleo-score-fii-nota-composicao = Composición del activo en el último informe mensual: CRI { $cri }, cuotas de FII { $fii }, inmuebles { $imoveis }.

## Renta fija

nucleo-score-rf-flag-vencido = Venció el { $data }
nucleo-score-rf-titulo-vencido = Título vencido
nucleo-score-rf-titulo-vencido-sentido = Título vencido ({ $data }): no hay nada que evaluar. Si todavía aparece con posición en la Cartera, registra el rescate.
nucleo-score-rf-periodo-taxa = tasa del { $data }
nucleo-score-rf-nota-ipca = IPCA de 12 meses hasta { $mes }: { $pct }%
nucleo-score-rf-nota-selic = Selic meta { $selic }% + prima { $agio }%; { $ipca }
nucleo-score-rf-periodo-hoje = hoy
nucleo-score-rf-periodo-meses = { $n } meses
nucleo-score-rf-fonte-bcb-selic-ipca = Banco Central (SGS 432 y 433)
nucleo-score-rf-nota-juro-real-selic = tasa real de la Selic de { $pct }% frente a { $meses } cierres de mes en { $anos } años
nucleo-score-rf-familia-ipca = IPCA+ (Renda+ y Educa+ incluidos)
nucleo-score-rf-familia-prefixados = prefijados
nucleo-score-rf-nota-percentil-taxa = tasa de { $pct }% frente a { $medianas } medianas diarias de los { $familia } en { $anos } años (mediana { $mediana }%)
nucleo-score-rf-historico-curto = historial de tasas de los { $familia } con menos de { $dias } días guardados
nucleo-score-rf-nota-prefixado-real = prefijado { $pct }% descontado el { $ipca }
nucleo-score-rf-nota-cupom = cupón semestral de { $pct }% anual; vence el { $data }
nucleo-score-rf-nota-parcelas = { $n } cuotas mensuales hasta el { $data }
nucleo-score-rf-nota-sem-cupom = sin cupón; vence el { $data }
nucleo-score-rf-periodo-pregoes = { $n } sesiones
nucleo-score-rf-nota-custodia-selic = { $pct }% anual sobre lo que supera { $isento } (posición de { $posicao })
nucleo-score-rf-nota-custodia = { $pct }% anual sobre el saldo
nucleo-score-rf-nota-pu = PU de compra { $compra } y de venta { $venda }
nucleo-score-rf-sem-termos-liquidez = sin las condiciones del título (liquidez y vencimiento)
nucleo-score-rf-fonte-cadastro = registro del título
nucleo-score-rf-sem-vencimento = sin fecha de vencimiento en el registro del título
nucleo-score-rf-nota-pos-fixado = posfijado: la tasa sigue al CDI/Selic, sensibilidad a las tasas casi nula
nucleo-score-rf-nota-aproximacao = aproximación: plazo hasta el vencimiento (sin el cronograma de intereses y amortización)
nucleo-score-rf-fonte-premissa = supuesto de la metodología (ASSUMPTIONS.md › Renta fija)
nucleo-score-rf-nota-tesouro-reais = Tesoro Nacional en reales
nucleo-score-rf-sem-taxas = sin tasas del Tesouro Transparente guardadas para este título (haz la recolección de cotizaciones con el Tesouro Direto encendido)
nucleo-score-rf-historico-selic-curto = historial de la Selic meta y del IPCA demasiado corto (haz la recolección de índices)
nucleo-score-rf-sem-selic-ipca = sin Selic meta o IPCA de 12 meses (haz la recolección de índices)
nucleo-score-rf-sem-ipca = sin IPCA de 12 meses (haz la recolección de índices)
nucleo-score-rf-nota-selic-duration = posfijado: el precio sigue a la Selic, sensibilidad a las tasas casi nula
nucleo-score-rf-vencimento-desconhecido = vencimiento desconocido: indica la fecha de vencimiento o el año en el código del título (p. ej.: Tesouro IPCA+ 2035)
nucleo-score-rf-fonte-pu = PU del Tesouro Transparente guardado para el activo
nucleo-score-rf-poucos-precos = menos de 20 precios del título en los últimos 12 meses (haz la recolección de cotizaciones)
nucleo-score-rf-fonte-regra-tesouro = regla del Tesouro Direto
nucleo-score-rf-nota-recompra = recompra diaria por el Tesoro
nucleo-score-rf-fonte-regra-b3 = regla de la B3
nucleo-score-rf-fora-de-venda = título fuera de venta en el Tesouro Direto (sin PU de compra)
nucleo-score-rf-nota-tipo-desconhecido = Tipo de título del Tesoro no reconocido en el código ni en el nombre (p. ej.: Tesouro IPCA+ 2035).
nucleo-score-rf-nota-soberano = Riesgo de crédito soberano: nota fija por supuesto (Tesoro Nacional en reales; sin fuente gratuita de calificación).
nucleo-score-rf-dado-termos = Condiciones del título
nucleo-score-rf-sem-termos = el título no tiene las condiciones registradas (tipo, indexador, tasa, vencimiento)
nucleo-score-rf-sem-roe-ifdata = sin utilidad y patrimonio de 12 meses en el IF.data
nucleo-score-rf-nota-emissor-banco-cnpj = Emisor: { $emissor } (IF.data del Banco Central, por el CNPJ del registro del título).
nucleo-score-rf-nota-emissor-banco-nome = Emisor: { $emissor } (IF.data del Banco Central), identificado por el nombre "{ $nome }"; revísalo e indica el CNPJ en el registro para no depender del nombre.
nucleo-score-rf-flag-inativa = { $emissor } no aparece como activa en el último registro del IF.data
nucleo-score-rf-fora-de-funcionamento = { $emissor } ya no funciona según el registro del Banco Central: los indicadores ya no se publican
nucleo-score-rf-banco-nao-identificado = emisor "{ $nome }" no identificado en el registro del Banco Central (IF.data): indica el CNPJ del banco emisor al editar el título en Activos
nucleo-score-rf-banco-nao-informado = emisor no indicado: completa el emisor y el CNPJ del banco en el registro del título
nucleo-score-rf-fgc-tipo-outro = tipo de título "otro": no se puede saber si tiene FGC (la LF no tiene; CDB, LC, LCI, LCA y ahorro sí)
nucleo-score-rf-fonte-regra-fgc = regla del FGC
nucleo-score-rf-sem-fgc = título sin garantía del FGC
nucleo-score-rf-flag-fgc = { $excesso } por encima del límite de { $limite } por CPF en esta institución
nucleo-score-rf-titular-desconhecido = titular no indicado
nucleo-score-rf-fgc-sem-titulares = las cuentas sin titulares registrados cuentan como un solo CPF (exposición de toda la familia)
nucleo-score-rf-fgc-por-titular = exposición por titular de las cuentas (cuenta conjunta dividida entre los titulares)
nucleo-score-rf-sem-posicao = sin posición: dentro del límite
nucleo-score-rf-nota-fgc = { $titulos } título(s) garantizado(s) de la familia en esta institución; mayor exposición por CPF { $exposicao }; titulares: { $titulares }; { $atribuicao }
nucleo-score-rf-fonte-fgc = posiciones de la cartera y regla del FGC
nucleo-score-rf-fonte-regra-poupanca = regla del ahorro
nucleo-score-rf-nota-liquidez-diaria = liquidez diaria
nucleo-score-rf-nota-heranca-banco = Salud del emisor heredada del banco (§7.2): indicadores prudenciales del IF.data del Banco Central frente a los bancos del mismo tamaño.
nucleo-score-rf-sem-cdi = sin CDI guardado (haz la recolección de índices)
nucleo-score-rf-fonte-bcb-poupanca = Banco Central (SGS 12, 432 y 226)
nucleo-score-rf-nota-poupanca = el ahorro rinde { $rende }% anual (Selic meta { $selic }%, TR 12 meses { $tr }%); exento, comparado con impuesto sobre la renta de { $ir }%
nucleo-score-rf-taxa-invalida = tasa no válida en el registro del título
nucleo-score-rf-pct-do-indice = { $pct }% del { $indice }
nucleo-score-rf-nota-prefixado-curva = prefijado { $pct }% frente al Tesouro Prefixado de plazo parecido ({ $curva }%)
nucleo-score-rf-fonte-bcb-cdi = Banco Central (SGS 12)
nucleo-score-rf-nota-prefixado-cdi = prefijado { $pct }% frente al CDI de hoy ({ $cdi }%; sin curva del Tesoro)
nucleo-score-rf-sem-curva-cdi = sin curva del Tesoro ni CDI (haz la recolección de cotizaciones y de índices)
nucleo-score-rf-nota-ipca-curva = inflación + { $pct }% frente al Tesouro IPCA+ de plazo parecido ({ $real }%) y prefijado ({ $pre }%)
nucleo-score-rf-fonte-bcb-cdi-ipca = Banco Central (SGS 12 y 433)
nucleo-score-rf-nota-ipca-cdi = inflación + { $pct }% con IPCA de 12 meses ({ $ipca }%) frente al CDI de hoy ({ $cdi }%; sin curva del Tesoro)
nucleo-score-rf-sem-curva-cdi-ipca = sin curva del Tesoro ni CDI e IPCA (haz la recolección de cotizaciones y de índices)
nucleo-score-rf-nota-isento = ; exento, equivale a { $pct }% del CDI gravado (impuesto sobre la renta de { $ir }% para { $dias } días)
nucleo-score-rf-via-cnpj = por el CNPJ del registro del título
nucleo-score-rf-via-nome = por el nombre "{ $nome }"; revísalo e indica el CNPJ en el registro
nucleo-score-rf-nota-emissor-cvm = Emisor: { $emissor } (CVM), { $como }.
nucleo-score-rf-periodo-dados-ate = datos hasta { $data }
nucleo-score-rf-fonte-score-acoes = score de acciones del emisor
nucleo-score-rf-nota-score-emissor = { $emissor }: score { $score } ({ $faixa }), salud { $saude }{ $parcial ->
    [sim] ; score parcial (demasiado peso imputado)
   *[nao] {""}
}
nucleo-score-rf-score-emissor-falhou = el score de acciones de { $emissor } no salió: { $faltam }
nucleo-score-rf-companhia-nao-identificada = emisor "{ $nome }" no identificado entre las compañías abiertas de la CVM: indica el CNPJ de la compañía en el registro del título (y haz la recolección del análisis de activos)
nucleo-score-rf-companhia-nao-informada = emisor no indicado: completa el emisor y el CNPJ de la compañía en el registro del título
nucleo-score-rf-nota-spread-ipca = inflación + { $pct }% frente al Tesouro IPCA+ de plazo parecido ({ $curva }%)
nucleo-score-rf-sem-curva-ipca = sin curva del Tesouro IPCA+ (haz la recolección de cotizaciones)
nucleo-score-rf-sem-curva-prefixado = sin curva del Tesouro Prefixado (haz la recolección de cotizaciones)
nucleo-score-rf-nota-pct-cdi = { $pct }% del CDI con CDI de { $cdi }% anual
nucleo-score-rf-isento = exento de impuesto sobre la renta para personas físicas
nucleo-score-rf-tributado = gravado por la tabla regresiva
nucleo-score-rf-nota-credito-privado = Garantías, covenants, subordinación y calificación no tienen fuente estructurada gratuita: la dimensión Estructura se imputa (§7.5). Spread calculado con la tasa contratada frente a la curva del Tesoro; las tasas indicativas de ANBIMA no son datos abiertos.

## Modelos clásicos

nucleo-score-piotroski-roa = ROA positivo
nucleo-score-piotroski-fco = Flujo de caja operativo positivo
nucleo-score-piotroski-roa-maior = ROA mayor que en el año anterior
nucleo-score-piotroski-accrual = Caja operativa ÷ activo mayor que el ROA (calidad del devengo)
nucleo-score-piotroski-alavancagem = Apalancamiento de largo plazo menor
nucleo-score-piotroski-liquidez = Liquidez corriente mayor
nucleo-score-piotroski-sem-emissao = No emitió acciones
nucleo-score-piotroski-margem = Margen bruto mayor
nucleo-score-piotroski-giro = Rotación del activo mayor
nucleo-score-leitura-forte = fuerte
nucleo-score-leitura-fraco = débil
nucleo-score-leitura-intermediario = intermedio
nucleo-score-piotroski-nota = { $n } de 9 criterios calculables con los datos disponibles
nucleo-score-leitura-zona-segura = zona segura
nucleo-score-leitura-zona-cinza = zona gris
nucleo-score-leitura-zona-estresse = zona de estrés
nucleo-score-altman-x1 = X1 capital de trabajo ÷ activo
nucleo-score-altman-x2 = X2 utilidades retenidas ÷ activo
nucleo-score-altman-x3 = X3 EBIT ÷ activo
nucleo-score-altman-x4 = X4 patrimonio contable ÷ pasivo
nucleo-score-altman-referencia = Altman (1968); Altman, Hartzell y Peck (1995)
nucleo-score-beneish-dsri = DSRI cuentas por cobrar/ingresos
nucleo-score-beneish-gmi = GMI margen bruto
nucleo-score-beneish-aqi = AQI calidad del activo
nucleo-score-beneish-sgi = SGI crecimiento de los ingresos
nucleo-score-beneish-depi = DEPI tasa de depreciación
nucleo-score-beneish-sgai = SGAI gastos de venta y administrativos
nucleo-score-beneish-tata = TATA devengos totales
nucleo-score-beneish-lvgi = LVGI apalancamiento
nucleo-score-leitura-manipulacao = probable manipulación (revisar)
nucleo-score-leitura-sem-manipulacao = sin señal de manipulación
nucleo-score-beneish-nota = sin dato para { $variaveis } (entraron como neutros)

## Evaluación y segmento

nucleo-score-erro-veredito = el veredicto es "discordo" (en desacuerdo) o "concordo" (de acuerdo)
nucleo-score-erro-motivo = explica el motivo (hasta 2.000 caracteres)
nucleo-score-erro-ticker = indica el ticker
nucleo-score-erro-segmento = el segmento "{ $segmento }" no existe para esta clase

## Fondos

nucleo-score-fundo-doc-informe = informe diario de la CVM hasta el { $data }
nucleo-score-fundo-sem-volume = sin volumen negociado (haz la recolección de cotizaciones)
nucleo-score-fundo-fonte-convertido = { $fonte }; convertido de { $moeda } con la tasa del { $data }
nucleo-score-fundo-sem-cambio = sin tipo de cambio { $moeda }/BRL para convertir el volumen
nucleo-score-fundo-fonte-cotas = cuotas del informe diario de la CVM
nucleo-score-fundo-poucas-cotas = menos de 12 meses de cuotas para { $para }
nucleo-score-fundo-para-indice = el índice
nucleo-score-fundo-para-benchmark = el benchmark
nucleo-score-fundo-para-drawdown = el drawdown
nucleo-score-fundo-para-recuperacao = la recuperación
nucleo-score-fundo-periodo-anos = { $de } a { $ate } ({ $anos } años)
nucleo-score-fundo-sem-cdi = sin el CDI del período (haz la recolección de índices)
nucleo-score-fundo-nota-sharpe = { $valor } (retorno { $retorno } anual × CDI { $cdi } anual; volatilidad { $volatilidade })
nucleo-score-fundo-volatilidade-zero = volatilidad cero
nucleo-score-fundo-sortino-teto = ningún día por debajo del CDI: Sortino en el tope
nucleo-score-fundo-nota-benchmark = benchmark: { $benchmark }
nucleo-score-fundo-benchmark-cdi = el benchmark es el CDI: el Sharpe ya mide el retorno sobre él
nucleo-score-fundo-nota-information-ratio = { $valor } (tracking error { $te })
nucleo-score-fundo-tracking-zero = tracking error cero
nucleo-score-fundo-sem-serie = sin la serie del { $benchmark } en el período
nucleo-score-fundo-nota-janelas = { $acima } de { $janelas } ventanas por encima del { $benchmark }
nucleo-score-fundo-poucas-janelas = menos de { $minimo } ventanas de 12 meses con cuotas y { $benchmark }
nucleo-score-fundo-sem-queda = sin caída en la ventana
nucleo-score-fundo-recuperada = , recuperada el { $data }
nucleo-score-fundo-nao-recuperada = , aún no recuperada
nucleo-score-fundo-periodo-cotas = { $n } cuotas hasta el { $data }
nucleo-score-fundo-poucas-cotas-12m = menos de { $n } cuotas en los últimos 12 meses
nucleo-score-fundo-periodo-extrato = extracto del { $data }
nucleo-score-fundo-fonte-extrato = CVM: extracto de información de los fondos
nucleo-score-fundo-sem-extrato = fondo sin extracto de información en la CVM
nucleo-score-fundo-nota-performance = sobre lo que supere { $benchmark }
nucleo-score-fundo-fonte-patrimonio = CVM: informe diario (VL_PATRIM_LIQ)
nucleo-score-fundo-periodo-registro = registro del { $data }
nucleo-score-fundo-fonte-registro = CVM: registro de fondos y clases
nucleo-score-fundo-sem-patrimonio = sin patrimonio en el informe diario ni en el registro de la CVM
nucleo-score-fundo-fonte-cotistas = CVM: informe diario (NR_COTST)
nucleo-score-fundo-sem-cotistas = sin número de cuotistas en el informe diario
nucleo-score-fundo-periodo-captacao = { $meses } meses, { $de } a { $ate } ({ $documento })
nucleo-score-fundo-fonte-captacao = CVM: informe diario (CAPTC_DIA − RESG_DIA)
nucleo-score-fundo-nota-captacao = captación neta de { $valor }
nucleo-score-fundo-pouca-captacao = menos de 11 meses de captación en el informe diario
nucleo-score-fundo-periodo-constituido = constituido el { $data }
nucleo-score-fundo-nota-tempo-gestao = aproximación del tiempo de gestión
nucleo-score-fundo-sem-constituicao = sin fecha de constitución en el registro de la CVM
nucleo-score-fundo-retorno-meses = { $meses } meses { $pct } anual{ $benchmark }
nucleo-score-fundo-nota-retorno = Retorno de la cuota: { $retornos }.
nucleo-score-fundo-flag-patrimonio = patrimonio de { $valor }
nucleo-score-fundo-flag-resgates = captación neta de { $pct } del patrimonio en 12 meses
nucleo-score-fundo-flag-queda = caída máxima de { $pct }
nucleo-score-fundo-cotacao-manual = cotización manual: registra las cuotas en el activo o elige la fuente CVM
nucleo-score-fundo-sem-cnpj-provavel = fondo sin CNPJ: por el nombre, el probable es { $cnpj } ({ $nome }, en el registro de la CVM); revísalo e indica el CNPJ en el activo para buscar las cuotas
nucleo-score-fundo-sem-cnpj = fondo sin CNPJ: indica el CNPJ en el activo para buscar las cuotas en la CVM
nucleo-score-fundo-cnpj-plano-provavel = el CNPJ { $cnpj } no está en el registro de fondos de la CVM (en previsión suele ser el del plan); por el nombre, el fondo probable es { $provavel } ({ $nome }); revísalo y cambia el CNPJ en el activo para buscar las cuotas
nucleo-score-fundo-fonte-nao-cvm = la fuente de cotización del activo no es la CVM: elige CVM (o déjala automática) para buscar las cuotas por el CNPJ
nucleo-score-fundo-cnpj-fora-do-registro = el CNPJ { $cnpj } no apareció en el registro de fondos de la CVM descargado el { $data }{ $tipo ->
    [previdencia] : en planes de pensiones, indica el CNPJ del fondo (FIE) que usa el plan, no el del plan en la Susep
   *[fundo] : revisa el CNPJ en el activo
}
nucleo-score-fundo-sem-cotas = sin cuotas guardadas (haz la recolección de cotizaciones)
nucleo-score-fundo-dado-cotas = Cuotas del fondo
nucleo-score-fundo-so-cadastrados = el score de fondos solo vale para fondos registrados (con CNPJ)
nucleo-score-fundo-dado-cotas-cvm = Cuotas del fondo (CVM)
nucleo-score-fundo-cadastro-classe = clase CVM { $classe }
nucleo-score-fundo-cadastro-indicador = indicador declarado: { $indicador }
nucleo-score-fundo-cadastro-gestor = gestor { $gestor }
nucleo-score-fundo-nota-cadastro = Registro en la CVM: { $cadastro }.
nucleo-score-fundo-nota-exclusivo = Fondo exclusivo: se esperan pocos cuotistas.
nucleo-score-fundo-nota-sem-cadastro = Fondo sin registro de la CVM descargado: segmento predeterminado y sin comisiones (haz la recolección del análisis de activos).
nucleo-score-fundo-nota-benchmark-pares = Benchmark: { $benchmark }. Los fondos no tienen universo de pares en la app: las notas usan solo las referencias absolutas de la clase.

## ETF

nucleo-score-etf-domicilio-br = ETF domiciliado en Brasil
nucleo-score-etf-domicilio-us = ETF domiciliado en EE. UU.: 30% retenido sobre dividendos y riesgo del impuesto a la herencia estadounidense
nucleo-score-etf-domicilio-ucits = ETF listado en Europa (en general UCITS irlandés): 15% retenido sobre dividendos estadounidenses
nucleo-score-etf-domicilio-outro = ETF listado fuera de Brasil, EE. UU. y Europa
nucleo-score-etf-dado-cotacoes = Cotizaciones del ETF
nucleo-score-etf-so-cadastrados = el score de ETF solo vale para ETF registrados
nucleo-score-etf-fonte-cotacoes = cotizaciones ({ $fonte })
nucleo-score-etf-sem-indice = sin serie del índice de referencia en la app
nucleo-score-etf-indice-convertido = { $indice } en { $moeda } con el tipo de cambio del día
nucleo-score-etf-sem-serie = sin la serie { $indice } (o el tipo de cambio) en el período (haz la recolección de índices)
nucleo-score-etf-sem-dados = sin datos para comparar
nucleo-score-etf-fonte-indice = { $fonte }; índice { $indice }
nucleo-score-etf-nota-tracking = ETF { $etf } × índice { $indice }
nucleo-score-etf-periodo-semanal = { $periodo }, retornos semanales
nucleo-score-etf-nota-cambio = índice convertido con el tipo de cambio del día: incluye el desfase de horario entre el tipo de cambio y los cierres
nucleo-score-etf-poucas-semanas = pocas semanas para el tracking error
nucleo-score-etf-sem-cnpj = ETF sin CNPJ encontrado en el registro de la CVM (indica el CNPJ en el activo y haz la recolección del análisis de activos)
nucleo-score-etf-sem-patrimonio = patrimonio de ETF en el exterior sin fuente estructurada gratuita
nucleo-score-etf-periodo-classe-bolsa = clase y bolsa del activo
nucleo-score-etf-fonte-tabela = tabla de la metodología
nucleo-score-etf-domicilio-sem-nota = domicilio sin nota en la metodología
nucleo-score-etf-sem-constituicao = sin fecha de constitución (solo ETF de la B3 con CNPJ en el registro de la CVM)
nucleo-score-etf-periodo-carteira = cartera teórica { $indice } del { $data }
nucleo-score-etf-fonte-carteira = B3: cartera teórica del índice
nucleo-score-etf-fonte-componentes = score de salud de cada componente
nucleo-score-etf-nota-componentes = { $com_score } de { $total } componentes con score ({ $peso } del peso)
nucleo-score-etf-pouca-cobertura = solo { $peso } del peso de la cartera con score (mínimo 50%; haz la recolección del análisis de activos)
nucleo-score-etf-sem-carteira = sin la cartera teórica del { $indice } (haz la recolección del análisis de activos)
nucleo-score-etf-sem-composicao = composición sin fuente estructurada gratuita (solo los índices de la B3)
nucleo-score-etf-periodo-fechamentos = { $n } cierres hasta el { $data }
nucleo-score-etf-poucos-fechamentos = menos de { $n } cierres en los últimos 12 meses (haz la recolección de cotizaciones)
nucleo-score-etf-nota-indice = Índice de referencia: { $indice }.
nucleo-score-etf-nota-cnpj = CNPJ en la CVM: { $cnpj }.
nucleo-score-etf-nota-sem-composicao = Sin la composición del ETF: la salud de los componentes y la concentración dan lugar al riesgo (volatilidad y drawdown).
nucleo-score-etf-nota-sem-pares = Los ETF no tienen universo de pares en la app: las notas usan solo las referencias absolutas.

## Fondos y ETF: recolección

nucleo-score-erro-b3-carteira-invalida = respuesta de la B3 no válida: { $erro }
nucleo-score-erro-b3-sem-carteira = respuesta de la B3 sin la cartera
nucleo-score-erro-carteira-vazia = cartera teórica vacía
nucleo-score-erro-registro-sem-classes = registro de la CVM sin registro_classe.csv
nucleo-score-passo-yahoo-etf = Yahoo: nombre del ETF { $ticker }
nucleo-score-passo-carteira = B3: cartera teórica del { $indice }

## Exterior y BDR

nucleo-score-intl-sec-desligada = { $falta }: la fuente SEC EDGAR está apagada en Ajustes › Fuentes de datos
nucleo-score-intl-sec-email = { $falta }: la SEC solo atiende a quien indica un correo de contacto; indica el tuyo en la pestaña Salud del activo (o en Ajustes › Fuentes de datos › Ajustes del análisis) y la recolección de la SEC corre en el momento
nucleo-score-intl-sec-coleta = { $falta } (la recolección diaria del análisis de activos aún no lo trajo; el resultado está en Activos › Datos del análisis)
nucleo-score-intl-dado-demonstracoes = Estados financieros
nucleo-score-intl-sem-fonte = sin fuente gratuita de estados financieros para bolsas fuera de Brasil y EE. UU. (solo cotización)
nucleo-score-intl-sem-sec = no hay estados financieros de la SEC descargados para este activo
nucleo-score-intl-dado-sec = Estados financieros (SEC EDGAR)
nucleo-score-intl-dado-historico = Historial de balances
nucleo-score-intl-listada-ha-pouco = empresa listada hace poco: aún sin balance anual (10-K) en la SEC; el score sale después del primer ejercicio completo
nucleo-score-intl-nota-sec = Estados financieros de la SEC (US GAAP) en dólares; crecimiento real descontado por el CPI de EE. UU. Sin dimensión de gobernanza (sin fuente estructurada gratuita).
nucleo-score-intl-nota-financeira = Banco o aseguradora estadounidense: sin indicadores prudenciales (Basilea, morosidad); score por el ROE, la valuación y la estabilidad.
nucleo-score-intl-dado-subjacente = Activo subyacente del BDR
nucleo-score-intl-bdr-sem-empresa = no fue posible identificar la empresa del BDR
nucleo-score-intl-bdr-sem-sec = sin estados financieros de la SEC para { $ticker }, subyacente probable del BDR
nucleo-score-intl-dado-subjacente-sec = Estados financieros del subyacente (SEC EDGAR)
nucleo-score-intl-nota-bdr = BDR: hereda el score de { $empresa } ({ $ticker }), el subyacente.

## Métricas de los estados financieros

nucleo-score-metrica-fonte-dfp = CVM DFP
nucleo-score-metrica-fonte-fca = CVM FCA (valores mobiliarios)
nucleo-score-metrica-nota-ebitda-medio = EBITDA promedio de los últimos ejercicios (normalización cíclica)
nucleo-score-metrica-ebitda-medio-negativo = EBITDA promedio ≤ 0
nucleo-score-metrica-capital-negativo = capital invertido (deuda neta + PN) ≤ 0
nucleo-score-metrica-divisao-zero = división por cero
nucleo-score-metrica-nota-divida-ebitda = deuda actual ÷ EBITDA promedio
nucleo-score-metrica-sem-altman = faltan capital de trabajo, utilidades retenidas, EBIT o pasivo
nucleo-score-metrica-sem-ciclo = faltan cuentas por cobrar, proveedores, ingresos o costo de ventas
nucleo-score-metrica-lucro-anterior-negativo = utilidad del ejercicio anterior ≤ 0
nucleo-score-metrica-lucro-negativo-razao = utilidad neta ≤ 0 (la razón no aplica)
nucleo-score-metrica-lpa-final-negativo = UPA final ≤ 0
nucleo-score-metrica-lpa-inicial-negativo = UPA inicial ≤ 0
nucleo-score-metrica-lucro-negativo-sem-proventos = utilidad ≤ 0 y sin dividendos
nucleo-score-metrica-poucos-fechamentos = menos de 120 cierres en el último año
nucleo-score-metrica-poucos-lucros-2 = menos de 2 ejercicios con utilidad
nucleo-score-metrica-poucos-ebitda = menos de 3 ejercicios con EBITDA
nucleo-score-metrica-poucos-roic = menos de 3 ejercicios con ROIC
nucleo-score-metrica-poucos-lucros-3 = menos de 3 ejercicios con utilidad neta
nucleo-score-metrica-poucos-roe = menos de 4 ejercicios con ROE
nucleo-score-metrica-poucos-lpa = menos de 6 ejercicios con utilidad y acciones
nucleo-score-metrica-periodo-media-exercicios = promedio de los últimos ejercicios
nucleo-score-metrica-sem-dividendos-12m = no pagó dividendos en los últimos 12 meses
nucleo-score-metrica-proventos-com-prejuizo = pagó dividendos con utilidad ≤ 0
nucleo-score-metrica-patrimonio-negativo = patrimonio neto ≤ 0
nucleo-score-metrica-sem-piotroski = necesita 2 ejercicios con las partidas del Piotroski
nucleo-score-metrica-receita-negativa = ingresos iniciales o finales ≤ 0
nucleo-score-metrica-nota-receitas-financeiras = los ingresos financieros superan los gastos financieros
nucleo-score-metrica-sem-segmento-listagem = segmento de listado no informado en el FCA
nucleo-score-metrica-sem-ebit = sin EBIT
nucleo-score-metrica-sem-ebit-12m = sin EBIT de 12 meses
nucleo-score-metrica-sem-ebit-despesas = sin EBIT o gastos financieros
nucleo-score-metrica-sem-ebitda = sin EBITDA
nucleo-score-metrica-sem-ipca = sin IPCA del período: crecimiento nominal
nucleo-score-metrica-sem-circulante = sin activo o pasivo corriente
nucleo-score-metrica-sem-ativo-anterior = sin activo total de 12 meses antes
nucleo-score-metrica-sem-caixa-divida-cp = sin caja o deuda de corto plazo
nucleo-score-metrica-sem-cotacao-acoes = sin cotización o número de acciones
nucleo-score-metrica-sem-cotacoes-12m = sin cotizaciones de 12 meses
nucleo-score-metrica-sem-regra-40 = sin crecimiento de ingresos o margen de flujo de caja libre
nucleo-score-metrica-sem-entregas = sin fechas de entrega de los estados financieros
nucleo-score-metrica-sem-dividendos = sin dividendos declarados o pagados
nucleo-score-metrica-sem-dividendos-pagos = sin dividendos pagados
nucleo-score-metrica-sem-divida-cp = sin deuda de corto plazo
nucleo-score-metrica-sem-divida-caixa = sin deuda y caja en el balance
nucleo-score-metrica-sem-divida-liquida = sin deuda neta
nucleo-score-metrica-sem-fcl = sin flujo de caja libre
nucleo-score-metrica-sem-fco-lucro = sin flujo de caja operativo o utilidad
nucleo-score-metrica-sem-fco-receita = sin flujo de caja operativo o ingresos
nucleo-score-metrica-sem-lucro = sin utilidad
nucleo-score-metrica-sem-lucro-bruto-ativo = sin utilidad bruta (o ingresos y costo de ventas) o activo total
nucleo-score-metrica-sem-lucro-12m = sin utilidad neta de 12 meses
nucleo-score-metrica-sem-lucro-ativo = sin utilidad o activo total
nucleo-score-metrica-sem-lucro-fco-ativo = sin utilidad, flujo de caja operativo o activo
nucleo-score-metrica-sem-acoes-anterior = sin número de acciones de 12 meses antes
nucleo-score-metrica-sem-patrimonio = sin patrimonio neto
nucleo-score-metrica-sem-patrimonio-ativo = sin patrimonio o activo
nucleo-score-metrica-sem-proventos-cotacao = sin dividendos o cotización
nucleo-score-metrica-sem-receita-ativo = sin ingresos o activo
nucleo-score-metrica-sem-receita-positiva = sin ingresos positivos
nucleo-score-metrica-sem-retornos = sin retornos
nucleo-score-metrica-ev-negativo = valor de la empresa ≤ 0 (caja por encima del valor de mercado)
nucleo-score-metrica-periodo-ultimo-balanco = último balance
nucleo-score-metrica-sem-lucro-liquido = sin utilidad neta
nucleo-score-metrica-media-exercicios = promedio de { $n } ejercicios
nucleo-score-metrica-periodo-exercicio = ejercicio hasta el { $data }
nucleo-score-metrica-poucos-exercicios-receita = menos de { $n } ejercicios con ingresos
nucleo-score-metrica-nota-nominal = nominal { $pct }% anual, IPCA descontado
nucleo-score-metrica-periodo-n-exercicios = { $n } ejercicios
nucleo-score-metrica-periodo-entregas = últimas { $n } entregas
nucleo-score-metrica-fonte-recebimento = CVM (fecha de recepción)
nucleo-score-metrica-periodo-pregoes = { $n } sesiones hasta el { $data }
nucleo-score-metrica-fonte-fechamentos = cierres diarios

## Cripto: documentos

nucleo-score-cripto-doc-market-chart = CoinGecko market_chart (365 días)
nucleo-score-fundo-dado-cotas-extrato = Cuotas del plan (extracto)
nucleo-score-fundo-efpc-sem-cotas = un fondo de pensión (EFPC, fiscalizado por la Previc) no es un fondo de la CVM: la cuota del plan solo viene del extracto del participante; importa el extracto en PDF en el activo para registrar la cuota mes a mes
nucleo-score-fii-trimestral-sem-item = el informe trimestral hasta el { $data } no trae el dato (campo en blanco o en cero)
nucleo-score-fii-nota-por-area = ponderada por el área: el informe trimestral no trae la participación de cada inmueble en los ingresos
nucleo-score-confianca-dy-estimado = DY estimado con los informes de la CVM (sin los proventos pagados en la B3)
