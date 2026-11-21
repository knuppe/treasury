/**
 * O agente na página do ativo (docs/asset-page.md): "Perguntar ao agente" abre o painel ao lado
 * com uma conversa nova (o contexto da tela diz que é a página do ativo, e a conversa fica ligada a
 * ele) e a aba "Conversas" lista o que já foi analisado ali, junto do atalho para os Estudos com IA.
 */
import { useAssetConversations } from '@/api/hooks/ai';
import { Button, Icon } from '@/design/components';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';
import { useCompanion } from '@/state/companion';

import { ErrorBlock, LoadingBlock, Notice } from '../portfolio/components/States';

interface AssetRef {
  id?: string | null;
  ticker: string | null;
  name: string;
}

const label = (a: AssetRef) => a.ticker ?? a.name;

/** Conversa nova com o agente sobre o ativo, no painel lateral (o envio fica com a pessoa). */
export function askAgentAbout(a: AssetRef) {
  const c = useCompanion.getState();
  c.setShareContext(true);
  c.setConversation(null);
  c.ask(t('ativo-agente-pergunta', { ativo: label(a) }));
}

export function AskAgentButton({ asset }: { asset: AssetRef }) {
  return (
    <Button
      variant="secondary"
      icon={<Icon name="agent" size={12} />}
      onClick={() => askAgentAbout(asset)}
    >
      {t('ativo-perguntar-agente')}
    </Button>
  );
}

export function AssetConversations({
  asset,
  onStudies,
}: {
  asset: AssetRef & { id: string };
  onStudies: () => void;
}) {
  const q = useAssetConversations(asset.id);
  const open = (id: string) => {
    const c = useCompanion.getState();
    c.setShareContext(true);
    c.setConversation(id);
    c.setOpen(true);
  };
  return (
    <section aria-label={t('ativo-aba-conversas')} className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-2">
        <p className="flex-1 text-[13px] text-dim">{t('ativo-conversas-texto')}</p>
        <Button variant="ghost" onClick={onStudies}>
          {t('ativo-ver-estudos')}
        </Button>
        <AskAgentButton asset={asset} />
      </div>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : !q.data?.length ? (
        <Notice title={t('ativo-conversas-vazio')}>
          {t('ativo-conversas-vazio-texto', { ativo: label(asset) })}
        </Notice>
      ) : (
        <ul className="divide-y divide-line rounded-control border border-line">
          {q.data.map((c) => (
            <li key={c.id} className="flex flex-wrap items-center gap-2 px-3 py-2">
              <span className="min-w-0 flex-1">
                <span className="block truncate text-[13px] text-fg">{c.title}</span>
                <span className="block text-[11.5px] text-dim">
                  {t('ativo-conversa-detalhe', {
                    quando: formatDateTime(c.updated_at),
                    n: c.message_count,
                  })}
                </span>
              </span>
              <Button size="sm" variant="ghost" onClick={() => open(c.id)}>
                {t('ativo-abrir-conversa')}
              </Button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
