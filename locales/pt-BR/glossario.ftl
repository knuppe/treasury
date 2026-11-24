# Glossário: os termos do app explicados no "O que é isso?" do painel simples e nos ícones ?
# ao lado dos rótulos (docs/profile.md, docs/i18n-design.md). pt-BR é o idioma de origem.
#
# Cada termo tem -termo (o nome), -simples (tom "bem simples"), -equilibrado (tons equilibrado e
# técnico; sem ela, vale a simples) e -busca: expressão regular testada contra o rótulo em
# minúsculas e sem acento, com as alternativas separadas por |. O primeiro termo que casar vence,
# na ordem deste arquivo (a mesma de content/glossary.ts), então os mais específicos vêm antes. A
# busca de cada idioma segue os rótulos daquele idioma. Sem chaves na busca (sintaxe do Fluent).

glossario-reserva-termo = Reserva de emergência
glossario-reserva-simples = Dinheiro guardado só para imprevistos, como uma consulta médica ou o conserto do carro. O ideal é ter o equivalente a 3 meses dos seus gastos do dia a dia, num lugar de onde dá para tirar na hora (uma poupança, por exemplo).
glossario-reserva-equilibrado = Reserva para imprevistos, de 3 a 6 meses dos gastos essenciais, aplicada com liquidez diária e baixo risco (Tesouro Selic, CDB de liquidez diária).
glossario-reserva-busca = reserva de emergencia|^reserva\b

glossario-sobra-termo = O que sobrou
glossario-sobra-simples = É o que entrou no mês menos o que saiu. Se ficar negativo, saiu mais do que entrou: vale ver em que categoria dá para cortar um pouco.
glossario-sobra-busca = \bsobra\b|o que sobrou|saldo do mes

glossario-essenciais-termo = Gastos essenciais
glossario-essenciais-simples = Os gastos que não dá para cortar, como aluguel, mercado, luz, água e transporte. A reserva é calculada em cima deles.
glossario-essenciais-busca = essencia

glossario-taxa_poupanca-termo = Taxa de poupança
glossario-taxa_poupanca-simples = De tudo o que entrou no mês, quanto ficou guardado. Se entrou R$ 1.000 e sobraram R$ 100, a taxa é de 10%.
glossario-taxa_poupanca-equilibrado = Parcela da renda do período que não foi gasta: (receitas − despesas) ÷ receitas. Aportes e resgates de investimentos não contam como gasto.
glossario-taxa_poupanca-busca = taxa de poupanca

glossario-orcamento-termo = Orçamento
glossario-orcamento-simples = Um plano de quanto você pretende gastar em cada coisa no mês. Ajuda a perceber antes quando um gasto está passando do combinado.
glossario-orcamento-busca = ^orcamento\b

glossario-patrimonio_liquido-termo = Patrimônio líquido
glossario-patrimonio_liquido-simples = Tudo o que você tem (dinheiro nas contas, investimentos e bens) menos tudo o que você deve. É a "foto" do quanto a família tem de verdade.
glossario-patrimonio_liquido-equilibrado = Ativos (contas, investimentos a valor de mercado e bens) menos passivos (saldo devedor das dívidas e faturas em aberto).
glossario-patrimonio_liquido-busca = patrimonio liquido

glossario-competencia-termo = Competência
glossario-competencia-simples = Conta o gasto no mês em que ele foi feito, mesmo que a fatura do cartão só seja paga depois. Mostra quanto você gastou de verdade em cada mês.
glossario-competencia-equilibrado = Regime de competência: cada lançamento conta no mês em que aconteceu (data da compra), não no mês em que o dinheiro saiu da conta (regime de caixa).
glossario-competencia-busca = competencia

glossario-juros_compostos-termo = Juros compostos
glossario-juros_compostos-simples = Juros que também rendem juros. Numa dívida, fazem o valor crescer rápido (por isso o rotativo do cartão pesa tanto); num investimento, fazem o dinheiro crescer mais com o tempo.
glossario-juros_compostos-busca = juros compostos

glossario-cdi-termo = CDI
glossario-cdi-simples = Uma taxa de referência dos bancos, que anda junto com a Selic. Quando um investimento rende "100% do CDI", ele acompanha essa taxa.
glossario-cdi-equilibrado = Certificado de Depósito Interbancário: taxa média dos empréstimos de um dia entre bancos, muito próxima da Selic. É o principal indexador da renda fixa pós-fixada.
glossario-cdi-busca = \bcdi\b

glossario-selic-termo = Selic
glossario-selic-simples = A taxa básica de juros do Brasil, definida pelo Banco Central. Quando ela sobe, empréstimos ficam mais caros e a renda fixa rende mais.
glossario-selic-equilibrado = Taxa básica de juros definida pelo Copom a cada 45 dias. Serve de piso para a renda fixa e de instrumento contra a inflação.
glossario-selic-busca = \bselic\b

glossario-ipca-termo = Inflação (IPCA)
glossario-ipca-simples = Quanto os preços subiram. Se a inflação foi de 5% no ano, o que custava R$ 100 passou a custar uns R$ 105.
glossario-ipca-equilibrado = IPCA: índice oficial de inflação do IBGE. O app usa para mostrar ganhos e saldos "reais", descontada a perda de poder de compra.
glossario-ipca-busca = \bipca\b|inflacao

glossario-xirr-termo = XIRR (rentabilidade anual)
glossario-xirr-simples = A rentabilidade da carteira por ano, levando em conta quando cada dinheiro entrou ou saiu. É como comparar com "quanto rendeu ao ano" de uma aplicação.
glossario-xirr-equilibrado = Taxa interna de retorno com datas irregulares: a taxa anual que zera o valor presente de todos os aportes, resgates, proventos e do valor atual. Não é distorcida pelo momento dos aportes.
glossario-xirr-busca = \bxirr\b|\btir\b

glossario-real-termo = Valor real
glossario-real-simples = O resultado já descontando a inflação. Se o dinheiro rendeu 10% e os preços subiram 5%, o ganho real foi de uns 5%: é o que de fato aumentou o que você consegue comprar.
glossario-real-equilibrado = Resultado deflacionado pelo índice escolhido (IPCA por padrão): (1 + nominal) ÷ (1 + inflação) − 1.
glossario-real-busca = \breal\b|\breais\b.*inflacao

glossario-nominal-termo = Valor nominal
glossario-nominal-simples = O resultado "de etiqueta", sem descontar a inflação. Parece maior do que o ganho de verdade quando os preços também subiram.
glossario-nominal-busca = \bnominal\b

glossario-rentabilidade-termo = Rentabilidade
glossario-rentabilidade-simples = Quanto o dinheiro aplicado rendeu, em porcentagem do que foi investido.
glossario-rentabilidade-equilibrado = Retorno sobre o capital investido no período, incluindo variação de preço e proventos recebidos.
glossario-rentabilidade-busca = rentabilidade

glossario-valor_mercado-termo = Valor de mercado
glossario-valor_mercado-simples = Quanto os seus investimentos valem hoje, pelo preço de agora. Se você vendesse tudo agora, receberia mais ou menos isso (antes de impostos).
glossario-valor_mercado-equilibrado = Quantidade × última cotação (ou preço da curva, na renda fixa), convertida para reais. Sem cotação, o app usa o custo e avisa.
glossario-valor_mercado-busca = valor de mercado

glossario-nao_realizado-termo = Resultado não realizado
glossario-nao_realizado-simples = O ganho (ou a perda) que só existe "no papel": o investimento valorizou, mas você ainda não vendeu. Pode mudar até a venda.
glossario-nao_realizado-busca = nao realizado

glossario-realizado-termo = Resultado realizado
glossario-realizado-simples = O ganho (ou a perda) que já virou dinheiro de verdade, porque você vendeu.
glossario-realizado-busca = ^realizado\b

glossario-preco_medio-termo = Preço médio
glossario-preco_medio-simples = Quanto você pagou, em média, por cada cota ou ação, somando todas as compras. É com ele que se calcula o lucro na venda.
glossario-preco_medio-equilibrado = Custo total das compras (com taxas) dividido pela quantidade em carteira; vendas não mudam o preço médio. Base do imposto sobre ganho de capital.
glossario-preco_medio-busca = preco medio|^pm\b

glossario-custo-termo = Custo
glossario-custo-simples = Quanto você pagou pelos investimentos que ainda tem, somando as compras.
glossario-custo-busca = ^custo\b

glossario-provento-termo = Proventos
glossario-provento-simples = Dinheiro que um investimento paga para você só por tê-lo: dividendos de ações, rendimentos de fundos imobiliários e juros de títulos.
glossario-provento-equilibrado = Distribuições ao investidor: dividendos, JCP, rendimentos de FII e amortizações. Entram no resultado total e no XIRR.
glossario-provento-busca = provento|renda passiva

glossario-dividendo-termo = Dividendos
glossario-dividendo-simples = A parte do lucro que uma empresa divide com quem tem ações dela. Cai na sua conta.
glossario-dividendo-busca = dividendo

glossario-jcp-termo = Juros sobre capital próprio (JCP)
glossario-jcp-simples = Outra forma de a empresa dividir o lucro com os acionistas, parecida com os dividendos, mas com imposto descontado na fonte (17,5% desde 2026; 15% até 2025).
glossario-jcp-busca = \bjcp\b|juros sobre capital

glossario-yield-termo = Yield (rendimento)
glossario-yield-simples = Quanto um investimento pagou de proventos em um ano, em porcentagem do preço. Um yield de 8% quer dizer que, a cada R$ 100, ele pagou uns R$ 8 no ano.
glossario-yield-busca = \byield\b|\bdy\b

glossario-come_cotas-termo = Come-cotas
glossario-come_cotas-simples = Um imposto que alguns fundos cobram duas vezes por ano (maio e novembro), diminuindo a quantidade de cotas, mesmo sem você resgatar.
glossario-come_cotas-equilibrado = Antecipação semestral de IR em fundos abertos (15% ou 20%), cobrada pela redução de cotas. Fundos de ações e FII não têm.
glossario-come_cotas-busca = come.cotas

glossario-alocacao-termo = Alocação
glossario-alocacao-simples = Como o seu dinheiro investido está dividido entre tipos de investimento (renda fixa, ações, fundos imobiliários...). Dividir ajuda a não depender de uma coisa só.
glossario-alocacao-busca = alocacao

glossario-rebalanceamento-termo = Rebalanceamento
glossario-rebalanceamento-simples = Ajustar a carteira para voltar à divisão que você planejou, colocando o próximo dinheiro no que ficou abaixo do alvo.
glossario-rebalanceamento-busca = rebalance

glossario-desvio-termo = Desvio do alvo
glossario-desvio-simples = Quanto cada tipo de investimento está acima ou abaixo da parte que você planejou para ele, em pontos percentuais.
glossario-desvio-busca = desvio

glossario-liquidez-termo = Liquidez
glossario-liquidez-simples = A facilidade de transformar um investimento em dinheiro na conta. A reserva de emergência precisa de liquidez: dá para tirar no mesmo dia.
glossario-liquidez-busca = liquidez

glossario-renda_fixa-termo = Renda fixa
glossario-renda_fixa-simples = Investimentos em que você empresta dinheiro (ao governo, a um banco ou a uma empresa) e recebe juros combinados. Poupança, Tesouro Direto e CDB são renda fixa.
glossario-renda_fixa-busca = renda fixa

glossario-renda_variavel-termo = Renda variável
glossario-renda_variavel-simples = Investimentos cujo valor sobe e desce todo dia, como ações e fundos imobiliários. Podem render mais, mas também podem cair.
glossario-renda_variavel-busca = renda variavel

glossario-fii-termo = Fundos imobiliários (FII)
glossario-fii-simples = Fundos que investem em imóveis (shoppings, galpões, escritórios) ou em dívidas ligadas a imóveis. Pagam rendimentos todo mês, normalmente sem imposto para pessoa física.
glossario-fii-busca = \bfiis?\b|fundos? imobiliario

glossario-etf-termo = ETF
glossario-etf-simples = Um fundo negociado na bolsa como se fosse uma ação, que copia um índice (como o Ibovespa). Com uma compra só, você tem um pedaço de várias empresas.
glossario-etf-busca = \betfs?\b

glossario-acoes-termo = Ações
glossario-acoes-simples = Pedacinhos de uma empresa. Quem tem ações vira sócio: ganha se a empresa valoriza e recebe parte do lucro (dividendos).
glossario-acoes-busca = ^acoes\b

glossario-tesouro-termo = Tesouro Direto
glossario-tesouro-simples = Títulos do governo federal que qualquer pessoa pode comprar a partir de uns R$ 30. É uma das aplicações mais seguras do país.
glossario-tesouro-equilibrado = Títulos públicos federais (Selic, Prefixado, IPCA+) negociados pela B3. Fora do vencimento, o preço segue a marcação a mercado.
glossario-tesouro-busca = tesouro

glossario-cdb-termo = CDB
glossario-cdb-simples = Um empréstimo que você faz a um banco, em troca de juros. Tem a garantia do FGC até R$ 250 mil por banco.
glossario-cdb-busca = \bcdbs?\b
glossario-rdb-termo = RDB
glossario-rdb-simples = Como o CDB, um empréstimo que você faz a um banco em troca de juros (Nubank e Inter usam esse nome). Tem a garantia do FGC até R$ 250 mil por banco e o IR regressivo.
glossario-rdb-busca = \brdbs?\b

glossario-lci_lca-termo = LCI e LCA
glossario-lci_lca-simples = Parecidas com o CDB, mas o banco usa o dinheiro para imóveis (LCI) ou para o agronegócio (LCA). Para pessoa física, não pagam imposto de renda.
glossario-lci_lca-busca = \blci\b|\blca\b

glossario-previdencia-termo = Previdência privada
glossario-previdencia-simples = Um investimento de longo prazo pensado para a aposentadoria. PGBL pode diminuir o imposto de quem declara no modelo completo; VGBL serve para os outros casos.
glossario-previdencia-busca = previdencia|\bpgbl\b|\bvgbl\b

glossario-aporte-termo = Aporte
glossario-aporte-simples = Dinheiro que você coloca num investimento. Não é um gasto: o dinheiro continua seu, só mudou de lugar.
glossario-aporte-busca = aporte

glossario-resgate-termo = Resgate
glossario-resgate-simples = Tirar dinheiro de um investimento de volta para a conta. Também não é uma receita nova: é o seu dinheiro voltando.
glossario-resgate-busca = resgate

glossario-indexador-termo = Indexador
glossario-indexador-simples = A taxa que serve de régua para os juros de um investimento ou dívida (por exemplo, CDI, IPCA ou uma taxa fixa).
glossario-indexador-busca = indexador

glossario-fatura-termo = Fatura
glossario-fatura-simples = A conta do cartão de crédito: soma o que você comprou no mês e vence numa data fixa. Pagar só o mínimo faz a dívida entrar no rotativo, que tem juros altíssimos.
glossario-fatura-busca = \bfaturas?\b

glossario-rotativo-termo = Rotativo do cartão
glossario-rotativo-simples = O que acontece quando a fatura não é paga inteira: o resto vira uma dívida com juros entre os mais altos do país. Vale evitar sempre que possível.
glossario-rotativo-busca = rotativo

glossario-cet-termo = Custo efetivo total (CET)
glossario-cet-simples = Quanto um empréstimo custa de verdade por ano, somando juros, tarifas, seguros e impostos. É o número para comparar ofertas.
glossario-cet-busca = \bcet\b|custo efetivo|\btaeg?\b|\bapr\b|\bcft\b|\bcae\b

glossario-amortizacao-termo = Amortização
glossario-amortizacao-simples = A parte da parcela que diminui a dívida de fato (o resto da parcela são os juros). Adiantar amortização reduz os juros que você ainda pagaria.
glossario-amortizacao-busca = amortiza

glossario-price_sac-termo = Tabela Price e SAC
glossario-price_sac-simples = Duas formas de calcular as parcelas de um financiamento. Na Price, as parcelas são iguais; no SAC, começam maiores e vão diminuindo, e no total costuma sair mais barato.
glossario-price_sac-busca = \bprice\b|\bsac\b

glossario-divida_cara-termo = Dívida cara
glossario-divida_cara-simples = Dívida com juros altos, como rotativo do cartão, cheque especial e alguns empréstimos pessoais. Costuma valer a pena quitar primeiro.
glossario-divida_cara-busca = divida cara

glossario-saldo_devedor-termo = Saldo devedor
glossario-saldo_devedor-simples = Quanto ainda falta pagar de uma dívida hoje, sem contar os juros futuros.
glossario-saldo_devedor-busca = saldo devedor

glossario-conciliado-termo = Conciliado
glossario-conciliado-simples = Um lançamento que você já conferiu com o extrato do banco. Serve para ter certeza de que o saldo do app bate com o do banco.
glossario-conciliado-busca = concilia

glossario-previsto-termo = Previsto
glossario-previsto-simples = Um lançamento que ainda vai acontecer (uma conta agendada, um salário que vai cair). Entra nas previsões, mas não mexe no saldo de hoje.
glossario-previsto-busca = previst

glossario-recorrencia-termo = Recorrência
glossario-recorrencia-simples = Um gasto ou uma entrada que se repete (aluguel todo mês, assinatura, salário). O app cria os próximos sozinho para você não esquecer.
glossario-recorrencia-busca = recorren

glossario-transferencia-termo = Transferência entre contas
glossario-transferencia-simples = Dinheiro que passou de uma conta sua para outra conta sua. Não é gasto nem receita, por isso não entra nos totais do mês.
glossario-transferencia-busca = transferencia

glossario-titular-termo = Membro da família
glossario-titular-simples = A pessoa da família dona da conta, do cartão, do bem ou da dívida. O que entra e sai numa conta de um membro só fica com ele; o que é de todos fica da família.
glossario-titular-busca = titular

glossario-vencimento-termo = Vencimento
glossario-vencimento-simples = A data-limite para pagar uma conta sem multa (ou, num investimento, a data em que ele termina e o dinheiro volta para você).
glossario-vencimento-busca = vencimento

glossario-cambio-termo = Câmbio
glossario-cambio-simples = O preço de uma moeda em reais (por exemplo, quantos reais custa um dólar). O app usa para somar contas e investimentos em outras moedas.
glossario-cambio-busca = cambio

glossario-deflator-termo = Deflator
glossario-deflator-simples = O índice de inflação usado para descontar a perda de poder de compra dos valores (o app usa o IPCA, se você não escolher outro).
glossario-deflator-busca = deflator

glossario-m2-termo = M2 (emissão de moeda)
glossario-m2-simples = Uma medida de quanto dinheiro existe na economia. Quando cresce muito mais rápido que a produção, tende a puxar os preços para cima.
glossario-m2-busca = \bm2\b

glossario-score_saude-termo = Score de saúde
glossario-score_saude-simples = Uma nota de 0 a 100 que resume se um investimento está bem (lucro, dívidas, preço, histórico), com a explicação de cada parte. Não é recomendação de compra ou venda.
glossario-score_saude-busca = \bscore\b|saude do ativo|saude dos ativos

glossario-volatilidade-termo = Volatilidade
glossario-volatilidade-simples = O quanto o preço de um investimento sobe e desce. Quanto maior, mais "montanha-russa" ele é.
glossario-volatilidade-busca = volatil

glossario-drawdown-termo = Queda máxima (drawdown)
glossario-drawdown-simples = A maior queda que o investimento teve, do ponto mais alto ao mais baixo, no período. Mostra o pior momento que quem investiu passou.
glossario-drawdown-busca = drawdown|queda maxima

glossario-aliquota-termo = Alíquota
glossario-aliquota-simples = A porcentagem de imposto cobrada sobre um valor (por exemplo, 15% sobre o lucro).
glossario-aliquota-busca = aliquota

glossario-darf-termo = DARF
glossario-darf-simples = O boleto da Receita Federal para pagar impostos, como o imposto sobre lucro com ações vendidas no mês.
glossario-darf-busca = \bdarf\b

glossario-ir-termo = Imposto de renda
glossario-ir-simples = Imposto federal sobre o que você ganha (salário, aluguel, lucro com investimentos). Uma parte já é descontada na fonte; a declaração anual acerta a conta.
glossario-ir-busca = imposto de renda|\birpf\b|\bir\b
