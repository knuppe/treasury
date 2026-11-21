# Política de segurança

[English](SECURITY.md)

## Como relatar uma falha

**Não abra um issue público** para uma falha de segurança. Use o relato privado do GitHub:
[Security › Report a vulnerability](../../security/advisories/new). Só o mantenedor vê o relato, e
a conversa continua ali até a correção sair.

No relato, diga:

- a versão do Treasury e o sistema (macOS, Linux ou Windows);
- o que a falha permite fazer e em que condições (cofre aberto ou bloqueado, servidor MCP ligado,
  arquivo importado, atualização automática);
- os passos para reproduzir, de preferência no **cofre de demonstração**.

Nunca envie extratos, senhas, chaves de API ou dados financeiros reais, nem no relato privado. Se
um arquivo é necessário para reproduzir, monte um com valores inventados.

## O que esperar

- Confirmo o recebimento em até 7 dias.
- Combino com você o prazo da correção e a data de publicar o aviso. A correção sai numa versão
  nova, entregue pela atualização automática, e o aviso de segurança dá o crédito a quem relatou,
  se a pessoa quiser.
- O Treasury é mantido por uma pessoa, sem programa de recompensa.

## Versões atendidas

Só a versão mais recente recebe correções de segurança. A atualização automática vem ligada por
padrão (Ajustes › Atualizações, que também pode só avisar ou ficar desligada) e só instala pacotes assinados com as chaves do
projeto (veja [docs/auto-update.md](docs/auto-update.md)).

## O que está no escopo

- A criptografia e o desbloqueio do cofre (SQLCipher, Argon2id, Keychain, Touch ID, chave física).
- O servidor MCP local: acesso sem chave válida, escrita fora da fila de aprovação, leitura além
  do que a chave permite.
- O agente de IA: alteração aplicada sem a aprovação da pessoa, dado enviado a um motor que não
  foi escolhido.
- A atualização automática: pacote aceito sem a assinatura certa, troca do endereço de download.
- A importação de arquivos (extratos, PDFs, planilhas, e-mail): arquivo malicioso que executa
  código, lê arquivos fora do cofre ou trava o app.
- A janela do app (webview): execução de script vindo de dados importados ou de respostas da IA.

## Fora do escopo

- Ataques que exigem o computador já comprometido com o cofre aberto (quem controla a sessão
  aberta vê o que a pessoa vê).
- O que acontece com os dados depois que a pessoa os envia a um motor de IA na nuvem ou a um
  cliente MCP que ela mesma configurou.
- Falhas em serviços de terceiros (cotações, índices, provedores de IA) que não afetam o app.
- Engenharia social e ataques que dependem de a pessoa instalar um pacote de fora das releases
  oficiais.

O modelo de ameaça completo está em [docs/security.md](docs/security.md) e as defesas locais em
[docs/local-security.md](docs/local-security.md).

## Boa-fé

Pesquisa feita de boa-fé, na sua própria instalação e no cofre de demonstração, sem acessar dados
de outras pessoas e dando tempo para a correção antes de divulgar, é bem-vinda. Não vou tomar
medidas contra quem age assim.
