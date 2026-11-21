/**
 * Prévia da declaração de ajuste anual (`tax.annual_draft`): rendimentos dos holerites,
 * deduções achadas nas categorias, PGBL e o IRRF "dedo-duro" que sobrou, com os dois modelos
 * (completo e simplificado), o mais vantajoso e o saldo a pagar ou a restituir. Os valores
 * podem ser ajustados com os informes; o recálculo é puro (`tax.annual_compute`) e não grava.
 */
import { useState } from 'react';

import type { AnnualInput } from '@/api/generated/AnnualInput';
import type { AnnualModel } from '@/api/generated/AnnualModel';
import { useAnnualCompute, useAnnualDraft } from '@/api/hooks/tax';
import { Button, InstrumentCard, Money, MoneyInput } from '@/design/components';
import { Notice } from '@/features/portfolio/components/States';
import { QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatPercent } from '@/lib/format';

/** Campos que a família pode ajustar (valores do ano, em centavos). */
type Field =
  | 'income'
  | 'social_security'
  | 'withheld'
  | 'self_employed'
  | 'carne_leao'
  | 'medical'
  | 'education'
  | 'private_pension'
  | 'alimony'
  | 'complementary_tax'
  | 'irrf_lei_11033';

const FIELDS: readonly Field[] = [
  'income',
  'social_security',
  'withheld',
  'self_employed',
  'carne_leao',
  'medical',
  'education',
  'private_pension',
  'alimony',
  'complementary_tax',
  'irrf_lei_11033',
];

const LABEL: Record<Field, () => string> = {
  income: () => t('ir-anual-campo-income'),
  social_security: () => t('ir-anual-campo-social_security'),
  withheld: () => t('ir-anual-campo-withheld'),
  self_employed: () => t('ir-anual-campo-self_employed'),
  carne_leao: () => t('ir-anual-campo-carne_leao'),
  medical: () => t('ir-anual-campo-medical'),
  education: () => t('ir-anual-campo-education'),
  private_pension: () => t('ir-anual-campo-private_pension'),
  alimony: () => t('ir-anual-campo-alimony'),
  complementary_tax: () => t('ir-anual-campo-complementary_tax'),
  irrf_lei_11033: () => t('ir-anual-campo-irrf_lei_11033'),
};

const sum = (v: readonly number[]) => v.reduce((a, b) => a + b, 0);

function valueOf(input: AnnualInput, f: Field): number {
  switch (f) {
    case 'income':
      return sum(input.employment.map((e) => e.income));
    case 'social_security':
      return sum(input.employment.map((e) => e.social_security));
    case 'withheld':
      return sum(input.employment.map((e) => e.withheld));
    case 'self_employed':
      return input.self_employed.income;
    case 'carne_leao':
      return input.self_employed.carne_leao;
    case 'education':
      return sum(input.education);
    default:
      return input[f];
  }
}

/** A entrada do cofre com os ajustes da tela (os somados viram uma linha só). */
function withEdits(base: AnnualInput, edits: Partial<Record<Field, number>>, deps: number | null) {
  const v = (f: Field) => edits[f] ?? valueOf(base, f);
  const touched = (fs: Field[]) => fs.some((f) => edits[f] != null);
  return {
    ...base,
    employment: touched(['income', 'social_security', 'withheld'])
      ? [{ income: v('income'), social_security: v('social_security'), withheld: v('withheld') }]
      : base.employment,
    self_employed: {
      ...base.self_employed,
      income: v('self_employed'),
      carne_leao: v('carne_leao'),
    },
    education: edits.education != null ? [edits.education] : base.education,
    medical: v('medical'),
    private_pension: v('private_pension'),
    alimony: v('alimony'),
    complementary_tax: v('complementary_tax'),
    irrf_lei_11033: v('irrf_lei_11033'),
    dependents: deps ?? base.dependents,
  } satisfies AnnualInput;
}

function ModelColumn({ title, m, best }: { title: string; m: AnnualModel; best: boolean }) {
  const row = (label: string, cents: number, strong = false) => (
    <div className="flex items-baseline justify-between gap-3 py-1 text-[13px]">
      <span className="text-dim">{label}</span>
      <Money cents={cents} className={strong ? 'font-medium' : undefined} />
    </div>
  );
  return (
    <div
      className={
        best ? 'rounded-card border border-accent p-3.5' : 'rounded-card border border-line p-3.5'
      }
    >
      <div className="mb-1 flex items-center justify-between gap-2">
        <span className="font-medium">{title}</span>
        {best ? (
          <span className="label-caps text-accent">{t('ir-anual-mais-vantajoso')}</span>
        ) : null}
      </div>
      {row(t('ir-anual-deducoes'), m.deductions)}
      {row(t('ir-anual-base'), m.base)}
      {row(t('ir-anual-imposto-tabela'), m.table_tax)}
      {m.reduction > 0 ? row(t('ir-anual-reducao'), -m.reduction) : null}
      {m.donations > 0 ? row(t('ir-anual-doacoes'), -m.donations) : null}
      {row(t('ir-anual-devido'), m.tax_due, true)}
      {row(t('ir-anual-pago'), m.tax_paid)}
      {m.to_pay > 0
        ? row(t('ir-anual-a-pagar'), m.to_pay, true)
        : row(t('ir-anual-a-restituir'), m.refund, true)}
    </div>
  );
}

export function AnnualReturn({ year, memberId }: { year: number; memberId: string | null }) {
  const draft = useAnnualDraft({ year, ...(memberId ? { member_id: memberId } : {}) });
  const [edits, setEdits] = useState<Partial<Record<Field, number>>>({});
  const [deps, setDeps] = useState<number | null>(null);
  const [key, setKey] = useState(`${year}-${memberId ?? ''}`);
  if (key !== `${year}-${memberId ?? ''}`) {
    setKey(`${year}-${memberId ?? ''}`);
    setEdits({});
    setDeps(null);
  }
  const base = draft.data?.input ?? null;
  const edited = Object.keys(edits).length > 0 || deps != null;
  const input = base && edited ? withEdits(base, edits, deps) : null;
  const computed = useAnnualCompute(input);
  const result = (edited ? computed.data : null) ?? draft.data?.result ?? null;
  return (
    <InstrumentCard index="09.8" title={t('ir-anual-titulo', { ano: String(year) })}>
      <QueryState loading={draft.isPending} error={draft.error} label={t('ir-anual-carregando')}>
        {base && result ? (
          <div className="flex flex-col gap-4">
            <p className="text-[12.5px] text-dim">{t('ir-anual-dica')}</p>
            <div className="grid gap-x-4 gap-y-2 @min-[530px]:grid-cols-2 @min-[860px]:grid-cols-3">
              {FIELDS.map((f) => (
                <label key={f} className="flex flex-col gap-1 text-[12.5px]">
                  <span className="text-dim">{LABEL[f]()}</span>
                  <MoneyInput
                    aria-label={LABEL[f]()}
                    value={edits[f] ?? valueOf(base, f)}
                    onValueChange={(v) => setEdits((e) => ({ ...e, [f]: v ?? 0 }))}
                    currency="BRL"
                  />
                </label>
              ))}
              <label className="flex flex-col gap-1 text-[12.5px]">
                <span className="text-dim">{t('ir-anual-campo-dependentes')}</span>
                <input
                  type="number"
                  aria-label={t('ir-anual-campo-dependentes')}
                  min={0}
                  max={20}
                  className="h-9 rounded-control border border-line bg-surface px-2.5 text-[13px] text-fg"
                  value={deps ?? base.dependents}
                  onChange={(e) => setDeps(Math.max(0, Math.min(20, Number(e.target.value) || 0)))}
                />
              </label>
            </div>
            <div className="grid gap-3 @min-[530px]:grid-cols-2">
              <ModelColumn
                title={t('ir-anual-completo')}
                m={result.full}
                best={result.best === 'completo'}
              />
              <ModelColumn
                title={t('ir-anual-simplificado')}
                m={result.simplified}
                best={result.best === 'simplificado'}
              />
            </div>
            <p className="text-[12.5px] text-dim">
              {t('ir-anual-aliquota-efetiva', {
                aliquota: formatPercent(result.effective_rate_bp / 10_000, { decimals: 2 }),
              })}
            </p>
            {draft.data?.irpfm?.applies ? (
              <div className="rounded-card border border-line p-3.5">
                <div className="mb-1 font-medium">{t('ir-irpfm-titulo')}</div>
                {(
                  [
                    [t('ir-irpfm-rendimentos'), draft.data.irpfm.total_income],
                    [
                      t('ir-irpfm-bruto', {
                        aliquota: formatPercent(draft.data.irpfm.rate_bp / 10_000, { decimals: 2 }),
                      }),
                      draft.data.irpfm.gross,
                    ],
                    [t('ir-irpfm-creditos'), -draft.data.irpfm.credits],
                    [t('ir-irpfm-devido'), draft.data.irpfm.due],
                  ] as const
                ).map(([label, cents]) => (
                  <div
                    key={label}
                    className="flex items-baseline justify-between gap-3 py-1 text-[13px]"
                  >
                    <span className="text-dim">{label}</span>
                    <Money cents={cents} />
                  </div>
                ))}
              </div>
            ) : null}
            {edited ? (
              <div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    setEdits({});
                    setDeps(null);
                  }}
                >
                  {t('ir-anual-restaurar')}
                </Button>
              </div>
            ) : null}
            <Notice title={t('ir-anual-de-onde')}>
              <ul className="flex list-disc flex-col gap-0.5 pl-4">
                {(draft.data?.notes ?? []).map((n) => (
                  <li key={n}>{n}</li>
                ))}
              </ul>
            </Notice>
          </div>
        ) : null}
      </QueryState>
    </InstrumentCard>
  );
}
