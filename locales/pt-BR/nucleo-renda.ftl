# Rendas da família e holerites (núcleo, `income`; docs/income.md): pagamentos extras da
# previsão, avisos da previsão e da leitura do holerite, motivos do histórico e erros. Os rótulos
# entre aspas são os do documento (holerite brasileiro) e ficam como estão. pt-BR é o idioma de
# origem.

nucleo-renda-erro-renda = renda { $id }
nucleo-renda-erro-bruto-negativo = o valor bruto não pode ser negativo
nucleo-renda-erro-liquido-negativo = o valor líquido não pode ser negativo
nucleo-renda-erro-liquido-maior = o líquido não pode ser maior que o bruto
nucleo-renda-erro-dia-pagamento = dia do pagamento de 1 a 31
nucleo-renda-erro-mes-ferias = mês das férias de 1 a 12
nucleo-renda-erro-termina-antes = a renda termina antes de começar

## Pagamentos extras (previsão e holerites que não são o mensal)

nucleo-renda-extra-13-1 = 1ª parcela do 13º
nucleo-renda-extra-13-2 = 2ª parcela do 13º
nucleo-renda-extra-terco-ferias = terço de férias
nucleo-renda-extra-subsidio-natal = subsídio de Natal
nucleo-renda-extra-subsidio-ferias = subsídio de férias
nucleo-renda-extra-paga-junho = paga extra de junho
nucleo-renda-extra-paga-dezembro = paga extra de dezembro
nucleo-renda-extra-ferias = férias
nucleo-renda-extra-plr = PLR
nucleo-renda-extra-adiantamento = adiantamento
nucleo-renda-extra-outro = outro pagamento

## Avisos da previsão

nucleo-renda-nota-sem-valor = { $renda }: informe o bruto ou o líquido (ou importe um holerite) para entrar na previsão.
nucleo-renda-nota-br = INSS e IRRF calculados pelas regras do ano quando não vêm do holerite; pensão alimentícia, plano de saúde e outros descontos só entram pelo holerite.
nucleo-renda-nota-us = FICA e imposto federal estimados pelas regras de 2026 quando não vêm do paystub, com a dedução padrão e o estado civil do cadastro da família (individual, chefe de família ou declaração conjunta); imposto estadual, 401(k) e plano de saúde só entram pelo paystub.
nucleo-renda-nota-pt = Segurança Social e IRS estimados pelas regras de 2026 quando não vêm do recibo de vencimento (o IRS pelo imposto anual dos escalões, não pelas tabelas de retenção).
nucleo-renda-nota-es = Seguridad Social e IRPF estimados pelas regras de 2026 quando não vêm da nómina (o IRPF pela escala geral, sem as escalas próprias de cada comunidade autónoma).
nucleo-renda-extra-aguinaldo = aguinaldo (SAC)
nucleo-renda-nota-adaptado = Contribuições e imposto estimados pelas regras de 2026 do país quando não vêm do recibo de salário (uma aproximação: sem as deduções e créditos pessoais além dos básicos).
nucleo-renda-nota-outro = Os descontos do salário não são calculados para este país: informe o líquido ou importe o recibo.

## Histórico de alterações

nucleo-renda-motivo-criada = Renda cadastrada: { $renda }
nucleo-renda-motivo-alterada = Renda alterada: { $renda }
nucleo-renda-motivo-apagada = Renda apagada: { $renda }
nucleo-renda-motivo-holerite-importado = Holerite importado
nucleo-renda-motivo-holerite-apagado = Holerite apagado

## Leitura e importação do holerite

nucleo-renda-aviso-sem-competencia = não achei a competência (mês/ano) do holerite: informe na importação
nucleo-renda-aviso-nao-bate = bruto menos descontos ({ $calculado }) não bate com o líquido ({ $liquido }): confira as linhas
nucleo-renda-aviso-sem-vencimentos = não achei os vencimentos do holerite
nucleo-renda-aviso-escolha-renda = escolha a renda ou o membro (para criar a renda deste empregador)
nucleo-renda-erro-base64 = { $arquivo }: base64 inválido
nucleo-renda-erro-imagem = { $arquivo }: é uma imagem, não um PDF. Leia os valores da imagem e mande o texto do holerite em `text` (as linhas com rótulo e valor, como "Líquido a receber 2.043,36")
nucleo-renda-erro-pdf-senha = PDF protegido por senha: informe a senha do holerite
nucleo-renda-erro-pdf-ilegivel = não foi possível ler o PDF: { $erro }
nucleo-renda-erro-sem-arquivo = envie o PDF do holerite (ou o texto)
nucleo-renda-falta-bruto = o total bruto ("Total de vencimentos", "Total de proventos" ou "Total bruto")
nucleo-renda-falta-liquido = o líquido ("Líquido a receber" ou "Valor líquido")
nucleo-renda-faltam-dois = { $um } nem { $outro }
nucleo-renda-erro-incompleto = o texto parece um holerite, mas não achei { $faltou }. Mande o texto inteiro do documento, sem resumir nem reescrever (as linhas com rótulo e valor, como "Líquido a receber 2.043,36"), ou o PDF anexado em `attachments` (ex.: ["1"])
nucleo-renda-erro-competencia = informe a competência do holerite (YYYY-MM)
nucleo-renda-erro-ilegivel = o holerite não tem bruto e líquido legíveis
nucleo-renda-erro-escolha-renda = escolha a renda ou o membro do holerite
nucleo-renda-erro-duplicado = já existe o holerite de { $mes } desta renda; use substituir para trocar
nucleo-renda-erro-holerite = holerite { $id }
# Nome da renda criada pela importação de um holerite de empregador novo.
nucleo-renda-nome-salario-de = Salário · { $empregador }
nucleo-renda-nome-salario = Salário
nucleo-renda-aviso-sem-sinal = { $linhas } sem sinal de desconto, mas os totais impressos só fecham com ela(s) nos descontos: confira
nucleo-renda-aviso-totais = as linhas não fecham com os totais impressos (vencimentos { $vencimentos } × { $bruto }, descontos { $descontos } × { $total_descontos }): os totais valem, confira as linhas
nucleo-renda-mensal = mensal
nucleo-renda-aviso-plr-irrf = IR retido da PLR ({ $retido }) diferente do da tabela exclusiva da PLR ({ $esperado }), contando as PLR anteriores do ano. Confira (pensão alimentícia descontada reduz o imposto).

## Simulação de demissão (`income.simulate_termination`)

nucleo-renda-erro-rescisao-data = `{ $campo }` deve ser uma data AAAA-MM-DD
nucleo-renda-erro-rescisao-salario = informe o salário bruto ou escolha uma renda com salário bruto
nucleo-renda-erro-rescisao-admissao = informe a data de admissão
nucleo-renda-erro-rescisao-ordem = o desligamento não pode ser antes da admissão
nucleo-renda-erro-rescisao-fim-contrato = informe o fim combinado do contrato de experiência
nucleo-renda-erro-rescisao-tabela = sem tabela de INSS e IRRF para o mês do desligamento
nucleo-renda-rescisao-saldo-salario = Saldo de salário
nucleo-renda-rescisao-aviso-previo = Aviso prévio indenizado
nucleo-renda-rescisao-decimo-terceiro = 13º salário proporcional
nucleo-renda-rescisao-ferias-proporcionais = Férias proporcionais + 1/3
nucleo-renda-rescisao-ferias-vencidas = Férias vencidas + 1/3
nucleo-renda-rescisao-indenizacao-479 = Indenização do contrato de experiência (art. 479)
nucleo-renda-rescisao-desconto-480 = Desconto pela quebra do contrato de experiência (art. 480)
nucleo-renda-rescisao-desconto-aviso = Desconto do aviso prévio não cumprido
nucleo-renda-rescisao-inss = INSS sobre o saldo de salário
nucleo-renda-rescisao-irrf = IRRF sobre o saldo de salário
nucleo-renda-rescisao-inss-13 = INSS sobre o 13º
nucleo-renda-rescisao-irrf-13 = IRRF sobre o 13º
nucleo-renda-rescisao-n-dias = { $n } dias
nucleo-renda-rescisao-n-dias-terco = { $n } dias + 1/3
nucleo-renda-rescisao-metade-dias = metade de { $n } dias
nucleo-renda-rescisao-aviso-estimativa = Estimativa pelas regras gerais da CLT. O valor oficial é o do termo de rescisão (TRCT) do empregador; convenção coletiva, médias de horas extras e comissões, faltas e adiantamentos podem mudar o resultado.
nucleo-renda-rescisao-aviso-13-adiantado = Se a 1ª parcela do 13º já foi paga, ela é descontada do 13º da rescisão.
nucleo-renda-rescisao-aviso-sem-fgts = Sem o saldo do FGTS deste vínculo, a multa e o saque ficaram só sobre o depósito do mês. Informe o saldo ou ligue a conta do FGTS à renda.
nucleo-renda-rescisao-aviso-aniversario = Na modalidade saque-aniversário o saldo do FGTS fica retido na demissão; só a multa pode ser sacada.
nucleo-renda-rescisao-seguro-motivo = O seguro-desemprego é só para dispensa sem justa causa.
nucleo-renda-rescisao-seguro-carencia = Sem direito: no { $pedido }º pedido são precisos { $meses } meses trabalhados nos últimos { $janela }.
nucleo-renda-rescisao-tabela-antiga = data anterior à tabela mais antiga do app
