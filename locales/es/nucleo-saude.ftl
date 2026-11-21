# Salud financiera (núcleo, `health`): los modelos de la nota, las franjas, las dimensiones, los
# indicadores (meta, explicación, cálculo y sugerencias), el resumen y los avisos
# (docs/health.md). Se arman en el momento, en el idioma de la instalación.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Modelos (Ajustes › Salud financiera)

nucleo-saude-modelo-isf = I-SFB (Febraban y Banco Central)
nucleo-saude-modelo-isf-descricao = Inspirado en el Índice de Salud Financiera del Brasileño: nota de 0 a 100 en cuatro dimensiones (comportamiento, habilidad, libertad y seguridad), comparada con el promedio nacional de 56,7 en 2024. Es una estimación a partir de tus datos, no el cuestionario oficial.
nucleo-saude-modelo-classicos = Indicadores clásicos
nucleo-saude-modelo-classicos-descricao = Las razones de la educación financiera, cada una con su meta: ahorrar el 20% de los ingresos, fondo de emergencia de 6 meses de gastos esenciales, cuotas hasta el 15% de los ingresos, vivienda hasta el 30%, nada de crédito rotativo y patrimonio en crecimiento.
nucleo-saude-modelo-desativado = Desactivado
nucleo-saude-modelo-desativado-descricao = Sin análisis de salud financiera: la tarjeta desaparece del panel y la sección de los informes queda vacía.
nucleo-saude-desativada = Análisis de salud financiera desactivado en Ajustes.
nucleo-saude-erro-mes-futuro = la salud financiera solo se puede evaluar hasta el mes actual
nucleo-saude-erro-so-interface = el modelo de salud financiera solo se cambia desde la app

## Valores medidos y períodos

nucleo-saude-valor-mes = { $n } mes
nucleo-saude-valor-meses = { $n } meses
nucleo-saude-valor-de-meses = { $n } de { $total } meses
nucleo-saude-valor-em-dia = { $n } de { $total } al día
nucleo-saude-nenhum-mes = ningún mes
nucleo-saude-n-meses = { $n ->
        [one] 1 mes
       *[other] { $n } meses
    }
nucleo-saude-sem-meses = sin meses
nucleo-saude-base-competencia = devengo
nucleo-saude-base-caixa = caja

## Valores que entran en el cálculo

nucleo-saude-quantia-receitas = Ingresos
nucleo-saude-quantia-despesas = Gastos
nucleo-saude-quantia-sobra = Lo que sobra
nucleo-saude-quantia-reserva = Fondo de emergencia
nucleo-saude-quantia-gasto-essencial = Gasto esencial por mes
nucleo-saude-quantia-parcelas = Cuotas
nucleo-saude-quantia-moradia = Vivienda
nucleo-saude-quantia-patrimonio-em = Patrimonio en { $mes }
nucleo-saude-quantia-variacao = Variación

## Sin datos (el motivo)

nucleo-saude-sem-dados-meses = Sin datos: se necesitan al menos { $minimo } meses completos de movimientos ({ $n ->
        [one] { $n } analizado
       *[other] { $n } analizados
    }).
nucleo-saude-sem-dados-receita = Sin datos: no hay ingresos registrados en { $periodo }.
nucleo-saude-sem-dados-gastos = Sin datos: se necesitan al menos { $minimo } meses completos de gastos para el promedio ({ $n ->
        [one] { $n } analizado
       *[other] { $n } analizados
    }).
nucleo-saude-sem-dados-essenciais = Sin datos: no hay gastos en categorías esenciales en los últimos meses. Marca las categorías esenciales en Ajustes › Categorías.
nucleo-saude-sem-dados-moradia = Sin datos: no hay categoría de gasto "Vivienda" ni crédito hipotecario.
nucleo-saude-sem-dados-credito = Sin datos: no hay cuenta corriente ni tarjeta con pago de resumen registrado.
nucleo-saude-sem-dados-patrimonio = Sin datos: se necesitan al menos { $minimo } meses de patrimonio ({ $n ->
        [one] { $n } encontrado
       *[other] { $n } encontrados
    }).
nucleo-saude-sem-dados-parcelas = Sin datos: ninguna cuota de préstamo ni resumen de tarjeta con pago registrado venció en el período.
nucleo-saude-sem-dados-orcamento = Sin datos: ningún mes de { $periodo } tiene presupuesto de gastos.
nucleo-saude-sem-dados-movimento = Sin datos: no hubo movimientos en el período.

## Cómo se calculó

nucleo-saude-conta-poupanca = (ingresos − gastos) ÷ ingresos de { $periodo } ({ $n } meses, base { $base }); las transferencias, los pagos de resumen y los aportes no cuentan.
nucleo-saude-conta-regular = Meses con tasa de ahorro del 10% o más ÷ meses con ingresos, en { $periodo }.
nucleo-saude-conta-reserva = Fondo de emergencia ({ $fonte }) ÷ promedio mensual de los gastos en categorías esenciales de los últimos { $n } meses completos.
nucleo-saude-fonte-meta = el valor actual de la meta de fondo de emergencia (cuentas y posiciones vinculadas a ella)
nucleo-saude-fonte-saldos = el saldo de las cuentas corriente, de ahorro y efectivo
nucleo-saude-conta-comprometimento = Cuotas con vencimiento en { $periodo } ÷ ingresos del período; { $contratos }. Las compras en cuotas con tarjeta cuentan como gasto, no aquí.
nucleo-saude-contratos-nenhum = ningún préstamo ni financiamiento con cuota en el período
nucleo-saude-contratos = cuotas de { $n ->
        [one] { $n } contrato
       *[other] { $n } contratos
    } (préstamos y financiamientos)
nucleo-saude-conta-moradia = Gastos en Vivienda (alquiler, expensas, cuentas de la casa…) y cuotas del crédito hipotecario registradas fuera de ella ÷ ingresos de { $periodo }.
nucleo-saude-credito-cheque = sobregiro en { $meses }
nucleo-saude-credito-rotativo = resumen no pagado completo en { $meses }
nucleo-saude-credito-nenhum = ningún uso
nucleo-saude-conta-credito = Meses de { $periodo } con saldo negativo en cuenta corriente (sobregiro) o resumen de tarjeta no pagado completo hasta { $dias } días después del vencimiento (rotativo): { $detalhe }.
nucleo-saude-conta-credito-sem-cartoes = Las tarjetas sin pago de resumen registrado quedan fuera.
nucleo-saude-base-patrimonio-inicial = el patrimonio inicial
nucleo-saude-base-despesas = los gastos del período (el patrimonio inicial es pequeño)
nucleo-saude-conta-patrimonio = Variación del patrimonio neto (cuentas, cartera, bienes y deudas) de { $de } a { $ate } ÷ { $base }. Valor nominal, sin descontar la inflación.
nucleo-saude-conta-em-dia = Cuotas de préstamos ({ $parcelas_em_dia } al día, { $parcelas_atraso } atrasadas) y resúmenes de tarjeta ({ $faturas_pagas } pagados completos, { $faturas_nao } no) vencidos en { $periodo }, con { $dias } días de margen. Una cuota no conciliada después del vencimiento cuenta como atrasada.
nucleo-saude-conta-orcamento = Meses con gastos hasta el total previsto ÷ meses con presupuesto en { $periodo } (base { $base }).
nucleo-saude-conta-azul = Meses en que los ingresos cubrieron los gastos ÷ meses con movimientos, en { $periodo } (base { $base }).

## Indicadores: nombre, meta, qué mide, sugerencias y qué hacer sin datos

nucleo-saude-sem-dados-3-meses = Registra o importa los ingresos y gastos de al menos 3 meses completos.

nucleo-saude-poupanca = Tasa de ahorro
nucleo-saude-poupanca-meta = ≥ 20% de los ingresos
nucleo-saude-poupanca-explicacao = Cuánto de los ingresos sobra después de los gastos, en los últimos 12 meses.
nucleo-saude-poupanca-dica-1 = Págate primero: programa una transferencia al ahorro o a la corredora el día del sueldo.
nucleo-saude-poupanca-dica-2 = Revisa las tres categorías de mayor gasto en Informes › Composición y ponles límites en el presupuesto.
nucleo-saude-poupanca-dica-3 = Cancela suscripciones y cuentas fijas que no usas (Informes › Recurrentes).

nucleo-saude-regular = Ahorro regular
nucleo-saude-regular-meta = 10% o más todos los meses
nucleo-saude-regular-explicacao = En cuántos meses fue posible ahorrar al menos el 10% de los ingresos.
nucleo-saude-regular-dica-1 = Automatiza un monto fijo de ahorro todos los meses, aunque sea pequeño.
nucleo-saude-regular-dica-2 = Separa a lo largo del año dinero para los gastos de temporada (IPVA, IPTU, útiles escolares) para que no se coman lo que sobra.
nucleo-saude-regular-dica-3 = En los meses más flojos, mira en Informes › Comparación qué cambió respecto de un mes bueno.

nucleo-saude-reserva = Fondo de emergencia
nucleo-saude-reserva-meta = ≥ 6 meses de gastos esenciales
nucleo-saude-reserva-explicacao = Durante cuántos meses el fondo de emergencia cubre los gastos esenciales sin ningún ingreso.
nucleo-saude-reserva-dica-1 = Define (o refuerza) la meta de fondo de emergencia en 05 · Presupuesto › Metas, con un objetivo de 6 meses de gastos esenciales.
# Lo que va entre paréntesis es igual a `nucleo-pais-reserva-br` (se cambia por los productos del país).
nucleo-saude-reserva-dica-2 = Guarda el fondo de emergencia en una inversión de liquidez diaria y bajo riesgo (Tesouro Selic o CDB con liquidez diaria).
nucleo-saude-reserva-dica-3 = Destina al fondo de emergencia ingresos extra como el 13º, la devolución del impuesto sobre la renta y la PLR hasta llegar al objetivo.
nucleo-saude-reserva-sem-dados = Marca las categorías esenciales en Ajustes › Categorías y registra al menos 3 meses de gastos.

nucleo-saude-comprometimento = Ingresos comprometidos con cuotas
nucleo-saude-comprometimento-meta = ≤ 15% de los ingresos
nucleo-saude-comprometimento-explicacao = Parte de los ingresos que va a cuotas de préstamos y financiamientos.
nucleo-saude-comprometimento-dica-1 = Evita cuotas nuevas hasta que lo comprometido quede por debajo del 15% de los ingresos.
nucleo-saude-comprometimento-dica-2 = Simula la portabilidad o un pago anticipado del contrato más caro en 07 · Deudas.
nucleo-saude-comprometimento-dica-3 = Paga primero las deudas con los intereses más altos (tarjeta, sobregiro, préstamo personal).
nucleo-saude-comprometimento-sem-dados = Registra los ingresos de al menos 3 meses completos y carga los contratos en 07 · Deudas.

nucleo-saude-moradia = Costo de la vivienda
nucleo-saude-moradia-meta = ≤ 30% de los ingresos
nucleo-saude-moradia-explicacao = Parte de los ingresos que va a la vivienda: alquiler o crédito hipotecario, expensas y cuentas de la casa.
nucleo-saude-moradia-dica-1 = Una vivienda por encima del 30% de los ingresos aprieta el resto del presupuesto: revisa servicios, expensas y seguros.
nucleo-saude-moradia-dica-2 = Al renovar el alquiler o en un crédito nuevo, apunta a un costo de hasta el 30% de los ingresos.
# El fragmento "amortizar con el FGTS…" es igual a `nucleo-pais-fgts-br`.
nucleo-saude-moradia-dica-3 = Si el crédito pesa, simula amortizar con el FGTS o pasarlo a una tasa menor en 07 · Deudas.
nucleo-saude-moradia-sem-dados = Registra los gastos de la casa en la categoría Vivienda (o carga el crédito hipotecario en 07 · Deudas).

nucleo-saude-credito = Rotativo y sobregiro
nucleo-saude-credito-meta = ningún mes
nucleo-saude-credito-explicacao = Meses en los que se usó crédito caro: sobregiro o el rotativo de la tarjeta.
nucleo-saude-credito-dica-1 = Paga el resumen de la tarjeta completo: el rotativo cobra los intereses más altos del mercado.
# Lo que va entre paréntesis es igual a `nucleo-pais-consignado-br`.
nucleo-saude-credito-dica-2 = Si te falta dinero, cambia el rotativo o el sobregiro por un crédito más barato (un préstamo con descuento por nómina, por ejemplo).
nucleo-saude-credito-dica-3 = Carga el límite y las tasas del sobregiro en la cuenta, en 03 · Cuentas, para ver el costo estimado.
nucleo-saude-credito-sem-dados = Registra los pagos de resumen de la tarjeta y los movimientos de la cuenta corriente.

nucleo-saude-patrimonio = Evolución del patrimonio
nucleo-saude-patrimonio-meta = crecer ≥ 5% en 12 meses
nucleo-saude-patrimonio-explicacao = Cuánto creció (o cayó) el patrimonio neto en los últimos 12 meses.
nucleo-saude-patrimonio-dica-1 = Destina parte de lo que sobra cada mes a inversiones o a amortizar deudas.
nucleo-saude-patrimonio-dica-2 = Actualiza el valor de los bienes y las cotizaciones de la cartera para que el patrimonio refleje la realidad.
nucleo-saude-patrimonio-dica-3 = Revisa los gastos grandes del año en Informes › Tendencia: explican la caída.
nucleo-saude-patrimonio-sem-dados = Carga las cuentas con su saldo inicial y mantén algunos meses de movimientos.

nucleo-saude-em-dia = Cuentas al día
nucleo-saude-em-dia-meta = todas pagadas antes del vencimiento
nucleo-saude-em-dia-explicacao = Cuotas de préstamos y resúmenes de tarjeta pagados completos hasta el vencimiento.
nucleo-saude-em-dia-dica-1 = Activa el débito automático de las cuotas y del resumen de la tarjeta.
nucleo-saude-em-dia-dica-2 = Concilia las cuotas en 07 · Deudas: las pagadas y no conciliadas cuentan como atrasadas.
nucleo-saude-em-dia-dica-3 = Usa los Próximos vencimientos del panel para organizarte.
nucleo-saude-em-dia-sem-dados = Carga los contratos en 07 · Deudas y registra los pagos de resumen de la tarjeta.

nucleo-saude-orcamento = Gastos dentro del presupuesto
nucleo-saude-meta-todos-os-meses = todos los meses
nucleo-saude-orcamento-explicacao = En cuántos meses los gastos quedaron dentro del total previsto en el presupuesto.
nucleo-saude-orcamento-dica-1 = Revisa el presupuesto de los meses en que te pasaste y ajusta los límites a la realidad.
nucleo-saude-orcamento-dica-2 = Sigue el ritmo del gasto en la tarjeta de presupuesto del panel a lo largo del mes.
nucleo-saude-orcamento-dica-3 = Deja una línea para gastos imprevistos en el presupuesto.
nucleo-saude-orcamento-sem-dados = Arma un presupuesto mensual en 05 · Presupuesto (copia el del mes anterior o usa la sugerencia).

nucleo-saude-azul = Meses en positivo
nucleo-saude-azul-explicacao = En cuántos meses los ingresos cubrieron los gastos.
nucleo-saude-azul-dica-1 = En los meses en rojo, mira en Informes › Comparación qué cambió respecto de un mes en positivo.
nucleo-saude-azul-dica-2 = Mantén un colchón para gastos irregulares, para que un mes malo no se convierta en deuda.
nucleo-saude-azul-dica-3 = Planifica los gastos anuales (seguros, impuestos, viajes) como cuotas mensuales en el presupuesto.

## I-SFB: franjas y dimensiones

nucleo-saude-faixa-ruim = Mala
nucleo-saude-faixa-muito-baixa = Muy baja
nucleo-saude-faixa-baixa = Baja
nucleo-saude-faixa-ok = Ok
nucleo-saude-faixa-boa = Buena
nucleo-saude-faixa-muito-boa = Muy buena
nucleo-saude-faixa-otima = Excelente

nucleo-saude-dimensao-comportamento = Comportamiento
nucleo-saude-dimensao-comportamento-descricao = Pagar las cuentas al día, gastar dentro del presupuesto y cerrar los meses en positivo.
nucleo-saude-dimensao-habilidade = Habilidad
nucleo-saude-dimensao-habilidade-descricao = Lograr ahorrar parte de los ingresos, con regularidad.
nucleo-saude-dimensao-liberdade = Libertad
nucleo-saude-dimensao-liberdade-descricao = Cuánto de los ingresos está libre de cuotas y de crédito caro (rotativo y sobregiro).
nucleo-saude-dimensao-seguranca = Seguridad
nucleo-saude-dimensao-seguranca-descricao = Un fondo para imprevistos y un patrimonio que crece.

## Indicadores clásicos: franjas

nucleo-saude-faixa-critica = Crítica
nucleo-saude-faixa-fragil = Frágil
nucleo-saude-faixa-estavel = Estable
nucleo-saude-faixa-saudavel = Saludable

## Resumen y avisos

nucleo-saude-resumo-acima = Nota { $nota } ({ $faixa }), por encima del promedio nacional de { $media }.
nucleo-saude-resumo-abaixo = Nota { $nota } ({ $faixa }), por debajo del promedio nacional de { $media }.
nucleo-saude-resumo-igual = Nota { $nota } ({ $faixa }), igual al promedio nacional de { $media }.
nucleo-saude-resumo-dimensao-fraca = Dimensión más débil: { $dimensao }.
nucleo-saude-resumo-nota = Nota { $nota } ({ $faixa }).
nucleo-saude-resumo-ponto-fraco = Punto más débil: { $indicador }.
nucleo-saude-resumo-tudo-na-meta = Todos los indicadores en la meta.
nucleo-saude-insuficiente-meses = Datos insuficientes: se necesitan al menos { $minimo } meses completos de movimientos (hay { $n }).
nucleo-saude-insuficiente-indicadores = Datos insuficientes: se pudieron medir pocos indicadores.
nucleo-saude-aviso-isf = Estimación a partir de tus datos (movimientos, cuotas, resúmenes y saldos), en el espíritu del I-SFB de Febraban y del Banco Central; el índice oficial sale de un cuestionario.
nucleo-saude-aviso-classicos = Indicadores clásicos de finanzas personales calculados con tus datos; metas habituales de la educación financiera.
nucleo-saude-aviso-pouco-historico = Con menos de { $minimo } meses de historial, los indicadores quedan sin datos en lugar de penalizar.
nucleo-saude-aviso-sem-cambio = Sin tipo de cambio para { $moedas }: esos valores quedaron fuera del cálculo.
nucleo-saude-aviso-contas-desatualizadas = Nota estimada: { $contas } sin movimientos recientes. El saldo puede estar incompleto, y su saldo negativo queda para revisar en vez de contar como sobregiro.
nucleo-saude-referencia = Promedio nacional del I-SFB en 2024
nucleo-saude-referencia-curta = Promedio nacional
nucleo-saude-credito-suspeito = posible sobregiro o importación incompleta en { $contas } (30 días o más en rojo sin el límite registrado): registre el sobregiro de la cuenta en 03 · Cuentas o importe el extracto completo
