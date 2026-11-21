import { useEffect, useEffectEvent, useRef, useState } from 'react';

import { touchVault } from '@/api/vaultQueries';
import { LiveIndicator } from '@/design/components';
import { t } from '@/i18n';

/** Eventos que contam como atividade do usuário. */
const ACTIVITY_EVENTS = ['pointerdown', 'pointermove', 'keydown', 'wheel', 'touchstart'] as const;

/** `vault.touch` no máximo uma vez por minuto. */
export const TOUCH_INTERVAL_MS = 60_000;
/** Aviso de bloqueio iminente: últimos 60 s. */
export const WARNING_MS = 60_000;

export interface AutoLockOptions {
  /** Minutos de inatividade até bloquear (0 ou menos: desligado). */
  minutes: number;
  onLock: () => void;
  /** Chamado com atividade, no máximo 1×/min (padrão: `vault.touch`). */
  onTouch?: () => void;
}

/**
 * Bloqueio automático por inatividade, no cliente. Atividade (mouse, teclado,
 * rolagem, toque) reinicia a contagem e avisa o núcleo (`vault.touch`) no
 * máximo uma vez por minuto, para ele adiar o próprio bloqueio. Devolve os
 * segundos restantes quando falta menos de um minuto (senão `null`).
 */
export function useAutoLock({ minutes, onLock, onTouch = touchVault }: AutoLockOptions) {
  const [secondsLeft, setSecondsLeft] = useState<number | null>(null);
  const lastActivity = useRef(0);
  const lastTouch = useRef(0);
  const locked = useRef(false);

  const lock = useEffectEvent(onLock);
  const touch = useEffectEvent(onTouch);

  useEffect(() => {
    if (!(minutes > 0)) return;
    const limit = minutes * 60_000;
    const now = Date.now();
    lastActivity.current = now;
    lastTouch.current = now;
    locked.current = false;

    const check = () => {
      if (locked.current) return;
      const left = limit - (Date.now() - lastActivity.current);
      if (left <= 0) {
        locked.current = true;
        setSecondsLeft(null);
        lock();
        return;
      }
      setSecondsLeft(left <= WARNING_MS ? Math.ceil(left / 1000) : null);
    };

    const onActivity = () => {
      if (locked.current) return;
      const at = Date.now();
      // Ao voltar de uma suspensão, o tempo já pode ter passado: confere antes.
      if (at - lastActivity.current >= limit) {
        check();
        return;
      }
      lastActivity.current = at;
      setSecondsLeft(null);
      if (at - lastTouch.current >= TOUCH_INTERVAL_MS) {
        lastTouch.current = at;
        touch();
      }
    };

    const interval = window.setInterval(check, 1000);
    for (const name of ACTIVITY_EVENTS) {
      window.addEventListener(name, onActivity, { capture: true, passive: true });
    }
    document.addEventListener('visibilitychange', check);
    return () => {
      window.clearInterval(interval);
      for (const name of ACTIVITY_EVENTS) {
        window.removeEventListener(name, onActivity, { capture: true });
      }
      document.removeEventListener('visibilitychange', check);
    };
  }, [minutes]);

  return { secondsLeft };
}

function formatCountdown(seconds: number): string {
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
}

/** Faixa de aviso do bloqueio automático (aparece no último minuto). */
export function AutoLock(props: AutoLockOptions) {
  const { secondsLeft } = useAutoLock(props);
  const warning = secondsLeft !== null;
  return (
    <div role="status" aria-live="polite" className={warning ? undefined : 'sr-only'}>
      {warning ? (
        <div className="flex flex-wrap items-center gap-x-4 gap-y-1 border-b border-warning/40 bg-warning/10 px-8 py-2 text-[13px] text-fg">
          <LiveIndicator label={t('casca-autobloqueio')} tone="warning" />
          <span>
            {t('casca-autobloqueio-em')}{' '}
            <span aria-hidden className="num font-medium text-warning">
              {formatCountdown(secondsLeft)}
            </span>
            <span className="sr-only">{t('casca-autobloqueio-menos-minuto')}</span>
            {t('casca-autobloqueio-continuar')}
          </span>
        </div>
      ) : null}
    </div>
  );
}
