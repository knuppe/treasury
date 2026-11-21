import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

test('conta, lançamento, saldo e desfazer no núcleo real', async ({ page }) => {
  await openVault(page);

  // Contas: nova conta corrente com saldo inicial.
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
  // Depois de criar, a conta abre no detalhe; na lista, é a linha dela. Nos dois, a tela traz o
  // nome e o saldo.
  const card = page.getByRole('main');
  await expect(card).toContainText('Banco E2E');
  await expect(card).toContainText('1.000,00');

  // Lançamentos: uma despesa nessa conta.
  await page
    .getByRole('link', { name: /Lançamentos/ })
    .first()
    .click();
  await expect(page.getByRole('columnheader', { name: 'Descrição' })).toBeVisible();
  // A tela termina de carregar (no Vite de desenvolvimento, a primeira visita pode recarregar a
  // página e fechar um diálogo aberto cedo demais).
  await page.waitForLoadState('networkidle');
  const dialog = page.getByRole('dialog', { name: 'Novo lançamento' });
  // O botão pode ser renderizado antes do estado da tela; tenta de novo até abrir.
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

  // O saldo da conta reflete a despesa.
  await page
    .getByRole('link', { name: /Contas/ })
    .first()
    .click();
  await expect(card).toContainText('972,50');

  // Ajustes → Histórico: desfazer o lançamento volta o saldo.
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
