# Línea de vida (núcleo, `longevity`; docs/life-expectancy.md): etapas, fuente y atribución de
# las tablas, hitos, la pregunta del cuestionario, errores y los resúmenes del MCP. La cita de la
# ONU queda en inglés, como la ONU pide.

## Etapas de la vida (sección 5)

nucleo-longevidade-fase-infancia = Infancia
nucleo-longevidade-fase-adolescencia = Adolescencia
nucleo-longevidade-fase-juventude = Juventud
nucleo-longevidade-fase-vida-adulta = Vida adulta
nucleo-longevidade-fase-longevidade = Longevidad

## Fuente y atribución (DATA_LICENSES.md)

nucleo-longevidade-fonte-nome = ONU, World Population Prospects 2024
nucleo-longevidade-fonte-nome-brasil = ONU, World Population Prospects 2024, y IBGE 2024
nucleo-longevidade-fonte-nome-sintetica = Tabla sintética de desarrollo (no son los datos de la ONU)
nucleo-longevidade-atribuicao-onu = United Nations, Department of Economic and Social Affairs, Population Division (2024). World Population Prospects 2024, Online Edition. Licencia CC BY 3.0 IGO. Datos recortados y transformados por Treasury (solo los países de la app, edades de 0 a 100, variante media, esperanza de vida por generación calculada en la app); la ONU no respalda Treasury ni este uso de los datos.
nucleo-longevidade-atribuicao-ibge = Referencia de Brasil: IBGE, Tábuas Completas de Mortalidade 2024.
nucleo-longevidade-atribuicao-sintetica = Atención: esta versión usa una tabla sintética de desarrollo, no las cifras de la ONU ni del IBGE.
nucleo-longevidade-ibge-fonte = IBGE 2024
nucleo-longevidade-ibge-fonte-sintetica = IBGE 2024 (sintético)

## Hitos sobre la barra

nucleo-longevidade-marco-aposentadoria = Jubilación
nucleo-longevidade-marco-inss = Inicio del INSS
nucleo-longevidade-marco-previdencia = Inicio de la jubilación pública
nucleo-longevidade-marco-fim-plano = Fin del plan de jubilación
nucleo-longevidade-marco-divida = Fin de { $nome }

## Cuestionario de bienvenida (8.ª pregunta, solo en el de la familia)

nucleo-longevidade-pergunta = ¿Quieres ver la línea de vida de la familia?
nucleo-longevidade-pergunta-dica = Una barra con las etapas de la vida de cada persona y los planes de dinero a lo largo de ella, basada en promedios de la población del país. Algunas personas prefieren no ver este tipo de información.
nucleo-longevidade-pergunta-sim = Sí, quiero verla
nucleo-longevidade-pergunta-nao = No, gracias
nucleo-longevidade-pergunta-depois = Decidir después

## Errores

nucleo-longevidade-erro-sem-nascimento = { $nome } no tiene fecha de nacimiento: la línea de vida la necesita.
nucleo-longevidade-erro-pais = No hay tabla de mortalidad para { $pais }.
nucleo-longevidade-erro-dados = No se pudieron leer las tablas de longevidad incluidas en la app: { $erro }
nucleo-longevidade-erro-so-interface = los ajustes de la línea de vida solo cambian desde la app
nucleo-longevidade-erro-desligada = La línea de vida está desactivada en esta bóveda (Ajustes › Perfil › Línea de vida).
nucleo-longevidade-erro-escondido = { $nome } prefirió no mostrar su propia línea de vida.

## Resúmenes del MCP (el agente habla de promedios de la población, nunca de predicciones)

nucleo-longevidade-mcp-membro = { $nome }, { $idade } años ({ $fase }): según los promedios de la población (tablas de la ONU de 2024), entre las personas como { $nome }, la mitad pasa de los { $mediana } años, 1 de cada 4 pasa de los { $p25 } y 1 de cada 10, de los { $p10 }.
nucleo-longevidade-mcp-membro-crianca = { $nome }, { $idade } años ({ $fase }): el final de la barra queda oculto para niños (ajuste de la familia).
nucleo-longevidade-mcp-familia = { $n ->
        [one] Línea de vida de 1 miembro.
       *[other] Línea de vida de { $n } miembros.
    }
nucleo-longevidade-mcp-casal = Probabilidad de que al menos uno de los dos adultos de la pareja esté vivo: baja al 25 % en { $ano25 } y al 10 % en { $ano10 }.
nucleo-longevidade-mcp-faltam = { $n ->
        [one] A 1 miembro le falta la fecha de nacimiento o el sexo.
       *[other] A { $n } miembros les falta la fecha de nacimiento o el sexo.
    }
nucleo-longevidade-mcp-fonte = Promedios de la población; fuente: { $fonte }.
nucleo-longevidade-mcp-desligada = La línea de vida está desactivada.
