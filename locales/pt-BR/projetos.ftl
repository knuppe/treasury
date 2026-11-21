# Orçamento › Projetos: obras, reformas, festas e viagens com orçamento, despesas e receitas,
# a linha do tempo, os destaques e as entradas; cadastro, itens lançados à mão e lançamentos
# postos num projeto (docs/i18n-design.md). pt-BR é o idioma de origem.

## Situação e rótulos repetidos

projetos-status-planejado = Planejado
projetos-status-andamento = Em andamento
projetos-status-concluido = Concluído
projetos-titulo = Projetos
projetos-projeto = Projeto
projetos-o-projeto = projeto
projetos-projeto-nome = Projeto { $nome }
projetos-novo = Novo projeto
projetos-novo-reticencias = Novo projeto…
projetos-sem-projeto = Sem projeto
projetos-nome = Nome
projetos-bem = Bem
projetos-nenhum = Nenhum
projetos-orcamento = Orçamento
projetos-situacao = Situação
projetos-observacoes = Observações
projetos-tipo = Tipo
projetos-descricao = Descrição
projetos-valor = Valor
projetos-data = Data
projetos-despesas = Despesas
projetos-receitas = Receitas
projetos-entradas = Entradas
projetos-sem-descricao = sem descrição
projetos-hoje = hoje
projetos-editar = Editar { $nome }
projetos-excluir = Excluir
projetos-acoes-de = Ações de { $nome }
projetos-tirar = Tirar do projeto
projetos-por = Pôr no projeto

## Lista e detalhe

projetos-ver-lancamentos = Ver lançamentos
projetos-lancamentos = Lançamentos
projetos-novo-item = Novo item
projetos-abrir = Abrir projeto
projetos-novo-item-menu = Novo item…
projetos-por-lancamentos-menu = Pôr lançamentos…
projetos-editar-menu = Editar…
projetos-excluir-menu = Excluir…
projetos-vazio-kicker = Sem projetos
projetos-vazio-titulo = Agrupe as despesas e receitas de uma obra, reforma, festa ou viagem
projetos-vazio-descricao = Um projeto soma as despesas e receitas ligadas a ele (lançamentos das contas e itens lançados à mão) e compara com o orçamento. Ligado a um bem, alimenta as benfeitorias: o custo do imóvel ou do carro passa a seguir o que a obra gastou.
projetos-situacao-lista = Situação dos projetos
projetos-todos-n = Todos ({ $n })
projetos-abertos-n = Em aberto ({ $n })
projetos-concluidos-n = Concluídos ({ $n })
projetos-nenhum-aberto = Nenhum projeto em aberto: veja os concluídos.
projetos-nenhum-concluido = Nenhum projeto concluído.
projetos-sem-bem = sem bem
projetos-periodo = Período
projetos-gasto = Gasto
projetos-resta = Resta
projetos-excluir-titulo = Excluir { $nome }
projetos-excluir-descricao = Os lançamentos e as benfeitorias continuam, só deixam de estar no projeto. Dá para desfazer pelo histórico.
projetos-excluir-projeto = Excluir projeto
projetos-excluido = Projeto excluído

## Linha do tempo, números e destaques

projetos-por-dia = Por dia
projetos-por-mes = Por mês
projetos-linha-tempo = Linha do tempo
projetos-visao-linha = Visão da linha do tempo
projetos-acumulado = Acumulado
projetos-grafico-acumulado = Despesa e receita acumuladas de { $nome } de { $de } a { $ate }, com o resultado.
projetos-grafico-acumulado-orcamento = Despesa e receita acumuladas de { $nome } de { $de } a { $ate }, com o resultado e o orçamento.
projetos-grafico-periodo-dia = Receitas e despesas de { $nome } por dia, com o resultado acumulado.
projetos-grafico-periodo-mes = Receitas e despesas de { $nome } por mês, com o resultado acumulado.
projetos-montando = Montando a linha do tempo…
projetos-linha-vazia = Nenhuma entrada ainda. { $lance } ou use “Lançamentos” para marcar os das contas.
projetos-linha-vazia-lance = Lance uma despesa ou receita à mão
projetos-maior-despesa = Maior despesa
projetos-maior-receita = Maior receita
projetos-dia-maior-gasto = Dia de maior gasto
projetos-mes-maior-gasto = Mês de maior gasto
projetos-orcamento-acaba = Orçamento acaba
projetos-orcamento-acaba-em = em { $mes }, no ritmo atual
projetos-acima-orcamento = acima do orçamento
projetos-resultado = Resultado
projetos-resultado-dica = receitas − despesas
projetos-passou = passou { $valor }
projetos-resta-valor = resta { $valor }
projetos-sem-orcamento = sem orçamento
projetos-pct-orcamento = { $pct }% do orçamento
projetos-gasto-mes = Gasto por mês
projetos-media-meses =
    { $n ->
        [1] média de { $n } mês
       *[other] média de { $n } meses
    }
projetos-entradas-dica =
    { $lancamentos ->
        [1] { $lancamentos } lançamento
       *[other] { $lancamentos } lançamentos
    } · { $itens ->
        [1] { $itens } item
       *[other] { $itens } itens
    }
projetos-destaques = Destaques
projetos-alimenta = Alimenta { $n } benfeitoria(s) de { $bem }: o custo do bem segue as despesas do projeto (as receitas não abatem).
projetos-um-bem = um bem
projetos-por-categoria = Por categoria
projetos-despesas-categoria = Despesas de { $nome } por categoria
projetos-nenhuma-despesa = Nenhuma despesa ainda.
projetos-receitas-categoria = Receitas de { $nome } por categoria
projetos-nenhuma-receita = Nenhuma receita.

## Entradas do projeto

projetos-entrada-sem-descricao = Entrada sem descrição
projetos-entrada-sem-descricao-min = entrada sem descrição
projetos-editar-item = Editar item…
projetos-editar-lancamento = Editar lançamento…
projetos-ver-em-lancamentos = Ver em Lançamentos
projetos-excluir-item = Excluir item…
projetos-tirar-menu = Tirar do projeto…
projetos-n-entradas-selecionadas = { $n } entradas selecionadas
projetos-tirar-n = Tirar { $n } entradas do projeto…
projetos-entradas-de = Entradas de { $nome }
projetos-acoes-n-entradas = Ações de { $n } entradas
projetos-entradas-vazio = Nenhuma entrada. Lance um { $item } ou marque { $lancamentos }.
projetos-entradas-vazio-item = item
projetos-entradas-vazio-lancamentos = lançamentos
projetos-origem = Origem
projetos-previsto = previsto
projetos-item-do-projeto = Item do projeto
projetos-n-entradas =
    { $n ->
        [1] { $n } entrada
       *[other] { $n } entradas
    }
projetos-excluir-item-titulo = Excluir item do projeto
projetos-tirar-lancamento-titulo = Tirar lançamento do projeto
projetos-tirar-n-titulo = Tirar { $n } entradas do projeto
projetos-tirar-itens-descricao = O item só existe no projeto e será apagado. Dá para desfazer pelo histórico.
projetos-tirar-misto-descricao = Os lançamentos continuam nas contas, só sem o projeto; os itens lançados à mão são apagados. Dá para desfazer pelo histórico.
projetos-tirar-lancamento-descricao = O lançamento continua na conta, só sem o projeto. Dá para desfazer pelo histórico.
projetos-entradas-tiradas = Entradas tiradas do projeto
projetos-entradas-tiradas-descricao =
    { $n ->
        [1] { $n } entrada de { $nome }.
       *[other] { $n } entradas de { $nome }.
    }

## Cadastro do projeto

projetos-salvo = Projeto salvo
projetos-criado = Projeto criado
projetos-form-descricao = Agrupe as despesas e receitas de uma obra, reforma, festa ou viagem. Ligado a um bem, o projeto alimenta as benfeitorias dele.
projetos-criar = Criar projeto
projetos-nome-exemplo = Reforma do banheiro, funilaria do carro…
projetos-bem-dica = Imóvel, veículo ou outro bem da obra (opcional).
projetos-orcamento-dica = Quanto pretende gastar (opcional).
projetos-inicio = Início
projetos-fim = Fim

## Pôr lançamentos num projeto

projetos-lanc-no-projeto = Lançamentos no projeto
projetos-lanc-fora = Lançamentos fora do projeto
projetos-n-lancamentos = { $n } lançamento(s).
projetos-titulo-lancamentos =
    { $n ->
        [1] Projeto do lançamento
       *[other] Projeto de { $n } lançamentos
    }
projetos-link-descricao = O gasto do projeto é a soma dos lançamentos dele; benfeitorias ligadas ao projeto passam a valer esse gasto.
projetos-nome-novo = Nome do novo projeto
projetos-nome-novo-exemplo = Reforma da cozinha
projetos-add-titulo = Lançamentos para { $nome }
projetos-add-descricao = Marque as despesas (compras, PIX, faturas) e as receitas do projeto. Lançamentos de outro projeto mudam para este.
projetos-add-marcados = { $n } marcado(s) · { $valor }
projetos-add-toast = Lançamentos em { $nome }
projetos-add-toast-descricao = { $n } lançamento(s) no projeto.
projetos-buscar = Buscar
projetos-buscar-exemplo = Descrição, favorecido ou observação
projetos-de = De
projetos-ate = Até
projetos-entradas-saidas = Entradas e saídas
projetos-so-saidas = Só saídas
projetos-so-entradas = Só entradas
projetos-encontrados = Lançamentos encontrados
projetos-buscando = Buscando…
projetos-nenhum-filtro = Nenhum lançamento neste filtro.
projetos-em-projeto = em { $nome }

## Item lançado à mão

projetos-item-receita-em = Receita em { $nome }.
projetos-item-despesa-em = Despesa em { $nome }.
projetos-item-salvo = Item salvo
projetos-lancamento-criado = Lançamento criado
projetos-item-lancado = Item lançado
projetos-item-novo-em = Novo item em { $nome }
projetos-item-editar-descricao = Item lançado à mão no projeto, fora das contas.
projetos-item-descricao = Despesa ou receita do projeto. Só no projeto não mexe em nenhuma conta; numa conta, vira um lançamento dela já no projeto.
projetos-lancar-conta = Lançar na conta
projetos-lancar-projeto = Lançar no projeto
projetos-item-exemplo-receita = Venda de sobras, ajuda da família…
projetos-item-exemplo-despesa = Pedreiro, material…
projetos-onde-lancar = Onde lançar
projetos-onde-conta = Entra no saldo e nos relatórios da conta.
projetos-onde-projeto = Só no projeto: fora das contas e dos relatórios.
projetos-so-no-projeto = Só no projeto
