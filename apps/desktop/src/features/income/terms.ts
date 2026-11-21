/**
 * Como o país da família chama as coisas da renda (docs/countries.md): os tipos de renda, os
 * descontos da folha, o documento do salário e os pagamentos extras do ano. Os nomes do Brasil
 * vêm do catálogo (`familia.ftl`, no idioma do app); os dos outros países vêm do núcleo, também
 * no idioma do app (`nucleo-pais.ftl`).
 */
import type { IncomeSourceKind } from '@/api/generated/IncomeSourceKind';
import { useCountryInfo } from '@/api/hooks/country';
import { t } from '@/i18n';

/** Tipos de renda do Brasil, no idioma do app. */
function brKinds(): Record<IncomeSourceKind, string> {
  return {
    clt: t('familia-tipo-clt'),
    public_servant: t('familia-tipo-servidor'),
    retirement: t('familia-tipo-aposentadoria'),
    pension: t('familia-tipo-pensao'),
    pj: t('familia-tipo-pj'),
    self_employed: t('familia-tipo-autonomo'),
    informal: t('familia-tipo-informal'),
    rental: t('familia-tipo-aluguel'),
    benefit: t('familia-tipo-beneficio'),
    other: t('familia-tipo-outra'),
  };
}

export interface IncomeTerms {
  country: string;
  kind: Record<IncomeSourceKind, string>;
  /** Contribuição do empregado (INSS, Segurança Social, FICA). */
  contribution: string;
  /** Imposto retido na fonte (IRRF, IRS, federal). */
  tax: string;
  /** Os dois juntos, para títulos de coluna. */
  both: string;
  /** O documento do salário, no singular e no plural. */
  payslip: string;
  payslips: string;
  /** Registro da empresa pagadora (CNPJ, NIF, EIN; `null`: o app não lê). */
  employerId: string | null;
  /** 13º salário (Brasil) ou subsídio de Natal (Portugal); `null` onde não há. */
  christmas: string | null;
  /** Mês das férias: o que entra (terço de férias, subsídio de férias); `null` onde não há. */
  vacation: string | null;
  /** O app calcula os descontos pelas regras do país. */
  payroll: boolean;
  /** Dependente no imposto. */
  dependentHint: string;
}

export function useIncomeTerms(): IncomeTerms {
  const info = useCountryInfo();
  const code = info?.code ?? 'BR';
  // Brasil (ou o país ainda carregando): os nomes saem do catálogo, no idioma do app.
  const p = code === 'BR' ? null : info?.payroll;
  const kind = { ...brKinds(), ...(p?.kinds ?? {}) } as Record<IncomeSourceKind, string>;
  const payslip = p?.payslip ?? t('familia-holerite');
  const contribution = p?.contribution ?? info?.payroll.contribution ?? 'INSS';
  const tax = p?.income_tax ?? info?.payroll.income_tax ?? 'IRRF';
  return {
    country: code,
    kind,
    contribution,
    tax,
    both:
      code === 'US'
        ? t('familia-descontos-eua')
        : t('familia-descontos', { contribuicao: contribution, imposto: tax }),
    payslip,
    payslips: p ? p.payslips : t('familia-holerites'),
    employerId: info ? info.payroll.employer_id : 'CNPJ',
    christmas:
      code === 'BR' ? t('familia-13-salario') : code === 'PT' ? t('familia-subsidio-natal') : null,
    vacation:
      code === 'BR'
        ? t('familia-terco-ferias')
        : code === 'PT'
          ? t('familia-subsidio-ferias')
          : null,
    payroll: info?.features.payroll ?? true,
    dependentHint: code === 'BR' ? t('familia-dependente-dica-br') : t('familia-dependente-dica'),
  };
}

/** Primeira letra maiúscula ("holerites" → "Holerites"). */
export const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
