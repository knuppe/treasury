/**
 * Comportamento de app nativo na janela do Tauri: sem o menu de contexto do navegador,
 * sem arrastar imagens e links, sem recarregar a página por atalho, e a janela nunca rola
 * (só as áreas do app). Campos de texto continuam com o menu de edição (copiar, colar) e
 * com seleção normal.
 *
 * Em desenvolvimento (`vite`), o menu fica liberado para o inspetor de elementos.
 */

const EDITABLE = 'input, textarea, select, [contenteditable=""], [contenteditable="true"]';

/** Elementos onde o menu e a seleção continuam valendo (campos e textos copiáveis). */
export function allowsNativeMenu(target: EventTarget | null): boolean {
  return target instanceof Element && Boolean(target.closest(`${EDITABLE}, [data-selectable]`));
}

export function isReloadShortcut(e: KeyboardEvent): boolean {
  const mod = e.metaKey || e.ctrlKey;
  return e.key === 'F5' || (mod && (e.key === 'r' || e.key === 'R'));
}

export function installNativeFeel(doc: Document = document, dev = import.meta.env.DEV): () => void {
  const onContextMenu = (e: MouseEvent) => {
    if (dev || allowsNativeMenu(e.target)) return;
    e.preventDefault();
  };
  const onKeyDown = (e: KeyboardEvent) => {
    if (!dev && isReloadShortcut(e)) e.preventDefault();
  };
  const onDragStart = (e: DragEvent) => {
    const t = e.target;
    if (t instanceof HTMLImageElement || t instanceof HTMLAnchorElement) e.preventDefault();
  };
  // `overflow: hidden` ainda deixa rolar por código (`scrollIntoView`, foco): volta ao topo.
  const win = doc.defaultView;
  const onScroll = () => {
    if (win && (win.scrollX !== 0 || win.scrollY !== 0)) win.scrollTo(0, 0);
  };
  doc.addEventListener('contextmenu', onContextMenu);
  doc.addEventListener('keydown', onKeyDown);
  doc.addEventListener('dragstart', onDragStart);
  win?.addEventListener('scroll', onScroll, { passive: true });
  return () => {
    doc.removeEventListener('contextmenu', onContextMenu);
    doc.removeEventListener('keydown', onKeyDown);
    doc.removeEventListener('dragstart', onDragStart);
    win?.removeEventListener('scroll', onScroll);
  };
}
