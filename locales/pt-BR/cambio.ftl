# Operações de câmbio entre contas próprias (features/fx): a lista da conta e o diálogo com a
# referência PTAX, a taxa, os encargos e a prévia do VET (docs/i18n-design.md). pt-BR é o idioma
# de origem.

## Operações da conta

cambio-data = Data
cambio-par = Câmbio
cambio-saiu = Saiu
cambio-entrou = Entrou
cambio-taxa = Taxa
cambio-iof-tarifa = IOF + tarifa
cambio-vet = VET
cambio-operacoes = Operações de câmbio
cambio-nova-operacao = Nova operação de câmbio
cambio-operacoes-conta = Operações de câmbio da conta
cambio-nenhuma-operacao = Nenhuma operação de câmbio nesta conta.

## Diálogo da operação

cambio-titulo = Operação de câmbio
cambio-descricao = Transferência entre suas contas em moedas diferentes, com a taxa e os encargos da nota de câmbio. O IOF e a tarifa entram como despesas.
cambio-lancado = Câmbio lançado
cambio-lancar = Lançar câmbio
cambio-de-conta = De (conta)
cambio-para-conta = Para (conta)
cambio-valor-conhecido = Valor conhecido
cambio-que-entra = Que entra
cambio-que-entra-moeda = Que entra ({ $moeda })
cambio-que-sai = Que sai
cambio-que-sai-moeda = Que sai ({ $moeda })
cambio-valor-moeda = Valor ({ $moeda })
cambio-referencia = { $base ->
        [BRL] Referência PTAX
       *[other] Câmbio de referência
    }
cambio-referencia-moeda = { $base ->
        [BRL] Referência PTAX (1 { $moeda } em R$)
       *[other] Câmbio de referência (1 { $moeda } em { $base })
    }
cambio-fechamento = Fechamento
cambio-referencia-dica = { $base ->
        [BRL] { $boletim } de { $data } · Banco Central
       *[other] { $boletim } de { $data }
    }
cambio-referencia-dica-outro-dia = { $base ->
        [BRL] { $boletim } de { $data } (de outro dia) · Banco Central
       *[other] { $boletim } de { $data } (de outro dia)
    }
cambio-sem-referencia = Sem cotação de referência: informe a taxa aplicada.
cambio-taxa-aplicada-opcional = Taxa aplicada (opcional)
cambio-taxa-aplicada-dica = Da nota de câmbio; sem ela, usa referência + spread.
cambio-spread = Spread (%)
cambio-spread-calculado = Calculado pela taxa aplicada.
cambio-spread-sobre = Sobre a referência.
cambio-iof = IOF (%)
cambio-iof-dica = Confira na nota: muda com o tipo de operação.
cambio-tarifa-moeda = Tarifa ({ $moeda })
cambio-observacoes = Observações
cambio-observacoes-exemplo = Ex.: contrato de câmbio nº…
cambio-mesma-moeda = As duas contas estão em { $moeda }: use uma transferência comum.

## Prévia

cambio-previa = Prévia do câmbio
cambio-taxa-aplicada = Taxa aplicada
cambio-sai = Sai
cambio-iof-rotulo = IOF
cambio-tarifa = Tarifa
cambio-total-sai = Total que sai
cambio-entra = Entra
cambio-vet-rotulo = { $base ->
        [BRL] VET (R$ por unidade)
       *[other] VET ({ $base } por unidade)
    }
