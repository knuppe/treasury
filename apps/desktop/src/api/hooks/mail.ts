/**
 * Hooks de `mail.*` (docs/email-ingest.md): contas (pasta local ou IMAP), regras, a fila de
 * revisão e a auditoria. A senha de app vai só no `set_password` e nunca volta.
 */
import type { MailAccount } from '../generated/MailAccount';
import type { MailAccountInput } from '../generated/MailAccountInput';
import type { MailAuditEntry } from '../generated/MailAuditEntry';
import type { MailConnectionTest } from '../generated/MailConnectionTest';
import type { MailDecision } from '../generated/MailDecision';
import type { MailDiscovery } from '../generated/MailDiscovery';
import type { MailListArgs } from '../generated/MailListArgs';
import type { MailMessage } from '../generated/MailMessage';
import type { MailProvider } from '../generated/MailProvider';
import type { MailRule } from '../generated/MailRule';
import type { MailRuleInput } from '../generated/MailRuleInput';
import type { MailRuleTestArgs } from '../generated/MailRuleTestArgs';
import type { MailStatus } from '../generated/MailStatus';
import type { SecretStatus } from '../generated/SecretStatus';
import { call } from '../client';
import { t } from '@/i18n';
import { useCore, useCoreMutation } from './coreQuery';

export const useMailProviders = () =>
  useCore<MailProvider[]>('mail', 'providers', undefined, { staleTime: Infinity });

/** Contas; enquanto alguma busca, relê a cada 3 s (o resumo da busca chega sozinho). */
export const useMailAccounts = () =>
  useCore<MailAccount[]>('mail', 'accounts.list', undefined, {
    refetchInterval: (q) => (q.state.data?.some((a) => a.syncing) ? 3000 : false),
  });

export const useMailRules = (accountId?: string | null) =>
  useCore<MailRule[]>('mail', 'rules.list', accountId ? { account_id: accountId } : {});

export const useMailMessages = (args: MailListArgs) =>
  useCore<MailMessage[]>('mail', 'list', args, {
    // A IA lendo uma peça: relê até ela terminar.
    refetchInterval: (q) =>
      q.state.data?.some((m) => m.parts.some((p) => p.status === 'ai_pending')) ? 4000 : false,
  });

export const useMailAudit = (accountId?: string | null) =>
  useCore<MailAuditEntry[]>(
    'mail',
    'audit',
    accountId ? { account_id: accountId, limit: 200 } : { limit: 200 },
  );

export const useWouldDelete = (accountId: string | null) =>
  useCore<MailAuditEntry[]>(
    'mail',
    'would_delete',
    { account_id: accountId },
    { enabled: Boolean(accountId) },
  );

export const useSaveMailAccount = () =>
  useCoreMutation<MailAccountInput, MailAccount>('mail.accounts.save', { affects: ['mail'] });

export const useDeleteMailAccount = () =>
  useCoreMutation<{ id: string }, null>('mail.accounts.delete', { affects: ['mail'] });

export const useSetMailPassword = () =>
  useCoreMutation<{ id: string; password: string }, SecretStatus>('mail.accounts.set_password', {
    affects: ['mail'],
    sensitive: true,
  });

export const useConfirmRetention = () =>
  useCoreMutation<{ id: string }, MailAccount>('mail.accounts.confirm_retention', {
    affects: ['mail'],
  });

export const useSaveMailRule = () =>
  useCoreMutation<MailRuleInput, MailRule>('mail.rules.save', { affects: ['mail'] });

export const useDeleteMailRule = () =>
  useCoreMutation<{ id: string }, null>('mail.rules.delete', { affects: ['mail'] });

export interface MailRuleTest {
  sample: MailDecision | null;
  recent: {
    message_id: string;
    subject: string | null;
    sender: string | null;
    matches: boolean;
    decision: MailDecision;
  }[];
}

export const useTestMailRule = () =>
  useCoreMutation<MailRuleTestArgs, MailRuleTest>('mail.rules.test', { affects: [] });

export const useDiscoverMail = () =>
  useCoreMutation<{ address: string }, MailDiscovery>('mail.discover', { affects: [] });

/** Testa a conexão (com a senha digitada agora ou a cadastrada); confirma a de um backup. */
export const useTestMailConnection = () =>
  useCoreMutation<{ id: string; password?: string }, MailConnectionTest | { files: number }>(
    'mail.test_connection',
    { affects: ['mail'], sensitive: true },
  );

export const useSyncMail = () =>
  useCoreMutation<{ id: string }, { started: boolean }>('mail.sync_now', { affects: ['mail'] });

export const useAskMailAi = () =>
  useCoreMutation<{ part_id: string }, { job_id: string }>('mail.ask_ai', { affects: ['mail'] });

/** Importa a peça na revisão (OFX, cupom, aviso do corpo). */
export const useApplyMailPart = () =>
  useCoreMutation<
    { part_id: string; account_id?: string },
    { status: MailStatus; message: string }
  >('mail.apply_part', { affects: ['mail', 'ledger', 'receipts'] });

export const useMarkMailPart = () =>
  useCoreMutation<{ id: string; status: MailStatus; result_ref?: string }, MailMessage>(
    'mail.mark_part',
    { affects: ['mail'] },
  );

export const useIgnoreMail = () =>
  useCoreMutation<{ id: string }, MailMessage>('mail.ignore', { affects: ['mail'] });

/** Conteúdo de uma peça como arquivo (para abrir no importador). */
export async function mailPartFile(id: string): Promise<File> {
  const c = await call<{ filename: string; mime: string | null; content_base64: string }>(
    'mail.part_content',
    { id },
  );
  const bin = atob(c.content_base64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return new File([bytes], c.filename, { type: c.mime ?? 'application/octet-stream' });
}

/** Escolhe a pasta vigiada pelo diálogo do sistema (só no app; no navegador, `null`). */
export async function pickMailFolder(): Promise<string | null> {
  const { isTauri } = await import('../client');
  if (!isTauri()) return null;
  const { invoke } = await import('@tauri-apps/api/core');
  const path = await invoke<string | string[] | null>('plugin:dialog|open', {
    options: { title: t('email-pasta-vigiar'), multiple: false, directory: true },
  });
  return typeof path === 'string' ? path : null;
}
