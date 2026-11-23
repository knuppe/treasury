# Estudio con IA de un activo (docs/agent-enrichment.md §7 y §8): pestaña Estudio con IA, aviso
# obligatorio, contexto, paquete, historial.
# Revisión: borrador de IA, aún sin revisión de un hablante nativo.

estudo-titulo = Estudio con IA
estudo-aviso-titulo = Generado por IA: puede contener errores
estudo-aviso-texto = Este estudio fue escrito por un modelo de inteligencia artificial a partir de los datos de Treasury y de las fuentes citadas. Puede contener, y probablemente contiene, información incorrecta, desactualizada o inventada, incluso cuando parece segura. Verifica cada dato en las fuentes oficiales antes de usarlo. Es una herramienta de estudio: no es una recomendación, oferta, análisis ni asesoría de inversión, y la decisión de comprar, vender o mantener es tuya.
estudo-aviso-curto = Generado por IA · puede contener errores · verifica las fuentes
estudo-aceite = Entiendo que el estudio puede estar equivocado y que debo verificar la información antes de decidir.
estudo-aceite-continuar = Continuar
estudo-contexto = Contexto para la IA
estudo-contexto-dica = Por qué tengo este activo y qué me haría vender; horizonte y objetivo (renta, crecimiento, protección, reserva); qué me preocupa ahora; qué ya sé o leí.
estudo-contexto-limite = { $n } de { $max } caracteres
estudo-contexto-salvar = Guardar contexto
estudo-contexto-salvo = Contexto guardado
estudo-nao-salvou = No se pudo guardar
estudo-pergunta = Pregunta
estudo-pergunta-dica = ¿Qué cambió desde el último resultado?
estudo-incluir-posicao = Incluir mi peso y ganancia en la cartera (%)
estudo-incluir-valores = Incluir cantidad, precio medio y montos en dinero
estudo-motor = Motor: { $motor }
estudo-sem-motor = Ningún motor de IA configurado. Registra uno en Ajustes › IA.
estudo-o-que-vai = Qué va a { $provedor }
estudo-o-motor = el motor
estudo-pacote = Paquete del estudio
estudo-pacote-local = Modelo local: el paquete se queda en el ordenador.
estudo-pacote-tamanho = { $n } caracteres, exactamente como abajo.
estudo-gerar = Generar estudio
estudo-gerando = Generando…
estudo-pronto = Estudio listo: está arriba de los estudios anteriores.
estudo-nao-gerou = No se pudo generar el estudio
estudo-anteriores = Estudios anteriores
estudo-nenhum = Ningún estudio todavía.
estudo-abrir = Abrir
estudo-abrindo = Abriendo el estudio
estudo-aberto = Estudio abierto
estudo-fechar = Cerrar
estudo-apagar = Eliminar
estudo-apagado = Estudio eliminado
estudo-copiar = Copiar
estudo-copiado = Estudio copiado, con el aviso al final
estudo-continuar-chat = Continuar en el chat
estudo-chat-pedido = Quiero conversar sobre el estudio con IA del { $data } de este activo (estudio { $id }). Recuerda que fue generado por IA y puede contener errores.
estudo-selo = Generado por IA el { $data } con { $motor } · puede contener errores
estudo-confianca = confianza { $valor }
estudo-confianca-alta = alta
estudo-confianca-media = media
estudo-confianca-baixa = baja
estudo-confianca-sem = no informada
estudo-nao-conferidos =
    { $n ->
        [one] { $n } número sin verificar
       *[other] { $n } números sin verificar
    }
estudo-resumo = Resumen
estudo-negocio = Qué hace
estudo-fortes = Puntos fuertes
estudo-riscos = Riesgos
estudo-numeros = Números clave
estudo-numero-ok = coincide con los datos
estudo-numero-diferente = no coincide: el dato es { $valor }
estudo-numero-diferente-sem = no coincide con los datos
estudo-numero-nao-encontrado = no encontrado en los datos
estudo-mudou = Qué cambió
estudo-seu-contexto = Tu contexto
estudo-conferir = Qué verificar a mano
estudo-fontes = Fuentes
estudo-faltou = Faltó: { $texto }
