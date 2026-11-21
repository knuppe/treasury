/**
 * Regras sugeridas (`rules.learned`, docs/learned-rules.md): estabelecimentos a que a
 * pessoa já deu a mesma categoria três vezes. "Criar regra" grava a regra e a aplica aos
 * lançamentos ainda sem categoria; "Não sugerir" tira a sugestão de vez. Nada é criado sozinho.
 */
import type { LearnedRule } from '@/api/generated/LearnedRule';
import {
  useApplyRules,
  useCreateRule,
  useIgnoreLearnedRule,
  useLearnedRules,
} from '@/api/hooks/rules';
import { Button, Icon, toast } from '@/design/components';
import { errorText } from '@/features/transactions/ui/errors';
import { t } from '@/i18n';

function Row({ item }: { item: LearnedRule }) {
  const create = useCreateRule();
  const apply = useApplyRules();
  const ignore = useIgnoreLearnedRule();
  const busy = create.isPending || apply.isPending || ignore.isPending;
  const accept = async () => {
    try {
      const rule = await create.mutateAsync(item.rule);
      // Os que já estão sem categoria recebem a regra agora (com desfazer no histórico).
      const applied =
        item.uncategorized > 0 ? (await apply.mutateAsync({ rule_ids: [rule.id] })).updated : 0;
      toast({
        tone: 'success',
        title: t('regras-sugeridas-criada'),
        description: t('regras-sugeridas-aplicada', { n: applied }),
      });
    } catch (e) {
      toast({ tone: 'error', title: t('regras-sugeridas-erro'), description: errorText(e) });
    }
  };
  return (
    <li className="flex flex-wrap items-center gap-x-4 gap-y-1.5 border-b border-line/60 py-2 last:border-b-0">
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="flex flex-wrap items-center gap-x-2 text-[13px] text-fg">
          <span className="font-medium uppercase">{item.key}</span>
          <Icon name="arrowRight" size={12} className="text-dim" />
          <span>{item.category_path}</span>
        </span>
        <span className="text-[12px] text-dim">
          {t('regras-sugeridas-ja', { n: item.count })}
          {item.uncategorized > 0
            ? ` · ${t('regras-sugeridas-faltam', { n: item.uncategorized })}`
            : ''}
        </span>
      </div>
      <div className="flex shrink-0 gap-1.5">
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void accept()}>
          {t('regras-sugeridas-criar')}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={busy}
          onClick={() => ignore.mutate({ key: item.key })}
        >
          {t('regras-sugeridas-ignorar')}
        </Button>
      </div>
    </li>
  );
}

export function LearnedRules() {
  const { data = [] } = useLearnedRules();
  if (data.length === 0) return null;
  return (
    <section
      aria-label={t('regras-sugeridas-titulo')}
      className="flex flex-col gap-1 rounded-card border border-line bg-surface px-4 py-3.5"
    >
      <h2 className="text-[13px] text-dim">{t('regras-sugeridas-titulo')}</h2>
      <p className="text-[12.5px] text-dim">{t('regras-sugeridas-explica')}</p>
      <ul className="flex flex-col">
        {data.map((item) => (
          <Row key={item.key} item={item} />
        ))}
      </ul>
    </section>
  );
}
