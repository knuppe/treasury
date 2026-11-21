# Componentes comunes de la interfaz (src/design): tablas, diálogos, avisos, valores y origen de
# los datos (docs/i18n-design.md). Revisión: borrador por IA, sin revisión nativa.

## Tablas

componentes-copiar-linha = Copiar fila
componentes-linhas-selecionadas = { $n } filas seleccionadas
componentes-copiar-linhas = Copiar { $n } filas
componentes-linhas-copiadas = { $n ->
        [one] Fila copiada
       *[other] { $n } filas copiadas
    }
componentes-copiadas-dica = Columnas visibles separadas por tabulación: pégalas en una hoja de cálculo.
componentes-nao-copiou = No se pudo copiar
componentes-acoes-linhas = Acciones de { $n } filas
componentes-acoes-linha = Acciones de la fila
componentes-nenhuma-linha = Ninguna fila.
componentes-ordenar-crescente = Ordenar de menor a mayor
componentes-ordenar-decrescente = Ordenar de mayor a menor
componentes-tirar-ordenacao = Quitar el orden
componentes-esconder-coluna = Ocultar "{ $coluna }"
componentes-larguras-padrao = Anchos predeterminados
componentes-larguras-automaticas = Anchos automáticos
componentes-restaurar-colunas = Restaurar columnas y orden
componentes-ordenar-por = Ordenar por { $coluna }
componentes-largura-coluna = Ancho de la columna { $coluna }
componentes-largura-dica-padrao = Arrastra para cambiar el ancho · doble clic: anchos predeterminados
componentes-largura-dica-automaticas = Arrastra para cambiar el ancho · doble clic: anchos automáticos
componentes-colunas-de = Columnas de { $tabela }
componentes-colunas-visiveis = Columnas visibles
componentes-colunas = Columnas
componentes-restaurar-tudo = Restaurar columnas, anchos y orden

## Diálogos y avisos

componentes-restaurar-tamanho = Restaurar tamaño
componentes-maximizar = Maximizar
componentes-redimensionar = Cambiar el tamaño de la ventana (arrastra; doble clic vuelve al tamaño predeterminado)
componentes-redimensionar-dica = Arrastra para cambiar el tamaño · doble clic vuelve al predeterminado
componentes-notificacoes = Notificaciones
componentes-fechar-notificacao = Cerrar notificación
componentes-toast-aviso = Aviso
componentes-toast-feito = Listo
componentes-toast-atencao = Atención
componentes-toast-erro = Error
componentes-ocultar-senha = Ocultar contraseña
componentes-mostrar-senha = Mostrar contraseña
componentes-sem-dados = Sin datos
componentes-ao-vivo = En vivo
componentes-figura = Fig. { $numero }
componentes-link-enganoso = El texto del enlace no es la dirección real: { $endereco }
componentes-link-abre = abre
componentes-o-que-e = ¿Qué es { $termo }?

## Valores y origen de los datos

componentes-valor-oculto = valor oculto
componentes-aproximadamente = aproximadamente
componentes-valor-estimado = Valor estimado
componentes-origem-titulo = Origen del valor
componentes-fonte = Fuente
componentes-data = Fecha
componentes-status = Estado
componentes-estimado = ≈ Estimado
componentes-oficial = Oficial
componentes-origem-falada = Origen: { $fonte }
componentes-origem-data-falada = fecha { $data }
componentes-valor-estimado-falado = valor estimado
componentes-valor-oficial-falado = valor oficial

## Teclas y deshacer

componentes-tecla-espaco = Espacio
componentes-desfazer = Deshacer
componentes-desfeito = Cambio deshecho
componentes-nao-desfez = No se pudo deshacer

## Sesión bursátil (lib/marketHours.ts)

componentes-pregao-pre = Preapertura
componentes-pregao-aberto = Mercado abierto
componentes-pregao-intervalo = Pausa
componentes-pregao-encerrado = Mercado cerrado
componentes-pregao-sem = Sin sesión

componentes-nucleo-falha = Falla desconocida al comunicarse con el núcleo.
componentes-nucleo-sem-conexao = No se pudo conectar con el núcleo de Treasury. ¿Está en ejecución?
componentes-nucleo-resposta-invalida = Respuesta inválida del núcleo (HTTP { $status }).
componentes-nucleo-resposta-inesperada = Respuesta inesperada del núcleo (HTTP { $status }).
