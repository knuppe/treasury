# Pantalla 05 · Presupuesto: pestañas, previsto frente a real del mes (resumen, tabla editable,
# copiar del mes anterior y sugerir por el promedio) (docs/i18n-design.md). Metas, proyectos,
# fondo de emergencia y jubilación tienen sus propios catálogos; suscripciones y escenarios
# están en planejamento.ftl. Revisión: borrador por IA, sin revisión de un hablante nativo.

## Pestañas

orcamento-abas = Secciones del presupuesto
orcamento-aba-previsto = Previsto frente a real
orcamento-aba-metas = Metas
orcamento-aba-projetos = Proyectos
orcamento-aba-reserva = Fondo de emergencia
orcamento-aba-aposentadoria = Jubilación
orcamento-aba-assinaturas = Suscripciones
orcamento-aba-parcelas = Cuotas
orcamento-aba-cenarios = Escenarios
orcamento-carregando = Sumando el presupuesto

## Etiquetas que se repiten

orcamento-previsto = Previsto
orcamento-realizado = Real
orcamento-recebido = Recibido
orcamento-comprometido = Comprometido
orcamento-restante = Restante
orcamento-definir = Definir
orcamento-erro-gravar = No se pudo guardar.

## Resumen del mes

orcamento-resumo-de = Resumen de { $mes }
orcamento-base = Base
orcamento-base-caixa = caja
orcamento-base-competencia = devengo
orcamento-mes-decorrido = Mes transcurrido
orcamento-despesas-de = Gastos de { $mes }
orcamento-comprometido-dica = previstos por efectuar
orcamento-ritmo = Ritmo del mes
orcamento-de = Presupuesto de { $mes }
orcamento-ritmo-texto = { $pct } de lo previsto con { $decorrido } del mes · proyección { $projecao }

## Copiar del mes anterior

orcamento-copiar-titulo = Copiar presupuesto
orcamento-copiar-descricao = Copia las líneas de { $de } a { $para }. Las categorías archivadas quedan fuera.
orcamento-copiar-anterior = Copiar del mes anterior
orcamento-copiado = Presupuesto copiado
orcamento-copiar = Copiar
orcamento-substituir = Reemplazar las líneas que ya existen en { $mes }

## Sugerir por el promedio

orcamento-aplicar = Aplicar
orcamento-atual = Actual
orcamento-media = Promedio
orcamento-diferenca = Diferencia
orcamento-sugestao = Sugerencia por el promedio
orcamento-aplicar-a = Aplicar a { $categoria }
orcamento-sugerir = Sugerir por el promedio
orcamento-sugerir-descricao = Promedio del gasto de cada categoría en los meses completos anteriores, redondeado hacia arriba al múltiplo de { $valor } siguiente.
orcamento-atualizado = Presupuesto actualizado
orcamento-atualizado-descricao =
    { $n ->
        [one] { $n } categoría por el promedio.
       *[other] { $n } categorías por el promedio.
    }
orcamento-aplicar-n =
    { $n ->
        [0] Aplicar
       *[other] Aplicar { $n }
    }
orcamento-meses-media = Meses en el promedio
orcamento-n-meses = { $n } meses
orcamento-calculando = Calculando el promedio…
orcamento-igual-media = El presupuesto ya coincide con el promedio.

## Tabla y avisos

orcamento-gravar-erro = No se pudo guardar el monto previsto
orcamento-adicionar-aria = Agregar categoría al presupuesto
orcamento-adicionar = Agregar categoría…
orcamento-sem-orcamento-aria = Categorías con gasto y sin presupuesto
orcamento-n-sem-orcamento =
    { $n ->
        [one] { $n } categoría con gasto y sin presupuesto
       *[other] { $n } categorías con gasto y sin presupuesto
    }
orcamento-despesas-categoria = Gastos por categoría
orcamento-vazio = Sin presupuesto ni gasto en { $mes }. Copia del mes anterior, usa el promedio o agrega una categoría.
orcamento-tabela-despesas = Presupuesto de gastos por categoría
orcamento-receitas-previstas = Ingresos previstos
orcamento-tabela-receitas = Presupuesto de ingresos por categoría
orcamento-cambio = Tipo de cambio
orcamento-sem-cambio = Sin tipo de cambio para { $moedas }: esos movimientos quedaron fuera de lo real.
orcamento-previsto-de = Previsto de { $categoria }
orcamento-soma-filhos = suma de las subcategorías
orcamento-editar-previsto-de = Editar previsto de { $categoria }
orcamento-gravando = guardando…
orcamento-soma-subcategorias-dica = Suma de las subcategorías (la categoría padre no tiene monto propio)
orcamento-soma-subcategorias = suma de las subcategorías
orcamento-previsto-realizado = Previsto frente a real
orcamento-recebido-previsto = Recibido frente a previsto
orcamento-percentual = Porcentaje
orcamento-editar-o-previsto = Editar el monto previsto
orcamento-definir-o-previsto = Definir el monto previsto
orcamento-ver-lancamentos-mes = Ver movimientos de { $mes }
orcamento-recolher-subcategorias = Contraer subcategorías
orcamento-expandir-subcategorias = Expandir subcategorías
orcamento-acoes-de = Acciones de { $categoria }
orcamento-recolher = Contraer { $categoria }
orcamento-expandir = Expandir { $categoria }
orcamento-arquivada = archivada
orcamento-clique-realizado = Haz clic en Real para ver los movimientos
orcamento-ver-lancamentos-categoria = Ver movimientos de { $categoria } en { $mes }

## A dónde va el ingreso (grupos de gasto)

orcamento-grupos-titulo = A dónde va el ingreso
orcamento-grupos-aria = Grupos de gasto de { $mes }
orcamento-grupos-carregando = Calculando los grupos…
orcamento-grupos-renda = Ingreso del mes
orcamento-grupos-total = En los cuatro grupos
orcamento-grupos-livre = Sin destino
orcamento-grupos-livre-dica = Ingreso que sobró después de los cuatro grupos.
orcamento-grupos-pct-renda = { $pct } del ingreso
orcamento-grupos-meta = meta { $pct }
orcamento-grupos-sem-meta = sin meta
orcamento-grupos-acima = por encima de la meta
orcamento-grupos-dentro = dentro de la meta
orcamento-grupos-abaixo = por debajo de la meta
orcamento-grupos-atingida = meta alcanzada
orcamento-grupos-barra = { $grupo }: { $pct } del ingreso
orcamento-grupos-sem-renda = Sin ingresos este mes: la parte del ingreso queda en blanco.
orcamento-grupos-futuro-nota = Futuro suma los aportes y las transferencias a cuentas de metas; los retiros no restan.
orcamento-grupos-ver-maiores = Ver las mayores de { $grupo }
orcamento-grupos-esconder-maiores = Ocultar las mayores de { $grupo }
orcamento-grupos-maiores = Mayores de { $grupo }
orcamento-grupos-nada = Nada en este grupo en el mes.
orcamento-grupos-relatorios = Ver la evolución en Informes
orcamento-grupos-categorias = Ajustar los grupos de las categorías
orcamento-grupos-editar-metas = Editar metas
orcamento-grupos-metas-titulo = Metas por grupo
orcamento-grupos-metas-descricao = Cuánto del ingreso quieres que vaya a cada grupo. Déjalo en blanco para no tener meta. Una referencia común es 50% para los esenciales, 30% para lo no esencial y 20% para el futuro.
orcamento-grupos-meta-de = Meta de { $grupo } (% del ingreso)
orcamento-grupos-soma = Suma de las metas: { $pct }
orcamento-grupos-soma-acima = La suma pasa del 100% del ingreso.
orcamento-grupos-padrao = Volver al 50/30/20
orcamento-grupos-meta-invalida = Usa un número de 0 a 100.
orcamento-grupos-metas-salvas = Metas por grupo guardadas
