# Perfil: bienvenida y cuestionario, panel simple y cuadro por miembro (docs/profile.md,
# docs/i18n-design.md). Revisión: borrador por IA, sin revisión nativa. El texto entre ** sale en
# negrita; los argumentos valor, meta, entrou, saiu, botao y link son elementos que la pantalla
# coloca en su lugar.

## Bienvenida y cuestionario

perfil-confianca = Tus respuestas ayudan a dejar la app a tu manera. Puedes cambiarlo después.
perfil-questionario-de = Cuestionario de { $nome }
perfil-refazer = Rehacer el cuestionario
perfil-boas-vindas = Bienvenido a Treasury
perfil-boas-vindas-texto = En 2 minutos la app se ajusta a ti: lo que aparece en la pantalla, cómo se explican las cosas y por dónde empezar.
perfil-personalizar = Quiero personalizar la app para mí
perfil-personalizar-dica = 7 preguntas rápidas, de opción múltiple.
perfil-completo = Ya sé lo que quiero: usar la interfaz completa
perfil-completo-dica = Modo avanzado, con todas las pantallas a la vista. Sin preguntas.
perfil-demo-convite = ¿Solo quieres conocer la app? { $botao } con datos de ejemplo, separada de esta.
perfil-demo-criar = Crear una bóveda de demostración
perfil-resultado = Preparamos la app en el modo { $nivel }
perfil-resultado-dica = Puedes cambiar el nivel cuando quieras en Ajustes › Perfil, y todo lo que queda oculto sigue a un clic.
perfil-comecar = Empezar
perfil-pergunta = Pregunta { $n } de { $total }
perfil-progresso = Progreso del cuestionario
perfil-pular = Saltar esta pregunta
perfil-concluir = Terminar

## Panel simple

perfil-mes-vazio = Todavía no hay ingresos ni gastos anotados este mes. ¿Qué tal anotar el primero?
perfil-mes-sem-entrada = Salieron { $saida } y todavía no hay ingresos anotados este mes.
perfil-mes-sobrou = Sobraron { $sobra }, o el { $pct }% de lo que entró. Vas por buen camino.
perfil-mes-faltou = Salió { $falta } más de lo que entró. Pasa: mira abajo a dónde fue el dinero y dónde se puede ajustar.
perfil-resumo-mes = Resumen del mes
perfil-entrou = Cuánto entró
perfil-saiu = Cuánto salió
perfil-sobrou = Lo que sobró
perfil-reserva = Tu fondo de emergencia
perfil-calculando = Calculando…
perfil-reserva-comecar = Anota tus gastos del día a día (supermercado, alquiler, cuentas) durante un mes y la app calcula cuánto guardar. Para empezar, una buena meta es **{ $valor }**.
perfil-reserva-meta = Meta: { $meta }, o { $meses } meses de tus gastos esenciales.
perfil-reserva-ja-tem = Ya tienes { $valor }.
perfil-reserva-nada = Todavía no hay nada guardado: se puede empezar con poco.
perfil-reserva-progresso = Progreso del fondo de emergencia
perfil-reserva-completa = ¡Felicidades! Tu fondo de emergencia está completo.
perfil-reserva-caminho = { $pct }% del camino.
perfil-reserva-criar = Crear la meta del fondo de emergencia
perfil-reserva-criar-frase = { $link } para seguir cuánto ya guardaste.
perfil-onde-foi = A dónde fue el dinero
perfil-onde-item = { $valor } ({ $pct }% de lo que salió)
perfil-onde-vazio = Todavía no hay gastos anotados este mes.
perfil-sem-categoria =
    { $n ->
        [one] { $n } movimiento sin categoría.
       *[other] { $n } movimientos sin categoría.
    }
perfil-escolher-categoria = Elegir la categoría
perfil-seu-mes = Tu mes de { $mes }
perfil-anotar-gasto = Anotar un gasto
perfil-anotar-entrada = Anotar un ingreso
perfil-dica-do-dia = Consejo del día
perfil-ver-completo = Ver panel completo
perfil-o-que-e = ¿Qué es { $termo }?

## Cuadro por miembro de la familia

perfil-membros-titulo = Por miembro de la familia
perfil-membros = Miembros
perfil-membro-nivel-familia = { $nivel } (de la familia)
perfil-membro-entrou = Entró
perfil-membro-saiu = Salió
perfil-membro-sobrou = Sobró
perfil-membro-patrimonio = Patrimonio { $valor }
perfil-membro-guardou = guardó { $pct } de lo que entró en 12 meses
perfil-membro-contas =
    { $n ->
        [one] { $n } cuenta
       *[other] { $n } cuentas
    }
perfil-membro-dividas =
    { $n ->
        [one] { $n } deuda
       *[other] { $n } deudas
    }
perfil-ver-como = Ver como { $nome }
perfil-fazer-questionario = Hacer el cuestionario
perfil-membros-familia = De la familia (sin miembro, como cuentas conjuntas): entró { $entrou }, salió { $saiu }.
