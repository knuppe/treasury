/**
 * Projects against the mock: create in 05 · Orçamento › Projetos, search and add transactions,
 * spending × budget; an asset improvement with a new project (from the context menu of the
 * asset list) so the project spending counts; project via the batch bar in 04 · Lançamentos;
 * hand-entered items (expense, income, on an account), the timeline and removing from the project.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { ProjectView } from '@/api/generated/ProjectView';
import type { TxnView } from '@/api/generated/TxnView';
import { mockCall } from '@/api/mock';
import { fixToday } from '@/features/portfolio/testing';
import { chartOption, clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeEach(() => {
  fixToday(); // 14/10/2026
  clearCharts();
});
afterEach(() => {
  vi.useRealTimers();
});

async function expense(description: string, amount: number, date = '2026-10-14') {
  const accounts = (await mockCall('accounts.list', {})) as AccountView[];
  const acc = accounts.find((a) => a.kind === 'checking')!;
  return (await mockCall('transactions.create', {
    account_id: acc.id,
    date,
    amount: -amount,
    description,
  })) as TxnView;
}

describe('Projects with the mock', () => {
  it('creates a project, adds transactions and compares the spending with the budget', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/orcamento?aba=projetos');
    await user.click(await screen.findByRole('button', { name: 'Novo projeto' }));
    const dialog = await screen.findByRole('dialog', { name: 'Novo projeto' });
    await user.type(within(dialog).getByLabelText('Nome'), 'Reforma da cozinha');
    await user.type(within(dialog).getByLabelText('Orçamento'), '10.000,00');
    await user.click(within(dialog).getByRole('button', { name: 'Criar projeto' }));
    const detail = await screen.findByRole('region', { name: 'Projeto Reforma da cozinha' });

    await expense('Marcenaria Silva', 400_000);
    await expense('PIX pintor', 150_000);
    await expense('Padaria', 2_000);
    await queryClient.invalidateQueries();
    await user.click(within(detail).getByRole('button', { name: 'Lançamentos' }));
    const pick = await screen.findByRole('dialog', { name: 'Lançamentos para Reforma da cozinha' });
    const found = within(pick).getByRole('list', { name: 'Lançamentos encontrados' });
    await user.click(await within(found).findByLabelText(/Marcenaria Silva/));
    await user.click(within(found).getByLabelText(/PIX pintor/));
    await user.click(within(pick).getByRole('button', { name: 'Pôr no projeto' }));
    expect(await screen.findByText('Lançamentos em Reforma da cozinha')).toBeTruthy();

    const [p] = (await mockCall('projects.list', {})) as ProjectView[];
    expect(p).toMatchObject({
      spent: 550_000,
      budget: 1_000_000,
      remaining: 450_000,
      txn_count: 2,
    });
    const d2 = await screen.findByRole('region', { name: 'Projeto Reforma da cozinha' });
    await waitFor(() =>
      expect(within(d2).getByRole('meter', { name: '55% do orçamento' })).toBeTruthy(),
    );
    const link = within(d2).getByRole('link', { name: 'Ver lançamentos' });
    expect(link.getAttribute('href')).toBe(`#/lancamentos?project=${p!.id}`);
  });

  it('improvement from the asset list menu with a new project counts the project spending', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/carteira?aba=bens');
    const list = await screen.findByRole('grid', { name: 'Bens' });
    const row = within(list).getAllByText('Onix LTZ 2022')[0]!.closest('tr')!;
    await user.pointer({ keys: '[MouseRight]', target: row });
    const menu = screen.getByRole('menu', { name: 'Ações de Onix LTZ 2022' });
    expect(within(menu).getByRole('menuitem', { name: 'Novo projeto do bem…' })).toBeTruthy();
    await user.click(within(menu).getByRole('menuitem', { name: 'Registrar benfeitoria…' }));
    const dialog = await screen.findByRole('dialog', { name: 'Registrar benfeitoria ou obra' });
    await user.type(within(dialog).getByLabelText('Descrição'), 'Funilaria');
    await user.click(within(dialog).getByRole('combobox', { name: 'Projeto dos gastos' }));
    await user.click(
      await screen.findByRole('option', { name: 'Novo projeto com o nome da benfeitoria' }),
    );
    await user.type(within(dialog).getByLabelText('Valor estimado'), '3.000,00');
    await user.click(within(dialog).getByRole('button', { name: 'Registrar' }));
    expect(await screen.findByText('Benfeitoria registrada')).toBeTruthy();

    const projects = (await mockCall('projects.list', {})) as ProjectView[];
    const funilaria = projects.find((p) => p.name === 'Funilaria')!;
    expect(funilaria.physical_asset_name).toBe('Onix LTZ 2022');
    const t1 = await expense('Oficina do Zé', 250_000, '2026-10-01');
    const t2 = await expense('Peças', 80_000, '2026-10-05');
    await mockCall('projects.link', { txn_ids: [t1.id, t2.id], project_id: funilaria.id });
    await queryClient.invalidateQueries();

    const car = (await mockCall('physical.get', { id: 'phys-onix' })) as PhysicalAssetView;
    const imp = car.improvements.find((i) => i.description === 'Funilaria')!;
    expect(imp).toMatchObject({ amount: 330_000, entered_amount: 300_000, linked_count: 2 });
    const detail = await screen.findByRole('region', { name: 'Bem Onix LTZ 2022' });
    const works = within(detail).getByRole('grid', { name: 'Benfeitorias de Onix LTZ 2022' });
    expect(await within(works).findByRole('link', { name: '2 lançamentos' })).toBeTruthy();
  });

  it('puts the checked transactions in a new project via the batch bar', async () => {
    const user = userEvent.setup();
    const a = await (async () => {
      renderApp('unlocked', {}, '#/lancamentos?q=Festa');
      return [await expense('Festa buffet', 300_000), await expense('Festa decoração', 90_000)];
    })();
    const table = await screen.findByRole('grid', { name: /Lançamentos/ });
    for (const t of a)
      await user.click(
        await within(table).findByRole('checkbox', { name: new RegExp(t.description) }),
      );
    const bar = screen.getByRole('toolbar', { name: 'Ações nos lançamentos marcados' });
    await user.click(within(bar).getByRole('button', { name: 'Projeto…' }));
    const dialog = await screen.findByRole('dialog', { name: 'Projeto de 2 lançamentos' });
    await user.click(within(dialog).getByRole('combobox', { name: 'Projeto' }));
    await user.click(await screen.findByRole('option', { name: 'Novo projeto…' }));
    await user.type(within(dialog).getByLabelText('Nome do novo projeto'), 'Festa de 15 anos');
    await user.click(within(dialog).getByRole('button', { name: 'Pôr no projeto' }));
    expect((await screen.findAllByText('Lançamentos no projeto')).length).toBeGreaterThan(0);
    const [p] = (await mockCall('projects.list', {})) as ProjectView[];
    expect(p).toMatchObject({ name: 'Festa de 15 anos', spent: 390_000, txn_count: 2 });
  });

  it('a completed project stays in the list, with the status filter', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento?aba=projetos&situacao=abertos');
    await user.click(await screen.findByRole('button', { name: 'Novo projeto' }));
    const dialog = await screen.findByRole('dialog', { name: 'Novo projeto' });
    await user.type(within(dialog).getByLabelText('Nome'), 'Pintura da casa');
    await user.click(within(dialog).getByRole('button', { name: 'Criar projeto' }));
    const detail = await screen.findByRole('region', { name: 'Projeto Pintura da casa' });
    await user.click(within(detail).getByRole('button', { name: 'Editar' }));
    const edit = await screen.findByRole('dialog', { name: 'Editar Pintura da casa' });
    await user.click(within(edit).getByRole('radio', { name: 'Concluído' }));
    await user.click(within(edit).getByRole('button', { name: 'Salvar' }));
    expect(await screen.findByText('Projeto salvo')).toBeTruthy();
    // Left the "Em aberto" filter: the list goes back to all and the project stays open.
    const d2 = await screen.findByRole('region', { name: 'Projeto Pintura da casa' });
    expect(within(d2).getByText(/^Concluído/)).toBeTruthy();
    const list = screen.getByRole('grid', { name: 'Projetos' });
    expect(within(list).getByText('Pintura da casa')).toBeTruthy();
    // Em aberto: empty, without the screen disappearing; Concluídos: the project.
    await user.click(screen.getByRole('radio', { name: 'Em aberto (0)' }));
    expect(await screen.findByText('Nenhum projeto em aberto: veja os concluídos.')).toBeTruthy();
    await user.click(screen.getByRole('radio', { name: 'Concluídos (1)' }));
    expect(
      within(screen.getByRole('grid', { name: 'Projetos' })).getByText('Pintura da casa'),
    ).toBeTruthy();
  });

  it('enters items by hand, income and on the account, with the timeline and the entries', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento?aba=projetos');
    await user.click(await screen.findByRole('button', { name: 'Novo projeto' }));
    const form = await screen.findByRole('dialog', { name: 'Novo projeto' });
    await user.type(within(form).getByLabelText('Nome'), 'Casamento');
    await user.type(within(form).getByLabelText('Orçamento'), '20.000,00');
    await user.click(within(form).getByRole('button', { name: 'Criar projeto' }));
    const detail = await screen.findByRole('region', { name: 'Projeto Casamento' });

    const newItem = async (
      desc: string,
      value: string,
      opts: { income?: boolean; account?: string } = {},
    ) => {
      await user.click(within(detail).getByRole('button', { name: 'Novo item' }));
      const d = await screen.findByRole('dialog', { name: 'Novo item em Casamento' });
      if (opts.income) await user.click(within(d).getByRole('radio', { name: 'Receita' }));
      await user.type(within(d).getByLabelText('Descrição'), desc);
      await user.type(within(d).getByLabelText('Valor'), value);
      if (opts.account) {
        await user.click(within(d).getByRole('combobox', { name: 'Onde lançar' }));
        await user.click(await screen.findByRole('option', { name: opts.account }));
      }
      await user.click(
        within(d).getByRole('button', {
          name: opts.account ? 'Lançar na conta' : 'Lançar no projeto',
        }),
      );
      await waitFor(() =>
        expect(screen.queryByRole('dialog', { name: 'Novo item em Casamento' })).toBeNull(),
      );
    };
    await newItem('Buffet (dinheiro)', '8.000,00');
    await newItem('Presente dos padrinhos', '1.500,00', { income: true });
    await newItem('Decoração', '2.000,00', { account: 'Itaú Conta Corrente' });

    const p = ((await mockCall('projects.list', {})) as ProjectView[]).find(
      (x) => x.name === 'Casamento',
    )!;
    expect(p).toMatchObject({
      spent: 1_000_000,
      income: 150_000,
      net: -850_000,
      txn_count: 1,
      item_count: 2,
      remaining: 1_000_000,
    });
    const entries = await within(detail).findByRole('grid', { name: 'Entradas de Casamento' });
    await waitFor(() => expect(within(entries).getAllByRole('row')).toHaveLength(5)); // header, 3, total
    expect(within(entries).getByText('Itaú Conta Corrente')).toBeTruthy();
    expect(within(entries).getAllByText('Item do projeto')).toHaveLength(2);
    expect(within(detail).getByRole('meter', { name: '50% do orçamento' })).toBeTruthy();

    // Timeline: cumulative with the budget; per day, income and expense bars.
    await waitFor(() => chartOption(/Despesa e receita acumuladas de Casamento/));
    const acc = chartOption(/Despesa e receita acumuladas de Casamento/);
    expect((acc.series as { name: string }[]).map((x) => x.name)).toEqual([
      'Despesa acumulada',
      'Receita acumulada',
      'Resultado',
    ]);
    await user.click(within(detail).getByRole('radio', { name: 'Por dia' }));
    const per = chartOption(/Receitas e despesas de Casamento por dia/);
    expect((per.series as { type: string }[]).map((x) => x.type)).toEqual(['bar', 'bar', 'line']);

    // Edit the item from the row (context menu).
    const gift = within(entries).getByText('Presente dos padrinhos').closest('tr')!;
    await user.pointer({ keys: '[MouseRight]', target: gift });
    const menu = screen.getByRole('menu', { name: 'Ações de Presente dos padrinhos' });
    await user.click(within(menu).getByRole('menuitem', { name: /Editar item…/ }));
    const edit = await screen.findByRole('dialog', { name: 'Editar Presente dos padrinhos' });
    const value = within(edit).getByLabelText('Valor');
    await user.clear(value);
    await user.type(value, '2.000,00');
    await user.click(within(edit).getByRole('button', { name: 'Salvar' }));
    expect(await screen.findByText('Item salvo')).toBeTruthy();

    // Removing the account transaction from the project: it stays on the account.
    const deco = within(entries).getByText('Decoração').closest('tr')!;
    await user.pointer({ keys: '[MouseRight]', target: deco });
    await user.click(
      within(screen.getByRole('menu', { name: 'Ações de Decoração' })).getByRole('menuitem', {
        name: 'Tirar do projeto…',
      }),
    );
    const confirm = await screen.findByRole('dialog', { name: 'Tirar lançamento do projeto' });
    await user.click(within(confirm).getByRole('button', { name: 'Tirar do projeto' }));
    expect(await screen.findByText('Entradas tiradas do projeto')).toBeTruthy();
    const after = (await mockCall('projects.get', { id: p.id })) as ProjectView;
    expect(after).toMatchObject({ spent: 800_000, income: 200_000, txn_count: 0, item_count: 2 });
    const txns = (await mockCall('transactions.query', { text: 'Decoração' })) as {
      items: TxnView[];
    };
    expect(txns.items[0]).toMatchObject({ amount: -200_000, project_id: null });

    // Several entries: Ctrl+A selects both and Ctrl+⌫ removes (deletes the items) at once.
    await waitFor(() => expect(within(entries).queryByText('Decoração')).toBeNull());
    await user.click(within(entries).getByText('Presente dos padrinhos'));
    await user.keyboard('{Control>}a{/Control}');
    expect(await screen.findByText('2 entradas selecionadas')).toBeTruthy();
    await user.pointer({
      keys: '[MouseRight]',
      target: within(entries).getByText('Buffet (dinheiro)'),
    });
    const many = screen.getByRole('menu', { name: 'Ações de 2 entradas' });
    expect(
      within(many).getByRole('menuitem', { name: /Tirar 2 entradas do projeto/ }),
    ).toBeTruthy();
    await user.keyboard('{Escape}');
    await user.keyboard('{Control>}{Backspace}{/Control}');
    const both = await screen.findByRole('dialog', { name: 'Tirar 2 entradas do projeto' });
    await user.click(within(both).getByRole('button', { name: 'Excluir' }));
    await waitFor(async () =>
      expect(await mockCall('projects.get', { id: p.id })).toMatchObject({
        spent: 0,
        income: 0,
        item_count: 0,
      }),
    );
  });
});
