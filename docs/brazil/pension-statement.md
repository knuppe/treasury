# Pension statement

Imports the "Extrato Financeiro do Participante" (participant financial statement) of pension
fund plans (EFPC, closed pension entities; first layout: Quanta Previdência) through the interface
(the asset's Pension tab, "Import statement (PDF)") or through the agent
(`preview_pension_statement` and `import_pension_statement`, the latter as a sensitive
proposal). Core in `crates/treasury-core/src/pension/statement.rs`; commands
`pension.statement_preview` (read only) and `pension.statement_import`.

## What is read

- **Header**: plan name (`PLANOPREV-Arrojado`), plan CNPJ, CNPB, enrollment, administration and
  load fees, and tax regime. The participant's name and CPF are not read.
- **Lines**: each one starts with `MM/AAAA DD/MM/AAAA DD/MM/AAAA` (competence, due date,
  payment date), followed by the description and nine numbers: amount paid, retirement, amount
  invested, return, return %, shares, disability, survivor pension and penalty. Reading does not
  depend on the columns or the spacing; descriptions broken across two lines
  ("Contribuições Extras" / "Empregador") do no harm.
- **Line type**: retirement > 0 is a contribution (the participant's, if the description says
  monthly; otherwise an extra one from the employer); only disability or only survivor pension
  are risk insurance.
- **Footer**: share balance, share value and date, and the participant's balance. The preview
  checks the sum of the shares against the share balance, and the balance by shares against the
  statement's balance, and warns if they do not match.

## What is saved

- The pension asset: the chosen one, the one with the same CNPJ, or a new one (manual quote),
  with the plan of type **pension fund** (in the IR, treated like PGBL: deductible contributions,
  tax on the full amount at redemption), regime, fees and enrollment from the statement.
- Each contribution becomes a share purchase in the chosen account (without touching cash), by
  the payment date and the share price (retirement ÷ shares), with external id
  `prev:<competência>:<pagamento>:<tipo>:<valor>:<cotas>`: importing again does not duplicate.
- The share value of each contribution and that of the statement become quotes of the asset.
- Risk insurance (disability and death) is a cost, not shares: it becomes an expense (Insurance ›
  Life insurance) only if an account is chosen.
