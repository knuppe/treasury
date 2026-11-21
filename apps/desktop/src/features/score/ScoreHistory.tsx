/** Histórico do score do ativo (snapshots diários do núcleo), com a visão em tabela. */
import { useCallback, useMemo } from 'react';

import type { AssetScoreArgs } from '@/api/generated/AssetScoreArgs';
import type { AssetScorePoint } from '@/api/generated/AssetScorePoint';
import { useScoreHistory } from '@/api/hooks/score';
import { ChartFrame, EChart, type ChartTheme } from '@/charts';
import { TechTable, type TechColumn } from '@/design/components';
import { t } from '@/i18n';

import { safeDate } from '../portfolio/labels';
import { linesOption } from '../research/charts';
import { BAND, bandKey, formatConfidence, formatScore } from './format';

const COLUMNS: TechColumn<AssetScorePoint>[] = [
  {
    key: 'as_of',
    get header() {
      return t('score-data');
    },
    cell: (p) => safeDate(p.as_of),
  },
  {
    key: 'score',
    get header() {
      return t('score-nota');
    },
    numeric: true,
    cell: (p) => formatScore(p.score),
  },
  {
    key: 'band',
    get header() {
      return t('score-faixa');
    },
    cell: (p) => BAND[bandKey(p.band)].label,
  },
  {
    key: 'confidence',
    get header() {
      return t('score-confianca');
    },
    numeric: true,
    cell: (p) => formatConfidence(p.confidence),
  },
  {
    key: 'methodology_version',
    get header() {
      return t('score-metodologia');
    },
  },
];

export function ScoreHistory({ subject, index }: { subject: AssetScoreArgs; index: string }) {
  const q = useScoreHistory(subject);
  const points = useMemo(() => q.data ?? [], [q.data]);
  const scored = points.filter((p) => p.score != null);
  const option = useCallback(
    (theme: ChartTheme) =>
      linesOption(theme, {
        labels: points.map((p) => safeDate(p.as_of)),
        series: [{ key: 'score', name: t('score-nota'), values: points.map((p) => p.score) }],
        colorKeys: ['score'],
        format: (v) => formatScore(v),
      }),
    [points],
  );
  if (scored.length < 2) return null;
  const first = scored[0]!;
  const last = scored[scored.length - 1]!;
  return (
    <ChartFrame
      index={index}
      title={t('score-historico')}
      source={t('score-historico-fonte', { de: safeDate(first.as_of), ate: safeDate(last.as_of) })}
      chart={
        <EChart
          option={option}
          height={200}
          ariaLabel={t('score-historico-aria', {
            primeira: formatScore(first.score),
            de: safeDate(first.as_of),
            ultima: formatScore(last.score),
            ate: safeDate(last.as_of),
          })}
        />
      }
      table={
        <TechTable
          label={t('score-historico')}
          prefsId="Histórico do score"
          columns={COLUMNS}
          rows={points}
          getRowId={(p) => `${p.as_of}-${p.methodology_version}`}
          customizable={false}
          dense
        />
      }
    />
  );
}
