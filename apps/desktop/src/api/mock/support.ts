/**
 * Mock de `support.*` (docs/support.md): canais ainda como "em breve" (placeholders) e o
 * lembrete com as mesmas regras do núcleo, simplificadas. `seedSupportPrompt` deixa um cartão
 * pendente para testar a Visão geral.
 */
import type { SupportChoice } from '@/api/generated/SupportChoice';
import type { SupportInfo } from '@/api/generated/SupportInfo';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { fail } from './ledger/errors';

interface State {
  pending: boolean;
  optedOut: boolean;
  supporter: boolean;
  /** Regras liberam o cartão no próximo evento (no mock, só por `seedSupportPrompt`). */
  eligible: boolean;
}

let state: State = { pending: false, optedOut: false, supporter: false, eligible: false };

onMockReset(() => {
  state = { pending: false, optedOut: false, supporter: false, eligible: false };
});

/** Testes: o próximo evento de valor libera o cartão. */
export function seedSupportEligible(): void {
  state.eligible = true;
}

registerMockHandlers(
  guarded({
    'support.info': (): SupportInfo => ({
      channels: {
        github_sponsors_url: null,
        repository_url: null,
        signature: 'Knuppe',
        annual_summary: null,
      },
      opted_out: state.optedOut,
      supporter: state.supporter,
    }),
    'support.event': (args) => {
      if (!['budget_month', 'import', 'tax_report'].includes(String(args.event)))
        fail('invalid', `evento desconhecido: ${String(args.event)}`);
      if (state.eligible && !state.optedOut && !args.view_as) state.pending = true;
      return { pending: state.pending };
    },
    'support.prompt': (args) => ({
      variant:
        state.pending && !state.optedOut && !args.view_as
          ? state.supporter
            ? 'thanks'
            : 'ask'
          : null,
    }),
    'support.seen': () => null,
    'support.respond': (args) => {
      const c = args.choice as SupportChoice;
      state.pending = false;
      state.eligible = false;
      if (c === 'supporter') state.supporter = true;
      if (c === 'never') state.optedOut = true;
      return null;
    },
    'support.set_opted_out': (args) => {
      state.optedOut = Boolean(args.opted_out);
      return null;
    },
  }),
);
