# FGTS (núcleo: `fgts`; docs/brazil/fgts.md): extrato da Caixa, conferência, saques, saque-aniversário,
# projeção e comparação do rendimento. pt-BR é o idioma de origem.

nucleo-fgts-empregador = empregador
nucleo-fgts-tipo-deposito = Depósito
nucleo-fgts-tipo-deposito-extra = Depósito complementar
nucleo-fgts-tipo-jam = JAM (juros e correção)
nucleo-fgts-tipo-ajuste-jam = Ajuste do JAM
nucleo-fgts-tipo-distribuicao = Distribuição de resultado
nucleo-fgts-tipo-saque = Saque
nucleo-fgts-tipo-reposicao = Reposição de saque
nucleo-fgts-saque-motivo-rescisao = rescisão
nucleo-fgts-saque-motivo-aposentadoria = aposentadoria
nucleo-fgts-saque-motivo-moradia = moradia
nucleo-fgts-saque-motivo-aniversario = saque-aniversário
nucleo-fgts-saque-motivo-doenca = doença
nucleo-fgts-saque-motivo-emergencial = saque emergencial
nucleo-fgts-saque-motivo-outros = outros
nucleo-fgts-lancamento-saque = Saque do FGTS ({ $motivo })

nucleo-fgts-motivo-extrato = Importar extrato do FGTS
nucleo-fgts-motivo-saque = Registrar saque do FGTS
nucleo-fgts-motivo-moradia = Usar o FGTS no financiamento
nucleo-fgts-motivo-modalidade = Mudar a modalidade de saque do FGTS
nucleo-fgts-motivo-conta = Editar a conta FGTS

nucleo-fgts-erro-extrato-vazio = Não achei lançamentos do FGTS neste extrato (linhas com data, valor e total).
nucleo-fgts-erro-sem-arquivo = Envie o PDF do extrato do FGTS ou o texto dele.
nucleo-fgts-erro-pdf = Não consegui ler o PDF do extrato: { $erro }
nucleo-fgts-erro-nao-e-fgts = A conta escolhida não é uma conta FGTS.
nucleo-fgts-erro-renda = fonte de renda
nucleo-fgts-erro-motivo = Motivo de saque desconhecido: { $motivo }.
nucleo-fgts-erro-valor-positivo = O valor precisa ser maior que zero.
nucleo-fgts-erro-saldo-insuficiente = O valor passa do saldo da conta FGTS nessa data.
nucleo-fgts-erro-contrato = contrato
nucleo-fgts-erro-contrato-imobiliario = O FGTS só pode ser usado em contrato de financiamento imobiliário.
nucleo-fgts-erro-modalidade = A modalidade precisa ser saque-rescisão ou saque-aniversário.
nucleo-fgts-erro-membro = membro
nucleo-fgts-erro-nascimento = Para projetar até uma idade, cadastre a data de nascimento do titular.
nucleo-fgts-erro-data-ou-idade = Informe a data ou a idade da projeção.

nucleo-fgts-aviso-saldo = { $n ->
    [one] Uma linha do extrato não bate com o saldo impresso.
   *[other] { $n } linhas do extrato não batem com o saldo impresso.
}
nucleo-fgts-aviso-jam = { $n ->
    [one] Um crédito de JAM ficou diferente do calculado pela regra.
   *[other] { $n } créditos de JAM ficaram diferentes do calculado pela regra.
}
nucleo-fgts-aviso-juros-progressivos = A conta tem juros progressivos (optante antes de 1971): o app guarda a taxa, mas não calcula os juros progressivos.
nucleo-fgts-aviso-codigo-99 = Saques com código 99 foram tratados como uso na moradia; mude o motivo se não foi.
nucleo-fgts-aviso-cabecalho = Não deu para ler todo o cabeçalho do extrato (empregador, CNPJ ou datas do vínculo). Confira esses dados na conta depois de importar.
nucleo-fgts-aviso-dois-anos = Houve uso do FGTS na moradia em { $data }: a amortização com o FGTS pede pelo menos 2 anos entre um uso e outro (Lei 8.036, art. 20).

nucleo-fgts-nota-aniversario = Estimativa pela tabela do saque-aniversário (Lei 8.036, Anexo): uma faixa só, pelo saldo somado das contas. O pedido de mudança é no app FGTS da Caixa.
nucleo-fgts-nota-antecipacao = Com antecipação contratada, a volta ao saque-rescisão só vale depois de quitar a antecipação.
nucleo-fgts-nota-conferencia = O JAM é recalculado pelo saldo até o crédito anterior, com o coeficiente do extrato, truncado em centavos; os depósitos são comparados com o FGTS dos holerites da renda ligada.
nucleo-fgts-nota-extrato-recente = Competências sem depósito podem estar só fora do último extrato importado: importe um extrato recente antes de cobrar o empregador.
nucleo-fgts-nota-projecao = Projeção com a última TR ({ $tr }% a.m.) mantida, juros de 3% a.a., distribuição pela média dos últimos índices e inflação de { $ipca }% em 12 meses para os valores em reais de hoje.
nucleo-fgts-nota-sem-renda = Conta ativa sem renda ligada: a projeção não soma depósitos novos.
nucleo-fgts-nota-comparar = Os mesmos depósitos e saques, nas mesmas datas, aplicados em cada destino. 85% do CDI é só uma régua para algo isento de IR, como o FGTS. Não é recomendação.
nucleo-fgts-nota-adi = Pela ADI 5090 (STF, 12/06/2024), a remuneração do FGTS (JAM + distribuição) deve ser pelo menos o IPCA a partir de então, sem efeito retroativo; a compensação é definida pelo Conselho Curador.
nucleo-fgts-nota-sem-serie = Falta no cofre alguma série do Banco Central (IPCA, poupança, CDI ou M2): sincronize os índices para completar a comparação.

nucleo-fgts-comparar-ipca = Corrigido pelo IPCA
nucleo-fgts-comparar-poupanca = Poupança
nucleo-fgts-comparar-cdi-85 = 85% do CDI
nucleo-fgts-comparar-cdi = 100% do CDI
nucleo-fgts-comparar-m2 = Expansão do M2
nucleo-fgts-evento-aposentadoria = Saque do FGTS na aposentadoria
nucleo-fgts-nota-aposentadoria = O saldo FGTS projetado ({ $valor } em reais de hoje) entra como saque na aposentadoria.
