import { useState } from 'react';

import type { EntryKind } from '@/api/generated/EntryKind';
import type { LoyaltyKind } from '@/api/generated/LoyaltyKind';
import type { LoyaltyProgram } from '@/api/generated/LoyaltyProgram';
import {
  useAddLoyaltyEntry,
  useCreateLoyalty,
  useDeleteLoyaltyEntry,
  useLoyaltyEntries,
  useLoyaltyPrograms,
  useLoyaltyTransfer,
  useUpdateLoyalty,
} from '@/api/hooks/cards';
import { useMembers } from '@/api/hooks/household';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Input,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { InlineError, QueryState, SectionTitle } from '@/features/transactions/ui/controls';
import { useBaseCurrency } from '@/state/currency';

const KIND: Record<LoyaltyKind, () => string> = {
  bank: () => t('contas-pontos-tipo-bank'),
  airline: () => t('contas-pontos-tipo-airline'),
  retail: () => t('contas-pontos-tipo-retail'),
  other: () => t('contas-pontos-tipo-other'),
};

const ENTRY: Record<EntryKind, () => string> = {
  earn: () => t('contas-pontos-mov-earn'),
  bonus: () => t('contas-pontos-mov-bonus'),
  purchase: () => t('contas-pontos-mov-purchase'),
  transfer_in: () => t('contas-pontos-mov-transfer_in'),
  transfer_out: () => t('contas-pontos-mov-transfer_out'),
  redeem: () => t('contas-pontos-mov-redeem'),
  expire: () => t('contas-pontos-mov-expire'),
  adjust: () => t('contas-pontos-mov-adjust'),
};

const pts = (n: number) => n.toLocaleString();
const NO_MEMBER = '__none__';

function ProgramDialog({
  program,
  onClose,
}: {
  program: LoyaltyProgram | null;
  onClose: () => void;
}) {
  const members = useMembers(true).data ?? [];
  const base = useBaseCurrency();
  const create = useCreateLoyalty();
  const update = useUpdateLoyalty();
  const [name, setName] = useState(program?.name ?? '');
  const [kind, setKind] = useState<LoyaltyKind>(program?.kind ?? 'bank');
  const [member, setMember] = useState(program?.member_id ?? NO_MEMBER);
  const [value, setValue] = useState<number | null>(program?.value_per_1000 ?? null);
  const done = {
    onSuccess: () => {
      toast({ tone: 'success', title: t('contas-pontos-programa-salvo') });
      onClose();
    },
  };
  const memberId = member === NO_MEMBER ? null : member;
  return (
    <Dialog
      open
      onOpenChange={(o) => (o ? null : onClose())}
      index="02"
      size="sm"
      title={program ? program.name : t('contas-pontos-programa-novo')}
      description={t('contas-pontos-programa-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={create.isPending || update.isPending}
            disabled={!name.trim()}
            onClick={() =>
              program
                ? update.mutate(
                    { id: program.id, name, kind, member_id: memberId, value_per_1000: value },
                    done,
                  )
                : create.mutate(
                    {
                      name,
                      kind,
                      ...(memberId ? { member_id: memberId } : {}),
                      ...(value != null ? { value_per_1000: value } : {}),
                    },
                    done,
                  )
            }
          >
            {t('contas-plastico-salvar')}
          </Button>
        </>
      }
    >
      <Field label={t('contas-pontos-nome')}>
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Livelo, Smiles…"
        />
      </Field>
      <div className="grid grid-cols-2 gap-3">
        <Field label={t('contas-plastico-tipo')}>
          <Select
            value={kind}
            onValueChange={(v) => setKind(v as LoyaltyKind)}
            options={(Object.keys(KIND) as LoyaltyKind[]).map((k) => ({
              value: k,
              label: KIND[k](),
            }))}
          />
        </Field>
        <Field label={t('contas-pontos-membro')}>
          <Select
            value={member}
            onValueChange={setMember}
            options={[
              { value: NO_MEMBER, label: t('contas-pontos-sem-membro') },
              ...members.map((m) => ({ value: m.id, label: m.name })),
            ]}
          />
        </Field>
      </div>
      <Field
        label={t('contas-pontos-valor-milheiro')}
        hint={t('contas-pontos-valor-milheiro-dica')}
      >
        <MoneyInput currency={base} value={value} onValueChange={setValue} />
      </Field>
      <InlineError error={create.error ?? update.error} />
    </Dialog>
  );
}

function EntryDialog({ programs, onClose }: { programs: LoyaltyProgram[]; onClose: () => void }) {
  const base = useBaseCurrency();
  const add = useAddLoyaltyEntry();
  const [program, setProgram] = useState(programs[0]?.id ?? '');
  const [kind, setKind] = useState<EntryKind>('earn');
  const [points, setPoints] = useState('');
  const [date, setDate] = useState<string | null>(todayIso());
  const [expires, setExpires] = useState<string | null>(null);
  const [cost, setCost] = useState<number | null>(null);
  const entering = ['earn', 'bonus', 'purchase'].includes(kind);
  return (
    <Dialog
      open
      onOpenChange={(o) => (o ? null : onClose())}
      index="02"
      size="sm"
      title={t('contas-pontos-movimento-novo')}
      description={t('contas-pontos-movimento-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={add.isPending}
            disabled={!program || !points || !date}
            onClick={() =>
              add.mutate(
                {
                  program_id: program,
                  kind,
                  points: Number(points),
                  date: date!,
                  ...(entering && expires ? { expires_on: expires } : {}),
                  ...(kind === 'purchase' && cost != null ? { cost } : {}),
                },
                {
                  onSuccess: () => {
                    toast({ tone: 'success', title: t('contas-pontos-movimento-salvo') });
                    onClose();
                  },
                },
              )
            }
          >
            {t('contas-plastico-salvar')}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-2 gap-3">
        <Field label={t('contas-pontos-programa')}>
          <Select
            value={program || undefined}
            onValueChange={setProgram}
            options={programs.map((p) => ({ value: p.id, label: p.name }))}
          />
        </Field>
        <Field label={t('contas-plastico-tipo')}>
          <Select
            value={kind}
            onValueChange={(v) => setKind(v as EntryKind)}
            options={(
              ['earn', 'bonus', 'purchase', 'redeem', 'expire', 'adjust'] as EntryKind[]
            ).map((k) => ({ value: k, label: ENTRY[k]() }))}
          />
        </Field>
        <Field label={t('contas-pontos-quantidade')}>
          <Input
            value={points}
            inputMode="numeric"
            onChange={(e) =>
              setPoints(e.target.value.replace(kind === 'adjust' ? /[^\d-]/g : /\D/g, ''))
            }
          />
        </Field>
        <Field label={t('contas-data')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        {entering ? (
          <Field label={t('contas-pontos-vence-em')}>
            <DateInput value={expires} onValueChange={setExpires} />
          </Field>
        ) : null}
        {kind === 'purchase' ? (
          <Field label={t('contas-pontos-custo')}>
            <MoneyInput currency={base} value={cost} onValueChange={setCost} />
          </Field>
        ) : null}
      </div>
      <InlineError error={add.error} />
    </Dialog>
  );
}

function TransferDialog({
  programs,
  onClose,
}: {
  programs: LoyaltyProgram[];
  onClose: () => void;
}) {
  const move = useLoyaltyTransfer();
  const [from, setFrom] = useState(programs[0]?.id ?? '');
  const [to, setTo] = useState(programs[1]?.id ?? '');
  const [out, setOut] = useState('');
  const [bonus, setBonus] = useState('0');
  const [date, setDate] = useState<string | null>(todayIso());
  const [expires, setExpires] = useState<string | null>(null);
  const inPoints = Math.round(Number(out || 0) * (1 + Number(bonus || 0) / 100));
  const options = programs.map((p) => ({ value: p.id, label: p.name }));
  return (
    <Dialog
      open
      onOpenChange={(o) => (o ? null : onClose())}
      index="02"
      size="sm"
      title={t('contas-pontos-transferir')}
      description={t('contas-pontos-transferir-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={move.isPending}
            disabled={!from || !to || from === to || !out || !date}
            onClick={() =>
              move.mutate(
                {
                  from_program_id: from,
                  to_program_id: to,
                  points_out: Number(out),
                  points_in: inPoints,
                  date: date!,
                  ...(expires ? { expires_on: expires } : {}),
                },
                {
                  onSuccess: () => {
                    toast({ tone: 'success', title: t('contas-pontos-transferencia-salva') });
                    onClose();
                  },
                },
              )
            }
          >
            {t('contas-pontos-transferir')}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-2 gap-3">
        <Field label={t('contas-pontos-de')}>
          <Select value={from || undefined} onValueChange={setFrom} options={options} />
        </Field>
        <Field label={t('contas-pontos-para')}>
          <Select value={to || undefined} onValueChange={setTo} options={options} />
        </Field>
        <Field label={t('contas-pontos-saem')}>
          <Input
            value={out}
            inputMode="numeric"
            onChange={(e) => setOut(e.target.value.replace(/\D/g, ''))}
          />
        </Field>
        <Field label={t('contas-pontos-bonus')}>
          <Input
            value={bonus}
            inputMode="numeric"
            onChange={(e) => setBonus(e.target.value.replace(/\D/g, '').slice(0, 3))}
          />
        </Field>
        <Field label={t('contas-data')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field label={t('contas-pontos-vence-em')}>
          <DateInput value={expires} onValueChange={setExpires} />
        </Field>
      </div>
      <p className="text-[13px]">{t('contas-pontos-entram', { n: pts(inPoints) })}</p>
      <InlineError error={move.error} />
    </Dialog>
  );
}

function Entries({ program }: { program: LoyaltyProgram }) {
  const { data = [], isLoading, error } = useLoyaltyEntries(program.id);
  const remove = useDeleteLoyaltyEntry();
  return (
    <QueryState loading={isLoading} error={error}>
      {data.length === 0 ? (
        <p className="px-3 py-2 text-[12.5px] text-dim">{t('contas-pontos-sem-movimentos')}</p>
      ) : (
        <ul className="flex flex-col divide-y divide-line/40">
          {data.map((e) => (
            <li
              key={e.id}
              className="flex flex-wrap items-baseline gap-x-4 px-3 py-1 text-[12.5px]"
            >
              <span className="num w-24 text-dim">{formatDate(e.date)}</span>
              <span className="w-32">{ENTRY[e.kind]()}</span>
              <span className={cx('num', e.points < 0 ? 'text-negative' : 'text-positive')}>
                {e.points > 0 ? '+' : ''}
                {pts(e.points)}
              </span>
              {e.expires_on ? (
                <span className="text-dim">
                  {t('contas-pontos-vence', { data: formatDate(e.expires_on) })}
                </span>
              ) : null}
              <Button
                size="sm"
                variant="ghost"
                className="ml-auto"
                loading={remove.isPending && remove.variables?.id === e.id}
                onClick={() => remove.mutate({ id: e.id })}
              >
                {t('contas-remover')}
              </Button>
            </li>
          ))}
        </ul>
      )}
    </QueryState>
  );
}

/** Pontos e milhas da família: programas por membro, saldo, vencimentos e valor estimado. */
export function LoyaltyPanel() {
  const { data = [], isLoading, error } = useLoyaltyPrograms();
  const [editing, setEditing] = useState<LoyaltyProgram | 'new' | null>(null);
  const [entry, setEntry] = useState(false);
  const [transfer, setTransfer] = useState(false);
  const [open, setOpen] = useState<string | null>(null);
  return (
    <section
      aria-label={t('contas-pontos-titulo')}
      className="flex flex-col gap-3 rounded-card border border-line bg-surface p-5"
    >
      <SectionTitle
        aside={
          <span className="flex flex-wrap gap-2">
            {data.length > 0 ? (
              <Button size="sm" variant="secondary" onClick={() => setEntry(true)}>
                {t('contas-pontos-movimento-novo')}
              </Button>
            ) : null}
            {data.length > 1 ? (
              <Button size="sm" variant="ghost" onClick={() => setTransfer(true)}>
                {t('contas-pontos-transferir')}
              </Button>
            ) : null}
            <Button size="sm" variant="ghost" onClick={() => setEditing('new')}>
              {t('contas-pontos-programa-novo')}
            </Button>
          </span>
        }
      >
        {t('contas-pontos-titulo')}
      </SectionTitle>
      <QueryState loading={isLoading} error={error}>
        {data.length === 0 ? (
          <p className="max-w-[70ch] text-[13px] text-dim">{t('contas-pontos-vazio')}</p>
        ) : (
          <ul className="flex flex-col divide-y divide-line/60 rounded-card border border-line">
            {data.map((p) => (
              <li key={p.id} className={cx(!p.active && 'opacity-60')}>
                <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1 px-3 py-2 text-[13px]">
                  <button
                    type="button"
                    className="min-w-40 text-left font-medium hover:text-accent"
                    aria-expanded={open === p.id}
                    onClick={() => setOpen(open === p.id ? null : p.id)}
                  >
                    {p.name}
                  </button>
                  <span className="label-caps">{KIND[p.kind]()}</span>
                  <span className="text-dim">{p.member_name ?? t('contas-pontos-sem-membro')}</span>
                  <span className="num">{t('contas-pontos-n', { n: pts(p.balance) })}</span>
                  {p.estimated_value != null ? (
                    <span className="text-dim">
                      ≈ <Money cents={p.estimated_value} currency={p.value_currency ?? undefined} />
                    </span>
                  ) : null}
                  {p.next_expiry ? (
                    <span className={cx(p.expiring_90d > 0 ? 'text-warning' : 'text-dim')}>
                      {t('contas-pontos-proximo-vencimento', {
                        n: pts(p.next_expiry_points),
                        data: formatDate(p.next_expiry),
                      })}
                    </span>
                  ) : null}
                  {p.cards.length > 0 ? (
                    <span className="text-dim">
                      {t('contas-pontos-cartoes', {
                        cartoes: p.cards.map((c) => c.account_name).join(', '),
                      })}
                    </span>
                  ) : null}
                  <Button
                    size="sm"
                    variant="ghost"
                    className="ml-auto"
                    onClick={() => setEditing(p)}
                  >
                    {t('contas-plastico-editar')}
                  </Button>
                </div>
                {open === p.id ? <Entries program={p} /> : null}
              </li>
            ))}
          </ul>
        )}
        <p className="text-[12px] text-dim">{t('contas-pontos-fora-patrimonio')}</p>
      </QueryState>
      {editing ? (
        <ProgramDialog
          program={editing === 'new' ? null : editing}
          onClose={() => setEditing(null)}
        />
      ) : null}
      {entry ? <EntryDialog programs={data} onClose={() => setEntry(false)} /> : null}
      {transfer ? <TransferDialog programs={data} onClose={() => setTransfer(false)} /> : null}
    </section>
  );
}
