import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router';
import { beforeEach, describe, expect, it } from 'vitest';

import type { BudgetLineStatus } from '@/api/generated/BudgetLineStatus';

import { BudgetTable } from './BudgetTable';
import type { BudgetRow } from './model';

function line(id: string, name: string, actual: number, parent: string | null = null) {
  return {
    category_id: id,
    category_name: name,
    category_path: name,
    parent_id: parent,
    kind: 'expense',
    planned: 0,
    actual,
    remaining: -actual,
    ratio: null,
    committed: 0,
    has_own_line: false,
    color: null,
    archived: false,
  } satisfies BudgetLineStatus;
}

function row(l: BudgetLineStatus, depth: number, children: BudgetRow[] = []): BudgetRow {
  return { line: l, depth, children, status: 'none', plannedFromChildren: false };
}

const ROWS: BudgetRow[] = [
  row(line('food', 'Alimentação', 50), 0, [
    row(line('market', 'Mercado', 40, 'food'), 1),
    row(line('rest', 'Restaurante', 10, 'food'), 1),
  ]),
  row(line('home', 'Moradia', 80), 0, [
    row(line('rent', 'Aluguel', 70, 'home'), 1),
    row(line('condo', 'Condomínio', 10, 'home'), 1),
  ]),
];

function names() {
  const table = screen.getByRole('table', { name: 'Despesas' });
  return within(table)
    .getAllByRole('row')
    .slice(1)
    .map((r) => r.getAttribute('data-category'));
}

describe('BudgetTable', () => {
  beforeEach(() => localStorage.clear());

  it('ordena entre irmãos: as subcategorias ficam sob o pai', async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter>
        <BudgetTable
          rows={ROWS}
          month="2026-10"
          basis="cash"
          elapsed={0.5}
          colors={{ get: () => undefined, keys: [] }}
          editing={null}
          onEditingChange={() => {}}
          onSave={() => Promise.resolve()}
          savingId={null}
          kind="expense"
          label="Despesas"
        />
      </MemoryRouter>,
    );
    await user.click(screen.getByRole('button', { name: 'Expandir Alimentação' }));
    await user.click(screen.getByRole('button', { name: 'Expandir Moradia' }));
    expect(names()).toEqual(['food', 'market', 'rest', 'home', 'rent', 'condo']);

    // Categoria decrescente: pais e filhos invertidos, cada filho sob o seu pai.
    const sortByName = screen.getByRole('button', { name: /^Categoria/ });
    await user.click(sortByName);
    await user.click(sortByName);
    expect(names()).toEqual(['home', 'condo', 'rent', 'food', 'rest', 'market']);

    // Realizado crescente.
    await user.click(screen.getByRole('button', { name: /^Realizado/ }));
    expect(names()).toEqual(['food', 'rest', 'market', 'home', 'condo', 'rent']);
    expect(screen.getByRole('columnheader', { name: /^Realizado/ }).getAttribute('aria-sort')).toBe(
      'ascending',
    );
  });
});
