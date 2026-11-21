# Pantalla Desarrollador (comentarios del agente) y catálogo interno de componentes
# (#/dev/componentes) (docs/i18n-design.md). Revisión: borrador por IA, sin revisión nativa.

## Áreas, tipos y estados de los comentarios

desenvolvedor-area-agente = Agente
desenvolvedor-area-mcp = MCP
desenvolvedor-area-importacao = Importación
desenvolvedor-area-lancamentos = Movimientos
desenvolvedor-area-carteira = Cartera
desenvolvedor-area-ui = Interfaz
desenvolvedor-area-dados = Datos
desenvolvedor-area-outro = Otro
desenvolvedor-tipo-bug = Error
desenvolvedor-tipo-melhoria = Mejora
desenvolvedor-tipo-ferramenta-faltando = Falta una herramienta
desenvolvedor-tipo-dados = Datos
desenvolvedor-tipo-documentacao = Documentación
desenvolvedor-status-aberto = Abierto
desenvolvedor-status-em-andamento = En curso
desenvolvedor-status-resolvido = Resuelto
desenvolvedor-status-descartado = Descartado
desenvolvedor-voce = tú

## Markdown para un issue

desenvolvedor-md-metadados = **Área:** { $area } · **Tipo:** { $tipo } · **Estado:** { $status }
desenvolvedor-md-importancia = { $n ->
    [one] **Importancia:** { $importancia } ({ $n } refuerzo)
   *[other] **Importancia:** { $importancia } ({ $n } refuerzos)
}
desenvolvedor-md-registrado = **Registrado por:** { $autor } el { $data }
desenvolvedor-md-reforcos = Refuerzos
desenvolvedor-md-titulo = Comentarios del agente · Treasury
desenvolvedor-md-exportado = { $n ->
    [one] Exportado el { $data } · { $n } comentario.
   *[other] Exportado el { $data } · { $n } comentarios.
}

## Pantalla

desenvolvedor-erro-copiar = No se pudo copiar
desenvolvedor-atualizado = Comentario actualizado
desenvolvedor-erro-gravar = No se pudo guardar
desenvolvedor-editar-titulo = Editar comentario
desenvolvedor-titulo = Título
desenvolvedor-area = Área
desenvolvedor-tipo = Tipo
desenvolvedor-importancia = Importancia
desenvolvedor-em-andamento = En curso
desenvolvedor-resolvido = Resuelto
desenvolvedor-descartar = Descartar
desenvolvedor-voltar-aberto = Volver a abierto
desenvolvedor-reabrir = Reabrir
desenvolvedor-feedback-rotulo = Comentario: { $titulo }
desenvolvedor-importancia-dica = Importancia: empieza en 1 y cada refuerzo suma 1
desenvolvedor-n-reforcos = { $n ->
    [one] { $n } refuerzo
   *[other] { $n } refuerzos
}
desenvolvedor-ultimo = último { $data }
desenvolvedor-reforcos = Refuerzos
desenvolvedor-sem-nota = (sin nota)
desenvolvedor-status-mudou = { $status ->
    [aberto] Comentario reabierto
    [em_andamento] Comentario en curso
    [resolvido] Comentario resuelto
   *[descartado] Comentario descartado
}
desenvolvedor-erro-status = No se pudo cambiar el estado
desenvolvedor-md-copiado = Markdown copiado
desenvolvedor-copiar-md = Copiar Markdown
desenvolvedor-apagar-titulo = Eliminar este comentario
desenvolvedor-apagar-texto = El comentario, sus refuerzos y vínculos desaparecen de la pantalla. Puedes deshacerlo desde el aviso o desde el historial de cambios.
desenvolvedor-apagado = Comentario eliminado
desenvolvedor-erro-apagar = No se pudo eliminar
desenvolvedor-descricao = Lo que el agente notó al trabajar y quiere que sepas como desarrollador: herramientas que faltaron, datos confusos, errores y vacíos de la documentación. Primero busca y refuerza lo que ya existe en lugar de duplicarlo.
desenvolvedor-n-feedbacks = { $n ->
    [one] { $n } comentario
   *[other] { $n } comentarios
}
desenvolvedor-n-copiados = { $n ->
    [one] { $n } comentario copiado
   *[other] { $n } comentarios copiados
}
desenvolvedor-copiar-tudo = Copiar todo
desenvolvedor-exportar-md = Exportar Markdown
desenvolvedor-exportar-salvar = Guardar comentarios en Markdown
desenvolvedor-exportados = { $n ->
    [one] { $n } comentario exportado
   *[other] { $n } comentarios exportados
}
desenvolvedor-erro-exportar = No se pudieron exportar los comentarios
desenvolvedor-desligado = Modo desarrollador desactivado
desenvolvedor-desligado-titulo = Activa las opciones de desarrollador para ver los comentarios del agente
desenvolvedor-desligado-texto = Con el modo desarrollador activado, el agente obtiene herramientas para registrar y reforzar comentarios técnicos sobre Treasury, que aparecen aquí.
desenvolvedor-abrir-ajustes = Abrir Ajustes
desenvolvedor-card = Comentarios del agente
desenvolvedor-card-rodape = El agente registra, refuerza y marca como resuelto, directamente y sin propuesta. Mover en el tablero, editar y eliminar quedan en el historial de cambios y se pueden deshacer.
desenvolvedor-filtro-status = Estado
desenvolvedor-buscar = Buscar en los comentarios
desenvolvedor-buscar-dica = Palabras del título, del texto, de los refuerzos o de quien lo registró
desenvolvedor-n-de-feedbacks = { $n } de { $total }
desenvolvedor-abertos = Abiertos y en curso
desenvolvedor-todos = Todos
desenvolvedor-todas-areas = Todas las áreas
desenvolvedor-nenhum-filtro = Ningún comentario con este filtro.
desenvolvedor-nenhum = No hay comentarios abiertos. Cuando el agente encuentre una herramienta que falta, un dato confuso o un error, lo registra aquí.
desenvolvedor-lista = Comentarios

## Tablero (vista predeterminada) y panel lateral

desenvolvedor-visao = Vista
desenvolvedor-visao-quadro = Tablero
desenvolvedor-visao-lista = Lista
desenvolvedor-quadro = Tablero de comentarios
desenvolvedor-coluna = { $status }: { $n ->
    [1] { $n } elemento
   *[other] { $n } elementos
}
desenvolvedor-coluna-vazia = Arrastra una tarjeta aquí.
desenvolvedor-todos-tipos = Todos los tipos
desenvolvedor-proximo = Siguiente en la fila
desenvolvedor-prioridade-n = Prioridad { $n }
desenvolvedor-prioridade-curta = prior.
desenvolvedor-reordenado = Orden de la columna actualizado
desenvolvedor-mover-para = { $status ->
    [aberto] Volver a abierto
    [em_andamento] Empezar
    [resolvido] Resuelto
   *[descartado] Descartar
}
desenvolvedor-subir = Subir
desenvolvedor-descer = Bajar
desenvolvedor-secao-problema = Problema
desenvolvedor-secao-proposta = Propuesta
desenvolvedor-secao-aceite = Criterio de aceptación
desenvolvedor-secao-onde = Dónde cambiar
desenvolvedor-secao-relacionados = Relacionados
desenvolvedor-secao-passos = Cómo reproducirlo
desenvolvedor-secao-vazia = No informado.
desenvolvedor-onde-dica = Una ruta por línea, con la línea si la sabes (crates/treasury-core/src/mod.rs:120).
desenvolvedor-responsavel = Responsable
desenvolvedor-etiquetas = Etiquetas
desenvolvedor-etiquetas-dica = Separadas por comas.
desenvolvedor-impacto = { $impacto ->
    [alto] Impacto alto
    [medio] Impacto medio
   *[baixo] Impacto bajo
}
desenvolvedor-confirmado = Confirmado por el usuario
desenvolvedor-nota-tipo = { $kind ->
    [reforco] refuerzo
    [status] cambio de estado
    [mescla] fusionado
   *[comentario] comentario
}
desenvolvedor-vinculo = { $kind ->
    [duplicado] { $entrada ->
        [sim] Tiene como duplicado
       *[nao] Duplicado de
    }
    [depende_de] { $entrada ->
        [sim] Es requisito de
       *[nao] Depende de
    }
   *[relacionado] Relacionado con
}
desenvolvedor-copiar-caminho = Copiar la ruta
desenvolvedor-caminho-copiado = Ruta copiada
desenvolvedor-origem = Registrado por { $autor } el { $data }
desenvolvedor-versao = versión { $versao }
desenvolvedor-cc-contexto = Comentario `{ $id }` de Treasury · área { $area } · tipo { $tipo } · prioridad { $prioridade } · { $status }.
desenvolvedor-cc-esperado = Esperado
desenvolvedor-cc-obtido = Obtenido
desenvolvedor-cc-sem-aceite = No informado: define una prueba que demuestre la corrección.
desenvolvedor-cc-fechar = Al terminar, ejecuta las pruebas y márcalo como resuelto en el MCP de Treasury: `resolve_dev_feedback` con `id` = `{ $id }` y el motivo (qué cambió, versión o commit).

## Catálogo de componentes (solo en desarrollo; valores ficticios)

desenvolvedor-comp-selo = Catálogo de componentes · dev
desenvolvedor-comp-tema = Tema
desenvolvedor-comp-privacidade = Modo privacidad
desenvolvedor-comp-voltar = Volver a la app →
desenvolvedor-comp-titulo = Sistema de diseño
desenvolvedor-comp-descricao = Componentes característicos de Treasury, con el aspecto de un instrumento de medición. Cambia el tema arriba para revisar el contraste y las paletas. Todos los valores son ficticios.
desenvolvedor-comp-meta = Revisión visual
desenvolvedor-comp-tokens = Tokens de color
desenvolvedor-comp-contraste = Contraste sobre --bg · tema { $tema }
desenvolvedor-comp-tipografia = Tipografía
desenvolvedor-comp-rotulo-tipo = Etiqueta · 11px · mayúsculas · 0,12em
desenvolvedor-comp-titulo-tela = Título de pantalla · Inter 600
desenvolvedor-comp-texto-corrido = Texto corrido en Inter. ¿A dónde fue el dinero del mes? ¿Gastamos más o solo subieron los precios? Todo número tiene un origen rastreable.
desenvolvedor-comp-texto-secundario = Texto secundario en --text-dim.
desenvolvedor-comp-medida = Medida { $tamanho } · { $px }px mono 500
desenvolvedor-comp-cards = Tarjetas de instrumento
desenvolvedor-comp-patrimonio = Patrimonio neto
desenvolvedor-comp-fonte = Fuente
desenvolvedor-comp-contas-carteira = Cuentas + cartera
desenvolvedor-comp-atualizado = Actualizado
desenvolvedor-comp-total-brl = Total en BRL
desenvolvedor-comp-12-meses = en 12 meses
desenvolvedor-comp-saldos-conciliados = Saldos conciliados
desenvolvedor-comp-patrimonio-subindo = Patrimonio subiendo en los últimos 24 meses
desenvolvedor-comp-ipca = IPCA · 12 meses
desenvolvedor-comp-ref = Ref.
desenvolvedor-comp-ref-valor = oct/26 ≈
desenvolvedor-comp-inflacao = Inflación acumulada
desenvolvedor-comp-delta-ipca = ▲ +0,12 p. p.
desenvolvedor-comp-projecao-focus = proyección Focus para octubre
desenvolvedor-comp-nota-focus = Octubre aún no se publicó: usa la mediana del Focus hasta que salga el dato oficial.
desenvolvedor-comp-poupanca = Tasa de ahorro
desenvolvedor-comp-periodo = Período
desenvolvedor-comp-periodo-valor = oct/26
desenvolvedor-comp-base = Base
desenvolvedor-comp-competencia = devengo
desenvolvedor-comp-formula-poupanca = Ingresos − gastos ÷ ingresos
desenvolvedor-comp-delta-poupanca = ▼ −2,4 p. p.
desenvolvedor-comp-vs-setembro = vs. septiembre
desenvolvedor-comp-receitas = Ingresos
desenvolvedor-comp-despesas = Gastos
desenvolvedor-comp-saldo = Saldo
desenvolvedor-comp-figura = Figura
desenvolvedor-comp-patrimonio-inflacao = Patrimonio × inflación
desenvolvedor-comp-patrimonio-24 = Patrimonio neto, 24 meses
desenvolvedor-comp-nominal = Nominal
desenvolvedor-comp-real-ipca = Real (IPCA)
desenvolvedor-comp-fonte-figura = Fuente · saldos conciliados; deflactor IBGE/SIDRA 1737
desenvolvedor-comp-grafico-rotulo = Patrimonio neto en 24 meses: el nominal subió de 412 a 548 mil reales; corregido por el IPCA, a 501 mil.
desenvolvedor-comp-tabela = Tabla técnica
desenvolvedor-comp-tabela-dica = ↑/↓ · Inicio/Fin · Enter · haz clic en el encabezado para ordenar
desenvolvedor-comp-lancamentos-outubro = Movimientos de octubre
desenvolvedor-comp-lancamentos-exemplo = Movimientos de ejemplo
desenvolvedor-comp-abrir = Abrir “{ $descricao }”
desenvolvedor-comp-ativado = Activado con Enter o doble clic.
desenvolvedor-comp-saldo-periodo = Saldo del período
desenvolvedor-comp-data = Fecha
desenvolvedor-comp-descricao-col = Descripción
desenvolvedor-comp-categoria = Categoría
desenvolvedor-comp-membro = Miembro
desenvolvedor-comp-valor = Valor
desenvolvedor-comp-curva-cdi = Curva CDI (Focus)
desenvolvedor-comp-rendimento-projetado = Rendimiento proyectado hasta el cierre del mes.
desenvolvedor-comp-ex-supermercado = Supermercado del Barrio
desenvolvedor-comp-ex-salario = Salario
desenvolvedor-comp-ex-farmacia = Farmacia Central
desenvolvedor-comp-ex-aluguel = Alquiler
desenvolvedor-comp-ex-escola = Escuela — mensualidad
desenvolvedor-comp-ex-cdb = Rendimiento CDB (estimado)
desenvolvedor-comp-ex-streaming = Streaming
desenvolvedor-comp-ex-luz = Factura de luz
desenvolvedor-comp-cat-alimentacao = Alimentación
desenvolvedor-comp-cat-receitas = Ingresos
desenvolvedor-comp-cat-saude = Salud
desenvolvedor-comp-cat-moradia = Vivienda
desenvolvedor-comp-cat-educacao = Educación
desenvolvedor-comp-cat-investimentos = Inversiones
desenvolvedor-comp-cat-assinaturas = Suscripciones
desenvolvedor-comp-dinheiro = Dinero y origen
desenvolvedor-comp-neutro = Neutro
desenvolvedor-comp-ganho = Ganancia
desenvolvedor-comp-perda = Pérdida
desenvolvedor-comp-outra-moeda = Otra moneda
desenvolvedor-comp-estimado = Estimado
desenvolvedor-comp-sem-moeda = Sin moneda
desenvolvedor-comp-sempre-visivel = Siempre visible
desenvolvedor-comp-selo-oficial = Sello oficial
desenvolvedor-comp-selo-estimado = Sello estimado
desenvolvedor-comp-focus-mediana = Focus (mediana)
desenvolvedor-comp-projecao-ipca = Proyección hasta que salga el IPCA oficial.
desenvolvedor-comp-privacidade-dica = Activa el modo privacidad ({ $atalho }) para ver todos los valores convertirse en •••••.
desenvolvedor-comp-campos = Campos
desenvolvedor-comp-texto = Texto
desenvolvedor-comp-ajuda = Ayuda debajo del campo.
desenvolvedor-comp-familia-exemplo = Familia Silva
desenvolvedor-comp-com-erro = Con error
desenvolvedor-comp-informe-familia = Ingresa el nombre de la familia.
desenvolvedor-comp-senha = Contraseña
desenvolvedor-comp-senha-exemplo = una contraseña cualquiera
desenvolvedor-comp-centavos = Centavos: { $valor }
desenvolvedor-comp-moeda = Moneda
desenvolvedor-comp-real = Real brasileño
desenvolvedor-comp-dolar = Dólar estadounidense
desenvolvedor-comp-euro = Euro
desenvolvedor-comp-iene = Yen
desenvolvedor-comp-bitcoin = Bitcoin (próximamente)
desenvolvedor-comp-botoes = Botones
desenvolvedor-comp-primario = Primario
desenvolvedor-comp-secundario = Secundario
desenvolvedor-comp-fantasma = Fantasma
desenvolvedor-comp-perigo = Peligro
desenvolvedor-comp-demonstracao = Demostración
desenvolvedor-comp-derivando = Derivando clave…
desenvolvedor-comp-carregando = Cargando
desenvolvedor-comp-desabilitado = Deshabilitado
desenvolvedor-comp-pequeno = Pequeño
desenvolvedor-comp-grande = Grande
desenvolvedor-comp-bloquear = Bloquear bóveda
desenvolvedor-comp-alternancia = Alternancia activada
desenvolvedor-comp-dialogos = Diálogo, pestañas y notificaciones
desenvolvedor-comp-confirmar-exclusao = Confirmar eliminación
desenvolvedor-comp-confirmar-texto = El movimiento va al historial de cambios y se puede deshacer.
desenvolvedor-comp-abrir-dialogo = Abrir diálogo
desenvolvedor-comp-excluir = Eliminar
desenvolvedor-comp-toast-info = Cotizaciones actualizadas
desenvolvedor-comp-toast-sucesso = Copia de seguridad verificada
desenvolvedor-comp-toast-atencao = IPCA estimado
desenvolvedor-comp-toast-erro = Falló la importación
desenvolvedor-comp-toast-texto = Ejemplo de notificación de Treasury.
desenvolvedor-comp-notificacao = { $tom ->
    [info] Notificación · aviso
    [success] Notificación · éxito
    [warning] Notificación · atención
   *[error] Notificación · error
}
desenvolvedor-comp-visao = Vista
desenvolvedor-comp-m2 = M2 · Fase 2
desenvolvedor-comp-rent-nominal = Rentabilidad nominal
desenvolvedor-comp-rent-real = Rentabilidad real
desenvolvedor-comp-estados = Estados e indicadores
desenvolvedor-comp-fase = Fase 2 · Índices y cartera
desenvolvedor-comp-carteira-fase = La cartera llega en la Fase 2
desenvolvedor-comp-vazio-texto = Estado vacío de una pantalla que aún no está implementada, con lo que aparecerá aquí.
desenvolvedor-comp-item-posicoes = Posiciones y precio promedio
desenvolvedor-comp-item-proventos = Dividendos
desenvolvedor-comp-item-twr = TWR y XIRR
desenvolvedor-comp-item-real = Rentabilidad real
desenvolvedor-comp-roadmap = Ver hoja de ruta
desenvolvedor-comp-importando = Importando
desenvolvedor-comp-sincronizando = Sincronizando cotizaciones
desenvolvedor-comp-pausado = En pausa
desenvolvedor-comp-rotulo-dim = Etiqueta dim
desenvolvedor-comp-rotulo-fg = Etiqueta fg
desenvolvedor-comp-rotulo-accent = Etiqueta accent
