/**
 * Troca do motor no chat (painel ⌘J): com mais de um motor pronto na rota de chat, o nome do
 * motor no cabeçalho vira um menu para escolher outro nesta conversa. Vale a partir da próxima
 * mensagem e fica na sessão (`state/agentSession`); o padrão (a rota em Ajustes › Agente) não
 * muda. "Automático" volta à ordem da rota.
 */
import { useEffect, useRef, useState } from 'react';

import type { Engine } from '@/api/generated/Engine';
import { Icon } from '@/design/components';
import { cx } from '@/design/cx';
import { useT } from '@/i18n';

import { ENGINE_KIND_LABEL } from './format';

export function EnginePicker({
  engines,
  value,
  current,
  onChange,
  disabled,
}: {
  /** Motores prontos da rota de chat, na ordem da rota. */
  engines: readonly Engine[];
  /** Motor escolhido nesta conversa (`null` = automático). */
  value: string | null;
  /** O motor que vai receber a próxima mensagem. */
  current: Engine | null;
  onChange: (engineId: string | null) => void;
  disabled?: boolean;
}) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (root.current && !root.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', onDown);
    return () => document.removeEventListener('mousedown', onDown);
  }, [open]);
  const pick = (id: string | null) => {
    onChange(id);
    setOpen(false);
    button.current?.focus();
  };
  const item = (id: string | null, label: string, hint: string) => (
    <button
      key={id ?? 'auto'}
      type="button"
      role="menuitemradio"
      aria-checked={value === id}
      onClick={() => pick(id)}
      className="flex w-full items-center gap-2 rounded-control px-2 py-1.5 text-left text-[12.5px] text-fg hover:bg-surface-2"
    >
      <span className="w-3 shrink-0 text-accent">{value === id ? '✓' : ''}</span>
      <span className="min-w-0 flex-1 truncate">{label}</span>
      <span className="shrink-0 text-[11px] text-dim">{hint}</span>
    </button>
  );
  return (
    <div
      ref={root}
      className="relative min-w-0"
      onKeyDown={(e) => {
        if (e.key === 'Escape' && open) {
          e.preventDefault();
          e.stopPropagation();
          setOpen(false);
          button.current?.focus();
        }
      }}
    >
      <button
        ref={button}
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={t('seletor-motor-rotulo')}
        title={t('seletor-motor-dica')}
        disabled={disabled}
        onClick={() => setOpen((o) => !o)}
        className="flex min-w-0 items-center gap-1.5 rounded-sm text-[11.5px] text-dim outline-none hover:text-fg focus-visible:text-fg disabled:opacity-60"
      >
        <span
          aria-hidden
          className={cx(
            'inline-block size-1.5 shrink-0 rounded-full',
            current ? 'bg-positive' : 'bg-warning',
          )}
        />
        {current ? (
          <>
            <span className="truncate text-fg">{current.name}</span>
            <span className="shrink-0">{`· ${ENGINE_KIND_LABEL[current.kind]}`}</span>
          </>
        ) : (
          <span className="truncate">{t('seletor-motor-automatico')}</span>
        )}
        <Icon name="chevronDown" size={11} className="shrink-0" />
      </button>
      {open ? (
        <div
          role="menu"
          aria-label={t('seletor-motor-rotulo')}
          className="absolute top-full left-0 z-30 mt-1 flex w-72 flex-col gap-0.5 rounded-card border border-line bg-surface p-1 shadow-lg"
        >
          {item(null, t('seletor-motor-automatico'), t('seletor-motor-rota'))}
          {engines.map((e) =>
            item(e.id, e.model ? `${e.name} · ${e.model}` : e.name, ENGINE_KIND_LABEL[e.kind]),
          )}
          <p className="px-2 pt-1 pb-0.5 text-[11px] leading-snug text-dim">
            {t('seletor-motor-dica')}
          </p>
        </div>
      ) : null}
    </div>
  );
}
