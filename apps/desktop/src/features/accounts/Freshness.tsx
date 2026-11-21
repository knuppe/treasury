/**
 * Quanto o saldo da conta está em dia (docs/account-freshness.md): "atualizado há N dias" em
 * cinza e "possivelmente incompleto" em âmbar, com o último lançamento na dica. Conta em dia
 * não mostra nada.
 */
import type { AccountView } from '@/api/generated/AccountView';
import { Tooltip } from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatMonthLong } from '@/lib/format';

/** A dica da conta possivelmente incompleta: o último lançamento, o mês em branco e o negativo. */
export function freshnessHint(a: AccountView): string {
  const parts = [
    a.last_txn_date
      ? t('contas-atualizacao-ultimo', { data: formatDate(a.last_txn_date) })
      : t('contas-atualizacao-sem-lancamentos'),
  ];
  if (a.missed_month) {
    parts.push(t('contas-atualizacao-mes-vazio', { mes: formatMonthLong(a.missed_month) }));
  }
  if (a.balance < 0 && a.kind !== 'credit_card') parts.push(t('contas-atualizacao-negativo'));
  return parts.join(' ');
}

export function FreshnessBadge({ account }: { account: AccountView }) {
  const days = account.staleness_days ?? 0;
  if (account.balance_status === 'aging') {
    return (
      <span className="text-[12px] text-dim">{t('contas-atualizacao-ha', { dias: days })}</span>
    );
  }
  if (account.balance_status !== 'possibly_incomplete') return null;
  return (
    <Tooltip content={freshnessHint(account)}>
      <span tabIndex={0} className="text-[12px] text-warning">
        {t('contas-atualizacao-incompleto')}
      </span>
    </Tooltip>
  );
}
