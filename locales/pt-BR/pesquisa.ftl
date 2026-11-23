# Análise de ativos: indicadores, preço calculado, histórico, documentos, rankings, screener,
# comparador e dados da análise (docs/i18n-design.md). pt-BR é o idioma de origem.
# Nomes e fórmulas dos indicadores vêm do núcleo e não passam por aqui.

## Seção da análise e análise por ticker

pesquisa-analise-fundamentalista = Análise fundamentalista
pesquisa-analise-do-ativo = Análise do ativo
pesquisa-aba-indicadores = Indicadores
pesquisa-aba-preco-justo = Preço calculado
pesquisa-aba-historico = Histórico
pesquisa-aba-documentos = Documentos
pesquisa-analisar-papel = Analisar papel
pesquisa-ticker-para-analisar = Ticker para analisar
pesquisa-ticker-exemplo-analise = Ticker (ex.: PETR4, HGLG11, AAPL)
pesquisa-analisar = Analisar
pesquisa-n-papeis-universo = { $n } papéis no universo
pesquisa-abrir-pagina-ativo = Abrir a página do ativo (posição e operações)
pesquisa-escolha-papel = Escolha um papel
pesquisa-escolha-papel-texto = Digite o ticker de qualquer ação ou FII do universo baixado (cadastrado ou não) para ver indicadores, preço calculado, histórico e fatos relevantes.

## Universo vazio e ativo sem dados

pesquisa-sem-coleta = Sem coleta
pesquisa-sem-universo = Nenhum papel no universo ainda
pesquisa-sem-universo-texto = { $quem ->
    [ticker] A análise por ticker rodam sobre o universo de ações e FIIs baixado da CVM, da B3 e da SEC. Rode a primeira coleta (cerca de 230 MB, 1 a 2 minutos) em Dados da análise; depois tudo funciona offline.
    [screener] O screener rodam sobre o universo de ações e FIIs baixado da CVM, da B3 e da SEC. Rode a primeira coleta (cerca de 230 MB, 1 a 2 minutos) em Dados da análise; depois tudo funciona offline.
    [rankings] Os rankings rodam sobre o universo de ações e FIIs baixado da CVM, da B3 e da SEC. Rode a primeira coleta (cerca de 230 MB, 1 a 2 minutos) em Dados da análise; depois tudo funciona offline.
   *[geral] Rankings e screener rodam sobre o universo de ações e FIIs baixado da CVM, da B3 e da SEC. Rode a primeira coleta (cerca de 230 MB, 1 a 2 minutos) em Dados da análise; depois tudo funciona offline.
}
pesquisa-ir-dados = Ir para Dados da análise
pesquisa-dados-analise = Dados da análise
pesquisa-sem-dados-ativo = Sem dados da análise para este ativo
pesquisa-sem-dados-ativo-texto = { $erro }. Os fundamentos vêm da coleta da CVM, da B3 e da SEC.

## Grupos de indicadores

pesquisa-grupo-mercado = Mercado
pesquisa-grupo-valuation = Valuation
pesquisa-grupo-rentabilidade = Rentabilidade
pesquisa-grupo-endividamento = Endividamento
pesquisa-grupo-crescimento = Crescimento
pesquisa-grupo-por-acao = Por ação
pesquisa-grupo-dividendos = Dividendos
pesquisa-grupo-preco-justo = Preço calculado
pesquisa-grupo-fii = Fundo imobiliário
pesquisa-grupo-bancos = Bancos
pesquisa-grupo-score = Score do Treasury

## Operadores e campos do screener

pesquisa-logica-e = E
pesquisa-logica-ou = OU
pesquisa-op-entre = entre
pesquisa-op-um-de = é um de
pesquisa-op-nao-um-de = não é um de
pesquisa-op-contem = contém
pesquisa-op-disponivel = disponível
pesquisa-op-indisponivel = indisponível
pesquisa-campo-ticker = Ticker
pesquisa-campo-nome = Nome
pesquisa-campo-classe = Classe
pesquisa-campo-setor = Setor
pesquisa-campo-segmento = Segmento
pesquisa-campo-moeda = Moeda
pesquisa-campo-faixa-score = Faixa do score (robusto, saudavel, atencao, risco)
pesquisa-campo-red-flags = Red flags do score
pesquisa-campo-na-carteira = Na carteira
pesquisa-campo-na-watchlist = Na watchlist

## Fontes e situação da coleta

pesquisa-fonte-fca = CVM FCA · cadastro e tickers
pesquisa-fonte-dfp = CVM DFP · demonstrações anuais
pesquisa-fonte-itr = CVM ITR · trimestres
pesquisa-fonte-fii = CVM · informe mensal de FII
pesquisa-fonte-fii-trimestral = CVM · informe trimestral de FII
pesquisa-fonte-ipe = CVM IPE · fatos relevantes
pesquisa-fonte-b3 = B3 COTAHIST · cotações
pesquisa-fonte-b3-listados = B3 empresas e fundos listados · proventos oficiais
pesquisa-fonte-sec = SEC EDGAR · empresas dos EUA
pesquisa-fonte-bacen = Banco Central IF.data · bancos
pesquisa-fonte-fundos = CVM e B3 · cadastro de fundos e ETFs
pesquisa-fonte-cripto = CoinGecko e DefiLlama · criptoativos
pesquisa-fonte-brapi = brapi.dev · conferência (opcional)
pesquisa-situacao-atualizado = Atualizado
pesquisa-situacao-sem-mudanca = Sem mudança
pesquisa-situacao-inexistente = Inexistente
pesquisa-situacao-pulado = Pulado
pesquisa-situacao-erro = Erro

## Valores compactos (o número já vem formatado, com a moeda)

pesquisa-compacto-tri = { $valor } tri
pesquisa-compacto-bi = { $valor } bi
pesquisa-compacto-mi = { $valor } mi
pesquisa-compacto-mil = { $valor } mil

## Indicadores

pesquisa-formula = Fórmula
pesquisa-indisponivel = Indisponível
pesquisa-periodo = Período
pesquisa-fonte = Fonte
pesquisa-status = Status
pesquisa-estimado = ≈ Estimado
pesquisa-conferencia = Conferência dos dados
pesquisa-conferencia-conferido = Dados de entrada conferidos
pesquisa-conferencia-corrigido = Corrigido com outra fonte oficial
pesquisa-conferencia-aviso = Com aviso
pesquisa-conferencia-bloqueado = Bloqueado: dado de entrada inválido
pesquisa-conferencia-marca-corrigido = { $indicador }: corrigido com outra fonte oficial
pesquisa-conferencia-marca-aviso = { $indicador }: com aviso sobre os dados
pesquisa-indicador-indisponivel = { $indicador }: indisponível
pesquisa-indisponivel-motivo = Indisponível: { $motivo }
pesquisa-calculando-indicadores = Calculando indicadores
pesquisa-cotacao = Cotação
pesquisa-cotacao-anterior = { $cotacao } (anterior)
pesquisa-sem-cotacao = sem cotação
pesquisa-fluxos = Fluxos
pesquisa-fluxos-12-meses = 12 meses até { $data }
pesquisa-informe-mensal = informe mensal
pesquisa-calculado-em = Calculado em
pesquisa-notas-calculo = Notas do cálculo
pesquisa-indicadores-grupo = Indicadores · { $grupo }
pesquisa-indicadores-dica = Passe o mouse ou clique no nome de um indicador para ver fórmula, período e fonte. Lucro e patrimônio são os dos controladores; fluxos dos últimos 12 meses (DFP + ITR).

## Preço calculado

pesquisa-preco-justo-metodo = Preço calculado · { $metodo }
pesquisa-preco-justo-de = Preço calculado ({ $metodo })
pesquisa-margem-cotacao = Margem até a cotação
pesquisa-cotacao-valor = cotação { $valor }
pesquisa-premissas = Premissas · { $metodo }
pesquisa-premissa = Premissa
pesquisa-valor = Valor
pesquisa-nao-recomendacao = Não é recomendação de investimento
pesquisa-aviso-estimativas = Estimativas a partir de fórmulas clássicas e dos dados informados; não são recomendação de investimento.
pesquisa-taxa-minima-bazin = Taxa mínima (Bazin)
pesquisa-taxa-invalida = Informe uma taxa entre 0 e 100%.
pesquisa-padrao-ajustes = padrão dos ajustes
pesquisa-media-dividendos = Média de dividendos
pesquisa-n-anos = { $n ->
    [1] { $n } ano
   *[other] { $n } anos
}
pesquisa-usar-padrao = Usar o padrão
pesquisa-calculando-preco-justo = Calculando o preço
pesquisa-cotacao-em = { $preco } em { $data } · { $fonte }

## Documentos

pesquisa-doc-entregue = Entregue
pesquisa-doc-categoria = Categoria
pesquisa-doc-assunto = Assunto
pesquisa-doc-documento = Documento
pesquisa-doc-sem-assunto = (sem assunto informado)
pesquisa-doc-abrir-cvm-rotulo = Abrir na CVM: { $categoria } de { $data }
pesquisa-doc-abrir-cvm = Abrir na CVM ↗
pesquisa-doc-categorias = Categoria dos documentos
pesquisa-doc-fatos = Fatos relevantes
pesquisa-doc-comunicados = Comunicados
pesquisa-doc-avisos = Avisos
pesquisa-doc-periodo = Últimos 12 meses · CVM IPE
pesquisa-doc-vazio = Nenhum documento no período
pesquisa-doc-vazio-texto = Nada nas categorias e no período escolhidos.
pesquisa-doc-tabela = Documentos da companhia
pesquisa-doc-rodape = Assunto e resumo são textos da própria companhia, mostrados como estão. O documento oficial abre no site da CVM.

## Histórico

pesquisa-hist-receita-liquida = Receita líquida
pesquisa-hist-lucro-liquido = Lucro líquido
pesquisa-hist-margem-ebitda = Margem EBITDA
pesquisa-hist-margem-liquida = Margem líquida
pesquisa-hist-roe = ROE
pesquisa-hist-dividendos-acao = Dividendos por ação
pesquisa-hist-rendimento-cota = Rendimento por cota
pesquisa-hist-vp-cota = Valor patrimonial por cota
pesquisa-hist-carregando = Carregando histórico
pesquisa-hist-vazio = Sem demonstrações baixadas para este ativo
pesquisa-hist-vazio-texto = O histórico vem das DFPs da CVM (ou do XBRL da SEC). Rode a coleta em Dados da análise.
pesquisa-hist-resultado-distribuivel = Resultado distribuível por exercício
pesquisa-hist-receita-lucro = Receita e lucro por exercício
pesquisa-hist-fonte-fii = Fonte: informes mensais da CVM · lucro dos controladores · { $moeda }
pesquisa-hist-fonte-dfp = Fonte: DFP da CVM / 10-K da SEC · lucro dos controladores · { $moeda }
pesquisa-hist-receita-lucro-rotulo = Receita e lucro líquido de { $nome } por exercício, { $periodo }.
pesquisa-hist-margens = Margens e ROE
pesquisa-hist-margens-fonte = Margem = resultado ÷ receita líquida; ROE = lucro ÷ patrimônio líquido no fim do exercício
pesquisa-hist-margens-rotulo = Margens e ROE de { $nome } por exercício, { $periodo }.
pesquisa-hist-margens-tabela = Margens e ROE por exercício
pesquisa-hist-dividendos = Dividendos por ação por ano
pesquisa-hist-dividendos-fonte = Proventos por ação com data-com no ano (agenda); sem agenda, os declarados na DFP
pesquisa-hist-dividendos-rotulo = Dividendos por ação de { $nome } por ano, { $periodo }.
pesquisa-hist-rendimento = Rendimento por cota, 24 meses
pesquisa-hist-fonte-informe-fii = Fonte: informe mensal de FII (CVM)
pesquisa-hist-rendimento-rotulo = Rendimento mensal por cota de { $nome }, 24 meses.
pesquisa-hist-rendimento-tabela = Rendimento por cota por mês
pesquisa-hist-vp = Valor patrimonial por cota, 24 meses
pesquisa-hist-vp-rotulo = Valor patrimonial por cota de { $nome }, 24 meses.
pesquisa-hist-vp-tabela = Valor patrimonial por cota por mês

## Comparador

pesquisa-cmp-tabela = Comparação de indicadores
pesquisa-cmp-indicador = Indicador
pesquisa-cmp-melhor = (melhor)
pesquisa-cmp-sem-historico = Sem histórico de cotações para comparar
pesquisa-cmp-sem-historico-texto = A cotação normalizada usa o histórico dos ativos cadastrados (Yahoo Finance). Papéis só do universo têm cotação de poucos pregões.
pesquisa-cmp-cotacao-normalizada = Cotação normalizada (base 100), 12 meses
pesquisa-cmp-fonte = Base 100 em { $data }, primeiro pregão com cotação de todos · fechamento sem ajuste por proventos
pesquisa-cmp-fonte-faltando = { $fonte } · sem histórico: { $tickers }
pesquisa-cmp-grafico-rotulo = Cotação normalizada em base 100 de { $tickers } desde { $data }.
pesquisa-cmp-tabela-mensal = Cotação normalizada por mês
pesquisa-cmp-data = Data
pesquisa-cmp-ja-esta = { $ticker } já está na comparação.
pesquisa-cmp-maximo = Compare no máximo { $n } ativos.
pesquisa-cmp-fora-universo = { $ticker } não está no universo baixado.
pesquisa-cmp-adicionar-rotulo = Adicionar à comparação
pesquisa-cmp-ticker = Ticker para comparar
pesquisa-cmp-ticker-exemplo = Ticker (ex.: VALE3)
pesquisa-cmp-adicionar = Adicionar
pesquisa-cmp-papeis = Papéis comparados
pesquisa-cmp-tirar = Tirar { $ticker } da comparação
pesquisa-cmp-escolha = Escolha de 2 a 6 papéis
pesquisa-cmp-escolha-texto = Ações, FIIs e ações americanas do universo, lado a lado: indicadores com o melhor de cada linha destacado e a cotação normalizada em base 100.
pesquisa-cmp-exemplo = Exemplo: PETR4 × VALE3 × WEGE3
pesquisa-cmp-comparando = Comparando
pesquisa-cmp-rodape = ◆ melhor valor da linha pela ordem natural do indicador (menor P/L, maior DY…); linhas sem ordem natural (cotação, valor da firma) não têm destaque. Valores na moeda de cada papel. Calculado em { $data }. Não é recomendação de investimento.

## Recorte do universo

pesquisa-recorte = Recorte do universo
pesquisa-classes = Classes
pesquisa-so-carteira = Só carteira e watchlist
pesquisa-setor-contem = Setor contém
pesquisa-varios-virgula = Vários: separe por vírgula
pesquisa-setor-exemplo = ex.: energia, financeiro
pesquisa-segmento-contem = Segmento contém
pesquisa-segmento-dica = FIIs: logística, shoppings, papel…
pesquisa-segmento-exemplo = ex.: bancos, logística
pesquisa-liquidez-minima = Liquidez diária mínima
pesquisa-liquidez-dica = Na moeda do ativo (R$ na B3, USD nos EUA)
pesquisa-liquidez-invalida = Informe um valor (ex.: 1.000.000).

## Montador de condições

pesquisa-tipo-texto = Texto
pesquisa-tipo-sim-nao = Sim/não
pesquisa-logica-todas = Todas (E)
pesquisa-logica-todas-dica = O papel precisa passar em todas
pesquisa-logica-qualquer = Qualquer (OU)
pesquisa-logica-qualquer-dica = Basta passar em uma
pesquisa-cond-n-grupo = { $n } do grupo { $grupo }
pesquisa-cond-campo = Campo da condição { $n }
pesquisa-cond-operador = Operador da condição { $n }
pesquisa-cond-valor = Valor da condição { $n }
pesquisa-cond-minimo = Mínimo da condição { $n }
pesquisa-cond-maximo = Máximo da condição { $n }
pesquisa-cond-remover = Remover condição { $n }
pesquisa-cond-classe = Classe
pesquisa-cond-min = mín.
pesquisa-cond-max = máx.
pesquisa-cond-lista = a, b, c
pesquisa-cond-texto = texto
pesquisa-cond-e = e
pesquisa-combinar = Combinar
pesquisa-combinar-tudo = Combinar condições e grupos
pesquisa-sem-condicoes = Sem condições: todos os papéis do recorte.
pesquisa-condicoes = Condições
pesquisa-grupo-n = Grupo { $n }
pesquisa-combinar-grupo = Combinar condições do grupo { $n }
pesquisa-remover-grupo = Remover grupo
pesquisa-condicoes-grupo = Condições do grupo { $n }
pesquisa-condicao-no-grupo = Condição no grupo
pesquisa-adicionar-condicao = Adicionar condição
pesquisa-adicionar-grupo = Adicionar grupo (OU…)

## Erros dos filtros

pesquisa-erro-operador = Operador não vale para este campo.
pesquisa-erro-lista = Informe um ou mais valores separados por vírgula.
pesquisa-erro-texto = Informe o texto.
pesquisa-erro-numero = Informe um número (ex.: 10 ou 6,5).
pesquisa-erro-maximo = Informe o máximo.
pesquisa-erro-maximo-menor = O máximo é menor que o mínimo.
pesquisa-erro-grupo-vazio = Grupo vazio: adicione uma condição.

## Screener

pesquisa-scr-salvar-titulo = Salvar tela do screener
pesquisa-scr-salvar-texto = A tela guarda os filtros e o recorte; rode de novo quando quiser (os dados do dia mudam).
pesquisa-scr-salvar-rotulo = Salvar tela
pesquisa-scr-nome = Nome
pesquisa-scr-informe-nome = Informe o nome.
pesquisa-scr-descricao = Descrição (opcional)
pesquisa-scr-salvar = Salvar tela
pesquisa-scr-colunas = Colunas do resultado ({ $n })
pesquisa-scr-tela-salva = Tela “{ $nome }” salva
pesquisa-scr-tela-atualizada = Tela “{ $nome }” atualizada
pesquisa-scr-tela-excluida = Tela “{ $nome }” excluída
pesquisa-scr-erro-excluir = Não foi possível excluir
pesquisa-scr-desfeita = Alteração desfeita
pesquisa-scr-erro-desfazer = Não foi possível desfazer
pesquisa-scr-ticker = Ticker
pesquisa-scr-nome-col = Nome
pesquisa-scr-universo-vazio = O universo ainda está vazio
pesquisa-scr-universo-vazio-texto = Dá para montar e salvar telas agora; os resultados aparecem depois da primeira coleta em Dados da análise.
pesquisa-scr-tela = Tela · { $nome }
pesquisa-scr-filtros = Filtros
pesquisa-scr-telas-salvas = Telas salvas
pesquisa-scr-abrir-tela = Abrir tela salva…
pesquisa-scr-nenhuma-tela = Nenhuma tela salva
pesquisa-scr-salva-em = Salva: { $resumo } · alterada em { $data }
pesquisa-ordenar-por = Ordenar por
pesquisa-scr-ordem = Ordem do resultado
pesquisa-ordem-natural = Natural
pesquisa-maior-primeiro = Maior primeiro
pesquisa-menor-primeiro = Menor primeiro
pesquisa-scr-rodar = Rodar screener
pesquisa-scr-salvar-alteracoes = Salvar alterações
pesquisa-scr-salvar-nova = Salvar como nova…
pesquisa-scr-salvar-tela = Salvar tela…
pesquisa-scr-nova = Nova tela
pesquisa-scr-excluir = Excluir tela
pesquisa-scr-excluir-texto = A tela “{ $nome }” sai da lista. Dá para desfazer logo depois ou em Ajustes › Histórico de alterações.
pesquisa-scr-corrija = Corrija os filtros marcados.
pesquisa-scr-desfazer = Desfazer
pesquisa-scr-desfazer-texto = Também dá para desfazer em Ajustes › Histórico de alterações.
pesquisa-scr-instrucoes = Monte os filtros e clique em { $botao }. Percentuais em % (DY ≥ 6), múltiplos em vezes (P/L < 12), dinheiro na moeda do ativo.
pesquisa-scr-rodando = Rodando o screener
pesquisa-scr-resultado = Resultado do screener
pesquisa-scr-n-papeis = { $n ->
    [1] { $n } papel
   *[other] { $n } papéis
}
pesquisa-universo-de = universo de { $data }
pesquisa-scr-nenhum = Nenhum papel passou nos filtros.
pesquisa-scr-mostrando = Mostrando os { $n } primeiros; refine os filtros para ver o resto.

## Rankings

pesquisa-rank-maiores-dy = Maiores DY
pesquisa-rank-menor-pl = Menor P/L
pesquisa-rank-margem-graham = Margem até Graham
pesquisa-rank-margem-bazin = Margem até Bazin
pesquisa-rank-sem-prejuizo = Nunca teve prejuízo
pesquisa-rank-maior-roe = Maior ROE
pesquisa-rank-valor-mercado = Valor de mercado
pesquisa-rank-liquidez-fii = Liquidez de FII
pesquisa-rank-pvp-fii = P/VP de FII
pesquisa-rank-menor-primeiro = menor primeiro
pesquisa-rank-maior-primeiro = maior primeiro
pesquisa-rank-nome = Nome
pesquisa-rank-liquidez = Liquidez diária
pesquisa-rank-prontos = Rankings prontos
pesquisa-rank-indicador = Indicador
pesquisa-rank-ordem = Ordem
pesquisa-rank-ordem-rotulo = Ordem do ranking
pesquisa-rank-mostrar = Mostrar
pesquisa-rank-ordem-natural = ordem natural: { $ordem }
pesquisa-rank-montando = Montando ranking
pesquisa-rank-tabela = Ranking · { $indicador }
pesquisa-rank-vazio = Nenhum papel do recorte tem este indicador.
pesquisa-rank-total = { $n } papéis com o indicador
pesquisa-rank-sem-valor = { $n } sem valor
pesquisa-rank-sem-valor-multiplo = { $n } sem valor (ou ≤ 0, que não conta como barato)
pesquisa-rank-rodape = universo de { $data } · moeda de cada papel (B3 em R$, EUA em USD). Não é recomendação de investimento.

## Dados da análise

pesquisa-dados-coleta-fonte = Coleta por fonte
pesquisa-dados-fonte = Fonte
pesquisa-dados-arquivos = Arquivos
pesquisa-dados-situacao = Situação
pesquisa-dados-baixado = Baixado
pesquisa-dados-linhas = Linhas
pesquisa-dados-com-problema = { $n } com problema
pesquisa-dados-n-atualizados = { $n ->
    [1] { $n } atualizado
   *[other] { $n } atualizados
}
pesquisa-dados-n-ok = { $n } ok
pesquisa-dados-n-pulados = { $n } pulado(s), sem problema
pesquisa-dados-total = Total
pesquisa-dados-aviso = Aviso da coleta
pesquisa-dados-concluida = Coleta da análise concluída
pesquisa-dados-concluida-avisos = Coleta concluída com avisos
pesquisa-dados-concluida-texto = { $tamanho } baixados · { $n } papéis no universo
pesquisa-dados-coleta-titulo = Coleta da análise de ativos
pesquisa-dados-coletando-live = COLETANDO
pesquisa-dados-fontes = Fontes
pesquisa-dados-primeira-coleta = Primeira coleta
pesquisa-dados-ultima-coleta = Última coleta
pesquisa-dados-nunca = nunca
pesquisa-dados-universo = Universo
pesquisa-dados-universo-valor = { $n } papéis · { $data }
pesquisa-dados-vazio = vazio
pesquisa-dados-emissores = Emissores
pesquisa-dados-papeis = Papéis
pesquisa-dados-fatos = Fatos
pesquisa-dados-documentos = Documentos
pesquisa-dados-cotacoes-ate = Cotações até
pesquisa-dados-coletando = Coletando…
pesquisa-dados-coletar-agora = Coletar agora
pesquisa-dados-baixando = Baixando e calculando
pesquisa-dados-primeira-leva = a primeira leva 1 a 2 minutos
pesquisa-dados-so-mudou = só o que mudou
pesquisa-dados-falhou = A coleta falhou
pesquisa-dados-ultima-por-fonte = Última coleta por fonte
pesquisa-dados-resumo = { $qual ->
    [esta] Esta coleta
   *[ultima] Última coleta
}: { $tamanho } em { $segundos } s · { $universo } papéis no universo · { $radar } proventos projetados pelo radar
pesquisa-dados-coletar-titulo = Coletar dados da análise
pesquisa-dados-coletar-primeira = A primeira coleta baixa cerca de 230 MB da CVM (DFP, ITR, FCA, FII, IPE), da B3 (COTAHIST) e da SEC e leva 1 a 2 minutos. Nada fica em disco além do banco cifrado.
pesquisa-dados-coletar-incremental = Baixa só os arquivos que mudaram desde a última coleta (normalmente poucos MB) e recalcula o universo e o radar de dividendos.
pesquisa-dados-baixar-230 = Baixar ~230 MB
pesquisa-dados-coletar = Coletar
pesquisa-dados-baixar-tudo = Baixar tudo de novo
pesquisa-dados-baixar-tudo-dica = Ignora o que não mudou (~230 MB de novo). Só para corrigir dados.
pesquisa-dados-ajustes-salvos = Ajustes da análise salvos
pesquisa-dados-ajustes = Ajustes da análise
pesquisa-dados-sec-email = Configure o e-mail da SEC
pesquisa-dados-sec-ua = User-Agent da SEC EDGAR
pesquisa-dados-sec-ua-dica = Nome do app e um e-mail de contato real; a SEC bloqueia pedidos sem identificação.
pesquisa-dados-sec-ua-erro = Inclua um e-mail (ex.: Treasury/0.1 seu.nome@email.com).
pesquisa-dados-sec-ua-exemplo = Treasury/0.1 seu.nome@email.com
pesquisa-dados-taxa-invalida = Entre 0 e 100%.
pesquisa-dados-bazin-dica = Premissas padrão do preço-teto de Bazin (média dos dividendos por ação ÷ taxa mínima); dá para mudar caso a caso na aba Preço calculado do ativo.
pesquisa-dados-descartar = Descartar
pesquisa-dados-salvar-ajustes = Salvar ajustes
pesquisa-doc-assembleias = Assembleias e reuniões
pesquisa-doc-janela = Período dos documentos
pesquisa-doc-12-meses = 12 meses
pesquisa-doc-tudo-coletado = Tudo coletado
pesquisa-doc-ultimo = O último documento guardado é de { $data }.
pesquisa-doc-ampliar = Ver desde { $data }
pesquisa-doc-sem-fonte = Sem fonte de documentos para esta classe
pesquisa-doc-sem-fonte-texto = Fatos relevantes e informes de FIIs, ETFs e fundos são publicados no Fundos.NET (B3), que o app ainda não coleta. Companhias abertas vêm do IPE da CVM.
pesquisa-doc-sem-emissor = Não encontramos o emissor deste ativo
pesquisa-doc-sem-emissor-texto = O ativo não tem CNPJ no cadastro e o ticker não está no cadastro de companhias da CVM (FCA). Informe o CNPJ da companhia no ativo para trazer os documentos.
pesquisa-doc-informar-cnpj = Informar o CNPJ
pesquisa-doc-sem-coleta = A coleta da CVM ainda não rodou
pesquisa-doc-sem-coleta-texto = Rode a coleta em Dados da análise para trazer os documentos das companhias.
pesquisa-doc-quieta = Nenhum documento desde { $data }
pesquisa-doc-quieta-texto = A companhia não publicou fato relevante, comunicado, aviso, assembleia nem reunião nas categorias guardadas desde essa data.
pesquisa-doc-diagnostico = Diagnóstico dos documentos
pesquisa-doc-diag-fonte = Fonte
pesquisa-doc-diag-fonte-ipe = IPE da CVM (companhias abertas)
pesquisa-doc-diag-emissor = Emissor
pesquisa-doc-diag-sem-emissor = não encontrado
pesquisa-doc-diag-coleta = Coleta
pesquisa-doc-diag-coleta-em = { $data }, guardando desde { $desde }
pesquisa-doc-diag-sem-coleta = ainda não rodou
pesquisa-doc-diag-guardados = Guardados
pesquisa-doc-diag-nenhum = nenhum
pesquisa-doc-diag-categorias = Categorias guardadas
pesquisa-nao-tem-ativo = Você não tem este ativo. Observar coloca o papel na watchlist, sem operação, e liga o score com histórico, os alertas e o Estudo com IA.
pesquisa-observado-pelo-mercado = Observado pelo Mercado: deixar de observar tira o ativo do cadastro se ele nunca tiver tido operação.
pesquisa-observar = Observar
pesquisa-deixar-observar = Deixar de observar
pesquisa-observando = Observando { $ticker }
pesquisa-observando-texto = Está na watchlist, sem operação.
pesquisa-deixou-observar = Deixou de observar
pesquisa-observar-erro = Não deu para observar
pesquisa-scr-metodos = Métodos
pesquisa-scr-avisar = Avisar quando um papel novo entrar
pesquisa-tags = Características
pesquisa-tags-ajuda = Marque as que o papel precisa ter (todas juntas). A regra aparece ao passar o mouse.
pesquisa-tags-limpar = Limpar características
pesquisa-tag-chip = { $tag } · { $n }
pesquisa-tag-vazia = { $regra }
    Nenhum papel do recorte atual tem esta característica.
pesquisa-tags-mais = { $tags } e mais { $n }
