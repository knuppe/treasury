/**
 * Aposentadoria pública fora do Brasil no planejador: a Social Security dos EUA
 * (`us_ssa_calc`: PIA do extrato ou AIME, créditos) e a pensão de velhice da Segurança Social
 * de Portugal (`pt_pension_calc`: carreira e remuneração de referência). As regras e o valor
 * vêm do núcleo, na mesma tabela do INSS.
 */
import type { InssEstimate } from '@/api/generated/InssEstimate';
import type { Member } from '@/api/generated/Member';
import type { PtPensionInput } from '@/api/generated/PtPensionInput';
import type { UsSsaInput } from '@/api/generated/UsSsaInput';
import { DateInput, Field, Input, MoneyInput, Select } from '@/design/components';
import { DecimalInput } from '@/features/portfolio/components/DecimalInput';
import { t } from '@/i18n';

import { PensionRulesTable } from './RetirementInss';

const num = (s: string | null) => {
  const n = s == null ? NaN : Number(s);
  return Number.isFinite(n) ? n : undefined;
};
const int = (s: string) => {
  const n = Number(s);
  return s.trim() && Number.isInteger(n) && n >= 0 ? n : undefined;
};
const birth = (member: Member | null | undefined) =>
  member?.profile?.birth_date ? { birth_date: member.profile.birth_date } : {};

export const usSsaDefaults = (member: Member | null | undefined): UsSsaInput => ({
  ...birth(member),
});

export const ptPensionDefaults = (member: Member | null | undefined): PtPensionInput => ({
  contribution_years: 0,
  reference_salary: 0,
  ...birth(member),
});

function BirthField({ value, onChange }: { value?: string; onChange: (v?: string) => void }) {
  return (
    <Field label={t('metas-inss-nascimento')} hint={t('metas-inss-nascimento-dica')}>
      <DateInput value={value ?? null} onValueChange={(v) => onChange(v ?? undefined)} />
    </Field>
  );
}

export function RetirementUsSsa({
  value,
  onChange,
  estimate,
  retirementAge,
}: {
  value: UsSsaInput;
  onChange: (v: UsSsaInput) => void;
  estimate: InssEstimate | null | undefined;
  retirementAge: number | undefined;
}) {
  const set = (p: Partial<UsSsaInput>) => onChange({ ...value, ...p });
  return (
    <div className="flex flex-col gap-3">
      <div className="grid gap-3 sm:grid-cols-4">
        <BirthField value={value.birth_date} onChange={(v) => set({ birth_date: v })} />
        <Field label={t('metas-ssa-pia')} hint={t('metas-ssa-pia-dica')}>
          <MoneyInput
            currency="USD"
            value={value.pia ?? null}
            onValueChange={(v) => set({ pia: v ?? undefined })}
          />
        </Field>
        <Field label={t('metas-ssa-aime')} hint={t('metas-ssa-aime-dica')}>
          <MoneyInput
            currency="USD"
            value={value.aime ?? null}
            onValueChange={(v) => set({ aime: v ?? undefined })}
          />
        </Field>
        <Field label={t('metas-ssa-creditos')} hint={t('metas-ssa-creditos-dica')}>
          <Input
            inputMode="numeric"
            placeholder="40"
            value={value.credits ?? ''}
            onChange={(e) => set({ credits: int(e.target.value) })}
          />
        </Field>
        <Field label={t('metas-ssa-trabalha-ate')}>
          <Input
            inputMode="numeric"
            placeholder={String(retirementAge ?? 65)}
            value={value.work_until_age ?? ''}
            onChange={(e) => set({ work_until_age: int(e.target.value) })}
          />
        </Field>
      </div>
      {estimate ? <PensionRulesTable estimate={estimate} /> : null}
    </div>
  );
}

export function RetirementPtPension({
  value,
  onChange,
  estimate,
  retirementAge,
}: {
  value: PtPensionInput;
  onChange: (v: PtPensionInput) => void;
  estimate: InssEstimate | null | undefined;
  retirementAge: number | undefined;
}) {
  const set = (p: Partial<PtPensionInput>) => onChange({ ...value, ...p });
  return (
    <div className="flex flex-col gap-3">
      <div className="grid gap-3 sm:grid-cols-4">
        <BirthField value={value.birth_date} onChange={(v) => set({ birth_date: v })} />
        <Field label={t('metas-pt-carreira')}>
          <DecimalInput
            maxDecimals={2}
            value={String(value.contribution_years)}
            onValueChange={(v) => set({ contribution_years: num(v) ?? 0 })}
          />
        </Field>
        <Field label={t('metas-pt-rr')} hint={t('metas-pt-rr-dica')}>
          <MoneyInput
            currency="EUR"
            value={value.reference_salary}
            onValueChange={(v) => set({ reference_salary: v ?? 0 })}
          />
        </Field>
        <Field label={t('metas-inss-contribui-ate')}>
          <Input
            inputMode="numeric"
            placeholder={String(retirementAge ?? 66)}
            value={value.contribute_until_age ?? ''}
            onChange={(e) => set({ contribute_until_age: int(e.target.value) })}
          />
        </Field>
        <Field label={t('metas-pt-antes-17')}>
          <Select
            aria-label={t('metas-pt-antes-17')}
            value={value.started_before_17 ? 'yes' : 'no'}
            onValueChange={(v) => set({ started_before_17: v === 'yes' || undefined })}
            options={[
              { value: 'no', label: t('comum-nao') },
              { value: 'yes', label: t('comum-sim') },
            ]}
          />
        </Field>
      </div>
      {estimate ? <PensionRulesTable estimate={estimate} /> : null}
    </div>
  );
}
