# Termos de uso (docs/terms.md): a tela "Antes de começar", o cartão em Ajustes e os avisos das
# telas. O texto completo fica em conteudo/<idioma>/termos.md. pt-BR é o idioma de origem.

termos-titulo = Antes de começar
termos-intro = O Treasury é um programa **gratuito e de código aberto**, feito por uma pessoa só, com a ajuda de quem quiser contribuir. Ele organiza as finanças da sua família **no seu computador**. Leia estes pontos com calma:
termos-ponto-dados = Seus dados ficam com você.
termos-ponto-dados-texto = Tudo fica guardado e criptografado neste computador. O Treasury não envia nada para mim nem para ninguém, e não tem rastreamento.
termos-ponto-senha = Se você esquecer a senha do cofre, os dados não podem ser recuperados.
termos-ponto-senha-texto = Nem eu consigo abrir. Guarde a senha em lugar seguro e faça backups.
termos-ponto-impostos = Impostos são só estimativas.
termos-ponto-impostos-texto = O Treasury não substitui a autoridade fiscal do seu país, os programas oficiais de declaração nem um contador. Confira sempre nos canais oficiais do governo antes de declarar ou pagar.
termos-ponto-investimentos = Investimentos: não é recomendação.
termos-ponto-investimentos-texto = Notas, carteira, simulações e "e se" são ferramentas de estudo. Não são conselho para comprar ou vender nada. Decisões de investimento são suas.
termos-ponto-erros = Pode haver erros.
termos-ponto-erros-texto = Dados de fontes externas podem atrasar ou vir errados, importações podem ler algo errado e a inteligência artificial pode se enganar. Confira os números importantes.
termos-ponto-nuvem = Se você usar IA na nuvem,
termos-ponto-nuvem-texto = o que você enviar para ela vai para a empresa que você escolheu (por exemplo, Anthropic, OpenAI ou Google), seguindo as regras dela.
termos-ponto-risco = Uso por sua conta e risco.
termos-ponto-risco-texto = O programa é oferecido como está, sem garantia. Eu me esforço para que ele funcione bem, mas não me responsabilizo por perdas causadas pelo uso, por informações erradas ou por mau uso.
termos-ler-completos = Ler os termos completos
termos-ler-licenca = Ler a licença AGPL
termos-marcar = Li e entendi. Sei que o Treasury não é ferramenta oficial de impostos, não faz recomendação de investimentos e que uso o programa por minha conta e risco.
termos-sair = Sair
termos-o-que-mudou = O que mudou

## Aviso de versão nova e cartão em Ajustes

termos-de-uso = Termos de uso
termos-atualizados = Os termos de uso foram atualizados (versão { $versao }).
termos-ler = Ler
termos-entendi = Entendi
termos-versao = Versão { $versao }. O aceite fica só neste computador; nada é enviado.
termos-versao-aceita = Versão { $versao } · aceita neste computador em { $data } (versão { $aceita }). O aceite fica só neste computador; nada é enviado.
termos-historico = Histórico

## Avisos das telas ($pais: BR, US, PT ou outro)

termos-aviso-estudo = Ferramenta de estudo. Não é recomendação de investimento.
termos-aviso-imposto = Estimativa com base em { $fonte } de { $data }. Não substitui as ferramentas oficiais { $pais ->
        [BR] da Receita Federal
        [US] do IRS
        [PT] da Autoridade Tributária e Aduaneira
       *[outro] da autoridade fiscal do seu país
    }.
# Divergências conhecidas da estimativa (docs/tax-validation-countries.md), ao lado do aviso.
termos-aviso-imposto-limites-us = Não considera ainda: a parte reembolsável do crédito por filho e o EITC, e o imposto estadual. O estado civil vem do cadastro da família, como no formulário W-4: sem cônjuge e com dependentes, chefe de família (head of household); com cônjuge sem renda cadastrada, declaração conjunta; com os dois com renda, cada um pela tabela individual.
termos-aviso-imposto-limites-pt = Não considera ainda: o acréscimo por dependente até 6 anos, o IRS Jovem e a tributação conjunta. Conta com a dedução de despesas gerais familiares no limite da lei (250 €, ou 335 € na família monoparental), como a própria regra do mínimo de existência.
