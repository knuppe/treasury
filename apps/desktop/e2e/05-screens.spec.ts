import { expect, test } from '@playwright/test';

import { openVault } from './helpers';

/** Todas as telas e abas: nenhuma pode cair na tela de "Falha na interface". */
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

test('todas as telas abrem sem falha na interface', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', (e) => errors.push(`${page.url()}: ${e.message}`));
  await openVault(page);
  for (const route of ROUTES) {
    await page.goto(`/#${route}`);
    await expect(page.locator('h1').first(), route).toBeVisible();
    await expect(page.getByText('Falha na interface'), route).toHaveCount(0);
  }
  expect(errors).toEqual([]);
});
