# Pantalla 04 · Movimientos: tabla, filtros, acciones en lote, menú de la fila, diálogo del
# movimiento, división, recurrencias, transferencias entre cuentas y vínculo con operaciones
# (docs/i18n-design.md). Revisión: borrador por IA, sin revisión de un hablante nativo.

## Etiquetas de campo que se repiten en la pantalla

lancamentos-campo-data = Fecha
lancamentos-campo-descricao = Descripción
lancamentos-campo-conta = Cuenta
lancamentos-campo-categoria = Categoría
lancamentos-campo-projeto = Proyecto
lancamentos-campo-membro = Miembro
lancamentos-campo-status = Estado
lancamentos-campo-valor = Monto
lancamentos-campo-tipo = Tipo
lancamentos-campo-tags = Etiquetas
lancamentos-campo-notas = Notas
lancamentos-sem-descricao = sin descripción
lancamentos-sem-descricao-parenteses = (sin descripción)
lancamentos-outra-conta = otra cuenta
lancamentos-contrato = contrato

## Encabezado, pestañas y barra de comandos

lancamentos-descricao = Ingresos, gastos y transferencias. Filtra, divide, categoriza en lote, importa extractos y enseña reglas.
lancamentos-importar = Importar
lancamentos-transferencia = Transferencia
lancamentos-cambio = Cambio de moneda
lancamentos-novo = Nuevo movimiento
lancamentos-abas = Secciones de movimientos
lancamentos-aba-lancamentos = Movimientos
lancamentos-aba-recorrencias = Recurrencias
lancamentos-aba-regras = Reglas
lancamentos-aba-importacoes = Importaciones
lancamentos-aba-caixa = Bandeja de entrada
lancamentos-aba-email = Correo

## Comandos (las palabras de búsqueda van separadas por coma)

lancamentos-cmd-novo-busca = gasto,ingreso,compra
lancamentos-cmd-receita = Nuevo ingreso
lancamentos-cmd-receita-busca = entrada,salario
lancamentos-cmd-transferencia = Nueva transferencia entre cuentas
lancamentos-cmd-transferencia-busca = transferir,pix,aporte,cambio
lancamentos-cmd-parcelado = Compra en cuotas con tarjeta
lancamentos-cmd-parcelado-busca = cuotas,tarjeta
lancamentos-cmd-importar = Importar extracto (OFX/CSV)
lancamentos-cmd-importar-busca = ofx,csv,banco,tarjeta
lancamentos-cmd-buscar-busca = buscar,filtrar
lancamentos-cmd-sem-categoria = Ver movimientos sin categoría
lancamentos-cmd-sem-categoria-busca = categorizar,pendientes
lancamentos-cmd-recorrencia-busca = suscripción,alquiler,mensual

## Avisos de las acciones y "Deshacer"

lancamentos-transferencia-desligada = Transferencia desvinculada
lancamentos-transferencia-desligada-descricao = Los dos lados quedaron sueltos: vincula cada uno con su par correcto o cambia el tipo a ingreso o gasto.
lancamentos-erro-desligar = No se pudo desvincular
lancamentos-erro-sugerir-regra = No se pudo sugerir una regla
lancamentos-status-rotulo = Estado: { $status }
lancamentos-n-alterados = { $n ->
        [one] { $n } movimiento modificado.
       *[other] { $n } movimientos modificados.
    }
lancamentos-erro-status = No se pudo cambiar el estado
lancamentos-n-marcados-toast = { $n ->
        [one] { $n } movimiento marcado
       *[other] { $n } movimientos marcados
    }
lancamentos-mesma-descricao-toast = Misma descripción (“{ $descricao }”), en todo el período. Categoriza o cambia en lote en la barra de arriba.
lancamentos-mesma-descricao-lista = Misma descripción ({ $descricoes }), en todo el período.
lancamentos-erro-marcar = No se pudo marcar
lancamentos-transferencia-ligada = Transferencia vinculada entre las cuentas
lancamentos-transferencia-ligada-descricao = Los dos lados quedaron vinculados.
lancamentos-erro-ligar = No se pudo vincular
lancamentos-feito = Hecho
lancamentos-desfeito = Deshecho
lancamentos-erro-desfazer = No se pudo deshacer
lancamentos-desfazer = Deshacer
lancamentos-dispensar = Descartar
lancamentos-apagados = Movimientos eliminados

## Por qué dos marcados no se vinculan como transferencia

lancamentos-par-mesma-pagina = Marca los dos en la misma página de la lista.
lancamentos-par-ja-ligada = Uno de ellos ya está vinculado a otra cuenta: desvincúlalo antes (menú de la fila).
lancamentos-par-sentidos = Para vincularlos, uno tiene que ser una salida y el otro una entrada.
lancamentos-par-mesma-conta = Los dos están en la misma cuenta: la transferencia vincula cuentas diferentes.
lancamentos-par-valores = Los montos son diferentes: en la misma moneda, lo que sale tiene que ser igual a lo que entra.
lancamentos-par-tipos = Solo se vinculan ingresos, gastos y transferencias sueltos (sin división ni cuotas).

## Tabla y paginación

lancamentos-atualizando = Actualizando
lancamentos-zero = 0 movimientos
lancamentos-faixa = { $de }–{ $ate } de { $total }
lancamentos-totais = Totales del filtro
lancamentos-entradas = Entradas
lancamentos-saidas = Salidas
lancamentos-resultado = Resultado
lancamentos-totais-nota = Totales de todo el filtro, no solo de la página
lancamentos-carregando = Cargando movimientos
lancamentos-vazio-kicker = Ningún movimiento
lancamentos-vazio-titulo = Empieza por el extracto del banco
lancamentos-vazio-descricao = Importa un OFX o CSV, o regístralos a mano. Las transferencias entre tus propias cuentas no cuentan como gasto.
lancamentos-importar-extrato = Importar extracto
lancamentos-paginas = Páginas
lancamentos-anterior = ← Anterior
lancamentos-proxima = Siguiente →
lancamentos-pagina-de = página { $pagina } de { $paginas }
lancamentos-atalho-novo = nuevo
lancamentos-atalho-buscar = buscar
lancamentos-atalho-importar = importar
lancamentos-atalho-navegar = navegar
lancamentos-atalho-marcar = marcar
lancamentos-atalho-marcar-faixa = marcar rango
lancamentos-atalho-categoria = categoría
lancamentos-atalho-abrir = abrir
lancamentos-apagar-1 = Eliminar 1 movimiento
lancamentos-apagar-1-descricao = “{ $descricao }”. Las transferencias se llevan el otro lado. Los movimientos conciliados no se pueden eliminar. Puedes deshacerlo desde el historial.

lancamentos-op-carteira = cartera
lancamentos-op-cotas = cuotas
lancamentos-op-meta = meta
lancamentos-ligada-a = Vinculada a { $conta }
lancamentos-ligada-perna = Vinculada al otro lado en { $conta }
lancamentos-solta-dica = Falta el otro lado. Marca la salida y la entrada y vincúlalas, o usa el menú de la fila.
lancamentos-transferencia-solta = Transferencia suelta
lancamentos-solta = suelta
lancamentos-dividido-em = Dividido en { $n }
lancamentos-categoria-de = Categoría de { $descricao }
lancamentos-erro-categoria = No se pudo cambiar la categoría
lancamentos-selecionar-pagina = Seleccionar todos en esta página
lancamentos-selecionar = Seleccionar { $descricao }
lancamentos-vencimento-fatura = Vencimiento de la tarjeta (caja)
lancamentos-fat = venc. { $data }
lancamentos-parcela-de = Cuota { $numero } de { $contrato }
lancamentos-parc = cuota { $numero }
lancamentos-vinculado = Vinculado: { $operacao }
lancamentos-recorrencia-dica = Generado por una recurrencia
lancamentos-recorrente = Recurrente
lancamentos-n-anexos = { $n ->
        [one] { $n } adjunto
       *[other] { $n } adjuntos
    }
lancamentos-acoes-marcados = Acciones para { $n } movimientos marcados
lancamentos-vazio-filtro = Ningún movimiento con estos filtros.
lancamentos-acoes-de = Acciones para { $descricao }
lancamentos-sem-descricao-acoes = movimiento sin descripción

## Filtros

lancamentos-filtros = Filtros de movimientos
lancamentos-buscar = Buscar movimientos
lancamentos-buscar-placeholder = Descripción, beneficiario o notas
lancamentos-filtro-receita = ingreso
lancamentos-filtro-arquivada = archivada
lancamentos-periodo = Período
lancamentos-periodo-aria = Período: { $periodo }
lancamentos-contas = Cuentas
lancamentos-todas = Todas
lancamentos-categorias = Categorías
lancamentos-membros = Miembros
lancamentos-projetos = Proyectos
lancamentos-faixa-valor = Rango de montos
lancamentos-base-datas = Base de fechas
lancamentos-competencia = Devengo
lancamentos-caixa = Caja
lancamentos-so-sem-categoria = Solo sin categoría
lancamentos-fatura-selecionada = Resumen de tarjeta seleccionado
lancamentos-lote-importacao = Lote de importación
lancamentos-limpar-filtros = Borrar filtros
lancamentos-tirar-filtro = Quitar filtro: { $filtro }
lancamentos-de = Desde
lancamentos-ate = Hasta
lancamentos-aplicar-periodo = Aplicar período
lancamentos-valor-absoluto = Monto absoluto: vale para entradas y salidas.
lancamentos-minimo = Mínimo
lancamentos-maximo = Máximo
lancamentos-limpar = Borrar
lancamentos-aplicar = Aplicar

lancamentos-periodo-tudo = Todo el período
lancamentos-periodo-este-mes = Este mes
lancamentos-periodo-mes-passado = Mes pasado
lancamentos-periodo-3-meses = Últimos 3 meses
lancamentos-periodo-12-meses = Últimos 12 meses
lancamentos-periodo-este-ano = Este año
lancamentos-periodo-personalizado = Personalizado

## Menú de la fila y menú del lote

lancamentos-marcar-como = Marcar como { $status }
lancamentos-menu-abrir = Abrir y editar
lancamentos-menu-categorizar = Categorizar
lancamentos-menu-ver-divisao = Ver la división
lancamentos-menu-desmarcar = Desmarcar
lancamentos-menu-marcar-lote = Marcar para acción en lote
lancamentos-menu-mesma-descricao = Marcar los de la misma descripción
lancamentos-menu-projeto = Proyecto: { $projeto }…
lancamentos-menu-por-projeto = Agregar a un proyecto…
lancamentos-menu-transferencia-para = Transferencia a otra cuenta…
lancamentos-menu-transferencia-de = Transferencia desde otra cuenta…
lancamentos-menu-desligar-de = Desvincular de { $conta }
lancamentos-desligar-transferencia = Desvincular la transferencia
lancamentos-menu-pagamento-fatura = Pago de tarjeta…
lancamentos-menu-parcela = Cuota { $numero } de { $contrato }…
lancamentos-menu-pagamento-parcela = Pago de cuota de contrato…
lancamentos-menu-vinculo = Vínculo: { $operacao }…
lancamentos-menu-vincular = Vincular a una operación…
lancamentos-menu-abrir-conta = Abrir la cuenta { $conta }
lancamentos-menu-so-conta = Solo movimientos de esta cuenta
lancamentos-criar-regra-deste = Crear regla a partir de este
lancamentos-menu-apagar = Eliminar…
lancamentos-conciliado-nao-apaga = Los movimientos conciliados no se pueden eliminar.
lancamentos-menu-n-marcados = { $n ->
        [one] { $n } movimiento marcado
       *[other] { $n } movimientos marcados
    }
lancamentos-menu-categorizar-marcados = Categorizar marcados…
lancamentos-menu-membro = Miembro…
lancamentos-menu-tags = Etiquetas…
lancamentos-menu-ligar-par = Vincular los dos como transferencia
lancamentos-menu-marcar-todos = Marcar los { $n } del filtro
lancamentos-criar-regra = Crear regla
lancamentos-menu-apagar-n = { $n ->
        [one] Eliminar { $n } movimiento…
       *[other] Eliminar { $n } movimientos…
    }

## Barra de acciones en lote

lancamentos-acoes-lote = Acciones en los movimientos marcados
lancamentos-n-marcado = { $n ->
        [one] { $n } marcado
       *[other] { $n } marcados
    }
lancamentos-mesma-descricao = Misma descripción
lancamentos-todos-do-filtro = Los { $n } del filtro
lancamentos-categorizar-marcados = Categorizar marcados
lancamentos-tirar-categoria = Quitar la categoría
lancamentos-categoria-definida = Categoría asignada
lancamentos-categoria-removida = Categoría quitada
lancamentos-erro-categorizar = No se pudo categorizar
lancamentos-mudar-status-marcados = Cambiar el estado de los marcados
lancamentos-projeto-reticencias = Proyecto…
lancamentos-membro-marcados = Miembro de los marcados
lancamentos-sem-membro = Sin miembro
lancamentos-membro-definido = Miembro: { $nome }
lancamentos-membro-removido = Miembro quitado
lancamentos-erro-membro = No se pudo cambiar el miembro
lancamentos-tags-marcados = Etiquetas de los marcados
lancamentos-tag = Etiqueta
lancamentos-nova-tag = Nueva etiqueta
lancamentos-por = Poner
lancamentos-tags-existentes = Etiquetas existentes
lancamentos-por-tag = Poner la etiqueta { $tag }
lancamentos-tirar-tag = Quitar la etiqueta { $tag }
lancamentos-tirar = Quitar
lancamentos-tag-posta = Etiqueta “{ $tag }” agregada
lancamentos-tag-tirada = Etiqueta “{ $tag }” quitada
lancamentos-erro-tags = No se pudieron cambiar las etiquetas
lancamentos-ligar-como-transferencia = Vincular como transferencia
lancamentos-vincular-operacao = Vincular a una operación…
lancamentos-apagar-n = { $n ->
        [one] Eliminar { $n } movimiento
       *[other] Eliminar { $n } movimientos
    }
lancamentos-apagar-n-descricao = Las transferencias se llevan el otro lado. Los movimientos conciliados no se pueden eliminar. Puedes deshacerlo desde el historial.
lancamentos-tags-exemplo = viaje, reembolsable…

## Diálogo del movimiento

lancamentos-membro-da-conta = De la cuenta ({ $nome })
lancamentos-familia = Familia
lancamentos-erro-conta = Elige la cuenta.
lancamentos-erro-data = Ingresa una fecha válida (dd/mm/aaaa).
lancamentos-erro-valor = Ingresa el monto (mayor que cero).
lancamentos-erro-destino = Elige la cuenta de destino.
lancamentos-erro-mesma-conta = La cuenta de origen y la de destino tienen que ser diferentes.
lancamentos-erro-valor-destino = Ingresa cuánto entra en { $conta } ({ $moeda }).
lancamentos-erro-descricao-compra = Ingresa la descripción de la compra.
lancamentos-erro-parcelas = El número de cuotas va de 1 a 120.
lancamentos-erro-valor-ligado = Ingresa cuánto entró (o salió) en { $conta } ({ $moeda }).
lancamentos-erro-duas-partes = Una división necesita al menos dos partes.
lancamentos-erro-valor-parte = Ingresa el monto de cada parte.
lancamentos-erro-soma-partes = La suma de las partes tiene que ser igual al total.
lancamentos-tipo-registrado = { $tipo ->
        [investment] Aporte registrado
        [card_payment] Pago de tarjeta registrado
       *[other] Transferencia registrada
    }
lancamentos-parcelada-registrada = Compra en cuotas registrada
lancamentos-criado = Movimiento creado
lancamentos-atualizado = Movimiento actualizado
lancamentos-editar-tipo = Editar { $tipo }
lancamentos-editar = Editar movimiento
lancamentos-salvando = Guardando…
lancamentos-salvar-alteracoes = Guardar cambios
lancamentos-lancar = Registrar
lancamentos-tipo-lancamento = Tipo de movimiento
lancamentos-parcelado-cartao = En cuotas con tarjeta
lancamentos-de-origem = Desde (origen)
lancamentos-para-destino = Hacia (destino)
lancamentos-escolha-conta = Elige la cuenta
lancamentos-valor-sai = Monto que sale ({ $moeda })
lancamentos-valor-entra = Monto que entra ({ $moeda })
lancamentos-cambio-implicito = Tipo de cambio implícito: 1 { $moeda } = { NUMBER($taxa, maximumFractionDigits: 4) } { $origem }
lancamentos-moedas-diferentes = Cuentas en monedas diferentes: ingresa el monto recibido.
lancamentos-descricao-transferencia-dica = Opcional: la app escribe “Transferencia a …”.
lancamentos-cartao = Tarjeta
lancamentos-data-compra = Fecha de la compra
lancamentos-exemplo-receita = Salario, reembolso…
lancamentos-exemplo-despesa = Supermercado, farmacia…
lancamentos-valor-total-moeda = Monto total ({ $moeda })
lancamentos-valor-moeda = Monto ({ $moeda })
lancamentos-parcelas-previa = { $n }× de { $valor }
lancamentos-parcelas-previa-primeira = { $n }× de { $valor } (1.ª de { $primeira })
lancamentos-parcelas = Cuotas
lancamentos-parcelas-dica = Las cuotas futuras quedan como previstas.
lancamentos-como-transferencia-dica = Dinero movido a otra cuenta (de la familia o de terceros). Con categoría, cuenta en el flujo de caja por ella; sin categoría, queda fuera de ingresos y gastos.
lancamentos-tipo-do-lancamento = Tipo del movimiento
lancamentos-para-conta-familia = A la cuenta de la familia
lancamentos-da-conta-familia = Desde la cuenta de la familia
lancamentos-ligar-dica = Opcional: lo vincula con el lado que ya existe en esa cuenta (mismo monto, fecha cercana) o crea ese lado.
lancamentos-nenhuma-terceiros = Ninguna (terceros)
lancamentos-valor-em = Monto en { $conta } ({ $moeda })
lancamentos-destinatario = Destinatario
lancamentos-favorecido = Beneficiario
lancamentos-destinatario-dica = Opcional: a quién fue (o de quién vino), cuando no es una cuenta de la familia.
lancamentos-favorecido-dica = Opcional: quién recibió o pagó.
lancamentos-projeto-dica = Obra, remodelación, fiesta o viaje en 05 · Presupuesto › Proyectos.
lancamentos-dividir = Dividir en categorías o miembros
lancamentos-anexos = Adjuntos
lancamentos-anexos-dica = Comprobantes y facturas (PDF o imagen, hasta 20 MB). Se envían después de registrar.
lancamentos-anexo-salvar = Guardar { $nome }
lancamentos-anexo-salvo = Adjunto guardado
lancamentos-nenhum-anexo = Ningún comprobante adjunto.
lancamentos-anexar = Adjuntar comprobante
lancamentos-apagar-item = Eliminar { $nome }
lancamentos-editar-item = Editar { $nome }

## División

lancamentos-divisao = División
lancamentos-categoria-parte = Categoría de la parte { $n }
lancamentos-membro-parte = Miembro de la parte { $n }
lancamentos-valor-parte = Monto de la parte { $n }
lancamentos-tirar-parte = Quitar la parte { $n }
lancamentos-adicionar-parte = Agregar parte
lancamentos-restante-ultima = Poner el resto en la última
lancamentos-divisao-soma = Suma { $soma } de { $total }
lancamentos-divisao-faltam = faltan { $valor }
lancamentos-divisao-sobram = sobran { $valor }
lancamentos-divisao-minimo = usa al menos dos partes
lancamentos-divisao-confere = cuadra ✓

## Recurrencias (días de la semana: abreviados de domingo a sábado; en las casillas, de lunes a domingo)

lancamentos-dias-semana-abrev = dom.,lun.,mar.,mié.,jue.,vie.,sáb.
lancamentos-dias-semana-caixas = lun,mar,mié,jue,vie,sáb,dom
lancamentos-erro-descricao = Ingresa la descripción.
lancamentos-erro-valor-simples = Ingresa el monto.
lancamentos-erro-inicio = Ingresa la fecha de inicio.
lancamentos-recorrencia-atualizada = Recurrencia actualizada
lancamentos-recorrencia-atualizada-descricao = Los previstos futuros se volvieron a generar.
lancamentos-recorrencia-criada = Recurrencia creada
lancamentos-recorrencia-criada-descricao = Los movimientos previstos de los próximos 2 meses ya aparecen.
lancamentos-editar-recorrencia = Editar recurrencia
lancamentos-nova-recorrencia = Nueva recurrencia
lancamentos-criar-recorrencia = Crear recurrencia
lancamentos-recorrencia-exemplo = Alquiler, salario, gimnasio…
lancamentos-repeticao = Repetición
lancamentos-frequencia = Frecuencia
lancamentos-freq-diaria = Diaria
lancamentos-freq-semanal = Semanal
lancamentos-freq-mensal = Mensual
lancamentos-freq-anual = Anual
lancamentos-a-cada = Cada
lancamentos-dia-do-mes = Día del mes
lancamentos-dia-do-mes-dica = 31 = último día en los meses cortos.
lancamentos-mes = Mes
lancamentos-dias-semana = Días de la semana
lancamentos-inicio = Inicio
lancamentos-fim = Fin (opcional)
lancamentos-ocorrencias = Repeticiones (opcional)
lancamentos-previa-datas = Vista previa de las próximas fechas
lancamentos-previa = Vista previa
lancamentos-previa-vazia = Completa la repetición para ver las fechas.
lancamentos-col-regra = Regla
lancamentos-col-proximas = Próximas
lancamentos-pausada = En pausa
lancamentos-col-por-mes = Por mes
lancamentos-acoes = Acciones
lancamentos-recorrencia-pausada = Recurrencia en pausa
lancamentos-recorrencia-retomada = Recurrencia reanudada
lancamentos-erro-alterar = No se pudo cambiar
lancamentos-pausar = Pausar
lancamentos-retomar = Reanudar
lancamentos-comprometido = Comprometido por mes ({ $moeda })
lancamentos-recorrencias-vazio-kicker = Ninguna recurrencia
lancamentos-recorrencias-vazio-titulo = Registra salario, alquiler y suscripciones
lancamentos-recorrencias-vazio-descricao = Las recurrencias generan movimientos previstos para los próximos dos meses, que entran en el saldo previsto y en el presupuesto.
lancamentos-apagar-recorrencia = Eliminar recurrencia
lancamentos-apagar-recorrencia-descricao = “{ $descricao }” deja de generar movimientos. Los ya efectuados se quedan, sin vínculo. Puedes deshacerlo desde el historial.
lancamentos-recorrencia-apagada = Recurrencia eliminada
lancamentos-previstos-removidos = { $n ->
        [one] { $n } previsto eliminado.
       *[other] { $n } previstos eliminados.
    }
lancamentos-manter-previstos = Mantener los movimientos previstos ya generados

## Vincular a otra cuenta (transferencia o pago de tarjeta)

lancamentos-pagamento-fatura = Pago de tarjeta
lancamentos-transferencia-para = Transferencia a otra cuenta
lancamentos-transferencia-de = Transferencia desde otra cuenta
lancamentos-fatura-ligada = Pago de tarjeta vinculado a la tarjeta
lancamentos-ligado-a = Vinculado a “{ $descricao }” en { $conta }.
lancamentos-criado-em = Se creó el movimiento en { $conta }.
lancamentos-ligar-cartao = Vincular a la tarjeta
lancamentos-ligar-contas = Vincular las cuentas
lancamentos-saiu-em = salió de { $conta }
lancamentos-entrou-em = entró en { $conta }
lancamentos-saiu-de = salió de { $conta }
lancamentos-nenhum-cartao = No hay ninguna tarjeta registrada. Registra la tarjeta en 03 · Cuentas para vincular el pago a su resumen.
lancamentos-nenhuma-outra-conta = No hay otra cuenta activa.
lancamentos-para-a-conta = A la cuenta
lancamentos-da-conta = Desde la cuenta
lancamentos-o-que-acontece = Qué va a pasar
lancamentos-procurando = Buscando el movimiento correspondiente en { $conta }…
lancamentos-achei-entrada = Encontré la entrada en { $conta }: { $data } · “{ $descricao }”. Las dos quedan vinculadas.
lancamentos-achei-saida = Encontré la salida en { $conta }: { $data } · “{ $descricao }”. Las dos quedan vinculadas.
lancamentos-crio-entrada = No hay un movimiento de { $valor } en { $conta } cerca del { $data }: creo la entrada allí, con la misma fecha y descripción.
lancamentos-crio-saida = No hay un movimiento de { $valor } en { $conta } cerca del { $data }: creo la salida allí, con la misma fecha y descripción.
lancamentos-informe-entrou = Ingresa cuánto entró en { $conta }.
lancamentos-informe-saiu = Ingresa cuánto salió de { $conta }.
lancamentos-vira-fatura = Pasa a ser un pago de tarjeta: sale de los gastos (las compras con la tarjeta ya cuentan).
lancamentos-quita-fatura = Paga el resumen que vence el { $data }.
lancamentos-vira-aporte = Pasa a ser un aporte o rescate de la corredora: queda fuera de ingresos y gastos.
lancamentos-vira-transferencia = Pasa a ser una transferencia entre cuentas de la familia: queda fuera de ingresos y gastos.

## Transferencias sugeridas (pares que llegaron sueltos)

lancamentos-aporte-ou-resgate = Aporte o rescate
lancamentos-ligar-par-rotulo = Vincular { $de } → { $para } el { $data }
lancamentos-sempre-ligar = vincular siempre
lancamentos-sai-em = Sale el { $data }: { $descricao }
lancamentos-entra-em = Entra el { $data }: { $descricao }
lancamentos-nao-e = No lo es
lancamentos-pares-aviso = { $n ->
        [one] 1 par de movimientos parece una transferencia entre tus cuentas
       *[other] { $n } pares de movimientos parecen transferencias entre tus cuentas
    }
lancamentos-pares-aviso-nota = · sin vincularlos, la salida cuenta como gasto y la entrada como ingreso otra vez.
lancamentos-revisar = Revisar
lancamentos-revisao-titulo = Transferencias entre tus cuentas
lancamentos-revisao-descricao = Pares del mismo monto que salen de una cuenta y entran en otra en hasta 4 días. Al vincularlos, pasan a ser transferencias: salen de ingresos y gastos (informes, presupuesto, salud financiera) y pierden la categoría.
lancamentos-ligar-sozinho = Vincular automáticamente los próximos entre estas cuentas
lancamentos-ligar-sozinho-descricao = Al importar extractos, los pares así entran ya vinculados.
lancamentos-agora-nao = Ahora no
lancamentos-n-transferencias-ligadas = { $n ->
        [one] 1 transferencia vinculada
       *[other] { $n } transferencias vinculadas
    }
lancamentos-ligar-n-pares = { $n ->
        [one] Vincular 1 par
       *[other] Vincular { $n } pares
    }
lancamentos-pares-sugeridos = Pares sugeridos

## Vincular a una operación (cartera, cuotas de capital, meta)

lancamentos-operacao-carteira = Cartera
lancamentos-operacao-cotas = Cuotas de capital
lancamentos-operacao-meta = Meta
lancamentos-bloqueio-transferencia = Es una transferencia entre tus cuentas: desvincula la transferencia para vincularlo.
lancamentos-bloqueio-previsto = El movimiento todavía está previsto: efectúalo antes de vincularlo.
lancamentos-bloqueio-dividido = Los movimientos divididos no se vinculan a una operación.
lancamentos-bloqueio-parcela = El movimiento paga una cuota de { $contrato }.
lancamentos-um-contrato = un contrato
lancamentos-operacoes = Operaciones
lancamentos-vinculado-toast = Movimiento vinculado
lancamentos-vinculado-descricao = { $operacao }. Salió de ingresos y gastos y pasó a ser un aporte.
lancamentos-vinculo-desfeito = Vínculo deshecho
lancamentos-vinculo-desfeito-descricao = El movimiento volvió al tipo y a la categoría de antes.
lancamentos-vincular-titulo = Vincular a una operación
lancamentos-vincular = Vincular
lancamentos-desvincular = Desvincular
lancamentos-vinculado-nota = Cuenta como aporte ({ $tipo }), fuera de ingresos y gastos. Desvincular devuelve el tipo y la categoría de antes.
lancamentos-procurando-operacoes = Buscando las operaciones…
lancamentos-nenhuma-operacao = Ninguna operación coincide. Registra la inversión en { $carteira } (hasta 7 días de la fecha, monto parecido), activa las cuotas de capital en la cuenta de la cooperativa o crea la meta, y vuelve aquí.
lancamentos-situacao = Situación
lancamentos-entra = ✓ Entra
lancamentos-fica-de-fora = Queda fuera: { $motivo }
lancamentos-n-vinculados = { $n ->
        [one] { $n } movimiento vinculado
       *[other] { $n } movimientos vinculados
    }
lancamentos-vinculados-descricao = { $operacao }. Salieron de ingresos y gastos y pasaron a ser aportes.
lancamentos-vinculados-descricao-fora = { $operacao }. { $fora ->
        [one] { $fora } quedó fuera.
       *[other] { $fora } quedaron fuera.
    } Salieron de ingresos y gastos y pasaron a ser aportes.
lancamentos-confirmar-vinculo = Confirmar el vínculo
lancamentos-n-marcados-destino = { $n ->
        [one] { $n } movimiento marcado
       *[other] { $n } movimientos marcados
    } → { $destino }
lancamentos-n-marcados-escolha = { $n ->
        [one] { $n } movimiento marcado
       *[other] { $n } movimientos marcados
    }: elige la operación de destino.
lancamentos-vincular-n = { $n ->
        [one] Vincular { $n } movimiento
       *[other] Vincular { $n } movimientos
    }
lancamentos-conferindo = Revisando los movimientos…
lancamentos-destino = Destino:
lancamentos-total = Total:
lancamentos-n-ficam-de-fora = { $n ->
        [one] { $n } movimiento queda fuera
       *[other] { $n } movimientos quedan fuera
    }
lancamentos-outros-vinculados = Los demás se vinculan; estos siguen como están.
lancamentos-nenhum-vinculavel = Ninguno de los marcados se puede vincular a esta operación.
lancamentos-a-vincular = Movimientos a vincular
lancamentos-vinculados-nota = Al vincularlos, salen de ingresos y gastos y pasan a ser aportes, sin categoría. Puedes deshacerlo desde el historial o desvincularlos uno por uno.
lancamentos-nenhuma-operacao-marcados = Ninguna operación coincide con los marcados. Las compras y ventas de la cartera se vinculan a un movimiento a la vez (menú de la fila); para varios, usa una meta o las cuotas de capital.

lancamentos-ajuste-saldo = Ajuste de saldo
lancamentos-ajustar = Ajustar saldo

## Cópias entre contas

lancamentos-copias-aviso = { $n ->
    [one] { $n } movimiento repite una compra ya registrada en otra cuenta
   *[other] { $n } movimientos repiten compras ya registradas en otra cuenta
}
lancamentos-copias-aviso-nota = Mismo identificador del banco: en general el archivo de una cuenta importado en otra. Cuentan dos veces en el flujo y en "sin categoría".
lancamentos-copias-titulo = Copias entre cuentas
lancamentos-copias-descricao = Cada fila es un movimiento registrado en dos cuentas con el mismo identificador del banco. Marca las copias para eliminar; queda el de la cuenta correcta (el de la tarjeta, en las compras). Se puede deshacer desde el historial.
lancamentos-copias-lista = Copias encontradas
lancamentos-copia-rotulo = Eliminar la copia en { $conta } del { $data }: { $descricao }
lancamentos-copia-lado = { $lado }: { $conta }, { $data } · { $descricao }
lancamentos-copia-apagar = Copia
lancamentos-copia-fica = Queda
lancamentos-copia-transferencia = Esta copia está vinculada como transferencia a otro movimiento: al eliminarla, se deshace el vínculo y el otro lado vuelve a ser gasto o ingreso.
lancamentos-apagar-n-copias = { $n ->
    [one] Eliminar { $n } copia
   *[other] Eliminar { $n } copias
}
lancamentos-n-copias-apagadas = { $n ->
    [one] { $n } copia eliminada
   *[other] { $n } copias eliminadas
}
lancamentos-copias-saldo-refeito = { $n ->
    [one] Se rehízo el ajuste de saldo que las contaba.
   *[other] Se rehicieron los { $n } ajustes de saldo que las contaban.
}
lancamentos-copias-erro = No se pudieron eliminar las copias
lancamentos-resgate = Rescate
lancamentos-sem-categoria-aporte = Aporte en inversiones
lancamentos-sem-categoria-resgate = Rescate de inversiones
lancamentos-sem-categoria-aporte-ajuda = Aporte y rescate no usan categoría: el dinero cambia de lugar en el patrimonio y queda fuera del flujo de caja y del presupuesto. En el Sankey cuenta la salida del banco hacia la corredora (aporte) y la vuelta (rescate); las compras y ventas dentro de la corredora no vuelven a entrar.
lancamentos-n-alterados-n-fora = { $n } cambiado(s); { $fora } aporte(s), rescate(s), pago(s) de tarjeta o ajuste(s) quedaron sin categoría, porque no la usan
lancamentos-campo-cartao = Tarjeta usada
lancamentos-cartao-dica = La compra queda con quien usa esta tarjeta, si no eliges el miembro.
lancamentos-cartao-nao-informado = Sin indicar
lancamentos-campo-moeda-origem = Moneda de la compra
lancamentos-moeda-origem-dica = Compra en el exterior: la moneda y el monto que cobró el comercio.
lancamentos-campo-valor-origem = Monto en la moneda de la compra
