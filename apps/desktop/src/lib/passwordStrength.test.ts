import { describe, expect, it } from 'vitest';

import { estimatePasswordStrength as estimate, strengthLabel } from './passwordStrength';

describe('estimatePasswordStrength', () => {
  it('an empty password is very weak and has no suggestions', () => {
    expect(estimate('')).toMatchObject({ score: 0, bits: 0, suggestions: [] });
  });

  it.each(['senha123', 'Tr3asury!2026', 'Brasil2026!', 'aaaaaaaaaaaa', 'abcdefghijkl'])(
    '%s is very weak (common patterns)',
    (password) => {
      expect(estimate(password).score).toBe(0);
    },
  );

  it('phrases of 4+ words are good or strong', () => {
    expect(estimate('cavalo bateria grampo correto').score).toBeGreaterThanOrEqual(3);
    expect(estimate('vaca amarela pulou a janela').score).toBe(4);
  });

  it('long random passwords are strong', () => {
    expect(estimate('k8#Lp2!qZ7@x').score).toBe(4);
  });

  it('short passwords never get past weak', () => {
    expect(estimate('Xq7$mPz2').score).toBeLessThanOrEqual(1);
    expect(estimate('Xq7$mP').score).toBe(0);
  });

  it('family names are worth almost nothing', () => {
    const context = ['Família Silva', 'Ana', 'Bruno'];
    const withNames = estimate('silvaanabruno', context);
    const without = estimate('silvaanabruno');
    expect(withNames.bits).toBeLessThan(without.bits);
    expect(withNames.suggestions.join(' ')).toMatch(/nome da família/);
  });

  it('more random length never lowers the estimate', () => {
    let previous = 0;
    for (const password of ['k8#L', 'k8#Lp2!q', 'k8#Lp2!qZ7@x', 'k8#Lp2!qZ7@xR4%w']) {
      const { bits } = estimate(password);
      expect(bits).toBeGreaterThanOrEqual(previous);
      previous = bits;
    }
  });

  it('gives suggestions in Portuguese', () => {
    const result = estimate('senha2024');
    expect(result.label).toBe(strengthLabel(0));
    expect(result.suggestions).toEqual(
      expect.arrayContaining([
        expect.stringMatching(/12 caracteres/),
        expect.stringMatching(/senhas comuns/),
        expect.stringMatching(/anos e datas/),
      ]),
    );
  });
});
