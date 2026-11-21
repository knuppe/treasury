/**
 * "Ver como" (docs/members.md): a família toda ou um membro. Sem login por membro: quem usa
 * o app escolhe a perspectiva no topo; o painel, a saúde financeira, o nível do perfil e o tom
 * do agente passam a ser os desse membro. Vale no cofre em que foi escolhido.
 */
import type { QueryClient } from '@tanstack/react-query';

import type { Member } from '@/api/generated/Member';
import { useMembers } from '@/api/hooks/household';
import type { VaultStatus } from '@/api/vault';
import { useVaultStatus, vaultKeys } from '@/api/vaultQueries';
import { useUi } from '@/state/ui';

export interface ViewAs {
  /** `null`: a família. */
  memberId: string | null;
  member: Member | null;
  members: Member[];
  setViewAs: (memberId: string | null) => void;
}

export function useViewAs(): ViewAs {
  const vaultId = useVaultStatus().data?.vault_id ?? '';
  const viewAs = useUi((s) => s.viewAs);
  const set = useUi((s) => s.setViewAs);
  const members = useMembers().data ?? [];
  const chosen = viewAs && viewAs.vaultId === vaultId ? viewAs.memberId : null;
  const member = members.find((m) => m.id === chosen) ?? null;
  return {
    memberId: member?.id ?? null,
    member,
    members,
    setViewAs: (memberId) => set(memberId ? { vaultId, memberId } : null),
  };
}

/** O membro em vista fora do React (ex.: ao mandar uma mensagem ao agente). */
export function currentMemberId(client: QueryClient): string | null {
  const vaultId = client.getQueryData<VaultStatus>(vaultKeys.status)?.vault_id;
  const v = useUi.getState().viewAs;
  return v && v.vaultId === vaultId ? v.memberId : null;
}
