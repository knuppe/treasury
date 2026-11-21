# Market: ticker search, asset picker, intraday quote and exchanges (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Ticker search

mercado-origem-cadastrado = registered
mercado-origem-analise = analysis
mercado-origem-yahoo = Yahoo
mercado-busca-placeholder = Ticker or name (PETR4, Vodafone, 7203…)
mercado-busca-rotulo = Search ticker
mercado-sugestoes = Asset suggestions
mercado-bolsa-fora = exchange not in the catalog
mercado-buscando = Searching…
mercado-so-locais = { $erro }. Showing local assets only.

## Asset picker

mercado-ativo-novo = New asset
mercado-bolsa-fora-catalogo = The exchange { $bolsa } is not in the catalog: add it in Assets › Exchanges.
mercado-cadastrar-usar = Add and use

## Intraday quote

mercado-atraso = ~{ $min } min delay
mercado-agora = now
mercado-intradiaria = Intraday quote
mercado-ultimo-negocio = Last trade: { $quando }
mercado-ultimo-negocio-bolsa = Last trade: { $quando } ({ $hora } at the exchange)
mercado-fonte-yahoo = Yahoo Finance, no key, about { $min } min delayed. History uses daily closes.
mercado-cotacao-com = Quote with { $atraso }
mercado-cotacao-com-estado = Quote with { $atraso }, { $estado }

## Intraday chart

mercado-fech-anterior = Prev. close
mercado-buscando-cotacao = Fetching quote…
mercado-sem-intradiaria = No intraday quote yet.
mercado-periodo-intradiario = Intraday period
mercado-periodo-1d = 1D
mercado-periodo-5d = 5D
mercado-nao-buscou = Could not fetch the quote right now.
mercado-intradiaria-aria = { $dias ->
    [one] Intraday quote for { $nome } (1 day)
   *[other] Intraday quote for { $nome } ({ $dias } days)
}
mercado-sem-pontos = No intraday points. They arrive while the market is open (every 5 min).

## Exchanges (Assets › Exchanges)

mercado-dias-seg-sex = Mon–Fri
mercado-dias-dom-qui = Sun–Thu
mercado-dias-curtos = Mon,Tue,Wed,Thu,Fri,Sat,Sun
mercado-almoco = {" "}(lunch { $inicio }–{ $fim })
mercado-bolsa = Exchange
mercado-pais = Country
mercado-moeda = Currency
mercado-cambio-manual = manual FX
mercado-cambio-manual-dica = No PTAX rate at Banco Central and no ECB reference rate: enter the exchange rate manually.
mercado-sem-sufixo = no suffix
mercado-sem-sufixo-dica = The Yahoo symbol is the ticker itself
mercado-pregao-local = Trading hours (local)
mercado-agora-col = Now
mercado-hora-local = { $hora } local
mercado-remover-bolsa = Remove exchange
mercado-remover-bolsa-descricao = { $codigo } leaves the catalog (only if no assets use it).
mercado-bolsa-removida = Exchange removed
mercado-filtrar-bolsas = Filter exchanges
mercado-filtrar-placeholder = Filter by code, country, currency…
mercado-resumo = { $n ->
    [one] { $n } exchange
   *[other] { $n } exchanges
} · { $abertas } trading now
mercado-cadastrar-bolsa = Add exchange
mercado-bolsas-ajuda = The exchange code is the asset's “market”: it sets the default currency and country, the Yahoo symbol (ticker + suffix) and the intraday collection hours (every 5 min, ~15 min delayed, only while the market is open). Holidays are not in the catalog.
mercado-nenhuma-bolsa = No exchanges
mercado-nada-filtro = Nothing matches the filter.
mercado-dialogo-descricao = Regular trading hours in the exchange's time zone (IANA, e.g. Europe/London).
mercado-erro-codigo = Up to 12 letters and numbers.
mercado-erro-nome = Enter the name.
mercado-erro-pais = 2-letter ISO code.
mercado-erro-moeda = 3-letter ISO code.
mercado-erro-fuso = Enter the time zone (e.g. Asia/Manila).
mercado-bolsa-atualizada = Exchange updated
mercado-bolsa-cadastrada = Exchange added
mercado-iso-2 = 2-letter ISO.
mercado-iso-3 = 3-letter ISO.
mercado-sufixo = Yahoo suffix
mercado-sufixo-dica = E.g. .PS; empty in the US.
mercado-fuso = Time zone (IANA)
mercado-mic-dica = Optional (ISO 10383).
mercado-abertura = Open
mercado-dias-pregao = Trading days
mercado-dias-pregao-dica = 1 = Monday … 7 = Sunday.
mercado-almoco-inicio = Lunch: start
mercado-almoco-fim = Lunch: end
mercado-apelidos = Aliases
mercado-apelidos-dica = Comma-separated (Yahoo codes, names).
mercado-salvar-bolsa = Save exchange

## Tela Mercado (docs/design/market-explorer.md)

mercado-secoes = Market sections
mercado-aba-panorama = Overview
mercado-aba-explorador = Explorer
mercado-aba-mapa = Map
mercado-aviso = List of what meets the criteria, from public data. Not investment advice.
mercado-panorama-titulo = The market today
mercado-acoes = Stocks
mercado-fiis = FIIs
mercado-todos = All
mercado-com-score = With score
mercado-score-de = Score computed on { $data }
mercado-score-pendente = The market score comes with the next collection
mercado-dados = Data from
mercado-coleta-em = Collected on { $data }
mercado-sem-coleta = The collection has not run yet
mercado-panorama-de = Overview of
mercado-acoes-por-setor = Stocks by sector
mercado-fiis-por-segmento = FIIs by segment
mercado-col-setor = Sector
mercado-col-segmento = Segment
mercado-col-papeis = Securities
mercado-col-saudaveis = Robust and healthy
mercado-col-score = Score (median)
mercado-col-pl = P/E (median)
mercado-col-pvp = P/B (median)
mercado-col-dy = DY (median)
mercado-panorama-rodape = Sector medians (P/E and P/B only with positive values). Click a sector's name (or double-click its row) to open the explorer with its healthy securities.
mercado-sem-universo = The universe has not been downloaded yet
mercado-sem-universo-texto = Run the analysis collection in Assets › Data to bring the B3 stocks and FIIs.
mercado-mapa-titulo = Health × Attractiveness
mercado-mapa-saude = Health
mercado-mapa-atratividade = Attractiveness
mercado-mapa-classe = Class on the map
mercado-mapa-aria = Map of { $n } securities: health on the vertical axis, attractiveness on the horizontal
mercado-mapa-vazio = No market score yet
mercado-mapa-vazio-texto = The map uses the score the collection computes for the whole universe. Run the collection in Assets › Data.
mercado-mapa-leitura = How to read the map
mercado-mapa-leitura-texto = Each bubble is a security, sized by market cap and colored by score band. The top right corner joins high health and attractiveness. An outline marks portfolio and watchlist securities. Click to open the security.
mercado-aba-metodos = Methods
mercado-metodo-criterios = Criteria
mercado-metodo-opcional = (missing data does not exclude)
mercado-metodo-adaptado = Adapted to the B3. Original: { $original }
mercado-metodo-higiene = Hygiene
mercado-metodo-ordem = Order
mercado-metodo-nao-se-aplica = Does not apply to
mercado-metodo-fonte = Source
mercado-col-score-papel = Score
mercado-metodo-por-que = Why a security meets it or not
mercado-metodo-por-que-exemplo = E.g. PETR4
mercado-metodo-explicar = Explain
mercado-metodo-passa = { $ticker } meets every criterion (#{ $rank } in the method's order).
mercado-metodo-fora = { $ticker } is left out before the criteria: { $motivo }.
mercado-metodo-nao-passa = { $ticker } does not meet every criterion:
mercado-metodo-sem-dado = no data
mercado-metodo-voltar = All methods
mercado-metodo-lista = What meets { $metodo }
mercado-metodo-resultado = { $passam } of { $avaliados } evaluated securities meet every criterion today.
mercado-metodo-ninguem = No security meets every criterion today.
mercado-metodo-cumprem = { $n } securities meet the criteria today
mercado-metodo-contando = Counting…
mercado-metodo-ver-lista = See the recipe and the list
mercado-metodo-fonte-versao = { $fonte } · v{ $versao }
mercado-abrir-setor = Open { $setor } in the explorer
