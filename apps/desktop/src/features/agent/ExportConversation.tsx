/**
 * Exportar a conversa aberta em Markdown ou PDF (`ai.conversations.export`): pergunta onde
 * salvar (diálogo do sistema) e o núcleo grava o arquivo, já sem chaves, senhas e outros
 * segredos. No navegador (modo de demonstração) não há diálogo: vai para Documentos.
 */
import { isTauri } from '@/api/client';
import type { ConversationExportFormat } from '@/api/generated/ConversationExportFormat';
import { useExportConversation } from '@/api/hooks/ai';
import { Button, Icon, toast } from '@/design/components';
import { t } from '@/i18n';
import { chooseSavePath } from '@/lib/saveDialog';

/** Nome de arquivo a partir do título da conversa (sem barras nem pontos soltos). */
export function exportFileName(title: string, ext: 'md' | 'pdf'): string {
  const base =
    title
      .normalize('NFD')
      .replace(/\p{M}/gu, '')
      .replace(/[^A-Za-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
      .slice(0, 60) || 'conversa';
  return `${base}.${ext}`;
}

export function ExportConversation({ id, title }: { id: string; title: string }) {
  const exp = useExportConversation();
  const run = async (format: ConversationExportFormat) => {
    const ext = format === 'pdf' ? 'pdf' : 'md';
    const fileName = exportFileName(title, ext);
    try {
      const path = isTauri()
        ? await chooseSavePath(t('agente-exportar-salvar'), fileName, [
            format === 'pdf'
              ? { name: 'PDF', extensions: ['pdf'] }
              : { name: 'Markdown', extensions: ['md'] },
          ])
        : `~/Documents/${fileName}`;
      if (!path) return;
      const r = await exp.mutateAsync({ id, format, path });
      toast({
        tone: 'success',
        title: t('agente-exportar-feito'),
        description:
          r.redacted > 0
            ? t('agente-exportar-feito-segredos', { caminho: r.path, n: r.redacted })
            : t('agente-exportar-feito-texto', { caminho: r.path }),
      });
    } catch (e) {
      toast({
        tone: 'error',
        title: t('agente-exportar-erro'),
        description: e instanceof Error ? e.message : String(e),
      });
    }
  };
  return (
    <span className="flex items-center gap-1" role="group" aria-label={t('agente-exportar')}>
      <span className="label-caps text-dim">{t('agente-exportar')}</span>
      <Button
        size="sm"
        variant="ghost"
        loading={exp.isPending && exp.variables?.format === 'markdown'}
        icon={<Icon name="arrowDown" size={12} />}
        title={t('agente-exportar-md-dica')}
        onClick={() => void run('markdown')}
      >
        {t('agente-exportar-md')}
      </Button>
      <Button
        size="sm"
        variant="ghost"
        loading={exp.isPending && exp.variables?.format === 'pdf'}
        icon={<Icon name="arrowDown" size={12} />}
        title={t('agente-exportar-pdf-dica')}
        onClick={() => void run('pdf')}
      >
        PDF
      </Button>
    </span>
  );
}
