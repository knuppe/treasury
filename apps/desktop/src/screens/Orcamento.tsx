import { useSearchParams } from 'react-router';

import type { Basis } from '@/api/generated/Basis';
import { useBudget } from '@/api/hooks/budget';
import { useSupportEventWhen } from '@/api/hooks/support';
import { screenMeta } from '@/app/screens';
import { ScreenHeader, Tabs, TabsContent, TabsList, TabsTrigger } from '@/design/components';
import { BudgetPanel } from '@/features/budget/BudgetPanel';
import { useMonthGroups } from '@/features/budget/SpendGroupsCard';
import { money, usePageContext } from '@/features/companion/pageContext';
import { EmergencyReserveCard } from '@/features/goals/EmergencyReserveCard';
import { RetirementPlanner } from '@/features/goals/RetirementPlanner';
import { GoalsPanel } from '@/features/goals/GoalsPanel';
import { ScenariosPanel } from '@/features/planning/ScenariosPanel';
import { InstallmentsPanel } from '@/features/planning/InstallmentsPanel';
import { SubscriptionsPanel } from '@/features/planning/SubscriptionsPanel';
import { ProjectsPanel } from '@/features/projects/ProjectsPanel';
import { BasisToggle, MonthPicker, QueryView } from '@/features/reports/controls';
import { ExplainDialog } from '@/features/reports/explain/ExplainDialog';
import { useCategoryColors } from '@/features/reports/useEntityColors';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';
import { monthOf, shiftMonth } from '@/lib/months';

const meta = screenMeta('orcamento');

type Tab =
  | 'orcamento'
  | 'metas'
  | 'projetos'
  | 'reserva'
  | 'aposentadoria'
  | 'assinaturas'
  | 'parcelas'
  | 'cenarios';
const TABS: readonly Tab[] = [
  'orcamento',
  'metas',
  'projetos',
  'reserva',
  'aposentadoria',
  'assinaturas',
  'parcelas',
  'cenarios',
];

export default function Orcamento() {
  const [params, setParams] = useSearchParams();
  const current = monthOf(todayIso());
  const m = params.get('mes');
  const month = m && /^\d{4}-\d{2}$/.test(m) ? m : current;
  const b = params.get('base');
  const chosenBasis: Basis | undefined = b === 'cash' || b === 'accrual' ? b : undefined;
  const aba = params.get('aba') as Tab | null;
  const tab: Tab = aba && TABS.includes(aba) ? aba : 'orcamento';
  const budget = useBudget({ month, basis: chosenBasis });
  const basis: Basis = chosenBasis ?? budget.data?.basis ?? 'accrual';
  // Um mês fechado revisto no orçamento é um dos momentos do lembrete de apoio.
  useSupportEventWhen(
    tab === 'orcamento' && month < current && Boolean(budget.data),
    'budget_month',
    month,
  );
  const colors = useCategoryColors();

  const update = (patch: Record<string, string | null>) => {
    const p = new URLSearchParams(params);
    for (const [k, v] of Object.entries(patch)) {
      if (v == null) p.delete(k);
      else p.set(k, v);
    }
    setParams(p, { replace: true });
  };

  // Contexto para o agente: mês, totais, as categorias mais perto (ou acima) do previsto e
  // para onde vai a renda (os quatro grupos de gasto com a meta de cada um).
  const shown = budget.data;
  const groups = useMonthGroups(month, basis, Boolean(shown) && tab === 'orcamento').data;
  const groupMonth = groups?.months.find((g) => g.month === month);
  usePageContext({
    screen: 'orcamento',
    title: `${meta.index} · ${meta.title}`,
    view: tab,
    period: { month, basis },
    figures: shown
      ? {
          planned_expense: money(shown.totals.planned_expense, shown.currency),
          actual_expense: money(shown.totals.actual_expense, shown.currency),
          planned_income: money(shown.totals.planned_income, shown.currency),
          actual_income: money(shown.totals.actual_income, shown.currency),
          month_elapsed: Math.round(shown.month_elapsed * 100) / 100,
        }
      : undefined,
    items: shown?.lines
      .filter((l) => l.kind === 'expense' && (l.planned > 0 || l.actual > 0))
      .sort((x, y) => (y.ratio ?? 0) - (x.ratio ?? 0))
      .slice(0, 12)
      .map((l) => ({
        category_id: l.category_id,
        category: l.category_path,
        planned: money(l.planned, shown.currency),
        actual: money(l.actual, shown.currency),
        ratio: l.ratio == null ? null : Math.round(l.ratio * 100) / 100,
      })),
    groups:
      groups && groupMonth
        ? [
            { group: 'income', amount: money(groupMonth.income, groups.currency) },
            ...groupMonth.groups.map((g) => ({
              group: g.group,
              amount: money(g.amount, groups.currency),
              share_of_income:
                g.share_of_income == null ? null : Math.round(g.share_of_income * 1000) / 1000,
              target: groups.targets[g.group],
              top: g.top.map((c) => ({
                category_id: c.category_id,
                category: c.label,
                amount: money(c.amount, groups.currency),
              })),
            })),
          ]
        : undefined,
  });

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={meta.summary}
        actions={
          tab === 'orcamento' ? (
            <>
              <MonthPicker
                month={month}
                max={shiftMonth(current, 12)}
                onChange={(next) => update({ mes: next === current ? null : next })}
              />
              <BasisToggle value={basis} onChange={(next) => update({ base: next })} />
            </>
          ) : undefined
        }
      />
      <Tabs value={tab} onValueChange={(v) => update({ aba: v === 'orcamento' ? null : v })}>
        <TabsList aria-label={t('orcamento-abas')}>
          <TabsTrigger value="orcamento">{t('orcamento-aba-previsto')}</TabsTrigger>
          <TabsTrigger value="metas">{t('orcamento-aba-metas')}</TabsTrigger>
          <TabsTrigger value="projetos">{t('orcamento-aba-projetos')}</TabsTrigger>
          <TabsTrigger value="reserva">{t('orcamento-aba-reserva')}</TabsTrigger>
          <TabsTrigger value="aposentadoria">{t('orcamento-aba-aposentadoria')}</TabsTrigger>
          <TabsTrigger value="assinaturas">{t('orcamento-aba-assinaturas')}</TabsTrigger>
          <TabsTrigger value="parcelas">{t('orcamento-aba-parcelas')}</TabsTrigger>
          <TabsTrigger value="cenarios">{t('orcamento-aba-cenarios')}</TabsTrigger>
        </TabsList>
        <TabsContent value="orcamento">
          <QueryView query={budget} loadingLabel={t('orcamento-carregando')}>
            {(status) => (
              <BudgetPanel key={status.month} status={status} basis={basis} colors={colors} />
            )}
          </QueryView>
        </TabsContent>
        <TabsContent value="metas">
          <GoalsPanel />
        </TabsContent>
        <TabsContent value="projetos">
          <ProjectsPanel />
        </TabsContent>
        <TabsContent value="reserva">
          <EmergencyReserveCard />
        </TabsContent>
        <TabsContent value="aposentadoria">
          <RetirementPlanner />
        </TabsContent>
        <TabsContent value="assinaturas">
          <SubscriptionsPanel />
        </TabsContent>
        <TabsContent value="parcelas">
          <InstallmentsPanel />
        </TabsContent>
        <TabsContent value="cenarios">
          <ScenariosPanel />
        </TabsContent>
      </Tabs>
      <ExplainDialog />
    </div>
  );
}
