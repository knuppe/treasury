# Presupuesto › Proyectos: obras, remodelaciones, fiestas y viajes con presupuesto, gastos e
# ingresos, la línea de tiempo, los destacados y las entradas; registro, ítems cargados a mano y
# movimientos puestos en un proyecto (docs/i18n-design.md).
# Revisión: borrador por IA, sin revisión nativa.

## Estado y etiquetas repetidas

projetos-status-planejado = Planificado
projetos-status-andamento = En curso
projetos-status-concluido = Terminado
projetos-titulo = Proyectos
projetos-projeto = Proyecto
projetos-o-projeto = proyecto
projetos-projeto-nome = Proyecto { $nome }
projetos-novo = Nuevo proyecto
projetos-novo-reticencias = Nuevo proyecto…
projetos-sem-projeto = Sin proyecto
projetos-nome = Nombre
projetos-bem = Bien
projetos-nenhum = Ninguno
projetos-orcamento = Presupuesto
projetos-situacao = Estado
projetos-observacoes = Observaciones
projetos-tipo = Tipo
projetos-descricao = Descripción
projetos-valor = Monto
projetos-data = Fecha
projetos-despesas = Gastos
projetos-receitas = Ingresos
projetos-entradas = Entradas
projetos-sem-descricao = sin descripción
projetos-hoje = hoy
projetos-editar = Editar { $nome }
projetos-excluir = Eliminar
projetos-acoes-de = Acciones de { $nome }
projetos-tirar = Quitar del proyecto
projetos-por = Poner en el proyecto

## Lista y detalle

projetos-ver-lancamentos = Ver movimientos
projetos-lancamentos = Movimientos
projetos-novo-item = Nuevo ítem
projetos-abrir = Abrir proyecto
projetos-novo-item-menu = Nuevo ítem…
projetos-por-lancamentos-menu = Poner movimientos…
projetos-editar-menu = Editar…
projetos-excluir-menu = Eliminar…
projetos-vazio-kicker = Sin proyectos
projetos-vazio-titulo = Agrupa los gastos e ingresos de una obra, remodelación, fiesta o viaje
projetos-vazio-descricao = Un proyecto suma los gastos e ingresos vinculados a él (movimientos de las cuentas e ítems cargados a mano) y los compara con el presupuesto. Vinculado a un bien, alimenta sus mejoras: el costo del inmueble o del auto pasa a seguir lo que gastó la obra.
projetos-situacao-lista = Estado de los proyectos
projetos-todos-n = Todos ({ $n })
projetos-abertos-n = Abiertos ({ $n })
projetos-concluidos-n = Terminados ({ $n })
projetos-nenhum-aberto = Ningún proyecto abierto: mira los terminados.
projetos-nenhum-concluido = Ningún proyecto terminado.
projetos-sem-bem = sin bien
projetos-periodo = Período
projetos-gasto = Gastado
projetos-resta = Queda
projetos-excluir-titulo = Eliminar { $nome }
projetos-excluir-descricao = Los movimientos y las mejoras siguen, solo dejan de estar en el proyecto. Puedes deshacerlo desde el historial.
projetos-excluir-projeto = Eliminar proyecto
projetos-excluido = Proyecto eliminado

## Línea de tiempo, cifras y destacados

projetos-por-dia = Por día
projetos-por-mes = Por mes
projetos-linha-tempo = Línea de tiempo
projetos-visao-linha = Vista de la línea de tiempo
projetos-acumulado = Acumulado
projetos-grafico-acumulado = Gastos e ingresos acumulados de { $nome } de { $de } a { $ate }, con el resultado.
projetos-grafico-acumulado-orcamento = Gastos e ingresos acumulados de { $nome } de { $de } a { $ate }, con el resultado y el presupuesto.
projetos-grafico-periodo-dia = Ingresos y gastos de { $nome } por día, con el resultado acumulado.
projetos-grafico-periodo-mes = Ingresos y gastos de { $nome } por mes, con el resultado acumulado.
projetos-montando = Armando la línea de tiempo…
projetos-linha-vazia = Todavía no hay entradas. { $lance } o usa “Movimientos” para marcar los de las cuentas.
projetos-linha-vazia-lance = Carga un gasto o ingreso a mano
projetos-maior-despesa = Mayor gasto
projetos-maior-receita = Mayor ingreso
projetos-dia-maior-gasto = Día de mayor gasto
projetos-mes-maior-gasto = Mes de mayor gasto
projetos-orcamento-acaba = El presupuesto se acaba
projetos-orcamento-acaba-em = en { $mes }, al ritmo actual
projetos-acima-orcamento = por encima del presupuesto
projetos-resultado = Resultado
projetos-resultado-dica = ingresos − gastos
projetos-passou = se pasó por { $valor }
projetos-resta-valor = quedan { $valor }
projetos-sem-orcamento = sin presupuesto
projetos-pct-orcamento = { $pct }% del presupuesto
projetos-gasto-mes = Gasto por mes
projetos-media-meses =
    { $n ->
        [one] promedio de { $n } mes
       *[other] promedio de { $n } meses
    }
projetos-entradas-dica =
    { $lancamentos ->
        [one] { $lancamentos } movimiento
       *[other] { $lancamentos } movimientos
    } · { $itens ->
        [one] { $itens } ítem
       *[other] { $itens } ítems
    }
projetos-destaques = Destacados
projetos-alimenta =
    { $n ->
        [one] Alimenta { $n } mejora de { $bem }: el costo del bien sigue los gastos del proyecto (los ingresos no lo reducen).
       *[other] Alimenta { $n } mejoras de { $bem }: el costo del bien sigue los gastos del proyecto (los ingresos no lo reducen).
    }
projetos-um-bem = un bien
projetos-por-categoria = Por categoría
projetos-despesas-categoria = Gastos de { $nome } por categoría
projetos-nenhuma-despesa = Todavía no hay gastos.
projetos-receitas-categoria = Ingresos de { $nome } por categoría
projetos-nenhuma-receita = Ningún ingreso.

## Entradas del proyecto

projetos-entrada-sem-descricao = Entrada sin descripción
projetos-entrada-sem-descricao-min = entrada sin descripción
projetos-editar-item = Editar ítem…
projetos-editar-lancamento = Editar movimiento…
projetos-ver-em-lancamentos = Ver en Movimientos
projetos-excluir-item = Eliminar ítem…
projetos-tirar-menu = Quitar del proyecto…
projetos-n-entradas-selecionadas = { $n } entradas seleccionadas
projetos-tirar-n = Quitar { $n } entradas del proyecto…
projetos-entradas-de = Entradas de { $nome }
projetos-acoes-n-entradas = Acciones de { $n } entradas
projetos-entradas-vazio = Ninguna entrada. Carga un { $item } o marca { $lancamentos }.
projetos-entradas-vazio-item = ítem
projetos-entradas-vazio-lancamentos = movimientos
projetos-origem = Origen
projetos-previsto = previsto
projetos-item-do-projeto = Ítem del proyecto
projetos-n-entradas =
    { $n ->
        [one] { $n } entrada
       *[other] { $n } entradas
    }
projetos-excluir-item-titulo = Eliminar ítem del proyecto
projetos-tirar-lancamento-titulo = Quitar movimiento del proyecto
projetos-tirar-n-titulo = Quitar { $n } entradas del proyecto
projetos-tirar-itens-descricao = El ítem solo existe en el proyecto y se eliminará. Puedes deshacerlo desde el historial.
projetos-tirar-misto-descricao = Los movimientos siguen en las cuentas, solo sin el proyecto; los ítems cargados a mano se eliminan. Puedes deshacerlo desde el historial.
projetos-tirar-lancamento-descricao = El movimiento sigue en la cuenta, solo sin el proyecto. Puedes deshacerlo desde el historial.
projetos-entradas-tiradas = Entradas quitadas del proyecto
projetos-entradas-tiradas-descricao =
    { $n ->
        [one] { $n } entrada de { $nome }.
       *[other] { $n } entradas de { $nome }.
    }

## Registro del proyecto

projetos-salvo = Proyecto guardado
projetos-criado = Proyecto creado
projetos-form-descricao = Agrupa los gastos e ingresos de una obra, remodelación, fiesta o viaje. Vinculado a un bien, el proyecto alimenta sus mejoras.
projetos-criar = Crear proyecto
projetos-nome-exemplo = Remodelación del baño, chapa y pintura del auto…
projetos-bem-dica = Inmueble, vehículo u otro bien de la obra (opcional).
projetos-orcamento-dica = Cuánto piensas gastar (opcional).
projetos-inicio = Inicio
projetos-fim = Fin

## Poner movimientos en un proyecto

projetos-lanc-no-projeto = Movimientos en el proyecto
projetos-lanc-fora = Movimientos fuera del proyecto
projetos-n-lancamentos =
    { $n ->
        [one] { $n } movimiento.
       *[other] { $n } movimientos.
    }
projetos-titulo-lancamentos =
    { $n ->
        [one] Proyecto del movimiento
       *[other] Proyecto de { $n } movimientos
    }
projetos-link-descricao = El gasto del proyecto es la suma de sus movimientos; las mejoras vinculadas al proyecto pasan a valer ese gasto.
projetos-nome-novo = Nombre del nuevo proyecto
projetos-nome-novo-exemplo = Remodelación de la cocina
projetos-add-titulo = Movimientos para { $nome }
projetos-add-descricao = Marca los gastos (compras, PIX, resúmenes de tarjeta) y los ingresos del proyecto. Los movimientos de otro proyecto pasan a este.
projetos-add-marcados =
    { $n ->
        [one] { $n } marcado · { $valor }
       *[other] { $n } marcados · { $valor }
    }
projetos-add-toast = Movimientos en { $nome }
projetos-add-toast-descricao =
    { $n ->
        [one] { $n } movimiento en el proyecto.
       *[other] { $n } movimientos en el proyecto.
    }
projetos-buscar = Buscar
projetos-buscar-exemplo = Descripción, beneficiario u observación
projetos-de = Desde
projetos-ate = Hasta
projetos-entradas-saidas = Entradas y salidas
projetos-so-saidas = Solo salidas
projetos-so-entradas = Solo entradas
projetos-encontrados = Movimientos encontrados
projetos-buscando = Buscando…
projetos-nenhum-filtro = Ningún movimiento con este filtro.
projetos-em-projeto = en { $nome }

## Ítem cargado a mano

projetos-item-receita-em = Ingreso en { $nome }.
projetos-item-despesa-em = Gasto en { $nome }.
projetos-item-salvo = Ítem guardado
projetos-lancamento-criado = Movimiento creado
projetos-item-lancado = Ítem cargado
projetos-item-novo-em = Nuevo ítem en { $nome }
projetos-item-editar-descricao = Ítem cargado a mano en el proyecto, fuera de las cuentas.
projetos-item-descricao = Gasto o ingreso del proyecto. Solo en el proyecto no toca ninguna cuenta; en una cuenta, se vuelve un movimiento de ella ya en el proyecto.
projetos-lancar-conta = Cargar en la cuenta
projetos-lancar-projeto = Cargar en el proyecto
projetos-item-exemplo-receita = Venta de sobrantes, ayuda de la familia…
projetos-item-exemplo-despesa = Albañil, materiales…
projetos-onde-lancar = Dónde cargar
projetos-onde-conta = Entra en el saldo y en los informes de la cuenta.
projetos-onde-projeto = Solo en el proyecto: fuera de las cuentas y de los informes.
projetos-so-no-projeto = Solo en el proyecto
