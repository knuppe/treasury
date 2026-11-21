/**
 * Sino da barra superior: quantos itens estão em "Para resolver". Fica em âmbar quando há algo
 * urgente ou de atenção. O clique abre o Painel na lista (uma lista só, sem painel flutuante).
 */
import { useNavigate } from 'react-router';

import { useAttention } from '@/api/hooks/attention';
import { Icon, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

export function AttentionBell() {
  const navigate = useNavigate();
  const counts = useAttention().data?.counts;
  const total = counts?.total ?? 0;
  const urgent = (counts?.urgent ?? 0) > 0;
  const label = t('atencao-sino', { n: total });
  return (
    <Tooltip content={label}>
      <button
        type="button"
        aria-label={label}
        onClick={() => void navigate('/painel?resolver=1')}
        className={cx(
          'relative inline-flex size-8 items-center justify-center rounded-control transition-colors duration-150 hover:bg-surface-2 hover:text-fg',
          total === 0 ? 'text-dim' : urgent ? 'text-warning' : 'text-accent',
        )}
      >
        <Icon name="bell" />
        {total > 0 ? (
          <span
            aria-hidden
            className={cx(
              'num absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[10px] leading-none',
              urgent ? 'bg-warning text-bg' : 'bg-accent text-accent-ink',
            )}
          >
            {total > 99 ? '99+' : total}
          </span>
        ) : null}
      </button>
    </Tooltip>
  );
}
