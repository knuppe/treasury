/** 08 · Carteira › Bens e 05 · Orçamento › Metas em inglês: rótulos e textos pelo catálogo. */
import { screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { fixToday } from '@/features/portfolio/testing';
import { applyLocale } from '@/i18n';
import { clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeEach(async () => {
  fixToday();
  clearCharts();
  await applyLocale('en');
});
afterEach(async () => {
  vi.useRealTimers();
  await applyLocale('pt-BR');
});

describe('Bens e metas em inglês', () => {
  it('mostra a lista e o detalhe dos bens em inglês', async () => {
    renderApp('unlocked', {}, '#/carteira?aba=bens');
    const list = await screen.findByRole('grid', { name: 'Physical assets' });
    expect(within(list).getByText('adjusted by IGP-M')).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Value of assets' })).toBeTruthy();
    const detail = screen.getByRole('region', { name: 'Asset Apartamento Vila Mariana' });
    expect(within(detail).getByText(/Real estate · Ana and Bruno · bought on/)).toBeTruthy();
    expect(within(detail).getByRole('button', { name: 'Record appraisal' })).toBeTruthy();
  });

  it('mostra as metas em inglês', async () => {
    renderApp('unlocked', {}, '#/orcamento?aba=metas');
    expect(await screen.findByRole('button', { name: 'New goal' })).toBeTruthy();
    expect(screen.getByText('Show archived')).toBeTruthy();
  });
});
