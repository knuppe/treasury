# Pantalla Agente y las piezas del agente que también usa el panel lateral: conversación,
# cuadro de mensaje, cola de aprobación, revisión de cambios, alertas, memoria,
# procedimientos, historial de herramientas y ajustes de los motores de IA
# (docs/i18n-design.md). Revisión: borrador por IA, sin revisión nativa.

## Pantalla y pestañas

agente-aba-conversa = Conversación
agente-aba-aprovacao = Aprobación
agente-aba-alertas = Alertas
agente-aba-ferramentas = Herramientas
agente-aba-memoria = Memoria
agente-aba-configurar = Configurar
agente-secoes = Secciones del agente
agente-cmd-aba = Agente: { $aba }
agente-cmd-busca = agente,ia
agente-configurar-motores = Configurar motores de IA
agente-n-pendentes = ({ $n ->
        [one] { $n } pendiente
       *[other] { $n } pendientes
    })
agente-n-novos = ({ $n ->
        [one] { $n } nueva
       *[other] { $n } nuevas
    })
agente-motores-ligados = { $n ->
        [one] { $n } motor activo
       *[other] { $n } motores activos
    }
agente-badge-propostas = { $n ->
        [one] { $n } propuesta pendiente
       *[other] { $n } propuestas pendientes
    }
agente-badge-alertas = { $n ->
        [one] { $n } alerta nueva
       *[other] { $n } alertas nuevas
    }

## Números, tiempos y comillas

agente-tokens-mil = { NUMBER($n, maximumFractionDigits: 1) } mil
agente-tokens-mi = { NUMBER($n, maximumFractionDigits: 1) } M
agente-menos-1-s = menos de 1 s
agente-segundos = { $s } s
agente-minutos = { $m } min
agente-minutos-segundos = { $m } min { $s } s
agente-aspas = “{ $texto }”
agente-lista-e = { $inicio } y { $fim }
agente-lista-e-mais = { $inicio } y { $n } más
agente-sim = sí
agente-nao = no

## Tareas, tipos de motor y estados

agente-tarefa-chat = Chat con el agente
agente-tarefa-categorizacao = Categorización
agente-tarefa-pdf = Lectura de PDF
agente-tarefa-fechamento = Cierre mensual
agente-tarefa-alertas = Alertas
agente-tarefa-estudo = Estudio de activos
agente-tipo-motor-api = API
agente-tipo-motor-cli = CLI local
agente-tipo-motor-local = Modelo local
agente-destino-api = datos enviados a la API del proveedor
agente-destino-cli = los datos pasan por la CLI local (y su proveedor)
agente-destino-local = los datos se quedan en esta computadora
agente-nivel-reversivel = Reversible · en lote
agente-nivel-sensivel = Sensible · una por una
agente-proposta-pendente = Pendiente
agente-proposta-aprovada = Aprobada
agente-proposta-rejeitada = Rechazada
agente-proposta-falhou = Falló
agente-proposta-retirada = Retirada
agente-alerta-novo = Nueva
agente-alerta-visto = Vista
agente-alerta-resolvido = Resuelta
agente-gravidade-info = Información
agente-gravidade-atencao = Atención
agente-gravidade-critico = Crítica
agente-job-andamento = En curso
agente-job-concluido = Terminado
agente-job-falhou = Falló
agente-job-cancelado = Cancelado
agente-memoria-perfil = Perfil de inversionista
agente-memoria-regra = Regla de la familia
agente-memoria-nota = Nota
agente-problema-desligado = motor apagado
agente-problema-sem-chave = sin clave de API guardada
agente-problema-sem-template = sin plantilla de comando
agente-problema-sem-modelo = sin modelo definido

## Comandos de escritura propuestos

agente-comando-criar-lancamento = Crear movimiento
agente-comando-alterar-lancamento = Modificar movimiento
agente-comando-categorizar = Categorizar movimientos
agente-comando-dividir = Dividir movimiento
agente-comando-criar-transferencia = Crear transferencia
agente-comando-apagar-lancamento = Borrar movimiento
agente-comando-apagar-feedback = Eliminar comentario al desarrollador
agente-comando-descartar-feedback = Descartar comentario al desarrollador
agente-comando-criar-etiqueta = Crear etiqueta
agente-comando-criar-categoria = Crear categoría
agente-comando-alterar-categoria = Modificar categoría
agente-comando-cadastrar-instituicao = Registrar institución
agente-comando-alterar-instituicao = Modificar institución
agente-comando-cadastrar-conta = Registrar cuenta o tarjeta
agente-comando-alterar-conta = Modificar cuenta
agente-comando-alterar-recorrencia = Modificar recurrencia
agente-comando-alterar-ativo = Modificar activo
agente-comando-cadastrar-bem = Registrar bien físico
agente-comando-alterar-bem = Modificar bien físico
agente-comando-avaliacao-bem = Registrar avalúo del bien
agente-comando-benfeitoria = Registrar mejora
agente-comando-criar-regra = Crear regla
agente-comando-alterar-regra = Modificar regla
agente-comando-aplicar-regra = Aplicar regla
agente-comando-criar-recorrencia = Crear recurrencia
agente-comando-definir-orcamento = Definir presupuesto
agente-comando-copiar-orcamento = Copiar presupuesto
agente-comando-criar-meta = Crear meta
agente-comando-alterar-meta = Modificar meta
agente-comando-cadastrar-ativo = Registrar activo
agente-comando-acompanhar-ativo = Seguir activo
agente-comando-registrar-operacao = Registrar operación
agente-comando-registrar-provento = Registrar dividendo
agente-comando-proventos-ticker = Registrar dividendos por ticker
agente-comando-evento-corporativo = Registrar evento corporativo
agente-comando-importar-extrato = Importar extracto
agente-comando-importar-lidos = Importar movimientos leídos por el agente
agente-comando-investimentos-lote = Registrar inversiones en lote
agente-comando-importar-b3 = Importar de B3
agente-comando-gravar-memoria = Guardar en la memoria del gestor

## Temas de los procedimientos

agente-assunto-extrato = Extracto
agente-assunto-fatura = Estado de cuenta de tarjeta
agente-assunto-investimentos = Inversiones
agente-assunto-proventos = Dividendos
agente-assunto-cotacoes = Cotizaciones e indicadores
agente-assunto-imposto = Impuesto sobre la renta
agente-assunto-cadastro = Registro
agente-assunto-outro = Otro

## Tablas de la base de datos en los cambios

agente-tabela-lancamento = Movimiento
agente-tabela-divisao = División de movimiento
agente-tabela-etiqueta-lancamento = Etiqueta del movimiento
agente-tabela-etiqueta = Etiqueta
agente-tabela-categoria = Categoría
agente-tabela-regra = Regla
agente-tabela-aplicacao-regra = Aplicación de regla
agente-tabela-recorrencia = Recurrencia
agente-tabela-orcamento = Presupuesto
agente-tabela-linha-orcamento = Línea del presupuesto
agente-tabela-meta = Meta
agente-tabela-conta-meta = Cuenta de la meta
agente-tabela-projeto = Proyecto
agente-tabela-ativo = Activo
agente-tabela-operacao = Operación
agente-tabela-provento = Dividendo
agente-tabela-evento-corporativo = Evento corporativo
agente-tabela-acompanhamento = Seguimiento
agente-tabela-lote-importacao = Lote de importación
agente-tabela-memoria = Memoria del gestor
agente-tabela-procedimento = Procedimiento del agente
agente-tabela-feedback = Comentario al desarrollador
agente-tabela-nota-feedback = Nota de comentario
agente-tabela-vinculo-feedback = Vínculo entre comentarios
agente-tabela-conta = Cuenta
agente-tabela-membro-conta = Miembro de la cuenta
agente-tabela-cartao = Tarjeta de crédito
agente-tabela-instituicao = Institución
agente-tabela-renda-fixa = Renta fija
agente-tabela-bem = Bien físico
agente-tabela-dono-bem = Miembro dueño del bien
agente-tabela-custo-aquisicao = Costo de adquisición
agente-tabela-benfeitoria = Mejora
agente-tabela-avaliacao-bem = Avalúo del bien
agente-tabela-financiamento-bem = Financiamiento del bien

## Conteo por tabla

agente-n-lancamento = { $n ->
        [one] { $n } movimiento
       *[other] { $n } movimientos
    }
agente-n-divisao = { $n ->
        [one] { $n } división de movimiento
       *[other] { $n } divisiones de movimiento
    }
agente-n-etiqueta-lancamento = { $n ->
        [one] { $n } etiqueta del movimiento
       *[other] { $n } etiquetas del movimiento
    }
agente-n-etiqueta = { $n ->
        [one] { $n } etiqueta
       *[other] { $n } etiquetas
    }
agente-n-categoria = { $n ->
        [one] { $n } categoría
       *[other] { $n } categorías
    }
agente-n-regra = { $n ->
        [one] { $n } regla
       *[other] { $n } reglas
    }
agente-n-aplicacao-regra = { $n ->
        [one] { $n } aplicación de regla
       *[other] { $n } aplicaciones de regla
    }
agente-n-recorrencia = { $n ->
        [one] { $n } recurrencia
       *[other] { $n } recurrencias
    }
agente-n-orcamento = { $n ->
        [one] { $n } presupuesto
       *[other] { $n } presupuestos
    }
agente-n-linha-orcamento = { $n ->
        [one] { $n } línea del presupuesto
       *[other] { $n } líneas del presupuesto
    }
agente-n-meta = { $n ->
        [one] { $n } meta
       *[other] { $n } metas
    }
agente-n-conta-meta = { $n ->
        [one] { $n } cuenta de la meta
       *[other] { $n } cuentas de la meta
    }
agente-n-projeto = { $n ->
        [one] { $n } proyecto
       *[other] { $n } proyectos
    }
agente-n-ativo = { $n ->
        [one] { $n } activo
       *[other] { $n } activos
    }
agente-n-operacao = { $n ->
        [one] { $n } operación
       *[other] { $n } operaciones
    }
agente-n-provento = { $n ->
        [one] { $n } dividendo
       *[other] { $n } dividendos
    }
agente-n-evento-corporativo = { $n ->
        [one] { $n } evento corporativo
       *[other] { $n } eventos corporativos
    }
agente-n-acompanhamento = { $n ->
        [one] { $n } seguimiento
       *[other] { $n } seguimientos
    }
agente-n-lote-importacao = { $n ->
        [one] { $n } lote de importación
       *[other] { $n } lotes de importación
    }
agente-n-memoria = { $n ->
        [one] { $n } nota de la memoria del gestor
       *[other] { $n } notas de la memoria del gestor
    }
agente-n-procedimento = { $n ->
        [one] { $n } procedimiento del agente
       *[other] { $n } procedimientos del agente
    }
agente-n-feedback = { $n ->
        [one] { $n } comentario al desarrollador
       *[other] { $n } comentarios al desarrollador
    }
agente-n-nota-feedback = { $n ->
        [one] { $n } nota de comentario
       *[other] { $n } notas de comentario
    }
agente-n-vinculo-feedback = { $n ->
        [one] { $n } vínculo entre comentarios
       *[other] { $n } vínculos entre comentarios
    }
agente-n-conta = { $n ->
        [one] { $n } cuenta
       *[other] { $n } cuentas
    }
agente-n-membro-conta = { $n ->
        [one] { $n } miembro de la cuenta
       *[other] { $n } miembros de la cuenta
    }
agente-n-cartao = { $n ->
        [one] { $n } tarjeta de crédito
       *[other] { $n } tarjetas de crédito
    }
agente-n-instituicao = { $n ->
        [one] { $n } institución
       *[other] { $n } instituciones
    }
agente-n-renda-fixa = { $n ->
        [one] { $n } renta fija
       *[other] { $n } rentas fijas
    }
agente-n-bem = { $n ->
        [one] { $n } bien físico
       *[other] { $n } bienes físicos
    }
agente-n-dono-bem = { $n ->
        [one] { $n } miembro dueño del bien
       *[other] { $n } miembros dueños del bien
    }
agente-n-custo-aquisicao = { $n ->
        [one] { $n } costo de adquisición
       *[other] { $n } costos de adquisición
    }
agente-n-benfeitoria = { $n ->
        [one] { $n } mejora
       *[other] { $n } mejoras
    }
agente-n-avaliacao-bem = { $n ->
        [one] { $n } avalúo del bien
       *[other] { $n } avalúos del bien
    }
agente-n-financiamento-bem = { $n ->
        [one] { $n } financiamiento del bien
       *[other] { $n } financiamientos del bien
    }
agente-n-alteracoes = { $n ->
        [one] { $n } cambio
       *[other] { $n } cambios
    }

## Resumen y frases de lo que cambia

agente-resumo-cria = crea { $itens }
agente-resumo-altera = modifica { $itens }
agente-resumo-apaga = borra { $itens }
agente-frase-cria = Crea { $itens }: { $nomes }
agente-frase-apaga = Borra { $itens }: { $nomes }
agente-frase-registra = Registra { $itens }: { $nomes }
agente-frase-grava = Guarda { $itens }: { $nomes }
agente-frase-acompanha = Sigue { $itens }: { $nomes }
agente-frase-inclui = Agrega { $itens }: { $nomes }
agente-frase-poe = Pone { $itens }: { $nomes }
agente-frase-altera = Cambia { $campo } de { $alvo }
agente-frase-altera-para = Cambia { $campo } de { $alvo } a { $para }
agente-frase-altera-de-para = Cambia { $campo } de { $alvo }: { $de } → { $para }
agente-artigo-campo = el campo
agente-artigo-categoria = la categoría
agente-artigo-valor = el monto
agente-artigo-membro = el miembro
agente-artigo-conta = la cuenta
agente-artigo-situacao = el estado
agente-artigo-tipo = el tipo
agente-artigo-nome = el nombre
agente-artigo-descricao = la descripción
agente-artigo-favorecido = el beneficiario
agente-artigo-notas = las notas
agente-artigo-data = la fecha
agente-artigo-ativacao = la activación
agente-artigo-arquivamento = el archivado
agente-artigo-prioridade = la prioridad
agente-artigo-categoria-pai = la categoría padre
agente-artigo-instituicao = la institución
agente-artigo-essencial = el “esencial”
agente-artigo-grupo-gasto = el grupo de gasto
agente-artigo-condicoes = las condiciones
agente-artigo-acoes = las acciones
agente-artigo-cor = el color

## Lectura amigable de los cambios: operaciones, títulos y valores

agente-op-criacao = Creación
agente-op-inclusao = Inclusión
agente-op-alteracao = Modificación
agente-op-exclusao = Eliminación
agente-lado-compra = Compra
agente-lado-venda = Venta
agente-sem-categoria = Sin categoría
agente-sem-membro = Sin miembro
agente-sem-pai = Ninguna (raíz)
agente-sem-descricao = sin descripción
agente-lancamento-sem-descricao = Movimiento sin descripción
agente-outro-lancamento = otro movimiento
agente-nao-encontrado = { $ref } (no encontrado)
agente-ref-categoria = categoría
agente-ref-conta = cuenta
agente-ref-membro = miembro
agente-ref-instituicao = institución
agente-ref-ativo = activo
agente-ref-contrato = contrato
agente-ref-etiqueta = etiqueta
agente-ref-regra = regla
agente-ref-meta = meta
agente-ref-recorrencia = recurrencia
agente-ref-lancamento = movimiento
agente-ref-bem = bien
agente-regra-categoria = categoría { $nome }
agente-regra-membro = miembro { $nome }
agente-regra-etiquetas = etiquetas { $etiquetas }
agente-regra-favorecido = beneficiario “{ $nome }”
agente-regra-tipo = tipo { $tipo }
agente-regra-nenhuma = ninguna
agente-titulo-etiqueta-em = #{ $etiqueta } en { $lancamento }
agente-titulo-parte-de = Parte de { $lancamento }
agente-titulo-de = { $tipo } de { $ativo }
agente-titulo-meta = meta { $nome }
agente-operacao = Operación
agente-provento = Dividendo
agente-evento = Evento
agente-nota-gestor = Nota del gestor

## Campos de los cambios

agente-campo-nome = Nombre
agente-campo-nome-completo = Nombre completo
agente-campo-nome-curto = Nombre corto
agente-campo-tipo = Tipo
agente-campo-classe = Clase
agente-campo-moeda = Moneda
agente-campo-valor = Monto
agente-campo-data = Fecha
agente-campo-situacao = Estado
agente-campo-descricao = Descripción
agente-campo-favorecido = Beneficiario
agente-campo-notas = Notas
agente-campo-nota = Nota
agente-campo-conteudo = Contenido
agente-campo-titulo = Título
agente-campo-categoria = Categoría
agente-campo-categoria-pai = Categoría padre
agente-campo-membro = Miembro
agente-campo-conta = Cuenta
agente-campo-conta-destino = Cuenta de destino
agente-campo-conta-debito = Cuenta de débito
agente-campo-conta-pagamento = Cuenta de pago
agente-campo-instituicao = Institución
agente-campo-emissor = Emisor
agente-campo-ativo = Activo
agente-campo-ativo-novo = Activo nuevo
agente-campo-contrato = Contrato
agente-campo-etiqueta = Etiqueta
agente-campo-regra = Regla
agente-campo-meta = Meta
agente-campo-recorrencia = Recurrencia
agente-campo-lancamento = Movimiento
agente-campo-transferencia = Transferencia
agente-campo-fatura = Estado de cuenta
agente-campo-parcelamento = Plan de cuotas
agente-campo-parcela = Cuota
agente-campo-parcelas = Cuotas
agente-campo-orcamento = Presupuesto
agente-campo-bem = Bien
agente-campo-arquivado = Archivado
agente-campo-ativa = Activa
agente-campo-ligada = Activada
agente-campo-essencial = Esencial
agente-campo-grupo-gasto = Grupo de gasto
agente-campo-principal = Principal
agente-campo-patrimonio = Cuenta en el patrimonio
agente-campo-todas-condicoes = Todas las condiciones
agente-campo-isento-ir = Exento de impuesto sobre la renta
agente-campo-day-trade = Day trade
agente-campo-fixado = Fijado
agente-campo-saldo-inicial = Saldo inicial
agente-campo-data-inicial = Fecha inicial
agente-campo-limite = Límite
agente-campo-dia-fechamento = Día de corte
agente-campo-dia-vencimento = Día de vencimiento
agente-campo-total-informado = Total informado
agente-campo-valor-total = Monto total
agente-campo-data-compra = Fecha de compra
agente-campo-primeira-parcela = Primera cuota
agente-campo-prioridade = Prioridad
agente-campo-condicoes = Condiciones
agente-campo-acoes = Acciones
agente-campo-repeticao = Repetición
agente-campo-inicio = Inicio
agente-campo-fim = Fin
agente-campo-taxa = Tasa
agente-campo-taxa-ano = Tasa anual
agente-campo-taxa-mes = Tasa mensual
agente-campo-indexador = Indexador
agente-campo-pct-indexador = % del indexador
agente-campo-liquidez = Liquidez
agente-campo-emissao = Emisión
agente-campo-vencimento = Vencimiento
agente-campo-carencia = Período de gracia hasta
agente-campo-ticker = Ticker
agente-campo-mercado = Mercado
agente-campo-pais = País
agente-campo-setor = Sector
agente-campo-segmento = Segmento
agente-campo-fonte-cotacao = Fuente de la cotización
agente-campo-simbolo-fonte = Símbolo en la fuente
agente-campo-operacao = Operación
agente-campo-quantidade = Cantidad
agente-campo-preco = Precio
agente-campo-preco-unitario = Precio unitario
agente-campo-custo-unitario = Costo unitario
agente-campo-custos = Costos
agente-campo-impostos = Impuestos
agente-campo-cambio = Tipo de cambio
agente-campo-nota-corretagem = Nota de corretaje
agente-campo-fator = Factor
agente-campo-data-com = Fecha de corte
agente-campo-pagamento = Pago
agente-campo-bruto = Bruto
agente-campo-liquido = Neto
agente-campo-ir-retido = Impuesto retenido
agente-campo-preco-alvo = Precio objetivo
agente-campo-alerta-acima = Alerta por encima de
agente-campo-alerta-abaixo = Alerta por debajo de
agente-campo-valor-meta = Monto de la meta
agente-campo-prazo = Plazo
agente-campo-valor-inicial = Monto inicial
agente-campo-aporte-mensal = Aporte mensual
agente-campo-valor-financiado = Monto financiado
agente-campo-prazo-meses = Plazo (meses)
agente-campo-sistema = Sistema
agente-campo-data-contrato = Fecha del contrato
agente-campo-numero-contrato = Número del contrato
agente-campo-primeiro-vencimento = Primer vencimiento
agente-campo-cor = Color
agente-campo-icone = Ícono
agente-campo-ordem = Orden
agente-campo-mes = Mes
agente-campo-custo = Costo
agente-campo-origem = Origen
agente-campo-dominio = Dominio
agente-campo-rotulo = Etiqueta
agente-campo-assunto = Tema
agente-campo-passos = Pasos
agente-campo-agencia = Sucursal

## Revisión de cambios y diff exacto

agente-campo = Campo
agente-valor = Valor
agente-antes = Antes
agente-depois = Después
agente-para = a
agente-nenhuma-linha-alteracao = Ninguna fila de la base de datos cambia con este cambio.
agente-nenhuma-linha-proposta = Ninguna fila de la base de datos cambia con esta propuesta.
agente-o-que-muda = Lo que cambia
agente-mostrar-linhas = Mostrar las { $n } filas
agente-ver-tecnicos = Ver detalles técnicos
agente-ocultar-tecnicos = Ocultar detalles técnicos
agente-diff-exato = Diff exacto de lo que se guardará: tablas, ids y valores en bruto (dinero en centavos).
agente-so-o-que-muda = Solo lo que cambia
agente-mostrar-campos = Mostrar todos los campos (+{ $n })

## Conversación

agente-motor-em-uso = Motor en uso
agente-proximo-motor = Próximo motor
agente-motor-em-uso-nome = Motor en uso: { $motor }
agente-proximo-motor-nome = Próximo motor: { $motor }
agente-nenhum-motor-dica = Ningún motor listo: configura uno en Ajustes › IA y agente
agente-motor-recebe = Motor que recibe los datos
agente-respondendo = Respondiendo
agente-recebendo = Recibiendo
agente-nenhum-motor-pronto = ningún motor listo
agente-proposta-criada = Propuesta creada: { $nome }
agente-aguarda-aprovacao = espera aprobación
agente-revisar-fila = Revisar en la cola
agente-fallback = { $motor } no respondió ({ $motivo }); probando el siguiente motor de la ruta.
agente-agora-com = Ahora con { $motor } ({ $tipo }).
agente-uso-resposta = Uso de esta respuesta
agente-tokens-entrada = { $n } tokens de entrada
agente-tokens-saida = { $n } de salida
agente-custo = costo { $valor }
agente-custo-parcial = costo { $valor } (parcial: modelo sin precio)
agente-voce = Tú
agente-motor-generico = motor
agente-alteracao = cambio
agente-sem-motor-dica = Registra un motor (API, CLI local o modelo local) y define la ruta de chat en { $link }.
agente-sem-motor-titulo = Ningún motor de IA configurado
agente-nao-respondeu = El agente no respondió
agente-aguardando-motor = esperando el motor
agente-resposta-cancelada = Respuesta cancelada.
agente-conversas = Conversaciones
agente-nova = Nueva
agente-nenhuma-conversa = Ninguna conversación guardada.
agente-confirmar-custo = Confirmar costo
agente-custo-texto = El primer turno con **{ $motor }** debería costar cerca de **{ $custo }**, por encima del aviso de { $limite } definido en los límites de gasto.
agente-enviar-mesmo-assim = Enviar de todos modos
agente-automatico = Automático (ruta de chat)
agente-conversa-com = Conversación con el agente
agente-historico-desligado = Historial apagado: cada pregunta empieza de cero y no se guarda nada.
agente-vazio-titulo = Pregunta sobre gastos, presupuesto, deudas o la cartera.
agente-vazio-texto = El agente usa las herramientas de Treasury; toda escritura espera tu aprobación en la cola. Las recomendaciones de inversión son análisis de tus datos, no asesoría profesional.
agente-vazio-texto-autonomo = El agente usa las herramientas de Treasury. Con el modo autónomo activado, las escrituras reversibles se aplican al instante y se pueden deshacer en el historial; las sensibles esperan tu aprobación. Las recomendaciones de inversión son análisis de tus datos, no asesoría profesional.
agente-mensagem = Mensaje para el agente
agente-placeholder = Ej.: categoriza los movimientos sin categoría de este mes
agente-motor = Motor
agente-enter-envia = Enter envía · Shift+Enter agrega una línea
agente-enviar = Enviar
agente-enviar-assim-mesmo = Enviar de todos modos
agente-erro-sem-job = El núcleo no devolvió el trabajo de la conversación.

## Actividad de la respuesta (razonamiento y herramientas)

agente-pensando = Pensando…
agente-raciocinio = Razonamiento
agente-raciocinio-tempo = Razonamiento · { $tempo }
agente-consultando = Consultando { $ferramenta }…
agente-consultou = Consultó { $n ->
        [one] { $n } herramienta
       *[other] { $n } herramientas
    }
agente-consultou-erro = Consultó { $n ->
        [one] { $n } herramienta
       *[other] { $n } herramientas
    } · { $erros } con error
agente-ferramenta-nome = Herramienta { $nome }
agente-ok = ok
agente-erro = error
agente-ferramenta-cli = Herramienta propia de la CLI, fuera de Treasury
agente-atividade = Actividad del agente
agente-raciocinio-agente = Razonamiento del agente
agente-ferramentas-usadas = Herramientas usadas

## Conversaciones guardadas

agente-conversa-meta = { $quando } · { $n } msj
agente-fixar = Fijar la conversación { $titulo }
agente-desafixar = Desfijar la conversación { $titulo }
agente-fixada-dica = Fijada: la limpieza automática no la toca
agente-fixar-dica = Fijar: la limpieza automática no la borra
agente-apagar-conversa = Borrar la conversación { $titulo }
agente-nao-apagou = No se pudo borrar
agente-apagar-todas = Borrar todas las conversaciones
agente-apagar-todas-texto = El historial de las conversaciones sale de la base de datos. Las propuestas, las alertas y la memoria del gestor se quedan.
agente-apagar-historico = Borrar historial
agente-apagar-tudo = Borrar todo

## Computadora

agente-computador-ligado-dica = En esta conversación, las CLI locales de agentes pueden leer archivos de tu carpeta personal cuando lo pidas. Terminal, escritura y web siguen bloqueados; los motores de API y locales siguen solo con Treasury.
agente-computador-desligado-dica = El agente usa solo las herramientas de Treasury y no mira tus archivos, apps ni navegador. Actívalo para dejar que una CLI lea un archivo que indiques en esta conversación.
agente-computador-rotulo = Acceso a la computadora en esta conversación
agente-computador-desligado = Acceso a la computadora desactivado
agente-computador-liberado = Lectura de la computadora permitida
agente-computador-volta = El agente vuelve a usar solo Treasury.
agente-computador = Computadora
agente-computador-leitura = Computadora: lectura permitida
agente-computador-bloqueado = Computadora: bloqueada

## Esfuerzo de razonamiento

agente-esforco-padrao = Predeterminado
agente-esforco-baixo = Bajo
agente-esforco-medio = Medio
agente-esforco-alto = Alto
agente-esforco-maximo = Máximo
agente-esforco-padrao-dica = lo que usa el motor
agente-esforco-baixo-dica = más rápido
agente-esforco-medio-dica = equilibrio
agente-esforco-alto-dica = piensa más
agente-esforco-maximo-dica = más lento y caro
agente-esforco-dica = Cuánto razona el modelo antes de responder en esta conversación. Más esfuerzo suele dar mejores respuestas en preguntas difíciles, con más demora y costo.
agente-esforco-sem-controle = { $motor } no tiene control de esfuerzo con este modelo; vale su valor predeterminado.
agente-este-motor = Este motor
agente-esforco-rotulo = Esfuerzo de razonamiento en esta conversación
agente-esforco-valor = Esfuerzo: { $nivel }

## Adjuntos

agente-anexo-colado = pegado.png
agente-anexos-a-mais = { $n ->
        [one] { $n } archivo de más
       *[other] { $n } archivos de más
    }
agente-anexos-maximo = como máximo { $max } adjuntos por mensaje
agente-anexo-grande = archivo demasiado grande (máx. 20 MB)
agente-anexo-paginas = { $n } pág.
agente-anexo-caracteres = { $n } caracteres
agente-anexo-escaneado = sin texto (escaneado)
agente-anexo-cortado = texto cortado
agente-anexar-dica = Adjunta imágenes, PDF, hojas de cálculo o textos. También puedes arrastrar archivos aquí o pegar con ⌘V / Ctrl+V.
agente-anexar-arquivos = Adjuntar archivos
agente-anexar = Adjuntar
agente-remover = Quitar { $nome }
agente-anexos-mensagem = Adjuntos del mensaje
agente-anexo-preparando = preparando…
agente-falhou = falló
agente-anexo-nao-lido = el motor no lee este adjunto
agente-anexo-motor-nao-le = { $motor } no lee { $tipo ->
        [imagem] imágenes
       *[pdf] PDF escaneados
    }. Si lo envías así, el agente recibe solo el nombre del archivo. Elige un motor con visión (de API o un modelo local con visión) o quita el adjunto.
agente-anexo-rota-nao-le = Ningún motor de la ruta de chat lee { $tipo ->
        [imagem] imágenes
       *[pdf] PDF escaneados
    } de este tipo. Si lo envías así, el agente recibe solo el nombre del archivo. Elige un motor con visión (de API o un modelo local con visión) o quita el adjunto.
agente-anexos = Adjuntos
agente-anexo-nao-guardado = { $detalhe } · no guardado
agente-anexo-guardado-dica = Guardado en la bóveda según la retención de adjuntos.
agente-anexo-nao-guardado-dica = El contenido se envió al motor y no quedó guardado (retención de adjuntos).
agente-solte = Suelta para adjuntar

## Cola de aprobación

agente-retiradas = { $n ->
        [one] { $n } propuesta ya había salido de la cola
       *[other] { $n } propuestas ya habían salido de la cola
    }
agente-retiradas-texto = Los mismos datos cambiaron antes de la aprobación; el cambio que hiciste se queda.
agente-aprovadas = { $n ->
        [one] { $n } propuesta aprobada
       *[other] { $n } propuestas aprobadas
    }
agente-rejeitadas = { $n ->
        [one] { $n } propuesta rechazada
       *[other] { $n } propuestas rechazadas
    }
agente-aprovadas-texto = Los cambios valen ahora y se pueden deshacer.
agente-falha-aplicar = Una propuesta no se pudo aplicar
agente-dados-mudaram = Los datos cambiaron después de la propuesta.
agente-proposta-rotulo = Propuesta: { $nome }
agente-sensivel-dica = Escritura sensible: aprobación una por una
agente-selecionar-lote = Seleccionar para aprobar en lote: { $nome }
agente-rejeitar = Rechazar
agente-aprovar = Aprobar
agente-aprovar-item = Aprobar este ítem
agente-carregando-diff = Cargando el diff
agente-erro-diff = No se pudo cargar el diff
agente-revisadas = Revisadas recientemente
agente-desfeita = Cambio deshecho
agente-nao-desfez = No se pudo deshacer
agente-nao-aprovou = No se pudo aprobar
agente-nao-rejeitou = No se pudo rechazar
agente-autonomo = Modo autónomo
agente-autonomo-ligado = Modo autónomo activado: las escrituras reversibles de la IA se aplican sin preguntar
agente-autonomo-desligado = Modo autónomo desactivado: toda escritura de la IA espera tu aprobación
agente-alterar-configurar = Cambiar en Configurar
agente-sensiveis-texto = Las escrituras sensibles (operaciones de inversión, importaciones, eliminaciones) siempre piden aprobación una por una. Todo lo que apruebes se puede deshacer.
agente-pendentes = Pendientes
agente-pendentes-n = Pendientes · { $n }
agente-selecionar-reversiveis = Seleccionar reversibles
agente-rejeitar-selecionadas = Rechazar seleccionadas
agente-aprovar-selecionadas = Aprobar seleccionadas ({ $n })
agente-fila-vazia = Ninguna propuesta espera aprobación.

## Alertas y rutinas

agente-filtro-abertos = Abiertas (nuevas y vistas)
agente-filtro-novos = Nuevas
agente-filtro-vistos = Vistas
agente-filtro-resolvidos = Resueltas
agente-nao-atualizou = No se pudo actualizar
agente-alerta-rotulo = Alerta: { $titulo }
agente-alerta-abrir-asset = Abrir activo
agente-alerta-abrir-account = Abrir cuenta
agente-alerta-abrir-budget = Abrir presupuesto
agente-alerta-abrir-transactions = Ver movimientos
agente-alerta-abrir-card-statement = Ver resumen de tarjeta
agente-alerta-abrir-physical-asset = Abrir bien
agente-alerta-abrir-allocation = Abrir asignación
agente-alerta-abrir-reports = Abrir informes
agente-alerta-abrir-sources = Abrir fuentes de datos
agente-alerta-abrir-asset-data = Ver datos de los activos
agente-alerta-abrir-import = Importar extracto
agente-alerta-avancado = Avanzado
agente-alerta-fontes = Fuentes de los números (herramientas de Treasury, para agentes): { $lista }
agente-marcar-visto = Marcar como vista
agente-resolver = Resolver
agente-reabrir = Reabrir
agente-apagar-alerta = Borrar la alerta { $titulo }
agente-rotinas = Rutinas del gestor
agente-rotina-ligada = activada
agente-rotina-desligada = desactivada
agente-rotina-com-motor = con motor de IA
agente-rotina-local = local y determinística (sin motor en la ruta)
agente-rotina-ultima = última: { $quando }
agente-rotina-nunca = nunca
agente-rotina-andamento = { $rotina }: en curso
agente-nao-rodou = No se pudo ejecutar
agente-rodando = Ejecutando…
agente-rodar-agora = Ejecutar ahora
agente-caixa-alertas = Bandeja de alertas
agente-marcar-todos-vistos = Marcar todas como vistas
agente-estado = Estado
agente-gravidade = Gravedad
agente-toda-gravidade = Toda gravedad
agente-tipo = Tipo
agente-todos-tipos = Todos los tipos
agente-sem-alertas = Ninguna alerta con estos filtros.

## Memoria del gestor

agente-gravada-voce = guardada por ti
agente-gravada-agente = guardada por el agente (aprobada)
agente-esquecer = Olvidar
agente-esquecer-titulo = Olvidar esta nota
agente-esquecer-texto = El agente deja de tenerla en cuenta en todas las conversaciones y rutinas. Se puede deshacer en el historial de cambios.
agente-nota-esquecida = Nota olvidada
agente-nao-esqueceu = No se pudo olvidar
agente-gravado-memoria = Guardado en la memoria del gestor
agente-nao-gravou = No se pudo guardar
agente-memoria-placeholder = Ej.: no sugerir cripto
agente-lembrar = Recordar
agente-memoria-rodape = Vale para todos los motores. El agente guarda notas nuevas solo con tu aprobación.
agente-memoria-vazia = La memoria está vacía.

## Procedimientos aprendidos

agente-proc-atualizado = Procedimiento actualizado
agente-proc-gravado = Procedimiento guardado
agente-proc-ocultados = { $n ->
        [one] { $n } fragmento sensible ocultado (contraseñas, números de cuenta o documentos).
       *[other] { $n } fragmentos sensibles ocultados (contraseñas, números de cuenta o documentos).
    }
agente-proc-editar = Editar procedimiento
agente-proc-novo = Nuevo procedimiento
agente-proc-dialogo-texto = Notas que el agente consulta antes de buscar de nuevo. No escribas contraseñas, códigos ni números de cuenta o documento.
agente-assunto = Tema
agente-titulo = Título
agente-site = Sitio
agente-site-dica = Solo el dominio, ej.: www.bbva.mx
agente-formato = Formato
agente-passos = Pasos
agente-passos-dica = Menús, filtros, el botón de exportar y lo que trae el archivo.
agente-proc-por-voce = Guardado por ti el { $data }
agente-proc-pelo-agente = Guardado por el agente el { $data }
agente-proc-pelo-cliente = Guardado por { $cliente } el { $data }
agente-proc-pelo-agente-cliente = Guardado por el agente ({ $cliente }) el { $data }
agente-proc-rotulo = Procedimiento: { $titulo }
agente-proc-funcionou-dica = Veces en que el agente siguió la nota y funcionó
agente-proc-funcionou = funcionó { $n }×
agente-proc-falhou-dica = Veces en que la nota no funcionó
agente-proc-falhou = falló { $n }×
agente-proc-arquivo = archivo { $formato }
agente-proc-observacao = Última observación: { $nota }
agente-proc-conversa = Conversación: { $titulo }
agente-proc-ver-conversa = Ver la conversación
agente-proc-atualizado-em = actualizado { $quando }
agente-proc-ultimo-uso = último uso { $quando }
agente-proc-ultimo-uso-resultado = último uso { $quando } ({ $resultado ->
        [worked] funcionó
       *[failed] falló
    })
agente-proc-apagar-titulo = Borrar este procedimiento
agente-proc-apagar-texto = El agente deja de usar esta nota. Se puede deshacer en el historial de cambios.
agente-proc-apagado = Procedimiento borrado
agente-procs-titulo = Procedimientos aprendidos
agente-adicionar = Agregar
agente-procs-rodape = El agente guarda y actualiza estas notas por su cuenta, sin propuesta: todo queda en el historial de cambios y se puede deshacer. Van junto con los pedidos relacionados como datos, nunca como instrucciones.
agente-buscar = Buscar
agente-procs-placeholder = Ej.: extracto bbva, b3, estado de cuenta
agente-todos-assuntos = Todos los temas
agente-procs-filtro-vazio = Ningún procedimiento con ese filtro.
agente-procs-vazio = El agente todavía no guardó procedimientos. Cuando descubre dónde descargar un extracto o de dónde viene un dato, la nota aparece aquí.
agente-procs = Procedimientos

## Historial de herramientas

agente-col-quando = Cuándo
agente-col-cliente = Cliente
agente-col-ferramenta = Herramienta
agente-col-parametros = Parámetros
agente-col-resultado = Resultado
agente-col-duracao = Duración
agente-col-inicio = Inicio
agente-col-tarefa = Tarea
agente-ferramentas-titulo = Herramientas usadas por la IA
agente-fonte = Fuente
agente-chamadas = Llamadas
agente-todos-clientes = Todos los clientes
agente-so-erros = Solo llamadas con error
agente-historico-ferramentas = Historial de herramientas
agente-nenhuma-chamada = Ninguna llamada registrada.
agente-jobs-titulo = Trabajos recientes del agente
agente-jobs = Trabajos del agente
agente-jobs-vazio = Ningún trabajo en los últimos 30 minutos.

## Ajustes de IA

agente-ajustes-titulo = Motores de IA y enrutamiento
agente-ajustes-ia = Ajustes de IA
agente-nao-salvou = No se pudo guardar
agente-opcoes-titulo = Ajustes del agente
agente-permissoes = Permisos
agente-autonomo-dica = Aplica sin preguntar las escrituras reversibles de la IA (categorizar, crear regla, presupuesto). Las escrituras sensibles siguen pidiendo aprobación una por una; todo se puede deshacer.
agente-mcp-escrita = Escritura por clientes MCP externos
agente-mcp-escrita-dica = Los asistentes de IA y agentes de código conectados por MCP pueden proponer escrituras (siempre como propuesta en la cola).
agente-guardar-historico = Guardar el historial de las conversaciones
agente-guardar-historico-dica = Sin historial, cada pregunta empieza de cero y nada de la conversación queda en la base de datos.
agente-rotinas-legenda = Rutinas (con la app abierta y desbloqueada)
agente-revisao-continua = Revisión continua
agente-revisao-continua-dica = Al importar extractos: categoriza, señala duplicados y sugiere reglas (como propuestas).
agente-fechamento-dica = Informe del mes con 3 a 5 recomendaciones, entregado como alerta.
agente-alertas-dica = Presupuesto excediéndose, vencimientos, deuda cara, cartera desbalanceada.

## Rutas por tarea

agente-rota-chat-nota = También vale para categorización y PDF cuando no tienen ruta propia.
agente-rota-rotina-nota = Rutina en segundo plano: solo usa los motores de esta lista.
agente-rota-estudo-nota = Estudio con IA de un activo. Sin motor aquí, usa la ruta del chat. Solo el paquete de datos del activo va al motor.
agente-rota-categorizacao-nota = Sin ruta propia, la revisión continua corre local y determinística.
agente-nao-salvou-rota = No se pudo guardar la ruta
agente-rota = Ruta: { $tarefa }
agente-sem-motor-rota = Sin motor en esta ruta.
agente-motor-removido = motor eliminado
agente-subir = Subir { $motor }
agente-descer = Bajar { $motor }
agente-tirar-rota = Quitar { $motor } de la ruta
agente-adicionar-motor-rota = Agregar motor a la ruta { $tarefa }
agente-adicionar-fallback = Agregar respaldo…
agente-adicionar-motor-mais = Agregar motor…
agente-roteamento = Enrutamiento por tarea
agente-rotas = Rutas por tarea

## Motores de IA

agente-respondeu-em = ✓ respondió en { $tempo }
agente-motor-rotulo = Motor { $nome }
agente-pronto = listo
agente-nao-alterou = No se pudo cambiar
agente-modelo = Modelo
agente-url-local = URL local
agente-endpoint = Endpoint
agente-api-key = Clave de API
agente-chave-cadastrada = guardada
agente-chave-cadastrada-dica = guardada · { $dica }
agente-chave-nao-cadastrada = no guardada
agente-dados = Datos
agente-testar = Probar
agente-excluir = Eliminar
agente-excluir-motor = Eliminar { $nome }
agente-excluir-motor-texto = El motor sale de las rutas de tarea y se borra la clave guardada. El historial de uso se queda.
agente-nao-excluiu = No se pudo eliminar
agente-motores-titulo = Motores de IA
agente-adicionar-motor = Agregar motor
agente-motores-rodape = Las API cobran por token; las CLI locales usan la suscripción que ya pagas; los modelos locales no envían nada hacia afuera.
agente-motores-vazio = Ningún motor registrado. Agrega una API, una CLI local o un modelo local.
agente-motores = Motores
agente-procurando-clis = Buscando CLI en el PATH
agente-clis-encontradas = CLI encontradas
agente-clis-detectadas = CLI detectadas
agente-cli-instalada = instalada
agente-cli-login-ok = · sesión encontrada
agente-cli-sem-login = · sin sesión
agente-cli-login-incerto = · sesión: no se puede saber sin ejecutarla
agente-cli-nao-encontrada = no encontrada
agente-cli-historico-fora = Esta CLI guarda el historial de las conversaciones en su propia carpeta, fuera de la bóveda. La CLI de Claude Code se ejecuta sin historial.
agente-motor-cadastrado = Motor registrado
agente-motor-atualizado = Motor actualizado
agente-editar-motor = Editar motor · { $nome }
agente-novo-motor = Nuevo motor de IA
agente-cadastrar = Registrar
agente-provider = Proveedor
agente-exige-chave = { $tipo } · requiere clave de API
agente-nome = Nombre
agente-nome-dica = Cómo aparece el motor en la app.
agente-modelo-dica = El nombre de los modelos cambia con el tiempo; revísalo con el proveedor.
agente-modelo-placeholder = ej.: llama3.2
agente-modelo-opcional = Modelo (opcional)
agente-modelo-opcional-dica = Va en el marcador { "{model}" } de la plantilla.
agente-url-servidor = URL del servidor local
agente-url-servidor-dica = http solo para esta computadora (localhost/127.0.0.1).
agente-endpoint-opcional = Endpoint (opcional)
agente-endpoint-padrao = Predeterminado: { $url }
agente-endpoint-dica = Base de la API compatible con OpenAI (https://…/v1).
agente-max-tokens = Máx. de tokens en la respuesta
agente-max-tokens-dica = Vacío: 4.096.
agente-max-tokens-erro = Usa un número entero positivo.
agente-chave-dica = Guardada cifrada en la bóveda; nunca se vuelve a mostrar.
agente-cadastre-de-novo = vuelve a ingresarla
agente-sem-chave = sin clave
agente-substituir-chave = Escribe para reemplazar la clave
agente-cole-chave = Pega la clave del proveedor
agente-chave-apagada = Clave borrada
agente-apagar-chave = Borrar la clave guardada
agente-template = Plantilla de comando
agente-template-dica = Marcadores: { "{prompt}" }, { "{mcp_config}" }, { "{model}" }, { "{workdir}" }. Los fragmentos entre [ ] desaparecen cuando el marcador está vacío.
agente-template-placeholder = mi-cli -p { "{prompt}" }
agente-template-padrao = Vacío usa el predeterminado, con las herramientas nativas de la CLI desactivadas: { $template }
agente-motor-ligado = Motor activado

## Límites de gasto

agente-limites-salvos = Límites de { $provider } guardados
agente-limites-rotulo = Límites: { $provider }
agente-gasto = hoy { $hoje } · mes { $mes }
agente-por-dia = Por día (USD)
agente-por-mes = Por mes (USD)
agente-confirmar-acima = Confirmar por encima de (USD)
agente-valor-invalido = Monto inválido
agente-sem-limite = sin límite
agente-nunca-perguntar = nunca preguntar
agente-limites-titulo = Límites de gasto
agente-limites-rodape = Costos en dólares, estimados con la tabla de precios. Por encima del límite el motor se salta (respaldo); por encima del aviso, el chat pide confirmación antes de enviar.
agente-sem-provider = Ningún proveedor de API en uso.
agente-limites-por-provider = Límites por proveedor
agente-limitar-outro = Limitar otro proveedor
agente-limitar-outro-mais = Limitar otro proveedor…

## Uso y costo

agente-periodo-1 = Este mes
agente-periodo-3 = Últimos 3 meses
agente-periodo-6 = Últimos 6 meses
agente-periodo-12 = Últimos 12 meses
agente-tokens-entrada-saida = Tokens entrada / salida
agente-tokens-entrada-saida-tooltip = Tokens (entrada / salida)
agente-col-custo = Costo
agente-erros = Errores
agente-sem-uso = Sin uso en el período.
agente-uso-titulo = Uso y costo
agente-periodo = Período
agente-moeda = Moneda
agente-fonte-uso = ai_usage · tabla de precios
agente-custo-estimado = Costo estimado
agente-tokens = Tokens
agente-sem-preco = { "*" } { $n ->
        [one] { $n } llamada sin precio registrado para el modelo
       *[other] { $n } llamadas sin precio registrado para el modelo
    }: costo fuera del total. Registra el precio en la tabla de abajo.
agente-custo-mes = Costo estimado por mes
agente-custo-fonte = Fuente: llamadas registradas × tabla de precios (las CLI y los modelos locales cuestan cero en la app)
agente-custo-ia-mes = Costo estimado de IA por mes
agente-mes = Mes
agente-por = Por
agente-agrupar = Agrupar uso por
agente-tarefa = Tarea
agente-uso-por-provider = Uso por proveedor
agente-uso-por-motor = Uso por motor
agente-uso-por-tarefa = Uso por tarea

## Precios por modelo

agente-preco-salvo = Precio guardado
agente-preco-de = Precio · { $modelo }
agente-novo-preco = Nuevo precio
agente-modelo-prefixo = Modelo o prefijo
agente-modelo-prefixo-dica = { "*" } = predeterminado del proveedor
agente-entrada-mtok = Entrada (USD / Mtok)
agente-saida-mtok = Salida (USD / Mtok)
agente-cache-mtok = Caché (USD / Mtok)
agente-opcional = Opcional
agente-entrada = Entrada
agente-saida = Salida
agente-cache = Caché
agente-origem = Origen
agente-editado = editado
agente-padrao = predeterminado
agente-acoes = Acciones
agente-apagar-preco = Borrar el precio de { $modelo }
agente-precos-titulo = Precios por modelo · USD por millón de tokens
agente-precos = Precios por modelo
agente-precos-vazio = Ningún precio registrado: el costo de las llamadas queda fuera del total.

## Historial y adjuntos en la bóveda

agente-ret-nenhum-dica = El archivo queda solo en la memoria de la app mientras la bóveda está abierta (el agente lo vuelve a leer en los siguientes turnos de la conversación) y no va a la bóveda; la conversación guarda solo el nombre y el tamaño.
agente-ret-7-dica = Guarda los archivos por 7 días y después los borra (la limpieza diaria se encarga).
agente-ret-sempre-dica = Guarda los archivos para siempre, hasta que borres la conversación.
agente-n-dias = { $n } días
agente-1-ano = 1 año
agente-nunca = Nunca
agente-faxina-feita = Limpieza hecha
agente-faxina-texto = { $conversas ->
        [one] { $conversas } conversación
       *[other] { $conversas } conversaciones
    } y { $anexos ->
        [one] { $anexos } adjunto vencido
       *[other] { $anexos } adjuntos vencidos
    } borrados.
agente-faxina-texto-encolheu = { $conversas ->
        [one] { $conversas } conversación
       *[other] { $conversas } conversaciones
    } y { $anexos ->
        [one] { $anexos } adjunto vencido
       *[other] { $anexos } adjuntos vencidos
    } borrados; la bóveda se redujo { $tamanho }.
agente-nao-limpou = No se pudo limpiar
agente-hist-titulo = Historial y adjuntos en la bóveda
agente-anexos-enviados = Adjuntos enviados al agente
agente-guardar-anexos = Guardar adjuntos en la bóveda
agente-nao-guardar = No guardar
agente-7-dias = 7 días
agente-sempre = Siempre
agente-recomendado = Recomendado: la bóveda queda liviana y ningún documento enviado al agente queda guardado.
agente-nao-recomendado = No recomendado: las imágenes y los PDF vuelven grande la base de datos cifrada (copias de seguridad más lentas y grandes) y mantienen copias de documentos sensibles. Prefiere "No guardar".
agente-nao-recomendado-hoje = No recomendado: las imágenes y los PDF vuelven grande la base de datos cifrada (copias de seguridad más lentas y grandes) y mantienen copias de documentos sensibles. Prefiere "No guardar". Hoy: { $tamanho } en adjuntos.
agente-apagar-sem-uso = Borrar conversaciones sin uso desde hace
agente-faxina-diaria = La limpieza corre una vez al día con la bóveda abierta. Las conversaciones fijadas se quedan.
agente-faxina-nunca = Las conversaciones se quedan hasta que las borres. La bóveda puede crecer con el tiempo.
agente-retencao-conversas = Retención de las conversaciones
agente-conversas-fixadas = { $n } ({ $fixadas } fijadas)
agente-mensagens = Mensajes
agente-anexos-guardados = { $n } ({ $guardados } guardados)
agente-historico = Historial
agente-cofre = Bóveda
agente-cofre-livres = { $tamanho } ({ $livres } libres)
agente-ultima-faxina = Última limpieza
agente-ainda-nao = todavía no
agente-limpar-agora = Limpiar ahora
agente-limpar-agora-dica = Aplica ya las reglas de arriba y compacta la bóveda si sobra espacio.
agente-conversas-abrir = Mostrar las conversaciones
agente-conversas-recolher = Ocultar las conversaciones
agente-conversa-nova-titulo = Conversación nueva
agente-exportar = Exportar
agente-exportar-salvar = Guardar la conversación
agente-exportar-md-dica = Guardar la conversación en Markdown (.md)
agente-exportar-pdf-dica = Guardar la conversación en PDF
agente-exportar-feito = Conversación exportada
agente-exportar-feito-texto = Guardada en { $caminho }.
agente-exportar-feito-segredos = Guardada en { $caminho }. { $n ->
    [one] Se quitó 1 fragmento con clave o contraseña.
   *[other] Se quitaron { $n } fragmentos con clave o contraseña.
}
agente-exportar-erro = No se pudo exportar la conversación
agente-exportar-md = Markdown
agente-mensagens-conversa = Mensajes de la conversación

# Imagens com texto lido pelo OCR: o que vai ao motor.
agente-anexo-texto-lido = texto leído en la computadora
agente-imagem-modo-rotulo = Qué enviar de las imágenes
agente-imagem-modo-pergunta = Texto leído en la computadora ({ $confianca }% de confianza). Enviar:
agente-imagem-modo-texto = Solo el texto
agente-imagem-modo-imagem = Solo la imagen
agente-imagem-modo-ambos = Los dos
agente-imagem-so-texto = Este motor no lee imágenes: va el texto leído en la computadora ({ $confianca }% de confianza).

## MCP extra (solo en esta bóveda)

agente-mcpx-titulo = MCP extra (solo en esta bóveda)
agente-mcpx-adicionar = Añadir MCP
agente-mcpx-vazio = Ningún MCP extra. Registra un servidor MCP externo (por ejemplo, uno que controla tu móvil) para que lo use el agente de esta bóveda.
agente-mcpx-explica = Cada llamada a una herramienta pide tu confirmación en la conversación, salvo que "Permitir sin preguntar" esté activado. Vale para todos los motores, incluidos los por CLI: las herramientas les llegan por el propio MCP de Treasury.
agente-mcpx-seguranca = Un comando local se ejecuta en este ordenador con los permisos de tu usuario. Añade solo servidores en los que confíes.
agente-mcpx-lista = MCP extra de esta bóveda
agente-mcpx-linha = MCP { $nome }
agente-mcpx-tipo-stdio = Comando local
agente-mcpx-tipo-http = Dirección HTTP
agente-mcpx-ferramentas-prefixo = Herramientas
agente-mcpx-permitir-sem-perguntar = Permitir sin preguntar
agente-mcpx-excluir = Eliminar { $nome }
agente-mcpx-excluir-texto = El servidor sale del agente de esta bóveda y los valores secretos guardados para él se borran.
agente-mcpx-nao-excluiu = No se pudo eliminar el MCP
agente-mcpx-testando = Conectando… (puede tardar hasta 20 s)
agente-mcpx-ferramentas-encontradas =
    { $n ->
        [one] 1 herramienta encontrada
       *[other] { $n } herramientas encontradas
    }
agente-mcpx-ferramentas = Herramientas del MCP
agente-mcpx-sem-ferramentas = Conectó, pero el servidor no ofrece herramientas.
agente-mcpx-novo = Nuevo MCP extra
agente-mcpx-editar = Editar { $nome }
agente-mcpx-rotulo = Etiqueta
agente-mcpx-rotulo-dica = Cómo aparece en la conversación (p. ej.: Mi móvil).
agente-mcpx-rotulo-erro = Indica una etiqueta de hasta 60 caracteres.
agente-mcpx-nome = Nombre
agente-mcpx-nome-dica = Prefijo de las herramientas ({ $exemplo }).
agente-mcpx-nome-erro = Usa de 2 a 24 letras minúsculas, números o _, empezando por letra (y no "treasury").
agente-mcpx-tipo = Tipo
agente-mcpx-comando = Comando
agente-mcpx-comando-dica = El programa que inicia el servidor MCP (p. ej.: npx o la ruta completa).
agente-mcpx-comando-erro = Indica el comando.
agente-mcpx-argumentos = Argumentos
agente-mcpx-argumentos-dica = Uno por línea.
agente-mcpx-variaveis = Variables de entorno
agente-mcpx-url = URL
agente-mcpx-url-dica = https://, o http://localhost para un servidor en este ordenador.
agente-mcpx-url-erro = La dirección debe empezar por https:// (o http://localhost).
agente-mcpx-cabecalhos = Cabeceras
agente-mcpx-cabecalhos-dica = Una clave de acceso va aquí, marcada como secreta: { $exemplo }
agente-mcpx-chave = Nombre
agente-mcpx-valor = Valor
agente-mcpx-secreta = Secreta
agente-mcpx-manter-segredo = déjalo en blanco para mantenerlo
agente-mcpx-variavel-n = Variable { $n }
agente-mcpx-cabecalho-n = Cabecera { $n }
agente-mcpx-adicionar-variavel = Añadir variable
agente-mcpx-adicionar-cabecalho = Añadir cabecera
agente-mcpx-remover = Quitar
agente-mcpx-ligado = Activado (el agente de esta bóveda puede usarlo)
agente-mcpx-permitir-dica = El agente llama a las herramientas sin pedir confirmación.
agente-mcpx-salvo = MCP guardado
agente-mcpx-pedido-rotulo = Confirmación de herramienta externa
agente-mcpx-pedido = El agente quiere usar { $ferramenta } del MCP { $servidor }
agente-mcpx-parametros = Parámetros
agente-mcpx-ver-tudo = Ver todo
agente-mcpx-recolher = Contraer
agente-mcpx-permitir = Permitir
agente-mcpx-recusar = Rechazar
agente-mcpx-permitida = Permitida
agente-mcpx-recusada = Rechazada
agente-mcpx-aguardando = Esperando tu respuesta
agente-mcpx-nao-respondeu = No se pudo responder a la solicitud

## Pestañas de Ajustes › IA

agente-ajustes-abas = Partes de los ajustes de IA
agente-ajustes-aba-motores = Motores
agente-ajustes-aba-motores-dica = Qué motores de IA usa Treasury y cuál atiende cada tarea, con el de reserva cuando uno falla.
agente-ajustes-aba-acesso = Acceso del agente
agente-ajustes-aba-acesso-dica = Lo que el agente puede hacer solo, lo que ve y qué herramientas externas quedan a su alcance.
agente-ajustes-aba-custos = Costes
agente-ajustes-aba-custos-dica = Cuánto ha costado la IA, los límites de gasto y el precio de cada modelo.
agente-ajustes-aba-historico = Historial
agente-ajustes-aba-historico-dica = Las conversaciones y los adjuntos guardados en la bóveda, y durante cuánto tiempo.

# MCP de las CLI del usuario (Ajustes › IA)
agente-climcp-titulo = MCP de tus CLI
agente-climcp-explica = MCP que ya tienes en Claude, Cursor, Grok, Codex, Gemini u OpenCode. Por defecto, ninguno entra en las conversaciones del agente de esta bóveda: solo Treasury y los MCP extra. Activa solo los que el agente necesita.
agente-climcp-seguranca = Un MCP permitido aquí lo llama la CLI directamente, sin la confirmación ni la auditoría de Treasury.
agente-climcp-vazio = Ninguna CLI instalada carga otros MCP.
agente-climcp-lista = MCP de las CLI
agente-climcp-nao-encontrado = no encontrado en las CLI ahora
agente-climcp-liberar = Usar { $nome } en las conversaciones
agente-mcpx-permitir-conversa = Permitir en esta conversación
agente-mcpx-permitir-conversa-dica = Permite todas las herramientas del MCP { $servidor } hasta el final de esta conversación, sin volver a preguntar. Vale hasta bloquear la bóveda; cada llamada sigue en la auditoría.
agente-mcpx-permitida-conversa = Permitida en esta conversación ({ $servidor })
agente-tempo-cli = Tiempo límite de las respuestas por CLI
agente-tempo-cli-dica = Las recolecciones largas (MCP extra, dispositivo) necesitan más tiempo. Cerca del final, se avisa al agente para que guarde lo recolectado; si el tiempo se acaba, la parte hecha queda en la conversación.
agente-tempo-cli-minutos = { $n } min

## Pregunta del agente en la conversación (ask_user)

agente-pergunta-rotulo = Pregunta del agente
agente-pergunta-uma = Elige una opción.
agente-pergunta-varias = Elige una o más opciones.
agente-pergunta-recomendado = Recomendada
agente-pergunta-outro = Otra
agente-pergunta-outro-texto = Tu respuesta
agente-pergunta-outro-dica = Escribe tu respuesta
agente-pergunta-enviar = Enviar
agente-pergunta-aguardando = El agente espera tu respuesta.
agente-pergunta-respondeu = Respondiste: { $resposta }
agente-pergunta-sem-resposta = Sin respuesta
agente-pergunta-nao-enviou = No se pudo enviar la respuesta
