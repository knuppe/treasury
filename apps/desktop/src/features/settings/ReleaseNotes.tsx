/**
 * The notes of a new version, with a way to read them in the interface language (the notes
 * are written in English): on the device when the system can (macOS 26 or later, language
 * downloaded), otherwise in the browser translator, opened only by the person's click.
 */
import { useEffect, useState } from 'react';

import { isTauri } from '@/api/client';
import { openExternal } from '@/app/externalLinks';
import { Button, toast } from '@/design/components';
import { t, useLocale } from '@/i18n';
import {
  canTranslateOnDevice,
  nativeTarget,
  translateOnDevice,
  webTranslateUrl,
} from '@/lib/translateNotes';

export function ReleaseNotes({ notes }: { notes: string }) {
  const locale = useLocale();
  const target = nativeTarget(locale);
  // null while asking the system; false sends the button to the browser translator.
  const [device, setDevice] = useState<boolean | null>(null);
  const [translated, setTranslated] = useState<{ notes: string; text: string } | null>(null);
  const [showing, setShowing] = useState(false);
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    if (!target) return;
    let alive = true;
    void canTranslateOnDevice(target).then((ok) => alive && setDevice(ok));
    return () => {
      alive = false;
    };
  }, [target]);
  // New notes (another version) drop the old translation.
  const current = translated?.notes === notes ? translated.text : null;
  const onDevice = device === true && !failed;

  const translate = () => {
    if (!target) return;
    if (current) return setShowing(true);
    setBusy(true);
    translateOnDevice(notes, target)
      .then((text) => {
        setTranslated({ notes, text });
        setShowing(true);
      })
      .catch(() => setFailed(true))
      .finally(() => setBusy(false));
  };

  const openWeb = () => {
    const web = webTranslateUrl(notes, locale);
    if (!web) return;
    const opened = isTauri()
      ? openExternal(web.url)
      : Promise.resolve(void window.open(web.url, '_blank', 'noopener'));
    opened
      .then(() => {
        if (web.clipped) toast({ tone: 'info', title: t('ajustes-atualizacao-traduzir-cortado') });
      })
      .catch((err: unknown) =>
        toast({
          tone: 'error',
          title: t('casca-navegador-falhou'),
          description: err instanceof Error ? err.message : String(err),
        }),
      );
  };

  const shown = showing && current ? current : notes;
  return (
    <div className="flex flex-col gap-1.5">
      <p className="leading-snug whitespace-pre-line" lang={showing && current ? locale : 'en'}>
        {shown}
      </p>
      {target && device !== null ? (
        <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-[12px] text-dim">
          {showing && current ? (
            <>
              <span>{t('ajustes-atualizacao-traduzido')}</span>
              <Button size="sm" variant="ghost" onClick={() => setShowing(false)}>
                {t('ajustes-atualizacao-ver-original')}
              </Button>
            </>
          ) : onDevice ? (
            <Button size="sm" variant="ghost" loading={busy} onClick={translate}>
              {t('ajustes-atualizacao-traduzir')}
            </Button>
          ) : (
            <>
              {failed ? <span>{t('ajustes-atualizacao-traduzir-falhou')}</span> : null}
              <Button size="sm" variant="ghost" onClick={openWeb}>
                {t('ajustes-atualizacao-traduzir-google')}
              </Button>
              <span>{t('ajustes-atualizacao-traduzir-google-dica')}</span>
            </>
          )}
        </div>
      ) : null}
    </div>
  );
}
