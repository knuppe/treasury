# Importación: la ventana única de importación, el asistente de extractos (OFX, CSV y resumen de
# tarjeta en PDF), la pestaña Importaciones y el aviso de la carpeta Descargas
# (docs/i18n-design.md). Revisión: borrador por IA, sin revisión nativa.
# Los nombres oficiales de los importadores solo de Brasil (Área do Investidor B3, notas SINACOR,
# CPF) quedan igual en cualquier idioma.

## Ventana única de importación

importacao-arquivos-titulo = Importar archivos
importacao-arquivos-descricao = Extractos OFX o CSV, resúmenes de tarjeta y notas de corretaje en PDF, tickets fiscales (foto, PDF o XML), informes del Área do Investidor de B3 y fotos de comprobantes. Treasury reconoce el archivo en este ordenador y abre el importador correcto; la IA solo entra en lo que no puede leer.
importacao-lendo-arquivos = Leyendo los archivos…
importacao-soltar = Suelta los archivos aquí o haz clic para elegir
importacao-formatos = OFX, QFX, CSV, PDF, XML, XLSX, XLS u ODS · imágenes PNG, JPEG o HEIC (leídas en este ordenador)
importacao-email-dica = ¿Recibes extractos y resúmenes por correo? { $link } y llegan solos para que los revises.
importacao-email-link = Configura la importación por correo

importacao-tipo-extratos = Extractos y resúmenes
importacao-tipo-notas = Notas de corretaje
importacao-tipo-b3 = Área do Investidor de B3
importacao-tipo-previdencia = Extractos de planes de pensión
importacao-tipo-imagens = Otras imágenes (para el agente de IA)
importacao-tipo-cupons = Tickets fiscales y facturas de compra
importacao-tipo-comprovantes = Comprobantes de pago y boletos
importacao-lendo-imagem = leyendo en este ordenador (QR y texto)…
importacao-cupom-qr = Ticket con QR: abre la factura en la consulta oficial de la SEFAZ
importacao-imagem-nao-lida = No se pudo leer la imagen en este ordenador
importacao-revisar-compra = Revisar compra
importacao-registrar-agente = Registrar con el agente
importacao-agente-comprovantes = { $n ->
    [one] Registra este comprobante o boleto en Treasury (movimiento o cuenta por pagar).
   *[other] Registra estos { $n } comprobantes o boletos en Treasury (movimientos o cuentas por pagar).
}
importacao-botao-topo = Importar
importacao-botao-topo-dica = Importar archivos: extractos, facturas, notas, tickets, informes de la B3 e imágenes
importacao-tipo-desconhecidos = No reconocidos

importacao-importar = Importar
importacao-importar-notas = { $n ->
        [one] Importar nota
       *[other] Importar { $n } notas
    }
importacao-importar-b3 = Importar de B3
importacao-importar-agente = Importar con el agente
importacao-previdencia-dica = Impórtalo en la pestaña Pensión del activo, en Activos.
importacao-anexar-agente = Adjuntar al agente
importacao-motor-imagens = Configura un motor de IA en Agente › Configurar para leer imágenes.
importacao-analisar-ia = Analizar con IA
importacao-motor-analisar = Configura un motor de IA en Agente › Configurar para analizar estos archivos.
importacao-fatura-cartao = Es un resumen de tarjeta
importacao-tirar = Quitar { $nome }

# Texto que va listo al agente de IA, con los nombres de los archivos.
importacao-agente-analisar = { $n ->
        [one] Analiza el archivo adjunto ({ $nomes }) y ayúdame a registrar en Treasury lo que encuentres.
       *[other] Analiza los archivos adjuntos ({ $nomes }) y ayúdame a registrar en Treasury lo que encuentres.
    }

importacao-n-lixeira = { $n ->
        [one] { $n } archivo de Descargas fue a la Papelera
       *[other] { $n } archivos de Descargas fueron a la Papelera
    }
importacao-erro-lixeira = No se pudo borrar de Descargas

## Aviso de la carpeta Descargas

importacao-downloads-regiao = Archivos nuevos en Descargas
importacao-downloads-titulo = { $n ->
        [one] Archivo nuevo en Descargas
       *[other] { $n } archivos nuevos en Descargas
    }
importacao-downloads-dispensar = Descartar { $nome }
importacao-downloads-apagar = Borrar de Descargas después de importar
importacao-downloads-importar = { $n ->
        [one] Importar
       *[other] Importar { $n }
    }
importacao-downloads-agora-nao = Ahora no
importacao-erro-apagar-arquivo = No se pudo borrar el archivo.
importacao-erro-ler-arquivo = No se pudo leer el archivo.
importacao-erro-nao-encontrado = { $nome }: archivo no encontrado

## Pestaña Importaciones

importacao-importacoes = Importaciones
importacao-col-quando = Cuándo
importacao-col-arquivo = Archivo
importacao-col-conta = Cuenta
importacao-col-inseridos = Agregados
importacao-col-duplicados = Duplicados
importacao-col-erros = Errores
importacao-col-acoes = Acciones
importacao-erro-linha = línea { $linha }: { $mensagem }
importacao-ver-lancamentos = Ver movimientos
importacao-desfeita = Importación deshecha
importacao-erro-desfazer = No se pudo deshacer
importacao-desfazer = Deshacer
importacao-importar-extrato = Importar extracto
importacao-vazio-kicker = Ninguna importación
importacao-vazio-titulo = Trae el extracto del banco
importacao-vazio-descricao = OFX o CSV de cuenta y de tarjeta, con detección de duplicados y tus reglas de categorización.

## Asistente de extractos: pasos y archivo

importacao-etapas = Pasos
importacao-etapa-arquivo = Archivo
importacao-etapa-leitura = Lectura
importacao-etapa-previa = Vista previa
importacao-conta-destino = Cuenta de destino
importacao-escolha-conta = Elige la cuenta…
importacao-escolha-conta-destino = Elige la cuenta de destino
importacao-arquivo-banco = Archivo del banco
importacao-arquivo-banco-dica = OFX (extracto o resumen) o CSV. El archivo queda solo en la memoria de la app hasta que confirmes.
importacao-lendo-arquivo = Leyendo el archivo…
importacao-gerar-previa = Generar vista previa

## Lectura: cuenta reconocida por el OFX

importacao-ofx-lembrada = Cuenta recordada de la última importación de este archivo.
importacao-ofx-lembrada-trocada = Cuenta recordada de la última importación de este archivo (cambiada respecto de la elegida antes).
importacao-ofx-reconhecida = Reconocida por el banco, la sucursal y el número del archivo.
importacao-ofx-reconhecida-trocada = Reconocida por el banco, la sucursal y el número del archivo (cambiada respecto de la elegida antes).
# $motivo viene del núcleo.
importacao-ofx-nao-reconheci = No reconocí la cuenta ({ $motivo }). Elige en qué cuenta importar.
importacao-ofx-nao-reconheci-lembra = No reconocí la cuenta ({ $motivo }). Elige en qué cuenta importar; la elección queda recordada para los próximos archivos de esta cuenta.
importacao-csv-sem-conta = El CSV no dice de qué cuenta es: revisa la cuenta antes de la vista previa.
importacao-pdf-conta-cartao = El resumen en PDF entra en la cuenta de la tarjeta.
importacao-pdf-conta-extrato = El extracto en PDF entra en la cuenta corriente del banco.
importacao-pdf-extrato-titulo = Extracto de cuenta en PDF
importacao-pdf-extrato-lidos = { $n } movimiento(s) leídos ({ $leiaute }).
importacao-pdf-extrato-lidos-ocr = { $n } movimiento(s) leídos en la computadora por OCR ({ $leiaute }), sin IA. Revisa los montos en la vista previa.
importacao-pdf-extrato-nao-cartao = La cuenta elegida es una tarjeta: elige la cuenta corriente de este extracto.
importacao-ref-gravada = Sucursal y cuenta guardadas
importacao-ref-gravada-descricao = Los próximos OFX de este banco van directo a { $conta }.

## Lectura: datos del archivo

importacao-info-arquivo = Archivo
importacao-info-formato = Formato
importacao-info-codificacao = Codificación
importacao-info-linhas = Filas
importacao-ofx-fatura = Resumen de tarjeta (OFX)
importacao-ofx-extrato = Extracto de cuenta (OFX)
importacao-ofx-banco = Banco
importacao-ofx-conta-arquivo = Cuenta en el archivo
importacao-ofx-periodo = Período
importacao-ofx-periodo-valor = { $inicio } a { $fim }
importacao-ofx-saldo = Saldo informado
importacao-banco-arquivo = Banco del archivo: { $banco }
importacao-gravar-ref = Guardar sucursal y cuenta en { $conta }
importacao-gravar-ref-dica = Así los próximos OFX de este banco caen solos en esta cuenta.
importacao-ofx-cartao-conta = El archivo es de tarjeta de crédito y la cuenta elegida no es una tarjeta.
importacao-ofx-conta-cartao = La cuenta elegida es una tarjeta, pero el archivo es un extracto de cuenta.

## Lectura: resumen en PDF

importacao-pdf-titulo = Resumen de tarjeta (PDF)
importacao-pdf-lidos = { $n ->
        [one] { $n } movimiento leído. Va al resumen del vencimiento impreso en el PDF, y el total del resumen se verifica en la vista previa.
       *[other] { $n } movimientos leídos. Van al resumen del vencimiento impreso en el PDF, y el total del resumen se verifica en la vista previa.
    }
importacao-pdf-protegido = El PDF parece protegido con contraseña (o no tiene texto). Ingresa la contraseña del resumen, en general los primeros dígitos del CPF.
importacao-pdf-senha = Contraseña del PDF (si tiene)
importacao-pdf-senha-dica = Se usa solo para leer el archivo ahora; no se guarda.
importacao-pdf-nao-cartao = La cuenta elegida no es una tarjeta de crédito: elige la tarjeta del resumen.

## Lectura: columnas del CSV

importacao-csv-modelo = Modelo del banco
importacao-csv-modelo-reconhecido = Reconocido por el encabezado.
importacao-csv-personalizado = Personalizado
importacao-csv-separador = Separador
importacao-csv-pular = Filas a omitir
importacao-csv-data = Fecha
importacao-csv-formato-data = Formato de la fecha
importacao-csv-descricao = Descripción
importacao-csv-valor = Monto
importacao-csv-valor-unico = Una columna de monto
importacao-csv-valor-separado = Débito y crédito separados
importacao-csv-valor-sinal = Monto (con signo)
importacao-csv-debito = Débito (salidas)
importacao-csv-credito = Crédito (entradas)
importacao-csv-identificador = Identificador (opcional)
importacao-csv-identificador-dica = Evita duplicados al volver a importar.
importacao-csv-cabecalho = La primera fila es el encabezado
importacao-csv-virgula = Coma decimal (1.234,56)
importacao-csv-inverter = Invertir signos (resumen con compras en positivo)
importacao-csv-coluna = Columna { $n }
importacao-amostra = Muestra del archivo

importacao-sep-ponto-virgula = Punto y coma (;)
importacao-sep-virgula = Coma (,)
importacao-sep-tabulacao = Tabulación
importacao-sep-barra = Barra vertical (|)

importacao-data-dmy = dd/mm/aaaa
importacao-data-ymd = aaaa-mm-dd
importacao-data-dmy-curto = dd/mm/aa
importacao-data-dmy-hifen = dd-mm-aaaa
importacao-data-dmy-ponto = dd.mm.aaaa
importacao-data-dmy-ou-ymd = dd/mm/aaaa o aaaa-mm-dd
importacao-data-mdy = mm/dd/aaaa (EE. UU.)
importacao-data-mdy-curto = mm/dd/aa (EE. UU.)
importacao-data-mdy-hifen = mm-dd-aaaa (EE. UU.)

## Vista previa

importacao-troca-conta-dica = Cambiar la cuenta rehace la vista previa (los duplicados se verifican en la cuenta elegida).
importacao-filtrar-linhas = Filtrar filas de la vista previa
importacao-todas = Todas
importacao-previa = Vista previa de la importación
importacao-col-importar = Importar
importacao-col-situacao = Estado
importacao-col-data = Fecha
importacao-col-descricao = Descripción
importacao-col-categoria = Categoría
importacao-col-valor = Monto
importacao-fatura = Resumen
importacao-cartao = Tarjeta
importacao-fatura-vence = { $emissor }, vence el { $data }
importacao-total-impresso = Total impreso
importacao-lido-linhas = Leído en las filas
importacao-confere = ✓ coincide
importacao-layout = Diseño
importacao-avisos = Avisos
importacao-incluir-possiveis = { $n ->
        [one] Importar también el { $n } posible duplicado
       *[other] Importar también los { $n } posibles duplicados
    }
importacao-incluir-possiveis-dica = Misma fecha y monto que un movimiento existente, con una descripción parecida.
importacao-importar-linha = Importar fila { $n }
importacao-descricao-linha = Descripción de la fila { $n }
importacao-categoria-linha = Categoría de la fila { $n }
importacao-sugestao-regra = sugerencia de regla
importacao-importar-n = { $n ->
        [one] Importar { $n } movimiento
       *[other] Importar { $n } movimientos
    }

## Resultado

importacao-n-importados = { $n ->
        [one] { $n } movimiento importado
       *[other] { $n } movimientos importados
    }
importacao-resultado = { $inseridos ->
        [one] { $inseridos } movimiento importado
       *[other] { $inseridos } movimientos importados
    } en { $conta }; { $pulados ->
        [one] { $pulados } fila quedó fuera.
       *[other] { $pulados } filas quedaron fuera.
    }
importacao-resultado-parcelas = { $n ->
        [one] { $n } cuota de contrato marcada como pagada por la categoría.
       *[other] { $n } cuotas de contrato marcadas como pagadas por la categoría.
    }
importacao-resultado-transferencias = { $n ->
        [one] { $n } transferencia entre tus cuentas ya vinculada.
       *[other] { $n } transferencias entre tus cuentas ya vinculadas.
    }
importacao-resultado-previstos = { $n ->
        [one] { $n } movimiento previsto quedó cumplido por el extracto.
       *[other] { $n } movimientos previstos quedaron cumplidos por el extracto.
    }
importacao-ver-importados = Ver movimientos importados

# Boleto, Pix e comprovante lidos no computador (docs/ocr-other-documents.md).
importacao-pagamento-titulo = Documento de pago
importacao-pagamento-titulo-boleto = Cuenta por pagar
importacao-pagamento-titulo-pix = Cobro Pix
importacao-pagamento-titulo-comprovante = Comprobante
importacao-pagamento-lendo = Leyendo en la computadora (código de barras, QR y texto)…
importacao-pagamento-revisar = Revisar
importacao-pagamento-pix-qr = Cobro Pix (QR): se vuelve una cuenta por pagar prevista
importacao-pagamento-nada = No encontré boleto, código Pix ni comprobante en esta imagen.
importacao-pagamento-pedir-agente = Pedir al agente
importacao-pagamento-linha = Línea digitable o código para pagar
importacao-pagamento-copiar = Copiar
importacao-pagamento-copiado = Copiado
importacao-pagamento-banco = Banco
importacao-pagamento-beneficiario = Beneficiario
importacao-pagamento-recebedor = Destinatario
importacao-pagamento-chave = Clave Pix
importacao-pagamento-descricao = Descripción
importacao-pagamento-valor = Monto
importacao-pagamento-vencimento = Vencimiento
importacao-pagamento-conta = Cuenta
importacao-pagamento-escolha-conta = Elige la cuenta…
importacao-pagamento-categoria = Categoría
importacao-pagamento-previsto-dica = Entra en las cuentas por vencer y en el recordatorio del vencimiento. La app no paga nada: copia la línea digitable en la app del banco.
importacao-pagamento-criar-previsto = Crear cuenta prevista
importacao-pagamento-previsto-criado = Cuenta por pagar prevista
importacao-pagamento-ja-lancado = Ya está registrada como prevista: { $descricao }, { $data }.
importacao-pagamento-tipo = Tipo
importacao-pagamento-tipo-pix = Pix
importacao-pagamento-tipo-transferencia = Transferencia
importacao-pagamento-tipo-boleto = Boleto pagado
importacao-pagamento-tipo-outro = Pago
importacao-pagamento-data = Fecha
importacao-pagamento-quem-recebeu = Quién recibió
importacao-pagamento-quem-pagou = Quién pagó
importacao-pagamento-instituicao = Institución
importacao-pagamento-e2e = ID de la transacción
importacao-pagamento-ligar-a = Vincular a
importacao-pagamento-nenhum = Ningún movimiento del extracto coincide (mismo monto, hasta 2 días de la fecha).
importacao-pagamento-criar-lancamento = Crear el movimiento
importacao-pagamento-usar-previsto = Usar la cuenta prevista como el pago
importacao-pagamento-quitar = Saldar la cuenta prevista { $descricao } ({ $data })
importacao-pagamento-favorecido = Beneficiario
importacao-pagamento-sem-imagem-dica = La imagen no se guarda: solo los datos leídos van a la nota del movimiento.
importacao-pagamento-salvar-comprovante = Guardar los datos del comprobante
importacao-pagamento-comprovante-salvo = Comprobante vinculado al movimiento
importacao-pagamento-ja-ligado = Este comprobante ya está vinculado a { $descricao } ({ $data }).
importacao-desfazer-historico = Para deshacerlo, usa Ajustes › Historial de cambios.

## Caixa de entrada de arquivos

caixa-titulo = Bandeja de entrada de archivos
caixa-explica = Los archivos puestos en { $pasta } (por ti o por un agente de IA) llegan aquí: Treasury los lee, los guarda en la bóveda y los saca de la carpeta. Importa cada uno cuando quieras.
caixa-desligada = El seguimiento de la bandeja de entrada está apagado (Ajustes › Preferencias).
caixa-mostrar-todos = Mostrar también los importados y descartados
caixa-vazia = Nada en la bandeja de entrada
caixa-vazia-texto = Pon un extracto, nota de corretaje o extracto de previsión en { $pasta } y aparece aquí en segundos.
caixa-lista = Archivos en la bandeja de entrada
caixa-importar = Importar
caixa-descartar = Descartar
caixa-descartar-nome = Descartar { $nome }
caixa-erro-abrir = No se pudo abrir el archivo
caixa-status-pendente = por importar
caixa-status-importado = importado
caixa-status-descartado = descartado
caixa-status-desconhecido = tipo no reconocido
caixa-aviso-novo = Nuevo archivo en la bandeja de entrada: { $tipo }
caixa-aviso-desconhecido = { $nome } llegó a la bandeja de entrada, pero no se reconoció el tipo
caixa-aviso-revisar = Revisar

## Documento como texto (Markdown)

documento-ver-texto = Ver como texto
documento-ver-texto-nome = Ver { $nome } como texto
documento-texto-titulo = { $nome } como texto
documento-texto-lendo = Leyendo el documento…
documento-texto-resumo = Así leen este documento los asistentes de IA: { $paginas } página(s), { $total } caracteres.
documento-texto-ocr = El PDF es solo imagen: el texto vino del OCR del ordenador y puede tener errores.
documento-texto-paginas-corridas = Páginas como texto corrido (la verificación no coincidió): { $paginas }.
documento-texto-conteudo = Contenido del documento
documento-texto-copiar = Copiar
documento-texto-copiado = Texto copiado
