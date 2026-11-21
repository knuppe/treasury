import { useMemo } from 'react';
import { useSearchParams } from 'react-router';

import type { Basis } from '@/api/generated/Basis';
import { useBudget } from '@/api/hooks/budget';
import {
  useDashboard,
  useMonthFlow,
  useNetWorthHistory,
  usePortfolioSummary,
  useSpending,
} from '@/api/hooks/reports';
import { useLevel } from '@/app/profile';
import { useViewAs } from '@/app/viewAs';
import { screenMeta } from '@/app/screens';
import { Button, ScreenHeader } from '@/design/components';
import {
  BudgetCard,
  FlowCard,
  NetWorthCard,
  PortfolioCard,
  TopCategoriesCard,
  UncategorizedCard,
  UpcomingCard,
} from '@/features/dashboard/cards';
import { AttentionCard } from '@/features/attention/AttentionCard';
import { CoverageLine } from '@/features/attention/CoverageLine';
import { MonthCloseCard } from '@/features/attention/MonthCloseCard';
import { NextStepCard } from '@/features/attention/NextStepCard';
import { money, usePageContext } from '@/features/companion/pageContext';
import { InsightsCard } from '@/features/dashboard/InsightsCard';
import { SupportCard } from '@/features/support/SupportCard';
import { composeDashboard } from '@/features/dashboard/model';
import { HealthCard } from '@/features/health/HealthCard';
import { DailyAllowanceCard } from '@/features/planning/DailyAllowanceCard';
import { MembersCard } from '@/features/profile/MembersCard';
import { MonthSummary, SimpleDashboard } from '@/features/profile/SimpleDashboard';
import { BasisToggle, MonthPicker, QueryView } from '@/features/reports/controls';
import { ExplainDialog } from '@/features/reports/explain/ExplainDialog';
import { useCategoryColors } from '@/features/reports/useEntityColors';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { monthBounds, monthOf, shiftMonth } from '@/lib/months';
import { useUi } from '@/state/ui';

const meta = screenMeta('painel');

/** `?mes=2026-09&base=cash` → mês e base escolhidos (o padrão é o mês atual e a base do cofre). */
function usePanelParams() {
  const [params, setParams] = useSearchParams();
  const today = todayIso();
  const current = monthOf(today);
  const m = params.get('mes');
  const month = m && /^\d{4}-\d{2}$/.test(m) && m <= current ? m : current;
  const b = params.get('base');
  const basis: Basis | null = b === 'cash' || b === 'accrual' ? b : null;
  const update = (next: { month?: string; basis?: Basis }) => {
    const p = new URLSearchParams(params);
    if (next.month) {
      if (next.month === current) p.delete('mes');
      else p.set('mes', next.month);
    }
    if (next.basis) p.set('base', next.basis);
    setParams(p, { replace: true });
  };
  return { month, basis, current, today, update };
}

export default function Painel() {
  const { month, basis: chosen, current, today, update } = usePanelParams();
  // Perfil (docs/profile.md): Iniciante vê o painel simples; Intermediário, o resumo do mês
  // e os quadros do dia a dia; "Ver painel completo" mostra tudo.
  const level = useLevel();
  const full = useUi((s) => s.fullDashboard);
  const setFull = useUi((s) => s.setFullDashboard);
  const simple = level === 'iniciante' && !full;
  const reduced = level === 'intermediario' && !full;
  // "Ver como" um membro (docs/members.md): o painel dele; sem orçamento e carteira da família.
  const { memberId, member } = useViewAs();
  const memberArg = memberId ? { member_id: memberId } : {};
  const dashboard = useDashboard({ month, ...memberArg });
  const basis: Basis = chosen ?? dashboard.data?.basis ?? 'accrual';
  const override = dashboard.data !== undefined && basis !== dashboard.data.basis;
  const flow = useMonthFlow(
    { from_month: shiftMonth(month, -11), to_month: month, basis, ...memberArg },
    override,
  );
  const spending = useSpending(
    {
      ...monthBounds(month),
      basis,
      kind: 'expense',
      group_by: 'parent_category',
      ...(memberId ? { member_ids: [memberId] } : {}),
    },
    override,
  );
  const budget = useBudget({ month, basis });
  const history = useNetWorthHistory({ months: 12, end_month: month, ...memberArg });
  const portfolio = usePortfolioSummary({});
  const colors = useCategoryColors();
  // "Quanto posso gastar hoje": só no mês atual e na visão da família (docs/forecast.md).
  const allowance = (simple || reduced) && month === current && !memberId;

  const view = useMemo(
    () =>
      dashboard.data
        ? composeDashboard(dashboard.data, basis, {
            flowHistory: flow.data,
            spending: spending.data,
            budget: budget.data,
          })
        : undefined,
    [dashboard.data, basis, flow.data, spending.data, budget.data],
  );

  // Contexto para o agente: o mês e os números principais do painel.
  const d = dashboard.data;
  usePageContext(
    d
      ? {
          screen: 'painel',
          title: `${meta.index} · ${meta.title}`,
          period: { month, basis },
          figures: {
            net_worth: money(d.net_worth.net, d.currency),
            income: money(d.flow.income, d.currency),
            expense: money(d.flow.expense, d.currency),
            savings_rate_12m: d.savings_rate_12m,
            budget_planned: money(d.budget_planned, d.currency),
            budget_actual: money(d.budget_actual, d.currency),
            uncategorized: d.uncategorized_count,
          },
          items: [
            ...d.top_categories.map((c) => ({
              top_category: c.label,
              amount: money(c.amount, d.currency),
            })),
            ...d.upcoming.slice(0, 6).map((u) => ({
              upcoming: u.description,
              date: u.date,
              amount: money(u.amount, u.currency),
            })),
          ],
        }
      : null,
  );

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        meta={t('painel-atualizado', { data: formatDate(today) })}
        description={
          member
            ? t('painel-descricao-membro', { nome: member.name })
            : simple
              ? t('painel-descricao-simples')
              : meta.summary
        }
        actions={
          <>
            <MonthPicker month={month} max={current} onChange={(m) => update({ month: m })} />
            {simple ? null : <BasisToggle value={basis} onChange={(b) => update({ basis: b })} />}
            {level !== 'avancado' && full ? (
              <Button size="sm" variant="ghost" onClick={() => setFull(false)}>
                {t('painel-voltar-simples')}
              </Button>
            ) : null}
          </>
        }
      />
      <QueryView query={dashboard} loadingLabel={t('painel-carregando')}>
        {(d) =>
          simple ? (
            <div className="flex flex-col gap-5">
              <CoverageLine />
              <AttentionCard />
              <div className="grid grid-cols-1 items-start gap-4 @min-[1040px]:grid-cols-2">
                <NextStepCard />
                <MonthCloseCard />
              </div>
              {allowance ? <DailyAllowanceCard /> : null}
              {memberId ? null : <InsightsCard limit={3} />}
              <SimpleDashboard dashboard={d} onShowFull={() => setFull(true)} />
              {memberId ? null : <MembersCard month={month} />}
              <SupportCard viewAs={Boolean(memberId)} />
            </div>
          ) : view ? (
            <div className="flex flex-col gap-5">
              <CoverageLine />
              <AttentionCard />
              <div className="grid grid-cols-1 items-start gap-4 @min-[1040px]:grid-cols-2">
                <NextStepCard />
                <MonthCloseCard />
              </div>
              {reduced ? <MonthSummary dashboard={d} /> : null}
              {allowance ? <DailyAllowanceCard /> : null}
              {memberId ? null : <InsightsCard />}
              <div className="grid grid-cols-1 gap-4 @min-[1040px]:grid-cols-12">
                {reduced ? null : <NetWorthCard netWorth={d.net_worth} history={history.data} />}
                <FlowCard view={view} />
                {memberId ? null : <BudgetCard view={view} />}
                {reduced || memberId ? null : <PortfolioCard summary={portfolio.data} />}
                {memberId ? null : <MembersCard month={month} />}
                <TopCategoriesCard view={view} colors={colors} />
                <UpcomingCard items={d.upcoming} today={today} />
                <UncategorizedCard view={view} />
                <HealthCard month={month} current={current} memberId={memberId} />
              </div>
              {reduced ? (
                <div>
                  <Button variant="ghost" size="sm" onClick={() => setFull(true)}>
                    {t('painel-ver-completo')}
                  </Button>
                </div>
              ) : null}
              <SupportCard viewAs={Boolean(memberId)} />
            </div>
          ) : null
        }
      </QueryView>
      <ExplainDialog />
    </div>
  );
}
