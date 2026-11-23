import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { Rule } from '@/api/generated/Rule';
import type { TxnPage } from '@/api/generated/TxnPage';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

async function mercpago() {
  return (await mockCall('transactions.query', {
    text: 'mercpago',
    date_from: '2000-01-01',
    limit: 200,
  })) as TxnPage;
}

describe('categorization rules', () => {
  it('creates a rule, tests, simulates and applies it to uncategorized transactions, with undo', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos?aba=regras');
    await screen.findByRole('grid', { name: 'Regras' });
    const before = await mercpago();
    expect(before.items.every((t) => t.category_id === null)).toBe(true);

    await user.click(screen.getByRole('button', { name: 'Nova regra' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova regra' });
    await user.type(within(dialog).getByLabelText('Nome'), 'Pagamentos Mercado Pago');
    await user.type(within(dialog).getByLabelText('Texto da condição 1'), 'mercpago');
    await user.click(within(dialog).getByRole('button', { name: /^Categoria: Não mudar/ }));
    await user.type(
      within(dialog).getByRole('combobox', { name: 'Buscar categoria' }),
      'casa e dec',
    );
    await user.keyboard('{Enter}');
    await user.click(within(dialog).getByRole('button', { name: 'Testar' }));
    expect(
      await within(dialog).findByText(
        before.total <= 1
          ? `${before.total} lançamento casa com esta regra`
          : `${before.total} lançamentos casam com esta regra`,
      ),
    ).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Criar regra' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Nova regra' })).toBeNull());
    expect(await screen.findByText('Pagamentos Mercado Pago')).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Aplicar regras…' }));
    const apply = await screen.findByRole('dialog', { name: 'Aplicar regras' });
    await user.click(within(apply).getByRole('button', { name: 'Ver prévia' }));
    expect(await within(apply).findByText(/Prévia: \d+ casaram · \d+ mudam/)).toBeTruthy();
    // Nothing saved yet.
    expect((await mercpago()).items.every((t) => t.category_id === null)).toBe(true);
    await user.click(within(apply).getByRole('button', { name: /^Aplicar a \d+/ }));
    expect(await within(apply).findByText(/Aplicado:/)).toBeTruthy();
    const after = await mercpago();
    expect(after.items.every((t) => t.category_path === 'Compras › Casa e decoração')).toBe(true);
    const rules = (await mockCall('rules.list', {})) as Rule[];
    expect(rules.find((r) => r.name === 'Pagamentos Mercado Pago')!.hits).toBe(before.total);

    await user.click(within(apply).getByRole('button', { name: 'Desfazer' }));
    await waitFor(async () =>
      expect((await mercpago()).items.every((t) => t.category_id === null)).toBe(true),
    );
  });

  it('suggests a rule from a transaction', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos?q=drogasil&periodo=tudo');
    const table = await screen.findByRole('grid', { name: 'Lançamentos' });
    const row = (await within(table).findAllByText('Drogasil'))[0]!.closest('tr')!;
    await user.dblClick(row);
    const dialog = await screen.findByRole('dialog', { name: 'Editar lançamento' });
    await user.click(within(dialog).getByRole('button', { name: 'Criar regra a partir deste' }));
    const rule = await screen.findByRole('dialog', { name: 'Nova regra' });
    expect(within(rule).getByLabelText<HTMLInputElement>('Nome').value).toMatch(
      /^Descrição contém “DROGASIL/,
    );
    expect(within(rule).getByLabelText<HTMLInputElement>('Texto da condição 1').value).toMatch(
      /^drogasil/,
    );
    expect(
      await within(rule).findByRole('button', { name: /^Categoria: (Saúde › Farmácia|Não mudar)/ }),
    ).toBeTruthy();
    expect(window.location.hash).toContain('aba=regras');
  });
});
