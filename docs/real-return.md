# Real result in the Result cards

In the "Result" cards of the asset page and the Portfolio, the user picks the perspective in the
title (nominal, IPCA, IGP-M, the country's inflation, US CPI, M2 BR, M2 US) and sees the result
through it.

## The calculation

- **Nominal:** market value − cost + realized + dividends.
- **Real:** each purchase, sale and dividend is brought to the position date by the chosen index
  (`indices::deflator_factor`, the same as the real XIRR). The market value is already at the date.
  The real result is the nominal one minus the effect of the index:

  ```
  efeito = Σ fluxo nominal − Σ fluxo corrigido até hoje   (compras saem, vendas e proventos entram)
  real   = nominal − efeito
  ```

  With a flat price, the nominal result is zero and the real result is the inflation loss on what
  was bought. Dividends count on both sides: an old dividend is worth more in today's money.
- **Sign of the effect:** positive when the index took from the result, the same as the
  `inflation_effect` of Sales ([sales-real-return.md](sales-real-return.md)). On the card it appears with the
  sign of the calculation ("IPCA effect −R$ 30.12").
- **Difference from Sales:** in Sales, the cost of each sale is adjusted to the date of the
  sale, to compare each sale in its own time. Here everything is brought to the position date,
  so the whole result is in today's money.
- **No index data** for some month of the period, the real result has no value ("No IPCA for
  the period") and the card shows the nominal one in the footer.

In the core: `PositionView.real_result_base` and `inflation_effect_base`, `PortfolioSummary.real_result`
and `inflation_effect` (sum of the positions, open and closed, like the realized result).

## The interface

- `ResultCard` (`features/portfolio/components/ResultCard.tsx`) is the standard card. The selector
  (`ResultPerspective`) sits in the title actions and is the same deflator as the portfolio's
  (`usePortfolioUi`): changing it in one place changes the real annual return, Sales and the other card.
- In the real perspective, the main measure is "Real result (IPCA)" with the percentage over
  cost. The footer shows the nominal result, the effect of the index and the nominal parts (unrealized,
  realized, dividends). In "Nominal", the card shows only the nominal parts.
