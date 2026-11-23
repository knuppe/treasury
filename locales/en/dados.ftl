# Asset data with sources (enrichment, phase 1; docs/agent-enrichment.md): sector, segment,
# ISIN, CNPJ and other record fields completed from the sources already downloaded, with origin,
# confidence, conflicts and differences.
# Revision: AI draft, not yet reviewed by a native speaker.

## Fields

dados-campo-sector = Sector
dados-campo-subsector = Subsector
dados-campo-segment = Segment
dados-campo-isin = ISIN
dados-campo-cnpj = CNPJ
dados-campo-issuer_cnpj = Issuer CNPJ
dados-campo-issuer_name = Issuer
dados-campo-share_kind = Share type
dados-campo-listing_segment = Listing segment
dados-campo-cvm_code = CVM code
dados-campo-cik = CIK (SEC)
dados-campo-fii_mandate = FII mandate
dados-campo-fii_management = FII management
dados-campo-fund_class_cvm = CVM class
dados-campo-fund_class_anbima = ANBIMA class
dados-campo-benchmark = Benchmark
dados-campo-manager = Manager
dados-campo-administrator = Administrator

## Sources and origins

dados-fonte-cvm_fca = CVM · FCA
dados-fonte-cvm_fii = CVM · FII report
dados-fonte-cvm_registro = CVM · fund registry
dados-fonte-sec = SEC EDGAR
dados-fonte-b3 = B3
dados-fonte-brapi = brapi
dados-fonte-fundamentus = Fundamentus
dados-fonte-outra = Other source
dados-origem-user = you
dados-origem-legacy = already filled in
dados-origem-legacy-dica = Filled in before this feature, with no recorded source: treated as yours.
dados-origem-import = import
dados-origem-enrichment = approved suggestion
dados-origem-search = record search
dados-faixa-high = high
dados-faixa-medium = medium
dados-faixa-low = low
dados-confianca = { $faixa } { $valor }
dados-nivel-1 = official
dados-nivel-2 = issuer
dados-nivel-3 = secondary
dados-nivel-4 = AI
dados-achado-pelo-nome = matched by name, not by CNPJ
dados-concordam = also: { $fontes }

## States

dados-estado-fill = Empty
dados-estado-update = Source changed
dados-estado-diverge = Differs from yours
dados-estado-conflict = Conflict
dados-estado-missing = No source
dados-estado-ok = Up to date

## Data health (Portfolio › Asset health)

dados-titulo = Asset data
dados-ajuda = Sector, segment, ISIN, CNPJ and other record fields suggested by the official sources Treasury has already downloaded (CVM, SEC, fund registry) and by the cached Fundamentus pages. Nothing is saved until you choose; what you typed is never replaced on its own.
dados-resumo = { $ativos } asset(s) · { $sem_setor } without sector · { $sem_isin } without ISIN · { $conflitos } conflict(s)
dados-filtro-todos = All
dados-filtro-vazios = To fill in
dados-filtro-conflitos = Conflicts
dados-filtro-divergencias = Different from yours
dados-col-ativo = Asset
dados-col-campo = Field
dados-col-atual = Current
dados-col-sugestao = Suggestion
dados-col-fonte = Source
dados-col-confianca = Confidence
dados-escolha = pick one
dados-vazio = empty
dados-nada = No fields to fill in: the asset data matches the sources in the vault.
dados-aplicar-marcados =
    { $n ->
        [one] Save selected ({ $n } field)
       *[other] Save selected ({ $n } fields)
    }
dados-marcar-altos = Select high confidence
dados-desmarcar = Clear selection
dados-aplicado =
    { $n ->
        [one] { $n } field saved
       *[other] { $n } fields saved
    }
dados-aplicado-desfazer = You can undo it from the change history.
dados-nao-aplicou = Could not save
dados-rodape = Data from external sources and AI may be wrong or outdated. Check each field's source before approving.
dados-trocar-seu = Replace your value with this one
dados-diverge-dica = The source says "{ $valor }". Your value stays until you replace it.

## Asset record

dados-cadastro-titulo = Record data
dados-completar = Fill in data
dados-completar-dica = Saves the high-confidence suggestions for empty fields. The others are left for you to pick field by field.
dados-completar-nada = No high-confidence suggestions for empty fields.
dados-usar = Use
dados-travar = Lock field
dados-destravar = Unlock field
dados-travado = locked
dados-travar-dica = Don't suggest changes to this field.
dados-de-onde-veio = Where it came from
dados-leituras = Source readings in the vault
dados-sem-leituras = No source in the vault has this field.
dados-historico = Saved from
dados-observado-em = read on { $data }

## Phase 2: network sources

dados-campo-lei = LEI
dados-campo-website = Website
dados-campo-activity = Activity
dados-campo-cnpj_status = CNPJ status
dados-fonte-gleif = GLEIF
dados-fonte-brasilapi = Federal Revenue · BrasilAPI
dados-consultar = Check sources
dados-consultar-dica = Asks B3 (sector in three levels, activity, website), GLEIF (LEI) and the Federal Revenue through BrasilAPI (legal name and status). Only the CVM code and the CNPJ are sent; a source checked in the last 30 days is not asked again.
dados-consultar-todos = Check sources for all
dados-consultado = { $n } new reading(s) from the sources
dados-consultado-nada = No new readings: the sources were checked recently, are turned off or don't have this asset.
dados-consultados-todos = { $ativos } asset(s) checked, { $n } new reading(s)
dados-consultados-erros =
    { $n ->
        [one] { $n } asset with an error in some source; try again later.
       *[other] { $n } assets with an error in some source; try again later.
    }
dados-nao-consultou = Could not check the sources

## Shortcuts (alert, notice, ⌘K)

dados-cmd-completar = Fill in asset data
dados-cmd-sin = sector,segment,isin,cnpj,data,record,fill,enrich
dados-aviso =
    { $n ->
        [one] { $n } piece of your asset data can be filled in from official sources
       *[other] { $n } pieces of your asset data can be filled in from official sources
    }{ $sem_setor ->
        [0] .
        [one] {" "}({ $sem_setor } asset without a sector).
       *[other] {" "}({ $sem_setor } assets without a sector).
    }
dados-aviso-ver = See data
dados-aviso-fechar = Close the asset data notice
dados-chip-ativo =
    { $n ->
        [one] { $n } field to fill in
       *[other] { $n } fields to fill in
    }
dados-campo-pension_type = Plan type
dados-valor-pension_type-efpc = Pension fund (EFPC)
dados-valor-pension_type-pgbl = PGBL
dados-valor-pension_type-vgbl = VGBL
