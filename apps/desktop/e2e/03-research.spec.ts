import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

/**
 * Asset analysis against the real core, with no collection (no network): empty universe with
 * an explanatory empty state in Rankings and in the Market Explorer (the old Assets link
 * leads there), a screener view saved by the core, undone and saved again, and the analysis
 * settings in Assets (SEC User-Agent). No React warnings in the console.
 */
test('screener saves a view and explains the empty universe on the real core', async ({ page }) => {
  const warnings: string[] = [];
  page.on('console', (m) => {
    if (m.type() === 'error' || m.type() === 'warning') warnings.push(m.text());
  });
  await openVault(page);
  // New vault: the welcome screen leads to the dashboard after saving the profile; wait for that
  // before opening the link, so the trip to the dashboard does not trample the navigation.
  await expect(page).toHaveURL(/#\/painel/);
  await page.goto('/#/ativos?aba=rankings');
  await expect(page).toHaveURL(/#\/mercado\?aba=rankings/);
  await expect(page.getByText('Nenhum papel no universo ainda')).toBeVisible();

  await page.getByRole('tab', { name: 'Explorador' }).click();
  await expect(page.getByText('O universo ainda está vazio')).toBeVisible();
  await page.getByRole('button', { name: 'Rodar screener' }).click();
  await expect(page.getByText('Nenhum papel no universo ainda')).toBeVisible();

  await page.getByRole('button', { name: 'Salvar tela…' }).click();
  const dialog = page.getByRole('dialog', { name: 'Salvar tela do screener' });
  await dialog.getByLabel('Nome').fill('Baratas com dividendos');
  await dialog.getByRole('button', { name: 'Salvar tela' }).click();
  await expect(dialog).toBeHidden();
  await expect(page.getByText('Tela · Baratas com dividendos')).toBeVisible();
  await expect(
    page.getByText(/Salva: P\/L < 12 E Dividend yield \(12 meses\) ≥ 0.06/),
  ).toBeVisible();

  // Undo right after: the view leaves the list.
  await page.getByRole('button', { name: 'Desfazer' }).click();
  await expect(page.getByText('Alteração desfeita').first()).toBeVisible();
  await expect(page.getByRole('combobox', { name: 'Telas salvas' })).toContainText(
    'Nenhuma tela salva',
  );

  // Save again and reopen after reloading.
  await page.getByRole('button', { name: 'Salvar tela…' }).click();
  await dialog.getByLabel('Nome').fill('Baratas com dividendos');
  await dialog.getByRole('button', { name: 'Salvar tela' }).click();
  await expect(dialog).toBeHidden();
  await page.reload();
  await page.getByRole('combobox', { name: 'Telas salvas' }).click();
  await page.getByRole('option', { name: 'Baratas com dividendos' }).click();
  await expect(page.getByText('Tela · Baratas com dividendos')).toBeVisible();
  await expect(page.getByLabel('Valor da condição 2')).toHaveValue('6');

  // Analysis data (in Assets): SEC notice and email adjustment.
  await page.goto('/#/ativos?aba=dados');
  await expect(page.getByRole('tab', { name: 'Dados da análise' })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  await expect(page.getByText('Configure o e-mail da SEC')).toBeVisible();
  await expect(page.getByText('nunca')).toBeVisible();
  const ua = page.getByLabel('User-Agent da SEC EDGAR');
  await ua.fill('Treasury/0.1 e2e@familia.org');
  await page.getByRole('button', { name: 'Salvar ajustes' }).click();
  await expect(page.getByText('Configure o e-mail da SEC')).toBeHidden();
  expect(warnings.filter((w) => /uncontrolled|controlled/i.test(w))).toEqual([]);
});
