/**
 * Pregão regular de uma bolsa no fuso dela (o mesmo cálculo de
 * `market::exchange::session_at` no núcleo), com `Intl` para o horário de verão.
 * Feriados não entram.
 */
import { t } from '@/i18n';
import type { SessionState } from '@/api/generated/SessionState';

export interface TradingHours {
  timezone: string;
  open_time: string;
  close_time: string;
  lunch_start: string | null;
  lunch_end: string | null;
  /** Numeração ISO: 1 = segunda … 7 = domingo. */
  trading_days: string;
}

const WEEKDAY: Record<string, number> = { Mon: 1, Tue: 2, Wed: 3, Thu: 4, Fri: 5, Sat: 6, Sun: 7 };

function minutes(hhmm: string): number {
  const [h, m] = hhmm.split(':').map(Number) as [number, number];
  return h * 60 + m;
}

/** Dia da semana (ISO) e minutos desde a meia-noite no fuso. */
export function localClock(timezone: string, at: Date): { weekday: number; minutes: number } {
  const parts = new Intl.DateTimeFormat('en-US', {
    timeZone: timezone,
    weekday: 'short',
    hour: '2-digit',
    minute: '2-digit',
    hourCycle: 'h23',
  }).formatToParts(at);
  const get = (t: string) => parts.find((p) => p.type === t)?.value ?? '';
  return {
    weekday: WEEKDAY[get('weekday')] ?? 1,
    minutes: Number(get('hour')) * 60 + Number(get('minute')),
  };
}

export function sessionAt(ex: TradingHours, at: Date = new Date()): SessionState {
  let clock: { weekday: number; minutes: number };
  try {
    clock = localClock(ex.timezone, at);
  } catch {
    return 'closed';
  }
  if (!ex.trading_days.includes(String(clock.weekday))) return 'closed';
  if (clock.minutes < minutes(ex.open_time)) return 'pre';
  if (clock.minutes >= minutes(ex.close_time)) return 'post';
  if (
    ex.lunch_start &&
    ex.lunch_end &&
    clock.minutes >= minutes(ex.lunch_start) &&
    clock.minutes < minutes(ex.lunch_end)
  ) {
    return 'lunch';
  }
  return 'open';
}

/** Hora local da bolsa (`14:05`). */
export function localTime(timezone: string, at: Date = new Date()): string {
  try {
    return new Intl.DateTimeFormat('pt-BR', {
      timeZone: timezone,
      hour: '2-digit',
      minute: '2-digit',
    }).format(at);
  } catch {
    return '—';
  }
}

export const SESSION_LABEL: Record<SessionState, string> = {
  get pre() {
    return t('componentes-pregao-pre');
  },
  get open() {
    return t('componentes-pregao-aberto');
  },
  get lunch() {
    return t('componentes-pregao-intervalo');
  },
  get post() {
    return t('componentes-pregao-encerrado');
  },
  get closed() {
    return t('componentes-pregao-sem');
  },
};
