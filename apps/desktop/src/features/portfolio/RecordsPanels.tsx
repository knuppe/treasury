/**
 * Tabelas de registros da carteira: operações (com editar/excluir), eventos
 * corporativos (com a origem: manual ou coletado do Yahoo) e proventos.
 * Reusadas na tela da carteira e na página do ativo (filtradas pelo ativo).
 *
 * As três selecionam várias linhas (Shift+↑/↓, Shift+clique, ⌘/Ctrl+clique, ⌘A) e têm
 * menu de contexto com atalhos que valem sem abrir o menu: copiar as linhas (⌘C) e
 * excluir (⌘⌫). Excluir várias vai num conjunto de alterações só: um "Desfazer" volta todas.
 */
import { useCallback, useMemo, useRef, useState } from 'react';
import type { UseMutationResult } from '@tanstack/react-query';

import { useAssets } from '@/api/hooks/assets';
import {
  useActions,
  useDeleteActions,
  useDeleteIncomes,
  useDeleteTrades,
  useIncome,
  useInvestmentAccounts,
  useTrades,
} from '@/api/hooks/portfolio';
import { useUndoToast } from '@/api/hooks/undoToast';
import type { BulkResult } from '@/api/generated/BulkResult';
import type { CorporateAction } from '@/api/generated/CorporateAction';
import type { IncomeEvent } from '@/api/generated/IncomeEvent';
import type { Trade } from '@/api/generated/Trade';
import {
  Button,
  Dialog,
  DialogClose,
  Icon,
  Money,
  OriginBadge,
  Private,
  Select,
  TechTable,
  toast,
  type ContextMenuEntry,
  type TechColumn,
  type TechTableMenuContext,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';
import { formatDate, formatMoney } from '@/lib/format';

import { ErrorBlock, errorMessage, LoadingBlock } from './components/States';
import { formatDecimal, formatQuantity, formatUnitPrice } from './decimal';
import { ACTION_KIND_LABEL, assetLabel, INCOME_KIND_LABEL, safeDate, sourceName } from './labels';
import { TradeDialog } from './TradeDialog';

/** Linhas listadas na confirmação de excluir (o resto vira "e mais N"). */
const LISTED = 8;

interface DeleteText<T> {
  /** `Excluir operação` / `Excluir 3 operações`. */
  title: (n: number) => string;
  /** Uma linha por registro na confirmação. */
  item: (row: T) => string;
  /** O que sai junto (lançamentos ligados, recálculo) e como desfazer. */
  note: (n: number) => string;
  /** Aviso depois de excluir: `3 operações excluídas`. */
  done: (n: number) => string;
}

/**
 * Excluir um ou vários registros com confirmação (a lista e a contagem). O lote vai num
 * conjunto de alterações só, e o aviso oferece "Desfazer" quando o núcleo devolve o id.
 */
export function useDeleteRecords<T extends { id: string }>(
  mutation: UseMutationResult<BulkResult, Error, { ids: string[] }>,
  text: DeleteText<T>,
) {
  // As linhas ficam depois de fechar (o título não muda enquanto o diálogo some).
  const [rows, setRows] = useState<T[]>([]);
  const [open, setOpen] = useState(false);
  const origin = useRef<HTMLElement | null>(null);
  const undoToast = useUndoToast();
  const n = rows.length;
  const ask = useCallback((list: T[]) => {
    origin.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setRows(list);
    setOpen(true);
  }, []);
  // Sem gatilho, o diálogo não devolve o foco: volta à linha (ou à tabela, se ela saiu).
  const close = () => {
    setOpen(false);
    const from = origin.current;
    const grid = from?.closest('[role="grid"]');
    setTimeout(() => {
      const to = from?.isConnected ? from : grid?.querySelector<HTMLElement>('tr[tabindex="0"]');
      to?.focus({ preventScroll: true });
    }, 0);
  };
  const confirm = () => {
    mutation.mutate(
      { ids: rows.map((r) => r.id) },
      {
        onSuccess: (res) => {
          close();
          undoToast({ title: text.done(res.updated), changeSetId: res.change_set_id });
        },
        onError: (e) =>
          toast({
            tone: 'error',
            title: t('carteira-erro-excluir'),
            description: errorMessage(e),
          }),
      },
    );
  };
  const dialog = (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next) close();
      }}
      size="sm"
      title={text.title(n)}
      description={text.note(n)}
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button variant="danger" loading={mutation.isPending} onClick={confirm}>
            {t('carteira-excluir-n', { n })}
          </Button>
        </>
      }
    >
      <ul
        aria-label={t('carteira-registros-excluir')}
        className="flex max-h-48 flex-col gap-1 overflow-y-auto text-[12.5px] text-fg"
      >
        {rows.slice(0, LISTED).map((r) => (
          <li key={r.id} className="truncate">
            {text.item(r)}
          </li>
        ))}
        {n > LISTED ? (
          <li className="text-dim">{t('carteira-e-mais', { n: n - LISTED })}</li>
        ) : null}
      </ul>
    </Dialog>
  );
  return { ask, dialog };
}

/**
 * Menus de contexto de uma tabela de registros: o da linha (editar, copiar, excluir) e o
 * de várias linhas (copiar e excluir todas). Os atalhos mostrados valem sem abrir o menu.
 */
function recordMenus<T>(opts: {
  /** Título do menu da linha (o registro em poucas palavras). */
  heading: (row: T) => string;
  /** `3 operações selecionadas`, `3 proventos selecionados`… */
  selected: (n: number) => string;
  /** `Excluir 3 operações…` */
  removeMany: (n: number) => string;
  /** Nome do menu da seleção: `Ações de 3 operações`. */
  actionsOf: (n: number) => string;
  edit?: (row: T) => void;
  remove: (rows: T[]) => void;
}) {
  return {
    rowMenu: (row: T, ctx: TechTableMenuContext): ContextMenuEntry[] => [
      { heading: opts.heading(row) },
      opts.edit && {
        label: t('carteira-menu-editar'),
        icon: 'sliders',
        shortcut: ['enter'],
        onSelect: () => opts.edit?.(row),
      },
      {
        label: t('carteira-menu-copiar-linha'),
        icon: 'copy',
        shortcut: ['mod', 'C'],
        onSelect: ctx.copy,
      },
      { separator: true },
      {
        label: t('carteira-menu-excluir'),
        icon: 'x',
        danger: true,
        shortcut: ['mod', 'Backspace'],
        onSelect: () => opts.remove([row]),
      },
    ],
    selectionMenu: (rows: T[], ctx: TechTableMenuContext): ContextMenuEntry[] => [
      { heading: opts.selected(rows.length) },
      {
        label: t('carteira-menu-copiar-n-linhas', { n: rows.length }),
        icon: 'copy',
        shortcut: ['mod', 'C'],
        onSelect: ctx.copy,
      },
      { separator: true },
      {
        label: opts.removeMany(rows.length),
        icon: 'x',
        danger: true,
        shortcut: ['mod', 'Backspace'],
        onSelect: () => opts.remove(rows),
      },
    ],
    selectionMenuLabel: (rows: T[]) => opts.actionsOf(rows.length),
  };
}

/** Soma de uma moeda só (com moedas misturadas não há total). */
function total<T>(
  rows: readonly T[],
  currency: (row: T) => string,
  cents: (row: T) => number,
): { cents: number; currency: string } | null {
  const first = rows[0];
  if (!first || rows.some((r) => currency(r) !== currency(first))) return null;
  return { cents: rows.reduce((s, r) => s + cents(r), 0), currency: currency(first) };
}

/** Linha de status da seleção: a contagem e, quando há, somas. */
function SelectionLine({
  count,
  sums = [],
}: {
  count: string;
  sums?: { label: string; value: { cents: number; currency: string } | null }[];
}) {
  return (
    <>
      <span className="num text-accent">{count}</span>
      {sums.map((s) =>
        s.value ? (
          <span key={s.label} className="flex items-baseline gap-1.5">
            <span className="label-caps">{s.label}</span>
            <Money cents={s.value.cents} currency={s.value.currency} />
          </span>
        ) : null,
      )}
    </>
  );
}

function SideBadge({ side }: { side: Trade['side'] }) {
  return (
    <span className={cx('whitespace-nowrap', side === 'buy' ? 'text-positive' : 'text-negative')}>
      <span aria-hidden>{side === 'buy' ? '▲ ' : '▼ '}</span>
      {side === 'buy' ? t('carteira-compra') : t('carteira-venda')}
    </span>
  );
}

const ALL = '__all__';

export const tradeText = (tr: Trade) => {
  // Renda fixa: "Aplicação de R$ 5.000,00 em CDB Itaú", sem quantidade.
  if (tr.by_value) {
    const args = {
      valor: formatMoney(tr.gross, tr.currency),
      ativo: tr.ticker ?? tr.asset_name,
      data: formatDate(tr.date),
    };
    return tr.side === 'buy' ? t('carteira-aplicacao-de', args) : t('carteira-resgate-de', args);
  }
  const args = {
    qtd: formatQuantity(tr.quantity),
    ativo: tr.ticker ?? tr.asset_name,
    data: formatDate(tr.date),
  };
  return tr.side === 'buy' ? t('carteira-compra-de', args) : t('carteira-venda-de', args);
};

export function TradesTable({ assetId, compact = false }: { assetId?: string; compact?: boolean }) {
  const [account, setAccount] = useState<string>(ALL);
  const [asset, setAsset] = useState<string>(assetId ?? ALL);
  const filter = {
    ...(asset !== ALL ? { asset_id: asset } : {}),
    ...(account !== ALL ? { account_id: account } : {}),
  };
  const q = useTrades(filter);
  const accounts = useInvestmentAccounts();
  const assets = useAssets({ held_only: true, include_archived: true }, { enabled: !assetId });
  const [editing, setEditing] = useState<Trade | null>(null);
  const { ask: askDelete, dialog: deleteDialog } = useDeleteRecords<Trade>(useDeleteTrades(), {
    title: (n) => t('carteira-excluir-operacoes', { n }),
    item: tradeText,
    note: (n) => t('carteira-excluir-operacoes-nota', { n }),
    done: (n) => t('carteira-operacoes-excluidas', { n }),
  });
  const menus = recordMenus<Trade>({
    heading: tradeText,
    selected: (n) => t('carteira-operacoes-selecionadas', { n }),
    removeMany: (n) => t('carteira-excluir-n-operacoes', { n }),
    actionsOf: (n) => t('carteira-acoes-de-n-operacoes', { n }),
    edit: setEditing,
    remove: askDelete,
  });

  const columns = useMemo<TechColumn<Trade>[]>(
    () => [
      {
        key: 'date',
        header: t('carteira-col-data'),
        cell: (tr) => formatDate(tr.date),
        sortValue: (tr) => tr.date,
        width: '96px',
      },
      ...(assetId
        ? []
        : [
            {
              key: 'asset',
              header: t('carteira-col-ativo'),
              cell: (tr: Trade) => (
                <span className="font-medium">{tr.ticker ?? tr.asset_name}</span>
              ),
              sortValue: (tr: Trade) => tr.ticker ?? tr.asset_name,
            },
          ]),
      { key: 'side', header: t('carteira-col-tipo'), cell: (tr) => <SideBadge side={tr.side} /> },
      {
        key: 'qty',
        header: t('carteira-col-qtd'),
        numeric: true,
        cell: (tr) => (tr.by_value ? '—' : <Private>{formatQuantity(tr.quantity)}</Private>),
      },
      {
        key: 'price',
        header: t('carteira-col-preco'),
        numeric: true,
        cell: (tr) =>
          tr.by_value ? '—' : <Private>{formatUnitPrice(tr.price, tr.currency)}</Private>,
      },
      {
        key: 'costs',
        header: t('carteira-col-custos'),
        numeric: true,
        cell: (tr) => <Money cents={tr.fees + tr.taxes} currency={tr.currency} />,
      },
      {
        key: 'gross',
        header: t('carteira-col-bruto'),
        numeric: true,
        cell: (tr) => <Money cents={tr.gross} currency={tr.currency} />,
      },
      {
        key: 'cash',
        header: t('carteira-col-caixa'),
        numeric: true,
        cell: (tr) =>
          tr.txn_id ? (
            <Money cents={tr.cash_amount} currency={tr.currency} tone="auto" />
          ) : (
            <span className="text-dim">{t('carteira-fora-do-caixa')}</span>
          ),
      },
      ...(compact
        ? []
        : [
            {
              key: 'fx',
              header: t('carteira-col-cambio'),
              numeric: true,
              cell: (tr: Trade) =>
                tr.fx_rate ? formatDecimal(tr.fx_rate, { minDecimals: 4 }) : '—',
            },
            {
              key: 'account',
              header: t('carteira-col-conta'),
              cell: (tr: Trade) => <span className="font-sans text-dim">{tr.account_name}</span>,
            },
            {
              key: 'note',
              header: t('carteira-col-nota'),
              cell: (tr: Trade) => tr.note_number ?? '—',
            },
          ]),
      {
        key: 'actions',
        header: <span className="sr-only">{t('carteira-col-acoes')}</span>,
        align: 'right',
        cell: (tr) => (
          <span className="flex justify-end gap-1" onKeyDown={(e) => e.stopPropagation()}>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setEditing(tr)}
              aria-label={t('carteira-editar-operacao-de', { data: formatDate(tr.date) })}
            >
              {t('comum-editar')}
            </Button>
            <Button size="sm" variant="ghost" onClick={() => askDelete([tr])}>
              {t('carteira-excluir')}
            </Button>
          </span>
        ),
      },
    ],
    [assetId, compact, askDelete],
  );

  return (
    <div className="flex flex-col gap-3">
      {!assetId ? (
        <div className="flex flex-wrap items-center gap-3">
          <div className="w-60">
            <Select
              aria-label={t('carteira-filtrar-ativo')}
              value={asset}
              onValueChange={setAsset}
              options={[
                { value: ALL, label: t('carteira-todos-ativos') },
                ...(assets.data ?? []).map((a) => ({ value: a.id, label: assetLabel(a) })),
              ]}
            />
          </div>
          <div className="w-56">
            <Select
              aria-label={t('carteira-filtrar-conta')}
              value={account}
              onValueChange={setAccount}
              options={[
                { value: ALL, label: t('carteira-todas-contas') },
                ...(accounts.data ?? []).map((a) => ({ value: a.id, label: a.name })),
              ]}
            />
          </div>
          <span className="ml-auto text-[12px] text-dim">
            {q.data ? t('carteira-n-operacoes', { n: q.data.length }) : ''}
          </span>
        </div>
      ) : null}
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : (
        <div className="rounded-card border border-line bg-surface">
          <TechTable
            label={t('carteira-operacoes')}
            prefsId="Operações"
            rows={q.data}
            columns={columns}
            getRowId={(tr) => tr.id}
            onActivate={setEditing}
            maxHeight="520px"
            empty={t('carteira-sem-operacoes')}
            multiSelect
            {...menus}
            rowMenuLabel={(tr) => t('carteira-acoes-de', { nome: tradeText(tr) })}
            selectionStatus={(rows) => (
              <SelectionLine
                count={t('carteira-operacoes-selecionadas', { n: rows.length })}
                sums={[
                  {
                    label: t('carteira-compras'),
                    value: total(
                      rows.filter((tr) => tr.side === 'buy'),
                      (tr) => tr.currency,
                      (tr) => tr.gross,
                    ),
                  },
                  {
                    label: t('carteira-vendas'),
                    value: total(
                      rows.filter((tr) => tr.side === 'sell'),
                      (tr) => tr.currency,
                      (tr) => tr.gross,
                    ),
                  },
                ]}
              />
            )}
          />
        </div>
      )}
      <TradeDialog
        open={editing != null}
        onOpenChange={(o) => !o && setEditing(null)}
        trade={editing}
      />
      {deleteDialog}
    </div>
  );
}

export function ActionsTable({ assetId }: { assetId?: string }) {
  const q = useActions(assetId);
  const assets = useAssets({ include_archived: true });
  const nameOf = (id: string | null) => {
    if (!id) return '—';
    const a = assets.data?.find((x) => x.id === id);
    return a ? assetLabel(a) : id;
  };
  const actionText = (a: CorporateAction) =>
    t('carteira-evento-texto', {
      tipo: ACTION_KIND_LABEL[a.kind],
      ativo: nameOf(a.asset_id),
      data: formatDate(a.date),
    });
  const removal = useDeleteRecords<CorporateAction>(useDeleteActions(), {
    title: (n) => t('carteira-excluir-eventos', { n }),
    item: actionText,
    note: () => t('carteira-excluir-eventos-nota'),
    done: (n) => t('carteira-eventos-excluidos', { n }),
  });
  const menus = recordMenus<CorporateAction>({
    heading: actionText,
    selected: (n) => t('carteira-eventos-selecionados', { n }),
    removeMany: (n) => t('carteira-excluir-n-eventos', { n }),
    actionsOf: (n) => t('carteira-acoes-de-n-eventos', { n }),
    remove: removal.ask,
  });
  const columns: TechColumn<CorporateAction>[] = [
    {
      key: 'date',
      header: t('carteira-col-data-ex'),
      cell: (a) => formatDate(a.date),
      sortValue: (a) => a.date,
      width: '96px',
    },
    ...(assetId
      ? []
      : [
          {
            key: 'asset',
            header: t('carteira-col-ativo'),
            cell: (a: CorporateAction) => <span className="font-medium">{nameOf(a.asset_id)}</span>,
          },
        ]),
    {
      key: 'kind',
      header: t('carteira-col-evento'),
      cell: (a) => <span className="font-sans">{ACTION_KIND_LABEL[a.kind]}</span>,
    },
    {
      key: 'detail',
      header: t('carteira-col-detalhe'),
      cell: (a) =>
        a.kind === 'capital_return' && a.amount != null ? (
          <Money cents={a.amount} />
        ) : a.new_asset_id ? (
          `→ ${nameOf(a.new_asset_id)}${a.factor ? ` × ${formatDecimal(a.factor)}` : ''}`
        ) : a.factor ? (
          a.unit_cost ? (
            t('carteira-fator-custo', {
              fator: formatDecimal(a.factor),
              custo: formatUnitPrice(a.unit_cost),
            })
          ) : (
            t('carteira-fator', { fator: formatDecimal(a.factor) })
          )
        ) : (
          '—'
        ),
    },
    {
      key: 'notes',
      header: t('carteira-col-observacoes'),
      cell: (a) => <span className="font-sans text-dim">{a.notes ?? ''}</span>,
    },
    {
      key: 'source',
      header: t('carteira-col-origem'),
      cell: (a) => (
        <span className="inline-flex items-center gap-1 font-sans text-dim">
          {a.source === 'manual' ? t('carteira-fonte-manual') : sourceName(a.source)}
          <OriginBadge
            source={
              a.source === 'manual'
                ? t('carteira-registrado-por-voce')
                : t('carteira-coleta-cotacoes', { fonte: sourceName(a.source) })
            }
            date={a.date}
            note={a.source === 'manual' ? undefined : t('carteira-evento-coletado-nota')}
          />
        </span>
      ),
    },
    {
      key: 'actions',
      header: <span className="sr-only">{t('carteira-col-acoes')}</span>,
      align: 'right',
      cell: (a) => (
        <span onKeyDown={(e) => e.stopPropagation()}>
          <Button size="sm" variant="ghost" onClick={() => removal.ask([a])}>
            {t('carteira-excluir')}
          </Button>
        </span>
      ),
    },
  ];
  if (q.isPending) return <LoadingBlock />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  return (
    <div className="rounded-card border border-line bg-surface">
      <TechTable
        label={t('carteira-eventos-corporativos')}
        prefsId="Eventos corporativos"
        rows={[...q.data].reverse()}
        columns={columns}
        getRowId={(a) => a.id}
        empty={t('carteira-sem-eventos')}
        dense
        multiSelect
        {...menus}
        rowMenuLabel={(a) => t('carteira-acoes-de', { nome: actionText(a) })}
        selectionStatus={(rows) => (
          <SelectionLine count={t('carteira-eventos-selecionados', { n: rows.length })} />
        )}
      />
      {removal.dialog}
    </div>
  );
}

export function IncomeTable({
  assetId,
  from,
  to,
}: {
  assetId?: string;
  from?: string;
  to?: string;
}) {
  const q = useIncome({ asset_id: assetId, from, to });
  const accounts = useInvestmentAccounts();
  const base = useBaseCurrency();
  const removal = useDeleteRecords<IncomeEvent>(useDeleteIncomes(), {
    title: (n) => t('carteira-excluir-proventos', { n }),
    item: incomeText,
    note: (n) => t('carteira-excluir-proventos-nota', { n }),
    done: (n) => t('carteira-proventos-excluidos', { n }),
  });
  const menus = recordMenus<IncomeEvent>({
    heading: incomeText,
    selected: (n) => t('carteira-proventos-selecionados', { n }),
    removeMany: (n) => t('carteira-excluir-n-proventos', { n }),
    actionsOf: (n) => t('carteira-acoes-de-n-proventos', { n }),
    remove: removal.ask,
  });
  const columns: TechColumn<IncomeEvent>[] = [
    {
      key: 'pay',
      header: t('carteira-col-pagamento'),
      cell: (i) => formatDate(i.pay_date),
      sortValue: (i) => i.pay_date,
      width: '104px',
    },
    {
      key: 'ex',
      header: t('carteira-col-data-com'),
      cell: (i) => safeDate(i.ex_date),
      sortValue: (i) => i.ex_date,
    },
    ...(assetId
      ? []
      : [
          {
            key: 'asset',
            header: t('carteira-col-ativo'),
            cell: (i: IncomeEvent) => (
              <span className="font-medium">{i.ticker ?? i.asset_name}</span>
            ),
            sortValue: (i: IncomeEvent) => i.ticker ?? i.asset_name,
          },
        ]),
    {
      key: 'kind',
      header: t('carteira-col-tipo'),
      cell: (i) => <span className="font-sans">{INCOME_KIND_LABEL[i.kind]}</span>,
    },
    {
      key: 'qty',
      header: t('carteira-col-base'),
      numeric: true,
      cell: (i) => <Private>{i.quantity ? formatQuantity(i.quantity) : '—'}</Private>,
    },
    {
      key: 'gross',
      header: t('carteira-col-bruto'),
      numeric: true,
      cell: (i) => <Money cents={i.gross} currency={i.currency} />,
    },
    {
      key: 'tax',
      header: t('carteira-col-ir-retido'),
      numeric: true,
      cell: (i) => <Money cents={i.tax_withheld} currency={i.currency} />,
    },
    {
      key: 'net',
      header: t('carteira-col-liquido'),
      numeric: true,
      cell: (i) => <Money cents={i.net} currency={i.currency} tone="positive" />,
      sortValue: (i) => i.net,
    },
    {
      key: 'status',
      header: t('carteira-col-situacao'),
      cell: (i) =>
        i.status === 'received' ? (
          <span className="text-positive">{t('carteira-recebido')}</span>
        ) : (
          <span className="inline-flex items-center gap-1 text-warning">
            <Icon name="info" size={12} /> {t('carteira-anunciado')}
          </span>
        ),
    },
    {
      key: 'account',
      header: t('carteira-col-conta'),
      cell: (i) => (
        <span className="font-sans text-dim">
          {accounts.data?.find((a) => a.id === i.account_id)?.name ?? '—'}
        </span>
      ),
    },
    {
      key: 'actions',
      header: <span className="sr-only">{t('carteira-col-acoes')}</span>,
      align: 'right',
      cell: (i) => (
        <span onKeyDown={(e) => e.stopPropagation()}>
          <Button size="sm" variant="ghost" onClick={() => removal.ask([i])}>
            {t('carteira-excluir')}
          </Button>
        </span>
      ),
    },
  ];
  if (q.isPending) return <LoadingBlock />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  // Total do que foi recebido na moeda base (os de outra moeda ficam fora: sem câmbio aqui).
  const totalBase = q.data
    .filter((i) => i.currency === base && i.status === 'received')
    .reduce((s, i) => s + i.net, 0);
  return (
    <div className="rounded-card border border-line bg-surface">
      <TechTable
        label={t('carteira-proventos')}
        prefsId="Proventos"
        rows={q.data}
        columns={columns}
        getRowId={(i) => i.id}
        empty={t('carteira-sem-proventos-periodo')}
        maxHeight="520px"
        footer={
          q.data.length
            ? {
                pay: t('carteira-recebido-moeda', { moeda: base }),
                net: <Money cents={totalBase} currency={base} tone="positive" />,
              }
            : undefined
        }
        multiSelect
        {...menus}
        rowMenuLabel={(i) => t('carteira-acoes-de', { nome: incomeText(i) })}
        selectionStatus={(rows) => (
          <SelectionLine
            count={t('carteira-proventos-selecionados', { n: rows.length })}
            sums={[
              {
                label: t('carteira-col-liquido'),
                value: total(
                  rows,
                  (i) => i.currency,
                  (i) => i.net,
                ),
              },
            ]}
          />
        )}
      />
      {removal.dialog}
    </div>
  );
}

const incomeText = (i: IncomeEvent) =>
  t('carteira-provento-texto', {
    tipo: INCOME_KIND_LABEL[i.kind],
    ativo: i.ticker ?? i.asset_name,
    data: formatDate(i.pay_date),
  });
