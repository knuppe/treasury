import { useEffect, useEffectEvent } from 'react';
import { useLocation, useNavigate } from 'react-router';

import { useCommandBar } from '@/commands/registry';
import { hasModKey } from '@/lib/platform';
import { useCompanion } from '@/state/companion';
import { useUi } from '@/state/ui';

import { isAgentScreen, SCREENS } from './screens';
import { useVisibleScreens } from './useVisibleScreens';

/**
 * Atalhos globais do shell:
 * - ⌘K / Ctrl+K: barra de comando
 * - ⌘J / Ctrl+J: abre e fecha o agente companheiro (painel lateral); na tela do Agente, que
 *   não mostra o companheiro, leva o foco à caixa da conversa
 * - ⌘⇧H / Ctrl+Shift+H: modo privacidade
 * - ⌘\ / Ctrl+\: recolhe e expande o menu lateral
 * - ⌘⇧L / Ctrl+Shift+L: bloquear o cofre
 * - ⌘1…⌘9, ⌘0: telas 01…10 do menu (as seguintes sem atalho); uma tela que o país da família
 *   não tem (ex.: Mercado fora do Brasil) não abre pelo atalho
 */
export function useGlobalHotkeys({ lock }: { lock: () => void }): void {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const visible = useVisibleScreens();

  const onKeyDown = useEffectEvent((event: KeyboardEvent) => {
    if (!hasModKey(event) || event.altKey || event.isComposing) return;
    const key = event.key.toLowerCase();

    if (!event.shiftKey && key === 'k') {
      event.preventDefault();
      useCommandBar.getState().toggle();
      return;
    }
    if (!event.shiftKey && key === 'j') {
      event.preventDefault();
      useCommandBar.getState().setOpen(false);
      // Na tela do Agente o companheiro não aparece: o atalho vai para a caixa da conversa.
      if (isAgentScreen(pathname)) {
        document.getElementById('agent-message')?.focus();
        return;
      }
      useCompanion.getState().toggle();
      return;
    }
    if (event.shiftKey && key === 'h') {
      event.preventDefault();
      useUi.getState().togglePrivacy();
      return;
    }
    if (!event.shiftKey && (key === '\\' || event.code === 'Backslash')) {
      event.preventDefault();
      useUi.getState().toggleSidebar();
      return;
    }
    if (event.shiftKey && key === 'l') {
      event.preventDefault();
      useCommandBar.getState().setOpen(false);
      lock();
      return;
    }
    const digit = /^Digit(\d)$/.exec(event.code)?.[1];
    if (!event.shiftKey && digit !== undefined) {
      const screen = SCREENS[(Number(digit) + 9) % 10];
      // Só as telas do menu: as escondidas pelo país (ou pelo modo desenvolvedor) ficam de fora.
      if (!screen || !visible.some((s) => s.id === screen.id)) return;
      event.preventDefault();
      useCommandBar.getState().setOpen(false);
      void navigate(screen.path);
    }
  });

  useEffect(() => {
    const handler = (event: KeyboardEvent) => onKeyDown(event);
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);
}
