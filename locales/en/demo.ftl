# Demo vault wizard (docs/demo.md, docs/i18n-design.md). Revision: AI draft, not yet reviewed by
# a native speaker. The valor argument is an element (the money amount) that the screen puts in
# place.

## Starting values and steps

demo-padrao-nome = Demo
demo-padrao-familia = Sample Family
demo-passo-familia = Family
demo-passo-renda = Income
demo-passo-moradia = Housing and expenses
demo-passo-contas = Accounts and investments
demo-passo-dividas = Debts and creation
demo-titulo = Demo vault
demo-descricao = Tell us about the family and the app creates a separate vault with a sample history to try out freely.
demo-passo = Step { $n } of { $total }: { $passo }
demo-progresso = Wizard progress

## What's missing to move on

demo-erro-nome = Give the vault a name.
demo-erro-familia = Enter the family name.
demo-erro-pessoa = Enter each person's name.
demo-erro-nomes-iguais = Each person needs a different name.
demo-erro-renda = Enter the income of at least one adult.
demo-erro-financiamento = Enter the mortgage payment.
demo-erro-banco = Enter the bank of the main account.
demo-erro-carro = Enter the car payment and how many are left.
demo-criado = Vault "{ $nome }" created
demo-criado-dica = { $n } transactions from { $data } to today. To go back to your vault, use Vaults.

## Step 1: family

demo-nome-cofre = Vault name
demo-nome-cofre-dica = Shows in the vault list.
demo-nome-familia = Family name
demo-adultos = Adults with income
demo-um = One
demo-dois = Two
demo-nome-adulto = Adult { $n }'s name
demo-filhos = Children
demo-nenhum = None
demo-nome-filho = Child { $n }'s name
demo-historico = History
demo-historico-dica = How many months of transactions, up to today.
demo-meses-historico = Months of history
demo-meses = { $n } months

## Step 2: income

demo-renda-de = { $nome }'s net income
demo-adulto-n = adult { $n }
demo-renda-dica = Per month, what lands in the account.
demo-renda-variavel = Variable income
demo-renda-variavel-dica = Self-employed, commissions: it changes from month to month, with no 13º.
demo-outras-rendas = Other income
demo-outras-rendas-dica = Per month: rent received, for example.
demo-renda-total = Total income per month: { $valor }. In the next steps, amounts are suggested from it; adjust as you like.

## Step 3: housing and expenses

demo-moradia = Housing
demo-aluguel = Rent
demo-financiamento = Mortgage
demo-propria = Owned outright
demo-sugerir = Suggest from income
demo-parcela-financiamento = Mortgage payment
demo-condominio = Building fees
demo-contas-casa = Household bills
demo-contas-casa-dica = Electricity, water, internet and phone.
demo-mercado = Groceries and bakery
demo-transporte = Transportation
demo-carro = Car
demo-aplicativo = Ride-hailing
demo-onibus = Bus and subway
demo-nao-tem = None
demo-combustivel = Fuel and parking
demo-escola = School, per child
demo-saude = Health
demo-saude-dica = Health insurance and pharmacy.
demo-lazer = Leisure
demo-lazer-dica = Restaurants, delivery, movies and outings.
demo-assinaturas = Subscriptions
demo-assinaturas-dica = Streaming and music.
demo-compras = Shopping
demo-compras-dica = Clothes, home and electronics.

## Step 4: accounts and investments

demo-banco = Bank of the main account
demo-cartao = Credit card
demo-cartao-dica = Everyday purchases go on the card and the bills are paid on the due date.
demo-limite = Card limit
demo-investimentos = Investments
demo-reserva = Emergency fund today
demo-reserva-mes = To the emergency fund, per month
demo-acoes = Stocks (at the start of the history)
demo-fiis = Real estate funds (at the start)
demo-tesouro = Tesouro Direto (at the start)
demo-aporte = Contribution to the brokerage, per month
demo-previdencia = VGBL pension plan (at the start)
demo-aporte-previdencia = Contribution to the pension plan, per month
demo-cotacoes-ficticias = Stocks and funds with real tickers but made-up quotes (manual quote): the sample doesn't mix real prices with invented trades.

## Step 5: debts and creation

demo-financiamento-carro = Car loan
demo-financiamento-carro-dica = Becomes a debt contract, with the installments paid in the history.
demo-parcela = Installment
demo-parcelas-faltam = Installments left
demo-financiamento-moradia = The mortgage also becomes a debt contract.
demo-resumo = Month summary
demo-entra = In
demo-sai = Out (spending)
demo-investe = Invested
demo-vermelho = Spending and contributions exceed income: the sample will show months in the red.
demo-nivel = Interface level in the demo vault
demo-nivel-rotulo = Interface level
demo-troca = The current vault closes (with a backup) and the demo vault opens. To go back, use Vaults: your vault stays the same.
demo-criar = Create demo vault

## Profile and country (first step)

demo-passo-perfil = Profile
demo-pais = Family's country
demo-pais-dica = Sets the currency, categories, taxes and assets in the sample.
demo-perfil = Family profile
demo-perfil-cofre = Demo · { $perfil }
demo-perfil-confortavel = Comfortable middle class
demo-perfil-confortavel-dica = Couple with one child, home mortgage, a car, an emergency fund and investments.
demo-perfil-baixa-renda = Low income
demo-perfil-baixa-renda-dica = Couple with two children, rent, public transport, a tight income, side jobs and a government benefit.
demo-perfil-dificuldade = Family in financial trouble
demo-perfil-dificuldade-dica = Spends more than it earns: overdraft, a revolving card balance, a loan and late bills.
demo-perfil-sozinho = Young person living alone
demo-perfil-sozinho-dica = One person, a small rental, ride-hailing and first investments.
demo-perfil-aposentado = Retiree
demo-perfil-aposentado-dica = Retirement income, own home, rent received, health insurance and savings.
demo-perfil-custom = Start from scratch
demo-perfil-custom-dica = You describe the family in the next steps.
demo-perfil-nota = Amounts come in the country's currency, and you can change everything in the next steps.

## Income, debts and creation

demo-aposentado = Retired
demo-aposentado-dica = The income is a retirement pension.
demo-beneficio = Government benefit
demo-beneficio-dica = Per month.
demo-emprestimo = Personal loan
demo-emprestimo-dica = Becomes a contract in Debts, with the installments paid from the account.
demo-erro-emprestimo = Enter the loan installment and how many are left.
demo-cheque-especial = Overdraft
demo-cheque-especial-dica = The account can go negative; in a month that ends in the red, interest comes at the start of the next one.
demo-contas-atrasadas = Some bills paid late
demo-contas-atrasadas-dica = One in three household bills goes out ten days late, with a late fee and interest.
demo-rotativo = Pays only part of the card bill
demo-rotativo-dica = Pays 70% of each bill; the rest stays as a revolving balance and the card charges interest.
demo-sem-senha = The demo vault opens without a password. Anyone with the app can open it: use only made-up data, never real data.

## Quick creation

demo-rapido-descricao = Choose the country and the profile: the demo vault is created and opens right away, without a password.
demo-rapido-criar = Create and open
demo-rapido-personalizar = Customize before creating
demo-idade = Age
demo-idade-de = { $nome }'s age
demo-idade-auto = From profile
demo-sexo-de = { $nome }'s sex
demo-erro-idade = Check the ages: adults from 18 to 100 and children from 0 to 17.
