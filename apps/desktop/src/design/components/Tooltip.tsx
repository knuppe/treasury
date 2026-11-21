import * as RTooltip from '@radix-ui/react-tooltip';
import type { ReactElement, ReactNode } from 'react';

import { cx } from '../cx';

export interface TooltipProps {
  content: ReactNode;
  /** Elemento focável que dispara a dica (botão). Recebe os props via `asChild`. */
  children: ReactElement;
  side?: 'top' | 'right' | 'bottom' | 'left';
  align?: 'start' | 'center' | 'end';
  delay?: number;
  className?: string;
}

/**
 * Dica flutuante (Radix Tooltip): abre ao passar o mouse e ao receber foco do
 * teclado. Traz o próprio Provider para funcionar em qualquer lugar.
 */
export function Tooltip({
  content,
  children,
  side = 'bottom',
  align = 'center',
  delay = 350,
  className,
}: TooltipProps) {
  return (
    <RTooltip.Provider delayDuration={delay} skipDelayDuration={200}>
      <RTooltip.Root>
        <RTooltip.Trigger asChild>{children}</RTooltip.Trigger>
        <RTooltip.Portal>
          <RTooltip.Content
            side={side}
            align={align}
            sideOffset={6}
            collisionPadding={8}
            className={cx(
              'z-[70] max-w-xs animate-fade-in rounded-control border border-panel pop-surface px-2.5 py-1.5 text-[12px] text-fg',
              className,
            )}
          >
            {content}
          </RTooltip.Content>
        </RTooltip.Portal>
      </RTooltip.Root>
    </RTooltip.Provider>
  );
}
