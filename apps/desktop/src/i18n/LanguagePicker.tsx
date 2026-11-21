/**
 * Seletor de idioma (docs/i18n-design.md): uma lista suspensa (combobox) que cresce com os
 * idiomas novos, digitar busca pelo nome. Cada idioma vem escrito na própria língua, com a
 * bandeira do país da variante (pt-BR: Brasil, pt-PT: Portugal; en: Reino Unido, es: Espanha,
 * onde a língua nasceu). As bandeiras são SVG embutidos (flag-icons, MIT, `flags/LICENSE`),
 * porque o Windows não desenha bandeiras em emoji. Usado na primeira tela e em Ajustes ›
 * Preferências; o pseudo-idioma só aparece no modo desenvolvedor.
 */
import { useEffect } from 'react';

import { Select } from '@/design/components';

import br from './flags/br.svg';
import es from './flags/es.svg';
import gb from './flags/gb.svg';
import pt from './flags/pt.svg';
import { LOCALES, t, type AppLocale } from './index';

const NAME = {
  'pt-BR': 'comum-idioma-pt-BR',
  en: 'comum-idioma-en',
  es: 'comum-idioma-es',
  'pt-PT': 'comum-idioma-pt-PT',
  xx: 'casca-idioma-pseudo',
} as const satisfies Record<AppLocale, string>;

/** Bandeira de cada idioma (4:3); o pseudo-idioma não tem país. */
const FLAG: Record<AppLocale, string | null> = {
  'pt-BR': br,
  en: gb,
  es,
  'pt-PT': pt,
  xx: null,
};

const PICKER_ID = 'seletor-idioma';

/** Trocar o idioma remonta o app; o seletor recebe o foco de volta. */
let refocus = false;

export function LanguageFlag({ code }: { code: AppLocale }) {
  const src = FLAG[code];
  return src ? (
    <img
      src={src}
      alt=""
      aria-hidden
      width={20}
      height={15}
      className="h-[15px] w-5 shrink-0 rounded-[2px] object-cover shadow-[0_0_0_1px_var(--line)]"
    />
  ) : (
    <span
      aria-hidden
      className="inline-block h-[15px] w-5 shrink-0 rounded-[2px] border border-dashed border-line-strong"
    />
  );
}

export function LanguagePicker({
  value,
  onChange,
  developer = false,
  label,
}: {
  value: AppLocale;
  onChange: (code: AppLocale) => void;
  developer?: boolean;
  label: string;
}) {
  useEffect(() => {
    if (refocus) {
      refocus = false;
      document.getElementById(PICKER_ID)?.focus();
    }
  }, []);
  const options: readonly AppLocale[] = developer ? [...LOCALES, 'xx'] : LOCALES;
  return (
    <Select
      id={PICKER_ID}
      aria-label={label}
      value={value}
      onValueChange={(code) => {
        if (code === value) return;
        refocus = true;
        onChange(code as AppLocale);
      }}
      options={options.map((code) => ({
        value: code,
        label: (
          <span className="flex items-center gap-2.5" lang={code === 'xx' ? undefined : code}>
            <LanguageFlag code={code} />
            {t(NAME[code])}
          </span>
        ),
      }))}
    />
  );
}
