/**
 * Extrato do ativo (`portfolio.statement`): operações e eventos corporativos na ordem em que a
 * carteira calcula, com a quantidade e o preço médio depois de cada linha, a origem e os
 * avisos (ex.: desdobramento contado duas vezes, compra que parece repetida).
 *
 * Menu de contexto da linha: editar a operação (Enter), copiar e excluir a operação ou o evento
 * (⌘⌫), sempre com confirmação e "Desfazer" no aviso.
 */
import { useMemo, useState } from 'react';

import type { CorporateAction } from '@/api/generated/CorporateAction';
import type { StatementLine } from '@/api/generated/StatementLine';
import type { AssetClass } from '@/api/generated/AssetClass';
import type { ComeCotasRegime } from '@/api/generated/ComeCotasRegime';
import type { Trade } from '@/api/generated/Trade';
import {
  useActions,
  useAssetStatement,
  useComeCotas,
  useDeleteActions,
  useDeleteTrades,
  useSetComeCotas,
  useTrades,
} from '@/api/hooks/portfolio';
import {
  copyOnlyMenus,
  Icon,
  Money,
  Private,
  Select,
  TechTable,
  type ContextMenuEntry,
  type TechTableMenuContext,
} from '@/design/components';
import { cx } from '@/design/cx';
import { ErrorBlock, LoadingBlock, Notice } from '@/features/portfolio/components/States';
import { formatQuantity, formatUnitPrice, productToMinor } from '@/features/portfolio/decimal';
import { ACTION_KIND_LABEL, isValueOnly } from '@/features/portfolio/labels';
import { tradeText, useDeleteRecords } from '@/features/portfolio/RecordsPanels';
import { TradeDialog } from '@/features/portfolio/TradeDialog';
import { t } from '@/i18n';
import { formatDate, formatMoney } from '@/lib/format';

const EVENT_KINDS = new Set([
  'split',
  'reverse_split',
  'bonus',
  'subscription',
  'merger',
  'ticker_change',
  'spin_off',
  'capital_return',
  'come_cotas',
  'inherited',
]);

const regimeOptions = (): { value: string; label: string }[] => [
  { value: 'auto', label: t('ativos-come-cotas-auto') },
  { value: 'long', label: t('ativos-come-cotas-longo') },
  { value: 'short', label: t('ativos-come-cotas-curto') },
  { value: 'exempt', label: t('ativos-come-cotas-isento') },
];

/**
 * Regra do come-cotas de um fundo (último dia útil de maio e novembro): automática pela
 * classificação da CVM ou escolhida. Os descontos entram no extrato e na posição.
 */
function ComeCotasBox({ assetId }: { assetId: string }) {
  const q = useComeCotas(assetId);
  const set = useSetComeCotas();
  if (!q.data) return null;
  const { rule, events, warnings } = q.data;
  const tax = events.reduce((s, e) => s + Number(e.tax), 0);
  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1.5 rounded-control border border-line bg-bg px-3 py-2 text-[12.5px]">
      <span className="label-caps text-dim">{t('ativos-come-cotas')}</span>
      <div className="w-52">
        <Select
          aria-label={t('ativos-come-cotas-regra')}
          value={rule.origin === 'user' ? rule.regime : 'auto'}
          onValueChange={(v) =>
            set.mutate({ asset_id: assetId, regime: v === 'auto' ? null : (v as ComeCotasRegime) })
          }
          options={regimeOptions()}
        />
      </div>
      <span className="text-dim">
        {rule.note}
        {events.length
          ? ` · ${t('ativos-come-cotas-descontos', {
              n: events.length,
              total: formatMoney(Math.round(tax * 100), 'BRL'),
            })}`
          : ''}
      </span>
      {warnings.length ? <span className="text-warning">{warnings[0]}</span> : null}
    </div>
  );
}

/**
 * Operações que parecem repetidas: outra do mesmo lado com a mesma data, quantidade e preço
 * (ex.: a mesma compra registrada à mão e pelo agente). Só sinaliza: excluir é com a pessoa.
 */
export function likelyDuplicates(lines: readonly StatementLine[]): Set<string> {
  const groups = new Map<string, string[]>();
  for (const l of lines) {
    if (!l.trade_id || (l.kind !== 'buy' && l.kind !== 'sell')) continue;
    const key = [l.kind, l.date, Number(l.quantity_change), Number(l.price ?? 0)].join('|');
    groups.set(key, [...(groups.get(key) ?? []), l.trade_id]);
  }
  return new Set([...groups.values()].filter((ids) => ids.length > 1).flat());
}

const lineText = (l: StatementLine) =>
  t('ativos-extrato-linha', { lancamento: l.label, data: formatDate(l.date) });

function signed(v: string) {
  const n = Number(v);
  if (!Number.isFinite(n) || n === 0) return '—';
  return `${n > 0 ? '+' : '−'}${formatQuantity(String(Math.abs(n)))}`;
}

export function AssetStatement({
  assetId,
  currency,
  assetClass,
}: {
  assetId: string;
  currency: string;
  assetClass?: AssetClass;
}) {
  const q = useAssetStatement(assetId);
  // Os registros por trás das linhas (para editar e excluir pelo menu).
  const trades = useTrades({ asset_id: assetId });
  const actions = useActions(assetId);
  const tradeById = useMemo(
    () => new Map((trades.data ?? []).map((x) => [x.id, x])),
    [trades.data],
  );
  const actionById = useMemo(
    () => new Map((actions.data ?? []).map((x) => [x.id, x])),
    [actions.data],
  );
  const [editing, setEditing] = useState<Trade | null>(null);
  const tradeRemoval = useDeleteRecords<Trade>(useDeleteTrades(), {
    title: (n) => t('carteira-excluir-operacoes', { n }),
    item: tradeText,
    note: (n) => t('carteira-excluir-operacoes-nota', { n }),
    done: (n) => t('carteira-operacoes-excluidas', { n }),
  });
  const actionRemoval = useDeleteRecords<CorporateAction>(useDeleteActions(), {
    title: (n) => t('carteira-excluir-eventos', { n }),
    item: (a) =>
      t('ativos-extrato-linha', {
        lancamento: ACTION_KIND_LABEL[a.kind],
        data: formatDate(a.date),
      }),
    note: () => t('carteira-excluir-eventos-nota'),
    done: (n) => t('carteira-eventos-excluidos', { n }),
  });
  const duplicates = useMemo(() => likelyDuplicates(q.data ?? []), [q.data]);
  // Renda fixa: o extrato fala em valores (aplicado, resgatado, custo), nunca em quantidade.
  const byValue = isValueOnly(assetClass);
  if (q.isPending) return <LoadingBlock label={t('ativos-extrato-montando')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const lines = q.data;
  const warnings = lines.filter((l) => l.warning);
  const tradeOf = (l: StatementLine) => (l.trade_id ? tradeById.get(l.trade_id) : undefined);
  const actionOf = (l: StatementLine) => (l.action_id ? actionById.get(l.action_id) : undefined);
  const copyOnly = copyOnlyMenus<StatementLine>();
  const rowMenu = (l: StatementLine, ctx: TechTableMenuContext): ContextMenuEntry[] => {
    const trade = tradeOf(l);
    const action = actionOf(l);
    return [
      { heading: lineText(l) },
      trade && {
        label: t('carteira-menu-editar'),
        icon: 'sliders',
        shortcut: ['enter'],
        onSelect: () => setEditing(trade),
      },
      {
        label: t('carteira-menu-copiar-linha'),
        icon: 'copy',
        shortcut: ['mod', 'C'],
        onSelect: ctx.copy,
      },
      (trade || action) && { separator: true },
      (trade || action) && {
        label: t('carteira-menu-excluir'),
        icon: 'x',
        danger: true,
        shortcut: ['mod', 'Backspace'],
        onSelect: () => (trade ? tradeRemoval.ask([trade]) : actionRemoval.ask([action!])),
      },
    ];
  };
  return (
    <div className="flex flex-col gap-3">
      {assetClass === 'fund' ? <ComeCotasBox assetId={assetId} /> : null}
      {warnings.length ? (
        <Notice tone="warning" title={t('ativos-extrato-confira')}>
          {t('ativos-extrato-aviso', { aviso: warnings[0]!.warning ?? '', n: warnings.length })}
        </Notice>
      ) : null}
      {duplicates.size ? (
        <Notice tone="warning" title={t('ativos-extrato-duplicatas-titulo')}>
          {t('ativos-extrato-duplicatas', { n: duplicates.size })}
        </Notice>
      ) : null}
      <p className="text-[12px] text-dim">{t('ativos-extrato-ajuda')}</p>
      <TechTable<StatementLine>
        label={t('ativos-extrato-tabela')}
        prefsId="Extrato do ativo"
        dense
        rows={[...lines].reverse()}
        getRowId={(l) => l.trade_id ?? l.action_id ?? `${l.date}-${l.kind}-${l.quantity_after}`}
        empty={t('ativos-extrato-vazio')}
        {...copyOnly}
        rowMenu={rowMenu}
        rowMenuLabel={(l) => t('carteira-acoes-de', { nome: lineText(l) })}
        onActivate={(l) => {
          const trade = tradeOf(l);
          if (trade) setEditing(trade);
        }}
        columns={[
          { key: 'date', header: t('ativos-data'), cell: (l) => formatDate(l.date) },
          {
            key: 'label',
            header: t('ativos-extrato-lancamento'),
            cell: (l) => (
              <span
                className={cx(
                  'inline-flex items-center gap-1.5',
                  EVENT_KINDS.has(l.kind) && 'text-accent',
                  l.kind === 'sell' && 'text-negative',
                )}
                title={l.notes ?? undefined}
              >
                {l.warning ? <Icon name="alert" size={11} className="text-warning" /> : null}
                {l.trade_id && duplicates.has(l.trade_id) ? (
                  <span
                    role="img"
                    aria-label={t('ativos-extrato-duplicata')}
                    title={t('ativos-extrato-duplicata')}
                  >
                    <Icon name="alert" size={11} className="text-warning" />
                  </span>
                ) : null}
                {l.kind === 'buy' ? '▲ ' : l.kind === 'sell' ? '▼ ' : '◆ '}
                {l.label}
              </span>
            ),
          },
          ...(byValue
            ? [
                {
                  key: 'value',
                  header: t('ativos-valor'),
                  numeric: true,
                  cell: (l: StatementLine) =>
                    l.price ? (
                      <Money
                        cents={productToMinor([l.quantity_change, l.price], currency)}
                        currency={currency}
                        tone="auto"
                        signed
                      />
                    ) : (
                      '—'
                    ),
                },
                {
                  key: 'cost-after',
                  header: t('ativos-extrato-custo-depois'),
                  numeric: true,
                  cell: (l: StatementLine) =>
                    l.average_price_after ? (
                      <Money
                        cents={productToMinor([l.quantity_after, l.average_price_after], currency)}
                        currency={currency}
                      />
                    ) : (
                      '—'
                    ),
                },
              ]
            : [
                {
                  key: 'change',
                  header: t('ativos-qtd'),
                  numeric: true,
                  cell: (l: StatementLine) => (
                    <Private>
                      {signed(l.quantity_change)}
                      {l.recorded_quantity ? (
                        <span className="ml-1 text-[11px] text-warning">
                          {t('ativos-extrato-qtd-registrada', {
                            qtd: formatQuantity(String(Math.abs(Number(l.recorded_quantity)))),
                          })}
                        </span>
                      ) : null}
                    </Private>
                  ),
                },
                {
                  key: 'price',
                  header: t('ativos-preco'),
                  numeric: true,
                  cell: (l: StatementLine) => (
                    <Private>{l.price ? formatUnitPrice(l.price, currency) : '—'}</Private>
                  ),
                },
                {
                  key: 'after',
                  header: t('ativos-extrato-posicao-depois'),
                  numeric: true,
                  cell: (l: StatementLine) => <Private>{formatQuantity(l.quantity_after)}</Private>,
                },
                {
                  key: 'avg',
                  header: t('ativos-preco-medio'),
                  numeric: true,
                  cell: (l: StatementLine) => (
                    <Private>
                      {l.average_price_after
                        ? formatUnitPrice(l.average_price_after, currency, { maxDecimals: 4 })
                        : '—'}
                    </Private>
                  ),
                },
              ]),
          {
            key: 'origin',
            header: t('ativos-extrato-origem'),
            cell: (l) => <span className="text-dim">{l.origin}</span>,
          },
        ]}
      />
      <TradeDialog
        open={editing != null}
        onOpenChange={(o) => !o && setEditing(null)}
        trade={editing}
      />
      {tradeRemoval.dialog}
      {actionRemoval.dialog}
    </div>
  );
}
