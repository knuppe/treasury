import { fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ContextMenu, type ContextMenuEntry } from './ContextMenu';
import { TechTable } from './TechTable';

function setup(extra: ContextMenuEntry[] = []) {
  const edit = vi.fn();
  const remove = vi.fn();
  const archive = vi.fn();
  render(
    <ContextMenu
      label="Ações de Nubank"
      items={[
        { heading: 'Nubank' },
        { label: 'Editar', icon: 'sliders', shortcut: ['E'], onSelect: edit },
        { label: 'Arquivar', onSelect: archive, disabled: true, disabledReason: 'Tem saldo' },
        false,
        { separator: true },
        { separator: true },
        ...extra,
        { label: 'Apagar', danger: true, onSelect: remove },
      ]}
    >
      <section aria-label="Card">
        <button type="button">Detalhes</button>
        <input aria-label="Campo" />
      </section>
    </ContextMenu>,
  );
  return { edit, remove, archive };
}

describe('ContextMenu', () => {
  it('opens on right click at the pointer, with titles, separators and disabled items', async () => {
    const user = userEvent.setup();
    setup();
    await user.pointer({
      keys: '[MouseRight]',
      target: screen.getByRole('region', { name: 'Card' }),
    });
    const menu = screen.getByRole('menu', { name: 'Ações de Nubank' });
    const items = within(menu).getAllByRole('menuitem');
    expect(items.map((i) => i.textContent)).toEqual(['EditarE', 'Arquivar', 'Apagar']);
    // Repeated separators become one; the `false` disappears.
    expect(within(menu).getAllByRole('separator')).toHaveLength(1);
    expect(within(menu).getByText('Nubank')).toBeTruthy();
    expect(items[1]!.getAttribute('aria-disabled')).toBe('true');
    expect(items[1]!.getAttribute('title')).toBe('Tem saldo');
    expect(document.activeElement).toBe(items[0]);
  });

  it('keyboard: Shift+F10 opens, arrows skip the disabled item, Enter chooses and returns focus', async () => {
    const user = userEvent.setup();
    const { remove } = setup();
    const trigger = screen.getByRole('button', { name: 'Detalhes' });
    trigger.focus();
    await user.keyboard('{Shift>}{F10}{/Shift}');
    const menu = screen.getByRole('menu');
    const [edit, , apagar] = within(menu).getAllByRole('menuitem');
    expect(document.activeElement).toBe(edit);
    await user.keyboard('{ArrowDown}');
    expect(document.activeElement).toBe(apagar);
    await user.keyboard('{ArrowDown}');
    expect(document.activeElement).toBe(edit);
    await user.keyboard('{ArrowUp}');
    expect(document.activeElement).toBe(apagar);
    await user.keyboard('{Enter}');
    expect(remove).toHaveBeenCalledOnce();
    expect(screen.queryByRole('menu')).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });

  it('Esc closes without choosing and returns focus; the Menu key also opens', async () => {
    const user = userEvent.setup();
    const { edit, remove } = setup();
    const trigger = screen.getByRole('button', { name: 'Detalhes' });
    trigger.focus();
    fireEvent.keyDown(trigger, { key: 'ContextMenu' });
    expect(screen.getByRole('menu')).toBeTruthy();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('menu')).toBeNull();
    expect(document.activeElement).toBe(trigger);
    expect(edit).not.toHaveBeenCalled();
    expect(remove).not.toHaveBeenCalled();
  });

  it('a letter jumps to the item, a click outside closes and the disabled item cannot be chosen', async () => {
    const user = userEvent.setup();
    const { archive, remove } = setup();
    await user.pointer({
      keys: '[MouseRight]',
      target: screen.getByRole('region', { name: 'Card' }),
    });
    await user.keyboard('a');
    expect(document.activeElement?.textContent).toBe('Apagar');
    await user.click(within(screen.getByRole('menu')).getByRole('menuitem', { name: 'Arquivar' }));
    expect(archive).not.toHaveBeenCalled();
    await user.click(document.body);
    expect(screen.queryByRole('menu')).toBeNull();
    expect(remove).not.toHaveBeenCalled();
  });

  it('text fields keep the native menu', async () => {
    const user = userEvent.setup();
    setup();
    await user.pointer({
      keys: '[MouseRight]',
      target: screen.getByRole('textbox', { name: 'Campo' }),
    });
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('TechTable row: right click selects the row and opens its menu', async () => {
    const user = userEvent.setup();
    const open = vi.fn();
    render(
      <TechTable<{ id: string; name: string }>
        label="Itens"
        rows={[
          { id: 'a', name: 'Mercado' },
          { id: 'b', name: 'Aluguel' },
        ]}
        columns={[{ key: 'name', header: 'Nome' }]}
        getRowId={(r) => r.id}
        rowMenuLabel={(r) => `Ações de ${r.name}`}
        rowMenu={(r) => [{ label: `Abrir ${r.name}`, onSelect: () => void open(r.id) }]}
      />,
    );
    const rows = within(screen.getByRole('grid')).getAllByRole('row');
    await user.pointer({ keys: '[MouseRight]', target: within(rows[2]!).getByText('Aluguel') });
    expect(rows[2]!.getAttribute('aria-selected')).toBe('true');
    const menu = screen.getByRole('menu', { name: 'Ações de Aluguel' });
    await user.click(within(menu).getByRole('menuitem', { name: 'Abrir Aluguel' }));
    expect(open).toHaveBeenCalledWith('b');
    expect(document.activeElement).toBe(rows[2]);
    // From the keyboard, starting at the focused row.
    await user.keyboard('{ArrowUp}{Shift>}{F10}{/Shift}');
    expect(screen.getByRole('menu', { name: 'Ações de Mercado' })).toBeTruthy();
    await user.keyboard('{Escape}');
    expect(document.activeElement).toBe(rows[1]);
  });
});
