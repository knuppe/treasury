# Termos de uso

[English](terms.md)

Os termos de uso do Treasury: a tela de primeiro uso, o texto completo e onde os avisos aparecem
nas telas. Este arquivo é a fonte do texto em pt-BR (a versão em inglês do documento é
[terms.md](terms.md)); o app usa a mesma cópia em `locales/conteudo/pt-BR/termos.md` (um teste confere
que as duas são iguais), com as versões em pt-PT, inglês e espanhol ao lado
(`locales/conteudo/<idioma>/termos.md`).

- **Núcleo:** `crates/treasury-core/src/terms.rs`: a versão vigente (`CURRENT`), o histórico e o
  aceite desta instalação em `terms-accepted.json`, na pasta do app e fora do cofre (versão e
  data/hora). Nada é enviado. Comandos `terms.status` e `terms.accept`, que valem sem cofre;
  aceitar é só pela interface.
- **Interface:** `apps/desktop/src/features/terms/` (a porta "Antes de começar", o cartão em
  Ajustes › Sobre e apoio › Termos de uso, os avisos das telas e os textos).

## Quando pedir novo aceite

Os termos têm versão própria MAIOR.MENOR, independente do app.

- Correção de texto, link ou tradução sem mudar o sentido: 1.0 → 1.1. Nada na abertura; um aviso
  discreto ("Os termos de uso foram atualizados", com Ler e Entendi) e o histórico em Ajustes.
- Nova função que envia dados para fora (integração na nuvem, sincronização, atualizador que
  contata servidor): → 2.0, mostra a tela e pede aceite.
- Mudança nos limites de responsabilidade, na licença ou nas doações: → 2.0, pede aceite.
- Nova área de risco (relatórios PDF para contador, Open Finance, ordens em corretora): → 2.0,
  pede aceite.
- Novo país com regras próprias de impostos: 1.x → 1.y, aviso discreto, sem bloquear.

Ao abrir, se o MAIOR aceito for menor que o atual, a tela aparece com o "O que mudou" no topo
(`HISTORY` em `terms.rs`). Vale por instalação (usuário do sistema); "Ver como" não pede aceite.
O aceite não é copiado para dentro de cada cofre.

## Tela de primeiro uso

Aparece antes de criar ou abrir qualquer cofre (inclusive o de demonstração), logo depois da
escolha do idioma, no idioma escolhido (pt-BR, pt-PT, inglês ou espanhol). Sem caixa pré-marcada;
"Continuar" fica desabilitado até marcar; "Sair" fecha o app (comando `quit_app` do Tauri). Links
para o texto completo e para a licença AGPL (gnu.org, no navegador). Os textos curtos estão em
`locales/<idioma>/termos.ftl`.

## Onde os avisos aparecem

- **Impostos** e **IR** (Relatórios): rodapé "Estimativa com base em [fonte] de [data]. Não
  substitui as ferramentas oficiais [do órgão]": Receita Federal (Brasil), IRS (EUA), Autoridade
  Tributária e Aduaneira (Portugal) ou "da autoridade fiscal do seu país".
- **Score de ativos, Carteira, Aposentadoria e Cenários:** "Ferramenta de estudo. Não é
  recomendação de investimento."
- **Agente (painel):** "A IA pode errar. Confira antes de aprovar." sempre, e "Esta conversa será
  enviada para [provedor]." numa conversa nova com motor de nuvem (API ou CLI).
- **E-mail de ingestão (caixa IMAP):** "O Treasury vai ler as pastas que você escolher usando esta
  senha de app. A senha fica no cofre."
- **Criação do cofre:** a caixa "Entendi que, se esquecer a senha e não tiver o kit de
  recuperação, perco os dados." (o app tem o kit de recuperação, por isso a menção a ele).

## Lacunas do texto

- `[data da primeira publicação]`: 21/11/2026, a primeira versão pública.
- `[e-mail de contato do projeto]`: não há e-mail; o contato é pelo repositório no GitHub (issues,
  e o relato privado para o que não deve ser público).
- `[URL do repositório]` e `[URL de issues do repositório]`: `https://github.com/knuppe/treasury`
  e a página de issues dele.
- `[link]` da licença: o endereço oficial da AGPL na gnu.org.
- Licenças de terceiros: o arquivo `THIRD_PARTY_LICENSES.md` (gerado por
  `scripts/third-party-licenses.py`, conferido no CI e levado nos instaladores).
- `[Foro: a definir com advogado.]`: sem cláusula de foro. Os termos dizem só que vale a lei
  brasileira, sem tirar direitos da lei do país de quem usa.
- **Licença:** AGPL-3.0-only: o texto oficial da gnu.org em
  `LICENSE`, na raiz do repositório e nos instaladores (`bundle.licenseFile`), e o campo de
  licença do Rust (`Cargo.toml` do workspace), do npm e do Tauri.

## Texto completo (pt-BR)

<!-- texto completo: igual a locales/conteudo/pt-BR/termos.md -->

## Termos de Uso do Treasury

**Versão dos termos:** 1.0 · **Vigente desde:** 21/11/2026

**Responsável pelo projeto:** Knuppe · Contato: pelo GitHub · Código-fonte: https://github.com/knuppe/treasury

### 1. O que é o Treasury

1.1. O Treasury é um aplicativo de computador (macOS, Linux e Windows) para organizar as finanças de uma família, com um assistente de inteligência artificial opcional. É um projeto de **código aberto**, mantido por uma pessoa (Knuppe). Não há empresa, equipe de suporte nem serviço contratado por trás dele.

1.2. O Treasury é distribuído sob a **GNU Affero General Public License, versão 3 (AGPL-3.0)**, que garante a você o direito de usar, estudar, modificar e redistribuir o programa: https://www.gnu.org/licenses/agpl-3.0.html. Nada nestes termos restringe esses direitos; se algum trecho daqui parecer conflitar com a AGPL, vale a AGPL. Componentes e dados de terceiros seguem as próprias licenças (THIRD_PARTY_LICENSES.md e DATA_LICENSES.md, que acompanham o programa).

### 2. Uso por sua conta e risco

2.1. Conforme as seções 15 e 16 da AGPL, o Treasury é fornecido **"como está"**, **sem garantia de nenhum tipo**, expressa ou implícita, incluindo garantias de funcionamento, de ausência de erros, de adequação a uma finalidade específica ou de exatidão das informações.

2.2. Na máxima extensão permitida pela lei aplicável, **o responsável pelo projeto e os contribuidores não respondem por danos** de qualquer natureza decorrentes do uso ou da impossibilidade de uso do programa, incluindo perda de dados, perdas financeiras, decisões de investimento, tributos, multas ou juros, e informações incorretas vindas do programa, de fontes externas ou de IA. Esta limitação não afasta direitos que a lei do seu país não permita afastar.

2.3. Cabe a você guardar a senha do cofre, manter backups, conferir as informações importantes e usar o programa de acordo com a lei.

### 3. Não é conselho profissional

3.1. **Impostos.** Os cálculos de impostos são **estimativas** feitas com regras e tabelas públicas da data indicada. O Treasury não é programa oficial e não substitui a autoridade fiscal do seu país, os programas oficiais de declaração nem um contador. Confira nos canais oficiais antes de declarar ou pagar.

3.2. **Investimentos.** Notas de ativos, rankings, análise de carteira, projeções e simulações são ferramentas de estudo: **não são recomendação, oferta, análise ou consultoria de investimento**. O Treasury e seu responsável não são registrados como analista, consultor ou administrador de carteira em nenhum órgão regulador. Rentabilidade passada não garante resultado futuro, e a decisão de comprar, vender ou manter qualquer ativo é sua.

3.3. **Inteligência artificial.** Respostas de IA podem estar erradas, incompletas ou inventadas, mesmo quando parecem confiantes. Mudanças nos seus dados propostas pela IA só entram com a sua aprovação, e a decisão é sua.

3.4. **Dados e importações.** Fontes externas podem atrasar ou publicar números errados, a leitura automática de extratos, faturas e notas pode errar, e o suporte a países fora do Brasil é parcial. Confira o que for importante. O Treasury não movimenta dinheiro: não acessa sua conta bancária e não faz pagamentos, transferências nem ordens de compra e venda.

### 4. Seus dados

4.1. Os dados ficam num **cofre criptografado no seu computador**. O Treasury não tem telemetria, rastreamento nem anúncios, e o responsável pelo projeto não recebe, não vê e não guarda nenhum dado seu. Sem a senha, ninguém abre o cofre, nem o responsável pelo projeto.

4.2. O programa só usa a internet para funções que você usa: fontes públicas de dados, atualizações pelas releases do GitHub (Ajustes › Atualizações) e, se você ativar, um provedor de inteligência artificial ou uma caixa de e-mail. Nesses casos, o que é enviado segue as regras do serviço que você escolheu.

4.3. Ao cadastrar dados de outras pessoas da família, faça isso com o conhecimento delas.

### 5. Doações e nome

5.1. O Treasury é gratuito. Doações (GitHub Sponsors) são voluntárias, não dão direito a suporte, prazos ou funções novas e não são reembolsáveis.

5.2. A AGPL cobre o código, não o nome: uma versão modificada deve usar outro nome ou deixar claro que não é a versão oficial.

### 6. Mudanças nestes termos

Os termos podem mudar com novas versões do programa; a versão e a data ficam no topo. Quando a mudança for relevante, o Treasury mostra o novo texto ao abrir e pede uma nova confirmação.

### 7. Lei aplicável e contato

7.1. Estes termos são regidos pelas leis do Brasil, sem prejuízo dos direitos que a lei do país onde você mora garante.

7.2. Contato: pelo repositório no GitHub. Relatos de erro e sugestões: https://github.com/knuppe/treasury/issues · Assuntos que não devem ser públicos, inclusive falhas de segurança: relato privado em https://github.com/knuppe/treasury/security/advisories/new

_Knuppe_
