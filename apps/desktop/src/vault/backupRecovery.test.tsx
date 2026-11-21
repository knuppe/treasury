/**
 * Recuperar de um backup na tela de desbloqueio (docs/backup-recovery.md): o cofre que não
 * abre com a senha certa leva à recuperação, que lista os backups da pasta e restaura um deles
 * com o cofre fechado; o cofre que não abria fica ao lado.
 */
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { MOCK_PASSWORD, mockCall, seedVaultDamaged } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

describe('recuperar de um backup', () => {
  it('cofre danificado: avisa, lista os backups e restaura com o cofre fechado', async () => {
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
    // O mais novo vem marcado.
    expect((options[0] as HTMLInputElement).checked).toBe(true);

    await user.type(screen.getByLabelText('Senha mestra'), MOCK_PASSWORD);
    await user.click(screen.getByRole('button', { name: 'Restaurar este backup' }));
    expect(await screen.findByRole('navigation', { name: 'Telas' })).toBeTruthy();
    expect(await screen.findByText(/ficou guardado em .*danificado/)).toBeTruthy();
  });

  it('o link abre a recuperação mesmo sem erro, e a senha errada é avisada', async () => {
    const user = userEvent.setup();
    renderApp('locked');
    await user.click(await screen.findByRole('button', { name: 'Recuperar de um backup' }));
    await screen.findByRole('radiogroup', { name: 'Backups encontrados' });
    await user.type(screen.getByLabelText('Senha mestra'), 'outra senha qualquer');
    await user.click(screen.getByRole('button', { name: 'Restaurar este backup' }));
    expect(await screen.findByText('Senha incorreta.')).toBeTruthy();
    // Voltar leva à senha de sempre.
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(await screen.findByRole('button', { name: 'Desbloquear' })).toBeTruthy();
  });

  it('trazer um backup de fora o põe na lista como importado', async () => {
    const r = (await mockCall('backup.import_file', {
      source: '/Volumes/PENDRIVE/treasury-2026-09-01T03-00-00.000Z-auto_daily.db.enc',
    })) as { imported: boolean };
    expect(r.imported).toBe(true);
    const files = (await mockCall('backup.scan', {})) as { imported: boolean }[];
    expect(files.some((f) => f.imported)).toBe(true);
  });
});
