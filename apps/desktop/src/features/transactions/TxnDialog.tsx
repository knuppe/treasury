import { useMemo, useState, type FormEvent, type ReactNode } from 'react';

import { isTauri } from '@/api/client';
import type { AccountView } from '@/api/generated/AccountView';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnPatch } from '@/api/generated/TxnPatch';
import type { TxnStatus } from '@/api/generated/TxnStatus';
import type { TxnView } from '@/api/generated/TxnView';
import { useAccounts } from '@/api/hooks/accounts';
import { useCardPlastics, useCreateInstallments } from '@/api/hooks/cards';
import { plasticLabel } from '@/features/cards/Plastics';
import { useMembers } from '@/api/hooks/household';
import { useProjects } from '@/api/hooks/projects';
import {
  fetchAttachment,
  saveAttachment,
  useAddAttachment,
  useAttachments,
  useCreateTransfer,
  useCreateTxn,
  useDeleteAttachment,
  useUpdateTxn,
} from '@/api/hooks/transactions';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Icon,
  IconButton,
  Input,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { ProjectSelect } from '@/features/projects/ProjectDialogs';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';
import { formatMoney, todayIso } from '@/lib/format';
import { chooseSavePath, extensionOf } from '@/lib/saveDialog';

import { newPart, splitBalance, SplitEditor, type SplitPart } from './SplitEditor';
import { TagInput } from './TagInput';
import { CategoryPicker } from './ui/CategoryPicker';
import { BalanceAdjustForm } from '@/features/accounts/BalanceAdjust';

import { Checkbox, FileInput, InlineError, Segmented, TextArea } from './ui/controls';
import { downloadBase64, fileToBase64, formatBytes } from './ui/hooks';
import { TXN_KIND, TXN_STATUS } from './ui/labels';
import { DescriptionInput } from './ui/DescriptionInput';

/** `adjustment`: ajuste de saldo numa data (accounts.sync_balance), fora dos relatórios. */
export type TxnDialogMode = 'expense' | 'income' | 'transfer' | 'installments' | 'adjustment';

export interface TxnDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Lançamento a editar; sem ele, cria um novo. */
  txn?: TxnView | null;
  initialMode?: TxnDialogMode;
  initialAccountId?: string;
  onSaved?: (txn: TxnView | null) => void;
  /** "Criar regra a partir deste lançamento". */
  onSuggestRule?: (txn: TxnView) => void;
}

const NO_MEMBER = '__none__';
const NO_PLASTIC = '__none__';
const NO_LINK = '__no_link__';

const TRANSFER_KINDS = ['transfer', 'investment', 'card_payment'] as const;

function modeOf(txn: TxnView): TxnDialogMode {
  if (txn.transfer_id) return 'transfer';
  return txn.amount < 0 ? 'expense' : 'income';
}

interface FormState {
  mode: TxnDialogMode;
  accountId: string;
  toAccountId: string;
  date: string | null;
  amount: number | null;
  toAmount: number | null;
  description: string;
  payee: string;
  notes: string;
  categoryId: string | null;
  memberId: string | null;
  status: TxnStatus;
  tags: string[];
  split: boolean;
  parts: SplitPart[];
  transferKind: 'transfer' | 'investment' | 'card_payment';
  installments: number;
  /** Edição de lançamento solto: marcado como transferência (para terceiros ou outra conta). */
  asTransfer: boolean;
  /** Outra conta da família da transferência (opcional): liga ou cria a outra perna. */
  linkAccountId: string;
  /** Valor na outra conta quando ela usa outra moeda. */
  linkAmount: number | null;
  /** Projeto (obra, reforma, viagem…). */
  projectId: string | null;
  /** Cartão usado (titular, adicional, virtual), em conta de cartão. */
  plasticId: string | null;
  /** Compra no exterior: valor positivo e moeda de origem. */
  origAmount: number | null;
  origCurrency: string;
}

function initialState(
  txn: TxnView | null | undefined,
  mode: TxnDialogMode,
  accountId: string,
): FormState {
  if (txn) {
    return {
      mode: modeOf(txn),
      accountId: txn.account_id,
      toAccountId: txn.counterpart_account_id ?? '',
      date: txn.date,
      amount: Math.abs(txn.amount),
      toAmount: null,
      description: txn.description,
      payee: txn.payee ?? '',
      notes: txn.notes ?? '',
      categoryId: txn.category_id,
      memberId: txn.member_id,
      plasticId: txn.card_plastic_id,
      origAmount: txn.orig_amount == null ? null : Math.abs(txn.orig_amount),
      origCurrency: txn.orig_currency ?? '',
      status: txn.status,
      tags: txn.tags.map((tag) => tag.name),
      split: txn.splits.length > 0,
      parts: txn.splits.map((s) =>
        newPart({
          category_id: s.category_id,
          member_id: s.member_id,
          amount: Math.abs(s.amount),
          notes: s.notes ?? '',
        }),
      ),
      transferKind:
        txn.kind === 'investment' || txn.kind === 'card_payment' ? txn.kind : 'transfer',
      installments: 2,
      asTransfer: txn.kind === 'transfer' && !txn.transfer_id,
      linkAccountId: '',
      linkAmount: null,
      projectId: txn.project_id,
    };
  }
  return {
    mode,
    accountId,
    toAccountId: '',
    date: todayIso(),
    amount: null,
    toAmount: null,
    description: '',
    payee: '',
    notes: '',
    categoryId: null,
    memberId: null,
    plasticId: null,
    origAmount: null,
    origCurrency: '',
    status: 'cleared',
    tags: [],
    split: false,
    parts: [newPart(), newPart()],
    transferKind: 'transfer',
    installments: 2,
    asTransfer: false,
    linkAccountId: '',
    linkAmount: null,
    projectId: null,
  };
}

function Row({ children }: { children: ReactNode }) {
  return <div className="grid gap-3 sm:grid-cols-2">{children}</div>;
}

/**
 * Criar ou editar um lançamento: despesa, receita, transferência (inclusive
 * entre moedas) ou compra parcelada no cartão; divisão em categorias/membros,
 * tags, notas e anexos.
 */
export function TxnDialog({
  open,
  onOpenChange,
  txn,
  initialMode = 'expense',
  initialAccountId,
  onSaved,
  onSuggestRule,
}: TxnDialogProps) {
  const { data: accounts = [] } = useAccounts(true);
  const { data: members = [] } = useMembers();
  const base = useBaseCurrency();
  const defaultAccount =
    initialAccountId ??
    accounts.find((a) => !a.archived && a.kind === 'checking')?.id ??
    accounts[0]?.id ??
    '';
  const [form, setForm] = useState<FormState>(() => initialState(txn, initialMode, defaultAccount));
  const isCardAccount = accounts.find((a) => a.id === form.accountId)?.kind === 'credit_card';
  const plasticsQuery = useCardPlastics(isCardAccount ? form.accountId : null);
  const plastics = plasticsQuery.data ?? [];
  const accountCurrency = accounts.find((a) => a.id === form.accountId)?.currency;
  const origCurrency = /^[A-Z]{3}$/.test(form.origCurrency) ? form.origCurrency : '';
  // O cartão escolhido só vale na conta dele (ao trocar de conta, nenhum).
  const plasticId = plastics.some((p) => p.id === form.plasticId) ? form.plasticId : null;
  const plasticOptions = [
    { value: NO_PLASTIC, label: t('lancamentos-cartao-nao-informado') },
    ...plastics
      .filter((p) => p.active || p.id === form.plasticId)
      .map((p) => ({
        value: p.id,
        label: p.member_name ? `${plasticLabel(p)} · ${p.member_name}` : plasticLabel(p),
      })),
  ];
  const [key, setKey] = useState(`${txn?.id ?? 'new'}:${String(open)}`);
  const [error, setError] = useState<unknown>(null);
  const [files, setFiles] = useState<File[]>([]);
  const nextKey = `${txn?.id ?? 'new'}:${String(open)}`;
  if (nextKey !== key) {
    setKey(nextKey);
    setForm(initialState(txn, initialMode, defaultAccount));
    setError(null);
    setFiles([]);
  }
  if (!form.accountId && defaultAccount && !txn)
    setForm((f) => ({ ...f, accountId: defaultAccount }));

  const createTxn = useCreateTxn();
  const updateTxn = useUpdateTxn();
  const createTransfer = useCreateTransfer();
  const createInstallments = useCreateInstallments();
  const addAttachment = useAddAttachment();
  const pending =
    createTxn.isPending ||
    updateTxn.isPending ||
    createTransfer.isPending ||
    createInstallments.isPending;

  const set = (patch: Partial<FormState>) => setForm((f) => ({ ...f, ...patch }));
  const { data: projects = [] } = useProjects();
  const hasProjects = projects.length > 0 || Boolean(txn?.project_id);
  const editing = Boolean(txn);
  const isLeg = Boolean(txn?.transfer_id);
  const account = accounts.find((a) => a.id === form.accountId);
  const toAccount = accounts.find((a) => a.id === form.toAccountId);
  const currency = account?.currency ?? base;
  const crossCurrency = Boolean(account && toAccount && account.currency !== toAccount.currency);
  const cashFlowKind: 'income' | 'expense' = form.mode === 'income' ? 'income' : 'expense';
  const kindOfEdit: TxnKind | undefined = txn?.kind;
  // Lançamento solto (receita, despesa ou transferência de uma perna) pode virar transferência.
  const canBeTransfer =
    editing &&
    !isLeg &&
    (kindOfEdit === 'income' || kindOfEdit === 'expense' || kindOfEdit === 'transfer');
  const asTransfer = canBeTransfer && form.asTransfer;
  const linkAccount = accounts.find((a) => a.id === form.linkAccountId);
  const linkCrossCurrency = Boolean(linkAccount && linkAccount.currency !== currency);
  const usesCategory =
    !isLeg &&
    (form.mode === 'expense' || form.mode === 'income' || form.mode === 'installments') &&
    (kindOfEdit === undefined ||
      kindOfEdit === 'income' ||
      kindOfEdit === 'expense' ||
      kindOfEdit === 'transfer');
  const canSplit = usesCategory && !asTransfer;

  const accountOptions = (filter: (a: AccountView) => boolean) =>
    accounts
      .filter(
        (a) => (!a.archived || a.id === form.accountId || a.id === form.toAccountId) && filter(a),
      )
      .map((a) => ({ value: a.id, label: a.name, hint: a.currency }));
  // Sem membro escolhido, o núcleo dá ao lançamento novo o membro do cartão usado ou, sem ele, o
  // da conta quando ela é de um só.
  const accountOwners = accounts.find((a) => a.id === form.accountId)?.member_ids ?? [];
  const plasticMember = plastics.find((p) => p.id === plasticId)?.member_name;
  const inherited = editing
    ? undefined
    : (plasticMember ??
      (accountOwners.length === 1
        ? members.find((m) => m.id === accountOwners[0])?.name
        : undefined));
  const memberOptions = [
    {
      value: NO_MEMBER,
      label: inherited
        ? t('lancamentos-membro-da-conta', { nome: inherited })
        : t('lancamentos-familia'),
    },
    ...members.map((m) => ({ value: m.id, label: m.name })),
  ];

  const { missing } = splitBalance(form.amount, form.parts);
  const installmentPreview = useMemo(() => {
    if (form.mode !== 'installments' || !form.amount || form.installments < 1) return null;
    const base = Math.floor(form.amount / form.installments);
    const first = base + (form.amount - base * form.installments);
    return { base, first };
  }, [form.mode, form.amount, form.installments]);

  const validate = (): string | null => {
    if (!form.accountId) return t('lancamentos-erro-conta');
    if (!form.date) return t('lancamentos-erro-data');
    if (!form.amount || form.amount <= 0) return t('lancamentos-erro-valor');
    if (form.mode === 'transfer' && !editing) {
      if (!form.toAccountId) return t('lancamentos-erro-destino');
      if (form.toAccountId === form.accountId) return t('lancamentos-erro-mesma-conta');
      if (crossCurrency && !form.toAmount)
        return t('lancamentos-erro-valor-destino', {
          conta: toAccount?.name ?? '',
          moeda: toAccount?.currency ?? '',
        });
    }
    if (form.mode === 'installments') {
      if (!form.description.trim()) return t('lancamentos-erro-descricao-compra');
      if (!(form.installments >= 1 && form.installments <= 120))
        return t('lancamentos-erro-parcelas');
    }
    if (asTransfer && linkCrossCurrency && !form.linkAmount)
      return t('lancamentos-erro-valor-ligado', {
        conta: linkAccount?.name ?? '',
        moeda: linkAccount?.currency ?? '',
      });
    if (form.split && canSplit) {
      if (form.parts.length < 2) return t('lancamentos-erro-duas-partes');
      if (form.parts.some((p) => !p.amount)) return t('lancamentos-erro-valor-parte');
      if (missing !== 0) return t('lancamentos-erro-soma-partes');
    }
    return null;
  };

  const signed = (cents: number) =>
    form.mode === 'expense' || (editing && (txn?.amount ?? 0) < 0) ? -cents : cents;

  const uploadFiles = async (txnId: string) => {
    for (const file of files) {
      await addAttachment.mutateAsync({
        txn_id: txnId,
        filename: file.name,
        mime: file.type || 'application/octet-stream',
        content_base64: await fileToBase64(file),
      });
    }
  };

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const problem = validate();
    if (problem) {
      setError(new Error(problem));
      return;
    }
    setError(null);
    const amount = form.amount!;
    try {
      let saved: TxnView | null = null;
      if (!editing && form.mode === 'transfer') {
        const res = await createTransfer.mutateAsync({
          from_account_id: form.accountId,
          to_account_id: form.toAccountId,
          date: form.date!,
          amount,
          ...(crossCurrency && form.toAmount ? { to_amount: form.toAmount } : {}),
          ...(form.description.trim() ? { description: form.description.trim() } : {}),
          kind: form.transferKind,
          status: form.status,
          ...(form.memberId ? { member_id: form.memberId } : {}),
        });
        saved = res.from;
        toast({
          tone: 'success',
          title: t('lancamentos-tipo-registrado', { tipo: form.transferKind }),
          description: `${res.from.account_name} → ${res.to.account_name}`,
        });
      } else if (!editing && form.mode === 'installments') {
        const plan = await createInstallments.mutateAsync({
          account_id: form.accountId,
          description: form.description.trim(),
          total_amount: amount,
          installments: form.installments,
          purchase_date: form.date!,
          ...(form.categoryId ? { category_id: form.categoryId } : {}),
          ...(form.memberId ? { member_id: form.memberId } : {}),
          ...(form.tags.length ? { tags: form.tags } : {}),
        });
        toast({
          tone: 'success',
          title: t('lancamentos-parcelada-registrada'),
          description: `${plan.installments}× · ${plan.description}`,
        });
      } else if (!editing) {
        saved = await createTxn.mutateAsync({
          account_id: form.accountId,
          date: form.date!,
          amount: signed(amount),
          kind: cashFlowKind,
          status: form.status,
          description: form.description.trim(),
          ...(form.payee.trim() ? { payee: form.payee.trim() } : {}),
          ...(form.notes.trim() ? { notes: form.notes.trim() } : {}),
          ...(form.memberId ? { member_id: form.memberId } : {}),
          ...(form.split
            ? {
                splits: form.parts.map((p) => ({
                  amount: signed(p.amount!),
                  ...(p.category_id ? { category_id: p.category_id } : {}),
                  ...(p.member_id ? { member_id: p.member_id } : {}),
                  ...(p.notes.trim() ? { notes: p.notes.trim() } : {}),
                })),
              }
            : form.categoryId
              ? { category_id: form.categoryId }
              : {}),
          ...(form.tags.length ? { tags: form.tags } : {}),
          ...(form.projectId ? { project_id: form.projectId } : {}),
          ...(plasticId ? { card_plastic_id: plasticId } : {}),
          ...(origCurrency && form.origAmount
            ? { orig_amount: form.origAmount, orig_currency: origCurrency }
            : {}),
        });
        await uploadFiles(saved.id);
        toast({
          tone: 'success',
          title: t('lancamentos-criado'),
          description: saved.description || undefined,
        });
      } else if (txn) {
        const patch: TxnPatch = { id: txn.id };
        if (form.date !== txn.date) patch.date = form.date!;
        const newAmount = txn.amount < 0 ? -amount : amount;
        if (newAmount !== txn.amount) patch.amount = newAmount;
        if (form.description.trim() !== txn.description)
          patch.description = form.description.trim();
        if (form.status !== txn.status) patch.status = form.status;
        if ((form.memberId ?? null) !== txn.member_id) patch.member_id = form.memberId;
        if (form.projectId !== txn.project_id) patch.project_id = form.projectId;
        if (
          form.origAmount !== (txn.orig_amount == null ? null : Math.abs(txn.orig_amount)) ||
          (origCurrency || null) !== txn.orig_currency
        ) {
          if (origCurrency && form.origAmount) {
            patch.orig_amount = form.origAmount;
            patch.orig_currency = origCurrency;
          } else {
            patch.orig_amount = null;
          }
        }
        if (
          plasticsQuery.isSuccess &&
          form.accountId === txn.account_id &&
          plasticId !== txn.card_plastic_id
        )
          patch.card_plastic_id = plasticId;
        if (!isLeg) {
          if (form.accountId !== txn.account_id) patch.account_id = form.accountId;
          if ((form.payee.trim() || null) !== txn.payee) patch.payee = form.payee.trim() || null;
          if ((form.notes.trim() || null) !== txn.notes) patch.notes = form.notes.trim() || null;
          const oldTags = txn.tags.map((tag) => tag.name).join('\u0000');
          if (form.tags.join('\u0000') !== oldTags) patch.tags = form.tags;
          const kind: TxnKind = asTransfer ? 'transfer' : cashFlowKind;
          if (canBeTransfer && kind !== txn.kind) patch.kind = kind;
          if (asTransfer && form.linkAccountId) {
            patch.transfer_account_id = form.linkAccountId;
            if (linkCrossCurrency && form.linkAmount) patch.transfer_amount = form.linkAmount;
          }
          if (usesCategory) {
            if (form.split && canSplit) {
              patch.splits = form.parts.map((p) => ({
                amount: newAmount < 0 ? -p.amount! : p.amount!,
                ...(p.category_id ? { category_id: p.category_id } : {}),
                ...(p.member_id ? { member_id: p.member_id } : {}),
                ...(p.notes.trim() ? { notes: p.notes.trim() } : {}),
              }));
              if (txn.category_id) patch.category_id = null;
            } else {
              if (txn.splits.length > 0) patch.splits = [];
              if (form.categoryId !== txn.category_id) patch.category_id = form.categoryId;
            }
          }
        }
        saved = await updateTxn.mutateAsync(patch);
        toast({ tone: 'success', title: t('lancamentos-atualizado') });
      }
      onSaved?.(saved);
      onOpenChange(false);
    } catch (err) {
      setError(err);
    }
  };

  const adjusting = !editing && form.mode === 'adjustment';
  const title = editing
    ? isLeg
      ? t('lancamentos-editar-tipo', { tipo: TXN_KIND[txn!.kind].toLowerCase() })
      : t('lancamentos-editar')
    : t('lancamentos-novo');

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={title}
      index="03"
      size="lg"
      footer={
        <>
          {editing && txn && onSuggestRule && !isLeg ? (
            <Button variant="ghost" className="mr-auto" onClick={() => onSuggestRule(txn)}>
              {t('lancamentos-criar-regra-deste')}
            </Button>
          ) : null}
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            type="submit"
            form={adjusting ? 'txn-adjust-form' : 'txn-form'}
            loading={pending}
            loadingText={t('lancamentos-salvando')}
          >
            {editing
              ? t('lancamentos-salvar-alteracoes')
              : adjusting
                ? t('lancamentos-ajustar')
                : t('lancamentos-lancar')}
          </Button>
        </>
      }
    >
      <form
        id="txn-form"
        className="flex flex-col gap-4"
        onSubmit={(e) => void submit(e)}
        noValidate
      >
        {!editing ? (
          <Segmented<TxnDialogMode>
            label={t('lancamentos-tipo-lancamento')}
            value={form.mode}
            onChange={(mode) => {
              const card = accounts.find((a) => a.kind === 'credit_card' && !a.archived);
              set({
                mode,
                ...(mode === 'installments' && account?.kind !== 'credit_card' && card
                  ? { accountId: card.id }
                  : {}),
              });
            }}
            options={[
              { value: 'expense', label: TXN_KIND.expense },
              { value: 'income', label: TXN_KIND.income },
              { value: 'transfer', label: TXN_KIND.transfer },
              {
                value: 'installments',
                label: t('lancamentos-parcelado-cartao'),
                disabled: !accounts.some((a) => a.kind === 'credit_card'),
              },
              { value: 'adjustment', label: t('lancamentos-ajuste-saldo') },
            ]}
          />
        ) : (
          <p className="label-caps">
            {`${TXN_KIND[txn!.kind]} · ${txn!.account_name}`}
            {txn!.counterpart_account_name
              ? ` ${txn!.amount < 0 ? '→' : '←'} ${txn!.counterpart_account_name}`
              : ''}
          </p>
        )}

        {adjusting ? null : (
          <>
            {form.mode === 'transfer' && !editing ? (
              <>
                <Row>
                  <Field label={t('lancamentos-de-origem')}>
                    <Select
                      value={form.accountId || undefined}
                      onValueChange={(v) => set({ accountId: v })}
                      options={accountOptions(() => true)}
                    />
                  </Field>
                  <Field label={t('lancamentos-para-destino')}>
                    <Select
                      value={form.toAccountId || undefined}
                      placeholder={t('lancamentos-escolha-conta')}
                      onValueChange={(v) => {
                        const target = accounts.find((a) => a.id === v);
                        set({
                          toAccountId: v,
                          ...(target?.kind === 'credit_card'
                            ? { transferKind: 'card_payment' }
                            : {}),
                        });
                      }}
                      options={accountOptions((a) => a.id !== form.accountId)}
                    />
                  </Field>
                </Row>
                <Row>
                  <Field label={t('lancamentos-valor-sai', { moeda: currency })}>
                    <MoneyInput
                      currency={currency}
                      value={form.amount}
                      onValueChange={(amount) => set({ amount })}
                    />
                  </Field>
                  {crossCurrency ? (
                    <Field
                      label={t('lancamentos-valor-entra', { moeda: toAccount!.currency })}
                      hint={
                        form.amount && form.toAmount
                          ? t('lancamentos-cambio-implicito', {
                              moeda: toAccount!.currency,
                              taxa: form.amount / form.toAmount,
                              origem: currency,
                            })
                          : t('lancamentos-moedas-diferentes')
                      }
                    >
                      <MoneyInput
                        currency={toAccount!.currency}
                        value={form.toAmount}
                        onValueChange={(toAmount) => set({ toAmount })}
                      />
                    </Field>
                  ) : (
                    <Field label={t('lancamentos-campo-tipo')}>
                      <Select
                        value={form.transferKind}
                        onValueChange={(v) => set({ transferKind: v as FormState['transferKind'] })}
                        options={TRANSFER_KINDS.map((k) => ({
                          value: k,
                          label: TXN_KIND[k],
                          disabled: k === 'card_payment' && toAccount?.kind !== 'credit_card',
                        }))}
                      />
                    </Field>
                  )}
                </Row>
                <Row>
                  <Field label={t('lancamentos-campo-data')}>
                    <DateInput value={form.date} onValueChange={(date) => set({ date })} />
                  </Field>
                  <Field
                    label={t('lancamentos-campo-descricao')}
                    hint={t('lancamentos-descricao-transferencia-dica')}
                  >
                    <Input
                      value={form.description}
                      onChange={(e) => set({ description: e.target.value })}
                    />
                  </Field>
                </Row>
              </>
            ) : (
              <>
                <Row>
                  <Field
                    label={
                      form.mode === 'installments'
                        ? t('lancamentos-cartao')
                        : t('lancamentos-campo-conta')
                    }
                  >
                    <Select
                      value={form.accountId || undefined}
                      onValueChange={(v) => set({ accountId: v })}
                      options={accountOptions((a) =>
                        form.mode === 'installments'
                          ? a.kind === 'credit_card'
                          : !editing || a.currency === txn!.currency,
                      )}
                      disabled={isLeg}
                    />
                  </Field>
                  <Field
                    label={
                      form.mode === 'installments'
                        ? t('lancamentos-data-compra')
                        : t('lancamentos-campo-data')
                    }
                  >
                    <DateInput value={form.date} onValueChange={(date) => set({ date })} />
                  </Field>
                </Row>
                <Row>
                  <Field label={t('lancamentos-campo-descricao')}>
                    <DescriptionInput
                      value={form.description}
                      onChange={(description) => set({ description })}
                      enabled={!editing}
                      accountId={form.accountId || undefined}
                      kind={form.mode === 'income' ? 'income' : 'expense'}
                      onPick={(sug) =>
                        setForm((f) => ({
                          ...f,
                          description: sug.description,
                          payee: f.payee || sug.payee || '',
                          categoryId: f.categoryId ?? sug.category_id,
                          amount: f.amount ?? Math.abs(sug.last_amount),
                        }))
                      }
                      placeholder={
                        form.mode === 'income'
                          ? t('lancamentos-exemplo-receita')
                          : t('lancamentos-exemplo-despesa')
                      }
                      autoFocus={!editing}
                    />
                  </Field>
                  <Field
                    label={
                      form.mode === 'installments'
                        ? t('lancamentos-valor-total-moeda', { moeda: currency })
                        : t('lancamentos-valor-moeda', { moeda: currency })
                    }
                    hint={
                      form.mode === 'installments' && installmentPreview
                        ? installmentPreview.first !== installmentPreview.base
                          ? t('lancamentos-parcelas-previa-primeira', {
                              n: form.installments,
                              valor: formatMoney(installmentPreview.base, currency),
                              primeira: formatMoney(installmentPreview.first, currency),
                            })
                          : t('lancamentos-parcelas-previa', {
                              n: form.installments,
                              valor: formatMoney(installmentPreview.base, currency),
                            })
                        : undefined
                    }
                  >
                    <MoneyInput
                      currency={currency}
                      value={form.amount}
                      onValueChange={(amount) => set({ amount })}
                    />
                  </Field>
                </Row>
                {form.mode === 'installments' ? (
                  <Row>
                    <Field label={t('lancamentos-parcelas')} hint={t('lancamentos-parcelas-dica')}>
                      <Input
                        type="number"
                        min={1}
                        max={120}
                        numeric
                        value={form.installments}
                        onChange={(e) => set({ installments: Number(e.target.value) })}
                      />
                    </Field>
                    <span />
                  </Row>
                ) : null}
                {canBeTransfer ? (
                  <Field
                    label={t('lancamentos-campo-tipo')}
                    hint={asTransfer ? t('lancamentos-como-transferencia-dica') : undefined}
                  >
                    <Segmented<'cash' | 'transfer'>
                      label={t('lancamentos-tipo-do-lancamento')}
                      value={asTransfer ? 'transfer' : 'cash'}
                      onChange={(v) => set({ asTransfer: v === 'transfer' })}
                      options={[
                        {
                          value: 'cash',
                          label: (txn?.amount ?? 0) < 0 ? TXN_KIND.expense : TXN_KIND.income,
                        },
                        { value: 'transfer', label: TXN_KIND.transfer },
                      ]}
                    />
                  </Field>
                ) : null}
                {asTransfer ? (
                  <Row>
                    <Field
                      label={
                        (txn?.amount ?? 0) < 0
                          ? t('lancamentos-para-conta-familia')
                          : t('lancamentos-da-conta-familia')
                      }
                      hint={t('lancamentos-ligar-dica')}
                    >
                      <Select
                        value={form.linkAccountId || NO_LINK}
                        onValueChange={(v) =>
                          set({ linkAccountId: v === NO_LINK ? '' : v, linkAmount: null })
                        }
                        options={[
                          { value: NO_LINK, label: t('lancamentos-nenhuma-terceiros') },
                          ...accountOptions((a) => a.id !== form.accountId && !a.archived),
                        ]}
                      />
                    </Field>
                    {linkCrossCurrency && linkAccount ? (
                      <Field
                        label={t('lancamentos-valor-em', {
                          conta: linkAccount.name,
                          moeda: linkAccount.currency,
                        })}
                      >
                        <MoneyInput
                          currency={linkAccount.currency}
                          value={form.linkAmount}
                          onValueChange={(linkAmount) => set({ linkAmount })}
                        />
                      </Field>
                    ) : (
                      <span />
                    )}
                  </Row>
                ) : null}
                {form.mode !== 'installments' && !isLeg && !(asTransfer && form.linkAccountId) ? (
                  <Field
                    label={asTransfer ? t('lancamentos-destinatario') : t('lancamentos-favorecido')}
                    hint={
                      asTransfer
                        ? t('lancamentos-destinatario-dica')
                        : t('lancamentos-favorecido-dica')
                    }
                  >
                    <Input value={form.payee} onChange={(e) => set({ payee: e.target.value })} />
                  </Field>
                ) : null}
              </>
            )}

            <Row>
              {usesCategory && !(form.split && canSplit) ? (
                <Field label={t('lancamentos-campo-categoria')}>
                  <CategoryPicker
                    kind={cashFlowKind}
                    value={form.categoryId}
                    onChange={(categoryId) => set({ categoryId })}
                    className="[&>button]:h-9 [&>button]:w-full"
                  />
                </Field>
              ) : (
                <span />
              )}
              <Field label={t('lancamentos-campo-membro')}>
                <Select
                  value={form.memberId ?? NO_MEMBER}
                  onValueChange={(v) => set({ memberId: v === NO_MEMBER ? null : v })}
                  options={memberOptions}
                />
              </Field>
            </Row>

            {isCardAccount && form.mode === 'expense' ? (
              <Row>
                <Field
                  label={t('lancamentos-campo-moeda-origem')}
                  hint={t('lancamentos-moeda-origem-dica')}
                >
                  <Input
                    value={form.origCurrency}
                    maxLength={3}
                    placeholder="USD"
                    onChange={(e) =>
                      set({ origCurrency: e.target.value.replace(/[^a-z]/gi, '').toUpperCase() })
                    }
                  />
                </Field>
                <Field label={t('lancamentos-campo-valor-origem')}>
                  <MoneyInput
                    currency={
                      origCurrency && origCurrency !== accountCurrency ? origCurrency : 'USD'
                    }
                    value={form.origAmount}
                    onValueChange={(v) => set({ origAmount: v })}
                    disabled={!origCurrency}
                  />
                </Field>
              </Row>
            ) : null}

            {isCardAccount && plastics.length > 1 && form.mode !== 'installments' ? (
              <Field label={t('lancamentos-campo-cartao')} hint={t('lancamentos-cartao-dica')}>
                <Select
                  value={plasticId ?? NO_PLASTIC}
                  onValueChange={(v) => set({ plasticId: v === NO_PLASTIC ? null : v })}
                  options={plasticOptions}
                />
              </Field>
            ) : null}

            {hasProjects &&
            (editing || form.mode !== 'transfer') &&
            form.mode !== 'installments' ? (
              <Field label={t('lancamentos-campo-projeto')} hint={t('lancamentos-projeto-dica')}>
                <ProjectSelect
                  value={form.projectId}
                  onChange={(v) => set({ projectId: v === 'new' ? null : v })}
                />
              </Field>
            ) : null}

            {form.mode !== 'installments' ? (
              <Field label={t('lancamentos-campo-status')}>
                <Segmented<TxnStatus>
                  label={t('lancamentos-campo-status')}
                  value={form.status}
                  onChange={(status) => set({ status })}
                  options={[
                    { value: 'planned', label: TXN_STATUS.planned },
                    { value: 'cleared', label: TXN_STATUS.cleared },
                    { value: 'reconciled', label: TXN_STATUS.reconciled, disabled: !editing },
                  ]}
                />
              </Field>
            ) : null}

            {canSplit && form.mode !== 'installments' ? (
              <>
                <Checkbox
                  label={t('lancamentos-dividir')}
                  checked={form.split}
                  onChange={(split) =>
                    set({
                      split,
                      parts:
                        split && form.parts.every((p) => !p.amount)
                          ? [
                              newPart({ category_id: form.categoryId, amount: form.amount }),
                              newPart(),
                            ]
                          : form.parts,
                    })
                  }
                />
                {form.split ? (
                  <SplitEditor
                    parts={form.parts}
                    onChange={(parts) => set({ parts })}
                    total={form.amount}
                    currency={currency}
                    kind={cashFlowKind}
                    members={members}
                  />
                ) : null}
              </>
            ) : null}

            {!isLeg ? (
              <Row>
                <Field label={t('lancamentos-campo-tags')}>
                  <TagInput value={form.tags} onChange={(tags) => set({ tags })} />
                </Field>
                {form.mode !== 'installments' ? (
                  <Field label={t('lancamentos-campo-notas')}>
                    <TextArea
                      value={form.notes}
                      onChange={(e) => set({ notes: e.target.value })}
                      rows={2}
                    />
                  </Field>
                ) : (
                  <span />
                )}
              </Row>
            ) : null}

            {form.mode === 'expense' || form.mode === 'income' ? (
              txn ? (
                <Attachments txnId={txn.id} />
              ) : (
                <Field label={t('lancamentos-anexos')} hint={t('lancamentos-anexos-dica')}>
                  <FileInput
                    multiple
                    accept="application/pdf,image/*"
                    onChange={(e) => setFiles([...(e.target.files ?? [])])}
                  />
                </Field>
              )
            ) : null}

            <InlineError error={error} />
          </>
        )}
      </form>
      {adjusting ? (
        <BalanceAdjustForm
          formId="txn-adjust-form"
          accounts={accounts}
          accountId={form.accountId}
          onAccountChange={(id) => set({ accountId: id })}
          onDone={() => onOpenChange(false)}
        />
      ) : null}
    </Dialog>
  );
}

function Attachments({ txnId }: { txnId: string }) {
  const { data: list = [] } = useAttachments(txnId);
  const add = useAddAttachment();
  const remove = useDeleteAttachment();
  const [error, setError] = useState<unknown>(null);
  // No app, pergunta onde salvar e o núcleo grava (link de Blob não baixa no WKWebView); no
  // navegador, baixa como antes.
  const openAttachment = async (a: { id: string; filename: string }) => {
    try {
      if (!isTauri()) {
        const c = await fetchAttachment(a.id);
        downloadBase64(c.content_base64, c.filename, c.mime);
        return;
      }
      const ext = extensionOf(a.filename);
      const path = await chooseSavePath(
        t('lancamentos-anexo-salvar', { nome: a.filename }),
        a.filename,
        ext ? [{ name: ext.toUpperCase(), extensions: [ext] }] : undefined,
      );
      if (!path) return;
      const r = await saveAttachment(a.id, path);
      toast({ tone: 'success', title: t('lancamentos-anexo-salvo'), description: r.path });
    } catch (e) {
      setError(e);
    }
  };
  return (
    <fieldset className="flex flex-col gap-2 rounded-card border border-line p-3">
      <legend className="label-caps px-1">{t('lancamentos-anexos')}</legend>
      {list.length === 0 ? (
        <p className="text-[12.5px] text-dim">{t('lancamentos-nenhum-anexo')}</p>
      ) : null}
      <ul className="flex flex-col gap-1">
        {list.map((a) => (
          <li key={a.id} className="flex items-center gap-2 text-[13px]">
            <button
              type="button"
              className="min-w-0 flex-1 truncate text-left text-accent hover:underline"
              onClick={() => void openAttachment(a)}
            >
              {a.filename}
            </button>
            <span className="num text-[11px] text-dim">{formatBytes(a.size_bytes)}</span>
            <IconButton
              size="sm"
              label={t('lancamentos-apagar-item', { nome: a.filename })}
              icon={<Icon name="x" size={12} />}
              tooltip={false}
              onClick={() => remove.mutate({ id: a.id }, { onError: setError })}
            />
          </li>
        ))}
      </ul>
      <input
        type="file"
        aria-label={t('lancamentos-anexar')}
        accept="application/pdf,image/*"
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (!file) return;
          void fileToBase64(file)
            .then((content_base64) =>
              add.mutateAsync({
                txn_id: txnId,
                filename: file.name,
                mime: file.type || 'application/octet-stream',
                content_base64,
              }),
            )
            .catch(setError);
          e.target.value = '';
        }}
        className="text-[12.5px] text-dim file:mr-3 file:h-8 file:rounded-control file:border file:border-line-strong file:bg-surface-2 file:px-3 file:text-fg"
      />
      <InlineError error={error} />
    </fieldset>
  );
}
