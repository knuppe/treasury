/** Hooks de `mcp.*`: configuração para clientes externos e auditoria das chamadas. */
import { useMutation, useQueryClient } from '@tanstack/react-query';

import { call, type ApiError } from '../client';
import type { AuditEntry } from '../generated/AuditEntry';
import type { AuditListArgs } from '../generated/AuditListArgs';
import type { McpApiKey } from '../generated/McpApiKey';
import type { McpKeyCreateArgs } from '../generated/McpKeyCreateArgs';
import type { McpKeyCreated } from '../generated/McpKeyCreated';
import type { McpServerSettings } from '../generated/McpServerSettings';
import { useCore } from './coreQuery';

export const useMcpConfig = (command?: string) =>
  useCore<Record<string, unknown>>('mcp', 'client_config', command ? { command } : {});

export const useMcpAudit = (args: AuditListArgs) =>
  useCore<AuditEntry[]>('mcp', 'audit_list', args);

// --- Chaves de API e servidor para agentes externos -----------------------------------

export const mcpAccessKeys = {
  keys: ['mcp', 'keys_list'] as const,
  settings: ['mcp', 'server_settings'] as const,
};

export const useMcpKeys = () => useCore<McpApiKey[]>('mcp', 'keys_list', {});

export const useMcpServerSettings = () => useCore<McpServerSettings>('mcp', 'server_settings', {});

/** A chave inteira volta só nesta resposta (fica no estado do diálogo, nunca no cache). */
export function useCreateMcpKey() {
  const client = useQueryClient();
  return useMutation<McpKeyCreated, ApiError, McpKeyCreateArgs>({
    mutationFn: (args) => call<McpKeyCreated>('mcp.key_create', args),
    gcTime: 0,
    onSuccess: () => void client.invalidateQueries({ queryKey: ['mcp'] }),
  });
}

export function useRevokeMcpKey() {
  const client = useQueryClient();
  return useMutation<McpApiKey[], ApiError, { id: string }>({
    mutationFn: (args) => call<McpApiKey[]>('mcp.key_revoke', args),
    onSuccess: () => void client.invalidateQueries({ queryKey: ['mcp'] }),
  });
}

/** Exclui uma chave já revogada (a auditoria das chamadas feitas com ela fica). */
export function useDeleteMcpKey() {
  const client = useQueryClient();
  return useMutation<McpApiKey[], ApiError, { id: string }>({
    mutationFn: (args) => call<McpApiKey[]>('mcp.key_delete', args),
    onSuccess: () => void client.invalidateQueries({ queryKey: ['mcp'] }),
  });
}

/** Dispensa o aviso de que a chave de API passou a ser exigida (cofres de antes da V139). */
export function useDismissRequireKeyNotice() {
  const client = useQueryClient();
  return useMutation<McpServerSettings, ApiError, void>({
    mutationFn: () => call<McpServerSettings>('mcp.dismiss_require_key_notice', {}),
    onSuccess: () => void client.invalidateQueries({ queryKey: ['mcp'] }),
  });
}

/** Dispensa o aviso de clientes MCP novos (docs/local-security.md, item 14). */
export function useDismissClientNotices() {
  const client = useQueryClient();
  return useMutation<McpServerSettings, ApiError, void>({
    mutationFn: () => call<McpServerSettings>('mcp.dismiss_client_notices', {}),
    onSuccess: () => void client.invalidateQueries({ queryKey: ['mcp'] }),
  });
}

export function useSaveMcpServerSettings() {
  const client = useQueryClient();
  return useMutation<McpServerSettings, ApiError, McpServerSettings>({
    mutationFn: (args) => call<McpServerSettings>('mcp.set_server_settings', args),
    onSuccess: () => void client.invalidateQueries({ queryKey: ['mcp'] }),
  });
}
