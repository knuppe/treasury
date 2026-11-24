# Glossary: the app's terms, explained in the simple dashboard's "What is this?" and in the ?
# icons next to labels (docs/profile.md, docs/i18n-design.md). Revision: AI draft, not yet
# reviewed by a native speaker.
#
# Each term has -termo (the name), -simples (the "very simple" tone), -equilibrado (balanced and
# technical tones; without it, the simple one is used) and -busca: a regular expression tested
# against the label in lowercase and without accents, with alternatives separated by |. The first
# term that matches wins, in the order of this file (the same as content/glossary.ts), so the
# more specific ones come first. Each language's search follows that language's labels. No curly
# braces in the search (Fluent syntax).

glossario-reserva-termo = Emergency fund
glossario-reserva-simples = Money set aside only for the unexpected, like a doctor's visit or a car repair. Ideally it covers 3 months of your day-to-day spending, kept somewhere you can take it out right away (a savings account, for example).
glossario-reserva-equilibrado = A reserve for the unexpected, worth 3 to 6 months of essential spending, invested with daily liquidity and low risk (Tesouro Selic, a CDB with daily liquidity).
glossario-reserva-busca = emergency (fund|reserve)|^reserve\b

glossario-sobra-termo = What's left over
glossario-sobra-simples = It's what came in during the month minus what went out. If it's negative, more went out than came in: it's worth checking which category you can cut back a little.
glossario-sobra-busca = ^left (over|for)\b|leftover|net for the month|\bsurplus\b

glossario-essenciais-termo = Essential expenses
glossario-essenciais-simples = The expenses you can't cut, like rent, groceries, electricity, water and transportation. The emergency fund is calculated from them.
glossario-essenciais-busca = essential

glossario-taxa_poupanca-termo = Savings rate
glossario-taxa_poupanca-simples = Of everything that came in during the month, how much was saved. If 1,000 came in and 100 was left over, the rate is 10%.
glossario-taxa_poupanca-equilibrado = The share of the period's income that wasn't spent: (income − expenses) ÷ income. Investment contributions and withdrawals don't count as spending.
glossario-taxa_poupanca-busca = savings rate

glossario-orcamento-termo = Budget
glossario-orcamento-simples = A plan for how much you intend to spend on each thing during the month. It helps you notice early when an expense is going over what you planned.
glossario-orcamento-busca = ^budget\b

glossario-patrimonio_liquido-termo = Net worth
glossario-patrimonio_liquido-simples = Everything you have (money in your accounts, investments and property) minus everything you owe. It's a "snapshot" of how much the family really has.
glossario-patrimonio_liquido-equilibrado = Assets (accounts, investments at market value and property) minus liabilities (balances owed on debts and open card bills).
glossario-patrimonio_liquido-busca = net worth

glossario-competencia-termo = Accrual
glossario-competencia-simples = Counts an expense in the month you made it, even if the credit card bill is only paid later. It shows how much you really spent each month.
glossario-competencia-equilibrado = Accrual basis: each transaction counts in the month it happened (the purchase date), not in the month the money left the account (cash basis).
glossario-competencia-busca = \baccrual\b

glossario-juros_compostos-termo = Compound interest
glossario-juros_compostos-simples = Interest that also earns interest. On a debt, it makes the amount grow fast (that's why an unpaid credit card balance weighs so much); on an investment, it makes your money grow more over time.
glossario-juros_compostos-busca = compound interest

glossario-cdi-termo = CDI
glossario-cdi-simples = A benchmark rate for Brazilian banks that moves together with the Selic. When an investment pays "100% of the CDI", it follows this rate.
glossario-cdi-equilibrado = Certificado de Depósito Interbancário: the average rate of one-day loans between banks, very close to the Selic. It's the main benchmark for floating-rate fixed income in Brazil.
glossario-cdi-busca = \bcdi\b

glossario-selic-termo = Selic
glossario-selic-simples = Brazil's base interest rate, set by the Central Bank. When it goes up, loans get more expensive and fixed income pays more.
glossario-selic-equilibrado = The base interest rate set by Copom (the Central Bank's monetary policy committee) every 45 days. It's the floor for fixed income and a tool against inflation.
glossario-selic-busca = \bselic\b

glossario-ipca-termo = Inflation (IPCA)
glossario-ipca-simples = How much prices went up. If inflation was 5% in the year, what used to cost 100 now costs about 105.
glossario-ipca-equilibrado = IPCA: Brazil's official inflation index, published by IBGE. The app uses it to show "real" gains and balances, after the loss of purchasing power.
glossario-ipca-busca = \bipca\b|inflation

glossario-xirr-termo = XIRR (annual return)
glossario-xirr-simples = The portfolio's return per year, taking into account when each amount went in or out. It's like comparing it with an investment's "yearly return".
glossario-xirr-equilibrado = Internal rate of return with irregular dates: the annual rate that brings the present value of all contributions, withdrawals, payouts and the current value to zero. It isn't distorted by when the contributions were made.
glossario-xirr-busca = \bxirr\b|\birr\b

glossario-real-termo = Real value
glossario-real-simples = The result after taking out inflation. If your money earned 10% and prices went up 5%, the real gain was about 5%: that's how much more you can actually buy.
glossario-real-equilibrado = The result deflated by the chosen index (IPCA by default): (1 + nominal) ÷ (1 + inflation) − 1.
glossario-real-busca = \breal\b(?! estate)

glossario-nominal-termo = Nominal value
glossario-nominal-simples = The "sticker" result, without taking out inflation. It looks bigger than the real gain when prices have also gone up.
glossario-nominal-busca = \bnominal\b

glossario-rentabilidade-termo = Return
glossario-rentabilidade-simples = How much the invested money earned, as a percentage of what was invested.
glossario-rentabilidade-equilibrado = The return on the capital invested in the period, including price changes and payouts received.
glossario-rentabilidade-busca = ^(?!.*(tax return|return of capital|refund)).*\breturns?\b(?! to\b)|^performance$

glossario-valor_mercado-termo = Market value
glossario-valor_mercado-simples = How much your investments are worth today, at today's prices. If you sold everything now, you'd get roughly this (before taxes).
glossario-valor_mercado-equilibrado = Quantity × latest quote (or the curve price, for fixed income), converted to the family's currency. Without a quote, the app uses the cost and lets you know.
glossario-valor_mercado-busca = market value

glossario-nao_realizado-termo = Unrealized gain or loss
glossario-nao_realizado-simples = The gain (or loss) that only exists "on paper": the investment went up, but you haven't sold it yet. It can change until you sell.
glossario-nao_realizado-busca = unrealized

glossario-realizado-termo = Realized gain or loss
glossario-realizado-simples = The gain (or loss) that has already turned into real money, because you sold.
glossario-realizado-busca = ^realized\b

glossario-preco_medio-termo = Average price
glossario-preco_medio-simples = How much you paid, on average, for each share or unit, adding up all your purchases. It's what the profit on a sale is calculated from.
glossario-preco_medio-equilibrado = The total cost of purchases (with fees) divided by the quantity held; sales don't change the average price. It's the basis for capital gains tax.
glossario-preco_medio-busca = average (price|cost)|^avg\.? (price|cost)

glossario-custo-termo = Cost
glossario-custo-simples = How much you paid for the investments you still hold, adding up the purchases.
glossario-custo-busca = ^cost\b

glossario-provento-termo = Dividends and other payouts
glossario-provento-simples = Money an investment pays you just for owning it: dividends from stocks, income from real estate funds and interest from bonds.
glossario-provento-equilibrado = Distributions to the investor: dividends, JCP (interest on equity), real estate fund income and amortizations. They count in the total result and in the XIRR.
glossario-provento-busca = payouts?|distributions?|passive income|dividend(?!s? yield)

glossario-dividendo-termo = Dividends
glossario-dividendo-simples = The part of a company's profit that it shares with its shareholders. It's paid into your account.
glossario-dividendo-busca = (stock|cash) dividends?

glossario-jcp-termo = Interest on equity (JCP)
glossario-jcp-simples = Another way for a Brazilian company to share its profit with shareholders, similar to dividends, but with tax withheld at source (17.5% since 2026; 15% through 2025).
glossario-jcp-busca = \bjcp\b|interest on (own )?(equity|capital)

glossario-yield-termo = Yield
glossario-yield-simples = How much an investment paid out in a year, as a percentage of its price. An 8% yield means that for every 100, it paid about 8 in the year.
glossario-yield-busca = \byield\b|\bdy\b

glossario-come_cotas-termo = Come-cotas
glossario-come_cotas-simples = A tax some Brazilian funds charge twice a year (May and November) by reducing the number of units you hold, even if you don't withdraw anything.
glossario-come_cotas-equilibrado = A semiannual advance of income tax on open-ended funds (15% or 20%), collected by reducing units. Equity funds and real estate funds (FII) don't have it.
glossario-come_cotas-busca = come.cotas

glossario-alocacao-termo = Allocation
glossario-alocacao-simples = How your invested money is split between kinds of investment (fixed income, stocks, real estate funds...). Spreading it out helps you not depend on a single thing.
glossario-alocacao-busca = allocation

glossario-rebalanceamento-termo = Rebalancing
glossario-rebalanceamento-simples = Adjusting the portfolio to get back to the split you planned, putting the next money into whatever fell below its target.
glossario-rebalanceamento-busca = rebalanc

glossario-desvio-termo = Deviation from target
glossario-desvio-simples = How far above or below its planned share each kind of investment is, in percentage points.
glossario-desvio-busca = deviation|\bdrift\b

glossario-liquidez-termo = Liquidity
glossario-liquidez-simples = How easily an investment turns into money in your account. The emergency fund needs liquidity: you can take it out the same day.
glossario-liquidez-busca = liquidity

glossario-renda_fixa-termo = Fixed income
glossario-renda_fixa-simples = Investments where you lend money (to the government, a bank or a company) and receive agreed interest. Savings accounts, Tesouro Direto and CDBs are fixed income.
glossario-renda_fixa-busca = fixed income

glossario-renda_variavel-termo = Variable income
glossario-renda_variavel-simples = Investments whose value goes up and down every day, like stocks and real estate funds. They can earn more, but they can also fall.
glossario-renda_variavel-busca = variable income|equities

glossario-fii-termo = Real estate funds (FII)
glossario-fii-simples = Funds that invest in real estate (malls, warehouses, offices) or in real-estate-backed debt. They pay income every month, usually tax-free for individuals in Brazil.
glossario-fii-busca = \bfiis?\b|real estate (investment )?funds?|\breits?\b

glossario-etf-termo = ETF
glossario-etf-simples = A fund traded on the stock exchange as if it were a stock, which copies an index (like the Ibovespa). With a single purchase, you own a piece of many companies.
glossario-etf-busca = \betfs?\b

glossario-acoes-termo = Stocks
glossario-acoes-simples = Small pieces of a company. Whoever owns stock becomes a part-owner: they gain if the company grows in value and receive part of the profit (dividends).
glossario-acoes-busca = ^stocks\b|\bstocks$

glossario-tesouro-termo = Tesouro Direto
glossario-tesouro-simples = Brazilian federal government bonds that anyone can buy, starting from a small amount. They're among the safest investments in the country.
glossario-tesouro-equilibrado = Federal government bonds (Selic, Prefixado, IPCA+) traded through B3. Before maturity, the price follows mark-to-market.
glossario-tesouro-busca = tesouro

glossario-cdb-termo = CDB
glossario-cdb-simples = A loan you make to a Brazilian bank in exchange for interest. It's covered by the FGC (the deposit guarantee fund) up to 250,000 reais per bank.
glossario-cdb-busca = \bcdbs?\b
glossario-rdb-termo = RDB
glossario-rdb-simples = Like a CDB, a loan you make to a Brazilian bank in exchange for interest (Nubank and Inter use this name). It's covered by the FGC up to 250,000 reais per bank and taxed on the regressive table.
glossario-rdb-busca = \brdbs?\b

glossario-lci_lca-termo = LCI and LCA
glossario-lci_lca-simples = Similar to a CDB, but the bank uses the money for real estate (LCI) or agribusiness (LCA). For individuals, they're exempt from income tax.
glossario-lci_lca-busca = \blci\b|\blca\b

glossario-previdencia-termo = Private pension plan
glossario-previdencia-simples = A long-term investment meant for retirement. A PGBL can lower the tax of those who file the full income tax return; a VGBL is for the other cases.
glossario-previdencia-busca = pension plan|private pension|\bpgbl\b|\bvgbl\b

glossario-aporte-termo = Contribution
glossario-aporte-simples = Money you put into an investment. It isn't spending: the money is still yours, it just moved.
glossario-aporte-busca = contribution|^investment\b

glossario-resgate-termo = Withdrawal
glossario-resgate-simples = Taking money out of an investment and back into your account. It isn't new income either: it's your own money coming back.
glossario-resgate-busca = withdrawal|redemption

glossario-indexador-termo = Benchmark index
glossario-indexador-simples = The rate used as the yardstick for the interest on an investment or debt (for example, CDI, IPCA or a fixed rate).
glossario-indexador-busca = \bindexer\b|benchmark|^index$|% of (the )?index

glossario-fatura-termo = Credit card bill
glossario-fatura-simples = The credit card's bill: it adds up what you bought during the month and is due on a fixed date. Paying only the minimum turns the rest into revolving credit, which has extremely high interest.
glossario-fatura-busca = card (bills?|statements?)|^bill\b|\bbill$

glossario-rotativo-termo = Revolving credit card debt
glossario-rotativo-simples = What happens when the card bill isn't paid in full: the rest becomes a debt with some of the highest interest rates in the country. It's worth avoiding whenever possible.
glossario-rotativo-busca = revolving

glossario-cet-termo = Total effective cost (CET)
glossario-cet-simples = How much a loan really costs per year, adding up interest, fees, insurance and taxes. It's the number to compare offers with.
glossario-cet-busca = \bcet\b|effective cost|\btaeg?\b|\bapr\b|\bcft\b|\bcae\b

glossario-amortizacao-termo = Amortization
glossario-amortizacao-simples = The part of the installment that actually reduces the debt (the rest of the installment is interest). Paying amortization early reduces the interest you would still pay.
glossario-amortizacao-busca = amortiz

glossario-price_sac-termo = Price and SAC tables
glossario-price_sac-simples = Two ways to calculate the installments of a loan. With Price, the installments are equal; with SAC, they start higher and go down, and in total it usually comes out cheaper.
glossario-price_sac-busca = price table|\bsac\b

glossario-divida_cara-termo = Expensive debt
glossario-divida_cara-simples = Debt with high interest, like revolving credit card debt, overdraft and some personal loans. It's usually worth paying off first.
glossario-divida_cara-busca = expensive debt|high.interest debt

glossario-saldo_devedor-termo = Balance owed
glossario-saldo_devedor-simples = How much is still left to pay on a debt today, without counting future interest.
glossario-saldo_devedor-busca = balance owed|outstanding balance

glossario-conciliado-termo = Reconciled
glossario-conciliado-simples = A transaction you've already checked against the bank statement. It's how you make sure the balance in the app matches the bank's.
glossario-conciliado-busca = reconcil

glossario-previsto-termo = Planned
glossario-previsto-simples = A transaction that hasn't happened yet (a scheduled bill, a salary that's about to come in). It counts in forecasts, but doesn't change today's balance.
glossario-previsto-busca = \bplanned\b|\bexpected\b|forecast balance

glossario-recorrencia-termo = Recurrence
glossario-recorrencia-simples = An expense or income that repeats (monthly rent, a subscription, a salary). The app creates the next ones on its own so you don't forget.
glossario-recorrencia-busca = recurr

glossario-transferencia-termo = Transfer between accounts
glossario-transferencia-simples = Money that moved from one of your accounts to another of your accounts. It isn't spending or income, so it doesn't count in the month's totals.
glossario-transferencia-busca = \btransfers?\b

glossario-titular-termo = Family member
glossario-titular-simples = The person in the family who owns the account, card, asset or debt. What comes in and goes out of a member's account stays with them; what belongs to everyone stays with the family.
glossario-titular-busca = \bholder\b|\bowner\b

glossario-vencimento-termo = Due date
glossario-vencimento-simples = The deadline to pay a bill without a late fee (or, for an investment, the date it ends and the money comes back to you).
glossario-vencimento-busca = due (date|day)|\bmaturity\b

glossario-cambio-termo = Exchange rate
glossario-cambio-simples = The price of a currency in your own currency (for example, how many reais one dollar costs). The app uses it to add up accounts and investments in other currencies.
glossario-cambio-busca = exchange rate|currency exchange|^exchange\b|\bfx\b

glossario-deflator-termo = Deflator
glossario-deflator-simples = The inflation index used to take the loss of purchasing power out of amounts (the app uses the IPCA, unless you choose another one).
glossario-deflator-busca = deflator

glossario-m2-termo = M2 (money supply)
glossario-m2-simples = A measure of how much money there is in the economy. When it grows much faster than production, it tends to push prices up.
glossario-m2-busca = \bm2\b

glossario-score_saude-termo = Health score
glossario-score_saude-simples = A score from 0 to 100 that sums up whether an investment is doing well (profit, debt, price, track record), with an explanation of each part. It isn't a recommendation to buy or sell.
glossario-score_saude-busca = asset health|health of (the )?assets?|asset score

glossario-volatilidade-termo = Volatility
glossario-volatilidade-simples = How much the price of an investment goes up and down. The higher it is, the more of a "roller coaster" it is.
glossario-volatilidade-busca = volatil

glossario-drawdown-termo = Maximum drawdown
glossario-drawdown-simples = The biggest drop the investment had, from its highest point to its lowest, in the period. It shows the worst moment its investors went through.
glossario-drawdown-busca = drawdown|max(imum)? (drop|decline)

glossario-aliquota-termo = Tax rate
glossario-aliquota-simples = The percentage of tax charged on an amount (for example, 15% on the profit).
glossario-aliquota-busca = tax rate

glossario-darf-termo = DARF
glossario-darf-simples = The Brazilian federal revenue service's payment slip for taxes, such as the tax on profits from stocks sold during the month.
glossario-darf-busca = \bdarf\b

glossario-ir-termo = Income tax
glossario-ir-simples = Federal tax on what you earn (salary, rent, investment profits). Part of it is already withheld at source; the yearly tax return settles the account.
glossario-ir-busca = income tax|\birpf\b|\bir\b|tax withheld|withheld tax
