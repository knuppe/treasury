/**
 * "Faculdade dos filhos" (docs/design/college-goal.md §8): os planos somados, a linha do tempo
 * dos cursos e os anos em que dois pagam ao mesmo tempo.
 */
import { useEducationOverview } from '@/api/hooks/goals';
import { InstrumentCard, Money } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

export function EducationOverviewCard() {
  const overview = useEducationOverview();
  const o = overview.data;
  if (!o || o.plans.length === 0) return null;
  const years = o.plans.flatMap((p) => [
    Number(p.brief.start_date.slice(0, 4)),
    Number(p.brief.end_date.slice(0, 4)),
  ]);
  const now = new Date().getFullYear();
  const first = Math.min(now, ...years);
  const last = Math.max(...years);
  const span = Math.max(1, last - first + 1);
  const pos = (y: number) => `${((y - first) / span) * 100}%`;
  return (
    <InstrumentCard
      title={t('educacao-visao-titulo')}
      footer={[
        {
          label: t('educacao-visao-aporte'),
          value: <Money cents={o.monthly_total} currency={o.currency} />,
        },
        {
          label: t('educacao-visao-guardado'),
          value: <Money cents={o.saved_total} currency={o.currency} />,
        },
        {
          label: t('educacao-visao-alvo'),
          value: <Money cents={o.target_total} currency={o.currency} />,
        },
      ]}
    >
      <div className="flex flex-col gap-2">
        <ul aria-label={t('educacao-visao-linha')} className="flex flex-col gap-1.5">
          {o.plans.map((p) => {
            const s = Number(p.brief.start_date.slice(0, 4));
            const e = Number(p.brief.end_date.slice(0, 4));
            return (
              <li key={p.goal_id} className="grid grid-cols-[110px_1fr] items-center gap-2">
                <span className="truncate text-[12.5px] text-fg">
                  {p.brief.child_name ?? p.name}
                </span>
                <span className="relative h-3 rounded-[2px] bg-line/50">
                  <span
                    className="absolute inset-y-0 rounded-[2px] bg-accent"
                    style={{ left: pos(s), width: `${((e - s + 1) / span) * 100}%` }}
                    title={t('educacao-visao-curso', { inicio: String(s), fim: String(e) })}
                  />
                </span>
              </li>
            );
          })}
        </ul>
        <div className="grid grid-cols-[110px_1fr] gap-2 text-[11px] text-dim">
          <span />
          <span className="flex justify-between num">
            <span>{first}</span>
            <span>{last}</span>
          </span>
        </div>
        <p className={cx('text-[12px]', o.overlap_years.length ? 'text-warning' : 'text-dim')}>
          {o.overlap_years.length
            ? t('educacao-visao-sobreposicao', { anos: o.overlap_years.join(', ') })
            : t('educacao-visao-sem-sobreposicao')}
        </p>
      </div>
    </InstrumentCard>
  );
}
