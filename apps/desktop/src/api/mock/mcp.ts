/**
 * Mock de `mcp.*`: configuração para clientes externos. A auditoria das chamadas
 * (`mcp.audit_list`) fica no mock da IA, junto com o histórico de ferramentas do agente.
 */
import type { McpApiKey } from '../generated/McpApiKey';
import type { McpKeyCreated } from '../generated/McpKeyCreated';
import type { McpScope } from '../generated/McpScope';
import type { McpServerSettings } from '../generated/McpServerSettings';
import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { ready } from './ledger/boot';
import { invalid, notFound } from './ledger/errors';
import { invoke } from './ledger/investImport';

/** Como no núcleo: o loopback sem chave só vale (e só liga) no modo desenvolvedor. */
const developer = () => invoke<{ developer_tools: boolean }>('settings.get').developer_tools;

let keys: McpApiKey[] = [];
let settings: McpServerSettings = {
  require_api_key: true,
  http_enabled: false,
  http_loopback_no_auth: false,
  http_port: 7655,
  allow_writes: false,
  require_key_notice: false,
  cli_when_closed: false,
  new_clients: [],
};
let counter = 0;
onMockReset(() => {
  keys = [];
  counter = 0;
  settings = {
    require_api_key: true,
    http_enabled: false,
    http_loopback_no_auth: false,
    http_port: 7655,
    allow_writes: false,
    require_key_notice: false,
    cli_when_closed: false,
    new_clients: [],
  };
});

function scopesOf(raw: unknown): McpScope[] {
  const list = (Array.isArray(raw) ? raw : []).filter(
    (s): s is McpScope => s === 'read' || s === 'write',
  );
  return (['read', 'write'] as const).filter((s) => s === 'read' || list.includes(s));
}

const COMMAND = '/Applications/Treasury.app/Contents/MacOS/treasury';

/** Testes: um cofre de antes da V139, com o aviso da chave obrigatória. */
export function setMockRequireKeyNotice(on: boolean): void {
  settings = { ...settings, require_key_notice: on };
}

export function setMockNewClients(list: McpServerSettings['new_clients']): void {
  settings = { ...settings, new_clients: list };
}

registerMockHandlers({
  'mcp.keys_list': () => {
    // Como no núcleo: revogada há mais de 7 dias sai da lista.
    const cutoff = Date.now() - 7 * 86_400_000;
    keys = keys.filter((k) => !k.revoked_at || Date.parse(k.revoked_at) > cutoff);
    return [...keys];
  },
  'mcp.key_delete': (args) => {
    const k = keys.find((x) => x.id === args.id);
    if (!k) notFound('chave de API');
    if (!k.revoked_at) invalid('revogue a chave antes de excluir');
    keys = keys.filter((x) => x.id !== args.id);
    return [...keys];
  },
  'mcp.key_create': (args): McpKeyCreated => {
    const name = typeof args.name === 'string' ? args.name.trim() : '';
    if (!name) invalid('dê um nome à chave (até 60 caracteres)');
    counter += 1;
    const key = `trsk_mock${String(counter).padStart(4, '0')}chaveDeTesteDoTreasury`;
    const days = typeof args.expires_days === 'number' ? args.expires_days : null;
    const info: McpApiKey = {
      id: `mock-mcp-key-${counter}`,
      name,
      prefix: key.slice(0, 11),
      scopes: scopesOf(args.scopes),
      created_at: new Date().toISOString(),
      expires_at: days ? new Date(Date.now() + days * 86_400_000).toISOString() : null,
      last_used_at: null,
      revoked_at: null,
    };
    keys = [info, ...keys];
    return { key, info };
  },
  'mcp.key_revoke': (args) => {
    keys = keys.map((k) =>
      k.id === args.id && !k.revoked_at ? { ...k, revoked_at: new Date().toISOString() } : k,
    );
    return [...keys];
  },
  'mcp.server_settings': () => ({
    ...settings,
    http_loopback_no_auth: settings.http_loopback_no_auth && developer(),
  }),
  'mcp.set_server_settings': (args) => {
    const port = Number(args.http_port);
    if (!(port > 1024)) invalid('use uma porta acima de 1024');
    settings = {
      require_api_key: args.require_api_key === true,
      http_enabled: args.http_enabled === true,
      http_loopback_no_auth: args.http_loopback_no_auth === true && developer(),
      http_port: port,
      allow_writes: args.allow_writes === true,
      require_key_notice: settings.require_key_notice,
      cli_when_closed: args.cli_when_closed === true,
      new_clients: settings.new_clients,
    };
    return { ...settings };
  },
  'mcp.dismiss_require_key_notice': () => {
    settings = { ...settings, require_key_notice: false };
    return { ...settings };
  },
  'mcp.dismiss_client_notices': () => {
    settings = { ...settings, new_clients: [] };
    return { ...settings };
  },
  'mcp.client_config': (args) => {
    ready();
    const command =
      typeof args.command === 'string' && args.command.trim() ? args.command.trim() : COMMAND;
    return { mcpServers: { treasury: { command, args: ['mcp'] } } };
  },
});
