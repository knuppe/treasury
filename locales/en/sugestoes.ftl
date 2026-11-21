# Companion agent suggestions (core, `ai.suggestions`): the card (title and detail) and the
# ready-made request sent to the agent as the user's message, in the installation language. Tool
# names (categorize_transactions…) stay as they are.
# Revision: AI draft, not yet reviewed by a native speaker.

sugestoes-nomes-mais = { $nomes } and { $n } more

## First setup

sugestoes-cadastro-titulo = Add the family's accounts
sugestoes-cadastro-detalhe = Tell the agent where you have accounts (banks, cards, brokers) and it proposes adding them and the best way to import each one.
sugestoes-cadastro-pedido = I want to record where the family keeps its money. Ask where we have accounts (banks, credit cards, brokers, accounts abroad) and, for each one: propose adding it (create_account with institution_name; cards with closing and due dates), tell me the best way to bring in the data (OFX or CSV from the bank, PDF card bill, B3 Investor Area report, PDF brokerage notes) and where to download each file. Don't make up balances: ask.
sugestoes-cadastro-pedido-exterior = I want to record where the family keeps its money. Ask where we have accounts (banks, credit cards, brokers, retirement accounts, accounts in another country) and, for each one: propose adding it (create_account with institution_name; cards with closing and due dates), tell me the best way to bring in the data (OFX, QFX or CSV from the bank, the broker statement) and where to download each file. Don't make up balances: ask.

## Uncategorized transactions

sugestoes-sem-categoria-titulo = { $n ->
    [one] { $n } uncategorized transaction
   *[other] { $n } uncategorized transactions
}
sugestoes-sem-categoria-detalhe = In the last 90 days. The agent proposes categories and rules for the next ones.
sugestoes-sem-categoria-pedido = There are { $n } uncategorized transactions since { $desde }. Propose categories (categorize_transactions) and, for patterns that repeat, a rule (create_rule).

## Accounts without a recent statement

sugestoes-sem-extrato-titulo = { $n ->
    [one] 1 account without a recent statement
   *[other] { $n } accounts without a recent statement
}
sugestoes-sem-extrato-detalhe = { $contas }: no transactions in more than { $dias } days.
sugestoes-sem-extrato-pedido = The accounts { $contas } have had no transactions in more than { $dias } days. For each one, tell me the best way to bring in the statement (OFX or CSV from the bank, PDF card bill) and where to download the file on the bank's website or app.

## Card bill due

sugestoes-fatura-titulo = { $cartao } bill due on { $data }
sugestoes-fatura-detalhe = { $valor } to pay.
sugestoes-fatura-detalhe-mais = { $valor } to pay (+{ $n } bill(s)).
sugestoes-fatura-pedido = The { $cartao } card bill is due on { $data }. Check the total with list_card_statements, tell me which account the payment comes from and whether its balance covers it.

## Budget exceeded

sugestoes-orcamento-titulo = { $n ->
    [one] Budget exceeded in { $n } category
   *[other] Budget exceeded in { $n } categories
}
sugestoes-orcamento-detalhe = { $categorias }: { $valor } over plan.
sugestoes-orcamento-pedido = The { $mes } budget was exceeded in { $categorias }. Show what drove the spending (spending_summary) and suggest a realistic adjustment for the rest of the month.

## Assets without a quote

sugestoes-cotacao-titulo = { $n ->
    [one] { $n } asset without a recent quote
   *[other] { $n } assets without a recent quote
}
sugestoes-cotacao-detalhe = { $ativos }: the portfolio uses the cost or the last quote.
sugestoes-cotacao-pedido = The assets { $ativos } have had no quote for more than 15 days. Check how they are set up (list_assets) and propose the right price source (update_asset with price_provider and provider_symbol).

## B3 import

sugestoes-b3-titulo = Import trades from B3
sugestoes-b3-detalhe = { $corretoras }: no trades in more than { $dias } days. The B3 Investor Area report brings trades and dividends.
sugestoes-b3-pedido = The brokers { $corretoras } have no recent trades in Treasury. Explain how to download the trading and dividends reports from the B3 Investor Area (or the PDF brokerage notes) and, if I want, ask to open the website.

## Proposals and alerts

sugestoes-propostas-titulo = { $n ->
    [one] { $n } proposal awaiting approval
   *[other] { $n } proposals awaiting approval
}
sugestoes-propostas-detalhe = Nothing changes until you approve; each one shows the exact diff.
sugestoes-propostas-pedido = Summarize the pending proposals (list_pending_proposals) and tell me which ones look safe to approve.
sugestoes-alertas-titulo = { $n ->
    [one] { $n } new alert
   *[other] { $n } new alerts
}
sugestoes-alertas-detalhe = From the manager routines (budget, due dates, portfolio).
sugestoes-alertas-pedido = Read the new alerts (list_alerts) and tell me what to do first.
sugestoes-reserva-titulo = { $n ->
    [one] One daily-liquidity investment outside the goals
   *[other] { $n } daily-liquidity investments outside the goals
}
sugestoes-reserva-detalhe = { $ativos }: linked to the emergency fund goal, they count in the reserve and in the Security of the financial health.
sugestoes-reserva-pedido = The investments { $ativos } have daily liquidity and are not linked to any goal. Look at the emergency fund goal (list_goals) and help me decide whether they belong in it.
