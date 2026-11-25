import type { AccountView } from '@/api/generated/AccountView';
import type { Member } from '@/api/generated/Member';
import { useConvert } from '@/api/hooks/fx';
import {
  InstrumentCard,
  KeyValue,
  Measure,
  Money,
  OriginBadge,
  Private,
} from '@/design/components';
import { cx } from '@/design/cx';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { ACCOUNT_KIND } from '@/features/transactions/ui/labels';
import { useBaseCurrency } from '@/state/currency';
import { FreshnessBadge } from './Freshness';
import { InstitutionSiteLink } from './InstitutionSiteLink';
import { OverdraftSummary } from './Overdraft';

function safeDate(iso: string | null) {
  if (!iso) return '—';
  try {
    return formatDate(iso);
  } catch {
    return iso;
  }
}

/** Saldo na moeda base da família de conta em outra moeda, com o selo de origem do câmbio. */
export function BaseBalance({ account }: { account: AccountView }) {
  const base = useBaseCurrency();
  const conv = useConvert(
    account.currency !== base
      ? { amount: account.balance, from: account.currency, to: base, date: todayIso() }
      : null,
  );
  if (account.currency === base) return null;
  if (account.balance_base == null) {
    return (
      <span className="text-[12px] text-warning">
        {t('contas-sem-cambio', { moeda: account.currency, base })}
      </span>
    );
  }
  const c = conv.data;
  return (
    <span className="inline-flex items-center gap-1.5">
      <Money cents={account.balance_base} currency={base} estimated={account.fx_estimated} />
      <OriginBadge
        source={
          c?.source === 'manual'
            ? t('contas-cambio-manual')
            : c?.source
              ? t('contas-cambio-fonte', { fonte: c.source })
              : t('contas-cambio')
        }
        date={c?.rate_date ?? undefined}
        status={account.fx_estimated ? 'estimado' : 'oficial'}
        note={
          c
            ? t(account.fx_estimated ? 'contas-cambio-nota-velha' : 'contas-cambio-nota', {
                moeda: account.currency,
                taxa: Number(c.rate),
                base,
              })
            : undefined
        }
      />
    </span>
  );
}

/** Card de instrumento de uma conta: saldo, previsto, conciliado, moeda base e crédito do cartão. */
export function AccountCard({
  account,
  index,
  members,
  selected,
  onOpen,
}: {
  account: AccountView;
  index: string;
  members: readonly Member[];
  selected?: boolean;
  onOpen: () => void;
}) {
  const base = useBaseCurrency();
  const card = account.card;
  const owners = account.member_ids
    .map((id) => members.find((m) => m.id === id)?.name)
    .filter(Boolean);
  const isCard = account.kind === 'credit_card';
  const used = card?.credit_limit
    ? Math.min(
        1,
        Math.max(0, (card.credit_limit - (account.available_credit ?? 0)) / card.credit_limit),
      )
    : 0;
  return (
    <InstrumentCard
      index={index}
      title={account.name}
      headingLevel={3}
      className={cx(
        'transition-colors',
        selected && 'border-accent',
        account.archived && 'opacity-70',
      )}
      actions={
        <span className="flex items-center gap-1">
          <InstitutionSiteLink institutionId={account.institution_id} />
          <button
            type="button"
            onClick={onOpen}
            aria-label={t('contas-abrir', { nome: account.name })}
            className="label-caps rounded-control px-1.5 py-0.5 hover:text-accent"
          >
            {t('contas-detalhes')}
          </button>
        </span>
      }
      footer={[
        { label: t('contas-tipo'), value: ACCOUNT_KIND[account.kind] },
        ...(account.institution_name
          ? [{ label: t('contas-instituicao'), value: account.institution_name }]
          : []),
        ...(owners.length
          ? [{ label: t('contas-membros-rotulo', { n: owners.length }), value: owners.join(', ') }]
          : []),
        { label: t('contas-ultimo'), value: safeDate(account.last_txn_date) },
      ]}
    >
      <button type="button" onClick={onOpen} className="block w-full text-left" tabIndex={-1}>
        <Measure
          size="md"
          label={isCard ? t('contas-fatura-saldo-devedor') : t('contas-saldo-hoje')}
          value={
            <Money
              cents={account.balance}
              currency={account.currency}
              tone={account.balance < 0 && !isCard ? 'negative' : 'neutral'}
            />
          }
          changeKey={account.balance}
          hint={
            account.closed_on
              ? t('contas-encerrada-em', { data: formatDate(account.closed_on) })
              : account.archived
                ? t('contas-arquivada')
                : undefined
          }
        />
      </button>
      <FreshnessBadge account={account} />
      <KeyValue
        layout="stacked"
        className="mt-4"
        items={[
          {
            label: t('contas-previsto-fim-mes'),
            value: <Money cents={account.projected_balance} currency={account.currency} />,
          },
          {
            label: t('contas-conciliado'),
            value: <Money cents={account.reconciled_balance} currency={account.currency} />,
          },
          ...(account.currency !== base
            ? [
                {
                  label: t('contas-em-moeda', { moeda: base }),
                  value: <BaseBalance account={account} />,
                },
              ]
            : []),
          ...(isCard && card
            ? [
                {
                  label: t('contas-disponivel'),
                  value:
                    account.available_credit != null ? (
                      <Money
                        cents={account.available_credit}
                        currency={account.currency}
                        tone={account.available_credit < 0 ? 'negative' : 'neutral'}
                      />
                    ) : (
                      t('contas-sem-limite')
                    ),
                },
                {
                  label: t('contas-ciclo'),
                  value: t('contas-ciclo-valor', {
                    fechamento: card.closing_day,
                    vencimento: card.due_day,
                  }),
                },
              ]
            : []),
        ]}
      />
      {isCard && card?.credit_limit ? (
        <div className="mt-3" aria-hidden>
          <div className="h-1.5 w-full overflow-hidden rounded-full bg-surface-2">
            <div
              className={cx(
                'h-full rounded-full',
                used > 0.9 ? 'bg-negative' : used > 0.7 ? 'bg-warning' : 'bg-accent',
              )}
              style={{ width: `${used * 100}%` }}
            />
          </div>
          <p className="label-caps mt-1">
            {withSlots(t('contas-limite-comprometido', { pct: slot(0) }), [
              <Private>{`${Math.round(used * 100)}%`}</Private>,
            ])}
          </p>
        </div>
      ) : null}
      {account.overdraft_status ? (
        <div className="mt-3">
          <Private>
            <OverdraftSummary status={account.overdraft_status} currency={account.currency} />
          </Private>
        </div>
      ) : null}
    </InstrumentCard>
  );
}
