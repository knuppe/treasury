# Demo vault

To try Treasury without real data: a wizard asks what the family looks like and the app creates a
**new, separate vault** ([vaults.md](vaults.md)), marked as a demo, with a sample history. The vault
in use is closed with a backup and does not change; to go back to it, use Vaults.

Where it is: when creating a vault (the first time, after the terms, and in "New vault"), the
first step is to choose between **Real vault** (with a master password) and **Demo**. The
demo asks only for the country (suggested by the open family or by the app's language) and one of the five
profiles, and "Create and open" creates and opens the vault, without a password; "Customize before creating" opens the
full wizard. The same shortcut is in Vaults › "Demo vault" (in the side menu,
in Settings › Vaults and in "Other vaults" on the lock screen) and in the welcome. When the chosen vault
has no file yet (first time or "New vault"), the demo takes that
vault's place, without leaving an empty vault in the list.

## No password

The demo vault **opens without a password**: `vaults.new_demo` without `password` creates the
encrypted file with the app's fixed key (`demo::DEMO_PASSWORD`) and marks the entry in the vault list
(`demo = true` in `config.toml`). When locked, the screen shows "Open the demo", which calls
`vault.unlock_demo` (refused on any vault that does not have this mark). Since anyone
with the app can open this vault, the screen warns: fictional data only, never real data. The copy of a
demo without a password also opens without a password. Real vaults do not change.

## The wizard

0. **Profile**: the family's country (currency, categories, taxes and the sample's assets) and a
   ready-made profile or "Start from scratch". The profiles (`features/demo/presets.ts`, amounts in reais
   converted by a typical-income factor of the currency, not by the exchange rate): **comfortable middle class** (couple, one child, mortgage, car, emergency fund and investments), **low income**
   (couple, two children, rent, bus, side jobs and Bolsa Família), **family in financial trouble**
   (spends more than it earns: overdraft, revolving card balance, personal loan and late bills), **young person living alone** and **retiree** (INSS with the 13th, own home, rent
   received, health insurance and emergency fund). Everything can be changed in the next steps.
1. **Family**: vault name and family name, one or two adults with income, children (up to 3 in the
   screen; the core accepts 4), the age and sex of each person and the months of history (6, 12 or
   24; the core accepts 3 to 36). The ready-made profiles already bring the scenario's ages (the retiree
   at 68, parents old enough for the children); a blank age is left to the core.
2. **Income**: net income of each adult (fixed, with the 13th in November and December, or variable,
   in two or three receipts that change from month to month, or retirement), other income
   (rent received) and social benefit per month.
3. **Housing and expenses**: rent, mortgage or own home; condo fees, household bills,
   groceries, transport (car, ride-hailing, bus and subway or none), school per child, health,
   leisure, subscriptions and shopping. The amounts come suggested from income ("Suggest from income").
4. **Accounts and investments**: bank of the main account, credit card with a limit, emergency fund
   (balance and monthly contribution), stocks, FIIs and Tesouro at the start of the history, monthly
   contribution at the broker and VGBL pension (balance and contribution).
5. **Debts and creation**: car financing and personal loan (installment and how many
   are left), overdraft (limit), bills paid late, revolving card balance, month summary (in, out, invested)
   and interface level. No password.

## What is generated (`vaults.new_demo` → `demo.generate`)

Everything in a single change set ("Demo vault generated"), through the same paths
as the interface, and always the same for the same input (a draw with a seed taken from the names):

- Members (adults and children) with birth date, sex and relationship (holder, spouse,
  child). The date comes from today's age (`age` in `DemoAdult` and `child_details`); with no age, the
  core picks one that fits the profile, with its own draw so as not to change the rest of the
  history: retiree from 64 to 71 (a spouse who still works, 57 to 63), young person living
  alone from 25 to 31, children from 3 to 11 (older with more siblings, 2 to 4 years apart) and
  parents 25 to 32 years older than the oldest child. This way the Life timeline, the INSS and the
  retirement horizon make sense from day one. With no sex given, it stays
  "not informed".
- A joint checking account, a card (closes on the 3rd, due on the 10th, paid
  from the account), an emergency fund account, a broker and a pension account, according to what was requested.
- Texts in the app's language (the `demodados.ftl` catalog: account names, descriptions, stores and
  companies) and the family's country in the vault (the country's currency and categories). Outside Brazil there is no
  13th; in Portugal the Christmas (December) and holiday (July) subsidies come in; subcategories
  that exist only in Brazil (IPVA, INSS…) fall back to the parent category.
- Fixed incomes registered in Family (salary or retirement), with the payslips of the
  last six months (gross, INSS and IRRF by the year's rules, from the net).
- Hardship: with an overdraft, the account starts with no surplus and each month that closes in the red
  pays 8% interest on day 1 of the next; late bills (one in three, ten days later, with
  penalty and interest); revolving balance (pays 70% of each card statement and the card charges 12% on the rest);
  personal loan (4.5% a month) as a debt contract.
- Transactions for each month up to today, with a holder when it makes sense: salaries and 13th, rent
  received, rent, condo fees, electricity (more expensive in summer), water, internet, mobile, fuel, parking, IPVA in January, car
  insurance in March, a service every six months, ride-hailing, school and school supplies in
  February, health insurance and pharmacy, a trip in July and shopping (double in November).
- Day to day (`demo/daily.rs`), with the weekday of each thing: the weekly groceries on
  Saturday (sometimes Sunday; the first of the month is the big wholesale-club run), the street market, the
  greengrocer and the butcher midweek, the bakery almost always on the weekend, coffee and
  snacks on workdays for those who work, restaurants from Friday to Sunday, delivery and cinema.
  In Brazil, the grocery purchases of the last three months come with the **item-by-item receipt**
  (grocery items with 2026 prices; the last, heavy one, closes the total), linked to the transaction,
  for the receipts screen and personal inflation.
- Subscriptions for each profile (4 in the middle class, 3 for the young person, 2 for the retiree and the
  family in financial trouble, 1 for low income), always on the same day and amount, with a **12%
  increase** three months before today (the subscriptions screen shows the rise).
- Each profile's own items, as a fraction of income (valid in any currency), with
  instant transfers named by the country (Pix, MB WAY, Zelle): in the middle class, the
  cleaner every two weeks, the gym, the pet (food and vet in April and October), the
  salon, the allowance for children aged 6 or older and Christmas gifts; in low income, the
  gas cylinder every 45 days and the salon; in the family in financial trouble, gas, the bank's package
  fee and help from the family at the end of the month (half of the months); for the young person, the gym, the
  barber and happy hour on the card with friends paying back their share; for the retiree,
  the pharmacy every week, appointments every three months, a monthly donation, the salon and
  gifts for the grandchildren. A profile built from scratch keeps only what was entered.
- A big purchase split into 10 installments on the card in the middle of the history (middle class, young
  person, family in financial trouble and the store-credit appliance in low income), as an installment
  plan. In low income, also the pharmacy (no health insurance), school supplies in
  February and the children's clothes in March and September.
- The apartment and the car of the financings enter as assets (`physical`), linked to the
  contracts: a 20% down payment on the property (adjusted by IPCA) and 40% on the car (12% a year
  depreciation). Net worth shows the asset and the debt.
- FGTS for each adult with a formal job in Brazil: an `fgts` account with the balance of the years at
  the company before the history, the 8% deposit of gross pay on day 7 (competence of the previous month,
  as on the payslip), the JAM on day 10 by Caixa's rule (3% a year plus the TR) and the profit
  distribution in August. The check (`fgts.check`) closes with no difference; the account has no
  hire date, so as not to charge the deposits from before the history.
- Card statements already due, paid on the due date.
- Financings (housing and car) as debt contracts: the history's installments paid by
  transactions in the contract's category (linked to the installments) and the earlier ones marked as paid.
- Emergency fund with monthly contribution and yield; a portfolio with PETR4, ITSA4, WEGE3, BBAS3, HGLG11,
  MXRF11, XPML11 and Tesouro Selic 2029 (in the US, VTI, VXUS, VNQ and BND; in other countries, VWCE,
  IWDA, IPRP and AGGH; pension becomes a retirement fund): initial position, rotating monthly contribution, dividends from the
  FIIs every month and from stocks quarterly; VGBL pension with the plan (regressive) and contributions.
  **Quotes are fictional** (manual quote, weekly): the sample does not mix real prices
  with invented trades.
- Recurrences for salaries and fixed bills from next month on (due dates), the month's
  budget from what was entered and the emergency fund goal (6 months of essential
  spending).
- Profile at the chosen level (no questionnaire) and the `demo_vault` mark, which lights up the
  "Demo" badge in the top bar (`vault.status.demo`).

`demo.generate` refuses a vault with transactions and only the app's interface calls it (never the agent
or MCP clients).
