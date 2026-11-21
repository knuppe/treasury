# Outside Brazil: what goes away and what comes in

Someone who lives outside Brazil does not see what only serves Brazil and gains what serves
their country. The adaptation levels and the rules of each country are in
[countries.md](countries.md); this document lists what changes across the whole app when the
country is not Brazil.

## What goes away outside Brazil

B3 and brokerage notes, IR and DARF, Banco Central interest rates and portability, FIPE, FGTS,
IPTU, NFC-e, Fundamentus, the IR report and INSS in the planner. In addition:

| Where | How it ends up |
|---|---|
| Agent and MCP | `ToolSpec::brazil_only` and `Access::outside_brazil` (`mcp/tools.rs`) remove the Brazilian tools from the list and from calls; the script and the registration suggestion speak of the country; the suggestion to import from B3 only appears in Brazil |
| Categories | No "FGTS withdrawal"; 13th salary and vacation pay only in Brazil, in Portugal and (as extra payments) in Spain. Vaults that already exist do not change |
| Taxes | The contribution column uses the country's name; no income statement and no IPVA warning |
| Assets and portfolio | No Rankings, Screener and Analysis data tabs (B3, CVM, FII); no Ibovespa in the portfolio chart |
| Accounts and debts | Overdraft without IOF, CET and Brazilian indices; debts without Brazilian post-fixed and adjustment and without payroll loans (consignado) |
| Glossary and profile | No Brazilian terms and questions |
| Institutions | The institution field is free text (the Banco Central list is only consulted in Brazil) |

## What comes in

- **CSV import**: `csv_file::guess_mapping` guesses columns, date format (DD/MM/YYYY,
  MM/DD/YYYY, YYYY-MM-DD) and decimal separator from the sample and the country.
- **Norma 43** (`import/norma43.rs`), the statement that every Spanish bank exports, follows the
  OFX path.
- **Retirement accounts** (account type `retirement`, V137) with the country's plan
  (`retirement::PLANS`) and the year's limit: 401(k) and Roth 401(k), IRA and Roth IRA, HSA and
  529 in the US; PPR in Portugal; individual and employer planes de pensiones in Spain; ISA,
  Lifetime ISA and pension in the United Kingdom; RRSP, TFSA and FHSA in Canada; Riester and
  Rürup in Germany; PER and PEA in France; APV in Chile. The limit is per person: accounts in the
  same group with the same holders add up together, and every inflow in the year counts as a
  contribution. A contribution to these accounts is a contribution (as at the broker), not
  spending.
- **Demo**: stores, brokers and apps follow the country (`demo/texts.rs`).
- **E-mail**: the server's automatic configuration works in every country.

## Limitations

- Euribor and SOFR do not exist as a debt index.
- The INE's IPC and the BLS's CPI without a key are not collected; the inflation from the index
  catalog applies.
- Local exchanges (S&P 500, PSI, IBEX 35) do not enter as the portfolio reference in place of the
  Ibovespa.
- Public retirement is only calculated in Brazil, the US and Portugal
  ([investments.md](investments.md#retirement)).
- The indices of physical assets outside Brazil are still IPCA, IGP-M and INCC.
