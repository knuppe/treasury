# Análisis de activos (núcleo: `research`): nombres y fórmulas de los indicadores, períodos,
# fuentes, motivos de no disponibilidad, notas, errores y textos de la recolección, en el idioma
# de la instalación. Los nombres de los documentos guardados con cada valor (DFP, ITR, informes
# de la CVM) y los textos que se comparan con los archivos de la CVM, la B3 y la SEC quedan en el
# código.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Historial y comparación

nucleo-pesquisa-hist-centavos = Montos en centavos de la moneda del activo.
nucleo-pesquisa-hist-sem-demonstracoes = No hay estados financieros descargados para este activo.
nucleo-pesquisa-hist-unit = Valores por título de la unit ({ $acoes } acciones).
nucleo-pesquisa-erro-comparar-minimo = compara al menos 2 activos
nucleo-pesquisa-erro-comparar-maximo = compara como máximo { $max } activos

## Comandos

nucleo-pesquisa-erro-ajustes-so-interface = los ajustes del análisis de activos solo se cambian desde la app
nucleo-pesquisa-erro-ajuste-desconhecido = ajuste desconocido: { $ajuste }
nucleo-pesquisa-erro-taxa = tasa no válida
nucleo-pesquisa-motivo-tela-criada = Filtro del screener creado: { $tela }
nucleo-pesquisa-motivo-tela-alterada = Filtro del screener modificado
nucleo-pesquisa-motivo-tela-removida = Filtro del screener eliminado

## Indicadores: nombre y fórmula

nucleo-pesquisa-ind-price = Cotización
nucleo-pesquisa-ind-price-formula = Último cierre disponible
nucleo-pesquisa-ind-market_cap = Valor de mercado
nucleo-pesquisa-ind-market_cap-formula = Σ acciones de cada clase × cotización de la clase (FII: cuotas × cotización)
nucleo-pesquisa-ind-ev = Valor de la empresa (EV)
nucleo-pesquisa-ind-ev-formula = Valor de mercado + deuda bruta − caja e inversiones financieras + participación no controladora
nucleo-pesquisa-ind-liquidity = Liquidez media diaria
nucleo-pesquisa-ind-liquidity-formula = Promedio del volumen financiero diario en las últimas sesiones (hasta 21)
nucleo-pesquisa-ind-pl = P/U
nucleo-pesquisa-ind-pl-formula = Valor de mercado ÷ utilidad neta de los controladores en los últimos 12 meses
nucleo-pesquisa-ind-pvp = P/VL
nucleo-pesquisa-ind-pvp-formula = Valor de mercado ÷ patrimonio neto de los controladores (FII: cotización ÷ valor patrimonial por cuota)
nucleo-pesquisa-ind-ev_ebitda = EV/EBITDA
nucleo-pesquisa-ind-ev_ebitda-formula = Valor de la empresa ÷ EBITDA de los últimos 12 meses (EBIT + depreciación y amortización)
nucleo-pesquisa-ind-ev_ebit = EV/EBIT
nucleo-pesquisa-ind-ev_ebit-formula = Valor de la empresa ÷ EBIT de los últimos 12 meses
nucleo-pesquisa-ind-psr = P/Ingresos (PSR)
nucleo-pesquisa-ind-psr-formula = Valor de mercado ÷ ingresos netos de los últimos 12 meses
nucleo-pesquisa-ind-p_ativo = P/Activo
nucleo-pesquisa-ind-p_ativo-formula = Valor de mercado ÷ activo total
nucleo-pesquisa-ind-dy = Dividend yield (12 meses)
nucleo-pesquisa-ind-dy-formula = Dividendos por acción con fecha de corte en los últimos 12 meses ÷ cotización
nucleo-pesquisa-ind-payout = Payout
nucleo-pesquisa-ind-payout-formula = Dividendos y JCP declarados en los últimos 12 meses ÷ utilidad neta
nucleo-pesquisa-ind-last_dividend = Último rendimiento
nucleo-pesquisa-ind-last_dividend-formula = Último pago por cuota (calendario; sin calendario, DY del mes × VP por cuota del informe)
nucleo-pesquisa-ind-roe = ROE
nucleo-pesquisa-ind-roe-formula = Utilidad neta de los últimos 12 meses ÷ patrimonio neto
nucleo-pesquisa-ind-roic = ROIC
nucleo-pesquisa-ind-roic-formula = EBIT de los últimos 12 meses × (1 − tasa nominal del impuesto sobre la renta) ÷ (patrimonio neto + participación no controladora + deuda neta)
nucleo-pesquisa-ind-roa = ROA
nucleo-pesquisa-ind-roa-formula = Utilidad neta consolidada (con participación no controladora) de los últimos 12 meses ÷ activo total
nucleo-pesquisa-ind-gross_margin = Margen bruto
nucleo-pesquisa-ind-gross_margin-formula = Utilidad bruta ÷ ingresos netos (12 meses)
nucleo-pesquisa-ind-ebitda_margin = Margen EBITDA
nucleo-pesquisa-ind-ebitda_margin-formula = EBITDA ÷ ingresos netos (12 meses)
nucleo-pesquisa-ind-ebit_margin = Margen EBIT
nucleo-pesquisa-ind-ebit_margin-formula = EBIT ÷ ingresos netos (12 meses)
nucleo-pesquisa-ind-net_margin = Margen neto
nucleo-pesquisa-ind-net_margin-formula = Utilidad neta consolidada (con participación no controladora) ÷ ingresos netos (12 meses)
nucleo-pesquisa-ind-net_debt_equity = Deuda neta / PN
nucleo-pesquisa-ind-net_debt_equity-formula = (Deuda bruta − caja e inversiones) ÷ patrimonio neto
nucleo-pesquisa-ind-net_debt_ebitda = Deuda neta / EBITDA
nucleo-pesquisa-ind-net_debt_ebitda-formula = (Deuda bruta − caja e inversiones) ÷ EBITDA de los últimos 12 meses
nucleo-pesquisa-ind-current_ratio = Liquidez corriente
nucleo-pesquisa-ind-current_ratio-formula = Activo corriente ÷ pasivo corriente
nucleo-pesquisa-ind-p_ebit = P/EBIT
nucleo-pesquisa-ind-p_ebit-formula = Valor de mercado ÷ EBIT de los últimos 12 meses
nucleo-pesquisa-ind-p_working_capital = P/Capital de trabajo
nucleo-pesquisa-ind-p_working_capital-formula = Valor de mercado ÷ (activo corriente − pasivo corriente)
nucleo-pesquisa-ind-p_ncav = P/Activo corriente neto
nucleo-pesquisa-ind-p_ncav-formula = Valor de mercado ÷ (activo corriente − pasivo total), el "net-net" de Graham
nucleo-pesquisa-ind-ebit_assets = EBIT/Activo
nucleo-pesquisa-ind-ebit_assets-formula = EBIT de los últimos 12 meses ÷ activo total
nucleo-pesquisa-ind-asset_turnover = Rotación de activos
nucleo-pesquisa-ind-asset_turnover-formula = Ingresos netos de los últimos 12 meses ÷ activo total
nucleo-pesquisa-ind-gross_debt_equity = Deuda bruta / PN
nucleo-pesquisa-ind-gross_debt_equity-formula = Préstamos y financiamientos ÷ patrimonio atribuible a los controladores
nucleo-pesquisa-ind-revenue_cagr_5y = CAGR de ingresos 5 años
nucleo-pesquisa-ind-revenue_cagr_5y-formula = (Ingresos del último ejercicio ÷ ingresos de 5 ejercicios antes)^(1/5) − 1
nucleo-pesquisa-ind-earnings_cagr_5y = CAGR de utilidad 5 años
nucleo-pesquisa-ind-earnings_cagr_5y-formula = (Utilidad del último ejercicio ÷ utilidad de 5 ejercicios antes)^(1/5) − 1
nucleo-pesquisa-ind-loss_years_10y = Años con pérdida (10 años)
nucleo-pesquisa-ind-loss_years_10y-formula = Ejercicios con utilidad neta negativa entre los últimos 10
nucleo-pesquisa-ind-lpa = UPA
nucleo-pesquisa-ind-lpa-formula = Utilidad neta de los últimos 12 meses ÷ acciones en circulación (× acciones por unit)
nucleo-pesquisa-ind-vpa = VLA
nucleo-pesquisa-ind-vpa-formula = Patrimonio neto ÷ acciones en circulación (× acciones por unit)
nucleo-pesquisa-ind-nav_per_share = Valor patrimonial por cuota
nucleo-pesquisa-ind-nav_per_share-formula = Patrimonio neto ÷ cuotas emitidas (informe mensual)
nucleo-pesquisa-ind-holders = Número de cuotistas
nucleo-pesquisa-ind-holders-formula = Total de cuotistas en el último informe mensual
nucleo-pesquisa-ind-vacancy = Vacancia
nucleo-pesquisa-ind-vacancy-formula = Área vacante ÷ área bruta alquilable
nucleo-pesquisa-ind-basel = Índice de Basilea
nucleo-pesquisa-ind-basel-formula = Patrimonio de referencia ÷ activos ponderados por riesgo
nucleo-pesquisa-ind-graham = Precio de Graham
nucleo-pesquisa-ind-graham-formula = √(22,5 × UPA × VLA)
nucleo-pesquisa-ind-graham_upside = Margen hasta el precio de Graham
nucleo-pesquisa-ind-graham_upside-formula = Precio de Graham ÷ cotización − 1
nucleo-pesquisa-ind-bazin = Precio techo de Bazin
nucleo-pesquisa-ind-bazin-formula = Promedio de los dividendos por acción de los últimos N años ÷ tasa mínima
nucleo-pesquisa-ind-bazin_upside = Margen hasta el precio de Bazin
nucleo-pesquisa-ind-bazin_upside-formula = Precio techo de Bazin ÷ cotización − 1

## Indicadores: períodos, fuentes y fórmulas armadas en el momento

nucleo-pesquisa-periodo-12m = 12 meses hasta el { $data }
nucleo-pesquisa-periodo-saldo = saldo al { $data }
nucleo-pesquisa-fonte-cotacao = cotización { $fonte } del { $data }
nucleo-pesquisa-aviso-preco-justo = Estimaciones a partir de fórmulas clásicas y de los datos informados; no son una recomendación de inversión.
nucleo-pesquisa-ind-periodo-informe = informe de { $mes }
nucleo-pesquisa-ind-periodo-vp-informe = VP del informe de { $mes }
nucleo-pesquisa-ind-periodo-data-com = fecha de corte { $data }
nucleo-pesquisa-ind-periodo-divida-ebitda = deuda al { $divida }; EBITDA de 12 meses hasta el { $ebitda }
nucleo-pesquisa-ind-periodo-exercicios = ejercicios { $de } a { $ate }
nucleo-pesquisa-ind-periodo-n-exercicios =
    { $n ->
        [one] { $n } ejercicio: { $de } a { $ate }
       *[other] { $n } ejercicios: { $de } a { $ate }
    }
nucleo-pesquisa-ind-cotas-x-cotacao = { $cotas } cuotas × cotización
nucleo-pesquisa-ind-acoes-x-cotacao = { $acoes } acciones × cotización por acción del título consultado
nucleo-pesquisa-ind-on-pn-x-cotacao = acciones ON × cotización ON + acciones PN × cotización PN
nucleo-pesquisa-ind-formula-valor-mercado = Valor de mercado = { $como }
nucleo-pesquisa-ind-formula-roic = EBIT de los últimos 12 meses × (1 − { $aliquota }%) ÷ (patrimonio neto + participación no controladora + deuda neta)
nucleo-pesquisa-ind-formula-graham = √(22,5 × UPA × VLA); 22,5 = P/U máximo de 15 × P/VL máximo de 1,5
nucleo-pesquisa-ind-formula-bazin = Promedio de los dividendos por acción de los últimos N años ÷ tasa mínima de retorno
nucleo-pesquisa-ind-formula-dy-dfc = Dividendos y JCP pagados en los últimos 12 meses por acción (estado de flujos de efectivo) ÷ cotización
nucleo-pesquisa-ind-formula-payout-pagos = Dividendos y JCP pagados en los últimos 12 meses ÷ utilidad neta
nucleo-pesquisa-ind-formula-pvp-fii = Cotización ÷ valor patrimonial por cuota (informe mensual)
nucleo-pesquisa-ind-fonte-agenda = calendario de dividendos
nucleo-pesquisa-ind-fonte-informe-fii = informe mensual de FII (DY del mes × VP por cuota)
nucleo-pesquisa-ind-fonte-dividendos-dfc = dividendos y JCP pagados (flujo de efectivo) · { $documento }
nucleo-pesquisa-ind-fonte-dividendos-sec = dividendos declarados por acción (SEC)
nucleo-pesquisa-ind-fonte-dividendos-dfc-acoes = dividendos y JCP pagados por acción en el ejercicio (flujo de efectivo ÷ acciones)
nucleo-pesquisa-ind-fonte-dfp-lucro = DFP (utilidad neta de los controladores)
nucleo-pesquisa-ind-media-anos =
    { $anos ->
        [one] promedio de { $anos } año con datos ({ $pedidos } pedidos)
       *[other] promedio de { $anos } años con datos ({ $pedidos } pedidos)
    }
nucleo-pesquisa-ind-media-anos-agenda =
    { $anos ->
        [one] el calendario de dividendos empieza en { $ano }: promedio de { $anos } año con datos ({ $pedidos } pedidos)
       *[other] el calendario de dividendos empieza en { $ano }: promedio de { $anos } años con datos ({ $pedidos } pedidos)
    }
nucleo-pesquisa-ind-premissa-lpa = UPA (12 meses)
nucleo-pesquisa-ind-premissa-vpa = VLA
nucleo-pesquisa-ind-premissa-acoes = Acciones en circulación
nucleo-pesquisa-ind-premissa-taxa = Tasa mínima
nucleo-pesquisa-ind-premissa-anos = Años
nucleo-pesquisa-ind-premissa-media = Promedio anual
nucleo-pesquisa-ind-premissa-dividendos-ano = Dividendos por acción en { $ano }

## Indicadores: notas

nucleo-pesquisa-ind-nota-escala-acoes = La cantidad de acciones de { $documento } parece estar en { $escala ->
    [milhares] miles
   *[unidades] unidades × 1.000
} (P/VL { $pvp } con la cantidad informada): corregida por un factor de { $fator }.
nucleo-pesquisa-ind-nota-financeira = Institución financiera: EBITDA, valor de la empresa, deuda neta, márgenes y liquidez corriente no aplican.
nucleo-pesquisa-ind-nota-holding = Holding o aseguradora: el resultado viene de sus participaciones (método de participación); EBIT, valor de la empresa y márgenes no aplican.
nucleo-pesquisa-ind-nota-banco-padrao-bc = Banco: utilidad y patrimonio según los estados individuales, con la norma contable del Banco Central (la que usan el mercado y los dividendos). En el consolidado IFRS: utilidad de 12 meses de { $lucro } y patrimonio de { $patrimonio }.
nucleo-pesquisa-ind-nota-unit = Unit con { $acoes } acciones: valores por acción multiplicados por la composición.
nucleo-pesquisa-ind-nota-classe = Los indicadores fundamentales valen para acciones y FII; para esta clase solo hay cotización y liquidez.
nucleo-pesquisa-ind-nota-sem-demonstracoes = No hay estados financieros descargados para este activo: haz la recolección del análisis de activos.
nucleo-pesquisa-ind-nota-acoes-sec = Acciones en circulación de la portada del último formulario (dei:EntityCommonStockSharesOutstanding).

## Indicadores: por qué no está disponible

nucleo-pesquisa-ind-indisponivel = no disponible
nucleo-pesquisa-ind-divisao-zero = división por cero
nucleo-pesquisa-ind-nao-se-aplica-financeiras = no aplica a instituciones financieras ni aseguradoras
nucleo-pesquisa-ind-nao-se-aplica-holdings = no aplica a holdings ni aseguradoras (el resultado viene del método de participación)
nucleo-pesquisa-ind-nao-se-aplica-fii = no aplica a FII
nucleo-pesquisa-ind-sem-cotacao = sin cotización
nucleo-pesquisa-ind-sem-cotas = sin el número de cuotas en el informe mensual
nucleo-pesquisa-ind-sem-acoes = sin el número de acciones
nucleo-pesquisa-ind-unit-desconhecida = composición de la unit desconocida
nucleo-pesquisa-ind-sem-rendimentos-informe = sin rendimientos en el informe mensual
nucleo-pesquisa-ind-sem-proventos = sin dividendos en el calendario ni en los estados financieros
nucleo-pesquisa-ind-lpa-negativo = UPA negativa o cero (pérdida en los últimos 12 meses)
nucleo-pesquisa-ind-vpa-negativo = VLA negativo o cero (patrimonio neto negativo)
nucleo-pesquisa-ind-taxa-positiva = la tasa mínima tiene que ser positiva
nucleo-pesquisa-ind-sem-historico-dividendos = sin historial de dividendos
nucleo-pesquisa-ind-sem-dividendos-periodo = sin dividendos en el período
nucleo-pesquisa-ind-sem-lucro-12m = sin utilidad de los últimos 12 meses
nucleo-pesquisa-ind-sem-receita-12m = sin ingresos de los últimos 12 meses
nucleo-pesquisa-ind-sem-ebit-12m = sin EBIT de los últimos 12 meses
nucleo-pesquisa-ind-sem-da-12m = sin depreciación y amortización de los últimos 12 meses
nucleo-pesquisa-ind-sem-lucro-bruto-12m = sin utilidad bruta de los últimos 12 meses
nucleo-pesquisa-ind-sem-patrimonio = sin patrimonio neto
nucleo-pesquisa-ind-sem-ativo-total = sin activo total
nucleo-pesquisa-ind-sem-divida-bruta = sin deuda bruta
nucleo-pesquisa-ind-sem-ativo-circulante = sin activo corriente
nucleo-pesquisa-ind-sem-passivo-circulante = sin pasivo corriente
nucleo-pesquisa-ind-sem-dividendos = sin dividendos
nucleo-pesquisa-ind-sem-volume = sin volumen negociado
nucleo-pesquisa-ind-sem-valor-mercado = sin valor de mercado
nucleo-pesquisa-ind-lucro-zero = utilidad cero
nucleo-pesquisa-ind-patrimonio-negativo = patrimonio neto negativo
nucleo-pesquisa-ind-ebitda-negativo = EBITDA negativo o cero
nucleo-pesquisa-ind-ebit-negativo = EBIT negativo o cero
nucleo-pesquisa-ind-receita-zero = ingresos cero
nucleo-pesquisa-ind-lucro-negativo = utilidad negativa o cero
nucleo-pesquisa-ind-denominador-negativo = denominador negativo o cero
nucleo-pesquisa-ind-capital-negativo = capital invertido negativo o cero
nucleo-pesquisa-ind-capital-pequeno = capital invertido por debajo del 10% del activo (mucha caja neta): el ROIC no dice mucho
nucleo-pesquisa-ind-capital-giro-negativo = capital de trabajo negativo o cero
nucleo-pesquisa-ind-sem-passivo-total = sin pasivo total en el último balance
nucleo-pesquisa-ind-ativo-circulante-liquido-negativo = activo corriente menor que el pasivo total
nucleo-pesquisa-ind-bloqueado = dato de entrada inválido: { $motivo }
nucleo-pesquisa-ind-milhoes = R$ { $valor } millones
nucleo-pesquisa-ind-check-lucro-individual = la utilidad de los controladores vino en cero en el consolidado ({ $informado }); se usó la de los estados individuales, { $usado }
nucleo-pesquisa-ind-check-lucro-total = la utilidad de los controladores vino en cero en el consolidado ({ $informado }); se usó el total menos la participación no controladora, { $usado }
nucleo-pesquisa-ind-check-lucro-diverge = la utilidad de los controladores en el consolidado ({ $consolidado }) y la de los estados individuales ({ $individual }) difieren { $diferenca }, sin fuente para decidir cuál vale
nucleo-pesquisa-ind-check-lucro-difere = la utilidad de los controladores en el consolidado ({ $consolidado }) difiere { $diferenca } de la de los estados individuales ({ $individual })
nucleo-pesquisa-ind-check-patrimonio-diverge = el patrimonio de los controladores en el consolidado ({ $consolidado }) y el de los estados individuales ({ $individual }) difieren { $diferenca }, sin fuente para decidir cuál vale
nucleo-pesquisa-ind-check-patrimonio-difere = el patrimonio de los controladores en el consolidado ({ $consolidado }) difiere { $diferenca } del de los estados individuales ({ $individual })
nucleo-pesquisa-ind-check-balanco-antigo = el último balance es del { $data }, con más de 200 días
nucleo-pesquisa-ind-check-reapresentado = la compañía reexpresó { $item } del período hasta { $periodo } ({ $diferenca }); los 12 meses usan la cifra publicada en su momento
nucleo-pesquisa-ind-check-proventos-estimados = sin dividendos oficiales de B3 para el papel: estimación por los dividendos e intereses sobre capital pagados en el flujo de efectivo (queda fuera de los rankings)
nucleo-pesquisa-ind-check-acoes-escala = la cantidad de acciones de { $documento } estaba en miles; verificada con B3 ({ $b3 } millones)
nucleo-pesquisa-ind-check-acoes-total-liquido = la compañía informó a la CVM el total de acciones ya sin las { $tesouraria } millones en tesorería ({ $cvm } millones); verificado con B3, que tiene { $b3 } millones el { $data }
nucleo-pesquisa-ind-check-acoes-evento = cantidad ajustada por eventos en acciones después del balance (factor { $fator }): { $cvm } millones en el balance, { $b3 } millones en B3 el { $data }
nucleo-pesquisa-ind-check-acoes-diverge = B3 tiene { $b3 } millones de acciones el { $data } y el balance { $cvm } millones ({ $diferenca }), sin evento que lo explique
nucleo-pesquisa-ind-check-acoes-difere = B3 tiene { $b3 } millones de acciones el { $data }, { $diferenca } distinto del balance ({ $cvm } millones), sin evento que lo explique
nucleo-pesquisa-ind-sem-exercicios = sin ejercicios
nucleo-pesquisa-ind-poucos-exercicios = menos de 6 ejercicios descargados
nucleo-pesquisa-ind-ponta-negativa = valor negativo o cero en uno de los extremos
nucleo-pesquisa-ind-basileia-fora-cvm = el Índice de Basilea no está en los archivos de la CVM (está en las notas de los estados financieros y en el IF.data del Banco Central)
nucleo-pesquisa-ind-sem-informe = sin informe mensual
nucleo-pesquisa-ind-sem-vp-cota = sin valor patrimonial por cuota
nucleo-pesquisa-ind-sem-rendimentos = sin rendimientos
nucleo-pesquisa-ind-vacancia-fora = sin vacancia en el informe trimestral de la CVM de los últimos 12 meses

## Partidas de los estados financieros

nucleo-pesquisa-item-revenue = Ingresos netos
nucleo-pesquisa-item-gross_profit = Utilidad bruta
nucleo-pesquisa-item-ebit = EBIT (resultado antes del resultado financiero y de los impuestos)
nucleo-pesquisa-item-da = Depreciación, amortización y agotamiento
nucleo-pesquisa-item-net_income = Utilidad neta (controladores)
nucleo-pesquisa-item-cfo = Efectivo neto de las actividades de operación
nucleo-pesquisa-item-capex = Inversiones en activo fijo e intangibles
nucleo-pesquisa-item-dividends_paid = Dividendos y JCP pagados
nucleo-pesquisa-item-dividends_declared = Dividendos y JCP declarados
nucleo-pesquisa-item-dps_declared = Dividendos declarados por acción
nucleo-pesquisa-item-cogs = Costo de los bienes y servicios vendidos
nucleo-pesquisa-item-sga = Gastos de venta, generales y administrativos
nucleo-pesquisa-item-fin_income = Ingresos financieros
nucleo-pesquisa-item-fin_expense = Gastos financieros
nucleo-pesquisa-item-ebt = Resultado antes de los impuestos sobre la utilidad
nucleo-pesquisa-item-income_tax = Impuesto sobre la renta y contribución social (negativo = gasto)
nucleo-pesquisa-item-buybacks = Recompra de acciones (adquisición de acciones propias)
nucleo-pesquisa-item-share_issuance = Aumento de capital y emisión de acciones
nucleo-pesquisa-item-total_assets = Activo total
nucleo-pesquisa-item-current_assets = Activo corriente
nucleo-pesquisa-item-cash = Caja, equivalentes e inversiones financieras
nucleo-pesquisa-item-current_liabilities = Pasivo corriente
nucleo-pesquisa-item-total_liabilities = Pasivo total (exigible)
nucleo-pesquisa-item-gross_debt = Deuda bruta (préstamos y financiamientos)
nucleo-pesquisa-item-equity = Patrimonio neto (controladores)
nucleo-pesquisa-item-shares_outstanding = Acciones en circulación (sin tesorería)
nucleo-pesquisa-item-shares_on = Acciones ordinarias en circulación
nucleo-pesquisa-item-shares_pn = Acciones preferentes en circulación
nucleo-pesquisa-item-receivables = Cuentas por cobrar (corriente)
nucleo-pesquisa-item-inventories = Inventarios (corriente)
nucleo-pesquisa-item-ppe = Activo fijo
nucleo-pesquisa-item-intangible = Intangible
nucleo-pesquisa-item-suppliers = Proveedores (corriente)
nucleo-pesquisa-item-debt_st = Préstamos y financiamientos de corto plazo
nucleo-pesquisa-item-debt_lt = Préstamos y financiamientos de largo plazo
nucleo-pesquisa-item-leases = Arrendamientos fuera de préstamos y financiamientos
nucleo-pesquisa-item-retained = Utilidades retenidas (reservas de utilidades + resultados acumulados)
nucleo-pesquisa-item-minority = Participación no controladora
nucleo-pesquisa-item-net_income_total = Utilidad neta consolidada (con participación no controladora)
nucleo-pesquisa-item-net_income_consolidated = Utilidad atribuible a los controladores en el consolidado, como se informó
nucleo-pesquisa-item-net_income_individual = Utilidad neta de los estados individuales
nucleo-pesquisa-item-equity_income = Resultado por participación en asociadas (método de participación)
nucleo-pesquisa-item-equity_consolidated = Patrimonio atribuible a los controladores en el consolidado
nucleo-pesquisa-item-equity_individual = Patrimonio de los estados individuales
nucleo-pesquisa-item-shares_total = Acciones del capital pagado (con tesorería)
nucleo-pesquisa-item-shares_treasury = Acciones en tesorería
nucleo-pesquisa-item-shares_b3 = Acciones del capital social en B3
nucleo-pesquisa-item-shares_b3_on = Acciones ordinarias del capital social en B3
nucleo-pesquisa-item-shares_b3_pn = Acciones preferentes del capital social en B3
nucleo-pesquisa-item-revenue_restated = Ingresos netos reexpresados en el documento siguiente
nucleo-pesquisa-item-ebit_restated = EBIT reexpresado en el documento siguiente
nucleo-pesquisa-item-net_income_restated = Utilidad neta reexpresada en el documento siguiente
nucleo-pesquisa-item-nav = Patrimonio neto del fondo
nucleo-pesquisa-item-nav_per_share = Valor patrimonial por cuota
nucleo-pesquisa-item-fii_shares = Cuotas emitidas
nucleo-pesquisa-item-holders = Número de cuotistas
nucleo-pesquisa-item-dy_month = Dividend yield del mes sobre el patrimonio (informado)
nucleo-pesquisa-item-income_per_share = Rendimiento por cuota (DY del mes × VP por cuota)
nucleo-pesquisa-item-fii_total_assets = Activo total del fondo
nucleo-pesquisa-item-fii_liabilities = Pasivo total del fondo
nucleo-pesquisa-item-fii_acq_obligations = Obligaciones por adquisición de inmuebles
nucleo-pesquisa-item-fii_securitization = Obligaciones por titulización de cuentas por cobrar
nucleo-pesquisa-item-fii_real_estate = Derechos sobre bienes inmuebles
nucleo-pesquisa-item-fii_cri = CRI y CRA en la cartera
nucleo-pesquisa-item-fii_fii_shares = Cuotas de otros FII en la cartera
nucleo-pesquisa-item-fii_liquidity = Activos para necesidades de liquidez
nucleo-pesquisa-item-fii_rent_receivable = Alquileres por cobrar
nucleo-pesquisa-item-fii_admin_fee_pct = Comisión de administración del mes sobre el patrimonio
nucleo-pesquisa-item-fii_equity_return_month = Rentabilidad patrimonial del mes
nucleo-pesquisa-item-fii_vacancy_financial = Vacancia financiera (ponderada por los ingresos)
nucleo-pesquisa-item-fii_vacancy_physical = Vacancia física (ponderada por el área)
nucleo-pesquisa-item-fii_default_rate = Morosidad (ponderada por los ingresos)
nucleo-pesquisa-item-fii_top_property_share = Mayor inmueble en los ingresos del fondo
nucleo-pesquisa-item-fii_properties = Número de inmuebles
nucleo-pesquisa-item-fii_top_tenant_share = Mayor inquilino en los ingresos del fondo (aproximación)
nucleo-pesquisa-item-fii_walt_years = Plazo medio de los contratos (WALT, años)
nucleo-pesquisa-item-fii_undetermined_share = Ingresos de contratos con plazo indeterminado
nucleo-pesquisa-item-fii_recurring_result = Resultado financiero neto del trimestre
nucleo-pesquisa-item-fii_declared_income = Rendimientos declarados en el trimestre
nucleo-pesquisa-item-fii_cri_top_issuer_share = Mayor emisor en los CRI/CRA (aproximación del deudor)
nucleo-pesquisa-item-desconhecido = Partida
nucleo-pesquisa-doc-b3-capital = B3, empresas listadas (capital social) el { $data }
nucleo-pesquisa-erro-tipo-periodo = tipo de período no válido: { $tipo }

## Ajustes del análisis

nucleo-pesquisa-aviso-sec = La SEC exige identificarse con un correo de contacto: cambia "contato@exemplo" por tu correo en Ajustes › Fuentes de datos (p. ej.: "Treasury/0.1 tu.nombre@email.com").
nucleo-pesquisa-erro-bazin-taxa = la tasa mínima de Bazin debe ser positiva (p. ej.: 0.06 o 6 para 6%)
nucleo-pesquisa-erro-bazin-anos = los años del promedio de Bazin van de 1 a 10
nucleo-pesquisa-erro-sec-ua-vazio = indica el User-Agent de la SEC (nombre de la app y correo)
nucleo-pesquisa-erro-sec-ua-invalido = User-Agent no válido
nucleo-pesquisa-erro-sec-ua-email = la SEC exige un correo de contacto en el User-Agent (p. ej.: Treasury/0.1 tu.nombre@email.com)

## Recolección: B3 y brapi

nucleo-pesquisa-erro-cotahist-sem-txt = COTAHIST sin archivo .TXT
nucleo-pesquisa-erro-b3-companhia = respuesta de la B3 (compañía) en un formato inesperado: { $erro }
nucleo-pesquisa-erro-b3-fundo = respuesta de la B3 (fondo) en un formato inesperado: { $erro }
nucleo-pesquisa-erro-b3-historico = historial de dividendos de la B3 en un formato inesperado: { $erro }
nucleo-pesquisa-erro-b3-vazia = la B3 respondió sin contenido
nucleo-pesquisa-erro-304 = respuesta 304 sin solicitud condicional
nucleo-pesquisa-erro-b3-api = falla en la API de la B3
nucleo-pesquisa-b3-log-proventos =
    { $n ->
        [one] { $n } dividendo
       *[other] { $n } dividendos
    }
nucleo-pesquisa-b3-log-proventos-eventos = { $proventos } dividendo(s), { $eventos } evento(s) en acciones
nucleo-pesquisa-b3-passo-historico = B3 historial de dividendos { $codigo }
nucleo-pesquisa-b3-passo-resumo =
    { $pedidos ->
        [one] B3 empresas y fondos listados · dividendos y eventos ({ $feitos } de { $total } emisores y fondos, { $pedidos } solicitud)
       *[other] B3 empresas y fondos listados · dividendos y eventos ({ $feitos } de { $total } emisores y fondos, { $pedidos } solicitudes)
    }
nucleo-pesquisa-b3-parou = la B3 no respondió { $vezes } veces seguidas; el resto queda para la próxima recolección
nucleo-pesquisa-b3-nada-a-atualizar = nada que actualizar: todos se consultaron hace poco
nucleo-pesquisa-erro-brapi-invalida = respuesta no válida de brapi: { $erro }
nucleo-pesquisa-erro-brapi-sem-resultados = brapi: respuesta sin resultados

## Archivos de la CVM y datos del activo

nucleo-pesquisa-erro-zip = archivo zip no válido: { $erro }
nucleo-pesquisa-erro-coluna-ausente = falta la columna { $coluna }
nucleo-pesquisa-erro-ipe-sem-csv = no se encontró el CSV del IPE en el zip
nucleo-pesquisa-liquidez-cotahist =
    { $n ->
        [one] promedio de { $n } sesión, B3 COTAHIST
       *[other] promedio de { $n } sesiónes, B3 COTAHIST
    }
nucleo-pesquisa-liquidez-cotacoes =
    { $n ->
        [one] promedio de { $n } sesión (cotización × volumen, { $fonte })
       *[other] promedio de { $n } sesiónes (cotización × volumen, { $fonte })
    }
nucleo-pesquisa-erro-asset-ou-ticker = indica asset_id o ticker
nucleo-pesquisa-erro-fora-do-universo = { $ticker } en el universo descargado (haz la recolección del análisis de activos)

## Dividendos: origen y comprobación con la B3

nucleo-pesquisa-div-fonte-b3 = B3 (oficial), dividendos en efectivo de las empresas y fondos listados
nucleo-pesquisa-div-fonte-agenda = calendario de dividendos ({ $fontes })
nucleo-pesquisa-div-fonte-b3-diverge = { $b3 }; el calendario ({ $fontes }) difiere de la B3
nucleo-pesquisa-div-fonte-lancados = dividendos registrados (bruto ÷ cantidad)
nucleo-pesquisa-div-nota-ja-corrige = { $fonte } ya corrige el valor por los desdoblamientos y bonificaciones posteriores a la fecha de corte
nucleo-pesquisa-div-nota-nao-corrige = { $fonte } no corrige el valor por los desdoblamientos y bonificaciones posteriores a la fecha de corte
nucleo-pesquisa-div-nota-houve-evento = hubo desdoblamiento, agrupamiento o bonificación después de la fecha de corte
nucleo-pesquisa-div-nota-sem-par = { $tipos } sin par en la B3
nucleo-pesquisa-div-nota-sem-b3 = No hay dividendos de la B3 para este título (haz la recolección del análisis de activos o verifica si la B3 lo conoce).
nucleo-pesquisa-div-nota-sem-agenda = No hay calendario de dividendos de Yahoo ni de Fundamentus para comparar.
nucleo-pesquisa-div-nota-b3-desde = La B3 cubre desde el { $data } (el historial completo solo se descarga para emisores de activos registrados).
nucleo-pesquisa-div-nota-valores-epoca = La suma de 12 meses del calendario ({ $agenda }) coincide con la de la B3 con los valores de la época ({ $b3 }); la diferencia viene de la corrección por desdoblamientos y bonificaciones, que el DY necesita (cotización de hoy).
nucleo-pesquisa-div-nota-divergencia = Diferencia material: los indicadores usan la B3 (fuente oficial).
nucleo-pesquisa-erro-sem-codigo = el activo no tiene código de negociación
nucleo-pesquisa-erro-conferencia-br = la comprobación con la B3 vale para acciones y FII brasileños

## Fundamentus

nucleo-pesquisa-erro-ticker-fundamentus = ticker no válido para Fundamentus: { $ticker }

## IF.data del Banco Central

nucleo-pesquisa-ifdata-convencao = semestre hasta la fecha (marzo/septiembre: 3 meses; junio/diciembre: 6 meses)
nucleo-pesquisa-ifdata-relatorio-cadastro = registro
nucleo-pesquisa-ifdata-relatorio-resumo = resumen
nucleo-pesquisa-ifdata-relatorio-ativo = activo
nucleo-pesquisa-ifdata-relatorio-resultado = resultado
nucleo-pesquisa-ifdata-relatorio-capital = capital
nucleo-pesquisa-ifdata-relatorio-carteira = cartera por riesgo
nucleo-pesquisa-ifdata-relatorio = informe
nucleo-pesquisa-erro-ifdata-invalida = respuesta no válida del IF.data: { $erro }
nucleo-pesquisa-ifdata-fonte-desligada = la fuente Banco Central está apagada en Ajustes › Fuentes de datos: enciéndela y la recolección diaria descarga el IF.data
nucleo-pesquisa-ifdata-nao-baixado = el IF.data del Banco Central aún no se descargó (la recolección del análisis de activos corre sola una vez al día; el resultado de cada fuente está en Activos › Datos del análisis)
nucleo-pesquisa-ifdata-fora-do-cadastro = el CNPJ { $cnpj } no aparece en el registro del IF.data del Banco Central (trimestre { $trimestre }): la compañía no es un banco que reporta al Banco Central
nucleo-pesquisa-ifdata-sem-valores = el IF.data del Banco Central no trajo valores de { $banco } ({ $codigo }) hasta el trimestre { $trimestre }

## Red

nucleo-pesquisa-erro-post = POST no admitido
nucleo-pesquisa-erro-servidor-recusou =
    { $tentativas ->
        [one] el servidor rechazó el acceso (HTTP { $status }) después de { $tentativas } intento; inténtalo más tarde
       *[other] el servidor rechazó el acceso (HTTP { $status }) después de { $tentativas } intentos; inténtalo más tarde
    }
nucleo-pesquisa-erro-token-recusado = acceso rechazado (HTTP { $status }): revisa el token
nucleo-pesquisa-erro-http-tentativas =
    { $tentativas ->
        [one] HTTP { $status } después de { $tentativas } intento
       *[other] HTTP { $status } después de { $tentativas } intentos
    }

## Documentos de los últimos 12 meses

nucleo-pesquisa-doc-exercicio = ejercicio: { $documento }
nucleo-pesquisa-doc-ltm = { $exercicio } + acumulado hasta { $ate } − acumulado hasta { $ate_anterior } ({ $documento }; { $documento_anterior })
nucleo-pesquisa-doc-ultimo-exercicio = último ejercicio: { $documento }

## Radar de dividendos

nucleo-pesquisa-radar-poucos-proventos = historial insuficiente (menos de 2 dividendos en 24 meses)
nucleo-pesquisa-radar-insuficiente = historial insuficiente

## Rankings y screener

nucleo-pesquisa-erro-indicador-desconhecido = indicador desconocido: { $indicador } (ver research.catalog)
nucleo-pesquisa-erro-ordenar-desconhecido = indicador desconocido para ordenar: { $indicador }
nucleo-pesquisa-erro-campo-desconhecido = campo desconocido en el filtro: { $campo } (indicadores de research.catalog, o { $campos })
nucleo-pesquisa-erro-filtro = filtro { $campo }: { $motivo }
nucleo-pesquisa-filtro-comparacao = la comparación de tamaño solo vale para indicadores
nucleo-pesquisa-filtro-numero = indica un número en `value`
nucleo-pesquisa-filtro-between = `between` necesita `value: [mínimo, máximo]` en un indicador
nucleo-pesquisa-filtro-in = `in`/`not_in` necesitan una lista en `value`
nucleo-pesquisa-filtro-contains = `contains` necesita un texto en un campo de texto
nucleo-pesquisa-filtro-value = indica `value`
nucleo-pesquisa-erro-grupo-vazio = grupo de filtros vacío
nucleo-pesquisa-filtro-op-entre = entre
nucleo-pesquisa-filtro-op-em = en
nucleo-pesquisa-filtro-op-fora-de = fuera de
nucleo-pesquisa-filtro-op-contem = contiene
nucleo-pesquisa-filtro-op-disponivel = disponible
nucleo-pesquisa-filtro-op-indisponivel = no disponible
nucleo-pesquisa-filtro-e = Y
nucleo-pesquisa-filtro-ou = O
nucleo-pesquisa-filtro-todos = todos los títulos
nucleo-pesquisa-ref-tela = filtro { $id }
nucleo-pesquisa-erro-tela-nome = indica el nombre del filtro
nucleo-pesquisa-erro-tela-duplicada = ya existe un filtro llamado { $nome }

## SEC y base local

nucleo-pesquisa-erro-sec-cadastro = registro de la SEC no válido: { $erro }
nucleo-pesquisa-erro-sec-tickers = company_tickers.json no válido: { $erro }
nucleo-pesquisa-erro-sec-companyfacts = companyfacts no válido: { $erro }
nucleo-pesquisa-doc-sec = { $formulario } { $numero } (presentado el { $data })
nucleo-pesquisa-erro-valor-banco = valor no válido: { $valor }
nucleo-pesquisa-erro-cotacao-banco = cotización no válida en la base de datos: { $cotacao }

## Recolección del análisis: pasos y avisos

nucleo-pesquisa-passo-fii-mensal = CVM informe mensual de FII { $ano }
nucleo-pesquisa-passo-fii-trimestral = CVM informe trimestral de FII { $ano }
nucleo-pesquisa-passo-sec-email = SEC EDGAR: indica el correo de contacto
nucleo-pesquisa-passo-sec-cadastro = SEC registro { $ticker }
nucleo-pesquisa-erro-sec-sem-ticker = { $ticker } no está en la lista de tickers de la SEC
nucleo-pesquisa-passo-ifdata-sondagem = Banco Central IF.data: sondeo de { $periodo }
nucleo-pesquisa-passo-parte = parte { $parte }/{ $partes }
nucleo-pesquisa-aviso-acoes-em-milhares =
    { $n ->
        [one] { $n } compañía informó la cantidad de acciones en miles en algún documento (corregida con la utilidad por acción): { $companhias }
       *[other] { $n } compañías informaron la cantidad de acciones en miles en algún documento (corregida con la utilidad por acción): { $companhias }
    }
nucleo-pesquisa-aviso-lucro-controladores =
    { $n ->
        [one] { $n } compañía dejó en cero la utilidad atribuible a los controladores en el consolidado en algún documento (corregida con los estados individuales): { $companhias }
       *[other] { $n } compañías dejaron en cero la utilidad atribuible a los controladores en el consolidado en algún documento (corregida con los estados individuales): { $companhias }
    }
nucleo-pesquisa-feedback-faixa-titulo = Indicadores fuera del rango plausible en el universo
nucleo-pesquisa-feedback-faixa-corpo = { $n } valor(es) de indicador fuera de los rangos amplios de plausibilidad (docs/indicators.md, E11). No se bloquearon, porque pueden ser ciertos; vale la pena revisar el dato de entrada de cada uno:
    { $lista }
nucleo-pesquisa-feedback-faixa-resolvido = En la recolección de hoy, ningún indicador quedó fuera de los rangos.

## Recolección: nombre del IF.data

nucleo-pesquisa-passo-ifdata = Banco Central IF.data

## brapi.dev: plan y cuota

nucleo-pesquisa-brapi-plano-gratuito = brapi.dev: la verificación de indicadores y dividendos es de los planes de pago; en el plan gratuito queda desactivada, sin gastar la cuota.
nucleo-pesquisa-brapi-limite = brapi.dev: se alcanzó el límite de uso del mes; el resto queda para después de la renovación o de un límite mayor en Ajustes › Fuentes de datos.

nucleo-pesquisa-erro-rede-bloqueada = la red de este ordenador bloqueó la conexión segura con { $site } (filtro de seguridad de la empresa, antivirus, proxy o DNS que bloquea el sitio); no es el sitio ni Treasury. Permite el sitio en el filtro o usa otra red. Detalle: { $detalhe }

# Indicadores dos métodos do explorador de mercado
nucleo-pesquisa-ind-earnings_yield = EBIT/EV
nucleo-pesquisa-ind-earnings_yield-formula = EBIT de los últimos 12 meses ÷ valor de la empresa (el inverso del EV/EBIT; Magic Formula)
nucleo-pesquisa-ind-magic_roc = ROC (Magic Formula)
nucleo-pesquisa-ind-magic_roc-formula = EBIT de los últimos 12 meses ÷ (capital de trabajo neto + inmovilizado)
nucleo-pesquisa-ind-fcf_yield = Flujo de caja libre / valor de mercado
nucleo-pesquisa-ind-fcf_yield-formula = (Caja de las operaciones − inversiones en inmovilizado, 12 meses) ÷ valor de mercado
nucleo-pesquisa-ind-peg = PEG
nucleo-pesquisa-ind-peg-formula = P/U ÷ crecimiento anual de la utilidad en 5 años (en puntos porcentuales; Peter Lynch)
nucleo-pesquisa-ind-eps_growth_10y = Crecimiento de la utilidad en 10 años
nucleo-pesquisa-ind-eps_growth_10y-formula = Utilidad media de los 3 últimos ejercicios ÷ utilidad media de hace 10 a 8 ejercicios − 1 (Graham defensivo; utilidad total, no por acción)
nucleo-pesquisa-ind-dividend_years = Años seguidos con dividendos
nucleo-pesquisa-ind-dividend_years-formula = Años completos seguidos, hasta el último, con algún dividendo pagado (hasta 20)
nucleo-pesquisa-ind-dividend_cagr_5y = Crecimiento de los dividendos en 5 años
nucleo-pesquisa-ind-dividend_cagr_5y-formula = Crecimiento anual compuesto de los dividendos por acción o cuota entre el año de hace 5 años y el último año completo
nucleo-pesquisa-ind-sem-imobilizado = Sin el inmovilizado en el último balance
nucleo-pesquisa-ind-sem-fco-12m = Sin la caja de las operaciones de los últimos 12 meses
nucleo-pesquisa-ind-fonte-peg = P/U y crecimiento de la utilidad en 5 años
nucleo-pesquisa-ind-peg-negativo = P/U o crecimiento de la utilidad negativo: el PEG no se aplica
nucleo-pesquisa-ind-peg-sem-dados = Sin P/U o sin crecimiento de la utilidad en 5 años

# Score em lote do universo
nucleo-pesquisa-ind-score = Score
nucleo-pesquisa-ind-score-formula = Score de salud del Treasury (0 a 100), calculado en la recogida para todo el universo
nucleo-pesquisa-ind-health = Salud (score)
nucleo-pesquisa-ind-health-formula = Sub-score Salud del negocio, sin valuación ni momentum (0 a 100)
nucleo-pesquisa-ind-attractiveness = Atractivo (score)
nucleo-pesquisa-ind-attractiveness-formula = Sub-score Atractivo del precio: valuación y momentum (0 a 100)
nucleo-pesquisa-ind-confidence = Confianza del score
nucleo-pesquisa-ind-confidence-formula = Cobertura de los datos × frescura × tamaño del grupo de pares (0 a 1)
nucleo-pesquisa-ind-piotroski = Piotroski F-Score
nucleo-pesquisa-ind-piotroski-formula = Criterios de Piotroski cumplidos (0 a 9)
nucleo-pesquisa-ind-altman_z = Altman Z''
nucleo-pesquisa-ind-altman_z-formula = Z'' de Altman para mercados emergentes (por encima de 2,6: zona segura)
nucleo-pesquisa-ind-beneish = Beneish M-Score
nucleo-pesquisa-ind-beneish-formula = M-Score de Beneish (por encima de −1,78 sugiere riesgo de manipulación contable)
nucleo-pesquisa-ind-momentum_12_1 = Momentum 12-1
nucleo-pesquisa-ind-momentum_12_1-formula = Rentabilidad de los últimos 12 meses sin el último mes
nucleo-pesquisa-ind-volatility_1y = Volatilidad 1 año
nucleo-pesquisa-ind-volatility_1y-formula = Desviación estándar anualizada de los rendimientos diarios de 1 año
nucleo-pesquisa-aviso-score-lote = Score del universo no calculado: { $erro }
nucleo-pesquisa-motivo-observar = Observar { $ticker } desde Mercado
nucleo-pesquisa-motivo-deixar-observar = Dejar de observar desde Mercado
nucleo-pesquisa-aviso-alerta-tela = Alertas de pantalla no revisados: { $erro }
nucleo-pesquisa-documento-link = El documento no tiene un enlace oficial de la CVM para descargar.
nucleo-pesquisa-documento-sumiu = La CVM no encontró el archivo del documento.
nucleo-pesquisa-documento-grande = El archivo del documento es demasiado grande para leer (máx. 25 MB).
nucleo-pesquisa-documento-sem-texto = No se pudo leer texto en el documento.
nucleo-pesquisa-documento-nao-encontrado = Documento no encontrado entre los capturados (usa list_corporate_documents).
nucleo-pesquisa-ind-periodo-trimestral = informe trimestral hasta el { $data }
nucleo-pesquisa-div-fonte-b3-sem-rendimentos = B3 (oficial): ningún rendimiento en efectivo del fondo en los últimos 12 meses
nucleo-pesquisa-ind-sem-rendimentos-b3 = la B3 no muestra rendimientos del fondo en los últimos 12 meses
nucleo-pesquisa-ind-fonte-declarados-fii = rendimientos declarados en los informes trimestrales de la CVM (4 trimestres hasta el { $data })
nucleo-pesquisa-ind-formula-dy-fii-declarados = Rendimientos declarados en los 4 últimos informes trimestrales ÷ cuotas emitidas ÷ cotización (estimación: sin los proventos de la B3)
nucleo-pesquisa-ind-formula-dy-fii-meses = Suma del rendimiento por cuota de los informes mensuales (DY del mes × VP por cuota) ÷ cotización (estimación: sin los proventos de la B3). Meses: { $meses }
nucleo-pesquisa-div-fonte-b3-amortizacao = { $fonte }; rendimientos anteriores a las amortizaciones del { $datas } en la escala de la cuota de hoy (sin el capital devuelto)
