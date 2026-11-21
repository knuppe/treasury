# IPTU (Brazilian municipal property tax) for homes and land: each year's bill in Assets, due-date
# reminders and IPTU in taxes paid (docs/brazil/iptu.md).
# Revision: AI draft, not yet reviewed by a native speaker.

iptu-titulo = IPTU
iptu-descricao = Each year's bill: assessed value, taxes, fees, discounts and due dates. Treasury reminds you of the due dates and adds IPTU to the taxes you paid.
iptu-registrar = Add { $ano } IPTU
iptu-registrar-outro = Add another year
iptu-vazio = No bill added yet. Add this year's IPTU to get due-date reminders and follow the assessed value.
iptu-editar = Edit
iptu-apagar = Delete
iptu-apagar-titulo = Delete the { $ano } IPTU
iptu-apagar-texto = The bill and the payments you marked are removed. You can undo it from the history.
iptu-editar-titulo = { $ano } IPTU · { $imovel }
iptu-novo-titulo = Add IPTU · { $imovel }

## Bill fields

iptu-ano = Year
iptu-cadastro = Property registration number (IQ)
iptu-cadastro-ajuda = The property's number at the city hall (inscrição imobiliária, indicação fiscal, SQL or índice cadastral), printed on the bill.
iptu-cidade = City
iptu-area-terreno = Land area (m²)
iptu-area-construida = Built area (m²)
iptu-secao-venal = Assessed value
iptu-valor-venal-terreno = Assessed value of the land
iptu-valor-venal-predial = Assessed value of the building
iptu-valor-venal-total = Total assessed value
iptu-secao-impostos = Taxes, fees and discounts
iptu-imposto-territorial = Land tax
iptu-imposto-predial = Building tax
iptu-total-impostos = Total taxes
iptu-taxa-iluminacao = Street lighting fee
iptu-taxa-lixo = Waste collection fee
iptu-outras-taxas = Other fees
iptu-desconto = Discount
iptu-desconto-bom-pagador = Good-payer discount
iptu-total-devido = Total due
iptu-total-calcular = Calculate from the bill
iptu-secao-pagamento = Payment
iptu-forma = How you'll pay
iptu-forma-indefinida = Not decided yet
iptu-forma-unica = Single payment
iptu-forma-parcelado = Installments
iptu-cota-unica-vencimento = Single payment due date
iptu-cota-unica-valor = Single payment amount
iptu-parcelas-quantidade = Number of installments
iptu-parcelas-primeiro = First due date
iptu-parcelas-valor = Amount of each installment
iptu-parcelas-ajuda = Installments fall due once a month, starting on the first due date.
iptu-notas = Notes
iptu-salvar = Save
iptu-cancelar = Cancel
iptu-salvo = { $ano } IPTU saved
iptu-erro-sem-total = Enter the total due.

## Installments and payment

iptu-parcelas-titulo = { $ano } IPTU installments

iptu-cota-unica = Single payment
iptu-parcela-n = Installment { $n }
iptu-coluna-parcela = Installment
iptu-coluna-vencimento = Due date
iptu-coluna-valor = Amount
iptu-coluna-situacao = Status
iptu-situacao-paga = Paid on { $data }
iptu-situacao-aberta = Open
iptu-situacao-vencida = Overdue
iptu-situacao-outra-forma = Other option
iptu-marcar-pago = Mark as paid
iptu-desfazer-pagamento = Undo payment
iptu-pagamento-marcado = Payment marked
iptu-pagamento-desfeito = Payment undone
iptu-pagar-titulo = Mark { $parcela } as paid
iptu-lancamentos-provaveis = Payment transaction
iptu-nenhum-lancamento = No similar transaction near the due date.
iptu-pago-sem-lancamento = Paid without a transaction in the app, on
iptu-sugestao = Looks paid: { $descricao } on { $data }
iptu-confirmar-sugestao = Confirm

## Derived figures

iptu-aliquota-efetiva = Effective rate
iptu-pago = Paid
iptu-em-aberto = Open
iptu-variacao-venal = Assessed value { $variacao } vs { $ano }
iptu-variacao-imposto = Taxes { $variacao } vs { $ano }
iptu-ipca-ano = { $ano } IPCA: { $ipca }
iptu-mercado = Assessed value = { $proporcao } of market value ({ $valor } on { $data })
iptu-cota-x-parcelas = Single payment of { $unica } × { $n } installments adding up to { $parcelas }
iptu-historico-titulo = Assessed value history
iptu-coluna-ano = Year

## Insights (core)

iptu-percepcao-a-vista-cdi = Paying the single amount saves { $economia }. The discount is worth { $taxa }% a month, more than the CDI ({ $cdi }% a month): pay upfront if the money is already set aside.
iptu-percepcao-a-vista = Paying the single amount saves { $economia }, a discount worth { $taxa }% a month.
iptu-percepcao-parcelado = The single payment saves { $economia }, but the discount ({ $taxa }% a month) earns less than the CDI ({ $cdi }% a month): paying in installments and keeping the money invested comes out slightly ahead.
iptu-percepcao-acima-ipca = IPTU went up { $variacao }% from last year, above the { $ano } IPCA ({ $ipca }%). Check the bill for changes in area, use or the city's value map; the city hall accepts a review request within the deadline on the bill.
iptu-percepcao-venal-acima = The assessed value is { $diferenca }% above the property's market value in the app. If that valuation is right, you can ask the city hall to review the assessed value (an appeal within the deadline on the bill).
iptu-percepcao-venal-abaixo = The assessed value is { $proporcao }% of the property's market value. It's usually below market; don't use it as an estimate of the property's price.
iptu-percepcao-total-diferente = Taxes + fees − discounts comes to { $calculado }, but the total due is { $total }. Check the amounts on the bill.

## Errors (core)

iptu-imovel = property
iptu-carne = IPTU bill
iptu-parcela = IPTU installment
iptu-erro-tipo = IPTU only applies to homes and land.
iptu-erro-ano = Invalid IPTU year.
iptu-erro-negativo = IPTU amounts can't be negative.
iptu-erro-area = Invalid area: use square meters, like 250 or 72.5.
iptu-erro-parcela = A bill has at most 24 installments.
iptu-erro-parcela-valor = Each installment needs an amount greater than zero.
iptu-erro-parcela-repetida = Two installments have the same number.
iptu-erro-lancamento-usado = That transaction already pays another IPTU installment.
iptu-erro-lancamento-saida = The payment must be an outflow (negative amount).
iptu-erro-pagamento = Choose the payment transaction or enter the date.

## Change history (core)

iptu-historico-salvo = { $ano } IPTU added
iptu-historico-apagado = IPTU deleted
iptu-historico-pago = IPTU installment paid
iptu-historico-desfeito = IPTU payment undone

## Reminders (core)

iptu-quando-hoje = today
iptu-quando-amanha = tomorrow
iptu-quando-dias = in { $dias } days
iptu-alerta-titulo = { $ano } IPTU · { $imovel }: { $parcela } due { $quando }
iptu-alerta-titulo-vencido = { $ano } IPTU · { $imovel }: { $parcela } is overdue
iptu-alerta-corpo = { $valor } due on { $data }. Once paid, mark it in Portfolio › Assets › { $imovel } › IPTU.
iptu-alerta-corpo-decidir = The single payment of { $valor } is due on { $data }. See in Portfolio › Assets › { $imovel } › IPTU whether to pay upfront or in installments.
iptu-alerta-falta-titulo = { $ano } IPTU · { $imovel }: add the bill
iptu-alerta-falta-corpo = The { $ano } bill usually arrives around now. Add it in Portfolio › Assets › { $imovel } › IPTU to get due-date reminders and follow the assessed value.

## Taxes paid (core)

iptu-impostos-nota = IPTU from the bills added in Assets (what falls due in the period). Record the payment in an IPTU category to use the amount actually paid.
iptu-impostos-linha = IPTU from the bill · { $imovel }
iptu-impostos-valor = Assessed value
