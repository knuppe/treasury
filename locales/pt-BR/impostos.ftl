# Tela Impostos: estimativa dos impostos pagos no ano, cenários, consumo por categoria,
# impostos lançados, renda do trabalho, informes de rendimentos, patrimônio e fontes
# (docs/taxes.md, docs/i18n-design.md). pt-BR é o idioma de origem.

## Tela

impostos-periodo = { $de } a { $ate }
impostos-descricao = Quanto do ano foi imposto, estimado a partir dos seus dados: consumo, renda do trabalho, patrimônio e impostos lançados. Cada número mostra de onde veio.
impostos-estimando = Estimando os impostos do ano
impostos-aviso-titulo = Estimativa, não valor oficial
impostos-desatualizados = Dados desatualizados
impostos-desatualizados-texto = Os números de { $dados } foram revisados pela última vez em { $data } (versão { $versao }), há mais de 6 meses. Alíquotas e tabelas podem ter mudado.
impostos-esboco-titulo = { $dados }: esboço
impostos-esboco-texto = Os dados deste país ainda são parciais: só parte dos impostos é estimada.
impostos-aviso-padrao = Esta é uma estimativa baseada em médias de carga tributária por categoria de consumo e regras de imposto de renda vigentes. Os valores reais podem variar conforme o estado, o regime tributário das empresas e benefícios fiscais específicos.

## Cenários

impostos-cenario = Cenário
impostos-cenario-estimativa = Cenário da estimativa
impostos-cenario-conservador = Conservador
impostos-cenario-conservador-dica = O menor valor razoável de cada grupo: o produto com menor carga ou a alíquota reduzida.
impostos-cenario-provavel = Provável
impostos-cenario-provavel-dica = O valor central de cada grupo: a mediana dos produtos ou o dado oficial mais recente.
impostos-cenario-elevado = Elevado
impostos-cenario-elevado-dica = O maior valor razoável de cada grupo: o produto com maior carga ou a alíquota padrão.
impostos-tres-cenarios = Três cenários
impostos-cenarios = Cenários
impostos-cenarios-nota = Os cenários mudam só o imposto de consumo; renda, INSS, impostos lançados e IPVA são iguais nos três.

## Partes da estimativa e grupos de imposto

impostos-parte-consumo = Consumo
impostos-parte-ir = Imposto de renda
impostos-parte-previdencia = Previdência (INSS)
impostos-parte-patrimonio = Patrimônio
impostos-parte-proventos = Proventos
impostos-parte-outros = Outros tributos
impostos-grupo-ir-pago = Imposto de renda pago
impostos-grupo-iof-pago = IOF pago
impostos-grupo-iptu = IPTU
impostos-grupo-ipva = IPVA e licenciamento
impostos-grupo-outros-pagos = Outros tributos pagos
impostos-grupo-financeiro = Financeiro (dívidas, juros, investimentos)
impostos-grupo-transferencias = Transferências a pessoas (pensão, mesada, doações)
impostos-sem-grupo = Sem grupo
impostos-origem-usuario = sua escolha
impostos-origem-padrao = padrão
impostos-origem-pai = do grupo pai
impostos-origem-cupons = cupons fiscais

## Fora da estimativa: motivos

impostos-motivo-sem-categoria = Sem categoria
impostos-motivo-sem-categoria-dica = Lançamentos sem categoria não têm grupo de imposto. Categorize-os em Lançamentos para que entrem.
impostos-motivo-sem-grupo = Sem grupo de imposto
impostos-motivo-sem-grupo-dica = Categorias criadas por você que não estão no conjunto inicial e não herdam grupo do pai. Escolha o grupo de cada uma.
impostos-motivo-fora-consumo = Fora do consumo
impostos-motivo-fora-consumo-dica = Dívidas, juros, investimentos e transferências a pessoas não são compra de produto ou serviço: não têm imposto embutido a estimar.
impostos-motivo-sem-dado = Sem dado confiável
impostos-motivo-sem-dado-dica = Não há fonte confiável da carga embutida nesses gastos (aluguel, plano de saúde, serviços sem detalhe…). Eles ficam à vista, mas fora da estimativa; você pode fixar uma alíquota própria.

## Alíquotas

impostos-pontos = { $valor } p.p.
impostos-sem-dado = sem dado
impostos-base-isento = isento
impostos-base-iva-padrao = IVA padrão do país
impostos-base-iva-reduzido = IVA reduzido do país
impostos-base-iva-super-reduzido = IVA super-reduzido do país
impostos-base-vendas = imposto sobre vendas do estado
impostos-regiao-uf = Estado (UF)
impostos-regiao-pais-membro = País-membro
impostos-regiao-estado = Estado

## Gráfico e composição

impostos-grafico-inclui-estimados = Inclui valores estimados
impostos-grafico-lancados = Valores lançados ou informados
impostos-imposto-estimado = Imposto estimado
impostos-composicao = Composição por tipo
impostos-composicao-vazia = Nenhum imposto estimado no período.
impostos-por-tipo = Imposto por tipo · { $cenario }
impostos-por-tipo-aria = Imposto estimado por tipo, cenário { $cenario }
impostos-composicao-fonte = Fonte: estimativa do app com os dados do país · ≈ inclui valores estimados
impostos-tipo = Tipo
impostos-valor = Valor
impostos-total = Total

## Resumo

impostos-estimativa-de = Estimativa de { $ano }
impostos-periodo-rotulo = Período
impostos-pais = País
impostos-nao-informado = não informado
impostos-dados = Dados
impostos-total-estimado = Total estimado
impostos-total-estimado-cenario = Total estimado · { $cenario }
impostos-origem-estimativa = Estimativa com os dados de { $dados } (versão { $versao })
impostos-origem-estimativa-nota = Soma do imposto de consumo do cenário, da renda do trabalho, do patrimônio, dos impostos lançados e do IR retido em proventos.
impostos-faixa = faixa de { $de } (conservador) a { $ate } (elevado)
impostos-pct-renda = % da renda
impostos-pct-renda-dica = imposto total ÷ renda bruta
impostos-pct-consumo = % do consumo
impostos-pct-consumo-dica = sobre { $valor } de consumo
impostos-sem-consumo = sem consumo estimado
impostos-dias-trabalho = Dias de trabalho para os impostos
impostos-n-dias = { $n } dias
impostos-dias-ate = até { $data }
impostos-dias-ate-media = até { $data } · referência: a carga tributária do país equivale a { $dias } dias (Tesouro Nacional, { $ano })
impostos-renda-bruta = Renda bruta estimada
impostos-renda-bruta-origem = Receitas lançadas no período mais o retido na fonte (salário e proventos)
impostos-renda-bruta-dica = receitas + retido na fonte
impostos-referencia-pais = Referência do país
impostos-em-ano = em { $ano }
impostos-dados-pais = Dados do país
impostos-referencia-texto = Arrecadação de todos os tributos do país dividida pelo PIB. Medida macroeconômica mostrada como contexto: não é comparável diretamente com a estimativa da família.
impostos-referencia-dias = { $rotulo }: { $dias } dias em { $ano } (até { $ate }), com carga de { $carga } do PIB.
impostos-sem-referencia = Os dados de { $dados } não trazem a carga tributária do país.
impostos-melhorar = Para melhorar a estimativa

## Controles

impostos-ano = Ano
impostos-erro-trocar-pais = Não foi possível trocar o país
impostos-erro-trocar-regiao = Não foi possível trocar { $regiao }
impostos-erro-trocar-dependentes = Não foi possível trocar os dependentes
impostos-pais-dados = País dos dados
impostos-esboco = esboço
impostos-sem-dados-estado = Sem dados por estado
impostos-nao-informado-opcao = Não informado
impostos-dependentes = Dependentes
impostos-dependentes-dica = Dependentes para o IR estimado a partir do salário líquido

## Fontes e metodologia

impostos-fontes = Fontes
impostos-fonte-publicada = Publicação: { $publicacao } · Consultada em { $consulta }
impostos-fonte-consultada = Consultada em { $consulta }
impostos-fontes-metodologia = Fontes e metodologia
impostos-fontes-descricao = Como a estimativa de { $dados } é feita e de onde vem cada número.
impostos-versao = Versão
impostos-atualizado-em = Atualizado em
impostos-cobertura = Cobertura
impostos-fontes-desatualizados = A última revisão destes números foi em { $data }, há mais de 6 meses. Alíquotas e tabelas podem ter mudado desde então.
impostos-esboco-maiusc = Esboço
impostos-fontes-esboco = Os dados de { $dados } ainda são parciais: só parte dos impostos é estimada.
impostos-secao-aviso = Aviso
impostos-metodologia = Metodologia
impostos-ajuste-estado = Ajuste do estado
impostos-ajuste-estado-texto = As tabelas de consumo usam o ICMS de referência de { $icms }. Nas mercadorias da alíquota modal, soma-se a diferença entre a modal do estado escolhido e essa referência.
impostos-renda-trabalho = Renda do trabalho
impostos-a-partir-de = a partir de { $data }
impostos-contexto = Contexto
impostos-contexto-texto = { $rotulo }: { $valor } em { $ano }.
impostos-fontes-n = Fontes ({ $n })
impostos-ver-fontes = Ver fontes e metodologia

## Fora da estimativa

impostos-fora = Fora da estimativa
impostos-gasto-fora = Gasto fora
impostos-gasto-estimado = Gasto estimado
impostos-fora-vazio = Todo o gasto do período entrou na estimativa.
impostos-escolher-grupo = Escolher grupo
impostos-escolher-grupo-de = Escolher grupo de { $categoria }
impostos-aliquota-propria = Alíquota própria
impostos-aliquota-propria-de = Alíquota própria de { $categoria }
impostos-categorizar = Categorizar em Lançamentos

## Grupos das categorias e ajustes de uma categoria

impostos-padrao-app = Padrão do app
impostos-padrao-app-grupo = Padrão do app: { $grupo }
impostos-padrao-app-longo = Padrão do app (pelo nome ou pelo grupo pai)
impostos-grupos-categorias = Grupos das categorias
impostos-grupos-descricao = Cada categoria de despesa cai num grupo de imposto. O padrão vem pelo nome da categoria (ou do grupo pai); troque o grupo ou fixe uma alíquota própria quando souber mais sobre o seu gasto. Vale para todos os anos.
impostos-filtrar-categorias = Filtrar categorias
impostos-filtrar-placeholder = Filtrar por categoria ou grupo
impostos-carregando-categorias = Carregando as categorias
impostos-categoria = Categoria
impostos-grupo = Grupo
impostos-acoes = Ações
impostos-filtro-vazio = Nenhuma categoria com esse filtro.
impostos-erro-salvar = Não foi possível salvar
impostos-aliquota-invalida = Alíquota inválida
impostos-aliquota-faixa = Use um número entre 0 e 100 (menor que 100).
impostos-sem-grupo-fora = sem grupo: fora da estimativa
impostos-grupo-de = Grupo de { $categoria }
impostos-do-pais = do país
impostos-voltar-padrao = Voltar ao padrão
impostos-voltar-padrao-de = Voltar { $categoria } ao padrão
impostos-imposto-pago = imposto pago
impostos-fora-consumo = fora do consumo
impostos-categoria-padrao = Categoria no padrão
impostos-grupo-trocado = Grupo trocado
impostos-grupo-padrao = Grupo no padrão
impostos-aliquota-pais = Alíquota do país
impostos-aliquota-salva = Alíquota própria salva
impostos-trocar-grupo-titulo = Trocar grupo de imposto
impostos-trocar-grupo-descricao = { $categoria }. O grupo diz que imposto incide sobre o gasto da categoria; o novo grupo vale para todos os anos.
impostos-hoje = Hoje: { $grupo } ({ $origem })
impostos-aliquota-descricao = { $categoria }. Fração do preço que você considera imposto, no lugar da tabela do país. Vale nos três cenários.
impostos-usar-pais = Usar a do país
impostos-aliquota-pct = Alíquota (% do preço)
impostos-aliquota-exemplo = ex.: 18,5
impostos-base-de = Base · { $grupo }
impostos-base-grupo = Base do grupo
impostos-produtos-servicos = Produtos e serviços
impostos-icms-modal = Mercadoria na alíquota modal do ICMS: recebe o ajuste do estado escolhido (diferença para a referência de { $icms }).
impostos-fontes-grupo = Fontes do grupo
impostos-valor-como-esta = { $grupo }: o valor lançado na categoria entra como está.
impostos-categoria-sem-grupo = A categoria não tem grupo de imposto.

## Consumo por categoria

impostos-menu-trocar-grupo = Trocar grupo…
impostos-menu-aliquota = Alíquota própria…
impostos-menu-ver-base = Ver base e fontes
impostos-ja-padrao = A categoria já usa o grupo e a alíquota padrão.
impostos-aliquota-propria-nota = Alíquota própria: vale nos três cenários.
impostos-ajuste-icms = Inclui o ajuste do ICMS do estado: { $pontos }.
impostos-aliquota-de = Alíquota de { $categoria }: { $valores }
impostos-propria = própria
impostos-consumo-categoria = Consumo por categoria
impostos-consumo-nota = Gasto do ano por competência × fração do preço que é tributo no grupo da categoria. Passe o mouse na alíquota para ver os três cenários; use o menu da linha (botão direito) para trocar o grupo ou fixar uma alíquota própria.
impostos-consumo-vazio = Nenhum gasto de consumo com grupo de imposto no período.
impostos-acoes-de = Ações de { $categoria }
impostos-gasto = Gasto
impostos-aliquota = Alíquota
impostos-aliquota-cenario = Alíquota no cenário { $cenario }
impostos-do-consumo = Do consumo
impostos-ver-base-de = Ver base de { $categoria }
impostos-trocar-grupo-de = Trocar grupo de { $categoria }

## Impostos lançados, renda do trabalho, patrimônio e proventos

impostos-lancados = Impostos lançados
impostos-lancados-nota = O que foi lançado nas categorias de imposto entra pelo valor lançado, igual nos três cenários.
impostos-lancados-vazio = Nenhum imposto lançado como despesa no período.
impostos-origem-holerites = holerites
impostos-origem-informado = informado
impostos-origem-estimado = ≈ estimado
impostos-renda-trabalho-nota = Sem informe, o bruto de cada mês é estimado a partir do salário líquido recebido nas categorias Salário, Férias e 13º, pelas tabelas de IR e INSS vigentes no mês, sem outros descontos (plano de saúde, pensão). Encargos do empregador não entram.
impostos-renda-trabalho-nota-exterior = Sem os recibos de salário, o bruto de cada mês é estimado a partir do salário líquido recebido na categoria Salário (e nos pagamentos extras do país), pelas regras da folha em vigor no ano, sem outros descontos. Os encargos do empregador não entram.
impostos-informe-inss = INSS
impostos-renda-trabalho-vazio = Nenhum salário recebido nem informe de rendimentos no período.
impostos-membro = Membro
impostos-sem-membro = Sem membro
impostos-origem = Origem
impostos-meses = Meses
impostos-liquido-recebido = Líquido recebido
impostos-bruto = Bruto
impostos-ir-retido = IR retido
impostos-regras-usadas = Regras usadas na estimativa
impostos-patrimonio-nota = IPTU e IPVA lançados entram pelo valor. Sem IPTU lançado, vale o carnê registrado no imóvel (Carteira › Bens). Sem IPVA lançado no ano, ele é estimado pelo valor de cada veículo no fim do período e a alíquota do estado.
impostos-patrimonio-lancados = IPTU e IPVA vêm dos lançamentos (veja Impostos lançados).
impostos-patrimonio-vazio = Nenhum imposto sobre patrimônio lançado ou estimado no período.
impostos-imposto-estimado-de = { $imposto } estimado · { $bem }
impostos-valor-vezes = Valor { $valor } × { $aliquota }
impostos-valor-venal-vezes = Valor venal { $valor } × { $aliquota }
impostos-ir-proventos = IR retido em proventos
impostos-retido-fonte = Retido na fonte
impostos-retido-fonte-origem = Proventos registrados na Carteira (JCP, dividendos do exterior)
impostos-retido-texto = Imposto retido nos proventos pagos no período, na moeda da casa.
impostos-ver-proventos = Ver proventos

## Informes de rendimentos

impostos-informe-alterado = Informe alterado
impostos-informe-lancado = Informe lançado
impostos-informe-editar-titulo = Editar informe de rendimentos · { $ano }
impostos-informe-lancar-titulo = Lançar informe de rendimentos · { $ano }
impostos-informe-descricao = Os valores do comprovante de rendimentos que a fonte pagadora entrega no início do ano seguinte (quadros 3 e 5).
impostos-fonte-pagadora = Fonte pagadora
impostos-sem-membro-casa = Sem membro (da casa)
impostos-rendimentos-tributaveis = Rendimentos tributáveis
impostos-sem-13 = sem o 13º
impostos-previdencia-oficial = Previdência oficial (INSS)
impostos-ir-retido-fonte = IR retido na fonte
impostos-sem-ir-13 = sem o do 13º
impostos-13-salario = 13º salário
impostos-ir-13 = IR retido sobre o 13º
impostos-observacoes = Observações
impostos-da-casa = Da casa
impostos-informes-de = Informes de rendimentos de { $ano }
impostos-informes-titulo = Informes de rendimentos · { $ano }
impostos-informes-texto = Com o informe, a renda do trabalho do membro usa os valores dele no lugar da estimativa.
impostos-lancar-informe = Lançar informe
impostos-editar-menu = Editar…
impostos-excluir = Excluir
impostos-acoes-informe = Ações do informe de { $fonte }
impostos-13-bruto = 13º bruto
impostos-ir-do-13 = IR do 13º
impostos-editar-informe = Editar informe de { $fonte }
impostos-excluir-informe = Excluir informe
impostos-excluir-informe-texto = O informe de { $fonte } ({ $ano }) sai da estimativa; a renda volta a ser estimada pelo líquido. Dá para desfazer no histórico.
impostos-informes-vazio = Nenhum informe de { $ano } lançado.

## Só no Brasil: para onde vai o imposto e a arrecadação da cidade
impostos-destino-titulo = Para onde vai o imposto
impostos-destino-texto = O imposto estimado, repartido como a despesa do governo geral (União, estados e municípios) em { $ano }, por função.
impostos-destino-ressalva = É uma repartição proporcional, não o destino de cada real: o governo também se financia com dívida e outras receitas, e a linha de serviços públicos gerais inclui os juros da dívida.
impostos-estado = Estado
impostos-cidade = Cidade
impostos-cidade-titulo = Arrecadação da sua cidade
impostos-cidade-titulo-de = Arrecadação de { $cidade }
impostos-cidade-escolha-texto = Escolha a cidade onde a família mora para ver quanto o município arrecadou em tributos. A lista de cidades vem do IBGE.
impostos-cidade-escolha-estado = Escolha o estado
impostos-cidade-escolha = Escolha a cidade
impostos-cidade-carregando = Carregando…
impostos-cidade-salvar = Salvar
impostos-cidade-cancelar = Cancelar
impostos-cidade-trocar = Trocar cidade
impostos-cidade-erro-salvar = Não deu para salvar a cidade
impostos-cidade-erro-buscar = Não deu para buscar a arrecadação
impostos-cidade-por-habitante = Por habitante, em 12 meses
impostos-cidade-habitantes = { $n } habitantes
impostos-cidade-total = Tributos municipais em 12 meses
impostos-cidade-periodo = até { $mes }
impostos-cidade-fonte = Tesouro Nacional (Siconfi)
impostos-cidade-por-tributo = Por tributo
impostos-cidade-por-hab = por hab.
impostos-cidade-ressalva = É o que a prefeitura arrecadou (IPTU, ISS, ITBI e taxas), não o que cada morador paga: empresas pagam boa parte, e impostos federais e estaduais ficam de fora.
impostos-cidade-conferido = Consultado em { $data }.
impostos-cidade-sem-dados = Nenhuma consulta feita ainda para esta cidade.
impostos-cidade-fonte-desligada = A fonte Tesouro Nacional (Siconfi) está desligada em Ajustes › Fontes de dados.
impostos-cidade-buscar = Buscar no Tesouro Nacional
impostos-cidade-atualizar = Atualizar
impostos-cidade-buscando = Buscando…
