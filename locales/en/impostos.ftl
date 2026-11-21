# Screen 11 · Taxes: estimate of the taxes paid in the year, scenarios, spending by category,
# recorded taxes, work income, income statements, property and sources (docs/taxes.md,
# docs/i18n-design.md). Revision: AI draft, not yet reviewed by a native speaker.

## Screen

impostos-periodo = { $de } to { $ate }
impostos-descricao = How much of the year went to taxes, estimated from your data: spending, work income, property and recorded taxes. Every number shows where it came from.
impostos-estimando = Estimating this year's taxes
impostos-aviso-titulo = An estimate, not an official figure
impostos-desatualizados = Outdated data
impostos-desatualizados-texto = The { $dados } figures were last reviewed on { $data } (version { $versao }), more than 6 months ago. Rates and tables may have changed.
impostos-esboco-titulo = { $dados }: draft
impostos-esboco-texto = The data for this country is still partial: only some of the taxes are estimated.
impostos-aviso-padrao = This is an estimate based on average tax burdens per spending category and the income tax rules in force. Actual amounts may vary with the state, how businesses are taxed and specific tax breaks.

## Scenarios

impostos-cenario = Scenario
impostos-cenario-estimativa = Estimate scenario
impostos-cenario-conservador = Conservative
impostos-cenario-conservador-dica = The lowest reasonable value in each group: the product with the lowest tax burden or the reduced rate.
impostos-cenario-provavel = Likely
impostos-cenario-provavel-dica = The middle value in each group: the median product or the latest official figure.
impostos-cenario-elevado = High
impostos-cenario-elevado-dica = The highest reasonable value in each group: the product with the highest tax burden or the standard rate.
impostos-tres-cenarios = Three scenarios
impostos-cenarios = Scenarios
impostos-cenarios-nota = The scenarios only change the tax on spending; income, INSS (social security), recorded taxes and IPVA (vehicle tax) are the same in all three.

## Estimate parts and tax groups

impostos-parte-consumo = Spending
impostos-parte-ir = Income tax
impostos-parte-previdencia = Social security (INSS)
impostos-parte-patrimonio = Property
impostos-parte-proventos = Investment income
impostos-parte-outros = Other taxes
impostos-grupo-ir-pago = Income tax paid
impostos-grupo-iof-pago = IOF paid (financial transactions tax)
impostos-grupo-iptu = IPTU (property tax)
impostos-grupo-ipva = IPVA (vehicle tax) and licensing
impostos-grupo-outros-pagos = Other taxes paid
impostos-grupo-financeiro = Financial (debts, interest, investments)
impostos-grupo-transferencias = Transfers to people (alimony, allowance, donations)
impostos-sem-grupo = No group
impostos-origem-usuario = your choice
impostos-origem-padrao = default
impostos-origem-pai = from parent group
impostos-origem-cupons = tax receipts

## Outside the estimate: reasons

impostos-motivo-sem-categoria = Uncategorized
impostos-motivo-sem-categoria-dica = Uncategorized transactions have no tax group. Categorize them in Transactions so they are included.
impostos-motivo-sem-grupo = No tax group
impostos-motivo-sem-grupo-dica = Categories you created that are not in the starter set and do not inherit a group from their parent. Choose a group for each one.
impostos-motivo-fora-consumo = Not spending
impostos-motivo-fora-consumo-dica = Debts, interest, investments and transfers to people are not purchases of goods or services: there is no built-in tax to estimate.
impostos-motivo-sem-dado = No reliable data
impostos-motivo-sem-dado-dica = There is no reliable source for the tax built into these expenses (rent, health plan, services without detail…). They stay visible but outside the estimate; you can set your own rate.

## Rates

impostos-pontos = { $valor } pp
impostos-sem-dado = no data
impostos-base-isento = exempt
impostos-base-iva-padrao = country's standard VAT
impostos-base-iva-reduzido = country's reduced VAT
impostos-base-iva-super-reduzido = country's super-reduced VAT
impostos-base-vendas = state sales tax
impostos-regiao-uf = State (UF)
impostos-regiao-pais-membro = Member state
impostos-regiao-estado = State

## Chart and breakdown

impostos-grafico-inclui-estimados = Includes estimated values
impostos-grafico-lancados = Recorded or reported values
impostos-imposto-estimado = Estimated tax
impostos-composicao = Breakdown by type
impostos-composicao-vazia = No tax estimated in this period.
impostos-por-tipo = Tax by type · { $cenario }
impostos-por-tipo-aria = Estimated tax by type, { $cenario } scenario
impostos-composicao-fonte = Source: the app's estimate with the country data · ≈ includes estimated values
impostos-tipo = Type
impostos-valor = Amount
impostos-total = Total

## Summary

impostos-estimativa-de = { $ano } estimate
impostos-periodo-rotulo = Period
impostos-pais = Country
impostos-nao-informado = not set
impostos-dados = Data
impostos-total-estimado = Estimated total
impostos-total-estimado-cenario = Estimated total · { $cenario }
impostos-origem-estimativa = Estimate with the { $dados } data (version { $versao })
impostos-origem-estimativa-nota = Sum of the scenario's tax on spending, work income, property, recorded taxes and income tax withheld on investment income.
impostos-faixa = range from { $de } (conservative) to { $ate } (high)
impostos-pct-renda = % of income
impostos-pct-renda-dica = total tax ÷ gross income
impostos-pct-consumo = % of spending
impostos-pct-consumo-dica = on { $valor } of spending
impostos-sem-consumo = no spending estimated
impostos-dias-trabalho = Days of work to pay taxes
impostos-n-dias = { $n ->
    [one] { $n } day
   *[other] { $n } days
}
impostos-dias-ate = until { $data }
impostos-dias-ate-media = until { $data } · reference: the country's tax burden equals { $dias } days (Tesouro Nacional, { $ano })
impostos-renda-bruta = Estimated gross income
impostos-renda-bruta-origem = Income recorded in the period plus tax withheld at source (salary and investment income)
impostos-renda-bruta-dica = income + withheld at source
impostos-referencia-pais = Country reference
impostos-em-ano = in { $ano }
impostos-dados-pais = Country data
impostos-referencia-texto = Revenue from all of the country's taxes divided by GDP. A macroeconomic measure shown for context: it cannot be compared directly with the family's estimate.
impostos-referencia-dias = { $rotulo }: { $dias } days in { $ano } (until { $ate }), with a burden of { $carga } of GDP.
impostos-sem-referencia = The { $dados } data does not include the country's tax burden.
impostos-melhorar = To improve the estimate

## Controls

impostos-ano = Year
impostos-erro-trocar-pais = Couldn't change the country
impostos-erro-trocar-regiao = Couldn't change { $regiao }
impostos-erro-trocar-dependentes = Couldn't change the dependents
impostos-pais-dados = Data country
impostos-esboco = draft
impostos-sem-dados-estado = No data by state
impostos-nao-informado-opcao = Not set
impostos-dependentes = Dependents
impostos-dependentes-dica = Dependents for the income tax estimated from the net salary

## Sources and methodology

impostos-fontes = Sources
impostos-fonte-publicada = Published: { $publicacao } · Accessed on { $consulta }
impostos-fonte-consultada = Accessed on { $consulta }
impostos-fontes-metodologia = Sources and methodology
impostos-fontes-descricao = How the { $dados } estimate is made and where each number comes from.
impostos-versao = Version
impostos-atualizado-em = Updated on
impostos-cobertura = Coverage
impostos-fontes-desatualizados = These figures were last reviewed on { $data }, more than 6 months ago. Rates and tables may have changed since then.
impostos-esboco-maiusc = Draft
impostos-fontes-esboco = The { $dados } data is still partial: only some of the taxes are estimated.
impostos-secao-aviso = Notice
impostos-metodologia = Methodology
impostos-ajuste-estado = State adjustment
impostos-ajuste-estado-texto = The spending tables use a reference ICMS (state sales tax) of { $icms }. For goods at the standard rate, the difference between the chosen state's standard rate and this reference is added.
impostos-renda-trabalho = Work income
impostos-a-partir-de = from { $data }
impostos-contexto = Context
impostos-contexto-texto = { $rotulo }: { $valor } in { $ano }.
impostos-fontes-n = Sources ({ $n })
impostos-ver-fontes = See sources and methodology

## Outside the estimate

impostos-fora = Outside the estimate
impostos-gasto-fora = Spending left out
impostos-gasto-estimado = Spending estimated
impostos-fora-vazio = All spending in the period is in the estimate.
impostos-escolher-grupo = Choose group
impostos-escolher-grupo-de = Choose group for { $categoria }
impostos-aliquota-propria = Own rate
impostos-aliquota-propria-de = Own rate for { $categoria }
impostos-categorizar = Categorize in Transactions

## Category groups and adjusting a category

impostos-padrao-app = App default
impostos-padrao-app-grupo = App default: { $grupo }
impostos-padrao-app-longo = App default (by name or parent group)
impostos-grupos-categorias = Category groups
impostos-grupos-descricao = Each expense category falls into a tax group. The default comes from the category name (or its parent group); change the group or set your own rate when you know more about your spending. Applies to every year.
impostos-filtrar-categorias = Filter categories
impostos-filtrar-placeholder = Filter by category or group
impostos-carregando-categorias = Loading categories
impostos-categoria = Category
impostos-grupo = Group
impostos-acoes = Actions
impostos-filtro-vazio = No category matches this filter.
impostos-erro-salvar = Couldn't save
impostos-aliquota-invalida = Invalid rate
impostos-aliquota-faixa = Use a number from 0 to 100 (less than 100).
impostos-sem-grupo-fora = no group: outside the estimate
impostos-grupo-de = Group for { $categoria }
impostos-do-pais = country's
impostos-voltar-padrao = Reset to default
impostos-voltar-padrao-de = Reset { $categoria } to default
impostos-imposto-pago = tax paid
impostos-fora-consumo = not spending
impostos-categoria-padrao = Category reset to default
impostos-grupo-trocado = Group changed
impostos-grupo-padrao = Group reset to default
impostos-aliquota-pais = Country rate
impostos-aliquota-salva = Own rate saved
impostos-trocar-grupo-titulo = Change tax group
impostos-trocar-grupo-descricao = { $categoria }. The group says which tax applies to spending in this category; the new group applies to every year.
impostos-hoje = Now: { $grupo } ({ $origem })
impostos-aliquota-descricao = { $categoria }. The share of the price you count as tax, instead of the country table. Applies in all three scenarios.
impostos-usar-pais = Use the country rate
impostos-aliquota-pct = Rate (% of price)
impostos-aliquota-exemplo = e.g. 18.5
impostos-base-de = Basis · { $grupo }
impostos-base-grupo = Group basis
impostos-produtos-servicos = Products and services
impostos-icms-modal = Goods at the standard ICMS rate: they get the chosen state's adjustment (the difference from the { $icms } reference).
impostos-fontes-grupo = Group sources
impostos-valor-como-esta = { $grupo }: the amount recorded in the category counts as is.
impostos-categoria-sem-grupo = The category has no tax group.

## Spending by category

impostos-menu-trocar-grupo = Change group…
impostos-menu-aliquota = Own rate…
impostos-menu-ver-base = See basis and sources
impostos-ja-padrao = The category already uses the default group and rate.
impostos-aliquota-propria-nota = Own rate: applies in all three scenarios.
impostos-ajuste-icms = Includes the state ICMS adjustment: { $pontos }.
impostos-aliquota-de = Rate for { $categoria }: { $valores }
impostos-propria = own
impostos-consumo-categoria = Spending by category
impostos-consumo-nota = The year's spending on an accrual basis × the share of the price that is tax in the category's group. Hover over the rate to see the three scenarios; use the row menu (right-click) to change the group or set your own rate.
impostos-consumo-vazio = No spending with a tax group in this period.
impostos-acoes-de = Actions for { $categoria }
impostos-gasto = Spent
impostos-aliquota = Rate
impostos-aliquota-cenario = Rate in the { $cenario } scenario
impostos-do-consumo = Of spending
impostos-ver-base-de = See basis for { $categoria }
impostos-trocar-grupo-de = Change group for { $categoria }

## Recorded taxes, work income, property and investment income

impostos-lancados = Recorded taxes
impostos-lancados-nota = What was recorded in tax categories counts at the recorded amount, the same in all three scenarios.
impostos-lancados-vazio = No tax recorded as an expense in this period.
impostos-origem-holerites = payslips
impostos-origem-informado = reported
impostos-origem-estimado = ≈ estimated
impostos-renda-trabalho-nota = Without an income statement, each month's gross pay is estimated from the net salary received in the Salary, Vacation and 13º (13th salary) categories, using the IR and INSS tables in force that month, without other deductions (health plan, alimony). Employer contributions are not included.
impostos-renda-trabalho-nota-exterior = Without payslips, each month's gross pay is estimated from the net salary received in the Salary category (and the country's extra payments), using the payroll rules in force that year, without other deductions. Employer contributions are not included.
impostos-informe-inss = INSS
impostos-renda-trabalho-vazio = No salary received and no income statement in this period.
impostos-membro = Member
impostos-sem-membro = No member
impostos-origem = Source
impostos-meses = Months
impostos-liquido-recebido = Net received
impostos-bruto = Gross
impostos-ir-retido = IR withheld
impostos-regras-usadas = Rules used in the estimate
impostos-patrimonio-nota = Recorded IPTU and IPVA count at their amount. With no IPTU recorded, the bill registered on the property (Portfolio › Physical assets) is used. With no IPVA recorded in the year, it is estimated from each vehicle's value at the end of the period and the state rate.
impostos-patrimonio-lancados = IPTU and IPVA come from your transactions (see Recorded taxes).
impostos-patrimonio-vazio = No property tax recorded or estimated in this period.
impostos-imposto-estimado-de = Estimated { $imposto } · { $bem }
impostos-valor-vezes = Value { $valor } × { $aliquota }
impostos-valor-venal-vezes = Assessed value { $valor } × { $aliquota }
impostos-ir-proventos = IR withheld on investment income
impostos-retido-fonte = Withheld at source
impostos-retido-fonte-origem = Investment income recorded in Portfolio (JCP, foreign dividends)
impostos-retido-texto = Tax withheld on investment income paid in the period, in the household currency.
impostos-ver-proventos = See investment income

## Income statements

impostos-informe-alterado = Income statement updated
impostos-informe-lancado = Income statement added
impostos-informe-editar-titulo = Edit income statement · { $ano }
impostos-informe-lancar-titulo = Add income statement · { $ano }
impostos-informe-descricao = The amounts from the income statement (comprovante de rendimentos) the payer gives you early the following year (boxes 3 and 5).
impostos-fonte-pagadora = Payer
impostos-sem-membro-casa = No member (household)
impostos-rendimentos-tributaveis = Taxable income
impostos-sem-13 = excluding the 13º
impostos-previdencia-oficial = Official social security (INSS)
impostos-ir-retido-fonte = IR withheld at source
impostos-sem-ir-13 = excluding the 13º's
impostos-13-salario = 13º salary
impostos-ir-13 = IR withheld on the 13º
impostos-observacoes = Notes
impostos-da-casa = Household
impostos-informes-de = { $ano } income statements
impostos-informes-titulo = Income statements · { $ano }
impostos-informes-texto = With an income statement, the member's work income uses its amounts instead of the estimate.
impostos-lancar-informe = Add income statement
impostos-editar-menu = Edit…
impostos-excluir = Delete
impostos-acoes-informe = Actions for the { $fonte } income statement
impostos-13-bruto = 13º gross
impostos-ir-do-13 = IR on the 13º
impostos-editar-informe = Edit the { $fonte } income statement
impostos-excluir-informe = Delete income statement
impostos-excluir-informe-texto = The { $fonte } income statement ({ $ano }) leaves the estimate; the income goes back to being estimated from the net amount. You can undo this in the history.
impostos-informes-vazio = No income statement added for { $ano }.

## Só no Brasil: para onde vai o imposto e a arrecadação da cidade
impostos-destino-titulo = Where the tax goes
impostos-destino-texto = The estimated tax, split the way general government (federal, state and municipal) spent in { $ano }, by function.
impostos-destino-ressalva = It's a proportional split, not where each real went: the government is also funded by debt and other revenue, and the general public services line includes debt interest.
impostos-estado = State
impostos-cidade = City
impostos-cidade-titulo = Your city's tax revenue
impostos-cidade-titulo-de = Tax revenue of { $cidade }
impostos-cidade-escolha-texto = Choose the city where the family lives to see how much the municipality collected in taxes. The list of cities comes from IBGE.
impostos-cidade-escolha-estado = Choose the state
impostos-cidade-escolha = Choose the city
impostos-cidade-carregando = Loading…
impostos-cidade-salvar = Save
impostos-cidade-cancelar = Cancel
impostos-cidade-trocar = Change city
impostos-cidade-erro-salvar = Couldn't save the city
impostos-cidade-erro-buscar = Couldn't fetch the revenue
impostos-cidade-por-habitante = Per inhabitant, over 12 months
impostos-cidade-habitantes = { $n } inhabitants
impostos-cidade-total = Municipal taxes over 12 months
impostos-cidade-periodo = up to { $mes }
impostos-cidade-fonte = Tesouro Nacional (Siconfi)
impostos-cidade-por-tributo = By tax
impostos-cidade-por-hab = per inh.
impostos-cidade-ressalva = It's what the city collected (IPTU, ISS, ITBI and fees), not what each resident pays: companies pay a large part, and federal and state taxes are left out.
impostos-cidade-conferido = Looked up on { $data }.
impostos-cidade-sem-dados = No lookup made yet for this city.
impostos-cidade-fonte-desligada = The Tesouro Nacional (Siconfi) source is turned off in Settings › Data sources.
impostos-cidade-buscar = Fetch from Tesouro Nacional
impostos-cidade-atualizar = Refresh
impostos-cidade-buscando = Fetching…
