Mocks per core module (`accounts.ts`, `transactions.ts`, `reports.ts`…).

Each file registers its commands with `registerMockHandlers` (from `../mockRegistry`)
and, if it has state, an `onMockReset`. `mock.ts` imports every `*.ts` here
automatically (`import.meta.glob`). Use the generated types from `../generated/` so
that the mock and the Rust core always have the same shapes.
