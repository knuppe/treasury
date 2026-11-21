/**
 * Corpo dos alertas em Markdown (fechamento do mês): título colado no texto anterior volta para
 * a própria linha, e títulos e tabelas aparecem desenhados, sem `###` nem `|---|` à mostra.
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

describe('alertas › corpo em Markdown', () => {
  it('separa o título colado e mantém o resto', () => {
    expect(tidyAlertBody('fim.## Fechamento')).toBe('fim.\n\n## Fechamento');
    for (const ok of ['Linguagem C# é citada.', '| # | Item |', 'Texto.\n\n## Título']) {
      expect(tidyAlertBody(ok)).toBe(ok);
    }
  });

  it('desenha títulos, tabela e negrito do fechamento do mês', () => {
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
