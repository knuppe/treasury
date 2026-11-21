# Monthly PDF report (06 · Reports › Download PDF), built by the core on your own computer
# (reports/pdf_report.rs), and the UI button. Revision: AI draft, not yet reviewed by a native
# speaker.

## Interface

pdf-botao = Download month PDF
pdf-botao-dica = { $mes } report as a PDF, made on your computer without sending anything to the internet.
pdf-salvar-titulo = Save the report as PDF
pdf-arquivo = treasury-report-{ $mes }.pdf
pdf-feito = Report saved
pdf-feito-paginas = { $paginas ->
    [one] 1 page at { $caminho }
   *[other] { $paginas } pages at { $caminho }
}
pdf-erro = Couldn't create the PDF

## Document

pdf-titulo = { $mes } report
pdf-mes-ano = { $mes } { $ano }
pdf-base-competencia = Accrual (purchase date)
pdf-base-caixa = Cash (payment date)
pdf-receitas = Income
pdf-despesas = Expenses
pdf-sobra = Left over
pdf-taxa-poupanca = { $taxa } of income saved
pdf-patrimonio = Net worth
pdf-sem-cambio = No exchange rate for { $moedas }: those amounts are left out of the totals.
pdf-secao-categorias = Where the money went
pdf-secao-categorias-dica = This month's expenses by category, with each one's share of the total and the change from last month.
pdf-sem-gastos = No expenses this month.
pdf-col-categoria = Category
pdf-col-valor = Amount
pdf-col-parte = Share
pdf-col-mes-anterior = Last month
pdf-novo = new
pdf-secao-12-meses = Money in and out, 12 months
pdf-secao-inflacao = Inflation
pdf-inflacao-resumo = Inflation over 12 months: { $inflacao } ({ $indice }, latest data from { $data }). The adjusted line shows what each month's spending would be worth today.
pdf-inflacao-sem-indice = There's no inflation data for your country yet to adjust the amounts. Update the indexes in Assets › Indices and quotes.
pdf-inflacao-estimado = The dashed part uses months the index hasn't published yet (estimate).
pdf-inflacao-nominal = Spending in the month (nominal)
pdf-inflacao-corrigido = Adjusted for inflation (today's money)
pdf-secao-inflacao-categorias = Categories vs. inflation
pdf-secao-inflacao-categorias-dica = Average of the first three months with spending and the last three. Above inflation, the category really got more expensive (or you spent more); below, it got cheaper.
pdf-col-inicio = Start
pdf-col-fim = End
pdf-col-variacao = Change
pdf-col-inflacao = Inflation
pdf-col-real = Real
pdf-acima = { $pp } above
pdf-abaixo = { $pp } below
pdf-igual = same
pdf-secao-patrimonio = Net worth, 12 months
pdf-secao-vencimentos = Coming up (30 days)
pdf-sem-vencimentos = Nothing due in the next 30 days.
pdf-rodape = Made on your computer by Treasury on { $data }. Amounts in { $moeda }.
pdf-pagina = Page { $n } of { $total }
