import { useMemo } from 'react';
import { useNavigate } from 'react-router';

import { useRegisterCommands, type AppCommand } from '@/commands/registry';
import { useGlobalCommands } from '@/commands/useGlobalCommands';
import { THEMES, themeIcon } from '@/design/themes';
import { useReportCommands } from '@/features/reports/commands';
import { useImportHub } from '@/features/import/importHub';
import { useCompanion } from '@/state/companion';
import { useUi } from '@/state/ui';
import { t } from '@/i18n';

import { SCREENS, screenShortcut } from './screens';
import { useVisibleScreens } from './useVisibleScreens';

/** Comandos do shell: navegar pelas telas, bloquear, privacidade, temas e agente. */
export function useAppCommands({ lock }: { lock: () => void }): void {
  const navigate = useNavigate();
  const privacy = useUi((s) => s.privacy);
  const theme = useUi((s) => s.theme);
  const togglePrivacy = useUi((s) => s.togglePrivacy);
  const toggleTheme = useUi((s) => s.toggleTheme);
  const setTheme = useUi((s) => s.setTheme);
  const sidebarCollapsed = useUi((s) => s.sidebarCollapsed);
  const toggleSidebar = useUi((s) => s.toggleSidebar);
  const screens = useVisibleScreens();

  const commands = useMemo<AppCommand[]>(
    () => [
      ...screens.map((screen): AppCommand => ({
        id: `nav.${screen.id}`,
        title: screen.title,
        prefix: screen.index,
        group: t('casca-cmd-grupo-navegacao'),
        keywords: [...t('casca-cmd-ir-para').split(','), screen.summary, ...screen.keywords],
        shortcut: screenShortcut(SCREENS.indexOf(screen)),
        order: SCREENS.indexOf(screen),
        perform: () => {
          void navigate(screen.path);
        },
      })),
      {
        id: 'import.files',
        title: t('casca-cmd-importar'),
        group: t('casca-cmd-grupo-cadastro'),
        icon: 'clip',
        keywords: t('casca-cmd-importar-busca').split(','),
        perform: () => useImportHub.getState().setOpen(true),
      },
      {
        id: 'vault.lock',
        title: t('casca-bloquear-cofre'),
        group: t('casca-cmd-grupo-cofre'),
        icon: 'lock',
        shortcut: ['mod', 'shift', 'L'],
        keywords: t('casca-cmd-bloquear-busca').split(','),
        perform: lock,
      },
      {
        id: 'ui.privacy',
        title: privacy ? t('casca-cmd-privacidade-desativar') : t('casca-cmd-privacidade-ativar'),
        group: t('casca-cmd-grupo-exibicao'),
        icon: privacy ? 'eye' : 'eyeOff',
        shortcut: ['mod', 'shift', 'H'],
        keywords: t('casca-cmd-privacidade-busca').split(','),
        order: 0,
        perform: togglePrivacy,
      },
      {
        id: 'ui.sidebar',
        title: sidebarCollapsed ? t('casca-menu-expandir') : t('casca-menu-recolher'),
        group: t('casca-cmd-grupo-exibicao'),
        icon: sidebarCollapsed ? 'chevronRight' : 'chevronLeft',
        shortcut: ['mod', '\\'],
        keywords: t('casca-cmd-menu-busca').split(','),
        order: 0.5,
        perform: toggleSidebar,
      },
      {
        id: 'ui.theme.toggle',
        title: t('casca-cmd-tema-alternar'),
        group: t('casca-cmd-grupo-exibicao'),
        icon: 'contrast',
        keywords: t('casca-cmd-tema-busca').split(','),
        order: 1,
        perform: toggleTheme,
      },
      ...THEMES.map((th, i): AppCommand => ({
        id: `ui.theme.${th.id}`,
        title: t('casca-cmd-tema', { tema: th.label }),
        group: t('casca-cmd-grupo-exibicao'),
        icon: themeIcon(th.id),
        hint: theme === th.id ? t('casca-cmd-atual') : undefined,
        keywords: [
          ...t('casca-cmd-temas-busca').split(','),
          th.scheme === 'light' ? t('casca-cmd-claro') : t('casca-cmd-escuro'),
          th.singleHue ? t('casca-cmd-cinza-busca') : '',
          th.id === 'glass' ? t('casca-cmd-vidro-busca') : '',
          th.description,
        ],
        order: 10 + i,
        perform: () => setTheme(th.id),
      })),
      {
        id: 'agent.companion',
        title: t('casca-cmd-agente-alternar'),
        group: t('casca-cmd-grupo-agente'),
        icon: 'agent',
        shortcut: ['mod', 'J'],
        keywords: t('casca-cmd-agente-busca').split(','),
        perform: () => useCompanion.getState().toggle(),
      },
      {
        id: 'agent.ask',
        title: t('casca-cmd-perguntar'),
        group: t('casca-cmd-grupo-agente'),
        icon: 'agent',
        alwaysShow: true,
        keywords: t('casca-cmd-perguntar-busca').split(','),
        render: (search) =>
          search.trim()
            ? t('casca-cmd-perguntar-texto', { texto: search.trim() })
            : t('casca-cmd-perguntar'),
        // Abre o agente ao lado com a pergunta no campo; o envio fica com o usuário, que vê
        // qual motor vai receber os dados (e o contexto da tela que vai junto).
        perform: (search) => useCompanion.getState().ask(search.trim()),
      },
    ],
    [
      screens,
      navigate,
      lock,
      privacy,
      theme,
      togglePrivacy,
      toggleTheme,
      setTheme,
      sidebarCollapsed,
      toggleSidebar,
    ],
  );

  useRegisterCommands(commands);
  useReportCommands();
  useGlobalCommands();
}
