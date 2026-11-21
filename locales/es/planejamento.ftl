# Planificación: cuánto puedo gastar hoy, previsión de saldo, plan para saldar deudas,
# suscripciones y escenarios "¿y si…?" (docs/i18n-design.md). Las explicaciones y notas de las
# simulaciones vienen del núcleo. Revisión: borrador por IA, sin revisión de un hablante nativo.

## Etiquetas que se repiten

planejamento-n-meses = { $n } meses
planejamento-n-anos = { $n } años
planejamento-data = Fecha
planejamento-nome = Nombre
planejamento-valor = Monto
planejamento-por-mes = Por mes
planejamento-mais-50-anos = más de 50 años
planejamento-tirar = Quitar { $nome }

## Cuánto puedo gastar hoy

planejamento-hoje-titulo = Cuánto puedo gastar hoy
planejamento-hoje-estourado = Este mes, todo lo que entró ya está comprometido. Aguanta los gastos que puedan esperar hasta el próximo sueldo.
planejamento-hoje-usado = El monto de hoy ya se usó. Mañana vuelve, recalculado con lo que sobre.
planejamento-hoje-apertado = Está ajustado: conviene priorizar solo lo esencial hasta fin de mes.
planejamento-hoje-ok = Dentro de este monto, el mes cierra sin comprometer las cuentas ni las metas.
planejamento-hoje-por-dia =
    { $n ->
        [one] { $valor } por día hasta el { $data } ({ $n } día)
       *[other] { $valor } por día hasta el { $data } ({ $n } días)
    }
planejamento-hoje-gasto = Ya gastado hoy: { $valor }
planejamento-hoje-esconder = Ocultar la cuenta
planejamento-hoje-como = Cómo lo calculé
planejamento-hoje-receitas = Ingresos del mes (recibidos y previstos)
planejamento-hoje-gastos = Gastado hasta ayer
planejamento-hoje-contas = Cuentas que aún vencen en el mes
planejamento-hoje-metas = Lo que falta ahorrar para las metas
planejamento-hoje-sobra = Disponible al empezar el día
planejamento-hoje-amanha-titulo = Por día a partir de mañana
planejamento-hoje-amanha =
    { $n ->
        [one] { $valor } por día a partir de mañana ({ $n } día)
       *[other] { $valor } por día a partir de mañana ({ $n } días)
    }
planejamento-hoje-gasto-linha = Gastado hoy
planejamento-hoje-sobra-amanha = Disponible de mañana en adelante
planejamento-hoje-dividido-amanha =
    { $n ->
        [one] Dividido entre { $n } día, a partir de mañana.
       *[other] Dividido entre { $n } días, a partir de mañana.
    }
planejamento-hoje-dividido =
    { $n ->
        [one] Dividido entre { $n } día, contando hoy.
       *[other] Dividido entre { $n } días, contando hoy.
    }

## Previsión de saldo

planejamento-origem-previsto = Previsto
planejamento-origem-recorrencia = Recurrencia
planejamento-origem-fatura = Resumen
planejamento-origem-parcela = Cuota
planejamento-origem-parcela-detectada = Cuota detectada (estimación)
planejamento-origem-sazonal = gasto anual (estimación)
planejamento-previsao-titulo = Previsión de saldo
planejamento-previsao-horizonte = Horizonte de la previsión
planejamento-previsao-total-hoje = Total hoy
planejamento-previsao-em = Al { $data }
planejamento-previsao-ponto-baixo = Punto más bajo
planejamento-previsao-valor-em = { $valor } el { $data }
planejamento-previsao-negativa = { $conta } queda en negativo el { $data } y llega a { $minimo } el { $quando }.
planejamento-previsao-ja-negativa = { $conta } ya está en negativo hoy y llega a { $minimo } el { $quando }.
planejamento-previsao-nenhuma-negativa = Ninguna cuenta queda en negativo hasta el { $data }.
planejamento-previsao-por-conta = Saldo previsto por cuenta
planejamento-previsao-fonte = Fuente: saldo de hoy + previstos, recurrencias, resúmenes abiertos (pagados con la cuenta de pago de la tarjeta) y cuotas de préstamos
planejamento-previsao-grafico = Saldo previsto del { $de } al { $ate }
planejamento-previsao-eventos = Lo que mueve el saldo
planejamento-previsao-conta = Cuenta
planejamento-previsao-descricao = Descripción
planejamento-previsao-origem = Origen
planejamento-previsao-total = Total

## Plan para saldar deudas

planejamento-quitacao-minimo = Solo el mínimo
planejamento-quitacao-bola-neve = Bola de nieve
planejamento-quitacao-avalanche = Avalancha
planejamento-quitacao-minimo-dica = Solo las cuotas, sin extra.
planejamento-quitacao-bola-neve-dica = Extra en la deuda más pequeña primero: victorias rápidas.
planejamento-quitacao-avalanche-dica = Extra en la tasa más alta primero: menos intereses.
planejamento-quitacao-menos-juros = menos intereses
planejamento-quitacao-livre = Sin deudas en
planejamento-quitacao-meses = Meses
planejamento-quitacao-juros-total = Intereses en total
planejamento-quitacao-sem-fim = sin fin
planejamento-quitacao-juros-a-menos = Intereses ahorrados
planejamento-quitacao-meses-a-menos = Meses ahorrados
planejamento-quitacao-titulo = Plan para saldar deudas
planejamento-quitacao-intro = ¿Cuánto sobra por mes para pagar además de las cuotas? Al saldar una deuda, su cuota también pasa a la siguiente. Incluye deudas que no están en la app, como el sobregiro o el saldo rotativo de la tarjeta.
planejamento-quitacao-extra = Extra por mes
planejamento-quitacao-fora-app = Deuda fuera de la app
planejamento-quitacao-exemplo = Sobregiro
planejamento-quitacao-saldo-devedor = Saldo adeudado
planejamento-quitacao-juros-ano = Interés anual
planejamento-quitacao-minimo-mes = Mínimo por mes
planejamento-quitacao-tirar-esta = Quitar esta deuda
planejamento-quitacao-grafico = Saldo adeudado total, mes a mes
planejamento-quitacao-fonte =
    { $n ->
        [one] Fuente: { $n } deuda; cuotas mínimas + { $extra } de extra por mes
       *[other] Fuente: { $n } deudas; cuotas mínimas + { $extra } de extra por mes
    }
planejamento-quitacao-grafico-aria = Saldo adeudado por estrategia
planejamento-quitacao-dividas = Deudas en la simulación
planejamento-quitacao-divida = Deuda
planejamento-quitacao-saldo = Saldo
planejamento-quitacao-juros-aa = Interés anual
planejamento-quitacao-taxa = { NUMBER($taxa, maximumFractionDigits: 2) }%
planejamento-quitacao-minimo-col = Mínimo
planejamento-quitacao-ordem = Orden de pago
planejamento-quitacao-estrategia = Estrategia del orden
planejamento-quitacao-ordem-de = Orden de pago: { $estrategia }
planejamento-quitacao-quitada-em = Saldada en
planejamento-quitacao-juros-pagos = Intereses pagados
planejamento-quitacao-aviso = Simulación con tasa fija y cuota constante; no es una recomendación financiera.

## Suscripciones y cobros recurrentes

planejamento-assinaturas-em-dia = Al día
planejamento-assinaturas-nova = Nueva
planejamento-assinaturas-nao-veio = No llegó
planejamento-assinaturas-recorrencia-criada = Recurrencia creada
planejamento-assinaturas-recorrencia-criada-descricao = { $nome }: los próximos cobros entran como previstos.
planejamento-assinaturas-criar-recorrencia = Crear recurrencia
planejamento-assinaturas-recorrencia-ok = recurrencia ok
planejamento-assinaturas-nao-e = No es suscripción
planejamento-assinaturas-assinatura = Suscripción
planejamento-assinaturas-frequencia = Frecuencia
planejamento-assinaturas-mensal = Mensual
planejamento-assinaturas-anual = Anual
planejamento-assinaturas-desde-anterior = { $pct } desde la anterior
planejamento-assinaturas-proxima = Próxima
planejamento-assinaturas-situacao = Estado
planejamento-assinaturas-vazio = Ninguna suscripción
planejamento-assinaturas-vazio-titulo = Todavía nada se repite cada mes en los movimientos
planejamento-assinaturas-vazio-descricao = Las suscripciones aparecen aquí solas después de tres cobros mensuales parecidos en el mismo comercio (o dos anuales). Importa los extractos y resúmenes de tarjeta para encontrar las tuyas.
planejamento-assinaturas-titulo = Suscripciones y cobros recurrentes
planejamento-parcelas-titulo = Cuotas en curso
planejamento-parcelas-vazio-titulo = Ninguna cuota en curso
planejamento-parcelas-vazio = Cuando la descripción de un movimiento dice que es una cuota ("Cuota 3/10", "PARC 02/12", "PREST.EMPREST 003/005"), la app muestra aquí cuánto falta y cuándo termina.
planejamento-parcelas-descricao = Compra o contrato
planejamento-parcelas-parcela = Cuota
planejamento-parcelas-valor = Monto de la cuota
planejamento-parcelas-falta = Falta pagar
planejamento-parcelas-termina = Termina en
planejamento-parcelas-total = Falta en total
planejamento-parcelas-proximo-mes = El próximo mes
planejamento-parcelas-ignorar = No es cuota
planejamento-parcelas-ignorada = Serie quitada de las cuotas
planejamento-parcelas-nao-apareceu = la siguiente no apareció
planejamento-parcelas-tipo-card = Compra con tarjeta
planejamento-parcelas-tipo-loan = Préstamo
planejamento-parcelas-tipo-bill = Cuotas del resumen
planejamento-parcelas-tipo-account = Cuota en la cuenta
planejamento-parcelas-nota = Estimación a partir de la descripción de los movimientos; entra en la previsión de saldo y en los compromisos de la tarjeta. Las compras en cuotas registradas en Treasury y los contratos de Deudas quedan fuera para no contar dos veces.
planejamento-assinaturas-por-ano = Por año
planejamento-assinaturas-detectadas = Detectadas
planejamento-assinaturas-subiu = { $nome } subió { $pct }: de { $antes } a { $agora } el { $data }.
planejamento-assinaturas-cobranca-nova = Cobro nuevo: { $nome }, { $valor } desde el { $data }. Si no lo reconoces, consúltalo con el banco.
planejamento-assinaturas-atrasada = { $nome } se esperaba el { $data } y no llegó: ¿se canceló?
planejamento-assinaturas-tabela = Suscripciones detectadas
planejamento-assinaturas-dispensadas =
    { $n ->
        [one] { $n } detección descartada
       *[other] { $n } detecciones descartadas
    }
planejamento-assinaturas-trazer = Recuperar
planejamento-assinaturas-fonte = Fuente: gastos de los últimos 2 años, sin compras en cuotas ni transferencias. El total por mes no incluye las que no llegaron.

## Escenarios "¿y si…?"

planejamento-cenarios-imovel = Comprar una vivienda
planejamento-cenarios-filho = Tener un hijo
planejamento-cenarios-pausa = Pausar la carrera
planejamento-cenarios-renda = Cambiar de ingresos
planejamento-cenarios-unico = Gasto o ingreso único
planejamento-cenarios-projeto = De un proyecto
planejamento-cenarios-evento-unico = { $valor } el { $data }
planejamento-cenarios-evento-mensal =
    { $meses ->
        [0] { $valor } por mes desde el { $inicio }
       *[other] { $valor } por mes desde el { $inicio } durante { $meses } meses
    }
planejamento-cenarios-evento-renda =
    { $meses ->
        [0] ingresos al { $pct }% desde el { $inicio }
       *[other] ingresos al { $pct }% desde el { $inicio } durante { $meses } meses
    }
planejamento-cenarios-evento-bem =
    { $taxa ->
        [0] bien de { $valor } el { $data }
       *[other] bien de { $valor } el { $data }, { $taxa }% anual real
    }
planejamento-cenarios-evento-financiamento = préstamo de { $valor } en { $meses } meses al { $taxa }% anual real
planejamento-cenarios-compra-imovel = Compra de la vivienda
planejamento-cenarios-imovel-bem = Vivienda
planejamento-cenarios-financiamento = Crédito hipotecario
planejamento-cenarios-filho-evento = Hijo
planejamento-cenarios-pausa-evento = Pausa en la carrera
planejamento-cenarios-renda-em = Ingresos al { $pct }%
planejamento-cenarios-mudanca-renda = Cambio de ingresos
planejamento-cenarios-gasto-unico = Gasto único
planejamento-cenarios-sem-orcamento = { $projeto }: sin presupuesto por gastar
planejamento-cenarios-a-partir = Desde
planejamento-cenarios-preco-imovel = Precio de la vivienda
planejamento-cenarios-entrada = Pago inicial
planejamento-cenarios-prazo = Plazo
planejamento-cenarios-anos = años
planejamento-cenarios-juros-reais = Interés real
planejamento-cenarios-juros-reais-dica = Por encima de la inflación (p. ej.: TR + 9% ≈ 5% real).
planejamento-cenarios-ao-ano = % anual
planejamento-cenarios-data-compra = Fecha de compra
planejamento-cenarios-custo-mes = Costo por mes
planejamento-cenarios-renda-pausa = Ingresos durante la pausa
planejamento-cenarios-renda-pausa-dica = 0% = sin ingresos; 50% = medio tiempo.
planejamento-cenarios-quanto-tempo = Por cuánto tiempo
planejamento-cenarios-meses = meses
planejamento-cenarios-novo-emprego = Nuevo empleo
planejamento-cenarios-exemplo-unico = Viaje, herencia…
planejamento-cenarios-diferenca-mes = Diferencia por mes
planejamento-cenarios-negativo-dica = Negativo para salidas (p. ej.: -5.000,00).
planejamento-cenarios-projeto-campo = Proyecto
planejamento-cenarios-projeto-dica = El presupuesto que falta gastar sale en la fecha de fin.
planejamento-cenarios-escolha = Elige
planejamento-cenarios-nenhum-projeto = Ningún proyecto
planejamento-cenarios-sem-data-fim = Si no tiene fecha de fin
planejamento-cenarios-adicionar = Agregar al escenario
planejamento-cenarios-patrimonio-base = Patrimonio (base)
planejamento-cenarios-patrimonio-cenario = Patrimonio (escenario)
planejamento-cenarios-liquido-cenario = Cuentas e inversiones (escenario)
planejamento-cenarios-base = Base
planejamento-cenarios-cenario = Escenario
planejamento-cenarios-patrimonio-em =
    { $anos ->
        [one] Patrimonio en { $anos } año
       *[other] Patrimonio en { $anos } años
    }
planejamento-cenarios-base-valor = base { $valor } · { $diferenca }
planejamento-cenarios-liquido = Cuentas e inversiones
planejamento-cenarios-acabam = se agotan en { $mes } · { $diferenca }
planejamento-cenarios-ja-negativos = ya están en negativo hoy · { $diferenca }
planejamento-cenarios-nao-acabam = no se agotan en el período · { $diferenca }
planejamento-cenarios-aposentadoria = Jubilación
# El texto entre ** va más pequeño, en gris.
planejamento-cenarios-chance = { $pct }% **de probabilidad**
planejamento-cenarios-aposentadoria-base = base { $pct }% · a los { $idade }: { $valor }
planejamento-cenarios-sem-idade = Completa la edad en Presupuesto › Jubilación para ver el efecto.
planejamento-cenarios-grafico-patrimonio = Patrimonio: escenario frente a base
planejamento-cenarios-fonte-patrimonio = Fuente: promedio de los últimos 12 meses, saldo y cartera de hoy, eventos del escenario · valores reales
planejamento-cenarios-grafico-patrimonio-aria = Patrimonio en el escenario y en la base
planejamento-cenarios-tabela-patrimonio = Patrimonio por mes
planejamento-cenarios-mes = Mes
planejamento-cenarios-grafico-saldo = Próximos 12 meses: saldo de las cuentas
planejamento-cenarios-saldo-negativo = El saldo queda en negativo el { $data } en el escenario
planejamento-cenarios-saldo-positivo = El saldo de las cuentas no queda en negativo en el escenario
planejamento-cenarios-grafico-saldo-aria = Previsión de saldo en el escenario y en la base
planejamento-cenarios-tabela-saldo = Previsión de saldo con el escenario
planejamento-cenarios-aviso = Simulación, no es una recomendación financiera.
planejamento-cenarios-titulo = Escenarios "¿y si…?"
planejamento-cenarios-intro = Arma un escenario con eventos de vida y compáralo con lo que pasa si nada cambia: el patrimonio a lo largo de los años, el saldo de las cuentas en los próximos 12 meses y la probabilidad de que la jubilación salga bien.
planejamento-cenarios-salvos = Guardados:
planejamento-cenarios-apagar = Eliminar { $nome }
planejamento-cenarios-evento = Evento de vida
planejamento-cenarios-horizonte = Horizonte
planejamento-cenarios-nome = Nombre del escenario
planejamento-cenarios-nome-exemplo = Departamento en 2027
planejamento-cenarios-salvo = Escenario guardado
planejamento-cenarios-limpar = Limpiar
planejamento-assinaturas-tipo = Tipo
planejamento-assinaturas-conta-fixa = Cuenta fija
planejamento-assinaturas-tipo-assinatura = Suscripción
planejamento-assinaturas-contas-fixas-mes = Cuentas fijas por mes
