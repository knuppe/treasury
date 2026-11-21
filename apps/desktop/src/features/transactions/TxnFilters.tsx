import { useMemo, useState, type RefObject } from 'react';

import type { CategoryNode } from '@/api/generated/CategoryNode';
import type { TxnFilter } from '@/api/generated/TxnFilter';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnStatus } from '@/api/generated/TxnStatus';
import { useAccounts } from '@/api/hooks/accounts';
import { useCategoryTree } from '@/api/hooks/categories';
import { useMembers } from '@/api/hooks/household';
import { useProjects } from '@/api/hooks/projects';
import { useTags } from '@/api/hooks/transactions';
import { Button, DateInput, Field, Icon, Input, Kbd, MoneyInput } from '@/design/components';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';

import {
  detectPeriod,
  hasActiveFilters,
  periodLabel,
  periodRange,
  PERIODS,
  type PeriodId,
} from './filterState';
import { Checkbox, Segmented } from './ui/controls';
import { MultiSelect, type MultiOption } from './ui/MultiSelect';
import { Popover } from './ui/Popover';
import { TXN_KIND, TXN_STATUS } from './ui/labels';

function flattenTree(nodes: CategoryNode[], depth = 0, out: MultiOption[] = []): MultiOption[] {
  for (const n of nodes) {
    out.push({
      value: n.id,
      label: n.name,
      depth,
      hint: n.kind === 'income' ? t('lancamentos-filtro-receita') : undefined,
    });
    flattenTree(n.children, depth + 1, out);
  }
  return out;
}

const KINDS: TxnKind[] = [
  'expense',
  'income',
  'transfer',
  'card_payment',
  'investment',
  'adjustment',
];
const STATUSES: TxnStatus[] = ['planned', 'cleared', 'reconciled'];

export interface TxnFiltersProps {
  filter: TxnFilter;
  onChange: (next: TxnFilter) => void;
  searchRef?: RefObject<HTMLInputElement | null>;
}

/** Barra de filtros da tabela de lançamentos (tudo refletido na URL). */
export function TxnFilters({ filter, onChange, searchRef }: TxnFiltersProps) {
  const { data: accounts = [] } = useAccounts(true);
  const base = useBaseCurrency();
  const { data: tree = [] } = useCategoryTree({ include_archived: true });
  const { data: members = [] } = useMembers(true);
  const { data: tags = [] } = useTags();
  const { data: projects = [] } = useProjects();
  const [text, setText] = useState(filter.text ?? '');
  const [syncedText, setSyncedText] = useState(filter.text ?? '');
  if ((filter.text ?? '') !== syncedText) {
    setSyncedText(filter.text ?? '');
    setText(filter.text ?? '');
  }

  const set = (patch: Partial<TxnFilter>) => {
    const next: TxnFilter = { ...filter, ...patch };
    for (const [k, v] of Object.entries(next)) {
      if (v === undefined || v === '' || (Array.isArray(v) && v.length === 0)) {
        delete (next as Record<string, unknown>)[k];
      }
    }
    onChange(next);
  };

  const categoryOptions = useMemo(() => flattenTree(tree), [tree]);
  const accountOptions = accounts.map((a) => ({
    value: a.id,
    label: a.name,
    hint: a.archived
      ? t('lancamentos-filtro-arquivada')
      : a.currency !== base
        ? a.currency
        : undefined,
  }));

  const period = detectPeriod(filter);
  const active = hasActiveFilters(filter);

  return (
    <div className="flex flex-col gap-2" role="search" aria-label={t('lancamentos-filtros')}>
      <div className="flex flex-wrap items-center gap-2">
        <form
          className="min-w-56 flex-1"
          onSubmit={(e) => {
            e.preventDefault();
            set({ text: text.trim() || undefined });
          }}
        >
          <Input
            ref={searchRef}
            type="search"
            aria-label={t('lancamentos-buscar')}
            aria-keyshortcuts="/"
            placeholder={t('lancamentos-buscar-placeholder')}
            value={text}
            prefix={<Icon name="search" size={13} />}
            suffix={text ? null : <Kbd keys={['/']} className="mr-1.5" />}
            onChange={(e) => setText(e.target.value)}
            onBlur={() => {
              if (text.trim() !== (filter.text ?? '')) set({ text: text.trim() || undefined });
            }}
            onKeyDown={(e) => {
              if (e.key === 'Escape' && text) {
                e.stopPropagation();
                setText('');
                set({ text: undefined });
              }
            }}
          />
        </form>

        <Popover
          active={period !== 'all'}
          ariaLabel={t('lancamentos-periodo-aria', { periodo: periodLabel(filter) })}
          label={
            <>
              <span className="label-caps">{t('lancamentos-periodo')}</span>
              <span>{periodLabel(filter)}</span>
              <Icon name="chevronDown" size={12} className="text-dim" />
            </>
          }
          panelClassName="w-80"
        >
          {(close) => <PeriodPanel filter={filter} onApply={(p) => (set(p), close())} />}
        </Popover>

        <MultiSelect
          label={t('lancamentos-contas')}
          emptyLabel={t('lancamentos-todas')}
          options={accountOptions}
          value={filter.account_ids ?? []}
          onChange={(v) => set({ account_ids: v })}
        />
        <MultiSelect
          label={t('lancamentos-categorias')}
          emptyLabel={t('lancamentos-todas')}
          options={categoryOptions}
          value={filter.category_ids ?? []}
          onChange={(v) => set({ category_ids: v, uncategorized: undefined })}
          searchable
        />
        <MultiSelect
          label={t('lancamentos-membros')}
          options={members.map((m) => ({ value: m.id, label: m.name }))}
          value={filter.member_ids ?? []}
          onChange={(v) => set({ member_ids: v })}
        />
        <MultiSelect
          label={t('lancamentos-campo-tipo')}
          options={KINDS.map((k) => ({ value: k, label: TXN_KIND[k] }))}
          value={filter.kinds ?? []}
          onChange={(v) => set({ kinds: v as TxnKind[] })}
        />
        <MultiSelect
          label={t('lancamentos-campo-status')}
          options={STATUSES.map((s) => ({ value: s, label: TXN_STATUS[s] }))}
          value={filter.statuses ?? []}
          onChange={(v) => set({ statuses: v as TxnStatus[] })}
        />
      </div>

      <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
        <div className="flex flex-wrap items-center gap-2">
          {tags.length > 0 ? (
            <MultiSelect
              label={t('lancamentos-campo-tags')}
              options={tags.map((tag) => ({ value: tag.id, label: tag.name, hint: tag.txn_count }))}
              value={filter.tag_ids ?? []}
              onChange={(v) => set({ tag_ids: v })}
            />
          ) : null}
          {projects.length > 0 ? (
            <MultiSelect
              label={t('lancamentos-projetos')}
              options={projects.map((p) => ({ value: p.id, label: p.name, hint: p.txn_count }))}
              value={filter.project_ids ?? []}
              onChange={(v) => set({ project_ids: v })}
            />
          ) : null}
          <Popover
            active={filter.amount_min != null || filter.amount_max != null}
            ariaLabel={t('lancamentos-faixa-valor')}
            label={
              <>
                <span className="label-caps">{t('lancamentos-campo-valor')}</span>
                <Icon name="chevronDown" size={12} className="text-dim" />
              </>
            }
            panelClassName="w-72"
          >
            {(close) => <AmountPanel filter={filter} onApply={(p) => (set(p), close())} />}
          </Popover>
        </div>
        <Segmented
          label={t('lancamentos-base-datas')}
          size="sm"
          value={filter.basis ?? 'accrual'}
          onChange={(v) => set({ basis: v === 'accrual' ? undefined : v })}
          options={[
            { value: 'accrual', label: t('lancamentos-competencia') },
            { value: 'cash', label: t('lancamentos-caixa') },
          ]}
        />
        <Checkbox
          label={t('lancamentos-so-sem-categoria')}
          checked={Boolean(filter.uncategorized)}
          onChange={(on) =>
            set({
              uncategorized: on ? true : undefined,
              category_ids: on ? undefined : filter.category_ids,
            })
          }
        />
        {filter.statement_id ? (
          <FilterChip
            label={t('lancamentos-fatura-selecionada')}
            onClear={() => set({ statement_id: undefined })}
          />
        ) : null}
        {filter.import_id ? (
          <FilterChip
            label={t('lancamentos-lote-importacao')}
            onClear={() => set({ import_id: undefined })}
          />
        ) : null}
        {filter.text ? (
          <FilterChip label={`“${filter.text}”`} onClear={() => set({ text: undefined })} />
        ) : null}
        {active ? (
          <Button
            size="sm"
            variant="ghost"
            onClick={() => onChange(filter.sort ? { sort: filter.sort } : {})}
          >
            {t('lancamentos-limpar-filtros')}
          </Button>
        ) : null}
      </div>
    </div>
  );
}

function FilterChip({ label, onClear }: { label: string; onClear: () => void }) {
  return (
    <span className="inline-flex h-6 items-center gap-1 rounded-control border border-accent/50 bg-accent-soft pr-0.5 pl-2 text-[12px] text-accent">
      {label}
      <button
        type="button"
        aria-label={t('lancamentos-tirar-filtro', { filtro: label })}
        onClick={onClear}
        className="inline-flex size-5 items-center justify-center rounded-control hover:bg-accent/20"
      >
        <Icon name="x" size={10} />
      </button>
    </span>
  );
}

function PeriodPanel({
  filter,
  onApply,
}: {
  filter: TxnFilter;
  onApply: (p: Partial<TxnFilter>) => void;
}) {
  const [from, setFrom] = useState<string | null>(filter.date_from ?? null);
  const [to, setTo] = useState<string | null>(filter.date_to ?? null);
  // O atalho preenche as datas (dá para ajustar) e o filtro só vale em "Aplicar período";
  // duplo clique aplica direto.
  const current = detectPeriod({
    ...filter,
    date_from: from ?? undefined,
    date_to: to ?? undefined,
  });
  const fill = (id: PeriodId) => {
    const r = periodRange(id);
    setFrom(r.from ?? null);
    setTo(r.to ?? null);
    return r;
  };
  const apply = () => onApply({ date_from: from ?? undefined, date_to: to ?? undefined });
  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-1">
        {PERIODS.filter((p) => p.id !== 'custom').map((p) => (
          <button
            key={p.id}
            type="button"
            aria-pressed={current === p.id}
            onClick={() => fill(p.id)}
            onDoubleClick={() => {
              const r = fill(p.id);
              onApply({ date_from: r.from, date_to: r.to });
            }}
            className="h-8 rounded-control px-2 text-left text-[13px] text-fg hover:bg-accent-soft aria-pressed:bg-accent-soft aria-pressed:text-accent"
          >
            {p.label}
          </button>
        ))}
      </div>
      <div className="grid grid-cols-2 gap-2 border-t border-line pt-3">
        <Field label={t('lancamentos-de')}>
          <DateInput value={from} onValueChange={setFrom} />
        </Field>
        <Field label={t('lancamentos-ate')}>
          <DateInput value={to} onValueChange={setTo} />
        </Field>
      </div>
      <Button
        size="sm"
        variant="primary"
        disabled={Boolean(from && to && from > to)}
        onClick={apply}
      >
        {t('lancamentos-aplicar-periodo')}
      </Button>
    </div>
  );
}

function AmountPanel({
  filter,
  onApply,
}: {
  filter: TxnFilter;
  onApply: (p: Partial<TxnFilter>) => void;
}) {
  const [min, setMin] = useState<number | null>(filter.amount_min ?? null);
  const [max, setMax] = useState<number | null>(filter.amount_max ?? null);
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[12px] text-dim">{t('lancamentos-valor-absoluto')}</p>
      <div className="grid grid-cols-2 gap-2">
        <Field label={t('lancamentos-minimo')}>
          <MoneyInput value={min} onValueChange={setMin} />
        </Field>
        <Field label={t('lancamentos-maximo')}>
          <MoneyInput value={max} onValueChange={setMax} />
        </Field>
      </div>
      <div className="flex gap-2">
        <Button
          size="sm"
          variant="ghost"
          onClick={() => onApply({ amount_min: undefined, amount_max: undefined })}
        >
          {t('lancamentos-limpar')}
        </Button>
        <Button
          size="sm"
          variant="primary"
          className="flex-1"
          onClick={() => onApply({ amount_min: min ?? undefined, amount_max: max ?? undefined })}
        >
          {t('lancamentos-aplicar')}
        </Button>
      </div>
    </div>
  );
}
