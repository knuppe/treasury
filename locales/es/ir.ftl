# Informes › IR: cálculo mensual del impuesto sobre la renta brasileño en la bolsa, DARF (guías de
# pago de impuestos federales) e informe anual para la declaración (solo en Brasil;
# docs/i18n-design.md). Los términos oficiales (DARF, IRRF, IR, JCP, FII, CNPJ, Sicalc, Receita
# Federal) quedan tal cual. Revisión: borrador por IA, sin revisión nativa.

## Situación del mes, grupos de pérdidas y DARF

ir-mes-sem-vendas = Sin ventas
ir-mes-sem-vendas-dica = Ninguna venta en bolsa en el mes.
ir-mes-sem-imposto = Sin impuesto
ir-mes-sem-imposto-dica = Hubo ventas, pero sin impuesto: exención, pérdida o compensación.
ir-mes-abaixo-minimo = Menos de R$ 10
ir-mes-abaixo-minimo-dica = Impuesto menor de R$ 10,00: no se paga DARF; el valor se suma al mes siguiente.
ir-mes-a-pagar = A pagar
ir-mes-a-pagar-dica = Hay impuesto a pagar hasta el último día hábil del mes siguiente.
ir-mes-pago = Pagado
ir-mes-pago-dica = DARF del mes pagado.
ir-grupo-comum = Común (acciones, ETF, BDR)
ir-grupo-comum-curto = Común
ir-grupo-day-trade = Day trade
ir-grupo-fii = FII (fondos inmobiliarios)
ir-grupo-fii-curto = FII
ir-darf-em-aberto = Pendiente
ir-darf-cancelado = Cancelado
ir-darf-vencido = Vencido
ir-aviso = Cálculo auxiliar hecho con tus datos; verifícalo en el programa de la Receita Federal (autoridad tributaria de Brasil). No es asesoría tributaria.

## Panel

ir-secao-apuracao = Cálculo mensual
ir-secao-darf = DARF
ir-secao-declaracao = Declaración anual
ir-aviso-titulo = No es asesoría tributaria
ir-aviso-regras = Reglas de bolsa para personas físicas (15 % en operaciones comunes, 20 % en day trade y FII, exención de R$ 20 mil al mes en ventas de acciones al contado).
ir-ano-calendario = Año calendario
ir-membro = Miembro
ir-familia-toda = Toda la familia
ir-secao = Sección del IR
ir-apurando = Calculando el año
ir-imposto-em = Impuesto en { $ano }
ir-renda-variavel = Renta variable
ir-darfs-total = DARF: { $valor }
ir-a-pagar = A pagar
ir-meses-sem-darf = Meses sin DARF
ir-meses-sem-darf-n = { $n ->
    [one] { $n } mes sin DARF
   *[other] { $n } meses sin DARF
}
ir-darfs-vencidos = { $n ->
    [one] ! { $n } DARF vencido
   *[other] ! { $n } DARF vencidos
}
ir-prejuizos = Pérdidas
ir-somando-grupos = Sumando los tres grupos
ir-fim-periodo = al final del período calculado
ir-carregando-darfs = Cargando los DARF
ir-sem-darf-em = Impuesto a pagar sin DARF en { $meses }.
ir-fonte-regras = reglas de renta variable vigentes en { $ano }

## Copiar

ir-copiar = Copiar
ir-copiado = ✓ Copiado
ir-copiar-algo = Copiar { $algo }
ir-copiar-a-mao = Selecciona el valor y cópialo a mano.

## Cálculo del mes

ir-meses-vazio = Ninguna venta en bolsa en este año.
ir-mes = Mes
ir-vendas-acoes = Ventas de acciones
ir-isento = exento
ir-imposto = Impuesto
ir-para-proximo = { $valor } → siguiente
ir-situacao = Situación
ir-grupo = Grupo
ir-resultado = Resultado
ir-prejuizo-antes = Pérdida anterior
ir-compensado = Compensado
ir-base = Base
ir-aliquota = Tasa
ir-prejuizo-compensar = Pérdida a compensar
ir-apuracao-de = Cálculo de { $mes }
ir-darf-gerado = DARF generado
ir-darf-gerado-texto = Código { $codigo }, vence el { $vencimento }. Emite la guía en Sicalc con estos valores.
ir-gerar-darf = Generar DARF de { $valor }
ir-ver-darf = Ver el DARF
ir-notas-apuracao = Notas del cálculo
ir-vendas-categoria = Ventas y resultado por categoría
ir-categorias-de = Categorías de { $mes }
ir-categoria = Categoría
ir-vendas = Ventas
ir-ganho-isento = Ganancia exenta
ir-prejuizo-mesmo-grupo = la pérdida solo se compensa en el mismo grupo
ir-prejuizos-base = Pérdidas y base de cálculo
ir-grupos-prejuizo = Grupos de pérdidas
ir-imposto-mes = Impuesto del mes
ir-irrf-estimado = IRRF (≈ estimado)
ir-irrf-mes = IRRF del mes
ir-irrf-anteriores = IRRF de meses anteriores
ir-irrf-compensado = IRRF compensado
ir-devido-mes = Adeudado en el mes
ir-acumulado = Acumulado (< R$ 10)
ir-valor-darf-de = valor del DARF de { $periodo }
ir-codigo-receita = Código de ingreso
ir-periodo-apuracao = Período de cálculo
ir-vencimento = Vencimiento
ir-vai-mes-seguinte = Pasa al mes siguiente
ir-irrf-depois = IRRF a compensar después
ir-vendas-apuracao = Ventas que entraron en el cálculo
ir-vendas-de = Ventas de { $mes }
ir-data = Fecha
ir-ativo = Activo
ir-quantidade = Cantidad
ir-venda = Venta
ir-custo-medio = Costo promedio

## DARF

ir-darfs-venceram = { $n ->
    [one] ! Un DARF venció sin pago: recalcúlalo en Sicalc con multa e intereses antes de pagar.
   *[other] ! { $n } DARF vencieron sin pago: recalcúlalos en Sicalc con multa e intereses antes de pagar.
}
ir-darfs-vazio = Ningún DARF generado en este año. Genéralo desde el cálculo del mes con impuesto a pagar.
ir-periodo = Período
ir-codigo = Código
ir-valor = Valor
ir-acoes = Acciones
ir-cancelar = Cancelar
ir-marcar-pago-de = Marcar el DARF de { $periodo } como pagado
ir-marcar-pago = Marcar pagado
ir-lancamento = Movimiento
ir-cancelar-darf = Cancelar DARF
ir-cancelar-darf-texto = El DARF de { $periodo } sale de la lista y el mes vuelve a quedar a pagar (puedes generar otro). Puedes deshacerlo en el historial de cambios.
ir-darf-cancelado-toast = DARF cancelado
ir-darf-pago-titulo = DARF de { $periodo } pagado
ir-darf-pago-descricao = Código { $codigo }, vencimiento { $vencimento }.
ir-darf-pago = DARF pagado
ir-despesa-lancada = Gasto registrado en Impuestos y tasas › Impuesto sobre la renta.
ir-marcar-como-pago = Marcar como pagado
ir-data-pagamento = Fecha de pago
ir-lancar-despesa = Registrar el gasto en la cuenta
ir-lancar-despesa-dica = Opcional: crea el gasto en Impuestos y tasas › Impuesto sobre la renta. Si ya importaste el extracto, déjalo en blanco.
ir-nao-lancar = No registrar
ir-pago-atrasado = Pagado después del vencimiento: el valor con multa e intereses (Sicalc) puede ser mayor que el registrado aquí.

## Declaración anual

ir-ficha-isentos = Rendimentos isentos (ingresos exentos)
ir-ficha-exclusiva = Tributação exclusiva (tributación exclusiva en la fuente)
ir-linha-dividendos = Utilidades y dividendos recibidos
ir-linha-isentos-acoes = Ganancias netas en acciones con ventas de hasta R$ 20 mil en el mes
ir-linha-fii = Rendimientos de FII
ir-linha-jcp = Intereses sobre capital propio (JCP)
ir-n-ativos = { $n ->
    [one] { $titulo } · { $n } activo
   *[other] { $titulo } · { $n } activos
}
ir-copiar-de = { $titulo } de { $ativo }
ir-rendimentos-ano = Rendimientos · año calendario { $ano }
ir-rendimentos-declaracao = Rendimientos para la declaración
ir-ficha = Ficha
ir-codigo-muda = El código cambia entre versiones del programa: verifícalo
ir-confira = verificar
ir-descricao = Descripción
ir-dividendos = Dividendos
ir-jcp = JCP
ir-rendimentos-nota = Valores netos recibidos en { $ano }, en reales. El programa pide la fuente pagadora (CNPJ, identificación fiscal de la empresa) de cada línea: verifícala en la constancia de ingresos de la corredora.
ir-resumo-renda-variavel = Renta variable · resumen del año
ir-impostos-ano = Impuestos del año
ir-imposto-apurado = Impuesto calculado
ir-darfs-ano = DARF del año
ir-total-darfs = total de DARF
ir-ganhos-isentos = Ganancias exentas (acciones)
ir-ganhos-isentos-copiar = ganancias exentas de acciones
ir-prejuizos-31-12 = Pérdidas a compensar al 31/12
ir-prejuizo-de = pérdida { $grupo }
ir-bens-em = Bienes y derechos al 31/12/{ $ano }
ir-bens-situacao = costo de adquisición (precio promedio), no valor de mercado
ir-codigos = Códigos
ir-codigos-sugeridos = sugeridos: verifícalos en el programa
ir-bens-direitos = Bienes y derechos
ir-bens-vazio = Ninguna posición al final del año.
ir-grupo-codigo = Grupo · código
ir-discriminacao = Descripción
ir-discriminacao-de = descripción de { $ativo }
ir-situacao-em = situación al 31/12/{ $ano } de { $ativo }
ir-cripto-titulo = Criptoactivos en Brasil (ganancia de capital)
ir-cripto-vendido = Vendido en cripto en el mes (Brasil y exterior)
ir-cripto-vendido-brasil = Vendido en exchanges de Brasil
ir-cripto-ganho = Ganancia de las ventas en Brasil
ir-cripto-perda = Pérdidas (no reducen la ganancia)
ir-cripto-situacao = Situación
ir-cripto-isento = Exento: ventas de hasta R$ 35 mil en el mes
ir-cripto-tributado = Gravado: ventas de más de R$ 35 mil en el mes
ir-cripto-nota = Cripto custodiada en Brasil: impuesto del 15 % al 22,5 % sobre la ganancia, en un DARF propio (código 4600), cuando el total vendido en cripto en el mes, en Brasil y en el exterior, supera R$ 35 mil. La cripto custodiada en el exterior va a Inversiones en el exterior, en el informe anual.
ir-cripto-exterior = Exterior
ir-cripto-brasil = Brasil
ir-cripto-custodia-auto = Automático ({ $lugar })
ir-cripto-custodia-de = Custodia de la cripto en { $conta }
ir-cripto-custodia-dica = Dónde está custodiada la cripto de cada cuenta. En automático, una cuenta en reales es Brasil y una cuenta en otra moneda es exterior.
ir-cripto-darf-pago = DARF 4600 pagado
ir-cripto-darf-gerado = DARF 4600 generado
ir-irrf-sobra-declaracao = IRRF "dedo-duro" para la declaración (Impuesto pagado/retenido)
ir-irrf-sobra-declaracao-rotulo = IRRF de la Ley 11.033 para la declaración
ir-irrf-sobra-day-trade = IRRF de day trade no compensado (solo por restitución)
ir-exterior = Inversiones en el exterior (Ley 14.754/2023)
ir-ganho-realizado = Ganancia realizada en el año
ir-ganho-exterior = ganancia en el exterior
ir-dividendos-juros = Dividendos e intereses (brutos)
ir-rendimentos-exterior = rendimientos del exterior
ir-imposto-pago-exterior = Impuesto pagado en el exterior
ir-imposto-pago-exterior-rotulo = impuesto pagado en el exterior
ir-credito-exterior = Crédito usado (hasta el 15 % de cada rendimiento)
ir-sem-credito-exterior = Sin crédito (país sin convenio ni reciprocidad)
ir-prejuizo-exterior-anterior = Pérdidas de años anteriores
ir-prejuizo-exterior-seguinte = Pérdidas para los años siguientes
ir-imposto-estimado-15 = Impuesto estimado (15 %)
ir-exterior-nota = Se calcula en la declaración anual: 15 % sobre la ganancia (después de las pérdidas de años anteriores) y sobre dividendos e intereses brutos, menos el impuesto pagado afuera, que solo vale hasta el impuesto brasileño de cada rendimiento. Incluye en la ganancia los criptoactivos de cuentas en otra moneda (los de cuentas en reales se calculan mes a mes, con DARF 4600).
ir-anual-titulo = Declaración anual de { $ano } (vista previa)
ir-anual-carregando = Armando la vista previa de la declaración…
ir-anual-dica = Valores del año tomados de la bóveda. Revísalos con los informes de rendimientos y los recibos y ajusta lo necesario: los dos modelos se recalculan al instante, sin guardar nada.
ir-anual-campo-income = Rendimientos gravables de empresas (sin 13.º ni participación en las ganancias)
ir-anual-campo-social_security = Seguridad social oficial (INSS)
ir-anual-campo-withheld = Impuesto retenido en la fuente
ir-anual-campo-self_employed = Rendimientos de personas físicas y del exterior
ir-anual-campo-carne_leao = Carnê-leão pagado
ir-anual-campo-medical = Gastos médicos
ir-anual-campo-education = Gastos de instrucción
ir-anual-campo-private_pension = Previsión complementaria (PGBL)
ir-anual-campo-alimony = Pensión alimenticia (judicial o escritura)
ir-anual-campo-complementary_tax = Impuesto complementario
ir-anual-campo-irrf_lei_11033 = IRRF "dedo-duro" (Ley 11.033)
ir-anual-campo-dependentes = Dependientes
ir-anual-completo = Completo
ir-anual-simplificado = Simplificado
ir-anual-mais-vantajoso = Más ventajoso
ir-anual-deducoes = Deducciones
ir-anual-base = Base de cálculo
ir-anual-imposto-tabela = Impuesto según la tabla
ir-anual-reducao = Reducción de la Ley 15.270
ir-anual-doacoes = Donaciones (fondos de niños y personas mayores)
ir-anual-devido = Impuesto debido
ir-anual-pago = Impuesto ya pagado
ir-anual-a-pagar = Saldo a pagar
ir-anual-a-restituir = A restituir
ir-anual-aliquota-efetiva = Alícuota efectiva en el modelo más ventajoso: { $aliquota }.
ir-anual-restaurar = Volver a los valores de la bóveda
ir-anual-de-onde = De dónde vienen los valores
ir-dividendos-acima-titulo = Dividendos de más de R$ 50 mil en el mes de una misma empresa
ir-dividendos-acima-linha = { $mes }: { $ativos }, { $valor } → retención del 10 % sobre el total ({ $retencao })
ir-dividendos-acima-nota = Desde 2026 la empresa retiene el 10 % de todo lo que pague en el mes por encima de ese límite (Ley 15.270/2025). Las ganancias de hasta 2025 con distribución aprobada hasta el 31/12/2025 quedan fuera: revisa el informe de la empresa. Lo retenido cuenta en el impuesto mínimo a las altas rentas.
ir-irpfm-titulo = Impuesto mínimo a las altas rentas (IRPFM)
ir-irpfm-rendimentos = Rendimientos del año en la base
ir-irpfm-bruto = IRPFM antes de las deducciones ({ $aliquota })
ir-irpfm-creditos = IR de la declaración, exclusivo y definitivo
ir-irpfm-devido = IRPFM a pagar (negativo: a compensar)
ir-darf-atraso-hoje = Pagando hoy: { $total } (multa { $multa } + intereses { $juros })
ir-darf-atraso-hoje-estimado = Pagando hoy: ≈ { $total } (multa { $multa } + intereses { $juros }; Selic estimada en algún mes)
ir-darf-com-acrescimos = Incluye multa { $multa } e intereses { $juros }
ir-carne-titulo = Carnê-leão del mes (rendimientos de personas físicas y del exterior)
ir-carne-dica = Para quien recibe de persona física o del exterior (alquiler de inquilino persona física, atención particular, trabajo para el exterior): el impuesto sale por la tabla mensual y se paga hasta el fin del mes siguiente.
ir-carne-recebido = Recibido en el mes
ir-carne-inss = Seguridad social oficial pagada
ir-carne-pensao = Pensión alimenticia pagada
ir-carne-livro-caixa = Libro-caja (gastos del trabajo)
ir-carne-imposto = Carnê-leão del mes
ir-carne-darf = código { $codigo }, vence el { $vencimento }
ir-carne-abaixo-minimo = Menos de R$ 10: se suma al mes siguiente
