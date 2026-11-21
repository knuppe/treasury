/** Metas: progresso, aporte necessário, ritmo, projeção e "no caminho"; criar e editar. */
import { useState } from 'react';
import { useSearchParams } from 'react-router';

import type { GoalKind } from '@/api/generated/GoalKind';
import type { GoalView } from '@/api/generated/GoalView';
import { isApiError } from '@/api/client';
import { useCreateGoal, useDeleteGoal, useGoals, useUpdateGoal } from '@/api/hooks/goals';
import { useAccounts } from '@/api/hooks/reports';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  EmptyState,
  Field,
  Icon,
  Input,
  InstrumentCard,
  Money,
  MoneyInput,
  Private,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { QueryView } from '@/features/reports/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatPercent, todayIso } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';

import { EducationDetail } from './education/EducationDetail';
import { EducationDialog, NumberInput } from './education/EducationDialog';
import { EducationOverviewCard } from './education/EducationOverviewCard';
import { spreadLabel } from './education/EducationResultPanel';
import { HoldingsEditor } from './education/HoldingsEditor';
import { GoalSources } from './GoalSources';

import {
  emptyGoalForm,
  formFromGoal,
  GOAL_KINDS,
  goalInput,
  goalKindLabel,
  goalPatch,
  goalSituation,
  hasChanges,
  paceSource,
  SITUATION_LABEL,
  validateGoal,
  type GoalForm,
  type GoalFormErrors,
} from './model';

const errorText = (e: unknown) =>
  isApiError(e) || e instanceof Error ? e.message : t('metas-erro-gravar');

/** Barra de progresso (medidor): trilha recessiva, preenchimento no tom da situação. */
export function ProgressMeter({
  value,
  label,
  tone = 'var(--accent)',
}: {
  value: number;
  label: string;
  tone?: string;
}) {
  const pct = Math.max(0, Math.min(1, value));
  return (
    <div
      role="meter"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(value * 100)}
      className="h-2 w-full overflow-hidden rounded-[2px] bg-line/60"
    >
      <div
        className="h-full rounded-r-[4px]"
        style={{ width: `${pct * 100}%`, background: tone }}
      />
    </div>
  );
}

// ── Formulário ───────────────────────────────────────────────────

export function GoalDialog({
  goal,
  initial,
  open,
  onOpenChange,
  onEducation,
}: {
  goal?: GoalView;
  initial?: Partial<GoalForm>;
  open: boolean;
  onOpenChange: (o: boolean) => void;
  /** Ao escolher "Educação" numa meta nova: o assistente da meta de faculdade. */
  onEducation?: () => void;
}) {
  const [form, setForm] = useState<GoalForm>(() =>
    goal ? formFromGoal(goal) : { ...emptyGoalForm(initial?.kind), ...initial },
  );
  const [errors, setErrors] = useState<GoalFormErrors>({});
  const accounts = useAccounts();
  const base = useBaseCurrency();
  const create = useCreateGoal();
  const update = useUpdateGoal();
  const pending = create.isPending || update.isPending;
  const error = create.error ?? update.error;
  const set = <K extends keyof GoalForm>(k: K, v: GoalForm[K]) =>
    setForm((f) => ({ ...f, [k]: v }));

  const submit = () => {
    const e = validateGoal(form, todayIso());
    setErrors(e);
    if (Object.keys(e).length) return;
    if (goal) {
      const patch = goalPatch(goal, form);
      if (!hasChanges(patch)) {
        onOpenChange(false);
        return;
      }
      update.mutate(patch, {
        onSuccess: () => {
          onOpenChange(false);
          toast({ tone: 'success', title: t('metas-atualizada'), description: form.name });
        },
      });
    } else {
      create.mutate(goalInput(form), {
        onSuccess: () => {
          onOpenChange(false);
          toast({ tone: 'success', title: t('metas-criada'), description: form.name });
        },
      });
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={goal ? t('metas-editar-titulo', { nome: goal.name }) : t('metas-nova')}
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button variant="primary" loading={pending} onClick={submit}>
            {goal ? t('comum-salvar') : t('metas-criar')}
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
        <Field label={t('metas-nome')} error={errors.name} className="sm:col-span-2">
          <Input value={form.name} onChange={(e) => set('name', e.target.value)} autoFocus />
        </Field>
        <Field label={t('metas-tipo')}>
          <Select
            value={form.kind}
            onValueChange={(v) => {
              if (v === 'education' && !goal && onEducation) {
                onOpenChange(false);
                onEducation();
                return;
              }
              set('kind', v as GoalKind);
            }}
            options={GOAL_KINDS.map((k) => ({ value: k.value, label: k.label }))}
          />
        </Field>
        <Field label={t('metas-valor-alvo')} error={errors.target}>
          <MoneyInput value={form.target} onValueChange={(v) => set('target', v)} />
        </Field>
        <Field
          label={t('metas-data-alvo')}
          hint={t('metas-data-alvo-dica')}
          error={errors.targetDate}
        >
          <DateInput value={form.targetDate} onValueChange={(v) => set('targetDate', v)} />
        </Field>
        <Field label={t('metas-ja-guardado')}>
          <MoneyInput value={form.initial} onValueChange={(v) => set('initial', v)} />
        </Field>
        <Field label={t('metas-aporte-planejado')} hint={t('metas-aporte-planejado-dica')}>
          <MoneyInput value={form.monthly} onValueChange={(v) => set('monthly', v)} />
        </Field>
        <Field
          label={t('metas-rendimento')}
          hint={t('metas-rendimento-dica')}
          error={errors.realReturn}
          className="sm:col-span-2"
        >
          <div className="w-32">
            <NumberInput
              value={form.realReturn}
              onValue={(v) => set('realReturn', v)}
              suffix="%"
              label={t('metas-rendimento')}
              invalid={!!errors.realReturn}
            />
          </div>
        </Field>
        <fieldset className="flex flex-col gap-1.5 sm:col-span-2">
          <legend className="label-caps mb-1.5">{t('metas-contas-vinculadas')}</legend>
          <p className="text-[12px] text-dim">{t('metas-contas-vinculadas-dica')}</p>
          <div className="grid grid-cols-2 gap-x-4 gap-y-1 md:grid-cols-3">
            {(accounts.data ?? [])
              .filter((a) => !a.archived && a.kind !== 'credit_card')
              .map((a) => (
                <label key={a.id} className="flex items-center gap-2 text-[13px] text-fg">
                  <input
                    type="checkbox"
                    checked={form.accountIds.includes(a.id)}
                    onChange={(e) =>
                      set(
                        'accountIds',
                        e.target.checked
                          ? [...form.accountIds, a.id]
                          : form.accountIds.filter((x) => x !== a.id),
                      )
                    }
                  />
                  <span className="truncate">{a.name}</span>
                  {a.currency !== base ? <span className="label-caps">{a.currency}</span> : null}
                </label>
              ))}
          </div>
        </fieldset>
        <Field label={t('metas-notas')} className="sm:col-span-2">
          <Input value={form.notes} onChange={(e) => set('notes', e.target.value)} />
        </Field>
        <button type="submit" hidden />
      </form>
      {error ? (
        <p role="alert" className="text-[12.5px] text-negative">
          {errorText(error)}
        </p>
      ) : null}
    </Dialog>
  );
}

// ── Card ─────────────────────────────────────────────────────────

/** "8 anos e 3 meses" até o início do curso. */
function timeLeft(months: number): string {
  const years = Math.floor(months / 12);
  const rest = months % 12;
  const parts = [
    years > 0 ? t('educacao-n-anos', { n: years }) : null,
    rest > 0 || years === 0 ? t('educacao-n-meses', { n: rest }) : null,
  ].filter((x): x is string => x != null);
  return parts.join(t('educacao-e'));
}

/** Meta com rendimento: quanto vem de aportes e de rendimento, e o alvo na moeda da época. */
function GrowthLine({ goal }: { goal: GoalView }) {
  const g = goal.growth!;
  const money = (key: string, cents: number) => (
    <Money key={key} cents={cents} currency={goal.currency} />
  );
  return (
    <div className="flex flex-col gap-1 text-[12px] text-dim">
      {g.from_contributions != null && g.from_returns != null ? (
        <p>
          {withSlots(
            t('metas-rendimento-linha', {
              taxa: formatPercent(g.real_return, { decimals: 1 }),
              aportes: slot(0),
              rendimento: slot(1),
              linear: slot(2),
            }),
            [
              money('aportes', g.from_contributions),
              money('rendimento', g.from_returns),
              money('linear', g.linear_monthly_needed ?? 0),
            ],
          )}
        </p>
      ) : null}
      {g.target_nominal != null && g.inflation != null ? (
        <p>
          {withSlots(
            t('metas-rendimento-alvo-epoca', {
              inflacao: formatPercent(g.inflation, { decimals: 1 }),
              valor: slot(0),
            }),
            [money('epoca', g.target_nominal)],
          )}
        </p>
      ) : null}
      <p>{t('metas-rendimento-premissa')}</p>
    </div>
  );
}

function GoalCard({
  goal,
  index,
  accountNames,
}: {
  goal: GoalView;
  index: number;
  accountNames: Map<string, string>;
}) {
  const [editing, setEditing] = useState(false);
  const [confirm, setConfirm] = useState(false);
  const [linking, setLinking] = useState(false);
  const edu = goal.education;
  const update = useUpdateGoal();
  const remove = useDeleteGoal();
  const sit = goalSituation(goal);
  const s = SITUATION_LABEL[sit];
  const pace = paceSource(goal);
  const tone =
    sit === 'done' || sit === 'on_track'
      ? 'var(--positive)'
      : sit === 'off_track'
        ? 'var(--warning)'
        : 'var(--accent)';
  return (
    <InstrumentCard
      index={String(index + 1).padStart(2, '0')}
      title={goal.name}
      footer={[
        { label: t('metas-tipo'), value: goalKindLabel(goal.kind) },
        edu
          ? {
              label: t('educacao-filho'),
              value: edu.child_name ?? t('educacao-filho-vai-nascer'),
            }
          : {
              label: t('metas-contas'),
              value: goal.account_ids.length
                ? goal.account_ids.map((id) => accountNames.get(id) ?? id).join(', ')
                : t('metas-nenhuma-conta'),
            },
      ]}
      className={cx(goal.archived && 'opacity-70')}
    >
      <div className="flex flex-col gap-3">
        <div className="flex items-baseline justify-between gap-3">
          <span className="num text-[22px] text-fg">
            <Money
              cents={goal.current_amount}
              currency={goal.currency}
              estimated={goal.estimated}
            />
          </span>
          <span className="num text-[12.5px] text-dim">
            {withSlots(t('metas-de-alvo', { valor: slot(0) }), [
              <Money key="alvo" cents={goal.target_amount} currency={goal.currency} />,
            ])}
          </span>
        </div>
        <ProgressMeter
          value={goal.progress}
          label={t('metas-progresso-de', { nome: goal.name })}
          tone={tone}
        />
        <div className="flex items-center justify-between text-[12px]">
          <span className={cx('label-caps', s.tone)}>
            <span aria-hidden>{s.icon}</span> {s.label}
          </span>
          <span className="num text-dim">
            <Private>{formatPercent(goal.progress, { decimals: 0 })}</Private>
          </span>
        </div>
        <dl className="grid grid-cols-2 gap-x-4 gap-y-2 border-t border-line pt-3 text-[12.5px]">
          <div>
            <dt className="label-caps">{t('metas-falta')}</dt>
            <dd className="num text-fg">
              <Money cents={goal.remaining} currency={goal.currency} />
            </dd>
          </div>
          <div>
            <dt className="label-caps">{t('metas-data-alvo')}</dt>
            <dd className="num text-fg">{goal.target_date ? formatDate(goal.target_date) : '—'}</dd>
          </div>
          <div>
            <dt className="label-caps">{t('metas-aporte-necessario')}</dt>
            <dd className="num text-fg">
              {goal.monthly_needed == null ? (
                '—'
              ) : (
                <Money cents={goal.monthly_needed} currency={goal.currency} />
              )}
              {goal.monthly_needed != null ? (
                <span className="text-dim">{t('metas-por-mes')}</span>
              ) : null}
            </dd>
          </div>
          <div>
            <dt className="label-caps">
              {pace === 'planned'
                ? t('metas-ritmo-planejado')
                : pace === 'real'
                  ? t('metas-ritmo-real')
                  : t('metas-ritmo')}
            </dt>
            <dd className="num text-fg">
              {pace === 'none' ? (
                '—'
              ) : (
                <>
                  <Money
                    cents={
                      pace === 'real'
                        ? goal.avg_monthly_contribution
                        : (goal.monthly_contribution ?? 0)
                    }
                    currency={goal.currency}
                  />
                  <span className="text-dim">{t('metas-por-mes')}</span>
                </>
              )}
            </dd>
          </div>
          <div className="col-span-2">
            <dt className="label-caps">{t('metas-projecao')}</dt>
            <dd className="num text-fg">
              {goal.projected_date
                ? pace === 'planned'
                  ? t('metas-projecao-planejada', { data: formatDate(goal.projected_date) })
                  : formatDate(goal.projected_date)
                : t('metas-sem-ritmo')}
            </dd>
          </div>
        </dl>
        {goal.growth ? <GrowthLine goal={goal} /> : null}
        {edu ? (
          <p className="text-[12px] text-dim">
            {[
              edu.phase === 'saving'
                ? t('educacao-card-faltam', { prazo: timeLeft(edu.months_to_start) })
                : edu.phase === 'paying'
                  ? t('educacao-card-em-curso')
                  : t('educacao-card-terminou'),
              spreadLabel(edu.spread),
            ].join(' · ')}
          </p>
        ) : null}
        {goal.missing_fx.length ? (
          <p className="text-[12px] text-warning">
            {t('metas-sem-cambio', { moedas: goal.missing_fx.join(', ') })}
          </p>
        ) : null}
        {goal.stale_sources.length ? (
          <p className="text-[12px] text-warning">
            {t('metas-fontes-desatualizadas', {
              fontes: goal.stale_sources.map((s) => s.name).join(', '),
            })}
          </p>
        ) : null}
        <div className="-mx-1 flex flex-wrap gap-1 border-t border-line pt-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setEditing(true)}
            aria-label={
              edu
                ? t('educacao-ver-plano-nome', { nome: goal.name })
                : t('metas-editar-nome', { nome: goal.name })
            }
          >
            {edu ? t('educacao-ver-plano') : t('comum-editar')}
          </Button>
          {edu ? null : (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setLinking(true)}
              aria-label={t('metas-fontes-nome', { nome: goal.name })}
            >
              {t('metas-fontes')}
            </Button>
          )}
          <Button
            variant="ghost"
            size="sm"
            aria-label={
              goal.archived
                ? t('metas-reativar-nome', { nome: goal.name })
                : t('metas-arquivar-nome', { nome: goal.name })
            }
            onClick={() =>
              update.mutate(
                { id: goal.id, archived: !goal.archived },
                {
                  onError: (e) =>
                    toast({
                      tone: 'error',
                      title: t('metas-nao-arquivou'),
                      description: errorText(e),
                    }),
                },
              )
            }
          >
            {goal.archived ? t('metas-reativar') : t('metas-arquivar')}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setConfirm(true)}
            aria-label={t('metas-excluir-nome', { nome: goal.name })}
          >
            {t('metas-excluir')}
          </Button>
        </div>
      </div>
      {editing && edu ? (
        <EducationDetail goalId={goal.id} open={editing} onOpenChange={setEditing} />
      ) : editing ? (
        <GoalDialog goal={goal} open={editing} onOpenChange={setEditing} />
      ) : null}
      {linking ? (
        <Dialog
          open={linking}
          onOpenChange={setLinking}
          size="lg"
          title={t('metas-fontes-titulo', { nome: goal.name })}
        >
          <div className="flex flex-col gap-6">
            <GoalSources goal={goal} />
            <HoldingsEditor goalId={goal.id} targetDate={goal.target_date} />
          </div>
        </Dialog>
      ) : null}
      <Dialog
        open={confirm}
        onOpenChange={setConfirm}
        size="sm"
        title={t('metas-excluir-titulo')}
        description={t('metas-excluir-descricao', { nome: goal.name })}
        footer={
          <>
            <DialogClose asChild>
              <Button variant="ghost">{t('comum-cancelar')}</Button>
            </DialogClose>
            <Button
              variant="danger"
              loading={remove.isPending}
              onClick={() =>
                remove.mutate(goal.id, {
                  onSuccess: () => {
                    setConfirm(false);
                    toast({ title: t('metas-excluida'), description: goal.name });
                  },
                  onError: (e) =>
                    toast({
                      tone: 'error',
                      title: t('metas-nao-excluiu'),
                      description: errorText(e),
                    }),
                })
              }
            >
              {t('metas-excluir')}
            </Button>
          </>
        }
      />
    </InstrumentCard>
  );
}

// ── Lista ────────────────────────────────────────────────────────

export function GoalsPanel() {
  const [showArchived, setShowArchived] = useState(false);
  // `?novo=meta` (barra ⌘K): a meta nova abre pela URL e, ao fechar, o parâmetro sai.
  const [params, setParams] = useSearchParams();
  const [manual, setManual] = useState(false);
  const [education, setEducation] = useState(false);
  const fromUrl = params.get('novo') === 'meta';
  const creating = manual || fromUrl;
  const setCreating = (open: boolean) => {
    setManual(open);
    if (!open && fromUrl) {
      const next = new URLSearchParams(params);
      next.delete('novo');
      setParams(next, { replace: true });
    }
  };
  const goals = useGoals({ include_archived: showArchived });
  const accounts = useAccounts();
  const names = new Map((accounts.data ?? []).map((a) => [a.id, a.name]));
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-3">
        <Button
          variant="primary"
          size="sm"
          icon={<Icon name="plus" size={13} />}
          onClick={() => setCreating(true)}
        >
          {t('metas-nova')}
        </Button>
        <Button variant="secondary" size="sm" onClick={() => setEducation(true)}>
          {t('educacao-nova')}
        </Button>
        <label className="flex items-center gap-2 text-[13px] text-dim">
          <input
            type="checkbox"
            checked={showArchived}
            onChange={(e) => setShowArchived(e.target.checked)}
          />
          {t('metas-mostrar-arquivadas')}
        </label>
      </div>
      <EducationOverviewCard />
      <QueryView
        query={goals}
        isEmpty={(d) => d.length === 0}
        empty={
          <EmptyState
            kicker={t('metas-vazio-kicker')}
            title={t('metas-vazio-titulo')}
            description={t('metas-vazio-descricao')}
            actions={
              <Button variant="primary" size="sm" onClick={() => setCreating(true)}>
                {t('metas-criar')}
              </Button>
            }
          />
        }
      >
        {(list) => (
          <div className="grid gap-4 @min-[530px]:grid-cols-2 @min-[1040px]:grid-cols-3">
            {list.map((g, i) => (
              <GoalCard key={g.id} goal={g} index={i} accountNames={names} />
            ))}
          </div>
        )}
      </QueryView>
      {creating ? (
        <GoalDialog
          open={creating}
          onOpenChange={setCreating}
          onEducation={() => setEducation(true)}
        />
      ) : null}
      {education ? <EducationDialog open={education} onOpenChange={setEducation} /> : null}
    </div>
  );
}
