/**
 * Imposto sobre o ganho na venda de um bem (`tax.physical_gain`), como o programa GCAP da
 * Receita: ganho bruto, reduções dos imóveis (Lei 7.713 e fatores FR1/FR2), único imóvel,
 * imóvel residencial com o dinheiro aplicado em outro, pequeno valor e imposto por faixas em
 * DARF 4600. Bem vendido usa os dados da venda; bem ainda na mão simula com data, preço e
 * custos digitados.
 */
import { useState } from 'react';

import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import { usePhysicalGain } from '@/api/hooks/tax';
import { DateInput, InstrumentCard, KeyValue, Money, MoneyInput } from '@/design/components';
import { InlineError, Switch } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';

export function CapitalGainCard({ asset }: { asset: PhysicalAssetView }) {
  const f = asset.figures;
  const realEstate = asset.kind === 'real_estate' || asset.kind === 'land';
  const [date, setDate] = useState<string | null>(todayIso());
  const [price, setPrice] = useState<number | null>(f.sold ? null : f.value);
  const [costs, setCosts] = useState<number | null>(0);
  const [only, setOnly] = useState(false);
  const [reinvested, setReinvested] = useState<number | null>(0);
  const args = f.sold
    ? { asset_id: asset.id, only_property: only, reinvested: reinvested ?? 0 }
    : date && price != null
      ? {
          asset_id: asset.id,
          sale_date: date,
          sale_price: price,
          sale_costs: costs ?? 0,
          only_property: only,
          reinvested: reinvested ?? 0,
        }
      : null;
  const q = usePhysicalGain(args);
  const g = q.data;
  const r = g?.reductions;
  return (
    <InstrumentCard
      index="07.5"
      title={f.sold ? t('bens-gcap-titulo-vendido') : t('bens-gcap-titulo-simular')}
      headingLevel={3}
    >
      <div className="flex flex-col gap-3">
        {!f.sold ? (
          <div className="grid gap-2 @min-[530px]:grid-cols-3">
            <label className="flex flex-col gap-1 text-[12.5px]">
              <span className="text-dim">{t('bens-gcap-data')}</span>
              <DateInput value={date} onValueChange={setDate} aria-label={t('bens-gcap-data')} />
            </label>
            <label className="flex flex-col gap-1 text-[12.5px]">
              <span className="text-dim">{t('bens-gcap-preco')}</span>
              <MoneyInput
                value={price}
                onValueChange={setPrice}
                currency={asset.currency}
                aria-label={t('bens-gcap-preco')}
              />
            </label>
            <label className="flex flex-col gap-1 text-[12.5px]">
              <span className="text-dim">{t('bens-gcap-custos')}</span>
              <MoneyInput
                value={costs}
                onValueChange={setCosts}
                currency={asset.currency}
                aria-label={t('bens-gcap-custos')}
              />
            </label>
          </div>
        ) : null}
        {realEstate ? (
          <div className="flex flex-col gap-2">
            <Switch
              checked={only}
              onChange={setOnly}
              label={t('bens-gcap-unico')}
              description={t('bens-gcap-unico-dica')}
            />
            {asset.kind === 'real_estate' ? (
              <label className="flex flex-col gap-1 text-[12.5px]">
                <span className="text-dim">{t('bens-gcap-reaplicado')}</span>
                <MoneyInput
                  value={reinvested}
                  onValueChange={setReinvested}
                  currency={asset.currency}
                  aria-label={t('bens-gcap-reaplicado')}
                />
              </label>
            ) : null}
          </div>
        ) : null}
        <InlineError error={q.error} />
        {g && r ? (
          <KeyValue
            layout="stacked"
            items={[
              { label: t('bens-gcap-ganho-bruto'), value: <Money cents={g.gross_gain} /> },
              ...(r.lei_7713 > 0
                ? [
                    {
                      label: t('bens-gcap-lei-7713', { pct: r.lei_7713_pct }),
                      value: <Money cents={-r.lei_7713} />,
                    },
                  ]
                : []),
              ...(r.fr1 > 0
                ? [
                    {
                      label: t('bens-gcap-fr1', { pct: r.fr1_pct }),
                      value: <Money cents={-r.fr1} />,
                    },
                  ]
                : []),
              ...(r.fr2 > 0
                ? [
                    {
                      label: t('bens-gcap-fr2', { pct: r.fr2_pct }),
                      value: <Money cents={-r.fr2} />,
                    },
                  ]
                : []),
              ...(r.only_property > 0
                ? [
                    {
                      label: t('bens-gcap-isencao-unico'),
                      value: <Money cents={-r.only_property} />,
                    },
                  ]
                : []),
              ...(r.reinvestment > 0
                ? [
                    {
                      label: t('bens-gcap-isencao-reaplicacao'),
                      value: <Money cents={-r.reinvestment} />,
                    },
                  ]
                : []),
              { label: t('bens-gcap-tributavel'), value: <Money cents={g.taxable_gain} /> },
              {
                label: t('bens-gcap-imposto'),
                value: g.small_value ? (
                  t('bens-gcap-pequeno-valor')
                ) : (
                  <Money cents={g.tax} className="font-medium" />
                ),
              },
              ...(g.tax > 0
                ? [
                    {
                      label: t('bens-gcap-darf'),
                      value: t('bens-gcap-darf-valor', {
                        codigo: g.revenue_code,
                        vencimento: formatDate(g.due_date),
                      }),
                    },
                  ]
                : []),
            ]}
          />
        ) : null}
        <p className="text-[12px] text-dim">{t('bens-gcap-nota')}</p>
      </div>
    </InstrumentCard>
  );
}
