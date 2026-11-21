# Meta de faculdade dos filhos (núcleo: `goals::education` e `goals::holdings`;
# docs/design/college-goal.md): erros, notas da conta e motivos do histórico. pt-BR é a origem.

nucleo-educacao-plano = plano de educação da meta { $id }
nucleo-educacao-conta = conta { $id }
nucleo-educacao-ativo = ativo { $id }
nucleo-educacao-erro-sem-filho = escolha o filho ou informe a data prevista do nascimento
nucleo-educacao-erro-sem-nascimento = cadastre a data de nascimento do filho (Família) ou informe a data de início do curso
nucleo-educacao-erro-inicio = a idade ou o mês de início do curso não é válido
nucleo-educacao-erro-duracao = a duração do curso vai de 1 a 180 meses
nucleo-educacao-erro-parcelas = as parcelas por ano são 1, 2, 4 ou 12
nucleo-educacao-erro-valor-negativo = mensalidade, extras e custo de vida não podem ser negativos
nucleo-educacao-erro-desconto = o desconto vai de 0% a 100%
nucleo-educacao-erro-retorno = o retorno real suposto vai de −5% a 15% ao ano
nucleo-educacao-erro-diferenca = informe a diferença sobre a inflação, de −10% a 20% ao ano
nucleo-educacao-erro-serie = a série de inflação da educação precisa ser uma série mensal do catálogo
nucleo-educacao-erro-posicao-repetida = a mesma conta (ou o mesmo ativo na conta) aparece duas vezes
nucleo-educacao-erro-fracao = a fração vai de 1% a 100%
nucleo-educacao-erro-fracao-soma = as frações de { $nome } entre as metas passariam de 100% (livre: { $livre })
nucleo-educacao-nota-sem-serie = Sem anos completos da série de inflação da educação: a conta usa só a inflação geral. Sincronize os índices ou informe uma diferença fixa.
nucleo-educacao-nota-inflacao-suposta = Sem série de inflação geral com anos completos: os valores em dinheiro da época supõem { $taxa } ao ano.
nucleo-educacao-nota-publica = Em faculdade pública, o custo costuma ser o de vida (moradia, transporte, alimentação em outra cidade). Vale preencher o custo de vida por mês.
nucleo-educacao-nota-exterior = Curso no exterior: a conta usa a moeda da meta e não inclui a variação do câmbio, que é um risco à parte.
nucleo-educacao-nota-sem-mensalidade = Falta a mensalidade de hoje: peça uma cotação à faculdade e informe no plano.
nucleo-educacao-nota-terminou = O curso terminou. A meta pode ser arquivada, e o que sobrou fica livre.
nucleo-educacao-meta-nome = Faculdade de { $nome }
nucleo-educacao-meta-nome-previsto = Faculdade (nascimento previsto em { $data })
nucleo-educacao-meta-nome-sem-filho = Faculdade
nucleo-educacao-motivo-meta-criada = Meta de faculdade criada
nucleo-educacao-motivo-posicoes = Posições ligadas à meta: { $nome }
