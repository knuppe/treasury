# Variante pt-PT (planeamento: gastar hoje, previsão de saldo, liquidação de dívidas,
# subscrições e cenários): o que faltar cai para pt-BR. Revisão: rascunho por IA.

## Rótulos repetidos

planejamento-tirar = Remover { $nome }

## Quanto posso gastar hoje

planejamento-hoje-estourado = Neste mês, o que entrou já está todo comprometido. Adie os gastos que puderem esperar até ao próximo salário.
planejamento-hoje-usado = O valor de hoje já foi usado. Amanhã volta, recalculado com o que sobrar.
planejamento-hoje-apertado = Está apertado: convém dar prioridade só ao essencial até ao fim do mês.
planejamento-hoje-esconder = Ocultar o cálculo
planejamento-hoje-dividido =
    { $n ->
        [1] Dividido por { $n } dia, incluindo hoje.
       *[other] Dividido por { $n } dias, incluindo hoje.
    }

## Previsão de saldo

planejamento-origem-fatura = Extrato do cartão
planejamento-origem-parcela = Prestação
planejamento-previsao-fonte = Fonte: saldo de hoje + previstos, recorrências, extratos de cartão em aberto (pagos pela conta de pagamento do cartão) e prestações de empréstimos

## Plano de liquidação

planejamento-quitacao-minimo-dica = Só as prestações, sem extra.
planejamento-quitacao-titulo = Plano de liquidação
planejamento-quitacao-intro = Quanto sobra por mês para pagar além das prestações? Liquidada uma dívida, a prestação dela também vai para a próxima. Inclua dívidas que não estão na aplicação, como o descoberto autorizado ou o crédito rotativo do cartão.
planejamento-quitacao-fora-app = Dívida fora da aplicação
planejamento-quitacao-exemplo = Descoberto autorizado
planejamento-quitacao-tirar-esta = Remover esta dívida
planejamento-quitacao-fonte = Fonte: { $n } dívida(s); prestações mínimas + { $extra } de extra por mês
planejamento-quitacao-juros-aa = Juros/ano
planejamento-quitacao-ordem = Ordem de liquidação
planejamento-quitacao-ordem-de = Ordem de liquidação: { $estrategia }
planejamento-quitacao-quitada-em = Liquidada em
planejamento-quitacao-aviso = Simulação com taxa fixa e prestação constante; não é uma recomendação financeira.

## Subscrições e cobranças recorrentes

planejamento-assinaturas-nao-e = Não é uma subscrição
planejamento-assinaturas-assinatura = Subscrição
planejamento-assinaturas-vazio = Nenhuma subscrição
planejamento-assinaturas-vazio-titulo = Ainda nada se repete todos os meses nos lançamentos
planejamento-assinaturas-vazio-descricao = As subscrições aparecem aqui sozinhas depois de três cobranças mensais parecidas no mesmo estabelecimento (ou duas anuais). Importe os extratos de conta e de cartão para encontrar as suas.
planejamento-assinaturas-titulo = Subscrições e cobranças recorrentes
planejamento-assinaturas-detectadas = Detetadas
planejamento-assinaturas-cobranca-nova = Cobrança nova: { $nome }, { $valor } desde { $data }. Se não a reconhece, confirme com o banco.
planejamento-assinaturas-tabela = Subscrições detetadas
planejamento-assinaturas-dispensadas = { $n } deteção(ões) dispensada(s)
planejamento-assinaturas-fonte = Fonte: despesas dos últimos 2 anos, sem compras em prestações nem transferências. O total por mês não inclui as que não vieram.

## Cenários "e se"

planejamento-cenarios-renda = Mudar de rendimento
planejamento-cenarios-evento-renda =
    { $meses ->
        [0] rendimento a { $pct }% desde { $inicio }
       *[other] rendimento a { $pct }% desde { $inicio } durante { $meses } meses
    }
planejamento-cenarios-evento-mensal =
    { $meses ->
        [0] { $valor } por mês desde { $inicio }
       *[other] { $valor } por mês desde { $inicio } durante { $meses } meses
    }
planejamento-cenarios-evento-bem =
    { $taxa ->
        [0] bem de { $valor } em { $data }
       *[other] bem de { $valor } em { $data }, { $taxa }% ao ano (real)
    }
planejamento-cenarios-evento-financiamento = crédito de { $valor } em { $meses } meses a { $taxa }% ao ano (real)
planejamento-cenarios-financiamento = Crédito habitação
planejamento-cenarios-renda-em = Rendimento a { $pct }%
planejamento-cenarios-mudanca-renda = Mudança de rendimento
planejamento-cenarios-juros-reais-dica = Acima da inflação (ex.: juro de 5% com inflação de 2% ≈ 3% real).
planejamento-cenarios-ao-ano = % ao ano
planejamento-cenarios-renda-pausa = Rendimento durante a pausa
planejamento-cenarios-renda-pausa-dica = 0% = sem rendimento; 50% = tempo parcial.
planejamento-cenarios-negativo-dica = Negativo para saída (ex.: -5000,00).
planejamento-cenarios-patrimonio-base = Património (base)
planejamento-cenarios-patrimonio-cenario = Património (cenário)
planejamento-cenarios-patrimonio-em = Património em { $anos } ano(s)
planejamento-cenarios-aposentadoria = Reforma
# O trecho entre ** sai mais pequeno, a cinzento.
planejamento-cenarios-chance = { $pct }% **de probabilidade**
planejamento-cenarios-sem-idade = Preencha a idade em Orçamento › Reforma para ver o efeito.
planejamento-cenarios-grafico-patrimonio = Património: cenário × base
planejamento-cenarios-grafico-patrimonio-aria = Património no cenário e na base
planejamento-cenarios-tabela-patrimonio = Património por mês
planejamento-cenarios-aviso = Simulação, não é uma recomendação financeira.
planejamento-cenarios-intro = Monte um cenário com eventos de vida e compare-o com o que acontece se nada mudar: o património ao longo dos anos, o saldo das contas nos próximos 12 meses e a probabilidade de a reforma correr bem.
planejamento-cenarios-salvos = Guardados:
planejamento-cenarios-apagar = Eliminar { $nome }
planejamento-cenarios-salvo = Cenário guardado
