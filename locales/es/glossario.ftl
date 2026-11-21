# Glosario: los términos de la app explicados en el "¿Qué es esto?" del panel simple y en los
# íconos ? junto a las etiquetas (docs/profile.md, docs/i18n-design.md). Revisión: borrador por
# IA, sin revisión nativa.
#
# Cada término tiene -termo (el nombre), -simples (tono "muy simple"), -equilibrado (tonos
# equilibrado y técnico; sin ella, vale la simple) y -busca: una expresión regular que se prueba
# contra la etiqueta en minúsculas y sin acentos, con las alternativas separadas por |. Gana el
# primer término que coincide, en el orden de este archivo (el mismo de content/glossary.ts), así
# que los más específicos van primero. La búsqueda de cada idioma sigue las etiquetas de ese
# idioma. Sin llaves en la búsqueda (sintaxis de Fluent).

glossario-reserva-termo = Fondo de emergencia
glossario-reserva-simples = Dinero guardado solo para imprevistos, como una consulta médica o el arreglo del auto. Lo ideal es tener el equivalente a 3 meses de tus gastos del día a día, en un lugar de donde puedas sacarlo en el momento (una cuenta de ahorros, por ejemplo).
glossario-reserva-equilibrado = Reserva para imprevistos, de 3 a 6 meses de los gastos esenciales, invertida con liquidez diaria y bajo riesgo (Tesouro Selic, CDB con liquidez diaria).
glossario-reserva-busca = fondo de emergencia|reserva de emergencia|^reserva\b

glossario-sobra-termo = Lo que sobró
glossario-sobra-simples = Es lo que entró en el mes menos lo que salió. Si da negativo, salió más de lo que entró: vale la pena ver en qué categoría se puede recortar un poco.
glossario-sobra-busca = \bsobra\b|lo que sobro|saldo del mes

glossario-essenciais-termo = Gastos esenciales
glossario-essenciais-simples = Los gastos que no se pueden recortar, como el alquiler, el supermercado, la luz, el agua y el transporte. El fondo de emergencia se calcula a partir de ellos.
glossario-essenciais-busca = esencial

glossario-taxa_poupanca-termo = Tasa de ahorro
glossario-taxa_poupanca-simples = De todo lo que entró en el mes, cuánto quedó guardado. Si entraron 1.000 y sobraron 100, la tasa es del 10%.
glossario-taxa_poupanca-equilibrado = Parte de los ingresos del período que no se gastó: (ingresos − gastos) ÷ ingresos. Los aportes y rescates de inversiones no cuentan como gasto.
glossario-taxa_poupanca-busca = tasa de ahorro

glossario-orcamento-termo = Presupuesto
glossario-orcamento-simples = Un plan de cuánto piensas gastar en cada cosa en el mes. Ayuda a darte cuenta a tiempo cuando un gasto se está pasando de lo acordado.
glossario-orcamento-busca = ^presupuesto\b

glossario-patrimonio_liquido-termo = Patrimonio neto
glossario-patrimonio_liquido-simples = Todo lo que tienes (dinero en las cuentas, inversiones y bienes) menos todo lo que debes. Es la "foto" de cuánto tiene la familia de verdad.
glossario-patrimonio_liquido-equilibrado = Activos (cuentas, inversiones a valor de mercado y bienes) menos pasivos (saldo deudor de las deudas y resúmenes de tarjeta abiertos).
glossario-patrimonio_liquido-busca = patrimonio neto

glossario-competencia-termo = Devengo
glossario-competencia-simples = Cuenta el gasto en el mes en que se hizo, aunque el resumen de la tarjeta se pague después. Muestra cuánto gastaste de verdad en cada mes.
glossario-competencia-equilibrado = Criterio de devengo: cada movimiento cuenta en el mes en que ocurrió (fecha de la compra), no en el mes en que el dinero salió de la cuenta (criterio de caja).
glossario-competencia-busca = devengo

glossario-juros_compostos-termo = Interés compuesto
glossario-juros_compostos-simples = Intereses que también generan intereses. En una deuda, hacen que el monto crezca rápido (por eso el saldo impago de la tarjeta pesa tanto); en una inversión, hacen que el dinero crezca más con el tiempo.
glossario-juros_compostos-busca = interes(es)? compuestos?

glossario-cdi-termo = CDI
glossario-cdi-simples = Una tasa de referencia de los bancos de Brasil, que se mueve junto con la Selic. Cuando una inversión rinde "100% del CDI", sigue esa tasa.
glossario-cdi-equilibrado = Certificado de Depósito Interbancário: tasa promedio de los préstamos de un día entre bancos, muy cercana a la Selic. Es el principal índice de la renta fija con tasa variable en Brasil.
glossario-cdi-busca = \bcdi\b

glossario-selic-termo = Selic
glossario-selic-simples = La tasa básica de interés de Brasil, definida por el Banco Central. Cuando sube, los préstamos se encarecen y la renta fija rinde más.
glossario-selic-equilibrado = Tasa básica de interés definida por el Copom (el comité de política monetaria del Banco Central) cada 45 días. Es el piso de la renta fija y una herramienta contra la inflación.
glossario-selic-busca = \bselic\b

glossario-ipca-termo = Inflación (IPCA)
glossario-ipca-simples = Cuánto subieron los precios. Si la inflación fue del 5% en el año, lo que costaba 100 pasó a costar unos 105.
glossario-ipca-equilibrado = IPCA: índice oficial de inflación de Brasil, publicado por el IBGE. La app lo usa para mostrar ganancias y saldos "reales", descontada la pérdida de poder de compra.
glossario-ipca-busca = \bipca\b|inflacion

glossario-xirr-termo = XIRR (rentabilidad anual)
glossario-xirr-simples = La rentabilidad de la cartera por año, teniendo en cuenta cuándo entró o salió cada monto. Es como compararla con "cuánto rindió al año" una inversión.
glossario-xirr-equilibrado = Tasa interna de retorno con fechas irregulares: la tasa anual que hace cero el valor presente de todos los aportes, rescates, dividendos y del valor actual. No se distorsiona por el momento de los aportes.
glossario-xirr-busca = \bxirr\b|\btir\b

glossario-real-termo = Valor real
glossario-real-simples = El resultado ya descontada la inflación. Si el dinero rindió 10% y los precios subieron 5%, la ganancia real fue de un 5%: es lo que de verdad aumentó lo que puedes comprar.
glossario-real-equilibrado = Resultado deflactado por el índice elegido (IPCA por defecto): (1 + nominal) ÷ (1 + inflación) − 1.
glossario-real-busca = ^(?!real$)(?!.*frente a real).*\breal\b

glossario-nominal-termo = Valor nominal
glossario-nominal-simples = El resultado "de etiqueta", sin descontar la inflación. Parece mayor que la ganancia de verdad cuando los precios también subieron.
glossario-nominal-busca = \bnominal\b

glossario-rentabilidade-termo = Rentabilidad
glossario-rentabilidade-simples = Cuánto rindió el dinero invertido, en porcentaje de lo que se invirtió.
glossario-rentabilidade-equilibrado = Retorno sobre el capital invertido en el período, incluyendo la variación de precio y los dividendos recibidos.
glossario-rentabilidade-busca = rentabilidad|^rendimiento\b

glossario-valor_mercado-termo = Valor de mercado
glossario-valor_mercado-simples = Cuánto valen hoy tus inversiones, al precio de ahora. Si vendieras todo ahora, recibirías más o menos eso (antes de impuestos).
glossario-valor_mercado-equilibrado = Cantidad × última cotización (o precio de la curva, en la renta fija), convertida a la moneda de la familia. Sin cotización, la app usa el costo y te avisa.
glossario-valor_mercado-busca = valor de mercado

glossario-nao_realizado-termo = Resultado no realizado
glossario-nao_realizado-simples = La ganancia (o la pérdida) que solo existe "en el papel": la inversión subió, pero todavía no vendiste. Puede cambiar hasta la venta.
glossario-nao_realizado-busca = no realizad[oa]

glossario-realizado-termo = Resultado realizado
glossario-realizado-simples = La ganancia (o la pérdida) que ya se convirtió en dinero de verdad, porque vendiste.
glossario-realizado-busca = ^realizad[oa]\b

glossario-preco_medio-termo = Precio promedio
glossario-preco_medio-simples = Cuánto pagaste, en promedio, por cada cuota o acción, sumando todas las compras. Con él se calcula la ganancia en la venta.
glossario-preco_medio-equilibrado = Costo total de las compras (con comisiones) dividido por la cantidad en cartera; las ventas no cambian el precio promedio. Es la base del impuesto sobre la ganancia de capital.
glossario-preco_medio-busca = precio (medio|promedio)|costo promedio|^pm\b

glossario-custo-termo = Costo
glossario-custo-simples = Cuánto pagaste por las inversiones que todavía tienes, sumando las compras.
glossario-custo-busca = ^costo\b

glossario-provento-termo = Dividendos y rendimientos
glossario-provento-simples = Dinero que una inversión te paga solo por tenerla: dividendos de acciones, rendimientos de fondos inmobiliarios e intereses de bonos.
glossario-provento-equilibrado = Distribuciones al inversor: dividendos, JCP (intereses sobre el capital propio), rendimientos de fondos inmobiliarios y amortizaciones. Entran en el resultado total y en el XIRR.
glossario-provento-busca = proventos?|distribuciones|renta pasiva|ingresos? pasivos?|dividendo

glossario-dividendo-termo = Dividendos
glossario-dividendo-simples = La parte de la ganancia que una empresa reparte con quienes tienen sus acciones. Te llega a la cuenta.
glossario-dividendo-busca = dividendos? (en efectivo|de acciones)

glossario-jcp-termo = Intereses sobre el capital propio (JCP)
glossario-jcp-simples = Otra forma en que una empresa de Brasil reparte la ganancia con los accionistas, parecida a los dividendos, pero con impuesto retenido en la fuente (17,5% desde 2026; 15% hasta 2025).
glossario-jcp-busca = \bjcp\b|intereses? sobre (el )?capital

glossario-yield-termo = Yield (rendimiento)
glossario-yield-simples = Cuánto pagó una inversión en dividendos y rendimientos en un año, en porcentaje del precio. Un yield del 8% quiere decir que, por cada 100, pagó unos 8 en el año.
glossario-yield-busca = \byield\b|\bdy\b

glossario-come_cotas-termo = Come-cotas
glossario-come_cotas-simples = Un impuesto que algunos fondos de Brasil cobran dos veces al año (mayo y noviembre) reduciendo la cantidad de cuotas, aunque no hagas ningún rescate.
glossario-come_cotas-equilibrado = Anticipo semestral del impuesto sobre la renta en fondos abiertos (15% o 20%), cobrado mediante la reducción de cuotas. Los fondos de acciones y los fondos inmobiliarios (FII) no lo tienen.
glossario-come_cotas-busca = come.cotas

glossario-alocacao-termo = Asignación
glossario-alocacao-simples = Cómo está repartido tu dinero invertido entre tipos de inversión (renta fija, acciones, fondos inmobiliarios...). Repartirlo ayuda a no depender de una sola cosa.
glossario-alocacao-busca = asignacion|alocacion

glossario-rebalanceamento-termo = Rebalanceo
glossario-rebalanceamento-simples = Ajustar la cartera para volver al reparto que planeaste, poniendo el próximo dinero en lo que quedó por debajo del objetivo.
glossario-rebalanceamento-busca = rebalance

glossario-desvio-termo = Desvío del objetivo
glossario-desvio-simples = Cuánto está cada tipo de inversión por encima o por debajo de la parte que planeaste para él, en puntos porcentuales.
glossario-desvio-busca = desvio|desviacion

glossario-liquidez-termo = Liquidez
glossario-liquidez-simples = La facilidad para convertir una inversión en dinero en la cuenta. El fondo de emergencia necesita liquidez: se puede sacar el mismo día.
glossario-liquidez-busca = liquidez

glossario-renda_fixa-termo = Renta fija
glossario-renda_fixa-simples = Inversiones en las que prestas dinero (al gobierno, a un banco o a una empresa) y recibes intereses acordados. La cuenta de ahorros, el Tesouro Direto y el CDB son renta fija.
glossario-renda_fixa-busca = renta fija

glossario-renda_variavel-termo = Renta variable
glossario-renda_variavel-simples = Inversiones cuyo valor sube y baja todos los días, como las acciones y los fondos inmobiliarios. Pueden rendir más, pero también pueden caer.
glossario-renda_variavel-busca = renta variable

glossario-fii-termo = Fondos inmobiliarios (FII)
glossario-fii-simples = Fondos que invierten en inmuebles (centros comerciales, galpones, oficinas) o en deudas ligadas a inmuebles. Pagan rendimientos todos los meses, normalmente sin impuesto para personas físicas en Brasil.
glossario-fii-busca = \bfiis?\b|fondos? inmobiliarios?|\bfibras?\b

glossario-etf-termo = ETF
glossario-etf-simples = Un fondo que se negocia en la bolsa como si fuera una acción y que copia un índice (como el Ibovespa). Con una sola compra, tienes un pedazo de varias empresas.
glossario-etf-busca = \betfs?\b

glossario-acoes-termo = Acciones
glossario-acoes-simples = Pedacitos de una empresa. Quien tiene acciones se vuelve socio: gana si la empresa se valoriza y recibe parte de la ganancia (dividendos).
glossario-acoes-busca = ^acciones\b

glossario-tesouro-termo = Tesouro Direto
glossario-tesouro-simples = Bonos del gobierno federal de Brasil que cualquier persona puede comprar desde un monto pequeño. Es una de las inversiones más seguras del país.
glossario-tesouro-equilibrado = Bonos públicos federales (Selic, Prefixado, IPCA+) negociados a través de la B3. Antes del vencimiento, el precio sigue la valuación a mercado.
glossario-tesouro-busca = tesouro

glossario-cdb-termo = CDB
glossario-cdb-simples = Un préstamo que le haces a un banco de Brasil a cambio de intereses. Tiene la garantía del FGC (el fondo de garantía de depósitos) hasta 250.000 reales por banco.
glossario-cdb-busca = \bcdbs?\b
glossario-rdb-termo = RDB
glossario-rdb-simples = Como el CDB, un préstamo que le haces a un banco de Brasil a cambio de intereses (Nubank e Inter usan este nombre). Tiene la garantía del FGC hasta 250.000 reales por banco y el impuesto regresivo.
glossario-rdb-busca = \brdbs?\b

glossario-lci_lca-termo = LCI y LCA
glossario-lci_lca-simples = Parecidas al CDB, pero el banco usa el dinero para inmuebles (LCI) o para el agronegocio (LCA). Para personas físicas, no pagan impuesto sobre la renta.
glossario-lci_lca-busca = \blci\b|\blca\b

glossario-previdencia-termo = Plan de pensión privado
glossario-previdencia-simples = Una inversión de largo plazo pensada para la jubilación. El PGBL puede reducir el impuesto de quien declara en el modelo completo; el VGBL sirve para los demás casos.
glossario-previdencia-busca = planes? de pension|pension privada|prevision privada|\bpgbl\b|\bvgbl\b

glossario-aporte-termo = Aporte
glossario-aporte-simples = Dinero que pones en una inversión. No es un gasto: el dinero sigue siendo tuyo, solo cambió de lugar.
glossario-aporte-busca = aporte

glossario-resgate-termo = Rescate
glossario-resgate-simples = Sacar dinero de una inversión de vuelta a la cuenta. Tampoco es un ingreso nuevo: es tu dinero que vuelve.
glossario-resgate-busca = rescate

glossario-indexador-termo = Indexador
glossario-indexador-simples = La tasa que sirve de referencia para los intereses de una inversión o deuda (por ejemplo, CDI, IPCA o una tasa fija).
glossario-indexador-busca = indexador|indice de referencia|^indice$

glossario-fatura-termo = Resumen de la tarjeta
glossario-fatura-simples = La cuenta de la tarjeta de crédito: suma lo que compraste en el mes y vence en una fecha fija. Pagar solo el mínimo hace que la deuda entre en el crédito rotativo, que tiene intereses altísimos.
glossario-fatura-busca = resumen(es)? de (la )?tarjeta|^resumen(es)?$|^resumen y saldo|\bfacturas?\b|estados? de cuenta

glossario-rotativo-termo = Crédito rotativo de la tarjeta
glossario-rotativo-simples = Lo que pasa cuando el resumen no se paga completo: el resto se convierte en una deuda con intereses entre los más altos del país. Conviene evitarlo siempre que se pueda.
glossario-rotativo-busca = rotativo

glossario-cet-termo = Costo efectivo total (CET)
glossario-cet-simples = Cuánto cuesta de verdad un préstamo por año, sumando intereses, comisiones, seguros e impuestos. Es el número para comparar ofertas.
glossario-cet-busca = \bcet\b|costo efectivo

glossario-amortizacao-termo = Amortización
glossario-amortizacao-simples = La parte de la cuota que de verdad reduce la deuda (el resto de la cuota son los intereses). Adelantar amortización reduce los intereses que todavía pagarías.
glossario-amortizacao-busca = amortiza

glossario-price_sac-termo = Sistema Price y SAC
glossario-price_sac-simples = Dos formas de calcular las cuotas de un financiamiento. En el Price, las cuotas son iguales; en el SAC, empiezan más altas y van bajando, y en total suele salir más barato.
glossario-price_sac-busca = \bprice\b|\bsac\b

glossario-divida_cara-termo = Deuda cara
glossario-divida_cara-simples = Deuda con intereses altos, como el crédito rotativo de la tarjeta, el sobregiro y algunos préstamos personales. Suele convenir pagarla primero.
glossario-divida_cara-busca = deudas? caras?|deudas? costosas?

glossario-saldo_devedor-termo = Saldo deudor
glossario-saldo_devedor-simples = Cuánto falta pagar hoy de una deuda, sin contar los intereses futuros.
glossario-saldo_devedor-busca = saldo (deudor|adeudado)

glossario-conciliado-termo = Conciliado
glossario-conciliado-simples = Un movimiento que ya revisaste contra el extracto del banco. Sirve para estar seguro de que el saldo de la app coincide con el del banco.
glossario-conciliado-busca = concilia

glossario-previsto-termo = Previsto
glossario-previsto-simples = Un movimiento que todavía va a ocurrir (una cuenta programada, un sueldo que va a llegar). Entra en las proyecciones, pero no cambia el saldo de hoy.
glossario-previsto-busca = previst

glossario-recorrencia-termo = Recurrencia
glossario-recorrencia-simples = Un gasto o un ingreso que se repite (el alquiler cada mes, una suscripción, el sueldo). La app crea los próximos sola para que no te olvides.
glossario-recorrencia-busca = recurren

glossario-transferencia-termo = Transferencia entre cuentas
glossario-transferencia-simples = Dinero que pasó de una cuenta tuya a otra cuenta tuya. No es gasto ni ingreso, por eso no entra en los totales del mes.
glossario-transferencia-busca = transferencia

glossario-titular-termo = Miembro de la familia
glossario-titular-simples = La persona de la familia dueña de la cuenta, la tarjeta, el bien o la deuda. Lo que entra y sale en una cuenta de un miembro queda con él; lo que es de todos queda de la familia.
glossario-titular-busca = titular

glossario-vencimento-termo = Vencimiento
glossario-vencimento-simples = La fecha límite para pagar una cuenta sin recargo (o, en una inversión, la fecha en que termina y el dinero vuelve a ti).
glossario-vencimento-busca = vencimiento

glossario-cambio-termo = Tipo de cambio
glossario-cambio-simples = El precio de una moneda en tu propia moneda (por ejemplo, cuántos reales cuesta un dólar). La app lo usa para sumar cuentas e inversiones en otras monedas.
glossario-cambio-busca = tipos? de cambio|operacion(es)? de cambio|cambio de moneda

glossario-deflator-termo = Deflactor
glossario-deflator-simples = El índice de inflación que se usa para descontar la pérdida de poder de compra de los valores (la app usa el IPCA, si no eliges otro).
glossario-deflator-busca = deflact|deflator

glossario-m2-termo = M2 (oferta monetaria)
glossario-m2-simples = Una medida de cuánto dinero hay en la economía. Cuando crece mucho más rápido que la producción, tiende a empujar los precios hacia arriba.
glossario-m2-busca = \bm2\b

glossario-score_saude-termo = Score de salud
glossario-score_saude-simples = Una nota de 0 a 100 que resume si una inversión está bien (ganancias, deudas, precio, historial), con la explicación de cada parte. No es una recomendación de compra o venta.
glossario-score_saude-busca = score de salud|salud de(l| los) activos?

glossario-volatilidade-termo = Volatilidad
glossario-volatilidade-simples = Cuánto sube y baja el precio de una inversión. Cuanto mayor, más "montaña rusa" es.
glossario-volatilidade-busca = volatil

glossario-drawdown-termo = Caída máxima (drawdown)
glossario-drawdown-simples = La mayor caída que tuvo la inversión, del punto más alto al más bajo, en el período. Muestra el peor momento que pasó quien invirtió.
glossario-drawdown-busca = drawdown|caida maxima|peor caida

glossario-aliquota-termo = Alícuota
glossario-aliquota-simples = El porcentaje de impuesto que se cobra sobre un valor (por ejemplo, 15% sobre la ganancia).
glossario-aliquota-busca = alicuota|tasa (impositiva|efectiva)|tipo impositivo

glossario-darf-termo = DARF
glossario-darf-simples = La boleta de la Receita Federal (el fisco de Brasil) para pagar impuestos, como el impuesto sobre la ganancia de acciones vendidas en el mes.
glossario-darf-busca = \bdarf\b

glossario-ir-termo = Impuesto sobre la renta
glossario-ir-simples = Impuesto federal sobre lo que ganas (sueldo, alquiler, ganancias de inversiones). Una parte ya se descuenta en la fuente; la declaración anual ajusta la cuenta.
glossario-ir-busca = impuesto (sobre|a) la renta|\birpf\b|\bir\b(?! al?\b)|impuesto retenido
