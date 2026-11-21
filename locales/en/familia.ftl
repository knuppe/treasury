# Screen 02 · Family: the members, each member's profile, the income sources, imported payslips,
# each member's accounts and cards and the family's income forecast (docs/i18n-design.md). Payroll
# names for other countries still come from the core.
# Revision: AI draft, not yet reviewed by a native speaker.

familia-nome-instituicoes = Family name and institutions

## What Brazil calls the income items (other countries come from the core)

familia-tipo-clt = CLT
familia-tipo-servidor = Public servant
familia-tipo-aposentadoria = Retirement
familia-tipo-pensao = Pension
familia-tipo-pj = PJ
familia-tipo-autonomo = Self-employed
familia-tipo-informal = Informal
familia-tipo-aluguel = Rent
familia-tipo-beneficio = Social benefit
familia-tipo-outra = Other
familia-holerite = payslip
familia-holerites = payslips
familia-descontos = { $contribuicao } and { $imposto }
familia-descontos-eua = FICA and federal tax
familia-13-salario = 13º salary
familia-subsidio-natal = Christmas allowance
familia-terco-ferias = vacation bonus (one third)
familia-subsidio-ferias = holiday allowance
familia-dependente-dica-br = Counts as a dependent on the tax return of whoever files (a deduction in the withheld income tax).
familia-dependente-dica = Counts as a dependent for the tax of whoever files (lowers the withheld tax).

## Member profile

familia-relacao-titular = Head of household
familia-relacao-conjuge = Spouse
familia-relacao-filho = Child
familia-relacao-pai-mae = Parent
familia-relacao-outro = Other
familia-cadastro-de = { $nome }'s profile
familia-relacao = Relationship in the family
familia-escolha = Choose
familia-nascimento = Date of birth
familia-sexo = Sex
familia-sexo-dica = Optional. Used in the life timeline and as the default in the INSS calculation.
familia-sexo-nao-informado = Not provided
familia-sexo-homem = Male
familia-sexo-mulher = Female
familia-sexo-nao-informar = Prefer not to say
familia-ocupacao = Profession or occupation
familia-observacoes = Notes
familia-dependente-ir = Dependent for income tax
familia-sem-cpf = The app does not store CPF numbers or documents.
familia-cadastro-salvo = Profile saved

## Allergies and dietary restrictions (docs/gtin-inflation.md §7.2)

familia-dieta-titulo = Allergies and dietary restrictions
familia-dieta-dica = Optional. Stays only in your vault; Treasury warns you in 06 · Consumption › Health when a receipt has an item with any of these (according to Open Food Facts).
familia-dieta-gluten = Gluten
familia-dieta-milk = Milk (lactose)
familia-dieta-eggs = Eggs
familia-dieta-peanuts = Peanuts
familia-dieta-nuts = Tree nuts
familia-dieta-soybeans = Soy
familia-dieta-fish = Fish
familia-dieta-crustaceans = Shellfish
familia-dieta-sesame = Sesame
familia-dieta-vegan = Vegan
familia-dieta-vegetarian = Vegetarian

## Income

familia-renda-alterada = Income changed
familia-renda-cadastrada = Income added
familia-editar-nome = Edit { $nome }
familia-nova-renda = New income
familia-renda-descricao = Reference gross and net amounts for the month. With an imported { $holerite }, the forecast uses its amounts.
familia-nome = Name
familia-nome-exemplo = Salary, retirement, side jobs…
familia-tipo = Type
familia-bruto-mes = Gross per month
familia-bruto-dica = Before deductions ({ $descontos }).
familia-opcional = Optional.
familia-liquido-mes = Net per month
familia-liquido-dica = What lands in the account.
familia-empregador-fonte = Employer or payer
familia-registro-empresa = Company registration
familia-registro-dica = Links the { $holerites } to this income automatically.
familia-dia-pagamento = Payday
familia-mes-ferias = Vacation month
familia-mes-ferias-dica = The { $ferias } goes into the forecast.
familia-sem-ferias = No vacation scheduled
familia-comeca-em = Starts on
familia-termina-em = Ends on
familia-termina-dica = Temporary contract, benefit with an end date.
familia-recebe = Receives { $extra }
familia-valor-variavel = Amount changes from month to month
familia-valor-variavel-dica = Commission, side jobs, sales: the forecast uses the average of the last 6 months of transactions in the income's category, when there are any.

## Payslip

familia-importar-holerite = Import { $holerite }
familia-importar-descricao = The { $holerite } PDF: you check it before saving. Names and personal documents are not stored.
familia-holerite-pdf = { $holerite } as PDF
familia-senha-se-tiver = Password (if any)
familia-ler-de-novo = Read again
familia-lendo = Reading the { $holerite }…
familia-competencia = Reference month
familia-empregador = Employer
familia-renda = Income
familia-renda-nova = new ({ $tipo })
familia-bruto = Gross
familia-outros-descontos = Other deductions
familia-liquido = Net
familia-salario-conta = Salary in the account
familia-nao-achado = not found
familia-competencia-mes = Reference month (YYYY-MM)
familia-sem-mes = The { $holerite } did not include the month.
familia-ja-importado = This { $holerite } was already imported: saving replaces the previous one.
familia-holerite-importado = { $holerite } imported
familia-gravar = Save { $holerite }

## Each member's accounts and cards

familia-contas-de = { $nome }'s accounts and cards
familia-contas-descricao = Transactions in an account or card that belongs to a single member go to that member, including old ones (except the ones you changed by hand). A joint account, with two or more members, belongs to the family.
familia-conjunta-com = joint with { $nomes }
familia-de-outros = belongs to { $nomes }
familia-pronto = Done

## Getting started from the questionnaire

familia-inicio-fixa-calcula = In the questionnaire, you said your main income is fixed. Add your salary with the gross and net amounts, or import a { $holerite }: the app calculates { $itens }.
familia-inicio-fixa-recibo = In the questionnaire, you said your main income is fixed. Add your salary with the gross and net amounts, or import a { $holerite } (deductions come from the pay stub; the app does not calculate this country's taxes).
familia-dica-fixa = Good practice: set your savings aside on payday, before spending (pay yourself first).
familia-inicio-variavel = In the questionnaire, you said your income varies. Add it as variable income: the forecast uses the average of the last 6 months.
familia-dica-variavel = Good practice: plan the month around your weakest months and keep an emergency fund of 6 to 12 months of expenses.
familia-inicio-beneficios = In the questionnaire, you said your income comes from benefits. Add each benefit ({ $exemplos }) with its monthly amount.
familia-beneficios-br = retirement, BPC, Bolsa Família
familia-beneficios-pt = retirement pension, family allowance, unemployment benefit
familia-beneficios-us = Social Security, SNAP, unemployment insurance
familia-beneficios-outros = retirement, pension, government benefits
familia-dica-beneficios = Good practice: the benefit is the steady part of the month; organize your fixed bills around it and treat the rest as extra.
familia-inicio-mista = In the questionnaire, you said your income is mixed. Add each part: the salary (or benefit) and the variable income separately.
familia-dica-mista = Good practice: count on the fixed part for your bills and save part of the variable income when it comes in.
familia-inicio-padrao = Add the income of each family member: salary, retirement, benefits, side jobs or rent. It is the basis for the balance forecast, the budget and the tips.
familia-dica-padrao = With your income added, the app knows how much comes in each month and how much you can save.
familia-comece = Start with the family's income
familia-boas-praticas-em = Good practices in { $pais }
familia-cadastrar-primeira = Add the first income

## New member

familia-membro-adicionado = Member added
familia-novo-membro = New family member
familia-adicionar = Add

## Income panel

familia-bruto-12m = Gross over the next 12 months
familia-liquido-12m = Net over the next 12 months
familia-descontos-previstos = { $descontos } expected
familia-impostos-holerites = Taxes on { $holerites } (12 months)
familia-renda-prevista = Expected income by month
familia-renda-fonte = Source: income sources added, the latest { $holerite } for each one and this year's { $descontos } rules
familia-renda-real-prevista = Income by month: actual and forecast
familia-renda-fonte-base = Actual: imported { $holerites }. Forecast: latest { $holerite } ({ $mes }) and this year's { $descontos } rules
familia-renda-tipo = Type
familia-renda-real = Actual
familia-renda-previsto = Forecast
familia-renda-real-faixa = ACTUAL
familia-renda-previsto-faixa = FORECAST
familia-renda-linha-prevista = Net forecast from the latest { $holerite }
familia-renda-diferenca = Difference from forecast
familia-renda-diferenca-coluna = Actual − forecast
familia-renda-igual = same
familia-renda-sem-holerite = No { $holerite } this month.
familia-renda-nota-real = Forecast: what the latest { $holerite } ({ $mes }) would give for this month.
familia-renda-nota-previsto = Forecast from the latest { $holerite } ({ $mes }).
familia-mes = Month
familia-extras = Extras
familia-cadastro = Profile
familia-dependente-ir-curto = income tax dependent
familia-cadastro-incompleto = Incomplete profile: relationship, date of birth and occupation help with the estimates.
familia-aliquota-efetiva = effective tax rate on payslips: { NUMBER($aliquota, maximumFractionDigits: 1) }%
familia-contas-cartoes = Accounts and cards:
familia-conta-conjunta-dica = Joint account: belongs to the family
familia-conjunta = (joint)
familia-nenhuma = none
familia-escolher = Choose
familia-rendas-de = { $nome }'s income
familia-nenhuma-renda = No income added.
familia-previsao = Forecast
familia-formal = formal
familia-informal = informal
familia-holerite-de = { $holerite } for { $mes }
familia-base-holerite = from the latest { $holerite }
familia-base-informados = from the amounts entered
familia-base-media = from the average of the last 6 months
familia-base-nenhum = no amount entered
familia-sem-membro = Accounts and cards without a member
familia-sem-membro-explicacao = Choose who each account belongs to: its transactions then get that member automatically. Leave without a member what belongs to everyone.
familia-membro-de = Member for { $nome }
familia-da-familia = The family's
familia-holerites-importados = { $holerites } imported
familia-membro = Member
familia-na-conta = In the account
familia-ligado = linked

## Simulate dismissal (CLT income, Brazil only)

familia-rescisao-botao = Simulate dismissal
familia-rescisao-titulo = Simulate dismissal · { $renda }
familia-rescisao-descricao = What you would receive when leaving this job: the termination items, the FGTS and unemployment insurance. An estimate under the general CLT rules; nothing is saved.
familia-rescisao-motivo = Reason
familia-rescisao-motivo-op = { $motivo ->
    [sem_justa_causa] Dismissal without just cause
    [pedido_demissao] Resignation
    [acordo] Mutual agreement (art. 484-A)
    [justa_causa] Dismissal for just cause
    [fim_experiencia] End of trial contract
    [quebra_experiencia_empregador] Trial ended early by the employer
   *[quebra_experiencia_empregado] Trial ended early by you
}
familia-rescisao-data = Last day of work
familia-rescisao-aviso = Notice period
familia-rescisao-aviso-op = { $aviso ->
    [indenizado] Paid (not worked)
    [trabalhado] Worked
   *[nao_cumprido] Not served (30-day deduction)
}
familia-rescisao-fim-contrato = Agreed end of the trial
familia-rescisao-admissao = Hiring date
familia-rescisao-ferias-vencidas = Overdue vacation (days)
familia-rescisao-ferias-dica = Vacation days from completed periods that you did not take.
familia-rescisao-saldo-fgts = FGTS balance for this job
familia-rescisao-saldo-fgts-dica = Empty: the one in the FGTS account linked to this income, if any.
familia-rescisao-pedidos = Already received unemployment insurance
familia-rescisao-pedidos-op = { $n ->
    [0] Never
    [1] Once
   *[other] Twice or more
}
familia-rescisao-preencha = Enter the dates to see the simulation.
familia-rescisao-verbas = Termination
familia-rescisao-liquido = Net termination pay
familia-rescisao-fgts = FGTS
familia-rescisao-fgts-saldo = Balance for termination purposes
familia-rescisao-fgts-deposito = Deposit on the termination items
familia-rescisao-fgts-multa = { $pct }% fine
familia-rescisao-fgts-sacavel = Can withdraw
familia-rescisao-seguro = Unemployment insurance
familia-rescisao-seguro-parcelas = { $n } installments of
familia-rescisao-seguro-total = Insurance total
familia-rescisao-total = Total available
familia-rescisao-meses = Covers about { $meses } months of your current net salary.
familia-rescisao-tabelas = Contract projected until { $fim }. INSS and IRRF: { $base }.
