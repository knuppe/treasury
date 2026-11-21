# Asistente de la bóveda de demostración (docs/demo.md, docs/i18n-design.md). Revisión: borrador
# por IA, sin revisión nativa. El argumento valor es un elemento (el valor en dinero) que la
# pantalla coloca en su lugar.

## Valores iniciales y pasos

demo-padrao-nome = Demostración
demo-padrao-familia = Familia Ejemplo
demo-passo-familia = Familia
demo-passo-renda = Ingresos
demo-passo-moradia = Vivienda y gastos
demo-passo-contas = Cuentas e inversiones
demo-passo-dividas = Deudas y creación
demo-titulo = Bóveda de demostración
demo-descricao = Cuéntanos cómo es la familia y la app crea una bóveda separada, con un historial de ejemplo para probar con libertad.
demo-passo = Paso { $n } de { $total }: { $passo }
demo-progresso = Progreso del asistente

## Lo que falta para seguir

demo-erro-nome = Ponle un nombre a la bóveda.
demo-erro-familia = Ingresa el nombre de la familia.
demo-erro-pessoa = Ingresa el nombre de cada persona.
demo-erro-nomes-iguais = Cada persona necesita un nombre distinto.
demo-erro-renda = Ingresa los ingresos de al menos un adulto.
demo-erro-financiamento = Ingresa la cuota del crédito hipotecario.
demo-erro-banco = Ingresa el banco de la cuenta principal.
demo-erro-carro = Ingresa la cuota del auto y cuántas faltan.
demo-criado = Bóveda "{ $nome }" creada
demo-criado-dica = { $n } movimientos desde el { $data } hasta hoy. Para volver a tu bóveda, usa Bóvedas.

## Paso 1: familia

demo-nome-cofre = Nombre de la bóveda
demo-nome-cofre-dica = Aparece en la lista de bóvedas.
demo-nome-familia = Nombre de la familia
demo-adultos = Adultos con ingresos
demo-um = Uno
demo-dois = Dos
demo-nome-adulto = Nombre del adulto { $n }
demo-filhos = Hijos
demo-nenhum = Ninguno
demo-nome-filho = Nombre del hijo { $n }
demo-historico = Historial
demo-historico-dica = Cuántos meses de movimientos, hasta hoy.
demo-meses-historico = Meses de historial
demo-meses = { $n } meses

## Paso 2: ingresos

demo-renda-de = Ingreso neto de { $nome }
demo-adulto-n = adulto { $n }
demo-renda-dica = Por mes, lo que llega a la cuenta.
demo-renda-variavel = Ingreso variable
demo-renda-variavel-dica = Independiente, comisiones: cambia de un mes a otro, sin 13º.
demo-outras-rendas = Otros ingresos
demo-outras-rendas-dica = Por mes: alquiler cobrado, por ejemplo.
demo-renda-total = Ingreso total por mes: { $valor }. En los próximos pasos, los valores se sugieren a partir de él; ajústalos a tu gusto.

## Paso 3: vivienda y gastos

demo-moradia = Vivienda
demo-aluguel = Alquiler
demo-financiamento = Crédito hipotecario
demo-propria = Propia pagada
demo-sugerir = Sugerir según los ingresos
demo-parcela-financiamento = Cuota del crédito hipotecario
demo-condominio = Gastos comunes
demo-contas-casa = Cuentas de la casa
demo-contas-casa-dica = Luz, agua, internet y celular.
demo-mercado = Supermercado y panadería
demo-transporte = Transporte
demo-carro = Auto
demo-aplicativo = Aplicación
demo-onibus = Autobús y metro
demo-nao-tem = No tiene
demo-combustivel = Combustible y estacionamiento
demo-escola = Escuela, por hijo
demo-saude = Salud
demo-saude-dica = Seguro médico y farmacia.
demo-lazer = Ocio
demo-lazer-dica = Restaurantes, delivery, cine y paseos.
demo-assinaturas = Suscripciones
demo-assinaturas-dica = Streaming y música.
demo-compras = Compras
demo-compras-dica = Ropa, casa y electrónicos.

## Paso 4: cuentas e inversiones

demo-banco = Banco de la cuenta principal
demo-cartao = Tarjeta de crédito
demo-cartao-dica = Las compras del día a día van con la tarjeta y los estados de cuenta se pagan al vencimiento.
demo-limite = Límite de la tarjeta
demo-investimentos = Inversiones
demo-reserva = Fondo de emergencia hoy
demo-reserva-mes = Al fondo de emergencia, por mes
demo-acoes = Acciones (al inicio del historial)
demo-fiis = Fondos inmobiliarios (al inicio)
demo-tesouro = Tesouro Direto (al inicio)
demo-aporte = Aporte a la corredora, por mes
demo-previdencia = Plan de pensión VGBL (al inicio)
demo-aporte-previdencia = Aporte al plan de pensión, por mes
demo-cotacoes-ficticias = Acciones y fondos con tickers reales, pero cotizaciones ficticias (cotización manual): el ejemplo no mezcla precios reales con operaciones inventadas.

## Paso 5: deudas y creación

demo-financiamento-carro = Crédito del auto
demo-financiamento-carro-dica = Se convierte en un contrato de deuda, con las cuotas pagadas en el historial.
demo-parcela = Cuota
demo-parcelas-faltam = Cuotas que faltan
demo-financiamento-moradia = El crédito hipotecario también se convierte en un contrato de deuda.
demo-resumo = Resumen del mes
demo-entra = Entra
demo-sai = Sale (gastos)
demo-investe = Invierte
demo-vermelho = Los gastos y aportes superan los ingresos: el ejemplo va a mostrar meses en rojo.
demo-nivel = Nivel de la interfaz en la bóveda de demostración
demo-nivel-rotulo = Nivel de la interfaz
demo-troca = La bóveda actual se cierra (con copia de seguridad) y se abre la de demostración. Para volver, usa Bóvedas: tu bóveda sigue igual.
demo-criar = Crear bóveda de demostración

## Perfil y país (primer paso)

demo-passo-perfil = Perfil
demo-pais = País de la familia
demo-pais-dica = Define la moneda, las categorías, los impuestos y los activos del ejemplo.
demo-perfil = Perfil de la familia
demo-perfil-cofre = Demostración · { $perfil }
demo-perfil-confortavel = Clase media holgada
demo-perfil-confortavel-dica = Pareja con un hijo, hipoteca, auto, fondo de emergencia e inversiones.
demo-perfil-baixa-renda = Bajos ingresos
demo-perfil-baixa-renda-dica = Pareja con dos hijos, alquiler, transporte público, ingresos justos, changas y una ayuda del gobierno.
demo-perfil-dificuldade = Familia con dificultades
demo-perfil-dificuldade-dica = Gasta más de lo que gana: descubierto, saldo de la tarjeta, un préstamo y facturas atrasadas.
demo-perfil-sozinho = Joven que vive solo
demo-perfil-sozinho-dica = Una persona, alquiler chico, apps de transporte y las primeras inversiones.
demo-perfil-aposentado = Jubilada
demo-perfil-aposentado-dica = Jubilación, casa propia, alquiler cobrado, seguro médico y ahorros.
demo-perfil-custom = Armar desde cero
demo-perfil-custom-dica = Cuentas cómo es la familia en los próximos pasos.
demo-perfil-nota = Los montos vienen en la moneda del país y puedes cambiar todo en los próximos pasos.

## Ingresos, deudas y creación

demo-aposentado = Jubilado
demo-aposentado-dica = El ingreso es la jubilación.
demo-beneficio = Ayuda del gobierno
demo-beneficio-dica = Por mes.
demo-emprestimo = Préstamo personal
demo-emprestimo-dica = Se vuelve un contrato en Deudas, con las cuotas pagadas desde la cuenta.
demo-erro-emprestimo = Indica la cuota del préstamo y cuántas faltan.
demo-cheque-especial = Descubierto
demo-cheque-especial-dica = La cuenta puede quedar en negativo; el mes que cierra en rojo, vienen intereses al comienzo del siguiente.
demo-contas-atrasadas = Algunas facturas pagadas con atraso
demo-contas-atrasadas-dica = Una de cada tres facturas de la casa sale diez días tarde, con recargo e intereses.
demo-rotativo = Paga solo parte del resumen
demo-rotativo-dica = Paga el 70% de cada resumen; el resto queda como saldo de la tarjeta y se cobran intereses.
demo-sem-senha = La bóveda de demostración se abre sin contraseña. Cualquiera con la app puede abrirla: usa solo datos ficticios, nunca datos reales.

## Creación rápida

demo-rapido-descricao = Elige el país y el perfil: la bóveda de demostración se crea y se abre enseguida, sin contraseña.
demo-rapido-criar = Crear y abrir
demo-rapido-personalizar = Personalizar antes de crear
demo-idade = Edad
demo-idade-de = Edad de { $nome }
demo-idade-auto = Según el perfil
demo-sexo-de = Sexo de { $nome }
demo-erro-idade = Revisa las edades: adultos de 18 a 100 años e hijos de 0 a 17.
