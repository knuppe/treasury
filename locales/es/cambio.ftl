# Cambio de moneda entre cuentas propias (features/fx): la lista de la cuenta y el diálogo con la
# referencia PTAX, la tasa, los cargos y la vista previa del VET (docs/i18n-design.md).
# Revisión: borrador por IA, sin revisión de un hablante nativo.

## Operaciones de la cuenta

cambio-data = Fecha
cambio-par = Cambio
cambio-saiu = Salió
cambio-entrou = Entró
cambio-taxa = Tasa
cambio-iof-tarifa = IOF + comisión
cambio-vet = VET
cambio-operacoes = Operaciones de cambio
cambio-nova-operacao = Nueva operación de cambio
cambio-operacoes-conta = Operaciones de cambio de la cuenta
cambio-nenhuma-operacao = Ninguna operación de cambio en esta cuenta.

## Diálogo de la operación

cambio-titulo = Operación de cambio
cambio-descricao = Transferencia entre tus cuentas en monedas distintas, con la tasa y los cargos del comprobante de cambio. El IOF y la comisión entran como gastos.
cambio-lancado = Cambio registrado
cambio-lancar = Registrar cambio
cambio-de-conta = De (cuenta)
cambio-para-conta = A (cuenta)
cambio-valor-conhecido = Monto conocido
cambio-que-entra = El que entra
cambio-que-entra-moeda = El que entra ({ $moeda })
cambio-que-sai = El que sale
cambio-que-sai-moeda = El que sale ({ $moeda })
cambio-valor-moeda = Monto ({ $moeda })
cambio-referencia = { $base ->
        [BRL] Referencia PTAX
       *[other] Tipo de cambio de referencia
    }
cambio-referencia-moeda = { $base ->
        [BRL] Referencia PTAX (1 { $moeda } en R$)
       *[other] Tipo de cambio de referencia (1 { $moeda } en { $base })
    }
cambio-fechamento = Cierre
cambio-referencia-dica = { $base ->
        [BRL] { $boletim } del { $data } · Banco Central de Brasil
       *[other] { $boletim } del { $data }
    }
cambio-referencia-dica-outro-dia = { $base ->
        [BRL] { $boletim } del { $data } (de otro día) · Banco Central de Brasil
       *[other] { $boletim } del { $data } (de otro día)
    }
cambio-sem-referencia = Sin cotización de referencia: ingresa la tasa aplicada.
cambio-taxa-aplicada-opcional = Tasa aplicada (opcional)
cambio-taxa-aplicada-dica = Del comprobante de cambio; sin ella, usa la referencia + spread.
cambio-spread = Spread (%)
cambio-spread-calculado = Calculado a partir de la tasa aplicada.
cambio-spread-sobre = Sobre la referencia.
cambio-iof = IOF (%)
cambio-iof-dica = Revisa el comprobante: cambia según el tipo de operación.
cambio-tarifa-moeda = Comisión ({ $moeda })
cambio-observacoes = Observaciones
cambio-observacoes-exemplo = Ej.: contrato de cambio n.º…
cambio-mesma-moeda = Las dos cuentas están en { $moeda }: usa una transferencia común.

## Vista previa

cambio-previa = Vista previa del cambio
cambio-taxa-aplicada = Tasa aplicada
cambio-sai = Sale
cambio-iof-rotulo = IOF
cambio-tarifa = Comisión
cambio-total-sai = Total que sale
cambio-entra = Entra
cambio-vet-rotulo = { $base ->
        [BRL] VET (R$ por unidad)
       *[other] VET ({ $base } por unidad)
    }
