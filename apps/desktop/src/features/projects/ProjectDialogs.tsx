/**
 * Projetos (reforma, benfeitoria, viagem…): seletor, cadastro, ligar lançamentos já
 * escolhidos (lote e menu da linha), buscar lançamentos para pôr num projeto e lançar itens
 * à mão (só no projeto ou numa conta).
 */
import { useMemo, useState } from 'react';

import type { ProjectEntry } from '@/api/generated/ProjectEntry';
import type { ProjectFlow } from '@/api/generated/ProjectFlow';
import type { ProjectStatus } from '@/api/generated/ProjectStatus';
import type { ProjectView } from '@/api/generated/ProjectView';
import type { TxnView } from '@/api/generated/TxnView';
import { useAccounts } from '@/api/hooks/accounts';
import { usePhysicalAssets } from '@/api/hooks/physical';
import {
  useAddProjectItem,
  useCreateProject,
  useLinkProject,
  useProjects,
  useUpdateProject,
  useUpdateProjectItem,
} from '@/api/hooks/projects';
import { useCreateTxn, useTxnQuery } from '@/api/hooks/transactions';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Input,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';
import { Checkbox, InlineError, Segmented, TextArea } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { addDays } from '@/lib/months';

export const PROJECT_STATUS: Record<ProjectStatus, string> = {
  get planned() {
    return t('projetos-status-planejado');
  },
  get active() {
    return t('projetos-status-andamento');
  },
  get done() {
    return t('projetos-status-concluido');
  },
};

const NONE = '__nenhum__';
const NEW = '__novo__';

/** Projetos para escolher: os em aberto e o atual (mesmo concluído). */
function choices(projects: readonly ProjectView[], current: string | null): ProjectView[] {
  return projects.filter((p) => p.status !== 'done' || p.id === current);
}

/** Seleção de projeto ("Sem projeto", os em aberto e, se pedido, "Novo projeto…"). */
export function ProjectSelect({
  value,
  onChange,
  allowNew,
  newLabel,
  'aria-label': ariaLabel,
}: {
  /** `'new'` = a opção de criar um projeto. */
  value: string | null;
  onChange: (id: string | null) => void;
  allowNew?: boolean;
  newLabel?: string;
  'aria-label'?: string;
}) {
  const { data: projects = [] } = useProjects();
  return (
    <Select
      aria-label={ariaLabel}
      value={value === 'new' ? NEW : (value ?? NONE)}
      onValueChange={(v) => onChange(v === NONE ? null : v === NEW ? 'new' : v)}
      options={[
        { value: NONE, label: t('projetos-sem-projeto') },
        ...choices(projects, value === 'new' ? null : value).map((p) => ({
          value: p.id,
          label: p.name,
          hint: p.physical_asset_name ?? undefined,
        })),
        ...(allowNew ? [{ value: NEW, label: newLabel ?? t('projetos-novo-reticencias') }] : []),
      ]}
    />
  );
}

interface FormState {
  name: string;
  assetId: string | null;
  status: ProjectStatus;
  budget: number | null;
  start: string | null;
  end: string | null;
  notes: string;
}

function formOf(p: ProjectView | null, assetId?: string | null): FormState {
  return {
    name: p?.name ?? '',
    assetId: p ? p.physical_asset_id : (assetId ?? null),
    status: p?.status ?? 'active',
    budget: p?.budget ?? null,
    start: p ? p.start_date : todayIso(),
    end: p?.end_date ?? null,
    notes: p?.notes ?? '',
  };
}

/** Cadastro e edição do projeto. */
export function ProjectFormDialog({
  open,
  onOpenChange,
  project,
  assetId,
  onSaved,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  project: ProjectView | null;
  /** Bem já escolhido ao criar (pela lista de bens). */
  assetId?: string | null;
  onSaved?: (p: ProjectView) => void;
}) {
  const create = useCreateProject();
  const update = useUpdateProject();
  const { data: assets = [] } = usePhysicalAssets({ include_archived: true });
  const [form, setForm] = useState<FormState>(() => formOf(project, assetId));
  const [lastOpen, setLastOpen] = useState(open);
  if (open !== lastOpen) {
    setLastOpen(open);
    if (open) setForm(formOf(project, assetId));
  }
  const set = (p: Partial<FormState>) => setForm((f) => ({ ...f, ...p }));
  const pending = create.isPending || update.isPending;
  const error = create.error ?? update.error;
  const submit = () => {
    const body = {
      name: form.name.trim(),
      physical_asset_id: form.assetId,
      status: form.status,
      budget: form.budget,
      start_date: form.start,
      end_date: form.end,
      notes: form.notes.trim() || null,
    };
    const done = (p: ProjectView) => {
      toast({ tone: 'success', title: project ? t('projetos-salvo') : t('projetos-criado') });
      onSaved?.(p);
      onOpenChange(false);
    };
    if (project) update.mutate({ id: project.id, ...body }, { onSuccess: done });
    else
      create.mutate(
        {
          name: body.name,
          status: body.status,
          ...(body.physical_asset_id ? { physical_asset_id: body.physical_asset_id } : {}),
          ...(body.budget != null ? { budget: body.budget } : {}),
          ...(body.start_date ? { start_date: body.start_date } : {}),
          ...(body.end_date ? { end_date: body.end_date } : {}),
          ...(body.notes ? { notes: body.notes } : {}),
        },
        { onSuccess: done },
      );
  };
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={project ? t('projetos-editar', { nome: project.name }) : t('projetos-novo')}
      description={t('projetos-form-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button variant="primary" loading={pending} disabled={!form.name.trim()} onClick={submit}>
            {project ? t('comum-salvar') : t('projetos-criar')}
          </Button>
        </>
      }
    >
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('projetos-nome')} className="sm:col-span-2">
          <Input
            value={form.name}
            onChange={(e) => set({ name: e.target.value })}
            placeholder={t('projetos-nome-exemplo')}
            autoFocus
          />
        </Field>
        <Field label={t('projetos-bem')} hint={t('projetos-bem-dica')}>
          <Select
            value={form.assetId ?? NONE}
            onValueChange={(v) => set({ assetId: v === NONE ? null : v })}
            options={[
              { value: NONE, label: t('projetos-nenhum') },
              ...assets.map((a) => ({ value: a.id, label: a.name })),
            ]}
          />
        </Field>
        <Field label={t('projetos-orcamento')} hint={t('projetos-orcamento-dica')}>
          <MoneyInput value={form.budget} onValueChange={(budget) => set({ budget })} />
        </Field>
        <Field label={t('projetos-inicio')}>
          <DateInput value={form.start} onValueChange={(start) => set({ start })} />
        </Field>
        <Field label={t('projetos-fim')}>
          <DateInput value={form.end} onValueChange={(end) => set({ end })} />
        </Field>
        <Field label={t('projetos-situacao')} className="sm:col-span-2">
          <Segmented<ProjectStatus>
            label={t('projetos-situacao')}
            value={form.status}
            onChange={(status) => set({ status })}
            options={(['planned', 'active', 'done'] as const).map((s) => ({
              value: s,
              label: PROJECT_STATUS[s],
            }))}
          />
        </Field>
        <Field label={t('projetos-observacoes')} className="sm:col-span-2">
          <TextArea value={form.notes} onChange={(e) => set({ notes: e.target.value })} rows={2} />
        </Field>
      </div>
      <InlineError error={error} />
    </Dialog>
  );
}

/** Põe lançamentos já escolhidos num projeto (existente ou novo), ou tira do projeto. */
export function LinkToProjectDialog({
  ids,
  current,
  open,
  onOpenChange,
  onDone,
}: {
  ids: readonly string[];
  /** Projeto atual (um lançamento só). */
  current?: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDone?: (label: string, changeSetId: string | null) => void;
}) {
  const link = useLinkProject();
  const create = useCreateProject();
  const [choice, setChoice] = useState<string | null>(current ?? null);
  const [name, setName] = useState('');
  const [lastOpen, setLastOpen] = useState(open);
  if (open !== lastOpen) {
    setLastOpen(open);
    if (open) {
      setChoice(current ?? null);
      setName('');
    }
  }
  const n = ids.length;
  const submit = async () => {
    try {
      const projectId =
        choice === 'new' ? (await create.mutateAsync({ name: name.trim() })).id : choice;
      const res = await link.mutateAsync({ txn_ids: [...ids], project_id: projectId });
      const label = projectId ? t('projetos-lanc-no-projeto') : t('projetos-lanc-fora');
      toast({
        tone: 'success',
        title: label,
        description: t('projetos-n-lancamentos', { n: res.updated }),
      });
      onDone?.(label, res.change_set_id);
      onOpenChange(false);
    } catch {
      // O erro aparece no diálogo.
    }
  };
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="sm"
      title={t('projetos-titulo-lancamentos', { n })}
      description={t('projetos-link-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={link.isPending || create.isPending}
            disabled={n === 0 || (choice === 'new' && !name.trim())}
            onClick={() => void submit()}
          >
            {choice === null ? t('projetos-tirar') : t('projetos-por')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <Field label={t('projetos-projeto')}>
          <ProjectSelect value={choice} onChange={setChoice} allowNew />
        </Field>
        {choice === 'new' ? (
          <Field label={t('projetos-nome-novo')}>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t('projetos-nome-novo-exemplo')}
              autoFocus
            />
          </Field>
        ) : null}
      </div>
      <InlineError error={link.error ?? create.error} />
    </Dialog>
  );
}

type Side = 'todos' | 'saidas' | 'entradas';

/** Busca lançamentos (texto e período) e põe os marcados no projeto. */
export function AddTxnsDialog({
  project,
  open,
  onOpenChange,
}: {
  project: ProjectView;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const link = useLinkProject();
  const [text, setText] = useState('');
  const [from, setFrom] = useState<string | null>(
    () => project.start_date ?? addDays(todayIso(), -365),
  );
  const [to, setTo] = useState<string | null>(() => project.end_date ?? null);
  const [side, setSide] = useState<Side>('todos');
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const q = useTxnQuery(
    {
      ...(text.trim() ? { text: text.trim() } : {}),
      ...(from ? { date_from: from } : {}),
      ...(to ? { date_to: to } : {}),
      limit: 200,
    },
    open,
  );
  const rows = useMemo(
    () =>
      (q.data?.items ?? []).filter(
        (txn) =>
          txn.project_id !== project.id &&
          (side === 'todos' || (side === 'saidas' ? txn.amount < 0 : txn.amount > 0)),
      ),
    [q.data, project.id, side],
  );
  const sum = rows.filter((txn) => picked.has(txn.id)).reduce((s, txn) => s + txn.amount, 0);
  const toggle = (txn: TxnView) =>
    setPicked((cur) => {
      const next = new Set(cur);
      if (next.has(txn.id)) next.delete(txn.id);
      else next.add(txn.id);
      return next;
    });
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={t('projetos-add-titulo', { nome: project.name })}
      description={t('projetos-add-descricao')}
      footer={
        <>
          <span className="mr-auto text-[12.5px] text-dim">
            {withSlots(t('projetos-add-marcados', { n: picked.size, valor: slot(0) }), [
              <Money key="soma" cents={sum} tone="auto" />,
            ])}
          </span>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={link.isPending}
            disabled={picked.size === 0}
            onClick={() =>
              link.mutate(
                { txn_ids: [...picked], project_id: project.id },
                {
                  onSuccess: (res) => {
                    toast({
                      tone: 'success',
                      title: t('projetos-add-toast', { nome: project.name }),
                      description: t('projetos-add-toast-descricao', { n: res.updated }),
                    });
                    setPicked(new Set());
                    onOpenChange(false);
                  },
                },
              )
            }
          >
            {t('projetos-por')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-3">
        <div className="grid gap-3 sm:grid-cols-[1fr_150px_150px]">
          <Field label={t('projetos-buscar')}>
            <Input
              value={text}
              onChange={(e) => setText(e.target.value)}
              placeholder={t('projetos-buscar-exemplo')}
              autoFocus
            />
          </Field>
          <Field label={t('projetos-de')}>
            <DateInput value={from} onValueChange={setFrom} />
          </Field>
          <Field label={t('projetos-ate')}>
            <DateInput value={to} onValueChange={setTo} />
          </Field>
        </div>
        <Segmented<Side>
          label={t('projetos-entradas-saidas')}
          size="sm"
          value={side}
          onChange={setSide}
          options={[
            { value: 'todos', label: t('controles-todos') },
            { value: 'saidas', label: t('projetos-so-saidas') },
            { value: 'entradas', label: t('projetos-so-entradas') },
          ]}
        />
        <ul
          aria-label={t('projetos-encontrados')}
          className="flex max-h-[42vh] flex-col overflow-y-auto rounded-card border border-line"
        >
          {rows.length === 0 ? (
            <li className="px-3 py-4 text-[13px] text-dim">
              {q.isPending ? t('projetos-buscando') : t('projetos-nenhum-filtro')}
            </li>
          ) : (
            rows.map((txn) => (
              <li
                key={txn.id}
                className="flex items-center gap-3 border-b border-line px-3 py-1.5 last:border-b-0"
              >
                <Checkbox
                  label={`${formatDate(txn.date)} · ${txn.description || t('projetos-sem-descricao')}`}
                  checked={picked.has(txn.id)}
                  onChange={() => toggle(txn)}
                />
                <span className="ml-auto truncate text-[12px] text-dim">
                  {txn.project_name
                    ? `${t('projetos-em-projeto', { nome: txn.project_name })} · `
                    : ''}
                  {txn.account_name}
                </span>
                <Money cents={txn.amount} currency={txn.currency} tone="auto" />
              </li>
            ))
          )}
        </ul>
      </div>
      <InlineError error={link.error ?? q.error} />
    </Dialog>
  );
}

const ONLY_PROJECT = '__projeto__';

interface ItemForm {
  flow: ProjectFlow;
  date: string | null;
  amount: number | null;
  description: string;
  categoryId: string | null;
  notes: string;
  /** Conta onde lançar; `ONLY_PROJECT` = item só no projeto. */
  accountId: string;
}

function itemFormOf(entry: ProjectEntry | null, flow: ProjectFlow): ItemForm {
  return {
    flow: entry?.flow ?? flow,
    date: entry?.date ?? todayIso(),
    amount: entry ? Math.abs(entry.amount) : null,
    description: entry?.description ?? '',
    categoryId: entry?.category_id ?? null,
    notes: entry?.notes ?? '',
    accountId: ONLY_PROJECT,
  };
}

/**
 * Item lançado à mão no projeto: despesa ou receita, só no projeto (pago em dinheiro, por
 * outra pessoa, antes do app…) ou numa conta (vira um lançamento da conta, já no projeto).
 * Com `entry`, edita um item já lançado.
 */
export function ProjectItemDialog({
  project,
  entry,
  initialFlow = 'expense',
  open,
  onOpenChange,
}: {
  project: ProjectView;
  entry?: ProjectEntry | null;
  initialFlow?: ProjectFlow;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const add = useAddProjectItem();
  const update = useUpdateProjectItem();
  const createTxn = useCreateTxn();
  const { data: accounts = [] } = useAccounts();
  const [form, setForm] = useState<ItemForm>(() => itemFormOf(entry ?? null, initialFlow));
  const [lastOpen, setLastOpen] = useState(open);
  if (open !== lastOpen) {
    setLastOpen(open);
    if (open) setForm(itemFormOf(entry ?? null, initialFlow));
  }
  const set = (p: Partial<ItemForm>) => setForm((f) => ({ ...f, ...p }));
  const editing = entry != null;
  const inAccount = !editing && form.accountId !== ONLY_PROJECT;
  const pending = add.isPending || update.isPending || createTxn.isPending;
  const error = add.error ?? update.error ?? createTxn.error;
  const valid = Boolean(form.description.trim() && form.date && form.amount && form.amount > 0);
  const done = (title: string) => {
    toast({
      tone: 'success',
      title,
      description:
        form.flow === 'income'
          ? t('projetos-item-receita-em', { nome: project.name })
          : t('projetos-item-despesa-em', { nome: project.name }),
    });
    onOpenChange(false);
  };
  const submit = () => {
    if (!valid || !form.date || !form.amount) return;
    const common = {
      date: form.date,
      description: form.description.trim(),
      ...(form.categoryId ? { category_id: form.categoryId } : {}),
      ...(form.notes.trim() ? { notes: form.notes.trim() } : {}),
    };
    if (editing)
      update.mutate(
        {
          id: entry.id,
          flow: form.flow,
          amount: form.amount,
          date: form.date,
          description: common.description,
          category_id: form.categoryId,
          notes: form.notes.trim() || null,
        },
        { onSuccess: () => done(t('projetos-item-salvo')) },
      );
    else if (inAccount)
      createTxn.mutate(
        {
          ...common,
          account_id: form.accountId,
          amount: form.flow === 'income' ? form.amount : -form.amount,
          kind: form.flow,
          project_id: project.id,
        },
        { onSuccess: () => done(t('projetos-lancamento-criado')) },
      );
    else
      add.mutate(
        { ...common, project_id: project.id, flow: form.flow, amount: form.amount },
        { onSuccess: () => done(t('projetos-item-lancado')) },
      );
  };
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={
        editing
          ? t('projetos-editar', { nome: entry.description })
          : t('projetos-item-novo-em', { nome: project.name })
      }
      description={editing ? t('projetos-item-editar-descricao') : t('projetos-item-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button variant="primary" loading={pending} disabled={!valid} onClick={submit}>
            {editing
              ? t('comum-salvar')
              : inAccount
                ? t('projetos-lancar-conta')
                : t('projetos-lancar-projeto')}
          </Button>
        </>
      }
    >
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('projetos-tipo')} className="sm:col-span-2">
          <Segmented<ProjectFlow>
            label={t('projetos-tipo')}
            value={form.flow}
            onChange={(flow) => set({ flow, categoryId: null })}
            options={[
              { value: 'expense', label: t('controles-tipo-despesa') },
              { value: 'income', label: t('controles-tipo-receita') },
            ]}
          />
        </Field>
        <Field label={t('projetos-descricao')} className="sm:col-span-2">
          <Input
            value={form.description}
            onChange={(e) => set({ description: e.target.value })}
            placeholder={
              form.flow === 'income'
                ? t('projetos-item-exemplo-receita')
                : t('projetos-item-exemplo-despesa')
            }
            autoFocus
          />
        </Field>
        <Field label={t('projetos-valor')}>
          <MoneyInput value={form.amount} onValueChange={(amount) => set({ amount })} />
        </Field>
        <Field label={t('projetos-data')}>
          <DateInput value={form.date} onValueChange={(date) => set({ date })} />
        </Field>
        <Field label={t('controles-categoria')}>
          <CategoryPicker
            kind={form.flow}
            value={form.categoryId}
            onChange={(categoryId) => set({ categoryId })}
            className="[&>button]:h-9 [&>button]:w-full"
          />
        </Field>
        {editing ? (
          <span />
        ) : (
          <Field
            label={t('projetos-onde-lancar')}
            hint={inAccount ? t('projetos-onde-conta') : t('projetos-onde-projeto')}
          >
            <Select
              aria-label={t('projetos-onde-lancar')}
              value={form.accountId}
              onValueChange={(accountId) => set({ accountId })}
              options={[
                { value: ONLY_PROJECT, label: t('projetos-so-no-projeto') },
                ...accounts.map((a) => ({ value: a.id, label: a.name })),
              ]}
            />
          </Field>
        )}
        <Field label={t('projetos-observacoes')} className="sm:col-span-2">
          <TextArea value={form.notes} onChange={(e) => set({ notes: e.target.value })} rows={2} />
        </Field>
      </div>
      <InlineError error={error} />
    </Dialog>
  );
}
