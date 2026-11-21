/**
 * Simular demissão (só Brasil): para uma renda CLT, o que a pessoa receberia ao sair do
 * emprego pelo motivo escolhido: as verbas da rescisão, INSS e IRRF, o FGTS (multa e saque),
 * o seguro-desemprego e quantos meses do salário líquido isso cobre. Estimativa; nada é
 * gravado (`income.simulate_termination`).
 */
import { useState } from 'react';

import type { IncomeSource } from '@/api/generated/IncomeSource';
import type { NoticeMode } from '@/api/generated/NoticeMode';
import type { TerminationInput } from '@/api/generated/TerminationInput';
import type { TerminationReason } from '@/api/generated/TerminationReason';
import type { TerminationResult } from '@/api/generated/TerminationResult';
import { useCore } from '@/api/hooks/coreQuery';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Input,
  Money,
  MoneyInput,
  Select,
} from '@/design/components';
import { cx } from '@/design/cx';
import { ErrorBlock, LoadingBlock } from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDate, formatLocale } from '@/lib/format';

const REASONS: TerminationReason[] = [
  'sem_justa_causa',
  'pedido_demissao',
  'acordo',
  'justa_causa',
  'fim_experiencia',
  'quebra_experiencia_empregador',
  'quebra_experiencia_empregado',
];

/** Opções de aviso para cada motivo (vazio = o motivo não tem aviso). */
const NOTICES: Partial<Record<TerminationReason, NoticeMode[]>> = {
  sem_justa_causa: ['indenizado', 'trabalhado'],
  acordo: ['indenizado', 'trabalhado'],
  pedido_demissao: ['trabalhado', 'nao_cumprido'],
};

const today = () => new Date().toISOString().slice(0, 10);

function Row({
  label,
  detail,
  cents,
  strong,
}: {
  label: string;
  detail?: string;
  cents: number;
  strong?: boolean;
}) {
  return (
    <div className={cx('flex items-baseline gap-3 py-1 text-[13px]', strong && 'font-semibold')}>
      <span className="min-w-0 flex-1 text-fg">
        {label}
        {detail ? <span className="ml-2 text-[11.5px] text-dim">{detail}</span> : null}
      </span>
      <Money cents={cents} />
    </div>
  );
}

function Result({ r }: { r: TerminationResult }) {
  const u = r.unemployment;
  const f = r.fgts;
  return (
    <div className="flex flex-col gap-4">
      <section aria-label={t('familia-rescisao-verbas')} className="flex flex-col">
        <h3 className="label-caps text-[11px] text-dim">{t('familia-rescisao-verbas')}</h3>
        {r.lines.map((l) => (
          <Row key={l.code} label={l.label} detail={l.detail || undefined} cents={l.amount} />
        ))}
        <div className="mt-1 border-t border-line pt-1">
          <Row label={t('familia-rescisao-liquido')} cents={r.net} strong />
        </div>
      </section>
      <section aria-label={t('familia-rescisao-fgts')} className="flex flex-col">
        <h3 className="label-caps text-[11px] text-dim">{t('familia-rescisao-fgts')}</h3>
        {f.balance != null ? (
          <Row label={t('familia-rescisao-fgts-saldo')} cents={f.balance} />
        ) : null}
        <Row label={t('familia-rescisao-fgts-deposito')} cents={f.deposit} />
        <Row label={t('familia-rescisao-fgts-multa', { pct: f.fine_rate })} cents={f.fine} />
        <Row label={t('familia-rescisao-fgts-sacavel')} cents={f.withdrawable} strong />
      </section>
      <section aria-label={t('familia-rescisao-seguro')} className="flex flex-col gap-1">
        <h3 className="label-caps text-[11px] text-dim">{t('familia-rescisao-seguro')}</h3>
        {u.eligible ? (
          <>
            <Row
              label={t('familia-rescisao-seguro-parcelas', { n: u.installments })}
              cents={u.installment}
            />
            <Row label={t('familia-rescisao-seguro-total')} cents={u.total} strong />
          </>
        ) : (
          <p className="text-[12.5px] text-dim">{u.reason}</p>
        )}
        <p className="text-[11px] text-dim">{u.table}</p>
      </section>
      <section
        aria-label={t('familia-rescisao-total')}
        className="rounded-control border border-accent/50 bg-accent-soft/40 px-3 py-2"
      >
        <Row label={t('familia-rescisao-total')} cents={r.total_available} strong />
        {r.months_covered != null ? (
          <p className="text-[12.5px] text-fg">
            {t('familia-rescisao-meses', {
              meses: r.months_covered.toLocaleString(formatLocale(), { maximumFractionDigits: 1 }),
            })}
          </p>
        ) : null}
      </section>
      <ul className="flex list-disc flex-col gap-1 pl-4 text-[11.5px] text-dim">
        {r.warnings.map((w) => (
          <li key={w}>{w}</li>
        ))}
        <li>
          {t('familia-rescisao-tabelas', {
            fim: formatDate(r.projected_end),
            base: r.payroll_rules,
          })}
        </li>
      </ul>
    </div>
  );
}

export function TerminationDialog({
  source,
  onClose,
}: {
  source: IncomeSource;
  onClose: () => void;
}) {
  const [reason, setReason] = useState<TerminationReason>('sem_justa_causa');
  const [date, setDate] = useState<string | null>(today());
  const [notice, setNotice] = useState<NoticeMode | null>(null);
  const [admission, setAdmission] = useState<string | null>(source.start_date);
  const [overdue, setOverdue] = useState('');
  const [fgts, setFgts] = useState<number | null>(null);
  const [contractEnd, setContractEnd] = useState<string | null>(null);
  const [claims, setClaims] = useState('0');
  const options = NOTICES[reason];
  const mode = options ? (notice && options.includes(notice) ? notice : options[0]!) : undefined;
  const breaks =
    reason === 'quebra_experiencia_empregador' || reason === 'quebra_experiencia_empregado';
  const ready = !!date && !!admission && (!breaks || !!contractEnd);
  const args: TerminationInput = {
    source_id: source.id,
    termination_date: date ?? '',
    reason,
    ...(admission ? { admission_date: admission } : {}),
    ...(mode ? { notice: mode } : {}),
    ...(overdue ? { overdue_vacation_days: Number(overdue) } : {}),
    ...(fgts != null ? { fgts_balance: fgts } : {}),
    ...(breaks && contractEnd ? { contract_end: contractEnd } : {}),
    previous_claims: Number(claims),
  };
  const q = useCore<TerminationResult>('income', 'simulate_termination', args, {
    enabled: ready,
    retry: false,
  });
  return (
    <Dialog
      open
      onOpenChange={(o) => (o ? null : onClose())}
      size="lg"
      title={t('familia-rescisao-titulo', { renda: source.name })}
      description={t('familia-rescisao-descricao')}
      footer={
        <Button variant="primary" onClick={onClose}>
          {t('comum-fechar')}
        </Button>
      }
    >
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <Field label={t('familia-rescisao-motivo')}>
          <Select
            value={reason}
            onValueChange={(v) => setReason(v as TerminationReason)}
            options={REASONS.map((r) => ({
              value: r,
              label: t('familia-rescisao-motivo-op', { motivo: r }),
            }))}
          />
        </Field>
        <Field label={t('familia-rescisao-data')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        {options ? (
          <Field label={t('familia-rescisao-aviso')}>
            <Select
              value={mode}
              onValueChange={(v) => setNotice(v as NoticeMode)}
              options={options.map((n) => ({
                value: n,
                label: t('familia-rescisao-aviso-op', { aviso: n }),
              }))}
            />
          </Field>
        ) : null}
        {breaks ? (
          <Field label={t('familia-rescisao-fim-contrato')}>
            <DateInput value={contractEnd} onValueChange={setContractEnd} />
          </Field>
        ) : null}
        <Field label={t('familia-rescisao-admissao')}>
          <DateInput value={admission} onValueChange={setAdmission} />
        </Field>
        <Field
          label={t('familia-rescisao-ferias-vencidas')}
          hint={t('familia-rescisao-ferias-dica')}
        >
          <Input
            inputMode="numeric"
            value={overdue}
            onChange={(e) => setOverdue(e.target.value.replace(/\D/g, '').slice(0, 2))}
          />
        </Field>
        <Field
          label={t('familia-rescisao-saldo-fgts')}
          hint={t('familia-rescisao-saldo-fgts-dica')}
        >
          <MoneyInput value={fgts} onValueChange={setFgts} />
        </Field>
        <Field label={t('familia-rescisao-pedidos')}>
          <Select
            value={claims}
            onValueChange={setClaims}
            options={['0', '1', '2'].map((n) => ({
              value: n,
              label: t('familia-rescisao-pedidos-op', { n: Number(n) }),
            }))}
          />
        </Field>
      </div>
      {!ready ? (
        <p className="text-[12.5px] text-dim">{t('familia-rescisao-preencha')}</p>
      ) : q.isPending ? (
        <LoadingBlock />
      ) : q.error ? (
        <ErrorBlock error={q.error} />
      ) : q.data ? (
        <Result r={q.data} />
      ) : null}
    </Dialog>
  );
}
