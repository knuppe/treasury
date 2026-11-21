import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { renderApp } from '@/test/renderApp';

describe('Ajustes · Banco de dados', () => {
  it('shows size and largest tables, runs maintenance and purges with a preview', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=banco');
    expect(await screen.findByText('V96')).toBeTruthy();
    const tables = await screen.findByRole('grid', { name: 'Maiores tabelas do banco' });
    expect(within(tables).getByText('fundamental')).toBeTruthy();

    const maint = screen.getByRole('region', { name: 'Manutenção' });
    await user.click(within(maint).getAllByRole('button', { name: 'Rodar' })[0]!);
    expect(await within(maint).findByText(/Verificar integridade: tudo certo/)).toBeTruthy();

    const purge = screen.getByRole('region', { name: /Expurgo de dados antigos/ });
    const row = within(purge).getByText('Auditoria do agente e do MCP').closest('li')!;
    await user.click(within(row as HTMLElement).getByRole('button', { name: 'Ver prévia' }));
    expect(await within(row as HTMLElement).findByText(/registro\(s\), cerca de/)).toBeTruthy();
    await user.click(within(row as HTMLElement).getByRole('button', { name: 'Apagar' }));
    const dialog = await screen.findByRole('dialog', { name: /Apagar: Auditoria/ });
    await user.click(within(dialog).getByRole('button', { name: 'Fazer backup e apagar' }));
    expect(await screen.findByText(/registro\(s\) apagado\(s\)/)).toBeTruthy();
    await waitFor(() => expect(screen.queryByRole('dialog', { name: /Apagar:/ })).toBeNull());
  });

  it('double click on a table shows the first rows, with Voltar', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=banco');
    const tables = await screen.findByRole('grid', { name: 'Maiores tabelas do banco' });
    await user.dblClick(await within(tables).findByText('txn'));
    const grid = await screen.findByRole('grid', { name: 'Primeiras linhas de txn' });
    expect(within(grid).getAllByRole('row').length).toBeGreaterThan(1);
    expect(screen.getByText(/só leitura/)).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(await screen.findByRole('grid', { name: 'Maiores tabelas do banco' })).toBeTruthy();
  });
});
