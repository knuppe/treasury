import { describe, expect, it } from 'vitest';

import type { IndicatorInfo } from '@/api/generated/IndicatorInfo';
import type { ResearchSyncReport } from '@/api/generated/ResearchSyncReport';
import type { ScreenQuery } from '@/api/generated/ScreenQuery';

import { bestOfRow, groupRows, rebaseCommon } from './compare';
import { summarizeReport } from './DataPanel';
import { formatAssumption } from './FairPricePanel';
import {
  compactAmount,
  formatBytes,
  formatIndicatorValue,
  formatUpside,
  unitAmount,
} from './format';
import {
  changeField,
  emptyDraft,
  filterFields,
  fromScreenQuery,
  newCondition,
  numberToInput,
  parseNumberInput,
  toScreenQuery,
  type DraftQuery,
} from './query';

const info = (key: string, unit: IndicatorInfo['unit']): IndicatorInfo => ({
  key,
  label: key,
  group: 'valuation',
  unit,
  formula: '',
  higher_is_better: null,
  applies_to: ['stock'],
});
const CATALOG = [
  info('pl', 'multiple'),
  info('dy', 'percent'),
  info('roe', 'percent'),
  info('liquidity', 'money'),
];

describe('formatação dos indicadores (frações, múltiplos, dinheiro)', () => {
  it('percentuais vêm em fração', () => {
    expect(formatIndicatorValue(0.0834, 'percent')).toBe('8,3%');
    expect(formatIndicatorValue(-0.021, 'percent')).toBe('−2,1%');
    expect(formatIndicatorValue(1.5, 'percent')).toBe('150,0%');
  });
  it('múltiplos, contagens e indisponíveis', () => {
    expect(formatIndicatorValue(5.4218, 'multiple')).toBe('5,42×');
    expect(formatIndicatorValue(-0.3, 'multiple')).toBe('−0,30×');
    expect(formatIndicatorValue(1_230_000, 'count')).toBe('1.230.000');
    expect(formatIndicatorValue(null, 'percent')).toBe('—');
    expect(formatIndicatorValue(Number.NaN, 'money', 'BRL')).toBe('—');
  });
  it('dinheiro em unidades da moeda: grande compacto, por ação com centavos', () => {
    expect(formatIndicatorValue(412_345_000_000, 'money', 'BRL')).toBe('R$ 412,3 bi');
    expect(formatIndicatorValue(9_500_000, 'money', 'USD')).toBe('USD 9,5 mi');
    expect(formatIndicatorValue(38.6, 'money', 'BRL', '38.6')).toBe('R$ 38,60');
    expect(formatIndicatorValue(0.0952, 'money', 'BRL')).toBe('R$ 0,0952');
    expect(compactAmount(-2_500_000_000, 'BRL')).toBe('−R$ 2,5 bi');
    expect(compactAmount(45_000, 'BRL')).toBe('R$ 45 mil');
    expect(unitAmount(4.1234, 'BRL')).toBe('R$ 4,12');
  });
  it('margem até o preço justo e tamanhos', () => {
    expect(formatUpside(0.123)).toBe('+12,3%');
    expect(formatUpside(-0.08)).toBe('−8,0%');
    expect(formatUpside(null)).toBe('—');
    expect(formatBytes(230_400_000)).toBe('230,4 MB');
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(1_500)).toBe('1,5 kB');
  });
  it('premissas do preço justo em pt-BR', () => {
    expect(formatAssumption('LPA (12 meses)', '5.8000', 'BRL')).toBe('R$ 5,80');
    expect(formatAssumption('Dividendos por ação em 2025', '0.0952', 'BRL')).toBe('R$ 0,0952');
    expect(formatAssumption('Ações em circulação', '13044000000', 'BRL')).toBe('13.044.000.000');
    expect(formatAssumption('Taxa mínima', '6.5%', 'BRL')).toBe('6,5%');
    expect(formatAssumption('Anos', '5', 'BRL')).toBe('5');
  });
});

describe('montador do screener ↔ ScreenQuery', () => {
  it('lê números pt-BR e converte percentuais em fração', () => {
    expect(parseNumberInput('6', true)).toBe(0.06);
    expect(parseNumberInput('6,5', true)).toBe(0.065);
    expect(parseNumberInput('1.000.000', false)).toBe(1_000_000);
    expect(parseNumberInput('−2,5', false)).toBe(-2.5);
    expect(parseNumberInput('abc', false)).toBeNull();
    expect(numberToInput(0.065, true)).toBe('6,5');
    expect(numberToInput(0.07, true)).toBe('7');
    expect(numberToInput(12.5, false)).toBe('12,5');
  });

  it('monta condições, grupos e recorte', () => {
    const d: DraftQuery = {
      logic: 'and',
      conditions: [
        { ...newCondition('pl', 'lt'), value: '10' },
        { ...newCondition('dy', 'between'), value: '6', value2: '12,5' },
        { ...newCondition('sector', 'in'), value: 'Energia, Bancos ;' },
        newCondition('roe', 'exists'),
        { ...newCondition('held', 'eq'), value: 'false' },
      ],
      groups: [
        {
          id: 'g1',
          logic: 'or',
          conditions: [
            { ...newCondition('class', 'eq'), value: 'fii' },
            { ...newCondition('liquidity', 'gte'), value: '1.000.000' },
          ],
        },
      ],
      universe: { classes: [], sectors: [' '], min_liquidity: 0, only_mine: true },
    };
    const { query, errors } = toScreenQuery(d, CATALOG);
    expect(errors).toEqual({});
    expect(query).toEqual({
      logic: 'and',
      conditions: [
        { field: 'pl', op: 'lt', value: 10 },
        { field: 'dy', op: 'between', value: [0.06, 0.125] },
        { field: 'sector', op: 'in', value: ['Energia', 'Bancos'] },
        { field: 'roe', op: 'exists' },
        { field: 'held', op: 'eq', value: false },
      ],
      groups: [
        {
          logic: 'or',
          conditions: [
            { field: 'class', op: 'eq', value: 'fii' },
            { field: 'liquidity', op: 'gte', value: 1_000_000 },
          ],
        },
      ],
      universe: { only_mine: true },
    } satisfies ScreenQuery);
    expect(filterFields(query!)).toEqual(['pl', 'dy', 'roe', 'liquidity']);
  });

  it('aponta erros por condição e grupo vazio', () => {
    const bad = { ...newCondition('pl', 'lt'), value: 'x' };
    const between = { ...newCondition('dy', 'between'), value: '10', value2: '5' };
    const empty = { id: 'gv', logic: 'and' as const, conditions: [] };
    const { query, errors } = toScreenQuery(
      { ...emptyDraft(), conditions: [bad, between], groups: [empty] },
      CATALOG,
    );
    expect(query).toBeNull();
    expect(errors[bad.id]).toMatch(/número/);
    expect(errors[between.id]).toMatch(/máximo é menor/);
    expect(errors.gv).toMatch(/Grupo vazio/);
  });

  it('volta de uma tela salva para o rascunho e ida de novo dá o mesmo filtro', () => {
    const q: ScreenQuery = {
      logic: 'or',
      conditions: [
        { field: 'dy', op: 'gte', value: 0.065 },
        { field: 'pl', op: 'between', value: [3, 8.5] },
        { field: 'segment', op: 'not_in', value: ['Bancos', 'Seguros'] },
        { field: 'ticker', op: 'contains', value: '11' },
      ],
      groups: [{ logic: 'and', conditions: [{ field: 'watched', op: 'eq', value: true }] }],
      universe: { classes: ['stock_br', 'fii'], min_liquidity: 500_000 },
    };
    const d = fromScreenQuery(q, CATALOG);
    expect(d.conditions[0]).toMatchObject({ field: 'dy', op: 'gte', value: '6,5' });
    expect(d.conditions[1]).toMatchObject({ value: '3', value2: '8,5' });
    expect(d.conditions[2]).toMatchObject({ value: 'Bancos, Seguros' });
    expect(toScreenQuery(d, CATALOG).query).toEqual(q);
  });

  it('trocar o campo mantém o operador quando ainda vale', () => {
    const c = { ...newCondition('pl', 'gte'), value: '5' };
    expect(changeField(c, 'roe')).toMatchObject({ field: 'roe', op: 'gte', value: '5' });
    expect(changeField(c, 'sector')).toMatchObject({ field: 'sector', op: 'contains', value: '' });
    expect(changeField(c, 'held')).toMatchObject({ field: 'held', op: 'eq', value: 'true' });
  });
});

describe('comparador', () => {
  it('melhor da linha com empates; sem ordem natural não destaca', () => {
    expect([...bestOfRow({ values: [5, 3, null, 3], best: 1 })]).toEqual([1, 3]);
    expect([...bestOfRow({ values: [0.1, 0.2], best: 1 })]).toEqual([1]);
    expect(bestOfRow({ values: [10, 20], best: null }).size).toBe(0);
    expect(bestOfRow({ values: [null, null], best: 0 }).size).toBe(0);
  });

  it('agrupa as linhas na ordem do painel, preço justo por último', () => {
    const row = (key: string, group: 'valuation' | 'market' | 'fair_price' | 'dividends') => ({
      key,
      label: key,
      group,
      unit: 'multiple' as const,
      values: [],
      best: null,
    });
    const g = groupRows([
      row('graham', 'fair_price'),
      row('pl', 'valuation'),
      row('dy', 'dividends'),
      row('price', 'market'),
    ]);
    expect(g.map((x) => x.group)).toEqual(['market', 'valuation', 'dividends', 'fair_price']);
  });

  it('rebaseia as cotações no primeiro dia comum (base 100)', () => {
    const r = rebaseCommon([
      {
        ticker: 'A',
        points: [
          { date: '2026-01-02', value: 100 },
          { date: '2026-01-05', value: 110 },
          { date: '2026-01-06', value: 121 },
        ],
      },
      {
        ticker: 'B',
        points: [
          { date: '2026-01-05', value: 100 },
          { date: '2026-01-06', value: 90 },
        ],
      },
      { ticker: 'C', points: [] },
    ]);
    expect(r.base).toBe('2026-01-05');
    expect(r.dates).toEqual(['2026-01-05', '2026-01-06']);
    expect(r.series).toEqual([
      { ticker: 'A', values: [100, 110] },
      { ticker: 'B', values: [100, 90] },
    ]);
    expect(r.missing).toEqual(['C']);
    expect(rebaseCommon([{ ticker: 'X', points: [] }]).base).toBeNull();
  });
});

describe('relatório da coleta', () => {
  it('agrega por fonte na ordem fixa', () => {
    const item = (
      source: 'dfp' | 'fca' | 'sec',
      status: 'updated' | 'unchanged' | 'skipped',
      bytes: number,
    ) => ({
      source,
      label: source,
      url: null,
      status,
      bytes,
      rows: 10,
      error: status === 'skipped' ? 'sem e-mail' : null,
    });
    const report: ResearchSyncReport = {
      started_at: '2026-10-14T10:00:00Z',
      finished_at: '2026-10-14T10:01:30Z',
      ok: true,
      items: [
        item('sec', 'skipped', 0),
        item('dfp', 'updated', 12e6),
        item('dfp', 'unchanged', 0),
        item('fca', 'updated', 4e5),
      ],
      bytes: 12.4e6,
      universe: 30,
      radar: 2,
      warnings: [],
    };
    const s = summarizeReport(report);
    expect(s.map((x) => x.source)).toEqual(['fca', 'dfp', 'sec']);
    expect(s[1]).toMatchObject({ files: 2, updated: 1, unchanged: 1, bytes: 12e6, rows: 20 });
    expect(s[2]).toMatchObject({ failed: 1, skipped: 0, errors: ['sem e-mail'] });
  });

  it('pulado sem aviso (plano gratuito da brapi) não conta como problema', () => {
    const report: ResearchSyncReport = {
      started_at: '2026-10-22T10:00:00Z',
      finished_at: '2026-10-22T10:00:05Z',
      ok: true,
      items: [
        {
          source: 'brapi',
          label: 'brapi.dev: a conferência é dos planos pagos',
          url: null,
          status: 'skipped',
          bytes: 0,
          rows: 0,
          error: null,
        },
      ],
      bytes: 0,
      universe: 0,
      radar: 0,
      warnings: [],
    };
    const [b] = summarizeReport(report);
    expect(b).toMatchObject({
      failed: 0,
      skipped: 1,
      notes: ['brapi.dev: a conferência é dos planos pagos'],
    });
  });
});
