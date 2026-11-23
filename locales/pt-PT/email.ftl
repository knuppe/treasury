# Variante pt-PT (e-mail de ingestão): o que faltar cai para pt-BR. Revisão: rascunho por IA.
# Ficam iguais: a palavra `membros` nas listas de remetentes, os grupos `valor`, `descricao` e
# `data` do modelo do corpo, INBOX e as marcas \Junk, \Trash… do servidor.

## Tipos de peça

email-tipo-fatura-pdf = Extrato de cartão em PDF
email-tipo-holerite = Recibo de vencimento
email-tipo-cupom = Talão

## Situação da mensagem e da peça

email-status-revisar = Para rever
email-status-ia-lendo = IA a ler

## O que a regra faz e a IA

email-acao-revisar = Rever
email-acao-auto = Aplicar automaticamente
email-ia-auto = Automática (até 10 por dia)

## Retenção (eliminar do fornecedor)

email-retencao-nunca = Nunca eliminar
email-estado-desligado = Não elimina nada
email-estado-confirmacao = A aguardar confirmação
email-estado-ativo = A eliminar

## Auditoria

email-auditoria-busca = Verificação
email-auditoria-apagada = Eliminada
email-auditoria-apagaria = Teste: eliminaria
email-auditoria-nao-apagou = Não eliminou

## Fila do e-mail

email-para-revisar = Para rever
email-vazio-titulo = Receba extratos, faturas e talões por e-mail
email-vazio-descricao = Crie uma caixa só para o Treasury (reencaminhe para ela os e-mails do banco) ou escolha uma pasta do computador onde guarda anexos e mensagens .eml. O que passar nos filtros vem para aqui; nada é eliminado sem que o ative e confirme.
email-nada-revisar = Nada à espera de revisão. As contas verificam automaticamente a cada intervalo, com a aplicação aberta e o cofre desbloqueado.

email-encaminhada-por = { $original } · reencaminhada por { $remetente }
email-sem-regra = sem regra (predefinição da conta)

## Peças da mensagem

email-importar-cupom = Importar talão
email-erro-abrir = Não foi possível abrir a peça
email-erro-anexar = Não foi possível anexar

## Modo de teste da eliminação

email-teste-ate = Modo de teste da eliminação até { $data }: nada é eliminado, só registado.
email-teste-acabou = O modo de teste acabou. A eliminação só começa depois da sua confirmação; até lá, continua em teste.
email-confirmar-exclusao = Confirmar a eliminação (sem desfazer)

## Contas

email-n-para-revisar = { $n } para rever
email-buscando = A verificar agora…
email-ultima-busca = Última verificação { $quando }: { $resultado }
email-busca-resumo = { $novas } novas, { $fila } na fila, { $recusadas } recusadas, { $aplicadas } aplicadas automaticamente
email-ainda-nao-buscou = Ainda não verificou.
email-cadastre-senha = Registe a palavra-passe de aplicação (Editar).
email-backup-restaurado = Cópia de segurança restaurada: teste a ligação antes de voltar a verificar.
email-buscar-agora = Verificar agora
email-conexao-ok = Ligação a funcionar
email-n-arquivos-pasta =
    { $n ->
        [one] { $n } ficheiro na pasta.
       *[other] { $n } ficheiros na pasta.
    }
email-teste-le = Lê { $pastas }; para eliminar, { $como }.
email-apagar-conta = Eliminar a conta?
email-apagar-conta-descricao = A fila e a auditoria desta conta saem do Treasury. Nada é eliminado no fornecedor nem na pasta, e o que já foi importado continua.

## Regras

email-sem-regras = Sem regras, a caixa IMAP ignora tudo (a pasta local envia para a revisão). Crie uma regra por banco ou loja: remetente, assunto e o que fazer.

## Auditoria (cartão)

email-nada-registrado = Nada registado ainda.

## Conta do e-mail de ingestão (janela)

email-conta-salva = Conta guardada
email-conta-descricao = Recomendado: uma caixa só para o Treasury, alimentada por reencaminhamento automático dos e-mails do banco. Só funciona com a aplicação aberta e o cofre desbloqueado; nada é enviado (sem SMTP).
email-salvar-testar = Guardar e testar
email-pasta-vigiada-dica = Guarde ali anexos, extratos e mensagens .eml (cada subpasta é uma pasta dos filtros). O que já foi tratado pode ir para Processados.
email-pasta-exemplo = /Users/utilizador/Documents/Treasury e-mail
email-provedor = Fornecedor
email-usuario = Utilizador
email-senha-app = Palavra-passe de aplicação
email-senha-cadastrada = Registada ({ $dica }); deixe vazio para manter.
email-senha-cifrada = Fica cifrada no cofre e nunca volta a aparecer.
email-senha-aviso = O Treasury vai ler as pastas que escolher usando esta palavra-passe de aplicação. A palavra-passe fica no cofre.
email-certificado-fixado = Certificado fixado: só ele é aceite.
email-caixa-principal-aviso = Na caixa principal nenhuma pasta é lida até a escolher, e só entra o que as regras apanharem.
email-gmail-lixeira-aviso = No Gmail, enviar para o lixo tira a mensagem de todas as etiquetas.

email-pastas-lidas-dica-imap = Nomes ou padrões com * (Bancos/*). A caixa exclusiva lê só a INBOX.
email-pastas-bloqueadas-dica = Pelas marcas do servidor (\Junk, \Trash…), nunca pelo nome. O bloqueio prevalece sobre a permissão.
email-remetentes-permitidos-dica = Vazio: qualquer um. Endereço, domínio (banco.pt), subdomínios (*.banco.pt), padrões com *, /regex/ ou membros.
email-enderecos-familia-dica = Quem reencaminha para a caixa (o padrão membros das regras). Uma mensagem reencaminhada por um deles só é aplicada automaticamente se o remetente original estiver na regra.
email-sem-regra-case = Sem regra que corresponda
email-padrao-ignorar = Ignorar (predefinição; só conta)
email-padrao-revisao = Enviar para a revisão

email-buscas-retencao = Verificações e retenção
email-intervalo = Verificar a cada (minutos)
email-apagar-provedor = Eliminar do fornecedor
email-quantos-dias-dica = Aplica-se também ao que ainda aguarda revisão ou não foi reconhecido (a cópia fica no cofre).
email-arquivo-destino = O que fazer com o ficheiro
email-mandar-lixeira = Enviar para o Lixo
email-purgar-gmail = Eliminar também do lixo do Gmail
email-purgar-gmail-dica = Desligado: a mensagem fica 30 dias no lixo antes de desaparecer.
email-retencao-aviso = Primeiro, 7 dias de modo de teste: o Treasury só regista o que teria eliminado. Depois, só elimina com a sua confirmação. Nunca elimina sem regra, fora das pastas lidas nem sem verificar a cópia guardada; a eliminação não pode ser desfeita.
email-guardar-original-dica = Recomendado: antes de eliminar do fornecedor, o Treasury verifica que a cópia está inteira.
email-conectou = Ligou-se. Lê: { $pastas }.
email-conectou-nenhuma = nenhuma pasta (verifique as pastas lidas)
email-para-apagar = Para eliminar, { $como }.

## Regra do e-mail (janela)

email-regra-salva = Regra guardada
email-regra-descricao = A ordem é sempre: pastas, bloqueios da conta e as regras pela ordem (a primeira que corresponder decide). Assunto sem distinguir maiúsculas nem acentos, já sem Fwd:/Enc:/RES:.
email-regra-nome-exemplo = Extratos do cartão
email-regra-de-dica = nome@banco.pt, banco.pt, *.banco.pt, extrato*@*.pt, /regex/ ou membros. Reencaminhada: conta o remetente original.
email-todos-termos = Todos os termos (predefinição: qualquer um)
email-auto-aviso = Automático só com leitor determinístico (OFX, talão, aviso de compra ou de Pix), remetente autenticado pelo fornecedor e listado acima, e a pré-visualização sem avisos. O resto vai para a revisão. O que a IA lê nunca entra automaticamente.
email-ia-dica = A IA recebe só a peça, como dados de terceiros, e só as ferramentas de pré-visualização e importação; o resultado é sempre uma proposta.
email-assunto-exemplo = Fwd: O seu extrato chegou
email-teste-recentes = Das { $total } mensagens recentes, esta regra apanharia { $pegaria }.
email-testar-nada = Testar não importa nem elimina nada.
