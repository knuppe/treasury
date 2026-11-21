# Pantalla 01 · Panel: encabezado, tarjetas de instrumento (patrimonio, flujo, presupuesto,
# cartera, categorías, vencimientos, sin categoría) y "Lo que notó Treasury"
# (docs/i18n-design.md). Revisión: borrador por IA, sin revisión de un hablante nativo.

## Encabezado

painel-atualizado = Actualizado { $data }
painel-descricao-membro = Las finanzas de { $nome }: los movimientos, cuentas, bienes y deudas que le pertenecen (las cuentas conjuntas por la parte de cada uno).
painel-descricao-simples = Cuánto entró, cuánto salió y qué sobró en el mes.
painel-voltar-simples = Volver al panel simple
painel-carregando = Midiendo el panel
painel-ver-completo = Ver el panel completo (patrimonio y cartera)

## Etiquetas que se repiten en las tarjetas

painel-base = Base
painel-base-caixa = caja
painel-base-competencia = devengo
painel-fonte = Fuente
painel-data = Fecha
painel-periodo = Período
painel-cambio = Tipo de cambio
painel-cambio-estimado = ≈ estimado

## 01 · Patrimonio neto

painel-patrimonio = Patrimonio neto
painel-patrimonio-fonte = Cuentas + cartera + bienes
painel-contas = Cuentas
painel-patrimonio-origem = Saldos de las cuentas, valor de mercado de la cartera y valor de los bienes
painel-patrimonio-estimado = Incluye valores estimados: tipo de cambio del día anterior (PTAX aún no publicada), cotizaciones o bienes valuados por estimación.
painel-patrimonio-em = Patrimonio neto al { $data }
painel-desde = desde { $mes }
painel-patrimonio-historico = Patrimonio neto en los últimos { $n } meses
painel-investimentos = Inversiones
painel-bens = Bienes
painel-dividas = Deudas
# El texto entre ** va en negrita.
painel-sem-cambio = **Sin tipo de cambio para { $moedas }**: { $contas } quedó fuera del total. Registra la tasa en Ajustes › Fuentes de datos.

## 02 · Flujo del mes

painel-fluxo = Flujo del mes
painel-receitas = Ingresos
painel-receitas-de = Ingresos de { $mes }
painel-despesas = Gastos
painel-despesas-de = Gastos de { $mes }
painel-saldo-mes = Saldo del mes
painel-taxa-poupanca = Tasa de ahorro
painel-taxa-poupanca-de = Tasa de ahorro de { $mes }
painel-media-12m = Promedio de 12 meses
painel-taxa-poupanca-historico = Tasa de ahorro mes a mes, { $n } meses

## 03 · Presupuesto del mes

painel-orcamento = Presupuesto del mes
painel-orcamento-carregando = Cargando el presupuesto…
painel-orcamento-abrir = Abrir presupuesto
painel-orcamento-sem = Sin presupuesto para { $mes }.
painel-orcamento-gasto = Gasto del mes: { $valor }. Define los montos o cópialos del mes anterior.
painel-orcamento-de-valor = de { $valor }
painel-orcamento-de = Presupuesto de { $mes }
painel-orcamento-ritmo = { $pct } de lo previsto · { $decorrido } del mes
painel-orcamento-comprometido = Comprometido (previstos): { $valor }

## 04 · Cartera

painel-real = Real
painel-nominal = nominal
painel-carteira-carregando = Cargando la cartera…
painel-carteira-vazia = No hay posiciones registradas.
painel-carteira-cadastrar = Registrar operaciones
painel-valor-mercado = Valor de mercado
painel-carteira-origem = Cotizaciones de la cartera
painel-sem-cotacao = Sin cotización: { $ativos }
painel-resultado-total = resultado total
painel-xirr-nominal = XIRR nominal
painel-xirr-real = XIRR real
painel-xirr-real-dica = Llega con los índices de inflación

## 05 · Categorías principales

painel-categorias = Categorías principales
painel-despesas-mes = Gastos del mes
painel-categorias-vazio = Ningún gasto en el mes.
painel-categorias-lista = Categorías con más gasto en el mes
painel-categoria-ver = { $categoria }: ver movimientos

## 06 · Próximos vencimientos

painel-vencimentos = Próximos vencimientos
painel-janela = Ventana
painel-30-dias = 30 días
painel-saidas-previstas = Salidas previstas
painel-itens = Ítems
painel-vencimentos-vazio = Nada vence en los próximos 30 días.
painel-hoje = hoy
painel-amanha = mañana
painel-em-dias = en { $n } d
painel-fatura = Resumen
painel-previsto = Previsto
painel-mais-itens =
    { $n ->
        [one] + { $n } ítem en los 30 días
       *[other] + { $n } ítems en los 30 días
    }

## 07 · Sin categoría

painel-para-revisar = Movimientos por revisar
painel-tudo-categorizado = ✓ todo categorizado
painel-fora-categorias = quedan fuera de las categorías en los informes
painel-categorizar = Categorizar ahora

## Lo que notó Treasury (calculado en la app, sin IA)

painel-percepcoes = Lo que notó Treasury
painel-percepcoes-sem-ia = calculado en tu computadora, sin IA
painel-inflacao-grupos = Inflación personal por grupo
painel-inflacao-grupo = Grupo del IPCA
painel-inflacao-peso = Peso en tus gastos
painel-inflacao-subiu = Subió
painel-esconder-grupos = Ocultar grupos
painel-ver-grupos = Ver por grupo
painel-abrir = Abrir
painel-dispensar = Descartar
painel-mostrar-menos = Mostrar menos
painel-ver-todas = Ver las { $n }
