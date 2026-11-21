/**
 * Carteira › Saúde dos ativos: o score de saúde de cada ativo com posição ou na watchlist
 * (`score.portfolio`), na ordem do núcleo (com score primeiro, do maior para o menor).
 * Cada linha leva à aba Saúde da página do ativo.
 */
import { Link, useNavigate } from 'react-router';

import type { AssetScoreSummary } from '@/api/generated/AssetScoreSummary';
import { useScorePortfolio } from '@/api/hooks/score';
import { EmptyState, TechTable, Tooltip, type TechColumn } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { ErrorBlock, LoadingBlock } from '../portfolio/components/States';
import { subjectHref } from '../research/common';
import { BAND, bandOfScore, formatConfidence, formatDelta, formatScore } from './format';
import { BandChip } from './parts';

/** Página do ativo já na aba Saúde. */
export function scoreHref(r: { asset_id: string | null; ticker: string | null }): string {
  return r.asset_id
    ? `/ativos?ativo=${encodeURIComponent(r.asset_id)}&secao=saude`
    : subjectHref(null, r.ticker);
}

function ScoreCell({ r }: { r: AssetScoreSummary }) {
  if (r.status === 'not_supported') {
    return (
      <span className="text-[12px] text-dim">
        {r.error ??
          (r.band_label !== 'Dados insuficientes' ? r.band_label : t('score-classe-nao-suportada'))}
      </span>
    );
  }
  if (r.status === 'insufficient_data') {
    const chip = (
      <span tabIndex={0} className="inline-flex cursor-help">
        <BandChip band="dados_insuficientes" />
      </span>
    );
    if (r.missing_critical.length === 0) return chip;
    return (
      <Tooltip
        side="left"
        content={
          <div className="flex flex-col gap-1">
            <span className="label-caps text-fg">{t('score-falta-critico')}</span>
            <span>{r.missing_critical.join(', ')}</span>
          </div>
        }
      >
        {chip}
      </Tooltip>
    );
  }
  const partial = r.status === 'partial';
  return (
    <span className="inline-flex items-center justify-end gap-2">
      <span
        className={cx(
          'num text-[13px]',
          partial ? 'text-fg italic' : BAND[bandOfScore(r.score)].text,
        )}
      >
        {formatScore(r.score)}
      </span>
      {partial ? (
        <span title={t('score-parcial-dica')}>
          <BandChip band="dados_insuficientes" label={t('score-status-parcial')} />
        </span>
      ) : (
        <BandChip band={r.band} label={r.band_label} />
      )}
    </span>
  );
}

const sub = (v: number | null) => (
  <span className={BAND[bandOfScore(v)].text}>{formatScore(v)}</span>
);

const COLUMNS: TechColumn<AssetScoreSummary>[] = [
  {
    key: 'ticker',
    get header() {
      return t('score-ativo');
    },
    hideable: false,
    sortValue: (r) => r.ticker ?? r.name,
    cell: (r) => (
      <span className="flex min-w-0 items-baseline gap-2">
        <Link
          to={scoreHref(r)}
          className="font-medium text-fg underline-offset-2 hover:text-accent hover:underline"
        >
          {r.ticker ?? r.name}
        </Link>
        {r.ticker ? <span className="max-w-28 truncate text-[12px] text-dim">{r.name}</span> : null}
      </span>
    ),
  },
  {
    key: 'segment_label',
    get header() {
      return t('score-segmento');
    },
    cell: (r) => <span className="block max-w-36 truncate text-dim">{r.segment_label || '—'}</span>,
  },
  {
    key: 'score',
    get header() {
      return t('score-nota');
    },
    numeric: true,
    sortValue: (r) => r.score,
    cell: (r) => <ScoreCell r={r} />,
  },
  {
    key: 'confidence',
    get header() {
      return t('score-confianca');
    },
    numeric: true,
    cell: (r) => (r.status === 'not_supported' ? '—' : formatConfidence(r.confidence)),
  },
  {
    key: 'health',
    get header() {
      return t('score-saude');
    },
    numeric: true,
    cell: (r) => sub(r.health),
  },
  {
    key: 'attractiveness',
    get header() {
      return t('score-atratividade');
    },
    numeric: true,
    cell: (r) => sub(r.attractiveness),
  },
  {
    key: 'delta',
    get header() {
      return t('score-variacao');
    },
    numeric: true,
    cell: (r) =>
      r.delta == null ? (
        <span className="text-dim">—</span>
      ) : (
        <span
          title={r.alerts.join(' · ') || undefined}
          className={r.delta > 0 ? 'text-positive' : r.delta < 0 ? 'text-negative' : 'text-fg'}
        >
          {r.delta > 0 ? '▲ ' : r.delta < 0 ? '▼ ' : ''}
          {formatDelta(r.delta)}
          {r.alerts.length > 0 ? ' !' : ''}
        </span>
      ),
  },
  {
    key: 'flags',
    get header() {
      return t('score-flags');
    },
    get title() {
      return t('score-red-flags');
    },
    numeric: true,
    sortValue: (r) => r.flags.length,
    cell: (r) =>
      r.flags.length === 0 ? (
        <span className="text-dim">0</span>
      ) : (
        <span className="text-warning" title={r.flags.join(' · ')}>
          {r.flags.length}
        </span>
      ),
  },
];

export function ScorePortfolioPanel() {
  const q = useScorePortfolio();
  const navigate = useNavigate();
  if (q.isPending) return <LoadingBlock label={t('score-calculando-ativos')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const rows = q.data;
  if (rows.length === 0) {
    return (
      <EmptyState
        kicker={t('score-sem-ativos')}
        title={t('score-carteira-vazia')}
        description={t('score-carteira-vazia-descricao')}
      />
    );
  }
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[12.5px] leading-snug text-dim">{t('score-carteira-ajuda')}</p>
      <TechTable
        label={t('score-carteira-tabela')}
        prefsId="Saúde dos ativos"
        columns={COLUMNS}
        rows={rows}
        getRowId={(r) => r.asset_id ?? r.ticker ?? r.name}
        onActivate={(r) => void navigate(scoreHref(r))}
        minWidth="880px"
      />
    </div>
  );
}
