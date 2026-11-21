/** Rótulos da apuração de IR (`tax.*`), pelo catálogo `ir`. */
import type { DarfStatus } from '@/api/generated/DarfStatus';
import type { LossPool } from '@/api/generated/LossPool';
import type { MonthStatus } from '@/api/generated/MonthStatus';
import { t } from '@/i18n';

export const MONTH_STATUS_LABEL: Record<MonthStatus, string> = {
  get no_activity() {
    return t('ir-mes-sem-vendas');
  },
  get no_tax() {
    return t('ir-mes-sem-imposto');
  },
  get below_minimum() {
    return t('ir-mes-abaixo-minimo');
  },
  get to_pay() {
    return t('ir-mes-a-pagar');
  },
  get paid() {
    return t('ir-mes-pago');
  },
};

export const MONTH_STATUS_HINT: Record<MonthStatus, string> = {
  get no_activity() {
    return t('ir-mes-sem-vendas-dica');
  },
  get no_tax() {
    return t('ir-mes-sem-imposto-dica');
  },
  get below_minimum() {
    return t('ir-mes-abaixo-minimo-dica');
  },
  get to_pay() {
    return t('ir-mes-a-pagar-dica');
  },
  get paid() {
    return t('ir-mes-pago-dica');
  },
};

export const POOL_LABEL: Record<LossPool, string> = {
  get common() {
    return t('ir-grupo-comum');
  },
  get day_trade() {
    return t('ir-grupo-day-trade');
  },
  get fii() {
    return t('ir-grupo-fii');
  },
};

export const POOL_SHORT: Record<LossPool, string> = {
  get common() {
    return t('ir-grupo-comum-curto');
  },
  get day_trade() {
    return t('ir-grupo-day-trade');
  },
  get fii() {
    return t('ir-grupo-fii-curto');
  },
};

export const DARF_STATUS_LABEL: Record<DarfStatus, string> = {
  get pending() {
    return t('ir-darf-em-aberto');
  },
  get paid() {
    return t('ir-mes-pago');
  },
  get cancelled() {
    return t('ir-darf-cancelado');
  },
};

/** Aviso padrão (o núcleo manda o dele junto com a apuração). */
export function disclaimer(): string {
  return t('ir-aviso');
}

/** Período de apuração `2026-10` → `10/2026`, como no DARF. */
export function periodLabel(month: string): string {
  return `${month.slice(5, 7)}/${month.slice(0, 4)}`;
}
