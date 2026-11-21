# Transactions › Tax receipts: reading Brazilian electronic receipts (NFC-e) from the QR link, photo,
# PDF, XML or text, items with group and tax, what the family buys and imported receipts (Brazil
# only; docs/receipts.md, docs/i18n-design.md). Official terms (NFC-e, DANFE, SEFAZ, XML)
# stay as is. Revision: AI draft, not yet reviewed by a native speaker.

## Receipt preview

cupons-cupom-de = Receipt from { $loja }
cupons-cupom-fiscal = Tax receipt
cupons-sem-data = no date
cupons-previa-impresso = { $data } · { $n ->
    [one] { $n } item
   *[other] { $n } items
} · tax printed on the receipt
cupons-previa-estimado = { $data } · { $n ->
    [one] { $n } item
   *[other] { $n } items
} · tax estimated from each consumption group's rate
cupons-itens = Receipt items
cupons-item = Item
cupons-grupo = Group
cupons-grupo-de = Group for { $item }
cupons-valor = Amount
cupons-imposto = Tax
cupons-pago-valor = Paid: { $valor }
cupons-desconto = ({ $valor } discount)
cupons-imposto-valor = Tax: { $valor }
cupons-tributos-nota = (approximate taxes from the receipt, Law 12,741/2012)
cupons-estimado-itens = (estimated item by item)
cupons-lancamento = Transaction: { $descricao } on { $data } ({ $conta })
cupons-sem-lancamento = No transaction with this amount near this date: the receipt stays unlinked.
cupons-quem-comprou = Who bought it
cupons-familia = Family
cupons-gravado = Receipt saved
cupons-gravado-texto = { $n ->
    [one] { $n } item from { $loja }
   *[other] { $n } items from { $loja }
}
cupons-loja = store
cupons-substituir = Replace receipt
cupons-gravar = Save receipt

## Read a receipt

cupons-ler-titulo = Read a tax receipt
cupons-ler-texto = The QR code on the receipt (NFC-e) leads to the SEFAZ (state tax office) lookup, with the items and the approximate taxes. Paste the link (your phone camera shows it when it reads the QR code), send a photo of the QR code, the receipt PDF or the XML file. Receipt without a QR code: send the photo to the agent, which reads the items.
cupons-link = Receipt QR link
cupons-ler = Read receipt
cupons-foto = Receipt photo
cupons-foto-dica = The image is read right here and is not stored.
cupons-qr-nao-achado = Couldn't find the QR code in the photo. Reading the receipt text instead…
cupons-pdf = Receipt PDF (DANFE NFC-e) or XML file
cupons-esconder-texto = Hide the text
cupons-colar-texto = Paste the text from the lookup page
cupons-texto = Receipt text
cupons-texto-placeholder = Open the QR link in the browser, select everything and paste it here.
cupons-ler-texto-botao = Read the text

## What the family buys

cupons-periodo-mes = This month
cupons-periodo-3m = Last 3 months
cupons-periodo-ano = This year
cupons-familia-compra = What the family buys
cupons-periodo-compras = Purchase period
cupons-resumo = { $n ->
    [one] { $n } receipt
   *[other] { $n } receipts
}, { $gasto } in purchases and { $imposto } in tax ({ $pct } of what was paid).
cupons-resumo-impresso = { $n ->
    [one] { $n } receipt
   *[other] { $n } receipts
}, { $gasto } in purchases and { $imposto } in tax ({ $pct } of what was paid; part of it printed on the receipts).
cupons-compras-grupo = Purchases by group
cupons-gasto = Spent
cupons-pct-compras = % of purchases
cupons-compras-de = Purchases by { $membro }
cupons-itens-pesam = Items that weigh the most
cupons-vazio-familia = No receipts in this period. Read the QR codes on your grocery receipts to see what each person buys and how much tax goes into each group.

## Receipt from a photo, without the QR (docs/ocr-receipts.md)
cupons-foto-dica-qr = The QR code first; without it, the receipt text. The image is read right here and is not stored.
cupons-foto-lendo = Reading the receipt…
cupons-foto-titulo = Read a receipt photo
cupons-foto-texto = Take a straight, well-lit photo of the whole receipt. The items are read on this computer and you review them before saving.
cupons-foto-leitura = Photo reading
cupons-foto-faixa-conferido = Checked: the items add up to the total, and the item count or the payment match too.
cupons-foto-faixa-fechou = The items add up to the total. Take a quick look at them.
cupons-foto-faixa-nao-fechou = The items don't add up to the total. Check and fix them.
cupons-foto-motor-vision = Read by Apple Vision.
cupons-foto-motor-paddle = Read by Treasury's reader.
cupons-foto-motor-ia = Read by the AI.
cupons-foto-total-impresso = Total on the receipt: { $valor }.
cupons-foto-faltam = The items are { $valor } short.
cupons-foto-sobram = The items are { $valor } over.
cupons-foto-pedir-ia = Ask the AI for help
cupons-foto-pedir-ia-dica = The photo goes to the engine in Settings › AI; if it's a remote one, it leaves this computer.

## Lookup in an app window, for the captcha (docs/ocr-receipts.md §4)
cupons-sefaz-abrir = Open the lookup here
cupons-sefaz-dica = If SEFAZ asks for a captcha, open the lookup here in the app: solve the captcha in the window and the receipt is read on its own.
cupons-sefaz-aberta = Solve the captcha in the SEFAZ window. When the receipt page shows up, it's read on its own.
cupons-sefaz-ler-pagina = Read the page now
cupons-sefaz-erro-link = The link must be the SEFAZ lookup (https, on a .gov.br site).

## The photo beside the purchase
cupons-foto-imagem = Receipt photo
cupons-foto-imagem-dica = Drag to move; use the mouse wheel or the buttons to zoom. Double-click zooms in.
cupons-foto-aproximar = Zoom in
cupons-foto-afastar = Zoom out
cupons-foto-girar = Rotate the photo
cupons-foto-ajustar = Fit to screen

## Receipt reader (ReceiptReader)

cupons-area = Read a receipt
cupons-area-titulo = Drop the receipt photo, PDF or XML here
cupons-area-titulo-foto = Drop the receipt photo here
cupons-area-dica = Or choose the file, or paste the QR link. The app figures out what it is: the QR leads to SEFAZ (or AFIP, for Argentine receipts) and, without a QR, the items come from the photo's text, read on this computer.
cupons-area-dica-foto = Or choose the file. The items come from the photo's text, read on this computer; the AFIP QR (Argentina) works too.
cupons-escolher-arquivo = Choose file
cupons-link-placeholder = or paste the QR link (https://…)
cupons-etapas = Reading steps
cupons-etapa-qr = Look for the QR in the photo
cupons-etapa-qr-nao-achado = no readable QR, reading the items from the photo
cupons-etapa-link = Recognize the link
cupons-etapa-sefaz = Query the receipt at SEFAZ
cupons-etapa-afip = Read the invoice from the QR (Argentina, Portugal, Spain)
cupons-etapa-afip-sem-itens = the QR has no items: add them in the review
cupons-etapa-ocr = Read the items from the photo's text
cupons-etapa-ocr-ia = with AI
cupons-etapa-ocr-itens = { $n } item line(s)
cupons-etapa-arquivo = Read the file
cupons-etapa-texto = Read the page text
cupons-etapa-conferir = Check items and total
cupons-etapa-conferir-ok = { $n } item(s), sum matches the total
cupons-etapa-conferir-diferenca = { $n } item(s), sum differs from the total
cupons-resultado = Reading result
cupons-resultado-ok = { $n } item(s) from { $loja }, total { $valor }. Everything matches.
cupons-resultado-diferenca = { $n } item(s) from { $loja }, total { $valor }. The items don't add up to the total.
cupons-resultado-foto-ok = { $n } item(s) read from the photo, total { $valor }. The sum matches.
cupons-resultado-foto-diferenca = { $n } item(s) read from the photo, total { $valor }. The sum doesn't match: check the items.
cupons-resultado-afip = { $pais ->
    [PT] Portuguese invoice
    [ES] Spanish invoice
   *[AR] Argentine receipt
}: { $documento }, { $valor }.
cupons-proximo-ligar = Review and save: the receipt links to the purchase transaction already in the app.
cupons-proximo-sem-lancamento = Review and save. No transaction with this amount near the date: the receipt stays unlinked.
cupons-proximo-foto = Review the items next to the photo and save the purchase.
cupons-proximo-foto-corrigir = Fix the items next to the photo, or ask the AI for help.
cupons-proximo-afip = Add the items from the photo or by hand, check the amount in your currency and save.
cupons-revisar-gravar = Review and save
cupons-revisar-corrigir = Review and fix
cupons-ler-pela-foto = Read from the photo
cupons-cadastrar-mao = Add by hand
cupons-ler-outro = Read another receipt
cupons-erro-link-fora = This link isn't from a receipt the app reads here. Use the receipt photo or add the purchase by hand.
cupons-erro-arquivo-fora = PDF and XML are for Brazilian receipts. Use the receipt photo or add the purchase by hand.
cupons-arquivo = Receipt file
cupons-foto-motor-texto = Read from the pasted text.
