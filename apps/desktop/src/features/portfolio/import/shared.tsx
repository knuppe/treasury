/**
 * Peças comuns dos assistentes de importação de investimentos (B3 e notas de
 * corretagem): etapas, selo de situação, filtros por situação, avisos, tabela
 * compacta e leitura dos arquivos em base64 (só na memória, até gravar).
 */
import { Fragment, useMemo, type ReactNode } from 'react';

import { useDataColumns, type DataColumn } from '@/design/components';
import { cx } from '@/design/cx';
import { fileToBase64 } from '@/features/transactions/ui/hooks';
import { t } from '@/i18n';

export interface PickedFile {
  filename: string;
  size: number;
  content_base64: string;
}

export async function readFiles(list: FileList | null | undefined): Promise<PickedFile[]> {
  const files = Array.from(list ?? []);
  return Promise.all(
    files.map(async (f) => ({
      filename: f.name,
      size: f.size,
      content_base64: await fileToBase64(f),
    })),
  );
}

export function Steps<S extends string>({
  steps,
  current,
}: {
  steps: readonly { id: S; label: string }[];
  current: S;
}) {
  const at = steps.findIndex((s) => s.id === current);
  return (
    <ol className="flex flex-wrap items-center gap-2" aria-label={t('b3-etapas')}>
      {steps.map((s, i) => (
        <li
          key={s.id}
          aria-current={s.id === current ? 'step' : undefined}
          className={cx(
            'label-caps flex items-center gap-2',
            s.id === current ? 'text-accent' : at > i ? 'text-fg' : undefined,
          )}
        >
          <span className="num">{String(i + 1).padStart(2, '0')}</span>
          {s.label}
          {i < steps.length - 1 ? (
            <span aria-hidden className="text-line-strong">
              —
            </span>
          ) : null}
        </li>
      ))}
    </ol>
  );
}

export type Tone = 'positive' | 'warning' | 'negative' | 'dim' | 'accent';

const TONE: Record<Tone, string> = {
  positive: 'border-positive/50 text-positive',
  warning: 'border-warning/60 text-warning',
  negative: 'border-negative/60 text-negative',
  dim: 'border-line-strong text-dim',
  accent: 'border-accent/60 text-accent',
};

export function StatusChip({
  tone,
  children,
  title,
}: {
  tone: Tone;
  children: ReactNode;
  title?: string;
}) {
  return (
    <span
      title={title}
      className={cx(
        'label-caps inline-flex h-5 items-center rounded-control border px-1.5 whitespace-nowrap',
        TONE[tone],
      )}
    >
      {children}
    </span>
  );
}

/** Botões de filtro por situação ("Todas 12", "Nova 8"…); some quando a contagem é zero. */
export function StatusFilter<S extends string>({
  label,
  value,
  onChange,
  options,
  total,
}: {
  label: string;
  value: S | 'all';
  onChange: (v: S | 'all') => void;
  options: readonly { value: S; label: string; tone: Tone; count: number }[];
  total: number;
}) {
  return (
    <div className="flex flex-wrap items-center gap-2" role="group" aria-label={label}>
      {[
        { value: 'all' as const, label: t('b3-todas'), tone: 'dim' as Tone, count: total },
        ...options,
      ]
        .filter((o) => o.value === 'all' || o.count > 0)
        .map((o) => (
          <button
            key={o.value}
            type="button"
            aria-pressed={value === o.value}
            onClick={() => onChange(o.value)}
            className={cx(
              'label-caps inline-flex h-7 items-center gap-1.5 rounded-control border px-2',
              value === o.value
                ? 'border-accent bg-accent-soft text-accent'
                : o.value === 'all'
                  ? 'border-line'
                  : TONE[o.tone],
            )}
          >
            {o.label} <span className="num">{o.count}</span>
          </button>
        ))}
    </div>
  );
}

export function Warnings({
  items,
  label = t('b3-avisos'),
}: {
  items: readonly string[];
  label?: string;
}) {
  if (!items.length) return null;
  return (
    <ul
      className="flex flex-col gap-1 rounded-card border border-warning/50 bg-warning/5 px-3 py-2 text-[12.5px] text-warning"
      aria-label={label}
    >
      {items.map((w) => (
        <li key={w} className="first-letter:uppercase">
          {w}
        </li>
      ))}
    </ul>
  );
}

export interface PreviewColumn<Row> extends DataColumn<Row> {
  cell: (row: Row) => ReactNode;
  /** Classe da célula (padrão: `TD`, ou `TD_NUM` nas numéricas). */
  className?: string;
}

/**
 * Tabela compacta das prévias (rolagem própria, cabeçalho fixo), com colunas ordenáveis,
 * de largura ajustável e escolhidas pelo usuário (lembrado por tabela).
 */
export function PreviewTable<Row>({
  label,
  prefsId,
  columns,
  rows,
  getRowId,
  rowClassName,
  after,
}: {
  label: string;
  /** Chave estável das preferências de colunas (o título em pt-BR de antes da tradução). */
  prefsId?: string;
  columns: readonly PreviewColumn<Row>[];
  rows: readonly Row[];
  getRowId: (row: Row) => string | number;
  rowClassName?: (row: Row) => string | false | undefined;
  /** Linhas extras logo abaixo da linha (ex.: editor que ocupa a largura toda). */
  after?: (row: Row, colSpan: number) => ReactNode;
}) {
  const cols = useDataColumns({ label, prefsId, columns, rows });
  const { sortRows } = cols;
  const sorted = useMemo(() => sortRows(rows), [rows, sortRows]);
  return (
    <div className="max-h-[42vh] overflow-auto rounded-card border border-line">
      <table
        className={cx('border-separate border-spacing-0 text-[12.5px]', !cols.fixed && 'w-full')}
        style={cols.tableStyle}
        aria-label={label}
      >
        {cols.colgroup}
        <thead className="sticky top-0 z-[1] sticky-surface">
          <tr>
            {cols.shown.map((c) =>
              cols.headerCell(c, {
                className:
                  'label-caps border-b border-line px-2 py-1.5 font-medium whitespace-nowrap',
              }),
            )}
            {cols.pickerCell('border-b border-line px-1 py-1.5')}
          </tr>
        </thead>
        <tbody>
          {sorted.length === 0 ? (
            <tr>
              <td colSpan={cols.colSpan} className="px-2 py-6 text-center text-dim">
                {t('componentes-nenhuma-linha')}
              </td>
            </tr>
          ) : (
            sorted.map((row) => {
              const id = getRowId(row);
              return (
                <Fragment key={id}>
                  <tr className={rowClassName?.(row) || undefined}>
                    {cols.shown.map((c) => (
                      <td
                        key={c.key}
                        className={cx(
                          c.className ?? (c.numeric ? TD_NUM : TD),
                          cols.fixed && 'overflow-hidden text-ellipsis',
                        )}
                      >
                        {c.cell(row)}
                      </td>
                    ))}
                    {cols.fillerCell('border-b border-line/60')}
                  </tr>
                  {after?.(row, cols.colSpan)}
                </Fragment>
              );
            })
          )}
        </tbody>
      </table>
      {cols.menus}
    </div>
  );
}

/** Valor decimal (texto do núcleo) como número, para ordenar. */
export const decimal = (v: string | null | undefined) => (v == null || v === '' ? null : Number(v));

export const TD = 'border-b border-line/60 px-2 py-1 align-top';
export const TD_NUM =
  'num border-b border-line/60 px-2 py-1 text-right align-top whitespace-nowrap';

/** Explicação curta embaixo do selo (nota ou erro da linha). */
export function RowNote({
  children,
  tone,
}: {
  children: ReactNode;
  tone?: 'negative' | 'warning';
}) {
  if (!children) return null;
  return (
    <span
      className={cx(
        'mt-0.5 block max-w-64 text-[11px] leading-snug first-letter:uppercase',
        tone === 'negative' ? 'text-negative' : tone === 'warning' ? 'text-warning' : 'text-dim',
      )}
    >
      {children}
    </span>
  );
}

export function SideLabel({ side }: { side: 'buy' | 'sell' }) {
  return (
    <span className={side === 'buy' ? 'text-positive' : 'text-negative'}>
      <span aria-hidden>{side === 'buy' ? '▲ ' : '▼ '}</span>
      {side === 'buy' ? t('b3-compra') : t('b3-venda')}
    </span>
  );
}
