# Presupuesto › Metas, Fondo de emergencia y Jubilación: metas con progreso y proyección, la
# calculadora del fondo, el planificador de jubilación (con el INSS por las reglas) y los
# escenarios (docs/i18n-design.md). Las notas y reglas de las simulaciones vienen del núcleo.
# Revisión: borrador por IA, sin revisión nativa.

## Tipos y situación de la meta

metas-tipo-emergencia = Fondo de emergencia
metas-tipo-viagem = Viaje
metas-tipo-quitacao = Pago de deuda
metas-tipo-compra = Compra
metas-tipo-aposentadoria = Jubilación
metas-tipo-educacao = Educación
metas-tipo-outra = Otra

metas-situacao-atingida = Alcanzada
metas-situacao-no-caminho = En camino
metas-situacao-fora-ritmo = Fuera de ritmo
metas-situacao-sem-aportes = Sin aportes
metas-situacao-sem-data = Sin fecha objetivo

## Formulario

metas-erro-nome = Escribe el nombre de la meta.
metas-erro-valor = Escribe un monto objetivo mayor que cero.
metas-erro-data-passou = La fecha objetivo ya pasó.
metas-erro-gravar = No se pudo guardar.
metas-atualizada = Meta actualizada
metas-criada = Meta creada
metas-editar-titulo = Editar meta · { $nome }
metas-nova = Nueva meta
metas-criar = Crear meta
metas-nome = Nombre
metas-tipo = Tipo
metas-valor-alvo = Monto objetivo
metas-data-alvo = Fecha objetivo
metas-data-alvo-dica = Opcional; sin fecha, no hay aporte necesario.
metas-ja-guardado = Ya ahorrado fuera de las cuentas
metas-aporte-planejado = Aporte mensual planificado
metas-aporte-planejado-dica = Se usa en la proyección cuando las cuentas vinculadas no tienen ritmo.
metas-contas-vinculadas = Cuentas vinculadas
metas-contas-vinculadas-dica = El saldo de hoy de las cuentas entra en el monto actual; el promedio de aportes netos de los últimos 6 meses es el ritmo.
metas-notas = Notas

## Tarjeta de la meta

metas-contas = Cuentas
metas-nenhuma-conta = ninguna
metas-de-alvo = de { $valor }
metas-progresso-de = Progreso de { $nome }
metas-falta = Falta
metas-aporte-necessario = Aporte necesario
metas-por-mes = /mes
metas-ritmo = Ritmo
metas-ritmo-planejado = Ritmo (planificado)
metas-ritmo-real = Ritmo (6 meses)
metas-projecao = Proyección
metas-projecao-planejada = { $data } (con el aporte planificado)
metas-sem-ritmo = sin ritmo de aportes para proyectar
metas-sem-cambio = Sin tipo de cambio para { $moedas }: saldo fuera del monto actual.
metas-editar-nome = Editar { $nome }
metas-reativar-nome = Reactivar { $nome }
metas-arquivar-nome = Archivar { $nome }
metas-excluir-nome = Eliminar { $nome }
metas-reativar = Reactivar
metas-arquivar = Archivar
metas-excluir = Eliminar
metas-nao-arquivou = No se pudo archivar
metas-excluir-titulo = Eliminar meta
metas-excluir-descricao = ¿Eliminar “{ $nome }”? Las cuentas vinculadas no cambian. Puedes deshacerlo desde el historial de cambios.
metas-excluida = Meta eliminada
metas-nao-excluiu = No se pudo eliminar

## Lista

metas-mostrar-arquivadas = Mostrar archivadas
metas-vazio-kicker = Sin metas
metas-vazio-titulo = Todavía no hay metas
metas-vazio-descricao = Crea una meta (fondo de emergencia, viaje, pago de una deuda) y vincula las cuentas donde guardas el dinero: el progreso y la proyección salen de los saldos reales.

## Fondo de emergencia

metas-reserva-base = Base
metas-reserva-caixa = caja
metas-reserva-competencia = devengo
metas-reserva-meses-analisados = Meses analizados
metas-reserva-meses-intervalo = { $de } a { $ate }
metas-reserva-fonte = Fuente
metas-reserva-fonte-valor = Categorías marcadas como esenciales
metas-reserva-meses-cobertura = Meses de cobertura
metas-reserva-media-de = Promedio de
metas-reserva-meses-na-media = Meses en el promedio
metas-reserva-n-meses = { $n } meses
metas-reserva-meta = Meta del fondo
metas-reserva-meta-dica = { $n } × gasto esencial
metas-reserva-guardado = Ahorrado
metas-reserva-guardado-dica = cuentas corrientes, ahorros y efectivo
metas-reserva-cobertura = Cobertura
metas-reserva-cobertura-valor =
    { $n ->
        [one] { $valor } mes
       *[other] { $valor } meses
    }
metas-reserva-cobertura-dica = del gasto esencial
metas-reserva-progresso = Fondo ahorrado en relación con la meta
metas-reserva-gasto-essencial = Gasto esencial promedio por mes
metas-reserva-gasto-total = Gasto total promedio por mes
metas-reserva-essenciais-total = Esenciales en el total
metas-reserva-guardado-meta = Ahorrado / meta
metas-reserva-sem-meses = Todavía no hay meses completos con gasto en categorías esenciales para calcular el promedio.
metas-reserva-criar = Crear meta de fondo de emergencia
metas-reserva-sem-meta = Sin meta, lo ahorrado es el saldo de las cuentas del día a día.
metas-reserva-atualizada = Meta del fondo actualizada
metas-reserva-atualizar = Actualizar la meta a { $valor }
metas-reserva-meta-em = La meta “{ $nome }” está en { $valor }.
metas-reserva-ja-usa = ✓ La meta “{ $nome }” ya usa este monto.

## INSS por las reglas de la reforma

metas-inss-sexo = Sexo
metas-inss-homem = Hombre
metas-inss-mulher = Mujer
metas-inss-nascimento = Fecha de nacimiento
metas-inss-nascimento-dica = Vacío: la edad de hoy.
metas-inss-contribuicao = Aportes hasta hoy (años)
metas-inss-contribuicao-reforma = Aportes al 13/11/2019
metas-inss-contribuicao-reforma-dica = Vacío: estimado sin interrupciones.
metas-inss-media = Promedio de salarios desde 07/1994
metas-inss-media-dica = Los salarios de aporte, actualizados.
metas-inss-contribui-ate = Aporta hasta los
metas-inss-regras = Reglas: { $sistema }
metas-inss-regra = Regla
metas-inss-cumpre-em = Se cumple el
metas-inss-beneficio-data = Beneficio en esa fecha
metas-inss-na-idade = A la edad del plan
metas-inss-usada = usada
metas-inss-cumpre-data = { $data } ({ $idade } años)
metas-inss-nao-cumpre = no se cumple
metas-inss-fator = factor previsional
metas-inss-aos = { $valor } a los { $idade }
metas-ssa-pia = Beneficio a la edad plena (PIA)
metas-ssa-pia-dica = El del extracto en ssa.gov/myaccount.
metas-ssa-aime = AIME (sin el PIA)
metas-ssa-aime-dica = Promedio mensual de los 35 mejores años corregidos.
metas-ssa-creditos = Créditos hasta hoy
metas-ssa-creditos-dica = Vacío: 40 (ya tiene derecho).
metas-ssa-trabalha-ate = Trabaja hasta los
metas-pt-carreira = Años cotizados hasta hoy
metas-pt-rr = Remuneración de referencia
metas-pt-rr-dica = Salarios de la carrera ÷ (14 × años).
metas-pt-antes-17 = Empezó a cotizar antes de los 17

## Planificador de jubilación

metas-apos-titulo = Jubilación e independencia financiera
metas-apos-intro = Miles de escenarios de mercado en valores reales (por encima de la inflación): el patrimonio de hoy, los aportes hasta dejar de trabajar, el gasto mensual deseado, la jubilación pública ({ $sistema }) y los eventos de vida.
metas-apos-idade-hoje = Edad hoy
metas-apos-aposentadoria-aos = Jubilación a los
metas-apos-expectativa = Esperanza de vida
metas-apos-patrimonio-hoje = Patrimonio invertido hoy
metas-apos-patrimonio-dica-valor = Vacío: la cartera ({ $valor }).
metas-apos-patrimonio-dica = Vacío: la cartera, con planes de pensión.
metas-apos-aporte = Aporte mensual hasta dejar de trabajar
metas-apos-gasto = Gasto mensual deseado
metas-apos-inss-informar = Indicar el monto
metas-apos-inss-calcular = Calcular por las reglas
metas-apos-inss-mensal = Valor mensual estimado ({ $sistema })
metas-apos-inss-a-partir = { $sistema } a partir de los
metas-apos-retorno = Rendimiento real (% anual)
metas-apos-volatilidade = Volatilidad (% anual)
metas-apos-eventos = Eventos de vida
metas-apos-idade = Edad
metas-apos-evento-valor = Monto (− si sale)
metas-apos-evento-o-que = Qué es
metas-apos-remover-evento = Quitar el evento { $n }
metas-apos-adicionar-evento = Agregar evento (herencia, venta de un inmueble, universidad…)
metas-apos-informe-idade = Indica la edad de hoy para simular.
metas-apos-simulando = Simulando los escenarios
metas-apos-chance = Probabilidad de que el dinero alcance
metas-apos-patrimonio-mediana = Patrimonio al jubilarse (mediana)
metas-apos-gasto-90 = Gasto mensual con 90% de probabilidad
metas-apos-necessario = Necesario al dejar de trabajar (90% de probabilidad)
metas-apos-acaba-aos = En el escenario malo, el dinero se acaba a los
metas-apos-n-anos = { $n } años
metas-apos-nao-acaba = no se acaba
metas-apos-patrimonio-idade = Patrimonio por edad
metas-apos-ruim = Escenario malo (10%)
metas-apos-mediana = Mediana
metas-apos-bom = Escenario bueno (90%)
metas-apos-salvo = Plan de jubilación guardado
metas-apos-usar-principal = Usar como plan principal
metas-apos-salvar = Guardar plan

## Escenarios y meta vinculada al plan

metas-cen-salvo = Escenario “{ $nome }” guardado
metas-cen-cenario = Escenario
metas-cen-principal = Plan principal
metas-cen-atualizado = Escenario actualizado
metas-cen-atualizar = Actualizar “{ $nome }”
metas-cen-apagado = Escenario eliminado
metas-cen-apagar = Eliminar escenario
metas-cen-novo = Nuevo escenario
metas-cen-exemplo = Ej.: jubilarse a los 55
metas-cen-salvar = Guardar como escenario
metas-cen-comparar = Comparar escenarios
metas-cen-chance = Probabilidad
metas-cen-gasto-90 = Gasto con 90%
metas-cen-necessario = Necesario al dejar de trabajar
metas-cen-previsto = Previsto al dejar de trabajar
metas-cen-na-tela = En pantalla ahora
metas-ligada = Meta vinculada: { $nome } · { $pct }% de { $valor }
metas-ligada-ate = Meta vinculada: { $nome } · { $pct }% de { $valor } hasta { $ano }
metas-ligada-desligar = Desvincular de la meta
metas-ligada-explica = Una meta de jubilación vinculada al plan sigue el patrimonio necesario, la fecha y el aporte del plan guardado.
metas-ligada-toast = Meta vinculada al plan
metas-ligada-toast-descricao = “{ $nome }” sigue el plan guardado en Presupuesto › Metas.
metas-ligada-criar = Crear meta vinculada al plan

## Metas com rendimento e gastos anuais

metas-rendimento = Rendimiento real al año
metas-rendimento-dica = Cuánto rinde el dinero ahorrado por encima de la inflación, como supuesto tuyo. Vacío: la cuenta solo divide lo que falta entre los meses.
metas-erro-retorno = Usa un rendimiento entre −5% y 15% al año.
metas-rendimento-linha = Con { $taxa } al año por encima de la inflación: { $aportes } de aportes y { $rendimento } de rendimiento. Sin rendimiento, serían { $linear }/mes.
metas-rendimento-alvo-epoca = En dinero de hoy. En la fecha, con una inflación de { $inflacao } al año, el objetivo vale cerca de { $valor }.
metas-rendimento-premissa = El rendimiento es un supuesto, no una promesa.
metas-reserva-anuais = Incluir gastos anuales
metas-reserva-anuais-dica = Impuestos del auto y del inmueble, seguros y útiles escolares entran divididos entre 12, en lugar de los meses pico.
metas-reserva-custo-mensal = Costo de vida mensual
metas-reserva-essenciais-regulares = Esenciales del mes, sin los anuales
metas-reserva-provisao-anual = Provisión de los gastos anuales esenciales
metas-reserva-anuais-titulo = Gastos anuales en los últimos 12 meses
metas-reserva-anuais-tabela = Gastos anuales
metas-reserva-anuais-mes = { $valor }/mes
metas-reserva-anuais-nao-essencial = fuera de la reserva
metas-reserva-anuais-sem-ano = Los gastos anuales aparecen cuando haya 12 meses completos de movimientos.
metas-reserva-anuais-nenhum = Ningún gasto anual en los últimos 12 meses (categorías con gasto en hasta 3 meses del año).
metas-reserva-meta-dica-custo = { $n } × costo de vida mensual

## Fuentes de la meta (docs/goal-sources.md)

metas-fontes = Fuentes
metas-fontes-nome = Fuentes de { $nome }
metas-fontes-titulo = Fuentes de { $nome }
metas-fontes-dica = El valor actual es la suma de las fuentes. Una cuenta cuenta por su saldo, nunca negativo (una cuenta en rojo es deuda, no reserva); una inversión cuenta por su valor neto de rescate cuando lo hay, si no por el de mercado; una etiqueta reúne todas las posiciones que la tienen, también las que la reciban después.
metas-fontes-vazio = Todavía no hay fuentes: vincula una cuenta, una etiqueta o las posiciones de abajo.
metas-fontes-desatualizadas = El valor puede estar desactualizado: { $fontes }.
metas-fonte-tipo-inicial = Guardado fuera
metas-fonte-tipo-carteira = Cartera
metas-fonte-tipo-conta = Cuenta
metas-fonte-tipo-conta-inteira = Cuenta entera
metas-fonte-tipo-ativo = Inversión
metas-fonte-tipo-etiqueta = Etiqueta
metas-fonte-tipo-lancamentos = Movimientos
metas-fonte-valor-saldo = saldo
metas-fonte-valor-mercado = valor de mercado
metas-fonte-valor-liquido = neto de rescate
metas-fonte-valor-custo = costo (sin cotización)
metas-fonte-valor-informado = valor informado
metas-fonte-remover = Desvincular { $nome }
metas-fonte-erro = No se pudieron cambiar las fuentes
metas-fonte-negativo-fora = El saldo negativo quedó fuera: es deuda, no reserva.
metas-fonte-desatualizada = Puede estar desactualizado (cuenta sin extracto reciente o activo sin cotización).
metas-fonte-ligadas = { $n ->
    [one] 1 inversión vinculada a la meta
   *[other] { $n } inversiones vinculadas a la meta
}
metas-fonte-liquidez = Usar mis inversiones de liquidez diaria
metas-fonte-liquidez-dica = Renta fija con liquidez diaria, Tesouro Selic y fondos DI o referenciados con posición abierta que todavía no están en esta meta.
metas-fonte-liquidez-buscando = Buscando las inversiones…
metas-fonte-liquidez-vazio = Ninguna inversión de liquidez diaria fuera de esta meta.
metas-fonte-adicionar = Agregar
metas-fonte-adicionar-todos = Agregar todas ({ $n })
metas-fonte-conta = Cuenta
metas-fonte-escolha-conta = Elige la cuenta
metas-fonte-etiqueta = Etiqueta de activos
metas-fonte-etiqueta-dica = Entran todas las posiciones con la etiqueta, también las que la reciban después.
metas-fonte-escolha-etiqueta = Elige la etiqueta
metas-fonte-sem-etiquetas = Todavía no hay etiquetas
