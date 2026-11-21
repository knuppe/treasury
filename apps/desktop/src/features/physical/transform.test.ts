import { describe, expect, it } from 'vitest';

import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { PhysicalHistoryPoint } from '@/api/generated/PhysicalHistoryPoint';
import { readChartTheme } from '@/charts/theme';

import { valueOption } from './chartOptions';
import { methodsFor } from './PhysicalDialogs';
import {
  acquisitionTotal,
  equityModel,
  freeLoans,
  methodText,
  ownersText,
  totalsByKind,
  valueChartData,
} from './transform';

const point = (month: string, value: number, cost: number, loan = 0): PhysicalHistoryPoint => ({
  month,
  as_of: `${month}-28`,
  cost_basis: cost,
  value,
  estimated: true,
  appraisal: null,
  loan_balance: loan,
  equity: value - loan,
});

function asset(over: Partial<PhysicalAssetView> = {}): PhysicalAssetView {
  return {
    id: 'a',
    kind: 'real_estate',
    name: 'Apê',
    member_ids: [],
    purchase_date: '2024-01-10',
    purchase_price: 100,
    currency: 'BRL',
    acquisition_costs: [
      { id: 'c1', kind: 'itbi', amount: 3, description: null },
      { id: 'c2', kind: 'registry', amount: 2, description: null },
    ],
    valuation_method: 'index',
    index_series: 'igpm',
    depreciation_rate: null,
    useful_life_years: null,
    residual_pct: null,
    fipe_vehicle_type: null,
    fipe_brand_code: null,
    fipe_model_code: null,
    fipe_year_code: null,
    fipe_code: null,
    address: null,
    description: null,
    notes: null,
    include_in_net_worth: true,
    archived: false,
    sale_date: null,
    sale_price: null,
    sale_costs: 0,
    improvements: [],
    valuations: [],
    loans: [],
    figures: { value: 120, sold: false } as PhysicalAssetView['figures'],
    ...over,
  };
}

describe('physical asset transformations', () => {
  it('describes the valuation method', () => {
    expect(methodText(asset())).toBe('corrigido pelo IGP-M');
    expect(methodText(asset({ valuation_method: 'declining', depreciation_rate: '12' }))).toBe(
      'depreciação de 12% a.a.',
    );
    expect(
      methodText(
        asset({ valuation_method: 'straight_line', useful_life_years: 5, residual_pct: '10' }),
      ),
    ).toBe('linear em 5 anos até 10%');
    expect(methodText(asset({ valuation_method: 'fipe', fipe_code: '004478-4' }))).toBe(
      'tabela FIPE 004478-4 + 12% a.a.',
    );
    expect(acquisitionTotal(asset())).toBe(5);
    expect(
      ownersText(
        ['m1', 'm2'],
        [
          { id: 'm1', name: 'Ana' },
          { id: 'm2', name: 'Bruno' },
        ],
      ),
    ).toBe('Ana e Bruno');
    expect(ownersText([], [])).toBe('Família');
  });

  it('methods by type: the core default comes first', () => {
    expect(methodsFor('real_estate')[0]).toBe('index');
    expect(methodsFor('vehicle')).toEqual(['declining', 'fipe', 'none']);
    expect(methodsFor('equipment')[0]).toBe('straight_line');
    expect(methodsFor('collectible')[0]).toBe('none');
  });

  it('series of the asset value and net worth', () => {
    const h = [
      point('2024-01', 100, 105, 80),
      point('2024-02', 101, 105, 79),
      point('2024-03', 103, 110, 78),
    ];
    const d = valueChartData(h, [
      { id: 'v', asset_id: 'a', date: '2024-02-15', value: 130, source: 'report', notes: null },
      { id: 'w', asset_id: 'a', date: '2023-12-01', value: 99, source: 'appraisal', notes: null },
    ]);
    expect(d.cost).toEqual([105, 105, 110]);
    expect(d.appraisals).toEqual([{ index: 1, date: '2024-02-15', value: 130, source: 'Laudo' }]);
    const opt = valueOption(readChartTheme('dark', false), d);
    const series = opt.series as { name: string; type: string; step?: string; data: unknown[] }[];
    expect(series.map((s) => s.name)).toEqual(['Custo acumulado', 'Valor', 'Avaliações']);
    expect(series.every((s) => s.type === 'line')).toBe(true);
    expect(series[2]!.data).toEqual([null, 130, null]);
    expect(series[0]!.step).toBe('end');
    expect(Array.isArray(opt.yAxis)).toBe(false);

    const eq = equityModel(h, 'BRL');
    expect(eq.series.map((s) => s.key)).toEqual(['equity', 'loan']);
    expect(eq.series.every((s) => s.stack === 'valor')).toBe(true);
    expect(eq.totals).toEqual([100, 101, 103]);
    expect(eq.series[0]!.values[0]! + eq.series[1]!.values[0]!).toBe(100);
  });

  it('totals by type and free contracts', () => {
    const car = asset({
      id: 'b',
      kind: 'vehicle',
      figures: { value: 50, sold: false } as PhysicalAssetView['figures'],
      loans: [
        {
          loan_id: 'loan-2',
          name: 'CDC',
          kind: 'vehicle',
          principal: 1,
          balance: 1,
          currency: 'BRL',
          archived: false,
        },
      ],
    });
    const sold = asset({
      id: 'c',
      figures: { value: 0, sold: true } as PhysicalAssetView['figures'],
    });
    expect(totalsByKind([asset(), car, sold])).toEqual([
      { kind: 'real_estate', label: 'Imóvel', count: 1, value: 120 },
      { kind: 'vehicle', label: 'Veículo', count: 1, value: 50 },
    ]);
    expect(freeLoans([{ id: 'loan-1' }, { id: 'loan-2' }], [asset(), car])).toEqual([
      { id: 'loan-1' },
    ]);
  });
});
