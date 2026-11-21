/**
 * Aportes programados (`portfolio.plans*`): "comprar R$ 500 do ativo X todo dia 10". Nada é
 * gravado sozinho: os aportes vencidos aparecem para confirmar (com o preço do dia e a
 * quantidade que o valor compra, editáveis) ou pular, como os proventos a confirmar.
 */
import { useState } from 'react';

import type { InvestmentPlanView } from '@/api/generated/InvestmentPlanView';
import type { PlanDue } from '@/api/generated/PlanDue';
import { useAssets } from '@/api/hooks/assets';
import { useGoals } from '@/api/hooks/goals';
import {
  useConfirmPlan,
  useDeletePlan,
  useInvestmentAccounts,
  useInvestmentPlans,
  usePlansPending,
  useSavePlan,
  useSkipPlan,
} from '@/api/hooks/portfolio';
import { useUndoToast } from '@/api/hooks/undoToast';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Input,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';

import { Check } from './components/Fields';
import { ErrorBlock, errorMessage, LoadingBlock } from './components/States';

const NONE = '__none__';

function DueRow({ due }: { due: PlanDue }) {
  const [qty, setQty] = useState(due.quantity ?? '');
  const [price, setPrice] = useState(due.price ?? '');
  const confirm = useConfirmPlan();
  const skip = useSkipPlan();
  const undoToast = useUndoToast();
  const label = due.ticker ?? due.asset_name;
  const onError = (e: Error) =>
    toast({ tone: 'error', title: t('carteira-plano-erro'), description: errorMessage(e) });
  return (
    <li className="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-line/60 py-2 last:border-0">
      <div className="flex min-w-48 flex-1 flex-col">
        <span className="text-[13px] text-fg">
          {label} · <Money cents={due.amount} currency={due.currency} />
        </span>
        <span className="text-[11.5px] text-dim">
          {formatDate(due.due_date)} · {due.plan_name} · {due.account_name}
          {due.price && due.price_date && due.price_date !== due.due_date
            ? ` · ${t('carteira-plano-preco-de', { data: formatDate(due.price_date) })}`
            : ''}
          {!due.price ? ` · ${t('carteira-plano-sem-preco')}` : ''}
        </span>
      </div>
      <div className="w-24">
        <Input
          numeric
          inputMode="decimal"
          aria-label={t('carteira-plano-quantidade-de', {
            ativo: label,
            data: formatDate(due.due_date),
          })}
          value={qty}
          onChange={(e) => setQty(e.target.value.replace(',', '.'))}
        />
      </div>
      <div className="w-28">
        <Input
          numeric
          inputMode="decimal"
          aria-label={t('carteira-plano-preco-rotulo', {
            ativo: label,
            data: formatDate(due.due_date),
          })}
          value={price}
          onChange={(e) => setPrice(e.target.value.replace(',', '.'))}
        />
      </div>
      <div className="flex gap-1">
        <Button
          size="sm"
          variant="primary"
          disabled={!qty || !price}
          loading={confirm.isPending}
          onClick={() =>
            confirm.mutate(
              { plan_id: due.plan_id, due_date: due.due_date, quantity: qty, price },
              {
                onSuccess: (r) =>
                  undoToast({
                    title: t('carteira-plano-confirmado', { ativo: label }),
                    changeSetId: r.change_set_id,
                  }),
                onError,
              },
            )
          }
        >
          {t('carteira-plano-confirmar')}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          loading={skip.isPending}
          onClick={() =>
            skip.mutate(
              { plan_id: due.plan_id, due_date: due.due_date },
              {
                onSuccess: (r) =>
                  undoToast({
                    title: t('carteira-plano-pulado', { ativo: label }),
                    changeSetId: r.change_set_id,
                  }),
                onError,
              },
            )
          }
        >
          {t('carteira-plano-pular')}
        </Button>
      </div>
    </li>
  );
}

interface PlanForm {
  name: string;
  assetId: string;
  accountId: string;
  amount: number | null;
  day: string;
  start: string | null;
  end: string | null;
  goalId: string;
  active: boolean;
}

function PlanDialog({
  plan,
  open,
  onOpenChange,
}: {
  plan?: InvestmentPlanView;
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const assets = useAssets({});
  const accounts = useInvestmentAccounts();
  const goals = useGoals({});
  const save = useSavePlan();
  const [f, setF] = useState<PlanForm>(() => ({
    name: plan?.name ?? '',
    assetId: plan?.asset_id ?? '',
    accountId: plan?.account_id ?? '',
    amount: plan?.amount ?? null,
    day: String(plan?.day_of_month ?? 10),
    start: plan?.start_date ?? todayIso(),
    end: plan?.end_date ?? null,
    goalId: plan?.goal_id ?? NONE,
    active: plan?.active ?? true,
  }));
  const set = <K extends keyof PlanForm>(k: K, v: PlanForm[K]) => setF((x) => ({ ...x, [k]: v }));
  const day = Number(f.day);
  const valid =
    f.name.trim() &&
    f.assetId &&
    f.accountId &&
    (f.amount ?? 0) > 0 &&
    Number.isInteger(day) &&
    day >= 1 &&
    day <= 28 &&
    f.start;
  const submit = () => {
    if (!valid) return;
    save.mutate(
      {
        ...(plan ? { id: plan.id } : {}),
        name: f.name.trim(),
        asset_id: f.assetId,
        account_id: f.accountId,
        amount: f.amount ?? 0,
        day_of_month: day,
        start_date: f.start!,
        ...(f.end ? { end_date: f.end } : {}),
        ...(f.goalId !== NONE ? { goal_id: f.goalId } : {}),
        active: f.active,
      },
      {
        onSuccess: () => {
          onOpenChange(false);
          toast({ tone: 'success', title: t('carteira-plano-salvo'), description: f.name });
        },
      },
    );
  };
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={plan ? t('carteira-plano-editar') : t('carteira-plano-novo')}
      description={t('carteira-plano-dialogo-texto')}
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button variant="primary" disabled={!valid} loading={save.isPending} onClick={submit}>
            {t('comum-salvar')}
          </Button>
        </>
      }
    >
      <form
        className="grid gap-4 sm:grid-cols-2"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <Field label={t('carteira-plano-nome')} className="sm:col-span-2">
          <Input value={f.name} onChange={(e) => set('name', e.target.value)} />
        </Field>
        <Field label={t('carteira-plano-ativo')}>
          <Select
            aria-label={t('carteira-plano-ativo')}
            value={f.assetId}
            onValueChange={(v) => set('assetId', v)}
            options={(assets.data ?? []).map((a) => ({
              value: a.id,
              label: a.ticker ? `${a.ticker} · ${a.name}` : a.name,
            }))}
          />
        </Field>
        <Field label={t('carteira-plano-conta')}>
          <Select
            aria-label={t('carteira-plano-conta')}
            value={f.accountId}
            onValueChange={(v) => set('accountId', v)}
            options={(accounts.data ?? []).map((a) => ({ value: a.id, label: a.name }))}
          />
        </Field>
        <Field label={t('carteira-plano-valor')}>
          <MoneyInput value={f.amount} onValueChange={(v) => set('amount', v)} />
        </Field>
        <Field label={t('carteira-plano-dia')} hint={t('carteira-plano-dia-dica')}>
          <Input
            numeric
            inputMode="numeric"
            value={f.day}
            onChange={(e) => set('day', e.target.value)}
          />
        </Field>
        <Field label={t('carteira-plano-inicio')}>
          <DateInput value={f.start} onValueChange={(v) => set('start', v)} />
        </Field>
        <Field label={t('carteira-plano-fim')} hint={t('carteira-plano-fim-dica')}>
          <DateInput value={f.end} onValueChange={(v) => set('end', v)} />
        </Field>
        <Field label={t('carteira-plano-meta')} hint={t('carteira-plano-meta-dica')}>
          <Select
            aria-label={t('carteira-plano-meta')}
            value={f.goalId}
            onValueChange={(v) => set('goalId', v)}
            options={[
              { value: NONE, label: t('carteira-plano-sem-meta') },
              ...(goals.data ?? []).map((g) => ({ value: g.id, label: g.name })),
            ]}
          />
        </Field>
        <div className="flex items-end">
          <Check
            checked={f.active}
            onChange={(v) => set('active', v)}
            label={t('carteira-plano-ativo-ligado')}
          />
        </div>
        <button type="submit" hidden />
      </form>
      {save.error ? (
        <p role="alert" className="text-[12.5px] text-negative">
          {errorMessage(save.error)}
        </p>
      ) : null}
    </Dialog>
  );
}

export function ScheduledContributions() {
  const plans = useInvestmentPlans();
  const due = usePlansPending();
  const remove = useDeletePlan();
  const undoToast = useUndoToast();
  const [editing, setEditing] = useState<InvestmentPlanView | 'new' | null>(null);
  return (
    <section aria-label={t('carteira-plano-titulo')} className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h3 className="label-caps text-fg">{t('carteira-plano-titulo')}</h3>
        <Button size="sm" onClick={() => setEditing('new')}>
          {t('carteira-plano-novo')}
        </Button>
      </div>
      {due.data?.length ? (
        <div className="rounded-card border border-accent/40 bg-accent-soft/30 px-3 py-2">
          <p className="text-[12.5px] text-fg">
            {t('carteira-plano-a-confirmar', { n: due.data.length })}
          </p>
          <ul aria-label={t('carteira-plano-a-confirmar-lista')} className="flex flex-col">
            {due.data.map((d) => (
              <DueRow key={`${d.plan_id}-${d.due_date}`} due={d} />
            ))}
          </ul>
        </div>
      ) : null}
      {plans.isPending ? (
        <LoadingBlock />
      ) : plans.isError ? (
        <ErrorBlock error={plans.error} onRetry={() => void plans.refetch()} />
      ) : plans.data.length === 0 ? (
        <p className="text-[12.5px] text-dim">{t('carteira-plano-vazio')}</p>
      ) : (
        <ul
          aria-label={t('carteira-plano-lista')}
          className="flex flex-col rounded-card border border-line bg-surface px-3"
        >
          {plans.data.map((p) => (
            <li
              key={p.id}
              className="flex flex-wrap items-center gap-x-4 gap-y-1 border-b border-line/60 py-2 last:border-0"
            >
              <div className="flex min-w-48 flex-1 flex-col">
                <span className="text-[13px] text-fg">
                  {p.name}
                  {p.active ? '' : ` · ${t('carteira-plano-pausado')}`}
                </span>
                <span className="text-[11.5px] text-dim">
                  {t('carteira-plano-resumo', {
                    ativo: p.ticker ?? p.asset_name,
                    dia: p.day_of_month,
                    conta: p.account_name,
                  })}
                  {p.goal_name ? ` · ${p.goal_name}` : ''}
                  {p.next_due
                    ? ` · ${t('carteira-plano-proximo', { data: formatDate(p.next_due) })}`
                    : ''}
                </span>
              </div>
              <span className="num text-[13px] text-fg">
                <Money cents={p.amount} currency={p.currency} />
              </span>
              <div className="flex gap-1">
                <Button size="sm" variant="ghost" onClick={() => setEditing(p)}>
                  {t('comum-editar')}
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  aria-label={t('carteira-plano-excluir-nome', { nome: p.name })}
                  onClick={() =>
                    remove.mutate(
                      { id: p.id },
                      {
                        onSuccess: (r) =>
                          undoToast({
                            title: t('carteira-plano-excluido'),
                            changeSetId: r.change_set_id,
                          }),
                      },
                    )
                  }
                >
                  {t('carteira-excluir')}
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}
      {editing ? (
        <PlanDialog
          plan={editing === 'new' ? undefined : editing}
          open
          onOpenChange={(o) => !o && setEditing(null)}
        />
      ) : null}
    </section>
  );
}
