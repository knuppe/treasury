/**
 * Agent › Configure against the real core: registering a local engine (no network),
 * chat route and spending limit, persisted in the vault. No model call is made:
 * the test never clicks "Testar" nor sends a message.
 *
 * The name starts with `z-` because Playwright runs files in alphabetical order and
 * `vault.spec.ts` needs to find the vault still to be created; this one creates or unlocks.
 */
import { expect, test, type Page } from '@playwright/test';

import { passGates } from './helpers';

const PASSWORD = 'senha-e2e-muito-forte-123';

async function openVault(page: Page) {
  await page.goto('/');
  await passGates(page);
  const create = page.getByLabel('Nome da família');
  const unlock = page.getByRole('button', { name: 'Desbloquear' });
  const shell = page.getByRole('navigation', { name: 'Telas' });
  // The core may already have the vault open (session from the previous test).
  await expect(create.or(unlock).or(shell)).toBeVisible();
  if (await shell.isVisible()) return;
  if (await create.isVisible()) {
    await create.fill('Família E2E');
    await page.getByLabel('Membro 1', { exact: true }).fill('Ana');
    await page.getByLabel('Senha mestra', { exact: true }).fill(PASSWORD);
    await page.getByLabel('Confirmar senha').fill(PASSWORD);
    await page
      .getByText(
        'Entendi que, se esquecer a senha e não tiver o kit de recuperação, perco os dados.',
      )
      .click();
    await page.getByRole('button', { name: 'Criar cofre cifrado' }).click();
  } else {
    await page.getByLabel('Senha mestra').fill(PASSWORD);
    await unlock.click();
  }
  await expect(page.getByText('Família E2E').first()).toBeVisible();
}

test('configures a local engine, chat route and spending limit on the real core', async ({
  page,
}) => {
  await openVault(page);
  await page.goto('/#/agente?aba=configurar');
  await expect(page.getByText('Nenhum motor cadastrado.', { exact: false })).toBeVisible();

  // Local engine (LM Studio): only saves the registration; nothing is called.
  await page.getByRole('button', { name: 'Adicionar motor' }).click();
  const dialog = page.getByRole('dialog', { name: 'Novo motor de IA' });
  await dialog.getByRole('combobox').first().click();
  await page.getByRole('option', { name: /LM Studio/ }).click();
  await dialog.getByLabel('Nome').fill('Estúdio local');
  await dialog.getByLabel('Modelo').fill('qwen3-8b');
  await dialog.getByLabel('URL do servidor local').fill('http://127.0.0.1:1234/v1');
  await dialog.getByRole('button', { name: 'Cadastrar' }).click();
  const engine = page.getByRole('listitem', { name: 'Motor Estúdio local' });
  await expect(engine).toBeVisible();
  await expect(engine.getByText('pronto')).toBeVisible();
  await expect(engine.getByText('dados ficam neste computador').first()).toBeVisible();

  // Chat route with the new engine.
  const chat = page.getByRole('listitem', { name: 'Rota: Chat com o agente' });
  await chat.getByRole('combobox', { name: /Adicionar motor à rota/ }).click();
  await page.getByRole('option', { name: /Estúdio local/ }).click();
  await expect(chat.getByText('Estúdio local')).toBeVisible();

  // Spending limit for an API provider ("Custos" part of the AI settings).
  const parts = page.getByRole('tablist', { name: 'Partes dos ajustes de IA' });
  await parts.getByRole('tab', { name: 'Custos' }).click();
  await page.getByRole('combobox', { name: 'Limitar outro provider' }).click();
  await page.getByRole('option', { name: 'Anthropic (Claude)' }).click();
  const limit = page.getByRole('listitem', { name: 'Limites: Anthropic (Claude)' });
  await limit.getByLabel('Por dia (USD)').fill('2,5');
  await limit.getByLabel('Confirmar acima de (USD)').fill('0,1');
  await limit.getByRole('button', { name: 'Salvar' }).click();
  await expect(page.getByText('Limites de Anthropic (Claude) salvos').first()).toBeVisible();

  // Everything persisted in the core: reload and check (the open part of the settings stays
  // in the URL).
  await page.reload();
  await page
    .getByRole('tablist', { name: 'Partes dos ajustes de IA' })
    .getByRole('tab', { name: 'Motores' })
    .click();
  await expect(page.getByRole('listitem', { name: 'Motor Estúdio local' })).toBeVisible();
  await expect(
    page.getByRole('listitem', { name: 'Rota: Chat com o agente' }).getByText('Estúdio local'),
  ).toBeVisible();
  await page
    .getByRole('tablist', { name: 'Partes dos ajustes de IA' })
    .getByRole('tab', { name: 'Custos' })
    .click();
  const saved = page.getByRole('listitem', { name: 'Limites: Anthropic (Claude)' });
  await expect(saved.getByLabel('Por dia (USD)')).toHaveValue('2,5');
  await expect(saved.getByLabel('Confirmar acima de (USD)')).toHaveValue('0,1');

  // The conversation shows the engine that will receive the data, and the queue starts empty.
  await page.getByRole('tab', { name: 'Conversa' }).click();
  const indicator = page.getByRole('status', { name: 'Motor que recebe os dados' });
  await expect(indicator.getByText('Estúdio local')).toBeVisible();
  await expect(indicator.getByText('dados ficam neste computador')).toBeVisible();
  await page.getByRole('tab', { name: /Aprovação/ }).click();
  await expect(page.getByText('Nenhuma proposta esperando aprovação.')).toBeVisible();
  await expect(page.getByText(/Modo autônomo desligado/)).toBeVisible();
});

test('procedure memory and computer access against the real core', async ({ page }) => {
  await openVault(page);
  // The conversation starts without computer access.
  await page.goto('/#/agente?aba=conversa');
  const sw = page.getByRole('switch', { name: 'Acesso ao computador nesta conversa' });
  await expect(sw).toHaveAttribute('aria-checked', 'false');
  await sw.click();
  await expect(sw).toHaveAttribute('aria-checked', 'true');

  // Note saved through the interface (the agent saves on its own; here only the user).
  await page.getByRole('tab', { name: /Memória/ }).click();
  await expect(page.getByText(/O agente ainda não gravou procedimentos/)).toBeVisible();
  await page.getByRole('button', { name: 'Adicionar' }).click();
  const add = page.getByRole('dialog', { name: 'Novo procedimento' });
  await add.getByLabel('Título').fill('Extrato OFX do banco');
  await add.getByLabel('Site').fill('https://www.banco.com.br/extrato?sessao=abc');
  await add.getByLabel('Formato').fill('ofx');
  await add.getByLabel('Passos').fill('Entrar (senha: 1234) › Extrato › Exportar › OFX');
  await add.getByRole('button', { name: 'Salvar' }).click();
  const row = page.getByRole('listitem', { name: 'Procedimento: Extrato OFX do banco' });
  await expect(row).toBeVisible();
  // The core keeps only the domain, the format in uppercase, and hides the password.
  await expect(row.getByText('www.banco.com.br')).toBeVisible();
  await expect(row.getByText('arquivo OFX')).toBeVisible();
  await expect(row.getByText(/senha: \[oculto\]/)).toBeVisible();
  await expect(row.getByText(/Gravado por você/)).toBeVisible();

  // Persisted in the vault; edit and delete.
  await page.reload();
  const saved = page.getByRole('listitem', { name: 'Procedimento: Extrato OFX do banco' });
  await saved.getByRole('button', { name: 'Editar' }).click();
  const edit = page.getByRole('dialog', { name: 'Editar procedimento' });
  await edit.getByLabel('Passos').fill('Menu Extratos › Exportar › OFX');
  await edit.getByRole('button', { name: 'Salvar' }).click();
  await expect(saved.getByText('Menu Extratos › Exportar › OFX')).toBeVisible();
  await saved.getByRole('button', { name: 'Apagar' }).click();
  await page
    .getByRole('dialog', { name: 'Apagar este procedimento' })
    .getByRole('button', { name: 'Apagar' })
    .click();
  await expect(page.getByText(/O agente ainda não gravou procedimentos/)).toBeVisible();
});
