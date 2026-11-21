/**
 * Ajustes › Sobre e apoio › Termos de uso (docs/terms.md): a versão vigente, quando esta
 * instalação aceitou, o texto completo (na língua do país da família) e o histórico.
 */
import { useState } from 'react';

import { useTermsStatus } from '@/api/hooks/terms';
import { openExternal } from '@/app/externalLinks';
import { Button, Dialog, InstrumentCard, Markdown } from '@/design/components';
import { t, useLocale } from '@/i18n';
import { formatDateTime } from '@/lib/format';

import { AGPL_URL, fullTerms } from './texts';

export function TermsCard() {
  const s = useTermsStatus().data;
  const locale = useLocale();
  const [open, setOpen] = useState(false);
  return (
    <InstrumentCard index="12.S7" title={t('termos-de-uso')}>
      <div className="flex flex-col gap-3 text-[13px] text-fg">
        <p>
          {s?.accepted
            ? t('termos-versao-aceita', {
                versao: s.current,
                data: formatDateTime(s.accepted.accepted_at),
                aceita: s.accepted.version,
              })
            : t('termos-versao', { versao: s?.current ?? '…' })}
        </p>
        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="secondary" onClick={() => setOpen(true)}>
            {t('termos-ler-completos')}
          </Button>
          <Button size="sm" variant="ghost" onClick={() => void openExternal(AGPL_URL)}>
            {t('termos-ler-licenca')}
          </Button>
        </div>
        {s?.history.length ? (
          <div>
            <h3 className="label-caps mb-1 text-dim">{t('termos-historico')}</h3>
            <ul className="flex flex-col gap-0.5 text-[12.5px] text-dim">
              {s.history.map((h) => (
                <li key={h.version}>
                  {h.version}: {h.summary}
                </li>
              ))}
            </ul>
          </div>
        ) : null}
      </div>
      <Dialog open={open} onOpenChange={setOpen} size="lg" resizable title={t('termos-de-uso')}>
        <Markdown text={fullTerms(locale)} />
      </Dialog>
    </InstrumentCard>
  );
}
