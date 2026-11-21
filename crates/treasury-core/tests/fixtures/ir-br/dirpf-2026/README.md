# Reference cases for the annual adjustment return (IRPF 2026, calendar year 2025)

The 65 `*.json` files in this folder were copied unchanged from the project
[MCP Imposto de Renda](https://github.com/ALASTecnology/mcp-imposto-de-renda), folder
`packages/core/test/golden/2026/`, at commit
[`6c37d2d5`](https://github.com/ALASTecnology/mcp-imposto-de-renda/tree/6c37d2d5f5a65e5b7aed205eb99fbfd81a50afde/packages/core/test/golden/2026)
(copied on 2026-10-20).

- **License:** GNU AGPL-3.0-only, the same as Treasury (see the `LICENSE` and ADR 0003 of the
  source project). Rights belong to the authors of MCP Imposto de Renda.
- **What they are:** each case has `entrada` (input: fictional data, with invented CPF, names and
  companies) and `esperado` (expected: taxable income, deductions, base, tax due under the complete and
  simplified models, withheld tax, balance to pay or refund, most advantageous model, effective
  rate), in cents. According to the source project, the expected values were generated with the
  official Receita program (PGD IRPF 2026). Some carry `esperadoGcap` (capital gains) and
  `esperadoLivroCaixaRural`.
- **What they are for here:** Treasury does not yet compute the annual adjustment (complete ×
  simplified), so the cases are kept for when it does. For now,
  `src/tax/tests_dirpf_golden.rs` only checks that all of them load and that the numbers are
  consistent with each other (base = income − deductions, simplified discount = 20% capped at
  R$ 16,754.34, most advantageous model = lowest tax due).
- The "exterior" cases are work income received from abroad (carnê-leão), not the
  foreign financial investments of Lei 14.754/2023; they do not serve to check the
  "exterior" summary of the Taxes screen.

Do not edit the files: to update, copy again from the source project and change the commit
cited above.
