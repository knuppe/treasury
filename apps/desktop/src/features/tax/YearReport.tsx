/**
 * Relatório anual para a declaração: rendimentos isentos e de tributação exclusiva (com
 * o detalhe por ativo), renda variável (impostos e prejuízos a compensar em 31/12), bens
 * e direitos pelo custo e aplicações no exterior. Todo valor tem "Copiar" no formato que
 * o programa da Receita aceita colado.
 */
import type { AssetHolding } from '@/api/generated/AssetHolding';
import type { IncomeLine } from '@/api/generated/IncomeLine';
import type { TaxYear } from '@/api/generated/TaxYear';
import { InstrumentCard, KeyValue, Money, Private, TechTable } from '@/design/components';
import { formatQuantity } from '@/features/portfolio/decimal';
import { SectionTitle } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatMoney, formatMonthLong } from '@/lib/format';
import { Notice } from '@/features/portfolio/components/States';

import { AnnualReturn } from './AnnualReturn';
import { CopyButton, CopyValue } from './CopyValue';
import { POOL_LABEL } from './labels';
import { declarationLines, lossesAtYearEnd, type DeclarationLine } from './transform';

function IncomeDetail({ title, lines }: { title: string; lines: readonly IncomeLine[] }) {
  if (lines.length === 0) return null;
  return (
    <details className="group rounded-control border border-line px-3 py-2">
      <summary className="label-caps cursor-pointer select-none text-fg">
        {t('ir-n-ativos', { titulo: title, n: lines.length })}
      </summary>
      <ul className="mt-2 flex flex-col divide-y divide-line/70">
        {[...lines]
          .sort((a, b) => b.amount - a.amount)
          .map((l) => (
            <li
              key={l.asset_id}
              className="flex items-center justify-between gap-3 py-1.5 text-[13px]"
            >
              <span className="min-w-0 truncate">
                <span className="num font-medium">{l.ticker ?? '—'}</span>{' '}
                <span className="text-dim">{l.name}</span>
              </span>
              <CopyValue
                cents={l.amount}
                label={t('ir-copiar-de', { titulo: title, ativo: l.ticker ?? l.name })}
              />
            </li>
          ))}
      </ul>
    </details>
  );
}

export function YearReport({ data, memberId }: { data: TaxYear; memberId: string | null }) {
  const lines = declarationLines(data);
  const losses = lossesAtYearEnd(data);
  const y = data.year;
  return (
    <div className="flex flex-col gap-6">
      <InstrumentCard index="09.4" title={t('ir-rendimentos-ano', { ano: String(y) })} padded>
        <div className="flex flex-col gap-3">
          <TechTable<DeclarationLine>
            label={t('ir-rendimentos-declaracao')}
            prefsId="Rendimentos para a declaração"
            dense
            rows={lines}
            getRowId={(l) => l.key}
            columns={[
              { key: 'sheet', header: t('ir-ficha'), cell: (l) => l.sheet },
              {
                key: 'code',
                header: t('ir-codigo'),
                width: '80px',
                cell: (l) =>
                  l.code ? (
                    <span className="num">{l.code}</span>
                  ) : (
                    <span className="text-dim" title={t('ir-codigo-muda')}>
                      {t('ir-confira')}
                    </span>
                  ),
              },
              { key: 'label', header: t('ir-descricao'), cell: (l) => l.label },
              {
                key: 'amount',
                header: t('ir-valor'),
                numeric: true,
                cell: (l) =>
                  l.amount === 0 ? (
                    <span className="text-dim">—</span>
                  ) : (
                    <CopyValue cents={l.amount} label={l.label} />
                  ),
              },
            ]}
          />
          <div className="grid gap-2 @min-[530px]:grid-cols-3">
            <IncomeDetail title={t('ir-dividendos')} lines={data.dividends} />
            <IncomeDetail title={t('ir-linha-fii')} lines={data.fii_income} />
            <IncomeDetail title={t('ir-jcp')} lines={data.jcp} />
          </div>
          <p className="text-[12px] text-dim">{t('ir-rendimentos-nota', { ano: String(y) })}</p>
          {data.dividends_over_limit.length > 0 ? (
            <Notice tone="warning" title={t('ir-dividendos-acima-titulo')}>
              <ul className="flex list-disc flex-col gap-0.5 pl-4">
                {data.dividends_over_limit.map((d) => (
                  <li key={`${d.month}-${d.tickers.join()}`}>
                    {t('ir-dividendos-acima-linha', {
                      mes: formatMonthLong(d.month),
                      ativos: d.tickers.join(', '),
                      valor: formatMoney(d.amount, 'BRL'),
                      retencao: formatMoney(d.expected_withholding, 'BRL'),
                    })}
                  </li>
                ))}
              </ul>
              <p className="mt-1">{t('ir-dividendos-acima-nota')}</p>
            </Notice>
          ) : null}
        </div>
      </InstrumentCard>

      <InstrumentCard index="09.5" title={t('ir-resumo-renda-variavel')}>
        <div className="grid gap-6 @min-[530px]:grid-cols-2">
          <div className="flex flex-col gap-2">
            <SectionTitle>{t('ir-impostos-ano')}</SectionTitle>
            <KeyValue
              layout="stacked"
              items={[
                { label: t('ir-imposto-apurado'), value: <Money cents={data.total_tax} /> },
                {
                  label: t('ir-darfs-ano'),
                  value: <CopyValue cents={data.total_darf} label={t('ir-total-darfs')} />,
                },
                {
                  label: t('ir-ganhos-isentos'),
                  value: (
                    <CopyValue
                      cents={data.exempt_stock_gains}
                      label={t('ir-ganhos-isentos-copiar')}
                    />
                  ),
                },
                ...(data.irrf_lei_11033_left > 0
                  ? [
                      {
                        label: t('ir-irrf-sobra-declaracao'),
                        value: (
                          <CopyValue
                            cents={data.irrf_lei_11033_left}
                            label={t('ir-irrf-sobra-declaracao-rotulo')}
                          />
                        ),
                      },
                    ]
                  : []),
                ...(data.irrf_day_trade_left > 0
                  ? [
                      {
                        label: t('ir-irrf-sobra-day-trade'),
                        value: <Money cents={data.irrf_day_trade_left} />,
                      },
                    ]
                  : []),
              ]}
            />
          </div>
          <div className="flex flex-col gap-2">
            <SectionTitle>{t('ir-prejuizos-31-12')}</SectionTitle>
            <KeyValue
              layout="stacked"
              items={losses.map((l) => ({
                label: POOL_LABEL[l.pool],
                value:
                  l.amount === 0 ? (
                    '—'
                  ) : (
                    <CopyValue
                      cents={l.amount}
                      label={t('ir-prejuizo-de', { grupo: POOL_LABEL[l.pool] })}
                    />
                  ),
              }))}
            />
          </div>
        </div>
      </InstrumentCard>

      <InstrumentCard
        index="09.6"
        title={t('ir-bens-em', { ano: String(y) })}
        footer={[
          { label: t('ir-situacao'), value: t('ir-bens-situacao') },
          { label: t('ir-codigos'), value: t('ir-codigos-sugeridos') },
        ]}
      >
        <TechTable<AssetHolding>
          label={t('ir-bens-direitos')}
          prefsId="Bens e direitos"
          dense
          rows={data.holdings}
          getRowId={(h) => h.asset_id}
          empty={t('ir-bens-vazio')}
          columns={[
            {
              key: 'code',
              header: t('ir-grupo-codigo'),
              width: '96px',
              cell: (h) => <span className="num">{h.group_code}</span>,
            },
            {
              key: 'description',
              header: t('ir-discriminacao'),
              cell: (h) => (
                <span className="flex items-start justify-between gap-2">
                  <span className="min-w-0">
                    <span className="num font-medium">{h.ticker ?? h.name}</span>{' '}
                    <span className="text-[12px] text-dim">
                      <Private>{formatQuantity(h.quantity)}</Private> · {h.class_label}
                      {h.country !== 'Brasil' ? ` · ${h.country}` : ''}
                    </span>
                  </span>
                  <CopyButton
                    text={h.description}
                    label={t('ir-discriminacao-de', { ativo: h.ticker ?? h.name })}
                  />
                </span>
              ),
            },
            {
              key: 'prev',
              header: `31/12/${y - 1}`,
              numeric: true,
              cell: (h) => (
                <CopyValue
                  cents={h.cost_prev}
                  label={t('ir-situacao-em', { ano: String(y - 1), ativo: h.ticker ?? h.name })}
                />
              ),
            },
            {
              key: 'cost',
              header: `31/12/${y}`,
              numeric: true,
              cell: (h) => (
                <CopyValue
                  cents={h.cost}
                  label={t('ir-situacao-em', { ano: String(y), ativo: h.ticker ?? h.name })}
                />
              ),
            },
          ]}
          footer={{
            prev: <Money cents={data.holdings.reduce((s, h) => s + h.cost_prev, 0)} />,
            cost: <Money cents={data.holdings.reduce((s, h) => s + h.cost, 0)} />,
          }}
        />
      </InstrumentCard>

      {data.foreign.realized !== 0 || data.foreign.income !== 0 ? (
        <InstrumentCard index="09.7" title={t('ir-exterior')}>
          <KeyValue
            layout="stacked"
            items={[
              {
                label: t('ir-ganho-realizado'),
                value: <CopyValue cents={data.foreign.realized} label={t('ir-ganho-exterior')} />,
              },
              {
                label: t('ir-dividendos-juros'),
                value: (
                  <CopyValue cents={data.foreign.income} label={t('ir-rendimentos-exterior')} />
                ),
              },
              ...(data.foreign.tax_paid_abroad > 0
                ? [
                    {
                      label: t('ir-imposto-pago-exterior'),
                      value: (
                        <CopyValue
                          cents={data.foreign.tax_paid_abroad}
                          label={t('ir-imposto-pago-exterior-rotulo')}
                        />
                      ),
                    },
                    {
                      label: t('ir-credito-exterior'),
                      value: <Money cents={data.foreign.tax_credit} />,
                    },
                    ...(data.foreign.tax_without_credit > 0
                      ? [
                          {
                            label: t('ir-sem-credito-exterior'),
                            value: <Money cents={data.foreign.tax_without_credit} />,
                          },
                        ]
                      : []),
                  ]
                : []),
              ...(data.foreign.loss_carried_in > 0 || data.foreign.loss_carried_out > 0
                ? [
                    {
                      label: t('ir-prejuizo-exterior-anterior'),
                      value: <Money cents={data.foreign.loss_carried_in} />,
                    },
                    {
                      label: t('ir-prejuizo-exterior-seguinte'),
                      value: <Money cents={data.foreign.loss_carried_out} />,
                    },
                  ]
                : []),
              {
                label: t('ir-imposto-estimado-15'),
                value: <Money cents={data.foreign.estimated_tax} estimated />,
              },
            ]}
          />
          <p className="mt-3 text-[12px] text-dim">{t('ir-exterior-nota')}</p>
        </InstrumentCard>
      ) : null}
      <AnnualReturn year={data.year} memberId={memberId} />
    </div>
  );
}
