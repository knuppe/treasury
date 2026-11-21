import { useMemo } from 'react';

import { useTags } from '@/api/hooks/transactions';
import { ChipsInput } from '@/design/components';
import { t } from '@/i18n';

/** Tags por nome: Enter ou vírgula adiciona, Backspace no campo vazio tira a última. */
export function TagInput({
  value,
  onChange,
  label = t('lancamentos-campo-tags'),
}: {
  value: string[];
  onChange: (tags: string[]) => void;
  label?: string;
}) {
  const { data: known = [] } = useTags();
  const suggestions = useMemo(() => known.map((k) => k.name), [known]);
  return (
    <ChipsInput
      value={value}
      onChange={onChange}
      label={label}
      removeLabel={(tag) => t('lancamentos-tirar-tag', { tag })}
      suggestions={suggestions}
      placeholder={t('lancamentos-tags-exemplo')}
      prefix="#"
    />
  );
}
