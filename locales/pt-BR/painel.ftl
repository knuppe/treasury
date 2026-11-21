# Tela 01 · Painel: cabeçalho, cards de instrumento (patrimônio, fluxo, orçamento, carteira,
# categorias, vencimentos, sem categoria) e "O que o Treasury percebeu"
# (docs/i18n-design.md). pt-BR é o idioma de origem.

## Cabeçalho

painel-atualizado = Atualizado { $data }
painel-descricao-membro = As finanças de { $nome }: os lançamentos, contas, bens e dívidas que são dela ou dele (contas conjuntas pela parte de cada um).
painel-descricao-simples = Quanto entrou, quanto saiu e o que sobrou no mês.
painel-voltar-simples = Voltar ao painel simples
painel-carregando = Medindo o painel
painel-ver-completo = Ver painel completo (patrimônio e carteira)

## Rótulos repetidos nos cards

painel-base = Base
painel-base-caixa = caixa
painel-base-competencia = competência
painel-fonte = Fonte
painel-data = Data
painel-periodo = Período
painel-cambio = Câmbio
painel-cambio-estimado = ≈ estimado

## 01 · Patrimônio líquido

painel-patrimonio = Patrimônio líquido
painel-patrimonio-fonte = Contas + carteira + bens
painel-contas = Contas
painel-patrimonio-origem = Saldos das contas, valor de mercado da carteira e valor dos bens
painel-patrimonio-estimado = Inclui valores estimados: câmbio do dia anterior (PTAX ainda não publicada), cotações ou bens avaliados por estimativa.
painel-patrimonio-em = Patrimônio líquido em { $data }
painel-desde = desde { $mes }
painel-patrimonio-historico = Patrimônio líquido nos últimos { $n } meses
painel-investimentos = Investimentos
painel-bens = Bens
painel-dividas = Dívidas
# O trecho entre ** sai em negrito.
painel-sem-cambio = **Sem câmbio para { $moedas }**: { $contas } ficou fora do total. Cadastre a taxa em Ajustes › Fontes de dados.

## 02 · Fluxo do mês

painel-fluxo = Fluxo do mês
painel-receitas = Receitas
painel-receitas-de = Receitas de { $mes }
painel-despesas = Despesas
painel-despesas-de = Despesas de { $mes }
painel-saldo-mes = Saldo do mês
painel-taxa-poupanca = Taxa de poupança
painel-taxa-poupanca-de = Taxa de poupança de { $mes }
painel-media-12m = Média de 12 meses
painel-taxa-poupanca-historico = Taxa de poupança mês a mês, { $n } meses

## 03 · Orçamento do mês

painel-orcamento = Orçamento do mês
painel-orcamento-carregando = Carregando o orçamento…
painel-orcamento-abrir = Abrir orçamento
painel-orcamento-sem = Sem orçamento para { $mes }.
painel-orcamento-gasto = Gasto do mês: { $valor }. Defina os valores ou copie do mês anterior.
painel-orcamento-de-valor = de { $valor }
painel-orcamento-de = Orçamento de { $mes }
painel-orcamento-ritmo = { $pct } do previsto · { $decorrido } do mês
painel-orcamento-comprometido = Comprometido (previstos): { $valor }

## 04 · Carteira

painel-real = Real
painel-nominal = nominal
painel-carteira-carregando = Carregando a carteira…
painel-carteira-vazia = Nenhuma posição cadastrada.
painel-carteira-cadastrar = Cadastrar operações
painel-valor-mercado = Valor de mercado
painel-carteira-origem = Cotações da carteira
painel-sem-cotacao = Sem cotação: { $ativos }
painel-resultado-total = resultado total
painel-xirr-nominal = XIRR nominal
painel-xirr-real = XIRR real
painel-xirr-real-dica = Chega com os índices de inflação

## 05 · Principais categorias

painel-categorias = Principais categorias
painel-despesas-mes = Despesas do mês
painel-categorias-vazio = Nenhuma despesa no mês.
painel-categorias-lista = Categorias com mais gasto no mês
painel-categoria-ver = { $categoria }: ver lançamentos

## 06 · Próximos vencimentos

painel-vencimentos = Próximos vencimentos
painel-janela = Janela
painel-30-dias = 30 dias
painel-saidas-previstas = Saídas previstas
painel-itens = Itens
painel-vencimentos-vazio = Nada vence nos próximos 30 dias.
painel-hoje = hoje
painel-amanha = amanhã
painel-em-dias = em { $n } d
painel-fatura = Fatura
painel-previsto = Previsto
painel-mais-itens =
    { $n ->
        [1] + { $n } item nos 30 dias
       *[other] + { $n } itens nos 30 dias
    }

## 07 · Sem categoria

painel-para-revisar = Lançamentos para revisar
painel-tudo-categorizado = ✓ tudo categorizado
painel-fora-categorias = ficam fora das categorias nos relatórios
painel-categorizar = Categorizar agora

## O que o Treasury percebeu (percepções calculadas no app, sem IA)

painel-percepcoes = O que o Treasury percebeu
painel-percepcoes-sem-ia = calculado no seu computador, sem IA
painel-inflacao-grupos = Inflação pessoal por grupo
painel-inflacao-grupo = Grupo do IPCA
painel-inflacao-peso = Peso nos seus gastos
painel-inflacao-subiu = Subiu
painel-esconder-grupos = Esconder grupos
painel-ver-grupos = Ver por grupo
painel-abrir = Abrir
painel-dispensar = Dispensar
painel-mostrar-menos = Mostrar menos
painel-ver-todas = Ver as { $n }
