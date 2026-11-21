import { useState } from 'react';

import type { AccountView } from '@/api/generated/AccountView';
import type { CardPlastic } from '@/api/generated/CardPlastic';
import type { PlasticKind } from '@/api/generated/PlasticKind';
import {
  useCardPlastics,
  useCreatePlastic,
  useDeletePlastic,
  useUpdatePlastic,
} from '@/api/hooks/cards';
import { useMembers } from '@/api/hooks/household';
import {
  Button,
  Dialog,
  EmptyState,
  Field,
  Input,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { ConfirmDialog, InlineError, QueryState } from '@/features/transactions/ui/controls';

export const PLASTIC_KIND: Record<PlasticKind, () => string> = {
  holder: () => t('contas-plastico-tipo-holder'),
  additional: () => t('contas-plastico-tipo-additional'),
  virtual: () => t('contas-plastico-tipo-virtual'),
  temporary: () => t('contas-plastico-tipo-temporary'),
};

/** "Final 1234" ou o tipo, para listas e seletores. */
export function plasticLabel(p: Pick<CardPlastic, 'kind' | 'last4' | 'label'>): string {
  const base = p.last4 ? t('contas-plastico-final', { final: p.last4 }) : PLASTIC_KIND[p.kind]();
  return p.label ? `${base} · ${p.label}` : base;
}

/** Valor do seletor para "titular da conta" (sem membro próprio). */
const OWNER = '__owner__';

function PlasticDialog({
  card,
  plastic,
  onClose,
}: {
  card: AccountView;
  plastic: CardPlastic | null;
  onClose: () => void;
}) {
  const members = useMembers(true).data ?? [];
  const create = useCreatePlastic();
  const update = useUpdatePlastic();
  const [kind, setKind] = useState<PlasticKind>(plastic?.kind ?? 'additional');
  const [last4, setLast4] = useState(plastic?.last4 ?? '');
  const [member, setMember] = useState(plastic?.member_id ?? OWNER);
  const [label, setLabel] = useState(plastic?.label ?? '');
  const [limit, setLimit] = useState<number | null>(plastic?.spend_limit ?? null);
  const [active, setActive] = useState(plastic?.active ?? true);
  const pending = create.isPending || update.isPending;
  const save = () => {
    const done = {
      onSuccess: () => {
        toast({ tone: 'success', title: t('contas-plastico-salvo') });
        onClose();
      },
    };
    const memberId = member === OWNER ? null : member;
    if (plastic) {
      update.mutate(
        {
          id: plastic.id,
          kind,
          last4: last4.trim() || null,
          member_id: memberId,
          label: label.trim() || null,
          spend_limit: limit,
          active,
        },
        done,
      );
    } else {
      create.mutate(
        {
          account_id: card.id,
          kind,
          last4: last4.trim() || undefined,
          member_id: memberId ?? undefined,
          label: label.trim() || undefined,
          spend_limit: limit ?? undefined,
        },
        done,
      );
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => (o ? null : onClose())}
      index="02"
      size="sm"
      title={plastic ? plasticLabel(plastic) : t('contas-plastico-novo')}
      description={t('contas-plastico-dialogo-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button variant="primary" loading={pending} onClick={save}>
            {t('contas-plastico-salvar')}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-2 gap-3">
        <Field label={t('contas-plastico-tipo')}>
          <Select
            value={kind}
            onValueChange={(v) => setKind(v as PlasticKind)}
            options={(Object.keys(PLASTIC_KIND) as PlasticKind[]).map((k) => ({
              value: k,
              label: PLASTIC_KIND[k](),
            }))}
          />
        </Field>
        <Field label={t('contas-plastico-final-rotulo')} hint={t('contas-plastico-final-dica')}>
          <Input
            value={last4}
            inputMode="numeric"
            maxLength={4}
            onChange={(e) => setLast4(e.target.value.replace(/\D/g, '').slice(0, 4))}
            placeholder="1234"
          />
        </Field>
      </div>
      <Field label={t('contas-plastico-membro')}>
        <Select
          value={member}
          onValueChange={setMember}
          options={[
            { value: OWNER, label: t('contas-plastico-titular-conta') },
            ...members.map((m) => ({ value: m.id, label: m.name })),
          ]}
        />
      </Field>
      <Field label={t('contas-plastico-apelido')}>
        <Input
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          placeholder={t('contas-plastico-apelido-exemplo')}
        />
      </Field>
      <Field label={t('contas-plastico-sublimite')} hint={t('contas-plastico-sublimite-dica')}>
        <MoneyInput currency={card.currency} value={limit} onValueChange={setLimit} />
      </Field>
      {plastic ? (
        <label className="flex items-center gap-2 text-[13px]">
          <input type="checkbox" checked={active} onChange={(e) => setActive(e.target.checked)} />
          {t('contas-plastico-ativo')}
        </label>
      ) : null}
      <InlineError error={create.error ?? update.error} />
    </Dialog>
  );
}

/** Uso do sublimite: aviso a partir de 80% e ao passar de 100%. */
function LimitUse({ p, currency }: { p: CardPlastic; currency: string }) {
  if (p.spend_limit == null || p.spend_limit <= 0) return null;
  const share = p.cycle_spent / p.spend_limit;
  const tone = share >= 1 ? 'text-negative' : share >= 0.8 ? 'text-warning' : 'text-dim';
  return (
    <span className={cx('text-[12px]', tone)}>
      {t('contas-plastico-uso-sublimite', { pct: Math.round(share * 100) })}{' '}
      <Money cents={p.spend_limit} currency={currency} />
      {share >= 1
        ? ` · ${t('contas-plastico-acima-sublimite')}`
        : share >= 0.8
          ? ` · ${t('contas-plastico-perto-sublimite')}`
          : ''}
    </span>
  );
}

/** Cartões da conta (titular, adicionais e virtuais) e quem usa cada um. */
export function PlasticsPanel({ card }: { card: AccountView }) {
  const { data = [], isLoading, error } = useCardPlastics(card.id);
  const members = useMembers(true).data ?? [];
  const remove = useDeletePlastic();
  const [editing, setEditing] = useState<CardPlastic | 'new' | null>(null);
  const [removing, setRemoving] = useState<CardPlastic | null>(null);
  const owner =
    card.member_ids.length === 1
      ? (members.find((m) => m.id === card.member_ids[0])?.name ?? null)
      : null;
  return (
    <div className="flex flex-col gap-3">
      <p className="max-w-[70ch] text-[13px] text-dim">{t('contas-plasticos-explicacao')}</p>
      <QueryState loading={isLoading} error={error}>
        {data.length === 0 ? (
          <EmptyState title={t('contas-plasticos-vazio')} />
        ) : (
          <ul
            className="flex flex-col divide-y divide-line/60 rounded-card border border-line"
            aria-label={t('contas-cartoes')}
          >
            {data.map((p) => (
              <li
                key={p.id}
                className={cx(
                  'flex flex-wrap items-center gap-x-4 gap-y-1 px-3 py-2 text-[13px]',
                  !p.active && 'opacity-60',
                )}
              >
                <span className="min-w-36 font-medium">{plasticLabel(p)}</span>
                <span className="label-caps">{PLASTIC_KIND[p.kind]()}</span>
                <span className="text-dim">
                  {p.member_name ??
                    (owner
                      ? t('contas-plastico-do-titular', { nome: owner })
                      : t('contas-plastico-titular-conta'))}
                </span>
                <span className="flex flex-col">
                  <span className="text-[12px] text-dim">
                    {t('contas-plastico-gasto-ciclo')}{' '}
                    <Money cents={p.cycle_spent} currency={card.currency} />
                  </span>
                  <LimitUse p={p} currency={card.currency} />
                </span>
                {!p.active ? (
                  <span className="label-caps text-dim">{t('contas-plastico-inativo')}</span>
                ) : null}
                <span className="ml-auto flex gap-2">
                  <Button size="sm" variant="ghost" onClick={() => setEditing(p)}>
                    {t('contas-plastico-editar')}
                  </Button>
                  <Button size="sm" variant="ghost" onClick={() => setRemoving(p)}>
                    {t('contas-plastico-remover')}
                  </Button>
                </span>
              </li>
            ))}
          </ul>
        )}
      </QueryState>
      <Button
        size="sm"
        variant="secondary"
        className="self-start"
        onClick={() => setEditing('new')}
      >
        {t('contas-plastico-novo')}
      </Button>
      {editing ? (
        <PlasticDialog
          card={card}
          plastic={editing === 'new' ? null : editing}
          onClose={() => setEditing(null)}
        />
      ) : null}
      {removing ? (
        <ConfirmDialog
          open
          onOpenChange={(o) => (o ? null : setRemoving(null))}
          title={t('contas-plastico-remover-titulo', { cartao: plasticLabel(removing) })}
          description={t('contas-plastico-remover-confirmar')}
          confirmLabel={t('contas-plastico-remover')}
          pending={remove.isPending}
          error={remove.error}
          onConfirm={() =>
            remove.mutate(
              { id: removing.id },
              {
                onSuccess: () => {
                  toast({ tone: 'success', title: t('contas-plastico-removido') });
                  setRemoving(null);
                },
              },
            )
          }
        />
      ) : null}
    </div>
  );
}
