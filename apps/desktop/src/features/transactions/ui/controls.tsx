/**
 * Controles pequenos usados nas telas do livro-caixa: caixa de seleção, chave,
 * controle segmentado, selos de status/tipo, estado de consulta e confirmação.
 */
import {
  useId,
  type InputHTMLAttributes,
  type ReactNode,
  type TextareaHTMLAttributes,
} from 'react';

import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnStatus } from '@/api/generated/TxnStatus';
import { Button, Dialog, LiveIndicator, useFieldControl } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { errorText } from './errors';
import { TXN_KIND, TXN_STATUS } from './labels';

export function Checkbox({
  checked,
  onChange,
  label,
  hideLabel,
  indeterminate,
  disabled,
  className,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: ReactNode;
  hideLabel?: boolean;
  indeterminate?: boolean;
  disabled?: boolean;
  className?: string;
}) {
  return (
    <label
      className={cx(
        'inline-flex cursor-pointer items-center gap-2 text-[13px] text-fg',
        disabled && 'opacity-50',
        className,
      )}
    >
      <input
        type="checkbox"
        className="size-3.5"
        checked={checked}
        disabled={disabled}
        ref={(el) => {
          if (el) el.indeterminate = Boolean(indeterminate) && !checked;
        }}
        onChange={(e) => onChange(e.target.checked)}
        onClick={(e) => e.stopPropagation()}
      />
      <span className={hideLabel ? 'sr-only' : undefined}>{label}</span>
    </label>
  );
}

export function Switch({
  checked,
  onChange,
  label,
  description,
  disabled,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: ReactNode;
  description?: ReactNode;
  disabled?: boolean;
}) {
  const id = useId();
  return (
    <div className="flex items-start justify-between gap-6">
      <div className="flex min-w-0 flex-col gap-0.5">
        <label htmlFor={id} className="text-[13.5px] text-fg">
          {label}
        </label>
        {description ? <p className="text-[12px] leading-snug text-dim">{description}</p> : null}
      </div>
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={cx(
          'relative mt-0.5 inline-flex h-5 w-9 shrink-0 items-center rounded-full border transition-colors disabled:opacity-45',
          checked ? 'border-accent bg-accent' : 'border-line-strong bg-surface-2',
        )}
      >
        <span
          aria-hidden
          className={cx(
            'inline-block size-3.5 rounded-full transition-transform',
            checked ? 'translate-x-[18px] bg-accent-ink' : 'translate-x-[2px] bg-dim',
          )}
        />
        <span className="sr-only">
          {checked ? t('controles-ligado') : t('controles-desligado')}
        </span>
      </button>
    </div>
  );
}

export interface SegmentOption<V extends string> {
  value: V;
  label: ReactNode;
  disabled?: boolean;
}

/** Grupo de botões de opção única (radiogroup), com setas ←/→. */
export function Segmented<V extends string>({
  value,
  onChange,
  options,
  label,
  size = 'md',
  fill = false,
  className,
}: {
  value: V;
  onChange: (value: V) => void;
  options: readonly SegmentOption<V>[];
  label: string;
  size?: 'sm' | 'md';
  /** Ocupa a largura do campo, na altura dos inputs (formulários). */
  fill?: boolean;
  className?: string;
}) {
  const move = (dir: number) => {
    const enabled = options.filter((o) => !o.disabled);
    const i = enabled.findIndex((o) => o.value === value);
    const next = enabled[(i + dir + enabled.length) % enabled.length];
    if (next) onChange(next.value);
  };
  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={cx(
        'rounded-control border border-line-strong bg-bg p-0.5',
        fill ? 'flex h-9 w-full items-stretch' : 'inline-flex',
        className,
      )}
      onKeyDown={(e) => {
        if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
          e.preventDefault();
          move(1);
        } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
          e.preventDefault();
          move(-1);
        }
      }}
    >
      {options.map((o) => {
        const selected = o.value === value;
        return (
          <button
            key={o.value}
            type="button"
            role="radio"
            aria-checked={selected}
            disabled={o.disabled}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(o.value)}
            className={cx(
              'label-caps rounded-[2px] transition-colors disabled:opacity-40',
              fill ? 'min-w-0 flex-1 truncate px-2' : size === 'sm' ? 'h-6 px-2' : 'h-7 px-2.5',
              selected ? 'bg-accent-soft text-accent' : 'hover:text-fg',
            )}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}

export function StatusBadge({ status }: { status: TxnStatus }) {
  const tone =
    status === 'planned'
      ? 'border-line-strong text-dim border-dashed'
      : status === 'reconciled'
        ? 'border-positive/50 text-positive'
        : 'border-line text-fg';
  return (
    <span
      className={cx('label-caps inline-flex h-5 items-center rounded-control border px-1.5', tone)}
    >
      {status === 'reconciled' ? (
        <span aria-hidden className="mr-1">
          {'✓'}
        </span>
      ) : null}
      {TXN_STATUS[status]}
    </span>
  );
}

/** Selo do tipo; `amount` separa aporte (sai) de resgate (entra: venda ou volta ao banco). */
export function KindBadge({ kind, amount }: { kind: TxnKind; amount?: number }) {
  if (kind === 'income' || kind === 'expense') return null;
  const label =
    kind === 'investment' && amount != null && amount > 0
      ? t('lancamentos-resgate')
      : TXN_KIND[kind];
  return (
    <span className="label-caps inline-flex h-5 shrink-0 items-center rounded-control border border-accent/40 px-1.5 whitespace-nowrap text-accent">
      {label}
    </span>
  );
}

/** Carregando / erro / conteúdo de uma consulta, no estilo do instrumento. */
export function QueryState({
  loading,
  error,
  onRetry,
  label = t('controles-carregando'),
  children,
}: {
  loading: boolean;
  error: unknown;
  onRetry?: () => void;
  label?: string;
  children: ReactNode;
}) {
  if (error) {
    return (
      <div
        role="alert"
        className="flex flex-wrap items-center gap-3 rounded-card border border-negative/50 bg-negative/5 px-4 py-3 text-[13px] text-negative"
      >
        <span>{errorText(error)}</span>
        {onRetry ? (
          <Button size="sm" variant="secondary" onClick={onRetry}>
            {t('controles-tentar-de-novo')}
          </Button>
        ) : null}
      </div>
    );
  }
  if (loading) {
    return (
      <div className="py-6" aria-busy>
        <LiveIndicator label={label} tone="accent" />
      </div>
    );
  }
  return <>{children}</>;
}

export function InlineError({ error }: { error: unknown }) {
  if (!error) return null;
  return (
    <p role="alert" className="flex items-start gap-1.5 text-[12.5px] leading-snug text-negative">
      <span aria-hidden className="num">
        {'×'}
      </span>
      <span>{errorText(error)}</span>
    </p>
  );
}

/** Confirmação de ação irreversível (ou desfazível só pelo histórico). */
export function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmLabel,
  onConfirm,
  pending,
  error,
  tone = 'danger',
  children,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: ReactNode;
  confirmLabel: string;
  onConfirm: () => void;
  pending?: boolean;
  error?: unknown;
  tone?: 'danger' | 'primary';
  children?: ReactNode;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={title}
      description={description}
      size="sm"
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button variant={tone} loading={pending} onClick={onConfirm}>
            {confirmLabel}
          </Button>
        </>
      }
    >
      {children}
      <InlineError error={error} />
    </Dialog>
  );
}

/** Título de seção técnica (`FIG.`/subtítulos dentro de cards). */
export function SectionTitle({ children, aside }: { children: ReactNode; aside?: ReactNode }) {
  return (
    <div className="flex items-baseline justify-between gap-3 border-b border-line pb-2">
      <h3 className="label-caps text-fg">{children}</h3>
      {aside}
    </div>
  );
}

/** `<input type="file">` ligado ao rótulo do `Field` em volta. */
export function FileInput({
  className,
  ...rest
}: Omit<InputHTMLAttributes<HTMLInputElement>, 'type'>) {
  const control = useFieldControl({ id: rest.id });
  return (
    <input
      type="file"
      id={control.id}
      aria-describedby={control['aria-describedby']}
      className={cx(
        'text-[12.5px] text-dim file:mr-3 file:h-8 file:rounded-control file:border file:border-line-strong file:bg-surface-2 file:px-3 file:text-fg',
        className,
      )}
      {...rest}
    />
  );
}

/** Área de texto no estilo dos campos, ligada ao rótulo do `Field` em volta. */
export function TextArea({ className, ...rest }: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  const control = useFieldControl({ id: rest.id });
  return (
    <textarea
      id={control.id}
      aria-describedby={control['aria-describedby']}
      className={cx(
        'min-h-9 rounded-control border border-line-strong bg-bg px-2.5 py-1.5 text-[13.5px] text-fg outline-none focus:border-accent',
        className,
      )}
      {...rest}
    />
  );
}
