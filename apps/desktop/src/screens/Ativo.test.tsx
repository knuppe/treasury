/**
 * Página dedicada do ativo (docs/asset-page.md) contra o mock: rota própria, trilha pela tela de
 * origem (Mercado ou Ativos), o agente com o contexto do ativo e a conversa guardada na página.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeAll, describe, expect, it, vi } from 'vitest';

import { mockChatStarts } from '@/api/mock/ai';
import { quietCanvas } from '@/features/portfolio/testing';
import { useCompanion } from '@/state/companion';
import { useNavOrigin } from '@/state/navOrigin';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));
beforeAll(quietCanvas);

const crumb = () => screen.getByRole('navigation', { name: 'Trilha' });

describe('página do ativo', () => {
  it('vindo do Mercado, a trilha volta ao mesmo recorte; o menu acende Mercado', async () => {
    const user = userEvent.setup();
    useNavOrigin.setState({ origin: null });
    renderApp('unlocked', {}, '#/mercado?aba=rankings');
    const grid = await screen.findByRole('grid', { name: /^Ranking ·/ }, { timeout: 15_000 });
    await user.click(within(grid).getByRole('link', { name: 'PETR4' }));
    await waitFor(() => expect(window.location.hash).toContain('#/ativo/'));
    const link = within(await screen.findByRole('navigation', { name: 'Trilha' })).getByRole(
      'link',
      { name: 'Mercado' },
    );
    expect(link.getAttribute('href')).toBe('#/mercado?aba=rankings');
    expect(within(crumb()).getByText('PETR4').getAttribute('aria-current')).toBe('page');
    // Sem as abas da tela Ativos em volta.
    expect(screen.queryByRole('tablist', { name: 'Seções de ativos' })).toBeNull();
    await user.click(link);
    await waitFor(() => expect(window.location.hash).toBe('#/mercado?aba=rankings'));
  });

  it('link antigo segue para a rota nova; abrindo direto, a trilha é Ativos', async () => {
    useNavOrigin.setState({ origin: null });
    renderApp('unlocked', {}, '#/ativos?ativo=ast-petr4&secao=saude');
    await waitFor(() => expect(window.location.hash).toBe('#/ativo/ast-petr4?secao=saude'));
    expect(
      within(await screen.findByRole('navigation', { name: 'Trilha' })).getByRole('link', {
        name: 'Ativos',
      }),
    ).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Saúde' }).getAttribute('aria-selected')).toBe('true');
  });

  it('o ticker de um ativo cadastrado vai para a página dele (uma página por ativo)', async () => {
    renderApp('unlocked', {}, '#/ativo/petr4');
    await waitFor(() => expect(window.location.hash).toBe('#/ativo/ast-petr4'));
    expect(await screen.findByRole('heading', { level: 2, name: /PETR4/ })).toBeTruthy();
  });

  it('perguntar ao agente na página: contexto do ativo e a conversa fica guardada nela', async () => {
    const user = userEvent.setup();
    useCompanion.setState({ open: false, conversationId: null, draft: null });
    renderApp('unlocked', {}, '#/ativo/ast-petr4');
    await screen.findByRole('heading', { level: 2, name: /PETR4/ });
    await user.click(screen.getByRole('button', { name: 'Perguntar ao agente' }));
    const panel = await screen.findByRole('complementary', { name: 'Agente companheiro' });
    const box = within(panel).getByLabelText<HTMLTextAreaElement>(
      'Mensagem para o agente (painel)',
    );
    await waitFor(() => expect(box.value).toMatch(/^Analise PETR4/));
    await user.click(within(panel).getByRole('button', { name: 'Enviar' }));
    await waitFor(() => expect(mockChatStarts()).toHaveLength(1));
    const ctx = mockChatStarts()[0]!.page_context as {
      screen: string;
      selection: { asset_id: string };
    };
    expect(ctx.screen).toBe('ativo');
    expect(ctx.selection.asset_id).toBe('ast-petr4');
    // A aba Conversas mostra a conversa começada aqui.
    await user.click(screen.getByRole('tab', { name: 'Conversas' }));
    const list = await screen.findByRole('region', { name: 'Conversas' });
    expect(await within(list).findByText(/^Analise PETR4/)).toBeTruthy();
    expect(within(list).getByRole('button', { name: 'Abrir' })).toBeTruthy();
  });

  it('visão geral com o básico e a aba Notícias, com o Google só como link', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativo/ast-petr4');
    const overview = await screen.findByRole('region', { name: 'Resumo do ativo' });
    expect(within(overview).getByText('Indicadores principais')).toBeTruthy();
    expect(await within(overview).findByText(/PETR4 sobe após resultado/)).toBeTruthy();
    await user.click(within(overview).getByRole('button', { name: 'Ver todas as notícias' }));
    const news = await screen.findByRole('region', { name: 'Notícias' });
    expect(within(news).getByText(/PETR4 sobe após resultado/)).toBeTruthy();
    // O app não consulta o Google: só oferece o link da busca, que abre no navegador.
    const link = within(news).getByRole('link', { name: 'Abrir no Google Notícias' });
    expect(link.getAttribute('href')).toBe(
      'https://news.google.com/search?q=PETR4&hl=pt-BR&gl=BR&ceid=BR:pt-419',
    );
  });
});
