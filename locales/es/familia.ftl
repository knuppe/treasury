# Pantalla 02 · Familia: los miembros, el registro de cada uno, los ingresos, los recibos de nómina
# importados, las cuentas y tarjetas de cada miembro y la previsión de ingresos de la familia
# (docs/i18n-design.md). Los nombres de la nómina de otros países todavía vienen del núcleo.
# Revisión: borrador por IA, sin revisión de un hablante nativo.

familia-nome-instituicoes = Nombre de la familia e instituciones

## Cómo llama Brasil a las cosas del ingreso (los otros países vienen del núcleo)

familia-tipo-clt = CLT
familia-tipo-servidor = Servidor público
familia-tipo-aposentadoria = Jubilación
familia-tipo-pensao = Pensión
familia-tipo-pj = PJ
familia-tipo-autonomo = Independiente
familia-tipo-informal = Informal
familia-tipo-aluguel = Alquiler
familia-tipo-beneficio = Beneficio social
familia-tipo-outra = Otro
familia-holerite = recibo de nómina
familia-holerites = recibos de nómina
familia-descontos = { $contribuicao } e { $imposto }
familia-descontos-eua = FICA e impuesto federal
familia-13-salario = 13º salario
familia-subsidio-natal = subsidio de Navidad
familia-terco-ferias = tercio de vacaciones
familia-subsidio-ferias = subsidio de vacaciones
familia-dependente-dica-br = Entra como dependiente en la declaración de quien declara (deducción en el impuesto retenido).
familia-dependente-dica = Cuenta como dependiente en el impuesto de quien declara (reduce el impuesto retenido).

## Registro del miembro

familia-relacao-titular = Responsable
familia-relacao-conjuge = Cónyuge
familia-relacao-filho = Hijo(a)
familia-relacao-pai-mae = Padre o madre
familia-relacao-outro = Otro
familia-cadastro-de = Registro de { $nome }
familia-relacao = Relación en la familia
familia-escolha = Elige
familia-nascimento = Fecha de nacimiento
familia-sexo = Sexo
familia-sexo-dica = Opcional. Se usa en la línea de vida y como valor predeterminado en el cálculo del INSS.
familia-sexo-nao-informado = No informado
familia-sexo-homem = Hombre
familia-sexo-mulher = Mujer
familia-sexo-nao-informar = Prefiero no decirlo
familia-ocupacao = Profesión u ocupación
familia-observacoes = Observaciones
familia-dependente-ir = Dependiente en el impuesto sobre la renta
familia-sem-cpf = La app no guarda el CPF ni documentos.
familia-cadastro-salvo = Registro guardado

## Alergias y restricciones alimentarias (docs/gtin-inflation.md §7.2)

familia-dieta-titulo = Alergias y restricciones alimentarias
familia-dieta-dica = Opcional. Se queda solo en tu bóveda; Treasury te avisa en 06 · Consumo › Salud cuando un recibo trae algo con alguno de estos (según Open Food Facts).
familia-dieta-gluten = Gluten
familia-dieta-milk = Leche (lactosa)
familia-dieta-eggs = Huevos
familia-dieta-peanuts = Cacahuate (maní)
familia-dieta-nuts = Frutos secos
familia-dieta-soybeans = Soya
familia-dieta-fish = Pescado
familia-dieta-crustaceans = Crustáceos
familia-dieta-sesame = Ajonjolí (sésamo)
familia-dieta-vegan = Vegano
familia-dieta-vegetarian = Vegetariano

## Ingresos

familia-renda-alterada = Ingreso modificado
familia-renda-cadastrada = Ingreso registrado
familia-editar-nome = Editar { $nome }
familia-nova-renda = Nuevo ingreso
familia-renda-descricao = Bruto y neto de referencia del mes. Con un { $holerite } importado, la previsión usa sus valores.
familia-nome = Nombre
familia-nome-exemplo = Salario, jubilación, trabajos ocasionales…
familia-tipo = Tipo
familia-bruto-mes = Bruto por mes
familia-bruto-dica = Antes de las deducciones ({ $descontos }).
familia-opcional = Opcional.
familia-liquido-mes = Neto por mes
familia-liquido-dica = Lo que llega a la cuenta.
familia-empregador-fonte = Empleador o fuente pagadora
familia-registro-empresa = Registro de la empresa
familia-registro-dica = Vincula los { $holerites } a este ingreso automáticamente.
familia-dia-pagamento = Día de pago
familia-mes-ferias = Mes de vacaciones
familia-mes-ferias-dica = El { $ferias } entra en la previsión.
familia-sem-ferias = Sin vacaciones programadas
familia-comeca-em = Empieza el
familia-termina-em = Termina el
familia-termina-dica = Contrato temporal, beneficio con plazo.
familia-recebe = Recibe { $extra }
familia-valor-variavel = El monto cambia de un mes a otro
familia-valor-variavel-dica = Comisiones, trabajos ocasionales, ventas: la previsión usa el promedio de los últimos 6 meses de movimientos de la categoría del ingreso, cuando los hay.

## Recibo de nómina

familia-importar-holerite = Importar { $holerite }
familia-importar-descricao = El PDF del { $holerite }: lo revisas antes de guardar. El nombre y los documentos personales no se guardan.
familia-holerite-pdf = { $holerite } en PDF
familia-senha-se-tiver = Contraseña (si tiene)
familia-ler-de-novo = Leer de nuevo
familia-lendo = Leyendo el { $holerite }…
familia-competencia = Mes de referencia
familia-empregador = Empleador
familia-renda = Ingreso
familia-renda-nova = nuevo ({ $tipo })
familia-bruto = Bruto
familia-outros-descontos = Otras deducciones
familia-liquido = Neto
familia-salario-conta = Salario en la cuenta
familia-nao-achado = no encontrado
familia-competencia-mes = Mes de referencia (AAAA-MM)
familia-sem-mes = El { $holerite } no trajo el mes.
familia-ja-importado = Este { $holerite } ya se importó: guardar reemplaza el anterior.
familia-holerite-importado = { $holerite } importado
familia-gravar = Guardar { $holerite }

## Cuentas y tarjetas de cada miembro

familia-contas-de = Cuentas y tarjetas de { $nome }
familia-contas-descricao = Los movimientos de una cuenta o tarjeta de un solo miembro quedan con él, incluso los antiguos (menos los que cambiaste a mano). Una cuenta conjunta, con dos o más miembros, queda de la familia.
familia-conjunta-com = conjunta con { $nomes }
familia-de-outros = de { $nomes }
familia-pronto = Listo

## Inicio guiado por el cuestionario

familia-inicio-fixa-calcula = En el cuestionario, dijiste que tu ingreso principal es fijo. Registra el salario con el bruto y el neto, o importa un { $holerite }: la app calcula { $itens }.
familia-inicio-fixa-recibo = En el cuestionario, dijiste que tu ingreso principal es fijo. Registra el salario con el bruto y el neto, o importa un { $holerite } (las deducciones vienen del recibo; la app no calcula los impuestos de este país).
familia-dica-fixa = Buena práctica: separa el ahorro el día de pago, antes de los gastos (págate primero).
familia-inicio-variavel = En el cuestionario, dijiste que tu ingreso es variable. Regístralo como ingreso variable: la previsión usa el promedio de los últimos 6 meses.
familia-dica-variavel = Buena práctica: planifica el mes según los meses más flojos y mantén un fondo de emergencia de 6 a 12 meses de gastos.
familia-inicio-beneficios = En el cuestionario, dijiste que tu ingreso viene de beneficios. Registra cada beneficio ({ $exemplos }) con el monto del mes.
familia-beneficios-br = jubilación, BPC, Bolsa Família
familia-beneficios-pt = pensión de jubilación, asignación familiar, subsidio de desempleo
familia-beneficios-us = Social Security, SNAP, seguro de desempleo
familia-beneficios-outros = jubilación, pensión, beneficios del gobierno
familia-dica-beneficios = Buena práctica: el beneficio es la parte segura del mes; organiza las cuentas fijas en torno a él y trata el resto como extra.
familia-inicio-mista = En el cuestionario, dijiste que tu ingreso es mixto. Registra cada parte: el salario (o beneficio) y el ingreso variable por separado.
familia-dica-mista = Buena práctica: cuenta con la parte fija para las cuentas y guarda una parte de la variable cuando llegue.
familia-inicio-padrao = Registra el ingreso de cada miembro de la familia: salario, jubilación, beneficio, trabajos ocasionales o alquiler. Es la base de la previsión de saldo, del presupuesto y de los consejos.
familia-dica-padrao = Con el ingreso registrado, la app sabe cuánto entra cada mes y cuánto puedes ahorrar.
familia-comece = Empieza por los ingresos de la familia
familia-boas-praticas-em = Buenas prácticas en { $pais }
familia-cadastrar-primeira = Registrar el primer ingreso

## Nuevo miembro

familia-membro-adicionado = Miembro agregado
familia-novo-membro = Nuevo miembro de la familia
familia-adicionar = Agregar

## Panel de ingresos

familia-bruto-12m = Bruto en los próximos 12 meses
familia-liquido-12m = Neto en los próximos 12 meses
familia-descontos-previstos = { $descontos } previstos
familia-impostos-holerites = Impuestos en los { $holerites } (12 meses)
familia-renda-prevista = Ingreso previsto por mes
familia-renda-fonte = Fuente: ingresos registrados, el último { $holerite } de cada uno y las reglas de { $descontos } del año
familia-renda-real-prevista = Ingresos por mes: reales y previstos
familia-renda-fonte-base = Reales: { $holerites } importados. Previsto: último { $holerite } ({ $mes }) y las reglas de { $descontos } del año
familia-renda-tipo = Tipo
familia-renda-real = Real
familia-renda-previsto = Previsto
familia-renda-real-faixa = REAL
familia-renda-previsto-faixa = PREVISTO
familia-renda-linha-prevista = Neto previsto por el último { $holerite }
familia-renda-diferenca = Diferencia con lo previsto
familia-renda-diferenca-coluna = Real − previsto
familia-renda-igual = igual
familia-renda-sem-holerite = Sin { $holerite } este mes.
familia-renda-nota-real = Previsto: lo que el último { $holerite } ({ $mes }) daría para este mes.
familia-renda-nota-previsto = Previsión por el último { $holerite } ({ $mes }).
familia-mes = Mes
familia-extras = Extras
familia-cadastro = Registro
familia-dependente-ir-curto = dependiente en el impuesto
familia-cadastro-incompleto = Registro incompleto: la relación, la fecha de nacimiento y la ocupación ayudan en las estimaciones.
familia-aliquota-efetiva = tasa efectiva en los recibos de nómina: { NUMBER($aliquota, maximumFractionDigits: 1) }%
familia-contas-cartoes = Cuentas y tarjetas:
familia-conta-conjunta-dica = Cuenta conjunta: queda de la familia
familia-conjunta = (conjunta)
familia-nenhuma = ninguna
familia-escolher = Elegir
familia-rendas-de = Ingresos de { $nome }
familia-nenhuma-renda = Ningún ingreso registrado.
familia-previsao = Previsión
familia-formal = formal
familia-informal = informal
familia-holerite-de = { $holerite } de { $mes }
familia-base-holerite = por el último { $holerite }
familia-base-informados = por los valores informados
familia-base-media = por el promedio de los últimos 6 meses
familia-base-nenhum = sin valor informado
familia-sem-membro = Cuentas y tarjetas sin miembro
familia-sem-membro-explicacao = Elige de quién es cada cuenta: sus movimientos pasan a tener ese miembro automáticamente. Deja sin miembro lo que es de todos.
familia-membro-de = Miembro de { $nome }
familia-da-familia = De la familia
familia-holerites-importados = { $holerites } importados
familia-membro = Miembro
familia-na-conta = En la cuenta
familia-ligado = vinculado

## Simular despido (ingreso CLT, solo Brasil)

familia-rescisao-botao = Simular despido
familia-rescisao-titulo = Simular despido · { $renda }
familia-rescisao-descricao = Cuánto recibirías al dejar este empleo: el finiquito, el FGTS y el seguro de desempleo. Es una estimación con las reglas generales de la CLT; no se guarda nada.
familia-rescisao-motivo = Motivo
familia-rescisao-motivo-op = { $motivo ->
    [sem_justa_causa] Despido sin justa causa
    [pedido_demissao] Renuncia
    [acordo] Acuerdo (art. 484-A)
    [justa_causa] Despido con justa causa
    [fim_experiencia] Fin del contrato de prueba
    [quebra_experiencia_empregador] Prueba terminada antes por la empresa
   *[quebra_experiencia_empregado] Prueba terminada antes por ti
}
familia-rescisao-data = Último día de trabajo
familia-rescisao-aviso = Preaviso
familia-rescisao-aviso-op = { $aviso ->
    [indenizado] Indemnizado (sin trabajar)
    [trabalhado] Trabajado
   *[nao_cumprido] No cumplido (descuento de 30 días)
}
familia-rescisao-fim-contrato = Fin acordado de la prueba
familia-rescisao-admissao = Contratación
familia-rescisao-ferias-vencidas = Vacaciones vencidas (días)
familia-rescisao-ferias-dica = Días de vacaciones de periodos completos que no tomaste.
familia-rescisao-saldo-fgts = Saldo del FGTS de este empleo
familia-rescisao-saldo-fgts-dica = Vacío: el de la cuenta del FGTS vinculada a este ingreso, si la hay.
familia-rescisao-pedidos = Ya recibió seguro de desempleo
familia-rescisao-pedidos-op = { $n ->
    [0] Nunca
    [1] Una vez
   *[other] Dos veces o más
}
familia-rescisao-preencha = Indica las fechas para ver la simulación.
familia-rescisao-verbas = Finiquito
familia-rescisao-liquido = Finiquito neto
familia-rescisao-fgts = FGTS
familia-rescisao-fgts-saldo = Saldo para fines de despido
familia-rescisao-fgts-deposito = Depósito sobre el finiquito
familia-rescisao-fgts-multa = Multa del { $pct }%
familia-rescisao-fgts-sacavel = Puedes retirar
familia-rescisao-seguro = Seguro de desempleo
familia-rescisao-seguro-parcelas = { $n } cuotas de
familia-rescisao-seguro-total = Total del seguro
familia-rescisao-total = Total disponible
familia-rescisao-meses = Cubre unos { $meses } meses de tu salario neto actual.
familia-rescisao-tabelas = Contrato proyectado hasta { $fim }. INSS e IRRF: { $base }.
