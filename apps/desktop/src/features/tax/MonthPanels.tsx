/**
 * Apuração mensal: a tabela dos meses do ano (resultado por grupo, imposto, IRRF, DARF e
 * situação) e o detalhe do mês escolhido — vendas por categoria, isenção de R$ 20 mil,
 * prejuízos compensados por grupo, IRRF, valores acumulados abaixo do mínimo, as vendas
 * que entraram e o botão de gerar o DARF.
 */
import { Link } from 'react-router';

import { useGenerateDarf } from '@/api/hooks/tax';
import type { MonthStatus } from '@/api/generated/MonthStatus';
import type { PoolMonth } from '@/api/generated/PoolMonth';
import type { RealizedItem } from '@/api/generated/RealizedItem';
import type { TaxMonth } from '@/api/generated/TaxMonth';
import { Button, KeyValue, Money, Private, TechTable, type TechColumn } from '@/design/components';
import { cx } from '@/design/cx';
import { Notice } from '@/features/portfolio/components/States';
import { formatQuantity } from '@/features/portfolio/decimal';
import { InlineError, SectionTitle } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatMonthLong, formatMonthShort } from '@/lib/format';
import { useUndoToast } from '@/api/hooks/undoToast';

import { CopyButton } from './CopyValue';
import { CarneLeaoCard } from './CarneLeaoCard';
import { CryptoMonthCard } from './CryptoMonthCard';
import {
  MONTH_STATUS_HINT,
  MONTH_STATUS_LABEL,
  periodLabel,
  POOL_LABEL,
  POOL_SHORT,
} from './labels';
import { copyText, poolResult, rateLabel } from './transform';

const STATUS_TONE: Record<MonthStatus, string> = {
  no_activity: 'border-line text-dim',
  no_tax: 'border-line-strong text-fg',
  below_minimum: 'border-warning/60 text-warning',
  to_pay: 'border-accent/60 text-accent',
  paid: 'border-positive/50 text-positive',
};

const STATUS_ICON: Partial<Record<MonthStatus, string>> = {
  paid: '✓',
  to_pay: '›',
  below_minimum: '≈',
};

export function MonthStatusBadge({ status }: { status: MonthStatus }) {
  return (
    <span
      title={MONTH_STATUS_HINT[status]}
      className={cx(
        'label-caps inline-flex h-5 items-center gap-1 rounded-control border px-1.5 whitespace-nowrap',
        STATUS_TONE[status],
      )}
    >
      {STATUS_ICON[status] ? <span aria-hidden>{STATUS_ICON[status]}</span> : null}
      {MONTH_STATUS_LABEL[status]}
    </span>
  );
}

export function MonthsTable({
  months,
  selected,
  onSelect,
}: {
  months: readonly TaxMonth[];
  selected: string | null;
  onSelect: (month: string) => void;
}) {
  const signed = (v: number) =>
    v === 0 ? <span className="text-dim">—</span> : <Money cents={v} tone="auto" />;
  return (
    <TechTable<TaxMonth>
      label={t('ir-secao-apuracao')}
      prefsId="Apuração mensal"
      dense
      rows={months}
      getRowId={(m) => m.month}
      selectedId={selected}
      onSelectedChange={(id) => id && onSelect(id)}
      empty={t('ir-meses-vazio')}
      columns={[
        {
          key: 'month',
          header: t('ir-mes'),
          cell: (m) => (
            <span className="flex flex-col">
              <span className="num">{formatMonthShort(m.month)}</span>
              {m.incomplete ? (
                <span className="label-caps text-warning" title={t('ir-incompleto-dica')}>
                  {t('ir-incompleto')}
                </span>
              ) : null}
            </span>
          ),
        },
        {
          key: 'stock_sales',
          header: t('ir-vendas-acoes'),
          numeric: true,
          cell: (m) =>
            m.stock_sales === 0 ? (
              <span className="text-dim">—</span>
            ) : (
              <span className="flex flex-col items-end">
                <Money cents={m.stock_sales} />
                {m.stock_exempt ? (
                  <span className="label-caps text-positive">{t('ir-isento')}</span>
                ) : null}
              </span>
            ),
        },
        {
          key: 'common',
          header: POOL_SHORT.common,
          numeric: true,
          cell: (m) => signed(poolResult(m, 'common')),
        },
        {
          key: 'day_trade',
          header: POOL_SHORT.day_trade,
          numeric: true,
          cell: (m) => signed(poolResult(m, 'day_trade')),
        },
        {
          key: 'fii',
          header: POOL_SHORT.fii,
          numeric: true,
          cell: (m) => signed(poolResult(m, 'fii')),
        },
        {
          key: 'tax',
          header: t('ir-imposto'),
          numeric: true,
          cell: (m) =>
            m.tax === 0 ? <span className="text-dim">—</span> : <Money cents={m.tax} />,
        },
        {
          key: 'irrf',
          header: 'IRRF',
          numeric: true,
          cell: (m) =>
            m.irrf === 0 ? (
              <span className="text-dim">—</span>
            ) : (
              <Money cents={m.irrf} estimated={m.irrf_estimated} />
            ),
        },
        {
          key: 'darf',
          header: 'DARF',
          numeric: true,
          cell: (m) =>
            m.darf_amount > 0 ? (
              <Money cents={m.darf_amount} className="font-medium" />
            ) : m.carried_out > 0 ? (
              <span className="text-[11.5px] text-warning">
                {withSlots(t('ir-para-proximo', { valor: slot(0) }), [
                  <Money key="v" cents={m.carried_out} />,
                ])}
              </span>
            ) : (
              <span className="text-dim">—</span>
            ),
        },
        {
          key: 'status',
          header: t('ir-situacao'),
          cell: (m) => <MonthStatusBadge status={m.status} />,
        },
      ]}
    />
  );
}

const dash = <span className="text-dim">—</span>;

const poolColumns = (): TechColumn<PoolMonth>[] => [
  {
    key: 'pool',
    header: t('ir-grupo'),
    hideable: false,
    sortValue: (p) => POOL_LABEL[p.pool],
    cell: (p) => POOL_LABEL[p.pool],
  },
  {
    key: 'result',
    header: t('ir-resultado'),
    numeric: true,
    cell: (p) => <Money cents={p.result} tone="auto" />,
  },
  {
    key: 'loss_before',
    header: t('ir-prejuizo-antes'),
    numeric: true,
    cell: (p) => <Money cents={p.loss_before} />,
  },
  {
    key: 'loss_used',
    header: t('ir-compensado'),
    numeric: true,
    cell: (p) => (p.loss_used > 0 ? <Money cents={-p.loss_used} /> : dash),
  },
  { key: 'base', header: t('ir-base'), numeric: true, cell: (p) => <Money cents={p.base} /> },
  {
    key: 'rate',
    header: t('ir-aliquota'),
    numeric: true,
    className: 'text-dim',
    sortValue: (p) => Number(p.rate),
    cell: (p) => rateLabel(p.rate),
  },
  { key: 'tax', header: t('ir-imposto'), numeric: true, cell: (p) => <Money cents={p.tax} /> },
  {
    key: 'loss_after',
    header: t('ir-prejuizo-compensar'),
    numeric: true,
    cell: (p) => <Money cents={p.loss_after} />,
  },
];

export function MonthDetail({
  month: m,
  memberId,
  onShowDarf,
}: {
  month: TaxMonth;
  memberId: string | null;
  onShowDarf: () => void;
}) {
  const generate = useGenerateDarf();
  const undoToast = useUndoToast();
  const irrfUsed = m.tax - m.due;
  return (
    <section
      aria-label={t('ir-apuracao-de', { mes: formatMonthLong(m.month) })}
      className="flex flex-col gap-4"
    >
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <h3 className="text-[16px] font-medium text-fg first-letter:uppercase">
            {formatMonthLong(m.month)}
          </h3>
          <MonthStatusBadge status={m.status} />
        </div>
        {m.status === 'to_pay' && !m.darf_id ? (
          <Button
            variant="primary"
            loading={generate.isPending}
            onClick={() =>
              generate.mutate(
                { month: m.month, ...(memberId ? { member_id: memberId } : {}) },
                {
                  onSuccess: (d) =>
                    undoToast({
                      tone: 'success',
                      title: t('ir-darf-gerado'),
                      description: t('ir-darf-gerado-texto', {
                        codigo: d.revenue_code,
                        vencimento: formatDate(d.due_date),
                      }),
                    }),
                },
              )
            }
          >
            {t('ir-gerar-darf', { valor: copyText(m.darf_amount) })}
          </Button>
        ) : m.darf_id ? (
          <Button variant="ghost" onClick={onShowDarf}>
            {t('ir-ver-darf')}
          </Button>
        ) : null}
      </div>
      <InlineError error={generate.error} />

      {m.incomplete ? (
        <Notice tone="warning" title={t('ir-incompleto-titulo')}>
          {t('ir-incompleto-texto')}{' '}
          <Link
            className="text-accent underline-offset-2 hover:underline"
            to="/carteira?aba=vendas"
          >
            {t('ir-incompleto-link')}
          </Link>
        </Notice>
      ) : null}
      {m.notes.length > 0 ? (
        <Notice title={t('ir-notas-apuracao')}>
          <ul className="flex list-disc flex-col gap-0.5 pl-4">
            {m.notes.map((n) => (
              <li key={n}>{n}</li>
            ))}
          </ul>
        </Notice>
      ) : null}

      {m.categories.length > 0 ? (
        <div className="flex flex-col gap-2">
          <SectionTitle>{t('ir-vendas-categoria')}</SectionTitle>
          <TechTable
            label={t('ir-categorias-de', { mes: formatMonthLong(m.month) })}
            prefsId="Categorias do mês"
            dense
            rows={m.categories}
            getRowId={(c) => c.category}
            columns={[
              { key: 'label', header: t('ir-categoria'), cell: (c) => c.label },
              {
                key: 'sales',
                header: t('ir-vendas'),
                numeric: true,
                cell: (c) => <Money cents={c.sales} />,
              },
              {
                key: 'result',
                header: t('ir-resultado'),
                numeric: true,
                cell: (c) => <Money cents={c.result} tone="auto" />,
              },
              {
                key: 'exempt',
                header: t('ir-ganho-isento'),
                numeric: true,
                cell: (c) =>
                  c.exempt_gain > 0 ? (
                    <Money cents={c.exempt_gain} />
                  ) : (
                    <span className="text-dim">—</span>
                  ),
              },
            ]}
          />
        </div>
      ) : null}

      {m.pools.length > 0 ? (
        <div className="flex flex-col gap-2">
          <SectionTitle aside={<span className="label-caps">{t('ir-prejuizo-mesmo-grupo')}</span>}>
            {t('ir-prejuizos-base')}
          </SectionTitle>
          <TechTable
            label={t('ir-grupos-prejuizo')}
            prefsId="Grupos de prejuízo"
            dense
            minWidth="720px"
            rows={m.pools}
            getRowId={(p) => p.pool}
            columns={poolColumns()}
          />
        </div>
      ) : null}

      <div className="grid gap-4 @min-[530px]:grid-cols-2">
        <div className="rounded-card border border-line p-3.5">
          <KeyValue
            layout="stacked"
            items={[
              { label: t('ir-imposto-mes'), value: <Money cents={m.tax} /> },
              {
                label: m.irrf_estimated ? t('ir-irrf-estimado') : t('ir-irrf-mes'),
                value: <Money cents={m.irrf} estimated={m.irrf_estimated} />,
              },
              ...(m.irrf_carried_in > 0
                ? [
                    {
                      label: t('ir-irrf-anteriores'),
                      value: <Money cents={m.irrf_carried_in} />,
                    },
                  ]
                : []),
              { label: t('ir-irrf-compensado'), value: <Money cents={-irrfUsed} /> },
              { label: t('ir-devido-mes'), value: <Money cents={m.due} /> },
              ...(m.carried_in > 0
                ? [{ label: t('ir-acumulado'), value: <Money cents={m.carried_in} /> }]
                : []),
            ]}
          />
        </div>
        <div className="flex flex-col gap-2 rounded-card border border-line p-3.5">
          <KeyValue
            layout="stacked"
            items={[
              {
                label: 'DARF',
                value:
                  m.darf_amount > 0 ? (
                    <span className="inline-flex items-center gap-2">
                      <Money cents={m.darf_amount} className="font-medium" />
                      <CopyButton
                        text={copyText(m.darf_amount)}
                        label={t('ir-valor-darf-de', { periodo: formatMonthLong(m.month) })}
                      />
                    </span>
                  ) : (
                    '—'
                  ),
              },
              { label: t('ir-codigo-receita'), value: m.revenue_code },
              { label: t('ir-periodo-apuracao'), value: periodLabel(m.month) },
              { label: t('ir-vencimento'), value: formatDate(m.due_date) },
              ...(m.carried_out > 0
                ? [{ label: t('ir-vai-mes-seguinte'), value: <Money cents={m.carried_out} /> }]
                : []),
              ...(m.irrf_carried_out > 0
                ? [
                    {
                      label: t('ir-irrf-depois'),
                      value: <Money cents={m.irrf_carried_out} />,
                    },
                  ]
                : []),
            ]}
          />
        </div>
      </div>

      {m.crypto ? <CryptoMonthCard month={m} memberId={memberId} /> : null}

      <CarneLeaoCard month={m.month} />

      {m.realized.length > 0 ? (
        <div className="flex flex-col gap-2">
          <SectionTitle>{t('ir-vendas-apuracao')}</SectionTitle>
          <TechTable<RealizedItem>
            label={t('ir-vendas-de', { mes: formatMonthLong(m.month) })}
            prefsId="Vendas do mês"
            dense
            rows={m.realized}
            getRowId={(r) => `${r.date}-${r.asset_id}-${r.trade_ids.join(',')}`}
            columns={[
              {
                key: 'date',
                header: t('ir-data'),
                cell: (r) => <span className="num">{formatDate(r.date)}</span>,
              },
              { key: 'ticker', header: t('ir-ativo'), cell: (r) => r.ticker ?? r.asset_id },
              {
                key: 'qty',
                header: t('ir-quantidade'),
                numeric: true,
                cell: (r) => <Private>{formatQuantity(r.quantity)}</Private>,
              },
              {
                key: 'sale',
                header: t('ir-venda'),
                numeric: true,
                cell: (r) => <Money cents={r.sale_value} />,
              },
              {
                key: 'cost',
                header: t('ir-custo-medio'),
                numeric: true,
                cell: (r) => <Money cents={r.cost} />,
              },
              {
                key: 'result',
                header: t('ir-resultado'),
                numeric: true,
                cell: (r) => <Money cents={r.result} tone="auto" />,
              },
            ]}
          />
        </div>
      ) : null}
    </section>
  );
}
