/**
 * Mock de `tax.capital_gain` e `tax.physical_gain`: o ganho de capital como `tax/gcap.rs`
 * (reduções da Lei 7.713 e FR1/FR2 nos imóveis, único imóvel, reaplicação, pequeno valor e
 * imposto por faixas, DARF 4600 no último dia útil do mês seguinte).
 */
import type { CapitalGain } from '../generated/CapitalGain';
import type { CapitalGainInput } from '../generated/CapitalGainInput';
import type { PhysicalAssetView } from '../generated/PhysicalAssetView';
import { registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { invalid } from './ledger/errors';
import { invoke } from './ledger/investImport';

const months = (from: string, to: string) =>
  Math.max(
    (Number(to.slice(0, 4)) - Number(from.slice(0, 4))) * 12 +
      Number(to.slice(5, 7)) -
      Number(from.slice(5, 7)) +
      1,
    0,
  );

const pct = (base: number, m: number) =>
  m <= 0 ? 0 : Number(((1 - 1 / base ** m) * 100).toFixed(6));

const apply = (v: number, p: number) => Math.trunc((v * p) / 100);

/** Último dia útil (seg–sex) do mês seguinte ao da venda. */
function dueDate(sold: string): string {
  const y = Number(sold.slice(0, 4));
  const m = Number(sold.slice(5, 7));
  const d = new Date(Date.UTC(y, m + 1, 0));
  while (d.getUTCDay() === 0 || d.getUTCDay() === 6) d.setUTCDate(d.getUTCDate() - 1);
  return d.toISOString().slice(0, 10);
}

export function capitalGain(i: CapitalGainInput): CapitalGain {
  const net = i.sale_value - Math.max(i.brokerage, 0);
  const gross = Math.max(net - i.cost, 0);
  let gain = gross;
  const r = {
    lei_7713: 0,
    fr1: 0,
    fr2: 0,
    only_property: 0,
    reinvestment: 0,
    lei_7713_pct: '0.000000',
    fr1_pct: '0.000000',
    fr2_pct: '0.000000',
  };
  if (i.kind === 'real_estate') {
    const y = Number(i.acquired_on.slice(0, 4));
    const p7713 = y <= 1969 ? 100 : y <= 1988 ? 100 - 5 * (y - 1969) : 0;
    r.lei_7713 = apply(gain, p7713);
    r.lei_7713_pct = p7713.toFixed(6);
    gain -= r.lei_7713;
    const p1 =
      i.acquired_on <= '2005-11-30'
        ? pct(
            1.006,
            months(i.acquired_on > '1996-01-01' ? i.acquired_on : '1996-01-01', '2005-11-30'),
          )
        : 0;
    r.fr1 = apply(gain, p1);
    r.fr1_pct = p1.toFixed(6);
    gain -= r.fr1;
    const p2 = pct(
      1.0035,
      months(i.acquired_on > '2005-12-01' ? i.acquired_on : '2005-12-01', i.sold_on),
    );
    r.fr2 = apply(gain, p2);
    r.fr2_pct = p2.toFixed(6);
    gain -= r.fr2;
    if (i.only_property && i.sale_value <= 44_000_000) {
      r.only_property = gain;
      gain = 0;
    } else if (i.reinvested > 0 && i.sale_value > 0) {
      r.reinvestment = Math.trunc((gain * Math.min(i.reinvested, i.sale_value)) / i.sale_value);
      gain -= r.reinvestment;
    }
  }
  const small = (i.month_sales_same_kind ?? i.sale_value) <= 3_500_000;
  let tax = 0;
  if (!small && gain > 0) {
    let prev = 0;
    for (const [top, rate] of [
      [500_000_000, 0.15],
      [1_000_000_000, 0.175],
      [3_000_000_000, 0.2],
      [Number.MAX_SAFE_INTEGER, 0.225],
    ] as const) {
      const slice = Math.min(gain, top) - prev;
      if (slice <= 0) break;
      tax += slice * rate;
      prev = top;
    }
    tax = Math.trunc(tax);
  }
  return {
    net_value: net,
    gross_gain: gross,
    reductions: r,
    taxable_gain: gain,
    small_value: small,
    tax,
    exempt: gross - (small ? 0 : gain),
    revenue_code: '4600',
    due_date: dueDate(i.sold_on),
  };
}

registerMockHandlers(
  guarded({
    'tax.capital_gain': (a) => capitalGain(a as unknown as CapitalGainInput),
    'tax.physical_gain': (a) => {
      const v = invoke<PhysicalAssetView>('physical.get', { id: a.asset_id });
      const sold = (a.sale_date as string | undefined) ?? v.sale_date;
      const price = (a.sale_price as number | undefined) ?? v.sale_price;
      if (!sold || price == null) invalid('informe a data e o valor da venda');
      return capitalGain({
        kind: v.kind === 'real_estate' || v.kind === 'land' ? 'real_estate' : 'movable',
        acquired_on: v.purchase_date,
        sold_on: sold,
        sale_value: price,
        brokerage: (a.sale_costs as number | undefined) ?? v.sale_costs,
        cost: v.figures.cost_basis,
        only_property: a.only_property === true,
        reinvested: Number(a.reinvested ?? 0),
      });
    },
  }),
);
