/**
 * Companion agent against the real core, with no network and no AI engine: the panel opens from the
 * side button and with ⌘J, shows the deterministic suggestions (`ai.suggestions`) and the screen
 * context preview, and the autonomous-mode switch writes to the vault. No message is
 * sent.
 */
import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

test('agent panel: suggestions, screen context and autonomous mode on the real core', async ({
  page,
}) => {
  await openVault(page);
  await page.goto('/#/contas');
  await expect(page.getByRole('heading', { name: /Contas/ }).first()).toBeVisible();

  await page.getByRole('button', { name: 'Abrir o agente' }).click();
  const panel = page.getByRole('complementary', { name: 'Agente companheiro' });
  await expect(panel).toBeVisible();
  const suggestions = panel.getByRole('region', { name: 'Sugestões do agente' });
  await expect(suggestions.getByText(/Estou vendo/)).toContainText(/\d\d · Contas/);
  // The core answers the suggestions without an engine (a list or "nothing pending").
  await expect(
    suggestions.getByRole('listitem').first().or(suggestions.getByText('Nada pendente por aqui.')),
  ).toBeVisible();

  // Preview of the context that would go with the message.
  await panel.getByRole('button', { name: /Contexto desta página: \d\d · Contas/ }).click();
  const preview = panel.getByRole('region', { name: 'Prévia do contexto da página' });
  await expect(preview).toContainText('"screen": "contas"');
  await expect(preview).toContainText('"route": "/contas"');

  // Autonomous mode (in agent Options): turn on, reload (persisted in the vault) and turn off again.
  const options = (p: typeof panel) => {
    return p.getByRole('dialog', { name: 'Opções do agente' });
  };
  await panel.getByRole('button', { name: 'Opções do agente' }).click();
  const sw = options(panel).getByRole('switch', { name: 'Modo autônomo' });
  await expect(sw).toHaveAttribute('aria-checked', 'false');
  await sw.click();
  await expect(sw).toHaveAttribute('aria-checked', 'true');
  await page.reload();
  const again = page.getByRole('complementary', { name: 'Agente companheiro' });
  await expect(again).toBeVisible();
  await again.getByRole('button', { name: 'Opções do agente' }).click();
  const sw2 = options(again).getByRole('switch', { name: 'Modo autônomo' });
  await expect(sw2).toHaveAttribute('aria-checked', 'true');
  await sw2.click();
  await expect(sw2).toHaveAttribute('aria-checked', 'false');
  await page.keyboard.press('Escape');

  // ⌘J / Ctrl+J closes.
  await page.keyboard.press('ControlOrMeta+j');
  await expect(again).toBeHidden();
  await expect(page.getByRole('button', { name: 'Abrir o agente' })).toBeVisible();
});
