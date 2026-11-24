/**
 * Página do ativo: posição (quantidade, preço médio, valor, resultado, XIRR nominal ×
 * real), termos de renda fixa, gráfico de preço com compras/vendas e preço médio
 * (Lightweight Charts), análise fundamentalista (indicadores, preço justo, histórico e
 * documentos, para ações, FIIs e ações americanas), score de saúde do ativo (aba Saúde),
 * operações, proventos e eventos corporativos.
 */
import { addMonthsToDate } from '@/lib/months';
import { useEffect, useMemo, useState, type FormEvent, type ReactNode } from 'react';
import { useSearchParams } from 'react-router';

import { isApiError } from '@/api/client';
import { useCountryFeatures } from '@/api/hooks/country';
import {
  useAsset,
  useDeleteAsset,
  usePriceHistory,
  useSetPrice,
  useUpdateAsset,
} from '@/api/hooks/assets';
import { useActions, useIncome, usePosition, useTrades, useWatchlist } from '@/api/hooks/portfolio';
import type { Asset } from '@/api/generated/Asset';
import type { Trade } from '@/api/generated/Trade';
import { ChartFrame } from '@/charts';
import {
  Button,
  copyOnlyMenus,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Figure,
  Icon,
  InstrumentCard,
  KeyValue,
  Measure,
  Money,
  OriginBadge,
  Private,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  TechTable,
  toast,
} from '@/design/components';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';
import { FormError } from '@/vault/FormError';
import { slot, withSlots } from '@/features/transactions/ui/slots';

import { ActionDialog } from '../portfolio/ActionDialog';
import { AskAgentButton, AssetConversations } from './AssetConversations';
import { AssetNews } from './AssetNews';
import { AssetOverview } from './AssetOverview';
import { AssetDataChip, AssetDataPanel } from './data/AssetDataPanel';
import { StudyPanel } from './study/StudyPanel';
import { PriceChart } from '../portfolio/charts/PriceChart';
import { DecimalInput } from '../portfolio/components/DecimalInput';
import { InflationMeter } from '../portfolio/components/InflationMeter';
import { Pct } from '../portfolio/components/Pct';
import { headlineReturn, RecentReturnNote } from '../portfolio/components/PositionReturn';
import { ResultCard } from '../portfolio/components/ResultCard';
import { QuoteCell } from '../portfolio/components/Quote';
import { Segmented } from '../portfolio/components/Segmented';
import { ConfirmButton, ErrorBlock, LoadingBlock, Notice } from '../portfolio/components/States';
import { formatQuantity, formatUnitPrice } from '../portfolio/decimal';
import { IncomeDialog } from '../portfolio/IncomeDialog';
import { AssetIncome } from '../portfolio/IncomeLedger';
import {
  FIXED_INCOME_KIND_LABEL,
  INDEXER_LABEL,
  isValueOnly,
  LIQUIDITY_LABEL,
  safeDate,
  sourceName,
  termsSummary,
} from '../portfolio/labels';
import { useDeflatorLabel } from '@/features/portfolio/useDeflatorLabel';
import { ActionsTable, TradesTable } from '../portfolio/RecordsPanels';
import { usePortfolioUi } from '../portfolio/state';
import { TradeDialog } from '../portfolio/TradeDialog';
import { meterScale, priceChartData } from '../portfolio/transform';
import { hasFundamentals } from '../research/labels';
import { ResearchSection } from '../research/ResearchSection';
import { AssetScorePanel } from '../score/AssetScorePanel';
import { assetSections } from './assetSections';
import { PensionPanel } from './PensionPanel';
import { FUNDAMENTUS_CLASSES, FundamentusPanel } from './FundamentusPanel';
import { IntradayChart } from '../market/IntradayChart';
import { AssetFormDialog } from './AssetFormDialog';
import { WatchDialog } from './Watchlist';
import { AssetStatement } from './AssetStatement';
import { TradeSummaryCards } from './TradeSummaryCards';

type Range = '6m' | '1a' | '2a' | 'max';

function rangeFrom(r: Range, today: string): string {
  const back = r === '6m' ? 6 : r === '1a' ? 12 : r === '2a' ? 24 : 600;
  return addMonthsToDate(today, -back);
}

function ManualPriceDialog({
  asset,
  open,
  onOpenChange,
}: {
  asset: Asset;
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="sm"
      index="07"
      title={t('ativos-cotacao-titulo', { nome: asset.ticker ?? asset.name })}
      description={t('ativos-cotacao-descricao')}
    >
      {open ? <ManualPriceForm asset={asset} onDone={() => onOpenChange(false)} /> : null}
    </Dialog>
  );
}

function ManualPriceForm({ asset, onDone }: { asset: Asset; onDone: () => void }) {
  const set = useSetPrice();
  const [date, setDate] = useState<string | null>(todayIso());
  const [close, setClose] = useState<string | null>(null);
  const [submitted, setSubmitted] = useState(false);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setSubmitted(true);
    if (!date || !close) return;
    set.mutate(
      { asset_id: asset.id, date, close },
      {
        onSuccess: () => {
          toast({
            tone: 'success',
            title: t('ativos-cotacao-informada'),
            description: t('ativos-cotacao-informada-em', {
              preco: formatUnitPrice(close, asset.currency),
              data: formatDate(date),
            }),
          });
          onDone();
        },
      },
    );
  };
  return (
    <form
      onSubmit={submit}
      noValidate
      className="flex flex-col gap-4"
      aria-label={t('ativos-cotacao-manual')}
    >
      <div className="grid grid-cols-2 gap-3">
        <Field
          label={t('ativos-data')}
          error={submitted && !date ? t('ativos-informe-data') : undefined}
        >
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field
          label={t('ativos-fechamento-moeda', { moeda: asset.currency })}
          error={submitted && !close ? t('ativos-informe-preco') : undefined}
        >
          <DecimalInput value={close} onValueChange={setClose} placeholder="0,00" />
        </Field>
      </div>
      <FormError error={set.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button type="submit" variant="primary" loading={set.isPending}>
          {t('ativos-salvar-cotacao')}
        </Button>
      </div>
    </form>
  );
}

function TermsCard({ asset }: { asset: Asset }) {
  const fi = asset.fixed_income;
  if (!fi) return null;
  return (
    <InstrumentCard
      index="05"
      title={t('ativos-termos-titulo')}
      footer={[{ label: t('ativos-termos-resumo'), value: termsSummary(fi) }]}
    >
      <KeyValue
        layout="stacked"
        items={[
          { label: t('ativos-termos-tipo'), value: FIXED_INCOME_KIND_LABEL[fi.kind] },
          { label: t('ativos-termos-indexador'), value: INDEXER_LABEL[fi.indexer] },
          { label: t('ativos-termos-taxa'), value: termsSummary({ ...fi, maturity_date: null }) },
          { label: t('ativos-termos-emissao'), value: safeDate(fi.issue_date) },
          { label: t('ativos-termos-vencimento'), value: safeDate(fi.maturity_date) },
          { label: t('ativos-termos-carencia'), value: safeDate(fi.grace_until) },
          { label: t('ativos-termos-liquidez'), value: LIQUIDITY_LABEL[fi.liquidity] },
          {
            label: t('ativos-termos-ir'),
            value: fi.ir_exempt ? t('ativos-termos-isento') : t('ativos-termos-regressiva'),
          },
          { label: t('ativos-termos-emissor'), value: fi.issuer ?? '—' },
        ]}
      />
    </InstrumentCard>
  );
}

type AssetDialog = 'trade' | 'income' | 'action' | 'watch' | 'price' | 'edit';

const DIALOG_PARAM: Record<string, AssetDialog> = {
  acompanhar: 'watch',
  cotacao: 'price',
  editar: 'edit',
};

export function AssetDetail({
  assetId,
  onBack,
  breadcrumb,
  notice,
}: {
  assetId: string;
  onBack: () => void;
  /** Trilha da página dedicada (`/ativo/<id>`, docs/asset-page.md) no lugar do "voltar". */
  breadcrumb?: ReactNode;
  /** Aviso logo abaixo do cabeçalho (ex.: "Observado pelo Mercado"). */
  notice?: ReactNode;
}) {
  const deflatorLabel = useDeflatorLabel();
  const deflator = usePortfolioUi((s) => s.deflator);
  const base = useBaseCurrency();
  const assetQ = useAsset(assetId);
  const posQ = usePosition({ asset_id: assetId, deflator });
  const tradesQ = useTrades({ asset_id: assetId });
  const actionsQ = useActions(assetId);
  const incomeQ = useIncome({ asset_id: assetId });
  const [range, setRange] = useState<Range>('1a');
  const today = todayIso();
  const pricesQ = usePriceHistory({ asset_id: assetId, from: rangeFrom(range, today), to: today });
  const watch = useWatchlist();
  const updateAsset = useUpdateAsset();
  const deleteAsset = useDeleteAsset();
  const [dialog, setDialog] = useState<AssetDialog | null>(null);
  // `?secao=` abre direto numa aba (ex.: links e o agente apontando os fundamentos).
  const [params, setParams] = useSearchParams();
  const [tab, setTab] = useState(() => params.get('secao') ?? 'resumo');
  // `?dialogo=` (menus de contexto de outras telas) abre um dos diálogos da página e sai da URL.
  const wanted = DIALOG_PARAM[params.get('dialogo') ?? ''] ?? null;
  const [lastWanted, setLastWanted] = useState<AssetDialog | null>(null);
  if (wanted !== lastWanted) {
    setLastWanted(wanted);
    if (wanted) setDialog(wanted);
  }
  useEffect(() => {
    if (!params.has('dialogo')) return;
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        next.delete('dialogo');
        return next;
      },
      { replace: true },
    );
  }, [params, setParams]);

  const trades = useMemo(() => tradesQ.data ?? [], [tradesQ.data]);
  const actions = useMemo(() => actionsQ.data ?? [], [actionsQ.data]);
  const prices = useMemo(() => pricesQ.data ?? [], [pricesQ.data]);
  // Operações e eventos dentro da janela do gráfico (o preço médio vem do início).
  const chartData = useMemo(
    () => priceChartData(prices, trades, actions),
    [prices, trades, actions],
  );

  // Fundamentus e proventos anunciados da B3: só no Brasil (docs/countries.md).
  const brMarket = useCountryFeatures().br_market;
  if (assetQ.isPending) return <LoadingBlock label={t('ativos-carregando-ativo')} />;
  if (assetQ.isError) {
    return (
      <div className="flex flex-col gap-3">
        {breadcrumb ?? <BackLink onBack={onBack} />}
        <ErrorBlock error={assetQ.error} title={t('ativos-nao-encontrado')} />
      </div>
    );
  }
  const asset = assetQ.data;
  const fundamentus = brMarket && (FUNDAMENTUS_CLASSES as readonly string[]).includes(asset.class);
  const p = posQ.data;
  const open = p && p.quantity !== '0';
  const watched = watch.data?.find((w) => w.asset.id === asset.id) ?? null;
  const lastPrice = prices[prices.length - 1];
  const pricesMissing = !pricesQ.isPending && prices.length === 0;
  const pricesUnsupported =
    pricesQ.isError && isApiError(pricesQ.error) && pricesQ.error.code === 'unknown_command';
  // Abas e botões conforme o tipo do ativo (assetSections.ts); `?secao=` de uma aba que não
  // existe para o tipo cai no Resumo.
  const sections = assetSections(asset.class, {
    income: (incomeQ.data?.length ?? 0) > 0,
    actions: actions.length > 0,
  });
  const hidden =
    (tab === 'proventos' && !sections.income) ||
    (tab === 'eventos' && !sections.actions) ||
    (tab === 'previdencia' && asset.class !== 'pension');
  const shownTab = hidden ? 'resumo' : tab;
  const showFundamentals = (hasFundamentals(asset.class) || fundamentus) && Boolean(asset.ticker);

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-3 border-b border-line pb-5">
        {breadcrumb ?? <BackLink onBack={onBack} />}
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div className="flex min-w-0 flex-col gap-1.5">
            <span className="label-caps">
              {asset.class_label} ·{' '}
              {asset.exchange?.name ?? asset.market ?? t('ativos-sem-mercado')} · {asset.currency}
              {asset.archived ? ` · ${t('ativos-arquivado')}` : ''}
            </span>
            <h2 className="flex items-baseline gap-3 text-[22px] leading-tight font-semibold text-fg">
              <span className="num tracking-[0.04em]">{asset.ticker ?? asset.name}</span>
              {asset.ticker ? (
                <span className="truncate text-[15px] font-normal text-dim">{asset.name}</span>
              ) : null}
            </h2>
            {asset.official_name ? (
              <span className="text-[12.5px] text-dim">
                {t('ativos-nome-oficial-de', { nome: asset.official_name })}
              </span>
            ) : null}
            {asset.sector || asset.segment ? (
              <span className="text-[12.5px] text-dim">
                {[asset.sector, asset.segment].filter(Boolean).join(' · ')}
              </span>
            ) : null}
            <AssetDataChip assetId={asset.id} onOpen={() => setTab('cadastro')} />
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <AskAgentButton asset={asset} />
            <Button variant="ghost" onClick={() => setDialog('edit')}>
              {t('comum-editar')}
            </Button>
            <Button variant="ghost" onClick={() => setDialog('price')}>
              {t('ativos-informar-cotacao')}
            </Button>
            <Button
              variant="ghost"
              onClick={() => setDialog('watch')}
              icon={<Icon name={watched ? 'eye' : 'plus'} size={12} />}
            >
              {watched ? t('ativos-aba-watchlist') : t('ativos-acompanhar')}
            </Button>
            {sections.actions ? (
              <Button variant="ghost" onClick={() => setDialog('action')}>
                {t('ativos-evento')}
              </Button>
            ) : null}
            {sections.income ? (
              <Button onClick={() => setDialog('income')}>{t('ativos-provento')}</Button>
            ) : null}
            <Button
              variant="primary"
              icon={<Icon name="plus" size={12} />}
              onClick={() => setDialog('trade')}
            >
              {t('ativos-menu-nova-operacao')}
            </Button>
          </div>
        </div>
      </div>

      {notice}
      {posQ.isError ? <ErrorBlock error={posQ.error} onRetry={() => void posQ.refetch()} /> : null}
      <Tabs value={shownTab} onValueChange={setTab}>
        <TabsList aria-label={t('ativos-secoes-ativo')}>
          <TabsTrigger value="resumo">{t('ativos-aba-resumo')}</TabsTrigger>
          {asset.class === 'pension' ? (
            <TabsTrigger value="previdencia">{t('ativos-aba-previdencia')}</TabsTrigger>
          ) : null}
          <TabsTrigger value="extrato">{t('ativos-aba-extrato')}</TabsTrigger>
          <TabsTrigger value="operacoes">{t('ativos-menu-operacoes')}</TabsTrigger>
          {sections.income ? (
            <TabsTrigger value="proventos">{t('ativos-menu-proventos')}</TabsTrigger>
          ) : null}
          {sections.actions ? (
            <TabsTrigger value="eventos">{t('ativos-aba-eventos')}</TabsTrigger>
          ) : null}
          <TabsTrigger value="grafico">{t('ativos-aba-grafico')}</TabsTrigger>
          {showFundamentals ? (
            <TabsTrigger value="fundamentos">{t('ativos-menu-fundamentos')}</TabsTrigger>
          ) : null}
          <TabsTrigger value="noticias">{t('ativo-aba-noticias')}</TabsTrigger>
          <TabsTrigger value="saude">{t('ativos-aba-saude')}</TabsTrigger>
          <TabsTrigger value="estudo">{t('estudo-titulo')}</TabsTrigger>
          <TabsTrigger value="conversas">{t('ativo-aba-conversas')}</TabsTrigger>
          <TabsTrigger value="cadastro">{t('ativos-aba-cadastro')}</TabsTrigger>
        </TabsList>
        <TabsContent value="resumo">
          <div className="flex flex-col gap-5">
            <AssetOverview
              subject={{ asset_id: asset.id }}
              {...(showFundamentals ? { onFundamentals: () => setTab('fundamentos') } : {})}
              onNews={() => setTab('noticias')}
            />
            {p && open ? (
              <div className="grid gap-4 @min-[530px]:grid-cols-2 @min-[1040px]:grid-cols-4">
                <InstrumentCard
                  index="01"
                  title={t('ativos-posicao')}
                  footer={[
                    { label: t('ativos-desde'), value: safeDate(p.first_date) },
                    { label: t('ativos-custo'), value: <Money cents={p.cost_base} /> },
                  ]}
                >
                  {isValueOnly(asset.class) ? (
                    // Renda fixa: o que a pessoa aplicou (o que ainda está investido), sem cotas.
                    <Measure
                      size="md"
                      label={t('ativos-aplicado')}
                      value={<Money cents={p.cost} currency={asset.currency} />}
                      changeKey={String(p.cost)}
                    />
                  ) : (
                    <>
                      <Measure
                        size="md"
                        label={t('ativos-quantidade')}
                        value={<Private>{formatQuantity(p.quantity)}</Private>}
                        changeKey={p.quantity}
                      />
                      <p className="mt-2 text-[12.5px] text-dim">
                        {t('ativos-preco-medio')}{' '}
                        <Private>
                          <span className="num text-fg">
                            {p.avg_price
                              ? formatUnitPrice(p.avg_price, asset.currency, { maxDecimals: 4 })
                              : '—'}
                          </span>
                        </Private>
                      </p>
                    </>
                  )}
                </InstrumentCard>
                <InstrumentCard
                  index="02"
                  title={t('ativos-valor')}
                  footer={
                    p.valuation === 'curve' && p.net_redemption_base != null
                      ? [
                          {
                            label: t('ativos-liquido-estimado'),
                            value: <Money cents={p.net_redemption_base} estimated />,
                          },
                        ]
                      : [
                          {
                            label: t('ativos-col-cotacao'),
                            value: (
                              <Private>
                                <QuoteCell position={p} />
                              </Private>
                            ),
                          },
                        ]
                  }
                >
                  <Measure
                    size="md"
                    label={
                      p.valuation === 'curve'
                        ? t('ativos-bruto-curva')
                        : p.valuation === 'missing'
                          ? t('ativos-pelo-custo')
                          : t('ativos-valor-mercado')
                    }
                    value={
                      <Money
                        cents={p.market_value_base ?? p.cost_base}
                        estimated={p.estimated || p.valuation === 'missing'}
                      />
                    }
                    changeKey={p.market_value_base ?? 0}
                    hint={
                      p.weight != null
                        ? withSlots(t('ativos-peso', { peso: slot(0) }), [
                            <Pct key="peso" value={p.weight} />,
                          ])
                        : undefined
                    }
                  />
                  {asset.currency !== base && p.market_value != null ? (
                    <p className="mt-2 text-[12px] text-dim">
                      {withSlots(t('ativos-na-moeda-do-ativo', { valor: slot(0) }), [
                        <Money key="valor" cents={p.market_value} currency={asset.currency} />,
                      ])}
                    </p>
                  ) : null}
                </InstrumentCard>
                <ResultCard
                  index="03"
                  title={t('ativos-resultado')}
                  deflator={deflator}
                  nominal={p.total_result_base}
                  real={p.real_result_base}
                  effect={p.inflation_effect_base}
                  cost={p.cost_base}
                  estimated={p.estimated}
                  parts={[
                    {
                      label: t('ativos-nao-realizado'),
                      value:
                        p.unrealized_base == null ? (
                          '—'
                        ) : (
                          <Money cents={p.unrealized_base} tone="auto" />
                        ),
                    },
                    {
                      label: t('ativos-realizado'),
                      value: <Money cents={p.realized_base} tone="auto" />,
                    },
                    {
                      label: t('ativos-menu-proventos'),
                      value: <Money cents={p.income_base} tone="auto" />,
                    },
                  ]}
                />
                <InstrumentCard
                  index="04"
                  // Posição recente: o retorno do período no lugar do anual (que exagera).
                  title={
                    p.return_basis === 'annualized'
                      ? t('ativos-retorno-anual')
                      : t('carteira-recente-titulo')
                  }
                  footer={[{ label: t('ativos-real-por'), value: deflatorLabel(deflator) }]}
                >
                  <div className="flex flex-col gap-3">
                    <div className="flex gap-6">
                      <Measure
                        size="md"
                        label={t('ativos-nominal')}
                        value={
                          <Pct
                            value={headlineReturn(p)}
                            signed
                            tone={p.return_basis === 'annualized' ? 'neutral' : 'auto'}
                          />
                        }
                        changeKey={headlineReturn(p) ?? 0}
                      />
                      {deflator !== 'none' ? (
                        <Measure
                          size="md"
                          label={t('ativos-real')}
                          value={
                            <Pct
                              value={headlineReturn(p, true)}
                              signed
                              estimated={p.estimated}
                              tone={p.return_basis === 'annualized' ? 'neutral' : 'auto'}
                            />
                          }
                          changeKey={headlineReturn(p, true) ?? 0}
                        />
                      ) : null}
                    </div>
                    <InflationMeter
                      nominal={headlineReturn(p)}
                      real={deflator === 'none' ? headlineReturn(p) : headlineReturn(p, true)}
                      scale={meterScale([headlineReturn(p), headlineReturn(p, true)])}
                      deflator={deflatorLabel(deflator)}
                      showValues={false}
                    />
                    <RecentReturnNote p={p} withReal={deflator !== 'none'} />
                  </div>
                </InstrumentCard>
              </div>
            ) : p && !posQ.isPending ? (
              <Notice
                title={trades.length ? t('ativos-posicao-encerrada') : t('ativos-sem-posicao')}
              >
                {trades.length ? t('ativos-encerrada-texto') : t('ativos-sem-posicao-texto')}
                {p.realized_base || p.income_base ? (
                  <span className="ml-2">
                    {withSlots(
                      t('ativos-realizado-proventos', { realizado: slot(0), proventos: slot(1) }),
                      [
                        <Money key="r" cents={p.realized_base} tone="auto" />,
                        <Money key="p" cents={p.income_base} />,
                      ],
                    )}
                  </span>
                ) : null}
              </Notice>
            ) : null}
            {p?.warnings.length ? (
              <Notice tone="warning" title={t('ativos-avisos-calculo')}>
                <ul className="list-inside list-disc">
                  {p.warnings.map((w) => (
                    <li key={w}>{w}</li>
                  ))}
                </ul>
              </Notice>
            ) : null}

            {asset.quote_symbol ? (
              <Figure
                number="00"
                title={t('ativos-hoje', { simbolo: asset.quote_symbol })}
                source={t('ativos-hoje-fonte')}
              >
                <IntradayChart asset={asset} />
              </Figure>
            ) : null}

            {asset.fixed_income ? <TermsCard asset={asset} /> : null}
          </div>
        </TabsContent>
        {asset.class === 'pension' ? (
          <TabsContent value="previdencia">
            <PensionPanel asset={asset} />
          </TabsContent>
        ) : null}
        <TabsContent value="extrato">
          <AssetStatement assetId={asset.id} currency={asset.currency} assetClass={asset.class} />
        </TabsContent>
        <TabsContent value="operacoes">
          <div className="flex flex-col gap-5">
            <TradeSummaryCards asset={asset} position={p} />
            <TradesTable assetId={asset.id} compact />
          </div>
        </TabsContent>
        <TabsContent value="proventos">
          <AssetIncome assetId={asset.id} fundamentus={fundamentus && Boolean(asset.ticker)} />
        </TabsContent>
        <TabsContent value="eventos">
          <div className="flex flex-col gap-3">
            <p className="text-[12px] text-dim">{t('ativos-eventos-ajuda')}</p>
            <ActionsTable assetId={asset.id} />
          </div>
        </TabsContent>
        <TabsContent value="grafico">
          <div
            className={
              asset.fixed_income
                ? 'grid gap-5 @min-[1040px]:grid-cols-[minmax(0,8fr)_minmax(0,4fr)]'
                : ''
            }
          >
            <ChartFrame
              index="01"
              title={t('ativos-grafico-titulo')}
              actions={
                <Segmented<Range>
                  size="sm"
                  label={t('ativos-grafico-periodo')}
                  value={range}
                  onChange={setRange}
                  options={[
                    { value: '6m', label: t('ativos-periodo-6m') },
                    { value: '1a', label: t('ativos-periodo-1a') },
                    { value: '2a', label: t('ativos-periodo-2a') },
                    { value: 'max', label: t('ativos-periodo-max') },
                  ]}
                />
              }
              source={
                lastPrice ? (
                  <span className="inline-flex items-center gap-1.5">
                    {t('ativos-grafico-fonte', { fonte: sourceName(lastPrice.source) })}
                    <OriginBadge source={sourceName(lastPrice.source)} date={lastPrice.date} />
                  </span>
                ) : (
                  t('ativos-grafico-sem-cotacoes')
                )
              }
              chart={
                pricesQ.isPending ? (
                  <LoadingBlock label={t('ativos-carregando-cotacoes')} />
                ) : (
                  <div className="flex flex-col gap-3">
                    {pricesMissing || pricesUnsupported ? (
                      <Notice tone="warning" title={t('ativos-sem-historico')}>
                        {asset.class === 'fixed_income'
                          ? t('ativos-sem-historico-renda-fixa')
                          : t('ativos-sem-historico-texto')}
                      </Notice>
                    ) : null}
                    {chartData.line.length ? (
                      <PriceChart
                        prices={prices}
                        trades={trades}
                        actions={actions}
                        currency={asset.currency}
                        ariaLabel={t('ativos-grafico-aria', {
                          nome: asset.ticker ?? asset.name,
                          n: trades.length,
                        })}
                      />
                    ) : null}
                    <div className="flex flex-wrap gap-4 text-[11px]">
                      <span className="label-caps inline-flex items-center gap-1.5">
                        <span aria-hidden className="inline-block h-[2px] w-3 bg-accent" />
                        {chartData.hasPrices ? t('ativos-fechamento') : t('ativos-preco-operacoes')}
                      </span>
                      <span className="label-caps inline-flex items-center gap-1.5">
                        <span
                          aria-hidden
                          className="inline-block h-[2px] w-3 border-t-2 border-dashed border-warning"
                        />
                        {t('ativos-preco-medio')}
                      </span>
                      <span className="label-caps inline-flex items-center gap-1.5 text-positive">
                        {t('ativos-compra')}
                      </span>
                      <span className="label-caps inline-flex items-center gap-1.5 text-negative">
                        {t('ativos-venda')}
                      </span>
                    </div>
                  </div>
                )
              }
              table={
                <TechTable
                  label={t('ativos-tabela-operacoes')}
                  prefsId="Operações e preço médio"
                  dense
                  rows={[...trades].reverse()}
                  getRowId={(tr) => tr.id}
                  {...copyOnlyMenus<Trade>()}
                  columns={[
                    { key: 'date', header: t('ativos-data'), cell: (tr) => formatDate(tr.date) },
                    {
                      key: 'side',
                      header: t('ativos-termos-tipo'),
                      cell: (tr) => (tr.side === 'buy' ? t('ativos-compra') : t('ativos-venda')),
                    },
                    ...(isValueOnly(asset.class)
                      ? [
                          {
                            key: 'gross',
                            header: t('ativos-valor'),
                            numeric: true,
                            cell: (tr: Trade) => <Money cents={tr.gross} currency={tr.currency} />,
                          },
                        ]
                      : [
                          {
                            key: 'qty',
                            header: t('ativos-qtd'),
                            numeric: true,
                            cell: (tr: Trade) => <Private>{formatQuantity(tr.quantity)}</Private>,
                          },
                          {
                            key: 'price',
                            header: t('ativos-preco'),
                            numeric: true,
                            cell: (tr: Trade) => (
                              <Private>{formatUnitPrice(tr.price, tr.currency)}</Private>
                            ),
                          },
                          {
                            key: 'avg',
                            header: t('ativos-preco-medio-depois'),
                            numeric: true,
                            cell: (tr: Trade) => {
                              const v = [...chartData.avg]
                                .reverse()
                                .find((a) => a.time <= tr.date)?.value;
                              return (
                                <Private>
                                  {v == null
                                    ? '—'
                                    : formatUnitPrice(String(Number(v.toFixed(6))), tr.currency, {
                                        maxDecimals: 4,
                                      })}
                                </Private>
                              );
                            },
                          },
                        ]),
                  ]}
                />
              }
            />
          </div>
        </TabsContent>
        <TabsContent value="fundamentos">
          <div className="flex flex-col gap-5">
            {fundamentus && asset.ticker ? <FundamentusPanel ticker={asset.ticker} /> : null}
            {hasFundamentals(asset.class) && asset.ticker ? (
              <ResearchSection
                subject={{ asset_id: asset.id }}
                title={t('ativos-analise-fundamentalista')}
              />
            ) : null}
          </div>
        </TabsContent>
        <TabsContent value="saude">
          <AssetScorePanel subject={{ asset_id: asset.id }} />
        </TabsContent>
        <TabsContent value="estudo">
          <StudyPanel assetId={asset.id} />
        </TabsContent>
        <TabsContent value="noticias">
          <AssetNews subject={{ asset_id: asset.id }} />
        </TabsContent>
        <TabsContent value="conversas">
          <AssetConversations asset={asset} onStudies={() => setTab('estudo')} />
        </TabsContent>
        <TabsContent value="cadastro">
          <AssetDataPanel assetId={asset.id} />
          <div className="mt-5 flex flex-wrap items-center gap-2 border-t border-line pt-4">
            <span className="label-caps">{t('ativos-aba-cadastro')}</span>
            <Button
              size="sm"
              variant="ghost"
              loading={updateAsset.isPending}
              onClick={() =>
                updateAsset.mutate(
                  { id: asset.id, archived: !asset.archived },
                  {
                    onSuccess: () =>
                      toast({
                        tone: 'success',
                        title: asset.archived ? t('ativos-reativado') : t('ativos-arquivado-toast'),
                      }),
                  },
                )
              }
            >
              {asset.archived ? t('ativos-desarquivar') : t('ativos-arquivar')}
            </Button>
            {trades.length === 0 ? (
              <ConfirmButton
                label={t('ativos-excluir')}
                title={t('ativos-excluir')}
                description={t('ativos-excluir-descricao', { nome: asset.ticker ?? asset.name })}
                pending={deleteAsset.isPending}
                onConfirm={() =>
                  deleteAsset.mutateAsync({ id: asset.id }).then(
                    () => {
                      toast({ tone: 'success', title: t('ativos-excluido') });
                      onBack();
                    },
                    (e: Error) =>
                      toast({
                        tone: 'error',
                        title: t('ativos-nao-excluiu'),
                        description: e.message,
                      }),
                  )
                }
              />
            ) : (
              <span className="text-[12px] text-dim">{t('ativos-so-arquivar')}</span>
            )}
          </div>
        </TabsContent>
      </Tabs>

      <TradeDialog
        open={dialog === 'trade'}
        onOpenChange={(o) => !o && setDialog(null)}
        assetId={asset.id}
      />
      <IncomeDialog
        open={dialog === 'income'}
        onOpenChange={(o) => !o && setDialog(null)}
        assetId={asset.id}
      />
      <ActionDialog
        open={dialog === 'action'}
        onOpenChange={(o) => !o && setDialog(null)}
        assetId={asset.id}
      />
      <WatchDialog
        open={dialog === 'watch'}
        onOpenChange={(o) => !o && setDialog(null)}
        item={watched}
        assetId={asset.id}
      />
      <ManualPriceDialog
        asset={asset}
        open={dialog === 'price'}
        onOpenChange={(o) => !o && setDialog(null)}
      />
      <AssetFormDialog
        open={dialog === 'edit'}
        onOpenChange={(o) => !o && setDialog(null)}
        asset={asset}
      />
    </div>
  );
}

function BackLink({ onBack }: { onBack: () => void }) {
  return (
    <button
      type="button"
      onClick={onBack}
      className="label-caps inline-flex w-fit items-center gap-1 hover:text-fg"
    >
      <Icon name="chevronLeft" size={12} /> {t('casca-tela-ativos')}
    </button>
  );
}
