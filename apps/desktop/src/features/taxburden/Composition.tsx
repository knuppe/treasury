/**
 * Composição da estimativa por tipo de imposto (consumo, IR, INSS, patrimônio, proventos,
 * outros) no cenário escolhido, com a visão em tabela, e os três cenários lado a lado.
 */
import { useCallback, useMemo } from 'react';

import type { Scenario } from '@/api/generated/Scenario';
import type { TaxEstimate } from '@/api/generated/TaxEstimate';
import { ChartFrame, EChart, type ChartTheme } from '@/charts';
import { InstrumentCard, Money, Private } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatPercent } from '@/lib/format';

import { PART_SHORT, pick, scenarioDef, SCENARIOS } from './model';
import { partsHeight, partsOption, type PartBar } from './partsChart';

export function CompositionCard({
  estimate: e,
  scenario,
}: {
  estimate: TaxEstimate;
  scenario: Scenario;
}) {
  const def = scenarioDef(scenario);
  const bars = useMemo<PartBar[]>(
    () =>
      e.parts.map((p) => ({
        key: p.kind,
        label: PART_SHORT[p.kind],
        amount: pick(p.amount, scenario),
        estimated: p.estimated,
      })),
    [e.parts, scenario],
  );
  const total = pick(e.totals, scenario);
  const option = useCallback(
    (theme: ChartTheme) => partsOption(theme, bars, e.currency),
    [bars, e.currency],
  );
  return (
    <InstrumentCard
      index="10.3"
      title={t('impostos-composicao')}
      className="@min-[1040px]:col-span-6"
    >
      {bars.length === 0 ? (
        <p className="text-[13px] text-dim">{t('impostos-composicao-vazia')}</p>
      ) : (
        <ChartFrame
          index="01"
          title={t('impostos-por-tipo', { cenario: def.label })}
          source={t('impostos-composicao-fonte')}
          chart={
            <EChart
              option={option}
              height={partsHeight(bars.length)}
              ariaLabel={t('impostos-por-tipo-aria', { cenario: def.label })}
            />
          }
          table={
            <table className="w-full text-[12.5px]">
              <caption className="sr-only">
                {t('impostos-por-tipo-aria', { cenario: def.label })}
              </caption>
              <thead>
                <tr className="label-caps text-left">
                  <th className="py-1.5 font-normal">{t('impostos-tipo')}</th>
                  <th className="py-1.5 text-right font-normal">{t('impostos-valor')}</th>
                  <th className="py-1.5 text-right font-normal">{t('graficos-do-total')}</th>
                </tr>
              </thead>
              <tbody>
                {e.parts.map((p) => {
                  const v = pick(p.amount, scenario);
                  return (
                    <tr key={p.kind} className="border-t border-line/70">
                      <td className="py-1.5 text-fg">{p.label}</td>
                      <td className="py-1.5 text-right">
                        <Money cents={v} currency={e.currency} estimated={p.estimated} />
                      </td>
                      <td className="num py-1.5 text-right text-dim">
                        <Private>{total > 0 ? formatPercent(v / total) : '—'}</Private>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          }
        />
      )}
    </InstrumentCard>
  );
}

export function ScenariosCard({
  estimate: e,
  scenario,
}: {
  estimate: TaxEstimate;
  scenario: Scenario;
}) {
  const col = (s: Scenario) => cx('py-1.5 pl-3 text-right', s === scenario && 'bg-accent-soft/40');
  return (
    <InstrumentCard
      index="10.4"
      title={t('impostos-tres-cenarios')}
      className="@min-[1040px]:col-span-6"
      footer={<p className="text-[12px] leading-snug text-dim">{t('impostos-cenarios-nota')}</p>}
    >
      <div className="overflow-x-auto">
        <table
          className="w-full min-w-[420px] text-[12.5px]"
          aria-label={t('impostos-tres-cenarios')}
        >
          <thead>
            <tr className="label-caps text-left">
              <th className="py-1.5 font-normal">{t('impostos-tipo')}</th>
              {SCENARIOS.map((s) => (
                <th
                  key={s.value}
                  scope="col"
                  className={cx(col(s.value), 'font-normal', s.value === scenario && 'text-accent')}
                  aria-current={s.value === scenario ? 'true' : undefined}
                  title={s.hint}
                >
                  {s.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {e.parts.map((p) => (
              <tr key={p.kind} className="border-t border-line/70">
                <th scope="row" className="py-1.5 text-left font-normal text-fg">
                  {PART_SHORT[p.kind]}
                </th>
                {SCENARIOS.map((s) => (
                  <td key={s.value} className={col(s.value)}>
                    <Money cents={pick(p.amount, s.value)} currency={e.currency} />
                  </td>
                ))}
              </tr>
            ))}
            <tr className="border-t border-line-strong">
              <th scope="row" className="py-1.5 text-left font-medium text-fg">
                {t('impostos-total')}
              </th>
              {SCENARIOS.map((s) => (
                <td key={s.value} className={cx(col(s.value), 'font-medium')}>
                  <Money cents={pick(e.totals, s.value)} currency={e.currency} />
                </td>
              ))}
            </tr>
            <tr className="border-t border-line/70">
              <th scope="row" className="py-1.5 text-left font-normal text-dim">
                {t('impostos-pct-renda')}
              </th>
              {SCENARIOS.map((s) => (
                <td key={s.value} className={cx(col(s.value), 'num text-dim')}>
                  <Private>{formatPercent(pick(e.share_of_income, s.value))}</Private>
                </td>
              ))}
            </tr>
          </tbody>
        </table>
      </div>
    </InstrumentCard>
  );
}
