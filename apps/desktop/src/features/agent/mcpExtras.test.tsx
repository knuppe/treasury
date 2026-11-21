import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog } from '@/api/mock';
import type { McpServer } from '@/api/generated/McpServer';
import type { McpServerInput } from '@/api/generated/McpServerInput';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

// Valores inventados: nada aqui é um token de verdade.
const TOKEN = 'tok-exemplo-1234';

async function servers(): Promise<McpServer[]> {
  return (await mockCall('ai.mcp_extra.list', {})) as McpServer[];
}

describe('Agente: MCPs extras do cofre', () => {
  it('cadastra um comando local com variável secreta, mascara, testa, liga/desliga e exclui', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/agente?aba=configurar&ia=acesso');
    await user.click(await screen.findByRole('button', { name: 'Adicionar MCP' }));
    const dialog = await screen.findByRole('dialog', { name: 'Novo MCP extra' });
    expect(within(dialog).getByText(/roda neste computador com as permissões/)).toBeTruthy();

    await user.type(within(dialog).getByLabelText('Rótulo'), 'Meu celular');
    // O nome segue a regra do núcleo: "treasury" é reservado.
    await user.type(within(dialog).getByLabelText('Nome'), 'treasury');
    expect(within(dialog).getByText(/não "treasury"/)).toBeTruthy();
    await user.clear(within(dialog).getByLabelText('Nome'));
    await user.type(within(dialog).getByLabelText('Nome'), 'celular');
    expect(within(dialog).queryByText(/não "treasury"/)).toBeNull();
    await user.type(within(dialog).getByLabelText('Comando'), 'npx');
    await user.type(within(dialog).getByLabelText('Argumentos'), '-y{Enter}mcp-celular-exemplo');
    await user.click(within(dialog).getByRole('button', { name: 'Adicionar variável' }));
    const row = within(dialog).getByRole('group', { name: 'Variável 1' });
    await user.type(within(row).getByLabelText('Nome'), 'API_TOKEN');
    await user.click(within(row).getByRole('checkbox', { name: 'Secreta' }));
    await user.type(within(row).getByLabelText('Valor'), TOKEN);
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Novo MCP extra' })).toBeNull(),
    );

    const item = await screen.findByRole('listitem', { name: 'MCP Meu celular' });
    expect(within(item).getByText('celular-*')).toBeTruthy();
    expect(within(item).getByText('Comando local')).toBeTruthy();
    expect(within(item).getByText('npx -y mcp-celular-exemplo')).toBeTruthy();
    expect(within(item).getByText('API_TOKEN=••••…1234')).toBeTruthy();
    expect(document.body.textContent).not.toContain(TOKEN);
    const saved = (await servers())[0]!;
    expect(saved.env).toEqual([
      { key: 'API_TOKEN', value: null, secret: true, set: true, hint: '…1234' },
    ]);
    expect(JSON.stringify(saved)).not.toContain(TOKEN);

    // Testar: conecta e lista as ferramentas.
    await user.click(within(item).getByRole('button', { name: 'Testar' }));
    const tools = await within(item).findByRole('list', { name: 'Ferramentas do MCP' });
    expect(within(tools).getByText('abrir_app')).toBeTruthy();
    expect(within(item).getByText(/2 ferramentas encontradas/)).toBeTruthy();

    // Interruptores: salvam sem tocar no segredo.
    const auto = within(item).getByRole<HTMLInputElement>('checkbox', {
      name: 'Permitir sem perguntar',
    });
    expect(auto.checked).toBe(false);
    await user.click(auto);
    await waitFor(async () => expect((await servers())[0]!.auto_approve).toBe(true));
    await user.click(within(item).getByRole('checkbox', { name: 'Ligado' }));
    await waitFor(async () => expect((await servers())[0]!.enabled).toBe(false));
    expect((await servers())[0]!.env[0]!.hint).toBe('…1234');

    // Editar: o segredo aparece mascarado e em branco fica o que está gravado.
    await user.click(within(item).getByRole('button', { name: 'Editar' }));
    const edit = await screen.findByRole('dialog', { name: 'Editar Meu celular' });
    expect(within(edit).getByText('••••…1234 · deixe em branco para manter')).toBeTruthy();
    expect(within(edit).getByLabelText<HTMLInputElement>('Valor').value).toBe('');
    await user.click(within(edit).getByRole('button', { name: 'Salvar' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Editar Meu celular' })).toBeNull(),
    );
    expect((await servers())[0]!.env[0]).toMatchObject({ set: true, hint: '…1234' });

    // Excluir com confirmação.
    await user.click(within(item).getByRole('button', { name: 'Excluir' }));
    const confirm = await screen.findByRole('dialog', { name: 'Excluir Meu celular' });
    await user.click(within(confirm).getByRole('button', { name: 'Excluir' }));
    await waitFor(() =>
      expect(screen.queryByRole('listitem', { name: 'MCP Meu celular' })).toBeNull(),
    );
    expect(await servers()).toEqual([]);
  });
});

describe('Agente: confirmação de ferramenta externa na conversa', () => {
  const server: McpServerInput = {
    name: 'celular',
    label: 'Meu celular',
    transport: 'stdio',
    command: 'npx',
    args: ['-y', 'mcp-celular-exemplo'],
    env: [{ key: 'API_TOKEN', value: TOKEN, secret: true }],
    headers: [],
    enabled: true,
    auto_approve: false,
  };

  async function ask() {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/agente');
    // Sem histórico, a resposta ao vivo fica no fio (não é trocada pela gravada).
    await mockCall('ai.settings.set', { keep_history: false });
    await mockCall('ai.mcp_extra.save', server);
    await user.type(
      await screen.findByLabelText('Mensagem para o agente'),
      'abra a agenda no celular{Enter}',
    );
    const card = await screen.findByRole('group', { name: 'Confirmação de ferramenta externa' });
    expect(within(card).getByText('O agente quer usar abrir_app do MCP Meu celular')).toBeTruthy();
    expect(within(card).getByText('{"app": "Agenda"}')).toBeTruthy();
    return { user, card };
  }

  it('Permitir responde ao pedido e o cartão mostra "Permitida"', async () => {
    const { user, card } = await ask();
    expect(mockCallLog()).not.toContain('ai.mcp_extra.decide');
    await user.click(within(card).getByRole('button', { name: 'Permitir' }));
    expect(await within(card).findByText('Permitida')).toBeTruthy();
    expect(mockCallLog()).toContain('ai.mcp_extra.decide');
    expect(within(card).queryByRole('button', { name: 'Permitir' })).toBeNull();
    expect(await screen.findByText(/Abri a agenda no aparelho/)).toBeTruthy();
    // O pedido já respondido não aceita outra resposta.
    const req = mockCallLog().filter((c) => c === 'ai.mcp_extra.decide').length;
    expect(req).toBe(1);
  });

  it('Permitir nesta conversa libera o MCP no resto da conversa', async () => {
    const { user, card } = await ask();
    await user.click(within(card).getByRole('button', { name: 'Permitir nesta conversa' }));
    expect(await within(card).findByText('Permitida nesta conversa (Meu celular)')).toBeTruthy();
    expect(mockCallLog()).toContain('ai.mcp_extra.decide');
    expect(await screen.findByText(/Abri a agenda no aparelho/)).toBeTruthy();
  });

  it('Recusar responde ao pedido e o cartão mostra "Recusada"', async () => {
    const { user, card } = await ask();
    await user.click(within(card).getByRole('button', { name: 'Recusar' }));
    expect(await within(card).findByText('Recusada')).toBeTruthy();
    expect(mockCallLog()).toContain('ai.mcp_extra.decide');
    expect(await screen.findByText(/não usei o aparelho/)).toBeTruthy();
  });
});

describe('Agente: MCPs das suas CLIs', () => {
  it('nenhum entra por padrão; a pessoa libera um a um', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/agente?aba=configurar&ia=acesso');
    const list = await screen.findByRole('list', { name: 'MCPs das CLIs' });
    expect(within(list).getByText('github')).toBeTruthy();
    const toggle = within(list).getByRole('checkbox', { name: /Usar github nas conversas/ });
    expect((toggle as HTMLInputElement).checked).toBe(false);
    expect(screen.getByText(/sem a confirmação nem a auditoria do Treasury/)).toBeTruthy();
    await user.click(toggle);
    await waitFor(async () => {
      const now = (await mockCall('ai.cli_mcps.list', {})) as { name: string; allowed: boolean }[];
      expect(now.find((m) => m.name === 'github')?.allowed).toBe(true);
      expect(now.find((m) => m.name === 'navegador')?.allowed).toBe(false);
    });
    expect(mockCallLog()).toContain('ai.cli_mcps.set');
  });
});

describe('Agente: tempo-limite das respostas por CLI', () => {
  it('troca de 15 para 60 minutos em Acesso do agente', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/agente?aba=configurar&ia=acesso');
    const group = await screen.findByRole('radiogroup', {
      name: 'Tempo-limite das respostas por CLI',
    });
    expect(within(group).getByRole('radio', { name: '15 min' }).getAttribute('aria-checked')).toBe(
      'true',
    );
    await user.click(within(group).getByRole('radio', { name: '60 min' }));
    await waitFor(async () => {
      const s = (await mockCall('ai.settings.get', {})) as { cli_timeout_minutes: number };
      expect(s.cli_timeout_minutes).toBe(60);
    });
  });
});
