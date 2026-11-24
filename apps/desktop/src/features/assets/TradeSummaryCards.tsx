import type { ReactNode } from 'react';

import type { Asset } from '@/api/generated/Asset';
import type { PositionView } from '@/api/generated/PositionView';
import type { TradeSideTotals } from '@/api/generated/TradeSideTotals';
import { useTradeSummary } from '@/api/hooks/portfolio';
import { InstrumentCard, Measure, Money, Private } from '@/design/components';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { Pct } from '../portfolio/components/Pct';
import { formatQuantity, formatUnitPrice } from '../portfolio/decimal';
import { isValueOnly } from '../portfolio/labels';

/**
 * Summary above the asset's trade list: buys, sells, costs and the current position.
 * The totals come from `portfolio.trade_summary`; the position (with splits, bonuses and
 * come-cotas) and the realized gain come from `portfolio.position`.
 */
export function TradeSummaryCards({
  asset,
  position,
}: {
  asset: Asset;
  position: PositionView | undefined;
}) {
  const q = useTradeSummary(asset.id);
  const s = q.data;
  // No summary while loading, on error (the list shows its own) or without trades.
  if (!s || s.buys.count + s.sells.count === 0) return null;
  const valueOnly = isValueOnly(asset.class);
  const ccy = s.currency;
  const price = (v: string | null) =>
    v ? <Private>{formatUnitPrice(v, ccy, { maxDecimals: 4 })}</Private> : '—';
  const qty = (v: string) => <Private>{formatQuantity(v)}</Private>;
  const sideRows = (side: TradeSideTotals, avgLabel: string) => [
    { label: t('ativos-resumo-ops-operacoes'), value: side.count },
    ...(valueOnly
      ? []
      : [
          { label: t('ativos-quantidade'), value: qty(side.quantity) },
          { label: avgLabel, value: price(side.avg_price) },
        ]),
  ];
  const differs =
    !valueOnly && position != null && s.events > 0 && position.quantity !== s.net_quantity;
  const period =
    s.first_date && s.last_date
      ? s.first_date === s.last_date
        ? t('ativos-resumo-ops-periodo-um-dia', { data: formatDate(s.first_date) })
        : t('ativos-resumo-ops-periodo', {
            de: formatDate(s.first_date),
            ate: formatDate(s.last_date),
          })
      : null;
  const notes = [
    period,
    !valueOnly && s.split_adjusted ? t('ativos-resumo-ops-ajustado') : null,
    differs ? t('ativos-resumo-ops-eventos', { n: s.events }) : null,
  ].filter((x): x is string => Boolean(x));

  return (
    <section aria-label={t('ativos-resumo-ops-aria')} className="flex flex-col gap-2">
      <div className="grid gap-4 @min-[530px]:grid-cols-2 @min-[1040px]:grid-cols-4">
        <SummaryCard
          index="01"
          title={t('ativos-resumo-ops-compras')}
          measure={t('ativos-resumo-ops-total-comprado')}
          value={<Money cents={s.buys.gross} currency={ccy} />}
          changeKey={s.buys.gross}
          rows={sideRows(s.buys, t('ativos-resumo-ops-preco-medio-compra'))}
        />
        <SummaryCard
          index="02"
          title={t('ativos-resumo-ops-vendas')}
          measure={t('ativos-resumo-ops-total-vendido')}
          value={<Money cents={s.sells.gross} currency={ccy} />}
          changeKey={s.sells.gross}
          rows={[
            ...sideRows(s.sells, t('ativos-resumo-ops-preco-medio-venda')),
            ...(s.sells.count > 0 && position
              ? [
                  {
                    label: t('ativos-resumo-ops-resultado'),
                    value: <Money cents={position.realized_base} tone="auto" />,
                  },
                ]
              : []),
          ]}
        />
        <SummaryCard
          index="03"
          title={t('ativos-resumo-ops-custos')}
          measure={t('ativos-resumo-ops-total-custos')}
          value={<Money cents={s.costs} currency={ccy} />}
          changeKey={s.costs}
          rows={[
            {
              label: t('ativos-resumo-ops-taxas'),
              value: <Money cents={s.buys.fees + s.sells.fees} currency={ccy} />,
            },
            {
              label: t('ativos-resumo-ops-ir'),
              value: <Money cents={s.buys.taxes + s.sells.taxes} currency={ccy} />,
            },
            {
              label: t('ativos-resumo-ops-do-volume'),
              value: <Pct value={s.cost_share} decimals={2} />,
            },
          ]}
        />
        <SummaryCard
          index="04"
          title={t('ativos-resumo-ops-posicao')}
          measure={valueOnly ? t('ativos-aplicado') : t('ativos-quantidade')}
          value={
            position == null ? (
              '—'
            ) : valueOnly ? (
              <Money cents={position.cost} currency={ccy} />
            ) : (
              qty(position.quantity)
            )
          }
          changeKey={position ? `${position.quantity}:${position.cost}` : ''}
          rows={
            position == null
              ? []
              : position.quantity === '0'
                ? [{ label: t('ativos-resumo-ops-situacao'), value: t('ativos-resumo-ops-zerada') }]
                : [
                    ...(valueOnly
                      ? []
                      : [{ label: t('ativos-preco-medio'), value: price(position.avg_price) }]),
                    {
                      label: t('ativos-resumo-ops-custo-posicao'),
                      value: <Money cents={position.cost} currency={ccy} />,
                    },
                  ]
          }
        />
      </div>
      {notes.length ? <p className="text-[12px] text-dim">{notes.join(' · ')}</p> : null}
    </section>
  );
}

function SummaryCard({
  index,
  title,
  measure,
  value,
  changeKey,
  rows,
}: {
  index: string;
  title: string;
  measure: string;
  value: ReactNode;
  changeKey: string | number;
  rows: { label: string; value: ReactNode }[];
}) {
  return (
    <InstrumentCard index={index} title={title} headingLevel={3}>
      <Measure size="md" label={measure} value={value} changeKey={changeKey} />
      {rows.length ? (
        // Label left, value right: long labels ("Preço médio de compra") do not push the
        // value onto a second line in a narrow card.
        <dl className="mt-3 flex flex-col gap-1.5">
          {rows.map((r) => (
            <div key={r.label} className="flex items-baseline justify-between gap-3">
              <dt className="label-caps min-w-0">{r.label}</dt>
              <dd className="num shrink-0 text-[12px] whitespace-nowrap text-fg">{r.value}</dd>
            </div>
          ))}
        </dl>
      ) : null}
    </InstrumentCard>
  );
}
