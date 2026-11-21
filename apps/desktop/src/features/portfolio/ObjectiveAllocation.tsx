/**
 * Classificação "Objetivo" (`goals.objectives`): a carteira dividida pelas metas, pelos pesos das
 * posições ligadas a cada uma (frações e pote comum), e o que não está ligado a nenhuma.
 */
import { Link } from 'react-router';

import { useObjectives } from '@/api/hooks/goals';
import { InstrumentCard, Money, Private } from '@/design/components';
import { t } from '@/i18n';
import { formatPercent } from '@/lib/format';

import { ErrorBlock, LoadingBlock } from './components/States';

function Bar({ share, muted }: { share: number; muted?: boolean }) {
  return (
    <div className="h-1.5 w-full overflow-hidden rounded-full bg-line" aria-hidden>
      <div
        className={muted ? 'h-full bg-dim/50' : 'h-full bg-accent'}
        style={{ width: `${Math.min(100, Math.max(0, share * 100))}%` }}
      />
    </div>
  );
}

export function ObjectiveAllocation({ index }: { index: string }) {
  const q = useObjectives();
  return (
    <InstrumentCard index={index} title={t('carteira-objetivo-titulo')}>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.slices.length === 0 ? (
        <p className="text-[12.5px] text-dim">
          {t('carteira-objetivo-vazio')}{' '}
          <Link to="/orcamento?aba=metas" className="text-accent hover:underline">
            {t('carteira-objetivo-ir-metas')}
          </Link>
        </p>
      ) : (
        <ul aria-label={t('carteira-objetivo-titulo')} className="flex flex-col gap-3">
          {q.data.slices.map((s) => (
            <li key={s.goal_id} className="flex flex-col gap-1">
              <div className="flex items-baseline justify-between gap-3 text-[12.5px]">
                <span className="min-w-0 truncate text-fg">{s.goal_name}</span>
                <span className="num shrink-0 text-fg">
                  <Money cents={s.value} currency={q.data.currency} />{' '}
                  <span className="text-dim">
                    <Private>{formatPercent(s.share, { decimals: 0 })}</Private>
                  </span>
                </span>
              </div>
              <Bar share={s.share} />
              <span className="text-[11px] text-dim">
                {t('carteira-objetivo-alvo', {
                  n: s.positions,
                  pct: formatPercent(s.target_amount > 0 ? s.value / s.target_amount : 0, {
                    decimals: 0,
                  }),
                })}
              </span>
            </li>
          ))}
          <li className="flex flex-col gap-1">
            <div className="flex items-baseline justify-between gap-3 text-[12.5px]">
              <span className="text-dim">{t('carteira-objetivo-sem')}</span>
              <span className="num text-fg">
                <Money cents={q.data.unassigned} currency={q.data.currency} />{' '}
                <span className="text-dim">
                  <Private>{formatPercent(q.data.unassigned_share, { decimals: 0 })}</Private>
                </span>
              </span>
            </div>
            <Bar share={q.data.unassigned_share} muted />
          </li>
          <li className="text-[11.5px] text-dim">{t('carteira-objetivo-nota')}</li>
        </ul>
      )}
    </InstrumentCard>
  );
}
