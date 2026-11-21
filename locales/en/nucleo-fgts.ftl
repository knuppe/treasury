# FGTS (core: `fgts`; docs/brazil/fgts.md): Caixa statement, checks, withdrawals, anniversary
# withdrawal, projection and yield comparison.

nucleo-fgts-empregador = employer
nucleo-fgts-tipo-deposito = Deposit
nucleo-fgts-tipo-deposito-extra = Supplementary deposit
nucleo-fgts-tipo-jam = JAM (interest and indexation)
nucleo-fgts-tipo-ajuste-jam = JAM adjustment
nucleo-fgts-tipo-distribuicao = Profit distribution
nucleo-fgts-tipo-saque = Withdrawal
nucleo-fgts-tipo-reposicao = Withdrawal reversal
nucleo-fgts-saque-motivo-rescisao = dismissal
nucleo-fgts-saque-motivo-aposentadoria = retirement
nucleo-fgts-saque-motivo-moradia = housing
nucleo-fgts-saque-motivo-aniversario = anniversary withdrawal
nucleo-fgts-saque-motivo-doenca = illness
nucleo-fgts-saque-motivo-emergencial = emergency withdrawal
nucleo-fgts-saque-motivo-outros = other
nucleo-fgts-lancamento-saque = FGTS withdrawal ({ $motivo })

nucleo-fgts-motivo-extrato = Import FGTS statement
nucleo-fgts-motivo-saque = Record FGTS withdrawal
nucleo-fgts-motivo-moradia = Use FGTS on the home loan
nucleo-fgts-motivo-modalidade = Change the FGTS withdrawal option
nucleo-fgts-motivo-conta = Edit the FGTS account

nucleo-fgts-erro-extrato-vazio = No FGTS entries found in this statement (lines with date, amount and total).
nucleo-fgts-erro-sem-arquivo = Send the FGTS statement PDF or its text.
nucleo-fgts-erro-pdf = Could not read the statement PDF: { $erro }
nucleo-fgts-erro-nao-e-fgts = The chosen account is not an FGTS account.
nucleo-fgts-erro-renda = income source
nucleo-fgts-erro-motivo = Unknown withdrawal reason: { $motivo }.
nucleo-fgts-erro-valor-positivo = The amount must be greater than zero.
nucleo-fgts-erro-saldo-insuficiente = The amount exceeds the FGTS account balance on that date.
nucleo-fgts-erro-contrato = contract
nucleo-fgts-erro-contrato-imobiliario = FGTS can only be used on a home financing contract.
nucleo-fgts-erro-modalidade = The option must be dismissal withdrawal or anniversary withdrawal.
nucleo-fgts-erro-membro = member
nucleo-fgts-erro-nascimento = To project to an age, enter the account holder's birth date.
nucleo-fgts-erro-data-ou-idade = Enter the date or the age for the projection.

nucleo-fgts-aviso-saldo = { $n ->
    [one] One statement line does not match the printed balance.
   *[other] { $n } statement lines do not match the printed balance.
}
nucleo-fgts-aviso-jam = { $n ->
    [one] One JAM credit differs from the amount computed by the rule.
   *[other] { $n } JAM credits differ from the amount computed by the rule.
}
nucleo-fgts-aviso-juros-progressivos = The account has progressive interest (opted in before 1971): the app stores the rate but does not compute progressive interest.
nucleo-fgts-aviso-codigo-99 = Withdrawals with code 99 were treated as housing use; change the reason if they were not.
nucleo-fgts-aviso-cabecalho = Part of the statement header could not be read (employer, CNPJ or employment dates). Check these details on the account after importing.
nucleo-fgts-aviso-dois-anos = FGTS was used for housing on { $data }: paying down the loan with FGTS requires at least 2 years between uses (Law 8,036, art. 20).

nucleo-fgts-nota-aniversario = Estimate by the anniversary withdrawal table (Law 8,036, Annex): a single band, by the combined balance of the accounts. The change is requested in Caixa's FGTS app.
nucleo-fgts-nota-antecipacao = With an advance taken out, going back to the dismissal withdrawal only applies after paying off the advance.
nucleo-fgts-nota-conferencia = JAM is recomputed from the balance up to the previous credit, with the statement's coefficient, truncated to cents; deposits are compared with the FGTS on the payslips of the linked income.
nucleo-fgts-nota-extrato-recente = Months without a deposit may simply be outside the last imported statement: import a recent statement before asking the employer.
nucleo-fgts-nota-projecao = Projection with the latest TR ({ $tr }% a month) held constant, 3% a year interest, distribution by the average of the latest indices and { $ipca }% inflation over 12 months for the values in today's reais.
nucleo-fgts-nota-sem-renda = Active account without a linked income: the projection adds no new deposits.
nucleo-fgts-nota-comparar = The same deposits and withdrawals, on the same dates, invested in each option. 85% of CDI is only a yardstick for something exempt from income tax, like FGTS. Not a recommendation.
nucleo-fgts-nota-adi = Under ADI 5090 (Supreme Court, 06/12/2024), FGTS remuneration (JAM + distribution) must be at least the IPCA from then on, without retroactive effect; the compensation is set by the Board of Trustees.
nucleo-fgts-nota-sem-serie = A Central Bank series is missing from the vault (IPCA, savings, CDI or M2): sync the indices to complete the comparison.

nucleo-fgts-comparar-ipca = Indexed to IPCA
nucleo-fgts-comparar-poupanca = Savings account (poupança)
nucleo-fgts-comparar-cdi-85 = 85% of CDI
nucleo-fgts-comparar-cdi = 100% of CDI
nucleo-fgts-comparar-m2 = M2 expansion
nucleo-fgts-evento-aposentadoria = FGTS withdrawal at retirement
nucleo-fgts-nota-aposentadoria = The projected FGTS balance ({ $valor } in today's reais) counts as a withdrawal at retirement.
