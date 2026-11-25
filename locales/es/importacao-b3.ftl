# Cartera › importación desde el Área del Inversionista de la B3 (bolsa de Brasil) y desde las
# notas de corretaje (PDF SINACOR): etapas, situación de cada fila, vista previa y resultado (solo
# en Brasil; docs/i18n-design.md). Los términos oficiales (B3, SINACOR, IRRF, ISS, CPF, JCP) quedan
# tal cual. Revisión: borrador por IA, sin revisión nativa.

## Piezas comunes

b3-etapas = Etapas
b3-todas = Todas
b3-avisos = Avisos
b3-compra = Compra
b3-venda = Venta

## Informes, situación de las filas y reglas

b3-relatorio-negociacao = Negociación
b3-relatorio-movimentacao = Movimientos
b3-relatorio-posicao = Posición
b3-status-nova = Nueva
b3-status-nova-dica = se guardará
b3-status-duplicada = Duplicada
b3-status-duplicada-dica = ya guardada antes: se omite
b3-status-possivel-duplicada = Posible duplicada
b3-status-possivel-duplicada-dica = parecida a un movimiento ingresado a mano: solo se guarda si lo pides
b3-status-sem-conta = Sin cuenta
b3-status-sem-conta-dica = la institución no tiene cuenta elegida: no se guarda
b3-status-revisar = Revisar
b3-status-revisar-dica = necesita aprobación (ej.: evento sin factor calculable)
b3-status-erro = Con error
b3-status-erro-dica = no se puede guardar
b3-origem-liquidacao = Liquidación (fecha estimada)
b3-origem-leilao = Subasta de fracciones
b3-origem-subscricao = Suscripción ejercida
b3-origem-atualizacao = Actualización de la custodia
b3-origem-abertura = Posición de apertura
b3-regra-retrato = Fotografía de la custodia
b3-regra-opcao = Opción
b3-regra-termo = Plazo
b3-regra-futuro = Futuro
b3-regra-mercado = Mercado no soportado
b3-regra-aluguel = Préstamo de acciones
b3-regra-custodia = Transferencia de custodia
b3-regra-liquidacao = Liquidación ya importada
b3-regra-subscricao = Derecho de suscripción
b3-regra-saldo = Actualización de saldo
b3-regra-manual = Evento a registrar a mano
b3-regra-repassado = Rendimiento transferido
b3-regra-estorno = Reversión de rendimiento
b3-regra-sem-valor = Sin valor
b3-regra-fracao = Fracción
b3-regra-desconhecida = Movimiento desconocido
b3-regra-invalida = Fila no válida
b3-posicao-confere = Coincide
b3-posicao-diverge = Difiere
b3-posicao-fora = Fuera de la B3
b3-nota-nova-dica = se registrará con los costos prorrateados
b3-nota-completar = Completar costos
b3-nota-completar-dica = ya existe sin costos (ej.: del informe de la B3): recibe los costos, el IRRF y el número de la nota
b3-nota-duplicada-dica = ya registrada con esta nota o con costos: se omite
b3-nota-sem-ativo = Sin activo
b3-nota-sem-ativo-dica = activo no identificado: elige el activo para guardar

## Importar desde la B3

b3-titulo = Importar desde la B3
b3-etapa-arquivos = Archivos
b3-etapa-contas = Cuentas
b3-etapa-previa = Vista previa
b3-etapa-resultado = Resultado
b3-periodo = { $de } a { $ate }
b3-origem-linha = { $aba } · fila { $linha }
b3-importados-toast = { $n ->
    [one] { $n } movimiento importado desde la B3
   *[other] { $n } movimientos importados desde la B3
}
b3-nao-importar = No importar
b3-sem-conta = sin cuenta
b3-importar-operacao = Importar operación { $n }
b3-importar-provento = Importar rendimiento { $n }
b3-importar-evento = Importar evento { $n }
b3-gerar-previa = Generar vista previa
b3-importar-n = { $n ->
    [one] Importar { $n } movimiento
   *[other] Importar { $n } movimientos
}
b3-desfeita = Importación de la B3 deshecha
b3-desfazer = Deshacer importación
b3-concluir = Terminar
b3-relatorios-area = Informes del Área del Inversionista
b3-relatorios-dica = Negociación, movimientos y posición, en Excel (.xlsx) o CSV. Envía juntos los del mismo período: la posición verifica el resultado. Los archivos quedan solo en la memoria de la app hasta que importes.
b3-lendo = Leyendo los informes…
b3-relatorios-escolhidos = Informes elegidos
b3-tirar = Quitar { $arquivo }
b3-relatorios = Informes
b3-nenhum-reconhecido = ninguno reconocido
b3-periodo-rotulo = Período
b3-instituicoes = Instituciones
b3-abas = Hojas
b3-aba-resumo = { $aba } ({ $tipo }, { $n ->
    [one] { $n } fila
   *[other] { $n } filas
})
b3-aba-ignorada = omitida
b3-avisos-de = Avisos de { $arquivo }
b3-instituicao = Institución
b3-conta-sugerida = cuenta sugerida por el nombre de la institución
b3-sem-conta-parecida = ninguna cuenta parecida: elige una o déjala fuera
b3-contas-texto = Elige la cuenta de cada institución del archivo. La sugerencia viene de una importación anterior, de la institución registrada en la cuenta o del nombre; si no hay cuenta, créala aquí mismo. Las instituciones marcadas como “No importar” quedan fuera.
b3-motivo-escolhida = elegida por ti
b3-motivo-salva = elección guardada de la importación anterior
b3-motivo-mesma-instituicao = misma institución de la cuenta
b3-motivo-nome = nombre parecido
b3-motivo-criada = cuenta nueva, creada con la importación
b3-conta-desconhecida = cuenta no encontrada
b3-criar-conta = Crear una cuenta nueva…
b3-tambem-como = también aparece como { $nomes }
b3-conta-b3 = cuenta en la B3: { $contas }
b3-nova-conta-para = Cuenta nueva para { $instituicao }
b3-nova-conta-nome = Nombre de la cuenta
b3-nova-conta-tipo = Tipo
b3-nova-conta-tipo-dica = corredora para acciones, FII y Tesouro; banco o cooperativa para RDB y CDB de la propia institución
b3-nova-conta-instituicao = Institución
b3-nova-conta-abertura = Apertura
b3-resumo-conta = { $linhas } filas: { $operacoes } operaciones, { $proventos } rendimientos, { $eventos } eventos
b3-resumo-linhas = { $linhas } filas en el archivo
b3-resumo-ativos = Posición antes y después de la importación
b3-aba-abertura = Apertura
b3-abertura-texto = El archivo empieza después de tu historial: estas posiciones aparecen en los rendimientos, en la fotografía de la custodia o en la posición de la B3. Marca las que quieras registrar el día anterior al archivo, con el costo medio, y rehaz la vista previa.
b3-abertura-tabela = Posiciones de apertura sugeridas
b3-abertura-incluir = Incluir { $ativo }
b3-abertura-custo = Costo medio
b3-abertura-custo-de = Costo medio de { $ativo }
b3-abertura-evidencia = De dónde viene
b3-abertura-aplicar = Rehacer la vista previa con las posiciones marcadas
b3-custo-operacao = Costo unitario de la operación { $n }
b3-custo = Costo
b3-linhas = Filas
b3-conta = Cuenta
b3-conta-para = Cuenta para { $instituicao }
b3-contas = Cuentas
b3-incluir-possiveis = Importar también los { $n } posibles duplicados
b3-incluir-possiveis-dica = Misma fecha, activo y cantidad (o valor) que un movimiento ingresado a mano.
b3-lancar-caixa = Registrar en la caja de la corredora
b3-lancar-caixa-dica = Apagado (predeterminado): la B3 no informa la caja ni las comisiones de la corredora; importa el extracto de la cuenta por separado. Encendido: compras, ventas y rendimientos mueven la caja.
b3-secoes = Secciones de la vista previa
b3-aba-ativos = Activos
b3-aba-operacoes = Operaciones
b3-aba-proventos = Rendimientos
b3-aba-eventos = Eventos
b3-aba-ignoradas = Omitidas
b3-aba-posicao = Posición
b3-ativos-texto = { $n ->
    [one] { $n } activo a registrar.
   *[other] { $n } activos a registrar.
} La clase sale de la hoja de posición, del final del código o del nombre; revisa los marcados.
b3-ativos-tabela = Activos de la importación
b3-ativo = Activo
b3-situacao = Situación
b3-cadastrar = Registrar
b3-ja-cadastrado = Ya registrado
b3-conferir = Revisar
b3-classe = Clase
b3-classe-de = Clase de { $ativo }
b3-usar-cadastrado = Usar activo ya registrado
b3-ativo-cadastrado-para = Activo registrado para { $ativo }
b3-cadastrar-novo = No: registrar uno nuevo
b3-por-que = Por qué
b3-filtrar-operacoes = Filtrar operaciones por situación
b3-operacoes-tabela = Operaciones de la importación
b3-importar = Importar
b3-data = Fecha
b3-tipo = Tipo
b3-qtd = Cant.
b3-preco = Precio
b3-valor = Valor
b3-filtrar-proventos = Filtrar rendimientos por situación
b3-proventos-tabela = Rendimientos de la importación
b3-pagamento = Pago
b3-bruto = Bruto
b3-liquido = Neto
b3-filtrar-eventos = Filtrar eventos por situación
b3-eventos-tabela = Eventos corporativos de la importación
b3-aprovar = Aprobar
b3-evento = Evento
b3-no-arquivo = En el archivo
b3-posicao-antes = Posición anterior
b3-fator = Factor
b3-fator-do-evento = Factor del evento { $n }
b3-fator-exemplo = ej.: 0,1
b3-custo-acao = Costo por acción
b3-custo-acao-evento = Costo por acción del evento { $n }
b3-eventos-revisao = Los eventos en revisión solo se guardan si se aprueban, con el factor (nueva cantidad = anterior × factor; en la bonificación, acciones nuevas por acción, ej.: 0,1 = 10 %).
b3-ignoradas-texto = Filas que no se convierten en movimiento, con la regla que las dejó fuera.
b3-ignoradas-tabela = Filas omitidas
b3-origem = Origen
b3-regra = Regla
b3-linha = Fila
b3-motivo = Motivo
b3-posicao-texto = Posición de la B3 al { $data } × cantidad calculada por la cartera con esta importación: { $conferem } coinciden, { $divergem } difieren.
b3-posicao-tabela = Verificación con la posición de la B3
b3-carteira = Cartera
b3-diferenca = Diferencia
b3-resultado = Importados desde la B3: { $operacoes ->
    [one] { $operacoes } operación
   *[other] { $operacoes } operaciones
}, { $proventos ->
    [one] { $proventos } rendimiento
   *[other] { $proventos } rendimientos
} y { $eventos ->
    [one] { $eventos } evento
   *[other] { $eventos } eventos
}; { $ativos ->
    [one] { $ativos } activo registrado
   *[other] { $ativos } activos registrados
}. { $ignoradas ->
    [one] { $ignoradas } fila quedó fuera.
   *[other] { $ignoradas } filas quedaron fuera.
}
b3-linhas-erro = Filas con error
b3-linha-erro = Fila { $n } de la vista previa: { $mensagem }
b3-backup-antes = Copia de seguridad antes
b3-backup-feito = hecha
b3-backup-nao-precisou = no hizo falta
b3-desfazer-rotulo = Deshacer
b3-desfazer-como = con el botón de abajo o en Ajustes › Historial

## Notas de corretaje

b3-notas-titulo = Importar notas de corretaje
b3-etapa-notas = Notas
b3-nota-aria = Nota { $nota }
b3-nota-titulo = Nota { $nota }
b3-sem-numero = sin número
b3-pregao = · rueda del { $data }
b3-nota-arquivo = { $arquivo } · { $n ->
    [one] { $n } operación
   *[other] { $n } operaciones
}
b3-custo-liquidacao = Liquidación
b3-custo-registro = Registro
b3-custo-emolumentos = Derechos de bolsa
b3-custo-corretagem = Corretaje
b3-custo-outras = Otras
b3-custo-total = Total de comisiones
b3-liquido-nota = Neto de la nota
b3-liquido-calculado = Neto calculado
b3-liquido-diferente = Neto de la nota ≠ neto calculado: puede que alguna línea de la nota no se haya leído. Revisa las operaciones contra la nota antes de guardar.
b3-avisos-nota = Avisos de la nota { $nota }
b3-ativo-para = Activo para { $titulo }
b3-ativo-cadastrado = Activo registrado
b3-ou-codigo = o el código
b3-codigo-para = Código para { $titulo }
b3-usar-codigo = Usar código
b3-voltar-automatico = Volver a automático
b3-notas-gravadas = { $n ->
    [one] { $n } operación guardada
   *[other] { $n } operaciones guardadas
}
b3-ler-notas = Leer notas
b3-gravar-n = { $n ->
    [one] Guardar { $n } operación
   *[other] Guardar { $n } operaciones
}
b3-notas-desfeita = Importación de las notas deshecha
b3-conta-corretora = Cuenta de la corredora
b3-conta-corretora-dica = Las notas SINACOR son de cuentas en reales. Las operaciones van a esta cuenta.
b3-notas-pdf = Notas de corretaje (PDF)
b3-notas-pdf-dica = Una o más notas en el formato SINACOR. Los archivos quedan solo en la memoria de la app hasta que guardes.
b3-arquivos-escolhidos = Archivos elegidos
b3-lendo-notas = Leyendo las notas…
b3-arquivos = Archivos
b3-notas-lidas = Notas leídas
b3-arquivos-nao-lidos = Archivos no leídos
b3-senha-pdf = Contraseña del PDF
b3-senha-pdf-dica = Muchas corredoras usan los 3 primeros dígitos del CPF (identificación fiscal brasileña) del titular. La contraseña solo abre los archivos ahora: no se guarda.
b3-abrir-senha = Abrir con la contraseña
b3-nada-gravado = No se guarda nada mientras algún archivo no se pueda leer; quita el archivo de la lista (Volver) o resuelve el problema.
b3-nao-identificados = Activos no identificados
b3-nao-identificados-texto = La nota trae el nombre de negociación, no el código. Elige el activo de cada título o ingresa el código (se registra al guardar); sin eso, sus operaciones quedan fuera.
b3-inconsistencias = Inconsistencias que no impiden guardar
b3-inconsistencia = { $data } · { $ativo }: { $aviso }. Se guarda de todos modos; revísalo después en la cartera.
b3-agrupar = Agrupar ejecuciones de la misma orden
b3-agrupar-dica = La corredora divide una orden en varias ejecuciones. Encendido (predeterminado): las del mismo título, lado y mercado en la nota se convierten en una operación con el precio promedio y los costos sumados.
b3-notas-operacoes-tabela = Operaciones de las notas
b3-gravar = Guardar
b3-gravar-operacao = Guardar operación { $n }
b3-nota = Nota
b3-execucoes = { $n } ejecuciones
b3-titulo-coluna = Título
b3-cadastra = registra { $ativo } · { $nome }
b3-corrigido = corregido por ti
b3-corrigir-ativo = Corregir activo
b3-taxas = Comisiones
b3-notas-lancar-caixa-dica = Apagado (predeterminado): el extracto de la cuenta suele importarse por separado, y registrar aquí duplicaría la caja. Encendido: cada operación nueva crea la salida o la entrada en la cuenta.
b3-notas-resultado = { $novas ->
    [one] { $novas } operación nueva
   *[other] { $novas } operaciones nuevas
} y { $completadas } completadas con los costos en { $conta }; { $ignoradas } quedaron fuera.
b3-ativos-cadastrados = Activos registrados
