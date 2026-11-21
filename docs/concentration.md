# Portfolio concentration

Shows how much an asset, a group of assets or a class weighs in the portfolio against a family
limit. It works without setup and does not recommend buying or selling: it only says where the
weight went over the limit. It appears in the concentration card of the Portfolio, in the weight
tag of the positions table and in the limits dialog.

## Limits

| What | Default limit | Where to change it |
|---|---|---|
| Asset | 20% of the portfolio (setting `concentration.asset_default_pct`) | Limits dialog; a custom limit per asset (`asset.concentration_limit_pct`) |
| Group | 30% (setting `concentration.group_default_pct`) | A label marked as a concentration group, with an optional custom limit |
| Class | Only when there is an allocation target: target ± tolerance | Allocation targets in the Portfolio |

- Above the limit, the alert is a **warning**; from 1.5 times the limit, it is **critical**.
- The default per-asset limit does not apply to Tesouro Direto (Brazilian federal government
  bonds) or to fixed income covered by the FGC (Brazil's deposit guarantee fund) while the value
  fits within the per-institution guarantee (CDB, RDB, LC, LCI, LCA and savings): the family's
  emergency fund is usually in a single security, and that is not the risk the alert is meant to
  show. A custom limit on the asset always applies.
- The weight is today's weight, at market price, in the base currency.

## Labels

Labels are free-form names attached to assets (up to 20 per asset, up to 40 characters), stored
in `asset_label` and `asset_label_link` (V161). They serve two purposes:

- **Concentration group**: the combined weight of the assets with the label is checked against
  the group limit (for example, several companies with the same controlling shareholder).
- **Goal sources**: all positions with the label count toward a goal
  ([goal-sources.md](goal-sources.md)).

Do not confuse them with the computed traits in Market (`research.tags`).

## Rebalancing

The contribution suggestion (`rebalance_suggestion`) respects the limits: each asset receives
only up to the room it has under its own limit and under that of each group. Without a class
goal, the contribution goes only to assets with room, and the app never suggests selling. An
asset exempt from the default limit keeps the default for spreading the contribution, but does
not show up as over the limit.

## In the code

- Core: `crates/treasury-core/src/portfolio/concentration.rs` (report, limits, room for the
  contribution) and `crates/treasury-core/src/assets/labels.rs` (labels).
- Commands: `portfolio.concentration`, `portfolio.set_concentration_limit`, `assets.labels_list`, `assets.labels_set`, `assets.label_update`.
- MCP: `get_concentration`, `list_asset_labels` (read); `set_asset_labels`,
  `tag_as_concentration_group`, `set_concentration_limit` (writes become proposals).
- UI: `apps/desktop/src/features/portfolio/Concentration.tsx`.

## Limitations

- The family's financial health has no diversification dimension; concentration does not enter
  the score.
