# FGTS (núcleo: `fgts`; docs/brazil/fgts.md): extracto de la Caixa, controles, retiros, retiro de
# cumpleaños, proyección y comparación del rendimiento.

nucleo-fgts-empregador = empleador
nucleo-fgts-tipo-deposito = Depósito
nucleo-fgts-tipo-deposito-extra = Depósito complementario
nucleo-fgts-tipo-jam = JAM (intereses y corrección)
nucleo-fgts-tipo-ajuste-jam = Ajuste del JAM
nucleo-fgts-tipo-distribuicao = Distribución de resultados
nucleo-fgts-tipo-saque = Retiro
nucleo-fgts-tipo-reposicao = Reposición de retiro
nucleo-fgts-saque-motivo-rescisao = despido
nucleo-fgts-saque-motivo-aposentadoria = jubilación
nucleo-fgts-saque-motivo-moradia = vivienda
nucleo-fgts-saque-motivo-aniversario = retiro de cumpleaños
nucleo-fgts-saque-motivo-doenca = enfermedad
nucleo-fgts-saque-motivo-emergencial = retiro de emergencia
nucleo-fgts-saque-motivo-outros = otros
nucleo-fgts-lancamento-saque = Retiro del FGTS ({ $motivo })

nucleo-fgts-motivo-extrato = Importar extracto del FGTS
nucleo-fgts-motivo-saque = Registrar retiro del FGTS
nucleo-fgts-motivo-moradia = Usar el FGTS en el préstamo de vivienda
nucleo-fgts-motivo-modalidade = Cambiar la modalidad de retiro del FGTS
nucleo-fgts-motivo-conta = Editar la cuenta FGTS

nucleo-fgts-erro-extrato-vazio = No encontré movimientos del FGTS en este extracto (líneas con fecha, valor y total).
nucleo-fgts-erro-sem-arquivo = Envía el PDF del extracto del FGTS o su texto.
nucleo-fgts-erro-pdf = No pude leer el PDF del extracto: { $erro }
nucleo-fgts-erro-nao-e-fgts = La cuenta elegida no es una cuenta FGTS.
nucleo-fgts-erro-renda = fuente de ingresos
nucleo-fgts-erro-motivo = Motivo de retiro desconocido: { $motivo }.
nucleo-fgts-erro-valor-positivo = El valor tiene que ser mayor que cero.
nucleo-fgts-erro-saldo-insuficiente = El valor supera el saldo de la cuenta FGTS en esa fecha.
nucleo-fgts-erro-contrato = contrato
nucleo-fgts-erro-contrato-imobiliario = El FGTS solo se puede usar en un contrato de financiación de vivienda.
nucleo-fgts-erro-modalidade = La modalidad tiene que ser retiro por despido o retiro de cumpleaños.
nucleo-fgts-erro-membro = miembro
nucleo-fgts-erro-nascimento = Para proyectar hasta una edad, registra la fecha de nacimiento del titular.
nucleo-fgts-erro-data-ou-idade = Indica la fecha o la edad de la proyección.

nucleo-fgts-aviso-saldo = { $n ->
    [one] Una línea del extracto no coincide con el saldo impreso.
   *[other] { $n } líneas del extracto no coinciden con el saldo impreso.
}
nucleo-fgts-aviso-jam = { $n ->
    [one] Un crédito de JAM quedó distinto del calculado por la regla.
   *[other] { $n } créditos de JAM quedaron distintos del calculado por la regla.
}
nucleo-fgts-aviso-juros-progressivos = La cuenta tiene intereses progresivos (optante antes de 1971): la app guarda la tasa, pero no calcula los intereses progresivos.
nucleo-fgts-aviso-codigo-99 = Los retiros con código 99 se trataron como uso en vivienda; cambia el motivo si no lo fueron.
nucleo-fgts-aviso-cabecalho = No se pudo leer todo el encabezado del extracto (empleador, CNPJ o fechas del vínculo). Revisa esos datos en la cuenta después de importar.
nucleo-fgts-aviso-dois-anos = Hubo uso del FGTS en vivienda el { $data }: la amortización con el FGTS exige al menos 2 años entre un uso y otro (Ley 8.036, art. 20).

nucleo-fgts-nota-aniversario = Estimación por la tabla del retiro de cumpleaños (Ley 8.036, Anexo): una sola franja, por el saldo sumado de las cuentas. El cambio se pide en la app FGTS de la Caixa.
nucleo-fgts-nota-antecipacao = Con anticipación contratada, la vuelta al retiro por despido solo vale después de pagar la anticipación.
nucleo-fgts-nota-conferencia = El JAM se recalcula por el saldo hasta el crédito anterior, con el coeficiente del extracto, truncado en centavos; los depósitos se comparan con el FGTS de las nóminas del ingreso vinculado.
nucleo-fgts-nota-extrato-recente = Los meses sin depósito pueden estar solo fuera del último extracto importado: importa un extracto reciente antes de reclamar al empleador.
nucleo-fgts-nota-projecao = Proyección con la última TR ({ $tr }% al mes) constante, intereses del 3% anual, distribución por el promedio de los últimos índices e inflación de { $ipca }% en 12 meses para los valores en reales de hoy.
nucleo-fgts-nota-sem-renda = Cuenta activa sin ingreso vinculado: la proyección no suma depósitos nuevos.
nucleo-fgts-nota-comparar = Los mismos depósitos y retiros, en las mismas fechas, aplicados en cada destino. El 85% del CDI es solo una referencia para algo exento de impuesto a la renta, como el FGTS. No es una recomendación.
nucleo-fgts-nota-adi = Por la ADI 5090 (STF, 12/06/2024), la remuneración del FGTS (JAM + distribución) debe ser al menos el IPCA desde entonces, sin efecto retroactivo; la compensación la define el Consejo Curador.
nucleo-fgts-nota-sem-serie = Falta en la bóveda alguna serie del Banco Central (IPCA, ahorro, CDI o M2): sincroniza los índices para completar la comparación.

nucleo-fgts-comparar-ipca = Corregido por el IPCA
nucleo-fgts-comparar-poupanca = Ahorro (poupança)
nucleo-fgts-comparar-cdi-85 = 85% del CDI
nucleo-fgts-comparar-cdi = 100% del CDI
nucleo-fgts-comparar-m2 = Expansión del M2
nucleo-fgts-evento-aposentadoria = Retiro del FGTS en la jubilación
nucleo-fgts-nota-aposentadoria = El saldo FGTS proyectado ({ $valor } en reales de hoy) entra como retiro en la jubilación.
