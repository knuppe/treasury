/**
 * Figuras do bem: valor ao longo do tempo (valor estimado × custo × avaliações, um eixo) e
 * patrimônio do bem × saldo devedor empilhados (a soma é o valor), com a visão em tabela.
 */
import { useCallback, useMemo } from 'react';

import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { PhysicalHistoryPoint } from '@/api/generated/PhysicalHistoryPoint';
import {
  ChartFrame,
  EChart,
  readChartTheme,
  resolveColor,
  StackedArea,
  type ChartTheme,
} from '@/charts';
import { LegendItem, Money, TechTable } from '@/design/components';
import { t } from '@/i18n';
import { formatMonthShort } from '@/lib/format';
import { useUi } from '@/state/ui';

import { APPRAISAL_COLOR, COST_COLOR, VALUE_COLOR, valueOption } from './chartOptions';
import { equityModel, valueChartData } from './transform';

function useSwatch(ref: string): string {
  const themeId = useUi((s) => s.theme);
  return resolveColor(readChartTheme(themeId, false), ref);
}

export function ValueFigure({
  index,
  asset,
  history,
}: {
  index: string;
  asset: PhysicalAssetView;
  history: readonly PhysicalHistoryPoint[];
}) {
  const data = useMemo(
    () => valueChartData(history, asset.valuations),
    [history, asset.valuations],
  );
  const option = useCallback((theme: ChartTheme) => valueOption(theme, data), [data]);
  const estimated = history.some((p) => p.estimated);
  const value = useSwatch(VALUE_COLOR);
  const cost = useSwatch(COST_COLOR);
  const appraisal = useSwatch(APPRAISAL_COLOR);
  return (
    <ChartFrame
      index={index}
      title={t('bens-fig-valor-titulo')}
      source={estimated ? t('bens-fig-valor-fonte-estimado') : t('bens-fig-valor-fonte')}
      actions={
        <span className="flex flex-wrap gap-3">
          <LegendItem color={value}>{t('bens-valor')}</LegendItem>
          <LegendItem color={cost} dashed>
            {t('bens-custo')}
          </LegendItem>
          {data.appraisals.length > 0 ? (
            <LegendItem color={appraisal}>{t('bens-avaliacoes')}</LegendItem>
          ) : null}
        </span>
      }
      chart={
        <EChart
          option={option}
          height={260}
          ariaLabel={t('bens-fig-valor-aria', { nome: asset.name, n: data.appraisals.length })}
        />
      }
      table={
        <TechTable<PhysicalHistoryPoint>
          label={t('bens-fig-valor-tabela', { nome: asset.name })}
          prefsId={`Valor mês a mês · ${asset.name}`}
          dense
          rows={history}
          getRowId={(r) => r.month}
          columns={[
            { key: 'month', header: t('bens-mes'), cell: (r) => formatMonthShort(r.month) },
            {
              key: 'cost',
              header: t('bens-custo'),
              numeric: true,
              cell: (r) => <Money cents={r.cost_basis} />,
            },
            {
              key: 'value',
              header: t('bens-valor'),
              numeric: true,
              cell: (r) => <Money cents={r.value} estimated={r.estimated} />,
            },
            {
              key: 'appraisal',
              header: t('bens-avaliacao'),
              numeric: true,
              cell: (r) => (r.appraisal != null ? <Money cents={r.appraisal} /> : '—'),
            },
          ]}
        />
      }
    />
  );
}

export function EquityFigure({
  index,
  asset,
  history,
}: {
  index: string;
  asset: PhysicalAssetView;
  history: readonly PhysicalHistoryPoint[];
}) {
  const input = useMemo(() => equityModel(history, asset.currency), [history, asset.currency]);
  return (
    <ChartFrame
      index={index}
      title={t('bens-fig-patrimonio-titulo')}
      source={t('bens-fig-patrimonio-fonte')}
      chart={
        <StackedArea
          input={input}
          height={260}
          ariaLabel={t('bens-fig-patrimonio-aria', { nome: asset.name })}
        />
      }
      table={
        <TechTable<PhysicalHistoryPoint>
          label={t('bens-fig-patrimonio-tabela', { nome: asset.name })}
          prefsId={`Patrimônio do bem mês a mês · ${asset.name}`}
          dense
          rows={history}
          getRowId={(r) => r.month}
          columns={[
            { key: 'month', header: t('bens-mes'), cell: (r) => formatMonthShort(r.month) },
            {
              key: 'value',
              header: t('bens-valor'),
              numeric: true,
              cell: (r) => <Money cents={r.value} />,
            },
            {
              key: 'loan',
              header: t('bens-saldo-devedor'),
              numeric: true,
              cell: (r) => <Money cents={r.loan_balance} />,
            },
            {
              key: 'equity',
              header: t('bens-patrimonio'),
              numeric: true,
              cell: (r) => <Money cents={r.equity} />,
            },
          ]}
        />
      }
    />
  );
}
