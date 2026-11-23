# Alerts in the Agent inbox (core): reminders (bills and card statements, record dates, watchlist,
# negative balance, subscriptions, budget), the manager routines (alerts, monthly close, review),
# data sources turned off and the asset health score, plus system notifications. Alerts store the
# key and the arguments (V128) and the text comes out in the reader's language
# (docs/i18n-design.md). Tool names in parentheses (get_budget_status…) stay as they are.
# Revision: AI draft, not yet reviewed by a native speaker.

## Formats

alertas-mes-ano = { $mes } { $ano }
alertas-mes-ano-curto = { $mes } { $ano }
alertas-item-valor = { $nome } { $valor }

## Alert source

alertas-origem-lembretes = reminders
alertas-origem-rotina-alertas = routine: alerts
alertas-origem-fechamento = routine: monthly close
alertas-origem-fechamento-motor = routine: monthly close ({ $motor })
alertas-origem-score = asset score
alertas-origem-fontes = data sources

## System notifications

alertas-notificacao-detalhes = Open Treasury to see the details.

## When

alertas-quando = { $dias ->
    [0] today
    [1] tomorrow
   *[other] in { $dias } days
}
alertas-quando-data = { $dias ->
    [0] today
    [1] tomorrow
   *[other] in { $dias } days ({ $data })
}

## Bills and card statements

alertas-vence-fatura = { $conta } card bill is due { $quando }
alertas-vence-conta = { $descricao } is due { $quando }
alertas-conta-sem-descricao = Bill
alertas-vence-corpo = { $valor } from { $conta }, due { $data }.
alertas-vence-corpo-painel = { $valor } from { $conta } (upcoming due dates on the dashboard).

## Dividend record dates

alertas-datacom-titulo = { $ativo }: record date { $quando } ({ $tipo ->
    [jcp] JCP
    [rendimento] FII income
   *[provento] dividend
})
alertas-datacom-corpo = Anyone holding { $ativo } at the end of { $data } receives { $moeda } { $valor } per share{ $pagamento }{ $liquido }.
alertas-datacom-pagamento = , paid on { $data }
alertas-datacom-liquido = { " " }(about { $valor } net for you)

## Watchlist

alertas-cotacao-acima = { $ativo }: price above { $limite }
alertas-cotacao-abaixo = { $ativo }: price below { $limite }
alertas-cotacao-ambos = { $ativo }: price above { $acima } and price below { $abaixo }
alertas-cotacao-gatilhos = { $ativo }: { $gatilhos }
alertas-cotacao-corpo = { $ativo } is trading at { $moeda } { $preco } (watchlist).
alertas-cotacao-normal = The price is back within the alerts.

## Negative balance in the forecast

alertas-saldo-negativo-titulo = { $conta }: forecast balance goes negative
alertas-saldo-negativo-corpo = According to the balance forecast, { $conta } goes negative from { $data } and reaches { $minimo } on { $data_minimo } (the lowest point in the next 30 days).
alertas-saldo-negativo-fim = As of the { $data } forecast, the balance no longer goes negative.

## Subscriptions

alertas-assinatura-aumento-titulo = { $nome }: price went up { $pct }
alertas-assinatura-aumento-corpo = From { $antes } to { $agora } in the charge on { $data } ({ $conta }).
alertas-assinatura-nova-titulo = New recurring charge: { $nome }
alertas-assinatura-nova-corpo = { $valor } per { $periodo ->
    [ano] year
   *[mes] month
} since { $data } on { $conta }. If you don't recognize it, check it.

## Budget

alertas-orcamento-estourado-titulo = { $categoria }: budget exceeded
alertas-orcamento-estourado-corpo = { $gasto } spent against { $previsto } planned for the month ({ $pct } of plan).
alertas-orcamento-estourado-corpo-mes = { $gasto } spent against { $previsto } planned in { $mes } ({ $pct } of plan).
alertas-orcamento-ritmo-titulo = { $categoria }: on pace to go over
alertas-orcamento-ritmo-corpo = { $gasto } of { $previsto } already spent ({ $pct }) with { $decorrido } of the month gone.

## Portfolio

alertas-rebalancear-titulo = Portfolio out of balance: { $classe }
alertas-rebalancear-corpo = { $atual } of the portfolio against a target of { $alvo }. An analysis of your own targets, not investment advice.

## Data sources

alertas-fonte-desligada-titulo = { $fonte } turned off after failures
alertas-fonte-desligada-corpo = Fetching from { $fonte } failed { $dias } days in a row, so the source was turned off to avoid saving wrong data. Last error: { $erro }. Once the problem is fixed, turn it back on in Settings › Data sources.
alertas-erro-desconhecido = unknown

## Asset health score

alertas-score-titulo = { $ativo }: health score { $nota } ({ $faixa })
alertas-score-titulo-sem-nota = { $ativo }: health score { $faixa }
alertas-score-caiu = Score fell { $pontos } points since { $desde }
alertas-score-faixa-piorou = Band worsened from { $antes } to { $agora }
alertas-score-red-flag = New red flag: { $flag }

## Manager routines

alertas-rotina-revisao = Ongoing review
alertas-rotina-fechamento = Monthly close
alertas-rotina-alertas = Alerts
alertas-rotina-sem-pendentes = No uncategorized transactions.
alertas-rotina-revisao-feita = { $sugeridos } of { $total } uncategorized transactions got a suggestion; { $propostas } proposals in the queue.
alertas-rotina-motor-categorizacao-fora = categorization engine unavailable: { $erro }
alertas-rotina-motor-fora = engine unavailable, report without AI: { $erro }
alertas-rotina-fechamento-gravado = { $mes } close saved to the alerts inbox.
alertas-rotina-alertas-novos = { $n ->
    [one] { $n } new alert.
   *[other] { $n } new alerts.
}

## Monthly close (without an AI engine)

alertas-fechamento-titulo = { $mes } close
alertas-fechamento-resumo = **{ $mes } summary**
alertas-fechamento-fluxo = - Income: { $receitas } · Expenses: { $despesas } · Net: { $saldo }{ $poupanca }
alertas-fechamento-poupanca = { " " }· Savings rate: { $pct }
alertas-fechamento-patrimonio = - Net worth: { $valor } ({ $sinal }{ $variacao } this month)
alertas-fechamento-maiores = - Largest expenses: { $lista }
alertas-fechamento-recomendacoes = **Recommendations**
alertas-fechamento-aviso = _Analysis based on your data, not professional financial advice._
alertas-rec-orcamento = { $categoria } went over budget: { $gasto } spent against { $previsto } planned. Review the category's transactions or adjust next month's plan.
alertas-rec-poupanca = The savings rate was { $pct }. Set a monthly savings goal and keep an eye on the largest spending categories.
alertas-rec-despesas-subiram = Expenses went up from { $antes } to { $agora } ({ $pct } more than the previous month). Check which expenses were one-offs.
alertas-rec-sem-categoria = { $n } transactions this month have no category: categorize them so the reports are right; the ongoing review already suggests categories.
alertas-rec-meta = The goal "{ $meta }" is off pace: monthly contribution needed of { $valor }.
alertas-rec-carteira = Portfolio off its target allocation in { $classe }: { $atual } of the portfolio against a target of { $alvo }. Consider directing your next contributions there.
alertas-rec-maiores = The month's largest expenses were { $lista }: worth checking whether any can be reduced.
alertas-rec-sem-orcamento = There is no budget for the month: set planned amounts for the main categories to follow your spending pace.
alertas-rec-sem-metas = There are no goals yet: start with an emergency fund of a few months of expenses.
alertas-rec-em-dia = Keep transactions and imports up to date so the next close is more accurate.

## Tipos de alerta (rodapé do alerta)

alertas-tipo-budget-over = Budget exceeded
alertas-tipo-budget-pace = Budget pace
alertas-tipo-due-soon = Due date
alertas-tipo-rebalance = Rebalancing
alertas-tipo-monthly-close = Monthly close
alertas-tipo-negative-balance = Forecast negative balance
alertas-tipo-asset-score = Health score
alertas-tipo-source-disabled = Data source turned off
alertas-tipo-asset-data-gaps = Asset data
alertas-tipo-stale-accounts = Accounts without a statement
alertas-tipo-iptu-due = IPTU due
alertas-tipo-iptu-missing = IPTU bill missing
alertas-tipo-dividend-datacom = Dividend record date
alertas-tipo-price-target = Price target
alertas-tipo-subscription-increase = Subscription price increase
alertas-tipo-subscription-new = New subscription

## Asset data (enrichment)

alertas-dados-ativos-titulo =
    { $n ->
        [one] { $n } piece of your asset data to fill in
       *[other] { $n } pieces of your asset data to fill in
    }
alertas-dados-ativos-corpo = The official sources Treasury has already downloaded have suggestions for **{ $campos } field(s)** of { $ativos } asset(s), such as sector, segment, ISIN and CNPJ (e.g. { $exemplos }).
alertas-dados-ativos-onde = Check each one's source and confidence and save what you want in Portfolio › Asset health › Asset data. Nothing changes until you choose.
alertas-reduflacao-titulo = { $produto }: smaller pack, higher price
alertas-reduflacao-corpo = { $produto } went from { $antes } to { $agora } { $unidade } and the price per { $unidade } changed { $variacao }. That's shrinkflation: less product for the same price (or more).
alertas-tela-entrou-titulo = { $ticker } now meets the screen { $tela }
alertas-tela-entrou-corpo = { $ticker } now meets the filters of your screen { $tela }. It is a list of what meets criteria, not investment advice.
alertas-tela-saiu-titulo = { $ticker } no longer meets the screen { $tela }
alertas-tela-saiu-corpo = { $ticker }, which is in your portfolio, no longer meets the filters of the screen { $tela }.
alertas-cartao-limite-titulo = { $cartao }: { $pct } of the limit used
alertas-cartao-limite-corpo = { $usado } used out of { $limite } ({ $disponivel } left), counting future installments.
alertas-cartao-anuidade-renova-titulo = { $cartao }: annual fee renews in { $mes }
alertas-cartao-anuidade-renova-corpo = The { $valor } annual fee renews soon; the last 12 months cost { $pago }. It's a good time to ask the bank for a waiver or discount.
alertas-cartao-anuidade-confira-titulo = { $cartao }: check the annual fee
alertas-cartao-anuidade-isenta-corpo = A { $valor } charge on { $data } on a bill that had reached the waiver.
alertas-cartao-anuidade-parcela-extra-corpo = A { $valor } charge on { $data } goes over the agreed number of installments in 12 months.
alertas-cartao-anuidade-valor-corpo = A { $valor } charge on { $data }; the agreed installment is { $esperado }.
alertas-cartao-rotativo-titulo = { $cartao }: the { $mes } bill went into revolving credit
alertas-cartao-rotativo-corpo-br = { $valor } was left unpaid on the due date ({ $data }). Revolving credit has the highest interest on the market and can only last until the next bill; after that the bank must offer an installment plan. If you can, pay off the balance or compare the plan with a cheaper loan.
alertas-cartao-rotativo-corpo = { $valor } was left unpaid on the due date ({ $data }). A balance left on the card usually carries high interest; if you can, pay it off or compare with a cheaper loan.
alertas-cartao-rotativo-teto-titulo = { $cartao }: charges above the debt amount
alertas-cartao-rotativo-teto-corpo = The interest and charges since the revolving credit started ({ $encargos }) are already above the unpaid amount ({ $valor }). In Brazil, Law 14,690/2023 caps these charges at 100% of the original debt; check the bill with the bank.
alertas-cartao-cashback-titulo = { $cartao }: cashback for the { $mes } bill didn't arrive
alertas-cartao-cashback-corpo = Expected { $previsto } and received { $recebido } by { $data }. Check the bank's app or ask for the credit.
alertas-cartao-pontos-titulo = { $cartao }: points for the { $mes } bill didn't arrive
alertas-cartao-pontos-corpo = { $previsto } points expected and { $recebido } credited by { $data }. Check the program or ask for the credit.
alertas-pontos-vencendo-titulo = { $programa }: points expire { $quando }
alertas-pontos-vencendo-corpo = { $pontos } points of { $membro } expire on { $data }. Use, transfer or renew them before then.

## Accounts without a recent statement

alertas-contas-desatualizadas-titulo = { $n ->
    [one] 1 account without a recent statement. Import it?
   *[other] { $n } accounts without a recent statement. Import them?
}
alertas-contas-desatualizadas-corpo = The balance may be incomplete: { $contas }. Import the statement or check the balance so financial health and the emergency fund use the right amount.
alertas-contas-desatualizadas-item = { $conta } (last transaction on { $data })
alertas-contas-desatualizadas-item-sem = { $conta } (no transactions)
