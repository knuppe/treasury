/**
 * Registro único das telas do app: índice, título, rota e o módulo de cada uma.
 *
 * A barra lateral, as rotas, os atalhos (⌘1…⌘0) e a barra de comando saem
 * daqui. Para preencher uma tela, edite o arquivo dela em `src/screens/`; para
 * criar uma nova, acrescente uma entrada nesta lista.
 */
import { lazy, type ComponentType, type LazyExoticComponent } from 'react';

import type { CountryFeatures } from '@/api/generated/CountryFeatures';
import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import { t } from '@/i18n';

export type ScreenGroup = 'Fluxo de caixa' | 'Investimentos' | 'Gestão';

export interface ScreenDef {
  id: string;
  /** Índice de dois dígitos (`03`). */
  index: string;
  /** Título em caixa normal (a caixa alta vem do CSS), no idioma atual. */
  readonly title: string;
  /** Rota (hash): `/lancamentos`. */
  path: string;
  group: ScreenGroup;
  /** Uma linha sobre o que a tela faz (usada no cabeçalho e na barra de comando). */
  readonly summary: string;
  /** Palavras extras para a busca da barra de comando. */
  readonly keywords: readonly string[];
  /** Só aparece no menu e na barra de comando com o modo desenvolvedor ligado. */
  devOnly?: boolean;
  /** Nível a partir do qual a tela fica no menu principal; abaixo dele, vai para "Mais telas". */
  minLevel?: ProfileLevel;
  /** Só aparece quando o país da família tem este recurso (docs/countries.md). */
  requires?: keyof CountryFeatures;
  load: () => Promise<{ default: ComponentType }>;
}

export const SCREENS = [
  {
    id: 'painel',
    index: '01',
    get title() {
      return t('casca-tela-painel');
    },
    path: '/painel',
    group: 'Fluxo de caixa',
    get summary() {
      return t('casca-tela-painel-resumo');
    },
    get keywords() {
      return t('casca-tela-painel-busca').split(',');
    },
    load: () => import('@/screens/Painel'),
  },
  {
    id: 'familia',
    index: '02',
    get title() {
      return t('casca-tela-familia');
    },
    path: '/familia',
    group: 'Fluxo de caixa',
    get summary() {
      return t('casca-tela-familia-resumo');
    },
    get keywords() {
      return t('casca-tela-familia-busca').split(',');
    },
    load: () => import('@/screens/Familia'),
  },
  {
    id: 'contas',
    index: '03',
    get title() {
      return t('casca-tela-contas');
    },
    path: '/contas',
    group: 'Fluxo de caixa',
    get summary() {
      return t('casca-tela-contas-resumo');
    },
    get keywords() {
      return t('casca-tela-contas-busca').split(',');
    },
    load: () => import('@/screens/Contas'),
  },
  {
    id: 'lancamentos',
    index: '04',
    get title() {
      return t('casca-tela-lancamentos');
    },
    path: '/lancamentos',
    group: 'Fluxo de caixa',
    get summary() {
      return t('casca-tela-lancamentos-resumo');
    },
    get keywords() {
      return t('casca-tela-lancamentos-busca').split(',');
    },
    load: () => import('@/screens/Lancamentos'),
  },
  {
    id: 'orcamento',
    index: '05',
    get title() {
      return t('casca-tela-orcamento');
    },
    path: '/orcamento',
    group: 'Fluxo de caixa',
    get summary() {
      return t('casca-tela-orcamento-resumo');
    },
    get keywords() {
      return t('casca-tela-orcamento-busca').split(',');
    },
    load: () => import('@/screens/Orcamento'),
  },
  {
    id: 'consumo',
    index: '06',
    get title() {
      return t('casca-tela-consumo');
    },
    path: '/consumo',
    group: 'Fluxo de caixa',
    get summary() {
      return t('casca-tela-consumo-resumo');
    },
    get keywords() {
      return t('casca-tela-consumo-busca').split(',');
    },
    // Em todos os países; só a leitura da NFC-e é do Brasil (docs/consumption.md).
    load: () => import('@/screens/Consumo'),
  },
  {
    id: 'dividas',
    index: '07',
    get title() {
      return t('casca-tela-dividas');
    },
    path: '/dividas',
    group: 'Fluxo de caixa',
    get summary() {
      return t('casca-tela-dividas-resumo');
    },
    get keywords() {
      return t('casca-tela-dividas-busca').split(',');
    },
    load: () => import('@/screens/Dividas'),
  },
  {
    id: 'carteira',
    minLevel: 'avancado',
    index: '08',
    get title() {
      return t('casca-tela-carteira');
    },
    path: '/carteira',
    group: 'Investimentos',
    get summary() {
      return t('casca-tela-carteira-resumo');
    },
    get keywords() {
      return t('casca-tela-carteira-busca').split(',');
    },
    load: () => import('@/screens/Carteira'),
  },
  {
    id: 'ativos',
    minLevel: 'avancado',
    index: '09',
    get title() {
      return t('casca-tela-ativos');
    },
    path: '/ativos',
    group: 'Investimentos',
    get summary() {
      return t('casca-tela-ativos-resumo');
    },
    get keywords() {
      return t('casca-tela-ativos-busca').split(',');
    },
    load: () => import('@/screens/Ativos'),
  },
  {
    id: 'mercado',
    minLevel: 'avancado',
    requires: 'br_market',
    index: '10',
    get title() {
      return t('casca-tela-mercado');
    },
    path: '/mercado',
    group: 'Investimentos',
    get summary() {
      return t('casca-tela-mercado-resumo');
    },
    get keywords() {
      return t('casca-tela-mercado-busca').split(',');
    },
    load: () => import('@/screens/Mercado'),
  },
  {
    id: 'relatorios',
    minLevel: 'intermediario',
    index: '11',
    get title() {
      return t('casca-tela-relatorios');
    },
    path: '/relatorios',
    group: 'Gestão',
    get summary() {
      return t('casca-tela-relatorios-resumo');
    },
    get keywords() {
      return t('casca-tela-relatorios-busca').split(',');
    },
    load: () => import('@/screens/Relatorios'),
  },
  {
    id: 'impostos',
    minLevel: 'avancado',
    requires: 'tax_burden',
    index: '12',
    get title() {
      return t('casca-tela-impostos');
    },
    path: '/impostos',
    group: 'Gestão',
    get summary() {
      return t('casca-tela-impostos-resumo');
    },
    get keywords() {
      return t('casca-tela-impostos-busca').split(',');
    },
    load: () => import('@/screens/Impostos'),
  },
  {
    id: 'agente',
    index: '13',
    get title() {
      return t('casca-tela-agente');
    },
    path: '/agente',
    group: 'Gestão',
    get summary() {
      return t('casca-tela-agente-resumo');
    },
    get keywords() {
      return t('casca-tela-agente-busca').split(',');
    },
    load: () => import('@/screens/Agente'),
  },
  {
    id: 'ajustes',
    index: '14',
    get title() {
      return t('casca-tela-ajustes');
    },
    path: '/ajustes',
    group: 'Gestão',
    get summary() {
      return t('casca-tela-ajustes-resumo');
    },
    get keywords() {
      return t('casca-tela-ajustes-busca').split(',');
    },
    load: () => import('@/screens/Ajustes'),
  },
  {
    id: 'desenvolvedor',
    index: '15',
    get title() {
      return t('casca-tela-desenvolvedor');
    },
    path: '/desenvolvedor',
    group: 'Gestão',
    get summary() {
      return t('casca-tela-desenvolvedor-resumo');
    },
    get keywords() {
      return t('casca-tela-desenvolvedor-busca').split(',');
    },
    devOnly: true,
    load: () => import('@/screens/Desenvolvedor'),
  },
  {
    id: 'calculadoras',
    index: '16',
    get title() {
      return t('casca-tela-calculadoras');
    },
    path: '/calculadoras',
    group: 'Gestão',
    get summary() {
      return t('casca-tela-calculadoras-resumo');
    },
    get keywords() {
      return t('casca-tela-calculadoras-busca').split(',');
    },
    load: () => import('@/screens/Calculadoras'),
  },
] as const satisfies readonly ScreenDef[];

export type ScreenId = (typeof SCREENS)[number]['id'];

export const SCREEN_GROUPS: readonly ScreenGroup[] = ['Fluxo de caixa', 'Investimentos', 'Gestão'];

/** Nome do grupo no idioma atual. */
export function groupLabel(group: ScreenGroup): string {
  return group === 'Fluxo de caixa'
    ? t('casca-grupo-fluxo')
    : group === 'Investimentos'
      ? t('casca-grupo-investimentos')
      : t('casca-grupo-gestao');
}

export const DEFAULT_SCREEN: ScreenId = 'painel';

/** A tela aparece no menu e na barra de comando? (as `devOnly`, só no modo desenvolvedor; as
 * que dependem do país, só onde ele tem o recurso) */
export function screenVisible(
  screen: ScreenDef,
  developerMode: boolean,
  features?: CountryFeatures,
): boolean {
  if (screen.requires && features && !features[screen.requires]) return false;
  return !screen.devOnly || developerMode;
}

export function screenMeta(id: ScreenId): ScreenDef {
  return SCREENS.find((s) => s.id === id)!;
}

/** Componentes carregados sob demanda (uma vez por tela). */
export const SCREEN_COMPONENTS: ReadonlyMap<string, LazyExoticComponent<ComponentType>> = new Map(
  SCREENS.map((s) => [s.id, lazy(s.load)]),
);

/** Teclas do atalho de cada tela: ⌘1 … ⌘9 e ⌘0 para a décima (as seguintes ficam sem atalho). */
export function screenShortcut(position: number): readonly string[] | undefined {
  if (position < 0 || position > 9) return undefined;
  return ['mod', String((position + 1) % 10)];
}

/** A tela Agente: o chat dela é o principal e o agente companheiro não aparece. */
export function isAgentScreen(pathname: string): boolean {
  return pathname === '/agente' || pathname.startsWith('/agente/');
}
