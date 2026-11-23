# Variante pt-PT (ecrã Ajustes e as suas secções): o que faltar cai para pt-BR.
# Revisão: rascunho por IA. Trechos entre ** saem a negrito; os argumentos link, arquivo, icone e
# url são elementos que o ecrã coloca no lugar.

## Ecrã: pesquisa, visão geral e avisos

ajustes-descricao = Tudo o que pode configurar, em cinco grupos. Pesquise o que procura ou escolha um grupo.
ajustes-secoes = Secções de ajustes
ajustes-buscar = Pesquisar nos ajustes
ajustes-buscar-placeholder = Pesquisar nos ajustes…
ajustes-busca-vazia = Nada com “{ $busca }”. Tente outra palavra (ex.: palavra-passe, tema, cópia de segurança).
ajustes-aviso-backup = Mais de 48 h sem cópia de segurança válida

## Grupos

ajustes-grupo-voce = Perfil e família
ajustes-grupo-voce-dica = Como a aplicação comunica consigo, quem é da família e o que avisar
ajustes-grupo-seguranca-dica = Palavra-passe, cofres, cópias de segurança, histórico, exportação e a base de dados
ajustes-grupo-treasury-dica = Sobre a aplicação e como apoiar quem a faz

## Secções (as palavras de pesquisa vão separadas por vírgula)

ajustes-secao-perfil-busca = perfil,nível,iniciante,intermédio,intermediário,avançado,linguagem,ajuda
ajustes-secao-familia-busca = titular,membro,banco,instituição,pessoa,país,moeda,onde vive,onde mora
ajustes-secao-seguranca-dica = Palavra-passe mestra, PIN, kit de recuperação e bloqueio automático
ajustes-secao-seguranca-busca = palavra-passe,senha,keychain,porta-chaves,recuperação,pin,bloqueio,touch id
ajustes-secao-cofres-dica = Várias bases de dados: criar, abrir e trocar
ajustes-secao-cofres-busca = cofre,base de dados,banco de dados,ficheiro,arquivo,.db,trocar,empresa,demonstração
ajustes-secao-backups = Cópias de segurança
ajustes-secao-backups-busca = backup,cópia de segurança,restaurar,cópia
ajustes-secao-historico-busca = desfazer,auditoria,alterações,mudanças,diff
ajustes-secao-dados-dica = Tudo em JSON ou CSV, para guardar ou levar para outra aplicação
ajustes-secao-dados-busca = exportar,json,csv,folha de cálculo,planilha
ajustes-secao-banco = Base de dados
ajustes-secao-atualizacoes = Atualizações
ajustes-secao-atualizacoes-dica = Como o Treasury recebe versões novas: transferir sozinho, só avisar ou desligado
ajustes-secao-atualizacoes-busca = atualização,atualizar,versão,nova versão,instalar,transferir,update
ajustes-secao-sobre-dica = Apoiar o Treasury: GitHub Sponsors ou ajudar sem dinheiro
ajustes-secao-sobre-busca = apoiar,donativo,doação,doar,sponsors,sobre,versão

## Palavras repetidas nas secções

ajustes-arquivo = Ficheiro
ajustes-senha-mestra = Palavra-passe mestra
ajustes-copiar-a-mao = Selecione o texto e copie-o à mão.

## Aparência e uso

ajustes-pref-exibicao = Visualização
ajustes-pref-tema-dica = Também no ícone de tema da barra superior e na barra de comando (⌘K / Ctrl+K): escreva “tema”.
ajustes-pref-privacidade = Modo de privacidade
ajustes-pref-privacidade-abrir = Abrir o cofre com o modo de privacidade ligado
ajustes-pref-privacidade-abrir-dica = Ao desbloquear, os valores começam ocultos; mostre-os com o ícone do olho na barra superior.
ajustes-pref-base = Base de datas predefinida
ajustes-pref-base-dica = Competência usa a data da compra; caixa usa o vencimento do extrato para compras no cartão. Lançamentos e relatórios podem trocar de imediato.
ajustes-pref-base-caixa = Predefinição: caixa
ajustes-pref-base-competencia = Predefinição: competência
ajustes-pref-bandeja = Manter as rotinas na barra de menus ao fechar a janela
ajustes-pref-bandeja-dica = As cópias de segurança e a recolha de dados continuam com a janela fechada.
ajustes-pref-downloads-ligado = A acompanhar a pasta Transferências
ajustes-pref-downloads-ligado-dica = Os próximos extratos e notas que descarregar aparecem num aviso para importar.
ajustes-pref-downloads-acompanhar-dica = Quando chegar à pasta Transferências um extrato OFX/CSV, uma nota de corretagem em PDF, um relatório da Área do Investidor B3 ou uma imagem (comprovativo, recibo, fatura, captura de ecrã), o Treasury mostra um aviso para importar. As imagens vão para o agente de IA, que as lê e propõe o registo. Só ficheiros novos, depois de a transferência terminar, e só com o cofre aberto.
ajustes-pref-downloads-apagar = Eliminar de Transferências depois de importar
ajustes-pref-downloads-apagar-dica = Quando a importação de um ficheiro oferecido pelo aviso termina com sucesso, ele vai para o Lixo (pode recuperá-lo de lá). As imagens enviadas ao agente não são eliminadas, porque o registo depende da sua aprovação.
ajustes-pref-downloads-privacidade = Para reconhecer o formato, o Treasury lê o ficheiro novo no seu computador (até 25 MB) sem gravar nada; o conteúdo só entra na importação se clicar em Importar. Nada sai do computador. Na primeira vez, o macOS pode pedir permissão para o Treasury aceder à pasta Transferências.
ajustes-pref-dev = Programador
ajustes-pref-dev-ligado = Opções de programador ligadas
ajustes-pref-dev-ligado-dica = O ecrã 15 · Programador aparece no menu.
ajustes-pref-dev-desligado = Opções de programador desligadas
ajustes-pref-dev-opcoes = Opções de programador
ajustes-pref-dev-opcoes-dica = Feedback do agente: ganha ferramentas para registar e reforçar feedback técnico sobre o Treasury (ferramentas que faltaram, dados confusos, erros, lacunas da documentação), que aparece no ecrã 15 · Programador.

## Fontes de dados

ajustes-brapi-texto = Fonte de dados da B3 com plano gratuito: reserva da cotação quando o Yahoo falha, aviso de tickers renomeados e, nos planos pagos, verificação dos indicadores e dividendos com as fontes oficiais (CVM e B3). Só seguem o código do título e o token, nunca o que tem. Crie um token em { $link }; fica cifrado no cofre e nunca volta ao ecrã.
ajustes-brapi-cadastrado = registado
ajustes-brapi-nao-cadastrado = não registado (a verificação usa só o Fundamentus)
ajustes-brapi-salvo = Token da brapi.dev registado
ajustes-brapi-salvar = Guardar token
ajustes-tiingo-titulo = Tiingo (opcional, cotações dos EUA)
ajustes-tiingo-texto = Cotações diárias, dividendos e desdobramentos de ações, ETF e REIT dos Estados Unidos, com a sua própria chave gratuita, em vez do Yahoo. Crie uma conta em { $link } e cole aqui o token da API. O plano gratuito é para uso pessoal: os dados ficam no seu cofre e não são transmitidos a terceiros.
ajustes-eodhd-titulo = EODHD (opcional, bolsas do mundo)
ajustes-eodhd-texto = Cotações diárias, dividendos e desdobramentos de bolsas de todo o mundo, com a sua própria chave, em vez do Yahoo. O plano gratuito da EODHD é só de teste (poucos pedidos por dia); para acompanhar uma carteira é preciso o plano pessoal pago. Obtenha a chave em { $link }. A EODHD só aceita a chave no endereço do pedido, sempre por ligação cifrada.
ajustes-chave-fonte = Chave
ajustes-chave-fonte-cadastrada = registada
ajustes-chave-fonte-nao-cadastrada = não registada
ajustes-chave-fonte-trocar = Trocar a chave
ajustes-chave-fonte-campo = Chave da { $fonte }
ajustes-chave-fonte-salva = Chave da { $fonte } registada
ajustes-chave-fonte-removida = Chave da { $fonte } removida
ajustes-chave-fonte-salvar = Guardar chave
ajustes-fontes-indices-texto = O Treasury obtém IPCA, Selic, CDI e PTAX no Banco Central, cotações no Yahoo Finance e no Tesouro Transparente e, conforme o país de referência, a inflação, os juros e o câmbio de outros países (BCE, Eurostat, OCDE, Bank of Canada, ONS, Bank of England e Banco Mundial). Os dados ficam no cofre; nada da sua carteira sai do computador.
ajustes-fontes-atualizando = A atualizar…
ajustes-fred-texto = CPI, M2 e Fed Funds vêm da FRED (Federal Reserve Bank of St. Louis), que exige uma chave gratuita. Crie uma conta em { $link } (menu “My Account › API Keys”) e cole aqui a chave de 32 letras e números. Fica cifrada no cofre e nunca volta ao ecrã.
ajustes-fred-cadastrada = registada
ajustes-fred-nao-cadastrada = não registada
ajustes-fred-salva = Chave da FRED registada
ajustes-fred-salvar = Guardar chave

## Fontes de dados: país de referência e fornecedores

ajustes-pais-zona-euro = Zona euro
ajustes-pais-ref-texto = O país de referência escolhe a inflação do deflator “Inflação do país (…)” e que índices, juros e câmbio a recolha obtém, além das séries de sempre. No Brasil nada muda: IPCA, CDI e PTAX do Banco Central.
ajustes-pais-ref-alterado-dica = { $pais }. Atualize os índices para descarregar as séries novas.
ajustes-pais-ref-nota = { $moeda ->
        [BRL] A moeda base da aplicação continua a ser o real: a carteira e os relatórios convertem para BRL pela PTAX.
       *[other] A moeda base da família continua a ser { $moeda }: a carteira e os relatórios convertem para { $moeda }.
    } O câmbio do país escolhido fica disponível como série (09 · Ativos › Índices e cotações) e para conversões entre outras moedas.
ajustes-provedor-nunca = Ainda não recolhido
ajustes-provedor-col = Fornecedor
ajustes-provedor-col-coletado = Recolhido em
ajustes-provedor-col-ate = Dados até
ajustes-provedores-titulo = Fornecedores de dados
ajustes-provedores-texto = Fontes oficiais e gratuitas, sem chave (exceto a FRED). Cada valor guarda a fonte e a data da recolha; uma fonte em baixo não impede as outras.
ajustes-provedores-vazio = Nenhum fornecedor.

## Fontes de dados: fontes externas

ajustes-externas-texto = Cada fonte abaixo é consultada pela rede só para obter dados públicos (cotações, índices, indicadores); nada da sua carteira é enviado. Desligue as que não quer usar: o que já foi descarregado fica no cofre.
ajustes-externas-buscar = Pesquisar fonte
ajustes-externas-buscar-placeholder = Pesquisar por nome, dado ou país…
ajustes-externas-desligada-app = Desligada pela aplicação depois de { $dias } dias seguidos de falha.
ajustes-externas-falhando =
    { $dias ->
        [one] A falhar há { $dias } dia; desliga-se sozinha aos 3.
       *[other] A falhar há { $dias } dias; desliga-se sozinha aos 3.
    }

## Exportar dados

ajustes-exportar-aviso = O ficheiro exportado sai **sem criptografia**: qualquer pessoa com acesso a ele vê os seus dados. Guarde-o num local seguro e elimine-o quando já não precisar dele. As pastas do Treasury não são aceites como destino.
ajustes-exportar-json-dica = Caminho completo do ficheiro (pasta existente, fora das pastas do Treasury).
ajustes-exportar-csv = Lançamentos, em CSV (folha de cálculo)

## Integrações MCP

ajustes-mcp-desktop-texto = Em Definições › Programador › Editar configuração, acrescente este excerto ao { $arquivo } e reinicie o Claude.
ajustes-mcp-code-texto = Com a aplicação aberta, o MCP comunica com ela (incluindo escritas, que entram no histórico como “IA”). Com a aplicação fechada, lê em modo só de leitura se o desbloqueio pelo keychain estiver ligado (Ajustes › Segurança).
ajustes-mcp-opencode-texto = Acrescente este excerto ao { $arquivo } (o global fica em ~/.config/opencode; também serve um opencode.json na pasta do projeto). Funciona no OpenCode 1.x e 2.x.
ajustes-mcp-opencode-config = Configuração MCP do OpenCode
ajustes-mcp-copiar-opencode = Copiar excerto do OpenCode
ajustes-mcp-chave-agora-titulo = A chave de API passou a ser obrigatória
ajustes-mcp-chave-agora-texto = Para proteger o cofre de outros programas do computador, os clientes MCP externos (assistentes de IA e agentes de código) passaram a precisar de uma chave. Crie uma chave abaixo e coloque-a no cliente; o agente da própria aplicação continua a funcionar sem ela.
ajustes-mcp-chave-agora-entendi = Entendi
ajustes-mcp-chave-sempre = A chave de API é sempre exigida: cada cliente MCP externo (assistente de IA ou agente de código) liga-se com a própria chave e só vê o que os âmbitos dela permitem.
ajustes-mcp-revogada-em = revogada em { $data }
ajustes-mcp-sai-da-lista = sai da lista em { $data }
ajustes-mcp-excluir = Eliminar
ajustes-mcp-chave-excluida = Chave eliminada
ajustes-mcp-cliente-novo-titulo = Ligou-se um cliente MCP novo
ajustes-mcp-cliente-novo-item = { $cliente } · { $programa } · PID { $pid } · { $quando }
ajustes-mcp-cliente-programa-desconhecido = programa desconhecido
ajustes-mcp-cliente-novo-texto = Se não foi você que configurou este cliente, revogue a chave de API dele abaixo e troque as outras.
ajustes-mcp-cliente-novo-toast = Ligou-se ao Treasury um cliente MCP novo. Confira em Ajustes › Integrações MCP.
ajustes-mcp-app-fechado = Responder com a aplicação fechada
ajustes-mcp-app-fechado-dica = Com a aplicação fechada, o treasury mcp abre o cofre em modo de leitura pelo desbloqueio do porta-chaves, e só para clientes com uma chave de API válida (TREASURY_API_KEY). Desligado, só pede para abrir o Treasury: nenhum outro programa lê o cofre sem si.
ajustes-mcp-salvar-porta = Guardar porta
ajustes-mcp-nao-subiu = Não arrancou: { $erro }
ajustes-mcp-iniciando = a iniciar…
ajustes-mcp-so-no-app = só na aplicação
ajustes-mcp-copie-agora = Copie agora: a chave não voltará a ser mostrada. Âmbitos: { $escopos }.
ajustes-mcp-escopos = Âmbitos
ajustes-mcp-sem-chaves = Nenhuma chave. Crie uma para cada agente externo, só com os âmbitos de que precisa.
ajustes-mcp-nova-chave-dica = Nenhum âmbito aprova propostas nem mexe em ajustes, chaves, cofre ou sessões.
ajustes-mcp-escrita-opcao = Escrita: cria propostas para aprovar

## Cópias de segurança

ajustes-backup-restaurado = Cópia de segurança restaurada
ajustes-backup-restaurado-dica = Dados de { $data }. O estado anterior passou a ser uma cópia de segurança.
ajustes-backup-restaurar-titulo = Restaurar cópia de segurança
ajustes-backup-restaurar-texto = Os dados voltam a { $data } ({ $tipo }). Antes, o Treasury faz uma cópia de segurança do estado atual, e a base de dados restaurada é recifrada com a sua palavra-passe mestra atual.
ajustes-backup-restaurando = A restaurar…
ajustes-backup-senha-atual = Palavra-passe mestra atual
ajustes-backup-senha = Palavra-passe da cópia de segurança
ajustes-backup-senha-dica = A palavra-passe usada quando esta cópia foi feita (palavra-passe mestra antiga ou palavra-passe própria das cópias).
ajustes-backup-outra-senha = Esta cópia usa outra palavra-passe
ajustes-backup-verificar-titulo = Verificar com outra palavra-passe
ajustes-backup-verificar-texto = Esta cópia não abriu com a chave atual. Indique a palavra-passe usada quando foi feita.
ajustes-backup-integro = Cópia íntegra
ajustes-backup-problema = Cópia com problema
ajustes-backup-hora = Cópia diária às
ajustes-backup-diarios = Diárias guardadas
ajustes-backup-ao-fechar = Cópia de segurança ao fechar a aplicação
ajustes-backup-ao-fechar-dica = Só quando houve alterações desde a última cópia.
ajustes-backup-destinos = Destinos extra (sempre cifrados)
ajustes-backup-chave = Chave das cópias de segurança
ajustes-backup-chave-mestra = A palavra-passe mestra
ajustes-backup-chave-propria = Palavra-passe própria das cópias
ajustes-backup-senha-propria = Palavra-passe das cópias (mín. 8)
ajustes-backup-senha-propria-dica = Defina-a antes de guardar e não a perca: sem ela, as cópias novas não abrem.
ajustes-backup-senha-definida = Palavra-passe das cópias definida
ajustes-backup-definir-senha = Definir palavra-passe
ajustes-backup-ajustes-salvos = Ajustes das cópias de segurança guardados
ajustes-backup-salvar-ajustes = Guardar ajustes das cópias de segurança
ajustes-backup-pasta-app = pasta da aplicação
ajustes-backup-sumiu = o ficheiro desapareceu
ajustes-backup-verificado = ✓ verificada
ajustes-backup-verificado-propria = ✓ verificada · palavra-passe própria
ajustes-backup-nao-verificado = não verificada
ajustes-backup-saude = Saúde das cópias de segurança
ajustes-backup-fazendo = A fazer a cópia de segurança
ajustes-backup-feito = Cópia de segurança feita
ajustes-backup-feito-dica = { $tamanho } · cifrada
ajustes-backup-fazer = Fazer cópia de segurança agora
ajustes-backup-atrasado = A última cópia válida tem { $horas } horas (mais de 48 h). Faça uma cópia de segurança agora.
ajustes-backup-nenhum = Não há nenhuma cópia de segurança válida. Faça uma agora.
ajustes-backup-ultimo = Última cópia válida
ajustes-backup-no-disco = Cópias no disco
ajustes-backup-pendente = Cópia diária pendente: sai na próxima verificação
ajustes-backup-ajustes = Ajustes das cópias de segurança

## Categorias

ajustes-cat-uso-parcelamentos =
    { $n ->
        [one] { $n } plano de prestações
       *[other] { $n } planos de prestações
    }
ajustes-cat-sem-uso = Sem uso: pode ser eliminada sem reatribuir.
ajustes-cat-juntar-titulo = Juntar { $nome } noutra
ajustes-cat-apagar-nome = Eliminar { $nome }
ajustes-cat-apagada = Categoria eliminada
ajustes-cat-juntar-texto = Tudo o que usa esta categoria (lançamentos, divisões, orçamento, regras, recorrências) passa para a escolhida, incluindo as subcategorias. Pode desfazer no histórico.
ajustes-cat-apagar-texto = Se estiver em uso, escolha quem recebe os lançamentos. Para apenas a ocultar dos seletores, arquive-a.
ajustes-cat-sugeridas-despesa =
    { $n ->
        [one] { $n } categoria sugerida do conjunto predefinido (pagamentos de contas, empréstimos e cartão; escola, saúde, seguros, poupança dos filhos…). As que já tem não mudam; em “Ver lista”, escolha quais entram.
       *[other] { $n } categorias sugeridas do conjunto predefinido (pagamentos de contas, empréstimos e cartão; escola, saúde, seguros, poupança dos filhos…). As que já tem não mudam; em “Ver lista”, escolha quais entram.
    }
ajustes-cat-sugeridas-receita =
    { $n ->
        [one] { $n } categoria sugerida do conjunto predefinido, { $receitas } de receita (resgates, venda de bens, dívidas recebidas, trabalho em plataformas, vendas e biscates, reforma e apoios do Estado…). As que já tem não mudam; em “Ver lista”, escolha quais entram.
       *[other] { $n } categorias sugeridas do conjunto predefinido, { $receitas } de receita (resgates, venda de bens, dívidas recebidas, trabalho em plataformas, vendas e biscates, reforma e apoios do Estado…). As que já tem não mudam; em “Ver lista”, escolha quais entram.
    }
ajustes-cat-esconder-lista = Ocultar lista

## Base de dados

ajustes-banco-integridade-dica = Verifica páginas, índices e referências entre tabelas.
ajustes-banco-estatisticas-dica = ANALYZE: ajuda a base de dados a escolher os índices certos.
ajustes-banco-compactar-dica = VACUUM: devolve ao disco o espaço livre dentro do ficheiro.
ajustes-banco-previa = Pré-visualizar
ajustes-banco-nada = Nada para eliminar.
ajustes-banco-previa-texto =
    { $n ->
        [one] { $n } registo, cerca de { $tamanho }. { $consequencia }
       *[other] { $n } registos, cerca de { $tamanho }. { $consequencia }
    }
ajustes-banco-apagar-titulo = Eliminar: { $item }
ajustes-banco-apagar-texto =
    { $n ->
        [one] { $n } registo. { $consequencia } Antes, o Treasury faz uma cópia de segurança verificada do cofre.
       *[other] { $n } registos. { $consequencia } Antes, o Treasury faz uma cópia de segurança verificada do cofre.
    }
ajustes-banco-apagar-confirmar = Fazer cópia de segurança e eliminar
ajustes-banco-apagados =
    { $n ->
        [one] { $n } registo eliminado
       *[other] { $n } registos eliminados
    }
ajustes-banco-apagados-dica = Cópia de segurança feita antes. Ficheiro: { $antes } → { $depois }.
ajustes-banco-rodar = Executar
ajustes-banco-maiores-rotulo = Maiores tabelas da base de dados
ajustes-banco-clique-duplo = Faça duplo clique numa tabela (ou Enter) para ver as 100 primeiras linhas.
ajustes-banco-expurgo-texto = Só o que não são dados da família: lançamentos, contas, carteira, contratos e registos nunca entram aqui, nem propostas do agente que aguardam aprovação. Cada expurgo mostra a pré-visualização, pede confirmação e faz uma cópia de segurança verificada antes de eliminar.
ajustes-tabela-lendo = A ler a tabela
ajustes-tabela-resumo = { $linhas } linha(s) no total · { $colunas } coluna(s) · ordem: { $ordem } · só de leitura

## Família: nome, país, membros e instituições

ajustes-familia-nome-dica = Aparece na barra lateral e no ecrã de desbloqueio.
ajustes-pais-onde-mora = País onde a família vive
ajustes-pais-mudar-fontes = As fontes de dados de { $pais } passam a ser recolhidas e as que não fazem sentido lá ficam desligadas. As que desligou à mão continuam como estão.
ajustes-pais-mudar-mantem = As categorias, regras e lançamentos que já tem não mudam.
ajustes-pais-trocar-moeda-dica = O cofre já tem contas e lançamentos: ficam na moeda deles e os totais passam a ser convertidos para { $moeda }. O custo dos investimentos já registados continua na moeda da época.
ajustes-membros-esconder = Ocultar arquivados
ajustes-membros-familia-tela = O rendimento, os recibos de vencimento e as contas e cartões de cada membro ficam no ecrã { $link }.
ajustes-inst-apagada = Instituição eliminada
ajustes-inst-nao-apagou = Não foi possível eliminar
ajustes-inst-nenhuma = Nenhuma instituição registada.

## Família: método de custo das vendas

ajustes-metodo-custo-dica = Como o livro de vendas calcula o custo de ações, fundos e ETF. Por omissão segue a regra fiscal do país da família; o resultado pelo preço médio continua ao lado, para o IR do Brasil.
ajustes-metodo-custo-fifo = Primeiro a entrar, primeiro a sair (FIFO)
ajustes-metodo-custo-salvo = Método de custo guardado
ajustes-metodo-custo-renda-fixa = O rendimento fixo e o Tesouro continuam pelos lotes (primeiro a entrar, primeiro a sair). O custo da posição e o resultado não realizado da carteira seguem pelo preço médio.

## Histórico de alterações

ajustes-hist-apagou = eliminou { $itens }
ajustes-hist-conflito-desfazer = Não pode ser desfeita isoladamente.
ajustes-hist-conflito-refazer = Não pode ser refeita isoladamente.
ajustes-hist-conflito-dica = Desfaça primeiro, na lista ao lado, as alterações mais recentes que mexem nas mesmas linhas.
ajustes-hist-escolha = Escolha uma alteração para ver o antes e o depois de cada linha. Importações, regras e ações da IA também ficam aqui, e tudo pode ser desfeito.

## Notificações

ajustes-notif-contas = Contas e extratos de cartão a vencer
ajustes-notif-contas-dica = Lançamentos previstos e extratos de cartão perto do vencimento.
ajustes-notif-saldo = Saldo a ficar negativo
ajustes-notif-orcamento = Orçamento ultrapassado
ajustes-notif-assinaturas = Subscrições
ajustes-notif-proventos = Datas de dividendos
ajustes-notif-proventos-dica = Ativos da carteira com o último dia com direito a dividendos nos próximos 3 dias.
ajustes-notif-watchlist-dica = Cotação acima ou abaixo do alerta que definiu.
ajustes-notif-agente-dica = Fecho do mês, carteira fora da banda.
ajustes-notif-so-no-app = Notificações só na aplicação instalada
ajustes-notif-teste-titulo = Extrato do cartão Exemplo vence amanhã
ajustes-notif-teste-valores = { $valor } na Conta à ordem, vencimento amanhã.
ajustes-notif-sistema-dica = Com a aplicação aberta e o cofre desbloqueado. Desligado, os lembretes continuam a ir para a caixa de alertas do Agente.
ajustes-notif-valores-dica = A notificação aparece no ecrã bloqueado e no centro de notificações. Desligado, só o título (sem valores).
ajustes-notif-conferir = Verificar lembretes agora

## Perfil

ajustes-perfil-tom-simples = Muito simples
ajustes-perfil-origem-pulou = ao saltar o questionário
ajustes-perfil-origem-padrao = predefinição de quem já usava a aplicação
ajustes-perfil-carregando = A carregar o perfil
ajustes-perfil-texto = O nível decide o que aparece por predefinição: no Iniciante, o painel é simples e os ecrãs avançados ficam em “Mais ecrãs”; no Avançado, tudo fica à vista. Nada é eliminado ao mudar. Cada membro da família pode ter o próprio nível e linguagem: escolha o membro em “Ver como”, no topo.
ajustes-perfil-linguagem-dica = É também a forma como o agente fala consigo.
ajustes-perfil-explicacoes-dica = Ícones { $icone } ao lado de termos como CDI, XIRR e competência; “Pelo nível” mostra-os no Iniciante e no Intermédio.
ajustes-perfil-respostas = As suas respostas

## Segurança

ajustes-seg-trocada = Palavra-passe mestra alterada
ajustes-seg-trocada-dica = Foi feita uma cópia de segurança antes da alteração. Se tinha um kit de recuperação, gere um novo: o antigo já não abre o cofre.
ajustes-seg-trocar-form = Alterar a palavra-passe mestra
ajustes-seg-senha-atual = Palavra-passe atual
ajustes-seg-informe-atual = Indique a palavra-passe atual.
ajustes-seg-nova = Nova palavra-passe
ajustes-seg-confirmar = Confirmar nova palavra-passe
ajustes-seg-recifrando = A recifrar o cofre…
ajustes-seg-trocar = Alterar palavra-passe mestra
ajustes-seg-kit-texto = O kit de recuperação abre o cofre se a palavra-passe mestra for esquecida. Quem tiver o código tem acesso a tudo: **imprima-o e guarde-o em papel, longe do computador**. Não o fotografe nem o guarde na nuvem.
ajustes-seg-kit-confirme = Confirme a palavra-passe mestra
ajustes-seg-kit-fechar = Fechar (o código desaparece do ecrã)
ajustes-seg-kit-alerta = Este código abre o cofre sem a palavra-passe. Imprima-o, guarde-o num local seguro e feche esta janela.
ajustes-seg-kit-como-valor = No ecrã de desbloqueio, “Esqueceu-se da palavra-passe?”, e introduza o código.
ajustes-seg-touch-id-dica = A chave do cofre fica no keychain e só sai depois do Touch ID (ou da palavra-passe do Mac). Também permite ao `treasury mcp` ler com a aplicação fechada.
ajustes-seg-keychain-ligar-dica = Confirme a palavra-passe mestra para guardar a chave do cofre no keychain.
ajustes-seg-bloqueio-dica = Depois deste tempo sem uso, o cofre fecha e a chave sai da memória.
ajustes-pin-desbloquear-dica = PIN de 6 a 12 dígitos, guardado só no keychain deste computador. Cinco erros desativam o PIN e pedem a palavra-passe mestra.
ajustes-pin-ligar-dica = Escolha o PIN e confirme a palavra-passe mestra.
ajustes-partes-texto = Guarde cada parte num local diferente (por exemplo: papel em casa, com o cônjuge, no cofre do banco). Quaisquer duas recuperam o cofre; uma sozinha não revela nada. Não voltarão a ser mostradas.
ajustes-partes-fido2-removidas =
    { $n ->
        [one] { $n } chave física deixou de valer: registe-a de novo.
       *[other] { $n } chaves físicas deixaram de valer: registe-as de novo.
    }
ajustes-partes-geradas = Partes geradas em { $data }. Continuam válidas depois de alterar a palavra-passe.
ajustes-partes-dica = Divide a recuperação em 3 partes: quaisquer duas abrem o cofre se se esquecer da palavra-passe.
ajustes-partes-trocar-dica = As partes antigas e as chaves físicas registadas deixam de valer.
ajustes-partes-confirme = Confirme a palavra-passe mestra para gerar as partes.
ajustes-fido2-cadastrada = Chave física registada
ajustes-fido2-cadastrada-em = registada { $data }
ajustes-fido2-texto = Uma YubiKey ou outra chave FIDO2 recupera o cofre se se esquecer da palavra-passe.
ajustes-fido2-so-no-app = Só na aplicação Treasury
ajustes-fido2-cadastrar = Registar chave física
ajustes-fido2-conecte = Ligue a chave física (USB ou NFC no leitor) e tente novamente.
ajustes-fido2-cadastrar-botao = Registar
ajustes-fido2-pin = PIN da chave física (se tiver)

## Cofres

ajustes-cofres-texto = Cada cofre é um ficheiro .db cifrado, com a própria palavra-passe, desbloqueio rápido (Touch ID, PIN) e cópias de segurança numa pasta própria. Use-os para separar finalidades, como a empresa ou as finanças dos pais. Trocar de cofre fecha o atual (com a cópia de segurança de fecho) e pede a palavra-passe do outro. Tirar da lista não elimina o ficheiro.

ajustes-seg-dormir = Bloquear ao suspender ou bloquear o ecrã
ajustes-seg-dormir-dica = O cofre fecha quando o computador entra em suspensão, quando o ecrã é bloqueado (pelo atalho, pela proteção de ecrã ou pelo canto ativo) e quando outra pessoa entra na sessão. No macOS; nos outros sistemas vale o bloqueio por inatividade.
ajustes-seg-capturas-titulo = Capturas de ecrã
ajustes-seg-capturas = Permitir capturas de ecrã do Treasury
ajustes-seg-capturas-dica = Desligado (predefinição): com o modo de privacidade ligado, a janela desaparece de capturas, gravações e da partilha de ecrã. No macOS e no Windows.
ajustes-seg-capturas-dica-liberadas = Ligado: a janela aparece em capturas, gravações e na partilha de ecrã, mesmo com o modo de privacidade ligado. Os valores continuam ocultos enquanto o modo de privacidade estiver ligado.
ajustes-seg-capturas-confirmar-titulo = Permitir capturas de ecrã?
ajustes-seg-capturas-confirmar-texto = Qualquer programa que capture o ecrã (gravadores, apps de reunião, um programa instalado sem dar por isso) passa a ver a janela do Treasury, mesmo com o modo de privacidade ligado. Desligue de novo quando terminar.
ajustes-seg-capturas-confirmar = Permitir capturas
ajustes-seg-capturas-liberadas = Capturas de ecrã permitidas
ajustes-seg-capturas-bloqueadas = Capturas de ecrã bloqueadas de novo
ajustes-atualizacao-titulo = Atualizações
ajustes-atualizacao-modo = O que fazer com uma versão nova
ajustes-atualizacao-modo-baixar = Transferir sozinho e avisar
ajustes-atualizacao-modo-baixar-dica = Procura ao abrir e a cada 6 horas, transfere e verifica a assinatura, e avisa que está pronta. Instalar espera sempre a sua confirmação.
ajustes-atualizacao-modo-avisar = Só avisar
ajustes-atualizacao-modo-avisar-dica = Procura ao mesmo ritmo e só avisa que há versão nova. Nada é transferido até pedir, bom para ligações limitadas.
ajustes-atualizacao-modo-desligado = Desligado
ajustes-atualizacao-modo-desligado-dica = Não procura sozinho. Versões novas, incluindo correções de segurança, só chegam quando procurar.
ajustes-atualizacao-modo-salvo = Atualizações: { $modo }
ajustes-atualizacao-disponivel = Versão { $versao } disponível
ajustes-atualizacao-baixar = Transferir e verificar
ajustes-atualizacao-privacidade = Só pergunta às releases do Treasury no GitHub qual é a versão mais recente: nada sobre si ou o cofre vai junto. Cada versão transferida é verificada pela assinatura do Treasury antes de poder ser instalada.
ajustes-atualizacao-versao = Versão instalada: { $versao }
ajustes-atualizacao-ultima = última procura: { $quando }
ajustes-atualizacao-indisponivel = A atualização automática começa quando o Treasury for publicado no GitHub. Até lá, as versões novas são instaladas à mão.
ajustes-atualizacao-erro = A última procura falhou: { $erro }
ajustes-atualizacao-pronta = Versão { $versao } pronta para instalar
ajustes-atualizacao-instalar = Instalar e reiniciar
ajustes-atualizacao-procurar = Procurar agora
ajustes-atualizacao-em-dia = Nenhuma versão nova encontrada
ajustes-atualizacao-confirmar-titulo = Instalar a versão { $versao }?
ajustes-atualizacao-confirmar-texto = O Treasury fecha, instala a versão nova e abre de novo. O cofre é guardado antes e vai pedir o desbloqueio.
ajustes-atualizacao-ver = Ver
ajustes-pref-caixa-entrada-dica = Ficheiros colocados em { $pasta } (por si ou por um agente de IA) entram na caixa de entrada em Movimentos: o Treasury lê, guarda no cofre e tira da pasta. Nada sai do computador.
