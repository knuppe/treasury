# O que o Treasury percebe sem IA (núcleo, `insights`): inflação pessoal, tarifas e juros,
# conta no vermelho, gasto fora do normal, compras pequenas, gastos acima da renda e a divisão
# da renda (docs/insights.md). Montados na hora, no idioma da instalação. pt-BR é o idioma de
# origem.

## Grupos do IPCA

insights-grupo-ipca-g1 = Alimentação e bebidas
insights-grupo-ipca-g2 = Habitação
insights-grupo-ipca-g3 = Artigos de residência
insights-grupo-ipca-g4 = Vestuário
insights-grupo-ipca-g5 = Transportes
insights-grupo-ipca-g6 = Saúde e cuidados pessoais
insights-grupo-ipca-g7 = Despesas pessoais
insights-grupo-ipca-g8 = Educação
insights-grupo-ipca-g9 = Comunicação

## Inflação pessoal

insights-inflacao-titulo = Sua inflação em 12 meses: { $pessoal }
insights-inflacao-titulo-ipca = Sua inflação em 12 meses: { $pessoal } (IPCA: { $ipca })
insights-inflacao-corpo = Com o peso que cada grupo tem nos seus gastos (e não na cesta média do IBGE), os preços do que você compra subiram { $pessoal } de { $de } a { $ate }.
insights-inflacao-mais-pesou = O que mais pesou: { $grupo } ({ $peso } dos gastos, subiu { $variacao }).
insights-inflacao-acao = Reajustes de renda e de orçamento menores que essa inflação significam perder poder de compra.

## Tarifas e juros

insights-tarifas-titulo = { $valor } em tarifas e juros em 12 meses
insights-tarifas-corpo = De { $de } a { $ate }, { $tarifas } saíram em tarifas bancárias ({ $n_tarifas } cobranças) e { $juros } em juros, IOF e encargos ({ $n_juros } cobranças). É dinheiro que não compra nada.
insights-tarifas-juros-caros = Juros de cheque especial e de rotativo do cartão estão entre os mais caros do mercado.
insights-tarifas-acao-pacote = Vale saber: os bancos são obrigados a oferecer um pacote de serviços essenciais sem tarifa (Resolução CMN 3.919), e há contas digitais gratuitas.
insights-tarifas-acao-credito = Vale saber: trocar o cheque especial ou o rotativo por um crédito mais barato e pagar a fatura inteira.
insights-tarifas-acao-ambos = Vale saber: os bancos são obrigados a oferecer um pacote de serviços essenciais sem tarifa (Resolução CMN 3.919), e há contas digitais gratuitas; trocar o cheque especial ou o rotativo por um crédito mais barato e pagar a fatura inteira.

## Conta no vermelho

insights-vermelho-titulo = { $conta } ficou no vermelho { $dias } dias
insights-vermelho-corpo = Nos últimos 90 dias, o saldo de { $conta } ficou negativo em { $dias } dias, chegando a { $minimo }.
insights-vermelho-taxa = O cheque especial custa em média { $taxa }% ao mês nos bancos (Banco Central).
insights-vermelho-acao = Ver a previsão de saldo e, se for recorrente, trocar o cheque especial por um crédito mais barato (Dívidas › Juros do mercado).
insights-fgts-titulo = O FGTS de { $mes } não foi depositado ({ $conta })
insights-fgts-corpo = { $n ->
    [one] O extrato da Caixa não mostra o depósito de { $mes }, e o prazo do empregador (dia 20 do mês seguinte) já passou.
   *[other] O extrato da Caixa não mostra { $n } depósitos, a partir de { $mes }, e o prazo do empregador (dia 20 do mês seguinte) já passou.
}
insights-fgts-acao = Confira o holerite e fale com o RH da empresa; se não resolver, a Caixa e o Ministério do Trabalho recebem a denúncia.
insights-previdencia-titulo = { $plano }: aportes para conferir
insights-previdencia-pendencias =
    { $n ->
        [one] { $n } pendência no histórico de pagamentos importado:
       *[other] { $n } pendências no histórico de pagamentos importado:
    }
insights-previdencia-sem-cota =
    { $aportes ->
        [one] { $aportes } aporte espera a cota da CVM desde { $desde } e entram no saldo pelo valor aportado.
       *[other] { $aportes } aportes esperam a cota da CVM desde { $desde } e entram no saldo pelo valor aportado.
    }
insights-previdencia-acao = Abra o plano em Ativos, confira as pendências e mande ao agente o pagamento que faltou (ou o print que mostra esse trecho).

## Gasto fora do normal

insights-acima-titulo = { $categoria } acima do normal em { $mes }
insights-acima-corpo = Em { $mes }, { $categoria } somou { $valor }, { $diferenca } a mais que o normal dos meses anteriores ({ $normal } por mês, a mediana de 6 meses).
insights-acima-acao = Abrir os lançamentos da categoria para ver o que mudou; se for um gasto que vai continuar, ajustar o orçamento.
insights-abaixo-titulo = { $categoria } abaixo do normal em { $mes }
insights-abaixo-corpo = Em { $mes }, { $categoria } somou { $valor }, { $diferenca } a menos que o normal ({ $normal } por mês).
insights-abaixo-acao = Se a economia continuar, dá para guardar a diferença numa meta.

## Compras pequenas

insights-pequenas-titulo = { $n } compras pequenas somaram { $total } em { $mes }
insights-pequenas-corpo = Compras de até { $limite } cada, que parecem pouco uma a uma. As que mais pesaram: { $lista }.
insights-pequenas-item = { $categoria } ({ $valor })
insights-pequenas-acao = Um teto mensal para essas categorias no orçamento ajuda a enxergar o total.
insights-sem-categoria = sem categoria

## Gastos acima da renda

insights-deficit-titulo = Gastos acima da renda em { $valor } nos últimos 3 meses
insights-deficit-corpo = De { $de } a { $ate }, entraram { $renda } e saíram { $despesas } em despesas. A diferença veio de saldo guardado, cheque especial, cartão ou empréstimo.
insights-deficit-acao = Ver as maiores categorias no orçamento e a previsão de saldo dos próximos meses.

## Divisão da renda (50/30/20)

insights-divisao-titulo = Da renda: { $essencial } no essencial, { $resto } no resto, { $sobra } sobrou
insights-divisao-corpo = De { $de } a { $ate }, entraram { $renda }: { $essencial } foram para o essencial (moradia, mercado, saúde, transporte… as categorias marcadas como essenciais), { $resto } para o resto e { $sobra } sobraram. Uma referência comum é 50% para o essencial, 30% para o resto e 20% para guardar.
insights-divisao-acao-folga = Boa folga: guardar a sobra numa meta (reserva de emergência primeiro) faz ela render.
insights-divisao-acao-essencial = O essencial pesa muito: renegociar contas fixas (aluguel, plano, internet, telefone) costuma dar mais resultado do que cortar pequenos gastos.
insights-divisao-acao-teto = Um teto no orçamento para o que não é essencial ajuda a sobrar mais no fim do mês.

insights-kit-titulo = Gere o kit de recuperação do cofre
insights-kit-corpo = Se a senha mestra for esquecida, ninguém consegue abrir o cofre, nem o Treasury. O kit de recuperação é um código que abre o cofre sem a senha: imprima e guarde em papel, longe do computador. Se você já gerou um antes, gere de novo para o app registrar (o código é o mesmo).
insights-kit-velho-titulo = O kit de recuperação antigo não abre mais o cofre
insights-kit-velho-corpo = A senha mestra mudou depois que o kit foi gerado, e o código impresso deixou de valer. Gere um kit novo e troque o papel guardado.
insights-kit-acao = Abra Ajustes › Segurança e gere o kit.
