# Figura 07 · Inflação da tela de Relatórios (features/reports/figures/InflationFigure.tsx e
# charts/inflationOption.ts). pt-BR é o idioma de origem.

inflacao-carregando = Carregando a inflação…
inflacao-pergunta = Gastamos mais ou só ficou mais caro? Cada mês trazido a dinheiro de { $mes } pela inflação oficial.
inflacao-indice-generico = índice de inflação
inflacao-titulo = Gasto nominal × corrigido pelo { $indice }, { $n } meses
inflacao-aria = Linhas do gasto nominal e do gasto corrigido pelo { $indice } em { $n } meses, em dinheiro de { $mes }
inflacao-fonte = Livro-caixa · base { $base } · valores corrigidos até { $mes }
inflacao-fonte-indice = { $indice } ({ $fonte }), último dado de { $data }
inflacao-nota-estimado = Os meses ainda não divulgados seguem a média das últimas variações do índice e ficam marcados com ≈.
inflacao-tabela = Gasto nominal e corrigido por mês

## Medidas

inflacao-acumulada = Inflação em { $n } meses
inflacao-sem-indice-curto = Sem índice
inflacao-gasto-nominal = Gasto nominal
inflacao-gasto-real = Gasto real
inflacao-tres-meses = Primeiros × últimos 3 meses
inflacao-correcao-total = Correção no período
inflacao-correcao-total-dica = Quanto a inflação soma ao gasto antigo
inflacao-sem-indice = Ainda não há dados do { $indice } para esse período. Atualize os índices em Ativos › Índices e cotações.
inflacao-sem-cambio = Sem câmbio para { $moedas }: esses valores ficaram fora dos totais.

## Gráfico

inflacao-serie-nominal = Nominal
inflacao-serie-corrigido = Corrigido
inflacao-tooltip-correcao = Correção
inflacao-tooltip-nao-divulgado = Índice do mês ainda não divulgado: correção estimada.
inflacao-tooltip-estimado = A correção usa meses ainda não divulgados (estimados).

## Tabela por mês

inflacao-col-corrigido = Em dinheiro de { $mes }
inflacao-col-correcao = Correção
inflacao-col-indice = Índice
inflacao-status-oficial = Oficial
inflacao-status-estimado = Estimado em parte
inflacao-status-nao-divulgado = Não divulgado

## Por categoria

inflacao-categorias-titulo = Categorias contra a inflação
inflacao-categorias-dica = { $acima ->
    [0] Média de { $inicio } a três meses depois contra os três meses até { $fim }. Nenhuma das { $total } categorias subiu acima da inflação.
    [1] Média de { $inicio } a três meses depois contra os três meses até { $fim }. 1 de { $total } categorias subiu acima da inflação.
   *[other] Média de { $inicio } a três meses depois contra os três meses até { $fim }. { $acima } de { $total } categorias subiram acima da inflação.
}
inflacao-categorias-tabela = Categorias contra a inflação
inflacao-col-inicio = Média no início
inflacao-col-fim = Média no fim
inflacao-col-variacao = Variação
inflacao-col-acima = Acima da inflação
