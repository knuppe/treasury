/**
 * Agente companheiro contra o núcleo real, sem rede e sem motor de IA: o painel abre pelo
 * botão lateral e por ⌘J, mostra as sugestões determinísticas (`ai.suggestions`) e a prévia
 * do contexto da tela, e o interruptor do modo autônomo grava no cofre. Nenhuma mensagem é
 * enviada.
 */
import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

test('painel do agente: sugestões, contexto da tela e modo autônomo no núcleo real', async ({
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
  // O núcleo responde as sugestões sem motor (lista ou "nada pendente").
  await expect(
    suggestions.getByRole('listitem').first().or(suggestions.getByText('Nada pendente por aqui.')),
  ).toBeVisible();

  // Prévia do contexto que iria com a mensagem.
  await panel.getByRole('button', { name: /Contexto desta página: \d\d · Contas/ }).click();
  const preview = panel.getByRole('region', { name: 'Prévia do contexto da página' });
  await expect(preview).toContainText('"screen": "contas"');
  await expect(preview).toContainText('"route": "/contas"');

  // Modo autônomo (em Opções do agente): liga, recarrega (persistiu no cofre) e desliga de novo.
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

  // ⌘J / Ctrl+J fecha.
  await page.keyboard.press('ControlOrMeta+j');
  await expect(again).toBeHidden();
  await expect(page.getByRole('button', { name: 'Abrir o agente' })).toBeVisible();
});
