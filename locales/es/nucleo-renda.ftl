# Ingresos de la familia y recibos de nómina (núcleo, `income`; docs/income.md): pagos extra de
# la previsión, avisos de la previsión y de la lectura del recibo, motivos del historial y
# errores. Las etiquetas entre comillas son las del documento (recibo brasileño) y quedan como
# están.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

nucleo-renda-erro-renda = ingreso { $id }
nucleo-renda-erro-bruto-negativo = el monto bruto no puede ser negativo
nucleo-renda-erro-liquido-negativo = el monto neto no puede ser negativo
nucleo-renda-erro-liquido-maior = el neto no puede ser mayor que el bruto
nucleo-renda-erro-dia-pagamento = día de pago del 1 al 31
nucleo-renda-erro-mes-ferias = mes de vacaciones del 1 al 12
nucleo-renda-erro-termina-antes = el ingreso termina antes de empezar

## Pagos extra (previsión y recibos que no son el mensual)

nucleo-renda-extra-13-1 = 1.ª cuota del 13º
nucleo-renda-extra-13-2 = 2.ª cuota del 13º
nucleo-renda-extra-terco-ferias = tercio de vacaciones
nucleo-renda-extra-subsidio-natal = subsidio de Navidad
nucleo-renda-extra-subsidio-ferias = subsidio de vacaciones
nucleo-renda-extra-paga-junho = paga extra de junio
nucleo-renda-extra-paga-dezembro = paga extra de diciembre
nucleo-renda-extra-ferias = vacaciones
nucleo-renda-extra-plr = participación en las ganancias (PLR)
nucleo-renda-extra-adiantamento = adelanto
nucleo-renda-extra-outro = otro pago

## Avisos de la previsión

nucleo-renda-nota-sem-valor = { $renda }: ingresa el bruto o el neto (o importa un recibo de nómina) para que entre en la previsión.
nucleo-renda-nota-br = INSS e IRRF calculados con las reglas del año cuando no vienen del recibo; la pensión alimenticia, el seguro de salud y otros descuentos solo entran por el recibo.
nucleo-renda-nota-us = FICA e impuesto federal estimados con las reglas de 2026 cuando no vienen del paystub, con la deducción estándar y el estado civil del registro de la familia (individual, cabeza de familia o declaración conjunta); el impuesto estatal, el 401(k) y el seguro de salud solo entran por el paystub.
nucleo-renda-nota-pt = Segurança Social e IRS estimados con las reglas de 2026 cuando no vienen del recibo de sueldo (el IRS por el impuesto anual de los tramos, no por las tablas de retención).
nucleo-renda-nota-es = Seguridad Social e IRPF estimados con las reglas de 2026 cuando no vienen de la nómina (el IRPF por la escala general, sin la escala propia de cada comunidad autónoma).
nucleo-renda-extra-aguinaldo = aguinaldo (SAC)
nucleo-renda-nota-adaptado = Aportes e impuesto estimados con las reglas de 2026 del país cuando no vienen del recibo de sueldo (una aproximación: sin deducciones ni créditos personales más allá de los básicos).
nucleo-renda-nota-outro = Los descuentos del sueldo no se calculan para este país: ingresa el neto o importa el recibo.

## Historial de cambios

nucleo-renda-motivo-criada = Ingreso registrado: { $renda }
nucleo-renda-motivo-alterada = Ingreso modificado: { $renda }
nucleo-renda-motivo-apagada = Ingreso eliminado: { $renda }
nucleo-renda-motivo-holerite-importado = Recibo de nómina importado
nucleo-renda-motivo-holerite-apagado = Recibo de nómina eliminado

## Lectura e importación del recibo

nucleo-renda-aviso-sem-competencia = no encontré el período (mes/año) del recibo: ingrésalo al importar
nucleo-renda-aviso-nao-bate = el bruto menos los descuentos ({ $calculado }) no coincide con el neto ({ $liquido }): revisa las líneas
nucleo-renda-aviso-sem-vencimentos = no encontré los haberes del recibo
nucleo-renda-aviso-escolha-renda = elige el ingreso o el miembro (para crear el ingreso de este empleador)
nucleo-renda-erro-base64 = { $arquivo }: base64 inválido
nucleo-renda-erro-imagem = { $arquivo }: es una imagen, no un PDF. Lee los valores de la imagen y envía el texto del recibo en `text` (las líneas con etiqueta y valor, como "Líquido a receber 2.043,36")
nucleo-renda-erro-pdf-senha = PDF protegido con contraseña: ingresa la contraseña del recibo
nucleo-renda-erro-pdf-ilegivel = no se pudo leer el PDF: { $erro }
nucleo-renda-erro-sem-arquivo = envía el PDF del recibo (o el texto)
nucleo-renda-falta-bruto = el total bruto ("Total de vencimentos", "Total de proventos" o "Total bruto")
nucleo-renda-falta-liquido = el neto ("Líquido a receber" o "Valor líquido")
nucleo-renda-faltam-dois = { $um } ni { $outro }
nucleo-renda-erro-incompleto = el texto parece un recibo de nómina, pero no encontré { $faltou }. Envía el texto completo del documento, sin resumir ni reescribir (las líneas con etiqueta y valor, como "Líquido a receber 2.043,36"), o el PDF adjunto en `attachments` (ej.: ["1"])
nucleo-renda-erro-competencia = ingresa el período del recibo (YYYY-MM)
nucleo-renda-erro-ilegivel = el recibo no tiene bruto y neto legibles
nucleo-renda-erro-escolha-renda = elige el ingreso o el miembro del recibo
nucleo-renda-erro-duplicado = ya existe el recibo de { $mes } de este ingreso; usa reemplazar para cambiarlo
nucleo-renda-erro-holerite = recibo de nómina { $id }
# Nombre del ingreso creado al importar un recibo de un empleador nuevo.
nucleo-renda-nome-salario-de = Sueldo · { $empregador }
nucleo-renda-nome-salario = Sueldo
nucleo-renda-aviso-sem-sinal = { $linhas } sin signo de descuento, pero los totales impresos solo cuadran con ellas como descuentos: revisa
nucleo-renda-aviso-totais = las líneas no cuadran con los totales impresos (percepciones { $vencimentos } × { $bruto }, descuentos { $descontos } × { $total_descontos }): valen los totales, revisa las líneas
nucleo-renda-mensal = mensual
nucleo-renda-aviso-plr-irrf = IR retenido de la PLR ({ $retido }) distinto del de la tabla exclusiva de la PLR ({ $esperado }), contando las PLR anteriores del año. Revísalo (la pensión alimenticia descontada reduce el impuesto).

## Simulación de despido (`income.simulate_termination`)

nucleo-renda-erro-rescisao-data = `{ $campo }` debe ser una fecha AAAA-MM-DD
nucleo-renda-erro-rescisao-salario = indica el salario bruto o elige un ingreso con salario bruto
nucleo-renda-erro-rescisao-admissao = indica la fecha de contratación
nucleo-renda-erro-rescisao-ordem = la desvinculación no puede ser antes de la contratación
nucleo-renda-erro-rescisao-fim-contrato = indica el fin acordado del contrato de prueba
nucleo-renda-erro-rescisao-tabela = no hay tabla de INSS e IRRF para el mes de la desvinculación
nucleo-renda-rescisao-saldo-salario = Saldo de salario
nucleo-renda-rescisao-aviso-previo = Preaviso indemnizado
nucleo-renda-rescisao-decimo-terceiro = 13º salario proporcional
nucleo-renda-rescisao-ferias-proporcionais = Vacaciones proporcionales + 1/3
nucleo-renda-rescisao-ferias-vencidas = Vacaciones vencidas + 1/3
nucleo-renda-rescisao-indenizacao-479 = Indemnización del contrato de prueba (art. 479)
nucleo-renda-rescisao-desconto-480 = Descuento por terminar antes el contrato de prueba (art. 480)
nucleo-renda-rescisao-desconto-aviso = Descuento del preaviso no cumplido
nucleo-renda-rescisao-inss = INSS sobre el saldo de salario
nucleo-renda-rescisao-irrf = IRRF sobre el saldo de salario
nucleo-renda-rescisao-inss-13 = INSS sobre el 13º
nucleo-renda-rescisao-irrf-13 = IRRF sobre el 13º
nucleo-renda-rescisao-n-dias = { $n } días
nucleo-renda-rescisao-n-dias-terco = { $n } días + 1/3
nucleo-renda-rescisao-metade-dias = la mitad de { $n } días
nucleo-renda-rescisao-aviso-estimativa = Estimación con las reglas generales de la CLT. El valor oficial es el del finiquito (TRCT) del empleador; el convenio colectivo, los promedios de horas extra y comisiones, las faltas y los anticipos pueden cambiar el resultado.
nucleo-renda-rescisao-aviso-13-adiantado = Si la 1ª cuota del 13º ya se pagó, se descuenta del 13º del finiquito.
nucleo-renda-rescisao-aviso-sem-fgts = Sin el saldo del FGTS de este empleo, la multa y el retiro quedan solo sobre el depósito del mes. Indica el saldo o vincula la cuenta del FGTS al ingreso.
nucleo-renda-rescisao-aviso-aniversario = En la modalidad de retiro por aniversario el saldo del FGTS queda retenido en el despido; solo se puede retirar la multa.
nucleo-renda-rescisao-seguro-motivo = El seguro de desempleo es solo para despido sin justa causa.
nucleo-renda-rescisao-seguro-carencia = Sin derecho: en la solicitud n.º { $pedido } se necesitan { $meses } meses trabajados en los últimos { $janela }.
nucleo-renda-rescisao-tabela-antiga = fecha anterior a la tabla más antigua de la app
