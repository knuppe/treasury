# Textos do histórico do cofre de demonstração (núcleo, demo/mod.rs): nomes de contas, descrições
# dos lançamentos e listas de lojas e empresas (separadas por vírgula). Ficam no idioma do app
# em que o cofre foi criado. pt-BR é o idioma de origem.

demodados-proposito = Demonstração: dados fictícios gerados pelo assistente
demodados-proposito-perfil = Demonstração ({ $perfil }): dados fictícios gerados pelo assistente
demodados-perfil-confortavel = classe média com folga
demodados-perfil-baixa-renda = baixa renda
demodados-perfil-dificuldade = família em dificuldade
demodados-perfil-sozinho = pessoa jovem morando só
demodados-perfil-aposentado = aposentadoria

## Contas

demodados-conta = Conta { $banco }
demodados-cartao = Cartão { $banco }
demodados-reserva = Reserva { $banco }
demodados-corretora = Corretora XP
demodados-corretora-instituicao = XP Investimentos
demodados-previdencia = Previdência { $banco }
demodados-fundo-aposentadoria = Fundo de aposentadoria { $banco }
demodados-ativo-nota = Ativo do cofre de demonstração: cotações fictícias.
demodados-meta-reserva = Reserva de emergência

## Renda

demodados-empresas = Acme Tecnologia, Grupo Horizonte, Hospital Santa Clara, Construtora Alvorada, Banco Meridiano, Escola Novo Saber
demodados-salario = Salário { $empresa }
demodados-aposentadoria = Aposentadoria INSS
demodados-13-primeira = 13º salário (1ª parcela)
demodados-13-segunda = 13º salário (2ª parcela)
demodados-subsidio-natal = Subsídio de Natal
demodados-subsidio-ferias = Subsídio de férias
demodados-clientes = Cliente A, Cliente B, Cliente C, Projeto
demodados-recebimento = Recebimento { $cliente }
demodados-aluguel-recebido = Aluguel recebido (apartamento)
demodados-beneficio = Bolsa Família

## Casa

demodados-aluguel = Aluguel
demodados-condominio = Condomínio
demodados-luz = Conta de luz
demodados-agua = Conta de água
demodados-internet = Internet fibra
demodados-celular = Plano de celular
demodados-multa-atraso = Multa e juros por atraso ({ $conta })

## Dia a dia

demodados-mercados = Supermercado Pão de Açúcar, Carrefour, Atacadão, Assaí
demodados-padaria = Padaria do bairro
demodados-postos = Posto Shell, Posto Ipiranga, Posto BR
demodados-estacionamento = Estacionamento
demodados-ipva = IPVA e licenciamento
demodados-seguro-carro = Seguro do carro
demodados-revisao = Revisão e troca de óleo
demodados-apps = Uber, 99
demodados-transporte-publico = Recarga do bilhete único
demodados-escola = Mensalidade escolar ({ $nome })
demodados-material = Material escolar ({ $nome })
demodados-plano-saude = Plano de saúde
demodados-farmacias = Drogasil, Drogasil, Pague Menos
demodados-restaurantes = Restaurante, Pizzaria, Churrascaria
demodados-delivery = iFood, iFood, Rappi
demodados-cinema = Cinemark, Ingresso Rápido, Teatro
demodados-passeios = Parque, Passeio, Museu
demodados-viagem = Viagem de férias
demodados-loja-roupa = Renner
demodados-loja-casa = Tok&Stok
demodados-loja-eletronicos = Amazon

## Dívidas e investimentos

demodados-financiamento-casa = Financiamento do apartamento
demodados-financiamento-carro = Financiamento do carro
demodados-emprestimo-pessoal = Empréstimo pessoal
demodados-parcela = Parcela { $contrato }
demodados-juros-rotativo = Juros do rotativo do cartão
demodados-juros-cheque-especial = Juros do cheque especial
demodados-rendimento-reserva = Rendimento da reserva
demodados-aporte-reserva = Aporte na reserva
demodados-aporte-corretora = Aporte na corretora
demodados-aporte-previdencia = Aporte na previdência

## Dia a dia (demo/daily.rs)

demodados-atacarejo = Atacadão (compra do mês)
demodados-mercado-bairro = Hortifrúti, Açougue, Feira livre, Mercadinho, Sacolão
demodados-lanches = Café, Lanchonete, Padaria (café da manhã), Cafeteria, Lanche
demodados-bares = Bar, Boteco, Choperia
demodados-streaming = Netflix, Spotify, Amazon Prime, Disney+, Globoplay, YouTube Premium, iCloud+, Max
demodados-pessoas = Cida (diarista), Marcos, Juliana, Tia Lúcia
demodados-envio-br = Pix para { $nome }
demodados-envio-pt = MB WAY para { $nome }
demodados-envio-us = Zelle para { $nome }
demodados-envio-outro = Transferência para { $nome }
demodados-recebido-br = Pix de { $nome }
demodados-recebido-pt = MB WAY de { $nome }
demodados-recebido-us = Zelle de { $nome }
demodados-recebido-outro = Transferência de { $nome }
demodados-tarifa-pacote = Tarifa do pacote de serviços
demodados-consulta = Consulta médica
demodados-doacao = Doação mensal (lar de idosos)
demodados-presente-neto = Presente dos netos
demodados-academia = Academia
demodados-pet-shop = Pet shop (ração)
demodados-veterinario = Veterinário
demodados-salao = Salão de beleza
demodados-gas = Gás de cozinha (botijão)
demodados-lojas-presente = Renner, Americanas, Livraria, Loja de brinquedos
demodados-compra-parcelada = Smart TV 55", Celular novo, Notebook, Geladeira
demodados-fgts = FGTS { $empresa }
demodados-fgts-deposito = Depósito FGTS (competência { $mes })
demodados-fgts-jam = Crédito de JAM { $coef }
demodados-fgts-distribuicao = Distribuição de resultado { $ano }
demodados-bem-imovel = Apartamento
demodados-bem-carro = Carro da família

## Espanha: lojas, corretoras e apps do país (iguais em todos os idiomas; demo/texts.rs)

demodados-corretora--es = Corredora MyInvestor
demodados-corretora-instituicao--es = MyInvestor
demodados-mercados--es = Mercadona, Carrefour, Lidl, Dia
demodados-postos--es = Repsol, Cepsa, BP
demodados-apps--es = Uber, Cabify
demodados-farmacias--es = Farmacia, Farmacia, Parafarmacia
demodados-delivery--es = Glovo, Just Eat, Uber Eats
demodados-cinema--es = Cines Yelmo, Entradas.com, Teatro
demodados-loja-roupa--es = Zara
demodados-loja-casa--es = IKEA
demodados-loja-eletronicos--es = MediaMarkt
demodados-atacarejo--es = Makro (compra del mes)
demodados-lojas-presente--es = El Corte Inglés, Fnac, Casa del Libro, Juguettos
