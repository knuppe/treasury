/**
 * Fontes da meta (docs/goal-sources.md): quanto cada fonte vale hoje e qual valor entrou, as
 * contas e as etiquetas ligadas, e o atalho "Usar meus investimentos de liquidez diária". As
 * posições (conta inteira ou ativo, com fração) ficam no editor de posições logo abaixo.
 */
import { useState } from 'react';

import { isApiError } from '@/api/client';
import type { GoalSourceKind } from '@/api/generated/GoalSourceKind';
import type { GoalSourceValue } from '@/api/generated/GoalSourceValue';
import type { GoalValueUsed } from '@/api/generated/GoalValueUsed';
import type { GoalView } from '@/api/generated/GoalView';
import {
  useAssetLabels,
  useLinkGoalSource,
  useLiquidityCandidates,
  useUnlinkGoalSource,
} from '@/api/hooks/goals';
import { useAccounts } from '@/api/hooks/reports';
import { Button, Field, Icon, Money, Select, toast } from '@/design/components';
import { t } from '@/i18n';

const errorText = (e: unknown) =>
  isApiError(e) || e instanceof Error ? e.message : t('metas-erro-gravar');

const KIND_LABEL: Record<GoalSourceKind, () => string> = {
  initial: () => t('metas-fonte-tipo-inicial'),
  plan: () => t('metas-fonte-tipo-carteira'),
  account: () => t('metas-fonte-tipo-conta'),
  account_positions: () => t('metas-fonte-tipo-conta-inteira'),
  asset: () => t('metas-fonte-tipo-ativo'),
  asset_tag: () => t('metas-fonte-tipo-etiqueta'),
  deposits: () => t('metas-fonte-tipo-lancamentos'),
};

const USED_LABEL: Record<GoalValueUsed, () => string> = {
  balance: () => t('metas-fonte-valor-saldo'),
  market: () => t('metas-fonte-valor-mercado'),
  net_redemption: () => t('metas-fonte-valor-liquido'),
  cost: () => t('metas-fonte-valor-custo'),
  amount: () => t('metas-fonte-valor-informado'),
};

/** Fontes que se desligam aqui (as posições têm o editor próprio). */
const REMOVABLE: readonly GoalSourceKind[] = ['account', 'asset_tag'];

function SourceRow({ goal, s }: { goal: GoalView; s: GoalSourceValue }) {
  const unlink = useUnlinkGoalSource();
  return (
    <li className="flex flex-wrap items-center gap-x-3 gap-y-1 px-3 py-2 text-[12.5px]">
      <span className="min-w-0 flex-1 truncate text-fg">{s.name}</span>
      <span className="label-caps">{KIND_LABEL[s.kind]()}</span>
      <span className="text-[11.5px] text-dim">{USED_LABEL[s.value_used]()}</span>
      <span className="num text-fg">
        <Money cents={s.value} currency={goal.currency} />
      </span>
      {REMOVABLE.includes(s.kind) && s.ref_id ? (
        <Button
          variant="ghost"
          size="sm"
          aria-label={t('metas-fonte-remover', { nome: s.name })}
          loading={unlink.isPending}
          onClick={() =>
            unlink.mutate(
              {
                goal_id: goal.id,
                kind: s.kind === 'account' ? 'account' : 'asset_tag',
                ref_id: s.ref_id!,
              },
              {
                onError: (e) =>
                  toast({ tone: 'error', title: t('metas-fonte-erro'), description: errorText(e) }),
              },
            )
          }
        >
          <Icon name="x" size={12} />
        </Button>
      ) : null}
      {s.ignored_negative != null ? (
        <span className="w-full text-[11.5px] text-dim">{t('metas-fonte-negativo-fora')}</span>
      ) : null}
      {s.stale ? (
        <span className="w-full text-[11.5px] text-warning">{t('metas-fonte-desatualizada')}</span>
      ) : null}
    </li>
  );
}

/** "Usar meus investimentos de liquidez diária": as posições sugeridas, uma a uma ou todas. */
function LiquidityShortcut({ goal }: { goal: GoalView }) {
  const [open, setOpen] = useState(false);
  const candidates = useLiquidityCandidates(goal.id, open);
  const link = useLinkGoalSource();
  const add = async (items: NonNullable<typeof candidates.data>) => {
    try {
      for (const c of items) {
        await link.mutateAsync({
          goal_id: goal.id,
          kind: 'asset',
          ref_id: c.asset_id,
          account_id: c.account_id,
        });
      }
      toast({ tone: 'success', title: t('metas-fonte-ligadas', { n: items.length }) });
    } catch (e) {
      toast({ tone: 'error', title: t('metas-fonte-erro'), description: errorText(e) });
    }
  };
  if (!open) {
    return (
      <Button variant="secondary" size="sm" onClick={() => setOpen(true)}>
        {t('metas-fonte-liquidez')}
      </Button>
    );
  }
  const list = candidates.data ?? [];
  return (
    <div className="flex flex-col gap-2 rounded-control border border-line p-3">
      <p className="text-[12px] text-dim">{t('metas-fonte-liquidez-dica')}</p>
      {candidates.isPending ? (
        <p className="text-[12px] text-dim">{t('metas-fonte-liquidez-buscando')}</p>
      ) : list.length === 0 ? (
        <p className="text-[12px] text-dim">{t('metas-fonte-liquidez-vazio')}</p>
      ) : (
        <>
          <ul className="flex flex-col divide-y divide-line">
            {list.map((c) => (
              <li
                key={`${c.account_id}:${c.asset_id}`}
                className="flex items-center gap-3 py-1.5 text-[12.5px]"
              >
                <span className="min-w-0 flex-1 truncate text-fg">
                  {c.ticker ?? c.name} · {c.account_name}
                </span>
                <span className="num">
                  <Money cents={c.value} currency={c.currency} />
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  loading={link.isPending}
                  onClick={() => void add([c])}
                >
                  {t('metas-fonte-adicionar')}
                </Button>
              </li>
            ))}
          </ul>
          <div>
            <Button
              variant="primary"
              size="sm"
              loading={link.isPending}
              onClick={() => void add(list)}
            >
              {t('metas-fonte-adicionar-todos', { n: list.length })}
            </Button>
          </div>
        </>
      )}
    </div>
  );
}

export function GoalSources({ goal }: { goal: GoalView }) {
  const accounts = useAccounts();
  const labels = useAssetLabels();
  const link = useLinkGoalSource();
  const [account, setAccount] = useState('');
  const [label, setLabel] = useState('');
  const linkOne = (kind: 'account' | 'asset_tag', ref: string, done: () => void) =>
    link.mutate(
      { goal_id: goal.id, kind, ref_id: ref },
      {
        onSuccess: done,
        onError: (e) =>
          toast({ tone: 'error', title: t('metas-fonte-erro'), description: errorText(e) }),
      },
    );
  const linkedAccounts = new Set(
    goal.sources.filter((s) => s.kind === 'account').map((s) => s.ref_id),
  );
  const linkedLabels = new Set(
    goal.sources.filter((s) => s.kind === 'asset_tag').map((s) => s.ref_id),
  );
  const accountOptions = (accounts.data ?? [])
    .filter((a) => !a.archived && a.kind !== 'credit_card' && !linkedAccounts.has(a.id))
    .map((a) => ({ value: a.id, label: a.name }));
  const labelOptions = (labels.data ?? [])
    .filter((l) => !linkedLabels.has(l.id))
    .map((l) => ({ value: l.id, label: l.name }));
  return (
    <section aria-label={t('metas-fontes')} className="flex flex-col gap-3">
      <div className="flex items-baseline justify-between gap-3">
        <h3 className="label-caps text-fg">{t('metas-fontes')}</h3>
        <span className="num text-[12.5px] text-fg">
          <Money cents={goal.current_amount} currency={goal.currency} />
        </span>
      </div>
      <p className="text-[12px] text-dim">{t('metas-fontes-dica')}</p>
      {goal.breakdown.length ? (
        <ul className="flex flex-col divide-y divide-line rounded-control border border-line">
          {goal.breakdown.map((s, i) => (
            <SourceRow key={`${s.kind}:${s.ref_id ?? i}:${s.account_id ?? ''}`} goal={goal} s={s} />
          ))}
        </ul>
      ) : (
        <p className="text-[12.5px] text-dim">{t('metas-fontes-vazio')}</p>
      )}
      <div className="grid items-end gap-3 sm:grid-cols-2">
        <div className="flex items-end gap-2">
          <Field label={t('metas-fonte-conta')} className="flex-1">
            <Select
              value={account}
              onValueChange={setAccount}
              options={accountOptions}
              placeholder={t('metas-fonte-escolha-conta')}
            />
          </Field>
          <Button
            variant="secondary"
            size="sm"
            disabled={!account}
            loading={link.isPending}
            onClick={() => linkOne('account', account, () => setAccount(''))}
          >
            {t('metas-fonte-adicionar')}
          </Button>
        </div>
        <div className="flex items-end gap-2">
          <Field
            label={t('metas-fonte-etiqueta')}
            hint={t('metas-fonte-etiqueta-dica')}
            className="flex-1"
          >
            <Select
              value={label}
              onValueChange={setLabel}
              options={labelOptions}
              placeholder={
                labelOptions.length
                  ? t('metas-fonte-escolha-etiqueta')
                  : t('metas-fonte-sem-etiquetas')
              }
            />
          </Field>
          <Button
            variant="secondary"
            size="sm"
            disabled={!label}
            loading={link.isPending}
            onClick={() => linkOne('asset_tag', label, () => setLabel(''))}
          >
            {t('metas-fonte-adicionar')}
          </Button>
        </div>
      </div>
      <LiquidityShortcut goal={goal} />
    </section>
  );
}
