import { Suspense, useCallback, useEffect, useRef } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router';

import { useTrackNavOrigin } from '@/state/navOrigin';

import { useHousehold } from '@/api/hooks/household';
import { useProfile } from '@/api/hooks/profile';
import { useLockVault, useVaultStatus } from '@/api/vaultQueries';
import { CommandBar } from '@/commands/CommandBar';
import { QuickActionsHost } from '@/commands/QuickActionsHost';
import { LiveIndicator, PhosphorRainHost, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { HelpProvider, type HelpSettings } from '@/design/help';
import { useCountryInfo } from '@/api/hooks/country';
import { CompanionHost, CompanionLauncher } from '@/features/companion/CompanionPanel';
import { listenApp } from '@/features/agent/tauriBridge';
import { DownloadOffers } from '@/features/import/DownloadOffers';
import { UniversalImport } from '@/features/import/UniversalImport';
import { useDataEvents } from '@/api/hooks/dataEvents';
import { useAppSettings } from '@/api/hooks/settings';
import { updateCapture } from '@/lib/osGuard';
import { usePortfolioCommands } from '@/features/portfolio/commands';
import { useCompanion } from '@/state/companion';
import { setBaseCurrency } from '@/state/currency';
import { useUi } from '@/state/ui';
import { t } from '@/i18n';

import { AutoLock } from './AutoLock';
import { isAgentScreen } from './screens';
import { Sidebar, SIDEBAR_WIDTH } from './Sidebar';
import { TopBar } from './TopBar';
import { useAppCommands } from './useAppCommands';
import { useGlobalHotkeys } from './useGlobalHotkeys';
import { useViewAs } from './viewAs';

function ScreenLoading() {
  return (
    <div className="py-10">
      <LiveIndicator label={t('casca-carregando-tela')} tone="accent" />
    </div>
  );
}

/** Shell do app desbloqueado: barra lateral, barra superior, tela atual e barra de comando. */
export function AppShell() {
  const { data: status } = useVaultStatus();
  const lockMutation = useLockVault();
  const location = useLocation();
  const mainRef = useRef<HTMLElement>(null);
  const firstRender = useRef(true);

  const { mutate: lockVault, isPending: locking } = lockMutation;
  const lock = useCallback(() => {
    if (!locking) lockVault();
  }, [lockVault, locking]);

  const autoLock = useCallback(() => {
    // O aviso vai antes: depois do bloqueio este componente já desmontou.
    toast({
      title: t('casca-bloqueado-inatividade'),
      description: t('casca-bloqueado-inatividade-dica'),
    });
    lockVault();
  }, [lockVault]);

  // Valores sem moeda própria saem na moeda base da família (a do país do cofre).
  const baseCurrency = useHousehold().data?.base_currency;
  useEffect(() => {
    if (baseCurrency) setBaseCurrency(baseCurrency);
  }, [baseCurrency]);

  useAppCommands({ lock });
  // A trilha da página do ativo volta à tela de onde a pessoa veio.
  useTrackNavOrigin();
  useGlobalHotkeys({ lock });
  usePortfolioCommands();
  // Na tela do Agente o chat dela é o principal: o companheiro (painel e botão) some.
  const onAgentScreen = isAgentScreen(location.pathname);
  const companionOpen = useCompanion((s) => s.open) && !onAgentScreen;
  const companionWidth = useCompanion((s) => s.width);
  const sidebarCollapsed = useUi((s) => s.sidebarCollapsed);
  const toggleSidebar = useUi((s) => s.toggleSidebar);
  const sidebarWidth = sidebarCollapsed ? SIDEBAR_WIDTH.collapsed : SIDEBAR_WIDTH.open;

  const help = useHelpSettings();

  // Cofre novo, sem perfil: primeiro as boas-vindas (questionário ou modo avançado).
  const navigate = useNavigate();
  const pending = useProfile().data?.onboarding_pending ?? false;
  useEffect(() => {
    if (pending && location.pathname !== '/boas-vindas')
      void navigate('/boas-vindas', { replace: true });
  }, [pending, location.pathname, navigate]);

  // Ao trocar de tela, leva o foco (e a rolagem) para o conteúdo novo.
  useEffect(() => {
    if (firstRender.current) {
      firstRender.current = false;
      return;
    }
    const main = mainRef.current;
    if (!main) return;
    main.scrollTo?.({ top: 0 });
    main.focus({ preventScroll: true });
  }, [location.pathname]);

  return (
    <HelpProvider value={help}>
      <div
        className="grid h-screen grid-rows-[auto_minmax(0,1fr)] bg-canvas"
        style={{
          gridTemplateColumns: companionOpen
            ? `${sidebarWidth}px minmax(0,1fr) ${companionWidth}px`
            : `${sidebarWidth}px minmax(0,1fr)`,
        }}
      >
        <a
          href="#conteudo"
          onClick={(event) => {
            event.preventDefault();
            mainRef.current?.focus();
          }}
          className="sr-only-focusable fixed top-2 left-2 z-[90] rounded-control bg-accent px-3 py-1.5 text-[12px] font-medium text-accent-ink"
        >
          {t('casca-pular-conteudo')}
        </a>
        <div className="row-span-2 min-h-0">
          <Sidebar
            householdName={status?.household_name ?? null}
            autoLockMinutes={status?.auto_lock_minutes ?? 0}
            onLock={lock}
            locking={locking}
            collapsed={sidebarCollapsed}
            onToggleCollapsed={toggleSidebar}
          />
        </div>
        <TopBar householdName={status?.household_name ?? null} onLock={lock} />
        {/* Sempre montado (fechar só esconde): a conversa sobrevive à navegação. */}
        <div
          className={cx(
            'col-start-3 row-span-2 row-start-1 min-h-0',
            companionOpen ? 'flex' : 'hidden',
          )}
        >
          <CompanionHost />
          <UniversalImport />
          <DataEventsListener />
        </div>
        <main
          id="conteudo"
          ref={mainRef}
          tabIndex={-1}
          className="@container min-h-0 overflow-y-auto focus:outline-none"
        >
          <AutoLock minutes={status?.auto_lock_minutes ?? 0} onLock={autoLock} />
          <div className="mx-auto w-full max-w-[1440px] px-8 pt-7 pb-16">
            <Suspense fallback={<ScreenLoading />}>
              <Outlet />
            </Suspense>
          </div>
        </main>
        {onAgentScreen ? null : <CompanionLauncher />}
        <CommandBar />
        <QuickActionsHost />
        <PhosphorRainHost />
        <DownloadOffers />
      </div>
    </HelpProvider>
  );
}

/**
 * Ícones `?` (design/help): nível e tom do perfil de quem está em vista ("Ver como"), ou
 * a escolha feita em Ajustes › Perfil (sempre / nunca).
 */
function useHelpSettings(): HelpSettings {
  const { memberId } = useViewAs();
  const profile = useProfile(memberId).data;
  const pref = useUi((s) => s.helpTips);
  const brazil = (useCountryInfo()?.code ?? 'BR') === 'BR';
  const tone = profile?.tone ?? 'equilibrado';
  if (pref === 'off') return { level: 'avancado', tone, brazil };
  if (pref === 'on') return { level: 'iniciante', tone, brazil };
  return { level: profile?.level ?? 'avancado', tone, brazil };
}

/** Tempo real: relê o que o agente, o MCP ou as rotinas gravaram. */
function DataEventsListener() {
  useDataEvents();
  // "Permitir capturas de tela" (Ajustes › Segurança): só vale com o cofre aberto.
  const allowCapture = useAppSettings().data?.allow_screen_capture === true;
  useEffect(() => {
    updateCapture({ allowed: allowCapture });
    return () => updateCapture({ allowed: false });
  }, [allowCapture]);
  const navigate = useNavigate();
  // macOS: Treasury › Apoiar o projeto…
  useEffect(
    () => listenApp<null>('menu://support', () => void navigate('/ajustes?secao=sobre')),
    [navigate],
  );
  // Versão nova baixada e conferida, ou só avisada (modo "só avisar"): baixar e instalar
  // ficam em Ajustes › Atualizações.
  useEffect(() => {
    const open = {
      label: t('ajustes-atualizacao-ver'),
      onClick: () => void navigate('/ajustes?secao=atualizacoes'),
    };
    const offReady = listenApp<{ version: string } | null>('update://ready', (u) =>
      toast({ title: t('ajustes-atualizacao-pronta', { versao: u?.version ?? '' }), action: open }),
    );
    const offAvailable = listenApp<{ version: string } | null>('update://available', (u) =>
      toast({
        title: t('ajustes-atualizacao-disponivel', { versao: u?.version ?? '' }),
        action: open,
      }),
    );
    return () => {
      offReady();
      offAvailable();
    };
  }, [navigate]);
  return null;
}
