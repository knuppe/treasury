import { expect, type Page } from '@playwright/test';

export const PASSWORD = 'senha-e2e-muito-forte-123';

/**
 * Telas que vêm antes do cofre na primeira abertura da instalação: a escolha do idioma, o
 * aceite dos termos de uso e a escolha entre cofre real e demonstração (fica o real). Passa por
 * elas (se aparecerem) até o cofre (criar, desbloquear ou o app já aberto).
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

/** Cria o cofre na primeira abertura ou desbloqueia o já criado pelo teste anterior. */
export async function openVault(page: Page, family = 'Família E2E'): Promise<void> {
  await page.goto('/');
  await passGates(page);
  const create = page.getByRole('button', { name: 'Criar cofre cifrado' });
  const unlock = page.getByRole('button', { name: 'Desbloquear' });
  const shell = page.getByRole('link', { name: /Lançamentos/ }).first();
  // O núcleo continua desbloqueado entre cenários; nesse caso o app já abre no painel.
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
    // Cofre novo: boas-vindas do perfil; os cenários usam o modo avançado (tudo à vista).
    await page.getByRole('button', { name: /Já sei o que quero/ }).click();
  } else {
    await page.getByLabel('Senha mestra').fill(PASSWORD);
    await unlock.click();
  }
  await expect(shell).toBeVisible();
}
