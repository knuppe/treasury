# Variante pt-PT (importação de ficheiros e assistente de extrato): o que faltar cai para pt-BR.
# Revisão: rascunho por IA. Os nomes oficiais dos importadores só do Brasil (Área do Investidor B3,
# notas SINACOR, CPF) ficam como estão.

## Janela única de importação

importacao-arquivos-titulo = Importar ficheiros
importacao-arquivos-descricao = Extratos OFX ou CSV, extratos de cartão e notas de corretagem em PDF e relatórios da Área do Investidor B3. O Treasury reconhece o formato e abre o importador certo.
importacao-lendo-arquivos = A ler os ficheiros…
importacao-soltar = Largue os ficheiros aqui ou clique para escolher
importacao-email-dica = Recebe extratos e faturas por e-mail? { $link } e chegam automaticamente para rever.

importacao-tipo-imagens = Imagens (comprovativos, recibos, faturas, capturas de ecrã)

importacao-previdencia-dica = Importe no separador Previdência do ativo, em Ativos.
importacao-motor-analisar = Configure um motor de IA em Agente › Configurar para analisar estes ficheiros.
importacao-fatura-cartao = É extrato de cartão
importacao-tirar = Remover { $nome }

# Texto que vai pronto para o agente de IA, com os nomes dos ficheiros.
importacao-agente-analisar = { $n ->
        [1] Analise o ficheiro anexado ({ $nomes }) e ajude-me a registar no Treasury o que encontrar.
       *[other] Analise os ficheiros anexados ({ $nomes }) e ajude-me a registar no Treasury o que encontrar.
    }

importacao-n-lixeira = { $n ->
        [one] { $n } ficheiro de Transferências foi para o Lixo
       *[other] { $n } ficheiros de Transferências foram para o Lixo
    }
importacao-erro-lixeira = Não foi possível eliminar de Transferências

## Aviso da pasta Transferências

importacao-downloads-regiao = Ficheiros novos em Transferências
importacao-downloads-titulo = { $n ->
        [1] Novo ficheiro em Transferências
       *[other] { $n } ficheiros novos em Transferências
    }
importacao-downloads-apagar = Eliminar de Transferências depois de importar
importacao-erro-apagar-arquivo = Não foi possível eliminar o ficheiro.
importacao-erro-ler-arquivo = Não foi possível ler o ficheiro.
importacao-erro-nao-encontrado = { $nome }: ficheiro não encontrado

## Separador Importações

importacao-col-arquivo = Ficheiro
importacao-vazio-descricao = OFX ou CSV de conta e de cartão, com deteção de duplicados e as suas regras de categorização.

## Assistente de extrato: etapas e ficheiro

importacao-etapa-arquivo = Ficheiro
importacao-etapa-previa = Pré-visualização
importacao-arquivo-banco = Ficheiro do banco
importacao-arquivo-banco-dica = OFX (extrato de conta ou de cartão) ou CSV. O ficheiro fica só na memória da aplicação até confirmar.
importacao-lendo-arquivo = A ler o ficheiro…
importacao-gerar-previa = Gerar pré-visualização

## Leitura: conta reconhecida pelo OFX

importacao-ofx-lembrada = Conta memorizada da última importação deste ficheiro.
importacao-ofx-lembrada-trocada = Conta memorizada da última importação deste ficheiro (substituiu a que estava escolhida).
importacao-ofx-reconhecida = Reconhecida pelo banco, balcão e número do ficheiro.
importacao-ofx-reconhecida-trocada = Reconhecida pelo banco, balcão e número do ficheiro (substituiu a que estava escolhida).
# $motivo vem do núcleo.
importacao-ofx-nao-reconheci = Não reconheci a conta ({ $motivo }). Escolha a conta onde importar.
importacao-ofx-nao-reconheci-lembra = Não reconheci a conta ({ $motivo }). Escolha a conta onde importar; a escolha fica memorizada para os próximos ficheiros desta conta.
importacao-csv-sem-conta = O CSV não diz de que conta é: verifique a conta antes da pré-visualização.
importacao-pdf-conta-cartao = O extrato em PDF entra na conta do cartão.
importacao-pdf-conta-extrato = O extrato em PDF entra na conta à ordem do banco.
importacao-pdf-extrato-titulo = Extrato de conta em PDF
importacao-pdf-extrato-lidos =
    { $n ->
        [one] { $n } movimento lido ({ $leiaute }).
       *[other] { $n } movimentos lidos ({ $leiaute }).
    }
importacao-pdf-extrato-lidos-ocr =
    { $n ->
        [one] { $n } movimento lido no computador pelo OCR ({ $leiaute }), sem IA. Confira os valores na pré-visualização.
       *[other] { $n } movimentos lidos no computador pelo OCR ({ $leiaute }), sem IA. Confira os valores na pré-visualização.
    }
importacao-pdf-extrato-nao-cartao = A conta escolhida é um cartão: escolha a conta à ordem deste extrato.
importacao-ref-gravada = Balcão e conta gravados
importacao-ref-gravada-descricao = Os próximos OFX deste banco vão diretamente para { $conta }.

## Leitura: dados do ficheiro

importacao-info-arquivo = Ficheiro
importacao-ofx-conta-arquivo = Conta no ficheiro
importacao-ofx-fatura = Extrato de cartão (OFX)
importacao-banco-arquivo = Banco do ficheiro: { $banco }
importacao-gravar-ref = Gravar balcão e conta em { $conta }
importacao-gravar-ref-dica = Assim, os próximos OFX deste banco entram automaticamente nesta conta.
importacao-ofx-cartao-conta = O ficheiro é de cartão de crédito e a conta escolhida não é um cartão.
importacao-ofx-conta-cartao = A conta escolhida é um cartão, mas o ficheiro é um extrato de conta.

## Leitura: extrato de cartão em PDF

importacao-pdf-titulo = Extrato de cartão (PDF)
importacao-pdf-lidos =
    { $n ->
        [one] { $n } lançamento lido. Vai para o extrato com o vencimento impresso no PDF, e o total do extrato é verificado na pré-visualização.
       *[other] { $n } lançamentos lidos. Vão para o extrato com o vencimento impresso no PDF, e o total do extrato é verificado na pré-visualização.
    }
importacao-pdf-protegido = O PDF parece protegido por palavra-passe (ou não tem texto). Indique a palavra-passe do extrato, em geral os primeiros dígitos do CPF.
importacao-pdf-senha = Palavra-passe do PDF (se tiver)
importacao-pdf-senha-dica = Usada só para ler o ficheiro agora; não fica guardada.
importacao-pdf-nao-cartao = A conta escolhida não é um cartão de crédito: escolha o cartão do extrato.

## Leitura: colunas do CSV

importacao-csv-pular = Linhas a saltar
importacao-csv-cabecalho = A primeira linha é o cabeçalho
importacao-csv-inverter = Inverter sinais (extrato de cartão com compras positivas)
importacao-amostra = Amostra do ficheiro

## Pré-visualização

importacao-troca-conta-dica = Trocar a conta refaz a pré-visualização (os duplicados são verificados na conta escolhida).
importacao-filtrar-linhas = Filtrar linhas da pré-visualização
importacao-previa = Pré-visualização da importação
importacao-fatura = Extrato
importacao-confere = ✓ bate certo

## Resultado

importacao-resultado-parcelas = { $n } prestação(ões) de contrato marcada(s) como paga(s) pela categoria.
importacao-resultado-transferencias =
    { $n ->
        [one] { $n } transferência entre as suas contas já ligada.
       *[other] { $n } transferências entre as suas contas já ligadas.
    }
importacao-resultado-previstos = { $n ->
        [one] { $n } movimento previsto foi efetivado pelo extrato.
       *[other] { $n } movimentos previstos foram efetivados pelo extrato.
    }
# Boleto, Pix e comprovante lidos no computador (docs/ocr-other-documents.md).
importacao-pagamento-titulo = Documento de pagamento
importacao-pagamento-titulo-boleto = Conta a pagar
importacao-pagamento-titulo-pix = Cobrança Pix
importacao-pagamento-titulo-comprovante = Comprovativo
importacao-pagamento-lendo = A ler no computador (código de barras, QR e texto)…
importacao-pagamento-revisar = Rever
importacao-pagamento-pix-qr = Cobrança Pix (QR): passa a conta a pagar prevista
importacao-pagamento-nada = Não encontrei boleto, código Pix nem comprovativo nesta imagem.
importacao-pagamento-pedir-agente = Pedir ao agente
importacao-pagamento-linha = Linha digitável ou código para pagar
importacao-pagamento-copiar = Copiar
importacao-pagamento-copiado = Copiado
importacao-pagamento-banco = Banco
importacao-pagamento-beneficiario = Beneficiário
importacao-pagamento-recebedor = Destinatário
importacao-pagamento-chave = Chave Pix
importacao-pagamento-descricao = Descrição
importacao-pagamento-valor = Valor
importacao-pagamento-vencimento = Vencimento
importacao-pagamento-conta = Conta
importacao-pagamento-escolha-conta = Escolha a conta…
importacao-pagamento-categoria = Categoria
importacao-pagamento-previsto-dica = Entra nas contas a vencer e no lembrete do vencimento. A aplicação não paga nada: copie a linha digitável na app do banco.
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
importacao-pagamento-nenhum = Nenhum movimento do extrato coincide (mesmo valor, até 2 dias da data).
importacao-pagamento-criar-lancamento = Criar o movimento
importacao-pagamento-usar-previsto = Usar a conta prevista como o pagamento
importacao-pagamento-quitar = Liquidar a conta prevista { $descricao } ({ $data })
importacao-pagamento-favorecido = Beneficiário
importacao-pagamento-sem-imagem-dica = A imagem não é guardada: só os dados lidos vão para a nota do movimento.
importacao-pagamento-salvar-comprovante = Guardar os dados do comprovativo
importacao-pagamento-comprovante-salvo = Comprovativo ligado ao movimento
importacao-pagamento-ja-ligado = Este comprovativo já está ligado a { $descricao } ({ $data }).
importacao-desfazer-historico = Para anular, use Ajustes › Histórico.
