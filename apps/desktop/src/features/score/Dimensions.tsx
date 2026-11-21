/**
 * Dimensões do score (uma linha por dimensão: barra da nota, peso, cobertura e selos) que
 * abrem a tabela das métricas: valor pela unidade, nota, percentil entre os pares, nota
 * pela âncora absoluta, período e fonte; sem dado, o motivo.
 */
import { useId, useState } from 'react';

import type { AssetScoreDimension } from '@/api/generated/AssetScoreDimension';
import type { AssetScoreMetric } from '@/api/generated/AssetScoreMetric';
import { Icon, TechTable, Tooltip, type TechColumn } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatNumber, formatPercent } from '@/lib/format';

import { BAND, bandOfScore, formatMetricValue, formatScore, percentileText } from './format';
import { ScoreBar, Tag } from './parts';

function MetricLabel({ m }: { m: AssetScoreMetric }) {
  return (
    <div className="flex min-w-0 flex-col gap-0.5 py-0.5">
      <span className="flex flex-wrap items-center gap-1.5">
        <Tooltip
          side="right"
          content={
            <div className="flex max-w-72 flex-col gap-1 py-0.5">
              <span className="label-caps text-fg">{m.label}</span>
              <span>{m.formula}</span>
              {m.source ? (
                <span className="text-dim">{t('score-fonte-de', { fonte: m.source })}</span>
              ) : null}
            </div>
          }
        >
          <span
            tabIndex={0}
            className="cursor-help text-fg underline decoration-line-strong decoration-dotted underline-offset-4"
          >
            {m.label}
          </span>
        </Tooltip>
        {m.critical ? (
          <Tag tone="warning" title={t('score-critica-dica')}>
            {t('score-critica')}
          </Tag>
        ) : null}
      </span>
      {m.missing ? (
        <span className="text-[11.5px] leading-snug whitespace-normal text-dim">
          {t('score-sem-dado', { motivo: m.missing })}
        </span>
      ) : null}
      {m.note ? (
        <span
          className={cx(
            'text-[11.5px] leading-snug whitespace-normal',
            m.worst ? 'text-negative' : 'text-dim',
          )}
        >
          {m.note}
        </span>
      ) : null}
    </div>
  );
}

const METRIC_COLUMNS: TechColumn<AssetScoreMetric>[] = [
  {
    key: 'label',
    get header() {
      return t('score-metrica');
    },
    hideable: false,
    width: '28%',
    cell: (m) => <MetricLabel m={m} />,
  },
  {
    key: 'value',
    get header() {
      return t('score-valor');
    },
    numeric: true,
    sortable: false,
    width: '10%',
    cell: (m) => (
      <span className={m.value == null ? 'text-dim' : 'text-fg'}>
        {formatMetricValue(m.value, m.unit)}
      </span>
    ),
  },
  {
    key: 'score',
    get header() {
      return t('score-col-nota');
    },
    numeric: true,
    width: '8%',
    cell: (m) => (
      <span className={BAND[bandOfScore(m.score)].text}>
        {formatScore(m.score)}
        {m.worst ? t('score-pior') : ''}
      </span>
    ),
  },
  {
    key: 'percentile',
    get header() {
      return t('score-entre-pares');
    },
    width: '24%',
    sortValue: (m) => m.percentile,
    cell: (m) => <span className="text-dim">{percentileText(m) ?? '—'}</span>,
  },
  {
    key: 'absolute',
    get header() {
      return t('score-absoluta');
    },
    numeric: true,
    width: '9%',
    cell: (m) => <span className="text-dim">{formatScore(m.absolute)}</span>,
  },
  {
    key: 'weight',
    get header() {
      return t('score-peso');
    },
    numeric: true,
    defaultHidden: true,
    cell: (m) => formatNumber(m.weight, 1),
  },
  {
    key: 'period',
    get header() {
      return t('score-periodo');
    },
    width: '21%',
    cell: (m) => <span className="text-dim">{m.period ?? '—'}</span>,
  },
  {
    key: 'source',
    get header() {
      return t('score-fonte');
    },
    defaultHidden: true,
    cell: (m) => <span className="text-dim">{m.source ?? '—'}</span>,
  },
];

function DimensionRow({ d }: { d: AssetScoreDimension }) {
  const [open, setOpen] = useState(false);
  const id = useId();
  return (
    <li className="border-b border-line/60 last:border-0" data-dimension={d.key}>
      <button
        type="button"
        aria-expanded={open}
        aria-controls={id}
        onClick={() => setOpen((o) => !o)}
        className="grid w-full grid-cols-[1fr_auto] items-center gap-x-4 gap-y-1.5 py-2.5 text-left hover:bg-surface-2/40 focus-visible:outline focus-visible:outline-accent @min-[790px]:grid-cols-[minmax(180px,2fr)_3fr_2.5rem_13rem]"
      >
        <span className="flex min-w-0 flex-wrap items-center gap-1.5">
          <Icon
            name={open ? 'chevronDown' : 'chevronRight'}
            size={12}
            className="shrink-0 text-dim"
          />
          <span className="text-[13px] text-fg">{d.label}</span>
          {d.imputed ? (
            <Tag tone="warning" title={t('score-imputada-dica')}>
              {t('score-imputada')}
            </Tag>
          ) : null}
          {d.attractiveness ? (
            <Tag tone="accent" title={t('score-selo-atratividade-dica')}>
              {t('score-selo-atratividade')}
            </Tag>
          ) : null}
        </span>
        <span
          className={cx(
            'num text-right text-[13px] @min-[790px]:order-3',
            BAND[bandOfScore(d.score)].text,
          )}
        >
          {formatScore(d.score)}
        </span>
        <span className="col-span-2 @min-[790px]:order-2 @min-[790px]:col-span-1">
          <ScoreBar score={d.score} label={d.label} muted={d.imputed} />
        </span>
        <span className="num col-span-2 text-[11.5px] text-dim @min-[790px]:order-4 @min-[790px]:col-span-1 @min-[790px]:text-right">
          {t('score-peso-cobertura', {
            peso: formatNumber(d.weight, 0),
            cobertura: formatPercent(d.coverage, { decimals: 0 }),
          })}
        </span>
      </button>
      {open ? (
        <div id={id} className="pb-3">
          <TechTable
            label={t('score-metricas-de', { dimensao: d.label })}
            prefsId="score-metricas"
            columns={METRIC_COLUMNS}
            rows={d.metrics}
            getRowId={(m) => m.key}
            minWidth="760px"
            dense
          />
        </div>
      ) : null}
    </li>
  );
}

export function DimensionsList({ dimensions }: { dimensions: readonly AssetScoreDimension[] }) {
  return (
    <ul aria-label={t('score-dimensoes')} className="flex flex-col">
      {dimensions.map((d) => (
        <DimensionRow key={d.key} d={d} />
      ))}
    </ul>
  );
}
