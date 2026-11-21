import type { CategoryKind } from '@/api/generated/CategoryKind';
import type { Member } from '@/api/generated/Member';
import { Button, Icon, IconButton, Money, MoneyInput, Select } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { CategoryPicker } from './ui/CategoryPicker';
import { slot, withSlots } from './ui/slots';

export interface SplitPart {
  key: string;
  category_id: string | null;
  member_id: string | null;
  /** Valor positivo (o sinal vem do tipo do lançamento). */
  amount: number | null;
  notes: string;
}

let counter = 0;
export function newPart(partial: Partial<SplitPart> = {}): SplitPart {
  counter += 1;
  return {
    key: `p${counter}`,
    category_id: null,
    member_id: null,
    amount: null,
    notes: '',
    ...partial,
  };
}

const NO_MEMBER = '__none__';

/** Soma das partes e quanto falta (positivo) ou sobra (negativo) em relação ao total. */
export function splitBalance(total: number | null, parts: readonly SplitPart[]) {
  const sum = parts.reduce((s, p) => s + (p.amount ?? 0), 0);
  return { sum, missing: (total ?? 0) - sum };
}

/**
 * Divisão de um lançamento em categorias e membros, com a soma conferida
 * contra o total enquanto se digita.
 */
export function SplitEditor({
  parts,
  onChange,
  total,
  currency,
  kind,
  members,
}: {
  parts: SplitPart[];
  onChange: (parts: SplitPart[]) => void;
  total: number | null;
  currency: string;
  kind: CategoryKind;
  members: readonly Member[];
}) {
  const { sum, missing } = splitBalance(total, parts);
  const update = (key: string, patch: Partial<SplitPart>) =>
    onChange(parts.map((p) => (p.key === key ? { ...p, ...patch } : p)));
  const balanced = missing === 0 && parts.length >= 2;

  return (
    <fieldset className="flex flex-col gap-2 rounded-card border border-line p-3">
      <legend className="label-caps px-1">{t('lancamentos-divisao')}</legend>
      <div className="grid grid-cols-[minmax(0,1fr)_120px_140px_28px] gap-2 text-[11px]">
        <span className="label-caps">{t('lancamentos-campo-categoria')}</span>
        <span className="label-caps">{t('lancamentos-campo-membro')}</span>
        <span className="label-caps text-right">{t('lancamentos-campo-valor')}</span>
        <span />
      </div>
      {parts.map((p, i) => (
        <div
          key={p.key}
          className="grid grid-cols-[minmax(0,1fr)_120px_140px_28px] items-center gap-2"
        >
          <CategoryPicker
            kind={kind}
            value={p.category_id}
            onChange={(category_id) => update(p.key, { category_id })}
            label={t('lancamentos-categoria-parte', { n: i + 1 })}
            className="min-w-0 [&>button]:w-full"
          />
          <Select
            aria-label={t('lancamentos-membro-parte', { n: i + 1 })}
            value={p.member_id ?? NO_MEMBER}
            onValueChange={(v) => update(p.key, { member_id: v === NO_MEMBER ? null : v })}
            options={[
              { value: NO_MEMBER, label: '—' },
              ...members.map((m) => ({ value: m.id, label: m.name })),
            ]}
          />
          <MoneyInput
            aria-label={t('lancamentos-valor-parte', { n: i + 1 })}
            currency={currency}
            value={p.amount}
            onValueChange={(amount) => update(p.key, { amount })}
          />
          <IconButton
            size="sm"
            label={t('lancamentos-tirar-parte', { n: i + 1 })}
            icon={<Icon name="x" size={12} />}
            tooltip={false}
            disabled={parts.length <= 2}
            onClick={() => onChange(parts.filter((x) => x.key !== p.key))}
          />
        </div>
      ))}
      <div className="flex flex-wrap items-center gap-2 pt-1">
        <Button
          size="sm"
          variant="ghost"
          icon={<Icon name="plus" size={12} />}
          onClick={() => onChange([...parts, newPart()])}
        >
          {t('lancamentos-adicionar-parte')}
        </Button>
        {missing > 0 && parts.length > 0 ? (
          <Button
            size="sm"
            variant="ghost"
            onClick={() => {
              const target =
                [...parts].reverse().find((p) => !p.amount) ?? parts[parts.length - 1]!;
              update(target.key, { amount: (target.amount ?? 0) + missing });
            }}
          >
            {t('lancamentos-restante-ultima')}
          </Button>
        ) : null}
        <p
          role="status"
          aria-live="polite"
          className={cx('ml-auto text-[12.5px]', balanced ? 'text-positive' : 'text-warning')}
        >
          {withSlots(t('lancamentos-divisao-soma', { soma: slot(0), total: slot(1) }), [
            <Money key="soma" cents={sum} currency={currency} />,
            <Money key="total" cents={total ?? 0} currency={currency} />,
          ])}
          {' · '}
          {missing > 0
            ? withSlots(t('lancamentos-divisao-faltam', { valor: slot(0) }), [
                <Money key="faltam" cents={missing} currency={currency} />,
              ])
            : missing < 0
              ? withSlots(t('lancamentos-divisao-sobram', { valor: slot(0) }), [
                  <Money key="sobram" cents={-missing} currency={currency} />,
                ])
              : parts.length < 2
                ? t('lancamentos-divisao-minimo')
                : t('lancamentos-divisao-confere')}
        </p>
      </div>
    </fieldset>
  );
}
