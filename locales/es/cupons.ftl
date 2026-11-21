# Movimientos › Comprobantes fiscales: lectura de la factura electrónica brasileña (NFC-e) por el enlace
# del QR, la foto, el PDF, el XML o el texto, ítems con grupo e impuesto, lo que compra la familia y
# los comprobantes importados (solo en Brasil; docs/receipts.md, docs/i18n-design.md). Los términos
# oficiales (NFC-e, DANFE, SEFAZ, XML) quedan tal cual. Revisión: borrador por IA, sin
# revisión nativa.

## Vista previa del comprobante

cupons-cupom-de = Comprobante de { $loja }
cupons-cupom-fiscal = Comprobante fiscal
cupons-sem-data = sin fecha
cupons-previa-impresso = { $data } · { $n ->
    [one] { $n } ítem
   *[other] { $n } ítems
} · impuesto impreso en el comprobante
cupons-previa-estimado = { $data } · { $n ->
    [one] { $n } ítem
   *[other] { $n } ítems
} · impuesto estimado con el porcentaje de cada grupo de consumo
cupons-itens = Ítems del comprobante
cupons-item = Ítem
cupons-grupo = Grupo
cupons-grupo-de = Grupo de { $item }
cupons-valor = Valor
cupons-imposto = Impuesto
cupons-pago-valor = Pagado: { $valor }
cupons-desconto = (descuento de { $valor })
cupons-imposto-valor = Impuesto: { $valor }
cupons-tributos-nota = (tributos aproximados del comprobante, Ley 12.741/2012)
cupons-estimado-itens = (estimado ítem por ítem)
cupons-lancamento = Movimiento: { $descricao } el { $data } ({ $conta })
cupons-sem-lancamento = Ningún movimiento con ese valor cerca de la fecha: el comprobante queda sin vincular.
cupons-quem-comprou = Quién compró
cupons-familia = Familia
cupons-gravado = Comprobante guardado
cupons-gravado-texto = { $n ->
    [one] { $n } ítem de { $loja }
   *[other] { $n } ítems de { $loja }
}
cupons-loja = tienda
cupons-substituir = Reemplazar comprobante
cupons-gravar = Guardar comprobante

## Leer comprobante

cupons-ler-titulo = Leer comprobante fiscal
cupons-ler-texto = El QR del comprobante (NFC-e) lleva a la consulta de la SEFAZ (secretaría de hacienda del estado), con los ítems y los tributos aproximados. Pega el enlace (la cámara del celular lo muestra al leer el QR), envía la foto del QR, el PDF del comprobante o el XML. Comprobante sin QR: envía la foto al agente, que lee los ítems.
cupons-link = Enlace del QR del comprobante
cupons-ler = Leer comprobante
cupons-foto = Foto del recibo
cupons-foto-dica = La imagen se lee aquí mismo y no se guarda.
cupons-qr-nao-achado = No encontré el QR en la foto. Leyendo el texto del recibo…
cupons-pdf = PDF del comprobante (DANFE NFC-e) o XML de la factura
cupons-esconder-texto = Ocultar el texto
cupons-colar-texto = Pegar el texto de la página de consulta
cupons-texto = Texto del comprobante
cupons-texto-placeholder = Abre el enlace del QR en el navegador, selecciona todo y pégalo aquí.
cupons-ler-texto-botao = Leer el texto

## Lo que compra la familia

cupons-periodo-mes = Este mes
cupons-periodo-3m = Últimos 3 meses
cupons-periodo-ano = Este año
cupons-familia-compra = Lo que compra la familia
cupons-periodo-compras = Período de las compras
cupons-resumo = { $n ->
    [one] { $n } comprobante
   *[other] { $n } comprobantes
}, { $gasto } en compras y { $imposto } de impuesto ({ $pct } de lo pagado).
cupons-resumo-impresso = { $n ->
    [one] { $n } comprobante
   *[other] { $n } comprobantes
}, { $gasto } en compras y { $imposto } de impuesto ({ $pct } de lo pagado; parte impreso en los comprobantes).
cupons-compras-grupo = Compras por grupo
cupons-gasto = Gasto
cupons-pct-compras = % de las compras
cupons-compras-de = Compras de { $membro }
cupons-itens-pesam = Ítems que más pesan
cupons-vazio-familia = Ningún comprobante en el período. Lee el QR de los comprobantes del supermercado para ver qué compra cada uno y cuánto impuesto va en cada grupo.

## Recibo desde una foto, sin el QR (docs/ocr-receipts.md)
cupons-foto-dica-qr = Primero el QR; sin QR, el texto del recibo. La imagen se lee aquí mismo y no se guarda.
cupons-foto-lendo = Leyendo el recibo…
cupons-foto-titulo = Leer recibo desde la foto
cupons-foto-texto = Toma una foto derecha y con buena luz del recibo entero. Los artículos se leen en esta computadora y los revisas antes de guardar.
cupons-foto-leitura = Lectura de la foto
cupons-foto-faixa-conferido = Verificado: la suma de los artículos coincide con el total, y también la cantidad de artículos o el pago.
cupons-foto-faixa-fechou = La suma de los artículos coincide con el total. Échales un vistazo.
cupons-foto-faixa-nao-fechou = La suma de los artículos no coincide con el total. Revísalos y corrígelos.
cupons-foto-motor-vision = Leído por Apple Vision.
cupons-foto-motor-paddle = Leído por el lector de Treasury.
cupons-foto-motor-ia = Leído por la IA.
cupons-foto-total-impresso = Total en el recibo: { $valor }.
cupons-foto-faltam = Faltan { $valor } en los artículos.
cupons-foto-sobram = Los artículos se pasan por { $valor }.
cupons-foto-pedir-ia = Pedir ayuda a la IA
cupons-foto-pedir-ia-dica = La foto va al motor de Ajustes › IA; si es remoto, sale de esta computadora.

## Consulta en una ventana de la app, para el captcha (docs/ocr-receipts.md §4)
cupons-sefaz-abrir = Abrir la consulta aquí
cupons-sefaz-dica = Si la SEFAZ pide un captcha, abre la consulta aquí en la app: resuelve el captcha en la ventana y el recibo se lee solo.
cupons-sefaz-aberta = Resuelve el captcha en la ventana de la SEFAZ. Cuando aparezca la página del recibo, se lee sola.
cupons-sefaz-ler-pagina = Leer la página ahora
cupons-sefaz-erro-link = El enlace debe ser el de la consulta de la SEFAZ (https, en un sitio .gov.br).

## La foto al lado de la compra
cupons-foto-imagem = Foto del recibo
cupons-foto-imagem-dica = Arrastra para mover; usa la rueda del mouse o los botones para acercar. Doble clic acerca.
cupons-foto-aproximar = Acercar
cupons-foto-afastar = Alejar
cupons-foto-girar = Girar la foto
cupons-foto-ajustar = Ajustar a la pantalla

## Lector de comprobantes (ReceiptReader)

cupons-area = Leer comprobante
cupons-area-titulo = Suelta aquí la foto, el PDF o el XML del comprobante
cupons-area-titulo-foto = Suelta aquí la foto del comprobante
cupons-area-dica = O elige el archivo, o pega el enlace del QR. La app descubre qué es: el QR lleva a la SEFAZ (o a la AFIP, en los comprobantes de Argentina) y, sin QR, los ítems salen del texto de la foto, leído en esta computadora.
cupons-area-dica-foto = O elige el archivo. Los ítems salen del texto de la foto, leído en esta computadora; el QR de la AFIP (Argentina) también sirve.
cupons-escolher-arquivo = Elegir archivo
cupons-link-placeholder = o pega el enlace del QR (https://…)
cupons-etapas = Etapas de la lectura
cupons-etapa-qr = Buscar el QR en la foto
cupons-etapa-qr-nao-achado = sin QR legible, leyendo los ítems de la foto
cupons-etapa-link = Reconocer el enlace
cupons-etapa-sefaz = Consultar el comprobante en la SEFAZ
cupons-etapa-afip = Leer el comprobante por el QR (Argentina, Portugal, España)
cupons-etapa-afip-sem-itens = el QR no trae los ítems: agrégalos en la revisión
cupons-etapa-ocr = Leer los ítems del texto de la foto
cupons-etapa-ocr-ia = con la IA
cupons-etapa-ocr-itens = { $n } línea(s) de ítem
cupons-etapa-arquivo = Leer el archivo
cupons-etapa-texto = Leer el texto de la página
cupons-etapa-conferir = Revisar ítems y total
cupons-etapa-conferir-ok = { $n } ítem(s), la suma coincide con el total
cupons-etapa-conferir-diferenca = { $n } ítem(s), la suma no coincide con el total
cupons-resultado = Resultado de la lectura
cupons-resultado-ok = { $n } ítem(s) de { $loja }, total { $valor }. Todo coincide.
cupons-resultado-diferenca = { $n } ítem(s) de { $loja }, total { $valor }. La suma de los ítems no coincide con el total.
cupons-resultado-foto-ok = { $n } ítem(s) leídos de la foto, total { $valor }. La suma coincide.
cupons-resultado-foto-diferenca = { $n } ítem(s) leídos de la foto, total { $valor }. La suma no coincide: revisa los ítems.
cupons-resultado-afip = { $pais ->
    [PT] Factura de Portugal
    [ES] Factura de España
   *[AR] Comprobante de Argentina
}: { $documento }, { $valor }.
cupons-proximo-ligar = Revisa y guarda: el comprobante se vincula al movimiento de la compra que ya está en la app.
cupons-proximo-sem-lancamento = Revisa y guarda. Ningún movimiento con ese importe cerca de la fecha: el comprobante queda sin vínculo.
cupons-proximo-foto = Revisa los ítems junto a la foto y guarda la compra.
cupons-proximo-foto-corrigir = Corrige los ítems junto a la foto, o pide ayuda a la IA.
cupons-proximo-afip = Agrega los ítems con la foto o a mano, revisa el importe en tu moneda y guarda.
cupons-revisar-gravar = Revisar y guardar
cupons-revisar-corrigir = Revisar y corregir
cupons-ler-pela-foto = Leer de la foto
cupons-cadastrar-mao = Registrar a mano
cupons-ler-outro = Leer otro comprobante
cupons-erro-link-fora = Ese enlace no es de un comprobante que la app lea aquí. Usa la foto del comprobante o registra la compra a mano.
cupons-erro-arquivo-fora = PDF y XML son de los comprobantes de Brasil. Usa la foto del comprobante o registra la compra a mano.
cupons-arquivo = Archivo del comprobante
cupons-foto-motor-texto = Leído del texto pegado.
