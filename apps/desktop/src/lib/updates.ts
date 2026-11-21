/**
 * Atualização automática (docs/auto-update.md): a casca do app procura, baixa e confere a
 * assinatura; aqui só o estado, a procura manual e a instalação, que sempre espera a pessoa.
 * Fora do app (navegador de desenvolvimento, testes) não há atualização: tudo devolve `null`.
 */
import { isTauri } from '@/api/client';

export interface UpdateStatus {
  /** Este app sabe onde procurar (release publicado com o endereço das versões). */
  configured: boolean;
  current: string;
  checking: boolean;
  last_check: string | null;
  available: { version: string; notes: string | null; date: string | null } | null;
  /** Baixada e conferida: falta só confirmar. Sem isso, `available` é só o aviso (modo "só
   * avisar"): baixar é `checkForUpdate(true)`. */
  ready: boolean;
  error: string | null;
}

async function native<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  if (!isTauri()) return null;
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

export const updateStatus = () => native<UpdateStatus>('update_status');

/** Procura agora; `download` força baixar e conferir (sem ele, o modo "só avisar" só avisa). */
export const checkForUpdate = (download?: boolean) =>
  native<UpdateStatus>('update_check', download === undefined ? undefined : { download });

export const installUpdate = () => native<null>('update_install');
