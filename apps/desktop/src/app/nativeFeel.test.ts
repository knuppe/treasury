import { afterEach, describe, expect, it } from 'vitest';

import { allowsNativeMenu, installNativeFeel, isReloadShortcut } from './nativeFeel';

let cleanup: (() => void) | null = null;
afterEach(() => {
  cleanup?.();
  cleanup = null;
  document.body.innerHTML = '';
});

function contextMenuOn(el: Element): boolean {
  const ev = new MouseEvent('contextmenu', { bubbles: true, cancelable: true });
  el.dispatchEvent(ev);
  return ev.defaultPrevented;
}

describe('aparência de app nativo', () => {
  it('bloqueia o menu de contexto fora de campos e textos copiáveis', () => {
    document.body.innerHTML =
      '<div id="card"><span id="txt">R$ 10</span></div><input id="f" /><p data-selectable><b id="msg">oi</b></p>';
    cleanup = installNativeFeel(document, false);
    expect(contextMenuOn(document.getElementById('txt')!)).toBe(true);
    expect(contextMenuOn(document.getElementById('f')!)).toBe(false);
    expect(contextMenuOn(document.getElementById('msg')!)).toBe(false);
  });

  it('em desenvolvimento o menu fica liberado (inspetor)', () => {
    document.body.innerHTML = '<div id="card"></div>';
    cleanup = installNativeFeel(document, true);
    expect(contextMenuOn(document.getElementById('card')!)).toBe(false);
  });

  it('reconhece os atalhos de recarregar', () => {
    expect(isReloadShortcut(new KeyboardEvent('keydown', { key: 'r', metaKey: true }))).toBe(true);
    expect(isReloadShortcut(new KeyboardEvent('keydown', { key: 'F5' }))).toBe(true);
    expect(isReloadShortcut(new KeyboardEvent('keydown', { key: 'k', metaKey: true }))).toBe(false);
    expect(allowsNativeMenu(null)).toBe(false);
  });
});
