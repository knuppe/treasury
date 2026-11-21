import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import type { ContextMenuEntry } from './ContextMenu';
import { TechTable, type TechColumn, type TechTableMenuContext } from './TechTable';

interface Row {
  id: string;
  name: string;
  cents: number;
}

const ROWS: Row[] = [
  { id: 'a', name: 'Mercado', cents: 23450 },
  { id: 'b', name: 'Aluguel', cents: 280000 },
  { id: 'c', name: 'Farmácia', cents: 4590 },
  { id: 'd', name: 'Escola', cents: 150000 },
];

const COLUMNS: TechColumn<Row>[] = [
  { key: 'name', header: 'Descrição', sortValue: (r) => r.name },
  { key: 'cents', header: 'Valor', numeric: true, sortValue: (r) => r.cents },
];

function rowNames(): string[] {
  const grid = screen.getByRole('grid', { name: 'Lançamentos' });
  return within(grid)
    .getAllByRole('row')
    .slice(1)
    .map((row) => within(row).getAllByRole('gridcell')[0]!.textContent ?? '');
}

function selectedName(): string | undefined {
  const grid = screen.getByRole('grid', { name: 'Lançamentos' });
  const row = within(grid)
    .getAllByRole('row')
    .find((r) => r.getAttribute('aria-selected') === 'true');
  return row ? (within(row).getAllByRole('gridcell')[0]!.textContent ?? '') : undefined;
}

describe('<TechTable>', () => {
  it('alinha colunas numéricas à direita com fonte mono', () => {
    render(<TechTable label="Lançamentos" columns={COLUMNS} rows={ROWS} getRowId={(r) => r.id} />);
    const cell = screen.getByText('23450');
    expect(cell.className).toContain('text-right');
    expect(cell.className).toContain('num');
  });

  it('só uma linha entra na ordem do Tab', async () => {
    const user = userEvent.setup();
    render(<TechTable label="Lançamentos" columns={COLUMNS} rows={ROWS} getRowId={(r) => r.id} />);
    // Tab passa pelos botões de ordenação e pelo seletor de colunas e chega à primeira linha.
    for (let i = 0; i < 10 && !document.activeElement?.getAttribute('data-row-id'); i++)
      await user.tab();
    expect(document.activeElement?.getAttribute('data-row-id')).toBe('a');
    const tabbable = screen.getAllByRole('row').filter((r) => r.getAttribute('tabindex') === '0');
    expect(tabbable).toHaveLength(1);
  });

  it('↑/↓ movem seleção e foco; Home/End vão às pontas; Enter ativa', async () => {
    const user = userEvent.setup();
    const onActivate = vi.fn();
    const onSelectedChange = vi.fn();
    render(
      <TechTable
        label="Lançamentos"
        columns={COLUMNS}
        rows={ROWS}
        getRowId={(r) => r.id}
        onActivate={onActivate}
        onSelectedChange={onSelectedChange}
      />,
    );
    await user.click(screen.getByText('Mercado'));
    expect(selectedName()).toBe('Mercado');

    await user.keyboard('{ArrowDown}');
    expect(selectedName()).toBe('Aluguel');
    expect(document.activeElement?.getAttribute('data-row-id')).toBe('b');

    await user.keyboard('{ArrowDown}{ArrowDown}{ArrowDown}');
    expect(selectedName()).toBe('Escola');

    await user.keyboard('{ArrowUp}');
    expect(selectedName()).toBe('Farmácia');

    await user.keyboard('{Home}');
    expect(selectedName()).toBe('Mercado');

    await user.keyboard('{End}');
    expect(selectedName()).toBe('Escola');
    expect(onSelectedChange).toHaveBeenLastCalledWith('d', ROWS[3]);

    await user.keyboard('{Enter}');
    expect(onActivate).toHaveBeenCalledWith(ROWS[3]);
  });

  it('ordena pelo cabeçalho e informa aria-sort', async () => {
    const user = userEvent.setup();
    render(<TechTable label="Lançamentos" columns={COLUMNS} rows={ROWS} getRowId={(r) => r.id} />);
    const valueHeader = screen.getByRole('columnheader', { name: /Valor/ });
    expect(valueHeader.getAttribute('aria-sort')).toBe('none');

    await user.click(within(valueHeader).getByRole('button'));
    expect(valueHeader.getAttribute('aria-sort')).toBe('ascending');
    expect(rowNames()).toEqual(['Farmácia', 'Mercado', 'Escola', 'Aluguel']);

    await user.click(within(valueHeader).getByRole('button'));
    expect(valueHeader.getAttribute('aria-sort')).toBe('descending');
    expect(rowNames()).toEqual(['Aluguel', 'Escola', 'Mercado', 'Farmácia']);

    const nameHeader = screen.getByRole('columnheader', { name: /Descrição/ });
    await user.click(within(nameHeader).getByRole('button'));
    // Ordem alfabética pt-BR (acentos não quebram a ordem).
    expect(rowNames()).toEqual(['Aluguel', 'Escola', 'Farmácia', 'Mercado']);
    expect(valueHeader.getAttribute('aria-sort')).toBe('none');
  });

  it('a navegação por teclado segue a ordem visível', async () => {
    const user = userEvent.setup();
    render(
      <TechTable
        label="Lançamentos"
        columns={COLUMNS}
        rows={ROWS}
        getRowId={(r) => r.id}
        defaultSort={{ key: 'cents', direction: 'desc' }}
      />,
    );
    await user.click(screen.getByText('Aluguel'));
    await user.keyboard('{ArrowDown}');
    expect(selectedName()).toBe('Escola');
  });

  it('mostra o estado vazio', () => {
    render(
      <TechTable
        label="Lançamentos"
        columns={COLUMNS}
        rows={[]}
        getRowId={(r) => r.id}
        empty="Nada por aqui."
      />,
    );
    expect(screen.getByText('Nada por aqui.')).toBeTruthy();
  });

  it('ordena pelo valor da linha sem sortValue e lembra a ordem', async () => {
    const user = userEvent.setup();
    const cols: TechColumn<Row>[] = [
      { key: 'name', header: 'Descrição', cell: (r) => <b>{r.name}</b> },
      { key: 'cents', header: 'Valor', numeric: true },
    ];
    const { unmount } = render(
      <TechTable label="Lançamentos" columns={cols} rows={ROWS} getRowId={(r) => r.id} />,
    );
    await user.click(screen.getByRole('button', { name: /Descrição/ }));
    expect(rowNames()).toEqual(['Aluguel', 'Escola', 'Farmácia', 'Mercado']);
    unmount();
    render(<TechTable label="Lançamentos" columns={cols} rows={ROWS} getRowId={(r) => r.id} />);
    expect(rowNames()).toEqual(['Aluguel', 'Escola', 'Farmácia', 'Mercado']);
  });

  it('esconde e mostra colunas pelo seletor e pelo menu do cabeçalho', async () => {
    const user = userEvent.setup();
    render(<TechTable label="Lançamentos" columns={COLUMNS} rows={ROWS} getRowId={(r) => r.id} />);
    await user.click(screen.getByRole('button', { name: 'Colunas de Lançamentos' }));
    const group = screen.getByRole('group', { name: 'Colunas visíveis' });
    await user.click(within(group).getByRole('checkbox', { name: 'Valor' }));
    expect(screen.queryByRole('columnheader', { name: /Valor/ })).toBeNull();
    // A última coluna visível não pode sumir.
    expect(
      within(group).getByRole('checkbox', { name: 'Descrição' }).hasAttribute('disabled'),
    ).toBe(true);
    await user.click(within(group).getByRole('checkbox', { name: 'Valor' }));
    expect(screen.getByRole('columnheader', { name: /Valor/ })).toBeTruthy();
    await user.keyboard('{Escape}');
    await user.pointer({
      keys: '[MouseRight]',
      target: screen.getByRole('columnheader', { name: /Valor/ }),
    });
    await user.click(await screen.findByRole('menuitem', { name: 'Esconder "Valor"' }));
    expect(screen.queryByRole('columnheader', { name: /Valor/ })).toBeNull();
    expect(JSON.parse(localStorage.getItem('treasury.table.lancamentos') ?? '{}')).toMatchObject({
      hidden: ['cents'],
    });
  });

  it('a largura muda pela alça (setas) e fica guardada; duplo clique volta ao automático', async () => {
    const user = userEvent.setup();
    render(<TechTable label="Lançamentos" columns={COLUMNS} rows={ROWS} getRowId={(r) => r.id} />);
    const handle = screen.getByRole('separator', { name: 'Largura da coluna Descrição' });
    handle.focus();
    await user.keyboard('{ArrowRight}');
    const saved = JSON.parse(localStorage.getItem('treasury.table.lancamentos') ?? '{}') as {
      widths: Record<string, number>;
    };
    expect(saved.widths.name).toBeGreaterThan(saved.widths.cents! - 1);
    expect(screen.getByRole('grid', { name: 'Lançamentos' }).style.tableLayout).toBe('fixed');
    await user.dblClick(handle);
    expect(screen.getByRole('grid', { name: 'Lançamentos' }).style.tableLayout).toBe('');
  });

  it('ordena pelo texto mostrado na célula, nunca pelo id escondido nela', async () => {
    const user = userEvent.setup();
    const names: Record<string, string> = {
      '0b3c2a10-0000-4000-8000-000000000001': 'Zeta',
      '0a3c2a10-0000-4000-8000-000000000002': 'Alfa',
      '0c3c2a10-0000-4000-8000-000000000003': 'Meio',
    };
    const rows: { id: string; account_id: string }[] = Object.keys(names).map((account_id, i) => ({
      id: String(i),
      account_id,
    }));
    render(
      <TechTable
        label="Lançamentos"
        columns={[
          {
            key: 'account_id',
            header: 'Conta',
            cell: (r) => <span>{names[r.account_id]}</span>,
          },
        ]}
        rows={rows}
        getRowId={(r) => r.id}
      />,
    );
    await user.click(screen.getByRole('button', { name: /Conta/ }));
    expect(rowNames()).toEqual(['Alfa', 'Meio', 'Zeta']);
  });
});

function selectedNames(): string[] {
  const grid = screen.getByRole('grid', { name: 'Lançamentos' });
  return within(grid)
    .getAllByRole('row')
    .filter((r) => r.getAttribute('aria-selected') === 'true')
    .map((r) => within(r).getAllByRole('gridcell')[0]!.textContent ?? '');
}

function cursorName(): string | undefined {
  const row = document.activeElement?.closest('tr[data-row-id]');
  return row
    ? (within(row as HTMLElement).getAllByRole('gridcell')[0]!.textContent ?? '')
    : undefined;
}

describe('<TechTable> com seleção de várias linhas', () => {
  function setup(
    opts: {
      selectionMenu?: (rows: Row[], ctx: TechTableMenuContext) => ContextMenuEntry[];
      rowMenu?: (row: Row, ctx: TechTableMenuContext) => ContextMenuEntry[];
    } = {},
  ) {
    const onSelectionChange = vi.fn();
    render(
      <TechTable
        label="Lançamentos"
        columns={COLUMNS}
        rows={ROWS}
        getRowId={(r) => r.id}
        multiSelect
        onSelectionChange={onSelectionChange}
        selectionMenu={opts.selectionMenu}
        rowMenu={opts.rowMenu}
        selectionStatus={(rows) =>
          `${rows.length} itens · ${rows.reduce((s, r) => s + r.cents, 0)}`
        }
      />,
    );
    return { onSelectionChange };
  }

  it('Shift+↓ estende a faixa, Shift+↑ encolhe e a seta sem Shift volta a uma linha', async () => {
    const user = userEvent.setup();
    const { onSelectionChange } = setup();
    expect(screen.getByRole('grid').getAttribute('aria-multiselectable')).toBe('true');
    await user.click(screen.getByText('Mercado'));
    expect(selectedNames()).toEqual(['Mercado']);

    await user.keyboard('{Shift>}{ArrowDown}{ArrowDown}{/Shift}');
    expect(selectedNames()).toEqual(['Mercado', 'Aluguel', 'Farmácia']);
    expect(cursorName()).toBe('Farmácia');
    // Linha de status com a soma que a tela deu.
    expect(screen.getByRole('status').textContent).toBe('3 itens · 308040');

    await user.keyboard('{Shift>}{ArrowUp}{/Shift}');
    expect(selectedNames()).toEqual(['Mercado', 'Aluguel']);
    expect(onSelectionChange).toHaveBeenLastCalledWith(new Set(['a', 'b']), [ROWS[0], ROWS[1]]);

    // Acima da âncora a faixa vira para cima.
    await user.keyboard('{Shift>}{ArrowUp}{ArrowUp}{/Shift}');
    expect(selectedNames()).toEqual(['Mercado']);
    await user.keyboard('{Shift>}{End}{/Shift}');
    expect(selectedNames()).toEqual(['Mercado', 'Aluguel', 'Farmácia', 'Escola']);

    await user.keyboard('{ArrowUp}');
    expect(selectedNames()).toEqual(['Farmácia']);
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('Ctrl+A seleciona tudo, Esc volta ao cursor, Espaço alterna e o mouse usa Shift e Ctrl', async () => {
    const user = userEvent.setup();
    setup();
    await user.click(screen.getByText('Aluguel'));
    await user.keyboard('{Control>}a{/Control}');
    expect(selectedNames()).toEqual(['Mercado', 'Aluguel', 'Farmácia', 'Escola']);
    await user.keyboard('{Escape}');
    expect(selectedNames()).toEqual(['Aluguel']);

    await user.keyboard('{ArrowDown}{ArrowDown}{ }');
    // Espaço na linha já selecionada sozinha não a deixa sem seleção.
    expect(selectedNames()).toEqual(['Escola']);

    await user.click(screen.getByText('Mercado'));
    await user.keyboard('{Shift>}');
    await user.click(screen.getByText('Farmácia'));
    await user.keyboard('{/Shift}');
    expect(selectedNames()).toEqual(['Mercado', 'Aluguel', 'Farmácia']);
    await user.keyboard('{Control>}');
    await user.click(screen.getByText('Aluguel'));
    await user.click(screen.getByText('Escola'));
    await user.keyboard('{/Control}');
    expect(selectedNames()).toEqual(['Mercado', 'Farmácia', 'Escola']);
    expect(cursorName()).toBe('Escola');
  });

  it('menu da seleção com título; fora dela, a seleção volta à linha e abre o da linha', async () => {
    const user = userEvent.setup();
    const remove = vi.fn();
    setup({
      selectionMenu: (rows) => [
        { heading: `${rows.length} lançamentos selecionados` },
        { label: `Excluir ${rows.length}…`, shortcut: ['mod', 'Backspace'], onSelect: remove },
      ],
      rowMenu: (row) => [{ heading: row.name }, { label: 'Editar…', onSelect: vi.fn() }],
    });
    await user.click(screen.getByText('Mercado'));
    await user.keyboard('{Shift>}{ArrowDown}{/Shift}');
    await user.pointer({ keys: '[MouseRight]', target: screen.getByText('Mercado') });
    const menu = screen.getByRole('menu', { name: 'Ações de 2 linhas' });
    expect(within(menu).getByText('2 lançamentos selecionados')).toBeTruthy();
    await user.click(within(menu).getByRole('menuitem', { name: /Excluir 2…/ }));
    expect(remove).toHaveBeenCalledTimes(1);
    // A seleção continua depois de escolher.
    expect(selectedNames()).toEqual(['Mercado', 'Aluguel']);

    await user.pointer({ keys: '[MouseRight]', target: screen.getByText('Escola') });
    expect(screen.getByRole('menu', { name: 'Ações da linha' })).toBeTruthy();
    expect(selectedNames()).toEqual(['Escola']);
    await user.keyboard('{Escape}');

    // Shift+F10 na linha da seleção abre o menu dela.
    await user.keyboard('{Shift>}{ArrowUp}{/Shift}');
    await user.keyboard('{Shift>}{F10}{/Shift}');
    expect(screen.getByRole('menu', { name: 'Ações de 2 linhas' })).toBeTruthy();
  });

  it('os atalhos do menu funcionam sem abrir o menu; desativados não disparam', async () => {
    const user = userEvent.setup();
    const remove = vi.fn();
    const edit = vi.fn();
    const blocked = vi.fn();
    setup({
      selectionMenu: (rows, ctx) => [
        { label: `Copiar ${rows.length} linhas`, shortcut: ['mod', 'C'], onSelect: ctx.copy },
        { label: 'Excluir…', shortcut: ['mod', 'Backspace'], onSelect: remove },
      ],
      rowMenu: () => [
        { label: 'Editar…', shortcut: ['enter'], onSelect: edit },
        { label: 'Bloqueado', shortcut: ['mod', 'Backspace'], onSelect: blocked, disabled: true },
      ],
    });
    await user.click(screen.getByText('Aluguel'));
    await user.keyboard('{Enter}');
    expect(edit).toHaveBeenCalledTimes(1);
    await user.keyboard('{Control>}{Backspace}{/Control}');
    expect(blocked).not.toHaveBeenCalled();

    await user.keyboard('{Shift>}{ArrowDown}{/Shift}');
    await user.keyboard('{Control>}{Backspace}{/Control}');
    expect(remove).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole('menu')).toBeNull();

    await user.keyboard('{Control>}c{/Control}');
    await vi.waitFor(async () =>
      expect(await navigator.clipboard.readText()).toBe(
        'Descrição\tValor\nAluguel\t280000\nFarmácia\t4590',
      ),
    );
  });

  it('marcas: as setas não desmarcam, Espaço alterna, Shift soma faixas e Esc limpa', async () => {
    const user = userEvent.setup();
    function Marked() {
      const [marked, setMarked] = useState<ReadonlySet<string>>(new Set(['d']));
      return (
        <TechTable
          label="Lançamentos"
          columns={COLUMNS}
          rows={ROWS}
          getRowId={(r) => r.id}
          selection={marked}
          onSelectionChange={setMarked}
          selectionMode="marks"
        />
      );
    }
    render(<Marked />);
    await user.click(screen.getByText('Mercado'));
    expect(selectedNames()).toEqual(['Escola']);
    await user.keyboard('{ }{ArrowDown}');
    expect(selectedNames()).toEqual(['Mercado', 'Escola']);
    await user.keyboard('{Shift>}{ArrowDown}{/Shift}');
    expect(selectedNames()).toEqual(['Mercado', 'Aluguel', 'Farmácia', 'Escola']);
    await user.keyboard('{Shift>}{ArrowUp}{/Shift}');
    expect(selectedNames()).toEqual(['Mercado', 'Aluguel', 'Escola']);
    await user.keyboard('{Escape}');
    expect(selectedNames()).toEqual([]);
  });

  it('sem seleção múltipla, nada muda: Shift+↓ só move e não há aria-multiselectable', async () => {
    const user = userEvent.setup();
    render(<TechTable label="Lançamentos" columns={COLUMNS} rows={ROWS} getRowId={(r) => r.id} />);
    expect(screen.getByRole('grid').hasAttribute('aria-multiselectable')).toBe(false);
    await user.click(screen.getByText('Mercado'));
    await user.keyboard('{Shift>}{ArrowDown}{/Shift}{Control>}a{/Control}');
    expect(selectedNames()).toEqual(['Aluguel']);
  });
});
