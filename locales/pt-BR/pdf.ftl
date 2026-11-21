# Relatório do mês em PDF (Relatórios › Baixar PDF), montado pelo núcleo no próprio
# computador (reports/pdf_report.rs), e o botão da interface. pt-BR é o idioma de origem.

## Interface

pdf-botao = Baixar PDF do mês
pdf-botao-dica = Relatório de { $mes } em PDF, gerado no seu computador, sem enviar nada para a internet.
pdf-salvar-titulo = Salvar o relatório em PDF
pdf-arquivo = relatorio-treasury-{ $mes }.pdf
pdf-feito = Relatório salvo
pdf-feito-paginas = { $paginas ->
    [1] 1 página em { $caminho }
   *[other] { $paginas } páginas em { $caminho }
}
pdf-erro = Não foi possível gerar o PDF

## Documento

pdf-titulo = Relatório de { $mes }
pdf-mes-ano = { $mes } de { $ano }
pdf-base-competencia = Competência (data da compra)
pdf-base-caixa = Caixa (data do pagamento)
pdf-receitas = Receitas
pdf-despesas = Despesas
pdf-sobra = Sobra
pdf-taxa-poupanca = { $taxa } da renda guardada
pdf-patrimonio = Patrimônio líquido
pdf-sem-cambio = Sem câmbio para { $moedas }: esses valores ficaram fora dos totais.
pdf-secao-categorias = Para onde foi o dinheiro
pdf-secao-categorias-dica = Despesas do mês por categoria, com a parte de cada uma no total e a variação sobre o mês anterior.
pdf-sem-gastos = Nenhuma despesa neste mês.
pdf-col-categoria = Categoria
pdf-col-valor = Valor
pdf-col-parte = Parte
pdf-col-mes-anterior = Mês anterior
pdf-novo = novo
pdf-secao-12-meses = Entradas e saídas, 12 meses
pdf-secao-inflacao = Inflação
pdf-inflacao-resumo = Inflação acumulada em 12 meses: { $inflacao } ({ $indice }, último dado de { $data }). A linha corrigida mostra quanto o gasto de cada mês valeria hoje.
pdf-inflacao-sem-indice = Ainda não há dados do índice de inflação do país para corrigir os valores. Atualize os índices em Ativos › Índices e cotações.
pdf-inflacao-estimado = O trecho tracejado usa meses que o índice ainda não divulgou (estimativa).
pdf-inflacao-nominal = Gasto no mês (nominal)
pdf-inflacao-corrigido = Corrigido pela inflação (valor de hoje)
pdf-secao-inflacao-categorias = Categorias × inflação
pdf-secao-inflacao-categorias-dica = Média dos três primeiros meses com gasto e dos três últimos. Acima da inflação, a categoria ficou mais cara de verdade (ou o gasto aumentou); abaixo, ficou mais barata.
pdf-col-inicio = Início
pdf-col-fim = Fim
pdf-col-variacao = Variação
pdf-col-inflacao = Inflação
pdf-col-real = Real
pdf-acima = { $pp } acima
pdf-abaixo = { $pp } abaixo
pdf-igual = igual
pdf-secao-patrimonio = Patrimônio líquido, 12 meses
pdf-secao-vencimentos = Próximos vencimentos (30 dias)
pdf-sem-vencimentos = Nada vencendo nos próximos 30 dias.
pdf-rodape = Gerado no seu computador pelo Treasury em { $data }. Valores em { $moeda }.
pdf-pagina = Página { $n } de { $total }
