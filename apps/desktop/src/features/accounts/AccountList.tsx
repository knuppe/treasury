/**
 * Lista das contas da tela 03 · Contas: uma linha por conta, agrupadas por tipo, com
 * instituição, membros, saldo (e o saldo na moeda base para contas em outra moeda), previsto e
 * último lançamento. Clicar no nome (ou Enter na linha) abre a página da conta; o menu de
 * contexto traz os mesmos atalhos de antes.
 */
import type { AccountView } from '@/api/generated/AccountView';
import type { Member } from '@/api/generated/Member';
import { Money, TechTable, type ContextMenuEntry, type TechColumn } from '@/design/components';
import { cx } from '@/design/cx';
import { ACCOUNT_GROUPS, ACCOUNT_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';

import { BaseBalance } from './AccountCard';
import { FreshnessBadge } from './Freshness';
import { InstitutionSiteLink } from './InstitutionSiteLink';

/** Ordem dos tipos na lista (a mesma dos grupos de antes). */
const ORDER = new Map(ACCOUNT_GROUPS.map((g, i) => [g.kind, i]));

export function AccountList({
  accounts,
  members,
  onOpen,
  rowMenu,
}: {
  accounts: readonly AccountView[];
  members: readonly Member[];
  onOpen: (account: AccountView) => void;
  rowMenu: (account: AccountView) => readonly ContextMenuEntry[];
}) {
  const base = useBaseCurrency();
  const name = (id: string) => members.find((m) => m.id === id)?.name;
  const rows = [...accounts].sort(
    (a, b) =>
      (ORDER.get(a.kind) ?? 99) - (ORDER.get(b.kind) ?? 99) ||
      a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }),
  );
  const columns: TechColumn<AccountView>[] = [
    {
      key: 'name',
      header: t('contas-col-conta'),
      cell: (a) => (
        <button
          type="button"
          onClick={() => onOpen(a)}
          aria-label={t('contas-abrir', { nome: a.name })}
          className={cx(
            'text-left font-sans text-[13.5px] font-medium whitespace-nowrap text-fg hover:text-accent hover:underline',
            a.archived && 'text-dim',
          )}
        >
          {a.name}
          {a.archived ? (
            <span className="ml-2 text-[11.5px] font-normal text-dim">{t('contas-arquivada')}</span>
          ) : null}
        </button>
      ),
    },
    {
      key: 'institution',
      header: t('contas-instituicao'),
      cell: (a) => (
        <span className="inline-flex items-center gap-1.5 font-sans text-dim">
          {a.institution_name ?? '—'}
          <InstitutionSiteLink institutionId={a.institution_id} />
        </span>
      ),
    },
    {
      key: 'kind',
      header: t('contas-tipo'),
      cell: (a) => <span className="font-sans whitespace-nowrap">{ACCOUNT_KIND[a.kind]}</span>,
    },
    {
      key: 'members',
      header: t('contas-col-membros'),
      cell: (a) => (
        <span className="font-sans text-dim">
          {a.member_ids.map(name).filter(Boolean).join(', ') || t('contas-sem-membro')}
        </span>
      ),
    },
    {
      key: 'balance',
      header: t('contas-col-saldo'),
      numeric: true,
      cell: (a) => (
        <span className="flex flex-col items-end">
          <Money
            cents={a.balance}
            currency={a.currency}
            tone={a.balance < 0 && a.kind !== 'credit_card' ? 'negative' : 'neutral'}
          />
          {a.currency !== base ? (
            <span className="text-[11.5px] text-dim">
              <BaseBalance account={a} />
            </span>
          ) : null}
        </span>
      ),
    },
    {
      key: 'projected',
      header: t('contas-previsto-fim-mes'),
      numeric: true,
      cell: (a) => <Money cents={a.projected_balance} currency={a.currency} />,
    },
    {
      key: 'last',
      header: t('contas-ultimo'),
      numeric: true,
      cell: (a) => (
        <span className="inline-flex flex-col items-end">
          <span>{a.last_txn_date ? formatDate(a.last_txn_date) : '—'}</span>
          <FreshnessBadge account={a} />
        </span>
      ),
    },
  ];
  return (
    <TechTable
      label={t('contas-lista')}
      prefsId="Contas"
      columns={columns}
      rows={rows}
      getRowId={(a) => a.id}
      onActivate={onOpen}
      rowMenu={(a) => rowMenu(a)}
      rowMenuLabel={(a) => t('contas-acoes-de', { nome: a.name })}
    />
  );
}
