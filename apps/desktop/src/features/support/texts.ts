/**
 * Textos do apoio (docs/support.md), nos catálogos `locales/<idioma>/apoio.ftl`: o pedido em dois
 * tons (Iniciante e Intermediário; Avançado) e os pedidos que vão para o agente. A língua é a do
 * app (docs/i18n-design.md), não a do país da família. Sem valores sugeridos, sem culpa, metas,
 * contagem regressiva nem comparações.
 */
import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import { t } from '@/i18n';

export function askText(level: ProfileLevel | undefined): string {
  return level === 'avancado' ? t('apoio-pedido-avancado') : t('apoio-pedido-simples');
}

export function thanksText(): string {
  return t('apoio-obrigado');
}

/** Pedido ao agente para relatar um erro, já com o roteiro e a versão do app. */
export function bugReport(version: string): string {
  return t('apoio-erro-pedido', { versao: version });
}
