import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { FxOperation } from '@/api/generated/FxOperation';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('operação de câmbio', () => {
  it('reais para dólar com PTAX, spread, IOF e tarifa', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    await user.click(screen.getByRole('button', { name: 'Câmbio' }));
    const dialog = await screen.findByRole('dialog', { name: 'Operação de câmbio' });
    await user.click(within(dialog).getByRole('combobox', { name: 'De (conta)' }));
    await user.click(await screen.findByRole('option', { name: /Itaú Conta Corrente/ }));
    await user.click(within(dialog).getByRole('combobox', { name: 'Para (conta)' }));
    await user.click(await screen.findByRole('option', { name: /Wise USD/ }));
    await user.type(within(dialog).getByLabelText(/^Valor \(USD\)/), '1.000,00');
    await user.type(within(dialog).getByLabelText('Spread (%)'), '1');
    const preview = await within(dialog).findByRole('status', { name: 'Prévia do câmbio' });
    expect(within(preview).getByText('VET (R$ por unidade)')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Lançar câmbio' }));
    await waitFor(async () =>
      expect(((await mockCall('fx.operations', {})) as FxOperation[]).length).toBe(1),
    );
    const [op] = (await mockCall('fx.operations', {})) as FxOperation[];
    expect(op).toMatchObject({ from_currency: 'BRL', to_currency: 'USD', amount_to: 100_000 });
    // PTAX 5,4321 × 1,01 = 5,486421 por dólar; IOF padrão 1,1% na remessa.
    expect(op!.amount_from).toBe(548_642);
    expect(op!.iof_amount).toBe(6_035);
  });

  it('numa família em dólar: dólar para euro com o câmbio de referência em USD, sem IOF', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', { country: 'US' }, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    await user.click(screen.getByRole('button', { name: 'Câmbio' }));
    const dialog = await screen.findByRole('dialog', { name: 'Operação de câmbio' });
    await user.click(within(dialog).getByRole('combobox', { name: 'De (conta)' }));
    await user.click(await screen.findByRole('option', { name: /Itaú Conta Corrente/ }));
    await user.click(within(dialog).getByRole('combobox', { name: 'Para (conta)' }));
    await user.click(await screen.findByRole('option', { name: /Revolut EUR/ }));
    expect(await within(dialog).findByText('Câmbio de referência (1 EUR em USD)')).toBeTruthy();
    expect(within(dialog).queryByLabelText('IOF (%)')).toBeNull();
    await user.type(within(dialog).getByLabelText(/^Valor \(EUR\)/), '100,00');
    const preview = await within(dialog).findByRole('status', { name: 'Prévia do câmbio' });
    expect(within(preview).getByText('VET (USD por unidade)')).toBeTruthy();
    expect(within(preview).queryByText('IOF')).toBeNull();
    await user.click(within(dialog).getByRole('button', { name: 'Lançar câmbio' }));
    await waitFor(async () =>
      expect(((await mockCall('fx.operations', {})) as FxOperation[]).length).toBe(1),
    );
    const [op] = (await mockCall('fx.operations', {})) as FxOperation[];
    expect(op).toMatchObject({ from_currency: 'USD', to_currency: 'EUR', amount_to: 10_000 });
    // Referência cruzada pela PTAX fictícia: 5,9012 ÷ 5,4321 ≈ 1,0864 dólar por euro.
    expect(op!.amount_from).toBe(10_864);
    expect(op!.iof_amount).toBe(0);
  });
});
