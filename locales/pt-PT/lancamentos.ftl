# Variante pt-PT (ecrã 04 · Lançamentos): o que faltar cai para pt-BR. Revisão: rascunho por IA.

## Rótulos de campo repetidos em várias partes do ecrã

lancamentos-campo-status = Estado

## Cabeçalho, separadores e barra de comando

lancamentos-abas = Secções de lançamentos

## Comandos (as palavras de pesquisa vão separadas por vírgula)

lancamentos-cmd-receita-busca = entrada,salário,ordenado
lancamentos-cmd-transferencia-busca = transferir,pix,mb way,reforço,aporte,câmbio
lancamentos-cmd-parcelado = Compra em prestações no cartão
lancamentos-cmd-parcelado-busca = prestações,parcelas,cartão
lancamentos-cmd-importar-busca = ofx,csv,banco,extrato,fatura
lancamentos-cmd-buscar-busca = procurar,pesquisar,filtrar
lancamentos-cmd-recorrencia-busca = subscrição,assinatura,renda,aluguel,mensal

## Avisos das ações e "Desfazer"

lancamentos-status-rotulo = Estado: { $status }
lancamentos-erro-status = Não foi possível mudar o estado
lancamentos-apagados = Lançamentos eliminados

## Porque é que duas marcadas não se ligam como transferência

lancamentos-par-ja-ligada = Uma delas já está ligada a outra conta: desligue-a primeiro (menu da linha).
lancamentos-par-sentidos = Para ligar, uma tem de ser saída e a outra entrada.
lancamentos-par-valores = Os valores são diferentes: na mesma moeda, o que sai tem de ser igual ao que entra.
lancamentos-par-tipos = Só receitas, despesas e transferências soltas (sem divisão nem prestações) se ligam.

## Tabela e paginação

lancamentos-atualizando = A atualizar
lancamentos-carregando = A carregar os lançamentos
lancamentos-vazio-descricao = Importe um OFX ou CSV, ou registe à mão. As transferências entre contas próprias não contam como despesa.
lancamentos-atalho-buscar = pesquisar
lancamentos-apagar-1 = Eliminar 1 lançamento
lancamentos-apagar-1-descricao = “{ $descricao }”. As transferências levam a outra perna. Os lançamentos conciliados não podem ser eliminados. Pode desfazer no histórico.

lancamentos-op-cotas = quotas
lancamentos-vencimento-fatura = Vencimento do extrato do cartão (caixa)
lancamentos-fat = ext. { $data }
lancamentos-parcela-de = Prestação { $numero } de { $contrato }
lancamentos-parc = prest. { $numero }

## Filtros

lancamentos-buscar = Pesquisar lançamentos
lancamentos-buscar-placeholder = Descrição, beneficiário ou notas
lancamentos-fatura-selecionada = Extrato do cartão selecionado
lancamentos-tirar-filtro = Remover filtro: { $filtro }
lancamentos-valor-absoluto = Valor absoluto: aplica-se a entradas e saídas.

## Menu da linha e menu do lote

lancamentos-menu-mesma-descricao = Marcar os que têm a mesma descrição
lancamentos-menu-pagamento-fatura = Pagamento do cartão…
lancamentos-menu-parcela = Prestação { $numero } de { $contrato }…
lancamentos-menu-pagamento-parcela = Pagamento de prestação de contrato…
lancamentos-menu-apagar = Eliminar…
lancamentos-conciliado-nao-apaga = Os lançamentos conciliados não podem ser eliminados.
lancamentos-menu-apagar-n = Eliminar { $n } lançamentos…

## Barra de ações em lote

lancamentos-tirar-categoria = Remover a categoria
lancamentos-mudar-status-marcados = Mudar o estado dos marcados
lancamentos-tirar-tag = Remover a tag { $tag }
lancamentos-tirar = Remover
lancamentos-tag-tirada = Tag “{ $tag }” removida
lancamentos-apagar-n = { $n ->
        [1] Eliminar { $n } lançamento
       *[other] Eliminar { $n } lançamentos
    }
lancamentos-apagar-n-descricao = As transferências levam a outra perna. Os lançamentos conciliados não podem ser eliminados. Pode desfazer no histórico.

## Diálogo do lançamento

lancamentos-erro-data = Indique uma data válida (dd/mm/aaaa).
lancamentos-erro-valor = Indique o valor (maior que zero).
lancamentos-erro-valor-destino = Indique quanto entra em { $conta } ({ $moeda }).
lancamentos-erro-descricao-compra = Indique a descrição da compra.
lancamentos-erro-parcelas = O número de prestações vai de 1 a 120.
lancamentos-erro-valor-ligado = Indique quanto entrou (ou saiu) em { $conta } ({ $moeda }).
lancamentos-erro-duas-partes = Uma divisão tem de ter pelo menos duas partes.
lancamentos-erro-valor-parte = Indique o valor de cada parte.
lancamentos-erro-soma-partes = A soma das partes tem de ser igual ao total.
lancamentos-tipo-registrado = { $tipo ->
        [investment] Reforço registado
        [card_payment] Pagamento do cartão registado
       *[other] Transferência registada
    }
lancamentos-parcelada-registrada = Compra em prestações registada
lancamentos-salvando = A guardar…
lancamentos-salvar-alteracoes = Guardar alterações
lancamentos-parcelado-cartao = Em prestações no cartão
lancamentos-moedas-diferentes = Contas em moedas diferentes: indique o valor recebido.
lancamentos-exemplo-despesa = Supermercado, farmácia…
lancamentos-parcelas = Prestações
lancamentos-parcelas-dica = As prestações futuras ficam como previstas.
lancamentos-favorecido = Beneficiário
lancamentos-projeto-dica = Obras, remodelação, festa ou viagem em 05 · Orçamento › Projetos.
lancamentos-anexos-dica = Comprovativos e faturas (PDF ou imagem, até 20 MB). Enviados depois de lançar.
lancamentos-nenhum-anexo = Nenhum comprovativo anexado.
lancamentos-anexar = Anexar comprovativo
lancamentos-apagar-item = Eliminar { $nome }

## Divisão

lancamentos-tirar-parte = Remover a parte { $n }
lancamentos-divisao-confere = bate certo ✓

## Recorrências (os dias da semana: abreviados de domingo a sábado; nas caixas, de segunda a domingo)

lancamentos-erro-descricao = Indique a descrição.
lancamentos-erro-valor-simples = Indique o valor.
lancamentos-erro-inicio = Indique a data de início.
lancamentos-recorrencia-exemplo = Renda, salário, ginásio…
lancamentos-previa-datas = Pré-visualização das próximas datas
lancamentos-previa = Pré-visualização
lancamentos-recorrencias-vazio-titulo = Registe salário, renda e subscrições
lancamentos-apagar-recorrencia = Eliminar recorrência
lancamentos-apagar-recorrencia-descricao = “{ $descricao }” deixa de gerar lançamentos. Os já efetuados ficam, sem vínculo. Pode desfazer no histórico.
lancamentos-recorrencia-apagada = Recorrência eliminada

## Ligar a outra conta (transferência ou pagamento do cartão)

lancamentos-pagamento-fatura = Pagamento do cartão
lancamentos-fatura-ligada = Pagamento ligado ao extrato do cartão
lancamentos-nenhum-cartao = Nenhum cartão registado. Registe o cartão em 03 · Contas para ligar o pagamento ao extrato.
lancamentos-procurando = A procurar o lançamento correspondente em { $conta }…
lancamentos-achei-entrada = Encontrei a entrada em { $conta }: { $data } · “{ $descricao }”. As duas ficam ligadas.
lancamentos-achei-saida = Encontrei a saída em { $conta }: { $data } · “{ $descricao }”. As duas ficam ligadas.
lancamentos-crio-entrada = Não há lançamento de { $valor } em { $conta } perto de { $data }: crio lá a entrada, com a mesma data e descrição.
lancamentos-crio-saida = Não há lançamento de { $valor } em { $conta } perto de { $data }: crio lá a saída, com a mesma data e descrição.
lancamentos-informe-entrou = Indique quanto entrou em { $conta }.
lancamentos-informe-saiu = Indique quanto saiu em { $conta }.
lancamentos-vira-fatura = Passa a pagamento do cartão: sai das despesas (as compras do cartão já contam).
lancamentos-quita-fatura = Liquida o extrato do cartão que vence em { $data }.
lancamentos-vira-aporte = Passa a reforço ou resgate da corretora: fica fora de receitas e despesas.
lancamentos-vira-transferencia = Passa a transferência entre contas da família: fica fora de receitas e despesas.

## Transferências sugeridas (pares que chegaram soltos)

lancamentos-aporte-ou-resgate = Reforço ou resgate
lancamentos-pares-aviso = { $n ->
        [1] 1 par de lançamentos parece uma transferência entre as suas contas
       *[other] { $n } pares de lançamentos parecem transferências entre as suas contas
    }
lancamentos-revisar = Rever
lancamentos-revisao-titulo = Transferências entre as suas contas
lancamentos-revisao-descricao = Pares de igual valor que saem de uma conta e entram noutra em até 4 dias. Ligados, passam a transferência: saem de receitas e despesas (relatórios, orçamento, saúde financeira) e perdem a categoria.
lancamentos-ligar-sozinho = Ligar automaticamente as próximas entre estas contas

## Vincular a uma operação (carteira, quotas de capital, meta)

lancamentos-operacao-cotas = Quotas de capital
lancamentos-bloqueio-transferencia = É uma transferência entre as suas contas: desligue a transferência para vincular.
lancamentos-bloqueio-previsto = O lançamento ainda está previsto: efetue-o antes de vincular.
lancamentos-bloqueio-parcela = O lançamento paga uma prestação de { $contrato }.
lancamentos-vinculado-descricao = { $operacao }. Saiu de receitas e despesas e passou a reforço.
lancamentos-vinculado-nota = Conta como reforço ({ $tipo }), fora de receitas e despesas. Desvincular devolve o tipo e a categoria de antes.
lancamentos-procurando-operacoes = A procurar as operações…
lancamentos-nenhuma-operacao = Nenhuma operação corresponde. Registe a aplicação em { $carteira } (até 7 dias da data, valor parecido), ative as quotas de capital na conta da cooperativa ou crie a meta, e volte aqui.
lancamentos-vinculados-descricao = { $operacao }. Saíram de receitas e despesas e passaram a reforço.
lancamentos-vinculados-descricao-fora = { $operacao }. { $fora } ficaram de fora. Saíram de receitas e despesas e passaram a reforço.
lancamentos-conferindo = A verificar os lançamentos…
lancamentos-vinculados-nota = Vinculados, saem de receitas e despesas e passam a reforço, sem categoria. Pode desfazer no histórico ou desvincular um a um.
lancamentos-nenhuma-operacao-marcados = Nenhuma operação corresponde aos marcados. As compras e vendas da carteira vinculam-se a um lançamento de cada vez (menu da linha); para vários, use uma meta ou as quotas de capital.
