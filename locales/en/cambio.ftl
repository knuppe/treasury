# Currency exchange between your own accounts (features/fx): the account's list and the dialog
# with the PTAX reference, the rate, the charges and the VET preview (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## The account's operations

cambio-data = Date
cambio-par = Exchange
cambio-saiu = Out
cambio-entrou = In
cambio-taxa = Rate
cambio-iof-tarifa = IOF + fee
cambio-vet = VET
cambio-operacoes = Currency exchanges
cambio-nova-operacao = New currency exchange
cambio-operacoes-conta = The account's currency exchanges
cambio-nenhuma-operacao = No currency exchanges in this account.

## Exchange dialog

cambio-titulo = Currency exchange
cambio-descricao = A transfer between your accounts in different currencies, with the rate and charges from the exchange receipt. IOF and the fee count as expenses.
cambio-lancado = Exchange recorded
cambio-lancar = Record exchange
cambio-de-conta = From (account)
cambio-para-conta = To (account)
cambio-valor-conhecido = Known amount
cambio-que-entra = Coming in
cambio-que-entra-moeda = Coming in ({ $moeda })
cambio-que-sai = Going out
cambio-que-sai-moeda = Going out ({ $moeda })
cambio-valor-moeda = Amount ({ $moeda })
cambio-referencia = { $base ->
        [BRL] PTAX reference
       *[other] Reference rate
    }
cambio-referencia-moeda = { $base ->
        [BRL] PTAX reference (1 { $moeda } in R$)
       *[other] Reference rate (1 { $moeda } in { $base })
    }
cambio-fechamento = Closing
cambio-referencia-dica = { $base ->
        [BRL] { $boletim } on { $data } · Central Bank of Brazil
       *[other] { $boletim } on { $data }
    }
cambio-referencia-dica-outro-dia = { $base ->
        [BRL] { $boletim } on { $data } (from another day) · Central Bank of Brazil
       *[other] { $boletim } on { $data } (from another day)
    }
cambio-sem-referencia = No reference rate: enter the rate applied.
cambio-taxa-aplicada-opcional = Rate applied (optional)
cambio-taxa-aplicada-dica = From the exchange receipt; without it, uses reference + spread.
cambio-spread = Spread (%)
cambio-spread-calculado = Calculated from the rate applied.
cambio-spread-sobre = Over the reference.
cambio-iof = IOF (%)
cambio-iof-dica = Check the receipt: it changes with the type of operation.
cambio-tarifa-moeda = Fee ({ $moeda })
cambio-observacoes = Notes
cambio-observacoes-exemplo = E.g. exchange contract no.…
cambio-mesma-moeda = Both accounts are in { $moeda }: use a regular transfer.

## Preview

cambio-previa = Exchange preview
cambio-taxa-aplicada = Rate applied
cambio-sai = Out
cambio-iof-rotulo = IOF
cambio-tarifa = Fee
cambio-total-sai = Total going out
cambio-entra = In
cambio-vet-rotulo = { $base ->
        [BRL] VET (R$ per unit)
       *[other] VET ({ $base } per unit)
    }
