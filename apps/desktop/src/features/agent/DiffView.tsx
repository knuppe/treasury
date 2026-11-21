/** Diff exato de uma proposta: uma tabela antes/depois por linha do banco. */
import { useState } from 'react';

import type { ChangeItem } from '@/api/generated/ChangeItem';
import { Private } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatMoney } from '@/lib/format';

import { diffItems, formatDiffValue, isMoneyField, visibleFields, type DiffRow } from './diff';
import { Pill } from './ui';

function Value({ field, value }: { field: string; value: unknown }) {
  return (
    <Private>
      <span className="font-mono text-[11.5px] break-all">{formatDiffValue(value)}</span>
      {isMoneyField(field, value) ? (
        <span className="ml-1.5 text-[11px] text-dim">{`= ${formatMoney(value)}`}</span>
      ) : null}
    </Private>
  );
}

const OP_TONE = { insert: 'positive', update: 'accent', delete: 'negative' } as const;

function RowDiff({ row }: { row: DiffRow }) {
  const [showAll, setShowAll] = useState(false);
  const fields = visibleFields(row, showAll);
  const hidden = row.fields.length - fields.length;
  return (
    <div className="rounded-control border border-line">
      <div className="flex flex-wrap items-center gap-2 border-b border-line bg-surface-2 px-3 py-1.5">
        <Pill tone={OP_TONE[row.op]}>{row.opLabel}</Pill>
        <span className="text-[12.5px] font-medium text-fg">{row.tableLabel}</span>
        <span className="font-mono text-[11px] text-dim">{row.rowId}</span>
        <span className="flex-1" />
        {hidden > 0 || showAll ? (
          <button
            type="button"
            className="label-caps text-[10px] hover:text-fg"
            onClick={() => setShowAll((v) => !v)}
          >
            {showAll ? t('agente-so-o-que-muda') : t('agente-mostrar-campos', { n: hidden })}
          </button>
        ) : null}
      </div>
      <table
        className="w-full border-collapse text-left"
        aria-label={`${row.opLabel} · ${row.tableLabel} ${row.rowId}`}
      >
        <thead>
          <tr className="label-caps text-[10px]">
            <th scope="col" className="w-[22%] px-3 py-1.5 font-normal">
              {t('agente-campo')}
            </th>
            {row.op !== 'insert' ? (
              <th scope="col" className="px-3 py-1.5 font-normal">
                {t('agente-antes')}
              </th>
            ) : null}
            {row.op !== 'delete' ? (
              <th scope="col" className="px-3 py-1.5 font-normal">
                {t('agente-depois')}
              </th>
            ) : null}
          </tr>
        </thead>
        <tbody>
          {fields.map((f) => (
            <tr
              key={f.field}
              data-change={f.change}
              className={cx('border-t border-line align-top', f.technical && 'text-dim')}
            >
              <th scope="row" className="px-3 py-1 font-mono text-[11.5px] font-normal text-dim">
                {f.field}
              </th>
              {row.op !== 'insert' ? (
                <td
                  className={cx(
                    'px-3 py-1',
                    f.change === 'changed' &&
                      'bg-[var(--negative-soft)] line-through decoration-negative/60',
                  )}
                >
                  <Value field={f.field} value={f.before} />
                </td>
              ) : null}
              {row.op !== 'delete' ? (
                <td
                  className={cx(
                    'px-3 py-1',
                    (f.change === 'changed' || f.change === 'added') &&
                      row.op === 'update' &&
                      'bg-[var(--positive-soft)]',
                  )}
                >
                  {f.change === 'changed' ? (
                    <span aria-hidden className="mr-1 text-positive">
                      {'→'}
                    </span>
                  ) : null}
                  <Value field={f.field} value={f.after} />
                </td>
              ) : null}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function DiffView({ items }: { items: readonly ChangeItem[] }) {
  const rows = diffItems(items);
  if (rows.length === 0) {
    return <p className="text-[12.5px] text-dim">{t('agente-nenhuma-linha-proposta')}</p>;
  }
  return (
    <div className="flex flex-col gap-2">
      {rows.map((r) => (
        <RowDiff key={r.key} row={r} />
      ))}
    </div>
  );
}
