/**
 * Posições em tabela técnica, agrupadas por classe, por conta ou sem grupo: ticker,
 * classe, quantidade, preço médio, cotação com origem, valor (bruto e líquido
 * estimado na renda fixa), peso, resultado, XIRR e o medidor nominal × real. Posição recente
 * (menos de 30 dias) mostra o retorno do período no lugar do anualizado; no topo, o card de
 * concentração com os ativos e grupos acima do limite (o peso ganha cor na tabela).
 */
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router';

import { useAssets } from '@/api/hooks/assets';
import { useQuoteMap } from '@/api/hooks/market';
import type { ConcentrationAlert } from '@/api/generated/ConcentrationAlert';
import type { MarketQuote } from '@/api/generated/MarketQuote';
import { useInvestmentAccounts, usePortfolioSummary } from '@/api/hooks/portfolio';
import type { PositionView } from '@/api/generated/PositionView';
import {
  Button,
  EmptyState,
  Icon,
  Money,
  Private,
  TechTable,
  Tooltip,
  type TechColumn,
} from '@/design/components';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';

import { assetMenuItems } from '../assets/assetMenu';
import { QuoteChange } from '../market/QuoteBits';
import { Check } from './components/Fields';
import { ConcentrationCard, ConcentrationLimitsDialog, WeightPill } from './Concentration';
import { InflationMeter } from './components/InflationMeter';
import { Pct } from './components/Pct';
import { headlineReturn, PositionReturnCell } from './components/PositionReturn';
import { QuoteCell } from './components/Quote';
import { Segmented } from './components/Segmented';
import { ErrorBlock, LoadingBlock } from './components/States';
import { formatQuantity, formatUnitPrice } from './decimal';
import { CLASS_LABEL, classLabel, isValueOnly, termsSummary } from './labels';
import { useDeflatorLabel } from './useDeflatorLabel';
import { usePortfolioUi } from './state';
import { groupPositions, meterScale, type GroupMode, type PositionGroup } from './transform';

function AssetCell({ p, terms }: { p: PositionView; terms?: string | null }) {
  return (
    <div className="flex min-w-0 flex-col leading-tight">
      <span className="flex min-w-0 items-center gap-1.5 font-medium text-fg">
        <span className="truncate">{p.asset.ticker ?? p.asset.name}</span>
        {p.warnings.length ? (
          <Tooltip
            content={
              <ul className="flex max-w-64 flex-col gap-1">
                {p.warnings.map((w) => (
                  <li key={w}>{w}</li>
                ))}
              </ul>
            }
          >
            <button
              type="button"
              aria-label={t('carteira-avisos', { lista: p.warnings.join('; ') })}
              className="text-warning"
            >
              <Icon name="alert" size={12} />
            </button>
          </Tooltip>
        ) : null}
      </span>
      <span className="truncate font-sans text-[11.5px] text-dim">
        {terms ?? (p.asset.ticker ? p.asset.name : CLASS_LABEL[p.asset.class])}
      </span>
    </div>
  );
}

function ValueCell({ p }: { p: PositionView }) {
  const base = useBaseCurrency();
  const value = p.market_value_base ?? p.cost_base;
  return (
    <div className="flex flex-col items-end leading-tight">
      <Money cents={value} estimated={p.estimated || p.valuation === 'missing'} />
      {p.valuation === 'curve' && p.net_redemption_base != null ? (
        <Tooltip content={t('carteira-liquido-dica')}>
          <button
            type="button"
            className="text-right text-[11px] whitespace-normal text-dim italic"
          >
            {withSlots(t('carteira-liquido-abrev', { valor: slot(0) }), [
              <Money key="v" cents={p.net_redemption_base} estimated />,
            ])}
          </button>
        </Tooltip>
      ) : p.asset.currency !== base && p.market_value != null ? (
        <span className="text-[11px] text-dim">
          <Money cents={p.market_value} currency={p.asset.currency} />
        </span>
      ) : null}
    </div>
  );
}

function columns(
  scale: number,
  deflator: string,
  termsOf: (id: string) => string | null,
  showClass: boolean,
  quotes: Map<string, MarketQuote>,
  alerts: Map<string, ConcentrationAlert>,
): TechColumn<PositionView>[] {
  const cols: TechColumn<PositionView>[] = [
    {
      key: 'asset',
      header: t('carteira-col-ativo'),
      width: showClass ? '14%' : '21%',
      cell: (p) => <AssetCell p={p} terms={termsOf(p.asset.id)} />,
      sortValue: (p) => p.asset.ticker ?? p.asset.name,
    },
    {
      key: 'class',
      header: t('carteira-col-classe'),
      width: '7%',
      cell: (p) => (
        <span className="font-sans text-dim">{classLabel(p.asset.class, p.asset.class_label)}</span>
      ),
      sortValue: (p) => classLabel(p.asset.class, p.asset.class_label),
    },
    {
      key: 'qty',
      header: t('carteira-col-qtd'),
      width: '7%',
      numeric: true,
      // Renda fixa não tem quantidade para a pessoa (só valores): fica em branco.
      cell: (p) =>
        isValueOnly(p.asset.class) ? '—' : <Private>{formatQuantity(p.quantity)}</Private>,
      sortValue: (p) => (isValueOnly(p.asset.class) ? -1 : Number(p.quantity)),
    },
    {
      key: 'avg',
      header: t('carteira-col-preco-medio'),
      width: '11%',
      numeric: true,
      cell: (p) => (
        <Private>
          {/* Sem espaço, corta com reticências à direita (a dica mostra o preço inteiro). */}
          <span className="block truncate">
            {p.avg_price && !isValueOnly(p.asset.class)
              ? formatUnitPrice(p.avg_price, p.asset.currency, { maxDecimals: 4 })
              : '—'}
          </span>
        </Private>
      ),
    },
    {
      key: 'quote',
      header: t('carteira-col-cotacao'),
      width: '12%',
      numeric: true,
      cell: (p) => {
        const q = quotes.get(p.asset.id);
        return (
          <span className="flex flex-col items-end leading-tight">
            <Private>
              <QuoteCell position={p} />
            </Private>
            {q ? (
              <QuoteChange
                quote={q}
                showPrice={false}
                className="max-w-full flex-wrap gap-y-0.5 text-[11px]"
              />
            ) : null}
          </span>
        );
      },
    },
    {
      key: 'value',
      header: t('carteira-col-valor'),
      width: '11%',
      numeric: true,
      cell: (p) => <ValueCell p={p} />,
      sortValue: (p) => p.market_value_base ?? p.cost_base,
    },
    {
      key: 'weight',
      header: t('carteira-col-peso'),
      width: '6%',
      numeric: true,
      cell: (p) => {
        const alert = alerts.get(p.asset.id);
        return alert ? <WeightPill alert={alert} weight={p.weight} /> : <Pct value={p.weight} />;
      },
      sortValue: (p) => p.weight,
    },
    {
      key: 'result',
      header: t('carteira-col-resultado'),
      width: '11%',
      numeric: true,
      cell: (p) =>
        p.total_result_base == null ? (
          '—'
        ) : (
          <div className="flex flex-col items-end leading-tight">
            <Money cents={p.total_result_base} tone="auto" />
            {p.cost_base > 0 ? (
              <Pct value={p.total_result_base / p.cost_base} tone="auto" className="text-[11px]" />
            ) : null}
          </div>
        ),
      sortValue: (p) => p.total_result_base,
    },
    {
      key: 'xirr',
      header: t('carteira-col-xirr'),
      width: '11%',
      numeric: true,
      // Posição recente (sem anualizado) fica no fim, nos dois sentidos.
      cell: (p) => <PositionReturnCell p={p} />,
      sortValue: (p) => p.xirr,
    },
    {
      key: 'meter',
      header: t('carteira-col-nominal-real'),
      width: '10%',
      // Mesma regra da coluna anterior: na posição recente, o retorno do período.
      cell: (p) => (
        <InflationMeter
          nominal={headlineReturn(p)}
          real={headlineReturn(p, true)}
          scale={scale}
          deflator={deflator}
        />
      ),
      sortValue: (p) => p.xirr_real,
    },
  ];
  return showClass ? cols : cols.filter((c) => c.key !== 'class');
}

function GroupTable({
  group,
  cols,
  onOpen,
  go,
}: {
  group: PositionGroup;
  cols: TechColumn<PositionView>[];
  onOpen: (p: PositionView) => void;
  go: (path: string) => void;
}) {
  return (
    <section aria-label={group.label} className="flex flex-col gap-2">
      <header className="flex flex-wrap items-baseline gap-x-4 gap-y-1 px-1">
        <h3 className="label-caps text-fg">{group.label}</h3>
        <span className="label-caps">{t('carteira-n-ativos', { n: group.positions.length })}</span>
        <span className="ml-auto flex items-baseline gap-4 text-[12.5px]">
          <Money cents={group.value} estimated={group.estimated} />
          <Pct value={group.weight} className="text-dim" />
          {group.result != null ? <Money cents={group.result} tone="auto" /> : null}
        </span>
      </header>
      <div className="rounded-card border border-line bg-surface">
        <TechTable
          // Larguras em %: abaixo disso as colunas se sobrepõem; a tabela rola na horizontal. Cada
          // célula recorta o que passar da largura dela (o texto de baixo quebra antes disso).
          className="[&_table]:table-fixed [&_td]:overflow-hidden"
          minWidth="1080px"
          label={t('carteira-posicoes-de', { grupo: group.label })}
          prefsId={group.prefsId}
          rows={group.positions}
          columns={cols}
          getRowId={(p) => `${p.asset.id}:${p.account_id ?? 'todas'}`}
          onActivate={onOpen}
          defaultSort={{ key: 'value', direction: 'desc' }}
          rowMenuLabel={(p) => t('carteira-acoes-de', { nome: p.asset.ticker ?? p.asset.name })}
          rowMenu={(p) => assetMenuItems(p.asset, go)}
        />
      </div>
    </section>
  );
}

export function PositionsPanel() {
  const deflatorLabel = useDeflatorLabel();
  const navigate = useNavigate();
  const deflator = usePortfolioUi((s) => s.deflator);
  const groupMode = usePortfolioUi((s) => s.groupMode);
  const setGroupMode = usePortfolioUi((s) => s.setGroupMode);
  const includeClosed = usePortfolioUi((s) => s.includeClosed);
  const setIncludeClosed = usePortfolioUi((s) => s.setIncludeClosed);
  const q = usePortfolioSummary({
    deflator,
    ...(groupMode === 'account' ? { by_account: true } : {}),
    ...(includeClosed ? { include_closed: true } : {}),
  });
  const accounts = useInvestmentAccounts();
  const assets = useAssets({ held_only: true, include_archived: true });
  const termsOf = useMemo(() => {
    const map = new Map(
      (assets.data ?? []).map((a) => [a.id, a.fixed_income ? termsSummary(a.fixed_income) : null]),
    );
    return (id: string) => map.get(id) ?? null;
  }, [assets.data]);
  const positions = useMemo(() => q.data?.positions ?? [], [q.data]);
  const groups = useMemo(
    () => groupPositions(positions, groupMode, accounts.data ?? []),
    [positions, groupMode, accounts.data],
  );
  const scale = useMemo(
    () => meterScale(positions.flatMap((p) => [headlineReturn(p), headlineReturn(p, true)])),
    [positions],
  );
  const quotes = useQuoteMap();
  const concentration = useMemo(() => q.data?.concentration_alerts ?? [], [q.data]);
  const assetAlerts = useMemo(
    () => new Map(concentration.filter((a) => a.kind === 'asset').map((a) => [a.ref, a])),
    [concentration],
  );
  const [limitsOpen, setLimitsOpen] = useState(false);
  const cols = useMemo(
    () =>
      columns(scale, deflatorLabel(deflator), termsOf, groupMode !== 'class', quotes, assetAlerts),
    [scale, deflator, deflatorLabel, termsOf, groupMode, quotes, assetAlerts],
  );
  const open = (p: PositionView) =>
    void navigate(`/ativos?ativo=${encodeURIComponent(p.asset.id)}`);

  return (
    <div className="flex flex-col gap-4">
      {concentration.length ? (
        <ConcentrationCard alerts={concentration} onAdjust={() => setLimitsOpen(true)} />
      ) : null}
      <ConcentrationLimitsDialog open={limitsOpen} onOpenChange={setLimitsOpen} />
      <div className="flex flex-wrap items-center gap-4">
        <Segmented<GroupMode>
          label={t('carteira-agrupar')}
          value={groupMode}
          onChange={setGroupMode}
          options={[
            { value: 'class', label: t('carteira-por-classe') },
            { value: 'account', label: t('carteira-por-conta') },
            { value: 'none', label: t('carteira-sem-grupo') },
          ]}
        />
        <Check
          checked={includeClosed}
          onChange={setIncludeClosed}
          label={t('carteira-incluir-encerradas')}
        />
        <Button
          size="sm"
          variant="ghost"
          icon={<Icon name="sliders" size={12} />}
          onClick={() => setLimitsOpen(true)}
        >
          {t('carteira-conc-botao')}
        </Button>
        <span className="ml-auto text-[12px] text-dim">{t('carteira-posicoes-atalhos')}</span>
      </div>
      {q.isPending ? (
        <LoadingBlock label={t('carteira-calculando-posicoes')} />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : groups.length === 0 ? (
        <EmptyState
          title={t('carteira-sem-posicoes')}
          description={t('carteira-sem-posicoes-texto')}
        />
      ) : (
        groups.map((g) => (
          <GroupTable
            key={g.key}
            group={g}
            cols={cols}
            onOpen={open}
            go={(path) => void navigate(path)}
          />
        ))
      )}
    </div>
  );
}
