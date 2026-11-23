# Dados dos ativos com fonte (enriquecimento, fase 1; docs/agent-enrichment.md): setor,
# segmento, ISIN, CNPJ e os demais campos do cadastro completados pelas fontes já baixadas,
# com origem, confiança, conflitos e divergências. pt-BR é o idioma de origem.

## Campos

dados-campo-sector = Setor
dados-campo-subsector = Subsetor
dados-campo-segment = Segmento
dados-campo-isin = ISIN
dados-campo-cnpj = CNPJ
dados-campo-issuer_cnpj = CNPJ do emissor
dados-campo-issuer_name = Emissor
dados-campo-share_kind = Tipo do papel
dados-campo-listing_segment = Segmento de listagem
dados-campo-cvm_code = Código CVM
dados-campo-cik = CIK (SEC)
dados-campo-fii_mandate = Mandato do FII
dados-campo-fii_management = Gestão do FII
dados-campo-fund_class_cvm = Classe CVM
dados-campo-fund_class_anbima = Classe ANBIMA
dados-campo-benchmark = Benchmark
dados-campo-manager = Gestor
dados-campo-administrator = Administrador

## Fontes e origens

dados-fonte-cvm_fca = CVM · FCA
dados-fonte-cvm_fii = CVM · informe de FII
dados-fonte-cvm_registro = CVM · registro de fundos
dados-fonte-sec = SEC EDGAR
dados-fonte-b3 = B3
dados-fonte-brapi = brapi
dados-fonte-fundamentus = Fundamentus
dados-fonte-outra = Outra fonte
dados-origem-user = você
dados-origem-legacy = já preenchido
dados-origem-legacy-dica = Preenchido antes desta função, sem fonte registrada: tratado como seu.
dados-origem-import = importação
dados-origem-enrichment = sugestão aprovada
dados-origem-search = busca do cadastro
dados-faixa-high = alta
dados-faixa-medium = média
dados-faixa-low = baixa
dados-confianca = { $faixa } { $valor }
dados-nivel-1 = oficial
dados-nivel-2 = emissor
dados-nivel-3 = secundária
dados-nivel-4 = IA
dados-achado-pelo-nome = achado pelo nome, não pelo CNPJ
dados-concordam = também: { $fontes }

## Estados

dados-estado-fill = Vazio
dados-estado-update = Fonte mudou
dados-estado-diverge = Diferente do seu
dados-estado-conflict = Conflito
dados-estado-missing = Sem fonte
dados-estado-ok = Em dia

## Saúde dos dados (Carteira › Saúde dos ativos)

dados-titulo = Dados dos ativos
dados-ajuda = Setor, segmento, ISIN, CNPJ e outros campos do cadastro sugeridos pelas fontes oficiais que o Treasury já baixou (CVM, SEC, registro de fundos) e pelo Fundamentus em cache. Nada é gravado sem você escolher; o que você digitou nunca é trocado sozinho.
dados-resumo = { $ativos } ativo(s) · { $sem_setor } sem setor · { $sem_isin } sem ISIN · { $conflitos } conflito(s)
dados-filtro-todos = Todos
dados-filtro-vazios = Para completar
dados-filtro-conflitos = Conflitos
dados-filtro-divergencias = Diferentes do seu
dados-col-ativo = Ativo
dados-col-campo = Campo
dados-col-atual = Atual
dados-col-sugestao = Sugestão
dados-col-fonte = Fonte
dados-col-confianca = Confiança
dados-escolha = escolha uma
dados-vazio = vazio
dados-nada = Nenhum campo para completar: os dados dos ativos batem com as fontes do cofre.
dados-aplicar-marcados =
    { $n ->
        [one] Gravar marcados ({ $n } campo)
       *[other] Gravar marcados ({ $n } campos)
    }
dados-marcar-altos = Marcar os de confiança alta
dados-desmarcar = Desmarcar todos
dados-aplicado =
    { $n ->
        [one] { $n } campo gravado
       *[other] { $n } campos gravados
    }
dados-aplicado-desfazer = Dá para desfazer pelo histórico de alterações.
dados-nao-aplicou = Não foi possível gravar
dados-rodape = Dados de fontes externas e de IA podem estar errados ou desatualizados. Confira a fonte de cada campo antes de aprovar.
dados-trocar-seu = Trocar o seu valor por este
dados-diverge-dica = A fonte diz "{ $valor }". O seu valor fica até você trocar.

## Cadastro do ativo

dados-cadastro-titulo = Dados do cadastro
dados-completar = Completar dados
dados-completar-dica = Grava as sugestões de confiança alta dos campos vazios. As outras ficam para você escolher campo a campo.
dados-completar-nada = Nenhuma sugestão de confiança alta para campos vazios.
dados-usar = Usar
dados-travar = Travar campo
dados-destravar = Destravar campo
dados-travado = travado
dados-travar-dica = Não sugerir mudanças neste campo.
dados-de-onde-veio = De onde veio
dados-leituras = Leituras das fontes no cofre
dados-sem-leituras = Nenhuma fonte no cofre tem este campo.
dados-historico = Gravado a partir de
dados-observado-em = lido em { $data }

## Fase 2: fontes da rede

dados-campo-lei = LEI
dados-campo-website = Site
dados-campo-activity = Atividade
dados-campo-cnpj_status = Situação do CNPJ
dados-fonte-gleif = GLEIF
dados-fonte-brasilapi = Receita Federal · BrasilAPI
dados-consultar = Consultar fontes
dados-consultar-dica = Pergunta à B3 (setor em três níveis, atividade, site), ao GLEIF (LEI) e à Receita pela BrasilAPI (razão social e situação). Só vão o código CVM e o CNPJ; uma fonte consultada há menos de 30 dias não é consultada de novo.
dados-consultar-todos = Consultar fontes de todos
dados-consultado =
    { $n ->
        [one] { $n } leitura nova das fontes
       *[other] { $n } leituras novas das fontes
    }
dados-consultado-nada = Nenhuma leitura nova: as fontes foram consultadas há pouco, estão desligadas ou não têm este ativo.
dados-consultados-todos = { $ativos } ativo(s) conferido(s), { $n } leitura(s) nova(s)
dados-consultados-erros =
    { $n ->
        [one] { $n } ativo com erro em alguma fonte; tente de novo mais tarde.
       *[other] { $n } ativos com erro em alguma fonte; tente de novo mais tarde.
    }
dados-nao-consultou = Não foi possível consultar as fontes

## Atalhos (alerta, aviso, ⌘K)

dados-cmd-completar = Completar dados dos ativos
dados-cmd-sin = setor,segmento,isin,cnpj,dados,cadastro,completar,enriquecer
dados-aviso =
    { $n ->
        [one] { $n } dado dos seus ativos pode ser completado pelas fontes oficiais
       *[other] { $n } dados dos seus ativos podem ser completados pelas fontes oficiais
    }{ $sem_setor ->
        [0] .
        [one] {" "}({ $sem_setor } ativo sem setor).
       *[other] {" "}({ $sem_setor } ativos sem setor).
    }
dados-aviso-ver = Ver dados
dados-aviso-fechar = Fechar o aviso dos dados dos ativos
dados-chip-ativo =
    { $n ->
        [one] { $n } dado para completar
       *[other] { $n } dados para completar
    }
dados-campo-pension_type = Tipo do plano
dados-valor-pension_type-efpc = Fundo de pensão (EFPC)
dados-valor-pension_type-pgbl = PGBL
dados-valor-pension_type-vgbl = VGBL
