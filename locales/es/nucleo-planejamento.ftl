# Planificación (núcleo: `forecast`, `scenarios`, `debt`, `pension`, `finance`): previsión de
# saldo y el "gastar hoy", escenarios, deudas (cronograma, liquidación, tasas de mercado,
# extractos y portabilidad), previsión privada y los errores. Se arman en el momento, en el
# idioma de la instalación. Errores en minúsculas, como frase de registro (la interfaz pone la
# inicial en mayúscula). INSS, IR, VGBL, PGBL, EFPC, CNPJ, CVM, BCB, SAC, MIP, DFI, CET, TR e
# IPCA quedan como están.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Previsión de saldo y cuánto puedes gastar hoy (`forecast`)

nucleo-planejamento-conta-ou-arquivada = cuenta (o archivada)
nucleo-planejamento-fatura = Resumen { $cartao }
nucleo-planejamento-parcela = Cuota { $contrato }
nucleo-planejamento-sazonal = { $descricao } (estimación por el valor de { $ano })
nucleo-planejamento-gastar-hoje =
    Ingresos del mes { $receitas } − gastos hasta ayer { $gastos } − cuentas que aún vencen { $contas } − falta ahorrar { $guardar } = { $disponivel } para { $dias ->
        [one] { $dias } día
       *[other] { $dias } días
    }: { $por_dia } por día.
nucleo-planejamento-gastar-amanha = Hoy ya se gastaron { $hoje }; a partir de mañana, { $amanha } por día.
nucleo-planejamento-gastar-sem-renda = Todavía no hay ingresos recibidos ni previstos este mes. Sin ellos no se puede calcular cuánto queda por día.

## Escenarios "y si" (`scenarios`)

nucleo-planejamento-cenario-nome = Escenario
nucleo-planejamento-cenario-explicacao =
    En { $anos ->
        [one] { $anos } año
       *[other] { $anos } años
    }, el patrimonio queda en { $cenario } en el escenario frente a { $base } en la base ({ $diferenca } de diferencia; { $liquido } en cuentas e inversiones).
nucleo-planejamento-cenario-dinheiro-acaba = El dinero disponible se acaba en { $mes }.
nucleo-planejamento-cenario-saldo-negativo = En los próximos 12 meses, el saldo de las cuentas queda negativo el { $data }.
nucleo-planejamento-cenario-aposentadoria = Jubilación: probabilidad de éxito de { $base } a { $cenario }.
nucleo-planejamento-cenario-nota-premissas = Valores reales de hoy; ingresos de { $renda } y gastos de { $gasto } por mes (promedio de los últimos 12 meses), rendimiento real de { $retorno } anual.
nucleo-planejamento-cenario-nota-bens = Los bienes y deudas que ya existen quedan con el valor de hoy; las cuotas actuales ya están en el gasto promedio.
nucleo-planejamento-cenario-nota-sem-idade = Sin edad en el plan de jubilación (Presupuesto › Jubilación): el efecto en la jubilación queda fuera.
nucleo-planejamento-cenario-erro-evento-sem-nome = cada evento necesita un nombre
nucleo-planejamento-cenario-erro-renda = { $evento }: ingresos de 0% a 1000%
nucleo-planejamento-cenario-erro-financiamento = { $evento }: financiamiento con monto, plazo (1 a 600 meses) y tasa válidos
nucleo-planejamento-cenario-erro-bem = { $evento }: el valor del bien debe ser positivo
nucleo-planejamento-cenario-erro-so-interface = escenarios guardados solo desde la interfaz de la app
nucleo-planejamento-cenario-erro-sem-nome = ponle un nombre al escenario

## Previsión privada: plan, rescate y extracto (`pension`). Los textos de las líneas del
## extracto ("Contribuição do participante"…) quedan como en el documento.

nucleo-planejamento-previdencia-plano = plan
nucleo-planejamento-previdencia-taxa-administracao = comisión de administración
nucleo-planejamento-previdencia-taxa-carregamento = comisión de carga
nucleo-planejamento-previdencia-erro-taxa-invalida = { $taxa } no válida: { $valor }
nucleo-planejamento-previdencia-erro-taxa-faixa = { $taxa } debe estar entre 0 y 100%
nucleo-planejamento-previdencia-erro-classe = el plan de previsión es para activos de la clase Planes de pensiones
nucleo-planejamento-previdencia-erro-resgate-positivo = indica un monto de rescate positivo
nucleo-planejamento-previdencia-erro-resgate-saldo = el rescate supera el saldo del plan ({ $saldo })
nucleo-planejamento-previdencia-nota-vgbl-regressivo = VGBL regresivo: IR definitivo solo sobre el rendimiento, según la antigüedad de cada aporte (los más antiguos salen primero).
nucleo-planejamento-previdencia-nota-pgbl-regressivo = PGBL regresivo: IR definitivo sobre el valor total, según la antigüedad de cada aporte (los más antiguos salen primero).
nucleo-planejamento-previdencia-nota-vgbl-progressivo = VGBL progresivo: 15% del rendimiento retenido en la fuente; el IR final sale de la tabla anual en la declaración.
nucleo-planejamento-previdencia-nota-efpc-regressivo = Fondo de pensión (EFPC), regresivo: IR definitivo sobre el valor total, según la antigüedad de cada contribución (las más antiguas salen primero).
nucleo-planejamento-previdencia-nota-efpc-progressivo = Fondo de pensión (EFPC), progresivo: 15% del valor total retenido en la fuente; el IR final sale de la tabla anual en la declaración.
nucleo-planejamento-previdencia-nota-pgbl-progressivo = PGBL progresivo: 15% del valor total retenido en la fuente; el IR final sale de la tabla anual en la declaración.
nucleo-planejamento-previdencia-aviso-sem-plano = indica el tipo de plan (VGBL o PGBL) y el régimen de tributación para que el IR salga bien; por ahora el cálculo supone VGBL regresivo
nucleo-planejamento-previdencia-aviso-sem-cota = sin cuota reciente: el valor de hoy usa la cuota del último aporte. Indica el CNPJ del fondo (datos de la CVM) o la cuota actual
nucleo-planejamento-previdencia-motivo-plano = Plan de previsión: { $nome }
nucleo-planejamento-previdencia-motivo-extrato = Extracto de previsión importado
nucleo-planejamento-previdencia-erro-pdf = no pude leer este PDF: { $erro }
nucleo-planejamento-previdencia-erro-sem-contribuicoes = no reconocí contribuciones en este extracto (líneas "MM/AAAA DD/MM/AAAA DD/MM/AAAA concepto montos")
nucleo-planejamento-previdencia-erro-sem-arquivo = envía el PDF del extracto o su texto
nucleo-planejamento-previdencia-erro-extrato-classe = el extracto de previsión va en un activo de la clase Planes de pensiones
nucleo-planejamento-previdencia-erro-conta = elige la cuenta de las cuotas de la previsión
nucleo-planejamento-previdencia-erro-interno = extracto de previsión: { $erro }
nucleo-planejamento-previdencia-aviso-cotas = la suma de las cuotas de las contribuciones ({ $soma }) no coincide con el saldo de cuotas del extracto ({ $saldo }): revisa si quedó fuera alguna línea
nucleo-planejamento-previdencia-aviso-saldo = el saldo por las cuotas ({ $calculado }) difiere del saldo del participante en el extracto ({ $extrato })
nucleo-planejamento-previdencia-aviso-sem-cnpj = no encontré el CNPJ del plan en el extracto
# Nombre del activo creado por el extracto cuando no trae el nombre del plan.
nucleo-planejamento-previdencia-ativo-padrao = Previsión (fondo de pensión)
# Notas guardadas en el aporte y en el gasto de riesgo ("Contribuição do participante (mes de referencia 03/2026)").
nucleo-planejamento-previdencia-nota-aporte = { $descricao } (mes de referencia { $competencia })
nucleo-planejamento-previdencia-nota-risco = Previsión { $plano } (mes de referencia { $competencia })

## Precio promedio de la cartera (`finance::average_cost`)

nucleo-planejamento-erro-compra-positiva = la cantidad de compra debe ser positiva
nucleo-planejamento-erro-venda-positiva = la cantidad de venta debe ser positiva
nucleo-planejamento-erro-venda-maior = venta mayor que la posición (la venta en corto no está soportada)
nucleo-planejamento-erro-fator-desdobramento = factor de desdoblamiento/agrupamiento no válido

## Deudas: contratos, cuotas, adelanto y portabilidad (`debt`)

nucleo-planejamento-divida-emprestimo = préstamo { $id }
nucleo-planejamento-divida-contrato = contrato
nucleo-planejamento-divida-parcela = cuota
nucleo-planejamento-divida-categoria = categoría
nucleo-planejamento-divida-parcela-do-mes = cuota del mes del pago
# Nombres de los campos en los errores de valor no válido ("tasa no válido: abc").
nucleo-planejamento-divida-campo-taxa = tasa
nucleo-planejamento-divida-campo-taxa-dfi = tasa del DFI
nucleo-planejamento-divida-campo-taxa-mip = tasa del MIP
nucleo-planejamento-divida-campo-percentual-indice = porcentaje del índice
nucleo-planejamento-divida-campo-cet = CET
nucleo-planejamento-divida-erro-invalido = valor no válido en { $campo }: { $valor }
nucleo-planejamento-divida-erro-taxa-seguro = { $campo }: indica la tasa mensual en % (ej.: 0,0247)
nucleo-planejamento-divida-erro-sem-nome = ponle un nombre al contrato
nucleo-planejamento-divida-erro-valor-positivo = el monto financiado debe ser positivo
nucleo-planejamento-divida-erro-bullet-carencia = en el bullet no hay período de gracia: la cuota ya es solo intereses
nucleo-planejamento-divida-erro-percentual-indice = porcentaje del índice fuera del intervalo (0 a 1000%)
nucleo-planejamento-divida-erro-defasagem-mensal = desfase del índice mensual de 0 a 3 meses
nucleo-planejamento-divida-erro-defasagem-diaria = desfase del índice de 0 a 10 días hábiles
nucleo-planejamento-divida-erro-posfixado-spread = en los de tasa variable la cuota cambia con el índice: indica el spread (0 si no hay)
nucleo-planejamento-divida-erro-taxa-ou-parcela = indica la tasa anual o el monto de la cuota para calcular la tasa
nucleo-planejamento-divida-erro-taxa-calculada = tasa calculada no válida
nucleo-planejamento-divida-erro-taxa-intervalo = tasa anual fuera del intervalo
nucleo-planejamento-divida-erro-indexador-diferente = la tasa trae un índice distinto del elegido en el contrato
nucleo-planejamento-divida-erro-taxa-invalida = tasa no válida: { $taxa }
nucleo-planejamento-divida-erro-primeira-parcela = la primera cuota no puede vencer antes del contrato
nucleo-planejamento-divida-erro-prazo = plazo no válido (el período de gracia debe ser menor que el plazo)
nucleo-planejamento-divida-erro-quantas-antecipar = indica cuántas cuotas adelantar
nucleo-planejamento-divida-erro-sem-futuras = no hay cuotas futuras pendientes para adelantar
nucleo-planejamento-divida-erro-poucas-futuras =
    { $n ->
        [one] solo hay { $n } cuota futura pendiente
       *[other] solo hay { $n } cuotas futuras pendientes
    }
nucleo-planejamento-divida-erro-parcela-seguros = la cuota debe ser mayor que los seguros y comisiones mensuales
nucleo-planejamento-divida-erro-taxa-nao-fecha = con esa cuota y ese plazo la tasa no cierra: revisa los valores (la cuota por el plazo debe superar el monto financiado)
nucleo-planejamento-divida-erro-pagamento-antes = el pago es anterior al primer vencimiento del contrato
nucleo-planejamento-divida-erro-pagamento-negativo = los montos del pago no pueden ser negativos
nucleo-planejamento-divida-erro-saldo-restante = todavía hay saldo deudor: indica cuántas cuotas faltan
nucleo-planejamento-divida-erro-amortizacao-positiva = el monto de la amortización debe ser positivo
nucleo-planejamento-divida-erro-imovel-negativo = el valor del inmueble no puede ser negativo
nucleo-planejamento-divida-erro-seguro-negativo = el valor del seguro no puede ser negativo
nucleo-planejamento-divida-erro-dfi-sem-imovel = el DFI en porcentaje se calcula sobre el valor del inmueble: indica el valor del inmueble (o el DFI en reales)
nucleo-planejamento-divida-erro-categoria-despesa = la categoría de los pagos debe ser de gasto
nucleo-planejamento-divida-erro-categoria-ligada = esa categoría ya está vinculada al contrato { $contrato }
nucleo-planejamento-divida-erro-conciliar-sem-conta = define la cuenta de débito (o la categoría de los pagos) del contrato para conciliar
nucleo-planejamento-divida-erro-pagamento-saida = el pago de una cuota debe ser una salida (monto negativo)
nucleo-planejamento-divida-erro-pagamento-transferencia = el movimiento es una transferencia entre tus cuentas y no puede pagar una cuota
nucleo-planejamento-divida-erro-pagamento-previsto = el movimiento todavía está previsto; efectúalo antes de vincularlo a la cuota
nucleo-planejamento-divida-erro-pagamento-moeda = el movimiento está en { $moeda } y el contrato en { $moeda_contrato }
nucleo-planejamento-divida-erro-pagamento-ja-ligado = el movimiento ya está vinculado a la cuota { $numero } de { $contrato }; desvincúlalo de allí antes
nucleo-planejamento-divida-erro-pagamento-amortizacao = el movimiento es de una amortización extraordinaria de { $contrato }
# Autor y motivo en el historial cuando los índices nuevos recalculan las cuotas.
nucleo-planejamento-divida-autor-indices = índices
nucleo-planejamento-divida-motivo-indices = Cuotas recalculadas con los índices más recientes
nucleo-planejamento-divida-motivo-extrato = Extracto del contrato importado
nucleo-planejamento-divida-motivo-cadastrado = Contrato registrado: { $nome }
nucleo-planejamento-divida-motivo-alterado = Contrato modificado
nucleo-planejamento-divida-motivo-removido = Contrato eliminado
nucleo-planejamento-divida-motivo-amortizacao = Amortización extraordinaria
nucleo-planejamento-divida-motivo-antecipadas = Cuotas adelantadas
nucleo-planejamento-divida-motivo-vencidas-pagas = Cuotas vencidas marcadas como pagadas
nucleo-planejamento-divida-motivo-conciliadas = Cuotas conciliadas con los pagos
nucleo-planejamento-divida-motivo-pagamento = Pago de cuota registrado
nucleo-planejamento-divida-motivo-parcela-paga = Cuota pagada
nucleo-planejamento-divida-motivo-parcela-ligada = Cuota { $numero } de { $contrato } vinculada al movimiento
nucleo-planejamento-divida-motivo-pagamento-desfeito = Pago de cuota deshecho
# Descripción de los movimientos creados por la amortización y por el adelanto.
nucleo-planejamento-divida-lancamento-amortizacao = Amortización extraordinaria · { $contrato }
nucleo-planejamento-divida-lancamento-antecipacao =
    { $n ->
        [one] Adelanto de { $n } cuota · { $contrato }
       *[other] Adelanto de { $n } cuotas · { $contrato }
    }

## Tasas de interés del mercado (Banco Central do Brasil, `debt::market_rates`)

nucleo-planejamento-mercado-modalidade-cheque-especial = Descubierto en cuenta
nucleo-planejamento-mercado-modalidade-cartao-rotativo = Tarjeta de crédito: rotativo
nucleo-planejamento-mercado-modalidade-cartao-parcelado = Tarjeta de crédito: en cuotas
nucleo-planejamento-mercado-modalidade-credito-pessoal = Préstamo personal
nucleo-planejamento-mercado-modalidade-consignado-privado = Préstamo con descuento de nómina (empresa privada)
nucleo-planejamento-mercado-modalidade-consignado-publico = Préstamo con descuento de nómina (servidor público)
nucleo-planejamento-mercado-modalidade-consignado-inss = Préstamo con descuento del INSS
nucleo-planejamento-mercado-modalidade-veiculo = Crédito automotriz
nucleo-planejamento-mercado-modalidade-outros-bens = Financiamiento de otros bienes
nucleo-planejamento-mercado-modalidade-imobiliario-pre = Crédito hipotecario (tasa fija)
nucleo-planejamento-mercado-modalidade-imobiliario-tr = Crédito hipotecario (TR + tasa)
nucleo-planejamento-mercado-modalidade-imobiliario-ipca = Crédito hipotecario (IPCA + tasa)
nucleo-planejamento-mercado-resumo-dentro = { $contrato }: { $taxa } mensual, dentro de lo que cobra el mercado ({ $mediana } mensual de mediana entre { $bancos } bancos).
nucleo-planejamento-mercado-resumo-acima = { $contrato }: { $taxa } mensual, por encima de la mediana del mercado ({ $mediana } mensual); { $mais_baratos } de { $bancos } bancos cobran menos{ $melhor }. Vale la pena simular la portabilidad.
# Va después de "cobran menos" (empieza con la coma).
nucleo-planejamento-mercado-resumo-melhor = , el más barato ({ $banco }) cobra { $taxa } mensual
nucleo-planejamento-mercado-fonte = Banco Central do Brasil — tasas de interés por institución financiera (promedio de las operaciones de cada banco; tu contrato depende de tu perfil).
nucleo-planejamento-mercado-aviso-fonte-desligada = La fuente Banco Central está desactivada en Ajustes › Fuentes de datos.
nucleo-planejamento-mercado-aviso-falha = No se pudieron actualizar las tasas del BCB ({ $erro }); valen las últimas descargadas.
nucleo-planejamento-mercado-aviso-sem-taxas = Todavía no hay tasas del BCB descargadas.
nucleo-planejamento-mercado-erro-resposta = respuesta del BCB no válida: { $erro }
nucleo-planejamento-mercado-erro-sem-periodo = el BCB no devolvió el período más reciente
nucleo-planejamento-mercado-erro-sem-taxas = el BCB no devolvió tasas

## Extracto del contrato pegado desde el banco (`debt::statement`)

nucleo-planejamento-divida-extrato-linhas-nao-entendidas =
    { $n ->
        [one] { $n } línea no entendida, por ejemplo: "{ $exemplo }"
       *[other] { $n } líneas no entendidas, por ejemplo: "{ $exemplo }"
    }
nucleo-planejamento-divida-extrato-erro-vazio = pega el extracto del contrato (o envía las líneas)
nucleo-planejamento-divida-extrato-erro-nenhuma-linha = no se entendió ninguna línea del extracto: pégalo con el encabezado (fecha, amortización, intereses, saldo deudor…)
nucleo-planejamento-divida-extrato-aviso-juros = los intereses del extracto equivalen a { $mensal }% mensual ({ $nominal }% anual nominal, { $efetiva }% anual efectiva); el contrato está con { $contrato }% anual efectiva. Ajusta la tasa para que la proyección coincida con el banco
nucleo-planejamento-divida-extrato-aviso-prazo = no se pudo calcular el plazo que falta por la última cuota: indica el plazo restante
nucleo-planejamento-divida-extrato-erro-prazo = indica el plazo restante (cuántas cuotas faltan después de la última línea)
nucleo-planejamento-divida-extrato-erro-antes-do-vencimento = la línea del { $data } es anterior al primer vencimiento del contrato ({ $vencimento }): corrige el primer vencimiento

## Plan de liquidación: bola de nieve × avalancha (`debt::payoff`)

nucleo-planejamento-quitacao-contrato = contrato { $id }
nucleo-planejamento-quitacao-erro-extra-negativo = el extra por mes no puede ser negativo
nucleo-planejamento-quitacao-erro-divida-incompleta = cada deuda informada necesita nombre, saldo y pago mínimo positivos
nucleo-planejamento-quitacao-erro-taxa-negativa = tasa negativa
nucleo-planejamento-quitacao-erro-sem-dividas = ninguna deuda con saldo: registra un préstamo o informa una deuda
nucleo-planejamento-quitacao-nota-indexador = { $contrato }: solo la tasa contratada, sin la corrección del índice.
nucleo-planejamento-quitacao-nota-indexador-valor = { $contrato }: tasa contratada más { $indice } de { $valor }% al mes (último valor, del { $data }), constante hasta el final.
nucleo-planejamento-quitacao-nota-sac = { $contrato }: en el SAC la cuota baja con el tiempo; aquí queda fija (se liquida un poco antes).
nucleo-planejamento-quitacao-nota-minimo-nao-quita = Solo con el mínimo alguna deuda no termina en 50 años: la cuota no cubre los intereses.
nucleo-planejamento-quitacao-mais-de-50-anos = más de 50 años
nucleo-planejamento-quitacao-avalanche = avalancha
nucleo-planejamento-quitacao-bola-de-neve = bola de nieve
nucleo-planejamento-quitacao-explicacao-minimo = Solo con el mínimo: libre de deudas en { $quando } con { $juros } de intereses.
nucleo-planejamento-quitacao-explicacao-minimo-nunca = Solo con el mínimo, alguna deuda nunca termina: la cuota no cubre los intereses.
nucleo-planejamento-quitacao-explicacao-estrategias = Bola de nieve: { $bola_quando } ({ $bola_juros } de intereses). Avalancha: { $avalanche_quando } ({ $avalanche_juros } de intereses).
nucleo-planejamento-quitacao-explicacao-iguais = Las dos dan el mismo resultado aquí; la bola de nieve liquida la primera deuda antes.
# La estrategia viene del catálogo ("avalancha", "bola de nieve").
nucleo-planejamento-quitacao-explicacao-economia = La { $estrategia } ahorra { $valor } de intereses frente al mínimo.
nucleo-planejamento-quitacao-explicacao-menos-juros = La { $estrategia } paga { $valor } menos de intereses.
nucleo-planejamento-cenario-negativo-hoje = El dinero disponible ya es negativo hoy.
nucleo-planejamento-divida-parcela-n = cuota { $n } de este préstamo
nucleo-planejamento-previdencia-motivo-pagamentos = Historial de pagos de la previsión importado
nucleo-planejamento-previdencia-pagamentos-erro-classe = El historial de pagos es para activos de la clase Previsión.
nucleo-planejamento-previdencia-pagamentos-erro-vazio = Indica al menos un pago (fecha e importe).
nucleo-planejamento-previdencia-pagamentos-erro-valor = El importe de cada pago debe ser mayor que cero.
nucleo-planejamento-previdencia-pagamentos-erro-conta = Elige la cuenta de las cuotas del plan.
nucleo-planejamento-previdencia-pagamentos-sem-cnpj = Sin cuota: indica en el activo el CNPJ del fondo del plan (el FIE), para buscar las cuotas en la CVM.
nucleo-planejamento-previdencia-pagamentos-sem-cota = { $n ->
        [one] { $n } aporte
       *[other] { $n } aportes
    } aún sin cuota de la CVM, desde { $data }: entran en la propuesta con la cantidad pendiente y se vuelven compra de cuotas solos cuando la recogida de cotizaciones traiga el historial del fondo.
nucleo-planejamento-previdencia-pagamentos-nada = Nada nuevo para grabar: los pagos ya se importaron.
nucleo-planejamento-previdencia-pagamentos-ilegivel-valor = Captura { $print }: la línea de { $lido } tiene el importe ilegible. Revísalo en la app de la aseguradora e indica el importe.
nucleo-planejamento-previdencia-pagamentos-ilegivel-data = Captura { $print }: el pago de { $lido } no tiene fecha legible. Revísalo en la app de la aseguradora e indica la fecha.
nucleo-planejamento-previdencia-pagamentos-cortada-alto = Captura { $print }: la primera línea está cortada (solo { $lido }). Envía la captura de arriba con la línea entera, o indica el pago.
nucleo-planejamento-previdencia-pagamentos-cortada-fim = Captura { $print }: la última línea está cortada (solo { $lido }). Envía la captura de abajo con la línea entera, o indica el pago.
nucleo-planejamento-previdencia-pagamentos-mes-faltando = Ningún pago en { $lido }, en medio de una serie mensual. Revisa si hubo aporte ese mes (puede faltar una captura).
nucleo-planejamento-previdencia-aviso-aportes-sem-cota =
    { $n ->
        [one] { $n } aporte aún sin cuota de la CVM cuenta en el saldo por el importe aportado, sin rendimiento, hasta que la recogida traiga la cuota del día
       *[other] { $n } aportes aún sin cuota de la CVM cuentan en el saldo por el importe aportado, sin rendimiento, hasta que la recogida traiga la cuota del día
    }
nucleo-planejamento-previdencia-pagamentos-vao = Capturas { $print }: el día { $lido } quedó con menos aportes que los otros días de la serie; un pago puede haber quedado entre las dos capturas. Envía una captura que muestre ese tramo, o indica el pago.
nucleo-planejamento-previdencia-pagamentos-conta-nova = Cuotas · { $plano }
nucleo-planejamento-previdencia-pagamentos-conta-nota = Cuenta de las cuotas del plan { $plano }, creada al importar el historial de pagos (sin movimiento de caja).
nucleo-planejamento-previdencia-motivo-pagamentos-n = { $plano }: { $com } aporte(s) con cuota, { $sem } a la espera de la cuota de la CVM y { $pendencias } pendiente(s) para revisar
nucleo-planejamento-previdencia-motivo-cotas-completadas =
    { $n ->
        [one] Cuotas de la CVM completadas en { $n } aporte de previsión
       *[other] Cuotas de la CVM completadas en { $n } aportes de previsión
    }
nucleo-previdencia-saldo-so-previdencia = el saldo informado es solo para planes de previsión
nucleo-previdencia-saldo-positivo = el saldo debe ser mayor que cero
nucleo-previdencia-saldo-sem-cotas = el plan no tiene cuotas el { $data }: registre los aportes antes de informar el saldo

## Cuotas detectadas en la descripción
nucleo-parcelas-ref-serie = serie de cuotas
nucleo-parcelas-historico-ignorar = No es cuota: { $descricao }
nucleo-parcelas-historico-voltar = Serie de cuotas restaurada
nucleo-parcelas-previsao = { $descricao } · cuota detectada
