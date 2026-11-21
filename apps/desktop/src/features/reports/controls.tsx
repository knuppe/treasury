/**
 * Controles compartilhados do Painel, do Orçamento e dos Relatórios: seletor de mês,
 * competência × caixa (botões de rádio) e estados de consulta (carregando, erro, vazio).
 */
import { useId, useRef, type KeyboardEvent, type ReactNode } from 'react';

import type { Basis } from '@/api/generated/Basis';
import { Button, EmptyState, Icon, IconButton, LiveIndicator } from '@/design/components';
import { cx } from '@/design/cx';
import { formatMonthLong } from '@/lib/format';
import { shiftMonth } from '@/lib/months';
import { t } from '@/i18n';

export interface SegmentedOption<T extends string> {
  value: T;
  label: ReactNode;
  hint?: string;
}

/** Grupo de opções exclusivas (rádio): ←/→ trocam, Tab entra e sai. */
export function Segmented<T extends string>({
  value,
  onChange,
  options,
  label,
  className,
}: {
  value: T;
  onChange: (v: T) => void;
  options: readonly SegmentedOption<T>[];
  label: string;
  className?: string;
}) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const onKey = (e: KeyboardEvent, i: number) => {
    const d =
      e.key === 'ArrowRight' || e.key === 'ArrowDown'
        ? 1
        : e.key === 'ArrowLeft' || e.key === 'ArrowUp'
          ? -1
          : 0;
    if (!d) return;
    e.preventDefault();
    const next = (i + d + options.length) % options.length;
    onChange(options[next]!.value);
    refs.current[next]?.focus();
  };
  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={cx(
        'inline-flex h-9 items-center rounded-control border border-line-strong bg-bg p-0.5',
        className,
      )}
    >
      {options.map((o, i) => {
        const checked = o.value === value;
        return (
          <button
            key={o.value}
            ref={(el) => {
              refs.current[i] = el;
            }}
            type="button"
            role="radio"
            aria-checked={checked}
            tabIndex={checked ? 0 : -1}
            title={o.hint}
            onClick={() => onChange(o.value)}
            onKeyDown={(e) => onKey(e, i)}
            className={cx(
              'label-caps h-full rounded-[2px] px-2.5 transition-colors',
              checked ? 'bg-accent-soft text-accent' : 'hover:text-fg',
            )}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}

export const BASIS_OPTIONS: readonly SegmentedOption<Basis>[] = [
  {
    value: 'accrual',
    get label() {
      return t('relatorios-competencia');
    },
    get hint() {
      return t('relatorios-competencia-dica');
    },
  },
  {
    value: 'cash',
    get label() {
      return t('relatorios-caixa');
    },
    get hint() {
      return t('relatorios-caixa-dica');
    },
  },
];

export function BasisToggle({ value, onChange }: { value: Basis; onChange: (b: Basis) => void }) {
  return (
    <Segmented
      label={t('relatorios-base-datas')}
      value={value}
      onChange={onChange}
      options={BASIS_OPTIONS}
    />
  );
}

export const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/** Mês com setas (← →) e o nome por extenso; `max` limita o avanço. */
export function MonthPicker({
  month,
  onChange,
  max,
}: {
  month: string;
  onChange: (m: string) => void;
  max?: string;
}) {
  const id = useId();
  const atMax = max != null && month >= max;
  return (
    <div className="inline-flex items-center gap-1" role="group" aria-labelledby={id}>
      <IconButton
        size="sm"
        variant="secondary"
        label={t('relatorios-mes-anterior')}
        icon={<Icon name="chevronLeft" size={14} />}
        onClick={() => onChange(shiftMonth(month, -1))}
      />
      <span id={id} className="num min-w-36 text-center text-[13px] text-fg" aria-live="polite">
        {capitalize(formatMonthLong(month))}
      </span>
      <IconButton
        size="sm"
        variant="secondary"
        label={t('relatorios-proximo-mes')}
        icon={<Icon name="chevronRight" size={14} />}
        disabled={atMax}
        onClick={() => onChange(shiftMonth(month, 1))}
      />
    </div>
  );
}

/** Carregando: indicador discreto (sem esqueleto que pisca). */
export function Loading({
  label = t('controles-carregando'),
  className,
}: {
  label?: string;
  className?: string;
}) {
  return (
    <div className={cx('flex min-h-24 items-center justify-center', className)}>
      <LiveIndicator label={label} tone="accent" />
    </div>
  );
}

/** Erro de consulta com a mensagem do núcleo e "tentar de novo". */
export function QueryError({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  const message = error instanceof Error ? error.message : t('relatorios-falha-carregar');
  return (
    <div
      role="alert"
      className="flex flex-col items-start gap-2 rounded-card border border-negative/50 bg-negative-soft px-4 py-3"
    >
      <span className="label-caps text-negative">{t('relatorios-erro-carregar')}</span>
      <p className="text-[13px] text-fg">{message}</p>
      {onRetry ? (
        <Button size="sm" onClick={onRetry} icon={<Icon name="refresh" size={13} />}>
          {t('relatorios-tentar-de-novo')}
        </Button>
      ) : null}
    </div>
  );
}

interface QueryLike<T> {
  data: T | undefined;
  error: unknown;
  isPending: boolean;
  isPlaceholderData?: boolean;
  refetch: () => unknown;
}

/**
 * Estados de uma consulta: carregando, erro, vazio (`isEmpty`) ou o conteúdo. Ao recarregar
 * com dados anteriores, mantém o quadro com opacidade reduzida (sem salto de layout).
 */
export function QueryView<T>({
  query,
  isEmpty,
  empty,
  children,
  loadingLabel,
}: {
  query: QueryLike<T>;
  isEmpty?: (d: T) => boolean;
  empty?: ReactNode;
  children: (d: T) => ReactNode;
  loadingLabel?: string;
}) {
  if (query.error && !query.data)
    return <QueryError error={query.error} onRetry={() => void query.refetch()} />;
  if (query.isPending || query.data === undefined) return <Loading label={loadingLabel} />;
  if (isEmpty?.(query.data))
    return (
      <>
        {empty ?? (
          <EmptyState
            title={t('relatorios-sem-dados-periodo')}
            kicker={t('componentes-sem-dados')}
          />
        )}
      </>
    );
  return (
    <div
      aria-busy={query.isPlaceholderData || undefined}
      className={cx('transition-opacity', query.isPlaceholderData && 'opacity-60')}
    >
      {children(query.data)}
    </div>
  );
}
