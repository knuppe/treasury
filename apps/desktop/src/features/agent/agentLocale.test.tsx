/**
 * Tela Agente e painel lateral em inglês (docs/i18n-design.md): abas, caixa de mensagem,
 * fila de aprovação e o resumo das alterações saem do catálogo `agente`; os pedidos rápidos
 * do painel (`companheiro`) vão para o motor no idioma do app.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { mockChatStarts } from '@/api/mock/ai';
import type { ChangeItem } from '@/api/generated/ChangeItem';
import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { describeChanges, type NameBook } from './changeText';
import { summaryText } from './format';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  await applyLocale('pt-BR');
});

const NAMES: Record<string, string> = { 'category:c1': 'Groceries', 'category:c2': 'Other' };

const book: NameBook = {
  name: (kind, id) => NAMES[`${kind}:${id}`],
  txn: () => undefined,
};

function setCategory(seq: number, id: string): ChangeItem {
  return {
    seq,
    table_name: 'txn',
    row_id: id,
    op: 'update',
    before: { id, category_id: 'c2' },
    after: { id, category_id: 'c1' },
  };
}

describe('Agente em inglês', () => {
  it('abas, caixa de mensagem e fila de aprovação no idioma escolhido', async () => {
    await applyLocale('en');
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/agente?aba=conversa');
    const tabs = await screen.findByRole('tablist', { name: 'Agent sections' });
    for (const name of ['Conversation', 'Alerts', 'Tools', 'Memory', 'Set up']) {
      expect(within(tabs).getByRole('tab', { name: new RegExp(`^${name}`) })).toBeTruthy();
    }
    expect(await screen.findByLabelText('Message to the agent')).toBeTruthy();
    expect(screen.getByRole('status', { name: 'Engine that receives the data' })).toBeTruthy();

    await user.click(within(tabs).getByRole('tab', { name: /^Approval/ }));
    expect(await screen.findByText('Change in Set up')).toBeTruthy();
    expect(screen.getAllByRole('button', { name: /^Approve/ }).length).toBeGreaterThan(0);
  });

  it('resume as alterações em frases do idioma', async () => {
    await applyLocale('en');
    expect(summaryText({ txn: { update: 2 }, rule: { insert: 1 } })).toBe(
      'changes 2 transactions · creates 1 rule',
    );
    expect(describeChanges([setCategory(1, 't1'), setCategory(2, 't2')], book)).toEqual([
      'Changes the category of 2 transactions: Other → Groceries',
    ]);
  });

  it('painel lateral: pedido rápido vai para o motor em inglês', async () => {
    await applyLocale('en');
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await user.click(await screen.findByRole('button', { name: 'Open the agent' }));
    const panel = screen.getByRole('complementary', { name: 'Agent companion' });
    expect(within(panel).getByRole('heading', { name: 'How can I help?' })).toBeTruthy();
    expect(
      within(panel).getByText('AI can make mistakes. Check before approving.', { exact: false }),
    ).toBeTruthy();
    const quick = within(panel).getByRole('group', { name: 'Quick requests' });
    await user.click(within(quick).getByRole('button', { name: 'Summarize this screen' }));
    await waitFor(() => expect(mockChatStarts()).toHaveLength(1));
    expect(mockChatStarts()[0]!.message).toMatch(/^Summarize what this screen shows right now/);
  });
});
