import { expect, type Page } from '@playwright/test';

export const PASSWORD = 'senha-e2e-muito-forte-123';

/**
 * Screens that come before the vault on the first launch after install: the language choice, the
 * terms-of-use acceptance and the choice between a real vault and the demo (the real one is kept).
 * Goes through them (if they show up) until the vault (create, unlock or the app already open).
 */
export async function passGates(page: Page): Promise<void> {
  const create = page.getByRole('button', { name: 'Criar cofre cifrado' });
  const unlock = page.getByRole('button', { name: 'Desbloquear' });
  const shell = page.getByRole('link', { name: /Lançamentos/ }).first();
  const language = page.getByRole('heading', { name: 'Escolha o idioma' });
  const terms = page.getByRole('checkbox', { name: /^Li e entendi/ });
  const real = page.getByRole('radio', { name: /^Cofre real/ });
  for (let i = 0; i < 5; i += 1) {
    await expect(create.or(unlock).or(shell).or(language).or(terms).or(real)).toBeVisible();
    if (await language.isVisible()) {
      await page.getByRole('button', { name: 'Continuar' }).click();
      await expect(language).toBeHidden();
    } else if (await terms.isVisible()) {
      await terms.check();
      await page.getByRole('button', { name: 'Continuar' }).click();
      await expect(terms).toBeHidden();
    } else if ((await real.isVisible()) && !(await create.isVisible())) {
      await real.click();
      await expect(create).toBeVisible();
    } else {
      return;
    }
  }
}

/** Creates the vault on first launch or unlocks the one the previous test created. */
export async function openVault(page: Page, family = 'Família E2E'): Promise<void> {
  await page.goto('/');
  await passGates(page);
  const create = page.getByRole('button', { name: 'Criar cofre cifrado' });
  const unlock = page.getByRole('button', { name: 'Desbloquear' });
  const shell = page.getByRole('link', { name: /Lançamentos/ }).first();
  // The core stays unlocked between scenarios; in that case the app already opens on the dashboard.
  await expect(create.or(unlock).or(shell)).toBeVisible();
  if (await shell.isVisible()) return;
  if (await create.isVisible()) {
    await page.getByLabel('Nome da família').fill(family);
    await page.getByLabel('Membro 1', { exact: true }).fill('Ana');
    await page.getByLabel('Senha mestra', { exact: true }).fill(PASSWORD);
    await page.getByLabel('Confirmar senha').fill(PASSWORD);
    await page
      .getByText(
        'Entendi que, se esquecer a senha e não tiver o kit de recuperação, perco os dados.',
      )
      .click();
    await create.click();
    // New vault: profile welcome; the scenarios use advanced mode (everything visible).
    await page.getByRole('button', { name: /Já sei o que quero/ }).click();
  } else {
    await page.getByLabel('Senha mestra').fill(PASSWORD);
    await unlock.click();
  }
  await expect(shell).toBeVisible();
}
