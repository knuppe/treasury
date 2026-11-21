/**
 * Primeira tela do app (docs/i18n-design.md), antes dos termos: escolher o idioma desta
 * instalação, já marcado com o palpite pelo sistema. Marcar uma opção mostra a tela naquele
 * idioma; "Continuar" grava no núcleo (`config.toml`) e segue para os termos.
 */
import { useState, type ReactNode } from 'react';

import { Brand } from '@/app/Brand';
import { Button } from '@/design/components';

import { applyLocale, setLocale, t, useI18n, useLocale } from './index';
import { LanguagePicker } from './LanguagePicker';

export function LanguageGate({ children }: { children: ReactNode }) {
  const chosen = useI18n((s) => s.chosen);
  return chosen ? children : <FirstLanguage />;
}

function FirstLanguage() {
  const locale = useLocale();
  const [saving, setSaving] = useState(false);
  return (
    <div className="flex min-h-screen items-start justify-center overflow-y-auto bg-canvas px-6 py-10">
      <main
        aria-labelledby="idioma-titulo"
        className="flex w-full max-w-[440px] flex-col gap-4 rounded-card border border-panel bg-surface px-7 py-6"
      >
        <Brand />
        <h1 id="idioma-titulo" className="text-[22px] font-medium text-fg">
          {t('casca-idioma-titulo')}
        </h1>
        <p className="text-[13.5px] text-dim">{t('casca-idioma-texto')}</p>
        <LanguagePicker
          label={t('casca-idioma')}
          value={locale}
          onChange={(code) => void applyLocale(code)}
        />
        <div className="flex justify-end">
          <Button
            variant="primary"
            loading={saving}
            onClick={() => {
              setSaving(true);
              setLocale(locale).catch(() => setSaving(false));
            }}
          >
            {t('comum-continuar')}
          </Button>
        </div>
      </main>
    </div>
  );
}
