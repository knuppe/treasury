/**
 * Ajustes › Atualizações contra o mock: a seção própria, o modo (baixar sozinho, só avisar,
 * desligado) gravado fora do cofre e nenhuma procura fora do app.
 */
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(() => {
  vi.restoreAllMocks();
});

describe('Atualizações', () => {
  it('baixam sozinhas por padrão, podem só avisar ou desligar, sem procura fora do app', async () => {
    const user = userEvent.setup();
    const fetchSpy = vi.spyOn(globalThis, 'fetch');
    renderApp('unlocked', {}, '#/ajustes?secao=atualizacoes');
    const group = await screen.findByRole('radiogroup', {
      name: 'O que fazer com uma versão nova',
    });
    const download = screen.getByRole('radio', { name: /Baixar sozinho e avisar/ });
    await waitFor(() => expect(download.getAttribute('aria-checked')).toBe('true'));
    expect(group).toBeTruthy();
    expect(screen.getByText(/nada sobre você ou o cofre vai junto/)).toBeTruthy();
    expect(screen.getByText(/começa quando o Treasury for publicado no GitHub/)).toBeTruthy();
    expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Procurar agora' }).disabled).toBe(
      true,
    );

    const notify = screen.getByRole('radio', { name: /Só avisar/ });
    await user.click(notify);
    await waitFor(() => expect(notify.getAttribute('aria-checked')).toBe('true'));
    expect(await mockCall('system.update_settings', {})).toEqual({ mode: 'notify' });

    const off = screen.getByRole('radio', { name: /Desligado/ });
    await user.click(off);
    await waitFor(() => expect(off.getAttribute('aria-checked')).toBe('true'));
    expect(await mockCall('system.update_settings', {})).toEqual({ mode: 'off' });
    expect(screen.getByText(/inclusive correções de segurança/)).toBeTruthy();
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it('não ficam mais em Sobre e apoio', async () => {
    renderApp('unlocked', {}, '#/ajustes?secao=sobre');
    expect(await screen.findByText(/Treasury é (gratuito|software livre)/)).toBeTruthy();
    expect(
      screen.queryByRole('radiogroup', { name: 'O que fazer com uma versão nova' }),
    ).toBeNull();
  });
});
