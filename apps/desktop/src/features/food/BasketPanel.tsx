/**
 * Inflação da sua cesta (docs/gtin-inflation.md §5): o mesmo produto, na mesma loja, mês a mês,
 * contra o índice oficial de alimentos (no Brasil, o IPCA "Alimentação no domicílio"). A manchete
 * é "as mesmas compras custariam R$ X há um ano"; embaixo, o gráfico e o que mais subiu e caiu.
 */
import { useMemo } from 'react';

import type { BasketInflation } from '@/api/generated/BasketInflation';
import type { BasketMover } from '@/api/generated/BasketMover';
import { useBasketInflation } from '@/api/hooks/food';
import { BasketChart } from '@/charts/BasketChart';
import { Button, InstrumentCard, Measure, Money } from '@/design/components';
import { QueryState } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';

import { pct, unitLabel } from './FoodPanels';

/** "As mesmas compras custariam R$ X há um ano" (ou o motivo de ainda não haver número). */
function Headline({ b }: { b: BasketInflation }) {
  if (b.same_basket_now != null && b.same_basket_year_ago != null) {
    const v = b.same_basket_year_ago > 0 ? b.same_basket_now / b.same_basket_year_ago - 1 : null;
    return (
      <p className="text-[14px] leading-relaxed text-fg">
        {withSlots(
          t('alimentacao-cesta-manchete', {
            antes: slot(0),
            agora: slot(1),
            variacao: pct(v),
          }),
          [
            <Money key="a" cents={b.same_basket_year_ago} />,
            <Money key="b" cents={b.same_basket_now} />,
          ],
        )}
      </p>
    );
  }
  return (
    <p className="text-[13px] text-dim">
      {b.enough ? t('alimentacao-cesta-sem-ano') : t('alimentacao-cesta-poucas')}
    </p>
  );
}

function Movers({ title, rows }: { title: string; rows: readonly BasketMover[] }) {
  return (
    <section aria-label={title} className="flex flex-col gap-1.5">
      <h3 className="label-caps text-dim">{title}</h3>
      {rows.length ? (
        <ul className="flex flex-col gap-1 text-[13px]">
          {rows.map((m) => (
            <li key={m.product_id} className="flex items-baseline justify-between gap-3">
              <span className="min-w-0 truncate" title={m.name}>
                {m.name}
              </span>
              <span className="shrink-0 text-right">
                <Money cents={m.before} />
                {' → '}
                <Money cents={m.now} />
                <span className="text-dim">{unitLabel(m.base_unit)}</span>{' '}
                <span className={m.change > 0 ? 'text-warning' : 'text-positive'}>
                  {pct(m.change)}
                </span>
              </span>
            </li>
          ))}
        </ul>
      ) : (
        <p className="text-[12.5px] text-dim">{t('alimentacao-cesta-nada')}</p>
      )}
    </section>
  );
}

/** Aba Preços de 06 · Consumo. */
export function BasketPanel() {
  const q = useBasketInflation({ months: 12 });
  const b = q.data;
  const chart = useMemo(
    () =>
      b
        ? {
            months: b.months.map((m) => m.month),
            basket: b.months.map((m, i) => (i === 0 || m.change != null ? m.index : null)),
            official: b.months.map((m) => m.official),
            officialName: b.official_name,
          }
        : null,
    [b],
  );
  return (
    <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
      {b && chart ? (
        <div className="flex flex-col gap-4">
          <InstrumentCard index="06.12" title={t('alimentacao-cesta-titulo')}>
            <div className="flex flex-col gap-4">
              <Headline b={b} />
              <div className="grid gap-5 @min-[600px]:grid-cols-2">
                <Measure
                  size="md"
                  label={t('alimentacao-cesta-sua')}
                  value={pct(b.change)}
                  hint={t('alimentacao-cesta-sua-dica')}
                />
                <Measure
                  size="md"
                  label={b.official_name ?? t('alimentacao-cesta-oficial')}
                  value={pct(b.official_change)}
                  hint={
                    b.official_change == null
                      ? t('alimentacao-cesta-oficial-sem')
                      : t('alimentacao-cesta-oficial-dica')
                  }
                />
              </div>
              {b.enough ? (
                <BasketChart input={chart} ariaLabel={t('alimentacao-cesta-grafico')} />
              ) : null}
              <p className="text-[12px] text-dim">{t('alimentacao-cesta-metodo')}</p>
            </div>
          </InstrumentCard>
          <InstrumentCard index="06.13" title={t('alimentacao-cesta-mudaram')}>
            <div className="grid gap-5 @min-[720px]:grid-cols-2">
              <Movers title={t('alimentacao-cesta-subiram')} rows={b.risers} />
              <Movers title={t('alimentacao-cesta-cairam')} rows={b.fallers} />
            </div>
          </InstrumentCard>
        </div>
      ) : null}
    </QueryState>
  );
}

/** Cartão curto na Visão geral, com o link para a aba Preços. */
export function BasketCard({ onOpen }: { onOpen: () => void }) {
  const q = useBasketInflation({ months: 12 });
  const b = q.data;
  if (!b || (!b.enough && b.same_basket_now == null)) return null;
  return (
    <InstrumentCard index="06.14" title={t('alimentacao-cesta-titulo')}>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Headline b={b} />
          {b.change != null ? (
            <p className="text-[12.5px] text-dim">
              {t('alimentacao-cesta-resumo', {
                sua: pct(b.change),
                oficial: pct(b.official_change),
                nome: b.official_name ?? t('alimentacao-cesta-oficial'),
              })}
            </p>
          ) : null}
        </div>
        <Button size="sm" variant="secondary" onClick={onOpen}>
          {t('alimentacao-cesta-ver')}
        </Button>
      </div>
    </InstrumentCard>
  );
}
