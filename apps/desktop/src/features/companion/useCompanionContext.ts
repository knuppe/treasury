/**
 * Contexto que o painel do agente envia com a mensagem: o que a tela aberta registrou com
 * `usePageContext` (ou, sem registro, só qual tela é) e a rota com os filtros da URL.
 */
import { useLocation } from 'react-router';

import { SCREENS } from '@/app/screens';
import { t } from '@/i18n';

import { fitContext, useCurrentPageContext, type PageContext } from './pageContext';

export interface CompanionContext {
  /** Objeto enviado como `page_context` (ou `null` se não couber). */
  value: (PageContext & Record<string, unknown>) | null;
  /** Rótulo curto da tela (`03 · Contas`). */
  label: string;
  /** Id da tela aberta. */
  screen: string | null;
}

/** Parâmetros da URL que não descrevem a tela (pedidos e rascunhos). */
const SKIP_PARAMS = new Set(['pergunta', 'conversa']);

export function useCompanionContext(): CompanionContext {
  const page = useCurrentPageContext();
  const location = useLocation();
  const def = SCREENS.find((s) => location.pathname.startsWith(s.path));

  const base: PageContext | null =
    page ?? (def ? { screen: def.id, title: `${def.index} · ${def.title}` } : null);
  if (!base) return { value: null, label: t('companheiro-fora-das-telas'), screen: null };
  const params = new URLSearchParams(location.search);
  for (const k of [...params.keys()]) if (SKIP_PARAMS.has(k)) params.delete(k);
  const search = params.toString();
  const extra: Record<string, unknown> = {
    route: search ? `${location.pathname}?${search}` : location.pathname,
  };
  return {
    value: fitContext({ ...base, ...extra }),
    label: base.title,
    screen: base.screen,
  };
}
