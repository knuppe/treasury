/**
 * Ambiente de teste (Vitest + jsdom): polyfills das APIs de layout que o jsdom
 * não implementa e que Radix/cmdk usam, e limpeza entre testes.
 */
import { cleanup, configure } from '@testing-library/react';
import { afterEach } from 'vitest';

// Os testes de fluxo renderizam o app inteiro; com a máquina carregada (ex.:
// compilando o núcleo Rust ao lado), 1 s de espera padrão do findBy é pouco. O primeiro
// teste de cada arquivo ainda carrega as telas do app no worker: com a carga das
// verificações da release (testes do Rust em paralelo, outras compilações), 5 s ainda
// estourava em Ativos e na análise fundamentalista. O prazo só pesa quando o teste falha.
configure({ asyncUtilTimeout: 15_000 });

class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

if (!('ResizeObserver' in globalThis)) {
  Object.defineProperty(globalThis, 'ResizeObserver', { value: ResizeObserverStub });
}

if (!Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = function scrollIntoView() {};
}

if (!Element.prototype.hasPointerCapture) {
  Element.prototype.hasPointerCapture = () => false;
  Element.prototype.setPointerCapture = () => {};
  Element.prototype.releasePointerCapture = () => {};
}

if (!window.matchMedia) {
  Object.defineProperty(window, 'matchMedia', {
    value: (query: string): MediaQueryList =>
      ({
        matches: false,
        media: query,
        onchange: null,
        addListener: () => {},
        removeListener: () => {},
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
      }) as MediaQueryList,
  });
}

afterEach(() => {
  cleanup();
  window.localStorage.clear();
  window.location.hash = '';
});
