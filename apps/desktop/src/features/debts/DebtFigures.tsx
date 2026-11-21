/**
 * Figuras do contrato: saldo devedor ao longo do tempo e composição das parcelas
 * (juros × amortização × correção × seguros e taxas), com a visão em tabela.
 */
import { memo, useCallback, useMemo, useState } from 'react';

import type { Installment } from '@/api/generated/Installment';
import type { LoanView } from '@/api/generated/LoanView';
import { ChartFrame, EChart, resolveColor, readChartTheme, type ChartTheme } from '@/charts';
import { LegendItem, Money, TechTable } from '@/design/components';
import { Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { useUi } from '@/state/ui';

import {
  balanceOption,
  breakdownOption,
  PART_LABEL,
  SIMULATED_COLOR,
  visibleParts,
} from './chartOptions';
import {
  balanceSeries,
  breakdown,
  defaultBreakdownMode,
  todayIndex,
  type BalancePoint,
  type BreakdownMode,
  type BreakdownRow,
} from './transform';

export function BalanceFigure({
  index,
  loan,
  installments,
  simulated,
}: {
  index: string;
  loan: LoanView;
  installments: readonly Installment[];
  /** Cronograma com a amortização simulada (linha tracejada). */
  simulated?: readonly Installment[] | null;
}) {
  const today = todayIso();
  const points = useMemo(
    () => balanceSeries(installments, loan.principal, loan.contract_date),
    [installments, loan.principal, loan.contract_date],
  );
  const sim = useMemo(
    () =>
      simulated
        ? balanceSeries(simulated, loan.principal, loan.contract_date).filter(
            (p) => p.date >= today,
          )
        : null,
    [simulated, loan.principal, loan.contract_date, today],
  );
  const option = useCallback(
    (theme: ChartTheme) =>
      balanceOption(theme, { points, todayIndex: todayIndex(points, today), simulated: sim }),
    [points, sim, today],
  );
  const estimated = installments.some((i) => i.estimated && !i.paid);
  const themeId = useUi((s) => s.theme);
  const simSwatch = resolveColor(readChartTheme(themeId, false), SIMULATED_COLOR);
  return (
    <ChartFrame
      index={index}
      title={t('dividas-figura-saldo-titulo')}
      source={
        loan.indexer !== 'none'
          ? t('dividas-figura-saldo-fonte-indice')
          : t('dividas-figura-saldo-fonte')
      }
      actions={
        sim ? (
          <span className="flex gap-3">
            <LegendItem>{t('dividas-saldo-devedor')}</LegendItem>
            <LegendItem color={simSwatch} dashed>
              {t('dividas-com-simulada')}
            </LegendItem>
          </span>
        ) : estimated ? (
          <span className="label-caps italic">{t('dividas-estimado')}</span>
        ) : null
      }
      chart={
        <EChart
          option={option}
          height={260}
          ariaLabel={t(sim ? 'dividas-figura-saldo-aria-simulada' : 'dividas-figura-saldo-aria', {
            nome: loan.name,
          })}
        />
      }
      table={
        <TechTable<BalancePoint>
          label={t('dividas-figura-saldo-tabela')}
          prefsId="Saldo devedor por vencimento"
          dense
          rows={points}
          getRowId={(r) => r.date}
          columns={[
            { key: 'date', header: t('dividas-data'), cell: (r) => formatDate(r.date) },
            {
              key: 'balance',
              header: t('dividas-figura-saldo-col'),
              numeric: true,
              cell: (r) => <Money cents={r.balance} estimated={r.estimated} />,
            },
          ]}
        />
      }
    />
  );
}

export const BreakdownFigure = memo(function BreakdownFigure({
  index,
  loan,
  installments,
}: {
  index: string;
  loan: LoanView;
  installments: readonly Installment[];
}) {
  const [mode, setMode] = useState<BreakdownMode>(() => defaultBreakdownMode(installments.length));
  const rows = useMemo(() => breakdown(installments, mode), [installments, mode]);
  const parts = visibleParts(rows);
  const option = useCallback(
    (theme: ChartTheme) => breakdownOption(theme, { rows, mode }),
    [rows, mode],
  );
  return (
    <ChartFrame
      index={index}
      title={t('dividas-figura-composicao-titulo', { modo: mode })}
      source={
        parts.includes('adjustment')
          ? t('dividas-figura-composicao-fonte-correcao')
          : t('dividas-figura-composicao-fonte')
      }
      actions={
        <Segmented<BreakdownMode>
          size="sm"
          label={t('dividas-figura-composicao-agrupar')}
          value={mode}
          onChange={setMode}
          options={[
            { value: 'parcela', label: t('dividas-parcela') },
            { value: 'ano', label: t('dividas-ano') },
          ]}
        />
      }
      chart={
        <EChart
          option={option}
          height={300}
          ariaLabel={t('dividas-figura-composicao-aria', {
            modo: mode,
            nome: loan.name,
            partes: parts.map((k) => PART_LABEL[k].toLowerCase()).join(', '),
          })}
        />
      }
      table={
        <TechTable<BreakdownRow>
          label={t('dividas-figura-composicao-tabela', { modo: mode })}
          prefsId={mode === 'ano' ? 'Composição por ano' : 'Composição por parcela'}
          dense
          rows={rows}
          getRowId={(r) => r.key}
          columns={[
            {
              key: 'label',
              header: mode === 'ano' ? t('dividas-ano') : t('dividas-parcela'),
              cell: (r) => (mode === 'ano' ? r.label : `${r.label} · ${formatDate(r.date)}`),
            },
            ...parts.map((k) => ({
              key: k,
              header: PART_LABEL[k],
              numeric: true,
              cell: (r: BreakdownRow) => <Money cents={r[k]} />,
            })),
            {
              key: 'total',
              header: mode === 'ano' ? t('dividas-figura-pago-no-ano') : t('dividas-parcela'),
              numeric: true,
              cell: (r) => <Money cents={r.total} />,
            },
          ]}
        />
      }
    />
  );
});
