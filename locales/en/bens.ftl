# Portfolio › Physical assets: homes, vehicles, land, equipment and collections, with cost,
# appraisals, improvements, sale and a linked loan (docs/i18n-design.md). The value formula and
# errors come from the core; IPCA, IGP-M, INCC, ITBI, FIPE and LTV stay as they are.
# Revision: AI draft, not yet reviewed by a native speaker.

## Kinds, methods, indexes and sources

bens-tipo-imovel = Real estate
bens-tipo-veiculo = Vehicle
bens-tipo-terreno = Land
bens-tipo-equipamento = Equipment
bens-tipo-colecao = Art and collectibles
bens-tipo-outro = Other

bens-metodo-nenhum = No estimate
bens-metodo-indice = Index adjustment
bens-metodo-declinante = Yearly depreciation
bens-metodo-linear = Straight-line depreciation
bens-metodo-fipe = FIPE table
bens-metodo-nenhum-dica = Uses the latest recorded appraisal; without one, the cost.
bens-metodo-indice-dica = Cost (or the latest appraisal) adjusted by the chosen index. It's an estimate: the property market doesn't follow the index exactly.
bens-metodo-declinante-dica = Loses a fixed percentage of the previous value each year. Cars lose ~15–20% in the 1st year and ~8–10% after that; 12% a year is the average of the first five years.
bens-metodo-linear-dica = Loses the same amount every year down to the residual value at the end of its useful life (electronics and machines: 5 to 10 years).
bens-metodo-fipe-dica = Average price from the FIPE table at the last lookup; between lookups, yearly depreciation.

bens-indice-ipca-dica = Official inflation: keeps the purchasing power of what you paid.
bens-indice-igpm-dica = Rent index: more volatile, follows wholesale prices.
bens-indice-incc-dica = Construction cost: approximates the cost of rebuilding the property.

bens-categoria-reforma = Renovation
bens-categoria-ampliacao = Extension
bens-categoria-benfeitoria = Improvement
bens-categoria-manutencao = Maintenance

bens-custo-escritura = Deed
bens-custo-registro = Registration
bens-custo-corretagem = Brokerage
bens-custo-outro = Other cost

bens-fonte-manual = Own estimate
bens-fonte-avaliacao = Appraisal
bens-fonte-laudo = Appraisal report
bens-fonte-indice = By index

bens-base-avaliacao = appraisal
bens-base-automatica = automatic estimate
bens-base-custo = cost
bens-base-vendido = sold
bens-base-nao-comprado = not bought yet

bens-fipe-carro = Car
bens-fipe-moto = Motorcycle
bens-fipe-caminhao = Truck

## How the value is estimated (short line) and owners

bens-metodo-texto-indice = adjusted by { $indice }
bens-metodo-texto-declinante = { $taxa } depreciation a year
bens-metodo-texto-linear = straight-line over { $anos } years
bens-metodo-texto-linear-residual = straight-line over { $anos } years down to { $residual }
bens-metodo-texto-fipe = FIPE table + { $taxa } a year
bens-metodo-texto-fipe-codigo = FIPE table { $codigo } + { $taxa } a year
bens-metodo-texto-nenhum = no automatic estimate
bens-donos-par = { $a } and { $b }
bens-familia = Family

## Repeated labels

bens-titulo = Physical assets
bens-bem = Asset
bens-o-bem = asset
bens-tipo = Type
bens-valor = Value
bens-custo = Cost
bens-ganho = Gain
bens-patrimonio = Equity
bens-avaliacao = Appraisal
bens-avaliacoes = Appraisals
bens-saldo-devedor = Outstanding balance
bens-data = Date
bens-descricao = Description
bens-fonte = Source
bens-projeto = Project
bens-observacoes = Notes
bens-opcional = Optional.
bens-nenhum = None
bens-mes = Month
bens-ao-ano-abrev = a year
bens-vendido = sold
bens-arquivado = archived
bens-acoes-de = Actions for { $nome }
bens-editar-nome = Edit { $nome }
bens-excluir-nome = Delete { $nome }
bens-excluir = Delete
bens-reativar = Reactivate
bens-arquivar = Archive
bens-reativado = Asset reactivated
bens-arquivado-toast = Asset archived
bens-excluido = Asset deleted
bens-excluir-titulo = Delete { $nome }
bens-excluir-bem = Delete asset
bens-novo = New asset
bens-nao-excluiu = Could not delete

## Figures

bens-valor-estimado = Value (≈ estimated)
bens-custo-acumulado = Cumulative cost
bens-avaliacao-em = { $fonte } on { $data }
bens-valor-do-bem = Asset value
bens-patrimonio-do-bem = Asset equity
bens-fig-valor-titulo = Value × cost over time
bens-fig-valor-fonte = Value: latest appraisal or automatic estimate · cost: purchase + costs + capital improvements
bens-fig-valor-fonte-estimado = Value: latest appraisal or automatic estimate (≈ estimated) · cost: purchase + costs + capital improvements
bens-fig-valor-aria =
    { $n ->
        [one] Value of { $nome } over time, with the cumulative cost and { $n } appraisal.
       *[other] Value of { $nome } over time, with the cumulative cost and { $n } appraisals.
    }
bens-fig-valor-tabela = Value month by month · { $nome }
bens-fig-patrimonio-titulo = Asset equity × outstanding balance
bens-fig-patrimonio-fonte = Outstanding balance from the schedule of the linked loans · the bands add up to the asset's value
bens-fig-patrimonio-aria = Equity of { $nome } and the loan's outstanding balance, stacked month by month.
bens-fig-patrimonio-tabela = Asset equity month by month · { $nome }

## List and summary

bens-resumo-valor = Value of assets
bens-n-bens =
    { $n ->
        [one] { $n } asset
       *[other] { $n } assets
    }
bens-resumo-ganho = Gain over cost
bens-custo-de = cost of { $valor }
bens-resumo-patrimonio = Equity in assets
bens-valor-menos-saldo-financiamentos = Value − outstanding balance of loans
bens-saldo-devedor-valor = outstanding balance { $valor }
bens-sem-financiamentos = no linked loans
bens-cmd-grupo = Assets
bens-cmd-benfeitoria = Record improvement on { $nome }
bens-cmd-benfeitoria-busca = work,renovation,extension,improvement,maintenance
bens-cmd-avaliacao = Record appraisal of { $nome }
bens-cmd-avaliacao-busca = appraisal,report,market value,fipe
bens-abrir = Open asset
bens-registrar-benfeitoria-menu = Record improvement…
bens-registrar-avaliacao-menu = Record appraisal…
bens-novo-projeto-menu = New project for the asset…
bens-ligar-financiamento-menu = Link loan…
bens-registrar-venda-menu = Record sale…
bens-editar-cadastro-menu = Edit details…
bens-excluir-menu = Delete…
bens-nao-foi-possivel = Could not do it
bens-avaliando = Valuing the assets
bens-vazio-kicker = No assets
bens-vazio-titulo = Add your first home, vehicle or plot of land
bens-vazio-descricao = Enter the purchase date and price with the costs (ITBI, deed, registration): the current value is estimated by the index, by depreciation or by the FIPE table, and improvements count toward the cost. Link the loan from 07 · Debts to see the asset's equity.
bens-mostrar-arquivados = Show archived and sold
bens-excluir-descricao-lista = Deletes the asset with its costs, improvements and appraisals. Loans and transactions stay. You can undo it from the history.

## Asset detail

bens-bem-nome = Asset { $nome }
bens-comprado-em = bought on { $data }
bens-vendido-em = sold on { $data }
bens-fora-patrimonio = left out of net worth
bens-registrar-avaliacao = Record appraisal
bens-fipe-consultada = FIPE table checked
bens-fipe-valor-atualizado = { $nome }: value updated
bens-fipe-falhou = Could not check FIPE
bens-consultar-fipe = Check FIPE
bens-vender = Sell
bens-benfeitoria = Improvement
bens-valor-atual = Current value
bens-vendido-maiusc = Sold
bens-por-base = By { $base }
bens-liquido-venda = net of the sale
bens-compra = Purchase
bens-custos-aquisicao = Acquisition costs
bens-benfeitorias = Improvements
bens-custo-total = Total cost
bens-manutencao-fora = Maintenance (not in cost)
bens-valorizacao = Appreciation
bens-depreciacao = Depreciation
bens-ganho-realizado = Realized gain
bens-ganho-nominal = Nominal gain
bens-sobre-custo = Over cost
bens-ganho-real = Real gain (− IPCA)
bens-ao-ano = Per year
bens-ao-ano-real = Per year, real
bens-valor-menos-saldo = Value − outstanding balance
bens-saldo-ltv = outstanding balance { $saldo } · LTV { $ltv }
bens-sem-financiamento = no linked loan
bens-valor-estimado-titulo = ≈ Estimated value
bens-valor-medido = Measured value
bens-calculando-historico = Working out the asset's history
bens-financiamento = Loan
bens-ligar-financiamento = Link loan
bens-financiamentos-de = Loans of { $nome }
bens-financiamentos-vazio = No linked contract. Link the loan registered in 07 · Debts to see the asset's equity and LTV.
bens-contrato = Contract
bens-financiado = Financed
bens-desligar-nome = Unlink { $nome }
bens-financiamento-desligado = Loan unlinked
bens-nao-desligou = Could not unlink
bens-desligar = Unlink
bens-benfeitorias-titulo = Improvements, renovations and works
bens-entram-custo = Count toward cost
bens-entram-custo-valor = renovation, extension, improvement
bens-fora-custo-rotulo = Not in cost
bens-fora-custo-valor = maintenance
bens-benfeitorias-de = Improvements of { $nome }
bens-benfeitorias-vazio = No work recorded.
bens-fora-custo = not in cost
bens-sem-lancamentos-estimado = no transactions: the estimate applies
bens-n-lancamentos =
    { $n ->
        [one] { $n } transaction
       *[other] { $n } transactions
    }
bens-benfeitoria-excluida = Improvement deleted
bens-avaliacoes-de = Appraisals of { $nome }
bens-avaliacoes-vazio = No appraisals: the value is estimated automatically from the cost.
bens-excluir-avaliacao-de = Delete appraisal from { $data }
bens-avaliacao-excluida = Appraisal deleted
bens-custos-aquisicao-prefixo = Acquisition costs:
bens-observacoes-texto = Notes: { $notas }
bens-excluir-descricao-detalhe = The asset, its improvements and appraisals are removed; the loan stays in 07 · Debts. You can undo it from the change history.

## Asset form

bens-editar-bem = Edit asset
bens-form-descricao = Real estate, vehicle, land, equipment or collection: the purchase with its acquisition costs, the valuation method and, if any, the loan.
bens-erro-nome = Give the asset a name.
bens-erro-data-compra = Enter the purchase date.
bens-erro-preco-compra = Enter the purchase price.
bens-erro-taxa = Rate between 0 and 100% a year.
bens-erro-vida-util = Useful life between 1 and 100 years.
bens-erro-fipe = Choose make, model and year in the FIPE table.
bens-erro-valor-invalido = Invalid amount.
bens-alterado = Asset changed
bens-cadastrado = Asset added
bens-nome-bem = Asset name
bens-nome-exemplo = Downtown apartment
bens-data-compra = Purchase date
bens-preco-compra = Purchase price
bens-preco-compra-dica = Price of the asset, without the costs.
bens-custos-aquisicao-dica = ITBI, deed, registration and brokerage count toward the asset's cost (and the capital gain).
bens-custo-n = Cost { $n }
bens-remover-custo = Remove cost { $n }
bens-remover = Remove
bens-adicionar-custo = Add cost
bens-estimado-por = Current value estimated by
bens-indice-correcao = Adjustment index
bens-depreciacao-aa = Depreciation (% a year)
bens-depreciacao-fipe-dica = Used between FIPE table lookups.
bens-depreciacao-dica = Default of 12% a year for cars.
bens-vida-util = Useful life (years)
bens-residual = Residual value (% of cost)
bens-residual-dica = What's left at the end of its useful life. Optional.
bens-endereco = Address or identification
bens-endereco-dica = Optional (property record, plate, chassis…).
bens-financiamento-dica = Loan from 07 · Debts that financed the asset. Optional.
bens-de-membro = Owned by { $nome }
bens-entra-patrimonio = Counts toward net worth
bens-salvar-alteracoes = Save changes
bens-cadastrar = Add asset

## FIPE table

bens-fipe-explica = With make, model and year, the app looks up the average price in the FIPE table (parallelum.com.br, free) when you ask.
bens-fipe-buscar = Look up in the FIPE table
bens-fipe-tipo-veiculo = Vehicle type
bens-fipe-marca = Make
bens-fipe-escolha-marca = Choose the make
bens-fipe-modelo = Model
bens-fipe-escolha-modelo = Choose the model
bens-fipe-ano = Year and fuel
bens-fipe-escolha-ano = Choose the year
bens-fipe-codigo = FIPE code
bens-fipe-codigo-dica = Optional: the lookup checks the code.

## Improvement, appraisal, sale and loan

bens-editar-benfeitoria = Edit improvement
bens-registrar-benfeitoria-obra = Record improvement or work
bens-benfeitoria-descricao = { $nome }: renovations, extensions and improvements count toward the cost; maintenance is an expense and stays out.
bens-erro-data = Enter the date.
bens-erro-descreva = Describe the work.
bens-erro-valor-projeto = Enter the amount (or link it to a project).
bens-benfeitoria-alterada = Improvement changed
bens-benfeitoria-registrada = Improvement recorded
bens-tipo-obra = Type of work
bens-benfeitoria-exemplo = Bathroom renovation
bens-valor-estimado-campo = Estimated amount
bens-valor-projeto-dica = With transactions in the project, what it spent applies.
bens-fica-fora-custo = Stays out of the asset's cost.
bens-soma-custo = Adds to the asset's cost.
bens-projeto-gastos = Spending project
bens-projeto-gastos-dica = The project's transactions become the amount of the work, each on its own date.
bens-novo-projeto-benfeitoria = New project named after the improvement
bens-sera-criado = “{ $nome }” will be created for this asset; then add the transactions to it.
bens-a-benfeitoria = the improvement
bens-registrar = Record
bens-avaliacao-descricao = { $nome }: the most recent appraisal becomes the asset's value; after it, the automatic estimate starts from it.
bens-erro-valor = Enter the amount.
bens-avaliacao-registrada = Appraisal recorded
bens-avaliacao-obs-dica = Who appraised it, comparables… Optional.
bens-vender-nome = Sell { $nome }
bens-venda-descricao = Records the sale: the asset leaves your net worth from that date and the gain is realized. The money from the sale comes in through the account, as a transaction.
bens-erro-venda-antes = The sale can't be before the purchase.
bens-erro-preco = Enter the price.
bens-venda-registrada = Sale recorded
bens-venda = Sale
bens-data-venda = Sale date
bens-preco-venda = Sale price
bens-custos-venda = Sale costs
bens-custos-venda-dica = Brokerage, taxes, fees.
bens-arquivar-bem = Archive the asset
bens-registrar-venda = Record sale
bens-ligar-descricao = The contract's outstanding balance counts in the equity of { $nome } (value − balance) and in the LTV. The debt still counts only once, in 07 · Debts.
bens-financiamento-ligado = Loan linked
bens-nenhum-contrato-livre = No free contract. Register the loan in 07 · Debts first.
bens-escolha-contrato = Choose the contract
bens-ligar-contrato = Link contract
bens-gcap-titulo-vendido = Tax on the sale gain
bens-gcap-titulo-simular = Simulate the tax on a sale
bens-gcap-data = Sale date
bens-gcap-preco = Sale price
bens-gcap-custos = Brokerage paid on the sale
bens-gcap-unico = Only property, with no other property sale in the last 5 years
bens-gcap-unico-dica = Exempt if the sale price is up to R$ 440,000.
bens-gcap-reaplicado = Amount invested in another home within 180 days
bens-gcap-ganho-bruto = Gross gain (sale − brokerage − cost)
bens-gcap-lei-7713 = Lei 7.713 reduction ({ $pct }%)
bens-gcap-fr1 = FR1 factor ({ $pct }%)
bens-gcap-fr2 = FR2 factor ({ $pct }%)
bens-gcap-isencao-unico = Only-property exemption
bens-gcap-isencao-reaplicacao = Exemption for buying another home
bens-gcap-tributavel = Taxable gain
bens-gcap-imposto = Tax
bens-gcap-pequeno-valor = Exempt: sales up to R$ 35,000 in the month
bens-gcap-darf = DARF
bens-gcap-darf-valor = code { $codigo }, due { $vencimento }
bens-gcap-nota = As in the Receita's GCAP program: cost includes purchase costs and improvements; tax of 15% to 22.5% by bracket (Lei 8.981/1995 art. 21). Check in GCAP before paying.
