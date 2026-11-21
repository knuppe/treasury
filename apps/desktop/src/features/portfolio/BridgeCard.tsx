/**
 * "De onde veio o resultado" (`portfolio.bridge`): do valor no início ao valor no fim do
 * período, em dinheiro, linha a linha (preço com o câmbio, proventos, taxas, impostos,
 * aportes e retiradas), o resultado e a perda para a inflação. Por ativo na ordem do núcleo
 * (desempenho passado, sem classificação de "melhor").
 */
import { useMemo, useState } from 'react';

import type { BridgeAsset } from '@/api/generated/BridgeAsset';
import type { BridgeLines } from '@/api/generated/BridgeLines';
import type { PerformanceArgs } from '@/api/generated/PerformanceArgs';
import { useBridge } from '@/api/hooks/portfolio';
import { Figure, Money, TechTable, type TechColumn } from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatMoney } from '@/lib/format';
import { usePrivacy } from '@/state/ui';

import { BridgeChart } from './charts/BridgeChart';
import { PeriodSelect, periodRange, usePeriod } from './components/PeriodSelect';
import { ErrorBlock, LoadingBlock } from './components/States';
import { bridgeSteps, type BridgeStepKey } from './transform';
import { useDeflatorLabel } from './useDeflatorLabel';

const STEP_LABEL: Record<BridgeStepKey, () => string> = {
  inicio: () => t('carteira-ponte-inicio'),
  preco: () => t('carteira-ponte-preco'),
  proventos: () => t('carteira-ponte-proventos'),
  taxas: () => t('carteira-ponte-taxas'),
  impostos: () => t('carteira-ponte-impostos'),
  aportes: () => t('carteira-ponte-aportes'),
  retiradas: () => t('carteira-ponte-retiradas'),
  fim: () => t('carteira-ponte-fim'),
};

/** Linhas por ativo, com as mesmas colunas da ponte (recolhidas até a pessoa abrir). */
function AssetsTable({ rows, currency }: { rows: BridgeAsset[]; currency: string }) {
  const money =
    (pick: (l: BridgeLines) => number, signed = false) =>
    (r: BridgeAsset) => <Money cents={pick(r.lines)} currency={currency} signed={signed} />;
  const hasFx = rows.some((r) => r.lines.fx_gain !== 0);
  const columns: TechColumn<BridgeAsset>[] = [
    {
      key: 'asset',
      header: t('carteira-ativo'),
      cell: (r) => r.asset.ticker ?? r.asset.name,
      sortValue: (r) => r.asset.ticker ?? r.asset.name,
      hideable: false,
    },
    {
      key: 'start',
      header: t('carteira-ponte-inicio'),
      numeric: true,
      cell: money((l) => l.start_value),
      sortValue: (r) => r.lines.start_value,
    },
    {
      key: 'price',
      header: t('carteira-ponte-preco'),
      numeric: true,
      cell: money((l) => l.price_gain, true),
      sortValue: (r) => r.lines.price_gain,
    },
    ...(hasFx
      ? [
          {
            key: 'fx',
            header: t('carteira-ponte-cambio-coluna'),
            numeric: true,
            cell: money((l) => l.fx_gain, true),
            sortValue: (r: BridgeAsset) => r.lines.fx_gain,
          },
        ]
      : []),
    {
      key: 'income',
      header: t('carteira-ponte-proventos'),
      numeric: true,
      cell: money((l) => l.income, true),
      sortValue: (r) => r.lines.income,
    },
    {
      key: 'fees',
      header: t('carteira-ponte-taxas'),
      numeric: true,
      cell: money((l) => -l.fees, true),
      sortValue: (r) => r.lines.fees,
    },
    {
      key: 'taxes',
      header: t('carteira-ponte-impostos'),
      numeric: true,
      cell: money((l) => -l.taxes, true),
      sortValue: (r) => r.lines.taxes,
    },
    {
      key: 'contributions',
      header: t('carteira-ponte-aportes'),
      numeric: true,
      cell: money((l) => l.contributions, true),
      sortValue: (r) => r.lines.contributions,
    },
    {
      key: 'withdrawals',
      header: t('carteira-ponte-retiradas'),
      numeric: true,
      cell: money((l) => -l.withdrawals, true),
      sortValue: (r) => r.lines.withdrawals,
    },
    {
      key: 'end',
      header: t('carteira-ponte-fim'),
      numeric: true,
      cell: money((l) => l.end_value),
      sortValue: (r) => r.lines.end_value,
    },
    {
      key: 'delta',
      header: t('carteira-ponte-resultado-curto'),
      numeric: true,
      cell: (r) => <Money cents={r.lines.delta} currency={currency} tone="auto" />,
      sortValue: (r) => r.lines.delta,
    },
  ];
  return (
    <TechTable
      label={t('carteira-ponte-tabela-ativos')}
      prefsId="Ponte do resultado por ativo"
      columns={columns}
      rows={rows}
      getRowId={(r) => r.asset.id}
      dense
      minWidth="880px"
    />
  );
}

function BridgeBody({ index, args }: { index: string; args: PerformanceArgs }) {
  const q = useBridge(args);
  const deflatorLabel = useDeflatorLabel();
  const privacy = usePrivacy();
  const [open, setOpen] = useState(false);
  const data = q.data;
  const steps = useMemo(() => (data ? bridgeSteps(data.lines) : []), [data]);

  if (q.isPending) return <LoadingBlock label={t('carteira-ponte-calculando')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;

  const b = q.data;
  const ccy = b.currency;
  const labels = steps.map((s) => STEP_LABEL[s.key]());
  const fx = b.lines.fx_gain;
  const fxText = privacy ? '•••••' : formatMoney(fx, ccy, { signed: true });
  const fxNote = fx !== 0 ? t('carteira-ponte-cambio', { valor: fxText }) : undefined;
  const period = t('carteira-ponte-periodo', { de: formatDate(b.from), ate: formatDate(b.to) });
  const est = b.estimated;

  return (
    <Figure
      number={index}
      title={t('carteira-ponte-titulo')}
      legend={
        <span className="label-caps inline-flex items-center gap-2">
          {period}
          {est ? (
            <span className="italic text-warning" title={t('carteira-ponte-estimado-dica')}>
              ≈ {t('carteira-ponte-estimado')}
            </span>
          ) : null}
        </span>
      }
      source={t('carteira-ponte-fonte')}
    >
      <div className="flex flex-col gap-4">
        <p className="text-[12.5px] text-dim">{t('carteira-ponte-explica')}</p>
        <div className="grid items-start gap-5 @min-[900px]:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
          <BridgeChart
            steps={steps}
            labels={labels}
            currency={ccy}
            fxNote={fxNote}
            ariaLabel={t('carteira-ponte-aria', { de: formatDate(b.from), ate: formatDate(b.to) })}
          />
          <table aria-label={t('carteira-ponte-linhas')} className="w-full text-[13px]">
            <thead className="sr-only">
              <tr>
                <th scope="col">{t('carteira-ponte-linha')}</th>
                <th scope="col">{t('carteira-ponte-valor')}</th>
              </tr>
            </thead>
            <tbody>
              {steps.map((s, i) => (
                <tr
                  key={s.key}
                  className={s.total ? 'border-t border-line first:border-t-0' : undefined}
                >
                  <th scope="row" className="py-1 pr-4 text-left font-normal text-dim">
                    {labels[i]}
                    {s.key === 'preco' && fxNote ? (
                      <span className="block pl-3 text-[11.5px]">{fxNote}</span>
                    ) : null}
                  </th>
                  <td className="py-1 text-right align-top">
                    <Money
                      cents={s.amount}
                      currency={ccy}
                      signed={!s.total}
                      estimated={est && s.key === 'fim'}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div role="group" aria-label={t('carteira-ponte-resultado')}>
          <dl className="flex flex-wrap items-baseline gap-x-8 gap-y-2 border-t border-line pt-3">
            <div className="flex items-baseline gap-2">
              <dt className="text-[13px] text-dim" title={t('carteira-ponte-resultado-dica')}>
                {t('carteira-ponte-resultado')}
              </dt>
              <dd className="text-[15px] font-medium">
                <Money cents={b.lines.delta} currency={ccy} tone="auto" estimated={est} />
              </dd>
            </div>
            {b.inflation_loss != null ? (
              <>
                <div className="flex items-baseline gap-2">
                  <dt className="text-[13px] text-dim">
                    {t('carteira-ponte-inflacao', { indice: deflatorLabel(b.deflator) })}
                  </dt>
                  <dd className="text-[13px]">
                    <Money cents={-b.inflation_loss} currency={ccy} signed estimated={est} />
                  </dd>
                </div>
                {b.delta_real != null ? (
                  <div className="flex items-baseline gap-2">
                    <dt className="text-[13px] text-dim">{t('carteira-ponte-real')}</dt>
                    <dd className="text-[13px]">
                      <Money cents={b.delta_real} currency={ccy} tone="auto" estimated={est} />
                    </dd>
                  </div>
                ) : null}
              </>
            ) : null}
          </dl>
        </div>
        {b.assets.length > 0 ? (
          <details
            className="rounded-card border border-line px-3 py-2"
            open={open}
            onToggle={(e) => setOpen(e.currentTarget.open)}
          >
            <summary className="label-caps cursor-pointer text-fg">
              {t('carteira-ponte-por-ativo', { n: b.assets.length })}
            </summary>
            {/* Só monta a tabela aberta: a ponte já está no gráfico acima. */}
            {open ? (
              <div className="mt-3">
                <AssetsTable rows={b.assets} currency={ccy} />
              </div>
            ) : null}
          </details>
        ) : null}
      </div>
    </Figure>
  );
}

/**
 * A ponte com o período próprio (seletor padrão da Carteira, escopo `ponte`; padrão: no ano),
 * fora do período do resto da Rentabilidade.
 */
export function BridgeCard({
  index,
  deflator,
}: {
  index: string;
  deflator: PerformanceArgs['deflator'];
}) {
  const [period, setPeriod] = usePeriod('ponte', 'ytd');
  const range = periodRange(period);
  const args: PerformanceArgs = {
    ...(deflator ? { deflator } : {}),
    ...(range.fromMonth ? { from_month: range.fromMonth } : {}),
    ...(range.toMonth ? { to_month: range.toMonth } : {}),
  };
  return (
    <div className="flex flex-col gap-2">
      <div className="flex justify-end">
        <PeriodSelect scope="ponte" value={period} onChange={setPeriod} />
      </div>
      <BridgeBody index={index} args={args} />
    </div>
  );
}
