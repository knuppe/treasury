/**
 * Revisão amigável de um conjunto de alterações (fila de aprovação, proposta na conversa e
 * histórico): frases do que muda, uma ficha por linha com nomes no lugar de ids, campos
 * em português e valores formatados. "Ver detalhes técnicos" abre o diff exato
 * (`DiffView`), com ids e valores brutos, como a fila sempre mostrou.
 */
import { useId, useMemo, useState } from 'react';

import type { ChangeItem } from '@/api/generated/ChangeItem';
import { Icon, Private } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { describeChanges, friendlyRows, type FriendlyRow } from './changeText';
import { DiffView } from './DiffView';
import { Pill } from './ui';
import { useChangeNames } from './useChangeNames';

const OP_TONE = { insert: 'positive', update: 'accent', delete: 'negative' } as const;

/** Linhas de ligação que só acompanham a principal: ficam nos detalhes técnicos. */
const QUIET = new Set(['rule_hit']);

function Value({ text, sensitive }: { text: string; sensitive: boolean }) {
  return sensitive ? <Private>{text}</Private> : <>{text}</>;
}

function RowCard({ row }: { row: FriendlyRow }) {
  const single = row.op !== 'update';
  return (
    <section
      aria-label={`${row.opLabel} · ${row.tableLabel}: ${row.title.name}`}
      className="rounded-control border border-line"
    >
      <header className="flex flex-wrap items-center gap-x-2 gap-y-1 border-b border-line bg-surface-2 px-3 py-1.5">
        <Pill tone={OP_TONE[row.op]}>{row.opLabel}</Pill>
        <span className="label-caps text-[10px]">{row.tableLabel}</span>
        <span className="min-w-0 truncate text-[12.5px] font-medium text-fg">{row.title.name}</span>
        {row.title.detail ? (
          <span className="num text-[11.5px] text-dim">{row.title.detail}</span>
        ) : null}
        {row.title.money ? (
          <span className="num ml-auto text-[12px] text-fg">
            <Private>{row.title.money}</Private>
          </span>
        ) : null}
      </header>
      {row.fields.length > 0 ? (
        <table className="w-full border-collapse text-left text-[12.5px]">
          <thead className="sr-only">
            <tr>
              <th scope="col">{t('agente-campo')}</th>
              {single ? (
                <th scope="col">{t('agente-valor')}</th>
              ) : (
                <>
                  <th scope="col">{t('agente-antes')}</th>
                  <th scope="col">{t('agente-depois')}</th>
                </>
              )}
            </tr>
          </thead>
          <tbody>
            {row.fields.map((f) => (
              <tr key={f.field} className="border-t border-line/60 align-top first:border-t-0">
                <th scope="row" className="label-caps w-44 px-3 py-1 text-[10px] font-normal">
                  {f.label}
                </th>
                {single ? (
                  <td
                    className={cx(
                      'px-3 py-1 break-words',
                      row.op === 'delete' ? 'text-dim line-through' : 'text-fg',
                    )}
                  >
                    <Value
                      text={row.op === 'delete' ? f.before : f.after}
                      sensitive={f.sensitive}
                    />
                  </td>
                ) : (
                  <td className="px-3 py-1 break-words">
                    <span className="text-dim line-through decoration-negative/50">
                      <Value text={f.before} sensitive={f.sensitive} />
                    </span>
                    <span aria-hidden className="mx-1.5 text-positive">
                      {'→'}
                    </span>
                    <span className="sr-only">{` ${t('agente-para')} `}</span>
                    <span className="text-fg">
                      <Value text={f.after} sensitive={f.sensitive} />
                    </span>
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      ) : null}
    </section>
  );
}

export function ChangeReview({
  items,
  maxRows = 8,
  showSummary = true,
}: {
  items: readonly ChangeItem[];
  /** Fichas mostradas antes de "Mostrar as N linhas". */
  maxRows?: number;
  showSummary?: boolean;
}) {
  const book = useChangeNames(items);
  const rows = useMemo(
    () =>
      friendlyRows(items, book).filter(
        (r) => !QUIET.has(r.table) && (r.op !== 'update' || r.fields.length > 0),
      ),
    [items, book],
  );
  const summary = useMemo(() => describeChanges(items, book), [items, book]);
  const [all, setAll] = useState(false);
  const [technical, setTechnical] = useState(false);
  const techId = useId();
  if (items.length === 0) {
    return <p className="text-[12.5px] text-dim">{t('agente-nenhuma-linha-alteracao')}</p>;
  }
  const visible = all ? rows : rows.slice(0, maxRows);
  return (
    <div className="flex flex-col gap-2">
      {showSummary && summary.length > 0 ? (
        <ul
          aria-label={t('agente-o-que-muda')}
          className="flex flex-col gap-0.5 text-[13px] text-fg"
        >
          {summary.map((s) => (
            <li key={s} className="flex gap-2">
              <span aria-hidden className="text-accent">
                {'›'}
              </span>
              <span className="min-w-0">{s}</span>
            </li>
          ))}
        </ul>
      ) : null}
      {visible.map((r) => (
        <RowCard key={r.key} row={r} />
      ))}
      <div className="flex flex-wrap items-center gap-3">
        {rows.length > visible.length ? (
          <button
            type="button"
            className="label-caps text-[10px] hover:text-fg"
            onClick={() => setAll(true)}
          >
            {t('agente-mostrar-linhas', { n: rows.length })}
          </button>
        ) : null}
        <button
          type="button"
          aria-expanded={technical}
          aria-controls={techId}
          onClick={() => setTechnical((v) => !v)}
          className="label-caps inline-flex items-center gap-1 text-[10px] hover:text-fg"
        >
          <Icon
            name="chevronRight"
            size={11}
            className={cx('transition-transform', technical && 'rotate-90')}
          />
          {technical ? t('agente-ocultar-tecnicos') : t('agente-ver-tecnicos')}
        </button>
      </div>
      {technical ? (
        <div id={techId} className="flex flex-col gap-1.5">
          <p className="text-[11.5px] text-dim">{t('agente-diff-exato')}</p>
          <DiffView items={items} />
        </div>
      ) : null}
    </div>
  );
}
