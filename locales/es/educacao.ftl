# Meta de universidad de los hijos (docs/design/college-goal.md): asistente, tarjeta, detalle,
# posiciones vinculadas y las alertas del recálculo anual.

## Alertas del recálculo anual

educacao-alerta-recalculo-titulo = Universidad: el aporte de { $nome } cambió
educacao-alerta-recalculo-corpo = En el recálculo anual, el aporte por mes pasó de { $antes } a { $depois } (en dinero de hoy). Revisa la mensualidad de hoy en el plan.
educacao-alerta-terminou-titulo = Universidad: la carrera de { $nome } terminó
educacao-alerta-terminou-corpo = La meta se puede archivar, y lo que sobró queda libre.

## Asistente

educacao-nova = Meta de universidad
educacao-editar-titulo = Plan de universidad · { $nome }
educacao-descricao = Cuánto ahorrar por mes para pagar la universidad de un hijo, con la inflación de la educación, en dinero de hoy.
educacao-passos = Pasos del plan
educacao-passo-quem = Quién
educacao-passo-faculdade = Qué universidad
educacao-passo-premissas = Supuestos
educacao-voltar = Volver
educacao-avancar = Siguiente
educacao-quem = Para quién es
educacao-filho-cadastrado = Hijo registrado
educacao-filho-vai-nascer = Todavía no nació
educacao-filho = Hijo
educacao-escolha-filho = Elige
educacao-filho-dica = { $idade } años hoy; cumple { $inicio } en { $ano }.
educacao-nascimento-previsto = Fecha prevista de nacimiento
educacao-idade-inicio = Edad de inicio
educacao-mes-inicio = Mes de inicio
educacao-mes-inicio-dica = En Brasil, el año lectivo empieza en febrero.
educacao-nome-meta = Nombre de la meta
educacao-nome-meta-dica = Opcional; el predeterminado es "Universidad de" y el nombre del hijo.
educacao-tipo = Tipo
educacao-tipo-particular = Privada
educacao-tipo-publica = Pública
educacao-tipo-exterior = En el exterior
educacao-curso = Carrera
educacao-curso-dica = Opcional, solo para recordar.
educacao-duracao = Duración
educacao-duracao-dica = Medicina suele durar 6 años.
educacao-anos = años
educacao-parcelas = Cuotas por año
educacao-parcelas-n = { $n ->
    [1] 1 por año (anual)
    [2] 2 por año (semestre)
    [4] 4 por año
   *[other] { $n } por año (mensualidad)
}
educacao-mensalidade = Mensualidad de hoy
educacao-valor-parcela = Valor de cada cuota hoy
educacao-mensalidade-dica = Pide una cotización a la universidad: el precio de hoy es la base del cálculo.
educacao-extras = Gastos extra por año
educacao-extras-dica = Matrícula, materiales, libros.
educacao-custo-vida = Costo de vida por mes
educacao-custo-vida-dica = Vivienda, transporte y comida en otra ciudad, si es el caso.
educacao-custo-vida-publica = En la pública, el costo suele ser este: vivienda, transporte y comida.
educacao-desconto = Beca o descuento
educacao-desconto-dica = Sobre la mensualidad. La app no estima becas.
educacao-inflacao = Inflación de la educación
educacao-inflacao-serie = Promedio de 10 años de la serie de educación
educacao-inflacao-serie-nome = Promedio de 10 años: { $serie }
educacao-inflacao-geral = Solo la inflación general
educacao-inflacao-fixa = Diferencia fija
educacao-inflacao-dica = Calculada por la app de { $de } a { $ate } ({ $anos } años completos), con las series ya descargadas.
educacao-inflacao-sem-dados = Todavía no hay años completos de la serie: el cálculo usa solo la inflación general hasta la próxima sincronización de los índices.
educacao-diferenca = Diferencia sobre la inflación
educacao-diferenca-dica = Cuánto sube la educación por encima de la inflación general, por año.
educacao-pct-ano = % anual
educacao-retorno = Retorno real supuesto
educacao-retorno-dica = Por encima de la inflación, para el dinero ahorrado. Es un supuesto de la familia.
educacao-aporte-modo = Aporte
educacao-aporte-corrigido = Ajustado por la inflación cada año
educacao-aporte-fixo = Monto fijo
educacao-ja-guardado-dica = Fuera de las cuentas y posiciones vinculadas a la meta.
educacao-hipoteses = Son supuestos para planificar, no una promesa de rendimiento ni una sugerencia de inversión.
educacao-falta-dado = Elige al hijo e indica la mensualidad para ver el cálculo.
educacao-erro-filho = Elige al hijo.
educacao-erro-nascimento = Registra la fecha de nacimiento del hijo en Familia.
educacao-erro-nascimento-previsto = Indica la fecha prevista de nacimiento.
educacao-erro-idade = La edad de inicio va de 0 a 60.
educacao-erro-duracao = La duración va de medio año a 15 años.
educacao-erro-mensalidade = Indica la mensualidad de hoy.
educacao-erro-desconto = El descuento va de 0% a 100%.
educacao-erro-retorno = El retorno va de −5% a 15% al año.
educacao-erro-diferenca = La diferencia va de −10% a 20% al año.

## Resultado

educacao-resultado = Resultado del plan
educacao-alvo-inicio = Necesario en { $data }
educacao-aporte-hoje = Aporte hoy
educacao-aporte-curso = Aporte durante la carrera
educacao-mensalidade-inicio = Mensualidad al inicio
educacao-mensalidade-epoca = cerca de { $valor } en dinero de la época
educacao-aporte-ano-que-vem = En { $ano }, cerca de { $valor } por mes (ajustado por la inflación).
educacao-aporte-fixo-valor = Si prefieres un monto fijo: { $valor } por mes.
educacao-so-ipca = Si la educación subiera solo con la inflación general: { $valor } por mes.
educacao-cobre-tudo = Lo ahorrado cubre la carrera hasta el final.
educacao-cobre-ate = { $ano ->
    [0] Lo ahorrado no cubre el primer año.
    [one] Lo ahorrado cubre hasta el 1.er año.
   *[other] Lo ahorrado cubre hasta el { $ano }.º año.
}
educacao-selo-diferenca = Educación { $pct } al año por encima de la inflación
educacao-selo-sem-diferenca = Educación con la inflación general
educacao-fonte-inflacao = { $selo } ({ $serie }, { $de } a { $ate }).

## Tarjeta y detalle

educacao-n-anos = { $n ->
    [one] 1 año
   *[other] { $n } años
}
educacao-n-meses = { $n ->
    [one] 1 mes
   *[other] { $n } meses
}
educacao-e = { " y " }
educacao-card-faltam = Empieza dentro de { $prazo }
educacao-card-em-curso = En curso
educacao-card-terminou = Carrera terminada
educacao-ver-plano = Ver plan
educacao-ver-plano-nome = Ver el plan de { $nome }
educacao-plano = Plan de universidad
educacao-periodo = { $inicio } a { $fim }
educacao-atualizar-mensalidade = Actualizar la mensualidad
educacao-editar-plano = Editar el plan
educacao-aba-anos = Año a año
educacao-aba-guardado = Lo ahorrado
educacao-aba-historico = Recálculos
educacao-col-ano = Año
educacao-col-mensalidade-hoje = Mensualidad (hoy)
educacao-col-mensalidade-epoca = Mensualidad (de la época)
educacao-col-pagamentos = Pagos en el año
educacao-col-saldo = Saldo al inicio del año
educacao-em-curso = carrera
educacao-tabela-nota = Valores en dinero de hoy, salvo la columna "de la época", que aplica la inflación general supuesta. El saldo sigue el plan: aportes hasta el inicio, pagos durante la carrera.
educacao-guardado-hoje = ahorrado hoy (cuentas, posiciones y movimientos vinculados).
educacao-historico-vazio = Todavía no hay recálculos.
educacao-revisao-anual = Recálculo anual
educacao-revisao-edicao = Plan editado
educacao-revisao-mensalidade = Mensualidad actualizada
educacao-revisao-criada = Plan creado

## Posiciones vinculadas (cualquier meta)

educacao-posicoes = Posiciones
educacao-posicoes-nome = Posiciones vinculadas a { $nome }
educacao-posicoes-titulo = Posiciones vinculadas · { $nome }
educacao-posicoes-dica = Vincula una cuenta entera (efectivo e inversiones a precio de mercado) o uno de sus activos. Con fracción, la meta se queda con esa parte; sin fracción, el valor pasa a ser un fondo común dividido entre las metas, y quien empieza primero recibe primero.
educacao-posicoes-vazio = Ninguna posición vinculada.
educacao-posicoes-salvas = Posiciones actualizadas
educacao-posicoes-erro = No se pudo vincular la posición
educacao-posicao-conta = Cuenta
educacao-posicao-escolha-conta = Elige la cuenta
educacao-posicao-ativo = Activo
educacao-posicao-conta-inteira = Cuenta entera
educacao-posicao-fracao = Fracción
educacao-posicao-fracao-dica = Vacío: fondo común.
educacao-posicao-ligar = Vincular
educacao-posicao-remover = Desvincular { $nome }
educacao-pote = Fondo común ({ $pct })
educacao-vencimento = Vence el { $vence }
educacao-vencimento-meta = Vence el { $vence }; la meta es para el { $meta }.
educacao-sem-cotacao = Sin cotización: vale por el costo.

## Universidad de los hijos

educacao-visao-titulo = Universidad de los hijos
educacao-visao-aporte = Aporte por mes
educacao-visao-guardado = Ahorrado
educacao-visao-alvo = Necesario al inicio
educacao-visao-linha = Línea de tiempo de las carreras
educacao-visao-curso = Carrera de { $inicio } a { $fim }
educacao-visao-sobreposicao = Dos carreras al mismo tiempo en { $anos }.
educacao-visao-sem-sobreposicao = Ningún año con dos carreras al mismo tiempo.
