# Portfolio › import from the B3 (Brazilian stock exchange) Investor Area and from brokerage notes
# (SINACOR PDF): steps, status of each row, preview and result (Brazil only; docs/i18n-design.md).
# Official terms (B3, SINACOR, IRRF, ISS, CPF, JCP) stay as is.
# Revision: AI draft, not yet reviewed by a native speaker.

## Shared pieces

b3-etapas = Steps
b3-todas = All
b3-avisos = Warnings
b3-compra = Buy
b3-venda = Sell

## Reports, row status and rules

b3-relatorio-negociacao = Trading
b3-relatorio-movimentacao = Movements
b3-relatorio-posicao = Position
b3-status-nova = New
b3-status-nova-dica = will be saved
b3-status-duplicada = Duplicate
b3-status-duplicada-dica = already saved before: skipped
b3-status-possivel-duplicada = Possible duplicate
b3-status-possivel-duplicada-dica = similar to a transaction entered by hand: only saved if you ask
b3-status-sem-conta = No account
b3-status-sem-conta-dica = no account chosen for the institution: not saved
b3-status-revisar = Review
b3-status-revisar-dica = needs approval (e.g. an event without a computable factor)
b3-status-erro = Error
b3-status-erro-dica = can't be saved
b3-status-pendencia = Pending item
b3-status-pendencia-dica = imported anyway; the purchase is missing
b3-origem-liquidacao = Settlement (estimated date)
b3-origem-leilao = Fraction auction
b3-origem-subscricao = Exercised subscription
b3-origem-atualizacao = Custody update
b3-origem-abertura = Opening position
b3-regra-retrato = Custody snapshot
b3-regra-opcao = Option
b3-regra-termo = Forward
b3-regra-futuro = Future
b3-regra-mercado = Unsupported market
b3-regra-aluguel = Stock lending
b3-regra-custodia = Custody transfer
b3-regra-liquidacao = Settlement already imported
b3-regra-subscricao = Subscription right
b3-regra-saldo = Balance update
b3-regra-manual = Event to enter by hand
b3-regra-repassado = Passed-through income
b3-regra-estorno = Income reversal
b3-regra-sem-valor = No value
b3-regra-fracao = Fraction
b3-regra-desconhecida = Unknown movement
b3-regra-invalida = Invalid row
b3-posicao-confere = Matches
b3-posicao-diverge = Differs
b3-posicao-fora = Not in B3
b3-nota-nova-dica = will be added with the costs split
b3-nota-completar = Fill in costs
b3-nota-completar-dica = already exists without costs (e.g. from the B3 report): it gets the costs, the IRRF and the note number
b3-nota-duplicada-dica = already recorded with this note or with costs: skipped
b3-nota-sem-ativo = No asset
b3-nota-sem-ativo-dica = asset not identified: choose the asset to save

## Import from B3

b3-titulo = Import from B3
b3-etapa-arquivos = Files
b3-etapa-contas = Accounts
b3-etapa-previa = Preview
b3-etapa-resultado = Result
b3-periodo = { $de } to { $ate }
b3-origem-linha = { $aba } · row { $linha }
b3-importados-toast = { $n ->
    [one] { $n } transaction imported from B3
   *[other] { $n } transactions imported from B3
}
b3-nao-importar = Don't import
b3-sem-conta = no account
b3-importar-operacao = Import trade { $n }
b3-importar-provento = Import income { $n }
b3-importar-evento = Import event { $n }
b3-gerar-previa = Create preview
b3-importar-n = { $n ->
    [one] Import { $n } transaction
   *[other] Import { $n } transactions
}
b3-desfeita = B3 import undone
b3-desfazer = Undo import
b3-concluir = Done
b3-relatorios-area = Investor Area reports
b3-relatorios-dica = Trading, movements and position, as Excel (.xlsx) or CSV. Send the ones from the same period together: the position checks the result. The files stay only in the app's memory until you import.
b3-lendo = Reading the reports…
b3-relatorios-escolhidos = Chosen reports
b3-tirar = Remove { $arquivo }
b3-relatorios = Reports
b3-nenhum-reconhecido = none recognized
b3-periodo-rotulo = Period
b3-instituicoes = Institutions
b3-abas = Sheets
b3-aba-resumo = { $aba } ({ $tipo }, { $n ->
    [one] { $n } row
   *[other] { $n } rows
})
b3-aba-ignorada = skipped
b3-avisos-de = Warnings for { $arquivo }
b3-instituicao = Institution
b3-conta-sugerida = account suggested by the institution's name
b3-sem-conta-parecida = no similar account: choose one or leave it out
b3-contas-texto = Choose the account for each institution in the file. The suggestion comes from an earlier import, from the institution registered on the account or from the name; if there is no account, create one right here. Institutions marked "Don't import" are left out.
b3-motivo-escolhida = chosen by you
b3-motivo-salva = choice saved from the last import
b3-motivo-mesma-instituicao = same institution as the account
b3-motivo-nome = similar name
b3-motivo-criada = new account, created with the import
b3-conta-desconhecida = account not found
b3-criar-conta = Create a new account…
b3-tambem-como = also appears as { $nomes }
b3-conta-b3 = B3 account: { $contas }
b3-nova-conta-para = New account for { $instituicao }
b3-nova-conta-nome = Account name
b3-nova-conta-tipo = Type
b3-nova-conta-tipo-dica = brokerage for stocks, REITs and Tesouro; bank or credit union for the institution's own RDB and CDB
b3-nova-conta-instituicao = Institution
b3-nova-conta-abertura = Opening date
b3-resumo-conta = { $linhas } rows: { $operacoes } trades, { $proventos } incomes, { $eventos } events
b3-resumo-linhas = { $linhas } rows in the file
b3-resumo-ativos = Position before and after the import
b3-aba-abertura = Opening
b3-abertura-texto = The file starts after your history: these positions show in the income rows, the custody snapshots or the B3 position. Tick the ones to record the day before the file, with the average cost, and redo the preview.
b3-abertura-tabela = Suggested opening positions
b3-abertura-incluir = Include { $ativo }
b3-abertura-custo = Average cost
b3-abertura-custo-de = Average cost of { $ativo }
b3-abertura-evidencia = Where it comes from
b3-abertura-aplicar = Redo the preview with the ticked positions
b3-custo-operacao = Unit cost of trade { $n }
b3-custo = Cost
b3-linhas = Rows
b3-conta = Account
b3-conta-para = Account for { $instituicao }
b3-contas = Accounts
b3-incluir-possiveis = Also import the { $n } possible duplicates
b3-incluir-possiveis-dica = Same date, asset and quantity (or amount) as a transaction entered by hand.
b3-lancar-caixa = Record in the broker's cash
b3-lancar-caixa-dica = Off (default): B3 doesn't report the broker's cash or fees; import the account statement separately. On: buys, sells and investment income move the cash.
b3-secoes = Preview sections
b3-aba-ativos = Assets
b3-aba-operacoes = Trades
b3-aba-proventos = Income
b3-aba-eventos = Events
b3-aba-ignoradas = Skipped
b3-aba-posicao = Position
b3-ativos-texto = { $n ->
    [one] { $n } asset to add.
   *[other] { $n } assets to add.
} The class comes from the position sheet, the end of the ticker or the name; check the flagged ones.
b3-ativos-tabela = Assets in the import
b3-ativo = Asset
b3-situacao = Status
b3-cadastrar = Add
b3-ja-cadastrado = Already added
b3-conferir = Check
b3-classe = Class
b3-classe-de = Class of { $ativo }
b3-usar-cadastrado = Use an existing asset
b3-ativo-cadastrado-para = Existing asset for { $ativo }
b3-cadastrar-novo = No: add a new one
b3-por-que = Why
b3-filtrar-operacoes = Filter trades by status
b3-operacoes-tabela = Trades in the import
b3-importar = Import
b3-data = Date
b3-tipo = Type
b3-qtd = Qty.
b3-preco = Price
b3-valor = Amount
b3-filtrar-proventos = Filter income by status
b3-proventos-tabela = Income in the import
b3-pagamento = Payment
b3-bruto = Gross
b3-liquido = Net
b3-filtrar-eventos = Filter events by status
b3-eventos-tabela = Corporate events in the import
b3-aprovar = Approve
b3-evento = Event
b3-no-arquivo = In the file
b3-posicao-antes = Position before
b3-fator = Factor
b3-fator-do-evento = Factor for event { $n }
b3-fator-exemplo = e.g. 0.1
b3-custo-acao = Cost per share
b3-custo-acao-evento = Cost per share for event { $n }
b3-eventos-revisao = Events under review are only saved if approved, with the factor (new quantity = old × factor; for a stock bonus, new shares per share, e.g. 0.1 = 10%).
b3-ignoradas-texto = Rows that don't become transactions, with the rule that left them out.
b3-ignoradas-tabela = Skipped rows
b3-origem = Source
b3-regra = Rule
b3-linha = Row
b3-motivo = Reason
b3-posicao-texto = B3 position on { $data } × quantity calculated by the portfolio with this import: { $conferem } match, { $divergem } differ.
b3-posicao-tabela = Check against the B3 position
b3-carteira = Portfolio
b3-diferenca = Difference
b3-resultado = Imported from B3: { $operacoes ->
    [one] { $operacoes } trade
   *[other] { $operacoes } trades
}, { $proventos } income entries and { $eventos ->
    [one] { $eventos } event
   *[other] { $eventos } events
}; { $ativos ->
    [one] { $ativos } asset added
   *[other] { $ativos } assets added
}. { $ignoradas ->
    [one] { $ignoradas } row was left out.
   *[other] { $ignoradas } rows were left out.
}
b3-linhas-erro = Rows with errors
b3-pendencias-titulo = { $n ->
    [one] 1 sale was left without a recorded purchase
   *[other] { $n } sales were left without a recorded purchase
}
b3-pendencias-texto = The import recorded the sales as B3 reports them. They stay out of the result until you record the purchase or accept them without a cost.
b3-pendencias-link = See them in Portfolio › Sales
b3-linha-erro = Preview row { $n }: { $mensagem }
b3-backup-antes = Backup first
b3-backup-feito = done
b3-backup-nao-precisou = not needed
b3-desfazer-rotulo = Undo
b3-desfazer-como = with the button below or in Settings › History

## Brokerage notes

b3-notas-titulo = Import brokerage notes
b3-etapa-notas = Notes
b3-nota-aria = Note { $nota }
b3-nota-titulo = Note { $nota }
b3-sem-numero = no number
b3-pregao = · trading day { $data }
b3-nota-arquivo = { $arquivo } · { $n ->
    [one] { $n } trade
   *[other] { $n } trades
}
b3-custo-liquidacao = Settlement
b3-custo-registro = Registration
b3-custo-emolumentos = Exchange fees
b3-custo-corretagem = Brokerage
b3-custo-outras = Other
b3-custo-total = Total fees
b3-liquido-nota = Note net
b3-liquido-calculado = Calculated net
b3-liquido-diferente = Note net ≠ calculated net: some line of the note may not have been read. Check the trades against the note before saving.
b3-avisos-nota = Warnings for note { $nota }
b3-ativo-para = Asset for { $titulo }
b3-ativo-cadastrado = Existing asset
b3-ou-codigo = or the ticker
b3-codigo-para = Ticker for { $titulo }
b3-usar-codigo = Use ticker
b3-voltar-automatico = Back to automatic
b3-notas-gravadas = { $n ->
    [one] { $n } trade saved
   *[other] { $n } trades saved
}
b3-ler-notas = Read notes
b3-gravar-n = { $n ->
    [one] Save { $n } trade
   *[other] Save { $n } trades
}
b3-notas-desfeita = Notes import undone
b3-conta-corretora = Broker account
b3-conta-corretora-dica = SINACOR notes are for accounts in reais. The trades go to this account.
b3-notas-pdf = Brokerage notes (PDF)
b3-notas-pdf-dica = One or more notes in the SINACOR format. The files stay only in the app's memory until you save.
b3-arquivos-escolhidos = Chosen files
b3-lendo-notas = Reading the notes…
b3-arquivos = Files
b3-notas-lidas = Notes read
b3-arquivos-nao-lidos = Files not read
b3-senha-pdf = PDF password
b3-senha-pdf-dica = Many brokers use the first 3 digits of the account holder's CPF (Brazilian taxpayer ID). The password only opens the files now: it is not stored.
b3-abrir-senha = Open with the password
b3-nada-gravado = Nothing is saved while any file can't be read; remove the file from the list (Back) or fix the problem.
b3-nao-identificados = Unidentified assets
b3-nao-identificados-texto = The note shows the trading name, not the ticker. Choose the asset for each security or enter the ticker (it is added when you save); without that, its trades are left out.
b3-inconsistencias = Inconsistencies that don't prevent saving
b3-inconsistencia = { $data } · { $ativo }: { $aviso }. It is saved anyway; check it later in the portfolio.
b3-agrupar = Group executions of the same order
b3-agrupar-dica = The broker splits an order into several executions. On (default): those for the same security, side and market on the note become one trade with the average price and the costs added up.
b3-notas-operacoes-tabela = Trades in the notes
b3-gravar = Save
b3-gravar-operacao = Save trade { $n }
b3-nota = Note
b3-execucoes = { $n } executions
b3-titulo-coluna = Security
b3-cadastra = adds { $ativo } · { $nome }
b3-corrigido = corrected by you
b3-corrigir-ativo = Fix asset
b3-taxas = Fees
b3-notas-lancar-caixa-dica = Off (default): the account statement is usually imported separately, and recording here would duplicate the cash. On: each new trade creates the outflow or inflow in the account.
b3-notas-resultado = { $novas ->
    [one] { $novas } new trade
   *[other] { $novas } new trades
} and { $completadas } completed with the costs in { $conta }; { $ignoradas } left out.
b3-ativos-cadastrados = Assets added
