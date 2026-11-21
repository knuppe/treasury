# Tela 05 · Orçamento: abas, previsto × realizado do mês (resumo, tabela editável, copiar do
# mês anterior e sugerir pela média) (docs/i18n-design.md). Metas, projetos, reserva e
# aposentadoria têm os próprios catálogos; assinaturas e cenários ficam em planejamento.ftl.
# pt-BR é o idioma de origem.

## Abas

orcamento-abas = Seções do orçamento
orcamento-aba-previsto = Previsto × realizado
orcamento-aba-metas = Metas
orcamento-aba-projetos = Projetos
orcamento-aba-reserva = Reserva de emergência
orcamento-aba-aposentadoria = Aposentadoria
orcamento-aba-assinaturas = Assinaturas
orcamento-aba-parcelas = Parcelas
orcamento-aba-cenarios = Cenários
orcamento-carregando = Somando o orçamento

## Rótulos repetidos

orcamento-previsto = Previsto
orcamento-realizado = Realizado
orcamento-recebido = Recebido
orcamento-comprometido = Comprometido
orcamento-restante = Restante
orcamento-definir = Definir
orcamento-erro-gravar = Falha ao gravar.

## Resumo do mês

orcamento-resumo-de = Resumo de { $mes }
orcamento-base = Base
orcamento-base-caixa = caixa
orcamento-base-competencia = competência
orcamento-mes-decorrido = Mês decorrido
orcamento-despesas-de = Despesas de { $mes }
orcamento-comprometido-dica = previstos a efetivar
orcamento-ritmo = Ritmo do mês
orcamento-de = Orçamento de { $mes }
orcamento-ritmo-texto = { $pct } do previsto com { $decorrido } do mês · projeção { $projecao }

## Copiar do mês anterior

orcamento-copiar-titulo = Copiar orçamento
orcamento-copiar-descricao = Copia as linhas de { $de } para { $para }. Categorias arquivadas ficam de fora.
orcamento-copiar-anterior = Copiar do mês anterior
orcamento-copiado = Orçamento copiado
orcamento-copiar = Copiar
orcamento-substituir = Substituir as linhas que já existem em { $mes }

## Sugerir pela média

orcamento-aplicar = Aplicar
orcamento-atual = Atual
orcamento-media = Média
orcamento-diferenca = Diferença
orcamento-sugestao = Sugestão pela média
orcamento-aplicar-a = Aplicar a { $categoria }
orcamento-sugerir = Sugerir pela média
orcamento-sugerir-descricao = Média do gasto de cada categoria nos meses completos anteriores, arredondada para cima ao múltiplo de { $valor }.
orcamento-atualizado = Orçamento atualizado
orcamento-atualizado-descricao = { $n } categorias pela média.
orcamento-aplicar-n =
    { $n ->
        [0] Aplicar
       *[other] Aplicar { $n }
    }
orcamento-meses-media = Meses na média
orcamento-n-meses = { $n } meses
orcamento-calculando = Calculando a média…
orcamento-igual-media = O orçamento já está igual à média.

## Tabela e avisos

orcamento-gravar-erro = Não foi possível gravar o previsto
orcamento-adicionar-aria = Adicionar categoria ao orçamento
orcamento-adicionar = Adicionar categoria…
orcamento-sem-orcamento-aria = Categorias com gasto e sem orçamento
orcamento-n-sem-orcamento =
    { $n ->
        [1] { $n } categoria com gasto e sem orçamento
       *[other] { $n } categorias com gasto e sem orçamento
    }
orcamento-despesas-categoria = Despesas por categoria
orcamento-vazio = Sem orçamento nem gasto em { $mes }. Copie do mês anterior, use a média ou adicione uma categoria.
orcamento-tabela-despesas = Orçamento de despesas por categoria
orcamento-receitas-previstas = Receitas previstas
orcamento-tabela-receitas = Orçamento de receitas por categoria
orcamento-cambio = Câmbio
orcamento-sem-cambio = Sem câmbio para { $moedas }: esses lançamentos ficaram fora do realizado.
orcamento-previsto-de = Previsto de { $categoria }
orcamento-soma-filhos = soma dos filhos
orcamento-editar-previsto-de = Editar previsto de { $categoria }
orcamento-gravando = gravando…
orcamento-soma-subcategorias-dica = Soma das subcategorias (o pai não tem valor próprio)
orcamento-soma-subcategorias = soma das subcategorias
orcamento-previsto-realizado = Previsto × realizado
orcamento-recebido-previsto = Recebido × previsto
orcamento-percentual = Percentual
orcamento-editar-o-previsto = Editar o previsto
orcamento-definir-o-previsto = Definir o previsto
orcamento-ver-lancamentos-mes = Ver lançamentos de { $mes }
orcamento-recolher-subcategorias = Recolher subcategorias
orcamento-expandir-subcategorias = Expandir subcategorias
orcamento-acoes-de = Ações de { $categoria }
orcamento-recolher = Recolher { $categoria }
orcamento-expandir = Expandir { $categoria }
orcamento-arquivada = arquivada
orcamento-clique-realizado = Clique em Realizado para ver os lançamentos
orcamento-ver-lancamentos-categoria = Ver lançamentos de { $categoria } em { $mes }

## Para onde vai a renda (grupos de gasto)

orcamento-grupos-titulo = Para onde vai a renda
orcamento-grupos-aria = Grupos de gasto de { $mes }
orcamento-grupos-carregando = Calculando os grupos…
orcamento-grupos-renda = Renda do mês
orcamento-grupos-total = Nos quatro grupos
orcamento-grupos-livre = Sem destino
orcamento-grupos-livre-dica = Renda que sobrou depois dos quatro grupos.
orcamento-grupos-pct-renda = { $pct } da renda
orcamento-grupos-meta = meta { $pct }
orcamento-grupos-sem-meta = sem meta
orcamento-grupos-acima = acima da meta
orcamento-grupos-dentro = dentro da meta
orcamento-grupos-abaixo = abaixo da meta
orcamento-grupos-atingida = meta atingida
orcamento-grupos-barra = { $grupo }: { $pct } da renda
orcamento-grupos-sem-renda = Sem receitas neste mês: a parte da renda fica em branco.
orcamento-grupos-futuro-nota = Futuro soma os aportes e as transferências para contas de metas; resgates não abatem.
orcamento-grupos-ver-maiores = Ver as maiores de { $grupo }
orcamento-grupos-esconder-maiores = Esconder as maiores de { $grupo }
orcamento-grupos-maiores = Maiores de { $grupo }
orcamento-grupos-nada = Nada neste grupo no mês.
orcamento-grupos-relatorios = Ver a evolução em Relatórios
orcamento-grupos-categorias = Ajustar os grupos das categorias
orcamento-grupos-editar-metas = Editar metas
orcamento-grupos-metas-titulo = Metas por grupo
orcamento-grupos-metas-descricao = Quanto da renda você quer que vá para cada grupo. Deixe em branco para ficar sem meta. Uma referência comum é 50% para os essenciais, 30% para o não essencial e 20% para o futuro.
orcamento-grupos-meta-de = Meta de { $grupo } (% da renda)
orcamento-grupos-soma = Soma das metas: { $pct }
orcamento-grupos-soma-acima = A soma passa de 100% da renda.
orcamento-grupos-padrao = Voltar ao 50/30/20
orcamento-grupos-meta-invalida = Use um número de 0 a 100.
orcamento-grupos-metas-salvas = Metas por grupo salvas
