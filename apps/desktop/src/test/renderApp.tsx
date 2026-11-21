import { render } from '@testing-library/react';

import { resetMock, type MockOptions, type MockSeed } from '@/api/mock';
import { createQueryClient } from '@/api/queryClient';
import { App } from '@/app/App';
import { createAppRouter } from '@/app/router';
import { useCommandBar } from '@/commands/registry';
import { useToastStore } from '@/design/components';
import { usePageContextStore } from '@/features/companion/pageContext';
import { useAgentSession } from '@/state/agentSession';
import { COMPANION_DEFAULT_WIDTH, useCompanion } from '@/state/companion';
import { setBaseCurrency } from '@/state/currency';
import { UI_DEFAULTS, useUi } from '@/state/ui';

/** Renderiza o app inteiro contra o núcleo falso, a partir de um estado do cofre. */
export function renderApp(seed: MockSeed = 'empty', options: MockOptions = {}, hash = '') {
  resetMock(seed, options);
  setBaseCurrency('BRL');
  useUi.setState({ ...UI_DEFAULTS });
  useCommandBar.setState({ open: false });
  useCompanion.setState({
    open: false,
    width: COMPANION_DEFAULT_WIDTH,
    shareContext: true,
    conversationId: null,
    draft: null,
  });
  usePageContextStore.setState({ stack: [] });
  useAgentSession.getState().reset();
  useToastStore.getState().clear();
  window.location.hash = hash;
  const queryClient = createQueryClient();
  const router = createAppRouter();
  const result = render(<App queryClient={queryClient} router={router} />);
  return { ...result, queryClient, router };
}
