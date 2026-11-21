import { describe, expect, it } from 'vitest';

import { inlineText, parseBlocks, parseInline, safeHref } from './markdown';

describe('markdown das respostas do agente', () => {
  it('títulos, negrito, listas e parágrafos como nas respostas de um agente de código', () => {
    const md = [
      '## Outros ajustes',
      '',
      '**Arquivar a conta** antiga e manter o histórico:',
      '',
      '- Conta *Itaú* com saldo `R$ 0,00`',
      '- Cartão:',
      '  1. fechamento dia 3',
      '  2. vencimento dia 10',
      '',
      'Pronto.',
    ].join('\n');
    const b = parseBlocks(md);
    expect(b.map((x) => x.t)).toEqual(['heading', 'paragraph', 'list', 'paragraph']);
    expect(b[0]).toMatchObject({ t: 'heading', level: 2 });
    const p = b[1] as Extract<(typeof b)[number], { t: 'paragraph' }>;
    expect(p.c[0]).toMatchObject({ t: 'strong' });
    expect(inlineText(p.c)).toBe('Arquivar a conta antiga e manter o histórico:');
    const list = b[2] as Extract<(typeof b)[number], { t: 'list' }>;
    expect(list.ordered).toBe(false);
    expect(list.items).toHaveLength(2);
    const nested = list.items[1]!.blocks[1] as Extract<(typeof b)[number], { t: 'list' }>;
    expect(nested).toMatchObject({ t: 'list', ordered: true, start: 1 });
    expect(nested.items).toHaveLength(2);
  });

  it('tabela com alinhamento, código cercado e citação', () => {
    const md = [
      '| Ativo | Qtd | Valor |',
      '|:--|:-:|--:|',
      '| PETR4 | 100 | R$ 3.050,00 |',
      '| HGLG11 | 12 | R$ 1.898,40 |',
      '',
      '```json',
      '{"a": 1}',
      '```',
      '> nota *importante*',
    ].join('\n');
    const b = parseBlocks(md);
    expect(b.map((x) => x.t)).toEqual(['table', 'code', 'quote']);
    const t = b[0] as Extract<(typeof b)[number], { t: 'table' }>;
    expect(t.align).toEqual(['left', 'center', 'right']);
    expect(t.rows).toHaveLength(2);
    expect(inlineText(t.rows[1]![0]!)).toBe('HGLG11');
    expect(b[1]).toMatchObject({ t: 'code', lang: 'json', v: '{"a": 1}' });
  });

  it('texto incompleto do streaming continua legível', () => {
    expect(inlineText(parseInline('um **negrito sem fim'))).toBe('um **negrito sem fim');
    const b = parseBlocks('```\nconst x = 1');
    expect(b[0]).toMatchObject({ t: 'code', v: 'const x = 1' });
    expect(parseBlocks('')).toEqual([]);
  });

  it('não confunde sublinhado em nomes, multiplicação ou linha horizontal', () => {
    expect(parseInline('arquivo nome_de_arquivo_final.csv')).toEqual([
      { t: 'text', v: 'arquivo nome_de_arquivo_final.csv' },
    ]);
    expect(parseInline('2 * 3 * 4')).toEqual([{ t: 'text', v: '2 * 3 * 4' }]);
    expect(parseBlocks('a\n\n---\n\nb').map((x) => x.t)).toEqual(['paragraph', 'hr', 'paragraph']);
    expect(parseInline('\\*literal\\*')).toEqual([{ t: 'text', v: '*literal*' }]);
  });

  it('links: só https, e-mail e telas do app; o resto vira texto', () => {
    const n = parseInline(
      'veja [a fila](/agente?aba=aprovacao), [site](https://b3.com.br) e https://x.com/a.',
    );
    const links = n.filter((x) => x.t === 'link');
    expect(links.map((l) => (l as { href: string }).href)).toEqual([
      '/agente?aba=aprovacao',
      'https://b3.com.br',
      'https://x.com/a',
    ]);
    expect(inlineText(n).endsWith('.')).toBe(true);
    expect(parseInline('[x](javascript:alert(1))').some((x) => x.t === 'link')).toBe(false);
    expect(safeHref('//evil.com')).toBeNull();
    expect(safeHref('file:///etc/passwd')).toBeNull();
    // HTML vira texto.
    expect(parseInline('<img src=x onerror=alert(1)>')).toEqual([
      { t: 'text', v: '<img src=x onerror=alert(1)>' },
    ]);
  });

  it('lista de tarefas, numeração inicial e quebra de linha simples', () => {
    const b = parseBlocks('3. três\n4. quatro\n\n- [x] feito\n- [ ] falta\n\nlinha 1\nlinha 2');
    expect(b[0]).toMatchObject({ t: 'list', ordered: true, start: 3 });
    const tasks = b[1] as Extract<(typeof b)[number], { t: 'list' }>;
    expect(tasks.items.map((i) => i.checked)).toEqual([true, false]);
    const p = b[2] as Extract<(typeof b)[number], { t: 'paragraph' }>;
    expect(p.c.some((x) => x.t === 'br')).toBe(true);
  });
});
