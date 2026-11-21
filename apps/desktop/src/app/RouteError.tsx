import { isRouteErrorResponse, useRouteError } from 'react-router';

import { Button, Icon, InstrumentCard } from '@/design/components';
import { t } from '@/i18n';

/** Erro inesperado ao renderizar uma rota: mostra o que houve e oferece recarregar. */
export function RouteError() {
  const error = useRouteError();
  const message = isRouteErrorResponse(error)
    ? `${error.status} ${error.statusText}`
    : error instanceof Error
      ? error.message
      : t('casca-erro-desconhecido');
  return (
    <div className="flex min-h-screen items-center justify-center overflow-y-auto bg-canvas px-6">
      <InstrumentCard index="!" title={t('casca-falha-titulo')} className="w-full max-w-[520px]">
        <div role="alert" className="flex flex-col gap-4">
          <p className="text-[14px] leading-relaxed text-fg">{t('casca-falha-texto')}</p>
          <pre
            data-selectable
            className="max-h-40 overflow-auto rounded-control border border-line bg-bg p-3 text-[12px] whitespace-pre-wrap text-dim"
          >
            {message}
          </pre>
          <div className="flex gap-2">
            <Button
              variant="primary"
              icon={<Icon name="refresh" size={14} />}
              onClick={() => window.location.reload()}
            >
              {t('casca-recarregar')}
            </Button>
            <Button variant="ghost" onClick={() => (window.location.hash = '#/painel')}>
              {t('casca-ir-painel')}
            </Button>
          </div>
        </div>
      </InstrumentCard>
    </div>
  );
}
