import { describe, expect, it } from 'vitest';

import { bulletGeometry, bulletStatus } from './bullet';
import { calendarBins, calendarOption, niceCeil, topDays, weekdayAverages } from './calendar';
import { cssColor, inkOn, mix, resolveColor } from './colorMath';
import { divergingOption, slopeOption, type ComparisonInput } from './comparison';
import { childShades, compositionOption, TREEMAP_FULL_LABEL, toEchartsTree } from './composition';
import { groupedBarsOption } from './groupedBarsOption';
import { categoricalPalette, sequentialRamp } from './palette';
import { nodeValues, sankeyHeight, sankeyOption, type SankeyInput } from './sankey';
import { stackedAreaOption } from './stackedAreaOption';
import { stackedBarsOption, topSegments } from './stackedBarsOption';
import type { ChartTheme } from './theme';
import { expandOccurrences, monthTicks, positionIn } from './timeline';
import { esc, tooltipHtml } from './tooltip';

const theme = (privacy = false): ChartTheme => ({
  id: 'dark',
  surface: '#0d131a',
  surface2: '#121a23',
  line: '#1e2a36',
  lineStrong: '#56687a',
  text: '#e6edf3',
  textDim: '#8b98a5',
  accent: '#4cc9f0',
  positive: '#3ddc97',
  negative: '#ff5c5c',
  warning: '#ffb020',
  fontText: 'Inter',
  fontNumeric: 'JetBrains Mono',
  series: categoricalPalette('dark'),
  sequential: sequentialRamp('dark'),
  privacy,
});

type AnyRec = Record<string, unknown>;
const series = (o: unknown, i = 0) => ((o as AnyRec).series as AnyRec[])[i]!;

describe('cores', () => {
  it('resolve tokens do tema e mantém hex de entidade', () => {
    const t = theme();
    expect(resolveColor(t, '@positive')).toBe(t.positive);
    expect(resolveColor(t, '@s2')).toBe(t.series[2]);
    expect(resolveColor(t, '@gray')).toBe(t.lineStrong);
    expect(resolveColor(t, '#123456')).toBe('#123456');
    expect(cssColor('@negative')).toBe('var(--negative)');
    expect(cssColor('@gray')).toBe('var(--line-strong)');
  });

  it('escolhe tinta legível sobre o preenchimento e mistura no mesmo matiz', () => {
    expect(inkOn('#ffffff')).toBe('#0b1218');
    expect(inkOn('#0d366b')).toBe('#ffffff');
    expect(mix('#000000', '#ffffff', 0.5)).toBe('#808080');
    expect(childShades('#3987e5', '#0d131a', 3)).toHaveLength(3);
  });

  it('escapa rótulos do usuário no tooltip', () => {
    expect(esc('<b>"x"</b>')).toBe('&lt;b&gt;&quot;x&quot;&lt;/b&gt;');
    expect(tooltipHtml(theme(), '<script>', [{ label: 'a&b', value: '1' }])).not.toContain(
      '<script>',
    );
  });
});

describe('Sankey', () => {
  const input: SankeyInput = {
    nodes: [
      { id: 'income:sal', name: 'Salário', color: '@positive', depth: 0 },
      { id: 'hub', name: 'Orçamento', color: '@gray', depth: 1 },
      { id: 'expense:mor', name: 'Moradia', color: '#3987e5', depth: 2 },
      { id: 'savings', name: 'Poupança', color: '@accent', depth: 2 },
    ],
    links: [
      { source: 'income:sal', target: 'hub', value: 1000 },
      { source: 'hub', target: 'expense:mor', value: 700 },
      { source: 'hub', target: 'savings', value: 300 },
    ],
    total: 1000,
    currency: 'BRL',
  };

  it('soma o valor de cada nó pelo maior lado', () => {
    const v = nodeValues(input);
    expect(v.get('hub')).toBe(1000);
    expect(v.get('expense:mor')).toBe(700);
  });

  it('pinta a fita pela ponta que não é o orçamento', () => {
    const t = theme();
    const s = series(sankeyOption(t, input));
    const links = s.links as { lineStyle: { color: string } }[];
    expect(links[0]!.lineStyle.color).toBe(t.positive);
    expect(links[1]!.lineStyle.color).toBe('#3987e5');
    expect(links[2]!.lineStyle.color).toBe(t.accent);
  });

  it('esconde os percentuais dos rótulos no modo privacidade', () => {
    const fmt = (
      series(sankeyOption(theme(true), input)).label as {
        formatter: (p: { name: string }) => string;
      }
    ).formatter;
    expect(fmt({ name: 'expense:mor' })).toBe('Moradia');
    const open = (
      series(sankeyOption(theme(), input)).label as { formatter: (p: { name: string }) => string }
    ).formatter;
    expect(open({ name: 'expense:mor' })).toContain('70,0%');
  });

  it('tooltip: valor, % da receita e a variação sobre o período anterior', () => {
    const withCompare: SankeyInput = {
      ...input,
      nodes: input.nodes.map((n) =>
        n.id === 'expense:mor'
          ? { ...n, compare: { previous: 500, delta: 200, deltaPct: 0.4 } }
          : n,
      ),
      shareLabel: 'Da receita',
      previousLabel: 'setembro de 2026',
    };
    type Fmt = (p: { dataType: string; name: string; data: AnyRec }) => string;
    const tip = (privacy: boolean) =>
      ((sankeyOption(theme(privacy), withCompare).tooltip as AnyRec).formatter as Fmt)({
        dataType: 'node',
        name: 'expense:mor',
        data: {},
      });
    const open = tip(false);
    expect(open).toContain('Da receita');
    expect(open).toContain('70,0%');
    expect(open).toContain('Setembro de 2026');
    expect(open).toContain('▲ +R$');
    expect(open).toContain('+40,0%');
    expect(tip(true)).not.toMatch(/R\$|%/);
    // A fita orçamento → Moradia compara a ponta Moradia.
    const edge = ((sankeyOption(theme(), withCompare).tooltip as AnyRec).formatter as Fmt)({
      dataType: 'edge',
      name: '',
      data: { source: 'hub', target: 'expense:mor', value: 700 },
    });
    expect(edge).toContain('Variação');
  });

  it('cresce a altura com a coluna mais cheia', () => {
    expect(sankeyHeight(input)).toBe(360);
    const many: SankeyInput = {
      ...input,
      nodes: Array.from({ length: 30 }, (_, i) => ({
        id: `s${i}`,
        name: '',
        color: '#000000',
        depth: 3,
      })),
    };
    expect(sankeyHeight(many)).toBe(30 * 24 + 40);
  });
});

describe('composição', () => {
  const nodes = [
    {
      id: 'a',
      name: 'Moradia',
      value: 900,
      color: '#3987e5',
      children: [
        { id: 'a1', name: 'Aluguel', value: 800, color: '#3987e5' },
        { id: 'a2', name: 'Água', value: 100, color: '#3987e5' },
      ],
    },
    { id: 'b', name: 'Outras', value: 100, color: '@gray' },
  ];

  it('subcategorias em passos do matiz da categoria, outras em cinza', () => {
    const tree = toEchartsTree(nodes, { ...theme(), surface: '#0d131a' });
    expect(tree[0]!.children![0]!.itemStyle.color).not.toBe(tree[0]!.children![1]!.itemStyle.color);
    expect(tree[1]!.itemStyle.color).toBe(theme().lineStrong);
  });

  it('rotula só blocos que comportam o texto e esconde valores na privacidade', () => {
    const fmt = (
      series(compositionOption(theme(), nodes, 'treemap', 'BRL')).label as {
        formatter: (p: { name: string; value: number }) => string;
      }
    ).formatter;
    expect(fmt({ name: 'Aluguel', value: 800 })).toContain('R$');
    expect(fmt({ name: 'Mínimo', value: (TREEMAP_FULL_LABEL / 10) * 1000 })).toBe('');
    const priv = (
      series(compositionOption(theme(true), nodes, 'treemap', 'BRL')).label as {
        formatter: (p: { name: string; value: number }) => string;
      }
    ).formatter;
    expect(priv({ name: 'Aluguel', value: 800 })).toBe('Aluguel');
  });

  it('sunburst usa a mesma árvore', () => {
    expect(series(compositionOption(theme(), nodes, 'sunburst', 'BRL')).type).toBe('sunburst');
  });
});

describe('calendário', () => {
  it('arredonda limites e calcula faixas por quantis', () => {
    expect(niceCeil(1234)).toBe(2000);
    expect(niceCeil(24_000)).toBe(25_000);
    const bins = calendarBins([0, 100, 200, 300, 400, 10_000], 4);
    expect(bins[bins.length - 1]).toBe(10_000);
    expect([...bins].sort((a, b) => a - b)).toEqual(bins);
    expect(calendarBins([0, 0], 5)).toEqual([]);
  });

  it('médias por dia da semana e dias de maior gasto', () => {
    const days = [
      { date: '2026-10-11', value: 100, count: 1 }, // domingo
      { date: '2026-10-12', value: 300, count: 2 }, // segunda
      { date: '2026-10-18', value: 300, count: 1 }, // domingo
    ];
    const avg = weekdayAverages(days);
    expect(avg[0]!.avg).toBe(200);
    expect(avg[1]!.avg).toBe(300);
    expect(topDays(days, 2).map((d) => d.date)).toEqual(['2026-10-12', '2026-10-18']);
  });

  it('esconde os limites da escala no modo privacidade', () => {
    const days = [{ date: '2026-10-01', value: 5000, count: 1 }];
    const vm = (
      calendarOption(theme(true), days, { from: '2026-10-01', to: '2026-10-31' }, 'BRL') as AnyRec
    ).visualMap as { pieces: { label: string }[] };
    expect(vm.pieces.map((p) => p.label).join(' ')).not.toContain('R$');
  });
});

describe('barras empilhadas', () => {
  const input = {
    months: ['2026-09', '2026-10'],
    bars: [
      { key: 'a', name: 'A', color: '#111111', values: [10, 10] },
      { key: 'b', name: 'B', color: '#222222', values: [5, 0] },
    ],
    lines: [{ key: 'avg', name: 'Média', color: '@text', values: [12, 13] }],
    currency: 'BRL',
  };

  it('arredonda só o segmento de cima de cada coluna', () => {
    expect(topSegments(input.bars, 2)).toEqual([1, 0]);
    const o = stackedBarsOption(theme(), input);
    const a = series(o, 0).data as { itemStyle: { borderRadius: unknown } }[];
    const b = series(o, 1).data as { itemStyle: { borderRadius: unknown } }[];
    expect(a[0]!.itemStyle.borderRadius).toBe(0);
    expect(b[0]!.itemStyle.borderRadius).toEqual([4, 4, 0, 0]);
    expect(a[1]!.itemStyle.borderRadius).toEqual([4, 4, 0, 0]);
  });

  it('um eixo só; barras ≤ 24 px; linhas de 2 px', () => {
    const o = stackedBarsOption(theme(), input) as AnyRec;
    expect(Array.isArray(o.yAxis)).toBe(false);
    expect(series(o, 0).barMaxWidth).toBe(24);
    expect((series(o, 2).lineStyle as { width: number }).width).toBe(2);
  });

  it('eixo sem valores no modo privacidade', () => {
    const o = stackedBarsOption(theme(true), input) as AnyRec;
    const fmt = (o.yAxis as { axisLabel: { formatter: (v: number) => string } }).axisLabel
      .formatter;
    expect(fmt(100_000)).toBe('');
  });
});

describe('bullet', () => {
  it('estados: sem orçamento, no ritmo, acima do ritmo, vai estourar, estourou', () => {
    expect(bulletStatus({ planned: 0, actual: 10, committed: 0, elapsed: 0.5 })).toBe('none');
    expect(bulletStatus({ planned: 100, actual: 40, committed: 0, elapsed: 0.5 })).toBe('ok');
    expect(bulletStatus({ planned: 100, actual: 70, committed: 0, elapsed: 0.5 })).toBe('pace');
    expect(bulletStatus({ planned: 100, actual: 70, committed: 40, elapsed: 0.5 })).toBe('risk');
    expect(bulletStatus({ planned: 100, actual: 120, committed: 0, elapsed: 0.5 })).toBe('over');
  });

  it('geometria: escala pelo maior entre previsto e realizado + comprometido', () => {
    const g = bulletGeometry({ planned: 100, actual: 50, committed: 50, elapsed: 0.25 });
    expect(g.actual).toBe(50);
    expect(g.committed).toBe(50);
    expect(g.planned).toBe(100);
    expect(g.pace).toBe(25);
    const over = bulletGeometry({ planned: 100, actual: 200, committed: 0, elapsed: 1 });
    expect(over.planned).toBe(50);
    expect(over.pace).toBeNull();
    expect(over.ratio).toBe(2);
  });
});

describe('comparação', () => {
  const rows: ComparisonInput[] = [
    { key: 'a', label: 'A', a: 100, b: 150, color: '#111111' },
    { key: 'b', label: 'B', a: 100, b: 60, color: '#222222' },
  ];
  const labels = { a: 'set', b: 'out', currency: 'BRL' };

  it('barras divergentes: subir é --negative, cair é --positive, ponta do lado certo', () => {
    const t = theme();
    const data = series(divergingOption(t, rows, labels)).data as {
      value: number;
      itemStyle: { color: string; borderRadius: number[] };
    }[];
    // Ordem invertida: a maior variação fica no topo.
    expect(data[1]!.value).toBe(50);
    expect(data[1]!.itemStyle.color).toBe(t.negative);
    expect(data[0]!.itemStyle.color).toBe(t.positive);
    expect(data[0]!.itemStyle.borderRadius).toEqual([4, 0, 0, 4]);
  });

  it('slope: uma linha por categoria na cor da entidade', () => {
    const o = slopeOption(theme(), rows, labels);
    expect(series(o, 0).data).toEqual([100, 150]);
    expect(series(o, 1).color).toBe('#222222');
  });
});

describe('barras agrupadas', () => {
  it('primeira categoria no topo (ECharts desenha de baixo para cima)', () => {
    const o = groupedBarsOption(theme(), {
      rows: [
        { key: 'x', label: 'X' },
        { key: 'y', label: 'Y' },
      ],
      groups: [{ key: 'ana', name: 'Ana', color: '#3987e5', values: [10, 20] }],
      currency: 'BRL',
    }) as AnyRec;
    expect((o.yAxis as { data: string[] }).data).toEqual(['Y', 'X']);
    expect(series(o).data).toEqual([20, 10]);
  });
});

describe('área empilhada', () => {
  it('empilha bandas e deixa a linha solta, sem preenchimento', () => {
    const o = stackedAreaOption(theme(), {
      months: ['2026-09', '2026-10'],
      currency: 'BRL',
      series: [
        { key: 'a', name: 'Contas', color: '@s0', values: [1, 2], stack: 'ativos' },
        { key: 'n', name: 'Líquido', color: '@text', values: [1, 2] },
      ],
    });
    expect(series(o, 0).stack).toBe('ativos');
    expect(series(o, 0).areaStyle).toBeDefined();
    expect(series(o, 1).areaStyle).toBeUndefined();
  });
});

describe('linha do tempo', () => {
  it('expande regras mensais, semanais e anuais', () => {
    expect(
      expandOccurrences({ freq: 'MONTHLY', by_month_day: 31 }, '2026-10-31', '2027-01-31'),
    ).toEqual(['2026-10-31', '2026-11-30', '2026-12-31', '2027-01-31']);
    expect(
      expandOccurrences(
        { freq: 'MONTHLY', interval: 2, by_month_day: 5 },
        '2026-10-05',
        '2027-02-28',
      ),
    ).toEqual(['2026-10-05', '2026-12-05', '2027-02-05']);
    expect(
      expandOccurrences({ freq: 'WEEKLY', by_day: ['MO', 'TH'] }, '2026-10-15', '2026-10-27'),
    ).toEqual(['2026-10-15', '2026-10-19', '2026-10-22', '2026-10-26']);
    expect(
      expandOccurrences(
        { freq: 'YEARLY', by_month: 3, by_month_day: 10 },
        '2027-03-10',
        '2029-01-01',
      ),
    ).toEqual(['2027-03-10', '2028-03-10']);
  });

  it('posição na janela e marcas de mês', () => {
    expect(positionIn('2026-10-01', '2026-10-01', '2026-10-31')).toBe(0);
    expect(positionIn('2026-10-31', '2026-10-01', '2026-10-31')).toBe(100);
    expect(monthTicks('2026-10-14', '2027-01-12')).toEqual([
      '2026-11-01',
      '2026-12-01',
      '2027-01-01',
    ]);
  });
});

describe('resposta ao mouse', () => {
  it('tooltip sem transição e destaque sem animação, sem passar por cima do gráfico', async () => {
    const { responsive } = await import('./EChart');
    const { baseOption, TOOLTIP_CSS } = await import('./theme');
    const { readChartTheme } = await import('./theme');
    const t = readChartTheme('dark', false);
    const base = baseOption(t) as { tooltip: Record<string, unknown>; stateAnimation: unknown };
    expect(base.tooltip.transitionDuration).toBe(0);
    expect(base.stateAnimation).toEqual({ duration: 0 });
    expect(TOOLTIP_CSS).not.toContain('backdrop-filter');
    // O rótulo acessível é o do app, não a descrição automática (em inglês) do ECharts.
    expect((base as { aria?: { enabled: boolean } }).aria?.enabled).toBe(false);

    const out = responsive({ tooltip: { trigger: 'item' }, series: [] });
    expect(out.tooltip).toMatchObject({ trigger: 'item', transitionDuration: 0 });
    expect(out.stateAnimation).toEqual({ duration: 0 });
    // Quem define o próprio valor fica com ele.
    const own = responsive({
      tooltip: [{ transitionDuration: 0.2 }],
      stateAnimation: { duration: 50 },
    });
    expect(own.tooltip).toEqual([{ transitionDuration: 0.2 }]);
    expect(own.stateAnimation).toEqual({ duration: 50 });
    expect(responsive({ series: [] }).tooltip).toBeUndefined();
  });

  it('sem retângulos sujos no init (apagavam fitas do Sankey no hover)', async () => {
    const { INIT_OPTS } = await import('./EChart');
    expect(INIT_OPTS.useDirtyRect).toBe(false);
  });
});
