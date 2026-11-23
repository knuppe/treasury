import { useCallback, useMemo, useRef, useState } from 'react';
import { Navigate, useNavigate, useSearchParams } from 'react-router';

import type { RuleInput } from '@/api/generated/RuleInput';
import type { TxnFilter } from '@/api/generated/TxnFilter';
import type { TxnStatus } from '@/api/generated/TxnStatus';
import type { TxnView } from '@/api/generated/TxnView';
import { useUndo } from '@/api/hooks/changes';
import { suggestRule } from '@/api/hooks/rules';
import { useAccounts } from '@/api/hooks/accounts';
import { useLoans } from '@/api/hooks/debt';
import {
  sameDescription,
  useDeleteTxns,
  useLinkTransfers,
  useUnlinkTransfer,
  useSetStatus,
  useTxnQuery,
} from '@/api/hooks/transactions';
import { screenMeta } from '@/app/screens';
import { useRegisterCommands, type AppCommand } from '@/commands/registry';
import {
  Button,
  EmptyState,
  Icon,
  InstrumentCard,
  Kbd,
  Money,
  ScreenHeader,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  toast,
} from '@/design/components';
import { pointerOrFocus } from '@/design/pointer';
import { LoanPaymentDialog } from '@/features/debts/LoanPaymentDialog';
import { BulkOperationLinkDialog } from '@/features/transactions/BulkOperationLinkDialog';
import { OperationLinkDialog } from '@/features/transactions/OperationLinkDialog';
import { ItemizeTxn, ItemizeTxnById } from '@/features/food/ItemizeTxn';
import { MailPanel } from '@/features/mail/MailPanel';
import { InboxPanel } from '@/features/import/InboxPanel';
import { ImportsPanel } from '@/features/import/ImportsPanel';
import { ImportWizard } from '@/features/import/ImportWizard';
import { useImportHub } from '@/features/import/importHub';
import { money, usePageContext } from '@/features/companion/pageContext';
import { LearnedRules } from '@/features/rules/LearnedRules';
import { RulesPanel } from '@/features/rules/RulesPanel';
import {
  allPeriods,
  BulkBar,
  type BulkBarHandle,
  type LastAction,
} from '@/features/transactions/BulkBar';
import {
  PAGE_SIZE,
  hasActiveFilters,
  queryFilter,
  readState,
  writeState,
} from '@/features/transactions/filterState';
import { RecurrenceDialog, RecurrencesPanel } from '@/features/transactions/Recurrences';
import { TxnDialog, type TxnDialogMode } from '@/features/transactions/TxnDialog';
import { TxnFilters } from '@/features/transactions/TxnFilters';
import { ExchangeDialog } from '@/features/fx/ExchangeDialog';
import { LinkToProjectDialog } from '@/features/projects/ProjectDialogs';
import {
  TransferLinkDialog,
  type TransferLinkMode,
} from '@/features/transactions/TransferLinkDialog';
import { CopySuggestions } from '@/features/transactions/CopySuggestions';
import { TransferSuggestions } from '@/features/transactions/TransferSuggestions';
import { TxnTable } from '@/features/transactions/TxnTable';
import {
  bulkMenuItems,
  isLooseTxn,
  txnMenuItems,
  UNLINKABLE,
} from '@/features/transactions/txnMenu';
import { ConfirmDialog, QueryState } from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { useKeyShortcut } from '@/features/transactions/ui/hooks';
import { TXN_STATUS } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';

const meta = screenMeta('lancamentos');

type Tab = 'lancamentos' | 'recorrencias' | 'regras' | 'importacoes' | 'caixa' | 'email';
const TABS: { id: Tab; readonly label: string }[] = [
  {
    id: 'lancamentos',
    get label() {
      return t('lancamentos-aba-lancamentos');
    },
  },
  {
    id: 'recorrencias',
    get label() {
      return t('lancamentos-aba-recorrencias');
    },
  },
  {
    id: 'regras',
    get label() {
      return t('lancamentos-aba-regras');
    },
  },
  {
    id: 'importacoes',
    get label() {
      return t('lancamentos-aba-importacoes');
    },
  },
  // Caixa de entrada de arquivos (docs/inbox.md).
  {
    id: 'caixa',
    get label() {
      return t('lancamentos-aba-caixa');
    },
  },
  // E-mail de ingestão: a fila de revisão, as contas e as regras (docs/email-ingest.md).
  {
    id: 'email',
    get label() {
      return t('lancamentos-aba-email');
    },
  },
];

export default function Lancamentos() {
  const [params, setParams] = useSearchParams();
  const state = useMemo(() => readState(params), [params]);
  const aba = params.get('aba');
  const tab = TABS.find((x) => x.id === aba)?.id ?? 'lancamentos';
  const [exchangeOpen, setExchangeOpen] = useState(false);
  const query = useTxnQuery(queryFilter(state), tab === 'lancamentos');
  // Lista vazia: o cofre não tem nenhum lançamento (convite para começar) ou só o período e os
  // filtros não acham nada? O período padrão ("Este mês") já é um filtro de datas, então a
  // pergunta vai ao cofre inteiro, só quando a página veio vazia.
  const pageEmpty = query.data?.total === 0;
  const anyTxn = useTxnQuery({ limit: 1, offset: 0 }, tab === 'lancamentos' && pageEmpty);
  const vaultEmpty = pageEmpty && anyTxn.data?.total === 0;
  const page = query.data;

  const [selection, setSelection] = useState<Set<string>>(new Set());
  const [focused, setFocused] = useState<string | null>(null);
  const [dialog, setDialog] = useState<{ mode: TxnDialogMode; txn: TxnView | null } | null>(null);
  const [importOpen, setImportOpen] = useState(false);
  // `?importar=1` (lembrete de contas sem extrato): abre a importação, com `conta` sugerida.
  const importFromUrl = params.get('importar') === '1';
  const importAccount = params.get('conta');
  const [recurrenceOpen, setRecurrenceOpen] = useState(false);
  const [ruleDraft, setRuleDraft] = useState<RuleInput | null>(null);
  const [lastAction, setLastAction] = useState<LastAction | null>(null);
  const [deleting, setDeleting] = useState<TxnView | null>(null);
  const [projecting, setProjecting] = useState<TxnView | null>(null);
  /** Lançamento sendo ligado a outra conta (transferência) ou a um cartão (fatura). */
  const [linking, setLinking] = useState<{ txn: TxnView; mode: TransferLinkMode } | null>(null);
  /** Lançamento sendo ligado a uma parcela de contrato. */
  const [payingLoan, setPayingLoan] = useState<TxnView | null>(null);
  /** Lançamento sendo vinculado a uma operação (carteira, cotas capital, meta). */
  const [operating, setOperating] = useState<TxnView | null>(null);
  /** Lançamento cujos itens estão sendo detalhados em Consumo. */
  const [itemizing, setItemizing] = useState<TxnView | null>(null);
  /** `?detalhar=<id>` (lembrete de Para resolver): abre os itens daquele lançamento. */
  const itemizeFromUrl = params.get('detalhar');
  /** Lançamentos marcados sendo vinculados a uma operação (com confirmação). */
  const [bulkOperating, setBulkOperating] = useState<string[] | null>(null);
  /** Lançamento que originou a marcação pela descrição (para "Criar regra"). */
  const [ruleFrom, setRuleFrom] = useState<TxnView | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const bulk = useRef<BulkBarHandle>(null);
  const undo = useUndo();
  const navigate = useNavigate();
  const setStatus = useSetStatus();
  const remove = useDeleteTxns();
  const linkTransfers = useLinkTransfers();
  const unlinkTransfer = useUnlinkTransfer();
  // Desliga a transferência da perna: as duas ficam soltas e o par não volta como sugestão.
  const unlink = (id: string, after?: () => void) =>
    unlinkTransfer.mutate(
      { txn_id: id },
      {
        onSuccess: (res) => {
          const label = t('lancamentos-transferencia-desligada');
          toast({
            tone: 'success',
            title: label,
            description: t('lancamentos-transferencia-desligada-descricao'),
          });
          setLastAction(res.change_set_id ? { label, changeSetId: res.change_set_id } : null);
          after?.();
        },
        onError: (e) =>
          toast({
            tone: 'error',
            title: t('lancamentos-erro-desligar'),
            description: errorText(e),
          }),
      },
    );
  const { data: accounts = [] } = useAccounts();
  const cardIds = new Set(accounts.filter((a) => a.kind === 'credit_card').map((a) => a.id));
  const hasLoans = (useLoans().data ?? []).length > 0;

  const update = useCallback(
    (next: TxnFilter, nextPage = 1) => {
      setParams(writeState({ filter: next, page: nextPage }, params));
      setSelection(new Set());
      setRuleFrom(null);
    },
    [params, setParams],
  );

  const setTab = (id: Tab) => {
    const next = new URLSearchParams(params);
    if (id === 'lancamentos') next.delete('aba');
    else next.set('aba', id);
    setParams(next);
  };

  const openNew = useCallback(
    (mode: TxnDialogMode = 'expense') => setDialog({ mode, txn: null }),
    [],
  );
  // `?novo=despesa|receita|transferencia|parcelado|ajuste` (painel simples e barra ⌘K): o
  // lançamento novo abre pela URL e,
  // ao fechar, o parâmetro sai.
  const novo = params.get('novo');
  const FROM_URL: Record<string, TxnDialogMode> = {
    despesa: 'expense',
    receita: 'income',
    transferencia: 'transfer',
    parcelado: 'installments',
    ajuste: 'adjustment',
  };
  const fromUrl: TxnDialogMode | null = (novo && FROM_URL[novo]) || null;
  const shown = dialog ?? (fromUrl ? { mode: fromUrl, txn: null } : null);
  const closeDialog = () => {
    setDialog(null);
    if (fromUrl) {
      const next = new URLSearchParams(params);
      next.delete('novo');
      setParams(next, { replace: true });
    }
  };
  const focusSearch = useCallback(() => {
    if (tab !== 'lancamentos') setTab('lancamentos');
    setTimeout(() => searchRef.current?.focus(), 0);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab]);

  const fromTxn = useCallback(
    async (txn: TxnView) => {
      try {
        const draft = await suggestRule(txn.id);
        setDialog(null);
        setRuleDraft(draft);
        const next = new URLSearchParams(params);
        next.set('aba', 'regras');
        setParams(next);
      } catch (e) {
        toast({
          tone: 'error',
          title: t('lancamentos-erro-sugerir-regra'),
          description: errorText(e),
        });
      }
    },
    [params, setParams],
  );

  useKeyShortcut('n', () => openNew());
  useKeyShortcut('/', focusSearch);
  useKeyShortcut('i', () => setImportOpen(true));

  // Contexto para o agente: filtros, totais do filtro, seleção e as primeiras linhas.
  const filters = Object.fromEntries(
    Object.entries(state.filter).filter(([k]) => !['date_from', 'date_to', 'basis'].includes(k)),
  );
  const focusedTxn = page?.items.find((x) => x.id === focused) ?? null;
  usePageContext({
    screen: 'lancamentos',
    title: `${meta.index} · ${meta.title}`,
    view: tab,
    period: {
      from: state.filter.date_from ?? undefined,
      to: state.filter.date_to ?? undefined,
      basis: state.filter.basis ?? undefined,
    },
    filters,
    selection:
      selection.size > 0 || focusedTxn
        ? {
            selected_txn_ids: [...selection].slice(0, 50),
            selected_count: selection.size,
            focused_txn_id: focusedTxn?.id,
          }
        : undefined,
    figures: page
      ? {
          matching: page.total,
          page: state.page,
          ...Object.fromEntries(
            page.totals.flatMap((tot) => [
              [`inflow_${tot.currency}`, money(tot.inflow, tot.currency)],
              [`outflow_${tot.currency}`, money(tot.outflow, tot.currency)],
            ]),
          ),
        }
      : undefined,
    items: page?.items.slice(0, 12).map((x) => ({
      id: x.id,
      date: x.date,
      description: x.description,
      amount: money(x.amount, x.currency),
      account: x.account_name,
      category: x.category_path,
      status: x.status,
    })),
  });

  const commands = useMemo<AppCommand[]>(
    () => [
      {
        id: 'lancamentos.importar',
        title: t('lancamentos-cmd-importar'),
        group: t('casca-tela-lancamentos'),
        icon: 'arrowDown',
        shortcut: ['I'],
        keywords: t('lancamentos-cmd-importar-busca').split(','),
        order: 4,
        perform: () => setImportOpen(true),
      },
      {
        id: 'lancamentos.buscar',
        title: t('lancamentos-buscar'),
        group: t('casca-tela-lancamentos'),
        icon: 'search',
        shortcut: ['/'],
        keywords: t('lancamentos-cmd-buscar-busca').split(','),
        order: 5,
        perform: focusSearch,
      },
      {
        id: 'lancamentos.sem_categoria',
        title: t('lancamentos-cmd-sem-categoria'),
        group: t('casca-tela-lancamentos'),
        icon: 'search',
        keywords: t('lancamentos-cmd-sem-categoria-busca').split(','),
        order: 6,
        perform: () => update({ uncategorized: true }),
      },
      {
        id: 'lancamentos.recorrencia',
        title: t('lancamentos-nova-recorrencia'),
        group: t('casca-tela-lancamentos'),
        icon: 'refresh',
        keywords: t('lancamentos-cmd-recorrencia-busca').split(','),
        order: 7,
        perform: () => setRecurrenceOpen(true),
      },
      {
        id: 'lancamentos.regras',
        title: t('regras-titulo'),
        group: t('casca-tela-lancamentos'),
        icon: 'reticle',
        keywords: t('regras-busca').split(','),
        order: 8,
        perform: () => setTab('regras'),
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [openNew, focusSearch, update],
  );
  useRegisterCommands(commands);

  // Menu de contexto da linha: as mesmas ações do diálogo, da barra de lote e dos atalhos.
  const rowMenu = (row: TxnView, ctx: { categorize?: () => void }) =>
    txnMenuItems(row, {
      open: () => setDialog({ mode: 'expense', txn: row }),
      categorize: ctx.categorize,
      marked: selection.has(row.id),
      toggleMark: () => {
        const next = new Set(selection);
        if (next.has(row.id)) next.delete(row.id);
        else next.add(row.id);
        setSelection(next);
      },
      setStatus: (status: TxnStatus) => {
        const label = t('lancamentos-status-rotulo', {
          status: TXN_STATUS[status].toLowerCase(),
        });
        setStatus.mutate(
          { ids: [row.id], status },
          {
            onSuccess: (res) => {
              toast({
                tone: 'success',
                title: label,
                description: t('lancamentos-n-alterados', { n: res.updated }),
              });
              setLastAction(res.change_set_id ? { label, changeSetId: res.change_set_id } : null);
            },
            onError: (e) =>
              toast({
                tone: 'error',
                title: t('lancamentos-erro-status'),
                description: errorText(e),
              }),
          },
        );
      },
      openAccount: () => void navigate(`/contas?conta=${encodeURIComponent(row.account_id)}`),
      filterAccount: () => update({ ...state.filter, account_ids: [row.account_id] }),
      suggestRule: () => void fromTxn(row),
      setProject: () => setProjecting(row),
      markSameDescription: () => {
        sameDescription([row.id], allPeriods(state.filter))
          .then((r) => {
            setSelection(new Set(r.ids));
            setRuleFrom(row);
            toast({
              tone: 'success',
              title: t('lancamentos-n-marcados-toast', { n: r.ids.length }),
              description: t('lancamentos-mesma-descricao-toast', {
                descricao: r.descriptions[0] ?? row.description,
              }),
            });
          })
          .catch((e: unknown) =>
            toast({
              tone: 'error',
              title: t('lancamentos-erro-marcar'),
              description: errorText(e),
            }),
          );
      },
      linkTransfer: isLooseTxn(row) ? () => setLinking({ txn: row, mode: 'transfer' }) : undefined,
      unlinkTransfer:
        row.transfer_id && UNLINKABLE.has(row.kind) ? () => unlink(row.id) : undefined,
      payCard:
        isLooseTxn(row) && row.amount < 0 && !cardIds.has(row.account_id) && cardIds.size > 0
          ? () => setLinking({ txn: row, mode: 'card' })
          : undefined,
      payLoan:
        hasLoans &&
        row.amount < 0 &&
        !row.transfer_id &&
        (row.kind === 'expense' || row.kind === 'transfer')
          ? () => setPayingLoan(row)
          : undefined,
      // Vinculado, ou solto sem parcela e sem divisão (o diálogo explica o resto).
      linkOperation:
        row.operation ||
        (!row.transfer_id &&
          !row.loan_id &&
          row.splits.length === 0 &&
          row.status !== 'planned' &&
          (row.kind === 'expense' ||
            row.kind === 'income' ||
            row.kind === 'transfer' ||
            row.kind === 'investment'))
          ? () => setOperating(row)
          : undefined,
      // Despesa de verdade (não é perna de transferência nem pagamento de fatura).
      itemize:
        row.kind === 'expense' && !row.transfer_id && row.status !== 'planned'
          ? () => setItemizing(row)
          : undefined,
      remove: () => setDeleting(row),
    });

  const total = page?.total ?? 0;
  const pages = Math.max(1, Math.ceil(total / PAGE_SIZE));
  const firstRow = total === 0 ? 0 : (state.page - 1) * PAGE_SIZE + 1;
  const lastRow = Math.min(total, state.page * PAGE_SIZE);
  const selectedIds = [...selection];
  const selectedSingle =
    selectedIds.length === 1 ? page?.items.find((x) => x.id === selectedIds[0]) : undefined;
  const clearSelection = () => {
    setSelection(new Set());
    setRuleFrom(null);
  };
  const ruleSource = selectedSingle ?? ruleFrom;

  // Duas marcadas, uma saída e uma entrada soltas em contas diferentes: o par de uma
  // transferência que chegou pelos dois extratos.
  const marked = (page?.items ?? []).filter((x) => selection.has(x.id));
  const [out, inn] = [...marked].sort((a, b) => a.amount - b.amount);
  const pair =
    selection.size === 2 &&
    out &&
    inn &&
    out.amount < 0 &&
    inn.amount > 0 &&
    out.account_id !== inn.account_id &&
    isLooseTxn(out) &&
    isLooseTxn(inn)
      ? { from_id: out.id, to_id: inn.id }
      : null;
  // Duas marcadas que não formam o par: o motivo, para a barra e o menu dizerem.
  const sameTransfer =
    selection.size === 2 &&
    out?.transfer_id != null &&
    out.transfer_id === inn?.transfer_id &&
    UNLINKABLE.has(out.kind)
      ? out
      : null;
  const pairIssue =
    selection.size !== 2 || pair || sameTransfer
      ? undefined
      : marked.length < 2
        ? t('lancamentos-par-mesma-pagina')
        : marked.some((x) => x.transfer_id)
          ? t('lancamentos-par-ja-ligada')
          : !(out!.amount < 0 && inn!.amount > 0)
            ? t('lancamentos-par-sentidos')
            : out!.account_id === inn!.account_id
              ? t('lancamentos-par-mesma-conta')
              : out!.currency === inn!.currency && -out!.amount !== inn!.amount
                ? t('lancamentos-par-valores')
                : t('lancamentos-par-tipos');
  const linkPair = () =>
    pair &&
    linkTransfers.mutate(
      { pairs: [pair] },
      {
        onSuccess: (res) => {
          const label = t('lancamentos-transferencia-ligada');
          toast({
            tone: 'success',
            title: label,
            description: t('lancamentos-transferencia-ligada-descricao'),
          });
          setLastAction(res.change_set_id ? { label, changeSetId: res.change_set_id } : null);
          clearSelection();
        },
        onError: (e) =>
          toast({ tone: 'error', title: t('lancamentos-erro-ligar'), description: errorText(e) }),
      },
    );

  // Menu de várias linhas marcadas: as ações da barra de lote (mesmos seletores e confirmação).
  const selectionMenu = () =>
    bulkMenuItems(selection.size, {
      // Pelo `c` (ou pelo menu), o seletor abre junto do mouse, não na barra lá em cima.
      categorize: () => bulk.current?.categorize(pointerOrFocus()),
      setStatus: (s) => bulk.current?.setStatus(s),
      member: () => bulk.current?.member(),
      tags: () => bulk.current?.tags(),
      project: () => bulk.current?.project(),
      markSame: () => bulk.current?.markSame(),
      markAll: () => bulk.current?.markAll(),
      filterTotal: total,
      suggestRule: ruleSource ? () => void fromTxn(ruleSource) : undefined,
      linkPair: pair ? linkPair : undefined,
      pairIssue,
      unlinkPair: sameTransfer ? () => unlink(sameTransfer.id, clearSelection) : undefined,
      linkOperation: () => setBulkOperating(selectedIds),
      clear: clearSelection,
      remove: () => bulk.current?.remove(),
    });

  // Os cupons fiscais moraram numa aba daqui até a 0.8; os links antigos vão para Consumo › Compras.
  if (aba === 'cupons') return <Navigate to="/consumo?aba=compras" replace />;

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={t('lancamentos-descricao')}
        actions={
          <>
            <Button
              variant="ghost"
              icon={<Icon name="arrowDown" size={13} />}
              onClick={() => useImportHub.getState().setOpen(true)}
            >
              {t('lancamentos-importar')}
            </Button>
            <Button variant="secondary" onClick={() => openNew('transfer')}>
              {t('lancamentos-transferencia')}
            </Button>
            <Button variant="secondary" onClick={() => setExchangeOpen(true)}>
              {t('lancamentos-cambio')}
            </Button>
            <Button
              variant="primary"
              icon={<Icon name="plus" size={13} />}
              onClick={() => openNew()}
            >
              {t('lancamentos-novo')} <Kbd keys={['N']} tone="onAccent" className="ml-1" />
            </Button>
          </>
        }
      />

      <Tabs value={tab} onValueChange={(v) => setTab(v as Tab)}>
        <TabsList aria-label={t('lancamentos-abas')}>
          {TABS.map((x) => (
            <TabsTrigger key={x.id} value={x.id}>
              {x.label}
            </TabsTrigger>
          ))}
        </TabsList>

        <TabsContent value="lancamentos" className="flex flex-col gap-4">
          <CopySuggestions />
          <TransferSuggestions />
          <TxnFilters filter={state.filter} onChange={(f) => update(f)} searchRef={searchRef} />

          {selection.size > 0 ? (
            <BulkBar
              ref={bulk}
              ids={selectedIds}
              filter={state.filter}
              total={total}
              onSelect={(ids, from) => {
                setSelection(new Set(ids));
                const src = from ? page?.items.find((x) => x.id === from) : undefined;
                if (src) setRuleFrom(src);
              }}
              onClear={clearSelection}
              onDone={setLastAction}
              onSuggestRule={ruleSource ? () => void fromTxn(ruleSource) : undefined}
              onLinkOperation={() => setBulkOperating(selectedIds)}
              transferPair={
                selection.size === 2
                  ? {
                      onLink: pair ? linkPair : undefined,
                      onUnlink: sameTransfer
                        ? () => unlink(sameTransfer.id, clearSelection)
                        : undefined,
                      issue: pairIssue,
                      pending: linkTransfers.isPending || unlinkTransfer.isPending,
                    }
                  : undefined
              }
            />
          ) : null}
          {lastAction ? (
            <div role="status" className="flex items-center gap-3 text-[12.5px] text-dim">
              <span>
                <span className="label-caps mr-2">{t('lancamentos-feito')}</span>
                {lastAction.label}
              </span>
              <Button
                size="sm"
                variant="ghost"
                loading={undo.isPending}
                onClick={() =>
                  undo.mutate(
                    { id: lastAction.changeSetId },
                    {
                      onSuccess: () => {
                        toast({
                          tone: 'success',
                          title: t('lancamentos-desfeito'),
                          description: lastAction.label,
                        });
                        setLastAction(null);
                      },
                      onError: (e) =>
                        toast({
                          tone: 'error',
                          title: t('lancamentos-erro-desfazer'),
                          description: errorText(e),
                        }),
                    },
                  )
                }
              >
                {t('lancamentos-desfazer')}
              </Button>
              <button
                type="button"
                className="label-caps hover:text-fg"
                onClick={() => setLastAction(null)}
              >
                {t('lancamentos-dispensar')}
              </button>
            </div>
          ) : null}

          <InstrumentCard
            index="04.1"
            title={t('lancamentos-aba-lancamentos')}
            padded={false}
            live={query.isFetching && !query.isLoading ? t('lancamentos-atualizando') : undefined}
            actions={
              <span className="label-caps num" aria-live="polite">
                {total === 0
                  ? t('lancamentos-zero')
                  : t('lancamentos-faixa', { de: firstRow, ate: lastRow, total })}
              </span>
            }
            footer={
              page && page.totals.length > 0 ? (
                <dl
                  className="flex flex-wrap items-baseline gap-x-8 gap-y-2"
                  aria-label={t('lancamentos-totais')}
                >
                  {page.totals.map((tot) => (
                    <div
                      key={tot.currency}
                      className="flex flex-wrap items-baseline gap-x-4 gap-y-1"
                    >
                      <span className="label-caps text-fg">{tot.currency}</span>
                      <span className="flex items-baseline gap-2">
                        <dt className="label-caps">{t('lancamentos-entradas')}</dt>
                        <dd>
                          <Money
                            cents={tot.inflow}
                            currency={tot.currency}
                            tone="positive"
                            signed
                          />
                        </dd>
                      </span>
                      <span className="flex items-baseline gap-2">
                        <dt className="label-caps">{t('lancamentos-saidas')}</dt>
                        <dd>
                          <Money cents={-tot.outflow} currency={tot.currency} tone="negative" />
                        </dd>
                      </span>
                      <span className="flex items-baseline gap-2">
                        <dt className="label-caps">{t('lancamentos-resultado')}</dt>
                        <dd>
                          <Money
                            cents={tot.net}
                            currency={tot.currency}
                            tone="auto"
                            className="font-medium"
                          />
                        </dd>
                      </span>
                    </div>
                  ))}
                  <span className="label-caps ml-auto">{t('lancamentos-totais-nota')}</span>
                </dl>
              ) : undefined
            }
          >
            <QueryState
              loading={query.isLoading}
              error={query.error}
              onRetry={() => void query.refetch()}
              label={t('lancamentos-carregando')}
            >
              {page && page.total === 0 && (vaultEmpty || !hasActiveFilters(state.filter)) ? (
                <div className="p-4">
                  <EmptyState
                    kicker={t('lancamentos-vazio-kicker')}
                    title={t('lancamentos-vazio-titulo')}
                    description={t('lancamentos-vazio-descricao')}
                    actions={
                      <>
                        <Button variant="primary" onClick={() => setImportOpen(true)}>
                          {t('lancamentos-importar-extrato')}
                        </Button>
                        <Button onClick={() => openNew()}>{t('lancamentos-novo')}</Button>
                      </>
                    }
                  />
                </div>
              ) : page ? (
                <TxnTable
                  page={page}
                  sort={state.filter.sort ?? 'date_desc'}
                  onSortChange={(sort) => update({ ...state.filter, sort })}
                  selection={selection}
                  onSelectionChange={(next) => (next.size ? setSelection(next) : clearSelection())}
                  onOpen={(txn) => setDialog({ mode: 'expense', txn })}
                  focusedId={focused}
                  onFocusedChange={setFocused}
                  rowMenu={rowMenu}
                  selectionMenu={selectionMenu}
                />
              ) : null}
            </QueryState>
            {pages > 1 ? (
              <nav
                aria-label={t('lancamentos-paginas')}
                className="flex items-center justify-end gap-2 border-t border-line px-3 py-2"
              >
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={state.page <= 1}
                  onClick={() => update(state.filter, state.page - 1)}
                >
                  {t('lancamentos-anterior')}
                </Button>
                <span className="num text-[12px] text-dim">
                  {t('lancamentos-pagina-de', { pagina: state.page, paginas: pages })}
                </span>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={state.page >= pages}
                  onClick={() => update(state.filter, state.page + 1)}
                >
                  {t('lancamentos-proxima')}
                </Button>
              </nav>
            ) : null}
          </InstrumentCard>
          <p className="label-caps flex flex-wrap gap-x-4 gap-y-1">
            <span>
              <Kbd keys={['N']} /> {t('lancamentos-atalho-novo')}
            </span>
            <span>
              <Kbd keys={['/']} /> {t('lancamentos-atalho-buscar')}
            </span>
            <span>
              <Kbd keys={['I']} /> {t('lancamentos-atalho-importar')}
            </span>
            <span>
              <Kbd keys={['↑']} /> <Kbd keys={['↓']} /> {t('lancamentos-atalho-navegar')}
            </span>
            <span>
              <Kbd keys={['Espaço']} /> {t('lancamentos-atalho-marcar')}
            </span>
            <span>
              <Kbd keys={['shift', '↑']} /> <Kbd keys={['shift', '↓']} />{' '}
              {t('lancamentos-atalho-marcar-faixa')}
            </span>
            <span>
              <Kbd keys={['C']} /> {t('lancamentos-atalho-categoria')}
            </span>
            <span>
              <Kbd keys={['enter']} /> {t('lancamentos-atalho-abrir')}
            </span>
          </p>
        </TabsContent>

        <TabsContent value="recorrencias">
          <RecurrencesPanel onNew={() => setRecurrenceOpen(true)} />
        </TabsContent>
        <TabsContent value="regras" className="flex flex-col gap-4">
          <LearnedRules />
          <RulesPanel
            onNew={() =>
              setRuleDraft({
                name: '',
                priority: 100,
                active: true,
                match_all: true,
                conditions: [{ field: 'description', op: 'contains', value: '' }],
                actions: {},
              })
            }
            draft={ruleDraft}
            onDraftConsumed={() => setRuleDraft(null)}
            filter={state.filter}
          />
        </TabsContent>
        <TabsContent value="importacoes">
          <ImportsPanel onImport={() => setImportOpen(true)} />
        </TabsContent>
        <TabsContent value="caixa">
          <InboxPanel />
        </TabsContent>
        <TabsContent value="email">
          <MailPanel />
        </TabsContent>
      </Tabs>

      <TxnDialog
        open={Boolean(shown)}
        onOpenChange={(o) => !o && closeDialog()}
        txn={shown?.txn}
        initialMode={shown?.mode}
        initialAccountId={
          state.filter.account_ids?.length === 1 ? state.filter.account_ids[0] : undefined
        }
        onSuggestRule={(txn) => void fromTxn(txn)}
      />
      <ImportWizard
        open={importOpen || importFromUrl}
        onOpenChange={(o) => {
          setImportOpen(o);
          if (!o && importFromUrl) {
            const next = new URLSearchParams(params);
            next.delete('importar');
            next.delete('conta');
            setParams(next, { replace: true });
          }
        }}
        initialAccountId={
          (importFromUrl ? importAccount : null) ??
          (state.filter.account_ids?.length === 1 ? state.filter.account_ids[0] : undefined)
        }
      />
      <RecurrenceDialog open={recurrenceOpen} onOpenChange={setRecurrenceOpen} recurrence={null} />
      {exchangeOpen ? <ExchangeDialog open onOpenChange={setExchangeOpen} /> : null}
      {payingLoan ? (
        <LoanPaymentDialog
          txn={payingLoan}
          open
          onOpenChange={(o) => (!o ? setPayingLoan(null) : undefined)}
        />
      ) : null}
      {bulkOperating ? (
        <BulkOperationLinkDialog
          ids={bulkOperating}
          open
          onOpenChange={(o) => (!o ? setBulkOperating(null) : undefined)}
          onDone={(label, changeSetId) => {
            setLastAction(changeSetId ? { label, changeSetId } : null);
            clearSelection();
          }}
        />
      ) : null}
      {itemizing ? <ItemizeTxn txn={itemizing} onClose={() => setItemizing(null)} /> : null}
      {itemizeFromUrl && !itemizing ? (
        <ItemizeTxnById
          id={itemizeFromUrl}
          onClose={() => {
            const p = new URLSearchParams(params);
            p.delete('detalhar');
            setParams(p, { replace: true });
          }}
        />
      ) : null}
      {operating ? (
        <OperationLinkDialog
          txn={operating}
          open
          onOpenChange={(o) => (!o ? setOperating(null) : undefined)}
          onDone={(label, changeSetId) =>
            setLastAction(changeSetId ? { label, changeSetId } : null)
          }
        />
      ) : null}
      {linking ? (
        <TransferLinkDialog
          txn={linking.txn}
          mode={linking.mode}
          open
          onOpenChange={(o) => (!o ? setLinking(null) : undefined)}
        />
      ) : null}
      <LinkToProjectDialog
        ids={projecting ? [projecting.id] : []}
        current={projecting?.project_id ?? null}
        open={projecting != null}
        onOpenChange={(o) => (!o ? setProjecting(null) : undefined)}
        onDone={(label, changeSetId) => setLastAction(changeSetId ? { label, changeSetId } : null)}
      />
      <ConfirmDialog
        open={Boolean(deleting)}
        onOpenChange={(o) => !o && setDeleting(null)}
        title={t('lancamentos-apagar-1')}
        description={t('lancamentos-apagar-1-descricao', {
          descricao: deleting?.description || t('lancamentos-sem-descricao'),
        })}
        confirmLabel={t('comum-apagar')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          deleting &&
          remove.mutate(
            { ids: [deleting.id] },
            {
              onSuccess: (res) => {
                const label = t('lancamentos-apagados');
                setDeleting(null);
                setSelection((s) => new Set([...s].filter((id) => id !== deleting.id)));
                toast({
                  tone: 'success',
                  title: label,
                  description: t('lancamentos-n-alterados', { n: res.updated }),
                });
                setLastAction(res.change_set_id ? { label, changeSetId: res.change_set_id } : null);
              },
            },
          )
        }
      />
    </div>
  );
}
