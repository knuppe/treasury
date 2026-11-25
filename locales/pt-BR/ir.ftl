# Relatórios › IR: apuração mensal de renda variável, DARFs e relatório anual para a declaração
# (só no Brasil; docs/i18n-design.md). Termos oficiais (DARF, IRRF, IR, JCP, FII, CNPJ, Sicalc,
# Receita Federal) ficam no original. pt-BR é o idioma de origem.

## Situação do mês, grupos de prejuízo e DARF

ir-mes-sem-vendas = Sem vendas
ir-mes-sem-vendas-dica = Nenhuma venda de renda variável no mês.
ir-mes-sem-imposto = Sem imposto
ir-mes-sem-imposto-dica = Houve vendas, mas sem imposto: isenção, prejuízo ou compensação.
ir-mes-abaixo-minimo = Abaixo de R$ 10
ir-mes-abaixo-minimo-dica = Imposto abaixo de R$ 10,00: não se paga DARF; o valor soma no mês seguinte.
ir-mes-a-pagar = A pagar
ir-mes-a-pagar-dica = Há imposto a pagar até o último dia útil do mês seguinte.
ir-mes-pago = Pago
ir-mes-pago-dica = DARF do mês pago.
ir-grupo-comum = Comum (ações, ETFs, BDRs)
ir-grupo-comum-curto = Comum
ir-grupo-day-trade = Day trade
ir-grupo-fii = FIIs
ir-grupo-fii-curto = FII
ir-darf-em-aberto = Em aberto
ir-darf-cancelado = Cancelado
ir-darf-vencido = Vencido
ir-aviso = Apuração auxiliar calculada com os seus dados; confira no programa da Receita Federal. Não é aconselhamento tributário.

## Painel

ir-secao-apuracao = Apuração mensal
ir-secao-darf = DARFs
ir-secao-declaracao = Declaração anual
ir-aviso-titulo = Não é aconselhamento tributário
ir-aviso-regras = Regras de renda variável para pessoa física (15% em operações comuns, 20% em day trade e FIIs, isenção de R$ 20 mil/mês em ações à vista).
ir-ano-calendario = Ano-calendário
ir-membro = Membro
ir-familia-toda = Família toda
ir-secao = Seção do IR
ir-apurando = Apurando o ano
ir-imposto-em = Imposto em { $ano }
ir-renda-variavel = Renda variável
ir-darfs-total = DARFs: { $valor }
ir-a-pagar = A pagar
ir-meses-sem-darf = Meses sem DARF
ir-meses-sem-darf-n = { $n ->
    [1] { $n } mês sem DARF
   *[other] { $n } meses sem DARF
}
ir-darfs-vencidos = { $n ->
    [1] ! { $n } DARF vencido
   *[other] ! { $n } DARFs vencidos
}
ir-prejuizos = Prejuízos
ir-somando-grupos = Somando os três grupos
ir-fim-periodo = ao fim do período apurado
ir-carregando-darfs = Carregando os DARFs
ir-sem-darf-em = Imposto a pagar sem DARF em { $meses }.
ir-fonte-regras = regras de renda variável vigentes em { $ano }

## Copiar

ir-copiar = Copiar
ir-copiado = ✓ Copiado
ir-copiar-algo = Copiar { $algo }
ir-copiar-a-mao = Selecione o valor e copie à mão.

## Apuração do mês

ir-meses-vazio = Nenhuma venda de renda variável neste ano.
ir-mes = Mês
ir-vendas-acoes = Vendas de ações
ir-isento = isento
ir-imposto = Imposto
ir-para-proximo = { $valor } → próximo
ir-situacao = Situação
ir-grupo = Grupo
ir-resultado = Resultado
ir-prejuizo-antes = Prejuízo antes
ir-compensado = Compensado
ir-base = Base
ir-aliquota = Alíquota
ir-prejuizo-compensar = Prejuízo a compensar
ir-apuracao-de = Apuração de { $mes }
ir-darf-gerado = DARF gerado
ir-darf-gerado-texto = Código { $codigo }, vence em { $vencimento }. Emita a guia no Sicalc com estes valores.
ir-gerar-darf = Gerar DARF de { $valor }
ir-ver-darf = Ver o DARF
ir-notas-apuracao = Notas da apuração
ir-incompleto = incompleto
ir-incompleto-dica = Há venda sem compra registrada neste mês
ir-incompleto-titulo = Apuração incompleta
ir-incompleto-texto = Uma venda deste mês não tem compra registrada. O valor vendido conta no total do mês, mas o resultado dela não entra na apuração, e o imposto pode estar menor que o devido.
ir-incompleto-link = Ver as vendas sem compra
ir-vendas-categoria = Vendas e resultado por categoria
ir-categorias-de = Categorias de { $mes }
ir-categoria = Categoria
ir-vendas = Vendas
ir-ganho-isento = Ganho isento
ir-prejuizo-mesmo-grupo = prejuízo só compensa no mesmo grupo
ir-prejuizos-base = Prejuízos e base de cálculo
ir-grupos-prejuizo = Grupos de prejuízo
ir-imposto-mes = Imposto do mês
ir-irrf-estimado = IRRF (≈ estimado)
ir-irrf-mes = IRRF do mês
ir-irrf-anteriores = IRRF de meses anteriores
ir-irrf-compensado = IRRF compensado
ir-devido-mes = Devido no mês
ir-acumulado = Acumulado (< R$ 10)
ir-valor-darf-de = valor do DARF de { $periodo }
ir-codigo-receita = Código da receita
ir-periodo-apuracao = Período de apuração
ir-vencimento = Vencimento
ir-vai-mes-seguinte = Vai para o mês seguinte
ir-irrf-depois = IRRF a compensar depois
ir-vendas-apuracao = Vendas que entraram na apuração
ir-vendas-de = Vendas de { $mes }
ir-data = Data
ir-ativo = Ativo
ir-quantidade = Quantidade
ir-venda = Venda
ir-custo-medio = Custo médio

## DARFs

ir-darfs-venceram = { $n ->
    [1] ! Um DARF venceu sem pagamento: recalcule no Sicalc com multa e juros antes de pagar.
   *[other] ! { $n } DARFs venceram sem pagamento: recalcule no Sicalc com multa e juros antes de pagar.
}
ir-darfs-vazio = Nenhum DARF gerado neste ano. Gere pela apuração do mês com imposto a pagar.
ir-periodo = Período
ir-codigo = Código
ir-valor = Valor
ir-acoes = Ações
ir-cancelar = Cancelar
ir-marcar-pago-de = Marcar DARF de { $periodo } como pago
ir-marcar-pago = Marcar pago
ir-lancamento = Lançamento
ir-cancelar-darf = Cancelar DARF
ir-cancelar-darf-texto = O DARF de { $periodo } sai da lista e o mês volta a ficar a pagar (dá para gerar outro). Dá para desfazer pelo histórico de alterações.
ir-darf-cancelado-toast = DARF cancelado
ir-darf-pago-titulo = DARF de { $periodo } pago
ir-darf-pago-descricao = Código { $codigo }, vencimento { $vencimento }.
ir-darf-pago = DARF pago
ir-despesa-lancada = Despesa lançada em Impostos e taxas › Imposto de renda.
ir-marcar-como-pago = Marcar como pago
ir-data-pagamento = Data do pagamento
ir-lancar-despesa = Lançar a despesa na conta
ir-lancar-despesa-dica = Opcional: cria a despesa em Impostos e taxas › Imposto de renda. Se já importou o extrato, deixe em branco.
ir-nao-lancar = Não lançar
ir-pago-atrasado = Pago depois do vencimento: o valor com multa e juros (Sicalc) pode ser maior que o registrado aqui.

## Declaração anual

ir-ficha-isentos = Rendimentos isentos
ir-ficha-exclusiva = Tributação exclusiva
ir-linha-dividendos = Lucros e dividendos recebidos
ir-linha-isentos-acoes = Ganhos líquidos em ações com vendas até R$ 20 mil no mês
ir-linha-fii = Rendimentos de FII
ir-linha-jcp = Juros sobre capital próprio
ir-n-ativos = { $n ->
    [1] { $titulo } · { $n } ativo
   *[other] { $titulo } · { $n } ativos
}
ir-copiar-de = { $titulo } de { $ativo }
ir-rendimentos-ano = Rendimentos · ano-calendário { $ano }
ir-rendimentos-declaracao = Rendimentos para a declaração
ir-ficha = Ficha
ir-codigo-muda = O código muda entre versões do programa: confira
ir-confira = confira
ir-descricao = Descrição
ir-dividendos = Dividendos
ir-jcp = JCP
ir-rendimentos-nota = Valores líquidos recebidos em { $ano }, em reais. O programa pede a fonte pagadora (CNPJ) de cada linha: confira no informe de rendimentos da corretora.
ir-resumo-renda-variavel = Renda variável · resumo do ano
ir-impostos-ano = Impostos do ano
ir-imposto-apurado = Imposto apurado
ir-darfs-ano = DARFs do ano
ir-total-darfs = total de DARFs
ir-ganhos-isentos = Ganhos isentos (ações)
ir-ganhos-isentos-copiar = ganhos isentos de ações
ir-prejuizos-31-12 = Prejuízos a compensar em 31/12
ir-prejuizo-de = prejuízo { $grupo }
ir-bens-em = Bens e direitos em 31/12/{ $ano }
ir-bens-situacao = custo de aquisição (preço médio), não valor de mercado
ir-codigos = Códigos
ir-codigos-sugeridos = sugeridos: confira no programa
ir-bens-direitos = Bens e direitos
ir-bens-vazio = Nenhuma posição no fim do ano.
ir-grupo-codigo = Grupo · código
ir-discriminacao = Discriminação
ir-discriminacao-de = discriminação de { $ativo }
ir-situacao-em = situação em 31/12/{ $ano } de { $ativo }
ir-cripto-titulo = Criptoativos no Brasil (ganho de capital)
ir-cripto-vendido = Vendido em cripto no mês (Brasil e exterior)
ir-cripto-vendido-brasil = Vendido em corretoras no Brasil
ir-cripto-ganho = Ganho das vendas no Brasil
ir-cripto-perda = Perdas (não abatem o ganho)
ir-cripto-situacao = Situação
ir-cripto-isento = Isento: vendas até R$ 35 mil no mês
ir-cripto-tributado = Tributado: vendas acima de R$ 35 mil no mês
ir-cripto-nota = Cripto custodiada no Brasil: imposto de 15% a 22,5% sobre o ganho, em DARF próprio (código 4600), quando o total vendido em cripto no mês, no Brasil e no exterior, passa de R$ 35 mil. Cripto custodiada no exterior vai para Aplicações no exterior, no relatório anual.
ir-cripto-exterior = Exterior
ir-cripto-brasil = Brasil
ir-cripto-custodia-auto = Automático ({ $lugar })
ir-cripto-custodia-de = Custódia da cripto em { $conta }
ir-cripto-custodia-dica = Onde a cripto de cada conta está custodiada. No automático, conta em reais é Brasil e conta em outra moeda é exterior.
ir-cripto-darf-pago = DARF 4600 pago
ir-cripto-darf-gerado = DARF 4600 gerado
ir-irrf-sobra-declaracao = IRRF "dedo-duro" para a declaração (Imposto pago/retido)
ir-irrf-sobra-declaracao-rotulo = IRRF da Lei 11.033 para a declaração
ir-irrf-sobra-day-trade = IRRF de day trade não compensado (só por restituição)
ir-exterior = Aplicações no exterior (Lei 14.754/2023)
ir-ganho-realizado = Ganho realizado no ano
ir-ganho-exterior = ganho no exterior
ir-dividendos-juros = Dividendos e juros (brutos)
ir-rendimentos-exterior = rendimentos do exterior
ir-imposto-pago-exterior = Imposto pago no exterior
ir-imposto-pago-exterior-rotulo = imposto pago no exterior
ir-credito-exterior = Crédito usado (até 15% de cada rendimento)
ir-sem-credito-exterior = Sem crédito (país sem acordo nem reciprocidade)
ir-prejuizo-exterior-anterior = Prejuízo de anos anteriores
ir-prejuizo-exterior-seguinte = Prejuízo para os anos seguintes
ir-imposto-estimado-15 = Imposto estimado (15%)
ir-exterior-nota = Apurado na declaração anual: 15% sobre o ganho (depois do prejuízo de anos anteriores) e sobre dividendos e juros brutos, menos o imposto pago lá fora, que só vale até o imposto brasileiro de cada rendimento. Inclui no ganho os criptoativos de contas em outra moeda (os de contas em reais são apurados mês a mês, com DARF 4600).
ir-anual-titulo = Declaração de ajuste de { $ano } (prévia)
ir-anual-carregando = Montando a prévia da declaração…
ir-anual-dica = Valores do ano tirados do cofre. Confira com os informes de rendimentos e os recibos e ajuste o que precisar: o cálculo refaz os dois modelos na hora, sem gravar nada.
ir-anual-campo-income = Rendimentos tributáveis de empresas (sem 13º e PLR)
ir-anual-campo-social_security = Previdência oficial (INSS)
ir-anual-campo-withheld = Imposto retido na fonte
ir-anual-campo-self_employed = Rendimentos de pessoas físicas e do exterior
ir-anual-campo-carne_leao = Carnê-leão pago
ir-anual-campo-medical = Despesas médicas
ir-anual-campo-education = Despesas com instrução
ir-anual-campo-private_pension = Previdência complementar (PGBL)
ir-anual-campo-alimony = Pensão alimentícia (judicial ou escritura)
ir-anual-campo-complementary_tax = Imposto complementar
ir-anual-campo-irrf_lei_11033 = IRRF "dedo-duro" (Lei 11.033)
ir-anual-campo-dependentes = Dependentes
ir-anual-completo = Completo
ir-anual-simplificado = Simplificado
ir-anual-mais-vantajoso = Mais vantajoso
ir-anual-deducoes = Deduções
ir-anual-base = Base de cálculo
ir-anual-imposto-tabela = Imposto pela tabela
ir-anual-reducao = Redução da Lei 15.270
ir-anual-doacoes = Doações (ECA e pessoa idosa)
ir-anual-devido = Imposto devido
ir-anual-pago = Imposto já pago
ir-anual-a-pagar = Saldo a pagar
ir-anual-a-restituir = A restituir
ir-anual-aliquota-efetiva = Alíquota efetiva no modelo mais vantajoso: { $aliquota }.
ir-anual-restaurar = Voltar aos valores do cofre
ir-anual-de-onde = De onde vêm os valores
ir-dividendos-acima-titulo = Dividendos acima de R$ 50 mil no mês de uma mesma empresa
ir-dividendos-acima-linha = { $mes }: { $ativos }, { $valor } → retenção de 10% sobre o total ({ $retencao })
ir-dividendos-acima-nota = Desde 2026 a empresa retém 10% de tudo o que pagar no mês acima desse limite (Lei 15.270/2025). Lucros de até 2025 com distribuição aprovada até 31/12/2025 ficam de fora: confira no informe da empresa. O valor retido conta no imposto mínimo das altas rendas.
ir-irpfm-titulo = Imposto mínimo das altas rendas (IRPFM)
ir-irpfm-rendimentos = Rendimentos do ano na base
ir-irpfm-bruto = IRPFM antes das deduções ({ $aliquota })
ir-irpfm-creditos = IR da declaração, exclusivo e definitivo
ir-irpfm-devido = IRPFM a pagar (negativo: a compensar)
ir-darf-atraso-hoje = Pagando hoje: { $total } (multa { $multa } + juros { $juros })
ir-darf-atraso-hoje-estimado = Pagando hoje: ≈ { $total } (multa { $multa } + juros { $juros }; Selic de algum mês estimada)
ir-darf-com-acrescimos = Inclui multa { $multa } e juros { $juros }
ir-carne-titulo = Carnê-leão do mês (rendimentos de pessoas físicas e do exterior)
ir-carne-dica = Para quem recebe de pessoa física ou de fora (aluguel de inquilino pessoa física, atendimento particular, trabalho para o exterior): o imposto sai pela tabela mensal e é pago até o fim do mês seguinte.
ir-carne-recebido = Recebido no mês
ir-carne-inss = Previdência oficial paga
ir-carne-pensao = Pensão alimentícia paga
ir-carne-livro-caixa = Livro-caixa (despesas do trabalho)
ir-carne-imposto = Carnê-leão do mês
ir-carne-darf = código { $codigo }, vence em { $vencimento }
ir-carne-abaixo-minimo = Abaixo de R$ 10: soma ao próximo mês
