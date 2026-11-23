# Datos de los activos con fuente (enriquecimiento, fase 1; docs/agent-enrichment.md): sector,
# segmento, ISIN, CNPJ y los demás campos del registro completados con las fuentes ya
# descargadas, con origen, confianza, conflictos y diferencias.
# Revisión: borrador de IA, aún sin revisión de un hablante nativo.

## Campos

dados-campo-sector = Sector
dados-campo-subsector = Subsector
dados-campo-segment = Segmento
dados-campo-isin = ISIN
dados-campo-cnpj = CNPJ
dados-campo-issuer_cnpj = CNPJ del emisor
dados-campo-issuer_name = Emisor
dados-campo-share_kind = Tipo de acción
dados-campo-listing_segment = Segmento de cotización
dados-campo-cvm_code = Código CVM
dados-campo-cik = CIK (SEC)
dados-campo-fii_mandate = Mandato del FII
dados-campo-fii_management = Gestión del FII
dados-campo-fund_class_cvm = Clase CVM
dados-campo-fund_class_anbima = Clase ANBIMA
dados-campo-benchmark = Índice de referencia
dados-campo-manager = Gestor
dados-campo-administrator = Administrador

## Fuentes y orígenes

dados-fonte-cvm_fca = CVM · FCA
dados-fonte-cvm_fii = CVM · informe de FII
dados-fonte-cvm_registro = CVM · registro de fondos
dados-fonte-sec = SEC EDGAR
dados-fonte-b3 = B3
dados-fonte-brapi = brapi
dados-fonte-fundamentus = Fundamentus
dados-fonte-outra = Otra fuente
dados-origem-user = tú
dados-origem-legacy = ya completado
dados-origem-legacy-dica = Completado antes de esta función, sin fuente registrada: se trata como tuyo.
dados-origem-import = importación
dados-origem-enrichment = sugerencia aprobada
dados-origem-search = búsqueda del registro
dados-faixa-high = alta
dados-faixa-medium = media
dados-faixa-low = baja
dados-confianca = { $faixa } { $valor }
dados-nivel-1 = oficial
dados-nivel-2 = emisor
dados-nivel-3 = secundaria
dados-nivel-4 = IA
dados-achado-pelo-nome = encontrado por el nombre, no por el CNPJ
dados-concordam = también: { $fontes }

## Estados

dados-estado-fill = Vacío
dados-estado-update = La fuente cambió
dados-estado-diverge = Distinto del tuyo
dados-estado-conflict = Conflicto
dados-estado-missing = Sin fuente
dados-estado-ok = Al día

## Salud de los datos (Cartera › Salud de los activos)

dados-titulo = Datos de los activos
dados-ajuda = Sector, segmento, ISIN, CNPJ y otros campos del registro sugeridos por las fuentes oficiales que Treasury ya descargó (CVM, SEC, registro de fondos) y por Fundamentus en caché. Nada se guarda sin que elijas; lo que escribiste nunca se cambia solo.
dados-resumo = { $ativos } activo(s) · { $sem_setor } sin sector · { $sem_isin } sin ISIN · { $conflitos } conflicto(s)
dados-filtro-todos = Todos
dados-filtro-vazios = Para completar
dados-filtro-conflitos = Conflictos
dados-filtro-divergencias = Distintos del tuyo
dados-col-ativo = Activo
dados-col-campo = Campo
dados-col-atual = Actual
dados-col-sugestao = Sugerencia
dados-col-fonte = Fuente
dados-col-confianca = Confianza
dados-escolha = elige una
dados-vazio = vacío
dados-nada = Ningún campo para completar: los datos de los activos coinciden con las fuentes de la bóveda.
dados-aplicar-marcados =
    { $n ->
        [one] Guardar marcados ({ $n } campo)
       *[other] Guardar marcados ({ $n } campos)
    }
dados-marcar-altos = Marcar los de confianza alta
dados-desmarcar = Desmarcar todos
dados-aplicado =
    { $n ->
        [one] { $n } campo guardado
       *[other] { $n } campos guardados
    }
dados-aplicado-desfazer = Puedes deshacerlo desde el historial de cambios.
dados-nao-aplicou = No se pudo guardar
dados-rodape = Los datos de fuentes externas y de IA pueden estar equivocados o desactualizados. Verifica la fuente de cada campo antes de aprobar.
dados-trocar-seu = Cambiar tu valor por este
dados-diverge-dica = La fuente dice "{ $valor }". Tu valor se mantiene hasta que lo cambies.

## Registro del activo

dados-cadastro-titulo = Datos del registro
dados-completar = Completar datos
dados-completar-dica = Guarda las sugerencias de confianza alta de los campos vacíos. Las demás quedan para que elijas campo por campo.
dados-completar-nada = Ninguna sugerencia de confianza alta para campos vacíos.
dados-usar = Usar
dados-travar = Bloquear campo
dados-destravar = Desbloquear campo
dados-travado = bloqueado
dados-travar-dica = No sugerir cambios en este campo.
dados-de-onde-veio = De dónde vino
dados-leituras = Lecturas de las fuentes en la bóveda
dados-sem-leituras = Ninguna fuente de la bóveda tiene este campo.
dados-historico = Guardado a partir de
dados-observado-em = leído el { $data }

## Fase 2: fuentes de la red

dados-campo-lei = LEI
dados-campo-website = Sitio web
dados-campo-activity = Actividad
dados-campo-cnpj_status = Situación del CNPJ
dados-fonte-gleif = GLEIF
dados-fonte-brasilapi = Receita Federal · BrasilAPI
dados-consultar = Consultar fuentes
dados-consultar-dica = Consulta a la B3 (sector en tres niveles, actividad, sitio web), a GLEIF (LEI) y a la Receita Federal por BrasilAPI (razón social y situación). Solo se envían el código CVM y el CNPJ; una fuente consultada hace menos de 30 días no se consulta de nuevo.
dados-consultar-todos = Consultar fuentes de todos
dados-consultado =
    { $n ->
        [one] { $n } lectura nueva de las fuentes
       *[other] { $n } lecturas nuevas de las fuentes
    }
dados-consultado-nada = Ninguna lectura nueva: las fuentes se consultaron hace poco, están desactivadas o no tienen este activo.
dados-consultados-todos = { $ativos } activo(s) revisado(s), { $n } lectura(s) nueva(s)
dados-consultados-erros =
    { $n ->
        [one] { $n } activo con error en alguna fuente; inténtalo más tarde.
       *[other] { $n } activos con error en alguna fuente; inténtalo más tarde.
    }
dados-nao-consultou = No se pudieron consultar las fuentes

## Atajos (alerta, aviso, ⌘K)

dados-cmd-completar = Completar datos de los activos
dados-cmd-sin = sector,segmento,isin,cnpj,datos,registro,completar,enriquecer
dados-aviso =
    { $n ->
        [one] { $n } dato de tus activos se puede completar con las fuentes oficiales
       *[other] { $n } datos de tus activos se pueden completar con las fuentes oficiales
    }{ $sem_setor ->
        [0] .
        [one] {" "}({ $sem_setor } activo sin sector).
       *[other] {" "}({ $sem_setor } activos sin sector).
    }
dados-aviso-ver = Ver datos
dados-aviso-fechar = Cerrar el aviso de los datos de los activos
dados-chip-ativo =
    { $n ->
        [one] { $n } dato para completar
       *[other] { $n } datos para completar
    }
dados-campo-pension_type = Tipo del plan
dados-valor-pension_type-efpc = Fondo de pensión (EFPC)
dados-valor-pension_type-pgbl = PGBL
dados-valor-pension_type-vgbl = VGBL
