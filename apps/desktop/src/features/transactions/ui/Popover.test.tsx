import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { Dialog } from '@/design/components';

import { Popover } from './Popover';

describe('Popover', () => {
  it('does not close when WebKit sends the focus to <main> on clicking inside the panel', () => {
    render(
      <main tabIndex={-1} data-testid="main">
        <Popover label="Período">
          <button type="button">Mês passado</button>
        </Popover>
      </main>,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Período' }));
    const option = screen.getByRole('button', { name: 'Mês passado' });
    // As in Safari/WKWebView: the button does not receive focus; it goes up to the focusable ancestor.
    fireEvent.mouseDown(option);
    fireEvent.blur(option, { relatedTarget: screen.getByTestId('main') });
    fireEvent.mouseUp(option);
    expect(screen.getByRole('dialog')).toBeTruthy();
    // Tab out (without a click) still closes.
    const outside = document.createElement('button');
    document.body.appendChild(outside);
    fireEvent.blur(option, { relatedTarget: outside });
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('inside a dialog, the floating panel accepts focus, typing and click', async () => {
    const user = userEvent.setup();
    let picked = '';
    render(
      <Dialog open onOpenChange={() => {}} title="Importar">
        <Popover label="Categoria" variant="bare">
          {(close) => (
            <div>
              <input aria-label="Buscar" />
              <button
                type="button"
                onClick={() => {
                  picked = 'Internet';
                  close();
                }}
              >
                Internet
              </button>
            </div>
          )}
        </Popover>
      </Dialog>,
    );
    await user.click(screen.getByRole('button', { name: 'Categoria' }));
    const search = screen.getByRole('textbox', { name: 'Buscar' });
    await waitFor(() => expect(document.activeElement).toBe(search));
    await user.keyboard('inter');
    expect((search as HTMLInputElement).value).toBe('inter');
    expect(
      screen.getByRole('button', { name: 'Internet' }).closest('[style]')?.getAttribute('style'),
    ).toContain('pointer-events: auto');
    await user.click(screen.getByRole('button', { name: 'Internet' }));
    expect(picked).toBe('Internet');
    expect(screen.getByRole('dialog', { name: /Importar/ })).toBeTruthy();
  });
});
