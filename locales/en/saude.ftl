# Financial health: Dashboard card 08, Reports › Financial health and Settings › Financial
# health (docs/i18n-design.md). Bands, dimensions, indicators, notes and suggestions come from
# the core. Revision: AI draft, not yet reviewed by a native speaker.

## Models and statuses

saude-modelo-classicos = Classic indicators
saude-modelo-desativado = Off
saude-status-bom = Good
saude-status-atencao = Watch
saude-status-ruim = Poor
saude-status-sem-dados = No data

## Scale and bars (names for screen readers)

saude-faixas-aria = Bands from 0 to 100: { $faixas }
saude-barra-sem-dados = { $rotulo }: no data
saude-barra-oculto = { $rotulo }: value hidden
saude-barra-pontos = { $rotulo }: { $pontos } out of 100

## Dashboard card

saude-titulo = Financial health
saude-modelo = Model
saude-janela = Window
saude-n-meses =
    { $n ->
        [one] { $n } month
       *[other] { $n } months
    }
saude-avaliado = Assessed
saude-ver-indicadores = See indicators
saude-nota-isf = Score (I-SFB estimate)
saude-nota = Score
saude-origem-isf = Estimate in the style of the I-SFB (Febraban and Banco Central) using your transactions, installments, card bills and balances
saude-origem-classicos = Classic indicators calculated from your transactions, installments, card bills and balances
saude-dimensoes = Dimensions
saude-indicadores = Indicators

## Reports › Financial health

saude-trocar-modelo = Change the model
saude-carregando = Measuring financial health
saude-desativada = Off
saude-desligada-titulo = Financial health analysis is off
saude-desligada-descricao = Choose a model in Settings › Financial health to see the score, the dimensions and the suggestions.
saude-abrir-ajustes = Open settings
saude-indicadores-descricao = Each ratio with the usual personal finance target and its weight in the score.
saude-titulo-modelo = Financial health · { $modelo }
saude-periodo = { $de } to { $ate }
saude-sem-lancamentos = no transactions
saude-meses = Months
saude-cambio = Exchange rate
saude-cambio-estimado = ≈ estimated
saude-contas-paradas = Accounts
saude-contas-paradas-valor = { $n ->
    [one] ≈ 1 without a recent statement
   *[other] ≈ { $n } without a recent statement
}
saude-nota-0-100 = Score from 0 to 100
saude-faixas = Bands
saude-sobre-isf = About the I-SFB (Meu Bolso em Dia, Febraban)
saude-medido = Measured
saude-sem-dados = no data
saude-meta = Target
saude-pontos = Points
saude-peso-nota = · weight { $peso }% of the score
saude-peso-dimensao = · weight { $peso }% of the dimension
saude-calculo = Calculation
saude-para-medir = To measure
saude-sugestoes = Suggestions

## Settings › Financial health

saude-desativada-aviso = Financial health turned off
saude-modelo-aviso = Model: { $modelo }
saude-modelo-titulo = Financial health model
saude-como-calculada = How the score is calculated
saude-calculo-dados = The score uses only the data in the vault: income and expenses from the last 12 full months (transfers, card bill payments and contributions don't count), loan installments, card bills, account balances, budget and net worth. Nothing leaves the computer.
# The part between ** is shown in italics.
saude-calculo-sem-dados = An indicator without enough history (less than 3 months) shows **no data** and is left out of the average, instead of pulling the score down.
saude-isf-resumo = behavior, skill, freedom and security; national average 56.7 (2024)
saude-classicos = Classic
saude-classicos-resumo = savings, emergency fund, installments, housing, revolving credit and net worth
saude-fonte-isf = Brazilian Financial Health Index 2024 (Meu Bolso em Dia)
