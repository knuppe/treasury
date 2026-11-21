/**
 * Última posição do mouse na janela, para abrir painéis "onde o usuário está" quando a
 * ação vem do teclado (ex.: `c` nos lançamentos marcados abre o seletor de categoria
 * junto do mouse, não no botão da barra lá em cima).
 */
export interface Point {
  x: number;
  y: number;
}

let last: Point | null = null;
let listening = false;

function listen() {
  if (listening || typeof window === 'undefined') return;
  listening = true;
  window.addEventListener(
    'pointermove',
    (e) => {
      last = { x: e.clientX, y: e.clientY };
    },
    { passive: true, capture: true },
  );
  // Mouse saiu da janela: a posição antiga não vale mais.
  document.addEventListener('mouseleave', () => {
    last = null;
  });
}
listen();

/** Posição do mouse se ele estiver dentro da janela; senão, `null`. */
export function lastPointer(): Point | null {
  if (!last) return null;
  const inside =
    last.x >= 0 && last.y >= 0 && last.x <= window.innerWidth && last.y <= window.innerHeight;
  return inside ? last : null;
}

/**
 * Onde abrir um painel pedido pelo teclado: no mouse, se ele estiver na janela; senão,
 * logo abaixo do elemento com foco (a linha da tabela).
 */
export function pointerOrFocus(): Point | null {
  const p = lastPointer();
  if (p) return p;
  const el = document.activeElement;
  if (!(el instanceof HTMLElement) || el === document.body) return null;
  const r = el.getBoundingClientRect();
  return { x: r.left + 16, y: r.bottom };
}

/** Só para testes: define a última posição conhecida. */
export function setLastPointerForTests(p: Point | null): void {
  last = p;
}
