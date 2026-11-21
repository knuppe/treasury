/**
 * Cabeçalho da tabela visível enquanto a página (ou o diálogo) rola. O `position: sticky` do
 * `<thead>` só vale dentro da caixa que rola: as tabelas ficam numa caixa com rolagem
 * horizontal (`overflow-x: auto`), que também prende a vertical, e o cabeçalho sumia ao
 * rolar a tela. Aqui as células do cabeçalho descem junto (`translateY`) até a borda de cima
 * da área que rola, sem passar do fim da tabela. Tabelas com altura máxima (rolagem própria)
 * continuam com o `sticky` nativo.
 *
 * No WebKit a rolagem anda antes do JavaScript reposicionar o cabeçalho; rolando para
 * cima, ele fica um instante abaixo da borda e o conteúdo apareceria por cima. Por isso,
 * enquanto flutua, o cabeçalho tem uma capa opaca para cima (styles.css), cortada pela
 * própria área que rola; preso no fim da tabela (`data-floating="end"`), sem capa.
 */
import { useEffect, useLayoutEffect, useRef, type RefObject } from 'react';

/** Primeiro ancestral que rola na vertical (o `<main>`, o corpo de um diálogo…). */
function scrollParent(el: HTMLElement | null): HTMLElement | null {
  for (let p = el; p; p = p.parentElement) {
    const y = getComputedStyle(p).overflowY;
    if (y === 'auto' || y === 'scroll') return p;
  }
  return null;
}

function place(box: HTMLElement, scroller: HTMLElement | null) {
  const head = box.querySelector<HTMLElement>(':scope > table > thead');
  if (!head) return;
  const top = scroller ? scroller.getBoundingClientRect().top : 0;
  const r = box.getBoundingClientRect();
  const foot = box.querySelector<HTMLElement>(':scope > table > tfoot')?.offsetHeight ?? 0;
  const max = r.height - head.offsetHeight - foot;
  const y = Math.round(Math.max(0, Math.min(top - r.top, max)));
  const cells = head.querySelectorAll<HTMLElement>(':scope > tr > th');
  if (y > 0) {
    // `end`: preso no fim da tabela, com linhas de verdade acima dele (sem a capa).
    head.dataset.floating = y >= max ? 'end' : '';
    for (const c of cells) c.style.transform = `translateY(${y}px)`;
  } else if ('floating' in head.dataset) {
    delete head.dataset.floating;
    for (const c of cells) c.style.transform = '';
  }
}

/**
 * `box` é a caixa da tabela (o pai direto do `<table>`). Desligado (`enabled = false`) quando a
 * tabela rola sozinha na vertical.
 */
export function useStickyHeader(box: RefObject<HTMLElement | null>, enabled = true) {
  const scroller = useRef<HTMLElement | null>(null);
  useEffect(() => {
    const el = box.current;
    if (!enabled || !el) return;
    scroller.current = scrollParent(el.parentElement);
    let frame = 0;
    const schedule = () => {
      if (!frame)
        frame = requestAnimationFrame(() => {
          frame = 0;
          place(el, scroller.current);
        });
    };
    // Na rolagem, na hora (o evento já vem uma vez por quadro): esperar o próximo quadro
    // deixava o cabeçalho um passo atrás e o conteúdo aparecia por cima dele.
    const onScroll = () => place(el, scroller.current);
    const target: HTMLElement | Window = scroller.current ?? window;
    target.addEventListener('scroll', onScroll, { passive: true });
    window.addEventListener('resize', schedule);
    const ro = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(schedule);
    ro?.observe(el);
    schedule();
    return () => {
      cancelAnimationFrame(frame);
      target.removeEventListener('scroll', onScroll);
      window.removeEventListener('resize', schedule);
      ro?.disconnect();
      const head = el.querySelector<HTMLElement>(':scope > table > thead');
      if (head) {
        delete head.dataset.floating;
        for (const c of head.querySelectorAll<HTMLElement>(':scope > tr > th'))
          c.style.transform = '';
      }
    };
  }, [box, enabled]);
  // Cabeçalho redesenhado (colunas, ordenação): volta para o lugar certo.
  useLayoutEffect(() => {
    if (enabled && box.current) place(box.current, scroller.current);
  });
}
