/**
 * Hooks TanStack Query da carteira (`portfolio.*`). Chaves no formato
 * `['portfolio', acao, args]`: o Painel lê `['portfolio', 'summary', args]`.
 *
 * Toda escrita invalida a carteira inteira e o que o caixa da corretora mexe
 * (contas, lançamentos, relatórios): compras, vendas e proventos geram lançamentos.
 */
import {
  keepPreviousData,
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
  type QueryClient,
} from '@tanstack/react-query';

import { call } from '@/api/client';
import type { AccountView } from '@/api/generated/AccountView';
import type { BenchmarkArgs } from '@/api/generated/BenchmarkArgs';
import type { BenchmarkSeries } from '@/api/generated/BenchmarkSeries';
import type { ContributionArgs } from '@/api/generated/ContributionArgs';
import type { ContributionReport } from '@/api/generated/ContributionReport';
import type { BulkResult } from '@/api/generated/BulkResult';
import type { ComeCotasEvent } from '@/api/generated/ComeCotasEvent';
import type { ConcentrationReport } from '@/api/generated/ConcentrationReport';
import type { SetConcentrationLimitArgs } from '@/api/generated/SetConcentrationLimitArgs';
import type { ComeCotasRegime } from '@/api/generated/ComeCotasRegime';
import type { ComeCotasRule } from '@/api/generated/ComeCotasRule';
import type { CompareFixedIncomeArgs } from '@/api/generated/CompareFixedIncomeArgs';
import type { CorporateAction } from '@/api/generated/CorporateAction';
import type { CostMethodState } from '@/api/generated/CostMethodState';
import type { SetCostMethodArgs } from '@/api/generated/SetCostMethodArgs';
import type { FeeCost } from '@/api/generated/FeeCost';
import type { TradeSummary } from '@/api/generated/TradeSummary';
import type { FixedIncomeComparison } from '@/api/generated/FixedIncomeComparison';
import type { FixedIncomeSale } from '@/api/generated/FixedIncomeSale';
import type { FixedIncomeSaleArgs } from '@/api/generated/FixedIncomeSaleArgs';
import type { PortfolioRisk } from '@/api/generated/PortfolioRisk';
import type { PortfolioRiskArgs } from '@/api/generated/PortfolioRiskArgs';
import type { FeeCostArgs } from '@/api/generated/FeeCostArgs';
import type { CorporateActionInput } from '@/api/generated/CorporateActionInput';
import type { Deflator } from '@/api/generated/Deflator';
import type { DividendCalendarArgs } from '@/api/generated/DividendCalendarArgs';
import type { DividendCalendarItem } from '@/api/generated/DividendCalendarItem';
import type { IncomeEvent } from '@/api/generated/IncomeEvent';
import type { InvestmentPlanInput } from '@/api/generated/InvestmentPlanInput';
import type { InvestmentPlanView } from '@/api/generated/InvestmentPlanView';
import type { PlanConfirmArgs } from '@/api/generated/PlanConfirmArgs';
import type { PlanDue } from '@/api/generated/PlanDue';
import type { PlanSkipArgs } from '@/api/generated/PlanSkipArgs';
import type { IncomeInput } from '@/api/generated/IncomeInput';
import type { IncomeLedger } from '@/api/generated/IncomeLedger';
import type { IncomeLedgerArgs } from '@/api/generated/IncomeLedgerArgs';
import type { IncomeRebuildArgs } from '@/api/generated/IncomeRebuildArgs';
import type { IncomeRebuildReport } from '@/api/generated/IncomeRebuildReport';
import type { PassiveIncome } from '@/api/generated/PassiveIncome';
import type { PassiveIncomeArgs } from '@/api/generated/PassiveIncomeArgs';
import type { PendingIncome } from '@/api/generated/PendingIncome';
import type { PendingIncomeArgs } from '@/api/generated/PendingIncomeArgs';
import type { SalesArgs } from '@/api/generated/SalesArgs';
import type { SalesReport } from '@/api/generated/SalesReport';
import type { UncoveredSaleAcceptArgs } from '@/api/generated/UncoveredSaleAcceptArgs';
import type { UncoveredSales } from '@/api/generated/UncoveredSales';
import type { UncoveredSalesArgs } from '@/api/generated/UncoveredSalesArgs';
import type { Performance } from '@/api/generated/Performance';
import type { PerformanceArgs } from '@/api/generated/PerformanceArgs';
import type { PerformanceBridge } from '@/api/generated/PerformanceBridge';
import type { PortfolioArgs } from '@/api/generated/PortfolioArgs';
import type { PortfolioSummary } from '@/api/generated/PortfolioSummary';
import type { PositionView } from '@/api/generated/PositionView';
import type { QuickIncomeArgs } from '@/api/generated/QuickIncomeArgs';
import type { QuickIncomeLine } from '@/api/generated/QuickIncomeLine';
import type { QuickIncomeReport } from '@/api/generated/QuickIncomeReport';
import type { Rebalance } from '@/api/generated/Rebalance';
import type { RebalanceArgs } from '@/api/generated/RebalanceArgs';
import type { Target } from '@/api/generated/Target';
import type { TargetInput } from '@/api/generated/TargetInput';
import type { TargetScope } from '@/api/generated/TargetScope';
import type { StatementLine } from '@/api/generated/StatementLine';
import type { Trade } from '@/api/generated/Trade';
import type { TradeFilter } from '@/api/generated/TradeFilter';
import type { TradeInput } from '@/api/generated/TradeInput';
import type { TradePatch } from '@/api/generated/TradePatch';
import type { WatchlistInput } from '@/api/generated/WatchlistInput';
import type { WatchlistItem } from '@/api/generated/WatchlistItem';

/** Tira chaves `undefined`/`null` para que `{}` e `{ deflator: undefined }` sejam a mesma chave. */
export function cleanArgs<T extends object>(args: T | undefined): Partial<T> {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(args ?? {})) {
    if (v !== undefined && v !== null && v !== '') out[k] = v;
  }
  return out as Partial<T>;
}

export const portfolioKeys = {
  all: ['portfolio'] as const,
  of: (action: string, args?: object) => ['portfolio', action, cleanArgs(args)] as const,
};

function query<T>(action: string, args?: object) {
  const a = cleanArgs(args);
  return {
    queryKey: portfolioKeys.of(action, a),
    queryFn: () => call<T>(`portfolio.${action}`, a),
  };
}

/**
 * Módulos cujo cache muda quando a carteira grava (caixa da corretora, relatórios e o IR
 * retido nos proventos da estimativa de impostos).
 */
const AFFECTED = new Set([
  'portfolio',
  'assets',
  'accounts',
  'transactions',
  'reports',
  'budget',
  'tax_burden',
]);

export function invalidatePortfolio(client: QueryClient): Promise<void> {
  return client.invalidateQueries({
    predicate: (q) => AFFECTED.has(String(q.queryKey[0])),
  });
}

// ── Leituras ─────────────────────────────────────────────────────

export function usePortfolioSummary(args: PortfolioArgs = {}) {
  return useQuery({
    ...query<PortfolioSummary>('summary', args),
    placeholderData: keepPreviousData,
  });
}

export interface PositionArgs {
  asset_id: string;
  account_id?: string;
  as_of?: string;
  deflator?: Deflator;
}

export function usePosition(args: PositionArgs | null) {
  return useQuery({
    ...query<PositionView>('position', args ?? {}),
    enabled: Boolean(args?.asset_id),
    placeholderData: keepPreviousData,
  });
}

/** Posição numa data (para validar vendas): não guarda dados antigos. */
export function fetchPosition(args: PositionArgs): Promise<PositionView> {
  return call<PositionView>('portfolio.position', cleanArgs(args));
}

export function usePerformance(args: PerformanceArgs = {}) {
  return useQuery({
    ...query<Performance>('performance', args),
    placeholderData: keepPreviousData,
  });
}

/** Ponte do resultado: do valor no início ao valor no fim do período, linha a linha. */
export function useBridge(args: PerformanceArgs = {}) {
  return useQuery({
    ...query<PerformanceBridge>('bridge', args),
    placeholderData: keepPreviousData,
  });
}

/** Rentabilidade e ponte juntas (o ajuste `twr_daily` muda o método da rentabilidade). */
export function invalidatePerformance(client: QueryClient): Promise<void> {
  return client.invalidateQueries({
    predicate: (q) =>
      q.queryKey[0] === 'portfolio' &&
      (q.queryKey[1] === 'performance' || q.queryKey[1] === 'bridge'),
  });
}

/** Livro de vendas: resultado nominal e real de cada venda (docs/sales-real-return.md). */
export function useSales(args: SalesArgs = {}) {
  return useQuery({ ...query<SalesReport>('sales', args), placeholderData: keepPreviousData });
}

/**
 * Sales larger than the position on their date: the purchase is older than the imported
 * history. They leave the list when the purchase is recorded or the person accepts them.
 */
export function useUncoveredSales(args: UncoveredSalesArgs = {}) {
  return useQuery(query<UncoveredSales>('uncovered_sales', args));
}

/** Accepts a sale without a cost (or puts it back on the list). */
export function useAcceptUncoveredSale() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: UncoveredSaleAcceptArgs) =>
      call<UncoveredSales>('portfolio.accept_uncovered_sale', args),
    onSuccess: () =>
      Promise.all([
        invalidatePortfolio(client),
        // "To solve" counts the sales still to explain.
        client.invalidateQueries({ queryKey: ['attention'] }),
      ]),
  });
}

/** Método de custo das vendas de renda variável (o do país da família ou o de Ajustes). */
export function useCostMethod() {
  return useQuery(query<CostMethodState>('cost_method'));
}

export function useSetCostMethod() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: SetCostMethodArgs) =>
      call<CostMethodState>('portfolio.set_cost_method', args),
    onSuccess: () => invalidatePortfolio(client),
  });
}

export function useTrades(filter: TradeFilter = {}) {
  return useQuery({ ...query<Trade[]>('list_trades', filter), placeholderData: keepPreviousData });
}

/** Totals of an asset's trades (summary above the trade list). */
export function useTradeSummary(assetId: string) {
  return useQuery(query<TradeSummary>('trade_summary', { asset_id: assetId }));
}

/** Extrato do ativo: operações e eventos com a posição depois de cada linha. */
export function useAssetStatement(assetId: string) {
  return useQuery(query<StatementLine[]>('statement', { asset_id: assetId }));
}

/** Come-cotas de um fundo: a regra e os descontos de maio e novembro. */
export function useComeCotas(assetId: string, enabled = true) {
  return useQuery({
    ...query<{ rule: ComeCotasRule; events: ComeCotasEvent[]; warnings: string[] }>('come_cotas', {
      asset_id: assetId,
    }),
    enabled,
  });
}

export const useSetComeCotas = () =>
  useWrite<{ asset_id: string; regime: ComeCotasRegime | null }, ComeCotasRule>(
    'portfolio.set_come_cotas',
  );

/** Custo das taxas de fundos e previdência em 10, 20 e 30 anos. */
export function useFeeCost(args: FeeCostArgs) {
  return useQuery({ ...query<FeeCost>('fee_cost', args), placeholderData: keepPreviousData });
}

/** Ofertas de renda fixa lado a lado, na ordem recebida (`null`: formulário incompleto). */
export function useCompareFixedIncome(args: CompareFixedIncomeArgs | null) {
  return useQuery({
    ...query<FixedIncomeComparison>('compare_fixed_income', args ?? {}),
    enabled: args != null,
    placeholderData: keepPreviousData,
  });
}

/** Vender um prefixado ou IPCA+ antes do vencimento (`null`: formulário incompleto). */
export function useFixedIncomeSale(args: FixedIncomeSaleArgs | null) {
  return useQuery({
    ...query<FixedIncomeSale>('simulate_fixed_income_sale', args ?? {}),
    enabled: args != null,
    placeholderData: keepPreviousData,
  });
}

/** Séries quaisquer (ativo, índice, CDI + x%) acumuladas nos meses da rentabilidade. */
export function useBenchmarkSeries(months: string[], targets: BenchmarkArgs['target'][]) {
  return useQueries({
    queries: targets.map((target) => ({
      ...query<BenchmarkSeries>('benchmark_series', { months, target } satisfies BenchmarkArgs),
      enabled: months.length > 0,
      placeholderData: keepPreviousData,
    })),
  });
}

/** Quanto cada ativo somou ou tirou do resultado no período. */
export function useContributions(args: ContributionArgs = {}) {
  return useQuery({
    ...query<ContributionReport>('contributions', args),
    placeholderData: keepPreviousData,
  });
}

/** Risco da carteira de hoje nos últimos `months` meses completos. */
export function useRisk(args: PortfolioRiskArgs) {
  return useQuery({ ...query<PortfolioRisk>('risk', args), placeholderData: keepPreviousData });
}

export function useActions(assetId?: string) {
  return useQuery(query<CorporateAction[]>('list_actions', { asset_id: assetId }));
}

export interface IncomeListArgs {
  asset_id?: string;
  from?: string;
  to?: string;
}

export function useIncome(args: IncomeListArgs = {}) {
  return useQuery({
    ...query<IncomeEvent[]>('list_income', args),
    placeholderData: keepPreviousData,
  });
}

export function useTargets() {
  return useQuery(query<Target[]>('list_targets'));
}

export function useRebalance(args: RebalanceArgs = {}) {
  return useQuery({ ...query<Rebalance>('rebalance', args), placeholderData: keepPreviousData });
}

/** Concentração: alertas, limites e o peso de cada ativo e grupo (mesmos filtros do resumo). */
export function useConcentration(args: PortfolioArgs = {}, options: { enabled?: boolean } = {}) {
  return useQuery({
    ...query<ConcentrationReport>('concentration', args),
    placeholderData: keepPreviousData,
    enabled: options.enabled ?? true,
  });
}

export function useWatchlist() {
  return useQuery(query<WatchlistItem[]>('list_watchlist'));
}

export function useDividendCalendar(args: DividendCalendarArgs) {
  return useQuery({
    ...query<DividendCalendarItem[]>('dividend_calendar', args),
    placeholderData: keepPreviousData,
  });
}

export function usePassiveIncome(args: PassiveIncomeArgs = {}) {
  return useQuery({
    ...query<PassiveIncome>('passive_income', args),
    placeholderData: keepPreviousData,
  });
}

/** Proventos anunciados que ninguém lançou (`input` pronto para `create_income`). */
export function usePendingIncome(args: PendingIncomeArgs = {}) {
  return useQuery(query<PendingIncome[]>('pending_income', args));
}

/** Income ledger: every distribution the family was entitled to against what is recorded. */
export function useIncomeLedger(args: IncomeLedgerArgs = {}) {
  return useQuery({
    ...query<IncomeLedger>('income_ledger', args),
    placeholderData: keepPreviousData,
  });
}

/** Aportes programados (os planos, com a próxima data e quantos esperam decisão). */
export function useInvestmentPlans() {
  return useQuery(query<InvestmentPlanView[]>('plans', {}));
}

/** Aportes programados vencidos esperando a confirmação, com a compra sugerida. */
export function usePlansPending() {
  return useQuery(query<PlanDue[]>('plans_pending', {}));
}

/** Prévia dos proventos digitados pelo ticker (o que o núcleo deduz de cada linha). */
export function useQuickIncomePreview(args: QuickIncomeArgs) {
  return useQuery({
    ...query<QuickIncomeLine[]>('quick_income_preview', args),
    enabled: args.rows.length > 0,
    placeholderData: keepPreviousData,
  });
}

/**
 * Contas onde a carteira opera (corretoras e contas no exterior). Lê
 * `accounts.list` com a mesma chave que a tela de contas usa.
 */
/** Tipos de conta onde um ativo pode ficar (sem cartão, dinheiro, benefício e FGTS). */
const CUSTODY_KINDS: AccountView['kind'][] = [
  'brokerage',
  'foreign',
  'checking',
  'savings',
  'retirement',
  'other',
];

/** Contas que podem guardar um ativo (custódia): corretoras, bancos e contas no exterior. */
export function useCustodyAccounts() {
  return useQuery({
    queryKey: ['accounts', 'list', {}] as const,
    queryFn: () => call<AccountView[]>('accounts.list', {}),
    select: (accounts) => accounts.filter((a) => !a.archived && CUSTODY_KINDS.includes(a.kind)),
  });
}

export function useInvestmentAccounts() {
  return useQuery({
    queryKey: ['accounts', 'list', {}] as const,
    queryFn: () => call<AccountView[]>('accounts.list', {}),
    select: (accounts) =>
      accounts.filter((a) => !a.archived && (a.kind === 'brokerage' || a.kind === 'foreign')),
  });
}

// ── Escritas ─────────────────────────────────────────────────────

function useWrite<A, R>(cmd: string) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: A) => call<R>(cmd, args),
    onSuccess: () => invalidatePortfolio(client),
  });
}

export const useSavePlan = () =>
  useWrite<InvestmentPlanInput, InvestmentPlanView>('portfolio.plan_save');
export const useDeletePlan = () =>
  useWrite<{ id: string }, { change_set_id: string | null }>('portfolio.plan_delete');
export const useConfirmPlan = () =>
  useWrite<PlanConfirmArgs, { trade: Trade; change_set_id: string | null }>(
    'portfolio.plan_confirm',
  );
export const useSkipPlan = () =>
  useWrite<PlanSkipArgs, { change_set_id: string | null }>('portfolio.plan_skip');

export const useCreateTrade = () => useWrite<TradeInput, Trade>('portfolio.create_trade');
export const useUpdateTrade = () => useWrite<TradePatch, Trade>('portfolio.update_trade');
export const useDeleteTrade = () => useWrite<{ id: string }, null>('portfolio.delete_trade');

export const useCreateAction = () =>
  useWrite<CorporateActionInput, CorporateAction | null>('portfolio.create_action');
export const useDeleteAction = () => useWrite<{ id: string }, null>('portfolio.delete_action');

export const useCreateIncome = () =>
  useWrite<IncomeInput, IncomeEvent | null>('portfolio.create_income');
/** Grava as linhas prontas numa alteração só. */
export const useQuickIncome = () =>
  useWrite<
    QuickIncomeArgs,
    { status: 'applied' | 'nothing_new'; change_set_id?: string | null; report: QuickIncomeReport }
  >('portfolio.quick_income');
/** Vários proventos de uma vez (os sugeridos que a família confirmou), um "Desfazer" só. */
export const useCreateIncomes = () =>
  useWrite<{ items: IncomeInput[] }, { created: number; change_set_id: string | null }>(
    'portfolio.create_incomes',
  );
export const useDeleteIncome = () => useWrite<{ id: string }, null>('portfolio.delete_income');
/** Applies the ledger's fixes in one change set (and turns the automatic recording on). */
export const useIncomeRebuild = () =>
  useWrite<IncomeRebuildArgs, IncomeRebuildReport>('portfolio.income_rebuild');
export const useIncomeAuto = () =>
  useWrite<{ enabled: boolean }, { auto: boolean }>('portfolio.income_auto');
/** Records the credit of distributions recorded as received without a cash entry. */
export const useIncomeCash = () =>
  useWrite<{ ids: string[] }, { created: number; change_set_id: string | null }>(
    'portfolio.income_cash',
  );

/** Apagar em lote: um conjunto de alterações só (um "Desfazer" restaura todos). */
export const useDeleteTrades = () =>
  useWrite<{ ids: string[] }, BulkResult>('portfolio.delete_trades');
export const useDeleteIncomes = () =>
  useWrite<{ ids: string[] }, BulkResult>('portfolio.delete_incomes');
export const useDeleteActions = () =>
  useWrite<{ ids: string[] }, BulkResult>('portfolio.delete_actions');

export const useSetTargets = () =>
  useWrite<{ scope: TargetScope; targets: TargetInput[] }, Target[]>('portfolio.set_targets');

/** Limite de concentração (de um ativo ou os padrões); devolve o relatório recalculado. */
export const useSetConcentrationLimit = () =>
  useWrite<SetConcentrationLimitArgs, ConcentrationReport>('portfolio.set_concentration_limit');
export const useWatch = () => useWrite<WatchlistInput, WatchlistItem[]>('portfolio.watch');
export const useUnwatch = () =>
  useWrite<{ asset_id: string }, WatchlistItem[]>('portfolio.unwatch');
