/**
 * Instrumentos do topo da carteira: valor de mercado e custo, resultado (não
 * realizado, realizado e proventos), retorno anual XIRR nominal × real com o
 * medidor de inflação, e renda passiva dos últimos 12 meses.
 */
import type { PassiveIncome } from '@/api/generated/PassiveIncome';
import type { PortfolioSummary } from '@/api/generated/PortfolioSummary';
import { InstrumentCard, Measure, Money } from '@/design/components';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { InflationMeter } from './components/InflationMeter';
import { ResultCard } from './components/ResultCard';
import { Pct } from './components/Pct';
import { useDeflatorLabel } from './useDeflatorLabel';
import { meterScale } from './transform';

export function SummaryInstruments({
  summary: s,
  passive,
}: {
  summary: PortfolioSummary;
  passive?: PassiveIncome;
}) {
  const deflatorLabel = useDeflatorLabel();
  const unrealizedPct = s.cost > 0 ? s.unrealized / s.cost : null;
  const realMissing = s.deflator !== 'none' && s.xirr != null && s.xirr_real == null;
  const origin = {
    source: s.missing_prices.length
      ? t('carteira-resumo-fonte-sem-cotacao', { n: s.missing_prices.length })
      : t('carteira-resumo-fonte'),
    date: s.as_of,
    status: s.estimated ? ('estimado' as const) : ('oficial' as const),
    note: s.estimated ? t('carteira-resumo-estimado') : undefined,
  };
  return (
    <div className="grid gap-4 @min-[530px]:grid-cols-2 @min-[1040px]:grid-cols-4">
      <InstrumentCard
        index="01"
        title={t('carteira-valor-mercado')}
        footer={[
          { label: t('carteira-custo'), value: <Money cents={s.cost} /> },
          { label: t('carteira-em'), value: formatDate(s.as_of) },
        ]}
      >
        <Measure
          size="md"
          label={t('carteira-patrimonio-investido')}
          origin={origin}
          value={<Money cents={s.market_value} estimated={s.estimated} />}
          changeKey={s.market_value}
          delta={<Money cents={s.unrealized} tone="auto" trend />}
          hint={tRich('carteira-nao-realizado-pct', {
            pct: unrealizedPct != null ? <Pct value={unrealizedPct} signed /> : null,
          })}
        />
      </InstrumentCard>

      <ResultCard
        index="02"
        title={t('carteira-resultado')}
        perspective={false}
        deflator={s.deflator}
        nominal={s.total_result}
        real={s.real_result}
        effect={s.inflation_effect}
        cost={s.cost}
        estimated={s.estimated}
        parts={[
          { label: t('carteira-nao-realizado'), value: <Money cents={s.unrealized} tone="auto" /> },
          { label: t('carteira-realizado'), value: <Money cents={s.realized} tone="auto" /> },
          { label: t('carteira-proventos'), value: <Money cents={s.income} tone="auto" /> },
        ]}
      />

      <InstrumentCard
        index="03"
        title={t('carteira-xirr-titulo')}
        footer={
          realMissing ? (
            <p className="text-[11.5px] leading-snug text-warning">
              {s.deflator === 'ipca'
                ? t('carteira-xirr-sem-dados-ipca', {
                    deflator: deflatorLabel(s.deflator),
                    data: formatDate(s.as_of),
                  })
                : t('carteira-xirr-sem-dados', {
                    deflator: deflatorLabel(s.deflator),
                    data: formatDate(s.as_of),
                  })}
            </p>
          ) : (
            [
              { label: t('carteira-deflator'), value: deflatorLabel(s.deflator) },
              { label: t('carteira-metodo'), value: t('carteira-metodo-xirr') },
            ]
          )
        }
      >
        <div className="flex flex-col gap-3">
          <div className="flex flex-wrap items-end gap-x-6 gap-y-2">
            <Measure
              size="md"
              label={t('carteira-nominal')}
              value={<Pct value={s.xirr} signed />}
              changeKey={s.xirr ?? 0}
            />
            {s.deflator !== 'none' ? (
              <Measure
                size="md"
                label={t('carteira-real-de', { deflator: deflatorLabel(s.deflator) })}
                value={<Pct value={s.xirr_real} signed estimated={s.estimated} />}
                changeKey={s.xirr_real ?? 0}
              />
            ) : null}
          </div>
          <InflationMeter
            nominal={s.xirr}
            real={s.deflator === 'none' ? s.xirr : s.xirr_real}
            scale={meterScale([s.xirr, s.xirr_real])}
            deflator={deflatorLabel(s.deflator)}
            showValues={false}
          />
        </div>
      </InstrumentCard>

      <InstrumentCard
        index="04"
        title={t('carteira-renda-12m')}
        footer={
          passive
            ? [
                { label: t('carteira-media-mensal'), value: <Money cents={passive.monthly_avg} /> },
                {
                  label: t('carteira-yield'),
                  value: <Pct value={passive.portfolio_yield_12m} decimals={2} />,
                },
              ]
            : undefined
        }
      >
        <Measure
          size="md"
          label={t('carteira-proventos-liquidos')}
          value={passive ? <Money cents={passive.last_12m} /> : '—'}
          changeKey={passive?.last_12m ?? 0}
          hint={
            passive
              ? t('carteira-recebidos-moeda', { moeda: s.base_currency })
              : t('carteira-carregando-curto')
          }
        />
      </InstrumentCard>
    </div>
  );
}
