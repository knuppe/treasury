import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog } from '@/api/mock';
import { mockChatStarts, mockPageContexts, mockSetSuggestions } from '@/api/mock/ai';
import type { Proposal } from '@/api/generated/Proposal';
import { useCompanion } from '@/state/companion';
import { renderApp } from '@/test/renderApp';

import { fitContext, MAX_CONTEXT_CHARS } from './pageContext';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

const panel = () => screen.getByRole('complementary', { name: 'Agente companheiro' });
const input = () => within(panel()).getByLabelText('Mensagem para o agente (painel)');

async function openPanel(user: ReturnType<typeof userEvent.setup>) {
  await user.click(await screen.findByRole('button', { name: 'Abrir o agente' }));
  return panel();
}

async function pending(): Promise<Proposal[]> {
  return (await mockCall('ai.list_proposals', { status: 'pending' })) as Proposal[];
}

describe('Companion agent (side panel)', () => {
  it('opens via the button and ⌘J, and keeps the conversation when switching screens', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    expect(screen.queryByRole('complementary', { name: 'Agente companheiro' })).toBeNull();
    const p = await openPanel(user);
    // Two engines ready on the chat route: the header lets you switch in this conversation.
    expect(within(p).getByRole('button', { name: 'Trocar o motor desta conversa' })).toBeTruthy();
    await user.type(input(), 'oi agente{Enter}');
    expect(await within(panel()).findByText(/Propus a categoria Transporte/)).toBeTruthy();

    // Navigates to Accounts: the panel stays open with the same conversation.
    await user.click(screen.getByRole('link', { name: /Contas/ }));
    await screen.findByRole('heading', { level: 1, name: /Contas/ });
    expect(within(panel()).getAllByText('oi agente').length).toBeGreaterThan(0);
    expect(within(panel()).getByText(/Propus a categoria Transporte/)).toBeTruthy();

    // ⌘J closes and reopens, without losing the conversation.
    fireEvent.keyDown(window, { key: 'j', code: 'KeyJ', metaKey: true, ctrlKey: true });
    await waitFor(() =>
      expect(screen.queryByRole('complementary', { name: 'Agente companheiro' })).toBeNull(),
    );
    fireEvent.keyDown(window, { key: 'j', code: 'KeyJ', metaKey: true, ctrlKey: true });
    expect(await within(panel()).findByText(/Propus a categoria Transporte/)).toBeTruthy();
    expect(useCompanion.getState().conversationId).toBeTruthy();
  });

  it('switches the engine in this conversation from the next message, without changing the default', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    const p = await openPanel(user);
    const picker = within(p).getByRole('button', { name: 'Trocar o motor desta conversa' });
    expect(picker.textContent).toContain('Claude');
    await user.click(picker);
    const menu = within(p).getByRole('menu', { name: 'Trocar o motor desta conversa' });
    const items = within(menu).getAllByRole('menuitemradio');
    expect(items[0]!.getAttribute('aria-checked')).toBe('true'); // Automatic
    await user.click(items.find((i) => /Ollama/.test(i.textContent ?? ''))!);
    expect(picker.textContent).toContain('Ollama');
    await user.type(input(), 'oi{Enter}');
    await waitFor(() => expect(mockChatStarts()).toHaveLength(1));
    expect(mockChatStarts()[0]!.engine_id).toBe('mock-ai-engine-ollama');
    await waitFor(() => expect(within(panel()).queryByText('Respondendo')).toBeNull());
    // The conversation continues with the chosen engine.
    await user.type(input(), 'e mais?{Enter}');
    await waitFor(() => expect(mockChatStarts()).toHaveLength(2));
    expect(mockChatStarts()[1]!.engine_id).toBe('mock-ai-engine-ollama');
    // The route (the default) did not change, and a new conversation goes back to automatic.
    const routes = (await mockCall('ai.routes.list', {})) as {
      task: string;
      engine_ids: string[];
    }[];
    expect(routes.find((r) => r.task === 'chat')!.engine_ids[0]).toBe('mock-ai-engine-claude');
    await waitFor(() =>
      expect(within(panel()).getByRole('button', { name: 'Nova conversa' })).not.toHaveProperty(
        'disabled',
        true,
      ),
    );
    await user.click(within(panel()).getByRole('button', { name: 'Nova conversa' }));
    await user.type(input(), 'outra{Enter}');
    await waitFor(() => expect(mockChatStarts()).toHaveLength(3));
    expect(mockChatStarts()[2]!.engine_id).toBeUndefined();
  });

  it('with a single engine, there is no switching', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await mockCall('ai.routes.set', { task: 'chat', engine_ids: ['mock-ai-engine-claude'] });
    const p = await openPanel(user);
    expect(
      await within(p).findByRole('status', { name: 'Motor que recebe os dados' }),
    ).toBeTruthy();
    expect(within(p).queryByRole('button', { name: 'Trocar o motor desta conversa' })).toBeNull();
  });

  it('computer access: off by default, granted only in the conversation where it was turned on', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    const p = await openPanel(user);
    const sw = within(p).getByRole('switch', { name: 'Acesso ao computador nesta conversa' });
    expect(sw.getAttribute('aria-checked')).toBe('false');
    await user.type(input(), 'oi{Enter}');
    expect(await within(panel()).findByText(/Propus a categoria Transporte/)).toBeTruthy();
    expect(mockChatStarts()[0]!.computer_access).toBeUndefined();

    // Turn it on in this conversation: the following messages carry the grant. The conversation created in the
    // first reply may redraw the panel: the switch is looked up again.
    const access = () =>
      within(panel()).getByRole('switch', { name: 'Acesso ao computador nesta conversa' });
    await user.click(access());
    await waitFor(() => expect(access().getAttribute('aria-checked')).toBe('true'));
    await user.type(input(), 'leia o PDF que baixei{Enter}');
    await waitFor(() => expect(mockChatStarts()).toHaveLength(2));
    expect(mockChatStarts()[1]).toMatchObject({ computer_access: true });
    await waitFor(() =>
      expect(within(panel()).getAllByText(/Propus a categoria Transporte/)).toHaveLength(2),
    );

    // A new conversation starts blocked.
    await user.click(within(panel()).getByRole('button', { name: 'Nova conversa' }));
    const fresh = within(panel()).getByRole('switch', {
      name: 'Acesso ao computador nesta conversa',
    });
    expect(fresh.getAttribute('aria-checked')).toBe('false');
    await user.type(input(), 'e agora?{Enter}');
    await waitFor(() => expect(mockChatStarts()).toHaveLength(3));
    expect(mockChatStarts()[2]!.computer_access).toBeUndefined();
  });

  it('sends the page context (with preview) and lets you not send it', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    await screen.findByRole('heading', { level: 1, name: /Contas/ });
    const p = await openPanel(user);
    const chip = await within(p).findByRole('button', {
      name: /Contexto desta página: 03 · Contas/,
    });
    await user.click(chip);
    const preview = within(p).getByRole('region', { name: 'Prévia do contexto da página' });
    await waitFor(() => expect(preview.textContent).toContain('"screen": "contas"'));
    expect(preview.textContent).toContain('in_accounts');

    await user.type(input(), 'quanto tenho?{Enter}');
    await within(panel()).findByText(/Propus a categoria Transporte/);
    const sent = mockPageContexts();
    expect(sent).toHaveLength(1);
    const ctx = sent[0] as Record<string, unknown>;
    expect(ctx.screen).toBe('contas');
    expect(ctx.route).toBe('/contas');
    expect(Array.isArray(ctx.items)).toBe(true);
    expect(JSON.stringify(ctx)).not.toMatch(/cookie|session/i);

    // Turn off: the next message goes without context.
    await user.click(within(panel()).getByRole('checkbox', { name: 'Enviar contexto' }));
    await waitFor(() => expect(within(panel()).getByText(/Desligado: só a mensagem/)).toBeTruthy());
    await user.type(input(), 'e agora?{Enter}');
    await waitFor(() => expect(mockCallLog().filter((c) => c === 'ai.chat_start').length).toBe(2));
    expect(mockPageContexts()).toHaveLength(1);
  });

  it('shows proactive suggestions for the screen and asks with one click', async () => {
    const user = userEvent.setup();
    mockSetSuggestions(null);
    renderApp('unlocked', {}, '#/lancamentos');
    mockSetSuggestions([
      {
        id: 'uncategorized',
        kind: 'uncategorized',
        severity: 'info',
        title: '7 lançamentos sem categoria',
        detail: 'Nos últimos 90 dias.',
        prompt: 'Categorize os 7 lançamentos sem categoria.',
        route: '/lancamentos?uncategorized=1',
        screens: ['lancamentos'],
        count: 7,
      },
      {
        id: 'b3_import',
        kind: 'b3_import',
        severity: 'info',
        title: 'Importar operações da B3',
        detail: 'XP: sem operações recentes.',
        prompt: 'Como importo da B3?',
        route: '/carteira',
        screens: ['carteira'],
        count: 1,
      },
    ]);
    await screen.findAllByRole('heading', { name: /Lançamentos/ });
    const p = await openPanel(user);
    const list = await within(p).findByRole('region', { name: 'Sugestões do agente' });
    const cards = await within(list).findAllByRole('listitem');
    // The one for the open screen comes first.
    expect(cards[0]!.textContent).toContain('7 lançamentos sem categoria');
    expect(within(list).getByText(/Estou vendo/).textContent).toContain('04 · Lançamentos');
    expect(mockCallLog()).toContain('ai.suggestions');
    await user.click(
      within(list).getByRole('button', {
        name: 'Perguntar ao agente: 7 lançamentos sem categoria',
      }),
    );
    expect(await within(panel()).findByText(/Propus a categoria Transporte/)).toBeTruthy();
    expect(within(panel()).getAllByText('Categorize os 7 lançamentos sem categoria.').length).toBe(
      1,
    );
  });

  it('guided setup with no accounts: suggestion and quick request "Onde vocês têm contas?"', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    const p = await openPanel(user);
    const quick = within(p).getByRole('group', { name: 'Pedidos rápidos' });
    expect(within(quick).getByRole('button', { name: 'O que posso fazer aqui?' })).toBeTruthy();
    await user.click(within(quick).getByRole('button', { name: 'Onde vocês têm contas?' }));
    expect(
      await within(panel()).findByText(/Quero cadastrar onde a família tem dinheiro/),
    ).toBeTruthy();
    expect(mockPageContexts()[0]).toMatchObject({ screen: 'painel' });
  });

  it('approves and rejects proposals in the conversation itself', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await openPanel(user);
    const before = (await pending()).length;
    await user.type(input(), 'categorize{Enter}');
    await within(panel()).findByText(/Propus a categoria Transporte/);
    // The live reply becomes the saved message (the card is redrawn).
    await waitFor(() => expect(within(panel()).queryByText(/Em andamento|Concluído/)).toBeNull());
    const card = await within(panel()).findByRole('article', {
      name: /Proposta: Categorizar “UBER \*TRIP” como Transporte/,
    });
    expect(within(card).getByText('Reversível · lote')).toBeTruthy();
    expect((await pending()).length).toBe(before + 1);
    await user.click(within(card).getByRole('button', { name: 'Aprovar' }));
    const name = /Proposta: Categorizar “UBER \*TRIP” como Transporte/;
    await waitFor(() =>
      expect(
        within(within(panel()).getByRole('article', { name })).getByText('Aprovada'),
      ).toBeTruthy(),
    );
    expect((await pending()).length).toBe(before);
    expect(
      within(within(panel()).getByRole('article', { name })).queryByRole('button', {
        name: 'Aprovar',
      }),
    ).toBeNull();

    // Second proposal: reject it right there.
    await user.type(input(), 'de novo{Enter}');
    await waitFor(() => expect(within(panel()).getAllByRole('article', { name })).toHaveLength(2));
    await waitFor(() => expect(within(panel()).queryByText(/Em andamento|Concluído/)).toBeNull());
    const open = within(panel())
      .getAllByRole('article', { name })
      .find((a) => within(a).queryByRole('button', { name: 'Rejeitar' }))!;
    await user.click(within(open).getByRole('button', { name: 'Rejeitar' }));
    await waitFor(() => expect(within(panel()).getAllByText('Rejeitada')).toHaveLength(1));
    expect((await pending()).length).toBe(before);
  });

  it('turns autonomous mode on and off in the options; when on, the header says so', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    const p = await openPanel(user);
    expect(within(p).queryByText('Autônomo')).toBeNull();
    await user.click(within(p).getByRole('button', { name: 'Opções do agente' }));
    const menu = within(p).getByRole('dialog', { name: 'Opções do agente' });
    const sw = within(menu).getByRole('switch', { name: 'Modo autônomo' });
    await waitFor(() => expect(sw.getAttribute('aria-checked')).toBe('false'));
    await user.click(sw);
    await waitFor(() => expect(sw.getAttribute('aria-checked')).toBe('true'));
    const s = (await mockCall('ai.settings.get', {})) as { autonomous: boolean };
    expect(s.autonomous).toBe(true);
    expect(within(p).getByText('Autônomo')).toBeTruthy();
    // Esc closes the options.
    await user.keyboard('{Escape}');
    expect(within(p).queryByRole('dialog', { name: 'Opções do agente' })).toBeNull();
  });

  it('previous conversations in their own view, outside the thread; suggestions only in the empty conversation', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    const p = await openPanel(user);
    const log = within(p).getByRole('log', { name: 'Conversa no painel do agente' });
    // Empty conversation: quick requests visible, no conversation listed in the thread.
    expect(within(log).getByRole('group', { name: 'Pedidos rápidos' })).toBeTruthy();
    expect(within(p).queryByText('Restaurantes em setembro')).toBeNull();

    await user.type(input(), 'oi{Enter}');
    expect(await within(panel()).findByText(/Propus a categoria Transporte/)).toBeTruthy();
    expect(within(panel()).queryByRole('group', { name: 'Pedidos rápidos' })).toBeNull();
    expect(within(panel()).queryByRole('region', { name: 'Sugestões do agente' })).toBeNull();

    // The history takes the place of the thread (and the field) while open.
    const toggle = within(panel()).getByRole('button', { name: 'Conversas anteriores' });
    await user.click(toggle);
    expect(toggle.getAttribute('aria-pressed')).toBe('true');
    const history = within(panel()).getByRole('region', { name: 'Conversas anteriores' });
    expect(within(panel()).queryByRole('log')).toBeNull();
    const recent = await within(history).findByRole('list', { name: 'Conversas recentes' });
    await waitFor(() => expect(within(recent).getAllByRole('listitem')).toHaveLength(2));
    // The current conversation is marked; searching filters by title.
    expect(within(recent).getByRole('button', { current: true }).textContent).toContain('oi');
    await user.type(within(history).getByRole('searchbox'), 'restaur');
    await waitFor(() => expect(within(recent).getAllByRole('listitem')).toHaveLength(1));

    // Pinning moves it to "Fixadas"; opening swaps the thread for the chosen conversation.
    await user.click(
      within(history).getByRole('button', { name: 'Fixar a conversa Restaurantes em setembro' }),
    );
    const pinned = await within(history).findByRole('list', { name: 'Conversas fixadas' });
    await user.click(within(pinned).getByRole('button', { name: /^Restaurantes em setembro/ }));
    expect(within(panel()).queryByRole('region', { name: 'Conversas anteriores' })).toBeNull();
    expect(
      await within(panel()).findByText('Quanto gastamos com restaurantes em setembro?'),
    ).toBeTruthy();
    expect(within(panel()).queryByText(/Propus a categoria Transporte/)).toBeNull();
    expect(useCompanion.getState().conversationId).toBe('mock-ai-conv-1');

    // A new conversation from the history view goes back to the empty thread.
    await user.click(within(panel()).getByRole('button', { name: 'Conversas anteriores' }));
    await user.click(within(panel()).getByRole('button', { name: 'Nova' }));
    expect(useCompanion.getState().conversationId).toBeNull();
    expect(within(panel()).getByRole('group', { name: 'Pedidos rápidos' })).toBeTruthy();
  });

  it('the command bar opens the panel with the question in the field', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('heading', { name: /Painel/ });
    act(() => useCompanion.getState().ask('quanto gastei com mercado?'));
    await waitFor(() =>
      expect((input() as HTMLTextAreaElement).value).toBe('quanto gastei com mercado?'),
    );
    expect(mockCallLog()).not.toContain('ai.chat_start');
    await user.click(within(panel()).getByRole('button', { name: 'Fechar o agente' }));
    expect(screen.getByRole('button', { name: 'Abrir o agente' })).toBeTruthy();
  });
});

describe('page context', () => {
  it('trims the lines to fit the limit', () => {
    const items = Array.from({ length: 200 }, (_, i) => ({ i, text: 'x'.repeat(200) }));
    const fit = fitContext({ screen: 'lancamentos', title: 't', items })!;
    expect(fit.items.length).toBeLessThanOrEqual(15);
    expect(JSON.stringify(fit).length).toBeLessThanOrEqual(MAX_CONTEXT_CHARS);
    expect(fitContext({ screen: 's', title: 'x'.repeat(MAX_CONTEXT_CHARS + 10) })).toBeNull();
  });
});
