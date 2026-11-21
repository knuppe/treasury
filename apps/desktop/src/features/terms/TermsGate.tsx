/**
 * "Antes de começar" (docs/terms.md): aparece antes de qualquer cofre (inclusive o de
 * demonstração) até esta instalação aceitar a versão vigente dos termos, e de novo quando o
 * MAIOR muda, com o "O que mudou" no topo. Sem caixa pré-marcada; "Continuar" fica desabilitado
 * até marcar; "Sair" fecha o app. O aceite (versão e data) fica fora do cofre, e nada é enviado.
 */
import { useState, type ReactNode } from 'react';

import { isTauri } from '@/api/client';
import { useAcceptTerms, useTermsStatus } from '@/api/hooks/terms';
import { openExternal } from '@/app/externalLinks';
import { Brand } from '@/app/Brand';
import { Button, Dialog, Markdown } from '@/design/components';
import { InlineError } from '@/features/transactions/ui/controls';
import { t, useLocale } from '@/i18n';

import { AGPL_URL, POINTS, SIGNATURE, fullTerms } from './texts';

async function quitApp() {
  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('quit_app');
  } else {
    window.close();
  }
}

/** Mostra a tela de aceite quando preciso; senão, o resto do app. */
export function TermsGate({ children }: { children: ReactNode }) {
  const status = useTermsStatus();
  if (status.isPending) return null;
  // Sem resposta do núcleo, segue (a porta do cofre mostra o erro dela).
  if (!status.data?.needs_acceptance)
    return (
      <>
        {children}
        {status.data?.updated ? <UpdatedNotice current={status.data.current} /> : null}
      </>
    );
  return <FirstUse />;
}

/** Mudança de MENOR: um aviso discreto, sem bloquear; "Entendi" registra a versão nova. */
function UpdatedNotice({ current }: { current: string }) {
  const accept = useAcceptTerms();
  const [full, setFull] = useState(false);
  const locale = useLocale();
  return (
    <div
      role="status"
      className="fixed bottom-4 left-1/2 z-[60] flex -translate-x-1/2 flex-wrap items-center gap-3 rounded-card border border-line pop-surface px-4 py-2 text-[13px] text-fg"
    >
      <span>{t('termos-atualizados', { versao: current })}</span>
      <Button size="sm" variant="ghost" onClick={() => setFull(true)}>
        {t('termos-ler')}
      </Button>
      <Button
        size="sm"
        variant="secondary"
        loading={accept.isPending}
        onClick={() => accept.mutate({ version: current })}
      >
        {t('termos-entendi')}
      </Button>
      <Dialog open={full} onOpenChange={setFull} size="lg" resizable title={t('termos-de-uso')}>
        <Markdown text={fullTerms(locale)} />
      </Dialog>
    </div>
  );
}

function FirstUse() {
  const status = useTermsStatus().data;
  const accept = useAcceptTerms();
  const [checked, setChecked] = useState(false);
  const [full, setFull] = useState(false);
  const locale = useLocale();
  const bold = (s: string) =>
    s.split('**').map((p, i) => (i % 2 ? <strong key={i}>{p}</strong> : p));
  return (
    <div className="flex min-h-screen items-start justify-center overflow-y-auto bg-canvas px-6 py-10">
      <main
        aria-labelledby="terms-title"
        className="flex w-full max-w-[680px] flex-col gap-4 rounded-card border border-panel bg-surface px-7 py-6"
      >
        <Brand />
        {status?.changes.length ? (
          <section
            aria-label={t('termos-o-que-mudou')}
            className="rounded-card border border-accent/50 bg-accent-soft px-3 py-2 text-[13px] text-fg"
          >
            <h2 className="label-caps mb-1 text-accent">{t('termos-o-que-mudou')}</h2>
            <ul className="flex flex-col gap-0.5">
              {status.changes.map((c) => (
                <li key={c.version}>
                  {c.version}: {c.summary}
                </li>
              ))}
            </ul>
          </section>
        ) : null}
        <h1 id="terms-title" className="text-[22px] font-medium text-fg">
          {t('termos-titulo')}
        </h1>
        <p className="text-[14px] leading-relaxed text-fg">{bold(t('termos-intro'))}</p>
        <ul className="flex list-disc flex-col gap-2 pl-5 text-[13.5px] leading-relaxed text-fg">
          {POINTS.map(([lead, rest]) => (
            <li key={lead}>
              <strong>{t(lead)}</strong> {t(rest)}
            </li>
          ))}
        </ul>
        <p className="flex flex-wrap gap-x-3 text-[13px]">
          <button
            type="button"
            className="text-accent underline-offset-2 hover:underline"
            onClick={() => setFull(true)}
          >
            {t('termos-ler-completos')}
          </button>
          <span aria-hidden className="text-dim">
            ·
          </span>
          <button
            type="button"
            className="text-accent underline-offset-2 hover:underline"
            onClick={() => void openExternal(AGPL_URL)}
          >
            {t('termos-ler-licenca')}
          </button>
        </p>
        <label className="flex cursor-pointer items-start gap-2.5 rounded-card border border-line px-3 py-2.5 text-[13.5px] text-fg">
          <input
            type="checkbox"
            className="mt-1 size-4 shrink-0"
            checked={checked}
            onChange={(e) => setChecked(e.target.checked)}
          />
          <strong>{t('termos-marcar')}</strong>
        </label>
        <div className="flex flex-wrap items-center justify-end gap-2">
          <Button variant="ghost" onClick={() => void quitApp()}>
            {t('termos-sair')}
          </Button>
          <Button
            variant="primary"
            disabled={!checked || !status}
            loading={accept.isPending}
            onClick={() => status && accept.mutate({ version: status.current })}
          >
            {t('comum-continuar')}
          </Button>
        </div>
        <InlineError error={accept.error} />
        <p className="text-right text-[12.5px] text-dim italic">— {SIGNATURE}</p>
      </main>
      <Dialog
        open={full}
        onOpenChange={setFull}
        size="lg"
        resizable
        title={t('termos-ler-completos')}
      >
        <Markdown text={fullTerms(locale)} />
      </Dialog>
    </div>
  );
}
