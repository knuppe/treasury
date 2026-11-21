import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { TxnPage } from '@/api/generated/TxnPage';
import type { TxnView } from '@/api/generated/TxnView';
import { mockCall } from '@/api/mock';
import type { BulkResult } from '@/api/generated/BulkResult';
import { setLastPointerForTests } from '@/design/pointer';
import { todayIso } from '@/lib/format';
import { renderApp } from '@/test/renderApp';

async function query(args: Record<string, unknown>) {
  return (await mockCall('transactions.query', args)) as TxnPage;
}

function table() {
  return screen.getByRole('grid', { name: 'Lançamentos' });
}

async function openTable(hash = '#/lancamentos') {
  const result = renderApp('unlocked', {}, hash);
  await screen.findByRole('grid', { name: 'Lançamentos' });
  return result;
}

describe('04 · Transactions', () => {
  it('opens on the current month with totals per currency and paginates', async () => {
    await openTable();
    expect(screen.getByRole('button', { name: /Período: Este mês/ })).toBeTruthy();
    const totals = screen.getByLabelText('Totais do filtro');
    expect(within(totals).getByText('BRL')).toBeTruthy();
    expect(within(totals).getByText('USD')).toBeTruthy();
    expect(within(totals).getAllByText('Entradas')).toHaveLength(2);
    expect(within(totals).getAllByText('Resultado')).toHaveLength(2);
  });

  it('sorts by description (in the core, applying to all pages) and goes back to date', async () => {
    const user = userEvent.setup();
    await openTable('#/lancamentos?periodo=tudo');
    const first = () => within(table()).getAllByRole('row')[1]!.textContent ?? '';
    await user.click(within(table()).getByRole('button', { name: /Descrição/ }));
    await waitFor(() => expect(window.location.hash).toContain('description_asc'));
    const asc = first();
    await user.click(within(table()).getByRole('button', { name: /Descrição/ }));
    await waitFor(() => expect(window.location.hash).toContain('description_desc'));
    await waitFor(() => expect(first()).not.toBe(asc));
    await user.click(within(table()).getByRole('button', { name: /Data/ }));
    await waitFor(() => expect(window.location.hash).toContain('date_asc'));
  }, 60_000); // "tudo" loads all the mock transactions: under build load it exceeds 20 s

  it('autocompletes the description from history and fills in category and amount', async () => {
    const user = userEvent.setup();
    await openTable();
    const accounts = (await mockCall('accounts.list', {})) as AccountView[];
    const groceries = (await mockCall('categories.list', {})) as { id: string; name: string }[];
    const cat = groceries.find((c) => c.name === 'Restaurantes') ?? groceries[0]!;
    await mockCall('transactions.create', {
      account_id: accounts[0]!.id,
      date: todayIso(),
      amount: -3_450,
      description: 'Café Fazenda Lagoa',
      category_id: cat.id,
    });
    await user.keyboard('n');
    const dialog = await screen.findByRole('dialog', { name: 'Novo lançamento' });
    await user.type(within(dialog).getByLabelText('Descrição'), 'cafe faz');
    const list = await within(dialog).findByRole('listbox', { name: 'Sugestões de lançamentos' });
    expect(within(list).getByText('Café Fazenda Lagoa')).toBeTruthy();
    await user.keyboard('{Enter}');
    expect(within(dialog).getByLabelText<HTMLInputElement>('Descrição').value).toBe(
      'Café Fazenda Lagoa',
    );
    expect(within(dialog).getByLabelText<HTMLInputElement>(/Valor \(BRL\)/).value).toMatch(/34,50/);
    expect(within(dialog).getByRole('button', { name: /Categoria: / }).textContent).toContain(
      cat.name,
    );
  });

  it('creates an expense via the N shortcut, with a searched category and tags', async () => {
    const user = userEvent.setup();
    await openTable();
    await user.keyboard('n');
    const dialog = await screen.findByRole('dialog', { name: 'Novo lançamento' });
    await user.type(within(dialog).getByLabelText('Descrição'), 'Sorveteria da praça');
    await user.type(within(dialog).getByLabelText(/Valor \(BRL\)/), '27,50');
    await user.click(within(dialog).getByRole('button', { name: /Categoria: Sem categoria/ }));
    await user.type(within(dialog).getByRole('combobox', { name: 'Buscar categoria' }), 'restaur');
    await user.keyboard('{Enter}');
    await user.type(within(dialog).getByLabelText('Tags'), 'passeio{Enter}');
    await user.click(within(dialog).getByRole('button', { name: 'Lançar' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Novo lançamento' })).toBeNull(),
    );

    const page = await query({ text: 'sorveteria' });
    expect(page.total).toBe(1);
    const t = page.items[0]!;
    expect(t.amount).toBe(-2_750);
    expect(t.kind).toBe('expense');
    expect(t.category_path).toBe('Alimentação › Restaurantes');
    expect(t.tags.map((x) => x.name)).toEqual(['passeio']);
    expect(await within(table()).findByText('Sorveteria da praça')).toBeTruthy();
  });

  it('edits and splits a transaction checking the sum', async () => {
    const user = userEvent.setup();
    const accounts = async () => (await mockCall('accounts.list', {})) as AccountView[];
    const { queryClient } = await openTable();
    const checking = (await accounts())[0]!;
    await mockCall('transactions.create', {
      account_id: checking.id,
      date: todayIso(),
      amount: -20_000,
      description: 'Compra no atacado',
    });
    await queryClient.invalidateQueries();
    const row = (await within(table()).findByText('Compra no atacado')).closest('tr')!;
    await user.dblClick(row);
    const dialog = await screen.findByRole('dialog', { name: 'Editar lançamento' });
    await user.click(within(dialog).getByRole('checkbox', { name: /Dividir em categorias/ }));
    const part1 = within(dialog).getByLabelText('Valor da parte 1');
    await user.clear(part1);
    await user.type(part1, '150,00');
    await user.type(within(dialog).getByLabelText('Valor da parte 2'), '40,00');
    expect(within(dialog).getByRole('status').textContent).toMatch(/faltam/);
    await user.click(within(dialog).getByRole('button', { name: 'Salvar alterações' }));
    expect(
      await within(dialog).findByText('A soma das partes precisa ser igual ao total.'),
    ).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Pôr o restante na última' }));
    expect(within(dialog).getByRole('status').textContent).toMatch(/confere/);
    await user.click(within(dialog).getByRole('button', { name: /Categoria da parte 1/ }));
    await user.type(
      within(dialog).getByRole('combobox', { name: 'Buscar categoria da parte 1' }),
      'mercado',
    );
    await user.keyboard('{Enter}');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar alterações' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Editar lançamento' })).toBeNull(),
    );

    const t = (await query({ text: 'atacado' })).items.find(
      (x) => x.description === 'Compra no atacado',
    )!;
    expect(t.splits.map((s) => s.amount)).toEqual([-15_000, -5_000]);
    const bought = (await within(table()).findByText('Compra no atacado')).closest('tr')!;
    expect(await within(bought).findByText('Dividido em 2')).toBeTruthy();
  });

  it('reads the filters from the drill-down link and reflects changes in the URL', async () => {
    const user = userEvent.setup();
    const from = `${todayIso().slice(0, 4)}-01-01`;
    await openTable(`#/lancamentos?q=uber&from=${from}&to=${todayIso()}&kind=expense`);
    expect(screen.getByLabelText<HTMLInputElement>('Buscar lançamentos').value).toBe('uber');
    expect(screen.getByRole('button', { name: /Tipo: Despesa/ })).toBeTruthy();
    const page = await query({
      text: 'uber',
      date_from: from,
      date_to: todayIso(),
      kinds: ['expense'],
    });
    await waitFor(() =>
      expect(within(table()).getAllByRole('row').length - 1).toBe(Math.min(page.total, 100)),
    );
    expect(page.total).toBeGreaterThan(0);
    for (const cell of within(table()).queryAllByText(/uber/i))
      expect(cell.textContent).toMatch(/uber/i);

    await user.click(screen.getByRole('checkbox', { name: 'Só sem categoria' }));
    await waitFor(() => expect(window.location.hash).toContain('uncategorized=1'));
    expect(window.location.hash).toContain('q=uber');

    await user.click(screen.getByRole('button', { name: /Período:/ }));
    await user.click(screen.getByRole('button', { name: 'Mês passado' }));
    // The shortcut fills in the dates and the panel stays open until "Aplicar período".
    expect(window.location.hash).toContain(`from=${from}`);
    expect(screen.getByRole('button', { name: 'Mês passado' }).getAttribute('aria-pressed')).toBe(
      'true',
    );
    await user.click(screen.getByRole('button', { name: 'Aplicar período' }));
    await waitFor(() => expect(window.location.hash).not.toContain(`from=${from}`));
  });

  it('categorizes the selected ones in bulk and undoes', async () => {
    const user = userEvent.setup();
    await openTable('#/lancamentos?uncategorized=1&kind=expense&periodo=tudo');
    const before = await query({ uncategorized: true, kinds: ['expense'] });
    expect(before.total).toBeGreaterThan(1);
    await user.click(screen.getByRole('checkbox', { name: 'Selecionar todos desta página' }));
    const bar = screen.getByRole('toolbar', { name: 'Ações nos lançamentos marcados' });
    await user.click(within(bar).getByRole('button', { name: /Categorizar marcados/ }));
    await user.type(
      screen.getByRole('combobox', { name: 'Buscar categorizar marcados' }),
      'outras desp',
    );
    await user.keyboard('{Enter}');
    await waitFor(async () =>
      expect((await query({ uncategorized: true, kinds: ['expense'] })).total).toBeLessThan(
        before.total,
      ),
    );
    await user.click(await screen.findByRole('button', { name: 'Desfazer' }));
    await waitFor(async () =>
      expect((await query({ uncategorized: true, kinds: ['expense'] })).total).toBe(before.total),
    );
  });

  it('contribution shows what it is in the category column and is left out when categorizing in bulk', async () => {
    await openTable('#/lancamentos?kind=investment&periodo=tudo');
    const inv = await query({ kinds: ['investment'] });
    const compra = inv.items.find((x) => x.amount < 0 && !x.counterpart_account_name)!;
    const row = await waitFor(() => {
      const r = table().querySelector<HTMLElement>(`tbody tr[data-row-id="${compra.id}"]`);
      expect(r).toBeTruthy();
      return r!;
    });
    expect(within(row).getByText('Aporte em investimentos')).toBeTruthy();
    // Investment inflow (sale or redemption): the badge says Redemption, not Contribution.
    const venda = inv.items.find((x) => x.amount > 0 && !x.counterpart_account_name);
    if (venda) {
      const vr = table().querySelector<HTMLElement>(`tbody tr[data-row-id="${venda.id}"]`)!;
      expect(within(vr).getByText('Resgate')).toBeTruthy();
      expect(within(vr).getByText('Resgate de investimentos')).toBeTruthy();
    }
    expect(within(row).getByText('Aporte')).toBeTruthy();
    // In bulk: the contribution is left out and comes back in the count; contributions only, error.
    const exp = (await query({ kinds: ['expense'], limit: 1 })).items[0]!;
    const cats = (await mockCall('categories.list', {})) as { id: string; kind: string }[];
    const cat = cats.find((c) => c.kind === 'expense')!;
    const r = (await mockCall('transactions.set_category', {
      ids: [exp.id, compra.id],
      category_id: cat.id,
    })) as BulkResult;
    expect(r.skipped).toBe(1);
    await expect(
      mockCall('transactions.set_category', { ids: [compra.id], category_id: cat.id }),
    ).rejects.toThrow(/não usam categoria/);
  });

  it('the c shortcut on selected rows opens the category picker next to the mouse, wide', async () => {
    const user = userEvent.setup();
    await openTable('#/lancamentos?uncategorized=1&kind=expense&periodo=tudo');
    await user.click(screen.getByRole('checkbox', { name: 'Selecionar todos desta página' }));
    const row = table().querySelector<HTMLElement>('tbody tr[data-row-id]')!;
    row.focus();
    setLastPointerForTests({ x: 300, y: 500 });
    await user.keyboard('c');
    const search = await screen.findByRole('combobox', { name: 'Buscar categorizar marcados' });
    const panel = search.closest<HTMLElement>('[role="dialog"]')!;
    await waitFor(() => expect(panel.style.left).toBe('300px'));
    expect(panel.style.position).toBe('fixed');
    expect(panel.style.top).toBe('504px');
    expect(panel.className).toContain('w-[min(28rem');
    setLastPointerForTests(null);
  });

  it('deletes with confirmation and does not delete reconciled ones', async () => {
    const user = userEvent.setup();
    await openTable('#/lancamentos?status=reconciled&periodo=tudo');
    const reconciled = await query({ statuses: ['reconciled'], limit: 1 });
    const name = reconciled.items[0]!.description;
    const row = (await within(table()).findAllByText(name))[0]!.closest('tr')!;
    await user.click(within(row).getByRole('checkbox'));
    await user.click(screen.getByRole('button', { name: 'Apagar' }));
    const confirm = await screen.findByRole('dialog', { name: /Apagar 1 lançamento/ });
    await user.click(within(confirm).getByRole('button', { name: 'Apagar' }));
    expect(
      await within(confirm).findByText(/está conciliado; desconcilie antes de apagar/),
    ).toBeTruthy();
  });

  it('creates a transfer between currencies', async () => {
    const user = userEvent.setup();
    await openTable();
    await user.click(screen.getByRole('button', { name: 'Transferência' }));
    const dialog = await screen.findByRole('dialog', { name: 'Novo lançamento' });
    await user.click(within(dialog).getByRole('combobox', { name: /Para \(destino\)/ }));
    await user.click(await screen.findByRole('option', { name: /Wise USD/ }));
    await user.type(within(dialog).getByLabelText(/Valor que sai/), '540,00');
    await user.type(within(dialog).getByLabelText(/Valor que entra \(USD\)/), '100,00');
    expect(within(dialog).getByText(/1 USD = 5,4 BRL/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Lançar' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Novo lançamento' })).toBeNull(),
    );
    const legs: TxnView[] = (
      await query({ kinds: ['transfer'], date_from: todayIso(), date_to: todayIso() })
    ).items;
    const usd = legs.find((t) => t.currency === 'USD' && t.amount === 10_000);
    expect(usd?.counterpart_account_name).toBe('Itaú Conta Corrente');
  });

  it('creates a recurrence with a preview of the next dates in Portuguese', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos?aba=recorrencias');
    await screen.findByRole('grid', { name: 'Recorrências' });
    await user.click(screen.getAllByRole('button', { name: 'Nova recorrência' })[0]!);
    const dialog = await screen.findByRole('dialog', { name: 'Nova recorrência' });
    await user.type(within(dialog).getByLabelText('Descrição'), 'Curso de inglês');
    await user.type(within(dialog).getByLabelText(/Valor/), '320,00');
    const day = within(dialog).getByLabelText(/Dia do mês/);
    await user.clear(day);
    await user.type(day, '20');
    expect(await within(dialog).findByText('todo mês no dia 20')).toBeTruthy();
    expect(within(dialog).getAllByRole('listitem').length).toBe(6);
    await user.click(within(dialog).getByRole('button', { name: 'Criar recorrência' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Nova recorrência' })).toBeNull(),
    );
    expect(await screen.findByText('Curso de inglês')).toBeTruthy();
    const planned = await query({ text: 'curso de ingles', statuses: ['planned'] });
    expect(planned.total).toBeGreaterThan(0);
  });

  it('row context menu: status, rule and delete with confirmation', async () => {
    const user = userEvent.setup();
    await openTable();
    const accounts = (await mockCall('accounts.list', {})) as AccountView[];
    await mockCall('transactions.create', {
      account_id: accounts[0]!.id,
      date: todayIso(),
      amount: -4_200,
      description: 'Banca do Zé',
      status: 'cleared',
    });
    await user.type(screen.getByLabelText('Buscar lançamentos'), 'banca do{Enter}');
    const row = (await within(table()).findByText('Banca do Zé')).closest('tr')!;
    await user.pointer({ keys: '[MouseRight]', target: row });
    // The row becomes the cursor (focus); `aria-selected` stays with the batch marks.
    expect(row.hasAttribute('data-cursor')).toBe(true);
    expect(row.getAttribute('aria-selected')).toBe('false');
    let menu = screen.getByRole('menu', { name: 'Ações de Banca do Zé' });
    // Already cleared: only offers the other status.
    expect(within(menu).queryByRole('menuitem', { name: 'Marcar como efetivado' })).toBeNull();
    expect(within(menu).getByRole('menuitem', { name: /Categorizar/ })).toBeTruthy();
    expect(within(menu).getByRole('menuitem', { name: 'Criar regra a partir deste' })).toBeTruthy();
    await user.click(within(menu).getByRole('menuitem', { name: 'Marcar como previsto' }));
    expect((await screen.findAllByText('Status: previsto')).length).toBeGreaterThan(0);
    await waitFor(async () =>
      expect((await query({ text: 'banca do ze' })).items[0]?.status).toBe('planned'),
    );

    // Shift+F10 on the focused row and "Apagar…" asks for confirmation.
    const again = within(table()).getByText('Banca do Zé').closest('tr')!;
    again.focus();
    await user.keyboard('{Shift>}{F10}{/Shift}');
    menu = screen.getByRole('menu', { name: 'Ações de Banca do Zé' });
    await user.click(within(menu).getByRole('menuitem', { name: 'Apagar…' }));
    const confirm = await screen.findByRole('dialog', { name: 'Apagar 1 lançamento' });
    await user.click(within(confirm).getByRole('button', { name: 'Apagar' }));
    await waitFor(async () => expect((await query({ text: 'banca do ze' })).total).toBe(0));
    expect(await screen.findByText('Lançamentos apagados')).toBeTruthy();
  });

  it('selects those with the same description via the menu and categorizes, sets holder and tag in bulk', async () => {
    const user = userEvent.setup();
    const { queryClient } = await openTable('#/lancamentos?q=padaria');
    const accounts = (await mockCall('accounts.list', {})) as AccountView[];
    const acc = accounts.find((a) => a.kind === 'checking')!;
    const mk = (date: string, description: string) =>
      mockCall('transactions.create', { account_id: acc.id, date, amount: -1_990, description });
    const today = todayIso();
    await mk(today, 'PADARIA BOM PAO');
    await mk('2026-01-05', 'Padaria  Bom Pão');
    await mk(today, 'Padaria Central');
    await queryClient.invalidateQueries();
    const row = (await within(table()).findAllByText('PADARIA BOM PAO'))[0]!.closest('tr')!;
    await user.pointer({ keys: '[MouseRight]', target: row });
    await user.click(screen.getByRole('menuitem', { name: 'Marcar os de mesma descrição' }));
    const bar = await screen.findByRole('toolbar', { name: 'Ações nos lançamentos marcados' });
    expect(within(bar).getByText('2 marcados')).toBeTruthy();
    expect(
      await screen.findByText(/Mesma descrição \(“PADARIA BOM PAO”\), em todo o período/),
    ).toBeTruthy();
    // A rule can be created from the selected description.
    expect(within(bar).getByRole('button', { name: 'Criar regra' })).toBeTruthy();

    await user.click(within(bar).getByRole('button', { name: /Categorizar marcados/ }));
    await user.type(
      screen.getByRole('combobox', { name: 'Buscar categorizar marcados' }),
      'padaria',
    );
    await user.keyboard('{Enter}');
    await waitFor(async () => {
      const r = await query({ text: 'bom pao' });
      expect(r.items.every((t) => t.category_id)).toBe(true);
    });
    expect((await query({ text: 'padaria central' })).items[0]!.category_id).toBeNull();

    await user.click(within(bar).getByRole('button', { name: 'Membro dos marcados' }));
    await user.click(await screen.findByRole('button', { name: 'Ana' }));
    await user.click(within(bar).getByRole('button', { name: 'Tags dos marcados' }));
    await user.type(await screen.findByLabelText('Tag'), 'Café da manhã{Enter}');
    await waitFor(async () => {
      const r = await query({ text: 'bom pao' });
      expect(r.items.map((t) => t.member_name)).toEqual(['Ana', 'Ana']);
      expect(r.items.every((t) => t.tags.some((g) => g.name === 'Café da manhã'))).toBe(true);
    });
  });

  it('Shift+↓ selects the range, the batch menu uses the bar actions and Esc deselects', async () => {
    const user = userEvent.setup();
    await openTable('#/lancamentos?periodo=tudo&kind=expense');
    const rows = () => [...table().querySelectorAll<HTMLElement>('tbody tr[data-row-id]')];
    const marked = () => rows().filter((r) => r.getAttribute('aria-selected') === 'true');
    await waitFor(() => expect(rows().length).toBeGreaterThan(4));
    const ids = rows()
      .slice(0, 3)
      .map((r) => r.dataset.rowId!);

    await user.click(rows()[0]!.querySelectorAll('td')[1]!);
    expect(marked()).toHaveLength(0);
    await user.keyboard('{Shift>}{ArrowDown}{ArrowDown}{/Shift}');
    expect(marked().map((r) => r.dataset.rowId)).toEqual(ids);
    const bar = screen.getByRole('toolbar', { name: 'Ações nos lançamentos marcados' });
    expect(within(bar).getByText('3 marcados')).toBeTruthy();
    expect(within(rows()[1]!).getByRole<HTMLInputElement>('checkbox').checked).toBe(true);

    // Right click on a selected row: the batch menu, with the title and shortcuts.
    await user.pointer({ keys: '[MouseRight]', target: rows()[1]!.querySelectorAll('td')[1]! });
    const menu = screen.getByRole('menu', { name: 'Ações de 3 lançamentos marcados' });
    expect(within(menu).getByText('3 lançamentos marcados')).toBeTruthy();
    await user.click(within(menu).getByRole('menuitem', { name: 'Marcar como previsto' }));
    await waitFor(async () => {
      const page = await query({ statuses: ['planned'], limit: 500 });
      expect(ids.every((id) => page.items.some((t) => t.id === id))).toBe(true);
    });

    // ⌘⌫/Ctrl+Backspace asks for the bar confirmation, without opening the menu.
    await user.keyboard('{Control>}{Backspace}{/Control}');
    const confirm = await screen.findByRole('dialog', { name: 'Apagar 3 lançamentos' });
    await user.click(within(confirm).getByRole('button', { name: 'Cancelar' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    rows()[1]!.focus();
    await user.keyboard('{Escape}');
    expect(marked()).toHaveLength(0);
    expect(screen.queryByRole('toolbar', { name: 'Ações nos lançamentos marcados' })).toBeNull();
  });
});
