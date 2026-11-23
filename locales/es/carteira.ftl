# Pantalla 08 · Cartera: pestañas, resumen, posiciones, rentabilidad, asignación y rebalanceo,
# operaciones, eventos corporativos, dividendos, ingresos pasivos, costo de las comisiones y
# riesgo, más las etiquetas de las clases, de los dividendos y de las fuentes que usan también
# otras pantallas (docs/i18n-design.md). Las importaciones de B3 y de las notas de corretaje
# quedan para la fase 5.
# Revisión: borrador por IA, sin revisión de un hablante nativo.

## Pestañas y encabezado

carteira-secoes = Secciones de la cartera
carteira-aba-posicoes = Posiciones
carteira-aba-rentabilidade = Rentabilidad
carteira-aba-alocacao = Asignación y rebalanceo
carteira-aba-operacoes = Operaciones
carteira-aba-proventos = Dividendos
carteira-aba-renda = Ingresos pasivos
carteira-aba-taxas = Costo de las comisiones
carteira-aba-risco = Riesgo
carteira-aba-saude = Salud de los activos
carteira-aba-bens = Bienes
carteira-carregando = Calculando la cartera
carteira-meta = Posición al { $data } · real por { $deflator }
carteira-evento-corporativo = Evento corporativo
carteira-registrar-proventos = Registrar dividendos
carteira-provento-detalhado = Dividendo detallado
carteira-nova-operacao = Nueva operación
carteira-importar-b3 = Importar de B3
carteira-importar-notas = Importar notas
carteira-real-descontado-por = Resultado real descontado por
carteira-eventos-corporativos = Eventos corporativos

## Cartera vacía

carteira-vazia-kicker = Cartera vacía
carteira-vazia-titulo = Registra la primera operación
carteira-vazia-texto = Registra el activo en 09 · Activos (o elige uno ya registrado) y anota la compra con sus costos. La cartera calcula el precio promedio, el resultado, la XIRR y el rendimiento real descontada la inflación.
carteira-cadastrar-ativo = Registrar activo

## Activos valuados al costo

carteira-avaliados-pelo-custo =
    { $n ->
        [one] 1 activo valuado
       *[other] { $n } activos valuados
    } al costo: { $lista }
carteira-lista-e-mais = { $lista } y { $n } más
carteira-avaliados-texto = Sin cotización, el valor de mercado es el costo y el resultado no realizado queda en cero. Actualiza las cotizaciones en { $link } (Yahoo Finance y Tesouro Transparente) o ingresa la cotización a mano en la página del activo. Los fondos (cuotas de la CVM) llegan en una próxima etapa.
carteira-link-indices = 09 · Activos › Índices y cotizaciones

## Barra de comandos

carteira-cmd-evento-busca = desdoblamiento,agrupamiento,bonificación,split,fusión
carteira-cmd-ver-aba = Ver { $aba }
carteira-cmd-ver-aba-busca = pestaña,mostrar
carteira-cmd-operacao = Nueva operación (compra o venta)
carteira-cmd-operacao-busca = registrar,comprar,vender,trade,nota de corretaje,inversión
carteira-cmd-provento = Registrar dividendo
carteira-cmd-provento-busca = dividendos,jcp,rendimiento,fii,cupón
carteira-cmd-proventos = Registrar varios dividendos (fecha, ticker y monto)
carteira-cmd-proventos-busca = dividendos,jcp,rendimientos,cuadrícula,lote,pegar
carteira-cmd-importar-b3 = Importar de B3 (Área del Inversionista)
carteira-cmd-importar-b3-busca = b3,área del inversionista,negociación,movimientos,posición,excel
carteira-cmd-importar-notas = Importar notas de corretaje (PDF)
carteira-cmd-importar-notas-busca = nota de corretaje,sinacor,pdf,comisiones,cargos,irrf
carteira-cmd-ativo-busca = nuevo activo,acción,fii,etf,tesouro,cdb,renta fija,cripto
carteira-cmd-cotacoes = Actualizar índices, tipo de cambio y cotizaciones
carteira-cmd-cotacoes-busca = ipca,cdi,ptax,dólar,sincronizar,yahoo,recolectar
carteira-cmd-agenda = Agenda de dividendos
carteira-cmd-agenda-busca = fecha de corte,pago,dividendos anunciados
carteira-cmd-watchlist = Watchlist
carteira-cmd-watchlist-busca = favoritos,seguir,precio objetivo,alertas
carteira-cmd-rankings = Rankings de activos
carteira-cmd-rankings-busca = dividend yield,p/u,graham,bazin,mayores,ranking
carteira-cmd-screener = Screener (filtro de activos)
carteira-cmd-screener-busca = filtro,pantallas,rastreador,indicadores
carteira-cmd-comparador = Comparar activos
carteira-cmd-comparador-busca = comparador,lado a lado,base 100
carteira-cmd-analise = Recolectar datos del análisis (CVM, B3, SEC)
carteira-cmd-analise-busca = fundamentales,cvm,sec,dfp,recolectar,sincronizar
carteira-cmd-grupo-ativos = Inversiones · activos
carteira-cmd-abrir-ativo-busca = abrir activo,cotización,gráfico

## Estados comunes

carteira-falha-desconhecida = Falla desconocida.
carteira-erro-carregar = No se pudo cargar
carteira-excluir = Eliminar
carteira-total = Total
carteira-total-valor = Total: { $valor }
carteira-opcional = Opcional.
carteira-valor-invalido = Monto no válido.
carteira-escolha-ativo = Elige el activo
carteira-escolha-conta = Elige la cuenta
carteira-nenhuma-corretora = Ninguna corredora registrada
carteira-n-meses = { $n } meses
carteira-no-ano = En el año
carteira-periodo = Período
carteira-acoes-de = Acciones de { $nome }
carteira-e = y
carteira-pp = p.p.
carteira-valor-pp = { $valor } p.p.
carteira-pct-aa = { $pct } anual
carteira-data-formato = dd/mm/aaaa

## Clases de activo (los mismos nombres del núcleo)

carteira-classe-stock-br = Acciones
carteira-classe-fii = FII
carteira-classe-etf-br = ETF
carteira-classe-bdr = BDR
carteira-classe-fixed-income = Renta fija
carteira-classe-treasury-br = Tesouro Direto
carteira-classe-fund = Fondos
carteira-classe-stock-us = Acciones en EE. UU.
carteira-classe-etf-us = ETF en EE. UU.
carteira-classe-reit = REIT
carteira-classe-crypto = Cripto
carteira-classe-pension = Planes de pensiones
carteira-classe-other = Otros
carteira-classe-stock-intl = Acciones en otras bolsas
carteira-classe-etf-intl = ETF en otras bolsas

## Tipos de dividendo y de evento corporativo

carteira-provento-dividend = Dividendos
carteira-provento-fii-income = Rendimientos de FII
carteira-provento-coupon = Cupón
carteira-provento-interest = Intereses
carteira-provento-rent = Préstamo de acciones
carteira-provento-amortization = Amortización
carteira-provento-other = Otros pagos

carteira-evento-split = Desdoblamiento
carteira-evento-reverse-split = Agrupamiento
carteira-evento-bonus = Bonificación
carteira-evento-subscription = Suscripción
carteira-evento-merger = Fusión
carteira-evento-ticker-change = Cambio de ticker
carteira-evento-spin-off = Escisión
carteira-evento-capital-return = Devolución de capital

## Deflactor del resultado real

carteira-deflator-rotulo = Deflactor del resultado real
carteira-deflator-ipca-dica = Inflación oficial de Brasil (IBGE, vía BCB/SGS 433)
carteira-deflator-pais = Inflación del país
carteira-deflator-pais-de = Inflación del país ({ $pais })
carteira-deflator-pais-dica = Inflación oficial del país de referencia (Ajustes › Fuentes de datos)
carteira-deflator-pais-serie = { $pais }: { $serie } ({ $fonte }). Cambia el país en Ajustes › Fuentes de datos.
carteira-deflator-us-cpi-dica = Inflación al consumidor de EE. UU. (FRED CPIAUCSL)
carteira-deflator-m2-br-dica = Expansión monetaria de Brasil (BCB/SGS 27810)
carteira-deflator-m2-us = M2 EE. UU.
carteira-deflator-m2-us-dica = Expansión monetaria de EE. UU. (FRED M2SL)
carteira-deflator-nominal = Nominal
carteira-deflator-nominal-dica = Sin descontar la inflación
carteira-fonte-oficial = fuente oficial

## Renta fija

carteira-rf-debenture = Debenture
carteira-rf-debenture-incentivada = Debenture incentivada
carteira-rf-poupanca = Cuenta de ahorros
carteira-rf-outro = Otro
carteira-indexador-prefixado = Prefijado
carteira-liquidez-diaria = Diaria
carteira-liquidez-vencimento = Al vencimiento
carteira-liquidez-carencia = Tras el período de carencia
carteira-taxa-dica-percentual = % del { $indice } (p. ej.: 110 para 110% del { $indice })
carteira-taxa-dica-prefixado = Tasa anual (p. ej.: 12,5 para 12,5% anual)
carteira-taxa-dica-real = Tasa real anual por encima del { $indice } (p. ej.: 6,2 para { $indice } + 6,2% anual)
carteira-termos-percentual = { $taxa }% del { $indice }
carteira-termos-pre = Prefijado { $taxa }% anual
carteira-termos-indice = { $indice } + { $taxa }% anual
carteira-termos-vencimento = vence { $data }

## Valuación y fuentes

carteira-avaliacao-mercado = Cotización de mercado
carteira-avaliacao-curva = Valuación en la curva
carteira-avaliacao-sem-cotacao = Sin cotización (al costo)
carteira-fonte-b3-listados = B3 (dividendos oficiales)
carteira-fonte-yahoo-intraday = Yahoo Finance (intradía, ~15 min de retraso)
carteira-fonte-manual-voce = Ingresado por ti
carteira-fonte-mock = Datos de ejemplo
carteira-fonte-radar = Radar de dividendos (proyección)
carteira-fonte-bcb = Banco Central de Brasil
carteira-fonte-oecd = OCDE
carteira-fonte-worldbank = Banco Mundial (anual, interpolado)
carteira-fonte-bce = BCE
carteira-fonte-bce-ref = BCE (referencia)
carteira-fonte-nao-informada = Fuente no indicada
carteira-fonte-manual = Manual
carteira-fonte-sem-cotacao = Sin cotización
carteira-fonte-na-curva = En la curva
carteira-fonte-manual-sem-fonte = Manual (sin fuente automática)
carteira-fonte-manual-bolsa = Manual (indica la bolsa)

## Cotización y medidor nominal × real

carteira-cotacao-intradia = Cotización de { $quando }, con unos { $min } min de retraso; { $sessao }. El historial usa los cierres diarios.
carteira-cotacao-defasada = Cotización desactualizada: último cierre conocido antes de la fecha de la posición.
carteira-cotacao-ultimo-pregao = Última sesión cerrada antes de la fecha de la posición.
carteira-cotacao-fechamento = Cierre de la sesión.
carteira-na-curva = en la curva
carteira-curva-fonte = Valuación en la curva (CDI/IPCA del BCB)
carteira-curva-nota = Renta fija valuada por la tasa contratada sobre el indexador, base 252 días hábiles.
carteira-sem-cotacao = sin cotización
carteira-sem-cotacao-dica = Sin cotización: el valor es el costo. Actualiza las cotizaciones o ingresa la cotización a mano en la página del activo.
carteira-medidor = Nominal { $nominal }; real ({ $deflator }) { $real }
carteira-medidor-oculto = Nominal × real ({ $deflator }): valores ocultos

## Resumen (instrumentos superiores)

carteira-resumo-fonte-sem-cotacao = Cotizaciones, curva y costo ({ $n } sin cotización)
carteira-resumo-fonte = Cotizaciones de cierre y valuación en la curva
carteira-resumo-estimado = Esta cifra incluye tipo de cambio, índice o cotización estimados (o activos valuados al costo).
carteira-valor-mercado = Valor de mercado
carteira-custo = Costo
carteira-em = Al
carteira-patrimonio-investido = Patrimonio invertido
carteira-nao-realizado-pct = no realizado { $pct }
carteira-resultado = Resultado
carteira-nao-realizado = No realizado
carteira-realizado = Realizado
carteira-proventos = Dividendos
carteira-resultado-total = Resultado total
carteira-resultado-perspectiva = Perspectiva del resultado
carteira-resultado-nominal = Nominal
carteira-resultado-real-de = Resultado real ({ $indice })
carteira-resultado-efeito = Efecto del { $indice }
carteira-resultado-sem-indice = Sin el { $indice } del período: falta recolectar el índice
carteira-sobre-custo = sobre el costo · { $pct }
carteira-xirr-titulo = Rendimiento anual (XIRR)
carteira-xirr-sem-dados-ipca = Sin datos de { $deflator } hasta el { $data }: actualiza los índices en 09 · Activos › Índices y cotizaciones.
carteira-xirr-sem-dados = Sin datos de { $deflator } hasta el { $data }: el índice sale con retraso y el núcleo todavía no estima los meses que faltan.
carteira-deflator = Deflactor
carteira-metodo = Método
carteira-metodo-xirr = XIRR (fechas y montos de cada flujo)
carteira-nominal = Nominal
carteira-real-de = Real · { $deflator }
carteira-renda-12m = Ingresos pasivos 12m
carteira-media-mensal = Promedio mensual
carteira-yield = Yield
carteira-proventos-liquidos = Dividendos netos
carteira-recebidos-moeda = recibidos, en { $moeda }
carteira-carregando-curto = cargando

## Posiciones

carteira-avisos = Avisos: { $lista }
carteira-liquido-dica = Valor neto estimado de un rescate hoy: impuesto sobre la renta regresivo (22,5% hasta 180 días → 15% por encima de 720) sobre la ganancia e IOF en los primeros 30 días; los exentos (LCI, LCA, CRI, CRA, debentures incentivadas) no pagan. Es una estimación, no el cálculo del impuesto.
carteira-liquido-abrev = neto { $valor }
carteira-col-ativo = Activo
carteira-col-classe = Clase
carteira-col-qtd = Cant.
carteira-col-preco-medio = Precio promedio
carteira-col-cotacao = Cotización
carteira-col-valor = Valor
carteira-col-peso = Peso
carteira-col-resultado = Resultado
carteira-col-xirr = XIRR anual
carteira-col-nominal-real = Nom. × real
carteira-n-ativos =
    { $n ->
        [one] { $n } activo
       *[other] { $n } activos
    }
carteira-posicoes-de = Posiciones · { $grupo }
carteira-agrupar = Agrupar posiciones
carteira-por-classe = Por clase
carteira-por-conta = Por cuenta
carteira-sem-grupo = Sin agrupar
carteira-incluir-encerradas = Incluir posiciones cerradas
carteira-posicoes-atalhos = Enter o doble clic abre el activo; el botón derecho muestra los atajos.
carteira-calculando-posicoes = Calculando posiciones
carteira-sem-posicoes = Todavía no hay posiciones
carteira-sem-posicoes-texto = Registra la primera compra para ver el precio promedio, el resultado y el rendimiento real de cada activo.
carteira-conta = Cuenta
carteira-todas-contas = Todas las cuentas
carteira-todas-posicoes = Todas las posiciones

## Rentabilidad

carteira-desde-inicio = Desde el inicio
carteira-calculando-rentabilidade = Calculando la rentabilidad
carteira-sem-historico = Sin historial para calcular
carteira-sem-historico-texto = La rentabilidad aparece después de la primera operación.
carteira-fonte-catalogo = { $indice } (catálogo de índices)
carteira-rent-fonte = Fuente: operaciones, cotizaciones y dividendos · { $taxa } · { $deflator }
carteira-rent-estimado = Los meses de inflación todavía no publicados usan la proyección Focus (o el último dato) hasta que salga el índice oficial.
carteira-desde = Cartera desde { $mes }
carteira-pct-do = % del { $indice }
carteira-acumulado-real = Acumulado real
carteira-serie-carteira = Cartera
carteira-serie-real = Cartera real ({ $deflator })
carteira-serie-sp500 = { $moeda ->
        [BRL] S&P 500 (en R$)
       *[other] S&P 500 (en { $moeda })
    }
carteira-serie-dolar = Dólar
carteira-mes = Mes
carteira-rent-acumulada-titulo = Rentabilidad acumulada × { $indice } × inflación
carteira-rent-acumulada-aria-desde = Rentabilidad acumulada de la cartera desde { $mes }, comparada con el { $indice } y la inflación.
carteira-rent-acumulada-aria-inicio = Rentabilidad acumulada de la cartera desde el inicio, comparada con el { $indice } y la inflación.
carteira-rent-acumulada-tabela = Rentabilidad acumulada por mes
carteira-retorno-mes = Rendimiento del mes
carteira-retorno-cada-mes = Rendimiento de cada mes
carteira-retorno-metodo = Método: TWR mensual (Modified Dietz), neutraliza aportes y retiros
carteira-retorno-aria = Rendimiento mensual de la cartera en barras, ganancias arriba y pérdidas debajo de cero.
carteira-retorno-mensal = Rendimiento mensual
carteira-retorno = Rendimiento
carteira-patrimonio = Patrimonio
carteira-aportes-liquidos = Aportes netos
carteira-patrimonio-aportes = Patrimonio × aportes netos
carteira-patrimonio-aportes-fonte = Aportes netos = compras − ventas, acumulados; la distancia es la ganancia
carteira-patrimonio-aportes-aria = Patrimonio invertido comparado con los aportes netos acumulados, por mes.
carteira-patrimonio-aportes-tabela = Patrimonio y aportes por mes

## Gráfico de precio (página del activo)

carteira-grafico-pm = PP
carteira-grafico-fechamento = Cierre
carteira-grafico-preco-operacao = Precio de la operación
carteira-marcador-compra = C { $qtd }
carteira-marcador-venda = V { $qtd }

## Asignación y rebalanceo

carteira-outras = Otras
carteira-outras-de = Otras ({ $classes })
carteira-investido = invertido
carteira-sem-alvo = sin objetivo
carteira-alvo-pct = objetivo { $pct }
carteira-alocacao-classe = Asignación por clase
carteira-alocacao-fonte = Valor de mercado (o en la curva) en { $moeda }; sin cotización, al costo
carteira-alocacao-aria = Asignación de la cartera por clase de activo, en dona.
carteira-col-alvo = Objetivo
carteira-col-desvio = Desvío
carteira-desvio-peso-alvo = peso { $peso } · objetivo { $alvo }
carteira-desvio-pp = desvío { $desvio } p.p.
carteira-fora-tolerancia = fuera de la tolerancia
carteira-desvio-titulo = Desvío del objetivo, en puntos porcentuales
carteira-desvio-fonte = Barras llenas: fuera de la tolerancia definida; claras: dentro de ella
carteira-desvio-aria = Desvío de cada clase respecto a la asignación objetivo.
carteira-desvio-tabela = Desvío del objetivo
carteira-alvos-salvos = Asignación objetivo guardada
carteira-alvos-titulo = Asignación objetivo por clase
carteira-alvos-soma-passa = Suma { $total }% · pasa del 100%
carteira-alvos-soma-completa = Suma { $total }% · completa
carteira-alvos-soma-faltam = Suma { $total }% · faltan { $falta } p.p.
carteira-descartar = Descartar
carteira-salvar-alvos = Guardar objetivos
carteira-alvos-tabela = Objetivos por clase
carteira-col-atual = Actual
carteira-col-alvo-pct = Objetivo %
carteira-col-tolerancia = Tol. p.p.
carteira-alvo-de = Objetivo de { $classe } (%)
carteira-tolerancia-de = Tolerancia de { $classe } (p.p.)
carteira-adicionar-classe = Agregar clase
carteira-adicionar-classe-aria = Agregar clase a la asignación objetivo
carteira-escolha-classe = Elige la clase
carteira-tolerancia-texto = Tolerancia: cuántos puntos porcentuales puede alejarse el peso del objetivo antes de que la clase quede “fuera de la banda” (por defecto 5).
carteira-fora-tolerancia-alvo = Fuera de la tolerancia del objetivo
carteira-fora-tolerancia-alvo-curto = fuera de la tolerancia del objetivo
carteira-col-atual-pct = Actual %
carteira-col-falta-sobra = Falta/sobra
carteira-col-do-aporte = Del aporte
carteira-rebal-titulo = Rebalanceo con un nuevo aporte
carteira-novo-aporte = Nuevo aporte
carteira-novo-aporte-dica = 0 = solo muestra el desvío. Falta/sobra: + falta comprar, − está por encima del objetivo.
carteira-carteira-hoje = Cartera hoy
carteira-depois-aporte = Después del aporte
carteira-alvos-incompletos = Los objetivos no suman 100%
carteira-alvos-incompletos-texto = La sugerencia reparte el aporte solo entre las clases con objetivo; ajusta la asignación objetivo de arriba para un cálculo completo.
carteira-aporte-sem-destino = Parte del aporte no cabe en los límites
carteira-aporte-sem-destino-texto = quedan sin destino: los activos llegaron a su límite de concentración. Sube un límite o elige un activo nuevo.
carteira-defina-alvos = Define la asignación objetivo para ver cuánto falta o sobra en cada clase.
carteira-rebal-tabela = Rebalanceo por clase
carteira-nao-recomendacao = No es una recomendación de inversión
carteira-rebal-aviso = Análisis basado en tus datos y en los objetivos que definiste; no es una recomendación de inversión.
carteira-rebal-aviso-sugestao = La sugerencia solo indica cómo el aporte acerca la cartera a los objetivos, sin vender nada.
carteira-calculando-alocacao = Calculando la asignación

## Operaciones, eventos y dividendos (tablas)

carteira-compra = Compra
carteira-venda = Venta
carteira-compras = Compras
carteira-vendas = Ventas
carteira-compra-de = Compra de { $qtd } { $ativo } el { $data }
carteira-venda-de = Venta de { $qtd } { $ativo } el { $data }
carteira-compra-qtd = Compra de { $qtd } { $ativo }
carteira-venda-qtd = Venta de { $qtd } { $ativo }
carteira-erro-excluir = No se pudo eliminar
carteira-excluir-n =
    { $n ->
        [one] Eliminar
       *[other] Eliminar { $n }
    }
carteira-registros-excluir = Registros a eliminar
carteira-e-mais = y { $n } más
carteira-menu-editar = Editar…
carteira-menu-copiar-linha = Copiar fila
carteira-menu-excluir = Eliminar…
carteira-menu-copiar-n-linhas = Copiar { $n } filas
carteira-excluir-operacoes =
    { $n ->
        [one] Eliminar operación
       *[other] Eliminar { $n } operaciones
    }
carteira-excluir-operacoes-nota =
    { $n ->
        [one] El movimiento vinculado en la caja de la corredora también se elimina. Puedes deshacerlo en el historial de cambios.
       *[other] Los movimientos vinculados en la caja de la corredora también se eliminan. Puedes deshacerlos todos a la vez en el historial de cambios.
    }
carteira-operacoes-excluidas =
    { $n ->
        [one] Operación eliminada
       *[other] { $n } operaciones eliminadas
    }
carteira-operacoes-selecionadas =
    { $n ->
        [one] { $n } operación seleccionada
       *[other] { $n } operaciones seleccionadas
    }
carteira-excluir-n-operacoes =
    { $n ->
        [one] Eliminar { $n } operación…
       *[other] Eliminar { $n } operaciones…
    }
carteira-acoes-de-n-operacoes =
    { $n ->
        [one] Acciones de { $n } operación
       *[other] Acciones de { $n } operaciones
    }
carteira-n-operacoes =
    { $n ->
        [one] { $n } operación
       *[other] { $n } operaciones
    }
carteira-col-data = Fecha
carteira-col-tipo = Tipo
carteira-col-preco = Precio
carteira-col-custos = Costos
carteira-col-bruto = Bruto
carteira-col-caixa = Caja
carteira-fora-do-caixa = fuera de la caja
carteira-col-cambio = Tipo de cambio
carteira-col-conta = Cuenta
carteira-col-nota = Nota
carteira-col-acoes = Acciones
carteira-editar-operacao-de = Editar operación del { $data }
carteira-filtrar-ativo = Filtrar por activo
carteira-todos-ativos = Todos los activos
carteira-filtrar-conta = Filtrar por cuenta
carteira-operacoes = Operaciones
carteira-sem-operacoes = Ninguna operación registrada.
carteira-evento-texto = { $tipo } de { $ativo } el { $data }
carteira-excluir-eventos =
    { $n ->
        [one] Eliminar evento corporativo
       *[other] Eliminar { $n } eventos corporativos
    }
carteira-excluir-eventos-nota = La cantidad y el precio promedio se recalculan. Puedes deshacerlo en el historial de cambios.
carteira-eventos-excluidos =
    { $n ->
        [one] Evento eliminado
       *[other] { $n } eventos eliminados
    }
carteira-eventos-selecionados =
    { $n ->
        [one] { $n } evento seleccionado
       *[other] { $n } eventos seleccionados
    }
carteira-excluir-n-eventos =
    { $n ->
        [one] Eliminar { $n } evento…
       *[other] Eliminar { $n } eventos…
    }
carteira-acoes-de-n-eventos =
    { $n ->
        [one] Acciones de { $n } evento
       *[other] Acciones de { $n } eventos
    }
carteira-col-data-ex = Fecha ex
carteira-col-evento = Evento
carteira-col-detalhe = Detalle
carteira-fator = factor { $fator }
carteira-fator-custo = factor { $fator } · costo { $custo }
carteira-col-observacoes = Observaciones
carteira-col-origem = Origen
carteira-registrado-por-voce = Registrado por ti
carteira-coleta-cotacoes = { $fonte } (recolección de cotizaciones)
carteira-evento-coletado-nota = Desdoblamiento informado por la fuente de cotizaciones; se puede deshacer en el historial de cambios.
carteira-sem-eventos = Ningún evento corporativo.
carteira-excluir-proventos =
    { $n ->
        [one] Eliminar dividendo
       *[other] Eliminar { $n } dividendos
    }
carteira-excluir-proventos-nota =
    { $n ->
        [one] El crédito vinculado en la caja también se elimina. Puedes deshacerlo en el historial de cambios.
       *[other] Los créditos vinculados en la caja también se eliminan. Puedes deshacerlos todos a la vez en el historial de cambios.
    }
carteira-proventos-excluidos =
    { $n ->
        [one] Dividendo eliminado
       *[other] { $n } dividendos eliminados
    }
carteira-proventos-selecionados =
    { $n ->
        [one] { $n } dividendo seleccionado
       *[other] { $n } dividendos seleccionados
    }
carteira-excluir-n-proventos =
    { $n ->
        [one] Eliminar { $n } dividendo…
       *[other] Eliminar { $n } dividendos…
    }
carteira-acoes-de-n-proventos =
    { $n ->
        [one] Acciones de { $n } dividendo
       *[other] Acciones de { $n } dividendos
    }
carteira-col-pagamento = Pago
carteira-col-data-com = Fecha de corte
carteira-col-base = Base
carteira-col-ir-retido = Impuesto retenido
carteira-col-liquido = Neto
carteira-col-situacao = Estado
carteira-recebido = Recibido
carteira-anunciado = Anunciado
carteira-sem-proventos-periodo = Ningún dividendo en el período.
carteira-recebido-moeda = Recibido ({ $moeda })
carteira-provento-texto = { $tipo } de { $ativo } pagado el { $data }

## Dividendos e ingresos pasivos (paneles)

carteira-col-12-meses = 12 meses
carteira-col-yield-12m = Yield 12m
carteira-somando-proventos = Sumando dividendos
carteira-ultimos-12-meses = Últimos 12 meses
carteira-por-mes = Por mes
carteira-yield-carteira = Yield de la cartera
carteira-yield-formula = Dividendos ÷ valor de mercado
carteira-meta-renda = Meta de ingreso mensual
carteira-patrimonio-necessario = Patrimonio necesario
carteira-falta = Falta
carteira-quanto-receber = Cuánto quieres recibir por mes
carteira-progresso-meta = Progreso de la meta
carteira-sem-yield = Sin yield para calcular el patrimonio necesario
carteira-sem-yield-texto = La cartera todavía no recibió dividendos en los últimos 12 meses.
carteira-por-ativo-12m = Por activo · 12 meses
carteira-renda-por-ativo = Ingresos pasivos por activo
carteira-sem-proventos-12m = Ningún dividendo en los últimos 12 meses.
carteira-patrimonio-necessario-texto = Patrimonio necesario = meta × 12 ÷ yield de los últimos 12 meses. Es un cálculo sobre el pasado de tu cartera, no una promesa de ingresos.
carteira-grafico-proventos-liquidos = Dividendos netos
carteira-meta-curta = Meta
carteira-meta-valor = Meta { $valor }
carteira-proventos-mes-titulo = Dividendos por mes, últimos 12 meses
carteira-proventos-mes-fonte = Dividendos recibidos, netos del impuesto retenido, en { $moeda } en la fecha de pago
carteira-meta-mensal = Meta mensual
carteira-proventos-mes-aria = Dividendos netos recibidos por mes en los últimos 12 meses.
carteira-proventos-mes-tabela = Dividendos por mes

## Costo de las comisiones

carteira-taxas-titulo = Costo de las comisiones a largo plazo
carteira-taxas-retorno = Rendimiento real antes de comisiones (% anual)
carteira-taxas-alternativa = Comisión del escenario alternativo (% anual)
carteira-taxas-aporte = Aporte mensual (opcional)
carteira-taxas-nenhum = Ningún fondo o plan con comisión conocida
carteira-taxas-nenhum-texto = Registra la comisión de administración de los planes de pensiones; la de los fondos viene de sus informes en la CVM (recolección del análisis de activos).
carteira-taxas-tabela = Costo de las comisiones
carteira-taxas-fundo = Fondo o plan
carteira-taxas-hoje = Hoy
carteira-taxas-em-anos = En { $anos } años
carteira-taxas-economia = +{ $valor } con una comisión menor
carteira-taxa-administracao = administración { $pct }
carteira-taxa-performance = desempeño { $pct }
carteira-taxa-carregamento = carga { $pct }
carteira-taxas-sem-taxa = Sin comisión conocida (fuera del cálculo): { $lista }.

## Riesgo

carteira-risco-sem-queda = sin caída
carteira-risco-queda-inicio = { $pct } desde el inicio hasta { $fundo }
carteira-risco-queda = { $pct } de { $pico } a { $fundo }
carteira-risco-recuperou = { $queda }, se recuperó en { $mes }
carteira-risco-nao-recuperou = { $queda }, todavía no se recuperó
carteira-risco-volatilidade = Volatilidad
carteira-risco-pior-queda = Peor caída
carteira-risco-retorno = Rendimiento
carteira-risco-sharpe-de = Sharpe ({ $indice })
carteira-risco-melhor-pior = Mejor y peor mes
carteira-risco-insuficiente = Historial mensual insuficiente.
carteira-risco-retorno-aa = Rendimiento anual
carteira-risco-sharpe = Sharpe
carteira-risco-parte = Parte del riesgo
carteira-risco-sem-dados = sin datos
carteira-risco-titulo = Riesgo de la cartera
carteira-risco-texto = Cuánto oscila la cartera, la mayor caída, el rendimiento por encima del { $indice } por unidad de riesgo y cuánto se mueven juntos los activos, a partir de los rendimientos mensuales de cada activo con los pesos de hoy.
carteira-risco-calculando = Calculando el riesgo
carteira-risco-sem-ativos = Ningún activo en la cartera
carteira-risco-sem-ativos-texto = Registra operaciones en Cartera › Operaciones para medir el riesgo.
carteira-risco-hoje = Cartera de hoy en el período
carteira-risco-historico = Historial real (compras y ventas de cada momento)
carteira-risco-por-classe = Riesgo por clase
carteira-risco-correlacao-classes = Correlación entre clases
carteira-risco-por-ativo = Riesgo por activo
carteira-risco-correlacao-ativos = Correlación entre los activos más grandes
carteira-risco-parte-texto = Parte del riesgo: cuánto responde cada fila por la oscilación de la cartera (suman 100%). En amarillo, las que pesan mucho más en el riesgo que en el valor.

## Registrar operación

carteira-op-editar = Editar operación
carteira-op-registrar = Registrar operación
carteira-op-editar-texto = La operación se vuelve a guardar con los nuevos valores, junto con el movimiento vinculado en la caja de la corredora.
carteira-op-registrar-texto = Compra o venta con costos. La caja de la corredora recibe el movimiento vinculado (se puede deshacer en el historial).
carteira-erro-escolha-ativo = Elige el activo.
carteira-erro-escolha-conta-corretora = Elige la cuenta de la corredora.
carteira-erro-data = Ingresa la fecha (dd/mm/aaaa).
carteira-erro-quantidade = Ingresa una cantidad mayor que cero.
carteira-erro-venda-maior = Venta de { $qtd } mayor que la posición de { $posicao } en esta cuenta el { $data }.
carteira-na-data = esa fecha
carteira-erro-preco = Ingresa el precio unitario.
carteira-erro-cambio = El tipo de cambio debe ser positivo.
carteira-op-alterada = Operación modificada
carteira-op-registrada = Operación registrada
carteira-op-form = Operación
carteira-op-tipo = Tipo de operación
carteira-op-renda-fixa = Renta fija: cantidad de títulos × PU (precio unitario); las ventas salen por PEPS.
carteira-buscar-ativo = Buscar activo
carteira-buscar-ativo-dica = Ticker o nombre: elige uno registrado o registra uno nuevo (B3, EE. UU., otras bolsas, cripto).
carteira-ativo = Activo
carteira-conta-corretora = Cuenta de la corredora
carteira-conta-moeda = Cuenta en { $conta }; activo en { $ativo }.
carteira-data-pregao = Fecha de la sesión
carteira-quantidade = Cantidad
carteira-posicao-na-data = Posición en esta cuenta en la fecha: { $qtd }
carteira-fracoes = Fracciones con hasta 8 decimales (exterior, cripto, Tesouro).
carteira-preco-unitario = Precio unitario ({ $moeda })
carteira-cambio-moeda = Tipo de cambio { $moeda }/{ $base }
carteira-cambio-dica = { $base ->
        [BRL] 1 unidad de la moneda del activo en reales. Vacío: PTAX del día guardada en la app.
       *[other] 1 unidad de la moneda del activo en { $base }. Vacío: tipo de cambio del día guardado en la app.
    }
carteira-ptax-do-dia = { $base ->
        [BRL] PTAX del día
       *[other] Tipo de cambio del día
    }
carteira-custos-corretagem = Costos (corretaje y tasas de bolsa)
carteira-impostos-retidos = Impuestos retenidos (IRRF)
carteira-nota-corretagem = Nota de corretaje
carteira-observacoes = Observaciones
carteira-day-trade = Day trade
carteira-day-trade-dica = Compra y venta el mismo día (cálculo del impuesto separado en la Fase 3).
carteira-lancar-caixa = Registrar en la caja de la corredora
carteira-lancar-caixa-dica = Crea el movimiento vinculado (aporte/retiro de inversión), que no cuenta como ingreso ni como gasto.
carteira-lancar-caixa-dica-edicao = Antes, revisa si el extracto de la corredora ya trajo ese débito o crédito: registrarlo de nuevo lo cuenta dos veces en el saldo.
carteira-bruto = Bruto
carteira-sai-caixa = Sale de la caja
carteira-entra-caixa = Entra en la caja
carteira-bruto-moeda = Bruto en { $moeda }
carteira-preco = Precio
carteira-salvar-alteracoes = Guardar cambios
carteira-registrar-compra = Registrar compra
carteira-registrar-venda = Registrar venta

## Registrar dividendo

carteira-prov-titulo = Registrar dividendo
carteira-prov-texto = Lo recibido entra en la caja de la corredora como ingreso (Rendimientos › Dividendos/JCP); lo anunciado queda solo en la agenda hasta el pago.
carteira-erro-escolha-conta = Elige la cuenta.
carteira-erro-data-pagamento = Ingresa la fecha de pago.
carteira-erro-bruto = Ingresa el monto bruto (o el monto por acción).
carteira-erro-imposto = Impuesto mayor que el bruto.
carteira-prov-registrado = Dividendo registrado
carteira-prov-anunciado-registrado = Dividendo anunciado registrado
carteira-prov-form = Dividendo
carteira-prov-situacao = Estado del dividendo
carteira-ativos-com-operacoes = Activos con operaciones.
carteira-tipo = Tipo
carteira-data-com = Fecha de corte
carteira-pagamento = Pago
carteira-valor-por-cota = Monto por acción ({ $moeda })
carteira-valor-por-cota-dica = Opcional: con la cantidad, calcula el bruto.
carteira-quantidade-base = Cantidad base
carteira-na-data-qtd = En la fecha: { $qtd }
carteira-cotas-data-com = Acciones en la fecha de corte.
carteira-valor-bruto = Monto bruto
carteira-bruto-calculado = Calculado: monto por acción × cantidad.
carteira-imposto-retido = Impuesto retenido
carteira-imposto-padrao = Por defecto: 17,5% en JCP (15% hasta 2025), 30% en dividendos de EE. UU., 0 en los demás.
carteira-cambio-pagamento = { $base ->
        [BRL] Vacío: PTAX de la fecha de pago.
       *[other] Vacío: tipo de cambio de la fecha de pago.
    }
carteira-lancar-credito = Registrar el crédito en la caja de la corredora
carteira-imposto = Impuesto
carteira-liquido = Neto

## Registrar evento corporativo

carteira-ev-titulo = Registrar evento corporativo
carteira-ev-texto = Los eventos de una fecha valen antes de las operaciones de esa misma fecha (fecha “ex”).
carteira-ev-fator-split = Mayor que 1: 2 = cada acción se convierte en dos (1:2).
carteira-ev-fator-grupamento = Entre 0 y 1: 0,1 = diez acciones se convierten en una (10:1).
carteira-ev-fator-bonificacao = Acciones nuevas por acción: 0,1 = 10% de bonificación.
carteira-ev-fator-incorporacao = Opcional: acciones del nuevo activo por acción antigua (por defecto 1).
carteira-ev-fator-ticker = Opcional: factor de canje (por defecto 1).
carteira-ev-erro-fator = Ingresa el factor.
carteira-ev-erro-split = El desdoblamiento necesita un factor mayor que 1.
carteira-ev-erro-grupamento = El agrupamiento necesita un factor entre 0 y 1.
carteira-ev-erro-bonificacao = La bonificación necesita un factor positivo.
carteira-ev-erro-data = Ingresa la fecha.
carteira-ev-erro-valor = Ingresa el monto total devuelto.
carteira-ev-erro-ativo-resultante = Elige el activo resultante.
carteira-ev-registrado = Evento corporativo registrado
carteira-ev-data = Fecha ex/efectiva
carteira-ev-fator = Factor
carteira-ev-custo-por-acao = Costo asignado por acción ({ $moeda })
carteira-ev-custo-dica = Informado en el aviso a los accionistas.
carteira-ev-valor-devolvido = Monto total devuelto
carteira-ev-valor-devolvido-dica = Reduce el costo de la posición.
carteira-ev-ativo-resultante = Activo resultante
carteira-ev-ativo-resultante-dica = La posición pasa a él en la fecha.
carteira-ev-cisao-texto = La escisión y la suscripción quedan registradas en el historial; registra aparte la operación resultante (la compra del nuevo activo o de las acciones suscritas).
carteira-ev-registrar = Registrar evento

## Registrar dividendos (cuadrícula y por confirmar)

carteira-grade-texto = Confirma los dividendos anunciados que recibiste o escribe (o pega) fecha, ticker y monto. Treasury encuentra el dividendo, la corredora, el tipo y el impuesto.
carteira-grade-como = Cómo registrar
carteira-grade-a-confirmar = Por confirmar ({ $n })
carteira-grade-digitar = Escribir
carteira-grade-ticker-linha = Ticker, fila { $n }
carteira-grade-ativos-linha = Activos para la fila { $n }
carteira-grade-em-carteira = en cartera
carteira-proventos-registrados =
    { $n ->
        [one] { $n } dividendo registrado
       *[other] { $n } dividendos registrados
    }
carteira-grade-nada-novo = Nada nuevo para registrar
carteira-grade-tabela = Dividendos a registrar
carteira-grade-ticker = Ticker
carteira-grade-valor-recebido = Monto recibido
carteira-grade-entendeu = Lo que entendió Treasury
carteira-grade-data-linha = Fecha de pago, fila { $n }
carteira-grade-valor-linha = Monto recibido, fila { $n }
carteira-grade-data-invalida = Fecha no válida.
carteira-grade-complete = Completa fecha, ticker y monto.
carteira-grade-remover-linha = Quitar fila { $n }
carteira-grade-ajuda = El monto recibido es el acreditado en la corredora (neto). Una fecha en blanco repite la de arriba; Enter pasa a la siguiente fila. Puedes pegar varias filas de una hoja de cálculo.
carteira-grade-prontas =
    { $n ->
        [one] { $n } lista
       *[other] { $n } listas
    }
carteira-grade-duplicadas =
    { $n ->
        [one] { $n } ya registrada
       *[other] { $n } ya registradas
    }
carteira-grade-problema = { $n } con problemas
carteira-grade-nenhuma-completa = Todavía no hay ninguna fila completa.
carteira-registrar-n-proventos =
    { $n ->
        [one] Registrar { $n } dividendo
       *[other] Registrar { $n } dividendos
    }
carteira-registrar = Registrar
carteira-bruto-ir = bruto { $bruto } · impuesto { $ir }
carteira-anunciado-curto = anunciado
carteira-conta-provento-de = Cuenta del dividendo de { $ativo }

carteira-conf-confere = coincide
carteira-conf-divergem = las fuentes difieren
carteira-conf-uma-fonte = una fuente
carteira-conf-fonte = { $fonte }: { $valor } por acción · fecha de corte { $data }
carteira-conf-fonte-pago = { $fonte }: { $valor } por acción · fecha de corte { $data } · pagado el { $pago }
carteira-conferencia = { $status }: { $fontes }
carteira-conferencia-usado = { $status }: { $fontes } · se usó { $fonte }
carteira-agente-pedido = Revisa estos dividendos por confirmar, en los que las fuentes difieren o solo una fuente tiene el dividendo (usa list_pending_income, check_dividends y get_fundamentus y, si puedes, la página de relación con inversionistas de la empresa o del fondo):
carteira-agente-fonte = { $fonte } { $valor }/acción, fecha de corte { $data }
carteira-agente-fonte-pago = { $fonte } { $valor }/acción, fecha de corte { $data }, pagado { $pago }
carteira-agente-item = - { $ativo } { $tipo } (fecha de corte { $data }): { $fontes }
carteira-agente-fecho = Para cada uno, di si de verdad es un problema y cuáles son el monto por acción y las fechas correctos. Cuando haya corrección, propón el registro con record_income_event (lo apruebo antes de guardar).
carteira-pend-vazio = Ningún dividendo anunciado por confirmar: todo lo que la agenda y B3 trajeron para tus activos ya está en la cartera. Usa "Escribir" para registrar uno que no apareció.
carteira-pend-texto = Anunciados con fecha de corte en la que tenías el activo y todavía no registrados. Desmarca lo que no recibiste y corrige el monto si el extracto de la corredora muestra otro.
carteira-pend-tabela = Dividendos por confirmar
carteira-pend-marcar-todos = Marcar todos
carteira-pend-qtd-cota = Cant. × por acción
carteira-pend-recebi = Recibí el dividendo de { $ativo } pagado el { $data }
carteira-pend-data-de = Fecha de pago de { $ativo }
carteira-pend-sem-data = sin fecha en la fuente: revísala
carteira-pend-detalhe = { $tipo } · { $conta } · fecha de corte { $data }
carteira-ir-valor = Impuesto { $valor }
carteira-pend-valor-de = Monto recibido de { $ativo }
carteira-pend-duvidosos =
    { $n ->
        [one] { $n } dividendo con fuentes que difieren o de una sola fuente.
       *[other] { $n } dividendos con fuentes que difieren o de una sola fuente.
    }
carteira-pend-pedir-agente = Pedir al agente que los revise
carteira-pend-marcados =
    { $total ->
        [one] { $n } de { $total } marcado
       *[other] { $n } de { $total } marcados
    }
carteira-confirmar-n-proventos =
    { $n ->
        [one] Confirmar { $n } dividendo
       *[other] Confirmar { $n } dividendos
    }
carteira-confirmar = Confirmar

## Series de índices

carteira-unidade-anualizada = % anual (anualizado, 252 días hábiles)

## Asignación por sector

carteira-setor-titulo = Por sector
carteira-setor-sem = Sin sector
carteira-setor-ativos =
    { $n ->
        [one] { $n } activo
       *[other] { $n } activos
    }
carteira-setor-vazio = Sin posiciones con valor de mercado.
carteira-setor-rodape = Sector del registro de cada activo (el tuyo o una sugerencia aprobada de las fuentes).
carteira-setor-completar =
    { $n ->
        [one] Completar el sector de { $n } activo en Salud de los activos
       *[other] Completar el sector de { $n } activos en Salud de los activos
    }

## Vendas (resultado nominal e real)

carteira-aba-vendas = Ventas
carteira-deflator-igpm-dica = Índice General de Precios del Mercado (FGV, vía BCB/SGS 189)
carteira-vendas-titulo = Resultado de las ventas
carteira-vendas-total-vendido = Total vendido
carteira-vendas-n =
    { $n ->
        [one] { $n } venta: { $ganhos } con ganancia, { $perdas } con pérdida
       *[other] { $n } ventas: { $ganhos } con ganancia, { $perdas } con pérdida
    }
carteira-vendas-nominal = Resultado de precio
carteira-vendas-nominal-dica = Venta menos el costo al precio medio, como en el impuesto
carteira-vendas-real = Resultado real ({ $indice })
carteira-vendas-real-dica = { $ganhos } con ganancia real, { $perdas } con pérdida real
carteira-vendas-faltam =
    { $n ->
        [one] { $n } venta sin { $indice } en el período quedó fuera
       *[other] { $n } ventas sin { $indice } en el período quedaron fuera
    }
carteira-vendas-efeito = Comido por el { $indice }
carteira-vendas-efeito-dica = Cuánto el costo corregido por el { $indice } supera el costo original
carteira-vendas-por-mes = Resultado por mes: nominal × real
carteira-vendas-tabela = Ventas
carteira-vendas-col-venda = Venta
carteira-vendas-col-custo = Costo
carteira-vendas-col-nominal = Nominal (precio)
carteira-vendas-col-real = Real ({ $indice })
carteira-vendas-col-indice = { $indice } en el plazo
carteira-vendas-col-prazo = Plazo medio
carteira-vendas-dias = { $n } d
carteira-vendas-sem-indice = Sin { $indice } para todo el período de esta venta
carteira-vendas-dica-custo = Costo corregido por el { $indice } de cada compra hasta la venta:
carteira-vendas-vazio-titulo = Ninguna venta en el período
carteira-vendas-vazio-texto = Las ventas registradas en Operaciones aparecen aquí, con la ganancia o la pérdida en dinero, antes y después de la inflación.
carteira-vendas-metodo = Costo al precio medio (renta fija: primero que entra, primero que sale). El resultado real lleva el costo de cada compra a la fecha de la venta por el { $indice }; el plazo medio se pondera por el costo. Valores en la moneda base, antes del impuesto.
carteira-vendas-dica-conta = { $custo } × (1 { $variacao }) = { $corrigido }
carteira-vendas-registrar = Registrar operación
carteira-vendas-menu-editar = Editar venta…
carteira-vendas-menu-compra = Registrar compra de este activo…
carteira-vendas-menu-venda = Registrar otra venta de este activo…
carteira-vendas-menu-evento = Registrar evento (desdoblamiento, bonificación…)…
carteira-vendas-menu-excluir = Eliminar venta…
carteira-vendas-erro-abrir = No se pudo abrir la venta
carteira-vendas-excluir-nota = Se recalculan el precio medio y el resultado de las ventas siguientes. Puedes deshacerlo enseguida.
carteira-vendas-col-proventos = Proventos
carteira-vendas-col-total = Con proventos
carteira-vendas-total = Con proventos
carteira-vendas-total-dica = Precio más { $proventos } de proventos recibidos por las cuotas vendidas; el impuesto es solo sobre el precio
carteira-vendas-dica-total = Precio { $preco } + proventos { $proventos } recibidos por las cuotas vendidas entre la compra y la venta
carteira-vendas-dica-total-real = Real con proventos ({ $indice }): { $valor }
carteira-conta-operacao = Cuenta (corredora o banco)
carteira-op-modo = Cómo informar
carteira-op-modo-total = Valor total
carteira-op-modo-unitario = Cantidad × precio
carteira-valor-aplicado = Valor aplicado
carteira-valor-bruto-resgate = Valor bruto del rescate
carteira-valor-bruto-resgate-dica = Antes del impuesto; el impuesto retenido va en Impuestos retenidos y el neto aparece en el resumen.
carteira-resgate-total = Rescatar toda la posición
carteira-erro-valor-total = Indica el valor.
carteira-erro-sem-posicao = No hay posición en esta cuenta en la fecha.
carteira-previdencia-saldo-titulo = { $n ->
        [one] Previsión sin saldo reciente
       *[other] { $n } previsiones sin saldo reciente
    }
carteira-previdencia-saldo-de = último saldo el { $data }
carteira-previdencia-sem-saldo = sin saldo informado (vale lo aportado)
carteira-previdencia-saldo-texto = Los planes sin cuota pública valen el saldo que informas. Abre el plan y usa Informar saldo con el valor que muestra la entidad.
carteira-saldo-na-data = Saldo en esta cuenta en la fecha: { $valor }
carteira-resgate-parcial = Rescate parcial: { $pct } del saldo.
carteira-aplicacao-de = Inversión de { $valor } en { $ativo } el { $data }
carteira-resgate-de = Rescate de { $valor } de { $ativo } el { $data }

## Calcular renta fija (comparar ofertas y vender antes del vencimiento; solo en Brasil)
carteira-rf-aba = Calcular renta fija
carteira-rf-cmd = Calcular renta fija
carteira-rf-cmd-busca = renta fija,comparar ofertas,cdb,lci,lca,tesoro,ahorro,equivalencia,valor de mercado,vender título
carteira-rf-comparar-titulo = Comparar ofertas
carteira-rf-comparar-texto = Escribe las ofertas que quieres ver lado a lado. Los resultados quedan en el orden en que las escribiste.
carteira-rf-valor = Monto invertido
carteira-rf-inicio = Fecha de la inversión
carteira-rf-inicio-dica = Vacío: hoy
carteira-rf-ofertas = Ofertas
carteira-rf-oferta-n = Oferta { $n }
carteira-rf-tipo = Tipo
carteira-rf-indice = Rendimiento
carteira-rf-taxa = Tasa (%)
carteira-rf-meses = Plazo (meses)
carteira-rf-resgate = O rescate el
carteira-rf-nome = Nombre (opcional)
carteira-rf-adicionar = Agregar oferta
carteira-rf-remover = Quitar oferta { $n }
carteira-rf-limite = Hasta { $max } ofertas.
carteira-rf-indice-cdi-pct = % del CDI
carteira-rf-indice-cdi-mais = CDI + %
carteira-rf-indice-selic-mais = Selic + %
carteira-rf-indice-pre = Tasa fija
carteira-rf-indice-ipca-mais = IPCA + %
carteira-rf-indice-poupanca = Cuenta de ahorro (poupança)
carteira-rf-premissas = Supuestos
carteira-rf-premissas-texto = Valen para todas las ofertas. Escribe otro valor para simular; bórralo para volver al de la bóveda.
carteira-rf-cdi = CDI (% anual)
carteira-rf-selic = Selic meta (% anual)
carteira-rf-ipca = IPCA esperado (% anual)
carteira-rf-tr = TR (% mensual)
carteira-rf-fonte-user = escrito por ti
carteira-rf-fonte-vault = de la bóveda
carteira-rf-fonte-vault-data = de la bóveda · { $data }
carteira-rf-fonte-projection = proyección (Focus)
carteira-rf-fonte-default = predeterminado (sin dato en la bóveda)
carteira-rf-incompleto = Completa las ofertas
carteira-rf-incompleto-texto = Indica el monto y, en cada oferta, la tasa y un plazo en meses o la fecha del rescate.
carteira-rf-tabela = Ofertas de renta fija
carteira-rf-col-oferta = Oferta
carteira-rf-col-prazo = Plazo
carteira-rf-dias = { $n ->
        [1] 1 día corrido
       *[other] { $n } días corridos
    }
carteira-rf-ate = hasta el { $data }
carteira-rf-col-bruto = Bruto
carteira-rf-col-iof = IOF
carteira-rf-col-ir = IR
carteira-rf-isento = exento
carteira-rf-col-custodia = Custodia
carteira-rf-col-liquido = Neto
carteira-rf-col-ano = Neto anual
carteira-rf-col-real = Sobre la inflación
carteira-rf-col-equivalencia = Equivalencia
carteira-rf-equiv-cdb = equivale al { $pct }% del CDI en un CDB
carteira-rf-equiv-lci = equivale al { $pct }% del CDI en una LCI/LCA
carteira-rf-col-fgc = FGC
carteira-rf-sim = Sí
carteira-rf-nao = No
carteira-rf-col-avisos = Avisos
carteira-rf-venda-titulo = Vender antes del vencimiento
carteira-rf-venda-texto = Cuánto vale un título de tasa fija o IPCA+ en la fecha de venta a la tasa de mercado, junto a llevarlo hasta el vencimiento.
carteira-rf-venda-carteira = Completar con un título de la cartera
carteira-rf-venda-carteira-escolha = Elige un título…
carteira-rf-venda-indice = Indexador
carteira-rf-venda-pre = Tasa fija
carteira-rf-venda-ipca = IPCA+
carteira-rf-venda-aplicacao = Fecha de la inversión
carteira-rf-venda-vencimento = Vencimiento
carteira-rf-venda-data = Fecha de la venta
carteira-rf-venda-contratada = Tasa contratada (% anual)
carteira-rf-venda-mercado = Tasa de mercado hoy (% anual)
carteira-rf-venda-taxa-real = Interés real, sobre el IPCA
carteira-rf-venda-isento = Exento de IR
carteira-rf-venda-ipca-premissa = IPCA supuesto (% anual)
carteira-rf-venda-incompleto = Completa el título
carteira-rf-venda-incompleto-texto = Indica el monto, las fechas (la venta entre la inversión y el vencimiento) y las dos tasas.
carteira-rf-venda-curva = Valor en la curva
carteira-rf-venda-preco = Precio de mercado
carteira-rf-venda-diferenca = Diferencia (valor de mercado)
carteira-rf-venda-diferenca-dica = Precio de mercado menos el valor en la curva: positiva cuando las tasas bajaron desde la inversión.
carteira-rf-venda-tabela = Vender hoy o llevar hasta el vencimiento
carteira-rf-venda-hoje = Vender hoy
carteira-rf-venda-levar = Llevar hasta el vencimiento
carteira-rf-venda-bruto = Bruto
carteira-rf-venda-ir = IR
carteira-rf-venda-iof = IOF
carteira-rf-venda-liquido = Neto
carteira-rf-venda-empate = Tasa de equilibrio
carteira-rf-venda-empate-pre = Para que el cambio quede en equilibrio, un título nuevo hasta el { $data } debe pagar { $taxa }% anual.
carteira-rf-venda-empate-ipca = Para que el cambio quede en equilibrio, un título nuevo hasta el { $data } debe pagar IPCA + { $taxa }% anual.
carteira-rf-venda-empate-nenhuma = Sin tasa de equilibrio para estos números.

## Puente del resultado (portfolio.bridge): del valor al inicio al valor al final, en dinero.
carteira-ponte-titulo = De dónde vino el resultado
carteira-ponte-explica = Primero en dinero: cuánto había al inicio, lo que cambiaron los precios, los dividendos y los costos, lo que aportó y retiró, y cuánto hay al final. Los porcentajes vienen después.
carteira-ponte-inicio = Valor al inicio
carteira-ponte-preco = Ganancia de precio
carteira-ponte-cambio = de la cual tipo de cambio: { $valor }
carteira-ponte-cambio-coluna = Tipo de cambio
carteira-ponte-proventos = Dividendos
carteira-ponte-taxas = Comisiones
carteira-ponte-impostos = Impuestos
carteira-ponte-aportes = Aportes
carteira-ponte-retiradas = Retiros
carteira-ponte-fim = Valor al final
carteira-ponte-resultado = Resultado del período
carteira-ponte-resultado-curto = Resultado
carteira-ponte-resultado-dica = Valor al final menos el valor al inicio, menos los aportes y más los retiros: lo que rindieron las inversiones en dinero.
carteira-ponte-inflacao = Pérdida por la inflación ({ $indice })
carteira-ponte-real = Resultado real
carteira-ponte-linhas = Líneas del puente del resultado
carteira-ponte-linha = Línea
carteira-ponte-valor = Valor
carteira-ponte-aria = Puente del resultado del { $de } al { $ate }: del valor al inicio al valor al final, línea por línea.
carteira-ponte-periodo = Del { $de } al { $ate }
carteira-ponte-fonte = Fuente: operaciones, cotizaciones, tipo de cambio y dividendos registrados · rendimiento pasado, no es recomendación de inversión
carteira-ponte-estimado = estimado
carteira-ponte-estimado-dica = Algún valor vino de una cotización desfasada, un tipo de cambio estimado o un índice aún no publicado.
carteira-ponte-por-ativo = Por activo ({ $n })
carteira-ponte-tabela-ativos = Puente del resultado por activo
carteira-ponte-calculando = Armando el puente del resultado
carteira-ponte-vazio = Sin valores en el período.

## Riesgo de la serie de rendimientos (semidesviación y caídas).
carteira-risco-semidesvio = Semidesviación (el tamaño típico de las caídas)
carteira-risco-queda-datas = { $pct } del { $pico } al { $fundo }
carteira-risco-voltou-em = { $queda }, volvió al máximo el { $data }
carteira-risco-sem-volta = { $queda }, aún no volvió al máximo
carteira-risco-tempo-abaixo = Mayor tiempo por debajo del máximo
carteira-risco-dias =
    { $n ->
        [one] { $n } día
       *[other] { $n } días
    }
carteira-risco-ainda-abaixo = aún por debajo
carteira-risco-metodo-mensal = Rendimiento mensual (Modified Dietz)
carteira-risco-metodo-diario = Rendimiento diario
carteira-risco-metodo-diario-fonte = Método: TWR diario encadenado, neutraliza aportes y retiros
carteira-risco-como = Cómo calculamos: { $metodo }. La semidesviación, la peor caída y el tiempo por debajo del máximo usan la misma serie de rendimientos. El rendimiento pasado no garantiza resultados futuros.

## Período estándar y comparación (docs/portfolio-compare.md)

carteira-periodo-de = Período: { $aba }
carteira-periodo-ponte = Puente del resultado
carteira-comparar-com = Comparar con
carteira-comparar-tipo = Tipo de serie para comparar
carteira-comparar-indice = Índice
carteira-comparar-ativo = Activo
carteira-comparar-cdi-mais = { $indice } + tasa
carteira-comparar-escolha = Elige la serie
carteira-comparar-taxa = Tasa anual, en %
carteira-comparar-ao-ano = % al año
carteira-comparar-adicionar = Añadir
carteira-comparar-escolhidas = Series comparadas
carteira-comparar-remover = Quitar { $serie } de la comparación
carteira-comparar-limite = Hasta { $n } series a la vez.
carteira-comparar-aviso = Rentabilidad pasada lado a lado, no es recomendación ni promesa de rendimiento.
carteira-mapa-titulo = Rentabilidad de cada mes, por año
carteira-mapa-fonte = Rentabilidad mensual por Modified Dietz; el total compone los meses del período en el año y la última columna es el { $indice } en los mismos meses.
carteira-mapa-ano = Año
carteira-mapa-total = Total
carteira-mapa-aria = Mapa de calor de las rentabilidades mensuales
carteira-contrib-titulo = Mayores contribuciones
carteira-contrib-calculando = Calculando las contribuciones…
carteira-contrib-somaram = Más sumaron
carteira-contrib-tiraram = Más restaron
carteira-contrib-fonte = Resultado de cada activo de { $de } a { $ate } (revalorización, ventas y dividendos), en dinero y en puntos de la rentabilidad de la cartera. Rentabilidad pasada.
carteira-contrib-vazio = Ningún activo tuvo resultado en el período.
carteira-contrib-tabela = Contribución de cada activo
carteira-contrib-ativo = Activo
carteira-contrib-resultado = Resultado
carteira-contrib-proventos = Dividendos
carteira-contrib-pontos = Puntos
carteira-prov-mes-titulo = Dividendos por mes, año contra año
carteira-prov-mes-calculando = Sumando los dividendos…
carteira-prov-mes-fonte = Dividendos recibidos, netos, en { $moeda }, en los tres últimos años.
carteira-prov-mes-aria = Dividendos de cada mes en los tres últimos años
carteira-prov-mes-tabela = Dividendos por mes y año

## Método de costo y retorno anualizado en el libro de ventas

carteira-vendas-col-medio = Precio promedio (IR)
carteira-vendas-col-irr = Retorno anual
carteira-vendas-irr-dica = Retorno anualizado de la venta: la tasa anual que iguala el costo de cada compra que salió, los dividendos recibidos y el valor de la venta. Desempeño pasado, no una promesa.
carteira-vendas-medio = Por precio promedio (IR)
carteira-vendas-medio-dica = El mismo resultado por precio promedio, la base del impuesto en Brasil
carteira-vendas-nominal-dica-fifo = Venta menos el costo de las compras más antiguas (primero en entrar, primero en salir)
carteira-vendas-nominal-dica-pool = Venta menos el costo por la regla del Reino Unido (mismo día, 30 días, pool)
carteira-vendas-metodo-fifo = Costo por primero en entrar, primero en salir (renta fija: los lotes); el resultado por precio promedio queda en la columna de al lado. El resultado real lleva el costo de cada compra a la fecha de la venta por el { $indice }; el plazo promedio se pondera por el costo. Valores en la moneda base, antes del impuesto.
carteira-vendas-metodo-pool = Costo por la regla del Reino Unido: las compras del mismo día se emparejan primero, después las de los 30 días siguientes, y el resto sale del pool por el costo promedio (renta fija: los lotes); el resultado por precio promedio queda en la columna de al lado. El resultado real lleva el costo de cada compra a la fecha de la venta por el { $indice }. Valores en la moneda base, antes del impuesto.
carteira-vendas-metodo-pais = El método sigue el país de la familia; puedes cambiarlo en Ajustes › Familia.
carteira-vendas-metodo-escolhido = Método elegido en Ajustes › Familia.

## Aportes programados e carteira por objetivo

carteira-objetivo-titulo = Por objetivo
carteira-objetivo-vazio = Ninguna posición vinculada a una meta todavía. Vincula posiciones a una meta para ver la cartera por objetivo.
carteira-objetivo-ir-metas = Ir a Metas
carteira-objetivo-alvo =
    { $n ->
        [one] { $n } posición vinculada · { $pct } del objetivo
       *[other] { $n } posiciónes vinculadas · { $pct } del objetivo
    }
carteira-objetivo-sem = Sin objetivo
carteira-objetivo-nota = Por los pesos de las posiciones vinculadas a cada meta (fracciones y fondo común).
carteira-plano-titulo = Aportes programados
carteira-plano-novo = Nuevo aporte programado
carteira-plano-editar = Editar aporte programado
carteira-plano-dialogo-texto = Cada mes, en el día elegido, el aporte aparece para que lo confirmes con el precio del día. Nada se registra sin tu confirmación.
carteira-plano-nome = Nombre
carteira-plano-ativo = Activo
carteira-plano-conta = Cuenta de la corredora
carteira-plano-valor = Valor de cada aporte
carteira-plano-dia = Día del mes
carteira-plano-dia-dica = De 1 a 28.
carteira-plano-inicio = Inicio
carteira-plano-fim = Fin
carteira-plano-fim-dica = Vacío: sin fecha de fin.
carteira-plano-meta = Meta
carteira-plano-meta-dica = Para qué meta va el aporte (solo informativo).
carteira-plano-sem-meta = Ninguna
carteira-plano-ativo-ligado = Activo (desmarca para pausar)
carteira-plano-salvo = Aporte programado guardado
carteira-plano-a-confirmar =
    { $n ->
        [one] { $n } aporte programado por confirmar. Ajusta la cantidad y el precio si hace falta.
       *[other] { $n } aportes programados por confirmar. Ajusta la cantidad y el precio si hace falta.
    }
carteira-plano-a-confirmar-lista = Aportes por confirmar
carteira-plano-preco-de = precio del { $data }
carteira-plano-sem-preco = sin cotización: indica el precio
carteira-plano-quantidade-de = Cantidad de { $ativo } el { $data }
carteira-plano-preco-rotulo = Precio de { $ativo } el { $data }
carteira-plano-confirmar = Confirmar
carteira-plano-pular = Omitir
carteira-plano-confirmado = Compra de { $ativo } registrada
carteira-plano-pulado = Aporte de { $ativo } omitido
carteira-plano-erro = No se pudo registrar el aporte
carteira-plano-vazio = Ningún aporte programado. Crea uno para no tener que registrar la misma compra cada mes.
carteira-plano-lista = Aportes programados
carteira-plano-pausado = pausado
carteira-plano-resumo = { $ativo } cada día { $dia } · { $conta }
carteira-plano-proximo = próximo el { $data }
carteira-plano-excluir-nome = Eliminar { $nome }
carteira-plano-excluido = Aporte programado eliminado
carteira-plano-aviso =
    { $n ->
        [one] { $n } aporte programado esperando tu confirmación.
       *[other] { $n } aportes programados esperando tu confirmación.
    }
carteira-plano-aviso-ver = Ver aportes

## Posición reciente (menos de 30 días: solo el retorno del período)
carteira-recente-pill = posición reciente
carteira-recente-comprada =
    { $dias ->
        [0] Comprada hoy
        [one] Comprada hace 1 día
       *[other] Comprada hace { $dias } días
    }
carteira-recente-dica = { $comprada }. Retorno en el período: { $retorno }. El anualizado aparece a partir de los 30 días.
carteira-recente-nota = { $comprada }. El anualizado aparece a partir de los 30 días.
carteira-recente-no-periodo = en el período
carteira-recente-anualizado = { $retorno } anualizado sobre { $dias } días
carteira-recente-anualizado-real = real { $retorno }
carteira-recente-titulo = Retorno en el período

## Concentración (límites de la familia; lectura de los datos, sin recomendación)
carteira-conc-titulo = Concentración
carteira-conc-ajustar = Ajustar límites
carteira-conc-botao = Límites de concentración
carteira-conc-lista = Alertas de concentración
carteira-conc-tipo-ativo = Activo
carteira-conc-tipo-grupo = Grupo
carteira-conc-tipo-classe = Clase
carteira-conc-limite = de la cartera · límite { $limite }
carteira-conc-acima = { $valor } por encima del límite
carteira-conc-sev-warning = Por encima del límite
carteira-conc-sev-critical = Muy por encima del límite
carteira-conc-rodape = Pesos según el valor de hoy, comparados con los límites que elegiste. Es una lectura de tus datos, no una recomendación de compra o venta.
carteira-conc-pill-dica = Peso por encima del límite de { $limite } para este activo
carteira-conc-dialogo-titulo = Límites de concentración
carteira-conc-dialogo-texto = Treasury te avisa cuando un activo o un grupo de activos supera el límite. Si no los ajustas, valen 20% por activo y 30% por grupo; la clase usa la meta de asignación con la tolerancia. El Tesoro Directo y la renta fija con FGC (hasta R$ 250 mil por activo) quedan fuera del límite por defecto.
carteira-conc-padroes = Valores predeterminados
carteira-conc-padrao-ativo = Límite por activo (%)
carteira-conc-padrao-grupo = Límite por grupo (%)
carteira-conc-por-ativo = Por activo
carteira-conc-por-ativo-texto = Deja el límite en blanco para usar el predeterminado.
carteira-conc-col-limite = Límite
carteira-conc-limite-de = Límite de { $ativo } (%)
carteira-conc-isento = sin límite
carteira-conc-isento-dica = El Tesoro Directo y la renta fija con FGC (hasta R$ 250 mil por activo) quedan fuera del límite por defecto. Escribe un límite para aplicarlo a este activo.
carteira-conc-limite-invalido = Usa límites mayores que 0% y de hasta 100%.
carteira-conc-salvar = Guardar límites
carteira-conc-salvos = Límites guardados
carteira-conc-sem-ativos = Todavía no hay posiciones en la cartera.
carteira-rebal-conc-texto = Sin metas por clase, la división de abajo sigue los límites de concentración: cada activo recibe una parte del aporte proporcional al espacio que aún tiene hasta el límite. No se vende nada.
carteira-rebal-conc-tabela = Aporte según los límites de concentración
carteira-rebal-conc-espaco = Espacio hasta el límite
carteira-rebal-conc-acima = por encima del límite, sin aporte

## Etiquetas de activos (grupos de concentración)
carteira-etiqueta-titulo = Etiquetas
carteira-etiqueta-texto = Las etiquetas agrupan activos (por sector, país o emisor, por ejemplo). Marca una como grupo de concentración para sumar el peso de sus activos.
carteira-etiqueta-de = Etiquetas de { $ativo }
carteira-etiqueta-tirar = Quitar la etiqueta { $etiqueta } de { $ativo }
carteira-etiqueta-exemplo = sector, país…
carteira-etiqueta-grupo = Grupo de concentración
carteira-etiqueta-grupo-de = { $etiqueta } es grupo de concentración
carteira-etiqueta-limite-de = Límite del grupo { $etiqueta } (%)
carteira-etiqueta-sem = Todavía no hay etiquetas. Escribe una en la fila de un activo.
carteira-etiqueta-sem-ativos = sin activos en la cartera
carteira-etiqueta-outros =
    { $n ->
        [one] y 1 activo fuera de la cartera
       *[other] y { $n } activos fuera de la cartera
    }
