/**
 * IPTU in the property detail (Portfolio › Assets) against the mock: the year's carnê with its installments,
 * marking an installment as paid by date, registering the next year's carnê, and the panel
 * hidden on the vehicle and outside Brazil.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { IptuView } from '@/api/generated/IptuView';
import { mockCall } from '@/api/mock';
import { fixToday } from '@/features/portfolio/testing';
import { clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeEach(() => {
  fixToday(); // 14/10/2026
  clearCharts();
});
afterEach(() => {
  vi.useRealTimers();
});

const iptu = async (asset: string) =>
  (await mockCall('physical.iptu_list', { asset_id: asset })) as IptuView[];

describe('property IPTU', () => {
  it('shows the carnê, marks the last installment as paid and registers the next year', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?aba=bens&bem=phys-apto');
    const detail = await screen.findByRole('region', { name: 'Bem Apartamento Vila Mariana' });
    const card = await within(detail).findByRole('heading', { name: 'IPTU' });
    const panel = card.closest('section') ?? detail;
    const grid = await within(panel).findByRole('grid', {
      name: 'Parcelas do IPTU de 2026',
    });
    expect(within(grid).getAllByText(/Paga em/)).toHaveLength(9);
    expect(within(grid).getByText('Outra forma')).toBeTruthy();
    const last = within(grid).getByText('Parcela 10').closest('tr') as HTMLElement;
    await user.click(within(last).getByRole('button', { name: 'Marcar como pago' }));
    const dialog = await screen.findByRole('dialog', { name: 'Marcar parcela 10 como paga' });
    await user.click(within(dialog).getByRole('button', { name: 'Marcar como pago' }));
    await waitFor(async () => {
      const b = (await iptu('phys-apto'))[0]!;
      expect(b.installments.find((i) => i.number === 10)!.status).toBe('paid');
    });

    await user.click(within(panel).getByRole('button', { name: 'Registrar outro ano' }));
    const form = await screen.findByRole('dialog', {
      name: 'Registrar IPTU · Apartamento Vila Mariana',
    });
    // Registration and municipality come from the previous carnê.
    expect(within(form).getByLabelText<HTMLInputElement>('Número do cadastro / IQ').value).toBe(
      '034.112.0021-4',
    );
    expect(within(form).getByLabelText<HTMLInputElement>('Ano (exercício)').value).toBe('2027');
    await user.type(within(form).getByLabelText('Total devido'), '5.600,00');
    await user.type(within(form).getByLabelText('Número de parcelas'), '10');
    await user.type(within(form).getByLabelText('Primeiro vencimento'), '10/02/2027');
    await user.type(within(form).getByLabelText('Valor de cada parcela'), '560,00');
    await user.click(within(form).getByRole('button', { name: 'Salvar' }));
    await waitFor(async () =>
      expect((await iptu('phys-apto')).map((b) => b.year)).toEqual([2027, 2026]),
    );
    const b = (await iptu('phys-apto'))[0]!;
    expect(b.total_due).toBe(560_000);
    expect(b.installments.map((i) => i.due_date).slice(0, 2)).toEqual(['2027-02-10', '2027-03-10']);
  });

  it('does not appear on the vehicle nor outside Brazil', async () => {
    renderApp('unlocked', {}, '#/carteira?aba=bens&bem=phys-onix');
    const detail = await screen.findByRole('region', { name: /Bem Onix/ });
    expect(within(detail).queryByRole('heading', { name: 'IPTU' })).toBeNull();
  });
});
