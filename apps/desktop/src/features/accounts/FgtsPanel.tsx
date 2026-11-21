/**
 * Conta FGTS na página da conta (docs/brazil/fgts.md): o vínculo (empregador, datas, valor para fins
 * rescisórios e a multa de 40%), a importação do extrato da Caixa com a conferência, os
 * lançamentos classificados, a conferência do JAM e dos depósitos com o holerite, o
 * rendimento comparado com IPCA, poupança, CDI e M2, o saque-aniversário e os saques (inclusive
 * o uso no financiamento). Só números com fonte: nada de recomendação.
 */
import { useState } from 'react';

import type { AccountView } from '@/api/generated/AccountView';
import type { FgtsBenchmark } from '@/api/generated/FgtsBenchmark';
import type { FgtsDepositCheck } from '@/api/generated/FgtsDepositCheck';
import type { FgtsEntryView } from '@/api/generated/FgtsEntryView';
import type { FgtsStatementPreview } from '@/api/generated/FgtsStatementPreview';
import type { FgtsYearRow } from '@/api/generated/FgtsYearRow';
import { useAccounts } from '@/api/hooks/accounts';
import { useLoans } from '@/api/hooks/debt';
import {
  previewFgtsStatement,
  useFgtsAccounts,
  useFgtsAnniversary,
  useFgtsCheck,
  useFgtsCompare,
  useFgtsEntries,
  useFgtsHousing,
  useFgtsModality,
  useFgtsWithdrawal,
  useImportFgtsStatement,
} from '@/api/hooks/fgts';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Icon,
  Input,
  KeyValue,
  Money,
  MoneyInput,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { fileToBase64 } from '@/features/transactions/ui/hooks';
import { FileInput, InlineError, QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';

const REASONS = [
  'rescisao',
  'aposentadoria',
  'moradia',
  'aniversario',
  'doenca',
  'emergencial',
  'outros',
] as const;
const REASON_KEY = {
  rescisao: 'fgts-motivo-rescisao',
  aposentadoria: 'fgts-motivo-aposentadoria',
  moradia: 'fgts-motivo-moradia',
  aniversario: 'fgts-motivo-aniversario',
  doenca: 'fgts-motivo-doenca',
  emergencial: 'fgts-motivo-emergencial',
  outros: 'fgts-motivo-outros',
} as const;
const reasonLabel = (r: string) =>
  t(REASON_KEY[r as keyof typeof REASON_KEY] ?? 'fgts-motivo-outros');
const pct = (v: number | null | undefined) =>
  v == null ? '—' : `${(v * 100).toLocaleString(undefined, { maximumFractionDigits: 2 })}%`;
const NONE = '__none__';

/** O vínculo e a multa da demissão (cabeçalho da conta FGTS). */
export function FgtsSummary({ account }: { account: AccountView }) {
  const q = useFgtsAccounts();
  const f = q.data?.find((a) => a.account_id === account.id);
  if (!f) return null;
  return (
    <KeyValue
      items={[
        { label: t('fgts-empregador'), value: f.employer_name ?? '—' },
        { label: t('fgts-cnpj'), value: f.employer_cnpj ?? '—' },
        {
          label: t('fgts-admissao'),
          value: f.admission_date ? formatDate(f.admission_date) : '—',
        },
        {
          label: t('fgts-situacao'),
          value: f.active
            ? t('fgts-ativa')
            : t('fgts-inativa', { data: f.leave_date ? formatDate(f.leave_date) : '—' }),
        },
        {
          label: t('fgts-valor-rescisorio'),
          value: f.rescission_base != null ? <Money cents={f.rescission_base} /> : '—',
        },
        {
          label: t('fgts-multa'),
          value: f.dismissal_fine != null ? <Money cents={f.dismissal_fine} /> : '—',
        },
        {
          label: t('fgts-ultimo-extrato'),
          value: f.last_statement_date ? formatDate(f.last_statement_date) : '—',
        },
      ]}
    />
  );
}

const ENTRY_COLUMNS: TechColumn<FgtsEntryView>[] = [
  {
    key: 'date',
    get header() {
      return t('fgts-data');
    },
    width: '100px',
    numeric: true,
    cell: (r) => formatDate(r.date),
  },
  {
    key: 'kind',
    get header() {
      return t('fgts-tipo');
    },
    cell: (r) =>
      r.kind === 'withdrawal' && r.withdrawal_reason
        ? `${r.kind_label} · ${reasonLabel(r.withdrawal_reason)}`
        : r.kind_label,
  },
  {
    key: 'detail',
    get header() {
      return t('fgts-detalhe');
    },
    cell: (r) =>
      r.competence
        ? t('fgts-competencia', { mes: r.competence })
        : r.coefficient
          ? t('fgts-coeficiente', { valor: r.coefficient.replace('.', ',') })
          : r.base_year
            ? t('fgts-ano-base', { ano: String(r.base_year) })
            : r.withdrawal_code
              ? t('fgts-codigo', { codigo: r.withdrawal_code })
              : '',
  },
  {
    key: 'amount',
    get header() {
      return t('fgts-valor');
    },
    numeric: true,
    cell: (r) => <Money cents={r.amount} />,
  },
  {
    key: 'balance',
    get header() {
      return t('fgts-saldo');
    },
    numeric: true,
    cell: (r) => <Money cents={r.balance} />,
  },
];

export function FgtsEntries({ account }: { account: AccountView }) {
  const q = useFgtsEntries(account.id);
  const rows = [...(q.data ?? [])].reverse();
  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <FgtsImport account={account} />
      </div>
      <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
        <TechTable
          label={t('fgts-lancamentos')}
          prefsId="Lançamentos do FGTS"
          columns={ENTRY_COLUMNS}
          rows={rows}
          getRowId={(r) => r.txn_id}
          maxHeight="420px"
          dense
          empty={t('fgts-sem-lancamentos')}
        />
      </QueryState>
    </div>
  );
}

/** Importar o extrato da Caixa para esta conta. */
export function FgtsImport({ account }: { account: AccountView }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button
        variant="secondary"
        size="sm"
        icon={<Icon name="arrowDown" size={13} />}
        onClick={() => setOpen(true)}
      >
        {t('fgts-importar')}
      </Button>
      <Dialog
        open={open}
        onOpenChange={setOpen}
        size="xl"
        index="03"
        title={t('fgts-importar-titulo')}
        description={t('fgts-importar-descricao')}
      >
        {open ? <ImportBody account={account} onDone={() => setOpen(false)} /> : null}
      </Dialog>
    </>
  );
}

function ImportBody({ account, onDone }: { account: AccountView; onDone: () => void }) {
  const accounts = (useAccounts().data ?? []).filter(
    (a) => !a.archived && a.kind !== 'fgts' && a.currency === 'BRL',
  );
  const save = useImportFgtsStatement();
  const [file, setFile] = useState<{ filename: string; content_base64: string } | null>(null);
  const [preview, setPreview] = useState<FgtsStatementPreview | null>(null);
  const [reasons, setReasons] = useState<Record<string, string>>({});
  const [dest, setDest] = useState<Record<string, string>>({});
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);

  const read = async (f: File | undefined) => {
    if (!f) return;
    setBusy(true);
    setError(null);
    try {
      const content = { filename: f.name, content_base64: await fileToBase64(f) };
      setFile(content);
      setPreview(await previewFgtsStatement({ files: [content], account_id: account.id }));
    } catch (e) {
      setError(e);
      setPreview(null);
    } finally {
      setBusy(false);
    }
  };

  const confirm = async () => {
    if (!file) return;
    try {
      const r = await save.mutateAsync({
        files: [file],
        account_id: account.id,
        reasons,
        destinations: dest,
      });
      toast({
        tone: 'success',
        title: t('fgts-importado', { n: r.inserted, existentes: r.skipped }),
      });
      onDone();
    } catch (e) {
      setError(e);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <Field label={t('fgts-extrato-pdf')}>
        <FileInput
          accept=".pdf,application/pdf"
          disabled={busy}
          onChange={(e) => void read(e.target.files?.[0])}
        />
      </Field>
      {busy ? <p className="label-caps text-accent">{t('fgts-lendo')}</p> : null}
      <InlineError error={error} />
      {preview ? (
        <>
          <KeyValue
            items={[
              { label: t('fgts-empregador'), value: preview.header.employer_name ?? '—' },
              { label: t('fgts-cnpj'), value: preview.header.employer_cnpj ?? '—' },
              {
                label: t('fgts-saldo-final'),
                value: <Money cents={preview.final_balance} />,
              },
              {
                label: t('fgts-linhas'),
                value: t('fgts-linhas-valor', {
                  novas: preview.new_count,
                  existentes: preview.existing_count,
                }),
              },
              {
                label: t('fgts-jam-conferido'),
                value: t('fgts-jam-conferido-valor', {
                  n: preview.jam_checked,
                  diferentes: preview.jam_mismatches,
                }),
              },
            ]}
          />
          {preview.warnings.length ? (
            <ul className="flex flex-col gap-1 rounded-card border border-warning/50 bg-warning/5 px-3 py-2 text-[12.5px] text-warning">
              {preview.warnings.map((w) => (
                <li key={w}>{w}</li>
              ))}
            </ul>
          ) : (
            <p className="flex items-center gap-2 text-[12.5px] text-positive">
              <Icon name="check" size={13} /> {t('fgts-confere')}
            </p>
          )}
          {preview.withdrawals.length ? (
            <div className="flex flex-col gap-2">
              <span className="label-caps text-dim">{t('fgts-saques')}</span>
              {preview.withdrawals.map((g) => (
                <div key={g.key} className="grid items-end gap-2 sm:grid-cols-[1fr_180px_220px]">
                  <span className="text-[13px]">
                    {formatDate(g.date)} · <Money cents={g.amount} />
                    {g.code ? ` · ${t('fgts-codigo', { codigo: g.code })}` : ''}
                  </span>
                  <Field label={t('fgts-motivo')}>
                    <Select
                      aria-label={t('fgts-motivo')}
                      value={reasons[g.key] ?? g.reason ?? 'outros'}
                      onValueChange={(v) => setReasons({ ...reasons, [g.key]: v })}
                      options={REASONS.map((r) => ({ value: r, label: reasonLabel(r) }))}
                    />
                  </Field>
                  <Field label={t('fgts-destino')}>
                    <Select
                      aria-label={t('fgts-destino')}
                      value={dest[g.key] ?? NONE}
                      onValueChange={(v) => {
                        const next = { ...dest };
                        if (v === NONE) delete next[g.key];
                        else next[g.key] = v;
                        setDest(next);
                      }}
                      options={[
                        { value: NONE, label: t('fgts-sem-destino') },
                        ...accounts.map((a) => ({ value: a.id, label: a.name })),
                      ]}
                    />
                  </Field>
                </div>
              ))}
            </div>
          ) : null}
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={onDone}>
              {t('comum-cancelar')}
            </Button>
            <Button
              disabled={preview.new_count === 0}
              loading={save.isPending}
              onClick={() => void confirm()}
            >
              {t('fgts-gravar', { n: preview.new_count })}
            </Button>
          </div>
        </>
      ) : null}
    </div>
  );
}

const DEPOSIT_STATUS = {
  ok: 'fgts-deposito-ok',
  missing: 'fgts-deposito-faltando',
  short: 'fgts-deposito-menor',
  pending: 'fgts-deposito-prazo',
} as const;

const DEPOSIT_COLUMNS: TechColumn<FgtsDepositCheck>[] = [
  {
    key: 'competence',
    get header() {
      return t('fgts-competencia-col');
    },
    width: '100px',
  },
  {
    key: 'deposited',
    get header() {
      return t('fgts-depositado');
    },
    numeric: true,
    cell: (r) => <Money cents={r.deposited} />,
  },
  {
    key: 'payslip',
    get header() {
      return t('fgts-holerite');
    },
    numeric: true,
    cell: (r) => (r.payslip != null ? <Money cents={r.payslip} /> : '—'),
  },
  {
    key: 'due_date',
    get header() {
      return t('fgts-prazo');
    },
    numeric: true,
    cell: (r) => formatDate(r.due_date),
  },
  {
    key: 'status',
    get header() {
      return t('fgts-situacao');
    },
    cell: (r) => (
      <span
        className={r.status === 'missing' || r.status === 'short' ? 'text-warning' : 'text-dim'}
      >
        {t(DEPOSIT_STATUS[r.status as keyof typeof DEPOSIT_STATUS] ?? 'fgts-deposito-ok')}
      </span>
    ),
  },
];

export function FgtsCheckPanel({ account }: { account: AccountView }) {
  const q = useFgtsCheck(account.id);
  const c = q.data;
  return (
    <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
      {c ? (
        <div className="flex flex-col gap-3">
          <p className="text-[13px]">
            {c.jam_mismatches.length
              ? t('fgts-jam-diferentes', { n: c.jam_mismatches.length, total: c.jam_checked })
              : t('fgts-jam-ok', { n: c.jam_checked })}
          </p>
          {c.jam_mismatches.length ? (
            <ul className="text-[12.5px] text-warning">
              {c.jam_mismatches.map((m) => (
                <li key={m.date}>
                  {formatDate(m.date)}: <Money cents={m.credited} /> · {t('fgts-calculado')}{' '}
                  <Money cents={m.expected} />
                </li>
              ))}
            </ul>
          ) : null}
          <TechTable
            label={t('fgts-depositos')}
            prefsId="Depósitos do FGTS"
            columns={DEPOSIT_COLUMNS}
            rows={[...c.deposits].reverse()}
            getRowId={(r) => r.competence}
            maxHeight="360px"
            dense
          />
          <ul className="text-[12px] text-dim">
            {c.notes.map((n) => (
              <li key={n}>{n}</li>
            ))}
          </ul>
        </div>
      ) : null}
    </QueryState>
  );
}

const BENCH_COLUMNS: TechColumn<FgtsBenchmark>[] = [
  {
    key: 'label',
    get header() {
      return t('fgts-destino-comparado');
    },
  },
  {
    key: 'final_value',
    get header() {
      return t('fgts-teria');
    },
    numeric: true,
    cell: (r) => (r.final_value != null ? <Money cents={r.final_value} /> : '—'),
  },
  {
    key: 'difference',
    get header() {
      return t('fgts-diferenca');
    },
    numeric: true,
    cell: (r) => (r.difference != null ? <Money cents={r.difference} signed /> : '—'),
  },
  {
    key: 'source',
    get header() {
      return t('fgts-fonte');
    },
    cell: (r) => <span className="text-dim">{r.source}</span>,
  },
];

const YEAR_COLUMNS: TechColumn<FgtsYearRow>[] = [
  {
    key: 'year',
    get header() {
      return t('fgts-ano');
    },
    width: '70px',
    cell: (r) => `${r.year}${r.partial ? '*' : ''}`,
  },
  {
    key: 'jam',
    get header() {
      return 'JAM';
    },
    numeric: true,
    cell: (r) => pct(r.jam),
  },
  {
    key: 'jam_distribution',
    get header() {
      return t('fgts-jam-distribuicao');
    },
    numeric: true,
    cell: (r) => pct(r.jam_distribution),
  },
  {
    key: 'ipca',
    get header() {
      return 'IPCA';
    },
    numeric: true,
    cell: (r) => pct(r.ipca),
  },
  {
    key: 'cdi',
    get header() {
      return 'CDI';
    },
    numeric: true,
    cell: (r) => pct(r.cdi),
  },
  {
    key: 'm2',
    get header() {
      return 'M2';
    },
    numeric: true,
    cell: (r) => pct(r.m2),
  },
  {
    key: 'compensation',
    get header() {
      return t('fgts-compensacao');
    },
    numeric: true,
    cell: (r) => (r.compensation != null ? <Money cents={r.compensation} /> : '—'),
  },
];

export function FgtsYield({ account }: { account: AccountView }) {
  const q = useFgtsCompare(account.id);
  const c = q.data;
  return (
    <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
      {c ? (
        <div className="flex flex-col gap-3">
          <KeyValue
            items={[
              {
                label: t('fgts-periodo'),
                value: `${formatDate(c.from)} – ${formatDate(c.to)}`,
              },
              { label: t('fgts-tir'), value: t('fgts-tir-valor', { taxa: pct(c.irr) }) },
              { label: t('fgts-depositos'), value: <Money cents={c.deposits} /> },
              { label: t('fgts-saques'), value: <Money cents={c.withdrawals} /> },
              { label: 'JAM', value: <Money cents={c.jam} /> },
              { label: t('fgts-distribuicao'), value: <Money cents={c.distribution} /> },
            ]}
          />
          <TechTable
            label={t('fgts-comparacao')}
            prefsId="Comparação do FGTS"
            columns={BENCH_COLUMNS}
            rows={c.benchmarks}
            getRowId={(r) => r.id}
            dense
          />
          <TechTable
            label={t('fgts-por-ano')}
            prefsId="FGTS por ano"
            columns={YEAR_COLUMNS}
            rows={[...c.years].reverse()}
            getRowId={(r) => String(r.year)}
            dense
          />
          <ul className="text-[12px] text-dim">
            {c.notes.map((n) => (
              <li key={n}>{n}</li>
            ))}
          </ul>
        </div>
      ) : null}
    </QueryState>
  );
}

/** Saque-aniversário, modalidade e saques (inclusive o uso no financiamento). */
export function FgtsWithdrawals({ account }: { account: AccountView }) {
  const member = account.member_ids.length === 1 ? account.member_ids[0] : null;
  const a = useFgtsAnniversary(member);
  const modality = useFgtsModality();
  const w = useFgtsWithdrawal();
  const housing = useFgtsHousing();
  const accounts = (useAccounts().data ?? []).filter(
    (x) => !x.archived && x.kind !== 'fgts' && x.currency === 'BRL',
  );
  const mortgages = (useLoans().data ?? []).filter((l) => l.kind === 'mortgage');
  const [date, setDate] = useState<string | null>(todayIso());
  const [amount, setAmount] = useState<number | null>(null);
  const [reason, setReason] = useState<string>('rescisao');
  const [code, setCode] = useState('');
  const [destination, setDestination] = useState<string>(NONE);
  const [loan, setLoan] = useState<string>(NONE);
  const [error, setError] = useState<unknown>(null);
  const v = a.data;

  const submit = async () => {
    if (!date || !amount) return;
    setError(null);
    try {
      if (reason === 'moradia' && loan !== NONE) {
        const r = await housing.mutateAsync({
          account_id: account.id,
          loan_id: loan,
          date,
          amount,
        });
        toast({
          tone: r.warnings.length ? 'warning' : 'success',
          title: t('fgts-moradia-feito'),
          description: r.warnings.join(' '),
        });
      } else {
        await w.mutateAsync({
          account_id: account.id,
          date,
          amount,
          reason,
          ...(code.trim() ? { code: code.trim() } : {}),
          ...(destination !== NONE ? { destination_account_id: destination } : {}),
        });
        toast({ tone: 'success', title: t('fgts-saque-feito') });
      }
      setAmount(null);
    } catch (e) {
      setError(e);
    }
  };

  return (
    <div className="flex flex-col gap-5">
      <QueryState loading={a.isLoading} error={a.error}>
        {v ? (
          <div className="flex flex-col gap-2">
            <span className="label-caps text-dim">{t('fgts-aniversario')}</span>
            <KeyValue
              items={[
                {
                  label: t('fgts-aniversario-valor'),
                  value: <Money cents={v.amount} />,
                },
                {
                  label: t('fgts-aniversario-janela'),
                  value:
                    v.window_start && v.window_end
                      ? `${formatDate(v.window_start)} – ${formatDate(v.window_end)}`
                      : t('fgts-sem-nascimento'),
                },
                {
                  label: t('fgts-bloqueado'),
                  value: <Money cents={v.blocked_on_dismissal} />,
                },
                {
                  label: t('fgts-multa-liberada'),
                  value: <Money cents={v.fine_on_dismissal} />,
                },
                {
                  label: t('fgts-volta'),
                  value: v.earliest_return ? formatDate(v.earliest_return) : '—',
                },
              ]}
            />
            {member ? (
              <Field label={t('fgts-modalidade')} hint={t('fgts-modalidade-dica')}>
                <Select
                  aria-label={t('fgts-modalidade')}
                  value={v.modality ?? 'rescisao'}
                  onValueChange={(m) =>
                    void modality.mutateAsync({ member_id: member, modality: m })
                  }
                  options={[
                    { value: 'rescisao', label: t('fgts-modalidade-rescisao') },
                    { value: 'aniversario', label: t('fgts-modalidade-aniversario') },
                  ]}
                />
              </Field>
            ) : null}
            <ul className="text-[12px] text-dim">
              {v.notes.map((n) => (
                <li key={n}>{n}</li>
              ))}
            </ul>
          </div>
        ) : null}
      </QueryState>
      <div className="flex flex-col gap-2">
        <span className="label-caps text-dim">{t('fgts-registrar-saque')}</span>
        <div className="grid gap-3 sm:grid-cols-3">
          <Field label={t('fgts-data')}>
            <DateInput value={date} onValueChange={setDate} />
          </Field>
          <Field label={t('fgts-valor')}>
            <MoneyInput value={amount} onValueChange={setAmount} aria-label={t('fgts-valor')} />
          </Field>
          <Field label={t('fgts-motivo')}>
            <Select
              aria-label={t('fgts-motivo')}
              value={reason}
              onValueChange={setReason}
              options={REASONS.map((r) => ({ value: r, label: reasonLabel(r) }))}
            />
          </Field>
          {reason === 'moradia' ? (
            <Field label={t('fgts-contrato')} hint={t('fgts-contrato-dica')}>
              <Select
                aria-label={t('fgts-contrato')}
                value={loan}
                onValueChange={setLoan}
                options={[
                  { value: NONE, label: t('fgts-sem-contrato') },
                  ...mortgages.map((l) => ({ value: l.id, label: l.name })),
                ]}
              />
            </Field>
          ) : null}
          {reason !== 'moradia' || loan === NONE ? (
            <>
              <Field label={t('fgts-codigo-campo')}>
                <Input value={code} onChange={(e) => setCode(e.target.value)} />
              </Field>
              <Field label={t('fgts-destino')}>
                <Select
                  aria-label={t('fgts-destino')}
                  value={destination}
                  onValueChange={setDestination}
                  options={[
                    { value: NONE, label: t('fgts-sem-destino') },
                    ...accounts.map((x) => ({ value: x.id, label: x.name })),
                  ]}
                />
              </Field>
            </>
          ) : null}
        </div>
        <InlineError error={error} />
        <div>
          <Button
            size="sm"
            disabled={!date || !amount}
            loading={w.isPending || housing.isPending}
            onClick={() => void submit()}
          >
            {t('fgts-registrar')}
          </Button>
        </div>
      </div>
    </div>
  );
}
