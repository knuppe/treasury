# Children's college goal (core: `goals::education` and `goals::holdings`;
# docs/design/college-goal.md): errors, notes of the calculation and history reasons.

nucleo-educacao-plano = education plan of goal { $id }
nucleo-educacao-conta = account { $id }
nucleo-educacao-ativo = asset { $id }
nucleo-educacao-erro-sem-filho = choose the child or enter the expected birth date
nucleo-educacao-erro-sem-nascimento = add the child's birth date (Family) or enter the course start date
nucleo-educacao-erro-inicio = the course start age or month is not valid
nucleo-educacao-erro-duracao = the course length goes from 1 to 180 months
nucleo-educacao-erro-parcelas = payments per year are 1, 2, 4 or 12
nucleo-educacao-erro-valor-negativo = tuition, extras and living costs can't be negative
nucleo-educacao-erro-desconto = the discount goes from 0% to 100%
nucleo-educacao-erro-retorno = the assumed real return goes from −5% to 15% a year
nucleo-educacao-erro-diferenca = enter the difference over inflation, from −10% to 20% a year
nucleo-educacao-erro-serie = the education inflation series must be a monthly series from the catalog
nucleo-educacao-erro-posicao-repetida = the same account (or the same asset in the account) appears twice
nucleo-educacao-erro-fracao = the share goes from 1% to 100%
nucleo-educacao-erro-fracao-soma = the shares of { $nome } across goals would go over 100% (free: { $livre })
nucleo-educacao-nota-sem-serie = No complete years of the education inflation series: the calculation uses only general inflation. Sync the indices or enter a fixed difference.
nucleo-educacao-nota-inflacao-suposta = No general inflation series with complete years: amounts in money of the time assume { $taxa } a year.
nucleo-educacao-nota-publica = At a public university, the cost is usually living costs (housing, transport, food in another city). It's worth filling in the monthly living cost.
nucleo-educacao-nota-exterior = Course abroad: the calculation uses the goal's currency and doesn't include exchange rate changes, which are a separate risk.
nucleo-educacao-nota-sem-mensalidade = Today's tuition is missing: ask the university for a quote and enter it in the plan.
nucleo-educacao-nota-terminou = The course has ended. The goal can be archived, and what is left is free.
nucleo-educacao-meta-nome = College for { $nome }
nucleo-educacao-meta-nome-previsto = College (birth expected on { $data })
nucleo-educacao-meta-nome-sem-filho = College
nucleo-educacao-motivo-meta-criada = College goal created
nucleo-educacao-motivo-posicoes = Holdings linked to the goal: { $nome }
