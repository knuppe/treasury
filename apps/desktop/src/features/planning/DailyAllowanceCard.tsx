/**
 * "Quanto posso gastar hoje" (docs/forecast.md): o que sobra do mês para o dia a dia,
 * dividido pelos dias que faltam. Destaque nos painéis Iniciante e Intermediário.
 */
import { useState } from 'react';

import type { DailyAllowance } from '@/api/generated/DailyAllowance';
import { useDailyAllowance } from '@/api/hooks/planning';
import { Button, Money } from '@/design/components';
import { cx } from '@/design/cx';
import { CoverageLine } from '@/features/attention/CoverageLine';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { monthBounds } from '@/lib/months';

/** Frase do dia, sem julgamento. */
export function allowanceSentence(a: DailyAllowance): string {
  if (a.status === 'over') return t('planejamento-hoje-estourado');
  if (a.today_left === 0) return t('planejamento-hoje-usado');
  if (a.status === 'tight') return t('planejamento-hoje-apertado');
  return t('planejamento-hoje-ok');
}

export function DailyAllowanceCard() {
  const { data: a } = useDailyAllowance();
  const [open, setOpen] = useState(false);
  if (!a) return null;
  const end = monthBounds(a.month).to;
  const tone =
    a.status === 'over' ? 'text-negative' : a.status === 'tight' ? 'text-warning' : 'text-positive';
  // O valor de hoje acabou, mas o mês ainda tem sobra: o destaque é o valor de amanhã em
  // diante (um R$ 0,00 grande não diz o que dá para gastar).
  const tomorrowFirst = a.today_left === 0 && a.per_day_from_tomorrow > 0;
  return (
    <section
      aria-label={t('planejamento-hoje-titulo')}
      className={cx(
        'flex flex-col gap-2 rounded-card border bg-surface px-4 py-3.5',
        a.status === 'over'
          ? 'border-negative/50'
          : a.status === 'tight'
            ? 'border-warning/50'
            : 'border-line',
      )}
    >
      <div className="flex flex-wrap items-baseline justify-between gap-x-6 gap-y-1">
        <div className="flex flex-col gap-1">
          <span className="text-[13px] text-dim">
            {tomorrowFirst ? t('planejamento-hoje-amanha-titulo') : t('planejamento-hoje-titulo')}
          </span>
          <Money
            cents={tomorrowFirst ? a.per_day_from_tomorrow : a.today_left}
            currency={a.currency}
            className={cx('text-[30px] leading-tight', tone)}
          />
        </div>
        <div className="flex flex-col items-end gap-0.5 text-right text-[12.5px] text-dim">
          <span>
            {tRich('planejamento-hoje-por-dia', {
              valor: <Money cents={a.per_day} currency={a.currency} className="text-fg" />,
              data: formatDate(end).slice(0, 5),
              n: a.days_left,
            })}
          </span>
          {!tomorrowFirst && a.spent_today > 0 && a.days_left > 1 ? (
            <span>
              {tRich('planejamento-hoje-amanha', {
                valor: (
                  <Money
                    cents={a.per_day_from_tomorrow}
                    currency={a.currency}
                    className="text-fg"
                  />
                ),
                n: a.days_left - 1,
              })}
            </span>
          ) : null}
          {a.spent_today > 0 ? (
            <span>
              {tRich('planejamento-hoje-gasto', {
                valor: <Money cents={a.spent_today} currency={a.currency} />,
              })}
            </span>
          ) : null}
        </div>
      </div>
      <p className="text-[13.5px] text-fg">{allowanceSentence(a)}</p>
      <CoverageLine onlyPartial />
      <div>
        <Button size="sm" variant="ghost" onClick={() => setOpen((o) => !o)} aria-expanded={open}>
          {open ? t('planejamento-hoje-esconder') : t('planejamento-hoje-como')}
        </Button>
      </div>
      {open ? (
        <dl className="grid grid-cols-[1fr_auto] gap-x-6 gap-y-1 text-[12.5px]">
          <dt className="text-dim">{t('planejamento-hoje-receitas')}</dt>
          <dd className="text-right">
            <Money cents={a.income} currency={a.currency} />
          </dd>
          <dt className="text-dim">{t('planejamento-hoje-gastos')}</dt>
          <dd className="text-right">
            <Money cents={-(a.spent - a.spent_today)} currency={a.currency} />
          </dd>
          <dt className="text-dim">{t('planejamento-hoje-contas')}</dt>
          <dd className="text-right">
            <Money cents={-a.fixed_remaining} currency={a.currency} />
          </dd>
          <dt className="text-dim">{t('planejamento-hoje-metas')}</dt>
          <dd className="text-right">
            <Money cents={-a.savings_remaining} currency={a.currency} />
          </dd>
          <dt className="border-t border-line pt-1 text-fg">{t('planejamento-hoje-sobra')}</dt>
          <dd className="border-t border-line pt-1 text-right">
            <Money cents={a.available + a.spent_today} currency={a.currency} tone="auto" />
          </dd>
          <dd className="col-span-2 pt-1 text-dim">
            {t('planejamento-hoje-dividido', { n: a.days_left })}
          </dd>
          {a.spent_today > 0 && a.days_left > 1 ? (
            <>
              <dt className="pt-2 text-dim">{t('planejamento-hoje-gasto-linha')}</dt>
              <dd className="pt-2 text-right">
                <Money cents={-a.spent_today} currency={a.currency} />
              </dd>
              <dt className="border-t border-line pt-1 text-fg">
                {t('planejamento-hoje-sobra-amanha')}
              </dt>
              <dd className="border-t border-line pt-1 text-right">
                <Money cents={a.available} currency={a.currency} tone="auto" />
              </dd>
              <dd className="col-span-2 pt-1 text-dim">
                {t('planejamento-hoje-dividido-amanha', { n: a.days_left - 1 })}
              </dd>
            </>
          ) : null}
        </dl>
      ) : null}
    </section>
  );
}
