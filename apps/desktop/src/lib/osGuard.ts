/**
 * Proteções do sistema para segredos (docs/local-security.md, itens 9, 11 e 12): cópia
 * oculta e apagada em 60 s, janela fora das capturas de tela com o modo privacidade e
 * Secure Event Input nos campos de senha. Fora do app (navegador de desenvolvimento,
 * testes), a cópia é a normal e o resto não faz nada.
 */
import { isTauri } from '@/api/client';

/** Copia uma parte de Shamir, chave de API ou código sem ficar no histórico. */
export async function copySecret(text: string): Promise<void> {
  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    try {
      await invoke('secret_copy', { text });
      return;
    } catch {
      // Sistema sem a cópia protegida (Windows, Linux): segue a cópia normal.
    }
  }
  await copyText(text);
}

/** Cópia comum (falha sem a área de transferência, como num navegador sem permissão). */
export async function copyText(text: string): Promise<void> {
  if (typeof navigator === 'undefined' || !navigator.clipboard) throw new Error('clipboard');
  await navigator.clipboard.writeText(text);
}

export function setSecureInput(on: boolean): void {
  if (!isTauri()) return;
  void import('@tauri-apps/api/core')
    .then(({ invoke }) => invoke('secure_input', { on }))
    .catch(() => undefined);
}

/**
 * Janela fora das capturas de tela: com o modo privacidade ligado, salvo se o ajuste
 * "Permitir capturas de tela" (só lido com o cofre aberto) liberar. Quem sabe de cada parte
 * avisa a sua; a proteção sai da combinação. Parcial: some de capturas e do
 * compartilhamento de tela, não de uma câmera apontada.
 */
const capture = { privacy: false, allowed: false };
let applied: boolean | null = null;

export function updateCapture(patch: Partial<typeof capture>): void {
  Object.assign(capture, patch);
  const on = capture.privacy && !capture.allowed;
  if (on === applied) return;
  applied = on;
  if (!isTauri()) return;
  void import('@tauri-apps/api/window')
    .then(({ getCurrentWindow }) => getCurrentWindow().setContentProtected(on))
    .catch(() => undefined);
}

/** A proteção que vale agora (para testes). */
export function captureProtected(): boolean {
  return capture.privacy && !capture.allowed;
}
