import { useCallback, useEffect, useRef, useState, type PointerEvent } from 'react';

/** Tamanho escolhido pelo usuário (px), lembrado por diálogo. */
export interface DialogSize {
  w: number;
  h: number;
}

const MIN_W = 420;
const MIN_H = 260;
const KEY = 'treasury.dialog-size.';

function bounds(): { w: number; h: number } {
  return { w: Math.round(window.innerWidth * 0.98), h: Math.round(window.innerHeight * 0.96) };
}

function clamp(s: DialogSize): DialogSize {
  const b = bounds();
  return {
    w: Math.max(Math.min(MIN_W, b.w), Math.min(Math.round(s.w), b.w)),
    h: Math.max(Math.min(MIN_H, b.h), Math.min(Math.round(s.h), b.h)),
  };
}

function load(id: string | undefined): DialogSize | null {
  if (!id) return null;
  try {
    const raw = localStorage.getItem(KEY + id);
    if (!raw) return null;
    const v = JSON.parse(raw) as Partial<DialogSize>;
    return typeof v.w === 'number' && typeof v.h === 'number' ? { w: v.w, h: v.h } : null;
  } catch {
    return null;
  }
}

function save(id: string | undefined, s: DialogSize | null) {
  if (!id) return;
  try {
    if (s) localStorage.setItem(KEY + id, JSON.stringify(s));
    else localStorage.removeItem(KEY + id);
  } catch {
    // Armazenamento indisponível: o tamanho só vale até fechar.
  }
}

export type ResizeEdge = 'corner' | 'right' | 'bottom';

/**
 * Redimensionar um diálogo centralizado pela alça do canto (ou pelas bordas): como ele
 * fica no meio da tela, cada pixel arrastado cresce dos dois lados, então o tamanho muda o
 * dobro do arrasto e a alça acompanha o ponteiro. Duplo clique na alça volta ao padrão;
 * "maximizar" ocupa quase a janela toda.
 */
export function useDialogResize(id: string | undefined) {
  const [size, setSize] = useState<DialogSize | null>(() => load(id));
  const [maximized, setMaximized] = useState(false);
  const drag = useRef<{ x: number; y: number; w: number; h: number; edge: ResizeEdge } | null>(
    null,
  );

  // A janela do app encolheu: o tamanho lembrado não pode passar dela.
  useEffect(() => {
    const onResize = () => setSize((s) => (s ? clamp(s) : s));
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);

  const start = useCallback(
    (edge: ResizeEdge) => (e: PointerEvent<HTMLElement>) => {
      if (e.button !== 0) return;
      const box = e.currentTarget.closest<HTMLElement>('[role="dialog"]')?.getBoundingClientRect();
      if (!box) return;
      e.preventDefault();
      e.currentTarget.setPointerCapture(e.pointerId);
      setMaximized(false);
      drag.current = { x: e.clientX, y: e.clientY, w: box.width, h: box.height, edge };
    },
    [],
  );

  const move = useCallback((e: PointerEvent<HTMLElement>) => {
    const d = drag.current;
    if (!d) return;
    const dx = d.edge === 'bottom' ? 0 : (e.clientX - d.x) * 2;
    const dy = d.edge === 'right' ? 0 : (e.clientY - d.y) * 2;
    setSize(clamp({ w: d.w + dx, h: d.h + dy }));
  }, []);

  const end = useCallback(
    (e: PointerEvent<HTMLElement>) => {
      if (!drag.current) return;
      drag.current = null;
      if (e.currentTarget.hasPointerCapture(e.pointerId))
        e.currentTarget.releasePointerCapture(e.pointerId);
      setSize((s) => {
        save(id, s);
        return s;
      });
    },
    [id],
  );

  const reset = useCallback(() => {
    setMaximized(false);
    setSize(null);
    save(id, null);
  }, [id]);

  const effective: DialogSize | null = maximized ? bounds() : size;
  return {
    size: effective,
    maximized,
    toggleMaximized: () => setMaximized((m) => !m),
    reset,
    handle: (edge: ResizeEdge) => ({
      onPointerDown: start(edge),
      onPointerMove: move,
      onPointerUp: end,
      onPointerCancel: end,
      onDoubleClick: reset,
    }),
  };
}
