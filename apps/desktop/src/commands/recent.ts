/**
 * Comandos usados por último na barra ⌘K (só neste computador, no armazenamento do
 * navegador): aparecem primeiro quando a barra abre sem nada digitado.
 */
const KEY = 'treasury.cmd.recentes';
const MAX = 6;

export function recentCommandIds(): string[] {
  try {
    const raw = globalThis.localStorage?.getItem(KEY);
    const list: unknown = raw ? JSON.parse(raw) : [];
    return Array.isArray(list) ? list.filter((x): x is string => typeof x === 'string') : [];
  } catch {
    return [];
  }
}

export function rememberCommand(id: string): void {
  try {
    const next = [id, ...recentCommandIds().filter((x) => x !== id)].slice(0, MAX);
    globalThis.localStorage?.setItem(KEY, JSON.stringify(next));
  } catch {
    // Sem armazenamento (modo privado): a lista de recentes só não aparece.
  }
}
