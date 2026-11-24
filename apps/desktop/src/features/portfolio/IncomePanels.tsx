/**
 * Proventos (lista por período) e renda passiva: últimos 12 meses por mês, yield,
 * meta de renda mensal e o patrimônio necessário para chegar lá no yield atual.
 */
import { useEffect, useMemo, useState } from 'react';

import { usePassiveIncome, usePortfolioSummary } from '@/api/hooks/portfolio';
import {
  Field,
  InstrumentCard,
  Measure,
  Money,
  MoneyInput,
  TechTable,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';

import { PassiveIncomeChart } from './charts/PassiveIncomeChart';
import { PeriodSelect, periodRange, usePeriod } from './components/PeriodSelect';
import { Pct } from './components/Pct';
import { ErrorBlock, LoadingBlock, Notice } from './components/States';
import { maskAmount } from './incomeMasks';
import { IncomeLedgerSummary } from './IncomeLedger';
import { IncomeByMonth } from './PortfolioCards';
import { classLabel } from './labels';
import { IncomeTable } from './RecordsPanels';
import { capitalGap } from './transform';
import type { PassiveIncomeAsset } from '@/api/generated/PassiveIncomeAsset';

export function IncomePanel() {
  const [period, setPeriod] = usePeriod('proventos', '12m');
  const range = periodRange(period);
  return (
    <div className="flex flex-col gap-3">
      <IncomeLedgerSummary />
      <PeriodSelect scope="proventos" value={period} onChange={setPeriod} />
      <IncomeByMonth />
      <IncomeTable
        {...(range.fromDate ? { from: range.fromDate } : {})}
        {...(range.toDate ? { to: range.toDate } : {})}
      />
    </div>
  );
}

const assetColumns = (): TechColumn<PassiveIncomeAsset>[] => [
  {
    key: 'asset',
    header: t('carteira-col-ativo'),
    cell: (a) => (
      <span className="flex flex-col leading-tight">
        <span className="font-medium">{a.asset.ticker ?? a.asset.name}</span>
        <span className="font-sans text-[11.5px] text-dim">
          {classLabel(a.asset.class, a.asset.class_label)}
        </span>
      </span>
    ),
  },
  {
    key: 'last',
    header: t('carteira-col-12-meses'),
    numeric: true,
    cell: (a) => <Money cents={a.last_12m} />,
    sortValue: (a) => a.last_12m,
  },
  {
    key: 'yield',
    header: t('carteira-col-yield-12m'),
    numeric: true,
    cell: (a) => <Pct value={a.yield_12m} decimals={2} />,
    sortValue: (a) => a.yield_12m,
  },
];

export function PassiveIncomePanel() {
  const [input, setInput] = useState<number | null>(null);
  const [target, setTarget] = useState<number | null>(null);
  useEffect(() => {
    const id = setTimeout(() => setTarget(input && input > 0 ? input : null), 300);
    return () => clearTimeout(id);
  }, [input]);
  const q = usePassiveIncome(target ? { target_monthly: target } : {});
  const summary = usePortfolioSummary({ deflator: 'none' });
  const columns = useMemo(() => assetColumns(), []);
  if (q.isPending) return <LoadingBlock label={t('carteira-somando-proventos')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const pi = q.data;
  const gap = capitalGap(pi.capital_needed, summary.data?.market_value ?? 0);
  const progress = target && pi.monthly_avg ? Math.min(1, pi.monthly_avg / target) : null;
  return (
    <div className="flex flex-col gap-5">
      <div className="grid gap-4 @min-[530px]:grid-cols-2 @min-[1040px]:grid-cols-4">
        <InstrumentCard index="01" title={t('carteira-ultimos-12-meses')}>
          <Measure
            size="md"
            label={t('carteira-proventos-liquidos')}
            value={<Money cents={pi.last_12m} />}
            changeKey={pi.last_12m}
          />
        </InstrumentCard>
        <InstrumentCard index="02" title={t('carteira-media-mensal')}>
          <Measure
            size="md"
            label={t('carteira-por-mes')}
            value={<Money cents={pi.monthly_avg} />}
            changeKey={pi.monthly_avg}
          />
        </InstrumentCard>
        <InstrumentCard index="03" title={t('carteira-yield-carteira')}>
          <Measure
            size="md"
            label={t('carteira-yield-formula')}
            value={<Pct value={pi.portfolio_yield_12m} decimals={2} />}
            changeKey={pi.portfolio_yield_12m ?? 0}
          />
        </InstrumentCard>
        <InstrumentCard
          index="04"
          title={t('carteira-meta-renda')}
          footer={
            target
              ? [
                  {
                    label: t('carteira-patrimonio-necessario'),
                    value: pi.capital_needed == null ? '—' : <Money cents={pi.capital_needed} />,
                  },
                  { label: t('carteira-falta'), value: gap == null ? '—' : <Money cents={gap} /> },
                ]
              : undefined
          }
        >
          <div className="flex flex-col gap-2">
            <Field label={t('carteira-quanto-receber')}>
              <MoneyInput value={input} onValueChange={setInput} placeholder={maskAmount(0)} />
            </Field>
            {progress != null ? (
              <div className="flex items-center gap-2" aria-label={t('carteira-progresso-meta')}>
                <div className="h-[5px] flex-1 rounded-full bg-line">
                  <div
                    className="h-full rounded-full bg-accent"
                    style={{ width: `${progress * 100}%` }}
                  />
                </div>
                <Pct value={progress} decimals={0} className="text-[11px] text-dim" />
              </div>
            ) : null}
          </div>
        </InstrumentCard>
      </div>
      {target && pi.capital_needed == null ? (
        <Notice tone="warning" title={t('carteira-sem-yield')}>
          {t('carteira-sem-yield-texto')}
        </Notice>
      ) : null}
      <div className="grid gap-5 @min-[1040px]:grid-cols-[minmax(0,7fr)_minmax(0,5fr)]">
        <PassiveIncomeChart index="05" data={pi} />
        <div className="flex flex-col gap-3">
          <h3 className="label-caps text-fg">{t('carteira-por-ativo-12m')}</h3>
          <div className="rounded-card border border-line bg-surface">
            <TechTable
              label={t('carteira-renda-por-ativo')}
              prefsId="Renda passiva por ativo"
              rows={pi.by_asset}
              columns={columns}
              getRowId={(a) => a.asset.id}
              defaultSort={{ key: 'last', direction: 'desc' }}
              maxHeight="300px"
              dense
              empty={t('carteira-sem-proventos-12m')}
            />
          </div>
        </div>
      </div>
      <p className="text-[12px] text-dim">{t('carteira-patrimonio-necessario-texto')}</p>
    </div>
  );
}
