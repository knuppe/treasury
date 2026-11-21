/**
 * Ajuda dos testes da carteira e dos ativos: data fixa (o mock gera cotações e
 * proventos a partir de hoje) e um contexto 2D falso (o jsdom não desenha; ECharts
 * e Lightweight Charts pintam num contexto que aceita tudo e não faz nada).
 */
import { vi } from 'vitest';

export const TEST_TODAY = new Date('2026-10-14T12:00:00');

export function fixToday(): void {
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(TEST_TODAY);
}

function fakeContext(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const store: Record<string | symbol, unknown> = { canvas };
  const noop = () => undefined;
  const special: Record<string, unknown> = {
    measureText: (text: string) => ({
      width: String(text).length * 6,
      actualBoundingBoxAscent: 8,
      actualBoundingBoxDescent: 2,
    }),
    getImageData: () => ({ data: new Uint8ClampedArray(4) }),
    createLinearGradient: () => ({ addColorStop: noop }),
    createRadialGradient: () => ({ addColorStop: noop }),
    createPattern: () => ({}),
    getLineDash: () => [],
    getTransform: () => ({ a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 }),
    isPointInPath: () => false,
  };
  return new Proxy(store, {
    get: (target, prop) => {
      if (prop in target) return target[prop];
      if (typeof prop === 'string' && prop in special) return special[prop];
      return noop;
    },
    set: (target, prop, value) => {
      target[prop] = value;
      return true;
    },
  }) as unknown as CanvasRenderingContext2D;
}

export function quietCanvas(): void {
  const contexts = new WeakMap<HTMLCanvasElement, CanvasRenderingContext2D>();
  Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
    configurable: true,
    value(this: HTMLCanvasElement) {
      let ctx = contexts.get(this);
      if (!ctx) {
        ctx = fakeContext(this);
        contexts.set(this, ctx);
      }
      return ctx;
    },
  });
}
