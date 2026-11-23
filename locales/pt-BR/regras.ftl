# Regras de categorização (aba Regras de 04 · Lançamentos): lista, diálogo da regra, teste e
# aplicação aos lançamentos existentes (docs/i18n-design.md). pt-BR é o idioma de origem.

## Aplicar regras

regras-n-alterados =
    { $n ->
        [one] { $n } lançamento alterado pelas regras
       *[other] { $n } lançamentos alterados pelas regras
    }
regras-aplicar = Aplicar regras
regras-aplicar-descricao = Primeiro veja o que mudaria; nada é gravado até você confirmar. Tudo fica num só item do histórico.
regras-aplicacao-desfeita = Aplicação desfeita
regras-desfazer = Desfazer
regras-ver-previa = Ver prévia
regras-aplicar-a = Aplicar a { $n }
regras-so-sem-categoria = Só lançamentos sem categoria
regras-so-sem-categoria-descricao = Desligado, as regras podem trocar categorias já escolhidas.
regras-so-filtro = Só os lançamentos do filtro atual
regras-afetados = Lançamentos afetados
regras-resultado-aplicado = Aplicado: { $casaram } casaram · { $mudam } mudam
regras-resultado-previa = Prévia: { $casaram } casaram · { $mudam } mudam
regras-nada-mudar = Nada para mudar.

## Lista de regras

regras-titulo = Regras de categorização
regras-busca = regra,automático
regras-tabela = Regras
regras-col-prioridade = Prior.
regras-col-regra = Regra
regras-entao = Então
regras-col-acertos = Acertos
regras-acoes = Ações
regras-acao-categoria = categoria
regras-acao-tipo = tipo { $tipo }
regras-acao-renomear = renomear para “{ $nome }”
regras-acao-membro = membro
regras-ultimo = Último: { $data }
regras-erro-alterar = Não foi possível alterar
regras-desativar = Desativar
regras-ativar = Ativar
regras-editar = Editar { $nome }
regras-apagar-item = Apagar { $nome }
regras-aplicar-reticencias = Aplicar regras…
regras-nova = Nova regra
regras-rodape-ativas = Ativas
regras-rodape-ordem = Ordem
regras-rodape-ordem-valor = menor prioridade primeiro; a primeira que define um campo vence
regras-vazio-kicker = Nenhuma regra
regras-vazio-titulo = Ensine o Treasury a categorizar
regras-vazio-descricao = Regras preenchem categoria, membro e tags nas importações. Crie uma do zero ou a partir de um lançamento (abra o lançamento e use “Criar regra a partir deste”).
regras-apagar = Apagar regra
regras-apagar-descricao = A regra “{ $nome }” para de valer nas próximas importações. Lançamentos já categorizados não mudam.
regras-apagada = Regra apagada

## Resumo das condições (`Descrição contém “uber” e Valor entre 10,00 e 20,00`)

regras-condicao = { $campo } { $op } { $valor }
regras-condicao-entre = { $campo } { $op } { $valor } e { $valor2 }
regras-condicao-termos = { $campo } contém { $termos }{ $mais ->
        [0] {""}
       *[other] {" "}e mais { $mais }
    }
regras-condicao-padrao = { $campo } segue um padrão (veja na edição)
regras-condicao-saida = é uma saída
regras-condicao-entrada = é uma entrada
regras-conta-escolhida = conta escolhida
regras-e = e
regras-ou = ou

## Diálogo da regra

regras-campo-condicao = Campo da condição { $n }
regras-operacao-condicao = Operação da condição { $n }
regras-valor-condicao = Valor da condição { $n }
regras-segundo-valor-condicao = Segundo valor da condição { $n }
regras-conta-condicao = Conta da condição { $n }
regras-tipo-condicao = Tipo da condição { $n }
regras-texto-condicao = Texto da condição { $n }
regras-tirar-condicao = Tirar a condição { $n }
regras-erro-nome = Dê um nome à regra.
regras-atualizada = Regra atualizada
regras-criada = Regra criada
regras-salva-descricao = Vale para as próximas importações; aplique para categorizar o que já existe.
regras-editar-titulo = Editar regra
regras-testar = Testar
regras-salvar = Salvar regra
regras-criar = Criar regra
regras-nome = Nome
regras-nome-exemplo = Corridas de aplicativo
regras-prioridade = Prioridade
regras-prioridade-dica = Menor vence.
regras-quando = Quando
regras-combinar = Combinar condições
regras-todas = Todas as condições
regras-qualquer = Qualquer uma
regras-adicionar-condicao = Adicionar condição
regras-condicoes-nota = Textos são comparados sem maiúsculas e sem acentos. Valores têm sinal: despesas são negativas.
regras-categoria = Categoria
regras-membro = Membro
regras-nao-mudar = Não mudar
regras-renomear = Renomear para
regras-renomear-dica = Opcional: a descrição original fica nas notas.
regras-mudar-tipo = Mudar o tipo para
regras-tags = Tags
regras-resultado-teste = Resultado do teste
regras-teste-nenhum = Nenhum lançamento existente casa com esta regra
regras-teste-n =
    { $n ->
        [one] { $n } lançamento casa com esta regra
       *[other] { $n } lançamentos casam com esta regra
    }
regras-teste-50 = 50 ou mais lançamento(s) casam com esta regra
regras-sem-categoria = sem categoria

## Regras sugeridas (rules.learned, docs/learned-rules.md)

regras-sugeridas-titulo = Regras sugeridas
regras-sugeridas-explica = Você já deu a mesma categoria três vezes a estes estabelecimentos. Com a regra, os próximos lançamentos entram com a categoria. Nada é criado sem você pedir.
regras-sugeridas-ja = { $n } com essa categoria
regras-sugeridas-faltam = { $n ->
    [one] 1 sem categoria
   *[other] { $n } sem categoria
}
regras-sugeridas-criar = Criar regra
regras-sugeridas-ignorar = Não sugerir
regras-sugeridas-criada = Regra criada
regras-sugeridas-aplicada = { $n ->
    [0] Os próximos lançamentos entram com a categoria.
    [one] 1 lançamento recebeu a categoria.
   *[other] { $n } lançamentos receberam a categoria.
}
regras-sugeridas-erro = Não foi possível criar a regra
