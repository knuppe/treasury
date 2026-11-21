/**
 * 10 · IMPOSTOS: estimativa dos impostos pagos no ano (docs/taxes.md), a partir dos dados
 * do app, em três cenários (conservador, provável, elevado). O núcleo devolve os três de uma
 * vez (`tax_burden.estimate`); o seletor troca o cenário na tela.
 *
 * Estado na URL: `ano` (padrão: o atual, até hoje), `cenario` (`conservador`, `provavel`,
 * `elevado`; padrão: provável) e `grupos=1` (editor dos grupos das categorias aberto). País, estado e dependentes ficam nos ajustes da estimativa.
 */
import { useSearchParams } from 'react-router';

import type { Scenario } from '@/api/generated/Scenario';
import {
  useIncomeReports,
  useTaxCategories,
  useTaxDatasets,
  useTaxEstimate,
  useTaxSettings,
} from '@/api/hooks/taxBurden';
import { useTxnQuery } from '@/api/hooks/transactions';
import { screenMeta } from '@/app/screens';
import { ScreenHeader } from '@/design/components';
import { money, usePageContext } from '@/features/companion/pageContext';
import { Notice } from '@/features/portfolio/components/States';
import { QueryView } from '@/features/reports/controls';
import { CityCard, DestinationCard } from '@/features/taxburden/Brazil';
import { useCategoryEditor } from '@/features/taxburden/CategoryDialogs';
import { CompositionCard, ScenariosCard } from '@/features/taxburden/Composition';
import { ConsumptionCard } from '@/features/taxburden/ConsumptionPanel';
import { PlaceControls, ScenarioBar, YearSelect } from '@/features/taxburden/Controls';
import { GroupsDialog } from '@/features/taxburden/GroupsDialog';
import {
  DEFAULT_SCENARIO,
  defaultDisclaimer,
  isDraft,
  pick,
  scenarioDef,
  scenarioFromParam,
  yearOptions,
} from '@/features/taxburden/model';
import {
  DirectCard,
  InvestmentsCard,
  PayrollCard,
  PropertyCard,
} from '@/features/taxburden/OtherTaxes';
import { SourcesButton, sourceDate } from '@/features/taxburden/SourcesDialog';
import { ContextCard, NotesCard, SummaryCard } from '@/features/taxburden/Summary';
import { UnestimatedCard } from '@/features/taxburden/Unestimated';
import { TaxNotice } from '@/features/terms/Disclaimer';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';
import { formatDate, todayIso } from '@/lib/format';

const meta = screenMeta('impostos');

export default function Impostos() {
  const [params, setParams] = useSearchParams();
  const today = todayIso();
  const current = Number(today.slice(0, 4));
  const first = useTxnQuery({ sort: 'date_asc', limit: 1, statuses: ['cleared', 'reconciled'] });
  const allReports = useIncomeReports();
  const firstDate = first.data?.items[0]?.date;
  const years = yearOptions(
    today,
    firstDate ? Number(firstDate.slice(0, 4)) : null,
    (allReports.data ?? []).map((r) => r.year),
  );
  const yearParam = Number(params.get('ano'));
  const year = years.includes(yearParam) ? yearParam : current;
  const scenario = scenarioFromParam(params.get('cenario'));

  const estimate = useTaxEstimate(year === current ? null : year);
  const base = useBaseCurrency();
  const settings = useTaxSettings();
  const datasets = useTaxDatasets();
  const categories = useTaxCategories();

  const e = estimate.data;
  const dataset = datasets.data?.find((d) => d.country === e?.dataset.country);
  const editor = useCategoryEditor({ dataset, categories: categories.data });

  const set = (patch: Record<string, string | null>) =>
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        for (const [k, v] of Object.entries(patch)) {
          if (v == null) next.delete(k);
          else next.set(k, v);
        }
        return next;
      },
      { replace: true },
    );
  const groupsOpen = params.get('grupos') === '1';
  const setGroupsOpen = (open: boolean) => set({ grupos: open ? '1' : null });
  const setScenario = (s: Scenario) =>
    set({ cenario: s === DEFAULT_SCENARIO ? null : scenarioDef(s).param });

  usePageContext(
    e
      ? {
          screen: 'impostos',
          title: `${meta.index} · ${meta.title}`,
          view: scenario,
          period: { from: e.from, to: e.to },
          filters: {
            year: e.year,
            scenario,
            country: e.dataset.country,
            region: e.settings.region,
            dataset_version: e.dataset.version,
          },
          figures: {
            total: money(pick(e.totals, scenario), e.currency),
            total_conservative: money(e.totals.conservative, e.currency),
            total_high: money(e.totals.high, e.currency),
            share_of_income: pick(e.share_of_income, scenario),
            share_of_consumption: pick(e.share_of_consumption, scenario),
            income_base: money(e.income_base, e.currency),
            consumption_spent: money(e.consumption_spent, e.currency),
          },
          items: [
            ...e.parts.map((p) => ({
              part: p.label,
              amount: money(pick(p.amount, scenario), e.currency),
              estimated: p.estimated,
            })),
            ...e.consumption.slice(0, 8).map((c) => ({
              category: c.path,
              group: c.group_label,
              rate: pick(c.rates, scenario),
              tax: money(pick(c.taxes, scenario), e.currency),
            })),
          ],
        }
      : null,
  );

  const actions = e && dataset ? <SourcesButton dataset={dataset} info={e.dataset} /> : null;

  return (
    <div className="flex flex-col gap-5">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        meta={
          e
            ? t('impostos-periodo', { de: formatDate(e.from), ate: formatDate(e.to) })
            : String(year)
        }
        description={t('impostos-descricao')}
        actions={
          <div className="flex flex-wrap items-end gap-3">
            <YearSelect
              year={year}
              years={years}
              onChange={(y) => set({ ano: y === current ? null : String(y) })}
            />
            {settings.data && datasets.data ? (
              <PlaceControls settings={settings.data} datasets={datasets.data} />
            ) : null}
          </div>
        }
      />
      <ScenarioBar
        scenario={scenario}
        onScenario={setScenario}
        total={e ? pick(e.totals, scenario) : null}
        currency={e?.currency ?? base}
        actions={actions}
      />
      <QueryView query={estimate} loadingLabel={t('impostos-estimando')}>
        {(e) => (
          <div className="flex flex-col gap-4">
            <Notice title={t('impostos-aviso-titulo')}>
              {dataset?.disclaimer ?? defaultDisclaimer()}
            </Notice>
            {e.dataset.stale ? (
              <Notice tone="warning" title={t('impostos-desatualizados')}>
                {t('impostos-desatualizados-texto', {
                  dados: e.dataset.name,
                  data: sourceDate(e.dataset.updated),
                  versao: e.dataset.version,
                })}
              </Notice>
            ) : null}
            {isDraft(e.dataset.coverage) ? (
              <Notice tone="warning" title={t('impostos-esboco-titulo', { dados: e.dataset.name })}>
                {dataset?.methodology[0] ?? t('impostos-esboco-texto')}
              </Notice>
            ) : null}
            <div className="grid grid-cols-1 gap-4 @min-[1040px]:grid-cols-12">
              <SummaryCard estimate={e} scenario={scenario} />
              <ContextCard estimate={e} dataset={dataset} />
              <NotesCard notes={e.notes} />
              <CompositionCard estimate={e} scenario={scenario} />
              <ScenariosCard estimate={e} scenario={scenario} />
              <ConsumptionCard
                estimate={e}
                scenario={scenario}
                editor={editor}
                onOpenGroups={() => setGroupsOpen(true)}
              />
              <UnestimatedCard estimate={e} editor={editor} />
              <DirectCard estimate={e} />
              <PropertyCard estimate={e} dataset={dataset} />
              <InvestmentsCard estimate={e} />
              <PayrollCard estimate={e} />
              <DestinationCard estimate={e} scenario={scenario} dataset={dataset} />
              <CityCard estimate={e} dataset={dataset} />
            </div>
            <TaxNotice
              source={e.dataset.name}
              date={sourceDate(e.dataset.updated)}
              country={e.dataset.country}
            />
          </div>
        )}
      </QueryView>
      {editor.dialogs}
      <GroupsDialog dataset={dataset} open={groupsOpen} onOpenChange={setGroupsOpen} />
    </div>
  );
}
