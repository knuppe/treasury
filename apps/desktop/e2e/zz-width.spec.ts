// The measuring code runs in the page, so this file also needs the DOM types.
/// <reference lib="dom" />
import { expect, test, type Page } from '@playwright/test';

import { openVault } from './helpers';

/**
 * Nothing wider than the window: at the app's default size (1440×900), in the demo vault (real
 * data from the core), every screen and every tab must fit. A table or block wider than its box
 * shows a column cut in half, with nothing telling the person there is more to the side.
 *
 * Runs last: it switches the core to the demo vault.
 */
const SCREENS = [
  'painel',
  'familia',
  'contas',
  'lancamentos',
  'orcamento',
  'consumo',
  'dividas',
  'carteira',
  'ativos',
  'mercado',
  'relatorios',
  'impostos',
  'agente',
  'ajustes',
  'calculadoras',
];

/** Tolerance for sub-pixel rounding and scrollbars. */
const SLACK = 8;

interface Overflow {
  where: string;
  extra: number;
  text: string;
}

/** Outermost blocks inside <main> whose content is wider than their box. */
function overflows(page: Page, where: string): Promise<Overflow[]> {
  return page.evaluate(
    ({ where, slack }) => {
      const main = document.querySelector('main');
      if (!main) return [];
      const found: { el: Element; extra: number }[] = [];
      for (const el of main.querySelectorAll('*')) {
        if (el.children.length === 0) continue;
        const box = el.getBoundingClientRect();
        if (box.width < 200 || box.height < 20) continue;
        if (getComputedStyle(el).overflowX === 'visible') continue;
        const extra = el.scrollWidth - el.clientWidth;
        if (extra > slack) found.push({ el, extra });
      }
      return found
        .filter((f) => !found.some((g) => g !== f && g.el.contains(f.el)))
        .map((f) => ({
          where,
          extra: Math.round(f.extra),
          text: (f.el.textContent ?? '').replace(/\s+/g, ' ').trim().slice(0, 80),
        }));
    },
    { where, slack: SLACK },
  );
}

test('every screen and tab fits the default window in the demo vault', async ({ page }) => {
  test.setTimeout(240_000);
  await page.setViewportSize({ width: 1440, height: 900 });
  await openVault(page);
  // Demo vault, from the vault menu: the "Classe média com folga" family, opens without password.
  await page
    .getByRole('button', { name: /Cofres/ })
    .first()
    .click();
  await page.getByRole('button', { name: 'Cofre de demonstração' }).click();
  await page.getByRole('button', { name: 'Criar e abrir' }).click();
  await expect(page.getByText('DEMONSTRAÇÃO').first()).toBeVisible({ timeout: 60_000 });
  // Animations off, so every card is measured at its final size.
  await page.addStyleTag({
    content:
      '*,*::before,*::after{animation-duration:0s!important;transition-duration:0s!important}',
  });

  const all: Overflow[] = [];
  for (const screen of SCREENS) {
    await page.evaluate((s) => {
      window.location.hash = `#/${s}`;
    }, screen);
    await expect(page.locator('main h1').first()).toBeVisible();
    await page.waitForTimeout(800);
    all.push(...(await overflows(page, screen)));
    const tabs = page.locator('main [role=tab]');
    const n = await tabs.count();
    for (let i = 1; i < n; i += 1) {
      const name = ((await tabs.nth(i).textContent()) ?? '').trim();
      await tabs.nth(i).click();
      await page.waitForTimeout(600);
      all.push(...(await overflows(page, `${screen} › ${name}`)));
    }
  }
  expect(all, JSON.stringify(all, null, 1)).toEqual([]);
});
