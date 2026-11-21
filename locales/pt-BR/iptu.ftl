# IPTU dos imóveis e terrenos (Brasil): o carnê de cada ano em Bens, os lembretes de vencimento
# e o IPTU nos impostos pagos (docs/brazil/iptu.md). pt-BR é o idioma de origem.

iptu-titulo = IPTU
iptu-descricao = O carnê de cada ano: valor venal, impostos, taxas, descontos e vencimentos. O Treasury lembra os vencimentos e soma o IPTU nos impostos pagos.
iptu-registrar = Registrar IPTU de { $ano }
iptu-registrar-outro = Registrar outro ano
iptu-vazio = Nenhum carnê registrado. Registre o IPTU deste ano para lembrar os vencimentos e acompanhar o valor venal.
iptu-editar = Editar
iptu-apagar = Apagar
iptu-apagar-titulo = Apagar o IPTU de { $ano }
iptu-apagar-texto = O carnê e os pagamentos marcados saem. Dá para desfazer pelo histórico.
iptu-editar-titulo = IPTU de { $ano } · { $imovel }
iptu-novo-titulo = Registrar IPTU · { $imovel }

## Campos do carnê

iptu-ano = Ano (exercício)
iptu-cadastro = Número do cadastro / IQ
iptu-cadastro-ajuda = Inscrição imobiliária, indicação fiscal, SQL ou índice cadastral: o número do imóvel na prefeitura, impresso no carnê.
iptu-cidade = Município
iptu-area-terreno = Área do terreno (m²)
iptu-area-construida = Área construída (m²)
iptu-secao-venal = Valor venal
iptu-valor-venal-terreno = Valor venal do terreno
iptu-valor-venal-predial = Valor venal predial
iptu-valor-venal-total = Valor venal total
iptu-secao-impostos = Impostos, taxas e descontos
iptu-imposto-territorial = Imposto territorial
iptu-imposto-predial = Imposto predial
iptu-total-impostos = Total de impostos
iptu-taxa-iluminacao = Taxa de iluminação pública
iptu-taxa-lixo = Taxa de lixo (coleta)
iptu-outras-taxas = Outras taxas
iptu-desconto = Desconto
iptu-desconto-bom-pagador = Desconto bom pagador
iptu-total-devido = Total devido
iptu-total-calcular = Calcular pelo carnê
iptu-secao-pagamento = Pagamento
iptu-forma = Forma de pagamento
iptu-forma-indefinida = Ainda não decidi
iptu-forma-unica = Cota única
iptu-forma-parcelado = Parcelado
iptu-cota-unica-vencimento = Vencimento da cota única
iptu-cota-unica-valor = Valor da cota única
iptu-parcelas-quantidade = Número de parcelas
iptu-parcelas-primeiro = Primeiro vencimento
iptu-parcelas-valor = Valor de cada parcela
iptu-parcelas-ajuda = As parcelas vencem uma por mês, a partir do primeiro vencimento.
iptu-notas = Observações
iptu-salvar = Salvar
iptu-cancelar = Cancelar
iptu-salvo = IPTU de { $ano } salvo
iptu-erro-sem-total = Informe o total devido.

## Parcelas e pagamento

iptu-parcelas-titulo = Parcelas do IPTU de { $ano }

iptu-cota-unica = Cota única
iptu-parcela-n = Parcela { $n }
iptu-coluna-parcela = Parcela
iptu-coluna-vencimento = Vencimento
iptu-coluna-valor = Valor
iptu-coluna-situacao = Situação
iptu-situacao-paga = Paga em { $data }
iptu-situacao-aberta = Em aberto
iptu-situacao-vencida = Vencida
iptu-situacao-outra-forma = Outra forma
iptu-marcar-pago = Marcar como pago
iptu-desfazer-pagamento = Desfazer pagamento
iptu-pagamento-marcado = Pagamento marcado
iptu-pagamento-desfeito = Pagamento desfeito
iptu-pagar-titulo = Marcar { $parcela } como paga
iptu-lancamentos-provaveis = Lançamento do pagamento
iptu-nenhum-lancamento = Nenhum lançamento parecido perto do vencimento.
iptu-pago-sem-lancamento = Pago sem lançamento no app, em
iptu-sugestao = Parece pago: { $descricao } em { $data }
iptu-confirmar-sugestao = Confirmar

## Números derivados

iptu-aliquota-efetiva = Alíquota efetiva
iptu-pago = Pago
iptu-em-aberto = Em aberto
iptu-variacao-venal = Valor venal { $variacao } contra { $ano }
iptu-variacao-imposto = Impostos { $variacao } contra { $ano }
iptu-ipca-ano = IPCA de { $ano }: { $ipca }
iptu-mercado = Valor venal = { $proporcao } do valor de mercado ({ $valor } em { $data })
iptu-cota-x-parcelas = Cota única de { $unica } × { $n } parcelas somando { $parcelas }
iptu-historico-titulo = Histórico do valor venal
iptu-coluna-ano = Ano

## Percepções (núcleo)

iptu-percepcao-a-vista-cdi = Pagar a cota única economiza { $economia }. O desconto equivale a { $taxa }% ao mês, mais que o CDI ({ $cdi }% ao mês): vale pagar à vista se o dinheiro já está guardado.
iptu-percepcao-a-vista = Pagar a cota única economiza { $economia }, um desconto equivalente a { $taxa }% ao mês.
iptu-percepcao-parcelado = A cota única economiza { $economia }, mas o desconto ({ $taxa }% ao mês) rende menos que o CDI ({ $cdi }% ao mês): parcelar e deixar o dinheiro aplicado sai um pouco melhor.
iptu-percepcao-acima-ipca = O IPTU subiu { $variacao }% contra o ano anterior, acima do IPCA de { $ano } ({ $ipca }%). Confira no carnê se mudou a área, o uso ou a planta de valores; a prefeitura aceita pedido de revisão no prazo do carnê.
iptu-percepcao-venal-acima = O valor venal está { $diferenca }% acima do valor de mercado do imóvel no app. Se a avaliação estiver certa, dá para pedir à prefeitura a revisão do valor venal (impugnação do lançamento, no prazo do carnê).
iptu-percepcao-venal-abaixo = O valor venal é { $proporcao }% do valor de mercado do imóvel. É comum ficar abaixo do mercado; não use o valor venal como estimativa do preço do imóvel.
iptu-percepcao-total-diferente = Impostos + taxas − descontos dá { $calculado }, mas o total devido é { $total }. Confira os valores do carnê.

## Erros (núcleo)

iptu-imovel = imóvel
iptu-carne = carnê de IPTU
iptu-parcela = parcela do IPTU
iptu-erro-tipo = IPTU só vale para imóveis e terrenos.
iptu-erro-ano = Ano do IPTU inválido.
iptu-erro-negativo = Os valores do IPTU não podem ser negativos.
iptu-erro-area = Área inválida: use metros quadrados, como 250 ou 72,5.
iptu-erro-parcela = O carnê tem no máximo 24 parcelas.
iptu-erro-parcela-valor = Cada parcela precisa de um valor maior que zero.
iptu-erro-parcela-repetida = Há duas parcelas com o mesmo número.
iptu-erro-lancamento-usado = Esse lançamento já paga outra parcela do IPTU.
iptu-erro-lancamento-saida = O pagamento precisa ser uma saída (valor negativo).
iptu-erro-pagamento = Escolha o lançamento do pagamento ou informe a data.

## Histórico de alterações (núcleo)

iptu-historico-salvo = IPTU de { $ano } registrado
iptu-historico-apagado = IPTU apagado
iptu-historico-pago = Parcela do IPTU paga
iptu-historico-desfeito = Pagamento do IPTU desfeito

## Lembretes (núcleo)

iptu-quando-hoje = hoje
iptu-quando-amanha = amanhã
iptu-quando-dias = em { $dias } dias
iptu-alerta-titulo = IPTU { $ano } · { $imovel }: { $parcela } vence { $quando }
iptu-alerta-titulo-vencido = IPTU { $ano } · { $imovel }: { $parcela } venceu
iptu-alerta-corpo = { $valor } com vencimento em { $data }. Quando pagar, marque em Carteira › Bens › { $imovel } › IPTU.
iptu-alerta-corpo-decidir = A cota única de { $valor } vence em { $data }. Veja em Carteira › Bens › { $imovel } › IPTU se vale pagar à vista ou parcelado.
iptu-alerta-falta-titulo = IPTU { $ano } · { $imovel }: registre o carnê
iptu-alerta-falta-corpo = O carnê de { $ano } costuma chegar nesta época. Registre os valores em Carteira › Bens › { $imovel } › IPTU para lembrar os vencimentos e acompanhar o valor venal.

## Impostos pagos (núcleo)

iptu-impostos-nota = IPTU pelos carnês registrados em Bens (o que vence no período). Lance o pagamento numa categoria de IPTU para usar o valor pago.
iptu-impostos-linha = IPTU pelo carnê · { $imovel }
iptu-impostos-valor = Valor venal
