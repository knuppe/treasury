# Asset analysis (core: `research`): indicator names and formulas, periods, sources, reasons for
# being unavailable, notes, errors and collection texts, in the installation language. The
# document names stored with each value (DFP, ITR, CVM reports) and the texts matched against
# the CVM, B3 and SEC files stay in the code.
# Revision: AI draft, not yet reviewed by a native speaker.

## History and comparison

nucleo-pesquisa-hist-centavos = Money amounts in cents of the asset's currency.
nucleo-pesquisa-hist-sem-demonstracoes = No financial statements downloaded for this asset.
nucleo-pesquisa-hist-unit = Values per unit security ({ $acoes } shares).
nucleo-pesquisa-erro-comparar-minimo = compare at least 2 assets
nucleo-pesquisa-erro-comparar-maximo = compare at most { $max } assets

## Commands

nucleo-pesquisa-erro-ajustes-so-interface = asset analysis settings can only be changed in the app
nucleo-pesquisa-erro-ajuste-desconhecido = unknown setting: { $ajuste }
nucleo-pesquisa-erro-taxa = invalid rate
nucleo-pesquisa-motivo-tela-criada = Screener created: { $tela }
nucleo-pesquisa-motivo-tela-alterada = Screener changed
nucleo-pesquisa-motivo-tela-removida = Screener removed

## Indicators: name and formula

nucleo-pesquisa-ind-price = Price
nucleo-pesquisa-ind-price-formula = Latest available close
nucleo-pesquisa-ind-market_cap = Market cap
nucleo-pesquisa-ind-market_cap-formula = Σ shares of each class × price of the class (FII: units × price)
nucleo-pesquisa-ind-ev = Enterprise value (EV)
nucleo-pesquisa-ind-ev-formula = Market cap + gross debt − cash and short-term investments
nucleo-pesquisa-ind-liquidity = Average daily liquidity
nucleo-pesquisa-ind-liquidity-formula = Average daily traded value over the latest sessions (up to 21)
nucleo-pesquisa-ind-pl = P/E
nucleo-pesquisa-ind-pl-formula = Market cap ÷ net income attributable to controlling shareholders over the last 12 months
nucleo-pesquisa-ind-pvp = P/B
nucleo-pesquisa-ind-pvp-formula = Market cap ÷ equity attributable to controlling shareholders (FII: price ÷ net asset value per unit)
nucleo-pesquisa-ind-ev_ebitda = EV/EBITDA
nucleo-pesquisa-ind-ev_ebitda-formula = Enterprise value ÷ EBITDA over the last 12 months (EBIT + depreciation and amortization)
nucleo-pesquisa-ind-ev_ebit = EV/EBIT
nucleo-pesquisa-ind-ev_ebit-formula = Enterprise value ÷ EBIT over the last 12 months
nucleo-pesquisa-ind-psr = P/Sales (PSR)
nucleo-pesquisa-ind-psr-formula = Market cap ÷ net revenue over the last 12 months
nucleo-pesquisa-ind-p_ativo = P/Assets
nucleo-pesquisa-ind-p_ativo-formula = Market cap ÷ total assets
nucleo-pesquisa-ind-dy = Dividend yield (12 months)
nucleo-pesquisa-ind-dy-formula = Dividends per share with an ex-date in the last 12 months ÷ price
nucleo-pesquisa-ind-payout = Payout
nucleo-pesquisa-ind-payout-formula = Dividends and JCP declared in the last 12 months ÷ net income
nucleo-pesquisa-ind-last_dividend = Last distribution
nucleo-pesquisa-ind-last_dividend-formula = Last distribution per unit (dividend calendar; without one, monthly DY × NAV per unit from the report)
nucleo-pesquisa-ind-roe = ROE
nucleo-pesquisa-ind-roe-formula = Net income over the last 12 months ÷ equity
nucleo-pesquisa-ind-roic = ROIC
nucleo-pesquisa-ind-roic-formula = EBIT over the last 12 months × (1 − statutory income tax rate) ÷ (equity + net debt)
nucleo-pesquisa-ind-roa = ROA
nucleo-pesquisa-ind-roa-formula = Net income over the last 12 months ÷ total assets
nucleo-pesquisa-ind-gross_margin = Gross margin
nucleo-pesquisa-ind-gross_margin-formula = Gross profit ÷ net revenue (12 months)
nucleo-pesquisa-ind-ebitda_margin = EBITDA margin
nucleo-pesquisa-ind-ebitda_margin-formula = EBITDA ÷ net revenue (12 months)
nucleo-pesquisa-ind-ebit_margin = EBIT margin
nucleo-pesquisa-ind-ebit_margin-formula = EBIT ÷ net revenue (12 months)
nucleo-pesquisa-ind-net_margin = Net margin
nucleo-pesquisa-ind-net_margin-formula = Net income ÷ net revenue (12 months)
nucleo-pesquisa-ind-net_debt_equity = Net debt / Equity
nucleo-pesquisa-ind-net_debt_equity-formula = (Gross debt − cash and investments) ÷ equity
nucleo-pesquisa-ind-net_debt_ebitda = Net debt / EBITDA
nucleo-pesquisa-ind-net_debt_ebitda-formula = (Gross debt − cash and investments) ÷ EBITDA over the last 12 months
nucleo-pesquisa-ind-current_ratio = Current ratio
nucleo-pesquisa-ind-current_ratio-formula = Current assets ÷ current liabilities
nucleo-pesquisa-ind-revenue_cagr_5y = Revenue CAGR 5 years
nucleo-pesquisa-ind-revenue_cagr_5y-formula = (Revenue of the latest fiscal year ÷ revenue 5 fiscal years earlier)^(1/5) − 1
nucleo-pesquisa-ind-earnings_cagr_5y = Earnings CAGR 5 years
nucleo-pesquisa-ind-earnings_cagr_5y-formula = (Net income of the latest fiscal year ÷ net income 5 fiscal years earlier)^(1/5) − 1
nucleo-pesquisa-ind-loss_years_10y = Years with a loss (10 years)
nucleo-pesquisa-ind-loss_years_10y-formula = Fiscal years with negative net income among the last 10
nucleo-pesquisa-ind-lpa = EPS
nucleo-pesquisa-ind-lpa-formula = Net income over the last 12 months ÷ shares outstanding (× shares per unit)
nucleo-pesquisa-ind-vpa = BVPS
nucleo-pesquisa-ind-vpa-formula = Equity ÷ shares outstanding (× shares per unit)
nucleo-pesquisa-ind-nav_per_share = Net asset value per unit
nucleo-pesquisa-ind-nav_per_share-formula = Net assets ÷ units issued (monthly report)
nucleo-pesquisa-ind-holders = Number of unitholders
nucleo-pesquisa-ind-holders-formula = Total unitholders in the latest monthly report
nucleo-pesquisa-ind-vacancy = Vacancy
nucleo-pesquisa-ind-vacancy-formula = Vacant area ÷ gross leasable area
nucleo-pesquisa-ind-basel = Basel ratio
nucleo-pesquisa-ind-basel-formula = Regulatory capital ÷ risk-weighted assets
nucleo-pesquisa-ind-graham = Graham price
nucleo-pesquisa-ind-graham-formula = √(22.5 × EPS × BVPS)
nucleo-pesquisa-ind-graham_upside = Upside to the Graham price
nucleo-pesquisa-ind-graham_upside-formula = Graham price ÷ price − 1
nucleo-pesquisa-ind-bazin = Bazin ceiling price
nucleo-pesquisa-ind-bazin-formula = Average dividends per share over the last N years ÷ minimum rate
nucleo-pesquisa-ind-bazin_upside = Upside to the Bazin price
nucleo-pesquisa-ind-bazin_upside-formula = Bazin ceiling price ÷ price − 1

## Indicators: periods, sources and formulas built on the fly

nucleo-pesquisa-periodo-12m = 12 months to { $data }
nucleo-pesquisa-periodo-saldo = balance on { $data }
nucleo-pesquisa-fonte-cotacao = { $fonte } quote from { $data }
nucleo-pesquisa-aviso-preco-justo = Estimates from classic formulas and the reported data; not investment advice.
nucleo-pesquisa-ind-periodo-informe = report for { $mes }
nucleo-pesquisa-ind-periodo-vp-informe = NAV from the report for { $mes }
nucleo-pesquisa-ind-periodo-data-com = ex-date { $data }
nucleo-pesquisa-ind-periodo-divida-ebitda = debt on { $divida }; 12-month EBITDA to { $ebitda }
nucleo-pesquisa-ind-periodo-exercicios = fiscal years { $de } to { $ate }
nucleo-pesquisa-ind-periodo-n-exercicios = { $n } fiscal year(s): { $de } to { $ate }
nucleo-pesquisa-ind-cotas-x-cotacao = { $cotas } units × price
nucleo-pesquisa-ind-acoes-x-cotacao = { $acoes } shares × price per share of the security looked up
nucleo-pesquisa-ind-on-pn-x-cotacao = ON shares × ON price + PN shares × PN price
nucleo-pesquisa-ind-formula-valor-mercado = Market cap = { $como }
nucleo-pesquisa-ind-formula-roic = EBIT over the last 12 months × (1 − { $aliquota }%) ÷ (equity + net debt)
nucleo-pesquisa-ind-formula-graham = √(22.5 × EPS × BVPS); 22.5 = maximum P/E of 15 × maximum P/B of 1.5
nucleo-pesquisa-ind-formula-bazin = Average dividends per share over the last N years ÷ minimum required return
nucleo-pesquisa-ind-formula-dy-dfc = Dividends and JCP paid in the last 12 months per share (cash flow statement) ÷ price
nucleo-pesquisa-ind-formula-payout-pagos = Dividends and JCP paid in the last 12 months ÷ net income
nucleo-pesquisa-ind-formula-pvp-fii = Price ÷ net asset value per unit (monthly report)
nucleo-pesquisa-ind-fonte-agenda = dividend calendar
nucleo-pesquisa-ind-fonte-informe-fii = FII monthly report (monthly DY × NAV per unit)
nucleo-pesquisa-ind-fonte-dividendos-dfc = dividends and JCP paid (cash flow statement) · { $documento }
nucleo-pesquisa-ind-fonte-dividendos-sec = dividends declared per share (SEC)
nucleo-pesquisa-ind-fonte-dividendos-dfc-acoes = dividends and JCP paid per share in the fiscal year (cash flow ÷ shares)
nucleo-pesquisa-ind-fonte-dfp-lucro = DFP (net income attributable to controlling shareholders)
nucleo-pesquisa-ind-media-anos = average of { $anos } year(s) with data ({ $pedidos } requested)
nucleo-pesquisa-ind-media-anos-agenda = the dividend calendar starts in { $ano }: average of { $anos } year(s) with data ({ $pedidos } requested)
nucleo-pesquisa-ind-premissa-lpa = EPS (12 months)
nucleo-pesquisa-ind-premissa-vpa = BVPS
nucleo-pesquisa-ind-premissa-acoes = Shares outstanding
nucleo-pesquisa-ind-premissa-taxa = Minimum rate
nucleo-pesquisa-ind-premissa-anos = Years
nucleo-pesquisa-ind-premissa-media = Annual average
nucleo-pesquisa-ind-premissa-dividendos-ano = Dividends per share in { $ano }

## Indicators: notes

nucleo-pesquisa-ind-nota-escala-acoes = The share count in { $documento } seems to be in { $escala ->
    [milhares] thousands
   *[unidades] units × 1,000
} (P/B of { $pvp } with the reported count): corrected by a factor of { $fator }.
nucleo-pesquisa-ind-nota-financeira = Financial institution: EBITDA, enterprise value, net debt, margins and current ratio don't apply.
nucleo-pesquisa-ind-nota-unit = Unit with { $acoes } shares: per-share values multiplied by the composition.
nucleo-pesquisa-ind-nota-classe = Fundamental indicators apply to stocks and FIIs; for this class there are only price and liquidity.
nucleo-pesquisa-ind-nota-sem-demonstracoes = No financial statements downloaded for this asset: run the asset analysis collection.
nucleo-pesquisa-ind-nota-acoes-sec = Shares outstanding from the cover of the latest filing (dei:EntityCommonStockSharesOutstanding).

## Indicators: why it is unavailable

nucleo-pesquisa-ind-indisponivel = unavailable
nucleo-pesquisa-ind-divisao-zero = division by zero
nucleo-pesquisa-ind-nao-se-aplica-financeiras = doesn't apply to financial institutions and insurers
nucleo-pesquisa-ind-nao-se-aplica-fii = doesn't apply to FIIs
nucleo-pesquisa-ind-sem-cotacao = no price
nucleo-pesquisa-ind-sem-cotas = no unit count in the monthly report
nucleo-pesquisa-ind-sem-acoes = no share count
nucleo-pesquisa-ind-unit-desconhecida = unknown unit composition
nucleo-pesquisa-ind-sem-rendimentos-informe = no distributions in the monthly report
nucleo-pesquisa-ind-sem-proventos = no dividends in the calendar or in the financial statements
nucleo-pesquisa-ind-lpa-negativo = EPS negative or zero (loss over the last 12 months)
nucleo-pesquisa-ind-vpa-negativo = BVPS negative or zero (negative equity)
nucleo-pesquisa-ind-taxa-positiva = the minimum rate must be positive
nucleo-pesquisa-ind-sem-historico-dividendos = no dividend history
nucleo-pesquisa-ind-sem-dividendos-periodo = no dividends in the period
nucleo-pesquisa-ind-sem-lucro-12m = no net income for the last 12 months
nucleo-pesquisa-ind-sem-receita-12m = no revenue for the last 12 months
nucleo-pesquisa-ind-sem-ebit-12m = no EBIT for the last 12 months
nucleo-pesquisa-ind-sem-da-12m = no depreciation and amortization for the last 12 months
nucleo-pesquisa-ind-sem-lucro-bruto-12m = no gross profit for the last 12 months
nucleo-pesquisa-ind-sem-patrimonio = no equity
nucleo-pesquisa-ind-sem-ativo-total = no total assets
nucleo-pesquisa-ind-sem-divida-bruta = no gross debt
nucleo-pesquisa-ind-sem-ativo-circulante = no current assets
nucleo-pesquisa-ind-sem-passivo-circulante = no current liabilities
nucleo-pesquisa-ind-sem-dividendos = no dividends
nucleo-pesquisa-ind-sem-volume = no traded volume
nucleo-pesquisa-ind-sem-valor-mercado = no market cap
nucleo-pesquisa-ind-lucro-zero = zero net income
nucleo-pesquisa-ind-patrimonio-negativo = negative equity
nucleo-pesquisa-ind-ebitda-negativo = EBITDA negative or zero
nucleo-pesquisa-ind-ebit-negativo = EBIT negative or zero
nucleo-pesquisa-ind-receita-zero = zero revenue
nucleo-pesquisa-ind-lucro-negativo = net income negative or zero
nucleo-pesquisa-ind-denominador-negativo = denominator negative or zero
nucleo-pesquisa-ind-capital-negativo = invested capital negative or zero
nucleo-pesquisa-ind-sem-exercicios = no fiscal years
nucleo-pesquisa-ind-poucos-exercicios = fewer than 6 fiscal years downloaded
nucleo-pesquisa-ind-ponta-negativa = negative or zero value at one of the ends
nucleo-pesquisa-ind-basileia-fora-cvm = the Basel ratio is not in the CVM files (it is in the notes to the financial statements and in the Central Bank's IF.data)
nucleo-pesquisa-ind-sem-informe = no monthly report
nucleo-pesquisa-ind-sem-vp-cota = no net asset value per unit
nucleo-pesquisa-ind-sem-rendimentos = no distributions
nucleo-pesquisa-ind-vacancia-fora = no vacancy in the CVM quarterly reports of the last 12 months

## Financial statement items

nucleo-pesquisa-item-revenue = Net revenue
nucleo-pesquisa-item-gross_profit = Gross profit
nucleo-pesquisa-item-ebit = EBIT (earnings before financial result and taxes)
nucleo-pesquisa-item-da = Depreciation, amortization and depletion
nucleo-pesquisa-item-net_income = Net income (controlling shareholders)
nucleo-pesquisa-item-cfo = Net cash from operating activities
nucleo-pesquisa-item-capex = Investments in property, plant and equipment and intangibles
nucleo-pesquisa-item-dividends_paid = Dividends and JCP paid
nucleo-pesquisa-item-dividends_declared = Dividends and JCP declared
nucleo-pesquisa-item-dps_declared = Dividends declared per share
nucleo-pesquisa-item-cogs = Cost of goods and services sold
nucleo-pesquisa-item-sga = Selling, general and administrative expenses
nucleo-pesquisa-item-fin_income = Financial income
nucleo-pesquisa-item-fin_expense = Financial expenses
nucleo-pesquisa-item-ebt = Earnings before income taxes
nucleo-pesquisa-item-income_tax = Income tax and social contribution (negative = expense)
nucleo-pesquisa-item-buybacks = Share buybacks (acquisition of treasury shares)
nucleo-pesquisa-item-share_issuance = Capital increase and share issuance
nucleo-pesquisa-item-total_assets = Total assets
nucleo-pesquisa-item-current_assets = Current assets
nucleo-pesquisa-item-cash = Cash, cash equivalents and short-term investments
nucleo-pesquisa-item-current_liabilities = Current liabilities
nucleo-pesquisa-item-total_liabilities = Total liabilities
nucleo-pesquisa-item-gross_debt = Gross debt (loans and financing)
nucleo-pesquisa-item-equity = Equity (controlling shareholders)
nucleo-pesquisa-item-shares_outstanding = Shares outstanding (excluding treasury shares)
nucleo-pesquisa-item-shares_on = Common shares outstanding
nucleo-pesquisa-item-shares_pn = Preferred shares outstanding
nucleo-pesquisa-item-receivables = Accounts receivable (current)
nucleo-pesquisa-item-inventories = Inventories (current)
nucleo-pesquisa-item-ppe = Property, plant and equipment
nucleo-pesquisa-item-intangible = Intangible assets
nucleo-pesquisa-item-suppliers = Suppliers (current)
nucleo-pesquisa-item-debt_st = Short-term loans and financing
nucleo-pesquisa-item-debt_lt = Long-term loans and financing
nucleo-pesquisa-item-leases = Leases outside loans and financing
nucleo-pesquisa-item-retained = Retained earnings (profit reserves + accumulated profits/losses)
nucleo-pesquisa-item-minority = Non-controlling interests
nucleo-pesquisa-item-nav = Fund net assets
nucleo-pesquisa-item-nav_per_share = Net asset value per unit
nucleo-pesquisa-item-fii_shares = Units issued
nucleo-pesquisa-item-holders = Number of unitholders
nucleo-pesquisa-item-dy_month = Monthly dividend yield on net assets (reported)
nucleo-pesquisa-item-income_per_share = Distribution per unit (monthly DY × NAV per unit)
nucleo-pesquisa-item-fii_total_assets = Fund total assets
nucleo-pesquisa-item-fii_liabilities = Fund total liabilities
nucleo-pesquisa-item-fii_acq_obligations = Obligations for property acquisitions
nucleo-pesquisa-item-fii_securitization = Obligations for receivables securitization
nucleo-pesquisa-item-fii_real_estate = Rights to real estate
nucleo-pesquisa-item-fii_cri = CRI and CRA in the portfolio
nucleo-pesquisa-item-fii_fii_shares = Units of other FIIs in the portfolio
nucleo-pesquisa-item-fii_liquidity = Assets held for liquidity needs
nucleo-pesquisa-item-fii_rent_receivable = Rent receivable
nucleo-pesquisa-item-fii_admin_fee_pct = Monthly management fee on net assets
nucleo-pesquisa-item-fii_equity_return_month = Monthly return on net assets
nucleo-pesquisa-item-fii_vacancy_financial = Financial vacancy (weighted by revenue)
nucleo-pesquisa-item-fii_vacancy_physical = Physical vacancy (weighted by area)
nucleo-pesquisa-item-fii_default_rate = Default rate (weighted by revenue)
nucleo-pesquisa-item-fii_top_property_share = Largest property in the fund's revenue
nucleo-pesquisa-item-fii_properties = Number of properties
nucleo-pesquisa-item-fii_top_tenant_share = Largest tenant in the fund's revenue (approximation)
nucleo-pesquisa-item-fii_walt_years = Weighted average lease term (WALT, years)
nucleo-pesquisa-item-fii_undetermined_share = Revenue from leases with no fixed term
nucleo-pesquisa-item-fii_recurring_result = Net financial result of the quarter
nucleo-pesquisa-item-fii_declared_income = Distributions declared in the quarter
nucleo-pesquisa-item-fii_cri_top_issuer_share = Largest issuer in the CRI/CRA (approximation of the debtor)
nucleo-pesquisa-item-desconhecido = Item
nucleo-pesquisa-erro-tipo-periodo = invalid period type: { $tipo }

## Analysis settings

nucleo-pesquisa-aviso-sec = The SEC requires identification with a contact email: replace "contato@exemplo" with your email in Settings › Data sources (e.g. "Treasury/0.1 your.name@email.com").
nucleo-pesquisa-erro-bazin-taxa = the Bazin minimum rate must be positive (e.g. 0.06 or 6 for 6%)
nucleo-pesquisa-erro-bazin-anos = the Bazin average years go from 1 to 10
nucleo-pesquisa-erro-sec-ua-vazio = enter the SEC User-Agent (app name and email)
nucleo-pesquisa-erro-sec-ua-invalido = invalid User-Agent
nucleo-pesquisa-erro-sec-ua-email = the SEC requires a contact email in the User-Agent (e.g. Treasury/0.1 your.name@email.com)

## Collection: B3 and brapi

nucleo-pesquisa-erro-cotahist-sem-txt = COTAHIST without a .TXT file
nucleo-pesquisa-erro-b3-companhia = B3 response (company) in an unexpected format: { $erro }
nucleo-pesquisa-erro-b3-fundo = B3 response (fund) in an unexpected format: { $erro }
nucleo-pesquisa-erro-b3-historico = B3 dividend history in an unexpected format: { $erro }
nucleo-pesquisa-erro-b3-vazia = B3 responded with no content
nucleo-pesquisa-erro-304 = 304 response without a conditional request
nucleo-pesquisa-erro-b3-api = B3 API failure
nucleo-pesquisa-b3-log-proventos = { $n } dividend(s)
nucleo-pesquisa-b3-log-proventos-eventos = { $proventos } dividend(s), { $eventos } stock event(s)
nucleo-pesquisa-b3-passo-historico = B3 dividend history { $codigo }
nucleo-pesquisa-b3-passo-resumo = B3 listed companies and funds · dividends and events ({ $feitos } of { $total } issuers and funds, { $pedidos } request(s))
nucleo-pesquisa-b3-parou = B3 didn't respond { $vezes } times in a row; the rest is left for the next collection
nucleo-pesquisa-b3-nada-a-atualizar = nothing to update: all checked recently
nucleo-pesquisa-erro-brapi-invalida = invalid response from brapi: { $erro }
nucleo-pesquisa-erro-brapi-sem-resultados = brapi: response without results

## CVM files and asset data

nucleo-pesquisa-erro-zip = invalid zip file: { $erro }
nucleo-pesquisa-erro-coluna-ausente = column { $coluna } missing
nucleo-pesquisa-erro-ipe-sem-csv = IPE CSV not found in the zip
nucleo-pesquisa-liquidez-cotahist = average of { $n } session(s), B3 COTAHIST
nucleo-pesquisa-liquidez-cotacoes = average of { $n } session(s) (price × volume, { $fonte })
nucleo-pesquisa-erro-asset-ou-ticker = enter asset_id or ticker
nucleo-pesquisa-erro-fora-do-universo = { $ticker } in the downloaded universe (run the asset analysis collection)

## Dividends: source and check against B3

nucleo-pesquisa-div-fonte-b3 = B3 (official), cash dividends of listed companies and funds
nucleo-pesquisa-div-fonte-agenda = dividend calendar ({ $fontes })
nucleo-pesquisa-div-fonte-b3-diverge = { $b3 }; the calendar ({ $fontes }) differs from B3
nucleo-pesquisa-div-fonte-lancados = recorded dividends (gross ÷ quantity)
nucleo-pesquisa-div-nota-ja-corrige = { $fonte } already adjusts the amount for splits and bonus shares after the ex-date
nucleo-pesquisa-div-nota-nao-corrige = { $fonte } doesn't adjust the amount for splits and bonus shares after the ex-date
nucleo-pesquisa-div-nota-houve-evento = there was a split, reverse split or bonus share issue after the ex-date
nucleo-pesquisa-div-nota-sem-par = { $tipos } with no match in B3
nucleo-pesquisa-div-nota-sem-b3 = No B3 dividends for this security (run the asset analysis collection or check whether B3 knows it).
nucleo-pesquisa-div-nota-sem-agenda = No dividend calendar from Yahoo or Fundamentus to compare.
nucleo-pesquisa-div-nota-b3-desde = B3 covers from { $data } (the full history is only downloaded for issuers of registered assets).
nucleo-pesquisa-div-nota-valores-epoca = The calendar's 12-month total ({ $agenda }) matches B3's with the amounts at the time ({ $b3 }); the difference comes from adjusting for splits and bonus shares, which the DY needs (today's price).
nucleo-pesquisa-div-nota-divergencia = Material difference: the indicators use B3 (official source).
nucleo-pesquisa-erro-sem-codigo = the asset has no trading code
nucleo-pesquisa-erro-conferencia-br = the check against B3 applies to Brazilian stocks and FIIs

## Fundamentus

nucleo-pesquisa-erro-ticker-fundamentus = invalid ticker for Fundamentus: { $ticker }

## Central Bank IF.data

nucleo-pesquisa-ifdata-convencao = half-year to date (March/September: 3 months; June/December: 6 months)
nucleo-pesquisa-ifdata-relatorio-cadastro = registry
nucleo-pesquisa-ifdata-relatorio-resumo = summary
nucleo-pesquisa-ifdata-relatorio-ativo = assets
nucleo-pesquisa-ifdata-relatorio-resultado = income statement
nucleo-pesquisa-ifdata-relatorio-capital = capital
nucleo-pesquisa-ifdata-relatorio-carteira = loan portfolio by risk
nucleo-pesquisa-ifdata-relatorio = report
nucleo-pesquisa-erro-ifdata-invalida = invalid response from IF.data: { $erro }
nucleo-pesquisa-ifdata-fonte-desligada = the Central Bank source is turned off in Settings › Data sources: turn it on and the daily collection downloads IF.data
nucleo-pesquisa-ifdata-nao-baixado = the Central Bank IF.data hasn't been downloaded yet (the asset analysis collection runs on its own once a day; the result of each source is in Assets › Analysis data)
nucleo-pesquisa-ifdata-fora-do-cadastro = CNPJ { $cnpj } is not in the Central Bank IF.data registry (quarter { $trimestre }): the company is not a bank that reports to the Central Bank
nucleo-pesquisa-ifdata-sem-valores = the Central Bank IF.data had no values for { $banco } ({ $codigo }) up to quarter { $trimestre }

## Network

nucleo-pesquisa-erro-post = POST not supported
nucleo-pesquisa-erro-servidor-recusou = the server refused access (HTTP { $status }) after { $tentativas } attempt(s); try again later
nucleo-pesquisa-erro-token-recusado = access refused (HTTP { $status }): check the token
nucleo-pesquisa-erro-http-tentativas = HTTP { $status } after { $tentativas } attempt(s)

## Documents for the last 12 months

nucleo-pesquisa-doc-exercicio = fiscal year: { $documento }
nucleo-pesquisa-doc-ltm = { $exercicio } + year to date to { $ate } − year to date to { $ate_anterior } ({ $documento }; { $documento_anterior })
nucleo-pesquisa-doc-ultimo-exercicio = latest fiscal year: { $documento }

## Dividend radar

nucleo-pesquisa-radar-poucos-proventos = not enough history (fewer than 2 dividends in 24 months)
nucleo-pesquisa-radar-insuficiente = not enough history

## Rankings and screener

nucleo-pesquisa-erro-indicador-desconhecido = unknown indicator: { $indicador } (see research.catalog)
nucleo-pesquisa-erro-ordenar-desconhecido = unknown indicator to sort by: { $indicador }
nucleo-pesquisa-erro-campo-desconhecido = unknown field in the filter: { $campo } (indicators from research.catalog, or { $campos })
nucleo-pesquisa-erro-filtro = filter { $campo }: { $motivo }
nucleo-pesquisa-filtro-comparacao = size comparisons only apply to indicators
nucleo-pesquisa-filtro-numero = enter a number in `value`
nucleo-pesquisa-filtro-between = `between` needs `value: [minimum, maximum]` on an indicator
nucleo-pesquisa-filtro-in = `in`/`not_in` need a list in `value`
nucleo-pesquisa-filtro-contains = `contains` needs text on a text field
nucleo-pesquisa-filtro-value = enter `value`
nucleo-pesquisa-erro-grupo-vazio = empty filter group
nucleo-pesquisa-filtro-op-entre = between
nucleo-pesquisa-filtro-op-em = in
nucleo-pesquisa-filtro-op-fora-de = not in
nucleo-pesquisa-filtro-op-contem = contains
nucleo-pesquisa-filtro-op-disponivel = available
nucleo-pesquisa-filtro-op-indisponivel = unavailable
nucleo-pesquisa-filtro-e = AND
nucleo-pesquisa-filtro-ou = OR
nucleo-pesquisa-filtro-todos = all securities
nucleo-pesquisa-ref-tela = screener { $id }
nucleo-pesquisa-erro-tela-nome = enter the screener name
nucleo-pesquisa-erro-tela-duplicada = a screener named { $nome } already exists

## SEC and local database

nucleo-pesquisa-erro-sec-cadastro = invalid SEC submissions: { $erro }
nucleo-pesquisa-erro-sec-tickers = invalid company_tickers.json: { $erro }
nucleo-pesquisa-erro-sec-companyfacts = invalid companyfacts: { $erro }
nucleo-pesquisa-doc-sec = { $formulario } { $numero } (filed on { $data })
nucleo-pesquisa-erro-valor-banco = invalid value: { $valor }
nucleo-pesquisa-erro-cotacao-banco = invalid price in the database: { $cotacao }

## Analysis collection: steps and warnings

nucleo-pesquisa-passo-fii-mensal = CVM FII monthly report { $ano }
nucleo-pesquisa-passo-fii-trimestral = CVM FII quarterly report { $ano }
nucleo-pesquisa-passo-sec-email = SEC EDGAR: enter the contact email
nucleo-pesquisa-passo-sec-cadastro = SEC submissions { $ticker }
nucleo-pesquisa-erro-sec-sem-ticker = { $ticker } is not on the SEC ticker list
nucleo-pesquisa-passo-ifdata-sondagem = Central Bank IF.data: probe for { $periodo }
nucleo-pesquisa-passo-parte = part { $parte }/{ $partes }
nucleo-pesquisa-aviso-acoes-em-milhares = { $n } company(ies) reported the share count in thousands in some document (corrected using earnings per share): { $companhias }

## Collection: IF.data name

nucleo-pesquisa-passo-ifdata = Central Bank IF.data

## brapi.dev: plan and quota

nucleo-pesquisa-brapi-plano-gratuito = brapi.dev: the indicator and dividend cross-check is for paid plans; on the free plan it stays off, without spending quota.
nucleo-pesquisa-brapi-limite = brapi.dev: this month's usage limit was reached; the rest waits for the renewal or a higher limit in Settings › Data sources.

nucleo-pesquisa-erro-rede-bloqueada = this computer's network blocked the secure connection to { $site } (company security filter, antivirus, proxy or DNS blocking the site); it is neither the site nor Treasury. Allow the site in the filter or use another network. Detail: { $detalhe }

# Indicadores dos métodos do explorador de mercado
nucleo-pesquisa-ind-earnings_yield = EBIT/EV
nucleo-pesquisa-ind-earnings_yield-formula = Last 12 months' EBIT ÷ enterprise value (the inverse of EV/EBIT; Magic Formula)
nucleo-pesquisa-ind-magic_roc = ROC (Magic Formula)
nucleo-pesquisa-ind-magic_roc-formula = Last 12 months' EBIT ÷ (net working capital + property, plant and equipment)
nucleo-pesquisa-ind-fcf_yield = Free cash flow / market cap
nucleo-pesquisa-ind-fcf_yield-formula = (Operating cash flow − capital expenditure, 12 months) ÷ market cap
nucleo-pesquisa-ind-peg = PEG
nucleo-pesquisa-ind-peg-formula = P/E ÷ annual earnings growth over 5 years (in percentage points; Peter Lynch)
nucleo-pesquisa-ind-eps_growth_10y = Earnings growth over 10 years
nucleo-pesquisa-ind-eps_growth_10y-formula = Average earnings of the last 3 years ÷ average earnings 10 to 8 years ago − 1 (defensive Graham; total earnings, not per share)
nucleo-pesquisa-ind-dividend_years = Consecutive years with dividends
nucleo-pesquisa-ind-dividend_years-formula = Consecutive full years, up to the last one, with some dividend paid (up to 20)
nucleo-pesquisa-ind-dividend_cagr_5y = Dividend growth over 5 years
nucleo-pesquisa-ind-dividend_cagr_5y-formula = Compound annual growth of dividends per share or unit between the year 5 years ago and the last full year
nucleo-pesquisa-ind-sem-imobilizado = No property, plant and equipment in the last balance sheet
nucleo-pesquisa-ind-sem-fco-12m = No operating cash flow for the last 12 months
nucleo-pesquisa-ind-fonte-peg = P/E and 5-year earnings growth
nucleo-pesquisa-ind-peg-negativo = Negative P/E or earnings growth: PEG does not apply
nucleo-pesquisa-ind-peg-sem-dados = No P/E or no 5-year earnings growth

# Score em lote do universo
nucleo-pesquisa-ind-score = Score
nucleo-pesquisa-ind-score-formula = Treasury health score (0 to 100), computed during the collection for the whole universe
nucleo-pesquisa-ind-health = Health (score)
nucleo-pesquisa-ind-health-formula = Business health sub-score, without valuation or momentum (0 to 100)
nucleo-pesquisa-ind-attractiveness = Attractiveness (score)
nucleo-pesquisa-ind-attractiveness-formula = Price attractiveness sub-score: valuation and momentum (0 to 100)
nucleo-pesquisa-ind-confidence = Score confidence
nucleo-pesquisa-ind-confidence-formula = Data coverage × freshness × peer group size (0 to 1)
nucleo-pesquisa-ind-piotroski = Piotroski F-Score
nucleo-pesquisa-ind-piotroski-formula = Piotroski criteria met (0 to 9)
nucleo-pesquisa-ind-altman_z = Altman Z''
nucleo-pesquisa-ind-altman_z-formula = Altman Z'' for emerging markets (above 2.6: safe zone)
nucleo-pesquisa-ind-beneish = Beneish M-Score
nucleo-pesquisa-ind-beneish-formula = Beneish M-Score (above −1.78 suggests a risk of earnings manipulation)
nucleo-pesquisa-ind-momentum_12_1 = Momentum 12-1
nucleo-pesquisa-ind-momentum_12_1-formula = Return of the last 12 months excluding the last month
nucleo-pesquisa-ind-volatility_1y = 1-year volatility
nucleo-pesquisa-ind-volatility_1y-formula = Annualized standard deviation of 1 year of daily returns
nucleo-pesquisa-aviso-score-lote = Universe score not computed: { $erro }
nucleo-pesquisa-motivo-observar = Watch { $ticker } from Market
nucleo-pesquisa-motivo-deixar-observar = Stop watching from Market
nucleo-pesquisa-aviso-alerta-tela = Screen alerts not checked: { $erro }
nucleo-pesquisa-documento-link = The document has no official CVM link to download.
nucleo-pesquisa-documento-sumiu = CVM couldn't find the document's file.
nucleo-pesquisa-documento-grande = The document's file is too large to read (max. 25 MB).
nucleo-pesquisa-documento-sem-texto = Couldn't read any text in the document.
nucleo-pesquisa-documento-nao-encontrado = Document not found among the captured ones (use list_corporate_documents).
nucleo-pesquisa-ind-periodo-trimestral = quarterly report to { $data }
nucleo-pesquisa-div-fonte-b3-sem-rendimentos = B3 (official): no cash distribution from the fund in the last 12 months
nucleo-pesquisa-ind-sem-rendimentos-b3 = the B3 shows no distribution from the fund in the last 12 months
nucleo-pesquisa-ind-fonte-declarados-fii = distributions declared in the CVM quarterly reports (4 quarters to { $data })
nucleo-pesquisa-ind-formula-dy-fii-declarados = Distributions declared in the last 4 quarterly reports ÷ units issued ÷ price (estimate: no B3 distributions)
nucleo-pesquisa-ind-formula-dy-fii-meses = Sum of the distribution per unit in the monthly reports (monthly DY × NAV per unit) ÷ price (estimate: no B3 distributions). Months: { $meses }
nucleo-pesquisa-div-fonte-b3-amortizacao = { $fonte }; distributions before the amortizations of { $datas } scaled to today's unit (without the capital returned)
