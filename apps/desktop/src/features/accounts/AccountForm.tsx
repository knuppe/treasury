import { useState } from 'react';

import type { AccountKind } from '@/api/generated/AccountKind';
import type { AccountView } from '@/api/generated/AccountView';
import type { CardDetails } from '@/api/generated/CardDetails';
import {
  useAccounts,
  useCreateAccount,
  useDeleteAccount,
  useUpdateAccount,
} from '@/api/hooks/accounts';
import { useCountryFeatures } from '@/api/hooks/country';
import { useRetirementPlans } from '@/api/hooks/retirement';
import { useCreateInstitution, useInstitutions, useMembers } from '@/api/hooks/household';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Input,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';
import { Checkbox, ConfirmDialog, InlineError, Switch } from '@/features/transactions/ui/controls';
import { ACCOUNT_KIND } from '@/features/transactions/ui/labels';
import { useBaseCurrency } from '@/state/currency';

import { BankPicker } from './BankPicker';
import { OverdraftFields, overdraftDraft, overdraftTerms, type OverdraftDraft } from './Overdraft';

const KINDS: AccountKind[] = [
  'checking',
  'savings',
  'cash',
  'credit_card',
  'brokerage',
  'foreign',
  'benefit',
  'fgts',
  'retirement',
  'other',
];
/** Tipos que só existem num país (só aparecem num cofre desse país; docs/brazil/fgts.md, 6.8). */
const COUNTRY_KINDS: Partial<Record<AccountKind, 'fgts'>> = { fgts: 'fgts' };
const NONE = '__none__';
const NEW_INSTITUTION = '__new__';
const CURRENCIES = ['BRL', 'USD', 'EUR', 'GBP', 'CHF', 'CAD', 'JPY', 'ARS'];

/** Moedas oferecidas: a base da família primeiro, depois as mais comuns. */
function currencyOptions(base: string): string[] {
  return [base, ...CURRENCIES.filter((c) => c !== base)];
}

/** Moeda sugerida para uma conta no exterior: dólar, ou euro numa família que já usa dólar. */
function foreignDefault(base: string): string {
  return base === 'USD' ? 'EUR' : 'USD';
}

interface Form {
  name: string;
  kind: AccountKind;
  currency: string;
  institutionId: string | null;
  memberIds: string[];
  openingBalance: number | null;
  openingDate: string | null;
  includeInNetWorth: boolean;
  notes: string;
  limit: number | null;
  closingDay: number;
  dueDay: number;
  paymentAccountId: string | null;
  network: string;
  last4: string;
  branch: string;
  accountNumber: string;
  overdraft: OverdraftDraft;
  /** Conta de aposentadoria: o plano do país e a cobertura (HSA). */
  retirementPlan: string | null;
  retirementCoverage: string;
}

function formOf(a: AccountView | null, kind: AccountKind, base: string): Form {
  return {
    name: a?.name ?? '',
    kind: a?.kind ?? kind,
    currency: a?.currency ?? (kind === 'foreign' ? foreignDefault(base) : base),
    institutionId: a?.institution_id ?? null,
    memberIds: a?.member_ids ?? [],
    openingBalance: a ? (a.kind === 'credit_card' ? -a.opening_balance : a.opening_balance) : null,
    openingDate: a?.opening_date ?? todayIso(),
    includeInNetWorth: a?.include_in_net_worth ?? true,
    notes: a?.notes ?? '',
    limit: a?.card?.credit_limit ?? null,
    closingDay: a?.card?.closing_day ?? 3,
    dueDay: a?.card?.due_day ?? 10,
    paymentAccountId: a?.card?.payment_account_id ?? null,
    network: a?.card?.network ?? '',
    last4: a?.card?.last4 ?? '',
    branch: a?.branch ?? '',
    accountNumber: a?.account_number ?? '',
    overdraft: overdraftDraft(a?.overdraft),
    retirementPlan: a?.retirement_plan ?? null,
    retirementCoverage: a?.retirement_coverage ?? 'self',
  };
}

/**
 * Criar ou editar conta, com os dados do cartão (limite, fechamento,
 * vencimento, conta pagadora), membros da família e instituição (criada aqui mesmo).
 */
export function AccountForm({
  open,
  onOpenChange,
  account,
  initialKind = 'checking',
  onSaved,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  account: AccountView | null;
  initialKind?: AccountKind;
  onSaved?: (a: AccountView) => void;
}) {
  const base = useBaseCurrency();
  const features = useCountryFeatures();
  const plans = useRetirementPlans().data ?? [];
  const { data: members = [] } = useMembers();
  const { data: institutions = [] } = useInstitutions();
  const { data: accounts = [] } = useAccounts();
  const create = useCreateAccount();
  const update = useUpdateAccount();
  const remove = useDeleteAccount();
  const createInstitution = useCreateInstitution();
  const [form, setForm] = useState<Form>(() => formOf(account, initialKind, base));
  const [key, setKey] = useState(`${account?.id ?? 'new'}:${String(open)}`);
  const [error, setError] = useState<unknown>(null);
  const [newInstitution, setNewInstitution] = useState<string | null>(null);
  const [newBankCode, setNewBankCode] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const nextKey = `${account?.id ?? 'new'}:${String(open)}`;
  if (key !== nextKey) {
    setKey(nextKey);
    setForm(formOf(account, initialKind, base));
    setError(null);
    setNewInstitution(null);
    setNewBankCode(null);
  }
  const set = (p: Partial<Form>) => setForm((f) => ({ ...f, ...p }));
  const isCard = form.kind === 'credit_card';
  const isRetirement = form.kind === 'retirement';
  const plan = isRetirement
    ? (plans.find((p) => p.id === form.retirementPlan) ?? plans[0] ?? null)
    : null;
  const retirementFields = plan
    ? {
        retirement_plan: plan.id,
        ...(plan.coverage ? { retirement_coverage: form.retirementCoverage } : {}),
      }
    : {};
  const canOverdraft = form.kind === 'checking' || form.kind === 'other';
  // Agência e conta: o que o OFX (`BRANCHID`, `ACCTID`) traz para reconhecer a conta.
  const hasBankRef = form.kind === 'checking' || form.kind === 'savings' || form.kind === 'other';
  // Editando: desligar o cheque especial remove o limite (`null`); criando, nada a mandar.
  const overdraftPatch = () =>
    canOverdraft && form.overdraft.enabled
      ? { overdraft: overdraftTerms(form.overdraft, features.br_credit_rates) }
      : account?.overdraft
        ? { overdraft: null }
        : {};
  const editing = Boolean(account);

  const card = (): CardDetails => ({
    credit_limit: form.limit,
    closing_day: form.closingDay,
    due_day: form.dueDay,
    payment_account_id: form.paymentAccountId,
    network: form.network.trim() || null,
    last4: form.last4.trim() || null,
  });

  const save = async () => {
    if (!form.name.trim()) return setError(new Error(t('contas-erro-nome')));
    if (!form.openingDate) return setError(new Error(t('contas-erro-data')));
    setError(null);
    try {
      let institutionId = form.institutionId;
      if (newInstitution?.trim()) {
        // Banco da lista oficial que já está cadastrado: reaproveita em vez de duplicar.
        const same = newBankCode ? institutions.find((i) => i.code === newBankCode) : undefined;
        const inst =
          same ??
          (await createInstitution.mutateAsync({
            name: newInstitution.trim(),
            kind: isCard ? 'issuer' : form.kind === 'brokerage' ? 'broker' : 'bank',
            ...(newBankCode ? { code: newBankCode } : {}),
          }));
        institutionId = inst.id;
      }
      const bankRef = hasBankRef
        ? { branch: form.branch.trim() || null, account_number: form.accountNumber.trim() || null }
        : {};
      // Cartão: o saldo inicial é dívida (negativo); nas demais, o que tem na conta.
      const opening = (form.openingBalance ?? 0) * (isCard ? -1 : 1);
      let saved: AccountView;
      if (account) {
        saved = await update.mutateAsync({
          id: account.id,
          name: form.name.trim(),
          institution_id: institutionId,
          member_ids: form.memberIds,
          opening_balance: opening,
          opening_date: form.openingDate,
          include_in_net_worth: form.includeInNetWorth,
          notes: form.notes.trim() || null,
          ...(isCard ? { card: card() } : {}),
          ...bankRef,
          ...overdraftPatch(),
          ...retirementFields,
        });
        toast({ tone: 'success', title: t('contas-conta-atualizada'), description: saved.name });
      } else {
        saved = await create.mutateAsync({
          name: form.name.trim(),
          kind: form.kind,
          currency: form.currency,
          ...(institutionId ? { institution_id: institutionId } : {}),
          member_ids: form.memberIds,
          opening_balance: opening,
          opening_date: form.openingDate,
          include_in_net_worth: form.includeInNetWorth,
          ...(form.notes.trim() ? { notes: form.notes.trim() } : {}),
          ...(isCard ? { card: card() } : {}),
          ...(form.branch.trim() && hasBankRef ? { branch: form.branch.trim() } : {}),
          ...(form.accountNumber.trim() && hasBankRef
            ? { account_number: form.accountNumber.trim() }
            : {}),
          ...(canOverdraft && form.overdraft.enabled
            ? { overdraft: overdraftTerms(form.overdraft, features.br_credit_rates) }
            : {}),
          ...retirementFields,
        });
        toast({ tone: 'success', title: t('contas-conta-criada'), description: saved.name });
      }
      onSaved?.(saved);
      onOpenChange(false);
    } catch (e) {
      setError(e);
    }
  };

  const institutionOptions = [
    { value: NONE, label: '—' },
    ...institutions.map((i) => ({ value: i.id, label: i.name })),
    { value: NEW_INSTITUTION, label: t('contas-nova-instituicao-opcao') },
  ];

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      index="02"
      size="lg"
      title={editing ? t('contas-editar-nome', { nome: account!.name }) : t('contas-nova-conta')}
      footer={
        <>
          {account ? (
            <>
              <Button
                variant="ghost"
                className="mr-auto"
                onClick={() =>
                  update.mutate(
                    { id: account.id, archived: !account.archived },
                    {
                      onSuccess: () => {
                        toast({
                          tone: 'success',
                          title: account.archived
                            ? t('contas-conta-reativada')
                            : t('contas-conta-arquivada'),
                          description: account.name,
                        });
                        onOpenChange(false);
                      },
                      onError: setError,
                    },
                  )
                }
              >
                {account.archived ? t('contas-reativar') : t('contas-arquivar')}
              </Button>
              <Button variant="danger" onClick={() => setConfirmDelete(true)}>
                {t('comum-apagar')}
              </Button>
            </>
          ) : null}
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={create.isPending || update.isPending}
            onClick={() => void save()}
          >
            {editing ? t('comum-salvar') : t('contas-criar-conta')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <Field label={t('contas-nome')}>
            <Input
              value={form.name}
              onChange={(e) => set({ name: e.target.value })}
              placeholder={t('contas-nome-exemplo')}
              autoFocus
            />
          </Field>
          <Field label={t('contas-tipo')} hint={editing ? t('contas-tipo-fixo') : undefined}>
            <Select
              value={form.kind}
              disabled={editing}
              onValueChange={(v) => {
                const kind = v as AccountKind;
                set({
                  kind,
                  currency:
                    kind === 'fgts'
                      ? 'BRL'
                      : kind === 'foreign'
                        ? foreignDefault(base)
                        : form.kind === 'foreign'
                          ? base
                          : form.currency,
                });
              }}
              options={KINDS.filter((k) => {
                // Aposentadoria: só onde o país tem planos no catálogo.
                if (k === 'retirement') return plans.length > 0 || k === form.kind;
                const feature = COUNTRY_KINDS[k];
                return !feature || features[feature] || k === form.kind;
              }).map((k) => ({ value: k, label: ACCOUNT_KIND[k] }))}
            />
          </Field>
          <Field
            label={t('contas-moeda')}
            hint={editing ? t('contas-moeda-fixa') : t('contas-moeda-dica', { moeda: base })}
          >
            <Select
              value={form.currency}
              disabled={editing || form.kind === 'fgts'}
              onValueChange={(currency) => set({ currency })}
              options={currencyOptions(base).map((c) => ({ value: c, label: c }))}
            />
          </Field>
          {plan ? (
            <Field
              label={t('contas-plano-aposentadoria')}
              hint={t('contas-plano-aposentadoria-dica')}
            >
              <Select
                aria-label={t('contas-plano-aposentadoria')}
                value={plan.id}
                onValueChange={(retirementPlan) => set({ retirementPlan })}
                options={plans.map((p) => ({ value: p.id, label: p.name }))}
              />
            </Field>
          ) : null}
          {plan?.coverage ? (
            <Field label={t('contas-cobertura')}>
              <Select
                aria-label={t('contas-cobertura')}
                value={form.retirementCoverage}
                onValueChange={(retirementCoverage) => set({ retirementCoverage })}
                options={[
                  { value: 'self', label: t('contas-cobertura-individual') },
                  { value: 'family', label: t('contas-cobertura-familia') },
                ]}
              />
            </Field>
          ) : null}
          <Field label={t('contas-instituicao')}>
            {newInstitution !== null ? (
              <BankPicker
                value={newInstitution}
                code={newBankCode}
                onChange={(name, code) => {
                  setNewInstitution(name);
                  setNewBankCode(code);
                }}
                onCancel={() => {
                  setNewInstitution(null);
                  setNewBankCode(null);
                }}
              />
            ) : (
              <Select
                value={form.institutionId ?? NONE}
                onValueChange={(v) => {
                  if (v === NEW_INSTITUTION) setNewInstitution('');
                  else set({ institutionId: v === NONE ? null : v });
                }}
                options={institutionOptions}
              />
            )}
          </Field>
          <Field
            label={
              isCard
                ? t('contas-fatura-inicial', { moeda: form.currency })
                : t('contas-saldo-inicial-moeda', { moeda: form.currency })
            }
          >
            <MoneyInput
              currency={form.currency}
              allowNegative={!isCard}
              value={form.openingBalance}
              onValueChange={(openingBalance) => set({ openingBalance })}
            />
          </Field>
          <Field label={t('contas-na-data')}>
            <DateInput
              value={form.openingDate}
              onValueChange={(openingDate) => set({ openingDate })}
            />
          </Field>
          {hasBankRef ? (
            <>
              <Field label={t('contas-agencia')} hint={t('contas-agencia-dica')}>
                <Input
                  value={form.branch}
                  onChange={(e) => set({ branch: e.target.value })}
                  placeholder="0001"
                  inputMode="numeric"
                  numeric
                />
              </Field>
              <Field label={t('contas-numero-conta')} hint={t('contas-numero-conta-dica')}>
                <Input
                  value={form.accountNumber}
                  onChange={(e) => set({ accountNumber: e.target.value })}
                  placeholder="12345-6"
                  inputMode="numeric"
                  numeric
                />
              </Field>
            </>
          ) : null}
        </div>

        <fieldset className="flex flex-col gap-2">
          <legend className="label-caps mb-1">{t('contas-membro-familia')}</legend>
          <div className="flex flex-wrap gap-4">
            {members.map((m) => (
              <Checkbox
                key={m.id}
                label={m.name}
                checked={form.memberIds.includes(m.id)}
                onChange={(on) =>
                  set({
                    memberIds: on
                      ? [...form.memberIds, m.id]
                      : form.memberIds.filter((x) => x !== m.id),
                  })
                }
              />
            ))}
            {members.length === 0 ? (
              <span className="text-[12.5px] text-dim">{t('contas-sem-membros')}</span>
            ) : null}
          </div>
          {members.length ? (
            <p className="text-[12px] text-dim">{t('contas-membro-explicacao')}</p>
          ) : null}
        </fieldset>

        {isCard ? (
          <fieldset className="grid gap-3 rounded-card border border-line p-3 sm:grid-cols-3">
            <legend className="label-caps px-1">{t('contas-cartao')}</legend>
            <Field label={t('contas-limite-moeda', { moeda: form.currency })}>
              <MoneyInput
                currency={form.currency}
                value={form.limit}
                onValueChange={(limit) => set({ limit })}
              />
            </Field>
            <Field label={t('contas-dia-fechamento')}>
              <Input
                type="number"
                min={1}
                max={31}
                numeric
                value={form.closingDay}
                onChange={(e) => set({ closingDay: Number(e.target.value) })}
              />
            </Field>
            <Field
              label={t('contas-dia-vencimento')}
              hint={
                form.dueDay <= form.closingDay
                  ? t('contas-vence-mes-seguinte')
                  : t('contas-vence-mesmo-mes')
              }
            >
              <Input
                type="number"
                min={1}
                max={31}
                numeric
                value={form.dueDay}
                onChange={(e) => set({ dueDay: Number(e.target.value) })}
              />
            </Field>
            <Field label={t('contas-conta-pagadora')}>
              <Select
                value={form.paymentAccountId ?? NONE}
                onValueChange={(v) => set({ paymentAccountId: v === NONE ? null : v })}
                options={[
                  { value: NONE, label: '—' },
                  ...accounts
                    .filter((a) => a.kind !== 'credit_card' && a.id !== account?.id)
                    .map((a) => ({ value: a.id, label: a.name })),
                ]}
              />
            </Field>
            <Field label={t('contas-bandeira')}>
              <Input
                value={form.network}
                onChange={(e) => set({ network: e.target.value })}
                placeholder={t('contas-bandeira-exemplo')}
              />
            </Field>
            <Field label={t('contas-final-cartao')}>
              <Input
                value={form.last4}
                inputMode="numeric"
                maxLength={4}
                numeric
                onChange={(e) => set({ last4: e.target.value.replace(/\D/g, '') })}
                placeholder="1234"
              />
            </Field>
          </fieldset>
        ) : null}

        {canOverdraft ? (
          <OverdraftFields
            draft={form.overdraft}
            currency={form.currency}
            onChange={(p) => set({ overdraft: { ...form.overdraft, ...p } })}
          />
        ) : null}

        <Switch
          checked={form.includeInNetWorth}
          onChange={(includeInNetWorth) => set({ includeInNetWorth })}
          label={t('contas-entra-patrimonio')}
          description={t('contas-entra-patrimonio-dica')}
        />
        <Field label={t('contas-notas')}>
          <Input value={form.notes} onChange={(e) => set({ notes: e.target.value })} />
        </Field>
        <InlineError error={error} />
      </div>
      {account ? (
        <ConfirmDialog
          open={confirmDelete}
          onOpenChange={setConfirmDelete}
          title={t('contas-apagar-nome', { nome: account.name })}
          description={t('contas-apagar-descricao')}
          confirmLabel={t('contas-apagar-conta')}
          pending={remove.isPending}
          error={remove.error}
          onConfirm={() =>
            remove.mutate(
              { id: account.id },
              {
                onSuccess: () => {
                  toast({
                    tone: 'success',
                    title: t('contas-conta-apagada'),
                    description: account.name,
                  });
                  setConfirmDelete(false);
                  onOpenChange(false);
                },
              },
            )
          }
        />
      ) : null}
    </Dialog>
  );
}
