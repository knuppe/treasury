# Estructura de la app: menú lateral, barra superior, barra de comandos, temas, tareas y bloqueo
# (docs/i18n-design.md). Revisión: borrador por IA, sin revisión nativa.

## Grupos y pantallas del menú (las palabras de búsqueda van separadas por coma)

casca-grupo-fluxo = Flujo de caja
casca-grupo-investimentos = Inversiones
casca-grupo-gestao = Gestión

casca-tela-painel = Panel
casca-tela-painel-resumo = Patrimonio neto, flujo del mes, tasa de ahorro, cartera frente a la inflación y vencimientos.
casca-tela-painel-busca = inicio,dashboard,resumen,patrimonio

casca-tela-familia = Familia
casca-tela-familia-resumo = Quién es la familia, los ingresos de cada miembro (salario, recibos de nómina, ingresos informales) y sus cuentas y tarjetas.
casca-tela-familia-busca = ingresos,salario,nómina,recibo,miembros,titulares,dependientes,ganancias,personas

casca-tela-contas = Cuentas
casca-tela-contas-resumo = Saldos por cuenta, tarjetas, corredoras y cuentas en el exterior; conciliación.
casca-tela-contas-busca = saldo,banco,tarjeta,conciliación

casca-tela-lancamentos = Movimientos
casca-tela-lancamentos-resumo = Ingresos, gastos y transferencias con filtros, división, importación y reglas.
casca-tela-lancamentos-busca = transacciones,gastos,ingresos,extracto,importar,ofx,csv

casca-tela-orcamento = Presupuesto
casca-tela-orcamento-resumo = Previsto frente a real por categoría, por devengo o por caja, y metas.
casca-tela-orcamento-busca = presupuesto,metas,categorías,previsto
casca-tela-consumo = Consumo
casca-tela-consumo-resumo = Todo lo que compra la familia, artículo por artículo: supermercado, ropa, electrónica y lo demás, con la comida en casa y fuera y el precio por kg en el tiempo.
casca-tela-consumo-busca = consumo,compras,alimentación,mercado,supermercado,ropa,electrónica,comprobante,productos,precio por kg,comida,restaurante,delivery

casca-tela-dividas = Deudas
casca-tela-dividas-resumo = Préstamos y financiamientos: contratos, cronograma y simulador.
casca-tela-dividas-busca = préstamo,financiamiento,hipoteca,amortización,cuotas

casca-tela-carteira = Cartera
casca-tela-carteira-resumo = Posiciones, operaciones, dividendos y rentabilidad nominal y real; bienes físicos.
casca-tela-carteira-busca = inversiones,posiciones,rentabilidad,rendimientos,dividendos,bienes,inmueble,vehículo,patrimonio físico

casca-tela-ativos = Activos
casca-tela-ativos-resumo = Detalle del activo, indicadores, precio calculado, comparador y agenda.
casca-tela-ativos-busca = acciones,fii,etf,watchlist,cotización,indicadores,comparador
casca-tela-mercado = Mercado
casca-tela-mercado-resumo = Encontrar activos en todo el mercado: panorama por sector, explorador con el score, mapa y rankings.
casca-tela-mercado-busca = mercado,explorador,screener,ranking,sector,score,observar,descubrir

casca-tela-relatorios = Informes
casca-tela-relatorios-resumo = Informes por período, categoría y miembro de la familia; impuesto sobre la renta y DARF.
casca-tela-relatorios-busca = impuesto,renta,darf,declaración,exportar,pdf
casca-tela-calculadoras = Calculadoras
casca-tela-calculadoras-resumo = Cuentas sueltas para probar una idea: interés compuesto, vivir de rentas, al contado × en cuotas y alquilar × comprar. Nada se guarda en la bóveda.
casca-tela-calculadoras-busca = calculadora,simulador,interés,interés compuesto,renta,independencia,cuotas,contado,alquiler,hipoteca,vivienda,comprar

casca-tela-impostos = Impuestos
casca-tela-impostos-resumo = Estimación de los impuestos pagados en el año (consumo, renta del trabajo, patrimonio), con las fuentes.
casca-tela-impostos-busca = tributos,carga tributaria,impuestos pagados,consumo,iva,seguridad social,renta,predial,tenencia,constancia de retenciones

casca-tela-agente = Agente
casca-tela-agente-resumo = Gestor financiero con IA: conversación, cola de aprobación, alertas e historial.
casca-tela-agente-busca = ia,ai,chat,aprobación,alertas,mcp

casca-tela-ajustes = Ajustes
casca-tela-ajustes-resumo = Seguridad, copias de seguridad, fuentes de datos, motores de IA, temas e historial.
casca-tela-ajustes-busca = configuración,preferencias,seguridad,copia de seguridad,tema,contraseña,idioma

casca-tela-desenvolvedor = Desarrollador
casca-tela-desenvolvedor-resumo = Comentarios del agente para el desarrollador de Treasury: errores, mejoras y carencias.
casca-tela-desenvolvedor-busca = comentarios,feedback,dev,error,bug,mejora,issue,github,modo desarrollador

## Menú lateral

casca-menu-expandir = Expandir menú lateral
casca-menu-recolher = Contraer menú lateral
casca-familia = Familia
casca-sem-nome = Sin nombre
casca-telas = Pantallas
casca-mais-telas = Más pantallas
casca-cofre-aberto = Bóveda abierta
casca-auto-bloqueio = Bloqueo automático
casca-minutos = { $minutos } min
casca-desligado = desactivado
casca-cifra = Cifrado
casca-bloquear-cofre = Bloquear bóveda

## Barra superior

casca-demo = Demostración
casca-demo-dica = Bóveda de demostración: datos ficticios creados por el asistente. Tu bóveda sigue igual; cambia en Bóvedas.
casca-mock = Mock
casca-mock-dica = Datos de un núcleo falso en memoria (desarrollo). No se guarda nada.
casca-versao-antiga = Actualiza la app
casca-versao-antiga-dica = Esta bóveda ya fue abierta por Treasury { $versao }, más nuevo que este. Actualiza la app: los ajustes guardados por la versión nueva pueden no aparecer aquí.
casca-buscar = Buscar o ejecutar…
casca-privacidade = Privacidad
casca-agente-ao-lado = Agente al lado
casca-modo-privacidade = Modo privacidad
casca-ver-como = Ver como
casca-familia-toda = Toda la familia

## Niveles del perfil

casca-nivel-iniciante = Principiante
casca-nivel-intermediario = Intermedio
casca-nivel-avancado = Avanzado
casca-nivel-iniciante-dica = Panel sencillo (cuánto entró, cuánto salió, qué sobró), meta de fondo de emergencia y consejos en lenguaje cotidiano. Las pantallas avanzadas están en “Más pantallas”.
casca-nivel-intermediario-dica = Panel con categorías y comparación mes a mes, presupuesto e informes. Inversiones e impuestos están en “Más pantallas”.
casca-nivel-avancado-dica = Todo a la vista: panel completo, cartera, activos, informes e impuestos.

## Temas

casca-tema-escolher = Elegir tema (actual: { $tema })
casca-temas = Temas
casca-temas-classicos = Clásicos
casca-temas-especiais = Especiales
casca-temas-divertidos = Divertidos
casca-tema-dark = Oscuro
casca-tema-dark-dica = Grafito azulado con acento cian. El predeterminado.
casca-tema-light = Claro
casca-tema-light-dica = Gris claro neutro, tarjetas blancas y tinta azul.
casca-tema-mono-dica = Tonos de gris, sin ningún color: solo la información.
casca-tema-midnight = Medianoche
casca-tema-midnight-dica = Azul marino profundo con acento champán.
casca-tema-paper = Papel
casca-tema-paper-dica = Claro y cálido, como papel de cuaderno, con tinta azul.
casca-tema-glass = Vidrio
casca-tema-glass-dica = Vidrio esmerilado sobre una malla de azul, verde agua y violeta.
casca-tema-scanner = Escáner
casca-tema-scanner-dica = Negro brillante, con un escáner rojo recorriendo la barra superior.
casca-tema-orbit = Órbita
casca-tema-orbit-dica = Un agujero negro en lo alto de la pantalla: estrellas y el disco dorado girando despacio.
casca-tema-neon = Neón
casca-tema-neon-dica = Una ciudad de noche: neón rosa y cian, bruma naranja y lluvia sin parar.
casca-tema-lofi-dica = Una tarde tranquila en Japón: papel washi, tinta sumi, índigo y hojas de otoño llevadas por el viento.
casca-tema-phosphor = Fósforo
casca-tema-phosphor-dica = Código verde cayendo sobre negro, como en una terminal antigua.

## Barra de comandos

casca-cmd-barra = Barra de comandos
casca-cmd-placeholder = Ir a una pantalla, ejecutar una acción o preguntar al agente…
casca-cmd-nenhum = No se encontró ningún comando.
casca-cmd-navegar = navegar
casca-cmd-executar = ejecutar
casca-cmd-abrir-fechar = abrir/cerrar
casca-cmd-grupo-navegacao = Navegación
casca-cmd-grupo-cadastro = Registros
casca-cmd-grupo-cofre = Bóveda
casca-cmd-grupo-exibicao = Visualización
casca-cmd-grupo-agente = Agente
casca-cmd-ir-para = ir a,abrir
casca-cmd-importar = Importar archivos
casca-cmd-importar-busca = ofx,csv,extracto,estado de cuenta,nota de corretaje,sinacor,b3,pdf,hoja de cálculo
casca-cmd-bloquear-busca = cerrar con llave,candado,salir,cerrar bóveda
casca-cmd-privacidade-ativar = Activar modo privacidad
casca-cmd-privacidade-desativar = Desactivar modo privacidad
casca-cmd-privacidade-busca = ocultar valores,esconder,mostrar valores,compartir pantalla
casca-cmd-menu-busca = barra lateral,menú,ocultar menú,más espacio,área de trabajo
casca-cmd-tema-alternar = Alternar tema claro/oscuro
casca-cmd-tema-busca = tema,modo oscuro,modo claro,apariencia
casca-cmd-tema = Tema: { $tema }
casca-cmd-atual = Actual
casca-cmd-temas-busca = tema,colores,apariencia
casca-cmd-claro = claro
casca-cmd-escuro = oscuro
casca-cmd-cinza-busca = gris blanco y negro
casca-cmd-vidro-busca = vidrio esmerilado translúcido glass
casca-cmd-agente-alternar = Abrir o cerrar el agente al lado
casca-cmd-agente-busca = ia,compañero,panel,chat,gestor,asistente
casca-cmd-perguntar = Preguntar al agente
casca-cmd-perguntar-busca = ia,pregunta,gestor
casca-cmd-perguntar-texto = Preguntar al agente: “{ $texto }”

## Tareas en segundo plano

casca-tarefas = Tareas en segundo plano
casca-tarefas-andamento = Tareas en segundo plano ({ $n } en curso)
casca-tarefas-nenhuma = Ninguna tarea en curso
casca-tarefas-n = { $n } en curso
casca-tarefas-vazio = Nada en ejecución ahora. Las recolecciones (índices, cotizaciones, fundamentales), las conversaciones con el agente y las rutinas (cierre del mes, alertas) aparecen aquí mientras trabajan.
casca-tarefas-em-andamento = Tareas en curso
casca-tarefas-ultimos = Últimos 30 minutos
casca-tarefas-recentes = Tareas recientes
casca-tarefa-andamento = En curso
casca-tarefa-concluida = Terminada
casca-tarefa-falhou = Falló
casca-tarefa-cancelada = Cancelada
casca-tarefa-cancelar = Cancelar { $tarefa }
casca-tarefa-cancelada-toast = Tarea cancelada
casca-tarefa-nao-cancelou = No se pudo cancelar
casca-cancelando = Cancelando…
casca-rotina = Rutina
casca-agora = ahora
casca-ha-segundos = hace { $s } s
casca-ha-minutos = hace { $m } min
casca-ha-horas = hace { $h } h { $m }

## Recolecciones y descargas del núcleo (docs/background-tasks.md)

casca-coleta-market-sync = Índices, tipos de cambio y cotizaciones
casca-coleta-indices-sync = Índices y tipos de cambio
casca-coleta-research-sync = Fundamentales y score del mercado
casca-coleta-mail-sync = Correo de ingesta
casca-coleta-update-download = Actualización de Treasury
casca-coleta-curta-market-sync = Cotizaciones
casca-coleta-curta-indices-sync = Índices
casca-coleta-curta-research-sync = Fundamentales
casca-coleta-curta-mail-sync = Correo
casca-coleta-curta-update-download = Actualización
casca-etapa-indices = Índices y tipos de cambio
casca-etapa-prices = Cotizaciones
casca-etapa-funds = Cuotas de fondos y pensiones (CVM)
casca-etapa-income = Dividendos anunciados
casca-etapa-fundamentals = Archivos de la CVM, la SEC y B3
casca-etapa-scores = Score del mercado
casca-etapa-mail = Buscando mensajes
casca-etapa-download = Descargando
casca-tarefa-automatica = Automática
casca-tarefa-de = { $feito } de { $total }
casca-tarefa-feitos = { $n } { $n ->
        [one] completado
       *[other] completados
    }
casca-tarefa-progresso = Progreso: { $tarefa }
casca-coleta-chip = { $tarefa } en curso
casca-coleta-chip-pct = { $tarefa } en curso, { $pct }
casca-coleta-chip-mais = y { $n } más
casca-coleta-cancelada-dica = { $tarefa }: lo que ya había llegado quedó guardado. La recolección automática vuelve en una hora.

## Bloqueo y errores

casca-autobloqueio = Bloqueo automático
casca-autobloqueio-em = La bóveda se bloqueará por inactividad en
casca-autobloqueio-menos-minuto = menos de un minuto
casca-autobloqueio-continuar = . Mueve el ratón o pulsa una tecla para continuar.
casca-bloqueado-inatividade = Bóveda bloqueada por inactividad
casca-bloqueado-inatividade-dica = Escribe la contraseña maestra para seguir donde lo dejaste.
casca-erro-desconhecido = Error desconocido.
casca-falha-titulo = Fallo en la interfaz
casca-falha-texto = Algo salió mal al mostrar esta pantalla. Tus datos siguen en la bóveda, intactos.
casca-recarregar = Recargar
casca-ir-painel = Ir al panel
casca-menu-apoiar = Apoyar el proyecto…
casca-pular-conteudo = Saltar al contenido
casca-carregando-tela = Cargando pantalla
casca-abrir = Abrir
casca-nao-encontrada = Pantalla no encontrada
casca-sem-sinal = Sin señal
casca-nao-encontrada-titulo = Esta dirección no lleva a ninguna pantalla
casca-nao-encontrada-texto = Usa el menú lateral o la barra de comandos (⌘K) para elegir una pantalla.
casca-navegador-falhou = No se pudo abrir el navegador

## Idioma (primera pantalla y Ajustes › Apariencia y uso)

casca-idioma = Idioma
casca-idioma-titulo = Elige el idioma
casca-idioma-texto = Puedes cambiarlo después en Ajustes › Apariencia y uso.
casca-idioma-dica = Vale para esta computadora, en todas las bóvedas, y cambia al instante, sin reiniciar. Los nombres de cuentas y categorías que creaste se quedan como están.
casca-idioma-pseudo = Pseudoidioma (prueba de traducción)


## Barra ⌘K: destinos, acciones rápidas y búsqueda en la bóveda

casca-cmd-grupo-recentes = Recientes
casca-cmd-grupo-acoes = Acciones rápidas
casca-cmd-grupo-ir = Ir a
casca-cmd-grupo-contas = Cuentas
casca-cmd-grupo-categorias = Categorías
casca-cmd-grupo-metas = Metas
casca-cmd-grupo-bens = Bienes
casca-cmd-grupo-lancamentos = Movimientos recientes
casca-cmd-ir-aba = { $tela } › { $aba }
casca-cmd-familia-renda = Ingresos y recibos de nómina
casca-cmd-novo-lancamento = Nuevo movimiento (gasto)
casca-cmd-novo-lancamento-busca = gasto,compra,pago,egreso,agregar
casca-cmd-nova-receita = Nuevo ingreso
casca-cmd-nova-receita-busca = ingreso,cobro,sueldo,salario,ganancia
casca-cmd-nova-transferencia = Nueva transferencia
casca-cmd-nova-transferencia-busca = transferencia,mover,entre cuentas
casca-cmd-compra-parcelada = Compra en cuotas con tarjeta
casca-cmd-compra-parcelada-busca = cuotas,tarjeta,credito,meses sin intereses
casca-cmd-ajuste-saldo = Ajustar el saldo de una cuenta
casca-cmd-ajuste-saldo-busca = ajuste,saldo,sincronizar,corregir saldo,banco,extracto
casca-cmd-importar-holerite = Importar recibo de nómina
casca-cmd-nova-conta = Nueva cuenta
casca-cmd-nova-conta-busca = cuenta,corriente,ahorro,efectivo,banco,abrir cuenta
casca-cmd-novo-cartao = Nueva tarjeta de crédito
casca-cmd-novo-cartao-busca = tarjeta,credito,resumen,limite
casca-cmd-nova-meta = Nueva meta
casca-cmd-nova-meta-busca = meta,objetivo,sueño,ahorrar
casca-cmd-trocar-cofre = Cambiar de bóveda
casca-cmd-trocar-cofre-busca = boveda,base de datos,abrir,cambiar,otra boveda
casca-cmd-cofre-demo = Crear bóveda de demostración
casca-cmd-cofre-demo-busca = demostracion,demo,ejemplo,prueba,ficticio
casca-cmd-idioma = Idioma: { $idioma }
casca-cmd-idioma-busca = idioma,lengua,language,traducir
casca-cmd-entidade-conta-busca = cuenta,saldo,banco
casca-cmd-entidade-categoria-busca = categoria,gastos,movimientos
casca-cmd-entidade-meta-busca = meta,objetivo
casca-cmd-sin-holerite = recibo de nomina,nomina,sueldo,salario,ingreso,payslip
casca-cmd-sin-extrato = extracto,ofx,csv,qif,importar,banco
casca-cmd-sin-cupom = comprobante,factura,nfc-e,codigo qr,supermercado
casca-cmd-sin-aposentadoria = jubilacion,pension,prevision,futuro
casca-cmd-sin-bens = bienes,inmueble,casa,departamento,auto,vehiculo
casca-cmd-sin-proventos = dividendos,rendimientos,intereses,pagos
casca-cmd-sin-indices = indices,inflacion,tasa,tipo de cambio,dolar,cotizaciones
casca-cmd-sin-motores = motor,ia,modelo,groq,openai,anthropic,clave de api

casca-estudo-pronto = Estudio con IA listo

# Conferência da assinatura ao abrir (src-tauri/src/integrity.rs)
casca-assinatura-titulo = Treasury fue modificado
casca-atualizacao-nada-baixado = No hay ninguna actualización descargada para instalar.
casca-atualizacao-sem-yubikey = La versión nueva llegó sin la firma de la YubiKey de Treasury y no se aceptó.
casca-atualizacao-sem-chaves = La versión nueva no se aceptó: esta instalación no tiene las claves de la YubiKey de Treasury para comprobar la firma.
casca-atualizacao-yubikey-invalida = La firma de la YubiKey de la versión nueva no coincide ({ $motivo }); no se aceptó.
casca-atualizacao-minisign-invalida = La firma minisign de la versión nueva no coincide ({ $motivo }); no se aceptó.
casca-atualizacao-versao-assinada = La versión nueva se anunció como { $anunciada }, pero el paquete se firmó para { $assinada }; no se aceptó.
casca-atualizacao-instalar-manual = La instalación automática no terminó ({ $motivo }). El paquete, con las firmas verificadas, quedó en { $caminho }: instálalo con el instalador de paquetes del sistema.
casca-assinatura-alterada = La firma de este Treasury no coincide: la app en Aplicaciones puede haber sido reemplazada o modificada después de instalarla. No escribas la contraseña maestra en ella. Instala Treasury de nuevo desde el archivo original y ábrelo otra vez.

## Actividad de la app (state/activity.ts) y piezas de Fósforo

casca-atividade-importacao = Importación
casca-atividade-cupom = Lectura de ticket
casca-atividade-email = Lectura de correo
casca-atividade-cenario = Recálculo del escenario
casca-atividade-cotacoes = Sincronización de cotizaciones
casca-phosphor-processando = Procesando
casca-phosphor-tarefas = { $n } tareas
casca-phosphor-falha = Fallo
casca-phosphor-agente = > agente conectado... analizando datos
