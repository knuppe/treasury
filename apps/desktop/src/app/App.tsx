import { QueryClientProvider, type QueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import { RouterProvider } from 'react-router/dom';

import { watchVaultUnlock } from '@/api/vaultQueries';
import { Toaster } from '@/design/components';
import { I18nProvider } from '@/i18n';
import { updateCapture } from '@/lib/osGuard';
import { applyPrivacyOnUnlock, syncThemeWithDocument, usePrivacy } from '@/state/ui';

import type { createAppRouter } from './router';

export interface AppProps {
  queryClient: QueryClient;
  router: ReturnType<typeof createAppRouter>;
}

export function App({ queryClient, router }: AppProps) {
  // Mantém data-theme no <html> igual à preferência (main.tsx já aplica antes do render).
  useEffect(() => syncThemeWithDocument(), []);
  // Ao abrir o cofre, os valores começam escondidos (Ajustes › Preferências desliga).
  useEffect(() => watchVaultUnlock(queryClient, applyPrivacyOnUnlock), [queryClient]);
  // Com o modo privacidade, a janela também some das capturas de tela (macOS e Windows).
  const privacy = usePrivacy();
  useEffect(() => updateCapture({ privacy }), [privacy]);
  return (
    <QueryClientProvider client={queryClient}>
      <I18nProvider>
        <RouterProvider router={router} />
        <Toaster />
      </I18nProvider>
    </QueryClientProvider>
  );
}
