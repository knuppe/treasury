/**
 * Conta de aposentadoria (docs/international.md): o que entrou no ano contra o limite do plano
 * (401(k), IRA, HSA, PPR, plan de pensiones…), com as contas que dividem o limite e a fonte.
 */
import type { AccountView } from '@/api/generated/AccountView';
import { useRetirementStatus } from '@/api/hooks/retirement';
import { Money } from '@/design/components';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';

export function RetirementSummary({ account }: { account: AccountView }) {
  const q = useRetirementStatus(account.id);
  const s = q.data;
  if (!s) return null;
  const cur = s.plan.currency;
  const share = s.limit ? Math.min(1, s.contributed / s.limit) : 0;
  const over = s.limit != null && s.contributed > s.limit;
  return (
    <section
      aria-label={t('contas-aposentadoria-titulo', { plano: s.plan.name, ano: s.year })}
      className="flex flex-col gap-2 rounded-card border border-line p-3 text-[13px]"
    >
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <span className="label-caps text-dim">
          {t('contas-aposentadoria-titulo', { plano: s.plan.name, ano: s.year })}
        </span>
        {s.limit != null ? (
          <span>
            {withSlots(t('contas-aposentadoria-de', { aportado: slot(0), limite: slot(1) }), [
              <Money key="a" cents={s.contributed} currency={cur} />,
              <Money key="l" cents={s.limit} currency={cur} />,
            ])}
          </span>
        ) : (
          <span>
            {withSlots(t('contas-aposentadoria-sem-limite', { aportado: slot(0) }), [
              <Money key="a" cents={s.contributed} currency={cur} />,
            ])}
          </span>
        )}
      </div>
      {s.limit != null ? (
        <span className="h-2 rounded-full bg-surface-2" aria-hidden>
          <span
            className={`block h-2 rounded-full ${over ? 'bg-warning' : 'bg-accent'}`}
            style={{ width: `${Math.round(share * 100)}%` }}
          />
        </span>
      ) : null}
      <p className="text-[12px] text-dim">
        {over
          ? t('contas-aposentadoria-acima')
          : s.remaining != null
            ? withSlots(t('contas-aposentadoria-falta', { falta: slot(0) }), [
                <Money key="f" cents={s.remaining} currency={cur} />,
              ])
            : null}{' '}
        {s.plan.kind === 'tax_benefit' ? t('contas-aposentadoria-beneficio') : null}{' '}
        {s.age != null ? t('contas-aposentadoria-idade', { idade: s.age }) : null}{' '}
        {s.shared_with.length
          ? t('contas-aposentadoria-divide', { contas: s.shared_with.join(', ') })
          : null}{' '}
        {s.limit_year != null
          ? t('contas-aposentadoria-ano-anterior', { ano: s.limit_year })
          : null}
      </p>
      {s.source ? (
        <a
          href={s.source}
          target="_blank"
          rel="noreferrer"
          className="self-start text-[11.5px] text-dim underline hover:text-accent"
        >
          {t('contas-aposentadoria-fonte')}
        </a>
      ) : null}
    </section>
  );
}
