import { useState } from 'react';

import type { AccountView } from '@/api/generated/AccountView';
import type { RewardDestination } from '@/api/generated/RewardDestination';
import type { RewardKind } from '@/api/generated/RewardKind';
import type { RewardsView } from '@/api/generated/RewardsView';
import { useAccounts } from '@/api/hooks/accounts';
import {
  useCardRewards,
  useEndRewardProgram,
  useLoyaltyPrograms,
  useSetRewardProgram,
} from '@/api/hooks/cards';

const NONE = '__none__';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Input,
  KeyValue,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate, formatMonthLong, todayIso } from '@/lib/format';
import {
  ConfirmDialog,
  InlineError,
  QueryState,
  SectionTitle,
} from '@/features/transactions/ui/controls';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';

/** Taxa em texto: cashback como fração ("0.015" → "1,5%"); pontos como número. */
function rateLabel(kind: RewardKind, rate: string, currency: string | null): string {
  const n = Number(rate);
  return kind === 'cashback'
    ? `${(n * 100).toLocaleString(undefined, { maximumFractionDigits: 2 })}%`
    : t('contas-recompensa-pontos-por', {
        n: n.toLocaleString(undefined, { maximumFractionDigits: 2 }),
        moeda: currency ?? '',
      });
}

interface MultiplierDraft {
  category_id: string | null;
  rate: string;
}

function ProgramDialog({
  card,
  view,
  onClose,
}: {
  card: AccountView;
  view: RewardsView;
  onClose: () => void;
}) {
  const save = useSetRewardProgram();
  const accounts = (useAccounts(false).data ?? []).filter((a) => a.kind !== 'credit_card');
  const p = view.program;
  // Cashback é digitado em % (1,5) e gravado como fração (0.015).
  const toInput = (kind: RewardKind, r: string) =>
    kind === 'cashback' ? String(Number(r) * 100) : r;
  const fromInput = (kind: RewardKind, r: string) => {
    const n = Number(r.replace(',', '.'));
    return kind === 'cashback' ? String(n / 100) : String(n);
  };
  const [kind, setKind] = useState<RewardKind>(p?.kind ?? 'cashback');
  const [base, setBase] = useState(p ? toInput(p.kind, p.base_rate) : '1');
  const [rateCurrency, setRateCurrency] = useState(p?.rate_currency ?? '');
  const [cap, setCap] = useState<number | null>(p?.cycle_cap ?? null);
  const [destination, setDestination] = useState<RewardDestination>(p?.destination ?? 'statement');
  const [destAccount, setDestAccount] = useState(p?.destination_account_id ?? '');
  const [payout, setPayout] = useState(String(p?.payout_days ?? 40));
  const [starts, setStarts] = useState<string | null>(todayIso());
  const programs = useLoyaltyPrograms().data ?? [];
  const [loyalty, setLoyalty] = useState(p?.loyalty_program_id ?? NONE);
  const [multipliers, setMultipliers] = useState<MultiplierDraft[]>(
    p?.multipliers.map((m) => ({ category_id: m.category_id, rate: toInput(p.kind, m.rate) })) ??
      [],
  );
  const submit = () =>
    save.mutate(
      {
        account_id: card.id,
        kind,
        base_rate: fromInput(kind, base),
        ...(kind === 'points' && rateCurrency ? { rate_currency: rateCurrency } : {}),
        ...(cap != null ? { cycle_cap: cap } : {}),
        destination,
        ...(destination !== 'statement' && destAccount
          ? { destination_account_id: destAccount }
          : {}),
        payout_days: Number(payout) || 0,
        ...(kind === 'points' && loyalty !== NONE ? { loyalty_program_id: loyalty } : {}),
        ...(starts ? { starts_on: starts } : {}),
        multipliers: multipliers
          .filter((m) => m.category_id && m.rate.trim())
          .map((m) => ({ category_id: m.category_id!, rate: fromInput(kind, m.rate) })),
      },
      {
        onSuccess: () => {
          toast({ tone: 'success', title: t('contas-recompensa-salva') });
          onClose();
        },
      },
    );
  return (
    <Dialog
      open
      onOpenChange={(o) => (o ? null : onClose())}
      index="02"
      size="md"
      title={t('contas-recompensa-regra')}
      description={t('contas-recompensa-regra-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button variant="primary" loading={save.isPending} onClick={submit}>
            {t('contas-plastico-salvar')}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-2 gap-3">
        <Field label={t('contas-recompensa-tipo')}>
          <Select
            value={kind}
            onValueChange={(v) => setKind(v as RewardKind)}
            options={[
              { value: 'cashback', label: t('contas-recompensa-cashback') },
              { value: 'points', label: t('contas-recompensa-pontos') },
            ]}
          />
        </Field>
        <Field
          label={
            kind === 'cashback'
              ? t('contas-recompensa-taxa-cashback')
              : t('contas-recompensa-taxa-pontos')
          }
        >
          <Input value={base} inputMode="decimal" onChange={(e) => setBase(e.target.value)} />
        </Field>
        {kind === 'points' ? (
          <Field label={t('contas-recompensa-programa-pontos')}>
            <Select
              value={loyalty}
              onValueChange={setLoyalty}
              options={[
                { value: NONE, label: t('contas-recompensa-programa-pontos-nenhum') },
                ...programs.map((x) => ({ value: x.id, label: x.name })),
              ]}
            />
          </Field>
        ) : null}
        {kind === 'points' ? (
          <Field
            label={t('contas-recompensa-moeda-pontos')}
            hint={t('contas-recompensa-moeda-pontos-dica')}
          >
            <Input
              value={rateCurrency}
              maxLength={3}
              placeholder={card.currency}
              onChange={(e) =>
                setRateCurrency(e.target.value.replace(/[^a-z]/gi, '').toUpperCase())
              }
            />
          </Field>
        ) : null}
        <Field
          label={
            kind === 'cashback'
              ? t('contas-recompensa-teto-cashback')
              : t('contas-recompensa-teto-pontos')
          }
        >
          {kind === 'cashback' ? (
            <MoneyInput currency={card.currency} value={cap} onValueChange={setCap} />
          ) : (
            <Input
              value={cap == null ? '' : String(cap)}
              inputMode="numeric"
              onChange={(e) => {
                const v = e.target.value.replace(/\D/g, '');
                setCap(v ? Number(v) : null);
              }}
            />
          )}
        </Field>
        {kind === 'cashback' ? (
          <>
            <Field label={t('contas-recompensa-destino')}>
              <Select
                value={destination}
                onValueChange={(v) => setDestination(v as RewardDestination)}
                options={[
                  { value: 'statement', label: t('contas-recompensa-destino-fatura') },
                  { value: 'account', label: t('contas-recompensa-destino-conta') },
                  { value: 'investment', label: t('contas-recompensa-destino-investimento') },
                ]}
              />
            </Field>
            {destination !== 'statement' ? (
              <Field label={t('contas-recompensa-conta-destino')}>
                <Select
                  value={destAccount || undefined}
                  onValueChange={setDestAccount}
                  options={accounts.map((a) => ({ value: a.id, label: a.name }))}
                  placeholder={t('contas-escolha-conta')}
                />
              </Field>
            ) : null}
            <Field label={t('contas-recompensa-prazo')} hint={t('contas-recompensa-prazo-dica')}>
              <Input
                value={payout}
                inputMode="numeric"
                onChange={(e) => setPayout(e.target.value.replace(/\D/g, '').slice(0, 3))}
              />
            </Field>
          </>
        ) : null}
        <Field label={t('contas-anuidade-inicio')}>
          <DateInput value={starts} onValueChange={setStarts} />
        </Field>
      </div>
      <div className="flex flex-col gap-2">
        <span className="label-caps">{t('contas-recompensa-por-categoria')}</span>
        {multipliers.map((m, i) => (
          <div key={i} className="flex items-end gap-2">
            <CategoryPicker
              kind="expense"
              value={m.category_id}
              allowNone={false}
              onChange={(category_id) =>
                setMultipliers(multipliers.map((x, j) => (j === i ? { ...x, category_id } : x)))
              }
              className="flex-1 [&>button]:h-9 [&>button]:w-full"
            />
            <Input
              className="w-24"
              value={m.rate}
              inputMode="decimal"
              aria-label={t('contas-recompensa-taxa-categoria')}
              onChange={(e) =>
                setMultipliers(
                  multipliers.map((x, j) => (j === i ? { ...x, rate: e.target.value } : x)),
                )
              }
            />
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setMultipliers(multipliers.filter((_, j) => j !== i))}
            >
              {t('contas-remover')}
            </Button>
          </div>
        ))}
        <Button
          size="sm"
          variant="ghost"
          className="self-start"
          onClick={() => setMultipliers([...multipliers, { category_id: null, rate: '' }])}
        >
          {t('contas-recompensa-nova-categoria')}
        </Button>
      </div>
      <InlineError error={save.error} />
    </Dialog>
  );
}

/** Cashback e pontos do cartão: a regra e o previsto × recebido de cada fatura. */
export function RewardsPanel({ card }: { card: AccountView }) {
  const { data, isLoading, error } = useCardRewards(card.id);
  const end = useEndRewardProgram();
  const [editing, setEditing] = useState(false);
  const [ending, setEnding] = useState(false);
  const p = data?.program ?? null;
  const cashback = p?.kind === 'cashback';
  const value = (v: number | null) =>
    v == null ? (
      '—'
    ) : cashback ? (
      <Money cents={v} currency={data!.currency} />
    ) : (
      t('contas-recompensa-n-pontos', { n: v })
    );
  return (
    <QueryState loading={isLoading} error={error}>
      {data ? (
        <div className="flex flex-col gap-4">
          <SectionTitle
            aside={
              <span className="flex gap-2">
                <Button size="sm" variant="secondary" onClick={() => setEditing(true)}>
                  {p ? t('contas-anuidade-mudar') : t('contas-recompensa-informar')}
                </Button>
                {p ? (
                  <Button size="sm" variant="ghost" onClick={() => setEnding(true)}>
                    {t('contas-recompensa-sem')}
                  </Button>
                ) : null}
              </span>
            }
          >
            {t('contas-recompensa-titulo')}
          </SectionTitle>
          {!p ? (
            <p className="max-w-[70ch] text-[13px] text-dim">{t('contas-recompensa-vazio')}</p>
          ) : (
            <>
              <KeyValue
                items={[
                  {
                    label: t('contas-recompensa-tipo'),
                    value: cashback
                      ? t('contas-recompensa-cashback')
                      : t('contas-recompensa-pontos'),
                  },
                  {
                    label: t('contas-recompensa-taxa'),
                    value: rateLabel(p.kind, p.base_rate, p.rate_currency ?? data.currency),
                  },
                  ...p.multipliers.map((m) => ({
                    label: m.category_path ?? '',
                    value: rateLabel(p.kind, m.rate, p.rate_currency ?? data.currency),
                  })),
                  {
                    label: t('contas-recompensa-previsto-12m'),
                    value: value(data.expected_12m),
                  },
                  ...(cashback
                    ? [
                        {
                          label: t('contas-recompensa-recebido-12m'),
                          value: <Money cents={data.received_12m} currency={data.currency} />,
                        },
                      ]
                    : []),
                ]}
              />
              {cashback ? (
                <p className="max-w-[70ch] text-[12.5px] text-dim">
                  {t('contas-recompensa-como-conta')}
                </p>
              ) : null}
              <ul
                className="flex flex-col divide-y divide-line/60 rounded-card border border-line"
                aria-label={t('contas-recompensa-por-fatura')}
              >
                {data.statements.map((s) => (
                  <li
                    key={s.statement_id}
                    className="grid grid-cols-[1fr_auto_auto] items-baseline gap-x-4 px-3 py-1.5 text-[12.5px]"
                  >
                    <span>
                      {formatMonthLong(s.reference_month)}{' '}
                      <span className="text-dim">
                        · <Money cents={s.eligible_spend} currency={data.currency} />
                      </span>
                    </span>
                    <span className="text-dim">
                      {t('contas-recompensa-previsto')} {value(s.expected)}
                    </span>
                    <span
                      className={cx(
                        s.missing ? 'text-warning' : 'text-dim',
                        s.received == null && 'hidden',
                      )}
                    >
                      {t('contas-recompensa-recebido')} {value(s.received)}
                      {s.missing
                        ? ` · ${t('contas-recompensa-nao-veio', { data: formatDate(s.due_by) })}`
                        : ''}
                    </span>
                  </li>
                ))}
              </ul>
            </>
          )}
          {editing ? (
            <ProgramDialog card={card} view={data} onClose={() => setEditing(false)} />
          ) : null}
          {ending ? (
            <ConfirmDialog
              open
              tone="primary"
              onOpenChange={(o) => (o ? null : setEnding(false))}
              title={t('contas-recompensa-sem-titulo')}
              description={t('contas-anuidade-sem-descricao')}
              confirmLabel={t('contas-recompensa-sem')}
              pending={end.isPending}
              error={end.error}
              onConfirm={() =>
                end.mutate({ account_id: card.id }, { onSuccess: () => setEnding(false) })
              }
            />
          ) : null}
        </div>
      ) : null}
    </QueryState>
  );
}
