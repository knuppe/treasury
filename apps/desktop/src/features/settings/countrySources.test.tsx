import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { CountryCatalog } from '@/api/generated/CountryCatalog';
import type { FactorResult } from '@/api/generated/FactorResult';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

describe('Ajustes › Fontes de dados: país de referência', () => {
  it('troca o país, mostra as séries do catálogo e os provedores em uso', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=fontes');
    const card = await screen.findByRole('region', { name: /País de referência/ });
    // Padrão: automático pela moeda base (BRL → Brasil), com IPCA.
    expect(await within(card).findByText(/Inflação do país \(Brasil\) = IPCA/)).toBeTruthy();

    await user.click(within(card).getByRole('combobox', { name: 'País de referência' }));
    await user.click(await screen.findByRole('option', { name: /^Portugal/ }));
    expect(
      await within(card).findByText(/Inflação do país \(Portugal\) = HICP Portugal/),
    ).toBeTruthy();
    expect(within(card).getByText(/Taxas de referência do BCE/)).toBeTruthy();
    const cat = (await mockCall('indices.catalog', {})) as CountryCatalog;
    expect(cat.country).toBe('PT');
    expect(cat.automatic).toBe(false);

    const providers = await screen.findByRole('grid', { name: 'Provedores de dados' });
    const ecb = within(providers)
      .getByText('Banco Central Europeu (ECB Data Portal)')
      .closest('tr')!;
    expect(within(ecb).getByText('Em dia')).toBeTruthy();
    const oecd = within(providers).getByText('OCDE (OECD Data Explorer)').closest('tr')!;
    expect(within(oecd).getByText('Fora de uso')).toBeTruthy();

    // Volta ao automático.
    await user.click(within(card).getByRole('combobox', { name: 'País de referência' }));
    await user.click(await screen.findByRole('option', { name: /^Automático/ }));
    await waitFor(async () =>
      expect(((await mockCall('indices.catalog', {})) as CountryCatalog).automatic).toBe(true),
    );
  });

  it('deflator "country" segue o país de referência', async () => {
    const br = (await mockCall('indices.factor', {
      deflator: 'country',
      from: '2025-01-01',
      to: '2025-06-01',
    })) as FactorResult;
    const ipca = (await mockCall('indices.factor', {
      deflator: 'ipca',
      from: '2025-01-01',
      to: '2025-06-01',
    })) as FactorResult;
    expect(br.series_id).toBe('ipca');
    expect(br.factor).toBe(ipca.factor);
    await mockCall('indices.set_country', { country: 'PT' });
    const pt = (await mockCall('indices.factor', {
      deflator: 'country',
      from: '2025-01-01',
      to: '2025-06-01',
    })) as FactorResult;
    expect(pt.series_id).toBe('hicp_pt');
    await mockCall('indices.set_country', { country: null });
  });
});
