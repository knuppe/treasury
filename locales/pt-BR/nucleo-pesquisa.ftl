# Análise de ativos (núcleo: `research`): nomes e fórmulas dos indicadores, períodos, fontes,
# motivos de indisponibilidade, notas, erros e textos da coleta, no idioma da instalação. pt-BR é
# o idioma de origem. Os nomes de documentos gravados com cada valor (DFP, ITR, informes da CVM)
# e os textos que casam com os arquivos da CVM, da B3 e da SEC ficam no código.

## Histórico e comparação

nucleo-pesquisa-hist-centavos = Valores monetários em centavos da moeda do ativo.
nucleo-pesquisa-hist-sem-demonstracoes = Sem demonstrações baixadas para este ativo.
nucleo-pesquisa-hist-unit = Valores por papel da unit ({ $acoes } ações).
nucleo-pesquisa-erro-comparar-minimo = compare pelo menos 2 ativos
nucleo-pesquisa-erro-comparar-maximo = compare no máximo { $max } ativos

## Comandos

nucleo-pesquisa-erro-ajustes-so-interface = ajustes da análise de ativos só pela interface do app
nucleo-pesquisa-erro-ajuste-desconhecido = ajuste desconhecido: { $ajuste }
nucleo-pesquisa-erro-taxa = taxa inválida
nucleo-pesquisa-motivo-tela-criada = Tela do screener criada: { $tela }
nucleo-pesquisa-motivo-tela-alterada = Tela do screener alterada
nucleo-pesquisa-motivo-tela-removida = Tela do screener removida

## Indicadores: nome e fórmula

nucleo-pesquisa-ind-price = Cotação
nucleo-pesquisa-ind-price-formula = Último fechamento disponível
nucleo-pesquisa-ind-market_cap = Valor de mercado
nucleo-pesquisa-ind-market_cap-formula = Σ ações de cada classe × cotação da classe (FII: cotas × cotação)
nucleo-pesquisa-ind-ev = Valor da firma (EV)
nucleo-pesquisa-ind-ev-formula = Valor de mercado + dívida bruta − caixa e aplicações financeiras + participação dos não controladores
nucleo-pesquisa-ind-liquidity = Liquidez média diária
nucleo-pesquisa-ind-liquidity-formula = Média do volume financeiro diário nos últimos pregões (até 21)
nucleo-pesquisa-ind-pl = P/L
nucleo-pesquisa-ind-pl-formula = Valor de mercado ÷ lucro líquido dos controladores nos últimos 12 meses
nucleo-pesquisa-ind-pvp = P/VP
nucleo-pesquisa-ind-pvp-formula = Valor de mercado ÷ patrimônio líquido dos controladores (FII: cotação ÷ valor patrimonial por cota)
nucleo-pesquisa-ind-ev_ebitda = EV/EBITDA
nucleo-pesquisa-ind-ev_ebitda-formula = Valor da firma ÷ EBITDA dos últimos 12 meses (EBIT + depreciação e amortização)
nucleo-pesquisa-ind-ev_ebit = EV/EBIT
nucleo-pesquisa-ind-ev_ebit-formula = Valor da firma ÷ EBIT dos últimos 12 meses
nucleo-pesquisa-ind-psr = P/Receita (PSR)
nucleo-pesquisa-ind-psr-formula = Valor de mercado ÷ receita líquida dos últimos 12 meses
nucleo-pesquisa-ind-p_ativo = P/Ativo
nucleo-pesquisa-ind-p_ativo-formula = Valor de mercado ÷ ativo total
nucleo-pesquisa-ind-dy = Dividend yield (12 meses)
nucleo-pesquisa-ind-dy-formula = Proventos por ação com data-com nos últimos 12 meses ÷ cotação
nucleo-pesquisa-ind-payout = Payout
nucleo-pesquisa-ind-payout-formula = Dividendos e JCP declarados nos últimos 12 meses ÷ lucro líquido
nucleo-pesquisa-ind-last_dividend = Último rendimento
nucleo-pesquisa-ind-last_dividend-formula = Último provento por cota (agenda; sem agenda, DY do mês × VP por cota do informe)
nucleo-pesquisa-ind-roe = ROE
nucleo-pesquisa-ind-roe-formula = Lucro líquido dos últimos 12 meses ÷ patrimônio líquido
nucleo-pesquisa-ind-roic = ROIC
nucleo-pesquisa-ind-roic-formula = EBIT dos últimos 12 meses × (1 − alíquota nominal de IR) ÷ (patrimônio líquido + não controladores + dívida líquida)
nucleo-pesquisa-ind-roa = ROA
nucleo-pesquisa-ind-roa-formula = Lucro líquido consolidado (com não controladores) dos últimos 12 meses ÷ ativo total
nucleo-pesquisa-ind-gross_margin = Margem bruta
nucleo-pesquisa-ind-gross_margin-formula = Lucro bruto ÷ receita líquida (12 meses)
nucleo-pesquisa-ind-ebitda_margin = Margem EBITDA
nucleo-pesquisa-ind-ebitda_margin-formula = EBITDA ÷ receita líquida (12 meses)
nucleo-pesquisa-ind-ebit_margin = Margem EBIT
nucleo-pesquisa-ind-ebit_margin-formula = EBIT ÷ receita líquida (12 meses)
nucleo-pesquisa-ind-net_margin = Margem líquida
nucleo-pesquisa-ind-net_margin-formula = Lucro líquido consolidado (com não controladores) ÷ receita líquida (12 meses)
nucleo-pesquisa-ind-net_debt_equity = Dívida líquida / PL
nucleo-pesquisa-ind-net_debt_equity-formula = (Dívida bruta − caixa e aplicações) ÷ patrimônio líquido
nucleo-pesquisa-ind-net_debt_ebitda = Dívida líquida / EBITDA
nucleo-pesquisa-ind-net_debt_ebitda-formula = (Dívida bruta − caixa e aplicações) ÷ EBITDA dos últimos 12 meses
nucleo-pesquisa-ind-current_ratio = Liquidez corrente
nucleo-pesquisa-ind-current_ratio-formula = Ativo circulante ÷ passivo circulante
nucleo-pesquisa-ind-p_ebit = P/EBIT
nucleo-pesquisa-ind-p_ebit-formula = Valor de mercado ÷ EBIT dos últimos 12 meses
nucleo-pesquisa-ind-p_working_capital = P/Capital de giro
nucleo-pesquisa-ind-p_working_capital-formula = Valor de mercado ÷ (ativo circulante − passivo circulante)
nucleo-pesquisa-ind-p_ncav = P/Ativo circulante líquido
nucleo-pesquisa-ind-p_ncav-formula = Valor de mercado ÷ (ativo circulante − passivo total), o "net-net" de Graham
nucleo-pesquisa-ind-ebit_assets = EBIT/Ativo
nucleo-pesquisa-ind-ebit_assets-formula = EBIT dos últimos 12 meses ÷ ativo total
nucleo-pesquisa-ind-asset_turnover = Giro dos ativos
nucleo-pesquisa-ind-asset_turnover-formula = Receita líquida dos últimos 12 meses ÷ ativo total
nucleo-pesquisa-ind-gross_debt_equity = Dívida bruta / PL
nucleo-pesquisa-ind-gross_debt_equity-formula = Empréstimos e financiamentos ÷ patrimônio líquido dos controladores
nucleo-pesquisa-ind-revenue_cagr_5y = CAGR receita 5 anos
nucleo-pesquisa-ind-revenue_cagr_5y-formula = (Receita do último exercício ÷ receita de 5 exercícios antes)^(1/5) − 1
nucleo-pesquisa-ind-earnings_cagr_5y = CAGR lucro 5 anos
nucleo-pesquisa-ind-earnings_cagr_5y-formula = (Lucro do último exercício ÷ lucro de 5 exercícios antes)^(1/5) − 1
nucleo-pesquisa-ind-loss_years_10y = Anos com prejuízo (10 anos)
nucleo-pesquisa-ind-loss_years_10y-formula = Exercícios com lucro líquido negativo entre os últimos 10
nucleo-pesquisa-ind-lpa = LPA
nucleo-pesquisa-ind-lpa-formula = Lucro líquido dos últimos 12 meses ÷ ações em circulação (× ações por unit)
nucleo-pesquisa-ind-vpa = VPA
nucleo-pesquisa-ind-vpa-formula = Patrimônio líquido ÷ ações em circulação (× ações por unit)
nucleo-pesquisa-ind-nav_per_share = Valor patrimonial por cota
nucleo-pesquisa-ind-nav_per_share-formula = Patrimônio líquido ÷ cotas emitidas (informe mensal)
nucleo-pesquisa-ind-holders = Número de cotistas
nucleo-pesquisa-ind-holders-formula = Total de cotistas no último informe mensal
nucleo-pesquisa-ind-vacancy = Vacância
nucleo-pesquisa-ind-vacancy-formula = Área vaga ÷ área bruta locável
nucleo-pesquisa-ind-basel = Índice de Basileia
nucleo-pesquisa-ind-basel-formula = Patrimônio de referência ÷ ativos ponderados pelo risco
nucleo-pesquisa-ind-graham = Preço de Graham
nucleo-pesquisa-ind-graham-formula = √(22,5 × LPA × VPA)
nucleo-pesquisa-ind-graham_upside = Margem até o preço de Graham
nucleo-pesquisa-ind-graham_upside-formula = Preço de Graham ÷ cotação − 1
nucleo-pesquisa-ind-bazin = Preço-teto de Bazin
nucleo-pesquisa-ind-bazin-formula = Média dos dividendos por ação dos últimos N anos ÷ taxa mínima
nucleo-pesquisa-ind-bazin_upside = Margem até o preço de Bazin
nucleo-pesquisa-ind-bazin_upside-formula = Preço-teto de Bazin ÷ cotação − 1

## Indicadores: períodos, fontes e fórmulas montadas na hora

nucleo-pesquisa-periodo-12m = 12 meses até { $data }
nucleo-pesquisa-periodo-saldo = saldo em { $data }
nucleo-pesquisa-fonte-cotacao = cotação { $fonte } de { $data }
nucleo-pesquisa-aviso-preco-justo = Estimativas a partir de fórmulas clássicas e dos dados informados; não são recomendação de investimento.
nucleo-pesquisa-ind-periodo-informe = informe de { $mes }
nucleo-pesquisa-ind-periodo-vp-informe = VP do informe de { $mes }
nucleo-pesquisa-ind-periodo-data-com = data-com { $data }
nucleo-pesquisa-ind-periodo-divida-ebitda = dívida em { $divida }; EBITDA de 12 meses até { $ebitda }
nucleo-pesquisa-ind-periodo-exercicios = exercícios { $de } a { $ate }
nucleo-pesquisa-ind-periodo-n-exercicios =
    { $n ->
        [one] { $n } exercício: { $de } a { $ate }
       *[other] { $n } exercícios: { $de } a { $ate }
    }
nucleo-pesquisa-ind-cotas-x-cotacao = { $cotas } cotas × cotação
nucleo-pesquisa-ind-acoes-x-cotacao = { $acoes } ações × cotação por ação do papel consultado
nucleo-pesquisa-ind-on-pn-x-cotacao = ações ON × cotação ON + ações PN × cotação PN
nucleo-pesquisa-ind-formula-valor-mercado = Valor de mercado = { $como }
nucleo-pesquisa-ind-formula-roic = EBIT dos últimos 12 meses × (1 − { $aliquota }%) ÷ (patrimônio líquido + não controladores + dívida líquida)
nucleo-pesquisa-ind-formula-graham = √(22,5 × LPA × VPA); 22,5 = P/L máximo de 15 × P/VP máximo de 1,5
nucleo-pesquisa-ind-formula-bazin = Média dos dividendos por ação dos últimos N anos ÷ taxa mínima de retorno
nucleo-pesquisa-ind-formula-dy-dfc = Dividendos e JCP pagos nos últimos 12 meses por ação (demonstração de fluxo de caixa) ÷ cotação
nucleo-pesquisa-ind-formula-payout-pagos = Dividendos e JCP pagos nos últimos 12 meses ÷ lucro líquido
nucleo-pesquisa-ind-formula-pvp-fii = Cotação ÷ valor patrimonial por cota (informe mensal)
nucleo-pesquisa-ind-fonte-agenda = agenda de proventos
nucleo-pesquisa-ind-fonte-informe-fii = informe mensal de FII (DY do mês × VP por cota)
nucleo-pesquisa-ind-fonte-dividendos-dfc = dividendos e JCP pagos (DFC) · { $documento }
nucleo-pesquisa-ind-fonte-dividendos-sec = dividendos declarados por ação (SEC)
nucleo-pesquisa-ind-fonte-dividendos-dfc-acoes = dividendos e JCP pagos por ação no exercício (DFC ÷ ações)
nucleo-pesquisa-ind-fonte-dfp-lucro = DFP (lucro líquido dos controladores)
nucleo-pesquisa-ind-media-anos =
    { $anos ->
        [one] média de { $anos } ano com dados (pedidos { $pedidos })
       *[other] média de { $anos } anos com dados (pedidos { $pedidos })
    }
nucleo-pesquisa-ind-media-anos-agenda =
    { $anos ->
        [one] a agenda de proventos começa em { $ano }: média de { $anos } ano com dados (pedidos { $pedidos })
       *[other] a agenda de proventos começa em { $ano }: média de { $anos } anos com dados (pedidos { $pedidos })
    }
nucleo-pesquisa-ind-premissa-lpa = LPA (12 meses)
nucleo-pesquisa-ind-premissa-vpa = VPA
nucleo-pesquisa-ind-premissa-acoes = Ações em circulação
nucleo-pesquisa-ind-premissa-taxa = Taxa mínima
nucleo-pesquisa-ind-premissa-anos = Anos
nucleo-pesquisa-ind-premissa-media = Média anual
nucleo-pesquisa-ind-premissa-dividendos-ano = Dividendos por ação em { $ano }

## Indicadores: notas

nucleo-pesquisa-ind-nota-escala-acoes = A quantidade de ações de { $documento } parece estar em { $escala ->
    [milhares] milhares
   *[unidades] unidades × 1.000
} (P/VP { $pvp } pela quantidade informada): corrigida por um fator de { $fator }.
nucleo-pesquisa-ind-nota-financeira = Instituição financeira: EBITDA, valor da firma, dívida líquida, margens e liquidez corrente não se aplicam.
nucleo-pesquisa-ind-nota-holding = Holding ou seguradora: o resultado vem das participações (equivalência patrimonial); EBIT, valor da firma e margens não se aplicam.
nucleo-pesquisa-ind-nota-banco-padrao-bc = Banco: lucro e patrimônio pelo balanço individual, no padrão contábil do Banco Central (o que o mercado e os dividendos usam). No consolidado em IFRS: lucro de 12 meses de { $lucro } e patrimônio de { $patrimonio }.
nucleo-pesquisa-ind-nota-unit = Unit com { $acoes } ações: valores por ação multiplicados pela composição.
nucleo-pesquisa-ind-nota-classe = Indicadores fundamentalistas valem para ações e FIIs; para esta classe só há cotação e liquidez.
nucleo-pesquisa-ind-nota-sem-demonstracoes = Sem demonstrações baixadas para este ativo: rode a coleta da análise de ativos.
nucleo-pesquisa-ind-nota-acoes-sec = Ações em circulação da capa do último formulário (dei:EntityCommonStockSharesOutstanding).

## Indicadores: por que está indisponível

nucleo-pesquisa-ind-indisponivel = indisponível
nucleo-pesquisa-ind-divisao-zero = divisão por zero
nucleo-pesquisa-ind-nao-se-aplica-financeiras = não se aplica a instituições financeiras e seguradoras
nucleo-pesquisa-ind-nao-se-aplica-holdings = não se aplica a holdings e seguradoras (o resultado vem da equivalência patrimonial)
nucleo-pesquisa-ind-nao-se-aplica-fii = não se aplica a FIIs
nucleo-pesquisa-ind-sem-cotacao = sem cotação
nucleo-pesquisa-ind-sem-cotas = sem o número de cotas no informe mensal
nucleo-pesquisa-ind-sem-acoes = sem o número de ações
nucleo-pesquisa-ind-unit-desconhecida = composição da unit desconhecida
nucleo-pesquisa-ind-sem-rendimentos-informe = sem rendimentos no informe mensal
nucleo-pesquisa-ind-sem-proventos = sem proventos na agenda nem nas demonstrações
nucleo-pesquisa-ind-lpa-negativo = LPA negativo ou zero (prejuízo nos últimos 12 meses)
nucleo-pesquisa-ind-vpa-negativo = VPA negativo ou zero (patrimônio líquido negativo)
nucleo-pesquisa-ind-taxa-positiva = a taxa mínima precisa ser positiva
nucleo-pesquisa-ind-sem-historico-dividendos = sem histórico de dividendos
nucleo-pesquisa-ind-sem-dividendos-periodo = sem dividendos no período
nucleo-pesquisa-ind-sem-lucro-12m = sem lucro dos últimos 12 meses
nucleo-pesquisa-ind-sem-receita-12m = sem receita dos últimos 12 meses
nucleo-pesquisa-ind-sem-ebit-12m = sem EBIT dos últimos 12 meses
nucleo-pesquisa-ind-sem-da-12m = sem depreciação e amortização dos últimos 12 meses
nucleo-pesquisa-ind-sem-lucro-bruto-12m = sem lucro bruto dos últimos 12 meses
nucleo-pesquisa-ind-sem-patrimonio = sem patrimônio líquido
nucleo-pesquisa-ind-sem-ativo-total = sem ativo total
nucleo-pesquisa-ind-sem-divida-bruta = sem dívida bruta
nucleo-pesquisa-ind-sem-ativo-circulante = sem ativo circulante
nucleo-pesquisa-ind-sem-passivo-circulante = sem passivo circulante
nucleo-pesquisa-ind-sem-dividendos = sem dividendos
nucleo-pesquisa-ind-sem-volume = sem volume negociado
nucleo-pesquisa-ind-sem-valor-mercado = sem valor de mercado
nucleo-pesquisa-ind-lucro-zero = lucro zero
nucleo-pesquisa-ind-patrimonio-negativo = patrimônio líquido negativo
nucleo-pesquisa-ind-ebitda-negativo = EBITDA negativo ou zero
nucleo-pesquisa-ind-ebit-negativo = EBIT negativo ou zero
nucleo-pesquisa-ind-receita-zero = receita zero
nucleo-pesquisa-ind-lucro-negativo = lucro negativo ou zero
nucleo-pesquisa-ind-denominador-negativo = denominador negativo ou zero
nucleo-pesquisa-ind-capital-negativo = capital investido negativo ou zero
nucleo-pesquisa-ind-capital-pequeno = capital investido abaixo de 10% do ativo (caixa líquido grande): o ROIC não diz muito
nucleo-pesquisa-ind-capital-giro-negativo = capital de giro negativo ou zero
nucleo-pesquisa-ind-sem-passivo-total = sem passivo total no último balanço
nucleo-pesquisa-ind-ativo-circulante-liquido-negativo = ativo circulante menor que o passivo total
nucleo-pesquisa-ind-bloqueado = dado de entrada inválido: { $motivo }
nucleo-pesquisa-ind-milhoes = R$ { $valor } mi
nucleo-pesquisa-ind-check-lucro-individual = o lucro dos controladores veio zerado no consolidado ({ $informado }); usado o da demonstração individual, { $usado }
nucleo-pesquisa-ind-check-lucro-total = o lucro dos controladores veio zerado no consolidado ({ $informado }); usado o total menos os não controladores, { $usado }
nucleo-pesquisa-ind-check-lucro-diverge = o lucro dos controladores no consolidado ({ $consolidado }) e o da demonstração individual ({ $individual }) diferem { $diferenca }, sem fonte para decidir qual vale
nucleo-pesquisa-ind-check-lucro-difere = o lucro dos controladores no consolidado ({ $consolidado }) difere { $diferenca } do da demonstração individual ({ $individual })
nucleo-pesquisa-ind-check-patrimonio-diverge = o patrimônio dos controladores no consolidado ({ $consolidado }) e o da demonstração individual ({ $individual }) diferem { $diferenca }, sem fonte para decidir qual vale
nucleo-pesquisa-ind-check-patrimonio-difere = o patrimônio dos controladores no consolidado ({ $consolidado }) difere { $diferenca } do da demonstração individual ({ $individual })
nucleo-pesquisa-ind-check-balanco-antigo = o último balanço é de { $data }, com mais de 200 dias
nucleo-pesquisa-ind-check-reapresentado = a companhia reapresentou { $item } do período até { $periodo } ({ $diferenca }); os 12 meses usam o número publicado na época
nucleo-pesquisa-ind-check-proventos-estimados = sem proventos oficiais da B3 para o papel: estimativa pelos dividendos e JCP pagos no fluxo de caixa (fica fora dos rankings)
nucleo-pesquisa-ind-check-acoes-escala = a quantidade de ações de { $documento } estava em milhares; conferida com a B3 ({ $b3 } milhões)
nucleo-pesquisa-ind-check-acoes-total-liquido = a companhia informou à CVM o total de ações já sem as { $tesouraria } milhões em tesouraria ({ $cvm } milhões); conferido com a B3, que tem { $b3 } milhões em { $data }
nucleo-pesquisa-ind-check-acoes-evento = quantidade ajustada por eventos em ações depois do balanço (fator { $fator }): { $cvm } milhões no balanço, { $b3 } milhões na B3 em { $data }
nucleo-pesquisa-ind-check-acoes-diverge = a B3 tem { $b3 } milhões de ações em { $data } e o balanço { $cvm } milhões ({ $diferenca }), sem evento que explique
nucleo-pesquisa-ind-check-acoes-difere = a B3 tem { $b3 } milhões de ações em { $data }, { $diferenca } diferente do balanço ({ $cvm } milhões), sem evento que explique
nucleo-pesquisa-ind-sem-exercicios = sem exercícios
nucleo-pesquisa-ind-poucos-exercicios = menos de 6 exercícios baixados
nucleo-pesquisa-ind-ponta-negativa = valor negativo ou zero numa das pontas
nucleo-pesquisa-ind-basileia-fora-cvm = o Índice de Basileia não está nos arquivos da CVM (fica nas notas explicativas e no IF.data do Banco Central)
nucleo-pesquisa-ind-sem-informe = sem informe mensal
nucleo-pesquisa-ind-sem-vp-cota = sem valor patrimonial por cota
nucleo-pesquisa-ind-sem-rendimentos = sem rendimentos
nucleo-pesquisa-ind-vacancia-fora = sem vacância no informe trimestral da CVM dos últimos 12 meses

## Itens das demonstrações

nucleo-pesquisa-item-revenue = Receita líquida
nucleo-pesquisa-item-gross_profit = Lucro bruto
nucleo-pesquisa-item-ebit = EBIT (resultado antes do resultado financeiro e dos tributos)
nucleo-pesquisa-item-da = Depreciação, amortização e exaustão
nucleo-pesquisa-item-net_income = Lucro líquido (controladores)
nucleo-pesquisa-item-cfo = Caixa líquido das atividades operacionais
nucleo-pesquisa-item-capex = Investimentos em imobilizado e intangível
nucleo-pesquisa-item-dividends_paid = Dividendos e JCP pagos
nucleo-pesquisa-item-dividends_declared = Dividendos e JCP declarados
nucleo-pesquisa-item-dps_declared = Dividendos declarados por ação
nucleo-pesquisa-item-cogs = Custo dos bens e serviços vendidos
nucleo-pesquisa-item-sga = Despesas com vendas, gerais e administrativas
nucleo-pesquisa-item-fin_income = Receitas financeiras
nucleo-pesquisa-item-fin_expense = Despesas financeiras
nucleo-pesquisa-item-ebt = Resultado antes dos tributos sobre o lucro
nucleo-pesquisa-item-income_tax = Imposto de renda e contribuição social (negativo = despesa)
nucleo-pesquisa-item-buybacks = Recompra de ações (aquisição de ações próprias)
nucleo-pesquisa-item-share_issuance = Aumento de capital e emissão de ações
nucleo-pesquisa-item-total_assets = Ativo total
nucleo-pesquisa-item-current_assets = Ativo circulante
nucleo-pesquisa-item-cash = Caixa, equivalentes e aplicações financeiras
nucleo-pesquisa-item-current_liabilities = Passivo circulante
nucleo-pesquisa-item-total_liabilities = Passivo total (exigível)
nucleo-pesquisa-item-gross_debt = Dívida bruta (empréstimos e financiamentos)
nucleo-pesquisa-item-equity = Patrimônio líquido (controladores)
nucleo-pesquisa-item-shares_outstanding = Ações em circulação (sem tesouraria)
nucleo-pesquisa-item-shares_on = Ações ordinárias em circulação
nucleo-pesquisa-item-shares_pn = Ações preferenciais em circulação
nucleo-pesquisa-item-receivables = Contas a receber (circulante)
nucleo-pesquisa-item-inventories = Estoques (circulante)
nucleo-pesquisa-item-ppe = Imobilizado
nucleo-pesquisa-item-intangible = Intangível
nucleo-pesquisa-item-suppliers = Fornecedores (circulante)
nucleo-pesquisa-item-debt_st = Empréstimos e financiamentos de curto prazo
nucleo-pesquisa-item-debt_lt = Empréstimos e financiamentos de longo prazo
nucleo-pesquisa-item-leases = Arrendamentos fora de empréstimos e financiamentos
nucleo-pesquisa-item-retained = Lucros retidos (reservas de lucro + lucros/prejuízos acumulados)
nucleo-pesquisa-item-minority = Participação dos não controladores
nucleo-pesquisa-item-net_income_total = Lucro líquido consolidado (com não controladores)
nucleo-pesquisa-item-net_income_consolidated = Lucro dos controladores no consolidado, como informado
nucleo-pesquisa-item-net_income_individual = Lucro líquido da demonstração individual
nucleo-pesquisa-item-equity_income = Resultado de equivalência patrimonial
nucleo-pesquisa-item-equity_consolidated = Patrimônio dos controladores no consolidado
nucleo-pesquisa-item-equity_individual = Patrimônio líquido da demonstração individual
nucleo-pesquisa-item-shares_total = Ações do capital integralizado (com tesouraria)
nucleo-pesquisa-item-shares_treasury = Ações em tesouraria
nucleo-pesquisa-item-shares_b3 = Ações do capital social na B3
nucleo-pesquisa-item-shares_b3_on = Ações ordinárias do capital social na B3
nucleo-pesquisa-item-shares_b3_pn = Ações preferenciais do capital social na B3
nucleo-pesquisa-item-revenue_restated = Receita líquida reapresentada no documento seguinte
nucleo-pesquisa-item-ebit_restated = EBIT reapresentado no documento seguinte
nucleo-pesquisa-item-net_income_restated = Lucro líquido reapresentado no documento seguinte
nucleo-pesquisa-item-nav = Patrimônio líquido do fundo
nucleo-pesquisa-item-nav_per_share = Valor patrimonial por cota
nucleo-pesquisa-item-fii_shares = Cotas emitidas
nucleo-pesquisa-item-holders = Número de cotistas
nucleo-pesquisa-item-dy_month = Dividend yield do mês sobre o patrimônio (informado)
nucleo-pesquisa-item-income_per_share = Rendimento por cota (DY do mês × VP por cota)
nucleo-pesquisa-item-fii_total_assets = Ativo total do fundo
nucleo-pesquisa-item-fii_liabilities = Passivo total do fundo
nucleo-pesquisa-item-fii_acq_obligations = Obrigações por aquisição de imóveis
nucleo-pesquisa-item-fii_securitization = Obrigações por securitização de recebíveis
nucleo-pesquisa-item-fii_real_estate = Direitos sobre bens imóveis
nucleo-pesquisa-item-fii_cri = CRI e CRA na carteira
nucleo-pesquisa-item-fii_fii_shares = Cotas de outros FIIs na carteira
nucleo-pesquisa-item-fii_liquidity = Ativos para necessidades de liquidez
nucleo-pesquisa-item-fii_rent_receivable = Aluguéis a receber
nucleo-pesquisa-item-fii_admin_fee_pct = Taxa de administração do mês sobre o patrimônio
nucleo-pesquisa-item-fii_equity_return_month = Rentabilidade patrimonial do mês
nucleo-pesquisa-item-fii_vacancy_financial = Vacância financeira (ponderada pela receita)
nucleo-pesquisa-item-fii_vacancy_physical = Vacância física (ponderada pela área)
nucleo-pesquisa-item-fii_default_rate = Inadimplência (ponderada pela receita)
nucleo-pesquisa-item-fii_top_property_share = Maior imóvel na receita do fundo
nucleo-pesquisa-item-fii_properties = Número de imóveis
nucleo-pesquisa-item-fii_top_tenant_share = Maior inquilino na receita do fundo (aproximação)
nucleo-pesquisa-item-fii_walt_years = Prazo médio dos contratos (WALT, anos)
nucleo-pesquisa-item-fii_undetermined_share = Receita de contratos com prazo indeterminado
nucleo-pesquisa-item-fii_recurring_result = Resultado financeiro líquido do trimestre
nucleo-pesquisa-item-fii_declared_income = Rendimentos declarados no trimestre
nucleo-pesquisa-item-fii_cri_top_issuer_share = Maior emissor nos CRI/CRA (aproximação do devedor)
nucleo-pesquisa-item-desconhecido = Item
nucleo-pesquisa-doc-b3-capital = B3, empresas listadas (capital social) em { $data }
nucleo-pesquisa-erro-tipo-periodo = tipo de período inválido: { $tipo }

## Ajustes da análise

nucleo-pesquisa-aviso-sec = A SEC exige identificação com um e-mail de contato: troque "contato@exemplo" pelo seu e-mail em Ajustes › Fontes de dados (ex.: "Treasury/0.1 seu.nome@email.com").
nucleo-pesquisa-erro-bazin-taxa = a taxa mínima do Bazin deve ser positiva (ex.: 0.06 ou 6 para 6%)
nucleo-pesquisa-erro-bazin-anos = os anos da média do Bazin vão de 1 a 10
nucleo-pesquisa-erro-sec-ua-vazio = informe o User-Agent da SEC (nome do app e e-mail)
nucleo-pesquisa-erro-sec-ua-invalido = User-Agent inválido
nucleo-pesquisa-erro-sec-ua-email = a SEC exige um e-mail de contato no User-Agent (ex.: Treasury/0.1 seu.nome@email.com)

## Coleta: B3 e brapi

nucleo-pesquisa-erro-cotahist-sem-txt = COTAHIST sem arquivo .TXT
nucleo-pesquisa-erro-b3-companhia = resposta da B3 (companhia) em formato inesperado: { $erro }
nucleo-pesquisa-erro-b3-fundo = resposta da B3 (fundo) em formato inesperado: { $erro }
nucleo-pesquisa-erro-b3-historico = histórico de proventos da B3 em formato inesperado: { $erro }
nucleo-pesquisa-erro-b3-vazia = a B3 respondeu sem conteúdo
nucleo-pesquisa-erro-304 = resposta 304 sem pedido condicional
nucleo-pesquisa-erro-b3-api = falha na API da B3
nucleo-pesquisa-b3-log-proventos =
    { $n ->
        [one] { $n } provento
       *[other] { $n } proventos
    }
nucleo-pesquisa-b3-log-proventos-eventos = { $proventos } provento(s), { $eventos } evento(s) em ações
nucleo-pesquisa-b3-passo-historico = B3 histórico de proventos { $codigo }
nucleo-pesquisa-b3-passo-resumo =
    { $pedidos ->
        [one] B3 empresas e fundos listados · proventos e eventos ({ $feitos } de { $total } emissores e fundos, { $pedidos } pedido)
       *[other] B3 empresas e fundos listados · proventos e eventos ({ $feitos } de { $total } emissores e fundos, { $pedidos } pedidos)
    }
nucleo-pesquisa-b3-parou = a B3 não respondeu { $vezes } vezes seguidas; o resto fica para a próxima coleta
nucleo-pesquisa-b3-nada-a-atualizar = nada a atualizar: todos consultados há pouco
nucleo-pesquisa-erro-brapi-invalida = resposta inválida da brapi: { $erro }
nucleo-pesquisa-erro-brapi-sem-resultados = brapi: resposta sem resultados

## Arquivos da CVM e dados do ativo

nucleo-pesquisa-erro-zip = arquivo zip inválido: { $erro }
nucleo-pesquisa-erro-coluna-ausente = coluna { $coluna } ausente
nucleo-pesquisa-erro-ipe-sem-csv = CSV do IPE não encontrado no zip
nucleo-pesquisa-liquidez-cotahist = média de { $n } pregão(ões), B3 COTAHIST
nucleo-pesquisa-liquidez-cotacoes = média de { $n } pregão(ões) (cotação × volume, { $fonte })
nucleo-pesquisa-erro-asset-ou-ticker = informe asset_id ou ticker
nucleo-pesquisa-erro-fora-do-universo = { $ticker } no universo baixado (rode a coleta da análise de ativos)

## Proventos: origem e conferência com a B3

nucleo-pesquisa-div-fonte-b3 = B3 (oficial), proventos em dinheiro das empresas e fundos listados
nucleo-pesquisa-div-fonte-agenda = agenda de proventos ({ $fontes })
nucleo-pesquisa-div-fonte-b3-diverge = { $b3 }; a agenda ({ $fontes }) diverge da B3
nucleo-pesquisa-div-fonte-lancados = proventos lançados (bruto ÷ quantidade)
nucleo-pesquisa-div-nota-ja-corrige = { $fonte } já corrige o valor pelos desdobramentos e bonificações depois da data-com
nucleo-pesquisa-div-nota-nao-corrige = { $fonte } não corrige o valor pelos desdobramentos e bonificações depois da data-com
nucleo-pesquisa-div-nota-houve-evento = houve desdobramento, grupamento ou bonificação depois da data-com
nucleo-pesquisa-div-nota-sem-par = { $tipos } sem par na B3
nucleo-pesquisa-div-nota-sem-b3 = Sem proventos da B3 para este papel (rode a coleta da análise de ativos ou confira se a B3 o conhece).
nucleo-pesquisa-div-nota-sem-agenda = Sem agenda de proventos do Yahoo ou do Fundamentus para comparar.
nucleo-pesquisa-div-nota-b3-desde = A B3 cobre desde { $data } (o histórico completo só é baixado para emissores de ativos cadastrados).
nucleo-pesquisa-div-nota-valores-epoca = A soma de 12 meses da agenda ({ $agenda }) bate com a da B3 com os valores da época ({ $b3 }); a diferença vem da correção pelos desdobramentos e bonificações, que o DY precisa (cotação de hoje).
nucleo-pesquisa-div-nota-divergencia = Divergência material: os indicadores usam a B3 (fonte oficial).
nucleo-pesquisa-erro-sem-codigo = o ativo não tem código de negociação
nucleo-pesquisa-erro-conferencia-br = a conferência com a B3 vale para ações e FIIs brasileiros

## Fundamentus

nucleo-pesquisa-erro-ticker-fundamentus = ticker inválido para o Fundamentus: { $ticker }

## IF.data do Banco Central

nucleo-pesquisa-ifdata-convencao = semestre até a data (março/setembro: 3 meses; junho/dezembro: 6 meses)
nucleo-pesquisa-ifdata-relatorio-cadastro = cadastro
nucleo-pesquisa-ifdata-relatorio-resumo = resumo
nucleo-pesquisa-ifdata-relatorio-ativo = ativo
nucleo-pesquisa-ifdata-relatorio-resultado = resultado
nucleo-pesquisa-ifdata-relatorio-capital = capital
nucleo-pesquisa-ifdata-relatorio-carteira = carteira por risco
nucleo-pesquisa-ifdata-relatorio = relatório
nucleo-pesquisa-erro-ifdata-invalida = resposta inválida do IF.data: { $erro }
nucleo-pesquisa-ifdata-fonte-desligada = a fonte Banco Central está desligada em Ajustes › Fontes de dados: ligue e a coleta diária baixa o IF.data
nucleo-pesquisa-ifdata-nao-baixado = o Bacen IF.data ainda não foi baixado (a coleta da análise de ativos roda sozinha uma vez por dia; o resultado de cada fonte fica em Ativos › Dados da análise)
nucleo-pesquisa-ifdata-fora-do-cadastro = o CNPJ { $cnpj } não aparece no cadastro do Bacen IF.data (trimestre { $trimestre }): a companhia não é um banco que reporta ao Banco Central
nucleo-pesquisa-ifdata-sem-valores = o Bacen IF.data não trouxe valores do { $banco } ({ $codigo }) até o trimestre { $trimestre }

## Rede

nucleo-pesquisa-erro-post = POST não suportado
nucleo-pesquisa-erro-servidor-recusou =
    { $tentativas ->
        [one] o servidor recusou o acesso (HTTP { $status }) depois de { $tentativas } tentativa; tente mais tarde
       *[other] o servidor recusou o acesso (HTTP { $status }) depois de { $tentativas } tentativas; tente mais tarde
    }
nucleo-pesquisa-erro-token-recusado = acesso recusado (HTTP { $status }): confira o token
nucleo-pesquisa-erro-http-tentativas =
    { $tentativas ->
        [one] HTTP { $status } depois de { $tentativas } tentativa
       *[other] HTTP { $status } depois de { $tentativas } tentativas
    }

## Documentos dos últimos 12 meses

nucleo-pesquisa-doc-exercicio = exercício: { $documento }
nucleo-pesquisa-doc-ltm = { $exercicio } + acumulado até { $ate } − acumulado até { $ate_anterior } ({ $documento }; { $documento_anterior })
nucleo-pesquisa-doc-ultimo-exercicio = último exercício: { $documento }

## Radar de proventos

nucleo-pesquisa-radar-poucos-proventos = histórico insuficiente (menos de 2 proventos em 24 meses)
nucleo-pesquisa-radar-insuficiente = histórico insuficiente

## Rankings e screener

nucleo-pesquisa-erro-indicador-desconhecido = indicador desconhecido: { $indicador } (veja research.catalog)
nucleo-pesquisa-erro-ordenar-desconhecido = indicador desconhecido para ordenar: { $indicador }
nucleo-pesquisa-erro-campo-desconhecido = campo desconhecido no filtro: { $campo } (indicadores de research.catalog, ou { $campos })
nucleo-pesquisa-erro-filtro = filtro { $campo }: { $motivo }
nucleo-pesquisa-filtro-comparacao = comparação de tamanho só vale para indicadores
nucleo-pesquisa-filtro-numero = informe um número em `value`
nucleo-pesquisa-filtro-between = `between` precisa de `value: [mínimo, máximo]` num indicador
nucleo-pesquisa-filtro-in = `in`/`not_in` precisam de uma lista em `value`
nucleo-pesquisa-filtro-contains = `contains` precisa de um texto num campo de texto
nucleo-pesquisa-filtro-value = informe `value`
nucleo-pesquisa-erro-grupo-vazio = grupo de filtros vazio
nucleo-pesquisa-filtro-op-entre = entre
nucleo-pesquisa-filtro-op-em = em
nucleo-pesquisa-filtro-op-fora-de = fora de
nucleo-pesquisa-filtro-op-contem = contém
nucleo-pesquisa-filtro-op-disponivel = disponível
nucleo-pesquisa-filtro-op-indisponivel = indisponível
nucleo-pesquisa-filtro-e = E
nucleo-pesquisa-filtro-ou = OU
nucleo-pesquisa-filtro-todos = todos os papéis
nucleo-pesquisa-ref-tela = tela { $id }
nucleo-pesquisa-erro-tela-nome = informe o nome da tela
nucleo-pesquisa-erro-tela-duplicada = já existe uma tela chamada { $nome }

## SEC e banco local

nucleo-pesquisa-erro-sec-cadastro = cadastro da SEC inválido: { $erro }
nucleo-pesquisa-erro-sec-tickers = company_tickers.json inválido: { $erro }
nucleo-pesquisa-erro-sec-companyfacts = companyfacts inválido: { $erro }
nucleo-pesquisa-doc-sec = { $formulario } { $numero } (arquivado em { $data })
nucleo-pesquisa-erro-valor-banco = valor inválido: { $valor }
nucleo-pesquisa-erro-cotacao-banco = cotação inválida no banco: { $cotacao }

## Coleta da análise: passos e avisos

nucleo-pesquisa-passo-fii-mensal = CVM informe mensal de FII { $ano }
nucleo-pesquisa-passo-fii-trimestral = CVM informe trimestral de FII { $ano }
nucleo-pesquisa-passo-sec-email = SEC EDGAR: informe o e-mail de contato
nucleo-pesquisa-passo-sec-cadastro = SEC cadastro { $ticker }
nucleo-pesquisa-erro-sec-sem-ticker = { $ticker } não está na lista de tickers da SEC
nucleo-pesquisa-passo-ifdata-sondagem = Banco Central IF.data: sondagem de { $periodo }
nucleo-pesquisa-passo-parte = parte { $parte }/{ $partes }
nucleo-pesquisa-aviso-acoes-em-milhares =
    { $n ->
        [one] { $n } companhia informou a quantidade de ações em milhares em algum documento (corrigida pelo lucro por ação): { $companhias }
       *[other] { $n } companhias informaram a quantidade de ações em milhares em algum documento (corrigida pelo lucro por ação): { $companhias }
    }
nucleo-pesquisa-aviso-lucro-controladores =
    { $n ->
        [one] { $n } companhia deixou zerado o lucro dos controladores no consolidado em algum documento (corrigido pela demonstração individual): { $companhias }
       *[other] { $n } companhias deixaram zerado o lucro dos controladores no consolidado em algum documento (corrigido pela demonstração individual): { $companhias }
    }
nucleo-pesquisa-feedback-faixa-titulo = Indicadores fora da faixa plausível no universo
nucleo-pesquisa-feedback-faixa-corpo = { $n } valor(es) de indicador fora das faixas largas de plausibilidade (docs/indicators.md, E11). Não foram bloqueados, porque podem ser verdade; vale conferir o dado de entrada de cada um:
    { $lista }
nucleo-pesquisa-feedback-faixa-resolvido = Na coleta de hoje, nenhum indicador ficou fora das faixas.

## Coleta: nome do IF.data

nucleo-pesquisa-passo-ifdata = Banco Central IF.data

## brapi.dev: plano e cota

nucleo-pesquisa-brapi-plano-gratuito = brapi.dev: a conferência de indicadores e proventos é dos planos pagos; no plano gratuito ela fica desligada, sem gastar a cota.
nucleo-pesquisa-brapi-limite = brapi.dev: limite de uso do mês atingido; o resto fica para depois da renovação ou de um limite maior em Ajustes › Fontes de dados.

nucleo-pesquisa-erro-rede-bloqueada = a rede deste computador bloqueou a conexão segura com { $site } (filtro de segurança da empresa, antivírus, proxy ou DNS que bloqueia o site); não é o site nem o Treasury. Libere o site no filtro ou use outra rede. Detalhe: { $detalhe }

# Indicadores dos métodos do explorador de mercado
nucleo-pesquisa-ind-earnings_yield = EBIT/EV
nucleo-pesquisa-ind-earnings_yield-formula = EBIT dos últimos 12 meses ÷ valor da firma (o inverso do EV/EBIT; Magic Formula)
nucleo-pesquisa-ind-magic_roc = ROC (Magic Formula)
nucleo-pesquisa-ind-magic_roc-formula = EBIT dos últimos 12 meses ÷ (capital de giro líquido + imobilizado)
nucleo-pesquisa-ind-fcf_yield = Fluxo de caixa livre / valor de mercado
nucleo-pesquisa-ind-fcf_yield-formula = (Caixa das operações − investimentos em imobilizado, 12 meses) ÷ valor de mercado
nucleo-pesquisa-ind-peg = PEG
nucleo-pesquisa-ind-peg-formula = P/L ÷ crescimento anual do lucro em 5 anos (em pontos percentuais; Peter Lynch)
nucleo-pesquisa-ind-eps_growth_10y = Crescimento do lucro em 10 anos
nucleo-pesquisa-ind-eps_growth_10y-formula = Lucro médio dos 3 últimos exercícios ÷ lucro médio de 10 a 8 exercícios atrás − 1 (Graham defensivo; lucro total, não por ação)
nucleo-pesquisa-ind-dividend_years = Anos seguidos com proventos
nucleo-pesquisa-ind-dividend_years-formula = Anos completos seguidos, até o último, com algum provento pago (até 20)
nucleo-pesquisa-ind-dividend_cagr_5y = Crescimento dos proventos em 5 anos
nucleo-pesquisa-ind-dividend_cagr_5y-formula = Crescimento anual composto dos proventos por ação ou cota entre o ano de 5 anos atrás e o último ano completo
nucleo-pesquisa-ind-sem-imobilizado = Sem o imobilizado no último balanço
nucleo-pesquisa-ind-sem-fco-12m = Sem o caixa das operações dos últimos 12 meses
nucleo-pesquisa-ind-fonte-peg = P/L e crescimento do lucro em 5 anos
nucleo-pesquisa-ind-peg-negativo = P/L ou crescimento do lucro negativo: o PEG não se aplica
nucleo-pesquisa-ind-peg-sem-dados = Sem P/L ou sem crescimento do lucro em 5 anos

# Score em lote do universo
nucleo-pesquisa-ind-score = Score
nucleo-pesquisa-ind-score-formula = Score de saúde do Treasury (0 a 100), calculado na coleta para todo o universo
nucleo-pesquisa-ind-health = Saúde (score)
nucleo-pesquisa-ind-health-formula = Sub-score Saúde do negócio, sem valuation nem momentum (0 a 100)
nucleo-pesquisa-ind-attractiveness = Atratividade (score)
nucleo-pesquisa-ind-attractiveness-formula = Sub-score Atratividade do preço: valuation e momentum (0 a 100)
nucleo-pesquisa-ind-confidence = Confiança do score
nucleo-pesquisa-ind-confidence-formula = Cobertura dos dados × frescor × tamanho do grupo de pares (0 a 1)
nucleo-pesquisa-ind-piotroski = Piotroski F-Score
nucleo-pesquisa-ind-piotroski-formula = Critérios de Piotroski cumpridos (0 a 9)
nucleo-pesquisa-ind-altman_z = Altman Z''
nucleo-pesquisa-ind-altman_z-formula = Z'' de Altman para mercados emergentes (acima de 2,6: zona segura)
nucleo-pesquisa-ind-beneish = Beneish M-Score
nucleo-pesquisa-ind-beneish-formula = M-Score de Beneish (acima de −1,78 sugere risco de manipulação contábil)
nucleo-pesquisa-ind-momentum_12_1 = Momentum 12-1
nucleo-pesquisa-ind-momentum_12_1-formula = Retorno dos últimos 12 meses sem o último mês
nucleo-pesquisa-ind-volatility_1y = Volatilidade 1 ano
nucleo-pesquisa-ind-volatility_1y-formula = Desvio padrão anualizado dos retornos diários de 1 ano
nucleo-pesquisa-aviso-score-lote = Score do universo não calculado: { $erro }
nucleo-pesquisa-motivo-observar = Observar { $ticker } pelo Mercado
nucleo-pesquisa-motivo-deixar-observar = Deixar de observar pelo Mercado
nucleo-pesquisa-aviso-alerta-tela = Alertas de tela não conferidos: { $erro }
nucleo-pesquisa-documento-link = O documento não tem um link oficial da CVM para baixar.
nucleo-pesquisa-documento-sumiu = A CVM não encontrou o arquivo do documento.
nucleo-pesquisa-documento-grande = O arquivo do documento é grande demais para ler (máx. 25 MB).
nucleo-pesquisa-documento-sem-texto = Não deu para ler texto no documento.
nucleo-pesquisa-documento-nao-encontrado = Documento não encontrado entre os capturados (use list_corporate_documents).
nucleo-pesquisa-ind-periodo-trimestral = informe trimestral até { $data }
nucleo-pesquisa-div-fonte-b3-sem-rendimentos = B3 (oficial): nenhum rendimento em dinheiro do fundo nos últimos 12 meses
nucleo-pesquisa-ind-sem-rendimentos-b3 = a B3 não mostra rendimento do fundo nos últimos 12 meses
nucleo-pesquisa-ind-fonte-declarados-fii = rendimentos declarados nos informes trimestrais da CVM (4 trimestres até { $data })
nucleo-pesquisa-ind-formula-dy-fii-declarados = Rendimentos declarados nos 4 últimos informes trimestrais ÷ cotas emitidas ÷ cotação (estimativa: sem os proventos da B3)
nucleo-pesquisa-ind-formula-dy-fii-meses = Soma do rendimento por cota dos informes mensais (DY do mês × VP por cota) ÷ cotação (estimativa: sem os proventos da B3). Meses: { $meses }
nucleo-pesquisa-div-fonte-b3-amortizacao = { $fonte }; rendimentos de antes das amortizações de { $datas } na escala da cota de hoje (sem o capital devolvido)
