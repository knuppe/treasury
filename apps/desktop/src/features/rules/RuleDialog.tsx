import { useState } from 'react';

import type { Rule } from '@/api/generated/Rule';
import type { RuleCondition } from '@/api/generated/RuleCondition';
import type { RuleField } from '@/api/generated/RuleField';
import type { RuleInput } from '@/api/generated/RuleInput';
import type { RuleOp } from '@/api/generated/RuleOp';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnView } from '@/api/generated/TxnView';
import { useAccounts } from '@/api/hooks/accounts';
import { useMembers } from '@/api/hooks/household';
import { testRule, useCreateRule, useUpdateRule } from '@/api/hooks/rules';
import {
  Button,
  Dialog,
  Field,
  Icon,
  IconButton,
  Input,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { formatDate, formatNumber } from '@/lib/format';
import { TagInput } from '@/features/transactions/TagInput';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';
import { InlineError, Segmented } from '@/features/transactions/ui/controls';
import { RULE_FIELD, RULE_OP, TXN_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';

const TEXT_OPS: RuleOp[] = ['contains', 'equals', 'starts_with', 'ends_with', 'regex'];
const AMOUNT_OPS: RuleOp[] = ['equals', 'between', 'gte', 'lte'];
const FIELDS: RuleField[] = ['description', 'payee', 'notes', 'amount', 'account_id', 'kind'];
const KINDS: TxnKind[] = [
  'expense',
  'income',
  'transfer',
  'card_payment',
  'investment',
  'adjustment',
];
const NONE = '__none__';

const opsFor = (field: RuleField): RuleOp[] =>
  field === 'amount'
    ? AMOUNT_OPS
    : field === 'account_id' || field === 'kind'
      ? ['equals']
      : TEXT_OPS;

function emptyCondition(): RuleCondition {
  return { field: 'description', op: 'contains', value: '' };
}

export function emptyRule(): RuleInput {
  return {
    name: '',
    priority: 100,
    active: true,
    match_all: true,
    conditions: [emptyCondition()],
    actions: {},
  };
}

/** Resumo legível das condições (`Descrição contém "uber" e Valor entre …`), no idioma do app. */
export function conditionsSummary(rule: Pick<RuleInput, 'conditions' | 'match_all'>): string {
  return rule.conditions
    .map((c) => {
      const value =
        c.field === 'account_id'
          ? t('regras-conta-escolhida')
          : c.field === 'amount' && typeof c.value === 'number'
            ? formatNumber(c.value / 100, 2)
            : c.field === 'kind'
              ? (TXN_KIND[c.value as TxnKind] ?? String(c.value))
              : `“${String(c.value)}”`;
      const parts = { campo: RULE_FIELD[c.field], op: RULE_OP[c.op], valor: value };
      return c.op === 'between' && c.value2 != null
        ? t('regras-condicao-entre', { ...parts, valor2: formatNumber(c.value2 / 100, 2) })
        : t('regras-condicao', parts);
    })
    .join(rule.match_all === false ? ` ${t('regras-ou')} ` : ` ${t('regras-e')} `);
}

function ConditionRow({
  c,
  index,
  onChange,
  onRemove,
  canRemove,
}: {
  c: RuleCondition;
  index: number;
  onChange: (c: RuleCondition) => void;
  onRemove: () => void;
  canRemove: boolean;
}) {
  const { data: accounts = [] } = useAccounts(true);
  const n = index + 1;
  return (
    <div className="grid grid-cols-[130px_150px_minmax(0,1fr)_28px] items-start gap-2">
      <Select
        aria-label={t('regras-campo-condicao', { n })}
        value={c.field}
        onValueChange={(f) => {
          const field = f as RuleField;
          const op = opsFor(field)[0]!;
          onChange({
            field,
            op,
            value:
              field === 'amount'
                ? 0
                : field === 'kind'
                  ? 'expense'
                  : field === 'account_id'
                    ? (accounts[0]?.id ?? '')
                    : '',
          });
        }}
        options={FIELDS.map((f) => ({ value: f, label: RULE_FIELD[f] }))}
      />
      <Select
        aria-label={t('regras-operacao-condicao', { n })}
        value={c.op}
        onValueChange={(op) => onChange({ ...c, op: op as RuleOp })}
        options={opsFor(c.field).map((o) => ({ value: o, label: RULE_OP[o] }))}
      />
      {c.field === 'amount' ? (
        <div className="flex items-center gap-2">
          <MoneyInput
            aria-label={t('regras-valor-condicao', { n })}
            allowNegative
            value={typeof c.value === 'number' ? c.value : null}
            onValueChange={(v) => onChange({ ...c, value: v ?? 0 })}
          />
          {c.op === 'between' ? (
            <>
              <span className="text-dim">{t('regras-e')}</span>
              <MoneyInput
                aria-label={t('regras-segundo-valor-condicao', { n })}
                allowNegative
                value={c.value2 ?? null}
                onValueChange={(v) => onChange({ ...c, value2: v ?? 0 })}
              />
            </>
          ) : null}
        </div>
      ) : c.field === 'account_id' ? (
        <Select
          aria-label={t('regras-conta-condicao', { n })}
          value={String(c.value) || undefined}
          onValueChange={(v) => onChange({ ...c, value: v })}
          options={accounts.map((a) => ({ value: a.id, label: a.name }))}
        />
      ) : c.field === 'kind' ? (
        <Select
          aria-label={t('regras-tipo-condicao', { n })}
          value={String(c.value)}
          onValueChange={(v) => onChange({ ...c, value: v })}
          options={KINDS.map((k) => ({ value: k, label: TXN_KIND[k] }))}
        />
      ) : (
        <Input
          aria-label={t('regras-texto-condicao', { n })}
          value={String(c.value)}
          onChange={(e) => onChange({ ...c, value: e.target.value })}
          placeholder={c.op === 'regex' ? 'uber|99 ?pop' : 'uber'}
          className={c.op === 'regex' ? 'num' : undefined}
        />
      )}
      <IconButton
        size="sm"
        label={t('regras-tirar-condicao', { n })}
        icon={<Icon name="x" size={12} />}
        tooltip={false}
        disabled={!canRemove}
        onClick={onRemove}
      />
    </div>
  );
}

/** Criar/editar regra, com teste contra os lançamentos existentes antes de salvar. */
export function RuleDialog({
  open,
  onOpenChange,
  rule,
  initial,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  rule: Rule | null;
  /** Rascunho (ex.: sugerido a partir de um lançamento). */
  initial?: RuleInput | null;
}) {
  const { data: members = [] } = useMembers();
  const toInput = (): RuleInput =>
    rule
      ? {
          name: rule.name,
          priority: rule.priority,
          active: rule.active,
          match_all: rule.match_all,
          conditions: rule.conditions,
          actions: rule.actions,
        }
      : (initial ?? emptyRule());
  const [form, setForm] = useState<RuleInput>(toInput);
  const [key, setKey] = useState(`${rule?.id ?? initial?.name ?? 'new'}:${String(open)}`);
  const [error, setError] = useState<unknown>(null);
  const [tested, setTested] = useState<TxnView[] | null>(null);
  const [testing, setTesting] = useState(false);
  const nextKey = `${rule?.id ?? initial?.name ?? 'new'}:${String(open)}`;
  if (key !== nextKey) {
    setKey(nextKey);
    setForm(toInput());
    setError(null);
    setTested(null);
  }
  const create = useCreateRule();
  const update = useUpdateRule();
  const set = (p: Partial<RuleInput>) => {
    setForm((f) => ({ ...f, ...p }));
    setTested(null);
  };
  const setActions = (p: Partial<RuleInput['actions']>) => {
    const actions = { ...form.actions, ...p };
    for (const [k, v] of Object.entries(actions)) {
      if (v === undefined || v === null || v === '' || (Array.isArray(v) && v.length === 0))
        delete (actions as Record<string, unknown>)[k];
    }
    set({ actions });
  };

  const runTest = async () => {
    setTesting(true);
    setError(null);
    try {
      setTested(await testRule({ rule: form, limit: 50 }));
    } catch (e) {
      setError(e);
    } finally {
      setTesting(false);
    }
  };

  const save = async () => {
    if (!form.name.trim()) return setError(new Error(t('regras-erro-nome')));
    setError(null);
    try {
      if (rule) await update.mutateAsync({ id: rule.id, ...form });
      else await create.mutateAsync(form);
      toast({
        tone: 'success',
        title: rule ? t('regras-atualizada') : t('regras-criada'),
        description: t('regras-salva-descricao'),
      });
      onOpenChange(false);
    } catch (e) {
      setError(e);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      index="03"
      size="lg"
      title={rule ? t('regras-editar-titulo') : t('regras-nova')}
      footer={
        <>
          <Button
            variant="ghost"
            className="mr-auto"
            loading={testing}
            onClick={() => void runTest()}
          >
            {t('regras-testar')}
          </Button>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={create.isPending || update.isPending}
            onClick={() => void save()}
          >
            {rule ? t('regras-salvar') : t('regras-criar')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-[1fr_120px]">
          <Field label={t('regras-nome')}>
            <Input
              value={form.name}
              onChange={(e) => set({ name: e.target.value })}
              placeholder={t('regras-nome-exemplo')}
            />
          </Field>
          <Field label={t('regras-prioridade')} hint={t('regras-prioridade-dica')}>
            <Input
              type="number"
              numeric
              value={form.priority ?? 100}
              onChange={(e) => set({ priority: Number(e.target.value) })}
            />
          </Field>
        </div>

        <fieldset className="flex flex-col gap-2 rounded-card border border-line p-3">
          <legend className="label-caps px-1">{t('regras-quando')}</legend>
          <Segmented
            label={t('regras-combinar')}
            size="sm"
            value={form.match_all === false ? 'any' : 'all'}
            onChange={(v) => set({ match_all: v === 'all' })}
            options={[
              { value: 'all', label: t('regras-todas') },
              { value: 'any', label: t('regras-qualquer') },
            ]}
          />
          {form.conditions.map((c, i) => (
            <ConditionRow
              key={i}
              c={c}
              index={i}
              canRemove={form.conditions.length > 1}
              onChange={(next) =>
                set({ conditions: form.conditions.map((x, j) => (j === i ? next : x)) })
              }
              onRemove={() => set({ conditions: form.conditions.filter((_, j) => j !== i) })}
            />
          ))}
          <Button
            size="sm"
            variant="ghost"
            className="self-start"
            icon={<Icon name="plus" size={12} />}
            onClick={() => set({ conditions: [...form.conditions, emptyCondition()] })}
          >
            {t('regras-adicionar-condicao')}
          </Button>
          <p className="text-[12px] text-dim">{t('regras-condicoes-nota')}</p>
        </fieldset>

        <fieldset className="grid gap-3 rounded-card border border-line p-3 sm:grid-cols-2">
          <legend className="label-caps px-1">{t('regras-entao')}</legend>
          <Field label={t('regras-categoria')}>
            <CategoryPicker
              value={form.actions.category_id ?? null}
              onChange={(category_id) => setActions({ category_id: category_id ?? undefined })}
              noneLabel={t('regras-nao-mudar')}
              className="[&>button]:h-9 [&>button]:w-full"
            />
          </Field>
          <Field label={t('regras-membro')}>
            <Select
              value={form.actions.member_id ?? NONE}
              onValueChange={(v) => setActions({ member_id: v === NONE ? undefined : v })}
              options={[
                { value: NONE, label: t('regras-nao-mudar') },
                ...members.map((m) => ({ value: m.id, label: m.name })),
              ]}
            />
          </Field>
          <Field label={t('regras-renomear')} hint={t('regras-renomear-dica')}>
            <Input
              value={form.actions.rename_to ?? ''}
              onChange={(e) => setActions({ rename_to: e.target.value || undefined })}
            />
          </Field>
          <Field label={t('regras-mudar-tipo')}>
            <Select
              value={form.actions.kind ?? NONE}
              onValueChange={(v) => setActions({ kind: v === NONE ? undefined : (v as TxnKind) })}
              options={[
                { value: NONE, label: t('regras-nao-mudar') },
                ...KINDS.map((k) => ({ value: k, label: TXN_KIND[k] })),
              ]}
            />
          </Field>
          <Field label={t('regras-tags')} className="sm:col-span-2">
            <TagInput value={form.actions.tags ?? []} onChange={(tags) => setActions({ tags })} />
          </Field>
        </fieldset>

        {tested ? (
          <section
            aria-label={t('regras-resultado-teste')}
            className="rounded-card border border-line"
          >
            <p className="label-caps border-b border-line px-3 py-2" role="status">
              {tested.length === 0
                ? t('regras-teste-nenhum')
                : tested.length >= 50
                  ? t('regras-teste-50')
                  : t('regras-teste-n', { n: tested.length })}
            </p>
            <ul className="max-h-48 overflow-auto">
              {tested.map((txn) => (
                <li
                  key={txn.id}
                  className="grid grid-cols-[88px_1fr_auto_120px] items-center gap-3 border-b border-line/60 px-3 py-1.5 text-[12.5px]"
                >
                  <span className="num text-dim">{formatDate(txn.date)}</span>
                  <span className="truncate">{txn.description}</span>
                  <span className="truncate text-dim">
                    {txn.category_path ?? t('regras-sem-categoria')}
                  </span>
                  <Money
                    cents={txn.amount}
                    currency={txn.currency}
                    tone="auto"
                    className="text-right"
                  />
                </li>
              ))}
            </ul>
          </section>
        ) : null}
        <InlineError error={error} />
      </div>
    </Dialog>
  );
}
