import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

/**
 * Physical assets against the real core: registers a property with acquisition costs,
 * records a renovation (added to the cost) and checks that the dashboard net worth
 * now includes the asset. No network: the E2E core has no downloaded indices, so the adjustment
 * stays at 1 and the value is the cost (estimated).
 */
test('registers a property and an improvement; net worth includes the asset', async ({ page }) => {
  await openVault(page);

  await page
    .getByRole('link', { name: /Painel/ })
    .first()
    .click();
  const bensValue = page.locator('dt:has-text("Bens") + dd').first();
  await expect(bensValue).toBeVisible();
  const net = page.getByRole('button', { name: /Explicar Patrimônio líquido em/ });
  await expect(net).toBeVisible();
  const before = (await net.textContent()) ?? '';

  await page
    .getByRole('link', { name: /Carteira/ })
    .first()
    .click();
  await page.getByRole('tab', { name: 'Bens' }).click();
  await expect(page.getByText('Cadastre o primeiro imóvel, veículo ou terreno')).toBeVisible();
  await page.getByRole('button', { name: 'Novo bem' }).first().click();

  const dialog = page.getByRole('dialog', { name: 'Novo bem' });
  await dialog.getByLabel('Nome do bem').fill('Casa de praia');
  await dialog.getByLabel('Preço de compra').fill('340.000,00');
  await dialog.getByRole('button', { name: 'Adicionar custo' }).click();
  await dialog.getByLabel('Valor', { exact: true }).fill('10.000,00');
  await dialog.getByRole('button', { name: 'Cadastrar bem' }).click();
  await expect(dialog).toBeHidden();

  const detail = page.getByRole('region', { name: 'Bem Casa de praia' });
  await expect(detail).toBeVisible();
  await expect(detail.getByText(/350\.000,00/).first()).toBeVisible();

  await detail.getByRole('button', { name: 'Benfeitoria', exact: true }).click();
  const works = page.getByRole('dialog', { name: 'Registrar benfeitoria ou obra' });
  await works.getByLabel('Descrição').fill('Reforma da cozinha');
  await works.getByLabel('Valor').fill('50.000,00');
  await works.getByRole('button', { name: 'Registrar' }).click();
  await expect(works).toBeHidden();
  const table = detail.getByRole('grid', { name: 'Benfeitorias de Casa de praia' });
  await expect(table.getByText('Reforma da cozinha')).toBeVisible();
  await expect(detail.getByText(/400\.000,00/).first()).toBeVisible();

  // Dashboard: the asset value (cost, no indices in the E2E) enters net worth.
  await page
    .getByRole('link', { name: /Painel/ })
    .first()
    .click();
  await expect(bensValue).toContainText('400.000,00');
  await expect(net).not.toHaveText(before);
});
