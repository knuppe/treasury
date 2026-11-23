# Variante pt-PT (opcional) de 09 · Ativos: o que faltar cai para pt-BR (docs/i18n-design.md).
# Revisão: rascunho por IA, ainda sem revisão de um falante nativo.

## Ecrã e separadores

ativos-cadastrar = Registar ativo
ativos-aba-agenda = Calendário de dividendos

## Menu de contexto do ativo

ativos-menu-registrar-provento = Registar provento
ativos-menu-editar = Editar registo…

## Lista de ativos

ativos-vazio-titulo = Nenhum ativo registado
ativos-vazio-descricao = Registe ações, FII, ETF, títulos do Tesouro, CDB, ativos no estrangeiro ou cripto para registar operações e acompanhar.
ativos-tabela = Ativos registados
ativos-posicao-todos = Todos os registados

## Página do ativo

ativos-cotacao-descricao = Para ativos sem fonte automática (fundos, previdência, títulos privados). Fica marcada como “Informado por si”.
ativos-informe-data = Indique a data.
ativos-informe-preco = Indique o preço.
ativos-salvar-cotacao = Guardar cotação
ativos-aba-cadastro = Registo
ativos-sem-posicao-texto = Registe uma compra para acompanhar o preço médio, o resultado e o retorno real.
ativos-eventos-ajuda = Desdobramento: fator maior que 1 (1 ação passa a 5 = fator 5). Agrupamento: fator menor que 1 (10 passam a 1 = 0,1). Bonificação: ações novas por ação (10% = 0,1). Registe cada facto uma só vez: se o extrato já traz a entrada a custo zero, não registe o evento.
ativos-grafico-sem-cotacoes = Sem cotações: a linha liga os preços das suas operações
ativos-sem-historico-texto = O gráfico mostra o preço de cada operação. Atualize as cotações em Índices e cotações ou indique a cotação manualmente.
ativos-excluir-descricao = { $nome } sai do registo, junto com a watchlist e os eventos dele. Ativos com operações ou proventos só podem ser arquivados.
ativos-nao-excluiu = Não foi possível eliminar
ativos-excluir = Eliminar ativo
ativos-excluido = Ativo eliminado

## Registo e edição de ativo

ativos-fonte-manual = Manual (eu indico)
ativos-tipo-other = Qualquer outro investimento: indique a moeda e, se quiser, o símbolo da cotação.
ativos-cnpj-fie-dica = Do fundo do plano, no extrato da seguradora: para a cota diária da CVM. Sem ele, indique a cota em Cotação.
ativos-grupo-exterior = Estrangeiro
ativos-form-editar-descricao = Mercado, moeda, país e identificadores não mudam depois do registo.
ativos-form-erro-nome = Indique o nome do ativo.
ativos-form-erro-taxa = Indique a taxa.
ativos-form-erro-bolsa = Escolha a bolsa (ou registe-a em Ativos › Bolsas).
ativos-form-cadastrado = Ativo registado
ativos-form-ou-busque = Ou procure pelo ticker ou nome:
ativos-form-buscar = Procurar ativo
ativos-form-preenchido-cadastrado = Preenchido com { $ativo }. Este ativo já está registado.
ativos-form-informar-termos = Indicar indexador e taxa
ativos-form-salvar = Guardar ativo
ativos-buscar-rotulo = Procurar ativo por ticker ou nome
ativos-buscar = Procurar por ticker ou nome…

## Proventos, watchlist e extrato

ativos-anunciados-novos-dica = Os que tinha direito aparecem em "Proventos a confirmar" para lançar.
ativos-anunciados-buscar = Procurar no Fundamentus
ativos-watch-buscar-dica = Ticker ou nome; um ativo novo é registado ao escolher.
ativos-watch-remover-descricao = { $nome } sai da watchlist; o registo do ativo continua.
ativos-watch-vazia-descricao = Acompanhe ativos que ainda não tem, com preço-alvo e alertas de cotação.

## Previdência

ativos-prev-salvar-plano = Guardar plano
ativos-prev-plano-salvo = Plano de previdência guardado
ativos-prev-nenhum-aporte = Nenhum aporte: registe os aportes como operações de compra do plano.
ativos-prev-resgatar-tudo = Se resgatar tudo hoje
ativos-prev-importar-descricao = O PDF do extrato do participante (fundo de pensões, como a Quanta Previdência): confere a pré-visualização antes de gravar.
ativos-prev-crie-corretora = Crie uma conta do tipo corretora
ativos-plano-efpc = Fundo de pensões

## Proventos a confirmar e calendário

ativos-pendentes-ajuda = Proventos anunciados com data-com em que a família tinha o ativo e que ainda não estão na carteira. Confirmar lança o crédito no caixa da corretora; dispensar esconde até reabrir a aplicação.
ativos-agenda-sua-qtd = A sua qtd.
ativos-agenda-so-minha = Só a minha carteira
ativos-agenda-de = Calendário de { $mes }
ativos-agenda-periodo = Período do calendário
ativos-agenda-vazia = Nenhum provento no calendário
ativos-agenda-vazia-descricao = O calendário vem dos proventos recolhidos com as cotações (Yahoo Finance) e das projeções do radar. Atualize as cotações em Índices e cotações, ou alargue o período.
ativos-agenda-ajuda = Previsto = a sua quantidade na data-com (ou hoje, se ainda não chegou) × valor por cota; JCP já com o IR retido (17,5% desde 2026; 15% até 2025). As projeções do radar usam o histórico de pagamentos e mudam quando o provento é anunciado.

# Página dedicada do ativo (docs/asset-page.md).
ativo-trilha = Trilho
ativo-perguntar-agente = Perguntar ao agente
ativo-agente-pergunta = Analise { $ativo } com os dados que o Treasury tem dele (posição, fundamentos, preço calculado, saúde, documentos e estudos) e diga o que chama a atenção. Se encontrar dados novos e conferidos, proponha completar o registo.
ativo-aba-conversas = Conversas
ativo-conversas-texto = As conversas com o agente começadas nesta página ficam aqui. As análises completas, com fontes conferidas, ficam em Estudo (IA).
ativo-ver-estudos = Ver estudos
ativo-conversas-vazio = Ainda não há conversas sobre este ativo
ativo-conversas-vazio-texto = Use "Perguntar ao agente" para conversar sobre { $ativo }; a conversa fica guardada aqui.
ativo-conversa-detalhe =
    { $n ->
        [one] { $quando } · { $n } mensagem
       *[other] { $quando } · { $n } mensagens
    }
ativo-abrir-conversa = Abrir

# Visão geral e notícias do ativo (docs/asset-overview.md).
ativo-visao-geral = Visão geral
ativo-empresa = Empresa
ativo-setor = Setor
ativo-cotacao = Cotação
ativo-cotacao-em = em { $data } ({ $fonte })
ativo-score = Score de saúde
ativo-visao-geral-sem-dados = Ainda não há dados de mercado deste ativo (cotação e indicadores vêm da recolha da análise de ativos).
ativo-indicadores-chave = Indicadores principais
ativo-indicadores-sem = Sem indicadores para este ativo.
ativo-ver-fundamentos = Ver todos os fundamentos
ativo-ultimas = Últimas notícias e documentos
ativo-ultimas-vazio = Nada recente por aqui.
ativo-ver-noticias = Ver todas as notícias
ativo-aba-noticias = Notícias
ativo-noticias-texto = Documentos oficiais da CVM e títulos de portais de economia, do mais recente para o mais antigo. Os títulos dos portais são filtrados no computador: nada da sua carteira sai dele. Só título, órgão, data e ligação.
ativo-noticias-google-dica = Quer mais notícias? A pesquisa do Google Notícias abre no seu navegador: o Treasury não consulta o Google.
ativo-noticias-abrir-google = Abrir no Google Notícias
ativo-noticias-vazio = Nenhuma notícia recente
ativo-noticias-vazio-texto = Os portais trazem só os últimos títulos de cada dia; os documentos da CVM aparecem depois da recolha da análise de ativos.
ativo-noticias-falhou = Não foi possível ler agora: { $fontes }.
ativo-noticia-origem-cvm = CVM
ativo-noticia-origem-portal = portal
ativo-resumo-do-ativo = Resumo do ativo
