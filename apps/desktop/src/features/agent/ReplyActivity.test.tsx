import { render, screen, within } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';

import { useAgentSession } from '@/state/agentSession';

import { ReplyActivity, formatSeconds, type ActivityTool } from './ReplyActivity';

const tool = (name: string, result: ActivityTool['result'], args = '{}'): ActivityTool => ({
  name,
  args,
  result,
});

function renderActivity(props: Partial<Parameters<typeof ReplyActivity>[0]> = {}) {
  return render(
    <>
      <ReplyActivity
        id="m1"
        reasoning={[]}
        reasoningMs={null}
        tools={[]}
        running={false}
        answering
        {...props}
      />
    </>,
    { wrapper: MemoryRouter },
  );
}

describe('atividade do agente recolhida', () => {
  beforeEach(() => useAgentSession.getState().reset());

  it('ao vivo: "Pensando…" com a última linha do raciocínio e a ferramenta em andamento', () => {
    const { rerender } = renderActivity({
      reasoning: ['Primeiro as contas.\nDepois o saldo.'],
      running: true,
      answering: false,
    });
    expect(screen.getByRole('button', { name: /Pensando…/ })).toBeTruthy();
    expect(screen.getByText('Depois o saldo.')).toBeTruthy();
    rerender(
      <>
        <ReplyActivity
          id="m1"
          reasoning={['Primeiro as contas.']}
          reasoningMs={null}
          tools={[tool('list_accounts', null)]}
          running
          answering={false}
        />
      </>,
    );
    expect(screen.getByRole('button', { name: /Consultando list_accounts…/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Raciocínio' })).toBeTruthy();
  });

  it('pronto: resumo com duração, contagem e erros; abre e o estado fica na mensagem', async () => {
    const user = userEvent.setup();
    const { unmount } = renderActivity({
      reasoning: ['Vou ver o saldo.'],
      reasoningMs: 12_400,
      tools: [
        tool('get_account_balance', { ok: true, summary: 'Saldo de R$ 10,00.' }, '{"id":"a"}'),
        tool('cli.Read', { ok: false, summary: 'Permissão negada' }),
      ],
    });
    const reasoning = screen.getByRole('button', { name: 'Raciocínio · 12 s' });
    const tools = screen.getByRole('button', { name: 'Consultou 2 ferramentas · 1 com erro' });
    expect(screen.queryByText('Vou ver o saldo.')).toBeNull();
    expect(screen.queryByText('get_account_balance')).toBeNull();
    await user.click(tools);
    const list = screen.getByRole('list', { name: 'Ferramentas usadas' });
    expect(within(list).getByText('get_account_balance')).toBeTruthy();
    expect(within(list).getByText('{"id":"a"}')).toBeTruthy();
    expect(within(list).getByText('Saldo de R$ 10,00.')).toBeTruthy();
    // Ferramenta nativa da CLI aparece marcada, com o erro.
    const read = within(list).getByRole('listitem', { name: 'Ferramenta Read' });
    expect(within(read).getByText('CLI')).toBeTruthy();
    expect(within(read).getByText('Permissão negada')).toBeTruthy();
    await user.click(reasoning);
    expect(screen.getByRole('region', { name: 'Raciocínio do agente' }).textContent).toBe(
      'Vou ver o saldo.',
    );
    // Reabrir a mensagem (ex.: depois de gravada) mantém o que estava aberto.
    unmount();
    renderActivity({
      reasoning: ['Vou ver o saldo.'],
      reasoningMs: 12_400,
      tools: [tool('get_account_balance', { ok: true, summary: '' })],
    });
    expect(
      screen.getByRole('button', { name: 'Raciocínio · 12 s' }).getAttribute('aria-expanded'),
    ).toBe('true');
  });

  it('sem raciocínio nem ferramentas não desenha nada; o alias leva o estado à mensagem', () => {
    const { container } = renderActivity();
    expect(container.textContent).toBe('');
    useAgentSession.getState().setExpanded('turn-1:tools', true);
    useAgentSession.getState().alias('turn-1', 'msg-9');
    expect(useAgentSession.getState().expanded['msg-9:tools']).toBe(true);
    expect(formatSeconds(400)).toBe('menos de 1 s');
    expect(formatSeconds(75_000)).toBe('1 min 15 s');
  });

  it('liberação do computador de uma conversa nova vai para o id dela', () => {
    const st = useAgentSession.getState();
    st.setComputer(null, true);
    useAgentSession.getState().carryComputer('conv-1');
    expect(useAgentSession.getState().computer).toEqual({ 'conv-1': true });
    useAgentSession.getState().carryComputer('conv-2');
    expect(useAgentSession.getState().computer['conv-2']).toBeUndefined();
  });
});
