# Informes (núcleo, `reports`): etiquetas de los grupos y del flujo del dinero, las fórmulas y las
# fuentes de "Explicar" (`explain_value`), los próximos vencimientos y los errores. Se arman en el
# momento, en el idioma de la instalación. Errores en minúsculas, como frase de registro (la
# interfaz pone la inicial en mayúscula).
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Períodos y base de fechas

nucleo-relatorios-erro-mes-inicial-posterior = el mes inicial ({ $de }) es posterior al final ({ $ate })
nucleo-relatorios-erro-periodo-longo = período demasiado largo (máximo de { $maximo } meses)
nucleo-relatorios-erro-data-inicial-posterior = la fecha inicial ({ $de }) es posterior a la final ({ $ate })
nucleo-relatorios-erro-calendario-longo = período demasiado largo para el calendario (máximo de 10 años)
nucleo-relatorios-base-competencia = devengo
nucleo-relatorios-base-caixa = caja

## Grupos y vencimientos

nucleo-relatorios-sem-categoria = Sin categoría
nucleo-relatorios-sem-membro = Sin miembro
nucleo-relatorios-outras = Otras
nucleo-relatorios-sem-descricao = (sin descripción)
# Próximos vencimientos: el resumen de una tarjeta ("Resumen Nubank (oct/2026)").
nucleo-relatorios-fatura = Resumen { $cartao } ({ $mes })

## A dónde fue el dinero (diagrama de flujo y los ítems de cada nodo)

nucleo-relatorios-fluxo-orcamento = Presupuesto
nucleo-relatorios-fluxo-receitas-sem-categoria = Ingresos sin categoría
nucleo-relatorios-fluxo-resgates = Rescates
nucleo-relatorios-fluxo-deficit = Déficit
nucleo-relatorios-fluxo-dividas = Pago de deudas
nucleo-relatorios-fluxo-investimentos = Inversiones
nucleo-relatorios-fluxo-sobra = Sobrante
nucleo-relatorios-fluxo-sem-subcategoria = { $categoria } (sin subcategoría)
nucleo-relatorios-fluxo-receita-de = El ingreso de { $categoria }
nucleo-relatorios-fluxo-pagamento-de = El pago de { $contrato }
nucleo-relatorios-fluxo-entradas = Entradas
nucleo-relatorios-fluxo-saidas = Salidas
nucleo-relatorios-fluxo-sem-cambio = Sin tipo de cambio de { $de } a { $para } en la fecha: queda fuera del total.
nucleo-relatorios-fluxo-grupo-negativo = { $grupo } sumó { $valor } en el período (reembolsos mayores que los movimientos); el diagrama deja fuera los grupos sin saldo positivo.
nucleo-relatorios-fluxo-sem-sobra-deficit = No hubo sobrante en el período: lo que salió superó a lo que entró en { $valor }.
nucleo-relatorios-fluxo-sem-sobra = No hubo sobrante en el período.
nucleo-relatorios-fluxo-sem-deficit-sobra = No hubo déficit en el período: sobraron { $valor }.
nucleo-relatorios-fluxo-sem-deficit = No hubo déficit en el período.
nucleo-relatorios-erro-no-desconhecido = nodo del flujo desconocido: { $no }
nucleo-relatorios-erro-sem-nos = indica al menos un nodo del flujo
nucleo-relatorios-erro-nos-sobrepostos = los nodos pedidos se superponen (una categoría y una subcategoría suya)

## "Explicar" (`explain_value`): fuentes de cada número

nucleo-relatorios-conta = cuenta { $id }
nucleo-relatorios-fonte-cambio = tipo de cambio: 1 { $moeda } = { $taxa } { $alvo }
nucleo-relatorios-fonte-cambio-estimado = tipo de cambio: 1 { $moeda } = { $taxa } { $alvo } (estimado)
nucleo-relatorios-fonte-saldo-inicial = saldo inicial de { $conta }
# Una cuenta (o movimiento) en otra moneda: "Cuenta EE. UU.: USD 100,00 × 5,43 = R$ 543,00".
nucleo-relatorios-fonte-conta-convertida = { $conta }: { $valor } × { $taxa } = { $convertido }
nucleo-relatorios-fonte-conta-sem-cambio = { $conta }: { $valor } (sin tipo de cambio, fuera del total)
nucleo-relatorios-fonte-convertida = { $rotulo } × { $taxa } = { $convertido }
nucleo-relatorios-fonte-carteira = Cartera de inversiones (valor de mercado o en la curva; consulta la cartera)
nucleo-relatorios-fonte-estimativa = estimación
nucleo-relatorios-fonte-avaliacao = tasación
nucleo-relatorios-fonte-dividas = Préstamos y financiamientos (saldo deudor según el cronograma)
nucleo-relatorios-fonte-partes =
    { $n ->
        [one] { $rotulo } (parte de la división)
       *[other] { $rotulo } ({ $n } partes de la división)
    }
nucleo-relatorios-fonte-fatura = { $rotulo } (compra el { $compra }, resumen con vencimiento el { $vencimento })
nucleo-relatorios-fonte-lancamento-sem-cambio = { $rotulo }: { $valor } sin tipo de cambio, fuera del total

## "Explicar": fórmulas

nucleo-relatorios-n-efetivados =
    { $n ->
        [one] { $n } movimiento efectuado
       *[other] { $n } movimientos efectuados
    }
nucleo-relatorios-n-despesas =
    { $n ->
        [one] { $n } movimiento de gasto efectuado
       *[other] { $n } movimientos de gasto efectuados
    }
nucleo-relatorios-n-receitas =
    { $n ->
        [one] { $n } movimiento de ingreso efectuado
       *[other] { $n } movimientos de ingreso efectuados
    }
# Va al final de una fórmula (empieza con el punto y coma).
nucleo-relatorios-formula-sem-cambio = ; sin tipo de cambio para { $moedas } (fuera del total)
nucleo-relatorios-formula-saldo = saldo inicial de { $inicial } + { $lancamentos } hasta el { $data } ({ $soma }) = { $valor }
nucleo-relatorios-formula-saldo-abertura = { $lancamentos } hasta el { $data } = { $valor } (la cuenta se abrió el { $abertura }; el saldo inicial de { $inicial } solo cuenta a partir de esa fecha)
nucleo-relatorios-formula-patrimonio =
    activos de { $ativos } − pasivos de { $passivos } = { $valor }, sumando el saldo de { $n ->
        [one] { $n } cuenta
       *[other] { $n } cuentas
    } al { $data } (saldo inicial + movimientos efectuados hasta la fecha; los saldos negativos, como la deuda de las tarjetas, son pasivos), la cartera, los bienes físicos (tasación o estimación) y las deudas{ $sem_cambio }
# Filtros de la fórmula de gastos e ingresos, después del período ("… (devengo) en Vivienda y
# subcategorías de Ana: …").
nucleo-relatorios-filtro-categoria = en { $categoria } y subcategorías
nucleo-relatorios-filtro-membro = de { $membro }
nucleo-relatorios-regra-despesa = gasto = −suma de los montos (los reembolsos reducen el total)
nucleo-relatorios-regra-receita = ingreso = suma de los montos
nucleo-relatorios-formula-convertidos = ; montos en otra moneda convertidos a { $moeda } al tipo de cambio de la fecha del movimiento
nucleo-relatorios-formula-fluxo = { $lancamentos } del { $de } al { $ate } ({ $base }){ $filtros }: { $regra } = { $valor }; no entran transferencias, pagos de resumen ni aportes; los movimientos divididos cuentan por sus partes{ $convertidos }{ $sem_cambio }
nucleo-relatorios-formula-poupanca = tasa de ahorro de { $mes } ({ $base }) = (ingresos de { $receitas } − gastos de { $despesas }) ÷ ingresos de { $receitas } = { $taxa }; monto = ingresos − gastos = { $valor }, de { $lancamentos }{ $sem_cambio }
nucleo-relatorios-formula-poupanca-sem-receitas = sin ingresos en { $mes }, la tasa de ahorro no aplica; ingresos − gastos = { $valor }, de { $lancamentos }{ $sem_cambio }

## A dónde van los ingresos (grupos de gasto de las categorías; nombres de los grupos en grupos.ftl)

# Lo que entra en Futuro sin categoría: aportes a inversiones y transferencias a las cuentas de
# las metas.
nucleo-relatorios-grupos-aportes = Aportes
nucleo-relatorios-grupos-metas = Transferencias a metas
# Sección del informe del mes en PDF.
nucleo-relatorios-grupos-pdf-titulo = A dónde fueron los ingresos
nucleo-relatorios-grupos-pdf-dica = Gastos del mes por el grupo de cada categoría; el futuro incluye los aportes y las metas.
nucleo-relatorios-grupos-pdf-parte = { $parte } de los ingresos
nucleo-relatorios-grupos-pdf-sem-renda = sin ingresos en el mes
nucleo-relatorios-grupos-pdf-meta = meta { $meta }
nucleo-relatorios-grupos-pdf-sem-meta = sin meta
