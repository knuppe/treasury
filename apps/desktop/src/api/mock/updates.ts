/**
 * Mock de `system.update_settings` e `system.set_update_settings` (docs/auto-update.md): o
 * modo da atualização, que no núcleo fica no `config.toml` e vale sem cofre.
 */
import type { UpdateSettings } from '@/api/generated/UpdateSettings';

import { onMockReset, registerMockHandlers } from '../mockRegistry';

let settings: UpdateSettings = { mode: 'download' };

onMockReset(() => {
  settings = { mode: 'download' };
});

registerMockHandlers({
  'system.update_settings': () => ({ ...settings }),
  'system.set_update_settings': (args) => {
    const mode = args.mode;
    if (mode !== 'download' && mode !== 'notify' && mode !== 'off')
      throw new Error(`modo inválido: ${String(mode)}`);
    settings = { mode };
    return { ...settings };
  },
});
