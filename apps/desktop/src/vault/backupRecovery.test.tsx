/**
 * Recovering from a backup on the unlock screen (docs/backup-recovery.md): a vault that does not
 * open with the right password leads to recovery, which lists the backups in the folder and restores one
 * with the vault closed; the vault that did not open is kept alongside.
 */
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { MOCK_PASSWORD, mockCall, seedVaultDamaged } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

describe('recovering from a backup', () => {
  it('damaged vault: warns, lists the backups and restores with the vault closed', async () => {
    const user = userEvent.setup();
    renderApp('locked');
    seedVaultDamaged();
    await user.type(await screen.findByLabelText('Senha mestra'), MOCK_PASSWORD);
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    expect(await screen.findByText(/Essa senha abre os seus backups/)).toBeTruthy();
    await user.click(screen.getAllByRole('button', { name: 'Recuperar de um backup' })[0]!);

    const list = await screen.findByRole('radiogroup', { name: 'Backups encontrados' });
    const options = within(list).getAllByRole('radio');
    expect(options.length).toBeGreaterThan(3);
    // The newest comes checked.
    expect((options[0] as HTMLInputElement).checked).toBe(true);

    await user.type(screen.getByLabelText('Senha mestra'), MOCK_PASSWORD);
    await user.click(screen.getByRole('button', { name: 'Restaurar este backup' }));
    expect(await screen.findByRole('navigation', { name: 'Telas' })).toBeTruthy();
    expect(await screen.findByText(/ficou guardado em .*danificado/)).toBeTruthy();
  });

  it('the link opens recovery even without an error, and the wrong password is flagged', async () => {
    const user = userEvent.setup();
    renderApp('locked');
    await user.click(await screen.findByRole('button', { name: 'Recuperar de um backup' }));
    await screen.findByRole('radiogroup', { name: 'Backups encontrados' });
    await user.type(screen.getByLabelText('Senha mestra'), 'outra senha qualquer');
    await user.click(screen.getByRole('button', { name: 'Restaurar este backup' }));
    expect(await screen.findByText('Senha incorreta.')).toBeTruthy();
    // Going back leads to the usual password.
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(await screen.findByRole('button', { name: 'Desbloquear' })).toBeTruthy();
  });

  it('bringing in an outside backup puts it in the list as imported', async () => {
    const r = (await mockCall('backup.import_file', {
      source: '/Volumes/PENDRIVE/treasury-2026-09-01T03-00-00.000Z-auto_daily.db.enc',
    })) as { imported: boolean };
    expect(r.imported).toBe(true);
    const files = (await mockCall('backup.scan', {})) as { imported: boolean }[];
    expect(files.some((f) => f.imported)).toBe(true);
  });
});
