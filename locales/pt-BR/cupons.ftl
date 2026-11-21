# Cupons fiscais (06 · Consumo › Compras): leitura da NFC-e (link do QR, foto, PDF, XML ou texto),
# itens com o grupo e o imposto e o que a família compra (só no Brasil; docs/receipts.md,
# docs/i18n-design.md). Termos oficiais (NFC-e, DANFE, SEFAZ, XML) ficam
# no original. pt-BR é o idioma de origem.

## Prévia do cupom

cupons-cupom-de = Cupom de { $loja }
cupons-cupom-fiscal = Cupom fiscal
cupons-sem-data = sem data
cupons-previa-impresso = { $data } · { $n } item(ns) · imposto impresso na nota
cupons-previa-estimado = { $data } · { $n } item(ns) · imposto estimado pelo percentual de cada grupo de consumo
cupons-itens = Itens do cupom
cupons-item = Item
cupons-grupo = Grupo
cupons-grupo-de = Grupo de { $item }
cupons-valor = Valor
cupons-imposto = Imposto
cupons-pago-valor = Pago: { $valor }
cupons-desconto = (desconto de { $valor })
cupons-imposto-valor = Imposto: { $valor }
cupons-tributos-nota = (tributos aproximados da nota, Lei 12.741/2012)
cupons-estimado-itens = (estimado item a item)
cupons-lancamento = Lançamento: { $descricao } em { $data } ({ $conta })
cupons-sem-lancamento = Nenhum lançamento com esse valor perto da data: o cupom fica sem ligação.
cupons-quem-comprou = Quem comprou
cupons-familia = Família
cupons-gravado = Cupom gravado
cupons-gravado-texto = { $n } itens de { $loja }
cupons-loja = loja
cupons-substituir = Substituir cupom
cupons-gravar = Gravar cupom

## Ler cupom

cupons-ler-titulo = Ler cupom fiscal
cupons-ler-texto = O QR do cupom (NFC-e) leva à consulta da SEFAZ, com os itens e os tributos aproximados da nota. Cole o link (a câmera do celular mostra ao ler o QR), mande a foto do QR, o PDF do cupom ou o XML da nota. Cupom sem QR: mande a foto ao agente, que lê os itens.
cupons-link = Link do QR do cupom
cupons-ler = Ler cupom
cupons-foto = Foto do cupom
cupons-foto-dica = A imagem é lida aqui mesmo e não é guardada.
cupons-qr-nao-achado = Não achei o QR na foto. Lendo o cupom pelo texto da foto…
cupons-pdf = PDF do cupom (DANFE NFC-e) ou XML da nota
cupons-esconder-texto = Esconder o texto
cupons-colar-texto = Colar o texto da página da consulta
cupons-texto = Texto do cupom
cupons-texto-placeholder = Abra o link do QR no navegador, selecione tudo e cole aqui.
cupons-ler-texto-botao = Ler o texto

## O que a família compra

cupons-periodo-mes = Este mês
cupons-periodo-3m = Últimos 3 meses
cupons-periodo-ano = Este ano
cupons-familia-compra = O que a família compra
cupons-periodo-compras = Período das compras
cupons-resumo = { $n } cupom(ns), { $gasto } em compras e { $imposto } de imposto ({ $pct } do que foi pago).
cupons-resumo-impresso = { $n } cupom(ns), { $gasto } em compras e { $imposto } de imposto ({ $pct } do que foi pago; parte dele impresso nas notas).
cupons-compras-grupo = Compras por grupo
cupons-gasto = Gasto
cupons-pct-compras = % das compras
cupons-compras-de = Compras de { $membro }
cupons-itens-pesam = Itens que mais pesam
cupons-vazio-familia = Nenhum cupom no período. Leia o QR dos cupons do mercado para ver o que cada um compra e quanto de imposto vai em cada grupo.

## Cupom pela foto, sem QR (docs/ocr-receipts.md)
cupons-foto-dica-qr = Primeiro o QR; sem QR, o texto do cupom. A imagem é lida aqui mesmo e não é guardada.
cupons-foto-lendo = Lendo o cupom…
cupons-foto-titulo = Ler cupom pela foto
cupons-foto-texto = Tire uma foto reta e com boa luz do cupom inteiro. Os itens são lidos neste computador e você revisa antes de gravar.
cupons-foto-leitura = Leitura da foto
cupons-foto-faixa-conferido = Conferido: a soma dos itens bate com o total, e a quantidade de itens ou o pagamento também.
cupons-foto-faixa-fechou = A soma dos itens bate com o total. Dê uma olhada nos itens.
cupons-foto-faixa-nao-fechou = A soma dos itens não bate com o total. Confira e corrija os itens.
cupons-foto-motor-vision = Lido pelo Apple Vision.
cupons-foto-motor-paddle = Lido pelo leitor do Treasury.
cupons-foto-motor-ia = Lido pela IA.
cupons-foto-total-impresso = Total no cupom: { $valor }.
cupons-foto-faltam = Faltam { $valor } nos itens.
cupons-foto-sobram = Os itens passam { $valor }.
cupons-foto-pedir-ia = Pedir ajuda à IA
cupons-foto-pedir-ia-dica = A foto vai para o motor de Ajustes › IA; se ele for remoto, ela sai do computador.

## Consulta numa janela do app, para o captcha (docs/ocr-receipts.md §4)
cupons-sefaz-abrir = Abrir a consulta aqui
cupons-sefaz-dica = Se a SEFAZ pedir captcha, abra a consulta aqui no app: resolva o captcha na janela e o cupom é lido sozinho.
cupons-sefaz-aberta = Resolva o captcha na janela da SEFAZ. Quando a página do cupom aparecer, ela é lida sozinha.
cupons-sefaz-ler-pagina = Ler a página agora
cupons-sefaz-erro-link = O link precisa ser o da consulta da SEFAZ (https, num site .gov.br).

## A foto ao lado da compra
cupons-foto-imagem = Foto do cupom
cupons-foto-imagem-dica = Arraste para mover, use a roda do mouse ou os botões para aproximar. Duplo clique aproxima.
cupons-foto-aproximar = Aproximar
cupons-foto-afastar = Afastar
cupons-foto-girar = Girar a foto
cupons-foto-ajustar = Ajustar à tela

## Leitor de cupons (ReceiptReader)

cupons-area = Ler cupom fiscal
cupons-area-titulo = Solte aqui a foto, o PDF ou o XML do cupom
cupons-area-titulo-foto = Solte aqui a foto do cupom
cupons-area-dica = Ou escolha o arquivo, ou cole o link do QR. O app descobre o que é: o QR leva à SEFAZ (ou à AFIP, nas notas da Argentina) e, sem QR, os itens saem do texto da foto, lido neste computador.
cupons-area-dica-foto = Ou escolha o arquivo. Os itens saem do texto da foto, lido neste computador; o QR da AFIP (Argentina) também vale.
cupons-escolher-arquivo = Escolher arquivo
cupons-link-placeholder = ou cole o link do QR (https://…)
cupons-etapas = Etapas da leitura
cupons-etapa-qr = Procurar o QR na foto
cupons-etapa-qr-nao-achado = sem QR legível, lendo os itens pela foto
cupons-etapa-link = Reconhecer o link
cupons-etapa-sefaz = Consultar a nota na SEFAZ
cupons-etapa-afip = Ler a nota pelo QR (Argentina, Portugal, Espanha)
cupons-etapa-afip-sem-itens = o QR não traz os itens: inclua-os na revisão
cupons-etapa-ocr = Ler os itens pelo texto da foto
cupons-etapa-ocr-ia = com a IA
cupons-etapa-ocr-itens = { $n } linha(s) de item
cupons-etapa-arquivo = Ler o arquivo
cupons-etapa-texto = Ler o texto da página
cupons-etapa-conferir = Conferir itens e total
cupons-etapa-conferir-ok = { $n } item(ns), soma bate com o total
cupons-etapa-conferir-diferenca = { $n } item(ns), soma diferente do total
cupons-resultado = Resultado da leitura
cupons-resultado-ok = { $n } item(ns) de { $loja }, total { $valor }. Tudo confere.
cupons-resultado-diferenca = { $n } item(ns) de { $loja }, total { $valor }. A soma dos itens não bate com o total.
cupons-resultado-foto-ok = { $n } item(ns) lidos pela foto, total { $valor }. A soma bate.
cupons-resultado-foto-diferenca = { $n } item(ns) lidos pela foto, total { $valor }. A soma não bate: confira os itens.
cupons-resultado-afip = { $pais ->
    [PT] Fatura de Portugal
    [ES] Fatura da Espanha
   *[AR] Nota da Argentina
}: { $documento }, { $valor }.
cupons-proximo-ligar = Revise e grave: o cupom liga ao lançamento da compra que já está no app.
cupons-proximo-sem-lancamento = Revise e grave. Nenhum lançamento com esse valor perto da data: o cupom fica sem ligação.
cupons-proximo-foto = Revise os itens ao lado da foto e grave a compra.
cupons-proximo-foto-corrigir = Corrija os itens ao lado da foto, ou peça ajuda à IA.
cupons-proximo-afip = Inclua os itens pela foto ou à mão, confira o valor na sua moeda e grave.
cupons-revisar-gravar = Revisar e gravar
cupons-revisar-corrigir = Revisar e corrigir
cupons-ler-pela-foto = Ler pela foto
cupons-cadastrar-mao = Cadastrar à mão
cupons-ler-outro = Ler outro cupom
cupons-erro-link-fora = Esse link não é de uma nota que o app lê aqui. Use a foto do cupom ou cadastre a compra à mão.
cupons-erro-arquivo-fora = PDF e XML são das notas do Brasil. Use a foto do cupom ou cadastre a compra à mão.
cupons-arquivo = Arquivo do cupom
cupons-foto-motor-texto = Lido do texto colado.
