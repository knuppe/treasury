# Sugerencias del agente compañero (núcleo, `ai.suggestions`): la tarjeta (título y detalle) y el
# pedido listo que va al agente como mensaje del usuario, en el idioma de la instalación. Los
# nombres de las herramientas (categorize_transactions…) quedan como están.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

sugestoes-nomes-mais = { $nomes } y { $n } más

## Primer registro

sugestoes-cadastro-titulo = Registrar las cuentas de la familia
sugestoes-cadastro-detalhe = Dile al agente dónde tienen cuentas (bancos, tarjetas, corredoras) y propone el registro y la mejor forma de importar cada una.
sugestoes-cadastro-pedido = Quiero registrar dónde tiene dinero la familia. Pregunta dónde tenemos cuentas (bancos, tarjetas de crédito, corredoras, cuentas en el exterior) y, para cada una: propone el registro (create_account con institution_name; tarjetas con cierre y vencimiento), dime la mejor forma de traer los datos (OFX o CSV del banco, resumen de tarjeta en PDF, informe del Área del Inversionista de B3, notas de corretaje en PDF) y dónde descargar cada archivo. No inventes saldos: pregunta.
sugestoes-cadastro-pedido-exterior = Quiero registrar dónde tiene dinero la familia. Pregunta dónde tenemos cuentas (bancos, tarjetas de crédito, corredoras, planes de pensiones, cuentas en otro país) y, para cada una: propone el registro (create_account con institution_name; tarjetas con cierre y vencimiento), dime la mejor forma de traer los datos (OFX, QFX, CSV o Norma 43 del banco, el extracto de la corredora) y dónde descargar cada archivo. No inventes saldos: pregunta.

## Movimientos sin categoría

sugestoes-sem-categoria-titulo = { $n ->
    [one] { $n } movimiento sin categoría
   *[other] { $n } movimientos sin categoría
}
sugestoes-sem-categoria-detalhe = En los últimos 90 días. El agente propone las categorías y reglas para los próximos.
sugestoes-sem-categoria-pedido = Hay { $n } movimientos sin categoría desde { $desde }. Propón las categorías (categorize_transactions) y, para los patrones que se repiten, una regla (create_rule).

## Cuentas sin extracto reciente

sugestoes-sem-extrato-titulo = { $n ->
    [one] 1 cuenta sin extracto reciente
   *[other] { $n } cuentas sin extracto reciente
}
sugestoes-sem-extrato-detalhe = { $contas }: sin movimientos hace más de { $dias } días.
sugestoes-sem-extrato-pedido = Las cuentas { $contas } no tienen movimientos hace más de { $dias } días. Para cada una, dime la mejor forma de traer el extracto (OFX o CSV del banco, resumen de tarjeta en PDF) y dónde descargar el archivo en el sitio o la app del banco.

## Resumen de tarjeta por vencer

sugestoes-fatura-titulo = El resumen de { $cartao } vence el { $data }
sugestoes-fatura-detalhe = { $valor } por pagar.
sugestoes-fatura-detalhe-mais = { $valor } por pagar (+{ $n } resumen(es)).
sugestoes-fatura-pedido = El resumen de la tarjeta { $cartao } vence el { $data }. Revisa el total con list_card_statements, dime de qué cuenta sale el pago y si su saldo alcanza.

## Presupuesto excedido

sugestoes-orcamento-titulo = { $n ->
    [one] Presupuesto excedido en { $n } categoría
   *[other] Presupuesto excedido en { $n } categorías
}
sugestoes-orcamento-detalhe = { $categorias }: { $valor } por encima de lo previsto.
sugestoes-orcamento-pedido = El presupuesto de { $mes } se excedió en { $categorias }. Muestra qué impulsó el gasto (spending_summary) y sugiere un ajuste realista para el resto del mes.

## Activos sin cotización

sugestoes-cotacao-titulo = { $n ->
    [one] { $n } activo sin cotización reciente
   *[other] { $n } activos sin cotización reciente
}
sugestoes-cotacao-detalhe = { $ativos }: la cartera usa el costo o la última cotización.
sugestoes-cotacao-pedido = Los activos { $ativos } no tienen cotización hace más de 15 días. Revisa el registro (list_assets) y propón la fuente de cotización correcta (update_asset con price_provider y provider_symbol).

## Importación de B3

sugestoes-b3-titulo = Importar operaciones de B3
sugestoes-b3-detalhe = { $corretoras }: sin operaciones hace más de { $dias } días. El informe del Área del Inversionista trae negociaciones y rendimientos.
sugestoes-b3-pedido = Las corredoras { $corretoras } no tienen operaciones recientes en Treasury. Explica cómo descargar el informe de negociación y de rendimientos en el Área del Inversionista de B3 (o las notas de corretaje en PDF) y, si quiero, pide abrir el sitio.

## Propuestas y alertas

sugestoes-propostas-titulo = { $n ->
    [one] { $n } propuesta esperando aprobación
   *[other] { $n } propuestas esperando aprobación
}
sugestoes-propostas-detalhe = Nada cambia hasta que apruebes; cada una muestra el diff exacto.
sugestoes-propostas-pedido = Resume las propuestas pendientes (list_pending_proposals) y dime cuáles parecen seguras para aprobar.
sugestoes-alertas-titulo = { $n ->
    [one] { $n } alerta nueva
   *[other] { $n } alertas nuevas
}
sugestoes-alertas-detalhe = De las rutinas del gestor (presupuesto, vencimientos, cartera).
sugestoes-alertas-pedido = Lee las alertas nuevas (list_alerts) y dime qué hacer primero.
sugestoes-reserva-titulo = { $n ->
    [one] Una inversión de liquidez diaria fuera de las metas
   *[other] { $n } inversiones de liquidez diaria fuera de las metas
}
sugestoes-reserva-detalhe = { $ativos }: vinculadas a la meta de fondo de emergencia, entran en la reserva y en la Seguridad de la salud financiera.
sugestoes-reserva-pedido = Las inversiones { $ativos } tienen liquidez diaria y no están vinculadas a ninguna meta. Mira la meta de fondo de emergencia (list_goals) y ayúdame a decidir si entran en ella.
