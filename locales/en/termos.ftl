# Terms of use (docs/terms.md): the "Before you start" screen, the card in Settings and the
# screen notices. The full text is in conteudo/<language>/termos.md.
# Revision: AI draft, not yet reviewed by a native speaker.

termos-titulo = Before you start
termos-intro = Treasury is a **free and open source** program, made by one person with help from anyone who wants to contribute. It organizes your family's finances **on your computer**. Please read these points carefully:
termos-ponto-dados = Your data stays with you.
termos-ponto-dados-texto = Everything is stored encrypted on this computer. Treasury sends nothing to me or to anyone else, and has no tracking.
termos-ponto-senha = If you forget the vault password, the data cannot be recovered.
termos-ponto-senha-texto = Not even I can open it. Keep the password somewhere safe and make backups.
termos-ponto-impostos = Taxes are only estimates.
termos-ponto-impostos-texto = Treasury does not replace your country's tax authority, the official filing software or an accountant. Always check the government's official channels before filing or paying.
termos-ponto-investimentos = Investments: not a recommendation.
termos-ponto-investimentos-texto = Ratings, portfolio, simulations and "what if" are learning tools. They are not advice to buy or sell anything. Investment decisions are yours.
termos-ponto-erros = There may be errors.
termos-ponto-erros-texto = Data from external sources can be late or wrong, imports can misread something and artificial intelligence can make mistakes. Check the numbers that matter.
termos-ponto-nuvem = If you use cloud AI,
termos-ponto-nuvem-texto = what you send to it goes to the company you chose (for example, Anthropic, OpenAI or Google), under its rules.
termos-ponto-risco = Use at your own risk.
termos-ponto-risco-texto = The program is provided as is, without warranty. I work hard to make it work well, but I am not responsible for losses caused by its use, by wrong information or by misuse.
termos-ler-completos = Read the full terms
termos-ler-licenca = Read the AGPL license
termos-marcar = I have read and understood. I know Treasury is not an official tax tool, does not make investment recommendations, and that I use the program at my own risk.
termos-sair = Quit
termos-o-que-mudou = What changed

## New version notice and Settings card

termos-de-uso = Terms of use
termos-atualizados = The terms of use were updated (version { $versao }).
termos-ler = Read
termos-entendi = Got it
termos-versao = Version { $versao }. Acceptance is stored only on this computer; nothing is sent.
termos-versao-aceita = Version { $versao } · accepted on this computer on { $data } (version { $aceita }). Acceptance is stored only on this computer; nothing is sent.
termos-historico = History

## Screen notices ($pais: BR, US, PT or outro)

termos-aviso-estudo = A learning tool. Not an investment recommendation.
termos-aviso-imposto = Estimate based on { $fonte } as of { $data }. It does not replace the official tools of { $pais ->
        [BR] Brazil's Federal Revenue Service
        [US] the IRS
        [PT] Portugal's Tax and Customs Authority
       *[outro] your country's tax authority
    }.
# Known gaps in the estimate (docs/tax-validation-countries.md), next to the notice.
termos-aviso-imposto-limites-us = Not yet covered: the refundable part of the child tax credit and the EITC, and state income tax. Filing status comes from the family registry, as on Form W-4: no spouse and dependants, head of household; a spouse with no income registered, married filing jointly; both with income, each on the individual table.
termos-aviso-imposto-limites-pt = Not yet covered: the extra allowance for dependants up to age 6, IRS Jovem and joint taxation. It assumes the general family expenses deduction at its legal limit (€250, or €335 for single-parent families), as the minimum-subsistence rule itself does.
