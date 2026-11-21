# Reports › IR: monthly calculation of Brazilian income tax on stock-market gains, DARFs (federal
# tax payment slips) and the yearly report for the tax return (Brazil only; docs/i18n-design.md).
# Official terms (DARF, IRRF, IR, JCP, FII, CNPJ, Sicalc, Receita Federal) stay as is.
# Revision: AI draft, not yet reviewed by a native speaker.

## Month status, loss groups and DARF

ir-mes-sem-vendas = No sales
ir-mes-sem-vendas-dica = No stock-market sales in the month.
ir-mes-sem-imposto = No tax
ir-mes-sem-imposto-dica = There were sales, but no tax: exemption, loss or offset.
ir-mes-abaixo-minimo = Below R$ 10
ir-mes-abaixo-minimo-dica = Tax below R$ 10.00: no DARF is paid; the amount carries over to the next month.
ir-mes-a-pagar = To pay
ir-mes-a-pagar-dica = Tax is due by the last business day of the following month.
ir-mes-pago = Paid
ir-mes-pago-dica = The month's DARF is paid.
ir-grupo-comum = Regular (stocks, ETFs, BDRs)
ir-grupo-comum-curto = Regular
ir-grupo-day-trade = Day trade
ir-grupo-fii = FIIs (real estate funds)
ir-grupo-fii-curto = FII
ir-darf-em-aberto = Open
ir-darf-cancelado = Cancelled
ir-darf-vencido = Overdue
ir-aviso = A helper calculation made with your data; check it in the Receita Federal (Brazilian tax authority) program. This is not tax advice.

## Panel

ir-secao-apuracao = Monthly calculation
ir-secao-darf = DARFs
ir-secao-declaracao = Yearly return
ir-aviso-titulo = Not tax advice
ir-aviso-regras = Stock-market rules for individuals (15% on regular trades, 20% on day trades and FIIs, exemption for stock sales up to R$ 20,000 a month).
ir-ano-calendario = Calendar year
ir-membro = Member
ir-familia-toda = Whole family
ir-secao = IR section
ir-apurando = Calculating the year
ir-imposto-em = Tax in { $ano }
ir-renda-variavel = Stock market
ir-darfs-total = DARFs: { $valor }
ir-a-pagar = To pay
ir-meses-sem-darf = Months without a DARF
ir-meses-sem-darf-n = { $n ->
    [one] { $n } month without a DARF
   *[other] { $n } months without a DARF
}
ir-darfs-vencidos = { $n ->
    [one] ! { $n } DARF overdue
   *[other] ! { $n } DARFs overdue
}
ir-prejuizos = Losses
ir-somando-grupos = All three groups combined
ir-fim-periodo = at the end of the calculated period
ir-carregando-darfs = Loading DARFs
ir-sem-darf-em = Tax due without a DARF in { $meses }.
ir-fonte-regras = stock-market rules in force in { $ano }

## Copy

ir-copiar = Copy
ir-copiado = ✓ Copied
ir-copiar-algo = Copy { $algo }
ir-copiar-a-mao = Select the value and copy it by hand.

## Month calculation

ir-meses-vazio = No stock-market sales this year.
ir-mes = Month
ir-vendas-acoes = Stock sales
ir-isento = exempt
ir-imposto = Tax
ir-para-proximo = { $valor } → next
ir-situacao = Status
ir-grupo = Group
ir-resultado = Result
ir-prejuizo-antes = Loss before
ir-compensado = Offset
ir-base = Base
ir-aliquota = Rate
ir-prejuizo-compensar = Loss to offset
ir-apuracao-de = Calculation for { $mes }
ir-darf-gerado = DARF created
ir-darf-gerado-texto = Code { $codigo }, due on { $vencimento }. Issue the slip in Sicalc with these amounts.
ir-gerar-darf = Create DARF for { $valor }
ir-ver-darf = See the DARF
ir-notas-apuracao = Calculation notes
ir-vendas-categoria = Sales and result by category
ir-categorias-de = Categories for { $mes }
ir-categoria = Category
ir-vendas = Sales
ir-ganho-isento = Exempt gain
ir-prejuizo-mesmo-grupo = a loss only offsets gains in the same group
ir-prejuizos-base = Losses and tax base
ir-grupos-prejuizo = Loss groups
ir-imposto-mes = Month's tax
ir-irrf-estimado = IRRF (≈ estimated)
ir-irrf-mes = Month's IRRF
ir-irrf-anteriores = IRRF from previous months
ir-irrf-compensado = IRRF offset
ir-devido-mes = Due this month
ir-acumulado = Carried over (< R$ 10)
ir-valor-darf-de = DARF amount for { $periodo }
ir-codigo-receita = Revenue code
ir-periodo-apuracao = Calculation period
ir-vencimento = Due date
ir-vai-mes-seguinte = Carries to next month
ir-irrf-depois = IRRF to offset later
ir-vendas-apuracao = Sales included in the calculation
ir-vendas-de = Sales for { $mes }
ir-data = Date
ir-ativo = Asset
ir-quantidade = Quantity
ir-venda = Sale
ir-custo-medio = Average cost

## DARFs

ir-darfs-venceram = { $n ->
    [one] ! One DARF is past due without payment: recalculate it in Sicalc with fine and interest before paying.
   *[other] ! { $n } DARFs are past due without payment: recalculate them in Sicalc with fine and interest before paying.
}
ir-darfs-vazio = No DARF created this year. Create one from the calculation of a month with tax due.
ir-periodo = Period
ir-codigo = Code
ir-valor = Amount
ir-acoes = Actions
ir-cancelar = Cancel
ir-marcar-pago-de = Mark the { $periodo } DARF as paid
ir-marcar-pago = Mark paid
ir-lancamento = Transaction
ir-cancelar-darf = Cancel DARF
ir-cancelar-darf-texto = The { $periodo } DARF leaves the list and the month goes back to tax due (you can create another one). You can undo this in the change history.
ir-darf-cancelado-toast = DARF cancelled
ir-darf-pago-titulo = { $periodo } DARF paid
ir-darf-pago-descricao = Code { $codigo }, due on { $vencimento }.
ir-darf-pago = DARF paid
ir-despesa-lancada = Expense recorded in Taxes and fees › Income tax.
ir-marcar-como-pago = Mark as paid
ir-data-pagamento = Payment date
ir-lancar-despesa = Record the expense in account
ir-lancar-despesa-dica = Optional: creates the expense in Taxes and fees › Income tax. If you already imported the statement, leave it blank.
ir-nao-lancar = Don't record
ir-pago-atrasado = Paid after the due date: the amount with fine and interest (Sicalc) may be higher than the one recorded here.

## Yearly return

ir-ficha-isentos = Rendimentos isentos (exempt income)
ir-ficha-exclusiva = Tributação exclusiva (taxed at source)
ir-linha-dividendos = Profits and dividends received
ir-linha-isentos-acoes = Net gains on stocks with sales up to R$ 20,000 in the month
ir-linha-fii = FII income
ir-linha-jcp = Interest on equity (JCP)
ir-n-ativos = { $n ->
    [one] { $titulo } · { $n } asset
   *[other] { $titulo } · { $n } assets
}
ir-copiar-de = { $titulo } from { $ativo }
ir-rendimentos-ano = Income · calendar year { $ano }
ir-rendimentos-declaracao = Income for the tax return
ir-ficha = Section
ir-codigo-muda = The code changes between versions of the program: check it
ir-confira = check
ir-descricao = Description
ir-dividendos = Dividends
ir-jcp = JCP
ir-rendimentos-nota = Net amounts received in { $ano }, in reais. The program asks for the payer (CNPJ, company tax ID) of each line: check the broker's income statement.
ir-resumo-renda-variavel = Stock market · year summary
ir-impostos-ano = Taxes for the year
ir-imposto-apurado = Tax calculated
ir-darfs-ano = DARFs for the year
ir-total-darfs = DARF total
ir-ganhos-isentos = Exempt gains (stocks)
ir-ganhos-isentos-copiar = exempt stock gains
ir-prejuizos-31-12 = Losses to offset at 12/31
ir-prejuizo-de = { $grupo } loss
ir-bens-em = Assets and rights at 12/31/{ $ano }
ir-bens-situacao = acquisition cost (average price), not market value
ir-codigos = Codes
ir-codigos-sugeridos = suggested: check them in the program
ir-bens-direitos = Assets and rights
ir-bens-vazio = No positions at year end.
ir-grupo-codigo = Group · code
ir-discriminacao = Description
ir-discriminacao-de = description of { $ativo }
ir-situacao-em = position at 12/31/{ $ano } for { $ativo }
ir-cripto-titulo = Crypto in Brazil (capital gain)
ir-cripto-vendido = Crypto sold in the month (Brazil and abroad)
ir-cripto-vendido-brasil = Sold at exchanges in Brazil
ir-cripto-ganho = Gain on sales in Brazil
ir-cripto-perda = Losses (do not reduce the gain)
ir-cripto-situacao = Status
ir-cripto-isento = Exempt: sales up to R$ 35,000 in the month
ir-cripto-tributado = Taxed: sales above R$ 35,000 in the month
ir-cripto-nota = Crypto held in Brazil: 15% to 22.5% tax on the gain, in its own DARF (code 4600), when total crypto sold in the month, in Brazil and abroad, exceeds R$ 35,000. Crypto held abroad goes to Investments abroad, in the yearly report.
ir-cripto-exterior = Abroad
ir-cripto-brasil = Brazil
ir-cripto-custodia-auto = Automatic ({ $lugar })
ir-cripto-custodia-de = Crypto custody at { $conta }
ir-cripto-custodia-dica = Where each account's crypto is held. On automatic, a BRL account counts as Brazil and an account in another currency as abroad.
ir-cripto-darf-pago = DARF 4600 paid
ir-cripto-darf-gerado = DARF 4600 generated
ir-irrf-sobra-declaracao = "Dedo-duro" withholding for the return (Tax paid/withheld)
ir-irrf-sobra-declaracao-rotulo = Lei 11.033 withholding for the return
ir-irrf-sobra-day-trade = Unused day-trade withholding (refund only)
ir-exterior = Investments abroad (Law 14,754/2023)
ir-ganho-realizado = Gain realized in the year
ir-ganho-exterior = gain abroad
ir-dividendos-juros = Dividends and interest (gross)
ir-rendimentos-exterior = income from abroad
ir-imposto-pago-exterior = Tax paid abroad
ir-imposto-pago-exterior-rotulo = tax paid abroad
ir-credito-exterior = Credit used (up to 15% of each income)
ir-sem-credito-exterior = No credit (country without a treaty or reciprocity)
ir-prejuizo-exterior-anterior = Losses from previous years
ir-prejuizo-exterior-seguinte = Losses carried to next years
ir-imposto-estimado-15 = Estimated tax (15%)
ir-exterior-nota = Calculated in the yearly return: 15% on the gain (after losses from previous years) and on gross dividends and interest, minus the tax paid abroad, which only counts up to the Brazilian tax on each income. Includes in the gain crypto held in accounts in another currency (crypto in BRL accounts is calculated monthly, with DARF 4600).
ir-anual-titulo = { $ano } tax return (preview)
ir-anual-carregando = Building the tax return preview…
ir-anual-dica = Values for the year taken from the vault. Check them against the income statements and receipts and adjust as needed: both models are recalculated right away, and nothing is saved.
ir-anual-campo-income = Taxable income from companies (without 13th salary and profit sharing)
ir-anual-campo-social_security = Official social security (INSS)
ir-anual-campo-withheld = Tax withheld at source
ir-anual-campo-self_employed = Income from individuals and from abroad
ir-anual-campo-carne_leao = Carnê-leão paid
ir-anual-campo-medical = Medical expenses
ir-anual-campo-education = Education expenses
ir-anual-campo-private_pension = Private pension (PGBL)
ir-anual-campo-alimony = Alimony (court order or deed)
ir-anual-campo-complementary_tax = Complementary tax
ir-anual-campo-irrf_lei_11033 = "Dedo-duro" withholding (Lei 11.033)
ir-anual-campo-dependentes = Dependents
ir-anual-completo = Itemized
ir-anual-simplificado = Simplified
ir-anual-mais-vantajoso = Better option
ir-anual-deducoes = Deductions
ir-anual-base = Tax base
ir-anual-imposto-tabela = Tax from the table
ir-anual-reducao = Lei 15.270 reduction
ir-anual-doacoes = Donations (children and elderly funds)
ir-anual-devido = Tax due
ir-anual-pago = Tax already paid
ir-anual-a-pagar = Balance to pay
ir-anual-a-restituir = Refund
ir-anual-aliquota-efetiva = Effective rate in the better option: { $aliquota }.
ir-anual-restaurar = Back to the vault values
ir-anual-de-onde = Where the values come from
ir-dividendos-acima-titulo = Dividends above R$ 50,000 in a month from one company
ir-dividendos-acima-linha = { $mes }: { $ativos }, { $valor } → 10% withheld on the total ({ $retencao })
ir-dividendos-acima-nota = Since 2026 the company withholds 10% of everything it pays in a month above this limit (Lei 15.270/2025). Profits up to 2025 with distribution approved by 31/12/2025 are excluded: check the company's statement. The amount withheld counts toward the minimum tax on high incomes.
ir-irpfm-titulo = Minimum tax on high incomes (IRPFM)
ir-irpfm-rendimentos = Income for the year in the base
ir-irpfm-bruto = IRPFM before deductions ({ $aliquota })
ir-irpfm-creditos = Return, exclusive and definitive tax
ir-irpfm-devido = IRPFM to pay (negative: to offset)
ir-darf-atraso-hoje = Paying today: { $total } (fine { $multa } + interest { $juros })
ir-darf-atraso-hoje-estimado = Paying today: ≈ { $total } (fine { $multa } + interest { $juros }; Selic estimated for some month)
ir-darf-com-acrescimos = Includes fine { $multa } and interest { $juros }
ir-carne-titulo = Monthly carnê-leão (income from individuals and from abroad)
ir-carne-dica = For income received from individuals or from abroad (rent from an individual tenant, private practice, work for abroad): the tax uses the monthly table and is paid by the end of the following month.
ir-carne-recebido = Received in the month
ir-carne-inss = Official social security paid
ir-carne-pensao = Alimony paid
ir-carne-livro-caixa = Livro-caixa (work expenses)
ir-carne-imposto = Carnê-leão for the month
ir-carne-darf = code { $codigo }, due { $vencimento }
ir-carne-abaixo-minimo = Below R$ 10: added to the next month
