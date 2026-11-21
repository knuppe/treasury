import { useLayoutEffect, useRef } from 'react';

import { prefersReducedMotion } from '../themes';

/**
 * Texto que chega "decodificando" no tema Fósforo (phosphor.css): ao montar ou mudar, os
 * caracteres aparecem embaralhados e vão travando no valor real da esquerda para a direita, em
 * 400 a 800ms. Fora do Fósforo, ou com movimento reduzido, é só o texto.
 *
 * O texto real fica sempre no DOM (leitores de tela e a largura do layout não mudam); o
 * embaralhado é um `::after` por cima, lido de `data-scramble`, que só este código escreve.
 * Um único laço de animação atende todos os textos, e no máximo `MAX_ACTIVE` decodificam ao
 * mesmo tempo (uma tabela grande chegando não vira centenas de animações).
 */

const NOISE = 'ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅ0123456789';
const MAX_ACTIVE = 48;

interface Job {
  el: HTMLElement;
  text: string;
  start: number;
  duration: number;
}

const jobs = new Map<HTMLElement, Job>();
let raf = 0;

function scramble(text: string, locked: number): string {
  let out = text.slice(0, locked);
  for (let i = locked; i < text.length; i++) {
    const ch = text[i]!;
    // Separadores e espaços ficam: o formato do número continua reconhecível.
    out += /[\s.,:/%()+\-−R$€£¥≈]/.test(ch) ? ch : NOISE[Math.floor(Math.random() * NOISE.length)]!;
  }
  return out;
}

function stop(job: Job) {
  job.el.removeAttribute('data-scramble');
  job.el.classList.remove('decoding');
  jobs.delete(job.el);
}

function tick(now: number) {
  for (const job of jobs.values()) {
    if (!job.el.isConnected) {
      jobs.delete(job.el);
      continue;
    }
    const p = (now - job.start) / job.duration;
    if (p >= 1) stop(job);
    else job.el.setAttribute('data-scramble', scramble(job.text, Math.floor(p * job.text.length)));
  }
  raf = jobs.size ? requestAnimationFrame(tick) : 0;
}

/** O Fósforo está no ar (no <html>), sem movimento reduzido. */
export function phosphorMotion(): boolean {
  return (
    typeof document !== 'undefined' &&
    document.documentElement.dataset.theme === 'matrix' &&
    !prefersReducedMotion()
  );
}

/** Começa a decodificar `el` (que já mostra `text`). */
export function decode(el: HTMLElement, text: string): void {
  if (!phosphorMotion() || !text.trim() || jobs.size >= MAX_ACTIVE) return;
  const job = { el, text, start: performance.now(), duration: 400 + Math.random() * 400 };
  jobs.set(el, job);
  el.classList.add('decoding');
  el.setAttribute('data-scramble', scramble(text, 0));
  if (!raf) raf = requestAnimationFrame(tick);
}

export function Decode({ text, className }: { text: string; className?: string }) {
  const ref = useRef<HTMLSpanElement>(null);
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    decode(el, text);
    return () => {
      const job = jobs.get(el);
      if (job) stop(job);
    };
  }, [text]);
  return (
    <span ref={ref} className={className ? `decode ${className}` : 'decode'}>
      {text}
    </span>
  );
}
