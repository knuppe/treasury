import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { MOCK_PASSWORD, MOCK_RECOVERY_CODE, mockCall, mockCallLog } from '@/api/mock';
import { MOCK_SHARES } from '@/api/mock/vaultExtras';
import { useUi } from '@/state/ui';
import { renderApp } from '@/test/renderApp';

const STRONG = 'vaca amarela pulou a janela';

/** A barra superior do shell (onde ficam privacidade, tema e cadeado). */
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

describe('fluxo do cofre', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('sem cofre: cria o cofre e entra no app', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('empty');

    // Primeiro a escolha: cofre real ou demonstração.
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

    // A senha não fica guardada na interface (docs/local-security.md, item 13).
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

  it('valida o formulário de criação antes de chamar o núcleo', async () => {
    const user = userEvent.setup();
    renderApp('empty');
    await user.click(await screen.findByRole('radio', { name: /Cofre real/ }));
    await user.click(await screen.findByRole('button', { name: 'Criar cofre cifrado' }));

    expect(screen.getByText('Informe o nome da família.')).toBeTruthy();
    expect(screen.getByText('Informe pelo menos um membro.')).toBeTruthy();
    expect(screen.getByText('Crie uma senha mestra.')).toBeTruthy();
    expect(screen.getByText('Confirme que entendeu o aviso acima.')).toBeTruthy();
    // O foco vai para o primeiro campo com erro.
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

  it('bloqueado: senha errada mostra erro; senha certa abre; o cadeado bloqueia', async () => {
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

    // Um dado qualquer em cache para conferir a limpeza ao bloquear.
    queryClient.setQueryData(['lancamentos', 'lista'], [{ id: 1 }]);

    await user.click(within(topBar()).getByRole('button', { name: 'Bloquear cofre' }));
    await expectLocked();
    expect(queryClient.getQueryData(['lancamentos', 'lista'])).toBeUndefined();
    expect(queryClient.getQueryData(['vault', 'status'])).toMatchObject({ state: 'locked' });
    expect(mockCallLog()).toContain('vault.lock');
  });

  it('kit de recuperação: código errado recusa; certo redefine a senha e abre', async () => {
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

    // A nova senha passa a valer.
    await user.keyboard('{Control>}{Shift>}l{/Shift}{/Control}');
    await expectLocked();
    await user.type(screen.getByLabelText('Senha mestra'), STRONG);
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    await expectShell();
  });

  it('PIN: erro conta tentativas; certo abre o cofre', async () => {
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

  it('partes de recuperação: duas abrem o cofre com senha nova', async () => {
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

  it('bloqueia sozinho por inatividade, avisando no último minuto', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    renderApp('unlocked', { autoLockMinutes: 1 });
    await expectShell();

    await act(() => vi.advanceTimersByTimeAsync(5_000));
    expect(await screen.findByText(/O cofre será bloqueado por inatividade/)).toBeTruthy();

    // Passa do limite de 1 min só um pouco, para o aviso (toast) ainda estar na tela.
    await act(() => vi.advanceTimersByTimeAsync(56_000));
    await expectLocked();
    expect(mockCallLog()).toContain('vault.lock');
    expect(await screen.findByText(/Cofre bloqueado por inatividade/)).toBeTruthy();
  });

  it('atividade adia o bloqueio e avisa o núcleo no máximo 1×/min', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    renderApp('unlocked', { autoLockMinutes: 2 });
    await expectShell();
    const touches = () => mockCallLog().filter((c) => c === 'vault.touch').length;

    // Atividade contínua por 3 minutos: nunca bloqueia.
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

  it('o modo privacidade (Ctrl+Shift+H) esconde os valores e aparece na barra superior', async () => {
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

  it('ao desbloquear, o modo privacidade liga sozinho (e o ajuste desliga isso)', async () => {
    const user = userEvent.setup();
    renderApp('locked');
    await expectLocked();
    await user.type(screen.getByLabelText('Senha mestra'), MOCK_PASSWORD);
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    await expectShell();
    const toggle = within(topBar()).getByRole('button', { name: 'Modo privacidade' });
    expect(toggle.getAttribute('aria-pressed')).toBe('true');

    // Desligado o ajuste, o próximo desbloqueio não esconde os valores.
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

  it('criar um cofre novo não liga o modo privacidade', async () => {
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

  it('a tela de desbloqueio tem o mesmo seletor de temas da barra superior', async () => {
    const user = userEvent.setup();
    renderApp('locked');
    await user.click(await screen.findByRole('button', { name: /Escolher tema/ }));
    const menu = await screen.findByRole('menu', { name: 'Temas' });
    // O menu flutua no body (acima de qualquer card).
    expect(menu.parentElement).toBe(document.body);
    expect(screen.getAllByRole('menuitemradio').length).toBeGreaterThan(5);
  });
});
