# Orçamento › Metas, Reserva de emergência e Aposentadoria: metas com progresso e projeção,
# a calculadora da reserva, o planejador de aposentadoria (com o INSS pelas regras) e os
# cenários (docs/i18n-design.md). Notas e regras das simulações vêm do núcleo.
# pt-BR é o idioma de origem.

## Tipos e situação da meta

metas-tipo-emergencia = Reserva de emergência
metas-tipo-viagem = Viagem
metas-tipo-quitacao = Quitação de dívida
metas-tipo-compra = Compra
metas-tipo-aposentadoria = Aposentadoria
metas-tipo-educacao = Educação
metas-tipo-outra = Outra

metas-situacao-atingida = Atingida
metas-situacao-no-caminho = No caminho
metas-situacao-fora-ritmo = Fora do ritmo
metas-situacao-sem-aportes = Sem aportes
metas-situacao-sem-data = Sem data-alvo

## Formulário

metas-erro-nome = Informe o nome da meta.
metas-erro-valor = Informe um valor-alvo maior que zero.
metas-erro-data-passou = A data-alvo já passou.
metas-erro-gravar = Falha ao gravar.
metas-atualizada = Meta atualizada
metas-criada = Meta criada
metas-editar-titulo = Editar meta · { $nome }
metas-nova = Nova meta
metas-criar = Criar meta
metas-nome = Nome
metas-tipo = Tipo
metas-valor-alvo = Valor-alvo
metas-data-alvo = Data-alvo
metas-data-alvo-dica = Opcional; sem data, não há aporte necessário.
metas-ja-guardado = Já guardado fora das contas
metas-aporte-planejado = Aporte mensal planejado
metas-aporte-planejado-dica = Usado na projeção quando as contas vinculadas não têm ritmo.
metas-contas-vinculadas = Contas vinculadas
metas-contas-vinculadas-dica = O saldo de hoje das contas entra no valor atual; a média de aportes líquidos dos últimos 6 meses é o ritmo.
metas-notas = Notas

## Card da meta

metas-contas = Contas
metas-nenhuma-conta = nenhuma
metas-de-alvo = de { $valor }
metas-progresso-de = Progresso de { $nome }
metas-falta = Falta
metas-aporte-necessario = Aporte necessário
metas-por-mes = /mês
metas-ritmo = Ritmo
metas-ritmo-planejado = Ritmo (planejado)
metas-ritmo-real = Ritmo (6 meses)
metas-projecao = Projeção
metas-projecao-planejada = { $data } (pelo aporte planejado)
metas-sem-ritmo = sem ritmo de aportes para projetar
metas-sem-cambio = Sem câmbio para { $moedas }: saldo fora do valor atual.
metas-editar-nome = Editar { $nome }
metas-reativar-nome = Reativar { $nome }
metas-arquivar-nome = Arquivar { $nome }
metas-excluir-nome = Excluir { $nome }
metas-reativar = Reativar
metas-arquivar = Arquivar
metas-excluir = Excluir
metas-nao-arquivou = Não foi possível arquivar
metas-excluir-titulo = Excluir meta
metas-excluir-descricao = Excluir “{ $nome }”? As contas vinculadas não mudam. Dá para desfazer pelo histórico de alterações.
metas-excluida = Meta excluída
metas-nao-excluiu = Não foi possível excluir

## Lista

metas-mostrar-arquivadas = Mostrar arquivadas
metas-vazio-kicker = Sem metas
metas-vazio-titulo = Nenhuma meta ainda
metas-vazio-descricao = Crie uma meta (reserva, viagem, quitação de dívida) e vincule as contas onde o dinheiro fica guardado: o progresso e a projeção saem dos saldos reais.

## Reserva de emergência

metas-reserva-base = Base
metas-reserva-caixa = caixa
metas-reserva-competencia = competência
metas-reserva-meses-analisados = Meses analisados
metas-reserva-meses-intervalo = { $de } a { $ate }
metas-reserva-fonte = Fonte
metas-reserva-fonte-valor = Categorias marcadas como essenciais
metas-reserva-meses-cobertura = Meses de cobertura
metas-reserva-media-de = Média de
metas-reserva-meses-na-media = Meses na média
metas-reserva-n-meses = { $n } meses
metas-reserva-meta = Meta da reserva
metas-reserva-meta-dica = { $n } × gasto essencial
metas-reserva-guardado = Guardado
metas-reserva-guardado-dica = contas correntes, poupança e dinheiro
metas-reserva-cobertura = Cobertura
metas-reserva-cobertura-valor =
    { $n ->
        [1] { $valor } mês
       *[other] { $valor } meses
    }
metas-reserva-cobertura-dica = do gasto essencial
metas-reserva-progresso = Reserva guardada em relação à meta
metas-reserva-gasto-essencial = Gasto essencial médio por mês
metas-reserva-gasto-total = Gasto total médio por mês
metas-reserva-essenciais-total = Essenciais no total
metas-reserva-guardado-meta = Guardado / meta
metas-reserva-sem-meses = Ainda não há meses completos com gasto em categorias essenciais para calcular a média.
metas-reserva-criar = Criar meta de reserva
metas-reserva-sem-meta = Sem meta, o guardado é o saldo das contas do dia a dia.
metas-reserva-atualizada = Meta de reserva atualizada
metas-reserva-atualizar = Atualizar a meta para { $valor }
metas-reserva-meta-em = A meta “{ $nome }” está em { $valor }.
metas-reserva-ja-usa = ✓ A meta “{ $nome }” já usa este valor.

## INSS pelas regras da reforma

metas-inss-sexo = Sexo
metas-inss-homem = Homem
metas-inss-mulher = Mulher
metas-inss-nascimento = Data de nascimento
metas-inss-nascimento-dica = Vazio: a idade de hoje.
metas-inss-contribuicao = Contribuição até hoje (anos)
metas-inss-contribuicao-reforma = Contribuição em 13/11/2019
metas-inss-contribuicao-reforma-dica = Vazio: estimada sem interrupções.
metas-inss-media = Média dos salários desde 07/1994
metas-inss-media-dica = Os salários de contribuição, corrigidos.
metas-inss-contribui-ate = Contribui até os
metas-inss-regras = Regras: { $sistema }
metas-inss-regra = Regra
metas-inss-cumpre-em = Cumpre em
metas-inss-beneficio-data = Benefício nessa data
metas-inss-na-idade = Na idade do plano
metas-inss-usada = usada
metas-inss-cumpre-data = { $data } ({ $idade } anos)
metas-inss-nao-cumpre = não cumpre
metas-inss-fator = fator previdenciário
metas-inss-aos = { $valor } aos { $idade }
metas-ssa-pia = Benefício na idade plena (PIA)
metas-ssa-pia-dica = O do extrato em ssa.gov/myaccount.
metas-ssa-aime = AIME (sem o PIA)
metas-ssa-aime-dica = Média mensal dos 35 melhores anos corrigidos.
metas-ssa-creditos = Créditos até hoje
metas-ssa-creditos-dica = Vazio: 40 (já tem direito).
metas-ssa-trabalha-ate = Trabalha até os
metas-pt-carreira = Anos com descontos até hoje
metas-pt-rr = Remuneração de referência
metas-pt-rr-dica = Salários da carreira ÷ (14 × anos).
metas-pt-antes-17 = Começou a descontar antes dos 17

## Planejador de aposentadoria

metas-apos-titulo = Aposentadoria e independência financeira
metas-apos-intro = Milhares de cenários de mercado em valores reais (acima da inflação): o patrimônio de hoje, os aportes até parar, o gasto mensal desejado, a aposentadoria pública ({ $sistema }) e os eventos de vida.
metas-apos-idade-hoje = Idade hoje
metas-apos-aposentadoria-aos = Aposentadoria aos
metas-apos-expectativa = Expectativa de vida
metas-apos-patrimonio-hoje = Patrimônio investido hoje
metas-apos-patrimonio-dica-valor = Vazio: a carteira ({ $valor }).
metas-apos-patrimonio-dica = Vazio: a carteira, com previdência.
metas-apos-aporte = Aporte mensal até parar
metas-apos-gasto = Gasto mensal desejado
metas-apos-inss-informar = Informar o valor
metas-apos-inss-calcular = Calcular pelas regras
metas-apos-inss-mensal = Valor mensal estimado ({ $sistema })
metas-apos-inss-a-partir = { $sistema } a partir dos
metas-apos-retorno = Retorno real (% a.a.)
metas-apos-volatilidade = Volatilidade (% a.a.)
metas-apos-eventos = Eventos de vida
metas-apos-idade = Idade
metas-apos-evento-valor = Valor (− se sai)
metas-apos-evento-o-que = O que é
metas-apos-remover-evento = Remover o evento { $n }
metas-apos-adicionar-evento = Adicionar evento (herança, venda de imóvel, faculdade…)
metas-apos-informe-idade = Informe a idade de hoje para simular.
metas-apos-simulando = Simulando os cenários
metas-apos-chance = Chance de o dinheiro durar
metas-apos-patrimonio-mediana = Patrimônio na aposentadoria (mediana)
metas-apos-gasto-90 = Gasto mensal com 90% de chance
metas-apos-necessario = Necessário ao parar (90% de chance)
metas-apos-acaba-aos = No cenário ruim, o dinheiro acaba aos
metas-apos-n-anos = { $n } anos
metas-apos-nao-acaba = não acaba
metas-apos-patrimonio-idade = Patrimônio por idade
metas-apos-ruim = Cenário ruim (10%)
metas-apos-mediana = Mediana
metas-apos-bom = Cenário bom (90%)
metas-apos-salvo = Plano de aposentadoria salvo
metas-apos-usar-principal = Usar como plano principal
metas-apos-salvar = Salvar plano

## Cenários e meta ligada ao plano

metas-cen-salvo = Cenário “{ $nome }” salvo
metas-cen-cenario = Cenário
metas-cen-principal = Plano principal
metas-cen-atualizado = Cenário atualizado
metas-cen-atualizar = Atualizar “{ $nome }”
metas-cen-apagado = Cenário apagado
metas-cen-apagar = Apagar cenário
metas-cen-novo = Novo cenário
metas-cen-exemplo = Ex.: aposentar aos 55
metas-cen-salvar = Salvar como cenário
metas-cen-comparar = Comparar cenários
metas-cen-chance = Chance
metas-cen-gasto-90 = Gasto com 90%
metas-cen-necessario = Necessário ao parar
metas-cen-previsto = Previsto ao parar
metas-cen-na-tela = Na tela agora
metas-ligada = Meta ligada: { $nome } · { $pct }% de { $valor }
metas-ligada-ate = Meta ligada: { $nome } · { $pct }% de { $valor } até { $ano }
metas-ligada-desligar = Desligar da meta
metas-ligada-explica = Uma meta de aposentadoria ligada ao plano acompanha o patrimônio necessário, a data e o aporte do plano salvo.
metas-ligada-toast = Meta ligada ao plano
metas-ligada-toast-descricao = “{ $nome }” acompanha o plano salvo em Orçamento › Metas.
metas-ligada-criar = Criar meta ligada ao plano

## Metas com rendimento e gastos anuais

metas-rendimento = Rendimento real ao ano
metas-rendimento-dica = Quanto o dinheiro guardado rende acima da inflação, como premissa sua. Vazio: a conta só divide o que falta pelos meses.
metas-erro-retorno = Use um rendimento entre −5% e 15% ao ano.
metas-rendimento-linha = Com { $taxa } ao ano acima da inflação: { $aportes } de aportes e { $rendimento } de rendimento. Sem rendimento, seriam { $linear }/mês.
metas-rendimento-alvo-epoca = Em dinheiro de hoje. Na data, com inflação de { $inflacao } ao ano, o alvo vale cerca de { $valor }.
metas-rendimento-premissa = O rendimento é uma premissa, não uma promessa.
metas-reserva-anuais = Incluir gastos anuais
metas-reserva-anuais-dica = IPVA, IPTU, seguros e material escolar entram divididos por 12, no lugar dos meses de pico.
metas-reserva-custo-mensal = Custo de vida mensal
metas-reserva-essenciais-regulares = Essenciais do mês, sem os anuais
metas-reserva-provisao-anual = Provisão dos gastos anuais essenciais
metas-reserva-anuais-titulo = Gastos anuais nos últimos 12 meses
metas-reserva-anuais-tabela = Gastos anuais
metas-reserva-anuais-mes = { $valor }/mês
metas-reserva-anuais-nao-essencial = fora da reserva
metas-reserva-anuais-sem-ano = Os gastos anuais aparecem quando houver 12 meses completos de lançamentos.
metas-reserva-anuais-nenhum = Nenhum gasto anual nos últimos 12 meses (categorias com gasto em até 3 meses do ano).
metas-reserva-meta-dica-custo = { $n } × custo de vida mensal

## Fontes da meta (docs/goal-sources.md)

metas-fontes = Fontes
metas-fontes-nome = Fontes de { $nome }
metas-fontes-titulo = Fontes de { $nome }
metas-fontes-dica = O valor atual é a soma das fontes. Conta entra pelo saldo, nunca negativo (conta no vermelho é dívida, não reserva); investimento entra pelo valor líquido de resgate quando há, senão pelo de mercado; uma etiqueta junta todas as posições com ela, também as que ganharem a etiqueta depois.
metas-fontes-vazio = Nenhuma fonte ainda: ligue uma conta, uma etiqueta ou as posições abaixo.
metas-fontes-desatualizadas = Valor pode estar desatualizado: { $fontes }.
metas-fonte-tipo-inicial = Guardado fora
metas-fonte-tipo-carteira = Carteira
metas-fonte-tipo-conta = Conta
metas-fonte-tipo-conta-inteira = Conta inteira
metas-fonte-tipo-ativo = Investimento
metas-fonte-tipo-etiqueta = Etiqueta
metas-fonte-tipo-lancamentos = Lançamentos
metas-fonte-valor-saldo = saldo
metas-fonte-valor-mercado = valor de mercado
metas-fonte-valor-liquido = líquido de resgate
metas-fonte-valor-custo = custo (sem cotação)
metas-fonte-valor-informado = valor informado
metas-fonte-remover = Desligar { $nome }
metas-fonte-erro = Não foi possível mudar as fontes
metas-fonte-negativo-fora = O saldo negativo ficou de fora: é dívida, não reserva.
metas-fonte-desatualizada = Pode estar desatualizado (conta sem extrato recente ou ativo sem cotação).
metas-fonte-ligadas = { $n ->
    [one] 1 investimento ligado à meta
   *[other] { $n } investimentos ligados à meta
}
metas-fonte-liquidez = Usar meus investimentos de liquidez diária
metas-fonte-liquidez-dica = Renda fixa com liquidez diária, Tesouro Selic e fundos DI ou referenciados com posição aberta que ainda não estão nesta meta.
metas-fonte-liquidez-buscando = Procurando os investimentos…
metas-fonte-liquidez-vazio = Nenhum investimento de liquidez diária fora desta meta.
metas-fonte-adicionar = Adicionar
metas-fonte-adicionar-todos = Adicionar todos ({ $n })
metas-fonte-conta = Conta
metas-fonte-escolha-conta = Escolha a conta
metas-fonte-etiqueta = Etiqueta de ativos
metas-fonte-etiqueta-dica = Todas as posições com a etiqueta entram, também as que ganharem a etiqueta depois.
metas-fonte-escolha-etiqueta = Escolha a etiqueta
metas-fonte-sem-etiquetas = Nenhuma etiqueta ainda
