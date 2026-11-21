/**
 * Planejador de aposentadoria e independência financeira (`goals.retirement`): Monte Carlo em
 * valores reais com o patrimônio da carteira (inclui previdência), aportes, gasto desejado,
 * INSS e eventos de vida. Mostra a chance de o dinheiro durar até a expectativa de vida, o
 * maior gasto com 90% de sucesso, o patrimônio necessário ao parar e a faixa do patrimônio
 * por idade. O INSS pode ser informado ou calculado pelas regras da reforma. "Salvar plano"
 * guarda os parâmetros (o agente usa o plano salvo); cenários com nome ficam ao lado para
 * comparar, e uma meta pode acompanhar o plano. Com a linha da vida ligada
 * (docs/life-expectancy.md, seção 7), a barra do titular (ou do casal) fica em cima do
 * resultado, a idade de hoje vem da data de nascimento, o horizonte sugerido é a idade
 * "1 em 10" (editável: o valor digitado vence) e o resultado diz o risco de longevidade.
 */
import { StudyNotice } from '@/features/terms/Disclaimer';
import { useDeferredValue, useMemo, useState } from 'react';

import { call } from '@/api/client';
import type { Member } from '@/api/generated/Member';
import type { RetirementEvent } from '@/api/generated/RetirementEvent';
import type { RetirementPlan } from '@/api/generated/RetirementPlan';
import type { RetirementSimulation } from '@/api/generated/RetirementSimulation';
import { useCountryInfo } from '@/api/hooks/country';
import { useRetirement, useRetirementDefaults, useSaveRetirement } from '@/api/hooks/goals';
import { useMembers } from '@/api/hooks/household';
import { useFamilyOutlook, useLongevityEnabled } from '@/api/hooks/longevity';
import { useViewAs } from '@/app/viewAs';
import {
  Button,
  Field,
  Icon,
  Input,
  InstrumentCard,
  KeyValue,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { DecimalInput } from '@/features/portfolio/components/DecimalInput';
import { PlannerLifeBars } from '@/features/longevity/MemberLifeBar';
import { ErrorBlock, LoadingBlock } from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatMoney } from '@/lib/format';
import { FormError } from '@/vault/FormError';

import { inssDefaults, RetirementInss } from './RetirementInss';
import {
  ptPensionDefaults,
  RetirementPtPension,
  RetirementUsSsa,
  usSsaDefaults,
} from './RetirementPension';
import { RetirementGoalLink, RetirementScenarios } from './RetirementScenarios';

const pctText = (v: number | undefined) => (v == null ? null : String(Math.round(v * 1000) / 10));
const fromPct = (s: string | null) => {
  const n = s == null ? NaN : Number(s);
  return Number.isFinite(n) ? n / 100 : undefined;
};
const age = (s: string) => {
  const n = Number(s);
  return s.trim() && Number.isInteger(n) ? n : undefined;
};

/** Sem cálculo pelas regras: só o valor informado. */
const NO_CALC: Partial<RetirementPlan> = {
  inss_calc: undefined,
  us_ssa_calc: undefined,
  pt_pension_calc: undefined,
};

/** Liga o cálculo do país (os dados já digitados ficam). */
function calcFor(
  rules: string | null,
  d: RetirementPlan,
  member: Member | null | undefined,
): Partial<RetirementPlan> {
  switch (rules) {
    case 'us_ssa':
      return { ...NO_CALC, us_ssa_calc: d.us_ssa_calc ?? usSsaDefaults(member) };
    case 'pt_ss':
      return { ...NO_CALC, pt_pension_calc: d.pt_pension_calc ?? ptPensionDefaults(member) };
    default:
      return { ...NO_CALC, inss_calc: d.inss_calc ?? inssDefaults(member) };
  }
}

/** O plano calcula pelas regras do país da família. */
function calcOn(rules: string | null, d: RetirementPlan): boolean {
  if (rules === 'br_inss') return d.inss_calc != null;
  if (rules === 'us_ssa') return d.us_ssa_calc != null;
  if (rules === 'pt_ss') return d.pt_pension_calc != null;
  return false;
}

export function RetirementPlanner() {
  // A aposentadoria pública do país (padrão: o Brasil, enquanto carrega).
  const pension = useCountryInfo()?.pension ?? { name: 'INSS', rules: 'br_inss' };
  const [draft, setDraft] = useState<RetirementPlan | null>(null);
  const [scenario, setScenario] = useState<string | null>(null);
  // Linha da vida: padrões pelo membro (o de "Ver como" ou o titular).
  const { memberId, member } = useViewAs();
  const members = useMembers().data ?? [];
  const planMember = member ?? members.find((m) => m.is_primary) ?? null;
  const defaults = useRetirementDefaults(memberId).data;
  const longevityOn = useLongevityEnabled();
  const family = useFamilyOutlook(longevityOn).data;
  const couple = !memberId && family?.any_adult_alive != null;
  const suggestion = defaults?.from_longevity ? defaults.life_expectancy : null;
  // Sem horizonte no formulário, vale a sugestão "1 em 10"; o valor digitado sempre vence.
  // (Antes de o formulário existir, a consulta é a do plano salvo, sem nada por cima.)
  const simulated = useMemo<RetirementPlan>(
    () =>
      draft != null && draft.life_expectancy == null && suggestion != null
        ? { ...draft, life_expectancy: suggestion }
        : (draft ?? {}),
    [draft, suggestion],
  );
  const plan = useDeferredValue(simulated);
  const q = useRetirement(plan);
  const save = useSaveRetirement();
  // Primeira resposta: o formulário começa com o plano salvo (e os padrões do núcleo).
  if (draft == null && q.data) setDraft(q.data.plan);
  // Sem plano salvo, a idade de hoje vem da data de nascimento (continua editável).
  if (
    draft == null &&
    !q.data &&
    q.isError &&
    defaults?.from_longevity &&
    defaults.current_age != null
  )
    setDraft({ current_age: defaults.current_age });
  const set = (p: Partial<RetirementPlan>) => setDraft((d) => ({ ...(d ?? {}), ...p }));
  const d = draft ?? {};
  const calc = calcOn(pension.rules, d);
  const events = d.events ?? [];
  const setEvent = (i: number, e: Partial<RetirementEvent>) =>
    set({ events: events.map((x, j) => (j === i ? { ...x, ...e } : x)) });

  const r = q.data;
  const needsAge = q.isError && d.current_age == null;
  return (
    <InstrumentCard index="05.6" title={t('metas-apos-titulo')}>
      <div className="flex flex-col gap-4">
        <RetirementScenarios
          plan={simulated}
          sim={r}
          selected={scenario}
          onSelect={(id, p) => {
            setScenario(id);
            if (p) setDraft(p);
            // Plano principal: o salvo, direto do núcleo (a consulta da tela ainda é a do
            // cenário).
            else
              void call<RetirementSimulation>('goals.retirement', {}).then(
                (res) => setDraft(res.plan),
                () => setDraft({}),
              );
          }}
        />
        <p className="max-w-[80ch] text-[13px] text-dim">
          {t('metas-apos-intro', { sistema: pension.name })}
        </p>
        <div className="grid gap-3 sm:grid-cols-4">
          <Field label={t('metas-apos-idade-hoje')}>
            <Input
              inputMode="numeric"
              value={d.current_age ?? ''}
              onChange={(e) => set({ current_age: age(e.target.value) })}
            />
          </Field>
          <Field label={t('metas-apos-aposentadoria-aos')}>
            <Input
              inputMode="numeric"
              placeholder={String(r?.plan.retirement_age ?? 65)}
              value={d.retirement_age ?? ''}
              onChange={(e) => set({ retirement_age: age(e.target.value) })}
            />
          </Field>
          <Field
            label={t('metas-apos-expectativa')}
            hint={
              suggestion != null ? (
                <>
                  {couple
                    ? t('longevidade-apos-sugestao-casal', { idade: suggestion })
                    : t('longevidade-apos-sugestao', { idade: suggestion })}
                  {d.life_expectancy != null && d.life_expectancy !== suggestion ? (
                    <button
                      type="button"
                      className="ml-1 text-accent hover:underline"
                      onClick={() => set({ life_expectancy: suggestion })}
                    >
                      {t('longevidade-apos-usar', { idade: suggestion })}
                    </button>
                  ) : null}
                </>
              ) : undefined
            }
          >
            <Input
              inputMode="numeric"
              placeholder={String(r?.plan.life_expectancy ?? suggestion ?? 95)}
              value={d.life_expectancy ?? ''}
              onChange={(e) => set({ life_expectancy: age(e.target.value) })}
            />
          </Field>
          <Field
            label={t('metas-apos-patrimonio-hoje')}
            hint={
              d.current_wealth == null && r
                ? t('metas-apos-patrimonio-dica-valor', {
                    valor: formatMoney(r.start_wealth),
                  })
                : t('metas-apos-patrimonio-dica')
            }
          >
            <MoneyInput
              value={d.current_wealth ?? null}
              onValueChange={(v) => set({ current_wealth: v ?? undefined })}
            />
          </Field>
          <Field label={t('metas-apos-aporte')}>
            <MoneyInput
              value={d.monthly_contribution ?? null}
              onValueChange={(v) => set({ monthly_contribution: v ?? undefined })}
            />
          </Field>
          <Field label={t('metas-apos-gasto')}>
            <MoneyInput
              value={d.monthly_spending ?? null}
              onValueChange={(v) => set({ monthly_spending: v ?? undefined })}
            />
          </Field>
          <Field label={pension.name}>
            <Select
              aria-label={pension.name}
              value={calc ? 'calc' : 'manual'}
              disabled={!pension.rules}
              onValueChange={(v) =>
                set(
                  v === 'calc'
                    ? { ...calcFor(pension.rules, d, planMember), inss_monthly: undefined }
                    : NO_CALC,
                )
              }
              options={[
                { value: 'manual', label: t('metas-apos-inss-informar') },
                ...(pension.rules ? [{ value: 'calc', label: t('metas-apos-inss-calcular') }] : []),
              ]}
            />
          </Field>
          {calc ? null : (
            <Field label={t('metas-apos-inss-mensal', { sistema: pension.name })}>
              <MoneyInput
                value={d.inss_monthly ?? null}
                onValueChange={(v) => set({ inss_monthly: v ?? undefined })}
              />
            </Field>
          )}
          <Field label={t('metas-apos-inss-a-partir', { sistema: pension.name })}>
            <Input
              inputMode="numeric"
              placeholder={String(r?.plan.inss_age ?? d.retirement_age ?? 65)}
              value={d.inss_age ?? ''}
              onChange={(e) => set({ inss_age: age(e.target.value) })}
            />
          </Field>
          <Field label={t('metas-apos-retorno')}>
            <DecimalInput
              value={pctText(d.real_return) ?? '4'}
              onValueChange={(v) => set({ real_return: fromPct(v) })}
            />
          </Field>
          <Field label={t('metas-apos-volatilidade')}>
            <DecimalInput
              value={pctText(d.volatility) ?? '10'}
              onValueChange={(v) => set({ volatility: fromPct(v) })}
            />
          </Field>
        </div>
        {d.inss_calc && pension.rules === 'br_inss' ? (
          <RetirementInss
            value={d.inss_calc}
            onChange={(v) => set({ inss_calc: v })}
            estimate={r?.inss}
            retirementAge={d.retirement_age}
          />
        ) : null}
        {d.us_ssa_calc && pension.rules === 'us_ssa' ? (
          <RetirementUsSsa
            value={d.us_ssa_calc}
            onChange={(v) => set({ us_ssa_calc: v })}
            estimate={r?.inss}
            retirementAge={d.retirement_age}
          />
        ) : null}
        {d.pt_pension_calc && pension.rules === 'pt_ss' ? (
          <RetirementPtPension
            value={d.pt_pension_calc}
            onChange={(v) => set({ pt_pension_calc: v })}
            estimate={r?.inss}
            retirementAge={d.retirement_age}
          />
        ) : null}
        <div className="flex flex-col gap-2">
          <span className="label-caps text-dim">{t('metas-apos-eventos')}</span>
          {events.map((e, i) => (
            <div key={i} className="grid grid-cols-[90px_180px_1fr_28px] items-end gap-2">
              <Field label={t('metas-apos-idade')}>
                <Input
                  inputMode="numeric"
                  value={e.age}
                  onChange={(ev) => setEvent(i, { age: age(ev.target.value) ?? 0 })}
                />
              </Field>
              <Field label={t('metas-apos-evento-valor')}>
                <MoneyInput
                  allowNegative
                  value={e.amount}
                  onValueChange={(v) => setEvent(i, { amount: v ?? 0 })}
                />
              </Field>
              <Field label={t('metas-apos-evento-o-que')}>
                <Input value={e.label} onChange={(ev) => setEvent(i, { label: ev.target.value })} />
              </Field>
              <button
                type="button"
                aria-label={t('metas-apos-remover-evento', { n: i + 1 })}
                className="mb-2 text-dim hover:text-fg"
                onClick={() => set({ events: events.filter((_, j) => j !== i) })}
              >
                <Icon name="x" size={13} />
              </button>
            </div>
          ))}
          <div>
            <Button
              size="sm"
              variant="ghost"
              onClick={() =>
                set({
                  events: [...events, { age: (d.retirement_age ?? 65) + 5, amount: 0, label: '' }],
                })
              }
            >
              {t('metas-apos-adicionar-evento')}
            </Button>
          </div>
        </div>
        <PlannerLifeBars memberId={memberId} sim={r} />
        {needsAge ? (
          <p className="text-[13px] text-dim">{t('metas-apos-informe-idade')}</p>
        ) : q.isPending && !r ? (
          <LoadingBlock label={t('metas-apos-simulando')} />
        ) : q.isError && !r ? (
          <ErrorBlock error={q.error} />
        ) : r ? (
          <div className="flex flex-col gap-3">
            <KeyValue
              items={[
                {
                  label: t('metas-apos-chance'),
                  value: (
                    <span
                      className={
                        r.success >= 0.9
                          ? 'text-positive'
                          : r.success >= 0.7
                            ? 'text-warning'
                            : 'text-negative'
                      }
                    >
                      {Math.round(r.success * 100)}%
                    </span>
                  ),
                },
                {
                  label: t('metas-apos-patrimonio-mediana'),
                  value: <Money cents={r.wealth_at_retirement} />,
                },
                {
                  label: t('metas-apos-gasto-90'),
                  value: <Money cents={r.sustainable_spending} />,
                },
                {
                  label: t('metas-apos-necessario'),
                  value: <Money cents={r.required_wealth} />,
                },
                {
                  label: t('metas-apos-acaba-aos'),
                  value: r.depletion_age_p10
                    ? t('metas-apos-n-anos', { n: r.depletion_age_p10 })
                    : t('metas-apos-nao-acaba'),
                },
              ]}
            />
            {r.longevity ? (
              <p className="max-w-[80ch] text-[13px] text-fg">
                {couple
                  ? t('longevidade-apos-risco-casal', {
                      idade: r.longevity.plan_age,
                      n: Math.round(r.longevity.survival * 100),
                    })
                  : t('longevidade-apos-risco', {
                      idade: r.longevity.plan_age,
                      n: Math.round(r.longevity.survival * 100),
                    })}
              </p>
            ) : null}
            <table
              className="w-full border-collapse text-[12.5px]"
              aria-label={t('metas-apos-patrimonio-idade')}
            >
              <thead>
                <tr className="label-caps text-left text-dim">
                  <th className="pb-1 font-normal">{t('metas-apos-idade')}</th>
                  <th className="pb-1 text-right font-normal">{t('metas-apos-ruim')}</th>
                  <th className="pb-1 text-right font-normal">{t('metas-apos-mediana')}</th>
                  <th className="pb-1 text-right font-normal">{t('metas-apos-bom')}</th>
                </tr>
              </thead>
              <tbody>
                {r.bands
                  .filter((_, i) => i % 5 === 0 || i === r.bands.length - 1)
                  .map((b) => (
                    <tr key={b.age} className="border-t border-line">
                      <td className="py-1">{b.age}</td>
                      <td className="py-1 text-right">
                        <Money cents={b.p10} />
                      </td>
                      <td className="py-1 text-right">
                        <Money cents={b.p50} />
                      </td>
                      <td className="py-1 text-right">
                        <Money cents={b.p90} />
                      </td>
                    </tr>
                  ))}
              </tbody>
            </table>
            <ul className="text-[12px] text-dim">
              {r.notes.map((n) => (
                <li key={n}>{n}</li>
              ))}
            </ul>
          </div>
        ) : null}
        <FormError error={save.error} />
        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-line pt-3">
          <RetirementGoalLink saved={r != null} />
          <Button
            variant="primary"
            disabled={d.current_age == null}
            loading={save.isPending}
            onClick={() =>
              save.mutate(simulated, {
                onSuccess: () => toast({ tone: 'success', title: t('metas-apos-salvo') }),
              })
            }
          >
            {scenario ? t('metas-apos-usar-principal') : t('metas-apos-salvar')}
          </Button>
        </div>
        <StudyNotice />
      </div>
    </InstrumentCard>
  );
}
