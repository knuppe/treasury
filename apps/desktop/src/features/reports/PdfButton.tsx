/**
 * Botão "Baixar PDF do mês" da tela Relatórios: pergunta onde salvar (diálogo do
 * sistema) e pede ao núcleo o relatório do mês em PDF, montado no próprio computador. No
 * navegador (modo de demonstração) não há diálogo: o arquivo vai para a pasta Documentos.
 */
import type { Basis } from '@/api/generated/Basis';
import { isTauri } from '@/api/client';
import { useReportPdf } from '@/api/hooks/reports';
import { Button, Icon, toast } from '@/design/components';
import { t } from '@/i18n';
import { formatMonthLong } from '@/lib/format';
import { chooseSavePath } from '@/lib/saveDialog';

async function choosePath(fileName: string): Promise<string | null> {
  if (!isTauri()) return `~/Documents/${fileName}`;
  return chooseSavePath(t('pdf-salvar-titulo'), fileName, [{ name: 'PDF', extensions: ['pdf'] }]);
}

export function PdfButton({ month, basis }: { month: string; basis: Basis | null }) {
  const pdf = useReportPdf();
  const run = async () => {
    try {
      const path = await choosePath(t('pdf-arquivo', { mes: month }));
      if (!path) return;
      const r = await pdf.mutateAsync({ month, basis: basis ?? undefined, path });
      toast({
        tone: 'success',
        title: t('pdf-feito'),
        description: t('pdf-feito-paginas', { paginas: r.pages, caminho: r.path }),
      });
    } catch (e) {
      toast({
        tone: 'error',
        title: t('pdf-erro'),
        description: e instanceof Error ? e.message : String(e),
      });
    }
  };
  return (
    <Button
      variant="secondary"
      loading={pdf.isPending}
      title={t('pdf-botao-dica', { mes: formatMonthLong(month) })}
      icon={<Icon name="arrowDown" size={14} />}
      onClick={() => void run()}
    >
      {t('pdf-botao')}
    </Button>
  );
}
