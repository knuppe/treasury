# Controles compartidos (features/transactions/ui): casillas, interruptores, selección múltiple,
# selector de categoría, estado de consulta, mensajes de error y las etiquetas de los tipos del
# núcleo que se usan en varias pantallas (docs/i18n-design.md).
# Revisión: borrador por IA, sin revisión de un hablante nativo.

## Errores, archivos y estado de consulta

controles-erro-generico = Algo salió mal.
controles-erro-ler-arquivo = no se pudo leer el archivo
controles-tamanho-kb = { NUMBER($n, maximumFractionDigits: 1) } KB
controles-tamanho-mb = { NUMBER($n, maximumFractionDigits: 1) } MB
controles-carregando = Cargando
controles-tentar-de-novo = Intentar de nuevo
controles-ligado = Activado
controles-desligado = Desactivado

## Selección múltiple y búsqueda

controles-todos = Todos
controles-n-selecionados = { $n } seleccionados
controles-buscar-em = Buscar en { $rotulo }
controles-buscar = Buscar…
controles-nada-encontrado = No se encontró nada.
controles-limpar-selecao = Borrar selección

## Categoría y descripción

controles-categoria = Categoría
controles-sem-categoria = Sin categoría
controles-buscar-rotulo = Buscar { $rotulo }
controles-buscar-categoria = Buscar categoría…
controles-nenhuma-categoria = Ninguna categoría.
controles-sugestoes = Sugerencias de movimientos

## Tipos de cuenta

controles-conta-corrente = Cuenta corriente
controles-conta-poupanca = Cuenta de ahorros
controles-conta-dinheiro = Efectivo
controles-conta-cartao = Tarjeta de crédito
controles-conta-corretora = Corredora
controles-conta-exterior = Cuenta en el exterior
controles-conta-beneficio = Vales de comida y alimentación
controles-conta-fgts = FGTS (Brasil)
controles-conta-aposentadoria = Jubilación (401(k), IRA, plan de pensiones…)
controles-conta-outra = Otra

## Grupos de la pantalla de cuentas

controles-grupo-correntes = Cuentas corrientes
controles-grupo-poupanca = Ahorros
controles-grupo-dinheiro = Efectivo
controles-grupo-cartoes = Tarjetas
controles-grupo-corretoras = Corredoras
controles-grupo-exterior = Exterior
controles-grupo-beneficio = Vales de comida y alimentación
controles-grupo-fgts = FGTS (Brasil)
controles-grupo-aposentadoria = Cuentas de jubilación
controles-grupo-outras = Otras

## Tipos de institución

controles-instituicao-banco = Banco
controles-instituicao-corretora = Corredora
controles-instituicao-emissor = Emisor de tarjetas
controles-instituicao-outra = Otra

## Tipos y estados de movimiento

controles-tipo-receita = Ingreso
controles-tipo-despesa = Gasto
controles-tipo-transferencia = Transferencia
controles-tipo-pagamento-fatura = Pago de tarjeta
controles-tipo-aporte = Aporte
controles-tipo-ajuste = Ajuste

controles-status-previsto = Previsto
controles-status-efetivado = Efectuado
controles-status-conciliado = Conciliado

## Estado del resumen de la tarjeta y de las filas importadas

controles-fatura-aberta = Abierto
controles-fatura-fechada = Cerrado
controles-fatura-paga = Pagado
controles-fatura-parcial = Pagado en parte

controles-linha-nova = Nueva
controles-linha-duplicada = Duplicada
controles-linha-possivel-duplicada = Posible duplicada
controles-linha-erro = Con error
controles-linha-ignorada = Ignorada

## Tipos de copia de seguridad

controles-backup-diario = Automática diaria
controles-backup-manual = Manual
controles-backup-migracao = Antes de actualizar la base de datos
controles-backup-importacao = Antes de una importación grande
controles-backup-ia = Antes de una acción de la IA
controles-backup-restauracao = Antes de restaurar
controles-backup-senha = Antes de cambiar la contraseña
controles-backup-fechar = Al cerrar la app

## Quién hizo el cambio

controles-autor-voce = Tú
controles-autor-importacao = Importación
controles-autor-ia = IA
controles-autor-sistema = Sistema

## Campos y operadores de las reglas

controles-campo-descricao = Descripción
controles-campo-favorecido = Beneficiario
controles-campo-notas = Notas
controles-campo-valor = Monto
controles-campo-conta = Cuenta
controles-campo-tipo = Tipo

controles-op-contem = contiene
controles-op-igual = es igual a
controles-op-comeca = empieza con
controles-op-termina = termina con
controles-op-regex = coincide con la regex
controles-op-entre = entre
controles-op-maior-igual = mayor o igual a
controles-op-menor-igual = menor o igual a

## Tablas del núcleo en el historial de cambios

controles-tabela-familia = Familia
controles-tabela-membro = Miembro
controles-tabela-instituicao = Institución
controles-tabela-conta = Cuenta
controles-tabela-membro-conta = Miembro de la cuenta
controles-tabela-cartao = Datos de la tarjeta
controles-tabela-fatura = Resumen de la tarjeta
controles-tabela-categoria = Categoría
controles-tabela-tag = Etiqueta
controles-tabela-lancamento = Movimiento
controles-tabela-parte-divisao = Parte de la división
controles-tabela-tag-lancamento = Etiqueta del movimiento
controles-tabela-anexo = Adjunto
controles-tabela-parcelamento = Plan de cuotas
controles-tabela-recorrencia = Recurrencia
controles-tabela-regra = Regla
controles-tabela-lote-importacao = Lote de importación
controles-tabela-cambio = Tipo de cambio
controles-tabela-orcamento = Presupuesto
controles-tabela-linha-orcamento = Línea del presupuesto
controles-tabela-meta = Meta
controles-tabela-conta-meta = Cuenta de la meta
controles-tabela-projeto = Proyecto
controles-tabela-ativo = Activo
controles-tabela-operacao = Operación
controles-tabela-tela-screener = Vista del screener

## Conteo por tabla en el historial

controles-n-familia = { $n ->
        [one] { $n } familia
       *[other] { $n } familias
    }
controles-n-membro = { $n ->
        [one] { $n } miembro
       *[other] { $n } miembros
    }
controles-n-instituicao = { $n ->
        [one] { $n } institución
       *[other] { $n } instituciones
    }
controles-n-conta = { $n ->
        [one] { $n } cuenta
       *[other] { $n } cuentas
    }
controles-n-membro-conta = { $n ->
        [one] { $n } miembro de la cuenta
       *[other] { $n } miembros de la cuenta
    }
controles-n-cartao = { $n ->
        [one] { $n } datos de la tarjeta
       *[other] { $n } datos de la tarjeta
    }
controles-n-fatura = { $n ->
        [one] { $n } resumen de la tarjeta
       *[other] { $n } resúmenes de la tarjeta
    }
controles-n-categoria = { $n ->
        [one] { $n } categoría
       *[other] { $n } categorías
    }
controles-n-tag = { $n ->
        [one] { $n } etiqueta
       *[other] { $n } etiquetas
    }
controles-n-lancamento = { $n ->
        [one] { $n } movimiento
       *[other] { $n } movimientos
    }
controles-n-parte-divisao = { $n ->
        [one] { $n } parte de la división
       *[other] { $n } partes de la división
    }
controles-n-tag-lancamento = { $n ->
        [one] { $n } etiqueta del movimiento
       *[other] { $n } etiquetas del movimiento
    }
controles-n-anexo = { $n ->
        [one] { $n } adjunto
       *[other] { $n } adjuntos
    }
controles-n-parcelamento = { $n ->
        [one] { $n } plan de cuotas
       *[other] { $n } planes de cuotas
    }
controles-n-recorrencia = { $n ->
        [one] { $n } recurrencia
       *[other] { $n } recurrencias
    }
controles-n-regra = { $n ->
        [one] { $n } regla
       *[other] { $n } reglas
    }
controles-n-lote-importacao = { $n ->
        [one] { $n } lote de importación
       *[other] { $n } lotes de importación
    }
controles-n-cambio = { $n ->
        [one] { $n } tipo de cambio
       *[other] { $n } tipos de cambio
    }
controles-n-orcamento = { $n ->
        [one] { $n } presupuesto
       *[other] { $n } presupuestos
    }
controles-n-linha-orcamento = { $n ->
        [one] { $n } línea del presupuesto
       *[other] { $n } líneas del presupuesto
    }
controles-n-meta = { $n ->
        [one] { $n } meta
       *[other] { $n } metas
    }
controles-n-conta-meta = { $n ->
        [one] { $n } cuenta de la meta
       *[other] { $n } cuentas de la meta
    }
controles-n-projeto = { $n ->
        [one] { $n } proyecto
       *[other] { $n } proyectos
    }
controles-n-ativo = { $n ->
        [one] { $n } activo
       *[other] { $n } activos
    }
controles-n-operacao = { $n ->
        [one] { $n } operación
       *[other] { $n } operaciones
    }
controles-n-tela-screener = { $n ->
        [one] { $n } vista del screener
       *[other] { $n } vistas del screener
    }
