import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

/**
 * Análise de ativos contra o núcleo real, sem coleta (nada de rede): universo vazio com
 * estado vazio explicativo nos Rankings e no Explorador do Mercado (o link antigo de Ativos
 * segue para lá), tela do screener salva pelo núcleo, desfeita e salva de novo, e os ajustes
 * da análise em Ativos (User-Agent da SEC). Sem avisos do React no console.
 */
test('screener salva tela e explica o universo vazio no núcleo real', async ({ page }) => {
  const warnings: string[] = [];
  page.on('console', (m) => {
    if (m.type() === 'error' || m.type() === 'warning') warnings.push(m.text());
  });
  await openVault(page);
  // Cofre novo: as boas-vindas levam ao painel depois de gravar o perfil; espera isso antes
  // de abrir o link, para a ida ao painel não atropelar a navegação.
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

  // Desfazer logo depois: a tela sai da lista.
  await page.getByRole('button', { name: 'Desfazer' }).click();
  await expect(page.getByText('Alteração desfeita').first()).toBeVisible();
  await expect(page.getByRole('combobox', { name: 'Telas salvas' })).toContainText(
    'Nenhuma tela salva',
  );

  // Salva de novo e reabre depois de recarregar.
  await page.getByRole('button', { name: 'Salvar tela…' }).click();
  await dialog.getByLabel('Nome').fill('Baratas com dividendos');
  await dialog.getByRole('button', { name: 'Salvar tela' }).click();
  await expect(dialog).toBeHidden();
  await page.reload();
  await page.getByRole('combobox', { name: 'Telas salvas' }).click();
  await page.getByRole('option', { name: 'Baratas com dividendos' }).click();
  await expect(page.getByText('Tela · Baratas com dividendos')).toBeVisible();
  await expect(page.getByLabel('Valor da condição 2')).toHaveValue('6');

  // Dados da análise (em Ativos): aviso da SEC e ajuste do e-mail.
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
