/** Peças comuns das figuras da tela Relatórios. */
import { useCallback, type ReactNode } from 'react';
import { useNavigate } from 'react-router';

import type { Basis } from '@/api/generated/Basis';
import type { TxnFilter } from '@/api/generated/TxnFilter';
import { Icon } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { txnLink } from '@/lib/txnLink';

import type { Period } from '../model/period';

export interface FigureProps {
  period: Period;
  /** Base escolhida no filtro (`null` = padrão do cofre). */
  basis: Basis | null;
  memberId: string | null;
  accountId: string | null;
  today: string;
}

/** Abre a tela de lançamentos com o filtro do pedaço clicado. */
export function useDrill() {
  const navigate = useNavigate();
  return useCallback((f: TxnFilter) => void navigate(txnLink(f)), [navigate]);
}

export const basisLabel = (b: Basis) =>
  b === 'cash' ? t('relatorios-base-caixa') : t('relatorios-base-competencia');

/** Medida pequena acima do gráfico (rótulo técnico + número). */
export function Stat({
  label,
  children,
  hint,
  className,
}: {
  label: ReactNode;
  children: ReactNode;
  hint?: ReactNode;
  className?: string;
}) {
  return (
    <dl className={cx('flex min-w-0 flex-col gap-1', className)}>
      <dt className="label-caps">{label}</dt>
      <dd className="num text-[18px] leading-tight font-medium text-fg">{children}</dd>
      {hint ? <dd className="text-[12px] text-dim">{hint}</dd> : null}
    </dl>
  );
}

export function Stats({ children }: { children: ReactNode }) {
  return (
    <div className="grid grid-cols-2 gap-x-6 gap-y-4 border-b border-line pb-4 @min-[530px]:grid-cols-4">
      {children}
    </div>
  );
}

/** Aviso quando titular/conta estão filtrados mas a figura não aceita esses filtros. */
export function FilterNote({ memberId, accountId }: Pick<FigureProps, 'memberId' | 'accountId'>) {
  if (!memberId && !accountId) return null;
  return (
    <p role="note" className="flex items-center gap-2 text-[12px] text-dim">
      <Icon name="info" size={13} className="shrink-0" />
      {memberId && accountId
        ? t('relatorios-filtro-nota-membro-conta')
        : memberId
          ? t('relatorios-filtro-nota-membro')
          : t('relatorios-filtro-nota-conta')}
    </p>
  );
}

/** Pergunta que a figura responde (uma linha, antes do gráfico). */
export function Question({ children }: { children: ReactNode }) {
  return <p className="text-[13.5px] text-dim">{children}</p>;
}
