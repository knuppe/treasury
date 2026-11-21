# Categorization rules (Rules tab of 04 · Transactions): list, rule dialog, test and applying
# rules to existing transactions (docs/i18n-design.md).
# Revision: AI draft, not yet reviewed by a native speaker.

## Apply rules

regras-n-alterados = { $n ->
        [one] { $n } transaction changed by the rules
       *[other] { $n } transactions changed by the rules
    }
regras-aplicar = Apply rules
regras-aplicar-descricao = First see what would change; nothing is saved until you confirm. Everything goes into a single history entry.
regras-aplicacao-desfeita = Rules undone
regras-desfazer = Undo
regras-ver-previa = Preview
regras-aplicar-a = Apply to { $n }
regras-so-sem-categoria = Uncategorized transactions only
regras-so-sem-categoria-descricao = When off, rules can replace categories you already chose.
regras-so-filtro = Only transactions in the current filter
regras-afetados = Affected transactions
regras-resultado-aplicado = Applied: { $casaram } matched · { $mudam } change
regras-resultado-previa = Preview: { $casaram } matched · { $mudam } change
regras-nada-mudar = Nothing to change.

## Rule list

regras-titulo = Categorization rules
regras-busca = rule,automatic
regras-tabela = Rules
regras-col-prioridade = Prior.
regras-col-regra = Rule
regras-entao = Then
regras-col-acertos = Hits
regras-acoes = Actions
regras-acao-categoria = category
regras-acao-tipo = type { $tipo }
regras-acao-renomear = rename to “{ $nome }”
regras-acao-membro = member
regras-ultimo = Last: { $data }
regras-erro-alterar = Could not change it
regras-desativar = Turn off
regras-ativar = Turn on
regras-editar = Edit { $nome }
regras-apagar-item = Delete { $nome }
regras-aplicar-reticencias = Apply rules…
regras-nova = New rule
regras-rodape-ativas = Active
regras-rodape-ordem = Order
regras-rodape-ordem-valor = lowest priority first; the first one that sets a field wins
regras-vazio-kicker = No rules
regras-vazio-titulo = Teach Treasury to categorize
regras-vazio-descricao = Rules fill in category, member and tags when you import. Create one from scratch or from a transaction (open the transaction and use “Create a rule from this one”).
regras-apagar = Delete rule
regras-apagar-descricao = The rule “{ $nome }” stops applying to future imports. Transactions already categorized do not change.
regras-apagada = Rule deleted

## Condition summary (`Description contains “uber” and Amount between 10.00 and 20.00`)

regras-condicao = { $campo } { $op } { $valor }
regras-condicao-entre = { $campo } { $op } { $valor } and { $valor2 }
regras-conta-escolhida = chosen account
regras-e = and
regras-ou = or

## Rule dialog

regras-campo-condicao = Field of condition { $n }
regras-operacao-condicao = Operator of condition { $n }
regras-valor-condicao = Amount of condition { $n }
regras-segundo-valor-condicao = Second amount of condition { $n }
regras-conta-condicao = Account of condition { $n }
regras-tipo-condicao = Type of condition { $n }
regras-texto-condicao = Text of condition { $n }
regras-tirar-condicao = Remove condition { $n }
regras-erro-nome = Give the rule a name.
regras-atualizada = Rule updated
regras-criada = Rule created
regras-salva-descricao = Applies to future imports; apply it to categorize what is already there.
regras-editar-titulo = Edit rule
regras-testar = Test
regras-salvar = Save rule
regras-criar = Create rule
regras-nome = Name
regras-nome-exemplo = Ride-hailing trips
regras-prioridade = Priority
regras-prioridade-dica = Lowest wins.
regras-quando = When
regras-combinar = Combine conditions
regras-todas = All conditions
regras-qualquer = Any of them
regras-adicionar-condicao = Add condition
regras-condicoes-nota = Text is compared ignoring case and accents. Amounts have a sign: expenses are negative.
regras-categoria = Category
regras-membro = Member
regras-nao-mudar = Don't change
regras-renomear = Rename to
regras-renomear-dica = Optional: the original description is kept in the notes.
regras-mudar-tipo = Change the type to
regras-tags = Tags
regras-resultado-teste = Test result
regras-teste-nenhum = No existing transaction matches this rule
regras-teste-n = { $n ->
        [one] { $n } transaction matches this rule
       *[other] { $n } transactions match this rule
    }
regras-teste-50 = 50 or more transactions match this rule
regras-sem-categoria = uncategorized

## Suggested rules (rules.learned, docs/learned-rules.md)

regras-sugeridas-titulo = Suggested rules
regras-sugeridas-explica = You have already given the same category three times to these merchants. With the rule, the next transactions come in with the category. Nothing is created unless you ask.
regras-sugeridas-ja = { $n } with this category
regras-sugeridas-faltam = { $n ->
    [one] 1 without a category
   *[other] { $n } without a category
}
regras-sugeridas-criar = Create rule
regras-sugeridas-ignorar = Do not suggest
regras-sugeridas-criada = Rule created
regras-sugeridas-aplicada = { $n ->
    [0] The next transactions come in with the category.
    [one] 1 transaction received the category.
   *[other] { $n } transactions received the category.
}
regras-sugeridas-erro = Could not create the rule
