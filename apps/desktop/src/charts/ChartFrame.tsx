/**
 * Moldura de figura com alternância gráfico ↔ tabela (a visão em tabela é obrigatória
 * para leitores de tela, impressão e temas de baixo contraste).
 */
import { useState, type ReactNode } from 'react';

import { Figure } from '@/design/components';
import { t } from '@/i18n';

export interface ChartFrameProps {
  /** Número da figura: `04` → `FIG. 04`. */
  index: string;
  title: string;
  /** Linha de origem: "Fonte: lançamentos efetivados · competência". */
  source?: ReactNode;
  actions?: ReactNode;
  chart: ReactNode;
  table: ReactNode;
}

export function ChartFrame({ index, title, source, actions, chart, table }: ChartFrameProps) {
  const [mode, setMode] = useState<'chart' | 'table'>('chart');
  return (
    <Figure
      number={index}
      title={title}
      source={source}
      legend={
        <span className="flex items-center gap-2">
          {actions}
          <button
            type="button"
            className="label-caps rounded-sm px-1 hover:text-text focus-visible:outline focus-visible:outline-accent"
            aria-pressed={mode === 'table'}
            onClick={() => setMode((m) => (m === 'chart' ? 'table' : 'chart'))}
          >
            {mode === 'chart' ? t('graficos-ver-tabela') : t('graficos-ver-grafico')}
          </button>
        </span>
      }
    >
      {mode === 'chart' ? chart : <div className="max-h-[360px] overflow-auto">{table}</div>}
    </Figure>
  );
}
