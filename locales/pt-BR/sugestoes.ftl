# Sugestões do agente companheiro (núcleo, `ai.suggestions`): o cartão (título e detalhe) e o
# pedido pronto que vai ao agente como mensagem do usuário, no idioma da instalação. Os nomes
# das ferramentas (categorize_transactions…) ficam como estão. pt-BR é o idioma de origem.

sugestoes-nomes-mais = { $nomes } e mais { $n }

## Primeiro cadastro

sugestoes-cadastro-titulo = Cadastrar as contas da família
sugestoes-cadastro-detalhe = Comece pela conta que você mais usa (banco, cartão ou corretora). Se tiver um assistente de IA configurado, ele pode propor o cadastro das outras e dizer como trazer o extrato de cada uma.
sugestoes-cadastro-pedido = Quero cadastrar onde a família tem dinheiro. Pergunte onde temos contas (bancos, cartões de crédito, corretoras, contas no exterior) e, para cada uma: proponha o cadastro (create_account com institution_name; cartões com fechamento e vencimento), diga o melhor caminho para trazer os dados (OFX ou CSV do banco, fatura em PDF, relatório da Área do Investidor B3, notas de corretagem em PDF) e onde baixar cada arquivo. Não invente saldos: pergunte.
sugestoes-cadastro-pedido-exterior = Quero cadastrar onde a família tem dinheiro. Pergunte onde temos contas (bancos, cartões de crédito, corretoras, contas de aposentadoria, contas em outro país) e, para cada uma: proponha o cadastro (create_account com institution_name; cartões com fechamento e vencimento), diga o melhor caminho para trazer os dados (OFX, QFX ou CSV do banco, o extrato da corretora) e onde baixar cada arquivo. Não invente saldos: pergunte.

## Lançamentos sem categoria

sugestoes-sem-categoria-titulo = { $n ->
    [1] { $n } lançamento sem categoria
   *[other] { $n } lançamentos sem categoria
}
sugestoes-sem-categoria-detalhe = Nos últimos 90 dias. O agente propõe as categorias e regras para os próximos.
sugestoes-sem-categoria-pedido = Há { $n } lançamentos sem categoria desde { $desde }. Proponha as categorias (categorize_transactions) e, para os padrões que se repetem, uma regra (create_rule).

## Contas sem extrato recente

sugestoes-sem-extrato-titulo = { $n ->
    [1] 1 conta sem extrato recente
   *[other] { $n } contas sem extrato recente
}
sugestoes-sem-extrato-detalhe = { $contas }: sem lançamentos há mais de { $dias } dias.
sugestoes-sem-extrato-pedido = As contas { $contas } estão sem lançamentos há mais de { $dias } dias. Para cada uma, diga o melhor jeito de trazer o extrato (OFX ou CSV do banco, fatura em PDF) e onde baixar o arquivo no site ou no app do banco.

## Fatura vencendo

sugestoes-fatura-titulo = Fatura do { $cartao } vence em { $data }
sugestoes-fatura-detalhe = { $valor } a pagar.
sugestoes-fatura-detalhe-mais =
    { $n ->
        [one] { $valor } a pagar (+{ $n } fatura).
       *[other] { $valor } a pagar (+{ $n } faturas).
    }
sugestoes-fatura-pedido = A fatura do cartão { $cartao } vence em { $data }. Confira o total com list_card_statements, diga de qual conta sai o pagamento e se o saldo dela cobre.

## Orçamento estourado

sugestoes-orcamento-titulo = { $n ->
    [1] Orçamento estourado em { $n } categoria
   *[other] Orçamento estourado em { $n } categorias
}
sugestoes-orcamento-detalhe = { $categorias }: { $valor } acima do previsto.
sugestoes-orcamento-pedido = O orçamento de { $mes } estourou em { $categorias }. Mostre o que puxou o gasto (spending_summary) e sugira um ajuste realista para o resto do mês.

## Ativos sem cotação

sugestoes-cotacao-titulo = { $n ->
    [1] { $n } ativo sem cotação recente
   *[other] { $n } ativos sem cotação recente
}
sugestoes-cotacao-detalhe = { $ativos }: a carteira usa o custo ou a última cotação.
sugestoes-cotacao-pedido = Os ativos { $ativos } estão sem cotação há mais de 15 dias. Confira o cadastro (list_assets) e proponha a fonte de cotação certa (update_asset com price_provider e provider_symbol).

## Importação da B3

sugestoes-b3-titulo = Importar operações da B3
sugestoes-b3-detalhe = { $corretoras }: sem operações há mais de { $dias } dias. O relatório da Área do Investidor traz negociações e proventos.
sugestoes-b3-pedido = As corretoras { $corretoras } estão sem operações recentes no Treasury. Explique como baixar o relatório de negociação e de proventos na Área do Investidor da B3 (ou as notas de corretagem em PDF) e, se eu quiser, peça para abrir o site.

## Propostas e alertas

sugestoes-propostas-titulo = { $n ->
    [1] { $n } proposta aguardando aprovação
   *[other] { $n } propostas aguardando aprovação
}
sugestoes-propostas-detalhe = Nada muda até você aprovar; cada uma mostra o diff exato.
sugestoes-propostas-pedido = Resuma as propostas pendentes (list_pending_proposals) e diga quais parecem seguras para aprovar.
sugestoes-alertas-titulo = { $n ->
    [1] { $n } alerta novo
   *[other] { $n } alertas novos
}
sugestoes-alertas-detalhe = Das rotinas do gestor (orçamento, vencimentos, carteira).
sugestoes-alertas-pedido = Leia os alertas novos (list_alerts) e diga o que fazer primeiro.
sugestoes-reserva-titulo = { $n ->
    [1] Uma aplicação de liquidez diária fora das metas
   *[other] { $n } aplicações de liquidez diária fora das metas
}
sugestoes-reserva-detalhe = { $ativos }: ligadas à meta de reserva de emergência, entram na reserva e na Segurança da saúde financeira.
sugestoes-reserva-pedido = As aplicações { $ativos } têm liquidez diária e não estão ligadas a nenhuma meta. Veja a meta de reserva de emergência (list_goals) e me ajude a decidir se elas entram nela.
