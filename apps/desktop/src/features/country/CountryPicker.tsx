/**
 * Escolha do país onde a família mora (docs/countries.md), na criação do cofre e em
 * Ajustes › Família: Brasil, Estados Unidos e Portugal primeiro, depois o catálogo.
 */
import type { CountryChoice } from '@/api/generated/CountryChoice';
import type { CountrySupport } from '@/api/generated/CountrySupport';
import { Select } from '@/design/components';
import { t } from '@/i18n';

/** O que o app faz no país, em uma frase. */
export function supportText(support: CountrySupport): string {
  switch (support) {
    case 'full':
      return t('cofre-pais-suporte-completo');
    case 'adapted':
      return t('cofre-pais-suporte-adaptado');
    case 'generic':
      return t('cofre-pais-suporte-generico');
  }
}

export function CountryPicker({
  choices,
  value,
  onChange,
  id,
  disabled,
}: {
  choices: readonly CountryChoice[];
  value: string;
  onChange: (code: string) => void;
  id?: string;
  disabled?: boolean;
}) {
  return (
    <div className="flex flex-col gap-1">
      <Select
        {...(id ? { id } : {})}
        aria-label={t('cofre-pais-rotulo')}
        value={value}
        disabled={disabled}
        onValueChange={(v) => v && onChange(v)}
        options={choices.map((c) => ({
          value: c.code,
          label: `${c.name} · ${c.currency}${c.support === 'generic' ? '' : ' ★'}`,
        }))}
      />
      <span className="text-[11.5px] text-dim">{t('cofre-pais-legenda')}</span>
    </div>
  );
}
