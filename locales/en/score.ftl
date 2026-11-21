# Asset health score: rating, bands, dimensions, metrics, red flags, events, history and feedback
# (docs/i18n-design.md). Explanations and notes from the core don't go through here.
# Revision: AI draft, not yet reviewed by a native speaker.

## Bands, status and severity

score-faixa-robusto = Robust
score-faixa-saudavel = Healthy
score-faixa-atencao = Caution
score-faixa-risco = Risk
score-dados-insuficientes = Not enough data
score-status-calculado = Calculated
score-status-parcial = Partial
score-status-nao-suportada = Class not supported
score-severidade-informativa = Informational
score-severidade-baixa = Low
score-severidade-media = Medium
score-severidade-alta = High
score-severidade-critica = Critical

## Metric values

score-dias = { $n ->
    [one] { $valor } day
   *[other] { $valor } days
}
score-anos = { $n ->
    [one] { $valor } year
   *[other] { $valor } years
}
score-percentil-pares = { $n ->
    [one] { $p } of { $n } peer
   *[other] { $p } of { $n } peers
}
score-teto = cap { $nota }
score-penalidade = { $n ->
    [one] { $pontos } point
   *[other] { $pontos } points
}

## Pieces: chip, bar and scale

score-chip-parcial = {" "}· partial
score-barra-sem-nota = { $rotulo }: no rating
score-barra-nota = { $rotulo }: { $nota } out of 100
score-faixas-regua = Bands from 0 to 100: { $faixas }

## Health tab: header

score-segmento = Segment
score-segmento-metodologia = Methodology segment
score-segmento-automatico = Automatic segment
score-segmento-trocado = Segment changed
score-segmento-refeito = { $ticker }: the score was recalculated with the new segment's weights.
score-segmento-auto-setor = Automatic (by sector)
score-escolhido-por-voce = Chosen by you ·{" "}
score-titulo = Asset health score
score-metodologia = Methodology
score-avaliado-em = Rated on
score-dados-de = Data from
score-defasado = outdated
score-defasado-dica = The latest data is more than 2 quarters old
score-nota = Score
score-nota-parcial = Score (partial)
score-pontos = { $delta } points
score-mudanca = { $pontos } since { $data } (was { $nota })
score-mudanca-faixa = { $pontos } since { $data } (was { $nota }, { $faixa })
score-confianca = Confidence
score-saude = Health
score-saude-dica = without valuation
score-atratividade = Attractiveness
score-atratividade-dica = price
score-pares = Peers
score-pares-setor = { $setor } in sector { $nome } · { $segmento } in segment · { $classe } in class
score-pares-resumo = { $setor } in sector · { $segmento } in segment · { $classe } in class
score-alertas = Score alerts

## Health tab: missing critical data and reasons

score-nao-calculado = Score not calculated: critical data missing
score-criticos-ausentes = Missing critical data
score-critica = critical
score-tambem-sem-dado = Also missing: { $lista }.
score-sem-metricas = Without these metrics the score isn't calculated (nothing is made up). The case was logged for the developer, in 15 · Developer, to review the data collection. The dimensions below show what has already been measured.
score-por-que = Why this score
score-a-favor = In favor
score-contra = Against
score-red-flags = Red flags
score-gravidade = severity { $nivel }
score-antes-flags = Before the red flags the score was { $nota }.
score-sem-dado-redistribuido = No data (the weight was redistributed within the dimension): { $lista }.

## Health tab: models, cross-checks and notices

score-piotroski = { $valor } of { $total }
score-itens = Items · { $modelo }
score-atende = passes
score-nao-atende = fails
score-metrica = Metric
score-fonte = Source
score-diferenca = Difference
score-conferido-com = Checked against
score-calculando = Calculating the score
score-notas-calculo = Calculation notes
score-cripto-fontes = Crypto data sources
score-cripto-fontes-texto = Data provided by:
score-indisponivel = Health score not available yet for { $classe }
score-cripto-titulo = Robustness within the class
score-cripto-texto = Crypto can't be compared directly with stocks: the score measures robustness within the crypto asset class.
score-parcial-titulo = Partial score: no final band
score-parcial-texto = { $pct }% of the weight came from imputed dimensions (peer median × 0.9) for lack of data. The number is shown, but the band stays as “Not enough data” until coverage improves.
score-dimensoes-titulo = Dimensions and metrics
score-dimensoes-ajuda = Click a dimension to see its metrics. Health adds up the dimensions without the “attractiveness” tag; Attractiveness, only the price ones. Percentiles already point in the “higher is better” direction.
score-modelos = Classic models
score-conferencia = Cross-check with other sources
score-conferencia-ajuda = Cross-validation only: the score uses the numbers Treasury calculates from CVM filings. Income is checked against B3 (official); if the calendar differs, the dividend yield uses B3. Differences over 10% are highlighted.

## Dimensions and metrics

score-fonte-de = Source: { $fonte }
score-critica-dica = Without this metric there is no score
score-sem-dado = No data: { $motivo }
score-valor = Value
score-col-nota = Rating
score-pior = {" "}(worst)
score-entre-pares = Among peers
score-absoluta = Absolute
score-peso = Weight
score-periodo = Period
score-imputada = imputed
score-imputada-dica = Coverage below the minimum: rating from the peer median × 0.9
score-selo-atratividade = attractiveness
score-selo-atratividade-dica = Counts toward the Attractiveness sub-score (price), not Health
score-peso-cobertura = weight { $peso } · coverage { $cobertura }
score-metricas-de = Metrics · { $dimensao }
score-dimensoes = Score dimensions

## Score history

score-data = Date
score-faixa = Band
score-historico = Score history
score-historico-fonte = Snapshots from { $de } to { $ate }
score-historico-aria = Score from { $primeira } on { $de } to { $ultima } on { $ate }

## Your opinion ("Disagree")

score-discordo = Disagree
score-concordo = Agree
score-sem-opiniao = Don't know / no opinion
score-opiniao-registrada = Opinion recorded
score-opiniao-registrada-dica = It goes into the review of the score methodology.
score-opiniao-form = Opinion on the score
score-veredito = Verdict
score-faixa-esperada = Band you would expect
score-opcional = Optional.
score-motivo = Reason
score-motivo-dica = What the score missed or weighed too much (e.g. debt already renegotiated, one-off profit).
score-motivo-erro = Tell us the reason.
score-registrar-opiniao = Record opinion
score-sua-opiniao-sobre = Your opinion on the score
score-sua-opiniao = Your opinion
score-sua-opiniao-dica = Does the score not match what you know about the company? Record it; the cases go into calibrating the methodology.
score-opinioes-registradas = Recorded opinions
score-opiniao-nota = · score { $nota } ({ $faixa }) · methodology { $versao }
score-esperava = · expected { $faixa }
score-discordo-titulo = Disagree with the score · { $nome }
score-discordo-descricao = Score { $nota } ({ $faixa }) under methodology { $versao }. Your opinion is recorded with the day's score.

## SEC contact e-mail

score-sec-baixadas = SEC filings downloaded
score-sec-texto = The SEC (the US regulator) only provides data to those who identify themselves with a contact e-mail. It only goes in the header of requests to the SEC, stays in the vault and can be changed in Settings › Data sources.
score-sec-email = Your contact e-mail for the SEC
score-sec-baixando = Downloading from the SEC…
score-sec-salvar = Save and download

## Qualitative events

score-evento-aguardando = Waiting for you
score-evento-aceito = Accepted (counts in the score)
score-evento-rejeitado = Rejected
score-evento-resolvido = Resolved
score-evidencia = evidence ↗
score-evento-hoje = · today: { $efeito }
score-aceitar = Accept
score-rejeitar = Reject
score-marcar-resolvido = Mark as resolved
score-voltar-valer = Count again
score-apagar-evento = Delete the event { $titulo }
score-evento-aplicado = Event applied to the score
score-casou = · matched “{ $termo }”
score-aplicar = Apply to the score
score-evento-registrado = Event recorded
score-nota-refeita = The score was recalculated.
score-novo-evento = New event
score-evento-tipo = Type
score-evento-tipo-dica = The cap or penalty for each type is in the methodology.
score-evento-o-que = What happened
score-evento-descreva = Describe the event.
score-informe-data = Enter the date.
score-evidencia-link = Evidence (link)
score-evidencia-dica = Material fact, news or document. Recommended.
score-observacao = Note
score-registrar-aplicar = Record and apply
score-eventos-titulo = Qualitative events
score-eventos-ajuda = Bankruptcy protection, fraud, litigation, disasters, governance and regulation. Only events you accept affect the score, and the effect fades over time until you mark them as resolved.
score-eventos-registrados = Recorded events
score-nenhum-evento = No events recorded.
score-sugestoes-cvm = Suggestions from CVM material facts
score-sugestoes = Event suggestions
score-novo-evento-de = New event · { $nome }
score-novo-evento-descricao = The event is accepted on entry and the score is recalculated right away.

## Portfolio › Asset health

score-classe-nao-suportada = class not supported
score-falta-critico = Critical data missing
score-parcial-dica = Over 30% of the weight imputed: no final band
score-ativo = Asset
score-variacao = Change
score-flags = Flags
score-calculando-ativos = Calculating the asset scores
score-sem-ativos = No assets
score-carteira-vazia = No assets in the portfolio or on the watchlist
score-carteira-vazia-descricao = The health score rates the fundamentals (profitability, debt, earnings quality, valuation…) of each stock or real estate fund you own or follow. Record a trade or watch an asset in 09 · Assets.
score-carteira-ajuda = Health score from 0 to 100 for each asset you hold or watch, based on public fundamentals (CVM, B3). Robust ≥ 75, Healthy 60–74, Caution 40–59, Risk below 40. Not investment advice.
score-carteira-tabela = Asset health
score-sec-email-exemplo = name@email.com
