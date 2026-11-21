/** Mock de `settings.*` e `app.info`: ajustes gerais guardados no cofre. */
import type { AppSettings } from '@/api/generated/AppSettings';

import { MOCK_DB_PATH } from '../mockFixtures';
import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { ready } from './ledger/boot';
import { invalid } from './ledger/errors';
import { invoke } from './ledger/investImport';

const DEFAULTS: AppSettings = {
  auto_lock_minutes: 15,
  budget_basis: 'accrual',
  flow_basis: 'cash',
  close_to_tray: false,
  locale: 'pt-BR',
  developer_tools: false,
  watch_downloads: false,
  downloads_delete_after_import: false,
  lock_on_sleep: true,
  allow_screen_capture: false,
  twr_daily: false,
  watch_inbox: true,
  inbox_folder: '',
};

let current: AppSettings = { ...DEFAULTS };
onMockReset(() => {
  current = { ...DEFAULTS };
});

registerMockHandlers({
  'settings.get': () => {
    ready();
    return current;
  },
  'settings.set': (args) => {
    ready();
    const next = { ...current } as Record<string, unknown>;
    for (const [k, v] of Object.entries(args)) {
      if (!(k in next)) invalid(`ajuste desconhecido: ${k}`);
      next[k] = v;
    }
    const s = next as unknown as AppSettings;
    if (typeof s.auto_lock_minutes !== 'number' || !Number.isInteger(s.auto_lock_minutes)) {
      invalid('invalid type: expected u64');
    }
    if (s.auto_lock_minutes < 0 || s.auto_lock_minutes > 1440) {
      invalid('bloqueio automático deve ficar entre 0 (desligado) e 1440 minutos');
    }
    if (s.budget_basis !== 'accrual' && s.budget_basis !== 'cash')
      invalid(`unknown variant \`${String(s.budget_basis)}\``);
    current = s;
    // Como no núcleo: o loopback sem chave do MCP sai junto com o modo desenvolvedor.
    if (!s.developer_tools) {
      const mcp = invoke<Record<string, unknown>>('mcp.server_settings');
      invoke('mcp.set_server_settings', { ...mcp, http_loopback_no_auth: false });
    }
    return current;
  },
});

/** `app.info` (versão e pastas do app), mostrado em Ajustes. */
registerMockHandlers({
  'app.info': () => ({
    version: '1.0.0',
    schema_version: 51,
    db_path: MOCK_DB_PATH,
    backups_dir: '/Users/ana/.treasury/backups',
    data_root: '/Users/ana/.treasury',
    platform: 'macos-aarch64',
  }),
});
