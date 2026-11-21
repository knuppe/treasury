/**
 * Cheque especial (limite de crédito) da conta corrente: os campos do contrato no cadastro
 * da conta e o resumo de uso (usado, disponível, saldo + limite e o custo estimado de 30
 * dias com juros e IOF) no cartão e no detalhe da conta.
 */
import type { LoanIndexer } from '@/api/generated/LoanIndexer';
import type { OverdraftStatus } from '@/api/generated/OverdraftStatus';
import type { OverdraftTerms } from '@/api/generated/OverdraftTerms';
import { useCountryFeatures } from '@/api/hooks/country';
import { Field, Input, Money, MoneyInput, Select } from '@/design/components';
import { INDEXER_LABEL } from '@/features/debts/labels';
import { localDecimal } from '@/features/fx/decimal';
import { Switch } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

export interface OverdraftDraft {
  enabled: boolean;
  limit: number | null;
  monthlyRate: string;
  discount: string;
  annualRate: string;
  iofDaily: string;
  iofAdditional: string;
  cetMonth: string;
  cetYear: string;
  /** Taxa pós-fixada ("4,75 + TR"): o índice; com ele, as taxas são o spread. */
  indexer: LoanIndexer;
}

/** Índices aceitos no limite (os mesmos das dívidas). */
const OVERDRAFT_INDEXERS: readonly LoanIndexer[] = [
  'none',
  'tr',
  'cdi',
  'selic',
  'ipca',
  'igpm',
  'inpc',
  'poupanca',
];

const pct = (v: string | null | undefined) => localDecimal(v ?? '');

export function overdraftDraft(terms: OverdraftTerms | null | undefined): OverdraftDraft {
  return {
    enabled: Boolean(terms),
    limit: terms?.limit ?? null,
    monthlyRate: pct(terms?.monthly_rate),
    discount: pct(terms?.discount_pct),
    annualRate: pct(terms?.annual_rate),
    // Padrões legais quando não informados: IOF básico 0,0082% ao dia e adicional 0,38%.
    iofDaily: terms ? pct(terms.iof_daily_rate) : pct('0.0082'),
    iofAdditional: terms ? pct(terms.iof_additional_rate) : pct('0.38'),
    cetMonth: pct(terms?.cet_month),
    cetYear: pct(terms?.cet_year),
    indexer: terms?.indexer ?? 'none',
  };
}

const opt = (v: string) => (v.trim() ? v.trim() : null);

/** Os termos para gravar. Fora do Brasil não há IOF nem CET (nem os padrões da lei daqui). */
export function overdraftTerms(d: OverdraftDraft, brazil = true): OverdraftTerms {
  if (!brazil) d = { ...d, iofDaily: '', iofAdditional: '', cetMonth: '', cetYear: '' };
  return {
    limit: d.limit ?? 0,
    monthly_rate: opt(d.monthlyRate),
    discount_pct: opt(d.discount),
    annual_rate: opt(d.annualRate),
    iof_daily_rate: opt(d.iofDaily),
    iof_additional_rate: opt(d.iofAdditional),
    cet_month: opt(d.cetMonth),
    cet_year: opt(d.cetYear),
    // "4,75 + TR" escrito na taxa também vale: o núcleo separa o spread do índice.
    ...(d.indexer !== 'none' ? { indexer: d.indexer } : {}),
  };
}

function Pct({
  label,
  value,
  onChange,
  hint,
  withIndex = false,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  hint?: string;
  /** Aceita também o índice na mesma linha ("4,75 + TR"). */
  withIndex?: boolean;
}) {
  return (
    <Field label={label} hint={hint}>
      <Input
        numeric
        inputMode={withIndex ? 'text' : 'decimal'}
        value={value}
        placeholder={withIndex ? t('contas-taxa-indexada-exemplo') : localDecimal('0.00')}
        onChange={(e) =>
          onChange(
            withIndex
              ? e.target.value.replace(/[^\d,.%+ A-Za-zçÇ-]/g, '')
              : e.target.value.replace(/[^\d,.]/g, ''),
          )
        }
      />
    </Field>
  );
}

export function OverdraftFields({
  draft,
  currency,
  onChange,
}: {
  draft: OverdraftDraft;
  currency: string;
  onChange: (p: Partial<OverdraftDraft>) => void;
}) {
  // IOF, CET e os índices (TR, CDI, Selic, poupança…) são do Brasil (docs/international.md).
  const brazil = useCountryFeatures().br_credit_rates;
  return (
    <fieldset className="flex flex-col gap-3 rounded-card border border-line p-3">
      <legend className="label-caps px-1">{t('contas-cheque-especial')}</legend>
      <Switch
        checked={draft.enabled}
        onChange={(enabled) => onChange({ enabled })}
        label={t('contas-tem-cheque-especial')}
        description={t('contas-cheque-especial-dica')}
      />
      {draft.enabled ? (
        <div className="grid gap-3 sm:grid-cols-4">
          <Field label={t('contas-limite-moeda', { moeda: currency })}>
            <MoneyInput
              currency={currency}
              value={draft.limit}
              onValueChange={(limit) => onChange({ limit })}
            />
          </Field>
          <Pct
            label={t('contas-juros-mes')}
            value={draft.monthlyRate}
            onChange={(monthlyRate) => onChange({ monthlyRate })}
          />
          <Pct
            label={t('contas-desconto')}
            value={draft.discount}
            onChange={(discount) => onChange({ discount })}
            hint={t('contas-desconto-dica')}
          />
          <Pct
            label={draft.indexer === 'none' ? t('contas-juros-ano') : t('contas-juros-ano-spread')}
            value={draft.annualRate}
            onChange={(annualRate) => onChange({ annualRate })}
            hint={t('contas-juros-ano-dica')}
            withIndex
          />
          {brazil ? (
            <>
              <Field label={t('contas-indexador')} hint={t('contas-indexador-dica')}>
                <Select
                  aria-label={t('contas-indexador')}
                  value={draft.indexer}
                  onValueChange={(v) => v && onChange({ indexer: v as LoanIndexer })}
                  options={OVERDRAFT_INDEXERS.map((k) => ({ value: k, label: INDEXER_LABEL[k] }))}
                />
              </Field>
              <Pct
                label={t('contas-iof-basico')}
                value={draft.iofDaily}
                onChange={(iofDaily) => onChange({ iofDaily })}
              />
              <Pct
                label={t('contas-iof-adicional')}
                value={draft.iofAdditional}
                onChange={(iofAdditional) => onChange({ iofAdditional })}
              />
              <Pct
                label={t('contas-cet-mes')}
                value={draft.cetMonth}
                onChange={(cetMonth) => onChange({ cetMonth })}
              />
              <Pct
                label={t('contas-cet-ano')}
                value={draft.cetYear}
                onChange={(cetYear) => onChange({ cetYear })}
              />
            </>
          ) : null}
        </div>
      ) : null}
    </fieldset>
  );
}

/** Resumo do uso do limite (cartão e detalhe da conta). */
export function OverdraftSummary({
  status,
  currency,
}: {
  status: OverdraftStatus;
  currency: string;
}) {
  const share = status.limit > 0 ? Math.min(1, status.used / status.limit) : 0;
  const cost = (status.estimated_interest ?? 0) + (status.estimated_iof ?? 0);
  return (
    <div className="flex flex-col gap-1.5" aria-label={t('contas-cheque-especial')}>
      <div className="flex items-baseline justify-between gap-2 text-[12px]">
        <span className="text-dim">{t('contas-cheque-especial')}</span>
        <span className="num text-fg">
          {withSlots(t('contas-cheque-uso', { usado: slot(0), limite: slot(1) }), [
            <Money cents={status.used} currency={currency} />,
            <Money cents={status.limit} currency={currency} />,
          ])}
        </span>
      </div>
      <div className="h-1.5 overflow-hidden rounded-full bg-surface-2" aria-hidden>
        <div
          className={
            share > 0.8 ? 'h-full bg-negative' : share > 0 ? 'h-full bg-warning' : 'h-full'
          }
          style={{ width: `${share * 100}%` }}
        />
      </div>
      <p className="text-[11.5px] text-dim">
        {withSlots(t('contas-cheque-disponivel', { valor: slot(0) }), [
          <Money cents={status.spendable} currency={currency} />,
        ])}
        {status.index ? (
          <>
            {' '}
            {t(
              status.index.estimated
                ? 'contas-cheque-taxa-indexada-estimada'
                : 'contas-cheque-taxa-indexada',
              {
                taxa: localDecimal(status.effective_monthly_rate ?? ''),
                indice: INDEXER_LABEL[status.index.indexer],
                valor: localDecimal(status.index.monthly_pct),
                data: formatDate(status.index.date),
              },
            )}
          </>
        ) : null}
        {status.used > 0 && (status.estimated_interest != null || status.estimated_iof != null) ? (
          <>
            {' '}
            {withSlots(
              status.effective_monthly_rate
                ? t('contas-cheque-custo-juros', {
                    valor: slot(0),
                    taxa: localDecimal(status.effective_monthly_rate),
                  })
                : t('contas-cheque-custo', { valor: slot(0) }),
              [<Money cents={cost} currency={currency} tone="negative" />],
            )}
          </>
        ) : null}
        {status.uncertain && status.used > 0 ? (
          <span className="block text-warning">{t('contas-cheque-incerto')}</span>
        ) : null}
      </p>
    </div>
  );
}
