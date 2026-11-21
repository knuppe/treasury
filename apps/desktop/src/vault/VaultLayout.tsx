import type { ReactNode } from 'react';

import { Brand } from '@/app/Brand';
import { ThemeMenu } from '@/app/ThemeMenu';
import { KeyValue } from '@/design/components';
import { t } from '@/i18n';

export interface VaultLayoutProps {
  /** Índice e título da coluna de especificação (`00 · Cofre`). */
  kicker: string;
  heading: ReactNode;
  lead: ReactNode;
  dbPath?: string;
  children: ReactNode;
  /** Abaixo do formulário: os outros cofres (docs/vaults.md). */
  footer?: ReactNode;
}

/**
 * Moldura das telas do cofre (criar, desbloquear, recuperar): à esquerda a
 * ficha técnica do cofre; à direita o formulário num card de instrumento.
 */
export function VaultLayout({ kicker, heading, lead, dbPath, children, footer }: VaultLayoutProps) {
  return (
    <div className="flex h-screen flex-col overflow-y-auto bg-canvas">
      <header className="flex h-12 items-center justify-between px-6" data-tauri-drag-region>
        <Brand />
        <ThemeMenu />
      </header>

      <main className="flex flex-1 items-center justify-center px-6 py-10">
        <div className="grid w-full max-w-[1040px] items-start gap-x-16 gap-y-10 lg:grid-cols-[minmax(0,1fr)_minmax(0,500px)]">
          <section className="flex flex-col gap-6 pt-2 lg:pt-10">
            <div className="flex flex-col gap-3">
              <span className="label-caps text-accent">{kicker}</span>
              <h1 className="text-[30px] leading-[1.15] font-semibold tracking-[-0.015em] text-fg">
                {heading}
              </h1>
              <p className="max-w-[46ch] text-[14.5px] leading-relaxed text-dim">{lead}</p>
            </div>
            <div className="max-w-[440px] border-t border-line pt-5">
              <KeyValue
                layout="stacked"
                items={[
                  { label: t('cofre-ficha-cifra'), value: 'SQLCipher · AES-256' },
                  { label: t('cofre-ficha-chave'), value: t('cofre-ficha-chave-valor') },
                  {
                    label: t('cofre-ficha-arquivo'),
                    value: dbPath ?? '~/.treasury/data/treasury.db',
                  },
                  { label: t('cofre-ficha-rede'), value: t('cofre-ficha-rede-valor') },
                ]}
              />
            </div>
            <p className="label-caps hidden lg:block">{t('cofre-lema')}</p>
          </section>
          <div className="flex min-w-0 flex-col gap-4">
            {children}
            {footer}
          </div>
        </div>
      </main>
    </div>
  );
}
