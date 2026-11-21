import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

/**
 * World exchanges against the real core (no network): the V85 catalog shows up in Assets ›
 * Bolsas, a new exchange is registered, and a stock from another exchange (LSE) is registered
 * with the exchange's currency and country. Ticker search and the intraday quote use Yahoo, so
 * here the quote is turned off ("Sem cotação") and search is not used.
 */
test('exchange catalog, new exchange and foreign stock on the real core', async ({ page }) => {
  await openVault(page);
  await page
    .getByRole('link', { name: /Ativos/ })
    .first()
    .click();
  await page.getByRole('tab', { name: 'Bolsas' }).click();
  const grid = page.getByRole('grid', { name: 'Bolsas' });
  await expect(grid.getByText('London Stock Exchange')).toBeVisible();
  await expect(grid.getByText('Bolsa de Tóquio (JPX)')).toBeVisible();

  await page.getByRole('button', { name: 'Cadastrar bolsa' }).click();
  const ex = page.getByRole('dialog', { name: 'Cadastrar bolsa' });
  await ex.getByLabel('Código', { exact: true }).fill('PSE');
  await ex.getByLabel('Nome', { exact: true }).fill('Philippine Stock Exchange');
  await ex.getByLabel('País', { exact: true }).fill('PH');
  await ex.getByLabel('Moeda', { exact: true }).fill('PHP');
  await ex.getByLabel('Sufixo no Yahoo').fill('.PS');
  await ex.getByLabel('Fuso (IANA)').fill('Asia/Manila');
  await ex.getByRole('button', { name: 'Salvar bolsa' }).click();
  await expect(ex).toBeHidden();
  await expect(grid.getByText('Philippine Stock Exchange')).toBeVisible();

  await page.getByRole('button', { name: 'Cadastrar ativo' }).first().click();
  const form = page.getByRole('dialog', { name: 'Cadastrar ativo' });
  await form.getByRole('radio', { name: 'Ações em outras bolsas' }).click();
  await form.getByLabel('Ticker', { exact: true }).fill('VOD');
  await form.getByLabel('Nome', { exact: true }).fill('Vodafone Group');
  await form.getByRole('combobox', { name: 'Bolsa' }).click();
  await page.getByRole('option', { name: /LSE · London Stock Exchange/ }).click();
  await expect(form.getByText(/GB · GBP/)).toBeVisible();
  // The quote source lives under "Cotação e identificadores" (collapsed).
  await form.getByText('Cotação e identificadores').click();
  await form.getByRole('combobox', { name: 'Fonte de cotação' }).click();
  await page.getByRole('option', { name: 'Sem cotação' }).click();
  await form.getByRole('button', { name: 'Cadastrar ativo' }).click();
  await expect(page.getByRole('heading', { level: 2, name: /VOD/ })).toBeVisible();
  await expect(
    page.getByText(/Ações em outras bolsas · London Stock Exchange · GBP/),
  ).toBeVisible();
});
