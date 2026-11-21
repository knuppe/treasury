# Pantalla 11 · Impuestos: estimación de los impuestos pagados en el año, escenarios, consumo por
# categoría, impuestos registrados, ingresos laborales, constancias de ingresos, patrimonio y
# fuentes (docs/taxes.md, docs/i18n-design.md). Revisión: borrador por IA, sin revisión nativa.

## Pantalla

impostos-periodo = { $de } a { $ate }
impostos-descricao = Cuánto del año se fue en impuestos, estimado a partir de tus datos: consumo, ingresos laborales, patrimonio e impuestos registrados. Cada número muestra de dónde salió.
impostos-estimando = Estimando los impuestos del año
impostos-aviso-titulo = Estimación, no valor oficial
impostos-desatualizados = Datos desactualizados
impostos-desatualizados-texto = Las cifras de { $dados } se revisaron por última vez el { $data } (versión { $versao }), hace más de 6 meses. Las tasas y tablas pueden haber cambiado.
impostos-esboco-titulo = { $dados }: borrador
impostos-esboco-texto = Los datos de este país todavía son parciales: solo se estima una parte de los impuestos.
impostos-aviso-padrao = Esta es una estimación basada en promedios de carga tributaria por categoría de consumo y en las reglas vigentes del impuesto sobre la renta. Los valores reales pueden variar según el estado, el régimen tributario de las empresas y beneficios fiscales específicos.

## Escenarios

impostos-cenario = Escenario
impostos-cenario-estimativa = Escenario de la estimación
impostos-cenario-conservador = Conservador
impostos-cenario-conservador-dica = El menor valor razonable de cada grupo: el producto con menor carga o la tasa reducida.
impostos-cenario-provavel = Probable
impostos-cenario-provavel-dica = El valor central de cada grupo: la mediana de los productos o el dato oficial más reciente.
impostos-cenario-elevado = Alto
impostos-cenario-elevado-dica = El mayor valor razonable de cada grupo: el producto con mayor carga o la tasa general.
impostos-tres-cenarios = Tres escenarios
impostos-cenarios = Escenarios
impostos-cenarios-nota = Los escenarios solo cambian el impuesto al consumo; ingresos, INSS (seguridad social), impuestos registrados e IPVA (impuesto vehicular) son iguales en los tres.

## Partes de la estimación y grupos de impuestos

impostos-parte-consumo = Consumo
impostos-parte-ir = Impuesto sobre la renta
impostos-parte-previdencia = Seguridad social (INSS)
impostos-parte-patrimonio = Patrimonio
impostos-parte-proventos = Rendimientos de inversiones
impostos-parte-outros = Otros tributos
impostos-grupo-ir-pago = Impuesto sobre la renta pagado
impostos-grupo-iof-pago = IOF pagado (impuesto a operaciones financieras)
impostos-grupo-iptu = IPTU (impuesto predial)
impostos-grupo-ipva = IPVA (impuesto vehicular) y licencia
impostos-grupo-outros-pagos = Otros tributos pagados
impostos-grupo-financeiro = Financiero (deudas, intereses, inversiones)
impostos-grupo-transferencias = Transferencias a personas (pensión, mesada, donaciones)
impostos-sem-grupo = Sin grupo
impostos-origem-usuario = tu elección
impostos-origem-padrao = predeterminado
impostos-origem-pai = del grupo padre
impostos-origem-cupons = comprobantes fiscales

## Fuera de la estimación: motivos

impostos-motivo-sem-categoria = Sin categoría
impostos-motivo-sem-categoria-dica = Los movimientos sin categoría no tienen grupo de impuesto. Categorízalos en Movimientos para que entren.
impostos-motivo-sem-grupo = Sin grupo de impuesto
impostos-motivo-sem-grupo-dica = Categorías creadas por ti que no están en el conjunto inicial y no heredan grupo del padre. Elige el grupo de cada una.
impostos-motivo-fora-consumo = Fuera del consumo
impostos-motivo-fora-consumo-dica = Deudas, intereses, inversiones y transferencias a personas no son compras de productos ni servicios: no tienen impuesto incluido que estimar.
impostos-motivo-sem-dado = Sin dato confiable
impostos-motivo-sem-dado-dica = No hay una fuente confiable de la carga incluida en estos gastos (alquiler, seguro médico, servicios sin detalle…). Quedan a la vista, pero fuera de la estimación; puedes fijar una tasa propia.

## Tasas

impostos-pontos = { $valor } p. p.
impostos-sem-dado = sin dato
impostos-base-isento = exento
impostos-base-iva-padrao = IVA general del país
impostos-base-iva-reduzido = IVA reducido del país
impostos-base-iva-super-reduzido = IVA superreducido del país
impostos-base-vendas = impuesto a las ventas del estado
impostos-regiao-uf = Estado (UF)
impostos-regiao-pais-membro = Estado miembro
impostos-regiao-estado = Estado

## Gráfico y composición

impostos-grafico-inclui-estimados = Incluye valores estimados
impostos-grafico-lancados = Valores registrados o informados
impostos-imposto-estimado = Impuesto estimado
impostos-composicao = Composición por tipo
impostos-composicao-vazia = Ningún impuesto estimado en el período.
impostos-por-tipo = Impuesto por tipo · { $cenario }
impostos-por-tipo-aria = Impuesto estimado por tipo, escenario { $cenario }
impostos-composicao-fonte = Fuente: estimación de la app con los datos del país · ≈ incluye valores estimados
impostos-tipo = Tipo
impostos-valor = Valor
impostos-total = Total

## Resumen

impostos-estimativa-de = Estimación de { $ano }
impostos-periodo-rotulo = Período
impostos-pais = País
impostos-nao-informado = no indicado
impostos-dados = Datos
impostos-total-estimado = Total estimado
impostos-total-estimado-cenario = Total estimado · { $cenario }
impostos-origem-estimativa = Estimación con los datos de { $dados } (versión { $versao })
impostos-origem-estimativa-nota = Suma del impuesto al consumo del escenario, de los ingresos laborales, del patrimonio, de los impuestos registrados y del IR retenido en rendimientos de inversiones.
impostos-faixa = rango de { $de } (conservador) a { $ate } (alto)
impostos-pct-renda = % de los ingresos
impostos-pct-renda-dica = impuesto total ÷ ingreso bruto
impostos-pct-consumo = % del consumo
impostos-pct-consumo-dica = sobre { $valor } de consumo
impostos-sem-consumo = sin consumo estimado
impostos-dias-trabalho = Días de trabajo para los impuestos
impostos-n-dias = { $n ->
    [one] { $n } día
   *[other] { $n } días
}
impostos-dias-ate = hasta el { $data }
impostos-dias-ate-media = hasta el { $data } · referencia: la carga tributaria del país equivale a { $dias } días (Tesouro Nacional, { $ano })
impostos-renda-bruta = Ingreso bruto estimado
impostos-renda-bruta-origem = Ingresos registrados en el período más lo retenido en la fuente (salario y rendimientos de inversiones)
impostos-renda-bruta-dica = ingresos + retenido en la fuente
impostos-referencia-pais = Referencia del país
impostos-em-ano = en { $ano }
impostos-dados-pais = Datos del país
impostos-referencia-texto = Recaudación de todos los tributos del país dividida por el PIB. Medida macroeconómica mostrada como contexto: no se puede comparar directamente con la estimación de la familia.
impostos-referencia-dias = { $rotulo }: { $dias } días en { $ano } (hasta el { $ate }), con una carga del { $carga } del PIB.
impostos-sem-referencia = Los datos de { $dados } no incluyen la carga tributaria del país.
impostos-melhorar = Para mejorar la estimación

## Controles

impostos-ano = Año
impostos-erro-trocar-pais = No se pudo cambiar el país
impostos-erro-trocar-regiao = No se pudo cambiar { $regiao }
impostos-erro-trocar-dependentes = No se pudieron cambiar los dependientes
impostos-pais-dados = País de los datos
impostos-esboco = borrador
impostos-sem-dados-estado = Sin datos por estado
impostos-nao-informado-opcao = No indicado
impostos-dependentes = Dependientes
impostos-dependentes-dica = Dependientes para el IR estimado a partir del salario neto

## Fuentes y metodología

impostos-fontes = Fuentes
impostos-fonte-publicada = Publicación: { $publicacao } · Consultada el { $consulta }
impostos-fonte-consultada = Consultada el { $consulta }
impostos-fontes-metodologia = Fuentes y metodología
impostos-fontes-descricao = Cómo se hace la estimación de { $dados } y de dónde sale cada número.
impostos-versao = Versión
impostos-atualizado-em = Actualizado el
impostos-cobertura = Cobertura
impostos-fontes-desatualizados = La última revisión de estas cifras fue el { $data }, hace más de 6 meses. Las tasas y tablas pueden haber cambiado desde entonces.
impostos-esboco-maiusc = Borrador
impostos-fontes-esboco = Los datos de { $dados } todavía son parciales: solo se estima una parte de los impuestos.
impostos-secao-aviso = Aviso
impostos-metodologia = Metodología
impostos-ajuste-estado = Ajuste del estado
impostos-ajuste-estado-texto = Las tablas de consumo usan un ICMS (impuesto estatal a la circulación de mercancías) de referencia de { $icms }. En las mercancías con la tasa general, se suma la diferencia entre la tasa general del estado elegido y esa referencia.
impostos-renda-trabalho = Ingresos laborales
impostos-a-partir-de = desde el { $data }
impostos-contexto = Contexto
impostos-contexto-texto = { $rotulo }: { $valor } en { $ano }.
impostos-fontes-n = Fuentes ({ $n })
impostos-ver-fontes = Ver fuentes y metodología

## Fuera de la estimación

impostos-fora = Fuera de la estimación
impostos-gasto-fora = Gasto fuera
impostos-gasto-estimado = Gasto estimado
impostos-fora-vazio = Todo el gasto del período entró en la estimación.
impostos-escolher-grupo = Elegir grupo
impostos-escolher-grupo-de = Elegir grupo de { $categoria }
impostos-aliquota-propria = Tasa propia
impostos-aliquota-propria-de = Tasa propia de { $categoria }
impostos-categorizar = Categorizar en Movimientos

## Grupos de las categorías y ajustes de una categoría

impostos-padrao-app = Predeterminado de la app
impostos-padrao-app-grupo = Predeterminado de la app: { $grupo }
impostos-padrao-app-longo = Predeterminado de la app (por el nombre o por el grupo padre)
impostos-grupos-categorias = Grupos de las categorías
impostos-grupos-descricao = Cada categoría de gasto cae en un grupo de impuesto. El predeterminado sale del nombre de la categoría (o del grupo padre); cambia el grupo o fija una tasa propia cuando sepas más sobre tu gasto. Vale para todos los años.
impostos-filtrar-categorias = Filtrar categorías
impostos-filtrar-placeholder = Filtrar por categoría o grupo
impostos-carregando-categorias = Cargando las categorías
impostos-categoria = Categoría
impostos-grupo = Grupo
impostos-acoes = Acciones
impostos-filtro-vazio = Ninguna categoría con ese filtro.
impostos-erro-salvar = No se pudo guardar
impostos-aliquota-invalida = Tasa no válida
impostos-aliquota-faixa = Usa un número entre 0 y 100 (menor que 100).
impostos-sem-grupo-fora = sin grupo: fuera de la estimación
impostos-grupo-de = Grupo de { $categoria }
impostos-do-pais = del país
impostos-voltar-padrao = Volver al predeterminado
impostos-voltar-padrao-de = Volver { $categoria } al predeterminado
impostos-imposto-pago = impuesto pagado
impostos-fora-consumo = fuera del consumo
impostos-categoria-padrao = Categoría en el predeterminado
impostos-grupo-trocado = Grupo cambiado
impostos-grupo-padrao = Grupo en el predeterminado
impostos-aliquota-pais = Tasa del país
impostos-aliquota-salva = Tasa propia guardada
impostos-trocar-grupo-titulo = Cambiar grupo de impuesto
impostos-trocar-grupo-descricao = { $categoria }. El grupo indica qué impuesto recae sobre el gasto de la categoría; el nuevo grupo vale para todos los años.
impostos-hoje = Ahora: { $grupo } ({ $origem })
impostos-aliquota-descricao = { $categoria }. La fracción del precio que consideras impuesto, en lugar de la tabla del país. Vale en los tres escenarios.
impostos-usar-pais = Usar la del país
impostos-aliquota-pct = Tasa (% del precio)
impostos-aliquota-exemplo = ej.: 18,5
impostos-base-de = Base · { $grupo }
impostos-base-grupo = Base del grupo
impostos-produtos-servicos = Productos y servicios
impostos-icms-modal = Mercancía con la tasa general del ICMS: recibe el ajuste del estado elegido (diferencia con la referencia de { $icms }).
impostos-fontes-grupo = Fuentes del grupo
impostos-valor-como-esta = { $grupo }: el valor registrado en la categoría entra tal cual.
impostos-categoria-sem-grupo = La categoría no tiene grupo de impuesto.

## Consumo por categoría

impostos-menu-trocar-grupo = Cambiar grupo…
impostos-menu-aliquota = Tasa propia…
impostos-menu-ver-base = Ver base y fuentes
impostos-ja-padrao = La categoría ya usa el grupo y la tasa predeterminados.
impostos-aliquota-propria-nota = Tasa propia: vale en los tres escenarios.
impostos-ajuste-icms = Incluye el ajuste del ICMS del estado: { $pontos }.
impostos-aliquota-de = Tasa de { $categoria }: { $valores }
impostos-propria = propia
impostos-consumo-categoria = Consumo por categoría
impostos-consumo-nota = Gasto del año por devengo × fracción del precio que es tributo en el grupo de la categoría. Pasa el mouse por la tasa para ver los tres escenarios; usa el menú de la fila (clic derecho) para cambiar el grupo o fijar una tasa propia.
impostos-consumo-vazio = Ningún gasto de consumo con grupo de impuesto en el período.
impostos-acoes-de = Acciones de { $categoria }
impostos-gasto = Gasto
impostos-aliquota = Tasa
impostos-aliquota-cenario = Tasa en el escenario { $cenario }
impostos-do-consumo = Del consumo
impostos-ver-base-de = Ver base de { $categoria }
impostos-trocar-grupo-de = Cambiar grupo de { $categoria }

## Impuestos registrados, ingresos laborales, patrimonio y rendimientos

impostos-lancados = Impuestos registrados
impostos-lancados-nota = Lo registrado en las categorías de impuesto entra por el valor registrado, igual en los tres escenarios.
impostos-lancados-vazio = Ningún impuesto registrado como gasto en el período.
impostos-origem-holerites = recibos de nómina
impostos-origem-informado = informado
impostos-origem-estimado = ≈ estimado
impostos-renda-trabalho-nota = Sin constancia de ingresos, el bruto de cada mes se estima a partir del salario neto recibido en las categorías Salario, Vacaciones y 13º (aguinaldo), con las tablas de IR e INSS vigentes en el mes, sin otros descuentos (seguro médico, pensión). Las cargas del empleador no entran.
impostos-renda-trabalho-nota-exterior = Sin las nóminas, el bruto de cada mes se estima a partir del salario neto recibido en la categoría Salario (y en las pagas extra del país), con las reglas de la nómina vigentes en el año, sin otros descuentos. Las cargas del empleador no entran.
impostos-informe-inss = INSS
impostos-renda-trabalho-vazio = Ningún salario recibido ni constancia de ingresos en el período.
impostos-membro = Miembro
impostos-sem-membro = Sin miembro
impostos-origem = Origen
impostos-meses = Meses
impostos-liquido-recebido = Neto recibido
impostos-bruto = Bruto
impostos-ir-retido = IR retenido
impostos-regras-usadas = Reglas usadas en la estimación
impostos-patrimonio-nota = El IPTU y el IPVA registrados entran por su valor. Sin IPTU registrado, vale el recibo registrado en el inmueble (Cartera › Bienes). Sin IPVA registrado en el año, se estima con el valor de cada vehículo al final del período y la tasa del estado.
impostos-patrimonio-lancados = El IPTU y el IPVA salen de los movimientos (ver Impuestos registrados).
impostos-patrimonio-vazio = Ningún impuesto al patrimonio registrado o estimado en el período.
impostos-imposto-estimado-de = { $imposto } estimado · { $bem }
impostos-valor-vezes = Valor { $valor } × { $aliquota }
impostos-valor-venal-vezes = Valor catastral { $valor } × { $aliquota }
impostos-ir-proventos = IR retenido en rendimientos de inversiones
impostos-retido-fonte = Retenido en la fuente
impostos-retido-fonte-origem = Rendimientos registrados en Cartera (JCP, dividendos del exterior)
impostos-retido-texto = Impuesto retenido en los rendimientos pagados en el período, en la moneda de la casa.
impostos-ver-proventos = Ver rendimientos

## Constancias de ingresos

impostos-informe-alterado = Constancia modificada
impostos-informe-lancado = Constancia registrada
impostos-informe-editar-titulo = Editar constancia de ingresos · { $ano }
impostos-informe-lancar-titulo = Registrar constancia de ingresos · { $ano }
impostos-informe-descricao = Los valores del comprobante de ingresos (comprovante de rendimentos) que la fuente pagadora entrega a comienzos del año siguiente (recuadros 3 y 5).
impostos-fonte-pagadora = Fuente pagadora
impostos-sem-membro-casa = Sin miembro (de la casa)
impostos-rendimentos-tributaveis = Ingresos gravables
impostos-sem-13 = sin el 13º
impostos-previdencia-oficial = Seguridad social oficial (INSS)
impostos-ir-retido-fonte = IR retenido en la fuente
impostos-sem-ir-13 = sin el del 13º
impostos-13-salario = 13º salario
impostos-ir-13 = IR retenido sobre el 13º
impostos-observacoes = Observaciones
impostos-da-casa = De la casa
impostos-informes-de = Constancias de ingresos de { $ano }
impostos-informes-titulo = Constancias de ingresos · { $ano }
impostos-informes-texto = Con la constancia, los ingresos laborales del miembro usan sus valores en lugar de la estimación.
impostos-lancar-informe = Registrar constancia
impostos-editar-menu = Editar…
impostos-excluir = Eliminar
impostos-acoes-informe = Acciones de la constancia de { $fonte }
impostos-13-bruto = 13º bruto
impostos-ir-do-13 = IR del 13º
impostos-editar-informe = Editar constancia de { $fonte }
impostos-excluir-informe = Eliminar constancia
impostos-excluir-informe-texto = La constancia de { $fonte } ({ $ano }) sale de la estimación; los ingresos vuelven a estimarse por el neto. Puedes deshacerlo en el historial.
impostos-informes-vazio = Ninguna constancia de { $ano } registrada.

## Só no Brasil: para onde vai o imposto e a arrecadação da cidade
impostos-destino-titulo = A dónde va el impuesto
impostos-destino-texto = El impuesto estimado, repartido como el gasto del gobierno general (Unión, estados y municipios) en { $ano }, por función.
impostos-destino-ressalva = Es un reparto proporcional, no el destino de cada real: el gobierno también se financia con deuda y otros ingresos, y la línea de servicios públicos generales incluye los intereses de la deuda.
impostos-estado = Estado
impostos-cidade = Ciudad
impostos-cidade-titulo = Recaudación de tu ciudad
impostos-cidade-titulo-de = Recaudación de { $cidade }
impostos-cidade-escolha-texto = Elige la ciudad donde vive la familia para ver cuánto recaudó el municipio en tributos. La lista de ciudades viene del IBGE.
impostos-cidade-escolha-estado = Elige el estado
impostos-cidade-escolha = Elige la ciudad
impostos-cidade-carregando = Cargando…
impostos-cidade-salvar = Guardar
impostos-cidade-cancelar = Cancelar
impostos-cidade-trocar = Cambiar ciudad
impostos-cidade-erro-salvar = No se pudo guardar la ciudad
impostos-cidade-erro-buscar = No se pudo buscar la recaudación
impostos-cidade-por-habitante = Por habitante, en 12 meses
impostos-cidade-habitantes = { $n } habitantes
impostos-cidade-total = Tributos municipales en 12 meses
impostos-cidade-periodo = hasta { $mes }
impostos-cidade-fonte = Tesouro Nacional (Siconfi)
impostos-cidade-por-tributo = Por tributo
impostos-cidade-por-hab = por hab.
impostos-cidade-ressalva = Es lo que recaudó la alcaldía (IPTU, ISS, ITBI y tasas), no lo que paga cada vecino: las empresas pagan buena parte, y los impuestos federales y estatales quedan fuera.
impostos-cidade-conferido = Consultado el { $data }.
impostos-cidade-sem-dados = Todavía no se hizo ninguna consulta para esta ciudad.
impostos-cidade-fonte-desligada = La fuente Tesouro Nacional (Siconfi) está apagada en Ajustes › Fuentes de datos.
impostos-cidade-buscar = Buscar en el Tesouro Nacional
impostos-cidade-atualizar = Actualizar
impostos-cidade-buscando = Buscando…
