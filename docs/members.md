# Finances by family member

What comes in, goes out and is left over for each person in the family, each person's net worth and
health, and the agent's tone according to who is using it. It lives in "View as" in the sidebar (below the family name), the "By family member" panel on the Dashboard and each
member's profile in Settings › Profile. Core in `reports/member.rs` (the slice), `household/overview.rs`
(overview) and `profile` (per-member profile).

Family members and holders are the same thing: the people in the vault (with no login of their own, decision
D6), registered on the **Family** screen (with each one's income, payslips, accounts and cards)
or in Settings › Family. Whoever uses the app picks the
perspective at the top: **Family** (everything, as always) or one member. In a member's view, the
dashboard, net worth, financial health, interface level and the agent's tone become
theirs.

## What belongs to each member

| Data | Rule |
|---|---|
| Transaction | The holder of the part (split) or of the transaction; with none, the account owner when it has **a single holder**. Joint account with no holder on the transaction: the family's ("Family" in the panel). |
| Account | The member's fraction: 1 ÷ number of holders (a joint account of 2 counts half in their net worth). Account with no holder: the family's. |
| Investments | Their fraction of the accounts with trades (brokerages). Capital shares stay with the family only. |
| Physical asset | Their fraction (the asset's holders). |
| Debt | The contracts where they are a holder. |
| Budget | The family's (it does not appear in a member's view). |

The transaction rule applies to all per-member reports (`engine::MEMBER_EXPR`).

## Member inherited from the account

The transaction stores the actual member (V122), not just at report time:

- A new transaction with no member in an account or card with **a single member** gets that member
  (`txn.member_origin = 'auto'`). It applies to typed, imported, recurring and agent transactions.
- On a card, a purchase made with an **additional or virtual** card that has a member (V155,
  `card_plastic.member_id`) gets that card's member, before the account holder. Changing the
  card's member, changing the purchase's card or deleting the card recomputes the inherited ones. The card's
  debt still belongs to the account holder ([design/cartoes.md](design/cards.md)).
- Choosing the member by hand (on the transaction, in a batch or through the agent), including "family" (no
  member), sets `member_origin = 'manual'`, and that is never overwritten.
- Changing the account's members recomputes the `auto` transactions (and those with no member and no origin): with a single
  member, they get the new one; with two or more, or none, they go back to the family.
- Moving the transaction to another account recomputes its member from the new account, if it was not chosen by hand.
- Migration V122 marks as `manual` the transactions that already had a member and gives the account's member
  to those that did not (only in single-member accounts).

Core in `transactions::sole_owner` and `transactions::reinherit_members`; the mock repeats the rules
(`ledger/domain.ts`).

## Commands

- `reports.dashboard`, `reports.month_flow`, `reports.net_worth_history` and `health.score`
  accept `member_id`; `reports.spending` already accepted `member_ids`.
- `household.members_overview {month?}`: per member, what came in, went out and was left over in the month, the 12-month
  savings rate, net worth, accounts and debts and the profile level; plus what belongs to the
  family (`shared`).
- `profile.get|answer|skip|set` accept `member_id`: each member answers their own questionnaire
  ([profile.md](profile.md)); without it, the family profile applies (`source = familia`).
- `ai.chat_start {member_id}`: the agent knows who it is talking to and uses that person's level and tone
  (`profile::prompt_line_for`).
- MCP: `get_dashboard` and `get_financial_health` accept `member_id`.
