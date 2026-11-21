import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react';

import type { TxnOperationKind } from '@/api/generated/TxnOperationKind';
import type { TxnPage } from '@/api/generated/TxnPage';
import type { TxnSort } from '@/api/generated/TxnSort';
import type { TxnView } from '@/api/generated/TxnView';
import { useUpdateTxn } from '@/api/hooks/transactions';
import {
  Icon,
  Money,
  TechTable,
  Tooltip,
  toast,
  type ContextMenuEntry,
  type SortState,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { CategoryPicker } from './ui/CategoryPicker';
import { Checkbox, KindBadge, StatusBadge } from './ui/controls';
import { errorText } from './ui/errors';

/** Marca curta do vínculo com uma operação, ao lado da descrição. */
function operationShort(kind: TxnOperationKind): string {
  if (kind === 'trade') return t('lancamentos-op-carteira');
  if (kind === 'capital_share') return t('lancamentos-op-cotas');
  return t('lancamentos-op-meta');
}

/** Colunas que o núcleo sabe ordenar (`<coluna>_asc` / `<coluna>_desc`). */
const SORT_KEYS = ['date', 'amount', 'description', 'account', 'category', 'member', 'status'];

function sortState(sort: TxnSort): SortState {
  const i = sort.lastIndexOf('_');
  return { key: sort.slice(0, i), direction: sort.slice(i + 1) as SortState['direction'] };
}

export interface TxnTableProps {
  page: TxnPage;
  sort: TxnSort;
  onSortChange: (sort: TxnSort) => void;
  selection: ReadonlySet<string>;
  onSelectionChange: (next: Set<string>) => void;
  onOpen: (txn: TxnView) => void;
  /** Foco/realce da linha atual (teclado). */
  focusedId: string | null;
  onFocusedChange: (id: string | null) => void;
  /** Menu de contexto da linha; `categorize` abre o seletor de categoria dela. */
  rowMenu?: (txn: TxnView, ctx: { categorize?: () => void }) => readonly ContextMenuEntry[];
  /** Menu de contexto com duas ou mais linhas marcadas (as ações da barra de lote). */
  selectionMenu?: () => readonly ContextMenuEntry[];
}

/** Receitas, despesas e transferências sem divisão têm a categoria editável na linha. */
const inlineCategory = (txn: TxnView) =>
  (txn.kind === 'income' || txn.kind === 'expense' || txn.kind === 'transfer') &&
  txn.splits.length === 0;

/**
 * Abaixo desta largura (painel do agente aberto, janela estreita) a tabela esconde
 * Membro e Status para a descrição e o valor continuarem legíveis; previstos já
 * aparecem em itálico.
 */
const NARROW_BELOW = 960;
const NARROW_HIDDEN = new Set(['member', 'status']);

function useNarrow(below: number) {
  const boxRef = useRef<HTMLDivElement>(null);
  const [narrow, setNarrow] = useState(false);
  useEffect(() => {
    const el = boxRef.current;
    if (!el || typeof ResizeObserver === 'undefined') return;
    const check = () => {
      const w = el.clientWidth;
      setNarrow(w > 0 && w < below);
    };
    check();
    const ro = new ResizeObserver(check);
    ro.observe(el);
    return () => ro.disconnect();
  }, [below]);
  return { boxRef, narrow };
}

function safeDate(iso: string) {
  try {
    return formatDate(iso);
  } catch {
    return iso;
  }
}

/** Categoria editável na própria linha (receitas e despesas sem divisão). */
/**
 * Na transferência, se ela está ligada: `→ conta` (ou `← conta`) com a outra perna; solta,
 * um aviso discreto de que falta a outra perna (as duas pernas ligadas saem de receitas e
 * despesas).
 */
function TransferMark({ txn }: { txn: TxnView }) {
  if (txn.transfer_id) {
    const other = txn.counterpart_account_name ?? t('lancamentos-outra-conta');
    return (
      <span
        className="shrink-0 truncate text-[11.5px] text-accent"
        aria-label={t('lancamentos-ligada-a', { conta: other })}
        title={t('lancamentos-ligada-perna', { conta: other })}
      >
        {`${txn.amount < 0 ? '→' : '←'} ${other}`}
      </span>
    );
  }
  return (
    <Tooltip content={t('lancamentos-solta-dica')}>
      <span
        tabIndex={-1}
        className="label-caps shrink-0 text-dim"
        aria-label={t('lancamentos-transferencia-solta')}
      >
        {t('lancamentos-solta')}
      </span>
    </Tooltip>
  );
}

function CategoryCell({
  txn,
  open,
  onOpenChange,
  onOpenTxn,
}: {
  txn: TxnView;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onOpenTxn: () => void;
}) {
  const update = useUpdateTxn();
  if (txn.splits.length > 0) {
    return (
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          onOpenTxn();
        }}
        className="inline-flex items-center gap-1.5 rounded-control text-[12.5px] text-accent hover:underline"
      >
        <Icon name="sort" size={11} />
        {t('lancamentos-dividido-em', { n: txn.splits.length })}
      </button>
    );
  }
  // Transferência também leva categoria (PIX que paga um serviço ou produto).
  // Aporte, resgate, pagamento de fatura e ajuste não usam categoria (ficam fora do fluxo de
  // caixa): a coluna diz o que são, com a conta do outro lado quando houver.
  if (txn.kind !== 'income' && txn.kind !== 'expense' && txn.kind !== 'transfer') {
    const what =
      txn.kind === 'investment'
        ? txn.amount < 0
          ? t('lancamentos-sem-categoria-aporte')
          : t('lancamentos-sem-categoria-resgate')
        : null;
    const other = txn.counterpart_account_name
      ? `${txn.amount < 0 ? '→' : '←'} ${txn.counterpart_account_name}`
      : null;
    const text = [what, other].filter(Boolean).join(' ');
    return (
      <span
        className="block truncate text-[12.5px] text-dim"
        title={txn.kind === 'investment' ? t('lancamentos-sem-categoria-aporte-ajuda') : undefined}
      >
        {text || '—'}
      </span>
    );
  }
  return (
    // A tabela trata ↑/↓/Enter das linhas: o seletor guarda as próprias teclas.
    <span
      className="block max-w-full min-w-0 overflow-hidden"
      onClick={(e) => e.stopPropagation()}
      onDoubleClick={(e) => e.stopPropagation()}
      onKeyDown={(e) => e.stopPropagation()}
    >
      <CategoryPicker
        compact
        kind={txn.kind === 'transfer' ? (txn.amount < 0 ? 'expense' : 'income') : txn.kind}
        value={txn.category_id}
        open={open}
        onOpenChange={onOpenChange}
        label={t('lancamentos-categoria-de', { descricao: txn.description })}
        onChange={(category_id) => {
          if (category_id === txn.category_id) return;
          update.mutate(
            { id: txn.id, category_id },
            {
              onError: (e) =>
                toast({
                  tone: 'error',
                  title: t('lancamentos-erro-categoria'),
                  description: errorText(e),
                }),
            },
          );
        }}
      />
    </span>
  );
}

/**
 * Tabela densa de lançamentos com seleção múltipla (as marcas da barra de lote). Teclado:
 * ↑/↓ movem, Espaço marca/desmarca, Shift+↑/↓ marcam a faixa, ⌘A/Ctrl+A marca a página,
 * Esc desmarca tudo, `c` edita a categoria (com várias marcadas, a delas), Enter abre.
 */
export function TxnTable({
  page,
  sort,
  onSortChange,
  selection,
  onSelectionChange,
  onOpen,
  focusedId,
  onFocusedChange,
  rowMenu,
  selectionMenu,
}: TxnTableProps) {
  const [editingCategory, setEditingCategory] = useState<string | null>(null);
  const rows = page.items;
  const allSelected = rows.length > 0 && rows.every((r) => selection.has(r.id));
  const someSelected = rows.some((r) => selection.has(r.id));

  const toggle = (id: string) => {
    const next = new Set(selection);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    onSelectionChange(next);
  };

  const { boxRef, narrow } = useNarrow(NARROW_BELOW);

  const columns = useMemo<TechColumn<TxnView>[]>(
    () => [
      {
        key: 'select',
        width: '36px',
        header: (
          <Checkbox
            hideLabel
            label={t('lancamentos-selecionar-pagina')}
            checked={allSelected}
            indeterminate={someSelected}
            onChange={(on) => {
              const next = new Set(selection);
              for (const r of rows) {
                if (on) next.add(r.id);
                else next.delete(r.id);
              }
              onSelectionChange(next);
            }}
          />
        ),
        cell: (row) => (
          <Checkbox
            hideLabel
            label={t('lancamentos-selecionar', { descricao: row.description })}
            checked={selection.has(row.id)}
            onChange={() => toggle(row.id)}
          />
        ),
      },
      {
        key: 'date',
        header: t('lancamentos-campo-data'),
        width: '96px',
        numeric: true,
        align: 'left',
        sortValue: (row) => row.date,
        cell: (row) => (
          <span className={row.status === 'planned' ? 'text-dim' : undefined}>
            {safeDate(row.date)}
            {row.statement_due_date && row.kind !== 'card_payment' ? (
              <span
                className="block text-[10.5px] text-dim"
                title={t('lancamentos-vencimento-fatura')}
              >
                {t('lancamentos-fat', { data: safeDate(row.statement_due_date).slice(0, 5) })}
              </span>
            ) : null}
          </span>
        ),
      },
      {
        key: 'description',
        sortValue: (row) => row.description,
        header: t('lancamentos-campo-descricao'),
        cell: (row) => (
          <div className="flex min-w-0 flex-col gap-0.5 py-1">
            <span className="flex min-w-0 items-center gap-1.5">
              <span className={row.status === 'planned' ? 'truncate text-dim italic' : 'truncate'}>
                {row.description || t('lancamentos-sem-descricao-parenteses')}
              </span>
              <KindBadge kind={row.kind} amount={row.amount} />
              {row.kind === 'transfer' ? <TransferMark txn={row} /> : null}
              {row.installment_number && row.installment_total ? (
                <span className="num text-[11px] text-dim">
                  {`${row.installment_number}/${row.installment_total}`}
                </span>
              ) : null}
              {row.loan_id ? (
                <Tooltip
                  content={t('lancamentos-parcela-de', {
                    numero: row.loan_installment_number ?? '?',
                    contrato: row.loan_name ?? t('lancamentos-contrato'),
                  })}
                >
                  <span
                    tabIndex={-1}
                    className="num shrink-0 text-[11px] text-accent"
                    aria-label={t('lancamentos-parcela-de', {
                      numero: row.loan_installment_number ?? '?',
                      contrato: row.loan_name ?? t('lancamentos-contrato'),
                    })}
                  >
                    {t('lancamentos-parc', { numero: row.loan_installment_number ?? '' })}
                  </span>
                </Tooltip>
              ) : null}
              {row.operation ? (
                <Tooltip content={t('lancamentos-vinculado', { operacao: row.operation.label })}>
                  <span
                    tabIndex={-1}
                    className="shrink-0 text-[11px] text-accent"
                    aria-label={t('lancamentos-vinculado', { operacao: row.operation.label })}
                  >
                    {operationShort(row.operation.kind)}
                  </span>
                </Tooltip>
              ) : null}
              {row.recurrence_id ? (
                <Tooltip content={t('lancamentos-recorrencia-dica')}>
                  <span tabIndex={-1} aria-label={t('lancamentos-recorrente')} className="text-dim">
                    <Icon name="refresh" size={11} />
                  </span>
                </Tooltip>
              ) : null}
              {row.receipt_id ? (
                <Tooltip content={t('alimentacao-tem-itens')}>
                  <span tabIndex={-1} aria-label={t('alimentacao-tem-itens')} className="text-dim">
                    <Icon name="sort" size={11} />
                  </span>
                </Tooltip>
              ) : null}
              {row.attachment_count > 0 ? (
                <span
                  className="num text-[11px] text-dim"
                  aria-label={t('lancamentos-n-anexos', { n: row.attachment_count })}
                >
                  {`⎘${row.attachment_count}`}
                </span>
              ) : null}
            </span>
            {row.payee || row.tags.length > 0 ? (
              <span className="flex min-w-0 items-center gap-1.5 text-[11.5px] text-dim">
                {row.payee ? <span className="truncate">{row.payee}</span> : null}
                {row.tags.map((tag) => (
                  <span
                    key={tag.id}
                    className="rounded-control border border-line px-1 text-[10.5px]"
                  >
                    {`#${tag.name}`}
                  </span>
                ))}
              </span>
            ) : null}
          </div>
        ),
      },
      {
        key: 'account',
        sortValue: (row) => row.account_name,
        header: t('lancamentos-campo-conta'),
        width: '15%',
        cell: (row) => <span className="block truncate text-dim">{row.account_name}</span>,
      },
      {
        key: 'category',
        sortValue: (row) => row.category_path,
        header: t('lancamentos-campo-categoria'),
        width: '19%',
        cell: (row) => (
          <CategoryCell
            txn={row}
            open={editingCategory === row.id}
            onOpenChange={(o) => setEditingCategory(o ? row.id : null)}
            onOpenTxn={() => onOpen(row)}
          />
        ),
      },
      {
        key: 'project',
        header: t('lancamentos-campo-projeto'),
        width: '120px',
        sortable: false,
        defaultHidden: true,
        cell: (row) => <span className="block truncate text-dim">{row.project_name ?? '—'}</span>,
      },
      {
        key: 'member',
        sortValue: (row) => row.member_name,
        header: t('lancamentos-campo-membro'),
        width: '90px',
        cell: (row) => <span className="block truncate text-dim">{row.member_name ?? '—'}</span>,
      },
      {
        key: 'status',
        sortValue: (row) => row.status,
        header: t('lancamentos-campo-status'),
        width: '104px',
        cell: (row) => <StatusBadge status={row.status} />,
      },
      {
        key: 'amount',
        header: t('lancamentos-campo-valor'),
        numeric: true,
        width: '136px',
        sortValue: (row) => row.amount,
        cell: (row) => (
          <Money cents={row.amount} currency={row.currency} tone="auto" estimated={false} />
        ),
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [rows, selection, allSelected, someSelected, editingCategory],
  );

  // Espaço, Shift+setas, ⌘A e Esc ficam com a tabela (marcas). `c` edita a categoria da
  // linha; numa linha de um lote (duas ou mais marcadas), fica com o menu do lote.
  const onKeyDownCapture = (e: KeyboardEvent<HTMLDivElement>) => {
    const row = (e.target as HTMLElement).closest?.('tr[data-row-id]');
    const id = row?.getAttribute('data-row-id');
    if (!id || e.target !== row) return;
    if (e.key === 'c' && !e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey) {
      if (selectionMenu && selection.size >= 2 && selection.has(id)) return;
      const txn = rows.find((r) => r.id === id);
      if (txn && inlineCategory(txn)) {
        e.preventDefault();
        setEditingCategory(id);
      }
    }
  };

  return (
    <div ref={boxRef} onKeyDownCapture={onKeyDownCapture}>
      <TechTable
        label={t('lancamentos-aba-lancamentos')}
        prefsId="Lançamentos"
        dense
        minWidth={narrow ? '640px' : undefined}
        columns={narrow ? columns.filter((c) => !NARROW_HIDDEN.has(c.key)) : columns}
        rows={rows}
        getRowId={(row) => row.id}
        selectedId={focusedId}
        onSelectedChange={(id) => onFocusedChange(id)}
        selection={selection}
        onSelectionChange={(ids) => onSelectionChange(ids)}
        selectionMode="marks"
        selectionStatus={false}
        selectionMenu={selectionMenu}
        selectionMenuLabel={() => t('lancamentos-acoes-marcados', { n: selection.size })}
        onActivate={onOpen}
        serverSort
        sort={sortState(sort)}
        onSortChange={(s) => {
          if (!s || !SORT_KEYS.includes(s.key)) return onSortChange('date_desc');
          onSortChange(`${s.key}_${s.direction}` as TxnSort);
        }}
        empty={t('lancamentos-vazio-filtro')}
        className="[&_table]:table-fixed"
        rowMenuLabel={(row) =>
          t('lancamentos-acoes-de', {
            descricao: row.description || t('lancamentos-sem-descricao-acoes'),
          })
        }
        rowMenu={
          rowMenu
            ? (row) =>
                rowMenu(row, {
                  categorize: inlineCategory(row) ? () => setEditingCategory(row.id) : undefined,
                })
            : undefined
        }
      />
    </div>
  );
}
