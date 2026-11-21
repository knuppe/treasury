import * as RTabs from '@radix-ui/react-tabs';
import type { ComponentPropsWithoutRef } from 'react';

import { cx } from '../cx';

/** Abas (Radix Tabs): setas ←/→ trocam de aba, Tab entra no conteúdo. */
export function Tabs({ className, ...props }: ComponentPropsWithoutRef<typeof RTabs.Root>) {
  return <RTabs.Root className={cx('flex flex-col', className)} {...props} />;
}

/**
 * Trilha das abas: borda e fundo próprios, para se ler como abas em qualquer tema (`.tab-list`
 * em styles.css, tokens `--tab-*`). Sem espaço, as abas descem para uma segunda linha (nunca
 * quebram o rótulo nem somem).
 */
export function TabsList({ className, ...props }: ComponentPropsWithoutRef<typeof RTabs.List>) {
  return <RTabs.List className={cx('tab-list', className)} {...props} />;
}

/** Aba: a ativa ganha o fundo e o texto do destaque e uma barra embaixo (`.tab-trigger`). */
export function TabsTrigger({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof RTabs.Trigger>) {
  return <RTabs.Trigger className={cx('tab-trigger', className)} {...props} />;
}

export function TabsContent({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof RTabs.Content>) {
  return <RTabs.Content className={cx('pt-4', className)} {...props} />;
}
