# Pantalla 07 · Deudas: contratos, cronograma, gráficos, simuladores, pago de cuotas, extracto y
# tasas del mercado (docs/i18n-design.md). Los resúmenes y avisos del núcleo no están aquí.
# Revisión: borrador por IA, sin revisión nativa.

## Etiquetas repetidas

dividas-contrato = Contrato
dividas-contrato-min = contrato
dividas-um-contrato = un contrato
dividas-contratos = Contratos
dividas-novo-contrato = Nuevo contrato
dividas-saldo-devedor = Saldo deudor
dividas-proxima-parcela = Próxima cuota
dividas-parcela = Cuota
dividas-parcelas = Cuotas
dividas-juros = Intereses
dividas-juros-a-pagar = Intereses por pagar
dividas-amortizacao = Amortización
dividas-amortizacao-extraordinaria = Amortización extraordinaria
dividas-com-simulada = Con la amortización simulada
dividas-correcao = Corrección
dividas-correcao-saldo = Corrección del saldo
dividas-seguros-taxas = Seguros y comisiones
dividas-sistema = Sistema
dividas-taxa = Tasa
dividas-data = Fecha
dividas-valor = Monto
dividas-ano = Año
dividas-hoje = Hoy
dividas-vencimento = Vencimiento
dividas-situacao = Estado
dividas-diferenca = Diferencia
dividas-acoes = Acciones
dividas-col-numero = N.º
dividas-data-pagamento = Fecha de pago
dividas-conta-de-debito = Cuenta de débito
dividas-conta-de-debito-min = cuenta de débito
dividas-categoria-pagamentos = Categoría de los pagos
dividas-custos-troca = Costos del cambio
dividas-parcelas-que-faltam = Cuotas que faltan
dividas-prazo-meses = Plazo (meses)
dividas-calculando = Calculando…
dividas-sem-descricao = sin descripción
dividas-opcional = Opcional.
dividas-nenhuma = Ninguna
dividas-nenhuma-min = ninguna
dividas-arquivado = archivado
dividas-arquivar = Archivar
dividas-reativar = Reactivar
dividas-excluir = Eliminar
dividas-desligar = Desvincular
dividas-estimado = ≈ estimado
dividas-quitado = Liquidado
dividas-quitado-min = liquidado
dividas-atrasada-alerta = ! atrasada
dividas-parcela-n-data = Cuota { $n } · { $data }
dividas-parcela-original = Cuota { $n } del contrato original
dividas-n-meses =
    { $n ->
        [one] { $n } mes
       *[other] { $n } meses
    }

## Tasas (anual, mensual)

dividas-taxa-aa = { $taxa } anual
dividas-taxa-am = { $taxa } mensual
dividas-taxa-linear = { $taxa } (lineal)
dividas-taxa-pct-cdi = { $pct } del CDI
dividas-taxa-pct-selic = { $pct } de la Selic

## Plazo y estado de las cuotas

dividas-prazo-parcelas =
    { $n ->
        [one] { $n } cuota
       *[other] { $n } cuotas
    }
dividas-prazo-longo =
    { $meses ->
        [0] { $n } cuotas ({ $anos } años)
        [one] { $n } cuotas ({ $anos } años y { $meses } mes)
       *[other] { $n } cuotas ({ $anos } años y { $meses } meses)
    }
dividas-status-paga = Pagada
dividas-status-atrasada = Atrasada
dividas-status-proxima = Próxima
dividas-status-prevista = Prevista
dividas-parcelas-atrasadas =
    { $n ->
        [one] { $n } cuota atrasada
       *[other] { $n } cuotas atrasadas
    }
dividas-igual = igual
dividas-pago-a-mais = Pagado de más respecto a la cuota
dividas-pago-a-menos = Pagado de menos respecto a la cuota

## Tipos de contrato y enums del núcleo

dividas-tipo-imobiliario = Crédito hipotecario
dividas-tipo-veiculo = Crédito automotriz
dividas-tipo-pessoal = Préstamo personal
dividas-tipo-consignado = Préstamo con descuento de nómina
dividas-tipo-estudantil = Crédito educativo
dividas-tipo-outro = Otro

dividas-sistema-sac-dica = Amortización constante: la cuota empieza más alta y baja con el tiempo.
dividas-sistema-price-dica = Cuota constante (sin índice): al principio casi todo es interés. Con TR o IPCA, el saldo se corrige antes y la cuota se recalcula cada mes.
dividas-sistema-bullet-dica = Solo intereses en cada vencimiento y todo el capital en la última cuota.
dividas-sistema-sac-dica-pos = Amortización constante + intereses del período por el índice: la cuota varía.
dividas-sistema-price-dica-pos = Cuota recalculada en cada vencimiento con el índice del período.
dividas-sistema-bullet-dica-pos = Solo los intereses del período en cada vencimiento; el capital va al final.

dividas-indexador-nenhum = Tasa fija
dividas-indexador-poupanca = Ahorro (poupança)

dividas-prorata-nenhum = Sin prorrateo (mes completo)
dividas-prorata-corridos = Por días corridos
dividas-prorata-uteis = Por días hábiles

dividas-base-mensal = Mensual equivalente
dividas-base-252 = Días hábiles (base 252)
dividas-base-365 = Días corridos (base 365)

dividas-referencia-mes = Variación del mes
dividas-referencia-aniversario = Fecha de aniversario

dividas-spread-exponencial = Exponencial (252 días hábiles)
dividas-spread-linear = Lineal (360 días corridos)

dividas-modo-prazo = Reducir plazo
dividas-modo-parcela = Reducir cuota
dividas-modo-prazo-dica = Mantiene la cuota y acorta el contrato: ahorra más intereses.
dividas-modo-parcela-dica = Mantiene el plazo y baja las próximas cuotas: alivia el mes.

## Pantalla y menú del contrato

dividas-menu-abrir = Abrir contrato
dividas-menu-amortizacao-extra = Amortización extra…
dividas-menu-editar = Editar…
dividas-menu-excluir = Eliminar…
dividas-conciliar = Conciliar con los pagos
dividas-conciliar-sem-conta = Define la cuenta de débito o la categoría de los pagos para conciliar
dividas-carregando-contratos = Cargando los contratos
dividas-vazio-kicker = Sin contratos
dividas-vazio-titulo = Registra el primer préstamo o crédito
dividas-vazio-descricao = Ingresa monto, tasa, sistema (SAC o Price), plazo e índice: el cronograma sale al instante, con intereses frente a amortización, simulador de amortización extraordinaria y el costo real descontada la inflación.
dividas-mostrar-arquivados = Mostrar archivados

## Resumen y lista de contratos

dividas-resumo-saldo-total = Saldo deudor total
dividas-resumo-contratos-ativos =
    { $n ->
        [one] { $n } contrato activo
       *[other] { $n } contratos activos
    }
dividas-resumo-compromisso = Compromiso mensual
dividas-resumo-proximas-somadas = Próximas cuotas sumadas
dividas-resumo-em-atraso = ! { $n } atrasadas
dividas-resumo-ate-o-fim = Hasta el fin de los contratos
dividas-resumo-pelo-cronograma = según el cronograma actual
dividas-acoes-de = Acciones de { $nome }
dividas-col-taxa-aa = Tasa anual
dividas-col-progresso = Progreso
dividas-col-custo-real = Costo real
dividas-numero-contrato = n.º { $numero }
dividas-pre = fija
dividas-pos = variable
dividas-atrasada-em = ! atrasada · { $data }

## Detalle del contrato

dividas-conciliadas =
    { $n ->
        [one] { $n } cuota conciliada
       *[other] { $n } cuotas conciliadas
    }
dividas-conciliar-nada = No se encontraron pagos nuevos
dividas-conciliar-procurado = Se buscó en { $conta }: monto de la cuota ±1% y hasta 7 días del vencimiento.
dividas-conciliar-erro = No se pudo conciliar
dividas-reativado = Contrato reactivado
dividas-arquivado-ok = Contrato archivado
dividas-contrato-aria = Contrato { $nome }
dividas-de-membro = de { $nome }
dividas-contrato-de-data = contrato del { $data }
dividas-contrato-numero-de-data = contrato n.º { $numero } del { $data }
dividas-importar-extrato = Importar extracto
dividas-amortizacao-extra = Amortización extra
dividas-atrasadas-texto =
    { $n ->
        [one] Vencida y sin pago registrado. Si ya pagaste, concilia con los movimientos de la cuenta de débito o márcala como pagada en la tabla.
       *[other] Vencidas y sin pago registrado. Si ya pagaste, concilia con los movimientos de la cuenta de débito o márcalas como pagadas en la tabla.
    }
dividas-marcadas-titulo = Cuotas marcadas como pagadas
dividas-marcadas-descricao =
    { $n ->
        [one] { $n } cuota vencida, en la fecha de vencimiento y por el monto del cronograma.
       *[other] { $n } cuotas vencidas, en la fecha de vencimiento y por el monto del cronograma.
    }
dividas-marcar-vencidas = Marcar las { $n } vencidas como pagadas
dividas-de-financiados = de { $valor } financiados
dividas-parcelas-pagas-de = { $pagas } de { $total } cuotas pagadas
dividas-numero-data = N.º { $n } · { $data }
dividas-em-aberto = { $prazo } pendientes
dividas-taxa-custo = Tasa y costo
dividas-sistema-pre = tasa fija
dividas-sistema-pos = tasa variable, intereses por el { $indice } acumulado en días hábiles
dividas-sistema-pos-defasagem = tasa variable, intereses por el { $indice } acumulado en días hábiles (D−{ $dias })
dividas-sistema-correcao = corrección { $indice } ({ $detalhes })
dividas-correcao-aniversario = en el aniversario
dividas-correcao-mes = del mes
dividas-correcao-piso = piso cero
dividas-correcao-prorata = 1.er período prorrateado
dividas-cet-contratado = { $sigla } del contrato
dividas-custo-realizado = Costo real pagado
dividas-cet-realizado = { $sigla } de lo ya pagado
dividas-custo-real-ipca = Costo real (− IPCA)
dividas-juros-pagos = Intereses pagados
dividas-seguros-taxas-pagos = Seguros y comisiones pagados
dividas-montando-cronograma = Armando el cronograma
dividas-simulador-amortizacao = Simulador de amortización extraordinaria
dividas-antecipar-parcelas = Adelantar cuotas
dividas-simulador-portabilidade = Simulador de portabilidad
dividas-tabela-amortizacao = Tabla de amortización
dividas-seguros-taxas-mes = Seguros + comisiones/mes
dividas-origem = Origen
dividas-origem-valor = cronograma del núcleo · ≈ índice estimado
dividas-observacoes-valor = Observaciones: { $notas }
dividas-excluir-titulo = Eliminar { $nome }
dividas-excluir-descricao = Se eliminan el contrato, el cronograma y las amortizaciones registradas. Los movimientos de las cuentas se mantienen. Se puede deshacer desde el historial de cambios.
dividas-excluir-confirmar = Eliminar contrato
dividas-excluido = Contrato eliminado

## Gráficos

dividas-grafico-saldo-estimado = Saldo deudor (≈ índice estimado)
dividas-grafico-parcelas-n = Cuotas ({ $n })
dividas-grafico-parcela-estimada = Cuota { $n } · { $data } · ≈ índice estimado
dividas-grafico-correcao-nota = La corrección no sale en la cuota: se suma al saldo deudor.
dividas-figura-saldo-titulo = Saldo deudor a lo largo del contrato
dividas-figura-saldo-fonte = Cronograma del contrato
dividas-figura-saldo-fonte-indice = Cronograma del contrato · corrección por el índice (≈ los meses sin dato oficial usan el último índice)
dividas-figura-saldo-aria = Saldo deudor de { $nome } a lo largo del contrato.
dividas-figura-saldo-aria-simulada = Saldo deudor de { $nome } a lo largo del contrato, con la amortización simulada.
dividas-figura-saldo-tabela = Saldo deudor por vencimiento
dividas-figura-saldo-col = Saldo
dividas-figura-composicao-titulo =
    { $modo ->
        [ano] Intereses × amortización por año
       *[parcela] Intereses × amortización por cuota
    }
dividas-figura-composicao-fonte-correcao = La corrección del saldo por el índice no se paga en la cuota: entra en el saldo deudor y se amortiza después.
dividas-figura-composicao-fonte = Composición de cada cuota del cronograma.
dividas-figura-composicao-agrupar = Agrupar la composición
dividas-figura-composicao-aria =
    { $modo ->
        [ano] Composición de las cuotas de { $nome } por año: { $partes }.
       *[parcela] Composición de las cuotas de { $nome } por cuota: { $partes }.
    }
dividas-figura-composicao-tabela =
    { $modo ->
        [ano] Composición por año
       *[parcela] Composición por cuota
    }
dividas-figura-pago-no-ano = Pagado en el año

## Tasas del mercado (Banco Central)

dividas-mercado-titulo = Tasas del mercado
dividas-mercado-intro = Cuánto cobran los bancos en cada tipo de crédito, según los datos del Banco Central de Brasil. Sirve para saber si tu contrato es caro y si vale la pena negociar o llevar la deuda a otro banco (portabilidad).
dividas-mercado-abaixo = Por debajo del mercado
dividas-mercado-media = En el promedio del mercado
dividas-mercado-acima = Por encima del mercado
dividas-mercado-bem-acima = Muy por encima del mercado
dividas-mercado-seus-contratos = Tus contratos × mercado
dividas-mercado-taxas = Tasas del mercado
dividas-mercado-credito = Crédito
dividas-mercado-mais-barato = Más barato (mensual)
dividas-mercado-mediana = Mediana (mensual)
dividas-mercado-mais-caro = Más caro (mensual)
dividas-mercado-bancos = Bancos más baratos
dividas-mercado-ao-ano = { $taxa } al año
dividas-mercado-atualizado = Actualizado el { $data }.
dividas-mercado-atualizar = Actualizar

## Adelantar cuotas

dividas-antecipar-dica-ultimas = Liquida las últimas: el contrato termina antes y el descuento es mayor.
dividas-antecipar-dica-proximas = Paga ahora las próximas: el descuento es menor y el plazo no cambia.
dividas-antecipar-em-aberto = Pendientes: { $n }
dividas-antecipar-ordem = Orden
dividas-antecipar-ordem-aria = Orden del adelanto
dividas-antecipar-ultimas = Desde las últimas
dividas-antecipar-proximas = Desde las próximas
dividas-antecipar-informe = Indica cuántas cuotas adelantar.
dividas-antecipar-resumo = Pagando { $valor } el { $data }, el descuento de intereses es de { $desconto }.
dividas-antecipar-resumo-encargos = Pagando { $valor } el { $data }, el descuento de intereses es de { $desconto } y dejan de cobrarse { $encargos } de seguros y comisiones.
dividas-antecipar-valor-cronograma = Monto en el cronograma
dividas-antecipar-a-pagar = A pagar ahora
dividas-antecipar-ficam = Quedan pendientes
dividas-antecipadas = Cuotas adelantadas
dividas-antecipadas-descricao =
    { $n ->
        [one] { $n } cuota por { $valor }
       *[other] { $n } cuotas por { $valor }
    }
dividas-antecipar-registrar = Registrar adelanto
dividas-antecipar-com-conta = La salida se registra en la cuenta de débito del contrato.
dividas-antecipar-sem-conta = Sin cuenta de débito: solo las cuotas quedan pagadas.

## Simulador de amortización extraordinaria

dividas-simulador-valor = Monto de la amortización
dividas-simulador-acima-saldo = Por encima del saldo deudor ({ $saldo }): liquida el contrato.
dividas-simulador-saldo-hoje = Saldo deudor hoy: { $saldo }
dividas-simulador-informe = Ingresa un monto para comparar las dos formas de amortizar. La simulación no guarda nada.
dividas-simulador-forma = Forma de amortizar
dividas-simulador-juros-economizados = Intereses ahorrados
dividas-simulador-parcelas-restantes = Cuotas restantes
dividas-simulador-a-menos = { $prazo } menos
dividas-simulador-quitacao = Liquidación
dividas-simulador-nota = Simulación según el cronograma actual. Confirma los montos con el banco antes de pagar.
dividas-simulador-nota-indice = Simulación según el cronograma actual, con el índice proyectado por el último conocido. Confirma los montos con el banco antes de pagar.
dividas-simulador-registrar = Registrar esta amortización

## Simulador de portabilidad

dividas-portabilidade-nova-taxa = Nueva tasa (% anual)
dividas-portabilidade-hoje = Hoy: { $taxa }
dividas-portabilidade-prazo-erro = Entre 1 y 600 meses.
dividas-portabilidade-restam = Quedan { $n }
dividas-portabilidade-sistema-dica = De la nueva deuda.
dividas-portabilidade-sistema-aria = Sistema de la nueva deuda
dividas-portabilidade-custos-dica = Comisiones, avalúo, registro.
dividas-portabilidade-informe = Ingresa la tasa que ofrece el otro banco para comparar. La simulación no guarda nada.
dividas-portabilidade-vale = Vale la pena: ahorro de { $valor } en total, ya con los costos del cambio.
dividas-portabilidade-nao-vale = No conviene: costo extra de { $valor } en total, ya con los costos del cambio.
dividas-portabilidade-saldo-levado = Saldo trasladado
dividas-portabilidade-falta-atual = Falta pagar en el actual
dividas-portabilidade-total-nova = Total en la nueva deuda
dividas-portabilidade-novo-prazo = Nuevo plazo
dividas-portabilidade-nota = La nueva deuda se calcula sin índice (tasa fija). Los contratos actuales con TR o IPCA pueden tener un costo real distinto.

## Vincular una cuota a un movimiento

dividas-ligar-ok = Cuota { $n } vinculada al movimiento
dividas-descricao = Descripción
dividas-conta = Cuenta
dividas-conta-debito-contrato = Cuenta de débito del contrato
dividas-debito = débito
dividas-ligar-titulo = Vincular la cuota { $n } a un movimiento
dividas-ligar-descricao = { $contrato } · vencimiento el { $data } · cuota de { $valor }. Salidas hasta 20 días del vencimiento que todavía no pagan otra cuota, las de monto más cercano primero. La cuota queda pagada en la fecha y por el monto del movimiento.
dividas-ligar-trocar = Cambiar por el movimiento
dividas-ligar-ao-lancamento = Vincular al movimiento
dividas-lancamento-ligado = Movimiento vinculado
dividas-paga-pelo-lancamento = Pagada por el movimiento
dividas-desligada = Cuota { $n } desvinculada del movimiento y pendiente
dividas-desligar-lancamento = Desvincular el movimiento
dividas-ligar-procurando = Buscando las salidas cerca del vencimiento…
dividas-ligar-tabela = Movimientos para la cuota { $n }
dividas-ligar-vazio = Ninguna salida sin cuota hasta 20 días del vencimiento. Importa el extracto de la cuenta o registra el pago en Movimientos.

## Tabla de amortización

dividas-paga-pelo-lancamento-titulo = Pagada por el movimiento { $lancamento }
dividas-paga-pelo-lancamento-titulo-conta = Pagada por el movimiento { $lancamento } en { $conta }
dividas-mais-extra = + extra { $valor }
dividas-juros-periodo = Intereses del período
dividas-juros-periodo-dica = Índice acumulado en los días hábiles del período × spread
dividas-dias-uteis =
    { $n ->
        [one] { $n } día hábil
       *[other] { $n } días hábiles
    }
dividas-encargos-detalhe = MIP { $mip } · DFI { $dfi } · otros seguros { $outros } · comisiones { $taxas }
dividas-pago-valor = pagado { $valor }
dividas-saldo-apos = Saldo después
dividas-paga-em = Pagada el { $data }
dividas-indice-estimado-dica = El índice del mes aún no se publicó: usa el último conocido
dividas-indice-estimado = índice estimado
dividas-ligar-reticencias = Vincular…
dividas-desligar-parcela = Desvincular el movimiento de la cuota { $n }
dividas-desfazer-parcela = Deshacer el pago de la cuota { $n }
dividas-desfazer = Deshacer
dividas-marcar-parcela-aria = Marcar la cuota { $n } como pagada
dividas-marcar-paga = Marcar pagada
dividas-menu-marcar-paga = Marcar como pagada…
dividas-menu-ligar-outro = Vincular a otro movimiento…
dividas-menu-ligar-um = Vincular a un movimiento…
dividas-menu-ver-lancamento = Ver el movimiento
dividas-menu-desfazer = Deshacer el pago
dividas-filtro-aria = Cuotas a mostrar
dividas-filtro-proximas = Próximas
dividas-filtro-abertas = Pendientes · { $n }
dividas-filtro-pagas = Pagadas · { $n }
dividas-filtro-todas = Todas · { $n }
dividas-tabela-de = Tabla de amortización · { $nome }
dividas-acoes-parcela = Acciones de la cuota { $n }
dividas-filtro-vazio = Ninguna cuota en este filtro.

## Marcar una cuota como pagada y deshacer

dividas-paga-sincronizada = Cuota { $n } pagada y saldo sincronizado
dividas-marcada-paga = Cuota { $n } marcada como pagada
dividas-pagar-titulo = Cuota { $n } pagada
dividas-pagar-descricao = Vencimiento el { $data }. Marca la cuota como pagada sin movimiento; si el pago ya está en Movimientos, usa “Vincular…” en la cuota (o “Conciliar con los pagos” para todas).
dividas-marcar-como-paga = Marcar como pagada
dividas-valor-pago = Monto pagado
dividas-valor-pago-dica = Con multa e intereses por atraso, si los hay.
dividas-saldo-depois = Saldo deudor después del pago
dividas-saldo-depois-dica = Opcional, del comprobante: sincroniza el saldo y recalcula las próximas cuotas.
dividas-amortizacao-parcela = Amortización de la cuota
dividas-amortizacao-parcela-dica = Con el saldo, separa la amortización extra (abre una nueva versión del contrato).
dividas-faltam-dica = Del comprobante; vacío mantiene el plazo actual.
dividas-desligar-descricao = La cuota deja de estar pagada por el movimiento { $lancamento } y vuelve a quedar pendiente; el movimiento sigue en la cuenta. Se puede deshacer desde el historial de cambios.
dividas-desfazer-descricao = La cuota vuelve a quedar pendiente. Se puede deshacer desde el historial de cambios.
dividas-desfazer-pagamento = Deshacer pago
dividas-em-aberto-de-novo = Cuota { $n } pendiente de nuevo

## Extracto del contrato

dividas-extrato-importadas =
    { $n ->
        [one] { $n } cuota del extracto registrada
       *[other] { $n } cuotas del extracto registradas
    }
dividas-extrato-projecao =
    { $n ->
        [one] Proyección a partir del saldo del { $data }, con { $n } cuota restante.
       *[other] Proyección a partir del saldo del { $data }, con { $n } cuotas restantes.
    }
dividas-extrato-titulo = Importar extracto del contrato
dividas-extrato-descricao = Pega el historial del banco (de la hoja de cálculo o de la banca en línea), con el encabezado. Las cuotas quedan con los montos del banco, las amortizaciones extraordinarias se detectan por el saldo y el cronograma futuro parte del último saldo. También puedes enviar la imagen o el PDF al agente.
dividas-extrato-conferir = Revisar
dividas-extrato-importar = Importar
dividas-extrato-importar-n =
    { $n ->
        [one] Importar { $n } cuota
       *[other] Importar { $n } cuotas
    }
dividas-extrato-colado = Extracto pegado
dividas-extrato-colado-dica = Columnas: fecha, cuota, amortización, intereses, seguros, comisiones, saldo deudor, corrección monetaria y monto pagado (bastan los nombres del encabezado; también se reconoce el orden de la hoja de la Caixa).
dividas-extrato-faltam-dica = Después de la última fila. Vacío: se calcula por la última cuota.
dividas-extrato-marcar-anteriores = Marcar como pagadas las cuotas anteriores al extracto
dividas-extrato-previa = Vista previa del extracto
dividas-extrato-periodo = Período
dividas-extrato-periodo-valor = { $de } a { $ate }
dividas-extrato-ultimo-saldo = Último saldo
dividas-extrato-prazo-restante = Plazo restante
dividas-extrato-juros = Intereses del extracto
dividas-extrato-juros-valor = { $mensal }% mensual ({ $nominal }% nominal, { $efetiva }% efectiva anual)
dividas-extrato-extras = Amortizaciones extraordinarias detectadas
dividas-extrato-extras-aria = Amortizaciones detectadas
dividas-extrato-avisos = Avisos
dividas-extrato-taxa-ajustada = Tasa ajustada a { $taxa }% efectiva anual
dividas-extrato-usar-taxa = Usar la tasa del extracto ({ $taxa }% efectiva)

## Pago de cuota desde un movimiento

dividas-pagamento-so-saida = Solo una salida (monto negativo) puede pagar una cuota.
dividas-pagamento-transferencia = Es una transferencia entre tus cuentas: deshaz la transferencia para vincularla a la cuota.
dividas-pagamento-previsto = El movimiento todavía está previsto: confírmalo antes de vincularlo a la cuota.
dividas-pagamento-ligado-ok = Movimiento vinculado a la cuota { $n } de { $contrato }
dividas-pagamento-paga-em = Cuota pagada el { $data }.
dividas-pagamento-desligado = Movimiento desvinculado de la cuota { $n } de { $contrato }
dividas-pagamento-voltou = La cuota volvió a quedar pendiente.
dividas-pagamento-este = Este movimiento
dividas-pagamento-titulo = Pago de cuota
dividas-pagamento-trocar-para = Cambiar a la cuota { $n }
dividas-pagamento-ligar-n = Vincular a la cuota { $n }
dividas-pagamento-ligar = Vincular a la cuota
dividas-pagamento-saiu-de = salió de { $conta }
dividas-pagamento-ligado = Este movimiento paga la cuota { $n } de { $contrato }.
dividas-pagamento-ligado-texto = Elige otra cuota para cambiar, o desvincula: la cuota vuelve a quedar pendiente.
dividas-pagamento-carregando-contratos = Cargando los contratos…
dividas-pagamento-sem-contrato = Ningún contrato activo. Registra el préstamo o crédito en { $tela }.
dividas-pagamento-sem-contrato-moeda = Ningún contrato activo en { $moeda }. Registra el préstamo o crédito en { $tela }.
dividas-pagamento-escolha = Elige el contrato
dividas-pagamento-carregando-parcelas = Cargando las cuotas…
dividas-pagamento-parcelas-de = Cuotas de { $nome }
dividas-pagamento-vazio = Ninguna cuota sin movimiento en este contrato.
dividas-pagamento-nota = Las 12 cuotas sin movimiento con vencimiento más cercano al { $data }. La elegida queda pagada el { $data } por el monto del movimiento.
dividas-pagamento-nota-categoria = Las 12 cuotas sin movimiento con vencimiento más cercano al { $data }. La elegida queda pagada el { $data } por el monto del movimiento, y el movimiento recibe la categoría de los pagos del contrato.

## Registro y edición del contrato

dividas-form-editar = Editar contrato
dividas-form-editar-descricao = Corrige cualquier campo: las cuotas pendientes se recalculan. Las cuotas pagadas (con el monto y la fecha que registraste), las amortizaciones extraordinarias y los adelantos quedan como están.
dividas-form-novo-descricao = Préstamo o crédito: el cronograma (SAC o Price, con período de gracia, índice, seguros y comisiones) se genera al instante.
dividas-form-erro-nome = Ponle un nombre al contrato.
dividas-form-erro-principal = Ingresa el monto financiado.
dividas-form-erro-parcela = Ingresa el monto de la cuota.
dividas-form-erro-parcela-nao-fecha = Con esa cuota y ese plazo la tasa no cuadra: la cuota por el plazo tiene que superar el monto financiado.
dividas-form-erro-taxa = Ingresa la tasa efectiva anual.
dividas-form-erro-taxa-intervalo = Tasa fuera del rango.
dividas-form-erro-data-contrato = Ingresa la fecha del contrato.
dividas-form-erro-primeiro-vencimento = Ingresa el primer vencimiento.
dividas-form-erro-vencimento-antes = La primera cuota no puede vencer antes del contrato.
dividas-form-erro-prazo = Ingresa el plazo en meses.
dividas-form-erro-carencia = Período de gracia en meses (0 si no hay).
dividas-form-erro-carencia-prazo = El período de gracia debe ser menor que el plazo.
dividas-form-erro-percentual = Porcentaje del índice entre 0 y 1000%.
dividas-form-erro-defasagem = Desfase de 0 a 10 días hábiles.
dividas-form-erro-bullet = En el bullet no hay período de gracia: la cuota ya es solo intereses.
dividas-form-erro-valor = Monto inválido.
dividas-form-alterado = Contrato modificado
dividas-form-cadastrado = Contrato registrado
dividas-form-cadastrado-descricao =
    { $n ->
        [one] { $nome }: { $n } cuota en el cronograma
       *[other] { $nome }: { $n } cuotas en el cronograma
    }
dividas-form-dados = Datos del contrato
dividas-form-nome = Nombre del contrato
dividas-form-nome-exemplo = Crédito del departamento
dividas-form-numero = Número del contrato
dividas-form-numero-dica = Como aparece en el contrato o en la banca en línea. Opcional.
dividas-form-principal = Monto financiado
dividas-form-primeiro-vencimento = Primer vencimiento
dividas-form-sistema = Sistema de amortización
dividas-form-tipo-taxa = Tipo de tasa
dividas-form-pre = Fija
dividas-form-pos = Variable (CDI, Selic)
dividas-form-indice = Índice
dividas-form-indice-dica = Intereses de cada período por el índice acumulado en días hábiles (B3, base 252).
dividas-form-percentual = Porcentaje del índice
dividas-form-percentual-dica = 100% en la mayoría de los contratos; p. ej.: 120% del CDI.
dividas-form-spread = Spread (% anual)
dividas-form-spread-dica = Tasa por encima del índice: { $indice } + spread. Déjalo vacío si es solo el porcentaje.
dividas-form-taxa-juros = Tasa de interés
dividas-form-pela-parcela = Calcular por la cuota
dividas-form-informar-taxa = Ingresar la tasa
dividas-form-primeira-parcela = Primera cuota
dividas-form-parcela-juros = Cuota de intereses
dividas-form-valor-parcela = Monto de la cuota
dividas-form-parcela-dica-encargos = Como en la boleta o en el extracto: los seguros y comisiones por mes (abajo) salen de ahí.
dividas-form-parcela-dica = Como en la boleta o en el extracto, con seguros y comisiones (ingrésalos abajo).
dividas-form-taxa-calculada = Tasa calculada
dividas-form-taxa-nominal = Tasa nominal (% anual)
dividas-form-taxa-efetiva = Tasa efectiva (% anual)
dividas-form-como-taxa = Cómo trae la tasa el contrato
dividas-form-efetiva = Efectiva
dividas-form-nominal = Nominal
dividas-form-nominal-dica = Nominal anual (la mensual es la anual ÷ 12, como en la Caixa).
dividas-form-nominal-dica-equivale = Nominal anual (la mensual es la anual ÷ 12, como en la Caixa). Equivale a { $taxa }% efectiva anual.
dividas-form-efetiva-dica = Tasa de interés del contrato, sin el índice. La mensual es la equivalente.
dividas-form-detalhes = Detalles
dividas-form-tipo = Tipo
dividas-form-instituicao = Institución
dividas-form-nao-informar = No indicar
dividas-form-membro = Miembro de la familia
dividas-form-familia = Familia
dividas-form-data-contrato = Fecha del contrato
dividas-form-data-contrato-dica = Un mes antes del 1.er vencimiento.
dividas-form-forma-spread = Forma del spread
dividas-form-forma-spread-dica = Exponencial es lo habitual (CCB, capital de trabajo); lineal aparece en contratos antiguos.
dividas-form-defasagem = Desfase del índice (días hábiles)
dividas-form-defasagem-dica = 0 en la mayoría; algunos bancos usan el DI de 1 o 2 días hábiles antes.
dividas-form-correcao-dica = Corrección monetaria mensual del saldo (TR en el SFH, IPCA o INCC en el hipotecario).
dividas-form-juros-dica = Cómo corre la tasa (o los intereses reales) en cada período. Mensual es lo más común.
dividas-form-indice-usado = Índice usado
dividas-form-indice-aniversario-dica = El del período que empieza en el aniversario del contrato (lo habitual en la TR y el ahorro).
dividas-form-indice-mes-dica = La variación de un mes, con el desfase al lado (el IPCA sale al mes siguiente).
dividas-form-mes-indice = Mes del índice
dividas-form-mes-indice-dica = Un vencimiento en M usa el índice de M − n.
dividas-form-mesmo-mes = Del mismo mes (M)
dividas-form-deflacao = Ignorar deflación
dividas-form-deflacao-dica = Los meses con índice negativo no reducen el saldo (piso cero).
dividas-form-prorata = Prorrateo del 1.er período
dividas-form-prorata-dica = Cuando la firma no es exactamente un mes antes del 1.er vencimiento.
dividas-form-prorata-pos = Por los días hábiles del período (variable)
dividas-form-carencia = Período de gracia (meses)
dividas-form-carencia-dica = Solo intereses en ese período.
dividas-form-valor-imovel = Valor del inmueble
dividas-form-valor-imovel-dica = Base del seguro DFI (daños físicos).
dividas-form-mip-taxa = Seguro MIP (% mensual)
dividas-form-mip-valor = Seguro MIP ({ $moeda } por mes)
dividas-form-como-mip = Cómo trae el MIP el contrato
dividas-form-mip-taxa-dica = Muerte e invalidez, sobre el saldo deudor: baja junto con la deuda.
dividas-form-mip-taxa-dica-hoje = Muerte e invalidez, sobre el saldo deudor: baja junto con la deuda. Hoy ≈ { $valor } por mes.
dividas-form-mip-valor-dica = Monto fijo de muerte e invalidez en cada cuota, como en el contrato.
dividas-form-mip-valor-dica-equivale = Monto fijo de muerte e invalidez en cada cuota, como en el contrato. Equivale a { $pct }% mensual del saldo de { $saldo }.
dividas-form-dfi-taxa = Seguro DFI (% mensual)
dividas-form-dfi-valor = Seguro DFI ({ $moeda } por mes)
dividas-form-como-dfi = Cómo trae el DFI el contrato
dividas-form-dfi-taxa-dica = Daños físicos, sobre el valor del inmueble.
dividas-form-dfi-taxa-dica-mes = Daños físicos, sobre el valor del inmueble. ≈ { $valor } por mes.
dividas-form-dfi-valor-dica = Monto fijo de daños físicos en cada cuota (no necesita el valor del inmueble).
dividas-form-dfi-valor-dica-equivale = Monto fijo de daños físicos en cada cuota (no necesita el valor del inmueble). Equivale a { $pct }% mensual del inmueble.
dividas-form-outros-seguros = Otros seguros por mes
dividas-form-seguros = Seguros por mes
dividas-form-outros-seguros-dica = Monto fijo, además del MIP y del DFI.
dividas-form-seguros-dica = Seguro de desgravamen y otros.
dividas-form-taxa-adm = Comisión de administración por mes
dividas-form-taxa-adm-dica = Comisión mensual del contrato (en el crédito hipotecario suele ser R$ 25).
dividas-form-cet = { $sigla } del contrato (% anual)
dividas-form-cet-dica = Del contrato o de la propuesta. Opcional: Treasury también lo calcula con el cronograma y muestra los dos lado a lado.
dividas-form-conta-dica = Para conciliar las cuotas con los movimientos.
dividas-form-categoria-dica = Un movimiento en esta categoría (importado o ingresado a mano) marca la cuota más cercana como pagada.
dividas-form-observacoes = Observaciones
dividas-form-ja-paguei =
    { $n ->
        [one] Ya pagué la { $n } cuota vencida hasta hoy
       *[other] Ya pagué las { $n } cuotas vencidas hasta hoy
    }
dividas-form-ja-paguei-dica = Quedan pagadas en el vencimiento, por el monto del cronograma. Desactívalo para marcarlas una por una (o conciliar con los movimientos de la cuenta).
dividas-form-pos-nota = La cuota varía con el { $indice }: al guardar, el cronograma usa las tasas ya publicadas y, para el futuro, la última conocida (cuotas marcadas como estimadas y rehechas en cada recolección de índices).
dividas-form-na-carencia = En el período de gracia
dividas-form-ultima-parcela = Última cuota
dividas-form-correcao-valor = + { $indice } en el saldo
dividas-form-salvar = Guardar cambios
dividas-form-cadastrar = Registrar contrato

## Amortización extraordinaria

dividas-extra-descricao = { $nome }: el pago reduce el saldo deudor y el cronograma pendiente se recalcula. Se puede deshacer desde el historial de cambios.
dividas-extra-erro-valor = Ingresa el monto.
dividas-extra-erro-data = Ingresa la fecha (dd/mm/aaaa).
dividas-extra-registrada = Amortización registrada
dividas-extra-registrada-descricao = { $nome }: saldo deudor y cronograma recalculados
dividas-extra-depois = Después de la amortización
dividas-extra-sai-da-conta = Sale de la cuenta
dividas-extra-sem-conta-dica = Se registra solo en el contrato, sin movimiento (p. ej.: FGTS o pago hecho por otra persona).
dividas-extra-com-conta-dica = Crea el movimiento de salida en esa cuenta.
dividas-extra-sem-lancamento = Sin movimiento (FGTS, otra fuente)
dividas-extra-fgts-saldo = cuenta FGTS, saldo { $saldo }
dividas-extra-fgts-descricao = La salida quedó en la cuenta FGTS, como retiro para vivienda.
dividas-extra-data-passada = Fecha pasada ({ $data }): entra después de la última cuota vencida hasta ese día.
dividas-extra-registrar = Registrar amortización

dividas-cet-sigla = { $pais ->
    [BR] CET
    [PT] TAEG
    [FR] TAEG
    [DE] Effektiver Jahreszins
    [US] APR
    [GB] APR
    [CA] APR
    [AR] CFT
    [CL] CAE
   *[otro] TAE
}
dividas-cet-formula = Σ FCⱼ ÷ (1 + { $sigla })^((dⱼ − d₀) ÷ 365) − FC₀ = 0
dividas-cet-calculado = { $sigla } según nuestro cálculo
dividas-cet-nao-informado = no informado
dividas-cet-explicacao-titulo = Cómo calculamos el { $sigla }
dividas-cet-explicacao-intro = Es la tasa que iguala lo que recibiste el día de la firma a todo lo que pagas después, cada monto descontado por los días corridos hasta el pago. Entran intereses, comisiones, impuestos y seguros; los índices variables (TR, IPCA, CDI) quedan fuera.
dividas-cet-formula-legenda = FC₀ = monto entregado · FCⱼ = cuota j (amortización, intereses, seguros y comisiones) · dⱼ − d₀ = días corridos desde la entrega
dividas-cet-passo-liberado = 1 · Lo que recibiste
dividas-cet-passo-pagamentos = 2 · Lo que pagas
dividas-cet-passo-resultado = 3 · Resultado
dividas-cet-valor-contrato = Monto del contrato
dividas-cet-iof = Impuesto en la firma (IOF)
dividas-cet-tarifas = Comisiones y registro
dividas-cet-seguro-vista = Seguro de prima única
dividas-cet-liberado = Monto entregado (FC₀)
dividas-cet-parcelas = { $n } cuotas, del { $de } al { $ate } ({ $dias } días)
dividas-cet-amortizacao = Amortización
dividas-cet-juros = Intereses
dividas-cet-seguros = Seguros en las cuotas
dividas-cet-taxas = Comisiones en las cuotas
dividas-cet-total-pago = Total pagado
dividas-cet-do-total = { $pct } del total
dividas-cet-resultado-valor = { $aa } anual · { $am } mensual
dividas-cet-apr-nominal = APR nominal (Regulation Z)
dividas-cet-diferenca-titulo = El { $sigla } del contrato es distinto del nuestro
dividas-cet-diferenca = El contrato informa { $informado } y el cronograma registrado da { $calculado } ({ $diferenca } p.p.).
dividas-cet-diferenca-custos = Probablemente el banco incluyó impuestos, comisiones o seguro que no están registrados: agrégalos en Editar.
dividas-cet-diferenca-confira = Revisa en el contrato la tasa, las fechas, los seguros y los costos iniciales.
dividas-cet-indice-fora = Índice fuera del cálculo: { $indice }. La regla deja fuera los índices variables y solo los informa.
dividas-cet-sem-custos = No hay costos iniciales registrados. Si hubo impuesto, comisión de apertura, registro o seguro de prima única, agrégalos en Editar para que el { $sigla } quede completo.
dividas-cet-fonte-br = Regla: Resolución CMN nº 4.881/2020 del Banco Central de Brasil (arts. 3 a 5), con dos decimales.
dividas-cet-fonte-outros = Mismo cálculo que el CET de Brasil (Res. CMN 4.881/2020) y la TAE de España (Ley 16/2011): tasa anual efectiva, días corridos ÷ 365. Compárala con el número de tu contrato.
dividas-cet-fonte-eua = En EE. UU. el APR del contrato es nominal (tasa mensual × 12, Regulation Z, Appendix J); la tasa efectiva aparece al lado para comparar con otros países.
dividas-cet-quedas = { $n ->
    [one] Una caída del saldo sin pago registrado ({ $valor }, cuota { $parcela }) se contó como pagada.
   *[other] { $n } caídas del saldo sin pago registrado ({ $valor } en total, la primera en la cuota { $parcela }) se contaron como pagadas.
} Si fue una amortización, regístrala; si el monto del contrato está mal, corrígelo.
dividas-form-custos-iniciais = Costos en la firma
dividas-form-custos-iniciais-dica = Pagados el día de la firma, aunque se hayan sumado al monto financiado. Salen del monto entregado en el cálculo del { $sigla }.
dividas-form-iof = Impuesto (IOF)
dividas-form-tarifas-iniciais = Comisiones y registro
dividas-form-seguro-vista = Seguro de prima única
