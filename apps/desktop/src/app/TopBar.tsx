import { useLocation } from 'react-router';

import { transportKind } from '@/api/client';
import { useVaultStatus } from '@/api/vaultQueries';
import { useCommandBar } from '@/commands/registry';
import {
  Icon,
  IconButton,
  Kbd,
  ScannerSweep,
  PhosphorActivity,
  ThemeActivity,
  Tooltip,
} from '@/design/components';
import { AttentionBell } from '@/features/attention/AttentionBell';
import { useImportHub } from '@/features/import/importHub';
import { useCompanion } from '@/state/companion';
import { useBusy } from '@/state/activity';
import { useUi } from '@/state/ui';
import { t } from '@/i18n';

import { ActivityBridge, BackgroundTasks } from './BackgroundTasks';
import { SCREENS } from './screens';
import { ThemeMenu } from './ThemeMenu';

export interface TopBarProps {
  householdName: string | null;
  onLock: () => void;
}

/**
 * Barra superior: tela atual, selo do cofre de demonstração, aviso de mock, gatilho da barra de comando, o botão
 * Importar (janela única de importação), o sino de "Para resolver", tarefas em segundo plano, agente, modo privacidade, tema e cadeado.
 */
export function TopBar({ householdName, onLock }: TopBarProps) {
  const location = useLocation();
  const privacy = useUi((s) => s.privacy);
  const togglePrivacy = useUi((s) => s.togglePrivacy);
  const openCommandBar = useCommandBar((s) => s.setOpen);
  const openImport = useImportHub((s) => s.setOpen);
  const openImportWith = useImportHub((s) => s.openWith);
  const companionOpen = useCompanion((s) => s.open);
  const toggleCompanion = useCompanion((s) => s.toggle);
  const screen = SCREENS.find((s) => location.pathname.startsWith(s.path));
  const mock = transportKind() === 'mock';
  const vault = useVaultStatus().data;
  const demo = vault?.demo ?? false;
  const newerApp = vault?.newer_app_version ?? null;
  // A atividade do app (state/activity.ts): o scanner do tema Scanner, a peça animada da
  // Órbita, do Neon e do Lo-fi e o chip do Fósforo só ligam com alguma tarefa
  // do agente, do núcleo ou de um comando demorado da interface em andamento.
  const busy = useBusy();

  return (
    <header
      className="topbar relative flex h-12 min-w-0 shrink-0 items-center gap-3 border-b border-line px-5"
      data-tauri-drag-region
    >
      <div className="flex min-w-0 items-baseline gap-2" data-tauri-drag-region>
        <span className="truncate text-[13px] font-medium text-fg">{householdName}</span>
        {screen ? (
          <span className="label-caps hidden truncate sm:inline" aria-hidden>
            / {screen.index} · {screen.title}
          </span>
        ) : null}
      </div>

      <div className="ml-auto flex items-center gap-1.5">
        {demo ? (
          <Tooltip content={t('casca-demo-dica')}>
            <span
              tabIndex={0}
              className="label-caps mr-1 inline-flex items-center gap-1.5 rounded-control border border-accent/60 px-1.5 py-0.5 text-accent"
            >
              <span aria-hidden className="size-1.5 rounded-full bg-current" />
              {t('casca-demo')}
            </span>
          </Tooltip>
        ) : null}
        {newerApp ? (
          <Tooltip content={t('casca-versao-antiga-dica', { versao: newerApp })}>
            <span
              tabIndex={0}
              className="label-caps mr-1 inline-flex items-center gap-1.5 rounded-control border border-warning/50 px-1.5 py-0.5 text-warning"
            >
              <span aria-hidden className="size-1.5 rounded-full bg-current" />
              {t('casca-versao-antiga')}
            </span>
          </Tooltip>
        ) : null}
        {mock ? (
          <Tooltip content={t('casca-mock-dica')}>
            <span
              tabIndex={0}
              className="label-caps mr-1 inline-flex items-center gap-1.5 rounded-control border border-warning/50 px-1.5 py-0.5 text-warning"
            >
              <span aria-hidden className="size-1.5 rounded-full bg-current" />
              {t('casca-mock')}
            </span>
          </Tooltip>
        ) : null}

        <button
          type="button"
          onClick={() => openCommandBar(true)}
          aria-keyshortcuts="Meta+K Control+K"
          className="mr-1 flex h-8 w-[min(34vw,320px)] items-center gap-2.5 rounded-control border border-line bg-surface px-2.5 text-left text-[12.5px] text-dim shadow-[var(--highlight)] transition-[border-color,color,box-shadow] duration-150 hover:border-line-strong hover:text-fg"
        >
          <Icon name="search" size={14} />
          <span className="flex-1 truncate">{t('casca-buscar')}</span>
          <Kbd keys={['mod', 'K']} />
        </button>
        {/* Importar em destaque ao lado da busca; aceita arquivos soltos em cima. */}
        <Tooltip content={t('importacao-botao-topo-dica')}>
          <button
            type="button"
            onClick={() => openImport(true)}
            onDragOver={(e) => e.preventDefault()}
            onDrop={(e) => {
              e.preventDefault();
              const files = Array.from(e.dataTransfer.files ?? []);
              if (files.length) openImportWith(files);
            }}
            className="mr-1 inline-flex h-8 items-center gap-1.5 rounded-control border border-accent/60 bg-accent-soft px-2.5 text-[12.5px] font-medium text-accent transition-colors duration-150 hover:border-accent hover:bg-accent hover:text-accent-ink"
          >
            <Icon name="clip" size={13} />
            {t('importacao-botao-topo')}
          </button>
        </Tooltip>

        {privacy ? (
          <span className="label-caps hidden text-accent md:inline" aria-hidden>
            {t('casca-privacidade')}
          </span>
        ) : null}
        <PhosphorActivity />
        <AttentionBell />
        <BackgroundTasks />
        <IconButton
          label={t('casca-agente-ao-lado')}
          pressed={companionOpen}
          shortcut={['mod', 'J']}
          icon={<Icon name="agent" />}
          onClick={toggleCompanion}
        />
        <IconButton
          label={t('casca-modo-privacidade')}
          pressed={privacy}
          shortcut={['mod', 'shift', 'H']}
          icon={<Icon name={privacy ? 'eyeOff' : 'eye'} />}
          onClick={togglePrivacy}
        />
        <ThemeMenu />
        <IconButton
          label={t('casca-bloquear-cofre')}
          shortcut={['mod', 'shift', 'L']}
          icon={<Icon name="lock" />}
          onClick={onLock}
        />
      </div>
      <ActivityBridge />
      <ScannerSweep active={busy} />
      <ThemeActivity active={busy} />
    </header>
  );
}
