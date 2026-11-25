# 09 · Activos: pestañas, lista, registro, página del activo, watchlist, calendario de dividendos,
# rendimientos por confirmar y previsión privada (docs/i18n-design.md).
# Revisión: borrador por IA, sin revisión de un hablante nativo.

## Pantalla y pestañas

ativos-cadastrar = Registrar activo
ativos-secoes = Secciones de activos
ativos-aba-watchlist = Watchlist
ativos-aba-agenda = Calendario de dividendos
ativos-aba-indices = Índices y cotizaciones
ativos-aba-indicadores = Indicadores
ativos-aba-rankings = Rankings
ativos-aba-screener = Screener
ativos-aba-comparador = Comparador
ativos-aba-dados = Datos del análisis
ativos-aba-bolsas = Bolsas

## Menú contextual del activo

ativos-menu-abrir = Abrir activo
ativos-menu-nova-operacao = Nueva operación
ativos-menu-registrar-provento = Registrar rendimiento
ativos-menu-evento-corporativo = Evento corporativo
ativos-menu-ver-extrato = Ver extracto
ativos-menu-operacoes = Operaciones
ativos-menu-proventos = Rendimientos
ativos-menu-grafico = Gráfico de precio
ativos-menu-fundamentos = Fundamentales
ativos-menu-acompanhar = Seguir en la watchlist…
ativos-menu-informar-cotacao = Ingresar cotización…
ativos-menu-comparar = Comparar con otros
ativos-menu-editar = Editar registro…

## Lista de activos

ativos-score-classe-sem-score = Clase todavía sin score
ativos-score-falta = Datos insuficientes: falta { $falta }
ativos-score-parcial = Score parcial (más del 30% del peso estimado) · confianza { $pct }%
ativos-score-faixa = { $faixa } · confianza { $pct }%
ativos-score-nota = Score { $nota } de 100: { $detalhe }
ativos-score-sem-nota = Score: { $detalhe }
ativos-score-sem-dados = sin datos
ativos-col-score = Score
ativos-col-ticker = Ticker
ativos-col-nome = Nombre
ativos-nome-oficial = Nombre oficial
ativos-nome-oficial-dica = Opcional. El nombre del registro (CVM, bolsa, aseguradora), guardado y buscable; la pantalla muestra el nombre de arriba.
ativos-nome-oficial-de = Nombre oficial: { $nome }
ativos-col-classe = Clase
ativos-col-mercado = Mercado
ativos-col-moeda = Moneda
ativos-col-setor-termos = Sector · términos
ativos-col-cotacao = Cotización
ativos-arquivado = archivado
ativos-buscar-rotulo = Buscar activo por ticker o nombre
ativos-buscar = Buscar por ticker o nombre…
ativos-filtrar-classe = Filtrar por clase
ativos-todas-classes = Todas las clases
ativos-filtrar-posicao = Filtrar por posición
ativos-posicao-aberta = En cartera
ativos-posicoes-encerradas = Posiciones cerradas
ativos-posicao-com-operacoes = Con operaciones (abiertas y cerradas)
ativos-posicao-todos = Todos los registrados
ativos-incluir-arquivados = Incluir archivados
ativos-n-ativos = { $n ->
    [one] { $n } activo
   *[other] { $n } activos
}
ativos-vazio-titulo = Ningún activo registrado
ativos-vazio-descricao = Registra acciones, fondos inmobiliarios, ETF, bonos del Tesoro de Brasil, CDB, activos en el exterior o cripto para registrar operaciones y darles seguimiento.
ativos-tabela = Activos registrados
ativos-nenhum-filtro = Ningún activo coincide con estos filtros.
ativos-acoes-de = Acciones de { $nome }

## Página del activo: cotización manual

ativos-cotacao-titulo = Ingresar cotización · { $nome }
ativos-cotacao-descricao = Para activos sin fuente automática (fondos, previsión privada, bonos privados). Queda marcada como “Ingresado por ti”.
ativos-cotacao-informada = Cotización ingresada
ativos-cotacao-informada-em = { $preco } el { $data }
ativos-cotacao-manual = Cotización manual
ativos-data = Fecha
ativos-informe-data = Ingresa la fecha.
ativos-fechamento-moeda = Cierre ({ $moeda })
ativos-informe-preco = Ingresa el precio.
ativos-salvar-cotacao = Guardar cotización

## Página del activo: términos de renta fija

ativos-termos-titulo = Términos de renta fija
ativos-termos-resumo = Resumen
ativos-termos-tipo = Tipo
ativos-termos-indexador = Indexador
ativos-termos-taxa = Tasa
ativos-termos-emissao = Emisión
ativos-termos-vencimento = Vencimiento
ativos-termos-carencia = Carencia
ativos-termos-liquidez = Liquidez
ativos-termos-ir = Impuesto a la renta
ativos-termos-isento = Exento
ativos-termos-regressiva = Tabla regresiva
ativos-termos-emissor = Emisor

## Página del activo: encabezado y pestañas

ativos-carregando-ativo = Cargando activo
ativos-nao-encontrado = Activo no encontrado
ativos-sem-mercado = sin mercado
ativos-informar-cotacao = Ingresar cotización
ativos-acompanhar = Seguir
ativos-evento = Evento
ativos-provento = Rendimiento
ativos-secoes-ativo = Secciones del activo
ativos-aba-resumo = Visión general
ativos-aba-previdencia = Previsión
ativos-aba-extrato = Extracto
ativos-aba-eventos = Eventos corporativos
ativos-aba-grafico = Gráfico
ativos-aba-saude = Salud
ativos-aba-cadastro = Registro

## Página del activo: resumen

ativos-posicao = Posición
ativos-desde = Desde
ativos-custo = Costo
ativos-quantidade = Cantidad
ativos-preco-medio = Precio promedio
ativos-valor = Valor
ativos-liquido-estimado = Neto estimado (impuestos)
ativos-bruto-curva = Bruto en la curva
ativos-pelo-custo = Al costo (sin cotización)
ativos-valor-mercado = Valor de mercado
ativos-peso = peso { $peso }
ativos-na-moeda-do-ativo = { $valor } en la moneda del activo
ativos-resultado = Resultado
ativos-nao-realizado = No realizado
ativos-realizado = Realizado
ativos-resultado-total = Resultado total
ativos-retorno-anual = Rentabilidad anual (XIRR)
ativos-real-por = Real por
ativos-nominal = Nominal
ativos-real = Real
ativos-posicao-encerrada = Posición cerrada
ativos-sem-posicao = Sin posición en este activo
ativos-encerrada-texto = Se vendieron todas las cuotas; el resultado realizado y los rendimientos siguen en el historial.
ativos-sem-posicao-texto = Registra una compra para seguir el precio promedio, el resultado y la rentabilidad real.
ativos-realizado-proventos = Realizado { $realizado } · Rendimientos { $proventos }
ativos-avisos-calculo = Avisos del cálculo
ativos-hoje = Hoy · { $simbolo }
ativos-hoje-fonte = Fuente · Yahoo Finance (no oficial), ~15 min de retraso · historial por los cierres diarios

## Página del activo: eventos y gráfico

ativos-eventos-ajuda = Desdoblamiento: factor mayor que 1 (1 acción pasa a 5 = factor 5). Agrupamiento: factor menor que 1 (10 pasan a 1 = 0,1). Bonificación: acciones nuevas por acción (10% = 0,1). Registra cada hecho una sola vez: si el extracto ya trae la entrada a costo cero, no registres el evento.
ativos-grafico-titulo = Precio, operaciones y precio promedio
ativos-grafico-periodo = Período del gráfico
ativos-periodo-6m = 6M
ativos-periodo-1a = 1A
ativos-periodo-2a = 2A
ativos-periodo-max = Máx.
ativos-grafico-fonte = Fuente: { $fonte } · cierre en la base de cada día (desdoblamientos aplicados en la fecha), sin ajuste por rendimientos
ativos-grafico-sem-cotacoes = Sin cotizaciones: la línea une los precios de tus operaciones
ativos-carregando-cotacoes = Cargando cotizaciones
ativos-sem-historico = Sin historial de cotizaciones para este activo
ativos-sem-historico-renda-fixa = La renta fija se valora por la curva (tasa sobre el indexador), sin cotización de mercado.
ativos-sem-historico-texto = El gráfico muestra el precio de cada operación. Actualiza las cotizaciones en Índices y cotizaciones o ingresa la cotización manualmente.
ativos-grafico-aria = { $n ->
    [one] Precio de { $nome } con { $n } operación marcada y el precio promedio en escalones.
   *[other] Precio de { $nome } con { $n } operaciones marcadas y el precio promedio en escalones.
}
ativos-fechamento = Cierre
ativos-preco-operacoes = Precio de las operaciones
ativos-compra = ▲ Compra
ativos-venda = ▼ Venta
ativos-tabela-operacoes = Operaciones y precio promedio
ativos-qtd = Cant.
ativos-preco = Precio
ativos-preco-medio-depois = Precio promedio después
ativos-analise-fundamentalista = Análisis fundamental

## Página del activo: registro

ativos-reativado = Activo reactivado
ativos-arquivado-toast = Activo archivado
ativos-desarquivar = Desarchivar
ativos-arquivar = Archivar
ativos-excluir = Eliminar activo
ativos-excluir-descricao = { $nome } sale del registro, junto con la watchlist y sus eventos. Los activos con operaciones o rendimientos solo se pueden archivar.
ativos-excluido = Activo eliminado
ativos-nao-excluiu = No se pudo eliminar
ativos-so-arquivar = Con operaciones, el activo solo se puede archivar.

## Registro: lo que pide cada tipo

ativos-fonte-manual = Manual (yo la ingreso)
ativos-fonte-nenhuma = Sin cotización
ativos-fonte-auto-yahoo = Automática (la fuente activada para este tipo)
ativos-fonte-auto-cripto = Automática (DefiLlama, en dólares)
ativos-fonte-yahoo = Yahoo Finance (fuente no oficial)
ativos-fonte-tiingo = Tiingo (con tu clave)
ativos-fonte-eodhd = EODHD (con tu clave)
ativos-fonte-brapi = brapi.dev (con token, una vez por día)
ativos-fonte-cotahist = B3 COTAHIST (oficial, cierre de la sesión anterior)
ativos-fonte-auto-cvm = Automática (cuota de la CVM por el CNPJ)
ativos-fonte-auto-tesouro = Automática (Tesouro Transparente)
ativos-fonte-auto-curva = Automática (por la curva de los términos)
ativos-fonte-auto-nenhuma = Automática (ninguna)
ativos-fonte-yahoo-simbolo = Yahoo Finance (por el símbolo)
ativos-fonte-cota-cvm = Cuota de la CVM
ativos-campo-setor = Sector
ativos-campo-setor-exemplo = Energía, Bancos…
ativos-campo-segmento = Segmento
ativos-opcional = Opcional
ativos-opcional-ponto = Opcional.
ativos-exemplo = Ej.: { $exemplo }.
ativos-segmento-fii-exemplo = Logística, Oficinas, Papel…
ativos-segmento-reit-exemplo = Comercio, Centros de datos…
ativos-tipo-stock-br = Acciones negociadas en la B3.
ativos-tipo-fii = Fondos inmobiliarios negociados en la B3.
ativos-tipo-etf-br = ETF negociados en la B3.
ativos-tipo-bdr = Certificados de acciones extranjeras negociados en la B3.
ativos-tipo-stock-us = Acciones en las bolsas de EE. UU. (NYSE, NASDAQ), en dólares.
ativos-tipo-etf-us = ETF en las bolsas de EE. UU., en dólares.
ativos-tipo-reit = Fondos inmobiliarios de EE. UU., en dólares.
ativos-tipo-stock-intl = Acciones de otras bolsas (Londres, Fráncfort, Tokio…): la bolsa define la moneda.
ativos-tipo-etf-intl = ETF de otras bolsas: la bolsa define la moneda.
ativos-tipo-crypto = Criptomonedas, cotizadas en Yahoo Finance por el par con la moneda (BTC-USD).
ativos-tipo-treasury-br = Bonos del Tesouro Direto: el precio viene del Tesouro Transparente por el título.
ativos-tipo-fixed-income = CDB, LCI, LCA, CRI, CRA, obligaciones y ahorro: valorados por la curva de los términos.
ativos-tipo-fund = Fondos de inversión: la cuota diaria viene de la CVM por el CNPJ.
ativos-tipo-pension = Previsión privada VGBL o PGBL: los aportes y rescates son las operaciones; la cuota viene de la CVM por el CNPJ del fondo del plan.
ativos-tipo-other = Cualquier otra inversión: ingresa la moneda y, si quieres, el símbolo de la cotización.
ativos-nome-cdb-exemplo = CDB Banco X 110% CDI 2028
ativos-cnpj-emissor = CNPJ del banco emisor
ativos-cnpj-emissor-dica = Para el score de salud: los indicadores del banco vienen del Banco Central de Brasil. Sin él, el emisor se busca por el nombre (Nubank, Itaú, Inter…).
ativos-nome-fundo = Nombre del fondo
ativos-cnpj-fundo = CNPJ del fondo
ativos-cnpj-fundo-dica = Para la cuota diaria de la CVM.
ativos-categoria = Categoría
ativos-categoria-fundo-exemplo = Multimercado, Acciones, Renta fija…
ativos-nome-plano = Nombre del plan
ativos-cnpj-fie = CNPJ del fondo (FIE)
ativos-cnpj-fie-dica = Del fondo del plan, en el extracto de la aseguradora: para la cuota diaria de la CVM. Sin él, ingresa la cuota en Cotización.
ativos-codigo = Código
ativos-grupo-bolsa-br = Bolsa en Brasil
ativos-grupo-renda-fixa = Renta fija
ativos-grupo-fundos = Fondos y previsión
ativos-grupo-exterior = Exterior
ativos-grupo-outros = Otros

## Registro y edición de activo

ativos-form-editar = Editar { $nome }
ativos-form-editar-descricao = El mercado, la moneda, el país y los identificadores no cambian después del registro.
ativos-form-novo-descricao = Elige el tipo de activo: el formulario muestra solo lo que necesita.
ativos-form-tipo = Tipo de activo
ativos-form-erro-tipo = Elige el tipo de activo.
ativos-form-erro-nome = Ingresa el nombre del activo.
ativos-form-erro-moeda = Usa el código ISO de 3 letras (BRL, USD).
ativos-form-erro-taxa = Ingresa la tasa.
ativos-form-erro-bolsa = Elige la bolsa (o regístrala en Activos › Bolsas).
ativos-form-atualizado = Activo actualizado
ativos-form-cadastrado = Activo registrado
ativos-form-bolsa = Bolsa
ativos-form-so-leitura = Solo lectura.
ativos-form-bolsa-obrigatoria = Obligatoria para otras bolsas.
ativos-form-padrao = Predeterminado: { $padrao }.
ativos-form-nenhuma = ninguna
ativos-form-escolha-bolsa = Elige la bolsa
ativos-form-padrao-opcao = Predeterminada ({ $padrao })
ativos-form-moeda-cotacao = Moneda de cotización
ativos-form-padrao-sem-ptax = Predeterminada: { $moeda }. Sin PTAX en el Banco Central de Brasil: el tipo de cambio a BRL es manual.
ativos-form-pais = País
ativos-form-pais-dica = Código de 2 letras (BR, US).
ativos-form-rotulo = Activo
ativos-form-ou-busque = O busca por ticker o nombre:
ativos-form-buscar = Buscar activo
ativos-form-preenchido = Completado con { $ativo }.
ativos-form-preenchido-cadastrado = Completado con { $ativo }. Este activo ya está registrado.
ativos-form-buscar-dica = Acciones, fondos inmobiliarios, ETF y BDR de la B3, EE. UU. y otras bolsas, cripto. Elige uno para completar el formulario.
ativos-form-titulo-tesouro = Título del Tesoro
ativos-form-titulo-tesouro-dica = Como en el Tesouro Transparente (ej.: Tesouro IPCA+ 2035).
ativos-form-informar-termos = Ingresar indexador y tasa
ativos-form-informar-termos-dica = Sin el precio unitario del Tesouro Transparente, el título se valora por la curva con estos términos.
ativos-form-carencia-ate = Carencia hasta
ativos-form-isento-ir = Exento de impuesto a la renta
ativos-form-isento-ir-dica = LCI, LCA, CRI, CRA, obligación incentivada y ahorro.
ativos-form-plano = Plan de previsión
ativos-form-plano-dica = Los aportes y rescates entran como operaciones del activo, en la Cartera.
ativos-form-cotacao-identificadores = Cotización e identificadores
ativos-form-fonte = Fuente de cotización
ativos-form-simbolo = Símbolo en la fuente
ativos-form-simbolo-automatico = Automático: { $simbolo }.
ativos-form-simbolo-dica = Opcional: PETR4.SA, VOD.L, BTC-USD…
ativos-form-observacoes = Observaciones
ativos-form-salvar = Guardar activo

## Rendimientos anunciados y Fundamentus (B3)

ativos-fonte = Fuente
ativos-pagamento = Pago
ativos-atualizar = Actualizar
ativos-anunciados-data-com = Fecha de corte
ativos-anunciados-valor-cota = Valor por cuota/acción
ativos-anunciados-fii = Rendimientos anunciados por el fondo
ativos-anunciados-titulo = Rendimientos anunciados
ativos-anunciados-novos = { $n ->
    [0] Ningún rendimiento nuevo
    [one] { $n } rendimiento nuevo anunciado
   *[other] { $n } rendimientos nuevos anunciados
}
ativos-anunciados-novos-dica = Los que te correspondían aparecen en "Rendimientos por confirmar" para registrar.
ativos-anunciados-buscar = Buscar en Fundamentus
ativos-anunciados-fonte = fundamentus.com.br (caché de un día)
ativos-anunciados-tabela = Rendimientos anunciados de { $ticker }
ativos-anunciados-nenhum = Ningún rendimiento anunciado en Fundamentus.
ativos-fundamentus-data-com = Fecha de corte
ativos-fundamentus-cache = De la caché, leído el
ativos-fundamentus-lido = Leído el
ativos-fundamentus-sem-papel = Fundamentus no tiene el papel { $ticker }.
ativos-fundamentus-fundo = Fondo
ativos-fundamentus-empresa = Empresa
ativos-fundamentus-gestao = Gestión
ativos-fundamentus-subsetor = Subsector
ativos-fundamentus-cotacao-site = Cotización en el sitio
ativos-fundamentus-cotacao-em = { $preco } el { $data }
ativos-fundamentus-oscilacao = Variación
ativos-fundamentus-tabela = Rendimientos de { $ticker } en Fundamentus

## Watchlist

ativos-acoes = Acciones
ativos-remover = Quitar
ativos-adicionar = Agregar
ativos-watch-titulo = Watchlist · { $nome }
ativos-watch-adicionar-titulo = Agregar a la watchlist
ativos-watch-descricao = Precio objetivo y alertas en la moneda del activo. Las alertas se disparan cuando la cotización obtenida cruza el límite.
ativos-watch-atualizada = Watchlist actualizada
ativos-watch-adicionado = Activo en la watchlist
ativos-watch-buscar-dica = Ticker o nombre; un activo nuevo se registra al elegirlo.
ativos-watch-escolha = Elige el activo.
ativos-watch-alvo-moeda = Precio objetivo ({ $moeda })
ativos-watch-alvo-dica = Mi precio objetivo.
ativos-watch-alertar-acima = Alertar por encima de
ativos-watch-alertar-abaixo = Alertar por debajo de
ativos-watch-notas = Notas
ativos-watch-sem-cotacao = sin cotización
ativos-watch-hoje = Hoy
ativos-watch-alvo = Precio objetivo
ativos-watch-ate-alvo = Hasta el objetivo
ativos-watch-alertas = Alertas
ativos-watch-acima-de = ▲ por encima de { $preco }
ativos-watch-abaixo-de = ▼ por debajo de { $preco }
ativos-watch-situacao = Situación
ativos-watch-remover-titulo = Quitar de la watchlist
ativos-watch-remover-descricao = { $nome } sale de la watchlist; el registro del activo sigue.
ativos-watch-removido = Quitado de la watchlist
ativos-watch-resumo = { $n ->
    [one] { $n } activo seguido
   *[other] { $n } activos seguidos
} · { $disparados } con alerta disparada
ativos-watch-vazia = Watchlist vacía
ativos-watch-vazia-descricao = Sigue activos que todavía no tienes, con precio objetivo y alertas de cotización.
ativos-watch-editar-alvo = Editar precio objetivo y alertas…

## Extracto del activo y come-cotas

ativos-come-cotas = Come-cotas
ativos-come-cotas-regra = Regla del come-cotas
ativos-come-cotas-auto = Automático
ativos-come-cotas-longo = Largo plazo (15%)
ativos-come-cotas-curto = Corto plazo (20%)
ativos-come-cotas-isento = Sin come-cotas
ativos-come-cotas-descontos = { $n ->
    [one] { $n } descuento, impuesto total { $total }
   *[other] { $n } descuentos, impuesto total { $total }
}
ativos-extrato-montando = Armando el extracto
ativos-extrato-confira = Revisa estas líneas
ativos-extrato-aviso = { $n ->
    [one] { $aviso } ({ $n } línea marcada).
   *[other] { $aviso } ({ $n } líneas marcadas).
}
ativos-extrato-ajuda = En el orden del cálculo: los eventos corporativos de una fecha valen antes que las operaciones del mismo día. La cantidad y el precio promedio son los de después de cada línea.
ativos-extrato-qtd-registrada = (operación de { $qtd })
ativos-extrato-tabela = Extracto del activo
ativos-extrato-vazio = Sin operaciones ni eventos.
ativos-extrato-lancamento = Movimiento
ativos-extrato-posicao-depois = Posición después
ativos-extrato-origem = Origen
ativos-extrato-linha = { $lancamento } el { $data }
ativos-extrato-duplicata = Parece repetida: otra línea tiene la misma fecha, cantidad y precio.
ativos-extrato-duplicatas-titulo = Movimientos que parecen repetidos
ativos-extrato-duplicatas = { $n ->
    [one] { $n } línea tiene la misma fecha, cantidad y precio que otra. Si es la misma operación registrada dos veces, elimina la que sobra desde el menú del botón derecho; nada se elimina sin que lo confirmes.
   *[other] { $n } líneas tienen la misma fecha, cantidad y precio que otra. Si es la misma operación registrada dos veces, elimina la que sobra desde el menú del botón derecho; nada se elimina sin que lo confirmes.
}

## Plan de previsión

ativos-plano-tipo = Tipo de plan
ativos-plano-tipo-dica = VGBL: impuesto solo sobre la ganancia, sin deducción. PGBL y fondo de pensiones (EFPC, previsión cerrada de empresa o cooperativa): aportes deducidos en la declaración (hasta el 12% de la renta gravable) e impuesto sobre el valor total en el rescate.
ativos-plano-efpc = Fondo de pensiones
ativos-plano-regime = Régimen de tributación
ativos-plano-regime-dica = Regresivo: 35% hasta 2 años de cada aporte, bajando hasta 10% después de 10 años (definitivo). Progresivo: 15% en la fuente y ajuste por la tabla anual.
ativos-plano-regressivo = Regresivo
ativos-plano-progressivo = Progresivo
ativos-plano-seguradora = Aseguradora
ativos-plano-inicio = Inicio del plan
ativos-plano-taxa-adm = Comisión de administración (% anual)
ativos-plano-taxa-carregamento = Comisión de carga (%)
ativos-plano-carregamento-dica = Sobre cada aporte, cuando la haya.

## Pestaña Previsión

ativos-prev-bruto = Bruto
ativos-prev-base-ir = Base imponible
ativos-prev-liquido = Neto
ativos-prev-aliquota = Tasa efectiva
ativos-prev-plano-salvo = Plan de previsión guardado
ativos-prev-salvar-plano = Guardar plan
ativos-prev-avisos = Avisos de la previsión
ativos-prev-saldo = Saldo y rescate total
ativos-prev-cota = Cuota
ativos-prev-aportado = Aportado
ativos-prev-valor-hoje = Valor hoy
ativos-prev-rendimento = Rendimiento
ativos-prev-resgatar-tudo = Si rescatas todo hoy
ativos-prev-mudam-faixa = { $valor } en aportes bajan de tramo en los próximos 90 días: esperar puede significar pagar menos impuesto.
ativos-prev-pagamentos-titulo = Historial de pagos
ativos-prev-pagamentos-tabela = Pagos de la previsión
ativos-prev-pagamentos-pendencias = Pendientes del historial de pagos
ativos-prev-pagamento-situacao = Situación
ativos-prev-pagamento-cotas = { $cotas } cuotas
ativos-prev-pagamento-sem-cota = A la espera de la cuota de la CVM
ativos-prev-pagamento-conta = Cuenta de las cuotas
ativos-prev-custodia = Cuenta de las cuotas del plan
ativos-prev-custodia-conta = { $n ->
        [one] Las cuotas de este plan están en la cuenta { $contas }.
       *[other] Las cuotas de este plan están en las cuentas { $contas }.
    }
ativos-prev-custodia-sem-conta = Este plan aún no tiene cuenta de cuotas. Importar el historial de pagos crea una junto con los aportes.
ativos-prev-aportes-pendentes =
    { $n ->
        [one] { $n } aporte, { $valor } en total, espera la cuota de la CVM desde { $data }: se vuelve cuotas solo cuando la recogida de cotizaciones traiga el historial del fondo.
       *[other] { $n } aportes, { $valor } en total, esperan la cuota de la CVM desde { $data }: se vuelven cuotas solos cuando la recogida de cotizaciones traiga el historial del fondo.
    }
ativos-prev-plano = Plan
ativos-prev-aportes-titulo = Aportes y tramo de impuesto
ativos-prev-aportes-tabela = Aportes de la previsión
ativos-prev-nenhum-aporte = Ningún aporte: registra los aportes como operaciones de compra del plan.
ativos-prev-aporte = Aporte
ativos-prev-ir-hoje = Impuesto regresivo hoy
ativos-prev-proxima-faixa = Próximo tramo
ativos-prev-proxima-em = { $pct } el { $data }
ativos-prev-ja-menor = ya en el menor (10%)
ativos-prev-simular-titulo = Simular rescate
ativos-prev-valor-bruto = Valor bruto a rescatar
ativos-prev-simular = Simular
ativos-prev-simular-dica = Salen primero los aportes más antiguos (menor tasa en el régimen regresivo).
ativos-prev-resultado = Resultado de la simulación

## Importar el extracto de la previsión (PDF)

ativos-prev-linha-participante = Tu aporte
ativos-prev-linha-empregador = Extra del empleador
ativos-prev-linha-invalidez = Seguro de invalidez
ativos-prev-linha-morte = Seguro de vida
ativos-prev-linha-nova = Nueva
ativos-prev-linha-duplicada = Ya importada
ativos-prev-linha-informativa = Solo informativa
ativos-prev-competencia = Período
ativos-prev-cotas = Cuotas
ativos-prev-importar = Importar extracto (PDF)
ativos-prev-importar-titulo = Importar extracto de la previsión
ativos-prev-importar-descricao = El PDF del extracto del participante (fondo de pensiones, como Quanta Previdência): revisas la vista previa antes de guardar.
ativos-prev-extrato-pdf = Extracto en PDF
ativos-prev-lendo = Leyendo el extracto…
ativos-prev-layout = Formato reconocido
ativos-prev-cnpj-plano = CNPJ del plan
ativos-prev-adesao = Adhesión
ativos-prev-taxas = Comisiones
ativos-prev-taxas-valor = administración { $adm }% anual · carga { $carregamento }%
ativos-prev-regime = Régimen
ativos-prev-suas-contribuicoes = Tus aportes
ativos-prev-extras-empregador = Extras del empleador
ativos-prev-seguros-risco = Seguros de riesgo
ativos-prev-saldo-extrato = Saldo en el extracto
ativos-prev-cotas-conferem = Las cuotas de los aportes suman el saldo de cuotas del extracto.
ativos-prev-linhas = Líneas del extracto
ativos-prev-conta-cotas = Cuenta de las cuotas
ativos-prev-conta-cotas-dica = Los aportes pasan a ser compra de cuotas sin tocar el efectivo de la cuenta.
ativos-prev-crie-corretora = Crea una cuenta de tipo corredora
ativos-prev-seguros-dica = Invalidez y vida son costo de seguro, no pasan a ser cuotas.
ativos-prev-nao-lancar = No registrar
ativos-prev-despesa-em = Gasto en { $conta }
ativos-prev-importado = Extracto importado
ativos-prev-importado-resumo = { $contribuicoes ->
    [one] { $contribuicoes } aporte
   *[other] { $contribuicoes } aportes
}{ $seguros ->
    [0] {""}
    [one] {" "}y { $seguros } seguro de riesgo
   *[other] {" "}y { $seguros } seguros de riesgo
}{ $puladas ->
    [0] {""}
   *[other] ; { $puladas } ya estaban en la app
}.
ativos-prev-importar-linhas = { $n ->
    [one] Importar { $n } línea
   *[other] Importar { $n } líneas
}
ativos-prev-nada-novo = Nada nuevo para importar

## Rendimientos por confirmar

ativos-por-cota = Por cuota
ativos-pendentes-data-pagamento = Fecha de pago de { $nome }
ativos-pendentes-palpite = ≈ estimación: fecha ex
ativos-pendentes-titulo = Rendimientos por confirmar
ativos-pendentes-titulo-n = Rendimientos por confirmar ({ $n })
ativos-pendentes-conta = Cuenta
ativos-pendentes-lancado = Rendimiento de { $nome } registrado
ativos-pendentes-pagamento-em = Pago el { $data } · { $conta }
ativos-pendentes-nao-lancou = No se pudo registrar
ativos-pendentes-ir = Impuesto { $valor }
ativos-pendentes-origem = Anunciado por la fuente de mercado; todavía no registrado en la cartera.
ativos-pendentes-confirmar = Confirmar
ativos-pendentes-confirmar-de = Confirmar rendimiento de { $nome }
ativos-pendentes-dispensar = Descartar
ativos-pendentes-dispensar-de = Descartar rendimiento de { $nome }
ativos-pendentes-ajuda = Rendimientos anunciados con fecha de corte en la que la familia tenía el activo y que todavía no están en la cartera. Confirmar registra el crédito en el efectivo de la corredora; descartar lo oculta hasta que vuelvas a abrir la app.

## Calendario de dividendos

ativos-agenda-sua-qtd = Tu cant.
ativos-agenda-previsto-liquido = Previsto neto
ativos-agenda-projecao = ≈ Proyección
ativos-agenda-ja-ocorreu = Ya ocurrió
ativos-agenda-anunciado = Anunciado
ativos-agenda-radar = Radar de dividendos (proyección por el historial)
ativos-agenda-jcp = Previsto ya con el impuesto retenido de los JCP (17,5% desde 2026; 15% hasta 2025).
ativos-agenda-so-data-com = La fuente informó solo la fecha de corte; el pago sale en el aviso de la empresa.
ativos-agenda-periodo = Período del calendario
ativos-agenda-ultimos-30 = Últimos 30 días
ativos-agenda-proximos = Próximos { $n }
ativos-agenda-so-minha = Solo mi cartera
ativos-agenda-mostrar-projecoes = Mostrar proyecciones del radar
ativos-agenda-previsto-moeda = Previsto ({ $moeda })
ativos-agenda-vazia = Ningún rendimiento en el calendario
ativos-agenda-vazia-descricao = El calendario viene de los rendimientos obtenidos con las cotizaciones (Yahoo Finance) y de las proyecciones del radar. Actualiza las cotizaciones en Índices y cotizaciones, o amplía el período.
ativos-agenda-n-eventos = { $n ->
    [one] { $n } evento
   *[other] { $n } eventos
}
ativos-agenda-de = Calendario de { $mes }
ativos-agenda-ajuda = Previsto = tu cantidad en la fecha de corte (o hoy, si todavía no llegó) × valor por cuota; JCP ya con el impuesto retenido (17,5% desde 2026; 15% hasta 2025). Las proyecciones del radar usan el historial de pagos y cambian cuando se anuncia el rendimiento.

# Página dedicada do ativo (docs/asset-page.md).
ativo-trilha = Ruta
ativo-perguntar-agente = Preguntar al agente
ativo-agente-pergunta = Analiza { $ativo } con los datos que Treasury tiene de él (posición, fundamentos, precio calculado, salud, documentos y estudios) y dime qué llama la atención. Si encuentras datos nuevos y verificados, propón completar el registro.
ativo-aba-conversas = Conversaciones
ativo-conversas-texto = Las conversaciones con el agente iniciadas en esta página quedan aquí. Los análisis completos, con fuentes verificadas, están en Estudio con IA.
ativo-ver-estudos = Ver estudios
ativo-conversas-vazio = Todavía no hay conversaciones sobre este activo
ativo-conversas-vazio-texto = Usa "Preguntar al agente" para conversar sobre { $ativo }; la conversación queda guardada aquí.
ativo-conversa-detalhe =
    { $n ->
        [one] { $quando } · { $n } mensaje
       *[other] { $quando } · { $n } mensajes
    }
ativo-abrir-conversa = Abrir

# Visão geral e notícias do ativo (docs/asset-overview.md).
ativo-visao-geral = Visión general
ativo-empresa = Empresa
ativo-setor = Sector
ativo-cotacao = Cotización
ativo-cotacao-em = el { $data } ({ $fonte })
ativo-score = Score de salud
ativo-visao-geral-sem-dados = Todavía no hay datos de mercado de este activo (la cotización y los indicadores vienen de la recolección del análisis de activos).
ativo-indicadores-chave = Indicadores principales
ativo-indicadores-sem = Sin indicadores para este activo.
ativo-ver-fundamentos = Ver todos los fundamentos
ativo-ultimas = Últimas noticias y documentos
ativo-ultimas-vazio = Nada reciente por aquí.
ativo-ver-noticias = Ver todas las noticias
ativo-aba-noticias = Noticias
ativo-noticias-texto = Documentos oficiales de la CVM y titulares de portales de economía, del más nuevo al más antiguo. Los titulares de los portales se filtran en la computadora: nada de tu cartera sale de ella. Solo título, medio, fecha y enlace.
ativo-noticias-google-dica = ¿Quieres más noticias? La búsqueda de Google Noticias se abre en tu navegador: Treasury no consulta Google.
ativo-noticias-abrir-google = Abrir en Google Noticias
ativo-noticias-vazio = Ninguna noticia reciente
ativo-noticias-vazio-texto = Los portales traen solo los últimos titulares de cada día; los documentos de la CVM aparecen después de la recolección del análisis de activos.
ativo-noticias-falhou = No se pudo leer ahora: { $fontes }.
ativo-noticia-origem-cvm = CVM
ativo-noticia-origem-portal = portal
ativo-resumo-do-ativo = Resumen del activo
ativos-conta-custodia = Dónde está (cuenta)
ativos-conta-custodia-dica = La corredora, el banco del CDB, LCI/LCA o fondo, o la cuenta en el exterior. La compra, la venta y el rescate usan esta cuenta.
ativos-conta-custodia-nenhuma = Sin cuenta definida
ativos-prev-informar-saldo = Informar saldo
ativos-prev-informar-saldo-dica = Para planes sin cuota pública (previsión cerrada, cooperativa): el saldo que muestra la entidad pasa a ser el valor del plan.
ativos-prev-saldo-data = Fecha del saldo
ativos-prev-saldo-valor = Saldo
ativos-prev-salvar-saldo = Guardar saldo
ativos-prev-saldo-salvo = Saldo guardado

# Renta fija sin cantidad (CDB, RDB, LCI…): solo valores.
ativos-aplicado = Valor invertido
ativos-extrato-custo-depois = Costo después

## Resumen de la pestaña Operaciones

ativos-resumo-ops-aria = Resumen de las operaciones
ativos-resumo-ops-compras = Compras
ativos-resumo-ops-vendas = Ventas
ativos-resumo-ops-custos = Costes
ativos-resumo-ops-posicao = Posición actual
ativos-resumo-ops-total-comprado = Total comprado
ativos-resumo-ops-total-vendido = Total vendido
ativos-resumo-ops-total-custos = Comisiones e impuestos
ativos-resumo-ops-operacoes = Operaciones
ativos-resumo-ops-preco-medio-compra = Precio promedio de compra
ativos-resumo-ops-preco-medio-venda = Precio promedio de venta
ativos-resumo-ops-resultado = Resultado realizado
ativos-resumo-ops-taxas = Corretaje y comisiones
ativos-resumo-ops-ir = Impuesto retenido
ativos-resumo-ops-do-volume = Del volumen negociado
ativos-resumo-ops-custo-posicao = Coste de la posición
ativos-resumo-ops-situacao = Situación
ativos-resumo-ops-zerada = Posición cerrada
ativos-resumo-ops-periodo = Operaciones del { $de } al { $ate }
ativos-resumo-ops-periodo-um-dia = Operaciones el { $data }
ativos-resumo-ops-ajustado = Cantidades y precios promedio ajustados por desdoblamientos y agrupamientos
ativos-resumo-ops-eventos = { $n ->
        [one] La posición actual incluye un evento (bonificación, come-cotas o cambio de ticker), por eso difiere de comprado − vendido.
       *[other] La posición actual incluye { $n } eventos (bonificaciones, come-cotas o cambios de ticker), por eso difiere de comprado − vendido.
    }
