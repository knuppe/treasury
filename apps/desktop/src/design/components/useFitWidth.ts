import { useLayoutEffect, useRef } from 'react';

/**
 * Encolhe o número quando ele não cabe na largura do cartão (valores longos em
 * cartões estreitos), sem nunca passar do tamanho do token. Use `calc(<tamanho> *
 * var(--fit, 1))` no font-size da caixa e `whitespace-nowrap`; o conteúdo vai num
 * elemento `inline-block` com `content`.
 */
export function useFitWidth(dep: unknown) {
  const box = useRef<HTMLElement>(null);
  const content = useRef<HTMLSpanElement>(null);
  useLayoutEffect(() => {
    const el = box.current;
    const inner = content.current;
    if (!el || !inner) return;
    let fit = 1;
    const apply = () => {
      const natural = inner.scrollWidth / fit;
      const avail = el.clientWidth;
      const next =
        natural > 0 && avail > 0 && natural > avail ? Math.max(0.55, avail / natural) : 1;
      if (Math.abs(next - fit) > 0.005) {
        fit = next;
        el.style.setProperty('--fit', String(Math.floor(next * 1000) / 1000));
      }
    };
    apply();
    if (typeof ResizeObserver === 'undefined') return;
    const ro = new ResizeObserver(apply);
    ro.observe(el);
    ro.observe(inner);
    return () => ro.disconnect();
  }, [dep]);
  return { box, content };
}
