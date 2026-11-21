# Índices y cotizaciones: series de índices, gráfico, tipo de cambio, clave de FRED e informe de
# la recolección (docs/i18n-design.md). Revisión: borrador por IA, sin revisión de un hablante nativo.

## Para qué sirve cada serie

indices-uso-ipca = Deflactor predeterminado (BRL) y Tesouro IPCA+
indices-uso-igpm = Contratos y bonos indexados al IGP-M
indices-uso-cdi = Renta fija de tasa variable y referencia
indices-uso-selic = Tesouro Selic y bonos de la Selic
indices-uso-selic-meta = Meta definida por el Copom
indices-uso-tr = Ahorro y financiamientos
indices-uso-m2-br = Deflactor alternativo: expansión monetaria de Brasil
indices-uso-us-cpi = Deflactor de activos en USD
indices-uso-us-m2 = Deflactor alternativo: expansión monetaria de EE. UU.
indices-uso-fed-funds = Tasa de interés de referencia de EE. UU.
indices-uso-ibov = Referencia de acciones de Brasil
indices-uso-sp500 = Referencia de acciones de EE. UU.
indices-uso-estr = Tasa de referencia de la zona del euro
indices-uso-corra = Tasa de referencia de Canadá
indices-uso-sonia = Tasa de referencia del Reino Unido
indices-uso-boe-bank-rate = Tasa de interés oficial del Reino Unido
indices-uso-hicp-ea = Inflación de la zona del euro
indices-uso-cpi-ca = Inflación de Canadá
indices-uso-cpi-gb = Inflación del Reino Unido
indices-uso-cpih-gb = Inflación del Reino Unido con vivienda
indices-uso-cpi-us-oecd = Inflación de EE. UU. sin clave (alternativa a FRED)
indices-uso-inflacao-pais = Inflación del país
indices-uso-wb-cpi = Inflación anual (Banco Mundial), meses estimados
indices-uso-rate3m = Tasa a 3 meses (referencia)
indices-uso-fx = Tipo de cambio de referencia

## Tabla de series

indices-freq-diaria = Diaria
indices-freq-mensal = Mensual
indices-serie = Serie
indices-unidade = Unidad
indices-freq = Frec.
indices-oficial-ate = Oficial hasta
indices-estimado-ate = Estimado hasta
indices-pontos = Puntos
indices-coletado-em = Obtenido el
indices-series = Series de índices
indices-figura-series = { $figura } · Series de índices
indices-nenhuma-serie = Ninguna serie registrada.
indices-selecione = Selecciona una serie (↑/↓) para ver el gráfico. Los estimados aparecen en cursiva con ≈ hasta que sale el dato oficial.

## Gráfico de una serie

indices-oficial = Oficial
indices-estimado = Estimado
indices-titulo-meses = { $serie }, { $n } meses
indices-titulo-anos = { $serie }, { $n } años
indices-periodo = Período de la serie
indices-periodo-5a = 5A
indices-fonte-unidade = Fuente: { $fonte } · { $unidade }
indices-nota-ipca = Los meses todavía no divulgados usan la proyección Focus (o el último dato) hasta que el IBGE publique.
indices-nota-wb = Inflación anual del Banco Mundial distribuida mes a mes: todos los puntos son estimados.
indices-sem-valores = Sin valores en el período. Actualiza los índices.
indices-grafico-aria = Serie { $serie } en el período, en { $unidade }.
indices-valores-de = Valores de { $serie }
indices-status = Estado

## Informe de la recolección

indices-gravados = Guardados
indices-ultimo-oficial = Último oficial
indices-resultado = Resultado
indices-ok = ok
indices-cotacoes = Cotizaciones
indices-desdobramentos = Desdobl.
indices-ultimo-pregao = Última sesión
indices-relatorio = Informe de la recolección
indices-coleta-concluida = Recolección terminada
indices-coleta-falhas = { $n ->
    [one] Recolección terminada con { $n } falla
   *[other] Recolección terminada con { $n } fallas
}
indices-coleta-resumo = { $inicio } → { $fim } · { $series } series, { $moedas } monedas y { $ativos } activos.
indices-ipca-estimado = { $n ->
    [one] {" "}IPCA con { $n } mes estimado (Focus).
   *[other] {" "}IPCA con { $n } meses estimados (Focus).
}
indices-indices-cambio = Índices y tipo de cambio
indices-coleta-indices = Recolección de índices y tipo de cambio
indices-coleta-cotacoes = Recolección de cotizaciones
indices-sem-fonte = Ningún activo con operaciones o en la watchlist tiene fuente automática de cotización.

## Clave de FRED

indices-fred = Clave de FRED
indices-fred-cadastrada = Clave de FRED registrada
indices-fred-removida = Clave de FRED eliminada
indices-trocar = Cambiar
indices-fred-rotulo = Clave de FRED (EE. UU.)
indices-fred-dica = Gratuita en fred.stlouisfed.org; necesaria para US CPI, US M2 y Fed Funds. Queda cifrada en la bóveda y nunca vuelve a la interfaz.
indices-salvar-chave = Guardar clave
indices-sem-fred-titulo = Series de EE. UU. sin clave de FRED
indices-sem-fred-texto = US CPI, US M2 y Fed Funds solo se obtienen con la clave gratuita de FRED; sin ellas, los deflactores US CPI y M2 de EE. UU. quedan sin datos.

## Recolección de datos de mercado

indices-coleta-titulo = Recolección de datos de mercado
indices-coletando = Recolectando
indices-coletando-reticencias = Recolectando…
indices-ultima-coleta = Última recolección
indices-rotina = Rutina
indices-rotina-valor = cada 12 h con la app abierta
indices-coleta-ajuda = Índices del Banco Central de Brasil (IPCA, CDI, Selic, IGP-M, TR, M2), tipo de cambio PTAX, series de EE. UU. en FRED y cotizaciones de cierre (Yahoo Finance para la B3, EE. UU. y cripto; Tesouro Transparente para el Tesouro Direto). Cada valor guarda la fuente y la fecha de recolección.
indices-atualizados = Índices, tipo de cambio y cotizaciones actualizados
indices-concluida-com-falhas = Recolección terminada con fallas
indices-coleta-falhou = La recolección falló
indices-atualizar = Actualizar índices, tipo de cambio y cotizaciones
indices-pode-demorar = Puede tardar unos minutos
indices-cambio-fontes = Tipo de cambio y fuentes
indices-em-data = el { $data }
