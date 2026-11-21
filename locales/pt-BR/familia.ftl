# Tela 02 · Família: os membros, o cadastro de cada um, as rendas, os holerites importados, as
# contas e cartões de cada membro e a previsão da renda da família (docs/i18n-design.md). Os nomes
# da folha de outros países ainda vêm do núcleo. pt-BR é o idioma de origem.

familia-nome-instituicoes = Nome da família e instituições

## Como o Brasil chama as coisas da renda (os outros países vêm do núcleo)

familia-tipo-clt = CLT
familia-tipo-servidor = Servidor público
familia-tipo-aposentadoria = Aposentadoria
familia-tipo-pensao = Pensão
familia-tipo-pj = PJ
familia-tipo-autonomo = Autônomo
familia-tipo-informal = Informal
familia-tipo-aluguel = Aluguel
familia-tipo-beneficio = Benefício social
familia-tipo-outra = Outra
familia-holerite = holerite
familia-holerites = holerites
familia-descontos = { $contribuicao } e { $imposto }
familia-descontos-eua = FICA e imposto federal
familia-13-salario = 13º salário
familia-subsidio-natal = subsídio de Natal
familia-terco-ferias = terço de férias
familia-subsidio-ferias = subsídio de férias
familia-dependente-dica-br = Entra como dependente na declaração de quem declara (dedução no IR retido).
familia-dependente-dica = Conta como dependente no imposto de quem declara (abate no imposto retido).

## Cadastro do membro

familia-relacao-titular = Responsável
familia-relacao-conjuge = Cônjuge
familia-relacao-filho = Filho(a)
familia-relacao-pai-mae = Pai ou mãe
familia-relacao-outro = Outro
familia-cadastro-de = Cadastro de { $nome }
familia-relacao = Relação na família
familia-escolha = Escolha
familia-nascimento = Data de nascimento
familia-sexo = Sexo
familia-sexo-dica = Opcional. Usado na linha da vida e como padrão no cálculo do INSS.
familia-sexo-nao-informado = Não informado
familia-sexo-homem = Homem
familia-sexo-mulher = Mulher
familia-sexo-nao-informar = Prefiro não informar
familia-ocupacao = Profissão ou ocupação
familia-observacoes = Observações
familia-dependente-ir = Dependente no imposto de renda
familia-sem-cpf = O app não guarda CPF nem documentos.
familia-cadastro-salvo = Cadastro salvo

## Alergias e restrições alimentares (docs/gtin-inflation.md §7.2)

familia-dieta-titulo = Alergias e restrições alimentares
familia-dieta-dica = Opcional. Fica só no seu cofre; o Treasury avisa em 06 · Consumo › Saúde quando entra no cupom algo com isso (segundo o Open Food Facts).
familia-dieta-gluten = Glúten
familia-dieta-milk = Leite (lactose)
familia-dieta-eggs = Ovos
familia-dieta-peanuts = Amendoim
familia-dieta-nuts = Castanhas
familia-dieta-soybeans = Soja
familia-dieta-fish = Peixe
familia-dieta-crustaceans = Crustáceos
familia-dieta-sesame = Gergelim
familia-dieta-vegan = Vegano
familia-dieta-vegetarian = Vegetariano

## Renda

familia-renda-alterada = Renda alterada
familia-renda-cadastrada = Renda cadastrada
familia-editar-nome = Editar { $nome }
familia-nova-renda = Nova renda
familia-renda-descricao = Bruto e líquido de referência do mês. Com { $holerite } importado, a previsão usa os valores dele.
familia-nome = Nome
familia-nome-exemplo = Salário, aposentadoria, bicos…
familia-tipo = Tipo
familia-bruto-mes = Bruto por mês
familia-bruto-dica = Antes dos descontos ({ $descontos }).
familia-opcional = Opcional.
familia-liquido-mes = Líquido por mês
familia-liquido-dica = O que cai na conta.
familia-empregador-fonte = Empregador ou fonte pagadora
familia-registro-empresa = Registro da empresa
familia-registro-dica = Liga os { $holerites } a esta renda sozinho.
familia-dia-pagamento = Dia do pagamento
familia-mes-ferias = Mês das férias
familia-mes-ferias-dica = O { $ferias } entra na previsão.
familia-sem-ferias = Sem férias marcadas
familia-comeca-em = Começa em
familia-termina-em = Termina em
familia-termina-dica = Contrato temporário, benefício com prazo.
familia-recebe = Recebe { $extra }
familia-valor-variavel = Valor muda de mês a mês
familia-valor-variavel-dica = Comissão, bicos, vendas: a previsão usa a média dos últimos 6 meses dos lançamentos da categoria da renda, quando houver.

## Holerite

familia-importar-holerite = Importar { $holerite }
familia-importar-descricao = O PDF do { $holerite }: você confere antes de gravar. Nome e documentos pessoais não são guardados.
familia-holerite-pdf = { $holerite } em PDF
familia-senha-se-tiver = Senha (se tiver)
familia-ler-de-novo = Ler de novo
familia-lendo = Lendo o { $holerite }…
familia-competencia = Competência
familia-empregador = Empregador
familia-renda = Renda
familia-renda-nova = nova ({ $tipo })
familia-bruto = Bruto
familia-outros-descontos = Outros descontos
familia-liquido = Líquido
familia-salario-conta = Salário na conta
familia-nao-achado = não achado
familia-competencia-mes = Competência (AAAA-MM)
familia-sem-mes = O { $holerite } não trouxe o mês.
familia-ja-importado = Este { $holerite } já foi importado: gravar substitui o anterior.
familia-holerite-importado = { $holerite } importado
familia-gravar = Gravar { $holerite }

## Contas e cartões de cada membro

familia-contas-de = Contas e cartões de { $nome }
familia-contas-descricao = Os lançamentos de uma conta ou cartão de um membro só já ficam com ele, inclusive os antigos (menos os que você mudou à mão). Conta conjunta, com dois ou mais membros, fica da família.
familia-conjunta-com = conjunta com { $nomes }
familia-de-outros = de { $nomes }
familia-pronto = Pronto

## Começo guiado pelo questionário

familia-inicio-fixa-calcula = No questionário, você disse que a renda principal é fixa. Cadastre o salário com o bruto e o líquido, ou importe um { $holerite }: o app calcula { $itens }.
familia-inicio-fixa-recibo = No questionário, você disse que a renda principal é fixa. Cadastre o salário com o bruto e o líquido, ou importe um { $holerite } (os descontos vêm do recibo; o app não calcula os impostos deste país).
familia-dica-fixa = Boa prática: separe a poupança no dia do pagamento, antes dos gastos (pague-se primeiro).
familia-inicio-variavel = No questionário, você disse que a renda é variável. Cadastre como renda variável: a previsão usa a média dos últimos 6 meses.
familia-dica-variavel = Boa prática: planeje o mês pelo valor dos meses mais fracos e mantenha uma reserva de 6 a 12 meses de gastos.
familia-inicio-beneficios = No questionário, você disse que a renda vem de benefícios. Cadastre cada benefício ({ $exemplos }) com o valor do mês.
familia-beneficios-br = aposentadoria, BPC, Bolsa Família
familia-beneficios-pt = reforma, abono de família, subsídio de desemprego
familia-beneficios-us = Social Security, SNAP, seguro-desemprego
familia-beneficios-outros = aposentadoria, pensão, benefícios do governo
familia-dica-beneficios = Boa prática: o benefício é a parte certa do mês; organize as contas fixas por ele e trate o resto como extra.
familia-inicio-mista = No questionário, você disse que a renda é mista. Cadastre cada parte: o salário (ou benefício) e a renda variável separados.
familia-dica-mista = Boa prática: conte com a parte fixa para as contas e guarde uma parte da variável quando ela vier.
familia-inicio-padrao = Cadastre a renda de cada membro da família: salário, aposentadoria, benefício, bicos ou aluguel. Ela é a base da previsão de saldo, do orçamento e das dicas.
familia-dica-padrao = Com a renda cadastrada, o app sabe quanto entra todo mês e quanto dá para guardar.
familia-comece = Comece pela renda da família
familia-boas-praticas-em = Boas práticas em { $pais }
familia-cadastrar-primeira = Cadastrar a primeira renda

## Novo membro

familia-membro-adicionado = Membro adicionado
familia-novo-membro = Novo membro da família
familia-adicionar = Adicionar

## Painel da renda

familia-bruto-12m = Bruto nos próximos 12 meses
familia-liquido-12m = Líquido nos próximos 12 meses
familia-descontos-previstos = { $descontos } previstos
familia-impostos-holerites = Impostos nos { $holerites } (12 meses)
familia-renda-prevista = Renda prevista por mês
familia-renda-fonte = Fonte: rendas cadastradas, último { $holerite } de cada uma e as regras de { $descontos } do ano
familia-renda-real-prevista = Renda por mês: real e prevista
familia-renda-fonte-base = Real: { $holerites } importados. Previsto: último { $holerite } ({ $mes }) e as regras de { $descontos } do ano
familia-renda-tipo = Tipo
familia-renda-real = Real
familia-renda-previsto = Previsto
familia-renda-real-faixa = REAL
familia-renda-previsto-faixa = PREVISTO
familia-renda-linha-prevista = Líquido previsto pelo último { $holerite }
familia-renda-diferenca = Diferença do previsto
familia-renda-diferenca-coluna = Real − previsto
familia-renda-igual = igual
familia-renda-sem-holerite = Sem { $holerite } neste mês.
familia-renda-nota-real = Previsto: o que o último { $holerite } ({ $mes }) daria para este mês.
familia-renda-nota-previsto = Previsão pelo último { $holerite } ({ $mes }).
familia-mes = Mês
familia-extras = Extras
familia-cadastro = Cadastro
familia-dependente-ir-curto = dependente no IR
familia-cadastro-incompleto = Cadastro incompleto: relação, nascimento e ocupação ajudam nas estimativas.
familia-aliquota-efetiva = alíquota efetiva nos holerites: { NUMBER($aliquota, maximumFractionDigits: 1) }%
familia-contas-cartoes = Contas e cartões:
familia-conta-conjunta-dica = Conta conjunta: fica da família
familia-conjunta = (conjunta)
familia-nenhuma = nenhuma
familia-escolher = Escolher
familia-rendas-de = Rendas de { $nome }
familia-nenhuma-renda = Nenhuma renda cadastrada.
familia-previsao = Previsão
familia-formal = formal
familia-informal = informal
familia-holerite-de = { $holerite } de { $mes }
familia-base-holerite = pelo último { $holerite }
familia-base-informados = pelos valores informados
familia-base-media = pela média dos últimos 6 meses
familia-base-nenhum = sem valor informado
familia-sem-membro = Contas e cartões sem membro
familia-sem-membro-explicacao = Escolha de quem é cada conta: os lançamentos dela passam a ter esse membro automaticamente. Deixe sem membro o que é de todos.
familia-membro-de = Membro de { $nome }
familia-da-familia = Da família
familia-holerites-importados = { $holerites } importados
familia-membro = Membro
familia-na-conta = Na conta
familia-ligado = ligado

## Simular demissão (renda CLT, só Brasil)

familia-rescisao-botao = Simular demissão
familia-rescisao-titulo = Simular demissão · { $renda }
familia-rescisao-descricao = Quanto você receberia ao sair deste emprego: as verbas da rescisão, o FGTS e o seguro-desemprego. É uma estimativa pelas regras gerais da CLT; nada é gravado.
familia-rescisao-motivo = Motivo
familia-rescisao-motivo-op = { $motivo ->
    [sem_justa_causa] Dispensa sem justa causa
    [pedido_demissao] Pedido de demissão
    [acordo] Acordo (art. 484-A)
    [justa_causa] Dispensa por justa causa
    [fim_experiencia] Fim do contrato de experiência
    [quebra_experiencia_empregador] Experiência encerrada antes pela empresa
   *[quebra_experiencia_empregado] Experiência encerrada antes por você
}
familia-rescisao-data = Último dia de trabalho
familia-rescisao-aviso = Aviso prévio
familia-rescisao-aviso-op = { $aviso ->
    [indenizado] Indenizado (sem trabalhar)
    [trabalhado] Trabalhado
   *[nao_cumprido] Não cumprido (desconto de 30 dias)
}
familia-rescisao-fim-contrato = Fim combinado da experiência
familia-rescisao-admissao = Admissão
familia-rescisao-ferias-vencidas = Férias vencidas (dias)
familia-rescisao-ferias-dica = Dias de férias de períodos já completos que você não tirou.
familia-rescisao-saldo-fgts = Saldo do FGTS deste emprego
familia-rescisao-saldo-fgts-dica = Vazio: o da conta do FGTS ligada a esta renda, se houver.
familia-rescisao-pedidos = Já recebeu seguro-desemprego
familia-rescisao-pedidos-op = { $n ->
    [0] Nunca
    [1] Uma vez
   *[other] Duas vezes ou mais
}
familia-rescisao-preencha = Informe as datas para ver a simulação.
familia-rescisao-verbas = Rescisão
familia-rescisao-liquido = Líquido da rescisão
familia-rescisao-fgts = FGTS
familia-rescisao-fgts-saldo = Saldo para fins rescisórios
familia-rescisao-fgts-deposito = Depósito sobre as verbas
familia-rescisao-fgts-multa = Multa de { $pct }%
familia-rescisao-fgts-sacavel = Pode sacar
familia-rescisao-seguro = Seguro-desemprego
familia-rescisao-seguro-parcelas = { $n } parcelas de
familia-rescisao-seguro-total = Total do seguro
familia-rescisao-total = Total disponível
familia-rescisao-meses = Cobre cerca de { $meses } meses do seu salário líquido de hoje.
familia-rescisao-tabelas = Contrato projetado até { $fim }. INSS e IRRF: { $base }.
