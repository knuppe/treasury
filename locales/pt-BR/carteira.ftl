# Tela 08 · Carteira: abas, resumo, posições, rentabilidade, alocação e rebalanceamento,
# operações, eventos corporativos, proventos, renda passiva, custo das taxas e risco, mais os
# rótulos das classes, dos proventos e das fontes usados também em outras telas
# (docs/i18n-design.md). As importações da B3 e das notas de corretagem ficam para a fase 5.
# pt-BR é o idioma de origem.

## Abas e cabeçalho

carteira-secoes = Seções da carteira
carteira-aba-posicoes = Posições
carteira-aba-rentabilidade = Rentabilidade
carteira-aba-alocacao = Alocação e rebalanceamento
carteira-aba-operacoes = Operações
carteira-aba-proventos = Proventos
carteira-aba-renda = Renda passiva
carteira-aba-taxas = Custo das taxas
carteira-aba-risco = Risco
carteira-aba-saude = Saúde dos ativos
carteira-aba-bens = Bens
carteira-carregando = Calculando a carteira
carteira-meta = Posição em { $data } · real por { $deflator }
carteira-evento-corporativo = Evento corporativo
carteira-registrar-proventos = Registrar proventos
carteira-provento-detalhado = Provento detalhado
carteira-nova-operacao = Nova operação
carteira-importar-b3 = Importar da B3
carteira-importar-notas = Importar notas
carteira-real-descontado-por = Resultado real descontado por
carteira-eventos-corporativos = Eventos corporativos

## Carteira vazia

carteira-vazia-kicker = Carteira vazia
carteira-vazia-titulo = Registre a primeira operação
carteira-vazia-texto = Cadastre o ativo em 09 · Ativos (ou escolha um já cadastrado) e registre a compra com custos. A carteira calcula preço médio, resultado, XIRR e o retorno real descontado pela inflação.
carteira-cadastrar-ativo = Cadastrar ativo

## Ativos avaliados pelo custo

carteira-avaliados-pelo-custo =
    { $n ->
        [1] 1 ativo avaliado
       *[other] { $n } ativos avaliados
    } pelo custo: { $lista }
carteira-lista-e-mais = { $lista } e mais { $n }
carteira-avaliados-texto = Sem cotação, o valor de mercado é o custo e o resultado não realizado fica zerado. Atualize as cotações em { $link } (Yahoo Finance e Tesouro Transparente) ou informe a cotação manualmente na página do ativo. Fundos (cotas da CVM) chegam numa próxima etapa.
carteira-link-indices = 09 · Ativos › Índices e cotações

## Barra de comando

carteira-cmd-evento-busca = desdobramento,grupamento,bonificação,split,incorporação
carteira-cmd-ver-aba = Ver { $aba }
carteira-cmd-ver-aba-busca = aba,mostrar
carteira-cmd-operacao = Nova operação (compra ou venda)
carteira-cmd-operacao-busca = registrar,comprar,vender,trade,nota de corretagem,investimento
carteira-cmd-provento = Registrar provento
carteira-cmd-provento-busca = dividendos,jcp,rendimento,fii,cupom
carteira-cmd-proventos = Registrar vários proventos (data, ticker e valor)
carteira-cmd-proventos-busca = proventos,dividendos,jcp,rendimentos,grade,lote,colar
carteira-cmd-importar-b3 = Importar da B3 (Área do Investidor)
carteira-cmd-importar-b3-busca = b3,área do investidor,negociação,movimentação,posição,excel
carteira-cmd-importar-notas = Importar notas de corretagem (PDF)
carteira-cmd-importar-notas-busca = nota de corretagem,sinacor,pdf,taxas,emolumentos,irrf
carteira-cmd-ativo-busca = novo ativo,ação,fii,etf,tesouro,cdb,renda fixa,cripto
carteira-cmd-cotacoes = Atualizar índices, câmbio e cotações
carteira-cmd-cotacoes-busca = ipca,cdi,ptax,dólar,sincronizar,yahoo,coletar
carteira-cmd-agenda = Agenda de dividendos
carteira-cmd-agenda-busca = data-com,pagamento,proventos anunciados
carteira-cmd-watchlist = Watchlist
carteira-cmd-watchlist-busca = favoritos,acompanhar,preço-alvo,alertas
carteira-cmd-rankings = Rankings de ativos
carteira-cmd-rankings-busca = dividend yield,p/l,graham,bazin,maiores,ranking
carteira-cmd-screener = Screener (rastreador de ativos)
carteira-cmd-screener-busca = filtro,telas,rastreador,indicadores
carteira-cmd-comparador = Comparar ativos
carteira-cmd-comparador-busca = comparador,lado a lado,base 100
carteira-cmd-analise = Coletar dados da análise (CVM, B3, SEC)
carteira-cmd-analise-busca = fundamentos,cvm,sec,dfp,coletar,sincronizar
carteira-cmd-grupo-ativos = Investimentos · ativos
carteira-cmd-abrir-ativo-busca = abrir ativo,cotação,gráfico

## Estados comuns

carteira-falha-desconhecida = Falha desconhecida.
carteira-erro-carregar = Não foi possível carregar
carteira-excluir = Excluir
carteira-total = Total
carteira-total-valor = Total: { $valor }
carteira-opcional = Opcional.
carteira-valor-invalido = Valor inválido.
carteira-escolha-ativo = Escolha o ativo
carteira-escolha-conta = Escolha a conta
carteira-nenhuma-corretora = Nenhuma corretora cadastrada
carteira-n-meses = { $n } meses
carteira-no-ano = No ano
carteira-periodo = Período
carteira-acoes-de = Ações de { $nome }
carteira-e = e
carteira-pp = p.p.
carteira-valor-pp = { $valor } p.p.
carteira-pct-aa = { $pct } a.a.
carteira-data-formato = dd/mm/aaaa

## Classes de ativo (os mesmos nomes do núcleo)

carteira-classe-stock-br = Ações
carteira-classe-fii = FIIs
carteira-classe-etf-br = ETFs
carteira-classe-bdr = BDRs
carteira-classe-fixed-income = Renda fixa
carteira-classe-treasury-br = Tesouro Direto
carteira-classe-fund = Fundos
carteira-classe-stock-us = Ações nos EUA
carteira-classe-etf-us = ETFs nos EUA
carteira-classe-reit = REITs
carteira-classe-crypto = Cripto
carteira-classe-pension = Previdência
carteira-classe-other = Outros
carteira-classe-stock-intl = Ações em outras bolsas
carteira-classe-etf-intl = ETFs em outras bolsas

## Tipos de provento e de evento corporativo

carteira-provento-dividend = Dividendos
carteira-provento-fii-income = Rendimentos de FII
carteira-provento-coupon = Cupom
carteira-provento-interest = Juros
carteira-provento-rent = Aluguel de ações
carteira-provento-amortization = Amortização
carteira-provento-other = Outros proventos

carteira-evento-split = Desdobramento
carteira-evento-reverse-split = Grupamento
carteira-evento-bonus = Bonificação
carteira-evento-subscription = Subscrição
carteira-evento-merger = Incorporação
carteira-evento-ticker-change = Mudança de ticker
carteira-evento-spin-off = Cisão
carteira-evento-capital-return = Devolução de capital

## Deflator do resultado real

carteira-deflator-rotulo = Deflator do resultado real
carteira-deflator-ipca-dica = Inflação oficial do Brasil (IBGE, via BCB/SGS 433)
carteira-deflator-pais = Inflação do país
carteira-deflator-pais-de = Inflação do país ({ $pais })
carteira-deflator-pais-dica = Inflação oficial do país de referência (Ajustes › Fontes de dados)
carteira-deflator-pais-serie = { $pais }: { $serie } ({ $fonte }). Troque o país em Ajustes › Fontes de dados.
carteira-deflator-us-cpi-dica = Inflação ao consumidor dos EUA (FRED CPIAUCSL)
carteira-deflator-m2-br-dica = Expansão monetária do Brasil (BCB/SGS 27810)
carteira-deflator-m2-us = M2 EUA
carteira-deflator-m2-us-dica = Expansão monetária dos EUA (FRED M2SL)
carteira-deflator-nominal = Nominal
carteira-deflator-nominal-dica = Sem descontar inflação
carteira-fonte-oficial = fonte oficial

## Renda fixa

carteira-rf-debenture = Debênture
carteira-rf-debenture-incentivada = Debênture incentivada
carteira-rf-poupanca = Poupança
carteira-rf-outro = Outro
carteira-indexador-prefixado = Prefixado
carteira-liquidez-diaria = Diária
carteira-liquidez-vencimento = No vencimento
carteira-liquidez-carencia = Após a carência
carteira-taxa-dica-percentual = % do { $indice } (ex.: 110 para 110% do { $indice })
carteira-taxa-dica-prefixado = Taxa ao ano (ex.: 12,5 para 12,5% a.a.)
carteira-taxa-dica-real = Taxa real ao ano acima do { $indice } (ex.: 6,2 para { $indice } + 6,2% a.a.)
carteira-termos-percentual = { $taxa }% do { $indice }
carteira-termos-pre = Pré { $taxa }% a.a.
carteira-termos-indice = { $indice } + { $taxa }% a.a.
carteira-termos-vencimento = venc. { $data }

## Avaliação e fontes

carteira-avaliacao-mercado = Cotação de mercado
carteira-avaliacao-curva = Marcação na curva
carteira-avaliacao-sem-cotacao = Sem cotação (pelo custo)
carteira-fonte-b3-listados = B3 (proventos oficiais)
carteira-fonte-yahoo-intraday = Yahoo Finance (intradiário, ~15 min de atraso)
carteira-fonte-manual-voce = Informado por você
carteira-fonte-mock = Dados de exemplo
carteira-fonte-radar = Radar de dividendos (projeção)
carteira-fonte-bcb = Banco Central
carteira-fonte-oecd = OCDE
carteira-fonte-worldbank = Banco Mundial (anual, interpolado)
carteira-fonte-bce = BCE
carteira-fonte-bce-ref = BCE (referência)
carteira-fonte-nao-informada = Fonte não informada
carteira-fonte-manual = Manual
carteira-fonte-sem-cotacao = Sem cotação
carteira-fonte-na-curva = Na curva
carteira-fonte-manual-sem-fonte = Manual (sem fonte automática)
carteira-fonte-manual-bolsa = Manual (informe a bolsa)

## Cotação e medidor nominal × real

carteira-cotacao-intradia = Cotação de { $quando }, com cerca de { $min } min de atraso; { $sessao }. O histórico usa os fechamentos diários.
carteira-cotacao-defasada = Cotação defasada: último fechamento conhecido antes da data da posição.
carteira-cotacao-ultimo-pregao = Último pregão fechado antes da data da posição.
carteira-cotacao-fechamento = Fechamento do pregão.
carteira-na-curva = na curva
carteira-curva-fonte = Marcação na curva (CDI/IPCA do BCB)
carteira-curva-nota = Renda fixa avaliada pela taxa contratada sobre o indexador, base 252 dias úteis.
carteira-sem-cotacao = sem cotação
carteira-sem-cotacao-dica = Sem cotação: o valor é o custo. Atualize as cotações ou informe a cotação manualmente na página do ativo.
carteira-medidor = Nominal { $nominal }; real ({ $deflator }) { $real }
carteira-medidor-oculto = Nominal × real ({ $deflator }): valores ocultos

## Resumo (instrumentos do topo)

carteira-resumo-fonte-sem-cotacao = Cotações, curva e custo ({ $n } sem cotação)
carteira-resumo-fonte = Cotações de fechamento e marcação na curva
carteira-resumo-estimado = Há câmbio, índice ou cotação estimados (ou ativos avaliados pelo custo) nesta conta.
carteira-valor-mercado = Valor de mercado
carteira-custo = Custo
carteira-em = Em
carteira-patrimonio-investido = Patrimônio investido
carteira-nao-realizado-pct = não realizado { $pct }
carteira-resultado = Resultado
carteira-nao-realizado = Não realizado
carteira-realizado = Realizado
carteira-proventos = Proventos
carteira-resultado-total = Resultado total
carteira-resultado-perspectiva = Perspectiva do resultado
carteira-resultado-nominal = Nominal
carteira-resultado-real-de = Resultado real ({ $indice })
carteira-resultado-efeito = Efeito do { $indice }
carteira-resultado-sem-indice = Sem o { $indice } do período: falta coletar o índice
carteira-sobre-custo = sobre o custo · { $pct }
carteira-xirr-titulo = Retorno anual (XIRR)
carteira-xirr-sem-dados-ipca = Sem dados de { $deflator } até { $data }: atualize os índices em 09 · Ativos › Índices e cotações.
carteira-xirr-sem-dados = Sem dados de { $deflator } até { $data }: o índice sai com atraso e o núcleo ainda não estima os meses que faltam.
carteira-deflator = Deflator
carteira-metodo = Método
carteira-metodo-xirr = XIRR (datas e valores de cada fluxo)
carteira-nominal = Nominal
carteira-real-de = Real · { $deflator }
carteira-renda-12m = Renda passiva 12m
carteira-media-mensal = Média mensal
carteira-yield = Yield
carteira-proventos-liquidos = Proventos líquidos
carteira-recebidos-moeda = recebidos, em { $moeda }
carteira-carregando-curto = carregando

## Posições

carteira-avisos = Avisos: { $lista }
carteira-liquido-dica = Valor líquido estimado num resgate hoje: IR regressivo (22,5% até 180 dias → 15% acima de 720) sobre o ganho e IOF nos primeiros 30 dias; isentos (LCI, LCA, CRI, CRA, debêntures incentivadas) não pagam. Estimativa, não a apuração de IR.
carteira-liquido-abrev = líq. { $valor }
carteira-col-ativo = Ativo
carteira-col-classe = Classe
carteira-col-qtd = Qtd.
carteira-col-preco-medio = Preço médio
carteira-col-cotacao = Cotação
carteira-col-valor = Valor
carteira-col-peso = Peso
carteira-col-resultado = Resultado
carteira-col-xirr = XIRR a.a.
carteira-col-nominal-real = Nom. × real
carteira-n-ativos =
    { $n ->
        [1] { $n } ativo
       *[other] { $n } ativos
    }
carteira-posicoes-de = Posições · { $grupo }
carteira-agrupar = Agrupar posições
carteira-por-classe = Por classe
carteira-por-conta = Por conta
carteira-sem-grupo = Sem grupo
carteira-incluir-encerradas = Incluir posições encerradas
carteira-posicoes-atalhos = Enter ou duplo clique abre o ativo; o botão direito mostra os atalhos.
carteira-calculando-posicoes = Calculando posições
carteira-sem-posicoes = Nenhuma posição ainda
carteira-sem-posicoes-texto = Registre a primeira compra para ver preço médio, resultado e retorno real de cada ativo.
carteira-conta = Conta
carteira-todas-contas = Todas as contas
carteira-todas-posicoes = Todas as posições

## Rentabilidade

carteira-desde-inicio = Desde o início
carteira-calculando-rentabilidade = Calculando rentabilidade
carteira-sem-historico = Sem histórico para calcular
carteira-sem-historico-texto = A rentabilidade aparece depois da primeira operação.
carteira-fonte-catalogo = { $indice } (catálogo de índices)
carteira-rent-fonte = Fonte: operações, cotações e proventos · { $taxa } · { $deflator }
carteira-rent-estimado = Meses de inflação ainda não divulgados usam a projeção Focus (ou o último dado) até o índice oficial sair.
carteira-desde = Carteira desde { $mes }
carteira-pct-do = % do { $indice }
carteira-acumulado-real = Acumulado real
carteira-serie-carteira = Carteira
carteira-serie-real = Carteira real ({ $deflator })
carteira-serie-sp500 = { $moeda ->
        [BRL] S&P 500 (em R$)
       *[other] S&P 500 (em { $moeda })
    }
carteira-serie-dolar = Dólar
carteira-mes = Mês
carteira-rent-acumulada-titulo = Rentabilidade acumulada × { $indice } × inflação
carteira-rent-acumulada-aria-desde = Rentabilidade acumulada da carteira desde { $mes }, comparada ao { $indice } e à inflação.
carteira-rent-acumulada-aria-inicio = Rentabilidade acumulada da carteira desde o início, comparada ao { $indice } e à inflação.
carteira-rent-acumulada-tabela = Rentabilidade acumulada por mês
carteira-retorno-mes = Retorno do mês
carteira-retorno-cada-mes = Retorno de cada mês
carteira-retorno-metodo = Método: TWR mensal (Modified Dietz), neutraliza aportes e resgates
carteira-retorno-aria = Retorno mensal da carteira em barras, ganhos acima e perdas abaixo de zero.
carteira-retorno-mensal = Retorno mensal
carteira-retorno = Retorno
carteira-patrimonio = Patrimônio
carteira-aportes-liquidos = Aportes líquidos
carteira-patrimonio-aportes = Patrimônio × aportes líquidos
carteira-patrimonio-aportes-fonte = Aportes líquidos = compras − vendas, acumulados; a distância é o ganho
carteira-patrimonio-aportes-aria = Patrimônio investido comparado aos aportes líquidos acumulados, por mês.
carteira-patrimonio-aportes-tabela = Patrimônio e aportes por mês

## Gráfico de preço (página do ativo)

carteira-grafico-pm = PM
carteira-grafico-fechamento = Fechamento
carteira-grafico-preco-operacao = Preço da operação
carteira-marcador-compra = C { $qtd }
carteira-marcador-venda = V { $qtd }

## Alocação e rebalanceamento

carteira-outras = Outras
carteira-outras-de = Outras ({ $classes })
carteira-investido = investido
carteira-sem-alvo = sem alvo
carteira-alvo-pct = alvo { $pct }
carteira-alocacao-classe = Alocação por classe
carteira-alocacao-fonte = Valor de mercado (ou na curva) em { $moeda }; sem cotação, pelo custo
carteira-alocacao-aria = Alocação da carteira por classe de ativo, em donut.
carteira-col-alvo = Alvo
carteira-col-desvio = Desvio
carteira-desvio-peso-alvo = peso { $peso } · alvo { $alvo }
carteira-desvio-pp = desvio { $desvio } p.p.
carteira-fora-tolerancia = fora da tolerância
carteira-desvio-titulo = Desvio do alvo, em pontos percentuais
carteira-desvio-fonte = Barras cheias: fora da tolerância definida; claras: dentro dela
carteira-desvio-aria = Desvio de cada classe em relação à alocação-alvo.
carteira-desvio-tabela = Desvio do alvo
carteira-alvos-salvos = Alocação-alvo salva
carteira-alvos-titulo = Alocação-alvo por classe
carteira-alvos-soma-passa = Soma { $total }% · passa de 100%
carteira-alvos-soma-completa = Soma { $total }% · completa
carteira-alvos-soma-faltam = Soma { $total }% · faltam { $falta } p.p.
carteira-descartar = Descartar
carteira-salvar-alvos = Salvar alvos
carteira-alvos-tabela = Alvos por classe
carteira-col-atual = Atual
carteira-col-alvo-pct = Alvo %
carteira-col-tolerancia = Tol. p.p.
carteira-alvo-de = Alvo de { $classe } (%)
carteira-tolerancia-de = Tolerância de { $classe } (p.p.)
carteira-adicionar-classe = Adicionar classe
carteira-adicionar-classe-aria = Adicionar classe à alocação-alvo
carteira-escolha-classe = Escolha a classe
carteira-tolerancia-texto = Tolerância: quantos pontos percentuais o peso pode se afastar do alvo antes de a classe ficar “fora da banda” (padrão 5).
carteira-fora-tolerancia-alvo = Fora da tolerância do alvo
carteira-fora-tolerancia-alvo-curto = fora da tolerância do alvo
carteira-col-atual-pct = Atual %
carteira-col-falta-sobra = Falta/sobra
carteira-col-do-aporte = Do aporte
carteira-rebal-titulo = Rebalanceamento com novo aporte
carteira-novo-aporte = Novo aporte
carteira-novo-aporte-dica = 0 = só mostra o desvio. Falta/sobra: + falta comprar, − está acima do alvo.
carteira-carteira-hoje = Carteira hoje
carteira-depois-aporte = Depois do aporte
carteira-alvos-incompletos = Os alvos não somam 100%
carteira-alvos-incompletos-texto = A sugestão distribui o aporte só entre as classes com alvo; ajuste a alocação-alvo acima para uma conta completa.
carteira-aporte-sem-destino = Parte do aporte não cabe nos limites
carteira-aporte-sem-destino-texto = ficam sem destino: os ativos chegaram ao limite de concentração. Aumente um limite ou escolha um ativo novo.
carteira-defina-alvos = Defina a alocação-alvo para ver quanto falta ou sobra em cada classe.
carteira-rebal-tabela = Rebalanceamento por classe
carteira-nao-recomendacao = Não é recomendação de investimento
carteira-rebal-aviso = Análise baseada nos seus dados e nos alvos que você definiu; não é recomendação de investimento.
carteira-rebal-aviso-sugestao = A sugestão só diz como o aporte aproxima a carteira dos alvos, sem vender nada.
carteira-calculando-alocacao = Calculando alocação

## Operações, eventos e proventos (tabelas)

carteira-compra = Compra
carteira-venda = Venda
carteira-compras = Compras
carteira-vendas = Vendas
carteira-compra-de = Compra de { $qtd } { $ativo } em { $data }
carteira-venda-de = Venda de { $qtd } { $ativo } em { $data }
carteira-compra-qtd = Compra de { $qtd } { $ativo }
carteira-venda-qtd = Venda de { $qtd } { $ativo }
carteira-erro-excluir = Não foi possível excluir
carteira-excluir-n =
    { $n ->
        [1] Excluir
       *[other] Excluir { $n }
    }
carteira-registros-excluir = Registros a excluir
carteira-e-mais = e mais { $n }
carteira-menu-editar = Editar…
carteira-menu-copiar-linha = Copiar linha
carteira-menu-excluir = Excluir…
carteira-menu-copiar-n-linhas = Copiar { $n } linhas
carteira-excluir-operacoes =
    { $n ->
        [1] Excluir operação
       *[other] Excluir { $n } operações
    }
carteira-excluir-operacoes-nota =
    { $n ->
        [1] O lançamento ligado no caixa da corretora também sai. Dá para desfazer pelo histórico de alterações.
       *[other] Os lançamentos ligados no caixa da corretora também saem. Dá para desfazer todas de uma vez pelo histórico de alterações.
    }
carteira-operacoes-excluidas =
    { $n ->
        [1] Operação excluída
       *[other] { $n } operações excluídas
    }
carteira-operacoes-selecionadas = { $n } operações selecionadas
carteira-excluir-n-operacoes = Excluir { $n } operações…
carteira-acoes-de-n-operacoes = Ações de { $n } operações
carteira-n-operacoes = { $n } operações
carteira-col-data = Data
carteira-col-tipo = Tipo
carteira-col-preco = Preço
carteira-col-custos = Custos
carteira-col-bruto = Bruto
carteira-col-caixa = Caixa
carteira-fora-do-caixa = fora do caixa
carteira-col-cambio = Câmbio
carteira-col-conta = Conta
carteira-col-nota = Nota
carteira-col-acoes = Ações
carteira-editar-operacao-de = Editar operação de { $data }
carteira-filtrar-ativo = Filtrar por ativo
carteira-todos-ativos = Todos os ativos
carteira-filtrar-conta = Filtrar por conta
carteira-operacoes = Operações
carteira-sem-operacoes = Nenhuma operação registrada.
carteira-evento-texto = { $tipo } de { $ativo } em { $data }
carteira-excluir-eventos =
    { $n ->
        [1] Excluir evento corporativo
       *[other] Excluir { $n } eventos corporativos
    }
carteira-excluir-eventos-nota = Quantidade e preço médio são recalculados. Dá para desfazer pelo histórico de alterações.
carteira-eventos-excluidos =
    { $n ->
        [1] Evento excluído
       *[other] { $n } eventos excluídos
    }
carteira-eventos-selecionados = { $n } eventos selecionados
carteira-excluir-n-eventos = Excluir { $n } eventos…
carteira-acoes-de-n-eventos = Ações de { $n } eventos
carteira-col-data-ex = Data ex
carteira-col-evento = Evento
carteira-col-detalhe = Detalhe
carteira-fator = fator { $fator }
carteira-fator-custo = fator { $fator } · custo { $custo }
carteira-col-observacoes = Observações
carteira-col-origem = Origem
carteira-registrado-por-voce = Registrado por você
carteira-coleta-cotacoes = { $fonte } (coleta de cotações)
carteira-evento-coletado-nota = Desdobramento informado pela fonte de cotações; desfazível no histórico de alterações.
carteira-sem-eventos = Nenhum evento corporativo.
carteira-excluir-proventos =
    { $n ->
        [1] Excluir provento
       *[other] Excluir { $n } proventos
    }
carteira-excluir-proventos-nota =
    { $n ->
        [1] O crédito ligado no caixa também sai. Dá para desfazer pelo histórico de alterações.
       *[other] Os créditos ligados no caixa também saem. Dá para desfazer todos de uma vez pelo histórico de alterações.
    }
carteira-proventos-excluidos =
    { $n ->
        [1] Provento excluído
       *[other] { $n } proventos excluídos
    }
carteira-proventos-selecionados = { $n } proventos selecionados
carteira-excluir-n-proventos = Excluir { $n } proventos…
carteira-acoes-de-n-proventos = Ações de { $n } proventos
carteira-col-pagamento = Pagamento
carteira-col-data-com = Data-com
carteira-col-base = Base
carteira-col-ir-retido = IR retido
carteira-col-liquido = Líquido
carteira-col-situacao = Situação
carteira-recebido = Recebido
carteira-anunciado = Anunciado
carteira-sem-proventos-periodo = Nenhum provento no período.
carteira-recebido-moeda = Recebido ({ $moeda })
carteira-provento-texto = { $tipo } de { $ativo } pago em { $data }

## Proventos e renda passiva (painéis)

carteira-col-12-meses = 12 meses
carteira-col-yield-12m = Yield 12m
carteira-somando-proventos = Somando proventos
carteira-ultimos-12-meses = Últimos 12 meses
carteira-por-mes = Por mês
carteira-yield-carteira = Yield da carteira
carteira-yield-formula = Proventos ÷ valor de mercado
carteira-meta-renda = Meta de renda mensal
carteira-patrimonio-necessario = Patrimônio necessário
carteira-falta = Falta
carteira-quanto-receber = Quanto quer receber por mês
carteira-progresso-meta = Progresso da meta
carteira-sem-yield = Sem yield para calcular o patrimônio necessário
carteira-sem-yield-texto = A carteira ainda não recebeu proventos nos últimos 12 meses.
carteira-por-ativo-12m = Por ativo · 12 meses
carteira-renda-por-ativo = Renda passiva por ativo
carteira-sem-proventos-12m = Nenhum provento nos últimos 12 meses.
carteira-patrimonio-necessario-texto = Patrimônio necessário = meta × 12 ÷ yield dos últimos 12 meses. É uma conta sobre o passado da sua carteira, não uma promessa de renda.
carteira-grafico-proventos-liquidos = Proventos líquidos
carteira-meta-curta = Meta
carteira-meta-valor = Meta { $valor }
carteira-proventos-mes-titulo = Proventos por mês, últimos 12 meses
carteira-proventos-mes-fonte = Proventos recebidos, líquidos de IR retido, em { $moeda } na data de pagamento
carteira-meta-mensal = Meta mensal
carteira-proventos-mes-aria = Proventos líquidos recebidos por mês nos últimos 12 meses.
carteira-proventos-mes-tabela = Proventos por mês

## Custo das taxas

carteira-taxas-titulo = Custo das taxas no longo prazo
carteira-taxas-retorno = Retorno real antes das taxas (% a.a.)
carteira-taxas-alternativa = Taxa do cenário alternativo (% a.a.)
carteira-taxas-aporte = Aporte mensal (opcional)
carteira-taxas-nenhum = Nenhum fundo ou plano com taxa conhecida
carteira-taxas-nenhum-texto = Cadastre a taxa de administração dos planos de previdência; a dos fundos vem do extrato na CVM (coleta da análise de ativos).
carteira-taxas-tabela = Custo das taxas
carteira-taxas-fundo = Fundo ou plano
carteira-taxas-hoje = Hoje
carteira-taxas-em-anos = Em { $anos } anos
carteira-taxas-economia = +{ $valor } com taxa menor
carteira-taxa-administracao = administração { $pct }
carteira-taxa-performance = performance { $pct }
carteira-taxa-carregamento = carregamento { $pct }
carteira-taxas-sem-taxa = Sem taxa conhecida (fora da conta): { $lista }.

## Risco

carteira-risco-sem-queda = sem queda
carteira-risco-queda-inicio = { $pct } do início a { $fundo }
carteira-risco-queda = { $pct } de { $pico } a { $fundo }
carteira-risco-recuperou = { $queda }, recuperou em { $mes }
carteira-risco-nao-recuperou = { $queda }, ainda não recuperou
carteira-risco-volatilidade = Volatilidade
carteira-risco-pior-queda = Pior queda
carteira-risco-retorno = Retorno
carteira-risco-sharpe-de = Sharpe ({ $indice })
carteira-risco-melhor-pior = Melhor e pior mês
carteira-risco-insuficiente = Histórico mensal insuficiente.
carteira-risco-retorno-aa = Retorno a.a.
carteira-risco-sharpe = Sharpe
carteira-risco-parte = Parte do risco
carteira-risco-sem-dados = sem dados
carteira-risco-titulo = Risco da carteira
carteira-risco-texto = Quanto a carteira oscila, a maior queda, o retorno acima do { $indice } por unidade de risco e o quanto os ativos andam juntos, pelos retornos mensais de cada ativo com os pesos de hoje.
carteira-risco-calculando = Calculando o risco
carteira-risco-sem-ativos = Nenhum ativo na carteira
carteira-risco-sem-ativos-texto = Registre operações em Carteira › Operações para medir o risco.
carteira-risco-hoje = Carteira de hoje no período
carteira-risco-historico = Histórico real (compras e vendas de cada época)
carteira-risco-por-classe = Risco por classe
carteira-risco-correlacao-classes = Correlação entre classes
carteira-risco-por-ativo = Risco por ativo
carteira-risco-correlacao-ativos = Correlação entre os maiores ativos
carteira-risco-parte-texto = Parte do risco: quanto cada linha responde pela oscilação da carteira (somam 100%). Em amarelo, as que pesam no risco bem mais do que no valor.

## Registrar operação

carteira-op-editar = Editar operação
carteira-op-registrar = Registrar operação
carteira-op-editar-texto = A operação é regravada com os novos valores, junto com o lançamento ligado no caixa da corretora.
carteira-op-registrar-texto = Compra ou venda com custos. O caixa da corretora recebe o lançamento ligado (desfazível no histórico).
carteira-erro-escolha-ativo = Escolha o ativo.
carteira-erro-escolha-conta-corretora = Escolha a conta da corretora.
carteira-erro-data = Informe a data (dd/mm/aaaa).
carteira-erro-quantidade = Informe uma quantidade maior que zero.
carteira-erro-venda-maior = Venda de { $qtd } maior que a posição de { $posicao } nesta conta em { $data }.
carteira-na-data = na data
carteira-erro-preco = Informe o preço unitário.
carteira-erro-cambio = O câmbio deve ser positivo.
carteira-op-alterada = Operação alterada
carteira-op-registrada = Operação registrada
carteira-op-form = Operação
carteira-op-tipo = Tipo de operação
carteira-op-renda-fixa = Renda fixa: quantidade de títulos × PU (preço unitário); vendas saem por PEPS.
carteira-buscar-ativo = Buscar ativo
carteira-buscar-ativo-dica = Ticker ou nome: escolhe um cadastrado ou cadastra um novo (B3, EUA, outras bolsas, cripto).
carteira-ativo = Ativo
carteira-conta-corretora = Conta da corretora
carteira-conta-moeda = Conta em { $conta }; ativo em { $ativo }.
carteira-data-pregao = Data do pregão
carteira-quantidade = Quantidade
carteira-posicao-na-data = Posição nesta conta na data: { $qtd }
carteira-fracoes = Frações com até 8 casas (exterior, cripto, Tesouro).
carteira-preco-unitario = Preço unitário ({ $moeda })
carteira-cambio-moeda = Câmbio { $moeda }/{ $base }
carteira-cambio-dica = { $base ->
        [BRL] 1 unidade da moeda do ativo em reais. Vazio: PTAX do dia guardada no app.
       *[other] 1 unidade da moeda do ativo em { $base }. Vazio: câmbio do dia guardado no app.
    }
carteira-ptax-do-dia = { $base ->
        [BRL] PTAX do dia
       *[other] Câmbio do dia
    }
carteira-custos-corretagem = Custos (corretagem e emolumentos)
carteira-impostos-retidos = Impostos retidos (IRRF)
carteira-nota-corretagem = Nota de corretagem
carteira-observacoes = Observações
carteira-day-trade = Day trade
carteira-day-trade-dica = Compra e venda no mesmo dia (apuração de IR separada na Fase 3).
carteira-lancar-caixa = Lançar no caixa da corretora
carteira-lancar-caixa-dica = Cria o lançamento ligado (aporte/resgate de investimento), que não conta como receita nem despesa.
carteira-lancar-caixa-dica-edicao = Confira antes se o extrato da corretora já trouxe esse débito ou crédito: lançar de novo conta duas vezes no saldo.
carteira-bruto = Bruto
carteira-sai-caixa = Sai do caixa
carteira-entra-caixa = Entra no caixa
carteira-bruto-moeda = Bruto em { $moeda }
carteira-preco = Preço
carteira-salvar-alteracoes = Salvar alterações
carteira-registrar-compra = Registrar compra
carteira-registrar-venda = Registrar venda

## Registrar provento

carteira-prov-titulo = Registrar provento
carteira-prov-texto = Recebido entra no caixa da corretora como receita (Rendimentos › Dividendos/JCP); anunciado fica só na agenda até o pagamento.
carteira-erro-escolha-conta = Escolha a conta.
carteira-erro-data-pagamento = Informe a data de pagamento.
carteira-erro-bruto = Informe o valor bruto (ou o valor por cota).
carteira-erro-imposto = Imposto maior que o bruto.
carteira-prov-registrado = Provento registrado
carteira-prov-anunciado-registrado = Provento anunciado registrado
carteira-prov-form = Provento
carteira-prov-situacao = Situação do provento
carteira-ativos-com-operacoes = Ativos com operações.
carteira-tipo = Tipo
carteira-data-com = Data-com
carteira-pagamento = Pagamento
carteira-valor-por-cota = Valor por cota ({ $moeda })
carteira-valor-por-cota-dica = Opcional: com a quantidade, calcula o bruto.
carteira-quantidade-base = Quantidade base
carteira-na-data-qtd = Na data: { $qtd }
carteira-cotas-data-com = Cotas na data-com.
carteira-valor-bruto = Valor bruto
carteira-bruto-calculado = Calculado: valor por cota × quantidade.
carteira-imposto-retido = Imposto retido
carteira-imposto-padrao = Padrão: 17,5% em JCP (15% até 2025), 30% em dividendos dos EUA, 0 nos demais.
carteira-cambio-pagamento = { $base ->
        [BRL] Vazio: PTAX da data de pagamento.
       *[other] Vazio: câmbio da data de pagamento.
    }
carteira-lancar-credito = Lançar o crédito no caixa da corretora
carteira-imposto = Imposto
carteira-liquido = Líquido

## Registrar evento corporativo

carteira-ev-titulo = Registrar evento corporativo
carteira-ev-texto = Eventos de uma data valem antes das operações da mesma data (data “ex”).
carteira-ev-fator-split = Maior que 1: 2 = cada ação vira duas (1:2).
carteira-ev-fator-grupamento = Entre 0 e 1: 0,1 = dez ações viram uma (10:1).
carteira-ev-fator-bonificacao = Ações novas por ação: 0,1 = 10% de bonificação.
carteira-ev-fator-incorporacao = Opcional: ações do novo ativo por ação antiga (padrão 1).
carteira-ev-fator-ticker = Opcional: fator de troca (padrão 1).
carteira-ev-erro-fator = Informe o fator.
carteira-ev-erro-split = Desdobramento precisa de fator maior que 1.
carteira-ev-erro-grupamento = Grupamento precisa de fator entre 0 e 1.
carteira-ev-erro-bonificacao = Bonificação precisa de fator positivo.
carteira-ev-erro-data = Informe a data.
carteira-ev-erro-valor = Informe o valor total devolvido.
carteira-ev-erro-ativo-resultante = Escolha o ativo resultante.
carteira-ev-registrado = Evento corporativo registrado
carteira-ev-data = Data ex/efetiva
carteira-ev-fator = Fator
carteira-ev-custo-por-acao = Custo atribuído por ação ({ $moeda })
carteira-ev-custo-dica = Informado no aviso aos acionistas.
carteira-ev-valor-devolvido = Valor total devolvido
carteira-ev-valor-devolvido-dica = Reduz o custo da posição.
carteira-ev-ativo-resultante = Ativo resultante
carteira-ev-ativo-resultante-dica = A posição passa para ele na data.
carteira-ev-cisao-texto = Cisão e subscrição ficam registradas no histórico; registre a operação resultante (a compra do novo ativo ou das ações subscritas) à parte.
carteira-ev-registrar = Registrar evento

## Registrar proventos (grade e a confirmar)

carteira-grade-texto = Confirme os proventos anunciados que você recebeu ou digite (ou cole) data, ticker e valor. O Treasury acha o provento, a corretora, o tipo e o imposto.
carteira-grade-como = Como registrar
carteira-grade-a-confirmar = A confirmar ({ $n })
carteira-grade-digitar = Digitar
carteira-grade-ticker-linha = Ticker, linha { $n }
carteira-grade-ativos-linha = Ativos para a linha { $n }
carteira-grade-em-carteira = em carteira
carteira-proventos-registrados = { $n } provento(s) registrado(s)
carteira-grade-nada-novo = Nada novo para registrar
carteira-grade-tabela = Proventos a registrar
carteira-grade-ticker = Ticker
carteira-grade-valor-recebido = Valor recebido
carteira-grade-entendeu = O que o Treasury entendeu
carteira-grade-data-linha = Data do pagamento, linha { $n }
carteira-grade-valor-linha = Valor recebido, linha { $n }
carteira-grade-data-invalida = Data inválida.
carteira-grade-complete = Complete data, ticker e valor.
carteira-grade-remover-linha = Remover linha { $n }
carteira-grade-ajuda = Valor recebido é o creditado na corretora (líquido). Data em branco repete a de cima; Enter vai para a próxima linha. Pode colar várias linhas de uma planilha.
carteira-grade-prontas = { $n } pronta(s)
carteira-grade-duplicadas = { $n } já registrada(s)
carteira-grade-problema = { $n } com problema
carteira-grade-nenhuma-completa = Nenhuma linha completa ainda.
carteira-registrar-n-proventos =
    { $n ->
        [1] Registrar { $n } provento
       *[other] Registrar { $n } proventos
    }
carteira-registrar = Registrar
carteira-bruto-ir = bruto { $bruto } · IR { $ir }
carteira-anunciado-curto = anunciado
carteira-conta-provento-de = Conta do provento de { $ativo }

carteira-conf-confere = confere
carteira-conf-divergem = fontes divergem
carteira-conf-uma-fonte = uma fonte
carteira-conf-fonte = { $fonte }: { $valor } por cota · data-com { $data }
carteira-conf-fonte-pago = { $fonte }: { $valor } por cota · data-com { $data } · pago em { $pago }
carteira-conferencia = { $status }: { $fontes }
carteira-conferencia-usado = { $status }: { $fontes } · usado { $fonte }
carteira-agente-pedido = Confira estes proventos a confirmar, em que as fontes divergem ou só uma fonte tem o provento (use list_pending_income, check_dividends e get_fundamentus e, se puder, a página de RI da empresa ou do fundo):
carteira-agente-fonte = { $fonte } { $valor }/cota, data-com { $data }
carteira-agente-fonte-pago = { $fonte } { $valor }/cota, data-com { $data }, pago { $pago }
carteira-agente-item = - { $ativo } { $tipo } (data-com { $data }): { $fontes }
carteira-agente-fecho = Para cada um, diga se é um problema de fato e qual o valor por cota e as datas corretos. Quando houver correção, proponha o lançamento com record_income_event (eu aprovo antes de gravar).
carteira-pend-vazio = Nenhum provento anunciado a confirmar: tudo o que a agenda e a B3 trouxeram para os seus ativos já está na carteira. Use "Digitar" para lançar um que não apareceu.
carteira-pend-texto = Anunciados com data-com em que você tinha o ativo e ainda não lançados. Desmarque o que não recebeu e corrija o valor se o extrato da corretora mostrar outro.
carteira-pend-tabela = Proventos a confirmar
carteira-pend-marcar-todos = Marcar todos
carteira-pend-qtd-cota = Qtd. × por cota
carteira-pend-recebi = Recebi o provento de { $ativo } pago em { $data }
carteira-pend-data-de = Data de pagamento de { $ativo }
carteira-pend-sem-data = sem data na fonte: confira
carteira-pend-detalhe = { $tipo } · { $conta } · data-com { $data }
carteira-ir-valor = IR { $valor }
carteira-pend-valor-de = Valor recebido de { $ativo }
carteira-pend-duvidosos = { $n } provento(s) com fontes divergentes ou de uma fonte só.
carteira-pend-pedir-agente = Pedir ao agente para conferir
carteira-pend-marcados = { $n } de { $total } marcado(s)
carteira-confirmar-n-proventos =
    { $n ->
        [1] Confirmar { $n } provento
       *[other] Confirmar { $n } proventos
    }
carteira-confirmar = Confirmar

## Séries de índices

carteira-unidade-anualizada = % a.a. (anualizado, 252 d.u.)

## Alocação por setor

carteira-setor-titulo = Por setor
carteira-setor-sem = Sem setor
carteira-setor-ativos = { $n } ativo(s)
carteira-setor-vazio = Sem posições com valor de mercado.
carteira-setor-rodape = Setor do cadastro de cada ativo (o seu ou uma sugestão aprovada das fontes).
carteira-setor-completar = Completar o setor de { $n } ativo(s) em Saúde dos ativos

## Vendas (resultado nominal e real)

carteira-aba-vendas = Vendas
carteira-deflator-igpm-dica = Índice Geral de Preços do Mercado (FGV, via BCB/SGS 189)
carteira-vendas-titulo = Resultado das vendas
carteira-vendas-total-vendido = Total vendido
carteira-vendas-n = { $n } venda(s): { $ganhos } com ganho, { $perdas } com perda
carteira-vendas-nominal = Resultado de preço
carteira-vendas-nominal-dica = Venda menos o custo pelo preço médio, como no IR
carteira-vendas-real = Resultado real ({ $indice })
carteira-vendas-real-dica = { $ganhos } com ganho real, { $perdas } com perda real
carteira-vendas-faltam = { $n } venda(s) sem { $indice } no período ficaram de fora
carteira-vendas-efeito = Comido pelo { $indice }
carteira-vendas-efeito-dica = Quanto o custo corrigido pelo { $indice } supera o custo original
carteira-vendas-por-mes = Resultado por mês: nominal × real
carteira-vendas-tabela = Vendas
carteira-vendas-col-venda = Venda
carteira-vendas-col-custo = Custo
carteira-vendas-col-nominal = Nominal (preço)
carteira-vendas-col-real = Real ({ $indice })
carteira-vendas-col-indice = { $indice } no prazo
carteira-vendas-col-prazo = Prazo médio
carteira-vendas-dias = { $n } d
carteira-vendas-sem-indice = Sem { $indice } para todo o período desta venda
carteira-vendas-dica-custo = Custo corrigido pelo { $indice } de cada compra até a venda:
carteira-vendas-vazio-titulo = Nenhuma venda no período
carteira-vendas-vazio-texto = As vendas registradas em Operações aparecem aqui, com o ganho ou a perda em dinheiro, antes e depois da inflação.
carteira-vendas-metodo = Custo pelo preço médio (renda fixa: primeiro que entra, primeiro que sai). O resultado real traz o custo de cada compra para a data da venda pelo { $indice }; o prazo médio é ponderado pelo custo. Valores na moeda base, antes do IR.
carteira-vendas-dica-conta = { $custo } × (1 { $variacao }) = { $corrigido }
carteira-vendas-registrar = Registrar operação
carteira-vendas-menu-editar = Editar venda…
carteira-vendas-menu-compra = Registrar compra deste ativo…
carteira-vendas-menu-venda = Registrar outra venda deste ativo…
carteira-vendas-menu-evento = Registrar evento (desdobramento, bonificação…)…
carteira-vendas-menu-excluir = Excluir venda…
carteira-vendas-erro-abrir = Não deu para abrir a venda
carteira-vendas-excluir-nota = O preço médio e o resultado das vendas seguintes são recalculados. Dá para desfazer em seguida.
carteira-vendas-col-proventos = Proventos
carteira-vendas-col-total = Com proventos
carteira-vendas-total = Com proventos
carteira-vendas-total-dica = Preço mais { $proventos } de proventos recebidos pelas cotas vendidas; o IR é só sobre o preço
carteira-vendas-dica-total = Preço { $preco } + proventos { $proventos } recebidos pelas cotas vendidas entre a compra e a venda
carteira-vendas-dica-total-real = Real com proventos ({ $indice }): { $valor }
carteira-conta-operacao = Conta (corretora ou banco)
carteira-op-modo = Como informar
carteira-op-modo-total = Valor total
carteira-op-modo-unitario = Quantidade × preço
carteira-valor-aplicado = Valor aplicado
carteira-valor-bruto-resgate = Valor bruto do resgate
carteira-valor-bruto-resgate-dica = Antes do IR; o IR retido vai em Impostos retidos e o líquido aparece no resumo.
carteira-resgate-total = Resgatar a posição inteira
carteira-erro-valor-total = Informe o valor.
carteira-erro-sem-posicao = Não há posição nesta conta na data.
carteira-previdencia-saldo-titulo = { $n ->
        [1] Previdência sem saldo recente
       *[other] { $n } previdências sem saldo recente
    }
carteira-previdencia-saldo-de = último saldo em { $data }
carteira-previdencia-sem-saldo = sem saldo informado (vale o que foi aportado)
carteira-previdencia-saldo-texto = Planos sem cota pública valem o saldo que você informa. Abra o plano e use Informar saldo com o valor que a entidade mostra.
carteira-saldo-na-data = Saldo nesta conta na data: { $valor }
carteira-resgate-parcial = Resgate parcial: { $pct } do saldo.
carteira-aplicacao-de = Aplicação de { $valor } em { $ativo } em { $data }
carteira-resgate-de = Resgate de { $valor } de { $ativo } em { $data }

## Calcular renda fixa (comparar ofertas e vender antes do vencimento; só no Brasil)
carteira-rf-aba = Calcular renda fixa
carteira-rf-cmd = Calcular renda fixa
carteira-rf-cmd-busca = renda fixa,comparar ofertas,cdb,lci,lca,tesouro,poupança,equivalência,marcação a mercado,vender título
carteira-rf-comparar-titulo = Comparar ofertas
carteira-rf-comparar-texto = Digite as ofertas que quer ver lado a lado. Os resultados ficam na ordem em que você digitou.
carteira-rf-valor = Valor aplicado
carteira-rf-inicio = Data da aplicação
carteira-rf-inicio-dica = Vazio: hoje
carteira-rf-ofertas = Ofertas
carteira-rf-oferta-n = Oferta { $n }
carteira-rf-tipo = Tipo
carteira-rf-indice = Rendimento
carteira-rf-taxa = Taxa (%)
carteira-rf-meses = Prazo (meses)
carteira-rf-resgate = Ou resgate em
carteira-rf-nome = Nome (opcional)
carteira-rf-adicionar = Adicionar oferta
carteira-rf-remover = Remover oferta { $n }
carteira-rf-limite = Até { $max } ofertas.
carteira-rf-indice-cdi-pct = % do CDI
carteira-rf-indice-cdi-mais = CDI + %
carteira-rf-indice-selic-mais = Selic + %
carteira-rf-indice-pre = Prefixado
carteira-rf-indice-ipca-mais = IPCA + %
carteira-rf-indice-poupanca = Poupança
carteira-rf-premissas = Premissas
carteira-rf-premissas-texto = Valem para todas as ofertas. Digite outro valor para simular; apague para voltar ao do cofre.
carteira-rf-cdi = CDI (% a.a.)
carteira-rf-selic = Selic meta (% a.a.)
carteira-rf-ipca = IPCA esperado (% a.a.)
carteira-rf-tr = TR (% a.m.)
carteira-rf-fonte-user = digitada por você
carteira-rf-fonte-vault = do cofre
carteira-rf-fonte-vault-data = do cofre · { $data }
carteira-rf-fonte-projection = projeção (Focus)
carteira-rf-fonte-default = padrão (sem dado no cofre)
carteira-rf-incompleto = Complete as ofertas
carteira-rf-incompleto-texto = Informe o valor e, em cada oferta, a taxa e um prazo em meses ou a data do resgate.
carteira-rf-tabela = Ofertas de renda fixa
carteira-rf-col-oferta = Oferta
carteira-rf-col-prazo = Prazo
carteira-rf-dias = { $n ->
        [1] 1 dia corrido
       *[other] { $n } dias corridos
    }
carteira-rf-ate = até { $data }
carteira-rf-col-bruto = Bruto
carteira-rf-col-iof = IOF
carteira-rf-col-ir = IR
carteira-rf-isento = isento
carteira-rf-col-custodia = Custódia
carteira-rf-col-liquido = Líquido
carteira-rf-col-ano = Líquido ao ano
carteira-rf-col-real = Acima da inflação
carteira-rf-col-equivalencia = Equivalência
carteira-rf-equiv-cdb = equivale a { $pct }% do CDI num CDB
carteira-rf-equiv-lci = equivale a { $pct }% do CDI numa LCI/LCA
carteira-rf-col-fgc = FGC
carteira-rf-sim = Sim
carteira-rf-nao = Não
carteira-rf-col-avisos = Avisos
carteira-rf-venda-titulo = Vender antes do vencimento
carteira-rf-venda-texto = Quanto um prefixado ou IPCA+ vale na data da venda pela taxa de mercado, ao lado de levar até o vencimento.
carteira-rf-venda-carteira = Preencher com um título da carteira
carteira-rf-venda-carteira-escolha = Escolha um título…
carteira-rf-venda-indice = Indexador
carteira-rf-venda-pre = Prefixado
carteira-rf-venda-ipca = IPCA+
carteira-rf-venda-aplicacao = Data da aplicação
carteira-rf-venda-vencimento = Vencimento
carteira-rf-venda-data = Data da venda
carteira-rf-venda-contratada = Taxa contratada (% a.a.)
carteira-rf-venda-mercado = Taxa de mercado hoje (% a.a.)
carteira-rf-venda-taxa-real = Juro real, acima do IPCA
carteira-rf-venda-isento = Isento de IR
carteira-rf-venda-ipca-premissa = IPCA suposto (% a.a.)
carteira-rf-venda-incompleto = Complete o título
carteira-rf-venda-incompleto-texto = Informe o valor, as datas (a venda entre a aplicação e o vencimento) e as duas taxas.
carteira-rf-venda-curva = Valor na curva
carteira-rf-venda-preco = Preço de mercado
carteira-rf-venda-diferenca = Diferença (marcação a mercado)
carteira-rf-venda-diferenca-dica = Preço de mercado menos o valor na curva: positiva quando os juros caíram desde a aplicação.
carteira-rf-venda-tabela = Vender hoje ou levar até o vencimento
carteira-rf-venda-hoje = Vender hoje
carteira-rf-venda-levar = Levar até o vencimento
carteira-rf-venda-bruto = Bruto
carteira-rf-venda-ir = IR
carteira-rf-venda-iof = IOF
carteira-rf-venda-liquido = Líquido
carteira-rf-venda-empate = Taxa de empate
carteira-rf-venda-empate-pre = Para a troca empatar, um título novo até { $data } precisa pagar { $taxa }% a.a.
carteira-rf-venda-empate-ipca = Para a troca empatar, um título novo até { $data } precisa pagar IPCA + { $taxa }% a.a.
carteira-rf-venda-empate-nenhuma = Sem taxa de empate para estes números.

## Ponte do resultado (portfolio.bridge): do valor no início ao valor no fim, em dinheiro.
carteira-ponte-titulo = De onde veio o resultado
carteira-ponte-explica = Primeiro em dinheiro: quanto havia no início, o que os preços, os proventos e os custos mudaram, o que você aplicou e retirou, e quanto há no fim. Os percentuais vêm depois.
carteira-ponte-inicio = Valor no início
carteira-ponte-preco = Ganho de preço
carteira-ponte-cambio = dos quais câmbio: { $valor }
carteira-ponte-cambio-coluna = Câmbio
carteira-ponte-proventos = Proventos
carteira-ponte-taxas = Taxas
carteira-ponte-impostos = Impostos
carteira-ponte-aportes = Aportes
carteira-ponte-retiradas = Retiradas
carteira-ponte-fim = Valor no fim
carteira-ponte-resultado = Resultado no período
carteira-ponte-resultado-curto = Resultado
carteira-ponte-resultado-dica = Valor no fim menos o valor no início, menos os aportes e mais as retiradas: o que os investimentos renderam em dinheiro.
carteira-ponte-inflacao = Perda para a inflação ({ $indice })
carteira-ponte-real = Resultado real
carteira-ponte-linhas = Linhas da ponte do resultado
carteira-ponte-linha = Linha
carteira-ponte-valor = Valor
carteira-ponte-aria = Ponte do resultado de { $de } a { $ate }: do valor no início ao valor no fim, linha a linha.
carteira-ponte-periodo = De { $de } a { $ate }
carteira-ponte-fonte = Fonte: operações, cotações, câmbio e proventos registrados · desempenho passado, não é recomendação de investimento
carteira-ponte-estimado = estimado
carteira-ponte-estimado-dica = Algum valor veio de cotação defasada, câmbio estimado ou índice ainda não divulgado.
carteira-ponte-por-ativo = Por ativo ({ $n })
carteira-ponte-tabela-ativos = Ponte do resultado por ativo
carteira-ponte-calculando = Montando a ponte do resultado
carteira-ponte-vazio = Sem valores no período.

## Risco da série de retornos da rentabilidade (semidesvio e quedas).
carteira-risco-semidesvio = Semidesvio (o tamanho típico das quedas)
carteira-risco-queda-datas = { $pct } de { $pico } a { $fundo }
carteira-risco-voltou-em = { $queda }, voltou ao topo em { $data }
carteira-risco-sem-volta = { $queda }, ainda não voltou ao topo
carteira-risco-tempo-abaixo = Maior tempo abaixo do topo
carteira-risco-dias =
    { $n ->
        [one] { $n } dia
       *[other] { $n } dias
    }
carteira-risco-ainda-abaixo = ainda abaixo
carteira-risco-metodo-mensal = Rentabilidade mensal (Modified Dietz)
carteira-risco-metodo-diario = Rentabilidade diária
carteira-risco-metodo-diario-fonte = Método: TWR diário encadeado, neutraliza aportes e resgates
carteira-risco-como = Como calculamos: { $metodo }. Semidesvio, pior queda e tempo abaixo do topo usam a mesma série de retornos. Desempenho passado não garante resultado futuro.

## Período padronizado e comparação (docs/portfolio-compare.md)

carteira-periodo-de = Período: { $aba }
carteira-periodo-ponte = Ponte do resultado
carteira-comparar-com = Comparar com
carteira-comparar-tipo = Tipo de série para comparar
carteira-comparar-indice = Índice
carteira-comparar-ativo = Ativo
carteira-comparar-cdi-mais = { $indice } + taxa
carteira-comparar-escolha = Escolha a série
carteira-comparar-taxa = Taxa ao ano, em %
carteira-comparar-ao-ano = % ao ano
carteira-comparar-adicionar = Adicionar
carteira-comparar-escolhidas = Séries comparadas
carteira-comparar-remover = Remover { $serie } da comparação
carteira-comparar-limite = Até { $n } séries por vez.
carteira-comparar-aviso = Desempenho passado lado a lado, não recomendação nem promessa de retorno.
carteira-mapa-titulo = Retorno de cada mês, por ano
carteira-mapa-fonte = Retorno mensal pelo Modified Dietz; o total junta os meses do período no ano e a última coluna é o { $indice } nos mesmos meses.
carteira-mapa-ano = Ano
carteira-mapa-total = Total
carteira-mapa-aria = Mapa de calor dos retornos mensais
carteira-contrib-titulo = Maiores contribuições
carteira-contrib-calculando = Calculando as contribuições…
carteira-contrib-somaram = Mais somaram
carteira-contrib-tiraram = Mais tiraram
carteira-contrib-fonte = Resultado de cada ativo de { $de } a { $ate } (valorização, vendas e proventos), em dinheiro e em pontos do retorno da carteira. Desempenho passado.
carteira-contrib-vazio = Nenhum ativo teve resultado no período.
carteira-contrib-tabela = Contribuição de cada ativo
carteira-contrib-ativo = Ativo
carteira-contrib-resultado = Resultado
carteira-contrib-proventos = Proventos
carteira-contrib-pontos = Pontos
carteira-prov-mes-titulo = Proventos por mês, ano contra ano
carteira-prov-mes-calculando = Somando os proventos…
carteira-prov-mes-fonte = Proventos recebidos, líquidos, em { $moeda }, nos três últimos anos.
carteira-prov-mes-aria = Proventos de cada mês nos três últimos anos
carteira-prov-mes-tabela = Proventos por mês e ano

## Método de custo e retorno anualizado no livro de vendas

carteira-vendas-col-medio = Preço médio (IR)
carteira-vendas-col-irr = Retorno a.a.
carteira-vendas-irr-dica = Retorno anualizado da venda: a taxa ao ano que iguala o custo de cada compra que saiu, os proventos recebidos e o valor da venda. Desempenho passado, não promessa.
carteira-vendas-medio = Pelo preço médio (IR)
carteira-vendas-medio-dica = O mesmo resultado pelo preço médio, a base do IR no Brasil
carteira-vendas-nominal-dica-fifo = Venda menos o custo das compras mais antigas (primeiro que entra, primeiro que sai)
carteira-vendas-nominal-dica-pool = Venda menos o custo pela regra do Reino Unido (mesmo dia, 30 dias, pool)
carteira-vendas-metodo-fifo = Custo pelo primeiro que entra, primeiro que sai (renda fixa: os lotes); o resultado pelo preço médio fica na coluna ao lado. O resultado real traz o custo de cada compra para a data da venda pelo { $indice }; o prazo médio é ponderado pelo custo. Valores na moeda base, antes do imposto.
carteira-vendas-metodo-pool = Custo pela regra do Reino Unido: as compras do mesmo dia casam primeiro, depois as dos 30 dias seguintes, e o resto sai do pool pelo custo médio (renda fixa: os lotes); o resultado pelo preço médio fica na coluna ao lado. O resultado real traz o custo de cada compra para a data da venda pelo { $indice }. Valores na moeda base, antes do imposto.
carteira-vendas-metodo-pais = O método segue o país da família; dá para trocar em Ajustes › Família.
carteira-vendas-metodo-escolhido = Método escolhido em Ajustes › Família.

## Aportes programados e carteira por objetivo

carteira-objetivo-titulo = Por objetivo
carteira-objetivo-vazio = Nenhuma posição ligada a uma meta ainda. Ligue posições numa meta para ver a carteira por objetivo.
carteira-objetivo-ir-metas = Ir para Metas
carteira-objetivo-alvo = { $n } posição(ões) ligada(s) · { $pct } do alvo
carteira-objetivo-sem = Sem objetivo
carteira-objetivo-nota = Pelos pesos das posições ligadas em cada meta (frações e pote comum).
carteira-plano-titulo = Aportes programados
carteira-plano-novo = Novo aporte programado
carteira-plano-editar = Editar aporte programado
carteira-plano-dialogo-texto = Todo mês, no dia escolhido, o aporte aparece para você confirmar com o preço do dia. Nada é gravado sem a sua confirmação.
carteira-plano-nome = Nome
carteira-plano-ativo = Ativo
carteira-plano-conta = Conta da corretora
carteira-plano-valor = Valor de cada aporte
carteira-plano-dia = Dia do mês
carteira-plano-dia-dica = De 1 a 28.
carteira-plano-inicio = Início
carteira-plano-fim = Fim
carteira-plano-fim-dica = Vazio: sem data para acabar.
carteira-plano-meta = Meta
carteira-plano-meta-dica = Para qual meta o aporte vai (só informativo).
carteira-plano-sem-meta = Nenhuma
carteira-plano-ativo-ligado = Ativo (desmarque para pausar)
carteira-plano-salvo = Aporte programado salvo
carteira-plano-a-confirmar = { $n } aporte(s) programado(s) para confirmar. Ajuste a quantidade e o preço se precisar.
carteira-plano-a-confirmar-lista = Aportes a confirmar
carteira-plano-preco-de = preço de { $data }
carteira-plano-sem-preco = sem cotação: informe o preço
carteira-plano-quantidade-de = Quantidade de { $ativo } em { $data }
carteira-plano-preco-rotulo = Preço de { $ativo } em { $data }
carteira-plano-confirmar = Confirmar
carteira-plano-pular = Pular
carteira-plano-confirmado = Compra de { $ativo } registrada
carteira-plano-pulado = Aporte de { $ativo } pulado
carteira-plano-erro = Não deu para gravar o aporte
carteira-plano-vazio = Nenhum aporte programado. Crie um para não precisar lançar a mesma compra todo mês.
carteira-plano-lista = Aportes programados
carteira-plano-pausado = pausado
carteira-plano-resumo = { $ativo } todo dia { $dia } · { $conta }
carteira-plano-proximo = próximo em { $data }
carteira-plano-excluir-nome = Excluir { $nome }
carteira-plano-excluido = Aporte programado excluído
carteira-plano-aviso = { $n } aporte(s) programado(s) esperando a sua confirmação.
carteira-plano-aviso-ver = Ver aportes

## Posição recente (menos de 30 dias: só o retorno do período)
carteira-recente-pill = posição recente
carteira-recente-comprada =
    { $dias ->
        [0] Comprada hoje
        [1] Comprada há 1 dia
       *[other] Comprada há { $dias } dias
    }
carteira-recente-dica = { $comprada }. Retorno no período: { $retorno }. O anualizado aparece a partir de 30 dias.
carteira-recente-nota = { $comprada }. O anualizado aparece a partir de 30 dias.
carteira-recente-no-periodo = no período
carteira-recente-anualizado = { $retorno } anualizado sobre { $dias } dias
carteira-recente-anualizado-real = real { $retorno }
carteira-recente-titulo = Retorno no período

## Concentração (limites da família; leitura dos dados, sem recomendação)
carteira-conc-titulo = Concentração
carteira-conc-ajustar = Ajustar limites
carteira-conc-botao = Limites de concentração
carteira-conc-lista = Alertas de concentração
carteira-conc-tipo-ativo = Ativo
carteira-conc-tipo-grupo = Grupo
carteira-conc-tipo-classe = Classe
carteira-conc-limite = da carteira · limite { $limite }
carteira-conc-acima = { $valor } acima do limite
carteira-conc-sev-warning = Acima do limite
carteira-conc-sev-critical = Bem acima do limite
carteira-conc-rodape = Pesos pelo valor de hoje, comparados com os limites que você escolheu. É uma leitura dos seus dados, não uma recomendação de compra ou venda.
carteira-conc-pill-dica = Peso acima do limite de { $limite } para este ativo
carteira-conc-dialogo-titulo = Limites de concentração
carteira-conc-dialogo-texto = O Treasury avisa quando um ativo ou um grupo de ativos passa do limite. Sem ajuste, valem 20% por ativo e 30% por grupo; a classe usa o alvo de alocação com a tolerância. O Tesouro Direto e a renda fixa com FGC (até R$ 250 mil por ativo) ficam fora do limite padrão.
carteira-conc-padroes = Padrões
carteira-conc-padrao-ativo = Limite por ativo (%)
carteira-conc-padrao-grupo = Limite por grupo (%)
carteira-conc-por-ativo = Por ativo
carteira-conc-por-ativo-texto = Deixe o limite em branco para usar o padrão.
carteira-conc-col-limite = Limite
carteira-conc-limite-de = Limite de { $ativo } (%)
carteira-conc-isento = sem limite
carteira-conc-isento-dica = Tesouro Direto e renda fixa com FGC (até R$ 250 mil por ativo) ficam fora do limite padrão. Digite um limite para valer neste ativo.
carteira-conc-limite-invalido = Use limites maiores que 0% e de até 100%.
carteira-conc-salvar = Salvar limites
carteira-conc-salvos = Limites salvos
carteira-conc-sem-ativos = Nenhuma posição na carteira ainda.
carteira-rebal-conc-texto = Sem alvos por classe, a divisão abaixo segue os limites de concentração: cada ativo recebe uma parte do aporte proporcional ao espaço que ainda tem até o limite. Nada é vendido.
carteira-rebal-conc-tabela = Aporte pelos limites de concentração
carteira-rebal-conc-espaco = Espaço até o limite
carteira-rebal-conc-acima = acima do limite, sem aporte

## Etiquetas de ativos (grupos de concentração)
carteira-etiqueta-titulo = Etiquetas
carteira-etiqueta-texto = Etiquetas juntam ativos (por setor, país ou emissor, por exemplo). Marque uma como grupo de concentração para somar o peso dos ativos dela.
carteira-etiqueta-de = Etiquetas de { $ativo }
carteira-etiqueta-tirar = Tirar a etiqueta { $etiqueta } de { $ativo }
carteira-etiqueta-exemplo = setor, país…
carteira-etiqueta-grupo = Grupo de concentração
carteira-etiqueta-grupo-de = { $etiqueta } é grupo de concentração
carteira-etiqueta-limite-de = Limite do grupo { $etiqueta } (%)
carteira-etiqueta-sem = Nenhuma etiqueta ainda. Escreva uma na linha de um ativo.
carteira-etiqueta-sem-ativos = sem ativos na carteira
carteira-etiqueta-outros =
    { $n ->
        [1] mais 1 ativo fora da carteira
       *[other] mais { $n } ativos fora da carteira
    }
