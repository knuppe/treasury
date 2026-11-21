/**
 * Ícones `?` ao lado dos termos técnicos (docs/profile.md): para quem disse no questionário
 * que está começando, cada rótulo que casa com um termo do glossário ganha um `?` que, ao
 * passar o mouse (ou focar com o Tab), explica o termo no tom escolhido no perfil.
 *
 * O nível e o tom chegam por contexto (`HelpProvider`, montado no AppShell com o perfil
 * de quem está em vista); sem provedor, nenhum `?` aparece. Os componentes do design
 * (Measure, InstrumentCard, KeyValue, Field, cabeçalho das tabelas) usam `AutoHelp` com o
 * próprio rótulo, então as telas não precisam marcar termo por termo.
 */
import { createContext, useContext, type ReactNode } from 'react';

import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import type { ProfileTone } from '@/api/generated/ProfileTone';
import {
  explanation,
  findTerm,
  GLOSSARY,
  showsAt,
  termInSentence,
  type GlossaryId,
} from '@/content/glossary';
import { t } from '@/i18n';

import { Tooltip } from './components/Tooltip';
import { cx } from './cx';

export interface HelpSettings {
  /** Nível em que os `?` são decididos (`avancado` = nenhum). */
  level: ProfileLevel;
  tone: ProfileTone;
  /** A família mora no Brasil (fora, os termos só de lá ficam sem `?`). Padrão: sim. */
  brazil?: boolean;
}

const HelpContext = createContext<HelpSettings>({ level: 'avancado', tone: 'equilibrado' });

export function HelpProvider({ value, children }: { value: HelpSettings; children: ReactNode }) {
  return <HelpContext.Provider value={value}>{children}</HelpContext.Provider>;
}

export const useHelp = () => useContext(HelpContext);

/** `?` com a explicação do termo, se ele vale no nível atual. */
export function HelpTip({ term, className }: { term: GlossaryId; className?: string }) {
  const { level, tone } = useHelp();
  if (!showsAt(term, level)) return null;
  const g = GLOSSARY[term];
  return (
    <Tooltip
      side="top"
      className="max-w-[300px] normal-case"
      content={
        <span className="flex flex-col gap-1 text-left tracking-normal normal-case">
          <span className="font-medium text-fg">{g.term}</span>
          <span className="leading-snug text-dim">{explanation(term, tone)}</span>
        </span>
      }
    >
      <button
        type="button"
        aria-label={t('componentes-o-que-e', { termo: termInSentence(g.term) })}
        data-help={term}
        // Dentro de botões de ordenação ou de cards clicáveis, o `?` não dispara a ação.
        onClick={(e) => e.stopPropagation()}
        className={cx(
          'inline-flex size-[15px] shrink-0 cursor-help items-center justify-center rounded-full border border-line-strong font-sans text-[10px] leading-none font-semibold tracking-normal text-dim normal-case transition-colors hover:border-accent hover:text-accent focus-visible:border-accent focus-visible:text-accent',
          className,
        )}
      >
        ?
      </button>
    </Tooltip>
  );
}

/** `?` do termo que o rótulo menciona (só para rótulos de texto). */
export function AutoHelp({ label, className }: { label: ReactNode; className?: string }) {
  const { level, brazil } = useHelp();
  if (level === 'avancado' || typeof label !== 'string') return null;
  const term = findTerm(label, brazil !== false);
  return term ? <HelpTip term={term} className={className} /> : null;
}
