/**
 * Relatórios › IR: apuração mensal de renda variável, DARFs e o relatório anual para
 * a declaração, sempre com o aviso de que é uma apuração auxiliar.
 *
 * Estado na URL (junto de `v=ir`): `ano`, `mes` (`YYYY-MM`), `titular` e `ir`
 * (`apuracao` | `darf` | `declaracao`).
 */
import { useSearchParams } from 'react-router';

import { useMembers } from '@/api/hooks/household';
import { useSupportEventWhen } from '@/api/hooks/support';
import { useDarfs, useTaxYear } from '@/api/hooks/tax';
import { TaxNotice } from '@/features/terms/Disclaimer';
import { Field, InstrumentCard, Measure, Money, Select } from '@/design/components';
import { Notice } from '@/features/portfolio/components/States';
import { QueryState, Segmented } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatMonthLong, todayIso } from '@/lib/format';

import { DarfPanel } from './DarfPanel';
import { disclaimer } from './labels';
import { MonthDetail, MonthsTable } from './MonthPanels';
import { activeMonths, darfOverdue, defaultMonth, yearOptions } from './transform';
import { YearReport } from './YearReport';

type Section = 'apuracao' | 'darf' | 'declaracao';
const SECTIONS: readonly { value: Section; label: string }[] = [
  {
    value: 'apuracao',
    get label() {
      return t('ir-secao-apuracao');
    },
  },
  {
    value: 'darf',
    get label() {
      return t('ir-secao-darf');
    },
  },
  {
    value: 'declaracao',
    get label() {
      return t('ir-secao-declaracao');
    },
  },
];
const FAMILY = '__familia__';

export function TaxPanel() {
  const [params, setParams] = useSearchParams();
  const today = todayIso();
  const years = yearOptions(today);
  const yearParam = Number(params.get('ano'));
  const year = years.includes(yearParam) ? yearParam : years[0]!;
  const memberId = params.get('titular') || null;
  const section = SECTIONS.find((s) => s.value === params.get('ir'))?.value ?? 'apuracao';
  const members = useMembers().data ?? [];
  const report = useTaxYear({ year, ...(memberId ? { member_id: memberId } : {}) });
  const darfs = useDarfs({ year, ...(memberId ? { member_id: memberId } : {}) });

  const set = (patch: Record<string, string | null>) =>
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        for (const [k, v] of Object.entries(patch)) {
          if (v == null) next.delete(k);
          else next.set(k, v);
        }
        return next;
      },
      { replace: true },
    );

  const data = report.data;
  // O relatório anual do IR visto é um dos momentos do lembrete de apoio.
  useSupportEventWhen(section === 'declaracao' && Boolean(data), 'tax_report', String(year));
  const months = data ? activeMonths(data) : [];
  const selectedMonth =
    months.find((m) => m.month === params.get('mes'))?.month ?? (data ? defaultMonth(data) : null);
  const month = data?.months.find((m) => m.month === selectedMonth) ?? null;
  const toPay = months.filter((m) => m.status === 'to_pay' && !m.darf_id);
  const overdue = (darfs.data ?? []).filter((d) => darfOverdue(d, today));

  return (
    <div className="flex flex-col gap-5">
      <Notice title={t('ir-aviso-titulo')}>
        {data?.disclaimer ?? disclaimer()} {t('ir-aviso-regras')}
      </Notice>

      <div className="flex flex-wrap items-end gap-4">
        <Field label={t('ir-ano-calendario')} className="w-40">
          <Select
            value={String(year)}
            onValueChange={(v) => v && set({ ano: v, mes: null })}
            options={years.map((y) => ({ value: String(y), label: String(y) }))}
          />
        </Field>
        <Field label={t('ir-membro')} className="w-48">
          <Select
            value={memberId ?? FAMILY}
            onValueChange={(v) => v && set({ titular: v === FAMILY ? null : v, mes: null })}
            options={[
              { value: FAMILY, label: t('ir-familia-toda') },
              ...members.map((m) => ({ value: m.id, label: m.name })),
            ]}
          />
        </Field>
        <Segmented<Section>
          label={t('ir-secao')}
          value={section}
          onChange={(v) => set({ ir: v === 'apuracao' ? null : v })}
          options={SECTIONS}
          className="ml-auto"
        />
      </div>

      <QueryState
        loading={report.isPending}
        error={report.error}
        onRetry={() => void report.refetch()}
        label={t('ir-apurando')}
      >
        {data ? (
          <>
            <div className="grid gap-4 @min-[530px]:grid-cols-3">
              <InstrumentCard
                index="09.1"
                title={t('ir-imposto-em', { ano: String(year) })}
                headingLevel={3}
              >
                <Measure
                  size="md"
                  label={t('ir-renda-variavel')}
                  value={<Money cents={data.total_tax} />}
                  hint={
                    <>
                      {withSlots(t('ir-darfs-total', { valor: slot(0) }), [
                        <Money key="v" cents={data.total_darf} />,
                      ])}
                    </>
                  }
                />
              </InstrumentCard>
              <InstrumentCard index="09.2" title={t('ir-a-pagar')} headingLevel={3}>
                <Measure
                  size="md"
                  label={
                    toPay.length > 0
                      ? t('ir-meses-sem-darf-n', { n: toPay.length })
                      : t('ir-meses-sem-darf')
                  }
                  value={<Money cents={toPay.reduce((s, m) => s + m.darf_amount, 0)} />}
                  hint={
                    overdue.length > 0 ? (
                      <span className="text-negative">
                        {t('ir-darfs-vencidos', { n: overdue.length })}
                      </span>
                    ) : undefined
                  }
                />
              </InstrumentCard>
              <InstrumentCard index="09.3" title={t('ir-prejuizos')} headingLevel={3}>
                <Measure
                  size="md"
                  label={t('ir-somando-grupos')}
                  value={<Money cents={data.loss_common + data.loss_day_trade + data.loss_fii} />}
                  hint={t('ir-fim-periodo')}
                />
              </InstrumentCard>
            </div>

            {section === 'apuracao' ? (
              <div className="flex flex-col gap-5">
                <MonthsTable
                  months={months}
                  selected={selectedMonth}
                  onSelect={(m) => set({ mes: m })}
                />
                {month ? (
                  <MonthDetail
                    key={month.month}
                    month={month}
                    memberId={memberId}
                    onShowDarf={() => set({ ir: 'darf' })}
                  />
                ) : null}
              </div>
            ) : section === 'darf' ? (
              <QueryState
                loading={darfs.isPending}
                error={darfs.error}
                label={t('ir-carregando-darfs')}
              >
                <DarfPanel darfs={darfs.data ?? []} />
              </QueryState>
            ) : (
              <YearReport data={data} memberId={memberId} />
            )}
            {section === 'apuracao' && toPay.length > 0 && month?.month !== toPay.at(-1)?.month ? (
              <p className="text-[12.5px] text-dim">
                {t('ir-sem-darf-em', {
                  meses: toPay.map((m) => formatMonthLong(m.month)).join(', '),
                })}
              </p>
            ) : null}
          </>
        ) : null}
      </QueryState>
      <TaxNotice
        source={t('ir-fonte-regras', { ano: String(year) })}
        date={String(year)}
        country="BR"
      />
    </div>
  );
}
