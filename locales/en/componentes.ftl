# Shared interface components (src/design): tables, dialogs, notices, amounts and data sources
# (docs/i18n-design.md). Revision: AI draft, not yet reviewed by a native speaker.

## Tables

componentes-copiar-linha = Copy row
componentes-linhas-selecionadas = { $n } rows selected
componentes-copiar-linhas = Copy { $n } rows
componentes-linhas-copiadas = { $n ->
        [one] Row copied
       *[other] { $n } rows copied
    }
componentes-copiadas-dica = Visible columns separated by tabs: paste into a spreadsheet.
componentes-nao-copiou = Couldn't copy
componentes-acoes-linhas = Actions for { $n } rows
componentes-acoes-linha = Row actions
componentes-nenhuma-linha = No rows.
componentes-ordenar-crescente = Sort ascending
componentes-ordenar-decrescente = Sort descending
componentes-tirar-ordenacao = Remove sorting
componentes-esconder-coluna = Hide "{ $coluna }"
componentes-larguras-padrao = Default widths
componentes-larguras-automaticas = Automatic widths
componentes-restaurar-colunas = Restore columns and order
componentes-ordenar-por = Sort by { $coluna }
componentes-largura-coluna = Width of column { $coluna }
componentes-largura-dica-padrao = Drag to change the width · double-click: default widths
componentes-largura-dica-automaticas = Drag to change the width · double-click: automatic widths
componentes-colunas-de = Columns of { $tabela }
componentes-colunas-visiveis = Visible columns
componentes-colunas = Columns
componentes-restaurar-tudo = Restore columns, widths and order

## Dialogs and notices

componentes-restaurar-tamanho = Restore size
componentes-maximizar = Maximize
componentes-redimensionar = Resize the window (drag; double-click returns to the default size)
componentes-redimensionar-dica = Drag to resize · double-click returns to the default
componentes-notificacoes = Notifications
componentes-fechar-notificacao = Close notification
componentes-toast-aviso = Notice
componentes-toast-feito = Done
componentes-toast-atencao = Warning
componentes-toast-erro = Error
componentes-ocultar-senha = Hide password
componentes-mostrar-senha = Show password
componentes-sem-dados = No data
componentes-ao-vivo = Live
componentes-figura = Fig. { $numero }
componentes-link-enganoso = The link text isn't the real address: { $endereco }
componentes-link-abre = opens
componentes-o-que-e = What is { $termo }?

## Amounts and data sources

componentes-valor-oculto = hidden amount
componentes-aproximadamente = approximately
componentes-valor-estimado = Estimated amount
componentes-origem-titulo = Where this number comes from
componentes-fonte = Source
componentes-data = Date
componentes-status = Status
componentes-estimado = ≈ Estimated
componentes-oficial = Official
componentes-origem-falada = Source: { $fonte }
componentes-origem-data-falada = date { $data }
componentes-valor-estimado-falado = estimated amount
componentes-valor-oficial-falado = official amount

## Keys and undo

componentes-tecla-espaco = Space
componentes-desfazer = Undo
componentes-desfeito = Change undone
componentes-nao-desfez = Couldn't undo

## Trading session (lib/marketHours.ts)

componentes-pregao-pre = Pre-market
componentes-pregao-aberto = Market open
componentes-pregao-intervalo = Midday break
componentes-pregao-encerrado = Market closed
componentes-pregao-sem = No trading

componentes-nucleo-falha = Unknown failure talking to the core.
componentes-nucleo-sem-conexao = Couldn't connect to the Treasury core. Is it running?
componentes-nucleo-resposta-invalida = Invalid response from the core (HTTP { $status }).
componentes-nucleo-resposta-inesperada = Unexpected response from the core (HTTP { $status }).
