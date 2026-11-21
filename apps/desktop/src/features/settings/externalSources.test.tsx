import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { FundamentusSnapshot } from '@/api/generated/FundamentusSnapshot';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

import { fixToday, quietCanvas } from '../portfolio/testing';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('external sources and Fundamentus', () => {
  it('turns off a source in Settings › Data sources', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=fontes');
    const card = await screen.findByRole('region', { name: /Fontes externas/ });
    // Open Food Facts comes turned off (only the person turns it on).
    expect(await within(card).findByText('31 de 32 ligadas')).toBeTruthy();
    await user.click(within(card).getByRole('switch', { name: 'Fundamentus' }));
    expect(await within(card).findByText('30 de 32 ligadas')).toBeTruthy();
    await expect(mockCall('research.fundamentus', { ticker: 'ITSA4' })).rejects.toThrow(
      /Fundamentus está desligada/,
    );
  });

  it('searches and filters the sources by country on screen only', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=fontes');
    const card = await screen.findByRole('region', { name: /Fontes externas/ });
    await within(card).findByRole('switch', { name: 'Yahoo Finance' });
    const country = within(card).getByRole('radiogroup', { name: 'Filtrar fontes por país' });
    await user.click(within(country).getByRole('radio', { name: /Brasil/ }));
    expect(within(card).getByText('12 de 32')).toBeTruthy();
    expect(
      within(card).getByRole('switch', { name: 'B3 (COTAHIST e empresas listadas)' }),
    ).toBeTruthy();
    expect(within(card).queryByRole('switch', { name: 'SEC EDGAR' })).toBeNull();
    await user.click(within(country).getByRole('radio', { name: 'Todas' }));
    const search = within(card).getByRole('searchbox', { name: 'Buscar fonte' });
    await user.type(search, 'inflacao reino');
    await waitFor(() => expect(within(card).getByText('1 de 32')).toBeTruthy());
    expect(within(card).getByRole('switch', { name: 'ONS (Reino Unido)' })).toBeTruthy();
    await user.clear(search);
    await user.type(search, 'estados unidos');
    await waitFor(() => expect(within(card).getByText('4 de 32')).toBeTruthy());
    await user.type(search, ' xyz');
    expect(await within(card).findByText(/Nenhuma fonte encontrada/)).toBeTruthy();
  });

  it('shows Fundamentus indicators and dividends in the fundamentals tab', async () => {
    quietCanvas();
    fixToday();
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos?ativo=ast-hglg11&secao=fundamentos');
    const card = await screen.findByRole('region', { name: 'Fundamentus' });
    expect(await within(card).findByText('Div. Yield')).toBeTruthy();
    expect(within(card).getByText('Vacância Média')).toBeTruthy();
    expect(
      within(card).getByRole('grid', { name: 'Proventos de HGLG11 no Fundamentus' }),
    ).toBeTruthy();
    expect(within(card).getByText('Lido em')).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Atualizar' }));
    await waitFor(async () =>
      expect(
        ((await mockCall('research.fundamentus', { ticker: 'HGLG11' })) as FundamentusSnapshot)
          .from_cache,
      ).toBe(true),
    );
    vi.useRealTimers();
  });

  it('FII: announced yields in the Proventos tab and lookup on Fundamentus', async () => {
    quietCanvas();
    fixToday();
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos?ativo=ast-hglg11&secao=proventos');
    const card = await screen.findByRole('region', { name: 'Rendimentos anunciados pelo fundo' });
    expect(
      await within(card).findByRole('grid', { name: 'Proventos anunciados de HGLG11' }),
    ).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Buscar no Fundamentus' }));
    expect(await screen.findByText(/proventos novos anunciados/)).toBeTruthy();
    vi.useRealTimers();
  });
});
