# IPTU (impuesto municipal a la propiedad en Brasil) de inmuebles y terrenos: la boleta de cada año
# en Bienes, los recordatorios de vencimiento y el IPTU en los impuestos pagados (docs/brazil/iptu.md).
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

iptu-titulo = IPTU
iptu-descricao = La boleta de cada año: valor catastral, impuestos, tasas, descuentos y vencimientos. Treasury te recuerda los vencimientos y suma el IPTU a los impuestos pagados.
iptu-registrar = Registrar el IPTU de { $ano }
iptu-registrar-outro = Registrar otro año
iptu-vazio = Ninguna boleta registrada. Registra el IPTU de este año para recordar los vencimientos y seguir el valor catastral.
iptu-editar = Editar
iptu-apagar = Eliminar
iptu-apagar-titulo = Eliminar el IPTU de { $ano }
iptu-apagar-texto = Se eliminan la boleta y los pagos marcados. Puedes deshacerlo desde el historial.
iptu-editar-titulo = IPTU de { $ano } · { $imovel }
iptu-novo-titulo = Registrar IPTU · { $imovel }

## Campos de la boleta

iptu-ano = Año (ejercicio)
iptu-cadastro = Número de registro / IQ
iptu-cadastro-ajuda = El número del inmueble en la municipalidad (inscrição imobiliária, indicação fiscal, SQL o índice cadastral), impreso en la boleta.
iptu-cidade = Municipio
iptu-area-terreno = Área del terreno (m²)
iptu-area-construida = Área construida (m²)
iptu-secao-venal = Valor catastral
iptu-valor-venal-terreno = Valor catastral del terreno
iptu-valor-venal-predial = Valor catastral de la construcción
iptu-valor-venal-total = Valor catastral total
iptu-secao-impostos = Impuestos, tasas y descuentos
iptu-imposto-territorial = Impuesto territorial
iptu-imposto-predial = Impuesto predial
iptu-total-impostos = Total de impuestos
iptu-taxa-iluminacao = Tasa de alumbrado público
iptu-taxa-lixo = Tasa de recolección de basura
iptu-outras-taxas = Otras tasas
iptu-desconto = Descuento
iptu-desconto-bom-pagador = Descuento por buen pagador
iptu-total-devido = Total a pagar
iptu-total-calcular = Calcular según la boleta
iptu-secao-pagamento = Pago
iptu-forma = Forma de pago
iptu-forma-indefinida = Aún no lo decidí
iptu-forma-unica = Pago único
iptu-forma-parcelado = En cuotas
iptu-cota-unica-vencimento = Vencimiento del pago único
iptu-cota-unica-valor = Monto del pago único
iptu-parcelas-quantidade = Número de cuotas
iptu-parcelas-primeiro = Primer vencimiento
iptu-parcelas-valor = Monto de cada cuota
iptu-parcelas-ajuda = Las cuotas vencen una por mes, a partir del primer vencimiento.
iptu-notas = Observaciones
iptu-salvar = Guardar
iptu-cancelar = Cancelar
iptu-salvo = IPTU de { $ano } guardado
iptu-erro-sem-total = Indica el total a pagar.

## Cuotas y pago

iptu-parcelas-titulo = Cuotas del IPTU de { $ano }

iptu-cota-unica = Pago único
iptu-parcela-n = Cuota { $n }
iptu-coluna-parcela = Cuota
iptu-coluna-vencimento = Vencimiento
iptu-coluna-valor = Monto
iptu-coluna-situacao = Estado
iptu-situacao-paga = Pagada el { $data }
iptu-situacao-aberta = Pendiente
iptu-situacao-vencida = Vencida
iptu-situacao-outra-forma = Otra forma
iptu-marcar-pago = Marcar como pagado
iptu-desfazer-pagamento = Deshacer el pago
iptu-pagamento-marcado = Pago marcado
iptu-pagamento-desfeito = Pago deshecho
iptu-pagar-titulo = Marcar { $parcela } como pagada
iptu-lancamentos-provaveis = Movimiento del pago
iptu-nenhum-lancamento = Ningún movimiento parecido cerca del vencimiento.
iptu-pago-sem-lancamento = Pagado sin movimiento en la app, el
iptu-sugestao = Parece pagado: { $descricao } el { $data }
iptu-confirmar-sugestao = Confirmar

## Cifras derivadas

iptu-aliquota-efetiva = Tasa efectiva
iptu-pago = Pagado
iptu-em-aberto = Pendiente
iptu-variacao-venal = Valor catastral { $variacao } frente a { $ano }
iptu-variacao-imposto = Impuestos { $variacao } frente a { $ano }
iptu-ipca-ano = IPCA de { $ano }: { $ipca }
iptu-mercado = Valor catastral = { $proporcao } del valor de mercado ({ $valor } el { $data })
iptu-cota-x-parcelas = Pago único de { $unica } × { $n } cuotas que suman { $parcelas }
iptu-historico-titulo = Historial del valor catastral
iptu-coluna-ano = Año

## Percepciones (núcleo)

iptu-percepcao-a-vista-cdi = Pagar en un solo pago ahorra { $economia }. El descuento equivale a { $taxa }% al mes, más que el CDI ({ $cdi }% al mes): conviene pagar al contado si el dinero ya está reservado.
iptu-percepcao-a-vista = Pagar en un solo pago ahorra { $economia }, un descuento equivalente a { $taxa }% al mes.
iptu-percepcao-parcelado = El pago único ahorra { $economia }, pero el descuento ({ $taxa }% al mes) rinde menos que el CDI ({ $cdi }% al mes): pagar en cuotas y dejar el dinero invertido sale un poco mejor.
iptu-percepcao-acima-ipca = El IPTU subió { $variacao }% frente al año anterior, por encima del IPCA de { $ano } ({ $ipca }%). Revisa en la boleta si cambió el área, el uso o la tabla de valores; la municipalidad acepta un pedido de revisión dentro del plazo de la boleta.
iptu-percepcao-venal-acima = El valor catastral está { $diferenca }% por encima del valor de mercado del inmueble en la app. Si esa valuación es correcta, puedes pedir a la municipalidad la revisión del valor catastral (impugnación dentro del plazo de la boleta).
iptu-percepcao-venal-abaixo = El valor catastral es { $proporcao }% del valor de mercado del inmueble. Suele estar por debajo del mercado; no lo uses como estimación del precio del inmueble.
iptu-percepcao-total-diferente = Impuestos + tasas − descuentos da { $calculado }, pero el total a pagar es { $total }. Revisa los montos de la boleta.

## Errores (núcleo)

iptu-imovel = inmueble
iptu-carne = boleta de IPTU
iptu-parcela = cuota del IPTU
iptu-erro-tipo = El IPTU solo aplica a inmuebles y terrenos.
iptu-erro-ano = Año del IPTU no válido.
iptu-erro-negativo = Los montos del IPTU no pueden ser negativos.
iptu-erro-area = Área no válida: usa metros cuadrados, como 250 o 72,5.
iptu-erro-parcela = La boleta tiene como máximo 24 cuotas.
iptu-erro-parcela-valor = Cada cuota necesita un monto mayor que cero.
iptu-erro-parcela-repetida = Hay dos cuotas con el mismo número.
iptu-erro-lancamento-usado = Ese movimiento ya paga otra cuota del IPTU.
iptu-erro-lancamento-saida = El pago debe ser una salida (monto negativo).
iptu-erro-pagamento = Elige el movimiento del pago o indica la fecha.

## Historial de cambios (núcleo)

iptu-historico-salvo = IPTU de { $ano } registrado
iptu-historico-apagado = IPTU eliminado
iptu-historico-pago = Cuota del IPTU pagada
iptu-historico-desfeito = Pago del IPTU deshecho

## Recordatorios (núcleo)

iptu-quando-hoje = hoy
iptu-quando-amanha = mañana
iptu-quando-dias = en { $dias } días
iptu-alerta-titulo = IPTU { $ano } · { $imovel }: { $parcela } vence { $quando }
iptu-alerta-titulo-vencido = IPTU { $ano } · { $imovel }: { $parcela } está vencida
iptu-alerta-corpo = { $valor } con vencimiento el { $data }. Cuando pagues, márcalo en Cartera › Bienes › { $imovel } › IPTU.
iptu-alerta-corpo-decidir = El pago único de { $valor } vence el { $data }. Mira en Cartera › Bienes › { $imovel } › IPTU si conviene pagar al contado o en cuotas.
iptu-alerta-falta-titulo = IPTU { $ano } · { $imovel }: registra la boleta
iptu-alerta-falta-corpo = La boleta de { $ano } suele llegar en esta época. Regístrala en Cartera › Bienes › { $imovel } › IPTU para recordar los vencimientos y seguir el valor catastral.

## Impuestos pagados (núcleo)

iptu-impostos-nota = IPTU según las boletas registradas en Bienes (lo que vence en el período). Registra el pago en una categoría de IPTU para usar el monto realmente pagado.
iptu-impostos-linha = IPTU según la boleta · { $imovel }
iptu-impostos-valor = Valor catastral
