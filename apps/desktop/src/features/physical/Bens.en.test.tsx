/** 08 · Carteira › Bens and 05 · Orçamento › Metas in English: labels and texts from the catalog. */
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

describe('Bens and goals in English', () => {
  it('shows the list and detail of the assets in English', async () => {
    renderApp('unlocked', {}, '#/carteira?aba=bens');
    const list = await screen.findByRole('grid', { name: 'Physical assets' });
    expect(within(list).getByText('adjusted by IGP-M')).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Value of assets' })).toBeTruthy();
    const detail = screen.getByRole('region', { name: 'Asset Apartamento Vila Mariana' });
    expect(within(detail).getByText(/Real estate · Ana and Bruno · bought on/)).toBeTruthy();
    expect(within(detail).getByRole('button', { name: 'Record appraisal' })).toBeTruthy();
  });

  it('shows the goals in English', async () => {
    renderApp('unlocked', {}, '#/orcamento?aba=metas');
    expect(await screen.findByRole('button', { name: 'New goal' })).toBeTruthy();
    expect(screen.getByText('Show archived')).toBeTruthy();
  });
});
