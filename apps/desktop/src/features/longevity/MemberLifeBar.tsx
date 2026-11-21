/**
 * A linha da vida de um membro no cadastro dele e no planejador de aposentadoria (docs/
 * life-expectancy.md, seção 6.3): só com a função ligada e a barra do membro não escondida.
 * No planejador, a barra do titular (ou do casal) leva os marcos do plano em edição.
 */
import type { LifeMilestone } from '@/api/generated/LifeMilestone';
import type { MemberOutlook } from '@/api/generated/MemberOutlook';
import type { RetirementSimulation } from '@/api/generated/RetirementSimulation';
import { useMembers } from '@/api/hooks/household';
import {
  useFamilyOutlook,
  useLifeStages,
  useLongevitySettings,
  useMemberOutlook,
} from '@/api/hooks/longevity';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';

import { LifeBar, LifeSourceLine } from './LifeBar';
import { ageAxis } from './model';
import { useLifeView } from './useLifeView';

/** A barra de um membro (nada com a função desligada, a barra escondida ou sem nascimento). */
export function MemberLifeBar({ memberId }: { memberId: string }) {
  const settings = useLongevitySettings().data;
  const on = settings?.enabled === true && !settings.hidden_members.includes(memberId);
  const q = useMemberOutlook(memberId, on);
  const family = useFamilyOutlook(on);
  const stages = useLifeStages().data;
  const view = useLifeView();
  if (!on || !q.data) return null;
  const country = family.data?.country ?? 'BR';
  return (
    <LifeBar
      outlook={q.data}
      tone={view.tone}
      place={view.place(country)}
      stages={stages}
      advanced={settings?.show_advanced_later_life}
      source={family.data?.source}
    />
  );
}

/** Os marcos do plano em edição (aposentadoria, INSS, até onde vai e os eventos). */
function planMilestones(o: MemberOutlook, sim: RetirementSimulation): LifeMilestone[] {
  const p = sim.plan;
  const thisYear = Number(todayIso().slice(0, 4));
  const year = (age: number) => thisYear + (age - o.age);
  const out: LifeMilestone[] = o.milestones.filter(
    (m) =>
      m.kind !== 'retirement' &&
      m.kind !== 'inss' &&
      m.kind !== 'plan_end' &&
      m.kind !== 'life_event',
  );
  if (p.retirement_age != null)
    out.push({
      kind: 'retirement',
      age: p.retirement_age,
      year: year(p.retirement_age),
      label: t('longevidade-marco-aposentadoria'),
      amount: null,
    });
  if (p.inss_age != null && (p.inss_monthly || p.inss_calc || p.us_ssa_calc || p.pt_pension_calc))
    out.push({
      kind: 'inss',
      age: p.inss_age,
      year: year(p.inss_age),
      label: t('longevidade-marco-inss'),
      amount: p.inss_monthly ?? sim.inss?.monthly ?? null,
    });
  const end = sim.depletion_age_p10 ?? p.life_expectancy;
  if (end != null)
    out.push({
      kind: 'plan_end',
      age: end,
      year: year(end),
      label: t('longevidade-marco-plano'),
      amount: null,
    });
  for (const e of p.events ?? [])
    out.push({
      kind: 'life_event',
      age: e.age,
      year: year(e.age),
      label: e.label,
      amount: e.amount,
    });
  return out;
}

/**
 * Planejador de aposentadoria: a barra de quem o plano cobre (o membro em "Ver como" ou o
 * titular e o cônjuge), em cima do resultado, com os marcos do plano em edição na barra de
 * quem o plano é.
 */
export function PlannerLifeBars({
  memberId,
  sim,
}: {
  /** O membro em "Ver como" (`null`: a família, com o titular e o cônjuge). */
  memberId: string | null;
  sim: RetirementSimulation | undefined;
}) {
  const settings = useLongevitySettings().data;
  const on = settings?.enabled === true;
  const family = useFamilyOutlook(on);
  const stages = useLifeStages().data;
  const members = useMembers().data ?? [];
  const view = useLifeView();
  const f = family.data;
  if (!on || !f) return null;
  const primary = members.find((m) => m.is_primary);
  const spouse = members.find((m) => !m.is_primary && m.profile.relationship === 'conjuge');
  const planMember = memberId ?? primary?.id ?? null;
  const ids = memberId ? [memberId] : [primary?.id, spouse?.id].filter(Boolean);
  const hidden = new Set(settings.hidden_members);
  const list = f.members.filter((o) => ids.includes(o.member_id) && !hidden.has(o.member_id));
  if (!list.length) return null;
  const axis = ageAxis(list);
  return (
    <section aria-label={t('longevidade-titulo')} className="flex flex-col gap-3">
      <span className="label-caps text-dim">{t('longevidade-titulo')}</span>
      {list.map((o, i) => (
        <LifeBar
          key={o.member_id}
          outlook={o}
          tone={view.tone}
          place={view.place(f.country)}
          stages={stages}
          axis={axis}
          advanced={settings.show_advanced_later_life}
          milestones={sim && o.member_id === planMember ? planMilestones(o, sim) : undefined}
          source={f.source}
          showSource={false}
          showPhaseNames={i === 0}
        />
      ))}
      <LifeSourceLine source={f.source} />
    </section>
  );
}
