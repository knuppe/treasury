/**
 * Caixa de entrada de arquivos (`import.inbox_*`, docs/inbox.md): o que chegou na
 * pasta de entrada (padrão ~/Treasury/Inbox), guardado no cofre até a importação.
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import type { InboxFile } from '@/api/generated/InboxFile';
import type { InboxItem } from '@/api/generated/InboxItem';
import type { InboxStatus } from '@/api/generated/InboxStatus';

export const INBOX_KEY = ['import', 'inbox'] as const;

/** Lê a pasta e lista a caixa (pendentes e não reconhecidos; `includeDone`: todos). */
export function useInbox(includeDone = false) {
  return useQuery({
    queryKey: [...INBOX_KEY, { includeDone }] as const,
    queryFn: () => call<InboxItem[]>('import.inbox_list', { include_done: includeDone }),
  });
}

export function useSetInboxStatus() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (a: { id: string; status: InboxStatus }) =>
      call<InboxItem>('import.inbox_set_status', a),
    onSuccess: () => client.invalidateQueries({ queryKey: INBOX_KEY }),
  });
}

/** O arquivo guardado, pronto para a janela de importação. */
export async function inboxFile(id: string): Promise<File> {
  const c = await call<InboxFile>('import.inbox_file', { id });
  const bin = atob(c.content_base64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return new File([bytes], c.filename, { type: 'application/octet-stream' });
}
