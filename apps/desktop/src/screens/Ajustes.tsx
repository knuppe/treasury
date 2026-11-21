/**
 * Ajustes. As seções ficam em cinco grupos ("Você e a família", "Organização do
 * dinheiro", "Segurança e dados", "Integrações", "O Treasury"), com busca por nome, descrição e
 * palavras-chave. Sem seção escolhida, a visão geral mostra todos os grupos em cartões,
 * com o estado do que pede atenção (backup atrasado). `?secao=<id>` abre uma seção; os
 * ids antigos continuam valendo.
 */
import { useState, type KeyboardEvent } from 'react';
import { useSearchParams } from 'react-router';

import { useBackupHealth } from '@/api/hooks/backup';
import { screenMeta } from '@/app/screens';
import { normalizeSearch } from '@/commands/registry';
import { Icon, ScreenHeader, type IconName } from '@/design/components';
import { cx } from '@/design/cx';
import { AiSettingsSection } from '@/features/agent/settings';
import { usePageContext } from '@/features/companion/pageContext';
import { HealthSettingsSection } from '@/features/health/HealthSettingsSection';
import { BackupsSection } from '@/features/settings/BackupsSection';
import { CategoriesSection } from '@/features/settings/CategoriesSection';
import { DatabaseSection } from '@/features/settings/DatabaseSection';
import { FamilySection } from '@/features/settings/FamilySection';
import {
  DataSection,
  DataSourcesSection,
  McpSection,
  PreferencesSection,
} from '@/features/settings/GeneralSections';
import { HistorySection } from '@/features/settings/HistorySection';
import { NotificationsSection } from '@/features/settings/NotificationsSection';
import { ProfileSection } from '@/features/settings/ProfileSection';
import { SecuritySection } from '@/features/settings/SecuritySection';
import { UpdatesSection } from '@/features/settings/UpdatesSection';
import { VaultsSection } from '@/features/settings/VaultsSection';
import { SupportPage } from '@/features/support/SupportPage';
import { t } from '@/i18n';

const meta = screenMeta('ajustes');

const GROUPS = [
  {
    id: 'voce',
    get label() {
      return t('ajustes-grupo-voce');
    },
    get hint() {
      return t('ajustes-grupo-voce-dica');
    },
  },
  {
    id: 'dinheiro',
    get label() {
      return t('ajustes-grupo-dinheiro');
    },
    get hint() {
      return t('ajustes-grupo-dinheiro-dica');
    },
  },
  {
    id: 'seguranca',
    get label() {
      return t('ajustes-grupo-seguranca');
    },
    get hint() {
      return t('ajustes-grupo-seguranca-dica');
    },
  },
  {
    id: 'integracoes',
    get label() {
      return t('ajustes-grupo-integracoes');
    },
    get hint() {
      return t('ajustes-grupo-integracoes-dica');
    },
  },
  {
    id: 'treasury',
    get label() {
      return t('ajustes-grupo-treasury');
    },
    get hint() {
      return t('ajustes-grupo-treasury-dica');
    },
  },
] as const;

type GroupId = (typeof GROUPS)[number]['id'];

interface SectionDef {
  id: string;
  group: GroupId;
  readonly label: string;
  readonly hint: string;
  icon: IconName;
  readonly keywords: readonly string[];
}

/** Seções (rótulo, descrição e palavras de busca vêm do catálogo, no idioma atual). */
const SECTIONS = [
  {
    id: 'perfil',
    group: 'voce',
    get label() {
      return t('ajustes-secao-perfil');
    },
    get hint() {
      return t('ajustes-secao-perfil-dica');
    },
    icon: 'info',
    get keywords() {
      return t('ajustes-secao-perfil-busca').split(',');
    },
  },
  {
    id: 'familia',
    group: 'voce',
    get label() {
      return t('ajustes-secao-familia');
    },
    get hint() {
      return t('ajustes-secao-familia-dica');
    },
    icon: 'eye',
    get keywords() {
      return t('ajustes-secao-familia-busca').split(',');
    },
  },
  {
    id: 'preferencias',
    group: 'voce',
    get label() {
      return t('ajustes-secao-preferencias');
    },
    get hint() {
      return t('ajustes-secao-preferencias-dica');
    },
    icon: 'sliders',
    get keywords() {
      return t('ajustes-secao-preferencias-busca').split(',');
    },
  },
  {
    id: 'notificacoes',
    group: 'voce',
    get label() {
      return t('ajustes-secao-notificacoes');
    },
    get hint() {
      return t('ajustes-secao-notificacoes-dica');
    },
    icon: 'alert',
    get keywords() {
      return t('ajustes-secao-notificacoes-busca').split(',');
    },
  },
  {
    id: 'categorias',
    group: 'dinheiro',
    get label() {
      return t('ajustes-secao-categorias');
    },
    get hint() {
      return t('ajustes-secao-categorias-dica');
    },
    icon: 'reticleDashed',
    get keywords() {
      return t('ajustes-secao-categorias-busca').split(',');
    },
  },
  {
    id: 'saude',
    group: 'dinheiro',
    get label() {
      return t('ajustes-secao-saude');
    },
    get hint() {
      return t('ajustes-secao-saude-dica');
    },
    icon: 'activity',
    get keywords() {
      return t('ajustes-secao-saude-busca').split(',');
    },
  },
  {
    id: 'seguranca',
    group: 'seguranca',
    get label() {
      return t('ajustes-secao-seguranca');
    },
    get hint() {
      return t('ajustes-secao-seguranca-dica');
    },
    icon: 'shield',
    get keywords() {
      return t('ajustes-secao-seguranca-busca').split(',');
    },
  },
  {
    id: 'cofres',
    group: 'seguranca',
    get label() {
      return t('ajustes-secao-cofres');
    },
    get hint() {
      return t('ajustes-secao-cofres-dica');
    },
    icon: 'lock',
    get keywords() {
      return t('ajustes-secao-cofres-busca').split(',');
    },
  },
  {
    id: 'backups',
    group: 'seguranca',
    get label() {
      return t('ajustes-secao-backups');
    },
    get hint() {
      return t('ajustes-secao-backups-dica');
    },
    icon: 'refresh',
    get keywords() {
      return t('ajustes-secao-backups-busca').split(',');
    },
  },
  {
    id: 'historico',
    group: 'seguranca',
    get label() {
      return t('ajustes-secao-historico');
    },
    get hint() {
      return t('ajustes-secao-historico-dica');
    },
    icon: 'history',
    get keywords() {
      return t('ajustes-secao-historico-busca').split(',');
    },
  },
  {
    id: 'dados',
    group: 'seguranca',
    get label() {
      return t('ajustes-secao-dados');
    },
    get hint() {
      return t('ajustes-secao-dados-dica');
    },
    icon: 'copy',
    get keywords() {
      return t('ajustes-secao-dados-busca').split(',');
    },
  },
  {
    id: 'banco',
    group: 'seguranca',
    get label() {
      return t('ajustes-secao-banco');
    },
    get hint() {
      return t('ajustes-secao-banco-dica');
    },
    icon: 'key',
    get keywords() {
      return t('ajustes-secao-banco-busca').split(',');
    },
  },
  {
    id: 'fontes',
    group: 'integracoes',
    get label() {
      return t('ajustes-secao-fontes');
    },
    get hint() {
      return t('ajustes-secao-fontes-dica');
    },
    icon: 'arrowDown',
    get keywords() {
      return t('ajustes-secao-fontes-busca').split(',');
    },
  },
  {
    id: 'ia',
    group: 'integracoes',
    get label() {
      return t('ajustes-secao-ia');
    },
    get hint() {
      return t('ajustes-secao-ia-dica');
    },
    icon: 'agent',
    get keywords() {
      return t('ajustes-secao-ia-busca').split(',');
    },
  },
  {
    id: 'mcp',
    group: 'integracoes',
    get label() {
      return t('ajustes-secao-mcp');
    },
    get hint() {
      return t('ajustes-secao-mcp-dica');
    },
    icon: 'pin',
    get keywords() {
      return t('ajustes-secao-mcp-busca').split(',');
    },
  },
  {
    id: 'atualizacoes',
    group: 'treasury',
    get label() {
      return t('ajustes-secao-atualizacoes');
    },
    get hint() {
      return t('ajustes-secao-atualizacoes-dica');
    },
    icon: 'refresh',
    get keywords() {
      return t('ajustes-secao-atualizacoes-busca').split(',');
    },
  },
  {
    id: 'sobre',
    group: 'treasury',
    get label() {
      return t('ajustes-secao-sobre');
    },
    get hint() {
      return t('ajustes-secao-sobre-dica');
    },
    icon: 'info',
    get keywords() {
      return t('ajustes-secao-sobre-busca').split(',');
    },
  },
] as const satisfies readonly SectionDef[];

type SectionId = (typeof SECTIONS)[number]['id'];

const RENDER: Record<SectionId, () => React.JSX.Element> = {
  seguranca: SecuritySection,
  cofres: VaultsSection,
  backups: BackupsSection,
  historico: HistorySection,
  preferencias: PreferencesSection,
  perfil: ProfileSection,
  notificacoes: NotificationsSection,
  saude: HealthSettingsSection,
  fontes: DataSourcesSection,
  dados: DataSection,
  banco: DatabaseSection,
  ia: AiSection,
  mcp: McpSection,
  categorias: CategoriesSection,
  familia: FamilySection,
  atualizacoes: UpdatesSection,
  sobre: SupportPage,
};

function AiSection() {
  return <AiSettingsSection heading={null} withMemory />;
}

/** Seções que casam com a busca (nome, descrição, grupo ou palavras-chave). */
export function matchSections(query: string): SectionId[] {
  const q = normalizeSearch(query.trim());
  if (!q) return SECTIONS.map((s) => s.id);
  const words = q.split(/\s+/);
  return SECTIONS.filter((s) => {
    const group = GROUPS.find((g) => g.id === s.group)!;
    const hay = normalizeSearch([s.label, s.hint, group.label, ...s.keywords].join(' '));
    return words.every((w) => hay.includes(w));
  }).map((s) => s.id);
}

export default function Ajustes() {
  const [params, setParams] = useSearchParams();
  const current = SECTIONS.find((s) => s.id === params.get('secao'));
  const [query, setQuery] = useState('');
  const health = useBackupHealth();
  const go = (id: SectionId | null) => {
    setQuery('');
    setParams(id ? { secao: id } : {});
  };
  usePageContext({
    screen: 'ajustes',
    title: `${meta.index} · ${meta.title}`,
    view: current?.id ?? 'visao-geral',
  });

  // As seções entram na barra ⌘K pelos comandos globais (commands/useGlobalCommands.ts).

  const found = matchSections(query);
  const shown = new Set(found);
  const attention = (id: SectionId) => {
    if (id === 'backups' && health.data?.warning) return t('ajustes-aviso-backup');
    return null;
  };

  const onSearchKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && found.length) {
      e.preventDefault();
      go(found[0]!);
    } else if (e.key === 'Escape' && query) {
      e.preventDefault();
      setQuery('');
    }
  };

  const Section = current ? RENDER[current.id] : null;
  const group = current ? GROUPS.find((g) => g.id === current.group)! : null;

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader index={meta.index} title={meta.title} description={t('ajustes-descricao')} />
      <div className="grid gap-6 @min-[790px]:grid-cols-[240px_minmax(0,1fr)]">
        <nav
          aria-label={t('ajustes-secoes')}
          className="flex flex-col gap-4 @min-[790px]:sticky @min-[790px]:top-0 @min-[790px]:self-start"
        >
          <label className="flex h-8 items-center gap-2 rounded-control border border-line-strong bg-surface px-2.5 text-dim focus-within:border-accent">
            <Icon name="search" size={13} />
            <input
              type="search"
              aria-label={t('ajustes-buscar')}
              placeholder={t('ajustes-buscar-placeholder')}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={onSearchKey}
              className="min-w-0 flex-1 bg-transparent text-[13px] text-fg outline-none placeholder:text-dim"
            />
          </label>
          {/* Em telas estreitas a lista fica escondida: a visão geral (cartões) e o caminho
              "Ajustes › grupo" no topo da seção fazem o papel dela. */}
          <button
            type="button"
            aria-current={!current ? 'page' : undefined}
            onClick={() => go(null)}
            className={cx(
              'hidden items-center gap-2 rounded-control px-2.5 py-1.5 text-left text-[13px] transition-colors @min-[790px]:flex',
              !current ? 'bg-accent-soft text-accent' : 'text-fg hover:bg-surface-2',
            )}
          >
            <Icon name="sliders" size={13} />
            {t('ajustes-visao-geral')}
          </button>
          {GROUPS.map((g) => {
            const items = SECTIONS.filter((s) => s.group === g.id && shown.has(s.id));
            if (!items.length) return null;
            return (
              <div
                key={g.id}
                className={cx('flex-col gap-1 @min-[790px]:flex', query ? 'flex' : 'hidden')}
              >
                <p className="label-caps px-2.5 text-[10px]">{g.label}</p>
                <ul className="flex flex-col gap-0.5">
                  {items.map((s) => {
                    const warn = attention(s.id);
                    return (
                      <li key={s.id}>
                        <button
                          type="button"
                          aria-current={current?.id === s.id ? 'page' : undefined}
                          onClick={() => go(s.id)}
                          title={s.hint}
                          className={cx(
                            'flex w-full min-w-0 items-center gap-2 rounded-control px-2.5 py-1.5 text-left text-[13px] transition-colors',
                            current?.id === s.id
                              ? 'bg-accent-soft text-accent'
                              : 'text-fg hover:bg-surface-2',
                          )}
                        >
                          <Icon name={s.icon} size={13} className="shrink-0 text-dim" />
                          <span className="min-w-0 flex-1 truncate">{s.label}</span>
                          {warn ? (
                            <span className="text-negative" title={warn}>
                              <Icon name="alert" size={12} title={warn} />
                            </span>
                          ) : null}
                        </button>
                      </li>
                    );
                  })}
                </ul>
              </div>
            );
          })}
          {found.length === 0 ? (
            <p role="status" className="px-2.5 text-[12.5px] text-dim">
              {t('ajustes-busca-vazia', { busca: query })}
            </p>
          ) : null}
        </nav>

        {current && Section && group ? (
          <section aria-label={current.label} className="flex min-w-0 flex-col gap-5">
            <header className="flex flex-col gap-1 border-b border-line pb-4">
              <p className="label-caps flex items-center gap-1.5 text-[10px]">
                <button
                  type="button"
                  onClick={() => go(null)}
                  className="rounded-control hover:text-fg"
                >
                  {meta.title}
                </button>
                <span aria-hidden>{'›'}</span>
                <span>{group.label}</span>
              </p>
              <h2 className="text-[20px] leading-tight font-medium text-fg">{current.label}</h2>
              <p className="text-[13px] text-dim">{current.hint}</p>
            </header>
            <Section />
          </section>
        ) : (
          <section
            aria-label={t('ajustes-visao-geral-rotulo')}
            className="flex min-w-0 flex-col gap-6"
          >
            {GROUPS.map((g) => {
              const items = SECTIONS.filter((s) => s.group === g.id && shown.has(s.id));
              if (!items.length) return null;
              return (
                <div key={g.id} className="flex flex-col gap-3">
                  <div>
                    <h2 className="text-[15px] font-medium text-fg">{g.label}</h2>
                    <p className="text-[12.5px] text-dim">{g.hint}</p>
                  </div>
                  <ul className="grid gap-2.5 @min-[560px]:grid-cols-2 @min-[1100px]:grid-cols-3">
                    {items.map((s) => {
                      const warn = attention(s.id);
                      return (
                        <li key={s.id}>
                          <button
                            type="button"
                            onClick={() => go(s.id)}
                            aria-label={t('ajustes-cartao', { secao: s.label, dica: s.hint })}
                            className={cx(
                              'panel flex h-full w-full items-start gap-3 rounded-card border p-3.5 text-left transition-colors hover:border-accent',
                              warn ? 'border-negative/60' : 'border-panel',
                            )}
                          >
                            <span className="mt-0.5 inline-flex size-7 shrink-0 items-center justify-center rounded-control bg-accent-soft text-accent">
                              <Icon name={s.icon} size={14} />
                            </span>
                            <span className="flex min-w-0 flex-col gap-0.5">
                              <span className="text-[13.5px] font-medium text-fg">{s.label}</span>
                              <span className="text-[12px] leading-snug text-dim">{s.hint}</span>
                              {warn ? (
                                <span className="mt-1 inline-flex items-center gap-1 text-[12px] text-negative">
                                  <Icon name="alert" size={12} />
                                  {warn}
                                </span>
                              ) : null}
                            </span>
                          </button>
                        </li>
                      );
                    })}
                  </ul>
                </div>
              );
            })}
          </section>
        )}
      </div>
    </div>
  );
}
