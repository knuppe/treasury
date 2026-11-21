/** Hooks de `household.*`: família, titulares e instituições. */
import type { Bank } from '../generated/Bank';
import type { Household } from '../generated/Household';
import type { HouseholdPatch } from '../generated/HouseholdPatch';
import type { Institution } from '../generated/Institution';
import type { InstitutionInput } from '../generated/InstitutionInput';
import type { InstitutionPatch } from '../generated/InstitutionPatch';
import type { Member } from '../generated/Member';
import type { MemberInput } from '../generated/MemberInput';
import type { MemberPatch } from '../generated/MemberPatch';
import type { MembersOverview } from '../generated/MembersOverview';
import { useQueryClient } from '@tanstack/react-query';

import { vaultKeys } from '../vaultQueries';
import { useCore, useCoreMutation } from './coreQuery';

export const useHousehold = () => useCore<Household>('household', 'get');

export const useMembers = (includeArchived = false) =>
  useCore<Member[]>('household', 'list_members', { include_archived: includeArchived });

/** Lista oficial de bancos (Banco Central), por nome, código COMPE ou ISPB. */
export const useBanks = (query: string, enabled = true) =>
  useCore<Bank[]>('household', 'banks', { query, limit: 12 }, { enabled });

export const useInstitutions = () => useCore<Institution[]>('household', 'list_institutions');

/** Finanças de cada membro no mês (docs/members.md): o que entrou, saiu e sobrou, patrimônio
 * e nível do perfil, mais o que é da família. */
export const useMembersOverview = (month: string) =>
  useCore<MembersOverview>('household', 'members_overview', { month });

/** O nome da família também aparece no estado do cofre (barra lateral). */
export function useUpdateHousehold() {
  const client = useQueryClient();
  return useCoreMutation<HouseholdPatch, Household>('household.update', {
    affects: ['household'],
    onSuccess: () => void client.invalidateQueries({ queryKey: vaultKeys.status }),
  });
}

export const useCreateMember = () =>
  useCoreMutation<MemberInput, Member>('household.create_member', { affects: ['household'] });

export const useUpdateMember = () =>
  useCoreMutation<MemberPatch, Member>('household.update_member', { affects: ['household'] });

export const useArchiveMember = () =>
  useCoreMutation<{ id: string; archived: boolean }, Member>('household.archive_member', {
    affects: ['household'],
  });

export const useCreateInstitution = () =>
  useCoreMutation<InstitutionInput, Institution>('household.create_institution', {
    affects: ['household'],
  });

export const useUpdateInstitution = () =>
  useCoreMutation<InstitutionPatch, Institution>('household.update_institution', {
    affects: ['household'],
  });

export const useDeleteInstitution = () =>
  useCoreMutation<{ id: string }, null>('household.delete_institution', { affects: ['household'] });
