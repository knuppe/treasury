/**
 * 07 · CARTEIRA: resumo (valor, custo, resultado, proventos, XIRR nominal e real),
 * posições, rentabilidade × CDI × inflação, alocação com alvo e rebalanceamento,
 * operações, proventos e renda passiva.
 *
 * Estado na URL: `?aba=` (posicoes | rentabilidade | alocacao | operacoes | vendas |
 * proventos | renda | taxas | renda-fixa | risco | saude | bens) e `?nova=operacao`, `?novo=provento`, `?novo=proventos` (grade), `?novo=evento` abrem os
 * formulários (usados pela barra de comando); `&ativo=<id>` pré-escolhe o ativo.
 * `?importar=b3` e `?importar=notas` abrem as importações da B3 e das notas.
 *
 * A aba Bens é o patrimônio físico (imóveis, veículos, terrenos, equipamentos): fica aqui,
 * ao lado dos investimentos, porque é patrimônio que se valoriza ou deprecia; o
 * financiamento de cada bem continua em 07 · Dívidas. Parâmetros dela em `PhysicalPanel`.
 *
 * A aba Calcular renda fixa (`FixedIncomeCalcPanel`) só aparece no Brasil e não depende da
 * carteira: compara ofertas digitadas e simula a venda antes do vencimento.
 *
 * A aba Saúde dos ativos é o score de saúde (fundamentos) de cada ativo com posição ou na
 * watchlist (`ScorePortfolioPanel`); não é a saúde financeira da família.
 */
import { StudyNotice } from '@/features/terms/Disclaimer';
import { useCallback, useMemo } from 'react';
import { Link, useSearchParams } from 'react-router';

import type { BalanceDue } from '@/api/generated/BalanceDue';

import { useCountryFeatures } from '@/api/hooks/country';
import { screenMeta } from '@/app/screens';
import { usePassiveIncome, usePlansPending, usePortfolioSummary } from '@/api/hooks/portfolio';
import { useRegisterCommands, type AppCommand } from '@/commands/registry';
import {
  Button,
  EmptyState,
  Icon,
  ScreenHeader,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/design/components';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { ActionDialog } from '@/features/portfolio/ActionDialog';
import { AllocationPanel } from '@/features/portfolio/AllocationPanel';
import { DeflatorSelect } from '@/features/portfolio/components/DeflatorSelect';
import { ErrorBlock, LoadingBlock, Notice } from '@/features/portfolio/components/States';
import { B3ImportDialog } from '@/features/portfolio/import/B3ImportDialog';
import { NotesImportDialog } from '@/features/portfolio/import/NotesImportDialog';
import { FeeCostPanel } from '@/features/portfolio/FeeCostPanel';
import { FixedIncomeCalcPanel } from '@/features/portfolio/FixedIncomeCalcPanel';
import { RiskPanel } from '@/features/portfolio/RiskPanel';
import { IncomeDialog } from '@/features/portfolio/IncomeDialog';
import { usePendingToConfirm } from '@/features/portfolio/PendingIncomeConfirm';
import { QuickIncomeDialog } from '@/features/portfolio/QuickIncomeDialog';
import { IncomePanel, PassiveIncomePanel } from '@/features/portfolio/IncomePanels';
import { useDeflatorLabel } from '@/features/portfolio/useDeflatorLabel';
import { PerformancePanel } from '@/features/portfolio/PerformancePanel';
import { SalesPanel } from '@/features/portfolio/SalesPanel';
import { ScheduledContributions } from '@/features/portfolio/ScheduledContributions';
import { PhysicalPanel } from '@/features/physical/PhysicalPanel';
import { ScorePortfolioPanel } from '@/features/score/ScorePortfolioPanel';
import { DataGapsNotice } from '@/features/assets/data/DataGapsNotice';
import { DataGapsPanel } from '@/features/assets/data/DataGapsPanel';
import { PositionsPanel } from '@/features/portfolio/PositionsPanel';
import { ActionsTable, TradesTable } from '@/features/portfolio/RecordsPanels';
import { usePortfolioUi } from '@/features/portfolio/state';
import { SummaryInstruments } from '@/features/portfolio/SummaryInstruments';
import { TradeDialog } from '@/features/portfolio/TradeDialog';
import { money, usePageContext } from '@/features/companion/pageContext';

const meta = screenMeta('carteira');

const TAB_LABEL = {
  posicoes: 'carteira-aba-posicoes',
  rentabilidade: 'carteira-aba-rentabilidade',
  alocacao: 'carteira-aba-alocacao',
  operacoes: 'carteira-aba-operacoes',
  vendas: 'carteira-aba-vendas',
  proventos: 'carteira-aba-proventos',
  renda: 'carteira-aba-renda',
  taxas: 'carteira-aba-taxas',
  'renda-fixa': 'carteira-rf-aba',
  risco: 'carteira-aba-risco',
  saude: 'carteira-aba-saude',
  bens: 'carteira-aba-bens',
} as const;

type TabId = keyof typeof TAB_LABEL;

/** Abas só do Brasil (renda fixa brasileira: CDI, IR regressivo, Tesouro, poupança). */
const BR_ONLY_TABS: ReadonlySet<TabId> = new Set<TabId>(['renda-fixa']);

/** Abas na ordem de exibição; o rótulo vem do catálogo na hora (getter). */
const TABS: readonly { id: TabId; readonly label: string }[] = (
  Object.keys(TAB_LABEL) as TabId[]
).map((id) => ({
  id,
  get label() {
    return t(TAB_LABEL[id]);
  },
}));

function ValuationNotice({ missing }: { missing: readonly string[] }) {
  if (missing.length === 0) return null;
  const head = missing.slice(0, 6).join(', ');
  const lista =
    missing.length > 6 ? t('carteira-lista-e-mais', { lista: head, n: missing.length - 6 }) : head;
  return (
    <Notice tone="warning" title={t('carteira-avaliados-pelo-custo', { n: missing.length, lista })}>
      {tRich('carteira-avaliados-texto', {
        link: (
          <Link className="text-accent underline-offset-2 hover:underline" to="/ativos?aba=indices">
            {t('carteira-link-indices')}
          </Link>
        ),
      })}
    </Notice>
  );
}

/** Previdência sem saldo informado (ou com saldo antigo): um lembrete, não o aviso de custo. */
function BalanceDueNotice({ due }: { due: readonly BalanceDue[] }) {
  if (due.length === 0) return null;
  return (
    <Notice tone="info" title={t('carteira-previdencia-saldo-titulo', { n: due.length })}>
      <ul className="flex flex-col gap-0.5">
        {due.map((b) => (
          <li key={b.asset_id}>
            <Link
              className="text-accent underline-offset-2 hover:underline"
              to={`/ativo/${encodeURIComponent(b.asset_id)}`}
            >
              {b.name}
            </Link>
            {' · '}
            {b.as_of
              ? t('carteira-previdencia-saldo-de', { data: formatDate(b.as_of) })
              : t('carteira-previdencia-sem-saldo')}
          </li>
        ))}
      </ul>
      <p className="mt-1">{t('carteira-previdencia-saldo-texto')}</p>
    </Notice>
  );
}

/** Aportes programados vencidos esperando a confirmação (fora da aba Operações). */
function PlansDueNotice({ onOpen }: { onOpen: () => void }) {
  const due = usePlansPending();
  const n = due.data?.length ?? 0;
  if (n === 0) return null;
  return (
    <Notice tone="info" title={t('carteira-plano-aviso', { n })}>
      <Button size="sm" variant="ghost" onClick={onOpen}>
        {t('carteira-plano-aviso-ver')}
      </Button>
    </Notice>
  );
}

export default function Carteira() {
  const deflatorLabel = useDeflatorLabel();
  const [params, setParams] = useSearchParams();
  const deflator = usePortfolioUi((s) => s.deflator);
  const summary = usePortfolioSummary({ deflator });
  const passive = usePassiveIncome();
  // Proventos anunciados e não lançados: o botão mostra quantos esperam confirmação.
  const pendingIncome = usePendingToConfirm();

  const brMarket = useCountryFeatures().br_market;
  const tabs = useMemo(
    () => (brMarket ? TABS : TABS.filter((x) => !BR_ONLY_TABS.has(x.id))),
    [brMarket],
  );
  const tab = (
    tabs.some((x) => x.id === params.get('aba')) ? params.get('aba') : 'posicoes'
  ) as TabId;
  const dialog =
    params.get('nova') === 'operacao'
      ? 'trade'
      : params.get('novo') === 'provento'
        ? 'income'
        : params.get('novo') === 'proventos'
          ? 'quick_income'
          : params.get('novo') === 'evento'
            ? 'action'
            : null;
  const presetAsset = params.get('ativo') ?? undefined;
  const importing = params.get('importar');
  const update = useCallback(
    (patch: Record<string, string | null>) => {
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
    },
    [setParams],
  );
  const openDialog = useCallback(
    (kind: 'trade' | 'income' | 'quick_income' | 'action') =>
      update({
        nova: kind === 'trade' ? 'operacao' : null,
        novo:
          kind === 'income'
            ? 'provento'
            : kind === 'quick_income'
              ? 'proventos'
              : kind === 'action'
                ? 'evento'
                : null,
      }),
    [update],
  );
  const closeDialog = () => update({ nova: null, novo: null, ativo: null });
  // Área do Investidor B3 e notas de corretagem: só no Brasil (docs/countries.md).
  const importButtons = brMarket ? (
    <>
      <Button onClick={() => update({ importar: 'b3' })}>{t('carteira-importar-b3')}</Button>
      <Button onClick={() => update({ importar: 'notas' })}>{t('carteira-importar-notas')}</Button>
    </>
  ) : null;

  const commands = useMemo<AppCommand[]>(
    () => [
      {
        id: 'carteira.evento',
        title: t('carteira-ev-titulo'),
        group: t('casca-grupo-investimentos'),
        keywords: t('carteira-cmd-evento-busca').split(','),
        perform: () => openDialog('action'),
      },
    ],
    [openDialog],
  );
  useRegisterCommands(commands);

  const s = summary.data;
  const empty = s && s.positions.length === 0 && s.realized === 0 && s.income === 0;

  // Contexto para o agente: totais da carteira e as maiores posições.
  usePageContext({
    screen: 'carteira',
    title: `${meta.index} · ${meta.title}`,
    view: tab,
    selection: presetAsset ? { asset_id: presetAsset } : undefined,
    figures: s
      ? {
          as_of: s.as_of,
          market_value: money(s.market_value, s.base_currency),
          cost: money(s.cost, s.base_currency),
          total_result: money(s.total_result, s.base_currency),
          income: money(s.income, s.base_currency),
          xirr: s.xirr,
          xirr_real: s.xirr_real,
          deflator: s.deflator,
          positions: s.positions.length,
          missing_prices: s.missing_prices.length,
        }
      : undefined,
    items: s?.positions
      .slice()
      .sort((x, y) => (y.market_value_base ?? 0) - (x.market_value_base ?? 0))
      .slice(0, 12)
      .map((p) => ({
        asset_id: p.asset.id,
        ticker: p.asset.ticker ?? p.asset.name,
        class: p.asset.class,
        quantity: p.quantity,
        market_value: money(p.market_value_base, s.base_currency),
        weight: p.weight == null ? null : Math.round(p.weight * 1000) / 1000,
        valuation: p.valuation,
      })),
  });
  const physical = tab === 'bens';
  const tabsList = (
    <TabsList aria-label={t('carteira-secoes')}>
      {tabs.map((tab) => (
        <TabsTrigger key={tab.id} value={tab.id}>
          {tab.label}
        </TabsTrigger>
      ))}
    </TabsList>
  );
  const onTab = (v: string) => update({ aba: v === 'posicoes' ? null : v });

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={meta.summary}
        meta={
          s && !physical
            ? t('carteira-meta', {
                data: formatDate(s.as_of),
                deflator: deflatorLabel(s.deflator),
              })
            : undefined
        }
        actions={
          physical ? undefined : (
            <>
              <Button variant="ghost" onClick={() => openDialog('action')}>
                {t('carteira-evento-corporativo')}
              </Button>
              <Button onClick={() => openDialog('quick_income')}>
                {t('carteira-registrar-proventos')}
                {pendingIncome.length ? (
                  <span className="ml-1.5 rounded-full bg-accent px-1.5 text-[10.5px] text-accent-ink">
                    {pendingIncome.length}
                  </span>
                ) : null}
              </Button>
              <Button variant="ghost" onClick={() => openDialog('income')}>
                {t('carteira-provento-detalhado')}
              </Button>
              <Button
                variant="primary"
                icon={<Icon name="plus" size={12} />}
                onClick={() => openDialog('trade')}
              >
                {t('carteira-nova-operacao')}
              </Button>
            </>
          )
        }
      />

      {physical ? (
        <Tabs value={tab} onValueChange={onTab}>
          {tabsList}
          <TabsContent value="bens">
            <PhysicalPanel />
          </TabsContent>
        </Tabs>
      ) : summary.isPending ? (
        <LoadingBlock label={t('carteira-carregando')} />
      ) : summary.isError ? (
        <ErrorBlock error={summary.error} onRetry={() => void summary.refetch()} />
      ) : !s ? null : empty ? (
        <>
          <Tabs value={tab} onValueChange={onTab}>
            {tabsList}
            {/* A watchlist pode ter ativos mesmo sem operações. */}
            <TabsContent value="saude">
              <ScorePortfolioPanel />
              <DataGapsPanel />
            </TabsContent>
            {/* A calculadora de renda fixa não precisa de posições. */}
            <TabsContent value="renda-fixa">
              <FixedIncomeCalcPanel />
            </TabsContent>
          </Tabs>
          {tab === 'saude' || tab === 'renda-fixa' ? null : (
            <EmptyState
              kicker={t('carteira-vazia-kicker')}
              title={t('carteira-vazia-titulo')}
              description={t('carteira-vazia-texto')}
              actions={
                <>
                  <Button variant="primary" onClick={() => openDialog('trade')}>
                    {t('carteira-nova-operacao')}
                  </Button>
                  {importButtons}
                  <Link
                    to="/ativos?novo=ativo"
                    className="inline-flex h-9 items-center px-3 font-mono text-[11.5px] tracking-[0.08em] text-accent uppercase"
                  >
                    {t('carteira-cadastrar-ativo')}
                  </Link>
                </>
              }
            />
          )}
        </>
      ) : (
        <>
          <div className="flex flex-wrap items-center gap-3">
            <span className="label-caps">{t('carteira-real-descontado-por')}</span>
            <DeflatorSelect />
          </div>
          <ValuationNotice missing={s.missing_prices} />
          <BalanceDueNotice due={s.balance_due} />
          {tab === 'operacoes' ? null : <PlansDueNotice onOpen={() => onTab('operacoes')} />}
          {tab === 'saude' ? null : <DataGapsNotice />}
          <SummaryInstruments summary={s} passive={passive.data} />
          <Tabs value={tab} onValueChange={onTab}>
            {tabsList}
            <TabsContent value="posicoes">
              <PositionsPanel />
            </TabsContent>
            <TabsContent value="rentabilidade">
              <PerformancePanel />
            </TabsContent>
            <TabsContent value="alocacao">
              <AllocationPanel />
            </TabsContent>
            <TabsContent value="vendas">
              <SalesPanel />
            </TabsContent>
            <TabsContent value="operacoes" className="flex flex-col gap-6">
              <div className="flex flex-wrap justify-end gap-2">{importButtons}</div>
              <ScheduledContributions />
              <TradesTable />
              <section className="flex flex-col gap-2">
                <h3 className="label-caps text-fg">{t('carteira-eventos-corporativos')}</h3>
                <ActionsTable />
              </section>
            </TabsContent>
            <TabsContent value="proventos">
              <IncomePanel />
            </TabsContent>
            <TabsContent value="renda">
              <PassiveIncomePanel />
            </TabsContent>
            <TabsContent value="taxas">
              <FeeCostPanel />
            </TabsContent>
            <TabsContent value="renda-fixa">
              <FixedIncomeCalcPanel />
            </TabsContent>
            <TabsContent value="risco">
              <RiskPanel />
            </TabsContent>
            <TabsContent value="saude">
              <ScorePortfolioPanel />
              <DataGapsPanel />
            </TabsContent>
          </Tabs>
        </>
      )}

      <StudyNotice />
      <TradeDialog
        open={dialog === 'trade'}
        onOpenChange={(o) => !o && closeDialog()}
        assetId={presetAsset}
      />
      <IncomeDialog
        open={dialog === 'income'}
        onOpenChange={(o) => !o && closeDialog()}
        assetId={presetAsset}
      />
      <QuickIncomeDialog
        open={dialog === 'quick_income'}
        onOpenChange={(o) => !o && closeDialog()}
      />
      <ActionDialog
        open={dialog === 'action'}
        onOpenChange={(o) => !o && closeDialog()}
        assetId={presetAsset}
      />
      <B3ImportDialog
        open={importing === 'b3'}
        onOpenChange={(o) => !o && update({ importar: null })}
      />
      <NotesImportDialog
        open={importing === 'notas'}
        onOpenChange={(o) => !o && update({ importar: null })}
      />
    </div>
  );
}
