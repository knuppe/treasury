import { describe, expect, it } from 'vitest';

import { assignColors, categoricalPalette, seriesCap } from './palette';
import { compactMoney } from './theme';

describe('chart palette', () => {
  it('assigns colors in a fixed order, tied to the entity', () => {
    const all = ['moradia', 'alimentacao', 'transporte'];
    const map = assignColors(all, 'dark');
    expect(map.get('moradia')).toBe(categoricalPalette('dark')[0]);
    expect(map.get('transporte')).toBe(categoricalPalette('dark')[2]);
  });

  it('limits series in Mono (a single hue)', () => {
    expect(seriesCap('mono')).toBe(4);
    expect(assignColors(['a', 'b', 'c', 'd', 'e'], 'mono').has('e')).toBe(false);
  });

  it('formats compact values in pt-BR and respects privacy', () => {
    expect(compactMoney(1_290_000, { privacy: false })).toBe('R$ 12,9 mil');
    expect(compactMoney(-120_000_000, { privacy: false })).toBe('−R$ 1,2 mi');
    expect(compactMoney(4_550, { privacy: false })).toBe('R$ 45,5');
    expect(compactMoney(1_290_000, { privacy: true })).toBe('•••••');
  });
});
