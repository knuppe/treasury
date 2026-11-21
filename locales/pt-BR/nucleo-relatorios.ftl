# Relatórios (núcleo, `reports`): rótulos dos grupos e do fluxo do dinheiro, as fórmulas e as
# fontes do "Explicar" (`explain_value`), os próximos vencimentos e os erros. Montados na hora, no
# idioma da instalação. Erros em minúsculas, como frase de log (a interface põe a inicial
# maiúscula). pt-BR é o idioma de origem.

## Períodos e base de datas

nucleo-relatorios-erro-mes-inicial-posterior = o mês inicial ({ $de }) é posterior ao final ({ $ate })
nucleo-relatorios-erro-periodo-longo = período longo demais (máximo de { $maximo } meses)
nucleo-relatorios-erro-data-inicial-posterior = a data inicial ({ $de }) é posterior à final ({ $ate })
nucleo-relatorios-erro-calendario-longo = período longo demais para o calendário (máximo de 10 anos)
nucleo-relatorios-base-competencia = competência
nucleo-relatorios-base-caixa = caixa

## Grupos e vencimentos

nucleo-relatorios-sem-categoria = Sem categoria
nucleo-relatorios-sem-membro = Sem membro
nucleo-relatorios-outras = Outras
nucleo-relatorios-sem-descricao = (sem descrição)
# Próximos vencimentos: a fatura de um cartão ("Fatura Nubank (out/2026)").
nucleo-relatorios-fatura = Fatura { $cartao } ({ $mes })

## Para onde foi o dinheiro (diagrama de fluxo e os itens de cada nó)

nucleo-relatorios-fluxo-orcamento = Orçamento
nucleo-relatorios-fluxo-receitas-sem-categoria = Receitas sem categoria
nucleo-relatorios-fluxo-resgates = Resgates
nucleo-relatorios-fluxo-deficit = Déficit
nucleo-relatorios-fluxo-dividas = Pagamento de dívidas
nucleo-relatorios-fluxo-investimentos = Investimentos
nucleo-relatorios-fluxo-sobra = Sobra
nucleo-relatorios-fluxo-sem-subcategoria = { $categoria } (sem subcategoria)
nucleo-relatorios-fluxo-receita-de = A receita de { $categoria }
nucleo-relatorios-fluxo-pagamento-de = O pagamento de { $contrato }
nucleo-relatorios-fluxo-entradas = Entradas
nucleo-relatorios-fluxo-saidas = Saídas
nucleo-relatorios-fluxo-sem-cambio = Sem câmbio de { $de } para { $para } na data: fica fora do total.
nucleo-relatorios-fluxo-grupo-negativo = { $grupo } somou { $valor } no período (estornos maiores que os lançamentos); o diagrama deixa de fora grupos sem saldo positivo.
nucleo-relatorios-fluxo-sem-sobra-deficit = Não houve sobra no período: o que saiu passou do que entrou em { $valor }.
nucleo-relatorios-fluxo-sem-sobra = Não houve sobra no período.
nucleo-relatorios-fluxo-sem-deficit-sobra = Não houve déficit no período: sobraram { $valor }.
nucleo-relatorios-fluxo-sem-deficit = Não houve déficit no período.
nucleo-relatorios-erro-no-desconhecido = nó do fluxo desconhecido: { $no }
nucleo-relatorios-erro-sem-nos = informe ao menos um nó do fluxo
nucleo-relatorios-erro-nos-sobrepostos = os nós pedidos se sobrepõem (uma categoria e uma subcategoria dela)

## "Explicar" (`explain_value`): fontes de cada número

nucleo-relatorios-conta = conta { $id }
nucleo-relatorios-fonte-cambio = câmbio: 1 { $moeda } = { $taxa } { $alvo }
nucleo-relatorios-fonte-cambio-estimado = câmbio: 1 { $moeda } = { $taxa } { $alvo } (estimado)
nucleo-relatorios-fonte-saldo-inicial = saldo inicial de { $conta }
# Uma conta (ou lançamento) em outra moeda: "Conta EUA: USD 100,00 × 5,43 = R$ 543,00".
nucleo-relatorios-fonte-conta-convertida = { $conta }: { $valor } × { $taxa } = { $convertido }
nucleo-relatorios-fonte-conta-sem-cambio = { $conta }: { $valor } (sem câmbio, fora do total)
nucleo-relatorios-fonte-convertida = { $rotulo } × { $taxa } = { $convertido }
nucleo-relatorios-fonte-carteira = Carteira de investimentos (valor de mercado ou na curva; veja a carteira)
nucleo-relatorios-fonte-estimativa = estimativa
nucleo-relatorios-fonte-avaliacao = avaliação
nucleo-relatorios-fonte-dividas = Empréstimos e financiamentos (saldo devedor pelo cronograma)
nucleo-relatorios-fonte-partes =
    { $n ->
        [1] { $rotulo } (parte da divisão)
       *[other] { $rotulo } ({ $n } partes da divisão)
    }
nucleo-relatorios-fonte-fatura = { $rotulo } (compra em { $compra }, fatura com vencimento em { $vencimento })
nucleo-relatorios-fonte-lancamento-sem-cambio = { $rotulo }: { $valor } sem câmbio, fora do total

## "Explicar": fórmulas

nucleo-relatorios-n-efetivados =
    { $n ->
        [1] { $n } lançamento efetivado
       *[other] { $n } lançamentos efetivados
    }
nucleo-relatorios-n-despesas =
    { $n ->
        [1] { $n } lançamento de despesa efetivado
       *[other] { $n } lançamentos de despesa efetivados
    }
nucleo-relatorios-n-receitas =
    { $n ->
        [1] { $n } lançamento de receita efetivado
       *[other] { $n } lançamentos de receita efetivados
    }
# Vai no fim de uma fórmula (começa com o ponto e vírgula).
nucleo-relatorios-formula-sem-cambio = ; sem câmbio para { $moedas } (fora do total)
nucleo-relatorios-formula-saldo = saldo inicial de { $inicial } + { $lancamentos } até { $data } ({ $soma }) = { $valor }
nucleo-relatorios-formula-saldo-abertura = { $lancamentos } até { $data } = { $valor } (a conta foi aberta em { $abertura }; o saldo inicial de { $inicial } só conta a partir dessa data)
nucleo-relatorios-formula-patrimonio =
    ativos de { $ativos } − passivos de { $passivos } = { $valor }, somando o saldo de { $n ->
        [1] { $n } conta
       *[other] { $n } contas
    } em { $data } (saldo inicial + lançamentos efetivados até a data; saldos negativos, como a dívida dos cartões, são passivos), a carteira, os bens físicos (avaliação ou estimativa) e as dívidas{ $sem_cambio }
# Filtros da fórmula de gastos e receitas, depois do período ("… (competência) em Moradia e
# subcategorias de Ana: …").
nucleo-relatorios-filtro-categoria = em { $categoria } e subcategorias
nucleo-relatorios-filtro-membro = de { $membro }
nucleo-relatorios-regra-despesa = despesa = −soma dos valores (estornos reduzem o total)
nucleo-relatorios-regra-receita = receita = soma dos valores
nucleo-relatorios-formula-convertidos = ; valores em outra moeda convertidos para { $moeda } pelo câmbio da data do lançamento
nucleo-relatorios-formula-fluxo = { $lancamentos } de { $de } a { $ate } ({ $base }){ $filtros }: { $regra } = { $valor }; transferências, pagamentos de fatura e aportes não entram; lançamentos divididos contam pelas partes{ $convertidos }{ $sem_cambio }
nucleo-relatorios-formula-poupanca = taxa de poupança de { $mes } ({ $base }) = (receitas de { $receitas } − despesas de { $despesas }) ÷ receitas de { $receitas } = { $taxa }; valor = receitas − despesas = { $valor }, de { $lancamentos }{ $sem_cambio }
nucleo-relatorios-formula-poupanca-sem-receitas = sem receitas em { $mes }, a taxa de poupança não se aplica; receitas − despesas = { $valor }, de { $lancamentos }{ $sem_cambio }

## Para onde vai a renda (grupos de gasto das categorias; nomes dos grupos em grupos.ftl)

# O que entra no Futuro sem categoria: aportes em investimentos e transferências para as contas
# das metas.
nucleo-relatorios-grupos-aportes = Aportes
nucleo-relatorios-grupos-metas = Transferências para metas
# Seção do relatório do mês em PDF.
nucleo-relatorios-grupos-pdf-titulo = Para onde foi a renda
nucleo-relatorios-grupos-pdf-dica = Despesas do mês pelo grupo de cada categoria; o futuro inclui os aportes e as metas.
nucleo-relatorios-grupos-pdf-parte = { $parte } da renda
nucleo-relatorios-grupos-pdf-sem-renda = sem receitas no mês
nucleo-relatorios-grupos-pdf-meta = meta { $meta }
nucleo-relatorios-grupos-pdf-sem-meta = sem meta
