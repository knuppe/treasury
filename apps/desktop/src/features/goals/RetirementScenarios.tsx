/**
 * Cenários de aposentadoria (`goals.retirement_scenarios`): carregar um cenário no
 * formulário, salvar o que está na tela como cenário novo ou por cima do escolhido, apagar e
 * comparar lado a lado. E a meta de aposentadoria ligada ao plano principal.
 */
import { useState } from 'react';

import type { RetirementPlan } from '@/api/generated/RetirementPlan';
import type { RetirementSimulation } from '@/api/generated/RetirementSimulation';
import {
  useDeleteRetirementScenario,
  useGoals,
  useLinkRetirementGoal,
  useRetirementScenarios,
  useSaveRetirementScenario,
  useUnlinkRetirementGoal,
} from '@/api/hooks/goals';
import { Button, Field, Input, Money, Select, toast } from '@/design/components';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { FormError } from '@/vault/FormError';

const MAIN = '__principal__';

const pct = (v: number | null | undefined) => (v == null ? '—' : `${Math.round(v * 100)}%`);

export function RetirementScenarios({
  plan,
  sim,
  selected,
  onSelect,
}: {
  /** O plano que está no formulário. */
  plan: RetirementPlan;
  sim: RetirementSimulation | undefined;
  /** Cenário carregado (`null` = plano principal). */
  selected: string | null;
  /** Carrega um cenário (`null` = volta ao plano principal). */
  onSelect: (id: string | null, plan: RetirementPlan | null) => void;
}) {
  const q = useRetirementScenarios();
  const save = useSaveRetirementScenario();
  const remove = useDeleteRetirementScenario();
  const list = q.data ?? [];
  const current = list.find((s) => s.id === selected);
  const [name, setName] = useState('');
  const saveNew = () =>
    save.mutate(
      { name, plan },
      {
        onSuccess: (s) => {
          toast({ tone: 'success', title: t('metas-cen-salvo', { nome: s.name }) });
          setName('');
          onSelect(s.id, s.plan);
        },
      },
    );
  return (
    <div className="flex flex-col gap-3 border-b border-line pb-4">
      <div className="flex flex-wrap items-end gap-3">
        <Field label={t('metas-cen-cenario')}>
          <Select
            aria-label={t('metas-cen-cenario')}
            value={selected ?? MAIN}
            onValueChange={(v) => {
              if (v === MAIN) onSelect(null, null);
              else onSelect(v, list.find((s) => s.id === v)?.plan ?? null);
            }}
            options={[
              { value: MAIN, label: t('metas-cen-principal') },
              ...list.map((s) => ({ value: s.id, label: s.name })),
            ]}
          />
        </Field>
        {current ? (
          <>
            <Button
              size="sm"
              loading={save.isPending}
              onClick={() =>
                save.mutate(
                  { id: current.id, name: current.name, plan },
                  { onSuccess: () => toast({ tone: 'success', title: t('metas-cen-atualizado') }) },
                )
              }
            >
              {t('metas-cen-atualizar', { nome: current.name })}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              loading={remove.isPending}
              onClick={() =>
                remove.mutate(current.id, {
                  onSuccess: () => {
                    toast({ tone: 'success', title: t('metas-cen-apagado') });
                    onSelect(null, null);
                  },
                })
              }
            >
              {t('metas-cen-apagar')}
            </Button>
          </>
        ) : null}
        <form
          className="ml-auto flex items-end gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (name.trim()) saveNew();
          }}
        >
          <Field label={t('metas-cen-novo')}>
            <Input
              placeholder={t('metas-cen-exemplo')}
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </Field>
          <Button size="sm" type="submit" disabled={!name.trim()} loading={save.isPending}>
            {t('metas-cen-salvar')}
          </Button>
        </form>
      </div>
      <FormError error={save.error ?? remove.error} />
      {list.length > 0 ? (
        <table
          className="w-full border-collapse text-[12.5px]"
          aria-label={t('metas-cen-comparar')}
        >
          <thead>
            <tr className="label-caps text-left text-dim">
              <th className="pb-1 font-normal">{t('metas-cen-cenario')}</th>
              <th className="pb-1 text-right font-normal">{t('metas-cen-chance')}</th>
              <th className="pb-1 text-right font-normal">{t('metas-cen-gasto-90')}</th>
              <th className="pb-1 text-right font-normal">{t('metas-cen-necessario')}</th>
              <th className="pb-1 text-right font-normal">{t('metas-cen-previsto')}</th>
            </tr>
          </thead>
          <tbody>
            {sim ? (
              <tr className="border-t border-line text-accent">
                <td className="py-1">{t('metas-cen-na-tela')}</td>
                <td className="py-1 text-right">{pct(sim.success)}</td>
                <td className="py-1 text-right">
                  <Money cents={sim.sustainable_spending} />
                </td>
                <td className="py-1 text-right">
                  <Money cents={sim.required_wealth} />
                </td>
                <td className="py-1 text-right">
                  <Money cents={sim.wealth_at_retirement} />
                </td>
              </tr>
            ) : null}
            {list.map((s) => (
              <tr key={s.id} className="border-t border-line">
                <td className="py-1">
                  <button
                    type="button"
                    className="text-left hover:underline"
                    onClick={() => onSelect(s.id, s.plan)}
                  >
                    {s.name}
                  </button>
                </td>
                {s.error ? (
                  <td colSpan={4} className="py-1 text-right text-dim">
                    {s.error}
                  </td>
                ) : (
                  <>
                    <td className="py-1 text-right">{pct(s.success)}</td>
                    <td className="py-1 text-right">
                      {s.sustainable_spending != null ? (
                        <Money cents={s.sustainable_spending} />
                      ) : (
                        '—'
                      )}
                    </td>
                    <td className="py-1 text-right">
                      {s.required_wealth != null ? <Money cents={s.required_wealth} /> : '—'}
                    </td>
                    <td className="py-1 text-right">
                      {s.wealth_at_retirement != null ? (
                        <Money cents={s.wealth_at_retirement} />
                      ) : (
                        '—'
                      )}
                    </td>
                  </>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      ) : null}
    </div>
  );
}

/** A meta de aposentadoria ligada ao plano principal (criar, ver o progresso, desligar). */
export function RetirementGoalLink({ saved }: { saved: boolean }) {
  const goals = useGoals();
  const link = useLinkRetirementGoal();
  const unlink = useUnlinkRetirementGoal();
  const goal = (goals.data ?? []).find((g) => g.linked_to_plan);
  return (
    <div className="flex flex-wrap items-center gap-3 text-[13px]">
      {goal ? (
        <>
          <span>
            {withSlots(
              goal.target_date
                ? t('metas-ligada-ate', {
                    nome: slot(0),
                    pct: Math.round(goal.progress * 100),
                    valor: slot(1),
                    ano: goal.target_date.slice(0, 4),
                  })
                : t('metas-ligada', {
                    nome: slot(0),
                    pct: Math.round(goal.progress * 100),
                    valor: slot(1),
                  }),
              [
                <span key="nome" className="font-medium">
                  {goal.name}
                </span>,
                <Money key="alvo" cents={goal.target_amount} />,
              ],
            )}
          </span>
          <Button
            size="sm"
            variant="ghost"
            loading={unlink.isPending}
            onClick={() => unlink.mutate()}
          >
            {t('metas-ligada-desligar')}
          </Button>
        </>
      ) : (
        <>
          <span className="text-dim">{t('metas-ligada-explica')}</span>
          <Button
            size="sm"
            variant="ghost"
            disabled={!saved}
            loading={link.isPending}
            onClick={() =>
              link.mutate(undefined, {
                onSuccess: (g) =>
                  toast({
                    tone: 'success',
                    title: t('metas-ligada-toast'),
                    description: t('metas-ligada-toast-descricao', { nome: g.name }),
                  }),
              })
            }
          >
            {t('metas-ligada-criar')}
          </Button>
        </>
      )}
      <FormError error={link.error ?? unlink.error} />
    </div>
  );
}
