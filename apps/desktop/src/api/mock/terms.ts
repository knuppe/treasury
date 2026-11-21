/**
 * Mock de `terms.*` (docs/terms.md): vale com o cofre bloqueado e, por padrão, os termos já
 * aceitos (as telas de teste não passam pela porta). `seedTermsPending` mostra a tela.
 */
import type { TermsStatus } from '@/api/generated/TermsStatus';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { conflict } from './ledger/errors';

const CURRENT = '1.0';
let accepted: TermsStatus['accepted'] = { version: CURRENT, accepted_at: '2026-01-01T00:00:00Z' };

onMockReset(() => {
  accepted = { version: CURRENT, accepted_at: '2026-01-01T00:00:00Z' };
});

/** Testes: a instalação ainda não aceitou os termos. */
export function seedTermsPending(): void {
  accepted = null;
}

const status = (): TermsStatus => ({
  current: CURRENT,
  accepted,
  needs_acceptance: accepted == null,
  updated: false,
  changes: [],
  history: [{ version: CURRENT, summary: 'Primeira versão dos termos.' }],
});

registerMockHandlers({
  'terms.status': () => status(),
  'terms.accept': (args) => {
    if (args.version !== CURRENT)
      conflict(`os termos mudaram (vigente: ${CURRENT}); leia de novo antes de aceitar`);
    accepted = { version: CURRENT, accepted_at: new Date().toISOString() };
    return status();
  },
});
