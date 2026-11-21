/**
 * INSS do planejador de aposentadoria: o valor informado ou o calculado pelas regras da
 * Reforma da Previdência (`inss_calc`). Com o cálculo, mostra cada regra (quando é cumprida,
 * quanto paga e quanto paga na idade do plano) e a escolhida.
 */
import type { InssEstimate } from '@/api/generated/InssEstimate';
import type { InssInput } from '@/api/generated/InssInput';
import type { Member } from '@/api/generated/Member';
import { DateInput, Field, Input, Money, MoneyInput, Select } from '@/design/components';
import { cx } from '@/design/cx';
import { DecimalInput } from '@/features/portfolio/components/DecimalInput';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

const num = (s: string | null) => {
  const n = s == null ? NaN : Number(s);
  return Number.isFinite(n) ? n : undefined;
};
const age = (s: string) => {
  const n = Number(s);
  return s.trim() && Number.isInteger(n) ? n : undefined;
};

export const DEFAULT_INSS: InssInput = { sex: 'male', contribution_years: 0, average_salary: 0 };

/** O cálculo começa com o sexo e a data de nascimento do membro (continuam editáveis). */
export function inssDefaults(member: Member | null | undefined): InssInput {
  const p = member?.profile;
  return {
    ...DEFAULT_INSS,
    sex: p?.sex === 'female' ? 'female' : 'male',
    ...(p?.birth_date ? { birth_date: p.birth_date } : {}),
  };
}

export function RetirementInss({
  value,
  onChange,
  estimate,
  retirementAge,
}: {
  value: InssInput;
  onChange: (v: InssInput) => void;
  estimate: InssEstimate | null | undefined;
  retirementAge: number | undefined;
}) {
  const set = (p: Partial<InssInput>) => onChange({ ...value, ...p });
  return (
    <div className="flex flex-col gap-3">
      <div className="grid gap-3 sm:grid-cols-4">
        <Field label={t('metas-inss-sexo')}>
          <Select
            aria-label={t('metas-inss-sexo')}
            value={value.sex}
            onValueChange={(v) => set({ sex: v as InssInput['sex'] })}
            options={[
              { value: 'male', label: t('metas-inss-homem') },
              { value: 'female', label: t('metas-inss-mulher') },
            ]}
          />
        </Field>
        <Field label={t('metas-inss-nascimento')} hint={t('metas-inss-nascimento-dica')}>
          <DateInput
            value={value.birth_date ?? null}
            onValueChange={(v) => set({ birth_date: v ?? undefined })}
          />
        </Field>
        <Field label={t('metas-inss-contribuicao')}>
          <DecimalInput
            maxDecimals={2}
            value={String(value.contribution_years)}
            onValueChange={(v) => set({ contribution_years: num(v) ?? 0 })}
          />
        </Field>
        <Field
          label={t('metas-inss-contribuicao-reforma')}
          hint={t('metas-inss-contribuicao-reforma-dica')}
        >
          <DecimalInput
            maxDecimals={2}
            value={
              value.contribution_at_reform == null ? null : String(value.contribution_at_reform)
            }
            onValueChange={(v) => set({ contribution_at_reform: num(v) })}
          />
        </Field>
        <Field label={t('metas-inss-media')} hint={t('metas-inss-media-dica')}>
          <MoneyInput
            value={value.average_salary}
            onValueChange={(v) => set({ average_salary: v ?? 0 })}
          />
        </Field>
        <Field label={t('metas-inss-contribui-ate')}>
          <Input
            inputMode="numeric"
            placeholder={String(retirementAge ?? 65)}
            value={value.contribute_until_age ?? ''}
            onChange={(e) => set({ contribute_until_age: age(e.target.value) })}
          />
        </Field>
      </div>
      {estimate ? <PensionRulesTable estimate={estimate} /> : null}
    </div>
  );
}

const SYSTEM_CURRENCY: Record<string, string> = { br_inss: 'BRL', us_ssa: 'USD', pt_ss: 'EUR' };
const SYSTEM_NAME: Record<string, string> = {
  br_inss: 'INSS',
  us_ssa: 'Social Security',
  pt_ss: 'Segurança Social',
};

/** Regras do benefício público: quando cada uma é cumprida, quanto paga e a escolhida. */
export function PensionRulesTable({ estimate }: { estimate: InssEstimate }) {
  const currency = SYSTEM_CURRENCY[estimate.system] ?? 'BRL';
  return (
    <div className="flex flex-col gap-1.5">
      <table
        className="w-full border-collapse text-[12.5px]"
        aria-label={t('metas-inss-regras', { sistema: SYSTEM_NAME[estimate.system] ?? 'INSS' })}
      >
        <thead>
          <tr className="label-caps text-left text-dim">
            <th className="pb-1 font-normal">{t('metas-inss-regra')}</th>
            <th className="pb-1 font-normal">{t('metas-inss-cumpre-em')}</th>
            <th className="pb-1 text-right font-normal">{t('metas-inss-beneficio-data')}</th>
            <th className="pb-1 text-right font-normal">{t('metas-inss-na-idade')}</th>
          </tr>
        </thead>
        <tbody>
          {estimate.rules.map((r) => {
            const chosen = r.id === estimate.chosen;
            return (
              <tr
                key={r.id}
                className={cx('border-t border-line', chosen && 'text-accent')}
                aria-current={chosen ? 'true' : undefined}
              >
                <td className="py-1" title={r.requirement}>
                  {r.name}
                  {chosen ? <span className="label-caps ml-2">{t('metas-inss-usada')}</span> : null}
                </td>
                <td className="py-1">
                  {r.eligible_date
                    ? t('metas-inss-cumpre-data', {
                        data: formatDate(r.eligible_date),
                        idade: Math.floor(r.eligible_age ?? 0),
                      })
                    : t('metas-inss-nao-cumpre')}
                </td>
                <td className="py-1 text-right">
                  {r.monthly_at_eligibility != null ? (
                    <Money cents={r.monthly_at_eligibility} currency={currency} />
                  ) : r.eligible_date ? (
                    t('metas-inss-fator')
                  ) : (
                    '—'
                  )}
                </td>
                <td className="py-1 text-right">
                  {r.monthly_at_start != null && r.start_age != null
                    ? withSlots(t('metas-inss-aos', { valor: slot(0), idade: r.start_age }), [
                        <Money key="valor" cents={r.monthly_at_start} currency={currency} />,
                      ])
                    : '—'}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <ul className="text-[12px] text-dim">
        {estimate.notes.map((n) => (
          <li key={n}>{n}</li>
        ))}
      </ul>
    </div>
  );
}
