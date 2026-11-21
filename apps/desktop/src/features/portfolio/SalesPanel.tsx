/**
 * Vendas (docs/sales-real-return.md): cada venda com o resultado nominal e o real, com o
 * custo trazido para a data da venda pelo deflator escolhido (IPCA, IGP-M, inflação do país,
 * US CPI ou M2). O topo soma o período; o gráfico compara nominal e real mês a mês.
 */
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router';

import { call } from '@/api/client';
import type { CostMethod } from '@/api/generated/CostMethod';
import type { SaleRow } from '@/api/generated/SaleRow';
import type { Trade } from '@/api/generated/Trade';
import type { TradeSide } from '@/api/generated/TradeSide';
import { useDeleteTrades, useSales } from '@/api/hooks/portfolio';
import { GroupedBars } from '@/charts/GroupedBars';
import {
  Button,
  EmptyState,
  Icon,
  InstrumentCard,
  Measure,
  Money,
  Private,
  TechTable,
  Tooltip,
  toast,
  type ContextMenuEntry,
  type TechColumn,
  type TechTableMenuContext,
} from '@/design/components';
import { errorMessage } from '@/features/portfolio/components/States';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatMoney, formatMonthShort } from '@/lib/format';

import { ActionDialog } from './ActionDialog';
import { useDeleteRecords } from './RecordsPanels';
import { TradeDialog } from './TradeDialog';
import { DeflatorSelect } from './components/DeflatorSelect';
import { PeriodSelect, periodRange, usePeriod } from './components/PeriodSelect';
import { Pct } from './components/Pct';
import { ErrorBlock, LoadingBlock } from './components/States';
import { formatQuantity } from './decimal';
import { classLabel, isValueOnly } from './labels';
import { usePortfolioUi } from './state';
import { useDeflatorLabel } from './useDeflatorLabel';

/** Resultado real com o cálculo na dica: custo, inflação do período e custo corrigido. */
function RealCell({ r, index }: { r: SaleRow; index: string }) {
  if (r.real_result == null || r.real_cost == null) {
    return (
      <Tooltip content={t('carteira-vendas-sem-indice', { indice: index })}>
        <span className="num text-dim">—</span>
      </Tooltip>
    );
  }
  return (
    <Tooltip
      content={
        <span className="flex flex-col gap-0.5">
          <span>{t('carteira-vendas-dica-custo', { indice: index })}</span>
          <span>
            {withSlots(
              t('carteira-vendas-dica-conta', {
                custo: slot(0),
                variacao: slot(1),
                corrigido: slot(2),
              }),
              [
                <Money key="c" cents={r.cost} />,
                <Pct key="v" value={r.index_change} signed decimals={2} />,
                <Money key="r" cents={r.real_cost} />,
              ],
            )}
          </span>
        </span>
      }
    >
      <span>
        <Money cents={r.real_result} tone="auto" signed estimated={r.estimated} />
      </span>
    </Tooltip>
  );
}

/** Resultado com proventos (preço + proventos), com o real na dica. */
function TotalCell({ r, index }: { r: SaleRow; index: string }) {
  return (
    <Tooltip
      content={
        <span className="flex flex-col gap-0.5">
          <span>
            {withSlots(t('carteira-vendas-dica-total', { preco: slot(0), proventos: slot(1) }), [
              <Money key="p" cents={r.result} signed />,
              <Money key="i" cents={r.income} signed />,
            ])}
          </span>
          <span>
            {r.real_total_result == null
              ? t('carteira-vendas-sem-indice', { indice: index })
              : withSlots(t('carteira-vendas-dica-total-real', { indice: index, valor: slot(0) }), [
                  <Money key="r" cents={r.real_total_result} signed />,
                ])}
          </span>
        </span>
      }
    >
      <span className="flex flex-col items-end">
        <Money cents={r.total_result} tone="auto" signed />
        <Pct value={r.total_pct} signed tone="auto" className="text-[11.5px]" />
      </span>
    </Tooltip>
  );
}

/** "Venda de 100 ABCD3 em 02/01/2026". */
function saleText(r: SaleRow): string {
  if (isValueOnly(r.class))
    return t('carteira-resgate-de', {
      valor: formatMoney(r.proceeds),
      ativo: r.ticker ?? r.name,
      data: formatDate(r.date),
    });
  return t('carteira-venda-de', {
    qtd: formatQuantity(r.quantity),
    ativo: r.ticker ?? r.name,
    data: formatDate(r.date),
  });
}

/** Linha da venda com o `id` da operação (para excluir pelo mesmo diálogo das Operações). */
type SaleRecord = SaleRow & { id: string };

/**
 * Ajustes pela linha da venda (menu de contexto e Enter): editar a venda, registrar a compra
 * ou a venda que faltou, um evento corporativo (desdobramento, bonificação, amortização),
 * abrir o extrato do ativo e excluir. Cada gravação recalcula o livro de vendas.
 */
function useSaleActions() {
  const navigate = useNavigate();
  const [editing, setEditing] = useState<Trade | null>(null);
  const [creating, setCreating] = useState<{ assetId?: string; side?: TradeSide } | null>(null);
  const [event, setEvent] = useState<string | null>(null);
  const { ask: askDelete, dialog: deleteDialog } = useDeleteRecords<SaleRecord>(useDeleteTrades(), {
    title: (n) => t('carteira-excluir-operacoes', { n }),
    item: saleText,
    note: () => t('carteira-vendas-excluir-nota'),
    done: (n) => t('carteira-operacoes-excluidas', { n }),
  });
  const edit = (r: SaleRow) => {
    call<Trade>('portfolio.get_trade', { id: r.trade_id })
      .then(setEditing)
      .catch((e: unknown) =>
        toast({
          tone: 'error',
          title: t('carteira-vendas-erro-abrir'),
          description: errorMessage(e),
        }),
      );
  };
  const page = (r: SaleRow, extra = '') =>
    `/ativos?ativo=${encodeURIComponent(r.asset_id)}${extra}`;
  const rowMenu = (r: SaleRow, ctx: TechTableMenuContext): ContextMenuEntry[] => [
    { heading: saleText(r) },
    {
      label: t('carteira-vendas-menu-editar'),
      icon: 'sliders',
      shortcut: ['enter'],
      onSelect: () => edit(r),
    },
    {
      label: t('carteira-vendas-menu-compra'),
      icon: 'plus',
      onSelect: () => setCreating({ assetId: r.asset_id, side: 'buy' }),
    },
    {
      label: t('carteira-vendas-menu-venda'),
      icon: 'plus',
      onSelect: () => setCreating({ assetId: r.asset_id, side: 'sell' }),
    },
    {
      label: t('carteira-vendas-menu-evento'),
      icon: 'plus',
      onSelect: () => setEvent(r.asset_id),
    },
    { separator: true },
    {
      label: t('ativos-menu-ver-extrato'),
      icon: 'history',
      onSelect: () => void navigate(page(r, '&secao=extrato')),
    },
    {
      label: t('ativos-menu-abrir'),
      icon: 'arrowRight',
      onSelect: () => void navigate(page(r)),
    },
    {
      label: t('carteira-menu-copiar-linha'),
      icon: 'copy',
      shortcut: ['mod', 'C'],
      onSelect: ctx.copy,
    },
    { separator: true },
    {
      label: t('carteira-vendas-menu-excluir'),
      icon: 'x',
      danger: true,
      shortcut: ['mod', 'Backspace'],
      onSelect: () => askDelete([{ ...r, id: r.trade_id }]),
    },
  ];
  const dialogs = (
    <>
      <TradeDialog
        open={editing != null || creating != null}
        onOpenChange={(o) => {
          if (!o) {
            setEditing(null);
            setCreating(null);
          }
        }}
        trade={editing}
        {...(creating?.assetId ? { assetId: creating.assetId } : {})}
        {...(creating?.side ? { side: creating.side } : {})}
      />
      <ActionDialog
        open={event != null}
        onOpenChange={(o) => !o && setEvent(null)}
        {...(event ? { assetId: event } : {})}
      />
      {deleteDialog}
    </>
  );
  return { rowMenu, edit, dialogs, newTrade: () => setCreating({}) };
}

export function SalesPanel() {
  const deflatorLabel = useDeflatorLabel();
  const actions = useSaleActions();
  const deflator = usePortfolioUi((s) => s.deflator);
  const [period, setPeriod] = usePeriod('vendas', 'ytd');
  const range = periodRange(period);
  const q = useSales({
    deflator,
    ...(range.fromDate ? { from: range.fromDate } : {}),
    ...(range.toDate ? { to: range.toDate } : {}),
  });
  const r = q.data;
  const index = deflatorLabel(r?.deflator ?? deflator);

  const method: CostMethod = r?.cost_method ?? 'average';
  const columns = useMemo((): TechColumn<SaleRow>[] => {
    const byLot = method !== 'average';
    return [
      {
        key: 'date',
        header: t('carteira-col-data'),
        width: '9%',
        cell: (x) => <span className="num">{formatDate(x.date)}</span>,
        sortValue: (x) => x.date,
      },
      {
        key: 'asset',
        header: t('carteira-col-ativo'),
        width: '17%',
        cell: (x) => (
          <span className="flex min-w-0 flex-col">
            <span className="num truncate text-fg">{x.ticker ?? x.name}</span>
            <span className="truncate text-[11.5px] text-dim">
              {classLabel(x.class)} · {x.account_name}
            </span>
          </span>
        ),
        sortValue: (x) => x.ticker ?? x.name,
      },
      {
        key: 'qty',
        header: t('carteira-col-qtd'),
        width: '7%',
        numeric: true,
        cell: (x) => (isValueOnly(x.class) ? '—' : <Private>{formatQuantity(x.quantity)}</Private>),
        sortValue: (x) => (isValueOnly(x.class) ? -1 : Number(x.quantity)),
      },
      {
        key: 'proceeds',
        header: t('carteira-vendas-col-venda'),
        width: '11%',
        numeric: true,
        cell: (x) => <Money cents={x.proceeds} />,
        sortValue: (x) => x.proceeds,
      },
      {
        key: 'cost',
        header: t('carteira-vendas-col-custo'),
        width: '11%',
        numeric: true,
        cell: (x) => <Money cents={x.cost} />,
        sortValue: (x) => x.cost,
      },
      {
        key: 'result',
        header: t('carteira-vendas-col-nominal'),
        width: '12%',
        numeric: true,
        cell: (x) => (
          <span className="flex flex-col items-end">
            <Money cents={x.result} tone="auto" signed />
            <Pct value={x.result_pct} signed tone="auto" className="text-[11.5px]" />
          </span>
        ),
        sortValue: (x) => x.result,
      },
      ...(byLot
        ? [
            {
              key: 'average',
              header: t('carteira-vendas-col-medio'),
              width: '11%',
              numeric: true,
              cell: (x: SaleRow) =>
                x.average_result == null ? (
                  <span className="text-dim">—</span>
                ) : (
                  <Money cents={x.average_result} tone="auto" signed />
                ),
              sortValue: (x: SaleRow) => x.average_result ?? x.result,
            },
          ]
        : []),
      {
        key: 'income',
        header: t('carteira-vendas-col-proventos'),
        width: '10%',
        numeric: true,
        cell: (x) =>
          x.income ? <Money cents={x.income} signed /> : <span className="text-dim">—</span>,
        sortValue: (x) => x.income,
      },
      {
        key: 'total',
        header: t('carteira-vendas-col-total'),
        width: '12%',
        numeric: true,
        cell: (x) => <TotalCell r={x} index={index} />,
        sortValue: (x) => x.total_result,
      },
      {
        key: 'real',
        header: t('carteira-vendas-col-real', { indice: index }),
        width: '13%',
        numeric: true,
        cell: (x) => (
          <span className="flex flex-col items-end">
            <RealCell r={x} index={index} />
            <Pct value={x.real_pct} signed tone="auto" className="text-[11.5px]" />
          </span>
        ),
        sortValue: (x) => x.real_result ?? Number.NEGATIVE_INFINITY,
      },
      {
        key: 'index',
        header: t('carteira-vendas-col-indice', { indice: index }),
        width: '9%',
        numeric: true,
        cell: (x) => <Pct value={x.index_change} signed decimals={2} estimated={x.estimated} />,
        sortValue: (x) => x.index_change ?? 0,
      },
      {
        key: 'days',
        header: t('carteira-vendas-col-prazo'),
        width: '9%',
        numeric: true,
        cell: (x) => (
          <span className="num">
            {x.holding_days == null ? '—' : t('carteira-vendas-dias', { n: x.holding_days })}
          </span>
        ),
        sortValue: (x) => x.holding_days ?? 0,
      },
      {
        key: 'irr',
        header: t('carteira-vendas-col-irr'),
        width: '9%',
        numeric: true,
        cell: (x) =>
          x.irr == null ? (
            <span className="text-dim">—</span>
          ) : (
            <Tooltip content={t('carteira-vendas-irr-dica')}>
              <span>
                <Pct value={x.irr} signed tone="auto" />
              </span>
            </Tooltip>
          ),
        sortValue: (x) => x.irr ?? Number.NEGATIVE_INFINITY,
      },
    ];
  }, [index, method]);

  const chart = useMemo(() => {
    if (!r || r.by_month.length < 2) return null;
    return {
      rows: r.by_month.map((g) => ({ key: g.key, label: formatMonthShort(g.key) })),
      groups: [
        {
          key: 'nominal',
          name: t('carteira-vendas-nominal'),
          color: 'accent',
          values: r.by_month.map((g) => g.totals.result),
        },
        {
          key: 'real',
          name: t('carteira-vendas-real', { indice: index }),
          color: 'warning',
          values: r.by_month.map((g) => g.totals.real_result ?? 0),
        },
      ],
      currency: r.currency,
    };
  }, [r, index]);

  const controls = (
    <div className="flex flex-wrap items-center gap-3">
      <PeriodSelect scope="vendas" value={period} onChange={setPeriod} />
      <DeflatorSelect />
      <span className="flex-1" />
      <Button
        size="sm"
        variant="secondary"
        icon={<Icon name="plus" size={12} />}
        onClick={actions.newTrade}
      >
        {t('carteira-vendas-registrar')}
      </Button>
    </div>
  );

  if (q.isLoading) return <LoadingBlock />;
  if (q.error) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  if (!r) return null;
  const tot = r.totals;
  return (
    <div className="flex flex-col gap-4">
      {controls}
      {tot.count === 0 ? (
        <EmptyState
          kicker={t('carteira-aba-vendas')}
          title={t('carteira-vendas-vazio-titulo')}
          description={t('carteira-vendas-vazio-texto')}
        />
      ) : (
        <>
          <InstrumentCard index="07.30" title={t('carteira-vendas-titulo')}>
            <div className="grid gap-5 @min-[720px]:grid-cols-3 @min-[1100px]:grid-cols-5">
              <Measure
                size="md"
                label={t('carteira-vendas-total-vendido')}
                value={<Money cents={tot.proceeds} />}
                hint={t('carteira-vendas-n', {
                  n: tot.count,
                  ganhos: tot.gains,
                  perdas: tot.losses,
                })}
              />
              <Measure
                size="md"
                label={t('carteira-vendas-nominal')}
                value={<Money cents={tot.result} tone="auto" signed />}
                hint={
                  method === 'fifo'
                    ? t('carteira-vendas-nominal-dica-fifo')
                    : method === 'uk_pool'
                      ? t('carteira-vendas-nominal-dica-pool')
                      : t('carteira-vendas-nominal-dica')
                }
              />
              {method !== 'average' && tot.average_result != null ? (
                <Measure
                  size="md"
                  label={t('carteira-vendas-medio')}
                  value={<Money cents={tot.average_result} tone="auto" signed />}
                  hint={t('carteira-vendas-medio-dica')}
                />
              ) : null}
              <Measure
                size="md"
                label={t('carteira-vendas-total')}
                value={<Money cents={tot.total_result} tone="auto" signed />}
                hint={withSlots(t('carteira-vendas-total-dica', { proventos: slot(0) }), [
                  <Money key="i" cents={tot.income} />,
                ])}
              />
              <Measure
                size="md"
                label={t('carteira-vendas-real', { indice: index })}
                value={
                  tot.real_result == null ? (
                    '—'
                  ) : (
                    <Money cents={tot.real_result} tone="auto" signed />
                  )
                }
                hint={
                  tot.missing > 0
                    ? t('carteira-vendas-faltam', { n: tot.missing, indice: index })
                    : t('carteira-vendas-real-dica', {
                        ganhos: tot.real_gains,
                        perdas: tot.real_losses,
                      })
                }
              />
              <Measure
                size="md"
                label={t('carteira-vendas-efeito', { indice: index })}
                value={
                  tot.inflation_effect == null ? (
                    '—'
                  ) : (
                    <Money cents={-tot.inflation_effect} signed />
                  )
                }
                hint={t('carteira-vendas-efeito-dica', { indice: index })}
              />
            </div>
          </InstrumentCard>
          {chart ? (
            <InstrumentCard index="07.31" title={t('carteira-vendas-por-mes')}>
              <GroupedBars input={chart} ariaLabel={t('carteira-vendas-por-mes')} />
            </InstrumentCard>
          ) : null}
          <TechTable
            className="[&_table]:table-fixed"
            minWidth="1000px"
            label={t('carteira-vendas-tabela')}
            prefsId="carteira-vendas"
            rows={r.rows}
            columns={columns}
            getRowId={(x) => x.trade_id}
            defaultSort={{ key: 'date', direction: 'desc' }}
            onActivate={actions.edit}
            rowMenu={actions.rowMenu}
            rowMenuLabel={(x) => t('carteira-acoes-de', { nome: saleText(x) })}
          />
          <p className="text-[12px] text-dim">
            {method === 'fifo'
              ? t('carteira-vendas-metodo-fifo', { indice: index })
              : method === 'uk_pool'
                ? t('carteira-vendas-metodo-pool', { indice: index })
                : t('carteira-vendas-metodo', { indice: index })}{' '}
            {r.cost_method_automatic
              ? t('carteira-vendas-metodo-pais')
              : t('carteira-vendas-metodo-escolhido')}
          </p>
        </>
      )}
      {actions.dialogs}
    </div>
  );
}
