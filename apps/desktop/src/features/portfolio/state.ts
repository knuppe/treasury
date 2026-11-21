/**
 * Preferências de exibição da carteira (só interface, em memória): deflator do
 * resultado real, agrupamento das posições, posições encerradas, o período de cada aba e as
 * séries escolhidas para comparar com a rentabilidade.
 */
import { create } from 'zustand';

import type { BenchmarkTarget } from '@/api/generated/BenchmarkTarget';
import type { Deflator } from '@/api/generated/Deflator';

import type { PeriodKey } from './components/PeriodSelect';
import type { GroupMode } from './transform';

/** Abas com o seletor de período padronizado (cada uma guarda a sua escolha). */
export type PeriodScope = 'rentabilidade' | 'risco' | 'ponte' | 'proventos' | 'vendas';

interface PortfolioUiState {
  deflator: Deflator;
  groupMode: GroupMode;
  includeClosed: boolean;
  periods: Partial<Record<PeriodScope, PeriodKey>>;
  compare: BenchmarkTarget[];
  setDeflator: (d: Deflator) => void;
  setPeriod: (scope: PeriodScope, key: PeriodKey) => void;
  setGroupMode: (g: GroupMode) => void;
  setIncludeClosed: (v: boolean) => void;
  setCompare: (c: BenchmarkTarget[]) => void;
}

export const PORTFOLIO_UI_DEFAULTS = {
  deflator: 'ipca' as Deflator,
  groupMode: 'class' as GroupMode,
  includeClosed: false,
  periods: {} as Partial<Record<PeriodScope, PeriodKey>>,
  compare: [] as BenchmarkTarget[],
};

export const usePortfolioUi = create<PortfolioUiState>()((set) => ({
  ...PORTFOLIO_UI_DEFAULTS,
  setDeflator: (deflator) => set({ deflator }),
  setGroupMode: (groupMode) => set({ groupMode }),
  setIncludeClosed: (includeClosed) => set({ includeClosed }),
  setCompare: (compare) => set({ compare }),
  setPeriod: (scope, key) => set((s) => ({ periods: { ...s.periods, [scope]: key } })),
}));
