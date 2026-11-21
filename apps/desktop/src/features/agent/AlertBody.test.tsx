/**
 * Alert body in Markdown (month close): a heading glued to the previous text goes back to
 * its own line, and headings and tables render drawn, with no `###` or `|---|` showing.
 */
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Markdown } from '@/design/components';

import { tidyAlertBody } from './AlertsPanel';

const BODY = [
  'Vou somar os números do mês.## Fechamento de outubro de 2026',
  '',
  '### Resultado',
  '',
  '| Item | Valor |',
  '|---|---:|',
  '| Receitas | R$ 10.000,00 |',
  '| Despesas | R$ 8.000,00 |',
  '',
  '**Taxa de poupança:** 20%',
].join('\n');

describe('alerts › body in Markdown', () => {
  it('splits the glued heading and keeps the rest', () => {
    expect(tidyAlertBody('fim.## Fechamento')).toBe('fim.\n\n## Fechamento');
    for (const ok of ['Linguagem C# é citada.', '| # | Item |', 'Texto.\n\n## Título']) {
      expect(tidyAlertBody(ok)).toBe(ok);
    }
  });

  it('renders headings, table and bold of the month close', () => {
    const { container } = render(<Markdown text={tidyAlertBody(BODY)} tone="dim" />);
    expect(screen.getByRole('heading', { name: 'Fechamento de outubro de 2026' })).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Resultado' })).toBeTruthy();
    const table = screen.getByRole('table');
    expect(table.textContent).toContain('Receitas');
    expect(container.textContent).not.toContain('|---');
    expect(container.textContent).not.toContain('##');
    expect(screen.getByText('Taxa de poupança:').tagName).toBe('STRONG');
  });
});
