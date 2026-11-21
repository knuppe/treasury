# Cartera › Bienes: patrimonio físico (inmuebles, vehículos, terrenos, equipos y colecciones),
# con costo, tasaciones, mejoras, venta y préstamo vinculado (docs/i18n-design.md). La fórmula
# del valor y los errores vienen del núcleo; IPCA, IGP-M, INCC, ITBI, FIPE y LTV quedan en el
# original. Revisión: borrador por IA, sin revisión nativa.

## Tipos, métodos, índices y fuentes

bens-tipo-imovel = Inmueble
bens-tipo-veiculo = Vehículo
bens-tipo-terreno = Terreno
bens-tipo-equipamento = Equipo
bens-tipo-colecao = Arte y colección
bens-tipo-outro = Otro

bens-metodo-nenhum = Sin estimación
bens-metodo-indice = Actualización por índice
bens-metodo-declinante = Depreciación anual
bens-metodo-linear = Depreciación lineal
bens-metodo-fipe = Tabla FIPE
bens-metodo-nenhum-dica = Vale la última tasación registrada; sin tasación, el costo.
bens-metodo-indice-dica = Costo (o la última tasación) actualizado por el índice elegido. Es una estimación: el mercado inmobiliario no sigue el índice al pie de la letra.
bens-metodo-declinante-dica = Pierde un porcentaje fijo por año sobre el valor anterior. Los autos pierden ~15–20% el 1.er año y ~8–10% después; 12% anual es el promedio de los primeros cinco años.
bens-metodo-linear-dica = Pierde el mismo valor cada año hasta el residual al final de la vida útil (electrónica y máquinas: 5 a 10 años).
bens-metodo-fipe-dica = Precio promedio de la tabla FIPE en la última consulta; entre consultas, depreciación anual.

bens-indice-ipca-dica = Inflación oficial: mantiene el poder de compra de lo que pagaste.
bens-indice-igpm-dica = Índice de los alquileres: más volátil, sigue los precios mayoristas.
bens-indice-incc-dica = Costo de la construcción: aproxima el costo de reconstruir el inmueble.

bens-categoria-reforma = Remodelación
bens-categoria-ampliacao = Ampliación
bens-categoria-benfeitoria = Mejora
bens-categoria-manutencao = Mantenimiento

bens-custo-escritura = Escritura
bens-custo-registro = Registro
bens-custo-corretagem = Corretaje
bens-custo-outro = Otro costo

bens-fonte-manual = Estimación propia
bens-fonte-avaliacao = Tasación
bens-fonte-laudo = Informe de tasación
bens-fonte-indice = Por índice

bens-base-avaliacao = tasación
bens-base-automatica = estimación automática
bens-base-custo = costo
bens-base-vendido = vendido
bens-base-nao-comprado = todavía no comprado

bens-fipe-carro = Auto
bens-fipe-moto = Moto
bens-fipe-caminhao = Camión

## Cómo se estima el valor (línea corta) y dueños

bens-metodo-texto-indice = actualizado por el { $indice }
bens-metodo-texto-declinante = depreciación de { $taxa } anual
bens-metodo-texto-linear = lineal en { $anos } años
bens-metodo-texto-linear-residual = lineal en { $anos } años hasta { $residual }
bens-metodo-texto-fipe = tabla FIPE + { $taxa } anual
bens-metodo-texto-fipe-codigo = tabla FIPE { $codigo } + { $taxa } anual
bens-metodo-texto-nenhum = sin estimación automática
bens-donos-par = { $a } y { $b }
bens-familia = Familia

## Etiquetas repetidas

bens-titulo = Bienes
bens-bem = Bien
bens-o-bem = bien
bens-tipo = Tipo
bens-valor = Valor
bens-custo = Costo
bens-ganho = Ganancia
bens-patrimonio = Patrimonio
bens-avaliacao = Tasación
bens-avaliacoes = Tasaciones
bens-saldo-devedor = Saldo pendiente
bens-data = Fecha
bens-descricao = Descripción
bens-fonte = Fuente
bens-projeto = Proyecto
bens-observacoes = Observaciones
bens-opcional = Opcional.
bens-nenhum = Ninguno
bens-mes = Mes
bens-ao-ano-abrev = anual
bens-vendido = vendido
bens-arquivado = archivado
bens-acoes-de = Acciones de { $nome }
bens-editar-nome = Editar { $nome }
bens-excluir-nome = Eliminar { $nome }
bens-excluir = Eliminar
bens-reativar = Reactivar
bens-arquivar = Archivar
bens-reativado = Bien reactivado
bens-arquivado-toast = Bien archivado
bens-excluido = Bien eliminado
bens-excluir-titulo = Eliminar { $nome }
bens-excluir-bem = Eliminar bien
bens-novo = Nuevo bien
bens-nao-excluiu = No se pudo eliminar

## Figuras

bens-valor-estimado = Valor (≈ estimado)
bens-custo-acumulado = Costo acumulado
bens-avaliacao-em = { $fonte } el { $data }
bens-valor-do-bem = Valor del bien
bens-patrimonio-do-bem = Patrimonio del bien
bens-fig-valor-titulo = Valor × costo a lo largo del tiempo
bens-fig-valor-fonte = Valor: última tasación o estimación automática · costo: compra + costos + mejoras de capital
bens-fig-valor-fonte-estimado = Valor: última tasación o estimación automática (≈ estimado) · costo: compra + costos + mejoras de capital
bens-fig-valor-aria =
    { $n ->
        [one] Valor de { $nome } a lo largo del tiempo, con el costo acumulado y { $n } tasación.
       *[other] Valor de { $nome } a lo largo del tiempo, con el costo acumulado y { $n } tasaciones.
    }
bens-fig-valor-tabela = Valor mes a mes · { $nome }
bens-fig-patrimonio-titulo = Patrimonio del bien × saldo pendiente
bens-fig-patrimonio-fonte = Saldo pendiente según el cronograma de los contratos vinculados · la suma de las franjas es el valor del bien
bens-fig-patrimonio-aria = Patrimonio de { $nome } y saldo pendiente del préstamo, apilados mes a mes.
bens-fig-patrimonio-tabela = Patrimonio del bien mes a mes · { $nome }

## Lista y resumen

bens-resumo-valor = Valor de los bienes
bens-n-bens =
    { $n ->
        [one] { $n } bien
       *[other] { $n } bienes
    }
bens-resumo-ganho = Ganancia sobre el costo
bens-custo-de = costo de { $valor }
bens-resumo-patrimonio = Patrimonio de los bienes
bens-valor-menos-saldo-financiamentos = Valor − saldo pendiente de los préstamos
bens-saldo-devedor-valor = saldo pendiente { $valor }
bens-sem-financiamentos = sin préstamos vinculados
bens-cmd-grupo = Patrimonio
bens-cmd-benfeitoria = Registrar mejora en { $nome }
bens-cmd-benfeitoria-busca = obra,remodelación,ampliación,mejora,mantenimiento
bens-cmd-avaliacao = Registrar tasación de { $nome }
bens-cmd-avaliacao-busca = informe,tasación,valor de mercado,fipe
bens-abrir = Abrir bien
bens-registrar-benfeitoria-menu = Registrar mejora…
bens-registrar-avaliacao-menu = Registrar tasación…
bens-novo-projeto-menu = Nuevo proyecto del bien…
bens-ligar-financiamento-menu = Vincular préstamo…
bens-registrar-venda-menu = Registrar venta…
bens-editar-cadastro-menu = Editar registro…
bens-excluir-menu = Eliminar…
bens-nao-foi-possivel = No se pudo
bens-avaliando = Valuando los bienes
bens-vazio-kicker = Sin bienes
bens-vazio-titulo = Registra el primer inmueble, vehículo o terreno
bens-vazio-descricao = Indica la fecha y el precio de compra con los costos (ITBI, escritura, registro): el valor actual se estima por el índice, por la depreciación o por la tabla FIPE, y las mejoras entran en el costo. Vincula el préstamo de 07 · Deudas para ver el patrimonio del bien.
bens-mostrar-arquivados = Mostrar archivados y vendidos
bens-excluir-descricao-lista = Elimina el bien con costos, mejoras y tasaciones. Los préstamos y los movimientos siguen. Puedes deshacerlo desde el historial.

## Detalle del bien

bens-bem-nome = Bien { $nome }
bens-comprado-em = comprado el { $data }
bens-vendido-em = vendido el { $data }
bens-fora-patrimonio = fuera del patrimonio neto
bens-registrar-avaliacao = Registrar tasación
bens-fipe-consultada = Tabla FIPE consultada
bens-fipe-valor-atualizado = { $nome }: valor actualizado
bens-fipe-falhou = No se pudo consultar la FIPE
bens-consultar-fipe = Consultar FIPE
bens-vender = Vender
bens-benfeitoria = Mejora
bens-valor-atual = Valor actual
bens-vendido-maiusc = Vendido
bens-por-base = Por { $base }
bens-liquido-venda = neto de la venta
bens-compra = Compra
bens-custos-aquisicao = Costos de adquisición
bens-benfeitorias = Mejoras
bens-custo-total = Costo total
bens-manutencao-fora = Mantenimiento (fuera del costo)
bens-valorizacao = Valorización
bens-depreciacao = Depreciación
bens-ganho-realizado = Ganancia realizada
bens-ganho-nominal = Ganancia nominal
bens-sobre-custo = Sobre el costo
bens-ganho-real = Ganancia real (− IPCA)
bens-ao-ano = Por año
bens-ao-ano-real = Por año, real
bens-valor-menos-saldo = Valor − saldo pendiente
bens-saldo-ltv = saldo pendiente { $saldo } · LTV { $ltv }
bens-sem-financiamento = sin préstamo vinculado
bens-valor-estimado-titulo = ≈ Valor estimado
bens-valor-medido = Valor medido
bens-calculando-historico = Calculando el historial del bien
bens-financiamento = Préstamo
bens-ligar-financiamento = Vincular préstamo
bens-financiamentos-de = Préstamos de { $nome }
bens-financiamentos-vazio = Ningún contrato vinculado. Vincula el préstamo registrado en 07 · Deudas para ver el patrimonio del bien y el LTV.
bens-contrato = Contrato
bens-financiado = Financiado
bens-desligar-nome = Desvincular { $nome }
bens-financiamento-desligado = Préstamo desvinculado
bens-nao-desligou = No se pudo desvincular
bens-desligar = Desvincular
bens-benfeitorias-titulo = Mejoras, remodelaciones y obras
bens-entram-custo = Entran en el costo
bens-entram-custo-valor = remodelación, ampliación, mejora
bens-fora-custo-rotulo = Fuera del costo
bens-fora-custo-valor = mantenimiento
bens-benfeitorias-de = Mejoras de { $nome }
bens-benfeitorias-vazio = Ninguna obra registrada.
bens-fora-custo = fuera del costo
bens-sem-lancamentos-estimado = sin movimientos: vale lo estimado
bens-n-lancamentos =
    { $n ->
        [one] { $n } movimiento
       *[other] { $n } movimientos
    }
bens-benfeitoria-excluida = Mejora eliminada
bens-avaliacoes-de = Tasaciones de { $nome }
bens-avaliacoes-vazio = Ninguna tasación: el valor se estima automáticamente a partir del costo.
bens-excluir-avaliacao-de = Eliminar tasación del { $data }
bens-avaliacao-excluida = Tasación eliminada
bens-custos-aquisicao-prefixo = Costos de adquisición:
bens-observacoes-texto = Observaciones: { $notas }
bens-excluir-descricao-detalhe = Se quitan el bien, las mejoras y las tasaciones; el préstamo sigue en 07 · Deudas. Puedes deshacerlo desde el historial de cambios.

## Registro del bien

bens-editar-bem = Editar bien
bens-form-descricao = Inmueble, vehículo, terreno, equipo o colección: la compra con los costos de adquisición, el método de valuación y, si lo hay, el préstamo.
bens-erro-nome = Ponle un nombre al bien.
bens-erro-data-compra = Indica la fecha de compra.
bens-erro-preco-compra = Indica el precio de compra.
bens-erro-taxa = Tasa entre 0 y 100% anual.
bens-erro-vida-util = Vida útil entre 1 y 100 años.
bens-erro-fipe = Elige marca, modelo y año en la tabla FIPE.
bens-erro-valor-invalido = Monto no válido.
bens-alterado = Bien modificado
bens-cadastrado = Bien registrado
bens-nome-bem = Nombre del bien
bens-nome-exemplo = Departamento del centro
bens-data-compra = Fecha de compra
bens-preco-compra = Precio de compra
bens-preco-compra-dica = Valor del bien, sin los costos.
bens-custos-aquisicao-dica = ITBI, escritura, registro y corretaje entran en el costo del bien (y en la ganancia de capital).
bens-custo-n = Costo { $n }
bens-remover-custo = Quitar costo { $n }
bens-remover = Quitar
bens-adicionar-custo = Agregar costo
bens-estimado-por = Valor actual estimado por
bens-indice-correcao = Índice de actualización
bens-depreciacao-aa = Depreciación (% anual)
bens-depreciacao-fipe-dica = Se usa entre las consultas a la tabla FIPE.
bens-depreciacao-dica = 12% anual por defecto para autos.
bens-vida-util = Vida útil (años)
bens-residual = Valor residual (% del costo)
bens-residual-dica = Cuánto queda al final de la vida útil. Opcional.
bens-endereco = Dirección o identificación
bens-endereco-dica = Opcional (matrícula, patente, chasis…).
bens-financiamento-dica = Contrato de 07 · Deudas que financió el bien. Opcional.
bens-de-membro = De { $nome }
bens-entra-patrimonio = Entra en el patrimonio neto
bens-salvar-alteracoes = Guardar cambios
bens-cadastrar = Registrar bien

## Tabla FIPE

bens-fipe-explica = Con marca, modelo y año, la app consulta el precio promedio en la tabla FIPE (parallelum.com.br, gratuita) cuando lo pidas.
bens-fipe-buscar = Buscar en la tabla FIPE
bens-fipe-tipo-veiculo = Tipo de vehículo
bens-fipe-marca = Marca
bens-fipe-escolha-marca = Elige la marca
bens-fipe-modelo = Modelo
bens-fipe-escolha-modelo = Elige el modelo
bens-fipe-ano = Año y combustible
bens-fipe-escolha-ano = Elige el año
bens-fipe-codigo = Código FIPE
bens-fipe-codigo-dica = Opcional: la consulta verifica el código.

## Mejora, tasación, venta y préstamo

bens-editar-benfeitoria = Editar mejora
bens-registrar-benfeitoria-obra = Registrar mejora u obra
bens-benfeitoria-descricao = { $nome }: las remodelaciones, ampliaciones y mejoras entran en el costo; el mantenimiento es un gasto y queda fuera.
bens-erro-data = Indica la fecha.
bens-erro-descreva = Describe la obra.
bens-erro-valor-projeto = Indica el monto (o vincúlala a un proyecto).
bens-benfeitoria-alterada = Mejora modificada
bens-benfeitoria-registrada = Mejora registrada
bens-tipo-obra = Tipo de obra
bens-benfeitoria-exemplo = Remodelación del baño
bens-valor-estimado-campo = Monto estimado
bens-valor-projeto-dica = Con movimientos en el proyecto, vale lo que este gastó.
bens-fica-fora-custo = Queda fuera del costo del bien.
bens-soma-custo = Suma al costo del bien.
bens-projeto-gastos = Proyecto de los gastos
bens-projeto-gastos-dica = Los movimientos del proyecto se vuelven el monto de la obra, cada uno en su fecha.
bens-novo-projeto-benfeitoria = Nuevo proyecto con el nombre de la mejora
bens-sera-criado = Se creará “{ $nome }” para este bien; después pon los movimientos en él.
bens-a-benfeitoria = la mejora
bens-registrar = Registrar
bens-avaliacao-descricao = { $nome }: la tasación más reciente se vuelve el valor del bien; después de ella, la estimación automática parte de ella.
bens-erro-valor = Indica el monto.
bens-avaliacao-registrada = Tasación registrada
bens-avaliacao-obs-dica = Quién tasó, comparables… Opcional.
bens-vender-nome = Vender { $nome }
bens-venda-descricao = Registra la venta: el bien sale del patrimonio a partir de la fecha y la ganancia queda realizada. El dinero de la venta entra por la cuenta, como un movimiento.
bens-erro-venda-antes = La venta no puede ser antes de la compra.
bens-erro-preco = Indica el precio.
bens-venda-registrada = Venta registrada
bens-venda = Venta
bens-data-venda = Fecha de venta
bens-preco-venda = Precio de venta
bens-custos-venda = Costos de la venta
bens-custos-venda-dica = Corretaje, impuestos, comisiones.
bens-arquivar-bem = Archivar el bien
bens-registrar-venda = Registrar venta
bens-ligar-descricao = El saldo pendiente del contrato entra en el patrimonio de { $nome } (valor − saldo) y en el LTV. La deuda sigue contando una sola vez, en 07 · Deudas.
bens-financiamento-ligado = Préstamo vinculado
bens-nenhum-contrato-livre = Ningún contrato libre. Registra primero el préstamo en 07 · Deudas.
bens-escolha-contrato = Elige el contrato
bens-ligar-contrato = Vincular contrato
bens-gcap-titulo-vendido = Impuesto sobre la ganancia de la venta
bens-gcap-titulo-simular = Simular el impuesto de una venta
bens-gcap-data = Fecha de la venta
bens-gcap-preco = Precio de venta
bens-gcap-custos = Corretaje pagado en la venta
bens-gcap-unico = Único inmueble, sin otra venta de inmueble en los últimos 5 años
bens-gcap-unico-dica = Exento si el precio de venta es de hasta R$ 440 mil.
bens-gcap-reaplicado = Valor aplicado en otro inmueble residencial en hasta 180 días
bens-gcap-ganho-bruto = Ganancia bruta (venta − corretaje − costo)
bens-gcap-lei-7713 = Reducción de la Ley 7.713 ({ $pct } %)
bens-gcap-fr1 = Factor FR1 ({ $pct } %)
bens-gcap-fr2 = Factor FR2 ({ $pct } %)
bens-gcap-isencao-unico = Exención del único inmueble
bens-gcap-isencao-reaplicacao = Exención por la compra de otro inmueble residencial
bens-gcap-tributavel = Ganancia gravable
bens-gcap-imposto = Impuesto
bens-gcap-pequeno-valor = Exento: ventas de hasta R$ 35 mil en el mes
bens-gcap-darf = DARF
bens-gcap-darf-valor = código { $codigo }, vence el { $vencimento }
bens-gcap-nota = Como el programa GCAP de la Receita: el costo incluye los gastos de compra y las mejoras; impuesto del 15 % al 22,5 % por tramo (Ley 8.981/1995 art. 21). Revísalo en el GCAP antes de pagar.
