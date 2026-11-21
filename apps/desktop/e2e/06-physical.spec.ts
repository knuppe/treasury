import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

/**
 * Patrimônio físico contra o núcleo real: cadastra um imóvel com custos de aquisição,
 * registra uma reforma (entra no custo) e confere que o patrimônio líquido do painel passa
 * a incluir o bem. Sem rede: o núcleo do E2E não tem índices baixados, então a correção
 * fica em 1 e o valor é o custo (estimado).
 */
test('cadastra um imóvel e uma benfeitoria; o patrimônio líquido inclui o bem', async ({
  page,
}) => {
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

  // Painel: o valor do bem (custo, sem índices no E2E) entra no patrimônio líquido.
  await page
    .getByRole('link', { name: /Painel/ })
    .first()
    .click();
  await expect(bensValue).toContainText('400.000,00');
  await expect(net).not.toHaveText(before);
});
