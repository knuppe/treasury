# Ativos, mercado e índices (núcleo: `assets`, `market`, `indices`): erros de validação, motivos
# do histórico, avisos e erros da coleta de cotações e índices, nomes e termos dos provedores e
# nomes e notas dos países do catálogo, no idioma da instalação. pt-BR é o idioma de origem. Os
# nomes das classes vêm de carteira.ftl (`carteira-classe-*`) e os dos países de
# nucleo-pais.ftl; os nomes das séries de índices (gravados no cofre) ficam como estão.

## Ativos

nucleo-ativos-ref-ativo = ativo { $id }
nucleo-ativos-motivo-cadastrado = Ativo cadastrado: { $ativo }
nucleo-ativos-motivo-atualizado = Ativo atualizado
nucleo-ativos-motivo-removido = Ativo removido
nucleo-ativos-erro-classe-invalida = classe de ativo inválida: { $classe }
nucleo-ativos-erro-taxa-invalida = taxa inválida: { $taxa }
nucleo-ativos-erro-nome = informe o nome do ativo
nucleo-ativos-erro-renda-fixa-termos = renda fixa precisa dos termos (indexador, taxa, vencimento)
nucleo-ativos-erro-previdencia-classe = o plano de previdência é para ativos da classe Previdência
nucleo-ativos-erro-bolsa-fora = bolsa { $bolsa } não está no catálogo: cadastre a bolsa (código, país, moeda, fuso e pregão) antes
nucleo-ativos-erro-mercado = mercado { $mercado } não existe: use uma bolsa do catálogo ou um destes: { $lista }
nucleo-ativos-erro-bolsa-falta = informe a bolsa (LSE, XETRA, TSE…) de ações e ETFs de outras bolsas
nucleo-ativos-erro-duplicado = o ativo { $ticker } já está cadastrado
nucleo-ativos-erro-moeda-invalida = moeda inválida: { $moeda }
nucleo-ativos-erro-moeda-em-uso = o ativo já tem operações ou proventos nesta moeda; crie outro ativo
nucleo-ativos-erro-bolsa-catalogo = ações e ETFs de outras bolsas precisam de uma bolsa do catálogo (LSE, XETRA, TSE…)
nucleo-ativos-erro-apagar-em-uso = o ativo tem operações ou proventos; arquive em vez de apagar

## Busca de tickers

nucleo-ativos-tipo-acao = Ação
nucleo-ativos-tipo-fundo = Fundo
nucleo-ativos-erro-busca-yahoo = busca do Yahoo indisponível: { $erro }

## Bolsas

nucleo-ativos-ref-bolsa = bolsa { $bolsa }
nucleo-ativos-motivo-bolsa-salva = Bolsa salva: { $bolsa }
nucleo-ativos-motivo-bolsa-removida = Bolsa removida: { $bolsa }
nucleo-ativos-erro-horario = horário inválido: { $horario } (use HH:MM)
nucleo-ativos-erro-fuso = fuso horário desconhecido: { $fuso }
nucleo-ativos-erro-bolsa-codigo = código da bolsa: até 12 letras e números (ex.: LSE)
nucleo-ativos-erro-bolsa-nome = informe o nome da bolsa
nucleo-ativos-erro-bolsa-pais = país: código ISO de 2 letras (ex.: GB)
nucleo-ativos-erro-bolsa-abertura = a abertura precisa ser antes do fechamento
nucleo-ativos-erro-bolsa-almoco = o almoço precisa ficar dentro do pregão
nucleo-ativos-erro-bolsa-dias = dias de pregão: dígitos de 1 (segunda) a 7 (domingo)
nucleo-ativos-erro-bolsa-sufixo = sufixo do Yahoo começa com ponto (ex.: .L)
nucleo-ativos-erro-bolsa-do-catalogo = bolsas do catálogo não podem ser apagadas (ajuste o horário, se precisar)
nucleo-ativos-erro-bolsa-com-ativos = há ativos nesta bolsa

## Cotações

nucleo-ativos-erro-chaves-so-interface = chaves de fontes de dados só pela interface do app
nucleo-ativos-erro-chave-fred = a chave da FRED tem 32 letras e números
nucleo-ativos-erro-token-brapi = o token da brapi.dev tem letras, números, hífen ou sublinhado
nucleo-ativos-erro-preco-invalido = preço inválido: { $preco }
nucleo-ativos-erro-preco-positivo = o preço deve ser positivo
nucleo-ativos-erro-preco-banco = preço inválido no banco: { $preco }
nucleo-ativos-erro-intradiario-periodo = período intradiário: 1d ou 5d
nucleo-ativos-erro-moeda-yahoo = moeda do Yahoo ({ $moeda_yahoo }) difere da do ativo ({ $moeda_ativo })
nucleo-ativos-sem-cotacao-manual = cotação manual: registre os preços no ativo
nucleo-ativos-sem-cotacao-fundo-sem-cnpj = fundo sem CNPJ: informe o CNPJ para buscar a cota na CVM
nucleo-ativos-sem-cotacao-sem-ticker = sem ticker: informe o ticker ou o símbolo do provedor no ativo
nucleo-ativos-sem-cotacao-sem-bolsa = sem bolsa do catálogo: informe a bolsa (LSE, XETRA, TSE…) no ativo
nucleo-ativos-sem-cotacao-sem-provedor = classe sem provedor padrão: informe o símbolo do Yahoo no ativo
nucleo-ativos-motivo-desdobramento-yahoo = Desdobramento informado pelo Yahoo: { $ativo }
nucleo-ativos-nota-bonificacao-yahoo = Bonificação informada automaticamente pela coleta de cotações (o Yahoo a trata como desdobramento); informe o custo atribuído pela empresa, se houver
nucleo-ativos-nota-evento-automatico = Informado automaticamente pela coleta de cotações
nucleo-ativos-erro-yahoo-sem-resultado = Yahoo: sem resultado
nucleo-ativos-erro-yahoo-sem-preco = Yahoo: cotação sem preço
nucleo-ativos-erro-yahoo-sem-horario = Yahoo: cotação sem horário
nucleo-ativos-erro-yahoo-intervalo = intervalo { $intervalo }/{ $periodo } não suportado
nucleo-ativos-erro-yahoo-busca = resposta da busca do Yahoo inválida: { $erro }
nucleo-ativos-erro-cvm-layout = layout do informe diário da CVM não reconhecido
nucleo-ativos-erro-cvm-status = CVM respondeu { $status } para { $url }
nucleo-ativos-erro-cvm-fundo-ausente = fundo { $fundo } não encontrado no informe diário da CVM
nucleo-ativos-erro-tesouro-titulo-ausente = título não encontrado no Tesouro Transparente: { $titulo }
nucleo-ativos-erro-tesouro-colunas = layout do CSV mudou: faltam colunas esperadas ({ $colunas })
nucleo-ativos-erro-tesouro-poucas-linhas = CSV com poucas linhas válidas ({ $linhas }): o arquivo pode estar incompleto ou ter mudado
nucleo-ativos-erro-tesouro-linhas-ruins = { $ruins } de { $total } linhas ilegíveis ou com preço fora da faixa: o formato pode ter mudado
nucleo-ativos-erro-tesouro-parado = dados parados: o preço mais recente é de { $data }
nucleo-ativos-erro-tesouro-http = Tesouro Transparente respondeu HTTP { $status }
nucleo-ativos-erro-tesouro-pagina = o Tesouro Transparente devolveu uma página no lugar do CSV

## Índices: catálogo de países

nucleo-ativos-erro-pais-fora = país fora do catálogo: { $pais }
nucleo-ativos-erro-pais-so-interface = o país de referência só muda pela interface do app
nucleo-ativos-pais-nota-br = Comportamento de sempre: IPCA, CDI/Selic e PTAX do Banco Central.
nucleo-ativos-pais-nota-us = CPI-U da FRED exige a chave gratuita; sem ela, o CPI da OCDE (sem chave) é usado.
nucleo-ativos-pais-nota-banco-mundial = Inflação anual do Banco Mundial, interpolada mês a mês (estimada).

## Índices: provedores (nome e termos de uso)

nucleo-ativos-provedor-bcb_sgs-nome = Banco Central do Brasil (SGS, PTAX, Focus)
nucleo-ativos-provedor-bcb_sgs-termos = Dados abertos do BCB, uso livre com citação da fonte.
nucleo-ativos-provedor-fred-nome = FRED (Federal Reserve Bank of St. Louis)
nucleo-ativos-provedor-fred-termos = Gratuito com chave pessoal; cada série segue os termos da fonte original.
nucleo-ativos-provedor-yahoo-nome = Yahoo Finance
nucleo-ativos-provedor-yahoo-termos = Fonte não oficial, sem garantia; usada só para fechamentos diários.
nucleo-ativos-provedor-ecb-nome = Banco Central Europeu (ECB Data Portal)
nucleo-ativos-provedor-ecb-termos = Reutilização livre com citação da fonte (termos do ECB Data Portal).
nucleo-ativos-provedor-eurostat-nome = Eurostat
nucleo-ativos-provedor-eurostat-termos = Reutilização livre com citação (política de dados abertos da Comissão Europeia, CC BY 4.0).
nucleo-ativos-provedor-oecd-nome = OCDE (OECD Data Explorer)
nucleo-ativos-provedor-oecd-termos = Dados da OCDE sob CC BY 4.0; a API pública limita o número de consultas por hora.
nucleo-ativos-provedor-boc-nome = Bank of Canada (Valet)
nucleo-ativos-provedor-boc-termos = Uso livre com citação (termos do Bank of Canada).
nucleo-ativos-provedor-ons-nome = Office for National Statistics (Reino Unido)
nucleo-ativos-provedor-ons-termos = Open Government Licence v3.0.
nucleo-ativos-provedor-boe-nome = Bank of England (Database)
nucleo-ativos-provedor-boe-termos = Uso livre com citação (termos do Bank of England).
nucleo-ativos-provedor-worldbank-nome = Banco Mundial (World Development Indicators)
nucleo-ativos-provedor-worldbank-termos = CC BY 4.0. Inflação anual; os meses são interpolados e ficam marcados como estimados.
nucleo-ativos-provedor-ibge_sidra-nome = IBGE (SIDRA)
nucleo-ativos-provedor-ibge_sidra-termos = Dados públicos do IBGE, uso livre com citação da fonte.
nucleo-ativos-provedor-nyfed-nome = Federal Reserve Bank of New York (Markets Data)
nucleo-ativos-provedor-nyfed-termos = Dados públicos do Federal Reserve Bank of New York, uso livre com citação da fonte.

## Índices: coleta

# Fontes femininas em português ficam com "da" (da FRED, da ONS, da PTAX); as outras, "do".
nucleo-ativos-erro-resposta-invalida = resposta { $fonte ->
    [FRED] da FRED
    [ONS] da ONS
    [PTAX] da PTAX
   *[outra] do { $fonte }
} inválida: { $erro }
nucleo-ativos-erro-resposta-invalida-sem-detalhe = resposta { $fonte ->
    [FRED] da FRED
    [ONS] da ONS
    [PTAX] da PTAX
   *[outra] do { $fonte }
} inválida
nucleo-ativos-aviso-sem-chave-fred = para baixar CPI, M2 e Fed Funds dos EUA, cadastre a chave gratuita da FRED em Ajustes › Fontes de dados
nucleo-ativos-erro-numero = número inválido: { $numero }
nucleo-ativos-erro-periodo = período inválido: { $periodo }
nucleo-ativos-erro-decimal-banco = decimal inválido no banco: { $valor }
nucleo-ativos-erro-fonte-desconhecida = fonte desconhecida: { $fonte }
nucleo-ativos-erro-adaptador-proprio = { $fonte } é coletado pelo adaptador próprio
nucleo-ativos-erro-oecd-sem-resposta = OCDE sem resposta
nucleo-ativos-erro-codigo-ons = código ONS inválido: { $codigo }
nucleo-ativos-erro-codigo-sidra = código SIDRA inválido: { $codigo }
nucleo-ativos-erro-sgs-data = data do SGS inválida: { $data }
nucleo-ativos-erro-boc-sem-observacoes = resposta do Bank of Canada sem observações
nucleo-ativos-erro-eurostat-sem-tempo = resposta do Eurostat sem a dimensão de tempo
nucleo-ativos-erro-eurostat-varias-series = a consulta ao Eurostat deve escolher uma série só (geo, unit e coicop18)
nucleo-ativos-erro-eurostat-indice-tempo = índice de tempo do Eurostat inválido
nucleo-ativos-erro-eurostat-valor = valor do Eurostat inválido: { $valor }
nucleo-ativos-erro-eurostat-posicao = posição do Eurostat inválida
nucleo-ativos-erro-eurostat-valores = valores do Eurostat inválidos
nucleo-ativos-erro-nyfed-sem-taxas = resposta do NY Fed sem taxas
nucleo-ativos-erro-sdmx-nao-csv = resposta SDMX inesperada (não é CSV)
nucleo-ativos-erro-sdmx-csv = CSV SDMX inválido: { $erro }
nucleo-ativos-erro-sdmx-coluna = CSV SDMX sem a coluna { $coluna }
nucleo-ativos-erro-ons-mes = mês da ONS inválido: { $mes }
nucleo-ativos-erro-boe-nao-csv = resposta do Bank of England inesperada (não é CSV)
nucleo-ativos-erro-boe-data = data do Bank of England inválida: { $data }

## Dados dos ativos com fonte (enriquecimento, fase 1)

nucleo-ativos-motivo-dados = Dados dos ativos completados pelas fontes
nucleo-ativos-motivo-campo-travado = Campo do ativo travado para sugestões
nucleo-ativos-motivo-campo-destravado = Campo do ativo destravado para sugestões
nucleo-ativos-dados-erro-campo = campo de dados do ativo desconhecido: { $campo }
nucleo-ativos-dados-erro-vazio = nenhum campo escolhido
nucleo-ativos-dados-erro-repetido = o campo { $campo } de { $ativo } aparece mais de uma vez
nucleo-ativos-dados-erro-travado = o campo { $campo } de { $ativo } está travado para sugestões
nucleo-ativos-dados-erro-sem-fonte = o valor "{ $valor }" para { $campo } de { $ativo } não está em nenhuma fonte registrada no cofre
nucleo-ativos-dados-erro-preenchido = o campo { $campo } de { $ativo } já tem um valor seu; para trocar, peça a troca (replace)
nucleo-ativos-dados-efeito-score = o score passa a comparar com os pares de { $segmento }

## Estudo com IA

nucleo-ativos-estudo-ref = estudo { $id }
nucleo-ativos-estudo-erro-contexto = o contexto para a IA pode ter no máximo { $max } caracteres
nucleo-ativos-estudo-erro-pergunta = a pergunta pode ter no máximo { $max } caracteres
nucleo-ativos-estudo-erro-sem-motor = Nenhum motor de IA configurado. Cadastre um em Ajustes › IA e agente e ponha-o na rota "Estudo de ativos" ou na do chat.
nucleo-ativos-estudo-erro-formato = O motor respondeu fora do formato do estudo. Tente de novo ou escolha outro motor.
nucleo-ativos-estudo-erro-aviso = Antes do primeiro estudo, leia e aceite o aviso na aba Estudo (IA) do ativo.
nucleo-ativos-estudo-erro-rodando = Já há um estudo sendo gerado; espere ele terminar.
nucleo-ativos-estudo-erro-so-interface = só pela interface do app
nucleo-ativos-estudo-titulo-job = Estudo com IA
nucleo-ativos-motivo-estudo = Estudo com IA: { $ativo }
nucleo-ativos-motivo-contexto-estudo = Contexto para a IA do ativo
nucleo-ativos-motivo-estudo-apagado = Estudo com IA apagado
nucleo-ativos-motivo-estudo-pedido = Estudo com IA pedido pelo agente: { $ativo }
nucleo-ativos-estudo-pronto = Estudo pronto
nucleo-ativos-consulta-cvm-sem-registro = O registro de fundos da CVM ainda não foi baixado: use "Coletar agora" em Coleta da análise de ativos e consulte de novo.

## brapi.dev

nucleo-ativos-brapi-sem-token = brapi.dev sem token: cadastre um em Ajustes › Fontes de dados.
nucleo-ativos-erro-cripto-sem-cambio = sem câmbio do dólar para { $moeda }: a cotação de cripto vem em dólar (rode a coleta de índices ou informe o câmbio)
nucleo-ativos-erro-fonte-sem-chave = sem a chave da fonte: cadastre em Ajustes › Fontes de dados
nucleo-ativos-erro-moeda-fonte = moeda da fonte ({ $moeda_fonte }) difere da do ativo ({ $moeda_ativo })
nucleo-ativos-motivo-desdobramento-coleta = Desdobramento informado pela coleta de cotações: { $ativo }
nucleo-ativos-erro-eodhd-rede = não foi possível falar com a EODHD
nucleo-ativos-erro-chave-fonte = chave inválida: confira o valor copiado do site da fonte
nucleo-ativos-brapi-ja-hoje = brapi.dev: cotação deste papel já buscada hoje (uma por dia, para poupar a cota).
nucleo-ativos-brapi-limite = brapi.dev: limite de uso do mês atingido; a cotação volta depois da renovação ou com um limite maior em Ajustes › Fontes de dados.
nucleo-ativos-erro-limite-brapi = o limite de uso da brapi vai de 10% a 100% da cota
nucleo-ativos-erro-conta-custodia = conta de custódia não encontrada: { $conta }
nucleo-ativos-erro-custodia-cartao = a conta onde o ativo fica não pode ser um cartão de crédito
