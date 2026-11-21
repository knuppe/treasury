# Como contribuir

[English](CONTRIBUTING.md)

Obrigado pelo interesse no Treasury. Relatos de erro, ideias, traduções e código são bem-vindos.
Issues e pull requests podem ser escritos em português, inglês ou espanhol.

## Quem compila e publica o Treasury

As versões do Treasury são compiladas e publicadas **exclusivamente pelo Knuppe**, o dono do
projeto. Nenhuma versão é publicada sem a assinatura do dono, feita com uma chave de hardware que
só ele tem, e o app instalado recusa qualquer atualização sem essa assinatura.

O motivo é segurança. Quem usa o Treasury põe nele a vida financeira da família e confia que o
programa está livre de código malicioso. Essa confiança é numa pessoa identificada, e o Knuppe
assume o compromisso de fazer o possível para que cada versão saia limpa: compilada a partir do
código revisado deste repositório, seguindo as regras do projeto e conferida antes de ser
assinada. Como isso funciona e como qualquer pessoa pode conferir: [docs/trust.md](docs/trust.md).

Na prática, para quem contribui:

- uma contribuição aceita entra no código, mas só chega a quem usa na versão seguinte compilada e
  assinada pelo Knuppe;
- um build feito por outra pessoa, inclusive de um fork, não é uma versão do Treasury: não recebe
  atualização e não deve ser distribuído com esse nome;
- mudanças nos arquivos que decidem o que o app aceita como atualização e como uma versão é
  feita (a lista está no [.github/CODEOWNERS](.github/CODEOWNERS)) só entram com a revisão dele.

## Contribuições e a visão do projeto

O Treasury segue a visão que o Knuppe tem para o software ([docs/vision.md](docs/vision.md)): o
que entra, o que fica de fora e como as coisas funcionam. Uma contribuição pode ser recusada e
fechada sem discussão quando não combina com essa visão, mesmo que esteja correta e bem escrita.
Não é um julgamento sobre quem contribuiu: é o que mantém o projeto coerente e possível de manter
por uma pessoa. Para não perder trabalho, abra um issue antes de começar algo grande e espere a
resposta.

## Antes de tudo: nunca publique dados financeiros

O Treasury guarda a vida financeira de uma família. Um issue ou pull request é **público e fica
para sempre**, mesmo que depois seja apagado.

- **Não anexe** extratos, faturas, holerites, notas de corretagem, informes de rendimentos,
  planilhas ou prints com valores, nomes, CPF, números de conta ou de cartão.
- Para mostrar um problema, use o **cofre de demonstração**: ao criar um cofre, escolha
  "Cofre de demonstração" (o atalho também fica em Cofres). Ele tem só dados fictícios e abre
  sem senha.
- Se o erro só acontece com um arquivo seu (um extrato de um banco, por exemplo), descreva o
  **formato** (banco, tipo de arquivo, o que muda em relação ao esperado), nunca o conteúdo. Se
  precisar, invente um arquivo com o mesmo formato e valores falsos.
- Falhas de segurança **não** vão em issue público: siga o [SECURITY.md](SECURITY.md).

## Relatar um erro ou sugerir uma ideia

Abra um [issue](../../issues/new/choose) e escolha o modelo: erro, ideia ou tradução. Diga a
versão do Treasury (Ajustes › Sobre e apoio), o sistema (macOS, Linux ou Windows) e os passos para
reproduzir no cofre de demonstração.

## Preparar o ambiente

- Rust estável (1.89 ou mais novo) e Node.js 22.
- macOS: Xcode Command Line Tools. Linux: as bibliotecas do WebKitGTK listadas em
  [.github/workflows/ci.yml](.github/workflows/ci.yml).
- Ligue o gancho que recusa documentos e dados pessoais nos commits:

  ```bash
  git config core.hooksPath .githooks
  ```

O guia completo (estrutura do código, como rodar a interface contra o núcleo real, testes ponta a
ponta e convenções) está em [docs/development.md](docs/development.md).

## Antes de abrir um pull request

1. Para mudanças grandes (tela nova, migração do banco, dependência nova), abra antes um issue
   e espere a resposta: o que não combina com a visão do projeto é recusado.
2. Rode o mesmo que o CI roda:

   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cd apps/desktop && npm run lint && npm run typecheck && npm test && npm run format:check
   ```

3. Testes e fixtures usam **dados inventados**: copie o formato de um documento real, nunca o
   conteúdo (veja "Dados pessoais" em [docs/development.md](docs/development.md#personal-data)).
4. Texto novo na interface ou no núcleo vira chave nos catálogos de `locales/`: escreva em pt-BR e
   inclua en e es (um teste confere que os três têm as mesmas chaves). As regras estão em
   [locales/GUIDE.md](locales/GUIDE.md).
5. Migrações do banco seguem as regras de [docs/development.md](docs/development.md): só para a
   frente, número maior que o da última publicada, nunca editar uma publicada.
6. Commits no imperativo, com o prefixo do módulo (`core:`, `ui:`, `mcp:`, `docs:`, `build:`).

## Traduções

Os textos ficam em `locales/<idioma>/*.ftl` (Fluent). As versões em inglês e espanhol começaram
como rascunho por IA, e a revisão de um falante nativo é uma das contribuições mais úteis. Siga o
tom de cada idioma descrito em [locales/GUIDE.md](locales/GUIDE.md) e mantenha os argumentos
(`{ $nome }`) iguais aos do pt-BR. Para um idioma novo, abra antes um issue com o modelo de
tradução.

## Licença e DCO

O Treasury é distribuído sob a [AGPL-3.0-only](LICENSE). Ao contribuir, você concorda que sua
contribuição seja distribuída sob a mesma licença.

Não há CLA. Pedimos o [Developer Certificate of Origin](https://developercertificate.org/) (DCO):
cada commit leva uma linha `Signed-off-by` com seu nome e e-mail, dizendo que você tem o direito
de enviar aquele código. O Git acrescenta a linha com `-s`:

```bash
git commit -s -m "ui: fix the statement total in the dark theme"
```

As mensagens de commit são em inglês: uma primeira linha com um resumo curto no imperativo e o
prefixo do módulo (`core:`, `ui:`, `mcp:`, `docs:`, `build:`), com até uns 72 caracteres, e, se
precisar, uma linha em branco e no máximo dois parágrafos curtos sobre o que mudou e por quê
([docs/development.md](docs/development.md#commits)).

Contribuições feitas com ajuda de IA são bem-vindas; quem assina o DCO responde pelo código
enviado. Não cole código, textos, ícones ou fontes de outro projeto sem conferir que a licença é
compatível com a AGPL-3.0 (a lista aceita está no `deny.toml`). Dependência nova é exceção: a
política está em [docs/development.md](docs/development.md#dependencies), o CI barra o nome que
não está em `scripts/deps-approved.txt`, e ela entra só com a aprovação do mantenedor e com a
licença em `THIRD_PARTY_LICENSES.md` (`python3 -I scripts/third-party-licenses.py`).

## O que o projeto não faz

- **Não opina sobre ativos.** Issues, discussões e exemplos não dizem se uma ação, fundo ou cripto
  é bom, barato ou caro, nem sugerem compra ou venda. O app calcula com os dados e os critérios de
  quem usa; perguntas sobre o que comprar ficam fora daqui.
- **Não usa nomes de filmes, séries ou marcas** em temas, textos ou easter eggs.
- **Não compara com produtos concorrentes pelo nome.**

## O que esperar

O Treasury é mantido por uma pessoa, o Knuppe, que decide o que entra. A revisão pode demorar
alguns dias, e pull requests pequenos e focados são revisados mais rápido. Todos os espaços do projeto seguem o
[Código de Conduta](CODE_OF_CONDUCT.md).
