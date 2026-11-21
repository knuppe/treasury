import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { MOCK_PASSWORD, MOCK_RECOVERY_CODE, mockCall, mockCallLog } from '@/api/mock';
import { MOCK_SHARES } from '@/api/mock/vaultExtras';
import { useUi } from '@/state/ui';
import { renderApp } from '@/test/renderApp';

const STRONG = 'vaca amarela pulou a janela';

/** The shell top bar (where privacy, theme and the lock sit). */
function topBar(): HTMLElement {
  return screen.getByRole('button', { name: /Buscar ou executar/ }).closest('header')!;
}

async function expectShell() {
  expect(await screen.findByRole('navigation', { name: 'Telas' })).toBeTruthy();
}

async function expectLocked() {
  expect(await screen.findByRole('button', { name: 'Desbloquear' })).toBeTruthy();
  expect(screen.queryByRole('navigation', { name: 'Telas' })).toBeNull();
}

describe('vault flow', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('no vault: creates the vault and enters the app', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('empty');

    // First the choice: real vault or demo.
    await user.click(await screen.findByRole('radio', { name: /Cofre real/ }));
    expect(await screen.findByRole('button', { name: 'Criar cofre cifrado' })).toBeTruthy();
    expect(screen.getByText(/não há como recuperar os dados/)).toBeTruthy();

    await user.type(screen.getByLabelText('Nome da família'), 'Família Silva');
    await user.type(screen.getByLabelText('Membro 1'), 'Ana');
    await user.click(screen.getByRole('button', { name: 'Adicionar membro' }));
    await user.type(screen.getByLabelText('Membro 2'), 'Bruno');
    await user.type(screen.getByLabelText('Senha mestra'), STRONG);
    await user.type(screen.getByLabelText('Confirmar senha'), STRONG);
    await user.click(screen.getByRole('checkbox', { name: /perco os dados/ }));
    await user.click(screen.getByRole('button', { name: 'Criar cofre cifrado' }));

    await expectShell();
    expect(screen.getAllByText('Família Silva').length).toBeGreaterThan(0);
    expect(await screen.findByRole('heading', { level: 1, name: /Painel/ })).toBeTruthy();
    expect(mockCallLog()).toContain('vault.create');

    // The password is not kept in the interface (docs/local-security.md, item 13).
    const stored = [localStorage, sessionStorage]
      .flatMap((st) => Object.keys(st).map((k) => `${k}=${st.getItem(k) ?? ''}`))
      .join('\n');
    expect(stored).not.toContain(STRONG);
    await waitFor(() =>
      expect(
        queryClient
          .getMutationCache()
          .getAll()
          .some((m) => JSON.stringify(m.state.variables ?? null).includes(STRONG)),
      ).toBe(false),
    );
  });

  it('validates the creation form before calling the core', async () => {
    const user = userEvent.setup();
    renderApp('empty');
    await user.click(await screen.findByRole('radio', { name: /Cofre real/ }));
    await user.click(await screen.findByRole('button', { name: 'Criar cofre cifrado' }));

    expect(screen.getByText('Informe o nome da família.')).toBeTruthy();
    expect(screen.getByText('Informe pelo menos um membro.')).toBeTruthy();
    expect(screen.getByText('Crie uma senha mestra.')).toBeTruthy();
    expect(screen.getByText('Confirme que entendeu o aviso acima.')).toBeTruthy();
    // The focus goes to the first field with an error.
    expect(document.activeElement).toBe(screen.getByLabelText('Nome da família'));

    await user.type(screen.getByLabelText('Senha mestra'), 'senha123');
    await user.type(screen.getByLabelText('Confirmar senha'), 'senha124');
    expect(screen.getByText('Use pelo menos 10 caracteres.')).toBeTruthy();
    expect(screen.getByText('As senhas não conferem.')).toBeTruthy();
    expect(
      screen.getByRole('meter', { name: 'Força da senha' }).getAttribute('aria-valuenow'),
    ).toBe('0');
    expect(mockCallLog()).not.toContain('vault.create');
  });

  it('locked: wrong password shows an error; right password opens; the lock locks', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('locked', { householdName: 'Família Exemplo' });

    await expectLocked();
    expect(screen.getByRole('heading', { level: 1, name: 'Família Exemplo' })).toBeTruthy();

    const password = screen.getByLabelText<HTMLInputElement>('Senha mestra');
    await user.type(password, 'senha errada');
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    expect(await screen.findByText('Senha incorreta.')).toBeTruthy();
    expect(password.value).toBe('');
    expect(password.getAttribute('aria-invalid')).toBe('true');

    await user.type(password, MOCK_PASSWORD);
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    await expectShell();

    // Some cached data to check the cleanup on lock.
    queryClient.setQueryData(['lancamentos', 'lista'], [{ id: 1 }]);

    await user.click(within(topBar()).getByRole('button', { name: 'Bloquear cofre' }));
    await expectLocked();
    expect(queryClient.getQueryData(['lancamentos', 'lista'])).toBeUndefined();
    expect(queryClient.getQueryData(['vault', 'status'])).toMatchObject({ state: 'locked' });
    expect(mockCallLog()).toContain('vault.lock');
  });

  it('recovery kit: wrong code is refused; right one resets the password and opens', async () => {
    const user = userEvent.setup();
    renderApp('locked');

    await user.click(await screen.findByRole('button', { name: 'Recuperar o cofre' }));
    await user.type(screen.getByLabelText('Código de recuperação'), 'AAAA-BBBB');
    await user.type(screen.getByLabelText('Nova senha mestra'), STRONG);
    await user.type(screen.getByLabelText('Confirmar nova senha'), STRONG);
    await user.click(screen.getByRole('button', { name: 'Redefinir senha e abrir' }));
    expect(await screen.findByText('Código de recuperação inválido.')).toBeTruthy();

    const code = screen.getByLabelText('Código de recuperação');
    await user.clear(code);
    await user.type(code, MOCK_RECOVERY_CODE.toLowerCase());
    await user.click(screen.getByRole('button', { name: 'Redefinir senha e abrir' }));
    await expectShell();

    // The new password takes effect.
    await user.keyboard('{Control>}{Shift>}l{/Shift}{/Control}');
    await expectLocked();
    await user.type(screen.getByLabelText('Senha mestra'), STRONG);
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    await expectShell();
  });

  it('PIN: an error counts attempts; the right one opens the vault', async () => {
    const user = userEvent.setup();
    renderApp('unlocked');
    await expectShell();
    await mockCall('vault.pin_enable', { password: MOCK_PASSWORD, pin: '730418' });
    await user.keyboard('{Control>}{Shift>}l{/Shift}{/Control}');
    await expectLocked();
    await user.click(await screen.findByRole('button', { name: 'Usar PIN' }));
    const pin = screen.getByLabelText('PIN deste computador');
    await user.type(pin, '111222');
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    expect(await screen.findByText(/Restam 4 tentativa/)).toBeTruthy();
    await user.type(screen.getByLabelText('PIN deste computador'), '730418');
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    await expectShell();
  });

  it('recovery parts: two open the vault with a new password', async () => {
    const user = userEvent.setup();
    renderApp('locked');
    await user.click(await screen.findByRole('button', { name: 'Recuperar o cofre' }));
    await user.click(screen.getByRole('radio', { name: 'Partes' }));
    await user.type(screen.getByLabelText('Primeira parte'), MOCK_SHARES[2]!);
    await user.type(screen.getByLabelText('Segunda parte'), MOCK_SHARES[0]!);
    await user.type(screen.getByLabelText('Nova senha mestra'), STRONG);
    await user.type(screen.getByLabelText('Confirmar nova senha'), STRONG);
    await user.click(screen.getByRole('button', { name: 'Redefinir senha e abrir' }));
    await expectShell();
    expect(mockCallLog()).toContain('vault.unlock_shares');
  });

  it('locks by itself on inactivity, warning in the last minute', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    renderApp('unlocked', { autoLockMinutes: 1 });
    await expectShell();

    await act(() => vi.advanceTimersByTimeAsync(5_000));
    expect(await screen.findByText(/O cofre será bloqueado por inatividade/)).toBeTruthy();

    // Goes just past the 1 min limit, so the notice (toast) is still on screen.
    await act(() => vi.advanceTimersByTimeAsync(56_000));
    await expectLocked();
    expect(mockCallLog()).toContain('vault.lock');
    expect(await screen.findByText(/Cofre bloqueado por inatividade/)).toBeTruthy();
  });

  it('activity postpones the lock and notifies the core at most 1×/min', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    renderApp('unlocked', { autoLockMinutes: 2 });
    await expectShell();
    const touches = () => mockCallLog().filter((c) => c === 'vault.touch').length;

    // Continuous activity for 3 minutes: never locks.
    for (let i = 0; i < 18; i++) {
      await act(() => vi.advanceTimersByTimeAsync(10_000));
      act(() => {
        window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Shift' }));
      });
    }
    expect(screen.queryByRole('button', { name: 'Desbloquear' })).toBeNull();
    await waitFor(() => expect(touches()).toBeGreaterThanOrEqual(2));
    expect(touches()).toBeLessThanOrEqual(3);
  });

  it('privacy mode (Ctrl+Shift+H) hides the values and shows in the top bar', async () => {
    const user = userEvent.setup();
    renderApp('unlocked');
    await expectShell();
    const toggle = within(topBar()).getByRole('button', { name: 'Modo privacidade' });
    expect(toggle.getAttribute('aria-pressed')).toBe('false');

    await user.keyboard('{Control>}{Shift>}h{/Shift}{/Control}');
    expect(toggle.getAttribute('aria-pressed')).toBe('true');
    expect(within(topBar()).getByText('Privacidade')).toBeTruthy();
    const saved = JSON.parse(window.localStorage.getItem('treasury.ui')!) as {
      state: { privacy: boolean };
    };
    expect(saved.state.privacy).toBe(true);

    await user.click(toggle);
    expect(toggle.getAttribute('aria-pressed')).toBe('false');
  });

  it('on unlock, privacy mode turns on by itself (and the setting turns that off)', async () => {
    const user = userEvent.setup();
    renderApp('locked');
    await expectLocked();
    await user.type(screen.getByLabelText('Senha mestra'), MOCK_PASSWORD);
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    await expectShell();
    const toggle = within(topBar()).getByRole('button', { name: 'Modo privacidade' });
    expect(toggle.getAttribute('aria-pressed')).toBe('true');

    // With the setting off, the next unlock does not hide the values.
    await user.click(toggle);
    useUi.getState().setPrivacyOnUnlock(false);
    await user.click(within(topBar()).getByRole('button', { name: 'Bloquear cofre' }));
    await expectLocked();
    await user.type(screen.getByLabelText('Senha mestra'), MOCK_PASSWORD);
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    await expectShell();
    expect(
      within(topBar())
        .getByRole('button', { name: 'Modo privacidade' })
        .getAttribute('aria-pressed'),
    ).toBe('false');
  });

  it('creating a new vault does not turn on privacy mode', async () => {
    const user = userEvent.setup();
    renderApp('empty');
    await user.click(await screen.findByRole('radio', { name: /Cofre real/ }));
    await user.type(await screen.findByLabelText('Nome da família'), 'Família Silva');
    await user.type(screen.getByLabelText('Membro 1'), 'Ana');
    await user.type(screen.getByLabelText('Senha mestra'), STRONG);
    await user.type(screen.getByLabelText('Confirmar senha'), STRONG);
    await user.click(screen.getByRole('checkbox', { name: /perco os dados/ }));
    await user.click(screen.getByRole('button', { name: 'Criar cofre cifrado' }));
    await expectShell();
    expect(useUi.getState().privacy).toBe(false);
  });

  it('the unlock screen has the same theme picker as the top bar', async () => {
    const user = userEvent.setup();
    renderApp('locked');
    await user.click(await screen.findByRole('button', { name: /Escolher tema/ }));
    const menu = await screen.findByRole('menu', { name: 'Temas' });
    // The menu floats on the body (above any card).
    expect(menu.parentElement).toBe(document.body);
    expect(screen.getAllByRole('menuitemradio').length).toBeGreaterThan(5);
  });
});
