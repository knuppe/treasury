/**
 * De onde a pessoa veio ao abrir a página de um ativo (docs/asset-page.md): a última tela fora
 * de `/ativo/`, com a busca e os filtros, para a trilha "Mercado › PETR4" voltar ao mesmo recorte.
 * Só na memória da sessão.
 */
import { useEffect } from 'react';
import { useLocation } from 'react-router';
import { create } from 'zustand';

export interface NavOrigin {
  pathname: string;
  search: string;
}

interface NavOriginState {
  origin: NavOrigin | null;
  set: (o: NavOrigin) => void;
}

export const useNavOrigin = create<NavOriginState>()((set) => ({
  origin: null,
  set: (origin) => set({ origin }),
}));

export const isAssetPage = (pathname: string) => pathname.startsWith('/ativo/');

/** Guarda cada tela visitada que não é a página de um ativo (chamado no shell). */
export function useTrackNavOrigin() {
  const { pathname, search } = useLocation();
  const set = useNavOrigin((s) => s.set);
  useEffect(() => {
    if (isAssetPage(pathname)) return;
    // Link antigo de passagem (`/ativos?ativo=…`, `?aba=indicadores&ticker=…`): segue para a
    // página do ativo e não é a tela de origem.
    const q = new URLSearchParams(search);
    if (pathname === '/ativos' && (q.has('ativo') || q.has('ticker'))) return;
    set({ pathname, search });
  }, [pathname, search, set]);
}
