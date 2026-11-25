# Score de saúde por ativo (núcleo: `score`): notas, faixas, explicações, rótulos das fontes de
# dados, notas de cobertura, motivos de dado ausente, avisos e erros, no idioma da instalação.
# pt-BR é o idioma de origem. Os rótulos e fórmulas da metodologia (methodology/**: classes,
# segmentos, perfis, dimensões e métricas) ficam como estão; os nomes das faixas vêm de
# score.ftl (`score-faixa-*`).

## Faixas, aviso e motivos de não haver score

nucleo-score-aviso = Score calculado com dados públicos e regras da literatura, ainda não validado com o histórico (metodologia em rascunho). Não é recomendação de investimento.
nucleo-score-faixa-robusto = Fundamentos sólidos nos critérios medidos
nucleo-score-faixa-saudavel = Ok; monitorar trimestralmente
nucleo-score-faixa-atencao = Deterioração ou fraqueza relevante; revisar a tese
nucleo-score-faixa-risco = Fundamentos comprometidos nos critérios medidos
nucleo-score-nao-implementado = O score de { $classe } ainda não foi implementado (primeira entrega: ações BR, bancos e FIIs).
nucleo-score-sem-score = Sem score: { $motivo }.
nucleo-score-sem-score-falta = Sem score: falta { $metricas }, essencial para este tipo de ativo.
nucleo-score-parcial = Score parcial: { $imputado }% do peso foi imputado (limite { $limite }%), por isso sem faixa definitiva.
nucleo-score-sem-metricas = Sem métricas calculáveis.
nucleo-score-dimensao-dados = Dados
nucleo-score-dado-demonstracoes = Demonstrações financeiras (CVM)
nucleo-score-sem-demonstracoes = sem demonstrações da CVM baixadas para este ativo (rode a coleta da análise de ativos)

## Motor: valores, pares, notas e explicações

nucleo-score-valor-dias = { $valor } dias
nucleo-score-valor-anos = { $valor } anos
nucleo-score-valor-bi = R$ { $valor } bi
nucleo-score-valor-mi = R$ { $valor } mi
nucleo-score-valor-mil = R$ { $valor } mil
nucleo-score-pares-do-setor = do setor { $setor }
nucleo-score-pares-do-segmento = do segmento { $segmento }
nucleo-score-pares-da-classe = da classe
nucleo-score-sem-dado = sem dado
nucleo-score-nota-defasados = Dados defasados: os mais recentes são de { $data } (mais de 2 trimestres).
nucleo-score-nota-sem-data = Sem data dos dados.
nucleo-score-nota-confianca = Confiança × { $fator }: { $motivo }.
nucleo-score-pior-caso = pior caso
nucleo-score-explica-pares = { $metrica } { $valor }: p{ $percentil } entre { $pares } pares { $nivel }
nucleo-score-explica-absoluta = { $metrica } { $valor }: nota { $nota } pela referência absoluta
nucleo-score-lista = { $itens } e { $ultimo }

## Ações: confiança, Magic Formula e notas

nucleo-score-confianca-fonte-secundaria = fonte secundária (Fundamentus) em { $metricas }
nucleo-score-confianca-liquidez-baixa = liquidez baixa
nucleo-score-confianca-divergencia = divergência acima de 25% com o Fundamentus em algum indicador
nucleo-score-magic-roc = Retorno sobre o capital (EBIT ÷ capital de giro + imobilizado)
nucleo-score-magic-nota = Diagnóstico: o ROC entra em Rentabilidade e o EY em Valuation
nucleo-score-nota-financeira = Instituição financeira: Altman, Piotroski, Beneish e EBITDA não se aplicam.

## Comandos

nucleo-score-erro-segmento-so-interface = o segmento do score só pela interface do app
nucleo-score-erro-avaliacao-so-interface = a avaliação do score só pela interface do app
nucleo-score-erro-eventos-so-interface = eventos do score só pela interface do app
nucleo-score-motivo-avaliacao = avaliação do score de saúde do ativo
nucleo-score-motivo-evento = evento qualitativo do score de saúde
nucleo-score-motivo-decisao-evento = decisão sobre evento do score de saúde
nucleo-score-motivo-apagar-evento = apagar evento do score de saúde

## Bancos (IF.data)

nucleo-score-banco-grupo-grandes = Bancos grandes (S1–S2)
nucleo-score-banco-grupo-medios = Bancos médios (S3)
nucleo-score-banco-grupo-pequenos = Bancos pequenos (S4)
nucleo-score-banco-trimestre = trimestre até { $data }
nucleo-score-banco-sem-dados = sem dados do Bacen IF.data para este banco
nucleo-score-banco-sem-basileia = sem Índice de Basileia no IF.data
nucleo-score-banco-sem-capital-principal = sem capital principal no IF.data
nucleo-score-banco-sem-alavancagem = sem razão de alavancagem no IF.data
nucleo-score-banco-sem-inadimplencia = sem inadimplência ou carteira no IF.data
nucleo-score-banco-sem-ativos-problematicos = sem ativos problemáticos no IF.data
nucleo-score-banco-sem-provisao = sem provisão ou inadimplência no IF.data
nucleo-score-banco-sem-despesa-provisao = sem despesa de provisão de 12 meses no IF.data
nucleo-score-banco-sem-carteira-anterior = sem carteira de 12 meses antes no IF.data
nucleo-score-banco-sem-despesas-receitas = sem despesas e receitas de 12 meses no IF.data
nucleo-score-banco-nota-relatorios = relatórios 2 e 4 do IF.data só são baixados para bancos S1–S3, listados e emissores da carteira
nucleo-score-sem-fonte = sem fonte
nucleo-score-banco-nota-prudenciais = Indicadores prudenciais do { $banco } (Bacen IF.data, { $periodo }); resultados de 12 meses somam os semestres ({ $convencao }).
nucleo-score-banco-nota-sem-correspondencia = Banco sem correspondência no Bacen IF.data (pelo CNPJ da instituição líder): rode a coleta da análise de ativos com o Banco Central ligado.
nucleo-score-banco-roe-abaixo = ROE abaixo do crescimento: P/VP justo ≤ 0
nucleo-score-banco-pvp-justo = P/VP justo { $pvp } com Ke { $ke }% e g { $g }%
nucleo-score-banco-sem-pvp-roe = sem P/VP ou ROE
nucleo-score-banco-flag-basileia = Basileia de { $valor }%

## Cripto

nucleo-score-cripto-nao-comparavel = Cripto não se compara diretamente com ações: o score mede a robustez dentro da classe de criptoativos.
nucleo-score-cripto-periodo-coleta = coleta de { $data }
nucleo-score-cripto-sem-coingecko = sem { $dado } da CoinGecko (rode a coleta da análise de ativos)
nucleo-score-cripto-dado-valor-mercado = valor de mercado
nucleo-score-cripto-dado-ranking = ranking
nucleo-score-cripto-dado-volume = volume de 24 horas
nucleo-score-cripto-dado-volume-valor = volume e valor de mercado
nucleo-score-cripto-dado-fdv = valor totalmente diluído
nucleo-score-cripto-dado-oferta = oferta em circulação
nucleo-score-cripto-dado-historico = histórico de 12 meses
nucleo-score-cripto-dado-usuarios = usuários que acompanham
nucleo-score-cripto-dado-commits = commits
nucleo-score-cripto-dado-estrelas = estrelas no GitHub
nucleo-score-cripto-dado-maxima = máxima histórica
nucleo-score-cripto-fonte-dolar = { $fonte }; dólar a R$ { $taxa } ({ $data })
nucleo-score-cripto-sem-ptax = sem a cotação do dólar (PTAX) para converter
nucleo-score-cripto-sem-valor-mercado = sem valor de mercado da CoinGecko nem da DefiLlama (rode a coleta da análise de ativos)
nucleo-score-cripto-sem-oferta-maxima = sem oferta máxima (emissão contínua): vale a inflação da oferta
nucleo-score-cripto-sem-tvl = não é uma rede com TVL na DefiLlama (ou falta a coleta)
nucleo-score-cripto-periodo-genese = gênese em { $data }
nucleo-score-cripto-sem-genese = sem data do bloco gênese na CoinGecko
nucleo-score-cripto-periodo-media-30d = média de 30 dias até { $data }
nucleo-score-cripto-nota-coin-metrics = Coin Metrics, dados abertos da API Community (CC BY-NC 4.0)
nucleo-score-cripto-sem-coin-metrics = a API aberta da Coin Metrics não cobre esta rede (ou falta a coleta)
nucleo-score-cripto-periodo-semana = semana até { $data }
nucleo-score-cripto-nota-nakamoto = pools de mineração com mais da metade dos blocos da semana
nucleo-score-cripto-sem-nakamoto = só para o bitcoin (pools de mineração do mempool.space); outras redes sem fonte gratuita
nucleo-score-cripto-fonte-ataques = DefiLlama (lista de ataques)
nucleo-score-cripto-nota-ataques = { $n ->
    [0] nenhum ataque à rede na lista da DefiLlama
    [1] 1 ataque à rede na lista da DefiLlama
   *[other] { $n } ataques à rede na lista da DefiLlama
}
nucleo-score-cripto-sem-ataques = sem a lista de ataques da DefiLlama (rode a coleta da análise de ativos)
nucleo-score-cripto-periodo-regra = regra da rede
nucleo-score-cripto-fonte-categorias = CoinGecko (categorias)
nucleo-score-cripto-nota-pow = prova de trabalho: sem alocações com desbloqueio; a emissão nova entra na inflação da oferta
nucleo-score-cripto-sem-desbloqueios = agendas de desbloqueio de tokens só em serviços pagos (Token Unlocks, Messari)
nucleo-score-cripto-fonte-github = CoinGecko (dados do GitHub)
nucleo-score-cripto-fonte-precos-dolar = preços diários em dólar (DefiLlama)
nucleo-score-cripto-fonte-cotacoes-app = cotações do app em { $moeda }
nucleo-score-periodo-de-a = { $de } a { $ate }
nucleo-score-cripto-poucas-cotacoes = menos de { $n } cotações nos últimos 12 meses (rode a coleta de cotações)
nucleo-score-nota-pico-fundo = pico em { $pico }, fundo em { $fundo }
nucleo-score-cripto-menos-12-meses = menos de 12 meses de cotações
nucleo-score-cripto-periodo-stablecoins = lista de stablecoins
nucleo-score-cripto-lastro-sem-nota = lastro "{ $lastro }" sem nota na metodologia
nucleo-score-cripto-fora-stablecoins = fora da lista de stablecoins da DefiLlama (ou falta a coleta)
nucleo-score-cripto-periodo-dias = { $de } a { $ate } ({ $dias } dias)
nucleo-score-cripto-nota-paridade = paridade com 1 { $moeda }
nucleo-score-cripto-sem-precos-paridade = sem preços diários na moeda da paridade ({ $moeda }) dos últimos 12 meses
nucleo-score-cripto-nota-cambio = Volatilidade e drawdown medidos em { $moeda }: incluem a variação do câmbio.
nucleo-score-cripto-lastro-fiat = moeda fiduciária e títulos
nucleo-score-cripto-lastro-cripto = outros criptoativos (sobrecolateralizado)
nucleo-score-cripto-lastro-algoritmico = algorítmico, sem reservas
nucleo-score-cripto-flag-desvio = desvio máximo de { $desvio }
nucleo-score-cripto-flag-algoritmico = lastro algorítmico (DefiLlama)
nucleo-score-cripto-flag-fdv = FDV { $vezes } o valor de mercado
nucleo-score-cripto-flag-valor-mercado = valor de mercado de { $valor }
nucleo-score-cripto-dado-criptoativo = Dados do criptoativo
nucleo-score-cripto-so-cadastrados = o score de cripto só vale para criptoativos cadastrados
nucleo-score-cripto-nota-id = Id na CoinGecko: { $id }.
nucleo-score-cripto-nota-coingecko-antiga = Dados da CoinGecko coletados em { $data }, há mais de 24 horas: rode a coleta da análise de ativos para renovar.
nucleo-score-cripto-nota-sem-id = Criptoativo sem id da CoinGecko: informe o ticker (símbolo) ou fixe o id com a fonte de cotação "coingecko" e o id no símbolo do provedor.
nucleo-score-cripto-nota-sem-pares = Criptoativos não têm universo de pares no app: as notas usam só as referências absolutas da classe.

## Cripto: erros das fontes

nucleo-score-erro-coingecko = resposta da CoinGecko inválida: { $erro }
nucleo-score-erro-coingecko-sem-moedas = resposta da CoinGecko sem a lista de moedas
nucleo-score-erro-coingecko-sem-id = resposta da CoinGecko sem id
nucleo-score-erro-defillama = resposta da DefiLlama inválida: { $erro }
nucleo-score-erro-defillama-sem-redes = resposta da DefiLlama sem a lista de redes
nucleo-score-erro-defillama-sem-stablecoins = resposta da DefiLlama sem a lista de stablecoins
nucleo-score-erro-defillama-sem-valores = resposta da DefiLlama sem valores de mercado
nucleo-score-erro-defillama-sem-precos = resposta da DefiLlama sem preços
nucleo-score-erro-coin-metrics-catalogo = catálogo da Coin Metrics inválido: { $erro }
nucleo-score-erro-coin-metrics-sem-ativos = catálogo da Coin Metrics sem a lista de ativos
nucleo-score-erro-coin-metrics = resposta da Coin Metrics inválida: { $erro }
nucleo-score-erro-coin-metrics-sem-serie = resposta da Coin Metrics sem a série
nucleo-score-erro-mempool = resposta do mempool.space inválida: { $erro }
nucleo-score-erro-mempool-sem-blocos = mempool.space sem blocos na semana
nucleo-score-erro-ataques = lista de ataques da DefiLlama inválida: { $erro }
nucleo-score-erro-ataques-sem-lista = lista de ataques da DefiLlama sem a lista

## Cripto: passos da coleta

nucleo-score-cripto-doc-coingecko = CoinGecko /coins/{ $id } em { $data }
nucleo-score-cripto-passo-tvl = DefiLlama: TVL das redes
nucleo-score-cripto-passo-stablecoins = DefiLlama: stablecoins
nucleo-score-cripto-passo-ids = CoinGecko: ids por símbolo
nucleo-score-cripto-passo-historico = CoinGecko: histórico de 12 meses de { $id }
nucleo-score-cripto-passo-precos = DefiLlama: preços diários de { $id }
nucleo-score-cripto-passo-valor-mercado = DefiLlama: valor de mercado
nucleo-score-cripto-passo-ataques = DefiLlama: ataques às redes
nucleo-score-cripto-passo-redes = Coin Metrics: redes da API aberta
nucleo-score-cripto-passo-enderecos = Coin Metrics: endereços ativos
nucleo-score-cripto-passo-pools = mempool.space: pools de mineração

## Ações: red flags, conferências e fontes secundárias

nucleo-score-flag-beneish = M = { $valor } (último exercício)
nucleo-score-flag-cobertura = cobertura de { $ultimo }x e { $anterior }x nos dois últimos trimestres (12 meses)
nucleo-score-flag-fcf-negativo = FCO − capex negativo nos 3 últimos exercícios
nucleo-score-flag-dividendos-acima = dividendos pagos acima do caixa livre nos 2 últimos exercícios
nucleo-score-flag-diluicao = ações em circulação +{ $pct }% em 12 meses
nucleo-score-flag-patrimonio-negativo = patrimônio líquido negativo no último balanço
nucleo-score-flag-entrega-atrasada = { $documento } de { $referencia } entregue em { $entrega }
nucleo-score-flag-auditor = { $antiga } substituída por { $nova } após { $anos } anos
nucleo-score-flag-volume-medio = volume médio de { $valor }
nucleo-score-recon-proventos-12m = Proventos por ação em 12 meses ({ $agenda } × B3)
nucleo-score-fonte-b3-oficial = B3 (oficial)
nucleo-score-recon-valor-diferente = valor diferente
nucleo-score-recon-so-b3 = só na B3
nucleo-score-recon-fora-b3 = fora da B3
nucleo-score-recon-provento = { $tipo ->
    [amortizacao] Amortização
   *[provento] Provento
} com data-com { $data } ({ $fonte }: { $situacao })
nucleo-score-recon-nota-divergente = Proventos de { $ticker } conferidos com a B3 (oficial) em 24 meses: { $iguais } iguais, { $diferentes } com valor diferente, { $so_b3 } só na B3, { $so_agenda } só na agenda ({ $agenda }). O DY usa a B3, fonte oficial.
nucleo-score-recon-nota-igual = Proventos de { $ticker } conferidos com a B3 (oficial) em 24 meses: { $iguais } iguais à agenda ({ $agenda }).
nucleo-score-fonte-brapi = brapi.dev, lido em { $data }
nucleo-score-fonte-fundamentus = Fundamentus ({ $indicador }), lido em { $data }
nucleo-score-conferencia-mesma-regra = { $metrica } (refeita com a fórmula do Fundamentus)
nucleo-score-secundaria-concordam = duas fontes secundárias concordam (média)
nucleo-score-secundaria-divergem = fontes secundárias divergem ({ $fonte1 }: { $valor1 }; { $fonte2 }: { $valor2 }); vale a primeira
nucleo-score-secundaria-uma = uma fonte secundária
nucleo-score-secundaria-marca = fonte secundária
nucleo-score-secundaria-nota-motivo = fonte oficial sem o dado ({ $motivo }); { $nota }
nucleo-score-secundaria-nota = fonte oficial sem o dado; { $nota }
nucleo-score-secundaria-periodo = informado pela fonte secundária

## Eventos qualitativos

nucleo-score-erro-evento-link = a evidência precisa ser um link http(s)
nucleo-score-erro-evento-tipo = tipo de evento desconhecido: { $tipo }
nucleo-score-erro-evento-titulo = descreva o evento (até 300 caracteres)
nucleo-score-erro-evento-ativo = informe o ativo (asset_id ou ticker)
nucleo-score-erro-evento-limites = teto e penalidade vão de 0 a 100
nucleo-score-ref-evento = evento
nucleo-score-flag-evento = Evento: { $tipo }

## Avisos para o desenvolvedor

nucleo-score-feedback-titulo = Score de ativos: dados críticos ausentes em { $ativo }
nucleo-score-feedback-resolvido-intrinseco = Resolvido sozinho em { $data }: não é falta da coleta ({ $detalhe }).
nucleo-score-feedback-resolvido-calculado = Resolvido sozinho em { $data }: os dados chegaram e o score foi calculado ({ $detalhe }).
nucleo-score-feedback-resolvido = Resolvido sozinho em { $data }: { $detalhe }
nucleo-score-feedback-resolvido-encerrado = Resolvido sozinho em { $data }: a posição foi encerrada, o ativo não está mais na carteira.
nucleo-score-feedback-corpo = O score de saúde de { $ativo } ({ $classe }, perfil { $perfil }) não foi calculado em { $data } porque faltam dados críticos:
    { $itens }

    Metodologia { $versao }. Ver o ativo em Ativos › { $ativo } › Saúde.
nucleo-score-feedback-lacuna-conhecida = Resolvido sozinho: lacuna conhecida da metodologia (sem fonte estruturada gratuita), documentada em docs/asset-score.md. O score renormaliza a dimensão.

## FIIs

nucleo-score-periodo-12-meses = 12 meses
nucleo-score-periodo-ultimos-pregoes = últimos pregões
nucleo-score-fii-periodo-trimestral = informe trimestral até { $data }
nucleo-score-fii-periodo-mensal = informe mensal de { $mes }
nucleo-score-fii-sem-trimestral = sem o informe trimestral da CVM
nucleo-score-fii-periodo-4-trimestres = 4 trimestres até { $data }
nucleo-score-fii-fonte-trimestrais = informes trimestrais CVM
nucleo-score-fii-sem-rendimentos-declarados = sem rendimentos declarados em 4 trimestres
nucleo-score-fii-poucos-trimestrais = menos de 4 informes trimestrais com resultado e rendimentos
nucleo-score-fii-sem-passivo = sem passivo ou patrimônio no informe mensal
nucleo-score-fii-sem-obrigacoes = sem obrigações por aquisição no informe mensal
nucleo-score-fii-periodo-pvp = cotação ÷ VP do último informe
nucleo-score-fii-sem-pvp = sem cotação ou valor patrimonial
nucleo-score-fii-sem-dy = sem rendimentos ou cotação
nucleo-score-fii-fonte-dy-cdi = DY do fundo; CDI (Banco Central, SGS 12)
nucleo-score-fii-sem-dy-fundo = sem dividend yield
nucleo-score-fii-sem-cdi = sem CDI dos últimos 12 meses
nucleo-score-fii-periodo-taxa = média de { $n } informes mensais, anualizada
nucleo-score-fii-fonte-taxa = CVM informe mensal (Percentual_Despesas_Taxa_Administracao)
nucleo-score-fii-poucas-taxas = menos de 3 informes mensais com a taxa de administração
nucleo-score-fii-ipca-descontado = IPCA descontado
nucleo-score-fii-sem-ipca = sem IPCA do período: variação nominal
nucleo-score-fii-fonte-vp-cota = CVM informe mensal (valor patrimonial por cota)
nucleo-score-fii-sem-vp-anterior = sem valor patrimonial por cota de 12 meses antes
nucleo-score-fii-sem-cotistas = sem número de cotistas
nucleo-score-fii-flag-vacancia = vacância financeira de { $pct }
nucleo-score-fii-flag-inquilino = maior inquilino com { $pct } da receita
nucleo-score-fii-dado-informes = Informes do FII (CVM)
nucleo-score-fii-sem-informes = sem informes da CVM baixados para este fundo (rode a coleta da análise de ativos)
nucleo-score-fii-nota-composicao = Composição do ativo no último informe mensal: CRI { $cri }, cotas de FII { $fii }, imóveis { $imoveis }.

## Renda fixa

nucleo-score-rf-flag-vencido = Venceu em { $data }
nucleo-score-rf-titulo-vencido = Título vencido
nucleo-score-rf-titulo-vencido-sentido = Título vencido ({ $data }): não há o que avaliar. Se ainda aparece com posição na Carteira, registre o resgate.
nucleo-score-rf-periodo-taxa = taxa de { $data }
nucleo-score-rf-nota-ipca = IPCA de 12 meses até { $mes }: { $pct }%
nucleo-score-rf-nota-selic = Selic meta { $selic }% + ágio { $agio }%; { $ipca }
nucleo-score-rf-periodo-hoje = hoje
nucleo-score-rf-periodo-meses = { $n } meses
nucleo-score-rf-fonte-bcb-selic-ipca = Banco Central (SGS 432 e 433)
nucleo-score-rf-nota-juro-real-selic = juro real da Selic de { $pct }% contra { $meses } fins de mês em { $anos } anos
nucleo-score-rf-familia-ipca = IPCA+ (Renda+ e Educa+ inclusos)
nucleo-score-rf-familia-prefixados = prefixados
nucleo-score-rf-nota-percentil-taxa = taxa de { $pct }% contra { $medianas } medianas diárias dos { $familia } em { $anos } anos (mediana { $mediana }%)
nucleo-score-rf-historico-curto = histórico de taxas dos { $familia } com menos de { $dias } dias gravados
nucleo-score-rf-nota-prefixado-real = prefixado { $pct }% descontado o { $ipca }
nucleo-score-rf-nota-cupom = cupom semestral de { $pct }% a.a.; vence em { $data }
nucleo-score-rf-nota-parcelas = { $n } parcelas mensais até { $data }
nucleo-score-rf-nota-sem-cupom = sem cupom; vence em { $data }
nucleo-score-rf-periodo-pregoes = { $n } pregões
nucleo-score-rf-nota-custodia-selic = { $pct }% a.a. sobre o que passa de { $isento } (posição de { $posicao })
nucleo-score-rf-nota-custodia = { $pct }% a.a. sobre o saldo
nucleo-score-rf-nota-pu = PU de compra { $compra } e de venda { $venda }
nucleo-score-rf-sem-termos-liquidez = sem os termos do título (liquidez e vencimento)
nucleo-score-rf-fonte-cadastro = cadastro do título
nucleo-score-rf-sem-vencimento = sem data de vencimento no cadastro do título
nucleo-score-rf-nota-pos-fixado = pós-fixado: a taxa acompanha o CDI/Selic, sensibilidade a juros quase nula
nucleo-score-rf-nota-aproximacao = aproximação: prazo até o vencimento (sem o cronograma de juros e amortização)
nucleo-score-rf-fonte-premissa = premissa da metodologia (ASSUMPTIONS.md › Renda fixa)
nucleo-score-rf-nota-tesouro-reais = Tesouro Nacional em reais
nucleo-score-rf-sem-taxas = sem taxas do Tesouro Transparente gravadas para este título (rode a coleta de cotações com o Tesouro Direto ligado)
nucleo-score-rf-historico-selic-curto = histórico da Selic meta e do IPCA curto demais (rode a coleta de índices)
nucleo-score-rf-sem-selic-ipca = sem Selic meta ou IPCA de 12 meses (rode a coleta de índices)
nucleo-score-rf-sem-ipca = sem IPCA de 12 meses (rode a coleta de índices)
nucleo-score-rf-nota-selic-duration = pós-fixado: o preço acompanha a Selic, sensibilidade a juros quase nula
nucleo-score-rf-vencimento-desconhecido = vencimento desconhecido: informe a data de vencimento ou o ano no código do título (ex.: Tesouro IPCA+ 2035)
nucleo-score-rf-fonte-pu = PU do Tesouro Transparente gravado para o ativo
nucleo-score-rf-poucos-precos = menos de 20 preços do título nos últimos 12 meses (rode a coleta de cotações)
nucleo-score-rf-fonte-regra-tesouro = regra do Tesouro Direto
nucleo-score-rf-nota-recompra = recompra diária pelo Tesouro
nucleo-score-rf-fonte-regra-b3 = regra da B3
nucleo-score-rf-fora-de-venda = título fora de venda no Tesouro Direto (sem PU de compra)
nucleo-score-rf-nota-tipo-desconhecido = Tipo do título do Tesouro não reconhecido no código nem no nome (ex.: Tesouro IPCA+ 2035).
nucleo-score-rf-nota-soberano = Risco de crédito soberano: nota fixa por premissa (Tesouro Nacional em reais; sem feed gratuito de rating).
nucleo-score-rf-dado-termos = Termos do título
nucleo-score-rf-sem-termos = o título não tem os termos cadastrados (tipo, indexador, taxa, vencimento)
nucleo-score-rf-sem-roe-ifdata = sem lucro e patrimônio de 12 meses no IF.data
nucleo-score-rf-nota-emissor-banco-cnpj = Emissor: { $emissor } (Bacen IF.data, pelo CNPJ do cadastro do título).
nucleo-score-rf-nota-emissor-banco-nome = Emissor: { $emissor } (Bacen IF.data), identificado pelo nome "{ $nome }"; confira e informe o CNPJ no cadastro para não depender do nome.
nucleo-score-rf-flag-inativa = { $emissor } não aparece como ativa no último cadastro do IF.data
nucleo-score-rf-fora-de-funcionamento = { $emissor } está fora de funcionamento no cadastro do Banco Central: os indicadores não são mais publicados
nucleo-score-rf-banco-nao-identificado = emissor "{ $nome }" não identificado no cadastro do Banco Central (IF.data): informe o CNPJ do banco emissor ao editar o título em Ativos
nucleo-score-rf-banco-nao-informado = emissor não informado: preencha o emissor e o CNPJ do banco no cadastro do título
nucleo-score-rf-fgc-tipo-outro = tipo do título "outro": não dá para saber se tem FGC (LF não tem; CDB, LC, LCI, LCA e poupança têm)
nucleo-score-rf-fonte-regra-fgc = regra do FGC
nucleo-score-rf-sem-fgc = título sem garantia do FGC
nucleo-score-rf-flag-fgc = { $excesso } acima do limite de { $limite } por CPF nesta instituição
nucleo-score-rf-titular-desconhecido = titular não informado
nucleo-score-rf-fgc-sem-titulares = contas sem titulares cadastrados contam como um único CPF (exposição da família inteira)
nucleo-score-rf-fgc-por-titular = exposição por titular das contas (conta conjunta dividida entre os titulares)
nucleo-score-rf-sem-posicao = sem posição: dentro do limite
nucleo-score-rf-nota-fgc =
    { $titulos ->
        [one] { $titulos } título garantido da família nesta instituição; maior exposição por CPF { $exposicao }; titulares: { $titulares }; { $atribuicao }
       *[other] { $titulos } títulos garantidos da família nesta instituição; maior exposição por CPF { $exposicao }; titulares: { $titulares }; { $atribuicao }
    }
nucleo-score-rf-fonte-fgc = posições da carteira e regra do FGC
nucleo-score-rf-fonte-regra-poupanca = regra da poupança
nucleo-score-rf-nota-liquidez-diaria = liquidez diária
nucleo-score-rf-nota-heranca-banco = Saúde do emissor herdada do banco (§7.2): indicadores prudenciais do Bacen IF.data contra os bancos do mesmo porte.
nucleo-score-rf-sem-cdi = sem CDI gravado (rode a coleta de índices)
nucleo-score-rf-fonte-bcb-poupanca = Banco Central (SGS 12, 432 e 226)
nucleo-score-rf-nota-poupanca = poupança rende { $rende }% a.a. (Selic meta { $selic }%, TR 12 meses { $tr }%); isenta, comparada com IR de { $ir }%
nucleo-score-rf-taxa-invalida = taxa inválida no cadastro do título
nucleo-score-rf-pct-do-indice = { $pct }% do { $indice }
nucleo-score-rf-nota-prefixado-curva = prefixado { $pct }% contra Tesouro Prefixado de prazo parecido ({ $curva }%)
nucleo-score-rf-fonte-bcb-cdi = Banco Central (SGS 12)
nucleo-score-rf-nota-prefixado-cdi = prefixado { $pct }% contra o CDI de hoje ({ $cdi }%; sem curva do Tesouro)
nucleo-score-rf-sem-curva-cdi = sem curva do Tesouro nem CDI (rode a coleta de cotações e de índices)
nucleo-score-rf-nota-ipca-curva = inflação + { $pct }% contra Tesouro IPCA+ de prazo parecido ({ $real }%) e prefixado ({ $pre }%)
nucleo-score-rf-fonte-bcb-cdi-ipca = Banco Central (SGS 12 e 433)
nucleo-score-rf-nota-ipca-cdi = inflação + { $pct }% com IPCA de 12 meses ({ $ipca }%) contra o CDI de hoje ({ $cdi }%; sem curva do Tesouro)
nucleo-score-rf-sem-curva-cdi-ipca = sem curva do Tesouro nem CDI e IPCA (rode a coleta de cotações e de índices)
nucleo-score-rf-nota-isento = ; isento, equivale a { $pct }% do CDI tributado (IR de { $ir }% para { $dias } dias)
nucleo-score-rf-via-cnpj = pelo CNPJ do cadastro do título
nucleo-score-rf-via-nome = pelo nome "{ $nome }"; confira e informe o CNPJ no cadastro
nucleo-score-rf-nota-emissor-cvm = Emissor: { $emissor } (CVM), { $como }.
nucleo-score-rf-periodo-dados-ate = dados até { $data }
nucleo-score-rf-fonte-score-acoes = score de ações do emissor
nucleo-score-rf-nota-score-emissor = { $emissor }: score { $score } ({ $faixa }), saúde { $saude }{ $parcial ->
    [sim] ; score parcial (muito peso imputado)
   *[nao] {""}
}
nucleo-score-rf-score-emissor-falhou = o score de ações de { $emissor } não saiu: { $faltam }
nucleo-score-rf-companhia-nao-identificada = emissor "{ $nome }" não identificado entre as companhias abertas da CVM: informe o CNPJ da companhia no cadastro do título (e rode a coleta da análise de ativos)
nucleo-score-rf-companhia-nao-informada = emissor não informado: preencha o emissor e o CNPJ da companhia no cadastro do título
nucleo-score-rf-nota-spread-ipca = inflação + { $pct }% contra Tesouro IPCA+ de prazo parecido ({ $curva }%)
nucleo-score-rf-sem-curva-ipca = sem curva do Tesouro IPCA+ (rode a coleta de cotações)
nucleo-score-rf-sem-curva-prefixado = sem curva do Tesouro Prefixado (rode a coleta de cotações)
nucleo-score-rf-nota-pct-cdi = { $pct }% do CDI com CDI de { $cdi }% a.a.
nucleo-score-rf-isento = isento de IR para pessoa física
nucleo-score-rf-tributado = tributado pela tabela regressiva
nucleo-score-rf-nota-credito-privado = Garantias, covenants, subordinação e rating não têm fonte estruturada gratuita: a dimensão Estrutura é imputada (§7.5). Spread calculado com a taxa contratada contra a curva do Tesouro; as taxas indicativas da ANBIMA não são dado aberto.

## Modelos clássicos

nucleo-score-piotroski-roa = ROA positivo
nucleo-score-piotroski-fco = Fluxo de caixa operacional positivo
nucleo-score-piotroski-roa-maior = ROA maior que no ano anterior
nucleo-score-piotroski-accrual = Caixa operacional ÷ ativo maior que o ROA (qualidade do accrual)
nucleo-score-piotroski-alavancagem = Alavancagem de longo prazo menor
nucleo-score-piotroski-liquidez = Liquidez corrente maior
nucleo-score-piotroski-sem-emissao = Não emitiu ações
nucleo-score-piotroski-margem = Margem bruta maior
nucleo-score-piotroski-giro = Giro do ativo maior
nucleo-score-leitura-forte = forte
nucleo-score-leitura-fraco = fraco
nucleo-score-leitura-intermediario = intermediário
nucleo-score-piotroski-nota = { $n } de 9 critérios calculáveis com os dados disponíveis
nucleo-score-leitura-zona-segura = zona segura
nucleo-score-leitura-zona-cinza = zona cinza
nucleo-score-leitura-zona-estresse = zona de estresse
nucleo-score-altman-x1 = X1 capital de giro ÷ ativo
nucleo-score-altman-x2 = X2 lucros retidos ÷ ativo
nucleo-score-altman-x3 = X3 EBIT ÷ ativo
nucleo-score-altman-x4 = X4 PL contábil ÷ passivo
nucleo-score-altman-referencia = Altman (1968); Altman, Hartzell e Peck (1995)
nucleo-score-beneish-dsri = DSRI recebíveis/receita
nucleo-score-beneish-gmi = GMI margem bruta
nucleo-score-beneish-aqi = AQI qualidade do ativo
nucleo-score-beneish-sgi = SGI crescimento da receita
nucleo-score-beneish-depi = DEPI taxa de depreciação
nucleo-score-beneish-sgai = SGAI despesas de vendas e administrativas
nucleo-score-beneish-tata = TATA accruals totais
nucleo-score-beneish-lvgi = LVGI alavancagem
nucleo-score-leitura-manipulacao = provável manipulação (revisar)
nucleo-score-leitura-sem-manipulacao = sem sinal de manipulação
nucleo-score-beneish-nota = sem dado para { $variaveis } (entraram como neutros)

## Avaliação e segmento

nucleo-score-erro-veredito = o veredito é "discordo" ou "concordo"
nucleo-score-erro-motivo = explique o motivo (até 2.000 caracteres)
nucleo-score-erro-ticker = informe o ticker
nucleo-score-erro-segmento = segmento "{ $segmento }" não existe para esta classe

## Fundos

nucleo-score-fundo-doc-informe = informe diário da CVM até { $data }
nucleo-score-fundo-sem-volume = sem volume negociado (rode a coleta de cotações)
nucleo-score-fundo-fonte-convertido = { $fonte }; convertido de { $moeda } pela taxa de { $data }
nucleo-score-fundo-sem-cambio = sem câmbio { $moeda }/BRL para converter o volume
nucleo-score-fundo-fonte-cotas = cotas do informe diário da CVM
nucleo-score-fundo-poucas-cotas = menos de 12 meses de cotas para { $para }
nucleo-score-fundo-para-indice = o índice
nucleo-score-fundo-para-benchmark = o benchmark
nucleo-score-fundo-para-drawdown = o drawdown
nucleo-score-fundo-para-recuperacao = a recuperação
nucleo-score-fundo-periodo-anos = { $de } a { $ate } ({ $anos } anos)
nucleo-score-fundo-sem-cdi = sem o CDI do período (rode a coleta de índices)
nucleo-score-fundo-nota-sharpe = { $valor } (retorno { $retorno } a.a. × CDI { $cdi } a.a.; volatilidade { $volatilidade })
nucleo-score-fundo-volatilidade-zero = volatilidade zero
nucleo-score-fundo-sortino-teto = nenhum dia abaixo do CDI: Sortino no teto
nucleo-score-fundo-nota-benchmark = benchmark: { $benchmark }
nucleo-score-fundo-benchmark-cdi = benchmark é o CDI: o Sharpe já mede o retorno sobre ele
nucleo-score-fundo-nota-information-ratio = { $valor } (tracking error { $te })
nucleo-score-fundo-tracking-zero = tracking error zero
nucleo-score-fundo-sem-serie = sem a série do { $benchmark } no período
nucleo-score-fundo-nota-janelas = { $acima } de { $janelas } janelas acima do { $benchmark }
nucleo-score-fundo-poucas-janelas = menos de { $minimo } janelas de 12 meses com cotas e { $benchmark }
nucleo-score-fundo-sem-queda = sem queda na janela
nucleo-score-fundo-recuperada = , recuperada em { $data }
nucleo-score-fundo-nao-recuperada = , ainda não recuperada
nucleo-score-fundo-periodo-cotas = { $n } cotas até { $data }
nucleo-score-fundo-poucas-cotas-12m = menos de { $n } cotas nos últimos 12 meses
nucleo-score-fundo-periodo-extrato = extrato de { $data }
nucleo-score-fundo-fonte-extrato = CVM: extrato de informações dos fundos
nucleo-score-fundo-sem-extrato = fundo sem extrato de informações na CVM
nucleo-score-fundo-nota-performance = sobre o que exceder { $benchmark }
nucleo-score-fundo-fonte-patrimonio = CVM: informe diário (VL_PATRIM_LIQ)
nucleo-score-fundo-periodo-registro = registro de { $data }
nucleo-score-fundo-fonte-registro = CVM: registro de fundos e classes
nucleo-score-fundo-sem-patrimonio = sem patrimônio no informe diário nem no registro da CVM
nucleo-score-fundo-fonte-cotistas = CVM: informe diário (NR_COTST)
nucleo-score-fundo-sem-cotistas = sem número de cotistas no informe diário
nucleo-score-fundo-periodo-captacao = { $meses } meses, { $de } a { $ate } ({ $documento })
nucleo-score-fundo-fonte-captacao = CVM: informe diário (CAPTC_DIA − RESG_DIA)
nucleo-score-fundo-nota-captacao = captação líquida de { $valor }
nucleo-score-fundo-pouca-captacao = menos de 11 meses de captação no informe diário
nucleo-score-fundo-periodo-constituido = constituído em { $data }
nucleo-score-fundo-nota-tempo-gestao = proxy do tempo de gestão
nucleo-score-fundo-sem-constituicao = sem data de constituição no registro da CVM
nucleo-score-fundo-retorno-meses = { $meses } meses { $pct } a.a.{ $benchmark }
nucleo-score-fundo-nota-retorno = Retorno da cota: { $retornos }.
nucleo-score-fundo-flag-patrimonio = patrimônio de { $valor }
nucleo-score-fundo-flag-resgates = captação líquida de { $pct } do patrimônio em 12 meses
nucleo-score-fundo-flag-queda = queda máxima de { $pct }
nucleo-score-fundo-cotacao-manual = cotação manual: registre as cotas no ativo ou escolha a fonte CVM
nucleo-score-fundo-sem-cnpj-provavel = fundo sem CNPJ: pelo nome, o provável é { $cnpj } ({ $nome }, no registro da CVM); confira e informe o CNPJ no ativo para buscar as cotas
nucleo-score-fundo-sem-cnpj = fundo sem CNPJ: informe o CNPJ no ativo para buscar as cotas na CVM
nucleo-score-fundo-cnpj-plano-provavel = o CNPJ { $cnpj } não está no registro de fundos da CVM (em previdência, costuma ser o do plano); pelo nome, o fundo provável é { $provavel } ({ $nome }); confira e troque o CNPJ no ativo para buscar as cotas
nucleo-score-fundo-fonte-nao-cvm = a fonte de cotação do ativo não é a CVM: escolha CVM (ou deixe automática) para buscar as cotas pelo CNPJ
nucleo-score-fundo-cnpj-fora-do-registro = o CNPJ { $cnpj } não apareceu no registro de fundos da CVM baixado em { $data }{ $tipo ->
    [previdencia] : em previdência, informe o CNPJ do fundo (FIE) que o plano usa, não o do plano na Susep
   *[fundo] : confira o CNPJ no ativo
}
nucleo-score-fundo-sem-cotas = sem cotas gravadas (rode a coleta de cotações)
nucleo-score-fundo-dado-cotas = Cotas do fundo
nucleo-score-fundo-so-cadastrados = o score de fundos só vale para fundos cadastrados (com CNPJ)
nucleo-score-fundo-dado-cotas-cvm = Cotas do fundo (CVM)
nucleo-score-fundo-cadastro-classe = classe CVM { $classe }
nucleo-score-fundo-cadastro-indicador = indicador declarado: { $indicador }
nucleo-score-fundo-cadastro-gestor = gestor { $gestor }
nucleo-score-fundo-nota-cadastro = Cadastro na CVM: { $cadastro }.
nucleo-score-fundo-nota-exclusivo = Fundo exclusivo: poucos cotistas são esperados.
nucleo-score-fundo-nota-sem-cadastro = Fundo sem cadastro da CVM baixado: segmento padrão e sem taxas (rode a coleta da análise de ativos).
nucleo-score-fundo-nota-benchmark-pares = Benchmark: { $benchmark }. Fundos não têm universo de pares no app: as notas usam só as referências absolutas da classe.

## ETFs

nucleo-score-etf-domicilio-br = ETF domiciliado no Brasil
nucleo-score-etf-domicilio-us = ETF domiciliado nos EUA: 30% retidos sobre dividendos e risco de imposto sobre herança americano
nucleo-score-etf-domicilio-ucits = ETF listado na Europa (em geral UCITS irlandês): 15% retidos sobre dividendos americanos
nucleo-score-etf-domicilio-outro = ETF listado fora do Brasil, dos EUA e da Europa
nucleo-score-etf-dado-cotacoes = Cotações do ETF
nucleo-score-etf-so-cadastrados = o score de ETFs só vale para ETFs cadastrados
nucleo-score-etf-fonte-cotacoes = cotações ({ $fonte })
nucleo-score-etf-sem-indice = sem série do índice de referência no app
nucleo-score-etf-indice-convertido = { $indice } em { $moeda } pelo câmbio do dia
nucleo-score-etf-sem-serie = sem a série { $indice } (ou o câmbio) no período (rode a coleta de índices)
nucleo-score-etf-sem-dados = sem dados para comparar
nucleo-score-etf-fonte-indice = { $fonte }; índice { $indice }
nucleo-score-etf-nota-tracking = ETF { $etf } × índice { $indice }
nucleo-score-etf-periodo-semanal = { $periodo }, retornos semanais
nucleo-score-etf-nota-cambio = índice convertido pelo câmbio do dia: inclui o descasamento de horário entre o câmbio e os fechamentos
nucleo-score-etf-poucas-semanas = poucas semanas para o tracking error
nucleo-score-etf-sem-cnpj = ETF sem CNPJ achado no registro da CVM (informe o CNPJ no ativo e rode a coleta da análise de ativos)
nucleo-score-etf-sem-patrimonio = patrimônio de ETFs no exterior sem fonte estruturada gratuita
nucleo-score-etf-periodo-classe-bolsa = classe e bolsa do ativo
nucleo-score-etf-fonte-tabela = tabela da metodologia
nucleo-score-etf-domicilio-sem-nota = domicílio sem nota na metodologia
nucleo-score-etf-sem-constituicao = sem data de constituição (só ETFs da B3 com CNPJ no registro da CVM)
nucleo-score-etf-periodo-carteira = carteira teórica { $indice } de { $data }
nucleo-score-etf-fonte-carteira = B3: carteira teórica do índice
nucleo-score-etf-fonte-componentes = score de saúde de cada componente
nucleo-score-etf-nota-componentes = { $com_score } de { $total } componentes com score ({ $peso } do peso)
nucleo-score-etf-pouca-cobertura = só { $peso } do peso da carteira com score (mínimo 50%; rode a coleta da análise de ativos)
nucleo-score-etf-sem-carteira = sem a carteira teórica do { $indice } (rode a coleta da análise de ativos)
nucleo-score-etf-sem-composicao = composição sem fonte estruturada gratuita (só os índices da B3)
nucleo-score-etf-periodo-fechamentos = { $n } fechamentos até { $data }
nucleo-score-etf-poucos-fechamentos = menos de { $n } fechamentos nos últimos 12 meses (rode a coleta de cotações)
nucleo-score-etf-nota-indice = Índice de referência: { $indice }.
nucleo-score-etf-nota-cnpj = CNPJ na CVM: { $cnpj }.
nucleo-score-etf-nota-sem-composicao = Sem a composição do ETF: a saúde dos componentes e a concentração dão lugar ao risco (volatilidade e drawdown).
nucleo-score-etf-nota-sem-pares = ETFs não têm universo de pares no app: as notas usam só as referências absolutas.

## Fundos e ETFs: coleta

nucleo-score-erro-b3-carteira-invalida = resposta da B3 inválida: { $erro }
nucleo-score-erro-b3-sem-carteira = resposta da B3 sem a carteira
nucleo-score-erro-carteira-vazia = carteira teórica vazia
nucleo-score-erro-registro-sem-classes = registro da CVM sem registro_classe.csv
nucleo-score-passo-yahoo-etf = Yahoo: nome do ETF { $ticker }
nucleo-score-passo-carteira = B3: carteira teórica do { $indice }

## Exterior e BDRs

nucleo-score-intl-sec-desligada = { $falta }: a fonte SEC EDGAR está desligada em Ajustes › Fontes de dados
nucleo-score-intl-sec-email = { $falta }: a SEC só atende quem informa um e-mail de contato; informe o seu na aba Saúde do ativo (ou em Ajustes › Fontes de dados › Ajustes da análise) e a coleta da SEC roda na hora
nucleo-score-intl-sec-coleta = { $falta } (a coleta diária da análise de ativos ainda não trouxe; o resultado fica em Ativos › Dados da análise)
nucleo-score-intl-dado-demonstracoes = Demonstrações financeiras
nucleo-score-intl-sem-fonte = sem fonte gratuita de demonstrações para bolsas fora do Brasil e dos EUA (só cotação)
nucleo-score-intl-sem-sec = sem demonstrações da SEC baixadas para este ativo
nucleo-score-intl-dado-sec = Demonstrações financeiras (SEC EDGAR)
nucleo-score-intl-dado-historico = Histórico de balanços
nucleo-score-intl-listada-ha-pouco = empresa listada há pouco: ainda sem balanço anual (10-K) na SEC; o score sai depois do primeiro exercício completo
nucleo-score-intl-nota-sec = Demonstrações da SEC (US GAAP) em dólar; crescimento real descontado pelo CPI dos EUA. Sem dimensão de governança (sem fonte estruturada gratuita).
nucleo-score-intl-nota-financeira = Banco ou seguradora americana: sem indicadores prudenciais (Basileia, inadimplência); score pelo ROE, valuation e estabilidade.
nucleo-score-intl-dado-subjacente = Ativo subjacente do BDR
nucleo-score-intl-bdr-sem-empresa = não foi possível identificar a empresa do BDR
nucleo-score-intl-bdr-sem-sec = sem demonstrações da SEC para { $ticker }, subjacente provável do BDR
nucleo-score-intl-dado-subjacente-sec = Demonstrações do subjacente (SEC EDGAR)
nucleo-score-intl-nota-bdr = BDR: herda o score de { $empresa } ({ $ticker }), o subjacente.

## Métricas das demonstrações

nucleo-score-metrica-fonte-dfp = CVM DFP
nucleo-score-metrica-fonte-fca = CVM FCA (valores mobiliários)
nucleo-score-metrica-nota-ebitda-medio = EBITDA médio dos últimos exercícios (normalização cíclica)
nucleo-score-metrica-ebitda-medio-negativo = EBITDA médio ≤ 0
nucleo-score-metrica-capital-negativo = capital investido (dívida líquida + PL) ≤ 0
nucleo-score-metrica-divisao-zero = divisão por zero
nucleo-score-metrica-nota-divida-ebitda = dívida atual ÷ EBITDA médio
nucleo-score-metrica-sem-altman = faltam capital de giro, lucros retidos, EBIT ou passivo
nucleo-score-metrica-sem-ciclo = faltam recebíveis, fornecedores, receita ou CPV
nucleo-score-metrica-lucro-anterior-negativo = lucro do exercício anterior ≤ 0
nucleo-score-metrica-lucro-negativo-razao = lucro líquido ≤ 0 (a razão não se aplica)
nucleo-score-metrica-lpa-final-negativo = lucro por ação final ≤ 0
nucleo-score-metrica-lpa-inicial-negativo = lucro por ação inicial ≤ 0
nucleo-score-metrica-lucro-negativo-sem-proventos = lucro ≤ 0 e sem proventos
nucleo-score-metrica-poucos-fechamentos = menos de 120 fechamentos no último ano
nucleo-score-metrica-poucos-lucros-2 = menos de 2 exercícios com lucro
nucleo-score-metrica-poucos-ebitda = menos de 3 exercícios com EBITDA
nucleo-score-metrica-poucos-roic = menos de 3 exercícios com ROIC
nucleo-score-metrica-poucos-lucros-3 = menos de 3 exercícios com lucro líquido
nucleo-score-metrica-poucos-roe = menos de 4 exercícios com ROE
nucleo-score-metrica-poucos-lpa = menos de 6 exercícios com lucro e ações
nucleo-score-metrica-periodo-media-exercicios = média dos últimos exercícios
nucleo-score-metrica-sem-dividendos-12m = não pagou dividendos nos últimos 12 meses
nucleo-score-metrica-proventos-com-prejuizo = pagou proventos com lucro ≤ 0
nucleo-score-metrica-patrimonio-negativo = patrimônio líquido ≤ 0
nucleo-score-metrica-sem-piotroski = precisa de 2 exercícios com os itens do Piotroski
nucleo-score-metrica-receita-negativa = receita inicial ou final ≤ 0
nucleo-score-metrica-nota-receitas-financeiras = receitas financeiras superam as despesas financeiras
nucleo-score-metrica-sem-segmento-listagem = segmento de listagem não informado no FCA
nucleo-score-metrica-sem-ebit = sem EBIT
nucleo-score-metrica-sem-ebit-12m = sem EBIT de 12 meses
nucleo-score-metrica-sem-ebit-despesas = sem EBIT ou despesas financeiras
nucleo-score-metrica-sem-ebitda = sem EBITDA
nucleo-score-metrica-sem-ipca = sem IPCA do período: crescimento nominal
nucleo-score-metrica-sem-circulante = sem ativo ou passivo circulante
nucleo-score-metrica-sem-ativo-anterior = sem ativo total de 12 meses antes
nucleo-score-metrica-sem-caixa-divida-cp = sem caixa ou dívida de curto prazo
nucleo-score-metrica-sem-cotacao-acoes = sem cotação ou número de ações
nucleo-score-metrica-sem-cotacoes-12m = sem cotações de 12 meses
nucleo-score-metrica-sem-regra-40 = sem crescimento da receita ou margem de caixa livre
nucleo-score-metrica-sem-entregas = sem datas de entrega das demonstrações
nucleo-score-metrica-sem-dividendos = sem dividendos declarados ou pagos
nucleo-score-metrica-sem-dividendos-pagos = sem dividendos pagos
nucleo-score-metrica-sem-divida-cp = sem dívida de curto prazo
nucleo-score-metrica-sem-divida-caixa = sem dívida e caixa no balanço
nucleo-score-metrica-sem-divida-liquida = sem dívida líquida
nucleo-score-metrica-sem-fcl = sem fluxo de caixa livre
nucleo-score-metrica-sem-fco-lucro = sem fluxo de caixa operacional ou lucro
nucleo-score-metrica-sem-fco-receita = sem fluxo de caixa operacional ou receita
nucleo-score-metrica-sem-lucro = sem lucro
nucleo-score-metrica-sem-lucro-bruto-ativo = sem lucro bruto (ou receita e CPV) ou ativo total
nucleo-score-metrica-sem-lucro-12m = sem lucro líquido de 12 meses
nucleo-score-metrica-sem-lucro-ativo = sem lucro ou ativo total
nucleo-score-metrica-sem-lucro-fco-ativo = sem lucro, fluxo de caixa operacional ou ativo
nucleo-score-metrica-sem-acoes-anterior = sem número de ações de 12 meses antes
nucleo-score-metrica-sem-patrimonio = sem patrimônio líquido
nucleo-score-metrica-sem-patrimonio-ativo = sem patrimônio ou ativo
nucleo-score-metrica-sem-proventos-cotacao = sem proventos ou cotação
nucleo-score-metrica-sem-receita-ativo = sem receita ou ativo
nucleo-score-metrica-sem-receita-positiva = sem receita positiva
nucleo-score-metrica-sem-retornos = sem retornos
nucleo-score-metrica-ev-negativo = valor da firma ≤ 0 (caixa acima do valor de mercado)
nucleo-score-metrica-periodo-ultimo-balanco = último balanço
nucleo-score-metrica-sem-lucro-liquido = sem lucro líquido
nucleo-score-metrica-media-exercicios = média de { $n } exercícios
nucleo-score-metrica-periodo-exercicio = exercício até { $data }
nucleo-score-metrica-poucos-exercicios-receita = menos de { $n } exercícios com receita
nucleo-score-metrica-nota-nominal = nominal { $pct }% a.a., IPCA descontado
nucleo-score-metrica-periodo-n-exercicios = { $n } exercícios
nucleo-score-metrica-periodo-entregas = últimas { $n } entregas
nucleo-score-metrica-fonte-recebimento = CVM (data de recebimento)
nucleo-score-metrica-periodo-pregoes = { $n } pregões até { $data }
nucleo-score-metrica-fonte-fechamentos = fechamentos diários

## Cripto: documentos

nucleo-score-cripto-doc-market-chart = CoinGecko market_chart (365 dias)
nucleo-score-fundo-dado-cotas-extrato = Cotas do plano (extrato)
nucleo-score-fundo-efpc-sem-cotas = fundo de pensão (EFPC, fiscalizado pela Previc) não é fundo da CVM: a cota do plano só vem do extrato do participante; importe o extrato em PDF no ativo para registrar a cota mês a mês
nucleo-score-fii-trimestral-sem-item = o informe trimestral até { $data } não traz o dado (campo em branco ou zerado)
nucleo-score-fii-nota-por-area = ponderada pela área: o informe trimestral não traz a participação de cada imóvel na receita
nucleo-score-fii-nota-walt-acima-36 = { $pct } da receita vence em mais de 36 meses, a última faixa do informe; conta como 5 anos (premissa). Com 3 anos nessa faixa, o mínimo possível, o prazo médio seria { $minimo } anos.
nucleo-score-fii-nota-cobertura-vendas = { $pct } do resultado desses trimestres veio de venda de ativos; sem ela, a cobertura seria { $sem }.
nucleo-score-confianca-dy-estimado = DY estimado pelos informes da CVM (sem os proventos pagos na B3)
