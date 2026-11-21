# Family income and payslips (core, `income`; docs/income.md): extra payments in the forecast,
# notes from the forecast and from reading the payslip, change history reasons and errors. The
# labels in quotes are the document's own (a Brazilian payslip) and stay as they are.
# Revision: AI draft, not yet reviewed by a native speaker.

nucleo-renda-erro-renda = income { $id }
nucleo-renda-erro-bruto-negativo = the gross amount can't be negative
nucleo-renda-erro-liquido-negativo = the net amount can't be negative
nucleo-renda-erro-liquido-maior = the net amount can't be greater than the gross amount
nucleo-renda-erro-dia-pagamento = payday from 1 to 31
nucleo-renda-erro-mes-ferias = vacation month from 1 to 12
nucleo-renda-erro-termina-antes = the income ends before it starts

## Extra payments (forecast and payslips other than the monthly one)

nucleo-renda-extra-13-1 = 1st installment of the 13º
nucleo-renda-extra-13-2 = 2nd installment of the 13º
nucleo-renda-extra-terco-ferias = vacation bonus (one third)
nucleo-renda-extra-subsidio-natal = Christmas allowance
nucleo-renda-extra-subsidio-ferias = holiday allowance
nucleo-renda-extra-paga-junho = June extra pay
nucleo-renda-extra-paga-dezembro = December extra pay
nucleo-renda-extra-ferias = vacation pay
nucleo-renda-extra-plr = profit sharing (PLR)
nucleo-renda-extra-adiantamento = advance
nucleo-renda-extra-outro = other payment

## Forecast notes

nucleo-renda-nota-sem-valor = { $renda }: enter the gross or net amount (or import a payslip) to include it in the forecast.
nucleo-renda-nota-br = INSS and IRRF calculated with this year's rules when they don't come from the payslip; alimony, health insurance and other deductions only come in through the payslip.
nucleo-renda-nota-us = FICA and federal tax estimated with the 2026 rules when they don't come from the paystub, with the standard deduction and the filing status from the family registry (single, head of household or married filing jointly); state tax, 401(k) and health insurance only come in through the paystub.
nucleo-renda-nota-pt = Segurança Social and IRS estimated with the 2026 rules when they don't come from the payslip (IRS from the annual tax brackets, not the withholding tables).
nucleo-renda-nota-es = Seguridad Social and IRPF estimated with the 2026 rules when they don't come from the payslip (IRPF from the general scale, without each autonomous community's own scale).
nucleo-renda-extra-aguinaldo = aguinaldo (SAC)
nucleo-renda-nota-adaptado = Contributions and tax estimated with the country's 2026 rules when they don't come from the payslip (an approximation: without personal deductions and credits beyond the basic ones).
nucleo-renda-nota-outro = Salary deductions aren't calculated for this country: enter the net amount or import the payslip.

## Change history

nucleo-renda-motivo-criada = Income added: { $renda }
nucleo-renda-motivo-alterada = Income changed: { $renda }
nucleo-renda-motivo-apagada = Income deleted: { $renda }
nucleo-renda-motivo-holerite-importado = Payslip imported
nucleo-renda-motivo-holerite-apagado = Payslip deleted

## Reading and importing the payslip

nucleo-renda-aviso-sem-competencia = couldn't find the payslip's reference month (month/year): enter it when importing
nucleo-renda-aviso-nao-bate = gross minus deductions ({ $calculado }) doesn't match the net amount ({ $liquido }): check the lines
nucleo-renda-aviso-sem-vencimentos = couldn't find the payslip's earnings
nucleo-renda-aviso-escolha-renda = choose the income or the member (to create an income for this employer)
nucleo-renda-erro-base64 = { $arquivo }: invalid base64
nucleo-renda-erro-imagem = { $arquivo }: this is an image, not a PDF. Read the values from the image and send the payslip text in `text` (the lines with label and value, such as "Líquido a receber 2.043,36")
nucleo-renda-erro-pdf-senha = password-protected PDF: enter the payslip's password
nucleo-renda-erro-pdf-ilegivel = couldn't read the PDF: { $erro }
nucleo-renda-erro-sem-arquivo = send the payslip PDF (or its text)
nucleo-renda-falta-bruto = the gross total ("Total de vencimentos", "Total de proventos" or "Total bruto")
nucleo-renda-falta-liquido = the net amount ("Líquido a receber" or "Valor líquido")
nucleo-renda-faltam-dois = { $um } or { $outro }
nucleo-renda-erro-incompleto = the text looks like a payslip, but I couldn't find { $faltou }. Send the document's full text, without summarizing or rewriting it (the lines with label and value, such as "Líquido a receber 2.043,36"), or the PDF attached in `attachments` (e.g. ["1"])
nucleo-renda-erro-competencia = enter the payslip's reference month (YYYY-MM)
nucleo-renda-erro-ilegivel = the payslip has no readable gross and net amounts
nucleo-renda-erro-escolha-renda = choose the payslip's income or member
nucleo-renda-erro-duplicado = there's already a payslip for { $mes } for this income; use replace to swap it
nucleo-renda-erro-holerite = payslip { $id }
# Name of the income created when importing a payslip from a new employer.
nucleo-renda-nome-salario-de = Salary · { $empregador }
nucleo-renda-nome-salario = Salary
nucleo-renda-aviso-sem-sinal = { $linhas } without a deduction sign, but the printed totals only add up with them as deductions: check
nucleo-renda-aviso-totais = the lines don't add up to the printed totals (earnings { $vencimentos } × { $bruto }, deductions { $descontos } × { $total_descontos }): the totals win, check the lines
nucleo-renda-mensal = monthly
nucleo-renda-aviso-plr-irrf = Profit-sharing (PLR) tax withheld ({ $retido }) differs from the PLR exclusive table ({ $esperado }), counting earlier PLR payments this year. Check it (alimony deducted lowers the tax).

## Termination simulation (`income.simulate_termination`)

nucleo-renda-erro-rescisao-data = `{ $campo }` must be a YYYY-MM-DD date
nucleo-renda-erro-rescisao-salario = enter the gross salary or choose an income with a gross salary
nucleo-renda-erro-rescisao-admissao = enter the hiring date
nucleo-renda-erro-rescisao-ordem = the termination cannot be before the hiring date
nucleo-renda-erro-rescisao-fim-contrato = enter the agreed end of the trial contract
nucleo-renda-erro-rescisao-tabela = no INSS and IRRF table for the termination month
nucleo-renda-rescisao-saldo-salario = Salary balance
nucleo-renda-rescisao-aviso-previo = Paid notice period
nucleo-renda-rescisao-decimo-terceiro = Prorated 13th salary
nucleo-renda-rescisao-ferias-proporcionais = Prorated vacation + 1/3
nucleo-renda-rescisao-ferias-vencidas = Overdue vacation + 1/3
nucleo-renda-rescisao-indenizacao-479 = Trial contract indemnity (art. 479)
nucleo-renda-rescisao-desconto-480 = Deduction for ending the trial contract early (art. 480)
nucleo-renda-rescisao-desconto-aviso = Deduction for notice not served
nucleo-renda-rescisao-inss = INSS on the salary balance
nucleo-renda-rescisao-irrf = IRRF on the salary balance
nucleo-renda-rescisao-inss-13 = INSS on the 13th
nucleo-renda-rescisao-irrf-13 = IRRF on the 13th
nucleo-renda-rescisao-n-dias = { $n } days
nucleo-renda-rescisao-n-dias-terco = { $n } days + 1/3
nucleo-renda-rescisao-metade-dias = half of { $n } days
nucleo-renda-rescisao-aviso-estimativa = Estimate under the general CLT rules. The official amount is the employer's termination statement (TRCT); collective agreements, averages of overtime and commissions, absences and advances can change the result.
nucleo-renda-rescisao-aviso-13-adiantado = If the 1st installment of the 13th was already paid, it is deducted from the 13th in the termination.
nucleo-renda-rescisao-aviso-sem-fgts = Without this job's FGTS balance, the fine and the withdrawal only cover the month's deposit. Enter the balance or link the FGTS account to the income.
nucleo-renda-rescisao-aviso-aniversario = Under the anniversary withdrawal option the FGTS balance stays locked on dismissal; only the fine can be withdrawn.
nucleo-renda-rescisao-seguro-motivo = Unemployment insurance is only for dismissal without just cause.
nucleo-renda-rescisao-seguro-carencia = Not eligible: the claim number { $pedido } needs { $meses } months worked in the last { $janela }.
nucleo-renda-rescisao-tabela-antiga = date before the oldest table in the app
