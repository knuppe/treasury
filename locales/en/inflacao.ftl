# Figure 07 · Inflation on the Reports screen.

inflacao-carregando = Loading inflation…
inflacao-pergunta = Did you spend more, or did things just get pricier? Each month brought to { $mes } money by official inflation.
inflacao-indice-generico = inflation index
inflacao-titulo = Nominal spending × adjusted by { $indice }, { $n } months
inflacao-aria = Lines for nominal spending and spending adjusted by { $indice } over { $n } months, in { $mes } money
inflacao-fonte = Ledger · { $base } basis · values adjusted to { $mes }
inflacao-fonte-indice = { $indice } ({ $fonte }), latest data { $data }
inflacao-nota-estimado = Months not yet released follow the average of the index's latest changes and are marked with ≈.
inflacao-tabela = Nominal and adjusted spending by month

## Measures

inflacao-acumulada = Inflation over { $n } months
inflacao-sem-indice-curto = No index
inflacao-gasto-nominal = Nominal spending
inflacao-gasto-real = Real spending
inflacao-tres-meses = First vs last 3 months
inflacao-correcao-total = Adjustment over the period
inflacao-correcao-total-dica = How much inflation adds to past spending
inflacao-sem-indice = There's no { $indice } data for this period yet. Update the indices in Assets › Indices and quotes.
inflacao-sem-cambio = No exchange rate for { $moedas }: those amounts were left out of the totals.

## Chart

inflacao-serie-nominal = Nominal
inflacao-serie-corrigido = Adjusted
inflacao-tooltip-correcao = Adjustment
inflacao-tooltip-nao-divulgado = This month's index isn't out yet: estimated adjustment.
inflacao-tooltip-estimado = The adjustment uses months not yet released (estimated).

## Table by month

inflacao-col-corrigido = In { $mes } money
inflacao-col-correcao = Adjustment
inflacao-col-indice = Index
inflacao-status-oficial = Official
inflacao-status-estimado = Partly estimated
inflacao-status-nao-divulgado = Not released

## By category

inflacao-categorias-titulo = Categories against inflation
inflacao-categorias-dica = { $acima ->
    [0] Average from { $inicio } over three months against the three months up to { $fim }. None of the { $total } categories rose above inflation.
    [one] Average from { $inicio } over three months against the three months up to { $fim }. 1 of { $total } categories rose above inflation.
   *[other] Average from { $inicio } over three months against the three months up to { $fim }. { $acima } of { $total } categories rose above inflation.
}
inflacao-categorias-tabela = Categories against inflation
inflacao-col-inicio = Average at start
inflacao-col-fim = Average at end
inflacao-col-variacao = Change
inflacao-col-acima = Above inflation
