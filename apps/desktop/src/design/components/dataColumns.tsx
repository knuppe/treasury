/**
 * Colunas ajustáveis de uma tabela de dados: ordenar, mudar a largura e escolher as
 * colunas visíveis, lembrado por tabela (`tablePrefs`). É o miolo da `TechTable`, e serve
 * às tabelas com marcação própria (linhas editáveis, hierarquia, tabela transposta): elas
 * desenham as linhas e pedem aqui o cabeçalho, o `<colgroup>` e a ordem.
 */
import {
  isValidElement,
  useCallback,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type PointerEvent,
  type ReactNode,
} from 'react';

import { Checkbox } from '@/features/transactions/ui/controls';
import { Popover } from '@/features/transactions/ui/Popover';
import { t } from '@/i18n';

import { cx } from '../cx';
import { AutoHelp } from '../help';
import { useContextMenu, type ContextMenuEntry } from './ContextMenu';
import { Icon } from './Icon';
import { tableKey, useTablePrefs } from './tablePrefs';

export type SortDirection = 'asc' | 'desc';

export interface SortState {
  key: string;
  direction: SortDirection;
}

export type SortValue = string | number | null | undefined;

export interface DataColumn<Row> {
  key: string;
  header: ReactNode;
  /** Coluna numérica: alinhada à direita (a menos que `align` diga outra coisa). */
  numeric?: boolean;
  align?: 'left' | 'right' | 'center';
  /** Largura CSS (`120px`, `20%`) enquanto o usuário não escolhe uma. */
  width?: string;
  /**
   * Valor para ordenar. Sem ele, a coluna ordena pelo valor da linha na mesma chave
   * (`row[key]`), quando é texto ou número.
   */
  sortValue?: (row: Row) => SortValue;
  /** `false`: não ordena (nem pelo valor da linha). */
  sortable?: boolean;
  /** Nome da coluna no seletor de colunas, quando `header` não é texto. */
  title?: string;
  /** `false`: sempre visível (a coluna que identifica a linha, seleção). */
  hideable?: boolean;
  /** Começa escondida; o usuário pode mostrar no seletor de colunas. */
  defaultHidden?: boolean;
  /** Conteúdo da célula; sem `sortValue`, a coluna ordena pelo texto que ele mostra. */
  cell?: (row: Row, index: number) => ReactNode;
}

export interface DataColumnsOptions<Row, C extends DataColumn<Row> = DataColumn<Row>> {
  /** Nome acessível da tabela (também nomeia o seletor: "Colunas de <label>"). */
  label: string;
  /** Onde guardar as preferências; padrão: o nome da tabela. */
  prefsId?: string;
  columns: readonly C[];
  /** Linhas (para descobrir as colunas ordenáveis pelo valor bruto). */
  rows: readonly Row[];
  /** `false`: sem ajuste de colunas. */
  customizable?: boolean;
  /** Ordenação (controlada). Omita para o hook cuidar sozinho. */
  sort?: SortState | null;
  defaultSort?: SortState | null;
  onSortChange?: (sort: SortState | null) => void;
  /**
   * Larguras padrão em px por coluna: a tabela já começa com `table-layout: fixed` e o
   * duplo clique na alça volta a elas.
   */
  defaultWidths?: Readonly<Record<string, number>>;
  /** Com larguras fixas, ocupa ao menos a largura do contêiner (as colunas crescem juntas). */
  fill?: boolean;
  /** Largura mínima da tabela no layout automático (`880px`). */
  minWidth?: string;
}

const collator = new Intl.Collator('pt-BR', { numeric: true, sensitivity: 'base' });

export function compareValues(a: SortValue, b: SortValue): number {
  const aNull = a == null || a === '';
  const bNull = b == null || b === '';
  // Vazios sempre no fim, nos dois sentidos.
  if (aNull || bNull) return aNull === bNull ? 0 : aNull ? 1 : -1;
  if (typeof a === 'number' && typeof b === 'number') return a - b;
  return collator.compare(String(a), String(b));
}

/** Largura de uma coluna nova quando as outras já têm largura escolhida. */
const NEW_WIDTH = 120;
const MIN_WIDTH = 48;
/** Coluna do seletor de colunas, no fim do cabeçalho. */
export const PICKER_WIDTH = 32;

const isPlain = (v: unknown): v is string | number =>
  typeof v === 'string' || typeof v === 'number';

/** Id interno (UUID): nunca serve de ordem para quem lê a tabela. */
const isId = (v: unknown) =>
  typeof v === 'string' &&
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(v);

/** Texto visível de um conteúdo de célula (textos e filhos de elementos, em ordem). */
export function textOf(node: ReactNode): string {
  if (node == null || typeof node === 'boolean') return '';
  if (typeof node === 'string' || typeof node === 'number') return String(node);
  if (Array.isArray(node)) return node.map(textOf).join(' ');
  if (isValidElement(node)) {
    const props = node.props as { children?: ReactNode; title?: unknown };
    const inner = textOf(props.children);
    return inner || (typeof props.title === 'string' ? props.title : '');
  }
  return '';
}

/** Coluna que o usuário pode esconder: tem nome (ações e seleção ficam sempre). */
export function canHide<Row>(c: DataColumn<Row>): boolean {
  return c.hideable ?? (c.title ? true : isPlain(c.header) && String(c.header).trim() !== '');
}

export function columnName<Row>(c: DataColumn<Row>): string {
  return c.title ?? (isPlain(c.header) ? String(c.header) : c.key);
}

export function alignClass<Row>(c: DataColumn<Row>): string {
  const a = c.align ?? (c.numeric ? 'right' : 'left');
  return a === 'right' ? 'text-right' : a === 'center' ? 'text-center' : 'text-left';
}

export interface HeaderCellOptions {
  className?: string;
  /**
   * O cabeçalho tem um controle próprio (link): a ordenação vira um botão de ícone ao
   * lado, em vez de envolver o conteúdo.
   */
  sortIconOnly?: boolean;
}

export function useDataColumns<Row, C extends DataColumn<Row> = DataColumn<Row>>({
  label,
  prefsId,
  columns,
  rows,
  customizable = true,
  sort: sortProp,
  defaultSort = null,
  onSortChange,
  defaultWidths,
  fill = false,
  minWidth,
}: DataColumnsOptions<Row, C>) {
  const headerMenu = useContextMenu();
  const { prefs, update, reset } = useTablePrefs(tableKey(prefsId ?? label));
  const [sortInternal, setSortInternal] = useState<SortState | null>(
    customizable && prefs.sort !== undefined ? prefs.sort : defaultSort,
  );
  const sort = sortProp !== undefined ? sortProp : sortInternal;
  const headRefs = useRef(new Map<string, HTMLTableCellElement>());

  // Valor de ordenação de cada coluna: o declarado; sem ele, o valor bruto da linha ou,
  // quando a chave guarda um id (ou nada), o texto que a célula mostra.
  const sorters = useMemo(() => {
    const m = new Map<string, (row: Row) => SortValue>();
    const raw = (r: Row, key: string) => (r as Record<string, unknown>)[key];
    for (const c of columns) {
      if (c.sortable === false) continue;
      if (c.sortValue) {
        m.set(c.key, c.sortValue);
        continue;
      }
      const cell = c.cell;
      if (cell) {
        // O valor bruto quando é dado de verdade (data ISO, número, texto); o texto da
        // célula quando a chave guarda um id ou nada.
        const value = (r: Row, i: number): SortValue => {
          const v = raw(r, c.key);
          if (isPlain(v) && !isId(v)) return v;
          const t = textOf(cell(r, i)).replace(/\s+/g, ' ').trim();
          return t && t !== '—' ? t : null;
        };
        if (rows.some((r, i) => value(r, i) != null)) {
          const index = new Map(rows.map((r, i) => [r, i]));
          m.set(c.key, (r) => value(r, index.get(r) ?? 0));
        }
      } else if (rows.some((r) => isPlain(raw(r, c.key)) && !isId(raw(r, c.key))))
        m.set(c.key, (r) => {
          const v = raw(r, c.key);
          return isPlain(v) && !isId(v) ? v : null;
        });
    }
    return m;
  }, [columns, rows]);

  const hidden = useMemo(
    () =>
      new Set(
        customizable
          ? (prefs.hidden ?? columns.filter((c) => c.defaultHidden).map((c) => c.key))
          : [],
      ),
    [customizable, prefs.hidden, columns],
  );
  const shown = useMemo(() => {
    const v = columns.filter((c) => !canHide(c) || !hidden.has(c.key));
    return v.length ? v : columns.slice(0, 1);
  }, [columns, hidden]);
  const hideable = columns.filter(canHide);
  const shownHideable = hideable.filter((c) => !hidden.has(c.key)).length;
  const widths = customizable ? prefs.widths : undefined;
  const chosen = Boolean(widths && Object.keys(widths).length);
  const fixed = chosen || Boolean(defaultWidths);
  const widthOf = (c: DataColumn<Row>) => widths?.[c.key] ?? defaultWidths?.[c.key] ?? NEW_WIDTH;
  const extra = customizable ? 1 : 0;
  const total = shown.reduce((a, c) => a + widthOf(c), 0) + (customizable ? PICKER_WIDTH : 0);

  const setHidden = (key: string, hide: boolean) =>
    update((p) => {
      const cur = new Set(p.hidden ?? columns.filter((c) => c.defaultHidden).map((c) => c.key));
      if (hide) cur.add(key);
      else cur.delete(key);
      return { hidden: [...cur] };
    });

  // Largura: na primeira mudança, as colunas visíveis guardam a largura que têm agora e a
  // tabela passa a usar as larguras escolhidas.
  const measured = (): Record<string, number> => {
    const out: Record<string, number> = { ...(widths ?? {}) };
    for (const c of shown) {
      if (out[c.key]) continue;
      const preset = defaultWidths?.[c.key];
      const w = preset ?? headRefs.current.get(c.key)?.getBoundingClientRect().width;
      out[c.key] = Math.max(MIN_WIDTH, Math.round(w || NEW_WIDTH));
    }
    return out;
  };
  const drag = useRef<{ key: string; x: number; w: number; base: Record<string, number> } | null>(
    null,
  );
  const resizeHandle = (c: DataColumn<Row>) => ({
    onPointerDown: (e: PointerEvent<HTMLElement>) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      e.currentTarget.setPointerCapture?.(e.pointerId);
      const base = measured();
      drag.current = { key: c.key, x: e.clientX, w: base[c.key] ?? NEW_WIDTH, base };
      update({ widths: base }, false);
    },
    onPointerMove: (e: PointerEvent<HTMLElement>) => {
      const d = drag.current;
      if (!d) return;
      const w = Math.max(MIN_WIDTH, Math.round(d.w + e.clientX - d.x));
      update({ widths: { ...d.base, [d.key]: w } }, false);
    },
    onPointerUp: () => {
      if (!drag.current) return;
      drag.current = null;
      update({}, true);
    },
    onPointerCancel: () => {
      drag.current = null;
    },
    onKeyDown: (e: KeyboardEvent<HTMLElement>) => {
      if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return;
      e.preventDefault();
      const base = measured();
      const step = e.shiftKey ? 64 : 16;
      const w = (base[c.key] ?? NEW_WIDTH) + (e.key === 'ArrowRight' ? step : -step);
      update({ widths: { ...base, [c.key]: Math.max(MIN_WIDTH, w) } });
    },
    onDoubleClick: (e: { stopPropagation: () => void }) => {
      e.stopPropagation();
      update({ widths: {} });
    },
    onClick: (e: { stopPropagation: () => void }) => e.stopPropagation(),
  });

  const compare = useMemo(() => {
    if (!sort) return null;
    const sortValue = sorters.get(sort.key);
    if (!sortValue) return null;
    const factor = sort.direction === 'asc' ? 1 : -1;
    return (a: Row, b: Row) => compareValues(sortValue(a), sortValue(b)) * factor;
  }, [sorters, sort]);

  /** As linhas na ordem escolhida (estável: empates mantêm a ordem original). */
  const sortRows = useCallback(
    <R extends Row>(list: readonly R[]): readonly R[] => {
      if (!compare) return list;
      return list
        .map((row, index) => ({ row, index }))
        .sort((a, b) => compare(a.row, b.row) || a.index - b.index)
        .map((e) => e.row);
    },
    [compare],
  );

  const setSort = (next: SortState | null) => {
    if (sortProp === undefined) {
      setSortInternal(next);
      if (customizable) update({ sort: next });
    }
    onSortChange?.(next);
  };
  const toggleSort = (key: string) =>
    setSort(
      sort?.key === key
        ? { key, direction: sort.direction === 'asc' ? 'desc' : 'asc' }
        : { key, direction: 'asc' },
    );

  const restore = () => {
    reset();
    if (sortProp === undefined) setSortInternal(defaultSort);
  };

  const headerEntries = (c: DataColumn<Row>): ContextMenuEntry[] => {
    const name = columnName(c);
    const canSort = sorters.has(c.key);
    return [
      canSort && {
        label: t('componentes-ordenar-crescente'),
        icon: 'arrowUp',
        onSelect: () => setSort({ key: c.key, direction: 'asc' }),
      },
      canSort && {
        label: t('componentes-ordenar-decrescente'),
        icon: 'arrowDown',
        onSelect: () => setSort({ key: c.key, direction: 'desc' }),
      },
      canSort && sort
        ? { label: t('componentes-tirar-ordenacao'), onSelect: () => setSort(null) }
        : null,
      canSort && { separator: true },
      canHide(c) && shownHideable > 1
        ? {
            label: t('componentes-esconder-coluna', { coluna: name }),
            icon: 'eyeOff',
            onSelect: () => setHidden(c.key, true),
          }
        : null,
      chosen
        ? {
            label: defaultWidths
              ? t('componentes-larguras-padrao')
              : t('componentes-larguras-automaticas'),
            onSelect: () => update({ widths: {} }),
          }
        : null,
      { label: t('componentes-restaurar-colunas'), onSelect: restore },
    ];
  };

  const tableStyle: CSSProperties | undefined = fixed
    ? fill
      ? { tableLayout: 'fixed', width: '100%', minWidth: total }
      : { tableLayout: 'fixed', width: total }
    : minWidth
      ? { minWidth }
      : undefined;

  const colgroup = (
    <colgroup>
      {shown.map((c) => (
        <col
          key={c.key}
          style={fixed ? { width: widthOf(c) } : c.width ? { width: c.width } : undefined}
        />
      ))}
      {customizable ? <col style={{ width: PICKER_WIDTH }} /> : null}
    </colgroup>
  );

  const sortIcon = (active: boolean) => (
    <Icon
      name={active ? (sort?.direction === 'asc' ? 'arrowUp' : 'arrowDown') : 'sort'}
      size={11}
      className={active ? 'text-accent' : 'opacity-50'}
    />
  );

  /** O `<th>` da coluna: botão de ordenar, alça de largura e menu de contexto. */
  const headerCell = (c: DataColumn<Row>, opts: HeaderCellOptions = {}) => {
    const sortable = sorters.has(c.key);
    const active = sort?.key === c.key;
    const ariaSort = sortable
      ? active
        ? sort.direction === 'asc'
          ? 'ascending'
          : 'descending'
        : 'none'
      : undefined;
    const right = alignClass(c) === 'text-right';
    return (
      <th
        key={c.key}
        ref={(el) => {
          if (el) headRefs.current.set(c.key, el);
          else headRefs.current.delete(c.key);
        }}
        scope="col"
        aria-sort={ariaSort}
        onContextMenu={
          customizable
            ? (event) =>
                headerMenu.onContextMenu(event, {
                  entries: headerEntries(c),
                  label: `Coluna ${columnName(c)}`,
                })
            : undefined
        }
        className={cx(
          'relative',
          opts.className,
          fixed && 'overflow-hidden text-ellipsis',
          alignClass(c),
          active && 'text-fg',
        )}
      >
        {sortable && opts.sortIconOnly ? (
          <span className={cx('inline-flex items-start gap-1', right && 'flex-row-reverse')}>
            <button
              type="button"
              aria-label={t('componentes-ordenar-por', { coluna: columnName(c) })}
              onClick={() => toggleSort(c.key)}
              className="mt-0.5 inline-flex rounded-control text-dim transition-colors hover:text-fg"
            >
              {sortIcon(active)}
            </button>
            {c.header}
            <AutoHelp label={c.header} className="ml-0.5 self-center" />
          </span>
        ) : sortable ? (
          <button
            type="button"
            onClick={() => toggleSort(c.key)}
            className={cx(
              'inline-flex items-center gap-1 rounded-control uppercase transition-colors hover:text-fg',
              right && 'flex-row-reverse',
            )}
          >
            {c.header}
            {sortIcon(active)}
          </button>
        ) : (
          c.header
        )}
        {sortable && opts.sortIconOnly ? null : (
          <AutoHelp label={c.header} className="ml-1 inline-flex align-middle" />
        )}
        {customizable ? (
          <span
            role="separator"
            aria-orientation="vertical"
            aria-label={t('componentes-largura-coluna', { coluna: columnName(c) })}
            aria-valuenow={fixed ? widthOf(c) : undefined}
            // Fora da ordem do Tab (seria uma parada por coluna); com o foco nela
            // (clique), as setas ajustam.
            tabIndex={-1}
            title={
              defaultWidths
                ? t('componentes-largura-dica-padrao')
                : t('componentes-largura-dica-automaticas')
            }
            className="absolute top-1 right-0 bottom-1 z-[2] w-1.5 cursor-col-resize touch-none border-r border-transparent outline-none hover:border-accent focus-visible:border-accent"
            {...resizeHandle(c)}
          />
        ) : null}
      </th>
    );
  };

  /** Última célula do cabeçalho: o seletor de colunas visíveis. */
  const pickerCell = (className = 'h-8 border-b border-line px-1') =>
    customizable ? (
      <th scope="col" className={cx('w-8 text-right', className)}>
        <Popover
          variant="bare"
          align="end"
          ariaLabel={t('componentes-colunas-de', { tabela: label })}
          label={<Icon name="sliders" size={12} className="text-dim" />}
          panelClassName="w-64"
        >
          <div
            className="flex flex-col gap-1.5"
            role="group"
            aria-label={t('componentes-colunas-visiveis')}
          >
            <p className="label-caps px-1 pb-1">{t('componentes-colunas')}</p>
            {hideable.map((c) => (
              <Checkbox
                key={c.key}
                className="px-1"
                label={columnName(c)}
                checked={!hidden.has(c.key)}
                disabled={!hidden.has(c.key) && shownHideable <= 1}
                onChange={(on) => setHidden(c.key, !on)}
              />
            ))}
            <button
              type="button"
              onClick={restore}
              className="mt-1 self-start rounded-control px-1 text-[12px] text-dim hover:text-accent"
            >
              {t('componentes-restaurar-tudo')}
            </button>
          </div>
        </Popover>
      </th>
    ) : null;

  /** Célula vazia sob o seletor, no fim de cada linha. */
  const fillerCell = (className?: string) =>
    customizable ? <td aria-hidden className={className} /> : null;

  return {
    shown,
    sort,
    setSort,
    compare,
    sortRows,
    fixed,
    customizable,
    tableStyle,
    colgroup,
    headerCell,
    pickerCell,
    fillerCell,
    /** Colunas na linha (as visíveis mais a do seletor), para `colSpan`. */
    colSpan: shown.length + extra,
    /** Menu do cabeçalho; renderize uma vez junto da tabela. */
    menus: headerMenu.menu,
  };
}
