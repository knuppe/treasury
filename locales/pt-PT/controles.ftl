# Variante pt-PT (controlos partilhados e rótulos dos tipos do núcleo): o que faltar cai para
# pt-BR. Revisão: rascunho por IA.

## Erros, ficheiros e estado de consulta

controles-erro-generico = Algo correu mal.
controles-erro-ler-arquivo = não foi possível ler o ficheiro
controles-carregando = A carregar
controles-tentar-de-novo = Tentar novamente

## Seleção múltipla e pesquisa

controles-buscar-em = Pesquisar em { $rotulo }
controles-buscar = Pesquisar…

## Categoria e descrição

controles-buscar-rotulo = Pesquisar { $rotulo }
controles-buscar-categoria = Pesquisar categoria…

## Tipos de conta

controles-conta-corrente = Conta à ordem
controles-conta-exterior = Conta no estrangeiro
controles-conta-beneficio = Cartão refeição

## Grupos do ecrã de contas

controles-grupo-correntes = Contas à ordem
controles-grupo-exterior = Estrangeiro
controles-grupo-beneficio = Cartão refeição

## Tipos e estado de lançamento

controles-tipo-pagamento-fatura = Pagamento do cartão
controles-tipo-aporte = Reforço

controles-status-efetivado = Efetuado

## Estado do extrato do cartão e das linhas importadas

controles-fatura-aberta = Aberto
controles-fatura-fechada = Fechado
controles-fatura-paga = Pago
controles-fatura-parcial = Pago em parte

## Tipos de cópia de segurança

controles-backup-migracao = Antes de atualizar a base de dados
controles-backup-importacao = Antes de uma importação grande
controles-backup-ia = Antes de uma ação da IA
controles-backup-senha = Antes de alterar a palavra-passe
controles-backup-fechar = Ao fechar a aplicação

## Autores das alterações

controles-autor-voce = Utilizador

## Campos e operações das regras

controles-campo-favorecido = Beneficiário

controles-op-comeca = começa por
controles-op-termina = termina em
controles-op-regex = corresponde à regex

## Tabelas do núcleo no histórico de alterações

controles-tabela-fatura = Extrato do cartão
controles-tabela-parcelamento = Plano de prestações
controles-tabela-tela-screener = Ecrã do screener

## Contagem por tabela no histórico (singular só para 1)

controles-n-fatura = { $n ->
        [1] { $n } extrato do cartão
       *[other] { $n } extratos do cartão
    }
controles-n-parcelamento = { $n ->
        [1] { $n } plano de prestações
       *[other] { $n } planos de prestações
    }
controles-n-tela-screener = { $n ->
        [1] { $n } ecrã do screener
       *[other] { $n } ecrãs do screener
    }
