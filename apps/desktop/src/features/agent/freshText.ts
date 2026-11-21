/**
 * O texto que acabou de chegar na resposta do agente, em verde mais claro por um instante, no
 * tema Fósforo (phosphor.css, `::highlight(agent-fresh)`). Usa a CSS Custom Highlight API: marca
 * só os caracteres novos, sem mexer no DOM que o React desenha. Sem a API (WebView antiga,
 * testes), ou fora do Fósforo, ou com movimento reduzido, nada acontece.
 */
import { useEffect, useRef } from 'react';

import { phosphorMotion } from '@/design/components';

const NAME = 'agent-fresh';
const FRESH_MS = 600;

interface HighlightRegistry {
  get(name: string): { add(r: Range): void; delete(r: Range): boolean } | undefined;
  set(name: string, h: unknown): void;
}

function registry(): HighlightRegistry | null {
  const css = (globalThis as { CSS?: { highlights?: HighlightRegistry } }).CSS;
  const Ctor = (globalThis as { Highlight?: new () => unknown }).Highlight;
  if (!css?.highlights || !Ctor) return null;
  if (!css.highlights.get(NAME)) css.highlights.set(NAME, new Ctor());
  return css.highlights;
}

/** Último nó de texto com conteúdo dentro de `root`. */
function lastText(root: Node): Text | null {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let last: Text | null = null;
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    if ((n as Text).data.trim()) last = n as Text;
  }
  return last;
}

export function useFreshText<T extends HTMLElement>(text: string, running: boolean) {
  const ref = useRef<T>(null);
  const seen = useRef(text.length);
  useEffect(() => {
    const before = seen.current;
    seen.current = text.length;
    const added = text.length - before;
    if (!running || added <= 0 || !ref.current || !phosphorMotion()) return;
    const reg = registry();
    const node = lastText(ref.current);
    if (!reg || !node) return;
    const range = document.createRange();
    range.setStart(node, Math.max(0, node.data.length - added));
    range.setEnd(node, node.data.length);
    reg.get(NAME)?.add(range);
    window.setTimeout(() => reg.get(NAME)?.delete(range), FRESH_MS);
  }, [text, running]);
  return ref;
}
