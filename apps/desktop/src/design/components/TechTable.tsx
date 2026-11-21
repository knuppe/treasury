import {
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
  type MouseEvent,
  type PointerEvent,
  type ReactNode,
} from 'react';

import { hasModKey, matchesShortcut } from '@/lib/platform';
import { t } from '@/i18n';

import { cx } from '../cx';
import { useContextMenu, type ContextMenuAction, type ContextMenuEntry } from './ContextMenu';
import {
  alignClass,
  columnName,
  useDataColumns,
  type DataColumn,
  type SortState,
} from './dataColumns';
import { toast } from './Toast';
import { useStickyHeader } from './useStickyHeader';

export type { SortDirection, SortState } from './dataColumns';

/** O que os menus da tabela recebem além das linhas. */
export interface TechTableMenuContext {
  /**
   * Copia as linhas do menu para a área de transferência: as colunas visíveis com nome,
   * separadas por tabulação (cola direto numa planilha), com o texto que a tela mostra.
   */
  copy: () => void;
}

export interface TechColumn<Row> extends DataColumn<Row> {
  /** Conteúdo da célula; padrão: `String(row[key])`. */
  cell?: (row: Row, index: number) => ReactNode;
  className?: string;
}

export interface TechTableProps<Row> {
  columns: readonly TechColumn<Row>[];
  rows: readonly Row[];
  getRowId: (row: Row) => string;
  /** Nome acessível da tabela. */
  label: string;
  /** Linha selecionada (controlado). Omita para a tabela cuidar sozinha. */
  selectedId?: string | null;
  defaultSelectedId?: string | null;
  onSelectedChange?: (id: string | null, row: Row | null) => void;
  /** Enter ou duplo clique na linha. */
  onActivate?: (row: Row) => void;
  /** Ordenação (controlada). Omita para a tabela cuidar sozinha. */
  sort?: SortState | null;
  defaultSort?: SortState | null;
  onSortChange?: (sort: SortState | null) => void;
  /** Conteúdo quando não há linhas. */
  empty?: ReactNode;
  /** Linha de totais: conteúdo por chave de coluna. */
  footer?: Partial<Record<string, ReactNode>>;
  /** Altura máxima com rolagem e cabeçalho fixo (`420px`). */
  maxHeight?: string;
  /** Largura mínima da tabela; abaixo dela a tabela rola na horizontal (`880px`). */
  minWidth?: string;
  dense?: boolean;
  className?: string;
  /**
   * Menu de contexto da linha (botão direito, Shift+F10 ou tecla Menu). A linha fica
   * selecionada antes de abrir. Os itens repetem ações que já existem na tela. Os atalhos
   * dos itens (`shortcut`) funcionam com o foco na linha, sem abrir o menu.
   */
  rowMenu?: (row: Row, ctx: TechTableMenuContext) => readonly ContextMenuEntry[];
  /** Nome acessível do menu da linha (padrão: `Ações da linha`). */
  rowMenuLabel?: (row: Row) => string;
  /**
   * Seleção de várias linhas (controlada). Com ela (ou `multiSelect`): Shift+↑/↓ (e
   * Shift+Home/End/PageUp/PageDown) estendem ou encolhem a faixa a partir da âncora,
   * ⌘A/Ctrl+A seleciona todas, Esc volta à linha do cursor, Espaço alterna a linha;
   * Shift+clique pega a faixa e ⌘/Ctrl+clique alterna. `selectedId` segue sendo o cursor.
   */
  selection?: ReadonlySet<string>;
  onSelectionChange?: (ids: Set<string>, rows: Row[]) => void;
  /** Seleção de várias linhas cuidada pela própria tabela. */
  multiSelect?: boolean;
  /**
   * `follow` (padrão): a seleção acompanha o cursor (clique e setas selecionam só a linha).
   * `marks`: a seleção são marcas que ficam enquanto o cursor anda (caixas de marcar, como
   * em Lançamentos); Esc limpa as marcas.
   */
  selectionMode?: 'follow' | 'marks';
  /**
   * Menu de contexto de várias linhas: abre sobre uma linha de uma seleção de duas ou mais
   * (fora dela, a seleção volta à linha e abre o `rowMenu`). Os atalhos dos itens também
   * funcionam com o foco na tabela.
   */
  selectionMenu?: (rows: Row[], ctx: TechTableMenuContext) => readonly ContextMenuEntry[];
  /** Nome acessível do menu da seleção (padrão: `Ações de N linhas`). */
  selectionMenuLabel?: (rows: Row[]) => string;
  /**
   * Linha de status com duas ou mais linhas selecionadas (contagem, soma). Padrão:
   * `N linhas selecionadas`; `false` esconde (quando a tela já mostra a seleção).
   */
  selectionStatus?: false | ((rows: Row[]) => ReactNode);
  /**
   * Onde guardar as preferências do usuário (ordenação, larguras e colunas visíveis);
   * padrão: o nome da tabela. Tabelas com o mesmo nome dividem as preferências.
   */
  prefsId?: string;
  /** `false`: sem ajuste de colunas (tabelas de duas ou três colunas fixas). */
  customizable?: boolean;
  /**
   * A ordenação é feita por quem fornece as linhas (consulta paginada no núcleo): a
   * tabela só mostra e avisa a escolha (`onSortChange`), sem reordenar a página.
   */
  serverSort?: boolean;
}

const PAGE = 10;

/**
 * Texto cortado com reticências (`truncate`) ganha o texto inteiro na dica
 * nativa ao passar o mouse, se a célula não tiver uma dica própria.
 */
function revealTruncated(event: PointerEvent<HTMLElement>) {
  for (
    let el = event.target instanceof HTMLElement ? event.target : null;
    el && el.tagName !== 'TD' && el.tagName !== 'TH';
    el = el.parentElement
  ) {
    if (el.title) return;
    if (el.scrollWidth > el.clientWidth && getComputedStyle(el).textOverflow === 'ellipsis') {
      const text = el.textContent?.trim();
      if (text) el.title = text;
      return;
    }
  }
}

function defaultCell(row: unknown, key: string): ReactNode {
  const value = (row as Record<string, unknown>)[key];
  return typeof value === 'string' || typeof value === 'number' ? value : null;
}

const EMPTY: ReadonlySet<string> = new Set();

/** Teclas que movem o cursor (com Shift, estendem a seleção). */
const NAV_KEYS = new Set(['ArrowDown', 'ArrowUp', 'Home', 'End', 'PageDown', 'PageUp']);

/** Controles dentro da célula: o clique neles não usa Shift/⌘ para selecionar. */
const CONTROL = 'input, button, a, select, textarea, label, [role="checkbox"], [role="button"]';

const isAction = (e: ContextMenuEntry): e is ContextMenuAction =>
  typeof e === 'object' && e !== null && 'onSelect' in e;

/** Coluna com nome (as de ações e de marcar ficam fora da cópia). */
function hasName<Row>(c: DataColumn<Row>): boolean {
  if (c.title) return true;
  return (
    (typeof c.header === 'string' || typeof c.header === 'number') && String(c.header).trim() !== ''
  );
}

/**
 * Texto que a célula mostra: sem o que só os leitores de tela ouvem, e sem os enfeites
 * `aria-hidden` (setas, ≈) quando há texto ao lado; a máscara do modo privacidade fica.
 */
function visibleText(cell: HTMLTableCellElement): string {
  const copy = cell.cloneNode(true) as HTMLElement;
  for (const el of copy.querySelectorAll('.sr-only')) el.remove();
  for (const el of copy.querySelectorAll('[aria-hidden="true"]')) {
    const parent = el.parentElement;
    if (parent && (parent.textContent ?? '').trim() !== (el.textContent ?? '').trim()) el.remove();
  }
  return (copy.textContent ?? '').replace(/\s+/g, ' ').trim();
}

function writeClipboard(text: string): Promise<boolean> {
  const clip = typeof navigator !== 'undefined' ? navigator.clipboard : undefined;
  if (!clip?.writeText) return Promise.resolve(false);
  return clip.writeText(text).then(
    () => true,
    () => false,
  );
}

/**
 * Seleção de várias linhas só para copiar (tabelas de leitura, como extratos): os menus da
 * linha e da seleção têm "Copiar" com ⌘C/Ctrl+C. Uso: `<TechTable {...copyOnlyMenus()} />`.
 */
export function copyOnlyMenus<Row>(): Pick<
  TechTableProps<Row>,
  'multiSelect' | 'rowMenu' | 'selectionMenu'
> {
  return {
    multiSelect: true,
    rowMenu: (_row, ctx) => [
      {
        label: t('componentes-copiar-linha'),
        icon: 'copy',
        shortcut: ['mod', 'C'],
        onSelect: ctx.copy,
      },
    ],
    selectionMenu: (rows, ctx) => [
      { heading: t('componentes-linhas-selecionadas', { n: rows.length }) },
      {
        label: t('componentes-copiar-linhas', { n: rows.length }),
        icon: 'copy',
        shortcut: ['mod', 'C'],
        onSelect: ctx.copy,
      },
    ],
  };
}

/**
 * Tabela técnica: linhas finas, colunas numéricas mono à direita, linha
 * selecionada com borda `--accent`.
 *
 * Teclado: ↑/↓ movem a seleção, Home/End vão ao início/fim, PageUp/PageDown
 * pulam 10 linhas, Enter ativa a linha. Só a linha selecionada (ou a primeira)
 * entra na ordem do Tab. Cabeçalhos ordenáveis são botões (`aria-sort`).
 *
 * Com seleção de várias linhas (`multiSelect` ou `selection`), a linha com o foco é o
 * cursor (borda `--accent`) e as selecionadas ganham fundo; Shift estende a faixa a
 * partir da âncora (a última linha escolhida sem Shift). Os atalhos mostrados nos menus
 * valem com o foco na linha (nunca dentro de um campo ou botão da célula).
 */
export function TechTable<Row>({
  columns,
  rows,
  getRowId,
  label,
  selectedId: selectedIdProp,
  defaultSelectedId = null,
  onSelectedChange,
  onActivate,
  sort: sortProp,
  defaultSort = null,
  onSortChange,
  empty,
  footer,
  maxHeight,
  minWidth,
  dense = false,
  className,
  rowMenu,
  rowMenuLabel,
  selection: selectionProp,
  onSelectionChange,
  multiSelect = false,
  selectionMode = 'follow',
  selectionMenu,
  selectionMenuLabel,
  selectionStatus,
  prefsId,
  customizable = true,
  serverSort = false,
}: TechTableProps<Row>) {
  const menu = useContextMenu();
  const cols = useDataColumns({
    label,
    prefsId,
    columns,
    rows,
    customizable,
    sort: sortProp,
    defaultSort,
    onSortChange,
    minWidth,
  });
  const { shown, fixed } = cols;
  const [selectedInternal, setSelectedInternal] = useState<string | null>(defaultSelectedId);
  const selectedId = selectedIdProp !== undefined ? selectedIdProp : selectedInternal;
  const rowRefs = useRef(new Map<string, HTMLTableRowElement>());
  const boxRef = useRef<HTMLDivElement>(null);
  // Sem altura máxima, a tabela rola com a página: o cabeçalho acompanha.
  useStickyHeader(boxRef, !maxHeight);

  const multi = selectionProp !== undefined || multiSelect;
  const marks = selectionMode === 'marks';
  const [selectionInternal, setSelectionInternal] = useState<ReadonlySet<string>>(EMPTY);
  /** Início da faixa do Shift e a seleção que já havia quando ela começou (`null`: a atual). */
  const anchor = useRef<{ id: string; base: ReadonlySet<string> | null } | null>(null);

  const { sortRows } = cols;
  const sortedRows = useMemo(
    () => (serverSort ? rows : sortRows(rows)),
    [rows, sortRows, serverSort],
  );

  const ids = useMemo(() => sortedRows.map(getRowId), [sortedRows, getRowId]);
  const tabbableId = selectedId != null && ids.includes(selectedId) ? selectedId : ids[0];

  const rawSelection = selectionProp ?? selectionInternal;
  const selection = useMemo((): ReadonlySet<string> => {
    // Sem seleção múltipla, a seleção é a linha do cursor.
    if (!multi) return selectedId != null ? new Set([selectedId]) : EMPTY;
    // Marcas podem incluir linhas de outras páginas (quem controla sabe quantas são).
    if (marks) return rawSelection;
    // Seguindo o cursor: só linhas que existem (as excluídas saem), e nunca vazia.
    const live = ids.filter((id) => rawSelection.has(id));
    if (live.length) return new Set(live);
    return selectedId != null && ids.includes(selectedId) ? new Set([selectedId]) : EMPTY;
  }, [multi, marks, rawSelection, selectedId, ids]);
  const selectedIds = ids.filter((id) => selection.has(id));
  const selectedCount = marks ? selection.size : selectedIds.length;
  const rowsOf = (list: readonly string[]) => {
    const set = new Set(list);
    return sortedRows.filter((_, i) => set.has(ids[i]!));
  };

  const select = (id: string | null) => {
    if (selectedIdProp === undefined) setSelectedInternal(id);
    const row = id == null ? null : (sortedRows[ids.indexOf(id)] ?? null);
    onSelectedChange?.(id, row);
  };

  const setSelection = (next: Set<string>) => {
    if (selectionProp === undefined) setSelectionInternal(next);
    onSelectionChange?.(next, rowsOf([...next]));
  };

  /** Cursor numa linha (foco e rolagem); `select` só avisa se mudou. */
  const focusRow = (id: string) => {
    const element = rowRefs.current.get(id);
    element?.focus();
    element?.scrollIntoView({ block: 'nearest' });
    if (id !== selectedId) select(id);
  };

  /** Escolha sem Shift: vira a âncora; seguindo o cursor, a seleção fica só nela. */
  const pick = (id: string) => {
    if (!multi) return;
    if (marks) {
      anchor.current = { id, base: null };
      return;
    }
    anchor.current = { id, base: EMPTY };
    if (!(rawSelection.size === 1 && rawSelection.has(id))) setSelection(new Set([id]));
  };

  const clampIndex = (index: number) => Math.max(0, Math.min(ids.length - 1, index));

  const moveTo = (index: number) => {
    const id = ids[clampIndex(index)];
    if (id == null) return;
    focusRow(id);
    pick(id);
  };

  /** Shift: seleciona da âncora até `index` (somada à seleção de quando a faixa começou). */
  const extendTo = (index: number, from: number) => {
    const to = clampIndex(index);
    const id = ids[to];
    if (id == null) return;
    let a = anchor.current;
    if (!a || !ids.includes(a.id)) a = { id: ids[clampIndex(from)]!, base: null };
    const base = a.base ?? (marks ? selection : EMPTY);
    anchor.current = { id: a.id, base };
    const start = ids.indexOf(a.id);
    const next = new Set(base);
    for (let i = Math.min(start, to); i <= Math.max(start, to); i++) next.add(ids[i]!);
    setSelection(next);
    focusRow(id);
  };

  const toggle = (id: string) => {
    const next = new Set(selection);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    anchor.current = { id, base: next };
    setSelection(next);
  };

  const copyRows = (list: readonly string[]) => {
    const named = shown.flatMap((c, i) => (hasName(c) ? [{ c, i }] : []));
    const lines = [named.map(({ c }) => columnName(c)).join('\t')];
    for (const id of list) {
      const tr = rowRefs.current.get(id);
      if (tr)
        lines.push(named.map(({ i }) => (tr.cells[i] ? visibleText(tr.cells[i]) : '')).join('\t'));
    }
    const n = lines.length - 1;
    void writeClipboard(lines.join('\n')).then((ok) =>
      toast(
        ok
          ? {
              title: t('componentes-linhas-copiadas', { n }),
              description: t('componentes-copiadas-dica'),
            }
          : { tone: 'error', title: t('componentes-nao-copiou') },
      ),
    );
  };

  /** A linha está numa seleção de duas ou mais (e há menu para ela)? */
  const inSelection = (id: string) =>
    multi && selectionMenu !== undefined && selectedCount >= 2 && selection.has(id);

  /** O menu que abre (e cujos atalhos valem) com o cursor em `index`. */
  const menuFor = (index: number) => {
    const id = ids[index]!;
    const row = sortedRows[index]!;
    const returnFocus = rowRefs.current.get(id) ?? null;
    if (inSelection(id)) {
      const list = rowsOf(selectedIds);
      return {
        entries: selectionMenu!(list, { copy: () => copyRows(selectedIds) }),
        label: selectionMenuLabel?.(list) ?? t('componentes-acoes-linhas', { n: selectedCount }),
        returnFocus,
      };
    }
    return {
      entries: rowMenu ? rowMenu(row, { copy: () => copyRows([id]) }) : [],
      label: rowMenuLabel?.(row) ?? t('componentes-acoes-linha'),
      returnFocus,
    };
  };

  /** Antes de abrir o menu: fora da seleção, ela volta à linha (o menu é só dela). */
  const prepareMenu = (id: string) => {
    if (multi && !inSelection(id)) pick(id);
    else if (multi) anchor.current ??= { id, base: null };
    if (id !== selectedId) select(id);
  };

  /** Atalho de um item do menu que abriria agora (itens desativados não disparam). */
  const runShortcut = (event: KeyboardEvent<HTMLTableRowElement>, index: number) => {
    const action = menuFor(index).entries.find(
      (e): e is ContextMenuAction =>
        isAction(e) &&
        !e.disabled &&
        e.shortcut !== undefined &&
        matchesShortcut(event, e.shortcut),
    );
    if (!action) return false;
    event.preventDefault();
    event.stopPropagation();
    action.onSelect();
    return true;
  };

  /** Teclas da seleção múltipla; `true` se tratou. */
  const selectionKey = (event: KeyboardEvent<HTMLTableRowElement>, index: number) => {
    const id = ids[index]!;
    if (event.altKey) return false;
    if (hasModKey(event)) {
      if (event.shiftKey || event.key.toLowerCase() !== 'a') return false;
      const next = new Set(marks ? selection : EMPTY);
      for (const x of ids) next.add(x);
      anchor.current = { id, base: marks ? null : EMPTY };
      setSelection(next);
      return true;
    }
    if (event.shiftKey && NAV_KEYS.has(event.key)) {
      const to =
        event.key === 'ArrowDown'
          ? index + 1
          : event.key === 'ArrowUp'
            ? index - 1
            : event.key === 'Home'
              ? 0
              : event.key === 'End'
                ? ids.length - 1
                : event.key === 'PageDown'
                  ? index + PAGE
                  : index - PAGE;
      extendTo(to, index);
      return true;
    }
    if (event.shiftKey) return false;
    if (event.key === ' ') {
      toggle(id);
      return true;
    }
    if (event.key === 'Escape') {
      if (marks ? selection.size === 0 : selectedIds.length <= 1) return false;
      anchor.current = { id, base: marks ? null : EMPTY };
      setSelection(marks ? new Set() : new Set([id]));
      return true;
    }
    return false;
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLTableRowElement>, index: number, row: Row) => {
    const id = ids[index]!;
    if (rowMenu || selectionMenu) {
      if (menu.onKeyDown(event, menuFor(index))) {
        prepareMenu(id);
        return;
      }
    }
    // Teclas vindas de um campo ou botão da célula ficam com ele.
    const onRow = event.target === event.currentTarget;
    if (multi && onRow && selectionKey(event, index)) {
      event.preventDefault();
      return;
    }
    if (onRow && !event.defaultPrevented && runShortcut(event, index)) return;
    if (event.altKey || event.metaKey || event.ctrlKey) return;
    switch (event.key) {
      case 'ArrowDown':
        moveTo(index + 1);
        break;
      case 'ArrowUp':
        moveTo(index - 1);
        break;
      case 'Home':
        moveTo(0);
        break;
      case 'End':
        moveTo(ids.length - 1);
        break;
      case 'PageDown':
        moveTo(index + PAGE);
        break;
      case 'PageUp':
        moveTo(index - PAGE);
        break;
      case 'Enter':
        onActivate?.(row);
        break;
      case ' ':
        select(id);
        break;
      default:
        return;
    }
    event.preventDefault();
  };

  const handleClick = (event: MouseEvent<HTMLTableRowElement>, index: number) => {
    const id = ids[index]!;
    const control = event.target instanceof Element ? event.target.closest(CONTROL) : null;
    const fromControl =
      control !== null && control !== event.currentTarget && event.currentTarget.contains(control);
    if (multi && !fromControl && event.shiftKey) {
      const from = selectedId != null && ids.includes(selectedId) ? ids.indexOf(selectedId) : index;
      extendTo(index, from);
      return;
    }
    if (multi && !fromControl && hasModKey(event)) {
      toggle(id);
      if (id !== selectedId) select(id);
      return;
    }
    if (id !== selectedId || !multi) select(id);
    pick(id);
  };

  const showStatus = multi && selectionStatus !== false && selectedCount >= 2;

  return (
    <div
      ref={boxRef}
      className={cx('tech-table relative overflow-auto', className)}
      style={maxHeight ? { maxHeight } : undefined}
      onPointerOver={revealTruncated}
    >
      <table
        role="grid"
        aria-label={label}
        aria-multiselectable={multi || undefined}
        className={cx('border-separate border-spacing-0', !fixed && 'w-full')}
        style={cols.tableStyle}
      >
        {cols.colgroup}
        <thead className="sticky top-0 z-[1] sticky-surface">
          <tr>
            {shown.map((c) =>
              cols.headerCell(c, {
                className: 'label-caps h-8 border-b border-line px-3 font-medium whitespace-nowrap',
              }),
            )}
            {cols.pickerCell()}
          </tr>
        </thead>
        <tbody>
          {sortedRows.length === 0 ? (
            <tr>
              <td
                role="gridcell"
                colSpan={cols.colSpan}
                className="px-3 py-10 text-center text-[13px] text-dim"
              >
                {empty ?? t('componentes-nenhuma-linha')}
              </td>
            </tr>
          ) : (
            sortedRows.map((row, index) => {
              const id = ids[index]!;
              const cursor = id === selectedId;
              return (
                <tr
                  key={id}
                  ref={(el) => {
                    if (el) rowRefs.current.set(id, el);
                    else rowRefs.current.delete(id);
                  }}
                  data-row-id={id}
                  data-cursor={cursor || undefined}
                  aria-selected={multi ? selection.has(id) : cursor}
                  tabIndex={id === tabbableId ? 0 : -1}
                  onClick={(event) => handleClick(event, index)}
                  onDoubleClick={() => onActivate?.(row)}
                  onKeyDown={(event) => handleKeyDown(event, index, row)}
                  onContextMenu={
                    rowMenu || selectionMenu
                      ? (event) => {
                          const el = rowRefs.current.get(id);
                          if (menu.onContextMenu(event, menuFor(index))) {
                            prepareMenu(id);
                            if (!el?.contains(document.activeElement)) el?.focus();
                          }
                        }
                      : undefined
                  }
                  className="tech-row cursor-default outline-none"
                >
                  {shown.map((c) => (
                    <td
                      role="gridcell"
                      key={c.key}
                      className={cx(
                        'border-b border-line/70 px-3 py-1 text-[13px] text-fg',
                        dense ? 'h-8' : 'h-10',
                        c.numeric && 'num whitespace-nowrap',
                        fixed && 'overflow-hidden text-ellipsis',
                        alignClass(c),
                        c.className,
                      )}
                    >
                      {c.cell ? c.cell(row, index) : defaultCell(row, c.key)}
                    </td>
                  ))}
                  {cols.fillerCell('border-b border-line/70')}
                </tr>
              );
            })
          )}
        </tbody>
        {footer ? (
          <tfoot>
            <tr>
              {shown.map((c) => (
                <td
                  role="gridcell"
                  key={c.key}
                  className={cx(
                    'h-10 border-t border-line-strong/60 px-3 text-[13px] font-medium text-fg',
                    c.numeric && 'num whitespace-nowrap',
                    alignClass(c),
                  )}
                >
                  {footer[c.key]}
                </td>
              ))}
              {cols.fillerCell('border-t border-line-strong/60')}
            </tr>
          </tfoot>
        ) : null}
      </table>
      {showStatus ? (
        <div
          role="status"
          className="sticky bottom-0 left-0 flex h-8 items-center gap-3 border-t border-line sticky-surface px-3 text-[12px] text-dim"
        >
          {selectionStatus ? (
            selectionStatus(rowsOf(selectedIds))
          ) : (
            <span className="num text-accent">
              {t('componentes-linhas-selecionadas', { n: selectedCount })}
            </span>
          )}
        </div>
      ) : null}
      {menu.menu}
      {cols.menus}
    </div>
  );
}
