/** Cards de instrumento do 01 · Painel. */
import { useMemo } from 'react';
import { Link, useNavigate } from 'react-router';

import type { NetWorth } from '@/api/generated/NetWorth';
import type { NetWorthPoint } from '@/api/generated/NetWorthPoint';
import type { PortfolioSummary } from '@/api/generated/PortfolioSummary';
import type { UpcomingItem } from '@/api/generated/UpcomingItem';
import { BulletBar, BulletStatusLabel } from '@/charts/BulletBar';
import { bulletStatus } from '@/charts/bullet';
import {
  Button,
  Icon,
  InstrumentCard,
  Measure,
  Money,
  Private,
  Sparkline,
} from '@/design/components';
import { cx } from '@/design/cx';
import type { EntityColors } from '@/features/reports/model/colors';
import { categoryDrill, uncategorizedDrill, upcomingDrill } from '@/features/reports/model/drill';
import { Explainable } from '@/features/reports/explain/ExplainButton';
import { useDeflatorLabel } from '@/features/portfolio/useDeflatorLabel';
import { capitalize } from '@/features/reports/controls';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import { formatDate, formatMonthLong, formatPercent } from '@/lib/format';
import { daysBetween, monthBounds } from '@/lib/months';
import { txnLink } from '@/lib/txnLink';

import { budgetPace, type DashboardView } from './model';

const basisText = (b: string) =>
  b === 'cash' ? t('painel-base-caixa') : t('painel-base-competencia');

// ── 01 · Patrimônio líquido ─────────────────────────────────────

export function NetWorthCard({
  netWorth,
  history,
}: {
  netWorth: NetWorth;
  history: readonly NetWorthPoint[] | undefined;
}) {
  const accounts = netWorth.assets - netWorth.investments - netWorth.physical;
  const missing = netWorth.missing_fx;
  const missingAccounts = netWorth.accounts.filter((a) => a.balance_base == null);
  const first = history?.[0];
  const change = first ? netWorth.net - first.net : null;
  return (
    <InstrumentCard
      index="01"
      title={t('painel-patrimonio')}
      className="@min-[1040px]:col-span-7"
      footer={[
        { label: t('painel-fonte'), value: t('painel-patrimonio-fonte') },
        { label: t('painel-data'), value: formatDate(netWorth.as_of) },
        { label: t('painel-contas'), value: String(netWorth.accounts.length) },
      ]}
    >
      <div className="flex flex-wrap items-end justify-between gap-6">
        <Measure
          size="xl"
          label={t('painel-patrimonio')}
          origin={{
            source: t('painel-patrimonio-origem'),
            date: netWorth.as_of,
            status: netWorth.estimated ? 'estimado' : 'oficial',
            note: netWorth.estimated ? t('painel-patrimonio-estimado') : undefined,
          }}
          changeKey={netWorth.net}
          value={
            <Explainable
              target={{
                title: t('painel-patrimonio-em', { data: formatDate(netWorth.as_of) }),
                args: { metric: 'net_worth', params: { as_of: netWorth.as_of } },
              }}
            >
              <Money
                cents={netWorth.net}
                currency={netWorth.base_currency}
                estimated={netWorth.estimated}
              />
            </Explainable>
          }
          delta={
            change != null && first ? (
              <Money cents={change} currency={netWorth.base_currency} tone="auto" trend />
            ) : undefined
          }
          hint={first ? t('painel-desde', { mes: formatMonthLong(first.month) }) : undefined}
        />
        {history && history.length > 1 ? (
          <Sparkline
            values={history.map((p) => p.net)}
            width={220}
            height={48}
            label={t('painel-patrimonio-historico', { n: history.length })}
          />
        ) : null}
      </div>
      <dl className="mt-5 grid grid-cols-2 gap-4 border-t border-line pt-4 @min-[400px]:grid-cols-4">
        <div className="flex flex-col gap-1">
          <dt className="label-caps">{t('painel-contas')}</dt>
          <dd className="num text-[15px] text-fg">
            <Money cents={accounts} currency={netWorth.base_currency} />
          </dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="label-caps">{t('painel-investimentos')}</dt>
          <dd className="num text-[15px] text-fg">
            <Money cents={netWorth.investments} currency={netWorth.base_currency} />
          </dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="label-caps">
            <Link to="/carteira?aba=bens" className="hover:text-fg hover:underline">
              {t('painel-bens')}
            </Link>
          </dt>
          <dd className="num text-[15px] text-fg">
            <Money cents={netWorth.physical} currency={netWorth.base_currency} />
          </dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="label-caps">{t('painel-dividas')}</dt>
          <dd className="num text-[15px] text-fg">
            <Money cents={-netWorth.liabilities} currency={netWorth.base_currency} />
          </dd>
        </div>
      </dl>
      {missing.length > 0 ? (
        <p
          role="note"
          className="mt-4 flex items-start gap-2 rounded-control border border-warning/40 bg-warning-soft px-3 py-2 text-[12.5px] text-fg"
        >
          <Icon name="alert" size={14} className="mt-0.5 shrink-0 text-warning" />
          <span>
            {tRich(
              'painel-sem-cambio',
              {
                moedas: missing.join(', '),
                contas: missingAccounts.map((a) => a.name).join(', '),
              },
              (c) => (
                <strong className="font-medium">{c}</strong>
              ),
            )}
          </span>
        </p>
      ) : null}
    </InstrumentCard>
  );
}

// ── 02 · Fluxo do mês ───────────────────────────────────────────

export function FlowCard({ view }: { view: DashboardView }) {
  const { flow, flowHistory, basis } = view;
  const { from, to } = monthBounds(view.month);
  const monthName = formatMonthLong(view.month);
  const flowParams = { from, to, basis };
  return (
    <InstrumentCard
      index="02"
      title={t('painel-fluxo')}
      className="@min-[1040px]:col-span-5"
      footer={[
        { label: t('painel-base'), value: basisText(basis) },
        { label: t('painel-periodo'), value: capitalize(monthName) },
        ...(flow.estimated
          ? [{ label: t('painel-cambio'), value: t('painel-cambio-estimado') }]
          : []),
      ]}
    >
      <dl className="grid grid-cols-2 gap-x-6 gap-y-4">
        <div className="flex flex-col gap-1">
          <dt className="label-caps">{t('painel-receitas')}</dt>
          <dd className="num text-[20px] text-fg">
            <Explainable
              target={{
                title: t('painel-receitas-de', { mes: monthName }),
                args: { metric: 'income', params: flowParams },
                link: txnLink(categoryDrill({ period: { from, to }, basis }, [], 'income')),
              }}
            >
              <Money cents={flow.income} estimated={flow.estimated} />
            </Explainable>
          </dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="label-caps">{t('painel-despesas')}</dt>
          <dd className="num text-[20px] text-fg">
            <Explainable
              target={{
                title: t('painel-despesas-de', { mes: monthName }),
                args: { metric: 'spending', params: flowParams },
                link: txnLink(categoryDrill({ period: { from, to }, basis }, [])),
              }}
            >
              <Money cents={flow.expense} estimated={flow.estimated} />
            </Explainable>
          </dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="label-caps">{t('painel-saldo-mes')}</dt>
          <dd className="num text-[20px]">
            <Money cents={flow.net} tone="auto" trend />
          </dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="label-caps">{t('painel-taxa-poupanca')}</dt>
          <dd className="num text-[20px] text-fg">
            {flow.savings_rate == null ? (
              <span className="text-dim">—</span>
            ) : (
              <Explainable
                target={{
                  title: t('painel-taxa-poupanca-de', { mes: monthName }),
                  args: { metric: 'savings_rate', params: { month: view.month, basis } },
                }}
              >
                <Private>{formatPercent(flow.savings_rate)}</Private>
              </Explainable>
            )}
          </dd>
        </div>
      </dl>
      <div className="mt-4 flex items-end justify-between gap-4 border-t border-line pt-3">
        <div className="flex flex-col gap-0.5">
          <span className="label-caps">{t('painel-media-12m')}</span>
          <span className="num text-[14px] text-fg">
            {view.savingsRate12m == null ? (
              '—'
            ) : (
              <Private>{formatPercent(view.savingsRate12m)}</Private>
            )}
          </span>
        </div>
        <Sparkline
          values={flowHistory.map((f) => f.savings_rate ?? 0)}
          width={170}
          height={36}
          label={t('painel-taxa-poupanca-historico', { n: flowHistory.length })}
        />
      </div>
    </InstrumentCard>
  );
}

// ── 03 · Orçamento do mês ───────────────────────────────────────

export function BudgetCard({ view }: { view: DashboardView }) {
  const b = view.budget;
  const monthName = formatMonthLong(view.month);
  if (!b) {
    return (
      <InstrumentCard index="03" title={t('painel-orcamento')} className="@min-[1040px]:col-span-4">
        <p className="text-[13px] text-dim">{t('painel-orcamento-carregando')}</p>
      </InstrumentCard>
    );
  }
  const p = budgetPace(b);
  const status = bulletStatus({
    planned: p.planned,
    actual: p.actual,
    committed: p.committed,
    elapsed: p.elapsed,
  });
  return (
    <InstrumentCard
      index="03"
      title={t('painel-orcamento')}
      className="@min-[1040px]:col-span-4"
      footer={
        <Link
          to="/orcamento"
          className="label-caps inline-flex items-center gap-1.5 hover:text-accent"
        >
          {t('painel-orcamento-abrir')} <Icon name="arrowRight" size={12} />
        </Link>
      }
    >
      {p.planned <= 0 ? (
        <div className="flex flex-col gap-2">
          <p className="text-[13.5px] text-fg">{t('painel-orcamento-sem', { mes: monthName })}</p>
          <p className="text-[12.5px] text-dim">
            {tRich('painel-orcamento-gasto', { valor: <Money cents={p.actual} /> })}
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          <div className="flex items-baseline justify-between gap-3">
            <span className="num text-[22px] text-fg">
              <Money cents={p.actual} />
            </span>
            <span className="num text-[12.5px] text-dim">
              {tRich('painel-orcamento-de-valor', { valor: <Money cents={p.planned} /> })}
            </span>
          </div>
          <BulletBar
            label={t('painel-orcamento-de', { mes: monthName })}
            planned={p.planned}
            actual={p.actual}
            committed={p.committed}
            elapsed={p.elapsed}
          />
          <div className="flex flex-wrap items-center justify-between gap-2 text-[12px]">
            <BulletStatusLabel status={status} />
            <span className="num text-dim">
              {tRich('painel-orcamento-ritmo', {
                pct: (
                  <Private>
                    {p.ratio != null ? formatPercent(p.ratio, { decimals: 0 }) : '—'}
                  </Private>
                ),
                decorrido: formatPercent(p.elapsed, { decimals: 0 }),
              })}
            </span>
          </div>
          <p className="text-[12px] text-dim">
            {tRich('painel-orcamento-comprometido', { valor: <Money cents={p.committed} /> })}
          </p>
        </div>
      )}
    </InstrumentCard>
  );
}

// ── 04 · Carteira ───────────────────────────────────────────────

export function PortfolioCard({ summary }: { summary: PortfolioSummary | undefined }) {
  const deflatorLabel = useDeflatorLabel();
  return (
    <InstrumentCard
      index="04"
      title={t('casca-tela-carteira')}
      className="@min-[1040px]:col-span-4"
      footer={
        summary
          ? [
              { label: t('painel-data'), value: formatDate(summary.as_of) },
              {
                label: t('painel-real'),
                value:
                  summary.deflator === 'none'
                    ? t('painel-nominal')
                    : deflatorLabel(summary.deflator),
              },
            ]
          : undefined
      }
    >
      {!summary ? (
        <p className="text-[13px] text-dim">{t('painel-carteira-carregando')}</p>
      ) : summary.market_value === 0 && summary.cost === 0 ? (
        <div className="flex flex-col items-start gap-2">
          <p className="text-[13.5px] text-fg">{t('painel-carteira-vazia')}</p>
          <Link
            to="/carteira"
            className="label-caps inline-flex items-center gap-1.5 hover:text-accent"
          >
            {t('painel-carteira-cadastrar')} <Icon name="arrowRight" size={12} />
          </Link>
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          <Measure
            size="md"
            label={t('painel-valor-mercado')}
            origin={{
              source: t('painel-carteira-origem'),
              date: summary.as_of,
              status: summary.estimated || summary.missing_prices.length ? 'estimado' : 'oficial',
              note: summary.missing_prices.length
                ? t('painel-sem-cotacao', { ativos: summary.missing_prices.join(', ') })
                : undefined,
            }}
            value={<Money cents={summary.market_value} estimated={summary.estimated} />}
            delta={<Money cents={summary.total_result} tone="auto" trend />}
            hint={t('painel-resultado-total')}
          />
          <dl className="grid grid-cols-2 gap-3 border-t border-line pt-3">
            <div className="flex flex-col gap-0.5">
              <dt className="label-caps">{t('painel-xirr-nominal')}</dt>
              <dd className="num text-[14px] text-fg">
                {summary.xirr == null ? (
                  '—'
                ) : (
                  <Private>{formatPercent(summary.xirr, { signed: true })}</Private>
                )}
              </dd>
            </div>
            <div className="flex flex-col gap-0.5">
              <dt className="label-caps">{t('painel-xirr-real')}</dt>
              <dd className="num text-[14px] text-fg">
                {summary.xirr_real == null ? (
                  <span className="text-dim" title={t('painel-xirr-real-dica')}>
                    {'—'}
                  </span>
                ) : (
                  <Private>{formatPercent(summary.xirr_real, { signed: true })}</Private>
                )}
              </dd>
            </div>
          </dl>
        </div>
      )}
    </InstrumentCard>
  );
}

// ── 05 · Principais categorias ──────────────────────────────────

export function TopCategoriesCard({ view, colors }: { view: DashboardView; colors: EntityColors }) {
  const navigate = useNavigate();
  const period = monthBounds(view.month);
  const max = Math.max(1, ...view.topCategories.map((g) => g.amount));
  return (
    <InstrumentCard
      index="05"
      title={t('painel-categorias')}
      className="@min-[1040px]:col-span-4"
      footer={[
        { label: t('painel-despesas-mes'), value: <Money cents={view.flow.expense} /> },
        { label: t('painel-base'), value: basisText(view.basis) },
      ]}
    >
      {view.topCategories.length === 0 ? (
        <p className="text-[13px] text-dim">{t('painel-categorias-vazio')}</p>
      ) : (
        <ol className="flex flex-col gap-2.5" aria-label={t('painel-categorias-lista')}>
          {view.topCategories.map((g) => {
            const color = colors.get(g.key) ?? 'var(--line-strong)';
            return (
              <li key={g.key ?? 'none'}>
                <button
                  type="button"
                  onClick={() =>
                    void navigate(txnLink(categoryDrill({ period, basis: view.basis }, [g.key])))
                  }
                  className="group flex w-full flex-col gap-1 rounded-control text-left focus-visible:outline-2 focus-visible:outline-accent"
                  aria-label={t('painel-categoria-ver', { categoria: g.label })}
                >
                  <span className="flex items-baseline justify-between gap-3 text-[13px]">
                    <span className="flex min-w-0 items-center gap-2 text-fg group-hover:text-accent">
                      <span
                        aria-hidden
                        className="size-2 shrink-0 rounded-[2px]"
                        style={{ background: color }}
                      />
                      <span className="truncate">{g.label}</span>
                    </span>
                    <span className="num shrink-0 text-fg">
                      <Money cents={g.amount} />
                      <span className="ml-2 text-[11.5px] text-dim">
                        <Private>{formatPercent(g.share, { decimals: 0 })}</Private>
                      </span>
                    </span>
                  </span>
                  <span aria-hidden className="block h-1.5 w-full rounded-[2px] bg-line/50">
                    <span
                      className="block h-full rounded-r-[4px]"
                      style={{ width: `${(g.amount / max) * 100}%`, background: color }}
                    />
                  </span>
                </button>
              </li>
            );
          })}
        </ol>
      )}
    </InstrumentCard>
  );
}

// ── 06 · Próximos vencimentos ───────────────────────────────────

const UPCOMING_LIMIT = 8;

export function UpcomingCard({ items, today }: { items: readonly UpcomingItem[]; today: string }) {
  const navigate = useNavigate();
  const shown = items.slice(0, UPCOMING_LIMIT);
  const out = useMemo(
    () => items.filter((i) => i.amount < 0).reduce((s, i) => s + i.amount, 0),
    [items],
  );
  return (
    <InstrumentCard
      index="06"
      title={t('painel-vencimentos')}
      className="@min-[1040px]:col-span-8"
      padded={false}
      footer={[
        { label: t('painel-janela'), value: t('painel-30-dias') },
        { label: t('painel-saidas-previstas'), value: <Money cents={out} /> },
        { label: t('painel-itens'), value: String(items.length) },
      ]}
    >
      {items.length === 0 ? (
        <p className="p-4 text-[13px] text-dim">{t('painel-vencimentos-vazio')}</p>
      ) : (
        <ul className="flex flex-col">
          {shown.map((i) => {
            const days = daysBetween(today, i.date);
            return (
              <li key={`${i.kind}:${i.ref_id}`} className="border-b border-line/60 last:border-b-0">
                <button
                  type="button"
                  onClick={() => void navigate(txnLink(upcomingDrill(i)))}
                  className="grid w-full grid-cols-[72px_1fr_auto] items-center gap-3 px-4 py-2 text-left hover:bg-surface-2 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-accent"
                >
                  <span className="flex flex-col">
                    <span className="num text-[12.5px] text-fg">
                      {formatDate(i.date).slice(0, 5)}
                    </span>
                    <span className="label-caps text-[10px]">
                      {days <= 0
                        ? t('painel-hoje')
                        : days === 1
                          ? t('painel-amanha')
                          : t('painel-em-dias', { n: days })}
                    </span>
                  </span>
                  <span className="flex min-w-0 flex-col">
                    <span className="truncate text-[13px] text-fg">{i.description}</span>
                    <span className="flex items-center gap-2 text-[11.5px] text-dim">
                      <span
                        className={cx(
                          'label-caps rounded-[2px] border px-1 text-[9.5px]',
                          i.kind === 'card_due'
                            ? 'border-warning/50 text-warning'
                            : 'border-line-strong',
                        )}
                      >
                        {i.kind === 'card_due' ? t('painel-fatura') : t('painel-previsto')}
                      </span>
                      <span className="truncate">{i.account_name}</span>
                    </span>
                  </span>
                  <Money
                    cents={i.amount}
                    currency={i.currency}
                    tone="auto"
                    className="text-[13px]"
                  />
                </button>
              </li>
            );
          })}
        </ul>
      )}
      {items.length > UPCOMING_LIMIT ? (
        <p className="border-t border-line px-4 py-2 text-[12px] text-dim">
          {t('painel-mais-itens', { n: items.length - UPCOMING_LIMIT })}
        </p>
      ) : null}
    </InstrumentCard>
  );
}

// ── 07 · Sem categoria ──────────────────────────────────────────

export function UncategorizedCard({ view }: { view: DashboardView }) {
  const navigate = useNavigate();
  const n = view.dashboard.uncategorized_count;
  const period = monthBounds(view.month);
  return (
    <InstrumentCard
      index="07"
      title={t('controles-sem-categoria')}
      className="@min-[1040px]:col-span-4"
      footer={[{ label: t('painel-periodo'), value: capitalize(formatMonthLong(view.month)) }]}
    >
      <div className="flex h-full flex-col items-start gap-3">
        <Measure
          size="md"
          label={t('painel-para-revisar')}
          value={<span className={n > 0 ? 'text-warning' : 'text-positive'}>{n}</span>}
          hint={n === 0 ? t('painel-tudo-categorizado') : t('painel-fora-categorias')}
        />
        {n > 0 ? (
          <Button
            size="sm"
            iconRight={<Icon name="arrowRight" size={12} />}
            onClick={() => void navigate(txnLink(uncategorizedDrill(period, view.dashboard.basis)))}
          >
            {t('painel-categorizar')}
          </Button>
        ) : null}
      </div>
    </InstrumentCard>
  );
}
