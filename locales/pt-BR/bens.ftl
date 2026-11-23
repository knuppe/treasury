# Carteira › Bens: patrimônio físico (imóveis, veículos, terrenos, equipamentos e coleções),
# com custo, avaliação, benfeitorias, venda e financiamento ligado (docs/i18n-design.md). A
# fórmula do valor e os erros vêm do núcleo; IPCA, IGP-M, INCC, ITBI, FIPE e LTV ficam no
# original. pt-BR é o idioma de origem.

## Tipos, métodos, índices e fontes

bens-tipo-imovel = Imóvel
bens-tipo-veiculo = Veículo
bens-tipo-terreno = Terreno
bens-tipo-equipamento = Equipamento
bens-tipo-colecao = Arte e coleção
bens-tipo-outro = Outro

bens-metodo-nenhum = Sem estimativa
bens-metodo-indice = Correção por índice
bens-metodo-declinante = Depreciação anual
bens-metodo-linear = Depreciação linear
bens-metodo-fipe = Tabela FIPE
bens-metodo-nenhum-dica = Vale a última avaliação registrada; sem avaliação, o custo.
bens-metodo-indice-dica = Custo (ou a última avaliação) corrigido pelo índice escolhido. É uma estimativa: o mercado de imóveis não segue o índice à risca.
bens-metodo-declinante-dica = Perde uma porcentagem fixa por ano sobre o valor anterior. Carros perdem ~15–20% no 1º ano e ~8–10% depois; 12% a.a. é a média dos primeiros cinco anos.
bens-metodo-linear-dica = Perde o mesmo valor todo ano até o residual no fim da vida útil (eletrônicos e máquinas: 5 a 10 anos).
bens-metodo-fipe-dica = Preço médio da tabela FIPE na última consulta; entre consultas, depreciação anual.

bens-indice-ipca-dica = Inflação oficial: mantém o poder de compra do que foi pago.
bens-indice-igpm-dica = Índice dos aluguéis: mais volátil, segue preços no atacado.
bens-indice-incc-dica = Custo da construção: aproxima o custo de reconstruir o imóvel.

bens-categoria-reforma = Reforma
bens-categoria-ampliacao = Ampliação
bens-categoria-benfeitoria = Benfeitoria
bens-categoria-manutencao = Manutenção

bens-custo-escritura = Escritura
bens-custo-registro = Registro
bens-custo-corretagem = Corretagem
bens-custo-outro = Outro custo

bens-fonte-manual = Estimativa própria
bens-fonte-avaliacao = Avaliação
bens-fonte-laudo = Laudo
bens-fonte-indice = Por índice

bens-base-avaliacao = avaliação
bens-base-automatica = estimativa automática
bens-base-custo = custo
bens-base-vendido = vendido
bens-base-nao-comprado = ainda não comprado

bens-fipe-carro = Carro
bens-fipe-moto = Moto
bens-fipe-caminhao = Caminhão

## Como o valor é estimado (linha curta) e donos

bens-metodo-texto-indice = corrigido pelo { $indice }
bens-metodo-texto-declinante = depreciação de { $taxa } a.a.
bens-metodo-texto-linear = linear em { $anos } anos
bens-metodo-texto-linear-residual = linear em { $anos } anos até { $residual }
bens-metodo-texto-fipe = tabela FIPE + { $taxa } a.a.
bens-metodo-texto-fipe-codigo = tabela FIPE { $codigo } + { $taxa } a.a.
bens-metodo-texto-nenhum = sem estimativa automática
bens-donos-par = { $a } e { $b }
bens-familia = Família

## Rótulos repetidos

bens-titulo = Bens
bens-bem = Bem
bens-o-bem = bem
bens-tipo = Tipo
bens-valor = Valor
bens-custo = Custo
bens-ganho = Ganho
bens-patrimonio = Patrimônio
bens-avaliacao = Avaliação
bens-avaliacoes = Avaliações
bens-saldo-devedor = Saldo devedor
bens-data = Data
bens-descricao = Descrição
bens-fonte = Fonte
bens-projeto = Projeto
bens-observacoes = Observações
bens-opcional = Opcional.
bens-nenhum = Nenhum
bens-mes = Mês
bens-ao-ano-abrev = a.a.
bens-vendido = vendido
bens-arquivado = arquivado
bens-acoes-de = Ações de { $nome }
bens-editar-nome = Editar { $nome }
bens-excluir-nome = Excluir { $nome }
bens-excluir = Excluir
bens-reativar = Reativar
bens-arquivar = Arquivar
bens-reativado = Bem reativado
bens-arquivado-toast = Bem arquivado
bens-excluido = Bem excluído
bens-excluir-titulo = Excluir { $nome }
bens-excluir-bem = Excluir bem
bens-novo = Novo bem
bens-nao-excluiu = Não foi possível excluir

## Figuras

bens-valor-estimado = Valor (≈ estimado)
bens-custo-acumulado = Custo acumulado
bens-avaliacao-em = { $fonte } em { $data }
bens-valor-do-bem = Valor do bem
bens-patrimonio-do-bem = Patrimônio do bem
bens-fig-valor-titulo = Valor × custo ao longo do tempo
bens-fig-valor-fonte = Valor: última avaliação ou estimativa automática · custo: compra + custos + benfeitorias de capital
bens-fig-valor-fonte-estimado = Valor: última avaliação ou estimativa automática (≈ estimado) · custo: compra + custos + benfeitorias de capital
bens-fig-valor-aria = Valor de { $nome } ao longo do tempo, com o custo acumulado e { $n } avaliações.
bens-fig-valor-tabela = Valor mês a mês · { $nome }
bens-fig-patrimonio-titulo = Patrimônio do bem × saldo devedor
bens-fig-patrimonio-fonte = Saldo devedor pelo cronograma dos contratos ligados · a soma das faixas é o valor do bem
bens-fig-patrimonio-aria = Patrimônio de { $nome } e saldo devedor do financiamento, empilhados mês a mês.
bens-fig-patrimonio-tabela = Patrimônio do bem mês a mês · { $nome }

## Lista e resumo

bens-resumo-valor = Valor dos bens
bens-n-bens =
    { $n ->
        [1] { $n } bem
       *[other] { $n } bens
    }
bens-resumo-ganho = Ganho sobre o custo
bens-custo-de = custo de { $valor }
bens-resumo-patrimonio = Patrimônio dos bens
bens-valor-menos-saldo-financiamentos = Valor − saldo devedor dos financiamentos
bens-saldo-devedor-valor = saldo devedor { $valor }
bens-sem-financiamentos = sem financiamentos ligados
bens-cmd-grupo = Patrimônio
bens-cmd-benfeitoria = Registrar benfeitoria em { $nome }
bens-cmd-benfeitoria-busca = obra,reforma,ampliação,benfeitoria,manutenção
bens-cmd-avaliacao = Registrar avaliação de { $nome }
bens-cmd-avaliacao-busca = laudo,avaliação,valor de mercado,fipe
bens-abrir = Abrir bem
bens-registrar-benfeitoria-menu = Registrar benfeitoria…
bens-registrar-avaliacao-menu = Registrar avaliação…
bens-novo-projeto-menu = Novo projeto do bem…
bens-ligar-financiamento-menu = Ligar financiamento…
bens-registrar-venda-menu = Registrar venda…
bens-editar-cadastro-menu = Editar cadastro…
bens-excluir-menu = Excluir…
bens-nao-foi-possivel = Não foi possível
bens-avaliando = Avaliando os bens
bens-vazio-kicker = Sem bens
bens-vazio-titulo = Cadastre o primeiro imóvel, veículo ou terreno
bens-vazio-descricao = Informe a data e o preço de compra com os custos (ITBI, escritura, registro): o valor atual é estimado pelo índice, pela depreciação ou pela tabela FIPE, e as benfeitorias entram no custo. Ligue o financiamento de 07 · Dívidas para ver o patrimônio do bem.
bens-mostrar-arquivados = Mostrar arquivados e vendidos
bens-excluir-descricao-lista = Apaga o bem com custos, benfeitorias e avaliações. Os financiamentos e os lançamentos continuam. Dá para desfazer pelo histórico.

## Detalhe do bem

bens-bem-nome = Bem { $nome }
bens-comprado-em = comprado em { $data }
bens-vendido-em = vendido em { $data }
bens-fora-patrimonio = fora do patrimônio líquido
bens-registrar-avaliacao = Registrar avaliação
bens-fipe-consultada = Tabela FIPE consultada
bens-fipe-valor-atualizado = { $nome }: valor atualizado
bens-fipe-falhou = Não foi possível consultar a FIPE
bens-consultar-fipe = Consultar FIPE
bens-vender = Vender
bens-benfeitoria = Benfeitoria
bens-valor-atual = Valor atual
bens-vendido-maiusc = Vendido
bens-por-base = Por { $base }
bens-liquido-venda = líquido da venda
bens-compra = Compra
bens-custos-aquisicao = Custos de aquisição
bens-benfeitorias = Benfeitorias
bens-custo-total = Custo total
bens-manutencao-fora = Manutenção (fora do custo)
bens-valorizacao = Valorização
bens-depreciacao = Depreciação
bens-ganho-realizado = Ganho realizado
bens-ganho-nominal = Ganho nominal
bens-sobre-custo = Sobre o custo
bens-ganho-real = Ganho real (− IPCA)
bens-ao-ano = Ao ano
bens-ao-ano-real = Ao ano, real
bens-valor-menos-saldo = Valor − saldo devedor
bens-saldo-ltv = saldo devedor { $saldo } · LTV { $ltv }
bens-sem-financiamento = sem financiamento ligado
bens-valor-estimado-titulo = ≈ Valor estimado
bens-valor-medido = Valor medido
bens-calculando-historico = Calculando o histórico do bem
bens-financiamento = Financiamento
bens-ligar-financiamento = Ligar financiamento
bens-financiamentos-de = Financiamentos de { $nome }
bens-financiamentos-vazio = Nenhum contrato ligado. Ligue o financiamento cadastrado em 07 · Dívidas para ver o patrimônio do bem e o LTV.
bens-contrato = Contrato
bens-financiado = Financiado
bens-desligar-nome = Desligar { $nome }
bens-financiamento-desligado = Financiamento desligado
bens-nao-desligou = Não foi possível desligar
bens-desligar = Desligar
bens-benfeitorias-titulo = Benfeitorias, reformas e obras
bens-entram-custo = Entram no custo
bens-entram-custo-valor = reforma, ampliação, benfeitoria
bens-fora-custo-rotulo = Fora do custo
bens-fora-custo-valor = manutenção
bens-benfeitorias-de = Benfeitorias de { $nome }
bens-benfeitorias-vazio = Nenhuma obra registrada.
bens-fora-custo = fora do custo
bens-sem-lancamentos-estimado = sem lançamentos: vale o estimado
bens-n-lancamentos =
    { $n ->
        [one] { $n } lançamento
       *[other] { $n } lançamentos
    }
bens-benfeitoria-excluida = Benfeitoria excluída
bens-avaliacoes-de = Avaliações de { $nome }
bens-avaliacoes-vazio = Nenhuma avaliação: o valor é estimado automaticamente a partir do custo.
bens-excluir-avaliacao-de = Excluir avaliação de { $data }
bens-avaliacao-excluida = Avaliação excluída
bens-custos-aquisicao-prefixo = Custos de aquisição:
bens-observacoes-texto = Observações: { $notas }
bens-excluir-descricao-detalhe = O bem, as benfeitorias e as avaliações saem; o financiamento continua em 07 · Dívidas. Dá para desfazer pelo histórico de alterações.

## Cadastro do bem

bens-editar-bem = Editar bem
bens-form-descricao = Imóvel, veículo, terreno, equipamento ou coleção: compra com os custos de aquisição, o método de avaliação e, se houver, o financiamento.
bens-erro-nome = Dê um nome ao bem.
bens-erro-data-compra = Informe a data da compra.
bens-erro-preco-compra = Informe o preço de compra.
bens-erro-taxa = Taxa entre 0 e 100% a.a.
bens-erro-vida-util = Vida útil entre 1 e 100 anos.
bens-erro-fipe = Escolha marca, modelo e ano na tabela FIPE.
bens-erro-valor-invalido = Valor inválido.
bens-alterado = Bem alterado
bens-cadastrado = Bem cadastrado
bens-nome-bem = Nome do bem
bens-nome-exemplo = Apartamento Vila Mariana
bens-data-compra = Data da compra
bens-preco-compra = Preço de compra
bens-preco-compra-dica = Valor do bem, sem os custos.
bens-custos-aquisicao-dica = ITBI, escritura, registro e corretagem entram no custo do bem (e no ganho de capital).
bens-custo-n = Custo { $n }
bens-remover-custo = Remover custo { $n }
bens-remover = Remover
bens-adicionar-custo = Adicionar custo
bens-estimado-por = Valor atual estimado por
bens-indice-correcao = Índice de correção
bens-depreciacao-aa = Depreciação (% a.a.)
bens-depreciacao-fipe-dica = Usada entre as consultas à tabela FIPE.
bens-depreciacao-dica = Padrão de 12% a.a. para carros.
bens-vida-util = Vida útil (anos)
bens-residual = Valor residual (% do custo)
bens-residual-dica = Quanto sobra no fim da vida útil. Opcional.
bens-endereco = Endereço ou identificação
bens-endereco-dica = Opcional (matrícula, placa, chassi…).
bens-financiamento-dica = Contrato de 07 · Dívidas que financiou o bem. Opcional.
bens-de-membro = De { $nome }
bens-entra-patrimonio = Entra no patrimônio líquido
bens-salvar-alteracoes = Salvar alterações
bens-cadastrar = Cadastrar bem

## Tabela FIPE

bens-fipe-explica = Com marca, modelo e ano, o app consulta o preço médio na tabela FIPE (parallelum.com.br, gratuito) quando você pedir.
bens-fipe-buscar = Buscar na tabela FIPE
bens-fipe-tipo-veiculo = Tipo de veículo
bens-fipe-marca = Marca
bens-fipe-escolha-marca = Escolha a marca
bens-fipe-modelo = Modelo
bens-fipe-escolha-modelo = Escolha o modelo
bens-fipe-ano = Ano e combustível
bens-fipe-escolha-ano = Escolha o ano
bens-fipe-codigo = Código FIPE
bens-fipe-codigo-dica = Opcional: a consulta confere o código.

## Benfeitoria, avaliação, venda e financiamento

bens-editar-benfeitoria = Editar benfeitoria
bens-registrar-benfeitoria-obra = Registrar benfeitoria ou obra
bens-benfeitoria-descricao = { $nome }: reformas, ampliações e benfeitorias entram no custo; manutenção é despesa e fica fora.
bens-erro-data = Informe a data.
bens-erro-descreva = Descreva a obra.
bens-erro-valor-projeto = Informe o valor (ou ligue a um projeto).
bens-benfeitoria-alterada = Benfeitoria alterada
bens-benfeitoria-registrada = Benfeitoria registrada
bens-tipo-obra = Tipo de obra
bens-benfeitoria-exemplo = Reforma do banheiro
bens-valor-estimado-campo = Valor estimado
bens-valor-projeto-dica = Com lançamentos no projeto, vale o que ele gastou.
bens-fica-fora-custo = Fica fora do custo do bem.
bens-soma-custo = Soma no custo do bem.
bens-projeto-gastos = Projeto dos gastos
bens-projeto-gastos-dica = Os lançamentos do projeto viram o valor da obra, cada um na sua data.
bens-novo-projeto-benfeitoria = Novo projeto com o nome da benfeitoria
bens-sera-criado = Será criado “{ $nome }” deste bem; depois ponha os lançamentos nele.
bens-a-benfeitoria = a benfeitoria
bens-registrar = Registrar
bens-avaliacao-descricao = { $nome }: a avaliação mais recente vira o valor do bem; depois dela, a estimativa automática parte dela.
bens-erro-valor = Informe o valor.
bens-avaliacao-registrada = Avaliação registrada
bens-avaliacao-obs-dica = Quem avaliou, comparáveis… Opcional.
bens-vender-nome = Vender { $nome }
bens-venda-descricao = Registra a venda: o bem sai do patrimônio a partir da data e o ganho fica realizado. O dinheiro da venda entra pela conta, como um lançamento.
bens-erro-venda-antes = A venda não pode ser antes da compra.
bens-erro-preco = Informe o preço.
bens-venda-registrada = Venda registrada
bens-venda = Venda
bens-data-venda = Data da venda
bens-preco-venda = Preço de venda
bens-custos-venda = Custos da venda
bens-custos-venda-dica = Corretagem, impostos, taxas.
bens-arquivar-bem = Arquivar o bem
bens-registrar-venda = Registrar venda
bens-ligar-descricao = O saldo devedor do contrato entra no patrimônio de { $nome } (valor − saldo) e no LTV. A dívida continua contando uma vez só, em 07 · Dívidas.
bens-financiamento-ligado = Financiamento ligado
bens-nenhum-contrato-livre = Nenhum contrato livre. Cadastre o financiamento em 07 · Dívidas primeiro.
bens-escolha-contrato = Escolha o contrato
bens-ligar-contrato = Ligar contrato
bens-gcap-titulo-vendido = Imposto sobre o ganho da venda
bens-gcap-titulo-simular = Simular o imposto de uma venda
bens-gcap-data = Data da venda
bens-gcap-preco = Preço de venda
bens-gcap-custos = Corretagem paga na venda
bens-gcap-unico = Único imóvel, sem outra venda de imóvel nos últimos 5 anos
bens-gcap-unico-dica = Isento se o preço de venda for de até R$ 440 mil.
bens-gcap-reaplicado = Valor aplicado em outro imóvel residencial em até 180 dias
bens-gcap-ganho-bruto = Ganho bruto (venda − corretagem − custo)
bens-gcap-lei-7713 = Redução da Lei 7.713 ({ $pct }%)
bens-gcap-fr1 = Fator FR1 ({ $pct }%)
bens-gcap-fr2 = Fator FR2 ({ $pct }%)
bens-gcap-isencao-unico = Isenção do único imóvel
bens-gcap-isencao-reaplicacao = Isenção pela compra de outro imóvel residencial
bens-gcap-tributavel = Ganho tributável
bens-gcap-imposto = Imposto
bens-gcap-pequeno-valor = Isento: vendas de até R$ 35 mil no mês
bens-gcap-darf = DARF
bens-gcap-darf-valor = código { $codigo }, vence em { $vencimento }
bens-gcap-nota = Como o programa GCAP da Receita: custo com os custos da compra e as benfeitorias; imposto de 15% a 22,5% por faixa (Lei 8.981/1995 art. 21). Confira no GCAP antes de pagar.
