import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { describe, expect, it } from 'vitest';

import { labelPointsElsewhere, linkHost, Markdown } from './Markdown';

describe('<Markdown>', () => {
  it('links https saem do app (navegador do sistema ou aba nova); caminhos do app navegam nele', () => {
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

describe('links das respostas: domínio à vista antes de abrir', () => {
  it('mostra o domínio de verdade ao lado do link externo', () => {
    render(<Markdown text="Veja [o extrato](https://www.itau.com.br/extrato) no banco." />);
    const host = document.querySelector('[data-link-host]')!;
    expect(host.textContent).toBe('↗ itau.com.br');
  });

  it('alerta quando o texto parece outro endereço', () => {
    render(<Markdown text="Acesse [itau.com.br](https://itau-seguro.example.net/login)." />);
    const host = document.querySelector('[data-link-host]')!;
    expect(host.textContent).toBe('⚠ abre itau-seguro.example.net');
    expect(host.getAttribute('title')).toContain('não é o endereço de verdade');
  });

  it('subdomínio do mesmo site e links internos não alertam', () => {
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
