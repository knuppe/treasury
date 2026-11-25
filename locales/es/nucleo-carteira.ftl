# Cartera y bienes (núcleo, `portfolio` y `physical`): errores, motivos del historial,
# descripciones de los movimientos que crea la app, avisos y notas de las posiciones,
# dividendos, costo de las comisiones, riesgo, rebalanceo, el extracto del activo, la carga en
# lote del agente, los dividendos escritos a las apuradas y la fórmula del valor de los bienes
# (el "Explicar" del patrimonio). Errores en minúsculas, como línea de log (la interfaz pone la
# mayúscula inicial). Los términos oficiales y de mercado (JCP, FII, CVM, B3, FIPE, IPCA, IGP-M,
# INCC, BCB SGS, CDI, TWR, come-cotas) quedan como están.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Campos (entran en "<campo> inválido: <valor>")

nucleo-carteira-erro-campo-invalido = { $campo } inválido: { $valor }
nucleo-carteira-campo-quantidade = cantidad
nucleo-carteira-campo-preco = precio
nucleo-carteira-campo-cambio = tipo de cambio
nucleo-carteira-campo-fator = factor
nucleo-carteira-campo-custo = costo
nucleo-carteira-campo-alvo = objetivo
nucleo-carteira-campo-tolerancia = tolerancia
nucleo-carteira-campo-preco-alvo = precio objetivo
nucleo-carteira-campo-alerta = alerta
nucleo-carteira-campo-valor-cota = monto por acción

## Tipos de dividendo

nucleo-carteira-provento-dividendos = Dividendos
nucleo-carteira-provento-jcp = JCP
nucleo-carteira-provento-fii = Rendimientos de FII
nucleo-carteira-provento-cupom = Cupón
nucleo-carteira-provento-juros = Intereses
nucleo-carteira-provento-aluguel = Préstamo de acciones
nucleo-carteira-provento-amortizacao = Amortización
nucleo-carteira-provento-outros = Otros pagos
# Descripción del movimiento de un dividendo ("Dividendos PETR4").
nucleo-carteira-provento-lancamento = { $tipo } { $ativo }

## Operaciones, eventos corporativos, dividendos, objetivos y watchlist (`records`)

nucleo-carteira-operacao = operación { $id }
# Descripción del movimiento de una operación ("Compra 10 PETR4 @ R$ 38,50").
nucleo-carteira-operacao-compra = Compra { $quantidade } { $ativo } @ { $preco }
nucleo-carteira-operacao-venda = Venta { $quantidade } { $ativo } @ { $preco }
nucleo-carteira-erro-moedas = la cuenta está en { $conta } y el activo en { $ativo }: indica el tipo de cambio o desactiva el movimiento de caja
nucleo-carteira-erro-quantidade = la cantidad debe ser positiva
nucleo-carteira-erro-preco = el precio no puede ser negativo
nucleo-carteira-erro-custos = los costos e impuestos no pueden ser negativos
nucleo-carteira-erro-cambio = el tipo de cambio debe ser positivo
nucleo-carteira-erro-venda-maior = venta de { $quantidade } mayor que la posición de { $posicao } en esta cuenta en la fecha
nucleo-carteira-erro-ajuste-com-caixa = un ajuste de posición no mueve la caja: quita el movimiento de caja de la operación (cash_movement falso)
nucleo-carteira-erro-ajuste-importada = esta operación vino de la corredora con su propio identificador: es una operación real, no un ajuste de posición
nucleo-carteira-erro-desdobramento = un desdoblamiento necesita un factor mayor que 1 (p. ej.: 2 para 1→2)
nucleo-carteira-erro-grupamento = una agrupación necesita un factor entre 0 y 1 (p. ej.: 0,1 para 10→1)
nucleo-carteira-erro-bonificacao = una bonificación necesita el factor (acciones nuevas por acción, p. ej.: 0,1 = 10%)
nucleo-carteira-erro-devolucao = una devolución de capital necesita el monto total
nucleo-carteira-erro-ativo-resultante = indica el activo resultante
nucleo-carteira-erro-bruto = el monto bruto del dividendo debe ser positivo
nucleo-carteira-erro-imposto-retido = impuesto retenido inválido
nucleo-carteira-erro-alvos-escopo = todos los objetivos deben tener el mismo alcance
nucleo-carteira-erro-alvo = el objetivo debe estar entre 0% y 100%
nucleo-carteira-erro-alvos-soma = la suma de los objetivos por clase pasa del 100%
nucleo-carteira-erro-alvos-soma-classe = la suma de los objetivos de los activos de { $classe } pasa del 100% de la clase
nucleo-carteira-watchlist-acima = cotización por encima de { $valor }
nucleo-carteira-watchlist-abaixo = cotización por debajo de { $valor }

## Comandos (`portfolio.*`): motivos del historial y respuestas

nucleo-carteira-erro-nada-selecionado = ningún registro seleccionado
nucleo-carteira-nada-novo = Nada nuevo: todo ya estaba registrado o se omitió (mira el informe).
nucleo-carteira-nenhuma-linha-pronta = Ninguna línea lista: todas ya registradas o con error (mira el informe).
nucleo-carteira-motivo-operacao-registrada = Operación registrada
nucleo-carteira-motivo-operacao-alterada = Operación modificada
nucleo-carteira-motivo-operacoes-removidas = { $n ->
        [one] Operación eliminada
       *[other] { $n } operaciones eliminadas
    }
nucleo-carteira-motivo-evento-registrado = Evento corporativo registrado
nucleo-carteira-motivo-eventos-removidos = { $n ->
        [one] Evento corporativo eliminado
       *[other] { $n } eventos corporativos eliminados
    }
nucleo-carteira-motivo-provento-registrado = Dividendo registrado
nucleo-carteira-erro-sem-proventos = ningún dividendo para registrar
nucleo-carteira-erro-proventos-demais = como máximo 500 dividendos a la vez
nucleo-carteira-motivo-proventos-confirmados = { $n ->
        [one] { $n } dividendo confirmado
       *[other] { $n } dividendos confirmados
    }
nucleo-carteira-motivo-proventos-removidos = { $n ->
        [one] Dividendo eliminado
       *[other] { $n } dividendos eliminados
    }
nucleo-carteira-motivo-come-cotas = Regla del come-cotas
nucleo-carteira-motivo-alvos = Asignación objetivo definida
nucleo-carteira-motivo-watchlist = Activo en la watchlist
nucleo-carteira-motivo-watchlist-removido = Activo quitado de la watchlist

## Carga en lote del agente (`bulk`)

nucleo-carteira-erro-lote-fonte = di de dónde vinieron los datos (`source`)
nucleo-carteira-erro-lote-vazio = nada para registrar: indica activos, operaciones, posiciones o dividendos
nucleo-carteira-erro-lote-itens = como máximo { $maximo } elementos por llamada; divídelo en partes
nucleo-carteira-erro-lote-conta = las operaciones, posiciones y dividendos necesitan la cuenta (`account_id` o `account`)
nucleo-carteira-erro-conta-cartao = la cuenta de inversiones no puede ser una tarjeta de crédito
# Motivo del historial: "Inversiones en lote (2 activos, 5 operaciones): extracto de XP".
nucleo-carteira-motivo-lote = Inversiones en lote ({ $itens }): { $fonte }
nucleo-carteira-lote-ativos = { $n ->
        [one] { $n } activo
       *[other] { $n } activos
    }
nucleo-carteira-lote-operacoes = { $n ->
        [one] { $n } operación
       *[other] { $n } operaciones
    }
nucleo-carteira-lote-posicoes = { $n ->
        [one] { $n } posición
       *[other] { $n } posiciones
    }
nucleo-carteira-lote-proventos = { $n ->
        [one] { $n } dividendo
       *[other] { $n } dividendos
    }
nucleo-carteira-lote-conta = cuenta
# La cuenta del informe del lote.
nucleo-carteira-lote-conta-existente-id = existente { $id }
nucleo-carteira-lote-conta-existente = existente: { $nome }
nucleo-carteira-lote-conta-nova = nueva: { $nome }
# Elementos omitidos del informe ("operación 3: ya registrada").
nucleo-carteira-lote-ativo-erro = activo { $n } ({ $ativo }): { $erro }
nucleo-carteira-lote-operacao-sem-ativo = operación { $n }: activo { $ativo } no registrado (inclúyelo en `assets`)
nucleo-carteira-lote-operacao-repetida = operación { $n }: ya registrada
nucleo-carteira-lote-operacao-erro = operación { $n }: { $erro }
nucleo-carteira-lote-posicao-sem-ativo = posición { $n }: activo { $ativo } no registrado (inclúyelo en `assets`)
nucleo-carteira-lote-posicao-quantidade = posición { $n }: la cantidad debe ser positiva
nucleo-carteira-lote-posicao-erro = posición { $n }: { $erro }
nucleo-carteira-lote-posicao-confere = posición { $n } ({ $ativo }): ya coincide
nucleo-carteira-lote-posicao-maior =
    { $registrado ->
        [one] posición { $n } ({ $ativo }): ya hay { $registrado } registrado, más que { $informado }; registra la venta con `record_trade` (`side: sell`; `cash_movement: false` si el dinero ya entró en la cuenta)
       *[other] posición { $n } ({ $ativo }): ya hay { $registrado } registrados, más que { $informado }; registra la venta con `record_trade` (`side: sell`; `cash_movement: false` si el dinero ya entró en la cuenta)
    }
nucleo-carteira-lote-posicao-repetida = posición { $n }: ya registrada
nucleo-carteira-lote-provento-sem-ativo = dividendo { $n }: activo { $ativo } no registrado (inclúyelo en `assets`)
nucleo-carteira-lote-provento-repetido = dividendo { $n }: ya registrado
nucleo-carteira-lote-provento-erro = dividendo { $n }: { $erro }
# Nota que se guarda en las operaciones del lote. El comienzo de cada una (antes del primer
# argumento) marca la operación como del agente en el extracto del activo, en cualquier idioma.
nucleo-carteira-nota-lote = Lote del agente: { $fonte }
nucleo-carteira-nota-posicao = Posición informada el { $data } ({ $fonte }): ajuste de { $diferenca } al precio promedio

## Dividendos escritos a las apuradas (`quick_income`)

nucleo-carteira-rapido-erro-ticker = indica el ticker
nucleo-carteira-rapido-erro-valor = el monto debe ser positivo
nucleo-carteira-rapido-erro-ativo = { $ticker } no está registrado en la cartera
nucleo-carteira-rapido-erro-conta = cuenta { $conta } no encontrada
nucleo-carteira-rapido-erro-sem-operacoes = ninguna operación de { $ticker } registrada: elige la cuenta
nucleo-carteira-rapido-erro-sem-posicao = sin posición de { $ticker } el { $data }: elige la cuenta
nucleo-carteira-rapido-ja-registrado = ya registrado
nucleo-carteira-rapido-repetido = repite una línea de arriba
nucleo-carteira-rapido-erro-sem-linhas = ninguna línea de dividendo
nucleo-carteira-rapido-erro-linhas = como máximo { $maximo } líneas a la vez (recibidas { $recebidas })
nucleo-carteira-rapido-nota-sem-posicao = sin posición de { $ticker } en la fecha
nucleo-carteira-rapido-nota-maior-posicao = activo en más de una cuenta: se eligió la de mayor posición
nucleo-carteira-rapido-nota-conta = cuenta por la posición que explica el monto: { $conta }
# "Dividendos de 0,5 por acción (B3, fecha de corte 10/10/2026, pagado el 20/10/2026)".
nucleo-carteira-rapido-nota-anunciado = { $tipo } de { $valor } por acción ({ $fonte }, pagado el { $pagamento })
nucleo-carteira-rapido-nota-anunciado-com = { $tipo } de { $valor } por acción ({ $fonte }, fecha de corte { $datacom }, pagado el { $pagamento })
nucleo-carteira-rapido-nota-confere = coincide: { $quantidade } × { $valor } = { $total }
nucleo-carteira-rapido-nota-diferente = monto distinto del esperado ({ $esperado } bruto para { $quantidade } acciones): revísalo
nucleo-carteira-rapido-nota-bruto = monto escrito tratado como bruto (antes del IR)
nucleo-carteira-rapido-nota-sem-anuncio = ningún dividendo anunciado cerca del { $data }: tipo según la clase ({ $tipo })
nucleo-carteira-rapido-nota-ir = IR retenido presunto: { $imposto } (bruto { $bruto })
nucleo-carteira-rapido-nota-futuro = pago futuro: queda como anunciado, sin caja
# Motivo del historial: "3 dividendos registrados: PETR4, VALE3, ITUB4".
nucleo-carteira-rapido-motivo = { $n ->
        [one] { $n } dividendo registrado: { $lista }
       *[other] { $n } dividendos registrados: { $lista }
    }
nucleo-carteira-rapido-e-mais = { $lista } y { $n } más

## Dividendos anunciados a revisar (`pending`)

nucleo-carteira-pendente-data-agenda = fecha de pago de la agenda (la fuente oficial todavía no la informó)
nucleo-carteira-pendente-valor-divergente = { $fonte } decía { $valor } por acción; se usó el valor oficial ({ $b3 })
nucleo-carteira-pendente-data-divergente = { $fonte } decía pago el { $data }; se usó la fecha oficial ({ $b3 })
nucleo-carteira-pendente-tipos-divergentes = las fuentes dan tipos distintos ({ $lista }); se usó el de { $fonte }
nucleo-carteira-pendente-fontes-discordam = el dividendo no está en las fuentes oficiales y las fuentes no coinciden ({ $lista }); se usó el de { $fonte }: revísalo en el extracto
nucleo-carteira-pendente-so-uma-fonte = solo en { $fonte }: no hay aviso oficial de este dividendo (todavía no descargado o fuera de la lista oficial)

## Posiciones, rendimiento y rebalanceo (`engine`)

nucleo-carteira-erro-cadeia-ticker = cadena de cambios de ticker demasiado larga (o circular)
nucleo-carteira-aviso-sem-cambio = sin tipo de cambio { $moeda }/{ $base } el { $data }
nucleo-carteira-aviso-venda-maior = venta mayor que la posición el { $data }
nucleo-carteira-aviso-cisao = escisión/suscripción: registra la operación resultante manualmente
nucleo-carteira-aviso-sem-cotacao = sin cotización: valuado al costo
nucleo-carteira-aviso-aportes-sem-cota = { $valor } en aportes de previsión aún sin cuota: valorados por el importe aportado
nucleo-carteira-aviso-cotacao-parada = sin cotización nueva desde el { $data } y sin negocios: el activo puede haber dejado de cotizar o haber sido absorbido; revisa antes de registrar
nucleo-carteira-erro-periodo-longo = período demasiado largo
# Nombre de la serie de comparación fuera de Brasil, cuando el país no tiene un nombre para ella.
nucleo-carteira-juro-referencia = Tasa de referencia
nucleo-carteira-rebalanceamento-aviso = Análisis basado en tus datos y en los objetivos que definiste; no es una recomendación de inversión.

## Come-cotas (`come_cotas`, el impuesto semestral de los fondos brasileños)

nucleo-carteira-come-cotas-so-fundos = solo los fondos abiertos tienen come-cotas
nucleo-carteira-come-cotas-escolhida = regla elegida por ti
nucleo-carteira-come-cotas-cvm-isento = clasificación de la CVM: { $classe } (sin come-cotas)
nucleo-carteira-come-cotas-cvm-longo = clasificación de la CVM: { $classe }; largo plazo (15%) hasta que indiques corto plazo
nucleo-carteira-come-cotas-sem-classificacao = fondo sin clasificación descargada de la CVM: largo plazo (15%); ajústalo si es de acciones o de corto plazo
nucleo-carteira-come-cotas-sem-cota = sin valor de cuota el { $data }: no se aplicó el come-cotas de esa fecha
nucleo-carteira-come-cotas-erro-valor = valor inválido en el come-cotas: { $valor }

## Costo de las comisiones (`fee_cost`)

nucleo-carteira-custo-erro-parametros = retorno real entre −50% y 50%, comisión alternativa entre 0 y 20% y exceso entre 0 y 50%
nucleo-carteira-custo-erro-horizontes = horizontes de 1 a 60 años
nucleo-carteira-custo-fonte-previdencia = registro del plan de previsión
nucleo-carteira-custo-fonte-cvm = extracto del fondo en la CVM
nucleo-carteira-custo-fonte-cvm-data = extracto del fondo en la CVM ({ $data })
nucleo-carteira-custo-nota-retorno = Valores reales de hoy, con un retorno de { $retorno } anual por encima de la inflación antes de las comisiones.
nucleo-carteira-custo-nota-alternativa = Escenario alternativo: comisión de administración de { $taxa } anual (o la de hoy, si ya es menor), sin comisión de rendimiento ni de carga.
nucleo-carteira-custo-nota-performance = Comisión de rendimiento sin costo proyectado: solo pesa si el fondo supera el índice de referencia (indica el exceso esperado).

## Riesgo de la cartera (`risk`)

nucleo-carteira-risco-erro-meses = analiza de 12 a 120 meses
nucleo-carteira-risco-nota-retornos = Retornos mensuales de { $de } a { $ate } (TWR, con dividendos, en { $moeda }). La cartera y las clases usan los pesos de hoy; en un mes en que un activo no existía, su peso pasa a los demás.
nucleo-carteira-risco-nota-pouco-historico = Menos de { $minimo } meses de historial (sin métricas propias ni correlación): { $ativos }.
nucleo-carteira-risco-nota-sem-cdi = Sin la serie del CDI en el período: el índice de Sharpe queda fuera (Ajustes › Índices).
nucleo-carteira-risco-nota-estimados = Hay precios estimados o faltantes en el período: los números son aproximados.

## Extracto del activo (`statement`)

nucleo-carteira-extrato-desdobramento = Desdoblamiento { $proporcao }
nucleo-carteira-extrato-grupamento = Agrupación { $proporcao }
nucleo-carteira-extrato-bonificacao-pct = Bonificación del { $pct }%
nucleo-carteira-extrato-bonificacao = Bonificación
nucleo-carteira-extrato-subscricao = Suscripción
nucleo-carteira-extrato-incorporacao = Absorción
nucleo-carteira-extrato-troca-ticker = Cambio de ticker
nucleo-carteira-extrato-cisao = Escisión
nucleo-carteira-extrato-devolucao = Devolución de capital
nucleo-carteira-extrato-herdada = Posición heredada (cambio de ticker o absorción)
nucleo-carteira-extrato-entrada-sem-custo = Entrada sin costo (desdoblamiento o bonificación registrado como compra)
nucleo-carteira-extrato-compra = Compra
nucleo-carteira-extrato-venda = Venta
nucleo-carteira-extrato-ajuste = Ajuste de posición
nucleo-carteira-extrato-come-cotas =
    { $cotas ->
        [one] Come-cotas: { $cotas } cuota rescatada para el IR
       *[other] Come-cotas: { $cotas } cuotas rescatadas para el IR
    }
# De dónde vino la línea del extracto.
nucleo-carteira-extrato-origem-nota = nota { $numero }
nucleo-carteira-extrato-origem-agente = agente
nucleo-carteira-extrato-origem-importacao = importación
nucleo-carteira-extrato-origem-manual = manual
nucleo-carteira-extrato-origem-sistema = sistema
nucleo-carteira-extrato-origem-yahoo = descarga de Yahoo
nucleo-carteira-extrato-origem-coleta = descarga de cotizaciones
nucleo-carteira-extrato-origem-b3 = importación B3
nucleo-carteira-extrato-aviso-desdobramento-duplo = posible desdoblamiento contado dos veces: hay una entrada a costo cero y un evento corporativo con pocos días de diferencia; elimina uno de los dos
nucleo-carteira-extrato-aviso-bonificacao-dupla = posible bonificación contada dos veces: hay un evento corporativo y una compra con la misma cantidad de acciones nuevas con pocos días de diferencia; elimina uno de los dos

## Bienes (`physical`): registro, mejoras, tasaciones y financiamiento

nucleo-carteira-bem = bien { $id }
nucleo-carteira-bem-benfeitoria = mejora { $id }
nucleo-carteira-bem-avaliacao = tasación { $id }
nucleo-carteira-bem-ligacao = vínculo del préstamo con el bien
nucleo-carteira-bem-campo-taxa-depreciacao = tasa de depreciación
nucleo-carteira-bem-campo-valor-residual = valor residual
nucleo-carteira-bem-campo-taxa = tasa
nucleo-carteira-bem-campo-residual = residual
nucleo-carteira-bem-erro-fora-intervalo = { $campo } fuera del rango
nucleo-carteira-bem-erro-nome = ponle un nombre al bien
nucleo-carteira-bem-erro-preco = el precio de compra no puede ser negativo
nucleo-carteira-bem-erro-vida-util = vida útil entre 1 y 100 años
nucleo-carteira-bem-erro-fipe-dados = para usar la tabla FIPE indica tipo, marca, modelo y año del vehículo
nucleo-carteira-bem-erro-fipe-ref = indica tipo, marca, modelo y año del vehículo en la tabla FIPE
nucleo-carteira-bem-erro-venda-antes = la venta no puede ser anterior a la compra
nucleo-carteira-bem-erro-venda-negativa = los montos de la venta no pueden ser negativos
nucleo-carteira-bem-erro-custos = los costos de adquisición no pueden ser negativos
nucleo-carteira-bem-erro-benfeitoria = describe la mejora o la obra
nucleo-carteira-bem-erro-valor-obra = el monto de la obra debe ser positivo
nucleo-carteira-bem-erro-valor-projeto = indica el monto (estimado) de la obra o vincula movimientos al proyecto
nucleo-carteira-bem-erro-avaliacao = el monto de la tasación no puede ser negativo
nucleo-carteira-bem-erro-contrato-outro = este préstamo ya financia otro bien ({ $bem }); desvincúlalo de allí primero
nucleo-carteira-bem-motivo-cadastrado = Bien registrado: { $nome }
nucleo-carteira-bem-motivo-alterado = Bien modificado
nucleo-carteira-bem-motivo-removido = Bien eliminado
nucleo-carteira-bem-motivo-benfeitoria = Mejora registrada: { $descricao }
nucleo-carteira-bem-motivo-benfeitoria-alterada = Mejora modificada
nucleo-carteira-bem-motivo-benfeitoria-removida = Mejora eliminada
nucleo-carteira-bem-motivo-avaliacao = Tasación registrada
nucleo-carteira-bem-motivo-avaliacao-removida = Tasación eliminada
nucleo-carteira-bem-motivo-financiamento-ligado = Préstamo vinculado al bien
nucleo-carteira-bem-motivo-financiamento-desligado = Préstamo desvinculado del bien
nucleo-carteira-bem-motivo-venda = Venta del bien registrada

## Bienes: tabla FIPE (`physical::fipe`)

nucleo-carteira-fipe-erro-codigo = código FIPE inválido: { $codigo }
nucleo-carteira-fipe-erro-ilegivel = respuesta ilegible de la tabla FIPE
nucleo-carteira-fipe-erro-sem-modelos = respuesta de la tabla FIPE sin modelos
nucleo-carteira-fipe-erro-inesperada = respuesta inesperada de la tabla FIPE
nucleo-carteira-fipe-erro-sem-codigo = opción de la tabla FIPE sin código
nucleo-carteira-fipe-erro = tabla FIPE: { $erro }
nucleo-carteira-fipe-erro-sem-preco = respuesta de la tabla FIPE sin precio
nucleo-carteira-fipe-erro-rede = no se pudo consultar la tabla FIPE ({ $erro }); inténtalo de nuevo más tarde o registra la tasación manualmente
nucleo-carteira-fipe-erro-codigo-diferente = el código FIPE del vehículo ({ $esperado }) no coincide con el de la consulta ({ $consulta })
nucleo-carteira-fipe-motivo = Cotización FIPE de { $mes }

## Bienes: valor en una fecha y la fórmula del "Explicar" del patrimonio (`physical::valuation`)

# Fuente de una tasación registrada.
nucleo-carteira-bem-fonte-manual = estimación propia
nucleo-carteira-bem-fonte-avaliacao = tasación
nucleo-carteira-bem-fonte-laudo = informe de tasación
nucleo-carteira-bem-fonte-fipe = FIPE
nucleo-carteira-bem-fonte-indice = índice
# Avisos del índice de ajuste (el mes viene como MM/AAAA).
nucleo-carteira-bem-indice-sem-dados = sin datos del índice: sincroniza los índices
nucleo-carteira-bem-indice-publicado-ate = índice publicado hasta { $mes }; el período siguiente queda sin ajuste
nucleo-carteira-bem-indice-desde = índice disponible desde { $mes }
nucleo-carteira-bem-indice-faltando = serie del índice con meses faltantes: valor sin ajuste
nucleo-carteira-bem-vida-encerrada = vida útil ya terminada en la tasación
# Partes de la fórmula: "tasación del 10/01/2025 (informe de tasación): R$ 500.000,00 ajustado por el IPCA del … = R$ 520.000,00".
nucleo-carteira-bem-formula-indice = ajustado por el { $indice } del { $de } al { $ate } (factor { $fator })
nucleo-carteira-bem-formula-depreciacao = depreciado al { $taxa }% anual durante { $anos } años (factor { $fator })
nucleo-carteira-bem-formula-linear = depreciado linealmente en { $anos ->
        [one] { $anos } año
       *[other] { $anos } años
    } hasta el { $residual }% (factor { $fator })
nucleo-carteira-bem-formula-comprado = comprado recién el { $data }
nucleo-carteira-bem-formula-vendido = vendido el { $data } por { $valor } (neto)
nucleo-carteira-bem-formula-avaliacao = tasación del { $data } ({ $fonte }): { $valor }
nucleo-carteira-bem-formula-custo = costo de adquisición de { $valor } el { $data }
nucleo-carteira-bem-benfeitorias = { $n ->
        [one] 1 mejora
       *[other] { $n } mejoras
    }
nucleo-carteira-bem-formula-benfeitorias-custo = { $benfeitorias } al costo ({ $valor })
nucleo-carteira-bem-formula-benfeitorias-regra = { $benfeitorias } de { $custo } con la misma regla ({ $valor })
nucleo-carteira-bem-formula-sem-avaliacao = sin tasación ni estimación automática: { $formula }; registra una tasación para el valor de mercado
# Fuente corta del valor.
nucleo-carteira-bem-fonte-sem-estimativa = sin estimación automática
nucleo-carteira-bem-fonte-indice-bcb = { $indice } (BCB SGS)
nucleo-carteira-bem-fonte-fipe-depreciacao = última consulta FIPE + depreciación del { $taxa }% anual
nucleo-carteira-bem-fonte-depreciacao = depreciación del { $taxa }% anual
nucleo-carteira-bem-fonte-linear = depreciación lineal en { $anos ->
        [one] { $anos } año
       *[other] { $anos } años
    }
nucleo-carteira-bem-fonte-sem-posse = sin posesión
nucleo-carteira-bem-fonte-venda = venta
nucleo-carteira-bem-fonte-avaliacao-de = tasación · { $fonte }
nucleo-carteira-bem-fonte-custo = costo de adquisición
nucleo-carteira-bem-fonte-avaliacao-regra = tasación · { $fonte } + { $regra }

## Calculadoras de renta fija (portfolio.compare_fixed_income y simulate_fixed_income_sale)
nucleo-rf-erro-valor = indica el valor invertido
nucleo-rf-erro-ofertas = indica de 1 a { $max } ofertas
nucleo-rf-erro-prazo = indica un plazo en meses o una fecha de rescate posterior a la inversión
nucleo-rf-erro-taxa = tasa inválida
nucleo-rf-erro-datas-venda = la venta debe quedar entre la inversión y el vencimiento
nucleo-rf-tipo-cdb = CDB
nucleo-rf-tipo-rdb = RDB
nucleo-rf-tipo-lc = LC
nucleo-rf-tipo-lci = LCI
nucleo-rf-tipo-lca = LCA
nucleo-rf-tipo-cri = CRI
nucleo-rf-tipo-cra = CRA
nucleo-rf-tipo-debenture = Debenture
nucleo-rf-tipo-debenture-incentivada = Debenture incentivada
nucleo-rf-tipo-tesouro-selic = Tesouro Selic
nucleo-rf-tipo-tesouro-prefixado = Tesouro Prefixado
nucleo-rf-tipo-tesouro-ipca = Tesouro IPCA+
nucleo-rf-tipo-tesouro-renda = Tesouro Renda+
nucleo-rf-tipo-poupanca = Cuenta de ahorro (poupança)
nucleo-rf-tipo-outro = Renta fija
nucleo-rf-taxa-cdi-pct = { $taxa }% del CDI
nucleo-rf-taxa-cdi-mais = CDI + { $taxa }%
nucleo-rf-taxa-selic-mais = Selic + { $taxa }%
nucleo-rf-taxa-pre = { $taxa }% anual
nucleo-rf-taxa-ipca-mais = IPCA + { $taxa }%
nucleo-rf-nota-iof = Rescate antes de 30 días: paga IOF sobre el rendimiento.
nucleo-rf-nota-aniversario = La poupança solo rinde en el aniversario mensual: rescatar antes pierde el mes en curso.
nucleo-rf-nota-custodia = Incluye la tasa de custodia de B3 (0,20% anual; en el Tesouro Selic, solo por encima de R$ 10.000).
nucleo-rf-nota-venda = Vender antes del vencimiento recibe el precio de mercado del día, que puede quedar por encima o por debajo de la curva.
nucleo-rf-nota-venda-ipca = El IPCA hasta la venta y hasta el vencimiento es la premisa indicada, no el índice publicado.
nucleo-rf-fonte-ir = IR regresivo (22,5% a 15%) y exención de LCI, LCA, CRI, CRA y poupança: Ley 11.033/2004; debentures incentivadas: Ley 12.431/2011.
nucleo-rf-fonte-iof = IOF regresivo sobre el rendimiento hasta el día 29: Decreto 6.306/2007.
nucleo-rf-fonte-custodia = Custodia del Tesouro Direto: 0,20% anual de B3; Tesouro Selic sin tasa hasta R$ 10.000.
nucleo-rf-fonte-poupanca = Poupança: 0,5% mensual + TR con la Selic por encima del 8,5% anual; si no, 70% de la Selic + TR (Ley 8.177/1991).
nucleo-rf-aviso = Estimación con las premisas mostradas, no promesa de rendimiento ni recomendación de inversión.
nucleo-carteira-erro-ponte-periodo = el fin del período debe ser posterior al inicio

## Comparar con cualquier serie (`portfolio.benchmark_series`)

nucleo-carteira-comparar-serie-desconhecida = serie de índice desconocida
nucleo-carteira-comparar-sem-juro = el país de referencia no tiene tipo de interés de referencia en el catálogo
nucleo-carteira-comparar-cdi-mais = { $indice } + { $taxa } al año

nucleo-carteira-motivo-metodo-custo = Método de costo de las ventas

## Aportes programados

nucleo-carteira-plano = aporte programado { $id }
nucleo-carteira-plano-erro-nome = indica el nombre del aporte programado
nucleo-carteira-plano-erro-valor = el valor del aporte debe ser positivo
nucleo-carteira-plano-erro-dia = el día del aporte debe estar entre 1 y 28
nucleo-carteira-plano-erro-fim = el fin del aporte programado no puede ser antes del inicio
nucleo-carteira-plano-erro-data = esa fecha no es un aporte pendiente de este plan
nucleo-carteira-plano-nota = Aporte programado: { $plano }
nucleo-carteira-motivo-plano-criado = Aporte programado creado
nucleo-carteira-motivo-plano-alterado = Aporte programado modificado
nucleo-carteira-motivo-plano-removido = Aporte programado eliminado
nucleo-carteira-motivo-aporte-confirmado = Aporte programado confirmado
nucleo-carteira-motivo-aporte-pulado = Aporte programado omitido

## Etiquetas de activos y concentración (V161)
nucleo-ativos-etiqueta-invalida = la etiqueta debe tener de 1 a 40 caracteres (como máximo 20 por activo)
nucleo-ativos-motivo-etiquetas = Etiquetas de activos
nucleo-carteira-limite-invalido = el límite debe quedar entre 0 y 100% de la cartera
nucleo-carteira-limite-sem-ativo = indica el activo del límite
nucleo-carteira-motivo-limite = Límite de concentración

## Income ledger (portfolio/income_ledger.rs)
nucleo-carteira-razao-nota-creditado = valor acreditado por la B3 en el extracto importado: prevalece sobre el cálculo
nucleo-carteira-razao-nota-pagamento-estimado = ninguna fuente informó la fecha de pago: se usó la fecha de corte
nucleo-carteira-razao-nota-repete = repite el dividendo pagado el { $data }
nucleo-carteira-razao-nota-data-improvavel = fecha de pago muy en el futuro: revisa si el año está bien escrito
nucleo-carteira-razao-nota-sem-fonte = ninguna fuente describe este dividendo: revísalo en el extracto de la corredora
nucleo-carteira-razao-nota-sem-caixa = registrado como recibido, sin el abono en la cuenta: revisa el extracto de la corredora y registra el abono
nucleo-carteira-razao-nota-historico-sem-caixa = historial: registrado sin mover la caja de la corredora
nucleo-carteira-razao-erro-caixa-ja-lancado = este dividendo ya tiene su abono en la cuenta
nucleo-carteira-razao-erro-caixa-anunciado = dividendo aún no recibido: no hay abono que registrar
nucleo-carteira-razao-motivo-caixa = Abono de { $n } dividendo(s) registrado en la cuenta
nucleo-carteira-razao-motivo-reconstruir =
    { $n ->
        [one] Dividendos reconstruidos con los avisos oficiales ({ $n } corrección)
       *[other] Dividendos reconstruidos con los avisos oficiales ({ $n } correcciones)
    }
nucleo-carteira-razao-motivo-automatico =
    { $n ->
        [one] Dividendo oficial registrado automáticamente ({ $n })
       *[other] Dividendos oficiales registrados automáticamente ({ $n })
    }
nucleo-carteira-razao-erro-so-usuario = Solo tú puedes activar o desactivar el registro automático de dividendos.
