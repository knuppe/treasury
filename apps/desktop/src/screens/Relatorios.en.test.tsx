/** Tela Relatórios em inglês: abas, filtros e a figura do fluxo pelo catálogo. */
import { screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { applyLocale } from '@/i18n';
import { clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  clearCharts();
  await applyLocale('pt-BR');
});

describe('Relatórios em inglês', () => {
  it('mostra as abas, os filtros e o fluxo traduzidos', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/relatorios');
    expect(await screen.findByText(/Fig\. 01/)).toBeTruthy();
    expect(screen.getByRole('tablist', { name: 'Views' })).toBeTruthy();
    for (const name of [/Flow/, /Composition/, /Rhythm/, /Net worth/, /Financial health/])
      expect(screen.getByRole('tab', { name })).toBeTruthy();
    expect(screen.getByRole('toolbar', { name: 'Report filters' })).toBeTruthy();
    expect(screen.getByRole('radiogroup', { name: 'Detail' })).toBeTruthy();
    expect(screen.getByText(/^Where the income goes:/)).toBeTruthy();
    expect(screen.getByText('Savings rate')).toBeTruthy();
    expect(screen.getByText(/^Source: settled transactions · /)).toBeTruthy();
    expect(screen.queryByText('Taxa de poupança')).toBeNull();
  });
});
