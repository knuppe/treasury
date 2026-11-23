# Lo que Treasury percibe sin IA (núcleo, `insights`): inflación personal, comisiones e intereses,
# cuenta en rojo, gasto fuera de lo normal, compras pequeñas, gastos por encima de los ingresos y
# la división de los ingresos (docs/insights.md). Se arman en el momento, en el idioma de la
# instalación.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Grupos del IPCA

insights-grupo-ipca-g1 = Alimentos y bebidas
insights-grupo-ipca-g2 = Vivienda
insights-grupo-ipca-g3 = Artículos para el hogar
insights-grupo-ipca-g4 = Vestimenta
insights-grupo-ipca-g5 = Transporte
insights-grupo-ipca-g6 = Salud y cuidado personal
insights-grupo-ipca-g7 = Gastos personales
insights-grupo-ipca-g8 = Educación
insights-grupo-ipca-g9 = Comunicación

## Inflación personal

insights-inflacao-titulo = Tu inflación en 12 meses: { $pessoal }
insights-inflacao-titulo-ipca = Tu inflación en 12 meses: { $pessoal } (IPCA: { $ipca })
insights-inflacao-corpo = Con el peso que cada grupo tiene en tus gastos (y no en la canasta promedio del IBGE), los precios de lo que compras subieron { $pessoal } de { $de } a { $ate }.
insights-inflacao-mais-pesou = Lo que más pesó: { $grupo } ({ $peso } de los gastos, subió { $variacao }).
insights-inflacao-acao = Ajustes de ingresos y de presupuesto menores que esta inflación significan perder poder de compra.

## Comisiones e intereses

insights-tarifas-titulo = { $valor } en comisiones e intereses en 12 meses
insights-tarifas-corpo = De { $de } a { $ate }, { $tarifas } se fueron en comisiones bancarias ({ $n_tarifas } cobros) y { $juros } en intereses, IOF y cargos ({ $n_juros } cobros). Es dinero que no compra nada.
insights-tarifas-juros-caros = Los intereses del sobregiro y del crédito rotativo de la tarjeta están entre los más caros del mercado.
insights-tarifas-acao-pacote = Bueno saber: los bancos están obligados a ofrecer un paquete de servicios esenciales sin comisiones (Resolución CMN 3.919), y hay cuentas digitales gratuitas.
insights-tarifas-acao-credito = Bueno saber: cambiar el sobregiro o el rotativo por un crédito más barato y pagar el resumen completo.
insights-tarifas-acao-ambos = Bueno saber: los bancos están obligados a ofrecer un paquete de servicios esenciales sin comisiones (Resolución CMN 3.919), y hay cuentas digitales gratuitas; cambiar el sobregiro o el rotativo por un crédito más barato y pagar el resumen completo.

## Cuenta en rojo

insights-vermelho-titulo = { $conta } estuvo en rojo { $dias } días
insights-vermelho-corpo = En los últimos 90 días, el saldo de { $conta } fue negativo durante { $dias } días y llegó a { $minimo }.
insights-vermelho-taxa = El sobregiro cuesta en promedio { $taxa }% al mes en los bancos (Banco Central).
insights-vermelho-acao = Revisar la previsión de saldo y, si se repite, cambiar el sobregiro por un crédito más barato (Deudas › Tasas del mercado).
insights-fgts-titulo = El FGTS de { $mes } no se depositó ({ $conta })
insights-fgts-corpo = { $n ->
    [one] El extracto de la Caixa no muestra el depósito de { $mes }, y el plazo del empleador (día 20 del mes siguiente) ya pasó.
   *[other] El extracto de la Caixa no muestra { $n } depósitos, a partir de { $mes }, y el plazo del empleador (día 20 del mes siguiente) ya pasó.
}
insights-fgts-acao = Revisa la nómina y habla con RR. HH. de la empresa; si no se resuelve, la Caixa y el Ministerio de Trabajo reciben la denuncia.
insights-previdencia-titulo = { $plano }: aportes para revisar
insights-previdencia-pendencias =
    { $n ->
        [one] { $n } pendiente en el historial de pagos importado:
       *[other] { $n } pendientes en el historial de pagos importado:
    }
insights-previdencia-sem-cota =
    { $aportes ->
        [one] { $aportes } aporte espera la cuota de la CVM desde { $desde } y cuenta en el saldo por el importe aportado.
       *[other] { $aportes } aportes esperan la cuota de la CVM desde { $desde } y cuentan en el saldo por el importe aportado.
    }
insights-previdencia-acao = Abre el plan en Activos, revisa los pendientes y envía al agente el pago que faltó (o la captura que muestra ese tramo).

## Gasto fuera de lo normal

insights-acima-titulo = { $categoria } por encima de lo normal en { $mes }
insights-acima-corpo = En { $mes }, { $categoria } sumó { $valor }, { $diferenca } más que lo normal de los meses anteriores ({ $normal } por mes, la mediana de 6 meses).
insights-acima-acao = Abrir los movimientos de la categoría para ver qué cambió; si es un gasto que va a continuar, ajustar el presupuesto.
insights-abaixo-titulo = { $categoria } por debajo de lo normal en { $mes }
insights-abaixo-corpo = En { $mes }, { $categoria } sumó { $valor }, { $diferenca } menos que lo normal ({ $normal } por mes).
insights-abaixo-acao = Si el ahorro continúa, puedes guardar la diferencia en una meta.

## Compras pequeñas

insights-pequenas-titulo = { $n } compras pequeñas sumaron { $total } en { $mes }
insights-pequenas-corpo = Compras de hasta { $limite } cada una, que parecen poco de a una. Las que más pesaron: { $lista }.
insights-pequenas-item = { $categoria } ({ $valor })
insights-pequenas-acao = Un tope mensual para esas categorías en el presupuesto ayuda a ver el total.
insights-sem-categoria = sin categoría

## Gastos por encima de los ingresos

insights-deficit-titulo = Gastos por encima de los ingresos en { $valor } en los últimos 3 meses
insights-deficit-corpo = De { $de } a { $ate }, entraron { $renda } y salieron { $despesas } en gastos. La diferencia salió de ahorros, sobregiro, tarjeta o préstamos.
insights-deficit-acao = Revisar las categorías más grandes del presupuesto y la previsión de saldo de los próximos meses.

## División de los ingresos (50/30/20)

insights-divisao-titulo = De los ingresos: { $essencial } en lo esencial, { $resto } en el resto, { $sobra } sobró
insights-divisao-corpo = De { $de } a { $ate }, entraron { $renda }: { $essencial } fueron a lo esencial (vivienda, supermercado, salud, transporte… las categorías marcadas como esenciales), { $resto } al resto y { $sobra } sobraron. Una referencia común es 50% para lo esencial, 30% para el resto y 20% para ahorrar.
insights-divisao-acao-folga = Buen margen: guardar lo que sobra en una meta (primero el fondo de emergencia) lo hace rendir.
insights-divisao-acao-essencial = Lo esencial pesa mucho: renegociar cuentas fijas (alquiler, seguro médico, internet, teléfono) suele dar más resultado que recortar gastos pequeños.
insights-divisao-acao-teto = Un tope en el presupuesto para lo que no es esencial ayuda a que sobre más a fin de mes.

insights-kit-titulo = Genera el kit de recuperación de la bóveda
insights-kit-corpo = Si se olvida la contraseña maestra, nadie puede abrir la bóveda, ni siquiera Treasury. El kit de recuperación es un código que abre la bóveda sin la contraseña: imprímelo y guárdalo en papel, lejos del ordenador. Si ya generaste uno antes, genéralo de nuevo para que la app lo registre (el código es el mismo).
insights-kit-velho-titulo = El kit de recuperación antiguo ya no abre la bóveda
insights-kit-velho-corpo = La contraseña maestra cambió después de generar el kit y el código impreso dejó de valer. Genera un kit nuevo y cambia el papel guardado.
insights-kit-acao = Abre Ajustes › Seguridad y genera el kit.
