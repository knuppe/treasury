# What Treasury notices without AI (core, `insights`): personal inflation, fees and interest,
# overdrawn account, unusual spending, small purchases, spending above income and how income is
# split (docs/insights.md). Built on the fly, in the installation language.
# Revision: AI draft, not yet reviewed by a native speaker.

## IPCA groups

insights-grupo-ipca-g1 = Food and beverages
insights-grupo-ipca-g2 = Housing
insights-grupo-ipca-g3 = Household articles
insights-grupo-ipca-g4 = Apparel
insights-grupo-ipca-g5 = Transportation
insights-grupo-ipca-g6 = Health and personal care
insights-grupo-ipca-g7 = Personal expenses
insights-grupo-ipca-g8 = Education
insights-grupo-ipca-g9 = Communication

## Personal inflation

insights-inflacao-titulo = Your inflation over 12 months: { $pessoal }
insights-inflacao-titulo-ipca = Your inflation over 12 months: { $pessoal } (IPCA: { $ipca })
insights-inflacao-corpo = Weighting each group by your own spending (not by the IBGE average basket), the prices of what you buy went up { $pessoal } from { $de } to { $ate }.
insights-inflacao-mais-pesou = What weighed the most: { $grupo } ({ $peso } of spending, up { $variacao }).
insights-inflacao-acao = Income and budget increases below this inflation mean losing purchasing power.

## Fees and interest

insights-tarifas-titulo = { $valor } in fees and interest over 12 months
insights-tarifas-corpo = From { $de } to { $ate }, { $tarifas } went to bank fees ({ $n_tarifas } charges) and { $juros } to interest, IOF and charges ({ $n_juros } charges). That is money that buys nothing.
insights-tarifas-juros-caros = Overdraft and revolving credit card interest are among the most expensive on the market.
insights-tarifas-acao-pacote = Good to know: banks must offer a package of essential services with no fees (CMN Resolution 3.919), and there are free digital accounts.
insights-tarifas-acao-credito = Good to know: swap the overdraft or revolving credit for cheaper credit and pay the card bill in full.
insights-tarifas-acao-ambos = Good to know: banks must offer a package of essential services with no fees (CMN Resolution 3.919), and there are free digital accounts; swap the overdraft or revolving credit for cheaper credit and pay the card bill in full.

## Overdrawn account

insights-vermelho-titulo = { $conta } was overdrawn for { $dias } days
insights-vermelho-corpo = In the last 90 days, the balance of { $conta } was negative on { $dias } days, reaching { $minimo }.
insights-vermelho-taxa = Overdraft costs on average { $taxa }% a month at banks (Banco Central).
insights-vermelho-acao = Check the balance forecast and, if it keeps happening, swap the overdraft for cheaper credit (Debts › Market interest rates).
insights-fgts-titulo = The FGTS for { $mes } was not deposited ({ $conta })
insights-fgts-corpo = { $n ->
    [one] The Caixa statement shows no deposit for { $mes }, and the employer's deadline (the 20th of the next month) has passed.
   *[other] The Caixa statement shows { $n } missing deposits, starting in { $mes }, and the employer's deadline (the 20th of the next month) has passed.
}
insights-fgts-acao = Check the payslip and talk to the company's HR; if that does not solve it, Caixa and the Ministry of Labor take complaints.
insights-previdencia-titulo = { $plano }: contributions to check
insights-previdencia-pendencias = { $n } item(s) to check in the imported payment history:
insights-previdencia-sem-cota = { $aportes } contribution(s) have been waiting for the CVM unit price since { $desde } and count in the balance at the amount paid.
insights-previdencia-acao = Open the plan in Assets, check the items and send the agent the missing payment (or the screenshot showing that part).

## Unusual spending

insights-acima-titulo = { $categoria } above normal in { $mes }
insights-acima-corpo = In { $mes }, { $categoria } added up to { $valor }, { $diferenca } more than the usual of the previous months ({ $normal } a month, the 6-month median).
insights-acima-acao = Open the category's transactions to see what changed; if the expense will continue, adjust the budget.
insights-abaixo-titulo = { $categoria } below normal in { $mes }
insights-abaixo-corpo = In { $mes }, { $categoria } added up to { $valor }, { $diferenca } less than usual ({ $normal } a month).
insights-abaixo-acao = If the savings continue, you can set the difference aside in a goal.

## Small purchases

insights-pequenas-titulo = { $n } small purchases added up to { $total } in { $mes }
insights-pequenas-corpo = Purchases of up to { $limite } each, which seem small one by one. The ones that weighed the most: { $lista }.
insights-pequenas-item = { $categoria } ({ $valor })
insights-pequenas-acao = A monthly cap for these categories in the budget helps you see the total.
insights-sem-categoria = uncategorized

## Spending above income

insights-deficit-titulo = Spending above income by { $valor } in the last 3 months
insights-deficit-corpo = From { $de } to { $ate }, { $renda } came in and { $despesas } went out in expenses. The difference came from savings, overdraft, credit card or loans.
insights-deficit-acao = Check the largest categories in the budget and the balance forecast for the coming months.

## Income split (50/30/20)

insights-divisao-titulo = Of your income: { $essencial } on essentials, { $resto } on the rest, { $sobra } left over
insights-divisao-corpo = From { $de } to { $ate }, { $renda } came in: { $essencial } went to essentials (housing, groceries, health, transportation… the categories marked as essential), { $resto } to the rest and { $sobra } was left over. A common reference is 50% for essentials, 30% for the rest and 20% for saving.
insights-divisao-acao-folga = Good margin: putting what is left over into a goal (emergency fund first) makes it grow.
insights-divisao-acao-essencial = Essentials weigh a lot: renegotiating fixed bills (rent, health plan, internet, phone) usually does more than cutting small expenses.
insights-divisao-acao-teto = A cap in the budget for non-essentials helps you have more left at the end of the month.

insights-kit-titulo = Create the vault's recovery kit
insights-kit-corpo = If the master password is forgotten, nobody can open the vault, not even Treasury. The recovery kit is a code that opens the vault without the password: print it and keep it on paper, away from the computer. If you made one before, make it again so the app records it (the code is the same).
insights-kit-velho-titulo = The old recovery kit no longer opens the vault
insights-kit-velho-corpo = The master password changed after the kit was made, so the printed code is no longer valid. Make a new kit and replace the paper you kept.
insights-kit-acao = Open Settings › Security and create the kit.
