/**
 * Comandos globais da barra ⌘K (docs/design.md): ir para cada aba e seção das telas,
 * ações rápidas (lançamento, ajuste de saldo, importações, conta, meta, ativo, cofre de
 * demonstração, idioma) e, enquanto a pessoa digita, as contas, categorias, metas, bens e
 * lançamentos recentes do cofre (os ativos e as ações da carteira vêm de
 * features/portfolio/commands.ts).
 *
 * O que só existe no Brasil (IR, DARF, B3, notas de corretagem, análises da CVM, cupons
 * fiscais, Tesouro) some fora dele, pelos recursos do país (`useCountryFeatures`). As
 * palavras de busca vêm do catálogo no idioma do app (sinônimos como holerite/contracheque).
 * "Linha da vida" leva ao quadro da Família só com a função ligada (docs/life-expectancy.md).
 */
import { useMemo } from 'react';
import { useNavigate } from 'react-router';

import type { AccountView } from '@/api/generated/AccountView';
import type { Category } from '@/api/generated/Category';
import type { CountryFeatures } from '@/api/generated/CountryFeatures';
import type { GoalView } from '@/api/generated/GoalView';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import { useCore } from '@/api/hooks/coreQuery';
import { useCountryFeatures } from '@/api/hooks/country';
import { useLongevityEnabled } from '@/api/hooks/longevity';
import { useTxnQuery } from '@/api/hooks/transactions';
import { screenMeta, type ScreenId } from '@/app/screens';
import { useVisibleScreens } from '@/app/useVisibleScreens';
import { useQuickActions } from '@/commands/quickActions';
import { useCommandBar, useRegisterCommands, type AppCommand } from '@/commands/registry';
import type { IconName } from '@/design/components';
import { LOCALES, setLocale, t, useLocale, type AppLocale } from '@/i18n';
import { formatDate, formatMoney, todayIso } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';

/** Texto de uma chave montada na hora (abas, seções e sinônimos desta lista). */
const tk = (key: string): string => (t as unknown as (k: string) => string)(key);

/** Palavras de busca de uma chave do catálogo (separadas por vírgula). */
const words = (text: string) => text.split(',').map((w) => w.trim());

interface Destination {
  id: string;
  screen: ScreenId;
  /** Chave do rótulo da aba ou seção. */
  label: string;
  path: string;
  /** Chave com sinônimos extras (separados por vírgula). */
  synonyms?: string;
  /** Só aparece quando o país tem o recurso. */
  requires?: keyof CountryFeatures;
}

/** Abas e seções de cada tela, na ordem do menu. */
export const DESTINATIONS: readonly Destination[] = [
  {
    id: 'familia.renda',
    screen: 'familia',
    label: 'casca-cmd-familia-renda',
    path: '/familia',
    synonyms: 'casca-cmd-sin-holerite',
  },
  {
    id: 'lancamentos.recorrencias',
    screen: 'lancamentos',
    label: 'lancamentos-aba-recorrencias',
    path: '/lancamentos?aba=recorrencias',
  },
  {
    id: 'lancamentos.regras',
    screen: 'lancamentos',
    label: 'lancamentos-aba-regras',
    path: '/lancamentos?aba=regras',
  },
  {
    id: 'lancamentos.importacoes',
    screen: 'lancamentos',
    label: 'lancamentos-aba-importacoes',
    path: '/lancamentos?aba=importacoes',
    synonyms: 'casca-cmd-sin-extrato',
  },
  {
    id: 'lancamentos.email',
    screen: 'lancamentos',
    label: 'lancamentos-aba-email',
    path: '/lancamentos?aba=email',
  },
  {
    id: 'consumo.compras',
    screen: 'consumo',
    label: 'alimentacao-aba-compras',
    path: '/consumo?aba=compras',
    synonyms: 'casca-cmd-sin-cupom',
  },
  {
    id: 'orcamento.metas',
    screen: 'orcamento',
    label: 'orcamento-aba-metas',
    path: '/orcamento?aba=metas',
  },
  {
    id: 'orcamento.projetos',
    screen: 'orcamento',
    label: 'orcamento-aba-projetos',
    path: '/orcamento?aba=projetos',
  },
  {
    id: 'orcamento.reserva',
    screen: 'orcamento',
    label: 'orcamento-aba-reserva',
    path: '/orcamento?aba=reserva',
  },
  {
    id: 'orcamento.aposentadoria',
    screen: 'orcamento',
    label: 'orcamento-aba-aposentadoria',
    path: '/orcamento?aba=aposentadoria',
    synonyms: 'casca-cmd-sin-aposentadoria',
  },
  {
    id: 'orcamento.assinaturas',
    screen: 'orcamento',
    label: 'orcamento-aba-assinaturas',
    path: '/orcamento?aba=assinaturas',
  },
  {
    id: 'calculadoras.juros',
    screen: 'calculadoras',
    label: 'calc-aba-juros',
    path: '/calculadoras',
  },
  {
    id: 'calculadoras.renda',
    screen: 'calculadoras',
    label: 'calc-aba-renda',
    path: '/calculadoras?aba=renda',
  },
  {
    id: 'calculadoras.parcelado',
    screen: 'calculadoras',
    label: 'calc-aba-parcelado',
    path: '/calculadoras?aba=parcelado',
  },
  {
    id: 'calculadoras.aluguel',
    screen: 'calculadoras',
    label: 'calc-aba-aluguel',
    path: '/calculadoras?aba=aluguel',
  },
  {
    id: 'orcamento.cenarios',
    screen: 'orcamento',
    label: 'orcamento-aba-cenarios',
    path: '/orcamento?aba=cenarios',
  },
  {
    id: 'carteira.rentabilidade',
    screen: 'carteira',
    label: 'carteira-aba-rentabilidade',
    path: '/carteira?aba=rentabilidade',
  },
  {
    id: 'carteira.alocacao',
    screen: 'carteira',
    label: 'carteira-aba-alocacao',
    path: '/carteira?aba=alocacao',
  },
  {
    id: 'carteira.operacoes',
    screen: 'carteira',
    label: 'carteira-aba-operacoes',
    path: '/carteira?aba=operacoes',
  },
  {
    id: 'carteira.proventos',
    screen: 'carteira',
    label: 'carteira-aba-proventos',
    path: '/carteira?aba=proventos',
    synonyms: 'casca-cmd-sin-proventos',
  },
  {
    id: 'carteira.renda',
    screen: 'carteira',
    label: 'carteira-aba-renda',
    path: '/carteira?aba=renda',
  },
  {
    id: 'carteira.taxas',
    screen: 'carteira',
    label: 'carteira-aba-taxas',
    path: '/carteira?aba=taxas',
  },
  {
    id: 'carteira.risco',
    screen: 'carteira',
    label: 'carteira-aba-risco',
    path: '/carteira?aba=risco',
  },
  {
    id: 'carteira.saude',
    screen: 'carteira',
    label: 'carteira-aba-saude',
    path: '/carteira?aba=saude',
  },
  {
    id: 'carteira.dados',
    screen: 'carteira',
    label: 'dados-cmd-completar',
    path: '/carteira?aba=saude&secao=dados',
    synonyms: 'dados-cmd-sin',
  },
  {
    id: 'carteira.bens',
    screen: 'carteira',
    label: 'carteira-aba-bens',
    path: '/carteira?aba=bens',
    synonyms: 'casca-cmd-sin-bens',
  },
  {
    id: 'ativos.indices',
    screen: 'ativos',
    label: 'ativos-aba-indices',
    path: '/ativos?aba=indices',
    synonyms: 'casca-cmd-sin-indices',
  },
  {
    id: 'ativos.indicadores',
    screen: 'ativos',
    label: 'ativos-aba-indicadores',
    path: '/ativos?aba=indicadores',
    requires: 'br_market',
  },
  { id: 'ativos.bolsas', screen: 'ativos', label: 'ativos-aba-bolsas', path: '/ativos?aba=bolsas' },
  {
    id: 'agente.aprovacao',
    screen: 'agente',
    label: 'agente-aba-aprovacao',
    path: '/agente?aba=aprovacao',
  },
  {
    id: 'agente.alertas',
    screen: 'agente',
    label: 'agente-aba-alertas',
    path: '/agente?aba=alertas',
  },
  {
    id: 'agente.ferramentas',
    screen: 'agente',
    label: 'agente-aba-ferramentas',
    path: '/agente?aba=ferramentas',
  },
  {
    id: 'agente.memoria',
    screen: 'agente',
    label: 'agente-aba-memoria',
    path: '/agente?aba=memoria',
  },
  {
    id: 'agente.configurar',
    screen: 'agente',
    label: 'agente-aba-configurar',
    path: '/agente?aba=configurar',
    synonyms: 'casca-cmd-sin-motores',
  },
  ...(
    [
      'perfil',
      'familia',
      'preferencias',
      'notificacoes',
      'categorias',
      'saude',
      'seguranca',
      'cofres',
      'backups',
      'historico',
      'dados',
      'banco',
      'fontes',
      'ia',
      'mcp',
      'atualizacoes',
      'sobre',
    ] as const
  ).map((s): Destination => ({
    id: `ajustes.${s}`,
    screen: 'ajustes',
    label: `ajustes-secao-${s}`,
    path: `/ajustes?secao=${s}`,
    synonyms: `ajustes-secao-${s}-busca`,
  })),
];

interface QuickAction {
  id: string;
  label: string;
  icon: IconName;
  synonyms: string;
  path?: string;
  run?: () => void;
  shortcut?: readonly string[];
  requires?: keyof CountryFeatures;
}

/** Sem resultado de busca, as entidades ficam de fora (só aparecem com 2 letras ou mais). */
const ENTITY_LIMIT = 40;

export function useGlobalCommands(): void {
  const navigate = useNavigate();
  const features = useCountryFeatures();
  const visible = useVisibleScreens();
  const locale = useLocale();
  const open = useCommandBar((s) => s.open);
  const openDemo = useQuickActions((s) => s.openDemo);
  const longevity = useLongevityEnabled();

  // Entidades do cofre: só carregam com a barra aberta.
  const accounts = useCore<AccountView[]>(
    'accounts',
    'list',
    { include_archived: false },
    { enabled: open },
  );
  const categories = useCore<Category[]>('categories', 'list', {}, { enabled: open });
  const goals = useCore<GoalView[]>('goals', 'list', {}, { enabled: open });
  const physical = useCore<PhysicalAssetView[]>('physical', 'list', {}, { enabled: open });
  const recent = useTxnQuery({ date_to: todayIso(), limit: ENTITY_LIMIT }, open);

  const nav = useMemo<AppCommand[]>(() => {
    const shown = new Set(visible.map((s) => s.id));
    return DESTINATIONS.filter(
      (d) => shown.has(d.screen) && (!d.requires || features[d.requires]),
    ).map((d, i): AppCommand => {
      const screen = screenMeta(d.screen);
      const label = tk(d.label);
      return {
        id: `go.${d.id}`,
        title: t('casca-cmd-ir-aba', { tela: screen.title, aba: label }),
        prefix: screen.index,
        group: t('casca-cmd-grupo-ir'),
        keywords: [
          label,
          screen.title,
          ...words(t('casca-cmd-ir-para')),
          ...(d.synonyms ? words(tk(d.synonyms)) : []),
        ],
        order: i,
        perform: () => void navigate(d.path),
      };
    });
  }, [visible, features, navigate]);

  // Linha da vida: só com a função ligada, e o quadro fica na Família.
  const life = useMemo<AppCommand[]>(() => {
    if (!longevity || !visible.some((s) => s.id === 'familia')) return [];
    const screen = screenMeta('familia');
    const label = t('longevidade-titulo');
    return [
      {
        id: 'go.familia.linha-da-vida',
        title: t('casca-cmd-ir-aba', { tela: screen.title, aba: label }),
        prefix: screen.index,
        group: t('casca-cmd-grupo-ir'),
        keywords: [
          label,
          screen.title,
          ...words(t('casca-cmd-ir-para')),
          ...words(t('longevidade-cmd-busca')),
        ],
        order: DESTINATIONS.length,
        perform: () => void navigate('/familia?ver=linha-da-vida'),
      },
    ];
  }, [longevity, visible, navigate]);

  const actions = useMemo<AppCommand[]>(() => {
    const list: QuickAction[] = [
      {
        id: 'txn.expense',
        label: 'casca-cmd-novo-lancamento',
        icon: 'plus',
        synonyms: 'casca-cmd-novo-lancamento-busca',
        path: '/lancamentos?novo=despesa',
      },
      {
        id: 'txn.income',
        label: 'casca-cmd-nova-receita',
        icon: 'plus',
        synonyms: 'casca-cmd-nova-receita-busca',
        path: '/lancamentos?novo=receita',
      },
      {
        id: 'txn.transfer',
        label: 'casca-cmd-nova-transferencia',
        icon: 'arrowRight',
        synonyms: 'casca-cmd-nova-transferencia-busca',
        path: '/lancamentos?novo=transferencia',
      },
      {
        id: 'txn.installments',
        label: 'casca-cmd-compra-parcelada',
        icon: 'plus',
        synonyms: 'casca-cmd-compra-parcelada-busca',
        path: '/lancamentos?novo=parcelado',
      },
      {
        id: 'txn.adjustment',
        label: 'casca-cmd-ajuste-saldo',
        icon: 'refresh',
        synonyms: 'casca-cmd-ajuste-saldo-busca',
        path: '/lancamentos?novo=ajuste',
      },
      {
        id: 'import.payslip',
        label: 'casca-cmd-importar-holerite',
        icon: 'clip',
        synonyms: 'casca-cmd-sin-holerite',
        path: '/familia',
      },
      {
        id: 'account.new',
        label: 'casca-cmd-nova-conta',
        icon: 'plus',
        synonyms: 'casca-cmd-nova-conta-busca',
        path: '/contas?nova=conta',
      },
      {
        id: 'card.new',
        label: 'casca-cmd-novo-cartao',
        icon: 'plus',
        synonyms: 'casca-cmd-novo-cartao-busca',
        path: '/contas?nova=cartao',
      },
      {
        id: 'goal.new',
        label: 'casca-cmd-nova-meta',
        icon: 'plus',
        synonyms: 'casca-cmd-nova-meta-busca',
        path: '/orcamento?aba=metas&novo=meta',
      },
      {
        id: 'vault.switch',
        label: 'casca-cmd-trocar-cofre',
        icon: 'key',
        synonyms: 'casca-cmd-trocar-cofre-busca',
        path: '/ajustes?secao=cofres',
      },
      {
        id: 'vault.demo',
        label: 'casca-cmd-cofre-demo',
        icon: 'sliders',
        synonyms: 'casca-cmd-cofre-demo-busca',
        run: openDemo,
      },
    ];
    return list
      .filter((a) => !a.requires || features[a.requires])
      .map((a, i): AppCommand => ({
        id: `action.${a.id}`,
        title: tk(a.label),
        group: t('casca-cmd-grupo-acoes'),
        icon: a.icon,
        shortcut: a.shortcut,
        keywords: words(tk(a.synonyms)),
        order: i,
        perform: () => (a.run ? a.run() : void navigate(a.path!)),
      }));
  }, [features, navigate, openDemo]);

  const languages = useMemo<AppCommand[]>(
    () =>
      LOCALES.filter((l) => l !== locale).map((l: AppLocale, i): AppCommand => ({
        id: `ui.locale.${l}`,
        title: t('casca-cmd-idioma', { idioma: tk(`comum-idioma-${l}`) }),
        group: t('casca-cmd-grupo-exibicao'),
        icon: 'sliders',
        keywords: words(t('casca-cmd-idioma-busca')),
        order: 30 + i,
        perform: () => void setLocale(l),
      })),
    [locale],
  );

  const entities = useMemo<AppCommand[]>(() => {
    const out: AppCommand[] = [];
    const contas = screenMeta('contas');
    for (const a of accounts.data ?? []) {
      out.push({
        id: `entity.account.${a.id}`,
        title: a.name,
        group: t('casca-cmd-grupo-contas'),
        prefix: contas.index,
        hint: formatMoney(a.balance, a.currency),
        keywords: [a.institution_name ?? '', ...words(t('casca-cmd-entidade-conta-busca'))],
        searchOnly: true,
        perform: () => void navigate(`/contas?conta=${a.id}`),
      });
    }
    for (const c of (categories.data ?? []).filter((x) => !x.archived)) {
      out.push({
        id: `entity.category.${c.id}`,
        title: c.name,
        group: t('casca-cmd-grupo-categorias'),
        prefix: screenMeta('lancamentos').index,
        keywords: words(t('casca-cmd-entidade-categoria-busca')),
        searchOnly: true,
        perform: () => void navigate(txnLink({ category_ids: [c.id] })),
      });
    }
    for (const g of (goals.data ?? []).filter((x) => !x.archived)) {
      out.push({
        id: `entity.goal.${g.id}`,
        title: g.name,
        group: t('casca-cmd-grupo-metas'),
        prefix: screenMeta('orcamento').index,
        keywords: words(t('casca-cmd-entidade-meta-busca')),
        searchOnly: true,
        perform: () => void navigate('/orcamento?aba=metas'),
      });
    }
    for (const p of physical.data ?? []) {
      out.push({
        id: `entity.physical.${p.id}`,
        title: p.name,
        group: t('casca-cmd-grupo-bens'),
        prefix: screenMeta('carteira').index,
        keywords: words(t('casca-cmd-sin-bens')),
        searchOnly: true,
        perform: () => void navigate(`/carteira?aba=bens&bem=${p.id}`),
      });
    }
    for (const x of recent.data?.items ?? []) {
      out.push({
        id: `entity.txn.${x.id}`,
        title: x.description,
        group: t('casca-cmd-grupo-lancamentos'),
        prefix: screenMeta('lancamentos').index,
        hint: `${formatDate(x.date)} · ${formatMoney(x.amount, x.currency)}`,
        // Pela descrição e pelo favorecido (pela conta, a lista enchia de lançamentos sem relação).
        keywords: [x.payee ?? ''],
        searchOnly: true,
        perform: () => void navigate(txnLink({ text: x.description })),
      });
    }
    return out;
  }, [accounts.data, categories.data, goals.data, physical.data, recent.data, navigate]);

  const commands = useMemo(
    () => [...nav, ...life, ...actions, ...languages, ...entities],
    [nav, life, actions, languages, entities],
  );
  useRegisterCommands(commands);
}
