# "Por resolver" (núcleo `attention` y la interfaz): la lista única de lo que necesita a la
# persona y la línea que dice hasta cuándo valen los números (docs/to-resolve.md).

## Núcleo

atencao-erro-item = Este elemento no se puede descartar.
atencao-erro-so-interface = Esto solo se puede hacer en la interfaz de Treasury.
atencao-venceu-titulo = { $descricao } venció el { $data }
atencao-venceu-corpo = { $valor } en { $conta } sigue como previsto. Si ya se pagó, márcalo como pagado o importa el extracto; si no, págalo para evitar intereses.
atencao-caixa-titulo = { $n ->
    [one] 1 archivo esperando en la bandeja de entrada
   *[other] { $n } archivos esperando en la bandeja de entrada
}
atencao-caixa-corpo = Archivos que llegaron a la carpeta de entrada y todavía no se importaron.


## Núcleo: tablas anuales incluidas

tabelas-inss = INSS (piso y tope)
tabelas-pensao-pt = Segurança Social (IAS y pensiones mínimas)
tabelas-pensao-us = Social Security (puntos de corte)
atencao-regras-titulo = { $n ->
    [one] 1 regla de categoría lista para crear
   *[other] { $n } reglas de categoría listas para crear
}
atencao-regras-corpo = Ya le diste la misma categoría tres veces a { $nomes }. Con la regla, los próximos movimientos entran con la categoría.
atencao-tabelas-titulo = { $n ->
    [one] 1 tabla de la app puede estar desactualizada
   *[other] { $n } tablas de la app pueden estar desactualizadas
}
atencao-tabelas-item = { $nome } de { $ano }
atencao-tabelas-corpo = En uso: { $tabelas }. Las estimaciones siguen saliendo, con los valores de ese año.
atencao-tabelas-acao = Actualiza Treasury para recibir las tablas nuevas.

## Interfaz: tarjeta del Panel y campana de la barra superior

atencao-titulo = Por resolver
atencao-origem = calculado por la app, sin IA
atencao-vazio = Todo al día. Nada por resolver ahora.
atencao-resolver = Marcar como resuelto
atencao-dispensar-dica = Sale de la lista ahora. Si sigue vigente, vuelve en una semana.
atencao-ver-todos = Ver los { $n }
atencao-ver-alertas = Ver todas las alertas
atencao-sino = { $n ->
    [0] Nada por resolver
    [one] 1 asunto por resolver
   *[other] { $n } asuntos por resolver
}
atencao-gravidade-critical = Urgente
atencao-gravidade-warning = Atención
atencao-gravidade-info = Aviso

## Interfaz: hasta cuándo valen los números

atencao-cobertura-completa = Cuentas al día: datos hasta el { $data }.
atencao-cobertura-completa-sem-data = Cuentas al día.
atencao-cobertura-parcial = { $n ->
    [one] 1 cuenta sin extracto reciente ({ $contas }).
   *[other] { $n } cuentas sin extracto reciente ({ $contas }).
} Los totales pueden estar incompletos.
atencao-cobertura-mais = { $nomes } y { $n } más
atencao-cobertura-importar = Importar extracto

## Núcleo: tu próximo paso (los escalones)

degraus-know-titulo = Saber adónde va el dinero
degraus-balance-titulo = Gastar menos de lo que ganas
degraus-credit-titulo = Salir del crédito caro
degraus-reserve-titulo = Armar el fondo de emergencia
degraus-debts-titulo = Aligerar las deudas
degraus-invest-titulo = Invertir y proteger lo que tienes
degraus-know-sem-contas = Todavía no hay cuentas registradas.
degraus-know-sem-contas-acao = Registra la cuenta que más usas. Se puede empezar solo con ella.
degraus-know-meses = { $n ->
    [0] Todavía no hay un mes completo de movimientos.
    [one] Hay 1 mes completo de movimientos.
   *[other] Hay { $n } meses completos de movimientos.
} Con { $alvo }, Treasury puede decir adónde va el dinero.
degraus-know-meses-acao = Importa los extractos de los últimos meses o sigue anotando los gastos.
degraus-know-categorias = El { $pct } de los movimientos de los últimos 90 días tiene categoría. A partir del { $alvo }, los totales por categoría pasan a ser confiables.
degraus-know-categorias-acao = { $n ->
    [one] Ponle categoría al movimiento que falta.
   *[other] Ponles categoría a los { $n } movimientos que faltan.
}
degraus-balance-sem-renda = { $n ->
    [one] No hay ingresos registrados en el último mes
   *[other] No hay ingresos registrados en los últimos { $n } meses
}, así que no se puede saber si sobra o falta.
degraus-balance-sem-renda-acao = Registra el salario y los demás ingresos, o importa el extracto de la cuenta donde entran.
degraus-balance-acima = { $n ->
    [one] En el último mes
   *[other] En los últimos { $n } meses
} salió un { $pct } más de lo que entró{ $n ->
    [one] .
   *[other] { $meses ->
        [one] , y en 1 de ellos los gastos superaron los ingresos.
       *[other] , y en { $meses } de ellos los gastos superaron los ingresos.
    }
}
degraus-balance-acima-acao = Mira en el presupuesto las categorías que más crecieron y elige una para reducir este mes.
degraus-credit-rotativo = En los últimos meses, el resumen de la tarjeta quedó sin pagar completo en { $rotativo ->
    [one] 1 de ellos
   *[other] { $rotativo } de ellos
}. El crédito rotativo está entre los más caros.
degraus-credit-cheque = En los últimos meses, la cuenta quedó en rojo en { $cheque ->
    [one] 1 de ellos
   *[other] { $cheque } de ellos
}. El descubierto está entre los créditos más caros.
degraus-credit-ambos = En los últimos meses hubo resumen de tarjeta sin pagar completo y cuenta en rojo. El rotativo y el descubierto están entre los créditos más caros.
degraus-credit-acao = Cambia esa deuda por un crédito más barato y mira el plan de pago en Deudas.
degraus-reserve-essenciais = El fondo cubre { $meses } meses de gastos esenciales. La próxima marca es { $alvo ->
    [one] 1 mes
   *[other] { $alvo } meses
}.
degraus-reserve-gastos = El fondo cubre { $meses } meses de gastos. La próxima marca es { $alvo ->
    [one] 1 mes
   *[other] { $alvo } meses
}.
degraus-reserve-acao = Aparta un monto cada mes, en una inversión que se pueda rescatar el mismo día, hasta llegar.
degraus-debts-peso = Las cuotas de préstamos y financiaciones se llevan el { $pct } de los ingresos. Por encima del { $alvo }, el presupuesto queda ajustado.
degraus-debts-acao = Simula en el plan de pago qué pasa al pagar un poco más por mes en la deuda más cara.
degraus-invest-porque = Las cuentas cierran, no hay crédito caro, el fondo está armado y las deudas caben en los ingresos.
degraus-invest-comecar-acao = Lo que pase del fondo puede ir a objetivos de plazo más largo. Crea una meta y define cuánto guardar por mes.
degraus-invest-carteira-acao = Revisa la asignación, la concentración y el costo de las comisiones de la cartera.

## Núcleo: cerrar el mes

fechamento-erro-mes = Solo se puede cerrar un mes que ya terminó.
fechamento-erro-passo = Paso desconocido del cierre del mes.
fechamento-erro-incompleto = Todavía faltan pasos para cerrar este mes.
fechamento-statements-titulo = Traer los extractos
fechamento-statements-falta = { $n ->
    [one] 1 cuenta sin extracto reciente: { $contas }.
   *[other] { $n } cuentas sin extracto reciente: { $contas }.
}
fechamento-categories-titulo = Poner categoría a los movimientos
fechamento-categories-falta = { $n ->
    [one] 1 movimiento del mes sin categoría.
   *[other] { $n } movimientos del mes sin categoría.
}
fechamento-balances-titulo = Revisar los saldos
fechamento-balances-falta = Compara el saldo de cada cuenta con el del banco y márcalo cuando coincida.
fechamento-review-titulo = Resolver lo pendiente
fechamento-review-falta = { $n ->
    [one] 1 asunto en Por resolver.
   *[other] { $n } asuntos en Por resolver.
}
fechamento-summary-titulo = Leer el resumen del mes
fechamento-summary-falta = Mira en los informes adónde fue el dinero y márcalo cuando lo leas.

## Interfaz: próximo paso y cerrar el mes

degraus-cartao = Tu próximo paso
degraus-posicao = Escalón { $n } de { $total }
degraus-estimado = Lectura provisional: hay cuentas sin extracto reciente.
degraus-feito = hecho
degraus-atual = ahora
degraus-depois = después
fechamento-cartao = Cerrar { $mes }
fechamento-progresso = { $n } de { $total }
fechamento-origem = cinco pasos, sin IA
fechamento-marcar = Marcar como hecho
fechamento-desmarcar = Desmarcar
fechamento-fechar = Marcar el mes como cerrado
fechamento-falta = Faltan pasos para cerrar el mes.
fechamento-passo-feito = Hecho
fechamento-passo-pendente = Pendiente
atencao-compras-titulo = { $n } compras de supermercado de este mes sin artículos
atencao-compras-corpo = Detalle los artículos que quiera seguir (carne, café, leche): el precio de cada producto y la inflación de su cesta salen de ahí. No hace falta el ticket entero.
