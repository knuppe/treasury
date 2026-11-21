/** "O que é isso?": explicação curta de um termo do glossário, aberta sob demanda. */
import { explanation, GLOSSARY, termInSentence, type GlossaryId } from '@/content/glossary';
import { cx } from '@/design/cx';
import { useHelp } from '@/design/help';
import { t } from '@/i18n';

export function Explain({ id, className }: { id: GlossaryId; className?: string }) {
  const g = GLOSSARY[id];
  const { tone } = useHelp();
  return (
    <details className={cx('group/explain text-[12.5px] leading-snug', className)}>
      <summary className="cursor-pointer text-accent underline-offset-2 hover:underline">
        {t('perfil-o-que-e', { termo: termInSentence(g.term) })}
      </summary>
      <p className="mt-1.5 text-dim">{explanation(id, tone)}</p>
    </details>
  );
}
