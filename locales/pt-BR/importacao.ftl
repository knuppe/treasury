# Importação: janela única de importação, assistente de extrato (OFX, CSV e fatura em PDF), aba
# Importações e aviso da pasta Downloads (docs/i18n-design.md). pt-BR é o idioma de origem.
# Os nomes oficiais dos importadores só do Brasil (Área do Investidor B3, notas SINACOR, CPF)
# ficam como estão em qualquer idioma.

## Janela única de importação

importacao-arquivos-titulo = Importar arquivos
importacao-arquivos-descricao = Extratos OFX ou CSV, faturas de cartão e notas de corretagem em PDF, cupons fiscais (foto, PDF ou XML), relatórios da Área do Investidor B3 e fotos de comprovantes. O Treasury reconhece o arquivo no computador e abre o importador certo; a IA só entra no que ele não lê.
importacao-lendo-arquivos = Lendo os arquivos…
importacao-soltar = Solte os arquivos aqui ou clique para escolher
importacao-formatos = OFX, QFX, CSV, PDF, XML, XLSX, XLS ou ODS · imagens PNG, JPEG ou HEIC (lidas no computador)
importacao-email-dica = Recebe extratos e faturas por e-mail? { $link } e eles chegam sozinhos para revisar.
importacao-email-link = Configure o e-mail de ingestão

importacao-tipo-extratos = Extratos e faturas
importacao-tipo-notas = Notas de corretagem
importacao-tipo-b3 = Área do Investidor B3
importacao-tipo-previdencia = Extratos de previdência
importacao-tipo-imagens = Outras imagens (para o agente de IA)
importacao-tipo-cupons = Cupons fiscais e notas de compra
importacao-tipo-comprovantes = Comprovantes de pagamento e boletos
importacao-lendo-imagem = lendo no computador (QR e texto)…
importacao-cupom-qr = Cupom fiscal com QR: abre a nota na consulta oficial da SEFAZ
importacao-imagem-nao-lida = Não deu para ler a imagem no computador
importacao-revisar-compra = Revisar compra
importacao-registrar-agente = Registrar com o agente
importacao-agente-comprovantes = { $n ->
    [one] Registre este comprovante ou boleto no Treasury (lançamento ou conta a pagar).
   *[other] Registre estes { $n } comprovantes ou boletos no Treasury (lançamentos ou contas a pagar).
}
importacao-botao-topo = Importar
importacao-botao-topo-dica = Importar arquivos: extratos, faturas, notas, cupons, relatórios da B3 e imagens
importacao-tipo-desconhecidos = Não reconhecidos

importacao-importar = Importar
importacao-importar-notas = { $n ->
        [1] Importar nota
       *[other] Importar { $n } notas
    }
importacao-importar-b3 = Importar da B3
importacao-importar-agente = Importar com o agente
importacao-previdencia-dica = Importe na aba Previdência do ativo, em Ativos.
importacao-anexar-agente = Anexar ao agente
importacao-motor-imagens = Configure um motor de IA em Agente › Configurar para ler imagens.
importacao-analisar-ia = Analisar com IA
importacao-motor-analisar = Configure um motor de IA em Agente › Configurar para analisar estes arquivos.
importacao-fatura-cartao = É fatura de cartão
importacao-tirar = Tirar { $nome }

# Texto que vai pronto para o agente de IA, com os nomes dos arquivos.
importacao-agente-analisar = { $n ->
        [1] Analise o arquivo anexado ({ $nomes }) e me ajude a cadastrar no Treasury o que encontrar.
       *[other] Analise os arquivos anexados ({ $nomes }) e me ajude a cadastrar no Treasury o que encontrar.
    }

importacao-n-lixeira = { $n ->
        [one] { $n } arquivo de Downloads foi para a Lixeira
       *[other] { $n } arquivos de Downloads foram para a Lixeira
    }
importacao-erro-lixeira = Não foi possível apagar de Downloads

## Aviso da pasta Downloads

importacao-downloads-regiao = Arquivos novos em Downloads
importacao-downloads-titulo = { $n ->
        [1] Novo arquivo em Downloads
       *[other] { $n } arquivos novos em Downloads
    }
importacao-downloads-dispensar = Dispensar { $nome }
importacao-downloads-apagar = Apagar de Downloads depois de importar
importacao-downloads-importar = { $n ->
        [1] Importar
       *[other] Importar { $n }
    }
importacao-downloads-agora-nao = Agora não
importacao-erro-apagar-arquivo = Não foi possível apagar o arquivo.
importacao-erro-ler-arquivo = Não foi possível ler o arquivo.
importacao-erro-nao-encontrado = { $nome }: arquivo não encontrado

## Aba Importações

importacao-importacoes = Importações
importacao-col-quando = Quando
importacao-col-arquivo = Arquivo
importacao-col-conta = Conta
importacao-col-inseridos = Inseridos
importacao-col-duplicados = Duplicados
importacao-col-erros = Erros
importacao-col-acoes = Ações
importacao-erro-linha = linha { $linha }: { $mensagem }
importacao-ver-lancamentos = Ver lançamentos
importacao-desfeita = Importação desfeita
importacao-erro-desfazer = Não foi possível desfazer
importacao-desfazer = Desfazer
importacao-importar-extrato = Importar extrato
importacao-vazio-kicker = Nenhuma importação
importacao-vazio-titulo = Traga o extrato do banco
importacao-vazio-descricao = OFX ou CSV de conta e de cartão, com detecção de duplicados e as suas regras de categorização.

## Assistente de extrato: etapas e arquivo

importacao-etapas = Etapas
importacao-etapa-arquivo = Arquivo
importacao-etapa-leitura = Leitura
importacao-etapa-previa = Prévia
importacao-conta-destino = Conta de destino
importacao-escolha-conta = Escolha a conta…
importacao-escolha-conta-destino = Escolha a conta de destino
importacao-arquivo-banco = Arquivo do banco
importacao-arquivo-banco-dica = OFX (extrato ou fatura) ou CSV. O arquivo fica só na memória do app até você confirmar.
importacao-lendo-arquivo = Lendo o arquivo…
importacao-gerar-previa = Gerar prévia

## Leitura: conta reconhecida pelo OFX

importacao-ofx-lembrada = Conta lembrada da última importação deste arquivo.
importacao-ofx-lembrada-trocada = Conta lembrada da última importação deste arquivo (trocada da escolhida antes).
importacao-ofx-reconhecida = Reconhecida pelo banco, agência e número do arquivo.
importacao-ofx-reconhecida-trocada = Reconhecida pelo banco, agência e número do arquivo (trocada da escolhida antes).
# $motivo vem do núcleo.
importacao-ofx-nao-reconheci = Não reconheci a conta ({ $motivo }). Escolha em qual conta importar.
importacao-ofx-nao-reconheci-lembra = Não reconheci a conta ({ $motivo }). Escolha em qual conta importar; a escolha fica lembrada para os próximos arquivos desta conta.
importacao-csv-sem-conta = O CSV não diz de qual conta é: confira a conta antes da prévia.
importacao-pdf-conta-cartao = A fatura em PDF entra na conta do cartão.
importacao-pdf-conta-extrato = O extrato em PDF entra na conta corrente do banco.
importacao-pdf-extrato-titulo = Extrato de conta em PDF
importacao-pdf-extrato-lidos = { $n } lançamento(s) lidos ({ $leiaute }).
importacao-pdf-extrato-lidos-ocr = { $n } lançamento(s) lidos no computador pelo OCR ({ $leiaute }), sem IA. Confira os valores na prévia.
importacao-pdf-extrato-nao-cartao = A conta escolhida é um cartão: escolha a conta corrente deste extrato.
importacao-ref-gravada = Agência e conta gravadas
importacao-ref-gravada-descricao = Os próximos OFX deste banco vão direto para { $conta }.

## Leitura: dados do arquivo

importacao-info-arquivo = Arquivo
importacao-info-formato = Formato
importacao-info-codificacao = Codificação
importacao-info-linhas = Linhas
importacao-ofx-fatura = Fatura de cartão (OFX)
importacao-ofx-extrato = Extrato de conta (OFX)
importacao-ofx-banco = Banco
importacao-ofx-conta-arquivo = Conta no arquivo
importacao-ofx-periodo = Período
importacao-ofx-periodo-valor = { $inicio } a { $fim }
importacao-ofx-saldo = Saldo informado
importacao-banco-arquivo = Banco do arquivo: { $banco }
importacao-gravar-ref = Gravar agência e conta em { $conta }
importacao-gravar-ref-dica = Assim os próximos OFX deste banco caem nesta conta sozinhos.
importacao-ofx-cartao-conta = O arquivo é de cartão de crédito e a conta escolhida não é cartão.
importacao-ofx-conta-cartao = A conta escolhida é cartão, mas o arquivo é extrato de conta.

## Leitura: fatura em PDF

importacao-pdf-titulo = Fatura de cartão (PDF)
importacao-pdf-lidos = { $n } lançamento(s) lidos. Eles vão para a fatura do vencimento impresso no PDF, e o total da fatura é conferido na prévia.
importacao-pdf-protegido = O PDF parece protegido por senha (ou não traz texto). Informe a senha da fatura, em geral os primeiros dígitos do CPF.
importacao-pdf-senha = Senha do PDF (se tiver)
importacao-pdf-senha-dica = Usada só para ler o arquivo agora; não fica guardada.
importacao-pdf-nao-cartao = A conta escolhida não é um cartão de crédito: escolha o cartão da fatura.

## Leitura: colunas do CSV

importacao-csv-modelo = Modelo do banco
importacao-csv-modelo-reconhecido = Reconhecido pelo cabeçalho.
importacao-csv-personalizado = Personalizado
importacao-csv-separador = Separador
importacao-csv-pular = Linhas a pular
importacao-csv-data = Data
importacao-csv-formato-data = Formato da data
importacao-csv-descricao = Descrição
importacao-csv-valor = Valor
importacao-csv-valor-unico = Uma coluna de valor
importacao-csv-valor-separado = Débito e crédito separados
importacao-csv-valor-sinal = Valor (com sinal)
importacao-csv-debito = Débito (saídas)
importacao-csv-credito = Crédito (entradas)
importacao-csv-identificador = Identificador (opcional)
importacao-csv-identificador-dica = Evita duplicados ao reimportar.
importacao-csv-cabecalho = A primeira linha é cabeçalho
importacao-csv-virgula = Vírgula decimal (1.234,56)
importacao-csv-inverter = Inverter sinais (fatura com compras positivas)
importacao-csv-coluna = Coluna { $n }
importacao-amostra = Amostra do arquivo

importacao-sep-ponto-virgula = Ponto e vírgula (;)
importacao-sep-virgula = Vírgula (,)
importacao-sep-tabulacao = Tabulação
importacao-sep-barra = Barra vertical (|)

importacao-data-dmy = dd/mm/aaaa
importacao-data-ymd = aaaa-mm-dd
importacao-data-dmy-curto = dd/mm/aa
importacao-data-dmy-hifen = dd-mm-aaaa
importacao-data-dmy-ponto = dd.mm.aaaa
importacao-data-dmy-ou-ymd = dd/mm/aaaa ou aaaa-mm-dd
importacao-data-mdy = mm/dd/aaaa (EUA)
importacao-data-mdy-curto = mm/dd/aa (EUA)
importacao-data-mdy-hifen = mm-dd-aaaa (EUA)

## Prévia

importacao-troca-conta-dica = Trocar a conta refaz a prévia (os duplicados são conferidos na conta escolhida).
importacao-filtrar-linhas = Filtrar linhas da prévia
importacao-todas = Todas
importacao-previa = Prévia da importação
importacao-col-importar = Importar
importacao-col-situacao = Situação
importacao-col-data = Data
importacao-col-descricao = Descrição
importacao-col-categoria = Categoria
importacao-col-valor = Valor
importacao-fatura = Fatura
importacao-cartao = Cartão
importacao-fatura-vence = { $emissor }, vence { $data }
importacao-total-impresso = Total impresso
importacao-lido-linhas = Lido nas linhas
importacao-confere = ✓ confere
importacao-layout = Layout
importacao-avisos = Avisos
importacao-incluir-possiveis = Importar também os { $n } possíveis duplicados
importacao-incluir-possiveis-dica = Mesma data e valor de um lançamento existente, com descrição parecida.
importacao-importar-linha = Importar linha { $n }
importacao-descricao-linha = Descrição da linha { $n }
importacao-categoria-linha = Categoria da linha { $n }
importacao-sugestao-regra = sugestão de regra
importacao-importar-n = Importar { $n } lançamento(s)

## Resultado

importacao-n-importados = { $n ->
        [one] { $n } lançamento importado
       *[other] { $n } lançamentos importados
    }
importacao-resultado = { $inseridos ->
        [one] { $inseridos } lançamento importado
       *[other] { $inseridos } lançamentos importados
    } em { $conta }; { $pulados ->
        [0] nenhuma linha ficou de fora.
        [one] { $pulados } linha ficou de fora.
       *[other] { $pulados } linhas ficaram de fora.
    }
importacao-resultado-parcelas = { $n } parcela(s) de contrato marcada(s) como paga(s) pela categoria.
importacao-resultado-transferencias = { $n } transferência(s) entre suas contas já ligada(s).
importacao-resultado-previstos = { $n ->
        [one] { $n } lançamento previsto foi efetivado pelo extrato.
       *[other] { $n } lançamentos previstos foram efetivados pelo extrato.
    }
importacao-ver-importados = Ver lançamentos importados

# Boleto, Pix e comprovante lidos no computador (docs/ocr-other-documents.md).
importacao-pagamento-titulo = Documento de pagamento
importacao-pagamento-titulo-boleto = Conta a pagar
importacao-pagamento-titulo-pix = Cobrança Pix
importacao-pagamento-titulo-comprovante = Comprovante
importacao-pagamento-lendo = Lendo no computador (código de barras, QR e texto)…
importacao-pagamento-revisar = Revisar
importacao-pagamento-pix-qr = Cobrança Pix (QR): vira uma conta a pagar prevista
importacao-pagamento-nada = Não achei boleto, código Pix nem comprovante nesta imagem.
importacao-pagamento-pedir-agente = Pedir ao agente
importacao-pagamento-linha = Linha digitável ou código para pagar
importacao-pagamento-copiar = Copiar
importacao-pagamento-copiado = Copiado
importacao-pagamento-banco = Banco
importacao-pagamento-beneficiario = Beneficiário
importacao-pagamento-recebedor = Recebedor
importacao-pagamento-chave = Chave Pix
importacao-pagamento-descricao = Descrição
importacao-pagamento-valor = Valor
importacao-pagamento-vencimento = Vencimento
importacao-pagamento-conta = Conta
importacao-pagamento-escolha-conta = Escolha a conta…
importacao-pagamento-categoria = Categoria
importacao-pagamento-previsto-dica = Entra nas contas a vencer e no lembrete do vencimento. O app não paga nada: copie a linha digitável no app do banco.
importacao-pagamento-criar-previsto = Criar conta prevista
importacao-pagamento-previsto-criado = Conta a pagar prevista
importacao-pagamento-ja-lancado = Já está lançado como previsto: { $descricao }, { $data }.
importacao-pagamento-tipo = Tipo
importacao-pagamento-tipo-pix = Pix
importacao-pagamento-tipo-transferencia = Transferência
importacao-pagamento-tipo-boleto = Boleto pago
importacao-pagamento-tipo-outro = Pagamento
importacao-pagamento-data = Data
importacao-pagamento-quem-recebeu = Quem recebeu
importacao-pagamento-quem-pagou = Quem pagou
importacao-pagamento-instituicao = Instituição
importacao-pagamento-e2e = ID da transação
importacao-pagamento-ligar-a = Ligar a
importacao-pagamento-nenhum = Nenhum lançamento do extrato combina (mesmo valor, até 2 dias da data).
importacao-pagamento-criar-lancamento = Criar o lançamento
importacao-pagamento-usar-previsto = Usar a conta prevista como o pagamento
importacao-pagamento-quitar = Quitar a conta prevista { $descricao } ({ $data })
importacao-pagamento-favorecido = Favorecido
importacao-pagamento-sem-imagem-dica = A imagem não é guardada: só os dados lidos vão para a nota do lançamento.
importacao-pagamento-salvar-comprovante = Guardar os dados do comprovante
importacao-pagamento-comprovante-salvo = Comprovante ligado ao lançamento
importacao-pagamento-ja-ligado = Este comprovante já está ligado a { $descricao } ({ $data }).
importacao-desfazer-historico = Para desfazer, use Ajustes › Histórico.

## Caixa de entrada de arquivos

caixa-titulo = Caixa de entrada de arquivos
caixa-explica = Arquivos colocados em { $pasta } (por você ou por um agente de IA) entram aqui: o Treasury lê, guarda no cofre e tira da pasta. Importe cada um quando quiser.
caixa-desligada = O acompanhamento da caixa de entrada está desligado (Ajustes › Preferências).
caixa-mostrar-todos = Mostrar também os importados e descartados
caixa-vazia = Nada na caixa de entrada
caixa-vazia-texto = Coloque um extrato, nota de corretagem ou extrato de previdência em { $pasta } e ele aparece aqui em segundos.
caixa-lista = Arquivos na caixa de entrada
caixa-importar = Importar
caixa-descartar = Descartar
caixa-descartar-nome = Descartar { $nome }
caixa-erro-abrir = Não deu para abrir o arquivo
caixa-status-pendente = para importar
caixa-status-importado = importado
caixa-status-descartado = descartado
caixa-status-desconhecido = tipo não reconhecido
caixa-aviso-novo = Novo arquivo na caixa de entrada: { $tipo }
caixa-aviso-desconhecido = { $nome } chegou na caixa de entrada, mas o tipo não foi reconhecido
caixa-aviso-revisar = Revisar

## Documento como texto (Markdown)

documento-ver-texto = Ver como texto
documento-ver-texto-nome = Ver { $nome } como texto
documento-texto-titulo = { $nome } como texto
documento-texto-lendo = Lendo o documento…
documento-texto-resumo = É assim que os assistentes de IA leem este documento: { $paginas } página(s), { $total } caracteres.
documento-texto-ocr = O PDF é só imagem: o texto veio do OCR do computador e pode ter erros.
documento-texto-paginas-corridas = Páginas como texto corrido (a conferência não bateu): { $paginas }.
documento-texto-conteudo = Conteúdo do documento
documento-texto-copiar = Copiar
documento-texto-copiado = Texto copiado
