import { Money, Tooltip } from '@/design/components';
import { formatDate, formatMonthShort } from '@/lib/format';
import { t } from '@/i18n';

import { monthTicks, positionIn } from './timeline';

export interface TimelineRow {
  id: string;
  label: string;
  /** Linha secundária ("todo mês no dia 10 · Itaú"). */
  detail: string;
  /** Valor de cada ocorrência (com sinal). */
  amount: number;
  /** Equivalente mensal (com sinal). */
  monthly: number;
  currency: string;
  color: string;
  dates: readonly string[];
}

export interface RecurrenceTimelineProps {
  rows: readonly TimelineRow[];
  from: string;
  to: string;
  today: string;
  label: string;
  onOccurrenceClick?: (row: TimelineRow, date: string) => void;
}

/**
 * Linha do tempo das recorrências: uma linha por assinatura/conta fixa, um ponto (≥ 8 px,
 * anel na cor da superfície) por ocorrência na janela, régua de meses e marca de hoje.
 * Cada ponto é um botão: foco e mouse mostram o mesmo tooltip; Enter abre os lançamentos.
 */
export function RecurrenceTimeline({
  rows,
  from,
  to,
  today,
  label,
  onOccurrenceClick,
}: RecurrenceTimelineProps) {
  const ticks = monthTicks(from, to);
  const todayPos = positionIn(today, from, to);
  return (
    <div role="group" aria-label={label} className="flex flex-col">
      <div className="grid grid-cols-[minmax(160px,240px)_1fr_120px] items-end gap-x-4 border-b border-line pb-1.5">
        <span className="label-caps">{t('graficos-recorrencia')}</span>
        <div className="relative h-4">
          {ticks.map((d) => (
            <span
              key={d}
              className="label-caps absolute -translate-x-1/2 text-[10px]"
              style={{ left: `${positionIn(d, from, to)}%` }}
            >
              {formatMonthShort(d)}
            </span>
          ))}
        </div>
        <span className="label-caps text-right">{t('graficos-por-mes')}</span>
      </div>
      <ul className="flex flex-col">
        {rows.map((r) => (
          <li
            key={r.id}
            className="grid grid-cols-[minmax(160px,240px)_1fr_120px] items-center gap-x-4 border-b border-line/60 py-1.5"
          >
            <div className="flex min-w-0 items-center gap-2">
              <span
                aria-hidden
                className="size-2 shrink-0 rounded-full"
                style={{ background: r.color }}
              />
              <div className="flex min-w-0 flex-col">
                <span className="truncate text-[13px] text-fg">{r.label}</span>
                <span className="truncate text-[11.5px] text-dim">{r.detail}</span>
              </div>
            </div>
            <div className="relative h-6">
              <span aria-hidden className="absolute inset-x-0 top-1/2 h-px bg-line" />
              {ticks.map((d) => (
                <span
                  key={d}
                  aria-hidden
                  className="absolute inset-y-0 w-px bg-line"
                  style={{ left: `${positionIn(d, from, to)}%` }}
                />
              ))}
              <span
                aria-hidden
                className="absolute -inset-y-1 w-px bg-accent/70"
                style={{ left: `${todayPos}%` }}
              />
              {r.dates.map((d) => (
                <Tooltip
                  key={d}
                  side="top"
                  content={
                    <div className="flex flex-col gap-0.5">
                      <span className="label-caps text-fg">{r.label}</span>
                      <span className="num text-[12px]">{formatDate(d)}</span>
                      <Money
                        cents={r.amount}
                        currency={r.currency}
                        tone="auto"
                        className="text-[12.5px]"
                      />
                    </div>
                  }
                >
                  <button
                    type="button"
                    aria-label={`${r.label}, ${formatDate(d)}`}
                    onClick={() => onOccurrenceClick?.(r, d)}
                    className="absolute top-1/2 grid size-6 -translate-x-1/2 -translate-y-1/2 place-items-center rounded-full focus-visible:outline-2 focus-visible:outline-accent"
                    style={{ left: `${positionIn(d, from, to)}%` }}
                  >
                    <span
                      className="block size-2.5 rounded-full"
                      style={{ background: r.color, boxShadow: '0 0 0 2px var(--surface)' }}
                    />
                  </button>
                </Tooltip>
              ))}
            </div>
            <Money cents={r.monthly} currency={r.currency} className="text-right text-[13px]" />
          </li>
        ))}
      </ul>
    </div>
  );
}
