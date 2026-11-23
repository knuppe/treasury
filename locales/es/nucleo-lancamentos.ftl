# Movimientos, recurrencias, reglas y categorías (núcleo: `transactions`, `recurrence`, `rules`,
# `categories`): errores de validación, motivos del historial, motivos de las sugerencias y
# etiquetas que el núcleo arma en el momento, en el idioma de la instalación. Los nombres de las
# categorías predeterminadas están en categorias.ftl; los textos que se comparan con el extracto
# del banco (reglas predefinidas) quedan en el código.
# Revisión: borrador por IA, aún sin revisar por un hablante nativo.

## Común

nucleo-lancamentos-sem-descricao = (sin descripción)
nucleo-lancamentos-ref-lancamento = movimiento { $id }
nucleo-lancamentos-ref-membro = miembro { $id }
nucleo-lancamentos-ref-tag = etiqueta { $id }
nucleo-lancamentos-ref-anexo = adjunto { $id }
nucleo-lancamentos-erro-nome-vazio = { $coisa ->
    [conta] escribe el nombre de la cuenta
    [categoria] escribe el nombre de la categoría
    [tag] escribe el nombre de la etiqueta
    [membro] escribe el nombre del miembro
    [familia] escribe el nombre de la familia
    [instituicao] escribe el nombre de la institución
    [renda] escribe el nombre del ingreso
   *[outro] escribe el nombre
}
nucleo-lancamentos-erro-nome-longo = { $coisa ->
    [conta] el nombre de la cuenta puede tener como máximo { $max } caracteres
    [categoria] el nombre de la categoría puede tener como máximo { $max } caracteres
    [tag] el nombre de la etiqueta puede tener como máximo { $max } caracteres
    [membro] el nombre del miembro puede tener como máximo { $max } caracteres
    [familia] el nombre de la familia puede tener como máximo { $max } caracteres
    [instituicao] el nombre de la institución puede tener como máximo { $max } caracteres
    [renda] el nombre del ingreso puede tener como máximo { $max } caracteres
   *[outro] el nombre puede tener como máximo { $max } caracteres
}
nucleo-lancamentos-erro-cor = color no válido: { $cor }
nucleo-lancamentos-erro-periodo-invertido = la fecha inicial es posterior a la fecha final
nucleo-lancamentos-erro-maximo-lote = como máximo { $max } movimientos a la vez

## Tipos y estados (en minúscula, para la mitad de la frase)

nucleo-lancamentos-tipo-receita = ingreso
nucleo-lancamentos-tipo-despesa = gasto
nucleo-lancamentos-tipo-transferencia = transferencia
nucleo-lancamentos-tipo-pagamento-fatura = pago de tarjeta
nucleo-lancamentos-tipo-investimento = aporte/rescate de inversión
nucleo-lancamentos-tipo-ajuste = ajuste
nucleo-lancamentos-status-previsto = previsto
nucleo-lancamentos-status-efetivado = efectivo
nucleo-lancamentos-status-conciliado = conciliado

## Historial (motivo de cada cambio)

nucleo-lancamentos-historico-criado = Movimiento creado: { $descricao }
nucleo-lancamentos-historico-alterado = Movimiento modificado: { $descricao }
nucleo-lancamentos-historico-apagado = Movimiento eliminado: { $descricao }
nucleo-lancamentos-historico-apagados = { $n } movimientos eliminados
nucleo-lancamentos-historico-copias-apagadas = { $n } copias entre cuentas eliminadas
nucleo-lancamentos-historico-transferencia = Transferencia: { $origem } → { $destino }
nucleo-lancamentos-historico-pagamento-fatura = Pago de tarjeta: { $origem } → { $destino }
nucleo-lancamentos-historico-aporte = Aporte: { $origem } → { $destino }
nucleo-lancamentos-historico-categoria-definida = { $n ->
    [one] Categoría { $categoria } asignada a { $n } movimiento
   *[other] Categoría { $categoria } asignada a { $n } movimientos
}
nucleo-lancamentos-historico-categoria-removida = { $n ->
    [one] Categoría quitada de { $n } movimiento
   *[other] Categoría quitada de { $n } movimientos
}
nucleo-lancamentos-historico-status = { $n ->
    [one] Estado cambiado a { $status } en { $n } movimiento
   *[other] Estado cambiado a { $status } en { $n } movimientos
}
nucleo-lancamentos-historico-membro-definido = { $n ->
    [one] Miembro { $membro } asignado a { $n } movimiento
   *[other] Miembro { $membro } asignado a { $n } movimientos
}
nucleo-lancamentos-historico-membro-removido = { $n ->
    [one] Miembro quitado de { $n } movimiento
   *[other] Miembro quitado de { $n } movimientos
}
nucleo-lancamentos-historico-tags = { $n ->
    [one] Etiquetas cambiadas en { $n } movimiento
   *[other] Etiquetas cambiadas en { $n } movimientos
}
nucleo-lancamentos-historico-par-ligado = Movimientos vinculados como transferencia entre cuentas
nucleo-lancamentos-historico-pares-ligados = { $n } pares vinculados como transferencias entre cuentas
nucleo-lancamentos-historico-vinculados = { $n ->
    [one] { $n } movimiento vinculado a { $operacao }
   *[other] { $n } movimientos vinculados a { $operacao }
}
nucleo-lancamentos-historico-vinculado = Movimiento vinculado: { $descricao } → { $operacao }
nucleo-lancamentos-historico-desvinculado = Vínculo deshecho: { $descricao }
nucleo-lancamentos-historico-transferencia-desligada = Transferencia desvinculada: las dos partes quedaron sueltas
nucleo-lancamentos-historico-tag-criada = Etiqueta creada: { $nome }
nucleo-lancamentos-historico-tag-alterada = Etiqueta modificada
nucleo-lancamentos-historico-tag-apagada = Etiqueta eliminada: { $nome }
nucleo-lancamentos-historico-anexo-adicionado = Adjunto agregado: { $descricao }
nucleo-lancamentos-historico-anexo-apagado = Adjunto eliminado: { $descricao }

## Movimientos

nucleo-lancamentos-erro-nenhum-selecionado = ningún movimiento seleccionado
nucleo-lancamentos-erro-escolha-um = elige al menos un movimiento
nucleo-lancamentos-erro-informe-lancamento = indica el movimiento
nucleo-lancamentos-erro-tags-lote = indica las etiquetas que quieres agregar o quitar
nucleo-lancamentos-erro-tipo-invalido = tipo de movimiento no válido: { $valor }
nucleo-lancamentos-erro-status-invalido = estado no válido: { $valor }
nucleo-lancamentos-erro-valor-zero = el importe del movimiento no puede ser cero
nucleo-lancamentos-erro-categoria-arquivada = la categoría { $nome } está archivada
nucleo-lancamentos-erro-tipo-sem-categoria = los movimientos de tipo { $tipo } no usan categoría
nucleo-lancamentos-erro-categoria-de-outro-tipo = la categoría { $nome } es de { $categoria ->
    [receita] ingresos
   *[despesa] gastos
} y el movimiento es un { $lancamento ->
    [receita] ingreso
   *[despesa] gasto
}
nucleo-lancamentos-erro-tipo-sem-divisao = los movimientos de tipo { $tipo } no se pueden dividir
nucleo-lancamentos-erro-lote-sem-categoria =
    { $n ->
        [one] aporte, pago de tarjeta y ajuste no usan categoría: mueven el patrimonio y quedan fuera del flujo de caja y del presupuesto ({ $n } marcado)
       *[other] aporte, pago de tarjeta y ajuste no usan categoría: mueven el patrimonio y quedan fuera del flujo de caja y del presupuesto ({ $n } marcados)
    }
nucleo-lancamentos-erro-divisao-partes = una división necesita al menos dos partes
nucleo-lancamentos-erro-divisao-soma = la suma de las partes ({ $soma }) es distinta del total ({ $total })
nucleo-lancamentos-erro-divisao-sinal = las partes de la división deben tener el mismo signo que el total y no pueden ser cero
nucleo-lancamentos-erro-divisao-desatualizada = { $erro }; envía la división actualizada junto (o una lista vacía para quitar la división)
nucleo-lancamentos-erro-pagamento-duas-pernas = los pagos de tarjeta tienen dos partes; usa transactions.create_transfer
nucleo-lancamentos-erro-conciliado = este movimiento está conciliado; desconcílialo (cambia el estado a efectivo) antes de cambiar el importe, la fecha o la cuenta
nucleo-lancamentos-erro-outra-perna-conciliada-previsto = la otra parte de la transferencia está conciliada; desconcíliala antes de volver a previsto
nucleo-lancamentos-erro-outra-perna-conciliada = la otra parte de la transferencia está conciliada; desconcíliala antes de cambiar el importe o la fecha
nucleo-lancamentos-erro-apagar-conciliado = el movimiento "{ $descricao }" del { $data } está conciliado; desconcílialo antes de eliminarlo
nucleo-lancamentos-erro-conta-outra-moeda = la cuenta elegida usa { $moeda_conta } y el movimiento está en { $moeda }; crea un movimiento nuevo en esa cuenta
nucleo-lancamentos-erro-pagamento-destino-cartao = el destino de un pago de tarjeta tiene que ser una cuenta de tarjeta
nucleo-lancamentos-erro-perna-mudar-tipo = no se puede cambiar el tipo de una parte de transferencia; elimina la transferencia y crea otro movimiento
nucleo-lancamentos-erro-perna-sinal = en una transferencia la parte de origen es negativa y la de destino es positiva; mantén el signo
nucleo-lancamentos-erro-pernas-mesma-conta = las dos partes de la transferencia no pueden quedar en la misma cuenta
nucleo-lancamentos-erro-transferencia-dividida = las transferencias no se pueden dividir
nucleo-lancamentos-erro-virar-pagamento-fatura = para convertirlo en pago de tarjeta, vincúlalo con la parte de la tarjeta (transferencias sugeridas)
nucleo-lancamentos-erro-ja-ligado = este movimiento ya está vinculado a otra cuenta
nucleo-lancamentos-erro-recusar-so-interface = las sugerencias de transferencia solo se descartan en la app

## Etiquetas y adjuntos

nucleo-lancamentos-erro-tag-repetida = ya existe una etiqueta llamada { $nome }
nucleo-lancamentos-erro-base64 = contenido base64 no válido
nucleo-lancamentos-erro-arquivo-nome = indica el nombre del archivo
nucleo-lancamentos-erro-arquivo-nome-longo = nombre de archivo demasiado largo
nucleo-lancamentos-erro-arquivo-tipo = tipo de archivo no válido: { $tipo }
nucleo-lancamentos-erro-anexo-grande = el adjunto supera el límite de { $mb } MB
nucleo-lancamentos-erro-arquivo-vazio = el archivo está vacío

## Transferencias

# Descripciones predeterminadas de una transferencia nueva (se guardan como la descripción del movimiento).
nucleo-lancamentos-descricao-pagamento-fatura = Pago de la tarjeta { $conta }
nucleo-lancamentos-descricao-pagamento-recebido = Pago recibido de { $conta }
nucleo-lancamentos-descricao-aporte-em = Aporte en { $conta }
nucleo-lancamentos-descricao-aporte-vindo = Aporte desde { $conta }
nucleo-lancamentos-descricao-transferencia-para = Transferencia a { $conta }
nucleo-lancamentos-descricao-transferencia-de = Transferencia de { $conta }
nucleo-lancamentos-erro-transferencia-mesma-conta = la cuenta de origen y la de destino deben ser distintas
nucleo-lancamentos-erro-transferencia-valor = indica el importe de la transferencia como un número positivo
nucleo-lancamentos-erro-transferencia-mesma-moeda = cuentas en la misma moneda: el importe que entra debe ser igual al que sale
nucleo-lancamentos-erro-transferencia-outra-moeda = el origen está en { $moeda_origem } y el destino en { $moeda_destino }; indica el importe que entra en el destino (to_amount)
nucleo-lancamentos-erro-pagamento-so-cartao = el pago de tarjeta solo puede tener una cuenta de tarjeta como destino
nucleo-lancamentos-erro-pagamento-de-cartao = la tarjeta se debe pagar desde una cuenta que no sea tarjeta
nucleo-lancamentos-erro-transferencia-tipo = las transferencias no pueden ser de tipo { $tipo }
# Por qué un par aparece en las transferencias sugeridas (unidos con "; ").
nucleo-lancamentos-motivo-mesmo-valor-mesmo-dia = mismo importe el mismo día
nucleo-lancamentos-motivo-mesmo-valor-dias = { $dias ->
    [one] mismo importe con { $dias } día de diferencia
   *[other] mismo importe con { $dias } días de diferencia
}
nucleo-lancamentos-motivo-texto-transferencia = texto de transferencia (PIX, TED…)
nucleo-lancamentos-motivo-nome-outra-conta = el nombre de la otra cuenta aparece en la descripción
nucleo-lancamentos-erro-par-ja-transferencia = uno de los movimientos ya forma parte de una transferencia
nucleo-lancamentos-erro-par-tipo = solo los ingresos, gastos y transferencias sueltas se convierten en transferencia
nucleo-lancamentos-erro-par-mesma-conta = las dos partes tienen que estar en cuentas distintas
nucleo-lancamentos-erro-par-sinal = la parte de origen tiene que ser una salida y la de destino una entrada
nucleo-lancamentos-erro-par-valor = cuentas en la misma moneda: el importe que sale tiene que ser igual al que entra
nucleo-lancamentos-erro-par-dividido = los movimientos divididos no se convierten en transferencia
nucleo-lancamentos-erro-outra-conta = elige una cuenta distinta de la del movimiento
nucleo-lancamentos-erro-outra-conta-moeda = la otra cuenta usa { $moeda }: indica cuánto entró (o salió) en ella
nucleo-lancamentos-erro-nao-ligado = este movimiento no está vinculado a otra cuenta
nucleo-lancamentos-erro-ligacao-historico = este vínculo ({ $tipo }) no se deshace aquí; usa el historial para deshacerlo
nucleo-lancamentos-erro-nenhum-par = ningún par seleccionado

## Vínculo con operaciones (cartera, aportaciones de capital, metas)

nucleo-lancamentos-operacao = operación
nucleo-lancamentos-operacao-apagada = operación eliminada
nucleo-lancamentos-operacao-aporte = Aporte en { $ativo } (compra de { $quantidade } el { $data })
nucleo-lancamentos-operacao-resgate = Rescate en { $ativo } (venta de { $quantidade } el { $data })
nucleo-lancamentos-operacao-cotas-capital = Aportaciones de capital · { $conta }
nucleo-lancamentos-operacao-meta = Meta { $nome }
nucleo-lancamentos-ref-operacao-carteira = operación de la cartera
nucleo-lancamentos-ref-cotas-capital = aportaciones de capital en esa cuenta
nucleo-lancamentos-ref-meta = meta
# Por qué la operación aparece como candidata.
nucleo-lancamentos-motivo-operacao = { $valor ->
    [igual] mismo importe
   *[parecido] importe parecido
} { $dias ->
    [0] el mismo día
    [one] a { $dias } día
   *[other] a { $dias } días
}; { $diferenca } de diferencia
nucleo-lancamentos-motivo-texto-cota-capital = el texto del extracto habla de aportaciones de capital
nucleo-lancamentos-motivo-conta-cotas-capital = la cuenta tiene aportaciones de capital
nucleo-lancamentos-motivo-texto-meta = el texto del extracto menciona la meta
nucleo-lancamentos-motivo-poupanca-programada = el texto habla de ahorro programado
nucleo-lancamentos-motivo-meta-ativa = meta activa
nucleo-lancamentos-motivo-combina-com = { $motivo }; coincide con { $n } de { $total }
nucleo-lancamentos-erro-vinculado = el movimiento está vinculado a { $operacao }; desvincúlalo antes
nucleo-lancamentos-erro-ja-vinculado = el movimiento ya está vinculado a { $operacao }; desvincúlalo antes
nucleo-lancamentos-erro-sem-valor = movimiento sin importe
nucleo-lancamentos-erro-vincular-previsto = el movimiento todavía está previsto; hazlo efectivo antes de vincularlo
nucleo-lancamentos-erro-vincular-transferencia = el movimiento es una parte de transferencia; desvincula la transferencia antes
nucleo-lancamentos-erro-vincular-tipo = solo los ingresos, gastos, transferencias sueltas y aportes se vinculan a una operación
nucleo-lancamentos-erro-vincular-dividido = los movimientos divididos no se vinculan a una operación
nucleo-lancamentos-erro-vincular-parcela = el movimiento paga una cuota de { $divida }; desvincúlalo de allí antes
nucleo-lancamentos-erro-vincular-caixa-corretora = el movimiento ya es la caja de esa operación en el bróker
nucleo-lancamentos-erro-operacao-ocupada = esa operación ya tiene un movimiento vinculado; desvincula el otro antes
nucleo-lancamentos-erro-aporte-saida = un aporte (compra) se paga con una salida (importe negativo)
nucleo-lancamentos-erro-resgate-entrada = un rescate (venta) llega como una entrada (importe positivo)
nucleo-lancamentos-erro-meta-arquivada = la meta está archivada
nucleo-lancamentos-erro-vinculo-nao-gravado = vínculo no guardado
nucleo-lancamentos-erro-nao-vinculado = el movimiento no está vinculado a una operación
nucleo-lancamentos-erro-operacao-um-lancamento = la operación de la cartera acepta un solo movimiento
nucleo-lancamentos-erro-nenhum-vinculavel = ninguno de los movimientos se puede vincular a { $operacao } ({ $motivo })

## Recurrencias

nucleo-lancamentos-ref-recorrencia = recurrencia { $id }
nucleo-lancamentos-historico-recorrencia-criada = Recurrencia creada: { $descricao }
nucleo-lancamentos-historico-recorrencia-alterada = Recurrencia modificada: { $descricao }
nucleo-lancamentos-historico-recorrencia-apagada = Recurrencia eliminada: { $descricao }
nucleo-lancamentos-historico-previstos = Movimientos previstos de recurrencias
nucleo-lancamentos-historico-parcelas-efetivadas = Cuotas de la tarjeta hechas efectivas
# Quién hizo el cambio en el historial ("Sistema · recurrencias").
nucleo-lancamentos-autor-recorrencias = recurrencias
nucleo-lancamentos-autor-cartoes = tarjetas
nucleo-lancamentos-recorrencia-erro-horizonte = el horizonte debe estar entre 0 y { $max } días
nucleo-lancamentos-recorrencia-erro-valor-zero = el importe de la recurrencia no puede ser cero
nucleo-lancamentos-recorrencia-erro-destino-so-transferencia = la cuenta de destino solo vale para transferencias recurrentes
nucleo-lancamentos-recorrencia-erro-informe-destino = indica la cuenta de destino de la transferencia recurrente
nucleo-lancamentos-recorrencia-erro-moedas = las transferencias recurrentes entre monedas distintas ({ $origem } → { $destino }) no son compatibles
nucleo-lancamentos-recorrencia-erro-pagamento-cartao = un pago de tarjeta recurrente necesita una tarjeta como destino
nucleo-lancamentos-recorrencia-erro-transferencia-categoria = las transferencias no usan categoría
nucleo-lancamentos-recorrencia-erro-ajuste = los ajustes no pueden ser recurrentes
nucleo-lancamentos-recorrencia-erro-descricao = indica la descripción de la recurrencia
nucleo-lancamentos-recorrencia-erro-fim-antes = la fecha final es anterior a la fecha inicial
nucleo-lancamentos-recorrencia-erro-sem-destino = transferencia recurrente sin cuenta de destino

## Regla de repetición (RRULE)

nucleo-lancamentos-rrule-erro-byday-posicao = BYDAY con posición ({ $valor }) no es compatible; usa solo el día de la semana (p. ej.: MO)
nucleo-lancamentos-rrule-erro-dia-semana = día de la semana no válido: { $valor }
nucleo-lancamentos-rrule-erro-valor = valor no válido en { $parte }: { $valor }
nucleo-lancamentos-rrule-erro-parte = parte no válida en la regla: { $parte }
nucleo-lancamentos-rrule-erro-frequencia = frecuencia no compatible: { $valor } (usa DAILY, WEEKLY, MONTHLY o YEARLY)
nucleo-lancamentos-rrule-erro-varios-dias = BYMONTHDAY con varios días no es compatible; crea una recurrencia por día
nucleo-lancamentos-rrule-erro-varios-meses = BYMONTH con varios meses no es compatible; crea una recurrencia por mes
nucleo-lancamentos-rrule-erro-parte-nao-suportada = parte de la regla no compatible: { $parte } (el fin de la recurrencia es la fecha final)
nucleo-lancamentos-rrule-erro-sem-freq = la regla necesita FREQ
nucleo-lancamentos-rrule-erro-intervalo = el intervalo debe estar entre 1 y { $max }
nucleo-lancamentos-rrule-erro-ocorrencias = el número de repeticiones debe estar entre 1 y { $max }
nucleo-lancamentos-rrule-erro-dia-mes = el día del mes debe estar entre 1 y 31
nucleo-lancamentos-rrule-erro-dia-mes-frequencia = el día del mes solo vale para recurrencias mensuales o anuales
nucleo-lancamentos-rrule-erro-mes = el mes debe estar entre 1 y 12
nucleo-lancamentos-rrule-erro-mes-frequencia = el mes solo vale para recurrencias anuales
nucleo-lancamentos-rrule-erro-sem-dias = elige al menos un día de la semana
nucleo-lancamentos-rrule-erro-dias-frequencia = los días de la semana solo valen para recurrencias diarias o semanales
# Descripción legible: "todos los meses el día 5", "cada 2 semanas los lunes, 3 veces". El
# género solo importa en portugués.
nucleo-lancamentos-rrule-dia-mo = lunes
nucleo-lancamentos-rrule-dia-tu = martes
nucleo-lancamentos-rrule-dia-we = miércoles
nucleo-lancamentos-rrule-dia-th = jueves
nucleo-lancamentos-rrule-dia-fr = viernes
nucleo-lancamentos-rrule-dia-sa = sábados
nucleo-lancamentos-rrule-dia-su = domingos
nucleo-lancamentos-rrule-lista = { $inicio } y { $ultimo }
nucleo-lancamentos-rrule-dia-com-artigo = { $genero ->
    [f] los { $dia }
   *[m] los { $dia }
}
nucleo-lancamentos-rrule-dias = { $genero ->
    [f] los { $lista }
    [m] los { $lista }
   *[misto] { $lista }
}
nucleo-lancamentos-rrule-diaria = { $n ->
    [1] todos los días
   *[other] cada { $n } días
}
nucleo-lancamentos-rrule-semanal = { $n ->
    [1] todas las semanas
   *[other] cada { $n } semanas
}
nucleo-lancamentos-rrule-mensal = { $n ->
    [1] todos los meses
   *[other] cada { $n } meses
}
nucleo-lancamentos-rrule-anual = { $n ->
    [1] todos los años
   *[other] cada { $n } años
}
nucleo-lancamentos-rrule-dia-util = todos los días hábiles (de lunes a viernes)
nucleo-lancamentos-rrule-so-dias = { $base }, solo { $dias }
nucleo-lancamentos-rrule-nos-dias = { $base } { $dias }
nucleo-lancamentos-rrule-ultimo-dia = { $base } el último día
nucleo-lancamentos-rrule-no-dia = { $base } el día { $dia }
nucleo-lancamentos-rrule-em-data = { $base } el { $dia } de { $mes }
nucleo-lancamentos-rrule-em-mes = { $base } en { $mes }
nucleo-lancamentos-rrule-ultimo-dia-mes = { $base } el último día del mes
nucleo-lancamentos-rrule-vezes = { $n ->
    [1] { $regra }, una vez
   *[other] { $regra }, { $n } veces
}
nucleo-lancamentos-rrule-ate = { $regra }, hasta el { $data }

## Reglas de categorización

nucleo-lancamentos-ref-regra = regla { $id }
nucleo-lancamentos-historico-regra-criada = Regla creada: { $nome }
nucleo-lancamentos-historico-regra-alterada = Regla modificada: { $nome }
nucleo-lancamentos-historico-regra-excluida = Regla eliminada: { $nome }
nucleo-lancamentos-historico-regras-aplicadas = Aplicación de reglas de categorización
nucleo-lancamentos-historico-regras-prontas = Reglas predefinidas de categoría (iFood, Netflix, Uber…)
nucleo-lancamentos-historico-regras-prontas-pix = PIX, TED y DOC pasan a ser transferencias
nucleo-lancamentos-historico-regras-prontas-restaurantes = Reglas predefinidas: restaurantes y créditos de vales de alimentación y comida
nucleo-lancamentos-historico-regras-prontas-encargos = Reglas predefinidas: intereses y cargos de la tarjeta
nucleo-lancamentos-autor-regras-prontas = reglas predefinidas
# Nombre de la regla sugerida a partir de un movimiento (se vuelve el nombre de la regla, si se guarda).
nucleo-lancamentos-regra-nome-sugerido = La descripción contiene “{ $trecho }”
nucleo-lancamentos-regra-erro-texto = la condición necesita un texto
nucleo-lancamentos-regra-erro-centavos = el importe de la condición debe ser entero, en centavos (p. ej.: -5000 = { $exemplo })
nucleo-lancamentos-regra-erro-numero = la condición de importe necesita un número
nucleo-lancamentos-regra-erro-regex = expresión regular no válida: { $erro }
nucleo-lancamentos-regra-erro-texto-vazio = el texto de la condición necesita letras o números
nucleo-lancamentos-regra-erro-op-texto = las condiciones de texto usan contiene, igual, empieza con, termina con o regex
nucleo-lancamentos-regra-erro-entre = la condición "entre" necesita value2
nucleo-lancamentos-regra-erro-op-valor = las condiciones de importe usan igual, entre, mayor o igual, menor o igual
nucleo-lancamentos-regra-erro-op-conta = la condición de cuenta usa "igual" con el id de la cuenta
nucleo-lancamentos-regra-erro-op-tipo = la condición de tipo usa "igual" con el tipo del movimiento
nucleo-lancamentos-regra-erro-sem-condicao = la regla necesita al menos una condición
nucleo-lancamentos-regra-erro-nome = ponle un nombre a la regla
nucleo-lancamentos-regra-erro-sem-acao = la regla necesita al menos una acción (categoría, miembro, etiquetas, renombrar o tipo)
nucleo-lancamentos-regra-erro-renomear = el nuevo nombre no puede quedar vacío
nucleo-lancamentos-regra-erro-tags = las etiquetas no pueden quedar vacías
nucleo-lancamentos-regra-erro-sem-palavras = la descripción del movimiento no tiene palabras para armar una regla

## Categorías

nucleo-lancamentos-ref-categoria = categoría { $id }
nucleo-lancamentos-ref-categoria-pai = categoría superior { $id }
nucleo-lancamentos-historico-categorias-traduzidas = Categorías predeterminadas traducidas
nucleo-lancamentos-historico-categorias-sugeridas = Categorías sugeridas agregadas
nucleo-lancamentos-historico-categoria-criada = Categoría creada: { $nome }
nucleo-lancamentos-historico-categoria-alterada = Categoría modificada
nucleo-lancamentos-historico-categoria-apagada = Categoría eliminada: { $nome }
nucleo-lancamentos-historico-categoria-apagada-reatribuida = Categoría eliminada con reasignación: { $nome }
nucleo-lancamentos-historico-categorias-juntadas = Categorías unidas: { $origem } → { $destino }
nucleo-lancamentos-historico-categorias-grupo = { $n ->
        [one] 1 categoría pasó al grupo { $grupo }
       *[other] { $n } categorías pasaron al grupo { $grupo }
    }
nucleo-lancamentos-categoria-erro-repetida = ya existe una categoría llamada { $nome } en este nivel
nucleo-lancamentos-categoria-erro-tipo-pai = la subcategoría debe ser del mismo tipo que la categoría superior ({ $pai } es de { $tipo ->
    [receita] ingresos
   *[despesa] gastos
})
nucleo-lancamentos-categoria-erro-ciclo = una categoría no puede quedar dentro de sí misma ni de una de sus subcategorías
nucleo-lancamentos-categoria-erro-grupo-receita = las categorías de ingresos no tienen grupo de gasto (esenciales, deudas, futuro, no esencial)
nucleo-lancamentos-categoria-erro-mesmo-destino = elige una categoría distinta para recibir los movimientos
nucleo-lancamentos-categoria-erro-juntar-tipos = { $origem } es de { $tipo_origem ->
    [receita] ingresos
   *[despesa] gastos
} y { $destino } es de { $tipo_destino ->
    [receita] ingresos
   *[despesa] gastos
}; solo se pueden unir categorías del mismo tipo
nucleo-lancamentos-categoria-erro-juntar-sub = { $destino } es subcategoría de { $origem }; elige una categoría fuera de ella
nucleo-lancamentos-categoria-erro-tem-subcategorias = { $n ->
    [one] { $nome } tiene { $n } subcategoría; elimínala, muévela o únela antes
   *[other] { $nome } tiene { $n } subcategorías; elimínalas, muévelas o únelas antes
}
nucleo-lancamentos-categoria-erro-em-uso = { $nome } está en uso ({ $lancamentos } movimiento(s), { $divisoes } división(es), { $orcamento } línea(s) de presupuesto, { $regras } regla(s), { $recorrencias } recurrencia(s), { $parcelamentos } plan(es) de cuotas); elige una categoría para recibir ese uso
nucleo-lancamentos-erro-anexo-so-interface = guardar un adjunto en el disco solo desde la interfaz de la app
nucleo-lancamentos-erro-copias-so-interface = eliminar copias entre cuentas solo desde la interfaz de la app
nucleo-lancamentos-erro-origem-mesma-moeda = la moneda de origen es la misma de la cuenta; deja vacío el monto de origen
nucleo-lancamentos-erro-origem-valor = el monto en la moneda de origen no puede ser cero
nucleo-lancamentos-erro-origem-par = indica el monto y la moneda de origen juntos
