# Figura 07 · Inflación de la pantalla de Informes.

inflacao-carregando = Cargando la inflación…
inflacao-pergunta = ¿Gastaste más o solo subieron los precios? Cada mes llevado a dinero de { $mes } por la inflación oficial.
inflacao-indice-generico = índice de inflación
inflacao-titulo = Gasto nominal × ajustado por { $indice }, { $n } meses
inflacao-aria = Líneas del gasto nominal y del gasto ajustado por { $indice } en { $n } meses, en dinero de { $mes }
inflacao-fonte = Libro de caja · base { $base } · valores ajustados hasta { $mes }
inflacao-fonte-indice = { $indice } ({ $fonte }), último dato de { $data }
inflacao-nota-estimado = Los meses aún no publicados siguen el promedio de las últimas variaciones del índice y quedan marcados con ≈.
inflacao-tabela = Gasto nominal y ajustado por mes

## Medidas

inflacao-acumulada = Inflación en { $n } meses
inflacao-sem-indice-curto = Sin índice
inflacao-gasto-nominal = Gasto nominal
inflacao-gasto-real = Gasto real
inflacao-tres-meses = Primeros × últimos 3 meses
inflacao-correcao-total = Ajuste en el período
inflacao-correcao-total-dica = Cuánto suma la inflación al gasto anterior
inflacao-sem-indice = Todavía no hay datos de { $indice } para este período. Actualiza los índices en Activos › Índices y cotizaciones.
inflacao-sem-cambio = Sin tipo de cambio para { $moedas }: esos valores quedaron fuera de los totales.

## Gráfico

inflacao-serie-nominal = Nominal
inflacao-serie-corrigido = Ajustado
inflacao-tooltip-correcao = Ajuste
inflacao-tooltip-nao-divulgado = El índice del mes aún no se publicó: ajuste estimado.
inflacao-tooltip-estimado = El ajuste usa meses aún no publicados (estimados).

## Tabla por mes

inflacao-col-corrigido = En dinero de { $mes }
inflacao-col-correcao = Ajuste
inflacao-col-indice = Índice
inflacao-status-oficial = Oficial
inflacao-status-estimado = Estimado en parte
inflacao-status-nao-divulgado = No publicado

## Por categoría

inflacao-categorias-titulo = Categorías frente a la inflación
inflacao-categorias-dica = { $acima ->
    [0] Promedio desde { $inicio } durante tres meses frente a los tres meses hasta { $fim }. Ninguna de las { $total } categorías subió por encima de la inflación.
    [one] Promedio desde { $inicio } durante tres meses frente a los tres meses hasta { $fim }. 1 de { $total } categorías subió por encima de la inflación.
   *[other] Promedio desde { $inicio } durante tres meses frente a los tres meses hasta { $fim }. { $acima } de { $total } categorías subieron por encima de la inflación.
}
inflacao-categorias-tabela = Categorías frente a la inflación
inflacao-col-inicio = Promedio al inicio
inflacao-col-fim = Promedio al final
inflacao-col-variacao = Variación
inflacao-col-acima = Sobre la inflación
