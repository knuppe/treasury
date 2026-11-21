# Score de salud por activo: nota, tramos, dimensiones, métricas, red flags, eventos, historial y
# opinión (docs/i18n-design.md). Las explicaciones y notas que vienen del núcleo no pasan por aquí.
# Revisión: borrador por IA, sin revisión de un hablante nativo.

## Tramos, situación y severidad

score-faixa-robusto = Robusto
score-faixa-saudavel = Saludable
score-faixa-atencao = Atención
score-faixa-risco = Riesgo
score-dados-insuficientes = Datos insuficientes
score-status-calculado = Calculado
score-status-parcial = Parcial
score-status-nao-suportada = Clase no admitida
score-severidade-informativa = Informativa
score-severidade-baixa = Baja
score-severidade-media = Media
score-severidade-alta = Alta
score-severidade-critica = Crítica

## Valores de las métricas

score-dias = { $n ->
    [one] { $valor } día
   *[other] { $valor } días
}
score-anos = { $n ->
    [one] { $valor } año
   *[other] { $valor } años
}
score-percentil-pares = { $n ->
    [one] { $p } de { $n } par
   *[other] { $p } de { $n } pares
}
score-teto = tope { $nota }
score-penalidade = { $n ->
    [one] { $pontos } punto
   *[other] { $pontos } puntos
}

## Piezas: sello, barra y regla

score-chip-parcial = {" "}· parcial
score-barra-sem-nota = { $rotulo }: sin nota
score-barra-nota = { $rotulo }: { $nota } de 100
score-faixas-regua = Tramos de 0 a 100: { $faixas }

## Pestaña Salud: encabezado

score-segmento = Segmento
score-segmento-metodologia = Segmento de la metodología
score-segmento-automatico = Segmento automático
score-segmento-trocado = Segmento cambiado
score-segmento-refeito = { $ticker }: el score se recalculó con los pesos del nuevo segmento.
score-segmento-auto-setor = Automático (por el sector)
score-escolhido-por-voce = Elegido por ti ·{" "}
score-titulo = Score de salud del activo
score-metodologia = Metodología
score-avaliado-em = Evaluado el
score-dados-de = Datos de
score-defasado = desactualizado
score-defasado-dica = Los datos más recientes tienen más de 2 trimestres
score-nota = Score
score-nota-parcial = Score (parcial)
score-pontos = { $delta } puntos
score-mudanca = { $pontos } desde el { $data } (era { $nota })
score-mudanca-faixa = { $pontos } desde el { $data } (era { $nota }, { $faixa })
score-confianca = Confianza
score-saude = Salud
score-saude-dica = sin valoración
score-atratividade = Atractivo
score-atratividade-dica = precio
score-pares = Pares
score-pares-setor = { $setor } en el sector { $nome } · { $segmento } en el segmento · { $classe } en la clase
score-pares-resumo = { $setor } en el sector · { $segmento } en el segmento · { $classe } en la clase
score-alertas = Alertas del score

## Pestaña Salud: dato crítico ausente y motivos

score-nao-calculado = Score no calculado: falta un dato crítico
score-criticos-ausentes = Datos críticos ausentes
score-critica = crítica
score-tambem-sem-dado = También sin dato: { $lista }.
score-sem-metricas = Sin estas métricas la nota no se calcula (nada se inventa). El caso quedó registrado para el desarrollador, en 15 · Desarrollador, para revisar la recolección. Las dimensiones de abajo muestran lo que ya se midió.
score-por-que = Por qué esta nota
score-a-favor = A favor
score-contra = En contra
score-red-flags = Red flags
score-gravidade = gravedad { $nivel }
score-antes-flags = Antes de las red flags el score era { $nota }.
score-sem-dado-redistribuido = Sin dato (el peso se redistribuyó en la dimensión): { $lista }.

## Pestaña Salud: modelos, verificación y avisos

score-piotroski = { $valor } de { $total }
score-itens = Ítems · { $modelo }
score-atende = cumple
score-nao-atende = no cumple
score-metrica = Métrica
score-fonte = Fuente
score-diferenca = Diferencia
score-conferido-com = Verificado con
score-calculando = Calculando el score
score-notas-calculo = Notas del cálculo
score-cripto-fontes = Fuentes de los datos de criptoactivos
score-cripto-fontes-texto = Datos proporcionados por:
score-indisponivel = Score de salud todavía no disponible para { $classe }
score-cripto-titulo = Solidez dentro de la clase
score-cripto-texto = Las cripto no se comparan directamente con acciones: el score mide la solidez dentro de la clase de criptoactivos.
score-parcial-titulo = Score parcial: sin tramo definitivo
score-parcial-texto = El { $pct }% del peso vino de dimensiones imputadas (mediana de los pares × 0,9) por falta de datos. El número se muestra, pero el tramo queda como “Datos insuficientes” hasta que mejore la cobertura.
score-dimensoes-titulo = Dimensiones y métricas
score-dimensoes-ajuda = Haz clic en una dimensión para ver las métricas. La Salud suma las dimensiones sin el sello “atractivo”; el Atractivo, solo las de precio. Los percentiles ya están en la dirección “mayor es mejor”.
score-modelos = Modelos clásicos
score-conferencia = Verificación con otras fuentes
score-conferencia-ajuda = Solo validación cruzada: el score usa los números que Treasury calcula con los estados financieros de la CVM. Los rendimientos se verifican con la B3 (oficial); si el calendario difiere, el DY usa la B3. Las diferencias de más del 10% quedan destacadas.

## Dimensiones y métricas

score-fonte-de = Fuente: { $fonte }
score-critica-dica = Sin esta métrica no hay score
score-sem-dado = Sin dato: { $motivo }
score-valor = Valor
score-col-nota = Nota
score-pior = {" "}(peor)
score-entre-pares = Entre los pares
score-absoluta = Absoluta
score-peso = Peso
score-periodo = Período
score-imputada = imputada
score-imputada-dica = Cobertura por debajo del mínimo: nota de la mediana de los pares × 0,9
score-selo-atratividade = atractivo
score-selo-atratividade-dica = Entra en el sub-score Atractivo (precio), no en la Salud
score-peso-cobertura = peso { $peso } · cobertura { $cobertura }
score-metricas-de = Métricas · { $dimensao }
score-dimensoes = Dimensiones del score

## Historial del score

score-data = Fecha
score-faixa = Tramo
score-historico = Historial del score
score-historico-fonte = Capturas del { $de } al { $ate }
score-historico-aria = Score de { $primeira } el { $de } a { $ultima } el { $ate }

## Tu opinión ("No estoy de acuerdo")

score-discordo = No estoy de acuerdo
score-concordo = Estoy de acuerdo
score-sem-opiniao = No sé / sin opinión
score-opiniao-registrada = Opinión registrada
score-opiniao-registrada-dica = Entra en la revisión de la metodología del score.
score-opiniao-form = Opinión sobre el score
score-veredito = Veredicto
score-faixa-esperada = Tramo que esperarías
score-opcional = Opcional.
score-motivo = Motivo
score-motivo-dica = Lo que la nota no vio o pesó de más (ej.: deuda ya renegociada, ganancia no recurrente).
score-motivo-erro = Cuéntanos el motivo.
score-registrar-opiniao = Registrar opinión
score-sua-opiniao-sobre = Tu opinión sobre el score
score-sua-opiniao = Tu opinión
score-sua-opiniao-dica = ¿La nota no coincide con lo que sabes de la empresa? Regístralo; los casos entran en la calibración de la metodología.
score-opinioes-registradas = Opiniones registradas
score-opiniao-nota = · nota { $nota } ({ $faixa }) · metodología { $versao }
score-esperava = · esperaba { $faixa }
score-discordo-titulo = No estoy de acuerdo con el score · { $nome }
score-discordo-descricao = Nota { $nota } ({ $faixa }) en la metodología { $versao }. Tu opinión queda registrada con la nota del día.

## Correo de contacto de la SEC

score-sec-baixadas = Estados financieros de la SEC descargados
score-sec-texto = La SEC (el regulador de EE. UU.) solo entrega datos a quien se identifica con un correo de contacto. Va solo en el encabezado de los pedidos a la SEC, queda en la bóveda y se puede cambiar en Ajustes › Fuentes de datos.
score-sec-email = Tu correo de contacto para la SEC
score-sec-baixando = Descargando de la SEC…
score-sec-salvar = Guardar y descargar

## Eventos cualitativos

score-evento-aguardando = Esperando por ti
score-evento-aceito = Aceptado (cuenta en la nota)
score-evento-rejeitado = Rechazado
score-evento-resolvido = Resuelto
score-evidencia = evidencia ↗
score-evento-hoje = · hoy: { $efeito }
score-aceitar = Aceptar
score-rejeitar = Rechazar
score-marcar-resolvido = Marcar como resuelto
score-voltar-valer = Volver a contar
score-apagar-evento = Eliminar el evento { $titulo }
score-evento-aplicado = Evento aplicado al score
score-casou = · coincidió “{ $termo }”
score-aplicar = Aplicar al score
score-evento-registrado = Evento registrado
score-nota-refeita = La nota se recalculó.
score-novo-evento = Nuevo evento
score-evento-tipo = Tipo
score-evento-tipo-dica = El tope o la penalización de cada tipo están en la metodología.
score-evento-o-que = Qué pasó
score-evento-descreva = Describe el evento.
score-informe-data = Ingresa la fecha.
score-evidencia-link = Evidencia (enlace)
score-evidencia-dica = Hecho relevante, noticia o documento. Recomendado.
score-observacao = Observación
score-registrar-aplicar = Registrar y aplicar
score-eventos-titulo = Eventos cualitativos
score-eventos-ajuda = Recuperación judicial, fraude, litigios, desastres, gobernanza y regulación. Solo los eventos que aceptas afectan la nota, y el efecto disminuye con el tiempo hasta que lo marques como resuelto.
score-eventos-registrados = Eventos registrados
score-nenhum-evento = Ningún evento registrado.
score-sugestoes-cvm = Sugerencias de los hechos relevantes de la CVM
score-sugestoes = Sugerencias de eventos
score-novo-evento-de = Nuevo evento · { $nome }
score-novo-evento-descricao = El evento entra aceptado y la nota se recalcula en el momento.

## Cartera › Salud de los activos

score-classe-nao-suportada = clase no admitida
score-falta-critico = Falta un dato crítico
score-parcial-dica = Más del 30% del peso imputado: sin tramo definitivo
score-ativo = Activo
score-variacao = Variación
score-flags = Flags
score-calculando-ativos = Calculando el score de los activos
score-sem-ativos = Sin activos
score-carteira-vazia = Ningún activo en la cartera ni en la watchlist
score-carteira-vazia-descricao = El score de salud evalúa los fundamentales (rentabilidad, deuda, calidad de las ganancias, valoración…) de cada acción o fondo inmobiliario que tienes o sigues. Registra una operación o sigue un activo en 09 · Activos.
score-carteira-ajuda = Score de salud de 0 a 100 de cada activo con posición o en la watchlist, por los fundamentales públicos (CVM, B3). Robusto ≥ 75, Saludable 60–74, Atención 40–59, Riesgo por debajo de 40. No es una recomendación de inversión.
score-carteira-tabela = Salud de los activos
score-sec-email-exemplo = nombre@correo.com
