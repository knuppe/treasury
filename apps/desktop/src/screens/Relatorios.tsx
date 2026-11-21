import { useMemo, type ComponentType } from 'react';
import { useSearchParams } from 'react-router';

import { useBudget } from '@/api/hooks/budget';
import { useAppSettings } from '@/api/hooks/settings';
import { useCountryFeatures } from '@/api/hooks/country';
import { screenMeta } from '@/app/screens';
import { ScreenHeader, Tabs, TabsContent, TabsList, TabsTrigger } from '@/design/components';
import { HealthPanel } from '@/features/health/HealthPanel';
import { ExplainDialog } from '@/features/reports/explain/ExplainDialog';
import { FilterBar } from '@/features/reports/FilterBar';
import { PdfButton } from '@/features/reports/PdfButton';
import { TaxPanel } from '@/features/tax/TaxPanel';
import type { FigureProps } from '@/features/reports/figures/common';
import { GroupsFigure } from '@/features/reports/figures/GroupsFigure';
import { InflationFigure } from '@/features/reports/figures/InflationFigure';
import {
  CompositionFigure,
  FlowFigure,
  RhythmFigure,
} from '@/features/reports/figures/FlowFigures';
import {
  BudgetFigure,
  ComparisonFigure,
  TrendFigure,
} from '@/features/reports/figures/TrendFigures';
import {
  CardsFigure,
  MembersFigure,
  NetWorthFigure,
  RecurringFigure,
} from '@/features/reports/figures/WealthFigures';
import {
  filtersPeriod,
  filtersSearch,
  parseFilters,
  periodLabel,
  VIEWS,
  type ReportFilters,
  type ViewId,
} from '@/features/reports/model/period';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';
import { monthOf } from '@/lib/months';
import { usePageContext } from '@/features/companion/pageContext';

const meta = screenMeta('relatorios');

const FIGURES: Record<ViewId, ComponentType<FigureProps>> = {
  fluxo: FlowFigure,
  composicao: CompositionFigure,
  ritmo: RhythmFigure,
  tendencia: TrendFigure,
  orcamento: BudgetFigure,
  comparacao: ComparisonFigure,
  inflacao: InflationFigure,
  titulares: MembersFigure,
  recorrentes: RecurringFigure,
  cartoes: CardsFigure,
  patrimonio: NetWorthFigure,
  grupos: GroupsFigure,
};

/** Visualizações que usam o período dos filtros (as outras olham para a frente ou para 24 meses). */
const USES_PERIOD: ReadonlySet<ViewId> = new Set([
  'fluxo',
  'composicao',
  'ritmo',
  'tendencia',
  'orcamento',
  'comparacao',
  'titulares',
  'patrimonio',
  'grupos',
]);

/** Visualizações que olham para a frente (compromissos futuros), sem período. */
const FORWARD_VIEWS: ReadonlySet<ViewId> = new Set(['recorrentes', 'cartoes']);

export default function Relatorios() {
  const [params, setParams] = useSearchParams();
  const today = todayIso();
  const filters = useMemo(() => parseFilters(params, today), [params, today]);
  const period = useMemo(() => filtersPeriod(filters, today), [filters, today]);
  // A base padrão do cofre vem do orçamento do mês (o núcleo resolve `budget_basis`).
  const defaults = useBudget({ month: monthOf(today) });
  const settings = useAppSettings();
  const view = VIEWS.find((v) => v.id === filters.view)!;
  // O diagrama de fluxo tem a base própria (padrão caixa): resgates e aportes são dinheiro.
  const defaultBasis =
    filters.view === 'fluxo'
      ? (settings.data?.flow_basis ?? 'cash')
      : (defaults.data?.basis ?? 'accrual');
  const setFilters = (f: ReportFilters) =>
    setParams(new URLSearchParams(filtersSearch(f)), { replace: true });
  // As abas IR e Saúde financeira têm filtros próprios (ano, titular, mês): ficam fora de
  // `ReportFilters`.
  // Apuração de IR e DARF: só no Brasil (docs/countries.md).
  const features = useCountryFeatures();
  const ir = features.br_income_tax && params.get('v') === 'ir';
  const saude = params.get('v') === 'saude';
  const own = ir || saude;
  usePageContext({
    screen: 'relatorios',
    title: `${meta.index} · ${meta.title}`,
    view: ir ? 'ir' : saude ? 'saude' : view.id,
    period: own
      ? undefined
      : { from: period.from, to: period.to, basis: filters.basis ?? undefined },
    filters: own
      ? Object.fromEntries(params.entries())
      : { member_id: filters.member, account_id: filters.account },
  });
  const props: FigureProps = {
    period,
    basis: filters.basis,
    memberId: filters.member,
    accountId: filters.account,
    today,
  };

  return (
    <div className="flex flex-col gap-5">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={
          ir
            ? t('relatorios-descricao-ir')
            : saude
              ? t('relatorios-descricao-saude')
              : t('relatorios-descricao-tela')
        }
        meta={
          ir
            ? t('relatorios-imposto-renda')
            : saude
              ? t('relatorios-saude-financeira')
              : USES_PERIOD.has(filters.view)
                ? periodLabel(period)
                : FORWARD_VIEWS.has(filters.view)
                  ? t('relatorios-visao-frente')
                  : view.label
        }
        actions={own ? undefined : <PdfButton month={monthOf(period.to)} basis={filters.basis} />}
      />
      {own ? null : (
        <FilterBar
          filters={filters}
          onChange={setFilters}
          defaultBasis={defaultBasis}
          today={today}
          usesPeriod={USES_PERIOD.has(filters.view)}
        />
      )}
      <Tabs
        value={ir ? 'ir' : saude ? 'saude' : filters.view}
        onValueChange={(v) =>
          v === 'ir' || v === 'saude'
            ? setParams(new URLSearchParams({ v }), { replace: true })
            : setFilters({ ...filters, view: v as ViewId })
        }
      >
        <TabsList aria-label={t('relatorios-visualizacoes')}>
          {VIEWS.map((v, i) => (
            <TabsTrigger key={v.id} value={v.id} title={v.question}>
              <span className="num text-[10px] text-accent">{String(i + 1).padStart(2, '0')}</span>
              {v.label}
            </TabsTrigger>
          ))}
          {features.br_income_tax ? (
            <TabsTrigger value="ir" title={t('relatorios-aba-ir-dica')}>
              <span className="num text-[10px] text-accent">{t('relatorios-aba-ir-sigla')}</span>
              {t('relatorios-imposto-renda')}
            </TabsTrigger>
          ) : null}
          <TabsTrigger value="saude" title={t('relatorios-aba-saude-dica')}>
            <span className="num text-[10px] text-accent">{t('relatorios-aba-saude-sigla')}</span>
            {t('relatorios-saude-financeira')}
          </TabsTrigger>
        </TabsList>
        {VIEWS.map((v) => {
          const Figure = FIGURES[v.id];
          return (
            <TabsContent
              key={v.id}
              value={v.id}
              aria-label={t('relatorios-aba-aria', { vista: v.label, pergunta: v.question })}
            >
              {!own && v.id === view.id ? <Figure {...props} /> : null}
            </TabsContent>
          );
        })}
        <TabsContent value="ir" aria-label={t('relatorios-aba-ir-aria')}>
          {ir ? <TaxPanel /> : null}
        </TabsContent>
        <TabsContent value="saude" aria-label={t('relatorios-aba-saude-aria')}>
          {saude ? <HealthPanel /> : null}
        </TabsContent>
      </Tabs>
      <ExplainDialog />
    </div>
  );
}
