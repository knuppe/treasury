/**
 * Simulador de quitação (docs/debt-payoff.md): com um extra por mês, compara só o mínimo,
 * bola de neve (menor saldo primeiro) e avalanche (maior taxa primeiro), com os
 * empréstimos cadastrados e dívidas informadas na hora (cheque especial, rotativo).
 */
import { useCallback, useState } from 'react';

import type { CustomDebt } from '@/api/generated/CustomDebt';
import type { PayoffPlan } from '@/api/generated/PayoffPlan';
import type { PayoffResult } from '@/api/generated/PayoffResult';
import type { PayoffStrategy } from '@/api/generated/PayoffStrategy';
import { useDebtPayoff } from '@/api/hooks/planning';
import {
  ChartFrame,
  EChart,
  baseOption,
  categoryAxis,
  compactMoney,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import {
  Button,
  Field,
  Icon,
  IconButton,
  Input,
  InstrumentCard,
  Money,
  MoneyInput,
  TechTable,
} from '@/design/components';
import { cx } from '@/design/cx';
import { InlineError, Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatMoney, formatMonthShort } from '@/lib/format';

const LABEL: Record<PayoffStrategy, string> = {
  get minimum() {
    return t('planejamento-quitacao-minimo');
  },
  get snowball() {
    return t('planejamento-quitacao-bola-neve');
  },
  get avalanche() {
    return t('planejamento-quitacao-avalanche');
  },
};

/** Nome em pt-BR de cada estratégia: guarda as preferências da tabela em qualquer idioma. */
const PREFS_LABEL: Record<PayoffStrategy, string> = {
  minimum: 'Só o mínimo',
  snowball: 'Bola de neve',
  avalanche: 'Avalanche',
};

const HINT: Record<PayoffStrategy, string> = {
  get minimum() {
    return t('planejamento-quitacao-minimo-dica');
  },
  get snowball() {
    return t('planejamento-quitacao-bola-neve-dica');
  },
  get avalanche() {
    return t('planejamento-quitacao-avalanche-dica');
  },
};

interface Draft {
  name: string;
  balance: number | null;
  rate: string;
  min: number | null;
}

const emptyDraft = (): Draft => ({ name: '', balance: null, rate: '', min: null });

function toCustom(d: Draft): CustomDebt | null {
  if (!d.name.trim() || !d.balance || !d.min || !d.rate.trim()) return null;
  return {
    name: d.name.trim(),
    balance: d.balance,
    annual_rate: d.rate.replace(',', '.'),
    min_payment: d.min,
  };
}

function when(r: PayoffResult) {
  return r.debt_free ? formatMonthShort(r.debt_free) : t('planejamento-mais-50-anos');
}

function StrategyColumn({ plan, s }: { plan: PayoffPlan; s: PayoffStrategy }) {
  const r = plan[s];
  const best =
    plan.recommended === s && plan.snowball.total_interest !== plan.avalanche.total_interest;
  return (
    <div
      className={cx(
        'flex flex-col gap-2 rounded-card border bg-surface px-4 py-3',
        best ? 'border-accent' : 'border-line',
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="label-caps text-fg">{LABEL[s]}</span>
        {best ? (
          <span className="label-caps text-[10px] text-accent">
            {t('planejamento-quitacao-menos-juros')}
          </span>
        ) : null}
      </div>
      <span className="text-[12px] text-dim">{HINT[s]}</span>
      <dl className="grid grid-cols-[1fr_auto] gap-x-4 gap-y-1 text-[12.5px]">
        <dt className="text-dim">{t('planejamento-quitacao-livre')}</dt>
        <dd className="num text-right text-fg">{when(r)}</dd>
        <dt className="text-dim">{t('planejamento-quitacao-meses')}</dt>
        <dd className="num text-right">{r.feasible ? r.months : '—'}</dd>
        <dt className="text-dim">{t('planejamento-quitacao-juros-total')}</dt>
        <dd className="text-right">
          {r.feasible ? <Money cents={r.total_interest} /> : t('planejamento-quitacao-sem-fim')}
        </dd>
        {s !== 'minimum' && plan.minimum.feasible ? (
          <>
            <dt className="text-dim">{t('planejamento-quitacao-juros-a-menos')}</dt>
            <dd className="text-right">
              <Money
                cents={r.interest_saved}
                tone={r.interest_saved > 0 ? 'positive' : 'neutral'}
              />
            </dd>
            <dt className="text-dim">{t('planejamento-quitacao-meses-a-menos')}</dt>
            <dd className="num text-right">
              {r.feasible && plan.minimum.feasible ? r.months_saved : '—'}
            </dd>
          </>
        ) : null}
      </dl>
    </div>
  );
}

/** `hasLoans`: há contrato com saldo (sem ele, só simula com as dívidas informadas). */
export function PayoffSimulator({ hasLoans }: { hasLoans: boolean }) {
  const [extra, setExtra] = useState<number | null>(null);
  const [drafts, setDrafts] = useState<Draft[]>([]);
  const [view, setView] = useState<'snowball' | 'avalanche'>('avalanche');
  const custom = drafts.map(toCustom).filter((c): c is CustomDebt => c !== null);
  const ready = hasLoans || custom.length > 0;
  const q = useDebtPayoff({ extra_monthly: extra ?? 0, custom }, ready);
  const plan = ready ? q.data : undefined;

  const set = (i: number, patch: Partial<Draft>) =>
    setDrafts((ds) => ds.map((d, k) => (k === i ? { ...d, ...patch } : d)));

  const option = useCallback(
    (theme: ChartTheme) => {
      if (!plan) return {};
      const base = baseOption(theme);
      // O mínimo que nunca quita (juros acima da parcela) não estica o eixo.
      const all = [plan.minimum, plan.snowball, plan.avalanche];
      const shown = all.some((r) => r.feasible) ? all.filter((r) => r.feasible) : all;
      const longest = shown.reduce((a, b) => (b.series.length > a.series.length ? b : a));
      const months = longest.series.map((m) => m.month);
      const line = (r: PayoffResult, color: string, dashed = false) => ({
        name: LABEL[r.strategy],
        type: 'line',
        symbol: 'none',
        color,
        lineStyle: { width: 1.75, ...(dashed ? { type: 'dashed' } : {}) },
        data: months.map((_, i) => r.series[i]?.balance ?? 0),
      });
      return {
        ...base,
        legend: { ...(base.legend as object), show: true, top: 0 },
        grid: { left: 8, right: 16, top: 32, bottom: 8, containLabel: true },
        tooltip: {
          ...(base.tooltip as object),
          trigger: 'axis',
          valueFormatter: (v: number) => tooltipMoney(v, theme),
        },
        xAxis: (() => {
          const axis = categoryAxis(theme, months.map(formatMonthShort));
          return {
            ...axis,
            boundaryGap: false,
            axisLabel: { ...(axis.axisLabel as object), hideOverlap: true },
          };
        })(),
        yAxis: valueAxis(theme, (v) => compactMoney(v, theme)),
        series: [
          line(plan.minimum, theme.textDim, true),
          line(plan.snowball, theme.series[1] ?? theme.accent),
          line(plan.avalanche, theme.series[0] ?? theme.accent),
        ],
      };
    },
    [plan],
  );

  return (
    <InstrumentCard index="06.Q" title={t('planejamento-quitacao-titulo')}>
      <div className="flex flex-col gap-4">
        <p className="text-[13px] text-dim">{t('planejamento-quitacao-intro')}</p>
        <div className="flex flex-wrap items-end gap-4">
          <Field label={t('planejamento-quitacao-extra')}>
            <MoneyInput value={extra} onValueChange={setExtra} className="w-40" />
          </Field>
          <Button
            variant="secondary"
            icon={<Icon name="plus" size={13} />}
            onClick={() => setDrafts((ds) => [...ds, emptyDraft()])}
          >
            {t('planejamento-quitacao-fora-app')}
          </Button>
        </div>
        {drafts.length ? (
          <div className="flex flex-col gap-2">
            {drafts.map((d, i) => (
              <div
                key={i}
                className="grid items-end gap-2 @min-[720px]:grid-cols-[minmax(0,1.4fr)_repeat(3,minmax(0,1fr))_auto]"
              >
                <Field label={t('planejamento-nome')}>
                  <Input
                    value={d.name}
                    onChange={(e) => set(i, { name: e.target.value })}
                    placeholder={t('planejamento-quitacao-exemplo')}
                  />
                </Field>
                <Field label={t('planejamento-quitacao-saldo-devedor')}>
                  <MoneyInput value={d.balance} onValueChange={(v) => set(i, { balance: v })} />
                </Field>
                <Field label={t('planejamento-quitacao-juros-ano')}>
                  <Input
                    numeric
                    inputMode="decimal"
                    value={d.rate}
                    onChange={(e) => set(i, { rate: e.target.value })}
                    suffix="%"
                    placeholder="150"
                  />
                </Field>
                <Field label={t('planejamento-quitacao-minimo-mes')}>
                  <MoneyInput value={d.min} onValueChange={(v) => set(i, { min: v })} />
                </Field>
                <IconButton
                  label={
                    d.name
                      ? t('planejamento-tirar', { nome: d.name })
                      : t('planejamento-quitacao-tirar-esta')
                  }
                  icon={<Icon name="x" size={13} />}
                  onClick={() => setDrafts((ds) => ds.filter((_, k) => k !== i))}
                />
              </div>
            ))}
          </div>
        ) : null}
        {ready ? <InlineError error={q.error} /> : null}
        {plan ? (
          <>
            <div className="grid gap-3 @min-[720px]:grid-cols-3">
              {(['minimum', 'snowball', 'avalanche'] as const).map((s) => (
                <StrategyColumn key={s} plan={plan} s={s} />
              ))}
            </div>
            <p className="text-[13px] text-fg">{plan.explanation}</p>
            <ChartFrame
              index="06.Q1"
              title={t('planejamento-quitacao-grafico')}
              source={t('planejamento-quitacao-fonte', {
                n: plan.debts.length,
                extra: formatMoney(plan.extra_monthly),
              })}
              chart={
                <EChart
                  option={option}
                  height={240}
                  ariaLabel={t('planejamento-quitacao-grafico-aria')}
                />
              }
              table={
                <TechTable
                  label={t('planejamento-quitacao-dividas')}
                  prefsId="Dívidas na simulação"
                  dense
                  rows={plan.debts}
                  getRowId={(d) => d.loan_id ?? d.name}
                  columns={[
                    { key: 'name', header: t('planejamento-quitacao-divida') },
                    {
                      key: 'balance',
                      header: t('planejamento-quitacao-saldo'),
                      numeric: true,
                      cell: (d) => <Money cents={d.balance} />,
                    },
                    {
                      key: 'annual_rate',
                      header: t('planejamento-quitacao-juros-aa'),
                      numeric: true,
                      cell: (d) => t('planejamento-quitacao-taxa', { taxa: Number(d.annual_rate) }),
                    },
                    {
                      key: 'min_payment',
                      header: t('planejamento-quitacao-minimo-col'),
                      numeric: true,
                      cell: (d) => <Money cents={d.min_payment} />,
                    },
                  ]}
                />
              }
            />
            <div className="flex flex-col gap-2">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <h3 className="label-caps text-fg">{t('planejamento-quitacao-ordem')}</h3>
                <Segmented<'snowball' | 'avalanche'>
                  label={t('planejamento-quitacao-estrategia')}
                  size="sm"
                  value={view}
                  onChange={setView}
                  options={[
                    { value: 'avalanche', label: LABEL.avalanche },
                    { value: 'snowball', label: LABEL.snowball },
                  ]}
                />
              </div>
              <TechTable
                label={t('planejamento-quitacao-ordem-de', { estrategia: LABEL[view] })}
                prefsId={`Ordem de quitação: ${PREFS_LABEL[view]}`}
                dense
                rows={plan[view].debts}
                getRowId={(d) => `${d.order}:${d.name}`}
                columns={[
                  { key: 'order', header: '#', width: '40px', numeric: true },
                  { key: 'name', header: t('planejamento-quitacao-divida') },
                  {
                    key: 'paid_off',
                    header: t('planejamento-quitacao-quitada-em'),
                    cell: (d) =>
                      d.paid_off ? formatMonthShort(d.paid_off) : t('planejamento-mais-50-anos'),
                  },
                  {
                    key: 'interest',
                    header: t('planejamento-quitacao-juros-pagos'),
                    numeric: true,
                    cell: (d) => <Money cents={d.interest} />,
                  },
                ]}
              />
            </div>
            <ul className="flex flex-col gap-1 text-[12px] text-dim">
              {plan.notes.map((n) => (
                <li key={n}>{n}</li>
              ))}
              <li>{t('planejamento-quitacao-aviso')}</li>
            </ul>
          </>
        ) : null}
      </div>
    </InstrumentCard>
  );
}
