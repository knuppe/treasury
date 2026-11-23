# Import: the single import window, the statement wizard (OFX, CSV and PDF card bill), the
# Imports tab and the Downloads folder notice (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.
# The official names of Brazil-only importers (Área do Investidor B3, SINACOR notes, CPF) stay
# as they are in every language.

## Single import window

importacao-arquivos-titulo = Import files
importacao-arquivos-descricao = OFX or CSV statements, PDF card bills and brokerage notes, receipts (photo, PDF or XML), B3 Área do Investidor reports and photos of payment proofs. Treasury recognizes the file on this computer and opens the right importer; AI only steps in for what it cannot read.
importacao-lendo-arquivos = Reading the files…
importacao-soltar = Drop the files here or click to choose
importacao-formatos = OFX, QFX, CSV, PDF, XML, XLSX, XLS or ODS · PNG, JPEG or HEIC images (read on this computer)
importacao-email-dica = Do you get statements and card bills by email? { $link } and they arrive on their own for you to review.
importacao-email-link = Set up email import

importacao-tipo-extratos = Statements and card bills
importacao-tipo-notas = Brokerage notes
importacao-tipo-b3 = B3 Área do Investidor
importacao-tipo-previdencia = Pension plan statements
importacao-tipo-imagens = Other images (for the AI agent)
importacao-tipo-cupons = Receipts and purchase invoices
importacao-tipo-comprovantes = Payment receipts and bank slips
importacao-lendo-imagem = reading on this computer (QR and text)…
importacao-cupom-qr = Receipt with a QR code: opens the invoice in the official SEFAZ lookup
importacao-imagem-nao-lida = Could not read the image on this computer
importacao-revisar-compra = Review purchase
importacao-registrar-agente = Record with the agent
importacao-agente-comprovantes = { $n ->
    [one] Record this payment receipt or bank slip in Treasury (transaction or bill to pay).
   *[other] Record these { $n } payment receipts or bank slips in Treasury (transactions or bills to pay).
}
importacao-botao-topo = Import
importacao-botao-topo-dica = Import files: statements, card bills, brokerage notes, receipts, B3 reports and images
importacao-tipo-desconhecidos = Not recognized

importacao-importar = Import
importacao-importar-notas = { $n ->
        [one] Import note
       *[other] Import { $n } notes
    }
importacao-importar-b3 = Import from B3
importacao-importar-agente = Import with the agent
importacao-previdencia-dica = Import it in the asset's Pension tab, under Assets.
importacao-anexar-agente = Attach to the agent
importacao-motor-imagens = Set up an AI engine in Agent › Configure to read images.
importacao-analisar-ia = Analyze with AI
importacao-motor-analisar = Set up an AI engine in Agent › Configure to analyze these files.
importacao-fatura-cartao = It's a card bill
importacao-tirar = Remove { $nome }

# Text sent ready-made to the AI agent, with the file names.
importacao-agente-analisar = { $n ->
        [one] Analyze the attached file ({ $nomes }) and help me record in Treasury what you find.
       *[other] Analyze the attached files ({ $nomes }) and help me record in Treasury what you find.
    }

importacao-n-lixeira = { $n ->
        [one] { $n } file from Downloads went to the Trash
       *[other] { $n } files from Downloads went to the Trash
    }
importacao-erro-lixeira = Couldn't delete from Downloads

## Downloads folder notice

importacao-downloads-regiao = New files in Downloads
importacao-downloads-titulo = { $n ->
        [one] New file in Downloads
       *[other] { $n } new files in Downloads
    }
importacao-downloads-dispensar = Dismiss { $nome }
importacao-downloads-apagar = Delete from Downloads after importing
importacao-downloads-importar = { $n ->
        [one] Import
       *[other] Import { $n }
    }
importacao-downloads-agora-nao = Not now
importacao-erro-apagar-arquivo = Couldn't delete the file.
importacao-erro-ler-arquivo = Couldn't read the file.
importacao-erro-nao-encontrado = { $nome }: file not found

## Imports tab

importacao-importacoes = Imports
importacao-col-quando = When
importacao-col-arquivo = File
importacao-col-conta = Account
importacao-col-inseridos = Added
importacao-col-duplicados = Duplicates
importacao-col-erros = Errors
importacao-col-acoes = Actions
importacao-erro-linha = line { $linha }: { $mensagem }
importacao-ver-lancamentos = View transactions
importacao-desfeita = Import undone
importacao-erro-desfazer = Couldn't undo
importacao-desfazer = Undo
importacao-importar-extrato = Import statement
importacao-vazio-kicker = No imports
importacao-vazio-titulo = Bring in your bank statement
importacao-vazio-descricao = OFX or CSV from accounts and cards, with duplicate detection and your categorization rules.

## Statement wizard: steps and file

importacao-etapas = Steps
importacao-etapa-arquivo = File
importacao-etapa-leitura = Reading
importacao-etapa-previa = Preview
importacao-conta-destino = Destination account
importacao-escolha-conta = Choose the account…
importacao-escolha-conta-destino = Choose the destination account
importacao-arquivo-banco = Bank file
importacao-arquivo-banco-dica = OFX (statement or card bill) or CSV. The file stays only in the app's memory until you confirm.
importacao-lendo-arquivo = Reading the file…
importacao-gerar-previa = Build preview

## Reading: account recognized from the OFX

importacao-ofx-lembrada = Account remembered from the last import of this file.
importacao-ofx-lembrada-trocada = Account remembered from the last import of this file (changed from the one chosen before).
importacao-ofx-reconhecida = Recognized by the bank, branch and number in the file.
importacao-ofx-reconhecida-trocada = Recognized by the bank, branch and number in the file (changed from the one chosen before).
# $motivo comes from the core.
importacao-ofx-nao-reconheci = I didn't recognize the account ({ $motivo }). Choose which account to import into.
importacao-ofx-nao-reconheci-lembra = I didn't recognize the account ({ $motivo }). Choose which account to import into; your choice is remembered for the next files from this account.
importacao-csv-sem-conta = The CSV doesn't say which account it's from: check the account before the preview.
importacao-pdf-conta-cartao = The PDF card bill goes into the card's account.
importacao-pdf-conta-extrato = The PDF statement goes into the bank's checking account.
importacao-pdf-extrato-titulo = Account statement in PDF
importacao-pdf-extrato-lidos =
    { $n ->
        [one] { $n } transaction read ({ $leiaute }).
       *[other] { $n } transactions read ({ $leiaute }).
    }
importacao-pdf-extrato-lidos-ocr =
    { $n ->
        [one] { $n } transaction read on this computer by OCR ({ $leiaute }), without AI. Check the amounts in the preview.
       *[other] { $n } transactions read on this computer by OCR ({ $leiaute }), without AI. Check the amounts in the preview.
    }
importacao-pdf-extrato-nao-cartao = The chosen account is a card: choose this statement's checking account.
importacao-ref-gravada = Branch and account saved
importacao-ref-gravada-descricao = The next OFX files from this bank go straight to { $conta }.

## Reading: file details

importacao-info-arquivo = File
importacao-info-formato = Format
importacao-info-codificacao = Encoding
importacao-info-linhas = Rows
importacao-ofx-fatura = Card bill (OFX)
importacao-ofx-extrato = Account statement (OFX)
importacao-ofx-banco = Bank
importacao-ofx-conta-arquivo = Account in the file
importacao-ofx-periodo = Period
importacao-ofx-periodo-valor = { $inicio } to { $fim }
importacao-ofx-saldo = Reported balance
importacao-banco-arquivo = Bank in the file: { $banco }
importacao-gravar-ref = Save branch and account to { $conta }
importacao-gravar-ref-dica = That way the next OFX files from this bank land in this account on their own.
importacao-ofx-cartao-conta = The file is from a credit card and the chosen account isn't a card.
importacao-ofx-conta-cartao = The chosen account is a card, but the file is an account statement.

## Reading: PDF card bill

importacao-pdf-titulo = Card bill (PDF)
importacao-pdf-lidos = { $n ->
        [one] { $n } transaction read. It goes into the bill for the due date printed on the PDF, and the bill total is checked in the preview.
       *[other] { $n } transactions read. They go into the bill for the due date printed on the PDF, and the bill total is checked in the preview.
    }
importacao-pdf-protegido = The PDF seems to be password-protected (or has no text). Enter the bill's password, usually the first digits of the CPF.
importacao-pdf-senha = PDF password (if any)
importacao-pdf-senha-dica = Used only to read the file now; it isn't stored.
importacao-pdf-nao-cartao = The chosen account isn't a credit card: choose the card for this bill.

## Reading: CSV columns

importacao-csv-modelo = Bank template
importacao-csv-modelo-reconhecido = Recognized from the header.
importacao-csv-personalizado = Custom
importacao-csv-separador = Separator
importacao-csv-pular = Rows to skip
importacao-csv-data = Date
importacao-csv-formato-data = Date format
importacao-csv-descricao = Description
importacao-csv-valor = Amount
importacao-csv-valor-unico = One amount column
importacao-csv-valor-separado = Separate debit and credit
importacao-csv-valor-sinal = Amount (with sign)
importacao-csv-debito = Debit (money out)
importacao-csv-credito = Credit (money in)
importacao-csv-identificador = Identifier (optional)
importacao-csv-identificador-dica = Avoids duplicates when importing again.
importacao-csv-cabecalho = The first row is a header
importacao-csv-virgula = Decimal comma (1.234,56)
importacao-csv-inverter = Flip signs (card bill with purchases as positive)
importacao-csv-coluna = Column { $n }
importacao-amostra = File sample

importacao-sep-ponto-virgula = Semicolon (;)
importacao-sep-virgula = Comma (,)
importacao-sep-tabulacao = Tab
importacao-sep-barra = Vertical bar (|)

importacao-data-dmy = dd/mm/yyyy
importacao-data-ymd = yyyy-mm-dd
importacao-data-dmy-curto = dd/mm/yy
importacao-data-dmy-hifen = dd-mm-yyyy
importacao-data-dmy-ponto = dd.mm.yyyy
importacao-data-dmy-ou-ymd = dd/mm/yyyy or yyyy-mm-dd
importacao-data-mdy = mm/dd/yyyy (US)
importacao-data-mdy-curto = mm/dd/yy (US)
importacao-data-mdy-hifen = mm-dd-yyyy (US)

## Preview

importacao-troca-conta-dica = Changing the account rebuilds the preview (duplicates are checked in the chosen account).
importacao-filtrar-linhas = Filter preview rows
importacao-todas = All
importacao-previa = Import preview
importacao-col-importar = Import
importacao-col-situacao = Status
importacao-col-data = Date
importacao-col-descricao = Description
importacao-col-categoria = Category
importacao-col-valor = Amount
importacao-fatura = Card bill
importacao-cartao = Card
importacao-fatura-vence = { $emissor }, due { $data }
importacao-total-impresso = Printed total
importacao-lido-linhas = Read from the rows
importacao-confere = ✓ matches
importacao-layout = Layout
importacao-avisos = Warnings
importacao-incluir-possiveis = { $n ->
        [one] Also import the { $n } possible duplicate
       *[other] Also import the { $n } possible duplicates
    }
importacao-incluir-possiveis-dica = Same date and amount as an existing transaction, with a similar description.
importacao-importar-linha = Import row { $n }
importacao-descricao-linha = Description for row { $n }
importacao-categoria-linha = Category for row { $n }
importacao-sugestao-regra = rule suggestion
importacao-importar-n = { $n ->
        [one] Import { $n } transaction
       *[other] Import { $n } transactions
    }

## Result

importacao-n-importados = { $n ->
        [one] { $n } transaction imported
       *[other] { $n } transactions imported
    }
importacao-resultado = { $inseridos ->
        [one] { $inseridos } transaction imported
       *[other] { $inseridos } transactions imported
    } into { $conta }; { $pulados ->
        [one] { $pulados } row was left out.
       *[other] { $pulados } rows were left out.
    }
importacao-resultado-parcelas = { $n ->
        [one] { $n } contract installment marked as paid by its category.
       *[other] { $n } contract installments marked as paid by their category.
    }
importacao-resultado-transferencias = { $n ->
        [one] { $n } transfer between your accounts already linked.
       *[other] { $n } transfers between your accounts already linked.
    }
importacao-resultado-previstos = { $n ->
        [one] { $n } planned transaction was settled by the statement.
       *[other] { $n } planned transactions were settled by the statement.
    }
importacao-ver-importados = View imported transactions

# Boleto, Pix e comprovante lidos no computador (docs/ocr-other-documents.md).
importacao-pagamento-titulo = Payment document
importacao-pagamento-titulo-boleto = Bill to pay
importacao-pagamento-titulo-pix = Pix charge
importacao-pagamento-titulo-comprovante = Payment receipt
importacao-pagamento-lendo = Reading on this computer (barcode, QR and text)…
importacao-pagamento-revisar = Review
importacao-pagamento-pix-qr = Pix charge (QR): becomes a planned bill
importacao-pagamento-nada = I didn't find a boleto, Pix code or payment receipt in this image.
importacao-pagamento-pedir-agente = Ask the agent
importacao-pagamento-linha = Payment line or code
importacao-pagamento-copiar = Copy
importacao-pagamento-copiado = Copied
importacao-pagamento-banco = Bank
importacao-pagamento-beneficiario = Payee
importacao-pagamento-recebedor = Recipient
importacao-pagamento-chave = Pix key
importacao-pagamento-descricao = Description
importacao-pagamento-valor = Amount
importacao-pagamento-vencimento = Due date
importacao-pagamento-conta = Account
importacao-pagamento-escolha-conta = Choose the account…
importacao-pagamento-categoria = Category
importacao-pagamento-previsto-dica = It goes into the upcoming bills and the due-date reminder. The app doesn't pay anything: copy the payment line into your bank's app.
importacao-pagamento-criar-previsto = Create planned bill
importacao-pagamento-previsto-criado = Planned bill created
importacao-pagamento-ja-lancado = Already recorded as planned: { $descricao }, { $data }.
importacao-pagamento-tipo = Type
importacao-pagamento-tipo-pix = Pix
importacao-pagamento-tipo-transferencia = Transfer
importacao-pagamento-tipo-boleto = Paid boleto
importacao-pagamento-tipo-outro = Payment
importacao-pagamento-data = Date
importacao-pagamento-quem-recebeu = Paid to
importacao-pagamento-quem-pagou = Paid by
importacao-pagamento-instituicao = Institution
importacao-pagamento-e2e = Transaction ID
importacao-pagamento-ligar-a = Link to
importacao-pagamento-nenhum = No statement transaction matches (same amount, within 2 days of the date).
importacao-pagamento-criar-lancamento = Create the transaction
importacao-pagamento-usar-previsto = Use the planned bill as the payment
importacao-pagamento-quitar = Settle the planned bill { $descricao } ({ $data })
importacao-pagamento-favorecido = Payee
importacao-pagamento-sem-imagem-dica = The image isn't kept: only the data read goes into the transaction's note.
importacao-pagamento-salvar-comprovante = Save the receipt data
importacao-pagamento-comprovante-salvo = Receipt linked to the transaction
importacao-pagamento-ja-ligado = This receipt is already linked to { $descricao } ({ $data }).
importacao-desfazer-historico = To undo it, use Settings › Change history.

## Caixa de entrada de arquivos

caixa-titulo = File inbox
caixa-explica = Files put in { $pasta } (by you or by an AI agent) land here: Treasury reads them, keeps them in the vault and removes them from the folder. Import each one when you want.
caixa-desligada = The inbox watch is off (Settings › Preferences).
caixa-mostrar-todos = Also show imported and dismissed
caixa-vazia = Nothing in the inbox
caixa-vazia-texto = Put a statement, brokerage note or pension statement in { $pasta } and it shows up here in seconds.
caixa-lista = Files in the inbox
caixa-importar = Import
caixa-descartar = Dismiss
caixa-descartar-nome = Dismiss { $nome }
caixa-erro-abrir = Could not open the file
caixa-status-pendente = to import
caixa-status-importado = imported
caixa-status-descartado = dismissed
caixa-status-desconhecido = unrecognized type
caixa-aviso-novo = New file in the inbox: { $tipo }
caixa-aviso-desconhecido = { $nome } arrived in the inbox, but its type was not recognized
caixa-aviso-revisar = Review

## Documento como texto (Markdown)

documento-ver-texto = View as text
documento-ver-texto-nome = View { $nome } as text
documento-texto-titulo = { $nome } as text
documento-texto-lendo = Reading the document…
documento-texto-resumo =
    { $paginas ->
        [one] This is how AI assistants read this document: { $paginas } page, { $total } characters.
       *[other] This is how AI assistants read this document: { $paginas } pages, { $total } characters.
    }
documento-texto-ocr = The PDF is only an image: the text came from the computer's OCR and may have errors.
documento-texto-paginas-corridas = Pages as plain text (the check did not match): { $paginas }.
documento-texto-conteudo = Document content
documento-texto-copiar = Copy
documento-texto-copiado = Text copied
