# Tela 04 · Lançamentos: tabela, filtros, ações em lote, menu da linha, diálogo do lançamento,
# divisão, recorrências, transferências entre contas e vínculo com operações
# (docs/i18n-design.md). pt-BR é o idioma de origem.

## Rótulos de campo repetidos em várias partes da tela

lancamentos-campo-data = Data
lancamentos-campo-descricao = Descrição
lancamentos-campo-conta = Conta
lancamentos-campo-categoria = Categoria
lancamentos-campo-projeto = Projeto
lancamentos-campo-membro = Membro
lancamentos-campo-status = Status
lancamentos-campo-valor = Valor
lancamentos-campo-tipo = Tipo
lancamentos-campo-tags = Tags
lancamentos-campo-notas = Notas
lancamentos-sem-descricao = sem descrição
lancamentos-sem-descricao-parenteses = (sem descrição)
lancamentos-outra-conta = outra conta
lancamentos-contrato = contrato

## Cabeçalho, abas e barra de comando

lancamentos-descricao = Receitas, despesas e transferências. Filtre, divida, categorize em lote, importe extratos e ensine regras.
lancamentos-importar = Importar
lancamentos-transferencia = Transferência
lancamentos-cambio = Câmbio
lancamentos-novo = Novo lançamento
lancamentos-abas = Seções de lançamentos
lancamentos-aba-lancamentos = Lançamentos
lancamentos-aba-recorrencias = Recorrências
lancamentos-aba-regras = Regras
lancamentos-aba-importacoes = Importações
lancamentos-aba-caixa = Caixa de entrada
lancamentos-aba-email = E-mail

## Comandos (as palavras de busca vão separadas por vírgula)

lancamentos-cmd-novo-busca = despesa,receita,gasto
lancamentos-cmd-receita = Nova receita
lancamentos-cmd-receita-busca = entrada,salário
lancamentos-cmd-transferencia = Nova transferência entre contas
lancamentos-cmd-transferencia-busca = transferir,pix,aporte,câmbio
lancamentos-cmd-parcelado = Compra parcelada no cartão
lancamentos-cmd-parcelado-busca = parcelas,cartão
lancamentos-cmd-importar = Importar extrato (OFX/CSV)
lancamentos-cmd-importar-busca = ofx,csv,banco,fatura
lancamentos-cmd-buscar-busca = procurar,filtrar
lancamentos-cmd-sem-categoria = Ver lançamentos sem categoria
lancamentos-cmd-sem-categoria-busca = categorizar,pendentes
lancamentos-cmd-recorrencia-busca = assinatura,aluguel,mensal

## Avisos das ações e "Desfazer"

lancamentos-transferencia-desligada = Transferência desligada
lancamentos-transferencia-desligada-descricao = As duas pernas ficaram soltas: ligue cada uma ao par certo ou mude o tipo para receita ou despesa.
lancamentos-erro-desligar = Não foi possível desligar
lancamentos-erro-sugerir-regra = Não foi possível sugerir uma regra
lancamentos-status-rotulo = Status: { $status }
lancamentos-n-alterados =
    { $n ->
        [one] { $n } lançamento alterado.
       *[other] { $n } lançamentos alterados.
    }
lancamentos-erro-status = Não foi possível mudar o status
lancamentos-n-marcados-toast =
    { $n ->
        [one] { $n } lançamento marcado
       *[other] { $n } lançamentos marcados
    }
lancamentos-mesma-descricao-toast = Mesma descrição (“{ $descricao }”), em todo o período. Categorize ou mude em lote na barra acima.
lancamentos-mesma-descricao-lista = Mesma descrição ({ $descricoes }), em todo o período.
lancamentos-erro-marcar = Não foi possível marcar
lancamentos-transferencia-ligada = Transferência ligada entre as contas
lancamentos-transferencia-ligada-descricao = As duas pernas ficaram ligadas.
lancamentos-erro-ligar = Não foi possível ligar
lancamentos-feito = Feito
lancamentos-desfeito = Desfeito
lancamentos-erro-desfazer = Não foi possível desfazer
lancamentos-desfazer = Desfazer
lancamentos-dispensar = Dispensar
lancamentos-apagados = Lançamentos apagados

## Por que duas marcadas não se ligam como transferência

lancamentos-par-mesma-pagina = Marque as duas na mesma página da lista.
lancamentos-par-ja-ligada = Uma delas já está ligada a outra conta: desligue antes (menu da linha).
lancamentos-par-sentidos = Para ligar, uma precisa ser saída e a outra entrada.
lancamentos-par-mesma-conta = As duas estão na mesma conta: a transferência liga contas diferentes.
lancamentos-par-valores = Os valores são diferentes: na mesma moeda, o que sai precisa ser igual ao que entra.
lancamentos-par-tipos = Só receitas, despesas e transferências soltas (sem divisão nem parcelamento) se ligam.

## Tabela e paginação

lancamentos-atualizando = Atualizando
lancamentos-zero = 0 lançamentos
lancamentos-faixa = { $de }–{ $ate } de { $total }
lancamentos-totais = Totais do filtro
lancamentos-entradas = Entradas
lancamentos-saidas = Saídas
lancamentos-resultado = Resultado
lancamentos-totais-nota = Totais de todo o filtro, não só da página
lancamentos-carregando = Carregando lançamentos
lancamentos-vazio-kicker = Nenhum lançamento
lancamentos-vazio-titulo = Comece pelo extrato do banco
lancamentos-vazio-descricao = Importe um OFX ou CSV, ou lance à mão. Transferências entre contas próprias não contam como despesa.
lancamentos-importar-extrato = Importar extrato
lancamentos-paginas = Páginas
lancamentos-anterior = ← Anterior
lancamentos-proxima = Próxima →
lancamentos-pagina-de = página { $pagina } de { $paginas }
lancamentos-atalho-novo = novo
lancamentos-atalho-buscar = buscar
lancamentos-atalho-importar = importar
lancamentos-atalho-navegar = navegar
lancamentos-atalho-marcar = marcar
lancamentos-atalho-marcar-faixa = marcar faixa
lancamentos-atalho-categoria = categoria
lancamentos-atalho-abrir = abrir
lancamentos-apagar-1 = Apagar 1 lançamento
lancamentos-apagar-1-descricao = “{ $descricao }”. Transferências levam a outra perna junto. Lançamentos conciliados não podem ser apagados. Dá para desfazer pelo histórico.

lancamentos-op-carteira = carteira
lancamentos-op-cotas = cotas
lancamentos-op-meta = meta
lancamentos-ligada-a = Ligada a { $conta }
lancamentos-ligada-perna = Ligada à perna em { $conta }
lancamentos-solta-dica = Sem a outra perna. Marque a saída e a entrada e ligue as duas, ou use o menu da linha.
lancamentos-transferencia-solta = Transferência solta
lancamentos-solta = solta
lancamentos-dividido-em = Dividido em { $n }
lancamentos-categoria-de = Categoria de { $descricao }
lancamentos-erro-categoria = Não foi possível mudar a categoria
lancamentos-selecionar-pagina = Selecionar todos desta página
lancamentos-selecionar = Selecionar { $descricao }
lancamentos-vencimento-fatura = Vencimento da fatura (caixa)
lancamentos-fat = fat. { $data }
lancamentos-parcela-de = Parcela { $numero } de { $contrato }
lancamentos-parc = parc. { $numero }
lancamentos-vinculado = Vinculado: { $operacao }
lancamentos-recorrencia-dica = Gerado por uma recorrência
lancamentos-recorrente = Recorrente
lancamentos-n-anexos =
    { $n ->
        [one] { $n } anexo
       *[other] { $n } anexos
    }
lancamentos-acoes-marcados = Ações de { $n } lançamentos marcados
lancamentos-vazio-filtro = Nenhum lançamento com estes filtros.
lancamentos-acoes-de = Ações de { $descricao }
lancamentos-sem-descricao-acoes = lançamento sem descrição

## Filtros

lancamentos-filtros = Filtros de lançamentos
lancamentos-buscar = Buscar lançamentos
lancamentos-buscar-placeholder = Descrição, favorecido ou notas
lancamentos-filtro-receita = receita
lancamentos-filtro-arquivada = arquivada
lancamentos-periodo = Período
lancamentos-periodo-aria = Período: { $periodo }
lancamentos-contas = Contas
lancamentos-todas = Todas
lancamentos-categorias = Categorias
lancamentos-membros = Membros
lancamentos-projetos = Projetos
lancamentos-faixa-valor = Faixa de valor
lancamentos-base-datas = Base de datas
lancamentos-competencia = Competência
lancamentos-caixa = Caixa
lancamentos-so-sem-categoria = Só sem categoria
lancamentos-fatura-selecionada = Fatura selecionada
lancamentos-lote-importacao = Lote de importação
lancamentos-limpar-filtros = Limpar filtros
lancamentos-tirar-filtro = Tirar filtro: { $filtro }
lancamentos-de = De
lancamentos-ate = Até
lancamentos-aplicar-periodo = Aplicar período
lancamentos-valor-absoluto = Valor absoluto: vale para entradas e saídas.
lancamentos-minimo = Mínimo
lancamentos-maximo = Máximo
lancamentos-limpar = Limpar
lancamentos-aplicar = Aplicar

lancamentos-periodo-tudo = Todo o período
lancamentos-periodo-este-mes = Este mês
lancamentos-periodo-mes-passado = Mês passado
lancamentos-periodo-3-meses = Últimos 3 meses
lancamentos-periodo-12-meses = Últimos 12 meses
lancamentos-periodo-este-ano = Este ano
lancamentos-periodo-personalizado = Personalizado

## Menu da linha e menu do lote

lancamentos-marcar-como = Marcar como { $status }
lancamentos-menu-abrir = Abrir e editar
lancamentos-menu-categorizar = Categorizar
lancamentos-menu-ver-divisao = Ver a divisão
lancamentos-menu-desmarcar = Desmarcar
lancamentos-menu-marcar-lote = Marcar para ação em lote
lancamentos-menu-mesma-descricao = Marcar os de mesma descrição
lancamentos-menu-projeto = Projeto: { $projeto }…
lancamentos-menu-por-projeto = Pôr num projeto…
lancamentos-menu-transferencia-para = Transferência para outra conta…
lancamentos-menu-transferencia-de = Transferência de outra conta…
lancamentos-menu-desligar-de = Desligar de { $conta }
lancamentos-desligar-transferencia = Desligar a transferência
lancamentos-menu-pagamento-fatura = Pagamento de fatura…
lancamentos-menu-parcela = Parcela { $numero } de { $contrato }…
lancamentos-menu-pagamento-parcela = Pagamento de parcela de contrato…
lancamentos-menu-vinculo = Vínculo: { $operacao }…
lancamentos-menu-vincular = Vincular a uma operação…
lancamentos-menu-abrir-conta = Abrir a conta { $conta }
lancamentos-menu-so-conta = Só lançamentos desta conta
lancamentos-criar-regra-deste = Criar regra a partir deste
lancamentos-menu-apagar = Apagar…
lancamentos-conciliado-nao-apaga = Lançamentos conciliados não podem ser apagados.
lancamentos-menu-n-marcados = { $n } lançamentos marcados
lancamentos-menu-categorizar-marcados = Categorizar marcados…
lancamentos-menu-membro = Membro…
lancamentos-menu-tags = Tags…
lancamentos-menu-ligar-par = Ligar as duas como transferência
lancamentos-menu-marcar-todos = Marcar todos os { $n } do filtro
lancamentos-criar-regra = Criar regra
lancamentos-menu-apagar-n = Apagar { $n } lançamentos…

## Barra de ações em lote

lancamentos-acoes-lote = Ações nos lançamentos marcados
lancamentos-n-marcado = { $n ->
        [1] { $n } marcado
       *[other] { $n } marcados
    }
lancamentos-mesma-descricao = Mesma descrição
lancamentos-todos-do-filtro = Todos os { $n } do filtro
lancamentos-categorizar-marcados = Categorizar marcados
lancamentos-tirar-categoria = Tirar a categoria
lancamentos-categoria-definida = Categoria definida
lancamentos-categoria-removida = Categoria removida
lancamentos-erro-categorizar = Não foi possível categorizar
lancamentos-mudar-status-marcados = Mudar status dos marcados
lancamentos-projeto-reticencias = Projeto…
lancamentos-membro-marcados = Membro dos marcados
lancamentos-sem-membro = Sem membro
lancamentos-membro-definido = Membro: { $nome }
lancamentos-membro-removido = Membro removido
lancamentos-erro-membro = Não foi possível mudar o membro
lancamentos-tags-marcados = Tags dos marcados
lancamentos-tag = Tag
lancamentos-nova-tag = Nova tag
lancamentos-por = Pôr
lancamentos-tags-existentes = Tags existentes
lancamentos-por-tag = Pôr a tag { $tag }
lancamentos-tirar-tag = Tirar a tag { $tag }
lancamentos-tirar = Tirar
lancamentos-tag-posta = Tag “{ $tag }” posta
lancamentos-tag-tirada = Tag “{ $tag }” tirada
lancamentos-erro-tags = Não foi possível mudar as tags
lancamentos-ligar-como-transferencia = Ligar como transferência
lancamentos-vincular-operacao = Vincular a operação…
lancamentos-apagar-n = { $n ->
        [1] Apagar { $n } lançamento
       *[other] Apagar { $n } lançamentos
    }
lancamentos-apagar-n-descricao = Transferências levam a outra perna junto. Lançamentos conciliados não podem ser apagados. Dá para desfazer pelo histórico.
lancamentos-tags-exemplo = viagem, reembolsável…

## Diálogo do lançamento

lancamentos-membro-da-conta = Da conta ({ $nome })
lancamentos-familia = Família
lancamentos-erro-conta = Escolha a conta.
lancamentos-erro-data = Informe uma data válida (dd/mm/aaaa).
lancamentos-erro-valor = Informe o valor (maior que zero).
lancamentos-erro-destino = Escolha a conta de destino.
lancamentos-erro-mesma-conta = A conta de origem e a de destino devem ser diferentes.
lancamentos-erro-valor-destino = Informe quanto entra em { $conta } ({ $moeda }).
lancamentos-erro-descricao-compra = Informe a descrição da compra.
lancamentos-erro-parcelas = O número de parcelas vai de 1 a 120.
lancamentos-erro-valor-ligado = Informe quanto entrou (ou saiu) em { $conta } ({ $moeda }).
lancamentos-erro-duas-partes = Uma divisão precisa de pelo menos duas partes.
lancamentos-erro-valor-parte = Informe o valor de cada parte.
lancamentos-erro-soma-partes = A soma das partes precisa ser igual ao total.
lancamentos-tipo-registrado = { $tipo ->
        [investment] Aporte registrado
        [card_payment] Pagamento de fatura registrado
       *[other] Transferência registrada
    }
lancamentos-parcelada-registrada = Compra parcelada registrada
lancamentos-criado = Lançamento criado
lancamentos-atualizado = Lançamento atualizado
lancamentos-editar-tipo = Editar { $tipo }
lancamentos-editar = Editar lançamento
lancamentos-salvando = Salvando…
lancamentos-salvar-alteracoes = Salvar alterações
lancamentos-lancar = Lançar
lancamentos-tipo-lancamento = Tipo de lançamento
lancamentos-parcelado-cartao = Parcelado no cartão
lancamentos-de-origem = De (origem)
lancamentos-para-destino = Para (destino)
lancamentos-escolha-conta = Escolha a conta
lancamentos-valor-sai = Valor que sai ({ $moeda })
lancamentos-valor-entra = Valor que entra ({ $moeda })
lancamentos-cambio-implicito = Câmbio implícito: 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $origem }
lancamentos-moedas-diferentes = Contas em moedas diferentes: informe o valor recebido.
lancamentos-descricao-transferencia-dica = Opcional: o núcleo escreve “Transferência para …”.
lancamentos-cartao = Cartão
lancamentos-data-compra = Data da compra
lancamentos-exemplo-receita = Salário, reembolso…
lancamentos-exemplo-despesa = Mercado, farmácia…
lancamentos-valor-total-moeda = Valor total ({ $moeda })
lancamentos-valor-moeda = Valor ({ $moeda })
lancamentos-parcelas-previa = { $n }× de { $valor }
lancamentos-parcelas-previa-primeira = { $n }× de { $valor } (1ª de { $primeira })
lancamentos-parcelas = Parcelas
lancamentos-parcelas-dica = Parcelas futuras ficam como previstas.
lancamentos-como-transferencia-dica = Dinheiro movido para outra conta (da família ou de terceiros). Com categoria, conta no fluxo de caixa por ela; sem categoria, fica fora de receitas e despesas.
lancamentos-tipo-do-lancamento = Tipo do lançamento
lancamentos-para-conta-familia = Para a conta da família
lancamentos-da-conta-familia = Da conta da família
lancamentos-ligar-dica = Opcional: liga à perna que já existe nessa conta (mesmo valor, data próxima) ou cria essa perna.
lancamentos-nenhuma-terceiros = Nenhuma (terceiros)
lancamentos-valor-em = Valor em { $conta } ({ $moeda })
lancamentos-destinatario = Destinatário
lancamentos-favorecido = Favorecido
lancamentos-destinatario-dica = Opcional: para quem foi (ou de quem veio), quando não é conta da família.
lancamentos-favorecido-dica = Opcional: quem recebeu ou pagou.
lancamentos-projeto-dica = Obra, reforma, festa ou viagem em 05 · Orçamento › Projetos.
lancamentos-dividir = Dividir em categorias ou membros
lancamentos-anexos = Anexos
lancamentos-anexos-dica = Comprovantes e notas (PDF ou imagem, até 20 MB). Enviados depois de lançar.
lancamentos-anexo-salvar = Salvar { $nome }
lancamentos-anexo-salvo = Anexo salvo
lancamentos-nenhum-anexo = Nenhum comprovante anexado.
lancamentos-anexar = Anexar comprovante
lancamentos-apagar-item = Apagar { $nome }
lancamentos-editar-item = Editar { $nome }

## Divisão

lancamentos-divisao = Divisão
lancamentos-categoria-parte = Categoria da parte { $n }
lancamentos-membro-parte = Membro da parte { $n }
lancamentos-valor-parte = Valor da parte { $n }
lancamentos-tirar-parte = Tirar a parte { $n }
lancamentos-adicionar-parte = Adicionar parte
lancamentos-restante-ultima = Pôr o restante na última
lancamentos-divisao-soma = Soma { $soma } de { $total }
lancamentos-divisao-faltam = faltam { $valor }
lancamentos-divisao-sobram = sobram { $valor }
lancamentos-divisao-minimo = use pelo menos duas partes
lancamentos-divisao-confere = confere ✓

## Recorrências (os dias da semana: abreviados de domingo a sábado; nas caixas, de segunda a domingo)

lancamentos-dias-semana-abrev = dom.,seg.,ter.,qua.,qui.,sex.,sáb.
lancamentos-dias-semana-caixas = seg,ter,qua,qui,sex,sáb,dom
lancamentos-erro-descricao = Informe a descrição.
lancamentos-erro-valor-simples = Informe o valor.
lancamentos-erro-inicio = Informe a data de início.
lancamentos-recorrencia-atualizada = Recorrência atualizada
lancamentos-recorrencia-atualizada-descricao = Os previstos futuros foram refeitos.
lancamentos-recorrencia-criada = Recorrência criada
lancamentos-recorrencia-criada-descricao = Os lançamentos previstos dos próximos 2 meses já aparecem.
lancamentos-editar-recorrencia = Editar recorrência
lancamentos-nova-recorrencia = Nova recorrência
lancamentos-criar-recorrencia = Criar recorrência
lancamentos-recorrencia-exemplo = Aluguel, salário, academia…
lancamentos-repeticao = Repetição
lancamentos-frequencia = Frequência
lancamentos-freq-diaria = Diária
lancamentos-freq-semanal = Semanal
lancamentos-freq-mensal = Mensal
lancamentos-freq-anual = Anual
lancamentos-a-cada = A cada
lancamentos-dia-do-mes = Dia do mês
lancamentos-dia-do-mes-dica = 31 = último dia em meses curtos.
lancamentos-mes = Mês
lancamentos-dias-semana = Dias da semana
lancamentos-inicio = Início
lancamentos-fim = Fim (opcional)
lancamentos-ocorrencias = Ocorrências (opcional)
lancamentos-previa-datas = Prévia das próximas datas
lancamentos-previa = Prévia
lancamentos-previa-vazia = Preencha a repetição para ver as datas.
lancamentos-col-regra = Regra
lancamentos-col-proximas = Próximas
lancamentos-pausada = Pausada
lancamentos-col-por-mes = Por mês
lancamentos-acoes = Ações
lancamentos-recorrencia-pausada = Recorrência pausada
lancamentos-recorrencia-retomada = Recorrência retomada
lancamentos-erro-alterar = Não foi possível alterar
lancamentos-pausar = Pausar
lancamentos-retomar = Retomar
lancamentos-comprometido = Comprometido por mês ({ $moeda })
lancamentos-recorrencias-vazio-kicker = Nenhuma recorrência
lancamentos-recorrencias-vazio-titulo = Cadastre salário, aluguel e assinaturas
lancamentos-recorrencias-vazio-descricao = Recorrências geram lançamentos previstos para os próximos dois meses, que entram no saldo previsto e no orçamento.
lancamentos-apagar-recorrencia = Apagar recorrência
lancamentos-apagar-recorrencia-descricao = “{ $descricao }” deixa de gerar lançamentos. Os já efetivados ficam, sem vínculo. Dá para desfazer pelo histórico.
lancamentos-recorrencia-apagada = Recorrência apagada
lancamentos-previstos-removidos =
    { $n ->
        [one] { $n } previsto removido.
       *[other] { $n } previstos removidos.
    }
lancamentos-manter-previstos = Manter os lançamentos previstos já gerados

## Ligar a outra conta (transferência ou pagamento de fatura)

lancamentos-pagamento-fatura = Pagamento de fatura
lancamentos-transferencia-para = Transferência para outra conta
lancamentos-transferencia-de = Transferência de outra conta
lancamentos-fatura-ligada = Pagamento de fatura ligado ao cartão
lancamentos-ligado-a = Ligado a “{ $descricao }” em { $conta }.
lancamentos-criado-em = Criado o lançamento em { $conta }.
lancamentos-ligar-cartao = Ligar ao cartão
lancamentos-ligar-contas = Ligar as contas
lancamentos-saiu-em = saiu em { $conta }
lancamentos-entrou-em = entrou em { $conta }
lancamentos-saiu-de = saiu de { $conta }
lancamentos-nenhum-cartao = Nenhum cartão cadastrado. Cadastre o cartão em 03 · Contas para ligar o pagamento à fatura.
lancamentos-nenhuma-outra-conta = Nenhuma outra conta ativa.
lancamentos-para-a-conta = Para a conta
lancamentos-da-conta = Da conta
lancamentos-o-que-acontece = O que vai acontecer
lancamentos-procurando = Procurando o lançamento correspondente em { $conta }…
lancamentos-achei-entrada = Achei a entrada em { $conta }: { $data } · “{ $descricao }”. As duas ficam ligadas.
lancamentos-achei-saida = Achei a saída em { $conta }: { $data } · “{ $descricao }”. As duas ficam ligadas.
lancamentos-crio-entrada = Não há lançamento de { $valor } em { $conta } perto de { $data }: crio a entrada lá, com a mesma data e descrição.
lancamentos-crio-saida = Não há lançamento de { $valor } em { $conta } perto de { $data }: crio a saída lá, com a mesma data e descrição.
lancamentos-informe-entrou = Informe quanto entrou em { $conta }.
lancamentos-informe-saiu = Informe quanto saiu em { $conta }.
lancamentos-vira-fatura = Vira pagamento de fatura: sai das despesas (as compras do cartão já contam).
lancamentos-quita-fatura = Quita a fatura que vence em { $data }.
lancamentos-vira-aporte = Vira aporte ou resgate da corretora: fica fora de receitas e despesas.
lancamentos-vira-transferencia = Vira transferência entre contas da família: fica fora de receitas e despesas.

## Transferências sugeridas (pares que chegaram soltos)

lancamentos-aporte-ou-resgate = Aporte ou resgate
lancamentos-ligar-par-rotulo = Ligar { $de } → { $para } em { $data }
lancamentos-sempre-ligar = sempre ligar
lancamentos-sai-em = Sai em { $data }: { $descricao }
lancamentos-entra-em = Entra em { $data }: { $descricao }
lancamentos-nao-e = Não é
lancamentos-pares-aviso = { $n ->
        [1] 1 par de lançamentos parece transferência entre suas contas
       *[other] { $n } pares de lançamentos parecem transferências entre suas contas
    }
lancamentos-pares-aviso-nota = · sem ligar, a saída conta como despesa e a entrada como receita de novo.
lancamentos-revisar = Revisar
lancamentos-revisao-titulo = Transferências entre suas contas
lancamentos-revisao-descricao = Pares de mesmo valor que saem de uma conta e entram em outra em até 4 dias. Ligados, viram transferência: saem de receitas e despesas (relatórios, orçamento, saúde financeira) e perdem a categoria.
lancamentos-ligar-sozinho = Ligar sozinho as próximas entre estas contas
lancamentos-ligar-sozinho-descricao = Na importação de extratos, pares assim entram já ligados.
lancamentos-agora-nao = Agora não
lancamentos-n-transferencias-ligadas = { $n ->
        [1] 1 transferência ligada
       *[other] { $n } transferências ligadas
    }
lancamentos-ligar-n-pares = { $n ->
        [1] Ligar 1 par
       *[other] Ligar { $n } pares
    }
lancamentos-pares-sugeridos = Pares sugeridos

## Vincular a uma operação (carteira, cotas capital, meta)

lancamentos-operacao-carteira = Carteira
lancamentos-operacao-cotas = Cotas capital
lancamentos-operacao-meta = Meta
lancamentos-bloqueio-transferencia = É uma transferência entre suas contas: desligue a transferência para vincular.
lancamentos-bloqueio-previsto = O lançamento ainda está previsto: efetive-o antes de vincular.
lancamentos-bloqueio-dividido = Lançamentos divididos não se vinculam a uma operação.
lancamentos-bloqueio-parcela = O lançamento paga uma parcela de { $contrato }.
lancamentos-um-contrato = um contrato
lancamentos-operacoes = Operações
lancamentos-vinculado-toast = Lançamento vinculado
lancamentos-vinculado-descricao = { $operacao }. Saiu de receitas e despesas e virou aporte.
lancamentos-vinculo-desfeito = Vínculo desfeito
lancamentos-vinculo-desfeito-descricao = O lançamento voltou ao tipo e à categoria de antes.
lancamentos-vincular-titulo = Vincular a uma operação
lancamentos-vincular = Vincular
lancamentos-desvincular = Desvincular
lancamentos-vinculado-nota = Conta como aporte ({ $tipo }), fora de receitas e despesas. Desvincular devolve o tipo e a categoria de antes.
lancamentos-procurando-operacoes = Procurando as operações…
lancamentos-nenhuma-operacao = Nenhuma operação combina. Registre a aplicação em { $carteira } (até 7 dias da data, valor parecido), ative as cotas capital na conta da cooperativa ou crie a meta, e volte aqui.
lancamentos-situacao = Situação
lancamentos-entra = ✓ Entra
lancamentos-fica-de-fora = Fica de fora: { $motivo }
lancamentos-n-vinculados = { $n ->
        [1] { $n } lançamento vinculado
       *[other] { $n } lançamentos vinculados
    }
lancamentos-vinculados-descricao = { $operacao }. Saíram de receitas e despesas e viraram aporte.
lancamentos-vinculados-descricao-fora = { $operacao }. { $fora } ficaram de fora. Saíram de receitas e despesas e viraram aporte.
lancamentos-confirmar-vinculo = Confirmar o vínculo
lancamentos-n-marcados-destino = { $n ->
        [1] { $n } lançamento marcado
       *[other] { $n } lançamentos marcados
    } → { $destino }
lancamentos-n-marcados-escolha = { $n ->
        [1] { $n } lançamento marcado
       *[other] { $n } lançamentos marcados
    }: escolha a operação de destino.
lancamentos-vincular-n = { $n ->
        [1] Vincular { $n } lançamento
       *[other] Vincular { $n } lançamentos
    }
lancamentos-conferindo = Conferindo os lançamentos…
lancamentos-destino = Destino:
lancamentos-total = Total:
lancamentos-n-ficam-de-fora = { $n ->
        [1] { $n } lançamento fica de fora
       *[other] { $n } lançamentos ficam de fora
    }
lancamentos-outros-vinculados = Os outros são vinculados; estes continuam como estão.
lancamentos-nenhum-vinculavel = Nenhum dos marcados pode ser vinculado a esta operação.
lancamentos-a-vincular = Lançamentos a vincular
lancamentos-vinculados-nota = Vinculados, saem de receitas e despesas e viram aporte, sem categoria. Dá para desfazer pelo histórico ou desvincular um a um.
lancamentos-nenhuma-operacao-marcados = Nenhuma operação combina com os marcados. Compras e vendas da carteira se vinculam a um lançamento por vez (menu da linha); para vários, use uma meta ou as cotas capital.

lancamentos-ajuste-saldo = Ajuste de saldo
lancamentos-ajustar = Ajustar saldo

## Cópias entre contas

lancamentos-copias-aviso = { $n ->
    [one] { $n } lançamento repete uma compra já gravada em outra conta
   *[other] { $n } lançamentos repetem compras já gravadas em outra conta
}
lancamentos-copias-aviso-nota = Mesmo identificador do banco: em geral o arquivo de uma conta importado em outra. Contam duas vezes no fluxo e em "sem categoria".
lancamentos-copias-titulo = Cópias entre contas
lancamentos-copias-descricao = Cada linha é um lançamento gravado em duas contas com o mesmo identificador do banco. Marque as cópias para apagar; fica o da conta certa (a do cartão, nas compras). Dá para desfazer pelo histórico.
lancamentos-copias-lista = Cópias encontradas
lancamentos-copia-rotulo = Apagar a cópia em { $conta } de { $data }: { $descricao }
lancamentos-copia-lado = { $lado }: { $conta }, { $data } · { $descricao }
lancamentos-copia-apagar = Cópia
lancamentos-copia-fica = Fica
lancamentos-copia-transferencia = Esta cópia está ligada como transferência a outro lançamento: ao apagar, a ligação é desfeita e o outro lado volta a ser despesa ou receita.
lancamentos-apagar-n-copias = { $n ->
    [one] Apagar { $n } cópia
   *[other] Apagar { $n } cópias
}
lancamentos-n-copias-apagadas = { $n ->
    [one] { $n } cópia apagada
   *[other] { $n } cópias apagadas
}
lancamentos-copias-saldo-refeito = { $n ->
    [one] O ajuste de saldo que contava com elas foi refeito.
   *[other] Os { $n } ajustes de saldo que contavam com elas foram refeitos.
}
lancamentos-copias-erro = Não deu para apagar as cópias
lancamentos-resgate = Resgate
lancamentos-sem-categoria-aporte = Aporte em investimentos
lancamentos-sem-categoria-resgate = Resgate de investimentos
lancamentos-sem-categoria-aporte-ajuda = Aporte e resgate não usam categoria: o dinheiro muda de lugar no patrimônio e fica fora do fluxo de caixa e do orçamento. No Sankey, conta a saída do banco para a corretora (aporte) e a volta (resgate); compras e vendas dentro da corretora não entram de novo.
lancamentos-n-alterados-n-fora = { $n } alterado(s); { $fora } aporte(s), resgate(s), pagamento(s) de fatura ou ajuste(s) ficaram sem categoria, porque não usam
lancamentos-campo-cartao = Cartão usado
lancamentos-cartao-dica = A compra fica com quem usa este cartão, se você não escolher o membro.
lancamentos-cartao-nao-informado = Não informado
lancamentos-campo-moeda-origem = Moeda da compra
lancamentos-moeda-origem-dica = Compra no exterior: a moeda e o valor que o estabelecimento cobrou.
lancamentos-campo-valor-origem = Valor na moeda da compra
