# Pantalla 03 · Cuentas: lista y resumen de las cuentas, tarjeta de cada cuenta, formulario,
# detalle (historial de saldo, conciliación, sincronía con el banco, tipo de cambio, aportaciones
# de cooperativa, sobregiro) y los resúmenes y cuotas de la tarjeta (docs/i18n-design.md).
# Revisión: borrador por IA, sin revisión de un hablante nativo.

## Etiquetas repetidas

contas-data = Fecha
contas-saldo = Saldo
contas-tipo = Tipo
contas-moeda = Moneda
contas-valor = Monto
contas-total = Total
contas-restante = Pendiente
contas-calculado = Calculado
contas-conciliado = Conciliado
contas-previsto = Previsto
contas-pago = Pagado
contas-vence = Vence
contas-mes = Mes
contas-em-moeda = En { $moeda }
contas-sem-limite = sin límite
contas-cartao = Tarjeta
contas-faturas = Resúmenes
contas-parcelamentos = Cuotas
contas-historico-saldo = Historial de saldo
contas-conciliacao = Conciliación
contas-cambio = Tipo de cambio
contas-reativar = Reactivar
contas-arquivar = Archivar
contas-desfazer = Deshacer
contas-limite-moeda = Límite ({ $moeda })

## Encabezado, resumen y grupos

contas-descricao = Saldos medidos (saldo inicial + movimientos), previstos y conciliados; tarjetas con resúmenes y límite; cuentas en otra moneda convertidas a { $moeda }.
contas-membros-familia = Miembros de la familia
contas-novo-cartao = Nueva tarjeta
contas-nova-conta = Nueva cuenta
contas-carregando = Cargando cuentas
contas-vazio-kicker = Ninguna cuenta
contas-vazio-titulo = Registra la primera cuenta
contas-vazio-descricao = Cuenta corriente, de ahorros, efectivo, tarjeta de crédito, corredora o cuenta en el exterior. El saldo es el saldo inicial más los movimientos.
contas-resumo = Resumen de las cuentas
contas-saldo-liquido = Saldo neto en { $moeda }
contas-saldo-liquido-fonte = Cuentas que entran en el patrimonio (sin inversiones a valor de mercado)
contas-saldo-liquido-estimado = Hay una cuenta en otra moneda con un tipo de cambio de hace más de 7 días.
contas-sem-cambio-fora = { $n ->
        [one] { $n } cuenta sin tipo de cambio fuera de la suma
       *[other] { $n } cuentas sin tipo de cambio fuera de la suma
    }
contas-em-contas = En cuentas
contas-devido-cartoes = Deuda en tarjetas
contas-credito-disponivel = Crédito disponible
contas-mostrar-arquivadas = Mostrar archivadas
contas-convertidas-moeda = convertidas a { $moeda }
contas-cambio-origem = Tipo de cambio (PTAX o ingresado a mano)
contas-acoes-de = Acciones de { $nome }
contas-conta-reativada = Cuenta reactivada
contas-conta-arquivada = Cuenta archivada
contas-erro-alterar = No se pudo cambiar la cuenta

## Cerrar la cuenta (cerrada en el banco o en la casa de bolsa)

contas-encerrar = Cerrar cuenta
contas-menu-encerrar = Cerrar cuenta…
contas-reabrir = Reabrir cuenta
contas-conta-encerrada = Cuenta cerrada
contas-conta-reaberta = Cuenta reabierta
contas-encerrada-em = cerrada el { $data }
contas-encerrada-em-nome = { $nome }, cerrada el { $data }
contas-encerrada-descricao = Cerrada el { $data }. Queda fuera de las listas y de los avisos y no acepta movimientos después de esa fecha; el historial sigue en los informes.
contas-encerrar-titulo = Cerrar { $nome }
contas-encerrar-descricao = Para la cuenta que cerraste en el banco o en la casa de bolsa. Sale de las listas y deja de generar avisos; el historial sigue en los informes. Puedes reabrirla después.
contas-encerrar-data = Fecha de cierre
contas-encerrar-data-dica = El día en que el banco cerró la cuenta, o el del último movimiento.
contas-encerrar-saldo = Saldo el { $data }
contas-encerrar-bloqueios = Antes de cerrar
contas-encerrar-bloqueio-futuro = La fecha no puede ser posterior a hoy.
contas-encerrar-bloqueio-abertura = La fecha es anterior a la apertura de la cuenta ({ $data }).
contas-encerrar-bloqueio-encerrada = La cuenta ya se cerró el { $data }.
contas-encerrar-bloqueio-saldo = Todavía hay { $valor } en la cuenta en esa fecha. Transfiere el saldo a otra cuenta o registra un ajuste.
contas-encerrar-bloqueio-depois = { $n ->
        [one] Hay { $n } movimiento después de la fecha (el último el { $data }).
       *[other] Hay { $n } movimientos después de la fecha (el último el { $data }).
    }
contas-encerrar-bloqueio-posicoes = La cuenta todavía tiene { $ativos }. Registra la venta o el traspaso a otra casa de bolsa.
contas-encerrar-bloqueio-parcelas = { $n ->
        [one] La tarjeta todavía tiene { $n } cuota por vencer.
       *[other] La tarjeta todavía tiene { $n } cuotas por vencer.
    }
contas-encerrar-transferir = Transferir el saldo
contas-encerrar-ajuste = Registrar ajuste
contas-encerrar-efeitos = Qué pasa al cerrar
contas-encerrar-pronto = La cuenta está en cero en esa fecha y se puede cerrar.
contas-encerrar-sem-avisos = Sale de las listas y de los avisos; los movimientos después de la fecha quedan bloqueados.
contas-encerrar-recorrencias = { $n ->
        [one] Se pausará { $n } recurrencia: { $nomes }.
       *[other] Se pausarán { $n } recurrencias: { $nomes }.
    }
contas-encerrar-previstos = { $n ->
        [one] Se borrará { $n } movimiento previsto.
       *[other] Se borrarán { $n } movimientos previstos.
    }
contas-encerrar-emprestimos = Préstamos que se debitan de esta cuenta: { $nomes }. Cambia la cuenta de débito en Deudas.
contas-encerrar-cartoes = Tarjetas pagadas desde esta cuenta: { $nomes }. Cambia la cuenta que paga el estado de cuenta en los datos de la tarjeta.
contas-encerrar-confirmar = Cerrar cuenta

## Comandos (las palabras de búsqueda van separadas por coma)

contas-cmd-nova-busca = banco,tarjeta,corredora
contas-cmd-novo-cartao = Nueva tarjeta de crédito
contas-cmd-novo-cartao-busca = resumen,estado de cuenta,límite
contas-cmd-abrir = Abrir cuenta: { $nome }
contas-cmd-abrir-busca = saldo

## Tarjeta de cada cuenta

contas-sem-cambio = Sin tipo de cambio { $moeda }/{ $base }: ingresa una tasa.
contas-cambio-manual = Tipo de cambio ingresado a mano
contas-cambio-fonte = Tipo de cambio { $fonte }
contas-cambio-nota = 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $base }.
contas-cambio-nota-velha = 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $base }. La tasa es de hace más de 7 días.
contas-abrir = Abrir { $nome }
contas-detalhes = Detalles →
contas-instituicao = Institución
contas-membros-rotulo = { $n ->
        [one] Miembro
       *[other] Miembros
    }
contas-ultimo = Último movimiento
contas-fatura-saldo-devedor = Resumen y saldo deudor
contas-saldo-hoje = Saldo hoy
contas-arquivada = archivada
contas-previsto-fim-mes = Previsto (fin de mes)
contas-disponivel = Disponible
contas-ciclo = Ciclo
contas-ciclo-valor = cierra el día { $fechamento } · vence el día { $vencimento }
contas-limite-comprometido = { $pct } del límite utilizado
contas-abrir-site = Abrir el sitio de { $nome } en el navegador
contas-site = Sitio ↗

## Menú contextual de la cuenta

contas-menu-abrir-detalhes = Abrir detalles
contas-menu-nova-compra = Nueva compra con esta tarjeta
contas-menu-novo-lancamento = Nuevo movimiento en esta cuenta
contas-menu-compra-parcelada = Compra en cuotas
contas-menu-transferir = Transferir desde esta cuenta
contas-menu-ver-lancamentos = Ver movimientos
contas-menu-importar = Importar extracto o resumen
contas-menu-conciliar = Conciliar con el extracto
contas-menu-cambio = Tipo de cambio { $moeda }/{ $base }
contas-menu-editar = Editar…

## Detalle de la cuenta

contas-novo-lancamento = Nuevo movimiento
contas-lancamentos-link = Movimientos →
contas-fechar-detalhe = Cerrar detalle
contas-saldo-inicial = Saldo inicial
contas-desde = Desde
contas-patrimonio = Patrimonio
contas-patrimonio-entra = entra
contas-patrimonio-fora = no entra
contas-saldo-devedor = Saldo deudor
contas-previsto-ate-fim-mes = Previsto hasta fin de mes
contas-limite-de = de { $limite }
contas-cotas-capital = Aportaciones de capital
contas-secoes-de = Secciones de { $nome }

## Conciliación con el extracto

contas-n-conciliados = { $n ->
        [one] { $n } movimiento conciliado
       *[other] { $n } movimientos conciliados
    }
contas-ate-data = hasta el { $data }
contas-conciliar-explicacao = Toma el saldo del extracto del banco en una fecha y compáralo con el calculado (saldo inicial + movimientos efectuados hasta esa fecha). Si coincide, los movimientos hasta ahí pasan a { $conciliados } y quedan protegidos contra cambios accidentales de monto o fecha.
contas-conciliados-destaque = conciliados
contas-data-extrato = Fecha del extracto
contas-saldo-extrato = Saldo del extracto ({ $moeda })
contas-comparar = Comparar
contas-extrato-banco = Extracto del banco
contas-diferenca = Diferencia
contas-n-marcados-conciliados = { $n ->
        [one] ✓ { $n } movimiento marcado como conciliado.
       *[other] ✓ { $n } movimientos marcados como conciliados.
    }
contas-conciliacao-desfeita = Conciliación deshecha
contas-confere = Coincide ✓
contas-marcar-conciliado = Marcar como conciliado hasta el { $data }
contas-nao-confere = No coincide. Busca movimientos faltantes o duplicados hasta el { $data }: { $link }.
contas-nao-confere-link = ver los efectuados aún no conciliados

## Sincronía del saldo con el banco

contas-sincronia-sem-motivo = sin motivo
contas-sincronia-explicada = explicada
contas-sincronia-aceita = aceptada
contas-sincronizar-explicacao = ¿No sabes el motivo? Sincroniza con el banco: la diferencia se vuelve un movimiento de { $ajuste } (no cuenta como ingreso ni gasto), el saldo coincide con el extracto y el punto de control queda guardado para investigar después.
contas-ajuste-destaque = ajuste
contas-observacao-opcional = Observación (opcional)
contas-observacao-exemplo = Ej.: saldo en la app del banco a las 18 h
contas-saldo-sincronizado = Saldo sincronizado con el banco
contas-ajuste-lancado = Ajuste registrado el { $data }.
contas-sincronizar = Sincronizar con el banco
contas-ajuste-resumo = Ajuste de { $diferenca } el { $data }: el banco mostraba { $banco } y Treasury { $treasury }.
contas-causas-provaveis = Causas probables
contas-ver-lancamentos-periodo = Ver los movimientos del período
contas-pedir-agente = Pedir al agente que investigue
contas-pedido-agente = Investiga la diferencia de saldo de la cuenta { $conta } el { $data } (sincronía { $id }): el banco mostraba { $banco } y Treasury { $treasury }. Usa get_balance_sync_hints y los movimientos del período para encontrar el motivo y, si lo encuentras, regístralo con resolve_balance_sync.
contas-motivo = Motivo, si ya lo sabes
contas-motivo-exemplo = Ej.: la comisión de septiembre no vino en el OFX
contas-anotar-motivo = Anotar el motivo
contas-aceitar-sem-investigar = Aceptar sin investigar
contas-sincronias = Sincronías de saldo
contas-conferia = Coincidía con el banco
contas-ajuste-de = Ajuste de { $valor }
contas-investigar = Investigar

## Tipo de cambio manual de la cuenta

contas-cambio-explicacao = { $base ->
        [BRL] El patrimonio en BRL usa la PTAX cuando la recolección está activada
       *[other] El patrimonio en { $base } usa el tipo de cambio recolectado cuando la recolección está activada
    }; sin ella, o para corregirla, ingresa la tasa a mano. Las tasas de hace más de 7 días marcan el valor como estimado (≈).
contas-taxa-rotulo = 1 { $moeda } =
contas-taxa-dica = { $base ->
        [BRL] En reales, ej.: 5,4321
       *[other] En { $base }
    }
contas-cambio-salvo = Tipo de cambio guardado
contas-cambio-salvo-descricao = { $moeda }/{ $base } el { $data }
contas-salvar-taxa = Guardar tasa
contas-taxas-cambio = Tipos de cambio
contas-taxa-valor = 1 { $base } = { NUMBER($taxa, maximumFractionDigits: 6) } { $cotacao }
contas-informada-mao = ingresada a mano
contas-remover = Quitar
contas-nenhuma-taxa = Todavía no hay tasas { $moeda }/{ $base }.

## Historial de saldo

contas-periodo-grafico = Período del gráfico
contas-n-meses = { $n } meses
contas-desde-inicio = Desde el inicio
contas-incluir-previstos = Incluir previstos (próximos 2 meses)
contas-variacao-periodo = Variación en el período: { $valor }
contas-saldo-de = Saldo de { $nome }
contas-saldo-fonte = Fuente: saldo inicial + movimientos { $previstos ->
        [sim] efectuados y previstos
       *[nao] efectuados
    } · { $intervalo ->
        [dia] diario
       *[mes] fin de cada mes
    }
contas-saldo-grafico = Saldo de { $nome } del { $de } al { $ate }
contas-saldo-por-data = Saldo de { $nome } por fecha
contas-hoje = hoy

## Formulario de la cuenta

contas-erro-nome = Ingresa el nombre de la cuenta.
contas-erro-data = Ingresa la fecha del saldo inicial.
contas-conta-atualizada = Cuenta actualizada
contas-conta-criada = Cuenta creada
contas-nova-instituicao-opcao = + Nueva institución…
contas-editar-nome = Editar { $nome }
contas-criar-conta = Crear cuenta
contas-nome = Nombre
contas-nome-exemplo = Cuenta del banco, Nubank, Billetera…
contas-tipo-fixo = El tipo no cambia después de creada.
contas-moeda-fixa = La moneda no cambia después de creada.
contas-moeda-dica = Las cuentas en el exterior usan el tipo de cambio para el patrimonio en { $moeda }.
contas-fatura-inicial = Resumen abierto al inicio ({ $moeda })
contas-saldo-inicial-moeda = Saldo inicial ({ $moeda })
contas-na-data = En la fecha
contas-agencia = Sucursal
contas-agencia-dica = Sin el dígito, si el banco no lo usa.
contas-numero-conta = Cuenta
contas-numero-conta-dica = Con el dígito; reconoce la cuenta en los extractos OFX.
contas-membro-familia = Miembro de la familia
contas-sem-membros = Registra los miembros en la pantalla Familia.
contas-membro-explicacao = Con un solo miembro, los movimientos de esta cuenta quedan con él (se pueden cambiar uno por uno). Con dos o más, o ninguno, quedan de la familia.
contas-dia-fechamento = Día de cierre
contas-dia-vencimento = Día de vencimiento
contas-vence-mes-seguinte = Vence el mes siguiente al cierre.
contas-vence-mesmo-mes = Vence el mismo mes del cierre.
contas-conta-pagadora = Cuenta que paga el resumen
contas-bandeira = Marca
contas-bandeira-exemplo = Visa, Mastercard, Elo…
contas-final-cartao = Últimos 4 dígitos
contas-entra-patrimonio = Entra en el patrimonio neto
contas-entra-patrimonio-dica = Desactívalo para cuentas de terceros que solo sigues.
contas-notas = Notas
contas-apagar-nome = Eliminar { $nome }
contas-apagar-descricao = Solo se pueden eliminar cuentas sin movimientos, recurrencias ni cuotas. Para guardar el historial, archívala.
contas-apagar-conta = Eliminar cuenta
contas-conta-apagada = Cuenta eliminada

## Banco de la lista oficial

contas-banco-exemplo = Nombre o código del banco (ej.: 260, Itaú)
contas-banco-exemplo-exterior = Nombre del banco o de la institución
contas-plano-aposentadoria = Plan
contas-plano-aposentadoria-dica = El límite de aportes del año depende del plan.
contas-cobertura = Cobertura
contas-cobertura-individual = Individual
contas-cobertura-familia = Familiar
contas-aposentadoria-titulo = { $plano } en { $ano }
contas-aposentadoria-de = { $aportado } de { $limite }
contas-aposentadoria-sem-limite = { $aportado } en el año (sin límite anual)
contas-aposentadoria-falta = Todavía caben { $falta } este año.
contas-aposentadoria-acima = Los aportes del año superaron el límite: revísalo con el plan antes del plazo de corrección.
contas-aposentadoria-beneficio = Es el aporte que ya da todo el beneficio fiscal.
contas-aposentadoria-idade = Límite para { $idade } años.
contas-aposentadoria-divide = Comparte el límite con { $contas }.
contas-aposentadoria-ano-anterior = El límite de este año todavía no salió: vale el de { $ano }.
contas-aposentadoria-fonte = Fuente del límite
contas-nova-instituicao = Nueva institución
contas-bancos-bc = Bancos del Banco Central
contas-banco-oficial = Banco de la lista oficial; el código reconoce los extractos OFX.
contas-lista-oficial = Lista oficial del Banco Central de Brasil, actualizada cada semana.

## Sobregiro

contas-cheque-especial = Sobregiro
contas-tem-cheque-especial = La cuenta tiene límite de sobregiro
contas-cheque-especial-dica = El saldo puede quedar negativo hasta el límite; Treasury muestra cuánto está en uso y una estimación de intereses e IOF.
contas-juros-mes = Interés mensual (%)
contas-desconto = Descuento (%)
contas-desconto-dica = Sobre la tasa de interés.
contas-juros-ano = Interés anual (%)
contas-juros-ano-spread = Interés anual sobre el índice (%)
contas-juros-ano-dica = Puedes escribirlo con el índice, como lo informa el banco: 4,75 + TR.
contas-taxa-indexada-exemplo = ej.: 4,75 + TR
contas-indexador = Índice
contas-indexador-dica = Tasa variable: la tasa de arriba se suma al índice (TR, CDI, IPCA…).
contas-cheque-taxa-indexada = · tasa efectiva ≈ { $taxa }% al mes (con { $indice } de { $valor }% el { $data })
contas-cheque-taxa-indexada-estimada = · tasa efectiva ≈ { $taxa }% al mes (con { $indice } proyectado de { $valor }% el { $data })
contas-iof-basico = IOF básico (% por día)
contas-iof-adicional = IOF adicional (%)
contas-cet-mes = CET mensual (%)
contas-cet-ano = CET anual (%)
contas-cheque-uso = { $usado } de { $limite }
contas-cheque-disponivel = Disponible con el límite: { $valor }
contas-cheque-custo = · costo estimado en 30 días: { $valor }
contas-cheque-incerto = Cuenta sin extracto reciente: el saldo negativo puede ser un movimiento faltante, no necesariamente sobregiro.
contas-atualizacao-ha = actualizado hace { $dias } días
contas-atualizacao-incompleto = posiblemente incompleto
contas-atualizacao-ultimo = Último movimiento el { $data }.
contas-atualizacao-sem-lancamentos = Todavía no hay movimientos.
contas-atualizacao-mes-vazio = { $mes } quedó sin movimientos.
contas-atualizacao-negativo = El saldo negativo puede ser un movimiento faltante, no necesariamente sobregiro.
contas-cheque-custo-juros = · costo estimado en 30 días: { $valor } (interés de { $taxa }% mensual + IOF)

## Aportaciones de capital

contas-cotas-explicacao = En las cooperativas de crédito, el socio tiene aportaciones de capital: una participación que rinde según lo que decida la cooperativa (intereses al capital, excedentes). Actívalo para seguir el saldo de las aportaciones, los aportes que salen de esta cuenta y el rendimiento de cada período (variación del saldo menos los aportes).
contas-cotas-ativar = Activar aportaciones de capital en esta cuenta
contas-cotas-saldo = Saldo de las aportaciones
contas-cotas-aportes-periodo = Aportes en el período
contas-cotas-rentabilidade = Rendimiento
contas-cotas-nota = Nota
contas-cotas-pct = { NUMBER($valor, maximumFractionDigits: 2) }%
contas-cotas-em = el { $data }
contas-cotas-informe-saldo = ingresa el saldo
contas-cotas-aportes = Aportes
contas-cotas-depois-ultimo = { $valor } después del último saldo
contas-cotas-rentabilidade-apurada = Rendimiento calculado
contas-cotas-12-meses = Últimos 12 meses
contas-cotas-data-saldo = Fecha del saldo
contas-cotas-saldo-moeda = Saldo de las aportaciones ({ $moeda })
contas-cotas-nota-opcional = Nota (opcional)
contas-cotas-nota-exemplo = Ej.: extracto anual de la cooperativa
contas-cotas-saldo-informado = Saldo de las aportaciones registrado
contas-cotas-informar-saldo = Registrar saldo
contas-cotas-saldos = Saldos de las aportaciones de capital
contas-cotas-aportes-identificados = Aportes identificados
contas-cotas-aportes-identificados-n = Aportes identificados ({ $n })
contas-cotas-ja-fora = Los aportes ya estaban fuera de los gastos
contas-cotas-n-tirados = { $n ->
        [one] { $n } aporte sacado de los gastos
       *[other] { $n } aportes sacados de los gastos
    }
contas-cotas-tirar = Sacar los aportes de los gastos
contas-cotas-como-identificar = Cómo identificar los aportes
contas-cotas-texto-extrato = Texto en el extracto
contas-cotas-texto-dica = Separa con | (ej.: COTA CAPITAL|INTEGRALIZACAO).
contas-cotas-ou-categoria = O la categoría
contas-cotas-categoria = Categoría de los aportes
contas-cotas-nenhuma = Ninguna
contas-cotas-somar = Sumar el saldo de las aportaciones al patrimonio
contas-cotas-somar-dica = Entra como inversión, por el último saldo registrado.

## Resúmenes de la tarjeta

contas-pagar-fatura-de = Pagar el resumen de { $mes }
contas-pagar-fatura-descricao = Registra una transferencia del tipo “pago de tarjeta”: no cuenta como gasto (las compras ya contaron). Las fechas futuras quedan como previstas.
contas-pagamento-lancado = Pago registrado
contas-pagar = Pagar
contas-ja-pago = Ya pagado
contas-vencimento = Vencimiento
contas-pagar-com = Pagar con
contas-escolha-conta = Elige la cuenta
contas-fatura-de = Resumen de { $mes }
contas-ver-em-lancamentos = Ver en movimientos →
contas-compras = Compras
contas-compras-periodo = { $inicio } al { $fim }
contas-lancamentos = Movimientos
contas-total-informado = Total informado por el banco
contas-total-informado-salvo = Total informado guardado
contas-salvar-total-informado = Guardar total informado
contas-confere-calculado = Coincide con el calculado ✓
contas-diferenca-banco = Diferencia banco − calculado: { $valor }
contas-pagar-fatura = Pagar resumen
contas-lancamentos-fatura = Movimientos del resumen
contas-pagamento = Pago
contas-sem-categoria = sin categoría
contas-fatura = Resumen
contas-situacao = Estado
contas-banco-calc = Banco × calc.
contas-confere-curto = coincide
contas-faturas-vazio-kicker = Sin resúmenes
contas-faturas-vazio-titulo = Todavía no hay compras con esta tarjeta
contas-faturas-vazio-descricao = Los resúmenes aparecen solos a partir de las compras, según el día de cierre y de vencimiento.

## Cuotas y compromisos futuros de la tarjeta

contas-a-pagar = A pagar
contas-compromissos-titulo = Compromisos futuros de la tarjeta, 12 meses
contas-compromissos-fonte = Fuente: resúmenes abiertos, con cuotas futuras y recurrencias previstas
contas-compromissos-grafico = Monto a pagar por mes en los próximos resúmenes
contas-compromissos-tabela = Compromisos futuros de la tarjeta por mes
contas-compras-parceladas = Compras en cuotas
contas-sem-parceladas = Ninguna compra en cuotas. Registra una en Movimientos › Nuevo movimiento › En cuotas con tarjeta.
contas-compra-em = compra del { $data }
contas-parcelas-pagas = { $pagas }/{ $total } pagadas
contas-falta = falta
contas-apagar-parcelamento = Eliminar compra en cuotas
contas-apagar-parcelamento-descricao = Todas las cuotas (pasadas y futuras) salen de la tarjeta. Puedes deshacerlo desde el historial.
contas-apagar-parcelas = Eliminar cuotas
contas-n-parcelas-apagadas = { $n ->
        [one] { $n } cuota eliminada
       *[other] { $n } cuotas eliminadas
    }

## Lista y página de la cuenta

contas-lista = Cuentas
contas-col-conta = Cuenta
contas-col-membros = Miembros
contas-col-saldo = Saldo
contas-sem-membro = Familia
contas-voltar-lista = Todas las cuentas
contas-detalhe-resumo = Resumen de la cuenta
contas-ultimos-lancamentos = Movimientos
contas-col-descricao = Descripción
contas-col-categoria = Categoría
contas-col-valor = Monto
contas-sem-lancamentos = Todavía no hay movimientos en esta cuenta.
contas-ver-todos-lancamentos = Ver todos en Movimientos

## Ajuste de saldo

contas-ajustar-saldo = Ajustar saldo
contas-ajuste-explicacao = Indica el saldo que la cuenta tenía en una fecha (el del banco o del extracto). Treasury registra la diferencia como ajuste: cuenta en el saldo y el historial de la cuenta, pero queda fuera de ingresos, gastos y presupuesto. Los movimientos hasta esa fecha quedan conciliados.
contas-ajuste-conta = Cuenta
contas-ajuste-data = Fecha del saldo
contas-ajuste-saldo = Saldo en esa fecha ({ $moeda })
contas-ajuste-saldo-dica = Lo que muestra el banco al final del día.
contas-ajuste-previa = Vista previa del ajuste
contas-ajuste-calculado = Calculado el { $data }
contas-ajuste-informado = Indicado
contas-ajuste-valor = Ajuste
contas-ajuste-confere = Ya coincide: solo marca la fecha como conciliada.
contas-ajuste-fora-relatorios = Queda fuera de ingresos, gastos y presupuesto.
contas-ajuste-sem-diferenca = Saldo confirmado

## Tarjetas de la cuenta de crédito: titular, adicionales y virtuales
contas-cartoes = Tarjetas
contas-plasticos-explicacao = Cada tarjeta (la tuya, las adicionales de la familia y las virtuales) tiene sus últimos dígitos y la persona que la usa. El resumen y el límite siguen siendo uno; cada compra queda con quien usó la tarjeta.
contas-plasticos-vazio = Ninguna tarjeta registrada. Registra la tuya y las adicionales para saber quién gastó.
contas-plastico-tipo = Tipo
contas-plastico-tipo-holder = Titular
contas-plastico-tipo-additional = Adicional
contas-plastico-tipo-virtual = Virtual
contas-plastico-tipo-temporary = Temporal
contas-plastico-final = Terminada en { $final }
contas-plastico-final-rotulo = Últimos dígitos
contas-plastico-final-dica = Solo los 4 últimos dígitos.
contas-plastico-membro = Quién la usa
contas-plastico-titular-conta = Titular de la cuenta
contas-plastico-do-titular = Titular de la cuenta ({ $nome })
contas-plastico-apelido = Apodo
contas-plastico-apelido-exemplo = Ej.: suscripciones, viaje
contas-plastico-sublimite = Sublímite por resumen
contas-plastico-sublimite-dica = Solo un aviso: la app no bloquea compras.
contas-plastico-gasto-ciclo = En este resumen:
contas-plastico-uso-sublimite = { $pct }% del sublímite de
contas-plastico-acima-sublimite = pasó el sublímite
contas-plastico-perto-sublimite = cerca del sublímite
contas-plastico-novo = Nueva tarjeta
contas-plastico-dialogo-descricao = Las compras hechas con esta tarjeta quedan con la persona elegida.
contas-plastico-salvar = Guardar
contas-plastico-salvo = Tarjeta guardada
contas-plastico-editar = Editar
contas-plastico-ativo = Activa
contas-plastico-inativo = Inactiva
contas-plastico-remover = Eliminar
contas-plastico-remover-titulo = ¿Eliminar { $cartao }?
contas-plastico-remover-confirmar = Las compras hechas con ella vuelven al titular de la cuenta.
contas-plastico-removido = Tarjeta eliminada
contas-fatura-por-cartao = Por tarjeta
contas-fatura-sem-cartao = Sin tarjeta indicada

## Límite, fechas del resumen y anualidad
contas-limite-anuidade = Límite y anualidad
contas-limite-uso = Uso del límite
contas-limite-usado = Usado:
contas-limite-de-total = de
contas-limite-disponivel = Disponible:
contas-limite-parcelas-futuras = Las cuotas futuras ya ocupan
contas-limite-sem = Esta tarjeta no tiene límite registrado.
contas-melhor-dia = Mejor día de compra: { $data }. Lo que compres desde ese día vence el { $vencimento } ({ $dias } días para pagar).
contas-limite-historico = Historial del límite
contas-limite-historico-vazio = Sin registros de límite.
contas-limite-novo = Registrar cambio de límite
contas-limite-novo-descricao = ¿El banco subió o bajó el límite? Registra la fecha y el nuevo monto.
contas-limite-salvo = Límite registrado
contas-limite-da-fatura = Del resumen
contas-limite-informado = Informado
contas-anuidade = Anualidad
contas-anuidade-vazio = Indica la anualidad de la tarjeta para que la app revise los cobros, muestre cuánto falta para la exención y te avise antes de la renovación.
contas-anuidade-informar = Indicar anualidad
contas-anuidade-mudar = Cambiar regla
contas-anuidade-sem = Sin anualidad
contas-anuidade-sem-titulo = ¿Esta tarjeta ya no tiene anualidad?
contas-anuidade-sem-descricao = La regla termina hoy. Los cobros anteriores siguen en el historial.
contas-anuidade-regra = Anualidad de la tarjeta
contas-anuidade-regra-descricao = Lo acordado con el banco. Cambiar la regla crea una nueva desde la fecha de inicio.
contas-anuidade-salva = Anualidad guardada
contas-anuidade-valor-ano = Monto anual
contas-anuidade-parcelas = Cuotas
contas-anuidade-em-parcelas = en { $n }× de
contas-anuidade-mes = Mes de renovación
contas-anuidade-mes-nao-sei = No sé
contas-anuidade-inicio = Vale desde
contas-anuidade-isencao = Exención
contas-anuidade-isencao-none = Sin exención
contas-anuidade-isencao-full = Exenta
contas-anuidade-isencao-spend = Por gasto en el resumen
contas-anuidade-isencao-investment = Por monto invertido en el banco
contas-anuidade-isencao-discount = Descuento fijo
contas-anuidade-gasto-minimo = Gasto en el resumen que la exime
contas-anuidade-investimento-minimo = Monto invertido que la exime
contas-anuidade-desconto = Descuento (%)
contas-anuidade-notas = Observaciones
contas-anuidade-pago-12m = Pagado en 12 meses
contas-anuidade-renova = Se renueva el
contas-anuidade-isenta-agora = El resumen abierto ya alcanzó la exención.
contas-anuidade-falta-gastar = Falta gastar en este resumen para eximir la cuota:
contas-anuidade-falta-investir = Falta invertir en el banco para la exención:
contas-anuidade-cobrancas = Cobros de anualidad
contas-anuidade-sem-cobrancas = Ningún cobro de anualidad en los últimos 12 meses.
contas-anuidade-esperado = Esperado:
contas-anuidade-problema-isenta = Cobrada en un resumen que había alcanzado la exención.
contas-anuidade-problema-valor = Monto distinto de la cuota acordada.
contas-anuidade-problema-parcela = Pasa el número de cuotas del año.
contas-fatura-ajustar-botao = Ajustar fechas y montos
contas-fatura-ajustar = Resumen de { $mes }
contas-fatura-ajustar-descricao = Úsalo cuando el banco posterga el cierre o cambia el vencimiento de un mes. Las fechas ajustadas ya no siguen el ciclo de la tarjeta.
contas-fatura-datas-calculadas = Volver a las fechas del ciclo
contas-fatura-datas-manuais = ajustado
contas-fatura-datas-impressas = del resumen del banco
contas-fatura-fechamento = Cierre
contas-fatura-minimo = Pago mínimo
contas-fatura-limite = Límite en el resumen
contas-fatura-saldo-anterior = Resumen anterior
contas-fatura-alterada = Resumen modificado
contas-rotativo-titulo = El resumen de { $mes } entró en crédito rotativo
contas-rotativo-sem-pagar = Quedó sin pagar:
contas-rotativo-encargos = Intereses y cargos desde entonces:
contas-rotativo-regras-br = El rotativo solo puede durar hasta el próximo resumen, y los intereses y cargos del rotativo y de las cuotas del resumen no pueden superar el 100% de la deuda original (Ley 14.690/2023).
contas-rotativo-regras = El saldo que queda en la tarjeta suele tener intereses altos; cancelarlo o cambiarlo por un préstamo más barato suele convenir.
contas-rotativo-acima-teto = Los cargos ya superan la deuda original: revisa el resumen con el banco.
contas-rotativo-alem-da-fatura = Ya pasó más de un resumen en rotativo: el banco debería haber ofrecido el pago en cuotas.

## Cashback y puntos
contas-recompensa-aba = Cashback y puntos
contas-recompensa-titulo = Cashback y puntos
contas-recompensa-vazio = Indica el cashback o los puntos de la tarjeta para que la app calcule lo previsto de cada resumen, revise lo que llegó y avise cuando el crédito no llegue.
contas-recompensa-informar = Indicar programa
contas-recompensa-sem = Sin programa
contas-recompensa-sem-titulo = ¿Esta tarjeta ya no tiene cashback ni puntos?
contas-recompensa-regra = Cashback y puntos de la tarjeta
contas-recompensa-regra-descricao = Lo que la tarjeta devuelve en cada compra. Cambiar la regla crea una nueva desde la fecha de inicio.
contas-recompensa-salva = Programa guardado
contas-recompensa-tipo = Tipo
contas-recompensa-cashback = Cashback
contas-recompensa-pontos = Puntos o millas
contas-recompensa-taxa = Tasa básica
contas-recompensa-taxa-cashback = Cashback (%)
contas-recompensa-taxa-pontos = Puntos por unidad de moneda
contas-recompensa-pontos-por = { $n } puntos por { $moeda }
contas-recompensa-moeda-pontos = Moneda de la regla
contas-recompensa-moeda-pontos-dica = Muchos programas cuentan puntos por dólar gastado.
contas-recompensa-teto-cashback = Tope por resumen
contas-recompensa-teto-pontos = Tope de puntos por resumen
contas-recompensa-destino = Dónde llega el cashback
contas-recompensa-destino-fatura = Crédito en el resumen
contas-recompensa-destino-conta = En una cuenta
contas-recompensa-destino-investimento = En una inversión
contas-recompensa-conta-destino = Cuenta que recibe
contas-recompensa-prazo = Plazo del crédito (días)
contas-recompensa-prazo-dica = Días después del cierre en que suele llegar el cashback.
contas-recompensa-por-categoria = Tasas por categoría
contas-recompensa-taxa-categoria = Tasa de la categoría
contas-recompensa-nova-categoria = Agregar categoría
contas-recompensa-previsto-12m = Previsto en 12 meses
contas-recompensa-recebido-12m = Recibido en 12 meses
contas-recompensa-como-conta = El cashback cuenta como ingreso (categoría Cashback y recompensas), sin bajar el gasto de las compras. Los créditos con "cashback" en la descripción también cuentan.
contas-recompensa-por-fatura = Por resumen
contas-recompensa-previsto = Previsto
contas-recompensa-recebido = Recibido
contas-recompensa-nao-veio = no llegó hasta el { $data }
contas-recompensa-n-pontos = { $n } puntos

## Puntos y millas
contas-pontos-titulo = Puntos y millas
contas-pontos-vazio = Registra los programas de puntos y millas de la familia para seguir el saldo de cada persona, los vencimientos y las transferencias con bonificación.
contas-pontos-fora-patrimonio = Los puntos no entran en el patrimonio; el valor estimado usa el valor por 1.000 puntos que indiques.
contas-pontos-programa = Programa
contas-pontos-programa-novo = Nuevo programa
contas-pontos-programa-descricao = Un programa por persona: la cuenta de puntos es de quien la tiene.
contas-pontos-programa-salvo = Programa guardado
contas-pontos-nome = Nombre
contas-pontos-membro = De quién es
contas-pontos-sem-membro = Sin miembro
contas-pontos-valor-milheiro = Valor de 1.000 puntos
contas-pontos-valor-milheiro-dica = Tu estimación, para ver cuánto vale el saldo. No entra en el patrimonio.
contas-pontos-tipo-bank = Banco
contas-pontos-tipo-airline = Aerolínea
contas-pontos-tipo-retail = Comercio
contas-pontos-tipo-other = Otro
contas-pontos-mov-earn = Acumulación
contas-pontos-mov-bonus = Bonificación
contas-pontos-mov-purchase = Compra de puntos
contas-pontos-mov-transfer_in = Transferencia recibida
contas-pontos-mov-transfer_out = Transferencia enviada
contas-pontos-mov-redeem = Canje
contas-pontos-mov-expire = Vencimiento
contas-pontos-mov-adjust = Ajuste
contas-pontos-movimento-novo = Registrar puntos
contas-pontos-movimento-descricao = Acumulación, bonificación, compra, canje o vencimiento, como aparece en el extracto del programa.
contas-pontos-movimento-salvo = Puntos registrados
contas-pontos-quantidade = Puntos
contas-pontos-vence-em = Vencen el
contas-pontos-vence = vencen el { $data }
contas-pontos-custo = Cuánto costó
contas-pontos-transferir = Transferir
contas-pontos-transferir-descricao = Los puntos salen de un programa y entran en otro, con la bonificación de la campaña.
contas-pontos-transferencia-salva = Transferencia registrada
contas-pontos-de = De
contas-pontos-para = A
contas-pontos-saem = Puntos que salen
contas-pontos-bonus = Bonificación (%)
contas-pontos-entram = Entran { $n } puntos en el destino.
contas-pontos-sem-movimentos = Ningún movimiento todavía.
contas-pontos-n = { $n } puntos
contas-pontos-proximo-vencimento = { $n } vencen el { $data }
contas-pontos-cartoes = Acumula con: { $cartoes }
contas-recompensa-programa-pontos = Programa de puntos
contas-recompensa-programa-pontos-nenhum = Ninguno (solo lo previsto)

## ¿Vale la pena esta tarjeta?
contas-vale-titulo = ¿Vale la pena esta tarjeta?
contas-vale-periodo = Del { $de } al { $ate }: lo que costó la tarjeta y lo que devolvió.
contas-vale-anuidade = Anualidad
contas-vale-encargos = Intereses y cargos
contas-vale-iof = IOF
contas-vale-cashback = Cashback recibido
contas-vale-pontos = Puntos (valor estimado)
contas-vale-resultado = Resultado
contas-vale-simulacao = Si todas las compras de la familia fueran a una sola tarjeta
contas-vale-simulacao-descricao = Retorno según las reglas de cada tarjeta sobre las compras de todas las tarjetas en los últimos 12 meses:
contas-vale-retorno = Retorno
contas-vale-aviso = Es un análisis de los datos de la familia, no una recomendación de producto financiero. Beneficios como seguros y salas VIP no entran en la cuenta.
