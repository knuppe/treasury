/**
 * Cofres (docs/vaults.md): a lista, trocar de cofre, criar um novo, abrir um arquivo .db
 * existente e criar um cofre de demonstração (docs/demo.md). Cada cofre tem menu de contexto
 * (botão direito, Shift+F10 ou o botão ⋯): abrir, renomear, criar uma cópia, copiar o caminho e
 * remover (com confirmação: só da lista ou também o arquivo, para a Lixeira). Usado na tela de
 * desbloqueio, no menu do app e em Ajustes › Cofres (com o botão Editar à vista).
 */
import { useState } from 'react';

import type { VaultInfo } from '@/api/generated/VaultInfo';
import {
  pickVaultFile,
  useAddVault,
  useCopyVault,
  useNewVault,
  useRemoveVault,
  useSelectVault,
  useUpdateVault,
  useVaults,
} from '@/api/hooks/vaults';
import { isTauri } from '@/api/client';
import {
  Button,
  Field,
  Icon,
  Input,
  toast,
  useContextMenu,
  type ContextMenuEntry,
} from '@/design/components';
import { cx } from '@/design/cx';
import { DemoQuickDialog } from '@/features/demo/DemoQuick';
import { ConfirmDialog, InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatNumber } from '@/lib/format';

function formatBytes(n: number | null): string {
  if (n == null) return '';
  return n >= 1024 ** 3
    ? `${formatNumber(n / 1024 ** 3, 1)} GB`
    : `${formatNumber(Math.max(1, Math.round(n / 1024 ** 2)))} MB`;
}

/** Remover um cofre: só da lista (padrão) ou também o arquivo, para a Lixeira. */
function RemoveVault({ v, onClose }: { v: VaultInfo; onClose: () => void }) {
  const remove = useRemoveVault();
  const [deleteFile, setDeleteFile] = useState(false);
  return (
    <ConfirmDialog
      open
      onOpenChange={(o) => !o && onClose()}
      title={t('cofre-remover-titulo', { nome: v.name })}
      description={t('cofre-remover-descricao')}
      confirmLabel={
        deleteFile ? t('cofre-remover-confirmar-lixeira') : t('cofre-remover-confirmar-lista')
      }
      pending={remove.isPending}
      error={remove.error}
      onConfirm={() =>
        remove.mutate(
          { id: v.id, delete_file: deleteFile },
          {
            onSuccess: () => {
              toast({
                tone: 'success',
                title: t('cofre-remover-toast-titulo', { nome: v.name }),
                description: deleteFile
                  ? t('cofre-remover-toast-lixeira')
                  : t('cofre-remover-toast-lista'),
              });
              onClose();
            },
          },
        )
      }
    >
      <div role="radiogroup" aria-label={t('cofre-remover-opcoes')} className="flex flex-col gap-2">
        {(
          [
            [
              false,
              t('cofre-remover-so-lista'),
              t('cofre-remover-so-lista-dica', { caminho: v.path }),
            ],
            [true, t('cofre-remover-lixeira'), t('cofre-remover-lixeira-dica')],
          ] as const
        ).map(([value, label, hint]) => (
          <label
            key={label}
            className={cx(
              'flex cursor-pointer items-start gap-2.5 rounded-control border px-3 py-2 text-[13px]',
              deleteFile === value ? 'border-accent bg-accent-soft/40' : 'border-line',
            )}
          >
            <input
              type="radio"
              name={`remover-${v.id}`}
              checked={deleteFile === value}
              onChange={() => setDeleteFile(value)}
              className="mt-1"
            />
            <span className="flex flex-col gap-0.5">
              <span className="text-fg">{label}</span>
              <span className="text-[12px] break-all text-dim">{hint}</span>
            </span>
          </label>
        ))}
      </div>
    </ConfirmDialog>
  );
}

function VaultRow({ v, manage }: { v: VaultInfo; manage: boolean }) {
  const select = useSelectVault();
  const update = useUpdateVault();
  const copy = useCopyVault();
  const menu = useContextMenu();
  const [editing, setEditing] = useState(false);
  const [removing, setRemoving] = useState(false);
  const [name, setName] = useState(v.name);
  const [purpose, setPurpose] = useState(v.purpose ?? '');
  const label = t('cofre-acoes-de', { nome: v.name });
  const entries: ContextMenuEntry[] = [
    !v.active && v.exists
      ? {
          label: t('cofre-acao-abrir'),
          icon: 'unlock',
          onSelect: () => select.mutate({ id: v.id }),
        }
      : null,
    { label: t('cofre-acao-renomear'), onSelect: () => setEditing(true) },
    {
      label: t('cofre-acao-copiar'),
      icon: 'copy',
      disabled: !v.exists,
      disabledReason: t('cofre-acao-copiar-indisponivel'),
      onSelect: () =>
        copy.mutate(
          { id: v.id },
          {
            onSuccess: () =>
              toast({
                tone: 'success',
                title: t('cofre-copia-toast-titulo'),
                description: t('cofre-copia-toast-descricao', { nome: v.name }),
              }),
          },
        ),
    },
    {
      label: t('cofre-acao-copiar-caminho'),
      onSelect: () => {
        void navigator.clipboard?.writeText(v.path);
        toast({ tone: 'success', title: t('cofre-caminho-copiado'), description: v.path });
      },
    },
    { separator: true },
    {
      label: t('cofre-acao-remover'),
      icon: 'x',
      danger: true,
      disabled: v.active,
      disabledReason: t('cofre-acao-remover-indisponivel'),
      onSelect: () => setRemoving(true),
    },
  ];
  return (
    <li
      onContextMenu={(e) => menu.onContextMenu(e, { entries, label })}
      onKeyDown={(e) => menu.onKeyDown(e, { entries, label })}
      className={cx(
        'flex flex-col gap-2 rounded-control border px-3 py-2.5',
        v.active ? 'border-accent bg-accent-soft/40' : 'border-line',
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-0.5">
          <span className="text-[13.5px] text-fg">
            {v.name}
            {v.active ? (
              <span className="ml-2 rounded-full bg-accent-soft px-2 py-0.5 text-[11px] text-accent">
                {t('cofre-em-uso')}
              </span>
            ) : null}
          </span>
          {v.purpose ? <span className="text-[12px] text-dim">{v.purpose}</span> : null}
          <span className="truncate font-mono text-[11px] text-dim" title={v.path}>
            {v.path}
            {v.exists ? ` · ${formatBytes(v.bytes)}` : ` · ${t('cofre-ainda-nao-criado')}`}
          </span>
        </div>
        <div className="flex shrink-0 gap-1.5">
          {v.active ? null : (
            <Button
              size="sm"
              variant="secondary"
              loading={select.isPending}
              onClick={() => select.mutate({ id: v.id })}
            >
              {t('cofre-acao-abrir')}
            </Button>
          )}
          {manage ? (
            <Button size="sm" variant="ghost" onClick={() => setEditing((e) => !e)}>
              {editing ? t('comum-fechar') : t('comum-editar')}
            </Button>
          ) : null}
          <Button
            size="sm"
            variant="ghost"
            aria-label={label}
            aria-haspopup="menu"
            title={t('cofre-mais-acoes')}
            onClick={(e) => menu.open({ entries, label, at: e.currentTarget })}
          >
            {'⋯'}
          </Button>
        </div>
      </div>
      {editing ? (
        <form
          className="flex flex-col gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            update.mutate(
              { id: v.id, name, purpose: purpose.trim() ? purpose : null },
              { onSuccess: () => setEditing(false) },
            );
          }}
        >
          <div className="grid gap-2 sm:grid-cols-2">
            <Field label={t('cofre-nome')}>
              <Input value={name} onChange={(e) => setName(e.target.value)} />
            </Field>
            <Field label={t('cofre-finalidade')}>
              <Input value={purpose} onChange={(e) => setPurpose(e.target.value)} />
            </Field>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button size="sm" type="submit" variant="primary" loading={update.isPending}>
              {t('comum-salvar')}
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setEditing(false)}>
              {t('comum-cancelar')}
            </Button>
          </div>
        </form>
      ) : null}
      <InlineError error={select.error ?? update.error ?? copy.error} />
      {menu.menu}
      {removing ? <RemoveVault v={v} onClose={() => setRemoving(false)} /> : null}
    </li>
  );
}

function NewVaultForm({ onDone }: { onDone?: () => void }) {
  const create = useNewVault();
  const [name, setName] = useState('');
  const [purpose, setPurpose] = useState('');
  return (
    <form
      className="flex flex-col gap-2"
      onSubmit={(e) => {
        e.preventDefault();
        create.mutate(
          { name, ...(purpose.trim() ? { purpose } : {}) },
          { onSuccess: () => onDone?.() },
        );
      }}
    >
      <div className="grid gap-2 sm:grid-cols-2">
        <Field label={t('cofre-novo-nome')}>
          <Input
            value={name}
            placeholder={t('cofre-novo-nome-exemplo')}
            onChange={(e) => setName(e.target.value)}
          />
        </Field>
        <Field label={t('cofre-novo-finalidade')}>
          <Input value={purpose} onChange={(e) => setPurpose(e.target.value)} />
        </Field>
      </div>
      <p className="text-[12px] text-dim">{t('cofre-novo-ajuda')}</p>
      <div>
        <Button
          type="submit"
          size="sm"
          variant="primary"
          loading={create.isPending}
          disabled={!name.trim()}
        >
          {t('cofre-novo-botao')}
        </Button>
      </div>
      <InlineError error={create.error} />
    </form>
  );
}

function AddExistingForm({ onDone }: { onDone?: () => void }) {
  const add = useAddVault();
  const [path, setPath] = useState('');
  const [name, setName] = useState('');
  const native = isTauri();
  return (
    <form
      className="flex flex-col gap-2"
      onSubmit={(e) => {
        e.preventDefault();
        add.mutate({ path, ...(name.trim() ? { name } : {}) }, { onSuccess: () => onDone?.() });
      }}
    >
      <Field label={t('cofre-existente-arquivo')} hint={t('cofre-existente-arquivo-dica')}>
        <div className="flex gap-2">
          <Input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            className="min-w-0 flex-1"
          />
          {native ? (
            <Button
              size="sm"
              variant="secondary"
              onClick={() => void pickVaultFile().then((p) => p && setPath(p))}
            >
              {t('cofre-existente-escolher')}
            </Button>
          ) : null}
        </div>
      </Field>
      <Field label={t('cofre-existente-nome')}>
        <Input value={name} onChange={(e) => setName(e.target.value)} />
      </Field>
      <div>
        <Button
          type="submit"
          size="sm"
          variant="primary"
          loading={add.isPending}
          disabled={!path.trim()}
        >
          {t('cofre-existente-abrir')}
        </Button>
      </div>
      <InlineError error={add.error} />
    </form>
  );
}

/** Lista de cofres com trocar, criar e abrir arquivo; `manage` também renomeia e tira da lista. */
export function VaultSwitcher({
  manage = false,
  onDone,
}: {
  manage?: boolean;
  onDone?: () => void;
}) {
  const vaults = useVaults();
  const [adding, setAdding] = useState<'new' | 'file' | null>(null);
  const [demo, setDemo] = useState(false);
  return (
    <div className="flex flex-col gap-3">
      {vaults.data ? (
        <ul aria-label={t('cofre-lista-rotulo')} className="flex flex-col gap-2">
          {vaults.data.vaults.map((v) => (
            <VaultRow key={v.id} v={v} manage={manage} />
          ))}
        </ul>
      ) : null}
      <InlineError error={vaults.error} />
      <div className="flex flex-wrap gap-2">
        <Button
          size="sm"
          variant={adding === 'new' ? 'primary' : 'secondary'}
          icon={<Icon name="plus" size={14} />}
          onClick={() => setAdding(adding === 'new' ? null : 'new')}
        >
          {t('cofre-novo-cofre')}
        </Button>
        <Button
          size="sm"
          variant={adding === 'file' ? 'primary' : 'secondary'}
          onClick={() => setAdding(adding === 'file' ? null : 'file')}
        >
          {t('cofre-abrir-existente')}
        </Button>
      </div>
      {/* A demonstração numa linha própria, na cor dela, separada dos botões do cofre real. */}
      <div className="mt-1 border-t border-line/60 pt-3">
        <Button
          size="sm"
          variant="demo"
          icon={<Icon name="sliders" size={14} />}
          onClick={() => setDemo(true)}
        >
          {t('cofre-demonstracao')}
        </Button>
      </div>
      <DemoQuickDialog
        open={demo}
        onOpenChange={(o) => {
          setDemo(o);
          if (!o) onDone?.();
        }}
      />
      {adding === 'new' ? <NewVaultForm onDone={onDone} /> : null}
      {adding === 'file' ? <AddExistingForm onDone={onDone} /> : null}
    </div>
  );
}

/** Na tela do cofre (desbloquear ou criar): os outros cofres, recolhidos. */
export function OtherVaults({ count }: { count: number }) {
  return (
    <details className="rounded-card border border-line px-4 py-3">
      <summary className="cursor-pointer text-[13px] text-fg">
        {count > 1 ? t('cofre-outros-n', { n: count - 1 }) : t('cofre-outros')}
      </summary>
      <div className="mt-3">
        <VaultSwitcher />
      </div>
    </details>
  );
}
