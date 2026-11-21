/**
 * Apoio ao Treasury contra o mock: a página em Ajustes › Sobre e apoio (só o GitHub Sponsors,
 * "em breve" sem endereço, nenhuma rede), o cartão da Visão geral depois de um evento de valor e
 * os textos vindos do catálogo `apoio.ftl` no idioma do app (não no do país da família).
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import { seedSupportEligible } from '@/api/mock/support';
import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { bugReport } from './texts';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  vi.restoreAllMocks();
  await applyLocale('pt-BR');
});

describe('Apoiar o Treasury', () => {
  it('mostra a página só com o GitHub Sponsors, em breve, sem rede', async () => {
    const fetchSpy = vi.spyOn(globalThis, 'fetch');
    renderApp('unlocked', {}, '#/ajustes?secao=sobre');
    expect(await screen.findByText(/Treasury é (gratuito|software livre)/)).toBeTruthy();
    const money = screen.getByRole('region', { name: /Com dinheiro/ });
    const names = within(money)
      .getAllByRole('heading', { level: 3 })
      .map((h) => h.textContent);
    expect(names).toEqual(['GitHub Sponsors']);
    expect(within(money).getAllByText('em breve')).toHaveLength(1);
    expect(within(money).queryByText(/Pix|Liberapay/)).toBeNull();
    expect(within(money).getByText(/não cobra taxa de contas pessoais/)).toBeTruthy();
    expect(screen.getByText('Sem anúncios.')).toBeTruthy();
    // Atribuição pedida pela licença do Lightweight Charts (NOTICE e link para a TradingView).
    expect(screen.getByText(/TradingView Lightweight Charts™, Copyright \(c\) 2025/)).toBeTruthy();
    expect(screen.getByRole('link', { name: 'tradingview.com' }).getAttribute('href')).toBe(
      'https://www.tradingview.com/',
    );
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it('o cartão só aparece depois de um evento de valor e some com "Agora não"', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    seedSupportEligible();
    await mockCall('support.event', { event: 'import' });
    await user.click(await screen.findByRole('link', { name: /Painel/ }));
    const card = await screen.findByRole('region', { name: 'Apoiar o Treasury' });
    expect(within(card).getByRole('button', { name: 'Já apoio' })).toBeTruthy();
    expect(within(card).getByRole('button', { name: 'Não mostrar mais' })).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Agora não' }));
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Apoiar o Treasury' })).toBeNull(),
    );
  });

  it('com "Ver como", o evento não libera o cartão', async () => {
    renderApp('unlocked', {}, '#/painel');
    seedSupportEligible();
    const r = (await mockCall('support.event', { event: 'import', view_as: true })) as {
      pending: boolean;
    };
    expect(r.pending).toBe(false);
  });

  it('o pedido de relato de erro sai do catálogo com o roteiro em linhas', async () => {
    expect(bugReport('0.7.1')).toBe(`Quero relatar um erro no Treasury (versão 0.7.1).

O que eu fiz:
O que eu esperava:
O que aconteceu:
Em que tela:

Me ajude a escrever o relatório sem dados pessoais (sem nomes, valores reais nem números de conta).`);
    await applyLocale('es');
    expect(bugReport('0.7.1')).toBe(`Quiero reportar un error en Treasury (versión 0.7.1).

Lo que hice:
Lo que esperaba:
Lo que pasó:
En qué pantalla:

Ayúdame a escribir el reporte sin datos personales (sin nombres, montos reales ni números de cuenta).`);
  });

  it('a página segue o idioma do app, só com o GitHub Sponsors', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/ajustes?secao=sobre');
    expect(await screen.findByText(/Treasury is free (and|software)/)).toBeTruthy();
    const money = screen.getByRole('region', { name: /With money/ });
    const names = within(money)
      .getAllByRole('heading', { level: 3 })
      .map((h) => h.textContent);
    expect(names).toEqual(['GitHub Sponsors']);
    expect(within(money).getAllByText('coming soon')).toHaveLength(1);
    expect(screen.getByText('No ads.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Report a bug' })).toBeTruthy();
  });
});
