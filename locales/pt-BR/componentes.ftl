# Componentes comuns da interface (src/design): tabelas, diálogos, avisos, valores e origem dos
# dados (docs/i18n-design.md). pt-BR é o idioma de origem.

## Tabelas

componentes-copiar-linha = Copiar linha
componentes-linhas-selecionadas = { $n } linhas selecionadas
componentes-copiar-linhas = Copiar { $n } linhas
componentes-linhas-copiadas = { $n ->
        [one] Linha copiada
       *[other] { $n } linhas copiadas
    }
componentes-copiadas-dica = Colunas visíveis separadas por tabulação: cole numa planilha.
componentes-nao-copiou = Não foi possível copiar
componentes-acoes-linhas = Ações de { $n } linhas
componentes-acoes-linha = Ações da linha
componentes-nenhuma-linha = Nenhuma linha.
componentes-ordenar-crescente = Ordenar crescente
componentes-ordenar-decrescente = Ordenar decrescente
componentes-tirar-ordenacao = Tirar a ordenação
componentes-esconder-coluna = Esconder "{ $coluna }"
componentes-larguras-padrao = Larguras padrão
componentes-larguras-automaticas = Larguras automáticas
componentes-restaurar-colunas = Restaurar colunas e ordem
componentes-ordenar-por = Ordenar por { $coluna }
componentes-largura-coluna = Largura da coluna { $coluna }
componentes-largura-dica-padrao = Arraste para mudar a largura · duplo clique: larguras padrão
componentes-largura-dica-automaticas = Arraste para mudar a largura · duplo clique: larguras automáticas
componentes-colunas-de = Colunas de { $tabela }
componentes-colunas-visiveis = Colunas visíveis
componentes-colunas = Colunas
componentes-restaurar-tudo = Restaurar colunas, larguras e ordem

## Diálogos e avisos

componentes-restaurar-tamanho = Restaurar tamanho
componentes-maximizar = Maximizar
componentes-redimensionar = Redimensionar a janela (arraste; duplo clique volta ao tamanho padrão)
componentes-redimensionar-dica = Arraste para redimensionar · duplo clique volta ao padrão
componentes-notificacoes = Notificações
componentes-fechar-notificacao = Fechar notificação
componentes-toast-aviso = Aviso
componentes-toast-feito = Feito
componentes-toast-atencao = Atenção
componentes-toast-erro = Erro
componentes-ocultar-senha = Ocultar senha
componentes-mostrar-senha = Mostrar senha
componentes-sem-dados = Sem dados
componentes-ao-vivo = Ao vivo
componentes-figura = Fig. { $numero }
componentes-link-enganoso = O texto do link não é o endereço de verdade: { $endereco }
componentes-link-abre = abre
componentes-o-que-e = O que é { $termo }?

## Valores e origem dos dados

componentes-valor-oculto = valor oculto
componentes-aproximadamente = aproximadamente
componentes-valor-estimado = Valor estimado
componentes-origem-titulo = Origem do valor
componentes-fonte = Fonte
componentes-data = Data
componentes-status = Status
componentes-estimado = ≈ Estimado
componentes-oficial = Oficial
componentes-origem-falada = Origem: { $fonte }
componentes-origem-data-falada = data { $data }
componentes-valor-estimado-falado = valor estimado
componentes-valor-oficial-falado = valor oficial

## Teclas e desfazer

componentes-tecla-espaco = Espaço
componentes-desfazer = Desfazer
componentes-desfeito = Alteração desfeita
componentes-nao-desfez = Não foi possível desfazer

## Pregão (lib/marketHours.ts)

componentes-pregao-pre = Pré-abertura
componentes-pregao-aberto = Pregão aberto
componentes-pregao-intervalo = Intervalo
componentes-pregao-encerrado = Pregão encerrado
componentes-pregao-sem = Sem pregão

componentes-nucleo-falha = Falha desconhecida ao falar com o núcleo.
componentes-nucleo-sem-conexao = Não foi possível conectar ao núcleo do Treasury. Ele está rodando?
componentes-nucleo-resposta-invalida = Resposta inválida do núcleo (HTTP { $status }).
componentes-nucleo-resposta-inesperada = Resposta inesperada do núcleo (HTTP { $status }).
