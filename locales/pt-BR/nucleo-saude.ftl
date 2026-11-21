# Saúde financeira (núcleo, `health`): os modelos da nota, as faixas, as dimensões, os
# indicadores (meta, explicação, conta feita e sugestões), o resumo e os avisos (docs/health.md).
# Montados na hora, no idioma da instalação. pt-BR é o idioma de origem.

## Modelos (Ajustes › Saúde financeira)

nucleo-saude-modelo-isf = I-SFB (Febraban e Banco Central)
nucleo-saude-modelo-isf-descricao = Inspirado no Índice de Saúde Financeira do Brasileiro: nota de 0 a 100 em quatro dimensões (comportamento, habilidade, liberdade e segurança), comparada com a média nacional de 56,7 em 2024. Estimativa a partir dos seus dados, não o questionário oficial.
nucleo-saude-modelo-classicos = Indicadores clássicos
nucleo-saude-modelo-classicos-descricao = As razões da educação financeira, cada uma com meta: poupar 20% da renda, reserva de 6 meses de gastos essenciais, parcelas até 15% da renda, moradia até 30%, nenhum rotativo e patrimônio crescendo.
nucleo-saude-modelo-desativado = Desativado
nucleo-saude-modelo-desativado-descricao = Sem análise de saúde financeira: o card some do painel e a seção dos relatórios fica vazia.
nucleo-saude-desativada = Análise de saúde financeira desativada em Ajustes.
nucleo-saude-erro-mes-futuro = a saúde financeira só pode ser avaliada até o mês atual
nucleo-saude-erro-so-interface = o modelo de saúde financeira só pela interface do app

## Valores medidos e períodos

nucleo-saude-valor-mes = { $n } mês
nucleo-saude-valor-meses = { $n } meses
nucleo-saude-valor-de-meses = { $n } de { $total } meses
nucleo-saude-valor-em-dia = { $n } de { $total } em dia
nucleo-saude-nenhum-mes = nenhum mês
nucleo-saude-n-meses = { $n ->
        [1] 1 mês
       *[other] { $n } meses
    }
nucleo-saude-sem-meses = sem meses
nucleo-saude-base-competencia = competência
nucleo-saude-base-caixa = caixa

## Valores que entram na conta

nucleo-saude-quantia-receitas = Receitas
nucleo-saude-quantia-despesas = Despesas
nucleo-saude-quantia-sobra = Sobra
nucleo-saude-quantia-reserva = Reserva
nucleo-saude-quantia-gasto-essencial = Gasto essencial por mês
nucleo-saude-quantia-parcelas = Parcelas
nucleo-saude-quantia-moradia = Moradia
nucleo-saude-quantia-patrimonio-em = Patrimônio em { $mes }
nucleo-saude-quantia-variacao = Variação

## Sem dados (o motivo)

nucleo-saude-sem-dados-meses = Sem dados: são precisos ao menos { $minimo } meses completos de lançamentos ({ $n ->
        [1] { $n } analisado
       *[other] { $n } analisados
    }).
nucleo-saude-sem-dados-receita = Sem dados: nenhuma receita registrada em { $periodo }.
nucleo-saude-sem-dados-gastos = Sem dados: são precisos ao menos { $minimo } meses completos de gastos para a média ({ $n ->
        [1] { $n } analisado
       *[other] { $n } analisados
    }).
nucleo-saude-sem-dados-essenciais = Sem dados: nenhuma despesa em categoria essencial nos últimos meses. Marque as categorias essenciais em Ajustes › Categorias.
nucleo-saude-sem-dados-moradia = Sem dados: não há categoria de despesa "Moradia" nem financiamento imobiliário.
nucleo-saude-sem-dados-credito = Sem dados: nenhuma conta corrente nem cartão com pagamento de fatura registrado.
nucleo-saude-sem-dados-patrimonio = Sem dados: são precisos ao menos { $minimo } meses de patrimônio ({ $n ->
        [1] { $n } encontrado
       *[other] { $n } encontrados
    }).
nucleo-saude-sem-dados-parcelas = Sem dados: nenhuma parcela de empréstimo nem fatura de cartão com pagamento registrado venceu no período.
nucleo-saude-sem-dados-orcamento = Sem dados: nenhum mês de { $periodo } tem orçamento de despesas.
nucleo-saude-sem-dados-movimento = Sem dados: nenhum movimento no período.

## Como foi calculado

nucleo-saude-conta-poupanca = (receitas − despesas) ÷ receitas de { $periodo } ({ $n } meses, base { $base }); transferências, pagamento de fatura e aportes não contam.
nucleo-saude-conta-regular = Meses com taxa de poupança de 10% ou mais ÷ meses com receita, em { $periodo }.
nucleo-saude-conta-reserva = Reserva ({ $fonte }) ÷ média mensal das despesas em categorias essenciais dos últimos { $n } meses completos.
nucleo-saude-fonte-meta = o valor atual da meta de reserva de emergência (contas e posições ligadas a ela)
nucleo-saude-fonte-saldos = o saldo das contas corrente, poupança e dinheiro
nucleo-saude-conta-comprometimento = Parcelas com vencimento em { $periodo } ÷ receitas do período; { $contratos }. Parcelamentos no cartão contam como despesa, não aqui.
nucleo-saude-contratos-nenhum = nenhum empréstimo ou financiamento com parcela no período
nucleo-saude-contratos = parcelas de { $n ->
        [1] { $n } contrato
       *[other] { $n } contratos
    } (empréstimos e financiamentos)
nucleo-saude-conta-moradia = Despesas em Moradia (aluguel, condomínio, contas da casa…) e parcelas de financiamento imobiliário lançadas fora dela ÷ receitas de { $periodo }.
nucleo-saude-credito-cheque = cheque especial em { $meses }
nucleo-saude-credito-rotativo = fatura não paga inteira em { $meses }
nucleo-saude-credito-nenhum = nenhum uso
nucleo-saude-conta-credito = Meses de { $periodo } com saldo negativo em conta corrente (cheque especial) ou fatura de cartão não paga inteira até { $dias } dias após o vencimento (rotativo): { $detalhe }.
nucleo-saude-conta-credito-sem-cartoes = Cartões sem pagamento de fatura registrado ficam de fora.
nucleo-saude-base-patrimonio-inicial = o patrimônio inicial
nucleo-saude-base-despesas = as despesas do período (o patrimônio inicial é pequeno)
nucleo-saude-conta-patrimonio = Variação do patrimônio líquido (contas, carteira, bens e dívidas) de { $de } a { $ate } ÷ { $base }. Valor nominal, sem descontar a inflação.
nucleo-saude-conta-em-dia = Parcelas de empréstimos ({ $parcelas_em_dia } em dia, { $parcelas_atraso } em atraso) e faturas de cartão ({ $faturas_pagas } pagas inteiras, { $faturas_nao } não) vencidas em { $periodo }, com folga de { $dias } dias. Parcela não conciliada depois do vencimento conta como em atraso.
nucleo-saude-conta-orcamento = Meses com despesas até o total previsto ÷ meses com orçamento em { $periodo } (base { $base }).
nucleo-saude-conta-azul = Meses em que as receitas cobriram as despesas ÷ meses com movimento, em { $periodo } (base { $base }).

## Indicadores: nome, meta, o que mede, sugestões e o que fazer sem dados

nucleo-saude-sem-dados-3-meses = Registre ou importe as receitas e despesas de pelo menos 3 meses completos.

nucleo-saude-poupanca = Taxa de poupança
nucleo-saude-poupanca-meta = ≥ 20% da renda
nucleo-saude-poupanca-explicacao = Quanto da renda sobra depois das despesas, nos últimos 12 meses.
nucleo-saude-poupanca-dica-1 = Pague-se primeiro: programe uma transferência para a poupança ou a corretora no dia do salário.
nucleo-saude-poupanca-dica-2 = Reveja as três maiores categorias de gasto em Relatórios › Composição e ponha limites no orçamento.
nucleo-saude-poupanca-dica-3 = Corte assinaturas e contas fixas que não usa (Relatórios › Recorrentes).

nucleo-saude-regular = Poupança regular
nucleo-saude-regular-meta = 10% ou mais em todo mês
nucleo-saude-regular-explicacao = Em quantos meses foi possível guardar ao menos 10% da renda.
nucleo-saude-regular-dica-1 = Automatize um valor fixo de poupança todo mês, mesmo que pequeno.
nucleo-saude-regular-dica-2 = Provisione ao longo do ano os gastos sazonais (IPVA, IPTU, material escolar) para eles não zerarem a sobra.
nucleo-saude-regular-dica-3 = Nos meses mais fracos, veja em Relatórios › Comparação o que mudou em relação a um mês bom.

nucleo-saude-reserva = Reserva de emergência
nucleo-saude-reserva-meta = ≥ 6 meses de gastos essenciais
nucleo-saude-reserva-explicacao = Por quantos meses a reserva cobre os gastos essenciais sem nenhuma renda.
nucleo-saude-reserva-dica-1 = Defina (ou reforce) a meta de reserva de emergência em 05 · Orçamento › Metas, com alvo de 6 meses de gastos essenciais.
# O trecho entre parênteses é o mesmo de `nucleo-pais-reserva-br` (trocado pelos produtos do país).
nucleo-saude-reserva-dica-2 = Guarde a reserva em aplicação de liquidez diária e baixo risco (Tesouro Selic ou CDB com liquidez diária).
nucleo-saude-reserva-dica-3 = Direcione para a reserva extras como 13º, restituição do IR e PLR até chegar ao alvo.
nucleo-saude-reserva-sem-dados = Marque as categorias essenciais em Ajustes › Categorias e registre ao menos 3 meses de gastos.

nucleo-saude-comprometimento = Renda comprometida com parcelas
nucleo-saude-comprometimento-meta = ≤ 15% da renda
nucleo-saude-comprometimento-explicacao = Parte da renda que vai para parcelas de empréstimos e financiamentos.
nucleo-saude-comprometimento-dica-1 = Evite novas parcelas até o comprometimento ficar abaixo de 15% da renda.
nucleo-saude-comprometimento-dica-2 = Simule portabilidade ou amortização extraordinária do contrato mais caro em 07 · Dívidas.
nucleo-saude-comprometimento-dica-3 = Quite primeiro as dívidas de juros mais altos (cartão, cheque especial, pessoal).
nucleo-saude-comprometimento-sem-dados = Registre as receitas de pelo menos 3 meses completos e cadastre os contratos em 07 · Dívidas.

nucleo-saude-moradia = Custo da moradia
nucleo-saude-moradia-meta = ≤ 30% da renda
nucleo-saude-moradia-explicacao = Parte da renda que vai para morar: aluguel ou financiamento, condomínio e contas da casa.
nucleo-saude-moradia-dica-1 = Moradia acima de 30% da renda aperta o resto do orçamento: reveja contas de consumo, condomínio e seguros.
nucleo-saude-moradia-dica-2 = Na renovação do aluguel ou num financiamento novo, mire num custo de até 30% da renda.
# O trecho "amortizar com o FGTS…" é o mesmo de `nucleo-pais-fgts-br`.
nucleo-saude-moradia-dica-3 = Se o financiamento pesa, simule amortizar com o FGTS ou portar para uma taxa menor em 07 · Dívidas.
nucleo-saude-moradia-sem-dados = Lance os gastos da casa na categoria Moradia (ou cadastre o financiamento em 07 · Dívidas).

nucleo-saude-credito = Rotativo e cheque especial
nucleo-saude-credito-meta = nenhum mês
nucleo-saude-credito-explicacao = Meses em que se usou crédito caro: cheque especial ou o rotativo do cartão.
nucleo-saude-credito-dica-1 = Pague a fatura do cartão inteira: o rotativo cobra os juros mais altos do mercado.
# O trecho entre parênteses é o mesmo de `nucleo-pais-consignado-br`.
nucleo-saude-credito-dica-2 = Se faltar dinheiro, troque o rotativo ou o cheque especial por um crédito mais barato (consignado, por exemplo).
nucleo-saude-credito-dica-3 = Cadastre o limite e as taxas do cheque especial na conta, em 03 · Contas, para ver o custo estimado.
nucleo-saude-credito-sem-dados = Registre os pagamentos de fatura do cartão e os lançamentos da conta corrente.

nucleo-saude-patrimonio = Evolução do patrimônio
nucleo-saude-patrimonio-meta = crescer ≥ 5% em 12 meses
nucleo-saude-patrimonio-explicacao = Quanto o patrimônio líquido cresceu (ou caiu) nos últimos 12 meses.
nucleo-saude-patrimonio-dica-1 = Direcione parte da sobra de todo mês para investimentos ou para amortizar dívidas.
nucleo-saude-patrimonio-dica-2 = Atualize o valor dos bens e as cotações da carteira para o patrimônio refletir a realidade.
nucleo-saude-patrimonio-dica-3 = Reveja gastos grandes do ano em Relatórios › Tendência: eles explicam a queda.
nucleo-saude-patrimonio-sem-dados = Cadastre as contas com o saldo inicial e mantenha alguns meses de lançamentos.

nucleo-saude-em-dia = Contas em dia
nucleo-saude-em-dia-meta = todas pagas até o vencimento
nucleo-saude-em-dia-explicacao = Parcelas de empréstimos e faturas de cartão pagas inteiras até o vencimento.
nucleo-saude-em-dia-dica-1 = Ative o débito automático das parcelas e da fatura do cartão.
nucleo-saude-em-dia-dica-2 = Concilie as parcelas em 07 · Dívidas: as pagas e não conciliadas contam como em atraso.
nucleo-saude-em-dia-dica-3 = Use os Próximos vencimentos do painel para se programar.
nucleo-saude-em-dia-sem-dados = Cadastre os contratos em 07 · Dívidas e registre os pagamentos de fatura do cartão.

nucleo-saude-orcamento = Gastos dentro do orçamento
nucleo-saude-meta-todos-os-meses = todos os meses
nucleo-saude-orcamento-explicacao = Em quantos meses as despesas ficaram dentro do total previsto no orçamento.
nucleo-saude-orcamento-dica-1 = Reveja o orçamento dos meses em que estourou e ajuste os limites à realidade.
nucleo-saude-orcamento-dica-2 = Acompanhe o ritmo do gasto no card de orçamento do painel ao longo do mês.
nucleo-saude-orcamento-dica-3 = Deixe uma linha para gastos imprevistos no orçamento.
nucleo-saude-orcamento-sem-dados = Monte um orçamento mensal em 05 · Orçamento (copie do mês anterior ou use a sugestão).

nucleo-saude-azul = Meses no azul
nucleo-saude-azul-explicacao = Em quantos meses as receitas cobriram as despesas.
nucleo-saude-azul-dica-1 = Nos meses no vermelho, veja em Relatórios › Comparação o que mudou em relação a um mês no azul.
nucleo-saude-azul-dica-2 = Mantenha um colchão para gastos irregulares, para um mês ruim não virar dívida.
nucleo-saude-azul-dica-3 = Planeje as despesas anuais (seguros, impostos, viagens) como parcelas mensais no orçamento.

## I-SFB: faixas e dimensões

nucleo-saude-faixa-ruim = Ruim
nucleo-saude-faixa-muito-baixa = Muito baixa
nucleo-saude-faixa-baixa = Baixa
nucleo-saude-faixa-ok = Ok
nucleo-saude-faixa-boa = Boa
nucleo-saude-faixa-muito-boa = Muito boa
nucleo-saude-faixa-otima = Ótima

nucleo-saude-dimensao-comportamento = Comportamento
nucleo-saude-dimensao-comportamento-descricao = Pagar as contas em dia, gastar dentro do orçamento e fechar os meses no azul.
nucleo-saude-dimensao-habilidade = Habilidade
nucleo-saude-dimensao-habilidade-descricao = Conseguir poupar parte da renda, com regularidade.
nucleo-saude-dimensao-liberdade = Liberdade
nucleo-saude-dimensao-liberdade-descricao = Quanto da renda está livre de parcelas e de crédito caro (rotativo e cheque especial).
nucleo-saude-dimensao-seguranca = Segurança
nucleo-saude-dimensao-seguranca-descricao = Reserva para imprevistos e patrimônio que cresce.

## Indicadores clássicos: faixas

nucleo-saude-faixa-critica = Crítica
nucleo-saude-faixa-fragil = Frágil
nucleo-saude-faixa-estavel = Estável
nucleo-saude-faixa-saudavel = Saudável

## Resumo e avisos

nucleo-saude-resumo-acima = Nota { $nota } ({ $faixa }), acima da média nacional de { $media }.
nucleo-saude-resumo-abaixo = Nota { $nota } ({ $faixa }), abaixo da média nacional de { $media }.
nucleo-saude-resumo-igual = Nota { $nota } ({ $faixa }), igual à média nacional de { $media }.
nucleo-saude-resumo-dimensao-fraca = Dimensão mais fraca: { $dimensao }.
nucleo-saude-resumo-nota = Nota { $nota } ({ $faixa }).
nucleo-saude-resumo-ponto-fraco = Ponto mais fraco: { $indicador }.
nucleo-saude-resumo-tudo-na-meta = Todos os indicadores na meta.
nucleo-saude-insuficiente-meses = Dados insuficientes: são precisos ao menos { $minimo } meses completos de lançamentos (há { $n }).
nucleo-saude-insuficiente-indicadores = Dados insuficientes: poucos indicadores puderam ser medidos.
nucleo-saude-aviso-isf = Estimativa a partir dos seus dados (lançamentos, parcelas, faturas e saldos), no espírito do I-SFB da Febraban e do Banco Central; o índice oficial vem de um questionário.
nucleo-saude-aviso-classicos = Indicadores clássicos de finanças pessoais calculados com os seus dados; metas usuais da educação financeira.
nucleo-saude-aviso-pouco-historico = Com menos de { $minimo } meses de histórico, os indicadores ficam sem dados em vez de penalizar.
nucleo-saude-aviso-sem-cambio = Sem câmbio para { $moedas }: esses valores ficaram fora das contas.
nucleo-saude-aviso-contas-desatualizadas = Nota estimada: { $contas } sem lançamentos recentes. O saldo pode estar incompleto, e o negativo delas fica para conferir em vez de contar como cheque especial.
nucleo-saude-referencia = Média nacional do I-SFB em 2024
nucleo-saude-referencia-curta = Média nacional
nucleo-saude-credito-suspeito = possível cheque especial ou importação incompleta em { $contas } (30 dias ou mais no vermelho sem o limite cadastrado): cadastre o cheque especial da conta em 03 · Contas ou importe o extrato completo
