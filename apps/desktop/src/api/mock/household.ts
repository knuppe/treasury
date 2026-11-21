/** Mock de `household.*`: família, membros (titulares) e instituições. */
import type { Household } from '@/api/generated/Household';
import type { Institution } from '@/api/generated/Institution';
import type { InstitutionKind } from '@/api/generated/InstitutionKind';
import type { Member } from '@/api/generated/Member';
import type { MemberProfile } from '@/api/generated/MemberProfile';
import type { MembersOverview } from '@/api/generated/MembersOverview';

import { familyCountry } from './country';
import { getFor as profileFor } from './profile';

import { registerMockHandlers } from '../mockRegistry';
import { searchBanks } from './ledger/bankDetails';
import { guarded } from './ledger/boot';
import { nowUtc } from './ledger/dates';
import { household as currentHousehold } from './ledger/domain';
import { cleanName, cleanOpt, conflict, has, invalid, optBool, str, text } from './ledger/errors';
import {
  type InstitutionRow,
  type MemberRow,
  del,
  EMPTY_PROFILE,
  must,
  newId,
  patch,
  put,
  rows,
  write,
} from './ledger/store';

const INSTITUTION_KINDS: InstitutionKind[] = ['bank', 'broker', 'issuer', 'other'];

function toMember(m: MemberRow): Member {
  return {
    id: m.id,
    name: m.name,
    color: m.color,
    is_primary: m.is_primary,
    archived: m.archived,
    sort_order: m.sort_order,
    profile: m.profile ?? EMPTY_PROFILE,
  };
}

function toInstitution(i: InstitutionRow): Institution {
  return { id: i.id, name: i.name, kind: i.kind, code: i.code, website: i.website };
}

function members(includeArchived: boolean): Member[] {
  return rows('member')
    .filter((m) => includeArchived || !m.archived)
    .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name, 'pt-BR'))
    .map(toMember);
}

function uniqueMember(name: string, except?: string): void {
  if (rows('member').some((m) => m.id !== except && m.name.toLowerCase() === name.toLowerCase())) {
    conflict(`já existe um membro chamado ${name}`);
  }
}

function institutionKind(v: unknown): InstitutionKind {
  if (v === undefined || v === null) return 'bank';
  if (!INSTITUTION_KINDS.includes(v as InstitutionKind)) {
    invalid(`argumentos inválidos: unknown variant \`${text(v)}\``);
  }
  return v as InstitutionKind;
}

function updateMember(args: Record<string, unknown>): Member {
  const id = str(args, 'id');
  const cur = must('member', id, `membro ${id}`);
  const changes: Partial<MemberRow> = {};
  if (has(args, 'name')) changes.name = cleanName(args.name, 'do membro');
  if (has(args, 'color')) changes.color = cleanOpt(args.color);
  if (has(args, 'archived')) changes.archived = Boolean(optBool(args, 'archived'));
  if (has(args, 'sort_order')) changes.sort_order = Number(args.sort_order);
  if (has(args, 'profile') && args.profile)
    changes.profile = { ...EMPTY_PROFILE, ...(args.profile as Partial<MemberProfile>) };
  if (has(args, 'is_primary')) {
    const primary = Boolean(optBool(args, 'is_primary'));
    changes.is_primary = primary;
    if (primary) {
      for (const m of rows('member')) {
        if (m.id !== id && m.is_primary)
          patch('member', m.id, { is_primary: false, updated_at: nowUtc() });
      }
    }
  }
  const archived = changes.archived ?? cur.archived;
  const primary = changes.is_primary ?? cur.is_primary;
  if (archived && primary) {
    invalid(
      cur.archived
        ? 'um membro arquivado não pode ser o principal'
        : 'o membro principal não pode ser arquivado; escolha outro membro principal antes',
    );
  }
  if (changes.name) uniqueMember(changes.name, id);
  const next = patch('member', id, { ...changes, updated_at: nowUtc() });
  return toMember(next.updated_at === cur.updated_at ? cur : next);
}

registerMockHandlers(
  guarded({
    'household.get': (): Household => {
      familyCountry();
      const h = currentHousehold();
      return { id: h.id, name: h.name, base_currency: h.base_currency, country: h.country ?? 'BR' };
    },
    'household.update': (args): Household => {
      const h = currentHousehold();
      return write('Dados da família alterados', () => {
        const changes: Record<string, unknown> = {};
        if (has(args, 'name')) changes.name = cleanName(args.name, 'da família');
        if (has(args, 'base_currency')) {
          const c = String(args.base_currency).trim().toUpperCase();
          if (!/^[A-Z]{3}$/.test(c)) invalid(`moeda inválida: ${String(args.base_currency)}`);
          changes.base_currency = c;
        }
        const next = patch('household', h.id, { ...changes, updated_at: nowUtc() });
        return {
          id: next.id,
          name: next.name,
          base_currency: next.base_currency,
          country: next.country ?? 'BR',
        };
      }).value;
    },
    'household.list_members': (args) => members(Boolean(args.include_archived)),
    // Finanças por membro (como `household::overview` do núcleo, sem câmbio nem patrimônio).
    'household.members_overview': (args): MembersOverview => {
      const month =
        typeof args.month === 'string' && args.month
          ? args.month
          : new Date().toISOString().slice(0, 7);
      const owners = new Map<string, string[]>();
      for (const am of rows('account_member'))
        owners.set(am.account_id, [...(owners.get(am.account_id) ?? []), am.member_id]);
      const who = (t: { member_id: string | null; account_id: string }) => {
        if (t.member_id) return t.member_id;
        const o = owners.get(t.account_id) ?? [];
        return o.length === 1 ? o[0]! : null;
      };
      const sums = new Map<string | null, { income: number; expense: number }>();
      for (const t of rows('txn')) {
        if (!t.date.startsWith(month) || (t.kind !== 'income' && t.kind !== 'expense')) continue;
        const k = who(t);
        const cur = sums.get(k) ?? { income: 0, expense: 0 };
        if (t.kind === 'income') cur.income += t.amount;
        else cur.expense += -t.amount;
        sums.set(k, cur);
      }
      const monthOf = (k: string | null) => {
        const c = sums.get(k) ?? { income: 0, expense: 0 };
        return { ...c, net: c.income - c.expense, savings_rate_12m: null };
      };
      return {
        month,
        currency: currentHousehold().base_currency,
        members: members(false).map((m) => ({
          member: m,
          level: profileFor(m.id).level,
          tone: profileFor(m.id).tone,
          has_own_profile: profileFor(m.id).source !== 'familia',
          month: monthOf(m.id),
          net_worth: 0,
          accounts: [...owners.values()].filter((o) => o.includes(m.id)).length,
          loans: 0,
        })),
        shared: monthOf(null),
      };
    },
    'household.create_member': (args): Member => {
      const name = cleanName(args.name, 'do membro');
      return write(`Membro criado: ${name}`, () => {
        uniqueMember(name);
        const now = nowUtc();
        const primary = Boolean(args.is_primary);
        if (primary)
          for (const m of rows('member'))
            patch('member', m.id, { is_primary: false, updated_at: now });
        const sort = Math.max(-1, ...rows('member').map((m) => m.sort_order)) + 1;
        return toMember(
          put('member', {
            id: newId(),
            name,
            color: cleanOpt(args.color),
            is_primary: primary,
            archived: false,
            sort_order: sort,
            profile: { ...EMPTY_PROFILE, ...((args.profile as Partial<MemberProfile>) ?? {}) },
            created_at: now,
            updated_at: now,
          }),
        );
      }).value;
    },
    'household.update_member': (args) => write('Membro alterado', () => updateMember(args)).value,
    'household.archive_member': (args) => {
      const archived =
        args.archived === undefined || args.archived === null ? true : Boolean(args.archived);
      return write(archived ? 'Membro arquivado' : 'Membro reativado', () =>
        updateMember({ id: args.id, archived }),
      ).value;
    },
    'household.banks': (args) =>
      searchBanks(
        typeof args.query === 'string' ? args.query : undefined,
        typeof args.limit === 'number' ? args.limit : 20,
      ),
    'household.list_institutions': () =>
      rows('institution')
        .sort((a, b) => a.name.localeCompare(b.name, 'pt-BR', { sensitivity: 'base' }))
        .map(toInstitution),
    'household.create_institution': (args): Institution => {
      const name = cleanName(args.name, 'da instituição');
      return write(`Instituição criada: ${name}`, () => {
        const now = nowUtc();
        return toInstitution(
          put('institution', {
            id: newId(),
            name,
            kind: institutionKind(args.kind),
            code: cleanOpt(args.code),
            website: cleanOpt(args.website),
            created_at: now,
            updated_at: now,
          }),
        );
      }).value;
    },
    'household.update_institution': (args): Institution => {
      const id = str(args, 'id');
      must('institution', id, `instituição ${id}`);
      return write('Instituição alterada', () => {
        const changes: Partial<InstitutionRow> = {};
        if (has(args, 'name')) changes.name = cleanName(args.name, 'da instituição');
        if (has(args, 'kind')) changes.kind = institutionKind(args.kind);
        if (has(args, 'code')) changes.code = cleanOpt(args.code);
        if (has(args, 'website')) changes.website = cleanOpt(args.website);
        return toInstitution(patch('institution', id, { ...changes, updated_at: nowUtc() }));
      }).value;
    },
    'household.delete_institution': (args) => {
      const id = str(args, 'id');
      const inst = must('institution', id, `instituição ${id}`);
      write('Instituição apagada', () => {
        const used = rows('account').filter((a) => a.institution_id === id);
        if (used.length > 0) {
          conflict(
            `a instituição ${inst.name} é usada por ${used.length} conta(s); troque a instituição dessas contas antes de apagar`,
          );
        }
        del('institution', id);
      });
      return null;
    },
  }),
);
