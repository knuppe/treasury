/** Figura 07 · Inflação and the PDF button on the Reports screen (demo mode). */
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { mockCallLog } from '@/api/mock';
import { applyLocale } from '@/i18n';
import { chartOption, clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  clearCharts();
  await applyLocale('pt-BR');
});

describe('Relatórios · Inflation', () => {
  it('shows nominal and adjusted spending, the index and the categories', async () => {
    renderApp('unlocked', {}, '#/relatorios?v=inflacao');
    expect(await screen.findByText(/Fig\. 07/)).toBeTruthy();
    expect(screen.getByText(/Gasto nominal × corrigido pelo IPCA/)).toBeTruthy();
    expect(screen.getByText(/Gastamos mais ou só ficou mais caro\?/)).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Categorias contra a inflação' })).toBeTruthy();
    const option = chartOption(/^Linhas do gasto nominal/);
    const series = option.series as { name: string; data: unknown[] }[];
    expect(series.map((s) => s.name)).toEqual(['Nominal', 'Corrigido']);
    // Months with no published IPCA yet show up with "≈" on the axis.
    const axis = (option.xAxis as { data: string[] }).data;
    expect(axis.at(-1)).toMatch(/≈$/);
    expect(mockCallLog().includes('reports.inflation')).toBe(true);
  });

  it('in English', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/relatorios?v=inflacao');
    expect(await screen.findByText(/Nominal spending × adjusted by IPCA/)).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Categories against inflation' })).toBeTruthy();
    expect(screen.getByRole('button', { name: /Download month PDF/ })).toBeTruthy();
  });

  it("generates the month's PDF and says where it was saved", async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios');
    await user.click(await screen.findByRole('button', { name: /Baixar PDF do mês/ }));
    await waitFor(() => expect(mockCallLog().includes('reports.pdf')).toBe(true));
    expect(await screen.findByText('Relatório salvo')).toBeTruthy();
    expect(screen.getByText(/^2 páginas em /)).toBeTruthy();
  });
});
