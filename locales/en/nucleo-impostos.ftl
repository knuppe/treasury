# Taxes (core, `tax` and `tax_burden`): the income tax calculation on stock-market gains and the
# DARFs (09 · Taxes › IR, Brazil only) and the estimate of taxes paid (09 · Taxes): errors,
# history reasons, parts, groups, notes and the texts of each country's data
# (`tax_burden/data/*.json`). Errors in lowercase, like a log line (the interface capitalizes
# them). Official terms (DARF, IRRF, IR, INSS, ICMS, IPVA, IPTU, IOF, NFC-e, FICA, IRS)
# stay as is.
# Revision: AI draft, not yet reviewed by a native speaker.
#
# Stay in pt-BR in any language: the description, group and country for Bens e Direitos (they go
# into the Receita Federal program), the cited sources, the states and the legal bases.

## IR: categories, notes and disclaimer of the calculation (`tax`)

nucleo-impostos-ir-categoria-acoes = Stocks (regular trades)
nucleo-impostos-ir-categoria-etf = ETFs (regular trades)
nucleo-impostos-ir-categoria-bdr = BDRs (regular trades)
nucleo-impostos-ir-categoria-fii = FIIs
nucleo-impostos-ir-categoria-day-trade = Day trade
nucleo-impostos-ir-nota-isencao = Stock sales in the month of { $valor } (up to R$ 20,000): exempt gain; a loss can still be offset.
nucleo-impostos-ir-nota-irrf-estimado = Estimated IRRF (0.005% of sales; 1% of the day-trade gain): check your brokerage notes.
nucleo-impostos-ir-nota-venda-maior = Sale of { $ativo } on { $data } is larger than the recorded position ({ $quantidade } more): the amount sold counts in the month's total, but the result of that part stays out of the calculation until the purchase is recorded.
nucleo-impostos-ir-aviso = A helper calculation made with your data; check it in the Receita Federal (Brazilian tax authority) program. This is not tax advice.
nucleo-impostos-ir-erro-ano = invalid year

## IR: DARF

nucleo-impostos-ir-erro-gerar-darf = generating a DARF requires the user's approval
nucleo-impostos-ir-erro-abaixo-minimo = tax below R$ 10.00: it carries over to the next month
nucleo-impostos-ir-erro-sem-imposto = there's no tax to pay this month
nucleo-impostos-ir-erro-darf-existe = there's already a DARF for this month; cancel the previous one to generate another
nucleo-impostos-ir-erro-pagar-interface = recording a DARF payment only from the interface
nucleo-impostos-ir-erro-darf-pago = a paid DARF can't be cancelled
# History reasons; the month stays as YYYY-MM.
nucleo-impostos-ir-motivo-darf = DARF { $codigo } for { $mes }
nucleo-impostos-ir-motivo-pagamento = Payment of the DARF for { $mes }
nucleo-impostos-ir-motivo-cancelado = DARF for { $mes } cancelled
nucleo-impostos-ir-motivo-custodia-cripto = Where the account's crypto is held (tax)
# Description of the payment transaction.
nucleo-impostos-ir-lancamento-darf = DARF { $codigo } — IR on stock-market gains { $mes }

## Taxes paid: settings, categories and income statements (`tax_burden`)

nucleo-impostos-erro-ajustes-interface = country and state of the estimate only from the app's interface
nucleo-impostos-erro-dependentes = too many dependents
nucleo-impostos-erro-brasil = Brazil's tax data only applies to families living in Brazil
nucleo-impostos-cidade-iptu = IPTU (property tax)
nucleo-impostos-cidade-iss = ISS (services tax)
nucleo-impostos-cidade-itbi = ITBI (property transfer tax)
nucleo-impostos-cidade-irrf = Income tax withheld from the city's employees
nucleo-impostos-cidade-outros = Fees and betterment levies
nucleo-impostos-cidade-erro-codigo = invalid city: choose the municipality from the list
nucleo-impostos-cidade-erro-uf = invalid state (UF)
nucleo-impostos-cidade-erro-consulta = the lookup failed: { $erro }
nucleo-impostos-cidade-erro-resposta = the source answered in a format the app doesn't recognise
nucleo-impostos-cidade-erro-sem-cidade = choose the family's city first
nucleo-impostos-cidade-erro-sem-relatorio = { $cidade } has no revenue report at the Tesouro Nacional for the last six two-month periods
nucleo-impostos-erro-estado = state { $estado } has no data in { $dados }
nucleo-impostos-dados-de = tax data for { $pais }
nucleo-impostos-categoria-despesa = expense category { $id }
nucleo-impostos-erro-grupo = unknown tax group: { $grupo }
nucleo-impostos-erro-aliquota = your own rate must be between 0% and 100% of the price
nucleo-impostos-erro-fonte-pagadora = enter the payer
nucleo-impostos-erro-ano-informe = invalid statement year
nucleo-impostos-erro-informe-negativo = statement amounts can't be negative
nucleo-impostos-informe = income statement { $id }
nucleo-impostos-motivo-grupo = Category tax group changed
nucleo-impostos-motivo-informe-alterado = Income statement changed: { $fonte }
nucleo-impostos-motivo-informe-lancado = Income statement added: { $fonte }
nucleo-impostos-motivo-informe-removido = Income statement removed

## Taxes paid: the estimate

nucleo-impostos-erro-ano-futuro = the year hasn't started yet
nucleo-impostos-parte-consumo = Consumption (built into prices)
nucleo-impostos-parte-renda = Income tax
nucleo-impostos-parte-previdencia = Social security (INSS)
nucleo-impostos-parte-patrimonio = Property (IPTU, IPVA)
nucleo-impostos-parte-proventos = IR withheld on investment income
nucleo-impostos-parte-outros = IOF and other taxes
nucleo-impostos-sem-categoria = Uncategorized
nucleo-impostos-cupons = Scanned receipts
nucleo-impostos-grupo-cupons = Tax on the receipts (NFC-e)
nucleo-impostos-nota-escolha-uf = Choose the state (UF): without it, IPVA isn't estimated.
nucleo-impostos-nota-escolha-estado = Choose the state: without it, sales tax isn't estimated.
nucleo-impostos-nota-escolha-pais = Choose the country: without it, VAT isn't estimated.
# parte: whole percentage (no decimals).
nucleo-impostos-nota-cupons = { $parte }% of consumption came from scanned receipts: for those purchases the tax on the receipt counts (the printed one, or the one estimated item by item from the consumption group's rate).
nucleo-impostos-nota-sem-grupo = { $n ->
        [one] { $n } category without a tax group was left out: choose its group.
       *[other] { $n } categories without a tax group were left out: choose their group.
    }
nucleo-impostos-nota-sem-iptu = There's real estate registered and no IPTU recorded in the year: IPTU isn't part of the estimate.
nucleo-impostos-nota-sem-regras = The { $dados } data has no income rules: tax on wages isn't estimated (add the income statements).
nucleo-impostos-nota-salario-estimado = Tax and INSS on wages estimated from the net pay received, without deductions such as health insurance or alimony: with the income statement, the estimate uses its amounts.
nucleo-impostos-nota-sem-ipva = There's a vehicle registered and no IPVA recorded: choose the state to estimate IPVA.
nucleo-impostos-nota-ipva-estimado = IPVA estimated from each vehicle's value at the end of the period and the state's rate: record the IPVA paid to use the actual amount.

## Data of each country (`tax_burden/data/*.json`): name, disclaimer, methodology, context and
## consumption groups. The translation only shows while the pt-BR text is the same as in the
## file: when the data is updated, update these lines too; the test
## `dataset_texts_are_in_the_catalog` points out what changed.

nucleo-impostos-dados-br-nome = Brazil
nucleo-impostos-dados-br-aviso = This is an estimate based on national average taxes by consumption category and the income tax rules in force. Actual amounts vary with the product, the state, companies' tax regime and specific tax benefits. Sources: IBGE (National Accounts), Receita Federal, Previdência Social, Tesouro Nacional and Petrobras.
nucleo-impostos-dados-br-metodo-1 = Consumption: what was spent in each category times the share of the price that is tax on the product. The percentages come from IBGE's National Accounts (the 2021 Supply and Use Tables, the latest year published with 128 products): taxes on the product (import duty, IPI, ICMS and other taxes less subsidies) divided by supply at purchasers' prices. For petrol, diesel and cooking gas the price breakdown published by Petrobras is used; for insurance and bank fees, the statutory rate.
nucleo-impostos-dados-br-metodo-2 = Scenarios: groups calculated from the National Accounts have the same value in all three scenarios. When a group combines several products, the value is the average weighted by household consumption. The scenarios only differ where the source gives a range (fuels, insurance and bank fees).
nucleo-impostos-dados-br-metodo-3 = State: the consumption percentages are national averages and don't change with the chosen state. The state is used for IPVA.
nucleo-impostos-dados-br-metodo-4 = Income from work: with the income statement, its amounts count. Without one, each month's gross pay is estimated from the net wage received, using the IRPF and INSS tables in force that month (no dependents unless adjusted, and no other deductions); the 13th salary is taxed separately (Law 8,134/1990, art. 16).
nucleo-impostos-dados-br-metodo-5 = Taxes paid directly (IR, IOF, IPTU, IPVA and other taxes recorded in their categories) count at the recorded amount. IPVA is estimated from each vehicle's value and the state's rate (typical flex-fuel car; motorcycles at the general rate) only when no IPVA was recorded in the year. Vehicles old enough for the state's exemption, or 20 years or older from 2026 on (EC 137/2025), don't pay.
nucleo-impostos-dados-br-metodo-6 = IR withheld on investment income (JCP, foreign dividends) comes from the portfolio records.
nucleo-impostos-dados-br-metodo-7 = What the IBGE figure measures: only the taxes charged on the product. Taxes paid earlier in the chain and not recovered, payroll charges and corporate income tax are not included. That's why the percentages are lower than estimates that add those taxes, and the result should be read as a floor.
nucleo-impostos-dados-br-metodo-8 = Not included: charges paid by the employer (employer INSS, FGTS), groups with no official data (health plans, domestic services, vehicle natural gas, motorcycles, purchases with no detail) and the 2026 tax reform trial (CBS 0.9% and IBS 0.1%, offset against PIS/Cofins).
nucleo-impostos-dados-br-metodo-9 = Updates: National Accounts with this level of detail are published about two years late. Rate changes after 2021, such as the 2022 ICMS cap on electricity and telecommunications, only show up when IBGE publishes the corresponding year.
nucleo-impostos-dados-br-contexto = Brazil's gross tax burden (% of GDP)
nucleo-impostos-dados-br-dias = Brazil's gross tax burden in days of the year (Tesouro Nacional)
nucleo-impostos-dados-br-destino = General government spending by function (federal, state and municipal)
nucleo-impostos-funcao-servicos-publicos-gerais = General public services (includes debt interest)
nucleo-impostos-funcao-defesa = Defence
nucleo-impostos-funcao-ordem-publica-seguranca = Public order and safety
nucleo-impostos-funcao-assuntos-economicos = Economic affairs
nucleo-impostos-funcao-protecao-ambiental = Environmental protection
nucleo-impostos-funcao-habitacao-servicos-comunitarios = Housing and community amenities
nucleo-impostos-funcao-saude = Health
nucleo-impostos-funcao-lazer-cultura-religiao = Recreation, culture and religion
nucleo-impostos-funcao-educacao = Education
nucleo-impostos-funcao-protecao-social = Social protection (pensions and welfare)

nucleo-impostos-dados-us-nome = United States
nucleo-impostos-dados-us-aviso = Draft estimate: it only considers the state's average sales tax and the taxes recorded or entered. Actual amounts vary by city, product and each person's situation. Source: Tax Foundation.
nucleo-impostos-dados-us-metodo-1 = Draft: only state and local sales tax, at the state's average combined rate (Tax Foundation, mid-2026), as a share of the amount paid: rate / (100 + rate).
nucleo-impostos-dados-us-metodo-2 = Income: federal tax is estimated month by month with the 2026 brackets for single filers, with the standard deduction and the child tax credit divided by 12; FICA is Social Security 6.2% up to the annual cap (US$ 184,500, divided by 12) and Medicare 1.45% (plus 0.9% above US$ 200,000 a year). State and local income taxes aren't included; with the W-2, add the income statement to use the actual amounts.

nucleo-impostos-dados-eu-nome = European Union
nucleo-impostos-dados-eu-aviso = Draft estimate: it only considers the chosen country's VAT and the taxes recorded or entered. Reduced rates vary by product in each country. Source: European Commission.
nucleo-impostos-dados-eu-metodo-1 = Draft: only VAT, at the standard, reduced (the lowest) and super-reduced rates of the chosen country (the European Commission's "Your Europe" table, checked on 07/13/2026).
nucleo-impostos-dados-eu-metodo-2 = Income (Portugal only): IRS is estimated from the annual tax of the 2026 mainland brackets, divided by the year's 14 payments (12 salaries plus the holiday and Christmas allowances), with the specific deduction (the greater of €4,587.09 and Social Security), €600 per dependent and the subsistence minimum; Social Security is 11% of gross pay. These aren't the monthly withholding tables (which withhold a little differently and are settled in the tax return). The other European Union countries don't have income rules yet: add the income statements.

# Consumption groups (the same in all three countries).
nucleo-impostos-grupo-alimentos-basicos = Basic foods
nucleo-impostos-grupo-alimentos-industrializados = Processed foods
nucleo-impostos-grupo-bebidas-nao-alcoolicas = Non-alcoholic drinks
nucleo-impostos-grupo-bebidas-alcoolicas = Alcoholic drinks
nucleo-impostos-grupo-restaurantes = Restaurants, bars and delivery
nucleo-impostos-grupo-gasolina = Regular gasoline (gasoline C)
nucleo-impostos-grupo-diesel = Diesel (diesel B)
nucleo-impostos-grupo-etanol = Hydrous ethanol
nucleo-impostos-grupo-gnv = Compressed natural gas (CNG)
nucleo-impostos-grupo-glp = Cooking gas (LPG, 13 kg cylinder)
nucleo-impostos-grupo-energia-eletrica = Electricity (home power bill)
nucleo-impostos-grupo-agua-esgoto = Water and sewage
nucleo-impostos-grupo-telefonia-internet = Phone and internet
nucleo-impostos-grupo-tv-assinatura-streaming = Pay TV and streaming
nucleo-impostos-grupo-eletronicos-nacionais = Electronics (domestic or unspecified)
nucleo-impostos-grupo-eletronicos-importados = Imported electronics
nucleo-impostos-grupo-eletrodomesticos = Home appliances
nucleo-impostos-grupo-moveis = Furniture and mattresses
nucleo-impostos-grupo-vestuario = Clothing
nucleo-impostos-grupo-calcados = Footwear
nucleo-impostos-grupo-medicamentos = Medicines for human use
nucleo-impostos-grupo-higiene-pessoal = Personal hygiene
nucleo-impostos-grupo-cosmeticos-perfumes = Cosmetics and perfumes
nucleo-impostos-grupo-servicos-pessoais = Personal services (salon, gym/club)
nucleo-impostos-grupo-educacao-mensalidade = Education – school tuition
nucleo-impostos-grupo-educacao-livros = Education – books
nucleo-impostos-grupo-saude-plano-consulta = Health – health insurance and appointments
nucleo-impostos-grupo-transporte-publico = Public transport
nucleo-impostos-grupo-transporte-aplicativo = Ride-hailing apps
nucleo-impostos-grupo-passagens-aereas = Airfare
nucleo-impostos-grupo-hotel-turismo = Hotels and tourism
nucleo-impostos-grupo-automovel-novo = New car
nucleo-impostos-grupo-motocicleta = Motorcycle
nucleo-impostos-grupo-pecas-manutencao-veiculo = Vehicle parts and maintenance
nucleo-impostos-grupo-seguros = Insurance (IOF on the premium)
nucleo-impostos-grupo-tarifas-bancarias = Bank fees
nucleo-impostos-grupo-brinquedos = Toys
nucleo-impostos-grupo-pet = Pets (food, medicine, vet)
nucleo-impostos-grupo-materiais-construcao = Building materials
nucleo-impostos-grupo-cigarros = Cigarettes
nucleo-impostos-grupo-mercado = Groceries (mixed basket)
nucleo-impostos-grupo-aluguel = Home rent
nucleo-impostos-grupo-condominio = HOA and building fees
nucleo-impostos-grupo-servicos-domesticos = Household services (cleaner, nanny)
nucleo-impostos-grupo-estacionamento = Parking
nucleo-impostos-grupo-optica = Glasses and lenses
nucleo-impostos-grupo-material-escolar = School supplies
nucleo-impostos-grupo-lazer = Leisure (outings, tickets, parties)
nucleo-impostos-grupo-presentes = Gifts (assorted items)
nucleo-impostos-grupo-compras-diversas = Assorted purchases (no detail)
nucleo-impostos-grupo-contas-diversas = Bills and payment slips (no detail)
nucleo-impostos-grupo-limpeza = Cleaning supplies
nucleo-impostos-anual-nota-sem-holerite = No payslips for the year in the vault: enter the income from the employer's income statement.
nucleo-impostos-anual-nota-holerites = { $fonte }: payslips for { $meses } months, { $valor } taxable (without the 13th salary and profit sharing, which are taxed exclusively). Check against the income statement.
nucleo-impostos-anual-nota-65 = Retirement income from age 65: { $valor } left out as the exempt portion (R$ 1,903.98 a month).
nucleo-impostos-anual-nota-despesas = Deductible expenses from the categories: health { $saude } (insurance, appointments, tests, dentist, therapy) and education { $instrucao } (school, daycare, university). Pharmacy, glasses, school supplies, language and free courses are not included.
nucleo-impostos-anual-nota-pgbl = Contributions to PGBL and pension funds: { $valor } (deductible up to 12% of taxable income).
nucleo-impostos-anual-nota-dedo-duro = Unused stock-sale withholding ("dedo-duro") left in the year: { $valor }, counted as tax paid.
nucleo-impostos-anual-nota-familia = Whole family: the return is per person. Pick a member for the preview of each return.
nucleo-impostos-anual-nota-confira = Preview to check: adjust the values with the statements and receipts. The official return is filed in the Receita's program.
nucleo-impostos-anual-nota-irpfm = Income above R$ 600,000 in the year: the minimum tax on high incomes applies (IRPFM, Lei 15.270/2025). The estimate does not apply the company-tax reducer (art. 16-B), which may lower it or bring it to zero.
nucleo-impostos-gcap-erro-venda = Enter the sale date and price to calculate the capital gain.
