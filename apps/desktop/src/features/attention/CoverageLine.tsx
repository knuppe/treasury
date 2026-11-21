/**
 * Até quando os números valem (`attention.list` → `coverage`, docs/account-freshness.md).
 * Com todas as contas do dia a dia em dia, uma linha discreta com a data; com alguma sem
 * extrato recente, um aviso em âmbar com os nomes e o atalho para importar. `onlyPartial`
 * mostra só o aviso (ao lado da previsão de saldo e do "quanto posso gastar hoje").
 */
import { Link } from 'react-router';

import type { DataCoverage } from '@/api/generated/DataCoverage';
import { useAttention } from '@/api/hooks/attention';
import { Icon } from '@/design/components';
import { cx } from '@/design/cx';
import { alertHref } from '@/features/agent/AlertsPanel';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

/** Os nomes das contas paradas: até dois, depois "e mais N". */
export function staleNames(c: DataCoverage): string {
  const names = c.stale.slice(0, 2).map((a) => a.name);
  const rest = c.stale.length - names.length;
  const list = names.join(', ');
  return rest > 0 ? t('atencao-cobertura-mais', { nomes: list, n: rest }) : list;
}

export function CoverageLine({
  onlyPartial = false,
  className,
}: {
  onlyPartial?: boolean;
  className?: string;
}) {
  const c = useAttention().data?.coverage;
  if (!c || c.status === 'empty') return null;
  if (c.status === 'complete') {
    if (onlyPartial) return null;
    return (
      <p className={cx('flex items-center gap-1.5 text-[12px] text-dim', className)}>
        <Icon name="check" size={12} className="text-positive" />
        {c.complete_until
          ? t('atencao-cobertura-completa', { data: formatDate(c.complete_until) })
          : t('atencao-cobertura-completa-sem-data')}
      </p>
    );
  }
  const href = alertHref({
    screen: 'import',
    id: null,
    account_id: c.stale[0]?.account_id ?? null,
    month: null,
    date: null,
  });
  return (
    <p
      role="status"
      className={cx(
        'flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[12px] text-warning',
        className,
      )}
    >
      <Icon name="alert" size={12} />
      <span>{t('atencao-cobertura-parcial', { n: c.stale.length, contas: staleNames(c) })}</span>
      {href ? (
        <Link to={href} className="text-accent underline-offset-2 hover:underline">
          {t('atencao-cobertura-importar')}
        </Link>
      ) : null}
    </p>
  );
}
