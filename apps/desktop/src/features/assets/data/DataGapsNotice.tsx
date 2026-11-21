/**
 * Aviso curto de que há dados dos ativos para completar (sugestões das fontes oficiais),
 * com o atalho para Carteira › Saúde dos ativos › Dados dos ativos. Some quando não há
 * nada a fazer ou quando o usuário fecha (até haver mais campos do que quando fechou).
 */
import { useState } from 'react';
import { Link } from 'react-router';

import { useDataGaps } from '@/api/hooks/assetData';
import { Icon } from '@/design/components';
import { t } from '@/i18n';

const KEY = 'treasury.dataGapsNotice.dismissed';

function dismissedAt(): number {
  try {
    return Number(localStorage.getItem(KEY) ?? 0) || 0;
  } catch {
    return 0;
  }
}

export function DataGapsNotice() {
  const q = useDataGaps();
  const [closed, setClosed] = useState(dismissedAt);
  const s = q.data?.summary;
  const pending = s ? s.fill + s.conflicts + s.updates : 0;
  if (!s || pending === 0 || pending <= closed) return null;
  const close = () => {
    try {
      localStorage.setItem(KEY, String(pending));
    } catch {
      // Sem armazenamento: fecha só nesta sessão.
    }
    setClosed(pending);
  };
  return (
    <div
      role="note"
      className="flex flex-wrap items-center gap-x-3 gap-y-1 rounded-sm border border-accent/40 bg-accent/5 px-3 py-2 text-[12.5px]"
    >
      <Icon name="info" size={13} className="text-accent" />
      <span className="grow text-fg">
        {t('dados-aviso', { n: pending, sem_setor: s.missing_sector })}
      </span>
      <Link
        to="/carteira?aba=saude&secao=dados"
        className="font-mono text-[11.5px] tracking-[0.08em] text-accent uppercase hover:underline"
      >
        {t('dados-aviso-ver')}
      </Link>
      <button
        type="button"
        onClick={close}
        aria-label={t('dados-aviso-fechar')}
        className="text-dim hover:text-fg"
      >
        <Icon name="x" size={12} />
      </button>
    </div>
  );
}
