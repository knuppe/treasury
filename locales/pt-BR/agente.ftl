# Tela Agente e peças do agente usadas também no painel lateral: conversa, caixa de
# mensagem, fila de aprovação, revisão das alterações, alertas, memória, procedimentos,
# histórico de ferramentas e ajustes dos motores de IA (docs/i18n-design.md). pt-BR é o idioma
# de origem.

## Tela e abas

agente-aba-conversa = Conversa
agente-aba-aprovacao = Aprovação
agente-aba-alertas = Alertas
agente-aba-ferramentas = Ferramentas
agente-aba-memoria = Memória
agente-aba-configurar = Configurar
agente-secoes = Seções do agente
agente-cmd-aba = Agente: { $aba }
agente-cmd-busca = agente,ia
agente-configurar-motores = Configurar motores de IA
agente-n-pendentes = ({ $n } pendentes)
agente-n-novos = ({ $n } novos)
agente-motores-ligados = { $n ->
        [1] { $n } motor ligado
       *[other] { $n } motores ligados
    }
agente-badge-propostas = { $n ->
        [1] { $n } proposta pendente
       *[other] { $n } propostas pendentes
    }
agente-badge-alertas = { $n ->
        [1] { $n } alerta novo
       *[other] { $n } alertas novos
    }

## Números, tempos e aspas

agente-tokens-mil = { NUMBER($n, maximumFractionDigits: 1) } mil
agente-tokens-mi = { NUMBER($n, maximumFractionDigits: 1) } mi
agente-menos-1-s = menos de 1 s
agente-segundos = { $s } s
agente-minutos = { $m } min
agente-minutos-segundos = { $m } min { $s } s
agente-aspas = “{ $texto }”
agente-lista-e = { $inicio } e { $fim }
agente-lista-e-mais = { $inicio } e mais { $n }
agente-sim = sim
agente-nao = não

## Tarefas, tipos de motor e estados

agente-tarefa-chat = Chat com o agente
agente-tarefa-categorizacao = Categorização
agente-tarefa-pdf = Leitura de PDF
agente-tarefa-fechamento = Fechamento mensal
agente-tarefa-alertas = Alertas
agente-tarefa-estudo = Estudo de ativos
agente-tipo-motor-api = API
agente-tipo-motor-cli = CLI local
agente-tipo-motor-local = Modelo local
agente-destino-api = dados enviados à API do provider
agente-destino-cli = dados passam pela CLI local (e o provider dela)
agente-destino-local = dados ficam neste computador
agente-nivel-reversivel = Reversível · lote
agente-nivel-sensivel = Sensível · item a item
agente-proposta-pendente = Pendente
agente-proposta-aprovada = Aprovada
agente-proposta-rejeitada = Rejeitada
agente-proposta-falhou = Falhou
agente-proposta-retirada = Retirada
agente-alerta-novo = Novo
agente-alerta-visto = Visto
agente-alerta-resolvido = Resolvido
agente-gravidade-info = Informação
agente-gravidade-atencao = Atenção
agente-gravidade-critico = Crítico
agente-job-andamento = Em andamento
agente-job-concluido = Concluído
agente-job-falhou = Falhou
agente-job-cancelado = Cancelado
agente-memoria-perfil = Perfil de investidor
agente-memoria-regra = Regra da família
agente-memoria-nota = Nota
agente-problema-desligado = motor desligado
agente-problema-sem-chave = sem API key cadastrada
agente-problema-sem-template = sem template de comando
agente-problema-sem-modelo = sem modelo definido

## Comandos de escrita propostos

agente-comando-criar-lancamento = Criar lançamento
agente-comando-alterar-lancamento = Alterar lançamento
agente-comando-categorizar = Categorizar lançamentos
agente-comando-dividir = Dividir lançamento
agente-comando-criar-transferencia = Criar transferência
agente-comando-apagar-lancamento = Apagar lançamento
agente-comando-apagar-feedback = Apagar feedback ao desenvolvedor
agente-comando-descartar-feedback = Descartar feedback ao desenvolvedor
agente-comando-criar-etiqueta = Criar etiqueta
agente-comando-criar-categoria = Criar categoria
agente-comando-alterar-categoria = Alterar categoria
agente-comando-cadastrar-instituicao = Cadastrar instituição
agente-comando-alterar-instituicao = Alterar instituição
agente-comando-cadastrar-conta = Cadastrar conta ou cartão
agente-comando-alterar-conta = Alterar conta
agente-comando-alterar-recorrencia = Alterar recorrência
agente-comando-alterar-ativo = Alterar ativo
agente-comando-cadastrar-bem = Cadastrar bem físico
agente-comando-alterar-bem = Alterar bem físico
agente-comando-avaliacao-bem = Registrar avaliação do bem
agente-comando-benfeitoria = Registrar benfeitoria
agente-comando-criar-regra = Criar regra
agente-comando-alterar-regra = Alterar regra
agente-comando-aplicar-regra = Aplicar regra
agente-comando-criar-recorrencia = Criar recorrência
agente-comando-definir-orcamento = Definir orçamento
agente-comando-copiar-orcamento = Copiar orçamento
agente-comando-criar-meta = Criar meta
agente-comando-alterar-meta = Alterar meta
agente-comando-cadastrar-ativo = Cadastrar ativo
agente-comando-acompanhar-ativo = Acompanhar ativo
agente-comando-registrar-operacao = Registrar operação
agente-comando-registrar-provento = Registrar provento
agente-comando-proventos-ticker = Registrar proventos pelo ticker
agente-comando-evento-corporativo = Registrar evento corporativo
agente-comando-importar-extrato = Importar extrato
agente-comando-importar-lidos = Importar lançamentos lidos pelo agente
agente-comando-investimentos-lote = Cadastrar investimentos em lote
agente-comando-importar-b3 = Importar da B3
agente-comando-gravar-memoria = Gravar na memória do gestor

## Assuntos dos procedimentos

agente-assunto-extrato = Extrato
agente-assunto-fatura = Fatura
agente-assunto-investimentos = Investimentos
agente-assunto-proventos = Proventos
agente-assunto-cotacoes = Cotações e indicadores
agente-assunto-imposto = Imposto de renda
agente-assunto-cadastro = Cadastro
agente-assunto-outro = Outro

## Tabelas do banco nas alterações

agente-tabela-lancamento = Lançamento
agente-tabela-divisao = Divisão de lançamento
agente-tabela-etiqueta-lancamento = Etiqueta do lançamento
agente-tabela-etiqueta = Etiqueta
agente-tabela-categoria = Categoria
agente-tabela-regra = Regra
agente-tabela-aplicacao-regra = Aplicação de regra
agente-tabela-recorrencia = Recorrência
agente-tabela-orcamento = Orçamento
agente-tabela-linha-orcamento = Linha do orçamento
agente-tabela-meta = Meta
agente-tabela-conta-meta = Conta da meta
agente-tabela-projeto = Projeto
agente-tabela-ativo = Ativo
agente-tabela-operacao = Operação
agente-tabela-provento = Provento
agente-tabela-evento-corporativo = Evento corporativo
agente-tabela-acompanhamento = Acompanhamento
agente-tabela-lote-importacao = Lote de importação
agente-tabela-memoria = Memória do gestor
agente-tabela-procedimento = Procedimento do agente
agente-tabela-feedback = Feedback ao desenvolvedor
agente-tabela-nota-feedback = Nota de feedback
agente-tabela-vinculo-feedback = Vínculo entre feedbacks
agente-tabela-conta = Conta
agente-tabela-membro-conta = Membro da conta
agente-tabela-cartao = Cartão de crédito
agente-tabela-instituicao = Instituição
agente-tabela-renda-fixa = Renda fixa
agente-tabela-bem = Bem físico
agente-tabela-dono-bem = Membro dono do bem
agente-tabela-custo-aquisicao = Custo de aquisição
agente-tabela-benfeitoria = Benfeitoria
agente-tabela-avaliacao-bem = Avaliação do bem
agente-tabela-financiamento-bem = Financiamento do bem

## Contagem por tabela (singular só para 1)

agente-n-lancamento = { $n ->
        [1] { $n } lançamento
       *[other] { $n } lançamentos
    }
agente-n-divisao = { $n ->
        [1] { $n } divisão de lançamento
       *[other] { $n } divisões de lançamento
    }
agente-n-etiqueta-lancamento = { $n ->
        [1] { $n } etiqueta do lançamento
       *[other] { $n } etiquetas do lançamento
    }
agente-n-etiqueta = { $n ->
        [1] { $n } etiqueta
       *[other] { $n } etiquetas
    }
agente-n-categoria = { $n ->
        [1] { $n } categoria
       *[other] { $n } categorias
    }
agente-n-regra = { $n ->
        [1] { $n } regra
       *[other] { $n } regras
    }
agente-n-aplicacao-regra = { $n ->
        [1] { $n } aplicação de regra
       *[other] { $n } aplicações de regra
    }
agente-n-recorrencia = { $n ->
        [1] { $n } recorrência
       *[other] { $n } recorrências
    }
agente-n-orcamento = { $n ->
        [1] { $n } orçamento
       *[other] { $n } orçamentos
    }
agente-n-linha-orcamento = { $n ->
        [1] { $n } linha do orçamento
       *[other] { $n } linhas do orçamento
    }
agente-n-meta = { $n ->
        [1] { $n } meta
       *[other] { $n } metas
    }
agente-n-conta-meta = { $n ->
        [1] { $n } conta da meta
       *[other] { $n } contas da meta
    }
agente-n-projeto = { $n ->
        [1] { $n } projeto
       *[other] { $n } projetos
    }
agente-n-ativo = { $n ->
        [1] { $n } ativo
       *[other] { $n } ativos
    }
agente-n-operacao = { $n ->
        [1] { $n } operação
       *[other] { $n } operações
    }
agente-n-provento = { $n ->
        [1] { $n } provento
       *[other] { $n } proventos
    }
agente-n-evento-corporativo = { $n ->
        [1] { $n } evento corporativo
       *[other] { $n } eventos corporativos
    }
agente-n-acompanhamento = { $n ->
        [1] { $n } acompanhamento
       *[other] { $n } acompanhamentos
    }
agente-n-lote-importacao = { $n ->
        [1] { $n } lote de importação
       *[other] { $n } lotes de importação
    }
agente-n-memoria = { $n ->
        [1] { $n } memória do gestor
       *[other] { $n } memórias do gestor
    }
agente-n-procedimento = { $n ->
        [1] { $n } procedimento do agente
       *[other] { $n } procedimentos do agente
    }
agente-n-feedback = { $n ->
        [1] { $n } feedback ao desenvolvedor
       *[other] { $n } feedbacks ao desenvolvedor
    }
agente-n-nota-feedback = { $n ->
        [1] { $n } nota de feedback
       *[other] { $n } notas de feedback
    }
agente-n-vinculo-feedback = { $n ->
        [1] { $n } vínculo entre feedbacks
       *[other] { $n } vínculos entre feedbacks
    }
agente-n-conta = { $n ->
        [1] { $n } conta
       *[other] { $n } contas
    }
agente-n-membro-conta = { $n ->
        [1] { $n } membro da conta
       *[other] { $n } membros da conta
    }
agente-n-cartao = { $n ->
        [1] { $n } cartão de crédito
       *[other] { $n } cartões de crédito
    }
agente-n-instituicao = { $n ->
        [1] { $n } instituição
       *[other] { $n } instituições
    }
agente-n-renda-fixa = { $n ->
        [1] { $n } renda fixa
       *[other] { $n } rendas fixas
    }
agente-n-bem = { $n ->
        [1] { $n } bem físico
       *[other] { $n } bens físicos
    }
agente-n-dono-bem = { $n ->
        [1] { $n } membro dono do bem
       *[other] { $n } membros donos do bem
    }
agente-n-custo-aquisicao = { $n ->
        [1] { $n } custo de aquisição
       *[other] { $n } custos de aquisição
    }
agente-n-benfeitoria = { $n ->
        [1] { $n } benfeitoria
       *[other] { $n } benfeitorias
    }
agente-n-avaliacao-bem = { $n ->
        [1] { $n } avaliação do bem
       *[other] { $n } avaliações do bem
    }
agente-n-financiamento-bem = { $n ->
        [1] { $n } financiamento do bem
       *[other] { $n } financiamentos do bem
    }
agente-n-alteracoes = { $n ->
        [1] { $n } alteração
       *[other] { $n } alterações
    }

## Resumo e frases do que muda

agente-resumo-cria = cria { $itens }
agente-resumo-altera = altera { $itens }
agente-resumo-apaga = apaga { $itens }
agente-frase-cria = Cria { $itens }: { $nomes }
agente-frase-apaga = Apaga { $itens }: { $nomes }
agente-frase-registra = Registra { $itens }: { $nomes }
agente-frase-grava = Grava { $itens }: { $nomes }
agente-frase-acompanha = Acompanha { $itens }: { $nomes }
agente-frase-inclui = Inclui { $itens }: { $nomes }
agente-frase-poe = Põe { $itens }: { $nomes }
agente-frase-altera = Altera { $campo } de { $alvo }
agente-frase-altera-para = Altera { $campo } de { $alvo } para { $para }
agente-frase-altera-de-para = Altera { $campo } de { $alvo }: { $de } → { $para }
agente-artigo-campo = o campo
agente-artigo-categoria = a categoria
agente-artigo-valor = o valor
agente-artigo-membro = o membro
agente-artigo-conta = a conta
agente-artigo-situacao = a situação
agente-artigo-tipo = o tipo
agente-artigo-nome = o nome
agente-artigo-descricao = a descrição
agente-artigo-favorecido = o favorecido
agente-artigo-notas = as notas
agente-artigo-data = a data
agente-artigo-ativacao = a ativação
agente-artigo-arquivamento = o arquivamento
agente-artigo-prioridade = a prioridade
agente-artigo-categoria-pai = a categoria-pai
agente-artigo-instituicao = a instituição
agente-artigo-essencial = o “essencial”
agente-artigo-grupo-gasto = o grupo de gasto
agente-artigo-condicoes = as condições
agente-artigo-acoes = as ações
agente-artigo-cor = a cor

## Leitura amigável das alterações: operações, títulos e valores

agente-op-criacao = Criação
agente-op-inclusao = Inclusão
agente-op-alteracao = Alteração
agente-op-exclusao = Exclusão
agente-lado-compra = Compra
agente-lado-venda = Venda
agente-sem-categoria = Sem categoria
agente-sem-membro = Sem membro
agente-sem-pai = Nenhuma (raiz)
agente-sem-descricao = sem descrição
agente-lancamento-sem-descricao = Lançamento sem descrição
agente-outro-lancamento = outro lançamento
agente-nao-encontrado = { $ref } (não encontrado)
agente-ref-categoria = categoria
agente-ref-conta = conta
agente-ref-membro = membro
agente-ref-instituicao = instituição
agente-ref-ativo = ativo
agente-ref-contrato = contrato
agente-ref-etiqueta = etiqueta
agente-ref-regra = regra
agente-ref-meta = meta
agente-ref-recorrencia = recorrência
agente-ref-lancamento = lançamento
agente-ref-bem = bem
agente-regra-categoria = categoria { $nome }
agente-regra-membro = membro { $nome }
agente-regra-etiquetas = etiquetas { $etiquetas }
agente-regra-favorecido = favorecido “{ $nome }”
agente-regra-tipo = tipo { $tipo }
agente-regra-nenhuma = nenhuma
agente-titulo-etiqueta-em = #{ $etiqueta } em { $lancamento }
agente-titulo-parte-de = Parte de { $lancamento }
agente-titulo-de = { $tipo } de { $ativo }
agente-titulo-meta = meta { $nome }
agente-operacao = Operação
agente-provento = Provento
agente-evento = Evento
agente-nota-gestor = Nota do gestor

## Campos das alterações

agente-campo-nome = Nome
agente-campo-nome-completo = Nome completo
agente-campo-nome-curto = Nome curto
agente-campo-tipo = Tipo
agente-campo-classe = Classe
agente-campo-moeda = Moeda
agente-campo-valor = Valor
agente-campo-data = Data
agente-campo-situacao = Situação
agente-campo-descricao = Descrição
agente-campo-favorecido = Favorecido
agente-campo-notas = Notas
agente-campo-nota = Nota
agente-campo-conteudo = Conteúdo
agente-campo-titulo = Título
agente-campo-categoria = Categoria
agente-campo-categoria-pai = Categoria-pai
agente-campo-membro = Membro
agente-campo-conta = Conta
agente-campo-conta-destino = Conta de destino
agente-campo-conta-debito = Conta de débito
agente-campo-conta-pagamento = Conta de pagamento
agente-campo-instituicao = Instituição
agente-campo-emissor = Emissor
agente-campo-ativo = Ativo
agente-campo-ativo-novo = Ativo novo
agente-campo-contrato = Contrato
agente-campo-etiqueta = Etiqueta
agente-campo-regra = Regra
agente-campo-meta = Meta
agente-campo-recorrencia = Recorrência
agente-campo-lancamento = Lançamento
agente-campo-transferencia = Transferência
agente-campo-fatura = Fatura
agente-campo-parcelamento = Parcelamento
agente-campo-parcela = Parcela
agente-campo-parcelas = Parcelas
agente-campo-orcamento = Orçamento
agente-campo-bem = Bem
agente-campo-arquivado = Arquivado
agente-campo-ativa = Ativa
agente-campo-ligada = Ligada
agente-campo-essencial = Essencial
agente-campo-grupo-gasto = Grupo de gasto
agente-campo-principal = Principal
agente-campo-patrimonio = Entra no patrimônio
agente-campo-todas-condicoes = Todas as condições
agente-campo-isento-ir = Isento de IR
agente-campo-day-trade = Day trade
agente-campo-fixado = Fixado
agente-campo-saldo-inicial = Saldo inicial
agente-campo-data-inicial = Data inicial
agente-campo-limite = Limite
agente-campo-dia-fechamento = Dia do fechamento
agente-campo-dia-vencimento = Dia do vencimento
agente-campo-total-informado = Total informado
agente-campo-valor-total = Valor total
agente-campo-data-compra = Data da compra
agente-campo-primeira-parcela = Primeira parcela
agente-campo-prioridade = Prioridade
agente-campo-condicoes = Condições
agente-campo-acoes = Ações
agente-campo-repeticao = Repetição
agente-campo-inicio = Início
agente-campo-fim = Fim
agente-campo-taxa = Taxa
agente-campo-taxa-ano = Taxa ao ano
agente-campo-taxa-mes = Taxa ao mês
agente-campo-indexador = Indexador
agente-campo-pct-indexador = % do indexador
agente-campo-liquidez = Liquidez
agente-campo-emissao = Emissão
agente-campo-vencimento = Vencimento
agente-campo-carencia = Carência até
agente-campo-ticker = Ticker
agente-campo-mercado = Mercado
agente-campo-pais = País
agente-campo-setor = Setor
agente-campo-segmento = Segmento
agente-campo-fonte-cotacao = Fonte da cotação
agente-campo-simbolo-fonte = Símbolo na fonte
agente-campo-operacao = Operação
agente-campo-quantidade = Quantidade
agente-campo-preco = Preço
agente-campo-preco-unitario = Preço unitário
agente-campo-custo-unitario = Custo unitário
agente-campo-custos = Custos
agente-campo-impostos = Impostos
agente-campo-cambio = Câmbio
agente-campo-nota-corretagem = Nota de corretagem
agente-campo-fator = Fator
agente-campo-data-com = Data-com
agente-campo-pagamento = Pagamento
agente-campo-bruto = Bruto
agente-campo-liquido = Líquido
agente-campo-ir-retido = IR retido
agente-campo-preco-alvo = Preço-alvo
agente-campo-alerta-acima = Alerta acima de
agente-campo-alerta-abaixo = Alerta abaixo de
agente-campo-valor-meta = Valor da meta
agente-campo-prazo = Prazo
agente-campo-valor-inicial = Valor inicial
agente-campo-aporte-mensal = Aporte mensal
agente-campo-valor-financiado = Valor financiado
agente-campo-prazo-meses = Prazo (meses)
agente-campo-sistema = Sistema
agente-campo-data-contrato = Data do contrato
agente-campo-numero-contrato = Número do contrato
agente-campo-primeiro-vencimento = Primeiro vencimento
agente-campo-cor = Cor
agente-campo-icone = Ícone
agente-campo-ordem = Ordem
agente-campo-mes = Mês
agente-campo-custo = Custo
agente-campo-origem = Origem
agente-campo-dominio = Domínio
agente-campo-rotulo = Rótulo
agente-campo-assunto = Assunto
agente-campo-passos = Passos
agente-campo-agencia = Agência

## Revisão das alterações e diff exato

agente-campo = Campo
agente-valor = Valor
agente-antes = Antes
agente-depois = Depois
agente-para = para
agente-nenhuma-linha-alteracao = Nenhuma linha do banco muda com esta alteração.
agente-nenhuma-linha-proposta = Nenhuma linha do banco muda com esta proposta.
agente-o-que-muda = O que muda
agente-mostrar-linhas = Mostrar as { $n } linhas
agente-ver-tecnicos = Ver detalhes técnicos
agente-ocultar-tecnicos = Ocultar detalhes técnicos
agente-diff-exato = Diff exato do que será gravado: tabelas, ids e valores brutos (dinheiro em centavos).
agente-so-o-que-muda = Só o que muda
agente-mostrar-campos = Mostrar todos os campos (+{ $n })

## Conversa

agente-motor-em-uso = Motor em uso
agente-proximo-motor = Próximo motor
agente-motor-em-uso-nome = Motor em uso: { $motor }
agente-proximo-motor-nome = Próximo motor: { $motor }
agente-nenhum-motor-dica = Nenhum motor pronto: configure um em Ajustes › IA e agente
agente-motor-recebe = Motor que recebe os dados
agente-respondendo = Respondendo
agente-recebendo = Recebendo
agente-nenhum-motor-pronto = nenhum motor pronto
agente-proposta-criada = Proposta criada: { $nome }
agente-aguarda-aprovacao = aguarda aprovação
agente-revisar-fila = Revisar na fila
agente-fallback = { $motor } não respondeu ({ $motivo }); tentando o próximo motor da rota.
agente-agora-com = Agora com { $motor } ({ $tipo }).
agente-uso-resposta = Uso desta resposta
agente-tokens-entrada = { $n } tokens de entrada
agente-tokens-saida = { $n } de saída
agente-custo = custo { $valor }
agente-custo-parcial = custo { $valor } (parcial: modelo sem preço)
agente-voce = Você
agente-motor-generico = motor
agente-alteracao = alteração
agente-sem-motor-dica = Cadastre um motor (API, CLI local ou modelo local) e defina a rota de chat em { $link }.
agente-sem-motor-titulo = Nenhum motor de IA configurado
agente-nao-respondeu = O agente não respondeu
agente-aguardando-motor = aguardando o motor
agente-resposta-cancelada = Resposta cancelada.
agente-conversas = Conversas
agente-nova = Nova
agente-nenhuma-conversa = Nenhuma conversa gravada.
agente-confirmar-custo = Confirmar custo
agente-custo-texto = O primeiro turno com **{ $motor }** deve custar cerca de **{ $custo }**, acima do aviso de { $limite } definido nos limites de gasto.
agente-enviar-mesmo-assim = Enviar mesmo assim
agente-automatico = Automático (rota de chat)
agente-conversa-com = Conversa com o agente
agente-historico-desligado = Histórico desligado: cada pergunta começa do zero e nada fica gravado.
agente-vazio-titulo = Pergunte sobre gastos, orçamento, dívidas ou a carteira.
agente-vazio-texto = O agente usa as ferramentas do Treasury; toda escrita espera a sua aprovação na fila. Recomendações de investimento são análises dos seus dados, não aconselhamento profissional.
agente-vazio-texto-autonomo = O agente usa as ferramentas do Treasury. Com o modo autônomo ligado, as escritas reversíveis valem na hora e podem ser desfeitas no histórico; as sensíveis esperam a sua aprovação. Recomendações de investimento são análises dos seus dados, não aconselhamento profissional.
agente-mensagem = Mensagem para o agente
agente-placeholder = Ex.: categorize os lançamentos sem categoria deste mês
agente-motor = Motor
agente-enter-envia = Enter envia · Shift+Enter quebra a linha
agente-enviar = Enviar
agente-enviar-assim-mesmo = Enviar assim mesmo
agente-erro-sem-job = O núcleo não devolveu o job da conversa.

## Atividade da resposta (raciocínio e ferramentas)

agente-pensando = Pensando…
agente-raciocinio = Raciocínio
agente-raciocinio-tempo = Raciocínio · { $tempo }
agente-consultando = Consultando { $ferramenta }…
agente-consultou = Consultou { $n ->
        [1] { $n } ferramenta
       *[other] { $n } ferramentas
    }
agente-consultou-erro = Consultou { $n ->
        [1] { $n } ferramenta
       *[other] { $n } ferramentas
    } · { $erros } com erro
agente-ferramenta-nome = Ferramenta { $nome }
agente-ok = ok
agente-erro = erro
agente-ferramenta-cli = Ferramenta da própria CLI, fora do Treasury
agente-atividade = Atividade do agente
agente-raciocinio-agente = Raciocínio do agente
agente-ferramentas-usadas = Ferramentas usadas

## Conversas gravadas

agente-conversa-meta = { $quando } · { $n } msg
agente-fixar = Fixar a conversa { $titulo }
agente-desafixar = Desafixar a conversa { $titulo }
agente-fixada-dica = Fixada: não entra na faxina automática
agente-fixar-dica = Fixar: a faxina automática não apaga
agente-apagar-conversa = Apagar a conversa { $titulo }
agente-nao-apagou = Não foi possível apagar
agente-apagar-todas = Apagar todas as conversas
agente-apagar-todas-texto = O histórico das conversas some do banco. Propostas, alertas e a memória do gestor continuam.
agente-apagar-historico = Apagar histórico
agente-apagar-tudo = Apagar tudo

## Computador

agente-computador-ligado-dica = Nesta conversa, CLIs locais de agentes podem ler arquivos da sua pasta pessoal quando você pedir. Terminal, escrita e web continuam bloqueados; motores de API e locais seguem só com o Treasury.
agente-computador-desligado-dica = O agente usa só as ferramentas do Treasury e não olha seus arquivos, apps ou navegador. Ligue para deixar uma CLI ler um arquivo que você indicar nesta conversa.
agente-computador-rotulo = Acesso ao computador nesta conversa
agente-computador-desligado = Acesso ao computador desligado
agente-computador-liberado = Leitura do computador liberada
agente-computador-volta = O agente volta a usar só o Treasury.
agente-computador = Computador
agente-computador-leitura = Computador: leitura liberada
agente-computador-bloqueado = Computador: bloqueado

## Esforço de raciocínio

agente-esforco-padrao = Padrão
agente-esforco-baixo = Baixo
agente-esforco-medio = Médio
agente-esforco-alto = Alto
agente-esforco-maximo = Máximo
agente-esforco-padrao-dica = o que o motor usa
agente-esforco-baixo-dica = mais rápido
agente-esforco-medio-dica = equilíbrio
agente-esforco-alto-dica = pensa mais
agente-esforco-maximo-dica = mais lento e caro
agente-esforco-dica = Quanto o modelo raciocina antes de responder nesta conversa. Mais esforço costuma dar respostas melhores em perguntas difíceis, com mais demora e custo.
agente-esforco-sem-controle = { $motor } não tem controle de esforço com este modelo; vale o padrão dele.
agente-este-motor = Este motor
agente-esforco-rotulo = Esforço de raciocínio nesta conversa
agente-esforco-valor = Esforço: { $nivel }

## Anexos

agente-anexo-colado = colado.png
agente-anexos-a-mais = { $n } arquivo(s) a mais
agente-anexos-maximo = no máximo { $max } anexos por mensagem
agente-anexo-grande = arquivo grande demais (máx. 20 MB)
agente-anexo-paginas = { $n } pág.
agente-anexo-caracteres = { $n } caracteres
agente-anexo-escaneado = sem texto (escaneado)
agente-anexo-cortado = texto cortado
agente-anexar-dica = Anexar imagens, PDFs, planilhas ou textos. Também dá para arrastar arquivos para cá ou colar com ⌘V / Ctrl+V.
agente-anexar-arquivos = Anexar arquivos
agente-anexar = Anexar
agente-remover = Remover { $nome }
agente-anexos-mensagem = Anexos da mensagem
agente-anexo-preparando = preparando…
agente-falhou = falhou
agente-anexo-nao-lido = o motor não lê este anexo
agente-anexo-motor-nao-le = { $motor } não lê { $tipo ->
        [imagem] imagens
       *[pdf] PDFs escaneados
    }. Se enviar assim, o agente recebe só o nome do arquivo. Escolha um motor com visão (de API ou um modelo local com visão) ou remova o anexo.
agente-anexo-rota-nao-le = Nenhum motor da rota de chat lê { $tipo ->
        [imagem] imagens
       *[pdf] PDFs escaneados
    } desse tipo. Se enviar assim, o agente recebe só o nome do arquivo. Escolha um motor com visão (de API ou um modelo local com visão) ou remova o anexo.
agente-anexos = Anexos
agente-anexo-nao-guardado = { $detalhe } · não guardado
agente-anexo-guardado-dica = Guardado no cofre conforme a retenção de anexos.
agente-anexo-nao-guardado-dica = O conteúdo foi enviado ao motor e não ficou guardado (retenção de anexos).
agente-solte = Solte para anexar

## Fila de aprovação

agente-retiradas = { $n ->
        [1] { $n } proposta já tinha saído da fila
       *[other] { $n } propostas já tinham saído da fila
    }
agente-retiradas-texto = Os mesmos dados mudaram antes da aprovação; a mudança que você fez fica.
agente-aprovadas = { $n ->
        [1] { $n } proposta aprovada
       *[other] { $n } propostas aprovadas
    }
agente-rejeitadas = { $n ->
        [1] { $n } proposta rejeitada
       *[other] { $n } propostas rejeitadas
    }
agente-aprovadas-texto = As alterações valem agora e podem ser desfeitas.
agente-falha-aplicar = Uma proposta não pôde ser aplicada
agente-dados-mudaram = Os dados mudaram depois da proposta.
agente-proposta-rotulo = Proposta: { $nome }
agente-sensivel-dica = Escrita sensível: aprovação item a item
agente-selecionar-lote = Selecionar para aprovar em lote: { $nome }
agente-rejeitar = Rejeitar
agente-aprovar = Aprovar
agente-aprovar-item = Aprovar este item
agente-carregando-diff = Carregando o diff
agente-erro-diff = Não foi possível carregar o diff
agente-revisadas = Revisadas recentemente
agente-desfeita = Alteração desfeita
agente-nao-desfez = Não foi possível desfazer
agente-nao-aprovou = Não foi possível aprovar
agente-nao-rejeitou = Não foi possível rejeitar
agente-autonomo = Modo autônomo
agente-autonomo-ligado = Modo autônomo ligado: escritas reversíveis da IA são aplicadas sem perguntar
agente-autonomo-desligado = Modo autônomo desligado: toda escrita da IA espera a sua aprovação
agente-alterar-configurar = Alterar em Configurar
agente-sensiveis-texto = Escritas sensíveis (operações de investimento, importações, exclusões) sempre pedem aprovação item a item. Tudo o que for aprovado pode ser desfeito.
agente-pendentes = Pendentes
agente-pendentes-n = Pendentes · { $n }
agente-selecionar-reversiveis = Selecionar reversíveis
agente-rejeitar-selecionadas = Rejeitar selecionadas
agente-aprovar-selecionadas = Aprovar selecionadas ({ $n })
agente-fila-vazia = Nenhuma proposta esperando aprovação.

## Alertas e rotinas

agente-filtro-abertos = Abertos (novos e vistos)
agente-filtro-novos = Novos
agente-filtro-vistos = Vistos
agente-filtro-resolvidos = Resolvidos
agente-nao-atualizou = Não foi possível atualizar
agente-alerta-rotulo = Alerta: { $titulo }
agente-alerta-abrir-asset = Abrir ativo
agente-alerta-abrir-account = Abrir conta
agente-alerta-abrir-budget = Abrir orçamento
agente-alerta-abrir-transactions = Ver lançamentos
agente-alerta-abrir-card-statement = Ver fatura
agente-alerta-abrir-physical-asset = Abrir bem
agente-alerta-abrir-allocation = Abrir alocação
agente-alerta-abrir-reports = Abrir relatórios
agente-alerta-abrir-sources = Abrir fontes de dados
agente-alerta-abrir-asset-data = Ver dados dos ativos
agente-alerta-abrir-import = Importar extrato
agente-alerta-avancado = Avançado
agente-alerta-fontes = Fontes dos números (ferramentas do Treasury, para agentes): { $lista }
agente-marcar-visto = Marcar como visto
agente-resolver = Resolver
agente-reabrir = Reabrir
agente-apagar-alerta = Apagar o alerta { $titulo }
agente-rotinas = Rotinas do gestor
agente-rotina-ligada = ligada
agente-rotina-desligada = desligada
agente-rotina-com-motor = com motor de IA
agente-rotina-local = local e determinística (sem motor na rota)
agente-rotina-ultima = última: { $quando }
agente-rotina-nunca = nunca
agente-rotina-andamento = { $rotina }: em andamento
agente-nao-rodou = Não foi possível rodar
agente-rodando = Rodando…
agente-rodar-agora = Rodar agora
agente-caixa-alertas = Caixa de alertas
agente-marcar-todos-vistos = Marcar todos como vistos
agente-estado = Estado
agente-gravidade = Gravidade
agente-toda-gravidade = Toda gravidade
agente-tipo = Tipo
agente-todos-tipos = Todos os tipos
agente-sem-alertas = Nenhum alerta com estes filtros.

## Memória do gestor

agente-gravada-voce = gravada por você
agente-gravada-agente = gravada pelo agente (aprovada)
agente-esquecer = Esquecer
agente-esquecer-titulo = Esquecer esta nota
agente-esquecer-texto = O agente deixa de considerá-la em todas as conversas e rotinas. Dá para desfazer pelo histórico de alterações.
agente-nota-esquecida = Nota esquecida
agente-nao-esqueceu = Não foi possível esquecer
agente-gravado-memoria = Gravado na memória do gestor
agente-nao-gravou = Não foi possível gravar
agente-memoria-placeholder = Ex.: não sugerir cripto
agente-lembrar = Lembrar
agente-memoria-rodape = Vale para todos os motores. O agente grava notas novas só com a sua aprovação.
agente-memoria-vazia = A memória está vazia.

## Procedimentos aprendidos

agente-proc-atualizado = Procedimento atualizado
agente-proc-gravado = Procedimento gravado
agente-proc-ocultados = { $n } trecho(s) sensível(is) ocultado(s) (senhas, números de conta ou documentos).
agente-proc-editar = Editar procedimento
agente-proc-novo = Novo procedimento
agente-proc-dialogo-texto = Notas que o agente consulta antes de procurar de novo. Não escreva senhas, códigos nem números de conta ou documento.
agente-assunto = Assunto
agente-titulo = Título
agente-site = Site
agente-site-dica = Só o domínio, ex.: www.itau.com.br
agente-formato = Formato
agente-passos = Passos
agente-passos-dica = Menus, filtros, botão de exportar e o que o arquivo traz.
agente-proc-por-voce = Gravado por você em { $data }
agente-proc-pelo-agente = Gravado pelo agente em { $data }
agente-proc-pelo-cliente = Gravado pelo { $cliente } em { $data }
agente-proc-pelo-agente-cliente = Gravado pelo agente ({ $cliente }) em { $data }
agente-proc-rotulo = Procedimento: { $titulo }
agente-proc-funcionou-dica = Vezes em que o agente seguiu a nota e deu certo
agente-proc-funcionou = funcionou { $n }×
agente-proc-falhou-dica = Vezes em que a nota não funcionou
agente-proc-falhou = falhou { $n }×
agente-proc-arquivo = arquivo { $formato }
agente-proc-observacao = Última observação: { $nota }
agente-proc-conversa = Conversa: { $titulo }
agente-proc-ver-conversa = Ver a conversa
agente-proc-atualizado-em = atualizado { $quando }
agente-proc-ultimo-uso = último uso { $quando }
agente-proc-ultimo-uso-resultado = último uso { $quando } ({ $resultado ->
        [worked] funcionou
       *[failed] falhou
    })
agente-proc-apagar-titulo = Apagar este procedimento
agente-proc-apagar-texto = O agente deixa de usar esta nota. Dá para desfazer pelo histórico de alterações.
agente-proc-apagado = Procedimento apagado
agente-procs-titulo = Procedimentos aprendidos
agente-adicionar = Adicionar
agente-procs-rodape = O agente grava e atualiza estas notas sozinho, sem proposta: tudo fica no histórico de alterações e dá para desfazer. Elas vão junto com os pedidos relacionados como dados, nunca como instruções.
agente-buscar = Buscar
agente-procs-placeholder = Ex.: extrato itaú, b3, fatura
agente-todos-assuntos = Todos os assuntos
agente-procs-filtro-vazio = Nenhum procedimento com esse filtro.
agente-procs-vazio = O agente ainda não gravou procedimentos. Quando ele descobre onde baixar um extrato ou de onde vem um dado, a nota aparece aqui.
agente-procs = Procedimentos

## Histórico de ferramentas

agente-col-quando = Quando
agente-col-cliente = Cliente
agente-col-ferramenta = Ferramenta
agente-col-parametros = Parâmetros
agente-col-resultado = Resultado
agente-col-duracao = Duração
agente-col-inicio = Início
agente-col-tarefa = Tarefa
agente-ferramentas-titulo = Ferramentas usadas pela IA
agente-fonte = Fonte
agente-chamadas = Chamadas
agente-todos-clientes = Todos os clientes
agente-so-erros = Só chamadas com erro
agente-historico-ferramentas = Histórico de ferramentas
agente-nenhuma-chamada = Nenhuma chamada registrada.
agente-jobs-titulo = Jobs recentes do agente
agente-jobs = Jobs do agente
agente-jobs-vazio = Nenhum job nos últimos 30 minutos.

## Ajustes de IA

agente-ajustes-titulo = Motores de IA e roteamento
agente-ajustes-ia = Ajustes de IA
agente-nao-salvou = Não foi possível salvar
agente-opcoes-titulo = Ajustes do agente
agente-permissoes = Permissões
agente-autonomo-dica = Aplica sem perguntar as escritas reversíveis da IA (categorizar, criar regra, orçamento). Escritas sensíveis continuam pedindo aprovação item a item; tudo pode ser desfeito.
agente-mcp-escrita = Escrita por clientes MCP externos
agente-mcp-escrita-dica = Assistentes de IA e agentes de código conectados pelo MCP podem propor escritas (sempre como proposta na fila).
agente-guardar-historico = Guardar o histórico das conversas
agente-guardar-historico-dica = Sem histórico, cada pergunta começa do zero e nada da conversa fica no banco.
agente-rotinas-legenda = Rotinas (com o app aberto e desbloqueado)
agente-revisao-continua = Revisão contínua
agente-revisao-continua-dica = Ao importar extratos: categoriza, aponta duplicidades e sugere regras (como propostas).
agente-fechamento-dica = Relatório do mês com 3 a 5 recomendações, entregue como alerta.
agente-alertas-dica = Orçamento estourando, vencimentos, dívida cara, carteira desbalanceada.

## Roteamento por tarefa

agente-rota-chat-nota = Também vale para categorização e PDF quando elas não têm rota própria.
agente-rota-rotina-nota = Rotina em segundo plano: só usa os motores desta lista.
agente-rota-estudo-nota = Estudo com IA de um ativo. Sem motor aqui, usa a rota do chat. Só o pacote de dados do ativo vai ao motor.
agente-rota-categorizacao-nota = Sem rota própria, a revisão contínua roda local e determinística.
agente-nao-salvou-rota = Não foi possível salvar a rota
agente-rota = Rota: { $tarefa }
agente-sem-motor-rota = Sem motor nesta rota.
agente-motor-removido = motor removido
agente-subir = Subir { $motor }
agente-descer = Descer { $motor }
agente-tirar-rota = Tirar { $motor } da rota
agente-adicionar-motor-rota = Adicionar motor à rota { $tarefa }
agente-adicionar-fallback = Adicionar fallback…
agente-adicionar-motor-mais = Adicionar motor…
agente-roteamento = Roteamento por tarefa
agente-rotas = Rotas por tarefa

## Motores de IA

agente-respondeu-em = ✓ respondeu em { $tempo }
agente-motor-rotulo = Motor { $nome }
agente-pronto = pronto
agente-nao-alterou = Não foi possível alterar
agente-modelo = Modelo
agente-url-local = URL local
agente-endpoint = Endpoint
agente-api-key = API key
agente-chave-cadastrada = cadastrada
agente-chave-cadastrada-dica = cadastrada · { $dica }
agente-chave-nao-cadastrada = não cadastrada
agente-dados = Dados
agente-testar = Testar
agente-excluir = Excluir
agente-excluir-motor = Excluir { $nome }
agente-excluir-motor-texto = O motor sai das rotas de tarefa e a chave cadastrada é apagada. O histórico de uso continua.
agente-nao-excluiu = Não foi possível excluir
agente-motores-titulo = Motores de IA
agente-adicionar-motor = Adicionar motor
agente-motores-rodape = APIs cobram por token; CLIs locais usam a assinatura que você já paga; modelos locais não enviam nada para fora.
agente-motores-vazio = Nenhum motor cadastrado. Adicione uma API, uma CLI local ou um modelo local.
agente-motores = Motores
agente-procurando-clis = Procurando CLIs no PATH
agente-clis-encontradas = CLIs encontradas
agente-clis-detectadas = CLIs detectadas
agente-cli-instalada = instalada
agente-cli-login-ok = · login encontrado
agente-cli-sem-login = · sem login
agente-cli-login-incerto = · login: não dá para saber sem executar
agente-cli-nao-encontrada = não encontrada
agente-cli-historico-fora = Esta CLI guarda o histórico das conversas na pasta dela, fora do cofre. A CLI do Claude Code roda sem histórico.
agente-motor-cadastrado = Motor cadastrado
agente-motor-atualizado = Motor atualizado
agente-editar-motor = Editar motor · { $nome }
agente-novo-motor = Novo motor de IA
agente-cadastrar = Cadastrar
agente-provider = Provider
agente-exige-chave = { $tipo } · exige API key
agente-nome = Nome
agente-nome-dica = Como o motor aparece no app.
agente-modelo-dica = O nome dos modelos muda com o tempo; confira no provider.
agente-modelo-placeholder = ex.: llama3.2
agente-modelo-opcional = Modelo (opcional)
agente-modelo-opcional-dica = Vai no marcador { "{model}" } do template.
agente-url-servidor = URL do servidor local
agente-url-servidor-dica = http só para este computador (localhost/127.0.0.1).
agente-endpoint-opcional = Endpoint (opcional)
agente-endpoint-padrao = Padrão: { $url }
agente-endpoint-dica = Base da API compatível com OpenAI (https://…/v1).
agente-max-tokens = Máx. de tokens na resposta
agente-max-tokens-dica = Vazio: 4.096.
agente-max-tokens-erro = Use um número inteiro positivo.
agente-chave-dica = Gravada cifrada no cofre; nunca é mostrada de novo.
agente-cadastre-de-novo = cadastre de novo
agente-sem-chave = sem chave
agente-substituir-chave = Digite para substituir a chave
agente-cole-chave = Cole a chave do provider
agente-chave-apagada = Chave apagada
agente-apagar-chave = Apagar a chave cadastrada
agente-template = Template de comando
agente-template-dica = Marcadores: { "{prompt}" }, { "{mcp_config}" }, { "{model}" }, { "{workdir}" }. Trechos entre [ ] somem quando o marcador está vazio.
agente-template-placeholder = minha-cli -p { "{prompt}" }
agente-template-padrao = Vazio usa o padrão, com as ferramentas nativas da CLI desligadas: { $template }
agente-motor-ligado = Motor ligado

## Limites de gasto

agente-limites-salvos = Limites de { $provider } salvos
agente-limites-rotulo = Limites: { $provider }
agente-gasto = hoje { $hoje } · mês { $mes }
agente-por-dia = Por dia (USD)
agente-por-mes = Por mês (USD)
agente-confirmar-acima = Confirmar acima de (USD)
agente-valor-invalido = Valor inválido
agente-sem-limite = sem limite
agente-nunca-perguntar = nunca perguntar
agente-limites-titulo = Limites de gasto
agente-limites-rodape = Custos em dólar, estimados pela tabela de preços. Acima do limite o motor é pulado (fallback); acima do aviso, o chat pede confirmação antes de enviar.
agente-sem-provider = Nenhum provider de API em uso.
agente-limites-por-provider = Limites por provider
agente-limitar-outro = Limitar outro provider
agente-limitar-outro-mais = Limitar outro provider…

## Uso e custo

agente-periodo-1 = Este mês
agente-periodo-3 = Últimos 3 meses
agente-periodo-6 = Últimos 6 meses
agente-periodo-12 = Últimos 12 meses
agente-tokens-entrada-saida = Tokens entrada / saída
agente-tokens-entrada-saida-tooltip = Tokens (entrada / saída)
agente-col-custo = Custo
agente-erros = Erros
agente-sem-uso = Sem uso no período.
agente-uso-titulo = Uso e custo
agente-periodo = Período
agente-moeda = Moeda
agente-fonte-uso = ai_usage · tabela de preços
agente-custo-estimado = Custo estimado
agente-tokens = Tokens
agente-sem-preco = { "*" } { $n } chamada(s) sem preço cadastrado para o modelo: custo fora do total. Cadastre o preço na tabela abaixo.
agente-custo-mes = Custo estimado por mês
agente-custo-fonte = Fonte: chamadas registradas × tabela de preços (CLIs e modelos locais custam zero no app)
agente-custo-ia-mes = Custo estimado de IA por mês
agente-mes = Mês
agente-por = Por
agente-agrupar = Agrupar uso por
agente-tarefa = Tarefa
agente-uso-por-provider = Uso por provider
agente-uso-por-motor = Uso por motor
agente-uso-por-tarefa = Uso por tarefa

## Preços por modelo

agente-preco-salvo = Preço salvo
agente-preco-de = Preço · { $modelo }
agente-novo-preco = Novo preço
agente-modelo-prefixo = Modelo ou prefixo
agente-modelo-prefixo-dica = { "*" } = padrão do provider
agente-entrada-mtok = Entrada (USD / Mtok)
agente-saida-mtok = Saída (USD / Mtok)
agente-cache-mtok = Cache (USD / Mtok)
agente-opcional = Opcional
agente-entrada = Entrada
agente-saida = Saída
agente-cache = Cache
agente-origem = Origem
agente-editado = editado
agente-padrao = padrão
agente-acoes = Ações
agente-apagar-preco = Apagar o preço de { $modelo }
agente-precos-titulo = Preços por modelo · USD por milhão de tokens
agente-precos = Preços por modelo
agente-precos-vazio = Nenhum preço cadastrado: o custo das chamadas fica fora do total.

## Histórico e anexos no cofre

agente-ret-nenhum-dica = O arquivo fica só na memória do app enquanto o cofre está aberto (o agente o relê nos próximos turnos da conversa) e não vai para o cofre; a conversa guarda só o nome e o tamanho.
agente-ret-7-dica = Guarda os arquivos por 7 dias e depois apaga (a faxina diária cuida disso).
agente-ret-sempre-dica = Guarda os arquivos para sempre, até você apagar a conversa.
agente-n-dias = { $n } dias
agente-1-ano = 1 ano
agente-nunca = Nunca
agente-faxina-feita = Faxina feita
agente-faxina-texto = { $conversas } conversa(s) e { $anexos } anexo(s) vencidos apagados.
agente-faxina-texto-encolheu = { $conversas } conversa(s) e { $anexos } anexo(s) vencidos apagados; o cofre encolheu { $tamanho }.
agente-nao-limpou = Não foi possível limpar
agente-hist-titulo = Histórico e anexos no cofre
agente-anexos-enviados = Anexos enviados ao agente
agente-guardar-anexos = Guardar anexos no cofre
agente-nao-guardar = Não guardar
agente-7-dias = 7 dias
agente-sempre = Sempre
agente-recomendado = Recomendado: o cofre fica leve e nenhum documento enviado ao agente fica guardado.
agente-nao-recomendado = Não recomendado: imagens e PDFs deixam o banco cifrado grande (backups mais lentos e maiores) e mantêm cópias de documentos sensíveis. Prefira "Não guardar".
agente-nao-recomendado-hoje = Não recomendado: imagens e PDFs deixam o banco cifrado grande (backups mais lentos e maiores) e mantêm cópias de documentos sensíveis. Prefira "Não guardar". Hoje: { $tamanho } em anexos.
agente-apagar-sem-uso = Apagar conversas sem uso há
agente-faxina-diaria = A faxina roda uma vez por dia com o cofre aberto. Conversas fixadas ficam.
agente-faxina-nunca = As conversas ficam até você apagar. O cofre pode crescer com o tempo.
agente-retencao-conversas = Retenção das conversas
agente-conversas-fixadas = { $n } ({ $fixadas } fixadas)
agente-mensagens = Mensagens
agente-anexos-guardados = { $n } ({ $guardados } guardados)
agente-historico = Histórico
agente-cofre = Cofre
agente-cofre-livres = { $tamanho } ({ $livres } livres)
agente-ultima-faxina = Última faxina
agente-ainda-nao = ainda não
agente-limpar-agora = Limpar agora
agente-limpar-agora-dica = Aplica as regras acima já e compacta o cofre se sobrar espaço.
agente-conversas-abrir = Mostrar as conversas
agente-conversas-recolher = Recolher as conversas
agente-conversa-nova-titulo = Conversa nova
agente-exportar = Exportar
agente-exportar-salvar = Salvar a conversa
agente-exportar-md-dica = Salvar a conversa em Markdown (.md)
agente-exportar-pdf-dica = Salvar a conversa em PDF
agente-exportar-feito = Conversa exportada
agente-exportar-feito-texto = Salva em { $caminho }.
agente-exportar-feito-segredos = Salva em { $caminho }. { $n ->
    [one] 1 trecho com chave ou senha foi retirado.
   *[other] { $n } trechos com chave ou senha foram retirados.
}
agente-exportar-erro = Não deu para exportar a conversa
agente-exportar-md = Markdown
agente-mensagens-conversa = Mensagens da conversa

# Imagens com texto lido pelo OCR: o que vai ao motor.
agente-anexo-texto-lido = texto lido no computador
agente-imagem-modo-rotulo = O que mandar das imagens
agente-imagem-modo-pergunta = Texto lido no computador ({ $confianca }% de confiança). Mandar:
agente-imagem-modo-texto = Só o texto
agente-imagem-modo-imagem = Só a imagem
agente-imagem-modo-ambos = Os dois
agente-imagem-so-texto = Este motor não lê imagens: vai o texto lido no computador ({ $confianca }% de confiança).

## MCPs extras (só neste cofre)

agente-mcpx-titulo = MCPs extras (só neste cofre)
agente-mcpx-adicionar = Adicionar MCP
agente-mcpx-vazio = Nenhum MCP extra. Cadastre um servidor MCP de fora (por exemplo, um que controla o seu celular) para o agente deste cofre usar.
agente-mcpx-explica = Cada chamada de uma ferramenta pede a sua confirmação na conversa, a não ser que "Permitir sem perguntar" esteja ligado. Vale para todos os motores, inclusive os por CLI: as ferramentas chegam a eles pelo próprio MCP do Treasury.
agente-mcpx-seguranca = Um comando local roda neste computador com as permissões do seu usuário. Só adicione servidores em que você confia.
agente-mcpx-lista = MCPs extras deste cofre
agente-mcpx-linha = MCP { $nome }
agente-mcpx-tipo-stdio = Comando local
agente-mcpx-tipo-http = Endereço HTTP
agente-mcpx-ferramentas-prefixo = Ferramentas
agente-mcpx-permitir-sem-perguntar = Permitir sem perguntar
agente-mcpx-excluir = Excluir { $nome }
agente-mcpx-excluir-texto = O servidor sai do agente deste cofre e os valores secretos cadastrados para ele são apagados.
agente-mcpx-nao-excluiu = Não foi possível excluir o MCP
agente-mcpx-testando = Conectando… (pode levar até 20 s)
agente-mcpx-ferramentas-encontradas =
    { $n ->
        [one] 1 ferramenta encontrada
       *[other] { $n } ferramentas encontradas
    }
agente-mcpx-ferramentas = Ferramentas do MCP
agente-mcpx-sem-ferramentas = Conectou, mas o servidor não oferece ferramentas.
agente-mcpx-novo = Novo MCP extra
agente-mcpx-editar = Editar { $nome }
agente-mcpx-rotulo = Rótulo
agente-mcpx-rotulo-dica = Como aparece na conversa (ex.: Meu celular).
agente-mcpx-rotulo-erro = Informe um rótulo de até 60 caracteres.
agente-mcpx-nome = Nome
agente-mcpx-nome-dica = Prefixo das ferramentas ({ $exemplo }).
agente-mcpx-nome-erro = Use de 2 a 24 letras minúsculas, números ou _, começando por letra (e não "treasury").
agente-mcpx-tipo = Tipo
agente-mcpx-comando = Comando
agente-mcpx-comando-dica = O programa que inicia o servidor MCP (ex.: npx ou o caminho completo).
agente-mcpx-comando-erro = Informe o comando.
agente-mcpx-argumentos = Argumentos
agente-mcpx-argumentos-dica = Um por linha.
agente-mcpx-variaveis = Variáveis de ambiente
agente-mcpx-url = URL
agente-mcpx-url-dica = https://, ou http://localhost para um servidor neste computador.
agente-mcpx-url-erro = O endereço precisa começar com https:// (ou http://localhost).
agente-mcpx-cabecalhos = Cabeçalhos
agente-mcpx-cabecalhos-dica = Uma chave de acesso vai aqui, marcada como secreta: { $exemplo }
agente-mcpx-chave = Nome
agente-mcpx-valor = Valor
agente-mcpx-secreta = Secreta
agente-mcpx-manter-segredo = deixe em branco para manter
agente-mcpx-variavel-n = Variável { $n }
agente-mcpx-cabecalho-n = Cabeçalho { $n }
agente-mcpx-adicionar-variavel = Adicionar variável
agente-mcpx-adicionar-cabecalho = Adicionar cabeçalho
agente-mcpx-remover = Remover
agente-mcpx-ligado = Ligado (o agente deste cofre pode usar)
agente-mcpx-permitir-dica = O agente chama as ferramentas sem pedir confirmação.
agente-mcpx-salvo = MCP salvo
agente-mcpx-pedido-rotulo = Confirmação de ferramenta externa
agente-mcpx-pedido = O agente quer usar { $ferramenta } do MCP { $servidor }
agente-mcpx-parametros = Parâmetros
agente-mcpx-ver-tudo = Ver tudo
agente-mcpx-recolher = Recolher
agente-mcpx-permitir = Permitir
agente-mcpx-recusar = Recusar
agente-mcpx-permitida = Permitida
agente-mcpx-recusada = Recusada
agente-mcpx-aguardando = Aguardando a sua resposta
agente-mcpx-nao-respondeu = Não foi possível responder ao pedido

## Abas de Ajustes › IA

agente-ajustes-abas = Partes dos ajustes de IA
agente-ajustes-aba-motores = Motores
agente-ajustes-aba-motores-dica = Quais motores de IA o Treasury usa e qual atende cada tarefa, com o reserva quando um falha.
agente-ajustes-aba-acesso = Acesso do agente
agente-ajustes-aba-acesso-dica = O que o agente pode fazer sozinho, o que ele vê e quais ferramentas de fora ficam ao alcance dele.
agente-ajustes-aba-custos = Custos
agente-ajustes-aba-custos-dica = Quanto a IA custou, os limites de gasto e o preço de cada modelo.
agente-ajustes-aba-historico = Histórico
agente-ajustes-aba-historico-dica = As conversas e os anexos guardados no cofre, e por quanto tempo.

# MCPs das CLIs do usuário (Ajustes › IA)
agente-climcp-titulo = MCPs das suas CLIs
agente-climcp-explica = MCPs que você já tem no Claude, no Cursor, no Grok, no Codex, no Gemini ou no OpenCode. Por padrão, nenhum entra nas conversas do agente deste cofre: só o Treasury e os MCPs extras. Ligue só os que o agente precisa.
agente-climcp-seguranca = Um MCP liberado aqui é chamado pela CLI direto, sem a confirmação nem a auditoria do Treasury.
agente-climcp-vazio = Nenhuma CLI instalada carrega outros MCPs.
agente-climcp-lista = MCPs das CLIs
agente-climcp-nao-encontrado = não encontrado nas CLIs agora
agente-climcp-liberar = Usar { $nome } nas conversas
agente-mcpx-permitir-conversa = Permitir nesta conversa
agente-mcpx-permitir-conversa-dica = Permite todas as ferramentas do MCP { $servidor } até o fim desta conversa, sem perguntar de novo. Vale até bloquear o cofre; cada chamada segue na auditoria.
agente-mcpx-permitida-conversa = Permitida nesta conversa ({ $servidor })
agente-tempo-cli = Tempo-limite das respostas por CLI
agente-tempo-cli-dica = Coletas longas (MCPs extras, aparelho) precisam de mais tempo. Perto do fim, o agente é avisado para guardar o que coletou; se o tempo acabar, a parte feita fica na conversa.
agente-tempo-cli-minutos = { $n } min

## Pergunta do agente na conversa (ask_user)

agente-pergunta-rotulo = Pergunta do agente
agente-pergunta-uma = Escolha uma opção.
agente-pergunta-varias = Escolha uma ou mais opções.
agente-pergunta-recomendado = Recomendado
agente-pergunta-outro = Outro
agente-pergunta-outro-texto = A sua resposta
agente-pergunta-outro-dica = Escreva a sua resposta
agente-pergunta-enviar = Enviar
agente-pergunta-aguardando = O agente aguarda a sua resposta.
agente-pergunta-respondeu = Você respondeu: { $resposta }
agente-pergunta-sem-resposta = Sem resposta
agente-pergunta-nao-enviou = Não foi possível enviar a resposta
