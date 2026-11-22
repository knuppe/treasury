# Variante pt-PT (opcional) da importação da B3 e das notas de corretagem (só no Brasil): o que
# faltar cai para pt-BR. Revisão: rascunho por IA, ainda sem revisão de um falante nativo.

b3-status-nova-dica = será guardada
b3-status-duplicada-dica = já guardada antes: ignorada
b3-status-possivel-duplicada-dica = parecida com um movimento feito à mão: só é guardada se o pedir
b3-status-sem-conta-dica = a instituição não tem conta escolhida: não é guardada
b3-status-erro-dica = não é possível guardar
b3-regra-manual = Evento a registar à mão
b3-nota-nova-dica = será registada com os custos repartidos
b3-nota-sem-ativo-dica = ativo não identificado: escolha o ativo para guardar
b3-etapa-arquivos = Ficheiros
b3-importados-toast = { $n ->
    [one] { $n } movimento importado da B3
   *[other] { $n } movimentos importados da B3
}
b3-importar-n = Importar { $n } movimento(s)
b3-relatorios-dica = Negociação, movimentação e posição, em Excel (.xlsx) ou CSV. Envie os do mesmo período juntos: a posição confere o resultado. Os ficheiros ficam só na memória da aplicação até importar.
b3-lendo = A ler os relatórios…
b3-contas-texto = Escolha a corretora de cada instituição do ficheiro. A sugestão vem do nome; as linhas de instituições marcadas como “Não importar” ficam de fora.
b3-incluir-possiveis-dica = Mesma data, ativo e quantidade (ou valor) de um movimento feito à mão.
b3-lancar-caixa = Registar no saldo da corretora
b3-lancar-caixa-dica = Desligado (predefinição): a B3 não informa o saldo nem as comissões da corretora; importe o extrato da conta à parte. Ligado: compras, vendas e rendimentos movimentam o saldo.
b3-ativos-texto = { $n } ativo(s) a registar. A classe vem da folha da posição, do final do código ou do nome; confira os assinalados.
b3-cadastrar = Registar
b3-ja-cadastrado = Já registado
b3-usar-cadastrado = Usar ativo já registado
b3-ativo-cadastrado-para = Ativo registado para { $ativo }
b3-cadastrar-novo = Não: registar novo
b3-abas = Folhas
b3-eventos-revisao = Os eventos em revisão só são guardados se aprovados, com o fator (nova quantidade = antiga × fator; na bonificação, ações novas por ação, ex.: 0,1 = 10%).
b3-ignoradas-texto = Linhas que não viram movimento, com a regra que as deixou de fora.
b3-resultado = Importados da B3: { $operacoes } operação(ões), { $proventos } provento(s) e { $eventos } evento(s); { $ativos } ativo(s) registado(s). { $ignoradas } linha(s) ficaram de fora.
b3-backup-antes = Cópia de segurança antes
b3-backup-feito = feita
b3-custo-registro = Registo
b3-ativo-cadastrado = Ativo registado
b3-voltar-automatico = Voltar ao modo automático
b3-notas-gravadas = { $n } operação(ões) guardada(s)
b3-gravar-n = Guardar { $n } operação(ões)
b3-notas-pdf-dica = Uma ou mais notas no padrão SINACOR. Os ficheiros ficam só na memória da aplicação até guardar.
b3-arquivos-escolhidos = Ficheiros escolhidos
b3-lendo-notas = A ler as notas…
b3-arquivos = Ficheiros
b3-arquivos-nao-lidos = Ficheiros não lidos
b3-senha-pdf = Palavra-passe do PDF
b3-senha-pdf-dica = Muitas corretoras usam os 3 primeiros dígitos do CPF do titular. A palavra-passe só abre os ficheiros agora: não é guardada.
b3-abrir-senha = Abrir com a palavra-passe
b3-nada-gravado = Nada é guardado enquanto algum ficheiro não puder ser lido; retire o ficheiro da lista (Voltar) ou resolva o problema.
b3-nao-identificados-texto = A nota traz o nome do pregão, não o código. Escolha o ativo de cada título ou indique o código (é registado ao guardar); sem isso, as operações dele ficam de fora.
b3-inconsistencias = Inconsistências que não impedem guardar
b3-inconsistencia = { $data } · { $ativo }: { $aviso }. É guardada mesmo assim; confira depois na carteira.
b3-agrupar-dica = A corretora divide uma ordem em várias execuções. Ligado (predefinição): as do mesmo título, lado e mercado na nota passam a ser uma operação com o preço médio e os custos somados.
b3-gravar = Guardar
b3-gravar-operacao = Guardar operação { $n }
b3-cadastra = regista { $ativo } · { $nome }
b3-corrigido = corrigido por si
b3-notas-lancar-caixa-dica = Desligado (predefinição): o extrato da conta costuma ser importado à parte, e registar aqui duplicaria o saldo. Ligado: cada operação nova cria a saída ou entrada na conta.
b3-ativos-cadastrados = Ativos registados
