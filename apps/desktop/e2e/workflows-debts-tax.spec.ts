import { expect, test } from '@playwright/test';

import { passGates } from './helpers';

/** Same password as vault.spec.ts (the files share the core). */
const PASSWORD = 'senha-e2e-muito-forte-123';

/**
 * Debts and IR against the real core: registers a Price loan, checks the schedule,
 * simulates and records an extraordinary amortization with no transaction, and opens the IR tab.
 */
test('registers a loan, simulates and records amortization; opens the IR tab', async ({ page }) => {
  await page.goto('/');
  await passGates(page);
  // The E2E core is shared by all files: creates the vault, or uses the one from vault.spec.ts
  // (open or locked).
  const unlock = page.getByRole('button', { name: 'Desbloquear' });
  const create = page.getByRole('button', { name: 'Criar cofre cifrado' });
  const shell = page.getByRole('link', { name: /Dívidas/ }).first();
  await expect(unlock.or(create).or(shell)).toBeVisible();
  if (await create.isVisible()) {
    await page.getByLabel('Nome da família').fill('Família E2E');
    await page.getByLabel('Membro 1', { exact: true }).fill('Ana');
    await page.getByLabel('Senha mestra', { exact: true }).fill(PASSWORD);
    await page.getByLabel('Confirmar senha').fill(PASSWORD);
    await page
      .getByText(
        'Entendi que, se esquecer a senha e não tiver o kit de recuperação, perco os dados.',
      )
      .click();
    await create.click();
  } else if (await unlock.isVisible()) {
    await page.getByLabel('Senha mestra').fill(PASSWORD);
    await unlock.click();
  }
  await expect(page.getByText('Família E2E').first()).toBeVisible();

  await page
    .getByRole('link', { name: /Dívidas/ })
    .first()
    .click();
  await expect(page.getByText('Cadastre o primeiro empréstimo ou financiamento')).toBeVisible();
  await page.getByRole('button', { name: 'Novo contrato' }).first().click();

  const dialog = page.getByRole('dialog', { name: 'Novo contrato' });
  await dialog.getByLabel('Nome do contrato').fill('Empréstimo da reforma');
  await dialog.getByLabel('Valor financiado').fill('24.000,00');
  await dialog.getByRole('radio', { name: 'Informar a taxa' }).click();
  await dialog.getByLabel('Taxa efetiva (% a.a.)').fill('18');
  await dialog.getByLabel('Prazo (meses)').fill('24');
  await dialog.getByRole('button', { name: 'Cadastrar contrato' }).click();
  await expect(dialog).toBeHidden();

  const detail = page.getByRole('region', { name: 'Contrato Empréstimo da reforma' });
  await expect(detail).toBeVisible();
  const table = detail.getByRole('grid', { name: 'Tabela de amortização · Empréstimo da reforma' });
  await detail.getByRole('radio', { name: /^Todas · 24/ }).click();
  await expect(table.getByRole('row')).toHaveCount(26); // header + 24 + total

  await detail.getByLabel('Valor da amortização').fill('5.000,00');
  const choices = detail.getByRole('radiogroup', { name: 'Forma de amortizar' });
  await expect(choices.getByRole('radio', { name: 'Reduzir prazo' })).toContainText(
    'Juros economizados',
  );
  await expect(choices.getByRole('radio', { name: 'Reduzir parcela' })).toContainText(
    'Juros economizados',
  );
  await detail.getByRole('button', { name: 'Registrar esta amortização' }).click();
  const extra = page.getByRole('dialog', { name: 'Amortização extraordinária' });
  await extra.getByRole('button', { name: 'Registrar amortização' }).click();
  await expect(page.getByText('Amortização registrada').first()).toBeVisible();
  // Shorten the term: the schedule gets shorter.
  await expect(detail.getByRole('radio', { name: /^Todas · (1\d|2[0-3])$/ })).toBeVisible();

  await page
    .getByRole('link', { name: /Relatórios/ })
    .first()
    .click();
  await page.getByRole('tab', { name: /Imposto de renda/ }).click();
  await expect(page.getByText('Não é aconselhamento tributário').first()).toBeVisible();
  await expect(page.getByText('Nenhuma venda de renda variável neste ano.')).toBeVisible();
});
