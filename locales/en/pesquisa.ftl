# Asset research: indicators, calculated price, history, documents, rankings, screener, comparison
# and research data (docs/i18n-design.md). Indicator names and formulas come from the core.
# Revision: AI draft, not yet reviewed by a native speaker.

## Research section and analysis by ticker

pesquisa-analise-fundamentalista = Fundamental analysis
pesquisa-analise-do-ativo = Asset analysis
pesquisa-aba-indicadores = Indicators
pesquisa-aba-preco-justo = Calculated price
pesquisa-aba-historico = History
pesquisa-aba-documentos = Documents
pesquisa-analisar-papel = Analyze a security
pesquisa-ticker-para-analisar = Ticker to analyze
pesquisa-ticker-exemplo-analise = Ticker (e.g. PETR4, HGLG11, AAPL)
pesquisa-analisar = Analyze
pesquisa-n-papeis-universo = { $n ->
    [one] { $n } security in the universe
   *[other] { $n } securities in the universe
}
pesquisa-abrir-pagina-ativo = Open the asset page (position and trades)
pesquisa-escolha-papel = Pick a security
pesquisa-escolha-papel-texto = Type the ticker of any stock or REIT (FII) in the downloaded universe (registered or not) to see indicators, calculated price, history and material facts.

## Empty universe and asset without data

pesquisa-sem-coleta = No data collected
pesquisa-sem-universo = No securities in the universe yet
pesquisa-sem-universo-texto = { $quem ->
    [ticker] Analysis by ticker runs on the universe of stocks and FIIs downloaded from CVM, B3 and the SEC. Run the first collection (about 230 MB, 1 to 2 minutes) in Research data; after that everything works offline.
    [screener] The screener runs on the universe of stocks and FIIs downloaded from CVM, B3 and the SEC. Run the first collection (about 230 MB, 1 to 2 minutes) in Research data; after that everything works offline.
    [rankings] Rankings run on the universe of stocks and FIIs downloaded from CVM, B3 and the SEC. Run the first collection (about 230 MB, 1 to 2 minutes) in Research data; after that everything works offline.
   *[geral] Rankings and the screener run on the universe of stocks and FIIs downloaded from CVM, B3 and the SEC. Run the first collection (about 230 MB, 1 to 2 minutes) in Research data; after that everything works offline.
}
pesquisa-ir-dados = Go to Research data
pesquisa-dados-analise = Research data
pesquisa-sem-dados-ativo = No research data for this asset
pesquisa-sem-dados-ativo-texto = { $erro }. Fundamentals come from the CVM, B3 and SEC collection.

## Indicator groups

pesquisa-grupo-mercado = Market
pesquisa-grupo-valuation = Valuation
pesquisa-grupo-rentabilidade = Profitability
pesquisa-grupo-endividamento = Debt
pesquisa-grupo-crescimento = Growth
pesquisa-grupo-por-acao = Per share
pesquisa-grupo-dividendos = Dividends
pesquisa-grupo-preco-justo = Calculated price
pesquisa-grupo-fii = Real estate fund
pesquisa-grupo-bancos = Banks
pesquisa-grupo-score = Treasury score

## Screener operators and fields

pesquisa-logica-e = AND
pesquisa-logica-ou = OR
pesquisa-op-entre = between
pesquisa-op-um-de = is one of
pesquisa-op-nao-um-de = is not one of
pesquisa-op-contem = contains
pesquisa-op-disponivel = available
pesquisa-op-indisponivel = unavailable
pesquisa-campo-ticker = Ticker
pesquisa-campo-nome = Name
pesquisa-campo-classe = Class
pesquisa-campo-setor = Sector
pesquisa-campo-segmento = Segment
pesquisa-campo-moeda = Currency
pesquisa-campo-faixa-score = Score band (robusto, saudavel, atencao, risco)
pesquisa-campo-red-flags = Score red flags
pesquisa-campo-na-carteira = In portfolio
pesquisa-campo-na-watchlist = On watchlist

## Collection sources and status

pesquisa-fonte-fca = CVM FCA · registry and tickers
pesquisa-fonte-dfp = CVM DFP · annual statements
pesquisa-fonte-itr = CVM ITR · quarters
pesquisa-fonte-fii = CVM · monthly FII report
pesquisa-fonte-fii-trimestral = CVM · quarterly FII report
pesquisa-fonte-ipe = CVM IPE · material facts
pesquisa-fonte-b3 = B3 COTAHIST · quotes
pesquisa-fonte-b3-listados = B3 listed companies and funds · official distributions
pesquisa-fonte-sec = SEC EDGAR · US companies
pesquisa-fonte-bacen = Central Bank of Brazil IF.data · banks
pesquisa-fonte-fundos = CVM and B3 · fund and ETF registry
pesquisa-fonte-cripto = CoinGecko and DefiLlama · crypto assets
pesquisa-fonte-brapi = brapi.dev · cross-check (optional)
pesquisa-situacao-atualizado = Updated
pesquisa-situacao-sem-mudanca = No change
pesquisa-situacao-inexistente = Not found
pesquisa-situacao-pulado = Skipped
pesquisa-situacao-erro = Error

## Compact amounts (the number comes already formatted, with the currency)

pesquisa-compacto-tri = { $valor }T
pesquisa-compacto-bi = { $valor }B
pesquisa-compacto-mi = { $valor }M
pesquisa-compacto-mil = { $valor }K

## Indicators

pesquisa-formula = Formula
pesquisa-indisponivel = Unavailable
pesquisa-periodo = Period
pesquisa-fonte = Source
pesquisa-status = Status
pesquisa-estimado = ≈ Estimated
pesquisa-conferencia = Data check
pesquisa-conferencia-conferido = Input data checked
pesquisa-conferencia-corrigido = Corrected with another official source
pesquisa-conferencia-aviso = With a warning
pesquisa-conferencia-bloqueado = Blocked: invalid input data
pesquisa-conferencia-marca-corrigido = { $indicador }: corrected with another official source
pesquisa-conferencia-marca-aviso = { $indicador }: with a warning about the data
pesquisa-indicador-indisponivel = { $indicador }: unavailable
pesquisa-indisponivel-motivo = Unavailable: { $motivo }
pesquisa-calculando-indicadores = Calculating indicators
pesquisa-cotacao = Quote
pesquisa-cotacao-anterior = { $cotacao } (previous)
pesquisa-sem-cotacao = no quote
pesquisa-fluxos = Flows
pesquisa-fluxos-12-meses = 12 months to { $data }
pesquisa-informe-mensal = monthly report
pesquisa-calculado-em = Calculated on
pesquisa-notas-calculo = Calculation notes
pesquisa-indicadores-grupo = Indicators · { $grupo }
pesquisa-indicadores-dica = Hover or click an indicator name to see its formula, period and source. Earnings and equity are those attributable to controlling shareholders; flows cover the last 12 months (DFP + ITR).

## Calculated price

pesquisa-preco-justo-metodo = Calculated price · { $metodo }
pesquisa-preco-justo-de = Calculated price ({ $metodo })
pesquisa-margem-cotacao = Margin to the quote
pesquisa-cotacao-valor = quote { $valor }
pesquisa-premissas = Assumptions · { $metodo }
pesquisa-premissa = Assumption
pesquisa-valor = Value
pesquisa-nao-recomendacao = Not investment advice
pesquisa-aviso-estimativas = Estimates from classic formulas and the data provided; they are not investment advice.
pesquisa-taxa-minima-bazin = Minimum rate (Bazin)
pesquisa-taxa-invalida = Enter a rate between 0 and 100%.
pesquisa-padrao-ajustes = default from settings
pesquisa-media-dividendos = Dividend average
pesquisa-n-anos = { $n ->
    [one] { $n } year
   *[other] { $n } years
}
pesquisa-usar-padrao = Use the default
pesquisa-calculando-preco-justo = Calculating the price
pesquisa-cotacao-em = { $preco } on { $data } · { $fonte }

## Documents

pesquisa-doc-entregue = Filed
pesquisa-doc-categoria = Category
pesquisa-doc-assunto = Subject
pesquisa-doc-documento = Document
pesquisa-doc-sem-assunto = (no subject given)
pesquisa-doc-abrir-cvm-rotulo = Open on CVM: { $categoria } from { $data }
pesquisa-doc-abrir-cvm = Open on CVM ↗
pesquisa-doc-categorias = Document category
pesquisa-doc-fatos = Material facts
pesquisa-doc-comunicados = Announcements
pesquisa-doc-avisos = Notices
pesquisa-doc-periodo = Last 12 months · CVM IPE
pesquisa-doc-vazio = No documents in this period
pesquisa-doc-vazio-texto = Nothing in the chosen categories and period.
pesquisa-doc-tabela = Company documents
pesquisa-doc-rodape = Subject and summary are the company's own text, shown as is. The official document opens on the CVM website.

## History

pesquisa-hist-receita-liquida = Net revenue
pesquisa-hist-lucro-liquido = Net income
pesquisa-hist-margem-ebitda = EBITDA margin
pesquisa-hist-margem-liquida = Net margin
pesquisa-hist-roe = ROE
pesquisa-hist-dividendos-acao = Dividends per share
pesquisa-hist-rendimento-cota = Income per unit
pesquisa-hist-vp-cota = Book value per unit
pesquisa-hist-carregando = Loading history
pesquisa-hist-vazio = No statements downloaded for this asset
pesquisa-hist-vazio-texto = History comes from CVM's DFPs (or the SEC's XBRL). Run the collection in Research data.
pesquisa-hist-resultado-distribuivel = Distributable income by fiscal year
pesquisa-hist-receita-lucro = Revenue and earnings by fiscal year
pesquisa-hist-fonte-fii = Source: CVM monthly reports · earnings attributable to controlling shareholders · { $moeda }
pesquisa-hist-fonte-dfp = Source: CVM DFP / SEC 10-K · earnings attributable to controlling shareholders · { $moeda }
pesquisa-hist-receita-lucro-rotulo = Revenue and net income of { $nome } by fiscal year, { $periodo }.
pesquisa-hist-margens = Margins and ROE
pesquisa-hist-margens-fonte = Margin = result ÷ net revenue; ROE = earnings ÷ equity at fiscal year end
pesquisa-hist-margens-rotulo = Margins and ROE of { $nome } by fiscal year, { $periodo }.
pesquisa-hist-margens-tabela = Margins and ROE by fiscal year
pesquisa-hist-dividendos = Dividends per share by year
pesquisa-hist-dividendos-fonte = Distributions per share with record date in the year (calendar); without a calendar, those declared in the DFP
pesquisa-hist-dividendos-rotulo = Dividends per share of { $nome } by year, { $periodo }.
pesquisa-hist-rendimento = Income per unit, 24 months
pesquisa-hist-fonte-informe-fii = Source: monthly FII report (CVM)
pesquisa-hist-rendimento-rotulo = Monthly income per unit of { $nome }, 24 months.
pesquisa-hist-rendimento-tabela = Income per unit by month
pesquisa-hist-vp = Book value per unit, 24 months
pesquisa-hist-vp-rotulo = Book value per unit of { $nome }, 24 months.
pesquisa-hist-vp-tabela = Book value per unit by month

## Comparison

pesquisa-cmp-tabela = Indicator comparison
pesquisa-cmp-indicador = Indicator
pesquisa-cmp-melhor = (best)
pesquisa-cmp-sem-historico = No quote history to compare
pesquisa-cmp-sem-historico-texto = The normalized quote uses the history of registered assets (Yahoo Finance). Securities only in the universe have quotes for just a few sessions.
pesquisa-cmp-cotacao-normalizada = Normalized quote (base 100), 12 months
pesquisa-cmp-fonte = Base 100 on { $data }, first session with a quote for all · close not adjusted for distributions
pesquisa-cmp-fonte-faltando = { $fonte } · no history: { $tickers }
pesquisa-cmp-grafico-rotulo = Normalized quote, base 100, of { $tickers } since { $data }.
pesquisa-cmp-tabela-mensal = Normalized quote by month
pesquisa-cmp-data = Date
pesquisa-cmp-ja-esta = { $ticker } is already in the comparison.
pesquisa-cmp-maximo = Compare at most { $n } assets.
pesquisa-cmp-fora-universo = { $ticker } is not in the downloaded universe.
pesquisa-cmp-adicionar-rotulo = Add to comparison
pesquisa-cmp-ticker = Ticker to compare
pesquisa-cmp-ticker-exemplo = Ticker (e.g. VALE3)
pesquisa-cmp-adicionar = Add
pesquisa-cmp-papeis = Compared securities
pesquisa-cmp-tirar = Remove { $ticker } from the comparison
pesquisa-cmp-escolha = Pick 2 to 6 securities
pesquisa-cmp-escolha-texto = Stocks, FIIs and US stocks from the universe, side by side: indicators with the best of each row highlighted and the quote normalized to base 100.
pesquisa-cmp-exemplo = Example: PETR4 × VALE3 × WEGE3
pesquisa-cmp-comparando = Comparing
pesquisa-cmp-rodape = ◆ best value in the row by the indicator's natural order (lowest P/E, highest DY…); rows without a natural order (quote, enterprise value) are not highlighted. Values in each security's currency. Calculated on { $data }. Not investment advice.

## Universe filter

pesquisa-recorte = Universe filter
pesquisa-classes = Classes
pesquisa-so-carteira = Portfolio and watchlist only
pesquisa-setor-contem = Sector contains
pesquisa-varios-virgula = Several: separate with commas
pesquisa-setor-exemplo = e.g. energia, financeiro
pesquisa-segmento-contem = Segment contains
pesquisa-segmento-dica = FIIs: logística, shoppings, papel…
pesquisa-segmento-exemplo = e.g. bancos, logística
pesquisa-liquidez-minima = Minimum daily liquidity
pesquisa-liquidez-dica = In the asset's currency (R$ on B3, USD in the US)
pesquisa-liquidez-invalida = Enter an amount (e.g. 1,000,000).

## Condition builder

pesquisa-tipo-texto = Text
pesquisa-tipo-sim-nao = Yes/no
pesquisa-logica-todas = All (AND)
pesquisa-logica-todas-dica = The security must pass all of them
pesquisa-logica-qualquer = Any (OR)
pesquisa-logica-qualquer-dica = Passing one is enough
pesquisa-cond-n-grupo = { $n } in group { $grupo }
pesquisa-cond-campo = Field for condition { $n }
pesquisa-cond-operador = Operator for condition { $n }
pesquisa-cond-valor = Value for condition { $n }
pesquisa-cond-minimo = Minimum for condition { $n }
pesquisa-cond-maximo = Maximum for condition { $n }
pesquisa-cond-remover = Remove condition { $n }
pesquisa-cond-classe = Class
pesquisa-cond-min = min.
pesquisa-cond-max = max.
pesquisa-cond-lista = a, b, c
pesquisa-cond-texto = text
pesquisa-cond-e = and
pesquisa-combinar = Combine
pesquisa-combinar-tudo = Combine conditions and groups
pesquisa-sem-condicoes = No conditions: every security in the filter.
pesquisa-condicoes = Conditions
pesquisa-grupo-n = Group { $n }
pesquisa-combinar-grupo = Combine conditions in group { $n }
pesquisa-remover-grupo = Remove group
pesquisa-condicoes-grupo = Conditions in group { $n }
pesquisa-condicao-no-grupo = Condition in group
pesquisa-adicionar-condicao = Add condition
pesquisa-adicionar-grupo = Add group (OR…)

## Filter errors

pesquisa-erro-operador = This operator does not apply to this field.
pesquisa-erro-lista = Enter one or more values separated by commas.
pesquisa-erro-texto = Enter the text.
pesquisa-erro-numero = Enter a number (e.g. 10 or 6.5).
pesquisa-erro-maximo = Enter the maximum.
pesquisa-erro-maximo-menor = The maximum is lower than the minimum.
pesquisa-erro-grupo-vazio = Empty group: add a condition.

## Screener

pesquisa-scr-salvar-titulo = Save screener view
pesquisa-scr-salvar-texto = The view keeps the filters and the universe filter; run it again whenever you like (the day's data changes).
pesquisa-scr-salvar-rotulo = Save view
pesquisa-scr-nome = Name
pesquisa-scr-informe-nome = Enter a name.
pesquisa-scr-descricao = Description (optional)
pesquisa-scr-salvar = Save view
pesquisa-scr-colunas = Result columns ({ $n })
pesquisa-scr-tela-salva = View “{ $nome }” saved
pesquisa-scr-tela-atualizada = View “{ $nome }” updated
pesquisa-scr-tela-excluida = View “{ $nome }” deleted
pesquisa-scr-erro-excluir = Couldn't delete
pesquisa-scr-desfeita = Change undone
pesquisa-scr-erro-desfazer = Couldn't undo
pesquisa-scr-ticker = Ticker
pesquisa-scr-nome-col = Name
pesquisa-scr-universo-vazio = The universe is still empty
pesquisa-scr-universo-vazio-texto = You can build and save views now; results show up after the first collection in Research data.
pesquisa-scr-tela = View · { $nome }
pesquisa-scr-filtros = Filters
pesquisa-scr-telas-salvas = Saved views
pesquisa-scr-abrir-tela = Open saved view…
pesquisa-scr-nenhuma-tela = No saved views
pesquisa-scr-salva-em = Saved: { $resumo } · changed on { $data }
pesquisa-ordenar-por = Sort by
pesquisa-scr-ordem = Result order
pesquisa-ordem-natural = Natural
pesquisa-maior-primeiro = Highest first
pesquisa-menor-primeiro = Lowest first
pesquisa-scr-rodar = Run screener
pesquisa-scr-salvar-alteracoes = Save changes
pesquisa-scr-salvar-nova = Save as new…
pesquisa-scr-salvar-tela = Save view…
pesquisa-scr-nova = New view
pesquisa-scr-excluir = Delete view
pesquisa-scr-excluir-texto = The view “{ $nome }” leaves the list. You can undo right after or in Settings › Change history.
pesquisa-scr-corrija = Fix the highlighted filters.
pesquisa-scr-desfazer = Undo
pesquisa-scr-desfazer-texto = You can also undo in Settings › Change history.
pesquisa-scr-instrucoes = Build the filters and click { $botao }. Percentages in % (DY ≥ 6), multiples as times (P/E < 12), money in the asset's currency.
pesquisa-scr-rodando = Running the screener
pesquisa-scr-resultado = Screener result
pesquisa-scr-n-papeis = { $n ->
    [one] { $n } security
   *[other] { $n } securities
}
pesquisa-universo-de = universe as of { $data }
pesquisa-scr-nenhum = No security passed the filters.
pesquisa-scr-mostrando = Showing the first { $n }; narrow the filters to see the rest.

## Rankings

pesquisa-rank-maiores-dy = Highest DY
pesquisa-rank-menor-pl = Lowest P/E
pesquisa-rank-margem-graham = Margin to Graham
pesquisa-rank-margem-bazin = Margin to Bazin
pesquisa-rank-sem-prejuizo = Never had a loss
pesquisa-rank-maior-roe = Highest ROE
pesquisa-rank-valor-mercado = Market cap
pesquisa-rank-liquidez-fii = FII liquidity
pesquisa-rank-pvp-fii = FII P/B
pesquisa-rank-menor-primeiro = lowest first
pesquisa-rank-maior-primeiro = highest first
pesquisa-rank-nome = Name
pesquisa-rank-liquidez = Daily liquidity
pesquisa-rank-prontos = Ready-made rankings
pesquisa-rank-indicador = Indicator
pesquisa-rank-ordem = Order
pesquisa-rank-ordem-rotulo = Ranking order
pesquisa-rank-mostrar = Show
pesquisa-rank-ordem-natural = natural order: { $ordem }
pesquisa-rank-montando = Building ranking
pesquisa-rank-tabela = Ranking · { $indicador }
pesquisa-rank-vazio = No security in the filter has this indicator.
pesquisa-rank-total = { $n ->
    [one] { $n } security with the indicator
   *[other] { $n } securities with the indicator
}
pesquisa-rank-sem-valor = { $n } without a value
pesquisa-rank-sem-valor-multiplo = { $n } without a value (or ≤ 0, which doesn't count as cheap)
pesquisa-rank-rodape = universe as of { $data } · each security's currency (B3 in R$, US in USD). Not investment advice.

## Research data

pesquisa-dados-coleta-fonte = Collection by source
pesquisa-dados-fonte = Source
pesquisa-dados-arquivos = Files
pesquisa-dados-situacao = Status
pesquisa-dados-baixado = Downloaded
pesquisa-dados-linhas = Rows
pesquisa-dados-com-problema = { $n } with problems
pesquisa-dados-n-atualizados = { $n } updated
pesquisa-dados-n-ok = { $n } ok
pesquisa-dados-n-pulados = { $n } skipped, no problem
pesquisa-dados-total = Total
pesquisa-dados-aviso = Collection warning
pesquisa-dados-concluida = Research collection finished
pesquisa-dados-concluida-avisos = Collection finished with warnings
pesquisa-dados-concluida-texto = { $tamanho } downloaded · { $n } securities in the universe
pesquisa-dados-coleta-titulo = Asset research collection
pesquisa-dados-coletando-live = COLLECTING
pesquisa-dados-fontes = Sources
pesquisa-dados-primeira-coleta = First collection
pesquisa-dados-ultima-coleta = Last collection
pesquisa-dados-nunca = never
pesquisa-dados-universo = Universe
pesquisa-dados-universo-valor = { $n } securities · { $data }
pesquisa-dados-vazio = empty
pesquisa-dados-emissores = Issuers
pesquisa-dados-papeis = Securities
pesquisa-dados-fatos = Facts
pesquisa-dados-documentos = Documents
pesquisa-dados-cotacoes-ate = Quotes through
pesquisa-dados-coletando = Collecting…
pesquisa-dados-coletar-agora = Collect now
pesquisa-dados-baixando = Downloading and calculating
pesquisa-dados-primeira-leva = the first one takes 1 to 2 minutes
pesquisa-dados-so-mudou = only what changed
pesquisa-dados-falhou = The collection failed
pesquisa-dados-ultima-por-fonte = Last collection by source
pesquisa-dados-resumo = { $qual ->
    [esta] This collection
   *[ultima] Last collection
}: { $tamanho } in { $segundos } s · { $universo } securities in the universe · { $radar } distributions projected by the radar
pesquisa-dados-coletar-titulo = Collect research data
pesquisa-dados-coletar-primeira = The first collection downloads about 230 MB from CVM (DFP, ITR, FCA, FII, IPE), B3 (COTAHIST) and the SEC and takes 1 to 2 minutes. Nothing stays on disk besides the encrypted database.
pesquisa-dados-coletar-incremental = Downloads only the files that changed since the last collection (usually a few MB) and recalculates the universe and the dividend radar.
pesquisa-dados-baixar-230 = Download ~230 MB
pesquisa-dados-coletar = Collect
pesquisa-dados-baixar-tudo = Download everything again
pesquisa-dados-baixar-tudo-dica = Ignores what hasn't changed (~230 MB again). Only to fix data.
pesquisa-dados-ajustes-salvos = Research settings saved
pesquisa-dados-ajustes = Research settings
pesquisa-dados-sec-email = Set up the SEC email
pesquisa-dados-sec-ua = SEC EDGAR User-Agent
pesquisa-dados-sec-ua-dica = The app name and a real contact email; the SEC blocks requests without identification.
pesquisa-dados-sec-ua-erro = Include an email (e.g. Treasury/0.1 your.name@email.com).
pesquisa-dados-sec-ua-exemplo = Treasury/0.1 your.name@email.com
pesquisa-dados-taxa-invalida = Between 0 and 100%.
pesquisa-dados-bazin-dica = Default assumptions for Bazin's ceiling price (average dividends per share ÷ minimum rate); you can change them case by case in the asset's Calculated price tab.
pesquisa-dados-descartar = Discard
pesquisa-dados-salvar-ajustes = Save settings
pesquisa-doc-assembleias = Meetings
pesquisa-doc-janela = Document period
pesquisa-doc-12-meses = 12 months
pesquisa-doc-tudo-coletado = All collected
pesquisa-doc-ultimo = The latest stored document is from { $data }.
pesquisa-doc-ampliar = Show since { $data }
pesquisa-doc-sem-fonte = No document source for this class
pesquisa-doc-sem-fonte-texto = Material facts and reports of FIIs, ETFs and funds are published on Fundos.NET (B3), which the app does not collect yet. Listed companies come from the CVM IPE.
pesquisa-doc-sem-emissor = We could not find this asset's issuer
pesquisa-doc-sem-emissor-texto = The asset has no CNPJ in the registry and the ticker is not in the CVM company registry (FCA). Enter the company's CNPJ in the asset to bring its documents.
pesquisa-doc-informar-cnpj = Enter the CNPJ
pesquisa-doc-sem-coleta = The CVM collection has not run yet
pesquisa-doc-sem-coleta-texto = Run the collection in Analysis data to bring the companies' documents.
pesquisa-doc-quieta = No documents since { $data }
pesquisa-doc-quieta-texto = The company has not published a material fact, announcement, notice, shareholder meeting or board meeting in the kept categories since that date.
pesquisa-doc-diagnostico = Documents diagnosis
pesquisa-doc-diag-fonte = Source
pesquisa-doc-diag-fonte-ipe = CVM IPE (listed companies)
pesquisa-doc-diag-emissor = Issuer
pesquisa-doc-diag-sem-emissor = not found
pesquisa-doc-diag-coleta = Collection
pesquisa-doc-diag-coleta-em = { $data }, keeping since { $desde }
pesquisa-doc-diag-sem-coleta = has not run yet
pesquisa-doc-diag-guardados = Stored
pesquisa-doc-diag-nenhum = none
pesquisa-doc-diag-categorias = Kept categories
pesquisa-nao-tem-ativo = You do not have this asset. Watch puts it on the watchlist, without any trade, and turns on the score history, alerts and the AI Study.
pesquisa-observado-pelo-mercado = Watched from Market: stopping removes the asset from the registry if it never had a trade.
pesquisa-observar = Watch
pesquisa-deixar-observar = Stop watching
pesquisa-observando = Watching { $ticker }
pesquisa-observando-texto = It is on the watchlist, without trades.
pesquisa-deixou-observar = Stopped watching
pesquisa-observar-erro = Could not watch
pesquisa-scr-metodos = Methods
pesquisa-scr-avisar = Notify when a new security enters
pesquisa-tags = Characteristics
pesquisa-tags-ajuda = Pick the ones the security must have (all together). Hover to see the rule.
pesquisa-tags-limpar = Clear characteristics
pesquisa-tag-chip = { $tag } · { $n }
pesquisa-tag-vazia = { $regra }
    No security in the current results has this characteristic.
pesquisa-tags-mais = { $tags } and { $n } more
