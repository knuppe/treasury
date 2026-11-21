# Cost method in sales and IRR per sale

The sales ledger (Portfolio › Sales) calculates the cost of each sale by the family's country
rule and shows the annualized IRR of each sale.

## How it works

- **Cost method.** Three methods: average price, FIFO and the UK pool (same
  day, following 30 days and the section 104 pool). The default comes from the family's country
  (`country_profile.cost_method`, migration V152). In Settings › Family, the "Cost method in
  sales" card switches the method or returns to automatic (`portfolio.cost_method` in the
  settings; empty = automatic).
- **Average price continues.** Outside average price, the ledger also shows the "Average
  price" column and the result at that cost, which is the Brazilian IR one. The Taxes screen and the
  IR calculation do not change: they stay on average price.
- **Fixed income and Tesouro.** They already leave by lots (FIFO) and do not change with the method.
- **IRR per sale.** The annual rate that equates the purchases that formed the sale's cost (by the
  date of each one) to the net sale value plus the income received on the units
  sold. It is empty when all purchases are on the same day as the sale or after it.
- **Agent.** `list_sales` returns the method used, the average cost, the result at the average and the
  IRR of each line. The actions `portfolio.cost_method` and `portfolio.set_cost_method` read and
  write the choice.

## Rule by country

Each rule was checked against two official sources on 2026-10-31.

- **Brazil**
  - **Method in the app:** average price
  - **Rule:** weighted average of unit costs, a single one for the taxpayer across all brokers; day trade is calculated separately
  - **Sources:** Receita Federal, "Ganho Líquido" (variable income), updated 2024-06-04; Perguntas e Respostas IRPF 2026, v1.0, questions 719 and 720 (IN RFB 1.585/2015, art. 58)
- **Portugal**
  - **Method in the app:** FIFO
  - **Rule:** the ones sold are those acquired longest ago, per custodian institution
  - **Sources:** CIRS art. 43.º n.º 8 d) and n.º 9 (as worded by Lei 31/2024), Portal das Finanças; binding ruling, case 29146 (2026-05-19)
- **Spain**
  - **Method in the app:** FIFO
  - **Rule:** homogeneous securities: the ones sold are those acquired first (includes fund units)
  - **Sources:** Ley 35/2006 (LIRPF), art. 37.2, BOE, consolidated text of 2026-10-07; Manual práctico Renta 2025, Agencia Tributaria
- **Germany**
  - **Method in the app:** FIFO
  - **Rule:** the first bought are the first sold, per custody account (a sub-account counts as its own account); also applies to funds and ETFs
  - **Sources:** EStG § 20 (4) and InvStG § 19 (1), gesetze-im-internet.de; BMF, "Einzelfragen zur Abgeltungsteuer", 2025-05-14, Rn. 97 to 99
- **US**
  - **Method in the app:** FIFO
  - **Rule:** without specific identification, the sale comes from the oldest lot; average only for fund shares (RIC) and dividend reinvestment
  - **Sources:** IRS Publication 550 (2025); 26 CFR §1.1012-1 (edition of 2025-04-01)
- **United Kingdom**
  - **Method in the app:** UK pool
  - **Rule:** 1) same-day purchases; 2) purchases in the following 30 days; 3) section 104 pool, per person and share class
  - **Sources:** HMRC HS284 (2025), updated 2026-04-06; Capital Gains Manual CG51555 and CG51560
- **Canada**
  - **Method in the app:** average price
  - **Rule:** average cost of all identical properties of the taxpayer, recalculated at each purchase
  - **Sources:** Income Tax Act s. 47; CRA T4037, "Capital Gains – 2025"
- **France**
  - **Method in the app:** average price
  - **Rule:** weighted average acquisition price, mandatory
  - **Sources:** CGI art. 150-0 D, Légifrance; BOFiP BOI-RPPM-PVBMI-20-10-20-40
- **Others**
  - **Method in the app:** average price
  - **Rule:** no built-in rule

Source addresses:

- https://www.gov.br/receitafederal/pt-br/assuntos/meu-imposto-de-renda/pagamento/renda-variavel/bolsa-de-valores-1/ganho-liquido
- https://www.gov.br/receitafederal/pt-br/centrais-de-conteudo/publicacoes/perguntas-e-respostas/dirpf/p-r-irpf-2026-v1-00-2026-04-23.pdf
- https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/codigos_tributarios/cirs_rep/Pages/irs43.aspx
- https://info.portaldasfinancas.gov.pt/pt/informacao_fiscal/informacoes_vinculativas/rendimento/cirs/Documents/PIV_29146.pdf
- https://www.boe.es/buscar/act.php?id=BOE-A-2006-20764
- https://sede.agenciatributaria.gob.es/Sede/ayuda/manuales-videos-folletos/manuales-practicos/irpf-2025/c11-ganancias-perdidas-patrimoniales/determinacion-importe-ganancias-perdidas-patrimon-especificas/transmisiones-onerosas-valores-admitidos-negociacion/general-transmisiones-onerosas-valores-cotizados.html
- https://www.gesetze-im-internet.de/estg/__20.html
- https://www.bundesfinanzministerium.de/Content/DE/Downloads/BMF_Schreiben/Steuerarten/Abgeltungsteuer/2025-05-14-einzelfragen-zur-abgeltungsteuer.pdf
- https://www.irs.gov/publications/p550
- https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol13/xml/CFR-2025-title26-vol13-sec1-1012-1.xml
- https://www.gov.uk/government/publications/shares-and-capital-gains-tax-hs284-self-assessment-helpsheet/hs284-shares-and-capital-gains-tax-2025
- https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51560
- https://laws-lois.justice.gc.ca/eng/acts/I-3.3/section-47.html
- https://www.canada.ca/en/revenue-agency/services/forms-publications/publications/t4037/capital-gains.html
- https://www.legifrance.gouv.fr/codes/article_lc/LEGIARTI000048805702/
- https://bofip.impots.gouv.fr/doctrine/BOI-RPPM-PVBMI-20-10-20-40

What was not read directly at the source: the text of IN RFB 1.585/2015 (the Receita's
rules site only opens with JavaScript; questions 719 and 720 cite art. 58); the BMF letter was
read from a copy of the same PDF, because the ministry's server asked for a robot check; and,
in the United Kingdom, the "earliest first" order among several purchases within the 30 days comes from
TCGA 1992 s. 106A(5), which was not opened.

## How the app applies it

- **Scope.** The app matches sales with the asset's purchases across the whole vault, adding up all
  accounts. This is the rule in Brazil, Canada, the United Kingdom and France. In Germany (per
  custody account) and Portugal (per institution), someone who holds the same asset at more than one broker
  may see a difference, because the app does not yet separate lots by account.
- **Events.** Splits and reverse splits change the quantity of each lot without changing the
  cost; returns of capital reduce the cost of the lots proportionally. A position closed and
  restarted (a ticker change, for example) starts new lots.
- **UK pool.** Purchases matched by the 30-day rule enter with the sale date
  in the average holding period and the IRR, because they came after it. The UK residence
  condition on the purchase date is not checked: the app assumes the family was resident.
- **Out of scope.** Specific lot identification (US), day trade (Brazil) and the
  rules for sales at a loss (wash sale in the US, two months in Spain, superficial loss in
  Canada).
