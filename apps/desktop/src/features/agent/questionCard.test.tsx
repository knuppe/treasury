import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { ChatPoll } from '@/api/generated/ChatPoll';
import type { JobInfo } from '@/api/generated/JobInfo';
import { mockCall, mockCallLog } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('Agent: agent question in the conversation', () => {
  async function ask(message: string) {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/agente');
    // Without history, the live reply stays in the thread (it is not replaced by the saved one).
    await mockCall('ai.settings.set', { keep_history: false });
    await user.type(await screen.findByLabelText('Mensagem para o agente'), `${message}{Enter}`);
    const card = await screen.findByRole('group', { name: 'Pergunta do agente' });
    return { user, card };
  }

  it('single choice: the recommended one comes checked and Send answers', async () => {
    const { user, card } = await ask('pergunte qual conta usar');
    const form = within(card).getByRole('group', { name: 'Qual conta devo usar para o aluguel?' });
    const corrente = within(form).getByRole<HTMLInputElement>('radio', { name: /Conta corrente/ });
    expect(corrente.checked).toBe(true);
    expect(within(form).getByText('Recomendado')).toBeTruthy();
    expect(within(form).getByText('Onde cai o salário')).toBeTruthy();
    expect(within(form).getAllByRole('radio')).toHaveLength(4);
    expect(mockCallLog()).not.toContain('ai.question.answer');
    await user.click(within(card).getByRole('button', { name: 'Enviar' }));
    expect(await within(card).findByText(/Você respondeu:/)).toBeTruthy();
    expect(within(card).getByRole('status').textContent).toBe('Você respondeu: Conta corrente');
    expect(mockCallLog().filter((c) => c === 'ai.question.answer')).toHaveLength(1);
    expect(within(card).queryByRole('button', { name: 'Enviar' })).toBeNull();
    expect(await screen.findByText('Combinado: vou usar Conta corrente.')).toBeTruthy();
  });

  it('"Outro": choosing it focuses the field, the text goes in the answer', async () => {
    const { user, card } = await ask('pergunte qual conta usar');
    const send = within(card).getByRole<HTMLButtonElement>('button', { name: 'Enviar' });
    await user.click(within(card).getByRole('radio', { name: 'Outro' }));
    const input = within(card).getByRole('textbox', { name: 'A sua resposta' });
    expect(document.activeElement).toBe(input);
    // Other without text does not count.
    expect(send.disabled).toBe(true);
    await user.type(input, 'Conta da família');
    expect(send.disabled).toBe(false);
    await user.click(send);
    expect(await within(card).findByText(/Você respondeu:/)).toBeTruthy();
    expect(within(card).getByRole('status').textContent).toBe('Você respondeu: Conta da família');
    expect(await screen.findByText('Combinado: vou usar Conta da família.')).toBeTruthy();
  });

  it('typing in the field checks "Outro"', async () => {
    const { user, card } = await ask('pergunte qual conta usar');
    await user.type(within(card).getByRole('textbox', { name: 'A sua resposta' }), 'Dinheiro');
    expect(within(card).getByRole<HTMLInputElement>('radio', { name: 'Outro' }).checked).toBe(true);
    expect(
      within(card).getByRole<HTMLInputElement>('radio', { name: /Conta corrente/ }).checked,
    ).toBe(false);
  });

  it('multiple choice without "Outro": sends both options', async () => {
    const { user, card } = await ask('pergunte várias contas');
    expect(within(card).queryByRole('checkbox', { name: 'Outro' })).toBeNull();
    expect(within(card).queryByRole('textbox')).toBeNull();
    expect(within(card).queryByText('Recomendado')).toBeNull();
    const send = within(card).getByRole<HTMLButtonElement>('button', { name: 'Enviar' });
    expect(send.disabled).toBe(true);
    await user.click(within(card).getByRole('checkbox', { name: /Conta corrente/ }));
    await user.click(within(card).getByRole('checkbox', { name: /Cartão/ }));
    await user.click(send);
    expect(await within(card).findByText(/Você respondeu:/)).toBeTruthy();
    expect(within(card).getByRole('status').textContent).toBe(
      'Você respondeu: Conta corrente, Cartão',
    );
    expect(await screen.findByText('Combinado: vou usar Conta corrente, Cartão.')).toBeTruthy();
  });

  it('the mock checks the answer like the core', async () => {
    await ask('pergunte qual conta usar');
    const jobs = (await mockCall('ai.jobs.list', {})) as JobInfo[];
    const job = jobs.find((j) => j.status === 'running')!;
    const poll = (await mockCall('ai.chat_poll', { job_id: job.job_id, after_seq: 0 })) as ChatPoll;
    const q = poll.events.find((e) => e.event.type === 'user_question')!.event;
    const request_id = q.type === 'user_question' ? q.request_id : '';
    const answer = (args: object) => mockCall('ai.question.answer', { request_id, ...args });
    await expect(answer({ selected: ['Inexistente'] })).rejects.toBeTruthy();
    await expect(answer({ selected: ['Poupança', 'Cartão'] })).rejects.toBeTruthy();
    await expect(answer({ selected: ['Poupança'], other: 'Também' })).rejects.toBeTruthy();
    await expect(answer({ selected: [] })).rejects.toBeTruthy();
    await answer({ selected: ['Poupança'] });
    // After the answer, the question no longer exists.
    await expect(answer({ selected: ['Cartão'] })).rejects.toMatchObject({ code: 'not_found' });
  });
});
