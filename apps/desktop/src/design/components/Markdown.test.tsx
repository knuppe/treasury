import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { describe, expect, it } from 'vitest';

import { labelPointsElsewhere, linkHost, Markdown } from './Markdown';

describe('<Markdown>', () => {
  it('https links leave the app (system browser or new tab); app paths navigate inside it', () => {
    render(
      <MemoryRouter>
        <Markdown text="Veja [o extrato](https://www.itau.com.br/extrato) e [a fila](/agente?aba=aprovacao)." />
      </MemoryRouter>,
    );
    const external = screen.getByRole('link', { name: 'o extrato' });
    expect(external.getAttribute('href')).toBe('https://www.itau.com.br/extrato');
    expect(external.getAttribute('target')).toBe('_blank');
    expect(external.getAttribute('rel')).toContain('noopener');
    const internal = screen.getByRole('link', { name: 'a fila' });
    expect(internal.getAttribute('href')).toBe('/agente?aba=aprovacao');
    expect(internal.getAttribute('target')).toBeNull();
  });
});

describe('links in replies: domain in view before opening', () => {
  it('shows the real domain next to the external link', () => {
    render(<Markdown text="Veja [o extrato](https://www.itau.com.br/extrato) no banco." />);
    const host = document.querySelector('[data-link-host]')!;
    expect(host.textContent).toBe('↗ itau.com.br');
  });

  it('warns when the text looks like another address', () => {
    render(<Markdown text="Acesse [itau.com.br](https://itau-seguro.example.net/login)." />);
    const host = document.querySelector('[data-link-host]')!;
    expect(host.textContent).toBe('⚠ abre itau-seguro.example.net');
    expect(host.getAttribute('title')).toContain('não é o endereço de verdade');
  });

  it('subdomain of the same site and internal links do not warn', () => {
    expect(labelPointsElsewhere('itau.com.br', 'internet.itau.com.br')).toBe(false);
    expect(labelPointsElsewhere('o extrato', 'itau.com.br')).toBe(false);
    expect(linkHost('mailto:a@b.com')).toBeNull();
    render(
      <MemoryRouter>
        <Markdown text="[a fila](/agente?aba=aprovacao)" />
      </MemoryRouter>,
    );
    expect(document.querySelector('[data-link-host]')).toBeNull();
  });
});
