import { useMemo, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';

import type { AccountKind } from '@/api/generated/AccountKind';
import type { AccountView } from '@/api/generated/AccountView';
import { useAccount, useAccounts, useReopenAccount, useUpdateAccount } from '@/api/hooks/accounts';
import { useMembers } from '@/api/hooks/household';
import { screenMeta } from '@/app/screens';
import { Button, EmptyState, Icon, Measure, Money, ScreenHeader, toast } from '@/design/components';
import { LoyaltyPanel } from '@/features/cards/LoyaltyPanel';
import { BalanceForecastPanel } from '@/features/planning/BalanceForecastPanel';
import { accountMenuItems } from '@/features/accounts/accountMenu';
import { AccountDetail } from '@/features/accounts/AccountDetail';
import { AccountForm } from '@/features/accounts/AccountForm';
import { AccountList } from '@/features/accounts/AccountList';
import { CloseAccountDialog } from '@/features/accounts/CloseAccountDialog';
import { money, usePageContext } from '@/features/companion/pageContext';
import { useImportHub } from '@/features/import/importHub';
import { TxnDialog, type TxnDialogMode } from '@/features/transactions/TxnDialog';
import { QueryState, Switch } from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { useKeyShortcut } from '@/features/transactions/ui/hooks';
import { ACCOUNT_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';
import { useBaseCurrency } from '@/state/currency';

const meta = screenMeta('contas');

/** Soma na moeda base da família das contas que entram no patrimônio (estimada se algum câmbio for velho). */
function totals(accounts: AccountView[]) {
  let assets = 0;
  let cards = 0;
  let available = 0;
  let estimated = false;
  let missing = 0;
  for (const a of accounts) {
    if (a.archived || !a.include_in_net_worth) continue;
    if (a.balance_base == null) {
      missing++;
      continue;
    }
    if (a.fx_estimated) estimated = true;
    if (a.kind === 'credit_card') {
      cards += a.balance_base;
      available += a.available_credit ?? 0;
    } else assets += a.balance_base;
  }
  return { assets, cards, available, estimated, missing, net: assets + cards };
}

export default function Contas() {
  const [params, setParams] = useSearchParams();
  const [showArchived, setShowArchived] = useState(false);
  const { data: accounts = [], isLoading, error, refetch } = useAccounts(showArchived);
  const { data: members = [] } = useMembers(true);
  const [form, setForm] = useState<{ account: AccountView | null; kind: AccountKind } | null>(null);
  // `?nova=conta|cartao` (barra ⌘K): o cadastro abre pela URL e, ao fechar, o parâmetro sai.
  const nova = params.get('nova');
  const fromUrl =
    nova === 'conta' || nova === 'cartao'
      ? { account: null, kind: (nova === 'cartao' ? 'credit_card' : 'checking') as AccountKind }
      : null;
  const shownForm = form ?? fromUrl;
  const closeForm = () => {
    setForm(null);
    if (fromUrl) {
      const next = new URLSearchParams(params);
      next.delete('nova');
      setParams(next, { replace: true });
    }
  };
  const [txnFor, setTxnFor] = useState<{ accountId: string; mode: TxnDialogMode } | null>(null);
  const navigate = useNavigate();
  const updateAccount = useUpdateAccount();
  const reopenAccount = useReopenAccount();
  const [closing, setClosing] = useState<AccountView | null>(null);
  const reopen = (a: AccountView) =>
    reopenAccount.mutate(
      { id: a.id },
      {
        onSuccess: () =>
          toast({ tone: 'success', title: t('contas-conta-reaberta'), description: a.name }),
        onError: (e) =>
          toast({ tone: 'error', title: t('contas-erro-alterar'), description: errorText(e) }),
      },
    );
  const selectedId = params.get('conta');
  const listed = accounts.find((a) => a.id === selectedId) ?? null;
  // An archived or closed account is not in the list without the switch: its page still opens
  // (and stays open when the account is archived or closed from it).
  const one = useAccount(selectedId);
  const selected = listed ?? (one.data?.id === selectedId ? one.data : null) ?? null;
  const sum = useMemo(() => totals(accounts), [accounts]);
  const base = useBaseCurrency();

  const open = (id: string | null, tab?: string) => {
    const next = new URLSearchParams(params);
    if (id) next.set('conta', id);
    else next.delete('conta');
    if (tab) next.set('aba', tab);
    else next.delete('aba');
    setParams(next);
  };

  useKeyShortcut('n', () => setForm({ account: null, kind: 'checking' }));

  // Contexto para o agente: totais, contas visíveis com saldo e a conta aberta.
  usePageContext(
    isLoading
      ? null
      : {
          screen: 'contas',
          title: `${meta.index} · ${meta.title}`,
          view: selected ? (params.get('aba') ?? 'conta') : 'lista',
          filters: { show_archived: showArchived },
          selection: selected
            ? {
                account_id: selected.id,
                name: selected.name,
                kind: selected.kind,
                institution: selected.institution_name,
              }
            : undefined,
          figures: {
            net: money(sum.net, base),
            in_accounts: money(sum.assets, base),
            cards_due: money(sum.cards, base),
            available_credit: money(sum.available, base),
            accounts: accounts.length,
          },
          items: accounts.map((a) => ({
            id: a.id,
            name: a.name,
            kind: a.kind,
            institution: a.institution_name,
            balance: money(a.balance, a.currency),
            last_txn_date: a.last_txn_date,
            ...(a.card ? { closing_day: a.card.closing_day, due_day: a.card.due_day } : {}),
          })),
        },
  );

  // Nova conta, novo cartão e cada conta entram na barra ⌘K pelos comandos globais.

  // Menu de contexto do card: os mesmos atalhos do card, do detalhe e do cadastro.
  const menuOf = (a: AccountView) =>
    accountMenuItems(a, {
      open: (tab) => open(a.id, tab),
      newTxn: (mode) => setTxnFor({ accountId: a.id, mode }),
      viewTxns: () => void navigate(txnLink({ account_ids: [a.id] })),
      importStatement: () => useImportHub.getState().setOpen(true),
      edit: () => setForm({ account: a, kind: a.kind }),
      close: () => setClosing(a),
      reopen: () => reopen(a),
      toggleArchived: () =>
        updateAccount.mutate(
          { id: a.id, archived: !a.archived },
          {
            onSuccess: () =>
              toast({
                tone: 'success',
                title: a.archived ? t('contas-conta-reativada') : t('contas-conta-arquivada'),
                description: a.name,
              }),
            onError: (e) =>
              toast({
                tone: 'error',
                title: t('contas-erro-alterar'),
                description: errorText(e),
              }),
          },
        ),
    });

  const dialogs = (
    <>
      <AccountForm
        open={Boolean(shownForm)}
        onOpenChange={(o) => !o && closeForm()}
        account={shownForm?.account ?? null}
        initialKind={shownForm?.kind}
        onSaved={(a) => open(a.id)}
      />
      <TxnDialog
        open={Boolean(txnFor)}
        onOpenChange={(o) => !o && setTxnFor(null)}
        initialAccountId={txnFor?.accountId}
        initialMode={txnFor?.mode}
      />
      <CloseAccountDialog
        account={closing}
        open={Boolean(closing)}
        onOpenChange={(o) => !o && setClosing(null)}
        // The closing dialog gives way to the transfer or the adjustment; it opens again after.
        onTransfer={() => {
          if (closing) setTxnFor({ accountId: closing.id, mode: 'transfer' });
          setClosing(null);
        }}
        onAdjust={() => {
          if (closing) setTxnFor({ accountId: closing.id, mode: 'adjustment' });
          setClosing(null);
        }}
      />
    </>
  );

  // Página da conta: a lista sai e entra o detalhe, com o caminho de volta.
  if (selected) {
    return (
      <div className="flex flex-col gap-5">
        <button
          type="button"
          onClick={() => open(null)}
          className="label-caps inline-flex w-fit items-center gap-1.5 text-dim hover:text-accent"
        >
          <Icon name="arrowLeft" size={13} />
          {t('contas-voltar-lista')}
        </button>
        <ScreenHeader
          index={meta.index}
          title={selected.name}
          meta={[ACCOUNT_KIND[selected.kind], selected.institution_name]
            .filter(Boolean)
            .join(' · ')}
          description={
            selected.closed_on
              ? t('contas-encerrada-descricao', { data: formatDate(selected.closed_on) })
              : selected.archived
                ? t('contas-arquivada')
                : undefined
          }
          actions={
            selected.closed_on ? (
              <Button loading={reopenAccount.isPending} onClick={() => reopen(selected)}>
                {t('contas-reabrir')}
              </Button>
            ) : (
              <Button variant="ghost" onClick={() => setClosing(selected)}>
                {t('contas-encerrar')}
              </Button>
            )
          }
        />
        <AccountDetail
          key={selected.id}
          account={selected}
          tab={params.get('aba')}
          onTabChange={(tab) => open(selected.id, tab)}
          onEdit={() => setForm({ account: selected, kind: selected.kind })}
          onNewTxn={() => setTxnFor({ accountId: selected.id, mode: 'expense' })}
          onAdjust={() => setTxnFor({ accountId: selected.id, mode: 'adjustment' })}
        />
        {dialogs}
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={t('contas-descricao', { moeda: base })}
        actions={
          <>
            <Link
              to="/familia"
              className="label-caps inline-flex h-9 items-center rounded-control px-3 hover:text-accent"
            >
              {t('contas-membros-familia')}
            </Link>
            <Button
              variant="secondary"
              onClick={() => setForm({ account: null, kind: 'credit_card' })}
            >
              {t('contas-novo-cartao')}
            </Button>
            <Button
              variant="primary"
              icon={<Icon name="plus" size={13} />}
              onClick={() => setForm({ account: null, kind: 'checking' })}
            >
              {t('contas-nova-conta')}
            </Button>
          </>
        }
      />

      <QueryState
        loading={isLoading}
        error={error}
        onRetry={() => void refetch()}
        label={t('contas-carregando')}
      >
        {accounts.length === 0 ? (
          <EmptyState
            kicker={t('contas-vazio-kicker')}
            title={t('contas-vazio-titulo')}
            description={t('contas-vazio-descricao')}
            actions={
              <>
                <Button
                  variant="primary"
                  onClick={() => setForm({ account: null, kind: 'checking' })}
                >
                  {t('contas-nova-conta')}
                </Button>
                <Button onClick={() => setForm({ account: null, kind: 'credit_card' })}>
                  {t('contas-novo-cartao')}
                </Button>
              </>
            }
          />
        ) : (
          <>
            <section
              aria-label={t('contas-resumo')}
              className="grid gap-6 rounded-card border border-line bg-surface p-5 @min-[400px]:grid-cols-2 @min-[1040px]:grid-cols-[minmax(0,1.5fr)_repeat(3,minmax(0,1fr))]"
            >
              <Measure
                label={t('contas-saldo-liquido', { moeda: base })}
                value={<Money cents={sum.net} estimated={sum.estimated} />}
                changeKey={sum.net}
                origin={{
                  source: t('contas-saldo-liquido-fonte'),
                  status: sum.estimated ? 'estimado' : 'oficial',
                  note: sum.estimated ? t('contas-saldo-liquido-estimado') : undefined,
                }}
                hint={sum.missing ? t('contas-sem-cambio-fora', { n: sum.missing }) : undefined}
              />
              <Measure
                size="md"
                label={t('contas-em-contas')}
                value={
                  <span className="text-[24px]">
                    <Money cents={sum.assets} estimated={sum.estimated} />
                  </span>
                }
              />
              <Measure
                size="md"
                label={t('contas-devido-cartoes')}
                value={
                  <span className="text-[24px]">
                    <Money cents={sum.cards} tone={sum.cards < 0 ? 'negative' : 'neutral'} />
                  </span>
                }
              />
              <Measure
                size="md"
                label={t('contas-credito-disponivel')}
                value={
                  <span className="text-[24px]">
                    <Money cents={sum.available} />
                  </span>
                }
              />
            </section>

            <BalanceForecastPanel />

            <div className="flex justify-end">
              <div className="w-fit">
                <Switch
                  checked={showArchived}
                  onChange={setShowArchived}
                  label={t('contas-mostrar-arquivadas')}
                />
              </div>
            </div>

            <AccountList
              accounts={accounts}
              members={members}
              onOpen={(a) => open(a.id)}
              rowMenu={menuOf}
            />

            <LoyaltyPanel />
          </>
        )}
      </QueryState>

      {dialogs}
    </div>
  );
}
