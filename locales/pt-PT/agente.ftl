# Variante pt-PT (ecrã Agente e peças do agente usadas no painel lateral): o que faltar cai
# para pt-BR. Revisão: rascunho por IA.

## Ecrã e separadores

agente-secoes = Secções do agente

## Números, tempos e aspas

agente-tokens-mi = { NUMBER($n, maximumFractionDigits: 1) } M

## Tarefas, tipos de motor e estados

agente-tarefa-fechamento = Fecho mensal
agente-destino-api = dados enviados à API do fornecedor
agente-destino-cli = os dados passam pela CLI local (e pelo fornecedor dela)
agente-job-andamento = Em curso
agente-problema-sem-chave = sem API key registada

## Comandos de escrita propostos

agente-comando-apagar-lancamento = Eliminar lançamento
agente-comando-apagar-feedback = Eliminar feedback ao programador
agente-comando-descartar-feedback = Descartar feedback ao programador
agente-comando-cadastrar-instituicao = Registar instituição
agente-comando-cadastrar-conta = Registar conta ou cartão
agente-comando-cadastrar-bem = Registar bem físico
agente-comando-avaliacao-bem = Registar avaliação do bem
agente-comando-benfeitoria = Registar benfeitoria
agente-comando-cadastrar-ativo = Registar ativo
agente-comando-registrar-operacao = Registar operação
agente-comando-registrar-provento = Registar dividendo
agente-comando-proventos-ticker = Registar dividendos pelo ticker
agente-comando-evento-corporativo = Registar evento corporativo
agente-comando-investimentos-lote = Registar investimentos em lote

## Assuntos dos procedimentos

agente-assunto-fatura = Extrato do cartão
agente-assunto-proventos = Dividendos
agente-assunto-imposto = IRS
agente-assunto-cadastro = Registos

## Tabelas da base de dados nas alterações

agente-tabela-provento = Dividendo
agente-tabela-renda-fixa = Rendimento fixo
agente-tabela-financiamento-bem = Crédito do bem

## Contagem por tabela (singular só para 1)

agente-n-provento = { $n ->
        [1] { $n } dividendo
       *[other] { $n } dividendos
    }
agente-n-renda-fixa = { $n ->
        [1] { $n } rendimento fixo
       *[other] { $n } rendimentos fixos
    }
agente-n-financiamento-bem = { $n ->
        [1] { $n } crédito do bem
       *[other] { $n } créditos do bem
    }

## Resumo e frases do que muda

agente-resumo-apaga = elimina { $itens }
agente-frase-apaga = Elimina { $itens }: { $nomes }
agente-frase-registra = Regista { $itens }: { $nomes }
agente-artigo-favorecido = o beneficiário

## Leitura amigável das alterações: operações, títulos e valores

agente-op-exclusao = Eliminação
agente-regra-favorecido = beneficiário “{ $nome }”
agente-provento = Dividendo

## Campos das alterações

agente-campo-favorecido = Beneficiário
agente-campo-fatura = Extrato do cartão
agente-campo-parcelamento = Plano de prestações
agente-campo-parcela = Prestação
agente-campo-parcelas = Prestações
agente-campo-patrimonio = Entra no património
agente-campo-dia-fechamento = Dia do fecho
agente-campo-total-informado = Total indicado
agente-campo-primeira-parcela = Primeira prestação
agente-campo-aporte-mensal = Reforço mensal
agente-campo-valor-financiado = Montante financiado
agente-campo-agencia = Balcão

## Revisão das alterações e diff exato

agente-nenhuma-linha-alteracao = Nenhuma linha da base de dados muda com esta alteração.
agente-nenhuma-linha-proposta = Nenhuma linha da base de dados muda com esta proposta.
agente-diff-exato = Diff exato do que será gravado: tabelas, ids e valores brutos (dinheiro em cêntimos).

## Conversa

agente-respondendo = A responder
agente-recebendo = A receber
agente-revisar-fila = Rever na fila
agente-fallback = { $motor } não respondeu ({ $motivo }); a tentar o próximo motor da rota.
agente-voce = Utilizador
agente-sem-motor-dica = Registe um motor (API, CLI local ou modelo local) e defina a rota de chat em { $link }.
agente-aguardando-motor = a aguardar o motor
agente-vazio-texto = O agente usa as ferramentas do Treasury; toda a escrita aguarda a sua aprovação na fila. As recomendações de investimento são análises dos seus dados, não aconselhamento profissional.
agente-vazio-texto-autonomo = O agente usa as ferramentas do Treasury. Com o modo autónomo ligado, as escritas reversíveis aplicam-se logo e podem ser desfeitas no histórico; as sensíveis aguardam a sua aprovação. As recomendações de investimento são análises dos seus dados, não aconselhamento profissional.
agente-enter-envia = Enter envia · Shift+Enter muda de linha

## Atividade da resposta (raciocínio e ferramentas)

agente-pensando = A pensar…
agente-consultando = A consultar { $ferramenta }…

## Conversas gravadas

agente-fixada-dica = Fixada: não entra na limpeza automática
agente-fixar-dica = Fixar: a limpeza automática não a elimina
agente-apagar-conversa = Eliminar a conversa { $titulo }
agente-nao-apagou = Não foi possível eliminar
agente-apagar-todas = Eliminar todas as conversas
agente-apagar-todas-texto = O histórico das conversas desaparece da base de dados. As propostas, os alertas e a memória do gestor continuam.
agente-apagar-historico = Eliminar histórico
agente-apagar-tudo = Eliminar tudo

## Computador

agente-computador-ligado-dica = Nesta conversa, as CLIs locais de agentes podem ler ficheiros da sua pasta pessoal quando o pedir. O terminal, a escrita e a web continuam bloqueados; os motores de API e locais continuam só com o Treasury.
agente-computador-desligado-dica = O agente usa só as ferramentas do Treasury e não olha para os seus ficheiros, aplicações ou navegador. Ligue para deixar uma CLI ler um ficheiro que indicar nesta conversa.

## Esforço de raciocínio

agente-esforco-padrao = Predefinido
agente-esforco-sem-controle = { $motor } não tem controlo de esforço com este modelo; aplica-se a predefinição dele.

## Anexos

agente-anexos-a-mais =
    { $n ->
        [one] { $n } ficheiro a mais
       *[other] { $n } ficheiros a mais
    }
agente-anexo-grande = ficheiro demasiado grande (máx. 20 MB)
agente-anexar-dica = Anexar imagens, PDFs, folhas de cálculo ou textos. Também pode arrastar ficheiros para aqui ou colar com ⌘V / Ctrl+V.
agente-anexar-arquivos = Anexar ficheiros
agente-anexo-preparando = a preparar…
agente-anexo-motor-nao-le = { $motor } não lê { $tipo ->
        [imagem] imagens
       *[pdf] PDFs digitalizados
    }. Se enviar assim, o agente recebe só o nome do ficheiro. Escolha um motor com visão (de API ou um modelo local com visão) ou remova o anexo.
agente-anexo-rota-nao-le = Nenhum motor da rota de chat lê { $tipo ->
        [imagem] imagens
       *[pdf] PDFs digitalizados
    } desse tipo. Se enviar assim, o agente recebe só o nome do ficheiro. Escolha um motor com visão (de API ou um modelo local com visão) ou remova o anexo.
agente-anexo-escaneado = sem texto (digitalizado)
agente-solte = Largue para anexar

## Fila de aprovação

agente-retiradas-texto = Os mesmos dados mudaram antes da aprovação; a alteração que fez mantém-se.
agente-aprovadas-texto = As alterações aplicam-se agora e podem ser desfeitas.
agente-carregando-diff = A carregar o diff
agente-revisadas = Revistas recentemente
agente-autonomo = Modo autónomo
agente-autonomo-ligado = Modo autónomo ligado: as escritas reversíveis da IA são aplicadas sem perguntar
agente-autonomo-desligado = Modo autónomo desligado: toda a escrita da IA aguarda a sua aprovação
agente-sensiveis-texto = As escritas sensíveis (operações de investimento, importações, eliminações) pedem sempre aprovação item a item. Tudo o que for aprovado pode ser desfeito.
agente-fila-vazia = Nenhuma proposta à espera de aprovação.

## Alertas e rotinas

agente-apagar-alerta = Eliminar o alerta { $titulo }
agente-rotina-andamento = { $rotina }: em curso
agente-nao-rodou = Não foi possível executar
agente-rodando = A executar…
agente-rodar-agora = Executar agora

## Memória do gestor

agente-gravada-voce = gravada por si
agente-esquecer-texto = O agente deixa de a considerar em todas as conversas e rotinas. Pode desfazer no histórico de alterações.
agente-memoria-rodape = Aplica-se a todos os motores. O agente só grava notas novas com a sua aprovação.

## Procedimentos aprendidos

agente-proc-ocultados =
    { $n ->
        [one] { $n } trecho sensível ocultado (palavras-passe, números de conta ou documentos).
       *[other] { $n } trechos sensíveis ocultados (palavras-passe, números de conta ou documentos).
    }
agente-proc-dialogo-texto = Notas que o agente consulta antes de voltar a procurar. Não escreva palavras-passe, códigos nem números de conta ou de documento.
agente-site-dica = Só o domínio, ex.: www.banco.pt
agente-passos-dica = Menus, filtros, botão de exportar e o que o ficheiro traz.
agente-proc-por-voce = Gravado por si em { $data }
agente-proc-funcionou-dica = Vezes em que o agente seguiu a nota e correu bem
agente-proc-arquivo = ficheiro { $formato }
agente-proc-apagar-titulo = Eliminar este procedimento
agente-proc-apagar-texto = O agente deixa de usar esta nota. Pode desfazer no histórico de alterações.
agente-proc-apagado = Procedimento eliminado
agente-procs-rodape = O agente grava e atualiza estas notas sozinho, sem proposta: tudo fica no histórico de alterações e pode ser desfeito. Vão com os pedidos relacionados como dados, nunca como instruções.
agente-buscar = Pesquisar
agente-procs-placeholder = Ex.: extrato do banco, b3, cartão
agente-procs-vazio = O agente ainda não gravou procedimentos. Quando descobre onde descarregar um extrato ou de onde vem um dado, a nota aparece aqui.

## Histórico de ferramentas

agente-nenhuma-chamada = Nenhuma chamada registada.

## Ajustes de IA

agente-nao-salvou = Não foi possível guardar
agente-autonomo-dica = Aplica sem perguntar as escritas reversíveis da IA (categorizar, criar regra, orçamento). As escritas sensíveis continuam a pedir aprovação item a item; tudo pode ser desfeito.
agente-guardar-historico-dica = Sem histórico, cada pergunta começa do zero e nada da conversa fica na base de dados.
agente-rotinas-legenda = Rotinas (com a aplicação aberta e desbloqueada)
agente-revisao-continua-dica = Ao importar extratos: categoriza, aponta duplicados e sugere regras (como propostas).
agente-alertas-dica = Orçamento a ultrapassar, vencimentos, dívida cara, carteira desequilibrada.

## Roteamento por tarefa

agente-rota-chat-nota = Aplica-se também à categorização e ao PDF quando estes não têm rota própria.
agente-rota-categorizacao-nota = Sem rota própria, a revisão contínua corre localmente e de forma determinística.
agente-nao-salvou-rota = Não foi possível guardar a rota
agente-tirar-rota = Remover { $motor } da rota

## Motores de IA

agente-chave-cadastrada = registada
agente-chave-cadastrada-dica = registada · { $dica }
agente-chave-nao-cadastrada = não registada
agente-excluir = Eliminar
agente-excluir-motor = Eliminar { $nome }
agente-excluir-motor-texto = O motor sai das rotas de tarefa e a chave registada é eliminada. O histórico de utilização continua.
agente-nao-excluiu = Não foi possível eliminar
agente-motores-rodape = As APIs cobram por token; as CLIs locais usam a subscrição que já paga; os modelos locais não enviam nada para fora.
agente-motores-vazio = Nenhum motor registado. Adicione uma API, uma CLI local ou um modelo local.
agente-procurando-clis = A procurar CLIs no PATH
agente-clis-detectadas = CLIs detetadas
agente-cli-login-incerto = · login: não é possível saber sem executar
agente-cli-historico-fora = Esta CLI guarda o histórico das conversas na pasta dela, fora do cofre. A CLI do Claude Code corre sem histórico.
agente-motor-cadastrado = Motor registado
agente-cadastrar = Registar
agente-provider = Fornecedor
agente-nome-dica = Como o motor aparece na aplicação.
agente-modelo-dica = O nome dos modelos muda com o tempo; confirme no fornecedor.
agente-endpoint-padrao = Predefinição: { $url }
agente-max-tokens-dica = Vazio: 4096.
agente-chave-dica = Gravada cifrada no cofre; nunca volta a ser mostrada.
agente-cadastre-de-novo = registe-a de novo
agente-substituir-chave = Escreva para substituir a chave
agente-cole-chave = Cole a chave do fornecedor
agente-chave-apagada = Chave eliminada
agente-apagar-chave = Eliminar a chave registada
agente-template-dica = Marcadores: { "{prompt}" }, { "{mcp_config}" }, { "{model}" }, { "{workdir}" }. Trechos entre [ ] desaparecem quando o marcador está vazio.
agente-template-padrao = Vazio usa a predefinição, com as ferramentas nativas da CLI desligadas: { $template }

## Limites de gasto

agente-limites-salvos = Limites de { $provider } guardados
agente-limites-rodape = Custos em dólares, estimados pela tabela de preços. Acima do limite, o motor é saltado (fallback); acima do aviso, o chat pede confirmação antes de enviar.
agente-sem-provider = Nenhum fornecedor de API em uso.
agente-limites-por-provider = Limites por fornecedor
agente-limitar-outro = Limitar outro fornecedor
agente-limitar-outro-mais = Limitar outro fornecedor…

## Utilização e custo

agente-uso-titulo = Utilização e custo
agente-sem-preco =
    { $n ->
        [one] { "*" } { $n } chamada sem preço registado para o modelo: custo fora do total. Registe o preço na tabela abaixo.
       *[other] { "*" } { $n } chamadas sem preço registado para o modelo: custo fora do total. Registe o preço na tabela abaixo.
    }
agente-custo-fonte = Fonte: chamadas registadas × tabela de preços (as CLIs e os modelos locais custam zero na aplicação)
agente-uso-por-provider = Uso por fornecedor

## Preços por modelo

agente-preco-salvo = Preço guardado
agente-modelo-prefixo-dica = { "*" } = predefinição do fornecedor
agente-padrao = predefinido
agente-apagar-preco = Eliminar o preço de { $modelo }
agente-precos-vazio = Nenhum preço registado: o custo das chamadas fica fora do total.

## Histórico e anexos no cofre

agente-ret-nenhum-dica = O ficheiro fica só na memória da aplicação enquanto o cofre está aberto (o agente volta a lê-lo nos próximos turnos da conversa) e não vai para o cofre; a conversa guarda só o nome e o tamanho.
agente-ret-7-dica = Guarda os ficheiros durante 7 dias e depois elimina-os (a limpeza diária trata disso).
agente-ret-sempre-dica = Guarda os ficheiros para sempre, até eliminar a conversa.
agente-faxina-feita = Limpeza feita
agente-faxina-texto = { $conversas } conversa(s) e { $anexos } anexo(s) expirados eliminados.
agente-faxina-texto-encolheu = { $conversas } conversa(s) e { $anexos } anexo(s) expirados eliminados; o cofre encolheu { $tamanho }.
agente-nao-recomendado = Não recomendado: imagens e PDFs tornam grande a base de dados cifrada (cópias de segurança mais lentas e maiores) e mantêm cópias de documentos sensíveis. Prefira "Não guardar".
agente-nao-recomendado-hoje = Não recomendado: imagens e PDFs tornam grande a base de dados cifrada (cópias de segurança mais lentas e maiores) e mantêm cópias de documentos sensíveis. Prefira "Não guardar". Hoje: { $tamanho } em anexos.
agente-apagar-sem-uso = Eliminar conversas sem uso há
agente-faxina-diaria = A limpeza corre uma vez por dia com o cofre aberto. As conversas fixadas ficam.
agente-faxina-nunca = As conversas ficam até as eliminar. O cofre pode crescer com o tempo.
agente-ultima-faxina = Última limpeza
agente-limpar-agora-dica = Aplica já as regras acima e compacta o cofre se sobrar espaço.

# Imagens com texto lido pelo OCR: o que vai ao motor.
agente-anexo-texto-lido = texto lido no computador
agente-imagem-modo-rotulo = O que enviar das imagens
agente-imagem-modo-pergunta = Texto lido no computador ({ $confianca }% de confiança). Enviar:
agente-imagem-modo-texto = Só o texto
agente-imagem-modo-imagem = Só a imagem
agente-imagem-modo-ambos = Os dois
agente-imagem-so-texto = Este motor não lê imagens: vai o texto lido no computador ({ $confianca }% de confiança).

agente-pergunta-respondeu = Respondeu: { $resposta }
