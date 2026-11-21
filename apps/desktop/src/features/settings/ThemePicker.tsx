import { useRef, type KeyboardEvent } from 'react';

import { categoricalPalette } from '@/charts/palette';
import { Icon, ScannerSweep } from '@/design/components';
import { cx } from '@/design/cx';
import { THEME_FAMILIES, THEMES, type ThemeId } from '@/design/themes';
import { t } from '@/i18n';

/**
 * Miniatura do tema com os tokens reais: o `data-theme` no contêiner aplica o tema
 * só ali dentro (fundo, card, medida, destaque e as primeiras cores dos gráficos).
 */
export function ThemePreview({ id }: { id: ThemeId }) {
  const series = categoricalPalette(id).slice(0, 5);
  return (
    <div
      aria-hidden
      data-theme={id}
      className="relative isolate flex h-[88px] overflow-hidden rounded-control border border-line bg-bg [background-image:var(--canvas-art)] bg-cover"
    >
      <div className="chrome flex w-[22%] flex-col gap-1.5 border-r border-line px-1.5 py-2">
        <span className="h-1 w-3/4 rounded-full bg-accent" />
        <span className="h-1 w-full rounded-full bg-line-strong/60" />
        <span className="h-1 w-2/3 rounded-full bg-line-strong/60" />
        <span className="h-1 w-5/6 rounded-full bg-line-strong/60" />
      </div>
      <div className="flex min-w-0 flex-1 flex-col p-2">
        <div className="panel flex min-w-0 flex-1 flex-col gap-1.5 rounded-[6px] border border-panel px-2 py-1.5">
          <span className="flex items-center gap-[3px]">
            <span className="size-[4px] rounded-full bg-line-strong/80" />
            <span className="size-[4px] rounded-full bg-line-strong/55" />
            <span className="size-[4px] rounded-full bg-line-strong/35" />
            <span className="ml-1 h-[3px] w-6 rounded-full bg-dim/60" />
          </span>
          <span className="num truncate text-[12px] leading-none font-medium text-fg">
            12.480<span className="text-dim">,00</span>
          </span>
          <span className="flex items-end gap-[3px]">
            {series.map((color, i) => (
              <span
                key={color}
                className="w-full rounded-t-[2px]"
                style={{ background: color, height: `${[10, 16, 8, 13, 6][i]}px` }}
              />
            ))}
          </span>
        </div>
      </div>
      <ScannerSweep className="scanner-sweep-preview" />
    </div>
  );
}

export interface ThemePickerProps {
  value: ThemeId;
  onChange: (id: ThemeId) => void;
}

/** Seletor de tema: miniaturas agrupadas (clássicos, especiais, divertidos); setas navegam. */
export function ThemePicker({ value, onChange }: ThemePickerProps) {
  const groupRef = useRef<HTMLDivElement>(null);
  const move = (event: KeyboardEvent, dir: number) => {
    event.preventDefault();
    const i = THEMES.findIndex((theme) => theme.id === value);
    const next = THEMES[(i + dir + THEMES.length) % THEMES.length]!;
    onChange(next.id);
    groupRef.current?.querySelector<HTMLElement>(`[data-theme-option='${next.id}']`)?.focus();
  };
  return (
    <div
      ref={groupRef}
      role="radiogroup"
      aria-label={t('ajustes-pref-tema')}
      className="flex flex-col gap-4"
      onKeyDown={(event) => {
        if (event.key === 'ArrowRight' || event.key === 'ArrowDown') move(event, 1);
        else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') move(event, -1);
      }}
    >
      {THEME_FAMILIES.map((family) => (
        <div key={family.id} className="flex flex-col gap-2">
          <p className="label-caps text-[10px]" aria-hidden>
            {family.label}
          </p>
          <div className="grid grid-cols-2 gap-2.5 @min-[400px]:grid-cols-3 @min-[790px]:grid-cols-4">
            {THEMES.filter((theme) => theme.family === family.id).map((theme) => {
              const selected = value === theme.id;
              return (
                <button
                  key={theme.id}
                  type="button"
                  role="radio"
                  aria-checked={selected}
                  aria-label={theme.label}
                  title={theme.description}
                  tabIndex={selected ? 0 : -1}
                  data-theme-option={theme.id}
                  onClick={() => onChange(theme.id)}
                  className={cx(
                    'group flex flex-col gap-2 rounded-card border p-1.5 pb-2 text-left transition-[border-color,box-shadow,background-color] duration-150',
                    selected
                      ? 'border-accent bg-accent-faint shadow-[0_0_0_3px_var(--focus-ring)]'
                      : 'border-line hover:border-line-strong hover:bg-surface-2',
                  )}
                >
                  <ThemePreview id={theme.id} />
                  <span className="flex items-center justify-between gap-2 px-1">
                    <span
                      className={cx(
                        'truncate text-[12.5px] font-medium',
                        selected ? 'text-accent' : 'text-fg',
                      )}
                    >
                      {theme.label}
                    </span>
                    {selected ? <Icon name="check" size={13} className="text-accent" /> : null}
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}
