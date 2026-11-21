/**
 * Fundamentus no ativo (ações e FIIs da B3): indicadores, dados técnicos e proventos lidos
 * do site, com o cache local de um dia. "Atualizar" pede ao site de novo.
 */
import { useFundamentus, useRefreshFundamentus } from '@/api/hooks/dataSources';
import type { FundamentusItem } from '@/api/generated/FundamentusItem';
import {
  Button,
  Icon,
  InstrumentCard,
  KeyValue,
  Private,
  TechTable,
  type TechColumn,
} from '@/design/components';
import type { FundamentusDividend } from '@/api/generated/FundamentusDividend';
import { t } from '@/i18n';
import { formatDate, formatDateTime, formatMoney } from '@/lib/format';
import { InlineError, QueryState } from '@/features/transactions/ui/controls';

import { formatDecimal } from '../portfolio/decimal';

/** Classes que o Fundamentus cobre. */
export const FUNDAMENTUS_CLASSES = ['stock_br', 'fii'] as const;

/** Os mais usados primeiro; o resto segue na ordem do site. */
const FIRST = [
  'P/L',
  'P/VP',
  'Div. Yield',
  'ROE',
  'ROIC',
  'Marg. Líquida',
  'Dív Líq / Patrim',
  'EV / EBITDA',
  'Liquidez Corr',
  'FFO Yield',
  'Vacância Média',
  'Cap Rate',
];

function ordered(items: readonly FundamentusItem[]): FundamentusItem[] {
  const rank = (l: string) => {
    const i = FIRST.indexOf(l);
    return i < 0 ? FIRST.length : i;
  };
  return [...items].sort((a, b) => rank(a.label) - rank(b.label));
}

const DIV_COLUMNS: TechColumn<FundamentusDividend>[] = [
  {
    key: 'com',
    get header() {
      return t('ativos-fundamentus-data-com');
    },
    cell: (d) => formatDate(d.date_com),
  },
  {
    key: 'kind',
    get header() {
      return t('ativos-termos-tipo');
    },
    cell: (d) => d.kind,
  },
  {
    key: 'value',
    get header() {
      return t('ativos-valor');
    },
    numeric: true,
    cell: (d) => (
      <Private>
        {formatDecimal(d.value.toFixed(8), { minDecimals: 2, maxDecimals: 8 })}
        {d.per_shares > 1 ? ` / ${d.per_shares}` : ''}
      </Private>
    ),
  },
  {
    key: 'pay',
    get header() {
      return t('ativos-pagamento');
    },
    cell: (d) => (d.pay_date ? formatDate(d.pay_date) : '—'),
  },
];

export function FundamentusPanel({ ticker }: { ticker: string }) {
  const q = useFundamentus(ticker);
  const refresh = useRefreshFundamentus();
  const s = q.data;
  return (
    <InstrumentCard
      index="08.F"
      title="Fundamentus"
      actions={
        <Button
          size="sm"
          variant="ghost"
          loading={refresh.isPending}
          icon={<Icon name="refresh" size={13} />}
          onClick={() => refresh.mutate({ ticker, refresh: true })}
        >
          {t('ativos-atualizar')}
        </Button>
      }
      footer={
        s
          ? [
              { label: t('ativos-fonte'), value: 'fundamentus.com.br' },
              {
                label: s.from_cache ? t('ativos-fundamentus-cache') : t('ativos-fundamentus-lido'),
                value: formatDateTime(s.fetched_at),
              },
            ]
          : undefined
      }
    >
      <QueryState loading={q.isLoading} error={q.error}>
        {s && !s.found ? (
          <p className="text-[13px] text-dim">{t('ativos-fundamentus-sem-papel', { ticker })}</p>
        ) : s ? (
          <div className="flex flex-col gap-5">
            <KeyValue
              items={[
                {
                  label: s.fii ? t('ativos-fundamentus-fundo') : t('ativos-fundamentus-empresa'),
                  value: s.name ?? '—',
                },
                {
                  label: s.fii ? t('ativos-campo-segmento') : t('ativos-campo-setor'),
                  value: s.sector ?? '—',
                },
                {
                  label: s.fii ? t('ativos-fundamentus-gestao') : t('ativos-fundamentus-subsetor'),
                  value: s.subsector ?? '—',
                },
                {
                  label: t('ativos-fundamentus-cotacao-site'),
                  value:
                    s.price != null
                      ? s.price_date
                        ? t('ativos-fundamentus-cotacao-em', {
                            preco: formatMoney(Math.round(s.price * 100), 'BRL'),
                            data: formatDate(s.price_date),
                          })
                        : formatMoney(Math.round(s.price * 100), 'BRL')
                      : '—',
                },
              ]}
            />
            <div>
              <p className="label-caps mb-2">{t('ativos-aba-indicadores')}</p>
              <dl className="grid grid-cols-2 gap-x-6 gap-y-2 @min-[620px]:grid-cols-3 @min-[960px]:grid-cols-4">
                {ordered(s.indicators).map((x) => (
                  <div key={x.label} className="flex min-w-0 flex-col" title={x.hint ?? undefined}>
                    <dt className="truncate text-[11.5px] text-dim">{x.label}</dt>
                    <dd className="num text-[13.5px] text-fg">{x.text || '—'}</dd>
                  </div>
                ))}
              </dl>
            </div>
            {s.oscillations.length ? (
              <div>
                <p className="label-caps mb-2">{t('ativos-fundamentus-oscilacao')}</p>
                <dl className="flex flex-wrap gap-x-6 gap-y-2">
                  {s.oscillations.map((x) => (
                    <div key={x.label} className="flex flex-col">
                      <dt className="text-[11.5px] text-dim">{x.label}</dt>
                      <dd
                        className={
                          'num text-[13.5px] ' +
                          ((x.value ?? 0) < 0 ? 'text-negative' : 'text-positive')
                        }
                      >
                        {x.text}
                      </dd>
                    </div>
                  ))}
                </dl>
              </div>
            ) : null}
            {s.dividends.length ? (
              <TechTable
                label={t('ativos-fundamentus-tabela', { ticker })}
                prefsId={`Proventos de ${ticker} no Fundamentus`}
                columns={DIV_COLUMNS}
                rows={s.dividends.slice(0, 24)}
                getRowId={(d) => `${d.date_com}-${d.kind}-${d.value}`}
              />
            ) : null}
          </div>
        ) : null}
      </QueryState>
      <InlineError error={refresh.error} />
    </InstrumentCard>
  );
}
