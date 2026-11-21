# Variante pt-PT (ecrã 02 · Família): o que faltar cai para pt-BR. Revisão: rascunho por IA.
# "Renda" (do trabalho) é "rendimento"; o holerite é o "recibo de vencimento".

## Como o Brasil chama as coisas do rendimento (os outros países vêm do núcleo)

familia-tipo-servidor = Funcionário público
familia-tipo-aposentadoria = Reforma
familia-tipo-autonomo = Trabalhador independente
familia-tipo-aluguel = Rendas
familia-tipo-beneficio = Prestação social
familia-holerite = recibo de vencimento
familia-holerites = recibos de vencimento

## Ficha do membro

familia-cadastro-de = Ficha de { $nome }
familia-dependente-ir = Dependente no IRS
familia-sem-cpf = A aplicação não guarda NIF, CPF nem documentos.
familia-cadastro-salvo = Ficha guardada

## Rendimento

familia-renda-alterada = Rendimento alterado
familia-renda-cadastrada = Rendimento registado
familia-nova-renda = Novo rendimento
familia-nome-exemplo = Salário, reforma, biscates…
familia-liquido-dica = O que entra na conta.
familia-empregador-fonte = Empregador ou entidade pagadora
familia-registro-empresa = Número fiscal da empresa
familia-registro-dica = Liga automaticamente os { $holerites } a este rendimento.
familia-valor-variavel = O valor muda de mês para mês
familia-valor-variavel-dica = Comissões, biscates, vendas: a previsão usa a média dos últimos 6 meses dos lançamentos da categoria do rendimento, quando houver.

## Recibo de vencimento

familia-importar-descricao = O PDF do { $holerite }: verifica-o antes de gravar. O nome e os documentos pessoais não são guardados.
familia-senha-se-tiver = Palavra-passe (se tiver)
familia-ler-de-novo = Ler novamente
familia-lendo = A ler o { $holerite }…
familia-renda = Rendimento
familia-renda-nova = novo ({ $tipo })
familia-nao-achado = não encontrado

## Contas e cartões de cada membro

familia-contas-descricao = Os lançamentos de uma conta ou cartão de um único membro ficam logo com ele, incluindo os antigos (exceto os que mudou à mão). Uma conta conjunta, com dois ou mais membros, fica da família.

## Começo guiado pelo questionário

familia-inicio-fixa-calcula = No questionário, disse que o rendimento principal é fixo. Registe o salário com o bruto e o líquido, ou importe um { $holerite }: a aplicação calcula { $itens }.
familia-inicio-fixa-recibo = No questionário, disse que o rendimento principal é fixo. Registe o salário com o bruto e o líquido, ou importe um { $holerite } (os descontos vêm do recibo; a aplicação não calcula os impostos deste país).
familia-dica-fixa = Boa prática: separe a poupança no dia do pagamento, antes dos gastos (pague-se a si primeiro).
familia-inicio-variavel = No questionário, disse que o rendimento é variável. Registe-o como rendimento variável: a previsão usa a média dos últimos 6 meses.
familia-dica-variavel = Boa prática: planeie o mês pelo valor dos meses mais fracos e mantenha uma reserva de 6 a 12 meses de gastos.
familia-inicio-beneficios = No questionário, disse que o rendimento vem de prestações sociais. Registe cada uma ({ $exemplos }) com o valor do mês.
familia-beneficios-us = Social Security, SNAP, subsídio de desemprego
familia-beneficios-outros = reforma, pensão, apoios do Estado
familia-dica-beneficios = Boa prática: a prestação social é a parte certa do mês; organize as contas fixas por ela e trate o resto como extra.
familia-inicio-mista = No questionário, disse que o rendimento é misto. Registe cada parte: o salário (ou a prestação social) e o rendimento variável em separado.
familia-inicio-padrao = Registe o rendimento de cada membro da família: salário, reforma, prestações sociais, biscates ou rendas. É a base da previsão de saldo, do orçamento e das dicas.
familia-dica-padrao = Com o rendimento registado, a aplicação sabe quanto entra todos os meses e quanto pode guardar.
familia-comece = Comece pelo rendimento da família
familia-cadastrar-primeira = Registar o primeiro rendimento

## Painel do rendimento

familia-renda-prevista = Rendimento previsto por mês
familia-renda-fonte = Fonte: rendimentos registados, último { $holerite } de cada um e as regras de { $descontos } do ano
familia-cadastro = Ficha
familia-dependente-ir-curto = dependente no IRS
familia-cadastro-incompleto = Ficha incompleta: relação, nascimento e ocupação ajudam nas estimativas.
familia-aliquota-efetiva = taxa efetiva nos recibos de vencimento: { NUMBER($aliquota, maximumFractionDigits: 1) }%
familia-rendas-de = Rendimentos de { $nome }
familia-nenhuma-renda = Nenhum rendimento registado.
familia-base-informados = pelos valores indicados
familia-base-nenhum = sem valor indicado

familia-rescisao-descricao = Quanto receberia ao sair deste emprego: as verbas da rescisão, o FGTS e o seguro-desemprego. É uma estimativa pelas regras gerais da CLT; nada é guardado.
familia-rescisao-ferias-dica = Dias de férias de períodos já completos que não gozou.
familia-rescisao-meses = Cobre cerca de { $meses } meses do seu salário líquido atual.
