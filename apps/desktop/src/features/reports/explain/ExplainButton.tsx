import type { ReactNode } from 'react';

import { Icon, Kbd, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { useExplainStore, type ExplainTarget } from './store';

/**
 * Número que pode ser explicado: é um botão (Enter, Espaço ou clique abrem o painel com a
 * fórmula e as fontes). O ícone de retícula indica que a medida tem origem rastreável.
 */
export function Explainable({
  target,
  children,
  className,
}: {
  target: ExplainTarget;
  children: ReactNode;
  className?: string;
}) {
  const open = useExplainStore((s) => s.open);
  return (
    <Tooltip
      side="top"
      content={
        <span className="flex items-center gap-2">
          {t('relatorios-explicar-dica', { titulo: target.title })}
          <Kbd keys={['Enter']} />
        </span>
      }
    >
      <button
        type="button"
        aria-label={t('relatorios-explicar', { titulo: target.title })}
        aria-haspopup="dialog"
        onClick={() => open(target)}
        className={cx(
          'group inline-flex items-baseline gap-1.5 rounded-control text-left decoration-dotted underline-offset-4 hover:underline focus-visible:outline-2 focus-visible:outline-accent',
          className,
        )}
      >
        {children}
        <Icon
          name="reticle"
          size={12}
          className="shrink-0 self-center text-dim/60 transition-colors group-hover:text-accent"
        />
      </button>
    </Tooltip>
  );
}
