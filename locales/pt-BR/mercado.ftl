# Mercado: busca de tickers, escolha de ativo, cotação intradiária e bolsas (docs/i18n-design.md).
# pt-BR é o idioma de origem.

## Busca de tickers

mercado-origem-cadastrado = cadastrado
mercado-origem-analise = análise
mercado-origem-yahoo = Yahoo
mercado-busca-placeholder = Ticker ou nome (PETR4, Vodafone, 7203…)
mercado-busca-rotulo = Buscar ticker
mercado-sugestoes = Sugestões de ativos
mercado-bolsa-fora = bolsa fora do catálogo
mercado-buscando = Buscando…
mercado-so-locais = { $erro }. Mostrando só os ativos locais.

## Escolha de ativo

mercado-ativo-novo = Ativo novo
mercado-bolsa-fora-catalogo = A bolsa { $bolsa } não está no catálogo: cadastre-a em Ativos › Bolsas.
mercado-cadastrar-usar = Cadastrar e usar

## Cotação intradiária

mercado-atraso = atraso de ~{ $min } min
mercado-agora = agora
mercado-intradiaria = Cotação intradiária
mercado-ultimo-negocio = Último negócio: { $quando }
mercado-ultimo-negocio-bolsa = Último negócio: { $quando } ({ $hora } na bolsa)
mercado-fonte-yahoo = Yahoo Finance, sem chave, com cerca de { $min } min de atraso. O histórico usa os fechamentos diários.
mercado-cotacao-com = Cotação com { $atraso }
mercado-cotacao-com-estado = Cotação com { $atraso }, { $estado }

## Gráfico intradiário

mercado-fech-anterior = Fech. anterior
mercado-buscando-cotacao = Buscando cotação…
mercado-sem-intradiaria = Sem cotação intradiária ainda.
mercado-periodo-intradiario = Período intradiário
mercado-periodo-1d = 1D
mercado-periodo-5d = 5D
mercado-nao-buscou = Não foi possível buscar a cotação agora.
mercado-intradiaria-aria = { $dias ->
    [1] Cotação intradiária de { $nome } (1 dia)
   *[other] Cotação intradiária de { $nome } ({ $dias } dias)
}
mercado-sem-pontos = Sem pontos intradiários. Eles chegam com o pregão aberto (a cada 5 min).

## Bolsas (Ativos › Bolsas)

mercado-dias-seg-sex = seg–sex
mercado-dias-dom-qui = dom–qui
mercado-dias-curtos = seg,ter,qua,qui,sex,sáb,dom
mercado-almoco = {" "}(almoço { $inicio }–{ $fim })
mercado-bolsa = Bolsa
mercado-pais = País
mercado-moeda = Moeda
mercado-cambio-manual = câmbio manual
mercado-cambio-manual-dica = Sem PTAX no Banco Central nem câmbio de referência do BCE: informe o câmbio manualmente.
mercado-sem-sufixo = sem sufixo
mercado-sem-sufixo-dica = O símbolo no Yahoo é o próprio ticker
mercado-pregao-local = Pregão (local)
mercado-agora-col = Agora
mercado-hora-local = { $hora } local
mercado-remover-bolsa = Remover bolsa
mercado-remover-bolsa-descricao = { $codigo } sai do catálogo (só se não houver ativos nela).
mercado-bolsa-removida = Bolsa removida
mercado-filtrar-bolsas = Filtrar bolsas
mercado-filtrar-placeholder = Filtrar por código, país, moeda…
mercado-resumo = { $n } bolsas · { $abertas } em pregão agora
mercado-cadastrar-bolsa = Cadastrar bolsa
mercado-bolsas-ajuda = O código da bolsa é o “mercado” do ativo: dele vêm a moeda e o país padrão, o símbolo do Yahoo (ticker + sufixo) e o horário da coleta intradiária (a cada 5 min, com ~15 min de atraso, só com o pregão aberto). Feriados não estão no catálogo.
mercado-nenhuma-bolsa = Nenhuma bolsa
mercado-nada-filtro = Nada bate com o filtro.
mercado-dialogo-descricao = Horário do pregão regular no fuso da bolsa (IANA, ex.: Europe/London).
mercado-erro-codigo = Até 12 letras e números.
mercado-erro-nome = Informe o nome.
mercado-erro-pais = Código ISO de 2 letras.
mercado-erro-moeda = Código ISO de 3 letras.
mercado-erro-fuso = Informe o fuso (ex.: Asia/Manila).
mercado-bolsa-atualizada = Bolsa atualizada
mercado-bolsa-cadastrada = Bolsa cadastrada
mercado-iso-2 = ISO de 2 letras.
mercado-iso-3 = ISO de 3 letras.
mercado-sufixo = Sufixo no Yahoo
mercado-sufixo-dica = Ex.: .PS; vazio nos EUA.
mercado-fuso = Fuso (IANA)
mercado-mic-dica = Opcional (ISO 10383).
mercado-abertura = Abertura
mercado-dias-pregao = Dias de pregão
mercado-dias-pregao-dica = 1 = segunda … 7 = domingo.
mercado-almoco-inicio = Almoço: início
mercado-almoco-fim = Almoço: fim
mercado-apelidos = Apelidos
mercado-apelidos-dica = Separados por vírgula (códigos do Yahoo, nomes).
mercado-salvar-bolsa = Salvar bolsa

## Tela Mercado (docs/design/market-explorer.md)

mercado-secoes = Seções do Mercado
mercado-aba-panorama = Panorama
mercado-aba-explorador = Explorador
mercado-aba-mapa = Mapa
mercado-aviso = Lista de quem cumpre os critérios, a partir de dados públicos. Não é recomendação de investimento.
mercado-panorama-titulo = O mercado hoje
mercado-acoes = Ações
mercado-fiis = FIIs
mercado-todos = Todos
mercado-com-score = Com score
mercado-score-de = Score calculado em { $data }
mercado-score-pendente = O score do mercado sai na próxima coleta
mercado-dados = Dados de
mercado-coleta-em = Coleta em { $data }
mercado-sem-coleta = A coleta ainda não rodou
mercado-panorama-de = Panorama de
mercado-acoes-por-setor = Ações por setor
mercado-fiis-por-segmento = FIIs por segmento
mercado-col-setor = Setor
mercado-col-segmento = Segmento
mercado-col-papeis = Papéis
mercado-col-saudaveis = Robustos e saudáveis
mercado-col-score = Score (mediana)
mercado-col-pl = P/L (mediana)
mercado-col-pvp = P/VP (mediana)
mercado-col-dy = DY (mediana)
mercado-panorama-rodape = Medianas do setor (P/L e P/VP só com valores positivos). Clique no nome de um setor (ou dê dois cliques na linha) para abrir o explorador com os papéis saudáveis dele.
mercado-sem-universo = O universo ainda não foi baixado
mercado-sem-universo-texto = Rode a coleta da análise de ativos em Ativos › Dados para trazer as ações e os FIIs da B3.
mercado-mapa-titulo = Saúde × Atratividade
mercado-mapa-saude = Saúde
mercado-mapa-atratividade = Atratividade
mercado-mapa-classe = Classe no mapa
mercado-mapa-aria = Mapa de { $n } papéis: saúde no eixo vertical, atratividade no horizontal
mercado-mapa-vazio = Sem score do mercado ainda
mercado-mapa-vazio-texto = O mapa usa o score que a coleta calcula para todo o universo. Rode a coleta em Ativos › Dados.
mercado-mapa-leitura = Como ler o mapa
mercado-mapa-leitura-texto = Cada bolha é um papel, do tamanho do valor de mercado e na cor da faixa do score. O canto de cima à direita junta saúde e atratividade altas. Contorno marca os papéis da carteira e da watchlist. Clique para abrir o papel.
mercado-aba-metodos = Métodos
mercado-metodo-criterios = Critérios
mercado-metodo-opcional = (sem o dado, não derruba)
mercado-metodo-adaptado = Adaptado à B3. Original: { $original }
mercado-metodo-higiene = Higiene
mercado-metodo-ordem = Ordem
mercado-metodo-nao-se-aplica = Não se aplica a
mercado-metodo-fonte = Fonte
mercado-col-score-papel = Score
mercado-metodo-por-que = Por que um papel passa ou fica de fora
mercado-metodo-por-que-exemplo = Ex.: PETR4
mercado-metodo-explicar = Explicar
mercado-metodo-passa = { $ticker } cumpre todos os critérios ({ $rank }º na ordem do método).
mercado-metodo-fora = { $ticker } fica de fora antes dos critérios: { $motivo }.
mercado-metodo-nao-passa = { $ticker } não cumpre todos os critérios:
mercado-metodo-sem-dado = sem o dado
mercado-metodo-voltar = Todos os métodos
mercado-metodo-lista = Quem cumpre { $metodo }
mercado-metodo-resultado = { $passam } de { $avaliados } papéis avaliados cumprem todos os critérios hoje.
mercado-metodo-ninguem = Nenhum papel cumpre todos os critérios hoje.
mercado-metodo-cumprem = { $n } papel(éis) cumprem os critérios hoje
mercado-metodo-contando = Contando…
mercado-metodo-ver-lista = Ver a receita e a lista
mercado-metodo-fonte-versao = { $fonte } · v{ $versao }
mercado-abrir-setor = Abrir { $setor } no explorador
