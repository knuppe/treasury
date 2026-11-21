import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';

import { decimal, PreviewTable, TD, type PreviewColumn } from './shared';

interface Row {
  id: number;
  ticker: string;
  quantity: string;
}

const ROWS: Row[] = [
  { id: 1, ticker: 'VALE3', quantity: '1.5' },
  { id: 2, ticker: 'PETR4', quantity: '10' },
  { id: 3, ticker: 'WEGE3', quantity: '1.25' },
];

const COLUMNS: PreviewColumn<Row>[] = [
  { key: 'ticker', header: 'Ativo', hideable: false, cell: (r) => r.ticker },
  {
    key: 'quantity',
    header: 'Qtd.',
    numeric: true,
    sortValue: (r) => decimal(r.quantity),
    cell: (r) => r.quantity,
  },
  { key: 'note', header: 'Nota', sortable: false, cell: () => '—' },
];

function tickers() {
  const table = screen.getByRole('table', { name: 'Operações' });
  return within(table)
    .getAllByRole('row')
    .slice(1)
    .filter((r) => !r.querySelector('[data-editor]'))
    .map((r) => r.querySelector('td')?.textContent);
}

function renderTable(editing: number | null = null) {
  return render(
    <PreviewTable
      label="Operações"
      rows={ROWS}
      getRowId={(r) => r.id}
      columns={COLUMNS}
      after={(r, colSpan) =>
        r.id === editing ? (
          <tr>
            <td className={TD} colSpan={colSpan} data-editor>
              Corrigir {r.ticker}
            </td>
          </tr>
        ) : null
      }
    />,
  );
}

describe('PreviewTable', () => {
  beforeEach(() => localStorage.clear());

  it('ordena pela coluna (decimais como número) e o editor segue a linha', async () => {
    const user = userEvent.setup();
    renderTable(1);
    await user.click(screen.getByRole('button', { name: /Qtd\./ }));
    expect(tickers()).toEqual(['WEGE3', 'VALE3', 'PETR4']);
    // A linha extra fica logo abaixo da sua linha e ocupa a largura toda.
    const editor = screen.getByText('Corrigir VALE3');
    expect(editor.closest('tr')?.previousElementSibling?.textContent).toContain('VALE3');
    expect(editor.getAttribute('colspan')).toBe('4');
    await user.click(screen.getByRole('button', { name: /Qtd\./ }));
    expect(tickers()).toEqual(['PETR4', 'VALE3', 'WEGE3']);
    expect(screen.getByRole('columnheader', { name: /Qtd\./ }).getAttribute('aria-sort')).toBe(
      'descending',
    );
  });

  it('esconde uma coluna pelo seletor e lembra a escolha', async () => {
    const user = userEvent.setup();
    const { unmount } = renderTable();
    await user.click(screen.getByRole('button', { name: 'Colunas de Operações' }));
    const group = screen.getByRole('group', { name: 'Colunas visíveis' });
    // "Ativo" identifica a linha: não aparece para esconder.
    expect(within(group).queryByRole('checkbox', { name: 'Ativo' })).toBeNull();
    await user.click(within(group).getByRole('checkbox', { name: 'Qtd.' }));
    expect(screen.queryByRole('columnheader', { name: /Qtd\./ })).toBeNull();
    unmount();
    renderTable();
    expect(screen.queryByRole('columnheader', { name: /Qtd\./ })).toBeNull();
    expect(screen.getByRole('columnheader', { name: /Ativo/ })).toBeTruthy();
    expect(screen.getByRole('columnheader', { name: /Nota/ })).toBeTruthy();
  });
});
