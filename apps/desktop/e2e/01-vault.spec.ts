import { expect, test } from '@playwright/test';

import { PASSWORD, passGates } from './helpers';

test('cria o cofre no núcleo real, bloqueia e desbloqueia', async ({ page }) => {
  await page.goto('/');
  await passGates(page);
  await page.getByLabel('Nome da família').fill('Família E2E');
  await page.getByLabel('Membro 1', { exact: true }).fill('Ana');
  await page.getByLabel('Senha mestra', { exact: true }).fill(PASSWORD);
  await page.getByLabel('Confirmar senha').fill(PASSWORD);
  await page
    .getByText('Entendi que, se esquecer a senha e não tiver o kit de recuperação, perco os dados.')
    .click();
  await page.getByRole('button', { name: 'Criar cofre cifrado' }).click();

  // Cofre novo: boas-vindas do perfil, com a opção de ir direto ao modo avançado.
  await expect(page.getByRole('heading', { name: 'Boas-vindas ao Treasury' })).toBeVisible();
  await page.getByRole('button', { name: /Já sei o que quero/ }).click();

  // Shell do app com a família e as 11 telas.
  await expect(page.getByText('Família E2E').first()).toBeVisible();
  await expect(page.getByRole('link', { name: /Lançamentos/ }).first()).toBeVisible();
  await expect(page.getByText('MOCK')).toHaveCount(0);

  await page.getByRole('button', { name: 'Bloquear cofre' }).first().click();
  await expect(page.getByRole('button', { name: 'Desbloquear' })).toBeVisible();

  await page.getByLabel('Senha mestra').fill('senha-errada-123456');
  await page.getByRole('button', { name: 'Desbloquear' }).click();
  await expect(page.getByText(/senha incorreta/i)).toBeVisible();

  await page.getByLabel('Senha mestra').fill(PASSWORD);
  await page.getByRole('button', { name: 'Desbloquear' }).click();
  await expect(page.getByText('Família E2E').first()).toBeVisible();
});
