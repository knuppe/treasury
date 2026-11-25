# Cuentas, tarjetas y cambio de moneda (núcleo: `accounts`, `cards`, `fx`): errores de
# validación, motivos del historial, pistas de la sincronización de saldo y textos que el núcleo
# guarda al crear ajustes, pagos de tarjeta y operaciones de cambio, en el idioma de la
# instalación.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Cuentas

nucleo-contas-ref-conta = cuenta { $id }
nucleo-contas-ref-conta-pagadora = cuenta pagadora { $id }
nucleo-contas-ref-instituicao = institución { $id }
nucleo-contas-historico-criada = Cuenta creada: { $nome }
nucleo-contas-historico-alterada = Cuenta modificada: { $nome }
nucleo-contas-historico-apagada = Cuenta eliminada: { $nome }
nucleo-contas-historico-conciliacao = Conciliación de la cuenta { $nome } hasta el { $data }
nucleo-contas-erro-tipo-invalido = tipo de cuenta no válido: { $valor }
nucleo-contas-erro-tipo-so-brasil = Este tipo de cuenta solo existe en Brasil: la bóveda tiene que estar configurada para Brasil.
nucleo-contas-erro-historico-longo = período demasiado largo para puntos diarios (máximo { $max } días); usa interval = month
nucleo-contas-erro-agencia = sucursal: usa solo números y guiones
nucleo-contas-erro-numero-conta = número de cuenta: usa solo números y guiones
nucleo-contas-erro-cheque-especial-tipo = el sobregiro solo vale para una cuenta corriente
nucleo-contas-erro-apagar-lancamentos = { $n ->
    [one] la cuenta { $nome } tiene { $n } movimiento; elimínalo antes o archiva la cuenta
   *[other] la cuenta { $nome } tiene { $n } movimientos; elimínalos antes o archiva la cuenta
}
nucleo-contas-erro-apagar-recorrencias = { $n ->
    [one] la cuenta { $nome } se usa en { $n } recurrencia; elimínala antes
   *[other] la cuenta { $nome } se usa en { $n } recurrencias; elimínalas antes
}
nucleo-contas-erro-apagar-parcelamentos = { $n ->
    [one] la cuenta { $nome } tiene { $n } plan de cuotas; elimínalo antes
   *[other] la cuenta { $nome } tiene { $n } planes de cuotas; elimínalos antes
}
# Cierre de la cuenta (accounts/closing.rs).
nucleo-contas-historico-encerrada = Cuenta cerrada: { $nome }
nucleo-contas-historico-reaberta = Cuenta reabierta: { $nome }
nucleo-contas-erro-encerrada = La cuenta { $nome } se cerró el { $data }. Para registrar algo después de esa fecha, reabre la cuenta.
nucleo-contas-encerrar-erro-futuro = La fecha de cierre no puede ser posterior a hoy.
nucleo-contas-encerrar-erro-antes-abertura = La fecha de cierre es anterior a la apertura de la cuenta ({ $data }).
nucleo-contas-encerrar-erro-ja-encerrada = La cuenta ya se cerró el { $data }.
nucleo-contas-encerrar-erro-saldo = La cuenta todavía tiene un saldo de { $valor } en esa fecha. Transfiere el saldo o registra el ajuste antes de cerrarla.
nucleo-contas-encerrar-erro-depois = { $n ->
    [one] Hay { $n } movimiento después de esa fecha (el último el { $data }). Elige una fecha a partir de él.
   *[other] Hay { $n } movimientos después de esa fecha (el último el { $data }). Elige una fecha a partir de él.
}
nucleo-contas-encerrar-erro-posicoes = La cuenta todavía tiene posición en { $ativos }. Registra la venta o el traspaso de custodia antes.
nucleo-contas-encerrar-erro-parcelas = { $n ->
    [one] La tarjeta todavía tiene { $n } cuota por vencer.
   *[other] La tarjeta todavía tiene { $n } cuotas por vencer.
}
# Por qué el OFX no encontró la cuenta solo.
nucleo-contas-ofx-sem-numero = el OFX no trae el número de cuenta
nucleo-contas-ofx-sem-cartao = ninguna tarjeta termina en esos dígitos; completa los últimos dígitos de la tarjeta en su ficha
nucleo-contas-ofx-sem-conta = ninguna cuenta tiene ese número; completa banco, sucursal y cuenta en su ficha
nucleo-contas-ofx-varias = coincide más de una cuenta; elige cuál

## Tarjeta de crédito

nucleo-contas-cartao-erro-fechamento = el día de cierre debe estar entre 1 y 31
nucleo-contas-cartao-erro-vencimento = el día de vencimiento debe estar entre 1 y 31
nucleo-contas-cartao-erro-limite = el límite de la tarjeta no puede ser negativo
nucleo-contas-cartao-erro-paga-propria = la tarjeta no puede pagar su propio resumen
nucleo-contas-cartao-erro-pagadora-cartao = la cuenta que paga la tarjeta no puede ser otra tarjeta
nucleo-contas-cartao-erro-final = indica los 4 últimos dígitos de la tarjeta (solo números)
nucleo-contas-cartao-erro-dados = indica los datos de la tarjeta (día de cierre y de vencimiento)
nucleo-contas-cartao-erro-so-cartao = los datos de tarjeta solo valen para cuentas de tipo tarjeta de crédito
nucleo-contas-cartao-erro-nao-cartao = la cuenta elegida no es una tarjeta de crédito
nucleo-contas-cartao-erro-sem-ciclo = la tarjeta no tiene día de cierre y de vencimiento

## Resúmenes y cuotas

nucleo-contas-ref-fatura = resumen { $id }
nucleo-contas-ref-parcelamento = plan de cuotas { $id }
nucleo-contas-historico-pagamento-fatura = Pago de la tarjeta { $cartao } con vencimiento el { $vencimento }
nucleo-contas-historico-total-informado = Total del resumen informado
nucleo-contas-historico-movidos-fatura = { $n ->
    [one] { $n } movimiento movido a otro resumen
   *[other] { $n } movimientos movidos a otro resumen
}
nucleo-contas-historico-compra-parcelada = Compra en { $n } cuotas: { $descricao }
nucleo-contas-historico-parcelamento-apagado = Plan de cuotas eliminado: { $descricao }
# Descripción del pago de tarjeta (se guarda en el movimiento); $mes como 10/2026.
nucleo-contas-fatura-descricao-pagamento = Pago de la tarjeta { $cartao } ({ $mes })
nucleo-contas-fatura-erro-pagadora = indica la cuenta que paga la tarjeta
nucleo-contas-fatura-erro-moeda = la cuenta pagadora está en { $moeda_conta } y la tarjeta en { $moeda_cartao }; usa una cuenta en la moneda de la tarjeta
nucleo-contas-fatura-erro-valor-positivo = el importe del pago debe ser positivo
nucleo-contas-fatura-erro-nada-a-pagar = este resumen no tiene nada por pagar
nucleo-contas-fatura-erro-destino = indica el resumen de destino (statement_id, o account_id y reference_month)
nucleo-contas-fatura-erro-outro-cartao = el movimiento "{ $descricao }" no es de esta tarjeta
nucleo-contas-parcelas-erro-numero = el número de cuotas debe estar entre 1 y 120
nucleo-contas-parcelas-erro-total = indica el importe total de la compra (positivo)
nucleo-contas-parcelas-erro-menor = el importe total es menor que el número de cuotas
nucleo-contas-parcelas-erro-descricao = indica la descripción de la compra
nucleo-contas-parcelas-erro-conciliada = la cuota "{ $descricao }" del { $data } está conciliada; desconcíliala antes de eliminar el plan de cuotas

## Sobregiro

nucleo-contas-cheque-erro-limite = el límite del sobregiro no puede ser negativo
nucleo-contas-cheque-erro-indexador = la tasa anual y la mensual traen índices diferentes
nucleo-contas-cheque-erro-numero = { $campo }: número no válido ({ $valor })
nucleo-contas-cheque-erro-negativo = { $campo } no puede ser negativo
nucleo-contas-cheque-erro-acima = { $campo } por encima de { $max }%: revísalo
nucleo-contas-cheque-campo-taxa-mes = tasa mensual
nucleo-contas-cheque-campo-desconto = descuento
nucleo-contas-cheque-campo-taxa-ano = tasa anual
nucleo-contas-cheque-campo-iof-dia = IOF básico diario
nucleo-contas-cheque-campo-iof-adicional = IOF adicional
nucleo-contas-cheque-campo-cet-mes = costo total mensual (CET)
nucleo-contas-cheque-campo-cet-ano = costo total anual (CET)

## Aportaciones de capital

nucleo-contas-ref-saldo-cotas = saldo de las aportaciones
nucleo-contas-historico-cotas-capital = Aportaciones de capital: { $nome }
nucleo-contas-historico-cotas-removidas = Aportaciones de capital quitadas
nucleo-contas-historico-cotas-saldo = Saldo de las aportaciones de capital
nucleo-contas-historico-cotas-aportes = Aportaciones de capital fuera de los gastos
nucleo-contas-historico-cotas-saldo-removido = Saldo de las aportaciones de capital eliminado
nucleo-contas-cotas-erro-tipo-conta = las aportaciones de capital van en una cuenta corriente (o de ahorro) de la cooperativa
nucleo-contas-cotas-erro-ative = activa primero las aportaciones de capital en esta cuenta
nucleo-contas-cotas-erro-saldo-negativo = el saldo de las aportaciones no puede ser negativo

## Sincronización de saldo

nucleo-contas-ref-sincronia = sincronización de saldo { $id }
nucleo-contas-historico-sincronia = Sincronización de saldo: { $nome } el { $data }
nucleo-contas-historico-sincronia-motivo = Motivo de la diferencia de saldo
# Descripción y nota del ajuste (se guardan en el movimiento).
nucleo-contas-sincronia-descricao = Sincronización de saldo
nucleo-contas-sincronia-nota = Saldo en el banco el { $data }: { $banco }; Treasury calculaba { $calculado }.
nucleo-contas-sincronia-erro-futuro = la fecha del saldo no puede estar en el futuro
nucleo-contas-sincronia-erro-motivo = escribe el motivo de la diferencia
# Causas probables de la diferencia.
nucleo-contas-sincronia-dica-mesmo-valor = { $n ->
    [one] { $n } movimiento con el importe exacto de la diferencia ({ $valor }): puede que esté de más (o falte) en Treasury.
   *[other] { $n } movimientos con el importe exacto de la diferencia ({ $valor }): puede que uno esté de más (o falte) en Treasury.
}
nucleo-contas-sincronia-dica-previsto = Hay un previsto con el importe de la diferencia que todavía no es efectivo: quizá ya ocurrió en el banco.
nucleo-contas-sincronia-dica-sinal = Un movimiento de { $valor } con el signo cambiado explicaría la diferencia.
nucleo-contas-sincronia-dica-duplicados = { $n } movimientos parecen duplicados (misma fecha, importe y descripción).
nucleo-contas-sincronia-dica-conferir = { $n ->
    [one] Ninguna causa evidente: compara el { $n } movimiento del período con el extracto (suelen faltar comisiones, intereses y reembolsos).
   *[other] Ninguna causa evidente: compara los { $n } movimientos del período con el extracto (suelen faltar comisiones, intereses y reembolsos).
}

## Cambio de moneda

nucleo-contas-ref-taxa-cambio = tipo de cambio
nucleo-contas-historico-cambio = Cambio { $origem } → { $destino } el { $data }
nucleo-contas-historico-cambio-manual = Tipo de cambio informado a mano
nucleo-contas-historico-cambio-removido = Tipo de cambio eliminado
# Descripciones y nota de los movimientos de la operación (se guardan).
nucleo-contas-cambio-descricao = Cambio { $origem } → { $destino }
nucleo-contas-cambio-descricao-iof = IOF del cambio
nucleo-contas-cambio-descricao-tarifa = Comisión del cambio
nucleo-contas-cambio-nota-iof = Operación de cambio { $origem } → { $destino } ({ $pct }%).
nucleo-contas-cambio-erro-numero = { $campo } no válido: { $valor }
nucleo-contas-cambio-campo-referencia = tasa de referencia
nucleo-contas-cambio-campo-spread = spread
nucleo-contas-cambio-campo-taxa = tasa
nucleo-contas-cambio-campo-iof = IOF
nucleo-contas-cambio-campo-ptax = PTAX
nucleo-contas-cambio-erro-grande = importe demasiado grande para el cambio: revisa el importe y la tasa
nucleo-contas-cambio-erro-mesma-moeda = el cambio es entre monedas distintas; para la misma moneda usa una transferencia
nucleo-contas-cambio-erro-sem-taxa = indica la tasa aplicada o la de referencia (en Brasil, la PTAX)
nucleo-contas-cambio-erro-taxa-positiva = el tipo de cambio debe ser positivo
nucleo-contas-cambio-erro-valor = indica el importe que sale o el que entra (positivo)
nucleo-contas-cambio-erro-ptax = respuesta de la PTAX no válida: { $erro }
nucleo-contas-cambio-erro-referencia-brl = la referencia es de una moneda extranjera en la moneda de la familia
nucleo-contas-cambio-sem-referencia = no hay cotización de referencia para { $moeda }: indica la tasa a mano
nucleo-contas-cambio-erro-taxa = tasa no válida: { $valor }
nucleo-contas-cambio-erro-intervalo = importe convertido fuera de rango
nucleo-contas-cambio-erro-moedas-iguais = las monedas deben ser distintas
# Boletines PTAX del Banco Central de Brasil.
nucleo-contas-ptax-abertura = Apertura
nucleo-contas-ptax-intermediario = Intermedio
nucleo-contas-ptax-fechamento = Cierre

nucleo-contas-sincronia-copias-removidas = la diferencia venía de movimientos copiados de otra cuenta, ya eliminados

## Plásticos de la tarjeta (titular, adicionales, virtuales)
nucleo-contas-ref-plastico = tarjeta { $id }
nucleo-contas-plastico-final = terminada en { $final }
nucleo-contas-plastico-tipo-titular = tarjeta del titular
nucleo-contas-plastico-tipo-adicional = tarjeta adicional
nucleo-contas-plastico-tipo-virtual = tarjeta virtual
nucleo-contas-plastico-tipo-temporario = tarjeta temporal
nucleo-contas-plastico-erro-tipo = tipo de tarjeta desconocido (titular, adicional, virtual o temporal)
nucleo-contas-plastico-erro-numero-inteiro = guarda solo los 4 últimos dígitos de la tarjeta, nunca el número completo
nucleo-contas-plastico-erro-sublimite = el sublímite no puede ser negativo
nucleo-contas-plastico-erro-final-repetido = esta tarjeta ya tiene un plástico terminado en { $final }
nucleo-contas-plastico-erro-outro-cartao = el plástico es de otra tarjeta
nucleo-contas-historico-plastico-criado = Tarjeta adicional en { $cartao }
nucleo-contas-historico-plastico-alterado = Cambio en la { $plastico }
nucleo-contas-historico-plastico-apagado = Eliminación de la { $plastico }

## Límite, fechas del resumen y anualidad
nucleo-contas-ref-limite = registro de límite
nucleo-contas-ref-anuidade = anualidad de la tarjeta { $id }
nucleo-contas-fatura-erro-datas = el vencimiento no puede ser antes del cierre
nucleo-contas-fatura-erro-fechamento = el cierre no puede ser antes del primer día de compras del resumen
nucleo-contas-fatura-erro-valor-negativo = los montos informados del resumen no pueden ser negativos
nucleo-contas-anuidade-erro-valor = la anualidad no puede ser negativa
nucleo-contas-anuidade-erro-parcelas = la anualidad se cobra en 1 a 12 cuotas
nucleo-contas-anuidade-erro-mes = el mes de renovación va de 1 a 12
nucleo-contas-anuidade-erro-isencao = indica el monto que exime la anualidad (gasto en el resumen o monto invertido)
nucleo-contas-anuidade-erro-desconto = el descuento es un número entre 0 y 1 (0,5 = 50%)
nucleo-contas-historico-fatura-alterada = Resumen modificado
nucleo-contas-historico-limite = Límite de la tarjeta
nucleo-contas-historico-anuidade = Anualidad de la tarjeta
nucleo-contas-historico-anuidade-fim = Tarjeta sin anualidad
nucleo-contas-recompensa-erro-taxa = la tasa es un número decimal (0,01 = 1% de cashback; puntos por unidad de moneda)
nucleo-contas-recompensa-erro-teto = el tope por resumen no puede ser negativo
nucleo-contas-recompensa-erro-prazo = el plazo del crédito va de 0 a 180 días
nucleo-contas-historico-recompensa = Cashback y puntos de la tarjeta
nucleo-contas-historico-recompensa-fim = Tarjeta sin programa de recompensas

## Puntos y millas
nucleo-pontos-ref-programa = programa de puntos { $id }
nucleo-pontos-ref-fatura = resumen de la tarjeta
nucleo-pontos-ref-movimento = movimiento de puntos
nucleo-pontos-erro-nome = ponle un nombre al programa (ej.: Livelo, Smiles)
nucleo-pontos-erro-valor = los montos no pueden ser negativos
nucleo-pontos-erro-pontos = indica una cantidad de puntos distinta de cero
nucleo-pontos-erro-transferencia-par = la transferencia entre programas se registra por los dos lados a la vez
nucleo-pontos-erro-mesmo-programa = la transferencia necesita dos programas distintos
nucleo-pontos-historico-programa = Programa de puntos: { $descricao }
nucleo-pontos-historico-programa-alterado = Programa de puntos modificado: { $descricao }
nucleo-pontos-historico-programa-apagado = Programa de puntos eliminado: { $descricao }
nucleo-pontos-historico-movimento = Puntos en { $descricao }
nucleo-pontos-historico-transferencia = Transferencia de puntos desde { $descricao }
nucleo-pontos-historico-movimento-apagado = Movimiento de puntos eliminado

## Actualización de las cuentas (docs/account-freshness.md)

nucleo-contas-erro-limites-atualizacao = Los días para "posiblemente incompleto" deben ser más que los de "actualizado hace", y los dos mayores que cero.
nucleo-contas-historico-limites-atualizacao = Límites de actualización de las cuentas cambiados
