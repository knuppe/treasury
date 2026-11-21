import { describe, expect, it } from 'vitest';

import { assignColors, categoricalPalette, seriesCap } from './palette';
import { compactMoney } from './theme';

describe('paleta dos gráficos', () => {
  it('atribui cores em ordem fixa e presas à entidade', () => {
    const all = ['moradia', 'alimentacao', 'transporte'];
    const map = assignColors(all, 'dark');
    expect(map.get('moradia')).toBe(categoricalPalette('dark')[0]);
    expect(map.get('transporte')).toBe(categoricalPalette('dark')[2]);
  });

  it('limita séries no Mono (um só matiz)', () => {
    expect(seriesCap('mono')).toBe(4);
    expect(assignColors(['a', 'b', 'c', 'd', 'e'], 'mono').has('e')).toBe(false);
  });

  it('formata valores compactos em pt-BR e respeita a privacidade', () => {
    expect(compactMoney(1_290_000, { privacy: false })).toBe('R$ 12,9 mil');
    expect(compactMoney(-120_000_000, { privacy: false })).toBe('−R$ 1,2 mi');
    expect(compactMoney(4_550, { privacy: false })).toBe('R$ 45,5');
    expect(compactMoney(1_290_000, { privacy: true })).toBe('•••••');
  });
});
