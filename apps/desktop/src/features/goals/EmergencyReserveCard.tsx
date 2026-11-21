/** Calculadora de reserva de emergência com os gastos essenciais reais (`goals.emergency_reserve`). */
import { useState } from 'react';

import type { EmergencyReserve } from '@/api/generated/EmergencyReserve';

import { useEmergencyReserve, useGoals, useUpdateGoal } from '@/api/hooks/goals';
import { Button, InstrumentCard, Measure, Money, Private, toast } from '@/design/components';
import { Check } from '@/features/portfolio/components/Fields';
import { QueryView, Segmented } from '@/features/reports/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatMonthShort, formatNumber, formatPercent } from '@/lib/format';

import { GoalDialog, ProgressMeter } from './GoalsPanel';
import { reserveView } from './model';

const TARGETS = ['3', '6', '9', '12'] as const;
const BACK = ['3', '6', '12'] as const;

export function EmergencyReserveCard() {
  const [target, setTarget] = useState<(typeof TARGETS)[number]>('6');
  const [back, setBack] = useState<(typeof BACK)[number]>('6');
  const [creating, setCreating] = useState(false);
  // `null`: o padrão do núcleo (inclui quando há gastos anuais).
  const [annual, setAnnual] = useState<boolean | null>(null);
  const reserve = useEmergencyReserve({
    target_months: Number(target),
    months_back: Number(back),
    ...(annual == null ? {} : { include_annual: annual }),
  });
  const goals = useGoals({});
  const update = useUpdateGoal();
  return (
    <InstrumentCard
      index="R"
      title={t('metas-tipo-emergencia')}
      footer={
        reserve.data
          ? [
              {
                label: t('metas-reserva-base'),
                value:
                  reserve.data.basis === 'cash'
                    ? t('metas-reserva-caixa')
                    : t('metas-reserva-competencia'),
              },
              {
                label: t('metas-reserva-meses-analisados'),
                value: reserve.data.months.length
                  ? t('metas-reserva-meses-intervalo', {
                      de: formatMonthShort(reserve.data.months[0]!),
                      ate: formatMonthShort(reserve.data.months[reserve.data.months.length - 1]!),
                    })
                  : '—',
              },
              { label: t('metas-reserva-fonte'), value: t('metas-reserva-fonte-valor') },
            ]
          : undefined
      }
    >
      <div className="mb-4 flex flex-wrap items-center gap-x-6 gap-y-3">
        <div className="flex items-center gap-2">
          <span className="label-caps">{t('metas-reserva-meses-cobertura')}</span>
          <Segmented
            label={t('metas-reserva-meses-cobertura')}
            value={target}
            onChange={setTarget}
            options={TARGETS.map((m) => ({ value: m, label: m }))}
          />
        </div>
        <div className="flex items-center gap-2">
          <span className="label-caps">{t('metas-reserva-media-de')}</span>
          <Segmented
            label={t('metas-reserva-meses-na-media')}
            value={back}
            onChange={setBack}
            options={BACK.map((m) => ({
              value: m,
              label: t('metas-reserva-n-meses', { n: Number(m) }),
            }))}
          />
        </div>
      </div>
      <QueryView query={reserve}>
        {(r) => {
          const v = reserveView(r);
          const goal = goals.data?.find((g) => g.id === r.goal_id);
          return (
            <div className="flex flex-col gap-4">
              <div className="grid gap-5 @min-[400px]:grid-cols-2 @min-[790px]:grid-cols-4">
                <Measure
                  size="md"
                  label={t('metas-reserva-meta')}
                  value={<Money cents={r.target_amount} estimated={r.estimated} />}
                  hint={
                    r.include_annual
                      ? t('metas-reserva-meta-dica-custo', { n: r.target_months })
                      : t('metas-reserva-meta-dica', { n: r.target_months })
                  }
                />
                <Measure
                  size="md"
                  label={t('metas-reserva-guardado')}
                  value={<Money cents={r.current_amount} />}
                  hint={goal ? goal.name : t('metas-reserva-guardado-dica')}
                />
                <Measure
                  size="md"
                  label={t('metas-reserva-cobertura')}
                  value={
                    <Private>
                      {v.coverage == null
                        ? '—'
                        : t('metas-reserva-cobertura-valor', {
                            n: v.coverage,
                            valor: formatNumber(v.coverage, 1),
                          })}
                    </Private>
                  }
                  hint={t('metas-reserva-cobertura-dica')}
                />
                <Measure size="md" label={t('metas-falta')} value={<Money cents={v.missing} />} />
              </div>
              <ProgressMeter
                value={v.progress}
                label={t('metas-reserva-progresso')}
                tone={
                  v.status === 'ok'
                    ? 'var(--positive)'
                    : v.status === 'low'
                      ? 'var(--warning)'
                      : 'var(--accent)'
                }
              />
              <dl className="grid gap-x-6 gap-y-1 text-[12.5px] @min-[400px]:grid-cols-2">
                {r.include_annual ? (
                  <>
                    <div className="flex justify-between gap-3">
                      <dt className="text-dim">{t('metas-reserva-custo-mensal')}</dt>
                      <dd className="num text-fg">
                        <Money cents={r.monthly_cost} />
                      </dd>
                    </div>
                    <div className="flex justify-between gap-3">
                      <dt className="text-dim">{t('metas-reserva-essenciais-regulares')}</dt>
                      <dd className="num text-fg">
                        <Money cents={r.avg_essential_regular_monthly} />
                      </dd>
                    </div>
                    <div className="flex justify-between gap-3">
                      <dt className="text-dim">{t('metas-reserva-provisao-anual')}</dt>
                      <dd className="num text-fg">
                        <Money cents={r.annual.essential_monthly} />
                      </dd>
                    </div>
                  </>
                ) : null}
                <div className="flex justify-between gap-3">
                  <dt className="text-dim">{t('metas-reserva-gasto-essencial')}</dt>
                  <dd className="num text-fg">
                    <Money cents={r.avg_essential_monthly} />
                  </dd>
                </div>
                <div className="flex justify-between gap-3">
                  <dt className="text-dim">{t('metas-reserva-gasto-total')}</dt>
                  <dd className="num text-fg">
                    <Money cents={r.avg_total_expense_monthly} />
                  </dd>
                </div>
                <div className="flex justify-between gap-3">
                  <dt className="text-dim">{t('metas-reserva-essenciais-total')}</dt>
                  <dd className="num text-fg">
                    <Private>
                      {r.avg_total_expense_monthly > 0
                        ? formatPercent(r.avg_essential_monthly / r.avg_total_expense_monthly, {
                            decimals: 0,
                          })
                        : '—'}
                    </Private>
                  </dd>
                </div>
                <div className="flex justify-between gap-3">
                  <dt className="text-dim">{t('metas-reserva-guardado-meta')}</dt>
                  <dd className="num text-fg">
                    <Private>{formatPercent(v.progress, { decimals: 0 })}</Private>
                  </dd>
                </div>
              </dl>
              <AnnualExpensesBox
                reserve={r}
                include={r.include_annual}
                onInclude={(v) => setAnnual(v)}
              />
              {v.status === 'empty' ? (
                <p className="text-[12.5px] text-dim">{t('metas-reserva-sem-meses')}</p>
              ) : !goal ? (
                <div className="flex flex-wrap items-center gap-3">
                  <Button variant="primary" size="sm" onClick={() => setCreating(true)}>
                    {t('metas-reserva-criar')}
                  </Button>
                  <span className="text-[12px] text-dim">{t('metas-reserva-sem-meta')}</span>
                </div>
              ) : goal.target_amount !== r.target_amount ? (
                <div className="flex flex-wrap items-center gap-3">
                  <Button
                    size="sm"
                    loading={update.isPending}
                    onClick={() =>
                      update.mutate(
                        { id: goal.id, target_amount: r.target_amount },
                        {
                          onSuccess: () =>
                            toast({ tone: 'success', title: t('metas-reserva-atualizada') }),
                        },
                      )
                    }
                  >
                    {withSlots(t('metas-reserva-atualizar', { valor: slot(0) }), [
                      <Money key="alvo" cents={r.target_amount} />,
                    ])}
                  </Button>
                  <span className="text-[12px] text-dim">
                    {withSlots(t('metas-reserva-meta-em', { nome: goal.name, valor: slot(0) }), [
                      <Money key="meta" cents={goal.target_amount} />,
                    ])}
                  </span>
                </div>
              ) : (
                <p className="text-[12px] text-positive">
                  {t('metas-reserva-ja-usa', { nome: goal.name })}
                </p>
              )}
              {creating ? (
                <GoalDialog
                  open={creating}
                  onOpenChange={setCreating}
                  initial={{
                    kind: 'emergency',
                    name: t('metas-tipo-emergencia'),
                    target: r.target_amount,
                  }}
                />
              ) : null}
            </div>
          );
        }}
      </QueryView>
    </InstrumentCard>
  );
}

/** Gastos anuais dos últimos 12 meses e a escolha de contá-los pela provisão mensal. */
function AnnualExpensesBox({
  reserve: r,
  include,
  onInclude,
}: {
  reserve: EmergencyReserve;
  include: boolean;
  onInclude: (v: boolean) => void;
}) {
  const a = r.annual;
  if (!a.complete_year)
    return <p className="text-[12px] text-dim">{t('metas-reserva-anuais-sem-ano')}</p>;
  return (
    <div className="flex flex-col gap-2 border-t border-line pt-3">
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
        <span className="label-caps">{t('metas-reserva-anuais-titulo')}</span>
        {a.items.length ? (
          <Check checked={include} onChange={onInclude} label={t('metas-reserva-anuais')} />
        ) : null}
      </div>
      {a.items.length ? (
        <>
          <p className="text-[12px] text-dim">{t('metas-reserva-anuais-dica')}</p>
          <ul aria-label={t('metas-reserva-anuais-tabela')} className="flex flex-col text-[12.5px]">
            {a.items.map((i) => (
              <li
                key={i.category_id}
                className="flex items-baseline justify-between gap-3 border-b border-line/60 py-1 last:border-0"
              >
                <span className="min-w-0 truncate text-fg">
                  {i.category}
                  <span className="text-dim">
                    {' · '}
                    {i.months.map(formatMonthShort).join(', ')}
                    {i.essential ? '' : ` · ${t('metas-reserva-anuais-nao-essencial')}`}
                  </span>
                </span>
                <span className="num shrink-0 text-fg">
                  <Money cents={i.total} />{' '}
                  <span className="text-dim">
                    {withSlots(t('metas-reserva-anuais-mes', { valor: slot(0) }), [
                      <Money key="mes" cents={i.monthly} />,
                    ])}
                  </span>
                </span>
              </li>
            ))}
          </ul>
        </>
      ) : (
        <p className="text-[12px] text-dim">{t('metas-reserva-anuais-nenhum')}</p>
      )}
    </div>
  );
}
