# Controles compartilhados (features/transactions/ui): caixas, chaves, seleção múltipla, escolha
# de categoria, estado de consulta, mensagens de erro e os rótulos dos tipos do núcleo usados em
# várias telas (docs/i18n-design.md). pt-BR é o idioma de origem.

## Erros, arquivos e estado de consulta

controles-erro-generico = Algo deu errado.
controles-erro-ler-arquivo = não foi possível ler o arquivo
controles-tamanho-kb = { NUMBER($n, maximumFractionDigits: 1) } KB
controles-tamanho-mb = { NUMBER($n, maximumFractionDigits: 1) } MB
controles-carregando = Carregando
controles-tentar-de-novo = Tentar de novo
controles-ligado = Ligado
controles-desligado = Desligado

## Seleção múltipla e busca

controles-todos = Todos
controles-n-selecionados = { $n } selecionados
controles-buscar-em = Buscar em { $rotulo }
controles-buscar = Buscar…
controles-nada-encontrado = Nada encontrado.
controles-limpar-selecao = Limpar seleção

## Categoria e descrição

controles-categoria = Categoria
controles-sem-categoria = Sem categoria
controles-buscar-rotulo = Buscar { $rotulo }
controles-buscar-categoria = Buscar categoria…
controles-nenhuma-categoria = Nenhuma categoria.
controles-sugestoes = Sugestões de lançamentos

## Tipos de conta

controles-conta-corrente = Conta corrente
controles-conta-poupanca = Poupança
controles-conta-dinheiro = Dinheiro
controles-conta-cartao = Cartão de crédito
controles-conta-corretora = Corretora
controles-conta-exterior = Conta no exterior
controles-conta-beneficio = Vale-alimentação e refeição
controles-conta-fgts = FGTS
controles-conta-aposentadoria = Aposentadoria (401(k), IRA, PPR…)
controles-conta-outra = Outra

## Grupos da tela de contas

controles-grupo-correntes = Contas correntes
controles-grupo-poupanca = Poupança
controles-grupo-dinheiro = Dinheiro
controles-grupo-cartoes = Cartões
controles-grupo-corretoras = Corretoras
controles-grupo-exterior = Exterior
controles-grupo-beneficio = Vale-alimentação e refeição
controles-grupo-fgts = FGTS
controles-grupo-aposentadoria = Contas de aposentadoria
controles-grupo-outras = Outras

## Tipos de instituição

controles-instituicao-banco = Banco
controles-instituicao-corretora = Corretora
controles-instituicao-emissor = Emissor de cartão
controles-instituicao-outra = Outra

## Tipos e status de lançamento

controles-tipo-receita = Receita
controles-tipo-despesa = Despesa
controles-tipo-transferencia = Transferência
controles-tipo-pagamento-fatura = Pagamento de fatura
controles-tipo-aporte = Aporte
controles-tipo-ajuste = Ajuste

controles-status-previsto = Previsto
controles-status-efetivado = Efetivado
controles-status-conciliado = Conciliado

## Status da fatura e das linhas importadas

controles-fatura-aberta = Aberta
controles-fatura-fechada = Fechada
controles-fatura-paga = Paga
controles-fatura-parcial = Paga em parte

controles-linha-nova = Nova
controles-linha-duplicada = Duplicada
controles-linha-possivel-duplicada = Possível duplicada
controles-linha-erro = Com erro
controles-linha-ignorada = Ignorada

## Tipos de backup

controles-backup-diario = Automático diário
controles-backup-manual = Manual
controles-backup-migracao = Antes de atualizar o banco
controles-backup-importacao = Antes de importação grande
controles-backup-ia = Antes de ação da IA
controles-backup-restauracao = Antes de restaurar
controles-backup-senha = Antes de trocar a senha
controles-backup-fechar = Ao fechar o app

## Autores das alterações

controles-autor-voce = Você
controles-autor-importacao = Importação
controles-autor-ia = IA
controles-autor-sistema = Sistema

## Campos e operações das regras

controles-campo-descricao = Descrição
controles-campo-favorecido = Favorecido
controles-campo-notas = Notas
controles-campo-valor = Valor
controles-campo-conta = Conta
controles-campo-tipo = Tipo

controles-op-contem = contém
controles-op-igual = é igual a
controles-op-comeca = começa com
controles-op-termina = termina com
controles-op-regex = casa com a regex
controles-op-entre = entre
controles-op-maior-igual = maior ou igual a
controles-op-menor-igual = menor ou igual a

## Tabelas do núcleo no histórico de alterações

controles-tabela-familia = Família
controles-tabela-membro = Membro
controles-tabela-instituicao = Instituição
controles-tabela-conta = Conta
controles-tabela-membro-conta = Membro da conta
controles-tabela-cartao = Dados do cartão
controles-tabela-fatura = Fatura
controles-tabela-categoria = Categoria
controles-tabela-tag = Tag
controles-tabela-lancamento = Lançamento
controles-tabela-parte-divisao = Parte da divisão
controles-tabela-tag-lancamento = Tag do lançamento
controles-tabela-anexo = Anexo
controles-tabela-parcelamento = Parcelamento
controles-tabela-recorrencia = Recorrência
controles-tabela-regra = Regra
controles-tabela-lote-importacao = Lote de importação
controles-tabela-cambio = Câmbio
controles-tabela-orcamento = Orçamento
controles-tabela-linha-orcamento = Linha do orçamento
controles-tabela-meta = Meta
controles-tabela-conta-meta = Conta da meta
controles-tabela-projeto = Projeto
controles-tabela-ativo = Ativo
controles-tabela-operacao = Operação
controles-tabela-tela-screener = Tela do screener

## Contagem por tabela no histórico (singular só para 1)

controles-n-familia = { $n ->
        [1] { $n } família
       *[other] { $n } famílias
    }
controles-n-membro = { $n ->
        [1] { $n } membro
       *[other] { $n } membros
    }
controles-n-instituicao = { $n ->
        [1] { $n } instituição
       *[other] { $n } instituições
    }
controles-n-conta = { $n ->
        [1] { $n } conta
       *[other] { $n } contas
    }
controles-n-membro-conta = { $n ->
        [1] { $n } membro da conta
       *[other] { $n } membros da conta
    }
controles-n-cartao = { $n ->
        [1] { $n } dados do cartão
       *[other] { $n } dados do cartão
    }
controles-n-fatura = { $n ->
        [1] { $n } fatura
       *[other] { $n } faturas
    }
controles-n-categoria = { $n ->
        [1] { $n } categoria
       *[other] { $n } categorias
    }
controles-n-tag = { $n ->
        [1] { $n } tag
       *[other] { $n } tags
    }
controles-n-lancamento = { $n ->
        [1] { $n } lançamento
       *[other] { $n } lançamentos
    }
controles-n-parte-divisao = { $n ->
        [1] { $n } parte da divisão
       *[other] { $n } partes da divisão
    }
controles-n-tag-lancamento = { $n ->
        [1] { $n } tag do lançamento
       *[other] { $n } tags do lançamento
    }
controles-n-anexo = { $n ->
        [1] { $n } anexo
       *[other] { $n } anexos
    }
controles-n-parcelamento = { $n ->
        [1] { $n } parcelamento
       *[other] { $n } parcelamentos
    }
controles-n-recorrencia = { $n ->
        [1] { $n } recorrência
       *[other] { $n } recorrências
    }
controles-n-regra = { $n ->
        [1] { $n } regra
       *[other] { $n } regras
    }
controles-n-lote-importacao = { $n ->
        [1] { $n } lote de importação
       *[other] { $n } lotes de importação
    }
controles-n-cambio = { $n ->
        [1] { $n } câmbio
       *[other] { $n } câmbios
    }
controles-n-orcamento = { $n ->
        [1] { $n } orçamento
       *[other] { $n } orçamentos
    }
controles-n-linha-orcamento = { $n ->
        [1] { $n } linha do orçamento
       *[other] { $n } linhas do orçamento
    }
controles-n-meta = { $n ->
        [1] { $n } meta
       *[other] { $n } metas
    }
controles-n-conta-meta = { $n ->
        [1] { $n } conta da meta
       *[other] { $n } contas da meta
    }
controles-n-projeto = { $n ->
        [1] { $n } projeto
       *[other] { $n } projetos
    }
controles-n-ativo = { $n ->
        [1] { $n } ativo
       *[other] { $n } ativos
    }
controles-n-operacao = { $n ->
        [1] { $n } operação
       *[other] { $n } operações
    }
controles-n-tela-screener = { $n ->
        [1] { $n } tela do screener
       *[other] { $n } telas do screener
    }
