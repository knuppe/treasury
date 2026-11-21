/**
 * Tabela previsto × realizado com pais expansíveis e edição do previsto na própria linha
 * (Enter grava, Esc cancela, sair do campo grava se mudou).
 */
import { useMemo, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { useNavigate } from 'react-router';

import type { Basis } from '@/api/generated/Basis';
import { BulletBar, BulletStatusLabel } from '@/charts/BulletBar';
import { isAlert } from '@/charts/bullet';
import {
  Icon,
  Money,
  MoneyInput,
  Private,
  useContextMenu,
  useDataColumns,
  type ContextMenuEntry,
  type DataColumn,
  useStickyHeader,
} from '@/design/components';
import { cx } from '@/design/cx';
import type { EntityColors } from '@/features/reports/model/colors';
import { monthDrill } from '@/features/reports/model/drill';
import { t } from '@/i18n';
import { formatMonthLong, formatNumber, formatPercent } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';

import { visibleRows, type BudgetRow } from './model';

export interface BudgetTableProps {
  rows: readonly BudgetRow[];
  month: string;
  basis: Basis;
  elapsed: number;
  colors: EntityColors;
  /** Categoria em edição (controlado pela tela, para "Definir" vindo de fora). */
  editing: string | null;
  onEditingChange: (id: string | null) => void;
  /** Grava o previsto (centavos; 0 remove a linha própria). */
  onSave: (categoryId: string, cents: number) => Promise<unknown>;
  savingId: string | null;
  kind: 'expense' | 'income';
  label: string;
}

function PlannedEditor({
  row,
  onCancel,
  onSave,
  onKeyboardClose,
}: {
  row: BudgetRow;
  onCancel: () => void;
  onSave: (cents: number) => void;
  /** Fechou pelo teclado (Enter/Esc): o foco volta para a célula. */
  onKeyboardClose: () => void;
}) {
  const own = row.line.has_own_line ? row.line.planned : null;
  const [value, setValue] = useState<number | null>(own);
  const [text, setText] = useState('');
  const done = useRef(false);
  /**
   * Grava o valor digitado (0 remove a linha própria). Campo vazio ou igual ao atual não
   * muda nada; texto inválido não grava (no Enter o campo continua aberto, marcado).
   */
  const commit = (fromEnter: boolean) => {
    if (done.current) return;
    const empty = text.trim() === '';
    const invalid = !empty && value == null;
    if (invalid && fromEnter) return;
    done.current = true;
    if (fromEnter) onKeyboardClose();
    if (invalid || empty || value == null || value === (own ?? 0)) onCancel();
    else onSave(value);
  };
  const onKey = (e: KeyboardEvent<HTMLInputElement>) => {
    setText(e.currentTarget.value);
    if (e.key === 'Enter') {
      e.preventDefault();
      commit(true);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      done.current = true;
      onKeyboardClose();
      onCancel();
    }
  };
  return (
    <MoneyInput
      aria-label={t('orcamento-previsto-de', { categoria: row.line.category_name })}
      value={value}
      onValueChange={setValue}
      onInput={(e) => setText(e.currentTarget.value)}
      onKeyDown={onKey}
      onFocus={(e) => {
        setText(e.currentTarget.value);
        e.currentTarget.select();
      }}
      onBlur={() => commit(false)}
      autoFocus
      wrapperClassName="h-8 w-36 ml-auto"
      placeholder={row.plannedFromChildren ? t('orcamento-soma-filhos') : formatNumber(0, 2)}
    />
  );
}

function PlannedCell({
  row,
  editing,
  saving,
  onStart,
  onCancel,
  onSave,
  refocusRef,
}: {
  row: BudgetRow;
  editing: boolean;
  saving: boolean;
  onStart: () => void;
  onCancel: () => void;
  onSave: (cents: number) => void;
  /** Célula que deve receber o foco ao voltar do modo de edição. */
  refocusRef: { current: string | null };
}) {
  const id = row.line.category_id;
  if (editing) {
    return (
      <PlannedEditor
        row={row}
        onCancel={onCancel}
        onSave={onSave}
        onKeyboardClose={() => {
          refocusRef.current = id;
        }}
      />
    );
  }
  return (
    <button
      ref={(el) => {
        if (el && refocusRef.current === id) {
          refocusRef.current = null;
          el.focus();
        }
      }}
      type="button"
      onClick={() => {
        if (!saving) onStart();
      }}
      aria-label={t('orcamento-editar-previsto-de', { categoria: row.line.category_name })}
      aria-disabled={saving || undefined}
      aria-busy={saving || undefined}
      className="group ml-auto inline-flex h-8 min-w-24 items-center justify-end gap-1.5 rounded-control border border-transparent px-2 hover:border-line-strong focus-visible:outline-2 focus-visible:outline-accent aria-disabled:opacity-50"
    >
      {saving ? (
        <span className="label-caps">{t('orcamento-gravando')}</span>
      ) : row.line.planned > 0 ? (
        <span
          className="inline-flex items-baseline gap-1"
          title={row.plannedFromChildren ? t('orcamento-soma-subcategorias-dica') : undefined}
        >
          {row.plannedFromChildren ? (
            <span aria-label={t('orcamento-soma-subcategorias')} className="text-[11px] text-dim">
              {'Σ'}
            </span>
          ) : null}
          <Money cents={row.line.planned} />
        </span>
      ) : (
        <span className="label-caps text-dim group-hover:text-accent">
          {t('orcamento-definir')}
        </span>
      )}
    </button>
  );
}

/** Classe da célula de cada coluna (alinhamento e respiro). */
const CELL_CLASS: Record<string, string> = {
  category: 'h-11 px-3',
  bullet: 'min-w-40 px-3',
  planned: 'px-2 text-right',
  actual: 'num px-3 text-right',
  committed: 'num px-3 text-right text-dim',
  remaining: 'num px-3 text-right',
  ratio: 'num px-3 text-right text-dim',
};

function budgetColumns(kind: 'expense' | 'income'): DataColumn<BudgetRow>[] {
  return [
    {
      key: 'category',
      header: t('controles-categoria'),
      hideable: false,
      sortValue: (r) => r.line.category_name,
    },
    {
      key: 'bullet',
      header:
        kind === 'expense' ? t('orcamento-previsto-realizado') : t('orcamento-recebido-previsto'),
      width: '26%',
      sortValue: (r) => r.line.ratio,
    },
    {
      key: 'planned',
      header: t('orcamento-previsto'),
      numeric: true,
      sortValue: (r) => r.line.planned,
    },
    {
      key: 'actual',
      header: kind === 'expense' ? t('orcamento-realizado') : t('orcamento-recebido'),
      numeric: true,
      sortValue: (r) => r.line.actual,
    },
    {
      key: 'committed',
      header: t('orcamento-comprometido'),
      numeric: true,
      sortValue: (r) => r.line.committed,
    },
    {
      key: 'remaining',
      header: t('orcamento-restante'),
      numeric: true,
      sortValue: (r) =>
        r.line.planned > 0 ? (kind === 'expense' ? r.line.remaining : -r.line.remaining) : null,
    },
    {
      key: 'ratio',
      header: '%',
      title: t('orcamento-percentual'),
      numeric: true,
      sortValue: (r) => r.line.ratio,
    },
  ];
}

export function BudgetTable({
  rows,
  month,
  basis,
  elapsed,
  colors,
  editing,
  onEditingChange,
  onSave,
  savingId,
  kind,
  label,
}: BudgetTableProps) {
  const navigate = useNavigate();
  const [expandedState, setExpanded] = useState<Set<string>>(() => new Set());
  const refocusRef = useRef<string | null>(null);
  // Ao editar uma subcategoria vinda de fora ("Definir" no aviso), o pai abre junto.
  const editingParent = editing
    ? rows.find((r) => r.children.some((c) => c.line.category_id === editing))?.line.category_id
    : undefined;
  const expanded = editingParent ? new Set([...expandedState, editingParent]) : expandedState;
  const columns = useMemo(() => budgetColumns(kind), [kind]);
  // As preferências da tabela ficam no nome em pt-BR, para valerem em qualquer idioma.
  const prefsId =
    kind === 'expense'
      ? 'Orçamento de despesas por categoria'
      : 'Orçamento de receitas por categoria';
  const cols = useDataColumns({ label, prefsId, columns, rows });
  const boxRef = useRef<HTMLDivElement>(null);
  useStickyHeader(boxRef);
  const { sortRows } = cols;
  // Ordena entre irmãos: cada subcategoria continua logo abaixo do pai.
  const sorted = useMemo(() => {
    const walk = (list: readonly BudgetRow[]): BudgetRow[] =>
      sortRows(list).map((r) => (r.children.length ? { ...r, children: walk(r.children) } : r));
    return walk(rows);
  }, [rows, sortRows]);
  const flat = visibleRows(sorted, expanded);
  const toggle = (id: string) =>
    setExpanded((s) => {
      const n = new Set(s);
      if (n.has(id)) n.delete(id);
      else n.add(id);
      return n;
    });
  const monthName = formatMonthLong(month);
  const menu = useContextMenu();
  const drill = (id: string) =>
    void navigate(
      txnLink(monthDrill(month, basis, [id], kind === 'income' ? 'income' : 'expense')),
    );
  // Menu de contexto da categoria: definir o previsto, ver os lançamentos, abrir o grupo.
  const rowMenu = (row: BudgetRow) => {
    const l = row.line;
    const id = l.category_id;
    const entries: ContextMenuEntry[] = [
      { heading: l.category_path },
      {
        label: l.has_own_line
          ? t('orcamento-editar-o-previsto')
          : t('orcamento-definir-o-previsto'),
        icon: 'sliders',
        onSelect: () => onEditingChange(id),
      },
      {
        label: t('orcamento-ver-lancamentos-mes', { mes: monthName }),
        icon: 'search',
        onSelect: () => drill(id),
      },
      row.children.length > 0 && {
        label: expanded.has(id)
          ? t('orcamento-recolher-subcategorias')
          : t('orcamento-expandir-subcategorias'),
        icon: expanded.has(id) ? 'chevronDown' : 'chevronRight',
        onSelect: () => toggle(id),
      },
    ];
    return { entries, label: t('orcamento-acoes-de', { categoria: l.category_name }) };
  };
  const cell = (key: string, row: BudgetRow): ReactNode => {
    const l = row.line;
    const id = l.category_id;
    switch (key) {
      case 'category': {
        const root = row.depth === 0;
        const color = colors.get(root ? id : (l.parent_id ?? id)) ?? 'var(--line-strong)';
        const hasChildren = row.children.length > 0;
        const isOpen = expanded.has(id);
        return (
          <div className="flex items-center gap-2" style={{ paddingLeft: row.depth * 20 }}>
            {hasChildren ? (
              <button
                type="button"
                aria-expanded={isOpen}
                aria-label={
                  isOpen
                    ? t('orcamento-recolher', { categoria: l.category_name })
                    : t('orcamento-expandir', { categoria: l.category_name })
                }
                onClick={() => toggle(id)}
                className="inline-flex size-5 shrink-0 items-center justify-center rounded-control text-dim hover:text-fg focus-visible:outline-2 focus-visible:outline-accent"
              >
                <Icon name={isOpen ? 'chevronDown' : 'chevronRight'} size={12} />
              </button>
            ) : (
              <span aria-hidden className="inline-block size-5 shrink-0" />
            )}
            <span
              aria-hidden
              className="inline-block size-2 shrink-0 rounded-[2px]"
              style={{ background: color, opacity: root ? 1 : 0.6 }}
            />
            <span className={cx('min-w-0 truncate', root ? 'text-fg' : 'text-dim')}>
              {l.category_name}
              {l.archived ? (
                <span className="label-caps ml-2">{t('orcamento-arquivada')}</span>
              ) : null}
            </span>
            {kind === 'expense' && isAlert(row.status) ? (
              <span className="ml-1 hidden @min-[1040px]:inline">
                <BulletStatusLabel status={row.status} />
              </span>
            ) : null}
          </div>
        );
      }
      case 'bullet':
        return (
          <BulletBar
            label={l.category_path}
            planned={l.planned}
            actual={l.actual}
            committed={l.committed}
            elapsed={kind === 'expense' ? elapsed : 1}
            hint={t('orcamento-clique-realizado')}
          />
        );
      case 'planned':
        return (
          <PlannedCell
            row={row}
            editing={editing === id}
            saving={savingId === id}
            refocusRef={refocusRef}
            onStart={() => onEditingChange(id)}
            onCancel={() => onEditingChange(null)}
            onSave={(cents) => {
              onEditingChange(null);
              void onSave(id, cents);
            }}
          />
        );
      case 'actual':
        return (
          <button
            type="button"
            onClick={() => drill(id)}
            aria-label={t('orcamento-ver-lancamentos-categoria', {
              categoria: l.category_name,
              mes: monthName,
            })}
            className="rounded-control hover:text-accent hover:underline focus-visible:outline-2 focus-visible:outline-accent"
          >
            <Money cents={l.actual} />
          </button>
        );
      case 'committed':
        return l.committed ? <Money cents={l.committed} /> : '—';
      case 'remaining':
        return l.planned > 0 ? (
          <Money cents={kind === 'expense' ? l.remaining : -l.remaining} tone="auto" />
        ) : (
          <span className="text-dim">—</span>
        );
      case 'ratio':
        return l.ratio != null ? <Private>{formatPercent(l.ratio, { decimals: 0 })}</Private> : '—';
      default:
        return null;
    }
  };
  return (
    <div ref={boxRef} className="tech-table overflow-x-auto">
      <table
        aria-label={label}
        className={cx('border-separate border-spacing-0 text-[13px]', !cols.fixed && 'w-full')}
        style={cols.tableStyle}
      >
        {cols.colgroup}
        <thead>
          <tr>
            {cols.shown.map((c) =>
              cols.headerCell(c, {
                className: 'label-caps h-8 border-b border-line px-3 font-medium whitespace-nowrap',
              }),
            )}
            {cols.pickerCell()}
          </tr>
        </thead>
        <tbody>
          {flat.map((row) => {
            const id = row.line.category_id;
            return (
              <tr
                key={id}
                data-category={id}
                className="tech-row"
                onContextMenu={(e) => menu.onContextMenu(e, rowMenu(row))}
                onKeyDown={(e) => menu.onKeyDown(e, rowMenu(row))}
              >
                {cols.shown.map((c) => (
                  <td
                    key={c.key}
                    className={cx(
                      'border-b border-line/70',
                      CELL_CLASS[c.key],
                      cols.fixed && 'overflow-hidden',
                    )}
                  >
                    {cell(c.key, row)}
                  </td>
                ))}
                {cols.fillerCell('border-b border-line/70')}
              </tr>
            );
          })}
        </tbody>
      </table>
      {menu.menu}
      {cols.menus}
    </div>
  );
}
