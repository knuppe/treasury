# Informes (src/features/reports): los controles que también usan las pantallas principales (mes,
# devengo × caja, cargando, error) y el "Explicar" de cada valor (docs/i18n-design.md).
# Revisión: borrador por IA, sin revisión nativa.

## Controles

relatorios-competencia = Devengo
relatorios-competencia-dica = Por la fecha de la compra
relatorios-caixa = Caja
relatorios-caixa-dica = Por la fecha del pago (el vencimiento del resumen en las tarjetas)
relatorios-base-datas = Base de fechas
relatorios-mes-anterior = Mes anterior
relatorios-proximo-mes = Mes siguiente
relatorios-falha-carregar = No se pudo cargar.
relatorios-erro-carregar = Error al cargar
relatorios-tentar-de-novo = Intentar de nuevo
relatorios-sem-dados-periodo = Sin datos en el período

## Explicar un valor

relatorios-explicar = Explicar { $titulo }
relatorios-explicar-dica = Explicar: { $titulo }
relatorios-explicacao = Explicación
relatorios-explicacao-de = Explicación · { $titulo }
relatorios-ver-lancamentos = Ver movimientos
relatorios-calculando-origem = Calculando el origen
relatorios-fontes = Fuentes
relatorios-itens = { $n ->
        [one] { $n } elemento
       *[other] { $n } elementos
    }
relatorios-mostrando = mostrando { $n }
relatorios-formula = Fórmula
relatorios-inclui-estimado = ≈ Incluye un valor estimado (tipo de cambio o cotización del día anterior).
relatorios-sem-cambio = Sin tipo de cambio para { $moedas }: esos valores quedaron fuera del total.
relatorios-fontes-valor = Fuentes del valor
relatorios-nenhuma-fonte = Ninguna fuente: el valor es cero.
relatorios-lista-recentes = La lista muestra los { $n } movimientos más recientes; la suma completa está en la fórmula.
relatorios-descricao = Descripción
relatorios-taxa = tasa
relatorios-fonte-lancamento = Movimiento
relatorios-fonte-conta = Cuenta
relatorios-fonte-cambio = Tipo de cambio
relatorios-fonte-saldo-inicial = Saldo inicial
relatorios-fonte-indice = Índice
relatorios-fonte-carteira = Cartera
relatorios-fonte-divida = Deuda
relatorios-fonte-bem-fisico = Bien físico

## Pantalla 08 · Informes

relatorios-descricao-tela = Gastos en todas sus formas: a dónde fueron, qué pesa más, cuándo, si están subiendo, dónde te pasaste y cuánto ya está comprometido. Haz clic en cualquier parte para ver los movimientos.
relatorios-descricao-ir = Cálculo del impuesto sobre la renta en renta variable: el impuesto del mes, el DARF y los valores para la declaración anual.
relatorios-descricao-saude = Salud financiera: una nota de 0 a 100 calculada con tus datos, sus dimensiones, cada indicador × meta y qué mejorar.
relatorios-imposto-renda = Impuesto sobre la renta
relatorios-saude-financeira = Salud financiera
relatorios-visao-frente = Mirando hacia adelante
relatorios-visualizacoes = Vistas
relatorios-aba-aria = { $vista }: { $pergunta }
relatorios-aba-ir-sigla = IR
relatorios-aba-ir-dica = Impuesto sobre la renta, DARF y declaración
relatorios-aba-ir-aria = Impuesto sobre la renta: cálculo, DARF y declaración
relatorios-aba-saude-sigla = SF
relatorios-aba-saude-dica = Nota de salud financiera, dimensiones y sugerencias
relatorios-aba-saude-aria = Salud financiera: nota, indicadores y sugerencias

## Vistas (pestaña y pregunta)

relatorios-vista-fluxo = Flujo
relatorios-vista-fluxo-pergunta = ¿A dónde va el ingreso?
relatorios-vista-composicao = Composición
relatorios-vista-composicao-pergunta = ¿Qué pesa más en el presupuesto?
relatorios-vista-ritmo = Ritmo
relatorios-vista-ritmo-pergunta = ¿En qué días y semanas gastamos más?
relatorios-vista-tendencia = Tendencia
relatorios-vista-tendencia-pergunta = ¿Los gastos están subiendo? ¿En qué?
relatorios-vista-orcamento = Presupuesto
relatorios-vista-orcamento-pergunta = ¿Dónde nos pasamos?
relatorios-vista-comparacao = Comparación
relatorios-vista-comparacao-pergunta = ¿Qué cambió de un período a otro?
relatorios-vista-inflacao = Inflación
relatorios-vista-inflacao-pergunta = ¿Gastamos más o solo se puso más caro?
relatorios-vista-titulares = Miembros
relatorios-vista-titulares-pergunta = ¿Quién gasta en qué?
relatorios-vista-recorrentes = Recurrentes
relatorios-vista-recorrentes-pergunta = ¿Cuánto está comprometido cada mes?
relatorios-vista-cartoes = Tarjetas
relatorios-vista-cartoes-pergunta = ¿Cuánto ya está comprometido en los próximos meses?
relatorios-vista-patrimonio = Patrimonio
relatorios-vista-patrimonio-pergunta = ¿El patrimonio está creciendo?

## Períodos y filtros

relatorios-periodo-mes = Este mes
relatorios-periodo-mes-anterior = Mes pasado
relatorios-periodo-trimestre = Este trimestre
relatorios-periodo-trimestre-anterior = Trimestre pasado
relatorios-periodo-3m = Últimos 3 meses
relatorios-periodo-6m = Últimos 6 meses
relatorios-periodo-ano = Este año
relatorios-periodo-12m = Últimos 12 meses
relatorios-periodo-ano-anterior = Año pasado
relatorios-periodo-personalizado = Personalizado
# Período de varios meses o de fechas sueltas: "enero de 2026 a marzo de 2026".
relatorios-periodo-intervalo = { $de } a { $ate }
relatorios-filtros = Filtros de los informes
relatorios-periodo = Período
relatorios-periodo-anterior = Período anterior
relatorios-proximo-periodo = Período siguiente
relatorios-de = Desde
relatorios-ate = Hasta
relatorios-data-inicial = Fecha inicial
relatorios-data-final = Fecha final
relatorios-base = Base
relatorios-membro = Miembro
relatorios-conta = Cuenta
relatorios-todos = Todos
relatorios-todas = Todas
# Base de fechas en medio de la línea de fuente ("Fuente: … · caja · …").
relatorios-base-caixa = caja
relatorios-base-competencia = devengo
relatorios-filtro-nota-membro-conta = Esta figura muestra a toda la familia: el filtro de miembro y cuenta se aplica a Composición.
relatorios-filtro-nota-membro = Esta figura muestra a toda la familia: el filtro de miembro se aplica a Composición.
relatorios-filtro-nota-conta = Esta figura muestra a toda la familia: el filtro de cuenta se aplica a Composición.

## Piezas comunes de las figuras

relatorios-outras = Otras
relatorios-sem-categoria = Sin categoría
relatorios-direto = { $categoria } (directo)
relatorios-categoria = Categoría
relatorios-receita = Ingreso
relatorios-receitas = Ingresos
relatorios-despesas = Gastos
relatorios-total = Total
relatorios-mes = Mes
relatorios-grupo = Grupo
relatorios-forma = Forma
relatorios-novo = nuevo
relatorios-nenhum = ninguno
relatorios-pct-receita = % del ingreso
relatorios-pct-total = % del total
relatorios-da-receita = Del ingreso
relatorios-do-total = { $pct } del total
relatorios-item-em = { $nome } en { $periodo }
relatorios-receitas-de = Ingresos de { $periodo }
relatorios-despesas-de = Gastos de { $periodo }
relatorios-fonte-sem-cambio = { $fonte } · sin tipo de cambio: { $moedas }
relatorios-fonte-despesas = Fuente: movimientos de gasto efectivados · { $base }
relatorios-fonte-despesas-filtro = Fuente: movimientos de gasto efectivados · { $base } · con filtro de miembro/cuenta

## 01 · Flujo

relatorios-carregando-fluxo = Trazando el flujo
relatorios-detalhe = Detalle
relatorios-detalhe-categorias = Categorías
relatorios-detalhe-categorias-dica = Solo las categorías de primer nivel
relatorios-detalhe-subcategorias = Subcategorías
relatorios-detalhe-subcategorias-dica = Categorías y subcategorías
relatorios-grupo-in = Entradas
relatorios-grupo-out = Salidas
relatorios-grupo-result = Resultado
relatorios-vs-anterior = vs anterior:
relatorios-periodo-anterior-dica = Período anterior: { $periodo }
relatorios-ver-compoe-dica = Ver los movimientos que componen este valor
relatorios-ver-compoe-sr = (ver qué compone { $item })
relatorios-pontos = { $valor } p.p.
relatorios-para = Hacia
relatorios-anterior = Anterior
relatorios-o-periodo-anterior = el período anterior
relatorios-diferenca-de = la diferencia de { $nome }
relatorios-sobra = Sobrante
relatorios-deficit = Déficit
relatorios-fluxo-pergunta = A dónde va el ingreso: lo que entra (ingresos y rescates) pasa por el presupuesto y sale en gastos por categoría, pago de deudas, inversiones y el sobrante.
relatorios-fluxo-pergunta-comparado = A dónde va el ingreso: lo que entra (ingresos y rescates) pasa por el presupuesto y sale en gastos por categoría, pago de deudas, inversiones y el sobrante, comparado con { $periodo }.
relatorios-taxa-poupanca = Tasa de ahorro
relatorios-taxa-poupanca-de = Tasa de ahorro de { $periodo }
relatorios-no-periodo = en el período
relatorios-investimentos-aportes = Inversiones (aportes) { $valor }
relatorios-resgates = Rescates { $valor }
relatorios-pagamento-dividas = Pago de deudas { $valor } (dentro de los gastos)
relatorios-fluxo-em-andamento = El período todavía está en curso (hasta hoy) y se compara con { $periodo } completo. Para comparar períodos cerrados, usa las flechas o elige “Mes pasado”.
relatorios-fluxo-titulo = Flujo · { $periodo }
relatorios-fluxo-fonte = Fuente: movimientos efectivados · { $base } · las transferencias entre cuentas y el pago de la tarjeta no entran; aportes y rescates aparte
# $investimentos y $sobra: "sim" (sí) o "nao" (no).
relatorios-fluxo-aria = Flujo de { $periodo }: ingresos divididos entre { $n } categorías de gasto{ $investimentos ->
        [sim] , inversiones
       *[nao] {""}
    }{ $sobra ->
        [sim] {" "}y el sobrante
       *[nao] , con déficit
    }.
relatorios-sem-movimento = Sin movimiento en el período
relatorios-fluxo-tabela = Flujo: origen, destino y valor
relatorios-comparativo-titulo = Comparativo · { $anterior } × { $atual }
relatorios-comparativo-tabela = Comparativo del flujo: { $anterior } y { $atual }
relatorios-comparativo-dica = Haz clic en un valor para ver los movimientos que lo componen (en el mes, en el anterior o en la diferencia).

## 02 · Composición

relatorios-carregando-composicao = Sumando las categorías
relatorios-composicao-pergunta = Qué pesa más: cada bloque es una categoría, dividida en sus subcategorías, del tamaño del gasto.
relatorios-maior-categoria = Categoría más grande
relatorios-peso-maior = Peso de la más grande
relatorios-composicao-titulo = Composición · { $periodo }
relatorios-forma-treemap = Treemap
relatorios-forma-sunburst = Sunburst
relatorios-composicao-aria = Composición de los gastos de { $periodo }: { $maiores } son las más grandes.
relatorios-composicao-tabela = Composición por categoría

## 03 · Ritmo

relatorios-carregando-ritmo = Armando el calendario
relatorios-dia = Día
relatorios-semana = Semana
relatorios-ritmo-pergunta = En qué días y semanas gastamos más: un cuadrado por día, más oscuro cuanto mayor el gasto.
relatorios-despesas-12-meses = Gastos en 12 meses
relatorios-media-por-dia = Promedio por día
relatorios-dia-mais-caro = Día más caro
relatorios-dias-sem-gasto = Días sin gasto
relatorios-ritmo-titulo = Ritmo · { $periodo }
relatorios-ritmo-fonte = Fuente: movimientos de gasto efectivados · { $base } · franjas por cuantiles de los días con gasto
relatorios-ritmo-aria = Calendario de gastos de { $periodo }; el día más caro fue { $dia }.
relatorios-media-dia-semana-aria = Promedio de gasto por día de la semana
relatorios-media-dia-semana = Promedio por día de la semana
relatorios-dias-maior-gasto = Días de mayor gasto
relatorios-dia-semana-data = { $semana }, { $data }
relatorios-gasto-por-dia = Gasto por día

## 04 · Tendencia

relatorios-carregando-tendencia = Sumando los meses
relatorios-media-3m = Promedio 3m
relatorios-media-movel-3m = Media móvil (3 meses)
relatorios-tendencia-pergunta = ¿Los gastos están subiendo? ¿En qué? Barras por categoría, la media móvil de 3 meses y los ingresos como referencia.
relatorios-media-movel-ultimo = Media móvil (último mes)
relatorios-variacao-media-movel = Variación de la media móvil
relatorios-contra = frente a { $mes }
relatorios-media-meses = Promedio de { $n } meses
relatorios-receitas-periodo = Ingresos en el período
relatorios-tendencia-titulo = Tendencia · { $n } meses hasta { $mes }
relatorios-tendencia-fonte = Fuente: movimientos de gasto e ingreso efectivados · { $base } · media móvil del mes y de los dos anteriores
relatorios-tendencia-aria = Gastos por categoría en { $n } meses hasta { $mes }, con media móvil de 3 meses e ingresos.
relatorios-tendencia-tabela = Gastos por mes y categoría

## 05 · Presupuesto

relatorios-orcamento-vazio = Sin presupuesto ni gasto en { $mes }
relatorios-montar-orcamento = Armar el presupuesto
relatorios-previsto = Previsto
relatorios-realizado = Realizado
relatorios-comprometido = Comprometido
relatorios-restante = Restante
relatorios-situacao = Situación
relatorios-orcamento-pergunta = Dónde nos pasamos: previsto (marca), realizado (barra), comprometido (tramo claro) y el ritmo del mes (trazo fino).
relatorios-estouradas = Excedidas
relatorios-mes-decorrido = Mes transcurrido
relatorios-orcamento-titulo = Presupuesto · { $mes }
relatorios-orcamento-fonte = Fuente: presupuesto del mes y movimientos de gasto · { $base } · lo realizado incluye las subcategorías
relatorios-orcamento-aria = Previsto × realizado de { $mes }
relatorios-clique-nome = Haz clic en el nombre para ver los movimientos
relatorios-sem-previsto = sin previsto
relatorios-orcamento-tabela = Previsto × realizado

## 06 · Comparación

relatorios-carregando-comparacao = Comparando los períodos
relatorios-comparacao-pergunta = Qué cambió: la variación de cada categoría entre { $a } (A) y { $b } (B). Gastar más aparece con ▲.
relatorios-comparacao-a = A · { $periodo }
relatorios-comparacao-b = B · { $periodo }
relatorios-variacao-pct = Variación %
relatorios-comparar-com = Comparar con
relatorios-um-ano-antes = Un año antes
relatorios-comparacao-em-andamento = B todavía está en curso (hasta hoy): parte de los gastos del período aún no ocurrió. Para comparar meses cerrados, elige “Mes pasado”.
relatorios-comparacao-titulo = Comparación · { $a } × { $b }
relatorios-comparacao-fonte = Fuente: movimientos de gasto efectivados · { $base } · categorías de primer nivel
relatorios-forma-barras = Barras
relatorios-forma-slope = Pendiente
relatorios-comparacao-aria = Variación del gasto por categoría entre { $a } y { $b }.
relatorios-comparacao-tabela = Comparación por categoría

## 07 · Inflación (lugar reservado)


## 08 · Miembros

relatorios-carregando-membros = Separando por miembro
relatorios-membros-pergunta = Quién gasta en qué: el gasto de cada miembro por categoría (la parte de una división cuenta para el miembro de esa parte).
relatorios-membros-titulo = Miembros · { $periodo }
relatorios-membros-fonte = Fuente: movimientos de gasto efectivados · { $base } · { $n } categorías
relatorios-membros-fonte-outras = Fuente: movimientos de gasto efectivados · { $base } · { $n } categorías ({ $outras } en "Otras")
relatorios-membros-aria = Gasto por miembro y categoría en { $periodo }.
relatorios-membros-tabela = Gasto por miembro y categoría

## 09 · Recurrentes

relatorios-sem-recorrencias = Sin recurrencias
relatorios-sem-recorrencias-titulo = Ninguna suscripción ni cuenta fija registrada
relatorios-sem-recorrencias-descricao = Registra las recurrencias (alquiler, escuela, suscripciones) en la pantalla de movimientos: generan los previstos y aparecen aquí.
relatorios-carregando-recorrencias = Leyendo las recurrencias
relatorios-regra = Regla
relatorios-proximas = Próximas
relatorios-recorrentes-pergunta = Cuánto ya está comprometido cada mes con suscripciones y cuentas fijas, y cuándo vence cada una.
relatorios-comprometido-mes = Comprometido por mes
relatorios-receitas-recorrentes = Ingresos recurrentes
relatorios-das-receitas-recorrentes = De los ingresos recurrentes
relatorios-despesas-recorrentes = Gastos recurrentes
relatorios-recorrentes-titulo = Recurrentes · próximos { $n } días
relatorios-recorrentes-fonte = Fuente: recurrencias activas (regla y próximas fechas) · valor mensual equivalente según la frecuencia
relatorios-recorrentes-aria = Línea de tiempo de las recurrencias en los próximos { $n } días
relatorios-recorrencias = Recurrencias

## 10 · Tarjetas

relatorios-carregando-cartoes = Sumando los resúmenes
relatorios-sem-cartoes = Sin tarjetas
relatorios-sem-cartoes-titulo = Ninguna tarjeta de crédito registrada
relatorios-sem-cartoes-descricao = Con tarjetas y compras en cuotas registradas, esta figura muestra cuánto ya vence en cada mes.
relatorios-fatura = Resumen
relatorios-cartoes-pergunta = Cuánto ya está comprometido en los próximos meses: lo que falta pagar de cada resumen, con las cuotas futuras.
relatorios-proximos-meses = Próximos { $n } meses
relatorios-a-pagar-em = A pagar en { $mes }
relatorios-proxima-fatura = Próximo resumen
relatorios-cartoes-titulo = Tarjetas · resúmenes desde { $mes }
relatorios-cartoes-fonte = Fuente: resúmenes no pagados y cuotas previstas · mes de vencimiento del resumen · valores en la moneda base
relatorios-cartoes-aria = Resúmenes futuros de las tarjetas en los próximos { $n } meses.
relatorios-cartoes-tabela = Resúmenes futuros por tarjeta
relatorios-total-faturas = Total de los resúmenes

## 11 · Patrimonio

relatorios-carregando-patrimonio = Midiendo el patrimonio
relatorios-contas = Cuentas
relatorios-investimentos = Inversiones
relatorios-bens = Bienes
relatorios-dividas = Deudas
relatorios-liquido = Neto
relatorios-patrimonio-liquido = Patrimonio neto
relatorios-patrimonio-pergunta = ¿El patrimonio está creciendo? Cuentas, inversiones y bienes por encima de cero, deudas por debajo, y el neto en la línea.
relatorios-liquido-em = Neto en { $mes }
relatorios-patrimonio-liquido-em = Patrimonio neto al { $data }
relatorios-explicar-patrimonio = Explicar el patrimonio neto al { $data }
relatorios-variacao-desde = Variación desde { $mes }
relatorios-patrimonio-titulo = Patrimonio · { $n } meses hasta { $mes }
# $estimado y $semcambio: "sim" (sí) o "nao" (no).
relatorios-patrimonio-fonte = Fuente: saldos del último día de cada mes + valor de mercado de la cartera + valor de los bienes{ $estimado ->
        [sim] {" "}· ≈ incluye tipo de cambio o cotización estimados
       *[nao] {""}
    }{ $semcambio ->
        [sim] {" "}· sin tipo de cambio: { $moedas }
       *[nao] {""}
    } · haz clic en un mes para ver el origen
relatorios-patrimonio-aria = Patrimonio neto en { $n } meses hasta { $mes }: cuentas, inversiones y deudas.
relatorios-patrimonio-tabela = Patrimonio por mes

## Analítico de una línea del flujo

relatorios-analitico = Analítico
relatorios-analitico-de = Analítico · { $titulo }
relatorios-carregando-lancamentos = Sumando los movimientos
relatorios-diferenca-delta = Diferencia (Δ)
relatorios-agrupar-categoria-dica = Por categoría y subcategoría
relatorios-agrupar-descricao-dica = Por la descripción del movimiento
relatorios-dividido-dica = Movimiento dividido: la categoría de cada parte se cambia en el editor
relatorios-dividido = · dividido
relatorios-categoria-de = Categoría de { $nome }
relatorios-categoria-alterada = Categoría cambiada
relatorios-relatorio-recalculado = { $nome }: el informe se recalculó.
relatorios-categoria-nao-mudou = No se pudo cambiar la categoría
relatorios-data = Fecha
relatorios-linha = Línea
relatorios-editar-item = Editar { $nome }
relatorios-editar-dica = Cambia la categoría en la misma línea: el informe se recalcula al instante. "Editar" (o doble clic) abre el movimiento completo.
relatorios-parte-dividido = parte de un movimiento dividido
relatorios-fatura-vence = el resumen vence el { $data }
relatorios-soma-confere = { $n ->
        [one] La suma del ítem
       *[other] La suma de los { $n } ítems
    } es { $soma }, igual al valor de { $periodo } en la tabla.
relatorios-soma-difere = La suma de los ítems ({ $soma }) difiere del valor de la tabla ({ $valor }). Vuelve a cargar el informe.
relatorios-cambio-estimado = ≈ Incluye un valor convertido con tipo de cambio estimado.
relatorios-por-categoria-em = Por categoría en { $periodo }
relatorios-o-que-compoe = Qué compone { $nome } en { $periodo }
relatorios-nada-em = Nada en { $periodo }.
relatorios-abre-analitico = Enter o doble clic en una línea abre su analítico.
relatorios-fora-total = Fuera del total
relatorios-fora-total-n = Fuera del total ({ $n })
relatorios-fora-total-de = Fuera del total de { $nome } en { $periodo }
relatorios-por-que-fora = Por qué queda fuera
relatorios-diferenca-resumo = De { $anterior } ({ $antes }) a { $atual } ({ $depois }): { $diferenca }. Cada línea de abajo muestra cuánto cambió; la suma de las variaciones es la diferencia de la tabla.
relatorios-agrupar-por = Agrupar por
relatorios-agrupar-diferenca = Agrupar la diferencia por
relatorios-diferenca-por-categoria = Diferencia de { $nome } por categoría
relatorios-diferenca-por-descricao = Diferencia de { $nome } por descripción
relatorios-nada-mudou = Nada cambió entre los dos períodos.
relatorios-lancamentos-de = Movimientos de { $nome }
relatorios-lancamentos-dois-periodos = Movimientos de { $nome } en los dos períodos
relatorios-linhas-dois-periodos = Líneas de { $nome } en los dos períodos
relatorios-nenhum-lancamento = Ningún movimiento.

## Comandos de la barra ⌘K (las palabras de búsqueda van separadas por comas)

relatorios-cmd-grupo-busca = informe,gráfico
relatorios-cmd-fluxo = Informe: flujo del mes
relatorios-cmd-fluxo-busca = sankey,a dónde fue,a dónde va,ingresos,gastos,sobrante
relatorios-cmd-fluxo-trimestre = Informe: flujo del trimestre × trimestre anterior
relatorios-cmd-fluxo-trimestre-busca = sankey,ingreso,comparar,inversiones,deudas
relatorios-cmd-fluxo-anterior = Informe: flujo del mes pasado
relatorios-cmd-fluxo-anterior-busca = sankey
relatorios-cmd-composicao = Informe: composición de los gastos
relatorios-cmd-composicao-busca = treemap,sunburst,categorías,pesa
relatorios-cmd-ritmo = Informe: calendario de gastos
relatorios-cmd-ritmo-busca = heatmap,días,semana,ritmo
relatorios-cmd-tendencia = Informe: tendencia de los gastos (12 meses)
relatorios-cmd-tendencia-busca = evolución,media móvil,subiendo
relatorios-cmd-orcamento = Informe: previsto × realizado
relatorios-cmd-orcamento-busca = bullet,nos pasamos,presupuesto
relatorios-cmd-comparacao = Informe: comparar con el mes anterior
relatorios-cmd-comparacao-busca = a × b,variación,cambió
relatorios-cmd-titulares = Informe: gastos por miembro de la familia
relatorios-cmd-titulares-busca = miembros,familia,quién gasta
relatorios-cmd-recorrentes = Informe: suscripciones y cuentas fijas
relatorios-cmd-recorrentes-busca = recurrencias,comprometido,mensual
relatorios-cmd-cartoes = Informe: resúmenes futuros de las tarjetas
relatorios-cmd-cartoes-busca = cuotas,compras en cuotas,tarjeta
relatorios-cmd-patrimonio = Informe: patrimonio en 24 meses
relatorios-cmd-patrimonio-busca = activos,pasivos,deudas,inversiones
relatorios-cmd-inflacao = Informe: gasto corregido por la inflación
relatorios-cmd-inflacao-busca = ipca,real,nominal
relatorios-cmd-ir-mes = IR: cálculo del mes y DARF
relatorios-cmd-ir-mes-busca = impuesto sobre la renta,darf,ganancia de capital,day trade,exención,20 mil
relatorios-cmd-ir-ano = IR: informe para la declaración anual
relatorios-cmd-ir-ano-busca = impuesto sobre la renta,irpf,bienes y derechos,rendimientos exentos,declaración
relatorios-cmd-impostos-ano = Impuestos pagados en el año (estimación)
relatorios-cmd-impostos-ano-busca = carga tributaria,tributos,consumo,icms,inss,escenarios
relatorios-cmd-impostos-grupos = Impuestos: grupos de las categorías
relatorios-cmd-impostos-grupos-busca = alícuota propia,grupo de impuesto,categorías,mapeo
relatorios-cmd-divida-nova = Registrar préstamo o financiamiento
relatorios-cmd-divida-nova-busca = deuda,contrato,sac,price,préstamo de nómina,financiamiento,cdc,hipoteca
relatorios-cmd-bem-novo = Registrar inmueble, vehículo o bien
relatorios-cmd-bem-novo-busca = bien,patrimonio,inmueble,departamento,casa,auto,terreno,fipe
relatorios-cmd-bens = Ver bienes (patrimonio físico)
relatorios-cmd-bens-busca = bienes,inmuebles,vehículos,mejoras,avalúo,valorización
relatorios-cmd-orcamento-mes = Presupuesto del mes
relatorios-cmd-orcamento-mes-busca = previsto,realizado,presupuesto,budget
relatorios-cmd-metas = Metas
relatorios-cmd-metas-busca = objetivos,viaje,liquidación,progreso
relatorios-cmd-reserva = Calculadora de fondo de emergencia
relatorios-cmd-reserva-busca = reserva,emergencia,colchón,fondo
relatorios-cmd-sem-categoria = Movimientos sin categoría de este mes
relatorios-cmd-sem-categoria-busca = categorizar,revisar,sin categoría

## 12 · Grupos (a dónde va el ingreso)

relatorios-vista-grupos = Grupos
relatorios-vista-grupos-pergunta = ¿Cuánto del ingreso va a esenciales, deudas, futuro y el resto?
relatorios-cmd-grupos = Informe: a dónde va el ingreso (grupos)
relatorios-cmd-grupos-busca = grupos,esenciales,deudas,futuro,no esencial,50/30/20,ingreso
relatorios-grupos-pergunta = A dónde va el ingreso: cuánto de cada mes fue a los esenciales, las deudas, el futuro (aportes y metas) y lo no esencial, frente a la meta de cada grupo.
relatorios-grupos-carregando = Calculando los grupos…
relatorios-grupos-renda = Ingreso de { $periodo }
relatorios-grupos-stat-dica = { $valor } · meta { $meta }
relatorios-grupos-stat-dica-sem-meta = { $valor } · sin meta
relatorios-grupos-titulo = A dónde va el ingreso: { $n } meses hasta { $mes }
relatorios-grupos-fonte = Fuente: movimientos efectuados, aportes y transferencias a metas · { $base }
relatorios-grupos-medida = Medida
relatorios-grupos-medida-pct = % del ingreso
relatorios-grupos-medida-valor = Valor
relatorios-grupos-forma-barras = Barras por mes
relatorios-grupos-forma-linhas = Evolución
relatorios-grupos-aria-barras = Grupos de gasto por mes, { $n } meses hasta { $mes }
relatorios-grupos-aria-linhas = Parte del ingreso de cada grupo, { $n } meses hasta { $mes }
relatorios-grupos-tabela = Grupos de gasto por mes
relatorios-grupos-maiores-titulo = Mayores categorías de cada grupo en { $periodo }
relatorios-grupos-maiores-tabela = Mayores categorías por grupo
relatorios-grupos-grupo = Grupo
relatorios-grupos-pct-grupo = % del grupo
relatorios-grupos-pct-renda = % del ingreso
relatorios-grupos-meta = Meta
relatorios-grupos-sem-renda = Sin ingresos en el período: la parte del ingreso queda en blanco.
relatorios-grupos-nota-futuro = Futuro suma los aportes y las transferencias a cuentas de metas; los retiros no restan.
relatorios-grupos-metas-link = Editar metas
relatorios-grupos-categorias-link = Ajustar grupos
relatorios-grupos-composicao-link = Ver cuánto del ingreso va a cada grupo en 12 · Grupos
relatorios-fluxo-fatura-competencia = En devengo, las compras de los resúmenes pagados en este período ({ $valor }) cuentan en el mes en que se hicieron, mientras que rescates y aportes cuentan en el mes del dinero. En caja, el resumen aparece aquí, abierto en las categorías de las compras.
