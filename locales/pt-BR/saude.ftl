# Saúde financeira: card 08 do Painel, Relatórios › Saúde financeira e Ajustes › Saúde
# financeira (docs/i18n-design.md). Faixas, dimensões, indicadores, notas e sugestões vêm do
# núcleo. pt-BR é o idioma de origem.

## Modelos e estados

saude-modelo-classicos = Indicadores clássicos
saude-modelo-desativado = Desativado
saude-status-bom = Bom
saude-status-atencao = Atenção
saude-status-ruim = Ruim
saude-status-sem-dados = Sem dados

## Régua e barras (nomes para leitor de tela)

saude-faixas-aria = Faixas de 0 a 100: { $faixas }
saude-barra-sem-dados = { $rotulo }: sem dados
saude-barra-oculto = { $rotulo }: valor oculto
saude-barra-pontos = { $rotulo }: { $pontos } de 100

## Card do painel

saude-titulo = Saúde financeira
saude-modelo = Modelo
saude-janela = Janela
saude-n-meses =
    { $n ->
        [1] { $n } mês
       *[other] { $n } meses
    }
saude-avaliado = Avaliado
saude-ver-indicadores = Ver indicadores
saude-nota-isf = Nota (estimativa I-SFB)
saude-nota = Nota
saude-origem-isf = Estimativa no estilo do I-SFB (Febraban e Banco Central) com os seus lançamentos, parcelas, faturas e saldos
saude-origem-classicos = Indicadores clássicos calculados com os seus lançamentos, parcelas, faturas e saldos
saude-dimensoes = Dimensões
saude-indicadores = Indicadores

## Relatórios › Saúde financeira

saude-trocar-modelo = Trocar o modelo
saude-carregando = Medindo a saúde financeira
saude-desativada = Desativada
saude-desligada-titulo = Análise de saúde financeira desligada
saude-desligada-descricao = Escolha um modelo em Ajustes › Saúde financeira para ver a nota, as dimensões e as sugestões.
saude-abrir-ajustes = Abrir ajustes
saude-indicadores-descricao = Cada razão com a meta usual da educação financeira e o peso na nota.
saude-titulo-modelo = Saúde financeira · { $modelo }
saude-periodo = { $de } a { $ate }
saude-sem-lancamentos = sem lançamentos
saude-meses = Meses
saude-cambio = Câmbio
saude-cambio-estimado = ≈ estimado
saude-contas-paradas = Contas
saude-contas-paradas-valor = { $n ->
    [one] ≈ 1 sem extrato recente
   *[other] ≈ { $n } sem extrato recente
}
saude-nota-0-100 = Nota de 0 a 100
saude-faixas = Faixas
saude-sobre-isf = Sobre o I-SFB (Meu Bolso em Dia, Febraban)
saude-medido = Medido
saude-sem-dados = sem dados
saude-meta = Meta
saude-pontos = Pontos
saude-peso-nota = · peso { $peso }% da nota
saude-peso-dimensao = · peso { $peso }% da dimensão
saude-calculo = Cálculo
saude-para-medir = Para medir
saude-sugestoes = Sugestões

## Ajustes › Saúde financeira

saude-desativada-aviso = Saúde financeira desativada
saude-modelo-aviso = Modelo: { $modelo }
saude-modelo-titulo = Modelo de saúde financeira
saude-como-calculada = Como a nota é calculada
saude-calculo-dados = A nota usa só os dados do cofre: receitas e despesas dos últimos 12 meses completos (transferências, pagamento de fatura e aportes não contam), parcelas de empréstimos, faturas de cartão, saldos das contas, orçamento e patrimônio. Nada sai do computador.
# O trecho entre ** sai em itálico.
saude-calculo-sem-dados = Indicador sem histórico suficiente (menos de 3 meses) fica **sem dados** e sai da média, em vez de puxar a nota para baixo.
saude-isf-resumo = comportamento, habilidade, liberdade e segurança; média nacional 56,7 (2024)
saude-classicos = Clássicos
saude-classicos-resumo = poupança, reserva, parcelas, moradia, rotativo e patrimônio
saude-fonte-isf = Índice de Saúde Financeira do Brasileiro 2024 (Meu Bolso em Dia)
