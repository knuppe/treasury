/** O resultado da conta do plano de faculdade: alvo, aporte e a comparação "só com IPCA". */
import type { EducationResult } from '@/api/generated/EducationResult';
import { Money } from '@/design/components';
import { cx } from '@/design/cx';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatPercent } from '@/lib/format';

/** "+1,9% a.a. acima da inflação" (ou "só a inflação geral"). */
export function spreadLabel(spread: number): string {
  if (Math.abs(spread) < 0.00005) return t('educacao-selo-sem-diferenca');
  return t('educacao-selo-diferenca', {
    pct: formatPercent(spread, { decimals: 1, signed: true }),
  });
}

export function EducationResultPanel({
  result,
  error,
  loading,
  className,
}: {
  result: EducationResult | null;
  error?: string | null;
  loading?: boolean;
  className?: string;
}) {
  if (error && !result) {
    return (
      <p role="status" className={cx('text-[12.5px] text-dim', className)}>
        {error}
      </p>
    );
  }
  if (!result) return null;
  const r = result;
  // A diferença usada na conta (0 com "só a inflação geral"; a digitada com a fixa).
  const spread = r.inflation.spread;
  const nextYear = new Date().getFullYear() + 1;
  return (
    <section
      aria-label={t('educacao-resultado')}
      aria-busy={loading || undefined}
      className={cx('flex flex-col gap-3 rounded-control border border-line p-3', className)}
    >
      <dl className="grid gap-x-4 gap-y-2 text-[12.5px] sm:grid-cols-3">
        <div>
          <dt className="label-caps">
            {t('educacao-alvo-inicio', { data: formatDate(r.start_date) })}
          </dt>
          <dd className="num text-[18px] text-fg">
            <Money cents={r.target_at_start} />
          </dd>
        </div>
        <div>
          <dt className="label-caps">
            {r.phase === 'paying' ? t('educacao-aporte-curso') : t('educacao-aporte-hoje')}
          </dt>
          <dd className="num text-[18px] text-fg">
            <Money cents={r.monthly_needed} />
            <span className="text-[12px] text-dim">{t('metas-por-mes')}</span>
          </dd>
        </div>
        <div>
          <dt className="label-caps">{t('educacao-mensalidade-inicio')}</dt>
          <dd className="num text-fg">
            <Money cents={r.fee_at_start} />
            <span className="block text-[11.5px] text-dim">
              {withSlots(t('educacao-mensalidade-epoca', { valor: slot(0) }), [
                <Money key="n" cents={r.fee_at_start_nominal} />,
              ])}
            </span>
          </dd>
        </div>
      </dl>
      {r.phase === 'saving' ? (
        <ul className="flex flex-col gap-1 text-[12.5px] text-dim">
          <li>
            {withSlots(
              t('educacao-aporte-ano-que-vem', { ano: String(nextYear), valor: slot(0) }),
              [<Money key="p" cents={r.monthly_next_year} />],
            )}
          </li>
          <li>
            {withSlots(t('educacao-aporte-fixo-valor', { valor: slot(0) }), [
              <Money key="f" cents={r.monthly_flat} />,
            ])}
          </li>
          {spread !== 0 ? (
            <li>
              {withSlots(t('educacao-so-ipca', { valor: slot(0) }), [
                <Money key="h" cents={r.monthly_headline_only} />,
              ])}
            </li>
          ) : null}
        </ul>
      ) : null}
      {r.phase === 'paying' ? (
        <p className="text-[12.5px] text-fg">
          {r.uncovered_year == null
            ? t('educacao-cobre-tudo')
            : t('educacao-cobre-ate', { ano: r.uncovered_year - 1 })}
        </p>
      ) : null}
      <p className="text-[12px] text-dim">
        {r.inflation.mode === 'series_10y' && !r.inflation.missing
          ? t('educacao-fonte-inflacao', {
              selo: spreadLabel(r.inflation.spread),
              serie: r.inflation.series_name ?? '',
              de: r.inflation.from ?? '',
              ate: r.inflation.to ?? '',
            })
          : spreadLabel(spread)}
      </p>
      {r.notes.length ? (
        <ul className="flex list-disc flex-col gap-1 pl-4 text-[12px] text-warning">
          {r.notes.map((n) => (
            <li key={n}>{n}</li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
