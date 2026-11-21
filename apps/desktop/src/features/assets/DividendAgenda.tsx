/**
 * Agenda de dividendos (`portfolio.dividend_calendar`): data-com, pagamento, valor
 * por cota e o previsto para a família, por mês; filtro "só minha carteira" e as
 * projeções do radar marcadas como estimadas. No topo, os proventos anunciados que ainda
 * não foram lançados (proventos a confirmar).
 */
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router';

import { useDividendCalendar } from '@/api/hooks/portfolio';
import type { DividendCalendarItem } from '@/api/generated/DividendCalendarItem';
import {
  EmptyState,
  Money,
  OriginBadge,
  Private,
  TechTable,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';
import { formatMonthLong, todayIso } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';

import { Check } from '../portfolio/components/Fields';
import { Segmented } from '../portfolio/components/Segmented';
import { ErrorBlock, LoadingBlock } from '../portfolio/components/States';
import { formatQuantity, formatUnitPrice } from '../portfolio/decimal';
import { INCOME_KIND_LABEL, safeDate, sourceName } from '../portfolio/labels';
import { agendaByMonth } from '../portfolio/transform';
import { PendingIncomeBlock } from './PendingIncome';

type Range = 'past30' | 'next30' | 'next60' | 'next90';

function addDays(iso: string, days: number): string {
  const d = new Date(`${iso}T00:00:00Z`);
  d.setUTCDate(d.getUTCDate() + days);
  return d.toISOString().slice(0, 10);
}

export function rangeDates(range: Range, today: string): { from: string; to: string } {
  switch (range) {
    case 'past30':
      return { from: addDays(today, -30), to: today };
    case 'next30':
      return { from: today, to: addDays(today, 30) };
    case 'next60':
      return { from: today, to: addDays(today, 60) };
    case 'next90':
      return { from: today, to: addDays(today, 90) };
  }
}

const COLUMNS: TechColumn<DividendCalendarItem>[] = [
  {
    key: 'ex',
    get header() {
      return t('ativos-anunciados-data-com');
    },
    cell: (i) => safeDate(i.ex_date),
    sortValue: (i) => i.ex_date,
    width: '100px',
  },
  {
    key: 'pay',
    get header() {
      return t('ativos-pagamento');
    },
    cell: (i) => safeDate(i.pay_date),
    sortValue: (i) => i.pay_date,
    width: '100px',
  },
  {
    key: 'asset',
    get header() {
      return t('ativos-form-rotulo');
    },
    cell: (i) => (
      <span className="flex flex-col leading-tight">
        <span className="font-medium">{i.asset.ticker ?? i.asset.name}</span>
        <span className="truncate font-sans text-[11.5px] text-dim">{i.asset.class_label}</span>
      </span>
    ),
    sortValue: (i) => i.asset.ticker ?? i.asset.name,
  },
  {
    key: 'kind',
    get header() {
      return t('ativos-termos-tipo');
    },
    cell: (i) => <span className="font-sans">{INCOME_KIND_LABEL[i.kind]}</span>,
  },
  {
    key: 'per',
    get header() {
      return t('ativos-por-cota');
    },
    numeric: true,
    cell: (i) => (
      <span className={i.estimated ? 'italic' : undefined}>
        {i.estimated ? '≈ ' : ''}
        {formatUnitPrice(i.amount_per_share, i.currency)}
      </span>
    ),
  },
  {
    key: 'qty',
    get header() {
      return t('ativos-agenda-sua-qtd');
    },
    numeric: true,
    cell: (i) => <Private>{i.quantity ? formatQuantity(i.quantity) : '—'}</Private>,
  },
  {
    key: 'net',
    get header() {
      return t('ativos-agenda-previsto-liquido');
    },
    numeric: true,
    cell: (i) =>
      i.expected_net == null ? (
        '—'
      ) : (
        <Money cents={i.expected_net} currency={i.currency} estimated={i.estimated} />
      ),
    sortValue: (i) => i.expected_net,
  },
  {
    key: 'status',
    get header() {
      return t('ativos-watch-situacao');
    },
    cell: (i) => {
      const date = i.pay_date ?? i.ex_date ?? '';
      const past = date !== '' && date < todayIso();
      return (
        <span className="inline-flex items-center gap-1.5">
          {i.estimated ? (
            <span className="text-warning italic">{t('ativos-agenda-projecao')}</span>
          ) : past ? (
            <span className="text-dim">{t('ativos-agenda-ja-ocorreu')}</span>
          ) : (
            <span className="text-positive">{t('ativos-agenda-anunciado')}</span>
          )}
          <OriginBadge
            source={i.estimated ? t('ativos-agenda-radar') : sourceName(i.source)}
            date={i.ex_date ?? i.pay_date ?? undefined}
            status={i.estimated ? 'estimado' : 'oficial'}
            note={
              i.kind === 'jcp'
                ? t('ativos-agenda-jcp')
                : i.pay_date
                  ? undefined
                  : t('ativos-agenda-so-data-com')
            }
          />
        </span>
      );
    },
  },
];

export function DividendAgenda() {
  const navigate = useNavigate();
  const [range, setRange] = useState<Range>('next60');
  const [onlyMine, setOnlyMine] = useState(true);
  const [showEstimated, setShowEstimated] = useState(true);
  const today = todayIso();
  const { from, to } = rangeDates(range, today);
  const q = useDividendCalendar({ from, to, only_mine: onlyMine });
  const items = useMemo(
    () => (q.data ?? []).filter((i) => showEstimated || !i.estimated),
    [q.data, showEstimated],
  );
  const months = useMemo(() => agendaByMonth(items), [items]);
  const base = useBaseCurrency();
  // Previsto na moeda base (os de outra moeda ficam fora: sem câmbio aqui).
  const totalBase = items
    .filter((i) => i.currency === base)
    .reduce((s, i) => s + (i.expected_net ?? 0), 0);

  return (
    <div className="flex flex-col gap-4">
      <PendingIncomeBlock />
      <div className="flex flex-wrap items-center gap-4">
        <Segmented<Range>
          label={t('ativos-agenda-periodo')}
          value={range}
          onChange={setRange}
          options={[
            { value: 'past30', label: t('ativos-agenda-ultimos-30') },
            { value: 'next30', label: t('ativos-agenda-proximos', { n: 30 }) },
            { value: 'next60', label: t('ativos-agenda-proximos', { n: 60 }) },
            { value: 'next90', label: t('ativos-agenda-proximos', { n: 90 }) },
          ]}
        />
        <Check checked={onlyMine} onChange={setOnlyMine} label={t('ativos-agenda-so-minha')} />
        <Check
          checked={showEstimated}
          onChange={setShowEstimated}
          label={t('ativos-agenda-mostrar-projecoes')}
        />
        <span className="ml-auto flex items-baseline gap-2 text-[12.5px]">
          <span className="label-caps">{t('ativos-agenda-previsto-moeda', { moeda: base })}</span>
          <Money cents={totalBase} currency={base} />
        </span>
      </div>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : months.length === 0 ? (
        <EmptyState
          title={t('ativos-agenda-vazia')}
          description={t('ativos-agenda-vazia-descricao')}
        />
      ) : (
        months.map((m) => (
          <section
            key={m.month}
            className="flex flex-col gap-2"
            aria-label={formatMonthLong(m.month)}
          >
            <header className="flex flex-wrap items-baseline gap-4 px-1">
              <h3 className="label-caps text-fg">{formatMonthLong(m.month)}</h3>
              <span className="label-caps">
                {t('ativos-agenda-n-eventos', { n: m.items.length })}
              </span>
              <span className="ml-auto flex gap-4 text-[12.5px]">
                {[...m.totals.entries()].map(([ccy, v]) => (
                  <Money key={ccy} cents={v} currency={ccy} />
                ))}
              </span>
            </header>
            <div className="rounded-card border border-line bg-surface">
              <TechTable
                className="[&_table]:table-fixed"
                label={t('ativos-agenda-de', { mes: formatMonthLong(m.month) })}
                prefsId={`Agenda de ${formatMonthLong(m.month)}`}
                rows={m.items}
                columns={COLUMNS}
                getRowId={(i) => `${i.asset.id}:${i.kind}:${i.ex_date ?? ''}:${i.pay_date ?? ''}`}
                onActivate={(i) => void navigate(`/ativos?ativo=${encodeURIComponent(i.asset.id)}`)}
                dense
              />
            </div>
          </section>
        ))
      )}
      <p className="text-[12px] text-dim">{t('ativos-agenda-ajuda')}</p>
    </div>
  );
}
