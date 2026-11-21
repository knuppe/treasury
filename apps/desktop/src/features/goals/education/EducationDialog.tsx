/**
 * Assistente da meta de faculdade (docs/design/college-goal.md §9): quem, qual faculdade e
 * premissas, com o resultado da conta a cada passo (simulação no núcleo, sem gravar).
 */
import { useMemo, useState } from 'react';

import type { EducationPlanView } from '@/api/generated/EducationPlanView';
import type { InflationMode } from '@/api/generated/InflationMode';
import type { InstitutionType } from '@/api/generated/InstitutionType';
import { isApiError } from '@/api/client';
import { useEducationInflation, useEducationSimulate, useSaveEducation } from '@/api/hooks/goals';
import { useMembers } from '@/api/hooks/reports';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Input,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { Segmented } from '@/features/portfolio/components/Segmented';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';

import { EducationResultPanel } from './EducationResultPanel';
import {
  ageOn,
  childCandidates,
  emptyEducationForm,
  feeDateFor,
  firstErrorStep,
  formFromPlan,
  parseNumber,
  PAYMENTS_PER_YEAR,
  planFromForm,
  validateEducation,
  type EducationForm,
  type EducationFormErrors,
} from './model';

const errorText = (e: unknown) =>
  isApiError(e) || e instanceof Error ? e.message : t('metas-erro-gravar');

const STEPS = [
  () => t('educacao-passo-quem'),
  () => t('educacao-passo-faculdade'),
  () => t('educacao-passo-premissas'),
] as const;

/** Número com vírgula decimal (vazio = `null`). */
export function NumberInput({
  value,
  onValue,
  suffix,
  label,
  invalid,
}: {
  value: number | null;
  onValue: (v: number | null) => void;
  suffix?: string;
  label?: string;
  invalid?: boolean;
}) {
  const [text, setText] = useState(value == null ? '' : String(value).replace('.', ','));
  return (
    <Input
      numeric
      inputMode="decimal"
      aria-label={label}
      aria-invalid={invalid || undefined}
      value={text}
      suffix={suffix}
      onChange={(e) => {
        setText(e.target.value);
        onValue(parseNumber(e.target.value));
      }}
    />
  );
}

export function EducationDialog({
  plan,
  open,
  onOpenChange,
  initialStep = 0,
}: {
  /** Editar um plano (sem ele, cria a meta). */
  plan?: EducationPlanView;
  open: boolean;
  onOpenChange: (o: boolean) => void;
  initialStep?: 0 | 1 | 2;
}) {
  const members = useMembers();
  const inflation = useEducationInflation();
  const save = useSaveEducation();
  const today = todayIso();
  const candidates = useMemo(() => childCandidates(members.data ?? []), [members.data]);
  const [form, setForm] = useState<EducationForm>(() =>
    plan ? formFromPlan(plan) : emptyEducationForm(null),
  );
  const [step, setStep] = useState<0 | 1 | 2>(initialStep);
  const [errors, setErrors] = useState<EducationFormErrors>({});
  const set = <K extends keyof EducationForm>(k: K, v: EducationForm[K]) =>
    setForm((f) => ({ ...f, [k]: v }));
  // Sem filho escolhido, o primeiro da lista (filhos primeiro).
  const memberId = form.memberId ?? candidates[0]?.id ?? null;
  const current = { ...form, memberId };
  const member = candidates.find((m) => m.id === memberId);
  const birth = current.childMode === 'member' ? member?.profile.birth_date : current.expectedBirth;
  const feeDate = feeDateFor(current, plan?.plan ?? null, today);
  const planArgs = planFromForm(current, feeDate, plan?.plan);
  const ready = birth != null && (current.institution === 'public' || (current.fee ?? 0) > 0);
  const sim = useEducationSimulate(
    ready
      ? {
          plan: planArgs,
          goal_id: plan?.goal.id,
          ...(plan ? {} : { saved: current.initial ?? 0 }),
        }
      : null,
  );

  const submit = () => {
    const e = validateEducation(current, candidates);
    setErrors(e);
    const s = firstErrorStep(e);
    if (s != null) {
      setStep(s);
      return;
    }
    save.mutate(
      {
        goal_id: plan?.goal.id,
        name: current.name.trim() || undefined,
        plan: planArgs,
        initial_amount: current.initial ?? 0,
      },
      {
        onSuccess: (v) => {
          onOpenChange(false);
          toast({
            tone: 'success',
            title: plan ? t('metas-atualizada') : t('metas-criada'),
            description: v.goal.name,
          });
        },
      },
    );
  };

  const monthNames = t('comum-meses').split(',');
  const inflationOptions: { value: InflationMode; label: string }[] = [
    {
      value: 'series_10y',
      label: inflation.data?.series_name
        ? t('educacao-inflacao-serie-nome', { serie: inflation.data.series_name })
        : t('educacao-inflacao-serie'),
    },
    { value: 'headline', label: t('educacao-inflacao-geral') },
    { value: 'fixed_spread', label: t('educacao-inflacao-fixa') },
  ];

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={plan ? t('educacao-editar-titulo', { nome: plan.goal.name }) : t('educacao-nova')}
      description={t('educacao-descricao')}
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          {step > 0 ? (
            <Button variant="ghost" onClick={() => setStep((s) => (s - 1) as 0 | 1 | 2)}>
              {t('educacao-voltar')}
            </Button>
          ) : null}
          {step < 2 ? (
            <Button variant="primary" onClick={() => setStep((s) => (s + 1) as 0 | 1 | 2)}>
              {t('educacao-avancar')}
            </Button>
          ) : (
            <Button variant="primary" loading={save.isPending} onClick={submit}>
              {plan ? t('comum-salvar') : t('metas-criar')}
            </Button>
          )}
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <Segmented
          label={t('educacao-passos')}
          value={String(step)}
          onChange={(v) => setStep(Number(v) as 0 | 1 | 2)}
          options={STEPS.map((label, i) => ({ value: String(i), label: `${i + 1}. ${label()}` }))}
          size="sm"
        />
        <form
          className="grid gap-4 sm:grid-cols-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (step < 2) setStep((s) => (s + 1) as 0 | 1 | 2);
            else submit();
          }}
        >
          {step === 0 ? (
            <>
              <div className="sm:col-span-2">
                <Segmented
                  label={t('educacao-quem')}
                  value={current.childMode}
                  onChange={(v) => set('childMode', v)}
                  options={[
                    { value: 'member', label: t('educacao-filho-cadastrado') },
                    { value: 'unborn', label: t('educacao-filho-vai-nascer') },
                  ]}
                  size="sm"
                />
              </div>
              {current.childMode === 'member' ? (
                <Field
                  label={t('educacao-filho')}
                  error={errors.memberId}
                  hint={
                    member?.profile.birth_date
                      ? t('educacao-filho-dica', {
                          idade: ageOn(member.profile.birth_date, today),
                          ano: String(
                            Number(member.profile.birth_date.slice(0, 4)) + current.startAge,
                          ),
                          inicio: current.startAge,
                        })
                      : member
                        ? t('educacao-erro-nascimento')
                        : undefined
                  }
                  className="sm:col-span-2"
                >
                  <Select
                    value={memberId ?? ''}
                    onValueChange={(v) => set('memberId', v)}
                    options={candidates.map((m) => ({ value: m.id, label: m.name }))}
                    placeholder={t('educacao-escolha-filho')}
                  />
                </Field>
              ) : (
                <Field
                  label={t('educacao-nascimento-previsto')}
                  error={errors.expectedBirth}
                  className="sm:col-span-2"
                >
                  <DateInput
                    value={current.expectedBirth}
                    onValueChange={(v) => set('expectedBirth', v)}
                  />
                </Field>
              )}
              <Field label={t('educacao-idade-inicio')} error={errors.startAge}>
                <NumberInput
                  value={current.startAge}
                  onValue={(v) => set('startAge', Math.round(v ?? 0))}
                  label={t('educacao-idade-inicio')}
                  invalid={!!errors.startAge}
                />
              </Field>
              <Field label={t('educacao-mes-inicio')} hint={t('educacao-mes-inicio-dica')}>
                <Select
                  value={String(current.startMonth)}
                  onValueChange={(v) => set('startMonth', Number(v))}
                  options={monthNames.map((n, i) => ({ value: String(i + 1), label: n }))}
                />
              </Field>
              <Field
                label={t('educacao-nome-meta')}
                hint={t('educacao-nome-meta-dica')}
                className="sm:col-span-2"
              >
                <Input value={current.name} onChange={(e) => set('name', e.target.value)} />
              </Field>
            </>
          ) : null}
          {step === 1 ? (
            <>
              <div className="sm:col-span-2">
                <Segmented
                  label={t('educacao-tipo')}
                  value={current.institution}
                  onChange={(v: InstitutionType) => set('institution', v)}
                  options={[
                    { value: 'private', label: t('educacao-tipo-particular') },
                    { value: 'public', label: t('educacao-tipo-publica') },
                    { value: 'abroad', label: t('educacao-tipo-exterior') },
                  ]}
                  size="sm"
                />
              </div>
              <Field label={t('educacao-curso')} hint={t('educacao-curso-dica')}>
                <Input value={current.course} onChange={(e) => set('course', e.target.value)} />
              </Field>
              <Field
                label={t('educacao-duracao')}
                hint={t('educacao-duracao-dica')}
                error={errors.durationYears}
              >
                <NumberInput
                  value={current.durationYears}
                  onValue={(v) => set('durationYears', v ?? 0)}
                  suffix={t('educacao-anos')}
                  label={t('educacao-duracao')}
                  invalid={!!errors.durationYears}
                />
              </Field>
              <Field label={t('educacao-parcelas')}>
                <Select
                  value={String(current.paymentsPerYear)}
                  onValueChange={(v) => set('paymentsPerYear', Number(v))}
                  options={PAYMENTS_PER_YEAR.map((n) => ({
                    value: String(n),
                    label: t('educacao-parcelas-n', { n }),
                  }))}
                />
              </Field>
              <Field
                label={
                  current.paymentsPerYear === 12
                    ? t('educacao-mensalidade')
                    : t('educacao-valor-parcela')
                }
                hint={t('educacao-mensalidade-dica')}
                error={errors.fee}
              >
                <MoneyInput value={current.fee} onValueChange={(v) => set('fee', v)} />
              </Field>
              <Field label={t('educacao-extras')} hint={t('educacao-extras-dica')}>
                <MoneyInput
                  value={current.yearlyExtras}
                  onValueChange={(v) => set('yearlyExtras', v)}
                />
              </Field>
              <Field
                label={t('educacao-custo-vida')}
                hint={
                  current.institution === 'public'
                    ? t('educacao-custo-vida-publica')
                    : t('educacao-custo-vida-dica')
                }
              >
                <MoneyInput
                  value={current.monthlyLiving}
                  onValueChange={(v) => set('monthlyLiving', v)}
                />
              </Field>
              <Field
                label={t('educacao-desconto')}
                hint={t('educacao-desconto-dica')}
                error={errors.discountPct}
              >
                <NumberInput
                  value={current.discountPct}
                  onValue={(v) => set('discountPct', v ?? 0)}
                  suffix="%"
                  label={t('educacao-desconto')}
                  invalid={!!errors.discountPct}
                />
              </Field>
            </>
          ) : null}
          {step === 2 ? (
            <>
              <Field
                label={t('educacao-inflacao')}
                hint={
                  inflation.data && !inflation.data.missing
                    ? t('educacao-inflacao-dica', {
                        de: inflation.data.from ?? '',
                        ate: inflation.data.to ?? '',
                        anos: inflation.data.years,
                      })
                    : t('educacao-inflacao-sem-dados')
                }
                className="sm:col-span-2"
              >
                <Select
                  value={current.inflationMode}
                  onValueChange={(v) => set('inflationMode', v as InflationMode)}
                  options={inflationOptions}
                />
              </Field>
              {current.inflationMode === 'fixed_spread' ? (
                <Field
                  label={t('educacao-diferenca')}
                  hint={t('educacao-diferenca-dica')}
                  error={errors.fixedSpreadPct}
                >
                  <NumberInput
                    value={current.fixedSpreadPct}
                    onValue={(v) => set('fixedSpreadPct', v)}
                    suffix={t('educacao-pct-ano')}
                    label={t('educacao-diferenca')}
                    invalid={!!errors.fixedSpreadPct}
                  />
                </Field>
              ) : null}
              <Field
                label={t('educacao-retorno')}
                hint={t('educacao-retorno-dica')}
                error={errors.realReturnPct}
              >
                <NumberInput
                  value={current.realReturnPct}
                  onValue={(v) => set('realReturnPct', v ?? 0)}
                  suffix={t('educacao-pct-ano')}
                  label={t('educacao-retorno')}
                  invalid={!!errors.realReturnPct}
                />
              </Field>
              <div className="flex flex-col gap-1.5 sm:col-span-2">
                <span className="label-caps">{t('educacao-aporte-modo')}</span>
                <Segmented
                  label={t('educacao-aporte-modo')}
                  value={current.contributionMode}
                  onChange={(v) => set('contributionMode', v)}
                  options={[
                    { value: 'indexed', label: t('educacao-aporte-corrigido') },
                    { value: 'flat', label: t('educacao-aporte-fixo') },
                  ]}
                  size="sm"
                />
              </div>
              <Field label={t('metas-ja-guardado')} hint={t('educacao-ja-guardado-dica')}>
                <MoneyInput value={current.initial} onValueChange={(v) => set('initial', v)} />
              </Field>
              <p className="text-[12px] text-dim sm:col-span-2">{t('educacao-hipoteses')}</p>
            </>
          ) : null}
          <button type="submit" hidden />
        </form>
        <EducationResultPanel
          result={sim.data ?? null}
          error={sim.error ? errorText(sim.error) : !ready ? t('educacao-falta-dado') : null}
          loading={sim.isFetching}
        />
        {save.error ? (
          <p role="alert" className="text-[12.5px] text-negative">
            {errorText(save.error)}
          </p>
        ) : null}
      </div>
    </Dialog>
  );
}
