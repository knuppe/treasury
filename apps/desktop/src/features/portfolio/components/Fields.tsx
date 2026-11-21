import type { ReactNode } from 'react';

import type { Asset } from '@/api/generated/Asset';
import type { AccountView } from '@/api/generated/AccountView';
import { Select, type SelectOption } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { assetLabel, CLASS_LABEL } from '../labels';

/** Caixa de seleção nativa com rótulo e ajuda (teclado e leitores de tela de graça). */
export function Check({
  checked,
  onChange,
  label,
  hint,
  disabled,
  className,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: ReactNode;
  hint?: ReactNode;
  disabled?: boolean;
  className?: string;
}) {
  return (
    <label
      className={cx(
        'flex items-start gap-2.5 text-[13px] text-fg',
        disabled && 'opacity-50',
        className,
      )}
    >
      <input
        type="checkbox"
        className="mt-0.5 size-3.5"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className="flex flex-col gap-0.5">
        <span>{label}</span>
        {hint ? <span className="text-[12px] text-dim">{hint}</span> : null}
      </span>
    </label>
  );
}

export function assetOptions(assets: readonly Asset[]): SelectOption[] {
  return assets.map((a) => ({
    value: a.id,
    label: a.ticker ? `${a.ticker} · ${a.name}` : a.name,
    hint: CLASS_LABEL[a.class],
  }));
}

/** Seleção de ativo (digitar o ticker pula para ele). */
export function AssetSelect({
  assets,
  value,
  onChange,
  disabled,
  placeholder = t('carteira-escolha-ativo'),
}: {
  assets: readonly Asset[];
  value: string | undefined;
  onChange: (id: string) => void;
  disabled?: boolean;
  placeholder?: string;
}) {
  return (
    <Select
      value={value}
      // O Radix às vezes avisa '' ao trocar de não controlado para controlado.
      onValueChange={(v) => v && onChange(v)}
      options={assetOptions(assets)}
      placeholder={placeholder}
      disabled={disabled}
    />
  );
}

export function AccountSelect({
  accounts,
  value,
  onChange,
  disabled,
}: {
  accounts: readonly AccountView[];
  value: string | undefined;
  onChange: (id: string) => void;
  disabled?: boolean;
}) {
  return (
    <Select
      value={value}
      onValueChange={(v) => v && onChange(v)}
      disabled={disabled}
      placeholder={accounts.length ? t('carteira-escolha-conta') : t('carteira-nenhuma-corretora')}
      options={accounts.map((a) => ({ value: a.id, label: a.name, hint: a.currency }))}
    />
  );
}

export { assetLabel };
