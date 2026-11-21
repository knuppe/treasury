import type { AccountView } from '@/api/generated/AccountView';
import { useCardValue } from '@/api/hooks/cards';
import { Money } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { QueryState, SectionTitle } from '@/features/transactions/ui/controls';

/** "Este cartão vale a pena?": custo × retorno em 12 meses e a simulação de concentrar compras. */
export function CardValuePanel({ card }: { card: AccountView }) {
  const { data, isLoading, error } = useCardValue();
  const mine = data?.cards.find((c) => c.account_id === card.id);
  return (
    <QueryState loading={isLoading} error={error}>
      {data && mine ? (
        <section className="flex flex-col gap-3" aria-label={t('contas-vale-titulo')}>
          <SectionTitle>{t('contas-vale-titulo')}</SectionTitle>
          <p className="text-[12.5px] text-dim">
            {t('contas-vale-periodo', { de: formatDate(data.from), ate: formatDate(data.to) })}
          </p>
          <dl className="grid grid-cols-2 gap-3 text-[13px] @min-[500px]:grid-cols-3">
            {(
              [
                ['contas-vale-anuidade', -mine.annual_fee],
                ['contas-vale-encargos', -mine.charges],
                ['contas-vale-iof', -mine.iof],
                ['contas-vale-cashback', mine.cashback],
                ['contas-vale-pontos', mine.points_value],
              ] as const
            ).map(([label, cents]) => (
              <div key={label} className="flex flex-col gap-0.5">
                <dt className="label-caps">{t(label)}</dt>
                <dd>
                  {cents == null ? (
                    mine.points > 0 ? (
                      t('contas-recompensa-n-pontos', { n: mine.points })
                    ) : (
                      '—'
                    )
                  ) : (
                    <Money cents={cents} currency={mine.currency} tone="auto" />
                  )}
                </dd>
              </div>
            ))}
            <div className="flex flex-col gap-0.5">
              <dt className="label-caps">{t('contas-vale-resultado')}</dt>
              <dd className={cx('text-[16px]', mine.net >= 0 ? 'text-positive' : 'text-negative')}>
                <Money cents={mine.net} currency={mine.currency} />
              </dd>
            </div>
          </dl>
          {data.scenarios.length > 0 ? (
            <div className="flex flex-col gap-1">
              <span className="label-caps">{t('contas-vale-simulacao')}</span>
              <p className="text-[12.5px] text-dim">
                {t('contas-vale-simulacao-descricao')}{' '}
                <Money cents={data.total_spend} currency={data.currency} />
              </p>
              <ul className="flex flex-col divide-y divide-line/60 rounded-card border border-line">
                {data.scenarios.map((s) => (
                  <li
                    key={s.account_id}
                    className={cx(
                      'grid grid-cols-[1fr_auto_auto_auto] items-baseline gap-x-4 px-3 py-1.5 text-[12.5px]',
                      s.account_id === card.id && 'font-medium',
                    )}
                  >
                    <span>{s.account_name}</span>
                    <span className="text-dim">
                      {t('contas-vale-retorno')}{' '}
                      {s.reward != null ? (
                        <Money cents={s.reward} currency={data.currency} />
                      ) : (
                        t('contas-recompensa-n-pontos', { n: s.points ?? 0 })
                      )}
                    </span>
                    <span className="text-dim">
                      {t('contas-vale-anuidade')}{' '}
                      <Money cents={s.annual_fee} currency={data.currency} />
                    </span>
                    <span>
                      {s.net != null ? (
                        <Money cents={s.net} currency={data.currency} tone="auto" />
                      ) : (
                        '—'
                      )}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
          <p className="text-[12px] text-dim">{t('contas-vale-aviso')}</p>
        </section>
      ) : null}
    </QueryState>
  );
}
