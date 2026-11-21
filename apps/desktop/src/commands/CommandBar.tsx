import * as RDialog from '@radix-ui/react-dialog';
import { Command, defaultFilter } from 'cmdk';
import { useMemo, useState } from 'react';

import { cx } from '@/design/cx';
import { Icon, Kbd } from '@/design/components';
import { t } from '@/i18n';

import { recentCommandIds, rememberCommand } from './recent';
import {
  groupCommands,
  normalizeSearch,
  useCommandBar,
  useCommandRegistry,
  type AppCommand,
} from './registry';

/**
 * Busca da barra: cada termo digitado precisa começar uma palavra do título ou
 * das palavras-chave (sem acentos); o título pesa mais na ordem. Sem isso, o
 * casamento difuso do cmdk acha letras soltas em qualquer descrição longa.
 * Abreviações (`lnc` → Lançamentos) ainda valem, só contra o título.
 */
function filter(_value: string, search: string, keywords?: string[]): number {
  const query = normalizeSearch(search).trim();
  if (!query) return 1;
  const title = normalizeSearch(keywords?.[0] ?? '');
  const words = normalizeSearch((keywords ?? []).join(' ')).split(/[^a-z0-9]+/);
  const terms = query.split(/\s+/);
  if (terms.every((term) => words.some((word) => word.startsWith(term)))) {
    return 0.5 + 0.5 * defaultFilter(title, query);
  }
  const fuzzy = defaultFilter(title, query);
  return fuzzy >= 0.1 ? fuzzy * 0.5 : 0;
}

/** Texto pesquisável do comando; o título vem primeiro (pesa mais). */
function keywordsOf(command: AppCommand): string[] {
  return [command.title, command.prefix ?? '', ...(command.keywords ?? [])];
}

/**
 * Barra de comando (⌘K / Ctrl+K), com cmdk dentro de um Radix Dialog: navegar,
 * agir e, a partir da Fase 4, perguntar ao agente, tudo no mesmo campo.
 */
export function CommandBar() {
  const open = useCommandBar((s) => s.open);
  const setOpen = useCommandBar((s) => s.setOpen);
  const commands = useCommandRegistry((s) => s.commands);
  const [search, setSearch] = useState('');

  const typing = search.trim().length > 0;
  // Sem nada digitado, as entidades do cofre (contas, ativos, lançamentos…) ficam de fora.
  const grouped = useMemo(
    () => groupCommands([...commands.values()].filter((c) => typing || !c.searchOnly)),
    [commands, typing],
  );
  // Os usados por último, primeiro (com a barra vazia); `open` refaz a lista ao abrir.
  const recent = useMemo(
    () =>
      open && !typing
        ? recentCommandIds()
            .map((id) => commands.get(id))
            .filter((c): c is AppCommand => Boolean(c) && !c!.disabled)
        : [],
    [open, typing, commands],
  );
  // O cmdk 1.1 ordena os itens dentro de cada grupo, mas não os grupos entre si:
  // com busca, os grupos vêm na ordem do melhor resultado de cada um.
  const groups = useMemo(() => {
    if (!search.trim()) return grouped;
    const best = (items: AppCommand[]) =>
      Math.max(0, ...items.map((c) => filter(c.id, search, keywordsOf(c))));
    return grouped
      .map((g) => ({ ...g, score: best(g.commands) }))
      .sort((a, b) => b.score - a.score);
  }, [grouped, search]);

  const close = (next: boolean) => {
    setOpen(next);
    if (!next) setSearch('');
  };

  const run = (command: AppCommand) => {
    if (command.disabled) return;
    const typed = search;
    close(false);
    if (!command.alwaysShow) rememberCommand(command.id);
    command.perform(typed);
  };

  return (
    <RDialog.Root open={open} onOpenChange={close}>
      <RDialog.Portal>
        <RDialog.Overlay className="fixed inset-0 z-40 animate-fade-in bg-overlay backdrop-blur-[3px]" />
        <RDialog.Content
          aria-describedby={undefined}
          className="fixed top-[14vh] left-1/2 z-50 w-[min(92vw,640px)] -translate-x-1/2 animate-pop-in overflow-hidden rounded-card border border-panel pop-surface focus:outline-none"
        >
          <RDialog.Title className="sr-only">{t('casca-cmd-barra')}</RDialog.Title>
          <Command label={t('casca-cmd-barra')} filter={filter} loop className="flex flex-col">
            <div className="flex items-center gap-3 border-b border-line px-4">
              <Icon name="search" className="shrink-0 text-dim" />
              <Command.Input
                value={search}
                onValueChange={setSearch}
                placeholder={t('casca-cmd-placeholder')}
                className="h-13 min-w-0 flex-1 bg-transparent text-[14.5px] text-fg outline-none"
              />
              <Kbd keys={['esc']} />
            </div>
            <Command.List className="max-h-[min(420px,56vh)] overflow-y-auto overscroll-contain p-1.5">
              <Command.Empty className="px-3 py-6 text-center text-[13px] text-dim">
                {t('casca-cmd-nenhum')}
              </Command.Empty>
              {recent.length ? (
                <Command.Group
                  heading={t('casca-cmd-grupo-recentes')}
                  className="[&_[cmdk-group-heading]]:label-caps [&_[cmdk-group-heading]]:px-2.5 [&_[cmdk-group-heading]]:pt-2.5 [&_[cmdk-group-heading]]:pb-1.5"
                >
                  {recent.map((command) => (
                    <CommandRow
                      key={`recent:${command.id}`}
                      value={`recent:${command.id}`}
                      command={command}
                      search={search}
                      onRun={run}
                    />
                  ))}
                </Command.Group>
              ) : null}
              {groups.map(({ group, commands: items }) => (
                <Command.Group
                  key={group}
                  heading={group}
                  forceMount={items.some((c) => c.alwaysShow) || undefined}
                  className="[&_[cmdk-group-heading]]:label-caps [&_[cmdk-group-heading]]:px-2.5 [&_[cmdk-group-heading]]:pt-2.5 [&_[cmdk-group-heading]]:pb-1.5"
                >
                  {items.map((command) => (
                    <CommandRow
                      key={command.id}
                      value={command.id}
                      command={command}
                      search={search}
                      onRun={run}
                    />
                  ))}
                </Command.Group>
              ))}
            </Command.List>
            <div className="flex items-center gap-4 border-t border-line px-4 py-2">
              <span className="label-caps flex items-center gap-1.5">
                <Kbd keys={['↑']} />
                <Kbd keys={['↓']} />
                {t('casca-cmd-navegar')}
              </span>
              <span className="label-caps flex items-center gap-1.5">
                <Kbd keys={['enter']} />
                {t('casca-cmd-executar')}
              </span>
              <span className="label-caps ml-auto flex items-center gap-1.5">
                <Kbd keys={['mod', 'K']} />
                {t('casca-cmd-abrir-fechar')}
              </span>
            </div>
          </Command>
        </RDialog.Content>
      </RDialog.Portal>
    </RDialog.Root>
  );
}

/** Uma linha da barra: ícone, índice da tela, título, dica e atalho. */
function CommandRow({
  value,
  command,
  search,
  onRun,
}: {
  value: string;
  command: AppCommand;
  search: string;
  onRun: (command: AppCommand) => void;
}) {
  return (
    <Command.Item
      value={value}
      keywords={keywordsOf(command)}
      disabled={command.disabled}
      forceMount={command.alwaysShow}
      onSelect={() => onRun(command)}
      className={cx(
        'group flex h-10 cursor-default items-center gap-3 rounded-control px-2.5 text-[13.5px] text-fg select-none',
        'data-[selected=true]:bg-accent-soft data-[selected=true]:text-accent',
        'data-[disabled=true]:text-dim data-[disabled=true]:data-[selected=true]:bg-surface-2 data-[disabled=true]:data-[selected=true]:text-dim',
      )}
    >
      {command.icon ? (
        <Icon name={command.icon} className="shrink-0 opacity-80" />
      ) : (
        <span aria-hidden className="w-4 shrink-0" />
      )}
      {command.prefix ? (
        <span className="num w-5 shrink-0 text-[11px] text-dim group-data-[selected=true]:text-accent">
          {command.prefix}
        </span>
      ) : null}
      <span className="min-w-0 flex-1 truncate">
        {command.render ? command.render(search) : command.title}
      </span>
      {command.hint ? (
        // Valores e datas (contas, lançamentos) em texto; marcas curtas ("atual") em selo.
        /\d/.test(command.hint) ? (
          <span className="num shrink-0 text-[11.5px] text-dim">{command.hint}</span>
        ) : (
          <span className="label-caps shrink-0 rounded-control border border-line px-1.5 text-[10px]">
            {command.hint}
          </span>
        )
      ) : null}
      {command.shortcut ? <Kbd keys={command.shortcut} /> : null}
    </Command.Item>
  );
}
