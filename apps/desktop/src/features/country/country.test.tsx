/**
 * Family country against the mock (docs/countries.md): the choice when creating the vault, the switch
 * in Settings › Family and what disappears outside Brazil.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { CountryState } from '@/api/generated/CountryState';
import type { Household } from '@/api/generated/Household';
import type { SourceInfo } from '@/api/generated/SourceInfo';
import { mockCall } from '@/api/mock';
import { screenMeta, screenVisible } from '@/app/screens';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

const STRONG = 'vaca amarela pulou a janela';

async function enabled(): Promise<string[]> {
  const list = (await mockCall('sources.list', {})) as SourceInfo[];
  return list.filter((s) => s.enabled).map((s) => s.id);
}

describe('family country', () => {
  it('vault creation asks for the country and the vault is born in Portugal', async () => {
    const user = userEvent.setup();
    renderApp('empty');
    await user.click(await screen.findByRole('radio', { name: /Cofre real/ }));
    await user.type(await screen.findByLabelText('Nome da família'), 'Família Costa');
    await user.type(screen.getByLabelText('Membro 1'), 'Inês');
    await user.click(screen.getByRole('combobox', { name: 'País onde a família mora' }));
    await user.click(await screen.findByRole('option', { name: /^Portugal · EUR/ }));
    expect(screen.getByText(/Moeda base: EUR/)).toBeTruthy();
    await user.type(screen.getByLabelText('Senha mestra'), STRONG);
    await user.type(screen.getByLabelText('Confirmar senha'), STRONG);
    await user.click(screen.getByRole('checkbox', { name: /perco os dados/ }));
    await user.click(screen.getByRole('button', { name: 'Criar cofre cifrado' }));
    expect(await screen.findByRole('navigation', { name: 'Telas' })).toBeTruthy();

    const h = (await mockCall('household.get', {})) as Household;
    expect([h.country, h.base_currency]).toEqual(['PT', 'EUR']);
    const st = (await mockCall('country.get', {})) as CountryState;
    expect(st.info.inflation_label).toBe('IHPC');
    const on = await enabled();
    expect(on).toContain('eurostat');
    expect(on).not.toContain('b3');
  });

  it('switches to the United States in Settings and the Brazil screens disappear', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=familia');
    const card = await screen.findByRole('region', { name: /País/ });
    await user.click(within(card).getByRole('combobox', { name: 'País onde a família mora' }));
    await user.click(await screen.findByRole('option', { name: /^Estados Unidos · USD/ }));
    const dialog = await screen.findByRole('dialog', {
      name: 'Mudar o país da família para Estados Unidos?',
    });
    // The sample vault has transactions: the currency only changes if asked.
    expect(
      within(dialog).getByRole('checkbox', { name: /Trocar a moeda base de BRL para USD/ }),
    ).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Mudar para Estados Unidos' }));
    expect(await screen.findByText('País da família: Estados Unidos')).toBeTruthy();
    expect(await within(card).findByText(/inflação: CPI-U/)).toBeTruthy();
    const h = (await mockCall('household.get', {})) as Household;
    expect([h.country, h.base_currency]).toEqual(['US', 'BRL']);
    const on = await enabled();
    expect(on).toContain('ny_fed');
    expect(on).not.toContain('bcb');

    // Reports without the Brazil IR tab; Debts without the Banco Central rates.
    await user.click(screen.getByRole('link', { name: /Relatórios/ }));
    await screen.findByRole('heading', { level: 1, name: /Relatórios/ });
    expect(screen.queryByRole('tab', { name: /Imposto de renda/ })).toBeNull();
    await user.click(screen.getByRole('link', { name: /Lançamentos/ }));
    const tabs = await screen.findByRole('tablist', { name: 'Seções de lançamentos' });
    expect(within(tabs).getByRole('tab', { name: 'Importações' })).toBeTruthy();
    expect(within(tabs).queryByRole('tab', { name: 'Cupons fiscais' })).toBeNull();
    await user.click(screen.getByRole('link', { name: /Dívidas/ }));
    await screen.findByRole('heading', { level: 1, name: /Dívidas/ });
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: /Juros do mercado/ })).toBeNull(),
    );
  }, 60_000); // goes through four screens: under load, the whole suite exceeds the default 20 s

  it('in a country with no tax data, the taxes screen leaves the menu', async () => {
    const impostos = screenMeta('impostos');
    renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('navigation', { name: 'Telas' });
    const br = ((await mockCall('country.get', {})) as CountryState).info.features;
    expect(screenVisible(impostos, false, br)).toBe(true);
    const mx = ((await mockCall('country.set', { country: 'MX' })) as { state: CountryState }).state
      .info.features;
    expect(mx.tax_burden).toBe(false);
    expect(screenVisible(impostos, false, mx)).toBe(false);
    expect(screenVisible(screenMeta('familia'), false, mx)).toBe(true);
  });
});
