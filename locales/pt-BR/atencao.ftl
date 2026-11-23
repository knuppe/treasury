# "Para resolver" (núcleo `attention` e a interface): a lista única do que precisa da pessoa e a
# linha que diz até quando os números valem (docs/to-resolve.md). pt-BR é o idioma de origem.

## Núcleo

atencao-erro-item = Este item não pode ser dispensado.
atencao-erro-so-interface = Isto só pode ser feito na interface do Treasury.
atencao-venceu-titulo = { $descricao } venceu em { $data }
atencao-venceu-corpo = { $valor } em { $conta } continua como previsto. Se já foi pago, marque como pago ou importe o extrato; se não, pague para evitar juros.
atencao-caixa-titulo = { $n ->
    [one] 1 arquivo esperando na caixa de entrada
   *[other] { $n } arquivos esperando na caixa de entrada
}
atencao-caixa-corpo = Arquivos que chegaram à pasta de entrada e ainda não foram importados.


## Núcleo: tabelas anuais embutidas

tabelas-inss = INSS (piso e teto)
tabelas-pensao-pt = Segurança Social (IAS e pensões mínimas)
tabelas-pensao-us = Social Security (pontos de corte)
atencao-regras-titulo = { $n ->
    [one] 1 regra de categoria pronta para criar
   *[other] { $n } regras de categoria prontas para criar
}
atencao-regras-corpo = Você já deu a mesma categoria três vezes a { $nomes }. Com a regra, os próximos lançamentos entram com a categoria.
atencao-tabelas-titulo = { $n ->
    [one] 1 tabela do app pode estar desatualizada
   *[other] { $n } tabelas do app podem estar desatualizadas
}
atencao-tabelas-item = { $nome } de { $ano }
atencao-tabelas-corpo = Em uso: { $tabelas }. As estimativas continuam saindo, com os valores desse ano.
atencao-tabelas-acao = Atualize o Treasury para receber as tabelas novas.

## Interface: cartão do Painel e sino da barra superior

atencao-titulo = Para resolver
atencao-origem = calculado pelo app, sem IA
atencao-vazio = Tudo em dia. Nada para resolver agora.
atencao-resolver = Marcar como resolvido
atencao-dispensar-dica = Sai da lista agora. Se continuar valendo, volta em uma semana.
atencao-ver-todos = Ver os { $n }
atencao-ver-alertas = Ver todos os alertas
atencao-sino = { $n ->
    [0] Nada para resolver
    [one] 1 item para resolver
   *[other] { $n } itens para resolver
}
atencao-gravidade-critical = Urgente
atencao-gravidade-warning = Atenção
atencao-gravidade-info = Aviso

## Interface: até quando os números valem

atencao-cobertura-completa = Contas em dia: dados até { $data }.
atencao-cobertura-completa-sem-data = Contas em dia.
atencao-cobertura-parcial = { $n ->
    [one] 1 conta sem extrato recente ({ $contas }).
   *[other] { $n } contas sem extrato recente ({ $contas }).
} Os totais podem estar incompletos.
atencao-cobertura-mais = { $nomes } e mais { $n }
atencao-cobertura-importar = Importar extrato

## Núcleo: seu próximo passo (os degraus)

degraus-know-titulo = Saber para onde o dinheiro vai
degraus-balance-titulo = Gastar menos do que ganha
degraus-credit-titulo = Sair do crédito caro
degraus-reserve-titulo = Montar a reserva de emergência
degraus-debts-titulo = Deixar as dívidas leves
degraus-invest-titulo = Investir e proteger o que tem
degraus-know-sem-contas = Ainda não há contas cadastradas.
degraus-know-sem-contas-acao = Cadastre a conta que você mais usa. Dá para começar só com ela.
degraus-know-meses = { $n ->
    [0] Ainda não há um mês completo de lançamentos.
    [one] Há 1 mês completo de lançamentos.
   *[other] Há { $n } meses completos de lançamentos.
} Com { $alvo } o Treasury consegue dizer para onde o dinheiro vai.
degraus-know-meses-acao = Importe os extratos dos últimos meses ou continue anotando os gastos.
degraus-know-categorias = { $pct } dos lançamentos dos últimos 90 dias têm categoria. A partir de { $alvo }, os totais por categoria passam a ser confiáveis.
degraus-know-categorias-acao = { $n ->
    [one] Dê uma categoria ao lançamento que falta.
   *[other] Dê uma categoria aos { $n } lançamentos que faltam.
}
degraus-balance-sem-renda = { $n ->
    [one] Não há receitas lançadas no último mês
   *[other] Não há receitas lançadas nos últimos { $n } meses
}, então não dá para saber se sobra ou falta.
degraus-balance-sem-renda-acao = Lance o salário e as outras entradas, ou importe o extrato da conta em que elas caem.
degraus-balance-acima = { $n ->
    [one] No último mês
   *[other] Nos últimos { $n } meses
} saiu { $pct } a mais do que entrou{ $n ->
    [one] .
   *[other] { $meses ->
        [one] , e em 1 deles os gastos passaram da renda.
       *[other] , e em { $meses } deles os gastos passaram da renda.
    }
}
degraus-balance-acima-acao = Veja no orçamento as categorias que mais cresceram e escolha uma para reduzir este mês.
degraus-credit-rotativo = Nos últimos meses, a fatura do cartão ficou sem pagar inteira em { $rotativo ->
    [one] 1 deles
   *[other] { $rotativo } deles
}. O rotativo está entre os créditos mais caros.
degraus-credit-cheque = Nos últimos meses, a conta ficou no vermelho em { $cheque ->
    [one] 1 deles
   *[other] { $cheque } deles
}. O cheque especial está entre os créditos mais caros.
degraus-credit-ambos = Nos últimos meses houve fatura do cartão sem pagar inteira e conta no vermelho. Rotativo e cheque especial estão entre os créditos mais caros.
degraus-credit-acao = Troque essa dívida por um crédito mais barato e veja o plano de quitação em Dívidas.
degraus-reserve-essenciais = A reserva cobre { $meses } meses de gastos essenciais. O próximo marco é { $alvo ->
    [one] 1 mês
   *[other] { $alvo } meses
}.
degraus-reserve-gastos = A reserva cobre { $meses } meses de gastos. O próximo marco é { $alvo ->
    [one] 1 mês
   *[other] { $alvo } meses
}.
degraus-reserve-acao = Separe um valor todo mês, numa aplicação que dê para resgatar no mesmo dia, até chegar lá.
degraus-debts-peso = As parcelas de empréstimos e financiamentos levam { $pct } da renda. Acima de { $alvo } o orçamento fica apertado.
degraus-debts-acao = Simule no plano de quitação o que acontece ao pagar um pouco a mais por mês na dívida mais cara.
degraus-invest-porque = As contas fecham, não há crédito caro, a reserva está montada e as dívidas cabem na renda.
degraus-invest-comecar-acao = O que passar da reserva pode ir para objetivos de prazo mais longo. Crie uma meta e defina quanto guardar por mês.
degraus-invest-carteira-acao = Confira a alocação, a concentração e o custo das taxas da carteira.

## Núcleo: fechar o mês

fechamento-erro-mes = Só dá para fechar um mês que já terminou.
fechamento-erro-passo = Passo desconhecido do fechamento do mês.
fechamento-erro-incompleto = Ainda faltam passos para fechar este mês.
fechamento-statements-titulo = Trazer os extratos
fechamento-statements-falta = { $n ->
    [one] 1 conta sem extrato recente: { $contas }.
   *[other] { $n } contas sem extrato recente: { $contas }.
}
fechamento-categories-titulo = Dar categoria aos lançamentos
fechamento-categories-falta = { $n ->
    [one] 1 lançamento do mês sem categoria.
   *[other] { $n } lançamentos do mês sem categoria.
}
fechamento-balances-titulo = Conferir os saldos
fechamento-balances-falta = Compare o saldo de cada conta com o do banco e marque quando bater.
fechamento-review-titulo = Resolver as pendências
fechamento-review-falta = { $n ->
    [one] 1 item em Para resolver.
   *[other] { $n } itens em Para resolver.
}
fechamento-summary-titulo = Ler o resumo do mês
fechamento-summary-falta = Veja nos relatórios para onde o dinheiro foi e marque quando ler.

## Interface: próximo passo e fechar o mês

degraus-cartao = Seu próximo passo
degraus-posicao = Degrau { $n } de { $total }
degraus-estimado = Leitura provisória: há contas sem extrato recente.
degraus-feito = feito
degraus-atual = agora
degraus-depois = depois
fechamento-cartao = Fechar { $mes }
fechamento-progresso = { $n } de { $total }
fechamento-origem = cinco passos, sem IA
fechamento-marcar = Marcar como feito
fechamento-desmarcar = Desmarcar
fechamento-fechar = Marcar o mês como fechado
fechamento-falta = Faltam passos para fechar o mês.
fechamento-passo-feito = Feito
fechamento-passo-pendente = Pendente
atencao-compras-titulo = { $n } compras de mercado deste mês sem itens
atencao-compras-corpo = Detalhe os itens das que quiser acompanhar (carne, café, leite): o preço de cada produto e a inflação da sua cesta saem daí. Não precisa ser a nota inteira.

atencao-nao-avisar = Não avisar mais desta
atencao-nao-avisar-dica = Para despesas que vocês pagam sem precisar de lembrete, como diarista. Continua na previsão e no orçamento; o aviso volta pelo sino em Lançamentos › Recorrências.
atencao-nao-avisar-feito = Pronto: esta recorrência não avisa mais o vencimento.
atencao-nao-avisar-erro = Não deu para desligar o aviso
