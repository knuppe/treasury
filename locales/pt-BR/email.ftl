# E-mail de ingestão: aba E-mail de Lançamentos, contas (caixa IMAP ou pasta local), regras e
# auditoria (docs/email-ingest.md, docs/i18n-design.md). pt-BR é o idioma de origem.
# Ficam iguais em qualquer idioma: a palavra `membros` nas listas de remetentes e os grupos
# `valor`, `descricao` e `data` do modelo do corpo (o núcleo reconhece esses nomes), INBOX e as
# marcas \Junk, \Trash… do servidor.

## Tipos de peça

email-tipo-extrato = Extrato
email-tipo-fatura-pdf = Fatura de cartão em PDF
email-tipo-nota = Nota de corretagem
email-tipo-b3 = Relatório da B3
email-tipo-previdencia = Extrato de previdência
email-tipo-holerite = Holerite
email-tipo-cupom = Cupom fiscal
email-tipo-imagem = Imagem
email-tipo-compra = Aviso de compra
email-tipo-pix = Aviso de Pix
email-tipo-modelo = Pelo modelo da regra
email-tipo-desconhecido = Não reconhecido

## Situação da mensagem e da peça

email-status-revisar = Para revisar
email-status-importada = Importada
email-status-repetida = Repetida
email-status-falhou = Falhou
email-status-nao-reconhecida = Não reconhecida
email-status-ignorada = Ignorada
email-status-ia-lendo = IA lendo

## O que a regra faz e a IA

email-acao-revisar = Revisar
email-acao-auto = Aplicar sozinho
email-acao-ignorar = Ignorar
email-ia-nunca = Nunca
email-ia-a-pedido = A pedido
email-ia-auto = Sozinha (até 10 por dia)

## Retenção (apagar do provedor)

email-retencao-nunca = Nunca apagar
email-retencao-sucesso = Depois de importada (ou repetida)
email-retencao-dias = Depois de alguns dias
email-retencao-ambos = O que vier primeiro
email-estado-desligado = Não apaga nada
email-estado-teste = Modo de teste
email-estado-confirmacao = Esperando confirmação
email-estado-ativo = Apagando

## Auditoria

email-auditoria-busca = Busca
email-auditoria-recebida = Recebida
email-auditoria-importada = Importada
email-auditoria-repetida = Repetida
email-auditoria-apagada = Apagada
email-auditoria-apagaria = Teste: apagaria
email-auditoria-nao-apagou = Não apagou
email-auditoria-ia = IA
email-auditoria-revisao = Revisão
email-auditoria-erro = Erro

## Fila do e-mail

email-fila = Fila do e-mail
email-mensagens = Mensagens
email-para-revisar = Para revisar
email-tudo = Tudo
email-vazio-kicker = E-mail de ingestão
email-vazio-titulo = Receba extratos, faturas e cupons por e-mail
email-vazio-descricao = Crie uma caixa só para o Treasury (encaminhe para ela os e-mails do banco) ou escolha uma pasta do computador onde você salva anexos e mensagens .eml. O que passar nos filtros vem para cá; nada é apagado sem você ligar e confirmar.
email-configurar = Configurar o e-mail
email-nada-revisar = Nada esperando revisão. As contas buscam sozinhas a cada intervalo, com o app aberto e o cofre desbloqueado.
email-nenhuma-mensagem = Nenhuma mensagem ainda.

email-mensagem = Mensagem
email-sem-assunto = (sem assunto)
email-encaminhada-por = { $original } · encaminhada por { $remetente }
email-pasta-principal = Pasta principal
email-pasta = Pasta { $pasta }
email-regra-nome = regra "{ $regra }"
email-sem-regra = sem regra (padrão da conta)
email-autenticado = Remetente autenticado
email-nao-autenticado = Remetente não autenticado
email-ignorar-mensagem = Ignorar a mensagem

## Peças da mensagem

email-cartao-final = cartão final { $final }
email-ia-propostas =
    { $n ->
        [one] { $n } proposta na fila de aprovação do agente
       *[other] { $n } propostas na fila de aprovação do agente
    }
email-conta-lancamento = Conta do lançamento
email-escolha-conta = Escolha a conta
email-lancar = Lançar
email-importar-cupom = Importar cupom
email-importar = Importar
email-abrir-importador = Abrir no importador
email-pedir-ia = Pedir à IA
email-anexar-agente = Anexar ao agente
email-ja-importei = Já importei
email-ignorar = Ignorar
email-importado = Importado
email-ja-estava = Já estava no Treasury
email-ficou-revisao = Ficou na revisão
email-erro-abrir = Não deu para abrir a peça
email-erro-anexar = Não deu para anexar
# Texto que vai pronto para o agente de IA.
email-agente-chegou = Chegou pelo e-mail (de { $remetente }).

## Modo de teste da exclusão

email-teste-ate = Modo de teste da exclusão até { $data }: nada é apagado, só registrado.
email-teste-acabou = O modo de teste acabou. A exclusão só começa depois da sua confirmação; até lá, continua em teste.
email-teriam-saido =
    { $n ->
        [one] { $n } mensagem teria saído (veja na auditoria).
       *[other] { $n } mensagens teriam saído (veja na auditoria).
    }
email-nada-teria-saido = Nada teria saído até agora.
email-confirmar-exclusao = Confirmar a exclusão (sem desfazer)

## Contas

email-contas = Contas
email-nova-conta = Nova conta
email-pasta-local = Pasta local
email-conta-nome = Conta { $nome }
email-desligada = desligada
email-a-cada = a cada { $min } min
email-n-para-revisar = { $n } para revisar
email-buscando = Buscando agora…
email-ultima-busca = Última busca { $quando }: { $resultado }
email-busca-resumo = { $novas } novas, { $fila } na fila, { $recusadas } recusadas, { $aplicadas } aplicadas sozinhas
email-ainda-nao-buscou = Ainda não buscou.
email-cadastre-senha = Cadastre a senha de app (Editar).
email-backup-restaurado = Backup restaurado: teste a conexão antes de voltar a buscar.
email-buscar-agora = Buscar agora
email-testar = Testar
email-conexao-ok = Conexão funcionando
email-n-arquivos-pasta =
    { $n ->
        [one] { $n } arquivo na pasta.
       *[other] { $n } arquivos na pasta.
    }
email-teste-le = Lê { $pastas }; para apagar, { $como }.
email-nenhuma-pasta = nenhuma pasta
email-apagar-conta = Apagar a conta?
email-apagar-conta-descricao = A fila e a auditoria desta conta saem do Treasury. Nada é apagado no provedor nem na pasta, e o que já foi importado continua.

## Regras

email-regras = Regras
email-nova-regra = Nova regra
email-regras-tabela = Regras do e-mail
email-todas-contas = Todas as contas
email-col-ordem = Ordem
email-col-regra = Regra
email-col-conta = Conta
email-col-acao = Ação
email-col-ia = IA
email-col-acoes = Ações
email-regra-de = De: { $remetentes }
email-qualquer-remetente = Qualquer remetente
email-regra-assunto = assunto: { $termos }
email-sem-regras = Sem regras, a caixa IMAP ignora tudo (a pasta local manda para a revisão). Crie uma regra por banco ou loja: remetente, assunto e o que fazer.

## Auditoria (cartão)

email-auditoria = Auditoria
email-auditoria-tabela = Auditoria do e-mail
email-col-quando = Quando
email-col-o-que = O quê
email-col-detalhe = Detalhe
email-nada-registrado = Nada registrado ainda.

## Conta do e-mail de ingestão (janela)

email-conta-nome-padrao = Caixa da família
email-conta-salva = Conta salva
email-conta-titulo = Conta: { $nome }
email-conta-nova = Nova conta do e-mail de ingestão
email-conta-descricao = Recomendado: uma caixa só para o Treasury, alimentada por encaminhamento automático dos e-mails do banco. Só roda com o app aberto e o cofre desbloqueado; nada é enviado (sem SMTP).
email-salvar-testar = Salvar e testar
email-tipo-conta = Tipo da conta
email-tipo-imap = Caixa de e-mail (IMAP)
email-tipo-pasta = Pasta do computador
email-nome = Nome
email-pasta-vigiada = Pasta vigiada
email-pasta-vigiada-dica = Salve ali anexos, extratos e mensagens .eml (cada subpasta é uma pasta dos filtros). O que já foi tratado pode ir para Processados.
email-pasta-exemplo = /Users/voce/Documents/Treasury e-mail
email-escolher = Escolher…
email-endereco = Endereço da caixa
email-endereco-dica = Só o domínio é consultado (nunca o endereço).
email-endereco-exemplo = familia.treasury@gmail.com
email-procurar-config = Procurar a configuração
email-provedor = Provedor
email-provedor-manual = Outro (manual)
email-servidor = Servidor IMAP
email-servidor-exemplo = imap.exemplo.com
email-porta = Porta
email-onde-criar = Onde criar
email-seguranca = Segurança
email-seguranca-dica = Fora do Proton Bridge no próprio computador, só TLS (porta 993).
email-starttls = STARTTLS (só Bridge local)
email-usuario = Usuário
email-senha-app = Senha de app
email-senha-cadastrada = Cadastrada ({ $dica }); deixe vazio para manter.
email-senha-cifrada = Fica cifrada no cofre e nunca aparece de novo.
email-senha-aviso = O Treasury vai ler as pastas que você escolher usando esta senha de app. A senha fica no cofre.
email-certificado = Certificado do Bridge (cert.pem)
email-certificado-fixado = Certificado fixado: só ele é aceito.
email-certificado-onde = Exportado em Bridge › Ajustes › Avançado › Exportar certificados TLS.
email-que-caixa = Que caixa é
email-caixa-dedicada = Só para o Treasury (recomendado)
email-caixa-principal = A caixa principal de alguém
email-caixa-principal-aviso = Na caixa principal nenhuma pasta é lida até você escolher, e só o que as regras pegarem entra.
email-gmail-lixeira-aviso = No Gmail, mandar para a lixeira tira a mensagem de todos os marcadores.

email-pastas-remetentes = Pastas e remetentes
email-pastas-lidas = Pastas lidas
email-pastas-lidas-dica-pasta = INBOX é a raiz da pasta; subpastas pelo caminho (Bancos/*).
email-pastas-lidas-dica-imap = Nomes ou curinga (Bancos/*). A caixa exclusiva lê só a INBOX.
email-pastas-bloqueadas = Pastas bloqueadas
email-pastas-bloqueadas-dica = Pelas marcas do servidor (\Junk, \Trash…), nunca pelo nome. Bloqueio vence a permissão.
email-remetentes-permitidos = Remetentes permitidos (todas as regras)
email-remetentes-permitidos-dica = Vazio: qualquer um. Endereço, domínio (banco.com.br), subdomínios (*.itau.com.br), curinga, /regex/ ou membros.
email-remetentes-bloqueados = Remetentes bloqueados
email-enderecos-familia = Endereços da família
email-enderecos-familia-dica = Quem encaminha para a caixa (o padrão membros das regras). Encaminhada por um deles só vale sozinha com o remetente original na regra.
email-sem-regra-case = Sem regra que case
email-padrao-ignorar = Ignorar (padrão; só conta)
email-padrao-revisao = Mandar para a revisão

email-buscas-retencao = Buscas e retenção
email-intervalo = Buscar a cada (minutos)
email-intervalo-dica = De 5 a 120.
email-apagar-provedor = Apagar do provedor
email-carencia-importada = Carência depois de importada (dias)
email-quantos-dias = Depois de quantos dias
email-quantos-dias-dica = Vale também para o que ainda espera revisão ou não foi reconhecido (a cópia fica no cofre).
email-arquivo-destino = O que fazer com o arquivo
email-mover-processados = Mover para Processados
email-mandar-lixeira = Mandar para a Lixeira
email-purgar-gmail = Apagar também da lixeira do Gmail
email-purgar-gmail-dica = Desligado: a mensagem fica 30 dias na lixeira antes de sumir.
email-retencao-aviso = Primeiro, 7 dias de modo de teste: o Treasury só registra o que teria apagado. Depois, só apaga com a sua confirmação. Nunca apaga sem regra, fora das pastas lidas nem sem conferir a cópia guardada; a exclusão não tem desfazer.
email-guardar-original = Guardar a mensagem original no cofre
email-guardar-original-dica = Recomendado: antes de apagar do provedor, o Treasury confere que a cópia está inteira.
email-conectou = Conectou. Lê: { $pastas }.
email-conectou-nenhuma = nenhuma pasta (confira as pastas lidas)
email-para-apagar = Para apagar, { $como }.

## Regra do e-mail (janela)

email-regra-salva = Regra salva
email-regra-titulo = Regra: { $nome }
email-regra-nova = Nova regra do e-mail
email-regra-descricao = A ordem é sempre: pastas, bloqueios da conta e as regras pela ordem (a primeira que casar decide). Assunto sem diferenciar maiúsculas nem acentos, já sem Fwd:/Enc:/RES:.
email-regra-nome-exemplo = Faturas do Nubank
email-regra-conta = Conta
email-regra-ordem = Ordem
email-regra-ligada = Regra ligada
email-regra-de-label = De (remetentes)
email-regra-de-dica = nome@banco.com.br, banco.com.br, *.itau.com.br, fatura*@*.com.br, /regex/ ou membros. Encaminhada: vale o remetente original.
email-regra-bloqueados = Remetentes bloqueados nesta regra
email-assunto-contem = Assunto contém
email-assunto-contem-dica = Termos separados por vírgula.
email-assunto-contem-exemplo = fatura, extrato
email-assunto-nao-contem = Assunto não contém
email-assunto-nao-contem-exemplo = promoção
email-todos-termos = Todos os termos (padrão: qualquer um)
email-assunto-regex = Assunto por expressão regular
email-assunto-regex-dica = Opcional; o erro diz onde está o problema.
email-avancadas = Pastas e condições avançadas
email-so-pastas = Só nestas pastas
email-so-pastas-dica = Vazio: todas as lidas.
email-avancadas-combinadas = Esta regra tem condições avançadas combinadas (E/OU/NÃO) que ficam como estão.
email-anexo-nome = Tem anexo com nome
email-anexo-nome-exemplo = *.pdf, fatura*
email-corpo-contem = O corpo contém
email-ate-kb = Até (KB)
email-o-que-fazer = O que fazer
email-acao-regra = Ação da regra
email-auto-aviso = Sozinho só com leitor determinístico (OFX, cupom fiscal, aviso de compra ou de Pix), remetente autenticado pelo provedor e listado acima, e a prévia sem avisos. O resto vai para a revisão. O que a IA lê nunca entra sozinho.
email-ia = IA
email-ia-dica = A IA recebe só a peça, como dado de terceiros, e só as ferramentas de prévia e importação; o resultado é sempre proposta.
email-retencao = Retenção
email-retencao-conta = A da conta
email-carencia = Carência (dias)
email-depois-de = Depois de (dias)
email-modelo-corpo = Modelo do corpo (aviso que o Treasury não lê)
email-modelo-expressao = Expressão com os grupos valor, descricao e data
email-modelo-exemplo = Ex.: { $exemplo }
email-modelo-tipo = Tipo
email-modelo-saida = Saída
email-modelo-entrada = Entrada
email-modelo-conta = Conta dos lançamentos
email-modelo-escolher-revisao = Escolher na revisão
email-testar-exemplo = Testar com um exemplo
email-remetente = Remetente
email-assunto = Assunto
email-assunto-exemplo = Fwd: Sua fatura chegou
email-exemplo-passa = Exemplo: passa ({ $motivo })
email-exemplo-nao-passa = Exemplo: não passa ({ $motivo })
email-teste-recentes = Das { $total } mensagens recentes, esta regra pegaria { $pegaria }.
email-testar-nada = Testar não importa nem apaga nada.

email-pasta-vigiar = Pasta para o Treasury vigiar
