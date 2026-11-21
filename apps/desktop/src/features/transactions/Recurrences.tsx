import { useMemo, useState } from 'react';

import type { Freq } from '@/api/generated/Freq';
import type { Recurrence } from '@/api/generated/Recurrence';
import type { RuleSpec } from '@/api/generated/RuleSpec';
import { useAccounts } from '@/api/hooks/accounts';
import { useMembers } from '@/api/hooks/household';
import {
  useCreateRecurrence,
  useDeleteRecurrence,
  useRecurrencePreview,
  useRecurrences,
  useUpdateRecurrence,
} from '@/api/hooks/recurrences';
import {
  Button,
  DateInput,
  Dialog,
  EmptyState,
  Field,
  Icon,
  IconButton,
  Input,
  InstrumentCard,
  Money,
  MoneyInput,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';
import { formatDate, monthName, todayIso } from '@/lib/format';

import { CategoryPicker } from './ui/CategoryPicker';
import { Checkbox, ConfirmDialog, InlineError, QueryState, Segmented, Switch } from './ui/controls';
import { errorText } from './ui/errors';
import { TXN_KIND } from './ui/labels';

type RecKind = 'expense' | 'income' | 'transfer' | 'investment';

/** Códigos RRULE dos dias, de segunda a domingo (a ordem das caixas). */
const WEEKDAYS = ['MO', 'TU', 'WE', 'TH', 'FR', 'SA', 'SU'] as const;

/** `2026-10-05` → `seg. 05/10/2026` (dia da semana abreviado no idioma do app). */
export function dateWithWeekday(iso: string): string {
  const [y, m, d] = iso.split('-').map(Number) as [number, number, number];
  const wd = new Date(Date.UTC(y, m - 1, d)).getUTCDay();
  return `${t('lancamentos-dias-semana-abrev').split(',')[wd] ?? ''} ${formatDate(iso)}`;
}

const monthOptions = () =>
  Array.from({ length: 12 }, (_, i) => ({ value: String(i + 1), label: monthName(i + 1) }));

interface RecForm {
  description: string;
  kind: RecKind;
  accountId: string;
  toAccountId: string;
  amount: number | null;
  categoryId: string | null;
  memberId: string | null;
  freq: Freq;
  interval: number;
  monthDay: number;
  weekdays: string[];
  month: number;
  start: string | null;
  end: string | null;
  count: number | null;
}

function specOf(f: RecForm): RuleSpec {
  const spec: RuleSpec = { freq: f.freq, interval: Math.max(1, f.interval || 1) };
  if (f.freq === 'MONTHLY' || f.freq === 'YEARLY') spec.by_month_day = f.monthDay;
  if (f.freq === 'YEARLY') spec.by_month = f.month;
  if (f.freq === 'WEEKLY' && f.weekdays.length) spec.by_day = f.weekdays;
  if (f.count) spec.count = f.count;
  return spec;
}

function formFrom(r: Recurrence | null, defaultAccount: string): RecForm {
  const start = r?.start_date ?? todayIso();
  const day = Number(start.slice(8, 10));
  return {
    description: r?.description ?? '',
    kind: (r?.kind as RecKind | undefined) ?? 'expense',
    accountId: r?.account_id ?? defaultAccount,
    toAccountId: r?.to_account_id ?? '',
    amount: r ? Math.abs(r.amount) : null,
    categoryId: r?.category_id ?? null,
    memberId: r?.member_id ?? null,
    freq: r?.spec.freq ?? 'MONTHLY',
    interval: r?.spec.interval ?? 1,
    monthDay: r?.spec.by_month_day ?? day,
    weekdays: r?.spec.by_day ?? [],
    month: r?.spec.by_month ?? Number(start.slice(5, 7)),
    start,
    end: r?.end_date ?? null,
    count: r?.spec.count ?? null,
  };
}

const NO_MEMBER = '__none__';

/** Criar ou editar uma recorrência, com a prévia das próximas datas em português. */
export function RecurrenceDialog({
  open,
  onOpenChange,
  recurrence,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  recurrence: Recurrence | null;
}) {
  const { data: accounts = [] } = useAccounts();
  const { data: members = [] } = useMembers();
  const base = useBaseCurrency();
  const defaultAccount = accounts.find((a) => a.kind === 'checking')?.id ?? accounts[0]?.id ?? '';
  const [form, setForm] = useState<RecForm>(() => formFrom(recurrence, defaultAccount));
  const [key, setKey] = useState(`${recurrence?.id ?? 'new'}:${String(open)}`);
  const [error, setError] = useState<unknown>(null);
  const nextKey = `${recurrence?.id ?? 'new'}:${String(open)}`;
  if (key !== nextKey) {
    setKey(nextKey);
    setForm(formFrom(recurrence, defaultAccount));
    setError(null);
  }
  if (!form.accountId && defaultAccount) setForm((f) => ({ ...f, accountId: defaultAccount }));
  const set = (p: Partial<RecForm>) => setForm((f) => ({ ...f, ...p }));
  const editing = Boolean(recurrence);
  const transfer = form.kind === 'transfer' || form.kind === 'investment';
  const spec = specOf(form);
  const previewArgs = form.start
    ? { spec, start_date: form.start, count: 6, ...(form.end ? { end_date: form.end } : {}) }
    : null;
  const preview = useRecurrencePreview(open ? previewArgs : null);
  const create = useCreateRecurrence();
  const update = useUpdateRecurrence();
  const account = accounts.find((a) => a.id === form.accountId);

  const submit = async () => {
    if (!form.description.trim()) return setError(new Error(t('lancamentos-erro-descricao')));
    if (!form.amount) return setError(new Error(t('lancamentos-erro-valor-simples')));
    if (!form.start) return setError(new Error(t('lancamentos-erro-inicio')));
    if (transfer && !form.toAccountId) return setError(new Error(t('lancamentos-erro-destino')));
    setError(null);
    const sign = form.kind === 'income' ? 1 : -1;
    try {
      if (recurrence) {
        await update.mutateAsync({
          id: recurrence.id,
          description: form.description.trim(),
          amount: sign * form.amount,
          spec,
          end_date: form.end,
          ...(transfer ? {} : { category_id: form.categoryId }),
          member_id: form.memberId,
        });
        toast({
          tone: 'success',
          title: t('lancamentos-recorrencia-atualizada'),
          description: t('lancamentos-recorrencia-atualizada-descricao'),
        });
      } else {
        await create.mutateAsync({
          description: form.description.trim(),
          account_id: form.accountId,
          amount: sign * form.amount,
          kind: form.kind,
          spec,
          start_date: form.start,
          ...(form.end ? { end_date: form.end } : {}),
          ...(transfer
            ? { to_account_id: form.toAccountId }
            : form.categoryId
              ? { category_id: form.categoryId }
              : {}),
          ...(form.memberId ? { member_id: form.memberId } : {}),
        });
        toast({
          tone: 'success',
          title: t('lancamentos-recorrencia-criada'),
          description: t('lancamentos-recorrencia-criada-descricao'),
        });
      }
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
      title={editing ? t('lancamentos-editar-recorrencia') : t('lancamentos-nova-recorrencia')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={create.isPending || update.isPending}
            onClick={() => void submit()}
          >
            {editing ? t('comum-salvar') : t('lancamentos-criar-recorrencia')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        {!editing ? (
          <Segmented<RecKind>
            label={t('lancamentos-campo-tipo')}
            value={form.kind}
            onChange={(kind) => set({ kind })}
            options={(['expense', 'income', 'transfer', 'investment'] as const).map((k) => ({
              value: k,
              label: TXN_KIND[k],
            }))}
          />
        ) : null}
        <div className="grid gap-3 sm:grid-cols-2">
          <Field label={t('lancamentos-campo-descricao')}>
            <Input
              value={form.description}
              onChange={(e) => set({ description: e.target.value })}
              placeholder={t('lancamentos-recorrencia-exemplo')}
            />
          </Field>
          <Field label={t('lancamentos-valor-moeda', { moeda: account?.currency ?? base })}>
            <MoneyInput
              currency={account?.currency}
              value={form.amount}
              onValueChange={(amount) => set({ amount })}
            />
          </Field>
          <Field label={transfer ? t('lancamentos-de-origem') : t('lancamentos-campo-conta')}>
            <Select
              value={form.accountId || undefined}
              disabled={editing}
              onValueChange={(accountId) => set({ accountId })}
              options={accounts.map((a) => ({ value: a.id, label: a.name, hint: a.currency }))}
            />
          </Field>
          {transfer ? (
            <Field label={t('lancamentos-para-destino')}>
              <Select
                value={form.toAccountId || undefined}
                disabled={editing}
                placeholder={t('lancamentos-escolha-conta')}
                onValueChange={(toAccountId) => set({ toAccountId })}
                options={accounts
                  .filter((a) => a.id !== form.accountId && a.currency === account?.currency)
                  .map((a) => ({ value: a.id, label: a.name }))}
              />
            </Field>
          ) : (
            <Field label={t('lancamentos-campo-categoria')}>
              <CategoryPicker
                kind={form.kind === 'income' ? 'income' : 'expense'}
                value={form.categoryId}
                onChange={(categoryId) => set({ categoryId })}
                className="[&>button]:h-9 [&>button]:w-full"
              />
            </Field>
          )}
          <Field label={t('lancamentos-campo-membro')}>
            <Select
              value={form.memberId ?? NO_MEMBER}
              onValueChange={(v) => set({ memberId: v === NO_MEMBER ? null : v })}
              options={[
                { value: NO_MEMBER, label: '—' },
                ...members.map((m) => ({ value: m.id, label: m.name })),
              ]}
            />
          </Field>
        </div>

        <fieldset className="grid gap-3 rounded-card border border-line p-3 sm:grid-cols-[auto_1fr]">
          <legend className="label-caps px-1">{t('lancamentos-repeticao')}</legend>
          <Field label={t('lancamentos-frequencia')}>
            <Segmented<Freq>
              label={t('lancamentos-frequencia')}
              value={form.freq}
              onChange={(freq) => set({ freq })}
              options={[
                { value: 'DAILY', label: t('lancamentos-freq-diaria') },
                { value: 'WEEKLY', label: t('lancamentos-freq-semanal') },
                { value: 'MONTHLY', label: t('lancamentos-freq-mensal') },
                { value: 'YEARLY', label: t('lancamentos-freq-anual') },
              ]}
            />
          </Field>
          <div className="grid grid-cols-2 gap-3">
            <Field label={t('lancamentos-a-cada')}>
              <Input
                type="number"
                min={1}
                max={999}
                numeric
                value={form.interval}
                onChange={(e) => set({ interval: Number(e.target.value) })}
              />
            </Field>
            {form.freq === 'MONTHLY' || form.freq === 'YEARLY' ? (
              <Field label={t('lancamentos-dia-do-mes')} hint={t('lancamentos-dia-do-mes-dica')}>
                <Input
                  type="number"
                  min={1}
                  max={31}
                  numeric
                  value={form.monthDay}
                  onChange={(e) => set({ monthDay: Number(e.target.value) })}
                />
              </Field>
            ) : null}
            {form.freq === 'YEARLY' ? (
              <Field label={t('lancamentos-mes')}>
                <Select
                  value={String(form.month)}
                  onValueChange={(v) => set({ month: Number(v) })}
                  options={monthOptions()}
                />
              </Field>
            ) : null}
          </div>
          {form.freq === 'WEEKLY' || form.freq === 'DAILY' ? (
            <div
              className="flex flex-wrap gap-3 sm:col-span-2"
              role="group"
              aria-label={t('lancamentos-dias-semana')}
            >
              {WEEKDAYS.map((code, i) => (
                <Checkbox
                  key={code}
                  label={t('lancamentos-dias-semana-caixas').split(',')[i] ?? code}
                  checked={form.weekdays.includes(code)}
                  onChange={(on) =>
                    set({
                      weekdays: on
                        ? [...form.weekdays, code]
                        : form.weekdays.filter((d) => d !== code),
                    })
                  }
                />
              ))}
            </div>
          ) : null}
          <div className="grid grid-cols-3 gap-3 sm:col-span-2">
            <Field label={t('lancamentos-inicio')}>
              <DateInput
                value={form.start}
                onValueChange={(start) => set({ start })}
                disabled={editing}
              />
            </Field>
            <Field label={t('lancamentos-fim')}>
              <DateInput value={form.end} onValueChange={(end) => set({ end })} />
            </Field>
            <Field label={t('lancamentos-ocorrencias')}>
              <Input
                type="number"
                min={1}
                numeric
                value={form.count ?? ''}
                onChange={(e) => set({ count: e.target.value ? Number(e.target.value) : null })}
              />
            </Field>
          </div>
        </fieldset>

        <section
          aria-label={t('lancamentos-previa-datas')}
          className="rounded-card border border-line bg-bg/40 p-3"
        >
          <p className="label-caps mb-2">{t('lancamentos-previa')}</p>
          {preview.error ? (
            <InlineError error={preview.error} />
          ) : preview.data ? (
            <>
              <p className="text-[14px] text-fg first-letter:uppercase">{preview.data.human}</p>
              <ol className="num mt-2 grid grid-cols-2 gap-x-6 gap-y-0.5 text-[12.5px] text-dim sm:grid-cols-3">
                {preview.data.dates.map((d) => (
                  <li key={d}>{dateWithWeekday(d)}</li>
                ))}
              </ol>
            </>
          ) : (
            <p className="text-[12.5px] text-dim">{t('lancamentos-previa-vazia')}</p>
          )}
        </section>
        <InlineError error={error} />
      </div>
    </Dialog>
  );
}

/** Aba "Recorrências": lista, pausar/retomar, editar, apagar e o total mensal comprometido. */
export function RecurrencesPanel({ onNew }: { onNew: () => void }) {
  const { data = [], isLoading, error, refetch } = useRecurrences();
  const { data: accounts = [] } = useAccounts(true);
  const update = useUpdateRecurrence();
  const remove = useDeleteRecurrence();
  const [editing, setEditing] = useState<Recurrence | null>(null);
  const [deleting, setDeleting] = useState<Recurrence | null>(null);
  const [keepPlanned, setKeepPlanned] = useState(false);

  const monthly = useMemo(() => {
    const out = new Map<string, number>();
    for (const r of data) {
      if (!r.active || r.kind === 'transfer' || r.kind === 'investment') continue;
      out.set(r.currency, (out.get(r.currency) ?? 0) + r.monthly_equivalent);
    }
    return [...out.entries()];
  }, [data]);

  const columns: TechColumn<Recurrence>[] = [
    {
      key: 'description',
      header: t('lancamentos-campo-descricao'),
      sortValue: (r) => r.description,
      cell: (r) => (
        <span className={r.active ? undefined : 'text-dim line-through'}>
          {r.description}
          <span className="ml-2 text-[11.5px] text-dim">{TXN_KIND[r.kind]}</span>
        </span>
      ),
    },
    {
      key: 'account',
      header: t('lancamentos-campo-conta'),
      cell: (r) => {
        const to = accounts.find((a) => a.id === r.to_account_id)?.name;
        return (
          <span className="text-dim">
            {r.account_name}
            {to ? ` → ${to}` : ''}
          </span>
        );
      },
    },
    {
      key: 'rule',
      header: t('lancamentos-col-regra'),
      cell: (r) => <span className="block first-letter:uppercase">{r.human}</span>,
    },
    {
      key: 'next',
      header: t('lancamentos-col-proximas'),
      cell: (r) =>
        r.next_dates.length ? (
          <span className="num text-[12px] whitespace-nowrap text-dim">
            {r.next_dates.map((d) => formatDate(d).slice(0, 5)).join(' · ')}
          </span>
        ) : (
          <span className="label-caps">{t('lancamentos-pausada')}</span>
        ),
    },
    {
      key: 'monthly',
      header: t('lancamentos-col-por-mes'),
      numeric: true,
      sortValue: (r) => r.monthly_equivalent,
      cell: (r) => <Money cents={r.monthly_equivalent} currency={r.currency} tone="auto" />,
    },
    {
      key: 'actions',
      header: <span className="sr-only">{t('lancamentos-acoes')}</span>,
      width: '170px',
      align: 'right',
      cell: (r) => (
        <span className="inline-flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
          <Button
            size="sm"
            variant="ghost"
            onClick={() =>
              update.mutate(
                { id: r.id, active: !r.active },
                {
                  onSuccess: () =>
                    toast({
                      tone: 'success',
                      title: r.active
                        ? t('lancamentos-recorrencia-pausada')
                        : t('lancamentos-recorrencia-retomada'),
                    }),
                  onError: (e) =>
                    toast({
                      tone: 'error',
                      title: t('lancamentos-erro-alterar'),
                      description: errorText(e),
                    }),
                },
              )
            }
          >
            {r.active ? t('lancamentos-pausar') : t('lancamentos-retomar')}
          </Button>
          <IconButton
            size="sm"
            label={t('lancamentos-editar-item', { nome: r.description })}
            icon={<Icon name="arrowRight" size={13} />}
            onClick={() => setEditing(r)}
          />
          <IconButton
            size="sm"
            label={t('lancamentos-apagar-item', { nome: r.description })}
            icon={<Icon name="x" size={13} />}
            onClick={() => setDeleting(r)}
          />
        </span>
      ),
    },
  ];

  return (
    <InstrumentCard
      index="04.2"
      title={t('lancamentos-aba-recorrencias')}
      padded={false}
      actions={
        <Button size="sm" variant="secondary" icon={<Icon name="plus" size={12} />} onClick={onNew}>
          {t('lancamentos-nova-recorrencia')}
        </Button>
      }
      footer={
        monthly.length
          ? monthly.map(([cur, v]) => ({
              label: t('lancamentos-comprometido', { moeda: cur }),
              value: <Money cents={v} currency={cur} />,
            }))
          : undefined
      }
    >
      <QueryState loading={isLoading} error={error} onRetry={() => void refetch()}>
        {data.length === 0 ? (
          <div className="p-4">
            <EmptyState
              kicker={t('lancamentos-recorrencias-vazio-kicker')}
              title={t('lancamentos-recorrencias-vazio-titulo')}
              description={t('lancamentos-recorrencias-vazio-descricao')}
              actions={
                <Button variant="primary" onClick={onNew}>
                  {t('lancamentos-nova-recorrencia')}
                </Button>
              }
            />
          </div>
        ) : (
          <TechTable
            label={t('lancamentos-aba-recorrencias')}
            prefsId="Recorrências"
            columns={columns}
            rows={data}
            getRowId={(r) => r.id}
            onActivate={setEditing}
            dense
          />
        )}
      </QueryState>
      <RecurrenceDialog
        open={Boolean(editing)}
        onOpenChange={(o) => !o && setEditing(null)}
        recurrence={editing}
      />
      <ConfirmDialog
        open={Boolean(deleting)}
        onOpenChange={(o) => !o && setDeleting(null)}
        title={t('lancamentos-apagar-recorrencia')}
        description={t('lancamentos-apagar-recorrencia-descricao', {
          descricao: deleting?.description ?? '',
        })}
        confirmLabel={t('comum-apagar')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          deleting &&
          remove.mutate(
            { id: deleting.id, delete_planned: !keepPlanned },
            {
              onSuccess: (res) => {
                toast({
                  tone: 'success',
                  title: t('lancamentos-recorrencia-apagada'),
                  description: t('lancamentos-previstos-removidos', { n: res.deleted_planned }),
                });
                setDeleting(null);
              },
            },
          )
        }
      >
        <Switch
          checked={keepPlanned}
          onChange={setKeepPlanned}
          label={t('lancamentos-manter-previstos')}
        />
      </ConfirmDialog>
    </InstrumentCard>
  );
}
