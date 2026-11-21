# Variante pt-PT (opcional) da análise de ativos: o que faltar cai para pt-BR
# (docs/i18n-design.md). Revisão: rascunho por IA, ainda sem revisão de um falante nativo.

## Seção da análise e análise por ticker

pesquisa-analisar-papel = Analisar título
pesquisa-ticker-exemplo-analise = Ticker (p. ex.: PETR4, HGLG11, AAPL)
pesquisa-n-papeis-universo = { $n } títulos no universo
pesquisa-escolha-papel = Escolha um título
pesquisa-escolha-papel-texto = Escreva o ticker de qualquer ação ou FII do universo descarregado (registado ou não) para ver indicadores, preço calculado, histórico e factos relevantes.

## Universo vazio e ativo sem dados

pesquisa-sem-coleta = Sem recolha
pesquisa-sem-universo = Ainda não há títulos no universo
pesquisa-sem-universo-texto = { $quem ->
    [ticker] A análise por ticker funciona sobre o universo de ações e FII descarregado da CVM, da B3 e da SEC. Execute a primeira recolha (cerca de 230 MB, 1 a 2 minutos) em Dados da análise; depois tudo funciona sem ligação.
    [screener] O screener funciona sobre o universo de ações e FII descarregado da CVM, da B3 e da SEC. Execute a primeira recolha (cerca de 230 MB, 1 a 2 minutos) em Dados da análise; depois tudo funciona sem ligação.
    [rankings] Os rankings funcionam sobre o universo de ações e FII descarregado da CVM, da B3 e da SEC. Execute a primeira recolha (cerca de 230 MB, 1 a 2 minutos) em Dados da análise; depois tudo funciona sem ligação.
   *[geral] Os rankings e o screener funcionam sobre o universo de ações e FII descarregado da CVM, da B3 e da SEC. Execute a primeira recolha (cerca de 230 MB, 1 a 2 minutos) em Dados da análise; depois tudo funciona sem ligação.
}
pesquisa-sem-dados-ativo-texto = { $erro }. Os fundamentos vêm da recolha da CVM, da B3 e da SEC.

## Indicadores

pesquisa-indicadores-dica = Passe o rato ou clique no nome de um indicador para ver a fórmula, o período e a fonte. Lucro e capitais próprios são os dos acionistas de controlo; fluxos dos últimos 12 meses (DFP + ITR).

## Documentos

pesquisa-doc-fatos = Factos relevantes
pesquisa-doc-vazio-texto = Nada nas categorias e no período escolhidos.
pesquisa-doc-rodape = O assunto e o resumo são textos da própria empresa, mostrados tal como estão. O documento oficial abre no site da CVM.

## Histórico

pesquisa-hist-vazio-texto = O histórico vem das DFP da CVM (ou do XBRL da SEC). Execute a recolha em Dados da análise.

## Comparador

pesquisa-cmp-sem-historico-texto = A cotação normalizada usa o histórico dos ativos registados (Yahoo Finance). Os títulos só do universo têm cotação de poucas sessões.
pesquisa-cmp-ticker-exemplo = Ticker (p. ex.: VALE3)
pesquisa-cmp-tirar = Retirar { $ticker } da comparação
pesquisa-cmp-escolha = Escolha de 2 a 6 títulos
pesquisa-cmp-papeis = Títulos comparados

## Screener

pesquisa-scr-salvar-titulo = Guardar vista do screener
pesquisa-scr-salvar-texto = A vista guarda os filtros e o recorte; execute-a de novo quando quiser (os dados do dia mudam).
pesquisa-scr-salvar-rotulo = Guardar vista
pesquisa-scr-salvar = Guardar vista
pesquisa-scr-tela-salva = Vista “{ $nome }” guardada
pesquisa-scr-tela-atualizada = Vista “{ $nome }” atualizada
pesquisa-scr-tela-excluida = Vista “{ $nome }” eliminada
pesquisa-scr-erro-excluir = Não foi possível eliminar
pesquisa-scr-universo-vazio-texto = Já pode montar e guardar vistas; os resultados aparecem depois da primeira recolha em Dados da análise.
pesquisa-scr-tela = Vista · { $nome }
pesquisa-scr-telas-salvas = Vistas guardadas
pesquisa-scr-abrir-tela = Abrir vista guardada…
pesquisa-scr-nenhuma-tela = Nenhuma vista guardada
pesquisa-scr-salva-em = Guardada: { $resumo } · alterada em { $data }
pesquisa-scr-rodar = Executar screener
pesquisa-scr-salvar-alteracoes = Guardar alterações
pesquisa-scr-salvar-nova = Guardar como nova…
pesquisa-scr-salvar-tela = Guardar vista…
pesquisa-scr-nova = Nova vista
pesquisa-scr-excluir = Eliminar vista
pesquisa-scr-excluir-texto = A vista “{ $nome }” sai da lista. Pode anular logo a seguir ou em Ajustes › Histórico de alterações.
pesquisa-scr-desfazer = Anular
pesquisa-scr-desfazer-texto = Também pode anular em Ajustes › Histórico de alterações.
pesquisa-scr-desfeita = Alteração anulada
pesquisa-scr-erro-desfazer = Não foi possível anular
pesquisa-scr-instrucoes = Monte os filtros e clique em { $botao }. Percentagens em % (DY ≥ 6), múltiplos em vezes (P/L < 12), dinheiro na moeda do ativo.
pesquisa-scr-rodando = A executar o screener
pesquisa-scr-n-papeis = { $n ->
    [1] { $n } título
   *[other] { $n } títulos
}
pesquisa-scr-nenhum = Nenhum título passou nos filtros.

## Rankings

pesquisa-rank-montando = A montar o ranking
pesquisa-rank-vazio = Nenhum título do recorte tem este indicador.
pesquisa-rank-total = { $n } títulos com o indicador

## Dados da análise

pesquisa-dados-coleta-fonte = Recolha por fonte
pesquisa-dados-aviso = Aviso da recolha
pesquisa-dados-concluida = Recolha da análise concluída
pesquisa-dados-concluida-avisos = Recolha concluída com avisos
pesquisa-dados-concluida-texto = { $tamanho } descarregados · { $n } títulos no universo
pesquisa-dados-coleta-titulo = Recolha da análise de ativos
pesquisa-dados-coletando-live = A RECOLHER
pesquisa-dados-primeira-coleta = Primeira recolha
pesquisa-dados-ultima-coleta = Última recolha
pesquisa-dados-universo-valor = { $n } títulos · { $data }
pesquisa-dados-papeis = Títulos
pesquisa-dados-coletando = A recolher…
pesquisa-dados-coletar-agora = Recolher agora
pesquisa-dados-baixando = A descarregar e calcular
pesquisa-dados-falhou = A recolha falhou
pesquisa-dados-ultima-por-fonte = Última recolha por fonte
pesquisa-dados-resumo = { $qual ->
    [esta] Esta recolha
   *[ultima] Última recolha
}: { $tamanho } em { $segundos } s · { $universo } títulos no universo · { $radar } rendimentos projetados pelo radar
pesquisa-dados-coletar-titulo = Recolher dados da análise
pesquisa-dados-coletar-primeira = A primeira recolha descarrega cerca de 230 MB da CVM (DFP, ITR, FCA, FII, IPE), da B3 (COTAHIST) e da SEC e demora 1 a 2 minutos. Nada fica em disco além da base de dados cifrada.
pesquisa-dados-coletar-incremental = Descarrega só os ficheiros que mudaram desde a última recolha (normalmente poucos MB) e recalcula o universo e o radar de dividendos.
pesquisa-dados-baixar-230 = Descarregar ~230 MB
pesquisa-dados-coletar = Recolher
pesquisa-dados-baixar-tudo = Descarregar tudo de novo
pesquisa-dados-ajustes-salvos = Ajustes da análise guardados
pesquisa-dados-sec-email = Configure o e-mail da SEC
pesquisa-dados-sec-ua-erro = Inclua um e-mail (p. ex.: Treasury/0.1 o.seu.nome@email.com).
pesquisa-dados-sec-ua-exemplo = Treasury/0.1 o.seu.nome@email.com
pesquisa-dados-bazin-dica = Premissas predefinidas do preço-teto de Bazin (média dos dividendos por ação ÷ taxa mínima); pode alterá-las caso a caso no separador Preço calculado do ativo.
pesquisa-dados-salvar-ajustes = Guardar ajustes
pesquisa-dados-arquivos = Ficheiros
