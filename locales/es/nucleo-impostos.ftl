# Impuestos (núcleo, `tax` y `tax_burden`): el cálculo del impuesto sobre la renta sobre las ganancias
# en bolsa y los DARF (09 · Impuestos › IR, solo de Brasil) y la estimación de impuestos pagados
# (09 · Impuestos): errores, motivos del historial, partes, grupos, notas y los textos de los
# datos de cada país (`tax_burden/data/*.json`). Errores en minúsculas, como línea de log (la
# interfaz pone la mayúscula inicial). Los términos oficiales (DARF, IRRF, IR, INSS, ICMS, IPVA,
# IPTU, IOF, NFC-e, FICA, IRS) quedan como están.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.
#
# Quedan en pt-BR en cualquier idioma: la descripción, el grupo y el país de Bens e Direitos (van
# al programa de la Receita Federal), las fuentes citadas, los estados y las bases legales.

## IR: categorías, notas y aviso del cálculo (`tax`)

nucleo-impostos-ir-categoria-acoes = Acciones (operaciones comunes)
nucleo-impostos-ir-categoria-etf = ETF (operaciones comunes)
nucleo-impostos-ir-categoria-bdr = BDR (operaciones comunes)
nucleo-impostos-ir-categoria-fii = FII
nucleo-impostos-ir-categoria-day-trade = Day trade
nucleo-impostos-ir-nota-isencao = Ventas de acciones en el mes de { $valor } (hasta R$ 20.000): ganancia exenta; la pérdida sigue siendo compensable.
nucleo-impostos-ir-nota-irrf-estimado = IRRF estimado (0,005% de las ventas; 1% de la ganancia en day trade): revisa las notas de corretaje.
nucleo-impostos-ir-nota-venda-maior = Venta de { $ativo } el { $data } mayor que la posición registrada ({ $quantidade } de más): el valor vendido cuenta en el total del mes, pero el resultado de esa parte queda fuera del cálculo hasta registrar la compra.
nucleo-impostos-ir-aviso = Cálculo auxiliar hecho con tus datos; revísalo en el programa de la Receita Federal (autoridad tributaria de Brasil). No es asesoramiento tributario.
nucleo-impostos-ir-erro-ano = año inválido

## IR: DARF

nucleo-impostos-ir-erro-gerar-darf = generar un DARF requiere la aprobación del usuario
nucleo-impostos-ir-erro-abaixo-minimo = impuesto por debajo de R$ 10,00: se acumula para el mes siguiente
nucleo-impostos-ir-erro-sem-imposto = no hay impuesto a pagar este mes
nucleo-impostos-ir-erro-darf-existe = ya existe un DARF para este mes; cancela el anterior para generar otro
nucleo-impostos-ir-erro-pagar-interface = registrar el pago de un DARF solo desde la interfaz
nucleo-impostos-ir-erro-darf-pago = un DARF pagado no se puede cancelar
# Motivos del historial; el mes queda como AAAA-MM.
nucleo-impostos-ir-motivo-darf = DARF { $codigo } de { $mes }
nucleo-impostos-ir-motivo-pagamento = Pago del DARF de { $mes }
nucleo-impostos-ir-motivo-cancelado = DARF de { $mes } cancelado
nucleo-impostos-ir-motivo-custodia-cripto = Dónde está la cripto de la cuenta (impuesto)
# Descripción del movimiento del pago.
nucleo-impostos-ir-lancamento-darf = DARF { $codigo } — IR renta variable { $mes }

## Impuestos pagados: ajustes, categorías y constancias de ingresos (`tax_burden`)

nucleo-impostos-erro-ajustes-interface = país y estado de la estimación solo desde la interfaz de la app
nucleo-impostos-erro-dependentes = demasiados dependientes
nucleo-impostos-erro-brasil = los datos de impuestos de Brasil solo valen para familias en Brasil
nucleo-impostos-cidade-iptu = IPTU (impuesto predial)
nucleo-impostos-cidade-iss = ISS (impuesto sobre servicios)
nucleo-impostos-cidade-itbi = ITBI (impuesto de transmisión de inmuebles)
nucleo-impostos-cidade-irrf = Impuesto sobre la renta retenido a los empleados municipales
nucleo-impostos-cidade-outros = Tasas y contribuciones de mejora
nucleo-impostos-cidade-erro-codigo = ciudad inválida: elige el municipio de la lista
nucleo-impostos-cidade-erro-uf = estado (UF) inválido
nucleo-impostos-cidade-erro-consulta = la consulta falló: { $erro }
nucleo-impostos-cidade-erro-resposta = la fuente respondió en un formato que la app no reconoce
nucleo-impostos-cidade-erro-sem-cidade = elige primero la ciudad de la familia
nucleo-impostos-cidade-erro-sem-relatorio = { $cidade } no tiene informe de ingresos en el Tesouro Nacional en los últimos seis bimestres
nucleo-impostos-erro-estado = el estado { $estado } no tiene datos en { $dados }
nucleo-impostos-dados-de = datos de impuestos de { $pais }
nucleo-impostos-categoria-despesa = categoría de gasto { $id }
nucleo-impostos-erro-grupo = grupo de impuesto desconocido: { $grupo }
nucleo-impostos-erro-aliquota = la alícuota propia debe estar entre 0% y 100% del precio
nucleo-impostos-erro-fonte-pagadora = indica quién paga
nucleo-impostos-erro-ano-informe = año de la constancia inválido
nucleo-impostos-erro-informe-negativo = los montos de la constancia no pueden ser negativos
nucleo-impostos-informe = constancia { $id }
nucleo-impostos-motivo-grupo = Grupo de impuesto de la categoría modificado
nucleo-impostos-motivo-informe-alterado = Constancia de ingresos modificada: { $fonte }
nucleo-impostos-motivo-informe-lancado = Constancia de ingresos registrada: { $fonte }
nucleo-impostos-motivo-informe-removido = Constancia de ingresos eliminada

## Impuestos pagados: la estimación

nucleo-impostos-erro-ano-futuro = el año todavía no empezó
nucleo-impostos-parte-consumo = Consumo (incluido en los precios)
nucleo-impostos-parte-renda = Impuesto sobre la renta
nucleo-impostos-parte-previdencia = Seguridad social (INSS)
nucleo-impostos-parte-patrimonio = Patrimonio (IPTU, IPVA)
nucleo-impostos-parte-proventos = IR retenido en rendimientos de inversiones
nucleo-impostos-parte-outros = IOF y otros tributos
nucleo-impostos-sem-categoria = Sin categoría
nucleo-impostos-cupons = Tickets fiscales leídos
nucleo-impostos-grupo-cupons = Impuesto de los tickets (NFC-e)
nucleo-impostos-nota-escolha-uf = Elige el estado (UF): sin él, el IPVA no se estima.
nucleo-impostos-nota-escolha-estado = Elige el estado: sin él, el impuesto sobre las ventas no se estima.
nucleo-impostos-nota-escolha-pais = Elige el país: sin él, el IVA no se estima.
# parte: porcentaje entero (sin decimales).
nucleo-impostos-nota-cupons = { $parte }% del consumo vino de tickets fiscales leídos: en esas compras vale el impuesto del ticket (el impreso, o el estimado artículo por artículo con el porcentaje del grupo de consumo).
nucleo-impostos-nota-sem-grupo = { $n ->
        [one] { $n } categoría sin grupo de impuesto quedó fuera: elige su grupo.
       *[other] { $n } categorías sin grupo de impuesto quedaron fuera: elige su grupo.
    }
nucleo-impostos-nota-sem-iptu = Hay un inmueble registrado y ningún IPTU registrado en el año: el IPTU no entra en la estimación.
nucleo-impostos-nota-sem-regras = Los datos de { $dados } no tienen reglas de renta: el impuesto sobre el sueldo no se estima (registra las constancias de ingresos).
nucleo-impostos-nota-salario-estimado = Impuesto e INSS del sueldo estimados a partir del neto recibido, sin descuentos como seguro médico o pensión alimenticia: con la constancia de ingresos, la estimación usa sus montos.
nucleo-impostos-nota-sem-ipva = Hay un vehículo registrado y ningún IPVA registrado: elige el estado para estimar el IPVA.
nucleo-impostos-nota-ipva-estimado = IPVA estimado por el valor de cada vehículo al final del período y la alícuota del estado: registra el IPVA pagado para usar el valor real.

## Datos de cada país (`tax_burden/data/*.json`): nombre, aviso, metodología, contexto y grupos
## de consumo. La traducción solo aparece mientras el texto en pt-BR sea igual al del archivo:
## al actualizar los datos, actualiza también estas líneas; la prueba
## `dataset_texts_are_in_the_catalog` señala lo que cambió.

nucleo-impostos-dados-br-nome = Brasil
nucleo-impostos-dados-br-aviso = Esta es una estimación basada en promedios nacionales de impuesto por categoría de consumo y en las reglas vigentes del impuesto sobre la renta. Los valores reales varían según el producto, el estado, el régimen tributario de las empresas y beneficios fiscales específicos. Fuentes: IBGE (Cuentas Nacionales), Receita Federal, Previdência Social, Tesouro Nacional y Petrobras.
nucleo-impostos-dados-br-metodo-1 = Consumo: el gasto de cada categoría por la fracción del precio que es impuesto sobre el producto. Los porcentajes vienen de las Cuentas Nacionales del IBGE (Tablas de Recursos y Usos de 2021, el último año publicado con 128 productos): impuestos sobre el producto (impuesto de importación, IPI, ICMS y otros impuestos menos subsidios) divididos por la oferta a precio de consumidor. Para gasolina, diésel y gas de cocina vale la composición de precio publicada por Petrobras; para seguros y tarifas bancarias, la alícuota de la ley.
nucleo-impostos-dados-br-metodo-2 = Escenarios: los grupos calculados con las Cuentas Nacionales tienen el mismo valor en los tres escenarios. Cuando un grupo reúne varios productos, el valor es el promedio ponderado por el consumo de los hogares. Los escenarios solo difieren donde la fuente da un rango (combustibles, seguros y tarifas bancarias).
nucleo-impostos-dados-br-metodo-3 = Estado: los porcentajes de consumo son promedios nacionales y no cambian con el estado elegido. El estado entra en el IPVA.
nucleo-impostos-dados-br-metodo-4 = Renta del trabajo: con la constancia de ingresos, valen sus montos. Sin constancia, el bruto de cada mes se estima a partir del sueldo neto recibido, con las tablas del IRPF y del INSS vigentes en el mes (sin dependientes, salvo ajuste, y sin otros descuentos); el aguinaldo (13º) tributa aparte (Ley 8.134/1990, art. 16).
nucleo-impostos-dados-br-metodo-5 = Los impuestos pagados directamente (IR, IOF, IPTU, IPVA y otros tributos registrados en sus categorías) entran por el valor registrado. El IPVA se estima por el valor de cada vehículo y la alícuota del estado (auto flex típico; motos con la alícuota general) solo cuando no se registró ningún IPVA en el año. Los vehículos con antigüedad dentro de la exención del estado, o con 20 años o más a partir de 2026 (EC 137/2025), no pagan.
nucleo-impostos-dados-br-metodo-6 = El IR retenido en rendimientos de inversiones (JCP, dividendos del exterior) viene de los registros de la cartera.
nucleo-impostos-dados-br-metodo-7 = Qué mide el número del IBGE: solo los impuestos cobrados sobre el producto. Los impuestos pagados antes en la cadena y no recuperados, las cargas sobre la nómina y el impuesto sobre la renta de las empresas no entran. Por eso los porcentajes son menores que los de estimaciones que suman esos tributos, y el resultado debe leerse como un piso.
nucleo-impostos-dados-br-metodo-8 = No entran: cargas pagadas por el empleador (INSS patronal, FGTS), grupos sin dato oficial (plan de salud, servicios domésticos, gas vehicular, motocicletas, compras sin detalle) ni la prueba de la reforma tributaria de 2026 (CBS 0,9% e IBS 0,1%, compensados con PIS/Cofins).
nucleo-impostos-dados-br-metodo-9 = Actualización: las Cuentas Nacionales con ese detalle salen con cerca de dos años de atraso. Los cambios de alícuota posteriores a 2021, como el tope del ICMS de energía y telecomunicaciones de 2022, solo aparecen cuando el IBGE publique el año correspondiente.
nucleo-impostos-dados-br-contexto = Carga tributaria bruta de Brasil (% del PIB)
nucleo-impostos-dados-br-dias = Carga tributaria bruta de Brasil en días del año (Tesouro Nacional)
nucleo-impostos-dados-br-destino = Gasto del gobierno general por función (Unión, estados y municipios)
nucleo-impostos-funcao-servicos-publicos-gerais = Servicios públicos generales (incluye intereses de la deuda)
nucleo-impostos-funcao-defesa = Defensa
nucleo-impostos-funcao-ordem-publica-seguranca = Orden público y seguridad
nucleo-impostos-funcao-assuntos-economicos = Asuntos económicos
nucleo-impostos-funcao-protecao-ambiental = Protección ambiental
nucleo-impostos-funcao-habitacao-servicos-comunitarios = Vivienda y servicios comunitarios
nucleo-impostos-funcao-saude = Salud
nucleo-impostos-funcao-lazer-cultura-religiao = Ocio, cultura y religión
nucleo-impostos-funcao-educacao = Educación
nucleo-impostos-funcao-protecao-social = Protección social (pensiones y asistencia)

nucleo-impostos-dados-us-nome = Estados Unidos
nucleo-impostos-dados-us-aviso = Borrador de estimación: considera solo el impuesto sobre las ventas promedio del estado y los impuestos registrados o informados. Los valores reales varían según la ciudad, el producto y la situación de cada persona. Fuente: Tax Foundation.
nucleo-impostos-dados-us-metodo-1 = Borrador: solo el impuesto estatal y local sobre las ventas, con la alícuota combinada promedio del estado (Tax Foundation, mediados de 2026), como fracción del valor pagado: alícuota / (100 + alícuota).
nucleo-impostos-dados-us-metodo-2 = Renta: el impuesto federal se estima mes a mes con los tramos de 2026 de quien declara solo (single), con la deducción estándar y el crédito por hijo divididos por 12; el FICA es Social Security 6,2% hasta el tope anual (US$ 184.500, dividido por 12) y Medicare 1,45% (más 0,9% por encima de US$ 200.000 al año). No entran los impuestos estatales y locales sobre la renta; con el W-2, registra la constancia de ingresos para usar los valores reales.

nucleo-impostos-dados-eu-nome = Unión Europea
nucleo-impostos-dados-eu-aviso = Borrador de estimación: considera solo el IVA del país elegido y los impuestos registrados o informados. Las alícuotas reducidas varían por producto en cada país. Fuente: Comisión Europea.
nucleo-impostos-dados-eu-metodo-1 = Borrador: solo el IVA, con las alícuotas general, reducida (la menor) y superreducida del país elegido (tabla "Your Europe" de la Comisión Europea, revisada el 13/07/2026).
nucleo-impostos-dados-eu-metodo-2 = Renta (solo Portugal): el IRS se estima con el impuesto anual de los tramos de 2026 del continente, dividido por los 14 pagos del año (12 sueldos y las pagas extra de vacaciones y de Navidad), con la deducción específica (el mayor entre 4.587,09 € y la Seguridad Social), 600 € por dependiente y el mínimo de existencia; la Seguridad Social es el 11% del bruto. No son las tablas mensuales de retención (que retienen un poco distinto y se ajustan en la declaración). Los demás países de la Unión Europea todavía no tienen reglas de renta: registra las constancias de ingresos.

# Grupos de consumo (los mismos en los tres países).
nucleo-impostos-grupo-alimentos-basicos = Alimentos básicos
nucleo-impostos-grupo-alimentos-industrializados = Alimentos industrializados
nucleo-impostos-grupo-bebidas-nao-alcoolicas = Bebidas sin alcohol
nucleo-impostos-grupo-bebidas-alcoolicas = Bebidas alcohólicas
nucleo-impostos-grupo-restaurantes = Restaurantes, bares y delivery
nucleo-impostos-grupo-gasolina = Gasolina común (gasolina C)
nucleo-impostos-grupo-diesel = Diésel (diésel B)
nucleo-impostos-grupo-etanol = Etanol hidratado
nucleo-impostos-grupo-gnv = Gas natural vehicular (GNV)
nucleo-impostos-grupo-glp = Gas de cocina (GLP, garrafa de 13 kg)
nucleo-impostos-grupo-energia-eletrica = Energía eléctrica (factura de luz residencial)
nucleo-impostos-grupo-agua-esgoto = Agua y alcantarillado
nucleo-impostos-grupo-telefonia-internet = Telefonía e internet
nucleo-impostos-grupo-tv-assinatura-streaming = TV paga y streaming
nucleo-impostos-grupo-eletronicos-nacionais = Electrónica (fabricación nacional o sin distinción)
nucleo-impostos-grupo-eletronicos-importados = Electrónica importada
nucleo-impostos-grupo-eletrodomesticos = Electrodomésticos
nucleo-impostos-grupo-moveis = Muebles y colchones
nucleo-impostos-grupo-vestuario = Ropa
nucleo-impostos-grupo-calcados = Calzado
nucleo-impostos-grupo-medicamentos = Medicamentos de uso humano
nucleo-impostos-grupo-higiene-pessoal = Higiene personal
nucleo-impostos-grupo-cosmeticos-perfumes = Cosméticos y perfumes
nucleo-impostos-grupo-servicos-pessoais = Servicios personales (peluquería, gimnasio/club)
nucleo-impostos-grupo-educacao-mensalidade = Educación – cuota escolar
nucleo-impostos-grupo-educacao-livros = Educación – libros
nucleo-impostos-grupo-saude-plano-consulta = Salud – seguro médico y consultas
nucleo-impostos-grupo-transporte-publico = Transporte público
nucleo-impostos-grupo-transporte-aplicativo = Transporte por aplicación
nucleo-impostos-grupo-passagens-aereas = Pasajes aéreos
nucleo-impostos-grupo-hotel-turismo = Hotel y turismo
nucleo-impostos-grupo-automovel-novo = Auto nuevo
nucleo-impostos-grupo-motocicleta = Motocicleta
nucleo-impostos-grupo-pecas-manutencao-veiculo = Repuestos y mantenimiento del vehículo
nucleo-impostos-grupo-seguros = Seguros (IOF sobre la prima)
nucleo-impostos-grupo-tarifas-bancarias = Comisiones bancarias
nucleo-impostos-grupo-brinquedos = Juguetes
nucleo-impostos-grupo-pet = Mascotas (alimento, medicamentos, veterinario)
nucleo-impostos-grupo-materiais-construcao = Materiales de construcción
nucleo-impostos-grupo-cigarros = Cigarrillos
nucleo-impostos-grupo-mercado = Supermercado (canasta mixta)
nucleo-impostos-grupo-aluguel = Alquiler de vivienda
nucleo-impostos-grupo-condominio = Gastos comunes del edificio
nucleo-impostos-grupo-servicos-domesticos = Servicio doméstico (limpieza, niñera)
nucleo-impostos-grupo-estacionamento = Estacionamiento
nucleo-impostos-grupo-optica = Anteojos y lentes
nucleo-impostos-grupo-material-escolar = Útiles escolares
nucleo-impostos-grupo-lazer = Ocio (paseos, entradas, fiestas)
nucleo-impostos-grupo-presentes = Regalos (artículos variados)
nucleo-impostos-grupo-compras-diversas = Compras varias (sin detalle)
nucleo-impostos-grupo-contas-diversas = Cuentas y boletas (sin detalle)
nucleo-impostos-grupo-limpeza = Artículos de limpieza
nucleo-impostos-anual-nota-sem-holerite = No hay recibos de sueldo del año en la bóveda: ingresa los rendimientos del informe de la empresa.
nucleo-impostos-anual-nota-holerites = { $fonte }: recibos de { $meses } meses, { $valor } gravables (sin el 13.º ni la participación en las ganancias, que tributan de forma exclusiva). Confírmalo con el informe de rendimientos.
nucleo-impostos-anual-nota-65 = Jubilación desde los 65 años: { $valor } quedaron fuera como parte exenta (R$ 1.903,98 por mes).
nucleo-impostos-anual-nota-despesas = Gastos deducibles por las categorías: salud { $saude } (plan, consultas, exámenes, dentista, terapias) e instrucción { $instrucao } (escuela, guardería, universidad). Farmacia, lentes, material escolar, idiomas y cursos libres no entran.
nucleo-impostos-anual-nota-pgbl = Aportes a PGBL y fondo de pensión: { $valor } (deducibles hasta el 12 % de los rendimientos gravables).
nucleo-impostos-anual-nota-dedo-duro = IRRF "dedo-duro" de la bolsa que sobró en el año: { $valor }, cuenta como impuesto pagado.
nucleo-impostos-anual-nota-familia = Toda la familia: la declaración es por persona. Elige un miembro para la vista previa de cada declaración.
nucleo-impostos-anual-nota-confira = Vista previa para revisar: ajusta los valores con los informes y recibos. La declaración oficial se hace en el programa de la Receita.
nucleo-impostos-anual-nota-irpfm = Rendimientos por encima de R$ 600 mil en el año: se aplica el impuesto mínimo a las altas rentas (IRPFM, Ley 15.270/2025). La estimación no aplica el reductor por la carga de las empresas (art. 16-B), que puede reducirlo o anularlo.
nucleo-impostos-gcap-erro-venda = Ingresa la fecha y el valor de la venta para calcular la ganancia de capital.
