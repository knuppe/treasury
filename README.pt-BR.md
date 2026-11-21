# Treasury

> Finanças da família e investimentos, local-first, cifrado, com um agente de IA opcional.
> *Medido, nunca inventado: todo número tem origem rastreável.*

[English](README.md)

O Treasury é um aplicativo de computador para a família cuidar do dinheiro inteiro num lugar só:
contas, cartões, orçamento, dívidas, bens, investimentos e impostos. Os dados ficam no
computador, num banco cifrado, e um agente de IA opcional ajuda a analisar e a cadastrar, sempre
com aprovação.

## O que faz

- **Fluxo de caixa**: contas, cartões de crédito (faturas, adicionais, limite, custos, cashback e
  pontos), lançamentos com categorias, divisão, regras e recorrências, orçamento, metas e grupos
  de gasto, por membro da família.
- **Importação**: extratos OFX/CSV, faturas e notas de corretagem em PDF, relatórios da Área do
  Investidor B3, holerites, cupons fiscais (NFC-e e fotos lidas por OCR local), e-mail de ingestão
  e uma pasta de entrada. Tudo passa por prévia e deduplicação antes de gravar.
- **Dívidas e bens**: empréstimos e financiamentos (pré, CDI/Selic e corrigidos por índice),
  simulações de amortização, plano de quitação, imóveis e veículos com avaliações.
- **Investimentos**: carteira com preço médio, resultado realizado e não realizado, TWR e XIRR,
  resultado **real** (descontando a inflação do país, o CPI ou o M2), risco, concentração, livro de
  vendas, fundamentos, rankings, screener, o explorador do Mercado e um score de saúde por ativo.
- **Impostos**: IR de investimentos no Brasil (apuração mensal, DARF, relatório anual) e a
  estimativa de todos os impostos que a família pagou no ano.
- **Planejamento**: previsão de saldo, "quanto posso gastar hoje", assinaturas, cenários "e se",
  aposentadoria (Monte Carlo, com as regras públicas do Brasil, dos EUA e de Portugal) e o
  "próximo passo" para quem está começando.
- **Agente de IA e MCP**: o app expõe as suas funções como **servidor MCP** (`treasury mcp`). O
  agente do app e os clientes MCP externos usam as mesmas ferramentas; as escritas viram propostas
  que você aprova, com desfazer e auditoria. Motores: APIs, CLIs locais de agentes com a sua
  assinatura ou modelos locais por endpoint.
- **Países e idiomas**: completo para o Brasil; adaptado para EUA, Portugal, Espanha, Reino Unido,
  Canadá, França, Alemanha, Argentina e Chile; genérico nos outros. Interface em português do
  Brasil, inglês e espanhol (português de Portugal parcial).

## Segurança e privacidade

- Um arquivo SQLite + SQLCipher (AES-256) por cofre, com a chave derivada da senha mestra por
  Argon2id. Touch ID, PIN, chaves FIDO2 e recuperação dividida 2 de 3 são opcionais.
- Backups gerenciados, cifrados e verificados, com rotação e recuperação quando o cofre não abre.
- Sem telemetria. A internet só traz dados de mercado, índices e o motor de IA que você escolher,
  e cada fonte de dados pode ser desligada.
- O servidor MCP sempre exige uma chave de API criada por você, e clientes externos só leem ou
  propõem.

- **Só o mantenedor publica versões.** Os pacotes são compilados no GitHub Actions, com
  atestação pública de proveniência, e assinados no computador do mantenedor com uma chave
  física (YubiKey, toque e PIN). Todo Treasury instalado recusa atualização sem essa assinatura:
  uma conta do GitHub ou um CI invadidos não conseguem entregar uma versão. Por que e como
  conferir: [docs/trust.md](docs/trust.md).
- Vulnerabilidades conhecidas nas dependências (RustSec e npm) barram qualquer release.

Detalhes em [docs/security.md](docs/security.md) e [docs/local-security.md](docs/local-security.md).
Para relatar uma falha, veja o [SECURITY.md](SECURITY.md).

## Plataformas

| Sistema | Pacote | Situação |
|---|---|---|
| macOS (Apple Silicon) | `.dmg` | Uso diário; atualização automática |
| Linux (x86_64) | AppImage, `.deb`, `.rpm` | Gerado pelo CI; atualização automática; pré-visualização (ainda sem teste numa instalação real) |
| Windows (x64) | instalador NSIS, `.msi` | Gerado pelo CI; atualização automática; pré-visualização, ainda sem assinatura de código |

O app do macOS ainda não é notarizado: na primeira abertura, libere em Ajustes do Sistema ›
Privacidade e Segurança. Confira os downloads com o `SHA256SUMS.txt`
([docs/distribution.md](docs/distribution.md)).

## Compilar

Requisitos: Rust 1.92 ou mais novo, Node.js 22 e, no Linux, as bibliotecas do WebKitGTK listadas
em [.github/workflows/ci.yml](.github/workflows/ci.yml).

```bash
cargo test -p treasury-core
```

```bash
cd apps/desktop && npm ci && npm run tauri dev
```

O guia completo (estrutura do repositório, interface contra o núcleo real, testes ponta a ponta,
builds de release para macOS, Linux e Windows) está em [docs/development.md](docs/development.md).

## Documentação

O índice está em [docs/README.md](docs/README.md): visão do produto, decisões, arquitetura, cada
funcionalidade, fontes de dados e registros de validação. A documentação é escrita em inglês;
alguns documentos têm uma cópia em português ao lado (`*.pt-BR.md`).

## Como contribuir

Relatos de erro, ideias, traduções e código são bem-vindos, em português, inglês ou espanhol.
Nunca anexe extratos reais nem prints com valores: use o cofre de demonstração.

As versões são compiladas e assinadas exclusivamente pelo dono do projeto, o Knuppe, porque as
pessoas confiam a vida financeira ao Treasury. Contribuições que não combinam com a visão do
projeto podem ser recusadas sem discussão; abra um issue antes de começar algo grande. Veja o
[CONTRIBUTING.md](CONTRIBUTING.md) e o [Código de Conduta](CODE_OF_CONDUCT.md).

## Aviso

O Treasury é uma ferramenta para organizar e estudar os seus próprios dados. As análises, os
scores e as simulações não são recomendação de investimento nem orientação tributária ou jurídica.
Veja os termos de uso, mostrados na primeira abertura ([docs/terms.md](docs/terms.md)).

O Treasury é um projeto pessoal de código aberto. Não tem vínculo, endosso nem relação com nenhuma
empresa, banco, órgão de governo ou outro produto que use a palavra "treasury" no nome.

## Licença

[GNU AGPL-3.0-only](LICENSE). Contribuições entram sob a mesma licença, com a linha
`Signed-off-by` do DCO (`git commit -s`). Componentes de terceiros e dados embutidos:
[NOTICE](NOTICE) e [DATA_LICENSES.md](DATA_LICENSES.md).
