import { QueryClientProvider } from '@tanstack/react-query';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { mockCall, resetMock } from '@/api/mock';
import { createQueryClient } from '@/api/queryClient';

import { SecContactFix } from './SecContactFix';

describe('SEC e-mail on the Health tab', () => {
  it('saves the User-Agent with the e-mail and runs the SEC collection', async () => {
    resetMock('unlocked');
    const user = userEvent.setup();
    render(
      <QueryClientProvider client={createQueryClient()}>
        <SecContactFix />
      </QueryClientProvider>,
    );
    const button = screen.getByRole('button', { name: 'Salvar e baixar' });
    await user.type(screen.getByLabelText('Seu e-mail de contato para a SEC'), 'sem-arroba');
    expect((button as HTMLButtonElement).disabled).toBe(true);
    await user.clear(screen.getByLabelText('Seu e-mail de contato para a SEC'));
    await user.type(screen.getByLabelText('Seu e-mail de contato para a SEC'), 'eu@email.com');
    await user.click(button);
    await waitFor(async () => {
      const v = (await mockCall('research.get_settings', {})) as {
        settings: { sec_user_agent: string };
      };
      expect(v.settings.sec_user_agent).toMatch(/^Treasury\/[\d.]+ eu@email\.com$/);
    });
  });
});
