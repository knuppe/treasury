# Planejamento: quanto posso gastar hoje, previsão de saldo, plano de quitação, assinaturas e
# cenários "e se" (docs/i18n-design.md). Explicações e notas das simulações vêm do núcleo.
# pt-BR é o idioma de origem.

## Rótulos repetidos

planejamento-n-meses = { $n } meses
planejamento-n-anos = { $n } anos
planejamento-data = Data
planejamento-nome = Nome
planejamento-valor = Valor
planejamento-por-mes = Por mês
planejamento-assinaturas-por-mes-valor = { $valor } por mês
planejamento-mais-50-anos = mais de 50 anos
planejamento-tirar = Tirar { $nome }

## Quanto posso gastar hoje

planejamento-hoje-titulo = Quanto posso gastar hoje
planejamento-hoje-estourado = Neste mês o que entrou já está todo comprometido. Segure os gastos que puder esperar até o próximo salário.
planejamento-hoje-sem-renda = Anote a renda do mês (o salário, por exemplo) e o app calcula quanto dá para gastar por dia sem apertar as contas.
planejamento-hoje-usado = O valor de hoje já foi usado. Amanhã ele volta, recalculado com o que sobrar.
planejamento-hoje-apertado = Está apertado: vale priorizar só o essencial até o fim do mês.
planejamento-hoje-ok = Dentro desse valor, o mês fecha sem comprometer as contas e as metas.
planejamento-hoje-por-dia =
    { $n ->
        [1] { $valor } por dia até { $data } ({ $n } dia)
       *[other] { $valor } por dia até { $data } ({ $n } dias)
    }
planejamento-hoje-gasto = Já gasto hoje: { $valor }
planejamento-hoje-esconder = Esconder a conta
planejamento-hoje-como = Como calculei
planejamento-hoje-receitas = Receitas do mês (recebidas e previstas)
planejamento-hoje-gastos = Gastos até ontem
planejamento-hoje-contas = Contas que ainda vencem no mês
planejamento-hoje-metas = O que falta guardar para as metas
planejamento-hoje-sobra = Sobra no começo de hoje
planejamento-hoje-amanha-titulo = Por dia a partir de amanhã
planejamento-hoje-amanha =
    { $n ->
        [1] { $valor } por dia a partir de amanhã ({ $n } dia)
       *[other] { $valor } por dia a partir de amanhã ({ $n } dias)
    }
planejamento-hoje-gasto-linha = Gasto hoje
planejamento-hoje-sobra-amanha = Sobra para amanhã em diante
planejamento-hoje-dividido-amanha =
    { $n ->
        [1] Dividido por { $n } dia, a partir de amanhã.
       *[other] Dividido por { $n } dias, a partir de amanhã.
    }
planejamento-hoje-dividido =
    { $n ->
        [1] Dividido por { $n } dia, contando hoje.
       *[other] Dividido por { $n } dias, contando hoje.
    }

## Previsão de saldo

planejamento-origem-previsto = Previsto
planejamento-origem-recorrencia = Recorrência
planejamento-origem-fatura = Fatura
planejamento-origem-parcela = Parcela
planejamento-origem-parcela-detectada = Parcela detectada (estimativa)
planejamento-origem-sazonal = gasto anual (estimativa)
planejamento-previsao-titulo = Previsão de saldo
planejamento-previsao-horizonte = Horizonte da previsão
planejamento-previsao-total-hoje = Total hoje
planejamento-previsao-em = Em { $data }
planejamento-previsao-ponto-baixo = Ponto mais baixo
planejamento-previsao-valor-em = { $valor } em { $data }
planejamento-previsao-negativa = { $conta } fica negativa em { $data } e chega a { $minimo } em { $quando }.
planejamento-previsao-ja-negativa = { $conta } já está negativa hoje e chega a { $minimo } em { $quando }.
planejamento-previsao-nenhuma-negativa = Nenhuma conta fica negativa até { $data }.
planejamento-previsao-por-conta = Saldo previsto por conta
planejamento-previsao-fonte = Fonte: saldo de hoje + previstos, recorrências, faturas em aberto (pagas pela conta de pagamento do cartão) e parcelas de empréstimos
planejamento-previsao-grafico = Saldo previsto de { $de } a { $ate }
planejamento-previsao-eventos = O que mexe no saldo
planejamento-previsao-conta = Conta
planejamento-previsao-descricao = Descrição
planejamento-previsao-origem = Origem
planejamento-previsao-total = Total

## Plano de quitação

planejamento-quitacao-minimo = Só o mínimo
planejamento-quitacao-bola-neve = Bola de neve
planejamento-quitacao-avalanche = Avalanche
planejamento-quitacao-minimo-dica = Só as parcelas, sem extra.
planejamento-quitacao-bola-neve-dica = Extra na menor dívida primeiro: vitórias rápidas.
planejamento-quitacao-avalanche-dica = Extra na maior taxa primeiro: menos juros.
planejamento-quitacao-menos-juros = menos juros
planejamento-quitacao-livre = Livre das dívidas
planejamento-quitacao-meses = Meses
planejamento-quitacao-juros-total = Juros no total
planejamento-quitacao-sem-fim = sem fim
planejamento-quitacao-juros-a-menos = Juros a menos
planejamento-quitacao-meses-a-menos = Meses a menos
planejamento-quitacao-titulo = Plano de quitação
planejamento-quitacao-intro = Quanto sobra por mês para pagar além das parcelas? Quitada uma dívida, a parcela dela também vai para a próxima. Inclua dívidas que não estão no app, como cheque especial ou rotativo do cartão.
planejamento-quitacao-extra = Extra por mês
planejamento-quitacao-fora-app = Dívida fora do app
planejamento-quitacao-exemplo = Cheque especial
planejamento-quitacao-saldo-devedor = Saldo devedor
planejamento-quitacao-juros-ano = Juros ao ano
planejamento-quitacao-minimo-mes = Mínimo por mês
planejamento-quitacao-tirar-esta = Tirar esta dívida
planejamento-quitacao-grafico = Saldo devedor somado, mês a mês
planejamento-quitacao-fonte =
    { $n ->
        [one] Fonte: { $n } dívida; parcelas mínimas + { $extra } de extra por mês
       *[other] Fonte: { $n } dívidas; parcelas mínimas + { $extra } de extra por mês
    }
planejamento-quitacao-grafico-aria = Saldo devedor por estratégia
planejamento-quitacao-dividas = Dívidas na simulação
planejamento-quitacao-divida = Dívida
planejamento-quitacao-saldo = Saldo
planejamento-quitacao-juros-aa = Juros a.a.
planejamento-quitacao-taxa = { NUMBER($taxa, maximumFractionDigits: 2) }%
planejamento-quitacao-minimo-col = Mínimo
planejamento-quitacao-ordem = Ordem de quitação
planejamento-quitacao-estrategia = Estratégia da ordem
planejamento-quitacao-ordem-de = Ordem de quitação: { $estrategia }
planejamento-quitacao-quitada-em = Quitada em
planejamento-quitacao-juros-pagos = Juros pagos
planejamento-quitacao-aviso = Simulação com taxa fixa e parcela constante; não é recomendação financeira.

## Assinaturas e cobranças recorrentes

planejamento-assinaturas-em-dia = Em dia
planejamento-assinaturas-nova = Nova
planejamento-assinaturas-nao-veio = Não veio
planejamento-assinaturas-recorrencia-criada = Recorrência criada
planejamento-assinaturas-recorrencia-criada-descricao = { $nome }: as próximas cobranças entram como previstas.
planejamento-assinaturas-criar-recorrencia = Criar recorrência
planejamento-assinaturas-recorrencia-ok = recorrência ok
planejamento-assinaturas-nao-e = Não é assinatura
planejamento-assinaturas-assinatura = Assinatura
planejamento-assinaturas-frequencia = Frequência
planejamento-assinaturas-mensal = Mensal
planejamento-assinaturas-anual = Anual
planejamento-assinaturas-desde-anterior = { $pct } desde a anterior
planejamento-assinaturas-proxima = Próxima
planejamento-assinaturas-situacao = Situação
planejamento-assinaturas-vazio = Nenhuma assinatura
planejamento-assinaturas-vazio-titulo = Nada se repete todo mês nos lançamentos ainda
planejamento-assinaturas-vazio-descricao = Assinaturas aparecem aqui sozinhas depois de três cobranças mensais parecidas no mesmo estabelecimento (ou duas anuais). Importe os extratos e faturas para encontrar as suas.
planejamento-assinaturas-titulo = Assinaturas e cobranças recorrentes
planejamento-parcelas-titulo = Parcelas em andamento
planejamento-parcelas-vazio-titulo = Nenhuma parcela em andamento
planejamento-parcelas-vazio = Quando a descrição de um lançamento diz que ele é uma parcela ("Parcela 3/10", "PARC 02/12", "PREST.EMPREST 003/005"), o app mostra aqui quanto falta e quando termina.
planejamento-parcelas-descricao = Compra ou contrato
planejamento-parcelas-parcela = Parcela
planejamento-parcelas-valor = Valor da parcela
planejamento-parcelas-falta = Falta pagar
planejamento-parcelas-termina = Termina em
planejamento-parcelas-total = Falta no total
planejamento-parcelas-proximo-mes = No próximo mês
planejamento-parcelas-ignorar = Não é parcela
planejamento-parcelas-ignorada = Série tirada das parcelas
planejamento-parcelas-nao-apareceu = a seguinte não apareceu
planejamento-parcelas-tipo-card = Compra no cartão
planejamento-parcelas-tipo-loan = Empréstimo
planejamento-parcelas-tipo-bill = Parcelamento da fatura
planejamento-parcelas-tipo-account = Parcela na conta
planejamento-parcelas-nota = Estimativa a partir da descrição dos lançamentos; entra na previsão de saldo e nos compromissos do cartão. Compras parceladas lançadas no Treasury e contratos de Dívidas ficam de fora para não contar duas vezes.
planejamento-assinaturas-por-ano = Por ano
planejamento-assinaturas-detectadas = Detectadas
planejamento-assinaturas-subiu = { $nome } subiu { $pct }: de { $antes } para { $agora } em { $data }.
planejamento-assinaturas-cobranca-nova = Cobrança nova: { $nome }, { $valor } desde { $data }. Se não reconhece, confira com o banco.
planejamento-assinaturas-atrasada = { $nome } era esperada em { $data } e não veio: foi cancelada?
planejamento-assinaturas-tabela = Assinaturas detectadas
planejamento-assinaturas-dispensadas = { $n } detecção(ões) dispensada(s)
planejamento-assinaturas-trazer = Trazer de volta
planejamento-assinaturas-fonte = Fonte: despesas dos últimos 2 anos, sem compras parceladas nem transferências. O total por mês não inclui as que não vieram.

## Cenários "e se"

planejamento-cenarios-imovel = Comprar imóvel
planejamento-cenarios-filho = Ter um filho
planejamento-cenarios-pausa = Pausar a carreira
planejamento-cenarios-renda = Mudar de renda
planejamento-cenarios-unico = Gasto ou entrada única
planejamento-cenarios-projeto = De um projeto
planejamento-cenarios-evento-unico = { $valor } em { $data }
planejamento-cenarios-evento-mensal =
    { $meses ->
        [0] { $valor } por mês desde { $inicio }
       *[other] { $valor } por mês desde { $inicio } por { $meses } meses
    }
planejamento-cenarios-evento-renda =
    { $meses ->
        [0] renda em { $pct }% desde { $inicio }
       *[other] renda em { $pct }% desde { $inicio } por { $meses } meses
    }
planejamento-cenarios-evento-bem =
    { $taxa ->
        [0] bem de { $valor } em { $data }
       *[other] bem de { $valor } em { $data }, { $taxa }% a.a. real
    }
planejamento-cenarios-evento-financiamento = financiamento de { $valor } em { $meses } meses a { $taxa }% a.a. real
planejamento-cenarios-compra-imovel = Compra do imóvel
planejamento-cenarios-imovel-bem = Imóvel
planejamento-cenarios-financiamento = Financiamento do imóvel
planejamento-cenarios-filho-evento = Filho
planejamento-cenarios-pausa-evento = Pausa na carreira
planejamento-cenarios-renda-em = Renda em { $pct }%
planejamento-cenarios-mudanca-renda = Mudança de renda
planejamento-cenarios-gasto-unico = Gasto único
planejamento-cenarios-sem-orcamento = { $projeto }: sem orçamento a gastar
planejamento-cenarios-a-partir = A partir de
planejamento-cenarios-preco-imovel = Preço do imóvel
planejamento-cenarios-entrada = Entrada
planejamento-cenarios-prazo = Prazo
planejamento-cenarios-anos = anos
planejamento-cenarios-juros-reais = Juros reais
planejamento-cenarios-juros-reais-dica = Acima da inflação (ex.: TR + 9% ≈ 5% real).
planejamento-cenarios-ao-ano = % a.a.
planejamento-cenarios-data-compra = Data da compra
planejamento-cenarios-custo-mes = Custo por mês
planejamento-cenarios-renda-pausa = Renda durante a pausa
planejamento-cenarios-renda-pausa-dica = 0% = sem renda; 50% = meio período.
planejamento-cenarios-quanto-tempo = Por quanto tempo
planejamento-cenarios-meses = meses
planejamento-cenarios-novo-emprego = Novo emprego
planejamento-cenarios-exemplo-unico = Viagem, herança…
planejamento-cenarios-diferenca-mes = Diferença por mês
planejamento-cenarios-negativo-dica = Negativo para saída (ex.: -5.000,00).
planejamento-cenarios-projeto-campo = Projeto
planejamento-cenarios-projeto-dica = O orçamento que falta gastar sai na data de fim.
planejamento-cenarios-escolha = Escolha
planejamento-cenarios-nenhum-projeto = Nenhum projeto
planejamento-cenarios-sem-data-fim = Se não tiver data de fim
planejamento-cenarios-adicionar = Adicionar ao cenário
planejamento-cenarios-patrimonio-base = Patrimônio (base)
planejamento-cenarios-patrimonio-cenario = Patrimônio (cenário)
planejamento-cenarios-liquido-cenario = Contas e investimentos (cenário)
planejamento-cenarios-base = Base
planejamento-cenarios-cenario = Cenário
planejamento-cenarios-patrimonio-em =
    { $anos ->
        [one] Patrimônio em { $anos } ano
       *[other] Patrimônio em { $anos } anos
    }
planejamento-cenarios-base-valor = base { $valor } · { $diferenca }
planejamento-cenarios-liquido = Contas e investimentos
planejamento-cenarios-acabam = acabam em { $mes } · { $diferenca }
planejamento-cenarios-ja-negativos = já estão negativos hoje · { $diferenca }
planejamento-cenarios-nao-acabam = não acabam no período · { $diferenca }
planejamento-cenarios-aposentadoria = Aposentadoria
# O trecho entre ** sai menor, em cinza.
planejamento-cenarios-chance = { $pct }% **de chance**
planejamento-cenarios-aposentadoria-base = base { $pct }% · aos { $idade }: { $valor }
planejamento-cenarios-sem-idade = Preencha a idade em Orçamento › Aposentadoria para ver o efeito.
planejamento-cenarios-grafico-patrimonio = Patrimônio: cenário × base
planejamento-cenarios-fonte-patrimonio = Fonte: média dos últimos 12 meses, saldo e carteira de hoje, eventos do cenário · valores reais
planejamento-cenarios-grafico-patrimonio-aria = Patrimônio no cenário e na base
planejamento-cenarios-tabela-patrimonio = Patrimônio por mês
planejamento-cenarios-mes = Mês
planejamento-cenarios-grafico-saldo = Próximos 12 meses: saldo das contas
planejamento-cenarios-saldo-negativo = O saldo fica negativo em { $data } no cenário
planejamento-cenarios-saldo-positivo = O saldo das contas não fica negativo no cenário
planejamento-cenarios-grafico-saldo-aria = Previsão de saldo no cenário e na base
planejamento-cenarios-tabela-saldo = Previsão de saldo com o cenário
planejamento-cenarios-aviso = Simulação, não é recomendação financeira.
planejamento-cenarios-titulo = Cenários "e se"
planejamento-cenarios-intro = Monte um cenário com eventos de vida e compare com o que acontece se nada mudar: o patrimônio ao longo dos anos, o saldo das contas nos próximos 12 meses e a chance de a aposentadoria dar certo.
planejamento-cenarios-salvos = Salvos:
planejamento-cenarios-apagar = Apagar { $nome }
planejamento-cenarios-evento = Evento de vida
planejamento-cenarios-horizonte = Horizonte
planejamento-cenarios-nome = Nome do cenário
planejamento-cenarios-nome-exemplo = Apartamento em 2027
planejamento-cenarios-salvo = Cenário salvo
planejamento-cenarios-limpar = Limpar
planejamento-assinaturas-tipo = Tipo
planejamento-assinaturas-conta-fixa = Conta fixa
planejamento-assinaturas-tipo-assinatura = Assinatura
planejamento-assinaturas-contas-fixas-mes = Contas fixas por mês
