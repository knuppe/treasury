import { isApiError } from '@/api/client';
import { t } from '@/i18n';

/**
 * Mensagem de erro para mostrar: o núcleo já manda em pt-BR (em minúsculas,
 * como frase de log); aqui só ganha a inicial maiúscula e o ponto final. Sem mensagem, o
 * `fallback` (por padrão, "Algo deu errado." no idioma do app).
 */
export function errorText(error: unknown, fallback = t('controles-erro-generico')): string {
  const raw = isApiError(error)
    ? error.message
    : error instanceof Error
      ? error.message
      : typeof error === 'object' && error && 'message' in error
        ? String(error.message)
        : '';
  const text = raw.trim();
  if (!text) return fallback;
  const sentence = text.charAt(0).toUpperCase() + text.slice(1);
  return /[.!?…]$/.test(sentence) ? sentence : `${sentence}.`;
}

export function errorCode(error: unknown): string | null {
  return isApiError(error) ? error.code : null;
}
