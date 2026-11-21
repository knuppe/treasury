import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { call } from '@/api/client';
import type { Attention } from '@/api/generated/Attention';
import type { MonthClose } from '@/api/generated/MonthClose';
import { renderApp } from '@/test/renderApp';

import { attentionExcerpt, attentionHref } from './AttentionCard';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('Para resolver', () => {
  it('junta tudo numa lista só no Painel, do mais grave para o menos grave', async () => {
    renderApp('unlocked', {}, '#/painel');
    const card = await screen.findByRole('region', { name: 'Para resolver' }, { timeout: 15_000 });
    const data = await call<Attention>('attention.list', {});
    expect(data.items.length).toBeGreaterThan(0);
    // A ordem da lista é a da gravidade.
    const rank = { critical: 0, warning: 1, info: 2 } as const;
    const order = data.items.map((i) => rank[i.severity]);
    expect([...order].sort((a, b) => a - b)).toEqual(order);
    expect(data.counts.total).toBe(data.items.length);
    // Nenhum fato duas vezes.
    expect(new Set(data.items.map((i) => i.id)).size).toBe(data.items.length);
    await within(card).findByText(data.items[0]!.title);
    // O sino da barra superior mostra a mesma contagem.
    const bell = screen.getByRole('button', { name: /para resolver$/ });
    expect(bell.textContent).toContain(String(Math.min(data.counts.total, 99)));
  });

  it('dispensar tira o item da lista e do sino', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel?resolver=1');
    const card = await screen.findByRole('region', { name: 'Para resolver' }, { timeout: 15_000 });
    const before = await call<Attention>('attention.list', {});
    const target = before.items.find((i) => !i.alert_id)!;
    // Vindo do sino, a lista aparece inteira.
    const title = await within(card).findByText(target.title);
    const row = title.closest('li')!;
    await user.click(within(row).getByRole('button', { name: 'Dispensar' }));
    await waitFor(() => expect(within(card).queryByText(target.title)).toBeNull());
    const after = await call<Attention>('attention.list', {});
    expect(after.counts.total).toBe(before.counts.total - 1);
    expect(after.items.some((i) => i.id === target.id)).toBe(false);
  });

  it('mostra o próximo passo com o degrau, o porquê e a ação', async () => {
    renderApp('unlocked', {}, '#/painel');
    const card = await screen.findByRole(
      'region',
      { name: 'Seu próximo passo' },
      { timeout: 15_000 },
    );
    expect(within(card).getAllByText('Degrau 4 de 6').length).toBeGreaterThan(0);
    expect(within(card).getByText('Montar a reserva de emergência')).toBeTruthy();
    expect(within(card).getByText(/O próximo marco é 3 meses/)).toBeTruthy();
    // Os seis degraus: três feitos, o atual e dois depois.
    const rungs = within(card).getAllByRole('listitem');
    expect(rungs.length).toBe(6);
    expect(within(card).getByLabelText('1. Saber para onde o dinheiro vai (feito)')).toBeTruthy();
    expect(within(card).getByLabelText('4. Montar a reserva de emergência (agora)')).toBeTruthy();
    expect(within(card).getByLabelText('6. Investir e proteger o que tem (depois)')).toBeTruthy();
    expect(within(card).getByRole('button', { name: 'Abrir' })).toBeTruthy();
  });

  it('fechar o mês: a pessoa marca os passos manuais e só fecha com tudo feito', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    const close = await call<MonthClose>('attention.month_close', {});
    expect(close.steps.map((s) => s.key)).toEqual([
      'statements',
      'categories',
      'balances',
      'review',
      'summary',
    ]);
    const card = await screen.findByRole('region', { name: /^Fechar / }, { timeout: 15_000 });
    expect(within(card).getByText(`${close.done} de 5`)).toBeTruthy();
    const finish = within(card).getByRole('button', { name: 'Marcar o mês como fechado' });
    expect((finish as HTMLButtonElement).disabled).toBe(true);
    const balances = within(card).getByText('Conferir os saldos').closest('li')!;
    await user.click(within(balances).getByRole('button', { name: 'Marcar como feito' }));
    await within(card).findByText(`${close.done + 1} de 5`);
    expect(within(balances).getByRole('button', { name: 'Desmarcar' })).toBeTruthy();
    const after = await call<MonthClose>('attention.month_close', {});
    expect(after.steps.find((s) => s.key === 'balances')!.done).toBe(true);
    // Mês fechado some do Painel.
    await call('attention.month_close_set', { month: close.month, step: 'closed', done: true });
    expect((await call<MonthClose>('attention.month_close', {})).closed_at).toBeTruthy();
  });

  it('resume o corpo dos alertas e acha onde resolver', () => {
    expect(attentionExcerpt('## Fechamento\n\n**Sobrou** R$ 10 | ok')).toBe(
      'Fechamento Sobrou R$ 10 ok',
    );
    const long = attentionExcerpt('palavra '.repeat(80), 60);
    expect(long.length).toBeLessThanOrEqual(61);
    expect(long.endsWith('…')).toBe(true);
    const base = {
      id: 'todo:x',
      source: 'todo',
      kind: 'x',
      severity: 'info',
      title: '',
      body: '',
      action: null,
      alert_id: null,
      count: 1,
      date: null,
    } as const;
    expect(attentionHref({ ...base, route: '/orcamento', target: null })).toBe('/orcamento');
    expect(
      attentionHref({
        ...base,
        route: null,
        target: { screen: 'import', id: null, account_id: 'c1', month: null, date: null },
      }),
    ).toBe('/lancamentos?importar=1&conta=c1');
    expect(attentionHref({ ...base, route: null, target: null })).toBeNull();
  });
});
