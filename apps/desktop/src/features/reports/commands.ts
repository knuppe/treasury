/**
 * Comandos da barra ⌘K para relatórios, impostos, orçamento e metas (disponíveis em qualquer
 * tela).
 */
import { useMemo } from 'react';
import { useNavigate } from 'react-router';

import { useCountryFeatures } from '@/api/hooks/country';
import { screenMeta, type ScreenId } from '@/app/screens';
import { useRegisterCommands, type AppCommand } from '@/commands/registry';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';
import { monthBounds, monthOf } from '@/lib/months';
import { txnLink } from '@/lib/txnLink';

import { uncategorizedDrill } from './model/drill';
import { filtersSearch, parseFilters, type PresetId, type ViewId } from './model/period';

/** Índice da tela no menu (a ordem das telas mudou desde os primeiros comandos). */
const idx = (id: ScreenId) => screenMeta(id).index;

/** Palavras de busca de uma chave do catálogo (separadas por vírgula). */
const words = (text: string) => text.split(',');

/** Caminho da tela 08 numa visualização e período. */
export function reportPath(view: ViewId, preset: PresetId = 'mes'): string {
  const f = { ...parseFilters('', todayIso()), view, preset };
  return `/relatorios?${filtersSearch(f)}`;
}

type ReportCommandKey =
  | 'fluxo'
  | 'fluxo-trimestre'
  | 'fluxo-anterior'
  | 'composicao'
  | 'ritmo'
  | 'tendencia'
  | 'orcamento'
  | 'comparacao'
  | 'titulares'
  | 'recorrentes'
  | 'cartoes'
  | 'patrimonio'
  | 'inflacao'
  | 'grupos';

function reportCommand(id: string, key: ReportCommandKey, view: ViewId, preset?: PresetId) {
  return {
    id,
    view,
    preset,
    get title() {
      return t(`relatorios-cmd-${key}`);
    },
    get keywords() {
      return words(t(`relatorios-cmd-${key}-busca`));
    },
  };
}

export const REPORT_COMMANDS: readonly {
  id: string;
  title: string;
  view: ViewId;
  preset?: PresetId;
  keywords: string[];
}[] = [
  reportCommand('reports.fluxo', 'fluxo', 'fluxo'),
  reportCommand('reports.fluxo.trimestre', 'fluxo-trimestre', 'fluxo', 'trimestre'),
  reportCommand('reports.fluxo.anterior', 'fluxo-anterior', 'fluxo', 'mes-anterior'),
  reportCommand('reports.composicao', 'composicao', 'composicao'),
  reportCommand('reports.ritmo', 'ritmo', 'ritmo'),
  reportCommand('reports.tendencia', 'tendencia', 'tendencia', '12m'),
  reportCommand('reports.orcamento', 'orcamento', 'orcamento'),
  reportCommand('reports.comparacao', 'comparacao', 'comparacao'),
  reportCommand('reports.titulares', 'titulares', 'titulares'),
  reportCommand('reports.recorrentes', 'recorrentes', 'recorrentes'),
  reportCommand('reports.cartoes', 'cartoes', 'cartoes'),
  reportCommand('reports.patrimonio', 'patrimonio', 'patrimonio'),
  reportCommand('reports.inflacao', 'inflacao', 'inflacao'),
  reportCommand('reports.grupos', 'grupos', 'grupos', '12m'),
];

export function useReportCommands(): void {
  const navigate = useNavigate();
  const features = useCountryFeatures();
  const commands = useMemo<AppCommand[]>(() => {
    const go = (path: string) => () => void navigate(path);
    // IR e DARF só no Brasil; impostos pagos onde há dados do país (docs/countries.md).
    const ir = features.br_income_tax;
    const burden = features.tax_burden;
    const group = t('casca-tela-relatorios');
    return [
      ...REPORT_COMMANDS.map((c, i): AppCommand => ({
        id: c.id,
        title: c.title,
        group,
        prefix: idx('relatorios'),
        keywords: [...words(t('relatorios-cmd-grupo-busca')), ...c.keywords],
        order: i,
        perform: go(reportPath(c.view, c.preset)),
      })),
      ...(ir
        ? [
            {
              id: 'tax.month',
              title: t('relatorios-cmd-ir-mes'),
              group,
              prefix: idx('relatorios'),
              keywords: words(t('relatorios-cmd-ir-mes-busca')),
              order: 40,
              perform: go('/relatorios?v=ir'),
            },
            {
              id: 'tax.year',
              title: t('relatorios-cmd-ir-ano'),
              group,
              prefix: idx('relatorios'),
              keywords: words(t('relatorios-cmd-ir-ano-busca')),
              order: 41,
              perform: go('/relatorios?v=ir&ir=declaracao'),
            },
          ]
        : []),
      ...(burden
        ? [
            {
              id: 'taxburden.estimate',
              title: t('relatorios-cmd-impostos-ano'),
              group,
              prefix: idx('impostos'),
              keywords: words(t('relatorios-cmd-impostos-ano-busca')),
              order: 42,
              perform: go('/impostos'),
            },
            {
              id: 'taxburden.groups',
              title: t('relatorios-cmd-impostos-grupos'),
              group,
              prefix: idx('impostos'),
              keywords: words(t('relatorios-cmd-impostos-grupos-busca')),
              order: 43,
              perform: go('/impostos?grupos=1'),
            },
          ]
        : []),
      {
        id: 'debt.new',
        title: t('relatorios-cmd-divida-nova'),
        group,
        prefix: idx('dividas'),
        keywords: words(t('relatorios-cmd-divida-nova-busca')),
        order: 45,
        perform: go('/dividas?novo=contrato'),
      },
      {
        id: 'physical.new',
        title: t('relatorios-cmd-bem-novo'),
        group,
        prefix: idx('carteira'),
        keywords: words(t('relatorios-cmd-bem-novo-busca')),
        order: 46,
        perform: go('/carteira?aba=bens&novo=bem'),
      },
      {
        id: 'physical.list',
        title: t('relatorios-cmd-bens'),
        group,
        prefix: idx('carteira'),
        keywords: words(t('relatorios-cmd-bens-busca')),
        order: 47,
        perform: go('/carteira?aba=bens'),
      },
      {
        id: 'budget.month',
        title: t('relatorios-cmd-orcamento-mes'),
        group,
        prefix: idx('orcamento'),
        keywords: words(t('relatorios-cmd-orcamento-mes-busca')),
        order: 50,
        perform: go('/orcamento'),
      },
      {
        id: 'goals.list',
        title: t('relatorios-cmd-metas'),
        group,
        prefix: idx('orcamento'),
        keywords: words(t('relatorios-cmd-metas-busca')),
        order: 51,
        perform: go('/orcamento?aba=metas'),
      },
      {
        id: 'goals.reserve',
        title: t('relatorios-cmd-reserva'),
        group,
        prefix: idx('orcamento'),
        keywords: words(t('relatorios-cmd-reserva-busca')),
        order: 52,
        perform: go('/orcamento?aba=reserva'),
      },
      {
        id: 'txn.uncategorized.month',
        title: t('relatorios-cmd-sem-categoria'),
        group,
        prefix: idx('lancamentos'),
        keywords: words(t('relatorios-cmd-sem-categoria-busca')),
        order: 53,
        perform: () => void navigate(txnLink(uncategorizedDrill(monthBounds(monthOf(todayIso()))))),
      },
    ];
  }, [navigate, features]);
  useRegisterCommands(commands);
}
