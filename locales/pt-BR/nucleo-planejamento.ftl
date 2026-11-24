# Planejamento (núcleo: `forecast`, `scenarios`, `debt`, `pension`, `finance`): previsão de saldo e
# o "gastar hoje", cenários, dívidas (cronograma, quitação, taxas de mercado, extratos e
# portabilidade), previdência e os erros. Montados na hora, no idioma da instalação. Erros em
# minúsculas, como frase de log (a interface põe a inicial maiúscula). pt-BR é o idioma de origem.

## Previsão de saldo e quanto dá para gastar hoje (`forecast`)

nucleo-planejamento-conta-ou-arquivada = conta (ou arquivada)
nucleo-planejamento-fatura = Fatura { $cartao }
nucleo-planejamento-parcela = Parcela { $contrato }
nucleo-planejamento-sazonal = { $descricao } (estimativa pelo valor de { $ano })
nucleo-planejamento-gastar-hoje =
    Receitas do mês { $receitas } − gastos até ontem { $gastos } − contas que ainda vencem { $contas } − falta guardar { $guardar } = { $disponivel } para { $dias ->
        [one] { $dias } dia
       *[other] { $dias } dias
    }: { $por_dia } por dia.
nucleo-planejamento-gastar-amanha = Hoje já foram { $hoje }; a partir de amanhã, { $amanha } por dia.
nucleo-planejamento-gastar-sem-renda = Ainda não há receita recebida nem prevista neste mês. Sem ela não dá para calcular quanto sobra por dia.

## Cenários "e se" (`scenarios`)

nucleo-planejamento-cenario-nome = Cenário
nucleo-planejamento-cenario-explicacao =
    { $anos ->
        [one] Em { $anos } ano, o patrimônio fica { $cenario } no cenário contra { $base } na base ({ $diferenca } de diferença; { $liquido } em contas e investimentos).
       *[other] Em { $anos } anos, o patrimônio fica { $cenario } no cenário contra { $base } na base ({ $diferenca } de diferença; { $liquido } em contas e investimentos).
    }
nucleo-planejamento-cenario-dinheiro-acaba = O dinheiro disponível acaba em { $mes }.
nucleo-planejamento-cenario-saldo-negativo = Nos próximos 12 meses, o saldo das contas fica negativo em { $data }.
nucleo-planejamento-cenario-aposentadoria = Aposentadoria: chance de sucesso de { $base } para { $cenario }.
nucleo-planejamento-cenario-nota-premissas = Valores reais de hoje; renda de { $renda } e gasto de { $gasto } por mês (média dos últimos 12 meses), retorno real de { $retorno } a.a.
nucleo-planejamento-cenario-nota-bens = Bens e dívidas que já existem ficam no valor de hoje; as parcelas atuais já estão no gasto médio.
nucleo-planejamento-cenario-nota-sem-idade = Sem idade no plano de aposentadoria (Orçamento › Aposentadoria): o efeito na aposentadoria fica de fora.
nucleo-planejamento-cenario-erro-evento-sem-nome = cada evento precisa de um nome
nucleo-planejamento-cenario-erro-renda = { $evento }: renda de 0% a 1000%
nucleo-planejamento-cenario-erro-financiamento = { $evento }: financiamento com valor, prazo (1 a 600 meses) e taxa válidos
nucleo-planejamento-cenario-erro-bem = { $evento }: valor do bem positivo
nucleo-planejamento-cenario-erro-so-interface = cenários salvos só pela interface do app
nucleo-planejamento-cenario-erro-sem-nome = dê um nome ao cenário

## Previdência: plano, resgate e extrato (`pension`). Os textos das linhas do extrato
## ("Contribuição do participante"…) ficam como no documento.

nucleo-planejamento-previdencia-plano = plano
nucleo-planejamento-previdencia-taxa-administracao = taxa de administração
nucleo-planejamento-previdencia-taxa-carregamento = taxa de carregamento
nucleo-planejamento-previdencia-erro-taxa-invalida = { $taxa } inválida: { $valor }
nucleo-planejamento-previdencia-erro-taxa-faixa = { $taxa } deve ficar entre 0 e 100%
nucleo-planejamento-previdencia-erro-classe = o plano de previdência é para ativos da classe Previdência
nucleo-planejamento-previdencia-erro-resgate-positivo = informe um valor de resgate positivo
nucleo-planejamento-previdencia-erro-resgate-saldo = o resgate passa do saldo do plano ({ $saldo })
nucleo-planejamento-previdencia-nota-vgbl-regressivo = VGBL regressivo: IR definitivo só sobre o rendimento, pela idade de cada aporte (os mais antigos saem primeiro).
nucleo-planejamento-previdencia-nota-pgbl-regressivo = PGBL regressivo: IR definitivo sobre o valor todo, pela idade de cada aporte (os mais antigos saem primeiro).
nucleo-planejamento-previdencia-nota-vgbl-progressivo = VGBL progressivo: 15% do rendimento retidos na fonte; o IR final sai pela tabela anual na declaração.
nucleo-planejamento-previdencia-nota-efpc-regressivo = Fundo de pensão (EFPC), regressivo: IR definitivo sobre o valor todo, pela idade de cada contribuição (as mais antigas saem primeiro).
nucleo-planejamento-previdencia-nota-efpc-progressivo = Fundo de pensão (EFPC), progressivo: 15% do valor todo retidos na fonte; o IR final sai pela tabela anual na declaração.
nucleo-planejamento-previdencia-nota-pgbl-progressivo = PGBL progressivo: 15% do valor todo retidos na fonte; o IR final sai pela tabela anual na declaração.
nucleo-planejamento-previdencia-aviso-sem-plano = informe o tipo do plano (VGBL ou PGBL) e o regime de tributação para o IR sair certo; por enquanto o cálculo supõe VGBL regressivo
nucleo-planejamento-previdencia-aviso-sem-cota = sem cota recente: o valor de hoje usa a cota do último aporte. Informe o CNPJ do fundo (coleta da CVM) ou a cota atual
nucleo-planejamento-previdencia-motivo-plano = Plano de previdência: { $nome }
nucleo-planejamento-previdencia-motivo-extrato = Extrato de previdência importado
nucleo-planejamento-previdencia-erro-pdf = não consegui ler este PDF: { $erro }
nucleo-planejamento-previdencia-erro-sem-contribuicoes = não reconheci contribuições neste extrato (linhas "MM/AAAA DD/MM/AAAA DD/MM/AAAA histórico valores")
nucleo-planejamento-previdencia-erro-sem-arquivo = envie o PDF do extrato ou o texto dele
nucleo-planejamento-previdencia-erro-extrato-classe = o extrato de previdência vai num ativo da classe Previdência
nucleo-planejamento-previdencia-erro-conta = escolha a conta das cotas da previdência
nucleo-planejamento-previdencia-erro-interno = extrato de previdência: { $erro }
nucleo-planejamento-previdencia-aviso-cotas = a soma das cotas das contribuições ({ $soma }) não bate com o saldo de cotas do extrato ({ $saldo }): confira se alguma linha ficou de fora
nucleo-planejamento-previdencia-aviso-saldo = o saldo pelas cotas ({ $calculado }) difere do saldo do participante no extrato ({ $extrato })
nucleo-planejamento-previdencia-aviso-sem-cnpj = não achei o CNPJ do plano no extrato
# Nome do ativo criado pelo extrato quando ele não traz o nome do plano.
nucleo-planejamento-previdencia-ativo-padrao = Previdência (fundo de pensão)
# Notas gravadas no aporte e na despesa de risco ("Contribuição do participante (competência 03/2026)").
nucleo-planejamento-previdencia-nota-aporte = { $descricao } (competência { $competencia })
nucleo-planejamento-previdencia-nota-risco = Previdência { $plano } (competência { $competencia })

## Preço médio da carteira (`finance::average_cost`)

nucleo-planejamento-erro-compra-positiva = quantidade de compra deve ser positiva
nucleo-planejamento-erro-venda-positiva = quantidade de venda deve ser positiva
nucleo-planejamento-erro-venda-maior = venda maior que a posição (venda a descoberto não é suportada)
nucleo-planejamento-erro-fator-desdobramento = fator de desdobramento/grupamento inválido

## Dívidas: contratos, parcelas, antecipação e portabilidade (`debt`)

nucleo-planejamento-divida-emprestimo = empréstimo { $id }
nucleo-planejamento-divida-contrato = contrato
nucleo-planejamento-divida-parcela = parcela
nucleo-planejamento-divida-categoria = categoria
nucleo-planejamento-divida-parcela-do-mes = parcela do mês do pagamento
# Nomes dos campos nos erros de valor inválido ("taxa inválido: abc").
nucleo-planejamento-divida-campo-taxa = taxa
nucleo-planejamento-divida-campo-taxa-dfi = taxa do DFI
nucleo-planejamento-divida-campo-taxa-mip = taxa do MIP
nucleo-planejamento-divida-campo-percentual-indice = percentual do índice
nucleo-planejamento-divida-campo-cet = CET
nucleo-planejamento-divida-erro-invalido = { $campo } inválido: { $valor }
nucleo-planejamento-divida-erro-taxa-seguro = { $campo }: informe a taxa ao mês em % (ex.: 0,0247)
nucleo-planejamento-divida-erro-sem-nome = dê um nome ao contrato
nucleo-planejamento-divida-erro-valor-positivo = o valor financiado deve ser positivo
nucleo-planejamento-divida-erro-bullet-carencia = no bullet não há carência: a parcela já é só juros
nucleo-planejamento-divida-erro-percentual-indice = percentual do índice fora do intervalo (0 a 1000%)
nucleo-planejamento-divida-erro-defasagem-mensal = defasagem do índice mensal de 0 a 3 meses
nucleo-planejamento-divida-erro-defasagem-diaria = defasagem do índice de 0 a 10 dias úteis
nucleo-planejamento-divida-erro-posfixado-spread = nos pós-fixados a parcela varia com o índice: informe o spread (0 se não houver)
nucleo-planejamento-divida-erro-taxa-ou-parcela = informe a taxa anual ou o valor da parcela para calcular a taxa
nucleo-planejamento-divida-erro-taxa-calculada = taxa calculada inválida
nucleo-planejamento-divida-erro-taxa-intervalo = taxa anual fora do intervalo
nucleo-planejamento-divida-erro-indexador-diferente = a taxa traz um indexador diferente do escolhido no contrato
nucleo-planejamento-divida-erro-taxa-invalida = taxa inválida: { $taxa }
nucleo-planejamento-divida-erro-primeira-parcela = a primeira parcela não pode vencer antes do contrato
nucleo-planejamento-divida-erro-prazo = prazo inválido (a carência deve ser menor que o prazo)
nucleo-planejamento-divida-erro-quantas-antecipar = informe quantas parcelas antecipar
nucleo-planejamento-divida-erro-sem-futuras = não há parcelas futuras em aberto para antecipar
nucleo-planejamento-divida-erro-poucas-futuras =
    { $n ->
        [one] só há { $n } parcela futura em aberto
       *[other] só há { $n } parcelas futuras em aberto
    }
nucleo-planejamento-divida-erro-parcela-seguros = a parcela precisa ser maior que os seguros e taxas mensais
nucleo-planejamento-divida-erro-taxa-nao-fecha = com essa parcela e esse prazo a taxa não fecha: confira os valores (a parcela vezes o prazo precisa passar do valor financiado)
nucleo-planejamento-divida-erro-pagamento-antes = o pagamento é anterior ao primeiro vencimento do contrato
nucleo-planejamento-divida-erro-pagamento-negativo = valores do pagamento não podem ser negativos
nucleo-planejamento-divida-erro-saldo-restante = ainda há saldo devedor: informe quantas parcelas faltam
nucleo-planejamento-divida-erro-amortizacao-positiva = o valor da amortização deve ser positivo
nucleo-planejamento-divida-erro-imovel-negativo = o valor do imóvel não pode ser negativo
nucleo-planejamento-divida-erro-seguro-negativo = o valor do seguro não pode ser negativo
nucleo-planejamento-divida-erro-custo-inicial-negativo = os custos iniciais (IOF, tarifas, seguro) não podem ser negativos
nucleo-planejamento-divida-erro-custo-inicial-maior = os custos iniciais somados precisam ser menores que o valor do contrato
nucleo-planejamento-divida-erro-dfi-sem-imovel = o DFI em percentual é calculado sobre o valor do imóvel: informe o valor do imóvel (ou o DFI em reais)
nucleo-planejamento-divida-erro-categoria-despesa = a categoria dos pagamentos precisa ser de despesa
nucleo-planejamento-divida-erro-categoria-ligada = essa categoria já está ligada ao contrato { $contrato }
nucleo-planejamento-divida-erro-conciliar-sem-conta = defina a conta de débito (ou a categoria dos pagamentos) do contrato para conciliar
nucleo-planejamento-divida-erro-pagamento-saida = o pagamento de uma parcela precisa ser uma saída (valor negativo)
nucleo-planejamento-divida-erro-pagamento-transferencia = o lançamento é uma transferência entre suas contas e não pode pagar uma parcela
nucleo-planejamento-divida-erro-pagamento-previsto = o lançamento ainda está previsto; efetive-o antes de ligar à parcela
nucleo-planejamento-divida-erro-pagamento-moeda = o lançamento está em { $moeda } e o contrato em { $moeda_contrato }
nucleo-planejamento-divida-erro-pagamento-ja-ligado = o lançamento já está ligado à parcela { $numero } de { $contrato }; desligue-o de lá antes
nucleo-planejamento-divida-erro-pagamento-amortizacao = o lançamento é de uma amortização extraordinária de { $contrato }
# Autor e motivo no histórico quando os índices novos recalculam as parcelas.
nucleo-planejamento-divida-autor-indices = índices
nucleo-planejamento-divida-motivo-indices = Parcelas recalculadas com os índices mais recentes
nucleo-planejamento-divida-motivo-extrato = Extrato do contrato importado
nucleo-planejamento-divida-motivo-cadastrado = Contrato cadastrado: { $nome }
nucleo-planejamento-divida-motivo-alterado = Contrato alterado
nucleo-planejamento-divida-motivo-removido = Contrato removido
nucleo-planejamento-divida-motivo-amortizacao = Amortização extraordinária
nucleo-planejamento-divida-motivo-antecipadas = Parcelas antecipadas
nucleo-planejamento-divida-motivo-vencidas-pagas = Parcelas vencidas marcadas como pagas
nucleo-planejamento-divida-motivo-conciliadas = Parcelas conciliadas com os pagamentos
nucleo-planejamento-divida-motivo-pagamento = Pagamento de parcela registrado
nucleo-planejamento-divida-motivo-parcela-paga = Parcela paga
nucleo-planejamento-divida-motivo-parcela-ligada = Parcela { $numero } de { $contrato } ligada ao lançamento
nucleo-planejamento-divida-motivo-pagamento-desfeito = Pagamento de parcela desfeito
# Descrição dos lançamentos criados pela amortização e pela antecipação.
nucleo-planejamento-divida-lancamento-amortizacao = Amortização extraordinária · { $contrato }
nucleo-planejamento-divida-lancamento-antecipacao =
    { $n ->
        [one] Antecipação de { $n } parcela · { $contrato }
       *[other] Antecipação de { $n } parcelas · { $contrato }
    }

## Taxas de juros do mercado (Banco Central do Brasil, `debt::market_rates`)

nucleo-planejamento-mercado-modalidade-cheque-especial = Cheque especial
nucleo-planejamento-mercado-modalidade-cartao-rotativo = Cartão de crédito: rotativo
nucleo-planejamento-mercado-modalidade-cartao-parcelado = Cartão de crédito: parcelado
nucleo-planejamento-mercado-modalidade-credito-pessoal = Crédito pessoal
nucleo-planejamento-mercado-modalidade-consignado-privado = Consignado (empresa privada)
nucleo-planejamento-mercado-modalidade-consignado-publico = Consignado (servidor público)
nucleo-planejamento-mercado-modalidade-consignado-inss = Consignado INSS
nucleo-planejamento-mercado-modalidade-veiculo = Financiamento de veículo
nucleo-planejamento-mercado-modalidade-outros-bens = Financiamento de outros bens
nucleo-planejamento-mercado-modalidade-imobiliario-pre = Financiamento imobiliário (pré)
nucleo-planejamento-mercado-modalidade-imobiliario-tr = Financiamento imobiliário (TR + taxa)
nucleo-planejamento-mercado-modalidade-imobiliario-ipca = Financiamento imobiliário (IPCA + taxa)
nucleo-planejamento-mercado-resumo-dentro = { $contrato }: { $taxa } a.m., dentro do que o mercado cobra ({ $mediana } a.m. de mediana entre { $bancos } bancos).
nucleo-planejamento-mercado-resumo-acima = { $contrato }: { $taxa } a.m., acima da mediana do mercado ({ $mediana } a.m.); { $mais_baratos } de { $bancos } bancos cobram menos{ $melhor }. Vale simular a portabilidade.
# Vai depois de "cobram menos" (começa com a vírgula).
nucleo-planejamento-mercado-resumo-melhor = , o mais barato ({ $banco }) cobra { $taxa } a.m.
nucleo-planejamento-mercado-fonte = Banco Central do Brasil — taxas de juros por instituição financeira (média das operações de cada banco; o seu contrato depende do seu perfil).
nucleo-planejamento-mercado-aviso-fonte-desligada = A fonte Banco Central está desligada em Ajustes › Fontes de dados.
nucleo-planejamento-mercado-aviso-falha = Não deu para atualizar as taxas do BCB ({ $erro }); valem as últimas baixadas.
nucleo-planejamento-mercado-aviso-sem-taxas = Ainda não há taxas do BCB baixadas.
nucleo-planejamento-mercado-erro-resposta = resposta do BCB inválida: { $erro }
nucleo-planejamento-mercado-erro-sem-periodo = o BCB não devolveu o período mais recente
nucleo-planejamento-mercado-erro-sem-taxas = o BCB não devolveu taxas

## Extrato do contrato colado do banco (`debt::statement`)

nucleo-planejamento-divida-extrato-linhas-nao-entendidas =
    { $n ->
        [one] { $n } linha não entendida, por exemplo: "{ $exemplo }"
       *[other] { $n } linhas não entendidas, por exemplo: "{ $exemplo }"
    }
nucleo-planejamento-divida-extrato-erro-vazio = cole o extrato do contrato (ou envie as linhas)
nucleo-planejamento-divida-extrato-erro-nenhuma-linha = nenhuma linha do extrato foi entendida: cole com o cabeçalho (data, amortização, juros, saldo devedor…)
nucleo-planejamento-divida-extrato-aviso-juros = os juros do extrato equivalem a { $mensal }% ao mês ({ $nominal }% a.a. nominal, { $efetiva }% a.a. efetiva); o contrato está com { $contrato }% a.a. efetiva. Ajuste a taxa para a projeção bater com o banco
nucleo-planejamento-divida-extrato-aviso-prazo = não deu para calcular o prazo que falta pela última prestação: informe o prazo remanescente
nucleo-planejamento-divida-extrato-erro-prazo = informe o prazo remanescente (quantas parcelas faltam depois da última linha)
nucleo-planejamento-divida-extrato-erro-antes-do-vencimento = a linha de { $data } é anterior ao primeiro vencimento do contrato ({ $vencimento }): corrija o primeiro vencimento

## Plano de quitação: bola de neve × avalanche (`debt::payoff`)

nucleo-planejamento-quitacao-contrato = contrato { $id }
nucleo-planejamento-quitacao-erro-extra-negativo = o extra por mês não pode ser negativo
nucleo-planejamento-quitacao-erro-divida-incompleta = cada dívida informada precisa de nome, saldo e pagamento mínimo positivos
nucleo-planejamento-quitacao-erro-taxa-negativa = taxa negativa
nucleo-planejamento-quitacao-erro-sem-dividas = nenhuma dívida com saldo: cadastre um empréstimo ou informe uma dívida
nucleo-planejamento-quitacao-nota-indexador = { $contrato }: só a taxa contratada, sem a correção do indexador.
nucleo-planejamento-quitacao-nota-indexador-valor = { $contrato }: taxa contratada mais { $indice } de { $valor }% ao mês (último valor, de { $data }), mantido igual até o fim.
nucleo-planejamento-quitacao-nota-sac = { $contrato }: no SAC a parcela cai com o tempo; aqui ela fica fixa (quita um pouco antes).
nucleo-planejamento-quitacao-nota-minimo-nao-quita = Só com o mínimo alguma dívida não acaba em 50 anos: a parcela não cobre os juros.
nucleo-planejamento-quitacao-mais-de-50-anos = mais de 50 anos
nucleo-planejamento-quitacao-avalanche = avalanche
nucleo-planejamento-quitacao-bola-de-neve = bola de neve
nucleo-planejamento-quitacao-explicacao-minimo = Só com o mínimo: livre das dívidas em { $quando } com { $juros } de juros.
nucleo-planejamento-quitacao-explicacao-minimo-nunca = Só com o mínimo, alguma dívida nunca acaba: a parcela não cobre os juros.
nucleo-planejamento-quitacao-explicacao-estrategias = Bola de neve: { $bola_quando } ({ $bola_juros } de juros). Avalanche: { $avalanche_quando } ({ $avalanche_juros } de juros).
nucleo-planejamento-quitacao-explicacao-iguais = As duas dão o mesmo resultado aqui; a bola de neve quita a primeira dívida mais cedo.
# A estratégia vem do catálogo ("avalanche", "bola de neve").
nucleo-planejamento-quitacao-explicacao-economia = A { $estrategia } economiza { $valor } de juros em relação ao mínimo.
nucleo-planejamento-quitacao-explicacao-menos-juros = A { $estrategia } paga { $valor } a menos de juros.
nucleo-planejamento-cenario-negativo-hoje = O dinheiro disponível já está negativo hoje.
nucleo-planejamento-divida-parcela-n = parcela { $n } deste empréstimo
nucleo-planejamento-previdencia-motivo-pagamentos = Histórico de pagamentos da previdência importado
nucleo-planejamento-previdencia-pagamentos-erro-classe = O histórico de pagamentos é para ativos da classe Previdência.
nucleo-planejamento-previdencia-pagamentos-erro-vazio = Informe ao menos um pagamento (data e valor).
nucleo-planejamento-previdencia-pagamentos-erro-valor = O valor de cada pagamento deve ser maior que zero.
nucleo-planejamento-previdencia-pagamentos-erro-conta = Escolha a conta das cotas do plano.
nucleo-planejamento-previdencia-pagamentos-sem-cnpj = Sem cota: informe no ativo o CNPJ do fundo do plano (o FIE), para buscar as cotas na CVM.
nucleo-planejamento-previdencia-pagamentos-sem-cota =
    { $n ->
        [one] { $n } aporte ainda sem cota da CVM, desde { $data }: entra na proposta com a quantidade pendente e vira compra de cotas sozinho quando a coleta de cotações trouxer o histórico do fundo.
       *[other] { $n } aportes ainda sem cota da CVM, desde { $data }: entram na proposta com a quantidade pendente e viram compra de cotas sozinhos quando a coleta de cotações trouxer o histórico do fundo.
    }
nucleo-planejamento-previdencia-pagamentos-nada = Nada novo para gravar: os pagamentos já foram importados.
nucleo-planejamento-previdencia-pagamentos-ilegivel-valor = Print { $print }: a linha de { $lido } está com o valor ilegível. Confira no app da seguradora e informe o valor.
nucleo-planejamento-previdencia-pagamentos-ilegivel-data = Print { $print }: o pagamento de { $lido } está sem data legível. Confira no app da seguradora e informe a data.
nucleo-planejamento-previdencia-pagamentos-cortada-alto = Print { $print }: a primeira linha está cortada (só { $lido }). Mande o print de cima com a linha inteira, ou informe o pagamento.
nucleo-planejamento-previdencia-pagamentos-cortada-fim = Print { $print }: a última linha está cortada (só { $lido }). Mande o print de baixo com a linha inteira, ou informe o pagamento.
nucleo-planejamento-previdencia-pagamentos-mes-faltando = Nenhum pagamento em { $lido }, no meio de uma série mensal. Confira se houve aporte nesse mês (pode faltar um print).
nucleo-planejamento-previdencia-aviso-aportes-sem-cota =
    { $n ->
        [one] { $n } aporte ainda sem cota da CVM entra no saldo pelo valor aportado, sem rendimento, até a coleta trazer a cota do dia
       *[other] { $n } aportes ainda sem cota da CVM entram no saldo pelo valor aportado, sem rendimento, até a coleta trazer a cota do dia
    }
nucleo-planejamento-previdencia-pagamentos-vao = Prints { $print }: o dia { $lido } ficou com menos aportes que os outros dias da série; um pagamento pode ter ficado entre os dois prints. Mande um print que mostre esse trecho, ou informe o pagamento.
nucleo-planejamento-previdencia-pagamentos-conta-nova = Cotas · { $plano }
nucleo-planejamento-previdencia-pagamentos-conta-nota = Conta das cotas do plano { $plano }, criada na importação do histórico de pagamentos (sem movimento no caixa).
nucleo-planejamento-previdencia-motivo-pagamentos-n = { $plano }: { $com } aporte(s) com cota, { $sem } aguardando a cota da CVM e { $pendencias } pendência(s) para conferir
nucleo-planejamento-previdencia-motivo-cotas-completadas =
    { $n ->
        [one] Cotas da CVM completadas em { $n } aporte de previdência
       *[other] Cotas da CVM completadas em { $n } aportes de previdência
    }
nucleo-previdencia-saldo-so-previdencia = o saldo informado é só para planos de previdência
nucleo-previdencia-saldo-positivo = o saldo precisa ser maior que zero
nucleo-previdencia-saldo-sem-cotas = o plano não tem cotas em { $data }: registre os aportes antes de informar o saldo

## Parcelas detectadas pela descrição
nucleo-parcelas-ref-serie = série de parcelas
nucleo-parcelas-historico-ignorar = Não é parcela: { $descricao }
nucleo-parcelas-historico-voltar = Série de parcelas de volta
nucleo-parcelas-previsao = { $descricao } · parcela detectada
