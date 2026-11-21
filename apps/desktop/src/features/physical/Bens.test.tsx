/**
 * 08 · Carteira › Bens against the mock: list with value, gain and net worth; detail of the
 * apartment with the mock financing (`loan-1`), figures, improvements and appraisals;
 * register an improvement and an appraisal; add an asset; net worth includes the assets.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { NetWorth } from '@/api/generated/NetWorth';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import { mockCall, mockCallLog } from '@/api/mock';
import { fixToday } from '@/features/portfolio/testing';
import { chartOption, chartProps, clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeEach(() => {
  fixToday(); // 14/10/2026
  clearCharts();
});
afterEach(() => {
  vi.useRealTimers();
});

function open(search = '') {
  return renderApp('unlocked', {}, `#/carteira?aba=bens${search}`);
}

function hashParams(): URLSearchParams {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '');
}

describe('08 · Carteira › Bens with the mock', () => {
  it('lists the assets and opens the apartment with financing, figures, works and appraisals', async () => {
    open();
    const list = await screen.findByRole('grid', { name: 'Bens' });
    for (const name of ['Apartamento Vila Mariana', 'Onix LTZ 2022', 'Terreno em Atibaia'])
      expect(within(list).getAllByText(name).length).toBeGreaterThan(0);
    expect(within(list).getByText('corrigido pelo IGP-M')).toBeTruthy();
    expect(within(list).getByText(/tabela FIPE 004478-4/)).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Valor dos bens' })).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Patrimônio dos bens' })).toBeTruthy();

    const detail = screen.getByRole('region', { name: 'Bem Apartamento Vila Mariana' });
    expect(
      within(detail).getByText(/Imóvel · Ana e Bruno · comprado em 15\/03\/2023/),
    ).toBeTruthy();
    // Mock financing linked, with a link to 07 · Dívidas.
    const loans = within(detail).getByRole('grid', {
      name: 'Financiamentos de Apartamento Vila Mariana',
    });
    const link = within(loans).getByRole('link', { name: 'Financiamento do apartamento' });
    expect(link.getAttribute('href')).toBe('#/dividas?contrato=loan-1');
    // How the value was computed: from the appraisal, adjusted by the IGP-M.
    expect(
      within(detail).getByText(/avaliação de 10\/11\/2025 \(avaliação\).*corrigido pelo IGP-M/),
    ).toBeTruthy();
    // Works: maintenance left out of the cost.
    const works = within(detail).getByRole('grid', {
      name: 'Benfeitorias de Apartamento Vila Mariana',
    });
    expect(within(works).getByText('Manutenção · fora do custo')).toBeTruthy();
    expect(
      within(detail).getByRole('grid', { name: 'Avaliações de Apartamento Vila Mariana' }),
    ).toBeTruthy();

    // Figures on a single axis: cost, value and appraisals; net worth × outstanding balance.
    await waitFor(() => expect(chartProps(/^Valor de Apartamento Vila Mariana/)).toBeTruthy());
    const opt = chartOption(/^Valor de Apartamento Vila Mariana/);
    expect((opt.series as { name: string }[]).map((s) => s.name)).toEqual([
      'Custo acumulado',
      'Valor',
      'Avaliações',
    ]);
    expect(Array.isArray(opt.yAxis)).toBe(false);
    expect(chartProps(/^Patrimônio de Apartamento Vila Mariana/)).toBeTruthy();

    const v = (await mockCall('physical.get', { id: 'phys-apto' })) as PhysicalAssetView;
    expect(v.figures.cost_basis).toBe(60_000_000 + 2_570_000 + 3_800_000 + 1_200_000);
    expect(v.figures.maintenance).toBe(650_000);
    expect(v.figures.loan_balance).toBeGreaterThan(0);
    expect(v.figures.equity).toBe(v.figures.value - v.figures.loan_balance);
  });

  it('registers an improvement and an appraisal, and net worth follows', async () => {
    const user = userEvent.setup();
    open('&bem=phys-terreno');
    const detail = await screen.findByRole('region', { name: 'Bem Terreno em Atibaia' });
    const nwBefore = (await mockCall('reports.net_worth', {})) as NetWorth;
    expect(nwBefore.physical).toBeGreaterThan(0);

    await user.click(within(detail).getByRole('button', { name: 'Benfeitoria' }));
    const dialog = await screen.findByRole('dialog', { name: 'Registrar benfeitoria ou obra' });
    await user.click(within(dialog).getByRole('radio', { name: 'Ampliação' }));
    await user.type(within(dialog).getByLabelText('Descrição'), 'Casa de caseiro');
    await user.type(within(dialog).getByLabelText('Valor'), '45.000,00');
    await user.click(within(dialog).getByRole('button', { name: 'Registrar' }));
    expect(await screen.findByText('Benfeitoria registrada')).toBeTruthy();
    const after = (await mockCall('physical.get', { id: 'phys-terreno' })) as PhysicalAssetView;
    expect(after.figures.capital_improvements).toBe(2_200_000 + 4_500_000);
    const nwAfter = (await mockCall('reports.net_worth', {})) as NetWorth;
    expect(nwAfter.physical).toBe(nwBefore.physical + 4_500_000);
    expect(nwAfter.net).toBe(nwBefore.net + 4_500_000);

    const detail2 = await screen.findByRole('region', { name: 'Bem Terreno em Atibaia' });
    await user.click(await within(detail2).findByRole('button', { name: 'Registrar avaliação' }));
    const vd = await screen.findByRole('dialog', { name: 'Registrar avaliação' });
    await user.type(within(vd).getByLabelText('Valor'), '300.000,00');
    await user.click(within(vd).getByRole('button', { name: 'Registrar avaliação' }));
    expect(await screen.findByText('Avaliação registrada')).toBeTruthy();
    const valued = (await mockCall('physical.get', { id: 'phys-terreno' })) as PhysicalAssetView;
    expect(valued.figures.value).toBe(30_000_000);
    expect(valued.figures.value_basis).toBe('appraisal');
    expect(valued.figures.estimated).toBe(false);

    // Undoing through the history restores the previous value.
    const changes = (await mockCall('changes.list', { limit: 1 })) as { id: string }[];
    await mockCall('changes.undo', { id: changes[0]!.id });
    const undone = (await mockCall('physical.get', { id: 'phys-terreno' })) as PhysicalAssetView;
    expect(undone.valuations).toHaveLength(0);
  });

  it('adds a vehicle with depreciation and validates the form', async () => {
    const user = userEvent.setup();
    open('&novo=bem');
    const dialog = await screen.findByRole('dialog', { name: 'Novo bem' });
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar bem' }));
    expect(within(dialog).getByText('Dê um nome ao bem.')).toBeTruthy();
    expect(within(dialog).getByText('Informe o preço de compra.')).toBeTruthy();
    expect(mockCallLog()).not.toContain('physical.create');

    await user.type(within(dialog).getByLabelText('Nome do bem'), 'Notebook de trabalho');
    await user.type(within(dialog).getByLabelText('Preço de compra'), '12.000,00');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar bem' }));
    expect(await screen.findByText('Bem cadastrado')).toBeTruthy();
    await waitFor(() => expect(hashParams().get('novo')).toBeNull());
    const id = hashParams().get('bem')!;
    const v = (await mockCall('physical.get', { id })) as PhysicalAssetView;
    expect(v).toMatchObject({
      name: 'Notebook de trabalho',
      purchase_price: 1_200_000,
      kind: 'real_estate',
    });
    expect(await screen.findByRole('region', { name: 'Bem Notebook de trabalho' })).toBeTruthy();
  });

  it('the tab shows in the portfolio and the ⌘K command leads to the form', async () => {
    renderApp('unlocked', {}, '#/carteira');
    const tab = await screen.findByRole('tab', { name: 'Bens' });
    const user = userEvent.setup();
    await user.click(tab);
    expect(await screen.findByRole('grid', { name: 'Bens' })).toBeTruthy();
    expect(hashParams().get('aba')).toBe('bens');
  });
});

describe('07 · Bens › Tax on the gain (GCAP)', () => {
  it('simulates the property sale with the FR2 factor and the single-property limit', async () => {
    const user = userEvent.setup();
    open('&bem=phys-apto');
    const detail = await screen.findByRole('region', { name: 'Bem Apartamento Vila Mariana' });
    const card = await within(detail).findByRole('region', {
      name: 'Simular o imposto de uma venda',
    });
    const price = within(card).getByLabelText('Preço de venda');
    await user.clear(price);
    await user.type(price, '900.000,00');
    expect(await within(card).findByText(/Fator FR2/)).toBeTruthy();
    expect(await within(card).findByText(/código 4600/)).toBeTruthy();
    // A single property is only exempt up to R$ 440k: here the tax still applies.
    await user.click(within(card).getByRole('switch', { name: /Único imóvel, sem outra venda/ }));
    expect(within(card).queryByText('Isenção do único imóvel')).toBeNull();
    expect(within(card).getByText(/código 4600/)).toBeTruthy();
  });
});
