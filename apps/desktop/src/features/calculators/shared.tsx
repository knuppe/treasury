/**
 * Peças comuns das calculadoras livres: campos de valor e de porcentagem (estado em unidades de
 * moeda e em %), as taxas de referência oficiais com a data e o botão de usar cada uma.
 */
import { useQuery } from '@tanstack/react-query';
import type { ReactNode } from 'react';

import { call } from '@/api/client';
import type { RateReference } from '@/api/generated/RateReference';
import type { RateReferences } from '@/api/generated/RateReferences';
import { Field, Money, MoneyInput } from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatMonthShort, formatPercent, minorUnits } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';

import { DecimalInput } from '../portfolio/components/DecimalInput';

export function useRateReferences() {
  return useQuery({
    queryKey: ['indices', 'references'] as const,
    queryFn: () => call<RateReferences>('indices.references'),
  });
}

/** Unidades da moeda ↔ menor unidade (centavos), pela moeda base. */
export function useMinor() {
  const currency = useBaseCurrency();
  const f = 10 ** minorUnits(currency);
  return {
    currency,
    toMinor: (units: number) => Math.round(units * f),
    toUnits: (minor: number) => minor / f,
  };
}

/** Valor em dinheiro, a partir de unidades da moeda. */
export function Amount({
  value,
  tone,
  signed,
}: {
  value: number;
  tone?: 'auto';
  signed?: boolean;
}) {
  const { toMinor } = useMinor();
  if (!Number.isFinite(value)) return <span className="num">—</span>;
  return (
    <Money cents={toMinor(value)} {...(tone ? { tone } : {})} {...(signed ? { signed } : {})} />
  );
}

export function MoneyField({
  label,
  value,
  onChange,
  hint,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  hint?: ReactNode;
}) {
  const { toMinor, toUnits } = useMinor();
  return (
    <Field label={label} hint={hint}>
      <MoneyInput
        value={toMinor(value)}
        onValueChange={(c) => onChange(c == null ? 0 : toUnits(c))}
      />
    </Field>
  );
}

export function NumberField({
  label,
  value,
  onChange,
  hint,
  aside,
  maxDecimals = 2,
}: {
  label: string;
  /** Número em unidades de exibição (porcentagem como 10,5; anos como 30). */
  value: number;
  onChange: (v: number) => void;
  hint?: ReactNode;
  aside?: ReactNode;
  maxDecimals?: number;
}) {
  return (
    <Field label={label} hint={hint} aside={aside}>
      <DecimalInput
        value={Number.isFinite(value) ? String(value) : null}
        maxDecimals={maxDecimals}
        onValueChange={(v) => onChange(v == null ? 0 : Number(v))}
      />
    </Field>
  );
}

/** "14,90% a.a." */
export const pctLabel = (rate: number, decimals = 2) => formatPercent(rate, { decimals });

type RefKind = 'policy' | 'cdi' | 'inflation' | 'savings';

/** A inflação é do mês (o último oficial); as demais, do dia. */
function refDate(kind: RefKind, r: RateReference): string {
  return kind === 'inflation' ? formatMonthShort(r.date.slice(0, 7)) : formatDate(r.date);
}

/** Nome da referência para os textos. */
export function refName(kind: RefKind, r: RateReference): string {
  switch (kind) {
    case 'policy':
      return t('calc-ref-juro', { nome: r.name });
    case 'cdi':
      return t('calc-ref-cdi');
    case 'inflation':
      return t('calc-ref-inflacao', { nome: r.name });
    case 'savings':
      return t('calc-ref-poupanca');
  }
}

/** Botões "Usar CDI anualizado: 14,90% (30/10/2026)" que preenchem a taxa do campo. */
export function UseReference({
  refs,
  onUse,
}: {
  refs: readonly { kind: RefKind; ref: RateReference | null }[];
  /** Taxa em % (10,5). */
  onUse: (percent: number) => void;
}) {
  const shown = refs.filter((r): r is { kind: typeof r.kind; ref: RateReference } => r.ref != null);
  if (shown.length === 0) return null;
  return (
    <span className="flex flex-wrap gap-1.5">
      {shown.map(({ kind, ref }) => (
        <button
          key={kind}
          type="button"
          className="chip !h-6 !px-2 !text-[11.5px]"
          aria-pressed={false}
          onClick={() => onUse(Math.round(ref.rate * 10000) / 100)}
          title={t('calc-ref-usar', {
            nome: refName(kind, ref),
            taxa: pctLabel(ref.rate),
            data: refDate(kind, ref),
          })}
        >
          {t('calc-ref-usar-curto', { nome: refName(kind, ref) })}
        </button>
      ))}
    </span>
  );
}

/** Quadro das referências oficiais, com a data de cada uma. */
export function ReferencesPanel({ refs }: { refs: RateReferences | undefined }) {
  const rows = refs
    ? (
        [
          ['policy', refs.policy_rate],
          ['cdi', refs.cdi],
          ['inflation', refs.inflation_12m],
          ['savings', refs.savings],
        ] as const
      ).filter(([, r]) => r != null)
    : [];
  return (
    <section aria-label={t('calc-ref-titulo')} className="flex flex-col gap-1.5">
      <h3 className="label-caps">{t('calc-ref-titulo')}</h3>
      {rows.length === 0 ? (
        <p className="text-[12.5px] text-dim">{t('calc-ref-nenhuma')}</p>
      ) : (
        <ul className="flex flex-wrap gap-x-5 gap-y-1 text-[12.5px]">
          {rows.map(([kind, r]) => (
            <li key={kind} className="flex gap-1.5">
              <span className="text-dim">{refName(kind, r!)}</span>
              <span className="num text-fg">
                {t('calc-ref-de', { taxa: pctLabel(r!.rate), data: refDate(kind, r!) })}
              </span>
              {r!.estimated ? <span className="text-dim">({t('calc-ref-estimado')})</span> : null}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
