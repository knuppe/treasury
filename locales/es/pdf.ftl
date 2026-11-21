# Informe del mes en PDF (06 · Informes › Descargar PDF), armado por el núcleo en tu propia
# computadora (reports/pdf_report.rs), y el botón de la interfaz. Revisión: borrador por IA,
# sin revisión nativa.

## Interfaz

pdf-botao = Descargar PDF del mes
pdf-botao-dica = Informe de { $mes } en PDF, generado en tu computadora sin enviar nada a internet.
pdf-salvar-titulo = Guardar el informe en PDF
pdf-arquivo = informe-treasury-{ $mes }.pdf
pdf-feito = Informe guardado
pdf-feito-paginas = { $paginas ->
    [one] 1 página en { $caminho }
   *[other] { $paginas } páginas en { $caminho }
}
pdf-erro = No se pudo generar el PDF

## Documento

pdf-titulo = Informe de { $mes }
pdf-mes-ano = { $mes } de { $ano }
pdf-base-competencia = Devengo (fecha de compra)
pdf-base-caixa = Caja (fecha de pago)
pdf-receitas = Ingresos
pdf-despesas = Gastos
pdf-sobra = Lo que sobró
pdf-taxa-poupanca = { $taxa } de los ingresos ahorrado
pdf-patrimonio = Patrimonio neto
pdf-sem-cambio = Sin tipo de cambio para { $moedas }: esos montos quedaron fuera de los totales.
pdf-secao-categorias = A dónde fue el dinero
pdf-secao-categorias-dica = Gastos del mes por categoría, con la parte de cada una en el total y la variación respecto al mes anterior.
pdf-sem-gastos = Ningún gasto este mes.
pdf-col-categoria = Categoría
pdf-col-valor = Monto
pdf-col-parte = Parte
pdf-col-mes-anterior = Mes anterior
pdf-novo = nuevo
pdf-secao-12-meses = Entradas y salidas, 12 meses
pdf-secao-inflacao = Inflación
pdf-inflacao-resumo = Inflación acumulada en 12 meses: { $inflacao } ({ $indice }, último dato de { $data }). La línea ajustada muestra cuánto valdría hoy el gasto de cada mes.
pdf-inflacao-sem-indice = Todavía no hay datos del índice de inflación de tu país para ajustar los montos. Actualiza los índices en Activos › Índices y cotizaciones.
pdf-inflacao-estimado = El tramo punteado usa meses que el índice aún no publicó (estimación).
pdf-inflacao-nominal = Gasto del mes (nominal)
pdf-inflacao-corrigido = Ajustado por inflación (valor de hoy)
pdf-secao-inflacao-categorias = Categorías vs. inflación
pdf-secao-inflacao-categorias-dica = Promedio de los tres primeros meses con gasto y los tres últimos. Por encima de la inflación, la categoría se encareció de verdad (o gastaste más); por debajo, se abarató.
pdf-col-inicio = Inicio
pdf-col-fim = Fin
pdf-col-variacao = Variación
pdf-col-inflacao = Inflación
pdf-col-real = Real
pdf-acima = { $pp } por encima
pdf-abaixo = { $pp } por debajo
pdf-igual = igual
pdf-secao-patrimonio = Patrimonio neto, 12 meses
pdf-secao-vencimentos = Próximos vencimientos (30 días)
pdf-sem-vencimentos = Nada vence en los próximos 30 días.
pdf-rodape = Generado en tu computadora por Treasury el { $data }. Montos en { $moeda }.
pdf-pagina = Página { $n } de { $total }
