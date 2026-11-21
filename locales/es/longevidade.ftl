# Línea de vida (docs/life-expectancy.md): la barra con las etapas de la vida de cada miembro, los
# hitos financieros y el final difuminado según los promedios de la población, en la pantalla
# Familia, en el registro del miembro, en el planificador de jubilación, en Ajustes › Perfil y en
# la barra ⌘K. Los textos del núcleo están en nucleo-longevidade.ftl.
# Revisión: borrador por IA, sin revisión de un hablante nativo.

longevidade-titulo = Línea de vida
longevidade-titulo-de = Línea de vida de { $nome }

## Etapas de la vida (sección 5)

longevidade-fase-infancia = Infancia
longevidade-fase-adolescencia = Adolescencia
longevidade-fase-juventude = Juventud
longevidade-fase-vida-adulta = Vida adulta
longevidade-fase-longevidade = Longevidad
longevidade-fase-longevidade-avancada = Longevidad avanzada

## La barra

longevidade-cabecalho =
    { $nome } · { $idade ->
        [one] 1 año
       *[other] { $idade } años
    } · { $fase }
longevidade-hoje = { $idade } · hoy
longevidade-final-numeros = { $mediana } · { $p25 } · { $p10 }
longevidade-final-dica = La mitad pasa de los { $mediana }, 1 de cada 4 pasa de los { $p25 } y 1 de cada 10 pasa de los { $p10 }.
longevidade-marco-quando = a los { $idade } · en { $ano }
longevidade-marco-rotulo = { $rotulo }: a los { $idade }, en { $ano }
longevidade-marcos-rotulo = Hitos a los { $idade }: { $rotulos }
longevidade-marco-aposentadoria = Jubilación
longevidade-marco-inss = INSS
longevidade-marco-plano = Plan de jubilación
longevidade-marco-divida = Fin de la deuda
longevidade-marco-meta = Meta
longevidade-marco-evento = Evento de vida

## Texto junto a la barra, en el tono del perfil

# $onde: longevidade-onde (el país con la preposición correcta).
longevidade-onde =
    { $pais ->
        [US] de Estados Unidos
       *[other] de { $nome }
    }
longevidade-texto-simples =
    { $sexo ->
        [male] Según los promedios { $onde }, de cada 100 hombres nacidos en { $ano } que llegaron a los { $idade }, la mitad pasa de los { $mediana } y 1 de cada 4 pasa de los { $p25 }.
        [female] Según los promedios { $onde }, de cada 100 mujeres nacidas en { $ano } que llegaron a los { $idade }, la mitad pasa de los { $mediana } y 1 de cada 4 pasa de los { $p25 }.
       *[other] Según los promedios { $onde }, de cada 100 personas nacidas en { $ano } que llegaron a los { $idade }, la mitad pasa de los { $mediana } y 1 de cada 4 pasa de los { $p25 }.
    }
longevidade-texto-equilibrado =
    { $sexo ->
        [male] Según los promedios { $onde }, de cada 100 hombres nacidos en { $ano } que llegaron a los { $idade }, la mitad pasa de los { $mediana }, 1 de cada 4 pasa de los { $p25 } y 1 de cada 10 pasa de los { $p10 }.
        [female] Según los promedios { $onde }, de cada 100 mujeres nacidas en { $ano } que llegaron a los { $idade }, la mitad pasa de los { $mediana }, 1 de cada 4 pasa de los { $p25 } y 1 de cada 10 pasa de los { $p10 }.
       *[other] Según los promedios { $onde }, de cada 100 personas nacidas en { $ano } que llegaron a los { $idade }, la mitad pasa de los { $mediana }, 1 de cada 4 pasa de los { $p25 } y 1 de cada 10 pasa de los { $p10 }.
    }
longevidade-texto-tecnico =
    Mediana { $mediana } · P25 { $p25 } · P10 { $p10 } · e({ $idade }) = { $restante } (cohorte, { $fonte }, { $pais }, { $sexo ->
        [male] masculino
        [female] femenino
       *[other] ambos sexos
    })
longevidade-texto-coorte = Esperanza de vida al nacer de la generación de { $ano }: { $e0 } años.
longevidade-texto-periodo = Tabla de período ({ $fonte }): e0 = { $e0 } · e({ $idade }) = { $restante }.
longevidade-texto-crianca = Etapa actual: { $fase }. Para menores de 18, el final de la barra queda oculto; un adulto puede mostrarlo en Ajustes › Perfil › Línea de vida.
longevidade-fonte = Promedios de la población. Fuente: { $fonte }.
longevidade-fonte-sintetica = Datos sintéticos de desarrollo, no las tablas oficiales.

## Texto alternativo de la barra (los mismos números)

longevidade-alt =
    Línea de vida de { $nome }: { $idade ->
        [one] 1 año
       *[other] { $idade } años
    }, { $fase }.
longevidade-alt-final = La mitad de las personas de la misma generación pasa de los { $mediana }, 1 de cada 4 pasa de los { $p25 } y 1 de cada 10 pasa de los { $p10 }.
longevidade-alt-marcos = Hitos: { $marcos }.
longevidade-alt-marco = { $rotulo } a los { $idade } ({ $ano })

## Tarjeta de la familia (02 · Familia)

longevidade-familia-intro = La barra de cada persona muestra las etapas de la vida, dónde está hoy y los planes de dinero a lo largo de ella. El final difuminado viene de los promedios de la población del país, no de una predicción.
longevidade-vista = Vista de la línea de vida
longevidade-vista-idade = Por edad
longevidade-vista-tempo = Línea de tiempo
longevidade-legenda-final = Final difuminado: la mitad, 1 de cada 4 y 1 de cada 10 de las personas de la misma generación pasan de esas edades.
longevidade-casal = Al menos uno de ustedes dos: según los promedios, la probabilidad baja al 25% en { $ano25 } y al 10% en { $ano10 }.
longevidade-casal-marca = { $pct } en { $ano }
longevidade-tempo-na-aposentadoria =
    En la jubilación de { $nome } ({ $ano }): { $idade ->
        [one] 1 año
       *[other] { $idade } años
    } · { $fase }
longevidade-eixo-anos = Años
longevidade-nenhum = Nadie de la familia tiene fecha de nacimiento todavía.
longevidade-escondida = La barra de { $nome } está oculta (Ajustes › Perfil › Línea de vida).
longevidade-como = Cómo lo calculamos
longevidade-como-geracao = Por generación: sigue las tasas de mortalidad que enfrenta la generación de cada persona en cada edad (las pasadas y las proyectadas por la ONU), y no las de un solo año.
longevidade-como-idade = A partir de la edad de hoy: quien ya llegó a una edad pasó por los riesgos anteriores, y la edad esperada sube.
longevidade-como-limites = Son promedios de la población del país. No consideran salud, ingresos, tabaquismo, región ni antecedentes familiares, que cambian mucho el resultado de cada persona.
longevidade-como-futuro = El futuro es la proyección media de la ONU, no un hecho. Todo se calcula en la propia computadora; nada sale de ella.

## Lo que falta en el registro

longevidade-falta-titulo = Para completar la línea de vida
longevidade-falta-dica = La fecha de nacimiento y el sexo eligen la tabla correcta. “Prefiero no decirlo” usa el promedio de todas las personas.
longevidade-falta-de = { $nome }: falta { $itens }
longevidade-falta-nascimento = fecha de nacimiento
longevidade-falta-sexo = sexo
longevidade-falta-nascimento-de = Nacimiento de { $nome }
longevidade-falta-sexo-de = Sexo de { $nome }
longevidade-falta-salvo = Registro de { $nome } guardado

## Ajustes › Perfil › Línea de vida

longevidade-ajustes-ligar = Mostrar la línea de vida
longevidade-ajustes-ligar-dica = Una barra con las etapas de la vida de cada persona y los planes de dinero a lo largo de ella, basada en los promedios de la población del país. Algunas personas prefieren no ver este tipo de información.
longevidade-ajustes-minha = Mostrar mi barra
longevidade-ajustes-minha-dica = Si la ocultas, tu barra desaparece de Familia y del planificador de jubilación; los demás miembros siguen viendo las suyas.
longevidade-ajustes-barras = Barras visibles
longevidade-ajustes-barra-de = Mostrar la barra de { $nome }
longevidade-ajustes-criancas = Mostrar el final de la barra para niños
longevidade-ajustes-criancas-dica = Para menores de 18, la barra muestra las etapas y los hitos; el final difuminado solo aparece con este ajuste.
longevidade-ajustes-avancada = Destacar la longevidad avanzada (80+)
longevidade-ajustes-avancada-dica = Divide la última etapa a los 80 años, cuando la salud y los cuidados pesan más en el presupuesto.
longevidade-ajustes-nada-guardado = Desactivarla no borra nada: la línea se calcula en el momento, en la propia computadora, a partir de la fecha de nacimiento, el sexo y el país.
longevidade-ajustes-ligada = Línea de vida activada
longevidade-ajustes-desligada = Línea de vida desactivada

## Planificador de jubilación (sección 7)

longevidade-apos-sugestao = Sugerencia de la línea de vida: { $idade } (1 de cada 10 personas como tú pasa de esa edad).
longevidade-apos-sugestao-casal = Sugerencia de la línea de vida: { $idade } (la probabilidad de que al menos uno de ustedes dos pase de esa edad es de 1 de cada 10).
longevidade-apos-usar = Usar { $idade }
longevidade-apos-risco =
    Tu plan llega hasta los { $idade }. Según los promedios, { $n ->
        [0] menos de 1 de cada 100 personas como tú pasa de esa edad.
        [1] cerca de 1 de cada 100 personas como tú pasa de esa edad.
        [10] 1 de cada 10 personas como tú pasa de esa edad.
       *[other] cerca de { $n } de cada 100 personas como tú pasan de esa edad.
    }
longevidade-apos-risco-casal =
    Tu plan llega hasta los { $idade }. Según los promedios, la probabilidad de que al menos uno de ustedes dos pase de esa edad es de { $n ->
        [0] menos de 1 de cada 100.
        [10] 1 de cada 10.
       *[other] cerca de { $n } de cada 100.
    }

## Cuestionario y barra ⌘K

longevidade-onboarding-ligada = La línea de vida está activada. Si falta la fecha de nacimiento o el sexo de alguien, la pantalla Familia lo pide.
longevidade-onboarding-ir = Abrir Familia
longevidade-cmd-busca = línea de vida, esperanza de vida, longevidad, etapas de la vida, jubilación, edad
