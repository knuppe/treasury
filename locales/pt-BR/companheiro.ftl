# Agente companheiro: o painel lateral do agente presente em todas as telas (⌘J), com o
# contexto da tela, os pedidos rápidos e as propostas aprovadas ali mesmo
# (docs/i18n-design.md). pt-BR é o idioma de origem.

## Pedidos rápidos (o texto vai para o motor de IA)

companheiro-pedido-aqui-rotulo = O que posso fazer aqui?
companheiro-pedido-aqui = O que posso fazer nesta tela? Use o contexto da tela e os dados para sugerir os próximos passos concretos e ofereça-se para propor as alterações.
companheiro-pedido-resumo-rotulo = Resumir esta tela
companheiro-pedido-resumo = Resuma o que esta tela mostra agora, com os números principais, e aponte o que merece atenção.
companheiro-pedido-cadastro-rotulo = Onde vocês têm contas?
companheiro-pedido-cadastro = Quero cadastrar onde a família tem dinheiro. Pergunte onde temos contas (bancos, cartões de crédito, corretoras, contas no exterior) e, para cada uma: proponha o cadastro (create_account com institution_name; cartões com fechamento e vencimento), diga o melhor caminho para trazer os dados (OFX ou CSV do banco, fatura em PDF, relatório da Área do Investidor B3, notas de corretagem em PDF) e onde baixar cada arquivo. Não invente saldos: pergunte.

## Painel

companheiro-rotulo = Agente companheiro
companheiro-abrir = Abrir o agente
companheiro-fechar = Fechar o agente
companheiro-nova-conversa = Nova conversa
companheiro-anteriores = Conversas anteriores
companheiro-opcoes = Opções do agente
companheiro-largura = Largura do painel do agente
companheiro-log = Conversa no painel do agente
companheiro-mensagem = Mensagem para o agente (painel)
companheiro-placeholder = Pergunte ou peça um cadastro…
companheiro-enviar-assim = Enviar assim
companheiro-aviso = A IA pode errar. Confira antes de aprovar.
companheiro-aviso-envio = Esta conversa será enviada para { $provedor }. A IA pode errar. Confira antes de aprovar.
companheiro-autonomo-tag = Autônomo
companheiro-autonomo-tag-dica = Modo autônomo ligado: escritas reversíveis valem na hora. Desligue em Opções do agente.
companheiro-fora-das-telas = Fora das telas

## Opções e modo autônomo

companheiro-autonomo-ligado-dica = Escritas reversíveis (categorizar, cadastrar, orçamento) valem na hora e podem ser desfeitas. Sensíveis continuam pedindo aprovação item a item.
companheiro-autonomo-desligado-dica = Desligado: toda escrita do agente vira proposta para você aprovar.
companheiro-autonomo-ligado = Modo autônomo ligado
companheiro-autonomo-desligado = Modo autônomo desligado
companheiro-autonomo-ligado-texto = Escritas reversíveis valem na hora; sensíveis seguem para aprovação.
companheiro-autonomo-desligado-texto = Toda escrita do agente vira proposta.
companheiro-nao-foi-possivel = Não foi possível
companheiro-tela-cheia = Abrir em tela cheia ({ $tela })

## Contexto da tela

companheiro-contexto-vai = Vai para o motor de IA junto com cada mensagem, como dado (nunca instrução). Valores em centavos.
companheiro-contexto-desligado = Desligado: só a mensagem vai para o motor.
companheiro-contexto-grande = Contexto grande demais para enviar; só a mensagem vai.
companheiro-contexto-previa = Prévia do contexto da página
companheiro-contexto-ver = Ver o que vai junto com a mensagem
companheiro-contexto-pagina = Contexto desta página: { $tela }
companheiro-enviar-contexto = Enviar contexto

## Boas-vindas e sugestões

companheiro-sugestoes = Sugestões do agente
companheiro-como-ajudar = Como posso ajudar?
companheiro-vendo = Estou vendo { $tela }. Posso consultar, cadastrar e organizar; toda escrita vira proposta para você aprovar aqui.
companheiro-pedidos-rapidos = Pedidos rápidos
companheiro-nesta-tela = Nesta tela
companheiro-nada-pendente = Nada pendente nesta tela.
companheiro-perguntar = Perguntar ao agente: { $titulo }
companheiro-abrir-tela = Abrir a tela

## Conversas anteriores

companheiro-voltar = Voltar à conversa
companheiro-buscar-conversa = Buscar conversa
companheiro-buscar-titulo = Buscar pelo título
companheiro-nenhuma-com = Nenhuma conversa com “{ $busca }”.
companheiro-fixadas = Fixadas
companheiro-conversas-fixadas = Conversas fixadas
companheiro-recentes = Recentes
companheiro-conversas-recentes = Conversas recentes
companheiro-ver-na-tela = Ver na tela { $tela }

## Proposta na conversa

companheiro-falha-aplicar = A proposta não pôde ser aplicada
companheiro-aprovada = Proposta aprovada
companheiro-aprovada-texto = A alteração vale agora e pode ser desfeita no histórico.
companheiro-rejeitada = Proposta rejeitada
companheiro-carregando = Carregando a proposta…
companheiro-ver-alteracoes = Ver as alterações
companheiro-sensivel = Sensível: confira antes de aprovar.
