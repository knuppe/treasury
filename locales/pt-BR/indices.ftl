# Índices e cotações: séries de índices, gráfico, câmbio, chave da FRED e relatório da coleta
# (docs/i18n-design.md). pt-BR é o idioma de origem.

## Para que serve cada série

indices-uso-ipca = Deflator padrão (BRL) e Tesouro IPCA+
indices-uso-igpm = Contratos e títulos indexados ao IGP-M
indices-uso-cdi = Renda fixa pós-fixada e benchmark
indices-uso-selic = Tesouro Selic e títulos da Selic
indices-uso-selic-meta = Meta definida pelo Copom
indices-uso-tr = Poupança e financiamentos
indices-uso-m2-br = Deflator alternativo: expansão monetária BR
indices-uso-us-cpi = Deflator de ativos em USD
indices-uso-us-m2 = Deflator alternativo: expansão monetária EUA
indices-uso-fed-funds = Juros básicos dos EUA
indices-uso-ibov = Benchmark de ações BR
indices-uso-sp500 = Benchmark de ações EUA
indices-uso-estr = Juros de referência da zona do euro
indices-uso-corra = Juros de referência do Canadá
indices-uso-sonia = Juros de referência do Reino Unido
indices-uso-boe-bank-rate = Juros básicos do Reino Unido
indices-uso-hicp-ea = Inflação da zona do euro
indices-uso-cpi-ca = Inflação do Canadá
indices-uso-cpi-gb = Inflação do Reino Unido
indices-uso-cpih-gb = Inflação do Reino Unido com moradia
indices-uso-cpi-us-oecd = Inflação dos EUA sem chave (alternativa à FRED)
indices-uso-inflacao-pais = Inflação do país
indices-uso-wb-cpi = Inflação anual (Banco Mundial), meses estimados
indices-uso-rate3m = Juros de 3 meses (referência)
indices-uso-fx = Câmbio de referência

## Tabela de séries

indices-freq-diaria = Diária
indices-freq-mensal = Mensal
indices-serie = Série
indices-unidade = Unidade
indices-freq = Freq.
indices-oficial-ate = Oficial até
indices-estimado-ate = Estimado até
indices-pontos = Pontos
indices-coletado-em = Coletado em
indices-series = Séries de índices
indices-figura-series = { $figura } · Séries de índices
indices-nenhuma-serie = Nenhuma série cadastrada.
indices-selecione = Selecione uma série (↑/↓) para ver o gráfico. Estimados aparecem em itálico com ≈ até o dado oficial sair.

## Gráfico de uma série

indices-oficial = Oficial
indices-estimado = Estimado
indices-titulo-meses = { $serie }, { $n } meses
indices-titulo-anos = { $serie }, { $n } anos
indices-periodo = Período da série
indices-periodo-5a = 5A
indices-fonte-unidade = Fonte: { $fonte } · { $unidade }
indices-nota-ipca = Meses ainda não divulgados usam a projeção Focus (ou o último dado) até o IBGE publicar.
indices-nota-wb = Inflação anual do Banco Mundial distribuída mês a mês: todos os pontos são estimados.
indices-sem-valores = Sem valores no período. Atualize os índices.
indices-grafico-aria = Série { $serie } no período, em { $unidade }.
indices-valores-de = Valores de { $serie }
indices-status = Status

## Relatório da coleta

indices-gravados = Gravados
indices-ultimo-oficial = Último oficial
indices-resultado = Resultado
indices-ok = ok
indices-cotacoes = Cotações
indices-desdobramentos = Desdobr.
indices-ultimo-pregao = Último pregão
indices-relatorio = Relatório da coleta
indices-coleta-concluida = Coleta concluída
indices-coleta-falhas = { $n ->
    [1] Coleta concluída com { $n } falha
   *[other] Coleta concluída com { $n } falhas
}
indices-coleta-resumo = { $inicio } → { $fim } · { $series } séries, { $moedas } moedas e { $ativos } ativos.
indices-ipca-estimado = { $n ->
    [1] {" "}IPCA com { $n } mês estimado (Focus).
   *[other] {" "}IPCA com { $n } meses estimados (Focus).
}
indices-indices-cambio = Índices e câmbio
indices-coleta-indices = Coleta de índices e câmbio
indices-coleta-cotacoes = Coleta de cotações
indices-sem-fonte = Nenhum ativo com operações ou na watchlist tem fonte automática de cotação.

## Chave da FRED

indices-fred = Chave da FRED
indices-fred-cadastrada = Chave da FRED cadastrada
indices-fred-removida = Chave da FRED removida
indices-trocar = Trocar
indices-fred-rotulo = Chave da FRED (EUA)
indices-fred-dica = Gratuita em fred.stlouisfed.org; necessária para US CPI, US M2 e Fed Funds. Fica cifrada no cofre e nunca volta para a interface.
indices-salvar-chave = Salvar chave
indices-sem-fred-titulo = Séries dos EUA sem chave da FRED
indices-sem-fred-texto = US CPI, US M2 e Fed Funds só são coletados com a chave gratuita da FRED; sem elas, o deflator US CPI e M2 EUA ficam sem dados.

## Coleta de dados de mercado

indices-coleta-titulo = Coleta de dados de mercado
indices-coletando = Coletando
indices-coletando-reticencias = Coletando…
indices-ultima-coleta = Última coleta
indices-rotina = Rotina
indices-rotina-valor = a cada 12 h com o app aberto
indices-coleta-ajuda = Índices do Banco Central (IPCA, CDI, Selic, IGP-M, TR, M2), câmbio PTAX, séries dos EUA na FRED e cotações de fechamento (Yahoo Finance para B3, EUA e cripto; Tesouro Transparente para o Tesouro Direto). Cada valor guarda a fonte e a data de coleta.
indices-atualizados = Índices, câmbio e cotações atualizados
indices-concluida-com-falhas = Coleta concluída com falhas
indices-coleta-falhou = A coleta falhou
indices-atualizar = Atualizar índices, câmbio e cotações
indices-pode-demorar = Pode levar alguns minutos
indices-cambio-fontes = Câmbio e fontes
indices-em-data = em { $data }
