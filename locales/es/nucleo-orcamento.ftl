# Presupuesto, metas, jubilación, proyectos y suscripciones (núcleo: `budget`, `goals`,
# `projects`, `subscriptions`): motivos del historial, notas de la simulación de jubilación,
# reglas del INSS y errores. Se arman en el momento, en el idioma de la instalación. Errores en
# minúsculas, como frase de registro (la interfaz pone la inicial en mayúscula). INSS,
# EC 103/2019 y Meu INSS quedan como están.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Presupuesto del mes (`budget`)

nucleo-orcamento-categoria = categoría { $id }
nucleo-orcamento-orcamento-de = presupuesto de { $mes }
nucleo-orcamento-erro-previsto-negativo = el monto previsto no puede ser negativo
nucleo-orcamento-erro-meses-iguais = los meses de origen y destino son iguales
nucleo-orcamento-motivo-atualizado = Presupuesto de { $mes } actualizado
nucleo-orcamento-motivo-copiado = Presupuesto de { $de } copiado a { $para }
nucleo-orcamento-erro-meta-grupo = la meta de cada grupo de gasto va de 0% a 100% de los ingresos

## Suscripciones (`subscriptions`)

nucleo-orcamento-erro-assinaturas-so-interface = descartar suscripciones solo desde la interfaz de la app

## Proyectos (`projects`)

nucleo-orcamento-projeto = proyecto { $id }
nucleo-orcamento-item-projeto = ítem de proyecto { $id }
nucleo-orcamento-bem = bien { $id }
nucleo-orcamento-lancamento = movimiento { $id }
nucleo-orcamento-erro-projeto-sem-nome = ponle un nombre al proyecto
nucleo-orcamento-erro-projeto-negativo = el presupuesto del proyecto no puede ser negativo
nucleo-orcamento-erro-projeto-fim-antes = el proyecto termina antes de empezar
nucleo-orcamento-erro-item-sem-descricao = describe el ítem del proyecto
nucleo-orcamento-erro-item-valor = el monto del ítem tiene que ser mayor que cero (ingreso o gasto viene del tipo)
nucleo-orcamento-motivo-projeto-criado = Proyecto creado: { $nome }
nucleo-orcamento-motivo-projeto-alterado = Proyecto modificado: { $nome }
nucleo-orcamento-motivo-projeto-removido = Proyecto eliminado: { $nome }
nucleo-orcamento-motivo-lancamentos-no-projeto =
    { $n ->
        [one] { $n } movimiento en el proyecto { $nome }
       *[other] { $n } movimientos en el proyecto { $nome }
    }
nucleo-orcamento-motivo-lancamentos-tirados =
    { $n ->
        [one] { $n } movimiento quitado del proyecto
       *[other] { $n } movimientos quitados del proyecto
    }
nucleo-orcamento-motivo-item-adicionado = Ítem en el proyecto { $nome }: { $item }
nucleo-orcamento-motivo-item-alterado = Ítem de proyecto modificado: { $item }
nucleo-orcamento-motivo-entradas-tiradas =
    { $n ->
        [one] { $n } entrada quitada del proyecto
       *[other] { $n } entradas quitadas del proyecto
    }

## Metas (`goals`)

nucleo-orcamento-meta = meta { $id }
nucleo-orcamento-conta = cuenta { $id }
nucleo-orcamento-erro-meta-sem-nome = indica el nombre de la meta
nucleo-orcamento-erro-meta-alvo = el monto objetivo de la meta debe ser positivo
nucleo-orcamento-erro-meta-guardado = el monto ya ahorrado no puede ser negativo
nucleo-orcamento-erro-meta-aporte = el aporte mensual planificado no puede ser negativo
nucleo-orcamento-erro-meta-retorno = el rendimiento real esperado debe estar entre −5% y 15% al año
nucleo-orcamento-erro-meta-arquivada = la meta está archivada
nucleo-orcamento-motivo-meta-criada = Meta creada: { $nome }
nucleo-orcamento-motivo-meta-alterada = Meta modificada: { $nome }
nucleo-orcamento-motivo-meta-fonte-ligada = Fuente vinculada a la meta: { $nome }
nucleo-orcamento-motivo-meta-fonte-desligada = Fuente desvinculada de la meta: { $nome }
nucleo-orcamento-meta-fonte-inicial = Guardado fuera de las cuentas
nucleo-orcamento-meta-fonte-carteira = Cartera (plan de jubilación)
nucleo-orcamento-meta-fonte-lancamentos = Movimientos vinculados
nucleo-orcamento-meta-erro-etiqueta = la etiqueta “{ $etiqueta }” no existe
nucleo-orcamento-meta-erro-ativo-sem-conta = el activo no está en ninguna cuenta; indica la cuenta
nucleo-orcamento-meta-erro-ativo-varias-contas = el activo está en más de una cuenta; indica cuál
nucleo-orcamento-meta-erro-fonte-nao-ligada = esa fuente no está vinculada a la meta
nucleo-orcamento-motivo-meta-removida = Meta eliminada: { $nome }

## Jubilación (`goals::retirement`)

nucleo-orcamento-cenario = escenario { $id }
nucleo-orcamento-erro-idade-hoje = indica la edad de hoy
nucleo-orcamento-erro-idades = edades: hoy entre 16 y 100, jubilación a partir de hoy y esperanza de vida después de la jubilación (hasta 110)
nucleo-orcamento-erro-retorno-volatilidade = rendimiento real entre −20% y 30% anual y volatilidad entre 0 y 60%
nucleo-orcamento-erro-cenario-sem-nome = ponle un nombre al escenario
nucleo-orcamento-erro-cenario-repetido = ya existe un escenario “{ $nome }”
nucleo-orcamento-erro-cenarios-maximo = hasta { $maximo } escenarios; elimina uno antes
nucleo-orcamento-erro-plano-antes-da-meta = guarda el plan de jubilación antes de vincular una meta
nucleo-orcamento-motivo-plano-aposentadoria = Plan de jubilación
nucleo-orcamento-motivo-cenario-salvo = Escenario de jubilación: { $nome }
nucleo-orcamento-motivo-cenario-apagado = Escenario de jubilación eliminado: { $nome }
nucleo-orcamento-motivo-meta-ligada = Meta de jubilación vinculada al plan
nucleo-orcamento-motivo-meta-desligada = Meta de jubilación desvinculada del plan
nucleo-orcamento-aposentadoria-nota-premissas = Valores reales de hoy; rendimiento real de { $retorno } anual con volatilidad de { $volatilidade }, { $simulacoes } simulaciones.
nucleo-orcamento-aposentadoria-nota-patrimonio = Patrimonio de hoy: el valor de mercado de la cartera (incluye planes de pensión privados).
nucleo-orcamento-aposentadoria-nota-inss = { $sistema } calculado por las reglas: { $valor } al mes a partir de los { $idade } años ({ $regra }).
nucleo-orcamento-aposentadoria-regra-escolhida = regla elegida
nucleo-orcamento-aposentadoria-nota-inss-nenhuma = { $sistema } calculado por las reglas: ninguna regla se cumple hasta los 100 años; la simulación sigue sin él.
nucleo-orcamento-aposentadoria-nota-sem-inss = Sin valor de { $sistema } informado: la simulación supone que no hay beneficio público.
# Nombre y nota de la meta creada al vincular el plan (se guardan en el idioma de la instalación).
nucleo-orcamento-aposentadoria-meta = Jubilación
nucleo-orcamento-aposentadoria-meta-nota = Vinculada al plan de jubilación (Presupuesto › Jubilación).

## Estimación del INSS por las reglas de la EC 103/2019 (`goals::inss`)

nucleo-orcamento-inss-regra-permanente = Regla permanente (edad)
nucleo-orcamento-inss-regra-idade = Transición por edad
nucleo-orcamento-inss-regra-pontos = Transición por puntos
nucleo-orcamento-inss-regra-idade-progressiva = Transición por edad mínima progresiva
nucleo-orcamento-inss-regra-pedagio-50 = Transición con peaje del 50%
nucleo-orcamento-inss-regra-pedagio-100 = Transición con peaje del 100%
nucleo-orcamento-inss-req-permanente = { $idade } años de edad y { $contribuicao } de aportes
nucleo-orcamento-inss-req-idade = { $idade } años de edad y 15 de aportes
nucleo-orcamento-inss-req-pontos = { $contribuicao } años de aportes y edad + aportes de { $pontos } puntos en { $ano } (sube 1 por año hasta { $maximo })
# La edad viene en palabras ("64 años y 6 meses").
nucleo-orcamento-inss-req-idade-progressiva = { $contribuicao } años de aportes y { $idade } de edad en { $ano } (sube 6 meses por año hasta { $maximo })
nucleo-orcamento-inss-req-pedagio-50 = más de { $minimo } años de aportes en la reforma; { $contribuicao } años más la mitad de lo que faltaba; beneficio por el factor previsional
nucleo-orcamento-inss-req-pedagio-100 = { $idade } años de edad y { $total } de aportes ({ $contribuicao } más el tiempo que faltaba en la reforma); 100% del promedio
nucleo-orcamento-inss-anos =
    { $anos ->
        [one] { $anos } año
       *[other] { $anos } años
    }
nucleo-orcamento-inss-anos-meses =
    { $meses ->
        [one] { $anos } años y { $meses } mes
       *[other] { $anos } años y { $meses } meses
    }
nucleo-orcamento-inss-nota-estimativa = Estimación por las reglas de la EC 103/2019, en reales de hoy; mínimo y tope de { $ano } ({ $fonte }).
nucleo-orcamento-inss-nota-reforma-suposta = Tiempo de aportes en la reforma supuesto: { $tempo } (aportando sin parar desde entonces).
nucleo-orcamento-inss-nota-so-permanente = Sin aportes antes del 13/11/2019: solo vale la regla permanente.
nucleo-orcamento-inss-nota-teto = El promedio supera el tope: el cálculo usa el tope.
nucleo-orcamento-inss-nota-fora = Fuera del cálculo: actividad especial, docentes, personas con discapacidad, derecho adquirido antes de la reforma y el factor previsional del peaje del 50%. Consúltalo en Meu INSS.
nucleo-orcamento-inss-erro-idade-ou-nascimento = indica la edad de hoy o la fecha de nacimiento
nucleo-orcamento-inss-erro-idade = edad no válida
nucleo-orcamento-inss-erro-nascimento-futuro = fecha de nacimiento en el futuro
nucleo-orcamento-inss-erro-contribuicao = tiempo de aportes entre 0 y la edad menos 12 años
nucleo-orcamento-inss-erro-media-negativa = el promedio de salarios no puede ser negativo
nucleo-orcamento-inss-nota-13 = El INSS paga 13 veces al año (con el aguinaldo anual); el plan suma las 13.
nucleo-orcamento-ssa-erro-pia = informa el PIA (el beneficio a la edad plena del extracto de la SSA) o el AIME
nucleo-orcamento-ssa-regra-antecipada = Solicitud anticipada (62 años)
nucleo-orcamento-ssa-regra-plena = Edad plena (FRA)
nucleo-orcamento-ssa-regra-adiada = Solicitud diferida (70 años)
nucleo-orcamento-ssa-regra-idade-plano = Solicitud a los { $idade } años
nucleo-orcamento-ssa-req-antecipada = desde los 62 años, con 40 créditos; { $corte }% menos que a la edad plena ({ $plena })
nucleo-orcamento-ssa-req-plena = { $plena } de edad, con 40 créditos; 100% del PIA
nucleo-orcamento-ssa-req-adiada = 70 años; { $acrescimo }% más que a la edad plena (8% por año de espera)
nucleo-orcamento-ssa-req-idade-plano = a la edad en que el plan deja de trabajar
nucleo-orcamento-ssa-nota-estimativa = Estimación por la fórmula de la Social Security, en dólares de hoy ({ $fonte }).
nucleo-orcamento-ssa-nota-pia = PIA calculado del AIME: 90% hasta US$ { $bend1 }, 32% hasta US$ { $bend2 } y 15% por encima. El extracto en ssa.gov/myaccount trae el valor exacto.
nucleo-orcamento-ssa-nota-creditos = Sin los 40 créditos hasta la edad en que deja de trabajar: sin derecho al beneficio.
nucleo-orcamento-ssa-nota-fora = Fuera del cálculo: beneficio de cónyuge y de sobreviviente, la prueba de ingresos antes de la edad plena y el impuesto sobre el beneficio.
nucleo-orcamento-pt-regra-normal = Edad personal de jubilación
nucleo-orcamento-pt-regra-flexibilizacao = Anticipación por flexibilización
nucleo-orcamento-pt-regra-carreira-longa = Carrera de cotización muy larga
nucleo-orcamento-pt-req-normal = { $idade } en { $ano } (4 meses menos por año de carrera por encima de 40) y 15 años de cotización; bonificación por mes después, hasta los 70
nucleo-orcamento-pt-req-flexibilizacao = desde los 60 años con 40 de carrera; 0,5% menos por mes antes de la edad personal, y el factor de sostenibilidad si los 40 años no estaban cumplidos a los 60
nucleo-orcamento-pt-req-carreira-longa = desde los 60 años con 48 de carrera (46 si empezó antes de los 17), sin recorte
nucleo-orcamento-pt-nota-estimativa = Estimación por la fórmula de la Seguridad Social portuguesa (régimen general), en euros de hoy; valores de { $ano } ({ $fonte }).
nucleo-orcamento-pt-nota-14 = La pensión se paga 14 veces al año (con las pagas de Navidad y de vacaciones); el plan suma las 14.
nucleo-orcamento-pt-nota-idade-futura = La edad normal de acceso sube con la esperanza de vida a los 65; después de 2027 la app mantiene 66 años y 11 meses.
nucleo-orcamento-pt-nota-antes-2002 = Cotizaciones antes de 2002: la Seguridad Social mezcla la fórmula antigua (mejores 10 de los últimos 15 años), y el valor puede cambiar.
nucleo-orcamento-pt-nota-fora = Fuera del cálculo: regímenes especiales, desempleo de larga duración, pensión unificada y las reglas de 2018 cuando dan más. Consulta el simulador de la Segurança Social Direta.
