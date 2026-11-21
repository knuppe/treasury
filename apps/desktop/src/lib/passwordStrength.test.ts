import { describe, expect, it } from 'vitest';

import { estimatePasswordStrength as estimate, strengthLabel } from './passwordStrength';

describe('estimatePasswordStrength', () => {
  it('senha vazia é muito fraca e sem sugestões', () => {
    expect(estimate('')).toMatchObject({ score: 0, bits: 0, suggestions: [] });
  });

  it.each(['senha123', 'Tr3asury!2026', 'Brasil2026!', 'aaaaaaaaaaaa', 'abcdefghijkl'])(
    '%s é muito fraca (padrões comuns)',
    (password) => {
      expect(estimate(password).score).toBe(0);
    },
  );

  it('frases de 4+ palavras são boas ou fortes', () => {
    expect(estimate('cavalo bateria grampo correto').score).toBeGreaterThanOrEqual(3);
    expect(estimate('vaca amarela pulou a janela').score).toBe(4);
  });

  it('senhas aleatórias longas são fortes', () => {
    expect(estimate('k8#Lp2!qZ7@x').score).toBe(4);
  });

  it('senhas curtas nunca passam de fraca', () => {
    expect(estimate('Xq7$mPz2').score).toBeLessThanOrEqual(1);
    expect(estimate('Xq7$mP').score).toBe(0);
  });

  it('nomes da família valem quase nada', () => {
    const context = ['Família Silva', 'Ana', 'Bruno'];
    const withNames = estimate('silvaanabruno', context);
    const without = estimate('silvaanabruno');
    expect(withNames.bits).toBeLessThan(without.bits);
    expect(withNames.suggestions.join(' ')).toMatch(/nome da família/);
  });

  it('mais comprimento aleatório nunca diminui a estimativa', () => {
    let previous = 0;
    for (const password of ['k8#L', 'k8#Lp2!q', 'k8#Lp2!qZ7@x', 'k8#Lp2!qZ7@xR4%w']) {
      const { bits } = estimate(password);
      expect(bits).toBeGreaterThanOrEqual(previous);
      previous = bits;
    }
  });

  it('dá sugestões em português', () => {
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
