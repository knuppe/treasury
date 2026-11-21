# Alertas da caixa do Agente (núcleo): lembretes (contas e faturas, data-com, watchlist, saldo
# negativo, assinaturas, orçamento), rotinas do gestor (alertas, fechamento mensal, revisão),
# fontes de dados desligadas e score de saúde, mais as notificações do sistema. Os alertas
# guardam a chave e os argumentos (V128) e o texto sai no idioma de quem lê
# (docs/i18n-design.md). Os nomes das ferramentas entre parênteses (get_budget_status…) ficam
# como estão. pt-BR é o idioma de origem.

## Formatos

alertas-mes-ano = { $mes } de { $ano }
alertas-mes-ano-curto = { $mes }/{ $ano }
alertas-item-valor = { $nome } { $valor }

## Origem do alerta

alertas-origem-lembretes = lembretes
alertas-origem-rotina-alertas = rotina: alertas
alertas-origem-fechamento = rotina: fechamento mensal
alertas-origem-fechamento-motor = rotina: fechamento mensal ({ $motor })
alertas-origem-score = score
alertas-origem-fontes = sources

## Notificações do sistema

alertas-notificacao-detalhes = Abra o Treasury para ver os detalhes.

## Quando

alertas-quando = { $dias ->
    [0] hoje
    [1] amanhã
   *[other] em { $dias } dias
}
alertas-quando-data = { $dias ->
    [0] hoje
    [1] amanhã
   *[other] em { $dias } dias ({ $data })
}

## Contas e faturas a pagar

alertas-vence-fatura = Fatura { $conta } vence { $quando }
alertas-vence-conta = { $descricao } vence { $quando }
alertas-conta-sem-descricao = Conta
alertas-vence-corpo = { $valor } em { $conta }, vencimento { $data }.
alertas-vence-corpo-painel = { $valor } em { $conta } (próximos vencimentos do painel).

## Data-com de proventos

alertas-datacom-titulo = { $ativo }: data-com { $quando } ({ $tipo ->
    [jcp] JCP
    [rendimento] rendimento
   *[provento] provento
})
alertas-datacom-corpo = Quem tiver { $ativo } no fim de { $data } recebe { $moeda } { $valor } por cota{ $pagamento }{ $liquido }.
alertas-datacom-pagamento = , pago em { $data }
alertas-datacom-liquido = { " " }(cerca de { $valor } líquidos para você)

## Watchlist

alertas-cotacao-acima = { $ativo }: cotação acima de { $limite }
alertas-cotacao-abaixo = { $ativo }: cotação abaixo de { $limite }
alertas-cotacao-ambos = { $ativo }: cotação acima de { $acima } e cotação abaixo de { $abaixo }
alertas-cotacao-gatilhos = { $ativo }: { $gatilhos }
alertas-cotacao-corpo = { $ativo } cotado a { $moeda } { $preco } (watchlist).
alertas-cotacao-normal = A cotação voltou para dentro dos alertas.

## Saldo negativo na previsão

alertas-saldo-negativo-titulo = { $conta }: saldo previsto fica negativo
alertas-saldo-negativo-corpo = Pela previsão de saldo, { $conta } fica negativa a partir de { $data } e chega a { $minimo } em { $data_minimo } (o ponto mais baixo dos próximos 30 dias).
alertas-saldo-negativo-fim = Pela previsão de { $data }, o saldo não fica mais negativo.

## Assinaturas

alertas-assinatura-aumento-titulo = { $nome }: preço subiu { $pct }
alertas-assinatura-aumento-corpo = De { $antes } para { $agora } na cobrança de { $data } ({ $conta }).
alertas-assinatura-nova-titulo = Cobrança recorrente nova: { $nome }
alertas-assinatura-nova-corpo = { $valor } por { $periodo ->
    [ano] ano
   *[mes] mês
} desde { $data } em { $conta }. Se não reconhece, confira.

## Orçamento

alertas-orcamento-estourado-titulo = { $categoria }: orçamento estourado
alertas-orcamento-estourado-corpo = { $gasto } gastos para { $previsto } previstos no mês ({ $pct } do previsto).
alertas-orcamento-estourado-corpo-mes = { $gasto } gastos para { $previsto } previstos em { $mes } ({ $pct } do previsto).
alertas-orcamento-ritmo-titulo = { $categoria }: no ritmo de estourar
alertas-orcamento-ritmo-corpo = Já foram { $gasto } de { $previsto } ({ $pct }) com { $decorrido } do mês decorrido.

## Carteira

alertas-rebalancear-titulo = Carteira desbalanceada: { $classe }
alertas-rebalancear-corpo = { $atual } da carteira para um alvo de { $alvo }. Análise dos seus alvos, não recomendação de investimento.

## Fontes de dados

alertas-fonte-desligada-titulo = { $fonte } desligada por falhas
alertas-fonte-desligada-corpo = A coleta de { $fonte } falhou { $dias } dias seguidos e a fonte foi desligada para não gravar dados errados. Último erro: { $erro }. Quando o problema for resolvido, ligue de novo em Ajustes › Fontes de dados.
alertas-erro-desconhecido = desconhecido

## Score de saúde dos ativos

alertas-score-titulo = { $ativo }: score de saúde { $nota } ({ $faixa })
alertas-score-titulo-sem-nota = { $ativo }: score de saúde { $faixa }
alertas-score-caiu = Score caiu { $pontos } pontos desde { $desde }
alertas-score-faixa-piorou = Faixa piorou de { $antes } para { $agora }
alertas-score-red-flag = Nova red flag: { $flag }

## Rotinas do gestor

alertas-rotina-revisao = Revisão contínua
alertas-rotina-fechamento = Fechamento mensal
alertas-rotina-alertas = Alertas
alertas-rotina-sem-pendentes = Nenhum lançamento sem categoria.
alertas-rotina-revisao-feita = { $sugeridos } de { $total } lançamentos sem categoria receberam sugestão; { $propostas } propostas na fila.
alertas-rotina-motor-categorizacao-fora = motor de categorização indisponível: { $erro }
alertas-rotina-motor-fora = motor indisponível, relatório sem IA: { $erro }
alertas-rotina-fechamento-gravado = Fechamento de { $mes } gravado na caixa de alertas.
alertas-rotina-alertas-novos = { $n } alertas novos.

## Fechamento mensal (sem motor de IA)

alertas-fechamento-titulo = Fechamento de { $mes }
alertas-fechamento-resumo = **Resumo de { $mes }**
alertas-fechamento-fluxo = - Receitas: { $receitas } · Despesas: { $despesas } · Saldo: { $saldo }{ $poupanca }
alertas-fechamento-poupanca = { " " }· Taxa de poupança: { $pct }
alertas-fechamento-patrimonio = - Patrimônio líquido: { $valor } ({ $sinal }{ $variacao } no mês)
alertas-fechamento-maiores = - Maiores gastos: { $lista }
alertas-fechamento-recomendacoes = **Recomendações**
alertas-fechamento-aviso = _Análise baseada nos seus dados, não é aconselhamento financeiro profissional._
alertas-rec-orcamento = { $categoria } passou do orçamento: { $gasto } gastos para { $previsto } previstos. Revise os lançamentos da categoria ou ajuste o previsto do próximo mês.
alertas-rec-poupanca = A taxa de poupança ficou em { $pct }. Defina uma meta de poupança mensal e acompanhe as maiores categorias de gasto.
alertas-rec-despesas-subiram = As despesas subiram de { $antes } para { $agora } ({ $pct } a mais que no mês anterior). Veja quais gastos foram pontuais.
alertas-rec-sem-categoria = { $n } lançamentos do mês estão sem categoria: categorize para os relatórios ficarem corretos; a revisão contínua já sugere categorias.
alertas-rec-meta = A meta "{ $meta }" está fora do ritmo: aporte mensal necessário de { $valor }.
alertas-rec-carteira = Carteira fora da alocação-alvo em { $classe }: { $atual } da carteira para alvo de { $alvo }. Considere direcionar os próximos aportes.
alertas-rec-maiores = Os maiores gastos do mês foram { $lista }: vale conferir se algum pode ser reduzido.
alertas-rec-sem-orcamento = Não há orçamento para o mês: defina o previsto das principais categorias para acompanhar o ritmo de gastos.
alertas-rec-sem-metas = Não há metas cadastradas: comece por uma reserva de emergência de alguns meses de despesas.
alertas-rec-em-dia = Mantenha os lançamentos e as importações em dia para o próximo fechamento ser mais preciso.

## Tipos de alerta (rodapé do alerta)

alertas-tipo-budget-over = Orçamento estourado
alertas-tipo-budget-pace = Ritmo do orçamento
alertas-tipo-due-soon = Vencimento
alertas-tipo-rebalance = Rebalanceamento
alertas-tipo-monthly-close = Fechamento mensal
alertas-tipo-negative-balance = Saldo negativo previsto
alertas-tipo-asset-score = Score de saúde
alertas-tipo-source-disabled = Fonte de dados desligada
alertas-tipo-asset-data-gaps = Dados dos ativos
alertas-tipo-stale-accounts = Contas sem extrato
alertas-tipo-iptu-due = IPTU a vencer
alertas-tipo-iptu-missing = IPTU sem carnê
alertas-tipo-dividend-datacom = Data-com de provento
alertas-tipo-price-target = Preço-alvo
alertas-tipo-subscription-increase = Aumento de assinatura
alertas-tipo-subscription-new = Assinatura nova

## Dados dos ativos (enriquecimento)

alertas-dados-ativos-titulo = { $n } dado(s) dos seus ativos para completar
alertas-dados-ativos-corpo = As fontes oficiais que o Treasury já baixou têm sugestões para **{ $campos } campo(s)** de { $ativos } ativo(s), como setor, segmento, ISIN e CNPJ (ex.: { $exemplos }).
alertas-dados-ativos-onde = Confira a fonte e a confiança de cada um e grave o que quiser em Carteira › Saúde dos ativos › Dados dos ativos. Nada muda sem você escolher.
alertas-reduflacao-titulo = { $produto }: embalagem menor, preço maior
alertas-reduflacao-corpo = { $produto } passou de { $antes } para { $agora } { $unidade } e o preço por { $unidade } mudou { $variacao }. É a reduflação: menos produto pelo mesmo preço (ou mais caro).
alertas-tela-entrou-titulo = { $ticker } passou a cumprir a tela { $tela }
alertas-tela-entrou-corpo = { $ticker } passou a cumprir os filtros da sua tela { $tela }. É uma lista de quem cumpre critérios, não recomendação de investimento.
alertas-tela-saiu-titulo = { $ticker } deixou de cumprir a tela { $tela }
alertas-tela-saiu-corpo = { $ticker }, que está na sua carteira, deixou de cumprir os filtros da tela { $tela }.
alertas-cartao-limite-titulo = { $cartao }: { $pct } do limite usado
alertas-cartao-limite-corpo = { $usado } usados de { $limite } (sobram { $disponivel }), contando as parcelas futuras.
alertas-cartao-anuidade-renova-titulo = { $cartao }: anuidade renova em { $mes }
alertas-cartao-anuidade-renova-corpo = A anuidade de { $valor } renova em breve; nos últimos 12 meses foram { $pago }. É um bom momento para pedir isenção ou desconto ao banco.
alertas-cartao-anuidade-confira-titulo = { $cartao }: confira a anuidade
alertas-cartao-anuidade-isenta-corpo = Cobrança de { $valor } em { $data } numa fatura que tinha atingido a isenção.
alertas-cartao-anuidade-parcela-extra-corpo = Cobrança de { $valor } em { $data } passa do número de parcelas combinado em 12 meses.
alertas-cartao-anuidade-valor-corpo = Cobrança de { $valor } em { $data }; a parcela combinada é { $esperado }.
alertas-cartao-rotativo-titulo = { $cartao }: fatura de { $mes } entrou no rotativo
alertas-cartao-rotativo-corpo-br = Ficaram { $valor } sem pagar no vencimento ({ $data }). O rotativo tem os juros mais altos do mercado e só pode durar até a próxima fatura; depois o banco precisa oferecer o parcelamento. Se puder, quite o saldo ou compare o parcelamento com um empréstimo mais barato.
alertas-cartao-rotativo-corpo = Ficaram { $valor } sem pagar no vencimento ({ $data }). O saldo que fica no cartão costuma ter juros altos; se puder, quite ou compare com um empréstimo mais barato.
alertas-cartao-rotativo-teto-titulo = { $cartao }: encargos acima do valor da dívida
alertas-cartao-rotativo-teto-corpo = Os juros e encargos lançados desde o rotativo ({ $encargos }) já passam do valor que ficou sem pagar ({ $valor }). No Brasil, a Lei 14.690/2023 limita esses encargos a 100% da dívida original; vale conferir a fatura com o banco.
alertas-cartao-cashback-titulo = { $cartao }: cashback da fatura de { $mes } não veio
alertas-cartao-cashback-corpo = Previsto { $previsto } e recebido { $recebido } até { $data }. Confira no app do banco ou peça o crédito.
alertas-cartao-pontos-titulo = { $cartao }: pontos da fatura de { $mes } não vieram
alertas-cartao-pontos-corpo = Previstos { $previsto } pontos e creditados { $recebido } até { $data }. Confira no programa ou peça o crédito.
alertas-pontos-vencendo-titulo = { $programa }: pontos vencem { $quando }
alertas-pontos-vencendo-corpo = { $pontos } pontos de { $membro } vencem em { $data }. Use, transfira ou renove antes.

## Contas sem extrato recente

alertas-contas-desatualizadas-titulo = { $n ->
    [one] 1 conta sem extrato recente. Importar?
   *[other] { $n } contas sem extrato recente. Importar?
}
alertas-contas-desatualizadas-corpo = O saldo pode estar incompleto: { $contas }. Importe o extrato ou confira o saldo para a saúde financeira e a reserva usarem o valor certo.
alertas-contas-desatualizadas-item = { $conta } (último lançamento em { $data })
alertas-contas-desatualizadas-item-sem = { $conta } (sem lançamentos)
