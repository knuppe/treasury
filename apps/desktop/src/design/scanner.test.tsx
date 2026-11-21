import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { ScannerSweep, ScannerVoice, LiveIndicator } from './components';
import scannerCss from './scanner.css?raw';
import tokensCss from './tokens.css?raw';

describe('tema Scanner', () => {
  const reduced = scannerCss.slice(scannerCss.indexOf('@media (prefers-reduced-motion: reduce)'));

  it('o scanner tem oito lâmpadas, a luz e dois rastros, e é decorativo', () => {
    const { container } = render(<ScannerSweep />);
    const scanner = container.querySelector('.scanner-sweep')!;
    expect(scanner.getAttribute('aria-hidden')).toBe('true');
    expect(scanner.getAttribute('data-variant')).toBe('bar');
    expect(scanner.querySelectorAll('.scanner-beam')).toHaveLength(3);
    expect(scanner.querySelectorAll('.scanner-trail')).toHaveLength(2);
    // Oito janelas que escurecem nas pontas, com vidro, linhas de varredura e grade de TV.
    expect(scannerCss).toContain('mask-size: 12.5% 100%');
    expect(scannerCss).toContain('transparent 5%, #000 24% 76%, transparent 95%');
    expect(scannerCss).toContain('repeating-linear-gradient(180deg, rgb(0 0 0 / 0.4) 0 1px');
  });

  it('ligado por padrão; desligado, recolhe como TV de tubo e pausa a luz', () => {
    const { container, rerender } = render(<ScannerSweep />);
    const scanner = () => container.querySelector('.scanner-sweep')!;
    expect(scanner().getAttribute('data-active')).toBe('true');
    rerender(<ScannerSweep active={false} />);
    expect(scanner().getAttribute('data-active')).toBe('false');
    const at = scannerCss.indexOf(".scanner-sweep[data-active='false'] {");
    const rule = scannerCss.slice(at, scannerCss.indexOf('}', at));
    expect(rule).toContain('opacity: 0');
    expect(rule).toContain('transform: scale(0.03, 0.4)');
    expect(scannerCss).toContain('animation-play-state: paused');
  });

  it('a caixa de voz só pulsa enquanto o agente fala', () => {
    const { container, rerender } = render(<ScannerVoice speaking={false} />);
    const voice = () => container.querySelector('.scanner-voice')!;
    expect(voice().hasAttribute('data-speaking')).toBe(false);
    expect(voice().querySelectorAll('.scanner-voice-bars > span')).toHaveLength(3);
    rerender(<ScannerVoice speaking />);
    expect(voice().getAttribute('data-speaking')).toBe('true');
  });

  it('o indicador ativo ganha o mini scanner; o parado fica só com o ponto', () => {
    const { container, rerender } = render(<LiveIndicator label="Pensando" />);
    expect(container.querySelector('.scanner-mini')).not.toBeNull();
    expect(container.querySelector('.live-dot')).not.toBeNull();
    rerender(<LiveIndicator label="Parado" active={false} />);
    expect(container.querySelector('.scanner-mini')).toBeNull();
    expect(container.querySelector('.live-dot')).toBeNull();
  });

  it('as peças ficam escondidas fora do tema, inclusive dentro de miniaturas', () => {
    const css = tokensCss.replace(/\/\*[\s\S]*?\*\//g, '');
    const derived = /:root,\s*\[data-theme\]\s*\{([^}]*)\}/.exec(css)![1]!;
    expect(derived).toContain('--scanner-display: none');
    expect(derived).toContain('--scanner-hide: initial');
    const kitt = /\[data-theme='kitt'\]\s*\{([^}]*)\}/.exec(css)![1]!;
    expect(kitt).toContain('--scanner-display: block');
    expect(kitt).toContain('--scanner-hide: none');
    for (const sel of ['.scanner-sweep', '.scanner-mini', '.scanner-voice']) {
      const at = scannerCss.indexOf(`${sel} {`);
      expect(scannerCss.slice(at, scannerCss.indexOf('}', at)), sel).toContain(
        'display: var(--scanner-display)',
      );
    }
  });

  it('as animações só mexem em transform e opacidade', () => {
    const keyframes = [...scannerCss.matchAll(/@keyframes ([\w-]+) \{([\s\S]*?)\n\}\n/g)];
    expect(keyframes.map((k) => k[1])).toEqual([
      'scanner-flicker',
      'scanner-sweep',
      'scanner-sweep-mini',
      'scanner-voice',
    ]);
    for (const [, name, body] of keyframes) {
      const props = [...body!.matchAll(/([\w-]+):/g)].map((m) => m[1]);
      for (const p of props) expect(['opacity', 'transform'], `${name}: ${p}`).toContain(p);
    }
  });

  it('com movimento reduzido, a luz para no centro e a voz não pulsa', () => {
    expect(reduced.length).toBeGreaterThan(0);
    for (const sel of [
      '.scanner-lamps',
      '.scanner-beam',
      '.scanner-halo',
      '.scanner-mini::after',
      '.scanner-voice-bars > span',
    ]) {
      expect(reduced).toContain(sel);
    }
    expect(reduced).toContain('animation: none !important');
    expect(reduced).toContain('translateX(150%)');
  });
});
