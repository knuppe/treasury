/** Características no explorador: contagem facetada pelo recorte na tela (dados inventados). */
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { AssetTag } from '@/api/generated/AssetTag';

import { TagPicker } from './TagPicker';

const tag = (key: string, label: string, count: number): AssetTag => ({
  key,
  label,
  group: 'solidez',
  group_label: 'Solidez',
  rule: `Regra de ${label}`,
  classes: ['stock_br'],
  count,
  version: '0.1.0',
});

const TAGS = [
  tag('roe_alto', 'ROE alto', 113),
  tag('divida_baixa', 'Dívida baixa', 90),
  tag('dy_alto', 'DY alto', 70),
  tag('nunca_prejuizo', 'Nunca deu prejuízo', 40),
];

const chip = (label: string) => screen.getByRole('button', { name: new RegExp(`^${label} ·`) });

describe('TagPicker', () => {
  it('sem resultado, conta o universo; todas habilitadas', () => {
    render(<TagPicker tags={TAGS} value={[]} onChange={() => {}} />);
    expect(chip('ROE alto').textContent).toBe('ROE alto · 113');
    expect(TAGS.every((x) => !(chip(x.label) as HTMLButtonElement).disabled)).toBe(true);
  });

  it('com duas marcadas, conta o recorte; 0 desabilita sem sumir; a marcada segue habilitada', async () => {
    const onChange = vi.fn();
    render(
      <TagPicker
        tags={TAGS}
        value={['roe_alto', 'divida_baixa']}
        // As marcadas dão o total do recorte (25); das 25, 12 com DY alto e nenhuma sem prejuízo.
        counts={{ roe_alto: 25, divida_baixa: 25, dy_alto: 12 }}
        onChange={onChange}
      />,
    );
    expect(chip('ROE alto').textContent).toBe('ROE alto · 25');
    expect(chip('Dívida baixa').textContent).toBe('Dívida baixa · 25');
    expect(chip('DY alto').textContent).toBe('DY alto · 12');
    const empty = chip('Nunca deu prejuízo') as HTMLButtonElement;
    expect(empty.textContent).toBe('Nunca deu prejuízo · 0');
    expect(empty.disabled).toBe(true);
    expect(empty.title).toMatch(/Nenhum papel do recorte/);
    expect(empty.title).toContain('Regra de Nunca deu prejuízo');
    // Marcada com 0 no recorte (ex.: o filtro mudou): continua habilitada para desmarcar.
    render(
      <TagPicker tags={[TAGS[3]!]} value={['nunca_prejuizo']} counts={{}} onChange={onChange} />,
    );
    const pressed = screen
      .getAllByRole('button', { name: /^Nunca deu prejuízo ·/ })
      .find((b) => b.getAttribute('aria-pressed') === 'true') as HTMLButtonElement;
    expect(pressed.disabled).toBe(false);
    await userEvent.setup().click(pressed);
    expect(onChange).toHaveBeenCalledWith([]);
  });
});
