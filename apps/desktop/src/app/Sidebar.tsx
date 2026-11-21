import type { ReactNode } from 'react';
import { NavLink, useLocation } from 'react-router';

import { isAssetPage, useNavOrigin } from '@/state/navOrigin';

import {
  Button,
  Icon,
  IconButton,
  Kbd,
  KeyValue,
  LiveIndicator,
  Tooltip,
} from '@/design/components';
import { cx } from '@/design/cx';
import { isMac } from '@/lib/platform';
import { t } from '@/i18n';

import { Brand, BrandMark } from './Brand';
import { ScreenBadge } from './ScreenBadge';
import { VaultMenu } from './VaultMenu';
import { ViewAsMenu } from './ViewAsMenu';
import { atLeast, useLevel } from './profile';
import { groupLabel, SCREEN_GROUPS, SCREENS, screenShortcut, type ScreenDef } from './screens';
import { useVisibleScreens } from './useVisibleScreens';

function ariaShortcut(keys: readonly string[] | undefined): string | undefined {
  if (!keys) return undefined;
  const key = keys[keys.length - 1];
  return `${isMac() ? 'Meta' : 'Control'}+${key}`;
}

export interface SidebarProps {
  householdName: string | null;
  autoLockMinutes: number;
  onLock: () => void;
  locking?: boolean;
  /** Recolhida: só os números das telas (com dica), para dar área à tela. */
  collapsed?: boolean;
  onToggleCollapsed?: () => void;
}

/** Largura da barra lateral aberta e recolhida (px), usada também pelo grid do shell. */
export const SIDEBAR_WIDTH = { open: 236, collapsed: 60 } as const;

const TOGGLE_SHORTCUT = ['mod', '\\'];

/** Na página de um ativo, a tela do menu que fica acesa: Mercado, se veio de lá; senão Ativos. */
function useAssetPageOwner(): string | null {
  const { pathname } = useLocation();
  const origin = useNavOrigin((s) => s.origin);
  if (!isAssetPage(pathname)) return null;
  return origin?.pathname.startsWith('/mercado') ? 'mercado' : 'ativos';
}

/** Uma tela no menu: número, título, selo e atalho. */
function ScreenLink({ screen }: { screen: ScreenDef }) {
  const owned = useAssetPageOwner() === screen.id;
  const position = SCREENS.findIndex((s) => s.id === screen.id);
  const shortcut = screenShortcut(position);
  return (
    <li>
      <NavLink
        to={screen.path}
        aria-keyshortcuts={ariaShortcut(shortcut)}
        className={({ isActive }) =>
          cx(
            'group relative flex h-8 items-center gap-2.5 rounded-control px-2.5 transition-[background-color,color,box-shadow] duration-150',
            isActive || owned
              ? 'bg-accent-soft text-fg shadow-[inset_0_0_0_1px_var(--accent-soft)] before:absolute before:inset-y-2 before:-left-2 before:w-[3px] before:rounded-r-full before:bg-accent'
              : 'text-dim hover:bg-surface-2 hover:text-fg',
          )
        }
      >
        {({ isActive }) => (
          <>
            <span
              className={cx(
                'num w-5 text-[11px]',
                isActive || owned ? 'text-accent' : 'text-dim group-hover:text-fg',
              )}
            >
              {screen.index}
            </span>
            <span aria-hidden className="text-line-strong">
              ·
            </span>
            <span className="flex-1 truncate text-[13px] font-medium tracking-[-0.005em]">
              {screen.title}
            </span>
            <ScreenBadge screenId={screen.id} />
            {shortcut ? (
              <span
                aria-hidden
                className="opacity-0 transition-opacity group-hover:opacity-100 group-focus-visible:opacity-100"
              >
                <Kbd keys={shortcut} />
              </span>
            ) : null}
          </>
        )}
      </NavLink>
    </li>
  );
}

/** Barra lateral: as telas numeradas (a 12 só no modo desenvolvedor), em três grupos (as acima do nível do perfil em “Mais telas”), e o estado do cofre. */
export function Sidebar({
  householdName,
  autoLockMinutes,
  onLock,
  locking,
  collapsed = false,
  onToggleCollapsed,
}: SidebarProps) {
  const screens: readonly ScreenDef[] = useVisibleScreens();
  const level = useLevel();
  const { pathname } = useLocation();
  const primary = screens.filter((s) => atLeast(level, s.minLevel));
  const more = screens.filter((s) => !atLeast(level, s.minLevel));
  const moreActive = more.some((s) => pathname === s.path || pathname.startsWith(`${s.path}/`));
  const toggle = onToggleCollapsed ? (
    <IconButton
      label={collapsed ? t('casca-menu-expandir') : t('casca-menu-recolher')}
      shortcut={TOGGLE_SHORTCUT}
      tooltipSide="right"
      size="sm"
      icon={<Icon name={collapsed ? 'chevronRight' : 'chevronLeft'} size={14} />}
      onClick={onToggleCollapsed}
      aria-expanded={!collapsed}
      aria-controls="menu-telas"
    />
  ) : null;

  if (collapsed) return <CollapsedSidebar toggle={toggle} onLock={onLock} locking={locking} />;

  return (
    <aside className="chrome flex h-full min-h-0 flex-col border-r border-line select-none">
      <div
        className="flex h-12 shrink-0 items-center gap-2 border-b border-line pr-2 pl-4"
        data-tauri-drag-region
      >
        <Brand className="min-w-0 flex-1" />
        {toggle}
      </div>

      <div className="border-b border-line px-4 py-4">
        <div className="flex items-center justify-between gap-2">
          <span className="label-caps">{t('casca-familia')}</span>
          <VaultMenu />
        </div>
        <p className="mt-0.5 truncate text-[14px] font-medium text-fg" title={householdName ?? ''}>
          {householdName ?? t('casca-sem-nome')}
        </p>
        <ViewAsMenu />
      </div>

      <nav
        id="menu-telas"
        aria-label={t('casca-telas')}
        className="min-h-0 flex-1 overflow-y-auto px-2 py-2"
      >
        {SCREEN_GROUPS.map((group) => {
          const items = primary.filter((s) => s.group === group);
          if (!items.length) return null;
          return (
            <div key={group} className="pt-2">
              <p className="label-caps px-2.5 pt-2 pb-1.5 text-[10px]" aria-hidden>
                {groupLabel(group)}
              </p>
              <ul className="flex flex-col gap-px">
                {items.map((screen) => (
                  <ScreenLink key={screen.id} screen={screen} />
                ))}
              </ul>
            </div>
          );
        })}
        {more.length ? (
          <details className="group/more pt-3" open={moreActive || undefined}>
            <summary className="label-caps flex cursor-pointer items-center gap-1.5 rounded-control px-2.5 py-1.5 text-[10px] hover:bg-surface-2">
              <Icon
                name="chevronRight"
                size={12}
                className="transition-transform group-open/more:rotate-90"
              />
              {t('casca-mais-telas')}
            </summary>
            <ul aria-label={t('casca-mais-telas')} className="mt-1 flex flex-col gap-px">
              {more.map((screen) => (
                <ScreenLink key={screen.id} screen={screen} />
              ))}
            </ul>
          </details>
        ) : null}
      </nav>

      <div className="flex flex-col gap-3 border-t border-line px-4 py-4">
        <LiveIndicator label={t('casca-cofre-aberto')} active={false} />
        <KeyValue
          layout="stacked"
          items={[
            {
              label: t('casca-auto-bloqueio'),
              value:
                autoLockMinutes > 0
                  ? t('casca-minutos', { minutos: autoLockMinutes })
                  : t('casca-desligado'),
            },
            { label: t('casca-cifra'), value: 'AES-256' },
          ]}
        />
        <Button
          size="sm"
          variant="secondary"
          icon={<Icon name="lock" size={14} />}
          onClick={onLock}
          loading={locking}
          fullWidth
        >
          {t('casca-bloquear-cofre')}
        </Button>
      </div>
    </aside>
  );
}

/** Barra recolhida: marca, botão de expandir, números das telas e o cadeado. */
function CollapsedSidebar({
  toggle,
  onLock,
  locking,
}: {
  toggle: ReactNode;
  onLock: () => void;
  locking?: boolean;
}) {
  // A dica (Radix, asChild) mescla `className` como texto: o estado ativo vem da rota.
  const { pathname } = useLocation();
  const owner = useAssetPageOwner();
  const screens = useVisibleScreens();
  return (
    <aside className="chrome flex h-full min-h-0 flex-col items-center border-r border-line select-none">
      <div
        className="flex h-12 w-full shrink-0 items-center justify-center border-b border-line"
        data-tauri-drag-region
      >
        <BrandMark size={18} />
      </div>
      <div className="flex w-full justify-center border-b border-line py-2">{toggle}</div>

      <nav
        id="menu-telas"
        aria-label={t('casca-telas')}
        className="flex min-h-0 w-full flex-1 flex-col items-center gap-px overflow-y-auto py-2"
      >
        {SCREEN_GROUPS.map((group, gi) => (
          <ul
            key={group}
            aria-label={groupLabel(group)}
            className={cx(
              'flex w-full flex-col items-center gap-px',
              gi > 0 && 'mt-2 border-t border-line pt-2',
            )}
          >
            {screens
              .filter((s) => s.group === group)
              .map((screen) => {
                const position = SCREENS.findIndex((s) => s.id === screen.id);
                const shortcut = screenShortcut(position);
                const isActive =
                  pathname === screen.path ||
                  pathname.startsWith(`${screen.path}/`) ||
                  owner === screen.id;
                return (
                  <li key={screen.id}>
                    <Tooltip
                      side="right"
                      content={
                        <span className="flex items-center gap-2">
                          {screen.title}
                          {shortcut ? <Kbd keys={shortcut} /> : null}
                        </span>
                      }
                    >
                      <NavLink
                        to={screen.path}
                        aria-label={screen.title}
                        aria-keyshortcuts={ariaShortcut(shortcut)}
                        aria-current={isActive ? 'page' : undefined}
                        className={cx(
                          'relative flex size-9 items-center justify-center rounded-control transition-[background-color,color] duration-150',
                          isActive
                            ? 'bg-accent-soft text-accent before:absolute before:inset-y-2 before:-left-[11px] before:w-[3px] before:rounded-r-full before:bg-accent'
                            : 'text-dim hover:bg-surface-2 hover:text-fg',
                        )}
                      >
                        <span className="num text-[12px] font-medium">{screen.index}</span>
                        <span className="absolute -top-0.5 -right-0.5 scale-75">
                          <ScreenBadge screenId={screen.id} />
                        </span>
                      </NavLink>
                    </Tooltip>
                  </li>
                );
              })}
          </ul>
        ))}
      </nav>

      <div className="flex w-full flex-col items-center gap-2 border-t border-line py-3">
        <span
          className="size-1.5 rounded-full bg-positive"
          title={t('casca-cofre-aberto')}
          aria-hidden
        />
        <IconButton
          label={t('casca-bloquear-cofre')}
          tooltipSide="right"
          shortcut={['mod', 'shift', 'L']}
          icon={<Icon name="lock" size={15} />}
          onClick={onLock}
          disabled={locking}
        />
      </div>
    </aside>
  );
}
