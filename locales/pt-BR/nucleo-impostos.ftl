# Impostos (núcleo, `tax` e `tax_burden`): a apuração do IR sobre a renda variável e os DARFs
# (Impostos › IR, só do Brasil) e a estimativa de impostos pagos (Impostos): erros,
# motivos do histórico, partes, grupos, notas e os textos dos dados de cada país
# (`tax_burden/data/*.json`). Erros em minúsculas, como frase de log (a interface põe a inicial
# maiúscula). pt-BR é o idioma de origem.
#
# Ficam em pt-BR em qualquer idioma: a discriminação, o grupo e o país de Bens e Direitos (vão
# para o programa da Receita Federal), as fontes citadas, os estados e as bases legais.

## IR: categorias, notas e aviso da apuração (`tax`)

nucleo-impostos-ir-categoria-acoes = Ações (operações comuns)
nucleo-impostos-ir-categoria-etf = ETFs (operações comuns)
nucleo-impostos-ir-categoria-bdr = BDRs (operações comuns)
nucleo-impostos-ir-categoria-fii = FIIs
nucleo-impostos-ir-categoria-day-trade = Day trade
nucleo-impostos-ir-nota-isencao = Vendas de ações no mês de { $valor } (até R$ 20 mil): ganho isento; prejuízo continua compensável.
nucleo-impostos-ir-nota-irrf-estimado = IRRF estimado (0,005% das vendas; 1% do ganho em day trade): confira nas notas de corretagem.
nucleo-impostos-ir-nota-venda-maior = Venda de { $ativo } em { $data } maior que a posição registrada ({ $quantidade } a mais): o excedente ficou fora da apuração. Falta registrar compras antigas?
nucleo-impostos-ir-aviso = Apuração auxiliar calculada com os seus dados; confira no programa da Receita Federal. Não é aconselhamento tributário.
nucleo-impostos-ir-erro-ano = ano inválido

## IR: DARF

nucleo-impostos-ir-erro-gerar-darf = gerar DARF exige aprovação do usuário
nucleo-impostos-ir-erro-abaixo-minimo = imposto abaixo de R$ 10,00: acumula para o mês seguinte
nucleo-impostos-ir-erro-sem-imposto = não há imposto a pagar neste mês
nucleo-impostos-ir-erro-darf-existe = já existe um DARF para este mês; cancele o anterior para gerar outro
nucleo-impostos-ir-erro-pagar-interface = registrar pagamento de DARF só pela interface
nucleo-impostos-ir-erro-darf-pago = DARF pago não pode ser cancelado
# Motivos do histórico; o mês fica como AAAA-MM.
nucleo-impostos-ir-motivo-darf = DARF { $codigo } de { $mes }
nucleo-impostos-ir-motivo-pagamento = Pagamento do DARF de { $mes }
nucleo-impostos-ir-motivo-cancelado = DARF de { $mes } cancelado
nucleo-impostos-ir-motivo-custodia-cripto = Onde fica a cripto da conta (imposto)
# Descrição do lançamento do pagamento.
nucleo-impostos-ir-lancamento-darf = DARF { $codigo } — IR renda variável { $mes }

## Impostos pagos: ajustes, categorias e informes de rendimentos (`tax_burden`)

nucleo-impostos-erro-ajustes-interface = país e estado da estimativa só pela interface do app
nucleo-impostos-erro-dependentes = dependentes demais
nucleo-impostos-erro-brasil = os dados de impostos do Brasil só valem para famílias no Brasil
nucleo-impostos-cidade-iptu = IPTU
nucleo-impostos-cidade-iss = ISS
nucleo-impostos-cidade-itbi = ITBI
nucleo-impostos-cidade-irrf = Imposto de renda retido dos servidores
nucleo-impostos-cidade-outros = Taxas e contribuições de melhoria
nucleo-impostos-cidade-erro-codigo = cidade inválida: escolha o município na lista
nucleo-impostos-cidade-erro-uf = estado (UF) inválido
nucleo-impostos-cidade-erro-consulta = a consulta falhou: { $erro }
nucleo-impostos-cidade-erro-resposta = a fonte respondeu num formato que o app não reconhece
nucleo-impostos-cidade-erro-sem-cidade = escolha a cidade da família primeiro
nucleo-impostos-cidade-erro-sem-relatorio = { $cidade } não tem relatório de receita no Tesouro Nacional nos últimos seis bimestres
nucleo-impostos-erro-estado = estado { $estado } sem dados em { $dados }
nucleo-impostos-dados-de = dados de impostos de { $pais }
nucleo-impostos-categoria-despesa = categoria de despesa { $id }
nucleo-impostos-erro-grupo = grupo de imposto desconhecido: { $grupo }
nucleo-impostos-erro-aliquota = a alíquota própria fica entre 0% e 100% do preço
nucleo-impostos-erro-fonte-pagadora = informe a fonte pagadora
nucleo-impostos-erro-ano-informe = ano do informe inválido
nucleo-impostos-erro-informe-negativo = valores do informe não podem ser negativos
nucleo-impostos-informe = informe { $id }
nucleo-impostos-motivo-grupo = Grupo de imposto da categoria alterado
nucleo-impostos-motivo-informe-alterado = Informe de rendimentos alterado: { $fonte }
nucleo-impostos-motivo-informe-lancado = Informe de rendimentos lançado: { $fonte }
nucleo-impostos-motivo-informe-removido = Informe de rendimentos removido

## Impostos pagos: a estimativa

nucleo-impostos-erro-ano-futuro = o ano ainda não começou
nucleo-impostos-parte-consumo = Consumo (embutido nos preços)
nucleo-impostos-parte-renda = Imposto de renda
nucleo-impostos-parte-previdencia = Previdência (INSS)
nucleo-impostos-parte-patrimonio = Patrimônio (IPTU, IPVA)
nucleo-impostos-parte-proventos = IR retido em proventos
nucleo-impostos-parte-outros = IOF e outros tributos
nucleo-impostos-sem-categoria = Sem categoria
nucleo-impostos-cupons = Cupons fiscais lidos
nucleo-impostos-grupo-cupons = Imposto das notas (NFC-e)
nucleo-impostos-nota-escolha-uf = Escolha o estado (UF): sem ele, o IPVA não é estimado.
nucleo-impostos-nota-escolha-estado = Escolha o estado: sem ele, o imposto sobre vendas não é estimado.
nucleo-impostos-nota-escolha-pais = Escolha o país: sem ele, o IVA não é estimado.
# parte: porcentagem inteira (sem casas decimais).
nucleo-impostos-nota-cupons = { $parte }% do consumo veio de cupons fiscais lidos: nessas compras vale o imposto da nota (o impresso, ou o estimado item a item pelo percentual do grupo de consumo).
nucleo-impostos-nota-sem-grupo =
    { $n ->
        [one] { $n } categoria sem grupo de imposto ficou de fora: escolha o grupo dela.
       *[other] { $n } categorias sem grupo de imposto ficaram de fora: escolha o grupo delas.
    }
nucleo-impostos-nota-sem-iptu = Há imóvel cadastrado e nenhum IPTU lançado no ano: o IPTU não entra na estimativa.
nucleo-impostos-nota-sem-regras = Os dados de { $dados } não têm regras de renda: o imposto sobre o salário não é estimado (lance os informes de rendimentos).
nucleo-impostos-nota-salario-estimado = Imposto e INSS do salário estimados a partir do líquido recebido, sem descontos como plano de saúde ou pensão: com o informe de rendimentos, a estimativa usa os valores dele.
nucleo-impostos-nota-sem-ipva = Há veículo cadastrado e nenhum IPVA lançado: escolha o estado para estimar o IPVA.
nucleo-impostos-nota-ipva-estimado = IPVA estimado pelo valor de cada veículo no fim do período e a alíquota do estado: lance o IPVA pago para usar o valor real.

## Dados de cada país (`tax_burden/data/*.json`): nome, aviso, metodologia, contexto e grupos
## de consumo. A tradução só aparece enquanto o texto daqui for igual ao do arquivo: ao
## atualizar os dados, atualize também estas linhas (e as de en e es); o teste
## `dataset_texts_are_in_the_catalog` aponta o que mudou.

nucleo-impostos-dados-br-nome = Brasil
nucleo-impostos-dados-br-aviso = Esta é uma estimativa baseada em médias nacionais de imposto por categoria de consumo e nas regras de imposto de renda vigentes. Os valores reais variam conforme o produto, o estado, o regime tributário das empresas e benefícios fiscais específicos. Fontes: IBGE (Contas Nacionais), Receita Federal, Previdência Social, Tesouro Nacional e Petrobras.
nucleo-impostos-dados-br-metodo-1 = Consumo: o gasto de cada categoria vezes a fração do preço que é imposto sobre o produto. Os percentuais vêm das Contas Nacionais do IBGE (Tabelas de Recursos e Usos de 2021, o último ano publicado com 128 produtos): impostos sobre o produto (imposto de importação, IPI, ICMS e outros impostos menos subsídios) divididos pela oferta a preço de consumidor. Para gasolina, diesel e gás de cozinha vale a composição de preço publicada pela Petrobras; para seguros e tarifas bancárias, a alíquota da lei.
nucleo-impostos-dados-br-metodo-2 = Cenários: os grupos calculados pelas Contas Nacionais têm o mesmo valor nos três cenários. Quando um grupo junta vários produtos, o valor é a média ponderada pelo consumo das famílias. Os cenários só diferem onde a fonte dá uma faixa (combustíveis, seguros e tarifas bancárias).
nucleo-impostos-dados-br-metodo-3 = Estado: os percentuais de consumo são médias nacionais e não mudam com o estado escolhido. O estado entra no IPVA.
nucleo-impostos-dados-br-metodo-4 = Renda do trabalho: com o informe de rendimentos, valem os valores dele. Sem informe, o bruto de cada mês é estimado a partir do salário líquido recebido, pelas tabelas do IRPF e do INSS vigentes no mês (sem dependentes, salvo ajuste, e sem outros descontos); o 13º é tributado à parte (Lei 8.134/1990, art. 16).
nucleo-impostos-dados-br-metodo-5 = Impostos pagos direto (IR, IOF, IPTU, IPVA e outros tributos lançados nas categorias deles) entram pelo valor lançado. O IPVA é estimado pelo valor de cada veículo e a alíquota do estado (automóvel flex típico; motos pela alíquota geral) só quando nenhum IPVA foi lançado no ano. Veículos com idade de fabricação na isenção do estado, ou com 20 anos ou mais a partir de 2026 (EC 137/2025), não pagam.
nucleo-impostos-dados-br-metodo-6 = IR retido em proventos (JCP, dividendos do exterior) vem dos registros da carteira.
nucleo-impostos-dados-br-metodo-7 = O que o número do IBGE mede: só os impostos cobrados sobre o produto. Impostos pagos antes na cadeia e não recuperados, encargos sobre a folha e o imposto de renda das empresas não entram. Por isso os percentuais são menores que os de estimativas que somam esses tributos, e o resultado deve ser lido como um piso.
nucleo-impostos-dados-br-metodo-8 = Não entram: encargos pagos pelo empregador (INSS patronal, FGTS), grupos sem dado oficial (plano de saúde, serviços domésticos, gás veicular, motocicletas, compras sem detalhe) e o teste da reforma tributária de 2026 (CBS 0,9% e IBS 0,1%, compensados com PIS/Cofins).
nucleo-impostos-dados-br-metodo-9 = Atualização: as Contas Nacionais com esse detalhe saem com cerca de dois anos de atraso. Mudanças de alíquota depois de 2021, como o teto do ICMS de energia e telecomunicações de 2022, só aparecem quando o IBGE publicar o ano correspondente.
nucleo-impostos-dados-br-contexto = Carga tributária bruta do Brasil (% do PIB)
nucleo-impostos-dados-br-dias = Carga tributária bruta do Brasil em dias do ano (Tesouro Nacional)
nucleo-impostos-dados-br-destino = Despesa do governo geral por função (União, estados e municípios)
nucleo-impostos-funcao-servicos-publicos-gerais = Serviços públicos gerais (inclui juros da dívida)
nucleo-impostos-funcao-defesa = Defesa
nucleo-impostos-funcao-ordem-publica-seguranca = Ordem pública e segurança
nucleo-impostos-funcao-assuntos-economicos = Assuntos econômicos
nucleo-impostos-funcao-protecao-ambiental = Proteção ambiental
nucleo-impostos-funcao-habitacao-servicos-comunitarios = Habitação e serviços comunitários
nucleo-impostos-funcao-saude = Saúde
nucleo-impostos-funcao-lazer-cultura-religiao = Lazer, cultura e religião
nucleo-impostos-funcao-educacao = Educação
nucleo-impostos-funcao-protecao-social = Proteção social (previdência e assistência)

nucleo-impostos-dados-us-nome = Estados Unidos
nucleo-impostos-dados-us-aviso = Esboço de estimativa: considera só o imposto sobre vendas médio do estado e os impostos lançados ou informados. Os valores reais variam por cidade, produto e situação de cada pessoa. Fonte: Tax Foundation.
nucleo-impostos-dados-us-metodo-1 = Esboço: só o imposto estadual e local sobre vendas, pela alíquota combinada média do estado (Tax Foundation, meados de 2026), como fração do valor pago: alíquota / (100 + alíquota).
nucleo-impostos-dados-us-metodo-2 = Renda: o imposto federal é estimado mês a mês pelas faixas de 2026 de quem declara sozinho (single), com a dedução padrão e o crédito por filho divididos por 12; o FICA é Social Security 6,2% até o teto anual (US$ 184.500, dividido por 12) e Medicare 1,45% (mais 0,9% acima de US$ 200 mil por ano). Imposto estadual e local sobre a renda não entram; com o W-2, lance o informe de rendimentos para usar os valores reais.

nucleo-impostos-dados-eu-nome = União Europeia
nucleo-impostos-dados-eu-aviso = Esboço de estimativa: considera só o IVA do país escolhido e os impostos lançados ou informados. Alíquotas reduzidas variam por produto em cada país. Fonte: Comissão Europeia.
nucleo-impostos-dados-eu-metodo-1 = Esboço: só o IVA, pelas alíquotas padrão, reduzida (a menor) e super-reduzida do país escolhido (tabela "Your Europe" da Comissão Europeia, conferida em 13/07/2026).
nucleo-impostos-dados-eu-metodo-2 = Renda (só Portugal): o IRS é estimado pelo imposto anual dos escalões de 2026 do continente, dividido pelos 14 pagamentos do ano (12 salários e os subsídios de férias e de Natal), com a dedução específica (o maior entre 4 587,09 € e a Segurança Social), 600 € por dependente e o mínimo de existência; a Segurança Social é 11% do bruto. Não são as tabelas mensais de retenção (que retêm um pouco diferente e se acertam na declaração). Os outros países da União Europeia ainda não têm regras de renda: lance os informes de rendimentos.

# Grupos de consumo (os mesmos nos três países).
nucleo-impostos-grupo-alimentos-basicos = Alimentos básicos
nucleo-impostos-grupo-alimentos-industrializados = Alimentos industrializados
nucleo-impostos-grupo-bebidas-nao-alcoolicas = Bebidas não alcoólicas
nucleo-impostos-grupo-bebidas-alcoolicas = Bebidas alcoólicas
nucleo-impostos-grupo-restaurantes = Restaurantes, bares e delivery
nucleo-impostos-grupo-gasolina = Gasolina comum (gasolina C)
nucleo-impostos-grupo-diesel = Óleo diesel (diesel B)
nucleo-impostos-grupo-etanol = Etanol hidratado
nucleo-impostos-grupo-gnv = Gás natural veicular (GNV)
nucleo-impostos-grupo-glp = Gás de cozinha (GLP, botijão 13 kg)
nucleo-impostos-grupo-energia-eletrica = Energia elétrica (conta de luz residencial)
nucleo-impostos-grupo-agua-esgoto = Água e esgoto
nucleo-impostos-grupo-telefonia-internet = Telefonia e internet
nucleo-impostos-grupo-tv-assinatura-streaming = TV por assinatura e streaming
nucleo-impostos-grupo-eletronicos-nacionais = Eletrônicos (fabricação nacional ou sem distinção)
nucleo-impostos-grupo-eletronicos-importados = Eletrônicos importados
nucleo-impostos-grupo-eletrodomesticos = Eletrodomésticos
nucleo-impostos-grupo-moveis = Móveis e colchões
nucleo-impostos-grupo-vestuario = Vestuário
nucleo-impostos-grupo-calcados = Calçados
nucleo-impostos-grupo-medicamentos = Medicamentos de uso humano
nucleo-impostos-grupo-higiene-pessoal = Higiene pessoal
nucleo-impostos-grupo-cosmeticos-perfumes = Cosméticos e perfumes
nucleo-impostos-grupo-servicos-pessoais = Serviços pessoais (salão, academia/clube)
nucleo-impostos-grupo-educacao-mensalidade = Educação – mensalidade escolar
nucleo-impostos-grupo-educacao-livros = Educação – livros
nucleo-impostos-grupo-saude-plano-consulta = Saúde – plano de saúde e consultas
nucleo-impostos-grupo-transporte-publico = Transporte público
nucleo-impostos-grupo-transporte-aplicativo = Transporte por aplicativo
nucleo-impostos-grupo-passagens-aereas = Passagens aéreas
nucleo-impostos-grupo-hotel-turismo = Hotel e turismo
nucleo-impostos-grupo-automovel-novo = Automóvel novo
nucleo-impostos-grupo-motocicleta = Motocicleta
nucleo-impostos-grupo-pecas-manutencao-veiculo = Peças e manutenção de veículo
nucleo-impostos-grupo-seguros = Seguros (IOF sobre o prêmio)
nucleo-impostos-grupo-tarifas-bancarias = Tarifas bancárias
nucleo-impostos-grupo-brinquedos = Brinquedos
nucleo-impostos-grupo-pet = Pet (ração, remédio, veterinário)
nucleo-impostos-grupo-materiais-construcao = Materiais de construção
nucleo-impostos-grupo-cigarros = Cigarros
nucleo-impostos-grupo-mercado = Mercado (cesta mista)
nucleo-impostos-grupo-aluguel = Aluguel residencial
nucleo-impostos-grupo-condominio = Condomínio
nucleo-impostos-grupo-servicos-domesticos = Serviços domésticos (diarista, babá)
nucleo-impostos-grupo-estacionamento = Estacionamento
nucleo-impostos-grupo-optica = Óculos e lentes
nucleo-impostos-grupo-material-escolar = Material escolar
nucleo-impostos-grupo-lazer = Lazer (passeios, ingressos, festas)
nucleo-impostos-grupo-presentes = Presentes (itens variados)
nucleo-impostos-grupo-compras-diversas = Compras diversas (sem detalhe)
nucleo-impostos-grupo-contas-diversas = Contas e boletos (sem detalhe)
nucleo-impostos-grupo-limpeza = Material de limpeza
nucleo-impostos-anual-nota-sem-holerite = Nenhum holerite do ano no cofre: informe os rendimentos do informe de rendimentos da empresa.
nucleo-impostos-anual-nota-holerites = { $fonte }: holerites de { $meses } meses, { $valor } tributáveis (sem 13º e PLR, que são de tributação exclusiva). Confira com o informe de rendimentos.
nucleo-impostos-anual-nota-65 = Aposentadoria a partir dos 65 anos: { $valor } ficaram de fora como parcela isenta (R$ 1.903,98 por mês).
nucleo-impostos-anual-nota-despesas = Despesas dedutíveis pelas categorias: saúde { $saude } (plano, consultas, exames, dentista, terapias) e instrução { $instrucao } (escola, creche, faculdade). Farmácia, óculos, material escolar, idiomas e cursos livres não entram.
nucleo-impostos-anual-nota-pgbl = Aportes em PGBL e fundo de pensão: { $valor } (dedutíveis até 12% dos rendimentos tributáveis).
nucleo-impostos-anual-nota-dedo-duro = IRRF "dedo-duro" da bolsa que sobrou no ano: { $valor }, entra como imposto pago.
nucleo-impostos-anual-nota-familia = Família toda: a declaração é por pessoa. Escolha um membro para a prévia de cada declaração.
nucleo-impostos-anual-nota-confira = Prévia para conferir: ajuste os valores com os informes e recibos. A declaração oficial é feita no programa da Receita.
nucleo-impostos-anual-nota-irpfm = Rendimentos acima de R$ 600 mil no ano: entra o imposto mínimo das altas rendas (IRPFM, Lei 15.270/2025). A estimativa não aplica o redutor da carga das empresas (art. 16-B), que pode diminuir ou zerar o valor.
nucleo-impostos-gcap-erro-venda = Informe a data e o valor da venda para calcular o ganho de capital.
