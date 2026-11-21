# Fuentes externas de datos (núcleo: `sources.rs`): nombre y uso de cada fuente en Ajustes ›
# Fuentes de datos y los errores de cuando está apagada, en el idioma de la instalación. Los
# nombres propios (Yahoo Finance, Tesouro Direto, Bank of Canada…) quedan iguales.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Nombre y uso de cada fuente

nucleo-fontes-yahoo-nome = Yahoo Finance
nucleo-fontes-yahoo-uso = Cotizaciones diarias e intradía, dividendos, desdoblamientos y búsqueda de tickers.
nucleo-fontes-fundamentus-nome = Fundamentus
nucleo-fontes-fundamentus-uso = Indicadores (P/U, P/VL, DY, ROE…), dividendos y datos técnicos de acciones y FII, con caché local.
nucleo-fontes-tesouro-nome = Tesouro Direto
nucleo-fontes-tesouro-uso = Precios y tasas de los bonos del gobierno de Brasil.
nucleo-fontes-feedback-titulo = Recogida de { $fonte } fallando
nucleo-fontes-feedback-corpo-passageira = La recogida de { $fonte } falló { $dias } días seguidos con un error de red o del sitio (la fuente sigue activa, sin riesgo de grabar datos erróneos). Último error: { $erro }. Investigar: ¿el sitio ({ $site }) cambió la dirección, empezó a limitar o bloquear pedidos, o el problema es la red de quien lo usa (filtro, proxy, antivirus)?
nucleo-fontes-feedback-corpo-desligada = La recogida de { $fonte } falló { $dias } días seguidos y la fuente se desactivó para no grabar datos erróneos. Último error: { $erro }. Revisar si el sitio ({ $site }) cambió la dirección, el formato o las columnas del archivo.
nucleo-fontes-feedback-nota = Sigue fallando ({ $dias } días seguidos): { $erro }
nucleo-fontes-feedback-resolvido = La recogida volvió a funcionar el { $data }.
nucleo-fontes-cvm-nome = CVM Datos Abiertos
nucleo-fontes-cvm-uso = Cuotas de fondos (informe diario), estados financieros de compañías, informes de FII y documentos.
nucleo-fontes-b3-nome = B3 (COTAHIST y empresas listadas)
nucleo-fontes-b3-uso = Cierre y volumen del mercado al contado y dividendos y eventos corporativos oficiales (dividendos, JCP, rendimientos de FII, desdoblamientos) para el universo del análisis de activos.
nucleo-fontes-bcb-nome = Banco Central de Brasil
nucleo-fontes-bcb-uso = Índices (CDI, Selic, IPCA, TR…), tipo de cambio PTAX, Focus, la lista oficial de bancos y los indicadores prudenciales de los bancos (IF.data).
nucleo-fontes-sec-nome = SEC EDGAR
nucleo-fontes-sec-uso = Fundamentos de empresas estadounidenses.
nucleo-fontes-fred-nome = FRED (St. Louis Fed)
nucleo-fontes-fred-uso = Índices de Estados Unidos (inflación, tasas de interés).
nucleo-fontes-ecb-nome = Banco Central Europeo
nucleo-fontes-ecb-uso = Tipo de cambio y tasas de interés de la zona del euro.
nucleo-fontes-eurostat-nome = Eurostat
nucleo-fontes-eurostat-uso = Inflación de los países europeos.
nucleo-fontes-oecd-nome = OCDE
nucleo-fontes-oecd-uso = Inflación y tasas de interés de otros países del catálogo.
nucleo-fontes-boc-nome = Bank of Canada
nucleo-fontes-boc-uso = Índices de Canadá.
nucleo-fontes-ons-nome = ONS (Reino Unido)
nucleo-fontes-ons-uso = Inflación del Reino Unido.
nucleo-fontes-boe-nome = Bank of England
nucleo-fontes-boe-uso = Tasas de interés del Reino Unido.
nucleo-fontes-world_bank-nome = Banco Mundial
nucleo-fontes-world_bank-uso = Inflación anual de países sin fuente mensual.
nucleo-fontes-fipe-nome = Tabla FIPE
nucleo-fontes-fipe-uso = Valuación de los vehículos del patrimonio físico.
nucleo-fontes-coingecko-nome = CoinGecko
nucleo-fontes-coingecko-uso = Capitalización de mercado, oferta, volumen y desarrollo de los criptoactivos registrados (API pública, sin clave).
nucleo-fontes-defillama-nome = DefiLlama
nucleo-fontes-defillama-uso = TVL de las redes, respaldo y paridad de las stablecoins, precios en dólares de los criptoactivos registrados y la lista de ataques a las redes.
nucleo-fontes-coin_metrics-nome = Coin Metrics (Community)
nucleo-fontes-coin_metrics-uso = Direcciones activas por día de las redes que cubre la API abierta (CC BY-NC 4.0), para el score de cripto.
nucleo-fontes-mempool-nome = mempool.space
nucleo-fontes-mempool-uso = Bloques de la semana por pool de minería de bitcoin, para el coeficiente de Nakamoto del score de cripto.
nucleo-fontes-brapi-nome = brapi.dev
nucleo-fontes-brapi-uso = Opcional, con token: segunda fuente de indicadores y dividendos de acciones y FII de la B3, para comprobar los datos oficiales (solo los activos registrados).
nucleo-fontes-tiingo-nome = Tiingo
nucleo-fontes-tiingo-uso = Opcional, con tu clave gratuita: cierre diario, dividendos y desdoblamientos de acciones, ETF y REIT de Estados Unidos, en lugar de Yahoo.
nucleo-fontes-eodhd-nome = EODHD
nucleo-fontes-eodhd-uso = Opcional, con tu clave: cierre diario, dividendos y desdoblamientos de bolsas de todo el mundo, en lugar de Yahoo. El plan gratuito de EODHD es solo de prueba.
nucleo-fontes-gleif-nome = GLEIF
nucleo-fontes-gleif-uso = LEI (identificador global de entidad legal) de los emisores brasileños, por el CNPJ, en los datos de los activos.
nucleo-fontes-brasilapi-nome = BrasilAPI (Receita Federal)
nucleo-fontes-brasilapi-uso = Razón social y situación del CNPJ de los emisores y fondos, por el espejo público del registro de la Receita Federal; solo se envía el CNPJ.
nucleo-fontes-ibge-nome = IBGE (SIDRA)
nucleo-fontes-ibge-uso = IPCA por grupo (alimentación, vivienda, transporte…), para la inflación personal de la familia.
nucleo-fontes-sefaz-nome = SEFAZ (consulta de la NFC-e)
nucleo-fontes-sefaz-uso = Artículos, montos e impuestos aproximados del comprobante fiscal, por la consulta pública de la secretaría de hacienda del estado (el enlace del QR del comprobante).
nucleo-fontes-siconfi-nome = Tesouro Nacional (Siconfi)
nucleo-fontes-siconfi-uso = Recaudación de tributos del municipio de la familia (IPTU, ISS, ITBI y tasas) y la población, por el informe que la alcaldía entrega al Tesoro; solo se envía el código del municipio.
nucleo-fontes-mail_autoconfig-nome = Autoconfiguración de correo (Thunderbird)
nucleo-fontes-mail_autoconfig-uso = Servidor IMAP de un dominio fuera del catálogo, en el asistente del correo de importación: el autoconfig del propio dominio y la base de Thunderbird (solo se envía el dominio, nunca la dirección).
nucleo-fontes-ny_fed-nome = Federal Reserve Bank of New York
nucleo-fontes-ny_fed-uso = Fed Funds efectiva (EFFR), diaria y sin clave: la tasa de interés de referencia de las familias que viven en Estados Unidos.

## Errores

nucleo-fontes-erro-desligada = la fuente { $fonte } está apagada en Ajustes › Fuentes de datos
nucleo-fontes-erro-so-interface = las fuentes de datos solo se encienden y apagan desde la app
nucleo-fontes-open_food_facts-nome = Open Food Facts
nucleo-fontes-open_food_facts-uso = Nombre, marca, información nutricional, grupo NOVA y alérgenos de los productos por el código de barras; solo se envía el código, una vez por producto. Datos abiertos (ODbL), indicativos, sin uso médico.
nucleo-fontes-news_feeds-nome = Portales de noticias (RSS)
nucleo-fontes-news_feeds-uso = Titulares de InfoMoney, Money Times, Seu Dinheiro, E-Investidor y Suno para la pestaña Noticias del activo. El pedido es el mismo para todos: el filtro por ticker se hace en la computadora, así que nada de la cartera sale. Solo título, portal, fecha y enlace.

## Términos, licencia y atribución de cada fuente

nucleo-fontes-yahoo-termos = Fuente no oficial. Yahoo no ofrece una API pública, y sus términos de uso no autorizan la recolección automática ni guardar los datos. Con la fuente activada, las consultas salen de tu computadora por tu elección; desactívala si quieres solo fuentes oficiales.
nucleo-fontes-fundamentus-termos = El sitio no publica términos de uso. Treasury lee solo las páginas de tus activos, como máximo una vez al día, y no redistribuye los datos.
nucleo-fontes-tesouro-termos = Fuente: Tesoro Nacional de Brasil, Tesouro Transparente. Datos abiertos bajo la licencia ODbL.
nucleo-fontes-cvm-termos = Fuente: CVM (comisión de valores de Brasil), Portal de Datos Abiertos. Datos abiertos bajo la licencia ODbL.
nucleo-fontes-b3-termos = Datos de B3 para uso exclusivamente personal, como piden los términos del sitio. Quedan solo en tu bóveda y no se redistribuyen.
nucleo-fontes-sec-termos = Fuente: SEC, EDGAR. Datos públicos; la SEC pide que quien consulta se identifique con un correo de contacto.
nucleo-fontes-bcb-termos = Fuente: Banco Central do Brasil, datos abiertos.
nucleo-fontes-fred-termos = This product uses the FRED® API but is not endorsed or certified by the Federal Reserve Bank of St. Louis.
nucleo-fontes-ecb-termos = Fuente: Banco Central Europeo (BCE). Reutilización libre citando la fuente.
nucleo-fontes-eurostat-termos = Fuente: Eurostat. Licencia CC BY 4.0.
nucleo-fontes-oecd-termos = Fuente: OCDE. Licencia CC BY 4.0.
nucleo-fontes-boc-termos = Fuente: Bank of Canada. Los valores marcados como estimados son cálculo de Treasury.
nucleo-fontes-ons-termos = Fuente: Office for National Statistics. Contains public sector information licensed under the Open Government Licence v3.0.
nucleo-fontes-boe-termos = Fuente: Bank of England. SONIA and/or SONIA Compounded Index data licensed under the Open Government Licence v3.0 and copyright the Governor and Company of the Bank of England. The trade marks “Bank of England” and “SONIA” are registered trade marks of the Bank of England.
nucleo-fontes-world_bank-termos = Fuente: Banco Mundial, World Development Indicators. Licencia CC BY 4.0. La serie mensual es una estimación de Treasury a partir del dato anual.
nucleo-fontes-fipe-termos = La Tabla FIPE es de la Fundação Instituto de Pesquisas Econômicas (FIPE). La consulta pasa por parallelum.com.br, un servicio no oficial; la consulta oficial está en veiculos.fipe.org.br.
nucleo-fontes-coingecko-termos = Datos proporcionados por CoinGecko (coingecko.com).
nucleo-fontes-defillama-termos = Fuente: DefiLlama (defillama.com), API abierta.
nucleo-fontes-coin_metrics-termos = Fuente: Coin Metrics (coinmetrics.io), datos Community bajo la licencia CC BY-NC 4.0, para uso no comercial.
nucleo-fontes-mempool-termos = Fuente: mempool.space, proyecto de código abierto.
nucleo-fontes-brapi-termos = Uso personal, con tu propio token. Los datos quedan en tu bóveda y no se redistribuyen.
nucleo-fontes-ibge-termos = Fuente: IBGE, SIDRA.
nucleo-fontes-ny_fed-termos = © Federal Reserve Bank of New York. The Effective Federal Funds Rate (EFFR) is subject to the Terms of Use posted at newyorkfed.org. The New York Fed is not responsible for publication of the EFFR by Treasury, does not sanction or endorse any particular republication, and has no liability for your use.
nucleo-fontes-sefaz-termos = Consulta pública del comprobante, por el enlace del QR de tu ticket, hecha a tu pedido.
nucleo-fontes-siconfi-termos = Fuente: Tesouro Nacional, Siconfi. Datos abiertos, disponibles para libre utilización; las cifras son las declaradas por cada alcaldía.
nucleo-fontes-mail_autoconfig-termos = Base pública de configuración de correo de Thunderbird (ISPDB).
nucleo-fontes-gleif-termos = Fuente: GLEIF. Datos de LEI en dominio público (CC0).
nucleo-fontes-brasilapi-termos = Fuente: BrasilAPI, proyecto abierto, con datos públicos del registro de empresas (CNPJ).
nucleo-fontes-open_food_facts-termos = Fuente: Open Food Facts y sus contribuidores. Base bajo la licencia ODbL.
nucleo-fontes-news_feeds-termos = Titulares de los feeds RSS públicos de cada portal. El texto es del medio, y el enlace lleva a la nota en su sitio.
nucleo-fontes-tiingo-termos = Uso personal, con tu propia clave: los términos del plan gratuito de Tiingo no permiten mostrar ni pasar los datos a otras personas. Quedan solo en tu bóveda.
nucleo-fontes-eodhd-termos = Uso personal, no profesional, con tu propia clave: los términos de EODHD permiten guardar y analizar los datos para uso privado y prohíben pasarlos a terceros. Quedan solo en tu bóveda.
