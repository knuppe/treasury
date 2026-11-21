import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

test('account, transaction, balance and undo on the real core', async ({ page }) => {
  await openVault(page);

  // Accounts: new checking account with an opening balance.
  await page
    .getByRole('link', { name: /Contas/ })
    .first()
    .click();
  await page.getByRole('button', { name: 'Nova conta' }).first().click();
  const form = page.getByRole('dialog', { name: 'Nova conta' });
  await form.getByLabel('Nome').fill('Banco E2E');
  await form.getByLabel(/Saldo inicial/).fill('1.000,00');
  await form.getByRole('button', { name: 'Criar conta' }).click();
  await expect(form).toBeHidden();
  // After creating it, the account opens in the detail view; in the list, it is its row. In both, the screen shows the
  // name and the balance.
  const card = page.getByRole('main');
  await expect(card).toContainText('Banco E2E');
  await expect(card).toContainText('1.000,00');

  // Transactions: an expense on that account.
  await page
    .getByRole('link', { name: /Lançamentos/ })
    .first()
    .click();
  // Wait for the Transactions screen itself: the account detail also has a "Descrição" column and
  // a "Novo lançamento" button, and a dialog opened there closes when the navigation commits.
  await expect(page.getByRole('heading', { level: 1, name: /Lançamentos/ })).toBeVisible();
  await expect(page.getByRole('columnheader', { name: 'Descrição' })).toBeVisible();
  // The screen finishes loading (on the Vite dev server, the first visit may reload the
  // page and close a dialog opened too early).
  await page.waitForLoadState('networkidle');
  const dialog = page.getByRole('dialog', { name: 'Novo lançamento' });
  // The button may render before the screen state; retry until it opens.
  await expect(async () => {
    await page
      .getByRole('button', { name: /Novo lançamento/ })
      .first()
      .click();
    await expect(dialog).toBeVisible({ timeout: 2000 });
  }).toPass();
  await dialog.getByLabel('Descrição').fill('Padaria do bairro');
  await dialog.getByLabel(/Valor \(BRL\)/).fill('27,50');
  await dialog.getByRole('button', { name: 'Lançar' }).click();
  await expect(dialog).toBeHidden();
  await expect(
    page.getByRole('gridcell', { name: 'Padaria do bairro', exact: true }),
  ).toBeVisible();

  // The account balance reflects the expense.
  await page
    .getByRole('link', { name: /Contas/ })
    .first()
    .click();
  await expect(card).toContainText('972,50');

  // Settings → History: undoing the transaction restores the balance.
  await page
    .getByRole('link', { name: /Ajustes/ })
    .first()
    .click();
  await page
    .getByRole('navigation', { name: 'Seções de ajustes' })
    .getByRole('button', { name: /Histórico de alterações/ })
    .click();
  await expect(page.getByRole('grid', { name: 'Alterações' })).toBeVisible();
  await page.getByText(/Lançamento criado: Padaria do bairro/).click();
  const detail = page.getByRole('region', { name: 'Detalhe' });
  await detail.getByRole('button', { name: 'Desfazer' }).click();
  await expect(detail.getByRole('button', { name: 'Refazer' })).toBeVisible();
  await page
    .getByRole('link', { name: /Contas/ })
    .first()
    .click();
  await expect(card).toContainText('1.000,00');
});
