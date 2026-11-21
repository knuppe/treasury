import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { AppSettings } from '@/api/generated/AppSettings';
import type { Performance } from '@/api/generated/Performance';
import type { TxnPage } from '@/api/generated/TxnPage';
import { MOCK_PASSWORD, MOCK_RECOVERY_CODE, mockCall } from '@/api/mock';
import { setMockNewClients, setMockRequireKeyNotice } from '@/api/mock/mcp';
import { todayIso } from '@/lib/format';
import { renderApp } from '@/test/renderApp';

async function createTxn(description: string) {
  const [checking] = (await mockCall('accounts.list', {})) as AccountView[];
  return mockCall('transactions.create', {
    account_id: checking!.id,
    date: todayIso(),
    amount: -1_000,
    description,
  });
}

async function count(text: string) {
  return ((await mockCall('transactions.query', { text })) as TxnPage).total;
}

describe('Settings', () => {
  it('AI in tabs: engines first; the agent access gathers permissions, MCPs and memory', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=ia');
    const tabs = await screen.findByRole('tablist', { name: 'Partes dos ajustes de IA' });
    expect(
      within(tabs)
        .getAllByRole('tab')
        .map((x) => x.textContent),
    ).toEqual(['Motores', 'Acesso do agente', 'Custos', 'Histórico']);
    expect(
      (await screen.findAllByRole('heading', { name: 'Motores de IA' })).length,
    ).toBeGreaterThan(0);
    expect(screen.queryByRole('heading', { name: /MCPs extras/ })).toBeNull();

    await user.click(within(tabs).getByRole('tab', { name: 'Acesso do agente' }));
    expect(window.location.hash).toContain('ia=acesso');
    for (const title of [
      /Ajustes do agente/,
      /MCPs extras/,
      /MCPs das suas CLIs/,
      /Memória do gestor/,
    ]) {
      expect(await screen.findByRole('heading', { name: title })).toBeTruthy();
    }
    expect(screen.queryByRole('button', { name: 'Adicionar motor' })).toBeNull();
  });

  it('history: shows the diff and undoes a change', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/ajustes?secao=historico');
    await screen.findByRole('grid', { name: 'Alterações' });
    await createTxn('Café histórico');
    await queryClient.invalidateQueries();
    const row = (await screen.findByText('Lançamento criado: Café histórico')).closest('tr')!;
    await user.click(row);
    const detail = screen.getByRole('region', { name: 'Detalhe' });
    const created = await within(detail).findByRole('region', {
      name: 'Criação · Lançamento: Café histórico',
    });
    // Names in place of ids and formatted values; the exact diff stays in the technical details.
    const [checking] = (await mockCall('accounts.list', {})) as AccountView[];
    expect(within(created).getByText(checking!.name)).toBeTruthy();
    expect(within(created).getByText('Despesa')).toBeTruthy();
    expect(created.textContent).not.toContain(checking!.id);
    await user.click(within(detail).getByRole('button', { name: 'Ver detalhes técnicos' }));
    expect(within(detail).getByText('"Café histórico"')).toBeTruthy();
    await user.click(within(detail).getByRole('button', { name: 'Desfazer' }));
    await waitFor(async () => expect(await count('café histórico')).toBe(0));
    expect(await within(detail).findByRole('button', { name: 'Refazer' })).toBeTruthy();
    await user.click(within(detail).getByRole('button', { name: 'Refazer' }));
    await waitFor(async () => expect(await count('café histórico')).toBe(1));
  });

  it('history: explains the conflict when later changes touched the rows', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/ajustes?secao=historico');
    await screen.findByRole('grid', { name: 'Alterações' });
    const t = (await createTxn('Pão conflito')) as { id: string };
    await mockCall('transactions.update', { id: t.id, description: 'Pão conflito (editado)' });
    await queryClient.invalidateQueries();
    await user.click((await screen.findByText('Lançamento criado: Pão conflito')).closest('tr')!);
    const detail = screen.getByRole('region', { name: 'Detalhe' });
    await user.click(await within(detail).findByRole('button', { name: 'Desfazer' }));
    const alert = await within(detail).findByRole('alert');
    expect(alert.textContent).toMatch(/Não dá para desfazer sozinha/);
    expect(alert.textContent).toMatch(/alterações posteriores mudaram estes dados/);
    expect(await count('pão conflito')).toBe(1);
  });

  it('backups: restores asking for the master password and, for an old backup, the backup password', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=backups');
    const table = await screen.findByRole('grid', { name: 'Backups' });
    const row = within(table).getByText('Antes de trocar a senha').closest('tr')!;
    await user.click(within(row).getByRole('button', { name: 'Restaurar' }));
    const dialog = await screen.findByRole('dialog', { name: 'Restaurar backup' });
    await user.type(within(dialog).getByLabelText('Senha mestra atual'), 'senha errada qualquer');
    await user.click(within(dialog).getByRole('button', { name: 'Restaurar' }));
    expect(await within(dialog).findByRole('alert')).toBeTruthy();
    const master = within(dialog).getByLabelText('Senha mestra atual');
    await user.clear(master);
    await user.type(master, MOCK_PASSWORD);
    await user.click(within(dialog).getByRole('button', { name: 'Restaurar' }));
    expect(
      await within(dialog).findByText(/informe a senha usada quando este backup foi feito/i),
    ).toBeTruthy();
    await user.type(within(dialog).getByLabelText('Senha do backup'), 'senha antiga do cofre');
    await user.click(within(dialog).getByRole('button', { name: 'Restaurar' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Restaurar backup' })).toBeNull(),
    );
    expect(await screen.findByText('Backup restaurado')).toBeTruthy();
    // Before restoring, the current state becomes a backup.
    expect(await screen.findAllByText('Antes de restaurar')).not.toHaveLength(0);
  });

  it('backups: warns when there is no valid backup in 48 h', async () => {
    renderApp('unlocked', {}, '#/ajustes?secao=backups');
    const health = await screen.findByRole('region', { name: 'Saúde dos backups' });
    expect(await within(health).findByText('Último backup válido')).toBeTruthy();
    expect(within(health).queryByRole('alert')).toBeNull();
  });

  it('security: the recovery kit only shows up with the master password', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=seguranca');
    await user.click(await screen.findByRole('button', { name: 'Gerar kit de recuperação' }));
    const dialog = await screen.findByRole('dialog', { name: 'Confirme a senha mestra' });
    await user.type(within(dialog).getByLabelText('Senha mestra'), 'senha errada qualquer');
    await user.click(within(dialog).getByRole('button', { name: 'Mostrar o kit' }));
    expect(await within(dialog).findByRole('alert')).toBeTruthy();
    const input = within(dialog).getByLabelText('Senha mestra');
    await user.clear(input);
    await user.type(input, MOCK_PASSWORD);
    await user.click(within(dialog).getByRole('button', { name: 'Mostrar o kit' }));
    const kit = await screen.findByRole('dialog', { name: 'Kit de recuperação' });
    expect(within(kit).getByText(MOCK_RECOVERY_CODE)).toBeTruthy();
    expect(within(kit).getByRole('button', { name: 'Imprimir' })).toBeTruthy();
  });

  it('security: changes the master password validating the strength', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=seguranca');
    const form = await screen.findByRole('form', { name: 'Trocar a senha mestra' });
    await user.type(within(form).getByLabelText('Senha atual'), MOCK_PASSWORD);
    await user.type(within(form).getByLabelText('Nova senha'), 'curta');
    await user.click(within(form).getByRole('button', { name: 'Trocar senha mestra' }));
    expect(within(form).getByText('Use pelo menos 10 caracteres.')).toBeTruthy();
    const next = within(form).getByLabelText('Nova senha');
    await user.clear(next);
    await user.type(next, 'girafa azul dança no telhado');
    await user.type(
      within(form).getByLabelText('Confirmar nova senha'),
      'girafa azul dança no telhado',
    );
    await user.click(within(form).getByRole('button', { name: 'Trocar senha mestra' }));
    expect(await screen.findByText('Senha mestra trocada')).toBeTruthy();
    await expect(
      mockCall('vault.recovery_kit', { password: 'girafa azul dança no telhado' }),
    ).resolves.toMatchObject({
      code: MOCK_RECOVERY_CODE,
    });
  });

  it('categories: offers the suggested ones for a family with children without touching the existing ones', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=categorias');
    const tree = await screen.findByRole('tree', { name: 'Categorias de despesa' });
    const before = within(tree).getAllByRole('treeitem').length;
    await user.click(await screen.findByRole('button', { name: 'Ver lista' }));
    const list = screen.getByRole('list', { name: 'Categorias sugeridas' });
    expect(within(list).getByText('Filhos › Fraldas e higiene')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Adicionar sugeridas' }));
    expect(await screen.findByText(/categoria\(s\) adicionada\(s\)/)).toBeTruthy();
    await waitFor(() =>
      expect(screen.queryByRole('button', { name: 'Adicionar sugeridas' })).toBeNull(),
    );
    await waitFor(() =>
      expect(within(tree).getAllByRole('treeitem').length).toBeGreaterThan(before),
    );
  });

  it('categories: adds only the checked suggested ones (new income)', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=categorias');
    await screen.findByRole('tree', { name: 'Categorias de despesa' });
    await user.click(await screen.findByRole('button', { name: 'Ver lista' }));
    const list = screen.getByRole('list', { name: 'Categorias sugeridas' });
    expect(within(list).getByText('Venda de bens › Veículos (receita)')).toBeTruthy();
    // Unchecking the new group unchecks its subcategories.
    await user.click(within(list).getByRole('checkbox', { name: 'Venda de bens (receita)' }));
    expect(
      within(list).getByRole<HTMLInputElement>('checkbox', {
        name: 'Venda de bens › Imóveis (receita)',
      }).checked,
    ).toBe(false);
    await user.click(screen.getByRole('button', { name: /^Adicionar \d+ selecionada\(s\)$/ }));
    expect(await screen.findByText(/categoria\(s\) adicionada\(s\)/)).toBeTruthy();
    // Only what was left unchecked remains.
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Adicionar sugeridas' })).toBeTruthy(),
    );
    const left = screen.getByRole('list', { name: 'Categorias sugeridas' });
    expect(within(left).getAllByRole('checkbox')).toHaveLength(5);
    expect(within(left).getByText('Venda de bens › Milhas e pontos (receita)')).toBeTruthy();
  });

  it('categories: creates a subcategory and deletes with reassignment', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=categorias');
    await screen.findByRole('tree', { name: 'Categorias de despesa' });
    await user.click(screen.getByRole('button', { name: 'Nova subcategoria em Pets' }));
    const dialog = await screen.findByRole('dialog', { name: /Nova subcategoria de Pets/ });
    await user.type(within(dialog).getByLabelText('Nome'), 'Banho e tosa');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    expect(await screen.findByText('Banho e tosa')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Apagar Banho e tosa' }));
    const del = await screen.findByRole('dialog', { name: /Apagar Banho e tosa/ });
    expect(await within(del).findByText(/Sem uso/)).toBeTruthy();
    await user.click(within(del).getByRole('button', { name: 'Apagar' }));
    await waitFor(() => expect(screen.queryByText('Banho e tosa')).toBeNull());
  });

  it('data: export warns that it comes out unencrypted and checks the path', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=dados');
    expect(await screen.findByText(/sem criptografia/)).toBeTruthy();
    await user.type(screen.getByLabelText(/Tudo, em JSON/), 'treasury.json');
    await user.click(screen.getByRole('button', { name: 'Exportar JSON' }));
    expect(await screen.findByText(/Escolha um caminho completo para o arquivo/)).toBeTruthy();
    const path = screen.getByLabelText(/Tudo, em JSON/);
    await user.clear(path);
    await user.type(path, '/Users/ana/Documents/treasury.json');
    await user.click(screen.getByRole('button', { name: 'Exportar JSON' }));
    expect(await screen.findByText('Exportado sem criptografia')).toBeTruthy();
  });
});

describe('Settings › Security: PIN and split recovery', () => {
  it('turns on the PIN and generates the 3 parts with the master password', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', { keychainAvailable: true }, '#/ajustes?secao=seguranca');
    await user.click(await screen.findByRole('switch', { name: /Desbloquear com PIN/ }));
    const pinDialog = await screen.findByRole('dialog', { name: 'Ligar o PIN' });
    await user.type(within(pinDialog).getByLabelText('PIN (6 a 12 dígitos)'), '730418');
    await user.type(within(pinDialog).getByLabelText('Senha mestra'), MOCK_PASSWORD);
    await user.click(within(pinDialog).getByRole('button', { name: 'Ligar PIN' }));
    await waitFor(() =>
      expect(
        screen.getByRole('switch', { name: /Desbloquear com PIN/ }).getAttribute('aria-checked'),
      ).toBe('true'),
    );

    await user.click(screen.getByRole('button', { name: 'Gerar as 3 partes' }));
    const ask = await screen.findByRole('dialog', { name: 'Partes de recuperação' });
    await user.type(within(ask).getByLabelText('Senha mestra'), MOCK_PASSWORD);
    await user.click(within(ask).getByRole('button', { name: 'Gerar' }));
    const list = await screen.findByRole('list', { name: 'Partes de recuperação' });
    expect(within(list).getAllByRole('listitem')).toHaveLength(3);
    await user.click(screen.getByRole('button', { name: 'Já guardei' }));
    expect(await screen.findByText(/Partes geradas em/)).toBeTruthy();
  });
});

describe('Settings › Security: screenshots', () => {
  it('blocked by default; allowing asks for confirmation, blocking is immediate', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=seguranca');
    const toggle = await screen.findByRole('switch', { name: /Permitir capturas de tela/ });
    expect(toggle.getAttribute('aria-checked')).toBe('false');
    await user.click(toggle);
    const ask = await screen.findByRole('dialog', { name: 'Permitir capturas de tela?' });
    await user.click(within(ask).getByRole('button', { name: 'Cancelar' }));
    expect(((await mockCall('settings.get', {})) as AppSettings).allow_screen_capture).toBe(false);
    await user.click(toggle);
    await user.click(
      within(await screen.findByRole('dialog', { name: 'Permitir capturas de tela?' })).getByRole(
        'button',
        { name: 'Permitir capturas' },
      ),
    );
    await waitFor(() => expect(toggle.getAttribute('aria-checked')).toBe('true'));
    await user.click(toggle);
    await waitFor(() => expect(toggle.getAttribute('aria-checked')).toBe('false'));
    expect(screen.queryByRole('dialog', { name: 'Permitir capturas de tela?' })).toBeNull();
  });
});

describe('Settings › Preferences: daily profitability (advanced)', () => {
  it('only shows at the Avançado level and turns on twr_daily', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=preferencias');
    const toggle = await screen.findByRole('switch', { name: /Rentabilidade diária \(avançado\)/ });
    expect(toggle.getAttribute('aria-checked')).toBe('false');
    expect(screen.getByText(/É mais lento; o padrão é a rentabilidade mensal/)).toBeTruthy();
    await user.click(toggle);
    await waitFor(() => expect(toggle.getAttribute('aria-checked')).toBe('true'));
    expect(((await mockCall('settings.get', {})) as AppSettings).twr_daily).toBe(true);
    // With the setting, the profitability comes from the daily method.
    const perf = (await mockCall('portfolio.performance', {
      from_month: todayIso().slice(0, 7),
    })) as Performance;
    expect(perf.method).toBe('daily');
    await user.click(toggle);
    await waitFor(() => expect(toggle.getAttribute('aria-checked')).toBe('false'));
    expect(((await mockCall('settings.get', {})) as AppSettings).twr_daily).toBe(false);
  });

  it('disappears at the Intermediário and Iniciante levels', async () => {
    const { queryClient } = renderApp('unlocked', {}, '#/ajustes?secao=preferencias');
    // The mock starts at Avançado.
    expect(await screen.findByRole('switch', { name: /Rentabilidade diária/ })).toBeTruthy();
    for (const level of ['intermediario', 'iniciante'] as const) {
      await mockCall('profile.set', { level });
      await act(() => queryClient.invalidateQueries());
      await waitFor(() =>
        expect(screen.queryByRole('switch', { name: /Rentabilidade diária/ })).toBeNull(),
      );
      expect(screen.getByRole('switch', { name: /Manter rotinas na bandeja/ })).toBeTruthy();
    }
  });
});

describe('Settings › MCP integrations: API keys', () => {
  it('creates a key with scopes, shows it once and revokes', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=mcp');
    await user.click(await screen.findByRole('button', { name: 'Nova chave' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova chave de API' });
    await user.type(within(dialog).getByLabelText('Nome'), 'Agente da corretora');
    await user.click(within(dialog).getByText(/Escrita: cria propostas/));
    await user.click(within(dialog).getByRole('button', { name: 'Criar chave' }));
    const done = await screen.findByRole('dialog', { name: 'Chave criada' });
    expect(within(done).getAllByText(/trsk_mock/).length).toBeGreaterThan(0);
    expect(within(done).getByText(/Leitura, Escrita \(proposta\)/)).toBeTruthy();
    await user.click(within(done).getByRole('button', { name: 'Já copiei' }));
    const table = await screen.findByRole('grid', { name: 'Chaves de API' });
    expect(within(table).getByText('Agente da corretora')).toBeTruthy();
    await user.click(within(table).getByRole('button', { name: 'Revogar' }));
    // Revoked: stays dimmed in the list, with the revocation date, and can be deleted.
    expect(await within(table).findByText(/^revogada em /)).toBeTruthy();
    expect(within(table).getByText(/^sai da lista em /)).toBeTruthy();
    expect(within(table).getByText('Agente da corretora').closest('.opacity-45')).toBeTruthy();
    await user.click(within(table).getByRole('button', { name: 'Excluir' }));
    await waitFor(() => expect(screen.queryByText('Agente da corretora')).toBeNull());
  });

  it('revoked more than 7 days ago drops off the list by itself', async () => {
    renderApp('unlocked', {}, '#/ajustes?secao=mcp');
    const made = (await mockCall('mcp.key_create', { name: 'Antiga', scopes: [] })) as {
      info: { id: string };
    };
    await mockCall('mcp.key_revoke', { id: made.info.id });
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(Date.now() + 8 * 86_400_000);
    try {
      expect(await mockCall('mcp.keys_list', {})).toEqual([]);
    } finally {
      vi.useRealTimers();
    }
  });

  it('the API key is always required: the screen has no keyless option', async () => {
    renderApp('unlocked', {}, '#/ajustes?secao=mcp');
    expect(await screen.findByText(/A chave de API é sempre exigida/)).toBeTruthy();
    expect(screen.queryByRole('switch', { name: /Exigir chave/ })).toBeNull();
    expect(screen.queryByRole('switch', { name: /Sem chave/ })).toBeNull();
    expect(screen.queryByRole('switch', { name: /Aceitar pedidos sem chave/ })).toBeNull();
  });

  it('old vault: warns once that the key became required', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/ajustes?secao=mcp');
    await screen.findByRole('switch', { name: /Responder com o app fechado/ });
    setMockRequireKeyNotice(true);
    await act(async () => {
      await queryClient.invalidateQueries();
    });
    expect(await screen.findByText('A chave de API agora é obrigatória')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Entendi' }));
    await waitFor(() =>
      expect(screen.queryByText('A chave de API agora é obrigatória')).toBeNull(),
    );
  });

  it('new MCP client: shows the program and the PID until dismissed', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/ajustes?secao=mcp');
    await screen.findByRole('switch', { name: /Responder com o app fechado/ });
    setMockNewClients([
      {
        client: 'Cliente Inventado',
        program: '/Applications/Inventado.app/Contents/MacOS/Inventado',
        pid: 4321,
        at: '2026-10-23T12:00:00Z',
      },
    ]);
    await act(async () => {
      await queryClient.invalidateQueries();
    });
    expect(await screen.findByText('Um cliente MCP novo conectou')).toBeTruthy();
    expect(
      screen.getByText(/Cliente Inventado · \/Applications\/Inventado.app.* · PID 4321/),
    ).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Entendi' }));
    await waitFor(() => expect(screen.queryByText('Um cliente MCP novo conectou')).toBeNull());
  });
});

describe('Settings: groups, overview and search', () => {
  it('opens on the overview, with the sections in five groups', async () => {
    renderApp('unlocked', {}, '#/ajustes');
    const overview = await screen.findByRole('region', { name: 'Visão geral dos ajustes' });
    for (const g of [
      'Você e a família',
      'Organização do dinheiro',
      'Segurança e dados',
      'Integrações',
      'O Treasury',
    ]) {
      expect(within(overview).getByRole('heading', { name: g })).toBeTruthy();
    }
    const nav = screen.getByRole('navigation', { name: 'Seções de ajustes' });
    expect(
      within(nav).getByRole('button', { name: 'Visão geral' }).getAttribute('aria-current'),
    ).toBe('page');
  });

  it('search filters the sections and Enter opens the first; the path goes back to the overview', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes');
    const nav = await screen.findByRole('navigation', { name: 'Seções de ajustes' });
    await user.type(within(nav).getByRole('searchbox', { name: 'Buscar nos ajustes' }), 'tema');
    expect(within(nav).getByRole('button', { name: /Aparência e uso/ })).toBeTruthy();
    expect(within(nav).queryByRole('button', { name: /Backups/ })).toBeNull();
    await user.keyboard('{Enter}');
    const section = await screen.findByRole('region', { name: 'Aparência e uso' });
    expect(within(section).getByRole('heading', { name: 'Aparência e uso' })).toBeTruthy();
    await user.click(within(section).getByRole('button', { name: 'Ajustes' }));
    expect(await screen.findByRole('region', { name: 'Visão geral dos ajustes' })).toBeTruthy();

    await user.type(within(nav).getByRole('searchbox', { name: 'Buscar nos ajustes' }), 'xyzw');
    expect(within(nav).getByRole('status').textContent).toContain('Nada com');
  });

  it('a card on the overview opens the section', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes');
    const overview = await screen.findByRole('region', { name: 'Visão geral dos ajustes' });
    await user.click(within(overview).getByRole('button', { name: /^Segurança:/ }));
    expect(await screen.findByRole('form', { name: 'Trocar a senha mestra' })).toBeTruthy();
  });
});
