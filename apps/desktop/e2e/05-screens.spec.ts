import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

/** All screens and tabs: none may fall into the "Falha na interface" (interface failure) screen. */
const ROUTES = [
  '/painel',
  '/contas',
  '/lancamentos',
  '/orcamento',
  '/dividas',
  '/carteira',
  '/carteira?aba=operacoes',
  '/carteira?aba=proventos',
  '/carteira?aba=rentabilidade',
  '/carteira?aba=alocacao',
  '/carteira?aba=bens',
  '/ativos',
  '/ativos?aba=rankings',
  '/ativos?aba=screener',
  '/ativos?aba=comparador',
  '/ativos?aba=dados',
  '/relatorios',
  '/relatorios?v=ir',
  '/impostos',
  '/impostos?cenario=elevado&grupos=1',
  '/agente',
  '/agente?aba=aprovacao',
  '/agente?aba=alertas',
  '/agente?aba=ferramentas',
  '/agente?aba=memoria',
  '/agente?aba=configurar',
  '/ajustes',
  '/ajustes?secao=backups',
  '/ajustes?secao=historico',
  '/ajustes?secao=preferencias',
  '/ajustes?secao=fontes',
  '/ajustes?secao=dados',
  '/ajustes?secao=ia',
  '/ajustes?secao=mcp',
  '/ajustes?secao=categorias',
  '/ajustes?secao=familia',
];

test('all screens open without an interface failure', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', (e) => {
    // WebKit reports this when a ResizeObserver callback changes the layout again; it is harmless
    // and the spec treats it as a warning, not as an application error.
    if (e.message.startsWith('ResizeObserver loop')) return;
    // WebKit also reports a request to the E2E HTTP bridge that a navigation cancelled as an
    // "access control" failure. The bridge exists only in tests; the app talks to the core over
    // Tauri IPC.
    if (e.message.endsWith('/api due to access control checks.')) return;
    errors.push(`${page.url()}: ${e.message}`);
  });
  await openVault(page);
  for (const route of ROUTES) {
    await page.goto(`/#${route}`);
    await expect(page.locator('h1').first(), route).toBeVisible();
    await expect(page.getByText('Falha na interface'), route).toHaveCount(0);
  }
  expect(errors).toEqual([]);
});
