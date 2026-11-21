/**
 * Relatórios › Fluxo contra o mock: Sankey com aportes e sobra, comparação com o período
 * anterior (resumo, tooltip e tabela), troca de período pelas setas, profundidade e o modo
 * privacidade.
 */
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCallLog } from '@/api/mock';
import { formatMonthLong, todayIso } from '@/lib/format';
import { monthOf, shiftMonth } from '@/lib/months';
import { useUi } from '@/state/ui';
import { chartOption, chartProps, clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

const month = monthOf(todayIso());
const last = shiftMonth(month, -1);

beforeEach(() => clearCharts());

function hashSearch(): URLSearchParams {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '');
}

type AnyRec = Record<string, unknown>;
type Tip = (p: { dataType: string; name: string; data: AnyRec }) => string;

/**
 * Espera uma condição sobre a opção do gráfico sem `waitFor`: montar a opção lê o tema num
 * elemento do DOM, e a mutação dispararia o `waitFor` de novo, sem fim.
 */
async function eventually(check: () => boolean): Promise<void> {
  const ok = () => {
    try {
      return check();
    } catch {
      return false;
    }
  };
  for (let i = 0; i < 40 && !ok(); i++) await act(() => new Promise((r) => setTimeout(r, 50)));
  expect(check()).toBe(true);
}

function sankeyNodes(): string[] {
  const s = (chartOption(/^Fluxo de/).series as AnyRec[])[0]!;
  return (s.data as { name: string }[]).map((n) => n.name);
}

describe('Relatórios › Fluxo', () => {
  it('mostra para onde vai a receita, com o mês anterior ao lado', async () => {
    renderApp('unlocked', {}, '#/relatorios?v=fluxo&p=mes-anterior');
    expect(await screen.findByText(/Fig\. 01/)).toBeTruthy();
    expect(screen.getByText(/Para onde vai a receita/)).toBeTruthy();
    // Resumo: receita, despesas, sobra/déficit e taxa de poupança, cada um vs o anterior.
    for (const label of ['Receita', 'Despesas', 'Taxa de poupança'])
      expect(screen.getByText(label, { selector: 'dt' })).toBeTruthy();
    expect(screen.getByText(/^(Sobra|Déficit)$/, { selector: 'dt' })).toBeTruthy();
    expect(screen.getAllByText(/^vs /).length).toBeGreaterThanOrEqual(4);

    // Aportes saem do orçamento à parte; transferências entre contas não aparecem.
    const nodes = sankeyNodes();
    expect(nodes).toContain('hub');
    expect(nodes).toContain('investment');
    expect(nodes.some((n) => n.startsWith('sub:'))).toBe(true);

    // Tooltip: valor, % da receita e a variação sobre o mês retrasado.
    const tip = (chartOption(/^Fluxo de/).tooltip as AnyRec).formatter as Tip;
    const html = tip({ dataType: 'node', name: 'investment', data: {} });
    expect(html).toContain('Da receita');
    expect(html).toContain('Variação');

    // Comparativo abaixo do gráfico: atual, anterior, Δ e Δ%.
    const table = screen.getByRole('grid', { name: /^Comparativo do fluxo/ });
    expect(within(table).getByText('Investimentos')).toBeTruthy();
    expect(within(table).getByText('Δ %')).toBeTruthy();
    expect(within(table).getAllByText('Entradas').length).toBeGreaterThan(0);

    // Clique no nó abre o analítico: os lançamentos do período que somam o valor do nó.
    act(() =>
      chartProps(/^Fluxo de/).onClick?.({
        name: 'expense:cat-moradia',
        dataIndex: 0,
        value: 0,
        data: {},
      }),
    );
    const dialog = await screen.findByRole('dialog', { name: /Analítico · Moradia/ });
    expect(await within(dialog).findByText(/igual ao valor de/)).toBeTruthy();
    expect(mockCallLog()).toContain('reports.sankey_items');
    expect(within(dialog).getByRole('tab', { selected: true }).textContent.toLowerCase()).toContain(
      formatMonthLong(last).toLowerCase(),
    );
  });

  it('setas trocam o período; o detalhe esconde as subcategorias', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios?v=fluxo&p=trimestre');
    await screen.findByText(/Fig\. 01/);
    await user.click(screen.getByRole('button', { name: 'Período anterior' }));
    await waitFor(() => expect(hashSearch().get('p')).toBe('trimestre-anterior'));
    await user.click(screen.getByRole('button', { name: 'Período anterior' }));
    await waitFor(() => expect(hashSearch().get('p')).toBe('personalizado'));
    const from = hashSearch().get('from')!;
    expect(from.endsWith('-01')).toBe(true);
    await user.click(screen.getByRole('button', { name: 'Próximo período' }));
    await waitFor(() => expect(hashSearch().get('p')).toBe('trimestre-anterior'));
    expect(mockCallLog().filter((c) => c === 'reports.sankey').length).toBeGreaterThan(2);

    await screen.findByText(/Fig\. 01/);
    // O gráfico de cada período fica guardado pelo rótulo: começa do zero no período atual.
    clearCharts();
    await user.click(screen.getByRole('radio', { name: 'Categorias' }));
    await eventually(() => !sankeyNodes().some((n) => n.startsWith('sub:')));
    expect(sankeyNodes()).toContain('investment');
  });

  it('modo privacidade esconde valores do resumo, do tooltip e da tabela', async () => {
    renderApp('unlocked', {}, '#/relatorios?v=fluxo&p=mes-anterior');
    await screen.findByText(/Fig\. 01/);
    act(() => useUi.getState().setPrivacy(true));
    await waitFor(() =>
      expect(document.querySelectorAll('[data-money="hidden"]').length).toBeGreaterThan(10),
    );
    const table = screen.getByRole('grid', { name: /^Comparativo do fluxo/ });
    expect(within(table).queryByText(/R\$/)).toBeNull();
    const tip = (chartOption(/^Fluxo de/, true).tooltip as AnyRec).formatter as Tip;
    expect(tip({ dataType: 'node', name: 'investment', data: {} })).not.toMatch(/R\$|%/);
  });
});
