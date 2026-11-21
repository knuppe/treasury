/** Figura 07 · Inflação e o botão de PDF da tela de Relatórios (modo de demonstração). */
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

describe('Relatórios · Inflação', () => {
  it('mostra o gasto nominal e corrigido, o índice e as categorias', async () => {
    renderApp('unlocked', {}, '#/relatorios?v=inflacao');
    expect(await screen.findByText(/Fig\. 07/)).toBeTruthy();
    expect(screen.getByText(/Gasto nominal × corrigido pelo IPCA/)).toBeTruthy();
    expect(screen.getByText(/Gastamos mais ou só ficou mais caro\?/)).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Categorias contra a inflação' })).toBeTruthy();
    const option = chartOption(/^Linhas do gasto nominal/);
    const series = option.series as { name: string; data: unknown[] }[];
    expect(series.map((s) => s.name)).toEqual(['Nominal', 'Corrigido']);
    // Os meses ainda sem IPCA divulgado aparecem com "≈" no eixo.
    const axis = (option.xAxis as { data: string[] }).data;
    expect(axis.at(-1)).toMatch(/≈$/);
    expect(mockCallLog().includes('reports.inflation')).toBe(true);
  });

  it('em inglês', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/relatorios?v=inflacao');
    expect(await screen.findByText(/Nominal spending × adjusted by IPCA/)).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Categories against inflation' })).toBeTruthy();
    expect(screen.getByRole('button', { name: /Download month PDF/ })).toBeTruthy();
  });

  it('gera o PDF do mês e avisa onde salvou', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios');
    await user.click(await screen.findByRole('button', { name: /Baixar PDF do mês/ }));
    await waitFor(() => expect(mockCallLog().includes('reports.pdf')).toBe(true));
    expect(await screen.findByText('Relatório salvo')).toBeTruthy();
    expect(screen.getByText(/^2 páginas em /)).toBeTruthy();
  });
});
