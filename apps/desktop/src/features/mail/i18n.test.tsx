/**
 * 04 · Lançamentos › E-mail em inglês e espanhol: a fila, a janela da conta e os rótulos vêm do
 * catálogo `email.ftl` no idioma do app (docs/i18n-design.md).
 */
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it } from 'vitest';

import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { RETENTION_LABEL, STATUS_LABEL } from './labels';

afterEach(async () => {
  await applyLocale('pt-BR');
});

describe('04 · Lançamentos › E-mail em outros idiomas', () => {
  it('fila vazia e janela da conta em inglês', async () => {
    await applyLocale('en');
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos?aba=email');
    expect(await screen.findByRole('region', { name: /Email queue/ })).toBeTruthy();
    await user.click(await screen.findByRole('button', { name: 'Set up email' }));
    const dialog = await screen.findByRole('dialog', { name: 'New email import account' });
    expect(within(dialog).getByLabelText('Mailbox address')).toBeTruthy();
    expect(within(dialog).getByLabelText('IMAP server')).toBeTruthy();
    expect(within(dialog).getByLabelText<HTMLInputElement>('Name').value).toBe('Family mailbox');
    expect(within(dialog).getByRole('button', { name: 'Save and test' })).toBeTruthy();
  });

  it('rótulos em espanhol', async () => {
    await applyLocale('es');
    expect(STATUS_LABEL.awaiting_review).toBe('Por revisar');
    expect(Object.values(RETENTION_LABEL)).toContain('Lo que ocurra primero');
  });
});
