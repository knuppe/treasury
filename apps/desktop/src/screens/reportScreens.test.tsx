/**
 * Telas 01 · Painel, 05 · Orçamento e Relatórios renderizadas contra o mock: dados,
 * explicação das medidas, edição do orçamento, metas, filtros e drill-down.
 */
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCallLog } from '@/api/mock';
import { VIEWS } from '@/features/reports/model/period';
import { formatMonthLong, todayIso } from '@/lib/format';
import { monthBounds, monthOf, shiftMonth } from '@/lib/months';
import { chartOption, chartProps, clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

const month = monthOf(todayIso());

beforeEach(() => clearCharts());

function hashSearch(): URLSearchParams {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '');
}

describe('01 · Painel', () => {
  it('mostra os cards de instrumento com dados do mock', async () => {
    renderApp('unlocked', {}, '#/painel');
    expect(
      await screen.findByRole('heading', { level: 2, name: 'Patrimônio líquido' }),
    ).toBeTruthy();
    for (const title of [
      'Fluxo do mês',
      'Orçamento do mês',
      'Carteira',
      'Principais categorias',
      'Próximos vencimentos',
      'Sem categoria',
    ]) {
      expect(screen.getByRole('heading', { level: 2, name: title })).toBeTruthy();
    }
    expect(await screen.findByText(/Sem câmbio para EUR/)).toBeTruthy();
    expect(screen.getAllByText('Investimentos').length).toBeGreaterThan(0);
    expect(await screen.findByText('XIRR nominal')).toBeTruthy();
    expect(screen.getByRole('button', { name: /Categorizar agora/ })).toBeTruthy();
  });

  it('explica a medida: fórmula e fontes', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await user.click(await screen.findByRole('button', { name: /^Explicar Despesas de/ }));
    const dialog = await screen.findByRole('dialog');
    expect(await within(dialog).findByText(/lançamentos de despesa efetivados/)).toBeTruthy();
    expect(within(dialog).getByRole('grid', { name: 'Fontes do valor' })).toBeTruthy();
    expect(mockCallLog()).toContain('reports.explain');
    await user.click(within(dialog).getByRole('button', { name: 'Ver lançamentos' }));
    await waitFor(() => expect(window.location.hash).toMatch(/^#\/lancamentos\?/));
    expect(hashSearch().get('kind')).toBe('expense');
  });

  it('troca para caixa: refaz o fluxo na outra base', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('heading', { level: 2, name: 'Fluxo do mês' });
    await user.click(screen.getByRole('radio', { name: 'Caixa' }));
    await waitFor(() => expect(hashSearch().get('base')).toBe('cash'));
    await waitFor(() => expect(mockCallLog()).toContain('reports.month_flow'));
    expect((await screen.findAllByText('caixa', { selector: 'dd' })).length).toBeGreaterThan(0);
  });

  it('lançamentos sem categoria abrem a tela filtrada', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await user.click(await screen.findByRole('button', { name: /Categorizar agora/ }));
    await waitFor(() => expect(hashSearch().get('uncategorized')).toBe('1'));
    expect(hashSearch().get('from')).toBe(monthBounds(month).from);
  });
});

describe('05 · Orçamento', () => {
  it('edita o previsto na linha (Enter grava) e Esc cancela', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento');
    await user.click(await screen.findByRole('button', { name: 'Editar previsto de Moradia' }));
    const input = screen.getByRole('textbox', { name: 'Previsto de Moradia' });
    await user.clear(input);
    await user.type(input, '6.000,00{Enter}');
    await waitFor(() => expect(mockCallLog()).toContain('budget.set_lines'));
    const cell = await screen.findByRole('button', { name: 'Editar previsto de Moradia' });
    await waitFor(() => expect(cell.textContent).toContain('6.000,00'));
    expect(document.activeElement).toBe(cell);

    const before = mockCallLog().filter((c) => c === 'budget.set_lines').length;
    await user.click(screen.getByRole('button', { name: 'Editar previsto de Transporte' }));
    await user.type(screen.getByRole('textbox', { name: 'Previsto de Transporte' }), '9{Escape}');
    expect(screen.queryByRole('textbox', { name: 'Previsto de Transporte' })).toBeNull();
    expect(mockCallLog().filter((c) => c === 'budget.set_lines').length).toBe(before);
  });

  it('campo vazio não apaga o previsto', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento');
    await user.click(await screen.findByRole('button', { name: 'Editar previsto de Moradia' }));
    await user.clear(screen.getByRole('textbox', { name: 'Previsto de Moradia' }));
    await user.keyboard('{Enter}');
    expect(mockCallLog()).not.toContain('budget.set_lines');
  });

  it('mostra categorias com gasto e sem orçamento, e copia do mês anterior', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento');
    expect(
      await screen.findByRole('region', { name: 'Categorias com gasto e sem orçamento' }),
    ).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Próximo mês' }));
    expect(
      (await screen.findAllByText(/Sem orçamento nem gasto|Comprometido/)).length,
    ).toBeGreaterThan(0);
    await user.click(screen.getByRole('button', { name: /Copiar do mês anterior/ }));
    const dialog = await screen.findByRole('dialog');
    await user.click(within(dialog).getByRole('button', { name: 'Copiar' }));
    await waitFor(() => expect(mockCallLog()).toContain('budget.copy'));
    expect(await screen.findByRole('button', { name: 'Editar previsto de Moradia' })).toBeTruthy();
    expect(hashSearch().get('mes')).toBe(shiftMonth(month, 1));
  });

  it('sugere pela média e aplica', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento');
    await user.click(await screen.findByRole('button', { name: /Sugerir pela média/ }));
    const dialog = await screen.findByRole('dialog');
    expect(await within(dialog).findByRole('table', { name: 'Sugestão pela média' })).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: /^Aplicar/ }));
    await waitFor(() => expect(mockCallLog()).toContain('budget.set_lines'));
  });

  it('metas: cria uma meta com conta vinculada', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento?aba=metas');
    expect(
      await screen.findByRole('heading', { level: 2, name: 'Reserva de emergência' }),
    ).toBeTruthy();
    expect(screen.getAllByText('No caminho').length).toBeGreaterThan(0);
    await user.click(screen.getByRole('button', { name: /Nova meta/ }));
    const dialog = await screen.findByRole('dialog');
    await user.click(within(dialog).getByRole('button', { name: 'Criar meta' }));
    expect(within(dialog).getByText('Informe o nome da meta.')).toBeTruthy();
    await user.type(within(dialog).getByLabelText('Nome'), 'Intercâmbio');
    await user.type(within(dialog).getByLabelText('Valor-alvo'), '25.000,00');
    await user.click(within(dialog).getByRole('checkbox', { name: /Poupança Caixa/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Criar meta' }));
    expect(await screen.findByRole('heading', { level: 2, name: 'Intercâmbio' })).toBeTruthy();
    expect(mockCallLog()).toContain('goals.create');

    // Com rendimento: o aporte conta os juros e o cartão mostra a divisão.
    await user.click(screen.getByRole('button', { name: /Nova meta/ }));
    const second = await screen.findByRole('dialog');
    await user.type(within(second).getByLabelText('Nome'), 'Primeiro milhão');
    await user.type(within(second).getByLabelText('Valor-alvo'), '1.000.000,00');
    await user.type(within(second).getByLabelText('Data-alvo'), '01/10/2046');
    await user.type(within(second).getByLabelText('Rendimento real ao ano'), '4');
    await user.click(within(second).getByRole('button', { name: 'Criar meta' }));
    expect(await screen.findByRole('heading', { level: 2, name: 'Primeiro milhão' })).toBeTruthy();
    expect(await screen.findByText(/de rendimento\. Sem rendimento, seriam/)).toBeTruthy();
  });

  it('reserva de emergência: meses-alvo ajustáveis', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento?aba=reserva');
    expect(await screen.findByText('Meta da reserva')).toBeTruthy();
    // Com gastos anuais no ano, a conta usa o custo de vida (os anuais divididos por 12).
    const before = screen.getByText(/× custo de vida mensal/).textContent;
    await user.click(screen.getByRole('radio', { name: '12' }));
    await waitFor(() =>
      expect(screen.getByText(/× custo de vida mensal/).textContent).not.toBe(before),
    );
    expect(screen.getByText('12 × custo de vida mensal')).toBeTruthy();
    expect(screen.getByRole('list', { name: 'Gastos anuais' })).toBeTruthy();
    await user.click(screen.getByRole('checkbox', { name: 'Incluir gastos anuais' }));
    expect(await screen.findByText('12 × gasto essencial')).toBeTruthy();
  });
});

describe('Relatórios', () => {
  it('fluxo: o nó do Sankey e cada célula do comparativo abrem o analítico exato', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios');
    expect(await screen.findByText(/Fig\. 01/)).toBeTruthy();
    const sankey = chartProps(/^Fluxo de/);
    const option = chartOption(/^Fluxo de/);
    expect((option.series as { type: string }[])[0]!.type).toBe('sankey');
    act(() => sankey.onClick?.({ name: 'expense:cat-moradia', dataIndex: 0, value: 0, data: {} }));
    let dialog = await screen.findByRole('dialog', { name: /Analítico · Moradia/ });
    expect(await within(dialog).findByText(/igual ao valor de/)).toBeTruthy();
    await user.keyboard('{Escape}');
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    // O mês anterior vem antes do atual (a variação se lê da esquerda para a direita).
    const table = screen.getByRole('grid', { name: /^Comparativo do fluxo/ });
    const headers = within(table)
      .getAllByRole('columnheader')
      .map((h) => (h.textContent ?? '').toLowerCase());
    const at = (m: string) =>
      headers.findIndex((h) => h.includes(formatMonthLong(m).toLowerCase()));
    expect(at(shiftMonth(month, -1))).toBeGreaterThan(-1);
    expect(at(shiftMonth(month, -1))).toBeLessThan(at(month));

    // A célula do mês anterior: os lançamentos daquele mês, somando o valor da célula.
    const [previousCell] = screen.getAllByRole('button', { name: /ver o que compõe Moradia em/ });
    const value = previousCell!.querySelector('[data-money]')!.textContent;
    await user.click(previousCell!);
    dialog = await screen.findByRole('dialog', { name: /Analítico · Moradia/ });
    await within(dialog).findByText(/igual ao valor de/);
    const selected = within(dialog).getByRole('tab', { selected: true });
    expect(selected.textContent).toContain(value);
    const items = within(dialog).getByRole('grid', { name: /O que compõe Moradia em/ });
    const rows = within(items).getAllByRole('row').slice(1, -1);
    expect(rows.length).toBeGreaterThan(0);
    expect(within(items).getAllByRole('row').at(-1)!.textContent).toContain(value);

    // A diferença: por categoria e por descrição, com o total da linha.
    await user.click(within(dialog).getByRole('tab', { name: /Diferença/ }));
    const delta = within(dialog).getByRole('grid', { name: /Diferença de Moradia por categoria/ });
    expect(within(delta).getAllByRole('row').length).toBeGreaterThan(2);
    await user.click(within(dialog).getByRole('radio', { name: 'Descrição' }));
    expect(within(dialog).getByRole('grid', { name: /por descrição/ })).toBeTruthy();
    await user.keyboard('{Escape}');

    // Sobra: as linhas que entram na conta abrem o analítico delas.
    const [savingsCell] = screen.getAllByRole('button', { name: /ver o que compõe Sobra em/ });
    await user.click(savingsCell!);
    dialog = await screen.findByRole('dialog', { name: /Analítico · Sobra/ });
    const parts = await within(dialog).findByRole('grid', { name: /O que compõe Sobra em/ });
    const first = within(parts).getAllByRole('row')[1]!;
    await user.dblClick(first);
    expect(await screen.findByRole('button', { name: 'Voltar' })).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(await screen.findByRole('dialog', { name: /Analítico · Sobra/ })).toBeTruthy();
  });

  it('fluxo: o analítico corrige a categoria na linha e abre o lançamento', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios');
    await screen.findByText(/Fig\. 01/);
    const [previousCell] = screen.getAllByRole('button', { name: /ver o que compõe Moradia em/ });
    await user.click(previousCell!);
    const dialog = await screen.findByRole('dialog', { name: /Analítico · Moradia/ });
    await within(dialog).findByText(/igual ao valor de/);
    const before = within(dialog).getByRole('tab', { selected: true }).textContent;
    const grid = within(dialog).getByRole('grid', { name: /O que compõe Moradia em/ });
    const [picker] = within(grid).getAllByRole('button', { name: /^Categoria de .*: Moradia/ });
    await user.click(picker!);
    await user.type(screen.getByRole('combobox', { name: /Buscar categoria de/ }), 'Lazer');
    await user.keyboard('{Enter}');
    expect(await screen.findByText('Categoria alterada')).toBeTruthy();
    // O lançamento saiu de Moradia: o valor do mês e a soma conferida mudam juntos.
    await waitFor(() =>
      expect(within(dialog).getByRole('tab', { selected: true }).textContent).not.toBe(before),
    );
    expect(await within(dialog).findByText(/igual ao valor de/)).toBeTruthy();

    // Na diferença, a linha escolhida mostra os lançamentos dela nos dois meses, editáveis.
    await user.click(within(dialog).getByRole('tab', { name: /Diferença/ }));
    const detail = await within(dialog).findByRole('grid', { name: /^Lançamentos de / });
    const [edit] = within(detail).getAllByRole('button', { name: /^Editar / });
    await user.click(edit!);
    expect(await screen.findByRole('dialog', { name: /Editar lançamento/ })).toBeTruthy();
  });

  it('cada visualização responde à sua pergunta', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios');
    await screen.findByText(/Fig\. 01/);
    for (const [i, v] of VIEWS.entries()) {
      await user.click(screen.getByRole('tab', { name: new RegExp(v.label) }));
      await waitFor(() => expect(hashSearch().get('v') ?? 'fluxo').toBe(v.id));
      expect(
        await screen.findByText(new RegExp(`Fig\\. ${String(i + 1).padStart(2, '0')}`)),
      ).toBeTruthy();
    }
  });

  it('composição: treemap ↔ sunburst, tabela e drill-down com filtro de titular', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios?v=composicao&member=mem-ana');
    await screen.findByText(/Fig\. 02/);
    expect((chartOption(/^Composição das despesas/).series as { type: string }[])[0]!.type).toBe(
      'treemap',
    );
    await user.click(screen.getByRole('radio', { name: 'Sunburst' }));
    await waitFor(() =>
      expect((chartOption(/^Composição das despesas/).series as { type: string }[])[0]!.type).toBe(
        'sunburst',
      ),
    );
    await user.click(screen.getByRole('button', { name: 'Ver tabela' }));
    expect(screen.getByRole('grid', { name: 'Composição por categoria' })).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Ver gráfico' }));
    act(() =>
      chartProps(/^Composição das despesas/).onClick?.({
        name: 'Moradia',
        dataIndex: 0,
        value: 0,
        data: { id: 'cat-moradia' },
      }),
    );
    await waitFor(() => expect(hashSearch().get('category')).toBe('cat-moradia'));
    expect(hashSearch().get('member')).toBe('mem-ana');
  });

  it('ritmo e tendência: clique no dia e no segmento', async () => {
    renderApp('unlocked', {}, '#/relatorios?v=ritmo');
    await screen.findByText(/Fig\. 03/);
    act(() =>
      chartProps(/^Calendário de gastos/).onClick?.({
        name: '',
        dataIndex: 0,
        value: 0,
        data: ['2026-09-08', 1000],
      }),
    );
    await waitFor(() => expect(hashSearch().get('from')).toBe('2026-09-08'));
    expect(hashSearch().get('to')).toBe('2026-09-08');
  });

  it('tendência: barras na ordem fixa, média móvel e receitas no mesmo eixo', async () => {
    renderApp('unlocked', {}, '#/relatorios?v=tendencia');
    await screen.findByText(/Fig\. 04/);
    const option = chartOption(/^Despesas por categoria em/);
    const series = option.series as { type: string; name: string }[];
    expect(series[0]!.name).toBe('Moradia');
    expect(series.filter((s) => s.type === 'line').map((s) => s.name)).toEqual([
      'Média móvel (3 meses)',
      'Receitas',
    ]);
    expect(Array.isArray(option.yAxis)).toBe(false);
    act(() =>
      chartProps(/^Despesas por categoria em/).onClick?.({
        name: '',
        seriesName: 'Moradia',
        dataIndex: 11,
        value: 0,
        data: {},
      }),
    );
    await waitFor(() => expect(hashSearch().get('category')).toBe('cat-moradia'));
    expect(hashSearch().get('from')).toBe(monthBounds(month).from);
  });

  it('filtros na URL: período personalizado e base', async () => {
    const user = userEvent.setup();
    renderApp(
      'unlocked',
      {},
      '#/relatorios?v=fluxo&p=personalizado&from=2026-03-01&to=2026-03-31&basis=cash',
    );
    expect(await screen.findByText(/Fig\. 01/)).toBeTruthy();
    expect(screen.getByLabelText('Data inicial')).toBeTruthy();
    expect(screen.getByRole('radio', { name: 'Caixa' }).getAttribute('aria-checked')).toBe('true');
    await user.click(screen.getByRole('radio', { name: 'Competência' }));
    await waitFor(() => expect(hashSearch().get('basis')).toBe('accrual'));
    expect(hashSearch().get('from')).toBe('2026-03-01');
  });

  it('privacidade: tooltips e eixos sem valores', async () => {
    renderApp('unlocked', {}, '#/relatorios?v=tendencia');
    await screen.findByText(/Fig\. 04/);
    const option = chartOption(/^Despesas por categoria em/, true);
    const fmt = (option.yAxis as { axisLabel: { formatter: (v: number) => string } }).axisLabel
      .formatter;
    expect(fmt(1_000_000)).toBe('');
    const tooltip = (
      option.tooltip as { formatter: (p: { dataIndex: number }[]) => string }
    ).formatter([{ dataIndex: 0 }]);
    expect(tooltip).toContain('•••••');
    expect(tooltip).not.toMatch(/R\$ \d/);
  });

  it('barra ⌘K tem os relatórios', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('heading', { level: 2, name: 'Fluxo do mês' });
    await user.click(screen.getByRole('button', { name: /Buscar ou executar/ }));
    await user.keyboard('fluxo do mês');
    await user.click(await screen.findByRole('option', { name: /Relatório: fluxo do mês$/ }));
    await waitFor(() => expect(window.location.hash).toMatch(/^#\/relatorios\?v=fluxo/));
  });
});
