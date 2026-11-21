/**
 * Aviso da pasta Downloads: arquivos que acabaram de chegar e que o Treasury importa
 * (ajuste `watch_downloads`). Um cartão só, sem bloquear a tela; "Importar" abre a janela
 * única de importação com os arquivos já carregados, "Agora não" dispensa até fechar o app.
 */
import { useEffect, useState } from 'react';

import { useAppSettings, useSaveAppSettings } from '@/api/hooks/settings';
import { Button, Icon } from '@/design/components';
import { Checkbox, InlineError } from '@/features/transactions/ui/controls';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { t } from '@/i18n';

import { listenDownloadOffers, readOfferedFile, useDownloadOffers } from './downloadsWatch';
import { useInboxNotices } from './inboxNotices';
import { useImportHub } from './importHub';

export function DownloadOffers() {
  const offers = useDownloadOffers((s) => s.offers);
  const settle = useDownloadOffers((s) => s.settle);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const { data: settings } = useAppSettings();
  const save = useSaveAppSettings();

  useEffect(() => listenDownloadOffers(), []);
  useInboxNotices();

  if (!offers.length) return null;
  const many = offers.length > 1;

  const importAll = async () => {
    setBusy(true);
    setError(null);
    try {
      const list = [...offers];
      const files = await Promise.all(list.map(readOfferedFile));
      settle(list.map((o) => o.path));
      useImportHub
        .getState()
        .openWith(files, Object.fromEntries(list.map((o) => [o.name, o.path])));
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  };

  return (
    <section
      role="region"
      aria-label={t('importacao-downloads-regiao')}
      aria-live="polite"
      className="fixed top-14 right-4 z-[70] flex w-[min(92vw,380px)] animate-slide-in flex-col gap-3 rounded-card border border-l-2 border-panel border-l-accent pop-surface py-3 pr-2 pl-3.5"
    >
      <div className="flex items-start gap-3">
        <span className="mt-0.5 text-accent">
          <Icon name="clip" size={14} />
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-[13px] font-medium text-fg">
            {t('importacao-downloads-titulo', { n: offers.length })}
          </p>
          <ul className="mt-1 flex flex-col gap-1">
            {offers.map((o) => (
              <li key={o.path} className="flex items-center gap-2 text-[12.5px] leading-snug">
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-fg">{o.name}</span>
                  <span className="block text-dim">
                    {o.label} · {formatBytes(o.size)}
                  </span>
                </span>
                {many ? (
                  <button
                    type="button"
                    aria-label={t('importacao-downloads-dispensar', { nome: o.name })}
                    className="grid size-6 shrink-0 place-items-center rounded-control text-dim hover:bg-surface hover:text-fg"
                    onClick={() => settle([o.path])}
                  >
                    <Icon name="x" size={11} />
                  </button>
                ) : null}
              </li>
            ))}
          </ul>
          {settings ? (
            <Checkbox
              className="mt-2 text-[12.5px]"
              label={t('importacao-downloads-apagar')}
              checked={settings.downloads_delete_after_import}
              disabled={save.isPending}
              onChange={(on) => save.mutate({ downloads_delete_after_import: on })}
            />
          ) : null}
          <InlineError error={error ?? save.error} />
          <div className="mt-2.5 flex flex-wrap gap-2">
            <Button size="sm" variant="primary" loading={busy} onClick={() => void importAll()}>
              {t('importacao-downloads-importar', { n: offers.length })}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={busy}
              onClick={() => settle(offers.map((o) => o.path))}
            >
              {t('importacao-downloads-agora-nao')}
            </Button>
          </div>
        </div>
      </div>
    </section>
  );
}
