# Assistente do cofre de demonstração (docs/demo.md, docs/i18n-design.md). pt-BR é o idioma de
# origem. O argumento valor é um elemento (o valor em dinheiro) que a tela coloca no lugar.

## Valores iniciais e passos

demo-padrao-nome = Demonstração
demo-padrao-familia = Família Exemplo
demo-passo-familia = Família
demo-passo-renda = Renda
demo-passo-moradia = Moradia e despesas
demo-passo-contas = Contas e investimentos
demo-passo-dividas = Dívidas e criação
demo-titulo = Cofre de demonstração
demo-descricao = Conte como é a família e o app cria um cofre separado, com um histórico de exemplo para testar à vontade.
demo-passo = Passo { $n } de { $total }: { $passo }
demo-progresso = Progresso do assistente

## O que falta para seguir

demo-erro-nome = Dê um nome ao cofre.
demo-erro-familia = Informe o nome da família.
demo-erro-pessoa = Informe o nome de cada pessoa.
demo-erro-nomes-iguais = Cada pessoa precisa de um nome diferente.
demo-erro-renda = Informe a renda de ao menos um adulto.
demo-erro-financiamento = Informe a parcela do financiamento.
demo-erro-banco = Informe o banco da conta principal.
demo-erro-carro = Informe a parcela do carro e quantas faltam.
demo-criado = Cofre "{ $nome }" criado
demo-criado-dica = { $n } lançamentos de { $data } até hoje. Para voltar ao seu cofre, use Cofres.

## Passo 1: família

demo-nome-cofre = Nome do cofre
demo-nome-cofre-dica = Aparece na lista de cofres.
demo-nome-familia = Nome da família
demo-adultos = Adultos com renda
demo-um = Um
demo-dois = Dois
demo-nome-adulto = Nome do adulto { $n }
demo-filhos = Filhos
demo-nenhum = Nenhum
demo-nome-filho = Nome do filho { $n }
demo-historico = Histórico
demo-historico-dica = Quantos meses de lançamentos, até hoje.
demo-meses-historico = Meses de histórico
demo-meses = { $n } meses

## Passo 2: renda

demo-renda-de = Renda líquida de { $nome }
demo-adulto-n = adulto { $n }
demo-renda-dica = Por mês, o que cai na conta.
demo-renda-variavel = Renda variável
demo-renda-variavel-dica = Autônomo, comissões: muda de um mês para o outro, sem 13º.
demo-outras-rendas = Outras rendas
demo-outras-rendas-dica = Por mês: aluguel recebido, por exemplo.
demo-renda-total = Renda total por mês: { $valor }. Nos próximos passos, os valores vêm sugeridos por ela; ajuste à vontade.

## Passo 3: moradia e despesas

demo-moradia = Moradia
demo-aluguel = Aluguel
demo-financiamento = Financiamento
demo-propria = Própria quitada
demo-sugerir = Sugerir pela renda
demo-parcela-financiamento = Parcela do financiamento
demo-condominio = Condomínio
demo-contas-casa = Contas da casa
demo-contas-casa-dica = Luz, água, internet e celular.
demo-mercado = Mercado e padaria
demo-transporte = Transporte
demo-carro = Carro
demo-aplicativo = Aplicativo
demo-onibus = Ônibus e metrô
demo-nao-tem = Não tem
demo-combustivel = Combustível e estacionamento
demo-escola = Escola, por filho
demo-saude = Saúde
demo-saude-dica = Plano de saúde e farmácia.
demo-lazer = Lazer
demo-lazer-dica = Restaurantes, delivery, cinema e passeios.
demo-assinaturas = Assinaturas
demo-assinaturas-dica = Streaming e música.
demo-compras = Compras
demo-compras-dica = Roupas, casa e eletrônicos.

## Passo 4: contas e investimentos

demo-banco = Banco da conta principal
demo-cartao = Cartão de crédito
demo-cartao-dica = As compras do dia a dia vão no cartão e as faturas são pagas no vencimento.
demo-limite = Limite do cartão
demo-investimentos = Investimentos
demo-reserva = Reserva de emergência hoje
demo-reserva-mes = Para a reserva, por mês
demo-acoes = Ações (no início do histórico)
demo-fiis = Fundos imobiliários (no início)
demo-tesouro = Tesouro Direto (no início)
demo-aporte = Aporte na corretora, por mês
demo-previdencia = Previdência VGBL (no início)
demo-aporte-previdencia = Aporte na previdência, por mês
demo-cotacoes-ficticias = Ações e fundos com tickers reais, mas cotações fictícias (cotação manual): o exemplo não mistura preços reais com operações inventadas.

## Passo 5: dívidas e criação

demo-financiamento-carro = Financiamento do carro
demo-financiamento-carro-dica = Vira um contrato de dívida, com as parcelas pagas no histórico.
demo-parcela = Parcela
demo-parcelas-faltam = Parcelas que faltam
demo-financiamento-moradia = O financiamento da moradia também vira um contrato de dívida.
demo-resumo = Resumo do mês
demo-entra = Entra
demo-sai = Sai (gastos)
demo-investe = Investe
demo-vermelho = Gastos e aportes passam da renda: o exemplo vai mostrar meses no vermelho.
demo-nivel = Nível da interface no cofre de demonstração
demo-nivel-rotulo = Nível da interface
demo-troca = O cofre atual fecha (com backup) e o de demonstração abre. Para voltar, use Cofres: o seu cofre continua igual.
demo-criar = Criar cofre de demonstração

## Perfil e país (primeiro passo)

demo-passo-perfil = Perfil
demo-pais = País da família
demo-pais-dica = Dá a moeda, as categorias, os impostos e os ativos do exemplo.
demo-perfil = Perfil da família
demo-perfil-cofre = Demonstração · { $perfil }
demo-perfil-confortavel = Classe média com folga
demo-perfil-confortavel-dica = Casal com um filho, financiamento da casa, carro, reserva e investimentos.
demo-perfil-baixa-renda = Baixa renda
demo-perfil-baixa-renda-dica = Casal com dois filhos, aluguel, ônibus, renda apertada, bico e Bolsa Família.
demo-perfil-dificuldade = Família em dificuldade
demo-perfil-dificuldade-dica = Gasta mais do que ganha: cheque especial, rotativo do cartão, empréstimo e contas atrasadas.
demo-perfil-sozinho = Jovem morando só
demo-perfil-sozinho-dica = Uma pessoa, aluguel pequeno, aplicativo de transporte e os primeiros investimentos.
demo-perfil-aposentado = Aposentada
demo-perfil-aposentado-dica = Aposentadoria do INSS com 13º, casa própria, aluguel recebido, plano de saúde e reserva.
demo-perfil-custom = Montar do zero
demo-perfil-custom-dica = Você conta como é a família nos próximos passos.
demo-perfil-nota = Os valores vêm na moeda do país e dá para mudar tudo nos próximos passos.

## Rendas, dívidas e criação

demo-aposentado = Aposentado
demo-aposentado-dica = A renda é a aposentadoria (no Brasil, do INSS, com 13º).
demo-beneficio = Benefício social
demo-beneficio-dica = Por mês, como o Bolsa Família.
demo-emprestimo = Empréstimo pessoal
demo-emprestimo-dica = Vira um contrato em Dívidas, com as parcelas pagas pela conta.
demo-erro-emprestimo = Informe a parcela do empréstimo e quantas faltam.
demo-cheque-especial = Cheque especial
demo-cheque-especial-dica = A conta pode ficar negativa; no mês em que fecha no vermelho, vem juros no começo do seguinte.
demo-contas-atrasadas = Algumas contas pagas com atraso
demo-contas-atrasadas-dica = Uma em cada três contas da casa sai dez dias depois, com multa e juros.
demo-rotativo = Paga só parte da fatura
demo-rotativo-dica = Paga 70% de cada fatura; o resto fica no rotativo e o cartão cobra juros.
demo-sem-senha = O cofre de demonstração abre sem senha. Qualquer pessoa com o app consegue abrir: use só dados fictícios, nunca dados reais.

## Criação rápida

demo-rapido-descricao = Escolha o país e o perfil: o cofre de demonstração é criado e abre em seguida, sem senha.
demo-rapido-criar = Criar e abrir
demo-rapido-personalizar = Personalizar antes de criar
demo-idade = Idade
demo-idade-de = Idade de { $nome }
demo-idade-auto = Pelo perfil
demo-sexo-de = Sexo de { $nome }
demo-erro-idade = Confira as idades: adultos de 18 a 100 anos e filhos de 0 a 17.
