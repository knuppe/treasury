# Alertas de la bandeja del Agente (núcleo): recordatorios (cuentas y resúmenes de tarjeta, fechas
# de corte, watchlist, saldo negativo, suscripciones, presupuesto), rutinas del gestor (alertas,
# cierre mensual, revisión), fuentes de datos apagadas y el score de salud, además de las
# notificaciones del sistema. Las alertas guardan la clave y los argumentos (V128) y el texto sale
# en el idioma de quien lee (docs/i18n-design.md). Los nombres de las herramientas entre
# paréntesis (get_budget_status…) quedan como están.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Formatos

alertas-mes-ano = { $mes } de { $ano }
alertas-mes-ano-curto = { $mes }/{ $ano }
alertas-item-valor = { $nome } { $valor }

## Origen de la alerta

alertas-origem-lembretes = recordatorios
alertas-origem-rotina-alertas = rutina: alertas
alertas-origem-fechamento = rutina: cierre mensual
alertas-origem-fechamento-motor = rutina: cierre mensual ({ $motor })
alertas-origem-score = score
alertas-origem-fontes = fuentes de datos

## Notificaciones del sistema

alertas-notificacao-detalhes = Abre Treasury para ver los detalles.

## Cuándo

alertas-quando = { $dias ->
    [0] hoy
    [1] mañana
   *[other] en { $dias } días
}
alertas-quando-data = { $dias ->
    [0] hoy
    [1] mañana
   *[other] en { $dias } días ({ $data })
}

## Cuentas y resúmenes de tarjeta por pagar

alertas-vence-fatura = El resumen de { $conta } vence { $quando }
alertas-vence-conta = { $descricao } vence { $quando }
alertas-conta-sem-descricao = Cuenta
alertas-vence-corpo = { $valor } en { $conta }, vencimiento { $data }.
alertas-vence-corpo-painel = { $valor } en { $conta } (próximos vencimientos del panel).

## Fechas de corte de rendimientos

alertas-datacom-titulo = { $ativo }: fecha de corte { $quando } ({ $tipo ->
    [jcp] JCP
    [rendimento] rendimiento
   *[provento] dividendo
})
alertas-datacom-corpo = Quien tenga { $ativo } al cierre del { $data } recibe { $moeda } { $valor } por cuota{ $pagamento }{ $liquido }.
alertas-datacom-pagamento = , pagado el { $data }
alertas-datacom-liquido = { " " }(unos { $valor } netos para ti)

## Watchlist

alertas-cotacao-acima = { $ativo }: cotización por encima de { $limite }
alertas-cotacao-abaixo = { $ativo }: cotización por debajo de { $limite }
alertas-cotacao-ambos = { $ativo }: cotización por encima de { $acima } y por debajo de { $abaixo }
alertas-cotacao-gatilhos = { $ativo }: { $gatilhos }
alertas-cotacao-corpo = { $ativo } cotiza a { $moeda } { $preco } (watchlist).
alertas-cotacao-normal = La cotización volvió a estar dentro de las alertas.

## Saldo negativo en la previsión

alertas-saldo-negativo-titulo = { $conta }: el saldo previsto queda negativo
alertas-saldo-negativo-corpo = Según la previsión de saldo, { $conta } queda en negativo a partir del { $data } y llega a { $minimo } el { $data_minimo } (el punto más bajo de los próximos 30 días).
alertas-saldo-negativo-fim = Según la previsión del { $data }, el saldo ya no queda negativo.

## Suscripciones

alertas-assinatura-aumento-titulo = { $nome }: el precio subió { $pct }
alertas-assinatura-aumento-corpo = De { $antes } a { $agora } en el cobro del { $data } ({ $conta }).
alertas-assinatura-nova-titulo = Nuevo cobro recurrente: { $nome }
alertas-assinatura-nova-corpo = { $valor } por { $periodo ->
    [ano] año
   *[mes] mes
} desde el { $data } en { $conta }. Si no lo reconoces, revísalo.

## Presupuesto

alertas-orcamento-estourado-titulo = { $categoria }: presupuesto excedido
alertas-orcamento-estourado-corpo = { $gasto } gastados frente a { $previsto } previstos en el mes ({ $pct } de lo previsto).
alertas-orcamento-estourado-corpo-mes = { $gasto } gastados frente a { $previsto } previstos en { $mes } ({ $pct } de lo previsto).
alertas-orcamento-ritmo-titulo = { $categoria }: al ritmo de superarlo
alertas-orcamento-ritmo-corpo = Ya van { $gasto } de { $previsto } ({ $pct }) con { $decorrido } del mes transcurrido.

## Cartera

alertas-rebalancear-titulo = Cartera desequilibrada: { $classe }
alertas-rebalancear-corpo = { $atual } de la cartera frente a un objetivo de { $alvo }. Análisis de tus propios objetivos, no una recomendación de inversión.

## Fuentes de datos

alertas-fonte-desligada-titulo = { $fonte } apagada por fallas
alertas-fonte-desligada-corpo = La descarga de { $fonte } falló { $dias } días seguidos y la fuente se apagó para no guardar datos erróneos. Último error: { $erro }. Cuando se resuelva el problema, vuelve a encenderla en Ajustes › Fuentes de datos.
alertas-erro-desconhecido = desconocido

## Score de salud de los activos

alertas-score-titulo = { $ativo }: score de salud { $nota } ({ $faixa })
alertas-score-titulo-sem-nota = { $ativo }: score de salud { $faixa }
alertas-score-caiu = El score bajó { $pontos } puntos desde { $desde }
alertas-score-faixa-piorou = La franja empeoró de { $antes } a { $agora }
alertas-score-red-flag = Nueva red flag: { $flag }

## Rutinas del gestor

alertas-rotina-revisao = Revisión continua
alertas-rotina-fechamento = Cierre mensual
alertas-rotina-alertas = Alertas
alertas-rotina-sem-pendentes = Ningún movimiento sin categoría.
alertas-rotina-revisao-feita = { $sugeridos } de { $total } movimientos sin categoría recibieron una sugerencia; { $propostas } propuestas en la cola.
alertas-rotina-motor-categorizacao-fora = motor de categorización no disponible: { $erro }
alertas-rotina-motor-fora = motor no disponible, informe sin IA: { $erro }
alertas-rotina-fechamento-gravado = Cierre de { $mes } guardado en la bandeja de alertas.
alertas-rotina-alertas-novos = { $n ->
    [one] { $n } alerta nueva.
   *[other] { $n } alertas nuevas.
}

## Cierre mensual (sin motor de IA)

alertas-fechamento-titulo = Cierre de { $mes }
alertas-fechamento-resumo = **Resumen de { $mes }**
alertas-fechamento-fluxo = - Ingresos: { $receitas } · Gastos: { $despesas } · Saldo: { $saldo }{ $poupanca }
alertas-fechamento-poupanca = { " " }· Tasa de ahorro: { $pct }
alertas-fechamento-patrimonio = - Patrimonio neto: { $valor } ({ $sinal }{ $variacao } en el mes)
alertas-fechamento-maiores = - Mayores gastos: { $lista }
alertas-fechamento-recomendacoes = **Recomendaciones**
alertas-fechamento-aviso = _Análisis basado en tus datos, no es asesoramiento financiero profesional._
alertas-rec-orcamento = { $categoria } superó el presupuesto: { $gasto } gastados frente a { $previsto } previstos. Revisa los movimientos de la categoría o ajusta lo previsto para el próximo mes.
alertas-rec-poupanca = La tasa de ahorro quedó en { $pct }. Define una meta de ahorro mensual y sigue las categorías de mayor gasto.
alertas-rec-despesas-subiram = Los gastos subieron de { $antes } a { $agora } ({ $pct } más que el mes anterior). Mira qué gastos fueron puntuales.
alertas-rec-sem-categoria = { $n } movimientos del mes están sin categoría: categorízalos para que los informes sean correctos; la revisión continua ya sugiere categorías.
alertas-rec-meta = La meta "{ $meta }" va fuera de ritmo: aporte mensual necesario de { $valor }.
alertas-rec-carteira = Cartera fuera de la asignación objetivo en { $classe }: { $atual } de la cartera frente a un objetivo de { $alvo }. Considera dirigir allí los próximos aportes.
alertas-rec-maiores = Los mayores gastos del mes fueron { $lista }: vale la pena revisar si alguno se puede reducir.
alertas-rec-sem-orcamento = No hay presupuesto para el mes: define lo previsto de las categorías principales para seguir el ritmo de gastos.
alertas-rec-sem-metas = No hay metas registradas: empieza por un fondo de emergencia de algunos meses de gastos.
alertas-rec-em-dia = Mantén los movimientos y las importaciones al día para que el próximo cierre sea más preciso.

## Tipos de alerta (rodapé do alerta)

alertas-tipo-budget-over = Presupuesto superado
alertas-tipo-budget-pace = Ritmo del presupuesto
alertas-tipo-due-soon = Vencimiento
alertas-tipo-rebalance = Rebalanceo
alertas-tipo-monthly-close = Cierre mensual
alertas-tipo-negative-balance = Saldo negativo previsto
alertas-tipo-asset-score = Puntaje de salud
alertas-tipo-source-disabled = Fuente de datos desactivada
alertas-tipo-asset-data-gaps = Datos de los activos
alertas-tipo-stale-accounts = Cuentas sin extracto
alertas-tipo-iptu-due = IPTU por vencer
alertas-tipo-iptu-missing = IPTU sin boleta
alertas-tipo-dividend-datacom = Fecha de corte del dividendo
alertas-tipo-price-target = Precio objetivo
alertas-tipo-subscription-increase = Aumento de suscripción
alertas-tipo-subscription-new = Suscripción nueva

## Datos de los activos (enriquecimiento)

alertas-dados-ativos-titulo = { $n } dato(s) de tus activos para completar
alertas-dados-ativos-corpo = Las fuentes oficiales que Treasury ya descargó tienen sugerencias para **{ $campos } campo(s)** de { $ativos } activo(s), como sector, segmento, ISIN y CNPJ (ej.: { $exemplos }).
alertas-dados-ativos-onde = Revisa la fuente y la confianza de cada uno y guarda lo que quieras en Cartera › Salud de los activos › Datos de los activos. Nada cambia sin que elijas.
alertas-reduflacao-titulo = { $produto }: envase más chico, precio más alto
alertas-reduflacao-corpo = { $produto } pasó de { $antes } a { $agora } { $unidade } y el precio por { $unidade } cambió { $variacao }. Es la reduflación: menos producto por el mismo precio (o más caro).
alertas-tela-entrou-titulo = { $ticker } pasó a cumplir la pantalla { $tela }
alertas-tela-entrou-corpo = { $ticker } pasó a cumplir los filtros de tu pantalla { $tela }. Es una lista de quien cumple criterios, no una recomendación de inversión.
alertas-tela-saiu-titulo = { $ticker } dejó de cumplir la pantalla { $tela }
alertas-tela-saiu-corpo = { $ticker }, que está en tu cartera, dejó de cumplir los filtros de la pantalla { $tela }.
alertas-cartao-limite-titulo = { $cartao }: { $pct } del límite usado
alertas-cartao-limite-corpo = { $usado } usados de { $limite } (quedan { $disponivel }), contando las cuotas futuras.
alertas-cartao-anuidade-renova-titulo = { $cartao }: la anualidad se renueva en { $mes }
alertas-cartao-anuidade-renova-corpo = La anualidad de { $valor } se renueva pronto; en los últimos 12 meses fueron { $pago }. Es un buen momento para pedir exención o descuento al banco.
alertas-cartao-anuidade-confira-titulo = { $cartao }: revisa la anualidad
alertas-cartao-anuidade-isenta-corpo = Cobro de { $valor } el { $data } en un resumen que había alcanzado la exención.
alertas-cartao-anuidade-parcela-extra-corpo = Cobro de { $valor } el { $data } que pasa el número de cuotas acordado en 12 meses.
alertas-cartao-anuidade-valor-corpo = Cobro de { $valor } el { $data }; la cuota acordada es { $esperado }.
alertas-cartao-rotativo-titulo = { $cartao }: el resumen de { $mes } entró en crédito rotativo
alertas-cartao-rotativo-corpo-br = Quedaron { $valor } sin pagar al vencimiento ({ $data }). El rotativo tiene los intereses más altos del mercado y solo puede durar hasta el próximo resumen; después el banco debe ofrecer el pago en cuotas. Si puedes, cancela el saldo o compara las cuotas con un préstamo más barato.
alertas-cartao-rotativo-corpo = Quedaron { $valor } sin pagar al vencimiento ({ $data }). El saldo que queda en la tarjeta suele tener intereses altos; si puedes, cancélalo o compara con un préstamo más barato.
alertas-cartao-rotativo-teto-titulo = { $cartao }: cargos por encima del monto de la deuda
alertas-cartao-rotativo-teto-corpo = Los intereses y cargos desde el rotativo ({ $encargos }) ya superan el monto que quedó sin pagar ({ $valor }). En Brasil, la Ley 14.690/2023 limita estos cargos al 100% de la deuda original; conviene revisar el resumen con el banco.
alertas-cartao-cashback-titulo = { $cartao }: el cashback del resumen de { $mes } no llegó
alertas-cartao-cashback-corpo = Previsto { $previsto } y recibido { $recebido } hasta el { $data }. Revisa la app del banco o pide el crédito.
alertas-cartao-pontos-titulo = { $cartao }: los puntos del resumen de { $mes } no llegaron
alertas-cartao-pontos-corpo = Previstos { $previsto } puntos y acreditados { $recebido } hasta el { $data }. Revisa el programa o pide el crédito.
alertas-pontos-vencendo-titulo = { $programa }: los puntos vencen { $quando }
alertas-pontos-vencendo-corpo = { $pontos } puntos de { $membro } vencen el { $data }. Úsalos, transfiérelos o renuévalos antes.

## Cuentas sin extracto reciente

alertas-contas-desatualizadas-titulo = { $n ->
    [one] 1 cuenta sin extracto reciente. ¿Importar?
   *[other] { $n } cuentas sin extracto reciente. ¿Importar?
}
alertas-contas-desatualizadas-corpo = El saldo puede estar incompleto: { $contas }. Importa el extracto o revisa el saldo para que la salud financiera y el fondo de emergencia usen el valor correcto.
alertas-contas-desatualizadas-item = { $conta } (último movimiento el { $data })
alertas-contas-desatualizadas-item-sem = { $conta } (sin movimientos)
