# Salud financiera: tarjeta 08 del Panel, Informes › Salud financiera y Ajustes › Salud
# financiera (docs/i18n-design.md). Las franjas, dimensiones, indicadores, notas y sugerencias
# vienen del núcleo. Revisión: borrador por IA, sin revisión de un hablante nativo.

## Modelos y estados

saude-modelo-classicos = Indicadores clásicos
saude-modelo-desativado = Desactivado
saude-status-bom = Bueno
saude-status-atencao = Atención
saude-status-ruim = Malo
saude-status-sem-dados = Sin datos

## Escala y barras (nombres para lectores de pantalla)

saude-faixas-aria = Franjas de 0 a 100: { $faixas }
saude-barra-sem-dados = { $rotulo }: sin datos
saude-barra-oculto = { $rotulo }: valor oculto
saude-barra-pontos = { $rotulo }: { $pontos } de 100

## Tarjeta del panel

saude-titulo = Salud financiera
saude-modelo = Modelo
saude-janela = Ventana
saude-n-meses =
    { $n ->
        [one] { $n } mes
       *[other] { $n } meses
    }
saude-avaliado = Evaluado
saude-ver-indicadores = Ver indicadores
saude-nota-isf = Nota (estimación I-SFB)
saude-nota = Nota
saude-origem-isf = Estimación al estilo del I-SFB (Febraban y Banco Central) con tus movimientos, cuotas, resúmenes de tarjeta y saldos
saude-origem-classicos = Indicadores clásicos calculados con tus movimientos, cuotas, resúmenes de tarjeta y saldos
saude-dimensoes = Dimensiones
saude-indicadores = Indicadores

## Informes › Salud financiera

saude-trocar-modelo = Cambiar el modelo
saude-carregando = Midiendo la salud financiera
saude-desativada = Desactivada
saude-desligada-titulo = Análisis de salud financiera apagado
saude-desligada-descricao = Elige un modelo en Ajustes › Salud financiera para ver la nota, las dimensiones y las sugerencias.
saude-abrir-ajustes = Abrir ajustes
saude-indicadores-descricao = Cada razón con la meta habitual de la educación financiera y su peso en la nota.
saude-titulo-modelo = Salud financiera · { $modelo }
saude-periodo = { $de } a { $ate }
saude-sem-lancamentos = sin movimientos
saude-meses = Meses
saude-cambio = Tipo de cambio
saude-cambio-estimado = ≈ estimado
saude-contas-paradas = Cuentas
saude-contas-paradas-valor = { $n ->
    [one] ≈ 1 sin extracto reciente
   *[other] ≈ { $n } sin extracto reciente
}
saude-nota-0-100 = Nota de 0 a 100
saude-faixas = Franjas
saude-sobre-isf = Acerca del I-SFB (Meu Bolso em Dia, Febraban)
saude-medido = Medido
saude-sem-dados = sin datos
saude-meta = Meta
saude-pontos = Puntos
saude-peso-nota = · peso { $peso }% de la nota
saude-peso-dimensao = · peso { $peso }% de la dimensión
saude-calculo = Cálculo
saude-para-medir = Para medir
saude-sugestoes = Sugerencias

## Ajustes › Salud financiera

saude-desativada-aviso = Salud financiera desactivada
saude-modelo-aviso = Modelo: { $modelo }
saude-modelo-titulo = Modelo de salud financiera
saude-como-calculada = Cómo se calcula la nota
saude-calculo-dados = La nota usa solo los datos de la bóveda: ingresos y gastos de los últimos 12 meses completos (las transferencias, los pagos de resumen y los aportes no cuentan), cuotas de préstamos, resúmenes de tarjeta, saldos de las cuentas, presupuesto y patrimonio. Nada sale de la computadora.
# El texto entre ** va en cursiva.
saude-calculo-sem-dados = Un indicador sin historial suficiente (menos de 3 meses) queda **sin datos** y sale del promedio, en lugar de bajar la nota.
saude-isf-resumo = comportamiento, habilidad, libertad y seguridad; promedio nacional 56,7 (2024)
saude-classicos = Clásicos
saude-classicos-resumo = ahorro, fondo de emergencia, cuotas, vivienda, crédito rotativo y patrimonio
saude-fonte-isf = Índice de Salud Financiera del Brasileño 2024 (Meu Bolso em Dia)
