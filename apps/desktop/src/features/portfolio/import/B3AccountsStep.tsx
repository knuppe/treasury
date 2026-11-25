/**
 * Accounts step of the B3 import: one card per custodian of the files (the names of a
 * renamed broker come grouped), with the account the core proposes and why (saved from an
 * earlier import, same institution, similar name), the option to create a new account on
 * the spot (pre-filled from the file; created in the same change set as the import) and
 * what the import does in that account (rows, period, position before → after).
 */
import type { AccountKind } from '@/api/generated/AccountKind';
import type { AccountView } from '@/api/generated/AccountView';
import type { B3Institution } from '@/api/generated/B3Institution';
import type { B3MatchReason } from '@/api/generated/B3MatchReason';
import type { B3NewAccount } from '@/api/generated/B3NewAccount';
import type { InstitutionKind } from '@/api/generated/InstitutionKind';
import { DateInput, Field, Input, Private, Select } from '@/design/components';
import { formatDate } from '@/lib/format';
import { ACCOUNT_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';
import type { MessageKey, Messages } from '@/i18n/keys';

type PlainKey = { [K in MessageKey]: [Messages[K]] extends [never] ? K : never }[MessageKey];

import { formatQuantity } from '../decimal';
import { RowNote, StatusChip } from './shared';

/** The account chosen for a custodian: an existing one, none, or one to create. */
export interface B3Choice {
  accountId: string | null;
  newAccount: B3NewAccount | null;
}

export const NONE = '__none__';
export const NEW_ACCOUNT = '__nova_conta__';

const NEW_KINDS: AccountKind[] = ['brokerage', 'checking', 'savings', 'other', 'foreign'];

const REASON_KEY: Record<B3MatchReason, PlainKey> = {
  chosen: 'b3-motivo-escolhida',
  saved: 'b3-motivo-salva',
  same_institution: 'b3-motivo-mesma-instituicao',
  similar_name: 'b3-motivo-nome',
  created: 'b3-motivo-criada',
};

/** A new account pre-filled from the file, when the core did not propose one. */
export function defaultNewAccount(i: B3Institution): B3NewAccount {
  return (
    i.proposed_new_account ?? {
      name: i.name,
      kind: 'brokerage',
      institution_name: i.name,
      institution_kind: 'broker',
    }
  );
}

/** Name of an account for display: never the raw id. */
export function accountLabel(
  id: string | null,
  accounts: readonly AccountView[],
  institutions: readonly B3Institution[],
): string {
  if (!id) return t('b3-sem-conta');
  return (
    accounts.find((a) => a.id === id)?.name ??
    institutions.find((i) => i.account_id === id)?.account_name ??
    t('b3-conta-desconhecida')
  );
}

export function B3AccountsStep({
  institutions,
  accounts,
  choices,
  onChange,
}: {
  institutions: readonly B3Institution[];
  accounts: readonly AccountView[];
  choices: ReadonlyMap<string, B3Choice>;
  onChange: (institution: string, choice: B3Choice) => void;
}) {
  const options = [
    { value: NONE, label: t('b3-nao-importar') },
    ...accounts.map((a) => ({
      value: a.id,
      label: a.name,
      hint: ACCOUNT_KIND[a.kind],
    })),
    // A suggested account the list does not show (e.g. archived meanwhile): by name.
    ...institutions
      .filter(
        (i) =>
          i.suggested_account_id &&
          !accounts.some((a) => a.id === i.suggested_account_id) &&
          i.account_name,
      )
      .map((i) => ({ value: i.suggested_account_id!, label: i.account_name! })),
    { value: NEW_ACCOUNT, label: t('b3-criar-conta') },
  ];
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[13px] text-dim">{t('b3-contas-texto')}</p>
      <ul className="flex flex-col gap-3" aria-label={t('b3-instituicoes')}>
        {institutions.map((i) => {
          const c = choices.get(i.name) ?? { accountId: i.account_id, newAccount: null };
          const value = c.newAccount ? NEW_ACCOUNT : (c.accountId ?? NONE);
          const suggested = !c.newAccount && c.accountId && c.accountId === i.suggested_account_id;
          return (
            <li
              key={i.name}
              className="flex flex-col gap-3 rounded-card border border-line p-3"
              aria-label={i.name}
            >
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="min-w-0 flex-1">
                  <p className="text-[13.5px] text-fg">{i.name}</p>
                  {i.aliases.length > 1 ? (
                    <RowNote>
                      {t('b3-tambem-como', {
                        nomes: i.aliases.filter((a) => a !== i.name).join(' · '),
                      })}
                    </RowNote>
                  ) : null}
                  {i.b3_accounts.length ? (
                    <RowNote>{t('b3-conta-b3', { contas: i.b3_accounts.join(', ') })}</RowNote>
                  ) : null}
                  <p className="mt-1 flex flex-wrap items-center gap-1">
                    {suggested && i.match_reason ? (
                      <StatusChip tone={i.mapped ? 'positive' : 'dim'}>
                        {t(REASON_KEY[i.match_reason])}
                      </StatusChip>
                    ) : null}
                    {!i.suggested_account_id && !c.newAccount && !c.accountId ? (
                      <RowNote>{t('b3-sem-conta-parecida')}</RowNote>
                    ) : null}
                  </p>
                </div>
                <div className="w-full min-w-60 sm:w-72">
                  <Select
                    aria-label={t('b3-conta-para', { instituicao: i.name })}
                    value={value}
                    onValueChange={(v) => {
                      if (!v) return;
                      if (v === NEW_ACCOUNT)
                        onChange(i.name, { accountId: null, newAccount: defaultNewAccount(i) });
                      else onChange(i.name, { accountId: v === NONE ? null : v, newAccount: null });
                    }}
                    options={options}
                  />
                </div>
              </div>
              {c.newAccount ? (
                <NewAccountForm
                  institution={i.name}
                  value={c.newAccount}
                  onChange={(n) => onChange(i.name, { accountId: null, newAccount: n })}
                />
              ) : null}
              <Summary institution={i} rows={i.rows} />
            </li>
          );
        })}
      </ul>
    </div>
  );
}

function NewAccountForm({
  institution,
  value,
  onChange,
}: {
  institution: string;
  value: B3NewAccount;
  onChange: (n: B3NewAccount) => void;
}) {
  const set = (patch: Partial<B3NewAccount>) => onChange({ ...value, ...patch });
  return (
    <fieldset
      className="grid gap-3 rounded-card bg-surface-2 p-3 sm:grid-cols-2"
      aria-label={t('b3-nova-conta-para', { instituicao: institution })}
    >
      <Field label={t('b3-nova-conta-nome')}>
        <Input value={value.name} onChange={(e) => set({ name: e.target.value })} />
      </Field>
      <Field label={t('b3-nova-conta-tipo')} hint={t('b3-nova-conta-tipo-dica')}>
        <Select
          value={value.kind}
          onValueChange={(v) => {
            if (!v) return;
            const kind = v as AccountKind;
            const institution_kind: InstitutionKind = kind === 'brokerage' ? 'broker' : 'bank';
            set({ kind, institution_kind });
          }}
          options={NEW_KINDS.map((k) => ({ value: k, label: ACCOUNT_KIND[k] }))}
        />
      </Field>
      <Field label={t('b3-nova-conta-instituicao')}>
        <Input
          value={value.institution_name}
          onChange={(e) => set({ institution_name: e.target.value })}
        />
      </Field>
      <Field label={t('b3-nova-conta-abertura')}>
        <DateInput
          value={value.opening_date ?? null}
          onValueChange={(d) => set({ opening_date: d ?? undefined })}
        />
      </Field>
    </fieldset>
  );
}

function Summary({ institution: i, rows }: { institution: B3Institution; rows: number }) {
  const s = i.summary;
  const any = s.trades + s.incomes + s.actions > 0;
  return (
    <div className="flex flex-col gap-1 text-[12.5px] text-dim">
      <p>
        {any
          ? t('b3-resumo-conta', {
              linhas: rows,
              operacoes: s.trades,
              proventos: s.incomes,
              eventos: s.actions,
            })
          : t('b3-resumo-linhas', { linhas: rows })}
        {s.period_from
          ? ` · ${s.period_from === s.period_to ? formatDate(s.period_from) : t('b3-periodo', { de: formatDate(s.period_from), ate: formatDate(s.period_to!) })}`
          : ''}
      </p>
      {s.assets.length ? (
        <ul className="flex flex-wrap gap-x-4 gap-y-1" aria-label={t('b3-resumo-ativos')}>
          {s.assets.map((a) => (
            <li key={a.asset_key} className="whitespace-nowrap">
              <span className="text-fg">{a.ticker ?? a.name}</span>{' '}
              <Private>
                {formatQuantity(a.before)} → {formatQuantity(a.after)}
              </Private>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
