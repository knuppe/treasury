import { useEffect, useRef, useSyncExternalStore } from 'react';
import { createPortal } from 'react-dom';

import { useBusy } from '@/state/activity';
import { useUi } from '@/state/ui';

/**
 * A chuva de código do tema Fósforo (phosphor.css): colunas de katakana meio-largura, dígitos e
 * `R$ % + -` caindo devagar num canvas fixo atrás de tudo. Parada, só a faixa do topo aparece
 * (máscara de ~180px com fade); com alguma tarefa em andamento (state/activity.ts), a chuva
 * fica três vezes mais rápida, mais forte e cobre a tela inteira, e volta em ~600ms quando
 * tudo termina.
 *
 * Quem monta só monta no Fósforo e sem movimento reduzido; trocar de tema desmonta e destrói
 * o laço. Até ~30 quadros por segundo, pausa com a aba oculta e redesenha só o que mudou de
 * linha (cada coluna anda em passos de uma célula).
 */

const GLYPHS = 'ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ0123456789R$%+-';
const CELL = 16;
const TRAIL = 14;
const FRAME_MS = 1000 / 30;
/** Células por segundo, parada; com atividade, três vezes isso. */
const IDLE_SPEED = 6;
const BUSY_FACTOR = 3;
/** Quanto do caminho até a velocidade nova a chuva anda por segundo (~600ms no total). */
const EASE_PER_S = 5;

interface Column {
  /** Linha da cabeça (fracionária). */
  y: number;
  /** Multiplicador próprio, para as colunas não andarem juntas. */
  pace: number;
  glyphs: string[];
}

const pick = () => GLYPHS[Math.floor(Math.random() * GLYPHS.length)]!;

function column(rows: number, scatter: boolean): Column {
  return {
    y: scatter ? Math.random() * rows : -Math.random() * rows * 0.5,
    pace: 0.6 + Math.random() * 0.8,
    glyphs: Array.from({ length: TRAIL }, pick),
  };
}

export function PhosphorRain() {
  const busy = useBusy();
  const canvas = useRef<HTMLCanvasElement>(null);
  const busyRef = useRef(busy);
  useEffect(() => {
    busyRef.current = busy;
  }, [busy]);

  useEffect(() => {
    const el = canvas.current;
    const ctx = el?.getContext('2d');
    if (!el || !ctx) return;
    let cols: Column[] = [];
    let rows = 0;
    let raf = 0;
    let last = 0;
    let speed = 1;
    let painted = 0;
    const color = getComputedStyle(document.documentElement).getPropertyValue('--accent').trim();

    const resize = () => {
      const w = window.innerWidth;
      const h = window.innerHeight;
      el.width = w;
      el.height = h;
      rows = Math.ceil(h / CELL);
      const n = Math.ceil(w / CELL);
      cols = Array.from({ length: n }, (_, i) => cols[i] ?? column(rows, true));
      ctx.font = `${CELL - 2}px ${getComputedStyle(document.documentElement).getPropertyValue('--font-numeric')}`;
      ctx.textBaseline = 'top';
      ctx.fillStyle = color || '#00ff41';
    };

    const draw = (now: number) => {
      raf = requestAnimationFrame(draw);
      const dt = now - last;
      if (dt < FRAME_MS) return;
      last = now;
      const s = Math.min(dt, 100) / 1000;
      const target = busyRef.current ? BUSY_FACTOR : 1;
      speed += (target - speed) * Math.min(1, EASE_PER_S * s);
      // Parada, só a faixa do topo aparece: não vale desenhar abaixo dela.
      const visible = busyRef.current || speed > 1.05 ? rows : Math.ceil(200 / CELL);
      // Parada, só a faixa do topo foi pintada: só ela precisa ser limpa (mais o que o
      // quadro anterior pintou, quando a chuva acabou de voltar para a faixa).
      ctx.clearRect(0, 0, el.width, Math.min(el.height, (Math.max(visible, painted) + 1) * CELL));
      painted = visible;
      for (let i = 0; i < cols.length; i++) {
        const c = cols[i]!;
        c.y += IDLE_SPEED * speed * c.pace * s;
        if (c.y - TRAIL > rows) cols[i] = column(rows, false);
        const head = Math.floor(c.y);
        if (Math.random() < 0.05) c.glyphs[Math.floor(Math.random() * TRAIL)] = pick();
        for (let k = 0; k < TRAIL; k++) {
          const row = head - k;
          if (row < 0 || row >= visible) continue;
          // A cabeça mais forte; o rastro some aos poucos.
          ctx.globalAlpha = k === 0 ? 1 : 0.75 * (1 - k / TRAIL);
          ctx.fillText(c.glyphs[k]!, i * CELL, row * CELL);
        }
      }
      ctx.globalAlpha = 1;
    };

    const start = () => {
      cancelAnimationFrame(raf);
      last = 0;
      raf = requestAnimationFrame(draw);
    };
    const onVisibility = () => {
      if (document.hidden) cancelAnimationFrame(raf);
      else start();
    };

    resize();
    start();
    window.addEventListener('resize', resize);
    document.addEventListener('visibilitychange', onVisibility);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener('resize', resize);
      document.removeEventListener('visibilitychange', onVisibility);
    };
  }, []);

  return createPortal(
    <canvas
      ref={canvas}
      aria-hidden
      className="phosphor-rain"
      data-active={busy ? 'true' : 'false'}
    />,
    document.body,
  );
}

const REDUCED = '(prefers-reduced-motion: reduce)';

function subscribeReduced(onChange: () => void): () => void {
  if (typeof window.matchMedia !== 'function') return () => {};
  const mq = window.matchMedia(REDUCED);
  mq.addEventListener('change', onChange);
  return () => mq.removeEventListener('change', onChange);
}

const reducedNow = () =>
  typeof window.matchMedia === 'function' && window.matchMedia(REDUCED).matches;

/** Movimento reduzido pedido pelo sistema (acompanha a mudança). */
export function useReducedMotion(): boolean {
  return useSyncExternalStore(subscribeReduced, reducedNow, () => false);
}

/** Monta a chuva só no Fósforo e sem movimento reduzido; trocar de tema a destrói. */
export function PhosphorRainHost() {
  const theme = useUi((s) => s.theme);
  const reduced = useReducedMotion();
  return theme === 'matrix' && !reduced ? <PhosphorRain /> : null;
}
