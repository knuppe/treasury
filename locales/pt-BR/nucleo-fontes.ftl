# Fontes externas de dados (núcleo: `sources.rs`): nome e uso de cada fonte em Ajustes › Fontes
# de dados e os erros de quando ela está desligada, no idioma da instalação. pt-BR é o idioma de
# origem. Nomes próprios (Yahoo Finance, Tesouro Direto, Bank of Canada…) ficam iguais.

## Nome e uso de cada fonte

nucleo-fontes-yahoo-nome = Yahoo Finance
nucleo-fontes-yahoo-uso = Cotações diárias e intradiárias, proventos, desdobramentos e busca de tickers.
nucleo-fontes-fundamentus-nome = Fundamentus
nucleo-fontes-fundamentus-uso = Indicadores (P/L, P/VP, DY, ROE…), proventos e dados técnicos de ações e FIIs, com cache local.
nucleo-fontes-tesouro-nome = Tesouro Direto
nucleo-fontes-tesouro-uso = Preços e taxas dos títulos públicos.
nucleo-fontes-feedback-titulo = Coleta de { $fonte } falhando
nucleo-fontes-feedback-corpo-passageira = A coleta de { $fonte } falhou { $dias } dias seguidos com erro de rede ou do site (a fonte continua ligada, sem risco de gravar dado errado). Último erro: { $erro }. Investigar: o site ({ $site }) mudou o endereço, passou a limitar ou bloquear pedidos, ou o problema é da rede de quem usa (filtro, proxy, antivírus)?
nucleo-fontes-feedback-corpo-desligada = A coleta de { $fonte } falhou { $dias } dias seguidos e a fonte foi desligada para não gravar dados errados. Último erro: { $erro }. Investigar se o site ({ $site }) mudou o endereço, o formato ou as colunas do arquivo.
nucleo-fontes-feedback-nota = Ainda falhando ({ $dias } dias seguidos): { $erro }
nucleo-fontes-feedback-resolvido = A coleta voltou a funcionar em { $data }.
nucleo-fontes-cvm-nome = CVM Dados Abertos
nucleo-fontes-cvm-uso = Cotas de fundos (informe diário), demonstrações de companhias, informes de FIIs e documentos.
nucleo-fontes-b3-nome = B3 (COTAHIST e empresas listadas)
nucleo-fontes-b3-uso = Fechamento e volume do mercado à vista e proventos e eventos em ações oficiais (dividendos, JCP, rendimentos de FII, desdobramentos) para o universo da análise de ativos.
nucleo-fontes-bcb-nome = Banco Central do Brasil
nucleo-fontes-bcb-uso = Índices (CDI, Selic, IPCA, TR…), câmbio PTAX, Focus, a lista oficial de bancos e os indicadores prudenciais dos bancos (IF.data).
nucleo-fontes-sec-nome = SEC EDGAR
nucleo-fontes-sec-uso = Fundamentos de empresas americanas.
nucleo-fontes-fred-nome = FRED (St. Louis Fed)
nucleo-fontes-fred-uso = Índices dos Estados Unidos (inflação, juros).
nucleo-fontes-ecb-nome = Banco Central Europeu
nucleo-fontes-ecb-uso = Câmbio e juros da zona do euro.
nucleo-fontes-eurostat-nome = Eurostat
nucleo-fontes-eurostat-uso = Inflação dos países europeus.
nucleo-fontes-oecd-nome = OCDE
nucleo-fontes-oecd-uso = Inflação e juros de outros países do catálogo.
nucleo-fontes-boc-nome = Bank of Canada
nucleo-fontes-boc-uso = Índices do Canadá.
nucleo-fontes-ons-nome = ONS (Reino Unido)
nucleo-fontes-ons-uso = Inflação do Reino Unido.
nucleo-fontes-boe-nome = Bank of England
nucleo-fontes-boe-uso = Juros do Reino Unido.
nucleo-fontes-world_bank-nome = Banco Mundial
nucleo-fontes-world_bank-uso = Inflação anual de países sem fonte mensal.
nucleo-fontes-fipe-nome = Tabela FIPE
nucleo-fontes-fipe-uso = Avaliação de veículos do patrimônio físico.
nucleo-fontes-coingecko-nome = CoinGecko
nucleo-fontes-coingecko-uso = Valor de mercado, oferta, volume e desenvolvimento dos criptoativos cadastrados (API pública, sem chave).
nucleo-fontes-defillama-nome = DefiLlama
nucleo-fontes-defillama-uso = TVL das redes, lastro e paridade das stablecoins, preços em dólar dos criptoativos cadastrados e a lista de ataques às redes.
nucleo-fontes-coin_metrics-nome = Coin Metrics (Community)
nucleo-fontes-coin_metrics-uso = Endereços ativos por dia das redes cobertas pela API aberta (CC BY-NC 4.0), para o score de cripto.
nucleo-fontes-mempool-nome = mempool.space
nucleo-fontes-mempool-uso = Blocos da semana por pool de mineração do bitcoin, para o coeficiente de Nakamoto do score de cripto.
nucleo-fontes-brapi-nome = brapi.dev
nucleo-fontes-brapi-uso = Opcional, com token: segunda fonte de indicadores e proventos de ações e FIIs da B3, para conferir os dados oficiais (só os ativos cadastrados).
nucleo-fontes-tiingo-nome = Tiingo
nucleo-fontes-tiingo-uso = Opcional, com a sua chave gratuita: fechamento diário, dividendos e desdobramentos de ações, ETFs e REITs dos Estados Unidos, no lugar do Yahoo.
nucleo-fontes-eodhd-nome = EODHD
nucleo-fontes-eodhd-uso = Opcional, com a sua chave: fechamento diário, dividendos e desdobramentos de bolsas do mundo todo, no lugar do Yahoo. O plano gratuito da EODHD é só de teste.
nucleo-fontes-gleif-nome = GLEIF
nucleo-fontes-gleif-uso = LEI (identificador global de entidade legal) dos emissores brasileiros, pelo CNPJ, nos dados dos ativos.
nucleo-fontes-brasilapi-nome = BrasilAPI (Receita Federal)
nucleo-fontes-brasilapi-uso = Razão social e situação cadastral do CNPJ dos emissores e fundos, pelo espelho público do cadastro da Receita; só o CNPJ é enviado.
nucleo-fontes-ibge-nome = IBGE (SIDRA)
nucleo-fontes-ibge-uso = IPCA por grupo (alimentação, habitação, transportes…), para a inflação pessoal da família.
nucleo-fontes-sefaz-nome = SEFAZ (consulta da NFC-e)
nucleo-fontes-sefaz-uso = Itens, valores e tributos aproximados do cupom fiscal, pela consulta pública da secretaria da fazenda do estado (o link do QR do cupom).
nucleo-fontes-siconfi-nome = Tesouro Nacional (Siconfi)
nucleo-fontes-siconfi-uso = Arrecadação de tributos do município da família (IPTU, ISS, ITBI e taxas) e a população, pelo relatório que a prefeitura entrega ao Tesouro; só o código do município é enviado.
nucleo-fontes-mail_autoconfig-nome = Autoconfiguração de e-mail (Thunderbird)
nucleo-fontes-mail_autoconfig-uso = Servidor IMAP de um domínio fora do catálogo, no assistente do e-mail de ingestão: o autoconfig do próprio domínio e a base do Thunderbird (só o domínio é enviado, nunca o endereço).
nucleo-fontes-ny_fed-nome = Federal Reserve Bank of New York
nucleo-fontes-ny_fed-uso = Fed Funds efetiva (EFFR), diária e sem chave: o juro de referência das famílias que moram nos Estados Unidos.

## Erros

nucleo-fontes-erro-desligada = a fonte { $fonte } está desligada em Ajustes › Fontes de dados
nucleo-fontes-erro-so-interface = ligar e desligar fontes de dados só pela interface do app
nucleo-fontes-open_food_facts-nome = Open Food Facts
nucleo-fontes-open_food_facts-uso = Nome, marca, informação nutricional, grupo NOVA e alérgenos dos produtos pelo código de barras; só o código vai, uma vez por produto. Dados abertos (ODbL), indicativos, sem uso médico.
nucleo-fontes-news_feeds-nome = Portais de notícias (RSS)
nucleo-fontes-news_feeds-uso = Manchetes de InfoMoney, Money Times, Seu Dinheiro, E-Investidor e Suno para a aba Notícias do ativo. O pedido é o mesmo para todos: o filtro pelo ticker é feito no computador, então nada da carteira sai. Só título, portal, data e link.

## Termos, licença e atribuição de cada fonte (Ajustes › Fontes de dados e Sobre). Os textos em
## inglês são os avisos que a própria fonte exige, palavra por palavra: não traduzir.

nucleo-fontes-yahoo-termos = Fonte não oficial. O Yahoo não oferece API pública, e os termos de uso dele não autorizam coleta automática nem guardar os dados. Com a fonte ligada, as consultas saem do seu computador por sua escolha; desligue se quiser só fontes oficiais.
nucleo-fontes-fundamentus-termos = O site não publica termos de uso. O Treasury lê só as páginas dos seus ativos, no máximo uma vez por dia, e não redistribui os dados.
nucleo-fontes-tesouro-termos = Fonte: Tesouro Nacional, Tesouro Transparente. Dados abertos sob a licença ODbL.
nucleo-fontes-cvm-termos = Fonte: CVM, Portal de Dados Abertos. Dados abertos sob a licença ODbL.
nucleo-fontes-b3-termos = Dados da B3 para uso exclusivamente pessoal, como pedem os termos do site. Ficam só no seu cofre e não são redistribuídos.
nucleo-fontes-sec-termos = Fonte: SEC, EDGAR. Dados públicos; a SEC pede a identificação de quem consulta, com um e-mail de contato.
nucleo-fontes-bcb-termos = Fonte: Banco Central do Brasil, dados abertos.
nucleo-fontes-fred-termos = This product uses the FRED® API but is not endorsed or certified by the Federal Reserve Bank of St. Louis.
nucleo-fontes-ecb-termos = Fonte: Banco Central Europeu (BCE). Reuso livre com citação da fonte.
nucleo-fontes-eurostat-termos = Fonte: Eurostat. Licença CC BY 4.0.
nucleo-fontes-oecd-termos = Fonte: OCDE. Licença CC BY 4.0.
nucleo-fontes-boc-termos = Fonte: Bank of Canada. Os valores marcados como estimados são cálculo do Treasury.
nucleo-fontes-ons-termos = Fonte: Office for National Statistics. Contains public sector information licensed under the Open Government Licence v3.0.
nucleo-fontes-boe-termos = Fonte: Bank of England. SONIA and/or SONIA Compounded Index data licensed under the Open Government Licence v3.0 and copyright the Governor and Company of the Bank of England. The trade marks “Bank of England” and “SONIA” are registered trade marks of the Bank of England.
nucleo-fontes-world_bank-termos = Fonte: Banco Mundial, World Development Indicators. Licença CC BY 4.0. A série mensal é uma estimativa do Treasury a partir do dado anual.
nucleo-fontes-fipe-termos = A Tabela FIPE é da Fundação Instituto de Pesquisas Econômicas (FIPE). A consulta passa por parallelum.com.br, um serviço não oficial; a consulta oficial fica em veiculos.fipe.org.br.
nucleo-fontes-coingecko-termos = Dados fornecidos pela CoinGecko (coingecko.com).
nucleo-fontes-defillama-termos = Fonte: DefiLlama (defillama.com), API aberta.
nucleo-fontes-coin_metrics-termos = Fonte: Coin Metrics (coinmetrics.io), dados Community sob a licença CC BY-NC 4.0, para uso não comercial.
nucleo-fontes-mempool-termos = Fonte: mempool.space, projeto de código aberto.
nucleo-fontes-brapi-termos = Uso pessoal, com o seu token. Os dados ficam no seu cofre e não são redistribuídos.
nucleo-fontes-ibge-termos = Fonte: IBGE, Sistema IBGE de Recuperação Automática (SIDRA).
nucleo-fontes-ny_fed-termos = © Federal Reserve Bank of New York. The Effective Federal Funds Rate (EFFR) is subject to the Terms of Use posted at newyorkfed.org. The New York Fed is not responsible for publication of the EFFR by Treasury, does not sanction or endorse any particular republication, and has no liability for your use.
nucleo-fontes-sefaz-termos = Consulta pública da nota, pelo link do QR do seu cupom, feita a seu pedido.
nucleo-fontes-siconfi-termos = Fonte: Tesouro Nacional, Siconfi. Dados abertos, disponibilizados para livre utilização; os números são os declarados por cada prefeitura.
nucleo-fontes-mail_autoconfig-termos = Base pública de configuração de e-mail do Thunderbird (ISPDB).
nucleo-fontes-gleif-termos = Fonte: GLEIF. Dados de LEI em domínio público (CC0).
nucleo-fontes-brasilapi-termos = Fonte: BrasilAPI, projeto aberto, com dados públicos do CNPJ.
nucleo-fontes-open_food_facts-termos = Fonte: Open Food Facts e seus contribuidores. Base sob a licença ODbL.
nucleo-fontes-news_feeds-termos = Manchetes dos feeds RSS públicos de cada portal. O texto é do veículo, e o link leva à matéria no site dele.
nucleo-fontes-tiingo-termos = Uso pessoal, com a sua chave: os termos do plano gratuito da Tiingo não permitem mostrar nem repassar os dados a outras pessoas. Eles ficam só no seu cofre.
nucleo-fontes-eodhd-termos = Uso pessoal, não profissional, com a sua chave: os termos da EODHD permitem guardar e analisar os dados para uso privado e proíbem repassá-los. Eles ficam só no seu cofre.
